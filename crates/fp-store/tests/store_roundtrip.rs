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
