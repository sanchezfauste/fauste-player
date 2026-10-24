#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(clippy::indexing_slicing)]

//! Monkey's Audio decoding (audio formats spec F5), against reference
//! files whose decoded samples are known (see `fixtures/ape/README.md`).

use std::path::{Path, PathBuf};

use fp_decode::FileDecoder;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/ape")
        .join(name)
}

fn decode_all(path: &Path) -> Vec<f32> {
    let mut d = FileDecoder::open(path).unwrap();
    let mut out = Vec::new();
    while d.next_block(&mut out).unwrap() {}
    out
}

/// FNV-1a over the samples as little-endian PCM of `bits`, as in a WAV file.
fn pcm_hash(samples: &[f32], bits: u32) -> u64 {
    let scale = (1u64 << (bits - 1)) as f32;
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for s in samples {
        let v = (s * scale).round() as i32;
        for b in v.to_le_bytes().iter().take(bits as usize / 8) {
            hash ^= u64::from(*b);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    hash
}

#[test]
fn ape_decodes_to_the_reference_samples() {
    for (name, bits, frames, hash) in [
        (
            "multiframe_16s_c2000.ape",
            16,
            132_300,
            0x4042_e701_60d5_adb5,
        ),
        ("sine_24s_c2000.ape", 24, 44_100, 0x7536_c33e_edbc_89e5),
    ] {
        let path = fixture(name);
        let d = FileDecoder::open(&path).unwrap();
        assert_eq!(d.sample_rate(), 44_100, "{name}");
        assert_eq!(d.bits_per_sample(), Some(bits), "{name}");
        assert_eq!(d.channels(), 2, "{name}");
        assert_eq!(d.frames_hint(), Some(frames), "{name}");
        let samples = decode_all(&path);
        assert_eq!(samples.len() as u64, 2 * frames, "{name}");
        assert_eq!(pcm_hash(&samples, bits), hash, "{name}");
    }
}

#[test]
fn seeking_ape_lands_on_the_frame() {
    let path = fixture("multiframe_16s_c2000.ape");
    let full = decode_all(&path);
    let frames = full.len() / 2;
    for frame in [0, 1, 73_727, 73_728, 100_000, frames - 1] {
        let mut d = FileDecoder::open(&path).unwrap();
        d.seek(frame as f64 / 44_100.0).unwrap();
        let mut out = Vec::new();
        while d.next_block(&mut out).unwrap() {}
        assert!(out == full[2 * frame..], "seek to frame {frame}");
    }
    let mut d = FileDecoder::open(&path).unwrap();
    d.seek(1_000.0).unwrap();
    let mut out = Vec::new();
    while d.next_block(&mut out).unwrap() {}
    assert!(out.is_empty());
}

#[test]
fn a_damaged_ape_file_is_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let bytes = std::fs::read(fixture("multiframe_16s_c2000.ape")).unwrap();
    let path = dir.path().join("cut.ape");
    std::fs::write(&path, &bytes[..40]).unwrap();
    assert!(FileDecoder::open(&path).is_err());

    let mut damaged = bytes.clone();
    let mid = damaged.len() / 2;
    for b in &mut damaged[mid..mid + 64] {
        *b ^= 0x5a;
    }
    std::fs::write(&path, &damaged).unwrap();
    let mut d = FileDecoder::open(&path).unwrap();
    let mut out = Vec::new();
    let result = loop {
        match d.next_block(&mut out) {
            Ok(true) => continue,
            other => break other,
        }
    };
    assert!(result.is_err(), "the damaged frame is reported");
}

/// An empty ID3v2.4 tag with `padding` bytes of padding.
fn id3_prefix(padding: u8) -> Vec<u8> {
    let mut tag = b"ID3\x04\x00\x00\x00\x00\x00".to_vec();
    tag.push(padding);
    tag.extend(std::iter::repeat_n(0u8, usize::from(padding)));
    tag
}

#[test]
fn an_ape_file_with_a_leading_id3_tag_still_decodes() {
    let dir = tempfile::tempdir().unwrap();
    let mut bytes = id3_prefix(10);
    bytes.extend(std::fs::read(fixture("sine_24s_c2000.ape")).unwrap());
    let path = dir.path().join("tagged.ape");
    std::fs::write(&path, &bytes).unwrap();
    let samples = decode_all(&path);
    assert_eq!(pcm_hash(&samples, 24), 0x7536_c33e_edbc_89e5);
}

#[test]
fn a_file_named_ape_that_is_not_ape_is_refused() {
    // Never hand it to the general decoders, which may take it for MPEG
    // and play noise.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("fake.ape");
    let bytes = std::fs::read(fixture("sine_24s_c2000.ape")).unwrap();
    std::fs::write(&path, &bytes[4..]).unwrap();
    assert!(FileDecoder::open(&path).is_err());
}
