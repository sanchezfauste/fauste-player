#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Encoded bitstreams stored as 16-bit stereo PCM are refused at open.

use std::path::Path;

use fp_decode::FileDecoder;

/// Words of a stereo payload with `sync` every `period` words over
/// pseudo-random filler.
fn payload(sync: [u16; 3], period: usize, count: usize) -> Vec<i16> {
    let mut x = 99u32;
    let mut v: Vec<i16> = (0..period * count + 64)
        .map(|_| {
            x = x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            ((x >> 16) as i16) / 4
        })
        .collect();
    for n in 0..count {
        for (k, w) in sync.iter().enumerate() {
            v[20 + n * period + k] = *w as i16;
        }
    }
    v
}

fn write_wav(path: &Path, words: &[i16]) {
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 44_100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(path, spec).unwrap();
    for s in words {
        w.write_sample(*s).unwrap();
    }
    w.finalize().unwrap();
}

/// A big-endian 16-bit stereo 44.1 kHz AIFF.
fn write_aiff(path: &Path, words: &[i16]) {
    let frames = (words.len() / 2) as u32;
    let data: Vec<u8> = words.iter().flat_map(|s| s.to_be_bytes()).collect();
    let mut comm = Vec::new();
    comm.extend_from_slice(&2u16.to_be_bytes());
    comm.extend_from_slice(&frames.to_be_bytes());
    comm.extend_from_slice(&16u16.to_be_bytes());
    comm.extend_from_slice(&[0x40, 0x0E, 0xAC, 0x44, 0, 0, 0, 0, 0, 0]);
    let mut ssnd = vec![0u8; 8];
    ssnd.extend_from_slice(&data);
    let mut body = b"AIFF".to_vec();
    body.extend_from_slice(b"COMM");
    body.extend_from_slice(&(comm.len() as u32).to_be_bytes());
    body.extend_from_slice(&comm);
    body.extend_from_slice(b"SSND");
    body.extend_from_slice(&(ssnd.len() as u32).to_be_bytes());
    body.extend_from_slice(&ssnd);
    let mut file = b"FORM".to_vec();
    file.extend_from_slice(&(body.len() as u32).to_be_bytes());
    file.extend_from_slice(&body);
    std::fs::write(path, file).unwrap();
}

#[test]
fn a_dts_bitstream_in_a_wav_is_refused_with_a_reason() {
    let dir = tempfile::tempdir().unwrap();
    for (name, sync) in [
        ("be16", [0x7FFE, 0x8001, 0xFC00]),
        ("le16", [0xFE7F, 0x0180, 0x00FC]),
        ("be14", [0x1FFF, 0xE800, 0x07F1]),
        ("le14", [0xFF1F, 0x00E8, 0xF107]),
    ] {
        let path = dir.path().join(format!("{name}.wav"));
        write_wav(&path, &payload(sync, 2048, 12));
        let err = FileDecoder::open(&path).err().unwrap_or_default();
        assert!(
            err.contains("encoded bitstream (DTS) stored as PCM"),
            "{name}: {err:?}"
        );
    }
}

#[test]
fn an_iec_61937_burst_stream_in_a_wav_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("iec.wav");
    write_wav(&path, &payload([0xF872, 0x4E1F, 0x0001], 3072, 8));
    let err = FileDecoder::open(&path).err().unwrap_or_default();
    assert!(err.contains("IEC 61937"), "{err:?}");
}

#[test]
fn a_dts_bitstream_in_an_aiff_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("dts.aiff");
    write_aiff(&path, &payload([0x7FFE, 0x8001, 0xFC00], 2048, 12));
    let err = FileDecoder::open(&path).err().unwrap_or_default();
    assert!(err.contains("DTS"), "{err:?}");
}

#[test]
fn ordinary_pcm_still_opens_and_plays_from_its_start() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sine.wav");
    let words: Vec<i16> = (0..100_000)
        .map(|i| (((i / 2) as f32 * 0.05).sin() * 20_000.0) as i16)
        .collect();
    write_wav(&path, &words);
    let mut d = FileDecoder::open(&path).unwrap();
    let mut out = Vec::new();
    while d.next_block(&mut out).unwrap() {}
    // The sniff did not consume anything: every frame is still there.
    assert_eq!(out.len(), words.len());
    assert!((out[2] - words[2] as f32 / 32768.0).abs() < 1e-6);
}

#[test]
fn a_short_or_empty_wav_opens_without_panicking() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("short.wav");
    write_wav(&path, &[1, 2]);
    assert!(FileDecoder::open(&path).is_ok());
    write_wav(&path, &[]);
    let _ = FileDecoder::open(&path);
}

/// Local only: `test-music/` (or `FAUSTE_TEST_MUSIC`) holds a DTS-CD rip as
/// WAV, which must be refused while every other file there still opens.
#[test]
#[ignore = "needs local test music"]
fn real_music_dts_is_refused_and_the_rest_opens() {
    let dir = std::env::var_os("FAUSTE_TEST_MUSIC").map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test-music"),
        Into::into,
    );
    let Ok(entries) = std::fs::read_dir(&dir) else {
        eprintln!("no test music at {}", dir.display());
        return;
    };
    for e in entries.flatten() {
        let path = e.path();
        let name = path.file_name().unwrap().to_string_lossy().to_lowercase();
        if name.ends_with(".md") || name.contains("dts") {
            continue;
        }
        let r = FileDecoder::open(&path);
        eprintln!("{name}: {:?}", r.as_ref().err());
        if name.contains("private investigations.wav") {
            assert!(r.is_err(), "the DTS rip must be refused");
        } else {
            assert!(r.is_ok(), "{name} must open");
        }
    }
}
