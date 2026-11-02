#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::PathBuf;

use fp_model::{AppState, Command, Config, EngineAction, Transport, apply};
use fp_store::{AppPaths, Store};

use crate::support::{edit_db, edit_doc, save_all};

fn store(dir: &tempfile::TempDir) -> Store {
    Store::new(AppPaths::under(dir.path()), Config::default().limits)
}

#[test]
fn first_run_starts_with_defaults_and_no_warnings() {
    let dir = tempfile::tempdir().unwrap();
    let loaded = store(&dir).load("Main");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    assert_eq!(loaded.state.players.len(), 4);
    assert_eq!(loaded.state.playlists.len(), 1);
}

#[test]
fn saved_state_is_restored_paused_at_its_position() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let mut state = AppState::new(Config::default(), "Main");
    let playlist = state.playlists.first_id().unwrap();
    let paths = vec![PathBuf::from("/m/a.flac"), PathBuf::from("/m/b.flac")];
    apply(
        &mut state,
        Command::InsertPaths {
            playlist,
            index: 0,
            paths,
        },
    )
    .unwrap();
    for track_id in state.library.ids() {
        let t = state.library.get_mut(track_id).unwrap();
        t.duration_secs = 200.0;
    }
    let p = state.players[0].id;
    apply(&mut state, Command::Play(p)).unwrap();

    save_all(&mut s, &state, 42.5);

    let loaded = s.load("Main");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    assert_eq!(loaded.state.library, state.library);
    assert_eq!(loaded.state.playlists, state.playlists);
    let restored = loaded.state.player(p).unwrap();
    assert_eq!(restored.transport, Transport::Paused);
    assert_eq!(restored.current, state.players[0].current);
    assert!(loaded.actions.iter().any(|a| matches!(
        a,
        EngineAction::LoadPaused { player, request } if *player == p && (request.from_secs - 42.5).abs() < 1e-9
    )));
}

/// I2: a silent file's measurement (no integrated loudness) and the pass
/// version survive a save and a load of the track rows.
#[test]
fn the_loudness_measurement_round_trips() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let mut state = AppState::new(Config::default(), "Main");
    let playlist = state.playlists.first_id().unwrap();
    let paths = vec![PathBuf::from("/m/silent.flac"), PathBuf::from("/m/b.flac")];
    apply(
        &mut state,
        Command::InsertPaths {
            playlist,
            index: 0,
            paths,
        },
    )
    .unwrap();
    {
        let ids = state.library.ids();
        state.library.get_mut(ids[0]).unwrap().apply_loudness(
            Some(fp_model::LoudnessMeasurement {
                integrated_lufs: None,
                true_peak_dbtp: -60.5,
            }),
            fp_model::LOUDNESS_VERSION,
        );
        state.library.get_mut(ids[1]).unwrap().apply_loudness(
            Some(fp_model::LoudnessMeasurement {
                integrated_lufs: Some(-14.25),
                true_peak_dbtp: -0.5,
            }),
            fp_model::LOUDNESS_VERSION,
        );
    }
    save_all(&mut s, &state, 0.0);
    let loaded = s.load("Main");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    assert_eq!(loaded.state.library, state.library);
}

#[test]
fn newer_schema_is_not_quarantined() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let path = s.paths().config_file();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, r#"{"schema_version":99,"config":{}}"#).unwrap();
    let loaded = s.load("Main");
    assert!(
        loaded.warnings.iter().any(|w| w.contains("newer")),
        "{:?}",
        loaded.warnings
    );
    assert!(path.exists());
    assert_eq!(loaded.state.config, Config::default());
}

#[test]
fn corrupt_config_falls_back_to_defaults_with_a_warning() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let path = s.paths().config_file();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "not json").unwrap();
    let loaded = s.load("Main");
    assert!(!loaded.warnings.is_empty());
    assert_eq!(loaded.state.config, Config::default());
    let quarantined = fs::read_dir(path.parent().unwrap()).unwrap().any(|e| {
        e.unwrap()
            .file_name()
            .to_string_lossy()
            .contains(".corrupt-")
    });
    assert!(quarantined);
}

