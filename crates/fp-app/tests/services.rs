#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The services thread: analysis wiring and autosave (spec §6, §7).

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use fp_analysis::analyze_file_cancellable;
use fp_analysis::analyzer::{AnalyzeFn, Analyzer, LoudnessFn};
use fp_app::activity::{PassProgress, TrackActivity};
use fp_app::services::{MediaCache, ServiceRequest, Services, TrackMedia};
use fp_backends::{AudioBackend, OfflineBackend, OfflineDevice};
use fp_engine::conductor::{Conductor, ConductorHandle};
use fp_engine::engine::{Engine, EngineSettings};
use fp_engine::worker::file_opener;
use fp_model::{Command, EngineAction, FileState, Transport};
use fp_store::{AppPaths, Store};

fn wav(dir: &Path, name: &str, secs: u32) -> PathBuf {
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

struct Rig {
    conductor: Conductor,
    handle: Arc<ConductorHandle>,
    services: Services,
    media: MediaCache,
    device: OfflineDevice,
    paths: AppPaths,
    analyses: Arc<AtomicUsize>,
    /// The files of the analyses in the order they started.
    started: Arc<Mutex<Vec<PathBuf>>>,
    /// While closed, an analysis that started holds there (still
    /// cancellable): what is queued meanwhile stays queued, so a test can
    /// look at the queue's order without racing the worker. Open by default.
    gate: Arc<AtomicBool>,
    /// Loudness-only jobs that started.
    loudness_runs: Arc<AtomicUsize>,
    /// How many analyses had returned when the last loudness-only job
    /// started.
    seen_finished: Arc<AtomicUsize>,
    now: Instant,
    _dir: tempfile::TempDir,
}

fn rig(files: &[PathBuf], dir: tempfile::TempDir) -> Rig {
    rig_slow(files, dir, Duration::ZERO)
}

/// `delay`: each analysis and each loudness-only job waits this long first
/// (still cancellable).
fn rig_slow(files: &[PathBuf], dir: tempfile::TempDir, delay: Duration) -> Rig {
    rig_with(files, dir, delay, |_| {})
}

/// As `rig_slow`, with `prepare` applied to the loaded state first.
fn rig_with(
    files: &[PathBuf],
    dir: tempfile::TempDir,
    delay: Duration,
    prepare: impl FnOnce(&mut fp_model::AppState),
) -> Rig {
    rig_hooked(files, dir, delay, prepare, Arc::new(|_| {}))
}

/// Runs after each analysis, on the worker, before its result is sent.
type AfterAnalysis = Arc<dyn Fn(&Path) + Send + Sync>;

/// As `rig_with`, with `after` called after each analysis.
fn rig_hooked(
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
fn rig_pool(
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
fn rig_cached(
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
    let store = Store::new(paths.clone(), Default::default());
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
fn wait(delay: Duration, cancelled: &dyn Fn() -> bool) -> bool {
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
    fn run_until(&mut self, what: &str, mut done: impl FnMut(&Rig) -> bool) {
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

#[test]
fn unanalysed_tracks_are_submitted_once_and_results_reach_the_model() {
    let dir = tempfile::tempdir().unwrap();
    let a = wav(dir.path(), "Band - One.wav", 2);
    let mut r = rig(&[a], dir);
    r.run_until("analysis", |r| {
        r.handle.model.load().library.iter().all(|t| t.analyzed)
    });
    let model = r.handle.model.load();
    let track = model.library.iter().next().unwrap();
    assert!((track.duration_secs - 2.0).abs() < 1e-6);
    assert_eq!(
        (track.title.as_str(), track.artist.as_str()),
        ("One", "Band")
    );
    let media = r.media.get(track.id).unwrap();
    assert!(!media.peaks.is_empty());
    let until = r.now + Duration::from_millis(500);
    r.run_until("a quiet period", |r| r.now >= until);
    assert_eq!(r.analyses.load(Ordering::SeqCst), 1, "analysed once");
}

#[test]
fn missing_files_are_marked() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = rig(&[PathBuf::from("/definitely/missing.flac")], dir);
    r.run_until("missing state", |r| {
        r.handle
            .model
            .load()
            .library
            .iter()
            .all(|t| t.file_state == FileState::Missing)
    });
}

#[test]
fn changes_are_saved_after_the_debounce() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = rig(&[], dir);
    r.handle.send(Command::CreatePlaylist {
        name: "Night".into(),
    });
    r.conductor.tick(r.now);
    r.services.step(r.now);
    let file = r.paths.playlists_file();
    let saved = |p: &Path| {
        std::fs::read_to_string(p)
            .unwrap_or_default()
            .contains("Night")
    };
    assert!(!saved(&file), "not before the debounce");
    r.now += Duration::from_millis(1_100);
    r.conductor.tick(r.now);
    r.services.step(r.now);
    assert!(saved(&file));
}

#[test]
fn shutdown_saves_the_session_with_positions() {
    let dir = tempfile::tempdir().unwrap();
    let a = wav(dir.path(), "a.wav", 5);
    let mut r = rig(&[a], dir);
    let p = r.handle.model.load().players[0].id;
    r.handle.send(Command::Play(p));
    r.run_until("playback", |r| {
        r.handle
            .telemetry
            .load()
            .players
            .iter()
            .any(|(id, t)| *id == p && t.position_secs.is_some_and(|s| s > 0.3))
    });
    r.services.shutdown();
    let loaded = Store::new(r.paths.clone(), Default::default()).load("Main");
    assert_eq!(loaded.state.player(p).unwrap().transport, Transport::Paused);
    assert!(loaded.actions.iter().any(|a| matches!(
        a,
        EngineAction::LoadPaused { player, request } if *player == p && request.from_secs > 0.3
    )));
}

#[test]
fn changing_analysis_settings_during_analysis_analyses_again() {
    let dir = tempfile::tempdir().unwrap();
    let a = wav(dir.path(), "a.wav", 2);
    let mut r = rig_slow(&[a], dir, Duration::from_millis(300));
    r.run_until("analysis started", |r| {
        r.analyses.load(Ordering::SeqCst) == 1
    });
    let mut config = r.handle.model.load().config.clone();
    config.analysis.trim_threshold_db = -50.0;
    r.handle.send(Command::UpdateConfig(Box::new(config)));
    r.run_until("analysis", |r| {
        r.handle.model.load().library.iter().all(|t| t.analyzed)
    });
    assert_eq!(
        r.analyses.load(Ordering::SeqCst),
        2,
        "the running analysis was redone"
    );
}

#[test]
fn peaks_are_kept_only_for_tracks_on_a_player() {
    let dir = tempfile::tempdir().unwrap();
    let files: Vec<PathBuf> = (1..=3)
        .map(|n| wav(dir.path(), &format!("{n}.wav"), 1))
        .collect();
    let mut r = rig(&files, dir);
    r.run_until("analysis", |r| {
        r.handle.model.load().library.iter().all(|t| t.analyzed)
    });
    let until = r.now + Duration::from_millis(300);
    r.run_until("a quiet period", |r| r.now >= until);
    let model = r.handle.model.load_full();
    let tracks: Vec<_> = model.library.iter().map(|t| t.id).collect();
    let next = model.players[0].next.unwrap();
    let next_track = model.playlists.entry(next).unwrap().track;
    for t in &tracks {
        assert_eq!(r.media.contains(*t), *t == next_track, "{t:?}");
    }
    // A track that becomes next gets its peaks back.
    let p = model.players[0].id;
    let last = model.playlists.iter().next().unwrap().entries[2].clone();
    r.handle.send(Command::SetNext(p, last.id));
    r.run_until("peaks for the new next", |r| r.media.contains(last.track));
}

#[test]
fn a_panicking_step_does_not_stop_autosave() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = rig(&[], dir);
    // The analysis part fails on every step from now on.
    r.services.fail_steps(u32::MAX);
    r.services.step(r.now);
    assert_eq!(r.services.faults().load(Ordering::SeqCst), 1);
    r.handle.send(Command::CreatePlaylist {
        name: "After".into(),
    });
    r.conductor.tick(r.now);
    r.services.step(r.now);
    r.now += Duration::from_millis(1_100);
    r.conductor.tick(r.now);
    r.services.step(r.now);
    let saved = std::fs::read_to_string(r.paths.playlists_file()).unwrap_or_default();
    assert!(saved.contains("After"));
}

#[test]
fn cart_pages_are_saved_after_the_debounce() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = rig(&[], dir);
    r.handle.send(Command::CreateCartPage {
        name: "Sports".into(),
    });
    r.conductor.tick(r.now);
    r.services.step(r.now);
    r.now += Duration::from_millis(1_100);
    r.conductor.tick(r.now);
    r.services.step(r.now);
    let saved = std::fs::read_to_string(r.paths.carts_file()).unwrap_or_default();
    assert!(saved.contains("Sports"), "{saved}");
}

#[test]
fn resetting_markers_analyses_the_track_again() {
    let dir = tempfile::tempdir().unwrap();
    let a = wav(dir.path(), "a.wav", 2);
    let mut r = rig(&[a], dir);
    r.run_until("analysis", |r| {
        r.handle.model.load().library.iter().all(|t| t.analyzed)
    });
    let track = r.handle.model.load().library.iter().next().unwrap().id;
    r.handle.send(Command::ResetMarkers { track });
    r.run_until("a second analysis", |r| {
        r.analyses.load(Ordering::SeqCst) >= 2
    });
    r.run_until("analysed again", |r| {
        r.handle.model.load().library.iter().all(|t| t.analyzed)
    });
}

/// A library analysed by an older version: analysed, `older` says how.
fn outdated_rig(files: &[PathBuf], dir: tempfile::TempDir, older: fn(&mut fp_model::Track)) -> Rig {
    rig_with(files, dir, Duration::ZERO, move |state| {
        for track in state.library.iter_mut() {
            track.analyzed = true;
            track.duration_secs = 1.0;
            track.format = Some(fp_model::AudioFormat {
                sample_rate: 44_100,
                bits: Some(16),
                channels: 2,
                dsd_rate: None,
                bitrate: None,
            });
            older(track);
        }
    })
}

fn current(t: &fp_model::Track) -> bool {
    t.format.is_some() && t.analysis_version == fp_analysis::cache::ANALYSIS_VERSION
}

/// Re-analysing a whole library takes the processor for a while on an
/// on-air machine: tracks analysed by an older version keep their analysis
/// until the operator asks (the start-up notice, or Settings > Analysis).
/// Only the tracks on screen, which need their waveform, are analysed.
fn older_tracks_wait_for_the_operator(older: fn(&mut fp_model::Track)) {
    let dir = tempfile::tempdir().unwrap();
    let files: Vec<PathBuf> = (0..6)
        .map(|n| wav(dir.path(), &format!("{n}.wav"), 1))
        .collect();
    let mut r = outdated_rig(&files, dir, older);
    for _ in 0..150 {
        r.conductor.tick(r.now);
        r.services.step(r.now);
        let _ = r.device.render(480);
        r.now += Duration::from_millis(10);
        std::thread::sleep(Duration::from_millis(2));
    }
    let left = r
        .handle
        .model
        .load()
        .library
        .iter()
        .filter(|t| !current(t))
        .count();
    assert!(left >= 4, "only the shown tracks are analysed; {left} left");
    assert_eq!(
        fp_app::services::outdated_tracks(&r.handle.model.load()),
        left
    );
    assert!(
        r.services
            .requests()
            .try_send(ServiceRequest::AnalyseOutdated)
            .is_ok()
    );
    r.run_until("every track at the current version", |r| {
        r.handle.model.load().library.iter().all(current)
    });
    assert_eq!(fp_app::services::outdated_tracks(&r.handle.model.load()), 0);
}

#[test]
fn tracks_of_an_older_analysis_version_wait_for_the_operator() {
    older_tracks_wait_for_the_operator(|t| t.analysis_version = 0);
}

#[test]
fn tracks_analysed_before_formats_existed_wait_for_the_operator() {
    older_tracks_wait_for_the_operator(|t| t.format = None);
}

/// A cart bus that plays bit-perfect needs the format of its files, so a
/// cart's track that was analysed before formats were recorded is analysed
/// again at once, even though the operator said "Later" for the library.
#[test]
fn a_cart_track_without_a_format_is_analysed_even_when_the_operator_said_later() {
    let dir = tempfile::tempdir().unwrap();
    let files: Vec<PathBuf> = (0..6)
        .map(|n| wav(dir.path(), &format!("{n}.wav"), 1))
        .collect();
    let jingle = wav(dir.path(), "jingle.wav", 1);
    let mut r = rig_with(&files, dir, Duration::ZERO, |state| {
        let page = state.cartwall.pages.first().unwrap().id;
        fp_model::apply(
            state,
            Command::AssignCartFile {
                page,
                index: 0,
                path: jingle.clone(),
            },
        )
        .unwrap();
        for track in state.library.iter_mut() {
            track.analyzed = true;
            track.duration_secs = 1.0;
            track.format = None;
            track.analysis_version = fp_analysis::cache::ANALYSIS_VERSION;
        }
        // Not on screen: only the format rule can analyse it.
        state.cartwall.open = false;
    });
    let cart_track = r.handle.model.load().cartwall.pages[0].carts[0]
        .track
        .unwrap();
    r.run_until("the cart's track has its format", |r| {
        r.handle
            .model
            .load()
            .library
            .get(cart_track)
            .is_some_and(|t| t.format.is_some())
    });
    // "Later" still holds for the playlist tracks that are not on screen.
    let waiting = r
        .handle
        .model
        .load()
        .library
        .iter()
        .filter(|t| t.id != cart_track && t.format.is_none())
        .count();
    assert!(
        waiting >= 4,
        "only the cart's track jumps the queue; {waiting} wait"
    );
}

/// A cart track that already has its format, or whose file is gone, is not
/// analysed again, and nothing loops.
#[test]
fn a_cart_track_that_has_its_format_is_left_alone() {
    let dir = tempfile::tempdir().unwrap();
    let jingle = wav(dir.path(), "jingle.wav", 1);
    let mut r = rig_with(&[], dir, Duration::ZERO, |state| {
        let page = state.cartwall.pages.first().unwrap().id;
        fp_model::apply(
            state,
            Command::AssignCartFile {
                page,
                index: 0,
                path: jingle.clone(),
            },
        )
        .unwrap();
        for track in state.library.iter_mut() {
            track.analyzed = true;
            track.duration_secs = 1.0;
            // An older version's analysis, but with its format: it waits.
            track.format = Some(fp_model::AudioFormat {
                sample_rate: 48_000,
                bits: Some(16),
                channels: 1,
                dsd_rate: None,
                bitrate: None,
            });
            track.analysis_version = 0;
        }
        // Not on screen: no read for the waveform either.
        state.cartwall.open = false;
    });
    for _ in 0..100 {
        r.conductor.tick(r.now);
        r.services.step(r.now);
        r.now += Duration::from_millis(10);
        std::thread::sleep(Duration::from_millis(2));
    }
    assert_eq!(r.analyses.load(Ordering::SeqCst), 0);
}

#[test]
fn a_cart_track_whose_file_is_missing_is_not_analysed() {
    let dir = tempfile::tempdir().unwrap();
    let gone = dir.path().join("gone.wav");
    let mut r = rig_with(&[], dir, Duration::ZERO, |state| {
        let page = state.cartwall.pages.first().unwrap().id;
        fp_model::apply(
            state,
            Command::AssignCartFile {
                page,
                index: 0,
                path: gone.clone(),
            },
        )
        .unwrap();
        for track in state.library.iter_mut() {
            track.analyzed = true;
            track.format = None;
            track.file_state = FileState::Missing;
        }
    });
    for _ in 0..100 {
        r.conductor.tick(r.now);
        r.services.step(r.now);
        r.now += Duration::from_millis(10);
        std::thread::sleep(Duration::from_millis(2));
    }
    assert_eq!(r.analyses.load(Ordering::SeqCst), 0);
}

#[test]
fn a_result_lost_to_a_panic_is_asked_for_again() {
    let dir = tempfile::tempdir().unwrap();
    let a = wav(dir.path(), "Band - One.wav", 1);
    let mut r = rig(&[a], dir);
    // Applying the first result panics: that result is lost.
    r.services.fail_routes(1);
    r.run_until("analysis", |r| {
        r.handle.model.load().library.iter().all(|t| t.analyzed)
    });
    assert_eq!(r.services.faults().load(Ordering::SeqCst), 1);
    assert_eq!(r.analyses.load(Ordering::SeqCst), 2, "analysed again once");
}

/// A track whose file is gone cannot be analysed again: counting it would
/// bring the start-up notice back at every start, forever.
#[test]
fn an_outdated_track_whose_file_is_missing_is_not_counted() {
    let dir = tempfile::tempdir().unwrap();
    let present = wav(dir.path(), "present.wav", 1);
    let missing = PathBuf::from("/definitely/missing.flac");
    let mut r = outdated_rig(&[present, missing], dir, |t| t.analysis_version = 0);
    assert!(
        r.services
            .requests()
            .try_send(ServiceRequest::AnalyseOutdated)
            .is_ok()
    );
    r.run_until("the present track analysed, the other marked", |r| {
        let state = r.handle.model.load();
        state.library.iter().any(current)
            && state
                .library
                .iter()
                .any(|t| t.file_state == FileState::Missing)
    });
    assert_eq!(fp_app::services::outdated_tracks(&r.handle.model.load()), 0);
}

/// Files not found are looked for again every second here.
fn recheck_rig(files: &[PathBuf], dir: tempfile::TempDir) -> Rig {
    rig_with(files, dir, Duration::ZERO, |s| {
        s.config.tuning.missing_recheck_ms = 1_000.0;
    })
}

fn missing(r: &Rig) -> bool {
    r.handle
        .model
        .load()
        .library
        .iter()
        .all(|t| t.file_state == FileState::Missing)
}

/// A drive mounted after the start: its tracks come back by themselves.
#[test]
fn a_missing_file_that_comes_back_becomes_playable() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("later.wav");
    let staging = tempfile::tempdir().unwrap();
    let mut r = recheck_rig(std::slice::from_ref(&path), dir);
    r.run_until("missing state", missing);
    // Written elsewhere, then moved: never seen half-written.
    std::fs::rename(wav(staging.path(), "later.wav", 1), &path).unwrap();
    r.run_until("the file found again", |r| {
        r.handle
            .model
            .load()
            .library
            .iter()
            .all(|t| t.file_state == FileState::Ok && t.analyzed)
    });
}

/// Looking again for a file still missing changes nothing in the model, so
/// nothing is saved every interval, and keeps it out of the analysis pool:
/// a library on a share that is offline must not hold up real analyses.
#[test]
fn a_file_still_missing_is_looked_for_without_touching_the_model_or_the_pool() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = recheck_rig(&[PathBuf::from("/definitely/missing.flac")], dir);
    r.run_until("missing state", missing);
    let analyses = r.analyses.load(Ordering::SeqCst);
    let version = r.handle.telemetry.load().model_version;
    let until = r.now + Duration::from_millis(3_500);
    r.run_until("three intervals", |r| r.now >= until);
    assert_eq!(r.analyses.load(Ordering::SeqCst), analyses);
    assert_eq!(r.handle.telemetry.load().model_version, version);
}

