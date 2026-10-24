#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The volume fader's law: a mixing-desk fader, fine near 0 dB.

use fp_model::volume::{db_to_travel, fader_from_gain, gain_from_fader, travel_to_db};

fn db_at(pos: f32) -> f32 {
    20.0 * gain_from_fader(pos).log10()
}

#[test]
fn reference_points_of_the_curve() {
    for (pos, db) in [
        (1.0, 0.0),
        (0.9, -2.5),
        (0.8, -5.3),
        (0.5, -15.9),
        (0.3, -26.8),
        (0.1, -48.0),
    ] {
        assert!((db_at(pos) - db).abs() < 0.06, "{pos}: {}", db_at(pos));
    }
}

#[test]
fn the_ends_are_zero_db_and_silence() {
    assert_eq!(gain_from_fader(1.0), 1.0);
    assert_eq!(gain_from_fader(2.0), 1.0);
    assert_eq!(gain_from_fader(0.0), 0.0);
    assert_eq!(gain_from_fader(-0.5), 0.0);
    assert_eq!(gain_from_fader(f32::NAN), 0.0);
    assert_eq!(fader_from_gain(1.0), 1.0);
    assert_eq!(fader_from_gain(3.0), 1.0);
    assert_eq!(fader_from_gain(0.0), 0.0);
    assert_eq!(fader_from_gain(-1.0), 0.0);
    assert_eq!(fader_from_gain(f32::NAN), 0.0);
}

#[test]
fn travel_and_gain_round_trip() {
    for n in 1..=100 {
        let pos = n as f32 / 100.0;
        let back = fader_from_gain(gain_from_fader(pos));
        assert!((back - pos).abs() < 2e-4, "{pos} -> {back}");
    }
}

#[test]
fn db_and_travel_are_inverse() {
    assert!((travel_to_db(1.0) - 0.0).abs() < 1e-6);
    assert!((travel_to_db(0.5) + 15.9).abs() < 0.06);
    assert!((db_to_travel(0.0) - 1.0).abs() < 1e-6);
    assert!((db_to_travel(-15.9) - 0.5).abs() < 0.005);
    assert_eq!(db_to_travel(f32::NEG_INFINITY), 0.0);
    assert_eq!(db_to_travel(-200.0), 0.0);
}
