#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Spec §4.6 and §3 rule 26: a CUE needs a Cue output that is not the Main
//! output, because pre-listening must never go on air.

mod common;

use std::path::PathBuf;

use common::{entries, fixture, p0};
use fp_model::{
    AppState, CartwallRoutes, Command, EngineAction, PlayerId, PlayerRoutes, Route, TrackAnalysis,
    apply, availability, command_available, config::cue_equals_main, config::cue_is_usable,
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

fn equal_routes() -> (AppState, PlayerId) {
    with_routes(Some(route("desk", 0)), Some(route("desk", 0)))
}

#[test]
fn a_cue_route_on_another_device_is_usable() {
    assert!(cue_is_usable(
        Some(&route("desk", 0)),
        Some(&route("phones", 0))
    ));
}

#[test]
fn a_cue_route_on_another_channel_pair_of_the_main_device_is_usable() {
    assert!(cue_is_usable(
        Some(&route("desk", 0)),
        Some(&route("desk", 2))
    ));
}

#[test]
fn a_cue_route_on_another_backend_is_usable() {
    let mut other = route("desk", 0);
    other.backend = "jack".to_owned();
    assert!(cue_is_usable(Some(&route("desk", 0)), Some(&other)));
}

#[test]
fn a_cue_route_equal_to_main_is_not_usable() {
    assert!(!cue_is_usable(
        Some(&route("desk", 0)),
        Some(&route("desk", 0))
    ));
    assert!(cue_equals_main(
        Some(&route("desk", 0)),
        Some(&route("desk", 0))
    ));
}

#[test]
fn no_cue_route_is_not_usable_and_not_equal_to_main() {
    assert!(!cue_is_usable(Some(&route("desk", 0)), None));
    assert!(!cue_is_usable(None, None));
    assert!(!cue_equals_main(None, None));
    assert!(!cue_equals_main(Some(&route("desk", 0)), None));
}

/// The engine sends a missing Main route to the default output, which only
/// the running backend knows: from the configuration alone the Cue route is
/// taken as usable.
#[test]
fn a_cue_route_with_the_default_main_output_is_usable() {
    assert!(cue_is_usable(None, Some(&route("desk", 0))));
    assert!(!cue_equals_main(None, Some(&route("desk", 0))));
}

#[test]
fn outputs_config_answers_per_player_and_for_the_cartwall() {
    let (mut s, p) = equal_routes();
    let other = s.players[1].id;
    assert!(!s.config.outputs.player_has_cue(p));
    assert!(s.config.outputs.player_cue_equals_main(p));
    assert!(!s.config.outputs.player_has_cue(other), "no routes at all");
    assert!(!s.config.outputs.player_cue_equals_main(other));
    set_routes(&mut s, p, Some(route("desk", 0)), Some(route("phones", 0)));
    assert!(s.config.outputs.player_has_cue(p));
    assert!(!s.config.outputs.player_cue_equals_main(p));

    s.config.outputs.cartwall = CartwallRoutes::default();
    assert!(!s.config.outputs.cartwall_has_cue());
    s.config.outputs.cartwall = CartwallRoutes {
        main: Some(route("desk", 2)),
        cue: Some(route("desk", 2)),
    };
    assert!(!s.config.outputs.cartwall_has_cue());
    assert!(s.config.outputs.cartwall_cue_equals_main());
    s.config.outputs.cartwall.cue = Some(route("phones", 0));
    assert!(s.config.outputs.cartwall_has_cue());
    assert!(!s.config.outputs.cartwall_cue_equals_main());
}

#[test]
fn rule_26_cue_is_unavailable_when_the_cue_route_equals_main() {
    let (s, p) = equal_routes();
    assert!(!availability(&s, p).cue);
    assert!(!command_available(&s, &Command::ToggleCue(p)));
}

#[test]
fn rule_26_cue_is_unavailable_without_a_cue_route() {
    let (s, p) = with_routes(Some(route("desk", 0)), None);
    assert!(!availability(&s, p).cue);
}

#[test]
fn rule_26_cue_is_available_with_a_separate_cue_route() {
    let (s, p) = with_routes(Some(route("desk", 0)), Some(route("phones", 0)));
    assert!(availability(&s, p).cue);
}

#[test]
fn toggle_cue_does_nothing_without_a_usable_cue_output() {
    let (mut s, p) = equal_routes();
    let actions = apply(&mut s, Command::ToggleCue(p)).unwrap();
    assert!(actions.is_empty(), "{actions:?}");
    assert!(s.player(p).unwrap().cue.is_none());
}

#[test]
fn set_cue_on_does_nothing_without_a_usable_cue_output() {
    let (mut s, p) = equal_routes();
    let actions = apply(&mut s, Command::SetCue(p, true)).unwrap();
    assert!(actions.is_empty(), "{actions:?}");
    assert!(s.player(p).unwrap().cue.is_none());
}

#[test]
fn cue_entry_does_nothing_without_a_usable_cue_output() {
    let (mut s, p) = equal_routes();
    let e = entries(&s)[1];
    let actions = apply(&mut s, Command::CueEntry(p, e)).unwrap();
    assert!(actions.is_empty(), "{actions:?}");
    assert!(s.player(p).unwrap().cue.is_none());
}

#[test]
fn a_running_cue_still_stops_after_its_output_became_unusable() {
    let (mut s, p) = with_routes(Some(route("desk", 0)), Some(route("phones", 0)));
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    set_routes(&mut s, p, Some(route("desk", 0)), Some(route("desk", 0)));
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
fn a_cart_cue_does_nothing_when_the_cartwall_cue_equals_main() {
    let mut s = fixture(0);
    let cart = cart_with_file(&mut s);
    s.config.outputs.cartwall = CartwallRoutes {
        main: Some(route("desk", 0)),
        cue: Some(route("desk", 0)),
    };
    let actions = apply(&mut s, Command::CueCart(cart)).unwrap();
    assert!(actions.is_empty(), "{actions:?}");
    assert_eq!(s.cartwall.cue, None);
    let actions = apply(&mut s, Command::SetCartCue(cart, true)).unwrap();
    assert!(actions.is_empty(), "{actions:?}");
    assert_eq!(s.cartwall.cue, None);
}

#[test]
fn a_cart_cue_starts_with_a_separate_cartwall_cue_route() {
    let mut s = fixture(0);
    let cart = cart_with_file(&mut s);
    s.config.outputs.cartwall = CartwallRoutes {
        main: Some(route("desk", 0)),
        cue: Some(route("phones", 0)),
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
fn cue_entry_and_cart_cue_commands_are_unavailable_without_a_cue_output() {
    let (mut s, p) = equal_routes();
    let e = entries(&s)[1];
    assert!(!command_available(&s, &Command::CueEntry(p, e)));
    set_routes(&mut s, p, Some(route("desk", 0)), Some(route("phones", 0)));
    assert!(command_available(&s, &Command::CueEntry(p, e)));

    let cart = cart_with_file(&mut s);
    s.config.outputs.cartwall = CartwallRoutes {
        main: Some(route("phones", 0)),
        cue: Some(route("phones", 0)),
    };
    assert!(!command_available(&s, &Command::CueCart(cart)));
    s.config.outputs.cartwall.main = None;
    assert!(command_available(&s, &Command::CueCart(cart)));
    apply(&mut s, Command::CueCart(cart)).unwrap();
    s.config.outputs.cartwall.main = s.config.outputs.cartwall.cue.clone();
    assert!(
        command_available(&s, &Command::CueCart(cart)),
        "a running cart cue can be stopped"
    );
}
