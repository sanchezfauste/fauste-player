#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Track library spec §7.4, LB17: the library table's row menu, and
//! library rows dragged to a playlist or a cart.

use std::path::PathBuf;
use std::sync::Arc;

use crate::support;

use egui::accesskit::Role;
use egui::{Event, Modifiers, PointerButton, Pos2, Rect, pos2, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use fp_app::ui::app::AppUi;
use fp_model::library_view::LibraryView;
use fp_model::{AppState, CategoryId, Command, TrackId, TrackKind};
use support::{Fake, FakePlatform, harness_sized, state};

type H = Harness<'static, AppUi>;

fn lib_path(name: &str) -> PathBuf {
    PathBuf::from(format!("/lib/{name}.mp3"))
}

/// One player whose playlist holds Song 1 to Song 3 (library tracks too),
/// the library tracks `names` uncategorised, and the library window open
/// on the upper left, clear of the cartwall and of the right of the
/// playlist.
fn base(names: &[&str]) -> AppState {
    let mut s = state(1, 3);
    if !names.is_empty() {
        fp_model::apply(
            &mut s,
            Command::AddToLibrary {
                paths: names.iter().map(|n| lib_path(n)).collect(),
                category: None,
            },
        )
        .unwrap();
    }
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

fn open_with(s: AppState, platform: Option<Arc<FakePlatform>>) -> (H, Arc<Fake>) {
    let (mut h, fake) = harness_sized(s, vec2(1200.0, 800.0), move |ui| match platform {
        Some(p) => ui.with_platform(p),
        None => ui,
    });
    h.run_steps(2);
    fake.take_sent();
    (h, fake)
}

fn open(s: AppState) -> (H, Arc<Fake>) {
    open_with(s, None)
}

/// The commands sent, without the library views.
fn others(sent: Vec<Command>) -> Vec<Command> {
    sent.into_iter()
        .filter(|c| !matches!(c, Command::SetLibraryView(_)))
        .collect()
}

fn library_rect(h: &H) -> Rect {
    h.state()
        .library_rect()
        .expect("the library window is open")
}

/// The cell labelled `label` in the library window's table (right of its
/// category tree; the decks under the window name tracks too).
fn in_library(h: &H, label: &str) -> Rect {
    let window = library_rect(h);
    let table = Rect::from_min_max(pos2(window.left() + 240.0, window.top()), window.max);
    let found: Vec<Rect> = h
        .query_all_by_label(label)
        .map(|n| n.rect())
        .filter(|r| table.contains(r.center()))
        .collect();
    assert_eq!(found.len(), 1, "{label}: {found:?}");
    found[0]
}

/// The playlist cell labelled `label`: the lowest of its name on screen
/// (the decks above name the next track; the playlist is below the
/// library's rows here).
fn in_playlist(h: &H, label: &str) -> Rect {
    h.query_all_by_label(label)
        .map(|n| n.rect())
        .max_by(|a, b| a.top().total_cmp(&b.top()))
        .unwrap_or_else(|| panic!("no playlist cell {label}"))
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

/// The menu entry labelled `label`: the one furthest right (menus open
/// right of the tree, at the pointer).
fn item(h: &H, label: &str) -> Rect {
    h.query_all_by_label(label)
        .map(|n| n.rect())
        .max_by(|a, b| a.left().total_cmp(&b.left()))
        .unwrap_or_else(|| panic!("no menu entry {label}"))
}

fn choose(h: &mut H, path: &[&str]) {
    for label in path {
        let at = item(h, label).center();
        click_at(h, at, Modifiers::NONE, PointerButton::Primary);
    }
}

fn press(h: &mut H, at: Pos2, pressed: bool) {
    h.event(Event::PointerButton {
        pos: at,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    });
}

/// Presses at `from`, moves to `to` in steps and releases there.
fn drag(h: &mut H, from: Pos2, to: Pos2) {
    h.event(Event::PointerMoved(from));
    h.run_steps(1);
    press(h, from, true);
    h.run_steps(1);
    for step in 1..=10 {
        let t = step as f32 / 10.0;
        h.event(Event::PointerMoved(pos2(
            from.x + (to.x - from.x) * t,
            from.y + (to.y - from.y) * t,
        )));
        h.run_steps(1);
    }
    h.run_steps(2);
    press(h, to, false);
    h.run_steps(3);
}

#[test]
fn add_to_sends_insert_tracks_at_the_end() {
    let s = base(&["Lib A", "Lib B", "Lib C"]);
    let (a, c) = (track(&s, "Lib A"), track(&s, "Lib C"));
    let playlist = s.playlists.first_id().unwrap();
    let (mut h, fake) = open(s);
    select(&mut h, &["Lib C", "Lib A"]);
    menu(&mut h, "Lib A");
    choose(&mut h, &["Add to", "Main"]);
    assert_eq!(
        others(fake.take_sent()),
        vec![Command::InsertTracks {
            playlist,
            index: 3,
            tracks: vec![a, c],
        }],
        "in the table's order, at the playlist's end"
    );
}

#[test]
fn a_right_click_on_an_unselected_row_acts_on_that_row_alone() {
    let s = base(&["Lib A", "Lib B"]);
    let b = track(&s, "Lib B");
    let playlist = s.playlists.first_id().unwrap();
    let (mut h, fake) = open(s);
    select(&mut h, &["Lib A"]);
    menu(&mut h, "Lib B");
    choose(&mut h, &["Add to", "Main"]);
    assert_eq!(
        others(fake.take_sent()),
        vec![Command::InsertTracks {
            playlist,
            index: 3,
            tracks: vec![b],
        }]
    );
}

#[test]
fn add_to_cart_fills_the_chosen_and_next_empty_carts() {
    let mut s = base(&["Lib A", "Lib B", "Lib C"]);
    let page = s.cartwall.pages[0].id;
    // Cart 4 holds a file: it is skipped.
    fp_model::apply(
        &mut s,
        Command::AssignCartFile {
            page,
            index: 3,
            path: PathBuf::from("/carts/id.wav"),
        },
    )
    .unwrap();
    let [a, b, c] = [track(&s, "Lib A"), track(&s, "Lib B"), track(&s, "Lib C")];
    let (mut h, fake) = open(s);
    select(&mut h, &["Lib A", "Lib B", "Lib C"]);
    menu(&mut h, "Lib B");
    h.get_by_label("Add to cart").hover();
    h.run_steps(2);
    assert!(h.query_by_label("Cart 3").is_some());
    assert!(
        h.query_by_label("Cart 4").is_none(),
        "only the empty carts are offered"
    );
    choose(&mut h, &["Add to cart", "Cart 3"]);
    assert_eq!(
        others(fake.take_sent()),
        vec![
            Command::AssignCartTrack {
                page,
                index: 2,
                track: a
            },
            Command::AssignCartTrack {
                page,
                index: 4,
                track: b
            },
            Command::AssignCartTrack {
                page,
                index: 5,
                track: c
            },
        ]
    );
}

#[test]
fn add_to_cart_leaves_out_what_does_not_fit_with_a_notice() {
    let s = base(&["Lib A", "Lib B", "Lib C"]);
    let page = s.cartwall.pages[0].id;
    let last = s.cartwall.pages[0].carts.len();
    let (a, b) = (track(&s, "Lib A"), track(&s, "Lib B"));
    let (mut h, fake) = open(s);
    select(&mut h, &["Lib A", "Lib B", "Lib C"]);
    menu(&mut h, "Lib A");
    let one_before = format!("Cart {}", last - 1);
    choose(&mut h, &["Add to cart", one_before.as_str()]);
    assert_eq!(
        others(fake.take_sent()),
        vec![
            Command::AssignCartTrack {
                page,
                index: last - 2,
                track: a
            },
            Command::AssignCartTrack {
                page,
                index: last - 1,
                track: b
            },
        ]
    );
    assert!(h.query_by_label_contains("1 track did not fit").is_some());
}

#[test]
fn move_to_category_sends_the_selected_tracks() {
    let mut s = base(&["Lib A", "Lib B"]);
    let rock = create(&mut s, "Rock");
    let (a, b) = (track(&s, "Lib A"), track(&s, "Lib B"));
    let (mut h, fake) = open(s);
    select(&mut h, &["Lib A", "Lib B"]);
    menu(&mut h, "Lib B");
    choose(&mut h, &["Move to category", "Rock"]);
    assert_eq!(
        others(fake.take_sent()),
        vec![Command::MoveTracksToCategory {
            tracks: vec![a, b],
            category: Some(rock),
        }]
    );
}

#[test]
fn kind_sends_the_selected_tracks() {
    let s = base(&["Lib A", "Lib B"]);
    let (a, b) = (track(&s, "Lib A"), track(&s, "Lib B"));
    let (mut h, fake) = open(s);
    select(&mut h, &["Lib A", "Lib B"]);
    menu(&mut h, "Lib A");
    // The menu's Kind, not the table's column header.
    let kind = h
        .get_by_role_and_label(Role::Button, "Kind")
        .rect()
        .center();
    click_at(&mut h, kind, Modifiers::NONE, PointerButton::Primary);
    choose(&mut h, &["Jingle"]);
    assert_eq!(
        others(fake.take_sent()),
        vec![Command::SetTracksKind {
            tracks: vec![a, b],
            kind: TrackKind::Jingle,
        }]
    );
}

#[test]
fn reset_markers_sends_one_command_per_track() {
    let s = base(&["Lib A", "Lib B"]);
    let (a, b) = (track(&s, "Lib A"), track(&s, "Lib B"));
    let (mut h, fake) = open(s);
    select(&mut h, &["Lib A", "Lib B"]);
    menu(&mut h, "Lib A");
    choose(&mut h, &["Reset markers to automatic"]);
    assert_eq!(
        others(fake.take_sent()),
        vec![
            Command::ResetMarkers { track: a },
            Command::ResetMarkers { track: b },
        ]
    );
}

#[test]
fn edit_tags_is_offered_for_one_row_only() {
    let s = base(&["Lib A", "Lib B"]);
    let (mut h, _) = open(s);
    select(&mut h, &["Lib A", "Lib B"]);
    menu(&mut h, "Lib A");
    assert!(h.query_by_label("Re-analyse").is_some(), "the menu is open");
    for one_row in ["Edit tags…", "Loudness…", "Show file location"] {
        assert!(h.query_by_label(one_row).is_none(), "{one_row}");
    }
    h.key_press(egui::Key::Escape);
    h.run_steps(2);
    select(&mut h, &["Lib B"]);
    menu(&mut h, "Lib B");
    for one_row in ["Edit tags…", "Loudness…", "Show file location"] {
        assert!(h.query_by_label(one_row).is_some(), "{one_row}");
    }
}

#[test]
fn remove_from_library_asks_twice_when_tracks_are_used() {
    let s = base(&["Lib A"]);
    let song1 = s
        .library
        .track_at(&PathBuf::from("/music/Song 1.mp3"))
        .unwrap();
    let a = track(&s, "Lib A");
    let (mut h, fake) = open(s);
    select(&mut h, &["Lib A", "Song 1"]);
    menu(&mut h, "Song 1");
    choose(&mut h, &["Remove from library…"]);
    assert!(
        h.query_by_label("Remove 2 tracks from the library?")
            .is_some()
    );
    h.get_by_label("Remove").click();
    h.run_steps(3);
    assert!(others(fake.take_sent()).is_empty(), "a second question");
    assert!(
        h.query_by_label_contains("1 playlist entry and 0 carts use these tracks")
            .is_some()
    );
    h.get_by_label("Remove anyway").click();
    h.run_steps(3);
    let sent = others(fake.take_sent());
    assert_eq!(sent.len(), 1, "{sent:?}");
    let Command::RemoveTracks(mut removed) = sent[0].clone() else {
        panic!("{sent:?}");
    };
    removed.sort();
    let mut wanted = vec![a, song1];
    wanted.sort();
    assert_eq!(removed, wanted);
}

#[test]
fn remove_from_library_is_refused_while_on_air() {
    let mut s = base(&[]);
    let first = s.playlists.first_id().unwrap();
    let e0 = s.playlists.get(first).unwrap().entries[0].id;
    s.players[0].current = Some(e0);
    s.players[0].transport = fp_model::Transport::Playing;
    let (mut h, fake) = open(s);
    menu(&mut h, "Song 1");
    choose(&mut h, &["Remove from library…"]);
    assert!(
        h.query_by_label("Remove 1 track from the library?")
            .is_none(),
        "no question"
    );
    assert!(
        h.query_by_label(
            "Some of these tracks are on air: 1 in playlists and 0 on the cartwall. \
             Nothing was removed."
        )
        .is_some()
    );
    assert!(others(fake.take_sent()).is_empty());
}

#[test]
fn show_file_location_opens_the_folder() {
    let s = base(&["Lib A"]);
    let platform = Arc::new(FakePlatform::default());
    let (mut h, _) = open_with(s, Some(platform.clone()));
    menu(&mut h, "Lib A");
    choose(&mut h, &["Show file location"]);
    assert_eq!(
        *platform.asked.lock().unwrap(),
        vec!["open file:///lib/".to_owned()]
    );
}

#[test]
fn dragging_rows_onto_a_playlist_inserts_them_at_the_drop_line() {
    let s = base(&["Lib A", "Lib B", "Lib C"]);
    let (a, b) = (track(&s, "Lib A"), track(&s, "Lib B"));
    let playlist = s.playlists.first_id().unwrap();
    let (mut h, fake) = open(s);
    select(&mut h, &["Lib A", "Lib B"]);
    fake.take_sent();
    let from = in_library(&h, "Lib B").center();
    // The lower part of Song 1's row, right of the library window: the
    // boundary between Song 1 and Song 2.
    let song1 = in_playlist(&h, "Song 1");
    let to = pos2(900.0, song1.center().y + 10.0);
    drag(&mut h, from, to);
    assert_eq!(
        others(fake.take_sent()),
        vec![Command::InsertTracks {
            playlist,
            index: 1,
            tracks: vec![a, b],
        }]
    );
}

#[test]
fn dragging_an_unselected_row_drags_it_alone() {
    let s = base(&["Lib A", "Lib B"]);
    let b = track(&s, "Lib B");
    let playlist = s.playlists.first_id().unwrap();
    let (mut h, fake) = open(s);
    select(&mut h, &["Lib A"]);
    fake.take_sent();
    let from = in_library(&h, "Lib B").center();
    let song1 = in_playlist(&h, "Song 1");
    let to = pos2(900.0, song1.center().y + 10.0);
    drag(&mut h, from, to);
    assert_eq!(
        others(fake.take_sent()),
        vec![Command::InsertTracks {
            playlist,
            index: 1,
            tracks: vec![b],
        }]
    );
}

#[test]
fn dragging_rows_onto_the_library_window_inserts_nothing() {
    let s = base(&["Lib A", "Lib B"]);
    let (mut h, fake) = open(s);
    let from = in_library(&h, "Lib A").center();
    // Over the playlist's rows, but on the library window that covers them.
    let song1 = in_playlist(&h, "Song 1");
    let to = pos2(from.x, song1.center().y + 10.0);
    assert!(library_rect(&h).contains(to));
    drag(&mut h, from, to);
    assert!(others(fake.take_sent()).is_empty());
}

#[test]
fn dragging_rows_onto_a_cart_assigns_the_first() {
    let s = base(&["Lib A", "Lib B"]);
    let page = s.cartwall.pages[0].id;
    let a = track(&s, "Lib A");
    let (mut h, fake) = open(s);
    select(&mut h, &["Lib A", "Lib B"]);
    fake.take_sent();
    let from = in_library(&h, "Lib B").center();
    let to = h.get_by_label("Cart 2, empty").rect().center();
    drag(&mut h, from, to);
    assert_eq!(
        others(fake.take_sent()),
        vec![Command::AssignCartTrack {
            page,
            index: 1,
            track: a
        }]
    );
}

#[test]
fn a_search_drops_hidden_rows_from_the_selection() {
    let s = base(&["Lib A", "Lib B"]);
    let a = track(&s, "Lib A");
    let playlist = s.playlists.first_id().unwrap();
    let (mut h, fake) = open(s);
    select(&mut h, &["Lib A", "Lib B"]);
    h.get_by_label("Search the library").click();
    h.run_steps(1);
    h.get_by_label("Search the library").type_text("lib a");
    h.run_steps(2);
    // Lib B, hidden by the search, comes back unselected.
    for _ in 0.."lib a".len() {
        h.key_press(egui::Key::Backspace);
        h.run_steps(1);
    }
    h.run_steps(2);
    let b_row = in_library(&h, "Lib B").center();
    let b_selected = h
        .query_all_by_role(Role::Row)
        .filter(|n| n.accesskit_node().toggled() == Some(egui::accesskit::Toggled::True))
        .any(|n| n.rect().contains(b_row));
    assert!(!b_selected, "Lib B left the selection");
    menu(&mut h, "Lib A");
    choose(&mut h, &["Add to", "Main"]);
    assert_eq!(
        others(fake.take_sent()),
        vec![Command::InsertTracks {
            playlist,
            index: 3,
            tracks: vec![a],
        }],
        "Lib B left the selection when the search hid it"
    );
}

/// Files dropped on a dialog over the library window do not join the
/// library: the window's layer decides, not its rectangle.
#[test]
fn files_dropped_on_a_dialog_over_the_library_are_not_added() {
    #[derive(Debug)]
    struct Dropped(PathBuf);
    impl egui::DroppedFile for Dropped {
        fn path(&self) -> &std::path::Path {
            &self.0
        }
        fn bytes(&self) -> Result<Vec<u8>, String> {
            std::fs::read(&self.0).map_err(|e| e.to_string())
        }
    }
    let dir = tempfile::tempdir().unwrap();
    let file = fp_app::ui::files::canonical_path(dir.path()).join("new.wav");
    std::fs::write(&file, b"x").unwrap();
    let s = base(&[]);
    let (mut h, fake) = open(s);
    h.get_by_label("New category").click();
    h.run_steps(3);
    assert!(h.query_by_label("Create").is_some(), "the dialog is open");
    let window = library_rect(&h);
    h.hover_at(pos2(window.left() + 40.0, window.bottom() - 40.0));
    h.step();
    h.input_mut()
        .dropped_files
        .push(Arc::new(Dropped(file.clone())));
    for _ in 0..40 {
        h.run_steps(1);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let sent = others(fake.take_sent());
    assert!(
        !sent
            .iter()
            .any(|c| matches!(c, Command::AddToLibrary { .. })),
        "{sent:?}"
    );
}