/// After looks that found nothing, the file is still looked for.
#[test]
fn a_file_is_still_looked_for_after_looks_that_found_nothing() {
    let dir = tempfile::tempdir().unwrap();
    // Two folders that do not exist yet, as on a drive not mounted.
    let path = dir.path().join("drive/album/late.wav");
    let staging = tempfile::tempdir().unwrap();
    let root = dir.path().join("drive");
    let mut r = recheck_rig(std::slice::from_ref(&path), dir);
    r.run_until("missing state", missing);
    let until = r.now + Duration::from_millis(3_500);
    r.run_until("three looks", |r| r.now >= until);
    std::fs::create_dir_all(root.join("album")).unwrap();
    std::fs::rename(wav(staging.path(), "late.wav", 1), &path).unwrap();
    r.run_until("the file found again", |r| {
        r.handle
            .model
            .load()
            .library
            .iter()
            .all(|t| t.file_state == FileState::Ok)
    });
}

/// How many analyses started before the first one of `path`.
fn started_before(r: &Rig, path: &Path) -> usize {
    let started = r.started.lock().unwrap();
    started
        .iter()
        .position(|p| p == path)
        .unwrap_or_else(|| panic!("{} was never analysed: {started:?}", path.display()))
}

/// A track loaded on a player gets its waveform ahead of the library being
/// analysed, not after it.
#[test]
fn a_track_on_a_player_is_analysed_ahead_of_the_library() {
    let dir = tempfile::tempdir().unwrap();
    let files: Vec<PathBuf> = (1..=12)
        .map(|n| wav(dir.path(), &format!("{n}.wav"), 1))
        .collect();
    let mut r = rig_with(&files, dir, Duration::ZERO, |s| {
        let p = s.players[0].id;
        let last = s.playlists.iter().next().unwrap().entries[11].id;
        fp_model::apply(s, Command::SetNext(p, last)).unwrap();
    });
    let model = r.handle.model.load_full();
    let next = model.players[0].next.unwrap();
    let track = model.playlists.entry(next).unwrap().track;
    // The first round queues the whole library and the loaded track; the
    // worker holds whatever it took first until everything is queued, so
    // the order below is the queue's, not the scheduler's.
    r.gate.store(false, Ordering::SeqCst);
    r.conductor.tick(r.now);
    r.services.step(r.now);
    r.gate.store(true, Ordering::SeqCst);
    r.run_until("peaks for the loaded track", |r| r.media.contains(track));
    // At most the job the worker had already taken goes first.
    let before = started_before(&r, &files[11]);
    assert!(before <= 1, "{before} analyses started first");
}

