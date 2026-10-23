#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback spec §6.3: bindings turn MIDI messages into commands.

use std::path::PathBuf;

use fp_control::message::MidiMessage;
use fp_control::router::Router;
use fp_model::{
    AppState, Command, Config, EngineEvent, Holder, MidiAction, MidiConfig, MidiTrigger,
    OutputDevice, ShortcutAction, apply, device_settings, on_event,
};

/// Two players on one playlist of three tracks; P1 is playing.
fn state() -> AppState {
    let mut config = Config::default();
    config.players.count = 2;
    let mut s = AppState::new(config, "Main");
    give_cue_routes(&mut s);
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
        r.on_message("APC", cc_value(20, 0), &s).is_none(),
        "first heard"
    );
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

// Soft takeover (feedback spec §6.3).

use fp_model::volume::{fader_from_gain, gain_from_fader};

/// P1's volume set to the fader travel `travel`.
fn at_travel(mut s: AppState, travel: f32) -> AppState {
    let p = s.players[0].id;
    apply(&mut s, Command::SetVolume(p, gain_from_fader(travel))).unwrap();
    s
}

fn volume_router() -> Router {
    router(&[(cc(7), MidiAction::Volume(1))])
}

/// The travel a CC value stands for.
fn travel_of(value: u8) -> f32 {
    f32::from(value) / 127.0
}

fn volume_of(command: Option<Command>) -> Option<f32> {
    match command {
        Some(Command::SetVolume(_, v)) => Some(fader_from_gain(v)),
        _ => None,
    }
}

#[test]
fn a_fader_does_nothing_until_it_reaches_the_volume() {
    let s = at_travel(state(), travel_of(100));
    let mut r = volume_router();
    for v in [10, 40, 80, 99] {
        assert_eq!(r.on_message("APC", cc_value(7, v), &s), None, "{v}");
    }
    // Crossing from below picks it up.
    let got = volume_of(r.on_message("APC", cc_value(7, 105), &s)).unwrap();
    assert!((got - travel_of(105)).abs() < 1e-3);
}

#[test]
fn crossing_from_above_or_landing_exactly_picks_up() {
    let s = at_travel(state(), travel_of(50));
    let mut r = volume_router();
    assert_eq!(r.on_message("APC", cc_value(7, 90), &s), None);
    assert!(
        r.on_message("APC", cc_value(7, 40), &s).is_some(),
        "crossed from above"
    );
    let mut r = volume_router();
    assert!(
        r.on_message("APC", cc_value(7, 50), &s).is_some(),
        "landed on it"
    );
}

#[test]
fn once_picked_up_every_move_counts_even_jitter() {
    let mut s = at_travel(state(), travel_of(50));
    let mut r = volume_router();
    let p = s.players[0].id;
    for v in [50, 51, 50, 49, 50, 52] {
        let c = r.on_message("APC", cc_value(7, v), &s);
        assert!(c.is_some(), "{v}");
        apply(&mut s, c.unwrap()).unwrap();
    }
    assert!((fader_from_gain(s.player(p).unwrap().volume) - travel_of(52)).abs() < 1e-3);
}

#[test]
fn another_change_of_the_volume_re_arms_the_pickup() {
    let s = at_travel(state(), travel_of(50));
    let mut r = volume_router();
    assert!(r.on_message("APC", cc_value(7, 50), &s).is_some());
    // The on-screen fader moved the volume to 90.
    let s = at_travel(s, travel_of(90));
    assert_eq!(
        r.on_message("APC", cc_value(7, 55), &s),
        None,
        "not picked up any more"
    );
    assert!(
        r.on_message("APC", cc_value(7, 95), &s).is_some(),
        "crossed 90"
    );
}

#[test]
fn a_pitch_bend_fader_uses_its_full_range() {
    let s = at_travel(state(), 1.0);
    let mut r = router(&[(MidiTrigger::PitchBend { channel: 0 }, MidiAction::Volume(1))]);
    let top = MidiMessage::PitchBend {
        channel: 0,
        value: 16_383,
    };
    let got = volume_of(r.on_message("APC", top, &s)).unwrap();
    assert!((got - 1.0).abs() < 1e-4);
}

#[test]
fn a_quick_move_against_a_model_that_has_not_caught_up_keeps_control() {
    let s = at_travel(state(), travel_of(50));
    let mut r = volume_router();
    // Every message of the burst sees the same, stale snapshot.
    for v in [50, 53, 56, 59, 62, 65, 68] {
        assert!(r.on_message("APC", cc_value(7, v), &s).is_some(), "{v}");
    }
}

