#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::float_cmp
)]
//! Track library spec §7.1 (LB24–LB26): the Library tile in the top bar and
//! the floating library window.

use crate::support;

use egui::{Event, Modifiers, PointerButton, Pos2, Rect, pos2, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::AppUi;
use fp_model::library_view::LibraryView;
use fp_model::{AppState, Command};
use support::{Fake, harness, harness_sized, state};

const OPEN: &str = "Open the library";
const CLOSE_TILE: &str = "Close the library";
const TITLE: &str = "Library";
const MINIMISE: &str = "Minimize library";
const EXPAND: &str = "Expand library";
const MAXIMISE: &str = "Maximize library";
const RESTORE: &str = "Restore library";
const CLOSE: &str = "Close library";
/// The title bar's height.
const TITLE_BAR: f32 = 28.0;

/// One player with three tracks and the library window as `view`.
fn with_view(view: LibraryView) -> AppState {
    let mut s = state(1, 3);
    s.library_view = view;
    s
}

fn open() -> LibraryView {
    LibraryView {
        open: true,
        ..LibraryView::default()
    }
}

/// The library views sent since the last call.
fn views(fake: &Fake) -> Vec<LibraryView> {
    fake.take_sent()
        .into_iter()
        .filter_map(|c| match c {
            Command::SetLibraryView(v) => Some(*v),
            _ => None,
        })
        .collect()
}

fn press(h: &mut Harness<'_, AppUi>, at: Pos2, pressed: bool) {
    h.event(Event::PointerButton {
        pos: at,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    });
}

/// Presses at `from`, moves to `to` in steps and releases there.
fn drag(h: &mut Harness<'_, AppUi>, from: Pos2, to: Pos2) {
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
    press(h, to, false);
    h.run_steps(3);
}

#[test]
fn the_library_tile_opens_and_closes_the_window() {
    let (mut h, fake) = harness(state(1, 3));
    assert!(h.query_by_label(TITLE).is_none());
    h.get_by_label(OPEN).click();
    h.run_steps(2);
    let sent = views(&fake);
    assert_eq!(sent.len(), 1, "{sent:?}");
    assert!(sent[0].open);
    // A window's first frame is a hidden sizing pass.
    h.run_steps(1);
    assert!(
        h.query_by_label(TITLE).is_some(),
        "the window shows its title"
    );
    assert!(h.query_by_label(OPEN).is_none(), "the tile now closes it");
    h.get_by_label(CLOSE_TILE).click();
    h.run_steps(2);
    let sent = views(&fake);
    assert_eq!(sent.len(), 1, "{sent:?}");
    assert!(!sent[0].open);
    assert!(h.query_by_label(TITLE).is_none());
}

#[test]
fn the_tile_is_not_on_a_phone() {
    let (h, _) = harness_sized(with_view(open()), vec2(412.0, 892.0), |ui| ui.with_touch());
    assert!(h.query_by_label(OPEN).is_none());
    assert!(h.query_by_label(CLOSE_TILE).is_none());
    assert!(h.query_by_label(TITLE).is_none(), "no window on a phone");
    assert!(h.query_by_label(CLOSE).is_none());
}

#[test]
fn players_keep_working_while_the_library_is_open() {
    // The window low on the right, clear of the player's transport.
    let view = LibraryView {
        rect: Some([480.0, 300.0, 480.0, 320.0]),
        ..open()
    };
    let (mut h, fake) = harness(with_view(view));
    let window = h.state().library_rect().expect("the window is drawn");
    let play = h.get_by_label("Play").rect();
    assert!(!window.intersects(play), "{window:?} {play:?}");
    h.get_by_label("Play").click();
    h.run_steps(2);
    let p = fake.player(0);
    assert!(fake.take_sent().contains(&Command::Play(p)));
    assert!(h.query_by_label(TITLE).is_some(), "the window stays open");
}

#[test]
fn minimise_keeps_only_the_title_bar() {
    let (mut h, fake) = harness(with_view(open()));
    let before = h.state().library_rect().unwrap();
    h.get_by_label(MINIMISE).click();
    h.run_steps(2);
    let sent = views(&fake);
    assert_eq!(sent.len(), 1, "{sent:?}");
    assert!(sent[0].minimised && sent[0].open);
    let after = h.state().library_rect().unwrap();
    assert_eq!(after.height(), TITLE_BAR, "{after:?}");
    assert_eq!(after.min, before.min);
    assert!(h.query_by_label(TITLE).is_some());
    h.get_by_label(EXPAND).click();
    h.run_steps(2);
    let sent = views(&fake);
    assert!(!sent[0].minimised);
    assert_eq!(h.state().library_rect().unwrap(), before);
}

#[test]
fn maximise_fills_the_area_and_restores() {
    let (mut h, fake) = harness(with_view(open()));
    let before = h.state().library_rect().unwrap();
    h.get_by_label(MAXIMISE).click();
    h.run_steps(2);
    let sent = views(&fake);
    assert_eq!(sent.len(), 1, "{sent:?}");
    assert!(sent[0].maximised);
    // The harness draws in 1000 × 700 less kittest's 8-point margin; the
    // area runs from below the 34-point top bar to above the 24-point
    // status bar.
    let area = Rect::from_min_max(pos2(8.0, 42.0), pos2(992.0, 668.0));
    assert_eq!(h.state().library_rect().unwrap(), area);
    h.get_by_label(RESTORE).click();
    h.run_steps(2);
    let sent = views(&fake);
    assert!(!sent[0].maximised);
    assert_eq!(h.state().library_rect().unwrap(), before);
}

#[test]
fn the_default_placement_is_centred_at_seventy_percent() {
    let (h, _) = harness(with_view(open()));
    let area = Rect::from_min_max(pos2(8.0, 42.0), pos2(992.0, 668.0));
    let window = h.state().library_rect().unwrap();
    assert!(
        (window.width() - area.width() * 0.7).abs() < 0.5,
        "{window:?}"
    );
    assert!(
        (window.height() - area.height() * 0.7).abs() < 0.5,
        "{window:?}"
    );
    assert!(
        (window.center() - area.center()).length() < 0.5,
        "{window:?}"
    );
}

#[test]
fn close_closes_the_window() {
    let (mut h, fake) = harness(with_view(open()));
    h.get_by_label(CLOSE).click();
    h.run_steps(2);
    let sent = views(&fake);
    assert_eq!(sent.len(), 1, "{sent:?}");
    assert!(!sent[0].open);
    assert!(h.query_by_label(TITLE).is_none());
    assert!(h.query_by_label(OPEN).is_some());
}

#[test]
fn a_saved_rect_off_screen_is_drawn_inside_the_window() {
    let view = LibraryView {
        rect: Some([5000.0, 4000.0, 600.0, 400.0]),
        ..open()
    };
    let (h, _) = harness(with_view(view));
    let screen = h.ctx.content_rect();
    let window = h.state().library_rect().unwrap();
    assert!(screen.contains_rect(window), "{window:?} in {screen:?}");
    assert_eq!(window.size(), vec2(600.0, 400.0));
}

#[test]
fn a_rect_smaller_than_the_minimum_is_drawn_at_the_minimum() {
    let view = LibraryView {
        rect: Some([100.0, 100.0, 50.0, 40.0]),
        ..open()
    };
    let (h, _) = harness(with_view(view));
    let window = h.state().library_rect().unwrap();
    assert_eq!(window.size(), vec2(480.0, 320.0));
}

#[test]
fn dragging_the_title_bar_moves_the_window_and_sends_one_view() {
    let view = LibraryView {
        rect: Some([100.0, 100.0, 500.0, 360.0]),
        ..open()
    };
    let (mut h, fake) = harness(with_view(view));
    fake.take_sent();
    let window = h.state().library_rect().unwrap();
    let from = pos2(window.left() + 200.0, window.top() + TITLE_BAR / 2.0);
    drag(&mut h, from, from + vec2(60.0, 40.0));
    let sent = views(&fake);
    assert_eq!(sent.len(), 1, "one view when the drag ends: {sent:?}");
    assert_eq!(sent[0].rect, Some([160.0, 140.0, 500.0, 360.0]));
    assert_eq!(h.state().library_rect().unwrap().min, pos2(160.0, 140.0));
}

#[test]
fn dragging_the_right_edge_resizes_the_window() {
    let view = LibraryView {
        rect: Some([100.0, 100.0, 500.0, 360.0]),
        ..open()
    };
    let (mut h, fake) = harness(with_view(view));
    fake.take_sent();
    let window = h.state().library_rect().unwrap();
    let from = pos2(window.right() - 2.0, window.center().y);
    drag(&mut h, from, from + vec2(80.0, 0.0));
    let sent = views(&fake);
    assert_eq!(sent.len(), 1, "{sent:?}");
    assert_eq!(sent[0].rect, Some([100.0, 100.0, 580.0, 360.0]));
}

#[test]
fn dragging_the_splitter_sends_the_tree_width() {
    let (mut h, fake) = harness(with_view(open()));
    fake.take_sent();
    let window = h.state().library_rect().unwrap();
    // The body starts inside the 1-point border; the tree is 220 points
    // wide and the 6-point splitter follows it.
    let from = pos2(window.left() + 1.0 + 223.0, window.center().y);
    drag(&mut h, from, from + vec2(50.0, 0.0));
    let sent = views(&fake);
    assert_eq!(sent.len(), 1, "{sent:?}");
    assert_eq!(sent[0].tree_width, Some(270.0));
}