/// A track loaded while it waits in the queue with the library moves ahead.
#[test]
fn a_queued_track_loaded_on_a_player_moves_ahead() {
    let dir = tempfile::tempdir().unwrap();
    let files: Vec<PathBuf> = (1..=12)
        .map(|n| wav(dir.path(), &format!("{n}.wav"), 1))
        .collect();
    let mut r = rig(&files, dir);
    // The worker holds its first job: the rest of the library stays queued
    // however slowly the test runs.
    r.gate.store(false, Ordering::SeqCst);
    r.run_until("the library queued", |r| {
        r.analyses.load(Ordering::SeqCst) >= 1
    });
    let model = r.handle.model.load_full();
    let player = model.players[0].id;
    let last = model.playlists.iter().next().unwrap().entries[11].clone();
    r.handle.send(Command::SetNext(player, last.id));
    r.run_until("the track loaded", |r| {
        r.handle.model.load().players[0].next == Some(last.id)
    });
    // A round that sees it on the player.
    r.services.step(r.now);
    r.gate.store(true, Ordering::SeqCst);
    r.run_until("peaks for the loaded track", |r| {
        r.media.contains(last.track)
    });
    // Right after the job that was running when it was loaded.
    let before = started_before(&r, &files[11]);
    assert_eq!(before, 1, "{before} analyses started first");
}

/// Gives `path` a title, a genre and a comment, as a tagging tool would.
fn tag_file(path: &Path) {
    use lofty::prelude::*;
    let mut file = lofty::read_from_path(path).unwrap();
    let ty = file.primary_tag_type();
    file.insert_tag(lofty::tag::Tag::new(ty));
    let tag = file.primary_tag_mut().unwrap();
    tag.set_title("Tagged".into());
    tag.set_genre("Jazz".into());
    tag.set_comment("Take two".into());
    file.save_to_path(path, lofty::config::WriteOptions::default())
        .unwrap();
}

fn only_track(r: &Rig) -> fp_model::Track {
    r.handle.model.load().library.iter().next().unwrap().clone()
}

#[test]
fn the_tag_pass_fills_the_tags_of_analysed_tracks_once() {
    let dir = tempfile::tempdir().unwrap();
    let a = wav(dir.path(), "a.wav", 1);
    tag_file(&a);
    let mut r = rig(&[a], dir);
    r.run_until("the tags", |r| only_track(r).tags_read);
    let t = only_track(&r);
    assert_eq!(
        (t.title.as_str(), t.genre.as_str(), t.comment.as_str()),
        ("Tagged", "Jazz", "Take two")
    );
    assert!(t.analyzed);
    let version = r.handle.telemetry.load().model_version;
    let until = r.now + Duration::from_millis(500);
    r.run_until("a quiet period", |r| r.now >= until);
    // First: a second analysis would also bring a second read.
    assert_eq!(r.analyses.load(Ordering::SeqCst), 1, "no second analysis");
    assert_eq!(
        r.handle.telemetry.load().model_version,
        version,
        "no further ApplyTags once the tags are read"
    );
}

#[test]
fn a_track_whose_tags_cannot_be_read_is_not_asked_again() {
    let dir = tempfile::tempdir().unwrap();
    let junk = dir.path().join("Band - Junk.wav");
    std::fs::write(&junk, b"this is not audio").unwrap();
    // Analysed already, with its tags still unread (and its loudness
    // measured, so the loudness pass sends nothing during the count).
    let mut r = rig_with(&[junk], dir, Duration::ZERO, |state| {
        for track in state.library.iter_mut() {
            track.analyzed = true;
            track.apply_loudness(None, fp_model::LOUDNESS_VERSION);
            track.format = Some(fp_model::AudioFormat {
                sample_rate: 44_100,
                bits: Some(16),
                channels: 2,
                dsd_rate: None,
                bitrate: None,
            });
            track.analysis_version = fp_analysis::cache::ANALYSIS_VERSION;
        }
    });
    // It is the next track of a player: without its waveform on hand it
    // would be analysed for it, and that analysis (which finds the file
    // unreadable) could land at any time, before or after the tags.
    let track = only_track(&r).id;
    r.media.seed(
        track,
        TrackMedia {
            peaks: Vec::new(),
            peak_bucket_secs: 0.01,
            cover_png: None,
        },
    );
    r.run_until("the pass to answer", |r| only_track(r).tags_read);
    let t = only_track(&r);
    assert_eq!((t.title.as_str(), t.artist.as_str()), ("Junk", "Band"));
    let version = r.handle.telemetry.load().model_version;
    let until = r.now + Duration::from_millis(500);
    r.run_until("a quiet period", |r| r.now >= until);
    assert_eq!(r.handle.telemetry.load().model_version, version);
    assert_eq!(r.analyses.load(Ordering::SeqCst), 0, "no analysis ran");
}

