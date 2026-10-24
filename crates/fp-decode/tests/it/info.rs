#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};

use fp_decode::{FileDecoder, duration_from_frames};

#[test]
fn the_decoder_reports_channels_and_the_frame_count_when_known() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mono.wav");
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 22_050,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(&path, spec).unwrap();
    for i in 0..22_050 {
        w.write_sample((i % 100) as i16).unwrap();
    }
    w.finalize().unwrap();
    let d = FileDecoder::open(&path).unwrap();
    assert_eq!(d.channels(), 1);
    assert_eq!(d.frames_hint(), Some(22_050));
    assert_eq!(d.sample_rate(), 22_050);
}

#[test]
fn seeking_past_the_end_is_the_end_for_every_decoder() {
    // Like WavPack, Monkey's Audio and DSD: no error, nothing more to play.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("short.wav");
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 48_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(&path, spec).unwrap();
    for i in 0..4_800 {
        w.write_sample((i % 100) as i16).unwrap();
        w.write_sample((i % 100) as i16).unwrap();
    }
    w.finalize().unwrap();
    let mut d = FileDecoder::open(&path).unwrap();
    d.seek(1_000.0).unwrap();
    let mut out = Vec::new();
    while d.next_block(&mut out).unwrap() {}
    assert!(out.is_empty());
    // And back to the start: everything again.
    d.seek(0.0).unwrap();
    while d.next_block(&mut out).unwrap() {}
    assert_eq!(out.len(), 2 * 4_800);
}

fn write(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, bytes).unwrap();
    path
}

/// A FLAC file of `samples` stereo 16-bit frames at 44.1 kHz with its
/// STREAMINFO block and one frame header, and no audio: the length is in
/// the header. symphonia reads up to the first frame header when it opens
/// the file, but it decodes nothing.
fn flac_header_only(samples: u64) -> Vec<u8> {
    let mut b = b"fLaC".to_vec();
    // Last metadata block, type 0 (STREAMINFO), 34 bytes.
    b.extend_from_slice(&[0x80, 0x00, 0x00, 34]);
    b.extend_from_slice(&4096u16.to_be_bytes()); // min block size
    b.extend_from_slice(&4096u16.to_be_bytes()); // max block size
    b.extend_from_slice(&[0, 0, 0, 0, 0, 0]); // min and max frame size: unknown
    // Sample rate (20 bits), channels − 1 (3), bits − 1 (5), samples (36).
    let packed: u64 = (44_100u64 << 44) | (1 << 41) | (15 << 36) | samples;
    b.extend_from_slice(&packed.to_be_bytes());
    b.extend_from_slice(&[0u8; 16]); // MD5: not computed
    // A frame header: fixed block size 4096, 44.1 kHz, stereo, 16 bits,
    // frame number 0, then its CRC-8 (polynomial 0x07).
    let header = [0xFF, 0xF8, 0xC9, 0x18, 0x00];
    let mut crc = 0u8;
    for byte in header {
        crc ^= byte;
        for _ in 0..8 {
            crc = if crc & 0x80 != 0 {
                crc << 1 ^ 0x07
            } else {
                crc << 1
            };
        }
    }
    b.extend_from_slice(&header);
    b.push(crc);
    b
}

/// The length of one MPEG-1 Layer III frame at 128 kbit/s and 44.1 kHz.
const MP3_FRAME: usize = 417;

/// `frames` silent mono MP3 frames (128 kbit/s, 44.1 kHz). With `info`,
/// the first one is an Info frame that declares the other frames.
fn mp3(frames: usize, info: bool) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(frames * MP3_FRAME);
    for k in 0..frames {
        let mut frame = vec![0u8; MP3_FRAME];
        frame[..4].copy_from_slice(&[0xFF, 0xFB, 0x90, 0xC0]);
        if info && k == 0 {
            // After the header and 17 bytes of mono side information:
            // "Info", flags (the frame count is present), the frame count.
            frame[21..25].copy_from_slice(b"Info");
            frame[25..29].copy_from_slice(&1u32.to_be_bytes());
            frame[29..33].copy_from_slice(&((frames - 1) as u32).to_be_bytes());
        }
        bytes.extend_from_slice(&frame);
    }
    bytes
}

