#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! DSD carried through the f32 path (feedback 2 spec O25): words, DoP 1.1
//! and the native ALSA formats, byte for byte.

use fp_backends::SampleFormat;
use fp_backends::dsd::{
    DOP_MARKERS, DSD_SILENCE, DopEncoder, NativeDsdFormat, choose_native_format, dop_sample,
    pack_native, sample_to_word, silence_sample, word_to_sample,
};
use fp_backends::exclusive::write_samples;

#[test]
fn every_word_survives_the_f32_path() {
    for first in 0..=255u8 {
        for second in [0x00, 0x01, 0x69, 0x7F, 0x80, 0xFE, 0xFF] {
            assert_eq!(
                sample_to_word(word_to_sample(first, second)),
                [first, second]
            );
        }
    }
    assert_eq!(sample_to_word(silence_sample()), [DSD_SILENCE, DSD_SILENCE]);
}

/// The 24-bit DoP word the device receives for `sample`, through the same
/// integer conversion as a real stream.
fn device_word(format: SampleFormat, sample: f32) -> u32 {
    let mut bytes = [0u8; 4];
    write_samples(format, &[sample], &mut bytes);
    u32::from_le_bytes(bytes) >> 8
}

#[test]
fn a_dop_sample_is_the_marker_above_the_two_bytes_exactly() {
    for format in [SampleFormat::I24, SampleFormat::I32] {
        for marker in DOP_MARKERS {
            for (a, b) in [
                (0x00, 0x00),
                (0x69, 0x69),
                (0xAB, 0xCD),
                (0xFF, 0xFF),
                (0x80, 0x01),
            ] {
                let s = dop_sample(marker, word_to_sample(a, b));
                let expected = (u32::from(marker) << 16) | (u32::from(a) << 8) | u32::from(b);
                assert_eq!(
                    device_word(format, s),
                    expected,
                    "{format:?} {marker:#x} {a:#x}{b:#x}"
                );
            }
        }
    }
}

#[test]
fn markers_alternate_frame_by_frame_across_calls_and_channels() {
    let mut enc = DopEncoder::new();
    let w = word_to_sample(0x12, 0x34);
    let mut first = vec![w; 3 * 2]; // 3 stereo frames
    let mut second = vec![w; 2 * 2];
    enc.encode_in_place(&mut first, 2);
    enc.encode_in_place(&mut second, 2);
    let markers: Vec<u32> = first
        .iter()
        .chain(&second)
        .map(|s| device_word(SampleFormat::I24, *s) >> 16)
        .collect();
    assert_eq!(
        markers,
        vec![0x05, 0x05, 0xFA, 0xFA, 0x05, 0x05, 0xFA, 0xFA, 0x05, 0x05]
    );
}

#[test]
fn native_formats_pack_bytes_in_time_order() {
    // Two stereo word frames: left 0x0102 then 0x0304, right 0xA1A2 then 0xA3A4.
    let words = [
        word_to_sample(0x01, 0x02),
        word_to_sample(0xA1, 0xA2),
        word_to_sample(0x03, 0x04),
        word_to_sample(0xA3, 0xA4),
    ];
    let cases: [(NativeDsdFormat, &[u8]); 5] = [
        (
            NativeDsdFormat::U32Be,
            &[1, 2, 3, 4, 0xA1, 0xA2, 0xA3, 0xA4],
        ),
        (
            NativeDsdFormat::U32Le,
            &[4, 3, 2, 1, 0xA4, 0xA3, 0xA2, 0xA1],
        ),
        (
            NativeDsdFormat::U16Be,
            &[1, 2, 0xA1, 0xA2, 3, 4, 0xA3, 0xA4],
        ),
        (
            NativeDsdFormat::U16Le,
            &[2, 1, 0xA2, 0xA1, 4, 3, 0xA4, 0xA3],
        ),
        (NativeDsdFormat::U8, &[1, 0xA1, 2, 0xA2, 3, 0xA3, 4, 0xA4]),
    ];
    for (format, expected) in cases {
        let mut out = [0u8; 8];
        assert_eq!(pack_native(format, &words, 2, &mut out), 8, "{format:?}");
        assert_eq!(&out, expected, "{format:?}");
    }
}

#[test]
fn native_device_rates_follow_the_container() {
    let word_rate = 176_400; // DSD64
    assert_eq!(NativeDsdFormat::U32Be.device_rate(word_rate), 88_200);
    assert_eq!(NativeDsdFormat::U16Le.device_rate(word_rate), 176_400);
    assert_eq!(NativeDsdFormat::U8.device_rate(word_rate), 352_800);
}

#[test]
fn the_preferred_native_format_is_32_bit_big_endian() {
    assert_eq!(choose_native_format(|_| true), Some(NativeDsdFormat::U32Be));
    assert_eq!(
        choose_native_format(|f| matches!(f, NativeDsdFormat::U8 | NativeDsdFormat::U16Le)),
        Some(NativeDsdFormat::U16Le)
    );
    assert_eq!(choose_native_format(|_| false), None);
}
