#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Watched folders on the services thread (track library spec LB19-LB22):
//! the rescan at start, the category a file joins, and the watch following
//! the categories. Time is driven with `now`, never waited for.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use fp_app::ui::files::canonical_path;
use fp_app::watch::{FolderEvent, FolderTrouble, FolderWatcher};
use fp_model::{AppState, CategoryId, Command, FileState, TrackKind, apply};

use crate::support::FakeWatcher;
use crate::support::services_rig::{Rig, rig_with, wav};

fn category(state: &AppState, name: &str) -> CategoryId {
    state
        .categories
        .iter()
        .find(|c| c.name == name)
        .map(|c| c.id)
        .unwrap()
}

fn create(state: &mut AppState, name: &str, kind: TrackKind, folder: Option<&Path>) -> CategoryId {
    apply(
        state,
        Command::CreateCategory {
            parent: None,
            name: name.into(),
            color: None,
            kind,
        },
    )
    .unwrap();
    let id = category(state, name);
    if let Some(folder) = folder {
        apply(
            state,
            Command::SetWatchedFolder {
                category: id,
                folder: Some(folder.to_path_buf()),
            },
        )
        .unwrap();
    }
    id
}

/// A folder under the test's directory, in the canonical form the folder
/// dialog stores (on Windows without the verbatim `\\?\` prefix and with
/// long names; on macOS under `/private/var`), the form of the tracks' paths.
fn folder(dir: &tempfile::TempDir, name: &str) -> PathBuf {
    let path = dir.path().join(name);
    std::fs::create_dir_all(&path).unwrap();
    canonical_path(&path)
}

fn touch(path: &Path) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, b"not really audio").unwrap();
}

/// A rig with `prepare` applied, a fake watcher set on the services.
fn watching(dir: tempfile::TempDir, prepare: impl FnOnce(&mut AppState)) -> (Rig, FakeWatcher) {
    let mut r = rig_with(&[], dir, Duration::ZERO, prepare);
    let fake = FakeWatcher::default();
    r.services.set_watcher(Box::new(fake.clone()));
    (r, fake)
}

fn in_category(r: &Rig, category: Option<CategoryId>) -> Vec<PathBuf> {
    let model = r.handle.model.load();
    let mut paths: Vec<PathBuf> = model
        .library
        .iter()
        .filter(|t| t.category == category)
        .map(|t| t.path.clone())
        .collect();
    paths.sort();
    paths
}

fn tracks(r: &Rig) -> usize {
    r.handle.model.load().library.len()
}

fn quiet_for(r: &mut Rig, millis: u64) {
    let until = r.now + Duration::from_millis(millis);
    r.run_until("a quiet period", |r| r.now >= until);
}

#[test]
fn the_start_rescan_files_the_folder_into_its_category() {
    let dir = tempfile::tempdir().unwrap();
    let ids_folder = folder(&dir, "ids");
    touch(&ids_folder.join("a.mp3"));
    wav(&ids_folder, "b.wav", 1);
    let mut ids = None;
    let (mut r, fake) = watching(dir, |s| {
        ids = Some(create(s, "IDs", TrackKind::Jingle, Some(&ids_folder)));
    });
    let ids = ids.unwrap();
    r.run_until("both tracks", |r| tracks(r) == 2);
    let model = r.handle.model.load();
    assert!(
        model
            .library
            .iter()
            .all(|t| t.category == Some(ids) && t.kind == TrackKind::Jingle)
    );
    r.run_until("watched", |_| fake.watched() == vec![ids_folder.clone()]);
}

#[test]
fn only_audio_files_join_and_subfolders_make_no_categories() {
    let dir = tempfile::tempdir().unwrap();
    let ids_folder = folder(&dir, "ids");
    touch(&ids_folder.join("a.MP3"));
    touch(&ids_folder.join("x/y/b.flac"));
    touch(&ids_folder.join("film.mp4"));
    touch(&ids_folder.join("notes.txt"));
    let mut ids = None;
    let (mut r, _fake) = watching(dir, |s| {
        ids = Some(create(s, "IDs", TrackKind::Jingle, Some(&ids_folder)));
    });
    let ids = ids.unwrap();
    r.run_until("both tracks", |r| tracks(r) == 2);
    quiet_for(&mut r, 500);
    assert_eq!(tracks(&r), 2);
    assert_eq!(in_category(&r, Some(ids)).len(), 2);
    assert_eq!(r.handle.model.load().categories.len(), 1);
}

#[test]
fn a_known_file_keeps_its_category_and_an_uncategorised_one_joins() {
    let dir = tempfile::tempdir().unwrap();
    let ids_folder = folder(&dir, "ids");
    let c = ids_folder.join("c.mp3");
    let d = ids_folder.join("d.mp3");
    touch(&c);
    touch(&d);
    let (mut gold, mut ids) = (None, None);
    let (mut r, _fake) = watching(dir, |s| {
        ids = Some(create(s, "IDs", TrackKind::Jingle, Some(&ids_folder)));
        let g = create(s, "Gold", TrackKind::Music, None);
        gold = Some(g);
        apply(
            s,
            Command::AddToLibrary {
                paths: vec![c.clone()],
                category: Some(g),
            },
        )
        .unwrap();
        apply(
            s,
            Command::AddToLibrary {
                paths: vec![d.clone()],
                category: None,
            },
        )
        .unwrap();
    });
    let (gold, ids) = (gold.unwrap(), ids.unwrap());
    r.run_until("d joins IDs", |r| {
        r.handle
            .model
            .load()
            .library
            .iter()
            .any(|t| t.path == d && t.category == Some(ids))
    });
    quiet_for(&mut r, 300);
    assert_eq!(tracks(&r), 2, "one track per file");
    assert_eq!(in_category(&r, Some(gold)), vec![c]);
    assert_eq!(in_category(&r, Some(ids)), vec![d]);
}

#[test]
fn the_innermost_watched_folder_wins() {
    let dir = tempfile::tempdir().unwrap();
    let music = folder(&dir, "m");
    let gold_folder = folder(&dir, "m/gold");
    touch(&music.join("1.mp3"));
    touch(&gold_folder.join("2.mp3"));
    let (mut m, mut g) = (None, None);
    let (mut r, _fake) = watching(dir, |s| {
        m = Some(create(s, "Music", TrackKind::Music, Some(&music)));
        g = Some(create(s, "Gold", TrackKind::Music, Some(&gold_folder)));
    });
    let (m, g) = (m.unwrap(), g.unwrap());
    r.run_until("both tracks", |r| tracks(r) == 2);
    quiet_for(&mut r, 300);
    assert_eq!(in_category(&r, Some(m)).len(), 1);
    assert!(in_category(&r, Some(m))[0].ends_with("m/1.mp3"));
    assert_eq!(in_category(&r, Some(g)).len(), 1);
    assert!(in_category(&r, Some(g))[0].ends_with("m/gold/2.mp3"));
}

