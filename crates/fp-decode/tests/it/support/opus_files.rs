//! Files built for the tests: Ogg Opus through a pure-Rust encoder, and
//! minimal Matroska files around its packets or around 16-bit PCM. Shared
//! by `tests/it/opus.rs` and the unit tests of `symph.rs`.

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
    let track = [
        ebml(0x86, b"A_OPUS"),
        ebml(0x63A2, &head),
        uint(0x56AA, u64::from(pre_skip) * 1_000_000_000 / 48_000),
        uint(0x56BB, 80_000_000),
        audio(48_000.0, 2, None),
    ]
    .concat();
    let blocks: Vec<(u64, &[u8])> = packets
        .iter()
        .enumerate()
        .map(|(i, p)| (i as u64 * 20, p.as_slice()))
        .collect();
    let layout = Layout {
        blocks_per_cluster: usize::MAX,
        cues,
        unknown_segment_size: false,
        default_duration: false,
    };
    matroska_file(dir, name, &track, &blocks, layout)
}

/// How a test Matroska file is laid out.
#[derive(Debug, Clone, Copy)]
pub struct Layout {
    /// Blocks per cluster.
    pub blocks_per_cluster: usize,
    /// A cue point per cluster, with the position of its first block
    /// (`CueRelativePosition`), as muxers write a finished file.
    pub cues: bool,
    /// The segment's size left unknown, as a muxer writing to a pipe
    /// leaves it.
    pub unknown_segment_size: bool,
    /// The track's `DefaultDuration` (each block's), which audio muxers
    /// usually leave out.
    pub default_duration: bool,
}

impl Layout {
    /// 10 blocks per cluster, no cues, sizes known, no durations.
    pub const BARE: Self = Self {
        blocks_per_cluster: 10,
        cues: false,
        unknown_segment_size: false,
        default_duration: false,
    };
}

/// The `Audio` element of a track.
pub fn audio(rate: f64, channels: u64, bit_depth: Option<u64>) -> Vec<u8> {
    let mut body = [ebml(0xB5, &rate.to_be_bytes()), uint(0x9F, channels)].concat();
    if let Some(bits) = bit_depth {
        body.extend(uint(0x6264, bits));
    }
    ebml(0xE1, &body)
}

/// One block of a test Matroska file.
#[derive(Debug, Clone, Copy)]
pub struct Block<'a> {
    /// The track's number (1 for the first track).
    pub track: u64,
    /// Milliseconds.
    pub ts: u64,
    pub data: &'a [u8],
}

/// A Matroska file with one audio track (track number 1, 1 ms ticks) whose
/// `TrackEntry` holds `track` after its number, UID and type, and whose
/// blocks are `blocks` (timestamp in ms, frame), with no `BlockDuration`
/// (and no `DefaultDuration` unless `track` has one), as muxers write audio.
pub fn matroska_file(
    dir: &Path,
    name: &str,
    track: &[u8],
    blocks: &[(u64, &[u8])],
    layout: Layout,
) -> PathBuf {
    let entry = [uint(0x83, 2), track.to_vec()].concat();
    let blocks: Vec<Block> = blocks
        .iter()
        .map(|&(ts, data)| Block { track: 1, ts, data })
        .collect();
    let per = layout.blocks_per_cluster.max(1);
    let clusters: Vec<usize> = (0..blocks.len()).step_by(per).collect();
    let cues = if layout.cues {
        clusters.clone()
    } else {
        Vec::new()
    };
    matroska_tracks(
        dir,
        name,
        &[entry],
        &blocks,
        &clusters,
        &cues,
        layout.unknown_segment_size,
    )
}

