#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! On-air pre-listen spec (2026-10-15): the §2 terms and CA1–CA7, one test
//! per rule.

mod common;

use common::fixture;
use fp_model::cue_on_air::{
    HolderOutput, cue_is_on_air, holder_output, outputs_overlap, sounding_on,
};
use fp_model::{
    AppState, Command, EngineAction, EngineEvent, Holder, OutputDevice, PlayerId, Route, apply,
    device_settings, on_event,
};

fn route(device: &str, first_channel: u16) -> Route {
    Route {
        backend: "null".to_owned(),
        device: device.to_owned(),
        first_channel,
    }
}

fn device(name: &str) -> OutputDevice {
    OutputDevice {
        backend: "null".to_owned(),
        device: name.to_owned(),
    }
}

/// Configures `route` for `holder` (`None`: the default output) and reports
/// the engine placing `holder` on device `on` with it (live settings L3).
fn place(s: &mut AppState, holder: Holder, route: Option<Route>, on: &str) -> Vec<EngineAction> {
    let outputs = &mut s.config.outputs;
    match holder {
        Holder::PlayerMain(p) | Holder::PlayerCue(p) => {
            let routes = outputs.routes.iter_mut().find(|r| r.player == p).unwrap();
            if matches!(holder, Holder::PlayerMain(_)) {
                routes.main = route.clone();
            } else {
                routes.cue = route.clone();
            }
        }
        Holder::CartwallMain => outputs.cartwall.main = route.clone(),
        Holder::CartwallCue => outputs.cartwall.cue = route.clone(),
    }
    let running = device_settings(&s.config.outputs, &device(on));
    on_event(
        s,
        EngineEvent::Placed {
            holder,
            route,
            device: device(on),
            running,
        },
    )
}

/// Places `holder` on `desk`, on the pair from `first_channel`.
fn on_desk(s: &mut AppState, holder: Holder, first_channel: u16) -> Vec<EngineAction> {
    place(s, holder, Some(route("desk", first_channel)), "desk")
}

/// Four players over four tracks. Player 1's Main is on `speakers`, its Cue
/// and Player 2's Main share `desk` 1–2. Nothing plays.
fn shared() -> (AppState, PlayerId, PlayerId) {
    let mut s = fixture(4);
    let (p1, p2) = (s.players[0].id, s.players[1].id);
    place(
        &mut s,
        Holder::PlayerMain(p1),
        Some(route("speakers", 0)),
        "speakers",
    );
    on_desk(&mut s, Holder::PlayerCue(p1), 0);
    on_desk(&mut s, Holder::PlayerMain(p2), 0);
    (s, p1, p2)
}

fn out(name: &str, first_channel: u16) -> HolderOutput {
    HolderOutput {
        device: device(name),
        first_channel,
    }
}

#[test]
fn ca_terms_outputs_overlap_on_a_shared_channel_of_one_device() {
    assert!(outputs_overlap(&out("desk", 0), &out("desk", 0)));
    assert!(outputs_overlap(&out("desk", 0), &out("desk", 1)));
    assert!(outputs_overlap(&out("desk", 3), &out("desk", 2)));
    assert!(!outputs_overlap(&out("desk", 0), &out("desk", 2)));
    assert!(!outputs_overlap(&out("desk", 0), &out("phones", 0)));
}

#[test]
fn ca_terms_a_holder_plays_where_the_engine_placed_it() {
    let (s, p1, _) = shared();
    assert_eq!(
        holder_output(&s, Holder::PlayerCue(p1)),
        Some(out("desk", 0))
    );
    assert_eq!(
        holder_output(&s, Holder::PlayerMain(p1)),
        Some(out("speakers", 0))
    );
    let p3 = s.players[2].id;
    assert_eq!(
        holder_output(&s, Holder::PlayerMain(p3)),
        None,
        "not placed"
    );
}

