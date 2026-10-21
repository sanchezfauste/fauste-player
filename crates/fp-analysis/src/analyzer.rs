//! The background analysis pool (spec §2.2, item 4): a few worker threads
//! analyse submitted files; results come back on a channel.
//!
//! Every submission gets a generation number. A job only runs, and its
//! result is only delivered, while it is still the latest submission of its
//! track: this one rule handles cancellation, duplicates and resubmission.
//! Running jobs see cancellation (and shutdown) between decoded blocks. A
//! panic anywhere in a job is contained and reported as unreadable.
//!
//! Urgent jobs (tracks a player shows, which need their waveform now) have
//! their own queue, which workers always take from first.
//!
//! Loudness-only jobs (loudness normalization spec N6b) have a third queue,
//! taken last, with their own bookkeeping and results: a loudness job and
//! an analysis of the same track are independent of each other.
//!
//! A fresh job (main spec §6.1, an explicit re-analysis) skips the cache
//! lookup and analyses the file, then stores its result as any job does.
//! `activity` reports what is queued and running (main spec §6.1).

use std::collections::{HashMap, HashSet};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread::JoinHandle;
use std::time::SystemTime;

use crossbeam_channel::{Receiver, Sender};
use fp_decode::priority::Priority;
use fp_model::{AnalysisSettings, LOUDNESS_VERSION, Limits, LoudnessMeasurement, TrackId};

use crate::analyze::{
    Analysis, AnalysisError, analyze_file_cancellable, measure_loudness_cancellable,
};
use crate::cache::AnalysisCache;

/// The analysis step; injectable for tests. The last argument reports
/// whether the job has been cancelled.
pub type AnalyzeFn = Arc<
    dyn Fn(&Path, &AnalysisSettings, &Limits, &dyn Fn() -> bool) -> Result<Analysis, AnalysisError>
        + Send
        + Sync,
>;

/// The loudness-only measurement (N6b); injectable for tests. The last
/// argument reports whether the job has been cancelled.
pub type LoudnessFn = Arc<
    dyn Fn(&Path, &dyn Fn() -> bool) -> Result<Option<LoudnessMeasurement>, AnalysisError>
        + Send
        + Sync,
>;

/// The answer of a loudness-only job.
#[derive(Debug)]
pub struct LoudnessResult {
    pub track: TrackId,
    pub path: PathBuf,
    /// `Ok(None)`: the file decoded but nothing could be measured (shorter
    /// than 400 ms, or a failure inside the meter).
    pub outcome: Result<Option<LoudnessMeasurement>, AnalysisError>,
}

#[derive(Debug)]
pub struct AnalysisResult {
    pub track: TrackId,
    pub path: PathBuf,
    pub outcome: Result<Analysis, AnalysisError>,
    /// The file's size and modification time, read just before it was
    /// analysed: what the outcome belongs to. `None` if the `stat` failed.
    pub stamp: Option<(u64, SystemTime)>,
}

struct Job {
    track: TrackId,
    path: PathBuf,
    generation: u64,
}

/// What the pool is doing (main spec §6.1), from its bookkeeping. Each
/// list is sorted.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Activity {
    /// Tracks whose analysis waits for a worker.
    pub queued: Vec<TrackId>,
    /// Tracks a worker is analysing now (a cancelled job is not counted).
    pub running: Vec<TrackId>,
    /// Loudness-only jobs queued or running (N6b).
    pub loudness: usize,
}

#[derive(Default)]
struct Bookkeeping {
    /// Latest pending generation per track; absent means nothing pending.
    latest: HashMap<TrackId, u64>,
    /// Tracks whose pending job is urgent.
    urgent: HashSet<TrackId>,
    /// The generation a worker is analysing now, per track.
    running: HashMap<TrackId, u64>,
    /// Tracks whose pending job skips the cache lookup (an explicit
    /// re-analysis). Kept here, not in the job, so a job moved ahead
    /// (`promote`) stays fresh.
    fresh: HashSet<TrackId>,
    next_generation: u64,
    /// Bumped on every change, so a reader rebuilds its view only then.
    changes: u64,
}

impl Bookkeeping {
    fn submit(&mut self, track: TrackId) -> u64 {
        self.changes += 1;
        self.next_generation += 1;
        self.latest.insert(track, self.next_generation);
        self.next_generation
    }

    /// Whether the latest job of `track` is already running.
    fn is_running(&self, track: TrackId) -> bool {
        self.latest
            .get(&track)
            .is_some_and(|g| self.running.get(&track) == Some(g))
    }

