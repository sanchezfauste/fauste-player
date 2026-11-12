#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The schedule in the database (automation spec §10, schema 4).

use chrono::{NaiveDate, Weekday};
use fp_model::local_time::TimeOfDay;
use fp_model::schedule::{Block, Item, ensure_default, prune_dates};
use fp_model::{AppState, CategoryId, Command, Config, RotationId, TemplateId, TrackId, apply};
use fp_store::{AppPaths, Store};

use crate::support::{edit_db, edit_doc, save_all};

fn store(dir: &tempfile::TempDir) -> Store {
    Store::new(AppPaths::under(dir.path()), Config::default().limits)
}

fn count(s: &Store, sql: &str) -> i64 {
    let db = fp_store::Db::open(&s.paths().database_file()).unwrap();
    db.conn().query_row(sql, [], |r| r.get(0)).unwrap()
}

fn at(h: u32, m: u32) -> TimeOfDay {
    TimeOfDay::hms(h, m, 0).unwrap()
}

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
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

/// "Base" (07:00 with all three item kinds, 08:00, 09:00), "Weekend" on
/// Saturdays, and a Tuesday that changes 08:00, removes 09:00 and adds 10:00.
fn sample() -> (AppState, TemplateId) {
    let mut s = AppState::new(Config::default(), "Main");
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
    let mut seven = block_at(&s, base, at(7, 0));
    seven.items = vec![
        Item::Track {
            track: TrackId(5),
            time_critical: true,
        },
        Item::Category {
            category: CategoryId(6),
            time_critical: false,
        },
        Item::Rotation {
            rotation: RotationId(7),
        },
    ];
    apply(
        &mut s,
        Command::UpdateBlock {
            template: base,
            block: seven,
        },
    )
    .unwrap();
    apply(
        &mut s,
        Command::CreateTemplate {
            name: "Weekend".into(),
        },
    )
    .unwrap();
    let weekend = s.schedule.templates()[1].id;
    apply(
        &mut s,
        Command::SetWeekTemplate {
            weekday: Weekday::Sat,
            template: weekend,
        },
    )
    .unwrap();
    let tuesday = date(2026, 11, 17);
    let mut eight = block_at(&s, base, at(8, 0));
    eight.items = vec![Item::Track {
        track: TrackId(9),
        time_critical: false,
    }];
    apply(
        &mut s,
        Command::ChangeBlockOnDate {
            date: tuesday,
            block: eight,
        },
    )
    .unwrap();
    let nine = block_at(&s, base, at(9, 0)).id;
    apply(
        &mut s,
        Command::RemoveBlockOnDate {
            date: tuesday,
            block: nine,
        },
    )
    .unwrap();
    apply(
        &mut s,
        Command::AddBlockOnDate {
            date: tuesday,
            start: at(10, 0),
        },
    )
    .unwrap();
    apply(
        &mut s,
        Command::SetDateTemplate {
            date: date(2026, 12, 25),
            template: Some(weekend),
        },
    )
    .unwrap();
    (s, base)
}

#[test]
fn the_schedule_round_trips() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let (state, _) = sample();
    save_all(&mut s, &state, 0.0);
    let loaded = store(&dir).load("Main");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    assert_eq!(loaded.state.schedule, state.schedule);
    // Ids handed out after the load do not collide with saved blocks.
    assert!(loaded.state.ids.clone().next_raw() > state.schedule.max_raw_id());
}

#[test]
fn schedule_edits_are_saved_incrementally() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let (mut state, base) = sample();
    save_all(&mut s, &state, 0.0);
    apply(
        &mut state,
        Command::AddBlock {
            template: base,
            start: at(11, 0),
        },
    )
    .unwrap();
    let weekend = state.schedule.templates()[1].id;
    apply(
        &mut state,
        Command::SetWeekTemplate {
            weekday: Weekday::Sun,
            template: weekend,
        },
    )
    .unwrap();
    apply(
        &mut state,
        Command::SetDateTemplate {
            date: date(2026, 12, 25),
            template: None,
        },
    )
    .unwrap();
    save_all(&mut s, &state, 0.0);
    let loaded = store(&dir).load("Main");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    assert_eq!(loaded.state.schedule, state.schedule);
    assert_eq!(count(&s, "SELECT COUNT(*) FROM date_plans"), 1);
}

