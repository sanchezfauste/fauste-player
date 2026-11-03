#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Library plan 4 (LB32): the library CUE, a holder of its own.

use crate::common::fixture;
use fp_model::library_cue::LibraryCue;
use fp_model::{
    AppState, BusyCause, Config, Holder, OutputDevice, Route, Transport, causes, configured_route,
    holders,
};

fn route(device: &str, first_channel: u16) -> Route {
    Route {
        backend: "null".to_owned(),
        device: device.to_owned(),
        first_channel,
    }
}

#[test]
fn lb32_the_library_cue_is_the_last_holder() {
    let s = fixture(1);
    assert_eq!(holders(&s).last(), Some(&Holder::LibraryCue));
}

#[test]
fn lb32_its_route_is_outputs_library_cue() {
    let mut config = Config::default();
    assert_eq!(configured_route(&config, Holder::LibraryCue), None);
    assert!(!config.outputs.library_has_cue());
    config.outputs.library.cue = Some(route("lib", 2));
    assert_eq!(
        configured_route(&config, Holder::LibraryCue),
        Some(route("lib", 2))
    );
    assert!(config.outputs.library_has_cue());
}

#[test]
fn lb32_a_running_preview_is_a_cause() {
    let mut s = fixture(1);
    assert!(causes(&s, Holder::LibraryCue).is_empty());
    for transport in [Transport::Playing, Transport::Paused] {
        s.library_cue.transport = transport;
        assert_eq!(
            causes(&s, Holder::LibraryCue),
            vec![BusyCause::LibraryCue],
            "{transport:?}"
        );
    }
    s.library_cue.transport = Transport::Stopped;
    assert!(causes(&s, Holder::LibraryCue).is_empty());
}

#[test]
fn lb32_routed_devices_and_cue_shares_main_count_the_library_cue() {
    let mut s = fixture(1);
    let main = route("desk", 0);
    s.config.outputs.routes[0].main = Some(main);
    s.config.outputs.library.cue = Some(route("desk", 1));
    assert!(s.config.outputs.cue_shares_main(Holder::LibraryCue));
    s.config.outputs.library.cue = Some(route("monitor", 0));
    assert!(!s.config.outputs.cue_shares_main(Holder::LibraryCue));
    assert!(s.config.outputs.routed_devices().contains(&OutputDevice {
        backend: "null".to_owned(),
        device: "monitor".to_owned(),
    }));
}

#[test]
fn lb32_the_config_round_trips() {
    let mut config = Config::default();
    config.outputs.library.cue = Some(route("lib", 2));
    let json = serde_json::to_string(&config).unwrap();
    let back: Config = serde_json::from_str(&json).unwrap();
    assert_eq!(back, config);

    let mut value = serde_json::to_value(Config::default()).unwrap();
    value["outputs"].as_object_mut().unwrap().remove("library");
    let old: Config = serde_json::from_value(value).unwrap();
    assert_eq!(old.outputs.library.cue, None);
}

#[test]
fn rule10_a_fresh_state_has_the_library_cue_stopped() {
    let s = AppState::new(Config::default(), "Main");
    assert_eq!(s.library_cue, LibraryCue::default());
    assert_eq!(s.library_cue.transport, Transport::Stopped);
    assert_eq!(s.library_cue.track, None);
}
