#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Automation spec AU10–AU12: per-date templates and date overrides.

use chrono::{NaiveDate, Weekday};
use fp_model::local_time::TimeOfDay;
use fp_model::schedule::{Block, DateChange, Item, ensure_default};
use fp_model::{AppState, BlockId, Command, Config, ModelError, TemplateId, TrackId, apply};

fn at(h: u32, m: u32) -> TimeOfDay {
    TimeOfDay::hms(h, m, 0).unwrap()
}

/// A Tuesday.
fn tuesday() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 11, 17).unwrap()
}

/// "Base" with blocks at 07:00, 08:00 and 09:00, on every day.
fn based() -> (AppState, TemplateId) {
    let mut s = AppState::new(Config::default(), "P");
    ensure_default(&mut s, "Base");
    let base = s.schedule.templates()[0].id;
    for h in [7, 8, 9] {
        apply(
            &mut s,
            Command::AddBlock {
                template: base,
                start: at(h, 0),
            },
        )
        .unwrap();
    }
    (s, base)
}

fn block_at(s: &AppState, t: TemplateId, start: TimeOfDay) -> Block {
    s.schedule
        .template(t)
        .unwrap()
        .blocks
        .iter()
        .find(|b| b.start == start)
        .unwrap()
        .clone()
}

fn song(n: u64) -> Item {
    Item::Track {
        track: TrackId(n),
        time_critical: false,
    }
}

fn weekend(s: &mut AppState) -> TemplateId {
    apply(
        s,
        Command::CreateTemplate {
            name: "Weekend".into(),
        },
    )
    .unwrap();
    s.schedule.templates().last().unwrap().id
}

#[test]
fn a_date_can_use_another_template() {
    let (mut s, base) = based();
    let weekend = weekend(&mut s);
    let date = tuesday();
    apply(
        &mut s,
        Command::SetDateTemplate {
            date,
            template: Some(weekend),
        },
    )
    .unwrap();
    assert_eq!(s.schedule.template_for(date), Some(weekend));
    assert_eq!(
        s.schedule.template_for(date + chrono::Days::new(7)),
        Some(base)
    );
    apply(
        &mut s,
        Command::SetDateTemplate {
            date,
            template: None,
        },
    )
    .unwrap();
    assert!(s.schedule.date_plan(date).is_none());
    assert_eq!(s.schedule.template_for(date), Some(base));
    assert_eq!(
        apply(
            &mut s,
            Command::SetDateTemplate {
                date,
                template: Some(TemplateId(999))
            }
        ),
        Err(ModelError::UnknownTemplate(TemplateId(999)))
    );
}

#[test]
fn a_change_stores_only_that_block() {
    let (mut s, base) = based();
    let mut block = block_at(&s, base, at(8, 0));
    block.items = vec![song(1)];
    apply(
        &mut s,
        Command::ChangeBlockOnDate {
            date: tuesday(),
            block: block.clone(),
        },
    )
    .unwrap();
    let plan = s.schedule.date_plan(tuesday()).unwrap();
    assert_eq!(plan.template, None);
    assert_eq!(
        plan.changes,
        vec![DateChange::Changed {
            block: block.clone(),
            own_time: false
        }]
    );
    // A second change of the same block replaces the first.
    block.items = vec![song(2)];
    apply(
        &mut s,
        Command::ChangeBlockOnDate {
            date: tuesday(),
            block: block.clone(),
        },
    )
    .unwrap();
    let plan = s.schedule.date_plan(tuesday()).unwrap();
    assert_eq!(
        plan.changes,
        vec![DateChange::Changed {
            block,
            own_time: false
        }]
    );
    // The template is untouched.
    assert!(block_at(&s, base, at(8, 0)).items.is_empty());
}

#[test]
fn moving_a_block_on_a_date_sets_own_time() {
    let (mut s, base) = based();
    let mut block = block_at(&s, base, at(8, 0));
    block.start = at(8, 30);
    apply(
        &mut s,
        Command::ChangeBlockOnDate {
            date: tuesday(),
            block: block.clone(),
        },
    )
    .unwrap();
    let plan = s.schedule.date_plan(tuesday()).unwrap();
    assert_eq!(
        plan.changes,
        vec![DateChange::Changed {
            block: block.clone(),
            own_time: true
        }]
    );
    block.start = at(9, 0);
    assert_eq!(
        apply(
            &mut s,
            Command::ChangeBlockOnDate {
                date: tuesday(),
                block
            }
        ),
        Err(ModelError::BlockTimeTaken)
    );
}