#[test]
fn invalid_values_are_clamped_on_load() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let path = s.paths().config_file();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        r#"{"schema_version":1,"config":{"players":{"count":0}}}"#,
    )
    .unwrap();
    let loaded = s.load("Main");
    assert_eq!(loaded.state.players.len(), 1);
    assert!(
        loaded.warnings.iter().any(|w| w.contains("players.count")),
        "{:?}",
        loaded.warnings
    );
}

fn write_config(s: &Store, json: &str) {
    let path = s.paths().config_file();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, json).unwrap();
}

#[test]
fn hand_edited_values_of_the_wrong_type_are_repaired_not_discarded() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    write_config(
        &s,
        r#"{"schema_version":1,"config":{
            "players":{"count":-1,"fade_ms":1500.5,"end_warning_secs":"soon"},
            "outputs":{"sample_rate":1e3,"buffer_frames":99999999999},
            "ui":{"wave_color":"cyan"}}}"#,
    );
    let loaded = s.load("Main");
    let c = &loaded.state.config;
    assert_eq!(c.players.count, 1, "negative count clamps to the minimum");
    assert_eq!(c.players.fade_ms, 1501, "fractional values are rounded");
    assert_eq!(
        c.players.end_warning_secs, 10.0,
        "an invalid value falls back to its default"
    );
    assert_eq!(
        c.outputs.sample_rate, 8000,
        "1e3 is read as 1000 then clamped"
    );
    assert_eq!(
        c.outputs.buffer_frames, 16_384,
        "a huge value saturates then clamps"
    );
    assert_eq!(
        c.ui.wave_color, "cyan",
        "valid fields next to invalid ones are kept"
    );
    assert!(
        loaded
            .warnings
            .iter()
            .any(|w| w.contains("end_warning_secs")),
        "{:?}",
        loaded.warnings
    );
    assert!(
        s.paths().config_file().exists(),
        "a repairable file is not quarantined"
    );
}

#[test]
fn an_unknown_window_system_loads_as_auto_with_a_warning() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    write_config(
        &s,
        r#"{"schema_version":1,"config":{"ui":{"window_system":"Mir","wave_color":"cyan"}}}"#,
    );
    let loaded = s.load("Main");
    let c = &loaded.state.config;
    assert_eq!(c.ui.window_system, fp_model::WindowSystem::Auto);
    assert_eq!(c.ui.wave_color, "cyan");
    assert!(
        loaded.warnings.iter().any(|w| w.contains("window_system")),
        "{:?}",
        loaded.warnings
    );
}

#[test]
fn a_newer_file_is_preserved_beyond_backup_rotation() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let newer = r#"{"schema_version":99,"config":{}}"#;
    write_config(&s, newer);
    let loaded = s.load("Main");
    for _ in 0..5 {
        s.save_config(&loaded.state).unwrap();
    }
    let preserved: Vec<_> = fs::read_dir(s.paths().config_dir.clone())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.to_string_lossy().contains(".newer-"))
        .collect();
    assert_eq!(preserved.len(), 1, "{preserved:?}");
    assert_eq!(fs::read_to_string(&preserved[0]).unwrap(), newer);
}

#[test]
fn an_unknown_shortcut_does_not_discard_the_others() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    write_config(
        &s,
        r#"{ "schema_version": 1, "config": { "shortcuts": [
            { "action": { "PlayPlayer": 1 }, "chord": { "key": "Q" } },
            { "action": "SomethingFromTheFuture", "chord": { "key": "W" } },
            { "action": "StopAllCarts", "chord": { "key": "E", "ctrl": true } }
        ] } }"#,
    );
    let loaded = s.load("Main");
    let keys: Vec<String> = loaded
        .state
        .config
        .shortcuts
        .iter()
        .map(|s| s.chord.to_string())
        .collect();
    assert_eq!(keys, vec!["Q".to_owned(), "Ctrl+E".to_owned()]);
    assert!(
        loaded.warnings.iter().any(|w| w.contains("shortcuts")),
        "{:?}",
        loaded.warnings
    );
}

