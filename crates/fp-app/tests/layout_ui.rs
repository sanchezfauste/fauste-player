#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Resizable layout spec RL7, RL8, RL11, RL12 and RL15–RL18: the main
//! window's panels, driven headlessly. The harness's window has an 8-point
//! margin, so the tests compare rectangles with each other.

mod support;

use egui::{Rect, pos2, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::AppUi;
use fp_app::ui::controller::Controller;
use fp_app::ui::panel_layout::{MIN_PLAYER_HEIGHT, MIN_PLAYER_WIDTH, PANEL_GAP};
use fp_model::layout::Panel;
use fp_model::{Command, PlayerId};
use support::{harness, harness_sized, state};

/// The shown players' columns in the last frame, in display order.
fn columns(h: &Harness<'_, AppUi>) -> Vec<(PlayerId, Rect)> {
    h.state().player_rects()
}

fn column(h: &Harness<'_, AppUi>, player: PlayerId) -> Rect {
    columns(h)
        .into_iter()
        .find(|(p, _)| *p == player)
        .map(|(_, r)| r)
        .expect("the player is shown")
}

fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 1.0
}

/// Whether the track table in `col` shows `title`: a node with that label
/// below the column's top block (the info row may show a title too).
fn table_shows(h: &Harness<'_, AppUi>, col: Rect, title: &str) -> bool {
    h.get_all_by_label(title).any(|n| {
        let r = n.rect();
        col.contains(r.center()) && r.center().y > col.top() + 300.0
    })
}

#[test]
fn rl11_the_default_layout_draws_todays_rectangles() {
    let (h, fake) = harness(state(2, 3));
    let (p1, p2) = (column(&h, fake.player(0)), column(&h, fake.player(1)));
    let cart = h.state().cartwall_rect().unwrap();
    // Two equal columns 8 points apart, as wide together as the cartwall,
    // which keeps its content height (two rows) under them.
    assert!(near(p1.width(), p2.width()), "{p1:?} {p2:?}");
    assert!(near(p2.left(), p1.right() + PANEL_GAP), "{p1:?} {p2:?}");
    assert!(near(p1.left(), cart.left()) && near(p2.right(), cart.right()));
    assert!(near(cart.height(), 116.0), "{cart:?}");
    // The row ends 8 points above the cartwall; a column is 2 points
    // shorter than its row.
    assert!(
        near(p1.bottom() + 2.0 + PANEL_GAP, cart.top()),
        "{p1:?} {cart:?}"
    );
}

#[test]
fn rl11_the_columns_follow_the_shares() {
    let mut s = state(2, 3);
    let p = s.player_ids();
    s.layout.player_shares = vec![(p[0], 3.0), (p[1], 1.0)];
    let (h, _) = harness(s);
    let (a, b) = (column(&h, p[0]), column(&h, p[1]));
    let spare = a.width() + b.width() - 2.0 * MIN_PLAYER_WIDTH;
    assert!(
        near(a.width(), MIN_PLAYER_WIDTH + spare * 0.75),
        "{a:?} {b:?}"
    );
}

#[test]
fn rl11_a_hidden_player_leaves_no_column() {
    let mut s = state(3, 3);
    let p = s.player_ids();
    s.layout.hidden_players = vec![p[1]];
    let (h, _) = harness_sized(s, vec2(1400.0, 700.0), |ui| ui);
    let shown: Vec<PlayerId> = columns(&h).iter().map(|(id, _)| *id).collect();
    assert_eq!(shown, vec![p[0], p[2]]);
}

#[test]
fn rl11_a_hidden_cartwall_gives_its_height_to_the_players() {
    let (h, fake) = harness(state(1, 3));
    let with_cartwall = column(&h, fake.player(0)).height();
    let mut s = state(1, 3);
    s.layout.cartwall.hidden = true;
    let (h, fake) = harness(s);
    assert_eq!(h.state().cartwall_rect(), None);
    let without = column(&h, fake.player(0)).height();
    assert!(
        near(without, with_cartwall + 116.0 + PANEL_GAP),
        "{with_cartwall} -> {without}"
    );
}

#[test]
fn rl11_with_every_player_hidden_the_cartwall_fills_the_middle_area() {
    let mut s = state(2, 3);
    s.layout.hidden_players = s.player_ids();
    let (h, _) = harness(s);
    assert!(columns(&h).is_empty());
    let cart = h.state().cartwall_rect().unwrap();
    assert!(cart.height() > 500.0, "{cart:?}");
}

#[test]
fn rl11_too_narrow_a_window_scrolls_the_players_at_their_minimum() {
    let (h, _) = harness(state(4, 3));
    let c = columns(&h);
    assert_eq!(c.len(), 4);
    assert!(
        c.iter().all(|(_, r)| near(r.width(), MIN_PLAYER_WIDTH)),
        "{c:?}"
    );
}

#[test]
fn rl11_a_column_keeps_its_id_when_a_player_before_it_is_hidden() {
    let (mut h, fake) = harness_sized(state(3, 40), vec2(1400.0, 700.0), |ui| ui);
    let (p2, p3) = (fake.player(1), fake.player(2));
    let col = column(&h, p3);
    assert!(table_shows(&h, col, "Song 1"));
    h.hover_at(pos2(col.center().x, col.bottom() - 60.0));
    h.step();
    h.event(egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Line,
        delta: vec2(0.0, -10.0),
        phase: egui::TouchPhase::Move,
        modifiers: egui::Modifiers::NONE,
    });
    h.run_steps(20);
    assert!(!table_shows(&h, column(&h, p3), "Song 1"), "P3 scrolled");
    Controller::send(
        fake.as_ref(),
        Command::SetPanelHidden {
            panel: Panel::Player(p2),
            hidden: true,
        },
    );
    h.run_steps(3);
    // P3 is now second among the shown players; keyed by its id, its table
    // keeps its own scroll instead of taking P2's.
    assert!(!table_shows(&h, column(&h, p3), "Song 1"));
}

#[test]
fn rl8_three_rows_fit_a_player_at_its_least_height() {
    let mut s = state(1, 3);
    s.layout.cartwall.hidden = true;
    // The harness's margin, the top and status bars, the middle area's margin.
    let window = 2.0 * 8.0 + 34.0 + 24.0 + 2.0 * 8.0 + MIN_PLAYER_HEIGHT;
    let (h, fake) = harness_sized(s, vec2(1000.0, window), |ui| ui);
    let col = column(&h, fake.player(0));
    assert!(near(col.height(), MIN_PLAYER_HEIGHT - 2.0), "{col:?}");
    let third = h
        .get_all_by_label("Song 3")
        .map(|n| n.rect())
        .find(|r| r.center().y > col.top() + 300.0)
        .expect("the third row is built");
    // The footer is the column's last 24 points.
    assert!(
        third.bottom() <= col.bottom() - 24.0 + 0.5,
        "{third:?} in {col:?}"
    );
}
