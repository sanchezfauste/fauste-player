#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Pieces of the platform exclusive modes that do not need the platform
//! (Phase 4 plan 2): sample bytes, format negotiation, physical formats.

use fp_backends::SampleFormat;
use fp_backends::exclusive::{
    Candidate, PhysicalFormat, choose_physical_format, negotiate, write_samples,
};

fn i16_at(bytes: &[u8], i: usize) -> i16 {
    i16::from_le_bytes([bytes[2 * i], bytes[2 * i + 1]])
}

fn i32_at(bytes: &[u8], i: usize) -> i32 {
    i32::from_le_bytes(bytes[4 * i..4 * i + 4].try_into().unwrap())
}

#[test]
fn sixteen_bit_samples_are_written_exactly() {
    let values: Vec<i16> = (i16::MIN..=i16::MAX).step_by(7).collect();
    let samples: Vec<f32> = values.iter().map(|v| f32::from(*v) / 32_768.0).collect();
    let mut bytes = vec![0u8; samples.len() * 2];
    write_samples(SampleFormat::I16, &samples, &mut bytes);
    for (i, v) in values.iter().enumerate() {
        assert_eq!(i16_at(&bytes, i), *v);
    }
}

#[test]
fn twenty_four_bit_samples_are_msb_aligned_in_32_bits() {
    let values: Vec<i32> = (-(1 << 23)..(1 << 23)).step_by(101).collect();
    let samples: Vec<f32> = values.iter().map(|v| *v as f32 / 8_388_608.0).collect();
    let mut bytes = vec![0u8; samples.len() * 4];
    write_samples(SampleFormat::I24, &samples, &mut bytes);
    for (i, v) in values.iter().enumerate() {
        assert_eq!(i32_at(&bytes, i), v << 8);
    }
}

#[test]
fn float_samples_are_written_unchanged() {
    let samples = [0.25f32, -1.0, 0.123_456_79];
    let mut bytes = vec![0u8; 12];
    write_samples(SampleFormat::F32, &samples, &mut bytes);
    for (i, s) in samples.iter().enumerate() {
        let got = f32::from_le_bytes(bytes[4 * i..4 * i + 4].try_into().unwrap());
        assert_eq!(got.to_bits(), s.to_bits());
    }
}

#[test]
fn out_of_range_samples_saturate() {
    let mut bytes = vec![0u8; 4];
    write_samples(SampleFormat::I16, &[2.0, -2.0], &mut bytes);
    assert_eq!((i16_at(&bytes, 0), i16_at(&bytes, 1)), (i16::MAX, i16::MIN));
}

#[test]
fn negotiation_prefers_24_bit_integer_then_16_then_float() {
    let all = negotiate(|_| true).unwrap();
    assert_eq!(all.format, SampleFormat::I24);
    let no_24 = negotiate(|c| c.format != SampleFormat::I24).unwrap();
    assert_eq!(
        no_24.format,
        SampleFormat::I32,
        "32-bit integer still carries 24 bits"
    );
    let sixteen = negotiate(|c| c.format == SampleFormat::I16 || c.format == SampleFormat::F32);
    assert_eq!(sixteen.unwrap().format, SampleFormat::I16);
    let float_only = negotiate(|c| c.format == SampleFormat::F32).unwrap();
    assert_eq!(float_only.format, SampleFormat::F32);
    assert_eq!(negotiate(|_| false), None);
    assert_eq!(
        negotiate(|_| true),
        Some(Candidate {
            format: SampleFormat::I24,
            container_bits: 32,
            valid_bits: 24,
        })
    );
}

fn physical(bits: u32, integer: bool, rates: (f64, f64)) -> PhysicalFormat {
    PhysicalFormat {
        bits,
        integer,
        linear_pcm: true,
        channels: 2,
        min_rate: rates.0,
        max_rate: rates.1,
    }
}

#[test]
fn the_widest_integer_physical_format_at_the_rate_is_chosen() {
    let formats = [
        physical(16, true, (44_100.0, 192_000.0)),
        physical(24, true, (44_100.0, 96_000.0)),
        physical(32, false, (44_100.0, 192_000.0)),
    ];
    assert_eq!(choose_physical_format(&formats, 48_000, 2), Some(1));
    assert_eq!(
        choose_physical_format(&formats, 192_000, 2),
        Some(0),
        "24-bit does not reach 192 kHz"
    );
    assert_eq!(choose_physical_format(&formats, 8_000, 2), None);
    assert_eq!(
        choose_physical_format(&[physical(32, false, (8_000.0, 96_000.0))], 48_000, 2),
        None,
        "float only: nothing to choose"
    );
}

#[test]
fn encoded_or_narrower_physical_formats_are_never_chosen() {
    let ac3 = PhysicalFormat {
        linear_pcm: false,
        ..physical(24, true, (48_000.0, 48_000.0))
    };
    let mono = PhysicalFormat {
        channels: 1,
        ..physical(24, true, (48_000.0, 48_000.0))
    };
    let stereo16 = physical(16, true, (48_000.0, 48_000.0));
    assert_eq!(
        choose_physical_format(&[ac3, mono, stereo16], 48_000, 2),
        Some(2)
    );
}
