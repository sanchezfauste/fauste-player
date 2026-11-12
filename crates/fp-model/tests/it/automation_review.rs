#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Automation plan 4, final review: hand-set next, jumps, hand-over,
//! restart of a stopped automation, idle ticks.

use crate::automation_rig::{Rig, song, today};
use fp_model::{Command, TrackId};

#[test]
fn a_hand_set_next_that_disappears_never_replays_aired_rows() {
    let mut r = Rig::new(vec![
        (10, 0, vec![song(1), song(2)]),
        (12, 0, vec![song(3)]),
    ]);
    r.at_hms(9, 59, 0);
    r.apply(Command::StartAutomation);
    r.at_hms(10, 0, 0);
    let p = r.schedule_player();
    let f = r.filler_player();
    assert_eq!(r.playing(p), Some(TrackId(1)));
    r.end_current(p);
    r.end_current(p);
    r.at_hms(10, 7, 0);
    assert!(r.playing(f).is_some(), "filler after block");
    let item = r.item(today(), 12, 0, 0);
    r.apply(Command::SetScheduleNext(item));
    let base = r.base;
    let b = r.block(12, 0);
    r.apply(Command::RemoveBlock {
        template: base,
        block: b,
    });
    r.at_hms(10, 7, 1);
    // The next set by hand is gone: next follows the clock again.
    assert!(!r.s.automation.as_ref().unwrap().next_by_hand);
    r.end_current(f);
    for s in 1..20 {
        r.at_hms(10, 8, s);
        r.end_current(p);
    }
    let starts = r
        .log
        .iter()
        .filter(|a| matches!(a, fp_model::EngineAction::StartCurrent{player, ..} if *player == p))
        .count();
    // The block's first row was started; its second followed by itself.
    assert_eq!(starts, 1, "an aired row was started again");
    assert_eq!(r.playing(p), None, "schedule replays an old row");
    assert!(r.playing(f).is_some(), "the music goes on");
}

#[test]
fn a_forward_jump_during_a_block_misses_the_skipped_blocks() {
    let mut r = Rig::new(vec![
        (10, 0, vec![song(1), song(2)]),
        (11, 0, vec![song(3)]),
        (12, 0, vec![song(4)]),
        (13, 0, vec![song(5)]),
    ]);
    r.at_hms(9, 59, 0);
    r.apply(Command::StartAutomation);
    r.at_hms(10, 0, 0);
    let p = r.schedule_player();
    r.at_hms(10, 0, 1);
    r.jump_hms(12, 30, 0);
    r.at_hms(12, 30, 1);
    r.end_current(p);
    r.end_current(p);
    r.at_hms(12, 30, 2);
    assert_eq!(r.playing(p), None, "a block skipped by the jump plays");
}

#[test]
fn clearing_a_hand_set_next_lets_the_filler_go_on() {
    let mut r = Rig::new(vec![(11, 0, vec![song(3)])]);
    r.at_hms(10, 30, 0);
    r.apply(Command::StartAutomation);
    let f = r.filler_player();
    let item = r.item(today(), 11, 0, 0);
    r.apply(Command::SetScheduleNext(item));
    r.at_hms(10, 30, 0);
    assert!(r.player(f).stop_after_current);
    r.apply(Command::ClearScheduleNext);
    r.at_hms(10, 30, 1);
    assert!(
        !r.player(f).stop_after_current,
        "filler still stops after current"
    );
}

