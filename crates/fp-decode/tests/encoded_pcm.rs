#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Encoded bitstreams stored as 16-bit stereo PCM are refused at open.

use std::path::Path;

use fp_decode::FileDecoder;

/// The header words of a DTS core frame of 4096 bytes in one stream form.
fn dts_head(packed14: bool, little: bool) -> Vec<u16> {
    let fsize = 4095u16;
    let be = [
        0x7FFEu16,
        0x8001,
        0xFC00 | (15 << 2) | (fsize >> 12),
        (fsize << 4) | 2,
    ];
    let mut out: Vec<u16> = if packed14 {
        let bits = be.iter().fold(0u128, |a, w| a << 16 | u128::from(*w)) << 6;
        (0..5)
            .map(|i| {
                let v = ((bits >> (70 - 14 * (i + 1))) & 0x3FFF) as u16;
                if v & 0x2000 != 0 { v | 0xC000 } else { v }
            })
            .collect()
    } else {
        be.to_vec()
    };
    if little {
        for w in &mut out {
            *w = w.swap_bytes();
        }
    }
    out
}

/// Words of a stereo payload with `sync` every `period` words over
/// pseudo-random filler.
fn payload(sync: &[u16], period: usize, count: usize) -> Vec<i16> {
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
        ("be16", dts_head(false, false)),
        ("le16", dts_head(false, true)),
        ("be14", dts_head(true, false)),
        ("le14", dts_head(true, true)),
    ] {
        let path = dir.path().join(format!("{name}.wav"));
        write_wav(&path, &payload(&sync, 2400, 12));
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
    write_wav(&path, &payload(&[0xF872, 0x4E1F, 0x0001, 24576], 3072, 8));
    let err = FileDecoder::open(&path).err().unwrap_or_default();
    assert!(err.contains("IEC 61937"), "{err:?}");
}

#[test]
fn a_dts_bitstream_in_an_aiff_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("dts.aiff");
    write_aiff(&path, &payload(&dts_head(false, false), 2048, 12));
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
    // The sniff did not consume anything: every frame is still there and
    // equal to the file's.
    assert_eq!(out.len(), words.len());
    for (got, want) in out.iter().zip(&words) {
        assert!((got - *want as f32 / 32768.0).abs() < 1e-6);
    }
}

#[test]
fn a_square_wave_that_looks_like_a_dts_sync_opens() {
    // L = R, +384 / -385, 100 frames per period: each falling-to-rising
    // edge reads FE7F 0180 at an exact period.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("square.wav");
    let words: Vec<i16> = (0..40_000)
        .flat_map(|n| {
            let s: i16 = if n % 100 < 50 { 384 } else { -385 };
            [s, s]
        })
        .collect();
    write_wav(&path, &words);
    assert!(FileDecoder::open(&path).is_ok());
}

#[test]
fn a_dts_bitstream_after_leading_silence_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("late.wav");
    // Two seconds of digital zeros, then the payload.
    let mut words = vec![0i16; 2 * 88_200];
    words.extend(payload(&dts_head(true, false), 2400, 12));
    write_wav(&path, &words);
    let err = FileDecoder::open(&path).err().unwrap_or_default();
    assert!(err.contains("DTS"), "{err:?}");
    // Only silence stays a (silent) file.
    write_wav(&path, &vec![0i16; 2 * 400_000]);
    assert!(FileDecoder::open(&path).is_ok());
}

#[test]
fn open_header_does_not_sniff() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("dts.wav");
    write_wav(&path, &payload(&dts_head(false, false), 2048, 12));
    assert!(FileDecoder::open(&path).is_err());
    assert!(FileDecoder::open_header(&path).is_ok());
}

#[test]
fn a_short_or_empty_wav_opens_without_panicking() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("short.wav");
    write_wav(&path, &[1, 2]);
    assert!(FileDecoder::open(&path).is_ok());
    // An empty file is either opened or refused with a message, never a panic.
    write_wav(&path, &[]);
    match FileDecoder::open(&path) {
        Ok(mut d) => {
            let mut out = Vec::new();
            assert!(!d.next_block(&mut out).unwrap_or(false) || out.is_empty());
        }
        Err(e) => assert!(!e.is_empty()),
    }
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
        // A file named "dts" may be a genuine DTS rip under another name than
        // the one asserted below, or one that is meant to be refused: only
        // the named rip is asserted, so skip the others.
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
