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

// Rework plan 4: what the Log's list shows (AR11, AR14–AR16).

fn item_row(rows: &[ScheduleRow], want: fp_model::automation::ItemRef) -> ScheduleRow {
    rows.iter()
        .find(|r| matches!(r, ScheduleRow::Item { item, .. } if *item == want))
        .cloned()
        .unwrap()
}

#[test]
fn rows_carry_their_planned_start() {
    let mut r = rig();
    r.at_hms(9, 0, 0);
    let rows = schedule_view(&r.s);
    let ScheduleRow::Item { starts_at, .. } = item_row(&rows, r.item(today(), 10, 0, 1)) else {
        unreachable!()
    };
    // Song 1 is 180 s long.
    assert_eq!(starts_at, today().and_hms_opt(10, 3, 0).unwrap());
}

#[test]
fn a_loaded_category_item_shows_its_track() {
    use fp_model::schedule::Item;
    let mut r = Rig::new(vec![(10, 0, vec![song(1)])]);
    r.apply(Command::CreateCategory {
        parent: None,
        name: "Spots".into(),
        color: None,
        kind: fp_model::TrackKind::Ad,
    });
    let cat = r.s.categories.iter().last().unwrap().id;
    let mut t = fp_model::Track::new(
        fp_model::TrackId(901),
        std::path::PathBuf::from("/m/spot.wav"),
    );
    t.duration_secs = 30.0;
    t.category = Some(cat);
    t.kind = fp_model::TrackKind::Ad;
    r.s.library.insert(t);
    let base = r.s.schedule.templates()[0].id;
    let mut block =
        r.s.schedule
            .template(base)
            .unwrap()
            .block(r.block(10, 0))
            .unwrap()
            .clone();
    block.items = vec![Item::Category {
        category: cat,
        time_critical: false,
    }];
    r.apply(Command::UpdateBlock {
        template: base,
        block,
    });
    r.at_hms(9, 59, 0);
    r.apply(Command::Play(r.schedule_player()));
    r.at_hms(10, 0, 0);
    let rows = schedule_view(&r.s);
    let ScheduleRow::Item { track, .. } = item_row(&rows, r.item(today(), 10, 0, 0)) else {
        unreachable!()
    };
    assert_eq!(track, Some(fp_model::TrackId(901)));
}

#[test]
fn the_music_fill_row_is_on_air_between_blocks() {
    use fp_model::schedule::Item;
    let mut r = rig();
    let base = r.s.schedule.templates()[0].id;
    let default = r.s.rotations.default_id().unwrap();
    let mut block =
        r.s.schedule
            .template(base)
            .unwrap()
            .block(r.block(10, 0))
            .unwrap()
            .clone();
    block.items.push(Item::Rotation { rotation: default });
    r.apply(Command::UpdateBlock {
        template: base,
        block,
    });
    r.at_hms(9, 59, 0);
    r.apply(Command::Play(r.schedule_player()));
    r.at_hms(10, 0, 0);
    let log = r.schedule_player();
    r.end_current(log);
    r.end_current(log);
    r.at_hms(10, 7, 0);
    assert!(r.playing(r.filler_player()).is_some());
    let rows = schedule_view(&r.s);
    let ScheduleRow::Item { status, .. } = item_row(&rows, r.item(today(), 10, 0, 2)) else {
        unreachable!()
    };
    assert_eq!(status, RowStatus::OnAir);
}

#[test]
fn a_late_next_is_marked_late() {
    let mut r = rig();
    r.at_hms(10, 30, 0);
    r.apply(Command::SetScheduleNext(r.item(today(), 10, 0, 0)));
    let rows = schedule_view(&r.s);
    let ScheduleRow::Item { late, .. } = item_row(&rows, r.item(today(), 10, 0, 0)) else {
        unreachable!()
    };
    assert!(late);
    let ScheduleRow::Item { late, .. } = item_row(&rows, r.item(today(), 12, 0, 0)) else {
        unreachable!()
    };
    assert!(!late);
}

