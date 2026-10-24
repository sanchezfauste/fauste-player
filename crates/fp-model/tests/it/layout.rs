#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Resizable layout spec RL1–RL10, RL3 and the model side of RL19: one
//! test per rule, named after it.

use crate::common;

use common::{fixture, p0};
use fp_model::{
    AppState, CartwallLayout, Command, Config, DEFAULT_SIDE_SHARE, Dock, Layout,
    MAX_CARTWALL_SHARE, MIN_CARTWALL_SHARE, ModelError, Panel, PlayerId, RestoreParts, Transport,
    apply, settle_layout,
};

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-5
}

fn all_close(got: &[f32], want: &[f32]) -> bool {
    got.len() == want.len() && got.iter().zip(want).all(|(a, b)| close(*a, *b))
}

/// The fractions of the shown players, in display order.
fn fractions(s: &AppState) -> Vec<f32> {
    s.layout
        .shown_fractions(&s.player_ids())
        .into_iter()
        .map(|(_, f)| f)
        .collect()
}

#[test]
fn rl1_the_default_layout_is_todays_window() {
    let s = AppState::new(Config::default(), "Main");
    assert_eq!(s.layout, Layout::default());
    assert!(s.layout.player_shares.is_empty());
    assert!(s.layout.hidden_players.is_empty());
    assert_eq!(
        s.layout.cartwall,
        CartwallLayout {
            dock: Dock::Bottom,
            size: None,
            hidden: false
        }
    );
    assert!(!s.layout.locked);
    assert!(all_close(&fractions(&s), &[0.25; 4]));
}

#[test]
fn rl1_panels_are_the_shown_players_and_the_cartwall() {
    let mut s = fixture(0);
    let p = s.player_ids();
    assert_eq!(s.layout.shown_panels(&p), 5);
    s.layout.hidden_players = vec![p[0], p[1], p[2]];
    s.layout.cartwall.hidden = true;
    assert_eq!(s.layout.shown_panels(&p), 1);
    assert!(s.layout.is_hidden(Panel::Cartwall));
    assert!(s.layout.is_hidden(Panel::Player(p[0])));
    assert!(s.layout.is_last_shown(&p, Panel::Player(p[3])));
    assert!(
        !s.layout.is_last_shown(&p, Panel::Cartwall),
        "already hidden"
    );
    assert!(!s.layout.is_last_shown(&p, Panel::Player(PlayerId(999_999))));
}

#[test]
fn rl2_with_no_share_every_player_gets_one() {
    let s = fixture(0);
    let p = s.player_ids();
    assert!(p.iter().all(|id| close(s.layout.share(&p, *id), 1.0)));
}

#[test]
fn rl2_a_player_without_a_share_gets_the_mean_of_the_others() {
    let mut s = fixture(0);
    let p = s.player_ids();
    s.layout.player_shares = vec![(p[0], 2.0), (p[1], 1.0)];
    assert!(close(s.layout.share(&p, p[2]), 1.5));
    assert!(all_close(
        &fractions(&s),
        &[2.0 / 6.0, 1.0 / 6.0, 1.5 / 6.0, 1.5 / 6.0]
    ));
}

#[test]
fn rl2_a_bad_share_counts_as_missing() {
    let mut s = fixture(0);
    let p = s.player_ids();
    s.layout.player_shares = vec![(p[0], f32::NAN), (p[1], -1.0), (p[2], 0.0), (p[3], 2.0)];
    assert!(p.iter().all(|id| close(s.layout.share(&p, *id), 2.0)));
    assert!(all_close(&fractions(&s), &[0.25; 4]));
}

#[test]
fn rl2_a_share_of_a_player_that_does_not_exist_is_ignored() {
    let mut s = fixture(0);
    let p = s.player_ids();
    s.layout.player_shares = vec![(PlayerId(999_999), 10.0)];
    assert!(close(s.layout.share(&p, p[0]), 1.0));
}

