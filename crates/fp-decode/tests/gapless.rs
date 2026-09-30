#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Encoder delay and padding are removed once: a lossy file decodes to
//! exactly the frames its container declares.

use std::path::Path;

use fp_decode::FileDecoder;

#[test]
fn lossy_files_decode_to_their_declared_length() {
    for name in ["tone.ogg"] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/lossy")
            .join(name);
        let mut d = FileDecoder::open(&path).unwrap();
        let declared = d.frames_hint();
        let mut out = Vec::new();
        while d.next_block(&mut out).unwrap() {}
        assert_eq!(declared, Some(22_000), "{name} declares its length");
        assert_eq!(out.len() / 2, 22_000, "{name}");
    }
}

#[test]
fn a_vorbis_seek_lands_on_its_frame() {
    // What remains after a seek is exactly the rest of the file.
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/lossy/tone.ogg");
    for frame in [4_410u64, 11_025, 17_640] {
        let mut d = FileDecoder::open(&path).unwrap();
        d.seek(frame as f64 / 44_100.0).unwrap();
        let mut out = Vec::new();
        while d.next_block(&mut out).unwrap() {}
        let rest = (out.len() / 2) as u64;
        assert_eq!(rest, 22_000 - frame, "seek to frame {frame}");
    }
}
