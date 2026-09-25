#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Tags and cover art (spec §6).

use std::io::Cursor;
use std::path::{Path, PathBuf};

use fp_analysis::metadata::{read_tags, thumbnail_png, title_from_file_name};
use fp_model::Limits;
use lofty::config::WriteOptions;
use lofty::picture::{MimeType, Picture, PictureType};
use lofty::prelude::*;
use lofty::tag::Tag;

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

fn png(width: u32, height: u32) -> Vec<u8> {
    let img = image::RgbImage::from_pixel(width, height, image::Rgb([200, 30, 30]));
    let mut out = Vec::new();
    image::DynamicImage::ImageRgb8(img)
        .write_to(&mut Cursor::new(&mut out), image::ImageFormat::Png)
        .unwrap();
    out
}

fn tag(path: &Path, cover: Option<Vec<u8>>) {
    let mut tagged = lofty::read_from_path(path).unwrap();
    let ty = tagged.primary_tag_type();
    if tagged.primary_tag().is_none() {
        tagged.insert_tag(Tag::new(ty));
    }
    let t = tagged.primary_tag_mut().unwrap();
    t.set_title("Song Title".into());
    t.set_artist("The Artist".into());
    t.set_album("An Album".into());
    if let Some(bytes) = cover {
        t.push_picture(
            Picture::unchecked(bytes)
                .pic_type(PictureType::CoverFront)
                .mime_type(MimeType::Png)
                .build(),
        );
    }
    tagged.save_to_path(path, WriteOptions::default()).unwrap();
}

#[test]
fn tags_and_cover_are_read() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "x.wav");
    tag(&path, Some(png(300, 200)));
    let tags = read_tags(&path, &Limits::default());
    assert_eq!(tags.title.as_deref(), Some("Song Title"));
    assert_eq!(tags.artist.as_deref(), Some("The Artist"));
    assert_eq!(tags.album.as_deref(), Some("An Album"));
    let thumb = thumbnail_png(&tags.cover.unwrap(), &Limits::default(), 128).unwrap();
    let decoded = image::load_from_memory(&thumb).unwrap();
    assert_eq!((decoded.width(), decoded.height()), (128, 85));
}

#[test]
fn untagged_files_fall_back_to_the_file_name() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "Marta Oliva - Carretera norte.wav");
    let tags = read_tags(&path, &Limits::default());
    assert_eq!((tags.title, tags.artist, tags.cover), (None, None, None));
    assert_eq!(
        title_from_file_name(&path),
        (Some("Marta Oliva".to_owned()), "Carretera norte".to_owned())
    );
    assert_eq!(
        title_from_file_name(Path::new("/m/Just a title.flac")),
        (None, "Just a title".to_owned())
    );
}

#[test]
fn oversized_covers_are_ignored() {
    let mut limits = Limits {
        max_cover_pixels: 8_000,
        ..Limits::default()
    };
    assert_eq!(
        thumbnail_png(&png(9_000, 10), &limits, 128),
        None,
        "too many pixels on one side"
    );
    limits.max_cover_bytes = 100;
    assert_eq!(
        thumbnail_png(&png(300, 200), &limits, 128),
        None,
        "too many bytes"
    );
}

#[test]
fn corrupt_cover_bytes_are_ignored() {
    assert_eq!(
        thumbnail_png(b"\x89PNG not really", &Limits::default(), 128),
        None
    );
    assert_eq!(
        read_tags(Path::new("/definitely/missing.mp3"), &Limits::default()).title,
        None
    );
}