/// Review Focus 2.
#[test]
fn ca_terms_a_missing_main_route_plays_on_the_first_pair() {
    let mut s = fixture(4);
    let (p1, p2) = (s.players[0].id, s.players[1].id);
    place(&mut s, Holder::PlayerMain(p2), None, "desk");
    assert_eq!(
        holder_output(&s, Holder::PlayerMain(p2)),
        Some(out("desk", 0))
    );
    on_desk(&mut s, Holder::PlayerCue(p1), 0);
    assert!(cue_is_on_air(&s, Holder::PlayerCue(p1)));
    on_desk(&mut s, Holder::PlayerCue(p1), 2);
    assert!(!cue_is_on_air(&s, Holder::PlayerCue(p1)));
}

#[test]
fn ca_terms_a_route_the_engine_moved_to_another_device_plays_on_its_first_pair() {
    let mut s = fixture(4);
    let p2 = s.players[1].id;
    // A route to a backend this machine lacks: the engine plays it on the
    // default output, on channels 1–2.
    place(
        &mut s,
        Holder::PlayerMain(p2),
        Some(route("gone", 4)),
        "desk",
    );
    assert_eq!(
        holder_output(&s, Holder::PlayerMain(p2)),
        Some(out("desk", 0))
    );
}

#[test]
fn ca_terms_another_players_main_puts_a_cue_on_air() {
    let (s, p1, p2) = shared();
    assert!(cue_is_on_air(&s, Holder::PlayerCue(p1)));
    assert!(!cue_is_on_air(&s, Holder::PlayerCue(p2)), "not placed");
}

#[test]
fn ca_terms_unequal_pairs_overlap_and_separate_pairs_do_not() {
    let (mut s, p1, p2) = shared();
    on_desk(&mut s, Holder::PlayerMain(p2), 1);
    assert!(cue_is_on_air(&s, Holder::PlayerCue(p1)), "1–2 and 2–3");
    on_desk(&mut s, Holder::PlayerMain(p2), 2);
    assert!(!cue_is_on_air(&s, Holder::PlayerCue(p1)), "1–2 and 3–4");
}

#[test]
fn ca_terms_a_cue_next_to_another_cue_only_is_not_on_air() {
    let (mut s, p1, p2) = shared();
    place(
        &mut s,
        Holder::PlayerMain(p2),
        Some(route("speakers", 2)),
        "speakers",
    );
    let p3 = s.players[2].id;
    on_desk(&mut s, Holder::PlayerCue(p3), 0);
    assert!(!cue_is_on_air(&s, Holder::PlayerCue(p1)));
    apply(&mut s, Command::ToggleCue(p3)).unwrap();
    assert!(
        sounding_on(&s, Holder::PlayerCue(p1)).is_empty(),
        "not on air"
    );
}

#[test]
fn ca_terms_what_sounds_on_an_on_air_output() {
    let (mut s, p1, p2) = shared();
    let cue = Holder::PlayerCue(p1);
    assert!(sounding_on(&s, cue).is_empty(), "quiet");
    apply(&mut s, Command::Play(p2)).unwrap();
    assert_eq!(sounding_on(&s, cue), vec![Holder::PlayerMain(p2)]);
    apply(&mut s, Command::Pause(p2)).unwrap();
    assert_eq!(
        sounding_on(&s, cue),
        vec![Holder::PlayerMain(p2)],
        "a player paused with a track sounds (BR3.5)"
    );
    apply(&mut s, Command::Stop(p2)).unwrap();
    assert!(sounding_on(&s, cue).is_empty());
}

#[test]
fn ca_terms_another_pre_listen_sounds_and_the_cue_decided_is_left_out() {
    let (mut s, p1, _) = shared();
    let p3 = s.players[2].id;
    on_desk(&mut s, Holder::PlayerCue(p3), 0);
    apply(&mut s, Command::ToggleCue(p3)).unwrap();
    assert_eq!(
        sounding_on(&s, Holder::PlayerCue(p1)),
        vec![Holder::PlayerCue(p3)]
    );
    assert!(
        sounding_on(&s, Holder::PlayerCue(p3)).is_empty(),
        "its own pre-listen does not count"
    );
}

#[test]
fn ca_terms_the_runtime_state_starts_empty() {
    let s = fixture(1);
    assert_eq!(s.cue_on_air, fp_model::cue_on_air::CueOnAir::default());
    assert_eq!(common::roundtrip(&s).cue_on_air, s.cue_on_air);
}
