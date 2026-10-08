#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::PathBuf;

use fp_model::{AppState, Command, Config, EngineAction, Transport, apply};
use fp_store::{AppPaths, Store};

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
    let s = store(&dir);
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
    for t in state.library.iter_mut() {
        t.duration_secs = 200.0;
    }
    let p = state.players[0].id;
    apply(&mut state, Command::Play(p)).unwrap();

    s.save_config(&state).unwrap();
    s.save_playlists(&state).unwrap();
    s.save_session(&state, |_| 42.5).unwrap();

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

#[test]
fn newer_schema_is_not_quarantined() {
    let dir = tempfile::tempdir().unwrap();
    let s = store(&dir);
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
    let s = store(&dir);
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
    let s = store(&dir);
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
    let s = store(&dir);
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
fn a_newer_file_is_preserved_beyond_backup_rotation() {
    let dir = tempfile::tempdir().unwrap();
    let s = store(&dir);
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
    let s = store(&dir);
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
    let s = store(&dir);
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
fn saved_with_a_played_entry(s: &Store) -> AppState {
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
    s.save_config(&state).unwrap();
    s.save_playlists(&state).unwrap();
    s.save_session(&state, |_| 0.0).unwrap();
    state
}

#[test]
fn playlists_are_saved_with_the_schema_that_has_per_player_played_marks() {
    // Builds from before per-player marks require `played` on every entry;
    // a newer schema makes them keep the file aside instead of discarding it.
    let dir = tempfile::tempdir().unwrap();
    let s = store(&dir);
    saved_with_a_played_entry(&s);
    let doc: serde_json::Value =
        serde_json::from_slice(&fs::read(s.paths().playlists_file()).unwrap()).unwrap();
    assert_eq!(doc["schema_version"], 2);
}

#[test]
fn a_schema_1_playlists_file_counts_its_played_entries_for_every_player() {
    let dir = tempfile::tempdir().unwrap();
    let s = store(&dir);
    let state = saved_with_a_played_entry(&s);
    let path = s.paths().playlists_file();
    let mut doc: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    doc["schema_version"] = 1.into();
    for list in doc["playlists"].as_array_mut().unwrap() {
        for entry in list["entries"].as_array_mut().unwrap() {
            let obj = entry.as_object_mut().unwrap();
            let played = obj.remove("played_by").is_some();
            obj.insert("played".into(), played.into());
        }
    }
    fs::write(&path, serde_json::to_vec(&doc).unwrap()).unwrap();

    let loaded = s.load("Main");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    let playlist = loaded.state.playlists.first_id().unwrap();
    let entries = &loaded.state.playlists.get(playlist).unwrap().entries;
    let players: Vec<_> = state.players.iter().map(|p| p.id).collect();
    assert!(players.iter().all(|p| entries[0].is_played_by(*p)));
    assert!(players.iter().all(|p| !entries[1].is_played_by(*p)));
}

#[test]
fn repeated_playlist_ids_are_repaired_with_a_warning() {
    let dir = tempfile::tempdir().unwrap();
    let s = store(&dir);
    let mut state = s.load("Main").state;
    let playlist = state.playlists.first_id().unwrap();
    apply(
        &mut state,
        Command::InsertPaths {
            playlist,
            index: 0,
            paths: vec![PathBuf::from("/music/a.mp3")],
        },
    )
    .unwrap();
    s.save_playlists(&state).unwrap();
    // A hand edit copies the playlist, ids and all.
    let path = dir.path().join("data/playlists.json");
    let mut doc: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    let lists = doc["playlists"].as_array_mut().unwrap();
    let copy = lists[0].clone();
    lists.push(copy);
    fs::write(&path, serde_json::to_string(&doc).unwrap()).unwrap();
    let loaded = s.load("Main");
    assert_eq!(loaded.state.playlists.len(), 2);
    assert!(
        loaded.warnings.iter().any(|w| w.contains("duplicate id")),
        "{:?}",
        loaded.warnings
    );
}

#[test]
fn analysis_fields_this_version_dropped_are_ignored_with_a_warning() {
    let dir = tempfile::tempdir().unwrap();
    let s = store(&dir);
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
    let s = store(&dir);
    saved_with_a_played_entry(&s);
    let path = s.paths().session_file();
    let mut doc: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    doc["players"][0]["pending_start"] = serde_json::json!("garbage");
    fs::write(&path, serde_json::to_vec(&doc).unwrap()).unwrap();

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
    let s = store(&dir);
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
fn routes_of_a_removed_player(s: &Store) -> (AppState, fp_model::PlayerId) {
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
    s.save_playlists(&state).unwrap();
    (state, removed)
}

#[test]
fn the_routes_of_a_removed_player_and_their_devices_settings_are_dropped_on_load() {
    let dir = tempfile::tempdir().unwrap();
    let s = store(&dir);
    let (state, removed) = routes_of_a_removed_player(&s);
    s.save_session(&state, |_| 0.0).unwrap();
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
    // The players' ids are not known: a route may be for a player whose
    // session is only missing.
    let dir = tempfile::tempdir().unwrap();
    let s = store(&dir);
    let (state, removed) = routes_of_a_removed_player(&s);
    let loaded = s.load("Main");
    let o = &loaded.state.config.outputs;
    assert!(o.player_routes(removed).is_some());
    assert_eq!(o.device_overrides, state.config.outputs.device_overrides);
}

#[test]
fn an_old_restart_handoff_is_ignored_on_load() {
    let dir = tempfile::tempdir().unwrap();
    let s = store(&dir);
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
