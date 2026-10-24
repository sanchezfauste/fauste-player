#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Feedback 2 spec O23: the front cover in the tag sheet, read from and
//! written to real files.

use std::io::Cursor;
use std::path::{Path, PathBuf};

use fp_analysis::tags::{
    CoverError, TagWriteError, can_store_pictures, load_cover_file, read_tag_sheet,
    with_cover_thumbnail, write_tag_sheet,
};
use fp_model::{CoverArt, Limits, TagField, TagSheet};
use lofty::config::WriteOptions;
use lofty::id3::v2::{AttachedPictureFrame, Frame, Id3v2Tag};
use lofty::picture::{MimeType, Picture, PictureType};
use lofty::prelude::*;
use lofty::tag::{Tag, TagType};

fn limits() -> Limits {
    Limits::default()
}

fn encode(format: image::ImageFormat, width: u32, color: [u8; 3]) -> Vec<u8> {
    let img = image::RgbImage::from_pixel(width, 30, image::Rgb(color));
    let mut out = Vec::new();
    image::DynamicImage::ImageRgb8(img)
        .write_to(&mut Cursor::new(&mut out), format)
        .unwrap();
    out
}

/// A red PNG, a blue JPEG and a green PNG: three different images.
fn red_png() -> Vec<u8> {
    encode(image::ImageFormat::Png, 40, [200, 30, 30])
}

fn blue_jpeg() -> Vec<u8> {
    encode(image::ImageFormat::Jpeg, 50, [30, 30, 200])
}

fn green_png() -> Vec<u8> {
    encode(image::ImageFormat::Png, 60, [30, 200, 30])
}

/// A minimal MP3: MPEG-1 Layer III frames with silent payloads.
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

fn picture(data: Vec<u8>, kind: PictureType, mime: MimeType) -> Frame<'static> {
    Frame::Picture(AttachedPictureFrame::new(
        lofty::TextEncoding::UTF8,
        Picture::unchecked(data)
            .pic_type(kind)
            .mime_type(mime)
            .build(),
    ))
}

