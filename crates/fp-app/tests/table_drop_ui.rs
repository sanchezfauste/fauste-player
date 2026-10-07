#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Operator feedback 4, Q4: the violet drop line.

mod support;

use egui::{Event, Modifiers, PointerButton, Pos2, Rect, pos2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::AppUi;
use fp_app::ui::theme;
use fp_model::{Command, PlayerId};
use support::{Fake, harness, state};

fn press(h: &mut Harness<'_, AppUi>, at: Pos2, pressed: bool) {
    h.event(Event::PointerButton {
        pos: at,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    });
}

/// Presses at `from` and moves to `to` in steps, button still down.
fn begin_drag(h: &mut Harness<'_, AppUi>, from: Pos2, to: Pos2) {
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
}

fn release(h: &mut Harness<'_, AppUi>, at: Pos2) {
    press(h, at, false);
    h.run_steps(3);
}

/// The bottom of the table headers: what is drawn below it is a table body.
fn header_bottom(h: &Harness<'_, AppUi>) -> f32 {
    h.query_all_by_label("TITLE")
        .map(|n| n.rect().bottom())
        .fold(f32::INFINITY, f32::min)
}

/// The violet drop lines drawn in the last frame in the table bodies: wide,
/// 2 points high (the shown tab's top line is the same colour, above).
fn bars(h: &Harness<'_, AppUi>) -> Vec<Rect> {
    let top = header_bottom(h) - 1.0;
    h.output()
        .shapes
        .iter()
        .filter_map(|s| match &s.shape {
            egui::Shape::Rect(r)
                if r.fill == theme::ACCENT
                    && r.rect.width() > 100.0
                    && r.rect.height() < 3.0
                    && r.rect.top() >= top =>
            {
                Some(r.rect)
            }
            _ => None,
        })
        .collect()
}

/// The table cells labelled `label`, left to right (the deck above also
/// names the next track).
fn cells(h: &Harness<'_, AppUi>, label: &str) -> Vec<Rect> {
    let top = header_bottom(h);
    let mut found: Vec<Rect> = h
        .query_all_by_label(label)
        .map(|n| n.rect())
        .filter(|r| r.top() >= top)
        .collect();
    found.sort_by(|a, b| a.left().total_cmp(&b.left()));
    found
}

fn cell(h: &Harness<'_, AppUi>, label: &str) -> Rect {
    let found = cells(h, label);
    assert_eq!(found.len(), 1, "{label}: {found:?}");
    found[0]
}

fn moves(fake: &Fake) -> Vec<Command> {
    fake.take_sent()
        .into_iter()
        .filter(|c| matches!(c, Command::MoveEntry { .. }))
        .collect()
}

#[test]
fn the_bar_is_at_the_boundary_under_the_pointer() {
    let (mut h, _) = harness(state(1, 5));
    let from = cell(&h, "Song 1").center();
    let row3 = cell(&h, "Song 3").center();
    // The lower part of row 3 is the boundary between rows 3 and 4.
    let to = pos2(row3.x, row3.y + 10.0);
    begin_drag(&mut h, from, to);
    let found = bars(&h);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        (found[0].top() - (row3.y + 14.0)).abs() < 1.5,
        "{found:?} {row3:?}"
    );
}

#[test]
fn releasing_inserts_at_that_boundary_in_the_playlist() {
    let (mut h, fake) = harness(state(1, 5));
    let (e, playlist) = (
        fake.entries(),
        fake.state.load().playlists.first_id().unwrap(),
    );
    let from = cell(&h, "Song 1").center();
    let row3 = cell(&h, "Song 3").center();
    let to = pos2(row3.x, row3.y + 10.0);
    begin_drag(&mut h, from, to);
    fake.take_sent();
    release(&mut h, to);
    assert_eq!(
        moves(&fake),
        vec![Command::MoveEntry {
            entry: e[0],
            to: playlist,
            index: 3
        }]
    );
    assert!(bars(&h).is_empty(), "no bar is left after the drop");
}

#[test]
fn only_the_table_under_the_pointer_draws_the_bar() {
    let mut s = state(2, 5);
    let playlist = s.playlists.first_id().unwrap();
    let p2: PlayerId = s.players[1].id;
    fp_model::apply(&mut s, Command::ShowPlaylist(p2, playlist)).unwrap();
    let (mut h, _) = harness(s);
    let songs = cells(&h, "Song 1");
    let (left, right) = (songs[0], songs[1]);
    let to = pos2(left.center().x, left.center().y + 3.0 * 28.0);
    begin_drag(&mut h, left.center(), to);
    let found = bars(&h);
    assert_eq!(
        found.len(),
        1,
        "the other player shows the same playlist: {found:?}"
    );
    assert!(
        found[0].right() <= right.left() + 1.0,
        "{found:?} {right:?}"
    );
}

