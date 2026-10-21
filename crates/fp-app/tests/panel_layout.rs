#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Resizable layout spec RL8, RL11 and RL16: the pure pixel pass, one test
//! per rule, named after it.

use egui::{Rect, pos2, vec2};
use fp_app::ui::panel_layout::{
    Arrangement, CONTENT_HEIGHT_CAP, CartwallSizes, MIN_PLAYER_HEIGHT, MIN_PLAYER_WIDTH, PANEL_GAP,
    PassInput, arrange,
};
use fp_model::layout::Panel;
use fp_model::{AppState, Config, PlayerId};

/// The middle area of a 1920 × 1080 window, inside its 8-point margin:
/// under the 34-point top bar, over the 24-point status bar.
fn full_hd() -> Rect {
    Rect::from_min_max(pos2(8.0, 42.0), pos2(1912.0, 1048.0))
}

/// A page of two rows: today's content height (24 + 6 + 2 × 40 + 6) and
/// the least height (the rows at 28).
const SIZES: CartwallSizes = CartwallSizes {
    content_height: 116.0,
    min_height: 92.0,
};

/// The default state: four players.
fn four() -> AppState {
    AppState::new(Config::default(), "Main")
}

fn pass_with(
    s: &AppState,
    area: Rect,
    sizes: CartwallSizes,
    maximised: Option<Panel>,
) -> Arrangement {
    let players = s.player_ids();
    arrange(&PassInput {
        players: &players,
        layout: &s.layout,
        area,
        cartwall: sizes,
        maximised,
    })
}

fn pass(s: &AppState, area: Rect, maximised: Option<Panel>) -> Arrangement {
    pass_with(s, area, SIZES, maximised)
}

fn widths(a: &Arrangement) -> Vec<f32> {
    a.widths.iter().map(|(_, w)| *w).collect()
}

fn ids(a: &Arrangement) -> Vec<PlayerId> {
    a.widths.iter().map(|(p, _)| *p).collect()
}

fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.01
}

fn all_near(got: &[f32], want: &[f32]) -> bool {
    got.len() == want.len() && got.iter().zip(want).all(|(a, b)| near(*a, *b))
}

#[test]
fn rl11_the_default_layout_draws_todays_window() {
    let s = four();
    let a = pass(&s, full_hd(), None);
    // (1904 − 3 × 8) / 4, as the old equal split.
    assert!(all_near(&widths(&a), &[470.0; 4]), "{a:?}");
    assert_eq!(ids(&a), s.player_ids());
    assert!(!a.overflow);
    // The screenshot script crops the cartwall from y 932.
    assert_eq!(
        a.cartwall,
        Some(Rect::from_min_max(pos2(8.0, 932.0), pos2(1912.0, 1048.0)))
    );
    assert_eq!(
        a.players_area,
        Rect::from_min_max(pos2(8.0, 42.0), pos2(1912.0, 924.0))
    );
}

#[test]
fn rl11_the_spare_room_is_split_by_share() {
    let mut s = four();
    let p = s.player_ids();
    s.layout.player_shares = vec![(p[0], 3.0), (p[1], 1.0), (p[2], 1.0), (p[3], 1.0)];
    let a = pass(&s, full_hd(), None);
    // 1904 − 3 gaps − 4 × 380 = 360 spare: 3/6 of it, then 1/6 each.
    assert!(
        all_near(&widths(&a), &[560.0, 440.0, 440.0, 440.0]),
        "{a:?}"
    );
}

#[test]
fn rl11_hidden_players_leave_their_room_to_the_others() {
    let mut s = four();
    let p = s.player_ids();
    s.layout.hidden_players = vec![p[1]];
    let a = pass(&s, full_hd(), None);
    assert_eq!(ids(&a), vec![p[0], p[2], p[3]]);
    // (1904 − 2 × 8) / 3
    assert!(all_near(&widths(&a), &[1888.0 / 3.0; 3]), "{a:?}");
}

