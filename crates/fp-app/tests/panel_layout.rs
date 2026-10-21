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
    Arrangement, CONTENT_HEIGHT_CAP, CartwallDrag, CartwallSizes, LiveSplit, MIN_PLAYER_HEIGHT,
    MIN_PLAYER_WIDTH, Maximised, PANEL_GAP, PassInput, SCROLL_BAR_ROOM, SHARE_FLOOR, arrange,
    cartwall_range, drag_pair, pair_shares, panel_on_air,
};
use fp_model::layout::{Dock, MAX_CARTWALL_SHARE, MIN_CARTWALL_SHARE, Panel};
use fp_model::{AppState, Command, Config, PlayerId, TrackAnalysis, apply};

/// The middle area of a 1920 × 1080 window, inside its 8-point margin:
/// under the 34-point top bar, over the 24-point status bar.
fn full_hd() -> Rect {
    Rect::from_min_max(pos2(8.0, 42.0), pos2(1912.0, 1048.0))
}

/// A page of two rows of eight: today's content height (24 + 6 + 2 × 40 +
/// 6), the least height (the rows at 28) and the least width (eight columns
/// of 56 with seven gaps of 6); open.
const SIZES: CartwallSizes = CartwallSizes {
    content_height: 116.0,
    min_height: 92.0,
    min_width: 490.0,
    collapsed: None,
};