    fn is_current(&self, job: &Job) -> bool {
        self.latest.get(&job.track) == Some(&job.generation)
    }
}

struct Shared {
    book: Mutex<Bookkeeping>,
    settings: Mutex<AnalysisSettings>,
    stop: AtomicBool,
    limits: Mutex<Limits>,
    cache: Option<AnalysisCache>,
    analyze: AnalyzeFn,
    /// The loudness-only jobs' own bookkeeping (N6b).
    loudness_book: Mutex<Bookkeeping>,
    loudness: LoudnessFn,
}

/// Books an urgent job for `track`, unless one is already waiting or its
/// latest job is running; the generation to queue it with.
fn urgent_generation(book: &mut Bookkeeping, track: TrackId) -> Option<u64> {
    if book.urgent.contains(&track) || book.is_running(track) {
        return None;
    }
    book.urgent.insert(track);
    Some(book.submit(track))
}

/// Stale jobs the normal queue may hold beyond twice the live ones before
/// they are dropped from it: a few restarts are skipped by the workers.
const STALE_SLACK: usize = 64;

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

pub struct Analyzer {
    jobs: Option<Sender<Job>>,
    /// The normal queue's other end: stale jobs are dropped through it.
    queue: Receiver<Job>,
    urgent: Option<Sender<Job>>,
    loudness_jobs: Option<Sender<Job>>,
    results: Receiver<AnalysisResult>,
    loudness_results: Receiver<LoudnessResult>,
    /// How many loudness-only jobs may run at once (`loudness_share`).
    share: usize,
    shared: Arc<Shared>,
    threads: Vec<JoinHandle<()>>,
    /// Runs inside `promote` once it has looked at the track's job, so a
    /// test can let that job finish right there. It must stay between the
    /// look and the send: that is the window the regression test exercises.
    #[cfg(test)]
    promote_hook: Option<Box<dyn Fn() + Send + Sync>>,
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

    /// A pool running `analyze` and the real loudness-only measurement.
    pub fn with_analyze_fn(
        threads: usize,
        settings: AnalysisSettings,
        limits: Limits,
        cache: Option<AnalysisCache>,
        analyze: AnalyzeFn,
    ) -> std::io::Result<Self> {
        Self::with_fns(
            threads,
            settings,
            limits,
            cache,
            analyze,
            Arc::new(measure_loudness_cancellable),
        )
    }

    /// A pool running `analyze` for analyses and `loudness` for
    /// loudness-only jobs.
    pub fn with_fns(
        threads: usize,
        settings: AnalysisSettings,
        limits: Limits,
        cache: Option<AnalysisCache>,
        analyze: AnalyzeFn,
        loudness: LoudnessFn,
    ) -> std::io::Result<Self> {
        let (job_tx, job_rx) = crossbeam_channel::unbounded::<Job>();
        let (urgent_tx, urgent_rx) = crossbeam_channel::unbounded::<Job>();
        let (loudness_tx, loudness_rx) = crossbeam_channel::unbounded::<Job>();
        let (res_tx, res_rx) = crossbeam_channel::unbounded();
        let (lres_tx, lres_rx) = crossbeam_channel::unbounded();
        let shared = Arc::new(Shared {
            book: Mutex::new(Bookkeeping::default()),
            settings: Mutex::new(settings),
            stop: AtomicBool::new(false),
            limits: Mutex::new(limits),
            cache,
            analyze,
            loudness_book: Mutex::new(Bookkeeping::default()),
            loudness,
        });
        let workers = threads.max(1);
        let mut handles = Vec::new();
        for n in 0..workers {
            let lanes = Lanes {
                urgent: urgent_rx.clone(),
                normal: job_rx.clone(),
                loudness: loudness_rx.clone(),
            };
            let (tx, ltx, shared) = (res_tx.clone(), lres_tx.clone(), shared.clone());
            handles.push(
                std::thread::Builder::new()
                    .name(format!("fp-analysis-{n}"))
                    .spawn(move || {
                        // Background work (main spec §2.2): it must never
                        // take the processor from decoding or the interface.
                        fp_decode::priority::set_current(Priority::Low, "analysis");
                        // One worker tidies the cache before taking jobs.
                        if n == 0
                            && let Some(cache) = &shared.cache
                        {
                            cache.sweep();
                        }
                        worker(&lanes, &tx, &ltx, &shared)
                    })?,
            );
        }
        Ok(Self {
            jobs: Some(job_tx),
            queue: job_rx,
            urgent: Some(urgent_tx),
            loudness_jobs: Some(loudness_tx),
            results: res_rx,
            loudness_results: lres_rx,
            share: workers.saturating_sub(1).max(1),
            shared,
            threads: handles,
            #[cfg(test)]
            promote_hook: None,
        })
    }

