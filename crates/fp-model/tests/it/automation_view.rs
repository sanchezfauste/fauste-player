#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Automation plan 6: the rows of the Schedule column (§8.1, AU23).

use chrono::{Days, NaiveDate};
use fp_model::Command;
use fp_model::automation::{RowStatus, ScheduleRow, schedule_view};

use crate::automation_rig::{Rig, song, today};

/// 10:00 (two songs) and 12:00 (one song), every day.
fn rig() -> Rig {
    Rig::new(vec![
        (10, 0, vec![song(1), song(2)]),
        (12, 0, vec![song(3)]),
    ])
}

fn blocks(rows: &[ScheduleRow]) -> Vec<(NaiveDate, u32, RowStatus)> {
    rows.iter()
        .filter_map(|r| match r {
            ScheduleRow::Block {
                date,
                start,
                status,
                ..
            } => Some((*date, chrono::Timelike::hour(start), *status)),
            ScheduleRow::Item { .. } => None,
        })
        .collect()
}

fn item_status(
    rows: &[ScheduleRow],
    date: NaiveDate,
    hour: u32,
    index: usize,
    r: &Rig,
) -> RowStatus {
    item_status_at(rows, date, hour, 0, index, r)
}

fn item_status_at(
    rows: &[ScheduleRow],
    date: NaiveDate,
    hour: u32,
    minute: u32,
    index: usize,
    r: &Rig,
) -> RowStatus {
    let want = r.item(date, hour, minute, index);
    rows.iter()
        .find_map(|row| match row {
            ScheduleRow::Item { item, status, .. } if *item == want => Some(*status),
            _ => None,
        })
        .expect("the item is listed")
}

#[test]
fn the_view_lists_yesterdays_last_today_and_tomorrows_first() {
    let mut r = rig();
    r.at_hms(8, 0, 0);
    let rows = schedule_view(&r.s);
    let yesterday = today().checked_sub_days(Days::new(1)).unwrap();
    let tomorrow = today().checked_add_days(Days::new(1)).unwrap();
    let dates: Vec<(NaiveDate, u32)> = blocks(&rows).into_iter().map(|(d, h, _)| (d, h)).collect();
    assert_eq!(
        dates,
        vec![
            (yesterday, 12),
            (today(), 10),
            (today(), 12),
            (tomorrow, 10)
        ]
    );
    // Each block is followed by its items.
    assert_eq!(rows.len(), 4 + 1 + 2 + 1 + 2);
    assert!(matches!(rows[1], ScheduleRow::Item { .. }));
}

#[test]
fn the_item_on_air_and_next_are_marked() {
    let mut r = rig();
    r.at_hms(9, 59, 0);
    r.apply(Command::StartAutomation);
    r.at_hms(10, 0, 0);
    let rows = schedule_view(&r.s);
    assert_eq!(item_status(&rows, today(), 10, 0, &r), RowStatus::OnAir);
    assert_eq!(item_status(&rows, today(), 10, 1, &r), RowStatus::Coming);
    assert_eq!(item_status(&rows, today(), 12, 0, &r), RowStatus::Next);
    let b = blocks(&rows);
    assert!(b.contains(&(today(), 10, RowStatus::OnAir)));
    assert!(b.contains(&(today(), 12, RowStatus::Next)));
}

#[test]
fn missed_items_and_blocks_are_marked() {
    let mut r = rig();
    r.at_hms(9, 0, 0);
    r.at_hms(10, 5, 0);
    let rows = schedule_view(&r.s);
    assert_eq!(item_status(&rows, today(), 10, 0, &r), RowStatus::Missed);
    assert_eq!(item_status(&rows, today(), 10, 1, &r), RowStatus::Missed);
    assert!(blocks(&rows).contains(&(today(), 10, RowStatus::Missed)));
}

#[test]
fn aired_items_are_marked_after_their_block() {
    let mut r = rig();
    r.at_hms(9, 59, 0);
    r.apply(Command::StartAutomation);
    r.at_hms(10, 0, 0);
    let p = r.schedule_player();
    r.end_current(p);
    let rows = schedule_view(&r.s);
    assert_eq!(item_status(&rows, today(), 10, 0, &r), RowStatus::Aired);
    assert_eq!(item_status(&rows, today(), 10, 1, &r), RowStatus::OnAir);
    r.end_current(p);
    r.at_hms(10, 7, 0);
    let rows = schedule_view(&r.s);
    assert_eq!(item_status(&rows, today(), 10, 1, &r), RowStatus::Aired);
    assert!(blocks(&rows).contains(&(today(), 10, RowStatus::Aired)));
}