#[test]
fn a_change_checks_items_and_ids() {
    let (mut s, base) = based();
    let mut block = block_at(&s, base, at(8, 0));
    block.items = vec![
        Item::Rotation {
            rotation: fp_model::RotationId(1),
        },
        song(1),
    ];
    assert_eq!(
        apply(
            &mut s,
            Command::ChangeBlockOnDate {
                date: tuesday(),
                block
            }
        ),
        Err(ModelError::RotationNotLast)
    );
    let stranger = Block {
        id: BlockId(999),
        start: at(12, 0),
        items: Vec::new(),
    };
    assert_eq!(
        apply(
            &mut s,
            Command::ChangeBlockOnDate {
                date: tuesday(),
                block: stranger
            }
        ),
        Err(ModelError::UnknownBlock(BlockId(999)))
    );
    assert!(s.schedule.date_plan(tuesday()).is_none());
}

#[test]
fn removing_and_reverting_a_template_block() {
    let (mut s, base) = based();
    let nine = block_at(&s, base, at(9, 0)).id;
    let date = tuesday();
    apply(&mut s, Command::RemoveBlockOnDate { date, block: nine }).unwrap();
    assert_eq!(
        s.schedule.date_plan(date).unwrap().changes,
        vec![DateChange::Removed(nine)]
    );
    apply(&mut s, Command::RevertBlockOnDate { date, block: nine }).unwrap();
    assert!(s.schedule.date_plan(date).is_none());
    assert_eq!(
        apply(&mut s, Command::RevertBlockOnDate { date, block: nine }),
        Err(ModelError::UnknownBlock(nine))
    );
}

#[test]
fn removing_a_changed_block_replaces_the_change() {
    let (mut s, base) = based();
    let mut block = block_at(&s, base, at(8, 0));
    block.items = vec![song(1)];
    let date = tuesday();
    apply(
        &mut s,
        Command::ChangeBlockOnDate {
            date,
            block: block.clone(),
        },
    )
    .unwrap();
    apply(
        &mut s,
        Command::RemoveBlockOnDate {
            date,
            block: block.id,
        },
    )
    .unwrap();
    assert_eq!(
        s.schedule.date_plan(date).unwrap().changes,
        vec![DateChange::Removed(block.id)]
    );
}

#[test]
fn added_blocks_belong_to_the_date() {
    let (mut s, _) = based();
    let date = tuesday();
    apply(
        &mut s,
        Command::AddBlockOnDate {
            date,
            start: at(10, 0),
        },
    )
    .unwrap();
    let changes = s.schedule.date_plan(date).unwrap().changes.clone();
    let [DateChange::Added(added)] = changes.as_slice() else {
        panic!("{changes:?}")
    };
    assert_eq!(added.start, at(10, 0));
    // Changing it updates the same entry.
    let mut moved = added.clone();
    moved.start = at(10, 30);
    moved.items = vec![song(3)];
    apply(
        &mut s,
        Command::ChangeBlockOnDate {
            date,
            block: moved.clone(),
        },
    )
    .unwrap();
    assert_eq!(
        s.schedule.date_plan(date).unwrap().changes,
        vec![DateChange::Added(moved.clone())]
    );
    apply(
        &mut s,
        Command::RemoveBlockOnDate {
            date,
            block: moved.id,
        },
    )
    .unwrap();
    assert!(s.schedule.date_plan(date).is_none());
    assert_eq!(
        apply(
            &mut s,
            Command::AddBlockOnDate {
                date,
                start: at(8, 0)
            }
        ),
        Err(ModelError::BlockTimeTaken)
    );
}

#[test]
fn dates_naming_a_deleted_template_fall_back() {
    let (mut s, base) = based();
    let weekend = weekend(&mut s);
    let date = tuesday();
    apply(
        &mut s,
        Command::SetWeekTemplate {
            weekday: Weekday::Sat,
            template: weekend,
        },
    )
    .unwrap();
    apply(
        &mut s,
        Command::SetDateTemplate {
            date,
            template: Some(weekend),
        },
    )
    .unwrap();
    apply(
        &mut s,
        Command::SetWeekTemplate {
            weekday: Weekday::Sat,
            template: base,
        },
    )
    .unwrap();
    apply(&mut s, Command::DeleteTemplate(weekend)).unwrap();
    assert!(s.schedule.date_plan(date).is_none());
    assert_eq!(s.schedule.template_for(date), Some(base));
}
