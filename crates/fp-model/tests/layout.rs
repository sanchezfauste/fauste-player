#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Resizable layout spec RL1–RL10, RL3 and the model side of RL19: one
//! test per rule, named after it.

mod common;

use common::fixture;
use fp_model::{
    AppState, CartwallLayout, Command, Config, Dock, Layout, ModelError, Panel, PlayerId, apply,
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
