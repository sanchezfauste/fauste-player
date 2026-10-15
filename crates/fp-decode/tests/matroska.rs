#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(clippy::indexing_slicing)]

//! Seeking in Matroska files whose blocks carry no duration, as muxers
//! write audio: 16-bit PCM files built here, so every frame can be
//! compared exactly.

#[path = "support/opus_files.rs"]
mod opus_files;

use fp_decode::FileDecoder;
use opus_files::{Layout, pcm_matroska};

fn rest(d: &mut FileDecoder) -> Vec<f32> {
    let mut out = Vec::new();
    while d.next_block(&mut out).unwrap() {}
    out
}

/// A seek past the end of a file written to a pipe (segment size unknown,
/// no cues) ends the stream, as for every other file, instead of failing.
#[test]
fn a_seek_past_the_end_of_a_piped_file_ends_the_stream() {
    let dir = tempfile::tempdir().unwrap();
    let layout = Layout {
        blocks_per_cluster: 10,
        cues: false,
        unknown_segment_size: true,
    };
    let (path, pcm) = pcm_matroska(dir.path(), "pipe.mka", 1.0, 4800, layout);
    let mut d = FileDecoder::open(&path).unwrap();
    assert!(rest(&mut d) == pcm, "the file decodes whole");
    for secs in [1.5, 30.0] {
        let mut d = FileDecoder::open(&path).unwrap();
        d.seek(secs).unwrap();
        assert!(rest(&mut d).is_empty(), "{secs} s");
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
