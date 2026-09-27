//! DSD to PCM (audio formats spec F3): a linear-phase low-pass FIR at the
//! DSD rate, applied to the ±1 bit stream and decimated by 32. It is
//! evaluated through one 256-entry table per byte of the window, so the cost
//! per output sample is one lookup per byte whatever the filter length.

use std::f64::consts::PI;
use std::sync::OnceLock;

/// DSD samples per PCM sample.
pub(crate) const DECIMATION: u64 = 32;
/// DSD bytes per PCM sample.
pub(crate) const BYTES_PER_FRAME: u64 = DECIMATION / 8;
/// Filter length in DSD samples (a whole number of bytes).
const TAPS: usize = 800;
/// Bytes in the window of one output sample.
pub(crate) const WINDOW_BYTES: u64 = (TAPS / 8) as u64;
/// The pass band ends at 20 kHz of DSD64's 88.2 kHz output, as a fraction of
/// the output rate (higher DSD rates get a wider pass band).
const PASS_EDGE: f64 = 20_000.0 / 88_200.0;
/// The stop band starts at the output's Nyquist frequency.
const STOP_EDGE: f64 = 0.5;
/// Kaiser β for about 105 dB of stop-band attenuation.
const KAISER_BETA: f64 = 10.61;
/// The DSD idle pattern: as many ones as zeros, and no energy below a
/// quarter of the DSD rate, so it reads as silence through the filter.
pub(crate) const IDLE: u8 = 0x69;

/// The filter taps, with unity gain at DC.
pub(crate) fn taps() -> Vec<f64> {
    let cutoff = (PASS_EDGE + STOP_EDGE) / 2.0 / DECIMATION as f64;
    let mid = (TAPS - 1) as f64 / 2.0;
    let mut h: Vec<f64> = (0..TAPS)
        .map(|n| {
            let t = n as f64 - mid;
            let sinc = if t == 0.0 {
                2.0 * cutoff
            } else {
                (2.0 * PI * cutoff * t).sin() / (PI * t)
            };
            let r = t / mid;
            sinc * bessel_i0(KAISER_BETA * (1.0 - r * r).max(0.0).sqrt()) / bessel_i0(KAISER_BETA)
        })
        .collect();
    let sum: f64 = h.iter().sum();
    for tap in &mut h {
        *tap /= sum;
    }
    h
}

fn bessel_i0(x: f64) -> f64 {
    let (mut sum, mut term) = (1.0, 1.0);
    for k in 1..64 {
        term *= (x / (2.0 * f64::from(k))).powi(2);
        sum += term;
        if term < sum * 1e-17 {
            break;
        }
    }
    sum
}

/// For each byte of the window, what that byte adds to the output: the sum
/// of its eight taps, each times +1 or -1 for its bit (MSB first in time).
fn tables() -> &'static [[f32; 256]] {
    static TABLES: OnceLock<Vec<[f32; 256]>> = OnceLock::new();
    TABLES.get_or_init(|| {
        taps()
            .chunks(8)
            .map(|taps| {
                let mut table = [0f32; 256];
                for (byte, entry) in table.iter_mut().enumerate() {
                    let sum: f64 = taps
                        .iter()
                        .enumerate()
                        .map(|(bit, tap)| {
                            if byte & (0x80 >> bit) != 0 {
                                *tap
                            } else {
                                -*tap
                            }
                        })
                        .sum();
                    *entry = sum as f32;
                }
                table
            })
            .collect()
    })
}

/// One output sample from the `WINDOW_BYTES` bytes of its window.
pub(crate) fn filter(window: impl Iterator<Item = u8>) -> f32 {
    tables()
        .iter()
        .zip(window)
        .map(|(table, byte)| table.get(usize::from(byte)).copied().unwrap_or(0.0))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response_db(h: &[f64], freq: f64) -> f64 {
        let (mut re, mut im) = (0.0, 0.0);
        for (n, tap) in h.iter().enumerate() {
            let phase = 2.0 * PI * freq * n as f64;
            re += tap * phase.cos();
            im -= tap * phase.sin();
        }
        20.0 * re.hypot(im).log10()
    }

    #[test]
    fn the_decimation_filter_rejects_the_noise_shaping_band() {
        let h = taps();
        let dsd64 = 2_822_400.0;
        let output = dsd64 / DECIMATION as f64;
        // Pass band: flat to 20 kHz within ±0.1 dB.
        for step in 0..=200 {
            let f = 20_000.0 * f64::from(step) / 200.0;
            let gain = response_db(&h, f / dsd64);
            assert!(gain.abs() < 0.1, "{gain:.3} dB at {f:.0} Hz");
        }
        // Stop band: from the output's Nyquist frequency to the DSD one.
        let start = output / 2.0;
        let steps = 4000;
        for step in 0..=steps {
            let f = start + (dsd64 / 2.0 - start) * f64::from(step) / f64::from(steps);
            let gain = response_db(&h, f / dsd64);
            assert!(gain < -100.0, "{gain:.1} dB at {f:.0} Hz");
        }
    }

    #[test]
    fn the_idle_pattern_reads_as_silence() {
        let out = filter(std::iter::repeat_n(IDLE, WINDOW_BYTES as usize));
        assert!(out.abs() < 1e-5, "{out}");
        let full = filter(std::iter::repeat_n(0xff, WINDOW_BYTES as usize));
        assert!((full - 1.0).abs() < 1e-5, "{full}");
    }
}
