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
use fp_app::services::{MediaCache, Services};
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
    config.analysis.silence_threshold_db = -50.0;
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
    let last = model.playlists.iter().next().unwrap().entries[2];
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
