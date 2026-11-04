#![allow(
    dead_code,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The services rig: a conductor, an offline device and the services on a
//! temporary directory, shared by the services tests.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use fp_analysis::analyze_file_cancellable;
use fp_analysis::analyzer::{AnalyzeFn, Analyzer, LoudnessFn};
use fp_app::services::{MediaCache, Services};
use fp_backends::{AudioBackend, OfflineBackend, OfflineDevice};
use fp_engine::conductor::{Conductor, ConductorHandle};
use fp_engine::engine::{Engine, EngineSettings};
use fp_engine::worker::file_opener;
use fp_model::Command;
use fp_store::{AppPaths, Store};

pub fn wav(dir: &Path, name: &str, secs: u32) -> PathBuf {
    let path = dir.join(name);
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 48_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(&path, spec).unwrap();
    for i in 0..48_000 * secs {
        w.write_sample(((i as f32 * 0.05).sin() * 8_000.0) as i16)
            .unwrap();
    }
    w.finalize().unwrap();
    path
}

pub struct Rig {
    pub conductor: Conductor,
    pub handle: Arc<ConductorHandle>,
    pub services: Services,
    pub media: MediaCache,
    pub device: OfflineDevice,
    pub paths: AppPaths,
    pub analyses: Arc<AtomicUsize>,
    /// The files of the analyses in the order they started.
    pub started: Arc<Mutex<Vec<PathBuf>>>,
    /// While closed, an analysis that started holds there (still
    /// cancellable): what is queued meanwhile stays queued, so a test can
    /// look at the queue's order without racing the worker. Open by default.
    pub gate: Arc<AtomicBool>,
    /// Loudness-only jobs that started.
    pub loudness_runs: Arc<AtomicUsize>,
    /// How many analyses had returned when the last loudness-only job
    /// started.
    pub seen_finished: Arc<AtomicUsize>,
    pub now: Instant,
    pub _dir: tempfile::TempDir,
}

pub fn rig(files: &[PathBuf], dir: tempfile::TempDir) -> Rig {
    rig_slow(files, dir, Duration::ZERO)
}

/// `delay`: each analysis and each loudness-only job waits this long first
/// (still cancellable).
pub fn rig_slow(files: &[PathBuf], dir: tempfile::TempDir, delay: Duration) -> Rig {
    rig_with(files, dir, delay, |_| {})
}

/// As `rig_slow`, with `prepare` applied to the loaded state first.
pub fn rig_with(
    files: &[PathBuf],
    dir: tempfile::TempDir,
    delay: Duration,
    prepare: impl FnOnce(&mut fp_model::AppState),
) -> Rig {
    rig_hooked(files, dir, delay, prepare, Arc::new(|_| {}))
}

/// Runs after each analysis, on the worker, before its result is sent.
pub type AfterAnalysis = Arc<dyn Fn(&Path) + Send + Sync>;

/// As `rig_with`, with `after` called after each analysis.
pub fn rig_hooked(
    files: &[PathBuf],
    dir: tempfile::TempDir,
    delay: Duration,
    prepare: impl FnOnce(&mut fp_model::AppState),
    after: AfterAnalysis,
) -> Rig {
    rig_pool(files, dir, delay, prepare, after, 1, None)
}

/// As `rig_hooked`, with `threads` workers and, when given, `measure` in
/// place of the real loudness-only measurement.
pub fn rig_pool(
    files: &[PathBuf],
    dir: tempfile::TempDir,
    delay: Duration,
    prepare: impl FnOnce(&mut fp_model::AppState),
    after: AfterAnalysis,
    threads: usize,
    measure: Option<LoudnessFn>,
) -> Rig {
    rig_cached(files, dir, delay, prepare, after, threads, measure, false)
}

