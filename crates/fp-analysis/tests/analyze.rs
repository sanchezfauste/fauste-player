#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Whole-file analysis and the per-file cache (spec §6).

use std::path::{Path, PathBuf};

use fp_analysis::cache::{ANALYSIS_VERSION, AnalysisCache};
use fp_analysis::{AnalysisError, analyze_file, measure_loudness_cancellable};
use fp_model::{AnalysisSettings, LOUDNESS_VERSION, Limits, LoudnessMeasurement};

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
    AnalysisCache::new(dir.join("cache"))
}

#[test]
fn cache_round_trips() {
    let dir = tempfile::tempdir().unwrap();
    let path = fixture(dir.path(), "a.wav", 5);
    let a = analyze_file(&path, &settings(), &Limits::default()).unwrap();
    let c = cache(dir.path());
    assert!(c.load(&path, &settings(), &Limits::default()).is_none());
    c.store(&path, &settings(), &Limits::default(), &a).unwrap();
    assert_eq!(c.load(&path, &settings(), &Limits::default()), Some(a));
}

#[test]
fn a_modified_file_invalidates_the_cache() {
    let dir = tempfile::tempdir().unwrap();
    let path = fixture(dir.path(), "a.wav", 5);
    let a = analyze_file(&path, &settings(), &Limits::default()).unwrap();
    let c = cache(dir.path());
    c.store(&path, &settings(), &Limits::default(), &a).unwrap();
    fixture(dir.path(), "a.wav", 6); // re-recorded: different size
    assert!(c.load(&path, &settings(), &Limits::default()).is_none());
}

#[test]
fn changing_the_settings_invalidates_the_cache() {
    let dir = tempfile::tempdir().unwrap();
    let path = fixture(dir.path(), "a.wav", 5);
    let a = analyze_file(&path, &settings(), &Limits::default()).unwrap();
    let c = cache(dir.path());
    c.store(&path, &settings(), &Limits::default(), &a).unwrap();
    let changed = AnalysisSettings {
        segue_drop_db: 12.0,
        ..settings()
    };
    assert!(c.load(&path, &changed, &Limits::default()).is_none());
}

#[test]
fn a_corrupt_cache_entry_is_recomputed() {
    let dir = tempfile::tempdir().unwrap();
    let path = fixture(dir.path(), "a.wav", 5);
    let a = analyze_file(&path, &settings(), &Limits::default()).unwrap();
    let c = cache(dir.path());
    c.store(&path, &settings(), &Limits::default(), &a).unwrap();
    for entry in std::fs::read_dir(dir.path().join("cache")).unwrap() {
        std::fs::write(entry.unwrap().path(), b"garbage").unwrap();
    }
    assert!(c.load(&path, &settings(), &Limits::default()).is_none());
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
            dsd_rate: None,
        })
    );
}

#[test]
fn entries_of_older_analysis_versions_are_pruned() {
    let dir = tempfile::tempdir().unwrap();
    let path = fixture(dir.path(), "a.wav", 5);
    let a = analyze_file(&path, &settings(), &Limits::default()).unwrap();
    cache(dir.path())
        .store(&path, &settings(), &Limits::default(), &a)
        .unwrap();
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
        c.load(&path, &settings(), &Limits::default()),
        Some(a),
        "the current entry stays"
    );
}

/// A minimal stereo DSF at DSD64: `bytes` DSD bytes per channel.
fn dsf_file(dir: &Path, bytes: usize) -> PathBuf {
    const BLOCK: usize = 4096;
    let groups = bytes.div_ceil(BLOCK);
    let data = groups * BLOCK * 2;
    let mut f = Vec::new();
    f.extend_from_slice(b"DSD ");
    f.extend_from_slice(&28u64.to_le_bytes());
    f.extend_from_slice(&((92 + data) as u64).to_le_bytes());
    f.extend_from_slice(&0u64.to_le_bytes());
    f.extend_from_slice(b"fmt ");
    f.extend_from_slice(&52u64.to_le_bytes());
    for v in [1u32, 0, 2, 2, 2_822_400, 1] {
        f.extend_from_slice(&v.to_le_bytes());
    }
    f.extend_from_slice(&((bytes * 8) as u64).to_le_bytes());
    f.extend_from_slice(&(BLOCK as u32).to_le_bytes());
    f.extend_from_slice(&0u32.to_le_bytes());
    f.extend_from_slice(b"data");
    f.extend_from_slice(&((12 + data) as u64).to_le_bytes());
    // The DSD idle pattern (silence).
    f.extend(std::iter::repeat_n(0x69u8, data));
    let path = dir.join("a.dsf");
    std::fs::write(&path, f).unwrap();
    path
}