    /// Queues `path` for analysis as `track` (ignored if one is pending).
    pub fn submit(&self, track: TrackId, path: PathBuf) {
        self.queue_normal(track, path, false);
    }

    /// As `submit`, for an explicit re-analysis: the job skips the cache
    /// lookup and stores its result (main spec §6.1).
    pub fn submit_fresh(&self, track: TrackId, path: PathBuf) {
        self.queue_normal(track, path, true);
    }

    fn queue_normal(&self, track: TrackId, path: PathBuf, fresh: bool) {
        let generation = {
            let mut book = lock(&self.shared.book);
            if book.latest.contains_key(&track) {
                return;
            }
            self.drop_stale(&book);
            if fresh {
                book.fresh.insert(track);
            }
            book.submit(track)
        };
        if let Some(jobs) = &self.jobs {
            let _ = jobs.send(Job {
                track,
                path,
                generation,
            });
        }
    }

    /// Cancelled jobs stay in the normal queue until a worker skips them.
    /// Once they outnumber the live jobs (restarts of the whole library
    /// in a row), they are dropped from it, keeping the live ones in order.
    fn drop_stale(&self, book: &Bookkeeping) {
        if self.queue.len() <= 2 * book.latest.len() + STALE_SLACK {
            return;
        }
        let Some(jobs) = &self.jobs else {
            return;
        };
        let waiting: Vec<Job> = self.queue.try_iter().collect();
        for job in waiting.into_iter().filter(|j| book.is_current(j)) {
            let _ = jobs.send(job);
        }
    }

    /// Queues `path` ahead of every normal job. A normal job pending for
    /// `track` is superseded (it will be skipped); an urgent or running one
    /// is kept.
    pub fn submit_urgent(&self, track: TrackId, path: PathBuf) {
        let generation = urgent_generation(&mut lock(&self.shared.book), track);
        if let Some(generation) = generation {
            self.send_urgent(track, path, generation);
        }
    }

    /// As `submit_urgent`, for an explicit re-analysis (see
    /// `submit_fresh`).
    pub fn submit_urgent_fresh(&self, track: TrackId, path: PathBuf) {
        let generation = {
            let mut book = lock(&self.shared.book);
            let generation = urgent_generation(&mut book, track);
            if generation.is_some() {
                book.fresh.insert(track);
            }
            generation
        };
        if let Some(generation) = generation {
            self.send_urgent(track, path, generation);
        }
    }

    /// Moves whenever the pool's bookkeeping changes (a job queued,
    /// started, ended or cancelled, analysis or loudness-only): `activity`
    /// can only differ when this did.
    pub fn activity_version(&self) -> u64 {
        let analysis = lock(&self.shared.book).changes;
        analysis.wrapping_add(lock(&self.shared.loudness_book).changes)
    }

    /// What is queued and running now (main spec §6.1).
    pub fn activity(&self) -> Activity {
        let (mut queued, mut running) = (Vec::new(), Vec::new());
        {
            let book = lock(&self.shared.book);
            for &track in book.latest.keys() {
                if book.is_running(track) {
                    running.push(track);
                } else {
                    queued.push(track);
                }
            }
        }
        queued.sort_unstable();
        running.sort_unstable();
        Activity {
            queued,
            running,
            loudness: lock(&self.shared.loudness_book).latest.len(),
        }
    }

    /// Moves a normal job still waiting for `track` ahead of the queue;
    /// nothing if no job is waiting (none, or its result already sent).
    /// The look and the urgent submission are one step under the lock: a
    /// job that ends meanwhile is never started again.
    pub fn promote(&self, track: TrackId, path: &Path) {
        let generation = {
            let mut book = lock(&self.shared.book);
            if book.latest.contains_key(&track) {
                urgent_generation(&mut book, track)
            } else {
                None
            }
        };
        #[cfg(test)]
        if let Some(hook) = &self.promote_hook {
            hook();
        }
        if let Some(generation) = generation {
            self.send_urgent(track, path.to_path_buf(), generation);
        }
    }

    fn send_urgent(&self, track: TrackId, path: PathBuf, generation: u64) {
        if let Some(urgent) = &self.urgent {
            let _ = urgent.send(Job {
                track,
                path,
                generation,
            });
        }
    }

