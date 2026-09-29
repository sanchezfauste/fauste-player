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

/// A tick of a steady sine of peak `level` (linear), both channels.
fn peak(level: f32) -> MeterInput {
    MeterInput {
        peak: [level; 2],
        sum_sq: [f64::from(level * level) * 240.0 / 2.0; 2],
        // A sine's rectified average is 2/π of its peak.
        sum_abs: [f64::from(level) * 240.0 * std::f64::consts::FRAC_2_PI; 2],
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

/// The VU's step response to a sine of peak 1, per tick, as linear level.
fn vu_step(secs: f64) -> Vec<f32> {
    let c = config(MeterBallistics::Vu);
    let mut m = MeterState::default();
    (0..(secs / TICK).round() as usize)
        .map(|_| 10f32.powf(m.update(peak(1.0), TICK, &c).level_db[0] / 20.0))
        .collect()
}

#[test]
fn vu_reaches_99_percent_in_300_ms() {
    // IEC 60268-17: 99 % of the steady reading in 300 ms ± 10 %.
    let steps = vu_step(2.0);
    let first = steps.iter().position(|v| *v >= 0.99).unwrap();
    let t = (first + 1) as f64 * TICK;
    assert!((0.27..=0.33).contains(&t), "99 % at {t} s");
}

#[test]
fn vu_overshoots_between_1_and_1_5_percent() {
    // IEC 60268-17: the needle overshoots by 1 % to 1.5 %.
    let steps = vu_step(2.0);
    let max = steps.iter().copied().fold(0.0f32, f32::max);
    assert!((1.010..=1.015).contains(&max), "overshoot {max}");
    let settled = *steps.last().unwrap();
    assert!(
        (settled - 1.0).abs() < 0.001,
        "a sine reads its peak level: {settled}"
    );
}

#[test]
fn vu_reads_the_rectified_average_like_a_real_vu() {
    // A square wave of the same peak has a rectified average π/2 times a
    // sine's: it reads 3.92 dB higher, where an RMS meter shows 3.01.
    let c = config(MeterBallistics::Vu);
    let mut m = MeterState::default();
    let square = MeterInput {
        peak: [1.0; 2],
        sum_sq: [240.0; 2],
        sum_abs: [240.0; 2],
        frames: 240,
        ..MeterInput::default()
    };
    let r = run(&mut m, &c, square, 2.0);
    assert!((r.level_db[0] - 3.92).abs() < 0.05, "{}", r.level_db[0]);
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

#[test]
fn din_ppm_falls_20_db_in_1_5_s() {
    let c = config(MeterBallistics::DinPpm);
    let mut m = MeterState::default();
    run(&mut m, &c, peak(1.0), 0.5);
    let r = run(&mut m, &c, peak(0.0), 1.5);
    assert!((r.level_db[0] + 20.0).abs() < 0.3, "{}", r.level_db[0]);
}

#[test]
fn vu_is_steady_when_callbacks_skip_ticks() {
    // A 512-frame device buffer at 48 kHz fills every 10.7 ms: about every
    // other 5 ms tick carries nothing.
    let c = config(MeterBallistics::Vu);
    let mut m = MeterState::default();
    let full = MeterInput {
        frames: 512,
        sum_sq: [0.5 * 512.0; 2],
        sum_abs: [512.0 * std::f64::consts::FRAC_2_PI; 2],
        peak: [1.0; 2],
        ..MeterInput::default()
    };
    let mut r = MeterReading::default();
    for i in 0..400 {
        let input = if i % 2 == 0 {
            full
        } else {
            MeterInput::default()
        };
        r = m.update(input, TICK, &c);
    }
    assert!(
        r.level_db[0].abs() < 0.3,
        "a sine of peak 1 reads 0 dB: {}",
        r.level_db[0]
    );
}

#[test]
fn the_bar_still_falls_when_the_audio_stops() {
    let c = config(MeterBallistics::Vu);
    let mut m = MeterState::default();
    run(&mut m, &c, peak(1.0), 1.0);
    let r = run(&mut m, &c, MeterInput::default(), 1.0);
    assert!(r.level_db[0] < -20.0, "{}", r.level_db[0]);
}

#[test]
fn loudness_falls_after_a_stop_instead_of_freezing() {
    let c = MeterConfig::default();
    let mut m = MeterState::default();
    let mut before = MeterReading::default();
    for input in sine_ticks(-23.0, 1_000.0, 4.0) {
        before = m.update(input, TICK, &c);
    }
    let after = run(&mut m, &c, MeterInput::default(), 1.0);
    let (b, a) = (
        before.short_term_lufs.unwrap(),
        after.short_term_lufs.unwrap(),
    );
    assert!(a < b - 1.0, "short-term falls: {b} → {a}");
    assert_eq!(
        after.momentary_lufs, None,
        "400 ms of silence clears momentary"
    );
}

#[test]
fn a_long_stall_does_not_shrink_the_loudness_window() {
    let c = MeterConfig::default();
    let mut m = MeterState::default();
    for input in sine_ticks(-23.0, 1_000.0, 1.0) {
        m.update(input, TICK, &c);
    }
    // The conductor was held up for 5 s.
    m.update(MeterInput::default(), 5.0, &c);
    let mut r = MeterReading::default();
    for input in sine_ticks(-23.0, 1_000.0, 0.2) {
        r = m.update(input, TICK, &c);
    }
    let s = r.short_term_lufs;
    assert!(s.is_none_or(|s| s < -30.0), "3 s of mostly silence: {s:?}");
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
    let phases = fp_engine::truepeak::PHASES;
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

/// A programme of 1 kHz stereo tones (`Some(dBFS)`) and silences (`None`),
/// each lasting its seconds, measured like the mixer does in 5 ms ticks.
fn programme(parts: &[(Option<f64>, f64)]) -> Vec<MeterInput> {
    let rate = 48_000.0;
    let mut samples = Vec::new();
    let mut n = 0u64;
    for (level, secs) in parts {
        let amp = level.map_or(0.0, |db| 10f64.powf(db / 20.0));
        for _ in 0..(secs * rate).round() as usize {
            samples.push(amp * (std::f64::consts::TAU * 1_000.0 * n as f64 / rate).sin());
            n += 1;
        }
    }
    let mut k = [KWeighting::new(48_000), KWeighting::new(48_000)];
    samples
        .chunks(240)
        .map(|chunk| {
            let mut input = MeterInput {
                frames: chunk.len() as u64,
                ..MeterInput::default()
            };
            for x in chunk {
                for (ch, kw) in k.iter_mut().enumerate() {
                    let w = kw.process(*x);
                    input.k_sum[ch] += w * w;
                }
            }
            input
        })
        .collect()
}

fn readings(inputs: &[MeterInput]) -> Vec<MeterReading> {
    let c = MeterConfig::default();
    let mut m = MeterState::default();
    inputs.iter().map(|i| m.update(*i, TICK, &c)).collect()
}

#[test]
fn ebu_tech_3341_case_2_reads_minus_33_lufs() {
    let r = readings(&programme(&[(Some(-33.0), 5.0)]));
    let last = r.last().unwrap();
    assert!((last.momentary_lufs.unwrap() + 33.0).abs() <= 0.1);
    assert!((last.short_term_lufs.unwrap() + 33.0).abs() <= 0.1);
}

#[test]
fn ebu_tech_3341_case_9_short_term_is_constant_after_3_s() {
    let parts: Vec<_> = (0..5)
        .flat_map(|_| [(Some(-20.0), 1.34), (Some(-30.0), 1.66)])
        .collect();
    let r = readings(&programme(&parts));
    for (i, reading) in r.iter().enumerate().skip((3.0 / TICK) as usize) {
        let s = reading.short_term_lufs.unwrap();
        assert!(
            (s + 23.0).abs() <= 0.1,
            "at {:.3} s: S = {s}",
            i as f64 * TICK
        );
    }
}

#[test]
fn ebu_tech_3341_case_12_momentary_is_constant_after_1_s() {
    let parts: Vec<_> = (0..25)
        .flat_map(|_| [(Some(-20.0), 0.18), (Some(-30.0), 0.22)])
        .collect();
    let r = readings(&programme(&parts));
    for (i, reading) in r.iter().enumerate().skip((1.0 / TICK) as usize) {
        let m = reading.momentary_lufs.unwrap();
        assert!(
            (m + 23.0).abs() <= 0.1,
            "at {:.3} s: M = {m}",
            i as f64 * TICK
        );
    }
}

/// Tech 3341's live-meter cases: 20 segments of `lead · i` silence, a tone
/// of `tone` seconds at −38 + i dBFS, then `tone − lead · i` of silence.
/// The maximum of `pick` in each segment must step from −38 to −19 LUFS.
fn live_case(lead: f64, tone: f64, pick: fn(&MeterReading) -> Option<f32>) {
    let parts: Vec<_> = (0..20)
        .flat_map(|i| {
            let i = f64::from(i);
            [
                (None, lead * i),
                (Some(-38.0 + i), tone),
                (None, tone - lead * i),
            ]
        })
        .collect();
    let r = readings(&programme(&parts));
    let mut start = 0.0;
    for i in 0..20 {
        let span = 2.0 * tone;
        let from = (start / TICK) as usize;
        let to = (((start + span) / TICK) as usize).min(r.len());
        let max = r[from..to]
            .iter()
            .filter_map(pick)
            .fold(f32::NEG_INFINITY, f32::max);
        let expected = -38.0 + i as f32;
        assert!(
            (max - expected).abs() <= 0.1,
            "segment {i}: max {max}, expected {expected}"
        );
        start += span;
    }
}

#[test]
fn ebu_tech_3341_case_11_short_term_maxima_step_by_1_lu() {
    live_case(0.15, 3.0, |r| r.short_term_lufs);
}

#[test]
fn ebu_tech_3341_case_14_momentary_maxima_step_by_1_lu() {
    live_case(0.02, 0.4, |r| r.momentary_lufs);
}

#[test]
fn a_long_conductor_stall_costs_the_vu_no_more_than_a_second() {
    let c = config(MeterBallistics::Vu);
    let mut m = MeterState::default();
    run(&mut m, &c, peak(1.0), 0.5);
    let started = std::time::Instant::now();
    m.update(peak(1.0), 3_600.0, &c);
    assert!(
        started.elapsed().as_millis() < 100,
        "{:?}",
        started.elapsed()
    );
}

#[test]
fn switching_back_to_vu_starts_from_rest() {
    let vu = config(MeterBallistics::Vu);
    let digital = config(MeterBallistics::DigitalPeak);
    let mut m = MeterState::default();
    run(&mut m, &vu, peak(1.0), 1.0);
    run(&mut m, &digital, MeterInput::default(), 5.0);
    let r = m.update(MeterInput::default(), TICK, &vu);
    assert!(r.level_db[0] < -60.0, "no stale needle: {}", r.level_db[0]);
}

#[test]
fn a_sine_reads_its_level_on_the_k_system_average() {
    // K-System: peak and average ride on the same scale for a sine (AES17).
    let c = config(MeterBallistics::K20);
    let mut m = MeterState::default();
    let mut r = MeterReading::default();
    for input in sine_ticks(-20.0, 1000.0, 2.0) {
        r = m.update(input, TICK, &c);
    }
    assert!((r.rms_db[0] + 20.0).abs() < 0.1, "{}", r.rms_db[0]);
    assert!((r.rms_db[1] + 20.0).abs() < 0.1, "{}", r.rms_db[1]);
    assert!((r.level_db[0] + 20.0).abs() < 0.1, "{}", r.level_db[0]);
}

#[test]
fn the_k_system_average_integrates_over_600_ms() {
    let c = config(MeterBallistics::K20);
    let mut m = MeterState::default();
    let r = run(&mut m, &c, peak(1.0), 0.3);
    assert!(
        r.rms_db[0] < -0.5,
        "still rising at 300 ms: {}",
        r.rms_db[0]
    );
    let r = run(&mut m, &c, peak(1.0), 0.3);
    assert!(
        (-0.1..=0.0).contains(&r.rms_db[0]),
        "99 % of the RMS value at 600 ms: {}",
        r.rms_db[0]
    );
    run(&mut m, &c, peak(1.0), 2.0);
    let r = run(&mut m, &c, MeterInput::default(), 0.6);
    // The same two stages falling: 1.99 % of the mean square is left.
    assert!((r.rms_db[0] + 17.0).abs() < 0.2, "{}", r.rms_db[0]);
}

#[test]
fn the_k_system_peak_falls_26_db_in_3_s() {
    let c = config(MeterBallistics::K14);
    let mut m = MeterState::default();
    let r = m.update(peak(1.0), TICK, &c);
    assert!(
        r.level_db[0].abs() < 0.01,
        "one-sample rise: {}",
        r.level_db[0]
    );
    let r = run(&mut m, &c, MeterInput::default(), 3.0);
    assert!((r.level_db[0] + 26.0).abs() < 0.3, "{}", r.level_db[0]);
}

#[test]
fn the_k_system_average_falls_in_silence() {
    let c = config(MeterBallistics::K12);
    let mut m = MeterState::default();
    run(&mut m, &c, peak(1.0), 2.0);
    let r = run(&mut m, &c, MeterInput::default(), 2.0);
    assert!(r.rms_db[0] < -40.0, "{}", r.rms_db[0]);
}

#[test]
fn the_maximum_stays_after_the_bar_falls_until_it_is_reset() {
    let c = config(MeterBallistics::DigitalPeak);
    let mut m = MeterState::default();
    assert!(m.update(MeterInput::default(), TICK, &c).max_db <= -100.0);
    m.update(peak(0.5), TICK, &c);
    let r = run(&mut m, &c, peak(0.1), 3.0);
    assert!((r.max_db + 6.02).abs() < 0.05, "{}", r.max_db);
    m.reset_max();
    let r = m.update(peak(0.1), TICK, &c);
    assert!((r.max_db + 20.0).abs() < 0.05, "{}", r.max_db);
}

#[test]
fn a_restarted_maximum_ignores_the_bar_still_falling_from_before() {
    // A loud track cut straight into a quiet one: the new maximum is the
    // quiet track's level, not the old bar on its way down.
    let c = config(MeterBallistics::DigitalPeak);
    let mut m = MeterState::default();
    run(&mut m, &c, peak(0.9), 0.5);
    m.reset_max();
    let r = run(&mut m, &c, peak(0.1), 0.05);
    assert!((r.max_db + 20.0).abs() < 0.05, "{}", r.max_db);
}

#[test]
fn a_non_finite_block_does_not_break_the_meter() {
    let c = MeterConfig::default();
    let mut m = MeterState::default();
    let bad = MeterInput {
        peak: [f32::INFINITY, f32::NAN],
        sum_sq: [f64::NAN, f64::INFINITY],
        sum_abs: [f64::NAN; 2],
        k_sum: [f64::NAN; 2],
        frames: 240,
    };
    let r = m.update(bad, TICK, &c);
    assert!(r.max_db.is_finite(), "{r:?}");
    let c = config(MeterBallistics::K20);
    let r = run(&mut m, &c, peak(0.1), 2.0);
    assert!(r.max_db.is_finite() && r.max_db < 1.0, "{r:?}");
    assert!(
        (r.rms_db[0] + 20.0).abs() < 0.2,
        "the average recovers: {r:?}"
    );
    assert!((r.rms_db[1] + 20.0).abs() < 0.2, "{r:?}");
    assert!((r.level_db[0] + 20.0).abs() < 0.2, "{r:?}");
}
