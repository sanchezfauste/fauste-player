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
