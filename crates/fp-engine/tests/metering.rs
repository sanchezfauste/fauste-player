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

use fp_engine::meter::{MeterInput, MeterReading, MeterState};
use fp_model::{LoudnessReadout, MeterBallistics, MeterConfig};

const TICK: f64 = 0.005;

fn config(ballistics: MeterBallistics) -> MeterConfig {
    MeterConfig {
        ballistics,
        peak_hold_secs: 0.0,
        ..MeterConfig::default()
    }
}

fn db(linear: f32) -> f32 {
    20.0 * linear.log10()
}

/// A tick of steady peak `level` (linear), both channels.
fn peak(level: f32) -> MeterInput {
    MeterInput {
        peak: [level; 2],
        sum_sq: [f64::from(level * level) * 240.0 / 2.0; 2],
        k_sum: [0.0; 2],
        frames: 240,
    }
}

fn run(m: &mut MeterState, c: &MeterConfig, input: MeterInput, secs: f64) -> MeterReading {
    let mut r = MeterReading::default();
    for _ in 0..(secs / TICK).round() as usize {
        r = m.update(input, TICK, c);
    }
    r
}

#[test]
fn digital_peak_rises_at_once_and_falls_20_db_in_1_7_s() {
    let c = config(MeterBallistics::DigitalPeak);
    let mut m = MeterState::default();
    let r = m.update(peak(1.0), TICK, &c);
    assert!(
        r.level_db[0].abs() < 0.01,
        "instant rise: {}",
        r.level_db[0]
    );
    let r = run(&mut m, &c, peak(0.0), 1.7);
    assert!((r.level_db[0] + 20.0).abs() < 0.3, "{}", r.level_db[0]);
}

#[test]
fn ebu_ppm_falls_24_db_in_2_8_s() {
    let c = config(MeterBallistics::EbuPpm);
    let mut m = MeterState::default();
    run(&mut m, &c, peak(1.0), 0.5);
    let r = run(&mut m, &c, peak(0.0), 2.8);
    assert!((r.level_db[0] + 24.0).abs() < 0.3, "{}", r.level_db[0]);
}

#[test]
fn a_short_burst_reads_lower_on_a_ppm() {
    let c = config(MeterBallistics::EbuPpm);
    let mut m = MeterState::default();
    let burst = m.update(peak(1.0), TICK, &c);
    assert!(
        burst.level_db[0] < -0.5,
        "5 ms of a 10 ms integration: {}",
        burst.level_db[0]
    );
    let steady = run(&mut m, &c, peak(1.0), 0.2);
    assert!(steady.level_db[0].abs() < 0.1);
}

#[test]
fn vu_reaches_99_percent_in_300_ms() {
    let c = config(MeterBallistics::Vu);
    let mut m = MeterState::default();
    let early = run(&mut m, &c, peak(1.0), 0.1);
    let at_300 = run(&mut m, &c, peak(1.0), 0.2);
    // A steady sine of peak 1 reads 0 dB (RMS, sine-calibrated).
    assert!(
        early.level_db[0] < -1.0,
        "not instant: {}",
        early.level_db[0]
    );
    assert!(
        at_300.level_db[0] > db(0.99) - 0.05,
        "{}",
        at_300.level_db[0]
    );
    assert!(at_300.level_db[0] < 0.1);
}

#[test]
fn the_hold_stays_then_falls() {
    let c = MeterConfig {
        peak_hold_secs: 2.0,
        ..MeterConfig::default()
    };
    let mut m = MeterState::default();
    m.update(peak(1.0), TICK, &c);
    let held = run(&mut m, &c, peak(0.0), 1.9);
    assert!(held.hold_db[0].abs() < 0.01, "held: {}", held.hold_db[0]);
    assert!(held.level_db[0] < -15.0, "the bar itself falls");
    let fallen = run(&mut m, &c, peak(0.0), 1.0);
    assert!(
        fallen.hold_db[0] < -5.0,
        "then falls: {}",
        fallen.hold_db[0]
    );
}

#[test]
fn custom_ballistics_use_the_configured_rates() {
    let c = MeterConfig {
        ballistics: MeterBallistics::Custom,
        attack_ms: 0.0,
        release_db_per_sec: 40.0,
        peak_hold_secs: 0.0,
        ..MeterConfig::default()
    };
    let mut m = MeterState::default();
    m.update(peak(1.0), TICK, &c);
    let r = run(&mut m, &c, peak(0.0), 0.5);
    assert!((r.level_db[0] + 20.0).abs() < 0.3, "{}", r.level_db[0]);
}

/// Ticks of a stereo sine of `freq` at `dbfs`, measured like the mixer does.
fn sine_ticks(dbfs: f64, freq: f64, secs: f64) -> Vec<MeterInput> {
    let rate = 48_000u32;
    let amp = 10f64.powf(dbfs / 20.0);
    let mut k = [KWeighting::new(rate), KWeighting::new(rate)];
    let frames = 240usize;
    let mut out = Vec::new();
    let mut n = 0usize;
    for _ in 0..(secs / TICK).round() as usize {
        let mut input = MeterInput {
            frames: frames as u64,
            ..MeterInput::default()
        };
        for _ in 0..frames {
            let x = amp * (std::f64::consts::TAU * freq * n as f64 / f64::from(rate)).sin();
            n += 1;
            for (ch, kw) in k.iter_mut().enumerate() {
                let w = kw.process(x);
                input.k_sum[ch] += w * w;
                input.sum_sq[ch] += x * x;
                input.peak[ch] = input.peak[ch].max(x.abs() as f32);
            }
        }
        out.push(input);
    }
    out
}

#[test]
fn a_minus_23_dbfs_1_khz_stereo_sine_reads_minus_23_lufs() {
    // EBU Tech 3341, test case 1.
    let c = MeterConfig::default();
    let mut m = MeterState::default();
    let mut r = MeterReading::default();
    for input in sine_ticks(-23.0, 1_000.0, 4.0) {
        r = m.update(input, TICK, &c);
    }
    let momentary = r.momentary_lufs.unwrap();
    let short_term = r.short_term_lufs.unwrap();
    assert!((momentary + 23.0).abs() < 0.1, "M {momentary}");
    assert!((short_term + 23.0).abs() < 0.1, "S {short_term}");
    assert_eq!(c.loudness, LoudnessReadout::ShortTerm);
}

#[test]
fn silence_clears_the_loudness() {
    let c = MeterConfig::default();
    let mut m = MeterState::default();
    for input in sine_ticks(-23.0, 1_000.0, 1.0) {
        m.update(input, TICK, &c);
    }
    let r = run(&mut m, &c, MeterInput::default(), 3.2);
    assert_eq!(r.momentary_lufs, None);
    assert_eq!(r.short_term_lufs, None);
}
