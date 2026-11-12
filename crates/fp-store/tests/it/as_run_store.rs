#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Automation spec AU36, AU38: the as-run log in the database (schema 5).

use std::path::PathBuf;

use chrono::{Datelike, NaiveDate};
use fp_model::{AppState, Command, Config, Track, TrackId, TrackKind, apply};
use fp_store::{AppPaths, Store};

use crate::support::{edit_db, save_all};

fn store(dir: &tempfile::TempDir) -> Store {
    Store::new(AppPaths::under(dir.path()), Config::default().limits)
}

fn count(s: &Store, sql: &str) -> i64 {
    let db = fp_store::Db::open(&s.paths().database_file()).unwrap();
    db.conn().query_row(sql, [], |r| r.get(0)).unwrap()
}

fn day(d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 11, d).unwrap()
}

/// Automation started on 17 November at 08:00: the filler's first song
/// is the log's first entry.
fn started() -> AppState {
    let mut s = AppState::new(Config::default(), "Main");
    let mut t = Track::new(TrackId(900), PathBuf::from("/m/a.wav"));
    t.duration_secs = 180.0;
    t.kind = TrackKind::Music;
    s.library.insert(t);
    s.ids.observe(1000);
    fp_model::rotation::ensure_default(&mut s, "Music");
    fp_model::filler::ensure(&mut s, "Music filler");
    fp_model::automation::ensure(&mut s, "Schedule");
    fp_model::schedule::ensure_default(&mut s, "Base");
    s.local_now = day(17).and_hms_opt(8, 0, 0).unwrap();
    let mut actions = apply(&mut s, Command::StartAutomation).unwrap();
    // The first tick plays the filler; the engine reports its song on air.
    actions.extend(fp_model::automation::tick(&mut s).0);
    for action in actions {
        if let fp_model::EngineAction::StartCurrent { player, request } = action {
            fp_model::on_event(
                &mut s,
                fp_model::EngineEvent::TransitionStarted {
                    player,
                    entry: request.entry,
                    normalization: fp_model::Normalization::default(),
                },
            );
        }
    }
    assert_eq!(s.as_run.entries().count(), 1);
    s
}

#[test]
fn entries_are_appended_once() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let state = started();
    save_all(&mut s, &state, 0.0);
    save_all(&mut s, &state, 0.0);
    assert_eq!(count(&s, "SELECT COUNT(*) FROM as_run"), 1);
    // A new store (a restart) writes everything again: still once.
    let mut again = store(&dir);
    let loaded = again.load("Main");
    save_all(&mut again, &state, 0.0);
    assert_eq!(count(&again, "SELECT COUNT(*) FROM as_run"), 1);
    let _ = loaded;
}

#[test]
fn the_next_sequence_survives_a_restart() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let state = started();
    save_all(&mut s, &state, 0.0);
    let loaded = store(&dir).load("Main");
    assert_eq!(loaded.state.as_run.next_seq(), state.as_run.next_seq());
    assert!(loaded.state.as_run.next_seq() >= 1);
}

#[test]
fn rows_older_than_the_keep_days_are_pruned() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let mut state = started();
    state.config.automation.as_run_keep_days = 3;
    save_all(&mut s, &state, 0.0);
    edit_db(&s, |db| {
        for (seq, d) in [(500i64, 10u32), (501, 14), (502, 15)] {
            db.conn()
                .execute(
                    "INSERT INTO as_run (seq, date, doc) VALUES (?1, ?2, '{}')",
                    rusqlite::params![seq, day(d).num_days_from_ce()],
                )
                .unwrap();
        }
    });
    // The next save keeps only the last three days: the 15th on.
    let mut s = store(&dir);
    let _ = s.load("Main");
    save_all(&mut s, &state, 0.0);
    let dates: Vec<i64> = {
        let db = fp_store::Db::open(&s.paths().database_file()).unwrap();
        let mut stmt = db
            .conn()
            .prepare("SELECT date FROM as_run ORDER BY date")
            .unwrap();
        stmt.query_map([], |r| r.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect()
    };
    let from = i64::from(day(15).num_days_from_ce());
    assert!(dates.iter().all(|d| *d >= from), "{dates:?}");
    assert_eq!(dates.len(), 2);
}

#[test]
fn a_damaged_row_is_skipped_with_a_warning() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let state = started();
    save_all(&mut s, &state, 0.0);
    edit_db(&s, |db| {
        db.conn()
            .execute(
                "INSERT INTO as_run (seq, date, doc) VALUES (900, ?1, 'garbage')",
                [day(17).num_days_from_ce()],
            )
            .unwrap();
    });
    let db = fp_store::Db::open(&s.paths().database_file()).unwrap();
    let (entries, warnings) = fp_store::read_as_run(&db, day(1));
    assert_eq!(entries.len(), 1);
    assert_eq!(warnings.len(), 1);
}

/// A clock that jumps a year ahead while the app runs does not delete the
/// log: the cutoff moves no faster than real time (AU38, AU40).
#[test]
fn a_forward_clock_jump_does_not_prune_the_log() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let mut state = started();
    state.config.automation.as_run_keep_days = 3;
    let t0 = std::time::Instant::now();
    s.save_config(&state).unwrap();
    s.save_state(&state, |_| 0.0, t0).unwrap();
    state.local_now = NaiveDate::from_ymd_opt(2027, 11, 17)
        .unwrap()
        .and_hms_opt(8, 1, 0)
        .unwrap();
    let a_minute = t0 + std::time::Duration::from_secs(60);
    s.save_state(&state, |_| 0.0, a_minute).unwrap();
    assert_eq!(count(&s, "SELECT COUNT(*) FROM as_run"), 1);
    // Once the days have really passed, the old rows go.
    let five_days = t0 + std::time::Duration::from_secs(5 * 86_400);
    s.save_state(&state, |_| 0.0, five_days).unwrap();
    assert_eq!(count(&s, "SELECT COUNT(*) FROM as_run"), 0);
}
