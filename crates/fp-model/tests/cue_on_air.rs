#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! On-air pre-listen spec (2026-10-15): the §2 terms and CA1–CA7, one test
//! per rule.

mod common;

use std::path::PathBuf;

use common::{entries, fixture};
use fp_model::cue_on_air::{
    HolderOutput, cue_is_on_air, holder_output, needs_confirmation, outputs_overlap, sounding_on,
};
use fp_model::{
    AppState, CartId, Command, CueStart, EngineAction, EngineEvent, EntryId, Holder, OutputDevice,
    PlayerId, Route, TrackAnalysis, apply, device_settings, on_event,
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

fn next(s: &AppState, p: PlayerId) -> EntryId {
    s.player(p).unwrap().next.unwrap()
}

fn starts_cue(actions: &[EngineAction]) -> bool {
    actions
        .iter()
        .any(|a| matches!(a, EngineAction::StartCue { .. }))
}

fn cart_with_file(s: &mut AppState) -> CartId {
    let page = s.cartwall.pages[0].id;
    apply(
        s,
        Command::AssignCartFile {
            page,
            index: 0,
            path: PathBuf::from("/carts/jingle.wav"),
        },
    )
    .unwrap();
    let track = s.cartwall.pages[0].carts[0].track.unwrap();
    apply(
        s,
        Command::ApplyAnalysis {
            track,
            analysis: Box::new(TrackAnalysis {
                duration_secs: 5.0,
                ..TrackAnalysis::default()
            }),
        },
    )
    .unwrap();
    s.cartwall.pages[0].carts[0].id
}

#[test]
fn ca2_a_quiet_on_air_output_starts_the_cue_at_once() {
    let (mut s, p1, _) = shared();
    assert!(cue_is_on_air(&s, Holder::PlayerCue(p1)));
    let actions = apply(&mut s, Command::ToggleCue(p1)).unwrap();
    assert!(starts_cue(&actions), "{actions:?}");
    assert!(s.player(p1).unwrap().cue.is_some());
    assert_eq!(s.cue_on_air.request, None);
    assert!(s.cue_on_air.confirmed.is_empty(), "started, not confirmed");
}

#[test]
fn ca3_another_players_main_sounding_asks_first() {
    let (mut s, p1, p2) = shared();
    apply(&mut s, Command::Play(p2)).unwrap();
    let entry = next(&s, p1);
    let actions = apply(&mut s, Command::ToggleCue(p1)).unwrap();
    assert!(!starts_cue(&actions), "{actions:?}");
    assert!(s.player(p1).unwrap().cue.is_none());
    let request = s.cue_on_air.request.clone().unwrap();
    assert_eq!(request.holder, Holder::PlayerCue(p1));
    assert_eq!(request.start, CueStart::Entry { player: p1, entry });
    assert_eq!(request.sounding, vec![Holder::PlayerMain(p2)]);
}

#[test]
fn ca3_every_way_to_start_a_player_cue_asks() {
    let (mut s, p1, p2) = shared();
    apply(&mut s, Command::Play(p2)).unwrap();
    let e = entries(&s)[2];
    for command in [
        Command::ToggleCue(p1),
        Command::SetCue(p1, true),
        Command::CueEntry(p1, e),
    ] {
        s.cue_on_air.request = None;
        let actions = apply(&mut s, command.clone()).unwrap();
        assert!(!starts_cue(&actions), "{command:?}");
        assert!(s.cue_on_air.request.is_some(), "{command:?}");
    }
}

#[test]
fn ca3_overlapping_unequal_pairs_ask_and_separate_pairs_start() {
    let (mut s, p1, p2) = shared();
    on_desk(&mut s, Holder::PlayerMain(p2), 1);
    apply(&mut s, Command::Play(p2)).unwrap();
    apply(&mut s, Command::ToggleCue(p1)).unwrap();
    assert!(s.cue_on_air.request.is_some(), "1–2 and 2–3 overlap");
    apply(&mut s, Command::Stop(p2)).unwrap();
    on_desk(&mut s, Holder::PlayerMain(p2), 2);
    apply(&mut s, Command::Play(p2)).unwrap();
    let actions = apply(&mut s, Command::ToggleCue(p1)).unwrap();
    assert!(starts_cue(&actions), "1–2 and 3–4: no rule applies");
    assert_eq!(s.cue_on_air.request, None);
}

#[test]
fn ca3_a_paused_player_counts_as_sounding() {
    let (mut s, p1, p2) = shared();
    apply(&mut s, Command::Play(p2)).unwrap();
    apply(&mut s, Command::Pause(p2)).unwrap();
    apply(&mut s, Command::ToggleCue(p1)).unwrap();
    assert_eq!(
        s.cue_on_air.request.as_ref().map(|r| r.sounding.clone()),
        Some(vec![Holder::PlayerMain(p2)])
    );
}

#[test]
fn ca3_the_request_goes_when_the_output_stops_sounding() {
    let (mut s, p1, p2) = shared();
    apply(&mut s, Command::Play(p2)).unwrap();
    apply(&mut s, Command::ToggleCue(p1)).unwrap();
    assert!(s.cue_on_air.request.is_some());
    let actions = apply(&mut s, Command::Stop(p2)).unwrap();
    assert_eq!(s.cue_on_air.request, None);
    assert!(!starts_cue(&actions), "nothing starts by itself");
    assert!(s.player(p1).unwrap().cue.is_none());
}

#[test]
fn ca3_the_request_follows_what_sounds() {
    let (mut s, p1, p2) = shared();
    apply(&mut s, Command::Play(p2)).unwrap();
    apply(&mut s, Command::ToggleCue(p1)).unwrap();
    let p3 = s.players[2].id;
    on_desk(&mut s, Holder::PlayerMain(p3), 0);
    apply(&mut s, Command::Play(p3)).unwrap();
    assert_eq!(
        s.cue_on_air.request.as_ref().unwrap().sounding,
        vec![Holder::PlayerMain(p2), Holder::PlayerMain(p3)]
    );
}

/// Review Focus 3.
#[test]
fn ca3_a_cancel_answers_only_its_own_request() {
    let (mut s, p1, p2) = shared();
    apply(&mut s, Command::Play(p2)).unwrap();
    apply(&mut s, Command::ToggleCue(p1)).unwrap();
    let old = s.cue_on_air.request.clone().unwrap().id;
    let e = entries(&s)[2];
    apply(&mut s, Command::CueEntry(p1, e)).unwrap();
    let new = s.cue_on_air.request.clone().unwrap();
    assert!(new.id > old, "another request replaces it");
    assert_eq!(
        new.start,
        CueStart::Entry {
            player: p1,
            entry: e
        }
    );
    let actions = apply(&mut s, Command::CancelCueOnAir(old)).unwrap();
    assert!(actions.is_empty(), "{actions:?}");
    assert_eq!(s.cue_on_air.request.as_ref(), Some(&new), "a stale cancel");
    apply(&mut s, Command::CancelCueOnAir(new.id)).unwrap();
    assert_eq!(s.cue_on_air.request, None);
    assert!(s.player(p1).unwrap().cue.is_none());
}

/// Review Focus 4.
#[test]
fn ca3_a_request_goes_with_its_player() {
    let (mut s, _, p2) = shared();
    let p4 = s.players[3].id;
    on_desk(&mut s, Holder::PlayerCue(p4), 0);
    apply(&mut s, Command::Play(p2)).unwrap();
    apply(&mut s, Command::ToggleCue(p4)).unwrap();
    assert!(s.cue_on_air.request.is_some());
    apply(&mut s, Command::SetPlayerCount(3)).unwrap();
    assert_eq!(s.cue_on_air.request, None);
}

#[test]
fn ca3_a_cart_pre_listen_asks_first() {
    let (mut s, _, p2) = shared();
    let cart = cart_with_file(&mut s);
    on_desk(&mut s, Holder::CartwallCue, 0);
    apply(&mut s, Command::Play(p2)).unwrap();
    for command in [Command::CueCart(cart), Command::SetCartCue(cart, true)] {
        s.cue_on_air.request = None;
        let actions = apply(&mut s, command.clone()).unwrap();
        assert!(
            !actions
                .iter()
                .any(|a| matches!(a, EngineAction::StartCartCue(_))),
            "{command:?}: {actions:?}"
        );
        assert_eq!(s.cartwall.cue, None);
        assert_eq!(
            s.cue_on_air.request.as_ref().map(|r| r.start),
            Some(CueStart::Cart(cart))
        );
    }
}

#[test]
fn ca3_a_running_cue_that_moves_asks_nothing() {
    let (mut s, p1, p2) = shared();
    apply(&mut s, Command::Play(p2)).unwrap();
    let entry = next(&s, p1);
    apply(
        &mut s,
        Command::ForceCue(CueStart::Entry { player: p1, entry }),
    )
    .unwrap();
    let e = entries(&s)[3];
    let actions = apply(&mut s, Command::CueEntry(p1, e)).unwrap();
    assert!(starts_cue(&actions), "{actions:?}");
    assert_eq!(s.player(p1).unwrap().cue.map(|c| c.entry), Some(e));
    assert_eq!(s.cue_on_air.request, None);
    assert_eq!(s.cue_on_air.confirmed, vec![Holder::PlayerCue(p1)]);
}

#[test]
fn ca4_a_forced_start_plays_and_is_confirmed() {
    let (mut s, p1, p2) = shared();
    apply(&mut s, Command::Play(p2)).unwrap();
    apply(&mut s, Command::ToggleCue(p1)).unwrap();
    let start = s.cue_on_air.request.clone().unwrap().start;
    let actions = apply(&mut s, Command::ForceCue(start)).unwrap();
    assert!(starts_cue(&actions), "{actions:?}");
    assert!(s.player(p1).unwrap().cue.is_some());
    assert_eq!(s.cue_on_air.request, None, "answered");
    assert_eq!(s.cue_on_air.confirmed, vec![Holder::PlayerCue(p1)]);
}

#[test]
fn ca4_forcing_a_running_cue_only_confirms_it() {
    let (mut s, p1, _) = shared();
    apply(&mut s, Command::ToggleCue(p1)).unwrap();
    let entry = s.player(p1).unwrap().cue.unwrap().entry;
    let actions = apply(
        &mut s,
        Command::ForceCue(CueStart::Entry { player: p1, entry }),
    )
    .unwrap();
    assert!(!starts_cue(&actions), "no restart: {actions:?}");
    assert_eq!(s.cue_on_air.confirmed, vec![Holder::PlayerCue(p1)]);
}

#[test]
fn ca4_a_forced_cart_pre_listen_plays_and_is_confirmed() {
    let (mut s, _, p2) = shared();
    let cart = cart_with_file(&mut s);
    on_desk(&mut s, Holder::CartwallCue, 0);
    apply(&mut s, Command::Play(p2)).unwrap();
    let actions = apply(&mut s, Command::ForceCue(CueStart::Cart(cart))).unwrap();
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EngineAction::StartCartCue(_))),
        "{actions:?}"
    );
    assert_eq!(s.cartwall.cue, Some(cart));
    assert_eq!(s.cue_on_air.confirmed, vec![Holder::CartwallCue]);
}