#[test]
fn bit_perfect_devices_round_trip_and_older_configs_have_none() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    write_config(
        &s,
        r#"{ "schema_version": 1, "config": { "outputs": { "sample_rate": 44100 } } }"#,
    );
    let mut loaded = s.load("Main");
    assert!(loaded.state.config.outputs.bit_perfect.is_empty());
    // Kept only for a device a route names (operator feedback 4, Q12).
    loaded.state.config.outputs.cartwall.main = Some(fp_model::Route {
        backend: "alsa".into(),
        device: "hw:CARD=DAC,DEV=0".into(),
        first_channel: 0,
    });
    loaded.state.config.outputs.bit_perfect = vec![fp_model::OutputDevice {
        backend: "alsa".into(),
        device: "hw:CARD=DAC,DEV=0".into(),
    }];
    s.save_config(&loaded.state).unwrap();
    let again = s.load("Main");
    assert_eq!(
        again.state.config.outputs.bit_perfect,
        loaded.state.config.outputs.bit_perfect
    );
    assert!(again.warnings.is_empty(), "{:?}", again.warnings);
}

/// A saved state with one entry played by player 0.
fn saved_with_a_played_entry(s: &mut Store) -> AppState {
    let mut state = AppState::new(Config::default(), "Main");
    let playlist = state.playlists.first_id().unwrap();
    let paths = vec![PathBuf::from("/m/a.flac"), PathBuf::from("/m/b.flac")];
    apply(
        &mut state,
        Command::InsertPaths {
            playlist,
            index: 0,
            paths,
        },
    )
    .unwrap();
    let entry = state.playlists.get(playlist).unwrap().entries[0].id;
    state.playlists.mark_played(entry, state.players[0].id);
    save_all(s, &state, 0.0);
    state
}

#[test]
fn the_played_marks_of_every_player_round_trip() {
    // Per-player marks: an entry played by player 0 stays unplayed for the
    // other players after a save and a load.
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let state = saved_with_a_played_entry(&mut s);
    let loaded = s.load("Main");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    let playlist = loaded.state.playlists.first_id().unwrap();
    let entries = &loaded.state.playlists.get(playlist).unwrap().entries;
    let players: Vec<_> = state.players.iter().map(|p| p.id).collect();
    assert!(entries[0].is_played_by(players[0]));
    assert!(players[1..].iter().all(|p| !entries[0].is_played_by(*p)));
    assert!(players.iter().all(|p| !entries[1].is_played_by(*p)));
}

#[test]
fn analysis_fields_this_version_dropped_are_ignored_with_a_warning() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    write_config(
        &s,
        r#"{"schema_version":1,"config":{"analysis":{
            "silence_threshold_db":-45.0,"segue_threshold_db":-20.0,"outro_drop_db":8.0}}}"#,
    );
    let loaded = s.load("Main");
    let a = &loaded.state.config.analysis;
    assert_eq!(a.outro_drop_db, 8.0, "valid fields are kept");
    assert_eq!(a.trim_threshold_db, -60.0);
    assert_eq!(a.segue_drop_db, 15.0);
    for old in ["silence_threshold_db", "segue_threshold_db"] {
        assert!(
            loaded
                .warnings
                .iter()
                .any(|w| w.contains(old) && w.contains("not used")),
            "{old}: {:?}",
            loaded.warnings
        );
    }
}