#[test]
fn rl11_only_a_window_narrower_than_the_minimums_scrolls() {
    let s = four();
    let narrow = Rect::from_min_size(pos2(8.0, 42.0), vec2(968.0, 610.0));
    let a = pass(&s, narrow, None);
    assert!(a.overflow);
    assert!(all_near(&widths(&a), &[MIN_PLAYER_WIDTH; 4]));
    let exact = Rect::from_min_size(
        pos2(8.0, 42.0),
        vec2(4.0 * MIN_PLAYER_WIDTH + 3.0 * PANEL_GAP, 610.0),
    );
    let a = pass(&s, exact, None);
    assert!(!a.overflow, "the minimums fit exactly");
    assert!(all_near(&widths(&a), &[MIN_PLAYER_WIDTH; 4]));
}

#[test]
fn rl11_a_hidden_cartwall_leaves_the_players_the_middle_area() {
    let mut s = four();
    s.layout.cartwall.hidden = true;
    let a = pass(&s, full_hd(), None);
    assert_eq!(a.cartwall, None);
    assert_eq!(a.players_area, full_hd());
}

#[test]
fn rl11_with_no_player_shown_the_cartwall_takes_the_middle_area() {
    let mut s = four();
    s.layout.hidden_players = s.player_ids();
    let a = pass(&s, full_hd(), None);
    assert!(a.widths.is_empty());
    assert!(!a.overflow);
    assert_eq!(a.cartwall, Some(full_hd()));
}

#[test]
fn rl11_fitting_the_content_is_capped_at_sixty_percent_as_before() {
    let s = four();
    let tall = CartwallSizes {
        content_height: 900.0,
        min_height: 300.0,
    };
    let a = pass_with(&s, full_hd(), tall, None);
    assert!(near(
        a.cartwall.unwrap().height(),
        1006.0 * CONTENT_HEIGHT_CAP
    ));
}

#[test]
fn rl11_a_bottom_share_keeps_both_minimums() {
    let mut s = four();
    s.layout.cartwall.size = Some(0.95);
    let a = pass(&s, full_hd(), None);
    // RL8: the players keep their least height.
    assert!(near(
        a.cartwall.unwrap().height(),
        1006.0 - PANEL_GAP - MIN_PLAYER_HEIGHT
    ));
    assert!(near(a.players_area.height(), MIN_PLAYER_HEIGHT));
    s.layout.cartwall.size = Some(0.05);
    let a = pass(&s, full_hd(), None);
    assert!(
        near(a.cartwall.unwrap().height(), SIZES.min_height),
        "the cartwall keeps its own"
    );
}

#[test]
fn rl11_too_low_a_window_keeps_the_cartwall_minimum() {
    let mut s = four();
    s.layout.cartwall.size = Some(0.5);
    let low = Rect::from_min_size(pos2(8.0, 42.0), vec2(1904.0, 400.0));
    let a = pass(&s, low, None);
    assert!(near(a.cartwall.unwrap().height(), SIZES.min_height));
    assert!(near(
        a.players_area.height(),
        400.0 - SIZES.min_height - PANEL_GAP
    ));
}

#[test]
fn rl16_a_maximised_player_takes_the_spare_room_and_the_others_their_minimum() {
    let mut s = four();
    let p = s.player_ids();
    s.layout.player_shares = vec![(p[0], 3.0)];
    let a = pass(&s, full_hd(), Some(Panel::Player(p[1])));
    assert!(
        all_near(&widths(&a), &[380.0, 740.0, 380.0, 380.0]),
        "{a:?}"
    );
    assert!(
        near(a.cartwall.unwrap().height(), SIZES.min_height),
        "the cartwall shrinks to its minimum too"
    );
}

#[test]
fn rl16_a_maximised_cartwall_leaves_the_players_their_least_height() {
    let s = four();
    let a = pass(&s, full_hd(), Some(Panel::Cartwall));
    assert!(near(a.players_area.height(), MIN_PLAYER_HEIGHT));
    assert!(
        all_near(&widths(&a), &[470.0; 4]),
        "the widths do not change"
    );
}

#[test]
fn rl16_a_hidden_panel_is_never_maximised() {
    let mut s = four();
    let p = s.player_ids();
    s.layout.hidden_players = vec![p[0]];
    assert_eq!(
        pass(&s, full_hd(), Some(Panel::Player(p[0]))),
        pass(&s, full_hd(), None)
    );
    s.layout.cartwall.hidden = true;
    assert_eq!(
        pass(&s, full_hd(), Some(Panel::Cartwall)),
        pass(&s, full_hd(), None)
    );
}
