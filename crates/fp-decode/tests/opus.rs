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

/// Encodes `pcm` into Opus packets, with the encoder's pre-skip.
fn packets(pcm: &[f32]) -> (Vec<Vec<u8>>, u16) {
    let mut encoder = OpusEncoder::new(48_000, 2, Application::Audio).unwrap();
    let pre_skip = OpusHead::for_encoder(&encoder, 48_000).pre_skip;
    let mut packet = vec![0u8; MAX_PACKET_BYTES];
    let packets = pcm
        .as_chunks::<{ FRAME * 2 }>()
        .0
        .iter()
        .map(|block| {
            let n = encoder.encode(block, FRAME, &mut packet).unwrap();
            packet[..n].to_vec()
        })
        .collect();
    (packets, pre_skip)
}

/// One EBML element with an 8-byte size.
fn ebml(id: u32, body: &[u8]) -> Vec<u8> {
    let mut out: Vec<u8> = id
        .to_be_bytes()
        .into_iter()
        .skip_while(|b| *b == 0)
        .collect();
    out.push(0x01);
    out.extend_from_slice(&(body.len() as u64).to_be_bytes()[1..]);
    out.extend_from_slice(body);
    out
}

fn uint(id: u32, v: u64) -> Vec<u8> {
    ebml(id, &v.to_be_bytes())
}

/// A minimal Matroska file holding `packets` of stereo Opus, 20 ms each,
/// with the pre-skip as `CodecDelay`, the way muxers write it.
fn matroska(dir: &Path, name: &str, packets: &[Vec<u8>], pre_skip: u16) -> PathBuf {
    let mut head = b"OpusHead".to_vec();
    head.extend([1, 2]);
    head.extend(pre_skip.to_le_bytes());
    head.extend(48_000u32.to_le_bytes());
    head.extend([0, 0, 0]); // gain 0, mapping family 0
    let header = [
        uint(0x4286, 1),
        uint(0x42F7, 1),
        uint(0x42F2, 4),
        uint(0x42F3, 8),
        ebml(0x4282, b"matroska"),
        uint(0x4287, 4),
        uint(0x4285, 2),
    ]
    .concat();
    let info = ebml(
        0x1549_A966,
        &[
            uint(0x2A_D7B1, 1_000_000), // 1 ms ticks
            ebml(0x4D80, b"test"),
            ebml(0x5741, b"test"),
        ]
        .concat(),
    );
    let audio = ebml(
        0xE1,
        &[ebml(0xB5, &48_000f64.to_be_bytes()), uint(0x9F, 2)].concat(),
    );
    let track = ebml(
        0xAE,
        &[
            uint(0xD7, 1),
            uint(0x73C5, 1),
            uint(0x83, 2),
            ebml(0x86, b"A_OPUS"),
            ebml(0x63A2, &head),
            uint(0x56AA, u64::from(pre_skip) * 1_000_000_000 / 48_000),
            uint(0x56BB, 80_000_000),
            audio,
        ]
        .concat(),
    );
    let tracks = ebml(0x1654_AE6B, &track);
    let mut cluster = uint(0xE7, 0);
    for (i, packet) in packets.iter().enumerate() {
        let mut block = vec![0x81];
        block.extend(((i * 20) as i16).to_be_bytes());
        block.push(0x80);
        block.extend_from_slice(packet);
        cluster.extend(ebml(0xA3, &block));
    }
    let segment = ebml(
        0x1853_8067,
        &[info, tracks, ebml(0x1F43_B675, &cluster)].concat(),
    );
    let path = dir.join(name);
    std::fs::write(&path, [ebml(0x1A45_DFA3, &header), segment].concat()).unwrap();
    path
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
