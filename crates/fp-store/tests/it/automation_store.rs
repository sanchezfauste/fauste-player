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

/// A step counted 0–1 is kept as it is; one counted 0–0 could never play
/// and loads as 0–1.
#[test]
fn a_step_from_zero_loads_and_a_zero_maximum_is_raised() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let (state, _) = sample();
    save_all(&mut s, &state, 0.0);
    let night = state.rotations.default_id().unwrap();
    edit_doc(&s, "rotations", night.0, |doc| {
        doc["steps"][0]["min"] = 0.into();
        doc["steps"][0]["max"] = 1.into();
    });
    let loaded = store(&dir).load("Main");
    let step = loaded.state.rotations.get(night).unwrap().steps[0];
    assert_eq!((step.min, step.max), (0, 1));
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    edit_doc(&s, "rotations", night.0, |doc| {
        doc["steps"][0]["max"] = 0.into();
    });
    let loaded = store(&dir).load("Main");
    let step = loaded.state.rotations.get(night).unwrap().steps[0];
    assert_eq!((step.min, step.max), (0, 1));
}

#[test]
fn a_schema_2_database_migrates_with_no_rotations() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let state = AppState::new(Config::default(), "Main");
    save_all(&mut s, &state, 0.0);
    edit_db(&s, |db| {
        db.conn()
            .execute_batch(
                "DROP TABLE rotations; DROP TABLE air_times; \
                 DROP TABLE day_templates; DROP TABLE date_plans; DROP TABLE as_run; \
                 DELETE FROM kv WHERE key IN ('default_rotation', 'week_plan'); \
                 PRAGMA user_version = 2;",
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
    assert_eq!(
        count(&s, "PRAGMA user_version"),
        i64::from(fp_store::SCHEMA_VERSION)
    );
}

/// A state with the default rotation, music tracks and a filler that has
/// played two rows.
fn with_filler() -> AppState {
    let mut state = AppState::new(Config::default(), "Main");
    ensure_default(&mut state, "Music");
    state.config.automation.track_separation_mins = 0;
    state.config.automation.track_separation_percent = 0;
    state.config.automation.artist_separation_mins = 0;
    for n in 0..20 {
        let id = state.ids.track();
        let mut t = Track::new(id, PathBuf::from(format!("/m/{n}.wav")));
        t.duration_secs = 180.0;
        state.library.insert(t);
    }
    fp_model::filler::ensure(&mut state, "Music filler");
    let player = state.filler.as_ref().unwrap().player;
    for _ in 0..2 {
        apply(&mut state, Command::Play(player)).unwrap();
        fp_model::on_event(&mut state, fp_model::EngineEvent::FadeCompleted { player });
    }
    state
}

#[test]
fn the_filler_round_trips_not_playing_with_its_list() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let state = with_filler();
    let saved = state.filler.clone().unwrap();
    let saved_rows = state.playlists.get(saved.playlist).unwrap().entries.clone();
    save_all(&mut s, &state, 0.0);
    let loaded = store(&dir).load("Main");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    let back = loaded.state.filler.clone().unwrap();
    assert_eq!(back.player, saved.player);
    assert_eq!(back.playlist, saved.playlist);
    assert_eq!(back.generated, saved.generated);
    assert_eq!(back.rotation, saved.rotation);
    assert_eq!(back.cursor, saved.cursor);
    let player = loaded.state.players.last().unwrap();
    assert_eq!(player.id, saved.player);
    assert_ne!(player.transport, fp_model::Transport::Playing);
    assert_eq!(
        loaded.state.playlists.get(saved.playlist).unwrap().entries,
        saved_rows
    );
    assert_eq!(loaded.state.aux_players().count(), 4);
}

#[test]
fn a_lost_filler_record_degrades_to_an_ordinary_playlist() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let state = with_filler();
    let old = state.filler.clone().unwrap();
    save_all(&mut s, &state, 0.0);
    edit_db(&s, |db| {
        db.conn()
            .execute("DELETE FROM kv WHERE key = 'filler'", [])
            .unwrap();
    });
    let mut loaded = store(&dir).load("Main");
    assert!(loaded.state.filler.is_none());
    assert!(loaded.state.playlists.get(old.playlist).is_some());
    fp_model::filler::ensure(&mut loaded.state, "Music filler");
    let new = loaded.state.filler.clone().unwrap();
    assert_ne!(new.playlist, old.playlist);
    assert!(!loaded.state.is_filler_playlist(old.playlist));
}

#[test]
fn the_schedule_player_round_trips_stopped() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let mut state = with_filler();
    fp_model::automation::ensure(&mut state, "Schedule");
    let saved = state.automation.clone().unwrap();
    // A row of a block that was playing: rebuilt, not restored.
    let track = state.library.ids().into_iter().next().unwrap();
    let entry = state.ids.entry();
    state
        .playlists
        .get_mut(saved.playlist)
        .unwrap()
        .entries
        .push(fp_model::PlaylistEntry::new(entry, track));
    save_all(&mut s, &state, 0.0);
    let loaded = store(&dir).load("Main");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    let back = loaded.state.automation.clone().unwrap();
    assert_eq!(back.player, saved.player);
    assert_eq!(back.playlist, saved.playlist);
    assert!(!back.running);
    assert!(
        loaded
            .state
            .playlists
            .get(saved.playlist)
            .unwrap()
            .entries
            .is_empty()
    );
    let player = loaded
        .state
        .players
        .iter()
        .find(|p| p.id == saved.player)
        .unwrap();
    assert_ne!(player.transport, fp_model::Transport::Playing);
    assert_eq!(loaded.state.aux_players().count(), 4);
}
