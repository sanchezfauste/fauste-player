#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Formats beyond symphonia (audio formats spec F1–F3).

use fp_decode::{Kind, probe};

#[test]
fn each_format_is_recognised_by_its_magic_bytes() {
    assert_eq!(probe(b"DSD \x1c\0\0\0", Some("dsf")), Kind::Dsf);
    assert_eq!(probe(b"FRM8\0\0\0\0\0\0\0\x10DSD ", Some("dff")), Kind::Dff);
    assert_eq!(probe(b"wvpk\x20\0\0\0", Some("wv")), Kind::WavPack);
    assert_eq!(probe(b"MAC \x96\x0f", Some("ape")), Kind::Ape);
    assert_eq!(probe(b"RIFF\0\0\0\0WAVE", Some("wav")), Kind::Symphonia);
    assert_eq!(probe(b"fLaC", None), Kind::Symphonia);
    // The content decides, not the name.
    assert_eq!(probe(b"DSD \x1c\0\0\0", Some("mp3")), Kind::Dsf);
    assert_eq!(probe(b"", Some("dsf")), Kind::Symphonia);
    // A FRM8 container that is not DSD is not DFF.
    assert_eq!(probe(b"FRM8\0\0\0\0\0\0\0\x10AIFF", None), Kind::Symphonia);
}

#[test]
fn a_float_wav_with_nan_and_infinity_decodes_to_silence_there() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad.wav");
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 48_000,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut w = hound::WavWriter::create(&path, spec).unwrap();
    for i in 0..4_800 {
        let v = match i {
            100 => f32::NAN,
            101 => f32::INFINITY,
            102 => f32::NEG_INFINITY,
            _ => 0.25,
        };
        w.write_sample(v).unwrap();
    }
    w.finalize().unwrap();
    let mut d = fp_decode::FileDecoder::open(&path).unwrap();
    let mut out = Vec::new();
    while d.next_block(&mut out).unwrap() {}
    assert!(out.len() >= 4_800);
    assert!(out.iter().all(|v| v.is_finite()));
    assert_eq!(out.get(100..103), Some(&[0.0f32, 0.0, 0.0][..]));
    assert_eq!(out.get(99), Some(&0.25));
}