#[test]
fn a_saved_pending_start_that_does_not_parse_is_dropped_with_a_warning() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let state = saved_with_a_played_entry(&mut s);
    edit_doc(&s, "players", state.players[0].id.0, |doc| {
        doc["pending_start"] = serde_json::json!("garbage");
    });

    let loaded = s.load("Main");
    assert!(loaded.state.players[0].pending_start.is_none());
    assert!(
        loaded.warnings.iter().any(|w| w.contains("pending start")),
        "{:?}",
        loaded.warnings
    );
}

#[test]
fn a_devices_own_settings_are_dropped_on_load_when_no_route_names_it() {
    // Operator feedback 4, Q12: written by an earlier version, which kept
    // them for when the device was routed again.
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let path = s.paths().config_file();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        r#"{"schema_version":1,"config":{"outputs":{
            "routes":[{"player":1,"main":{"backend":"alsa","device":"dac","first_channel":0},"cue":null}],
            "bit_perfect":[{"backend":"alsa","device":"dac"},{"backend":"alsa","device":"gone"}],
            "dsd_output":[{"backend":"alsa","device":"gone","mode":"Dop"}],
            "device_overrides":[
                {"device":{"backend":"alsa","device":"dac"},"sample_rate":96000},
                {"device":{"backend":"alsa","device":"gone"},"buffer_frames":256}
            ]}}}"#,
    )
    .unwrap();
    let loaded = s.load("Main");
    let o = &loaded.state.config.outputs;
    let kept: Vec<&str> = o
        .device_overrides
        .iter()
        .map(|d| d.device.device.as_str())
        .collect();
    assert_eq!(kept, ["dac"]);
    let bit_perfect: Vec<&str> = o.bit_perfect.iter().map(|d| d.device.as_str()).collect();
    assert_eq!(bit_perfect, ["dac"]);
    assert!(o.dsd_output.is_empty());
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
}

/// Player 1 plays on `dac`; the last of four players, removed before the
/// save, played on `gone`. Both devices have their own rate.
fn routes_of_a_removed_player(s: &mut Store) -> (AppState, fp_model::PlayerId) {
    use fp_model::{OutputDevice, PlayerRoutes, Route};
    let mut state = AppState::new(Config::default(), "Main");
    let route = |device: &str| {
        Some(Route {
            backend: "alsa".into(),
            device: device.into(),
            first_channel: 0,
        })
    };
    let first = state.players[0].id;
    let removed = state.players[3].id;
    state.config.outputs.routes = vec![
        PlayerRoutes {
            player: first,
            main: route("dac"),
            cue: None,
        },
        PlayerRoutes {
            player: removed,
            main: route("gone"),
            cue: None,
        },
    ];
    for device in ["dac", "gone"] {
        let device = OutputDevice {
            backend: "alsa".into(),
            device: device.into(),
        };
        state.config.outputs.set_device_rate(&device, Some(96_000));
    }
    apply(&mut state, Command::SetPlayerCount(3)).unwrap();
    s.save_config(&state).unwrap();
    (state, removed)
}

#[test]
fn the_routes_of_a_removed_player_and_their_devices_settings_are_dropped_on_load() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let (state, removed) = routes_of_a_removed_player(&mut s);
    s.save_state(&state, |_| 0.0, std::time::Instant::now())
        .unwrap();
    let loaded = s.load("Main");
    let o = &loaded.state.config.outputs;
    assert!(o.player_routes(removed).is_none());
    assert!(o.player_routes(state.players[0].id).is_some());
    let own: Vec<&str> = o
        .device_overrides
        .iter()
        .map(|d| d.device.device.as_str())
        .collect();
    assert_eq!(own, ["dac"]);
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
}