    /// Cancels a pending or running job; no result will be delivered for it.
    pub fn cancel(&self, track: TrackId) {
        let mut book = lock(&self.shared.book);
        book.changes += 1;
        book.latest.remove(&track);
        book.urgent.remove(&track);
        book.fresh.remove(&track);
    }

    /// Settings for jobs that start after this call.
    pub fn update_settings(&self, settings: AnalysisSettings) {
        *lock(&self.shared.settings) = settings;
    }

    /// Limits for jobs that start after this call (live settings spec §6);
    /// the cache keys are built from the same snapshot, so entries made under the old limits
    /// miss. Nothing is analysed again.
    pub fn update_limits(&self, limits: Limits) {
        *lock(&self.shared.limits) = limits;
    }

    /// The limits jobs start with now.
    pub fn limits(&self) -> Limits {
        lock(&self.shared.limits).clone()
    }

    pub fn results(&self) -> &Receiver<AnalysisResult> {
        &self.results
    }

    /// N6b: queues a loudness-only measurement of `path` as `track`,
    /// behind every analysis job (ignored while one is pending for
    /// `track`).
    pub fn submit_loudness(&self, track: TrackId, path: PathBuf) {
        let generation = {
            let mut book = lock(&self.shared.loudness_book);
            if book.latest.contains_key(&track) {
                return;
            }
            book.submit(track)
        };
        if let Some(jobs) = &self.loudness_jobs {
            let _ = jobs.send(Job {
                track,
                path,
                generation,
            });
        }
    }

    /// Cancels a pending or running loudness-only job; no result will be
    /// delivered for it.
    pub fn cancel_loudness(&self, track: TrackId) {
        let mut book = lock(&self.shared.loudness_book);
        book.changes += 1;
        book.latest.remove(&track);
    }

    pub fn loudness_results(&self) -> &Receiver<LoudnessResult> {
        &self.loudness_results
    }

    /// How many loudness-only jobs may run at once: all workers but one,
    /// at least one (N6b), so new files and the operator's requests keep
    /// a worker. The caller keeps to it.
    pub fn loudness_share(&self) -> usize {
        self.share
    }
}

impl Drop for Analyzer {
    fn drop(&mut self) {
        // Stop at once: queued jobs are skipped and running ones cancelled.
        self.shared.stop.store(true, Ordering::Release);
        self.jobs = None;
        self.urgent = None;
        self.loudness_jobs = None;
        for t in self.threads.drain(..) {
            let _ = t.join();
        }
    }
}

/// A worker's three queues.
struct Lanes {
    urgent: Receiver<Job>,
    normal: Receiver<Job>,
    loudness: Receiver<Job>,
}

/// Which queue a job came from.
#[derive(Clone, Copy)]
enum Lane {
    Analysis,
    Loudness,
}

impl Lanes {
    /// The next job: urgent first, then normal, then loudness-only (N6b:
    /// below every analysis job); `None` once the analyzer is gone.
    fn next(&self) -> Option<(Lane, Job)> {
        if let Ok(job) = self.urgent.try_recv() {
            return Some((Lane::Analysis, job));
        }
        if let Ok(job) = self.normal.try_recv() {
            return Some((Lane::Analysis, job));
        }
        crossbeam_channel::select_biased! {
            recv(self.urgent) -> job => job.ok().map(|j| (Lane::Analysis, j)),
            recv(self.normal) -> job => job.ok().map(|j| (Lane::Analysis, j)),
            recv(self.loudness) -> job => job.ok().map(|j| (Lane::Loudness, j)),
        }
    }
}

fn worker(
    lanes: &Lanes,
    results: &Sender<AnalysisResult>,
    loudness_results: &Sender<LoudnessResult>,
    shared: &Shared,
) {
    while let Some((lane, job)) = lanes.next() {
        if shared.stop.load(Ordering::Acquire) {
            return;
        }
        match lane {
            Lane::Analysis => analysis_job(job, results, shared),
            Lane::Loudness => loudness_job(job, loudness_results, shared),
        }
    }
}