#[test]
fn a_missing_track_is_not_read() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = rig_with(
        &[PathBuf::from("/definitely/missing.flac")],
        dir,
        Duration::ZERO,
        |state| {
            for track in state.library.iter_mut() {
                track.analyzed = true;
                track.file_state = FileState::Missing;
            }
        },
    );
    let until = r.now + Duration::from_millis(500);
    r.run_until("a quiet period", |r| r.now >= until);
    assert!(!only_track(&r).tags_read);
}

#[test]
fn a_new_analysis_result_brings_the_tag_pass_back() {
    let dir = tempfile::tempdir().unwrap();
    let a = wav(dir.path(), "a.wav", 1);
    tag_file(&a);
    let mut r = rig(std::slice::from_ref(&a), dir);
    r.run_until("the tags", |r| only_track(r).tags_read);
    // The operator edits the genre with another tool, then asks for a new
    // analysis: the file's new tags must reach the library too.
    {
        use lofty::prelude::*;
        let mut file = lofty::read_from_path(&a).unwrap();
        file.primary_tag_mut().unwrap().set_genre("Blues".into());
        file.save_to_path(&a, lofty::config::WriteOptions::default())
            .unwrap();
    }
    let track = only_track(&r).id;
    r.handle.send(Command::ApplyAnalysis {
        track,
        analysis: Box::new(fp_model::TrackAnalysis::default()),
    });
    r.run_until("the genre read again", |r| only_track(r).genre == "Blues");
    assert!(only_track(&r).tags_read);
}

#[test]
fn a_stale_snapshot_does_not_read_a_track_twice() {
    let dir = tempfile::tempdir().unwrap();
    let junk = dir.path().join("Band - Junk.wav");
    std::fs::write(&junk, b"this is not audio").unwrap();
    let mut r = rig_with(&[junk], dir, Duration::ZERO, |state| {
        for track in state.library.iter_mut() {
            track.analyzed = true;
            track.format = Some(fp_model::AudioFormat {
                sample_rate: 44_100,
                bits: Some(16),
                channels: 2,
                dsd_rate: None,
                bitrate: None,
            });
            track.analysis_version = fp_analysis::cache::ANALYSIS_VERSION;
        }
    });
    // The engine's first report (where each holder plays) is one model
    // version of its own: let it land before counting.
    r.conductor.tick(r.now);
    let version = r.handle.telemetry.load().model_version;
    // A snapshot that never shows the answer, as one taken before the
    // command lands.
    let stale = r.handle.model.load_full();
    assert!(!only_track(&r).tags_read);
    let until = Instant::now() + Duration::from_millis(400);
    while Instant::now() < until {
        r.services.tag_pass_on(&stale);
        // Each tick with a command is one new model version.
        r.conductor.tick(r.now);
        std::thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(
        r.handle.telemetry.load().model_version,
        version + 1,
        "exactly one ApplyTags"
    );
}

/// A file with an audio extension that cannot be decoded.
fn garbage(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, b"not audio at all").unwrap();
    path
}

fn all_in(r: &Rig, state: FileState) -> bool {
    r.handle
        .model
        .load()
        .library
        .iter()
        .all(|t| t.file_state == state)
}

/// The track is unreadable and its first look has been answered, so its
/// size and modification time are recorded.
fn unreadable_and_stamped(r: &Rig) -> bool {
    all_in(r, FileState::Unreadable) && r.services.looked_at() == 1
}

fn quiet_for(r: &mut Rig, ms: u64) {
    let until = r.now + Duration::from_millis(ms);
    r.run_until("a quiet period", |r| r.now >= until);
}

/// Q10.2: neither the size nor the modification time changed, so nothing
/// happens: no analysis, no save.
#[test]
fn an_unreadable_file_that_did_not_change_is_not_analysed_again() {
    let dir = tempfile::tempdir().unwrap();
    let bad = garbage(dir.path(), "bad.wav");
    let mut r = recheck_rig(&[bad], dir);
    r.run_until("unreadable and stamped", unreadable_and_stamped);
    let analyses = r.analyses.load(Ordering::SeqCst);
    let version = r.handle.telemetry.load().model_version;
    quiet_for(&mut r, 3_500);
    assert_eq!(r.analyses.load(Ordering::SeqCst), analyses);
    assert_eq!(
        r.handle.telemetry.load().model_version,
        version,
        "nothing is saved"
    );
}

/// Q10.2 and Q10.5: a fixed file comes back playable by itself.
#[test]
fn an_unreadable_file_that_changed_size_is_analysed_again() {
    let dir = tempfile::tempdir().unwrap();
    let bad = garbage(dir.path(), "bad.wav");
    let staging = tempfile::tempdir().unwrap();
    let mut r = recheck_rig(std::slice::from_ref(&bad), dir);
    r.run_until("unreadable and stamped", unreadable_and_stamped);
    // Written elsewhere, then moved: never seen half-written.
    std::fs::rename(wav(staging.path(), "bad.wav", 1), &bad).unwrap();
    r.run_until("playable again", |r| {
        r.handle
            .model
            .load()
            .library
            .iter()
            .all(|t| t.file_state == FileState::Ok && t.analyzed)
    });
    assert_eq!(
        r.analyses.load(Ordering::SeqCst),
        2,
        "one failure and one success"
    );
}

/// The same size with a new modification time is a change too, and a file
/// that fails again is looked at with its new stat: no loop.
#[test]
fn a_file_that_fails_again_is_looked_at_with_its_new_stat() {
    let dir = tempfile::tempdir().unwrap();
    let bad = garbage(dir.path(), "bad.wav");
    let mut r = recheck_rig(std::slice::from_ref(&bad), dir);
    r.run_until("unreadable and stamped", unreadable_and_stamped);
    let file = std::fs::OpenOptions::new().write(true).open(&bad).unwrap();
    file.set_modified(SystemTime::now() + Duration::from_secs(60))
        .unwrap();
    drop(file);
    r.run_until("a second analysis", |r| {
        r.analyses.load(Ordering::SeqCst) == 2
    });
    r.run_until("stamped again", unreadable_and_stamped);
    quiet_for(&mut r, 3_500);
    assert_eq!(r.analyses.load(Ordering::SeqCst), 2, "no third analysis");
    assert!(all_in(&r, FileState::Unreadable));
}

/// Q10.1: the stat recorded is the one of the file the analysis read, not
/// the one at the first look. A copy that completes while the analysis runs
/// is analysed again.
#[test]
fn a_file_that_changes_while_its_analysis_fails_is_analysed_again() {
    let dir = tempfile::tempdir().unwrap();
    let bad = garbage(dir.path(), "bad.wav");
    let staging = tempfile::tempdir().unwrap();
    let complete = wav(staging.path(), "bad.wav", 1);
    let once = std::sync::Mutex::new(Some(complete));
    // After the first analysis failed, before its result is known: the
    // copy completes.
    let after: AfterAnalysis = Arc::new(move |path| {
        if let Some(complete) = once.lock().unwrap().take() {
            std::fs::rename(complete, path).unwrap();
        }
    });
    let mut r = rig_hooked(
        std::slice::from_ref(&bad),
        dir,
        Duration::ZERO,
        |s| s.config.tuning.missing_recheck_ms = 1_000.0,
        after,
    );
    r.run_until("playable again", |r| {
        r.handle
            .model
            .load()
            .library
            .iter()
            .all(|t| t.file_state == FileState::Ok && t.analyzed)
    });
    assert_eq!(r.analyses.load(Ordering::SeqCst), 2);
}

/// Q10.3: it follows the missing-file recheck from then on.
#[test]
fn an_unreadable_file_that_disappeared_becomes_missing() {
    let dir = tempfile::tempdir().unwrap();
    let bad = garbage(dir.path(), "bad.wav");
    let mut r = recheck_rig(std::slice::from_ref(&bad), dir);
    r.run_until("unreadable and stamped", unreadable_and_stamped);
    std::fs::remove_file(&bad).unwrap();
    r.run_until("missing", |r| all_in(r, FileState::Missing));
}

/// Q10.3: a file found gone is reported once, not again on each look while
/// the snapshot still says unreadable.
#[test]
fn an_unreadable_file_that_disappeared_is_reported_once() {
    let dir = tempfile::tempdir().unwrap();
    let bad = garbage(dir.path(), "bad.wav");
    let mut r = recheck_rig(std::slice::from_ref(&bad), dir);
    r.run_until("unreadable and stamped", unreadable_and_stamped);
    std::fs::remove_file(&bad).unwrap();
    r.run_until("missing", |r| all_in(r, FileState::Missing));
    quiet_for(&mut r, 2_500);
    assert_eq!(r.services.gone_reports(), 1);
}