#[test]
fn without_a_session_no_route_is_dropped() {
    // The players' ids are not known (the database has no player rows): a
    // route may be for a player whose session is only missing.
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let (state, removed) = routes_of_a_removed_player(&mut s);
    s.save_state(&state, |_| 0.0, std::time::Instant::now())
        .unwrap();
    edit_db(&s, |db| {
        db.conn().execute("DELETE FROM players", []).unwrap();
    });
    let loaded = s.load("Main");
    let o = &loaded.state.config.outputs;
    assert!(o.player_routes(removed).is_some());
    assert_eq!(o.device_overrides, state.config.outputs.device_overrides);
}

#[test]
fn an_old_restart_handoff_is_ignored_on_load() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    write_config(
        &s,
        r#"{"schema_version":1,"config":{"tuning":{"restart_handoff_ms":5000,"declick_ms":7}}}"#,
    );
    let loaded = s.load("Main");
    assert_eq!(
        loaded.state.config.tuning.declick_ms, 7.0,
        "the rest is kept"
    );
    assert!(
        loaded
            .warnings
            .iter()
            .any(|w| w.contains("restart_handoff_ms")),
        "{:?}",
        loaded.warnings
    );
}

fn backups(s: &Store) -> usize {
    (1..=9)
        .filter(|n| fp_store::backup_path(&s.paths().database_file(), *n).exists())
        .count()
}

#[test]
fn the_first_save_makes_a_backup_and_the_next_waits_for_the_interval() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let state = AppState::new(Config::default(), "Main");
    assert_eq!(state.config.tuning.backup_interval_secs, 300);
    let now = std::time::Instant::now();
    let secs = std::time::Duration::from_secs;
    s.save_state(&state, |_| 0.0, now).unwrap();
    assert_eq!(backups(&s), 1, "the first save");
    s.save_state(&state, |_| 0.0, now + secs(10)).unwrap();
    assert_eq!(backups(&s), 1, "within the interval");
    s.save_state(&state, |_| 0.0, now + secs(301)).unwrap();
    assert_eq!(backups(&s), 2, "after the interval");
}

#[test]
fn later_saves_write_on_top_of_what_load_read() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let state = saved_with_a_played_entry(&mut s);
    let mut loaded = s.load("Main");
    let playlist = loaded.state.playlists.first_id().unwrap();
    apply(
        &mut loaded.state,
        Command::InsertPaths {
            playlist,
            index: 0,
            paths: vec![PathBuf::from("/m/c.flac")],
        },
    )
    .unwrap();
    s.save_state(&loaded.state, |_| 0.0, std::time::Instant::now())
        .unwrap();
    let again = s.load("Main");
    assert!(again.warnings.is_empty(), "{:?}", again.warnings);
    assert_eq!(again.state.library, loaded.state.library);
    assert_eq!(again.state.playlists, loaded.state.playlists);
    assert_eq!(again.state.library.len(), state.library.len() + 1);
}

#[test]
fn a_repaired_row_is_written_back_so_the_warning_is_not_repeated() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let state = saved_with_a_played_entry(&mut s);
    let track = state.library.ids()[0].0;
    edit_db(&s, |db| {
        db.conn()
            .execute("UPDATE tracks SET doc = '{' WHERE id = ?1", [track as i64])
            .unwrap();
    });
    let loaded = s.load("Main");
    assert_eq!(loaded.warnings.len(), 1, "{:?}", loaded.warnings);
    let again = s.load("Main");
    assert!(again.warnings.is_empty(), "{:?}", again.warnings);
}

#[test]
fn a_corrupt_database_falls_back_to_the_backup_with_a_warning() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let state = saved_with_a_played_entry(&mut s);
    drop(s);
    let mut s = store(&dir);
    fs::write(s.paths().database_file(), b"this is not a database").unwrap();
    let loaded = s.load("Main");
    assert_eq!(loaded.state.playlists, state.playlists);
    assert!(!loaded.warnings.is_empty());
}

fn corrupt_aside(s: &Store) -> Vec<PathBuf> {
    let db = s.paths().database_file();
    fs::read_dir(db.parent().unwrap())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| {
            p.to_string_lossy().contains(".corrupt-")
                && !p.to_string_lossy().ends_with("-wal")
                && !p.to_string_lossy().ends_with("-shm")
        })
        .collect()
}

