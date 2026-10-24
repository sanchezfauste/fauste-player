#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Rule 29: `Command::CreatePlaylistIn` creates a playlist and shows it in
//! one player at once.

use crate::common;

use common::{fixture, p0};
use fp_model::{Command, ModelError, PlayerId, apply};

#[test]
fn it_creates_the_playlist_and_shows_it_in_that_player() {
    let mut s = fixture(2);
    let p = s.players[1].id;
    let before = s.playlists.len();
    apply(
        &mut s,
        Command::CreatePlaylistIn {
            name: "Late".into(),
            player: p,
        },
    )
    .unwrap();
    assert_eq!(s.playlists.len(), before + 1);
    let created = s.playlists.iter().find(|pl| pl.name == "Late").unwrap();
    assert!(created.entries.is_empty());
    assert_eq!(s.player(p).unwrap().playlist, created.id);
}

#[test]
fn the_other_players_keep_their_playlist() {
    let mut s = fixture(2);
    let main = s.playlists.first_id().unwrap();
    let p = s.players[1].id;
    apply(
        &mut s,
        Command::CreatePlaylistIn {
            name: "Late".into(),
            player: p,
        },
    )
    .unwrap();
    for other in s.players.iter().filter(|o| o.id != p) {
        assert_eq!(other.playlist, main);
    }
}

#[test]
fn an_unknown_player_is_refused_and_nothing_is_created() {
    let mut s = fixture(2);
    let before = s.playlists.len();
    let missing = PlayerId(999);
    let err = apply(
        &mut s,
        Command::CreatePlaylistIn {
            name: "Late".into(),
            player: missing,
        },
    )
    .unwrap_err();
    assert_eq!(err, ModelError::UnknownPlayer(missing));
    assert_eq!(s.playlists.len(), before);
}

#[test]
fn a_player_on_air_keeps_playing_its_entry() {
    let mut s = fixture(2);
    let p = p0(&s);
    apply(&mut s, Command::Play(p)).unwrap();
    let current = s.player(p).unwrap().current;
    apply(
        &mut s,
        Command::CreatePlaylistIn {
            name: "Late".into(),
            player: p,
        },
    )
    .unwrap();
    assert_eq!(s.player(p).unwrap().current, current);
}