/// Q10.4: the row menu's Re-analyse does not wait for a change, forgets the
/// failure and goes ahead of the library.
#[test]
fn reanalyse_track_analyses_a_failed_file_at_once_whatever_its_state() {
    let dir = tempfile::tempdir().unwrap();
    let bad = garbage(dir.path(), "bad.wav");
    let mut files = vec![bad.clone()];
    files.extend((1..=6).map(|n| wav(dir.path(), &format!("{n}.wav"), 1)));
    // The analyses in the order they end. Once the bad file has failed, the
    // gate closes: the next library job holds and the rest of the library
    // stays queued until the operator has asked, however slowly the test
    // runs.
    let order = Arc::new(std::sync::Mutex::new(Vec::<PathBuf>::new()));
    let gate = Arc::new(std::sync::OnceLock::<Arc<AtomicBool>>::new());
    let (seen, close) = (order.clone(), gate.clone());
    let failing = bad.clone();
    let after: AfterAnalysis = Arc::new(move |path| {
        let mut seen = seen.lock().unwrap();
        if path == failing && !seen.contains(&failing) {
            close.get().unwrap().store(false, Ordering::SeqCst);
        }
        seen.push(path.to_path_buf());
    });
    let mut r = rig_hooked(
        &files,
        dir,
        Duration::ZERO,
        |s| {
            s.config.tuning.missing_recheck_ms = 1_000.0;
            // The failed file is on no player, so only Re-analyse makes it
            // urgent.
            let second = s.playlists.iter().next().unwrap().entries[1].id;
            let players: Vec<_> = s.players.iter().map(|p| p.id).collect();
            for p in players {
                fp_model::apply(s, Command::SetNext(p, second)).unwrap();
            }
        },
        after,
    );
    gate.set(r.gate.clone()).unwrap();
    let track = r
        .handle
        .model
        .load()
        .library
        .iter()
        .find(|t| t.path == bad)
        .unwrap()
        .id;
    r.run_until("unreadable and stamped", |r| {
        r.handle.model.load().library.get(track).unwrap().file_state == FileState::Unreadable
            && r.services.looked_at() == 1
    });
    r.services
        .requests()
        .send(ServiceRequest::ReanalyseTrack(track))
        .unwrap();
    r.conductor.tick(r.now);
    r.services.step(r.now);
    assert_eq!(
        r.services.looked_at(),
        0,
        "the recorded failure is forgotten"
    );
    r.gate.store(true, Ordering::SeqCst);
    let twice = |order: &[PathBuf]| order.iter().filter(|p| **p == bad).count() == 2;
    r.run_until("a second analysis", |_| twice(&order.lock().unwrap()));
    // Before the second start: at most the track on the players (urgent,
    // it may go before the first failure) and the job held at the gate.
    let started = r.started.lock().unwrap().clone();
    let second = started
        .iter()
        .enumerate()
        .filter(|(_, p)| **p == bad)
        .nth(1)
        .unwrap()
        .0;
    let goods = started[..second].iter().filter(|p| **p != bad).count();
    assert!(goods <= 2, "{goods} library files were analysed first");
    r.run_until("stamped again", |r| r.services.looked_at() == 1);
}

/// The urgent mark of a Re-analyse is used once, even when the track went
/// to the urgent queue anyway (it is on a player).
#[test]
fn reanalysing_a_track_on_a_player_leaves_no_urgent_mark() {
    let dir = tempfile::tempdir().unwrap();
    let good = wav(dir.path(), "good.wav", 1);
    let mut r = rig_with(&[good], dir, Duration::ZERO, |s| {
        let p = s.players[0].id;
        let first = s.playlists.iter().next().unwrap().entries[0].id;
        fp_model::apply(s, Command::SetNext(p, first)).unwrap();
    });
    r.run_until("analysed", |r| {
        r.handle.model.load().library.iter().all(|t| t.analyzed)
    });
    let track = r.handle.model.load().library.iter().next().unwrap().id;
    r.services
        .requests()
        .send(ServiceRequest::ReanalyseTrack(track))
        .unwrap();
    r.run_until("a second analysis", |r| {
        r.analyses.load(Ordering::SeqCst) == 2
    });
    assert_eq!(r.services.urgent_marks(), 0);
}

/// Q10.1 and Q10.4: a track that playback marked unreadable is looked at
/// too, and Re-analyse makes it playable again.
#[test]
fn reanalyse_track_brings_back_a_track_that_playback_marked_unreadable() {
    let dir = tempfile::tempdir().unwrap();
    let good = wav(dir.path(), "good.wav", 1);
    let mut r = recheck_rig(&[good], dir);
    r.run_until("analysed", |r| {
        r.handle.model.load().library.iter().all(|t| t.analyzed)
    });
    let track = r.handle.model.load().library.iter().next().unwrap().id;
    // What a playback failure does (the reducer's `SourceFailed`).
    assert!(r.handle.send(Command::SetFileState {
        track,
        state: FileState::Unreadable
    }));
    r.run_until("unreadable and stamped", unreadable_and_stamped);
    r.services
        .requests()
        .send(ServiceRequest::ReanalyseTrack(track))
        .unwrap();
    r.run_until("playable again", |r| all_in(r, FileState::Ok));
    assert_eq!(r.analyses.load(Ordering::SeqCst), 2);
}

/// Operator feedback 4, Q1.1: a track has the length its file's header
/// declares long before its analysis ends.
#[test]
fn a_track_gets_its_header_duration_before_its_analysis() {
    let dir = tempfile::tempdir().unwrap();
    let file = wav(dir.path(), "a.wav", 2);
    let mut r = rig_slow(&[file], dir, Duration::from_secs(30));
    r.run_until("the header duration", |r| only_track(r).duration_secs > 0.0);
    let t = only_track(&r);
    assert!(!t.analyzed, "the analysis is still running");
    assert!((t.duration_secs - 2.0).abs() < 1e-6, "{}", t.duration_secs);
}

/// A file whose header cannot be read gets no length, and is asked once.
#[test]
fn a_file_whose_header_cannot_be_read_gets_no_duration_and_is_asked_once() {
    let dir = tempfile::tempdir().unwrap();
    let broken = dir.path().join("broken.wav");
    std::fs::write(&broken, b"RIFF\x00\x00\x00\x00WAVEjunkjunkjunkjunk").unwrap();
    let mut r = rig_slow(&[broken], dir, Duration::from_secs(30));
    let mut rounds = 0;
    r.run_until("fifty rounds", |_| {
        rounds += 1;
        rounds > 50
    });
    assert_eq!(only_track(&r).duration_secs, 0.0);
    assert_eq!(r.services.header_reads_sent(), 1);
}

/// A length that comes back after the analysis has landed is dropped: the
/// model would ignore it, but the command would still make a new model
/// version (a new snapshot and a save for nothing).
#[test]
fn a_header_length_read_after_the_analysis_changes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let file = wav(dir.path(), "a.wav", 2);
    // The real analysis is held back; the test applies its own.
    let mut r = rig_slow(&[file], dir, Duration::from_secs(30));
    // The first round asks for the header, after collecting answers: its
    // answer can only be taken on a later round.
    r.services.step(r.now);
    assert_eq!(r.services.header_reads_sent(), 1);
    let track = only_track(&r).id;
    r.handle.send(Command::ApplyAnalysis {
        track,
        analysis: Box::new(fp_model::TrackAnalysis {
            duration_secs: 2.0,
            ..Default::default()
        }),
    });
    // Its tags too, in the same tick, so the tag pass has nothing to send
    // meanwhile: the late length is the only command that could come.
    r.handle.send(Command::ApplyTags {
        track,
        tags: Box::default(),
    });
    r.conductor.tick(r.now);
    let t = only_track(&r);
    assert!(t.analyzed && t.tags_read);
    let version = r.handle.telemetry.load().model_version;
    let deadline = Instant::now() + Duration::from_secs(15);
    while r.services.header_reads_in_flight() > 0 {
        assert!(Instant::now() < deadline, "timed out waiting for the read");
        r.services.step(r.now);
        r.now += Duration::from_millis(10);
        std::thread::sleep(Duration::from_millis(2));
    }
    r.conductor.tick(r.now);
    assert_eq!(
        r.handle.telemetry.load().model_version,
        version,
        "a late length reached the model"
    );
}

/// A length read while the conductor's queue is full is kept and sent
/// later, not read from the file again.
#[test]
fn a_header_length_that_cannot_be_queued_is_sent_later_without_a_second_read() {
    let dir = tempfile::tempdir().unwrap();
    let file = wav(dir.path(), "a.wav", 2);
    let mut r = rig_slow(&[file], dir, Duration::from_secs(30));
    let p = r.handle.model.load().players[0].id;
    while r.handle.send(Command::SetVolume(p, 1.0)) {}
    // Only the services run: the queue stays full while the answer comes.
    let deadline = Instant::now() + Duration::from_secs(15);
    while !(r.services.header_reads_sent() >= 1 && r.services.header_reads_in_flight() == 0
        || r.services.header_reads_sent() > 1)
    {
        assert!(Instant::now() < deadline, "timed out waiting for the read");
        r.services.step(r.now);
        r.now += Duration::from_millis(10);
        std::thread::sleep(Duration::from_millis(2));
    }
    assert_eq!(r.services.header_reads_sent(), 1, "read again");
    r.run_until("the header duration", |r| only_track(r).duration_secs > 0.0);
    assert!((only_track(&r).duration_secs - 2.0).abs() < 1e-6);
    assert_eq!(r.services.header_reads_sent(), 1);
}

