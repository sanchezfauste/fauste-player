#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback 2 spec O8: which notice the player header shows for the
//! current entry's own repeat and stop-after flags.

mod common;

use common::{fixture, p0};
use fp_model::{AppState, Command, EntryNotice, FileState, entry_notice};

/// Player 0 is playing its first entry.
fn playing() -> (AppState, fp_model::PlayerId, fp_model::EntryId) {
    let mut s = fixture(3);
    let p = p0(&s);
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    let current = s.players[0].current.unwrap();
    (s, p, current)
}

fn flag(s: &mut AppState, command: Command) {
    fp_model::apply(s, command).unwrap();
}

#[test]
fn an_entry_without_flags_shows_no_notice() {
    let (s, p, _) = playing();
    assert_eq!(entry_notice(&s, p), None);
}

#[test]
fn a_repeating_current_entry_says_it_repeats() {
    let (mut s, p, e) = playing();
    flag(&mut s, Command::ToggleEntryRepeat(e));
    assert_eq!(entry_notice(&s, p), Some(EntryNotice::Repeats));
}

#[test]
fn a_stop_after_current_entry_says_it_stops_after() {
    let (mut s, p, e) = playing();
    flag(&mut s, Command::ToggleEntryStopAfter(e));
    assert_eq!(entry_notice(&s, p), Some(EntryNotice::StopsAfter));
}

#[test]
fn stop_after_wins_over_repeat_on_the_entry() {
    let (mut s, p, e) = playing();
    flag(&mut s, Command::ToggleEntryRepeat(e));
    flag(&mut s, Command::ToggleEntryStopAfter(e));
    assert_eq!(entry_notice(&s, p), Some(EntryNotice::StopsAfter));
}

#[test]
fn stop_after_current_takes_precedence_over_both() {
    let (mut s, p, e) = playing();
    flag(&mut s, Command::ToggleEntryRepeat(e));
    flag(&mut s, Command::ToggleStopAfterCurrent(p));
    assert_eq!(entry_notice(&s, p), None, "its own badge says it");
    flag(&mut s, Command::ToggleEntryStopAfter(e));
    assert_eq!(entry_notice(&s, p), None);
}

#[test]
fn a_stopped_player_shows_none_even_if_its_entries_are_flagged() {
    let mut s = fixture(3);
    let p = p0(&s);
    let first = s.players[0].next.unwrap();
    flag(&mut s, Command::ToggleEntryRepeat(first));
    assert_eq!(entry_notice(&s, p), None);
}

#[test]
fn a_flag_on_the_next_entry_is_not_shown_for_the_current_one() {
    let (mut s, p, _) = playing();
    let next = s.players[0].next.unwrap();
    flag(&mut s, Command::ToggleEntryRepeat(next));
    assert_eq!(entry_notice(&s, p), None);
}

#[test]
fn a_running_fade_stop_hides_the_repeat_notice() {
    let (mut s, p, e) = playing();
    flag(&mut s, Command::ToggleEntryRepeat(e));
    flag(&mut s, Command::FadeStop(p));
    assert_eq!(entry_notice(&s, p), None);
}

#[test]
fn an_unreadable_file_does_not_repeat() {
    let (mut s, p, e) = playing();
    flag(&mut s, Command::ToggleEntryRepeat(e));
    let track = s.playlists.entry(e).unwrap().track;
    flag(
        &mut s,
        Command::SetFileState {
            track,
            state: FileState::Unreadable,
        },
    );
    assert_eq!(entry_notice(&s, p), None);
}
