#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback spec §6.3: LED feedback.

use std::path::PathBuf;

use fp_control::feedback::{Feedback, Led, desired};
use fp_model::{
    AppState, Command, Config, MidiAction, MidiConfig, MidiDevice, MidiTrigger, ShortcutAction,
    apply,
};

fn state() -> AppState {
    let mut s = AppState::new(Config::default(), "Main");
    let playlist = s.playlists.first_id().unwrap();
    let paths = (1..=3)
        .map(|n| PathBuf::from(format!("/m/{n}.mp3")))
        .collect();
    apply(
        &mut s,
        Command::InsertPaths {
            playlist,
            index: 0,
            paths,
        },
    )
    .unwrap();
    for t in s.library.iter_mut() {
        t.duration_secs = 180.0;
    }
    s
}

fn note(n: u8) -> MidiTrigger {
    MidiTrigger::Note {
        channel: 0,
        note: n,
    }
}

fn config() -> MidiConfig {
    let mut m = MidiConfig::default();
    for (n, a) in [
        (1, ShortcutAction::PlayPlayer(1)),
        (2, ShortcutAction::PausePlayer(1)),
        (3, ShortcutAction::StopPlayer(1)),
        (4, ShortcutAction::CuePlayer(1)),
        (5, ShortcutAction::RestartPlayer(1)),
        (6, ShortcutAction::PreviousPlayer(1)),
    ] {
        m.bind(MidiAction::Button(a), "APC", note(n));
    }
    m.bind(
        MidiAction::Button(ShortcutAction::StopPlayer(2)),
        "APC",
        MidiTrigger::ControlChange {
            channel: 1,
            controller: 20,
        },
    );
    m
}

fn lit(leds: &[Led], bytes: [u8; 2]) -> Option<bool> {
    leds.iter()
        .find(|l| l.bytes[0] == bytes[0] && l.bytes[1] == bytes[1])
        .map(|l| l.bytes[2] == 127)
}

#[test]
fn a_stopped_player_lights_nothing_but_what_it_can_do() {
    let s = state();
    let leds = desired(&config(), &s, true);
    assert_eq!(lit(&leds, [0x90, 1]), Some(false), "Play: not playing");
    assert_eq!(lit(&leds, [0x90, 3]), Some(false), "Stop: unavailable");
    assert_eq!(lit(&leds, [0x90, 4]), Some(false), "Cue: not cueing");
    assert_eq!(lit(&leds, [0xB1, 20]), Some(false), "a CC button");
    assert!(leds.iter().all(|l| l.output == "APC"));
}

#[test]
fn a_playing_player_lights_play_stop_and_restart() {
    let mut s = state();
    let p = s.players[0].id;
    apply(&mut s, Command::Play(p)).unwrap();
    let leds = desired(&config(), &s, true);
    assert_eq!(lit(&leds, [0x90, 1]), Some(true));
    assert_eq!(lit(&leds, [0x90, 3]), Some(true));
    assert_eq!(lit(&leds, [0x90, 5]), Some(true));
    assert_eq!(lit(&leds, [0x90, 6]), Some(false), "no history yet");
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    assert_eq!(lit(&desired(&config(), &s, true), [0x90, 4]), Some(true));
}

#[test]
fn pause_blinks_while_paused() {
    let mut s = state();
    let p = s.players[0].id;
    apply(&mut s, Command::Play(p)).unwrap();
    apply(&mut s, Command::Pause(p)).unwrap();
    assert_eq!(lit(&desired(&config(), &s, true), [0x90, 2]), Some(true));
    assert_eq!(lit(&desired(&config(), &s, false), [0x90, 2]), Some(false));
    assert_eq!(
        lit(&desired(&config(), &s, true), [0x90, 1]),
        Some(false),
        "Play: paused"
    );
}

#[test]
fn only_changes_are_sent_until_a_reset() {
    let mut s = state();
    let mut f = Feedback::default();
    let first = f.changes(desired(&config(), &s, true));
    assert_eq!(first.len(), 7, "everything the first time");
    assert!(f.changes(desired(&config(), &s, true)).is_empty());
    let p = s.players[0].id;
    apply(&mut s, Command::Play(p)).unwrap();
    let changed = f.changes(desired(&config(), &s, true));
    assert!(!changed.is_empty() && changed.len() < 7, "{changed:?}");
    f.reset();
    assert_eq!(f.changes(desired(&config(), &s, true)).len(), 7);
}

#[test]
fn feedback_goes_to_the_override_port_or_nowhere_when_off() {
    let s = state();
    let mut c = config();
    c.devices.push(MidiDevice {
        input: "APC".into(),
        output: Some("APC out".into()),
    });
    assert!(desired(&c, &s, true).iter().all(|l| l.output == "APC out"));
    c.feedback = false;
    assert!(desired(&c, &s, true).is_empty());
}
