#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(clippy::indexing_slicing)]

//! DSD decoding (audio formats spec F3, F5). The DSD streams are generated
//! here by a 5th-order sigma-delta modulator, written as DSF or DFF, and
//! decoded through `FileDecoder`.

use std::f64::consts::PI;
use std::path::{Path, PathBuf};

use fp_decode::FileDecoder;

const DSD64: u32 = 2_822_400;
const PCM_RATE: u32 = DSD64 / 32;

// ---------------------------------------------------------------- modulator

fn mul(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0)
}

fn div(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    let d = b.0 * b.0 + b.1 * b.1;
    ((a.0 * b.0 + a.1 * b.1) / d, (a.1 * b.0 - a.0 * b.1) / d)
}

/// Poles (in z) of a 5th-order Butterworth high-pass with cutoff `fc`
/// (cycles per sample), through the bilinear transform.
fn poles(fc: f64) -> Vec<(f64, f64)> {
    let n = 5;
    let wa = 2.0 * (PI * fc).tan();
    (0..n)
        .map(|k| {
            let theta = PI * f64::from(2 * k + n + 1) / f64::from(2 * n);
            let p = (theta.cos(), theta.sin());
            let q = div((wa, 0.0), p);
            div((1.0 + q.0 / 2.0, q.1 / 2.0), (1.0 - q.0 / 2.0, -q.1 / 2.0))
        })
        .collect()
}

/// The monic denominator of NTF(z) = (1 - z^-1)^5 / A(z).
fn denominator(fc: f64) -> [f64; 6] {
    let mut coeffs = vec![(1.0, 0.0)];
    for p in poles(fc) {
        let mut next = vec![(0.0, 0.0); coeffs.len() + 1];
        for (i, c) in coeffs.iter().enumerate() {
            next[i].0 += c.0;
            next[i].1 += c.1;
            let t = mul(*c, p);
            next[i + 1].0 -= t.0;
            next[i + 1].1 -= t.1;
        }
        coeffs = next;
    }
    let mut a = [0.0; 6];
    for (i, c) in coeffs.iter().enumerate() {
        a[i] = c.0;
    }
    a
}