#[test]
fn a_hand_set_block_replaced_before_it_airs_is_not_lost() {
    let mut r = Rig::new(vec![
        (10, 0, vec![song(1), song(2)]),
        (11, 0, vec![song(3)]),
        (12, 0, vec![song(4)]),
    ]);
    r.at_hms(9, 59, 0);
    r.apply(Command::StartAutomation);
    r.at_hms(10, 0, 0);
    let p = r.schedule_player();
    let i12 = r.item(today(), 12, 0, 0);
    r.apply(Command::SetScheduleNext(i12));
    let i10 = r.item(today(), 10, 0, 1);
    r.apply(Command::SetScheduleNext(i10));
    r.end_current(p);
    assert_eq!(r.playing(p), Some(TrackId(2)));
    r.end_current(p);
    r.at_hms(10, 10, 0);
    r.at_hms(11, 0, 0);
    assert_eq!(r.playing(p), Some(TrackId(3)));
    r.end_current(p);
    r.at_hms(11, 5, 0);
    let next = r.s.automation.as_ref().unwrap().next;
    assert_eq!(next, Some(i12), "12:00 block lost");
}

#[test]
fn a_backward_jump_during_the_hand_over_brings_the_music_back() {
    let mut r = Rig::new(vec![(11, 0, vec![song(3)])]);
    r.at_hms(10, 58, 0);
    r.apply(Command::StartAutomation);
    let f = r.filler_player();
    assert!(r.playing(f).is_some());
    r.at(today().and_hms_milli_opt(10, 59, 59, 0).unwrap());
    r.at_hms(9, 59, 0);
    r.at_hms(9, 59, 1);
    r.at_hms(9, 59, 2);
    assert!(
        r.playing(f).is_some(),
        "dead air after a backward jump during the hand-over"
    );
}

#[test]
fn an_idle_tick_reports_no_change() {
    let mut r = Rig::new(vec![(11, 0, vec![song(3)])]);
    r.at_hms(9, 0, 0);
    let mut changes = 0;
    for s in 1..10 {
        r.s.local_now = today().and_hms_opt(9, 0, s).unwrap();
        let (_, changed) = fp_model::automation::tick(&mut r.s);
        changes += usize::from(changed);
    }
    assert_eq!(changes, 0);
}

#[test]
fn a_block_played_by_hand_keeps_its_time_critical_items() {
    let mut r = Rig::new(vec![(
        10,
        0,
        vec![crate::automation_rig::hourly_id(10), song(1)],
    )]);
    let mut t = fp_model::Track::new(TrackId(10), std::path::PathBuf::from("/m/id.wav"));
    t.duration_secs = 10.0;
    r.s.library.insert(t);
    r.at_hms(11, 0, 0);
    let item = r.item(today(), 10, 0, 0);
    r.apply(Command::SetScheduleNext(item));
    r.apply(Command::StartAutomation);
    assert_eq!(
        r.playing(r.schedule_player()),
        Some(TrackId(10)),
        "AU18: hand-set past block skips its time-critical item"
    );
}

#[test]
fn start_after_stop_after_current_keeps_the_block() {
    let mut r = Rig::new(vec![(10, 0, vec![song(1), song(2)])]);
    r.at_hms(9, 59, 0);
    r.apply(Command::StartAutomation);
    r.at_hms(10, 0, 0);
    let p = r.schedule_player();
    r.apply(Command::StopAutomationAfterCurrent);
    r.apply(Command::StartAutomation);
    r.at_hms(10, 1, 0);
    r.end_current(p);
    r.at_hms(10, 3, 0);
    assert_eq!(
        r.playing(p),
        Some(TrackId(2)),
        "restarted automation drops the rest of the block"
    );
}

#[test]
fn removing_the_coming_block_during_the_hand_over_brings_the_music_back() {
    let mut r = Rig::new(vec![(11, 0, vec![song(3)]), (14, 0, vec![song(4)])]);
    r.at_hms(10, 58, 0);
    r.apply(Command::StartAutomation);
    let f = r.filler_player();
    r.at(today().and_hms_milli_opt(10, 59, 59, 0).unwrap());
    let base = r.base;
    let b = r.block(11, 0);
    r.apply(Command::RemoveBlock {
        template: base,
        block: b,
    });
    r.at_hms(11, 0, 0);
    assert!(
        r.playing(f).is_some(),
        "dead air after the coming block was removed"
    );
}
