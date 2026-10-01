#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The services thread: analysis wiring and autosave (spec §6, §7).

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use std::sync::atomic::{AtomicUsize, Ordering};

use fp_analysis::analyze_file_cancellable;
use fp_analysis::analyzer::{AnalyzeFn, Analyzer};
use fp_app::services::{MediaCache, ServiceRequest, Services};
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
    now: Instant,
    _dir: tempfile::TempDir,
}

fn rig(files: &[PathBuf], dir: tempfile::TempDir) -> Rig {
    rig_slow(files, dir, Duration::ZERO)
}

/// `delay`: each analysis waits this long first (still cancellable).
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
    let counter = analyses.clone();
    let counting: AnalyzeFn = Arc::new(move |path, settings, limits, cancelled| {
        counter.fetch_add(1, Ordering::SeqCst);
        let until = Instant::now() + delay;
        while Instant::now() < until {
            if cancelled() {
                return Err(fp_analysis::AnalysisError::Cancelled);
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        analyze_file_cancellable(path, settings, limits, cancelled)
    });
    let analyzer =
        Analyzer::with_analyze_fn(1, Default::default(), Default::default(), None, counting)
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
        now,
        _dir: dir,
    }
}

impl Rig {
    fn run_until(&mut self, what: &str, mut done: impl FnMut(&Rig) -> bool) {
        let deadline = Instant::now() + Duration::from_secs(15);
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
/// nothing is saved every interval.
#[test]
fn a_file_still_missing_is_looked_for_without_touching_the_model() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = recheck_rig(&[PathBuf::from("/definitely/missing.flac")], dir);
    r.run_until("missing state", missing);
    let looked = r.analyses.load(Ordering::SeqCst);
    let version = r.handle.telemetry.load().model_version;
    let until = r.now + Duration::from_millis(3_500);
    r.run_until("three intervals", |r| r.now >= until);
    assert!(
        r.analyses.load(Ordering::SeqCst) >= looked + 3,
        "looked for again"
    );
    assert_eq!(r.handle.telemetry.load().model_version, version);
}