#[test]
fn a_control_already_up_when_first_heard_does_not_fire() {
    let s = state();
    let mut r = router(&[(cc(20), MidiAction::Button(ShortcutAction::StopPlayer(1)))]);
    // A latched control reports 127 on connection: not a press.
    assert!(r.on_message("APC", cc_value(20, 127), &s).is_none());
    assert!(r.on_message("APC", cc_value(20, 0), &s).is_none());
    assert!(r.on_message("APC", cc_value(20, 127), &s).is_some());
}

/// A Cue output (headphones) for every player and the cartwall, with Main on
/// the default output: a CUE needs a Cue route (spec §4.6), and one off the
/// on-air output never asks first (on-air pre-listen spec CA3).
fn give_cue_routes(state: &mut AppState) {
    let phones = fp_model::Route {
        backend: "null".to_owned(),
        device: "phones".to_owned(),
        first_channel: 0,
    };
    state.config.outputs.routes = state
        .players
        .iter()
        .map(|p| fp_model::PlayerRoutes {
            player: p.id,
            main: None,
            cue: Some(phones.clone()),
        })
        .collect();
    state.config.outputs.cartwall.cue = Some(phones);
}

// Bit-perfect rate and fader spec BF10, BR16.

fn dac() -> OutputDevice {
    OutputDevice {
        backend: "null".into(),
        device: "dac".into(),
    }
}

/// P1's Main placed on `dac`, which runs bit-perfect: its fader is locked.
fn locked(mut s: AppState) -> AppState {
    let p = s.players[0].id;
    let route = fp_model::Route {
        backend: "null".into(),
        device: "dac".into(),
        first_channel: 0,
    };
    s.config
        .outputs
        .routes
        .iter_mut()
        .find(|r| r.player == p)
        .unwrap()
        .main = Some(route.clone());
    s.config.outputs.bit_perfect = vec![dac()];
    let running = device_settings(&s.config.outputs, &dac());
    on_event(
        &mut s,
        EngineEvent::Placed {
            holder: Holder::PlayerMain(p),
            route: Some(route),
            device: dac(),
            running,
        },
    );
    assert!(fp_model::fader_locked(&s, p));
    s
}

/// P1 back on a device that is not bit-perfect: unlocked at 100 % (BF4).
fn unlocked(mut s: AppState) -> AppState {
    let p = s.players[0].id;
    let speakers = OutputDevice {
        backend: "null".into(),
        device: "speakers".into(),
    };
    let running = device_settings(&s.config.outputs, &speakers);
    on_event(
        &mut s,
        EngineEvent::Placed {
            holder: Holder::PlayerMain(p),
            route: None,
            device: speakers,
            running,
        },
    );
    assert!(!fp_model::fader_locked(&s, p));
    s
}

#[test]
fn bf10_a_fader_binding_of_a_locked_player_gives_nothing() {
    let s = locked(at_travel(state(), travel_of(50)));
    let mut r = volume_router();
    for v in [0, 50, 100, 127] {
        assert_eq!(r.on_message("APC", cc_value(7, v), &s), None, "{v}");
    }
}

#[test]
fn bf10_after_unlocking_a_control_below_the_top_waits_for_the_top() {
    let s = locked(state());
    let mut r = volume_router();
    assert_eq!(r.on_message("APC", cc_value(7, 40), &s), None);
    let s = unlocked(s);
    for v in [40, 50, 64, 100, 126] {
        assert_eq!(
            r.on_message("APC", cc_value(7, v), &s),
            None,
            "{v}: not picked up"
        );
    }
    let got = volume_of(r.on_message("APC", cc_value(7, 127), &s)).unwrap();
    assert!((got - 1.0).abs() < 1e-3, "picked up at the top");
    assert!(
        r.on_message("APC", cc_value(7, 100), &s).is_some(),
        "then it moves the volume"
    );
}

#[test]
fn bf10_a_control_at_the_top_picks_up_at_once_after_unlocking() {
    let s = locked(state());
    let mut r = volume_router();
    assert_eq!(r.on_message("APC", cc_value(7, 127), &s), None);
    let s = unlocked(s);
    assert!(r.on_message("APC", cc_value(7, 127), &s).is_some());
}