fn track_rows(path: &std::path::Path) -> i64 {
    let conn = rusqlite::Connection::open(path).unwrap();
    conn.query_row("SELECT count(*) FROM tracks", [], |r| r.get(0))
        .unwrap()
}

#[test]
fn a_table_that_cannot_be_read_is_never_written_over() {
    // Without a backup: the file is set aside with its rows and the session
    // starts empty; saving does not touch the set-aside file.
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let state = saved_with_a_played_entry(&mut s);
    let tracks = state.library.len() as i64;
    edit_db(&s, |db| {
        db.conn().execute("DROP TABLE cart_pages", []).unwrap();
    });
    for n in 1..=9 {
        let _ = fs::remove_file(fp_store::backup_path(&s.paths().database_file(), n));
    }
    let loaded = s.load("Main");
    assert!(
        loaded
            .warnings
            .iter()
            .any(|w| w.contains("cart_pages") || w.contains("cart pages")),
        "{:?}",
        loaded.warnings
    );
    let aside = corrupt_aside(&s);
    assert_eq!(aside.len(), 1, "{aside:?}");
    assert_eq!(track_rows(&aside[0]), tracks, "the rows are kept");
    s.save_state(&loaded.state, |_| 0.0, std::time::Instant::now())
        .unwrap();
    assert_eq!(track_rows(&aside[0]), tracks, "saving leaves it alone");
}

#[test]
fn a_table_that_cannot_be_read_falls_back_to_the_backup() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let state = saved_with_a_played_entry(&mut s);
    edit_db(&s, |db| {
        db.conn().execute("DROP TABLE cart_pages", []).unwrap();
    });
    let loaded = s.load("Main");
    assert_eq!(corrupt_aside(&s).len(), 1);
    assert_eq!(loaded.state.playlists, state.playlists);
    assert!(
        loaded.warnings.iter().any(|w| w.contains("restored from")),
        "{:?}",
        loaded.warnings
    );
}

#[test]
fn a_failed_write_back_of_a_repair_is_reported() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let state = saved_with_a_played_entry(&mut s);
    edit_doc(&s, "players", state.players[0].id.0, |doc| {
        doc["pending_start"] = serde_json::json!("garbage");
    });
    edit_db(&s, |db| {
        db.conn()
            .execute_batch(
                "CREATE TRIGGER no_tracks BEFORE INSERT ON tracks \
                 BEGIN SELECT RAISE(ABORT, 'refused'); END;",
            )
            .unwrap();
    });
    let loaded = s.load("Main");
    assert!(
        loaded
            .warnings
            .iter()
            .any(|w| w.contains("write back") || w.contains("written back")),
        "{:?}",
        loaded.warnings
    );
}

#[test]
fn a_failed_save_keeps_the_baseline_so_the_next_one_writes_the_difference() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    saved_with_a_played_entry(&mut s);
    let mut state = s.load("Main").state;
    let playlist = state.playlists.first_id().unwrap();
    apply(
        &mut state,
        Command::InsertPaths {
            playlist,
            index: 0,
            paths: vec![PathBuf::from("/m/new.flac")],
        },
    )
    .unwrap();
    edit_db(&s, |db| {
        db.conn()
            .execute_batch(
                "CREATE TRIGGER no_tracks BEFORE INSERT ON tracks \
                 BEGIN SELECT RAISE(ABORT, 'refused'); END;",
            )
            .unwrap();
    });
    let now = std::time::Instant::now();
    assert!(s.save_state(&state, |_| 0.0, now).is_err());
    edit_db(&s, |db| {
        db.conn().execute("DROP TRIGGER no_tracks", []).unwrap();
    });
    s.save_state(&state, |_| 0.0, now).unwrap();
    let again = s.load("Main");
    assert!(again.warnings.is_empty(), "{:?}", again.warnings);
    assert_eq!(again.state.library, state.library);
    assert_eq!(again.state.playlists, state.playlists);
}

