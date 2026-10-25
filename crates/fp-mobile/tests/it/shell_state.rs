#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! What the Kotlin shell polls: whether to run the foreground service and
//! keep the screen on (MB1, MB8), and the notification's text.

use std::path::PathBuf;

use fp_mobile::shell_state::{KEEP_SCREEN_ON, PLAYOUT_ACTIVE, SOUNDING, on_air_text, shell_state};
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
    assert_eq!(shell_state(&s), PLAYOUT_ACTIVE | KEEP_SCREEN_ON | SOUNDING);
    s.config.ui.keep_screen_on = false;
    assert_eq!(shell_state(&s), PLAYOUT_ACTIVE | SOUNDING);
}

#[test]
fn a_loaded_track_alone_is_not_sounding() {
    // Audio focus is held only while something sounds: opening the app
    // with saved playlists must not pause the operator's other apps.
    let s = with_tracks();
    assert_eq!(shell_state(&s) & SOUNDING, 0);
    assert_ne!(shell_state(&s) & PLAYOUT_ACTIVE, 0);
}

#[test]
fn a_paused_player_is_not_sounding_and_a_cue_is() {
    let mut s = with_tracks();
    s.players[0].transport = Transport::Paused;
    assert_eq!(shell_state(&s) & SOUNDING, 0);
    s.cartwall.cue = Some(s.cartwall.pages[0].carts[0].id);
    assert_ne!(shell_state(&s) & SOUNDING, 0);
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