#[test]
fn a_scan_answer_for_a_category_deleted_meanwhile_adds_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let ids_folder = folder(&dir, "ids");
    touch(&ids_folder.join("a.mp3"));
    touch(&ids_folder.join("b.mp3"));
    let mut ids = None;
    let (mut r, fake) = watching(dir, |s| {
        ids = Some(create(s, "IDs", TrackKind::Jingle, Some(&ids_folder)));
    });
    let ids = ids.unwrap();
    // The first step sends the start scan.
    r.services.step(r.now);
    assert_eq!(r.services.folder_scans(), 1);
    r.handle.send(Command::DeleteCategory(ids));
    r.conductor.tick(r.now);
    // The next step takes the answer, now for a category that is gone.
    r.services.step(r.now);
    r.run_until("not watched", |_| fake.watched().is_empty());
    quiet_for(&mut r, 500);
    assert_eq!(tracks(&r), 0, "nothing joined, not even uncategorised");
}

#[test]
fn clearing_a_folder_stops_watching_and_keeps_the_tracks() {
    let dir = tempfile::tempdir().unwrap();
    let ids_folder = folder(&dir, "ids");
    touch(&ids_folder.join("a.mp3"));
    touch(&ids_folder.join("b.mp3"));
    let mut ids = None;
    let (mut r, fake) = watching(dir, |s| {
        ids = Some(create(s, "IDs", TrackKind::Jingle, Some(&ids_folder)));
    });
    let ids = ids.unwrap();
    r.run_until("both tracks", |r| tracks(r) == 2);
    r.handle.send(Command::SetWatchedFolder {
        category: ids,
        folder: None,
    });
    r.run_until("not watched", |_| fake.watched().is_empty());
    quiet_for(&mut r, 300);
    assert_eq!(in_category(&r, Some(ids)).len(), 2);
}

#[test]
fn an_idle_watch_does_not_walk_the_library() {
    let dir = tempfile::tempdir().unwrap();
    let ids_folder = folder(&dir, "ids");
    wav(&ids_folder, "a.wav", 1);
    wav(&ids_folder, "b.wav", 1);
    let (mut r, _fake) = watching(dir, |s| {
        create(s, "IDs", TrackKind::Jingle, Some(&ids_folder));
    });
    r.run_until("analysed", |r| {
        let model = r.handle.model.load();
        model.library.len() == 2 && model.library.iter().all(|t| t.analyzed)
    });
    quiet_for(&mut r, 500);
    let (walks, scans) = (r.services.library_walks(), r.services.folder_scans());
    assert!(scans >= 1);
    for _ in 0..50 {
        r.conductor.tick(r.now);
        r.services.step(r.now);
        r.now += Duration::from_millis(10);
    }
    assert_eq!(r.services.library_walks(), walks);
    assert_eq!(r.services.folder_scans(), scans);
}

#[test]
fn a_bad_stored_folder_is_not_watched() {
    let dir = tempfile::tempdir().unwrap();
    let (mut r, fake) = watching(dir, |s| {
        let id = create(s, "IDs", TrackKind::Jingle, None);
        apply(
            s,
            Command::SetWatchedFolder {
                category: id,
                folder: Some(PathBuf::new()),
            },
        )
        .unwrap();
        let rel = create(s, "Rel", TrackKind::Jingle, None);
        apply(
            s,
            Command::SetWatchedFolder {
                category: rel,
                folder: Some(PathBuf::from("music/relative")),
            },
        )
        .unwrap();
    });
    quiet_for(&mut r, 300);
    assert!(fake.watched().is_empty());
    assert_eq!(r.services.folder_scans(), 0);
    assert_eq!(tracks(&r), 0);
    // An event cannot make it matter either.
    fake.push(FolderEvent::Rescan);
    quiet_for(&mut r, 300);
    assert_eq!(r.services.folder_scans(), 0);
}

#[test]
fn a_refused_folder_is_not_followed_but_still_scanned() {
    let dir = tempfile::tempdir().unwrap();
    let ids_folder = folder(&dir, "ids");
    touch(&ids_folder.join("a.mp3"));
    touch(&ids_folder.join("b.mp3"));
    let mut ids = None;
    let fake = FakeWatcher::default();
    fake.refuse(ids_folder.clone(), true);
    let mut r = rig_with(&[], dir, Duration::ZERO, |s| {
        ids = Some(create(s, "IDs", TrackKind::Jingle, Some(&ids_folder)));
    });
    r.services.set_watcher(Box::new(fake.clone()));
    let ids = ids.unwrap();
    let status = r.services.watch_status();
    r.run_until("not followed", |_| {
        status.load().troubles.get(&ids) == Some(&FolderTrouble::NotFollowed)
    });
    r.run_until("both tracks", |r| tracks(r) == 2);
    assert!(fake.watched().is_empty());
}

/// A watcher whose `watch` waits for the test.
struct Blocking {
    release: crossbeam_channel::Receiver<()>,
    entered: crossbeam_channel::Sender<()>,
    watched: Arc<Mutex<Vec<PathBuf>>>,
}

impl FolderWatcher for Blocking {
    fn watch(&mut self, folder: &Path) -> Result<(), String> {
        let _ = self.entered.send(());
        let _ = self.release.recv();
        self.watched.lock().unwrap().push(folder.to_path_buf());
        Ok(())
    }
    fn unwatch(&mut self, _: &Path) {}
    fn events(&mut self) -> Vec<FolderEvent> {
        Vec::new()
    }
}

#[test]
fn a_blocked_watch_call_does_not_stall_the_services_step() {
    let dir = tempfile::tempdir().unwrap();
    let ids_folder = folder(&dir, "ids");
    touch(&ids_folder.join("a.mp3"));
    let (release_tx, release) = crossbeam_channel::unbounded();
    let (entered_tx, entered) = crossbeam_channel::unbounded();
    let watched = Arc::new(Mutex::new(Vec::new()));
    let mut r = rig_with(&[], dir, Duration::ZERO, |s| {
        create(s, "IDs", TrackKind::Jingle, Some(&ids_folder));
    });
    r.services.set_watcher(Box::new(Blocking {
        release,
        entered: entered_tx,
        watched: watched.clone(),
    }));
    // Steps return while the watch call is stuck, and the scan goes on.
    r.run_until("the watch call started and the file joined", |r| {
        entered.try_recv().is_ok() || tracks(r) == 1
    });
    r.run_until("the file joined", |r| tracks(r) == 1);
    assert!(watched.lock().unwrap().is_empty(), "still blocked");
    release_tx.send(()).unwrap();
    r.run_until("watched", |_| !watched.lock().unwrap().is_empty());
}

