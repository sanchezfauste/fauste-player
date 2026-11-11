#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Rotations, the default rotation and air times in the database (schema 3).

use std::path::PathBuf;

use fp_model::rotation::{RotationStep, StepSource, ensure_default};
use fp_model::{AppState, Command, Config, Track, TrackKind, WallTime, apply};
use fp_store::{AppPaths, Store};

use crate::support::{edit_db, edit_doc, save_all};

fn store(dir: &tempfile::TempDir) -> Store {
    Store::new(AppPaths::under(dir.path()), Config::default().limits)
}

fn count(s: &Store, sql: &str) -> i64 {
    let db = fp_store::Db::open(&s.paths().database_file()).unwrap();
    db.conn().query_row(sql, [], |r| r.get(0)).unwrap()
}

/// A state with the default rotation, a second one made the default, and
/// one track that aired.
fn sample() -> (AppState, fp_model::TrackId) {
    let mut state = AppState::new(Config::default(), "Main");
    ensure_default(&mut state, "Music");
    apply(
        &mut state,
        Command::CreateRotation {
            name: "Night".into(),
            steps: vec![RotationStep {
                source: StepSource::Kind(TrackKind::Jingle),
                min: 2,
                max: 3,
            }],
        },
    )
    .unwrap();
    let night = state.rotations.iter().last().unwrap().id;
    apply(&mut state, Command::SetDefaultRotation(night)).unwrap();
    let id = state.ids.track();
    state
        .library
        .insert(Track::new(id, PathBuf::from("/m/a.wav")));
    state.air_times.record(id, WallTime(1_700_000_000_000));
    (state, id)
}

#[test]
fn rotations_and_air_times_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let (state, _) = sample();
    save_all(&mut s, &state, 0.0);
    let loaded = store(&dir).load("Main");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    assert_eq!(loaded.state.rotations, state.rotations);
    assert_eq!(loaded.state.air_times, state.air_times);
    // A new id never collides with a rotation's.
    let mut ids = loaded.state.ids.clone();
    assert!(ids.next_raw() > loaded.state.rotations.iter().map(|r| r.id.0).max().unwrap());
}

#[test]
fn a_changed_air_time_writes_one_row() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let (mut state, id) = sample();
    save_all(&mut s, &state, 0.0);
    state.air_times.record(id, WallTime(1_800_000_000_000));
    save_all(&mut s, &state, 0.0);
    assert_eq!(count(&s, "SELECT COUNT(*) FROM air_times"), 1);
    assert_eq!(count(&s, "SELECT at FROM air_times"), 1_800_000_000_000);
    state.library.remove(id);
    state.air_times.remove(id);
    save_all(&mut s, &state, 0.0);
    assert_eq!(count(&s, "SELECT COUNT(*) FROM air_times"), 0);
}

#[test]
fn damaged_rotation_and_air_time_rows_are_repaired_or_dropped() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let (state, _) = sample();
    save_all(&mut s, &state, 0.0);
    let night = state.rotations.default_id().unwrap();
    // min above max, and a count out of range.
    edit_doc(&s, "rotations", night.0, |doc| {
        doc["steps"][0]["min"] = 5.into();
        doc["steps"][0]["max"] = 2.into();
    });
    edit_db(&s, |db| {
        db.conn()
            .execute(
                "INSERT INTO rotations (id, position, doc) VALUES (9001, 9, '{not json')",
                [],
            )
            .unwrap();
        db.conn()
            .execute(
                "INSERT INTO rotations (id, position, doc) VALUES (9002, 10, \
                 '{\"id\":9002,\"name\":\"Odd\",\"steps\":[{\"source\":{\"Planet\":3},\"min\":1,\"max\":1}]}')",
                [],
            )
            .unwrap();
        db.conn()
            .execute("INSERT INTO air_times (track, at) VALUES (424242, 5)", [])
            .unwrap();
    });
    let loaded = store(&dir).load("Main");
    let rotations = &loaded.state.rotations;
    let repaired = rotations.get(night).unwrap();
    assert_eq!((repaired.steps[0].min, repaired.steps[0].max), (2, 5));
    assert!(rotations.get(fp_model::RotationId(9001)).is_none());
    assert!(rotations.get(fp_model::RotationId(9002)).is_none());
    assert_eq!(loaded.state.air_times.len(), 1);
    let text = loaded.warnings.join("\n");
    assert!(text.contains("9001"), "{text}");
    assert!(text.contains("9002"), "{text}");
    assert!(text.contains("rotation"), "{text}");
}

#[test]
fn a_schema_2_database_migrates_to_3_with_no_rotations() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let state = AppState::new(Config::default(), "Main");
    save_all(&mut s, &state, 0.0);
    edit_db(&s, |db| {
        db.conn()
            .execute_batch(
                "DROP TABLE rotations; DROP TABLE air_times; \
                 DELETE FROM kv WHERE key = 'default_rotation'; PRAGMA user_version = 2;",
            )
            .unwrap();
    });
    let mut loaded = store(&dir).load("Main");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    assert_eq!(loaded.state.rotations.iter().count(), 0);
    ensure_default(&mut loaded.state, "Music");
    assert_eq!(
        loaded.state.rotations.default_rotation().unwrap().name,
        "Music"
    );
    assert_eq!(count(&s, "PRAGMA user_version"), 3);
}
