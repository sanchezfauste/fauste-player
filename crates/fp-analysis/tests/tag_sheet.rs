#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Feedback 2 spec O23: the tag sheet read from and written to real files.

use std::io::Cursor;
use std::path::{Path, PathBuf};

use fp_analysis::tags::{
    TagWriteError, read_tag_sheet, storable_fields, write_tag_sheet, write_tags,
};
use fp_model::{Limits, TagField, TagSheet, TrackTags};
use lofty::config::WriteOptions;
use lofty::id3::v2::{
    AttachedPictureFrame, CommentFrame, ExtendedTextFrame, Frame, FrameId, Id3v2Tag, UrlLinkFrame,
};
use lofty::picture::{MimeType, Picture, PictureType};
use lofty::prelude::*;
use lofty::tag::{ItemKey, Tag, TagType};

fn limits() -> Limits {
    Limits::default()
}

fn wav(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join(name);
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 8_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(&path, spec).unwrap();
    for _ in 0..800 {
        w.write_sample(0i16).unwrap();
    }
    w.finalize().unwrap();
    path
}

/// A WAV that already has a RIFF INFO chunk (a fresh WAV would get an ID3v2
/// tag, lofty's primary tag type for it), with a title and an encoder.
fn riff_wav(dir: &Path, name: &str) -> PathBuf {
    let path = wav(dir, name);
    let mut file = lofty::read_from_path(&path).unwrap();
    file.insert_tag(Tag::new(TagType::RiffInfo));
    let tag = file.tag_mut(TagType::RiffInfo).unwrap();
    tag.set_title("Old title".into());
    tag.insert_text(ItemKey::EncoderSoftware, "An Encoder 1.0".into());
    file.save_to_path(&path, WriteOptions::default()).unwrap();
    path
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

fn png() -> Vec<u8> {
    let img = image::RgbImage::from_pixel(40, 30, image::Rgb([200, 30, 30]));
    let mut out = Vec::new();
    image::DynamicImage::ImageRgb8(img)
        .write_to(&mut Cursor::new(&mut out), image::ImageFormat::Png)
        .unwrap();
    out
}

/// An MP3 with frames the sheet does not own: a custom `TXXX`, a comment
/// with a description, a URL frame lofty maps (`WCOM`), a picture; and the
/// plain fields title and comment.
fn mp3_with_other_tags(dir: &Path) -> PathBuf {
    let path = mp3(dir, "other.mp3");
    let mut tag = Id3v2Tag::new();
    tag.insert(Frame::UserText(ExtendedTextFrame::new(
        lofty::TextEncoding::UTF8,
        "MY_CUSTOM".to_owned(),
        "custom value".to_owned(),
    )));
    tag.insert(Frame::Comment(CommentFrame::new(
        lofty::TextEncoding::UTF8,
        *b"eng",
        "ReplayNote".to_owned(),
        "000 111".to_owned(),
    )));
    tag.insert(Frame::Comment(CommentFrame::new(
        lofty::TextEncoding::UTF8,
        *b"eng",
        String::new(),
        "a plain comment".to_owned(),
    )));
    tag.insert(Frame::Url(UrlLinkFrame::new(
        FrameId::new("WCOM").unwrap(),
        "http://shop.example",
    )));
    tag.insert(Frame::Url(UrlLinkFrame::new(
        FrameId::new("WOAR").unwrap(),
        "http://artist.example",
    )));
    tag.insert(Frame::Picture(AttachedPictureFrame::new(
        lofty::TextEncoding::UTF8,
        Picture::unchecked(png())
            .pic_type(PictureType::CoverFront)
            .mime_type(MimeType::Png)
            .build(),
    )));
    tag.set_title("Old title".to_owned());
    tag.save_to_path(&path, WriteOptions::default()).unwrap();
    path
}

fn sheet(path: &Path) -> TagSheet {
    read_tag_sheet(path, &limits()).expect("a sheet")
}

fn raw_contains(path: &Path, needle: &[u8]) -> bool {
    std::fs::read(path)
        .unwrap()
        .windows(needle.len())
        .any(|w| w == needle)
}

fn names(dir: &Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    v.sort();
    v
}

/// Saves `edit` applied to the sheet of `path`, and returns the sheet read
/// back.
fn edit(path: &Path, change: impl FnOnce(&mut TagSheet)) -> TagSheet {
    let before = sheet(path);
    let mut after = before.clone();
    change(&mut after);
    write_tag_sheet(path, &before, &after, &limits()).unwrap();
    sheet(path)
}

#[test]
fn what_a_format_can_store_decides_the_fields() {
    let id3 = storable_fields(TagType::Id3v2);
    let missing: Vec<_> = TagField::ALL
        .into_iter()
        .filter(|f| !id3.contains(f))
        .collect();
    assert_eq!(
        missing,
        [TagField::Performer],
        "ID3v2 has no performer frame"
    );

    let riff = storable_fields(TagType::RiffInfo);
    for stored in [
        TagField::Title,
        TagField::Artist,
        TagField::Album,
        TagField::Date,
        TagField::TrackNumber,
        TagField::Genre,
        TagField::Composer,
        TagField::Comment,
        TagField::Copyright,
        TagField::Language,
        TagField::EncodedBy,
    ] {
        assert!(riff.contains(&stored), "{stored:?}");
    }
    for not_stored in [TagField::AlbumArtist, TagField::DiscNumber, TagField::Bpm] {
        assert!(!riff.contains(&not_stored), "{not_stored:?}");
    }
    assert_eq!(
        storable_fields(TagType::AiffText),
        [
            TagField::Title,
            TagField::Artist,
            TagField::Comment,
            TagField::Copyright
        ]
    );
    let vorbis = storable_fields(TagType::VorbisComments);
    assert!(vorbis.contains(&TagField::Bpm) && vorbis.contains(&TagField::Lyrics));
    let mp4 = storable_fields(TagType::Mp4Ilst);
    assert!(mp4.contains(&TagField::Bpm) && !mp4.contains(&TagField::Publisher));
    assert!(storable_fields(TagType::Id3v1).contains(&TagField::Title));
}

#[test]
fn add_field_never_offers_what_the_file_cannot_store() {
    let dir = tempfile::tempdir().unwrap();
    let s = sheet(&riff_wav(dir.path(), "x.wav"));
    let added = std::collections::BTreeSet::new();
    assert_eq!(
        s.addable_fields(&added),
        [TagField::Copyright, TagField::Language, TagField::EncodedBy],
        "RIFF INFO stores 11 fields; the 25 others cannot be added"
    );
    assert!(!s.can_store(TagField::AlbumArtist));
    let s = sheet(&mp3(dir.path(), "x.mp3"));
    let offered = s.addable_fields(&added);
    assert_eq!(
        offered.len(),
        26 - 1,
        "every optional field but the performer"
    );
    assert!(!offered.contains(&TagField::Performer));
}

#[test]
fn an_untagged_file_gives_an_empty_sheet_of_its_primary_tag() {
    let dir = tempfile::tempdir().unwrap();
    let s = sheet(&wav(dir.path(), "x.wav"));
    assert!(TagField::ALL.iter().all(|f| !s.has(*f)));
    assert!(s.can_store(TagField::AlbumArtist), "a new WAV gets ID3v2");
    assert_eq!(s.other_kept, 0);
}

#[test]
fn a_file_without_a_sheet_says_so() {
    let dir = tempfile::tempdir().unwrap();
    assert!(read_tag_sheet(&dir.path().join("missing.wav"), &limits()).is_none());
    let dsf = dir.path().join("x.dsf");
    std::fs::write(&dsf, b"DSD ").unwrap();
    assert!(
        read_tag_sheet(&dsf, &limits()).is_none(),
        "no writable tags"
    );
    let broken = dir.path().join("broken.wav");
    std::fs::write(&broken, b"RIFF\x04\x00\x00\x00WAVE").unwrap();
    assert!(read_tag_sheet(&broken, &limits()).is_none());
}

#[test]
fn riff_info_round_trips_its_fields_with_values_one_per_line() {
    let dir = tempfile::tempdir().unwrap();
    let path = riff_wav(dir.path(), "x.wav");
    let s = edit(&path, |s| {
        s.set_text(TagField::Artist, "First Artist\nSecond Artist", true);
        s.set_text(TagField::Album, "An Album", false);
        s.set_text(TagField::Date, "1999-03-07", false);
        s.set_pair(TagField::TrackNumber, "3", "12");
        s.set_text(TagField::Genre, "Pop", true);
        s.set_text(TagField::Composer, "A. Composer", true);
        s.set_text(TagField::Comment, "A note\nover two lines", false);
        s.set_text(TagField::Copyright, "(c) 1999", false);
        s.set_text(TagField::Language, "eng", true);
    });
    assert_eq!(s.values(TagField::Title), ["Old title"], "untouched");
    assert_eq!(
        s.values(TagField::Artist),
        ["First Artist", "Second Artist"]
    );
    assert_eq!(s.values(TagField::Album), ["An Album"]);
    assert_eq!(s.values(TagField::Date), ["1999-03-07"]);
    assert_eq!(s.pair(TagField::TrackNumber), ("3".into(), "12".into()));
    assert_eq!(s.values(TagField::Genre), ["Pop"]);
    assert_eq!(s.values(TagField::Composer), ["A. Composer"]);
    assert_eq!(s.values(TagField::Comment), ["A note\nover two lines"]);
    assert_eq!(s.values(TagField::Copyright), ["(c) 1999"]);
    assert_eq!(s.values(TagField::Language), ["eng"]);
    assert_eq!(s.other_kept, 1, "the encoder software item is not a field");
    // The tag stayed a RIFF INFO tag: no ID3v2 appeared.
    let file = lofty::read_from_path(&path).unwrap();
    assert!(file.tag(TagType::Id3v2).is_none());
}

#[test]
fn a_riff_number_written_as_one_value_is_split() {
    let dir = tempfile::tempdir().unwrap();
    let path = riff_wav(dir.path(), "x.wav");
    let mut file = lofty::read_from_path(&path).unwrap();
    file.tag_mut(TagType::RiffInfo)
        .unwrap()
        .insert_text(ItemKey::TrackNumber, "3/12".into());
    file.save_to_path(&path, WriteOptions::default()).unwrap();
    assert_eq!(
        sheet(&path).pair(TagField::TrackNumber),
        ("3".into(), "12".into())
    );
}

#[test]
fn a_field_the_format_cannot_store_is_refused_before_the_disk_is_touched() {
    let dir = tempfile::tempdir().unwrap();
    let path = riff_wav(dir.path(), "x.wav");
    let original = std::fs::read(&path).unwrap();
    let before = sheet(&path);
    let mut after = before.clone();
    after.set_text(TagField::AlbumArtist, "Various", true);
    assert_eq!(
        write_tag_sheet(&path, &before, &after, &limits()),
        Err(TagWriteError::InvalidField(TagField::AlbumArtist))
    );
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert_eq!(names(dir.path()), ["x.wav"]);
}

#[test]
fn every_id3v2_field_round_trips() {
    let dir = tempfile::tempdir().unwrap();
    let path = mp3(dir.path(), "x.mp3");
    let wanted: Vec<(TagField, &str)> = vec![
        (TagField::Title, "Song Title"),
        (TagField::Artist, "Artist One\nArtist Two"),
        (TagField::Album, "An Album"),
        (TagField::AlbumArtist, "Various Artists"),
        (TagField::Date, "2019-05-14"),
        (TagField::Genre, "Pop\nRock"),
        (TagField::Composer, "A. Composer\nB. Composer"),
        (TagField::Comment, "A note\nover two lines"),
        (TagField::Subtitle, "A subtitle"),
        (TagField::Grouping, "A grouping"),
        (TagField::Bpm, "128"),
        (TagField::InitialKey, "Am"),
        (TagField::Mood, "Calm"),
        (TagField::Isrc, "ESABC1900001"),
        (TagField::Publisher, "A Label"),
        (TagField::CatalogNumber, "CAT-001"),
        (TagField::Copyright, "(c) 2019"),
        (TagField::OriginalArtist, "Original Artist"),
        (TagField::OriginalAlbum, "Original Album"),
        (TagField::OriginalReleaseDate, "1975"),
        (TagField::Lyricist, "A Lyricist"),
        (TagField::Conductor, "A Conductor"),
        (TagField::Remixer, "A Remixer"),
        (TagField::Arranger, "An Arranger"),
        (TagField::Language, "eng"),
        (TagField::EncodedBy, "Someone"),
        (TagField::Lyrics, "first line\nsecond line"),
        (TagField::SortTitle, "Title, Song"),
        (TagField::SortArtist, "Artist, One"),
        (TagField::SortAlbum, "Album, An"),
        (TagField::SortAlbumArtist, "Artists, Various"),
        (TagField::SortComposer, "Composer, A."),
        (TagField::ArtistWebsite, "http://artist.example"),
    ];
    let before = sheet(&path);
    let mut after = before.clone();
    for (field, text) in &wanted {
        after.set_text(*field, text, after.shows_lines(*field));
    }
    after.set_pair(TagField::TrackNumber, "3", "12");
    after.set_pair(TagField::DiscNumber, "1", "2");
    write_tag_sheet(&path, &before, &after, &limits()).unwrap();

    let read = sheet(&path);
    for (field, _) in &wanted {
        assert_eq!(read.values(*field), after.values(*field), "{field:?}");
    }
    assert_eq!(read.pair(TagField::TrackNumber), ("3".into(), "12".into()));
    assert_eq!(read.pair(TagField::DiscNumber), ("1".into(), "2".into()));
    assert_eq!(read.values(TagField::Artist), ["Artist One", "Artist Two"]);
    assert_eq!(read.values(TagField::Lyrics), ["first line\nsecond line"]);
    assert!(!read.has(TagField::Performer));
    // The ID3v2.4 frames are the standard ones.
    for frame in [
        &b"TPE1"[..],
        b"TPE2",
        b"TDRC",
        b"TRCK",
        b"TPOS",
        b"TCON",
        b"TCOM",
        b"COMM",
        b"TIT3",
        b"TIT1",
        b"TBPM",
        b"TKEY",
        b"TMOO",
        b"TSRC",
        b"TPUB",
        b"TCOP",
        b"TOPE",
        b"TOAL",
        b"TDOR",
        b"TEXT",
        b"TPE3",
        b"TPE4",
        b"TLAN",
        b"TENC",
        b"USLT",
        b"TSOT",
        b"TSOP",
        b"TSOA",
        b"TSO2",
        b"TSOC",
        b"WOAR",
    ] {
        assert!(
            raw_contains(&path, frame),
            "frame {}",
            String::from_utf8_lossy(frame)
        );
    }
    let file = lofty::read_from_path(&path).unwrap();
    let tag = file.primary_tag().unwrap();
    assert_eq!(tag.get_strings(ItemKey::TrackArtist).count(), 2);
}

#[test]
fn only_the_changed_fields_are_written() {
    let dir = tempfile::tempdir().unwrap();
    let path = mp3(dir.path(), "x.mp3");
    edit(&path, |s| {
        s.set_text(TagField::Title, "Kept title", false);
        s.set_text(TagField::Mood, "Calm", false);
        s.set_text(TagField::Genre, "Pop", true);
    });
    let s = edit(&path, |s| {
        s.set_text(TagField::Genre, "Jazz", true);
    });
    assert_eq!(s.values(TagField::Genre), ["Jazz"]);
    assert_eq!(s.values(TagField::Title), ["Kept title"]);
    assert_eq!(s.values(TagField::Mood), ["Calm"]);
}

#[test]
fn a_cleared_field_is_removed_and_an_empty_added_field_is_not_written() {
    let dir = tempfile::tempdir().unwrap();
    let path = mp3(dir.path(), "x.mp3");
    edit(&path, |s| {
        s.set_text(TagField::Mood, "Calm", false);
        s.set_pair(TagField::TrackNumber, "3", "12");
        s.set_text(TagField::Date, "1999", false);
    });
    let s = edit(&path, |s| {
        s.set_text(TagField::Mood, "", false);
        s.set_pair(TagField::TrackNumber, "", "");
        s.set_text(TagField::Date, "", false);
        s.set_text(TagField::Isrc, "  ", false);
    });
    assert!(!s.has(TagField::Mood));
    assert!(!s.has(TagField::TrackNumber));
    assert!(!s.has(TagField::Date));
    assert!(!s.has(TagField::Isrc));
    assert!(!raw_contains(&path, b"TMOO"));
    assert!(!raw_contains(&path, b"TSRC"));
    assert!(!raw_contains(&path, b"TRCK"));
}

#[test]
fn custom_items_and_pictures_survive_an_edit_untouched() {
    let dir = tempfile::tempdir().unwrap();
    let path = mp3_with_other_tags(dir.path());
    let before = sheet(&path);
    assert_eq!(before.values(TagField::Comment), ["a plain comment"]);
    assert_eq!(
        before.values(TagField::ArtistWebsite),
        ["http://artist.example"]
    );
    assert_eq!(
        before.other_kept, 2,
        "the described comment and the WCOM url (the cover is the sheet's own)"
    );
    assert!(
        before.other_kept_more,
        "the TXXX frame is kept but lofty does not count it"
    );

    let after = edit(&path, |s| {
        s.set_text(TagField::Title, "New title", false);
        s.set_text(TagField::Comment, "a new comment", false);
        s.set_text(TagField::Genre, "Pop", true);
    });
    assert_eq!(after.values(TagField::Comment), ["a new comment"]);
    assert_eq!(after.other_kept, 2, "nothing was lost");
    assert!(raw_contains(&path, b"MY_CUSTOM"), "the custom frame");
    assert!(raw_contains(&path, b"custom value"));
    assert!(raw_contains(&path, b"ReplayNote"), "the described comment");
    assert!(raw_contains(&path, b"000 111"));
    assert!(
        raw_contains(&path, b"WCOM"),
        "the url lofty reads as a locator"
    );
    assert!(raw_contains(&path, b"http://shop.example"));
    assert!(raw_contains(&path, b"WOAR"));
    let file = lofty::read_from_path(&path).unwrap();
    let tag = file.primary_tag().unwrap();
    assert_eq!(tag.picture_count(), 1, "the cover");
    assert_eq!(tag.pictures()[0].data(), png().as_slice(), "byte for byte");
}

#[test]
fn a_summary_write_keeps_the_url_frames_too() {
    let dir = tempfile::tempdir().unwrap();
    let path = mp3_with_other_tags(dir.path());
    let before = fp_analysis::tags::read_track_tags(&path, &limits());
    let after = TrackTags {
        title: "Another".into(),
        ..before.clone()
    };
    write_tags(&path, &before, &after, &limits()).unwrap();
    assert!(raw_contains(&path, b"WCOM"));
    assert!(raw_contains(&path, b"WOAR"));
}

#[test]
fn an_invalid_value_is_refused_and_the_file_is_untouched() {
    let dir = tempfile::tempdir().unwrap();
    let path = mp3(dir.path(), "x.mp3");
    let original = std::fs::read(&path).unwrap();
    let before = sheet(&path);
    for (field, bad) in [
        (TagField::Date, "2019-13"),
        (TagField::OriginalReleaseDate, "May 1999"),
        (TagField::Bpm, "fast"),
    ] {
        let mut after = before.clone();
        after.set_text(field, bad, false);
        assert_eq!(
            write_tag_sheet(&path, &before, &after, &limits()),
            Err(TagWriteError::InvalidField(field))
        );
    }
    let mut after = before.clone();
    after.set_pair(TagField::TrackNumber, "x", "");
    assert_eq!(
        write_tag_sheet(&path, &before, &after, &limits()),
        Err(TagWriteError::InvalidField(TagField::TrackNumber))
    );
    let mut after = before.clone();
    after.set_pair(TagField::DiscNumber, "", "2");
    assert_eq!(
        write_tag_sheet(&path, &before, &after, &limits()),
        Err(TagWriteError::InvalidField(TagField::DiscNumber)),
        "a total without a number would be written as disc 0"
    );
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert_eq!(names(dir.path()), ["x.mp3"]);
}

#[test]
fn a_failed_sheet_write_leaves_the_original_and_no_temporary_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("broken.wav");
    std::fs::write(&path, b"RIFF\x04\x00\x00\x00WAVE").unwrap();
    let original = std::fs::read(&path).unwrap();
    let before = TagSheet::new(storable_fields(TagType::Id3v2), 0, false);
    let mut after = before.clone();
    after.set_text(TagField::Title, "x", false);
    let result = write_tag_sheet(&path, &before, &after, &limits());
    assert!(matches!(result, Err(TagWriteError::Other(_))), "{result:?}");
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert_eq!(names(dir.path()), ["broken.wav"]);
}