#[test]
fn a_dropped_folder_forgets_its_pending_events() {
    let dir = tempfile::tempdir().unwrap();
    let ids_folder = folder(&dir, "ids");
    let a = wav(&ids_folder, "a.wav", 1);
    let mut ids = None;
    let (mut r, fake) = watching(dir, |s| {
        ids = Some(create(s, "IDs", TrackKind::Jingle, Some(&ids_folder)));
    });
    let ids = ids.unwrap();
    r.run_until("analysed", |r| {
        let m = r.handle.model.load();
        m.library.len() == 1 && m.library.iter().all(|t| t.analyzed)
    });
    r.run_until("watched", |_| !fake.watched().is_empty());
    quiet_for(&mut r, 300);
    // An event waits for its quiet period; the folder is dropped and the
    // file deleted meanwhile.
    fake.push(FolderEvent::Touched(a.clone()));
    r.conductor.tick(r.now);
    r.services.step(r.now);
    r.handle.send(Command::SetWatchedFolder {
        category: ids,
        folder: None,
    });
    r.conductor.tick(r.now);
    r.services.step(r.now);
    std::fs::remove_file(&a).unwrap();
    r.now += Duration::from_secs(5);
    quiet_for(&mut r, 500);
    let model = r.handle.model.load();
    assert!(model.library.iter().all(|t| t.file_state == FileState::Ok));
}

fn set_debounce(s: &mut AppState, millis: u64) {
    s.config.library.watch_debounce_ms = millis;
}

/// Waits for the start rescan: every file in, analysed, the folder watched
/// and the scanner idle.
fn settled(r: &mut Rig, fake: &FakeWatcher, files: usize) {
    r.run_until("the start rescan", |r| {
        let m = r.handle.model.load();
        m.library.len() == files
            && m.library
                .iter()
                .all(|t| t.analyzed || t.path.extension().is_some_and(|e| e != "wav"))
    });
    r.run_until("watched", |_| !fake.watched().is_empty());
    quiet_for(r, 300);
}

fn track_of(r: &Rig, path: &Path) -> Option<fp_model::Track> {
    r.handle
        .model
        .load()
        .library
        .iter()
        .find(|t| t.path == path)
        .cloned()
}

fn push_and_step(r: &mut Rig, fake: &FakeWatcher, path: &Path) {
    fake.push(FolderEvent::Touched(path.to_path_buf()));
    r.conductor.tick(r.now);
    r.services.step(r.now);
}

/// Runs steps until `now` reaches `until`.
fn run_to(r: &mut Rig, until: std::time::Instant) {
    r.run_until("the instant", |r| r.now >= until);
}

#[test]
fn a_new_file_joins_after_the_debounce() {
    let dir = tempfile::tempdir().unwrap();
    let ids_folder = folder(&dir, "ids");
    touch(&ids_folder.join("a.mp3"));
    let (mut r, fake) = watching(dir, |s| {
        create(s, "IDs", TrackKind::Jingle, Some(&ids_folder));
        set_debounce(s, 1_000);
    });
    settled(&mut r, &fake, 1);
    let new = ids_folder.join("new.mp3");
    touch(&new);
    let t0 = r.now;
    push_and_step(&mut r, &fake, &new);
    run_to(&mut r, t0 + Duration::from_millis(990));
    assert!(track_of(&r, &new).is_none(), "not before the debounce");
    run_to(&mut r, t0 + Duration::from_millis(1_000));
    r.run_until("the new file", |r| track_of(r, &new).is_some());
}

#[test]
fn a_burst_of_events_adds_the_file_once_after_the_debounce() {
    let dir = tempfile::tempdir().unwrap();
    let ids_folder = folder(&dir, "ids");
    touch(&ids_folder.join("a.mp3"));
    let (mut r, fake) = watching(dir, |s| {
        create(s, "IDs", TrackKind::Jingle, Some(&ids_folder));
        set_debounce(s, 1_000);
    });
    settled(&mut r, &fake, 1);
    let scans = r.services.folder_scans();
    let new = ids_folder.join("new.mp3");
    touch(&new);
    let mut last = r.now;
    for _ in 0..5 {
        last = r.now;
        push_and_step(&mut r, &fake, &new);
        let until = r.now + Duration::from_millis(200);
        run_to(&mut r, until);
    }
    run_to(&mut r, last + Duration::from_millis(990));
    assert!(track_of(&r, &new).is_none(), "not before the quiet period");
    assert_eq!(r.services.folder_scans(), scans);
    r.run_until("the new file", |r| track_of(r, &new).is_some());
    quiet_for(&mut r, 1_500);
    assert_eq!(tracks(&r), 2);
    assert_eq!(r.services.folder_scans(), scans + 1);
}

#[test]
fn a_removed_file_marks_its_track_missing_and_its_return_restores_it() {
    let dir = tempfile::tempdir().unwrap();
    let ids_folder = folder(&dir, "ids");
    let a = wav(&ids_folder, "a.wav", 1);
    let (mut r, fake) = watching(dir, |s| {
        create(s, "IDs", TrackKind::Jingle, Some(&ids_folder));
        set_debounce(s, 100);
    });
    settled(&mut r, &fake, 1);
    let id = track_of(&r, &a).unwrap().id;
    std::fs::remove_file(&a).unwrap();
    push_and_step(&mut r, &fake, &a);
    r.run_until("missing", |r| {
        track_of(r, &a).is_some_and(|t| t.file_state == FileState::Missing)
    });
    assert_eq!(tracks(&r), 1);
    wav(&ids_folder, "a.wav", 1);
    push_and_step(&mut r, &fake, &a);
    r.run_until("back", |r| {
        track_of(r, &a).is_some_and(|t| t.file_state == FileState::Ok && t.analyzed)
    });
    assert_eq!(track_of(&r, &a).unwrap().id, id);
    assert_eq!(tracks(&r), 1);
}

