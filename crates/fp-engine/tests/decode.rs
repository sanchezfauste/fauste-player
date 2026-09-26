#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

mod support;

use fp_engine::decode::FileDecoder;
use fp_engine::worker::file_opener;
use support::{index_of, indexed_wav};

fn decode_all(d: &mut FileDecoder) -> Vec<f32> {
    let mut out = Vec::new();
    while d.next_block(&mut out).unwrap() {}
    out
}

#[test]
fn mono_files_decode_to_duplicated_stereo_with_every_frame() {
    let dir = tempfile::tempdir().unwrap();
    let path = indexed_wav(dir.path(), "mono.wav", 44_100, 1, 88_200);
    let mut d = FileDecoder::open(&path).unwrap();
    assert_eq!(d.sample_rate(), 44_100);
    let out = decode_all(&mut d);
    assert_eq!(out.len(), 88_200 * 2);
    assert_eq!((index_of(out[20]), index_of(out[21])), (10, 10));
}

#[test]
fn seeking_lands_on_the_exact_frame() {
    let dir = tempfile::tempdir().unwrap();
    let path = indexed_wav(dir.path(), "seek.wav", 48_000, 2, 96_000);
    let mut d = FileDecoder::open(&path).unwrap();
    d.seek(1.5).unwrap();
    let out = decode_all(&mut d);
    assert_eq!(index_of(out[0]), 72_000 % 20_000);
    assert_eq!(out.len(), (96_000 - 72_000) * 2);
}

#[test]
fn the_file_opener_resamples_to_the_bus_rate_with_an_exact_length() {
    let dir = tempfile::tempdir().unwrap();
    let path = indexed_wav(dir.path(), "cd.wav", 44_100, 2, 44_100);
    let mut source = file_opener()(&path, 0.0, 48_000).unwrap();
    let mut out = Vec::new();
    while source.next_block(&mut out).unwrap() {}
    assert_eq!(out.len(), 48_000 * 2);
}

#[test]
fn a_file_that_is_not_audio_is_reported_with_its_path() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.mp3");
    std::fs::write(&path, b"definitely not an mp3").unwrap();
    let error = FileDecoder::open(&path).err().unwrap();
    assert!(error.contains("notes.mp3"), "{error}");
    assert!(FileDecoder::open(&dir.path().join("missing.flac")).is_err());
}

#[test]
fn a_truncated_file_plays_what_it_has_and_ends_cleanly() {
    let dir = tempfile::tempdir().unwrap();
    let path = indexed_wav(dir.path(), "cut.wav", 48_000, 2, 48_000);
    let bytes = std::fs::read(&path).unwrap();
    std::fs::write(&path, &bytes[..bytes.len() / 2]).unwrap(); // an interrupted download
    let mut d = FileDecoder::open(&path).unwrap();
    let out = decode_all(&mut d);
    assert!(out.len() > 20_000 * 2, "most of the audio is still there");
}

fn sine_wav(dir: &std::path::Path, rate: u32, secs: f64) -> std::path::PathBuf {
    let path = dir.join("sine.wav");
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(&path, spec).unwrap();
    let frames = (secs * f64::from(rate)) as usize;
    for i in 0..frames {
        let t = i as f64 / f64::from(rate);
        w.write_sample(((t * 1000.0 * std::f64::consts::TAU).sin() * 16_000.0) as i16)
            .unwrap();
    }
    w.finalize().unwrap();
    path
}

fn open_all(path: &std::path::Path, from: f64, frames: usize) -> Vec<f32> {
    let mut source = file_opener()(path, from, 48_000).unwrap();
    let mut out = Vec::new();
    while out.len() < frames * 2 && source.next_block(&mut out).unwrap() {}
    out.truncate(frames * 2);
    out
}

#[test]
fn a_resampled_start_after_cue_in_matches_continuous_playback() {
    let dir = tempfile::tempdir().unwrap();
    let path = sine_wav(dir.path(), 44_100, 1.0);
    let continuous = open_all(&path, 0.0, 24_000);
    let started = open_all(&path, 0.25, 64);
    let reference = &continuous[12_000 * 2..12_064 * 2];
    let worst = started
        .iter()
        .zip(reference)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    assert!(worst < 1e-4, "start transient {worst}");
}
