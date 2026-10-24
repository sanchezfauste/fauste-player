#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Resizable layout spec RL19: the layout in `session.json`.

use std::fs;

use fp_model::{
    AppState, Command, Config, DEFAULT_SIDE_SHARE, Dock, Layout, MAX_CARTWALL_SHARE, Panel, apply,
};
use fp_store::{AppPaths, Store};
use serde_json::json;

fn store(dir: &tempfile::TempDir) -> Store {
    Store::new(AppPaths::under(dir.path()), Config::default().limits)
}

/// Saves a state whose first player is at half volume and whose layout is
/// not the default: P1 wider than P2, P3 hidden, the cartwall on the
/// left, locked.
fn saved(s: &Store) -> AppState {
    let mut state = AppState::new(Config::default(), "Main");
    let p = state.player_ids();
    apply(&mut state, Command::SetVolume(p[0], 0.5)).unwrap();
    apply(
        &mut state,
        Command::SetPlayerShares {
            left: (p[0], 1.5),
            right: (p[1], 0.5),
        },
    )
    .unwrap();
    apply(
        &mut state,
        Command::SetPanelHidden {
            panel: Panel::Player(p[2]),
            hidden: true,
        },
    )
    .unwrap();
    apply(&mut state, Command::SetCartwallDock(Dock::Left)).unwrap();
    apply(&mut state, Command::SetLayoutLocked(true)).unwrap();
    s.save_config(&state).unwrap();
    s.save_playlists(&state).unwrap();
    s.save_session(&state, |_| 0.0).unwrap();
    state
}

fn edit_session(s: &Store, edit: impl FnOnce(&mut serde_json::Value)) {
    let path = s.paths().session_file();
    let mut doc: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    edit(&mut doc);
    fs::write(&path, serde_json::to_vec(&doc).unwrap()).unwrap();
}

/// The rest of the session was kept: same players, same volume.
fn session_kept(loaded: &AppState, saved: &AppState) {
    assert_eq!(loaded.player_ids(), saved.player_ids());
    assert_eq!(loaded.players[0].volume, 0.5);
}

#[test]
fn rl19_the_layout_round_trips() {
    let dir = tempfile::tempdir().unwrap();
    let s = store(&dir);
    let state = saved(&s);
    let loaded = s.load("Main");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    assert_eq!(loaded.state.layout, state.layout);
}

#[test]
fn rl19_an_older_session_without_a_layout_loads_the_default() {
    let dir = tempfile::tempdir().unwrap();
    let s = store(&dir);
    let state = saved(&s);
    edit_session(&s, |doc| {
        doc.as_object_mut().unwrap().remove("layout");
    });
    let loaded = s.load("Main");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    assert_eq!(loaded.state.layout, Layout::default());
    session_kept(&loaded.state, &state);
}

#[test]
fn rl19_a_malformed_layout_keeps_the_rest_of_the_session() {
    let dir = tempfile::tempdir().unwrap();
    let s = store(&dir);
    let state = saved(&s);
    edit_session(&s, |doc| doc["layout"] = json!("wide"));
    let loaded = s.load("Main");
    assert_eq!(loaded.state.layout, Layout::default());
    session_kept(&loaded.state, &state);
    assert_eq!(loaded.warnings.len(), 1, "{:?}", loaded.warnings);
    assert!(
        loaded.warnings[0].contains("layout"),
        "{:?}",
        loaded.warnings
    );
}

