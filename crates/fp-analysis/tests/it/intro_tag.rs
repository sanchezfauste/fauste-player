#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Phase 2 spec P2.8: the INTRO tag sets the intro end.

use std::path::{Path, PathBuf};

use fp_analysis::analyze_file;
use fp_analysis::metadata::{parse_intro_time, read_intro};
use fp_model::{AnalysisSettings, Limits};
use lofty::config::WriteOptions;
use lofty::id3::v2::Id3v2Tag;
use lofty::tag::TagExt;

fn tone(dir: &Path, name: &str, secs: u32) -> PathBuf {
    let path = dir.join(name);
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 8_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(&path, spec).unwrap();
    for i in 0..8_000 * secs {
        w.write_sample(((i as f32 * 0.2).sin() * 12_000.0) as i16)
            .unwrap();
    }
    w.finalize().unwrap();
    path
}

fn tag_intro(path: &Path, value: &str) {
    let mut tag = Id3v2Tag::new();
    tag.insert_user_text("INTRO".into(), value.into());
    tag.save_to_path(path, WriteOptions::default()).unwrap();
}

#[test]
fn intro_times_parse_seconds_and_minutes() {
    assert_eq!(parse_intro_time("12.5"), Some(12.5));
    assert_eq!(parse_intro_time(" 7 "), Some(7.0));
    assert_eq!(parse_intro_time("1:02.5"), Some(62.5));
    assert_eq!(parse_intro_time("1:00:10"), Some(3610.0));
    for bad in ["", "abc", "-3", "1:75", "NaN", "inf", "1::2"] {
        assert_eq!(parse_intro_time(bad), None, "{bad}");
    }
}

#[test]
fn an_intro_tag_in_a_wav_file_sets_the_intro_end() {
    let dir = tempfile::tempdir().unwrap();
    let path = tone(dir.path(), "a.wav", 3);
    tag_intro(&path, "1.25");
    assert_eq!(read_intro(&path), Some(1.25));
    let analysis = analyze_file(&path, &AnalysisSettings::default(), &Limits::default()).unwrap();
    assert_eq!(analysis.analysis.intro_end, Some(1.25));
}

#[test]
fn an_intro_beyond_the_end_is_clamped() {
    let dir = tempfile::tempdir().unwrap();
    let path = tone(dir.path(), "a.wav", 2);
    tag_intro(&path, "0:30");
    let analysis = analyze_file(&path, &AnalysisSettings::default(), &Limits::default()).unwrap();
    let a = analysis.analysis;
    assert_eq!(a.intro_end, a.cue_out);
}

#[test]
fn a_malformed_intro_tag_is_ignored() {
    let dir = tempfile::tempdir().unwrap();
    let path = tone(dir.path(), "a.wav", 2);
    tag_intro(&path, "soon");
    let analysis = analyze_file(&path, &AnalysisSettings::default(), &Limits::default()).unwrap();
    assert_eq!(analysis.analysis.intro_end, None);
}

#[test]
fn files_without_tags_have_no_intro() {
    let dir = tempfile::tempdir().unwrap();
    let path = tone(dir.path(), "a.wav", 1);
    assert_eq!(read_intro(&path), None);
    assert_eq!(read_intro(Path::new("/definitely/missing.flac")), None);
}

#[test]
fn only_the_last_field_may_have_decimals_and_a_decimal_comma_is_accepted() {
    assert_eq!(parse_intro_time("1.5:30"), None);
    assert_eq!(parse_intro_time("12,5"), Some(12.5));
    assert_eq!(parse_intro_time("1:02,5"), Some(62.5));
}

fn wavpack(dir: &Path) -> PathBuf {
    let samples: Vec<i32> = (0..44_100)
        .map(|i| ((i as f32 * 0.1).sin() * 8_000.0) as i32)
        .collect();
    let params = wavicle::EncodeParams {
        channels: 1,
        sample_rate: 44_100,
        bits_per_sample: 16,
    };
    let path = dir.join("a.wv");
    std::fs::write(&path, wavicle::encode_int(params, &samples).unwrap()).unwrap();
    path
}