#[test]
fn rl2_a_hidden_player_keeps_its_share() {
    let mut s = fixture(0);
    let p = s.player_ids();
    s.layout.player_shares = vec![(p[0], 1.0), (p[1], 3.0), (p[2], 1.0), (p[3], 1.0)];
    s.layout.hidden_players = vec![p[1]];
    assert_eq!(s.layout.shown_players(&p), vec![p[0], p[2], p[3]]);
    assert!(close(s.layout.share(&p, p[1]), 3.0));
    assert!(all_close(&fractions(&s), &[1.0 / 3.0; 3]));
    s.layout.hidden_players.clear();
    assert!(
        close(fractions(&s)[1], 0.5),
        "shown again, it has its width back"
    );
}

#[test]
fn rl2_huge_or_tiny_shares_still_give_fractions_that_add_up_to_one() {
    let mut s = fixture(0);
    let p = s.player_ids();
    s.layout.player_shares = p.iter().map(|id| (*id, f32::MAX)).collect();
    assert!(all_close(&fractions(&s), &[0.25; 4]));
    s.layout.player_shares = vec![(p[0], 1e-30), (p[1], 1e-30)];
    let f = fractions(&s);
    assert!(f.iter().all(|x| x.is_finite() && *x >= 0.0), "{f:?}");
    assert!(close(f.iter().sum(), 1.0), "{f:?}");
}

#[test]
fn terms_neighbours_skip_hidden_players() {
    let mut s = fixture(0);
    let p = s.player_ids();
    assert!(s.layout.are_neighbours(&p, p[0], p[1]));
    assert!(!s.layout.are_neighbours(&p, p[1], p[0]), "left comes first");
    assert!(!s.layout.are_neighbours(&p, p[0], p[2]));
    s.layout.hidden_players = vec![p[1]];
    assert!(s.layout.are_neighbours(&p, p[0], p[2]));
    assert!(!s.layout.are_neighbours(&p, p[0], p[1]));
}

/// Every player's RL2 share, in display order.
fn shares(s: &AppState) -> Vec<f32> {
    let p = s.player_ids();
    p.iter().map(|id| s.layout.share(&p, *id)).collect()
}

#[test]
fn rl4_a_splitter_drag_sets_both_shares() {
    let mut s = fixture(0);
    let p = s.player_ids();
    apply(
        &mut s,
        Command::SetPlayerShares {
            left: (p[0], 1.5),
            right: (p[1], 0.5),
        },
    )
    .unwrap();
    assert!(all_close(&shares(&s), &[1.5, 0.5, 1.0, 1.0]));
}

#[test]
fn rl4_the_pair_keeps_its_sum_whatever_the_interface_sends() {
    let mut s = fixture(0);
    let p = s.player_ids();
    let before = fractions(&s);
    apply(
        &mut s,
        Command::SetPlayerShares {
            left: (p[1], 300.0),
            right: (p[2], 100.0),
        },
    )
    .unwrap();
    assert!(all_close(&shares(&s), &[1.0, 1.5, 0.5, 1.0]));
    let after = fractions(&s);
    assert!(close(after[0], before[0]) && close(after[3], before[3]));
}

#[test]
fn rl4_every_player_gets_its_share_written_down() {
    let mut s = fixture(0);
    let p = s.player_ids();
    s.layout.player_shares = vec![(p[0], 4.0), (PlayerId(999_999), 2.0)];
    apply(
        &mut s,
        Command::SetPlayerShares {
            left: (p[2], 1.0),
            right: (p[3], 7.0),
        },
    )
    .unwrap();
    let ids: Vec<PlayerId> = s.layout.player_shares.iter().map(|(id, _)| *id).collect();
    assert_eq!(ids, p, "one share per current player, the stranger gone");
    assert!(all_close(&shares(&s), &[4.0, 4.0, 1.0, 7.0]));
}

#[test]
fn rl4_neighbours_across_a_hidden_player() {
    let mut s = fixture(0);
    let p = s.player_ids();
    s.layout.hidden_players = vec![p[1]];
    apply(
        &mut s,
        Command::SetPlayerShares {
            left: (p[0], 0.5),
            right: (p[2], 1.5),
        },
    )
    .unwrap();
    assert!(all_close(&shares(&s), &[0.5, 1.0, 1.5, 1.0]));
}

