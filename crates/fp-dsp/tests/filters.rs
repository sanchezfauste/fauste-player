#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The shared signal processing (loudness normalization spec N6): the
//! K-weighting and the true-peak interpolator of ITU-R BS.1770, which the
//! level meters (meters spec M1) and the analysis use.

use fp_dsp::kweight::{KWeighting, coefficients};
use fp_dsp::truepeak::TruePeak;

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

/// Highest true-peak reading of `samples`, in dBTP.
fn dbtp(samples: &[f32]) -> f32 {
    let mut tp = TruePeak::default();
    let peak = samples.iter().fold(0.0f32, |m, x| m.max(tp.push(*x)));
    20.0 * peak.log10()
}

/// A sine at `fs / div` of amplitude `amp` (FFS) and `phase_deg`, 1 s at
/// 48 kHz, with 10 ms fades (EBU Tech 3341 cases 15–19).
fn tp_sine(div: f64, amp: f64, phase_deg: f64) -> Vec<f32> {
    let n = 48_000;
    let fade = 480.0;
    (0..n)
        .map(|i| {
            let i = i as f64;
            let env = (i / fade).min((n as f64 - 1.0 - i) / fade).min(1.0);
            let x = amp * (std::f64::consts::TAU * i / div + phase_deg.to_radians()).sin();
            (x * env) as f32
        })
        .collect()
}

fn within(v: f32, expected: f32) -> bool {
    // Tech 3341 tolerance: +0.2 / −0.4 dB.
    v <= expected + 0.2 && v >= expected - 0.4
}

#[test]
fn true_peak_meets_ebu_tech_3341_cases_15_to_19() {
    for (case, div, amp, phase, expected) in [
        (15, 4.0, 0.5, 0.0, -6.0),
        (16, 4.0, 0.5, 45.0, -6.0),
        (17, 6.0, 0.5, 60.0, -6.0),
        (18, 8.0, 0.5, 67.5, -6.0),
        (19, 4.0, 1.41, 45.0, 3.0),
    ] {
        let v = dbtp(&tp_sine(div, amp, phase));
        assert!(
            within(v, expected),
            "case {case}: {v} dBTP, expected {expected}"
        );
    }
}

/// Tech 3341 cases 20–23: fs/6 at 0.5 FFS with one period of fs/4 at 1.0
/// inserted (continuous in phase), synthesized at 4·fs, low-pass filtered
/// and decimated with an offset of 0–3 samples.
fn tp_burst(offset: usize) -> Vec<f32> {
    let up = 4usize;
    let n = 48_000 * up;
    let rate = 192_000.0;
    let mut phase = 0.0f64;
    let burst_start = n / 2;
    let burst_len = 16; // one period of fs/4 at 4·fs
    let fade = 0.01 * rate;
    let hi: Vec<f64> = (0..n)
        .map(|i| {
            let in_burst = (burst_start..burst_start + burst_len).contains(&i);
            let (freq, amp) = if in_burst {
                (12_000.0, 1.0)
            } else {
                (8_000.0, 0.5)
            };
            let x = amp * phase.sin();
            phase += std::f64::consts::TAU * freq / rate;
            let i = i as f64;
            x * (i / fade).min((n as f64 - 1.0 - i) / fade).min(1.0)
        })
        .collect();
    // Anti-aliasing low-pass at fs/2: 511-tap Blackman-windowed sinc.
    let taps = 511usize;
    let centre = (taps - 1) as f64 / 2.0;
    let h: Vec<f64> = (0..taps)
        .map(|k| {
            let t = (k as f64 - centre) / up as f64;
            let sinc = if t == 0.0 {
                1.0
            } else {
                (std::f64::consts::PI * t).sin() / (std::f64::consts::PI * t)
            };
            let x = std::f64::consts::TAU * k as f64 / (taps - 1) as f64;
            sinc * (0.42 - 0.5 * x.cos() + 0.08 * (2.0 * x).cos()) / up as f64
        })
        .collect();
    (0..(n - taps - up) / up)
        .map(|m| {
            let centre_in = m * up + offset + taps / 2;
            let acc: f64 = h
                .iter()
                .enumerate()
                .map(|(k, c)| c * hi[centre_in + k - taps / 2])
                .sum();
            acc as f32
        })
        .collect()
}

#[test]
fn true_peak_meets_ebu_tech_3341_cases_20_to_23() {
    for offset in 0..4 {
        let v = dbtp(&tp_burst(offset));
        assert!(
            within(v, 0.0),
            "case {}: {v} dBTP, expected 0.0",
            20 + offset
        );
    }
}

#[test]
fn silence_brings_the_k_weighting_to_exact_zero() {
    let mut k = KWeighting::new(48_000);
    for n in 0..48_000 {
        k.process((std::f64::consts::TAU * 1_000.0 * f64::from(n) / 48_000.0).sin());
    }
    let mut last = 1.0;
    for _ in 0..48_000 {
        last = k.process(0.0);
    }
    assert_eq!(last, 0.0, "no subnormal limit cycle");
}

#[test]
#[allow(clippy::excessive_precision)] // the published values, exact in f32
fn the_true_peak_filter_is_the_one_bs_1770_publishes() {
    // ITU-R BS.1770-5, Annex 2: order 48, 4 phases of 12 coefficients.
    let phases = fp_dsp::truepeak::PHASES;
    assert_eq!(phases[0][0], 0.001_708_984_375);
    assert_eq!(phases[0][5], 0.137_329_101_562_5);
    assert_eq!(phases[0][6], 0.972_167_968_75);
    assert_eq!(phases[1][5], 0.465_087_890_625);
    assert_eq!(phases[2][5], 0.779_785_156_25);
    assert_eq!(phases[3][5], 0.972_167_968_75);
    assert_eq!(phases[3][11], 0.001_708_984_375);
    // Each phase mirrors another.
    for p in 0..4 {
        for k in 0..12 {
            assert_eq!(phases[p][k], phases[3 - p][11 - k]);
        }
    }
}
