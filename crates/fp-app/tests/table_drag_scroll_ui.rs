#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Operator feedback 4: a playlist table scrolls while an entry is dragged
//! over it, by itself near its top and bottom edges and with the wheel.

mod support;

use egui::{Event, Modifiers, PointerButton, Pos2, Rect, pos2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::AppUi;
use fp_app::ui::theme;
use fp_model::Command;
use support::{Fake, harness, state};

const TRACKS: usize = 200;

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
}

fn release(h: &mut Harness<'_, AppUi>, at: Pos2) {
    press(h, at, false);
    h.run_steps(3);
}

fn wheel(h: &mut Harness<'_, AppUi>, dy: f32) {
    h.event(Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        delta: egui::vec2(0.0, dy),
        phase: egui::TouchPhase::Move,
        modifiers: Modifiers::NONE,
    });
}

/// The bottom of the table header: the body begins just below it.
fn header_bottom(h: &Harness<'_, AppUi>) -> f32 {
    h.get_by_label("TITLE").rect().bottom()
}

/// The top of the table footer (its buttons): the body ends just above it.
fn footer_top(h: &Harness<'_, AppUi>) -> f32 {
    h.get_by_label("Add tracks to this playlist").rect().top()
}

/// The `Song n` cells of the table body, by `n`.
fn rows(h: &Harness<'_, AppUi>) -> Vec<(usize, Rect)> {
    let (top, bottom) = (header_bottom(h), footer_top(h));
    (1..=TRACKS)
        .filter_map(|n| {
            h.query_all_by_label(&format!("Song {n}"))
                .map(|node| node.rect())
                .find(|r| r.top() >= top && r.bottom() <= bottom)
                .map(|r| (n, r))
        })
        .collect()
}

/// The first row shown whole below the header.
fn first_row(h: &Harness<'_, AppUi>) -> usize {
    let top = header_bottom(h);
    rows(h)
        .into_iter()
        .find(|(_, r)| r.top() - 7.0 >= top)
        .map(|(n, _)| n)
        .expect("a row is shown")
}

fn cell(h: &Harness<'_, AppUi>, n: usize) -> Rect {
    rows(h)
        .into_iter()
        .find(|(k, _)| *k == n)
        .map(|(_, r)| r)
        .unwrap_or_else(|| panic!("Song {n} is not shown"))
}

/// The violet drop line drawn in the last frame in the table body.
fn bars(h: &Harness<'_, AppUi>) -> Vec<Rect> {
    let top = header_bottom(h);
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

fn moves(fake: &Fake) -> Vec<Command> {
    fake.take_sent()
        .into_iter()
        .filter(|c| matches!(c, Command::MoveEntry { .. }))
        .collect()
}

/// Scrolls the table with the wheel (no drag) until row `k` or later is
/// the first one shown.
fn scroll_down_to(h: &mut Harness<'_, AppUi>, k: usize) {
    let over = cell(h, 2).center();
    h.event(Event::PointerMoved(over));
    wheel(h, -((k as f32) * 28.0 + 10.0));
    h.run_steps(30);
    h.event(Event::PointerMoved(pos2(over.x, 5.0)));
    h.run_steps(2);
    assert!(first_row(h) >= k, "scrolled: {}", first_row(h));
}

#[test]
fn holding_an_entry_near_the_bottom_edge_scrolls_down_and_drops_there() {
    let (mut h, fake) = harness(state(1, TRACKS));
    let (e, playlist) = (
        fake.entries(),
        fake.state.load().playlists.first_id().unwrap(),
    );
    let shown = rows(&h).last().unwrap().0;
    let from = cell(&h, 2).center();
    let near_bottom = pos2(from.x, footer_top(&h) - 12.0);
    begin_drag(&mut h, from, near_bottom);
    h.run_steps(50);
    let first = first_row(&h);
    assert!(
        first > shown,
        "rows that were off-screen are shown: first {first}, before {shown}"
    );
    // Where the line is drawn is where the entry lands: the boundary
    // below the row whose bottom is at the line (the line is drawn 2
    // points higher at the end of the shown part).
    let found = bars(&h);
    assert_eq!(found.len(), 1, "{found:?}");
    let line = found[0].top();
    let (n, _) = rows(&h)
        .into_iter()
        .find(|(_, r)| (r.center().y + 14.0 - line).abs() < 2.5)
        .unwrap_or_else(|| panic!("a row ends at the line {line}: {:?}", rows(&h)));
    fake.take_sent();
    release(&mut h, near_bottom);
    assert_eq!(
        moves(&fake),
        vec![Command::MoveEntry {
            entry: e[1],
            to: playlist,
            index: n
        }]
    );
    // Released: the table stays where it is.
    let after = first_row(&h);
    h.run_steps(20);
    assert_eq!(first_row(&h), after);
}

#[test]
fn holding_an_entry_at_the_bottom_stops_at_the_end_of_the_list() {
    let (mut h, _) = harness(state(1, 40));
    let from = cell(&h, 2).center();
    let near_bottom = pos2(from.x, footer_top(&h) - 4.0);
    begin_drag(&mut h, from, near_bottom);
    h.run_steps(300);
    let last = rows(&h).last().unwrap().0;
    assert_eq!(last, 40, "the last row is shown");
    let first = first_row(&h);
    h.run_steps(20);
    assert_eq!(first_row(&h), first, "and it stops there");
}

#[test]
fn holding_an_entry_near_the_top_edge_scrolls_up() {
    let (mut h, _) = harness(state(1, TRACKS));
    scroll_down_to(&mut h, 60);
    let before = first_row(&h);
    let from = cell(&h, before + 2).center();
    let near_top = pos2(from.x, header_bottom(&h) + 10.0);
    begin_drag(&mut h, from, near_top);
    h.run_steps(50);
    let after = first_row(&h);
    assert!(after + 5 < before, "scrolled up: {before} -> {after}");
}

#[test]
fn holding_an_entry_in_the_middle_does_not_scroll() {
    let (mut h, _) = harness(state(1, TRACKS));
    scroll_down_to(&mut h, 60);
    let before = first_row(&h);
    let from = cell(&h, before + 1).center();
    let middle = pos2(from.x, (header_bottom(&h) + footer_top(&h)) / 2.0);
    begin_drag(&mut h, from, middle);
    h.run_steps(50);
    assert_eq!(first_row(&h), before);
}

#[test]
fn the_wheel_scrolls_the_table_during_a_drag_once() {
    let (mut h, _) = harness(state(1, TRACKS));
    let from = cell(&h, 1).center();
    let middle = pos2(from.x, (header_bottom(&h) + footer_top(&h)) / 2.0);
    begin_drag(&mut h, from, middle);
    assert_eq!(first_row(&h), 1);
    wheel(&mut h, -(10.0 * 28.0));
    h.run_steps(30);
    let first = first_row(&h);
    assert!(
        (10..=12).contains(&first),
        "ten rows down, not twenty: {first}"
    );
}

#[test]
fn dragging_a_header_does_not_scroll_the_table() {
    let (mut h, _) = harness(state(1, TRACKS));
    let from = h.get_by_label("TITLE").rect().center();
    let near_bottom = pos2(from.x, footer_top(&h) - 12.0);
    begin_drag(&mut h, from, near_bottom);
    h.run_steps(50);
    assert_eq!(first_row(&h), 1);
    release(&mut h, near_bottom);
}
