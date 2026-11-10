#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! External editor spec EE3: launching the editor through the platform;
//! EE1, EE2, EE4, EE7.5, EE11, EE12: the menu entries, the review flag and
//! filter, and the settings.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use egui::accesskit::Role;
use egui::{Event, Modifiers, PointerButton, Pos2, Rect, pos2, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use fp_app::activity::AnalysisActivity;
use fp_app::i18n::I18n;
use fp_app::platform::Platform;
use fp_app::services::ServiceRequest;
use fp_app::ui::app::AppUi;
use fp_app::ui::controller::Controller;
use fp_app::ui::playlist_files::{FileOutcome, open_in_editor};
use fp_model::library_view::LibraryView;
use fp_model::{AppState, Command, LibrarySelection, MarkerKind, MarkersReview, TrackId};

use crate::support::{self, Fake, FakePlatform, activity_cell, harness_sized, state};

fn launch(platform: &Arc<FakePlatform>) -> crossbeam_channel::Receiver<FileOutcome> {
    let (tx, rx) = crossbeam_channel::unbounded();
    let dynamic: Arc<dyn Platform> = platform.clone();
    open_in_editor(
        &egui::Context::default(),
        &dynamic,
        PathBuf::from("/usr/bin/editor"),
        Path::new("/music/a.flac"),
        tx,
    );
    rx
}

#[test]
fn a_failed_launch_shows_a_notice() {
    let platform = Arc::new(FakePlatform {
        launch_result: Some("no such file".into()),
        ..FakePlatform::default()
    });
    let rx = launch(&platform);
    let Ok(FileOutcome::EditorFailed { program, error }) = rx.try_recv() else {
        panic!("a failure is reported");
    };
    assert_eq!(program, PathBuf::from("/usr/bin/editor"));
    assert_eq!(error, "no such file");
    let t = I18n::new(Some("en-US"));
    let text = t.tr_args(
        "notice-editor-failed",
        &[
            ("program", program.display().to_string().into()),
            ("error", error.into()),
        ],
    );
    assert!(text.contains("/usr/bin/editor") && text.contains("no such file"));
}

#[test]
fn a_started_editor_says_nothing() {
    let platform = Arc::new(FakePlatform::default());
    let rx = launch(&platform);
    assert!(rx.try_recv().is_err());
    assert_eq!(
        platform.launches.lock().unwrap().as_slice(),
        [(
            PathBuf::from("/usr/bin/editor"),
            PathBuf::from("/music/a.flac")
        )]
    );
}

#[cfg(unix)]
#[test]
fn the_desktop_reports_a_program_that_cannot_start() {
    let (tx, rx) = std::sync::mpsc::channel();
    fp_app::platform::DesktopPlatform.launch_editor(
        Path::new("/nonexistent/editor"),
        Path::new("/music/a.flac"),
        Box::new(move |r| tx.send(r).unwrap()),
    );
    let result = rx.recv_timeout(std::time::Duration::from_secs(10)).unwrap();
    assert!(result.is_err());
}

#[cfg(unix)]
#[test]
fn the_desktop_reports_a_started_program() {
    let (tx, rx) = std::sync::mpsc::channel();
    fp_app::platform::DesktopPlatform.launch_editor(
        // On every Unix system (it fails on the missing file later: only
        // the start is reported).
        Path::new("/bin/sh"),
        Path::new("/music/a.flac"),
        Box::new(move |r| tx.send(r).unwrap()),
    );
    let result = rx.recv_timeout(std::time::Duration::from_secs(10)).unwrap();
    assert_eq!(result, Ok(()));
}

// ---- The menu entries, the review flag and filter, the settings (UI) ----

type H = Harness<'static, AppUi>;

const OPEN: &str = "Open in external editor";
const REVIEWED: &str = "Mark as reviewed";
const EDITOR: &str = "/usr/bin/editor";

fn lib_path(name: &str) -> PathBuf {
    PathBuf::from(format!("/lib/{name}.mp3"))
}

/// One player whose playlist holds Song 1 to Song 3, the library tracks
/// `names` uncategorised, an editor set, and the library window open.
fn base(names: &[&str]) -> AppState {
    let mut s = state(1, 3);
    fp_model::apply(
        &mut s,
        Command::AddToLibrary {
            paths: names.iter().map(|n| lib_path(n)).collect(),
            category: None,
        },
    )
    .unwrap();
    s.config.library.external_editor = Some(PathBuf::from(EDITOR));
    s.library_view = LibraryView {
        open: true,
        rect: Some([20.0, 60.0, 600.0, 400.0]),
        ..LibraryView::default()
    };
    s
}

fn track(s: &AppState, name: &str) -> TrackId {
    s.library.track_at(&lib_path(name)).unwrap()
}

fn flag(s: &mut AppState, name: &str, dropped: Vec<MarkerKind>) {
    let id = track(s, name);
    s.library.get_mut(id).unwrap().markers_review = Some(MarkersReview { dropped });
}

fn platform(programs: bool) -> Arc<FakePlatform> {
    Arc::new(FakePlatform {
        programs,
        ..FakePlatform::default()
    })
}

/// The interface over `s` with `platform` and a services channel.
fn open_with(
    s: AppState,
    platform: Arc<FakePlatform>,
) -> (H, Arc<Fake>, crossbeam_channel::Receiver<ServiceRequest>) {
    let (tx, rx) = crossbeam_channel::unbounded();
    let (mut h, fake) = harness_sized(s, vec2(1200.0, 800.0), move |ui| {
        ui.with_platform(platform).with_services(tx)
    });
    h.run_steps(2);
    fake.take_sent();
    while rx.try_recv().is_ok() {}
    (h, fake, rx)
}

/// The commands sent, without the library views and the selected row.
fn others(sent: Vec<Command>) -> Vec<Command> {
    sent.into_iter()
        .filter(|c| !matches!(c, Command::SetLibraryView(_) | Command::SelectLibraryCue(_)))
        .collect()
}

/// The cell labelled `label` in the library window's table.
fn in_library(h: &H, label: &str) -> Rect {
    let window = h
        .state()
        .library_rect()
        .expect("the library window is open");
    let table = Rect::from_min_max(
        pos2(window.left() + 240.0, window.top() + 28.0 + 130.0),
        window.max,
    );
    let found: Vec<Rect> = h
        .query_all_by_label(label)
        .map(|n| n.rect())
        .filter(|r| table.contains(r.center()))
        .collect();
    assert_eq!(found.len(), 1, "{label}: {found:?}");
    found[0]
}

fn click_at(h: &mut H, at: Pos2, modifiers: Modifiers, button: PointerButton) {
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    for pressed in [true, false] {
        h.event(Event::PointerButton {
            pos: at,
            button,
            pressed,
            modifiers,
        });
    }
    h.run_steps(2);
}

/// Selects the library rows titled `names` (Ctrl+click after the first).
fn select(h: &mut H, names: &[&str]) {
    for (n, name) in names.iter().enumerate() {
        let at = in_library(h, name).center();
        let modifiers = if n == 0 {
            Modifiers::NONE
        } else {
            Modifiers::COMMAND
        };
        click_at(h, at, modifiers, PointerButton::Primary);
    }
}

/// Opens the row menu on the library row titled `name`.
fn menu(h: &mut H, name: &str) {
    let at = in_library(h, name).center();
    click_at(h, at, Modifiers::NONE, PointerButton::Secondary);
}

/// The menu entry labelled `label`: the one furthest right.
fn item(h: &H, label: &str) -> Option<Rect> {
    h.query_all_by_label(label)
        .map(|n| n.rect())
        .max_by(|a, b| a.left().total_cmp(&b.left()))
}

fn disabled(h: &H, label: &str) -> bool {
    h.query_all_by_label(label)
        .max_by(|a, b| a.rect().left().total_cmp(&b.rect().left()))
        .unwrap_or_else(|| panic!("no menu entry {label}"))
        .accesskit_node()
        .is_disabled()
}

/// EE2: Open in external editor acts on one track: shown for one row,
/// not for several.
#[test]
fn the_menu_offers_the_editor_for_one_track() {
    let s = base(&["Lib A", "Lib B"]);
    let (mut h, _fake, _rx) = open_with(s, platform(true));
    menu(&mut h, "Lib A");
    assert!(item(&h, OPEN).is_some(), "one track");
    assert!(!disabled(&h, OPEN));
    h.event(Event::Key {
        key: egui::Key::Escape,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(2);
    select(&mut h, &["Lib A", "Lib B"]);
    menu(&mut h, "Lib B");
    assert!(item(&h, "Add to").is_some(), "the menu is open");
    assert!(item(&h, OPEN).is_none(), "not for several tracks");
}

/// EE4: dimmed, with the reason, without an editor and while the track
/// sounds.
#[test]
fn the_editor_entry_is_dimmed_without_an_editor() {
    let mut s = base(&["Lib A"]);
    s.config.library.external_editor = None;
    let (mut h, _fake, _rx) = open_with(s, platform(true));
    menu(&mut h, "Lib A");
    assert!(disabled(&h, OPEN));
    let at = item(&h, OPEN).unwrap().center();
    h.event(Event::PointerMoved(at));
    h.run_steps(40);
    assert!(
        h.query_by_label_contains("No external editor is set")
            .is_some()
    );

    // The playlist's Song 1 is on air (its row menu, the library closed).
    let mut s = base(&[]);
    s.library_view.open = false;
    let p = s.players[0].id;
    let e = s.playlists.iter().next().unwrap().entries[0].id;
    fp_model::apply(&mut s, Command::SetNext(p, e)).unwrap();
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    let (mut h, _fake, _rx) = open_with(s, platform(true));
    h.get_all_by_label("Song 1")
        .max_by(|a, b| a.rect().min.y.total_cmp(&b.rect().min.y))
        .unwrap()
        .click_secondary();
    h.run_steps(3);
    assert!(disabled(&h, OPEN));
    let at = item(&h, OPEN).unwrap().center();
    h.event(Event::PointerMoved(at));
    h.run_steps(40);
    assert!(
        h.query_by_label_contains("cannot be opened in the editor while it plays")
            .is_some()
    );
}

/// EE2, §6: where programs cannot be started (Android, Flatpak, snap) the
/// entry is not there at all.
#[test]
fn the_editor_entry_is_absent_when_programs_cannot_be_launched() {
    let s = base(&["Lib A"]);
    let (mut h, _fake, _rx) = open_with(s, platform(false));
    menu(&mut h, "Lib A");
    assert!(item(&h, "Add to").is_some(), "the menu is open");
    assert!(item(&h, OPEN).is_none());
}

/// EE3, EE7.3: the entry starts the editor on the file and asks the
/// services to follow its folder.
#[test]
fn opening_asks_to_follow_the_file() {
    let s = base(&["Lib A"]);
    let a = track(&s, "Lib A");
    let fake_platform = platform(true);
    let (mut h, _fake, rx) = open_with(s, fake_platform.clone());
    menu(&mut h, "Lib A");
    let at = item(&h, OPEN).unwrap().center();
    click_at(&mut h, at, Modifiers::NONE, PointerButton::Primary);
    assert_eq!(
        fake_platform.launches.lock().unwrap().as_slice(),
        [(PathBuf::from(EDITOR), lib_path("Lib A"))]
    );
    let requests: Vec<ServiceRequest> = rx.try_iter().collect();
    assert!(
        requests.contains(&ServiceRequest::FollowEditedFile(a)),
        "{requests:?}"
    );
}

/// EE12: Mark as reviewed is offered when a selected track is flagged and
/// clears the flagged ones; it is not offered on a track without the flag.
#[test]
fn mark_as_reviewed_sends_the_flagged_tracks() {
    let mut s = base(&["Lib A", "Lib B", "Lib C"]);
    flag(&mut s, "Lib A", Vec::new());
    flag(&mut s, "Lib C", vec![MarkerKind::CueOut]);
    let (a, c) = (track(&s, "Lib A"), track(&s, "Lib C"));
    let (mut h, fake, _rx) = open_with(s, platform(true));
    menu(&mut h, "Lib B");
    assert!(item(&h, "Add to").is_some(), "the menu is open");
    assert!(item(&h, REVIEWED).is_none(), "Lib B is not flagged");
    h.event(Event::Key {
        key: egui::Key::Escape,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(2);
    select(&mut h, &["Lib A", "Lib B", "Lib C"]);
    menu(&mut h, "Lib B");
    let at = item(&h, REVIEWED).unwrap().center();
    click_at(&mut h, at, Modifiers::NONE, PointerButton::Primary);
    assert_eq!(
        others(fake.take_sent()),
        vec![Command::MarkReviewed { tracks: vec![a, c] }]
    );
}

/// EE11: a flagged row shows the warning with the markers dropped; one
/// with none dropped still says to check them.
#[test]
fn a_flagged_row_shows_the_review_flag_with_the_dropped_markers() {
    let mut s = base(&["Lib A", "Lib B"]);
    flag(
        &mut s,
        "Lib A",
        vec![MarkerKind::CueOut, MarkerKind::IntroEnd],
    );
    let (h, _fake, _rx) = open_with(s, platform(true));
    let t = I18n::new(Some("en-US"));
    let dropped = t.tr_args(
        "track-markers-review-tip-dropped",
        &[("markers", "cue out, intro end".into())],
    );
    assert!(dropped.contains("cue out, intro end"), "{dropped}");
    assert_eq!(h.query_all_by_label(&dropped).count(), 1, "one flagged row");
    assert_eq!(
        h.query_all_by_label_contains("The audio of this file changed")
            .count(),
        1
    );

    let mut s = base(&["Lib A"]);
    flag(&mut s, "Lib A", Vec::new());
    let (h, _fake, _rx) = open_with(s, platform(true));
    let none = t.tr("track-markers-review-tip");
    assert_eq!(
        none,
        "The audio of this file changed after its markers were set by hand. \
         Check them, then mark the track as reviewed."
    );
    assert_eq!(h.query_all_by_label(&none).count(), 1);
}

/// EE11: *Markers to review* is in the tree while a track is flagged and
/// lists the flagged tracks only; once none is left the view returns to
/// *All tracks*.
#[test]
fn the_review_filter_lists_only_flagged_tracks() {
    let (h, _fake, _rx) = open_with(base(&["Lib A", "Lib B"]), platform(true));
    assert!(
        h.query_by_role_and_label(Role::TreeItem, "Markers to review")
            .is_none(),
        "nothing flagged, no filter"
    );

    let mut s = base(&["Lib A", "Lib B", "Lib C"]);
    flag(&mut s, "Lib B", Vec::new());
    let b = track(&s, "Lib B");
    let (mut h, fake, _rx) = open_with(s, platform(true));
    h.get_by_role_and_label(Role::TreeItem, "Markers to review")
        .click();
    h.run_steps(3);
    assert_eq!(
        fake.state.load().library_view.selection,
        LibrarySelection::MarkersToReview
    );
    assert!(h.query_all_by_label("Lib A").next().is_none());
    assert!(h.query_all_by_label("Lib C").next().is_none());
    in_library(&h, "Lib B");

    fake.send(Command::MarkReviewed { tracks: vec![b] });
    h.run_steps(4);
    assert_eq!(
        fake.state.load().library_view.selection,
        LibrarySelection::All,
        "an empty filter is not left selected"
    );
    assert!(
        h.query_by_role_and_label(Role::TreeItem, "Markers to review")
            .is_none()
    );
}

fn settings_section(h: &mut H, section: &str) {
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, section).click();
    h.run_steps(3);
}

/// EE1: the editor's row in Settings → Library, hidden where programs
/// cannot be started; Clear forgets the program.
#[test]
fn the_settings_row_is_hidden_when_programs_cannot_be_launched() {
    let (mut h, _fake, _rx) = open_with(base(&[]), platform(false));
    settings_section(&mut h, "Library");
    assert!(h.query_by_label("External editor").is_none());
    assert!(h.query_by_label("Choose…").is_none());

    let (mut h, fake, _rx) = open_with(base(&[]), platform(true));
    settings_section(&mut h, "Library");
    assert!(h.query_by_label("External editor").is_some());
    assert!(h.query_by_label(EDITOR).is_some(), "the path is shown");
    h.get_by_role_and_label(Role::Button, "Clear").click();
    h.run_steps(2);
    assert_eq!(fake.state.load().config.library.external_editor, None);
    h.run_steps(2);
    assert!(h.query_by_label(EDITOR).is_none());
    assert!(
        h.query_by_label("None").is_some(),
        "a placeholder, not blank"
    );
}

/// EE1: Choose… asks the platform for a file off the interface thread.
#[test]
fn choose_asks_for_a_file() {
    let fake_platform = platform(true);
    let (mut h, _fake, _rx) = open_with(base(&[]), fake_platform.clone());
    settings_section(&mut h, "Library");
    h.get_by_role_and_label(Role::Button, "Choose…").click();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while !fake_platform
        .asked
        .lock()
        .unwrap()
        .contains(&"file".to_owned())
    {
        assert!(std::time::Instant::now() < deadline, "no file dialog");
        h.run_steps(1);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

/// EE7.5: Look for changed files sends the request, and waits while a
/// pass runs.
#[test]
fn look_for_changed_files_sends_the_request() {
    let cell = activity_cell(AnalysisActivity {
        looking: true,
        ..AnalysisActivity::default()
    });
    let shared = cell.clone();
    let (tx, rx) = crossbeam_channel::unbounded();
    let (mut h, _) = harness_sized(state(1, 3), vec2(1000.0, 1600.0), move |ui| {
        ui.with_services(tx).with_analysis_activity(shared)
    });
    h.run_steps(2);
    settings_section(&mut h, "Analysis");
    let label = "Look for changed files";
    h.get_by_role_and_label(Role::Button, label).scroll_to_me();
    h.run_steps(3);
    assert!(
        h.get_by_role_and_label(Role::Button, label)
            .accesskit_node()
            .is_disabled()
    );
    assert!(
        !rx.try_iter()
            .any(|r| r == ServiceRequest::LookForChangedFiles),
        "no request while a pass runs"
    );
    cell.store(Arc::new(AnalysisActivity::default()));
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, label).click();
    h.run_steps(2);
    assert!(
        rx.try_iter()
            .any(|r| r == ServiceRequest::LookForChangedFiles)
    );
}

/// EE7.5: the count of a finished pass is told once.
#[test]
fn a_finished_look_tells_how_many_files_changed() {
    let cell = activity_cell(AnalysisActivity::default());
    let shared = cell.clone();
    let (mut h, _) = harness_sized(state(1, 3), vec2(1000.0, 700.0), move |ui| {
        ui.with_analysis_activity(shared)
    });
    h.run_steps(2);
    cell.store(Arc::new(AnalysisActivity {
        changed_found: Some((1, 2)),
        ..AnalysisActivity::default()
    }));
    h.run_steps(2);
    assert!(
        h.query_by_label_contains("Changed files found: 2")
            .is_some()
    );
}

/// EE2, EE12: a playlist row's menu has the same two entries, for its
/// entry's track.
#[test]
fn the_playlist_row_menu_has_the_same_entries() {
    let mut s = state(1, 3);
    s.config.library.external_editor = Some(PathBuf::from(EDITOR));
    let song = support::nth_track(&s, 0);
    s.library.get_mut(song).unwrap().markers_review = Some(MarkersReview {
        dropped: Vec::new(),
    });
    let fake_platform = platform(true);
    let (mut h, fake, rx) = open_with(s, fake_platform.clone());
    let row = || "Song 1";
    h.get_all_by_label(row())
        .max_by(|a, b| a.rect().min.y.total_cmp(&b.rect().min.y))
        .unwrap()
        .click_secondary();
    h.run_steps(3);
    assert!(!disabled(&h, OPEN));
    let at = item(&h, REVIEWED).unwrap().center();
    click_at(&mut h, at, Modifiers::NONE, PointerButton::Primary);
    assert_eq!(
        others(fake.take_sent()),
        vec![Command::MarkReviewed { tracks: vec![song] }]
    );
    h.get_all_by_label(row())
        .max_by(|a, b| a.rect().min.y.total_cmp(&b.rect().min.y))
        .unwrap()
        .click_secondary();
    h.run_steps(3);
    let at = item(&h, OPEN).unwrap().center();
    click_at(&mut h, at, Modifiers::NONE, PointerButton::Primary);
    assert_eq!(
        fake_platform.launches.lock().unwrap().as_slice(),
        [(PathBuf::from(EDITOR), PathBuf::from("/music/Song 1.mp3"))]
    );
    assert!(
        rx.try_iter()
            .any(|r| r == ServiceRequest::FollowEditedFile(song))
    );
}

/// EE7.4: opening the markers for editing (the waveform's menu, here the
/// CUE window's) asks for the track's file to be checked, once per
/// opening; merely showing a waveform does not.
#[test]
fn opening_the_marker_menu_asks_for_a_check_of_its_track_once() {
    let mut s = state(1, 3);
    for id in s.library.ids() {
        s.library.get_mut(id).unwrap().duration_secs = 180.0;
    }
    let (tx, rx) = crossbeam_channel::unbounded();
    let (mut h, fake) = harness_sized(s, vec2(1000.0, 700.0), move |ui| ui.with_services(tx));
    fake.send(Command::ToggleCue(fake.player(0)));
    h.run_steps(5);
    let checks = |rx: &crossbeam_channel::Receiver<ServiceRequest>| -> Vec<TrackId> {
        rx.try_iter()
            .filter_map(|r| match r {
                ServiceRequest::CheckTrack(t) => Some(t),
                _ => None,
            })
            .collect()
    };
    assert_eq!(checks(&rx), Vec::<TrackId>::new(), "nothing opened yet");
    let cued = {
        let state = fake.state.load();
        let entry = state.players[0].cue.as_ref().map(|c| c.entry).unwrap();
        state.playlists.entry(entry).unwrap().track
    };
    let wave = h.get_by_label("CUE waveform: click to seek").rect();
    let at = pos2(wave.left() + wave.width() * 0.25, wave.center().y);
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    for pressed in [true, false] {
        h.event(Event::PointerButton {
            pos: at,
            button: PointerButton::Secondary,
            pressed,
            modifiers: Modifiers::NONE,
        });
    }
    h.run_steps(4);
    assert!(
        h.query_by_label("Set cue in here").is_some(),
        "the menu is open"
    );
    assert_eq!(checks(&rx), vec![cued]);
}
