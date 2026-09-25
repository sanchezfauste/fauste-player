//! The background analysis pool (spec §2.2, item 4): a few worker threads
//! analyse submitted files; results come back on a channel.
//!
//! Every submission gets a generation number. A job only runs, and its
//! result is only delivered, while it is still the latest submission of its
//! track: this one rule handles cancellation, duplicates and resubmission.
//! Running jobs see cancellation (and shutdown) between decoded blocks. A
//! panic anywhere in a job is contained and reported as unreadable.

use std::collections::HashMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread::JoinHandle;

use crossbeam_channel::{Receiver, Sender};
use fp_model::{AnalysisSettings, Limits, TrackId};

use crate::analyze::{Analysis, AnalysisError, analyze_file_cancellable};
use crate::cache::AnalysisCache;

/// The analysis step; injectable for tests. The last argument reports
/// whether the job has been cancelled.
pub type AnalyzeFn = Arc<
    dyn Fn(&Path, &AnalysisSettings, &Limits, &dyn Fn() -> bool) -> Result<Analysis, AnalysisError>
        + Send
        + Sync,
>;

#[derive(Debug)]
pub struct AnalysisResult {
    pub track: TrackId,
    pub path: PathBuf,
    pub outcome: Result<Analysis, AnalysisError>,
}

struct Job {
    track: TrackId,
    path: PathBuf,
    generation: u64,
}

#[derive(Default)]
struct Bookkeeping {
    /// Latest pending generation per track; absent means nothing pending.
    latest: HashMap<TrackId, u64>,
    next_generation: u64,
}

impl Bookkeeping {
    fn is_current(&self, job: &Job) -> bool {
        self.latest.get(&job.track) == Some(&job.generation)
    }
}

struct Shared {
    book: Mutex<Bookkeeping>,
    settings: Mutex<AnalysisSettings>,
    stop: AtomicBool,
    limits: Limits,
    cache: Option<AnalysisCache>,
    analyze: AnalyzeFn,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

pub struct Analyzer {
    jobs: Option<Sender<Job>>,
    results: Receiver<AnalysisResult>,
    shared: Arc<Shared>,
    threads: Vec<JoinHandle<()>>,
}

impl Analyzer {
    /// A pool of `threads` workers running the real analysis.
    pub fn spawn(
        threads: usize,
        settings: AnalysisSettings,
        limits: Limits,
        cache: Option<AnalysisCache>,
    ) -> std::io::Result<Self> {
        Self::with_analyze_fn(
            threads,
            settings,
            limits,
            cache,
            Arc::new(analyze_file_cancellable),
        )
    }

    pub fn with_analyze_fn(
        threads: usize,
        settings: AnalysisSettings,
        limits: Limits,
        cache: Option<AnalysisCache>,
        analyze: AnalyzeFn,
    ) -> std::io::Result<Self> {
        let (job_tx, job_rx) = crossbeam_channel::unbounded::<Job>();
        let (res_tx, res_rx) = crossbeam_channel::unbounded();
        let shared = Arc::new(Shared {
            book: Mutex::new(Bookkeeping::default()),
            settings: Mutex::new(settings),
            stop: AtomicBool::new(false),
            limits,
            cache,
            analyze,
        });
        let mut handles = Vec::new();
        for n in 0..threads.max(1) {
            let (rx, tx, shared) = (job_rx.clone(), res_tx.clone(), shared.clone());
            handles.push(
                std::thread::Builder::new()
                    .name(format!("fp-analysis-{n}"))
                    .spawn(move || worker(&rx, &tx, &shared))?,
            );
        }
        Ok(Self {
            jobs: Some(job_tx),
            results: res_rx,
            shared,
            threads: handles,
        })
    }

    /// Queues `path` for analysis as `track` (ignored if one is pending).
    pub fn submit(&self, track: TrackId, path: PathBuf) {
        let generation = {
            let mut book = lock(&self.shared.book);
            if book.latest.contains_key(&track) {
                return;
            }
            book.next_generation += 1;
            let generation = book.next_generation;
            book.latest.insert(track, generation);
            generation
        };
        if let Some(jobs) = &self.jobs {
            let _ = jobs.send(Job {
                track,
                path,
                generation,
            });
        }
    }

    /// Cancels a pending or running job; no result will be delivered for it.
    pub fn cancel(&self, track: TrackId) {
        lock(&self.shared.book).latest.remove(&track);
    }

    /// Settings for jobs that start after this call.
    pub fn update_settings(&self, settings: AnalysisSettings) {
        *lock(&self.shared.settings) = settings;
    }

    pub fn results(&self) -> &Receiver<AnalysisResult> {
        &self.results
    }
}

impl Drop for Analyzer {
    fn drop(&mut self) {
        // Stop at once: queued jobs are skipped and running ones cancelled.
        self.shared.stop.store(true, Ordering::Release);
        self.jobs = None;
        for t in self.threads.drain(..) {
            let _ = t.join();
        }
    }
}

fn worker(jobs: &Receiver<Job>, results: &Sender<AnalysisResult>, shared: &Shared) {
    for job in jobs {
        if shared.stop.load(Ordering::Acquire) {
            return;
        }
        if !lock(&shared.book).is_current(&job) {
            continue;
        }
        let outcome = catch_unwind(AssertUnwindSafe(|| run(&job, shared)))
            .unwrap_or_else(|_| Err(AnalysisError::Unreadable("the analysis crashed".to_owned())));
        let deliver = {
            let mut book = lock(&shared.book);
            let current = book.is_current(&job);
            if current {
                book.latest.remove(&job.track);
            }
            current
        };
        if deliver && !matches!(outcome, Err(AnalysisError::Cancelled)) {
            let _ = results.send(AnalysisResult {
                track: job.track,
                path: job.path,
                outcome,
            });
        }
    }
}

fn run(job: &Job, shared: &Shared) -> Result<Analysis, AnalysisError> {
    let settings = lock(&shared.settings).clone();
    // The key is taken before analysing: if the file changes meanwhile, the
    // result belongs to the old file and must not be cached for the new one.
    let key = shared
        .cache
        .as_ref()
        .and_then(|c| c.key(&job.path, &settings));
    if let (Some(cache), Some(key)) = (&shared.cache, &key)
        && let Some(analysis) = cache.load_key(key)
    {
        return Ok(analysis);
    }
    let cancelled = || shared.stop.load(Ordering::Acquire) || !lock(&shared.book).is_current(job);
    let outcome = (shared.analyze)(&job.path, &settings, &shared.limits, &cancelled);
    if let (Ok(analysis), Some(cache), Some(key)) = (&outcome, &shared.cache, &key)
        && cache.key(&job.path, &settings).as_ref() == Some(key)
        && let Err(e) = cache.store_key(key, analysis)
    {
        tracing::warn!(path = %job.path.display(), "cannot cache the analysis: {e}");
    }
    outcome
}
