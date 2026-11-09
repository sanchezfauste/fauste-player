#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! External editor spec EE12: when a track sounds.

use crate::common::{entries, fixture};
use fp_model::{
    AppState, CartPage, CueState, EntryId, Normalization, PlayingCart, TrackId, Transport,
    track_sounding,
};

fn track_of(s: &AppState, entry: EntryId) -> TrackId {
    s.playlists.entry(entry).unwrap().track
}

fn two() -> (AppState, EntryId, TrackId, TrackId) {
    let s = fixture(2);
    let e = entries(&s);
    let (a, b) = (track_of(&s, e[0]), track_of(&s, e[1]));
    (s, e[0], a, b)
}

#[test]
fn a_track_nowhere_is_not_sounding() {
    let (s, _, a, b) = two();
    assert!(!track_sounding(&s, a));
    assert!(!track_sounding(&s, b));
}

#[test]
fn a_players_current_sounds_in_any_transport() {
    for t in [Transport::Stopped, Transport::Playing, Transport::Paused] {
        let (mut s, e, a, b) = two();
        s.players[0].current = Some(e);
        s.players[0].transport = t;
        assert!(track_sounding(&s, a));
        assert!(!track_sounding(&s, b));
    }
}

#[test]
fn a_pending_start_sounds() {
    let (mut s, e, a, b) = two();
    s.players[0].pending_start = Some((e, 0.0));
    assert!(track_sounding(&s, a));
    assert!(!track_sounding(&s, b));
}

#[test]
fn a_player_cue_sounds() {
    let (mut s, e, a, b) = two();
    s.players[0].cue = Some(CueState {
        entry: e,
        paused: true,
    });
    assert!(track_sounding(&s, a));
    assert!(!track_sounding(&s, b));
}

#[test]
fn the_last_history_entry_sounds_only_while_fading() {
    let (mut s, e, a, b) = two();
    let other = entries(&s)[1];
    s.players[0].history = vec![other, e];
    assert!(!track_sounding(&s, a));
    s.players[0].fading = true;
    assert!(track_sounding(&s, a));
    assert!(!track_sounding(&s, b));
}

fn with_cart(s: &mut AppState, track: TrackId) -> fp_model::CartId {
    let page = CartPage::new(&mut s.ids, "p", 1, 1);
    s.cartwall.pages.push(page);
    let cart = s
        .cartwall
        .pages
        .last_mut()
        .unwrap()
        .carts
        .first_mut()
        .unwrap();
    cart.track = Some(track);
    cart.id
}

#[test]
fn a_playing_cart_sounds() {
    let (mut s, _, a, b) = two();
    let cart = with_cart(&mut s, a);
    s.cartwall.playing.push(PlayingCart {
        cart,
        looped: false,
        normalization: Normalization::default(),
    });
    assert!(track_sounding(&s, a));
    assert!(!track_sounding(&s, b));
}

#[test]
fn the_cart_in_cartwall_cue_sounds() {
    let (mut s, _, a, b) = two();
    let cart = with_cart(&mut s, a);
    s.cartwall.cue = Some(cart);
    assert!(track_sounding(&s, a));
    assert!(!track_sounding(&s, b));
}

#[test]
fn the_library_cue_track_sounds_in_any_transport() {
    for t in [Transport::Stopped, Transport::Playing, Transport::Paused] {
        let (mut s, _, a, b) = two();
        s.library_cue.track = Some(a);
        s.library_cue.transport = t;
        assert!(track_sounding(&s, a));
        assert!(!track_sounding(&s, b));
    }
}
