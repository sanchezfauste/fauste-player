#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Remote control spec §2: on/off settings set to a value. Unlike the
//! toggles, sending the same value twice leaves it in that state, even when
//! both were decided from the same snapshot.

mod common;

use std::path::PathBuf;

use common::{entries, fixture, p0};
use fp_model::{Command, PlayMode, apply};

#[test]
fn set_cue_twice_leaves_the_cue_on() {
    let mut s = fixture(3);
    let p = p0(&s);
    apply(&mut s, Command::SetCue(p, true)).unwrap();
    apply(&mut s, Command::SetCue(p, true)).unwrap();
    assert!(s.players[0].cue.is_some());
    apply(&mut s, Command::SetCue(p, false)).unwrap();
    apply(&mut s, Command::SetCue(p, false)).unwrap();
    assert!(s.players[0].cue.is_none());
}

#[test]
fn set_stop_after_current_twice_leaves_it_on() {
    let mut s = fixture(3);
    let p = p0(&s);
    apply(&mut s, Command::SetStopAfterCurrent(p, true)).unwrap();
    apply(&mut s, Command::SetStopAfterCurrent(p, true)).unwrap();
    assert!(s.players[0].stop_after_current);
    apply(&mut s, Command::SetStopAfterCurrent(p, false)).unwrap();
    assert!(!s.players[0].stop_after_current);
}

#[test]
fn set_stop_after_current_off_is_harmless_in_single_mode() {
    let mut s = fixture(3);
    let p = p0(&s);
    apply(&mut s, Command::SetMode(p, PlayMode::Single)).unwrap();
    assert!(apply(&mut s, Command::SetStopAfterCurrent(p, false)).is_ok());
    assert!(apply(&mut s, Command::SetStopAfterCurrent(p, true)).is_err());
}

#[test]
fn set_entry_flags_twice_leaves_them_on() {
    let mut s = fixture(3);
    let e = entries(&s)[1];
    for _ in 0..2 {
        apply(&mut s, Command::SetEntryRepeat(e, true)).unwrap();
        apply(&mut s, Command::SetEntryStopAfter(e, true)).unwrap();
    }
    let entry = s.playlists.entry(e).unwrap();
    assert!(entry.repeat && entry.stop_after);
    apply(&mut s, Command::SetEntryRepeat(e, false)).unwrap();
    assert!(!s.playlists.entry(e).unwrap().repeat);
}

#[test]
fn set_cart_cue_twice_leaves_the_cart_cueing() {
    let mut s = fixture(1);
    let page = s.cartwall.pages[0].id;
    apply(
        &mut s,
        Command::AssignCartFile {
            page,
            index: 0,
            path: PathBuf::from("/m/jingle.flac"),
        },
    )
    .unwrap();
    let cart = s.cartwall.pages[0].carts[0].id;
    apply(&mut s, Command::SetCartCue(cart, true)).unwrap();
    apply(&mut s, Command::SetCartCue(cart, true)).unwrap();
    assert_eq!(s.cartwall.cue, Some(cart));
    apply(&mut s, Command::SetCartCue(cart, false)).unwrap();
    apply(&mut s, Command::SetCartCue(cart, false)).unwrap();
    assert_eq!(s.cartwall.cue, None);
}