#[test]
fn a_rename_marks_the_old_track_missing_and_adds_the_new_name() {
    let dir = tempfile::tempdir().unwrap();
    let ids_folder = folder(&dir, "ids");
    let a = wav(&ids_folder, "a.wav", 2);
    let z = ids_folder.join("z.wav");
    let (mut r, fake) = watching(dir, |s| {
        create(s, "IDs", TrackKind::Jingle, Some(&ids_folder));
        set_debounce(s, 100);
    });
    settled(&mut r, &fake, 1);
    let id = track_of(&r, &a).unwrap().id;
    r.handle.send(Command::SetMarker {
        track: id,
        kind: fp_model::MarkerKind::CueIn,
        secs: Some(0.5),
    });
    r.run_until("the cue-in", |r| {
        track_of(r, &a).is_some_and(|t| t.markers.cue_in.is_some_and(|m| m.secs == 0.5))
    });
    let cue = track_of(&r, &a).unwrap().markers.cue_in;
    std::fs::rename(&a, &z).unwrap();
    fake.push(FolderEvent::Touched(a.clone()));
    fake.push(FolderEvent::Touched(z.clone()));
    r.run_until("renamed", |r| {
        track_of(r, &a).is_some_and(|t| t.file_state == FileState::Missing)
            && track_of(r, &z).is_some()
    });
    assert_eq!(track_of(&r, &a).unwrap().markers.cue_in, cue);
    assert_eq!(tracks(&r), 2);
}

#[test]
fn a_removed_subfolder_marks_its_tracks_missing() {
    let dir = tempfile::tempdir().unwrap();
    let ids_folder = folder(&dir, "ids");
    touch(&ids_folder.join("x/1.mp3"));
    touch(&ids_folder.join("x/2.mp3"));
    touch(&ids_folder.join("keep.mp3"));
    let (mut r, fake) = watching(dir, |s| {
        create(s, "IDs", TrackKind::Jingle, Some(&ids_folder));
        set_debounce(s, 100);
    });
    settled(&mut r, &fake, 3);
    std::fs::remove_dir_all(ids_folder.join("x")).unwrap();
    push_and_step(&mut r, &fake, &ids_folder.join("x"));
    r.run_until("both missing", |r| {
        r.handle
            .model
            .load()
            .library
            .iter()
            .filter(|t| t.file_state == FileState::Missing)
            .count()
            == 2
    });
    assert_eq!(tracks(&r), 3);
}

#[test]
fn a_missing_folder_is_reported_retried_and_scanned_when_it_appears() {
    let dir = tempfile::tempdir().unwrap();
    // Not there yet: the canonical form of the directory holding it.
    let gone = canonical_path(dir.path()).join("not-mounted");
    let known = folder(&dir, "elsewhere");
    let old = known.join("old.mp3");
    touch(&old);
    let mut ids = None;
    let fake = FakeWatcher::default();
    fake.refuse(gone.clone(), true);
    let mut r = rig_with(&[], dir, Duration::ZERO, |s| {
        ids = Some(create(s, "IDs", TrackKind::Jingle, Some(&gone)));
        apply(
            s,
            Command::AddToLibrary {
                paths: vec![gone.join("c.mp3")],
                category: ids,
            },
        )
        .unwrap();
        // Restored from an earlier session: analysed, so not looked at.
        let id = s.library.iter().next().unwrap().id;
        let track = s.library.get_mut(id).unwrap();
        track.analyzed = true;
        track.tags_read = true;
        track.loudness_pass = Some(fp_model::loudness::LOUDNESS_VERSION);
        track.analysis_version = fp_analysis::cache::ANALYSIS_VERSION;
        s.config.tuning.missing_recheck_ms = 5_000.0;
    });
    r.services.set_watcher(Box::new(fake.clone()));
    let ids = ids.unwrap();
    let status = r.services.watch_status();
    r.run_until("reported missing", |_| {
        status.load().troubles.get(&ids) == Some(&FolderTrouble::Missing)
    });
    quiet_for(&mut r, 1_000);
    let c = track_of(&r, &gone.join("c.mp3")).unwrap();
    assert_eq!(c.file_state, FileState::Ok, "an absent root marks nothing");
    touch(&gone.join("b.mp3"));
    fake.refuse(gone.clone(), false);
    let until = r.now + Duration::from_millis(5_500);
    run_to(&mut r, until);
    r.run_until("cleared and scanned", |r| {
        !status.load().troubles.contains_key(&ids) && track_of(r, &gone.join("b.mp3")).is_some()
    });
    assert!(fake.watched().contains(&gone));
}

#[test]
fn a_folder_the_system_refuses_is_still_scanned() {
    let dir = tempfile::tempdir().unwrap();
    let ids_folder = folder(&dir, "ids");
    touch(&ids_folder.join("a.mp3"));
    let mut ids = None;
    let fake = FakeWatcher::default();
    fake.refuse(ids_folder.clone(), true);
    let mut r = rig_with(&[], dir, Duration::ZERO, |s| {
        ids = Some(create(s, "IDs", TrackKind::Jingle, Some(&ids_folder)));
    });
    r.services.set_watcher(Box::new(fake.clone()));
    let ids = ids.unwrap();
    let status = r.services.watch_status();
    r.run_until("joined and flagged", |r| {
        tracks(r) == 1 && status.load().troubles.get(&ids) == Some(&FolderTrouble::NotFollowed)
    });
}

/// A watcher whose first `watch` waits for the test and then refuses; the
/// next ones work.
struct SlowRefusal {
    release: crossbeam_channel::Receiver<()>,
    entered: crossbeam_channel::Sender<()>,
    calls: usize,
}

impl FolderWatcher for SlowRefusal {
    fn watch(&mut self, _: &Path) -> Result<(), String> {
        self.calls += 1;
        if self.calls == 1 {
            let _ = self.entered.send(());
            let _ = self.release.recv();
            return Err("refused".into());
        }
        Ok(())
    }
    fn unwatch(&mut self, _: &Path) {}
    fn events(&mut self) -> Vec<FolderEvent> {
        Vec::new()
    }
}

#[test]
fn a_stale_refusal_does_not_leave_a_folder_unfollowed() {
    let dir = tempfile::tempdir().unwrap();
    let ids_folder = folder(&dir, "ids");
    touch(&ids_folder.join("a.mp3"));
    let mut ids = None;
    let (release_tx, release) = crossbeam_channel::unbounded();
    let (entered_tx, entered) = crossbeam_channel::unbounded();
    let mut r = rig_with(&[], dir, Duration::ZERO, |s| {
        ids = Some(create(s, "IDs", TrackKind::Jingle, Some(&ids_folder)));
    });
    r.services.set_watcher(Box::new(SlowRefusal {
        release,
        entered: entered_tx,
        calls: 0,
    }));
    let ids = ids.unwrap();
    let status = r.services.watch_status();
    r.run_until("the first watch call", |_| entered.try_recv().is_ok());
    // Dropped and set again while the first call is still stuck.
    r.handle.send(Command::SetWatchedFolder {
        category: ids,
        folder: None,
    });
    quiet_for(&mut r, 100);
    r.handle.send(Command::SetWatchedFolder {
        category: ids,
        folder: Some(ids_folder.clone()),
    });
    quiet_for(&mut r, 100);
    // The old call now ends with its refusal, then the new one works.
    release_tx.send(()).unwrap();
    quiet_for(&mut r, 500);
    assert!(
        status.load().troubles.is_empty(),
        "{:?}",
        status.load().troubles
    );
}