/// Tracks on a player get their length ahead of a large library.
#[test]
fn a_track_on_a_player_gets_its_header_duration_ahead_of_the_library() {
    let dir = tempfile::tempdir().unwrap();
    let files: Vec<PathBuf> = (1..=30)
        .map(|n| wav(dir.path(), &format!("{n}.wav"), 1))
        .collect();
    let mut r = rig_with(&files, dir, Duration::from_secs(30), |s| {
        let p = s.players[0].id;
        let last = s.playlists.iter().next().unwrap().entries[29].id;
        fp_model::apply(s, Command::SetNext(p, last)).unwrap();
    });
    let model = r.handle.model.load_full();
    let next = model.players[0].next.unwrap();
    let track = model.playlists.entry(next).unwrap().track;
    r.run_until("the loaded track's length", |r| {
        r.handle
            .model
            .load()
            .library
            .get(track)
            .is_some_and(|t| t.duration_secs > 0.0)
    });
    let known = r
        .handle
        .model
        .load()
        .library
        .iter()
        .filter(|t| t.duration_secs > 0.0)
        .count();
    // At most two rounds of eight reads were answered by then.
    assert!(known <= 16, "{known} lengths were read first");
}

#[test]
fn new_limits_reach_the_analyzer() {
    let dir = tempfile::tempdir().unwrap();
    let a = wav(dir.path(), "a.wav", 1);
    let mut r = rig(&[a], dir);
    r.run_until("analysis", |r| {
        r.handle.model.load().library.iter().all(|t| t.analyzed)
    });
    let analysed = r.analyses.load(Ordering::SeqCst);
    let mut config = r.handle.model.load().config.clone();
    config.limits.max_cover_pixels = 1_000;
    r.handle.send(Command::UpdateConfig(Box::new(config)));
    r.run_until("the analyzer has the new limits", |r| {
        r.services.analyzer_limits().max_cover_pixels == 1_000
    });
    assert_eq!(
        r.analyses.load(Ordering::SeqCst),
        analysed,
        "the library is not analysed again"
    );
}

/// Points every player at a new, empty playlist, so that no track is on
/// screen: a track a player shows gets a full analysis for its waveform
/// (`Services::wanted`), which would measure it before the pass could.
fn off_screen(state: &mut fp_model::AppState) {
    fp_model::apply(
        state,
        Command::CreatePlaylist {
            name: "Other".into(),
        },
    )
    .unwrap();
    let other = state.playlists.iter().last().unwrap().id;
    for p in &mut state.players {
        p.playlist = other;
        p.next = None;
        p.next_explicit = false;
    }
}

/// Marks every track as analysed by the previous release: markers and
/// length known, no loudness measured yet; no track on screen.
fn analysed_before_loudness(state: &mut fp_model::AppState) {
    off_screen(state);
    for t in state.library.iter_mut() {
        t.analyzed = true;
        t.analysis_version = fp_analysis::cache::ANALYSIS_VERSION;
        t.tags_read = true;
        t.duration_secs = 2.0;
    }
}

#[test]
fn n6b_old_tracks_are_measured_once_without_a_full_analysis() {
    let dir = tempfile::tempdir().unwrap();
    let a = wav(dir.path(), "a.wav", 2);
    let mut r = rig_with(&[a], dir, Duration::ZERO, analysed_before_loudness);
    r.run_until("the loudness", |r| only_track(r).loudness_pass.is_some());
    let t = only_track(&r);
    assert!(t.loudness.and_then(|l| l.integrated_lufs).is_some());
    assert_eq!(t.loudness_pass, Some(fp_model::LOUDNESS_VERSION));
    let until = r.now + Duration::from_millis(500);
    r.run_until("a quiet period", |r| r.now >= until);
    assert_eq!(r.analyses.load(Ordering::SeqCst), 0, "no full analysis");
    assert_eq!(r.loudness_runs.load(Ordering::SeqCst), 1, "measured once");
}

#[test]
fn n6b_a_new_file_is_measured_by_its_analysis_not_by_the_pass() {
    let dir = tempfile::tempdir().unwrap();
    let a = wav(dir.path(), "a.wav", 2);
    let mut r = rig(&[a], dir);
    r.run_until("the analysis", |r| only_track(r).analyzed);
    let until = r.now + Duration::from_millis(500);
    r.run_until("a quiet period", |r| r.now >= until);
    assert!(only_track(&r).loudness.is_some());
    assert_eq!(r.loudness_runs.load(Ordering::SeqCst), 0);
}

#[test]
fn the_pass_waits_for_normal_analysis() {
    let dir = tempfile::tempdir().unwrap();
    let old = wav(dir.path(), "old.wav", 1);
    let new = wav(dir.path(), "new.wav", 1);
    // `old` was analysed by the previous release, `new` was just added.
    let old_path = old.clone();
    let mut r = rig_with(&[old, new], dir, Duration::from_millis(300), move |s| {
        off_screen(s);
        for t in s.library.iter_mut().filter(|t| t.path == old_path) {
            t.analyzed = true;
            t.analysis_version = fp_analysis::cache::ANALYSIS_VERSION;
            t.tags_read = true;
        }
    });
    r.run_until("both measured", |r| {
        r.handle
            .model
            .load()
            .library
            .iter()
            .all(|t| t.loudness_pass.is_some())
    });
    assert_eq!(r.loudness_runs.load(Ordering::SeqCst), 1);
    assert_eq!(
        r.seen_finished.load(Ordering::SeqCst),
        1,
        "the loudness job started after the new file's analysis"
    );
}

#[test]
fn n6b_a_short_file_is_recorded_and_not_measured_again() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("blip.wav");
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 48_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(&path, spec).unwrap();
    for i in 0..9_600 {
        w.write_sample(((i as f32 * 0.05).sin() * 8_000.0) as i16)
            .unwrap();
    }
    w.finalize().unwrap();
    let mut r = rig_with(&[path], dir, Duration::ZERO, analysed_before_loudness);
    r.run_until("the attempt", |r| only_track(r).loudness_pass.is_some());
    assert_eq!(only_track(&r).loudness, None);
    let until = r.now + Duration::from_millis(500);
    r.run_until("a quiet period", |r| r.now >= until);
    assert_eq!(r.loudness_runs.load(Ordering::SeqCst), 1);
}

#[test]
fn n6b_an_unreadable_file_is_recorded_and_not_measured_again() {
    let dir = tempfile::tempdir().unwrap();
    let junk = garbage(dir.path(), "junk.wav");
    let mut r = rig_with(&[junk], dir, Duration::ZERO, analysed_before_loudness);
    r.run_until("the attempt", |r| only_track(r).loudness_pass.is_some());
    assert_eq!(only_track(&r).loudness, None);
    let until = r.now + Duration::from_millis(500);
    r.run_until("a quiet period", |r| r.now >= until);
    assert_eq!(r.loudness_runs.load(Ordering::SeqCst), 1);
}

#[test]
fn n6b_a_missing_file_is_marked_missing_not_recorded() {
    // Gone since its analysis: it is measured when it comes back.
    let dir = tempfile::tempdir().unwrap();
    let mut r = rig_with(
        &[dir.path().join("gone.wav")],
        dir,
        Duration::ZERO,
        analysed_before_loudness,
    );
    r.run_until("the file marked missing", |r| {
        only_track(r).file_state == FileState::Missing
    });
    assert_eq!(only_track(&r).loudness_pass, None);
}

#[test]
fn n6b_a_newer_loudness_version_measures_again() {
    let dir = tempfile::tempdir().unwrap();
    let a = wav(dir.path(), "a.wav", 1);
    let mut r = rig_with(&[a], dir, Duration::ZERO, |s| {
        analysed_before_loudness(s);
        for t in s.library.iter_mut() {
            t.apply_loudness(None, fp_model::LOUDNESS_VERSION - 1);
        }
    });
    r.run_until("the new pass", |r| {
        only_track(r).loudness_pass == Some(fp_model::LOUDNESS_VERSION)
    });
    assert!(only_track(&r).loudness.is_some());
    assert_eq!(r.loudness_runs.load(Ordering::SeqCst), 1);
}

#[test]
fn n6b_the_pass_runs_with_normalization_off() {
    let dir = tempfile::tempdir().unwrap();
    let a = wav(dir.path(), "a.wav", 1);
    let mut r = rig_with(&[a], dir, Duration::ZERO, |s| {
        analysed_before_loudness(s);
        s.config.loudness.normalize = false;
    });
    r.run_until("the loudness", |r| only_track(r).loudness_pass.is_some());
    assert!(only_track(&r).loudness.is_some());
    assert!(!r.handle.model.load().config.loudness.normalize);
}

