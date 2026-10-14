//! Opus files built for the tests: Ogg Opus through a pure-Rust encoder,
//! and minimal Matroska files around its packets. Shared by `tests/opus.rs`
//! and the unit tests of `symph.rs`.

#![allow(dead_code, clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(clippy::indexing_slicing)]

use std::f64::consts::PI;
use std::path::{Path, PathBuf};

use opus_pure::{Application, MAX_PACKET_BYTES, OggOpusWriter, OpusEncoder, OpusHead};

pub const RATE: usize = 48_000;
pub const FRAME: usize = 960;

/// Encodes interleaved stereo `pcm` at 48 kHz as an Ogg Opus file.
pub fn encode(dir: &Path, name: &str, pcm: &[f32]) -> PathBuf {
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

pub fn stereo(secs: f64, f: impl Fn(f64) -> f64) -> Vec<f32> {
    (0..(secs * RATE as f64) as usize)
        .flat_map(|i| {
            let v = f(i as f64 / RATE as f64) as f32;
            [v, v]
        })
        .collect()
}

/// A signal that never repeats within the file: three unrelated tones.
pub fn marker(t: f64) -> f64 {
    0.2 * (2.0 * PI * 441.0 * t).sin()
        + 0.15 * (2.0 * PI * 1234.5 * t + 1.0).sin()
        + 0.1 * (2.0 * PI * 3210.7 * t + 2.0).sin()
}

/// Encodes `pcm` into Opus packets, with the encoder's pre-skip.
pub fn packets(pcm: &[f32]) -> (Vec<Vec<u8>>, u16) {
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
pub fn ebml(id: u32, body: &[u8]) -> Vec<u8> {
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

pub fn uint(id: u32, v: u64) -> Vec<u8> {
    ebml(id, &v.to_be_bytes())
}

/// A minimal Matroska file holding `packets` of stereo Opus, 20 ms each,
/// with the pre-skip as `CodecDelay`, the way muxers write it, and no
/// cues (as a live recording leaves it).
pub fn matroska(dir: &Path, name: &str, packets: &[Vec<u8>], pre_skip: u16) -> PathBuf {
    matroska_with(dir, name, packets, pre_skip, false)
}

/// `matroska`, with a cue point at the start of its one cluster when
/// `cues` is set, as muxers write a finished file.
pub fn matroska_with(
    dir: &Path,
    name: &str,
    packets: &[Vec<u8>],
    pre_skip: u16,
    cues: bool,
) -> PathBuf {
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
    // The cue's cluster position counts from the segment's data.
    let cue = |at: usize| {
        ebml(
            0x1C53_BB6B,
            &ebml(
                0xBB,
                &[
                    uint(0xB3, 0),
                    ebml(0xB7, &[uint(0xF7, 1), uint(0xF1, at as u64)].concat()),
                ]
                .concat(),
            ),
        )
    };
    let cues = if cues {
        let at = info.len() + tracks.len() + cue(0).len();
        cue(at)
    } else {
        Vec::new()
    };
    let segment = ebml(
        0x1853_8067,
        &[info, tracks, cues, ebml(0x1F43_B675, &cluster)].concat(),
    );
    let path = dir.join(name);
    std::fs::write(&path, [ebml(0x1A45_DFA3, &header), segment].concat()).unwrap();
    path
}
