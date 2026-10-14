#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(clippy::indexing_slicing)]

//! Opus decoding (audio formats spec F5): Ogg Opus files encoded here with
//! a pure-Rust encoder decode at 48 kHz, at their level, and seek to the
//! right place.

#[path = "support/opus_files.rs"]
mod opus_files;

use std::f64::consts::PI;
use std::path::Path;

use fp_decode::FileDecoder;
use opus_files::{FRAME, RATE, encode, marker, matroska, matroska_with, packets, stereo};

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

#[test]
fn a_seek_pre_rolls_so_the_first_frame_is_already_right() {
    // RFC 7845 §4.6: decoding restarts 80 ms early, so what plays right
    // after a seek matches an uninterrupted decode.
    let dir = tempfile::tempdir().unwrap();
    let path = encode(dir.path(), "marker.opus", &stereo(3.0, marker));
    let full = left(&decode_all(&path));
    let mut d = FileDecoder::open(&path).unwrap();
    d.seek(1.5).unwrap();
    let mut out = Vec::new();
    while out.len() < 4 * FRAME && d.next_block(&mut out).unwrap() {}
    let first = &left(&out)[..FRAME];
    let at = (1.5 * RATE as f64) as usize;
    let error = first
        .iter()
        .zip(&full[at..at + FRAME])
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f64>()
        / FRAME as f64;
    let power = full[at..at + FRAME].iter().map(|v| v * v).sum::<f64>() / FRAME as f64;
    let snr = 10.0 * (power / error.max(1e-20)).log10();
    // Without the pre-roll this frame is about 5 dB; with it, 25 (a lossy
    // decoder converges, it does not snap).
    assert!(snr > 20.0, "first frame after the seek: {snr:.1} dB SNR");
}

#[test]
fn opus_in_matroska_drops_its_codec_delay_like_ogg() {
    let dir = tempfile::tempdir().unwrap();
    let pcm = stereo(1.0, marker);
    let ogg = left(&decode_all(&encode(dir.path(), "a.opus", &pcm)));
    let (packets, pre_skip) = packets(&pcm);
    assert!(pre_skip > 0);
    let mka = left(&decode_all(&matroska(
        dir.path(),
        "a.mka",
        &packets,
        pre_skip,
    )));
    let probe = &mka[..4800];
    let lag = (0..=400usize)
        .max_by(|a, b| {
            let score =
                |lag: usize| -> f64 { probe.iter().zip(&ogg[lag..]).map(|(p, f)| p * f).sum() };
            score(*a).total_cmp(&score(*b))
        })
        .unwrap();
    let back = (0..=400usize)
        .max_by(|a, b| {
            let score = |lag: usize| -> f64 {
                ogg[..4800]
                    .iter()
                    .zip(&mka[lag..])
                    .map(|(p, f)| p * f)
                    .sum()
            };
            score(*a).total_cmp(&score(*b))
        })
        .unwrap();
    assert_eq!((lag, back), (0, 0), "Matroska starts {back} samples late");
}

/// Programme-like audio: two tones and a little noise.
fn music(secs: f64) -> Vec<f32> {
    stereo(secs, |t| {
        let i = (t * RATE as f64) as u32;
        let seed = i.wrapping_mul(2_654_435_761).rotate_left(13) ^ i;
        let noise = f64::from(seed >> 8) / f64::from(1u32 << 24) - 0.5;
        0.3 * (2.0 * PI * 220.0 * t).sin() + 0.2 * (2.0 * PI * 1_760.0 * t).sin() + 0.05 * noise
    })
}

/// The same packets decoded by a reference decoder (byte-exact with libopus
/// where the format allows) after the pre-skip: the output must match it.
#[test]
fn opus_decodes_like_the_reference_decoder() {
    let dir = tempfile::tempdir().unwrap();
    let path = encode(dir.path(), "music.opus", &music(3.0));
    let ours = decode_all(&path);
    let mut reader = opus_pure::OggOpusReader::new(std::fs::File::open(&path).unwrap()).unwrap();
    let skip = usize::from(reader.head().pre_skip) * 2;
    let mut reference_decoder = opus_pure::OpusDecoder::new(48_000, 2).unwrap();
    let mut reference = Vec::new();
    let mut pcm = vec![0f32; 5760 * 2];
    while let Some(p) = reader.read_packet().unwrap() {
        let n = reference_decoder.decode(&p.data, 5760, &mut pcm).unwrap();
        reference.extend_from_slice(&pcm[..n * 2]);
    }
    // Ours drops the pre-skip; the reference's raw packets still hold it.
    let reference = &reference[skip..];
    let n = ours.len().min(reference.len());
    assert!(n > 2 * 48_000 * 2, "{n}");
    let worst = ours[..n]
        .iter()
        .zip(&reference[..n])
        .map(|(a, b)| (a - b).abs())
        .fold(0f32, f32::max);
    // `opus-decoder` goes through 16-bit PCM: about 3e-5 apart.
    assert!(worst < 1e-3, "worst difference {worst}");
}