#[test]
fn ca4_a_force_for_an_unknown_entry_or_cart_is_refused() {
    let (mut s, p1, _) = shared();
    let bad = EntryId(999_999);
    assert!(
        apply(
            &mut s,
            Command::ForceCue(CueStart::Entry {
                player: p1,
                entry: bad
            })
        )
        .is_err()
    );
    assert!(apply(&mut s, Command::ForceCue(CueStart::Cart(CartId(999_999)))).is_err());
}

#[test]
fn ca6_needs_confirmation_says_what_apply_would_do() {
    let (mut s, p1, p2) = shared();
    let e = entries(&s)[2];
    let commands = [
        Command::ToggleCue(p1),
        Command::SetCue(p1, true),
        Command::CueEntry(p1, e),
        Command::SetCue(p1, false),
        Command::Play(p1),
    ];
    let check = |s: &AppState| {
        for command in &commands {
            let mut copy = s.clone();
            apply(&mut copy, command.clone()).unwrap();
            assert_eq!(
                needs_confirmation(s, command),
                copy.cue_on_air.request.is_some(),
                "{command:?}"
            );
        }
    };
    check(&s);
    assert!(!needs_confirmation(&s, &Command::ToggleCue(p1)), "quiet");
    apply(&mut s, Command::Play(p2)).unwrap();
    check(&s);
    assert!(needs_confirmation(&s, &Command::ToggleCue(p1)), "sounding");
    let entry = next(&s, p1);
    assert!(!needs_confirmation(
        &s,
        &Command::ForceCue(CueStart::Entry { player: p1, entry })
    ));
    apply(
        &mut s,
        Command::ForceCue(CueStart::Entry { player: p1, entry }),
    )
    .unwrap();
    check(&s);
    assert!(!needs_confirmation(&s, &Command::ToggleCue(p1)), "a stop");
}

#[test]
fn ca6_needs_confirmation_for_a_cart() {
    let (mut s, _, p2) = shared();
    let cart = cart_with_file(&mut s);
    on_desk(&mut s, Holder::CartwallCue, 0);
    assert!(!needs_confirmation(&s, &Command::CueCart(cart)));
    apply(&mut s, Command::Play(p2)).unwrap();
    assert!(needs_confirmation(&s, &Command::CueCart(cart)));
    assert!(needs_confirmation(&s, &Command::SetCartCue(cart, true)));
    assert!(!needs_confirmation(&s, &Command::SetCartCue(cart, false)));
}
