#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! What the Kotlin shell polls: whether to run the foreground service and
//! keep the screen on (MB1, MB8), and the notification's text.

use std::path::PathBuf;

use fp_mobile::shell_state::{KEEP_SCREEN_ON, PLAYOUT_ACTIVE, on_air_text, shell_state};
use fp_model::{AppState, Command, Config, Transport, apply};

fn with_tracks() -> AppState {
    let mut s = AppState::new(Config::default(), "Main");
    let playlist = s.playlists.first_id().unwrap();
    apply(
        &mut s,
        Command::InsertPaths {
            playlist,
            index: 0,
            paths: vec![PathBuf::from("/music/Coastline.mp3")],
        },
    )
    .unwrap();
    s
}

#[test]
fn nothing_loaded_asks_for_nothing() {
    let s = AppState::new(Config::default(), "Main");
    assert_eq!(shell_state(&s), 0);
}

#[test]
fn playout_keeps_the_service_and_the_screen_on() {
    let mut s = AppState::new(Config::default(), "Main");
    s.players[0].transport = Transport::Playing;
    assert_eq!(shell_state(&s), PLAYOUT_ACTIVE | KEEP_SCREEN_ON);
    s.config.ui.keep_screen_on = false;
    assert_eq!(shell_state(&s), PLAYOUT_ACTIVE);
}

#[test]
fn the_notification_names_each_player_on_air() {
    let mut s = with_tracks();
    assert_eq!(on_air_text(&s), "");
    let p = s.players[0].id;
    apply(&mut s, Command::Play(p)).unwrap();
    let text = on_air_text(&s);
    assert!(text.starts_with("P1 — "), "{text}");
    assert!(text.contains("Coastline"), "{text}");
}