/// Decoding keeps far ahead of real time, so a long Opus recording can be
/// analysed and several can play at once (plan 4, item 15). The bound is
/// loose for shared CI runners in a debug build (3.3× real time); the
/// direct DFT took 4.7 s here, the fixed decoder runs far below it.
#[test]
fn opus_decodes_far_faster_than_real_time() {
    let dir = tempfile::tempdir().unwrap();
    let path = encode(dir.path(), "long.opus", &music(10.0));
    let t0 = std::time::Instant::now();
    let out = decode_all(&path);
    let took = t0.elapsed().as_secs_f64();
    assert!(out.len() >= 9 * 48_000 * 2);
    assert!(took < 3.0, "10 s of Opus took {took:.2} s to decode");
}

/// Where the loudest sample of `stereo`'s left channel is.
fn loudest(stereo: &[f32]) -> usize {
    left(stereo)
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.abs().total_cmp(&b.1.abs()))
        .unwrap()
        .0
}

/// The pre-skip (RFC 7845 §4.2) is dropped in Ogg and in Matroska: a click
/// comes out where it went in, and the file is not longer than its source.
#[test]
fn the_pre_skip_is_dropped_so_audio_stays_in_place() {
    let dir = tempfile::tempdir().unwrap();
    let mut pcm = vec![0f32; RATE * 2];
    for i in 24_000..24_048 {
        pcm[i * 2] = 0.8;
        pcm[i * 2 + 1] = 0.8;
    }
    let ogg = decode_all(&encode(dir.path(), "click.opus", &pcm));
    let (packets, pre_skip) = packets(&pcm);
    let mka = decode_all(&matroska(dir.path(), "click.mka", &packets, pre_skip));
    for (name, out) in [("ogg", &ogg), ("mka", &mka)] {
        let at = loudest(out);
        assert!(
            (24_000..24_048).contains(&at),
            "{name}: the click is at {at}"
        );
        assert!(out.len() / 2 <= RATE, "{name}: {} frames", out.len() / 2);
    }
}

/// A restart from 0 after reading plays exactly what the first play did:
/// the pre-skip is dropped by count again, not placed by timestamps
/// (Matroska's are whole milliseconds, 48 samples apart).
#[test]
fn a_seek_back_to_the_start_replays_the_first_play_exactly() {
    let dir = tempfile::tempdir().unwrap();
    let pcm = stereo(1.0, marker);
    let (packets, pre_skip) = packets(&pcm);
    for path in [
        encode(dir.path(), "a.opus", &pcm),
        matroska_with(dir.path(), "cues.mka", &packets, pre_skip, true),
        matroska(dir.path(), "a.mka", &packets, pre_skip),
    ] {
        let first = decode_all(&path);
        let mut d = FileDecoder::open(&path).unwrap();
        let mut out = Vec::new();
        while out.len() < 2 * RATE / 2 && d.next_block(&mut out).unwrap() {}
        d.seek(0.0).unwrap();
        let mut again = Vec::new();
        while d.next_block(&mut again).unwrap() {}
        let name = path.file_name().unwrap().to_string_lossy();
        assert_eq!(again.len(), first.len(), "{name}: length");
        assert!(again == first, "{name}: the replay differs");
    }
}

/// A Matroska file without cues seeks backwards too: it lands where a
/// fresh decoder seeking forward to the same place lands.
#[test]
fn a_matroska_file_without_cues_seeks_backwards() {
    let dir = tempfile::tempdir().unwrap();
    let pcm = stereo(1.0, marker);
    let (packets, pre_skip) = packets(&pcm);
    let path = matroska(dir.path(), "a.mka", &packets, pre_skip);
    let after_seek = |d: &mut FileDecoder| {
        d.seek(0.5).unwrap();
        let mut out = Vec::new();
        while d.next_block(&mut out).unwrap() {}
        out
    };
    let fresh = after_seek(&mut FileDecoder::open(&path).unwrap());
    let mut d = FileDecoder::open(&path).unwrap();
    let mut out = Vec::new();
    while out.len() < 2 * RATE * 8 / 10 && d.next_block(&mut out).unwrap() {}
    let back = after_seek(&mut d);
    assert_eq!(back.len(), fresh.len(), "length after the seek back");
    assert!(back == fresh, "the seek back differs");
}
