#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Whole-file analysis and the per-file cache (spec §6).

use std::path::{Path, PathBuf};

use fp_analysis::cache::{ANALYSIS_VERSION, AnalysisCache};
use fp_analysis::{AnalysisError, analyze_file};
use fp_model::{AnalysisSettings, Limits};

const RATE: u32 = 8_000;

/// 1 s silence, `tone_secs` of a sine at −9 dBFS RMS, 1 s silence (mono WAV).
fn fixture(dir: &Path, name: &str, tone_secs: u32) -> PathBuf {
    let path = dir.join(name);
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(&path, spec).unwrap();
    let total = RATE * (tone_secs + 2);
    for i in 0..total {
        let in_tone = i >= RATE && i < RATE * (tone_secs + 1);
        let v = if in_tone {
            0.5 * ((i as f32) * 0.3).sin()
        } else {
            0.0
        };
        w.write_sample((v * f32::from(i16::MAX)) as i16).unwrap();
    }
    w.finalize().unwrap();
    path
}

fn settings() -> AnalysisSettings {
    AnalysisSettings::default()
}

#[test]
fn analyzing_a_wav_fixture_finds_duration_markers_and_peaks() {
    let dir = tempfile::tempdir().unwrap();
    let path = fixture(dir.path(), "Band - Song.wav", 70);
    let a = analyze_file(&path, &settings(), &Limits::default()).unwrap();
    let t = &a.analysis;
    assert!((t.duration_secs - 72.0).abs() < 1e-6);
    assert!((t.cue_in.unwrap() - 1.0).abs() < 0.06);
    assert!((t.cue_out.unwrap() - 71.0).abs() < 0.06);
    assert!(t.segue_start.is_some());
    assert_eq!(
        (t.title.as_deref(), t.artist.as_deref()),
        (Some("Song"), Some("Band"))
    );
    assert_eq!(a.peaks.len(), 7_200, "one 10 ms bucket per 10 ms");
    assert!((a.peak_bucket_secs - 0.01).abs() < 1e-9);
}

#[test]
fn a_missing_file_is_reported_as_missing() {
    let r = analyze_file(
        Path::new("/definitely/not/here.flac"),
        &settings(),
        &Limits::default(),
    );
    assert!(matches!(r, Err(AnalysisError::Missing)));
}

#[test]
fn a_non_audio_file_is_reported_as_unreadable() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.mp3");
    std::fs::write(&path, b"not audio at all").unwrap();
    assert!(matches!(
        analyze_file(&path, &settings(), &Limits::default()),
        Err(AnalysisError::Unreadable(_))
    ));
}

fn cache(dir: &Path) -> AnalysisCache {
    AnalysisCache::new(dir.join("cache"), &Limits::default())
}

#[test]
fn cache_round_trips() {
    let dir = tempfile::tempdir().unwrap();
    let path = fixture(dir.path(), "a.wav", 5);
    let a = analyze_file(&path, &settings(), &Limits::default()).unwrap();
    let c = cache(dir.path());
    assert!(c.load(&path, &settings()).is_none());
    c.store(&path, &settings(), &a).unwrap();
    assert_eq!(c.load(&path, &settings()), Some(a));
}

#[test]
fn a_modified_file_invalidates_the_cache() {
    let dir = tempfile::tempdir().unwrap();
    let path = fixture(dir.path(), "a.wav", 5);
    let a = analyze_file(&path, &settings(), &Limits::default()).unwrap();
    let c = cache(dir.path());
    c.store(&path, &settings(), &a).unwrap();
    fixture(dir.path(), "a.wav", 6); // re-recorded: different size
    assert!(c.load(&path, &settings()).is_none());
}

#[test]
fn changing_the_settings_invalidates_the_cache() {
    let dir = tempfile::tempdir().unwrap();
    let path = fixture(dir.path(), "a.wav", 5);
    let a = analyze_file(&path, &settings(), &Limits::default()).unwrap();
    let c = cache(dir.path());
    c.store(&path, &settings(), &a).unwrap();
    let changed = AnalysisSettings {
        segue_drop_db: 12.0,
        ..settings()
    };
    assert!(c.load(&path, &changed).is_none());
}

