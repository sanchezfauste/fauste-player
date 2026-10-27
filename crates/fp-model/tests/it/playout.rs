#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Mobile spec MB1: while playout is active the app keeps its process
//! alive (the foreground service on Android).

use fp_model::{AppState, Config, CueState, PlayingCart, Transport, playout_active};

fn state() -> AppState {
    AppState::new(Config::default(), "Main")
}

#[test]
fn a_new_state_is_not_playout_active() {
    assert!(!playout_active(&state()));
}

#[test]
fn stopped_player_with_a_next_track_keeps_playout_active() {
    let mut s = state();
    let entry = s.ids.entry();
    s.players[0].next = Some(entry);
    assert_eq!(s.players[0].transport, Transport::Stopped);
    assert!(playout_active(&s));
}

#[test]
fn playing_or_paused_player_is_playout_active() {
    for transport in [Transport::Playing, Transport::Paused] {
        let mut s = state();
        s.players[0].transport = transport;
        assert!(playout_active(&s), "{transport:?}");
    }
}

#[test]
fn a_playing_cart_is_playout_active() {
    let mut s = state();
    let cart = s.cartwall.pages[0].carts[0].id;
    s.cartwall.playing.push(PlayingCart {
        cart,
        looped: false,
        normalization: fp_model::Normalization::default(),
    });
    assert!(playout_active(&s));
}

#[test]
fn a_player_cue_is_playout_active() {
    let mut s = state();
    let entry = s.ids.entry();
    s.players[0].cue = Some(CueState {
        entry,
        paused: false,
    });
    assert!(playout_active(&s));
}

#[test]
fn a_cart_cue_is_playout_active() {
    let mut s = state();
    s.cartwall.cue = Some(s.cartwall.pages[0].carts[0].id);
    assert!(playout_active(&s));
}

#[test]
fn a_player_with_nothing_loaded_and_nothing_sounding_is_not() {
    let mut s = state();
    s.players[0].current = None;
    s.players[0].next = None;
    s.players[0].transport = Transport::Stopped;
    assert!(s.cartwall.playing.is_empty());
    assert!(!playout_active(&s));
}