#[test]
fn rl19_a_malformed_part_keeps_the_other_parts() {
    let dir = tempfile::tempdir().unwrap();
    let s = store(&dir);
    let state = saved(&s);
    edit_session(&s, |doc| {
        doc["layout"]["cartwall"]["dock"] = json!("Top");
        doc["layout"]["locked"] = json!("yes");
        doc["layout"]["player_shares"][0] = json!("x");
    });
    let loaded = s.load("Main");
    let layout = &loaded.state.layout;
    assert_eq!(layout.cartwall.dock, Dock::Bottom, "an unknown dock");
    assert!(!layout.locked);
    assert_eq!(layout.player_shares.len(), 3, "the other shares are kept");
    assert_eq!(layout.hidden_players, state.layout.hidden_players);
    session_kept(&loaded.state, &state);
    assert_eq!(loaded.warnings.len(), 3, "{:?}", loaded.warnings);
    assert!(
        loaded
            .warnings
            .iter()
            .all(|w| w.starts_with("session: layout")),
        "{:?}",
        loaded.warnings
    );
}

#[test]
fn rl19_players_that_no_longer_exist_are_dropped_with_a_note() {
    let dir = tempfile::tempdir().unwrap();
    let s = store(&dir);
    let state = saved(&s);
    // The session has four players; the configuration now wants three.
    let mut fewer = state.clone();
    fewer.config.players.count = 3;
    s.save_config(&fewer).unwrap();
    edit_session(&s, |doc| {
        let layout = &mut doc["layout"];
        layout["player_shares"]
            .as_array_mut()
            .unwrap()
            .push(json!([999_999, 2.0]));
        layout["hidden_players"]
            .as_array_mut()
            .unwrap()
            .push(json!(999_999));
    });
    let loaded = s.load("Main");
    let players = loaded.state.player_ids();
    assert_eq!(players.len(), 3);
    let layout = &loaded.state.layout;
    assert!(
        layout
            .player_shares
            .iter()
            .all(|(id, _)| players.contains(id))
    );
    assert!(layout.hidden_players.iter().all(|id| players.contains(id)));
    let dropped = loaded
        .warnings
        .iter()
        .filter(|w| w.contains("no such player"))
        .count();
    assert_eq!(dropped, 3, "{:?}", loaded.warnings);
    assert_eq!(loaded.warnings.len(), 3, "{:?}", loaded.warnings);
}

#[test]
fn rl19_everything_hidden_is_shown_with_a_note() {
    let dir = tempfile::tempdir().unwrap();
    let s = store(&dir);
    let state = saved(&s);
    let all: Vec<u64> = state.player_ids().iter().map(|p| p.0).collect();
    edit_session(&s, |doc| {
        doc["layout"]["hidden_players"] = json!(all);
        doc["layout"]["cartwall"]["hidden"] = json!(true);
    });
    let loaded = s.load("Main");
    assert!(loaded.state.layout.hidden_players.is_empty());
    assert!(!loaded.state.layout.cartwall.hidden);
    assert_eq!(loaded.warnings.len(), 1, "{:?}", loaded.warnings);
    assert!(loaded.warnings[0].contains("every panel"));
}

#[test]
fn rl19_a_side_dock_gets_a_usable_size_with_a_note() {
    let dir = tempfile::tempdir().unwrap();
    let s = store(&dir);
    saved(&s);
    edit_session(&s, |doc| doc["layout"]["cartwall"]["size"] = json!(null));
    let loaded = s.load("Main");
    assert_eq!(loaded.state.layout.cartwall.size, Some(DEFAULT_SIDE_SHARE));
    assert_eq!(loaded.warnings.len(), 1, "{:?}", loaded.warnings);

    saved(&s);
    edit_session(&s, |doc| doc["layout"]["cartwall"]["size"] = json!(5.0));
    let loaded = s.load("Main");
    assert_eq!(loaded.state.layout.cartwall.size, Some(MAX_CARTWALL_SHARE));
    assert_eq!(loaded.warnings.len(), 1, "{:?}", loaded.warnings);
}

#[test]
fn rl19_a_field_from_a_newer_build_is_ignored() {
    let dir = tempfile::tempdir().unwrap();
    let s = store(&dir);
    let state = saved(&s);
    edit_session(&s, |doc| {
        doc["layout"]["named"] = json!([{"name": "Night"}])
    });
    let loaded = s.load("Main");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    assert_eq!(loaded.state.layout, state.layout);
}
