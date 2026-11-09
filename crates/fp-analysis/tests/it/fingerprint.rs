#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! External editor spec EE8: the audio fingerprint tells a tag edit from an
//! audio edit.

use std::path::{Path, PathBuf};

use fp_analysis::analyze_file;
use fp_model::{AnalysisSettings, Limits};
use lofty::config::WriteOptions;
use lofty::prelude::*;
use lofty::tag::{Tag, TagType};

fn samples(len: usize) -> Vec<i16> {
    (0..len)
        .map(|i| ((i * 37 % 2000) as i16).wrapping_sub(1000))
        .collect()
}

fn wav(dir: &Path, name: &str, data: &[i16], title: Option<&str>) -> PathBuf {
    let path = dir.join(name);
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 8_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(&path, spec).unwrap();
    for s in data {
        w.write_sample(*s).unwrap();
    }
    w.finalize().unwrap();
    if let Some(title) = title {
        let mut file = lofty::read_from_path(&path).unwrap();
        file.insert_tag(Tag::new(TagType::RiffInfo));
        file.tag_mut(TagType::RiffInfo)
            .unwrap()
            .set_title(title.into());
        file.save_to_path(&path, WriteOptions::default()).unwrap();
    }
    path
}

fn hash(path: &Path) -> u64 {
    analyze_file(path, &AnalysisSettings::default(), &Limits::default())
        .unwrap()
        .analysis
        .audio_hash
        .expect("every successful analysis has a fingerprint")
}

#[test]
fn the_same_audio_with_other_tags_has_the_same_fingerprint() {
    let dir = tempfile::tempdir().unwrap();
    let data = samples(8_000);
    let a = wav(dir.path(), "a.wav", &data, Some("First title"));
    let b = wav(dir.path(), "b.wav", &data, Some("A different title"));
    assert_eq!(hash(&a), hash(&b));
}

#[test]
fn one_changed_sample_changes_the_fingerprint() {
    let dir = tempfile::tempdir().unwrap();
    let mut data = samples(8_000);
    let a = wav(dir.path(), "a.wav", &data, None);
    data[4_000] = data[4_000].wrapping_add(1);
    let b = wav(dir.path(), "b.wav", &data, None);
    assert_ne!(hash(&a), hash(&b));
}

#[test]
fn a_trimmed_file_changes_the_fingerprint() {
    let dir = tempfile::tempdir().unwrap();
    let data = samples(8_000);
    let a = wav(dir.path(), "a.wav", &data, None);
    let b = wav(dir.path(), "b.wav", &data[..7_000], None);
    assert_ne!(hash(&a), hash(&b));
}