#[test]
fn an_unwritable_format_is_refused_without_touching_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("x.dsf");
    std::fs::write(&path, b"DSD fake").unwrap();
    let before = TagSheet::new([TagField::Title], 0, false);
    let mut after = before.clone();
    after.set_text(TagField::Title, "x", false);
    assert_eq!(
        write_tag_sheet(&path, &before, &after, &limits()),
        Err(TagWriteError::Unsupported)
    );
    assert_eq!(std::fs::read(&path).unwrap(), b"DSD fake");
    assert_eq!(names(dir.path()), ["x.dsf"]);
}

#[test]
fn the_text_and_the_values_are_cut_when_read() {
    let dir = tempfile::tempdir().unwrap();
    let path = mp3(dir.path(), "x.mp3");
    let many: Vec<String> = (0..50)
        .map(|n| format!("Artist {n} with a long name"))
        .collect();
    let big = Limits {
        max_tag_values: 100,
        ..limits()
    };
    let before = read_tag_sheet(&path, &big).unwrap();
    let mut after = before.clone();
    after.set_values(TagField::Artist, many);
    write_tag_sheet(&path, &before, &after, &big).unwrap();

    let small = Limits {
        max_tag_chars: 64,
        max_tag_values: 3,
        ..limits()
    };
    let s = read_tag_sheet(&path, &small).unwrap();
    assert_eq!(s.values(TagField::Artist).len(), 3);
    assert!(
        s.values(TagField::Artist)
            .iter()
            .all(|a| a.chars().count() <= 64)
    );
}