fn drop_table_in(file: &std::path::Path, table: &str) {
    let db = fp_store::Db::open(file).unwrap();
    db.conn()
        .execute(&format!("DROP TABLE {table}"), [])
        .unwrap();
}

fn bak(s: &Store, n: usize) -> PathBuf {
    fp_store::backup_path(&s.paths().database_file(), n)
}

#[test]
fn an_unreadable_backup_is_skipped_for_the_next_one() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let state = saved_with_a_played_entry(&mut s);
    fs::copy(bak(&s, 1), bak(&s, 2)).unwrap();
    drop_table_in(&bak(&s, 1), "cart_pages");
    edit_db(&s, |db| {
        db.conn().execute("DROP TABLE cart_pages", []).unwrap();
    });
    let before = fs::read(bak(&s, 1)).unwrap();
    let loaded = s.load("Main");
    assert_eq!(loaded.state.playlists, state.playlists);
    assert!(
        loaded
            .warnings
            .iter()
            .any(|w| w.contains("bak2") && w.contains("restored")),
        "{:?}",
        loaded.warnings
    );
    assert_eq!(
        fs::read(bak(&s, 1)).unwrap(),
        before,
        "the bad backup is untouched"
    );
}

#[test]
fn when_no_backup_is_readable_the_disk_is_left_alone_for_the_session() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let state = saved_with_a_played_entry(&mut s);
    fs::copy(bak(&s, 1), bak(&s, 2)).unwrap();
    drop_table_in(&bak(&s, 1), "cart_pages");
    drop_table_in(&bak(&s, 2), "players");
    edit_db(&s, |db| {
        db.conn().execute("DROP TABLE cart_pages", []).unwrap();
    });
    let backups_before: Vec<Vec<u8>> = (1..=3)
        .map(|n| fs::read(bak(&s, n)).unwrap_or_default())
        .collect();
    let loaded = s.load("Main");
    assert!(
        loaded.warnings.iter().any(|w| w.contains("not saved")),
        "{:?}",
        loaded.warnings
    );
    let now = std::time::Instant::now();
    s.save_state(&state, |_| 0.0, now).unwrap();
    s.save_state(&state, |_| 0.0, now + std::time::Duration::from_secs(1000))
        .unwrap();
    let aside = corrupt_aside(&s);
    assert_eq!(aside.len(), 1, "{aside:?}");
    assert!(
        !s.paths().database_file().exists(),
        "nothing was written to the main path"
    );
    let backups_after: Vec<Vec<u8>> = (1..=3)
        .map(|n| fs::read(bak(&s, n)).unwrap_or_default())
        .collect();
    assert_eq!(backups_before, backups_after);
}

/// The main file and its side files, removed.
fn remove_main_database(s: &Store) {
    let main = s.paths().database_file();
    for suffix in ["", "-wal", "-shm"] {
        let mut name = main.as_os_str().to_owned();
        name.push(suffix);
        let _ = fs::remove_file(PathBuf::from(name));
    }
}

#[test]
fn a_missing_database_is_restored_from_its_backup() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let state = saved_with_a_played_entry(&mut s);
    assert!(bak(&s, 1).exists());
    let kept = fs::read(bak(&s, 1)).unwrap();
    remove_main_database(&s);
    let loaded = s.load("Main");
    assert_eq!(loaded.state.library, state.library);
    assert_eq!(loaded.state.playlists, state.playlists);
    assert!(
        loaded
            .warnings
            .iter()
            .any(|w| w.contains("missing") && w.contains("backup")),
        "{:?}",
        loaded.warnings
    );
    // A save in this session rotates the backups; the restored content
    // must still be among them, not pushed out by an empty database.
    let now = std::time::Instant::now();
    s.save_state(&loaded.state, |_| 0.0, now).unwrap();
    let still_there = (1..=3).any(|n| fs::read(bak(&s, n)).is_ok_and(|b| b == kept));
    assert!(still_there, "the old backup was rotated away");
}

