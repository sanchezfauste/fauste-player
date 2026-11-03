#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Library plan 4 (LB32): the library CUE is placed from the start, without
//! a stream or a worker.

use crate::support;

use std::sync::Arc;
use std::time::Instant;

use fp_backends::{AudioBackend, OfflineBackend};
use fp_engine::engine::{Engine, EngineSettings};
use fp_model::{Config, EngineAction, EngineEvent, Holder, Route};
use support::tagged_opener;

fn route() -> Route {
    Route {
        backend: "offline".into(),
        device: "card".into(),
        first_channel: 0,
    }
}

fn rig(library: Option<Route>) -> (Engine, Config, Vec<EngineEvent>) {
    let backend = OfflineBackend::new();
    backend.add_device("card", 2);
    let mut config = Config::default();
    config.outputs.backend = Some("offline".into());
    config.outputs.library.cue = library;
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
    let mut engine = Engine::new(
        backends,
        EngineSettings::from_config(&config),
        tagged_opener(48_000),
    );
    let events = engine.tick(Instant::now());
    (engine, config, events)
}

#[test]
fn lb32_the_library_cue_is_placed_from_the_start() {
    let (_, _, events) = rig(Some(route()));
    assert!(
        events.iter().any(|e| matches!(
            e,
            EngineEvent::Placed { holder: Holder::LibraryCue, route: Some(r), .. } if *r == route()
        )),
        "{events:?}"
    );
    let (_, _, events) = rig(None);
    assert!(
        events.iter().any(|e| matches!(
            e,
            EngineEvent::Unplaced {
                holder: Holder::LibraryCue,
                route: None
            }
        )),
        "{events:?}"
    );
}

#[test]
fn lb32_a_route_set_later_is_reported_without_opening_a_stream() {
    let (mut engine, mut config, _) = rig(None);
    let before = format!("{:?}", engine.bus_status());
    config.outputs.library.cue = Some(route());
    engine.execute(
        EngineAction::UpdateSettings(Box::new(config)),
        Instant::now(),
    );
    let events = engine.tick(Instant::now());
    assert!(
        events.iter().any(|e| matches!(
            e,
            EngineEvent::Placed { holder: Holder::LibraryCue, route: Some(r), .. } if *r == route()
        )),
        "{events:?}"
    );
    assert_eq!(format!("{:?}", engine.bus_status()), before);
}