#[test]
fn rl4_a_stranger_a_non_neighbour_and_a_bad_share_are_rejected() {
    let mut s = fixture(0);
    let p = s.player_ids();
    s.layout.hidden_players = vec![p[3]];
    let stranger = PlayerId(999_999);
    let before = s.clone();
    for (left, right) in [
        ((stranger, 1.0), (p[0], 1.0)),
        ((p[0], 1.0), (p[2], 1.0)),
        ((p[1], 1.0), (p[0], 1.0)),
        ((p[2], 1.0), (p[3], 1.0)),
        ((p[0], f32::NAN), (p[1], 1.0)),
        ((p[0], 1.0), (p[1], f32::INFINITY)),
        ((p[0], 0.0), (p[1], 2.0)),
        ((p[0], -1.0), (p[1], 3.0)),
    ] {
        let err = apply(&mut s, Command::SetPlayerShares { left, right }).unwrap_err();
        assert_eq!(err, ModelError::BadLayout, "{left:?} {right:?}");
        assert_eq!(s, before, "a refused command changes nothing");
    }
}

#[test]
fn rl5_equal_widths_gives_every_player_the_same_share() {
    let mut s = fixture(0);
    let p = s.player_ids();
    s.layout.player_shares = vec![(p[0], 3.0), (p[1], 0.2)];
    s.layout.hidden_players = vec![p[2]];
    apply(&mut s, Command::EqualPlayerWidths).unwrap();
    assert!(all_close(&shares(&s), &[1.0; 4]));
    assert!(all_close(&fractions(&s), &[1.0 / 3.0; 3]));
    assert_eq!(s.layout.hidden_players, vec![p[2]], "it shows nothing");
}

fn hide(s: &mut AppState, panel: Panel, hidden: bool) -> Result<(), ModelError> {
    apply(s, Command::SetPanelHidden { panel, hidden }).map(|_| ())
}

#[test]
fn rl6_a_side_dock_gets_the_default_share_and_the_bottom_fits_the_content() {
    let mut s = fixture(0);
    apply(&mut s, Command::SetCartwallDock(Dock::Left)).unwrap();
    assert_eq!(s.layout.cartwall.dock, Dock::Left);
    assert_eq!(s.layout.cartwall.size, Some(DEFAULT_SIDE_SHARE));
    apply(&mut s, Command::SetCartwallSize(Some(0.5))).unwrap();
    apply(&mut s, Command::SetCartwallDock(Dock::Right)).unwrap();
    assert_eq!(
        s.layout.cartwall.size,
        Some(DEFAULT_SIDE_SHARE),
        "a dock change resets the size"
    );
    apply(&mut s, Command::SetCartwallDock(Dock::Bottom)).unwrap();
    assert_eq!(s.layout.cartwall.size, None);
}

#[test]
fn rl6_docking_where_it_already_is_keeps_the_size() {
    let mut s = fixture(0);
    apply(&mut s, Command::SetCartwallSize(Some(0.4))).unwrap();
    apply(&mut s, Command::SetCartwallDock(Dock::Bottom)).unwrap();
    assert_eq!(s.layout.cartwall.size, Some(0.4));
}

#[test]
fn rl6_the_size_is_clamped() {
    let mut s = fixture(0);
    apply(&mut s, Command::SetCartwallSize(Some(0.0))).unwrap();
    assert_eq!(s.layout.cartwall.size, Some(MIN_CARTWALL_SHARE));
    apply(&mut s, Command::SetCartwallSize(Some(2.0))).unwrap();
    assert_eq!(s.layout.cartwall.size, Some(MAX_CARTWALL_SHARE));
    apply(&mut s, Command::SetCartwallSize(None)).unwrap();
    assert_eq!(
        s.layout.cartwall.size, None,
        "fit the content at the bottom"
    );
    apply(&mut s, Command::SetCartwallDock(Dock::Left)).unwrap();
    apply(&mut s, Command::SetCartwallSize(None)).unwrap();
    assert_eq!(
        s.layout.cartwall.size,
        Some(DEFAULT_SIDE_SHARE),
        "a side has no content size"
    );
    let before = s.clone();
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let err = apply(&mut s, Command::SetCartwallSize(Some(bad))).unwrap_err();
        assert_eq!(err, ModelError::BadLayout);
        assert_eq!(s, before);
    }
}