#[test]
fn a_block_header_carries_its_total_and_overrun() {
    let mut r = Rig::new(vec![
        (10, 0, vec![song(1), song(2)]),
        (10, 5, vec![song(3)]),
    ]);
    r.at_hms(9, 0, 0);
    let rows = schedule_view(&r.s);
    let header = rows
        .iter()
        .find_map(|row| match row {
            ScheduleRow::Block {
                start,
                total_secs,
                overrun_secs,
                ..
            } if chrono::Timelike::minute(start) == 0 && chrono::Timelike::hour(start) == 10 => {
                Some((*total_secs, *overrun_secs))
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(header, (360.0, Some(60.0)));
}

#[test]
fn started_between_blocks_the_music_fill_row_is_on_air() {
    use fp_model::schedule::Item;
    let mut r = rig();
    let base = r.s.schedule.templates()[0].id;
    let default = r.s.rotations.default_id().unwrap();
    let mut block =
        r.s.schedule
            .template(base)
            .unwrap()
            .block(r.block(10, 0))
            .unwrap()
            .clone();
    block.items.push(Item::Rotation { rotation: default });
    r.apply(Command::UpdateBlock {
        template: base,
        block,
    });
    r.at_hms(10, 30, 0);
    r.apply(Command::Play(r.schedule_player()));
    assert!(r.playing(r.filler_player()).is_some());
    let rows = schedule_view(&r.s);
    let ScheduleRow::Item { status, .. } = item_row(&rows, r.item(today(), 10, 0, 2)) else {
        unreachable!()
    };
    assert_eq!(status, RowStatus::OnAir);
}

#[test]
fn a_category_items_track_stays_shown_after_its_block() {
    use fp_model::schedule::Item;
    let mut r = Rig::new(vec![(10, 0, vec![song(1)]), (11, 0, vec![song(3)])]);
    r.apply(Command::CreateCategory {
        parent: None,
        name: "Spots".into(),
        color: None,
        kind: fp_model::TrackKind::Ad,
    });
    let cat = r.s.categories.iter().last().unwrap().id;
    let mut t = fp_model::Track::new(
        fp_model::TrackId(901),
        std::path::PathBuf::from("/m/spot.wav"),
    );
    t.duration_secs = 30.0;
    t.category = Some(cat);
    t.kind = fp_model::TrackKind::Ad;
    r.s.library.insert(t);
    let base = r.s.schedule.templates()[0].id;
    let mut block =
        r.s.schedule
            .template(base)
            .unwrap()
            .block(r.block(10, 0))
            .unwrap()
            .clone();
    block.items = vec![Item::Category {
        category: cat,
        time_critical: false,
    }];
    r.apply(Command::UpdateBlock {
        template: base,
        block,
    });
    r.at_hms(9, 59, 0);
    r.apply(Command::Play(r.schedule_player()));
    r.at_hms(10, 0, 0);
    let log = r.schedule_player();
    r.end_current(log);
    r.at_hms(10, 1, 0);
    let rows = schedule_view(&r.s);
    let ScheduleRow::Item { track, .. } = item_row(&rows, r.item(today(), 10, 0, 0)) else {
        unreachable!()
    };
    assert_eq!(track, Some(fp_model::TrackId(901)));
}

#[test]
fn plan_view_lists_a_templates_blocks() {
    let mut r = rig();
    r.at_hms(10, 0, 30);
    let base = r.s.schedule.templates()[0].id;
    let rows =
        fp_model::automation::plan_view(&r.s, fp_model::log_edit::EditTarget::Template(base));
    let starts: Vec<(u32, usize)> = rows
        .iter()
        .filter_map(|row| match row {
            ScheduleRow::Block { start, count, .. } => {
                Some((chrono::Timelike::hour(start), *count))
            }
            ScheduleRow::Item { .. } => None,
        })
        .collect();
    assert_eq!(starts, vec![(10, 2), (12, 1)]);
    // A template never plays: every row is coming, nothing is next.
    assert!(rows.iter().all(|row| match row {
        ScheduleRow::Block { status, .. } | ScheduleRow::Item { status, .. } =>
            *status == RowStatus::Coming,
    }));
    assert_eq!(rows.len(), 5);
}

#[test]
fn plan_view_of_a_date_has_its_changes() {
    let mut r = rig();
    r.at_hms(9, 0, 0);
    let tomorrow = today().checked_add_days(Days::new(1)).unwrap();
    let block = r.block(12, 0);
    r.apply(Command::EditPlan {
        target: fp_model::log_edit::EditTarget::Date(tomorrow),
        edit: fp_model::log_edit::LogEdit::Insert {
            block,
            index: 0,
            items: vec![song(7)],
        },
    });
    let rows =
        fp_model::automation::plan_view(&r.s, fp_model::log_edit::EditTarget::Date(tomorrow));
    let changed = rows.iter().find_map(|row| match row {
        ScheduleRow::Block {
            block: b,
            origin,
            date,
            count,
            ..
        } if *b == block => Some((*origin, *date, *count)),
        _ => None,
    });
    assert_eq!(
        changed,
        Some((fp_model::schedule::BlockOrigin::Changed, tomorrow, 2))
    );
}
