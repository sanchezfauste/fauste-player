#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Automation rework spec §4–§5: the Log and the music fill are driven by
//! the player transport; next is what the operator selected.

use fp_model::automation::ItemRef;
use fp_model::{Command, TrackId};

use crate::automation_rig::{Rig, today};

fn two_blocks() -> Rig {
    use crate::automation_rig::song;
    Rig::new(vec![
        (10, 0, vec![song(1), song(2)]),
        (11, 0, vec![song(3)]),
    ])
}

fn next(r: &Rig) -> Option<ItemRef> {
    r.s.automation.as_ref().unwrap().next
}

fn running(r: &Rig) -> bool {
    r.s.automation.as_ref().unwrap().running
}

/// Started before 10:00, song 1 of the 10:00 block on air.
fn on_air_at_ten() -> Rig {
    let mut r = two_blocks();
    r.at_hms(9, 59, 0);
    r.apply(Command::StartAutomation);
    r.at_hms(10, 0, 0);
    assert_eq!(r.playing(r.schedule_player()), Some(TrackId(1)));
    r
}

// Task 1: AR10, AR12.

#[test]
fn stopping_after_an_item_leaves_next_on_the_row_below() {
    let mut r = on_air_at_ten();
    let log = r.schedule_player();
    r.apply(Command::StopAutomationAfterCurrent);
    r.end_current(log);
    r.at_hms(10, 30, 0);
    assert!(!running(&r));
    assert_eq!(next(&r), Some(r.item(today(), 10, 0, 1)));
    assert!(r.s.automation.as_ref().unwrap().next_by_hand);
    r.at_hms(10, 45, 0);
    assert_eq!(next(&r), Some(r.item(today(), 10, 0, 1)));
}

#[test]
fn a_pinned_next_survives_a_clock_jump() {
    let mut r = on_air_at_ten();
    let log = r.schedule_player();
    r.apply(Command::StopAutomationAfterCurrent);
    r.end_current(log);
    r.jump_hms(12, 0, 0);
    assert_eq!(next(&r), Some(r.item(today(), 10, 0, 1)));
}

#[test]
fn at_start_up_next_follows_the_clock() {
    let mut r = two_blocks();
    r.at_hms(10, 30, 0);
    assert_eq!(next(&r), Some(r.item(today(), 11, 0, 0)));
    r.at_hms(10, 50, 0);
    assert_eq!(next(&r), Some(r.item(today(), 11, 0, 0)));
}

// Task 2: AR6, the Log's Stop at end.

#[test]
fn stop_at_end_on_an_item_stops_after_it() {
    let mut r = on_air_at_ten();
    let log = r.schedule_player();
    r.apply(Command::ToggleStopAfterCurrent(log));
    assert!(running(&r), "running until the item ends");
    assert!(r.s.automation.as_ref().unwrap().stop_at_end);
    assert!(r.player(log).stop_after_current);
    r.end_current(log);
    r.at_hms(10, 0, 1);
    assert!(!running(&r));
    assert!(!r.s.automation.as_ref().unwrap().stop_at_end);
    r.at_hms(10, 3, 0);
    assert_eq!(r.playing(log), None);
    assert_eq!(r.playing(r.filler_player()), None);
}

#[test]
fn stop_at_end_during_music_lets_the_song_end_then_stops() {
    let mut r = two_blocks();
    r.at_hms(10, 30, 0);
    r.apply(Command::StartAutomation);
    let f = r.filler_player();
    assert!(r.playing(f).is_some());
    r.apply(Command::ToggleStopAfterCurrent(r.schedule_player()));
    assert!(r.player(f).stop_after_current);
    r.end_current(f);
    r.at_hms(10, 31, 0);
    assert_eq!(r.playing(f), None);
    assert!(!running(&r));
    assert_eq!(next(&r), Some(r.item(today(), 11, 0, 0)));
    r.at_hms(11, 0, 0);
    assert_eq!(r.playing(r.schedule_player()), None);
}

#[test]
fn stop_at_end_during_music_moves_to_the_block_when_it_comes_first() {
    let mut r = two_blocks();
    r.at_hms(10, 58, 30);
    r.apply(Command::StartAutomation);
    let log = r.schedule_player();
    r.apply(Command::ToggleStopAfterCurrent(log));
    r.at_hms(11, 0, 0);
    assert_eq!(r.playing(log), Some(TrackId(3)));
    assert!(r.player(log).stop_after_current);
    assert!(running(&r));
    r.end_current(log);
    r.at_hms(11, 1, 0);
    assert!(!running(&r));
    assert_eq!(r.playing(r.filler_player()), None);
}

#[test]
fn unsetting_stop_at_end_lets_automation_go_on() {
    let mut r = two_blocks();
    r.at_hms(10, 58, 30);
    r.apply(Command::StartAutomation);
    let log = r.schedule_player();
    r.apply(Command::ToggleStopAfterCurrent(log));
    r.apply(Command::ToggleStopAfterCurrent(log));
    assert!(!r.player(r.filler_player()).stop_after_current);
    r.at_hms(11, 0, 0);
    assert_eq!(r.playing(log), Some(TrackId(3)));
    assert!(!r.player(log).stop_after_current);
    assert!(running(&r));
}