#[test]
fn rl6_the_cartwall_share_is_usable_on_every_dock() {
    let at = |dock, size| {
        CartwallLayout {
            dock,
            size,
            hidden: false,
        }
        .share()
    };
    assert_eq!(at(Dock::Bottom, None), None);
    assert_eq!(at(Dock::Bottom, Some(0.4)), Some(0.4));
    assert_eq!(at(Dock::Bottom, Some(f32::NAN)), None);
    assert_eq!(at(Dock::Left, None), Some(DEFAULT_SIDE_SHARE));
    assert_eq!(at(Dock::Right, Some(5.0)), Some(MAX_CARTWALL_SHARE));
    assert_eq!(at(Dock::Left, Some(-1.0)), Some(MIN_CARTWALL_SHARE));
}

#[test]
fn rl7_hiding_and_showing_a_panel() {
    let mut s = fixture(0);
    let p = s.player_ids();
    hide(&mut s, Panel::Player(p[1]), true).unwrap();
    hide(&mut s, Panel::Player(p[1]), true).unwrap();
    assert_eq!(
        s.layout.hidden_players,
        vec![p[1]],
        "hiding twice is hiding once"
    );
    hide(&mut s, Panel::Cartwall, true).unwrap();
    assert!(s.layout.cartwall.hidden);
    hide(&mut s, Panel::Player(p[1]), false).unwrap();
    hide(&mut s, Panel::Cartwall, false).unwrap();
    assert_eq!(s.layout, Layout::default());
}

#[test]
fn rl7_the_last_shown_panel_cannot_be_hidden() {
    let mut s = fixture(0);
    let p = s.player_ids();
    for panel in [
        Panel::Cartwall,
        Panel::Player(p[0]),
        Panel::Player(p[1]),
        Panel::Player(p[2]),
    ] {
        hide(&mut s, panel, true).unwrap();
    }
    let before = s.clone();
    assert_eq!(
        hide(&mut s, Panel::Player(p[3]), true).unwrap_err(),
        ModelError::LastPanel
    );
    assert_eq!(s, before);
    hide(&mut s, Panel::Cartwall, false).unwrap();
    hide(&mut s, Panel::Player(p[3]), true).unwrap();
}

#[test]
fn rl7_hiding_a_player_that_does_not_exist_is_refused() {
    let mut s = fixture(0);
    let stranger = PlayerId(999_999);
    assert_eq!(
        hide(&mut s, Panel::Player(stranger), true).unwrap_err(),
        ModelError::UnknownPlayer(stranger)
    );
}

#[test]
fn rl7_a_hidden_player_keeps_playing() {
    let mut s = fixture(3);
    let p = p0(&s);
    apply(&mut s, Command::Play(p)).unwrap();
    let actions = apply(
        &mut s,
        Command::SetPanelHidden {
            panel: Panel::Player(p),
            hidden: true,
        },
    )
    .unwrap();
    assert!(actions.is_empty(), "{actions:?}");
    assert_eq!(s.player(p).unwrap().transport, Transport::Playing);
    let actions = apply(&mut s, Command::Stop(p)).unwrap();
    assert!(!actions.is_empty(), "commands still reach a hidden player");
    assert_eq!(s.player(p).unwrap().transport, Transport::Stopped);
}

#[test]
fn rl7_collapsing_the_cartwall_works_while_locked() {
    let mut s = fixture(0);
    apply(&mut s, Command::SetLayoutLocked(true)).unwrap();
    apply(&mut s, Command::SetCartwallOpen(false)).unwrap();
    assert!(!s.cartwall.open);
}

#[test]
fn rl9_a_locked_layout_rejects_every_layout_command() {
    let mut s = fixture(0);
    let p = s.player_ids();
    apply(&mut s, Command::SetLayoutLocked(true)).unwrap();
    assert!(s.layout.locked);
    let before = s.clone();
    for command in [
        Command::SetPlayerShares {
            left: (p[0], 1.5),
            right: (p[1], 0.5),
        },
        Command::SetPlayerShares {
            left: (PlayerId(999_999), 1.0),
            right: (p[1], 1.0),
        },
        Command::EqualPlayerWidths,
        Command::SetCartwallDock(Dock::Left),
        Command::SetCartwallSize(Some(0.4)),
        Command::SetPanelHidden {
            panel: Panel::Cartwall,
            hidden: true,
        },
        Command::ResetLayout,
    ] {
        let err = apply(&mut s, command.clone()).unwrap_err();
        assert_eq!(err, ModelError::LayoutLocked, "{command:?}");
        assert_eq!(s, before);
    }
    apply(&mut s, Command::SetLayoutLocked(true)).unwrap();
    apply(&mut s, Command::SetLayoutLocked(false)).unwrap();
    apply(&mut s, Command::SetCartwallDock(Dock::Left)).unwrap();
}