#[test]
fn a_database_that_could_not_be_opened_is_never_created_by_a_later_save() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    // The data directory cannot be created: a file is in its way.
    let data_dir = s.paths().data_dir.clone();
    fs::create_dir_all(data_dir.parent().unwrap()).unwrap();
    fs::write(&data_dir, b"in the way").unwrap();
    let loaded = s.load("Main");
    assert!(!loaded.warnings.is_empty());
    fs::remove_file(&data_dir).unwrap();
    let state = AppState::new(Config::default(), "Main");
    let _ = s.save_state(&state, |_| 0.0, std::time::Instant::now());
    assert!(
        !s.paths().database_file().exists(),
        "a save reopened the database after a failed open"
    );
}

#[test]
fn an_unreadable_track_row_does_not_stop_later_saves() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let state = saved_with_a_played_entry(&mut s);
    let track = state.library.ids()[0].0;
    edit_db(&s, |db| {
        db.conn()
            .execute(
                "UPDATE tracks SET path = X'FF00FE' WHERE id = ?1",
                [track as i64],
            )
            .unwrap();
    });
    let mut loaded = s.load("Main");
    assert!(!loaded.warnings.is_empty());
    let playlist = loaded.state.playlists.first_id().unwrap();
    assert!(
        loaded
            .state
            .playlists
            .get(playlist)
            .unwrap()
            .entries
            .iter()
            .all(|e| e.track.0 != track),
        "the dangling entry is gone"
    );
    apply(
        &mut loaded.state,
        Command::InsertPaths {
            playlist,
            index: 0,
            paths: vec![PathBuf::from("/m/c.flac")],
        },
    )
    .unwrap();
    s.save_state(&loaded.state, |_| 0.0, std::time::Instant::now())
        .unwrap();
    let again = s.load("Main");
    assert!(again.warnings.is_empty(), "{:?}", again.warnings);
    assert_eq!(again.state.playlists, loaded.state.playlists);
    assert_eq!(again.state.library, loaded.state.library);
}

#[test]
fn a_stale_wal_beside_a_missing_database_is_set_aside() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let state = saved_with_a_played_entry(&mut s);
    // Another database's `-wal`, written in a second home and kept
    // readable by holding a connection open while copying it.
    let other_dir = tempfile::tempdir().unwrap();
    let mut other = store(&other_dir);
    let mut other_state = AppState::new(Config::default(), "Other");
    let playlist = other_state.playlists.first_id().unwrap();
    apply(
        &mut other_state,
        Command::InsertPaths {
            playlist,
            index: 0,
            paths: vec![PathBuf::from("/o/x.flac")],
        },
    )
    .unwrap();
    save_all(&mut other, &other_state, 0.0);
    let other_wal = {
        let mut n = other.paths().database_file().into_os_string();
        n.push("-wal");
        PathBuf::from(n)
    };
    let wal_bytes = fs::read(&other_wal).unwrap();
    assert!(!wal_bytes.is_empty());
    remove_main_database(&s);
    let main = s.paths().database_file();
    let mut wal = main.clone().into_os_string();
    wal.push("-wal");
    let wal = PathBuf::from(wal);
    fs::write(&wal, &wal_bytes).unwrap();
    let loaded = s.load("Main");
    assert_eq!(loaded.state.library, state.library);
    assert_eq!(loaded.state.playlists, state.playlists);
    let aside: Vec<_> = fs::read_dir(main.parent().unwrap())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|n| n.contains("-wal") && n.contains("corrupt-"))
        .collect();
    assert_eq!(aside.len(), 1, "{aside:?}");
}