#[test]
fn a_corrupt_cache_entry_is_recomputed() {
    let dir = tempfile::tempdir().unwrap();
    let path = fixture(dir.path(), "a.wav", 5);
    let a = analyze_file(&path, &settings(), &Limits::default()).unwrap();
    let c = cache(dir.path());
    c.store(&path, &settings(), &a).unwrap();
    for entry in std::fs::read_dir(dir.path().join("cache")).unwrap() {
        std::fs::write(entry.unwrap().path(), b"garbage").unwrap();
    }
    assert!(c.load(&path, &settings()).is_none());
    assert_eq!(
        std::fs::read_dir(dir.path().join("cache")).unwrap().count(),
        0,
        "the bad entry is removed"
    );
}

#[test]
fn a_huge_embedded_cover_keeps_the_text_tags() {
    use lofty::config::WriteOptions;
    use lofty::picture::{MimeType, Picture, PictureType};
    use lofty::prelude::*;
    use lofty::tag::Tag;
    let dir = tempfile::tempdir().unwrap();
    let path = fixture(dir.path(), "file-name.wav", 2);
    let mut tagged = lofty::read_from_path(&path).unwrap();
    let ty = tagged.primary_tag_type();
    tagged.insert_tag(Tag::new(ty));
    let tag = tagged.primary_tag_mut().unwrap();
    tag.set_title("Song Title".into());
    // 17 MiB: under the 20 MiB cover limit, over lofty's own default allocation limit.
    tag.push_picture(
        Picture::unchecked(vec![0u8; 17 * 1024 * 1024])
            .pic_type(PictureType::CoverFront)
            .mime_type(MimeType::Png)
            .build(),
    );
    tagged.save_to_path(&path, WriteOptions::default()).unwrap();
    let a = analyze_file(&path, &settings(), &Limits::default()).unwrap();
    assert_eq!(a.analysis.title.as_deref(), Some("Song Title"));
}

#[cfg(unix)]
#[test]
fn a_file_without_read_permission_is_unreadable_not_missing() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let path = fixture(dir.path(), "locked.wav", 1);
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000)).unwrap();
    let r = analyze_file(&path, &settings(), &Limits::default());
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert!(matches!(r, Err(AnalysisError::Unreadable(_))), "{r:?}");
}

#[test]
fn the_analysis_records_the_rate_and_bits() {
    let dir = tempfile::tempdir().unwrap();
    let path = fixture(dir.path(), "format.wav", 1);
    let a = analyze_file(&path, &settings(), &Limits::default()).unwrap();
    assert_eq!(
        a.analysis.format,
        Some(fp_model::AudioFormat {
            sample_rate: RATE,
            bits: Some(16),
            channels: 1,
        })
    );
}

#[test]
fn entries_of_older_analysis_versions_are_pruned() {
    let dir = tempfile::tempdir().unwrap();
    let path = fixture(dir.path(), "a.wav", 5);
    let a = analyze_file(&path, &settings(), &Limits::default()).unwrap();
    cache(dir.path()).store(&path, &settings(), &a).unwrap();
    let cache_dir = dir.path().join("cache");
    let old = ANALYSIS_VERSION - 1;
    std::fs::write(cache_dir.join("0123456789abcdef.bin"), b"unversioned").unwrap();
    std::fs::write(
        cache_dir.join(format!("v{old}-0123456789abcdef.bin")),
        b"old",
    )
    .unwrap();
    let c = cache(dir.path());
    assert_eq!(
        std::fs::read_dir(&cache_dir).unwrap().count(),
        3,
        "opening is quick"
    );
    c.sweep();
    let names: Vec<String> = std::fs::read_dir(&cache_dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names.len(), 1, "{names:?}");
    assert!(
        names[0].starts_with(&format!("v{ANALYSIS_VERSION}-")),
        "{names:?}"
    );
    assert_eq!(
        c.load(&path, &settings()),
        Some(a),
        "the current entry stays"
    );
}