#[test]
fn a_dsf_records_its_dsd_rate_and_a_wav_does_not() {
    let dir = tempfile::tempdir().unwrap();
    let dsf = dsf_file(dir.path(), 2_822_400 / 8);
    let a = analyze_file(&dsf, &settings(), &Limits::default()).unwrap();
    let format = a.analysis.format.unwrap();
    assert_eq!(format.dsd_rate, Some(2_822_400));
    assert_eq!(format.sample_rate, 88_200);

    let wav = fixture(dir.path(), "a.wav", 2);
    let a = analyze_file(&wav, &settings(), &Limits::default()).unwrap();
    assert_eq!(a.analysis.format.unwrap().dsd_rate, None);
}

#[test]
fn a_vbr_mp3_without_a_length_frame_is_measured_to_its_end() {
    // 20 silent mono frames at 320 kbit/s, then 400 at 32 kbit/s (MPEG-1
    // Layer III, 44.1 kHz) and no Xing frame: a length estimated from the
    // first frames' bitrate is about 60 frames.
    let frame = |index: u8, len: usize| {
        let mut f = vec![0u8; len];
        f[..4].copy_from_slice(&[0xFF, 0xFB, index << 4, 0xC0]);
        f
    };
    let mut bytes = Vec::new();
    for _ in 0..20 {
        bytes.extend_from_slice(&frame(14, 1044));
    }
    for _ in 0..400 {
        bytes.extend_from_slice(&frame(1, 104));
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("vbr.mp3");
    std::fs::write(&path, bytes).unwrap();
    let a = analyze_file(&path, &settings(), &Limits::default()).unwrap();
    let expected = 420.0 * 1152.0 / 44_100.0;
    assert!(
        (a.analysis.duration_secs - expected).abs() < 0.05,
        "{} vs {expected}",
        a.analysis.duration_secs
    );
}

/// A stereo 1 kHz sine at `dbfs` (peak) for `secs`, 48 kHz, 16 bit.
fn stereo_tone(dir: &Path, name: &str, dbfs: f64, secs: f64) -> PathBuf {
    let path = dir.join(name);
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 48_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(&path, spec).unwrap();
    let amp = 10f64.powf(dbfs / 20.0);
    for i in 0..(secs * 48_000.0) as u64 {
        let x = amp * (std::f64::consts::TAU * 1_000.0 * i as f64 / 48_000.0).sin();
        let s = (x * f64::from(i16::MAX)) as i16;
        w.write_sample(s).unwrap();
        w.write_sample(s).unwrap();
    }
    w.finalize().unwrap();
    path
}

fn limits() -> Limits {
    Limits::default()
}

#[test]
fn n4_the_analysis_measures_the_loudness() {
    let dir = tempfile::tempdir().unwrap();
    let path = stereo_tone(dir.path(), "tone.wav", -23.0, 5.0);
    let a = analyze_file(&path, &settings(), &limits())
        .unwrap()
        .analysis;
    let l = a.loudness.unwrap();
    assert!((l.integrated_lufs.unwrap() + 23.0).abs() < 0.2, "{l:?}");
    assert!((l.true_peak_dbtp + 23.0).abs() < 0.5, "{l:?}");
    assert_eq!(a.loudness_pass, Some(LOUDNESS_VERSION));
    assert_eq!(
        ANALYSIS_VERSION, 8,
        "N6: the analysis version is not bumped"
    );
}

#[test]
fn n4_the_loudness_only_measurement_matches_the_full_analysis() {
    let dir = tempfile::tempdir().unwrap();
    let path = stereo_tone(dir.path(), "tone.wav", -18.0, 3.0);
    let full = analyze_file(&path, &settings(), &limits())
        .unwrap()
        .analysis;
    let only = measure_loudness_cancellable(&path, &|| false).unwrap();
    assert!(only.is_some());
    assert_eq!(full.loudness, only);
}

#[test]
fn n5_silence_is_measured_without_integrated_loudness() {
    let dir = tempfile::tempdir().unwrap();
    let path = stereo_tone(dir.path(), "silence.wav", -200.0, 2.0);
    let l = measure_loudness_cancellable(&path, &|| false)
        .unwrap()
        .unwrap();
    assert_eq!(l.integrated_lufs, None);
}

#[test]
fn n5_a_file_shorter_than_400_ms_is_not_measured() {
    let dir = tempfile::tempdir().unwrap();
    let path = stereo_tone(dir.path(), "blip.wav", -10.0, 0.2);
    let a = analyze_file(&path, &settings(), &limits())
        .unwrap()
        .analysis;
    assert_eq!(a.loudness, None);
    assert_eq!(
        a.loudness_pass,
        Some(LOUDNESS_VERSION),
        "tried, so not retried"
    );
    assert_eq!(
        measure_loudness_cancellable(&path, &|| false).unwrap(),
        None
    );
}

#[test]
fn n4_a_file_cut_short_is_measured_up_to_the_damage() {
    let dir = tempfile::tempdir().unwrap();
    let path = stereo_tone(dir.path(), "cut.wav", -20.0, 4.0);
    let bytes = std::fs::read(&path).unwrap();
    // Drop the last quarter of the data: the header still promises 4 s.
    std::fs::write(&path, &bytes[..bytes.len() * 3 / 4]).unwrap();
    let l = measure_loudness_cancellable(&path, &|| false)
        .unwrap()
        .unwrap();
    assert!((l.integrated_lufs.unwrap() + 20.0).abs() < 0.2, "{l:?}");
}

#[test]
fn the_loudness_measurement_can_be_cancelled() {
    let dir = tempfile::tempdir().unwrap();
    let path = stereo_tone(dir.path(), "tone.wav", -20.0, 2.0);
    assert_eq!(
        measure_loudness_cancellable(&path, &|| true),
        Err(AnalysisError::Cancelled)
    );
}

#[test]
fn the_loudness_of_a_missing_file_is_reported_as_missing() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(
        measure_loudness_cancellable(&dir.path().join("gone.wav"), &|| false),
        Err(AnalysisError::Missing)
    );
}