#[test]
fn rl10_reset_gives_the_default_layout() {
    let mut s = fixture(0);
    let p = s.player_ids();
    apply(
        &mut s,
        Command::SetPlayerShares {
            left: (p[0], 1.5),
            right: (p[1], 0.5),
        },
    )
    .unwrap();
    hide(&mut s, Panel::Player(p[2]), true).unwrap();
    apply(&mut s, Command::SetCartwallDock(Dock::Right)).unwrap();
    hide(&mut s, Panel::Cartwall, true).unwrap();
    apply(&mut s, Command::ResetLayout).unwrap();
    assert_eq!(s.layout, Layout::default());
    assert!(!s.layout.locked);
}

#[test]
fn rl3_fewer_players_drop_their_shares_and_hidden_flags() {
    let mut s = fixture(0);
    let p = s.player_ids();
    apply(
        &mut s,
        Command::SetPlayerShares {
            left: (p[2], 1.5),
            right: (p[3], 0.5),
        },
    )
    .unwrap();
    hide(&mut s, Panel::Player(p[3]), true).unwrap();
    apply(&mut s, Command::SetPlayerCount(2)).unwrap();
    assert!(
        s.layout
            .player_shares
            .iter()
            .all(|(id, _)| p[..2].contains(id))
    );
    assert!(s.layout.hidden_players.is_empty());
}

#[test]
fn rl3_a_lower_max_players_drops_them_too() {
    let mut s = fixture(0);
    let p = s.player_ids();
    apply(&mut s, Command::EqualPlayerWidths).unwrap();
    hide(&mut s, Panel::Player(p[3]), true).unwrap();
    let mut config = s.config.clone();
    config.limits.max_players = 2;
    apply(&mut s, Command::UpdateConfig(Box::new(config))).unwrap();
    assert_eq!(s.player_ids(), p[..2].to_vec());
    assert_eq!(s.layout.player_shares, vec![(p[0], 1.0), (p[1], 1.0)]);
    assert!(s.layout.hidden_players.is_empty());
}

#[test]
fn rl3_new_players_are_shown_with_an_equal_slice() {
    let mut s = fixture(0);
    let p = s.player_ids();
    hide(&mut s, Panel::Player(p[0]), true).unwrap();
    apply(&mut s, Command::SetPlayerCount(6)).unwrap();
    let now = s.player_ids();
    assert_eq!(s.layout.shown_players(&now), now[1..].to_vec());
    assert!(close(s.layout.share(&now, now[5]), 1.0));
}

#[test]
fn rl3_when_no_panel_is_left_every_panel_is_shown() {
    let mut s = fixture(0);
    let p = s.player_ids();
    for panel in [
        Panel::Cartwall,
        Panel::Player(p[0]),
        Panel::Player(p[1]),
        Panel::Player(p[2]),
    ] {
        hide(&mut s, panel, true).unwrap();
    }
    apply(&mut s, Command::SetPlayerCount(3)).unwrap();
    assert!(s.layout.hidden_players.is_empty());
    assert!(!s.layout.cartwall.hidden);
}

#[test]
fn rl19_settle_keeps_the_first_of_a_repeated_player() {
    let p = [PlayerId(1), PlayerId(2)];
    let mut layout = Layout {
        player_shares: vec![(p[0], 2.0), (p[0], 5.0), (p[1], -1.0), (p[1], 3.0)],
        hidden_players: vec![p[1], p[1]],
        ..Layout::default()
    };
    let notes = settle_layout(&mut layout, &p);
    assert_eq!(layout.player_shares, vec![(p[0], 2.0), (p[1], 3.0)]);
    assert_eq!(layout.hidden_players, vec![p[1]]);
    assert_eq!(notes.len(), 3, "{notes:?}");
    assert!(
        settle_layout(&mut layout, &p).is_empty(),
        "settling twice changes nothing"
    );
}

