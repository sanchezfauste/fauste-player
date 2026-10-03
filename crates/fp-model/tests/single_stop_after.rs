#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback 2 spec O38: stop after current in Single mode, while the current
//! entry repeats.

mod common;

use common::{entries, fixture, p0};
use fp_model::{
    AppState, Command, EngineEvent, EntryId, ModelError, PlayMode, PlayerId, TransitionPlan,
    Transport, apply, availability, command_available, on_event, plan_for,
};

fn single_repeating() -> (AppState, PlayerId, [EntryId; 2]) {
    let mut s = fixture(2);
    let (e, p) = (entries(&s), p0(&s));
    apply(&mut s, Command::SetMode(p, PlayMode::Single)).unwrap();
    apply(&mut s, Command::ToggleEntryRepeat(e[0])).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    (s, p, [e[0], e[1]])
}

fn plan(s: &AppState, p: PlayerId) -> Option<TransitionPlan> {
    plan_for(s, s.player(p).unwrap())
}

#[test]
fn o38_the_control_is_available_in_single_mode_while_the_entry_repeats() {
    let (s, p, _) = single_repeating();
    assert!(availability(&s, p).stop_after_current);
    assert!(command_available(&s, &Command::ToggleStopAfterCurrent(p)));
}

#[test]
fn o38_it_is_refused_in_single_mode_without_a_repeating_entry() {
    let mut s = fixture(2);
    let p = p0(&s);
    apply(&mut s, Command::SetMode(p, PlayMode::Single)).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    assert!(!availability(&s, p).stop_after_current);
    assert_eq!(
        apply(&mut s, Command::ToggleStopAfterCurrent(p)),
        Err(ModelError::StopAfterInSingle)
    );
}

#[test]
fn o38_it_ends_the_repeat_at_the_cue_out_of_the_pass_that_is_playing() {
    let (mut s, p, [a, _]) = single_repeating();
    assert!(
        matches!(plan(&s, p), Some(TransitionPlan::StartNextAt { .. })),
        "repeating"
    );
    apply(&mut s, Command::ToggleStopAfterCurrent(p)).unwrap();
    assert_eq!(plan(&s, p), Some(TransitionPlan::StopAt { at_secs: 180.0 }));
    on_event(
        &mut s,
        EngineEvent::ReachedEnd {
            player: p,
            entry: a,
        },
    );
    let player = s.player(p).unwrap();
    assert_eq!(player.transport, Transport::Stopped);
    assert!(!player.stop_after_current, "the flag ends with the stop");
}

#[test]
fn o38_the_notice_is_the_same_as_in_continuous_mode() {
    let (mut s, p, _) = single_repeating();
    assert_eq!(
        fp_model::entry_notice(&s, p),
        Some(fp_model::EntryNotice::Repeats)
    );
    apply(&mut s, Command::ToggleStopAfterCurrent(p)).unwrap();
    assert_eq!(
        fp_model::entry_notice(&s, p),
        None,
        "the stop-after badge says it"
    );
    assert!(s.player(p).unwrap().stop_after_current);
}

#[test]
fn o38_switching_to_single_keeps_the_flag_while_the_entry_repeats() {
    let mut s = fixture(2);
    let (e, p) = (entries(&s), p0(&s));
    apply(&mut s, Command::ToggleEntryRepeat(e[0])).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    apply(&mut s, Command::ToggleStopAfterCurrent(p)).unwrap();
    apply(&mut s, Command::SetMode(p, PlayMode::Single)).unwrap();
    assert!(s.player(p).unwrap().stop_after_current);
}

#[test]
fn switching_to_single_clears_the_flag_without_a_repeating_entry() {
    let mut s = fixture(2);
    let p = p0(&s);
    apply(&mut s, Command::Play(p)).unwrap();
    apply(&mut s, Command::ToggleStopAfterCurrent(p)).unwrap();
    apply(&mut s, Command::SetMode(p, PlayMode::Single)).unwrap();
    assert!(!s.player(p).unwrap().stop_after_current);
}

#[test]
fn the_single_mode_flag_is_dropped_when_the_repeat_stops_applying() {
    let (mut s, p, [a, _]) = single_repeating();
    apply(&mut s, Command::ToggleStopAfterCurrent(p)).unwrap();
    apply(&mut s, Command::ToggleEntryRepeat(a)).unwrap();
    assert!(!s.player(p).unwrap().stop_after_current, "repeat off");
    apply(&mut s, Command::ToggleEntryRepeat(a)).unwrap();
    apply(&mut s, Command::ToggleStopAfterCurrent(p)).unwrap();
    apply(&mut s, Command::ToggleEntryStopAfter(a)).unwrap();
    assert!(
        !s.player(p).unwrap().stop_after_current,
        "entry stop-after wins"
    );
}

#[test]
fn the_flag_ends_with_stop_and_does_not_reach_the_next_play() {
    let (mut s, p, _) = single_repeating();
    apply(&mut s, Command::ToggleStopAfterCurrent(p)).unwrap();
    apply(&mut s, Command::Stop(p)).unwrap();
    assert!(!s.player(p).unwrap().stop_after_current);
}

#[test]
fn next_into_a_plain_entry_drops_the_single_mode_flag() {
    let (mut s, p, _) = single_repeating();
    apply(&mut s, Command::ToggleStopAfterCurrent(p)).unwrap();
    apply(&mut s, Command::Play(p)).unwrap(); // Next: the repeating entry is left
    assert!(!s.player(p).unwrap().stop_after_current);
}

#[test]
fn set_stop_after_current_follows_the_same_guard() {
    let (mut s, p, _) = single_repeating();
    apply(&mut s, Command::SetStopAfterCurrent(p, true)).unwrap();
    assert!(s.player(p).unwrap().stop_after_current);
}

#[test]
fn restore_keeps_the_single_mode_flag_only_for_a_repeating_entry() {
    let (mut s, p, _) = single_repeating();
    apply(&mut s, Command::ToggleStopAfterCurrent(p)).unwrap();
    let restored = common::roundtrip(&s);
    let player = restored.player(p).unwrap();
    assert!(player.stop_after_current);
    assert_eq!(
        player.transport,
        Transport::Paused,
        "nothing goes on air by itself"
    );
    // Same state, but the repeat mark was removed:
    let mut s2 = s.clone();
    let a = s2.player(p).unwrap().current.unwrap();
    s2.playlists.entry_mut(a).unwrap().repeat = false;
    assert!(!common::roundtrip(&s2).player(p).unwrap().stop_after_current);
}