fn analysis_job(job: Job, results: &Sender<AnalysisResult>, shared: &Shared) {
    let fresh = {
        let mut book = lock(&shared.book);
        if !book.is_current(&job) {
            return;
        }
        book.changes += 1;
        book.urgent.remove(&job.track);
        book.running.insert(job.track, job.generation);
        book.fresh.remove(&job.track)
    };
    // Before the analysis: a file that changes while it runs (a copy
    // that completes) is told apart from what was analysed.
    let stamp = std::fs::metadata(&job.path)
        .ok()
        .and_then(|m| Some((m.len(), m.modified().ok()?)));
    let outcome = catch_unwind(AssertUnwindSafe(|| run(&job, fresh, shared)))
        .unwrap_or_else(|_| Err(AnalysisError::Unreadable("the analysis crashed".to_owned())));
    let deliver = {
        let mut book = lock(&shared.book);
        book.changes += 1;
        if book.running.get(&job.track) == Some(&job.generation) {
            book.running.remove(&job.track);
        }
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
            stamp,
        });
    }
}

fn run(job: &Job, fresh: bool, shared: &Shared) -> Result<Analysis, AnalysisError> {
    let settings = lock(&shared.settings).clone();
    let limits = lock(&shared.limits).clone();
    // The key is taken before analysing: if the file changes meanwhile, the
    // result belongs to the old file and must not be cached for the new one.
    // It and the analysis use the same `limits` snapshot, so a change of
    // limits during the job cannot store a result under the new key.
    let key = shared
        .cache
        .as_ref()
        .and_then(|c| c.key(&job.path, &settings, &limits));
    if !fresh
        && let (Some(cache), Some(key)) = (&shared.cache, &key)
        && let Some(analysis) = cache.load_key(key, &limits)
    {
        return Ok(analysis);
    }
    let cancelled = || shared.stop.load(Ordering::Acquire) || !lock(&shared.book).is_current(job);
    let outcome = (shared.analyze)(&job.path, &settings, &limits, &cancelled);
    if let (Ok(analysis), Some(cache), Some(key)) = (&outcome, &shared.cache, &key)
        && cache.key(&job.path, &settings, &limits).as_ref() == Some(key)
        && let Err(e) = cache.store_key(key, analysis, &limits)
    {
        tracing::warn!(path = %job.path.display(), "cannot cache the analysis: {e}");
    }
    outcome
}

fn loudness_job(job: Job, results: &Sender<LoudnessResult>, shared: &Shared) {
    {
        let mut book = lock(&shared.loudness_book);
        if !book.is_current(&job) {
            return;
        }
        book.changes += 1;
        book.running.insert(job.track, job.generation);
    }
    let outcome =
        catch_unwind(AssertUnwindSafe(|| run_loudness(&job, shared))).unwrap_or_else(|_| {
            Err(AnalysisError::Unreadable(
                "the loudness measurement crashed".to_owned(),
            ))
        });
    let deliver = {
        let mut book = lock(&shared.loudness_book);
        book.changes += 1;
        if book.running.get(&job.track) == Some(&job.generation) {
            book.running.remove(&job.track);
        }
        let current = book.is_current(&job);
        if current {
            book.latest.remove(&job.track);
        }
        current
    };
    if deliver && !matches!(outcome, Err(AnalysisError::Cancelled)) {
        let _ = results.send(LoudnessResult {
            track: job.track,
            path: job.path,
            outcome,
        });
    }
}

