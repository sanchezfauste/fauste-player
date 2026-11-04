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

/// The commands sent, without the library views and the selected row
/// reported to the preview bar.
fn others(sent: Vec<Command>) -> Vec<Command> {
    sent.into_iter()
        .filter(|c| !matches!(c, Command::SetLibraryView(_) | Command::SelectLibraryCue(_)))
        .collect()
}

fn library_rect(h: &H) -> Rect {
    h.state()
        .library_rect()
        .expect("the library window is open")
}

/// The cell labelled `label` in the library window's table (right of its
/// category tree, under the title bar and the preview bar, which names the
/// selected track; the decks under the window name tracks too).
fn in_library(h: &H, label: &str) -> Rect {
    let window = library_rect(h);
    let table = Rect::from_min_max(
        pos2(window.left() + 240.0, window.top() + 28.0 + 112.0),
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

/// A library whose cart 1 holds Song 1 (playing or cueing as asked).
fn with_cart_one(playing: bool, cueing: bool) -> (AppState, TrackId) {
    let mut s = base(&["Lib A"]);
    let page = s.cartwall.pages[0].id;
    let song1 = s
        .library
        .track_at(&PathBuf::from("/music/Song 1.mp3"))
        .unwrap();
    fp_model::apply(
        &mut s,
        Command::AssignCartTrack {
            page,
            index: 0,
            track: song1,
        },
    )
    .unwrap();
    let id = s.cartwall.pages[0].carts[0].id;
    if playing {
        fp_model::apply(&mut s, Command::FireCart(id)).unwrap();
        assert!(s.cartwall.is_playing(id));
    }
    if cueing {
        s.cartwall.cue = Some(id);
    }
    let a = track(&s, "Lib A");
    (s, a)
}

/// Presses at `from` and moves to `to`, the button still down.
fn hold_over(h: &mut H, from: Pos2, to: Pos2) {
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
}

fn drop_on_cart_one(s: AppState) -> (usize, Vec<Command>) {
    let (mut h, fake) = open(s);
    let from = in_library(&h, "Lib A").center();
    // The cart's name, below the library window (the decks name it too).
    let cart = h
        .query_all_by_label("Song 1")
        .map(|n| n.rect())
        .find(|r| r.top() > library_rect(&h).bottom() + 150.0)
        .expect("cart 1 names its track");
    hold_over(&mut h, from, cart.center());
    let outlines = h
        .output()
        .shapes
        .iter()
        .filter(|s| {
            matches!(&s.shape, egui::Shape::Rect(r)
                if r.stroke.color == fp_app::ui::theme::ACCENT
                    && r.stroke.width == 2.0
                    && r.rect.contains(cart.center()))
        })
        .count();
    press(&mut h, cart.center(), false);
    h.run_steps(3);
    (outlines, others(fake.take_sent()))
}

#[test]
fn dragging_rows_onto_an_idle_cart_replaces_its_file() {
    let (s, a) = with_cart_one(false, false);
    let page = s.cartwall.pages[0].id;
    let (outlines, sent) = drop_on_cart_one(s);
    assert_eq!(outlines, 1, "the cart is outlined as a target");
    assert_eq!(
        sent,
        vec![Command::AssignCartTrack {
            page,
            index: 0,
            track: a
        }]
    );
}

#[test]
fn dragging_rows_onto_a_playing_cart_does_nothing() {
    let (s, _) = with_cart_one(true, false);
    let (outlines, sent) = drop_on_cart_one(s);
    assert_eq!(outlines, 0, "a playing cart is not a target");
    assert!(sent.is_empty(), "{sent:?}");
}

#[test]
fn dragging_rows_onto_a_cueing_cart_does_nothing() {
    let (s, _) = with_cart_one(false, true);
    let (outlines, sent) = drop_on_cart_one(s);
    assert_eq!(outlines, 0, "a cart on CUE is not a target");
    assert!(sent.is_empty(), "{sent:?}");
}

#[test]
fn dragging_rows_onto_a_playlist_tab_appends_them() {
    let mut s = base(&["Lib A", "Lib B"]);
    fp_model::apply(
        &mut s,
        Command::CreatePlaylist {
            name: "Second".into(),
        },
    )
    .unwrap();
    let second = s.playlists.iter().find(|p| p.name == "Second").unwrap().id;
    let a = track(&s, "Lib A");
    let (mut h, fake) = open(s);
    let from = in_library(&h, "Lib A").center();
    let tab = in_playlist(&h, "Second");
    assert!(!library_rect(&h).contains(tab.center()), "{tab:?}");
    drag(&mut h, from, tab.center());
    assert_eq!(
        others(fake.take_sent()),
        vec![Command::InsertTracks {
            playlist: second,
            index: 0,
            tracks: vec![a],
        }]
    );
}

/// A finger held still at `at` for a second, then lifted.
fn long_press(h: &mut H, at: Pos2) {
    let touch = |phase| Event::Touch {
        device_id: egui::TouchDeviceId(0),
        id: egui::TouchId(0),
        phase,
        pos: at,
        force: None,
    };
    h.event(touch(egui::TouchPhase::Start));
    h.event(Event::PointerMoved(at));
    press(h, at, true);
    for _ in 0..60 {
        h.step();
    }
    h.event(touch(egui::TouchPhase::End));
    press(h, at, false);
    h.run_steps(3);
}

/// Whether the library row through `c` is selected.
fn row_selected(h: &H, c: Pos2) -> bool {
    h.query_all_by_role(Role::Row)
        .filter(|n| n.accesskit_node().toggled() == Some(egui::accesskit::Toggled::True))
        .any(|n| n.rect().contains(c))
}

#[test]
fn a_long_press_on_an_unselected_row_selects_it_alone() {
    let s = base(&["Lib A", "Lib B"]);
    let (mut h, _) = harness_sized(s, vec2(1200.0, 800.0), |ui| ui.with_touch());
    h.run_steps(2);
    let a = in_library(&h, "Lib A").center();
    click_at(&mut h, a, Modifiers::NONE, PointerButton::Primary);
    assert!(row_selected(&h, a));
    let b = in_library(&h, "Lib B").center();
    long_press(&mut h, b);
    assert!(h.query_by_label("Re-analyse").is_some(), "the menu is open");
    assert!(row_selected(&h, b), "the row is selected");
    assert!(!row_selected(&h, a), "alone");
}