/// As `rig_pool`; with `cached`, the pool has an analysis cache in the
/// test's directory.
#[allow(clippy::too_many_arguments)]
pub fn rig_cached(
    files: &[PathBuf],
    dir: tempfile::TempDir,
    delay: Duration,
    prepare: impl FnOnce(&mut fp_model::AppState),
    after: AfterAnalysis,
    threads: usize,
    measure: Option<LoudnessFn>,
    cached: bool,
) -> Rig {
    let cache = cached.then(|| fp_analysis::cache::AnalysisCache::new(dir.path().join("cache")));
    let paths = AppPaths::under(dir.path());
    let mut store = Store::new(paths.clone(), Default::default());
    let mut loaded = store.load("Main");
    loaded.state.config.outputs.backend = Some("offline".into());
    loaded.state.config.outputs.buffer_frames = 480;
    let playlist = loaded.state.playlists.first_id().unwrap();
    fp_model::apply(
        &mut loaded.state,
        Command::InsertPaths {
            playlist,
            index: 0,
            paths: files.to_vec(),
        },
    )
    .unwrap();
    prepare(&mut loaded.state);
    let backend = OfflineBackend::new();
    let device = backend.add_device("main", 2);
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
    let engine = Engine::new(
        backends,
        EngineSettings::from_config(&loaded.state.config),
        file_opener(),
    );
    let now = Instant::now();
    let (conductor, handle) = Conductor::new(loaded.state, loaded.actions, engine, now);
    let handle = Arc::new(handle);
    let analyses = Arc::new(AtomicUsize::new(0));
    let finished = Arc::new(AtomicUsize::new(0));
    let started = Arc::new(Mutex::new(Vec::new()));
    let gate = Arc::new(AtomicBool::new(true));
    let counter = analyses.clone();
    let done = finished.clone();
    let (order, open) = (started.clone(), gate.clone());
    let counting: AnalyzeFn = Arc::new(move |path, settings, limits, cancelled| {
        order.lock().unwrap().push(path.to_path_buf());
        counter.fetch_add(1, Ordering::SeqCst);
        while !open.load(Ordering::SeqCst) {
            if cancelled() {
                return Err(fp_analysis::AnalysisError::Cancelled);
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        if !wait(delay, cancelled) {
            return Err(fp_analysis::AnalysisError::Cancelled);
        }
        let outcome = analyze_file_cancellable(path, settings, limits, cancelled);
        after(path);
        done.fetch_add(1, Ordering::SeqCst);
        outcome
    });
    let loudness_runs = Arc::new(AtomicUsize::new(0));
    let seen_finished = Arc::new(AtomicUsize::new(0));
    let (runs, seen, done) = (
        loudness_runs.clone(),
        seen_finished.clone(),
        finished.clone(),
    );
    let loudness: LoudnessFn = Arc::new(move |path, cancelled| {
        runs.fetch_add(1, Ordering::SeqCst);
        seen.store(done.load(Ordering::SeqCst), Ordering::SeqCst);
        if !wait(delay, cancelled) {
            return Err(fp_analysis::AnalysisError::Cancelled);
        }
        match &measure {
            Some(measure) => measure(path, cancelled),
            None => fp_analysis::measure_loudness_cancellable(path, cancelled),
        }
    });
    let analyzer = Analyzer::with_fns(
        threads,
        Default::default(),
        Default::default(),
        cache,
        counting,
        loudness,
    )
    .unwrap();
    let media = MediaCache::default();
    let services = Services::new(handle.clone(), store, analyzer, media.clone());
    Rig {
        conductor,
        handle,
        services,
        media,
        device,
        paths,
        analyses,
        started,
        gate,
        loudness_runs,
        seen_finished,
        now,
        _dir: dir,
    }
}

/// Waits `delay`; false if cancelled meanwhile.
pub fn wait(delay: Duration, cancelled: &dyn Fn() -> bool) -> bool {
    let until = Instant::now() + delay;
    while Instant::now() < until {
        if cancelled() {
            return false;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    true
}

impl Rig {
    pub fn run_until(&mut self, what: &str, mut done: impl FnMut(&Rig) -> bool) {
        // Only bounds a hang: a loaded CI runner can take well over 15 s.
        let deadline = Instant::now() + Duration::from_secs(60);
        while !done(self) {
            assert!(Instant::now() < deadline, "timed out waiting for {what}");
            self.conductor.tick(self.now);
            self.services.step(self.now);
            let _ = self.device.render(480);
            self.now += Duration::from_millis(10);
            std::thread::sleep(Duration::from_millis(2));
        }
    }
}

/// How many analyses started before the first one of `path`.
pub fn started_before(r: &Rig, path: &Path) -> usize {
    let started = r.started.lock().unwrap();
    started
        .iter()
        .position(|p| p == path)
        .unwrap_or_else(|| panic!("{} was never analysed: {started:?}", path.display()))
}