fn run_loudness(job: &Job, shared: &Shared) -> Result<Option<LoudnessMeasurement>, AnalysisError> {
    let settings = lock(&shared.settings).clone();
    let limits = lock(&shared.limits).clone();
    // The key is taken first: a file that changes meanwhile is not
    // recorded under its new key.
    let key = shared
        .cache
        .as_ref()
        .and_then(|c| c.key(&job.path, &settings, &limits));
    // The entry may already hold this pass's answer (the library lost it,
    // or the full analysis cached it meanwhile): nothing to decode.
    if let (Some(cache), Some(key)) = (&shared.cache, &key)
        && let Some(cached) = cache.load_key(key, &limits)
        && cached.analysis.loudness_pass == Some(LOUDNESS_VERSION)
    {
        return Ok(cached.analysis.loudness);
    }
    let cancelled =
        || shared.stop.load(Ordering::Acquire) || !lock(&shared.loudness_book).is_current(job);
    let outcome = (shared.loudness)(&job.path, &cancelled);
    // A measurement (or a file too short or too quiet to have one) is
    // recorded in the file's entry so it is not measured again (N6b). A
    // failure may be transient (a share that dropped mid-read): only the
    // model records it, and a Re-analyse measures the file again.
    if let (Ok(loudness), Some(cache), Some(key)) = (&outcome, &shared.cache, &key)
        && cache.key(&job.path, &settings, &limits).as_ref() == Some(key)
        && let Err(e) = cache.update_loudness(key, *loudness, &limits)
    {
        tracing::warn!(path = %job.path.display(), "cannot cache the loudness: {e}");
    }
    outcome
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use std::time::{Duration, Instant};

    const WAIT: Duration = Duration::from_secs(30);

    /// A job that ends while `promote` looks at it is not started again:
    /// promote moves a waiting job ahead, it never queues a new analysis
    /// of a track whose result was just sent.
    #[test]
    fn a_job_that_ends_while_it_is_promoted_is_not_analysed_again() {
        let (a, b) = (PathBuf::from("a.wav"), PathBuf::from("b.wav"));
        let open = Arc::new(AtomicBool::new(false));
        let runs_of_a = Arc::new(AtomicUsize::new(0));
        let (gate, runs, held) = (open.clone(), runs_of_a.clone(), a.clone());
        // `a` holds until the gate opens; every job fails at once, which
        // still delivers a result.
        let analyze: AnalyzeFn = Arc::new(move |path, _, _, cancelled| {
            if path == held {
                runs.fetch_add(1, Ordering::SeqCst);
                while !gate.load(Ordering::SeqCst) && !cancelled() {
                    std::thread::sleep(Duration::from_millis(1));
                }
            }
            Err(AnalysisError::Unreadable("test".to_owned()))
        });
        let loudness: LoudnessFn = Arc::new(|_, _| Ok(None));
        let mut analyzer = Analyzer::with_fns(
            1,
            AnalysisSettings::default(),
            Limits::default(),
            None,
            analyze,
            loudness,
        )
        .unwrap();
        analyzer.submit(TrackId(1), a.clone());
        let deadline = Instant::now() + WAIT;
        while runs_of_a.load(Ordering::SeqCst) == 0 {
            assert!(Instant::now() < deadline, "the job never started");
            std::thread::sleep(Duration::from_millis(1));
        }
        // While promote looks at the running job, it ends and its result
        // is sent.
        let results = analyzer.results().clone();
        analyzer.promote_hook = Some(Box::new(move || {
            open.store(true, Ordering::SeqCst);
            let result = results.recv_timeout(WAIT).expect("the job's result");
            assert_eq!(result.track, TrackId(1));
        }));
        analyzer.promote(TrackId(1), &a);
        analyzer.promote_hook = None;
        // One worker, urgent jobs first: a second job of `a` would run
        // before `b`.
        analyzer.submit(TrackId(2), b);
        let next = analyzer.results().recv_timeout(WAIT).expect("b's result");
        assert_eq!(next.track, TrackId(2), "a was analysed again");
        assert_eq!(runs_of_a.load(Ordering::SeqCst), 1);
    }

    /// Main spec §6.1: restarting the whole library again and again leaves
    /// the cancelled jobs behind; the queue drops them instead of growing.
    #[test]
    fn repeated_restarts_do_not_grow_the_queue() {
        let open = Arc::new(AtomicBool::new(false));
        let gate = open.clone();
        // Every job holds until the gate opens: none is taken meanwhile
        // but the first.
        let analyze: AnalyzeFn = Arc::new(move |_, _, _, cancelled| {
            while !gate.load(Ordering::SeqCst) && !cancelled() {
                std::thread::sleep(Duration::from_millis(1));
            }
            Err(AnalysisError::Unreadable("test".to_owned()))
        });
        let loudness: LoudnessFn = Arc::new(|_, _| Ok(None));
        let analyzer = Analyzer::with_fns(
            1,
            AnalysisSettings::default(),
            Limits::default(),
            None,
            analyze,
            loudness,
        )
        .unwrap();
        let library = 100;
        for _ in 0..20 {
            for n in 0..library {
                analyzer.submit(TrackId(n), PathBuf::from("a.wav"));
            }
            for n in 0..library {
                analyzer.cancel(TrackId(n));
            }
        }
        for n in 0..library {
            analyzer.submit(TrackId(n), PathBuf::from("a.wav"));
        }
        let held = analyzer.queue.len();
        assert!(
            held <= 3 * library as usize + STALE_SLACK,
            "{held} jobs queued for {library} tracks"
        );
        // The live jobs are all still there, in order.
        open.store(true, Ordering::SeqCst);
        let mut answered: Vec<TrackId> = (0..library)
            .map(|_| analyzer.results().recv_timeout(WAIT).unwrap().track)
            .collect();
        let in_order = answered.is_sorted();
        answered.sort_unstable();
        answered.dedup();
        assert_eq!(answered.len(), library as usize);
        assert!(in_order, "out of order");
    }
}