fn opus(dir: &Path) -> PathBuf {
    use opus_pure::{Application, MAX_PACKET_BYTES, OggOpusWriter, OpusEncoder, OpusHead};
    let mut encoder = OpusEncoder::new(48_000, 1, Application::Audio).unwrap();
    let mut writer =
        OggOpusWriter::new(Vec::new(), OpusHead::for_encoder(&encoder, 48_000)).unwrap();
    let pcm: Vec<f32> = (0..48_000).map(|i| (i as f32 * 0.1).sin() * 0.3).collect();
    let mut packet = vec![0u8; MAX_PACKET_BYTES];
    for block in pcm.chunks(960) {
        let n = encoder.encode(block, 960, &mut packet).unwrap();
        writer.write_packet(&packet[..n]).unwrap();
    }
    let path = dir.join("a.opus");
    std::fs::write(&path, writer.finish().unwrap()).unwrap();
    path
}

fn ape_item(path: &Path, value: &str) {
    let mut tag = lofty::ape::ApeTag::new();
    tag.insert(
        lofty::ape::ApeItem::new("INTRO".into(), lofty::tag::ItemValue::Text(value.into()))
            .unwrap(),
    );
    tag.save_to_path(path, WriteOptions::default()).unwrap();
}

#[test]
fn an_intro_tag_is_read_from_wavpack_and_monkeys_audio() {
    let dir = tempfile::tempdir().unwrap();
    let wv = wavpack(dir.path());
    ape_item(&wv, "0.5");
    assert_eq!(read_intro(&wv), Some(0.5));

    let ape = dir.path().join("a.ape");
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../fp-decode/tests/fixtures/ape/sine_24s_c2000.ape"),
        &ape,
    )
    .unwrap();
    ape_item(&ape, "0:00.75");
    assert_eq!(read_intro(&ape), Some(0.75));
}

#[test]
fn an_intro_tag_is_read_from_opus() {
    let dir = tempfile::tempdir().unwrap();
    let path = opus(dir.path());
    let mut comments = lofty::ogg::tag::VorbisComments::new();
    comments.insert("INTRO".into(), "0.4".into());
    comments
        .save_to_path(&path, WriteOptions::default())
        .unwrap();
    assert_eq!(read_intro(&path), Some(0.4));
}

#[test]
fn an_intro_tag_is_read_from_dsf() {
    let dir = tempfile::tempdir().unwrap();
    let mut tag = Id3v2Tag::new();
    tag.insert_user_text("INTRO".into(), "0.1".into());
    let mut id3 = Vec::new();
    tag.dump_to(&mut id3, WriteOptions::default()).unwrap();
    let data = vec![0x69u8; 2 * 4096];
    let data_end = 92 + data.len() as u64;
    let mut f = b"DSD ".to_vec();
    f.extend(28u64.to_le_bytes());
    f.extend((data_end + id3.len() as u64).to_le_bytes());
    f.extend(data_end.to_le_bytes());
    f.extend(b"fmt ");
    f.extend(52u64.to_le_bytes());
    for v in [1u32, 0, 2, 2, 2_822_400, 1] {
        f.extend(v.to_le_bytes());
    }
    f.extend((4096u64 * 8).to_le_bytes());
    f.extend(4096u32.to_le_bytes());
    f.extend(0u32.to_le_bytes());
    f.extend(b"data");
    f.extend((12 + data.len() as u64).to_le_bytes());
    f.extend(data);
    f.extend(id3);
    let path = dir.path().join("a.dsf");
    std::fs::write(&path, f).unwrap();
    assert_eq!(read_intro(&path), Some(0.1));
}

#[test]
fn the_id3_intro_description_is_matched_without_regard_to_case() {
    // Vorbis comments and APE items ignore case; so does TXXX here, as
    // tagging tools write "Intro" or "intro" as often as "INTRO".
    let dir = tempfile::tempdir().unwrap();
    for (n, key) in ["intro", "Intro"].into_iter().enumerate() {
        let path = tone(dir.path(), &format!("t{n}.wav"), 2);
        let mut tag = Id3v2Tag::new();
        tag.insert_user_text(key.into(), "1.25".into());
        tag.save_to_path(&path, WriteOptions::default()).unwrap();
        assert_eq!(read_intro(&path), Some(1.25), "{key}");
    }
}
