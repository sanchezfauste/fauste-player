#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Resizable layout spec RL1–RL10, RL3 and the model side of RL19: one
//! test per rule, named after it.

mod common;

use common::fixture;
use fp_model::{AppState, CartwallLayout, Config, Dock, Layout, Panel, PlayerId};

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