/// An NTF whose gain at half the sampling rate is 1.5 (Lee's rule), so the
/// 1-bit loop is stable at the SACD reference level.
fn ntf() -> ([f64; 6], [f64; 6]) {
    let b = [1.0, -5.0, 10.0, -10.0, 5.0, -1.0];
    let gain = |fc: f64| {
        let a = denominator(fc);
        let at_nyquist: f64 = a
            .iter()
            .enumerate()
            .map(|(i, c)| c * (-1f64).powi(i as i32))
            .sum();
        32.0 / at_nyquist.abs()
    };
    let (mut lo, mut hi) = (1e-5, 0.25);
    for _ in 0..100 {
        let mid = (lo + hi) / 2.0;
        if gain(mid) > 1.5 {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    (b, denominator(lo))
}

/// DSD bits (true = +1) of `x(t)` at `rate`, from an error-feedback
/// modulator with the NTF above.
fn modulate(x: impl Fn(f64) -> f64, rate: u32, samples: usize) -> Vec<bool> {
    let (b, a) = ntf();
    let mut e = [0.0f64; 6];
    let mut f = [0.0f64; 6];
    (0..samples)
        .map(|n| {
            let mut fb = 0.0;
            for i in 1..6 {
                fb += (b[i] - a[i]) * e[i - 1] - a[i] * f[i - 1];
            }
            let w = x(n as f64 / f64::from(rate)) + fb;
            let bit = w >= 0.0;
            let y = if bit { 1.0 } else { -1.0 };
            e.rotate_right(1);
            f.rotate_right(1);
            e[0] = y - w;
            f[0] = fb;
            bit
        })
        .collect()
}

/// Bits packed MSB first (earliest bit in the high bit).
fn pack(bits: &[bool]) -> Vec<u8> {
    bits.chunks(8)
        .map(|c| {
            c.iter()
                .enumerate()
                .fold(0u8, |byte, (i, &b)| byte | (u8::from(b) << (7 - i)))
        })
        .collect()
}

fn sine(freq: f64, amplitude: f64) -> impl Fn(f64) -> f64 {
    move |t| amplitude * (2.0 * PI * freq * t).sin()
}

// ------------------------------------------------------------------ writers

const BLOCK: usize = 4096;

/// A DSF file: per-channel blocks of 4096 bytes, LSB first.
fn dsf(channels: &[Vec<u8>], sample_bits: u64) -> Vec<u8> {
    let len = channels[0].len();
    let blocks = len.div_ceil(BLOCK);
    let mut data = Vec::new();
    for g in 0..blocks {
        for ch in channels {
            let mut block = vec![0u8; BLOCK];
            for (i, byte) in ch.iter().skip(g * BLOCK).take(BLOCK).enumerate() {
                block[i] = byte.reverse_bits();
            }
            data.extend(block);
        }
    }
    let mut f = Vec::new();
    let total = 28 + 52 + 12 + data.len() as u64;
    f.extend(b"DSD ");
    f.extend(28u64.to_le_bytes());
    f.extend(total.to_le_bytes());
    f.extend(0u64.to_le_bytes());
    f.extend(b"fmt ");
    f.extend(52u64.to_le_bytes());
    f.extend(1u32.to_le_bytes()); // version
    f.extend(0u32.to_le_bytes()); // DSD raw
    f.extend((channels.len() as u32).to_le_bytes()); // channel type (1 mono, 2 stereo)
    f.extend((channels.len() as u32).to_le_bytes());
    f.extend(DSD64.to_le_bytes());
    f.extend(1u32.to_le_bytes()); // LSB first
    f.extend(sample_bits.to_le_bytes());
    f.extend((BLOCK as u32).to_le_bytes());
    f.extend(0u32.to_le_bytes());
    f.extend(b"data");
    f.extend((12 + data.len() as u64).to_le_bytes());
    f.extend(data);
    f
}

fn chunk(id: &[u8; 4], body: &[u8]) -> Vec<u8> {
    let mut c = id.to_vec();
    c.extend((body.len() as u64).to_be_bytes());
    c.extend(body);
    if body.len() % 2 == 1 {
        c.push(0);
    }
    c
}

/// A DSDIFF file: interleaved bytes, MSB first. `compression` is `DSD ` or
/// `DST `.
fn dff(channels: &[Vec<u8>], compression: &[u8; 4]) -> Vec<u8> {
    let mut prop = b"SND ".to_vec();
    prop.extend(chunk(b"FS  ", &DSD64.to_be_bytes()));
    let mut chnl = (channels.len() as u16).to_be_bytes().to_vec();
    for id in [b"SLFT", b"SRGT", b"C   ", b"LFE ", b"LS  ", b"RS  "]
        .iter()
        .take(channels.len())
    {
        chnl.extend(*id);
    }
    prop.extend(chunk(b"CHNL", &chnl));
    let mut cmpr = compression.to_vec();
    cmpr.extend([14]);
    cmpr.extend(b"not compressed");
    prop.extend(chunk(b"CMPR", &cmpr));
    let mut data = Vec::new();
    for i in 0..channels[0].len() {
        for ch in channels {
            data.push(ch[i]);
        }
    }
    let data_id = if compression == b"DST " {
        b"DST "
    } else {
        b"DSD "
    };
    let mut body = b"DSD ".to_vec();
    body.extend(chunk(b"FVER", &0x0105_0000u32.to_be_bytes()));
    body.extend(chunk(b"PROP", &prop));
    body.extend(chunk(data_id, &data));
    chunk(b"FRM8", &body)
}

fn write(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, bytes).unwrap();
    path
}

fn decode_all(path: &Path) -> Vec<f32> {
    let mut d = FileDecoder::open(path).unwrap();
    let mut out = Vec::new();
    while d.next_block(&mut out).unwrap() {}
    out
}

fn left(stereo: &[f32]) -> Vec<f64> {
    stereo.iter().step_by(2).map(|&s| f64::from(s)).collect()
}

/// Amplitude of the `freq` component and the rest, over whole cycles.
fn fit(x: &[f64], freq: f64) -> (f64, Vec<f64>) {
    let w = 2.0 * PI * freq / f64::from(PCM_RATE);
    let n = x.len() as f64;
    let mean = x.iter().sum::<f64>() / n;
    let (mut s, mut c) = (0.0, 0.0);
    for (i, v) in x.iter().enumerate() {
        s += (v - mean) * (w * i as f64).sin();
        c += (v - mean) * (w * i as f64).cos();
    }
    let (a, b) = (2.0 * s / n, 2.0 * c / n);
    let residual = x
        .iter()
        .enumerate()
        .map(|(i, v)| v - mean - a * (w * i as f64).sin() - b * (w * i as f64).cos())
        .collect();
    (a.hypot(b), residual)
}

/// A 20 kHz low-pass for measuring (Kaiser windowed sinc, ~100 dB).
fn audio_band(x: &[f64]) -> Vec<f64> {
    let taps = 511;
    let fc = 21_000.0 / f64::from(PCM_RATE);
    let beta = 10.0;
    let i0 = |x: f64| {
        let (mut sum, mut term) = (1.0, 1.0);
        for k in 1..50 {
            term *= (x / 2.0 / f64::from(k)).powi(2);
            sum += term;
        }
        sum
    };
    let mid = (taps - 1) as f64 / 2.0;
    let h: Vec<f64> = (0..taps)
        .map(|n| {
            let t = n as f64 - mid;
            let sinc = if t == 0.0 {
                2.0 * fc
            } else {
                (2.0 * PI * fc * t).sin() / (PI * t)
            };
            let r = t / mid;
            sinc * i0(beta * (1.0 - r * r).max(0.0).sqrt()) / i0(beta)
        })
        .collect();
    (0..x.len().saturating_sub(taps))
        .map(|i| h.iter().zip(&x[i..i + taps]).map(|(a, b)| a * b).sum())
        .collect()
}

fn db(x: f64) -> f64 {
    20.0 * x.log10()
}

// -------------------------------------------------------------------- tests

#[test]
fn dsd64_sine_decodes_at_the_sacd_reference_level() {
    let dir = tempfile::tempdir().unwrap();
    let samples = DSD64 as usize / 2;
    let bits = modulate(sine(1000.0, 0.5), DSD64, samples);
    let path = write(dir.path(), "tone.dsf", &dsf(&[pack(&bits)], samples as u64));
    let d = FileDecoder::open(&path).unwrap();
    assert_eq!(d.sample_rate(), PCM_RATE);
    assert_eq!(
        d.bits_per_sample(),
        None,
        "DSD is converted, never bit-perfect"
    );
    assert_eq!(d.channels(), 1);
    assert_eq!(d.frames_hint(), Some(samples as u64 / 32));
    let x = left(&decode_all(&path));
    assert_eq!(x.len(), samples / 32);
    // 0.1 s to 0.4 s: 300 whole cycles.
    let (amplitude, _) = fit(&x[8820..8820 + 26_460], 1000.0);
    assert!(
        (db(amplitude) - (-6.02)).abs() < 0.2,
        "level {:.2} dBFS",
        db(amplitude)
    );
}

#[test]
fn dsd_conversion_keeps_distortion_below_minus_80_db() {
    let dir = tempfile::tempdir().unwrap();
    let samples = DSD64 as usize / 2;
    let bits = modulate(sine(1000.0, 0.5), DSD64, samples);
    let path = write(dir.path(), "tone.dsf", &dsf(&[pack(&bits)], samples as u64));
    let x = left(&decode_all(&path));
    let (amplitude, residual) = fit(&x[8820..8820 + 26_460], 1000.0);
    let noise = audio_band(&residual);
    let noise_power = noise.iter().map(|v| v * v).sum::<f64>() / noise.len() as f64;
    let thd_n = 10.0 * (noise_power / (amplitude * amplitude / 2.0)).log10();
    assert!(thd_n < -80.0, "THD+N {thd_n:.1} dB");
}

fn stereo_pair() -> Vec<Vec<u8>> {
    let samples = DSD64 as usize / 10 + 1000; // not a whole number of blocks
    vec![
        pack(&modulate(sine(1000.0, 0.5), DSD64, samples)),
        pack(&modulate(sine(440.0, 0.25), DSD64, samples)),
    ]
}

#[test]
fn dsf_and_dff_readers_agree() {
    let dir = tempfile::tempdir().unwrap();
    let channels = stereo_pair();
    let bits = channels[0].len() as u64 * 8;
    let a = decode_all(&write(dir.path(), "a.dsf", &dsf(&channels, bits)));
    let b = decode_all(&write(dir.path(), "b.dff", &dff(&channels, b"DSD ")));
    assert_eq!(a.len(), b.len());
    assert!(a == b, "DSF and DFF decode differently");
    let l = left(&a);
    let r: Vec<f64> = a.iter().skip(1).step_by(2).map(|&s| f64::from(s)).collect();
    assert!(
        l.iter().zip(&r).any(|(l, r)| (l - r).abs() > 0.1),
        "channels kept apart"
    );
}

#[test]
fn seeking_a_dsd_file_lands_on_the_frame() {
    let dir = tempfile::tempdir().unwrap();
    let channels = stereo_pair();
    let bits = channels[0].len() as u64 * 8;
    for path in [
        write(dir.path(), "a.dsf", &dsf(&channels, bits)),
        write(dir.path(), "b.dff", &dff(&channels, b"DSD ")),
    ] {
        let full = decode_all(&path);
        let frames = full.len() / 2;
        for secs in [0.0, 0.05, frames as f64 / f64::from(PCM_RATE) - 0.001] {
            let mut d = FileDecoder::open(&path).unwrap();
            d.seek(secs).unwrap();
            let mut out = Vec::new();
            while d.next_block(&mut out).unwrap() {}
            let k = (secs * f64::from(PCM_RATE)).round() as usize;
            assert_eq!(
                out.len(),
                full.len() - 2 * k,
                "{} at {secs}",
                path.display()
            );
            assert!(out == full[2 * k..], "{} at {secs}", path.display());
        }
        let mut d = FileDecoder::open(&path).unwrap();
        d.seek(1000.0).unwrap();
        let mut out = Vec::new();
        assert!(!d.next_block(&mut out).unwrap() || out.is_empty());
    }
}

#[test]
fn dst_compressed_dff_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let path = write(dir.path(), "dst.dff", &dff(&stereo_pair(), b"DST "));
    let err = FileDecoder::open(&path).err().expect("DST is refused");
    assert!(err.contains("DST"), "{err}");
}

