#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Feedback 2 spec O23: reading the extra tags and writing them safely.

use std::io::Cursor;
use std::path::{Path, PathBuf};

use fp_analysis::tags::{TagWriteError, can_write_tags, read_track_tags, write_tags};
use fp_model::{Limits, TrackTags};
use lofty::config::WriteOptions;
use lofty::picture::{MimeType, Picture, PictureType};
use lofty::prelude::*;
use lofty::tag::{ItemKey, Tag};

fn wav(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join(name);
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 44_100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(&path, spec).unwrap();
    for i in 0..4_410 {
        w.write_sample((i % 1000) as i16).unwrap();
        w.write_sample(0i16).unwrap();
    }
    w.finalize().unwrap();
    path
}

fn limits() -> Limits {
    Limits::default()
}

fn full() -> TrackTags {
    TrackTags {
        title: "Song Title".into(),
        artist: "The Artist".into(),
        album: "An Album".into(),
        album_artist: "Various Artists".into(),
        date: Some("1999-03-07".into()),
        genre: "Pop".into(),
        composer: "A. Composer".into(),
        comment: "A note".into(),
    }
}

fn untagged_tags(path: &Path) -> TrackTags {
    read_track_tags(path, &limits())
}

fn png() -> Vec<u8> {
    let img = image::RgbImage::from_pixel(40, 30, image::Rgb([200, 30, 30]));
    let mut out = Vec::new();
    image::DynamicImage::ImageRgb8(img)
        .write_to(&mut Cursor::new(&mut out), image::ImageFormat::Png)
        .unwrap();
    out
}

/// Files in `dir`, to prove no temporary file is left.
fn names(dir: &Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    v.sort();
    v
}

#[test]
fn an_untagged_file_reads_as_its_file_name() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "Marta Oliva - Carretera norte.wav");
    let t = untagged_tags(&path);
    assert_eq!(t.title, "Carretera norte");
    assert_eq!(t.artist, "Marta Oliva");
    assert_eq!(
        (t.date, t.genre.as_str(), t.comment.as_str()),
        (None, "", "")
    );
}

#[test]
fn every_field_survives_a_write_and_a_read() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "x.wav");
    let before = untagged_tags(&path);
    write_tags(&path, &before, &full(), &limits()).unwrap();
    assert_eq!(read_track_tags(&path, &limits()), full());
    assert_eq!(names(dir.path()), ["x.wav"], "no temporary file is left");
}

#[test]
fn the_audio_is_untouched_by_a_write() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "x.wav");
    let before = untagged_tags(&path);
    let samples = |p: &Path| -> Vec<i16> {
        hound::WavReader::open(p)
            .unwrap()
            .samples::<i16>()
            .map(Result::unwrap)
            .collect()
    };
    let audio = samples(&path);
    write_tags(&path, &before, &full(), &limits()).unwrap();
    assert_eq!(samples(&path), audio);
}

#[test]
fn a_cleared_field_removes_the_tag() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "x.wav");
    let before = untagged_tags(&path);
    write_tags(&path, &before, &full(), &limits()).unwrap();
    let tagged = read_track_tags(&path, &limits());
    let cleared = TrackTags {
        album: String::new(),
        date: None,
        composer: String::new(),
        album_artist: String::new(),
        ..tagged.clone()
    };
    write_tags(&path, &tagged, &cleared, &limits()).unwrap();
    assert_eq!(read_track_tags(&path, &limits()), cleared);
}

#[test]
fn fields_that_did_not_change_are_not_written() {
    // The title shown for an untagged file is its file name; saving a new
    // artist must not turn that name into a title tag.
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "Just a name.wav");
    let before = untagged_tags(&path);
    let after = TrackTags {
        artist: "Someone".into(),
        ..before.clone()
    };
    write_tags(&path, &before, &after, &limits()).unwrap();
    let tag = lofty::read_from_path(&path).unwrap();
    let tag = tag.primary_tag().unwrap();
    assert_eq!(tag.artist().as_deref(), Some("Someone"));
    assert_eq!(
        tag.title(),
        None,
        "the file name was not written as a title"
    );
}

