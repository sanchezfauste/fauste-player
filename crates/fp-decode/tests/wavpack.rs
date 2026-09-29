#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(clippy::indexing_slicing)]

//! WavPack decoding (audio formats spec F5): files encoded here with
//! wavicle's encoder decode to exactly the source samples.

use std::path::{Path, PathBuf};

use fp_decode::FileDecoder;
use wavicle::{EncodeParams, encode_int};

/// Interleaved stereo integer samples: a 1 kHz sine left, a ramp right, both
/// reaching full scale for `bits`. Long enough for several blocks.
fn source(bits: u32, frames: usize) -> Vec<i32> {
    let full = (1i64 << (bits - 1)) - 1;
    (0..frames)
        .flat_map(|i| {
            let t = i as f64 / 44_100.0;
            let l = ((2.0 * std::f64::consts::PI * 1000.0 * t).sin() * full as f64) as i32;
            let r = ((i as i64 * 37) % (2 * full) - full) as i32;
            [l, r]
        })
        .collect()
}

fn encode(dir: &Path, name: &str, bits: u32, samples: &[i32]) -> PathBuf {
    let params = EncodeParams {
        channels: 2,
        sample_rate: 44_100,
        bits_per_sample: bits,
    };
    let path = dir.join(name);
    std::fs::write(&path, encode_int(params, samples).unwrap()).unwrap();
    path
}

fn decode_all(path: &Path) -> Vec<f32> {
    let mut d = FileDecoder::open(path).unwrap();
    let mut out = Vec::new();
    while d.next_block(&mut out).unwrap() {}
    out
}

fn scaled(samples: &[i32], bits: u32) -> Vec<f32> {
    let scale = 1.0 / (1u64 << (bits - 1)) as f32;
    samples.iter().map(|&s| s as f32 * scale).collect()
}

#[test]
fn wavpack_16_and_24_bit_decode_bit_exact() {
    let dir = tempfile::tempdir().unwrap();
    for bits in [16, 24] {
        let samples = source(bits, 100_000);
        let path = encode(dir.path(), &format!("t{bits}.wv"), bits, &samples);
        let d = FileDecoder::open(&path).unwrap();
        assert_eq!(d.sample_rate(), 44_100);
        assert_eq!(d.bits_per_sample(), Some(bits));
        assert_eq!(d.channels(), 2);
        assert_eq!(d.frames_hint(), Some(100_000));
        assert!(decode_all(&path) == scaled(&samples, bits), "{bits}-bit");
    }
}

#[test]
fn seeking_wavpack_lands_on_the_frame() {
    let dir = tempfile::tempdir().unwrap();
    let samples = source(16, 100_000);
    let path = encode(dir.path(), "t.wv", 16, &samples);
    let full = decode_all(&path);
    for frame in [0usize, 1, 32_767, 32_768, 50_000, 99_999] {
        let mut d = FileDecoder::open(&path).unwrap();
        d.seek(frame as f64 / 44_100.0).unwrap();
        let mut out = Vec::new();
        while d.next_block(&mut out).unwrap() {}
        assert!(out == full[2 * frame..], "seek to frame {frame}");
    }
    let mut d = FileDecoder::open(&path).unwrap();
    d.seek(10_000.0).unwrap();
    let mut out = Vec::new();
    while d.next_block(&mut out).unwrap() {}
    assert!(out.is_empty());
}

#[test]
fn a_corrupt_wavpack_block_is_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let samples = source(16, 100_000);
    let path = encode(dir.path(), "t.wv", 16, &samples);
    let mut bytes = std::fs::read(&path).unwrap();
    // Flip bits in the middle of the second block's audio.
    let second = bytes
        .windows(4)
        .enumerate()
        .skip(4)
        .find(|(_, w)| w == b"wvpk")
        .map(|(i, _)| i)
        .unwrap();
    for b in &mut bytes[second + 200..second + 260] {
        *b ^= 0x5a;
    }
    std::fs::write(&path, &bytes).unwrap();
    let mut d = FileDecoder::open(&path).unwrap();
    let mut out = Vec::new();
    let result = loop {
        match d.next_block(&mut out) {
            Ok(true) => continue,
            other => break other,
        }
    };
    assert!(result.is_err(), "the damaged block is reported");

    // A truncated file and a lying block size are errors too, never panics.
    std::fs::write(&path, &bytes[..20]).unwrap();
    assert!(FileDecoder::open(&path).is_err());
    let mut lying = bytes.clone();
    lying[4..8].copy_from_slice(&u32::MAX.to_le_bytes());
    std::fs::write(&path, &lying).unwrap();
    assert!(FileDecoder::open(&path).is_err());
}

#[test]
fn a_wavpack_file_with_a_leading_id3_tag_still_decodes() {
    let dir = tempfile::tempdir().unwrap();
    let samples = source(16, 50_000);
    let path = encode(dir.path(), "t.wv", 16, &samples);
    let mut bytes = b"ID3\x04\x00\x00\x00\x00\x00\x0a".to_vec();
    bytes.extend([0u8; 10]);
    bytes.extend(std::fs::read(&path).unwrap());
    std::fs::write(&path, &bytes).unwrap();
    assert!(decode_all(&path) == scaled(&samples, 16));
}