#[test]
fn lost_events_rescan_every_folder() {
    let dir = tempfile::tempdir().unwrap();
    let ids_folder = folder(&dir, "ids");
    touch(&ids_folder.join("a.mp3"));
    let (mut r, fake) = watching(dir, |s| {
        create(s, "IDs", TrackKind::Jingle, Some(&ids_folder));
    });
    settled(&mut r, &fake, 1);
    let late = ids_folder.join("late.mp3");
    touch(&late);
    fake.push(FolderEvent::Rescan);
    r.run_until("the late file", |r| track_of(r, &late).is_some());
}

#[test]
fn watched_folder_files_are_analysed_behind_a_players_track() {
    let dir = tempfile::tempdir().unwrap();
    let ids_folder = folder(&dir, "ids");
    for n in 0..12 {
        wav(&ids_folder, &format!("f{n}.wav"), 1);
    }
    let outside = wav(dir.path(), "player.wav", 1);
    let (mut r, _fake) = watching(dir, |s| {
        create(s, "IDs", TrackKind::Jingle, Some(&ids_folder));
    });
    // The worker holds the first job; the rest stays queued in order.
    r.gate.store(false, std::sync::atomic::Ordering::SeqCst);
    r.run_until("the folder queued", |r| {
        tracks(r) == 12 && r.analyses.load(std::sync::atomic::Ordering::SeqCst) >= 1
    });
    let playlist = r.handle.model.load().playlists.first_id().unwrap();
    r.handle.send(Command::InsertPaths {
        playlist,
        index: 0,
        paths: vec![outside.clone()],
    });
    r.run_until("on the playlist", |r| {
        r.handle
            .model
            .load()
            .playlists
            .iter()
            .next()
            .unwrap()
            .entries
            .len()
            == 1
    });
    let model = r.handle.model.load_full();
    let entry = model.playlists.iter().next().unwrap().entries[0].id;
    let player = model.players[0].id;
    r.handle.send(Command::SetNext(player, entry));
    r.run_until("loaded", |r| {
        r.handle.model.load().players[0].next == Some(entry)
    });
    quiet_for(&mut r, 100);
    r.gate.store(true, std::sync::atomic::Ordering::SeqCst);
    r.run_until("the player's track analysed", |r| {
        track_of(r, &outside).is_some_and(|t| t.analyzed)
    });
    let before = crate::support::services_rig::started_before(&r, &outside);
    assert!(before <= 1, "{before} analyses started first");
}

fn troubles(r: &Rig) -> std::collections::BTreeMap<CategoryId, FolderTrouble> {
    r.services.watch_status().load().troubles.clone()
}

fn all_ok(r: &Rig) -> bool {
    r.handle
        .model
        .load()
        .library
        .iter()
        .all(|t| t.file_state == FileState::Ok)
}

/// A watched folder with two analysed files, the watch settled.
fn settled_folder(retry_ms: f64) -> (tempfile::TempDir, PathBuf, Rig, FakeWatcher, CategoryId) {
    let dir = tempfile::tempdir().unwrap();
    let ids_folder = folder(&dir, "ids");
    wav(&ids_folder, "a.wav", 1);
    wav(&ids_folder, "b.wav", 1);
    let keep = tempfile::tempdir().unwrap();
    let mut ids = None;
    let (mut r, fake) = watching(dir, |s| {
        ids = Some(create(s, "IDs", TrackKind::Jingle, Some(&ids_folder)));
        set_debounce(s, 100);
        s.config.tuning.missing_recheck_ms = retry_ms;
    });
    settled(&mut r, &fake, 2);
    (keep, ids_folder, r, fake, ids.unwrap())
}

#[test]
fn an_event_on_the_root_itself_marks_nothing_and_reports_it_missing() {
    let (keep, ids_folder, mut r, fake, ids) = settled_folder(2_000.0);
    // Moved away (or unmounted): the event names the folder itself.
    let away = keep.path().join("away");
    std::fs::rename(&ids_folder, &away).unwrap();
    push_and_step(&mut r, &fake, &ids_folder);
    r.run_until("reported missing", |r| {
        troubles(r).get(&ids) == Some(&FolderTrouble::Missing)
    });
    quiet_for(&mut r, 500);
    assert!(all_ok(&r), "a root that is not there marks nothing");
    std::fs::rename(&away, &ids_folder).unwrap();
    let until = r.now + Duration::from_millis(2_500);
    run_to(&mut r, until);
    r.run_until("cleared", |r| troubles(r).is_empty());
    assert!(all_ok(&r));
}

#[test]
fn an_event_inside_a_vanished_root_marks_nothing() {
    let (keep, ids_folder, mut r, fake, ids) = settled_folder(30_000.0);
    let away = keep.path().join("away");
    std::fs::rename(&ids_folder, &away).unwrap();
    push_and_step(&mut r, &fake, &ids_folder.join("a.wav"));
    r.run_until("reported missing", |r| {
        troubles(r).get(&ids) == Some(&FolderTrouble::Missing)
    });
    quiet_for(&mut r, 500);
    assert!(
        all_ok(&r),
        "the file's folder is not there: nothing is marked"
    );
}

#[test]
fn an_empty_root_with_known_tracks_marks_nothing() {
    let dir = tempfile::tempdir().unwrap();
    // A mount point with nothing mounted: there, readable and empty.
    let mount = folder(&dir, "mnt");
    let mut ids = None;
    let mut r = rig_with(&[], dir, Duration::ZERO, |s| {
        ids = Some(create(s, "IDs", TrackKind::Jingle, Some(&mount)));
        apply(
            s,
            Command::AddToLibrary {
                paths: vec![mount.join("c.mp3")],
                category: ids,
            },
        )
        .unwrap();
        let id = s.library.iter().next().unwrap().id;
        let track = s.library.get_mut(id).unwrap();
        track.analyzed = true;
        track.tags_read = true;
        track.loudness_pass = Some(fp_model::loudness::LOUDNESS_VERSION);
        track.analysis_version = fp_analysis::cache::ANALYSIS_VERSION;
        s.config.tuning.missing_recheck_ms = 2_000.0;
    });
    let fake = FakeWatcher::default();
    r.services.set_watcher(Box::new(fake.clone()));
    let ids = ids.unwrap();
    r.run_until("reported missing", |r| {
        troubles(r).get(&ids) == Some(&FolderTrouble::Missing)
    });
    quiet_for(&mut r, 1_000);
    assert!(all_ok(&r), "an empty root marks nothing");
    // Mounted: the folder has files again.
    touch(&mount.join("c.mp3"));
    let until = r.now + Duration::from_millis(2_500);
    run_to(&mut r, until);
    r.run_until("cleared", |r| troubles(r).is_empty());
    assert!(all_ok(&r));
}

