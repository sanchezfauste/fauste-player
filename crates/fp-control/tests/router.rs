#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback spec §6.3: bindings turn MIDI messages into commands.

use std::path::PathBuf;

use fp_control::message::MidiMessage;
use fp_control::router::Router;
use fp_model::{
    AppState, Command, Config, MidiAction, MidiConfig, MidiTrigger, ShortcutAction, apply,
};

/// Two players on one playlist of three tracks; P1 is playing.
fn state() -> AppState {
    let mut config = Config::default();
    config.players.count = 2;
    let mut s = AppState::new(config, "Main");
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
    let p = s.players[0].id;
    apply(&mut s, Command::Play(p)).unwrap();
    s
}

fn note(n: u8) -> MidiTrigger {
    MidiTrigger::Note {
        channel: 0,
        note: n,
    }
}

fn cc(c: u8) -> MidiTrigger {
    MidiTrigger::ControlChange {
        channel: 0,
        controller: c,
    }
}

fn router(bindings: &[(MidiTrigger, MidiAction)]) -> Router {
    let mut m = MidiConfig::default();
    for (t, a) in bindings {
        m.bind(*a, "APC", *t);
    }
    Router::new(&m)
}

fn on(n: u8) -> MidiMessage {
    MidiMessage::NoteOn {
        channel: 0,
        note: n,
        velocity: 127,
    }
}

fn cc_value(c: u8, value: u8) -> MidiMessage {
    MidiMessage::ControlChange {
        channel: 0,
        controller: c,
        value,
    }
}

#[test]
fn each_button_action_sends_its_command() {
    let s = state();
    let (p1, p2) = (s.players[0].id, s.players[1].id);
    let actions = [
        (ShortcutAction::PlayPlayer(2), Command::Play(p2)),
        (ShortcutAction::PausePlayer(1), Command::Pause(p1)),
        (ShortcutAction::StopPlayer(1), Command::Stop(p1)),
        (ShortcutAction::FadeStopPlayer(1), Command::FadeStop(p1)),
        (ShortcutAction::RestartPlayer(1), Command::Restart(p1)),
        (ShortcutAction::CuePlayer(2), Command::ToggleCue(p2)),
    ];
    for (i, (action, command)) in actions.into_iter().enumerate() {
        let n = i as u8;
        let mut r = router(&[(note(n), MidiAction::Button(action))]);
        assert_eq!(r.on_message("APC", on(n), &s), Some(command), "{action:?}");
    }
}

#[test]
fn a_note_off_or_another_device_does_nothing() {
    let s = state();
    let mut r = router(&[(note(36), MidiAction::Button(ShortcutAction::StopPlayer(1)))]);
    assert_eq!(
        r.on_message(
            "APC",
            MidiMessage::NoteOff {
                channel: 0,
                note: 36
            },
            &s
        ),
        None
    );
    assert_eq!(r.on_message("Other", on(36), &s), None);
    assert_eq!(r.on_message("APC", on(37), &s), None);
}

#[test]
fn a_control_change_fires_when_it_rises_through_64() {
    let s = state();
    let mut r = router(&[(cc(20), MidiAction::Button(ShortcutAction::StopPlayer(1)))]);
    assert!(
        r.on_message("APC", cc_value(20, 127), &s).is_some(),
        "0 → 127"
    );
    assert!(r.on_message("APC", cc_value(20, 127), &s).is_none(), "held");
    assert!(
        r.on_message("APC", cc_value(20, 0), &s).is_none(),
        "released"
    );
    assert!(r.on_message("APC", cc_value(20, 30), &s).is_none());
    assert!(r.on_message("APC", cc_value(20, 60), &s).is_none());
    assert!(
        r.on_message("APC", cc_value(20, 70), &s).is_some(),
        "60 → 70"
    );
}

#[test]
fn an_unavailable_action_is_ignored() {
    let s = state();
    // P2 is stopped: Stop and Previous make no sense (R28).
    let mut r = router(&[
        (note(1), MidiAction::Button(ShortcutAction::StopPlayer(2))),
        (
            note(2),
            MidiAction::Button(ShortcutAction::PreviousPlayer(1)),
        ),
    ]);
    assert_eq!(r.on_message("APC", on(1), &s), None);
    assert_eq!(r.on_message("APC", on(2), &s), None);
}

#[test]
fn a_binding_for_a_player_that_does_not_exist_does_nothing() {
    let s = state();
    let mut r = router(&[(note(1), MidiAction::Button(ShortcutAction::PlayPlayer(5)))]);
    assert_eq!(r.on_message("APC", on(1), &s), None);
}
