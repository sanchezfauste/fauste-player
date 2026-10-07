#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The DSD badge, the "others muted" notice and the fader's tooltip (O25).

mod support;

use std::sync::Arc;

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable;
use fp_backends::{AudioBackend, OfflineBackend};
use fp_model::{DsdOnAir, OutputDevice, OutputsView, PlayerRoutes, Route, Transport};
use support::{Fake, harness, harness_with_backends, state};

fn publish(fake: &Fake, bit_perfect: bool, dsd: bool) {
    fake.telemetry
        .store(std::sync::Arc::new(fp_engine::conductor::Telemetry {
            players: vec![(
                fake.player(0),
                fp_engine::engine::PlayerTelemetry {
                    bit_perfect,
                    dsd,
                    ..Default::default()
                },
            )],
            ..Default::default()
        }));
}

#[test]
fn the_badge_says_dsd_while_dsd_reaches_the_device() {
    let (mut h, fake) = harness(state(1, 1));
    publish(&fake, true, false);
    h.run_steps(2);
    assert!(h.query_by_label("Bit-perfect: on").is_some());
    assert!(
        h.query_by_label("DSD reaches the device unchanged")
            .is_none()
    );
    publish(&fake, true, true);
    h.run_steps(2);
    assert!(
        h.query_by_label("DSD reaches the device unchanged")
            .is_some()
    );
    assert!(h.query_by_label("Bit-perfect: on").is_none());
}

#[test]
fn the_fader_says_dsd_plays_at_full_volume() {
    let (mut h, fake) = harness(state(1, 1));
    h.run_steps(2);
    assert!(h.query_by_label_contains("DSD plays at 100 %").is_none());
    publish(&fake, true, true);
    h.run_steps(2);
    assert!(h.query_by_label_contains("DSD plays at 100 %").is_some());
}

#[test]
fn others_muted_shows_while_dsd_holds_the_output() {
    let (mut h, fake) = harness(state(1, 1));
    h.run_steps(2);
    assert!(
        h.query_by_label_contains("other sources routed to it are muted")
            .is_none()
    );
    let mut s = (**fake.state.load()).clone();
    let entry = fake.entries()[0];
    let p = &mut s.players[0];
    p.current = Some(entry);
    p.transport = Transport::Playing;
    p.dsd = Some(DsdOnAir {
        entry,
        hold_others: true,
    });
    fake.state.store(std::sync::Arc::new(s));
    h.run_steps(2);
    assert!(
        h.query_by_label_contains("other sources routed to it are muted")
            .is_some()
    );
}

/// Player 1 plays on `dac` (exclusive-capable, no DSD format); Settings is
/// open on Audio outputs, Advanced.
fn advanced_outputs(bit_perfect: bool) -> egui_kittest::Harness<'static, fp_app::ui::app::AppUi> {
    let backend = OfflineBackend::new();
    backend.add_device("dac", 2).set_exclusive_capable(true);
    let mut s = state(1, 1);
    s.config.ui.outputs_view = OutputsView::Advanced;
    s.config.outputs.backend = Some("offline".into());
    s.config.outputs.routes = vec![PlayerRoutes {
        player: s.players[0].id,
        main: Some(Route {
            backend: "offline".into(),
            device: "dac".into(),
            first_channel: 0,
        }),
        cue: None,
    }];
    if bit_perfect {
        s.config.outputs.bit_perfect = vec![OutputDevice {
            backend: "offline".into(),
            device: "dac".into(),
        }];
    }
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
    let (mut h, _) = harness_with_backends(s, backends);
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Audio outputs")
        .click();
    for _ in 0..200 {
        h.run_steps(1);
        if h.query_by_role_and_label(Role::ComboBox, "DSD: dac")
            .is_some()
        {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    h
}

#[test]
fn a_device_that_is_not_bit_perfect_shows_its_dsd_row_and_why() {
    let h = advanced_outputs(false);
    assert!(
        h.query_by_role_and_label(Role::ComboBox, "DSD: dac")
            .is_some()
    );
    assert!(
        h.query_by_label("DoP and native DSD need this device to be bit-perfect.")
            .is_some()
    );
    assert!(
        h.query_all_by_label("By default, DSD is converted to PCM.")
            .next()
            .is_some()
    );
}

#[test]
fn a_bit_perfect_device_without_native_dsd_says_why() {
    let h = advanced_outputs(true);
    let why = if cfg!(target_os = "linux") {
        "This device does not take native DSD: its driver reports no DSD format."
    } else {
        "Native DSD needs Linux."
    };
    assert!(h.query_by_label(why).is_some(), "{why}");
    assert!(
        h.query_by_label("DoP and native DSD need this device to be bit-perfect.")
            .is_none()
    );
}
