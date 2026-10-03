#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The DSD badge, the "others muted" notice and the fader's tooltip (O25).

mod support;

use egui_kittest::kittest::Queryable;
use fp_model::{DsdOnAir, Transport};
use support::{Fake, harness, state};

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
