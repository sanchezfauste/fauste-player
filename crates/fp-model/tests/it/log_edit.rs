#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Automation rework AR17–AR19: the Log edited live, as date changes.

use fp_model::automation::ItemRef;
use fp_model::log_edit::LogEdit;
use fp_model::schedule::{BlockOrigin, Item};
use fp_model::{Command, ModelError, TrackId, apply};

use crate::automation_rig::{Rig, song, tod, today};

fn rig() -> Rig {
    Rig::new(vec![
        (10, 0, vec![song(1), song(2)]),
        (11, 0, vec![song(3)]),
    ])
}

fn edit(r: &mut Rig, edit: LogEdit) -> Result<(), ModelError> {
    apply(
        &mut r.s,
        Command::EditLog {
            date: today(),
            edit,
        },
    )
    .map(|actions| r.engine(&actions))
}

fn items(r: &Rig, date: chrono::NaiveDate, h: u32) -> (Vec<Item>, BlockOrigin) {
    let b =
        r.s.schedule
            .day(date)
            .into_iter()
            .find(|b| b.start == tod(h, 0))
            .unwrap();
    (b.items, b.origin)
}

#[test]
fn inserting_a_track_changes_only_that_date() {
    let mut r = rig();
    r.at_hms(9, 0, 0);
    let block = r.block(11, 0);
    edit(
        &mut r,
        LogEdit::Insert {
            block,
            index: 0,
            items: vec![song(7)],
        },
    )
    .unwrap();
    let (today_items, origin) = items(&r, today(), 11);
    assert_eq!(today_items, vec![song(7), song(3)]);
    assert_ne!(origin, BlockOrigin::Template);
    let tomorrow = today().succ_opt().unwrap();
    assert_eq!(items(&r, tomorrow, 11).0, vec![song(3)]);
}

#[test]
fn a_drop_past_a_music_fill_goes_before_it() {
    let mut r = rig();
    r.at_hms(9, 0, 0);
    let default = r.s.rotations.default_id().unwrap();
    let block = r.block(11, 0);
    edit(
        &mut r,
        LogEdit::Insert {
            block,
            index: 1,
            items: vec![Item::Rotation { rotation: default }],
        },
    )
    .unwrap();
    edit(
        &mut r,
        LogEdit::Insert {
            block,
            index: 9,
            items: vec![song(8)],
        },
    )
    .unwrap();
    assert_eq!(
        items(&r, today(), 11).0,
        vec![song(3), song(8), Item::Rotation { rotation: default }]
    );
}

#[test]
fn a_new_rotation_replaces_the_blocks() {
    let mut r = rig();
    r.at_hms(9, 0, 0);
    let default = r.s.rotations.default_id().unwrap();
    let block = r.block(11, 0);
    edit(
        &mut r,
        LogEdit::Insert {
            block,
            index: 1,
            items: vec![Item::Rotation { rotation: default }],
        },
    )
    .unwrap();
    edit(
        &mut r,
        LogEdit::Insert {
            block,
            index: 0,
            items: vec![Item::Rotation { rotation: default }],
        },
    )
    .unwrap();
    assert_eq!(
        items(&r, today(), 11).0,
        vec![song(3), Item::Rotation { rotation: default }]
    );
}

/// Song 1 of the 10:00 block on air.
fn on_air() -> Rig {
    let mut r = rig();
    r.at_hms(9, 59, 0);
    r.apply(Command::Play(r.schedule_player()));
    r.at_hms(10, 0, 0);
    assert_eq!(r.playing(r.schedule_player()), Some(TrackId(1)));
    r
}

#[test]
fn the_row_on_air_cannot_be_removed() {
    let mut r = on_air();
    let block = r.block(10, 0);
    assert_eq!(
        edit(&mut r, LogEdit::Remove { block, index: 0 }),
        Err(ModelError::LogRowLocked)
    );
}

#[test]
fn yesterday_cannot_be_edited() {
    let mut r = rig();
    r.at_hms(9, 0, 0);
    let block = r.block(10, 0);
    let yesterday = today().pred_opt().unwrap();
    assert_eq!(
        apply(
            &mut r.s,
            Command::EditLog {
                date: yesterday,
                edit: LogEdit::Remove { block, index: 0 }
            }
        )
        .err(),
        Some(ModelError::PastDate)
    );
}

#[test]
fn editing_the_block_on_air_reloads_after_the_row_on_air() {
    let mut r = on_air();
    let log = r.schedule_player();
    let block = r.block(10, 0);
    edit(
        &mut r,
        LogEdit::Insert {
            block,
            index: 1,
            items: vec![song(9)],
        },
    )
    .unwrap();
    assert_eq!(r.playing(log), Some(TrackId(1)), "never cut");
    r.end_current(log);
    assert_eq!(r.playing(log), Some(TrackId(9)));
    r.end_current(log);
    assert_eq!(r.playing(log), Some(TrackId(2)));
}

#[test]
fn a_block_can_be_added_moved_and_removed_today() {
    let mut r = rig();
    r.at_hms(9, 0, 0);
    edit(&mut r, LogEdit::AddBlock { start: tod(11, 30) }).unwrap();
    let added =
        r.s.schedule
            .day(today())
            .into_iter()
            .find(|b| b.start == tod(11, 30))
            .unwrap()
            .id;
    edit(
        &mut r,
        LogEdit::MoveBlock {
            block: added,
            start: tod(11, 45),
        },
    )
    .unwrap();
    assert!(
        r.s.schedule
            .day(today())
            .iter()
            .any(|b| b.start == tod(11, 45))
    );
    edit(&mut r, LogEdit::RemoveBlock { block: added }).unwrap();
    assert!(!r.s.schedule.day(today()).iter().any(|b| b.id == added));
}

#[test]
fn hard_start_toggles_on_one_item() {
    let mut r = rig();
    r.at_hms(9, 0, 0);
    let block = r.block(11, 0);
    edit(
        &mut r,
        LogEdit::SetHardStart {
            block,
            index: 0,
            on: true,
        },
    )
    .unwrap();
    assert_eq!(
        items(&r, today(), 11).0,
        vec![Item::Track {
            track: TrackId(3),
            time_critical: true
        }]
    );
    let _ = ItemRef {
        date: today(),
        block,
        index: 0,
    };
}
