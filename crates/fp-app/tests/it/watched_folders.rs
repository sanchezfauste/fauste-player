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

/// A folder under the test's directory, in its canonical form.
fn folder(dir: &tempfile::TempDir, name: &str) -> PathBuf {
    let path = dir.path().join(name);
    std::fs::create_dir_all(&path).unwrap();
    std::fs::canonicalize(&path).unwrap()
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
