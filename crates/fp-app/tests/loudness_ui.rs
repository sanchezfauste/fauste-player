#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Loudness normalization in the interface (loudness normalization spec
//! §8): the gain badge (N18), the tooltip lines (N19), the Loudness… popup
//! (N20), the cart editor (N21) and the Settings tab (N22).

mod support;

use egui_kittest::kittest::Queryable;
use fp_model::{AppState, Command, LoudnessMeasurement, apply};
use support::{harness, state};

/// One player on air with −20 LUFS tracks: +4.0 dB at the default target.
fn on_air() -> AppState {
    let mut s = state(1, 3);
    s.config.loudness.normalize = true;
    for t in s.library.iter_mut() {
        t.analyzed = true;
        t.duration_secs = 180.0;
        t.loudness = Some(LoudnessMeasurement {
            integrated_lufs: Some(-20.0),
            true_peak_dbtp: -10.0,
        });
    }
    let p = s.players[0].id;
    apply(&mut s, Command::Play(p)).unwrap();
    s
}

/// Places player 1's Main on a bit-perfect device.
fn bit_perfect(s: &mut AppState) {
    let p = s.players[0].id;
    let dac = fp_model::OutputDevice {
        backend: "null".into(),
        device: "dac".into(),
    };
    s.config.outputs.bit_perfect = vec![dac.clone()];
    let running = fp_model::device_settings(&s.config.outputs, &dac);
    fp_model::on_event(
        s,
        fp_model::EngineEvent::Placed {
            holder: fp_model::Holder::PlayerMain(p),
            route: None,
            device: dac,
            running,
        },
    );
}

#[test]
fn n18_the_header_shows_the_gain_and_says_why() {
    let (mut h, _fake) = harness(on_air());
    h.run_steps(2);
    // The tile's accessible name is its tooltip, as for the BP badge.
    assert!(
        h.query_by_label("Normalized to −16.0 LUFS: measured −20.0 LUFS, trim 0.0 dB.")
            .is_some()
    );
    assert!(
        h.query_by_label("+4.0 dB").is_none(),
        "named by its tooltip"
    );
}

#[test]
fn n18_a_track_not_measured_says_so() {
    let mut s = on_air();
    for t in s.library.iter_mut() {
        t.loudness = None;
    }
    // Start again so that the source on air is the unmeasured one.
    let p = s.players[0].id;
    apply(&mut s, Command::Stop(p)).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    let (mut h, _fake) = harness(s);
    h.run_steps(2);
    assert!(
        h.query_by_label("Loudness not measured yet: this track plays unchanged.")
            .is_some()
    );
}

#[test]
fn n18_no_badge_when_stopped_or_off() {
    let mut s = on_air();
    let p = s.players[0].id;
    apply(&mut s, Command::Stop(p)).unwrap();
    let (mut h, _fake) = harness(s);
    h.run_steps(2);
    assert!(h.query_by_label_contains("Normalized to").is_none());

    let mut s = on_air();
    s.config.loudness.normalize = false;
    apply(&mut s, Command::Stop(p)).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    let (mut h, _fake) = harness(s);
    h.run_steps(2);
    assert!(h.query_by_label_contains("Normalized to").is_none());
    assert!(h.query_by_label_contains("not measured").is_none());
}

#[test]
fn n18_a_bit_perfect_player_plays_at_unity_gain_and_says_so() {
    let mut s = on_air();
    let p = s.players[0].id;
    apply(&mut s, Command::Stop(p)).unwrap();
    bit_perfect(&mut s);
    apply(&mut s, Command::Play(p)).unwrap();
    let (mut h, _fake) = harness(s);
    h.run_steps(2);
    assert!(
        h.query_by_label("Bit-perfect output: tracks play unaltered.")
            .is_some()
    );
    assert!(h.query_by_label_contains("Normalized to").is_none());
}

#[test]
fn n18_a_paused_player_keeps_its_badge_and_gain() {
    let mut s = on_air();
    let p = s.players[0].id;
    apply(&mut s, Command::Pause(p)).unwrap();
    let mut config = s.config.clone();
    config.loudness.target_lufs = -10.0;
    apply(&mut s, Command::UpdateConfig(Box::new(config))).unwrap();
    let (mut h, _fake) = harness(s);
    h.run_steps(2);
    // The source on air keeps +4.0 dB; the tooltip says that it started
    // with earlier settings, which the next track will not use.
    assert!(
        h.query_by_label_contains("Normalized to −10.0 LUFS")
            .is_some()
    );
    assert!(
        h.query_by_label_contains("started before the last settings change")
            .is_some()
    );
}

#[test]
fn n19_the_row_tooltip_shows_the_loudness() {
    let (mut h, _) = harness(on_air());
    h.get_by_label("Song 3").hover();
    h.run_steps(40);
    assert!(
        h.query_by_label("Loudness: −20.0 LUFS, true peak −10.0 dBTP")
            .is_some()
    );
    assert!(h.query_by_label("Excluded from normalization").is_none());
}

#[test]
fn n19_the_row_tooltip_shows_the_edit() {
    let mut s = on_air();
    let playlist = s.playlists.first_id().unwrap();
    let track = s.playlists.get(playlist).unwrap().entries[2].track;
    apply(
        &mut s,
        Command::SetTrackLoudness {
            track,
            excluded: true,
            trim_db: 0.0,
        },
    )
    .unwrap();
    let (mut h, _) = harness(s);
    h.get_by_label("Song 3").hover();
    h.run_steps(40);
    assert!(h.query_by_label("Excluded from normalization").is_some());
}