#[test]
fn a_truncated_dsd_header_is_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let whole = dsf(&stereo_pair(), 1000);
    for cut in [4, 28, 60, 92] {
        let path = write(dir.path(), "cut.dsf", &whole[..cut]);
        assert!(FileDecoder::open(&path).is_err(), "cut at {cut}");
    }
    let whole = dff(&stereo_pair(), b"DSD ");
    let path = write(dir.path(), "cut.dff", &whole[..40]);
    assert!(FileDecoder::open(&path).is_err());

    // A PROP sub-chunk that claims an enormous length.
    let mut huge = dff(&stereo_pair(), b"DSD ");
    let fs = huge.windows(4).position(|w| w == b"FS  ").unwrap();
    huge[fs + 4..fs + 12].copy_from_slice(&(u64::MAX - 3).to_be_bytes());
    let path = write(dir.path(), "huge.dff", &huge);
    assert!(FileDecoder::open(&path).is_err());

    // A header that claims far more audio than the file holds is clamped to
    // the file, and bad channel counts are refused.
    let mut lying = dsf(&stereo_pair(), 8);
    lying[64..72].copy_from_slice(&(u64::MAX / 2).to_le_bytes());
    let path = write(dir.path(), "lying.dsf", &lying);
    let d = FileDecoder::open(&path).unwrap();
    assert!(d.frames_hint().unwrap() < 1_000_000);
    let mut zero_channels = dsf(&stereo_pair(), 8);
    zero_channels[52..56].copy_from_slice(&0u32.to_le_bytes());
    let path = write(dir.path(), "zero.dsf", &zero_channels);
    assert!(FileDecoder::open(&path).is_err());
}
