#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Track library spec §6 (LB19–LB23): the ⟳ mark and the menu items of
//! watched folders, and *Remove missing tracks…*.

use crate::support;

use std::path::PathBuf;
use std::sync::Arc;

use egui::vec2;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::AppUi;
use fp_app::ui::files::canonical_path;
use fp_app::watch::{FolderTrouble, WatchCell, WatchStatus, idle_watch_cell};
use fp_model::library_view::LibraryView;
use fp_model::{AppState, CategoryId, Command, FileState, TrackKind};
use support::{Fake, FakePlatform, harness_sized, state};

type H = Harness<'static, AppUi>;

struct Tree {
    rock: CategoryId,
    jingles: CategoryId,
}

fn create(s: &mut AppState, name: &str) -> CategoryId {
    fp_model::apply(
        s,
        Command::CreateCategory {
            parent: None,
            name: name.into(),
            color: None,
            kind: TrackKind::Music,
        },
    )
    .unwrap();
    s.categories.iter().find(|c| c.name == name).unwrap().id
}

/// The folder *Rock* watches: absolute on every system (a path without a
/// drive is relative on Windows, and ignored).
fn rock_folder() -> PathBuf {
    if cfg!(windows) {
        PathBuf::from(r"C:\m\rock")
    } else {
        PathBuf::from("/m/rock")
    }
}

/// *Rock* (watching [`rock_folder`], Song 1 in it) and *Jingles* (no folder),
/// the library window open.
fn tree_state() -> (AppState, Tree) {
    let mut s = state(1, 3);
    let rock = create(&mut s, "Rock");
    let jingles = create(&mut s, "Jingles");
    let song = support::nth_track(&s, 0);
    fp_model::apply(
        &mut s,
        Command::MoveTracksToCategory {
            tracks: vec![song],
            category: Some(rock),
        },
    )
    .unwrap();
    fp_model::apply(
        &mut s,
        Command::SetWatchedFolder {
            category: rock,
            folder: Some(rock_folder()),
        },
    )
    .unwrap();
    s.library_view = LibraryView {
        open: true,
        rect: Some([20.0, 60.0, 900.0, 600.0]),
        ..LibraryView::default()
    };
    (s, Tree { rock, jingles })
}

fn open_with(s: AppState, platform: Arc<FakePlatform>, cell: WatchCell) -> (H, Arc<Fake>) {
    let (mut h, fake) = harness_sized(s, vec2(1200.0, 800.0), move |ui| {
        ui.with_platform(platform).with_watch_status(cell)
    });
    h.run_steps(2);
    fake.take_sent();
    (h, fake)
}

fn open(s: AppState) -> (H, Arc<Fake>) {
    open_with(s, Arc::new(FakePlatform::default()), idle_watch_cell())
}

fn others(sent: Vec<Command>) -> Vec<Command> {
    sent.into_iter()
        .filter(|c| !matches!(c, Command::SetLibraryView(_)))
        .collect()
}

fn menu(h: &mut H, item: &str) {
    h.get_by_label(item).click_secondary();
    h.run_steps(2);
}

#[test]
fn a_watching_category_shows_the_mark_with_its_folder() {
    let (s, _) = tree_state();
    let (h, _) = open(s);
    let label = format!("Watches {}", rock_folder().display());
    assert!(h.query_by_label(&label).is_some());
    assert_eq!(h.query_all_by_label_contains("Watches").count(), 1);
}

#[test]
fn a_folder_in_trouble_shows_why() {
    let (s, tree) = tree_state();
    let cell = idle_watch_cell();
    let mut status = WatchStatus::default();
    status.troubles.insert(tree.rock, FolderTrouble::Missing);
    cell.store(Arc::new(status));
    let (h, _) = open_with(s, Arc::new(FakePlatform::default()), cell);
    let label = format!(
        "Watched folder not found: {}. It is looked for again.",
        rock_folder().display()
    );
    assert!(h.query_by_label(&label).is_some());
}

