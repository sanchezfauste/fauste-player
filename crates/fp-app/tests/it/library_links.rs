#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Track library spec §7.9, §9: the playlist menu reaches the library
//! (Show in library, Move to category), and Settings → Library manages the
//! categories.

use crate::support;

use egui::accesskit::Role;
use egui::{Event, Modifiers, PointerButton, Pos2, Rect, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::AppUi;
use fp_model::library_view::LibrarySelection;
use fp_model::{AppState, CategoryId, Command, TrackId, TrackKind};
use std::path::PathBuf;
use std::sync::Arc;
use support::{Fake, harness_sized, state};

type H = Harness<'static, AppUi>;

fn song(s: &AppState, n: usize) -> TrackId {
    s.library
        .track_at(&PathBuf::from(format!("/music/Song {n}.mp3")))
        .unwrap()
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

fn open(s: AppState) -> (H, Arc<Fake>) {
    let (mut h, fake) = harness_sized(s, vec2(1200.0, 800.0), |ui| ui);
    h.run_steps(2);
    fake.take_sent();
    (h, fake)
}

fn click_at(h: &mut H, at: Pos2, button: PointerButton) {
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    for pressed in [true, false] {
        h.event(Event::PointerButton {
            pos: at,
            button,
            pressed,
            modifiers: Modifiers::NONE,
        });
    }
    h.run_steps(2);
}

/// The lowest node labelled `label` (the playlist's row, below the decks).
fn lowest(h: &H, label: &str) -> Rect {
    h.query_all_by_label(label)
        .map(|n| n.rect())
        .max_by(|a, b| a.top().total_cmp(&b.top()))
        .unwrap_or_else(|| panic!("nothing labelled {label}"))
}

/// The menu entry labelled `label`: the one furthest right.
fn item(h: &H, label: &str) -> Rect {
    h.query_all_by_label(label)
        .map(|n| n.rect())
        .max_by(|a, b| a.left().total_cmp(&b.left()))
        .unwrap_or_else(|| panic!("no menu entry {label}"))
}

fn playlist_menu(h: &mut H, name: &str) {
    let at = lowest(h, name).center();
    click_at(h, at, PointerButton::Secondary);
}

fn others(sent: Vec<Command>) -> Vec<Command> {
    sent.into_iter()
        .filter(|c| !matches!(c, Command::SetLibraryView(_)))
        .collect()
}

#[test]
fn show_in_library_opens_the_window_on_the_tracks_category() {
    let mut s = state(1, 3);
    let jingles = create(&mut s, "Jingles");
    let song = song(&s, 2);
    fp_model::apply(
        &mut s,
        Command::MoveTracksToCategory {
            tracks: vec![song],
            category: Some(jingles),
        },
    )
    .unwrap();
    let (mut h, fake) = open(s);
    playlist_menu(&mut h, "Song 2");
    let at = item(&h, "Show in library").center();
    click_at(&mut h, at, PointerButton::Primary);
    let views: Vec<_> = fake
        .take_sent()
        .into_iter()
        .filter_map(|c| match c {
            Command::SetLibraryView(v) => Some(v),
            _ => None,
        })
        .collect();
    let last = views.last().expect("a view was sent");
    assert!(last.open, "the window opens");
    assert_eq!(last.selection, LibrarySelection::Category(jingles));
}

#[test]
fn show_in_library_selects_uncategorised_for_a_track_without_category() {
    let (mut h, fake) = open(state(1, 3));
    playlist_menu(&mut h, "Song 1");
    let at = item(&h, "Show in library").center();
    click_at(&mut h, at, PointerButton::Primary);
    let last = fake
        .take_sent()
        .into_iter()
        .rev()
        .find_map(|c| match c {
            Command::SetLibraryView(v) => Some(v),
            _ => None,
        })
        .expect("a view was sent");
    assert_eq!(last.selection, LibrarySelection::Uncategorised);
}

#[test]
fn move_to_category_from_a_playlist_sends_the_entrys_track() {
    let mut s = state(1, 3);
    let jingles = create(&mut s, "Jingles");
    let song = song(&s, 3);
    let (mut h, fake) = open(s);
    playlist_menu(&mut h, "Song 3");
    let at = item(&h, "Move to category").center();
    click_at(&mut h, at, PointerButton::Primary);
    let at = item(&h, "Jingles").center();
    click_at(&mut h, at, PointerButton::Primary);
    assert_eq!(
        others(fake.take_sent()),
        vec![Command::MoveTracksToCategory {
            tracks: vec![song],
            category: Some(jingles),
        }]
    );
}

#[test]
fn settings_library_lists_the_categories_and_creates_one() {
    let mut s = state(1, 1);
    create(&mut s, "Jingles");
    let (mut h, fake) = open(s);
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Library").click();
    h.run_steps(3);
    assert!(h.query_by_label("Jingles").is_some(), "the tree lists it");
    h.get_by_label("New category").click();
    h.run_steps(3);
    // The name dialog opens over Settings.
    assert!(h.query_all_by_label("Name").next().is_some());
    h.get_by_label("Name").type_text("Ads");
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Create").click();
    h.run_steps(3);
    assert!(others(fake.take_sent()).iter().any(|c| matches!(
        c,
        Command::CreateCategory { name, parent: None, .. } if name == "Ads"
    )));
}

/// The labels of the buttons in the open playlist menu, top to bottom.
fn menu_order(h: &H) -> Vec<String> {
    let mut found: Vec<(f32, String)> = Vec::new();
    for label in ["Loudness…", "Show in library", "Move to category"] {
        for n in h.query_all_by_label(label) {
            found.push((n.rect().top(), label.to_owned()));
        }
    }
    found.sort_by(|a, b| a.0.total_cmp(&b.0));
    found.into_iter().map(|(_, l)| l).collect()
}

#[test]
fn the_library_entries_come_right_after_loudness_on_the_desktop() {
    let (mut h, _fake) = open(state(1, 3));
    playlist_menu(&mut h, "Song 2");
    assert_eq!(
        menu_order(&h),
        vec!["Loudness…", "Show in library", "Move to category"]
    );
    // Nothing between Loudness… and the entries.
    let loud = item(&h, "Loudness…");
    let show = item(&h, "Show in library");
    let mv = item(&h, "Move to category");
    assert!(loud.bottom() <= show.top() + 1.0 && show.bottom() <= mv.top() + 1.0);
    assert!(show.top() - loud.bottom() < 12.0, "right after Loudness…");
}