#[test]
fn rl19_settle_fits_the_cartwall_size_to_its_dock() {
    let p = [PlayerId(1)];
    let mut layout = Layout {
        cartwall: CartwallLayout {
            dock: Dock::Left,
            size: None,
            hidden: false,
        },
        ..Layout::default()
    };
    assert_eq!(settle_layout(&mut layout, &p).len(), 1);
    assert_eq!(layout.cartwall.size, Some(DEFAULT_SIDE_SHARE));
    layout.cartwall = CartwallLayout {
        dock: Dock::Bottom,
        size: Some(5.0),
        hidden: false,
    };
    assert_eq!(settle_layout(&mut layout, &p).len(), 1);
    assert_eq!(layout.cartwall.size, Some(MAX_CARTWALL_SHARE));
}

fn restore_with(state: &AppState, layout: Layout) -> AppState {
    let parts = RestoreParts {
        config: state.config.clone(),
        library: state.library.clone(),
        playlists: state.playlists.clone(),
        cart_pages: state.cartwall.pages.clone(),
        cartwall_session: state.cartwall.session(),
        layout,
        ids: state.ids.clone(),
    };
    AppState::restore(parts, &state.sessions(|_| 0.0), "Main").0
}

#[test]
fn rl19_the_layout_survives_a_restart() {
    let mut s = fixture(0);
    let p = s.player_ids();
    apply(
        &mut s,
        Command::SetPlayerShares {
            left: (p[0], 1.5),
            right: (p[1], 0.5),
        },
    )
    .unwrap();
    hide(&mut s, Panel::Player(p[2]), true).unwrap();
    apply(&mut s, Command::SetCartwallDock(Dock::Right)).unwrap();
    apply(&mut s, Command::SetLayoutLocked(true)).unwrap();
    assert_eq!(common::roundtrip(&s).layout, s.layout);
}

#[test]
fn rl19_restore_drops_players_that_do_not_exist_and_shows_everything_hidden() {
    let s = fixture(0);
    let p = s.player_ids();
    let stranger = PlayerId(999_999);
    let layout = Layout {
        player_shares: vec![(p[0], 2.0), (stranger, 5.0)],
        hidden_players: vec![p[0], p[1], p[2], p[3], stranger],
        cartwall: CartwallLayout {
            dock: Dock::Left,
            size: None,
            hidden: true,
        },
        locked: true,
    };
    let restored = restore_with(&s, layout);
    assert_eq!(restored.layout.player_shares, vec![(p[0], 2.0)]);
    assert!(restored.layout.hidden_players.is_empty());
    assert!(!restored.layout.cartwall.hidden);
    assert_eq!(restored.layout.cartwall.size, Some(DEFAULT_SIDE_SHARE));
    assert!(restored.layout.locked, "the lock is kept");
}

#[test]
fn rl19_a_session_saved_with_more_players_drops_the_extra_ones() {
    let mut s = fixture(0);
    let p = s.player_ids();
    apply(&mut s, Command::EqualPlayerWidths).unwrap();
    hide(&mut s, Panel::Player(p[3]), true).unwrap();
    let layout = s.layout.clone();
    s.config.players.count = 3;
    let restored = restore_with(&s, layout);
    assert_eq!(restored.player_ids(), p[..3].to_vec());
    assert_eq!(
        restored.layout.player_shares,
        vec![(p[0], 1.0), (p[1], 1.0), (p[2], 1.0)]
    );
    assert!(restored.layout.hidden_players.is_empty());
}

#[test]
fn rl19_each_part_of_the_layout_loads_leniently() {
    let layout: Layout = serde_json::from_str(
        r#"{"player_shares": [[1, 2.0], ["x", 1.0], [2]],
            "hidden_players": "none",
            "cartwall": {"dock": "Top", "size": 0.4, "hidden": true},
            "locked": "yes"}"#,
    )
    .unwrap();
    assert_eq!(layout.player_shares, vec![(PlayerId(1), 2.0)]);
    assert!(layout.hidden_players.is_empty());
    assert_eq!(
        layout.cartwall,
        CartwallLayout {
            dock: Dock::Bottom,
            size: Some(0.4),
            hidden: true
        }
    );
    assert!(!layout.locked);
}

