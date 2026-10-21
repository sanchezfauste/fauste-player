#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Surround files are mixed down by their channel positions (audio formats
//! spec: downmix).

use std::path::Path;

use fp_decode::FileDecoder;

const FL: u32 = 0x1;
const FR: u32 = 0x2;
const FC: u32 = 0x4;
const LFE: u32 = 0x8;
const BL: u32 = 0x10;
const BR: u32 = 0x20;
const BC: u32 = 0x100;
const SL: u32 = 0x200;
const SR: u32 = 0x400;

/// Writes a 16-bit `WAVE_FORMAT_EXTENSIBLE` file of `frames` frames; every
/// channel of `mask` is silent except `loud` (a bit of `mask`), a constant 0.5.
fn write_wav(path: &Path, mask: u32, loud: u32, frames: usize) {
    let channels = mask.count_ones() as u16;
    let rate = 48_000u32;
    let block = channels * 2;
    let data_len = frames as u32 * u32::from(block);
    let mut b = Vec::new();
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(4 + 8 + 40 + 8 + data_len).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&40u32.to_le_bytes());
    b.extend_from_slice(&0xFFFEu16.to_le_bytes());
    b.extend_from_slice(&channels.to_le_bytes());
    b.extend_from_slice(&rate.to_le_bytes());
    b.extend_from_slice(&(rate * u32::from(block)).to_le_bytes());
    b.extend_from_slice(&block.to_le_bytes());
    b.extend_from_slice(&16u16.to_le_bytes());
    b.extend_from_slice(&22u16.to_le_bytes());
    b.extend_from_slice(&16u16.to_le_bytes());
    b.extend_from_slice(&mask.to_le_bytes());
    // KSDATAFORMAT_SUBTYPE_PCM
    b.extend_from_slice(&[
        1, 0, 0, 0, 0, 0, 0x10, 0, 0x80, 0, 0, 0xAA, 0, 0x38, 0x9B, 0x71,
    ]);
    b.extend_from_slice(b"data");
    b.extend_from_slice(&data_len.to_le_bytes());
    for _ in 0..frames {
        for bit in 0..32 {
            let pos = 1u32 << bit;
            if mask & pos != 0 {
                let v: i16 = if pos == loud { 16_384 } else { 0 };
                b.extend_from_slice(&v.to_le_bytes());
            }
        }
    }
    std::fs::write(path, b).unwrap();
}

/// The stereo peak of the whole file, as `(left, right)`.
fn peaks(path: &Path) -> (f32, f32) {
    let mut d = FileDecoder::open(path).unwrap();
    let mut out = Vec::new();
    while d.next_block(&mut out).unwrap() {}
    let peak = |ch: usize| {
        out.iter()
            .skip(ch)
            .step_by(2)
            .fold(0f32, |a, v| a.max(v.abs()))
    };
    (peak(0), peak(1))
}

fn mixed(mask: u32, loud: u32) -> (f32, f32) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("s.wav");
    write_wav(&path, mask, loud, 4_800);
    peaks(&path)
}

const SEVEN_ONE: u32 = FL | FR | FC | LFE | BL | BR | SL | SR;

#[test]
fn a_tone_in_the_side_left_of_a_7_1_file_is_heard_on_the_left() {
    let (l, r) = mixed(SEVEN_ONE, SL);
    assert!(l > 0.1, "{l}");
    assert!(r < 1e-6, "{r}");
}

#[test]
fn a_tone_in_the_rear_right_of_a_quad_file_is_heard_on_the_right() {
    let (l, r) = mixed(FL | FR | BL | BR, BR);
    assert!(r > 0.1, "{r}");
    assert!(l < 1e-6, "{l}");
}

#[test]
fn every_channel_of_a_7_1_file_lands_on_its_side_and_the_lfe_is_dropped() {
    for (loud, left, right) in [
        (FL, true, false),
        (FR, false, true),
        (BL, true, false),
        (BR, false, true),
        (SL, true, false),
        (SR, false, true),
        (FC, true, true),
    ] {
        let (l, r) = mixed(SEVEN_ONE, loud);
        assert_eq!(l > 1e-3, left, "{loud:#x}: {l} {r}");
        assert_eq!(r > 1e-3, right, "{loud:#x}: {l} {r}");
    }
    assert_eq!(mixed(SEVEN_ONE, LFE), (0.0, 0.0));
}

#[test]
fn a_6_1_file_puts_its_back_centre_on_both_sides() {
    let (l, r) = mixed(FL | FR | FC | LFE | BC | SL | SR, BC);
    assert!(l > 1e-3 && (l - r).abs() < 1e-6, "{l} {r}");
}

#[test]
fn a_2_1_file_does_not_fold_its_lfe_into_the_stereo() {
    let (l, r) = mixed(FL | FR | LFE, LFE);
    assert_eq!((l, r), (0.0, 0.0));
}