#[test]
fn br16_play_and_previous_bindings_work_on_a_bit_perfect_player() {
    let s = locked(state());
    let p1 = s.players[0].id;
    let mut r = router(&[
        (note(1), MidiAction::Button(ShortcutAction::PlayPlayer(1))),
        (
            note(2),
            MidiAction::Button(ShortcutAction::PreviousPlayer(1)),
        ),
    ]);
    assert_eq!(
        r.on_message("APC", on(1), &s),
        Some(Command::Play(p1)),
        "accepted"
    );
    // After a hard Next there is a history entry to go back to.
    let mut s = s;
    apply(&mut s, Command::Play(p1)).unwrap();
    assert!(!s.player(p1).unwrap().fading, "a hard start (BR7)");
    assert_eq!(r.on_message("APC", on(2), &s), Some(Command::Previous(p1)));
}

/// P1 (playing) and P2's Cue share `desk` (on-air pre-listen spec §2): a
/// pre-listen of P2 would be heard on air.
fn p2_cue_on_air(mut s: AppState) -> AppState {
    let desk = fp_model::Route {
        backend: "null".into(),
        device: "desk".into(),
        first_channel: 0,
    };
    let device = OutputDevice {
        backend: "null".into(),
        device: "desk".into(),
    };
    let (p1, p2) = (s.players[0].id, s.players[1].id);
    for r in &mut s.config.outputs.routes {
        if r.player == p1 {
            r.main = Some(desk.clone());
        }
        if r.player == p2 {
            r.cue = Some(desk.clone());
        }
    }
    let running = device_settings(&s.config.outputs, &device);
    for holder in [Holder::PlayerMain(p1), Holder::PlayerCue(p2)] {
        on_event(
            &mut s,
            EngineEvent::Placed {
                holder,
                route: Some(desk.clone()),
                device: device.clone(),
                running,
            },
        );
    }
    s
}

/// CA6, Review Focus 4.
#[test]
fn ca6_a_cue_button_that_would_ask_does_nothing() {
    let s = p2_cue_on_air(state());
    let (p1, p2) = (s.players[0].id, s.players[1].id);
    let mut r = router(&[(note(1), MidiAction::Button(ShortcutAction::CuePlayer(2)))]);
    assert_eq!(r.on_message("APC", on(1), &s), None, "MIDI cannot confirm");
    let mut quiet = s.clone();
    apply(&mut quiet, Command::Stop(p1)).unwrap();
    assert_eq!(
        r.on_message("APC", on(1), &quiet),
        Some(Command::ToggleCue(p2))
    );
    // A running pre-listen on the sounding output can always be stopped.
    let mut running = quiet.clone();
    apply(&mut running, Command::ToggleCue(p2)).unwrap();
    let entry = running.player(p2).unwrap().cue.unwrap().entry;
    apply(
        &mut running,
        Command::ForceCue(fp_model::CueStart::Entry { player: p2, entry }),
    )
    .unwrap();
    apply(&mut running, Command::Play(p1)).unwrap();
    assert_eq!(
        r.on_message("APC", on(1), &running),
        Some(Command::ToggleCue(p2))
    );
}

// Why a message did nothing, for the service's debug log (J6).

#[test]
fn route_says_why_a_message_did_nothing() {
    use fp_control::router::Ignored;
    let s = at_travel(state(), travel_of(100));
    let mut r = router(&[
        (note(1), MidiAction::Button(ShortcutAction::StopPlayer(2))),
        (note(2), MidiAction::Button(ShortcutAction::PlayPlayer(5))),
        (cc(20), MidiAction::Button(ShortcutAction::PlayPlayer(2))),
        (cc(7), MidiAction::Volume(1)),
    ]);
    let off = MidiMessage::NoteOff {
        channel: 0,
        note: 1,
    };
    assert_eq!(r.route("APC", off, &s), Err(Ignored::Release));
    assert_eq!(r.route("Other", on(1), &s), Err(Ignored::Unbound));
    assert_eq!(r.route("APC", on(9), &s), Err(Ignored::Unbound));
    assert_eq!(r.route("APC", on(1), &s), Err(Ignored::Unavailable));
    assert_eq!(r.route("APC", on(2), &s), Err(Ignored::NoSuchPlayer));
    assert_eq!(
        r.route("APC", cc_value(20, 127), &s),
        Err(Ignored::NotPressed),
        "the first value is the control's state"
    );
    assert_eq!(
        r.route("APC", cc_value(7, 10), &s),
        Err(Ignored::NotPickedUp)
    );
    assert!(r.route("APC", cc_value(7, 105), &s).is_ok());
}
