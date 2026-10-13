#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Bit-perfect rate and fader spec (2026-10-08): one test per rule BR1–BR14
//! and BF1–BF8 that the model decides. The engine is stood in for by the
//! events it reports.

mod common;

use common::{bp_device, entries, fixture, p0, place_on_dac, report_stream_rate, set_track_rates};
use fp_model::{
    AppState, Command, CueState, DsdOnAir, EngineEvent, EntryId, Holder, PlayerId, SourceRequest,
    apply, bit_perfect_player, fader_locked, rate_cut,
};

#[test]
fn br1_a_stream_rate_report_is_kept_per_device() {
    let mut s = fixture(3);
    let p = p0(&s);
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

/// `fixture(3)`: P1's Main on `dac`, which runs bit-perfect at 44.1 kHz;
/// every track 44.1 kHz 16-bit; P1 plays its first entry.
fn on_air() -> (AppState, PlayerId, [EntryId; 3]) {
    let mut s = fixture(3);
    let p = p0(&s);
    let e = entries(&s);
    place_on_dac(&mut s, Holder::PlayerMain(p));
    report_stream_rate(&mut s, 44_100, vec![]);
    set_track_rates(&mut s, &[44_100, 44_100, 44_100]);
    apply(&mut s, Command::Play(p)).unwrap();
    (s, p, [e[0], e[1], e[2]])
}

/// The request P1 preloads for `entry`, at its cue-in.
fn next_request(s: &AppState, entry: EntryId) -> SourceRequest {
    s.request_from_cue_in(entry).unwrap()
}

fn rates(s: &mut AppState, next: u32) {
    set_track_rates(s, &[44_100, next, 44_100]);
}

#[test]
fn br2_a_main_placed_on_a_device_running_bit_perfect() {
    let (s, p, _) = on_air();
    assert!(bit_perfect_player(&s, p));
    assert!(!bit_perfect_player(&s, s.players[1].id), "not placed yet");
}

#[test]
fn br2_the_running_value_counts_not_the_wanted_one() {
    let mut s = fixture(3);
    let p = p0(&s);
    place_on_dac(&mut s, Holder::PlayerMain(p));
    // Wanted off, still running on: pending until applied.
    s.config.outputs.bit_perfect.clear();
    assert!(bit_perfect_player(&s, p));
    let mut s = fixture(3);
    let p = p0(&s);
    place_on_dac(&mut s, Holder::PlayerMain(p));
    let running = s.live.devices.get_mut(&bp_device()).unwrap();
    running.bit_perfect = false;
    // Wanted on (the configuration lists it), not running yet.
    assert!(!bit_perfect_player(&s, p));
}

#[test]
fn br2_a_cue_route_on_the_device_does_not_count() {
    let mut s = fixture(3);
    let p = p0(&s);
    place_on_dac(&mut s, Holder::PlayerCue(p));
    assert!(!bit_perfect_player(&s, p));
}

#[test]
fn bf1_the_fader_is_locked_exactly_for_a_bit_perfect_player() {
    let (s, p, _) = on_air();
    assert!(fader_locked(&s, p));
    let q = s.players[1].id;
    assert_eq!(fader_locked(&s, q), bit_perfect_player(&s, q));
    assert!(!fader_locked(&s, q), "unplaced is not locked");
}

#[test]
fn br3_all_conditions_hold_gives_the_next_rate() {
    let (mut s, p, [_, b, _]) = on_air();
    rates(&mut s, 96_000);
    assert_eq!(rate_cut(&s, p, &next_request(&s, b)), Some(96_000));
}

#[test]
fn br3_1_not_a_bit_perfect_player() {
    let (mut s, p, [_, b, _]) = on_air();
    rates(&mut s, 96_000);
    s.live.devices.get_mut(&bp_device()).unwrap().bit_perfect = false;
    assert_eq!(rate_cut(&s, p, &next_request(&s, b)), None);
}

#[test]
fn br3_2_the_stream_rate_is_unknown() {
    let (mut s, p, [_, b, _]) = on_air();
    rates(&mut s, 96_000);
    s.live.stream_rates.clear();
    assert_eq!(rate_cut(&s, p, &next_request(&s, b)), None);
}

#[test]
fn br3_3_the_next_format_is_unknown_dsd_zero_or_the_same_rate() {
    let (mut s, p, [_, b, _]) = on_air();
    let track = s.playlists.entry(b).unwrap().track;
    for format in [
        None,
        Some(fp_model::AudioFormat {
            sample_rate: 88_200,
            bits: None,
            channels: 2,
            dsd_rate: Some(2_822_400),
        }),
        Some(fp_model::AudioFormat {
            sample_rate: 0,
            bits: Some(16),
            channels: 2,
            dsd_rate: None,
        }),
        Some(fp_model::AudioFormat {
            sample_rate: 44_100,
            bits: Some(16),
            channels: 2,
            dsd_rate: None,
        }),
    ] {
        s.library.get_mut(track).unwrap().format = format;
        assert_eq!(rate_cut(&s, p, &next_request(&s, b)), None, "{format:?}");
    }
}

#[test]
fn br3_4_a_rate_the_device_refused() {
    let (mut s, p, [_, b, _]) = on_air();
    rates(&mut s, 96_000);
    report_stream_rate(&mut s, 44_100, vec![96_000]);
    assert_eq!(rate_cut(&s, p, &next_request(&s, b)), None);
}

#[test]
fn br3_5_something_else_sounds_on_the_device() {
    // Another player playing there.
    let (mut s, p, [_, b, _]) = on_air();
    rates(&mut s, 96_000);
    let q = s.players[1].id;
    place_on_dac(&mut s, Holder::PlayerMain(q));
    assert_eq!(
        rate_cut(&s, p, &next_request(&s, b)),
        Some(96_000),
        "q is stopped"
    );
    apply(&mut s, Command::Play(q)).unwrap();
    assert_eq!(rate_cut(&s, p, &next_request(&s, b)), None);
    // This player's own CUE placed on the device.
    let (mut s, p, [_, b, c]) = on_air();
    rates(&mut s, 96_000);
    s.live.placement.insert(Holder::PlayerCue(p), bp_device());
    s.players[0].cue = Some(CueState {
        entry: c,
        paused: false,
    });
    assert_eq!(rate_cut(&s, p, &next_request(&s, b)), None);
}

#[test]
fn br3_6_a_dsd_stream_on_air() {
    let (mut s, p, [a, b, _]) = on_air();
    rates(&mut s, 96_000);
    s.players[0].dsd = Some(DsdOnAir {
        entry: a,
        hold_others: false,
    });
    assert_eq!(rate_cut(&s, p, &next_request(&s, b)), None);
}
