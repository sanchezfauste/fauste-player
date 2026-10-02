#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Feedback 2 spec O7: a table opens scrolled to its next entry.

mod support;

use std::sync::Arc;

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::AppUi;
use fp_model::{AppState, Command, EntryId, PlaylistId, apply};
use support::{harness, state};

const ROW_HEIGHT: f32 = 28.0;

fn entries(s: &AppState) -> Vec<EntryId> {
    s.playlists
        .iter()
        .next()
        .unwrap()
        .entries
        .iter()
        .map(|e| e.id)
        .collect()
}

fn with_next(tracks: usize, next: usize) -> AppState {
    let mut s = state(1, tracks);
    let p = s.players[0].id;
    let e = entries(&s)[next];
    apply(&mut s, Command::SetNext(p, e)).unwrap();
    s
}

/// The table's rows, as the vertical range between its header and the footer.
fn body(h: &Harness<'_, AppUi>) -> (f32, f32) {
    let top = h
        .query_all_by_label("TITLE")
        .map(|n| n.rect().bottom())
        .fold(f32::INFINITY, f32::min);
    let bottom = h
        .query_all_by_label("Add tracks to this playlist")
        .map(|n| n.rect().top())
        .fold(f32::INFINITY, f32::min);
    (top, bottom)
}

/// The row labelled `label` in the table, if it is on screen.
fn row_center(h: &Harness<'_, AppUi>, label: &str) -> Option<f32> {
    let (top, bottom) = body(h);
    h.query_all_by_label(label)
        .map(|n| n.rect())
        .find(|r| r.top() >= top && r.bottom() <= bottom)
        .map(|r| r.center().y)
}

#[test]
fn the_next_entry_is_visible_and_centred() {
    let (mut h, _) = harness(with_next(300, 150));
    h.run_steps(4);
    let (top, bottom) = body(&h);
    let y = row_center(&h, "Song 151").expect("the next row is on screen");
    let middle = (top + bottom) / 2.0;
    assert!(
        (y - middle).abs() <= ROW_HEIGHT,
        "row at {y}, middle of the table at {middle}"
    );
    assert!(row_center(&h, "Song 1").is_none());
}

#[test]
fn near_the_start_the_table_stays_at_the_top() {
    let (mut h, _) = harness(with_next(300, 2));
    h.run_steps(4);
    assert!(row_center(&h, "Song 1").is_some());
    assert!(row_center(&h, "Song 3").is_some());
}

#[test]
fn near_the_end_the_table_scrolls_as_far_as_it_can() {
    let (mut h, _) = harness(with_next(300, 298));
    h.run_steps(4);
    assert!(row_center(&h, "Song 299").is_some());
    assert!(
        row_center(&h, "Song 300").is_some(),
        "the last row is in view, not under the footer"
    );
}

#[test]
fn the_last_row_is_reachable_in_a_short_window() {
    // The table once kept a 200-point minimum height: in a short window
    // the footer covered its last rows, whatever the scroll.
    let (mut h, _) =
        support::harness_sized(with_next(300, 299), egui::vec2(1000.0, 600.0), |ui| ui);
    h.run_steps(4);
    assert!(row_center(&h, "Song 300").is_some());
}

#[test]
fn it_happens_once_a_later_next_does_not_scroll() {
    let (mut h, fake) = harness(with_next(300, 150));
    h.run_steps(4);
    let p = fake.player(0);
    let e = fake.entries();
    let mut s = (*fake.state.load_full()).clone();
    apply(&mut s, Command::SetNext(p, e[10])).unwrap();
    fake.state.store(Arc::new(s));
    h.run_steps(4);
    assert!(row_center(&h, "Song 11").is_none());
    assert!(
        row_center(&h, "Song 151").is_some(),
        "the view did not move"
    );
}

#[test]
fn a_short_list_needs_no_scrolling() {
    let (mut h, _) = harness(with_next(5, 3));
    h.run_steps(4);
    assert!(row_center(&h, "Song 1").is_some());
    assert!(row_center(&h, "Song 5").is_some());
}

#[test]
fn an_empty_list_is_fine() {
    let (mut h, _) = harness(state(1, 0));
    h.run_steps(4);
    assert!(h.query_by_label("TITLE").is_some());
}

#[test]
fn each_player_scrolls_to_its_own_next() {
    let mut s = state(2, 300);
    let e = entries(&s);
    let (p0, p1) = (s.players[0].id, s.players[1].id);
    apply(&mut s, Command::SetNext(p0, e[100])).unwrap();
    apply(&mut s, Command::SetNext(p1, e[250])).unwrap();
    let (mut h, _) = harness(s);
    h.run_steps(4);
    assert!(row_center(&h, "Song 101").is_some());
    assert!(row_center(&h, "Song 251").is_some());
}

#[test]
fn a_next_in_another_playlist_does_not_switch_the_tab_or_scroll() {
    let mut s = state(1, 300);
    let main: PlaylistId = s.playlists.first_id().unwrap();
    apply(
        &mut s,
        Command::CreatePlaylistFromPaths {
            name: "Other".into(),
            paths: (1..=50)
                .map(|n| std::path::PathBuf::from(format!("/music/Other {n}.mp3")))
                .collect(),
        },
    )
    .unwrap();
    let other = s.playlists.iter().nth(1).unwrap().entries[40].id;
    let p = s.players[0].id;
    apply(&mut s, Command::SetNext(p, other)).unwrap();
    let (mut h, fake) = harness(s);
    h.run_steps(4);
    assert!(
        !fake
            .take_sent()
            .iter()
            .any(|c| matches!(c, Command::ShowPlaylist(..)))
    );
    assert_eq!(fake.state.load().player(p).unwrap().playlist, main);
    assert!(
        row_center(&h, "Song 1").is_some(),
        "the table stays at the top"
    );
}