/// An MP3 with `pictures` (data, type, mime) and a title.
fn mp3_with(dir: &Path, pictures: Vec<(Vec<u8>, PictureType, MimeType)>) -> PathBuf {
    let path = mp3(dir, "x.mp3");
    let mut tag = Id3v2Tag::new();
    for (data, kind, mime) in pictures {
        tag.insert(picture(data, kind, mime));
    }
    tag.set_title("Old title".to_owned());
    tag.save_to_path(&path, WriteOptions::default()).unwrap();
    path
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

/// A WAV whose tag is RIFF INFO, which has no place for a picture.
fn riff_wav(dir: &Path) -> PathBuf {
    let path = wav(dir, "riff.wav");
    let mut file = lofty::read_from_path(&path).unwrap();
    file.insert_tag(Tag::new(TagType::RiffInfo));
    file.tag_mut(TagType::RiffInfo)
        .unwrap()
        .set_title("Old title".into());
    file.save_to_path(&path, WriteOptions::default()).unwrap();
    path
}

fn sheet(path: &Path) -> TagSheet {
    read_tag_sheet(path, &limits()).expect("a sheet")
}

fn pictures_of(path: &Path) -> Vec<(PictureType, Vec<u8>)> {
    let file = lofty::read_from_path(path).unwrap();
    file.primary_tag()
        .unwrap()
        .pictures()
        .iter()
        .map(|p| (p.pic_type(), p.data().to_vec()))
        .collect()
}

fn names(dir: &Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    v.sort();
    v
}

/// Saves `change` applied to the sheet of `path`.
fn edit(path: &Path, change: impl FnOnce(&mut TagSheet)) -> TagSheet {
    let before = sheet(path);
    let mut after = before.clone();
    change(&mut after);
    write_tag_sheet(path, &before, &after, &limits()).unwrap();
    sheet(path)
}

fn new_cover(data: Vec<u8>) -> CoverArt {
    CoverArt::new(data, true)
}

#[test]
fn the_sheet_shows_the_front_cover() {
    let dir = tempfile::tempdir().unwrap();
    let path = mp3_with(
        dir.path(),
        vec![
            (blue_jpeg(), PictureType::CoverBack, MimeType::Jpeg),
            (red_png(), PictureType::CoverFront, MimeType::Png),
        ],
    );
    let s = sheet(&path);
    let cover = s.cover().expect("a cover");
    assert!(cover.is_front());
    assert_eq!(
        cover.data(),
        red_png().as_slice(),
        "the front, not the first"
    );
    assert!(s.can_store_cover());
    assert_eq!(
        s.other_kept, 1,
        "the back cover is kept, the front is the sheet's"
    );
}

#[test]
fn without_a_front_cover_the_first_picture_is_shown_and_never_changed() {
    let dir = tempfile::tempdir().unwrap();
    let path = mp3_with(
        dir.path(),
        vec![(blue_jpeg(), PictureType::CoverBack, MimeType::Jpeg)],
    );
    let s = sheet(&path);
    let cover = s.cover().expect("the first picture");
    assert!(!cover.is_front());
    assert_eq!(cover.data(), blue_jpeg().as_slice());
    assert_eq!(s.other_kept, 1, "it is not the sheet's: it is kept");
    // Changing another field leaves it alone; so does Remove.
    let after = edit(&path, |s| {
        let original = s.clone();
        s.remove_front_cover(&original);
        s.set_text(TagField::Genre, "Pop", true);
    });
    assert_eq!(after.cover().unwrap().data(), blue_jpeg().as_slice());
    assert_eq!(
        pictures_of(&path),
        [(PictureType::CoverBack, blue_jpeg())],
        "the back cover is still the only picture"
    );
}

#[test]
fn a_file_without_pictures_has_no_cover() {
    let dir = tempfile::tempdir().unwrap();
    let s = sheet(&mp3(dir.path(), "x.mp3"));
    assert!(s.cover().is_none());
    assert!(s.can_store_cover());
    assert_eq!(s.other_kept, 0);
}

#[test]
fn the_front_cover_can_be_replaced() {
    let dir = tempfile::tempdir().unwrap();
    let path = mp3_with(
        dir.path(),
        vec![(red_png(), PictureType::CoverFront, MimeType::Png)],
    );
    let after = edit(&path, |s| s.set_cover(Some(new_cover(blue_jpeg()))));
    assert_eq!(after.cover().unwrap().data(), blue_jpeg().as_slice());
    assert!(after.cover().unwrap().is_front());
    assert_eq!(
        pictures_of(&path),
        [(PictureType::CoverFront, blue_jpeg())],
        "one front cover, the new one"
    );
    let file = lofty::read_from_path(&path).unwrap();
    let tag = file.primary_tag().unwrap();
    assert_eq!(tag.pictures()[0].mime_type(), Some(&MimeType::Jpeg));
    // The other direction: a PNG over a JPEG gets the PNG type.
    edit(&path, |s| s.set_cover(Some(new_cover(green_png()))));
    let file = lofty::read_from_path(&path).unwrap();
    let tag = file.primary_tag().unwrap();
    assert_eq!(tag.pictures()[0].mime_type(), Some(&MimeType::Png));
    assert_eq!(tag.pictures()[0].pic_type(), PictureType::CoverFront);
}

#[test]
fn a_cover_can_be_added_to_a_file_that_has_none() {
    let dir = tempfile::tempdir().unwrap();
    let path = mp3(dir.path(), "x.mp3");
    let after = edit(&path, |s| s.set_cover(Some(new_cover(green_png()))));
    assert_eq!(after.cover().unwrap().data(), green_png().as_slice());
    assert_eq!(pictures_of(&path), [(PictureType::CoverFront, green_png())]);
}

#[test]
fn the_front_cover_can_be_removed() {
    let dir = tempfile::tempdir().unwrap();
    let path = mp3_with(
        dir.path(),
        vec![(red_png(), PictureType::CoverFront, MimeType::Png)],
    );
    let after = edit(&path, |s| {
        let original = s.clone();
        s.remove_front_cover(&original);
    });
    assert!(after.cover().is_none());
    assert!(pictures_of(&path).is_empty());
    assert_eq!(
        after.values(TagField::Title),
        ["Old title"],
        "the text stays"
    );
}

#[test]
fn an_unrelated_edit_keeps_the_cover_byte_for_byte() {
    let dir = tempfile::tempdir().unwrap();
    let path = mp3_with(
        dir.path(),
        vec![(red_png(), PictureType::CoverFront, MimeType::Png)],
    );
    edit(&path, |s| {
        s.set_text(TagField::Title, "New title", false);
        s.set_text(TagField::Genre, "Pop", true);
    });
    assert_eq!(pictures_of(&path), [(PictureType::CoverFront, red_png())]);
    let raw = std::fs::read(&path).unwrap();
    let png = red_png();
    assert!(
        raw.windows(png.len()).any(|w| w == png.as_slice()),
        "the image is in the file as it was"
    );
}

#[test]
fn a_back_cover_survives_a_front_cover_change() {
    let dir = tempfile::tempdir().unwrap();
    let path = mp3_with(
        dir.path(),
        vec![
            (red_png(), PictureType::CoverFront, MimeType::Png),
            (blue_jpeg(), PictureType::CoverBack, MimeType::Jpeg),
            (green_png(), PictureType::Artist, MimeType::Png),
        ],
    );
    edit(&path, |s| s.set_cover(Some(new_cover(green_png()))));
    let mut pictures = pictures_of(&path);
    pictures.sort_by_key(|(kind, _)| kind.as_u8());
    assert_eq!(
        pictures,
        [
            (PictureType::CoverFront, green_png()),
            (PictureType::CoverBack, blue_jpeg()),
            (PictureType::Artist, green_png()),
        ]
    );
    // Removing the front cover leaves the two others too.
    let after = edit(&path, |s| {
        let original = s.clone();
        s.remove_front_cover(&original);
    });
    let mut pictures = pictures_of(&path);
    pictures.sort_by_key(|(kind, _)| kind.as_u8());
    assert_eq!(
        pictures,
        [
            (PictureType::CoverBack, blue_jpeg()),
            (PictureType::Artist, green_png()),
        ]
    );
    assert_eq!(after.other_kept, 2);
}

#[test]
fn a_cover_change_and_a_field_change_are_one_save() {
    let dir = tempfile::tempdir().unwrap();
    let path = mp3(dir.path(), "x.mp3");
    let after = edit(&path, |s| {
        s.set_text(TagField::Album, "An Album", false);
        s.set_cover(Some(new_cover(red_png())));
    });
    assert_eq!(after.values(TagField::Album), ["An Album"]);
    assert_eq!(after.cover().unwrap().data(), red_png().as_slice());
}

#[test]
fn a_format_without_pictures_shows_no_cover_and_refuses_one() {
    let dir = tempfile::tempdir().unwrap();
    let path = riff_wav(dir.path());
    let original = std::fs::read(&path).unwrap();
    let before = sheet(&path);
    assert!(!before.can_store_cover());
    assert!(before.cover().is_none());
    let mut after = before.clone();
    after.set_cover(Some(new_cover(red_png())));
    assert_eq!(
        write_tag_sheet(&path, &before, &after, &limits()),
        Err(TagWriteError::CoverNotStorable)
    );
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert_eq!(names(dir.path()), ["riff.wav"]);
    // A fresh WAV gets ID3v2, which stores one.
    let fresh = wav(dir.path(), "fresh.wav");
    assert!(sheet(&fresh).can_store_cover());
}

#[test]
fn which_tag_types_store_pictures() {
    for stored in [
        TagType::Id3v2,
        TagType::VorbisComments,
        TagType::Mp4Ilst,
        TagType::Ape,
    ] {
        assert!(can_store_pictures(stored), "{stored:?}");
    }
    for not_stored in [TagType::RiffInfo, TagType::AiffText, TagType::Id3v1] {
        assert!(!can_store_pictures(not_stored), "{not_stored:?}");
    }
}

#[test]
fn an_oversized_image_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let png = dir.path().join("big.png");
    std::fs::write(&png, red_png()).unwrap();
    let small = Limits {
        max_cover_bytes: 100,
        ..limits()
    };
    assert_eq!(
        load_cover_file(&png, &small, 64).unwrap_err(),
        CoverError::TooLarge
    );
    // Too many pixels is a decode failure, not a crash or a huge allocation.
    let few_pixels = Limits {
        max_cover_pixels: 16,
        ..limits()
    };
    assert_eq!(
        load_cover_file(&png, &few_pixels, 64).unwrap_err(),
        CoverError::Undecodable
    );
}

