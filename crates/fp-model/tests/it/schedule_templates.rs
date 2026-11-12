#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Automation spec AU5, AU9: day templates and the week plan.

use chrono::Weekday;
use fp_model::local_time::TimeOfDay;
use fp_model::schedule::{Block, Item, ensure_default};
use fp_model::{AppState, Command, Config, ModelError, RotationId, TemplateId, TrackId, apply};

const DAYS: [Weekday; 7] = [
    Weekday::Mon,
    Weekday::Tue,
    Weekday::Wed,
    Weekday::Thu,
    Weekday::Fri,
    Weekday::Sat,
    Weekday::Sun,
];

fn at(h: u32, m: u32) -> TimeOfDay {
    TimeOfDay::hms(h, m, 0).unwrap()
}

fn based() -> (AppState, TemplateId) {
    let mut s = AppState::new(Config::default(), "P");
    ensure_default(&mut s, "Base");
    let base = s.schedule.templates()[0].id;
    (s, base)
}

fn starts(s: &AppState, t: TemplateId) -> Vec<TimeOfDay> {
    s.schedule
        .template(t)
        .unwrap()
        .blocks
        .iter()
        .map(|b| b.start)
        .collect()
}

fn newest_template(s: &AppState) -> TemplateId {
    s.schedule.templates().last().unwrap().id
}

#[test]
fn a_fresh_install_has_base_on_every_day() {
    let (mut s, base) = based();
    ensure_default(&mut s, "Base");
    assert_eq!(s.schedule.templates().len(), 1);
    assert_eq!(s.schedule.templates()[0].name, "Base");
    for day in DAYS {
        assert_eq!(s.schedule.week().get(day), Some(base));
    }
}

#[test]
fn blocks_stay_sorted_and_unique() {
    let (mut s, base) = based();
    apply(
        &mut s,
        Command::AddBlock {
            template: base,
            start: at(8, 0),
        },
    )
    .unwrap();
    apply(
        &mut s,
        Command::AddBlock {
            template: base,
            start: at(7, 30),
        },
    )
    .unwrap();
    assert_eq!(starts(&s, base), vec![at(7, 30), at(8, 0)]);
    assert_eq!(
        apply(
            &mut s,
            Command::AddBlock {
                template: base,
                start: at(8, 0)
            }
        ),
        Err(ModelError::BlockTimeTaken)
    );
    let mut early = s.schedule.template(base).unwrap().blocks[0].clone();
    early.start = at(8, 0);
    assert_eq!(
        apply(
            &mut s,
            Command::UpdateBlock {
                template: base,
                block: early
            }
        ),
        Err(ModelError::BlockTimeTaken)
    );
    let mut early = s.schedule.template(base).unwrap().blocks[0].clone();
    early.start = at(9, 0);
    apply(
        &mut s,
        Command::UpdateBlock {
            template: base,
            block: early,
        },
    )
    .unwrap();
    assert_eq!(starts(&s, base), vec![at(8, 0), at(9, 0)]);
}

#[test]
fn update_block_checks_items() {
    let (mut s, base) = based();
    apply(
        &mut s,
        Command::AddBlock {
            template: base,
            start: at(8, 0),
        },
    )
    .unwrap();
    let before = s.schedule.template(base).unwrap().blocks[0].clone();
    let block = Block {
        items: vec![
            Item::Rotation {
                rotation: RotationId(1),
            },
            Item::Track {
                track: TrackId(1),
                time_critical: false,
            },
        ],
        ..before.clone()
    };
    assert_eq!(
        apply(
            &mut s,
            Command::UpdateBlock {
                template: base,
                block
            }
        ),
        Err(ModelError::RotationNotLast)
    );
    assert_eq!(s.schedule.template(base).unwrap().blocks[0], before);
}

#[test]
fn removing_a_block() {
    let (mut s, base) = based();
    apply(
        &mut s,
        Command::AddBlock {
            template: base,
            start: at(8, 0),
        },
    )
    .unwrap();
    let id = s.schedule.template(base).unwrap().blocks[0].id;
    apply(
        &mut s,
        Command::RemoveBlock {
            template: base,
            block: id,
        },
    )
    .unwrap();
    assert!(starts(&s, base).is_empty());
    assert_eq!(
        apply(
            &mut s,
            Command::RemoveBlock {
                template: base,
                block: id
            }
        ),
        Err(ModelError::UnknownBlock(id))
    );
}

#[test]
fn duplicate_gives_fresh_block_ids() {
    let (mut s, base) = based();
    apply(
        &mut s,
        Command::AddBlock {
            template: base,
            start: at(8, 0),
        },
    )
    .unwrap();
    apply(
        &mut s,
        Command::AddBlock {
            template: base,
            start: at(9, 0),
        },
    )
    .unwrap();
    apply(
        &mut s,
        Command::DuplicateTemplate {
            template: base,
            name: "Weekend".into(),
        },
    )
    .unwrap();
    let copy = newest_template(&s);
    assert_ne!(copy, base);
    let original = s.schedule.template(base).unwrap().clone();
    let copied = s.schedule.template(copy).unwrap().clone();
    assert_eq!(copied.name, "Weekend");
    assert_eq!(starts(&s, copy), starts(&s, base));
    for (a, b) in original.blocks.iter().zip(&copied.blocks) {
        assert_eq!(a.items, b.items);
    }
    assert!(
        copied
            .blocks
            .iter()
            .all(|b| original.blocks.iter().all(|o| o.id != b.id))
    );
}

#[test]
fn a_template_in_the_week_plan_cannot_be_deleted() {
    let (mut s, base) = based();
    assert_eq!(
        apply(&mut s, Command::DeleteTemplate(base)),
        Err(ModelError::LastTemplate)
    );
    apply(
        &mut s,
        Command::CreateTemplate {
            name: "Weekend".into(),
        },
    )
    .unwrap();
    let weekend = newest_template(&s);
    apply(
        &mut s,
        Command::SetWeekTemplate {
            weekday: Weekday::Sat,
            template: weekend,
        },
    )
    .unwrap();
    assert_eq!(
        apply(&mut s, Command::DeleteTemplate(weekend)),
        Err(ModelError::TemplateInUse)
    );
    apply(
        &mut s,
        Command::SetWeekTemplate {
            weekday: Weekday::Sat,
            template: base,
        },
    )
    .unwrap();
    apply(&mut s, Command::DeleteTemplate(weekend)).unwrap();
    assert!(s.schedule.template(weekend).is_none());
    assert_eq!(
        apply(
            &mut s,
            Command::SetWeekTemplate {
                weekday: Weekday::Sat,
                template: weekend
            }
        ),
        Err(ModelError::UnknownTemplate(weekend))
    );
}

#[test]
fn template_names_are_trimmed_and_required() {
    let (mut s, base) = based();
    assert_eq!(
        apply(&mut s, Command::CreateTemplate { name: "  ".into() }),
        Err(ModelError::EmptyTemplateName)
    );
    apply(
        &mut s,
        Command::RenameTemplate {
            template: base,
            name: " Weekend ".into(),
        },
    )
    .unwrap();
    assert_eq!(s.schedule.template(base).unwrap().name, "Weekend");
}
