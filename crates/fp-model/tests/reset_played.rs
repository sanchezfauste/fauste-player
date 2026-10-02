#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback 2 spec O22: `Command::ResetPlayed`.

mod common;

use common::{entries, fixture, p0};
use fp_model::{
    AppState, Command, ModelError, PlayerId, PlaylistId, Transport, apply, can_reset_played,
    resettable_entries,
};

/// Plays and stops the first `n` entries on player 0 (each is marked
/// played by it), leaving the player stopped with entry `n` as its next.
fn play_through(s: &mut AppState, n: usize) {
    let (e, p) = (entries(s), p0(s));
    for entry in e.iter().take(n) {
        apply(s, Command::SetNext(p, *entry)).unwrap();
        apply(s, Command::Play(p)).unwrap();
        apply(s, Command::Stop(p)).unwrap();
    }
}

fn marked(s: &AppState) -> Vec<bool> {
    let playlist = s.playlists.first_id().unwrap();
    s.playlists
        .get(playlist)
        .unwrap()
        .entries
        .iter()
        .map(|e| !e.played_by.is_empty())
        .collect()
}

fn playlist(s: &AppState) -> PlaylistId {
    s.playlists.first_id().unwrap()
}

fn reset(s: &mut AppState) {
    let list = playlist(s);
    apply(s, Command::ResetPlayed(list)).unwrap();
}

#[test]
fn every_mark_of_the_playlist_is_cleared() {
    let mut s = fixture(5);
    play_through(&mut s, 3);
    assert_eq!(marked(&s), [true, true, true, false, false]);
    reset(&mut s);
    assert_eq!(marked(&s), [false; 5]);
}

#[test]
fn the_marks_of_every_player_are_cleared() {
    let mut s = fixture(4);
    let (e, p1) = (entries(&s), s.players[1].id);
    play_through(&mut s, 1);
    apply(&mut s, Command::SetNext(p1, e[1])).unwrap();
    apply(&mut s, Command::Play(p1)).unwrap();
    apply(&mut s, Command::Stop(p1)).unwrap();
    assert_eq!(marked(&s), [true, true, false, false]);
    reset(&mut s);
    assert_eq!(marked(&s), [false; 4]);
}

#[test]
fn the_current_entry_is_left_as_it_is_while_it_plays() {
    let mut s = fixture(4);
    let (e, p) = (entries(&s), p0(&s));
    play_through(&mut s, 2);
    // Entry 0 is also marked by player 1, which plays it now.
    let p1 = s.players[1].id;
    apply(&mut s, Command::SetNext(p1, e[0])).unwrap();
    apply(&mut s, Command::Play(p1)).unwrap();
    reset(&mut s);
    assert_eq!(marked(&s), [true, false, false, false]);
    assert_eq!(s.player(p1).unwrap().current, Some(e[0]));
    assert_eq!(s.player(p1).unwrap().transport, Transport::Playing);
    assert_eq!(s.player(p).unwrap().transport, Transport::Stopped);
}

#[test]
fn a_current_entry_is_marked_when_it_is_left_as_usual() {
    let mut s = fixture(3);
    let (e, p) = (entries(&s), p0(&s));
    apply(&mut s, Command::SetNext(p, e[0])).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    reset(&mut s);
    apply(&mut s, Command::Stop(p)).unwrap();
    assert_eq!(marked(&s), [true, false, false]);
}

#[test]
fn another_playlist_keeps_its_marks() {
    let mut s = fixture(3);
    let main = playlist(&s);
    apply(
        &mut s,
        Command::CreatePlaylistFromPaths {
            name: "Other".into(),
            paths: vec!["/music/other.flac".into()],
        },
    )
    .unwrap();
    let other = s.playlists.iter().nth(1).unwrap().id;
    let other_entry = s.playlists.get(other).unwrap().entries[0].id;
    s.playlists.mark_played(other_entry, p0(&s));
    play_through(&mut s, 2);
    apply(&mut s, Command::ResetPlayed(main)).unwrap();
    assert_eq!(marked(&s), [false; 3]);
    assert!(s.playlists.entry(other_entry).unwrap().is_played_by(p0(&s)));
}

