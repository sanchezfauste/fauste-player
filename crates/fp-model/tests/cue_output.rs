#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Spec §4.6, rule 26 and C9 as amended by the on-air pre-listen spec CA1:
//! a CUE needs a Cue route; one that overlaps a Main route is usable.

mod common;

use std::path::PathBuf;

use common::{entries, fixture, p0};
use fp_model::Holder;
use fp_model::{
    AppState, CartwallRoutes, Command, EngineAction, PlayerId, PlayerRoutes, Route, TrackAnalysis,
    apply, availability, command_available,
    config::{cue_is_usable, routes_overlap},
};

fn route(device: &str, first_channel: u16) -> Route {
    Route {
        backend: "cpal".to_owned(),
        device: device.to_owned(),
        first_channel,
    }
}

fn set_routes(state: &mut AppState, player: PlayerId, main: Option<Route>, cue: Option<Route>) {
    state.config.outputs.routes = vec![PlayerRoutes { player, main, cue }];
}

/// A fixture whose first player has `main` and `cue` as its routes.
fn with_routes(main: Option<Route>, cue: Option<Route>) -> (AppState, PlayerId) {
    let mut s = fixture(3);
    let p = p0(&s);
    set_routes(&mut s, p, main, cue);
    (s, p)
}

fn no_cue_route() -> (AppState, PlayerId) {
    with_routes(Some(route("desk", 0)), None)
}

#[test]
fn ca1_any_cue_route_is_usable_even_the_main_one() {
    for main in [None, Some(route("desk", 0)), Some(route("phones", 0))] {
        assert!(cue_is_usable(Some(&route("desk", 0))), "{main:?}");
    }
    assert!(!cue_is_usable(None));
}

#[test]
fn ca_terms_routes_overlap_on_a_shared_channel_of_one_device() {
    assert!(routes_overlap(&route("desk", 0), &route("desk", 0)));
    assert!(routes_overlap(&route("desk", 0), &route("desk", 1)));
    assert!(routes_overlap(&route("desk", 3), &route("desk", 2)));
    assert!(!routes_overlap(&route("desk", 0), &route("desk", 2)));
    assert!(!routes_overlap(&route("desk", 0), &route("phones", 0)));
    let mut other = route("desk", 0);
    other.backend = "jack".to_owned();
    assert!(!routes_overlap(&route("desk", 0), &other));
}

#[test]
fn ca1_cue_shares_main_looks_at_every_main_route() {
    let mut s = fixture(3);
    let (p, q) = (s.players[0].id, s.players[1].id);
    s.config.outputs.routes = vec![
        PlayerRoutes {
            player: p,
            main: Some(route("speakers", 0)),
            cue: Some(route("desk", 0)),
        },
        PlayerRoutes {
            player: q,
            main: Some(route("desk", 1)),
            cue: Some(route("phones", 0)),
        },
    ];
    let outputs = &s.config.outputs;
    assert!(outputs.cue_shares_main(Holder::PlayerCue(p)), "Q's Main");
    assert!(!outputs.cue_shares_main(Holder::PlayerCue(q)));
    assert!(!outputs.cue_shares_main(Holder::PlayerMain(p)), "not a Cue");
    s.config.outputs.cartwall = CartwallRoutes {
        main: Some(route("phones", 0)),
        cue: Some(route("speakers", 1)),
    };
    let outputs = &s.config.outputs;
    assert!(
        outputs.cue_shares_main(Holder::PlayerCue(q)),
        "the cartwall's Main"
    );
    assert!(outputs.cue_shares_main(Holder::CartwallCue), "P's Main");
    s.config.outputs.cartwall.cue = None;
    assert!(!s.config.outputs.cue_shares_main(Holder::CartwallCue));
}

#[test]
fn ca1_outputs_config_answers_per_player_and_for_the_cartwall() {
    let (mut s, p) = with_routes(Some(route("desk", 0)), Some(route("desk", 0)));
    let other = s.players[1].id;
    assert!(s.config.outputs.player_has_cue(p), "Cue on Main is usable");
    assert!(!s.config.outputs.player_has_cue(other), "no routes at all");
    set_routes(&mut s, p, Some(route("desk", 0)), None);
    assert!(!s.config.outputs.player_has_cue(p));

    s.config.outputs.cartwall = CartwallRoutes::default();
    assert!(!s.config.outputs.cartwall_has_cue());
    s.config.outputs.cartwall = CartwallRoutes {
        main: Some(route("desk", 2)),
        cue: Some(route("desk", 2)),
    };
    assert!(s.config.outputs.cartwall_has_cue());
}