/// Steps the harness until `done` or about a second has passed.
fn wait(h: &mut H, fake: &Fake, mut done: impl FnMut(&[Command]) -> bool) -> Vec<Command> {
    let mut sent = Vec::new();
    for _ in 0..200 {
        h.run_steps(1);
        sent.extend(fake.take_sent());
        if done(&sent) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    sent
}

#[test]
fn watch_folder_picks_on_a_helper_thread_and_sends_the_command() {
    let (s, tree) = tree_state();
    let dir = tempfile::tempdir().unwrap();
    let platform = Arc::new(FakePlatform {
        folder: Some(dir.path().to_path_buf()),
        ..FakePlatform::default()
    });
    let (mut h, fake) = open_with(s, platform.clone(), idle_watch_cell());
    menu(&mut h, "Jingles");
    h.get_by_label("Watch folder…").click();
    let sent = wait(&mut h, &fake, |s| {
        s.iter()
            .any(|c| matches!(c, Command::SetWatchedFolder { .. }))
    });
    assert_eq!(
        others(sent),
        vec![Command::SetWatchedFolder {
            category: tree.jingles,
            folder: Some(canonical_path(dir.path())),
        }]
    );
    assert_eq!(*platform.asked.lock().unwrap(), vec!["folder".to_owned()]);
}

#[test]
fn a_cancelled_picker_sends_nothing() {
    let (s, _) = tree_state();
    let platform = Arc::new(FakePlatform::default());
    let (mut h, fake) = open_with(s, platform.clone(), idle_watch_cell());
    menu(&mut h, "Jingles");
    h.get_by_label("Watch folder…").click();
    for _ in 0..40 {
        h.run_steps(1);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(*platform.asked.lock().unwrap(), vec!["folder".to_owned()]);
    assert_eq!(others(fake.take_sent()), vec![]);
}

#[test]
fn stop_watching_sends_none() {
    let (s, tree) = tree_state();
    let (mut h, fake) = open(s);
    menu(&mut h, "Rock");
    assert!(h.query_by_label("Change watched folder…").is_some());
    h.get_by_label("Stop watching folder").click();
    h.run_steps(2);
    assert_eq!(
        others(fake.take_sent()),
        vec![Command::SetWatchedFolder {
            category: tree.rock,
            folder: None,
        }]
    );
    menu(&mut h, "Jingles");
    assert!(h.query_by_label("Watch folder…").is_some());
    assert!(h.query_by_label("Stop watching folder").is_none());
    assert!(h.query_by_label("Change watched folder…").is_none());
}

/// Rock gets two missing tracks nobody uses and a missing one in a
/// playlist.
fn with_missing(s: &mut AppState, tree: &Tree) {
    fp_model::apply(
        s,
        Command::AddToLibrary {
            paths: vec!["/m/a.mp3".into(), "/m/b.mp3".into()],
            category: Some(tree.rock),
        },
    )
    .unwrap();
    for id in s.library.ids() {
        let unused = s.library.get(id).is_some_and(|t| t.path.starts_with("/m/"));
        let used = id == support::nth_track(s, 0);
        if unused || used {
            s.library.get_mut(id).unwrap().file_state = FileState::Missing;
        }
    }
}

#[test]
fn remove_missing_asks_with_the_count_then_sends() {
    let (mut s, tree) = tree_state();
    with_missing(&mut s, &tree);
    let (mut h, fake) = open(s);
    menu(&mut h, "Rock");
    h.get_by_label("Remove missing tracks…").click();
    h.run_steps(3);
    assert!(
        h.query_by_label_contains("2 tracks of Rock have a missing file")
            .is_some()
    );
    h.get_by_label("Cancel").click();
    h.run_steps(2);
    assert_eq!(others(fake.take_sent()), vec![]);
    menu(&mut h, "Rock");
    h.get_by_label("Remove missing tracks…").click();
    h.run_steps(3);
    h.get_by_label("Remove").click();
    h.run_steps(2);
    assert_eq!(
        others(fake.take_sent()),
        vec![Command::RemoveMissingTracks(tree.rock)]
    );
}

#[test]
fn remove_missing_with_none_is_a_notice() {
    let (s, _) = tree_state();
    let (mut h, fake) = open(s);
    menu(&mut h, "Rock");
    h.get_by_label("Remove missing tracks…").click();
    h.run_steps(3);
    assert!(h.query_by_label("Cancel").is_none(), "no question");
    assert!(
        h.query_by_label("Rock has no unused tracks with a missing file.")
            .is_some()
    );
    assert_eq!(others(fake.take_sent()), vec![]);
}

#[test]
fn settings_library_offers_the_same_watch_items() {
    let (s, _) = tree_state();
    let (mut h, _) = open(s);
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(egui::accesskit::Role::Button, "Library")
        .click();
    h.run_steps(3);
    let rocks: Vec<_> = h.query_all_by_label("Rock").collect();
    rocks.last().unwrap().click_secondary();
    h.run_steps(2);
    assert!(h.query_by_label("Stop watching folder").is_some());
}

#[test]
fn an_ignored_stored_folder_shows_a_trouble_not_a_watch() {
    let (mut s, tree) = tree_state();
    fp_model::apply(
        &mut s,
        Command::SetWatchedFolder {
            category: tree.rock,
            folder: Some(PathBuf::from("music/relative")),
        },
    )
    .unwrap();
    let (h, _) = open(s);
    assert!(h.query_all_by_label_contains("Watches").next().is_none());
    assert!(
        h.query_by_label("Watched folder is not a full path: music/relative. It is not watched.")
            .is_some()
    );
}

#[test]
fn a_folder_another_category_watches_is_refused_naming_it() {
    let (mut s, tree) = tree_state();
    let dir = tempfile::tempdir().unwrap();
    fp_model::apply(
        &mut s,
        Command::SetWatchedFolder {
            category: tree.rock,
            folder: Some(canonical_path(dir.path())),
        },
    )
    .unwrap();
    let platform = Arc::new(FakePlatform {
        folder: Some(dir.path().to_path_buf()),
        ..FakePlatform::default()
    });
    let (mut h, fake) = open_with(s, platform, idle_watch_cell());
    menu(&mut h, "Jingles");
    h.get_by_label("Watch folder…").click();
    wait(&mut h, &fake, |s| {
        s.iter()
            .any(|c| matches!(c, Command::SetWatchedFolder { .. }))
    });
    h.run_steps(2);
    assert!(
        h.query_by_label_contains("“Rock” already watches this folder.")
            .is_some()
    );
}

/// LB52: where the device's music is listed (Android), the menu offers a
/// device folder instead of watching, and the tree names it without
/// trouble marks.
fn open_device(s: AppState) -> (H, Arc<Fake>) {
    let browser = fp_app::ui::media_browser::MediaBrowser::default();
    browser.set_files(vec![fp_app::ui::media_browser::MediaFile {
        path: rock_folder().join("one.mp3"),
        ..Default::default()
    }]);
    let (mut h, fake) = harness_sized(s, vec2(1200.0, 800.0), move |ui| {
        ui.with_media_browser(browser)
    });
    h.run_steps(2);
    fake.take_sent();
    (h, fake)
}

#[test]
fn lb52_the_menu_offers_a_device_folder() {
    let (s, tree) = tree_state();
    let (mut h, fake) = open_device(s);
    let label = format!("Takes files from {}", rock_folder().display());
    assert!(h.query_by_label(&label).is_some());
    assert!(h.query_by_label_contains("Watches").is_none());
    menu(&mut h, "Jingles");
    assert!(h.query_by_label("Use a device folder…").is_some());
    assert!(h.query_by_label("Watch folder…").is_none());
    assert!(h.query_by_label("Re-read device folder").is_none());
    h.key_press(egui::Key::Escape);
    h.run_steps(2);
    menu(&mut h, "Rock");
    assert!(h.query_by_label("Change device folder…").is_some());
    h.get_by_label("Forget device folder").click();
    h.run_steps(2);
    assert_eq!(
        others(fake.take_sent()),
        vec![Command::SetWatchedFolder {
            category: tree.rock,
            folder: None,
        }]
    );
}

#[test]
fn lb53_the_menu_re_reads_the_device_folder() {
    let (s, tree) = tree_state();
    let (mut h, fake) = open_device(s);
    menu(&mut h, "Rock");
    h.get_by_label("Re-read device folder").click();
    h.run_steps(2);
    assert_eq!(
        others(fake.take_sent()),
        vec![Command::AddToLibrary {
            paths: vec![rock_folder().join("one.mp3")],
            category: Some(tree.rock),
        }]
    );
}
