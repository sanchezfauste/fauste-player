#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(clippy::indexing_slicing)]

//! Opus decoding (audio formats spec F5): Ogg Opus files encoded here with
//! a pure-Rust encoder decode at 48 kHz, at their level, and seek to the
//! right place.

use std::f64::consts::PI;
use std::path::{Path, PathBuf};

use fp_decode::FileDecoder;
use opus_pure::{Application, MAX_PACKET_BYTES, OggOpusWriter, OpusEncoder, OpusHead};

const RATE: usize = 48_000;
const FRAME: usize = 960;

/// Encodes interleaved stereo `pcm` at 48 kHz as an Ogg Opus file.
fn encode(dir: &Path, name: &str, pcm: &[f32]) -> PathBuf {
    let mut encoder = OpusEncoder::new(48_000, 2, Application::Audio).unwrap();
    let head = OpusHead::for_encoder(&encoder, 48_000);
    let mut writer = OggOpusWriter::new(Vec::new(), head).unwrap();
    let mut packet = vec![0u8; MAX_PACKET_BYTES];
    for block in pcm.as_chunks::<{ FRAME * 2 }>().0 {
        let n = encoder.encode(block, FRAME, &mut packet).unwrap();
        writer.write_packet(&packet[..n]).unwrap();
    }
    let path = dir.join(name);
    std::fs::write(&path, writer.finish().unwrap()).unwrap();
    path
}

fn stereo(secs: f64, f: impl Fn(f64) -> f64) -> Vec<f32> {
    (0..(secs * RATE as f64) as usize)
        .flat_map(|i| {
            let v = f(i as f64 / RATE as f64) as f32;
            [v, v]
        })
        .collect()
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

#[test]
fn an_ogg_opus_tone_decodes_at_48_khz_with_its_level() {
    let dir = tempfile::tempdir().unwrap();
    let pcm = stereo(2.0, |t| 0.5 * (2.0 * PI * 1000.0 * t).sin());
    let path = encode(dir.path(), "tone.opus", &pcm);
    let d = FileDecoder::open(&path).unwrap();
    assert_eq!(d.sample_rate(), 48_000);
    assert_eq!(d.bits_per_sample(), None, "Opus is lossy");
    assert_eq!(d.channels(), 2);
    let x = left(&decode_all(&path));
    assert!(
        x.len().abs_diff(2 * RATE) <= FRAME,
        "{} frames for 2 s",
        x.len()
    );
    // 0.5 s to 1.5 s: 1000 whole cycles.
    let segment = &x[RATE / 2..RATE / 2 + RATE];
    let w = 2.0 * PI * 1000.0 / RATE as f64;
    let (mut s, mut c) = (0.0, 0.0);
    for (i, v) in segment.iter().enumerate() {
        s += v * (w * i as f64).sin();
        c += v * (w * i as f64).cos();
    }
    let n = segment.len() as f64;
    let level = 20.0 * (2.0 * s / n).hypot(2.0 * c / n).log10();
    assert!((level - (-6.02)).abs() < 0.5, "level {level:.2} dBFS");
}

/// A signal that never repeats within the file: three unrelated tones.
fn marker(t: f64) -> f64 {
    0.2 * (2.0 * PI * 441.0 * t).sin()
        + 0.15 * (2.0 * PI * 1234.5 * t + 1.0).sin()
        + 0.1 * (2.0 * PI * 3210.7 * t + 2.0).sin()
}

#[test]
fn seeking_opus_lands_within_a_frame() {
    let dir = tempfile::tempdir().unwrap();
    let path = encode(dir.path(), "marker.opus", &stereo(3.0, marker));
    let full = left(&decode_all(&path));
    for secs in [0.0, 1.0, 2.2] {
        let mut d = FileDecoder::open(&path).unwrap();
        d.seek(secs).unwrap();
        let mut out = Vec::new();
        while out.len() < 2 * (FRAME + 4800) && d.next_block(&mut out).unwrap() {}
        let seeked = left(&out);
        // Skip one frame while the decoder settles, then find where the
        // next 100 ms sit in the full decode.
        let probe = &seeked[FRAME..FRAME + 4800];
        let expected = (secs * RATE as f64) as isize + FRAME as isize;
        let lag = (-(FRAME as isize)..=FRAME as isize)
            .filter(|lag| expected + lag >= 0)
            .max_by(|a, b| {
                let score = |lag: isize| -> f64 {
                    let at = (expected + lag) as usize;
                    probe.iter().zip(&full[at..]).map(|(p, f)| p * f).sum()
                };
                score(*a).total_cmp(&score(*b))
            })
            .unwrap();
        assert!(lag.abs() <= 48, "seek to {secs} s is {lag} samples off");
    }
}