#[test]
fn releasing_over_the_header_drops_nothing() {
    let (mut h, fake) = harness(state(1, 5));
    let from = cell(&h, "Song 2").center();
    let header = h.get_by_label("TITLE").rect().center();
    begin_drag(&mut h, from, header);
    assert!(bars(&h).is_empty(), "{:?}", bars(&h));
    fake.take_sent();
    release(&mut h, header);
    assert!(moves(&fake).is_empty());
}

#[test]
fn releasing_on_a_column_edge_drops_nothing() {
    let (mut h, fake) = harness(state(1, 5));
    let from = cell(&h, "Song 2").center();
    // The Title column ends where the Artist header's cell begins.
    let artist = h.get_by_label("ARTIST").rect();
    let on_edge = pos2(artist.left() - 8.0, from.y + 28.0);
    begin_drag(&mut h, from, on_edge);
    assert!(bars(&h).is_empty(), "{:?}", bars(&h));
    fake.take_sent();
    release(&mut h, on_edge);
    assert!(moves(&fake).is_empty());
}

#[test]
fn an_empty_playlist_accepts_a_drop_at_zero() {
    // Player 1 shows the full list, player 2 an empty one.
    let mut s = state(2, 2);
    fp_model::apply(
        &mut s,
        Command::CreatePlaylist {
            name: "Other".into(),
        },
    )
    .unwrap();
    let first = s.playlists.first_id().unwrap();
    let other = s.playlists.iter().find(|p| p.id != first).unwrap().id;
    let p2: PlayerId = s.players[1].id;
    fp_model::apply(&mut s, Command::ShowPlaylist(p2, other)).unwrap();
    let (mut h, fake) = harness(s);
    let e = fake.entries();
    let from = cell(&h, "Song 1").center();
    let mut headers: Vec<Rect> = h.query_all_by_label("TITLE").map(|n| n.rect()).collect();
    headers.sort_by(|a, b| a.left().total_cmp(&b.left()));
    let empty = headers[1];
    let to = pos2(empty.left() + 40.0, empty.bottom() + 60.0);
    begin_drag(&mut h, from, to);
    let found = bars(&h);
    assert_eq!(found.len(), 1, "{found:?}");
    // The header row is 24 points high, its label centred in it: the body,
    // and its first boundary, begin 12 points below the label's centre.
    let body_top = empty.center().y + 12.0;
    assert!(
        (found[0].top() - body_top).abs() < 1.5,
        "{found:?} {empty:?}"
    );
    fake.take_sent();
    release(&mut h, to);
    assert_eq!(
        moves(&fake),
        vec![Command::MoveEntry {
            entry: e[0],
            to: other,
            index: 0
        }]
    );
}

#[test]
fn the_bar_follows_the_pointer_in_a_scrolled_long_list() {
    let (mut h, fake) = harness(state(1, 200));
    let (e, playlist) = (
        fake.entries(),
        fake.state.load().playlists.first_id().unwrap(),
    );
    // egui does not scroll an area while something is dragged, so the
    // table is scrolled first, by a distance that is not a whole row.
    let over = cell(&h, "Song 2").center();
    h.event(Event::PointerMoved(over));
    h.event(Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        delta: egui::vec2(0.0, -(40.0 * 28.0 + 10.0)),
        phase: egui::TouchPhase::Move,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(30);
    // Away from the rows, so that no track popup names a song too.
    h.event(Event::PointerMoved(pos2(over.x, 5.0)));
    h.run_steps(2);
    let top = header_bottom(&h);
    // The first row shown whole.
    let k = (1..=200)
        .find(|n| {
            cells(&h, &format!("Song {n}"))
                .first()
                .is_some_and(|r| r.top() - 7.0 >= top)
        })
        .unwrap();
    assert!(k > 30, "the table scrolled: first row shown {k}");
    let from = cell(&h, &format!("Song {k}")).center();
    // The lower half of the row two below: the boundary after it.
    let to = pos2(from.x, from.y + 2.0 * 28.0 + 10.0);
    begin_drag(&mut h, from, to);
    let found = bars(&h);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        (found[0].top() - (from.y + 2.0 * 28.0 + 14.0)).abs() < 1.5,
        "{found:?} {from:?}"
    );
    fake.take_sent();
    release(&mut h, to);
    // Song k is entry k - 1; the boundary is after Song k + 2.
    assert_eq!(
        moves(&fake),
        vec![Command::MoveEntry {
            entry: e[k - 1],
            to: playlist,
            index: k + 2
        }]
    );
}

#[test]
fn releasing_over_the_scroll_bar_drops_nothing() {
    let (mut h, fake) = harness(state(1, 200));
    let from = cell(&h, "Song 2").center();
    // The last column ends 8 points right of its header's label; the
    // scroll bar is beyond it.
    let last = h.get_by_label("DUR.").rect();
    let on_bar = pos2(last.right() + 8.0 + 6.0, from.y + 28.0);
    begin_drag(&mut h, from, on_bar);
    assert!(bars(&h).is_empty(), "{:?}", bars(&h));
    fake.take_sent();
    release(&mut h, on_bar);
    assert!(moves(&fake).is_empty());
}