#[test]
fn n6_the_cache_round_trips_the_loudness() {
    let dir = tempfile::tempdir().unwrap();
    let path = stereo_tone(dir.path(), "tone.wav", -23.0, 2.0);
    let a = analyze_file(&path, &settings(), &limits()).unwrap();
    assert!(a.analysis.loudness.is_some());
    let c = cache(dir.path());
    c.store(&path, &settings(), &limits(), &a).unwrap();
    let back = c.load(&path, &settings(), &limits()).unwrap();
    assert_eq!(back.analysis.loudness, a.analysis.loudness);
    assert_eq!(back.analysis.loudness_pass, Some(LOUDNESS_VERSION));
}

#[test]
fn an_entry_without_loudness_still_loads() {
    // What the previous release wrote: the entry with no tail record.
    let dir = tempfile::tempdir().unwrap();
    let path = fixture(dir.path(), "a.wav", 5);
    let mut a = analyze_file(&path, &settings(), &limits()).unwrap();
    a.analysis.loudness = None;
    a.analysis.loudness_pass = None;
    let c = cache(dir.path());
    c.store(&path, &settings(), &limits(), &a).unwrap();
    let back = c.load(&path, &settings(), &limits()).unwrap();
    assert_eq!(back, a, "markers, peaks and cover intact");
    assert_eq!(
        (back.analysis.loudness, back.analysis.loudness_pass),
        (None, None)
    );
}