#[test]
fn an_undecodable_or_other_image_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let load = |name: &str, bytes: &[u8]| {
        let path = dir.path().join(name);
        std::fs::write(&path, bytes).unwrap();
        load_cover_file(&path, &limits(), 64)
    };
    let png = red_png();
    assert_eq!(
        load("truncated.png", &png[..png.len() / 2]).unwrap_err(),
        CoverError::Undecodable,
        "a PNG cut in half"
    );
    assert_eq!(
        load("noise.jpg", b"not an image at all").unwrap_err(),
        CoverError::UnsupportedFormat
    );
    assert_eq!(
        load("empty.png", b"").unwrap_err(),
        CoverError::UnsupportedFormat
    );
    assert_eq!(
        load("anim.gif", b"GIF89a\x01\x00\x01\x00\x00\x00\x00;").unwrap_err(),
        CoverError::UnsupportedFormat,
        "only JPEG and PNG"
    );
    assert!(matches!(
        load_cover_file(&dir.path().join("missing.png"), &limits(), 64).unwrap_err(),
        CoverError::Unreadable(_)
    ));
    assert!(matches!(
        load_cover_file(dir.path(), &limits(), 64).unwrap_err(),
        CoverError::Unreadable(_)
    ));
}

#[test]
fn a_valid_image_loads_as_a_front_cover_with_a_thumbnail() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("cover.jpg");
    std::fs::write(&path, blue_jpeg()).unwrap();
    let cover = load_cover_file(&path, &limits(), 16).unwrap();
    assert!(cover.is_front());
    assert_eq!(cover.data(), blue_jpeg().as_slice(), "the file as it is");
    let thumb =
        image::load_from_memory_with_format(cover.thumb_png().unwrap(), image::ImageFormat::Png)
            .unwrap();
    assert!(thumb.width() <= 16 && thumb.height() <= 16);
}

