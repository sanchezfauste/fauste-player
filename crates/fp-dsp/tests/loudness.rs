#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Integrated loudness of a whole file (loudness normalization spec N4, N5),
//! against the EBU Tech 3341 cases that apply to a file measurement.

use fp_dsp::loudness::{Loudness, LoudnessMeter, SILENCE_DBTP};

/// Stereo 1 kHz sine programme: `(Some(dBFS peak), secs)` or `(None, secs)`
/// for silence, both channels identical, at `rate`.
fn programme(rate: u32, parts: &[(Option<f64>, f64)]) -> Vec<f32> {
    let mut out = Vec::new();
    let mut n = 0u64;
    for (level, secs) in parts {
        let amp = level.map_or(0.0, |db| 10f64.powf(db / 20.0));
        for _ in 0..(secs * f64::from(rate)).round() as usize {
            let x =
                (amp * (std::f64::consts::TAU * 1_000.0 * n as f64 / f64::from(rate)).sin()) as f32;
            out.extend_from_slice(&[x, x]);
            n += 1;
        }
    }
    out
}

/// Measures `samples` pushed in blocks of `block` frames (as a decoder does).
fn measure(rate: u32, samples: &[f32], block: usize) -> Option<Loudness> {
    let mut m = LoudnessMeter::new(rate);
    for chunk in samples.chunks(block * 2) {
        m.push(chunk);
    }
    m.finish()
}

fn lufs(rate: u32, parts: &[(Option<f64>, f64)]) -> f64 {
    measure(rate, &programme(rate, parts), 4_096)
        .unwrap()
        .integrated_lufs
        .unwrap()
}

#[test]
fn ebu_tech_3341_case_1_reads_minus_23_lufs_at_48_and_44_1_khz() {
    for rate in [48_000, 44_100] {
        let v = lufs(rate, &[(Some(-23.0), 20.0)]);
        assert!((v + 23.0).abs() <= 0.1, "{rate} Hz: {v}");
    }
}

#[test]
fn ebu_tech_3341_case_2_reads_minus_33_lufs() {
    let v = lufs(48_000, &[(Some(-33.0), 20.0)]);
    assert!((v + 33.0).abs() <= 0.1, "{v}");
}

#[test]
fn the_relative_gate_drops_the_quiet_parts() {
    // Tech 3341 case 3: −36, −23, −36 dBFS for 10, 60, 10 s.
    let v = lufs(
        48_000,
        &[
            (Some(-36.0), 10.0),
            (Some(-23.0), 60.0),
            (Some(-36.0), 10.0),
        ],
    );
    assert!((v + 23.0).abs() <= 0.1, "{v}");
}

#[test]
fn silent_parts_do_not_lower_the_integrated_loudness() {
    // Tech 3341 case 4: −72, −36, −23, −36, −72 dBFS for 10, 10, 60, 10, 10 s.
    let v = lufs(
        48_000,
        &[
            (Some(-72.0), 10.0),
            (Some(-36.0), 10.0),
            (Some(-23.0), 60.0),
            (Some(-36.0), 10.0),
            (Some(-72.0), 10.0),
        ],
    );
    assert!((v + 23.0).abs() <= 0.1, "{v}");
}

#[test]
fn loud_and_quiet_parts_average_by_energy() {
    // Tech 3341 case 5: −26, −20, −26 dBFS for 20, 20.1, 20 s.
    let v = lufs(
        48_000,
        &[
            (Some(-26.0), 20.0),
            (Some(-20.0), 20.1),
            (Some(-26.0), 20.0),
        ],
    );
    assert!((v + 23.0).abs() <= 0.1, "{v}");
}

#[test]
fn a_sine_reads_its_level_at_every_rate() {
    for rate in [8_000, 11_025, 22_050, 32_000, 88_200, 96_000] {
        let v = lufs(rate, &[(Some(-23.0), 5.0)]);
        // 1 kHz is well inside every band; the K-weighting is derived per rate.
        assert!((v + 23.0).abs() <= 0.15, "{rate} Hz: {v}");
    }
}

#[test]
fn block_size_does_not_change_the_result() {
    let p = programme(48_000, &[(Some(-30.0), 3.0), (Some(-18.0), 3.0)]);
    let a = measure(48_000, &p, 1).unwrap();
    let b = measure(48_000, &p, 4_800).unwrap();
    let c = measure(48_000, &p, 100_000).unwrap();
    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn the_true_peak_is_measured_on_both_channels() {
    // Tech 3341 case 19: fs/4 at 1.41 FFS, 45°, reads +3.0 dBTP (+0.2/−0.4).
    let rate = 48_000u32;
    let mut s = Vec::new();
    for i in 0..rate as usize {
        let x = (1.41 * (std::f64::consts::TAU * i as f64 / 4.0 + 45f64.to_radians()).sin()) as f32;
        // Left silent, right carries the signal: the peak is per channel.
        s.extend_from_slice(&[0.0, x]);
    }
    let tp = measure(rate, &s, 1_024).unwrap().true_peak_dbtp;
    assert!((2.6..=3.2).contains(&tp), "{tp}");
}

#[test]
fn silence_has_no_integrated_loudness_and_a_finite_peak() {
    let l = measure(48_000, &vec![0.0; 48_000 * 2 * 2], 1_024).unwrap();
    assert_eq!(l.integrated_lufs, None);
    assert_eq!(l.true_peak_dbtp, SILENCE_DBTP);
}

#[test]
fn near_silence_under_the_absolute_gate_has_no_integrated_loudness() {
    let l = measure(48_000, &programme(48_000, &[(Some(-75.0), 3.0)]), 1_024).unwrap();
    assert_eq!(l.integrated_lufs, None);
    assert!(l.true_peak_dbtp.is_finite() && l.true_peak_dbtp < -70.0);
}

#[test]
fn a_file_shorter_than_one_gating_block_is_not_measured() {
    // 200 ms (spec N5, Q7) and one frame short of 400 ms.
    assert_eq!(
        measure(48_000, &programme(48_000, &[(Some(-20.0), 0.2)]), 512),
        None
    );
    let almost = programme(48_000, &[(Some(-20.0), 0.4)]);
    assert_eq!(measure(48_000, &almost[..almost.len() - 2], 512), None);
    // Exactly 400 ms is one block: measured.
    assert!(measure(48_000, &almost, 512).is_some());
}

#[test]
fn non_finite_samples_count_as_silence() {
    let mut p = programme(48_000, &[(Some(-23.0), 5.0)]);
    for s in p.iter_mut().step_by(97) {
        *s = f32::NAN;
    }
    p[1_000] = f32::INFINITY;
    p[1_001] = f32::NEG_INFINITY;
    let l = measure(48_000, &p, 1_024).unwrap();
    assert!(l.integrated_lufs.unwrap().is_finite());
    assert!(l.true_peak_dbtp.is_finite());
}

#[test]
fn a_trailing_half_frame_is_ignored() {
    let mut m = LoudnessMeter::new(48_000);
    let p = programme(48_000, &[(Some(-23.0), 1.0)]);
    m.push(&p[..p.len() - 1]);
    m.push(&[0.5]);
    assert!(m.finish().is_some());
}

#[test]
fn a_zero_rate_is_treated_as_one_hertz_without_panicking() {
    let mut m = LoudnessMeter::new(0);
    m.push(&[0.1, 0.1, 0.2, 0.2, 0.3, 0.3, 0.4, 0.4]);
    // Four frames at 1 Hz are four hops of one frame: one block.
    let _ = m.finish();
}
