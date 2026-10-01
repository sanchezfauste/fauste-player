#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback 2 spec O6: what the close guard treats as on air.

mod common;

use common::{fixture, p0};
use fp_model::{Command, OnAir, PlayingCart, apply, on_air};

#[test]
fn a_stopped_state_has_nothing_on_air() {
    let s = fixture(3);
    assert!(on_air(&s).is_empty());
}

#[test]
fn playing_and_paused_players_are_on_air_with_their_entry() {
    let mut s = fixture(3);
    let p = p0(&s);
    apply(&mut s, Command::Play(p)).unwrap();
    let entry = s.players[0].current;
    assert!(entry.is_some());
    assert_eq!(on_air(&s), vec![OnAir::Player { player: p, entry }]);
    apply(&mut s, Command::Pause(p)).unwrap();
    assert_eq!(on_air(&s), vec![OnAir::Player { player: p, entry }]);
}

#[test]
fn playing_carts_are_on_air_after_the_players() {
    let mut s = fixture(3);
    let p = p0(&s);
    let cart = s.cartwall.pages[0].carts[0].id;
    s.cartwall.playing.push(PlayingCart {
        cart,
        looped: false,
    });
    apply(&mut s, Command::Play(p)).unwrap();
    let entry = s.players[0].current;
    assert_eq!(
        on_air(&s),
        vec![OnAir::Player { player: p, entry }, OnAir::Cart(cart)]
    );
}

#[test]
fn a_cue_alone_is_not_on_air() {
    let mut s = fixture(3);
    let p = p0(&s);
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    assert!(s.players[0].cue.is_some());
    let cart = s.cartwall.pages[0].carts[0].id;
    s.cartwall.cue = Some(cart);
    assert!(on_air(&s).is_empty());
}