#[test]
fn a_cover_and_other_items_survive_a_write() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "x.wav");
    {
        let mut file = lofty::read_from_path(&path).unwrap();
        let ty = file.primary_tag_type();
        file.insert_tag(Tag::new(ty));
        let tag = file.primary_tag_mut().unwrap();
        tag.set_title("Old".into());
        tag.insert_text(ItemKey::Publisher, "A Label".into());
        tag.push_picture(
            Picture::unchecked(png())
                .pic_type(PictureType::CoverFront)
                .mime_type(MimeType::Png)
                .build(),
        );
        file.save_to_path(&path, WriteOptions::default()).unwrap();
    }
    let before = untagged_tags(&path);
    let after = TrackTags {
        title: "New".into(),
        ..before.clone()
    };
    write_tags(&path, &before, &after, &limits()).unwrap();
    let file = lofty::read_from_path(&path).unwrap();
    let tag = file.primary_tag().unwrap();
    assert_eq!(tag.title().as_deref(), Some("New"));
    assert_eq!(tag.get_string(ItemKey::Publisher), Some("A Label"));
    assert_eq!(tag.pictures().len(), 1, "the cover is kept");
}

#[test]
fn a_failed_write_leaves_the_original_and_no_temporary_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("broken.wav");
    // A RIFF header with no body: lofty cannot parse it.
    std::fs::write(&path, b"RIFF\x04\x00\x00\x00WAVE").unwrap();
    let original = std::fs::read(&path).unwrap();
    let result = write_tags(&path, &TrackTags::default(), &full(), &limits());
    assert!(matches!(result, Err(TagWriteError::Other(_))), "{result:?}");
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert_eq!(names(dir.path()), ["broken.wav"]);
}

#[test]
fn a_missing_file_is_reported_as_not_found() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("gone.wav");
    let result = write_tags(&path, &TrackTags::default(), &full(), &limits());
    assert_eq!(result, Err(TagWriteError::NotFound));
    assert!(names(dir.path()).is_empty());
}

#[test]
fn an_unwritable_format_is_refused_without_touching_the_file() {
    let dir = tempfile::tempdir().unwrap();
    for name in ["a.dsf", "a.dff", "a.caf", "a.mka", "a.txt", "noextension"] {
        let path = dir.path().join(name);
        std::fs::write(&path, b"not audio").unwrap();
        assert!(!can_write_tags(&path), "{name}");
        let result = write_tags(&path, &TrackTags::default(), &full(), &limits());
        assert_eq!(result, Err(TagWriteError::Unsupported), "{name}");
        assert_eq!(std::fs::read(&path).unwrap(), b"not audio");
    }
    for name in [
        "a.mp3", "a.FLAC", "a.wav", "a.ogg", "a.opus", "a.m4a", "a.wv", "a.ape", "a.aiff",
    ] {
        assert!(can_write_tags(Path::new(name)), "{name}");
    }
}

#[cfg(unix)]
#[test]
fn a_read_only_folder_is_reported_as_denied_and_changes_nothing() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "x.wav");
    let original = std::fs::read(&path).unwrap();
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o555)).unwrap();
    let result = write_tags(&path, &untagged_tags(&path), &full(), &limits());
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
    // A superuser can write anywhere: the check only applies to the rest.
    if result.is_err() {
        assert_eq!(result, Err(TagWriteError::Denied));
        assert_eq!(std::fs::read(&path).unwrap(), original);
        assert_eq!(names(dir.path()), ["x.wav"]);
    }
}

#[cfg(unix)]
#[test]
fn a_symlink_stays_a_symlink() {
    let dir = tempfile::tempdir().unwrap();
    let real = wav(dir.path(), "real.wav");
    let link = dir.path().join("link.wav");
    std::os::unix::fs::symlink(&real, &link).unwrap();
    write_tags(&link, &untagged_tags(&real), &full(), &limits()).unwrap();
    assert!(
        std::fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(read_track_tags(&real, &limits()), full());
}

#[test]
fn a_huge_comment_is_cut_when_read() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "x.wav");
    let huge = TrackTags {
        comment: "x".repeat(5_000_000),
        ..TrackTags::default()
    };
    // A direct lofty write: `write_tags` would cut it first.
    {
        let mut file = lofty::read_from_path(&path).unwrap();
        let ty = file.primary_tag_type();
        file.insert_tag(Tag::new(ty));
        file.primary_tag_mut()
            .unwrap()
            .set_comment(huge.comment.clone());
        file.save_to_path(&path, WriteOptions::default()).unwrap();
    }
    let small = Limits {
        max_tag_chars: 100,
        ..Limits::default()
    };
    assert_eq!(read_track_tags(&path, &small).comment.chars().count(), 100);
}

