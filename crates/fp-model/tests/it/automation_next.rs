#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Automation spec AU16, AU23: next from the clock, or set by hand.

use chrono::{NaiveDate, NaiveDateTime};
use fp_model::automation::{ItemRef, clock_next, settle_next};
use fp_model::local_time::TimeOfDay;
use fp_model::schedule::{Block, Item, ensure_default};
use fp_model::{AppState, BlockId, Command, Config, ModelError, TemplateId, TrackId, apply};

fn today() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 11, 17).unwrap()
}

fn at(h: u32, m: u32) -> TimeOfDay {
    TimeOfDay::hms(h, m, 0).unwrap()
}

fn when(d: NaiveDate, h: u32, m: u32) -> NaiveDateTime {
    d.and_hms_opt(h, m, 0).unwrap()
}

fn song(n: u64) -> Item {
    Item::Track {
        track: TrackId(n),
        time_critical: false,
    }
}

/// Base: 10:00 (two songs) and 11:00 (one song), every day.
fn state() -> (AppState, BlockId, BlockId) {
    let mut s = AppState::new(Config::default(), "P");
    ensure_default(&mut s, "Base");
    fp_model::automation::ensure(&mut s, "Schedule");
    let base: TemplateId = s.schedule.templates()[0].id;
    let mut ids = Vec::new();
    for (h, items) in [(10, vec![song(1), song(2)]), (11, vec![song(3)])] {
        apply(
            &mut s,
            Command::AddBlock {
                template: base,
                start: at(h, 0),
            },
        )
        .unwrap();
        let id = s
            .schedule
            .template(base)
            .unwrap()
            .blocks
            .iter()
            .find(|b| b.start == at(h, 0))
            .unwrap()
            .id;
        apply(
            &mut s,
            Command::UpdateBlock {
                template: base,
                block: Block {
                    id,
                    start: at(h, 0),
                    items,
                },
            },
        )
        .unwrap();
        ids.push(id);
    }
    (s, ids[0], ids[1])
}

fn item(date: NaiveDate, block: BlockId, index: usize) -> ItemRef {
    ItemRef { date, block, index }
}

#[test]
fn next_is_the_coming_block() {
    let (mut s, _, eleven) = state();
    s.local_now = when(today(), 10, 58);
    assert_eq!(clock_next(&s), Some(item(today(), eleven, 0)));
}

#[test]
fn a_block_just_started_is_still_next() {
    let (mut s, ten, eleven) = state();
    s.config.automation.late_tolerance_secs = 120;
    s.local_now = when(today(), 11, 1);
    assert_eq!(clock_next(&s), Some(item(today(), eleven, 0)));
    s.local_now = when(today(), 11, 3);
    let tomorrow = today().succ_opt().unwrap();
    assert_eq!(clock_next(&s), Some(item(tomorrow, ten, 0)));
}

#[test]
fn blocks_passed_while_stopped_are_missed() {
    let (mut s, ten, eleven) = state();
    s.local_now = when(today(), 9, 0);
    settle_next(&mut s);
    assert!(s.automation.as_ref().unwrap().missed.is_empty());
    s.local_now = when(today(), 11, 30);
    settle_next(&mut s);
    let missed = &s.automation.as_ref().unwrap().missed;
    for i in [
        item(today(), ten, 0),
        item(today(), ten, 1),
        item(today(), eleven, 0),
    ] {
        assert!(missed.contains(&i), "{i:?} in {missed:?}");
    }
}

#[test]
fn a_hand_set_next_stays() {
    let (mut s, ten, _) = state();
    s.local_now = when(today(), 11, 30);
    apply(&mut s, Command::SetScheduleNext(item(today(), ten, 1))).unwrap();
    s.local_now = when(today(), 11, 45);
    settle_next(&mut s);
    let a = s.automation.as_ref().unwrap();
    assert_eq!(a.next, Some(item(today(), ten, 1)));
    assert!(a.next_by_hand);
    apply(&mut s, Command::ClearScheduleNext).unwrap();
    let a = s.automation.as_ref().unwrap();
    assert!(!a.next_by_hand);
    assert_eq!(a.next, Some(item(today().succ_opt().unwrap(), ten, 0)));
}

#[test]
fn next_crosses_midnight() {
    let (mut s, ten, _) = state();
    s.local_now = when(today(), 23, 59);
    assert_eq!(
        clock_next(&s),
        Some(item(today().succ_opt().unwrap(), ten, 0))
    );
}

#[test]
fn set_next_checks_the_item() {
    let (mut s, ten, _) = state();
    s.local_now = when(today(), 9, 0);
    assert_eq!(
        apply(&mut s, Command::SetScheduleNext(item(today(), ten, 5))),
        Err(ModelError::UnknownItem)
    );
    let far = today() + chrono::Days::new(10);
    assert_eq!(
        apply(&mut s, Command::SetScheduleNext(item(far, ten, 0))),
        Err(ModelError::UnknownBlock(ten))
    );
}

#[test]
fn late_tolerance_is_clamped() {
    let mut c = Config::default();
    assert_eq!(c.automation.late_tolerance_secs, 120);
    c.automation.late_tolerance_secs = 5_000;
    c.validate();
    assert_eq!(c.automation.late_tolerance_secs, 1_800);
}