#[test]
fn dropping_the_outer_of_two_nested_folders_keeps_the_inner_followed() {
    let dir = tempfile::tempdir().unwrap();
    let music = folder(&dir, "m");
    let gold_folder = folder(&dir, "m/gold");
    touch(&music.join("1.mp3"));
    touch(&gold_folder.join("2.mp3"));
    let mut m = None;
    let (mut r, fake) = watching(dir, |s| {
        m = Some(create(s, "Music", TrackKind::Music, Some(&music)));
        create(s, "Gold", TrackKind::Music, Some(&gold_folder));
    });
    r.run_until("both watched", |_| fake.watched().len() == 2);
    r.handle.send(Command::SetWatchedFolder {
        category: m.unwrap(),
        folder: None,
    });
    // The system drops the inner watch with the outer one: it is set again.
    r.run_until("the inner one watched again", |_| {
        fake.watched() == vec![gold_folder.clone()]
    });
}

#[test]
fn dropping_the_inner_of_two_nested_folders_watches_the_outer_again() {
    use crate::support::WatchCall;
    let dir = tempfile::tempdir().unwrap();
    let music = folder(&dir, "m");
    let gold_folder = folder(&dir, "m/gold");
    touch(&music.join("1.mp3"));
    let mut g = None;
    let (mut r, fake) = watching(dir, |s| {
        create(s, "Music", TrackKind::Music, Some(&music));
        g = Some(create(s, "Gold", TrackKind::Music, Some(&gold_folder)));
    });
    r.run_until("both watched", |_| fake.watched().len() == 2);
    r.handle.send(Command::SetWatchedFolder {
        category: g.unwrap(),
        folder: None,
    });
    r.run_until("the outer one watched again", |_| {
        let calls = fake.calls();
        let unwatch = calls
            .iter()
            .position(|c| *c == WatchCall::Unwatch(gold_folder.clone()));
        unwatch.is_some_and(|at| calls[at..].contains(&WatchCall::Watch(music.clone())))
    });
}

#[test]
fn a_folder_lost_while_running_is_reported_and_followed_again() {
    use crate::support::WatchCall;
    let (keep, ids_folder, mut r, fake, ids) = settled_folder(2_000.0);
    // Unplugged without an event (the system drops its watch quietly).
    let away = keep.path().join("away");
    std::fs::rename(&ids_folder, &away).unwrap();
    fake.refuse(ids_folder.clone(), true);
    let until = r.now + Duration::from_millis(2_500);
    run_to(&mut r, until);
    r.run_until("reported missing", |r| {
        troubles(r).get(&ids) == Some(&FolderTrouble::Missing)
    });
    r.run_until("unwatched", |_| {
        fake.calls()
            .contains(&WatchCall::Unwatch(ids_folder.clone()))
    });
    assert!(all_ok(&r), "a lost folder marks nothing");
    // Plugged back in, with a new file.
    std::fs::rename(&away, &ids_folder).unwrap();
    fake.refuse(ids_folder.clone(), false);
    let new = wav(&ids_folder, "new.wav", 1);
    let until = r.now + Duration::from_millis(2_500);
    run_to(&mut r, until);
    r.run_until("followed and scanned again", |r| {
        troubles(r).is_empty()
            && track_of(r, &new).is_some()
            && fake.watched() == vec![ids_folder.clone()]
    });
    assert!(all_ok(&r));
}

/// A watcher whose every `watch` waits for a release, counting the calls.
struct Stuck {
    release: crossbeam_channel::Receiver<()>,
    calls: Arc<Mutex<usize>>,
}

impl FolderWatcher for Stuck {
    fn watch(&mut self, _: &Path) -> Result<(), String> {
        *self.calls.lock().unwrap() += 1;
        let _ = self.release.recv();
        Err("refused".into())
    }
    fn unwatch(&mut self, _: &Path) {}
    fn events(&mut self) -> Vec<FolderEvent> {
        Vec::new()
    }
}

#[test]
fn a_stuck_watch_call_does_not_pile_up_retries() {
    let dir = tempfile::tempdir().unwrap();
    let gone = dir.path().join("not-mounted");
    let (release_tx, release) = crossbeam_channel::unbounded();
    let calls = Arc::new(Mutex::new(0));
    let mut ids = None;
    let mut r = rig_with(&[], dir, Duration::ZERO, |s| {
        ids = Some(create(s, "IDs", TrackKind::Jingle, Some(&gone)));
        s.config.tuning.missing_recheck_ms = 1_000.0;
    });
    r.services.set_watcher(Box::new(Stuck {
        release,
        calls: calls.clone(),
    }));
    let ids = ids.unwrap();
    r.run_until("reported missing", |r| {
        troubles(r).get(&ids) == Some(&FolderTrouble::Missing)
    });
    // Five retries while the first call is stuck.
    let until = r.now + Duration::from_millis(5_500);
    run_to(&mut r, until);
    for _ in 0..20 {
        release_tx.send(()).unwrap();
    }
    quiet_for(&mut r, 300);
    assert!(
        *calls.lock().unwrap() <= 2,
        "{} watch calls",
        calls.lock().unwrap()
    );
}

#[test]
fn a_retry_walks_a_refused_folder_once() {
    let dir = tempfile::tempdir().unwrap();
    let ids_folder = folder(&dir, "ids");
    touch(&ids_folder.join("a.mp3"));
    let mut ids = None;
    let fake = FakeWatcher::default();
    fake.refuse(ids_folder.clone(), true);
    let mut r = rig_with(&[], dir, Duration::ZERO, |s| {
        ids = Some(create(s, "IDs", TrackKind::Jingle, Some(&ids_folder)));
        s.config.tuning.missing_recheck_ms = 1_000.0;
    });
    r.services.set_watcher(Box::new(fake.clone()));
    let ids = ids.unwrap();
    r.run_until("joined and flagged", |r| {
        tracks(r) == 1 && troubles(r).get(&ids) == Some(&FolderTrouble::NotFollowed)
    });
    quiet_for(&mut r, 300);
    fake.refuse(ids_folder.clone(), false);
    let scans = r.services.folder_scans();
    r.run_until("followed", |r| troubles(r).is_empty());
    quiet_for(&mut r, 500);
    assert_eq!(r.services.folder_scans(), scans + 1);
}

