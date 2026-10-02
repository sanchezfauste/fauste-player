#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Feedback 2 spec O8: the player header names the entry's own repeat and
//! stop-after.

mod support;

use egui_kittest::kittest::Queryable;
use fp_model::{Command, apply};
use support::{harness, state};

const REPEAT: &str = "This track will repeat";
const STOPS: &str = "Stops after this track";

fn playing_with(flag: impl Fn(fp_model::EntryId) -> Option<Command>) -> fp_model::AppState {
    let mut s = state(1, 3);
    let p = s.players[0].id;
    apply(&mut s, Command::Play(p)).unwrap();
    let e = s.players[0].current.unwrap();
    if let Some(c) = flag(e) {
        apply(&mut s, c).unwrap();
    }
    s
}

#[test]
fn a_repeating_entry_shows_the_repeat_notice() {
    let (h, _) = harness(playing_with(|e| Some(Command::ToggleEntryRepeat(e))));
    assert!(h.query_by_label(REPEAT).is_some());
    assert!(h.query_by_label(STOPS).is_none());
}

#[test]
fn a_stop_after_entry_shows_the_stop_notice() {
    let (h, _) = harness(playing_with(|e| Some(Command::ToggleEntryStopAfter(e))));
    assert!(h.query_by_label(STOPS).is_some());
    assert!(h.query_by_label(REPEAT).is_none());
}

#[test]
fn no_flag_no_notice_and_stop_after_current_hides_it() {
    let (h, _) = harness(playing_with(|_| None));
    assert!(h.query_by_label(REPEAT).is_none() && h.query_by_label(STOPS).is_none());

    let mut s = playing_with(|e| Some(Command::ToggleEntryRepeat(e)));
    let p = s.players[0].id;
    apply(&mut s, Command::ToggleStopAfterCurrent(p)).unwrap();
    let (h, _) = harness(s);
    assert!(h.query_by_label(REPEAT).is_none());
}