#[test]
fn a_next_set_by_hand_is_marked_where_it_is() {
    let mut r = rig();
    r.at_hms(8, 0, 0);
    let tomorrow = today().checked_add_days(Days::new(1)).unwrap();
    let item = r.item(tomorrow, 10, 0, 1);
    r.apply(Command::SetScheduleNext(item));
    let rows = schedule_view(&r.s);
    assert_eq!(item_status(&rows, tomorrow, 10, 1, &r), RowStatus::Next);
    assert_eq!(item_status(&rows, today(), 10, 0, &r), RowStatus::Coming);
}

#[test]
fn a_missed_block_played_by_hand_shows_aired() {
    let mut r = rig();
    r.at_hms(9, 0, 0);
    r.at_hms(10, 5, 0);
    let first = r.item(today(), 10, 0, 0);
    r.apply(Command::SetScheduleNext(first));
    r.apply(Command::StartAutomation);
    let p = r.schedule_player();
    assert!(r.playing(p).is_some(), "the block plays");
    r.end_current(p);
    r.end_current(p);
    r.at_hms(10, 10, 0);
    let rows = schedule_view(&r.s);
    assert_eq!(item_status(&rows, today(), 10, 0, &r), RowStatus::Aired);
    assert_eq!(item_status(&rows, today(), 10, 1, &r), RowStatus::Aired);
    assert!(blocks(&rows).contains(&(today(), 10, RowStatus::Aired)));
}

#[test]
fn a_missed_item_set_as_next_is_marked_next() {
    let mut r = rig();
    r.at_hms(9, 0, 0);
    r.at_hms(10, 5, 0);
    let second = r.item(today(), 10, 0, 1);
    r.apply(Command::SetScheduleNext(second));
    let rows = schedule_view(&r.s);
    assert_eq!(item_status(&rows, today(), 10, 1, &r), RowStatus::Next);
}

#[test]
fn items_dropped_by_a_hand_set_next_are_missed() {
    let mut r = rig();
    r.at_hms(9, 59, 0);
    r.apply(Command::StartAutomation);
    r.at_hms(10, 0, 0);
    let twelve = r.item(today(), 12, 0, 0);
    r.apply(Command::SetScheduleNext(twelve));
    let rows = schedule_view(&r.s);
    assert_eq!(item_status(&rows, today(), 10, 0, &r), RowStatus::OnAir);
    assert_eq!(item_status(&rows, today(), 10, 1, &r), RowStatus::Missed);
}

#[test]
fn a_block_on_air_past_midnight_stays_listed() {
    let mut r = Rig::new(vec![
        (22, 0, vec![song(1), song(2)]),
        (23, 30, vec![song(3)]),
    ]);
    r.at_hms(21, 59, 0);
    r.apply(Command::StartAutomation);
    r.at_hms(22, 0, 0);
    let tomorrow = today().checked_add_days(Days::new(1)).unwrap();
    r.at(tomorrow.and_hms_opt(0, 10, 0).unwrap());
    let rows = schedule_view(&r.s);
    assert!(
        blocks(&rows).contains(&(today(), 22, RowStatus::OnAir)),
        "{:?}",
        blocks(&rows)
    );
    assert_eq!(item_status(&rows, today(), 22, 0, &r), RowStatus::OnAir);
}

#[test]
fn during_an_overrun_the_overdue_block_is_next() {
    let mut r = Rig::new(vec![
        (10, 0, vec![song(1), song(2)]),
        (10, 10, vec![song(3)]),
        (10, 40, vec![song(4)]),
    ]);
    r.at_hms(9, 59, 0);
    r.apply(Command::StartAutomation);
    r.at_hms(10, 0, 0);
    r.at_hms(10, 16, 0);
    let rows = schedule_view(&r.s);
    assert_eq!(
        item_status_at(&rows, today(), 10, 10, 0, &r),
        RowStatus::Next
    );
}