/// A watcher whose `watch` panics: the watch thread ends.
struct Broken;

impl FolderWatcher for Broken {
    fn watch(&mut self, _: &Path) -> Result<(), String> {
        panic!("the watcher broke");
    }
    fn unwatch(&mut self, _: &Path) {}
    fn events(&mut self) -> Vec<FolderEvent> {
        Vec::new()
    }
}

#[test]
fn a_dead_watch_thread_shows_every_folder_not_followed() {
    let dir = tempfile::tempdir().unwrap();
    let ids_folder = folder(&dir, "ids");
    let other = folder(&dir, "other");
    touch(&ids_folder.join("a.mp3"));
    let (mut ids, mut more) = (None, None);
    let mut r = rig_with(&[], dir, Duration::ZERO, |s| {
        ids = Some(create(s, "IDs", TrackKind::Jingle, Some(&ids_folder)));
        more = Some(create(s, "More", TrackKind::Jingle, None));
    });
    r.services.set_watcher(Box::new(Broken));
    let (ids, more) = (ids.unwrap(), more.unwrap());
    r.run_until("not followed", |r| {
        troubles(r).get(&ids) == Some(&FolderTrouble::NotFollowed)
    });
    r.handle.send(Command::SetWatchedFolder {
        category: more,
        folder: Some(other.clone()),
    });
    r.run_until("the new one too", |r| {
        troubles(r).get(&more) == Some(&FolderTrouble::NotFollowed)
    });
    r.run_until("still scanned", |r| tracks(r) == 1);
}

/// `/m` and `/m/gold` watched, a file analysed in each, the watch settled.
fn nested_settled(retry_ms: f64) -> (tempfile::TempDir, PathBuf, Rig, FakeWatcher, CategoryId) {
    let dir = tempfile::tempdir().unwrap();
    let music = folder(&dir, "m");
    let gold_folder = folder(&dir, "m/gold");
    wav(&music, "1.wav", 1);
    wav(&gold_folder, "2.wav", 1);
    let keep = tempfile::tempdir().unwrap();
    let mut g = None;
    let (mut r, fake) = watching(dir, |s| {
        create(s, "Music", TrackKind::Music, Some(&music));
        g = Some(create(s, "Gold", TrackKind::Music, Some(&gold_folder)));
        set_debounce(s, 100);
        s.config.tuning.missing_recheck_ms = retry_ms;
    });
    r.run_until("both analysed", |r| {
        let m = r.handle.model.load();
        m.library.len() == 2 && m.library.iter().all(|t| t.analyzed)
    });
    r.run_until("both watched", |_| fake.watched().len() == 2);
    quiet_for(&mut r, 300);
    (keep, gold_folder, r, fake, g.unwrap())
}

#[test]
fn a_lost_nested_folder_is_not_marked_by_the_outer_walk() {
    let (keep, gold_folder, mut r, fake, gold) = nested_settled(2_000.0);
    std::fs::rename(&gold_folder, keep.path().join("away")).unwrap();
    fake.refuse(gold_folder.clone(), true);
    let until = r.now + Duration::from_millis(2_500);
    run_to(&mut r, until);
    r.run_until("reported missing", |r| {
        troubles(r).get(&gold) == Some(&FolderTrouble::Missing)
    });
    // The outer folder is watched and walked again meanwhile.
    quiet_for(&mut r, 1_000);
    assert!(
        all_ok(&r),
        "the outer walk marks nothing under the lost folder"
    );
}

#[test]
fn an_unmounted_nested_folder_at_start_is_not_marked_by_the_outer_walk() {
    let dir = tempfile::tempdir().unwrap();
    let music = folder(&dir, "m");
    // A nested mount point with nothing mounted: there and empty.
    let gold_folder = folder(&dir, "m/gold");
    touch(&music.join("1.mp3"));
    let mut g = None;
    let mut r = rig_with(&[], dir, Duration::ZERO, |s| {
        create(s, "Music", TrackKind::Music, Some(&music));
        g = Some(create(s, "Gold", TrackKind::Music, Some(&gold_folder)));
        apply(
            s,
            Command::AddToLibrary {
                paths: vec![gold_folder.join("c.mp3")],
                category: g,
            },
        )
        .unwrap();
        let id = s.library.iter().next().unwrap().id;
        let track = s.library.get_mut(id).unwrap();
        track.analyzed = true;
        track.tags_read = true;
        track.loudness_pass = Some(fp_model::loudness::LOUDNESS_VERSION);
        track.analysis_version = fp_analysis::cache::ANALYSIS_VERSION;
    });
    let fake = FakeWatcher::default();
    r.services.set_watcher(Box::new(fake.clone()));
    let gold = g.unwrap();
    r.run_until("reported missing", |r| {
        troubles(r).get(&gold) == Some(&FolderTrouble::Missing)
    });
    quiet_for(&mut r, 1_000);
    let c = track_of(&r, &gold_folder.join("c.mp3")).unwrap();
    assert_eq!(c.file_state, FileState::Ok, "the outer walk marks nothing");
}

/// Puts a two-second file in place of `path` (a new size), never seen
/// half-written: as an editor that writes a copy and renames it.
fn replace_longer(path: &Path) {
    let staging = tempfile::tempdir().unwrap();
    let name = path.file_name().unwrap().to_str().unwrap();
    std::fs::rename(wav(staging.path(), name, 2), path).unwrap();
}

fn lasts_two_seconds(r: &Rig, path: &Path) -> bool {
    track_of(r, path).is_some_and(|t| (t.duration_secs - 2.0).abs() < 1e-6)
}

/// External editor spec EE7.2: an event on a file that is a known track
/// sends it for a change check; the changed file is analysed again, an
/// unchanged one is not.
#[test]
fn an_event_on_a_known_track_reports_it_touched() {
    use std::sync::atomic::Ordering;
    let dir = tempfile::tempdir().unwrap();
    let ids_folder = folder(&dir, "ids");
    let a = wav(&ids_folder, "a.wav", 1);
    let (mut r, fake) = watching(dir, |s| {
        create(s, "IDs", TrackKind::Jingle, Some(&ids_folder));
        set_debounce(s, 100);
    });
    settled(&mut r, &fake, 1);
    r.run_until("the start checks", |r| {
        r.services.change_checks_pending() == 0
    });
    let analyses = r.analyses.load(Ordering::SeqCst);
    push_and_step(&mut r, &fake, &a);
    quiet_for(&mut r, 500);
    assert_eq!(r.analyses.load(Ordering::SeqCst), analyses, "unchanged");
    replace_longer(&a);
    push_and_step(&mut r, &fake, &a);
    r.run_until("analysed again", |r| lasts_two_seconds(r, &a));
    assert_eq!(tracks(&r), 1);
}

