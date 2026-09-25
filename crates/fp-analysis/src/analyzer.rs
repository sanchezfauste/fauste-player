//! The background analysis pool (spec §2.2, item 4): a few worker threads
//! analyse submitted files; results come back on a channel. Jobs can be
//! cancelled, duplicates are skipped, the cache is consulted first, and a
//! panic while analysing one file is contained and reported as unreadable.

use std::collections::HashSet;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread::JoinHandle;

use crossbeam_channel::{Receiver, Sender};
use fp_model::{AnalysisSettings, Limits, TrackId};

use crate::analyze::{Analysis, AnalysisError, analyze_file};
use crate::cache::AnalysisCache;

/// The analysis step; injectable for tests.
pub type AnalyzeFn =
    Arc<dyn Fn(&Path, &AnalysisSettings, &Limits) -> Result<Analysis, AnalysisError> + Send + Sync>;

#[derive(Debug)]
pub struct AnalysisResult {
    pub track: TrackId,
    pub path: PathBuf,
    pub outcome: Result<Analysis, AnalysisError>,
}

struct Job {
    track: TrackId,
    path: PathBuf,
}

#[derive(Default)]
struct Bookkeeping {
    queued: HashSet<TrackId>,
    cancelled: HashSet<TrackId>,
}

struct Shared {
    book: Mutex<Bookkeeping>,
    settings: Mutex<AnalysisSettings>,
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
        Self::with_analyze_fn(threads, settings, limits, cache, Arc::new(analyze_file))
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

    /// Queues `path` for analysis as `track` (ignored if already queued).
    pub fn submit(&self, track: TrackId, path: PathBuf) {
        {
            let mut book = lock(&self.shared.book);
            book.cancelled.remove(&track);
            if !book.queued.insert(track) {
                return;
            }
        }
        if let Some(jobs) = &self.jobs {
            let _ = jobs.send(Job { track, path });
        }
    }

    /// Drops a queued job; a result already in flight is suppressed.
    pub fn cancel(&self, track: TrackId) {
        let mut book = lock(&self.shared.book);
        if book.queued.remove(&track) {
            book.cancelled.insert(track);
        }
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
        self.jobs = None; // disconnects the queue: workers finish and exit
        for t in self.threads.drain(..) {
            let _ = t.join();
        }
    }
}

fn worker(jobs: &Receiver<Job>, results: &Sender<AnalysisResult>, shared: &Shared) {
    for job in jobs {
        if lock(&shared.book).cancelled.remove(&job.track) {
            continue;
        }
        let settings = lock(&shared.settings).clone();
        let cached = shared
            .cache
            .as_ref()
            .and_then(|c| c.load(&job.path, &settings));
        let outcome = match cached {
            Some(analysis) => Ok(analysis),
            None => {
                let outcome = catch_unwind(AssertUnwindSafe(|| {
                    (shared.analyze)(&job.path, &settings, &shared.limits)
                }))
                .unwrap_or_else(|_| {
                    Err(AnalysisError::Unreadable("the analysis crashed".to_owned()))
                });
                if let (Ok(analysis), Some(cache)) = (&outcome, &shared.cache)
                    && let Err(e) = cache.store(&job.path, &settings, analysis)
                {
                    tracing::warn!(path = %job.path.display(), "cannot cache the analysis: {e}");
                }
                outcome
            }
        };
        let cancelled = {
            let mut book = lock(&shared.book);
            book.queued.remove(&job.track);
            book.cancelled.remove(&job.track)
        };
        if !cancelled {
            let _ = results.send(AnalysisResult {
                track: job.track,
                path: job.path,
                outcome,
            });
        }
    }
}