#[test]
fn an_explicit_next_and_a_derived_next_both_survive() {
    let mut s = fixture(5);
    let (e, p) = (entries(&s), p0(&s));
    play_through(&mut s, 2);
    assert_eq!(s.player(p).unwrap().next, Some(e[2]), "derived");
    reset(&mut s);
    assert_eq!(s.player(p).unwrap().next, Some(e[2]));
    assert!(!s.player(p).unwrap().next_explicit);
    apply(&mut s, Command::SetNext(p, e[4])).unwrap();
    reset(&mut s);
    assert_eq!(s.player(p).unwrap().next, Some(e[4]));
    assert!(s.player(p).unwrap().next_explicit);
}

#[test]
fn a_derived_next_after_the_current_is_recomputed() {
    let mut s = fixture(4);
    let (e, p) = (entries(&s), p0(&s));
    apply(&mut s, Command::SetNext(p, e[1])).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(s.player(p).unwrap().next, Some(e[2]));
    reset(&mut s);
    assert_eq!(s.player(p).unwrap().next, Some(e[2]));
}

#[test]
fn flags_and_history_are_not_touched() {
    let mut s = fixture(3);
    let (e, p) = (entries(&s), p0(&s));
    apply(&mut s, Command::ToggleEntryRepeat(e[0])).unwrap();
    apply(&mut s, Command::ToggleEntryStopAfter(e[1])).unwrap();
    play_through(&mut s, 2);
    let history = s.player(p).unwrap().history.clone();
    assert!(!history.is_empty());
    reset(&mut s);
    assert!(s.playlists.entry(e[0]).unwrap().repeat);
    assert!(s.playlists.entry(e[1]).unwrap().stop_after);
    assert_eq!(s.player(p).unwrap().history, history);
}

#[test]
fn an_unknown_playlist_is_refused_and_changes_nothing() {
    let mut s = fixture(3);
    play_through(&mut s, 2);
    let before = marked(&s);
    let err = apply(&mut s, Command::ResetPlayed(PlaylistId(999_999))).unwrap_err();
    assert_eq!(err, ModelError::UnknownPlaylist(PlaylistId(999_999)));
    assert_eq!(marked(&s), before);
}

#[test]
fn resetting_twice_or_an_untouched_list_is_harmless() {
    let mut s = fixture(3);
    reset(&mut s);
    play_through(&mut s, 1);
    reset(&mut s);
    reset(&mut s);
    assert_eq!(marked(&s), [false; 3]);
}

#[test]
fn the_button_has_something_to_do_only_when_a_mark_can_be_cleared() {
    let mut s = fixture(3);
    let list = playlist(&s);
    assert!(!can_reset_played(&s, list));
    play_through(&mut s, 1);
    assert!(can_reset_played(&s, list));
    assert_eq!(resettable_entries(&s, list), vec![entries(&s)[0]]);
    apply(&mut s, Command::ResetPlayed(list)).unwrap();
    assert!(!can_reset_played(&s, list));
    assert!(!can_reset_played(&s, PlaylistId(999_999)));
}

#[test]
fn a_mark_on_the_current_entry_alone_does_not_enable_the_button() {
    let mut s = fixture(2);
    let (e, p) = (entries(&s), p0(&s));
    play_through(&mut s, 1);
    let p1: PlayerId = s.players[1].id;
    apply(&mut s, Command::SetNext(p1, e[0])).unwrap();
    apply(&mut s, Command::Play(p1)).unwrap();
    assert_eq!(s.player(p).unwrap().current, None);
    assert!(!can_reset_played(&s, playlist(&s)));
}