#[test]
fn n6b_update_loudness_fills_the_entry_in_place() {
    let dir = tempfile::tempdir().unwrap();
    let path = fixture(dir.path(), "a.wav", 5);
    let mut a = analyze_file(&path, &settings(), &limits()).unwrap();
    a.analysis.loudness = None;
    a.analysis.loudness_pass = None;
    let c = cache(dir.path());
    c.store(&path, &settings(), &limits(), &a).unwrap();
    let file = only_entry(dir.path());
    let before = std::fs::read(&file).unwrap();
    let key = c.key(&path, &settings(), &limits()).unwrap();
    let m = LoudnessMeasurement {
        integrated_lufs: Some(-12.5),
        true_peak_dbtp: -0.7,
    };
    assert!(c.update_loudness(&key, Some(m), &limits()).unwrap());
    let after = std::fs::read(&file).unwrap();
    assert!(after.len() > before.len());
    assert!(
        after.starts_with(&before),
        "the entry's own bytes do not change"
    );
    let back = c.load_key(&key, &limits()).unwrap();
    assert_eq!(back.analysis.loudness, Some(m));
    assert_eq!(back.analysis.loudness_pass, Some(LOUDNESS_VERSION));
    assert_eq!(back.peaks, a.peaks);
    assert_eq!(back.analysis.cue_in, a.analysis.cue_in);
}

#[test]
fn update_loudness_without_an_entry_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let path = fixture(dir.path(), "a.wav", 1);
    let c = cache(dir.path());
    let key = c.key(&path, &settings(), &limits()).unwrap();
    assert!(!c.update_loudness(&key, None, &limits()).unwrap());
    assert!(c.load_key(&key, &limits()).is_none());
}

#[test]
fn a_broken_tail_is_ignored() {
    let dir = tempfile::tempdir().unwrap();
    let path = stereo_tone(dir.path(), "tone.wav", -23.0, 1.0);
    let a = analyze_file(&path, &settings(), &limits()).unwrap();
    assert!(a.analysis.loudness.is_some());
    let c = cache(dir.path());
    c.store(&path, &settings(), &limits(), &a).unwrap();
    let file = only_entry(dir.path());
    let mut bytes = std::fs::read(&file).unwrap();
    bytes.truncate(bytes.len() - 2); // cut into the tail
    bytes.push(0xff);
    std::fs::write(&file, &bytes).unwrap();
    let back = c
        .load(&path, &settings(), &limits())
        .expect("the entry is kept");
    assert_eq!(
        (back.analysis.loudness, back.analysis.loudness_pass),
        (None, None)
    );
    assert_eq!(back.peaks, a.peaks);
}

/// I2: a tail whose numbers are not finite (a corrupt entry) is no
/// measurement: saved as JSON it would make the playlists file unloadable.
#[test]
fn a_tail_with_a_non_finite_measurement_is_ignored() {
    let dir = tempfile::tempdir().unwrap();
    let path = stereo_tone(dir.path(), "tone.wav", -23.0, 1.0);
    let c = cache(dir.path());
    for bad in [
        LoudnessMeasurement {
            integrated_lufs: Some(f32::NAN),
            true_peak_dbtp: -1.0,
        },
        LoudnessMeasurement {
            integrated_lufs: None,
            true_peak_dbtp: f32::INFINITY,
        },
    ] {
        let mut a = analyze_file(&path, &settings(), &limits()).unwrap();
        a.analysis.loudness = Some(bad);
        c.store(&path, &settings(), &limits(), &a).unwrap();
        let back = c
            .load(&path, &settings(), &limits())
            .expect("the entry is kept");
        assert_eq!(
            (back.analysis.loudness, back.analysis.loudness_pass),
            (None, None)
        );
        assert_eq!(back.peaks, a.peaks);
    }
}

/// The one `.bin` entry under the test cache.
fn only_entry(dir: &Path) -> PathBuf {
    let entries: Vec<PathBuf> = std::fs::read_dir(dir.join("cache"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "bin"))
        .collect();
    assert_eq!(entries.len(), 1);
    entries[0].clone()
}