/// EE7.2: an editor that saves by removing the file and writing it again
/// (or renaming a copy over it) may leave it gone when the scan looks. It is
/// looked at once more after the debounce before it counts as missing.
#[test]
fn a_file_replaced_within_the_debounce_is_not_gone() {
    let dir = tempfile::tempdir().unwrap();
    let ids_folder = folder(&dir, "ids");
    let a = wav(&ids_folder, "a.wav", 1);
    let (mut r, fake) = watching(dir, |s| {
        create(s, "IDs", TrackKind::Jingle, Some(&ids_folder));
        set_debounce(s, 1_000);
    });
    settled(&mut r, &fake, 1);
    let missing = |r: &Rig| track_of(r, &a).is_some_and(|t| t.file_state == FileState::Missing);
    std::fs::remove_file(&a).unwrap();
    push_and_step(&mut r, &fake, &a);
    r.run_until("the first look", |r| {
        r.services.folder_doubts() == 1 || missing(r)
    });
    assert!(!missing(&r), "not missing at the first look");
    replace_longer(&a);
    let mut seen_missing = false;
    r.run_until("analysed again", |r| {
        seen_missing |= missing(r);
        lasts_two_seconds(r, &a)
    });
    assert!(!seen_missing);
    assert_eq!(r.services.folder_doubts(), 0);
    assert_eq!(tracks(&r), 1);
}

/// EE7.3: the folder of a file opened in the external editor is followed
/// (not recursively) for the run, once, and a save there is noticed.
#[test]
fn an_edited_file_outside_watched_folders_is_followed_flat() {
    use crate::support::WatchCall;
    use fp_app::services::ServiceRequest;
    let dir = tempfile::tempdir().unwrap();
    let loose = folder(&dir, "loose");
    let a = wav(&loose, "a.wav", 1);
    let mut r = rig_with(std::slice::from_ref(&a), dir, Duration::ZERO, |s| {
        set_debounce(s, 100)
    });
    let fake = FakeWatcher::default();
    r.services.set_watcher(Box::new(fake.clone()));
    r.run_until("analysed", |r| track_of(r, &a).is_some_and(|t| t.analyzed));
    let id = track_of(&r, &a).unwrap().id;
    let requests = r.services.requests();
    requests.send(ServiceRequest::FollowEditedFile(id)).unwrap();
    requests.send(ServiceRequest::FollowEditedFile(id)).unwrap();
    r.run_until("followed flat", |_| {
        fake.calls().contains(&WatchCall::WatchFlat(loose.clone()))
    });
    quiet_for(&mut r, 300);
    assert_eq!(fake.calls(), vec![WatchCall::WatchFlat(loose.clone())]);
    replace_longer(&a);
    push_and_step(&mut r, &fake, &a);
    r.run_until("analysed again", |r| lasts_two_seconds(r, &a));
    // M10: a folder followed alone is not a watched folder: a new file
    // there joins nothing.
    let b = wav(&loose, "b.wav", 1);
    push_and_step(&mut r, &fake, &b);
    quiet_for(&mut r, 500);
    assert_eq!(
        tracks(&r),
        1,
        "a new file in a followed folder was imported"
    );
}

/// M9: a track found gone after an event, then found again by a walk (not
/// an event), is no longer in doubt.
#[test]
fn a_doubt_ends_when_a_walk_finds_the_file_again() {
    let dir = tempfile::tempdir().unwrap();
    let ids_folder = folder(&dir, "ids");
    let a = wav(&ids_folder, "a.wav", 1);
    let (mut r, fake) = watching(dir, |s| {
        create(s, "IDs", TrackKind::Jingle, Some(&ids_folder));
        set_debounce(s, 5_000);
    });
    settled(&mut r, &fake, 1);
    std::fs::remove_file(&a).unwrap();
    push_and_step(&mut r, &fake, &a);
    r.run_until("the first look", |r| r.services.folder_doubts() == 1);
    // The second look is due one debounce later; a walk comes first.
    let second_look = r.now + Duration::from_millis(5_000);
    replace_longer(&a);
    fake.push(FolderEvent::Rescan);
    r.run_until("the doubt cleared", |r| r.services.folder_doubts() == 0);
    assert!(r.now < second_look, "cleared only by the second look");
}

/// M9: a doubt about a track that left the library is dropped.
#[test]
fn a_doubt_about_a_removed_track_is_dropped() {
    let dir = tempfile::tempdir().unwrap();
    let ids_folder = folder(&dir, "ids");
    let a = wav(&ids_folder, "a.wav", 1);
    let (mut r, fake) = watching(dir, |s| {
        create(s, "IDs", TrackKind::Jingle, Some(&ids_folder));
        set_debounce(s, 5_000);
    });
    settled(&mut r, &fake, 1);
    let id = track_of(&r, &a).unwrap().id;
    std::fs::remove_file(&a).unwrap();
    push_and_step(&mut r, &fake, &a);
    r.run_until("the first look", |r| r.services.folder_doubts() == 1);
    let second_look = r.now + Duration::from_millis(5_000);
    assert!(r.handle.send(Command::RemoveTracks(vec![id])));
    r.run_until("the doubt dropped", |r| r.services.folder_doubts() == 0);
    assert!(r.now < second_look, "dropped only by the second look");
}

/// EE7.3: a file in a watched folder (here a subfolder of one) needs no
/// other watch; its save is noticed through the folder's.
#[test]
fn an_edited_file_in_a_watched_folder_adds_no_watch() {
    use fp_app::services::ServiceRequest;
    let dir = tempfile::tempdir().unwrap();
    let ids_folder = folder(&dir, "ids");
    let sub = folder(&dir, "ids/sub");
    let a = wav(&sub, "a.wav", 1);
    let (mut r, fake) = watching(dir, |s| {
        create(s, "IDs", TrackKind::Jingle, Some(&ids_folder));
        set_debounce(s, 100);
    });
    settled(&mut r, &fake, 1);
    let calls = fake.calls();
    let id = track_of(&r, &a).unwrap().id;
    let requests = r.services.requests();
    requests.send(ServiceRequest::FollowEditedFile(id)).unwrap();
    quiet_for(&mut r, 300);
    assert_eq!(fake.calls(), calls);
    replace_longer(&a);
    push_and_step(&mut r, &fake, &a);
    r.run_until("analysed again", |r| lasts_two_seconds(r, &a));
}
