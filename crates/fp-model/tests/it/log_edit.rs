#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Automation rework AR17–AR19: the Log edited live, as date changes.

use fp_model::automation::ItemRef;
use fp_model::log_edit::{EditTarget, LogEdit};
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
        Command::EditPlan {
            target: EditTarget::Date(today()),
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
            Command::EditPlan {
                target: EditTarget::Date(yesterday),
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

// Review of rework plan 4.

fn status_of(r: &Rig, want: ItemRef) -> fp_model::automation::RowStatus {
    fp_model::automation::schedule_view(&r.s)
        .into_iter()
        .find_map(|row| match row {
            fp_model::automation::ScheduleRow::Item { item, status, .. } if item == want => {
                Some(status)
            }
            _ => None,
        })
        .unwrap()
}

#[test]
fn rows_queued_again_after_an_edit_are_not_missed() {
    let mut r = on_air();
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
    let inserted = ItemRef {
        date: today(),
        block,
        index: 1,
    };
    assert_ne!(
        status_of(&r, inserted),
        fp_model::automation::RowStatus::Missed
    );
    assert!(!r.missed().contains(&inserted));
}

#[test]
fn an_edit_late_in_the_block_keeps_its_hard_start_items() {
    use crate::automation_rig::hourly_id;
    let mut r = Rig::new(vec![(10, 0, vec![song(1), song(2), hourly_id(3)])]);
    r.at_hms(9, 59, 0);
    r.apply(Command::Play(r.schedule_player()));
    r.at_hms(10, 0, 0);
    r.at_hms(10, 5, 0);
    let block = r.block(10, 0);
    edit(&mut r, LogEdit::Remove { block, index: 1 }).unwrap();
    let log = r.schedule_player();
    let next = r
        .player(log)
        .next
        .and_then(|e| r.s.playlists.entry(e))
        .map(|e| e.track);
    assert_eq!(next, Some(TrackId(3)));
}

#[test]
fn moving_a_row_onto_the_one_below_changes_nothing() {
    let mut r = rig();
    r.at_hms(9, 0, 0);
    let block = r.block(10, 0);
    edit(
        &mut r,
        LogEdit::Move {
            from: (block, 0),
            to: (block, 1),
        },
    )
    .unwrap();
    assert_eq!(items(&r, today(), 10).1, BlockOrigin::Template);
}

fn base(r: &Rig) -> fp_model::TemplateId {
    r.s.schedule.templates()[0].id
}

fn edit_template(r: &mut Rig, edit: LogEdit) -> Result<(), ModelError> {
    let template = base(r);
    apply(
        &mut r.s,
        Command::EditPlan {
            target: EditTarget::Template(template),
            edit,
        },
    )
    .map(|actions| r.engine(&actions))
}

fn template_items(r: &Rig, h: u32) -> Vec<Item> {
    r.s.schedule
        .template(base(r))
        .unwrap()
        .blocks
        .iter()
        .find(|b| b.start == tod(h, 0))
        .unwrap()
        .items
        .clone()
}

#[test]
fn a_template_edit_changes_the_template_only() {
    let mut r = rig();
    r.at_hms(9, 0, 0);
    let block = r.block(11, 0);
    edit_template(
        &mut r,
        LogEdit::Insert {
            block,
            index: 0,
            items: vec![song(7)],
        },
    )
    .unwrap();
    assert_eq!(template_items(&r, 11), vec![song(7), song(3)]);
    // No date change: today follows the template.
    let (today_items, origin) = items(&r, today(), 11);
    assert_eq!(today_items, vec![song(7), song(3)]);
    assert_eq!(origin, BlockOrigin::Template);
    assert!(r.s.schedule.date_plan(today()).is_none());
    edit_template(&mut r, LogEdit::Remove { block, index: 1 }).unwrap();
    assert_eq!(template_items(&r, 11), vec![song(7)]);
}

#[test]
fn a_template_block_can_be_added_and_removed() {
    let mut r = rig();
    r.at_hms(9, 0, 0);
    edit_template(&mut r, LogEdit::AddBlock { start: tod(23, 30) }).unwrap();
    let added =
        r.s.schedule
            .template(base(&r))
            .unwrap()
            .blocks
            .iter()
            .find(|b| b.start == tod(23, 30))
            .map(|b| b.id)
            .unwrap();
    edit_template(
        &mut r,
        LogEdit::MoveBlock {
            block: added,
            start: tod(23, 45),
        },
    )
    .unwrap();
    assert!(
        r.s.schedule
            .template(base(&r))
            .unwrap()
            .block(added)
            .is_some_and(|b| b.start == tod(23, 45))
    );
    edit_template(&mut r, LogEdit::RemoveBlock { block: added }).unwrap();
    assert!(
        r.s.schedule
            .template(base(&r))
            .unwrap()
            .block(added)
            .is_none()
    );
    assert_eq!(
        edit_template(&mut r, LogEdit::Revert { block: added }).err(),
        Some(ModelError::UnknownBlock(added))
    );
}

#[test]
fn template_edits_are_never_locked() {
    let mut r = on_air();
    let block = r.block(10, 0);
    let log = r.schedule_player();
    // The template's first item is on air today; the template still changes.
    edit_template(&mut r, LogEdit::Remove { block, index: 0 }).unwrap();
    assert_eq!(template_items(&r, 10), vec![song(2)]);
    // The loaded block is not reloaded: the Log goes on as it was loaded.
    edit_template(
        &mut r,
        LogEdit::Insert {
            block,
            index: 0,
            items: vec![song(9)],
        },
    )
    .unwrap();
    assert_eq!(r.playing(log), Some(TrackId(1)));
    r.end_current(log);
    assert_eq!(r.playing(log), Some(TrackId(2)));
}

#[test]
fn an_item_moves_to_another_block() {
    let mut r = rig();
    r.at_hms(9, 0, 0);
    let ten = r.block(10, 0);
    let eleven = r.block(11, 0);
    edit(
        &mut r,
        LogEdit::Move {
            from: (ten, 1),
            to: (eleven, 0),
        },
    )
    .unwrap();
    assert_eq!(items(&r, today(), 10).0, vec![song(1)]);
    assert_eq!(items(&r, today(), 11).0, vec![song(2), song(3)]);
    // Within a block, down past the row below.
    edit(
        &mut r,
        LogEdit::Move {
            from: (eleven, 0),
            to: (eleven, 2),
        },
    )
    .unwrap();
    assert_eq!(items(&r, today(), 11).0, vec![song(3), song(2)]);
}

#[test]
fn an_unknown_row_is_refused() {
    let mut r = rig();
    r.at_hms(9, 0, 0);
    let block = r.block(11, 0);
    assert_eq!(
        edit(&mut r, LogEdit::Remove { block, index: 5 }),
        Err(ModelError::UnknownItem)
    );
    assert_eq!(
        edit(
            &mut r,
            LogEdit::Move {
                from: (block, 5),
                to: (block, 0),
            }
        ),
        Err(ModelError::UnknownItem)
    );
    assert_eq!(
        edit(
            &mut r,
            LogEdit::SetHardStart {
                block,
                index: 5,
                on: true,
            }
        ),
        Err(ModelError::UnknownItem)
    );
    let gone = fp_model::BlockId(999);
    assert_eq!(
        edit(
            &mut r,
            LogEdit::Remove {
                block: gone,
                index: 0
            }
        ),
        Err(ModelError::UnknownBlock(gone))
    );
}

#[test]
fn a_changed_block_goes_back_to_the_template() {
    let mut r = rig();
    r.at_hms(9, 0, 0);
    let block = r.block(11, 0);
    edit(&mut r, LogEdit::Remove { block, index: 0 }).unwrap();
    assert!(items(&r, today(), 11).0.is_empty());
    edit(&mut r, LogEdit::Revert { block }).unwrap();
    assert_eq!(
        items(&r, today(), 11),
        (vec![song(3)], BlockOrigin::Template)
    );
}

#[test]
fn a_template_edit_keeps_todays_block_on_air_as_loaded() {
    use fp_model::automation::{RowStatus, ScheduleRow, schedule_view};
    let mut r = on_air();
    let block = r.block(10, 0);
    edit_template(
        &mut r,
        LogEdit::Insert {
            block,
            index: 0,
            items: vec![song(9)],
        },
    )
    .unwrap();
    // The Log still shows what plays: today's block keeps its items (as a
    // change of today), the template has the new one for the days after.
    assert_eq!(items(&r, today(), 10).0, vec![song(1), song(2)]);
    let tomorrow = today().succ_opt().unwrap();
    assert_eq!(items(&r, tomorrow, 10).0, vec![song(9), song(1), song(2)]);
    let on_air: Vec<_> = schedule_view(&r.s)
        .into_iter()
        .filter_map(|row| match row {
            ScheduleRow::Item {
                status: RowStatus::OnAir,
                track,
                ..
            } => Some(track),
            _ => None,
        })
        .collect();
    assert_eq!(on_air, vec![Some(TrackId(1))]);
}

#[test]
fn a_refused_template_edit_changes_nothing() {
    let mut r = on_air();
    let block = r.block(10, 0);
    let before = r.s.schedule.clone();
    assert_eq!(
        edit_template(&mut r, LogEdit::Remove { block, index: 9 }),
        Err(ModelError::UnknownItem)
    );
    // Today did not keep the block as loaded for an edit that never was.
    assert_eq!(r.s.schedule, before);
}

#[test]
fn dropped_files_become_tracks_of_the_block() {
    let mut r = rig();
    r.at_hms(9, 0, 0);
    let block = r.block(11, 0);
    edit(
        &mut r,
        LogEdit::InsertPaths {
            block,
            index: 0,
            paths: vec!["/music/new.flac".into()],
        },
    )
    .unwrap();
    let items = items(&r, today(), 11).0;
    let Some(Item::Track { track, .. }) = items.first() else {
        panic!("a track first: {items:?}");
    };
    assert_eq!(
        r.s.library.get(*track).unwrap().path,
        std::path::PathBuf::from("/music/new.flac")
    );
    assert_eq!(items.len(), 2);
}

#[test]
fn files_dropped_on_a_locked_row_add_nothing() {
    let mut r = on_air();
    let block = r.block(10, 0);
    let before = r.s.library.len();
    assert_eq!(
        edit(
            &mut r,
            LogEdit::InsertPaths {
                block,
                index: 0,
                paths: vec!["/music/new.flac".into()],
            },
        ),
        Err(ModelError::LogRowLocked)
    );
    assert_eq!(r.s.library.len(), before);
}

/// 10:00: song 1, a "Spots" category item (only 901) and a "Jingles" one
/// (only 902), on air from 10:00.
fn with_categories() -> Rig {
    use fp_model::{Track, TrackKind};
    let mut r = Rig::new(vec![(11, 0, vec![song(3)])]);
    let category = |r: &mut Rig, name: &str, track: u64| {
        r.apply(Command::CreateCategory {
            parent: None,
            name: name.into(),
            color: None,
            kind: TrackKind::Ad,
        });
        let id = r.s.categories.iter().last().unwrap().id;
        let mut t = Track::new(TrackId(track), format!("/m/{track}.wav").into());
        t.duration_secs = 30.0;
        t.category = Some(id);
        t.kind = TrackKind::Ad;
        r.s.library.insert(t);
        id
    };
    let spots = category(&mut r, "Spots", 901);
    let jingles = category(&mut r, "Jingles", 902);
    let base = r.base;
    r.apply(Command::AddBlock {
        template: base,
        start: tod(10, 0),
    });
    let id =
        r.s.schedule
            .template(base)
            .unwrap()
            .blocks
            .iter()
            .find(|b| b.start == tod(10, 0))
            .unwrap()
            .id;
    r.blocks.insert((10, 0), id);
    r.apply(Command::UpdateBlock {
        template: base,
        block: fp_model::schedule::Block {
            id,
            start: tod(10, 0),
            items: vec![
                song(1),
                Item::Category {
                    category: spots,
                    time_critical: false,
                },
                Item::Category {
                    category: jingles,
                    time_critical: false,
                },
            ],
        },
    });
    r.at_hms(9, 59, 0);
    r.apply(Command::StartAutomation);
    r.at_hms(10, 0, 0);
    assert_eq!(r.playing(r.schedule_player()), Some(TrackId(1)));
    r
}

/// The Log's rows after the one on air.
fn queued(r: &Rig) -> Vec<TrackId> {
    let a = r.s.automation.as_ref().unwrap();
    let current = r.player(a.player).current;
    let list = r.s.playlists.get(a.playlist).unwrap();
    list.entries
        .iter()
        .skip_while(|e| Some(e.id) != current)
        .skip(1)
        .map(|e| e.track)
        .collect()
}

#[test]
fn a_row_inserted_before_category_items_keeps_their_chosen_tracks() {
    let mut r = with_categories();
    assert_eq!(queued(&r), vec![TrackId(901), TrackId(902)]);
    let block = r.block(10, 0);
    edit(
        &mut r,
        LogEdit::Insert {
            block,
            index: 1,
            items: vec![song(3)],
        },
    )
    .unwrap();
    assert_eq!(queued(&r), vec![TrackId(3), TrackId(901), TrackId(902)]);
}

#[test]
fn a_removed_row_before_category_items_keeps_their_chosen_tracks() {
    let mut r = with_categories();
    let block = r.block(10, 0);
    edit(&mut r, LogEdit::Remove { block, index: 1 }).unwrap();
    assert_eq!(queued(&r), vec![TrackId(902)]);
}

#[test]
fn moved_category_items_keep_their_chosen_tracks() {
    let mut r = with_categories();
    let block = r.block(10, 0);
    edit(
        &mut r,
        LogEdit::Move {
            from: (block, 2),
            to: (block, 1),
        },
    )
    .unwrap();
    assert_eq!(queued(&r), vec![TrackId(902), TrackId(901)]);
}

/// The 10:00 block on air: a song, then two items of one category of two
/// spots (901, 903), each with its own track.
fn two_spot_items() -> Rig {
    use fp_model::{Track, TrackKind};
    let mut r = Rig::new(vec![(11, 0, vec![song(3)])]);
    r.s.config.automation.track_separation_mins = 60;
    r.apply(Command::CreateCategory {
        parent: None,
        name: "Spots".into(),
        color: None,
        kind: TrackKind::Ad,
    });
    let spots = r.s.categories.iter().last().unwrap().id;
    for track in [901, 903] {
        let mut t = Track::new(TrackId(track), format!("/m/{track}.wav").into());
        t.duration_secs = 30.0;
        t.category = Some(spots);
        t.kind = TrackKind::Ad;
        r.s.library.insert(t);
    }
    let base = r.base;
    r.apply(Command::AddBlock {
        template: base,
        start: tod(10, 0),
    });
    let id =
        r.s.schedule
            .template(base)
            .unwrap()
            .blocks
            .iter()
            .find(|b| b.start == tod(10, 0))
            .unwrap()
            .id;
    r.blocks.insert((10, 0), id);
    let spot = Item::Category {
        category: spots,
        time_critical: false,
    };
    r.apply(Command::UpdateBlock {
        template: base,
        block: fp_model::schedule::Block {
            id,
            start: tod(10, 0),
            items: vec![song(1), spot.clone(), spot],
        },
    });
    r.at_hms(9, 59, 0);
    r.apply(Command::StartAutomation);
    r.at_hms(10, 0, 0);
    assert_eq!(r.playing(r.schedule_player()), Some(TrackId(1)));
    r
}

/// Review of #270: removing one of two items of a category drops its own
/// track, not the other's.
#[test]
fn removing_one_of_two_items_of_a_category_drops_its_own_track() {
    for index in [1, 2] {
        let mut r = two_spot_items();
        let before = queued(&r);
        assert_eq!(before.len(), 2);
        let block = r.block(10, 0);
        edit(&mut r, LogEdit::Remove { block, index }).unwrap();
        let kept = before[if index == 1 { 1 } else { 0 }];
        assert_eq!(queued(&r), vec![kept], "removed index {index}");
    }
}

/// Review of #270: moving an item of a category into the block on air
/// leaves every other item's track where it was.
#[test]
fn an_item_moved_into_the_block_on_air_keeps_every_track_in_place() {
    let mut r = two_spot_items();
    let before = queued(&r);
    let block = r.block(10, 0);
    // The second spot moves before the first.
    edit(
        &mut r,
        LogEdit::Move {
            from: (block, 2),
            to: (block, 1),
        },
    )
    .unwrap();
    assert_eq!(queued(&r), vec![before[1], before[0]]);
    // A song of the 11:00 block moves into the block on air, first after
    // the row on air: the spots keep their tracks.
    let later = r.block(11, 0);
    edit(
        &mut r,
        LogEdit::Move {
            from: (later, 0),
            to: (block, 1),
        },
    )
    .unwrap();
    assert_eq!(queued(&r), vec![TrackId(3), before[1], before[0]]);
}

#[test]
fn landing_names_the_row_an_insert_adds() {
    let default = Rig::new(vec![]).s.rotations.default_id().unwrap();
    let fill = Item::Rotation { rotation: default };
    let cases: Vec<(Vec<Item>, usize, Vec<Item>)> = vec![
        (vec![song(3)], 0, vec![song(7)]),
        (vec![song(3)], 1, vec![song(7), song(8)]),
        (vec![song(3), fill.clone()], 9, vec![song(7)]),
        (
            vec![song(3), fill.clone()],
            usize::MAX,
            vec![song(7), song(8)],
        ),
        (vec![song(3), fill.clone()], 0, vec![fill.clone()]),
        (vec![song(3)], 0, vec![fill.clone()]),
    ];
    // Each added item is in no list before, so its place names it.
    for (items, index, adding) in cases {
        let mut r = Rig::new(vec![(11, 0, items.clone())]);
        r.at_hms(9, 0, 0);
        let block = r.block(11, 0);
        let want = fp_model::log_edit::landing(&items, index, &adding).unwrap();
        let last = adding.last().unwrap().clone();
        edit(
            &mut r,
            LogEdit::Insert {
                block,
                index,
                items: adding,
            },
        )
        .unwrap();
        let after = items_of_block(&r, 11);
        assert_eq!(
            after.get(want),
            Some(&last),
            "{items:?} at {index}: {after:?}"
        );
    }
    assert_eq!(fp_model::log_edit::landing(&[song(3)], 0, &[]), None);
}

fn items_of_block(r: &Rig, h: u32) -> Vec<Item> {
    items(r, today(), h).0
}