/// The same cartwall collapsed: only its 24-point header.
const COLLAPSED: CartwallSizes = CartwallSizes {
    content_height: 24.0,
    min_height: 24.0,
    min_width: 490.0,
    collapsed: Some(24.0),
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
        ..SIZES
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
fn rl8_an_overflowing_row_keeps_its_least_height_above_the_scroll_bar() {
    let mut s = four();
    s.layout.cartwall.size = Some(0.95);
    let narrow = Rect::from_min_max(pos2(8.0, 42.0), pos2(1208.0, 1048.0));
    let a = pass(&s, narrow, None);
    assert!(a.overflow, "{a:?}");
    assert!(
        near(a.players_area.height(), MIN_PLAYER_HEIGHT + SCROLL_BAR_ROOM),
        "{a:?}"
    );
    let a = pass(&s, narrow, Some(Panel::Cartwall));
    assert!(
        near(a.players_area.height(), MIN_PLAYER_HEIGHT + SCROLL_BAR_ROOM),
        "{a:?}"
    );
    // Without the overflow, no bar and no room for it.
    let a = pass(&s, full_hd(), Some(Panel::Cartwall));
    assert!(near(a.players_area.height(), MIN_PLAYER_HEIGHT), "{a:?}");
}

#[test]
fn rl8_the_scroll_bar_room_holds_egui_s_bar() {
    let scroll = egui::Style::default().spacing.scroll;
    assert!(SCROLL_BAR_ROOM >= scroll.bar_width + scroll.bar_inner_margin);
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

#[test]
fn rl16_maximise_ends_when_the_arrangement_changes() {
    let s = four();
    let p = s.player_ids();
    let m = Maximised::new(Panel::Player(p[0]), &s);
    assert!(m.holds(&s));
    let mut resized = s.clone();
    apply(&mut resized, Command::EqualPlayerWidths).unwrap();
    assert!(m.holds(&resized), "a resize keeps it");
    for command in [
        Command::SetPanelHidden {
            panel: Panel::Player(p[1]),
            hidden: true,
        },
        Command::SetPanelHidden {
            panel: Panel::Cartwall,
            hidden: true,
        },
        Command::SetCartwallDock(Dock::Left),
        Command::SetLayoutLocked(true),
        Command::SetPlayerCount(3),
    ] {
        let mut t = s.clone();
        apply(&mut t, command.clone()).unwrap();
        assert!(!m.holds(&t), "{command:?}");
    }
}

// --- side docks (RL6, RL8, RL11, RL14, RL16)

/// Two players: room for both beside a side cartwall at 1904 points.
fn two() -> AppState {
    let mut config = Config::default();
    config.players.count = 2;
    AppState::new(config, "Main")
}

fn docked(mut s: AppState, dock: Dock, size: Option<f32>) -> AppState {
    s.layout.cartwall.dock = dock;
    s.layout.cartwall.size = size;
    s
}

fn range(s: &AppState, area: Rect, sizes: CartwallSizes) -> Option<(f32, f32)> {
    let players = s.player_ids();
    cartwall_range(&PassInput {
        players: &players,
        layout: &s.layout,
        area,
        cartwall: sizes,
        maximised: None,
    })
}

#[test]
fn rl6_a_left_dock_is_the_full_height_on_the_left() {
    let s = docked(two(), Dock::Left, Some(0.3));
    let a = pass(&s, full_hd(), None);
    let cart = a.cartwall.unwrap();
    // 0.3 × 1904, between the least width (490) and what leaves two players
    // theirs (1904 − 8 − 2 × 380 − 8).
    assert!(near(cart.width(), 571.2), "{a:?}");
    assert_eq!(
        (cart.left(), cart.top(), cart.bottom()),
        (8.0, 42.0, 1048.0)
    );
    assert!(
        near(a.players_area.left(), cart.right() + PANEL_GAP),
        "{a:?}"
    );
    assert_eq!(a.players_area.right(), 1912.0);
    assert_eq!(
        (a.players_area.top(), a.players_area.bottom()),
        (42.0, 1048.0)
    );
    // The row is 1324.8 wide: 1324.8 − 8 − 2 × 380 = 556.8 spare, half each.
    assert!(all_near(&widths(&a), &[658.4, 658.4]), "{a:?}");
    assert!(!a.overflow);
}

#[test]
fn rl6_a_right_dock_is_the_full_height_on_the_right() {
    let s = docked(two(), Dock::Right, Some(0.3));
    let a = pass(&s, full_hd(), None);
    let cart = a.cartwall.unwrap();
    assert!(near(cart.width(), 571.2), "{a:?}");
    assert_eq!(
        (cart.right(), cart.top(), cart.bottom()),
        (1912.0, 42.0, 1048.0)
    );
    assert_eq!(a.players_area.left(), 8.0);
    assert!(
        near(a.players_area.right() + PANEL_GAP, cart.left()),
        "{a:?}"
    );
    assert!(all_near(&widths(&a), &[658.4, 658.4]), "{a:?}");
}

#[test]
fn rl6_a_side_dock_without_a_size_takes_the_default_share() {
    let s = docked(two(), Dock::Left, None);
    assert!(near(
        pass(&s, full_hd(), None).cartwall.unwrap().width(),
        571.2
    ));
}

#[test]
fn rl8_a_side_cartwall_keeps_its_least_width() {
    let s = docked(two(), Dock::Left, Some(0.05));
    let a = pass(&s, full_hd(), None);
    assert!(near(a.cartwall.unwrap().width(), SIZES.min_width), "{a:?}");
}

#[test]
fn rl11_a_side_share_leaves_the_players_their_minimum() {
    let s = docked(two(), Dock::Left, Some(0.95));
    let a = pass(&s, full_hd(), None);
    // 1904 − 8 − (2 × 380 + 8)
    assert!(near(a.cartwall.unwrap().width(), 1128.0), "{a:?}");
    assert!(all_near(&widths(&a), &[MIN_PLAYER_WIDTH; 2]), "{a:?}");
    assert!(!a.overflow);
}

#[test]
fn rl11_too_narrow_a_window_on_a_side_keeps_the_cartwall_minimum() {
    // Four players need 4 × 380 + 3 × 8 = 1544; with the cartwall's 490 and
    // its gap that is 2042, more than 1904.
    let s = docked(four(), Dock::Right, Some(0.3));
    let a = pass(&s, full_hd(), None);
    assert!(near(a.cartwall.unwrap().width(), SIZES.min_width), "{a:?}");
    assert!(a.overflow);
    assert!(all_near(&widths(&a), &[MIN_PLAYER_WIDTH; 4]), "{a:?}");
    assert!(a.players_area.right() <= a.cartwall.unwrap().left() - PANEL_GAP + 0.01);
}

#[test]
fn rl11_a_window_narrower_than_the_cartwall_minimum_gives_it_the_whole_width() {
    let s = docked(two(), Dock::Left, Some(0.3));
    let narrow = Rect::from_min_size(pos2(8.0, 42.0), vec2(400.0, 1006.0));
    let a = pass(&s, narrow, None);
    assert_eq!(a.cartwall, Some(narrow));
    assert_eq!(a.players_area.width(), 0.0);
    assert!(a.overflow);
    assert!(widths(&a).iter().all(|w| w.is_finite()));
}

#[test]
fn rl11_with_no_player_shown_a_side_cartwall_takes_the_middle_area() {
    let mut s = docked(two(), Dock::Left, Some(0.3));
    s.layout.hidden_players = s.player_ids();
    let a = pass(&s, full_hd(), None);
    assert_eq!(a.cartwall, Some(full_hd()));
    assert!(a.widths.is_empty());
    assert_eq!(a.splitter, None);
}

#[test]
fn rl16_a_maximised_player_shrinks_a_side_cartwall_to_its_least_width() {
    let s = docked(two(), Dock::Left, Some(0.3));
    let p = s.player_ids();
    let a = pass(&s, full_hd(), Some(Panel::Player(p[0])));
    assert!(near(a.cartwall.unwrap().width(), SIZES.min_width), "{a:?}");
    // The row is 1904 − 490 − 8 = 1406: 1406 − 8 − 760 = 638 spare, all P1's.
    assert!(all_near(&widths(&a), &[1018.0, MIN_PLAYER_WIDTH]), "{a:?}");
}

#[test]
fn rl16_a_maximised_side_cartwall_leaves_the_players_their_minimum() {
    let s = docked(two(), Dock::Right, Some(0.3));
    let a = pass(&s, full_hd(), Some(Panel::Cartwall));
    assert!(near(a.cartwall.unwrap().width(), 1128.0), "{a:?}");
    assert!(all_near(&widths(&a), &[MIN_PLAYER_WIDTH; 2]), "{a:?}");
}

#[test]
fn rl12_the_cartwall_range_is_its_minimum_up_to_the_players_minimum() {
    assert_eq!(
        range(&four(), full_hd(), SIZES),
        Some((92.0, 1006.0 - PANEL_GAP - MIN_PLAYER_HEIGHT))
    );
    assert_eq!(
        range(&docked(two(), Dock::Left, None), full_hd(), SIZES),
        Some((490.0, 1128.0))
    );
    let low = Rect::from_min_size(pos2(8.0, 42.0), vec2(1904.0, 400.0));
    assert_eq!(
        range(&four(), low, SIZES),
        Some((92.0, 92.0)),
        "never below its own minimum"
    );
    // Four players overflow a 1350-point row: the players keep their least
    // height and the scroll bar's strip.
    assert_eq!(
        range(&four(), laptop(), SIZES),
        Some((
            92.0,
            laptop().height() - PANEL_GAP - MIN_PLAYER_HEIGHT - SCROLL_BAR_ROOM
        ))
    );
    let mut s = four();
    s.layout.cartwall.hidden = true;
    assert_eq!(range(&s, full_hd(), SIZES), None, "hidden");
    let mut s = four();
    s.layout.hidden_players = s.player_ids();
    assert_eq!(range(&s, full_hd(), SIZES), None, "no player shown");
    assert_eq!(range(&four(), full_hd(), COLLAPSED), None, "collapsed");
}

#[test]
fn rl12_the_cartwall_splitter_is_the_gap_beside_it() {
    let a = pass(&four(), full_hd(), None);
    assert_eq!(
        a.splitter,
        Some(Rect::from_min_max(pos2(8.0, 924.0), pos2(1912.0, 932.0)))
    );
    let a = pass(&docked(two(), Dock::Left, Some(0.3)), full_hd(), None);
    let (cart, gap) = (a.cartwall.unwrap(), a.splitter.unwrap());
    assert!(near(gap.left(), cart.right()) && near(gap.right(), a.players_area.left()));
    assert_eq!((gap.top(), gap.bottom()), (42.0, 1048.0));
    let a = pass(&docked(two(), Dock::Right, Some(0.3)), full_hd(), None);
    let (cart, gap) = (a.cartwall.unwrap(), a.splitter.unwrap());
    assert!(near(gap.left(), a.players_area.right()) && near(gap.right(), cart.left()));
    let mut s = four();
    s.layout.cartwall.hidden = true;
    assert_eq!(pass(&s, full_hd(), None).splitter, None);
}

#[test]
fn rl13_only_a_bottom_cartwall_without_a_share_fits_its_content() {
    assert!(pass(&four(), full_hd(), None).fit_content);
    let mut s = four();
    s.layout.cartwall.size = Some(0.3);
    assert!(!pass(&s, full_hd(), None).fit_content, "a share");
    assert!(
        !pass(&docked(two(), Dock::Left, None), full_hd(), None).fit_content,
        "a side"
    );
    assert!(
        !pass(&four(), full_hd(), Some(Panel::Cartwall)).fit_content,
        "maximised"
    );
    let mut s = four();
    s.layout.hidden_players = s.player_ids();
    assert!(
        !pass(&s, full_hd(), None).fit_content,
        "alone, it fills the area"
    );
}

#[test]
fn rl14_collapsed_on_a_side_is_a_strip_as_wide_as_the_header_is_high() {
    let s = docked(two(), Dock::Left, Some(0.4));
    let a = pass_with(&s, full_hd(), COLLAPSED, None);
    let cart = a.cartwall.unwrap();
    assert!(near(cart.width(), 24.0), "{a:?}");
    assert_eq!((cart.top(), cart.bottom()), (42.0, 1048.0));
    assert!(near(a.players_area.left(), 8.0 + 24.0 + PANEL_GAP), "{a:?}");
    assert!(a.splitter.is_some(), "the gap is still there");
    assert_eq!(
        range(&s, full_hd(), COLLAPSED),
        None,
        "but nothing to resize"
    );
    // Expanded, it has its share back: the model kept it.
    let a = pass(&s, full_hd(), None);
    assert!(near(a.cartwall.unwrap().width(), 0.4 * 1904.0), "{a:?}");
}

#[test]
fn rl14_collapsed_at_the_bottom_is_its_header_whatever_its_share() {
    let mut s = four();
    s.layout.cartwall.size = Some(0.5);
    let a = pass_with(&s, full_hd(), COLLAPSED, None);
    assert!(near(a.cartwall.unwrap().height(), 24.0), "{a:?}");
}

#[test]
fn rl14_a_collapsed_cartwall_is_never_maximised() {
    for s in [four(), docked(two(), Dock::Right, None)] {
        assert_eq!(
            pass_with(&s, full_hd(), COLLAPSED, Some(Panel::Cartwall)),
            pass_with(&s, full_hd(), COLLAPSED, None)
        );
    }
}

/// The middle area of a 1366 × 768 window: four players overflow its row.
fn laptop() -> Rect {
    Rect::from_min_max(pos2(8.0, 42.0), pos2(1358.0, 736.0))
}

/// A page of one row: 24 + 6 + 40 + 6, its least 24 + 6 + 28 + 6.
const ONE_ROW: CartwallSizes = CartwallSizes {
    content_height: 76.0,
    min_height: 64.0,
    min_width: 120.0,
    collapsed: None,
};

/// A page of six rows: 24 + 6 + 6 × 40 + 5 × 6 + 6, its least with the
/// rows at 28.
const SIX_ROWS: CartwallSizes = CartwallSizes {
    content_height: 306.0,
    min_height: 234.0,
    min_width: 490.0,
    collapsed: None,
};

#[test]
fn rl12_the_cartwall_range_keeps_the_size_s_shares() {
    // The middle area of a 3840 × 2160 window: a one-row page is less than
    // 5 % of its height and of its width.
    let tall = Rect::from_min_max(pos2(8.0, 42.0), pos2(3832.0, 2128.0));
    let (least, _) = range(&four(), tall, ONE_ROW).unwrap();
    assert!(near(least, MIN_CARTWALL_SHARE * tall.height()), "{least}");
    let (least, _) = range(&docked(two(), Dock::Left, None), tall, ONE_ROW).unwrap();
    assert!(near(least, MIN_CARTWALL_SHARE * tall.width()), "{least}");
    // So large an area that the players' minimum is less than 5 % of it.
    let huge = Rect::from_min_size(pos2(0.0, 0.0), vec2(20_000.0, 20_000.0));
    let (_, most) = range(&four(), huge, ONE_ROW).unwrap();
    assert!(near(most, MAX_CARTWALL_SHARE * 20_000.0), "{most}");
    let (_, most) = range(&docked(two(), Dock::Right, None), huge, ONE_ROW).unwrap();
    assert!(near(most, MAX_CARTWALL_SHARE * 20_000.0), "{most}");
    // Never below its own minimum, and the least is never above the most.
    let low = Rect::from_min_size(pos2(8.0, 42.0), vec2(1904.0, 400.0));
    assert_eq!(range(&four(), low, SIZES), Some((92.0, 92.0)));
}

// --- the cartwall drag (RL12)

#[test]
fn rl12_a_drag_of_a_cartwall_taller_than_its_range_starts_within_it() {
    // Fitting six rows (306 points) leaves less than the overflowing
    // players' least height: the cartwall is drawn taller than its range.
    let s = four();
    let a = pass_with(&s, laptop(), SIX_ROWS, None);
    let start = a.cartwall.unwrap().height();
    let range = range(&s, laptop(), SIX_ROWS).unwrap();
    assert!(start > range.1, "{start} {range:?}");
    let origin = pos2(600.0, 500.0);
    let mut drag = CartwallDrag::new(Dock::Bottom, laptop(), range, origin, start);
    assert_eq!(drag.size(), range.1, "it starts at the range's most");
    drag.follow(pos2(600.0, 506.0));
    drag.follow(origin);
    assert_eq!(drag.size(), range.1);
    assert_eq!(drag.release(None, 1.0), None, "back where it began");
}

#[test]
fn rl12_a_bottom_drag_grows_the_cartwall_upwards_within_its_range() {
    let mut drag = CartwallDrag::new(
        Dock::Bottom,
        full_hd(),
        (92.0, 566.0),
        pos2(900.0, 928.0),
        116.0,
    );
    drag.follow(pos2(900.0, 868.0));
    assert_eq!(drag.size(), 176.0);
    drag.follow(pos2(900.0, 100.0));
    assert_eq!(drag.size(), 566.0, "the players keep their least height");
    drag.follow(pos2(900.0, 1040.0));
    assert_eq!(drag.size(), 92.0, "the cartwall keeps its own");
    drag.follow(pos2(f32::NAN, f32::NAN));
    assert_eq!(
        drag.size(),
        92.0,
        "a pointer that is not a number moves nothing"
    );
}

#[test]
fn rl12_a_side_drag_grows_the_cartwall_towards_the_players() {
    let mut left = CartwallDrag::new(
        Dock::Left,
        full_hd(),
        (490.0, 1128.0),
        pos2(583.0, 500.0),
        571.2,
    );
    left.follow(pos2(633.0, 500.0));
    assert!(near(left.size(), 621.2));
    let mut right = CartwallDrag::new(
        Dock::Right,
        full_hd(),
        (490.0, 1128.0),
        pos2(1336.0, 500.0),
        571.2,
    );
    right.follow(pos2(1286.0, 500.0));
    assert!(near(right.size(), 621.2));
}

#[test]
fn rl12_a_cartwall_drag_sends_its_share_once_and_holds_until_the_echo() {
    let area = full_hd();
    let range = Some((92.0, 566.0));
    let mut drag = CartwallDrag::new(Dock::Bottom, area, (92.0, 566.0), pos2(900.0, 928.0), 116.0);
    // 0.2 of the 1006-point area is 201.2: 85.2 points up.
    drag.follow(pos2(900.0, 842.8));
    let sent = drag.release(None, 10.5);
    let Some(Command::SetCartwallSize(Some(share))) = sent else {
        panic!("{sent:?}")
    };
    assert!((share - 0.2).abs() < 1e-4, "{share}");
    assert!(!drag.is_dragging());
    assert_eq!(drag.release(None, 10.5), None, "sent once");
    drag.follow(pos2(900.0, 500.0));
    assert!(
        near(drag.size(), 201.2),
        "a released drag no longer follows the pointer"
    );
    assert!(!drag.stale(Dock::Bottom, area, range, None, 10.0), "held");
    assert!(
        drag.stale(Dock::Bottom, area, range, Some(0.2), 10.0),
        "the echo arrived"
    );
    assert!(
        drag.stale(Dock::Bottom, area, range, None, 10.5),
        "the hold ran out"
    );
}

#[test]
fn rl12_a_cartwall_drag_back_to_where_it_began_sends_nothing() {
    let mut drag = CartwallDrag::new(
        Dock::Left,
        full_hd(),
        (490.0, 1128.0),
        pos2(583.0, 500.0),
        571.2,
    );
    drag.follow(pos2(683.0, 500.0));
    drag.follow(pos2(583.2, 500.0));
    assert_eq!(drag.release(Some(0.3), 1.0), None);
}

#[test]
fn rl12_a_dock_change_a_resized_window_or_a_new_range_ends_a_cartwall_drag() {
    let area = full_hd();
    let range = Some((92.0, 566.0));
    let drag = CartwallDrag::new(Dock::Bottom, area, (92.0, 566.0), pos2(900.0, 928.0), 116.0);
    assert!(!drag.stale(Dock::Bottom, area, range, None, 0.0));
    assert!(
        drag.stale(Dock::Left, area, range, None, 0.0),
        "docked elsewhere"
    );
    let smaller = Rect::from_min_max(area.min, pos2(1912.0, 900.0));
    assert!(
        drag.stale(Dock::Bottom, smaller, range, None, 0.0),
        "the window was resized"
    );
    assert!(
        drag.stale(Dock::Bottom, area, Some((150.0, 566.0)), None, 0.0),
        "the page's rows changed"
    );
    assert!(
        drag.stale(Dock::Bottom, area, None, None, 0.0),
        "collapsed or hidden"
    );
}

#[test]
fn rl12_the_pass_draws_a_cartwall_drag_exactly() {
    let mut drag = CartwallDrag::new(
        Dock::Bottom,
        full_hd(),
        (92.0, 566.0),
        pos2(900.0, 928.0),
        116.0,
    );
    drag.follow(pos2(900.0, 777.0));
    let mut s = four();
    s.layout.cartwall.size = Some(drag.share());
    let a = pass(&s, full_hd(), None);
    assert!(near(a.cartwall.unwrap().height(), drag.size()), "{a:?}");

    // On a side, with a player hidden: the range leaves the one shown
    // player its minimum.
    for (dock, towards) in [(Dock::Left, 1.0), (Dock::Right, -1.0)] {
        let mut s = docked(two(), dock, Some(0.3));
        s.layout.hidden_players = vec![s.player_ids()[1]];
        let start = pass(&s, full_hd(), None).cartwall.unwrap().width();
        let range = range(&s, full_hd(), SIZES).unwrap();
        assert!(
            near(range.1, 1904.0 - PANEL_GAP - MIN_PLAYER_WIDTH),
            "{range:?}"
        );
        let origin = pos2(900.0, 500.0);
        let mut drag = CartwallDrag::new(dock, full_hd(), range, origin, start);
        for moved in [333.0, 5000.0, -5000.0] {
            drag.follow(pos2(900.0 + towards * moved, 500.0));
            s.layout.cartwall.size = Some(drag.share());
            let a = pass(&s, full_hd(), None);
            assert!(
                near(a.cartwall.unwrap().width(), drag.size()),
                "{dock:?} {moved} {a:?}"
            );
        }
    }

    // A one-row page on a tall window: dragged to its least, the drawn
    // size is the pointer's, not the share's floor.
    let tall = Rect::from_min_max(pos2(8.0, 42.0), pos2(3832.0, 2128.0));
    let s = four();
    let range = range(&s, tall, ONE_ROW).unwrap();
    let mut drag = CartwallDrag::new(Dock::Bottom, tall, range, pos2(900.0, 2000.0), 76.0);
    drag.follow(pos2(900.0, 2100.0));
    let mut s = s;
    s.layout.cartwall.size = Some(drag.share());
    let a = pass_with(&s, tall, ONE_ROW, None);
    assert!(near(a.cartwall.unwrap().height(), drag.size()), "{a:?}");
}