/// A Matroska file (1 ms ticks) with the tracks whose `TrackEntry` holds
/// each of `tracks` (type first) after its number and UID (`i + 1`), the blocks
/// `blocks` in file order, a cluster starting at each block of `clusters`
/// (indices into `blocks`, the first being 0) and a cue point for each block
/// of `cues`, with its cluster's position and its own within it.
pub fn matroska_tracks(
    dir: &Path,
    name: &str,
    tracks: &[Vec<u8>],
    blocks: &[Block],
    clusters: &[usize],
    cues: &[usize],
    unknown_segment_size: bool,
) -> PathBuf {
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
    let entries: Vec<u8> = tracks
        .iter()
        .enumerate()
        .flat_map(|(i, t)| {
            let n = i as u64 + 1;
            ebml(0xAE, &[uint(0xD7, n), uint(0x73C5, n), t.clone()].concat())
        })
        .collect();
    let tracks = ebml(0x1654_AE6B, &entries);
    // Each cluster, and where each block lies: its cluster and its offset in
    // the cluster's data.
    let mut bodies: Vec<Vec<u8>> = Vec::new();
    let mut at = vec![(0usize, 0usize); blocks.len()];
    let mut start = 0;
    for (i, b) in blocks.iter().enumerate() {
        if clusters.contains(&i) || bodies.is_empty() {
            start = b.ts;
            bodies.push(uint(0xE7, start));
        }
        let cluster = bodies.len() - 1;
        let body = bodies.last_mut().unwrap();
        at[i] = (cluster, body.len());
        let mut block = vec![0x80 | b.track as u8];
        block.extend(((b.ts - start) as i16).to_be_bytes());
        block.push(0x80);
        block.extend_from_slice(b.data);
        body.extend(ebml(0xA3, &block));
    }
    let clusters: Vec<Vec<u8>> = bodies.iter().map(|b| ebml(0x1F43_B675, b)).collect();
    // Cue positions count from the segment's data. Every number is written
    // with 8 bytes, so the cues' length does not depend on them.
    let cues_at = |offset: usize| {
        let points: Vec<u8> = cues
            .iter()
            .flat_map(|&i| {
                let (cluster, rel) = at[i];
                let pos = offset + clusters[..cluster].iter().map(Vec::len).sum::<usize>();
                ebml(
                    0xBB,
                    &[
                        uint(0xB3, blocks[i].ts),
                        ebml(
                            0xB7,
                            &[
                                uint(0xF7, blocks[i].track),
                                uint(0xF1, pos as u64),
                                uint(0xF0, rel as u64),
                            ]
                            .concat(),
                        ),
                    ]
                    .concat(),
                )
            })
            .collect();
        ebml(0x1C53_BB6B, &points)
    };
    let cues = if cues.is_empty() {
        Vec::new()
    } else {
        let before = info.len() + tracks.len();
        cues_at(before + cues_at(0).len())
    };
    let mut body = [info, tracks, cues].concat();
    for cluster in &clusters {
        body.extend_from_slice(cluster);
    }
    let segment = if unknown_segment_size {
        [
            &[
                0x18, 0x53, 0x80, 0x67, 0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            ][..],
            &body,
        ]
        .concat()
    } else {
        ebml(0x1853_8067, &body)
    };
    let path = dir.join(name);
    std::fs::write(&path, [ebml(0x1A45_DFA3, &header), segment].concat()).unwrap();
    path
}

/// `secs` of the test signal as 16-bit stereo PCM, little-endian, and as
/// `f32`, the way the decoder yields it.
pub fn pcm(secs: f64) -> (Vec<u8>, Vec<f32>) {
    let pcm: Vec<i16> = stereo(secs, marker)
        .iter()
        .map(|v| (v * 32_767.0).round() as i16)
        .collect();
    let bytes = pcm.iter().flat_map(|s| s.to_le_bytes()).collect();
    let samples = pcm.iter().map(|s| f32::from(*s) / 32_768.0).collect();
    (bytes, samples)
}

/// The `TrackEntry` body (after number, UID and type) of a 16-bit stereo
/// PCM track at `RATE`, with a `DefaultDuration` of `block` frames if
/// `default_duration`.
pub fn pcm_track(block: usize, default_duration: bool) -> Vec<u8> {
    let mut track = [
        ebml(0x86, b"A_PCM/INT/LIT"),
        audio(RATE as f64, 2, Some(16)),
    ]
    .concat();
    if default_duration {
        track.extend(uint(0x23_E383, (block * 1_000_000_000 / RATE) as u64));
    }
    track
}

/// A Matroska file of 16-bit stereo PCM (`A_PCM/INT/LIT`) at `RATE`, in
/// blocks of `block` frames, and the samples it holds as `f32`, the way the
/// decoder yields them. Block timestamps are rounded to whole milliseconds,
/// as muxers write them, so they are exact only for blocks of whole
/// milliseconds.
pub fn pcm_matroska(
    dir: &Path,
    name: &str,
    secs: f64,
    block: usize,
    layout: Layout,
) -> (PathBuf, Vec<f32>) {
    let (bytes, samples) = pcm(secs);
    let frames: Vec<&[u8]> = bytes.chunks(block * 4).collect();
    let blocks: Vec<(u64, &[u8])> = frames
        .iter()
        .enumerate()
        .map(|(i, f)| (((i * block * 1000 + RATE / 2) / RATE) as u64, *f))
        .collect();
    let track = pcm_track(block, layout.default_duration);
    let path = matroska_file(dir, name, &track, &blocks, layout);
    (path, samples)
}
