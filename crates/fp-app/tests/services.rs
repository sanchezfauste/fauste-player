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
        let outcome = analyze_file_cancellable(path, settings, limits, cancelled);
        after(path);
        outcome
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
                dsd_rate: None,
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
            });
            track.analysis_version = 0;
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

/// A track loaded on a player gets its waveform ahead of the library being
/// analysed, not after it.
#[test]
fn a_track_on_a_player_is_analysed_ahead_of_the_library() {
    let dir = tempfile::tempdir().unwrap();
    let files: Vec<PathBuf> = (1..=12)
        .map(|n| wav(dir.path(), &format!("{n}.wav"), 1))
        .collect();
    let mut r = rig_with(&files, dir, Duration::from_millis(40), |s| {
        let p = s.players[0].id;
        let last = s.playlists.iter().next().unwrap().entries[11].id;
        fp_model::apply(s, Command::SetNext(p, last)).unwrap();
    });
    let model = r.handle.model.load_full();
    let next = model.players[0].next.unwrap();
    let track = model.playlists.entry(next).unwrap().track;
    r.run_until("peaks for the loaded track", |r| r.media.contains(track));
    assert!(
        r.analyses.load(Ordering::SeqCst) <= 3,
        "{} analyses ran first",
        r.analyses.load(Ordering::SeqCst)
    );
}

/// A track loaded while it waits in the queue with the library moves ahead.
#[test]
fn a_queued_track_loaded_on_a_player_moves_ahead() {
    let dir = tempfile::tempdir().unwrap();
    let files: Vec<PathBuf> = (1..=12)
        .map(|n| wav(dir.path(), &format!("{n}.wav"), 1))
        .collect();
    let mut r = rig_slow(&files, dir, Duration::from_millis(40));
    r.run_until("the library queued", |r| {
        r.analyses.load(Ordering::SeqCst) >= 1
    });
    let model = r.handle.model.load_full();
    let last = model.playlists.iter().next().unwrap().entries[11].clone();
    r.handle
        .send(Command::SetNext(model.players[0].id, last.id));
    r.run_until("peaks for the loaded track", |r| {
        r.media.contains(last.track)
    });
    assert!(
        r.analyses.load(Ordering::SeqCst) <= 4,
        "{} analyses ran first",
        r.analyses.load(Ordering::SeqCst)
    );
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
    assert_eq!(
        r.handle.telemetry.load().model_version,
        version,
        "no further ApplyTags once the tags are read"
    );
    assert_eq!(r.analyses.load(Ordering::SeqCst), 1, "no second analysis");
}

#[test]
fn a_track_whose_tags_cannot_be_read_is_not_asked_again() {
    let dir = tempfile::tempdir().unwrap();
    let junk = dir.path().join("Band - Junk.wav");
    std::fs::write(&junk, b"this is not audio").unwrap();
    // Analysed already (so no analysis runs), with its tags still unread.
    let mut r = rig_with(&[junk], dir, Duration::ZERO, |state| {
        for track in state.library.iter_mut() {
            track.analyzed = true;
            track.format = Some(fp_model::AudioFormat {
                sample_rate: 44_100,
                bits: Some(16),
                channels: 2,
                dsd_rate: None,
            });
            track.analysis_version = fp_analysis::cache::ANALYSIS_VERSION;
        }
    });
    r.run_until("the pass to answer", |r| only_track(r).tags_read);
    let t = only_track(&r);
    assert_eq!((t.title.as_str(), t.artist.as_str()), ("Junk", "Band"));
    let version = r.handle.telemetry.load().model_version;
    let until = r.now + Duration::from_millis(500);
    r.run_until("a quiet period", |r| r.now >= until);
    assert_eq!(r.handle.telemetry.load().model_version, version);
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
            });
            track.analysis_version = fp_analysis::cache::ANALYSIS_VERSION;
        }
    });
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

/// Q10.4: the row menu's Re-analyse does not wait for a change.
#[test]
fn reanalyse_track_analyses_a_failed_file_at_once_whatever_its_state() {
    let dir = tempfile::tempdir().unwrap();
    let bad = garbage(dir.path(), "bad.wav");
    let mut r = recheck_rig(&[bad], dir);
    r.run_until("unreadable and stamped", unreadable_and_stamped);
    let track = r.handle.model.load().library.iter().next().unwrap().id;
    r.services
        .requests()
        .send(ServiceRequest::ReanalyseTrack(track))
        .unwrap();
    r.run_until("a second analysis", |r| {
        r.analyses.load(Ordering::SeqCst) == 2
    });
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