#[test]
fn an_unusable_cover_is_refused_before_the_disk_is_touched() {
    let dir = tempfile::tempdir().unwrap();
    let path = mp3_with(
        dir.path(),
        vec![(red_png(), PictureType::CoverFront, MimeType::Png)],
    );
    let original = std::fs::read(&path).unwrap();
    let before = sheet(&path);
    let png = red_png();
    for (bad, why) in [
        (b"junk".to_vec(), CoverError::UnsupportedFormat),
        (png[..png.len() / 2].to_vec(), CoverError::Undecodable),
    ] {
        let mut after = before.clone();
        after.set_cover(Some(new_cover(bad)));
        assert_eq!(
            write_tag_sheet(&path, &before, &after, &limits()),
            Err(TagWriteError::InvalidCover(why))
        );
    }
    let small = Limits {
        max_cover_bytes: 100,
        ..limits()
    };
    let mut after = before.clone();
    after.set_cover(Some(new_cover(blue_jpeg())));
    assert_eq!(
        write_tag_sheet(&path, &before, &after, &small),
        Err(TagWriteError::InvalidCover(CoverError::TooLarge))
    );
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert_eq!(names(dir.path()), ["x.mp3"]);
}

#[test]
fn a_cover_that_does_not_decode_is_shown_without_a_thumbnail_and_kept() {
    let dir = tempfile::tempdir().unwrap();
    let broken = red_png()[..30].to_vec();
    let path = mp3_with(
        dir.path(),
        vec![(broken.clone(), PictureType::CoverFront, MimeType::Png)],
    );
    let s = with_cover_thumbnail(sheet(&path), &limits(), 64);
    let cover = s.cover().expect("it is still the cover");
    assert!(cover.thumb_png().is_none());
    // An edit of another field leaves it as it is.
    edit(&path, |s| s.set_text(TagField::Genre, "Pop", true));
    assert_eq!(pictures_of(&path), [(PictureType::CoverFront, broken)]);
}