#[test]
fn duration_from_frames_rejects_what_is_not_a_length() {
    assert_eq!(duration_from_frames(Some(44_100), 44_100), Some(1.0));
    assert_eq!(duration_from_frames(None, 44_100), None);
    assert_eq!(duration_from_frames(Some(0), 44_100), None);
    assert_eq!(duration_from_frames(Some(44_100), 0), None);
}

#[test]
fn a_wav_header_gives_its_length() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("two.wav");
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 48_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(&path, spec).unwrap();
    for i in 0..96_000 {
        w.write_sample((i % 100) as i16).unwrap();
        w.write_sample((i % 100) as i16).unwrap();
    }
    w.finalize().unwrap();
    let d = FileDecoder::open(&path).unwrap();
    assert_eq!(d.duration_hint_secs(), Some(2.0));
}

#[test]
fn a_flac_streaminfo_gives_its_length_without_decoding() {
    let dir = tempfile::tempdir().unwrap();
    let path = write(dir.path(), "two.flac", &flac_header_only(88_200));
    let d = FileDecoder::open(&path).unwrap();
    assert_eq!(d.duration_hint_secs(), Some(2.0));
}

#[test]
fn an_mpeg_stream_without_a_length_frame_has_no_header_duration() {
    let dir = tempfile::tempdir().unwrap();
    // 40 frames: symphonia could estimate a length from the bitrate, which
    // a VBR file can get wrong: no estimate is made, and none is used.
    let long = write(dir.path(), "vbr.mp3", &mp3(40, false));
    let d = FileDecoder::open(&long).unwrap();
    assert_eq!(d.frames_hint(), None);
    assert_eq!(d.duration_hint_secs(), None);
    // 8 frames: too short even to estimate.
    let short = write(dir.path(), "short.mp3", &mp3(8, false));
    assert_eq!(
        FileDecoder::open(&short).unwrap().duration_hint_secs(),
        None
    );
}

#[test]
fn an_mpeg_stream_with_an_info_frame_gives_its_length() {
    let dir = tempfile::tempdir().unwrap();
    let path = write(dir.path(), "cbr.mp3", &mp3(40, true));
    let secs = FileDecoder::open(&path)
        .unwrap()
        .duration_hint_secs()
        .unwrap();
    let expected = 39.0 * 1152.0 / 44_100.0;
    assert!((secs - expected).abs() < 0.05, "{secs} vs {expected}");
}

/// A raw AAC (ADTS) stream: `big` frames of 400 bytes, then `small` frames
/// of 40 bytes, each one 1024 samples of 44.1 kHz stereo AAC-LC. Nothing in
/// the stream stores its length, so a reader can only estimate it.
fn adts(big: usize, small: usize) -> Vec<u8> {
    let frame = |len: usize| {
        let mut f = vec![0u8; len];
        // Sync, MPEG-4, layer 0, no CRC; AAC-LC, 44.1 kHz (index 4),
        // 2 channels; the 13-bit frame length; buffer fullness 0x7FF; one
        // raw data block.
        f[0] = 0xFF;
        f[1] = 0xF1;
        f[2] = (1 << 6) | (4 << 2);
        f[3] = (2 << 6) | ((len >> 11) & 0x3) as u8;
        f[4] = ((len >> 3) & 0xFF) as u8;
        f[5] = (((len & 0x7) << 5) as u8) | 0x1F;
        f[6] = 0xFC;
        f
    };
    let mut bytes = Vec::new();
    for _ in 0..big {
        bytes.extend_from_slice(&frame(400));
    }
    for _ in 0..small {
        bytes.extend_from_slice(&frame(40));
    }
    bytes
}

