#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Automation spec AU36: what aired and what was missed.

use chrono::Days;
use fp_model::as_run::{AsRunEntry, AsRunOutcome, AsRunSource};
use fp_model::{Command, Config};

use crate::automation_rig::{Rig, song, today};

fn rig() -> Rig {
    Rig::new(vec![
        (10, 0, vec![song(1), song(2)]),
        (12, 0, vec![song(3)]),
    ])
}

fn entries(r: &Rig) -> Vec<AsRunEntry> {
    r.s.as_run.entries().cloned().collect()
}

#[test]
fn a_block_item_that_starts_is_recorded_aired() {
    let mut r = rig();
    r.at_hms(9, 59, 0);
    r.apply(Command::StartAutomation);
    r.at_hms(10, 0, 0);
    let first = r.item(today(), 10, 0, 0);
    let e: Vec<AsRunEntry> = entries(&r)
        .into_iter()
        .filter(|e| e.source == AsRunSource::Schedule)
        .collect();
    assert_eq!(e.len(), 1, "{e:?}");
    assert_eq!(e[0].item, Some(first));
    assert_eq!(e[0].outcome, AsRunOutcome::Aired);
    assert_eq!(e[0].scheduled, today().and_hms_opt(10, 0, 0));
    assert_eq!(e[0].started, today().and_hms_opt(10, 0, 0));
    assert_eq!(e[0].track, Some(fp_model::TrackId(1)));
}

#[test]
fn a_filler_song_that_starts_is_recorded() {
    let mut r = rig();
    r.at_hms(8, 0, 0);
    r.apply(Command::StartAutomation);
    let e: Vec<AsRunEntry> = entries(&r)
        .into_iter()
        .filter(|e| e.source == AsRunSource::Filler)
        .collect();
    assert_eq!(e.len(), 1, "{e:?}");
    assert_eq!(e[0].scheduled, None);
    assert_eq!(e[0].started, today().and_hms_opt(8, 0, 0));
    assert_eq!(e[0].outcome, AsRunOutcome::Aired);
}

#[test]
fn a_missed_item_is_recorded_missed_once() {
    let mut r = rig();
    r.at_hms(9, 0, 0);
    r.at_hms(10, 5, 0);
    r.at_hms(10, 6, 0);
    let missed: Vec<AsRunEntry> = entries(&r)
        .into_iter()
        .filter(|e| e.outcome == AsRunOutcome::Missed)
        .collect();
    assert_eq!(missed.len(), 2, "{missed:?}");
    assert!(missed.iter().all(|e| e.started.is_none()));
    assert_eq!(missed[0].scheduled, today().and_hms_opt(10, 0, 0));
}

#[test]
fn a_missed_item_played_later_has_both_entries() {
    let mut r = rig();
    r.at_hms(9, 0, 0);
    r.at_hms(10, 5, 0);
    let first = r.item(today(), 10, 0, 0);
    r.apply(Command::SetScheduleNext(first));
    r.apply(Command::StartAutomation);
    let mine: Vec<AsRunOutcome> = entries(&r)
        .into_iter()
        .filter(|e| e.item == Some(first))
        .map(|e| e.outcome)
        .collect();
    assert_eq!(mine, vec![AsRunOutcome::Missed, AsRunOutcome::Aired]);
}

#[test]
fn old_entries_leave_the_window() {
    let mut r = rig();
    r.at_hms(9, 0, 0);
    r.at_hms(10, 5, 0);
    assert!(entries(&r).len() >= 2);
    let later = today().checked_add_days(Days::new(2)).unwrap();
    r.jump(later.and_hms_opt(8, 0, 0).unwrap());
    r.jump(later.and_hms_opt(8, 0, 1).unwrap());
    assert!(
        entries(&r)
            .iter()
            .all(|e| e.date >= later.pred_opt().unwrap()),
        "{:?}",
        entries(&r)
    );
}

#[test]
fn sequence_numbers_only_grow() {
    let mut r = rig();
    r.at_hms(9, 0, 0);
    r.at_hms(10, 5, 0);
    r.apply(Command::StartAutomation);
    let seqs: Vec<u64> = entries(&r).iter().map(|e| e.seq).collect();
    assert!(seqs.windows(2).all(|w| w[0] < w[1]), "{seqs:?}");
    assert_eq!(r.s.as_run.next_seq(), seqs.last().unwrap() + 1);
    r.s.as_run.resume_at(100);
    r.jump(today().and_hms_opt(10, 6, 0).unwrap());
    let p = r.filler_player();
    r.end_current(p);
    assert!(r.s.as_run.entries().last().unwrap().seq >= 100);
}

#[test]
fn as_run_keep_days_has_its_default_and_range() {
    let mut c = Config::default();
    assert_eq!(c.automation.as_run_keep_days, 90);
    c.automation.as_run_keep_days = 0;
    c.validate();
    assert_eq!(c.automation.as_run_keep_days, 1);
    c.automation.as_run_keep_days = 99_999;
    c.validate();
    assert_eq!(c.automation.as_run_keep_days, 3_650);
}

#[test]
fn a_pause_and_resume_is_one_airing() {
    let mut r = rig();
    r.at_hms(8, 0, 0);
    r.apply(Command::StartAutomation);
    let fill = r.filler_player();
    assert!(r.playing(fill).is_some());
    r.apply(Command::Pause(fill));
    r.at_hms(8, 0, 5);
    r.apply(Command::Play(fill));
    r.at_hms(8, 0, 10);
    r.at_hms(10, 0, 0);
    let sched = r.schedule_player();
    r.apply(Command::Pause(sched));
    r.at_hms(10, 0, 5);
    r.apply(Command::Play(sched));
    r.at_hms(10, 0, 10);
    let count = |source| entries(&r).iter().filter(|e| e.source == source).count();
    assert_eq!(count(AsRunSource::Filler), 1, "{:?}", entries(&r));
    assert_eq!(count(AsRunSource::Schedule), 1, "{:?}", entries(&r));
}