#[test]
fn ca1_rule_26_cue_is_available_with_a_cue_route_on_main() {
    let (s, p) = with_routes(Some(route("desk", 0)), Some(route("desk", 0)));
    assert!(availability(&s, p).cue);
    assert!(command_available(&s, &Command::ToggleCue(p)));
}

#[test]
fn rule_26_cue_is_unavailable_without_a_cue_route() {
    let (s, p) = no_cue_route();
    assert!(!availability(&s, p).cue);
}

#[test]
fn ca1_a_cue_on_the_main_route_starts_while_nothing_is_placed() {
    let (mut s, p) = with_routes(Some(route("desk", 0)), Some(route("desk", 0)));
    let actions = apply(&mut s, Command::ToggleCue(p)).unwrap();
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EngineAction::StartCue { .. })),
        "{actions:?}"
    );
    assert!(s.player(p).unwrap().cue.is_some());
}

#[test]
fn toggle_set_and_cue_entry_do_nothing_without_a_cue_route() {
    let (mut s, p) = no_cue_route();
    let e = entries(&s)[1];
    for command in [
        Command::ToggleCue(p),
        Command::SetCue(p, true),
        Command::CueEntry(p, e),
    ] {
        let actions = apply(&mut s, command.clone()).unwrap();
        assert!(actions.is_empty(), "{command:?}: {actions:?}");
        assert!(s.player(p).unwrap().cue.is_none(), "{command:?}");
    }
}

#[test]
fn a_running_cue_still_stops_after_its_cue_route_was_removed() {
    let (mut s, p) = with_routes(Some(route("desk", 0)), Some(route("phones", 0)));
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    set_routes(&mut s, p, Some(route("desk", 0)), None);
    assert!(availability(&s, p).cue, "a running cue can be stopped");
    let actions = apply(&mut s, Command::ToggleCue(p)).unwrap();
    assert_eq!(actions, vec![EngineAction::StopCue { player: p }]);
}

fn cart_with_file(state: &mut AppState) -> fp_model::CartId {
    let page = state.cartwall.pages[0].id;
    apply(
        state,
        Command::AssignCartFile {
            page,
            index: 0,
            path: PathBuf::from("/carts/jingle.wav"),
        },
    )
    .unwrap();
    let track = state.cartwall.pages[0].carts[0].track.unwrap();
    apply(
        state,
        Command::ApplyAnalysis {
            track,
            analysis: Box::new(TrackAnalysis {
                duration_secs: 5.0,
                ..TrackAnalysis::default()
            }),
        },
    )
    .unwrap();
    state.cartwall.pages[0].carts[0].id
}

#[test]
fn a_cart_cue_does_nothing_without_a_cartwall_cue_route() {
    let mut s = fixture(0);
    let cart = cart_with_file(&mut s);
    s.config.outputs.cartwall = CartwallRoutes {
        main: Some(route("desk", 0)),
        cue: None,
    };
    for command in [Command::CueCart(cart), Command::SetCartCue(cart, true)] {
        let actions = apply(&mut s, command).unwrap();
        assert!(actions.is_empty(), "{actions:?}");
        assert_eq!(s.cartwall.cue, None);
    }
}

#[test]
fn ca1_a_cart_cue_starts_on_the_cartwalls_main_route() {
    let mut s = fixture(0);
    let cart = cart_with_file(&mut s);
    s.config.outputs.cartwall = CartwallRoutes {
        main: Some(route("desk", 0)),
        cue: Some(route("desk", 0)),
    };
    let actions = apply(&mut s, Command::CueCart(cart)).unwrap();
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EngineAction::StartCartCue(_))),
        "{actions:?}"
    );
    assert_eq!(s.cartwall.cue, Some(cart));
}

#[test]
fn cue_entry_and_cart_cue_commands_are_unavailable_without_a_cue_route() {
    let (mut s, p) = no_cue_route();
    let e = entries(&s)[1];
    assert!(!command_available(&s, &Command::CueEntry(p, e)));
    set_routes(&mut s, p, Some(route("desk", 0)), Some(route("desk", 0)));
    assert!(command_available(&s, &Command::CueEntry(p, e)));

    let cart = cart_with_file(&mut s);
    s.config.outputs.cartwall = CartwallRoutes {
        main: Some(route("phones", 0)),
        cue: None,
    };
    assert!(!command_available(&s, &Command::CueCart(cart)));
    s.config.outputs.cartwall.cue = Some(route("phones", 0));
    assert!(command_available(&s, &Command::CueCart(cart)));
    apply(&mut s, Command::CueCart(cart)).unwrap();
    s.config.outputs.cartwall.cue = None;
    assert!(
        command_available(&s, &Command::CueCart(cart)),
        "a running cart cue can be stopped"
    );
}
