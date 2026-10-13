#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Bit-perfect rate and fader spec (2026-10-08): one test per rule BR1–BR14
//! and BF1–BF8 that the model decides. The engine is stood in for by the
//! events it reports.

mod common;

use common::{bp_device, fixture, place_on_dac, report_stream_rate};
use fp_model::{EngineEvent, Holder};

#[test]
fn br1_a_stream_rate_report_is_kept_per_device() {
    let mut s = fixture(3);
    let p = common::p0(&s);
    place_on_dac(&mut s, Holder::PlayerMain(p));
    assert_eq!(
        s.live.stream_rates.get(&bp_device()),
        None,
        "unknown until reported"
    );
    report_stream_rate(&mut s, 96_000, vec![]);
    assert_eq!(s.live.stream_rates.get(&bp_device()), Some(&96_000));
    assert_eq!(s.live.refused_rates.get(&bp_device()), Some(&vec![]));
    report_stream_rate(&mut s, 44_100, vec![192_000]);
    assert_eq!(s.live.stream_rates.get(&bp_device()), Some(&44_100));
    assert_eq!(s.live.refused_rates.get(&bp_device()), Some(&vec![192_000]));
}

#[test]
fn br1_the_rates_are_forgotten_with_the_last_holder() {
    let mut s = fixture(3);
    let (p, q) = (s.players[0].id, s.players[1].id);
    place_on_dac(&mut s, Holder::PlayerMain(p));
    place_on_dac(&mut s, Holder::PlayerMain(q));
    report_stream_rate(&mut s, 96_000, vec![44_100]);
    fp_model::on_event(
        &mut s,
        EngineEvent::Gone {
            holder: Holder::PlayerMain(p),
        },
    );
    assert_eq!(
        s.live.stream_rates.get(&bp_device()),
        Some(&96_000),
        "q still plays there"
    );
    fp_model::on_event(
        &mut s,
        EngineEvent::Unplaced {
            holder: Holder::PlayerMain(q),
            route: None,
        },
    );
    assert_eq!(s.live.stream_rates.get(&bp_device()), None);
    assert_eq!(s.live.refused_rates.get(&bp_device()), None);
}
