#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(clippy::indexing_slicing)]

//! Seeking in Matroska files whose blocks carry no duration, as muxers
//! write audio: 16-bit PCM files built here, so every frame can be
//! compared exactly.

#[path = "support/opus_files.rs"]
mod opus_files;

use fp_decode::FileDecoder;
use opus_files::{Block, Layout, RATE, ebml, matroska_tracks, pcm, pcm_matroska, pcm_track, uint};

fn rest(d: &mut FileDecoder) -> Vec<f32> {
    let mut out = Vec::new();
    while d.next_block(&mut out).unwrap() {}
    out
}

/// A seek past the end of a file written to a pipe (segment size unknown,
/// no cues) ends the stream, as for every other file, instead of failing.
/// With blocks that carry a duration, the reader's seek runs into the end
/// of the file and fails with it; without, the reader lands on the last
/// block, whose frames all come before the target.
#[test]
fn a_seek_past_the_end_of_a_piped_file_ends_the_stream() {
    let dir = tempfile::tempdir().unwrap();
    for default_duration in [false, true] {
        let layout = Layout {
            blocks_per_cluster: 10,
            cues: false,
            unknown_segment_size: true,
            default_duration,
        };
        let name = format!("pipe-{default_duration}.mka");
        let (path, pcm) = pcm_matroska(dir.path(), &name, 1.0, 4800, layout);
        let mut d = FileDecoder::open(&path).unwrap();
        assert!(rest(&mut d) == pcm, "{name}: the file decodes whole");
        for secs in [1.5, 30.0] {
            let mut d = FileDecoder::open(&path).unwrap();
            d.seek(secs).unwrap();
            assert!(rest(&mut d).is_empty(), "{name}: {secs} s");
        }
    }
}

/// A fresh seek lands on the target frame, with and without cues and in a
/// file written to a pipe: blocks with no duration are found by their
/// start (the block that holds the target, not the first one after it), and
/// a cue past the first cluster is followed into its own cluster, not the
/// next one.
#[test]
fn a_seek_lands_on_the_target_frame() {
    let dir = tempfile::tempdir().unwrap();
    for (name, cues, unknown_segment_size) in [
        ("cues.mka", true, false),
        ("bare.mka", false, false),
        ("pipe.mka", false, true),
    ] {
        let layout = Layout {
            blocks_per_cluster: 10,
            cues,
            unknown_segment_size,
            default_duration: false,
        };
        // 100 ms blocks, 1 s clusters.
        let (path, pcm) = pcm_matroska(dir.path(), name, 4.0, 4800, layout);
        for secs in [0.05, 0.37, 1.0, 1.55, 2.5, 3.95] {
            let mut d = FileDecoder::open(&path).unwrap();
            d.seek(secs).unwrap();
            let out = rest(&mut d);
            let at = (secs * f64::from(d.sample_rate())).round() as usize * 2;
            assert_eq!(out.len(), pcm.len() - at, "{name}: {secs} s: length");
            assert!(out[..] == pcm[at..], "{name}: {secs} s: the audio differs");
        }
    }
}

/// A seek into the middle of the block read last goes back into it: it
/// lands where a fresh decoder lands, not on the next block.
#[test]
fn a_seek_into_the_block_read_last_lands_on_the_target_frame() {
    let dir = tempfile::tempdir().unwrap();
    let layout = Layout::BARE;
    // 100 ms blocks.
    let (path, pcm) = pcm_matroska(dir.path(), "bare.mka", 1.0, 4800, layout);
    let mut d = FileDecoder::open(&path).unwrap();
    let mut out = Vec::new();
    for _ in 0..3 {
        assert!(d.next_block(&mut out).unwrap());
    }
    assert_eq!(out.len(), 3 * 4800 * 2, "three blocks read");
    d.seek(0.25).unwrap();
    let out = rest(&mut d);
    let at = 12_000 * 2;
    assert_eq!(out.len(), pcm.len() - at, "length");
    assert!(out[..] == pcm[at..], "the audio differs");
}

/// In a file with video, the cues and the clusters follow the video's
/// keyframes, and the audio block that holds a target can start before the
/// cued cluster: the seek still lands on it, not on the cluster's first
/// audio block.
#[test]
fn a_seek_through_cues_of_another_track_lands_on_the_target_frame() {
    let dir = tempfile::tempdir().unwrap();
    let block = 4800; // 100 ms
    let (bytes, samples) = pcm(4.0);
    let video = [0u8; 16];
    let mut blocks: Vec<Block> = bytes
        .chunks(block * 4)
        .enumerate()
        .map(|(i, data)| Block {
            track: 1,
            ts: (i * block * 1000 / RATE) as u64,
            data,
        })
        .collect();
    // 25 frames a second.
    blocks.extend((0..100).map(|i| Block {
        track: 2,
        ts: i * 40,
        data: &video,
    }));
    blocks.sort_by_key(|b| (b.ts, b.track));
    // Keyframes at 0, 1.04, 2.04 and 3.04 s: each starts a cluster and has a
    // cue; the audio block from 1.0 s lies in the cluster before.
    let keyframes: Vec<usize> = blocks
        .iter()
        .enumerate()
        .filter(|(_, b)| b.track == 2 && (b.ts == 0 || b.ts % 1000 == 40))
        .map(|(i, _)| i)
        .collect();
    let tracks = [
        [uint(0x83, 2), pcm_track(block, false)].concat(),
        [
            uint(0x83, 1),
            ebml(0x86, b"V_MPEG4/ISO/SP"),
            ebml(0xE0, &[uint(0xB0, 320), uint(0xBA, 240)].concat()),
        ]
        .concat(),
    ];
    let path = matroska_tracks(
        dir.path(),
        "av.mkv",
        &tracks,
        &blocks,
        &keyframes,
        &keyframes,
        false,
    );
    let mut d = FileDecoder::open(&path).unwrap();
    assert!(rest(&mut d) == samples, "the audio decodes whole");
    for secs in [0.5, 1.05, 2.01, 3.07] {
        let mut d = FileDecoder::open(&path).unwrap();
        d.seek(secs).unwrap();
        let out = rest(&mut d);
        let at = (secs * f64::from(d.sample_rate())).round() as usize * 2;
        assert_eq!(out.len(), samples.len() - at, "{secs} s: length");
        assert!(out[..] == samples[at..], "{secs} s: the audio differs");
    }
}

/// A seek into the last fraction of a millisecond of the block read last
/// still goes back into it: its end is not rounded down to the timestamp
/// before it. Blocks of 5015 frames (104.48 ms) at timestamps rounded to
/// whole milliseconds, as muxers write them: the second block starts at
/// 104 ms and the third at 209 ms.
#[test]
fn a_seek_into_the_end_of_the_block_read_last_lands_like_a_fresh_seek() {
    let dir = tempfile::tempdir().unwrap();
    let (path, _) = pcm_matroska(dir.path(), "bare.mka", 1.0, 5015, Layout::BARE);
    let secs = 0.2082;
    let mut fresh = FileDecoder::open(&path).unwrap();
    fresh.seek(secs).unwrap();
    let fresh = rest(&mut fresh);
    let mut d = FileDecoder::open(&path).unwrap();
    let mut out = Vec::new();
    for _ in 0..2 {
        assert!(d.next_block(&mut out).unwrap());
    }
    d.seek(secs).unwrap();
    let out = rest(&mut d);
    assert_eq!(out.len(), fresh.len(), "length");
    assert!(out == fresh, "the audio differs");
}