#[test]
fn a_schema_3_database_migrates_to_4_with_no_templates() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let state = AppState::new(Config::default(), "Main");
    save_all(&mut s, &state, 0.0);
    edit_db(&s, |db| {
        db.conn()
            .execute_batch(
                "DROP TABLE day_templates; DROP TABLE date_plans; \
                 DELETE FROM kv WHERE key = 'week_plan'; PRAGMA user_version = 3;",
            )
            .unwrap();
    });
    let mut loaded = store(&dir).load("Main");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    assert!(loaded.state.schedule.templates().is_empty());
    ensure_default(&mut loaded.state, "Base");
    assert_eq!(loaded.state.schedule.templates()[0].name, "Base");
    assert_eq!(count(&s, "PRAGMA user_version"), 4);
}

#[test]
fn pruned_dates_leave_the_table() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let (mut state, _) = sample();
    apply(
        &mut state,
        Command::AddBlockOnDate {
            date: date(2026, 1, 1),
            start: at(12, 0),
        },
    )
    .unwrap();
    save_all(&mut s, &state, 0.0);
    assert_eq!(count(&s, "SELECT COUNT(*) FROM date_plans"), 3);
    assert_eq!(prune_dates(&mut state, date(2026, 11, 12)), 1);
    save_all(&mut s, &state, 0.0);
    assert_eq!(count(&s, "SELECT COUNT(*) FROM date_plans"), 2);
}

#[test]
fn damaged_schedule_rows_are_repaired_or_dropped() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let (state, base) = sample();
    save_all(&mut s, &state, 0.0);
    let weekend = state.schedule.templates()[1].id;
    // A block at 25:00, and two blocks at 08:00.
    edit_doc(&s, "day_templates", base.0, |doc| {
        let blocks = doc["blocks"].as_array_mut().unwrap();
        blocks[0]["start"] = serde_json::json!(90_000);
        blocks[2]["start"] = serde_json::json!(8 * 3600);
    });
    // The week plan names a missing template; a date row's key disagrees
    // with its date; the weekend template's record is not JSON.
    let christmas = i64::from(chrono::Datelike::num_days_from_ce(&date(2026, 12, 25)));
    edit_db(&s, |db| {
        db.conn()
            .execute(
                "UPDATE kv SET value = ?1 WHERE key = 'week_plan'",
                [format!(
                    "[999,{b},{b},{b},{b},{w},{b}]",
                    b = base.0,
                    w = weekend.0
                )],
            )
            .unwrap();
        db.conn()
            .execute(
                "UPDATE date_plans SET id = ?1 WHERE id = ?2",
                [christmas + 1, christmas],
            )
            .unwrap();
        db.conn()
            .execute(
                "UPDATE day_templates SET doc = 'nope' WHERE id = ?1",
                [weekend.0 as i64],
            )
            .unwrap();
    });
    let loaded = store(&dir).load("Main");
    let w = &loaded.warnings;
    assert_eq!(w.len(), 6, "{w:?}");
    let schedule = &loaded.state.schedule;
    assert_eq!(schedule.templates().len(), 1);
    let starts: Vec<_> = schedule.templates()[0]
        .blocks
        .iter()
        .map(|b| b.start)
        .collect();
    assert_eq!(starts, vec![at(8, 0)]);
    // Monday's missing template and Saturday's dropped one: the first.
    assert_eq!(schedule.week().get(Weekday::Mon), Some(base));
    assert_eq!(schedule.week().get(Weekday::Sat), Some(base));
    assert!(schedule.date_plan(date(2026, 12, 25)).is_none());
    assert!(schedule.date_plan(date(2026, 12, 26)).is_none());
    assert!(schedule.date_plan(date(2026, 11, 17)).is_some());
}