#[test]
fn a_track_removed_while_measured_is_forgotten() {
    let dir = tempfile::tempdir().unwrap();
    let a = wav(dir.path(), "a.wav", 1);
    // Each loudness job takes a while: the removal lands while it runs.
    let mut r = rig_with(
        &[a],
        dir,
        Duration::from_millis(400),
        analysed_before_loudness,
    );
    r.run_until("the job started", |r| {
        r.loudness_runs.load(Ordering::SeqCst) == 1
    });
    let first = r.handle.model.load().playlists.first_id().unwrap();
    let entry = r.handle.model.load().playlists.get(first).unwrap().entries[0].id;
    r.handle.send(Command::RemoveEntry(entry));
    r.run_until("the track gone", |r| {
        r.handle.model.load().library.is_empty()
    });
    let version = r.handle.telemetry.load().model_version;
    let until = r.now + Duration::from_millis(800);
    r.run_until("a quiet period", |r| r.now >= until);
    assert_eq!(
        r.handle.telemetry.load().model_version,
        version,
        "no answer for the gone track"
    );
    assert_eq!(r.loudness_runs.load(Ordering::SeqCst), 1);
}

/// I1: a loudness-only answer that arrives after a Re-analyse of its track
/// is stale; the analysis's measurement stays.
#[test]
fn a_stale_loudness_answer_does_not_overwrite_a_new_analysis() {
    use std::sync::atomic::AtomicBool;
    let dir = tempfile::tempdir().unwrap();
    let a = wav(dir.path(), "a.wav", 2);
    // The loudness-only job holds until the test lets it answer (or it is
    // cancelled), then gives a measurement no analysis would.
    let gate = Arc::new(AtomicBool::new(false));
    let returned = Arc::new(AtomicBool::new(false));
    let (open, back) = (gate.clone(), returned.clone());
    let measure: LoudnessFn = Arc::new(move |_, cancelled| {
        loop {
            if open.load(Ordering::SeqCst) {
                back.store(true, Ordering::SeqCst);
                return Ok(Some(fp_model::LoudnessMeasurement {
                    integrated_lufs: Some(-99.0),
                    true_peak_dbtp: -99.0,
                }));
            }
            if cancelled() {
                back.store(true, Ordering::SeqCst);
                return Err(fp_analysis::AnalysisError::Cancelled);
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    });
    // Two workers: the analysis runs beside the loudness job.
    let mut r = rig_pool(
        &[a],
        dir,
        Duration::ZERO,
        analysed_before_loudness,
        Arc::new(|_| {}),
        2,
        Some(measure),
    );
    r.run_until("the loudness job", |r| {
        r.loudness_runs.load(Ordering::SeqCst) == 1
    });
    let track = only_track(&r).id;
    r.services
        .requests()
        .send(ServiceRequest::ReanalyseTrack(track))
        .unwrap();
    // Until the model holds the analysis, before the services see it.
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        assert!(
            Instant::now() < deadline,
            "timed out waiting for the analysis"
        );
        r.conductor.tick(r.now);
        if only_track(&r).loudness_pass.is_some() {
            break;
        }
        r.services.step(r.now);
        r.now += Duration::from_millis(10);
        std::thread::sleep(Duration::from_millis(2));
    }
    let measured = only_track(&r).loudness;
    assert!(measured.and_then(|l| l.integrated_lufs).is_some());
    // The loudness job answers now (unless it was cancelled).
    gate.store(true, Ordering::SeqCst);
    let deadline = Instant::now() + Duration::from_secs(5);
    while !returned.load(Ordering::SeqCst) {
        assert!(Instant::now() < deadline, "the loudness job never returned");
        std::thread::sleep(Duration::from_millis(1));
    }
    std::thread::sleep(Duration::from_millis(50));
    let until = r.now + Duration::from_millis(300);
    r.run_until("a quiet period", |r| r.now >= until);
    assert_eq!(
        only_track(&r).loudness,
        measured,
        "the analysis's measurement stays"
    );
}

/// A rig with one analysed, tag-read jingle on cart 1 of the first page and
/// an empty second page; `open` is whether the cartwall is expanded.
fn cart_rig(
    open: bool,
) -> (
    Rig,
    fp_model::TrackId,
    fp_model::CartPageId,
    fp_model::CartPageId,
) {
    let dir = tempfile::tempdir().unwrap();
    let jingle = wav(dir.path(), "jingle.wav", 1);
    let r = rig_with(&[], dir, Duration::ZERO, |state| {
        let page = state.cartwall.pages.first().unwrap().id;
        fp_model::apply(
            state,
            Command::AssignCartFile {
                page,
                index: 0,
                path: jingle.clone(),
            },
        )
        .unwrap();
        fp_model::apply(
            state,
            Command::CreateCartPage {
                name: "Other".into(),
            },
        )
        .unwrap();
        for track in state.library.iter_mut() {
            track.analyzed = true;
            track.duration_secs = 1.0;
            track.format = Some(fp_model::AudioFormat {
                sample_rate: 48_000,
                bits: Some(16),
                channels: 1,
                dsd_rate: None,
                bitrate: None,
            });
            track.analysis_version = fp_analysis::cache::ANALYSIS_VERSION;
            track.tags_read = true;
        }
        state.cartwall.open = open;
    });
    let s = r.handle.model.load();
    let track = s.cartwall.pages[0].carts[0].track.unwrap();
    let (first, second) = (s.cartwall.pages[0].id, s.cartwall.pages[1].id);
    (r, track, first, second)
}

/// Cart colour spec CC8: the carts of the page shown get their peaks once;
/// going to another page and back reads nothing again, and fetching the
/// peaks of an analysed track does not apply its analysis again (no tag
/// re-read, no new model version).
#[test]
fn the_carts_of_the_page_shown_get_their_peaks_once() {
    let (mut r, track, first, second) = cart_rig(true);
    r.run_until("peaks for the cart of the page shown", |r| {
        r.media.contains(track)
    });
    for page in [second, first, second, first] {
        r.handle.send(Command::ShowCartPage(page));
        for _ in 0..20 {
            r.conductor.tick(r.now);
            r.services.step(r.now);
            r.now += Duration::from_millis(10);
            std::thread::sleep(Duration::from_millis(2));
        }
    }
    assert!(r.media.contains(track), "the peaks are kept");
    assert_eq!(r.analyses.load(Ordering::SeqCst), 1);
    let model = r.handle.model.load();
    assert!(
        model.library.get(track).is_some_and(|t| t.tags_read),
        "the analysis is not applied again"
    );
}

/// A collapsed cartwall draws no waveform: no read for its peaks.
#[test]
fn a_collapsed_cartwall_reads_no_peaks() {
    let (mut r, track, _, _) = cart_rig(false);
    for _ in 0..100 {
        r.conductor.tick(r.now);
        r.services.step(r.now);
        r.now += Duration::from_millis(10);
        std::thread::sleep(Duration::from_millis(2));
    }
    assert_eq!(r.analyses.load(Ordering::SeqCst), 0);
    assert!(!r.media.contains(track));
}

/// The started analyses of `paths`, in order, as indexes into `paths`.
fn started_of(r: &Rig, paths: &[PathBuf]) -> Vec<usize> {
    r.started
        .lock()
        .unwrap()
        .iter()
        .filter_map(|p| paths.iter().position(|f| f == p))
        .collect()
}

/// Runs rounds for `ms` of the rig's time.
fn idle_for(r: &mut Rig, ms: u64) {
    let until = r.now + Duration::from_millis(ms);
    r.run_until("a quiet period", |r| r.now >= until);
}

/// Main spec §6.1: a second *Re-analyse all* while the library pass runs
/// cancels nothing and restarts nothing.
#[test]
fn a_second_reanalyse_all_while_the_pass_runs_cancels_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let files: Vec<PathBuf> = (1..=3)
        .map(|n| wav(dir.path(), &format!("{n}.wav"), 1))
        .collect();
    let mut r = rig(&files, dir);
    // Every first analysis applied and nothing in flight: the analysis
    // count goes up when a job starts, and a result routed after the
    // request below would count in the pass.
    r.run_until("analysed and idle", |r| {
        r.handle.model.load().library.iter().all(|t| t.analyzed)
            && !r.services.activity().load().is_busy()
    });
    assert_eq!(r.analyses.load(Ordering::SeqCst), 3);
    assert_eq!(r.services.activity().load().pass, None);
    // The pass's first job holds at the gate, so no job of it can end
    // before the gate opens.
    r.gate.store(false, Ordering::SeqCst);
    let requests = r.services.requests();
    requests.send(ServiceRequest::ReanalyseAll).unwrap();
    r.run_until("the pass started", |r| {
        r.analyses.load(Ordering::SeqCst) == 4
    });
    let progress = Some(PassProgress { done: 0, total: 3 });
    assert_eq!(r.services.activity().load().pass, progress);
    requests.send(ServiceRequest::ReanalyseAll).unwrap();
    idle_for(&mut r, 200);
    assert_eq!(
        r.analyses.load(Ordering::SeqCst),
        4,
        "the held job was not cancelled and started again"
    );
    assert_eq!(r.services.activity().load().pass, progress);
    r.gate.store(true, Ordering::SeqCst);
    r.run_until("the pass ended", |r| {
        r.services.activity().load().pass.is_none()
    });
    idle_for(&mut r, 100);
    assert_eq!(r.analyses.load(Ordering::SeqCst), 6, "each track once more");
    assert!(!r.services.activity().load().is_busy());
}

