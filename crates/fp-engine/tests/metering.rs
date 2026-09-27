#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Measurement for the level meters (meters spec M1): K-weighting and
//! true peak.

use fp_engine::kweight::{KWeighting, coefficients};
use fp_engine::truepeak::TruePeak;

#[test]
fn k_weighting_at_48_khz_matches_the_standard_coefficients() {
    // ITU-R BS.1770-4, table 1 (pre-filter) and table 2 (RLB high-pass).
    let (shelf, highpass) = coefficients(48_000);
    let close = |a: f64, b: f64| (a - b).abs() < 1e-8;
    assert!(close(shelf.b0, 1.535_124_859_586_97), "{shelf:?}");
    assert!(close(shelf.b1, -2.691_696_189_406_38));
    assert!(close(shelf.b2, 1.198_392_810_852_85));
    assert!(close(shelf.a1, -1.690_659_293_182_41));
    assert!(close(shelf.a2, 0.732_480_774_215_85));
    assert!(close(highpass.b0, 1.0) && close(highpass.b1, -2.0) && close(highpass.b2, 1.0));
    assert!(close(highpass.a1, -1.990_047_454_833_98), "{highpass:?}");
    assert!(close(highpass.a2, 0.990_072_250_366_21));
}

/// Mean-square gain of the K-weighting for a sine of `freq`, in dB.
fn gain_db(rate: u32, freq: f64) -> f64 {
    let mut k = KWeighting::new(rate);
    let n = rate as usize * 2;
    let mut sum_in = 0.0;
    let mut sum_out = 0.0;
    for i in 0..n {
        let x = (std::f64::consts::TAU * freq * i as f64 / f64::from(rate)).sin() * 0.5;
        let y = k.process(x);
        // Skip the first half second: the filters settle.
        if i > rate as usize / 2 {
            sum_in += x * x;
            sum_out += y * y;
        }
    }
    10.0 * (sum_out / sum_in).log10()
}

#[test]
fn k_weighting_gain_at_1_khz_is_the_same_at_every_rate() {
    let at_48 = gain_db(48_000, 997.0);
    for rate in [44_100, 96_000, 192_000] {
        let g = gain_db(rate, 997.0);
        assert!((g - at_48).abs() < 0.05, "{rate} Hz: {g} dB vs {at_48} dB");
    }
    // The +0.691 dB the loudness formula's −0.691 compensates.
    assert!((at_48 - 0.691).abs() < 0.05, "{at_48}");
}

#[test]
fn true_peak_finds_the_intersample_peak() {
    // fs/4 with a 45° phase: every sample is ±0.707, the waveform peaks at 1.
    let mut tp = TruePeak::default();
    let mut sample_peak = 0.0f32;
    let mut true_peak = 0.0f32;
    for n in 0..4_000 {
        let x = (std::f32::consts::FRAC_PI_2 * n as f32 + std::f32::consts::FRAC_PI_4).sin();
        sample_peak = sample_peak.max(x.abs());
        if n > 100 {
            true_peak = true_peak.max(tp.push(x));
        } else {
            tp.push(x);
        }
    }
    assert!((sample_peak - 0.707).abs() < 0.01);
    assert!(true_peak > 0.97 && true_peak < 1.03, "{true_peak}");
}

#[test]
fn true_peak_of_a_low_tone_equals_its_sample_peak() {
    let mut tp = TruePeak::default();
    let mut peak = 0.0f32;
    for n in 0..48_000 {
        let x = 0.5 * (std::f32::consts::TAU * 100.0 * n as f32 / 48_000.0).sin();
        let p = tp.push(x);
        if n > 100 {
            peak = peak.max(p);
        }
    }
    assert!((peak - 0.5).abs() < 0.005, "{peak}");
}