#[test]
fn what_is_written_is_cut_to_the_limits() {
    let dir = tempfile::tempdir().unwrap();
    let path = mp3(dir.path(), "x.mp3");
    let small = Limits {
        max_tag_chars: 64,
        max_tag_values: 2,
        ..limits()
    };
    let before = read_tag_sheet(&path, &small).unwrap();
    let mut after = before.clone();
    after.set_text(TagField::Genre, "A\nB\nC\nD", true);
    after.set_text(TagField::Comment, &"x".repeat(500), false);
    write_tag_sheet(&path, &before, &after, &small).unwrap();
    let read = read_tag_sheet(&path, &limits()).unwrap();
    assert_eq!(read.values(TagField::Genre), ["A", "B"]);
    assert_eq!(read.values(TagField::Comment)[0].chars().count(), 64);
}

#[test]
fn a_full_date_is_not_reduced_by_an_unrelated_edit() {
    let dir = tempfile::tempdir().unwrap();
    let path = mp3(dir.path(), "x.mp3");
    edit(&path, |s| {
        s.set_text(TagField::Date, "2019-05-14T10:30", false)
    });
    let s = edit(&path, |s| s.set_text(TagField::Genre, "Pop", true));
    assert_eq!(s.values(TagField::Date), ["2019-05-14T10:30"]);
}

