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