#[test]
fn rl19_a_layout_that_is_not_an_object_loads_as_the_default() {
    #[derive(serde::Deserialize)]
    struct Doc {
        #[serde(default, deserialize_with = "fp_model::layout::lenient_layout")]
        layout: Layout,
    }
    for json in [
        r#"{}"#,
        r#"{"layout": 5}"#,
        r#"{"layout": null}"#,
        r#"{"layout": {"cartwall": "left"}}"#,
    ] {
        let doc: Doc = serde_json::from_str(json).unwrap();
        assert_eq!(doc.layout, Layout::default(), "{json}");
    }
}

#[test]
fn rl19_settle_rescales_huge_or_tiny_shares_and_a_drag_still_works() {
    let mut s = fixture(0);
    let ids = s.player_ids();
    let layout = Layout {
        player_shares: vec![(ids[0], f32::MAX), (ids[1], f32::MAX)],
        ..Layout::default()
    };
    let mut settled = layout.clone();
    let notes = settle_layout(&mut settled, &ids);
    assert!(notes.iter().any(|n| n.starts_with("layout:")), "{notes:?}");
    assert!(
        settle_layout(&mut settled, &ids).is_empty(),
        "settling twice changes nothing"
    );
    s.layout = settled;
    apply(
        &mut s,
        Command::SetPlayerShares {
            left: (ids[0], 1.5),
            right: (ids[1], 0.5),
        },
    )
    .unwrap();

    let mut tiny = Layout {
        player_shares: vec![(ids[0], 1e-45), (ids[1], 3e-45)],
        ..Layout::default()
    };
    assert!(!settle_layout(&mut tiny, &ids).is_empty());
    assert!(settle_layout(&mut tiny, &ids).is_empty());
    let (a, b) = (tiny.player_shares[0].1, tiny.player_shares[1].1);
    assert!(close(a + b, 2.0) && close(b / a, 2.0), "{a} {b}");
}

#[test]
fn rl19_settle_keeps_tiny_shares_that_drags_leave() {
    // A drag to a player's minimum sends 1e-4 of the pair's room; a few in
    // a row leave a share far below 1e-6 (here about 5e-8).
    let mut s = fixture(0);
    let p = s.player_ids();
    for (left, right) in [(p[0], p[1]), (p[1], p[2]), (p[0], p[1])] {
        apply(
            &mut s,
            Command::SetPlayerShares {
                left: (left, 1e-4),
                right: (right, 1.0),
            },
        )
        .unwrap();
    }
    let tiny = s.layout.player_shares[0].1;
    assert!(tiny > 0.0 && tiny < 1e-6, "{tiny}");
    let before = fractions(&s);
    let mut settled = s.layout.clone();
    settle_layout(&mut settled, &p);
    assert_eq!(settled.player_shares.len(), 4, "{settled:?}");
    s.layout = settled.clone();
    let after = fractions(&s);
    assert!(
        before.iter().zip(&after).all(|(a, b)| (a - b).abs() < 1e-5),
        "{before:?} -> {after:?}"
    );
    assert!(
        settle_layout(&mut settled, &p).is_empty(),
        "settling twice changes nothing"
    );
}

#[test]
fn rl3_player_count_changes_while_locked() {
    let mut s = fixture(0);
    let p = s.player_ids();
    apply(
        &mut s,
        Command::SetPlayerShares {
            left: (p[2], 1.5),
            right: (p[3], 0.5),
        },
    )
    .unwrap();
    hide(&mut s, Panel::Player(p[3]), true).unwrap();
    s.layout.locked = true;
    apply(&mut s, Command::SetPlayerCount(3)).unwrap();
    assert!(s.layout.hidden_players.is_empty());
    assert!(
        s.layout
            .player_shares
            .iter()
            .all(|(id, _)| p[..3].contains(id))
    );
    apply(&mut s, Command::SetPlayerCount(5)).unwrap();
    let now = s.player_ids();
    assert!(s.layout.shown_players(&now).contains(&now[4]));
    let mut config = s.config.clone();
    config.limits.max_players = 2;
    apply(&mut s, Command::UpdateConfig(Box::new(config))).unwrap();
    let now = s.player_ids();
    assert_eq!(now, p[..2].to_vec());
    assert!(
        s.layout
            .player_shares
            .iter()
            .all(|(id, _)| now.contains(id))
    );
    assert!(s.layout.locked);
}