#[test]
fn the_thumbnail_of_a_cover_is_a_small_png() {
    let dir = tempfile::tempdir().unwrap();
    let path = mp3_with(
        dir.path(),
        vec![(red_png(), PictureType::CoverFront, MimeType::Png)],
    );
    let plain = sheet(&path);
    assert!(plain.cover().unwrap().thumb_png().is_none(), "not yet");
    let s = with_cover_thumbnail(plain.clone(), &limits(), 20);
    let thumb = image::load_from_memory_with_format(
        s.cover().unwrap().thumb_png().unwrap(),
        image::ImageFormat::Png,
    )
    .unwrap();
    assert!(thumb.width() <= 20 && thumb.height() <= 20);
    assert_eq!(s, plain, "the thumbnail does not make the sheet differ");
    let none = with_cover_thumbnail(sheet(&mp3(dir.path(), "none.mp3")), &limits(), 20);
    assert!(none.cover().is_none());
}

#[test]
fn a_summary_write_keeps_the_cover() {
    let dir = tempfile::tempdir().unwrap();
    let path = mp3_with(
        dir.path(),
        vec![(red_png(), PictureType::CoverFront, MimeType::Png)],
    );
    let before = fp_analysis::tags::read_track_tags(&path, &limits());
    let after = fp_model::TrackTags {
        title: "Another".into(),
        ..before.clone()
    };
    fp_analysis::tags::write_tags(&path, &before, &after, &limits()).unwrap();
    assert_eq!(pictures_of(&path), [(PictureType::CoverFront, red_png())]);
}

/// A FLAC file with a STREAMINFO block and a stub where the audio frames
/// go: enough for lofty to read and write its Vorbis comments and pictures.
fn flac(dir: &Path) -> PathBuf {
    let path = dir.join("x.flac");
    let mut bytes = b"fLaC".to_vec();
    // The last metadata block: STREAMINFO, 34 bytes.
    bytes.extend_from_slice(&[0x80, 0, 0, 34]);
    bytes.extend_from_slice(&4096u16.to_be_bytes()); // minimum block size
    bytes.extend_from_slice(&4096u16.to_be_bytes()); // maximum block size
    bytes.extend_from_slice(&[0, 0, 0, 0, 0, 0]); // frame sizes (unknown)
    // 44.1 kHz (20 bits), 2 channels (3 bits), 16 bits (5 bits), 0 samples.
    let packed: u64 = (44_100u64 << 44) | (1u64 << 41) | (15u64 << 36);
    bytes.extend_from_slice(&packed.to_be_bytes());
    bytes.extend_from_slice(&[0u8; 16]); // MD5
    // Something after the metadata, as in any real file.
    bytes.extend_from_slice(&[0xFF, 0xF8, 0, 0, 0, 0, 0, 0]);
    std::fs::write(&path, bytes).unwrap();
    path
}

#[test]
fn a_flac_file_stores_its_cover_as_a_picture_block() {
    let dir = tempfile::tempdir().unwrap();
    let path = flac(dir.path());
    let s = sheet(&path);
    assert!(s.can_store_cover());
    assert!(s.cover().is_none());
    let after = edit(&path, |s| s.set_cover(Some(new_cover(red_png()))));
    assert_eq!(after.cover().unwrap().data(), red_png().as_slice());
    let again = edit(&path, |s| s.set_cover(Some(new_cover(blue_jpeg()))));
    assert_eq!(again.cover().unwrap().data(), blue_jpeg().as_slice());
    let removed = edit(&path, |s| {
        let original = s.clone();
        s.remove_front_cover(&original);
    });
    assert!(removed.cover().is_none());
}