#[test]
fn a_raw_aac_stream_has_no_header_duration() {
    let dir = tempfile::tempdir().unwrap();
    // 400 frames, but the first quarter is ten times larger than the rest:
    // a length from sampled frame sizes is far too short.
    let path = write(dir.path(), "mix.aac", &adts(100, 300));
    let d = FileDecoder::open(&path).unwrap();
    assert!(
        d.frames_hint().is_some_and(|f| f < 400 * 1024),
        "symphonia's estimate is short: {:?}",
        d.frames_hint()
    );
    assert_eq!(d.duration_hint_secs(), None);
}

#[test]
fn a_length_tag_away_from_its_place_in_the_first_frame_is_not_trusted() {
    let dir = tempfile::tempdir().unwrap();
    let mut bytes = mp3(40, false);
    // "Xing" in the first frame's audio data, not after its side
    // information: not a length frame (symphonia estimates the length).
    bytes[200..204].copy_from_slice(b"Xing");
    let path = write(dir.path(), "stray.mp3", &bytes);
    assert_eq!(FileDecoder::open(&path).unwrap().duration_hint_secs(), None);
}

#[test]
fn a_length_frame_after_an_id3_tag_gives_its_length() {
    let dir = tempfile::tempdir().unwrap();
    // A 20-byte ID3v2.4 tag of padding, then the stream.
    let mut bytes = b"ID3\x04\x00\x00\x00\x00\x00\x14".to_vec();
    bytes.extend_from_slice(&[0u8; 20]);
    bytes.extend_from_slice(&mp3(40, true));
    let path = write(dir.path(), "tagged.mp3", &bytes);
    let secs = FileDecoder::open(&path)
        .unwrap()
        .duration_hint_secs()
        .unwrap();
    let expected = 39.0 * 1152.0 / 44_100.0;
    assert!((secs - expected).abs() < 0.05, "{secs} vs {expected}");
}

/// A VBR MP3 stream without a length frame: `big` silent mono frames at
/// 320 kbit/s, then `small` at 32 kbit/s (MPEG-1 Layer III, 44.1 kHz). A
/// length estimated from the first frames' sizes is far too short.
fn vbr_mp3(big: usize, small: usize) -> Vec<u8> {
    // (bitrate index, frame length): 144 * bitrate / 44_100, rounded down.
    let frame = |index: u8, len: usize| {
        let mut f = vec![0u8; len];
        f[..4].copy_from_slice(&[0xFF, 0xFB, index << 4, 0xC0]);
        f
    };
    let mut bytes = Vec::new();
    for _ in 0..big {
        bytes.extend_from_slice(&frame(14, 1044));
    }
    for _ in 0..small {
        bytes.extend_from_slice(&frame(1, 104));
    }
    bytes
}

/// Frames decoded from `d` until the end of the stream.
fn decoded_frames(d: &mut FileDecoder) -> usize {
    let mut out = Vec::new();
    while d.next_block(&mut out).unwrap() {}
    out.len() / 2
}

#[test]
fn a_vbr_mpeg_stream_without_a_length_frame_decodes_to_its_end() {
    let dir = tempfile::tempdir().unwrap();
    // 420 frames, but a length from the first frames' bitrate is ~60.
    let path = write(dir.path(), "vbr.mp3", &vbr_mp3(20, 400));
    let mut d = FileDecoder::open(&path).unwrap();
    assert_eq!(decoded_frames(&mut d), 420 * 1152);
}

#[test]
fn a_vbr_mpeg_stream_without_a_length_frame_seeks_past_the_estimate() {
    let dir = tempfile::tempdir().unwrap();
    let path = write(dir.path(), "vbr.mp3", &vbr_mp3(20, 400));
    let mut d = FileDecoder::open(&path).unwrap();
    // Frame 300, well past the estimated end.
    d.seek(300.0 * 1152.0 / 44_100.0).unwrap();
    assert_eq!(decoded_frames(&mut d), 120 * 1152);
}
