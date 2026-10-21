#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Resizable layout spec RL8, RL11 and RL16: the pure pixel pass, one test
//! per rule, named after it.

use std::path::PathBuf;

use egui::{Rect, pos2, vec2};
use fp_app::ui::panel_layout::{
    Arrangement, CONTENT_HEIGHT_CAP, CartwallSizes, LiveSplit, MIN_PLAYER_HEIGHT, MIN_PLAYER_WIDTH,
    PANEL_GAP, PassInput, SHARE_FLOOR, arrange, drag_pair, pair_shares, panel_on_air,
};
use fp_model::layout::Panel;
use fp_model::{AppState, Command, Config, PlayerId, TrackAnalysis, apply};

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

#[test]
fn rl12_a_drag_moves_the_pair_and_keeps_its_total_and_minimums() {
    assert_eq!(drag_pair((480.0, 480.0), 60.0), (540.0, 420.0));
    assert_eq!(drag_pair((480.0, 480.0), 400.0), (580.0, MIN_PLAYER_WIDTH));
    assert_eq!(drag_pair((480.0, 480.0), -400.0), (MIN_PLAYER_WIDTH, 580.0));
    assert_eq!(
        drag_pair((380.0, 380.0), 50.0),
        (380.0, 380.0),
        "no room: nothing moves"
    );
    assert_eq!(drag_pair((480.0, 480.0), f32::NAN), (480.0, 480.0));
}

#[test]
fn rl12_the_shares_are_the_room_above_the_minimums() {
    assert_eq!(pair_shares((540.0, 420.0)), Some((160.0, 40.0)));
    let (left, right) = pair_shares((580.0, 380.0)).unwrap();
    assert_eq!(left, 200.0);
    assert!(
        right > 0.0 && right <= 200.0 * SHARE_FLOOR,
        "a player at its minimum sends a sliver, never 0: {right}"
    );
    assert_eq!(pair_shares((380.0, 380.0)), None);
    assert_eq!(
        pair_shares((f32::NAN, 400.0)),
        Some((SHARE_FLOOR * 20.0, 20.0))
    );
}

#[test]
fn rl12_a_live_split_sends_once_and_holds_until_the_echo() {
    let (a, b, c) = (PlayerId(1), PlayerId(2), PlayerId(3));
    let shown = vec![a, b, c];
    let mut live = LiveSplit::new((a, b), shown.clone(), 1400.0, 500.0, (480.0, 480.0));
    assert!(live.is_dragging());
    live.follow(560.0);
    let mut w = vec![(a, 480.0), (b, 480.0), (c, 424.0)];
    live.apply(&mut w);
    assert_eq!(w, vec![(a, 540.0), (b, 420.0), (c, 424.0)]);
    let before: Vec<(PlayerId, f32)> = Vec::new();
    assert_eq!(
        live.release(before.clone(), 10.5),
        Some(Command::SetPlayerShares {
            left: (a, 160.0),
            right: (b, 40.0),
        })
    );
    assert!(!live.is_dragging());
    assert_eq!(live.release(before.clone(), 10.5), None, "sent once");
    live.follow(900.0);
    let mut w = vec![(a, 480.0), (b, 480.0), (c, 424.0)];
    live.apply(&mut w);
    assert_eq!(
        w[0],
        (a, 540.0),
        "a released drag no longer follows the pointer"
    );
    assert!(!live.stale(&shown, 1400.0, &before, 10.0), "held");
    assert!(
        live.stale(&shown, 1400.0, &[(a, 160.0)], 10.0),
        "the echo arrived"
    );
    assert!(
        live.stale(&shown, 1400.0, &before, 10.5),
        "the hold ran out"
    );
}

#[test]
fn rl12_a_drag_back_to_where_it_began_sends_nothing() {
    let (a, b) = (PlayerId(1), PlayerId(2));
    let mut live = LiveSplit::new((a, b), vec![a, b], 968.0, 500.0, (480.0, 480.0));
    live.follow(560.0);
    live.follow(500.2);
    assert_eq!(live.release(Vec::new(), 1.0), None);
}

#[test]
fn rl12_a_window_resized_or_a_player_gone_ends_a_drag() {
    let (a, b, c) = (PlayerId(1), PlayerId(2), PlayerId(3));
    let live = LiveSplit::new((a, b), vec![a, b, c], 1400.0, 500.0, (480.0, 480.0));
    assert!(!live.stale(&[a, b, c], 1400.0, &[], 0.0));
    assert!(
        live.stale(&[a, b, c], 1300.0, &[], 0.0),
        "the window was resized"
    );
    assert!(live.stale(&[a, b], 1400.0, &[], 0.0), "a player went away");
}

#[test]
fn rl15_a_panel_is_on_air_while_it_sounds() {
    let mut s = four();
    let playlist = s.playlists.first_id().unwrap();
    apply(
        &mut s,
        Command::InsertPaths {
            playlist,
            index: 0,
            paths: vec![
                PathBuf::from("/music/a.flac"),
                PathBuf::from("/music/b.flac"),
            ],
        },
    )
    .unwrap();
    let p = s.player_ids();
    assert!(!panel_on_air(&s, Panel::Player(p[0])));
    apply(&mut s, Command::Play(p[0])).unwrap();
    assert!(panel_on_air(&s, Panel::Player(p[0])));
    apply(&mut s, Command::Pause(p[0])).unwrap();
    assert!(
        panel_on_air(&s, Panel::Player(p[0])),
        "paused is still on air"
    );
    apply(&mut s, Command::Stop(p[0])).unwrap();
    assert!(!panel_on_air(&s, Panel::Player(p[0])));
    assert!(!panel_on_air(&s, Panel::Player(PlayerId(999_999))));

    assert!(!panel_on_air(&s, Panel::Cartwall));
    let page = s.cartwall.pages[0].id;
    apply(
        &mut s,
        Command::AssignCartFile {
            page,
            index: 0,
            path: PathBuf::from("/carts/id.wav"),
        },
    )
    .unwrap();
    let cart = &s.cartwall.pages[0].carts[0];
    let (id, track) = (cart.id, cart.track.unwrap());
    apply(
        &mut s,
        Command::ApplyAnalysis {
            track,
            analysis: Box::new(TrackAnalysis {
                duration_secs: 10.0,
                ..TrackAnalysis::default()
            }),
        },
    )
    .unwrap();
    apply(&mut s, Command::FireCart(id)).unwrap();
    assert!(panel_on_air(&s, Panel::Cartwall));
}