#[test]
fn what_is_written_is_cut_to_the_limit() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "x.wav");
    let small = Limits {
        max_tag_chars: 10,
        ..Limits::default()
    };
    let after = TrackTags {
        comment: "y".repeat(50),
        ..untagged_tags(&path)
    };
    write_tags(&path, &untagged_tags(&path), &after, &small).unwrap();
    assert_eq!(read_track_tags(&path, &small).comment, "y".repeat(10));
}

fn tagged_by_date(path: &Path, date: &str) -> TrackTags {
    let before = untagged_tags(path);
    let after = TrackTags {
        date: Some(date.into()),
        ..before.clone()
    };
    write_tags(path, &before, &after, &limits()).unwrap();
    read_track_tags(path, &limits())
}

#[test]
fn a_full_date_is_written_and_read_back() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "x.wav");
    assert_eq!(
        tagged_by_date(&path, "2019-05-14").date.as_deref(),
        Some("2019-05-14")
    );
}

#[test]
fn a_bare_year_is_written_and_read_back() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "x.wav");
    assert_eq!(tagged_by_date(&path, "2019").date.as_deref(), Some("2019"));
}

#[test]
fn a_date_with_a_time_is_written_and_read_back() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "x.wav");
    assert_eq!(
        tagged_by_date(&path, "2019-05-14T08:30").date.as_deref(),
        Some("2019-05-14T08:30")
    );
}

#[test]
fn an_unrelated_edit_leaves_a_full_date_untouched() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "x.wav");
    let tagged = tagged_by_date(&path, "2019-05-14");
    let after = TrackTags {
        artist: "Someone".into(),
        ..tagged.clone()
    };
    write_tags(&path, &tagged, &after, &limits()).unwrap();
    let read = read_track_tags(&path, &limits());
    assert_eq!(read.date.as_deref(), Some("2019-05-14"));
    assert_eq!(read.artist, "Someone");
}

#[test]
fn clearing_the_date_removes_it() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "x.wav");
    let tagged = tagged_by_date(&path, "2019-05-14");
    let after = TrackTags {
        date: None,
        ..tagged.clone()
    };
    write_tags(&path, &tagged, &after, &limits()).unwrap();
    assert_eq!(read_track_tags(&path, &limits()).date, None);
}

#[test]
fn an_invalid_date_is_refused_and_the_file_is_untouched() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "x.wav");
    let original = std::fs::read(&path).unwrap();
    let before = untagged_tags(&path);
    let after = TrackTags {
        date: Some("last spring".into()),
        ..before.clone()
    };
    let result = write_tags(&path, &before, &after, &limits());
    assert_eq!(result, Err(TagWriteError::InvalidDate));
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert_eq!(names(dir.path()), ["x.wav"]);
}

/// A minimal MP3: MPEG-1 Layer III frames (128 kbps, 44.1 kHz, 417 bytes
/// each) with silent payloads.
fn mp3(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join(name);
    let mut bytes = Vec::new();
    for _ in 0..10 {
        let mut frame = vec![0u8; 417];
        frame[..4].copy_from_slice(&0xFFFB_9064u32.to_be_bytes());
        bytes.extend_from_slice(&frame);
    }
    std::fs::write(&path, bytes).unwrap();
    path
}

#[test]
fn the_recording_date_round_trips_through_id3v2_and_survives_an_edit() {
    let dir = tempfile::tempdir().unwrap();
    let path = mp3(dir.path(), "x.mp3");
    assert!(can_write_tags(&path));

    let tagged = tagged_by_date(&path, "2019-05-14");
    assert_eq!(tagged.date.as_deref(), Some("2019-05-14"));

    let file = lofty::probe::Probe::open(&path)
        .unwrap()
        .guess_file_type()
        .unwrap()
        .read()
        .unwrap();
    let id3 = file.tag(lofty::tag::TagType::Id3v2).expect("an ID3v2 tag");
    assert_eq!(
        id3.get_string(ItemKey::RecordingDate),
        Some("2019-05-14"),
        "TDRC holds the full date"
    );

    let raw = std::fs::read(&path).unwrap();
    assert!(raw.windows(4).any(|w| w == b"TDRC"), "the frame is TDRC");

    let after = TrackTags {
        title: "Edited".into(),
        ..tagged.clone()
    };
    write_tags(&path, &tagged, &after, &limits()).unwrap();
    let read = read_track_tags(&path, &limits());
    assert_eq!(read.title, "Edited");
    assert_eq!(read.date.as_deref(), Some("2019-05-14"));
}