/// Main spec §6.1: Re-analyse of a track that is queued or running does
/// nothing: the running job is not restarted and the queued one keeps its
/// place.
#[test]
fn reanalysing_a_queued_or_running_track_does_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let files: Vec<PathBuf> = (1..=3)
        .map(|n| wav(dir.path(), &format!("{n}.wav"), 1))
        .collect();
    let mut r = rig(&files, dir);
    r.gate.store(false, Ordering::SeqCst);
    r.run_until("the first job running", |r| {
        r.analyses.load(Ordering::SeqCst) == 1
    });
    let model = r.handle.model.load_full();
    let id = |path: &PathBuf| model.library.iter().find(|t| &t.path == path).unwrap().id;
    let (running, queued) = (id(&files[0]), id(&files[2]));
    // A round publishes what the pool does.
    r.services.step(r.now);
    let activity = r.services.activity().load_full();
    assert_eq!(activity.of(running), Some(TrackActivity::Running));
    assert_eq!(activity.of(queued), Some(TrackActivity::Queued));
    let requests = r.services.requests();
    requests
        .send(ServiceRequest::ReanalyseTrack(running))
        .unwrap();
    requests
        .send(ServiceRequest::ReanalyseTrack(queued))
        .unwrap();
    r.services.step(r.now);
    r.gate.store(true, Ordering::SeqCst);
    r.run_until("analysed", |r| {
        r.handle.model.load().library.iter().all(|t| t.analyzed)
    });
    idle_for(&mut r, 100);
    assert_eq!(started_of(&r, &files), vec![0, 1, 2]);
}

/// Rewrites `path` with other samples, keeping its size and modification
/// time: an edit in place the cache key cannot see.
fn edit_in_place(path: &Path) {
    let before = std::fs::metadata(path).unwrap();
    let spec = hound::WavReader::open(path).unwrap().spec();
    let frames = hound::WavReader::open(path).unwrap().duration();
    let mut w = hound::WavWriter::create(path, spec).unwrap();
    for i in 0..frames {
        w.write_sample(((i as f32 * 0.05).sin() * 800.0) as i16)
            .unwrap();
    }
    w.finalize().unwrap();
    let file = std::fs::File::options().write(true).open(path).unwrap();
    file.set_modified(before.modified().unwrap()).unwrap();
    let after = std::fs::metadata(path).unwrap();
    assert_eq!(after.len(), before.len());
    assert_eq!(after.modified().unwrap(), before.modified().unwrap());
}

fn integrated(r: &Rig) -> f32 {
    let model = r.handle.model.load();
    let track = model.library.iter().next().unwrap();
    track
        .loudness
        .and_then(|l| l.integrated_lufs)
        .unwrap_or(f32::NAN)
}

/// Itr06 I11: Re-analyse (a track or the library) reads the file again,
/// even when the cache has an entry for its size and time.
#[test]
fn reanalyse_reads_a_file_edited_in_place() {
    let dir = tempfile::tempdir().unwrap();
    let file = wav(dir.path(), "a.wav", 1);
    let mut r = rig_cached(
        std::slice::from_ref(&file),
        dir,
        Duration::ZERO,
        |_| {},
        Arc::new(|_| {}),
        1,
        None,
        true,
    );
    r.run_until("analysed", |r| r.analyses.load(Ordering::SeqCst) == 1);
    r.run_until("measured", |r| {
        r.handle
            .model
            .load()
            .library
            .iter()
            .all(|t| t.loudness.is_some())
    });
    let loud = integrated(&r);
    edit_in_place(&file);
    let track = r.handle.model.load().library.iter().next().unwrap().id;
    r.services
        .requests()
        .send(ServiceRequest::ReanalyseTrack(track))
        .unwrap();
    r.run_until("measured again", |r| (integrated(r) - loud).abs() > 6.0);
    assert_eq!(r.analyses.load(Ordering::SeqCst), 2);
    // Back to the louder samples, and the whole library.
    let quiet = integrated(&r);
    let spec = hound::WavReader::open(&file).unwrap().spec();
    let before = std::fs::metadata(&file).unwrap().modified().unwrap();
    let mut w = hound::WavWriter::create(&file, spec).unwrap();
    for i in 0..48_000 {
        w.write_sample(((i as f32 * 0.05).sin() * 8_000.0) as i16)
            .unwrap();
    }
    w.finalize().unwrap();
    std::fs::File::options()
        .write(true)
        .open(&file)
        .unwrap()
        .set_modified(before)
        .unwrap();
    r.services
        .requests()
        .send(ServiceRequest::ReanalyseAll)
        .unwrap();
    r.run_until("measured once more", |r| {
        (integrated(r) - quiet).abs() > 6.0
    });
    assert_eq!(r.analyses.load(Ordering::SeqCst), 3);
}

fn set_trim(r: &Rig, db: f64) {
    let mut config = r.handle.model.load().config.clone();
    config.analysis.trim_threshold_db = db;
    assert!(r.handle.send(Command::UpdateConfig(Box::new(config))));
}

fn pass_running(r: &Rig) -> bool {
    r.services.activity().load().pass.is_some()
}

/// Review of #115: a settings change that restarts a *Re-analyse all*
/// keeps the restarted pass fresh, so an edited file is still read.
#[test]
fn a_settings_change_during_a_reanalyse_all_keeps_it_fresh() {
    let dir = tempfile::tempdir().unwrap();
    let file = wav(dir.path(), "a.wav", 1);
    let mut r = rig_cached(
        std::slice::from_ref(&file),
        dir,
        Duration::ZERO,
        |_| {},
        Arc::new(|_| {}),
        1,
        None,
        true,
    );
    r.run_until("analysed", |r| r.analyses.load(Ordering::SeqCst) == 1);
    // The cache gets an entry under both settings (the pool started with
    // its own defaults, so -60 is analysed once more).
    set_trim(&r, -50.0);
    r.run_until("analysed under -50", |r| {
        r.analyses.load(Ordering::SeqCst) == 2 && !pass_running(r)
    });
    set_trim(&r, -60.0);
    r.run_until("analysed under -60", |r| {
        r.analyses.load(Ordering::SeqCst) == 3 && !pass_running(r)
    });
    idle_for(&mut r, 100);
    let loud = integrated(&r);
    edit_in_place(&file);
    r.gate.store(false, Ordering::SeqCst);
    r.services
        .requests()
        .send(ServiceRequest::ReanalyseAll)
        .unwrap();
    r.run_until("the fresh job held", |r| {
        r.analyses.load(Ordering::SeqCst) == 4
    });
    set_trim(&r, -50.0);
    r.run_until("the restart", |r| {
        r.handle.model.load().config.analysis.trim_threshold_db == -50.0
    });
    idle_for(&mut r, 50);
    r.gate.store(true, Ordering::SeqCst);
    r.run_until("the pass ended", |r| !pass_running(r));
    idle_for(&mut r, 100);
    assert!((integrated(&r) - loud).abs() > 6.0, "measured again");
}

/// Review of #115: a track read only so a player can draw its waveform is
/// not analysis activity.
#[test]
fn a_waveform_read_for_a_player_is_not_activity() {
    let dir = tempfile::tempdir().unwrap();
    let files = [wav(dir.path(), "a.wav", 1), wav(dir.path(), "b.wav", 1)];
    let mut r = rig(&files, dir);
    r.run_until("analysed", |r| {
        r.handle.model.load().library.iter().all(|t| t.analyzed)
    });
    idle_for(&mut r, 100);
    let model = r.handle.model.load_full();
    let second = model.playlists.iter().next().unwrap().entries[1].clone();
    let track = second.track;
    assert!(!r.media.contains(track), "not shown, not kept");
    r.gate.store(false, Ordering::SeqCst);
    assert!(
        r.handle
            .send(Command::SetNext(model.players[0].id, second.id))
    );
    r.run_until("the waveform read held", |r| {
        r.analyses.load(Ordering::SeqCst) == 3
    });
    r.services.step(r.now);
    assert!(!r.services.activity().load().is_busy());
    r.gate.store(true, Ordering::SeqCst);
    r.run_until("the waveform", |r| r.media.contains(track));
}

/// Review of #115: an idle library does not rebuild the activity every
/// round.
#[test]
fn the_activity_is_rebuilt_only_when_something_changed() {
    let dir = tempfile::tempdir().unwrap();
    let file = wav(dir.path(), "a.wav", 1);
    let mut r = rig(std::slice::from_ref(&file), dir);
    r.run_until("analysed", |r| {
        r.handle.model.load().library.iter().all(|t| t.analyzed)
    });
    idle_for(&mut r, 200);
    let before = r.services.activity_builds();
    assert!(before > 0);
    idle_for(&mut r, 200);
    assert_eq!(r.services.activity_builds(), before);
}