/// The artists of a file, in the order they are stored.
fn artists(path: &Path) -> Vec<String> {
    sheet(path).values(TagField::Artist).to_vec()
}

#[test]
fn a_save_keeps_the_order_of_the_values_it_did_not_change() {
    // The order of the items of a parsed tag depends on the order of the
    // frames in the file, which lofty writes in no fixed order, so each
    // round tries another arrangement. lofty 0.25.4's `Tag::take_filter`
    // swaps items and reorders the ones it leaves: a field written next to
    // the artists must not turn "First, Second" into "Second, First".
    let dir = tempfile::tempdir().unwrap();
    for round in 0..16 {
        let path = mp3_with_other_tags(dir.path());
        edit(&path, |s| {
            s.set_text(TagField::Artist, "First\nSecond\nThird", true);
            s.set_text(TagField::Genre, "Pop", true);
            s.set_pair(TagField::TrackNumber, "3", "12");
        });
        assert_eq!(
            artists(&path),
            ["First", "Second", "Third"],
            "round {round}"
        );
        edit(&path, |s| {
            s.set_text(TagField::Genre, "Jazz", true);
            s.set_pair(TagField::TrackNumber, "3", "14");
            s.set_text(TagField::Mood, "Calm", true);
        });
        assert_eq!(
            artists(&path),
            ["First", "Second", "Third"],
            "round {round}: the artists did not change"
        );
    }
}
