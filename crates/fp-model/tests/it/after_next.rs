#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Phone layout spec PL10b, PL19: where an entry goes "after the next" of
//! a player.

use crate::common;

use common::{entries, fixture, p0};
use fp_model::{Command, PlayerId, after_next, apply};

#[test]
fn right_after_the_next_in_the_shown_playlist() {
    let mut s = fixture(4);
    let (e, p) = (entries(&s), p0(&s));
    apply(&mut s, Command::SetNext(p, e[1])).unwrap();
    let shown = s.players[0].playlist;
    assert_eq!(after_next(&s, p), Some((shown, 2)));
}

#[test]
fn at_the_end_without_a_next() {
    let mut s = fixture(0);
    let p = p0(&s);
    let shown = s.players[0].playlist;
    assert_eq!(after_next(&s, p), Some((shown, 0)));
    s = fixture(3);
    s.players[0].next = None;
    assert_eq!(after_next(&s, p), Some((shown, 3)));
}

/// A next in another playlist than the one the player shows: the end of
/// the shown one.
#[test]
fn at_the_end_when_the_next_is_elsewhere() {
    let mut s = fixture(3);
    let p = p0(&s);
    apply(
        &mut s,
        Command::CreatePlaylistIn {
            name: "Other".into(),
            player: p,
        },
    )
    .unwrap();
    let other = s.players[0].playlist;
    assert_ne!(Some(other), s.playlists.first_id());
    assert!(s.players[0].next.is_some(), "the next stays in Main");
    assert_eq!(after_next(&s, p), Some((other, 0)));
}

#[test]
fn none_for_an_unknown_player() {
    let s = fixture(1);
    assert_eq!(after_next(&s, PlayerId(99)), None);
}
