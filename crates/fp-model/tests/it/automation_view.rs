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
    let want = r.item(date, hour, 0, index);
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
