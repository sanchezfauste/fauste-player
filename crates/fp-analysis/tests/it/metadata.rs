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

/// A one-block stereo DSF file of DSD silence with `id3` as its metadata
/// chunk.
fn dsf_with_tag(dir: &Path, id3: &[u8]) -> PathBuf {
    let data = vec![0x69u8; 2 * 4096];
    let data_end = 92 + data.len() as u64;
    let total = data_end + id3.len() as u64;
    let mut f = Vec::new();
    f.extend(b"DSD ");
    f.extend(28u64.to_le_bytes());
    f.extend(total.to_le_bytes());
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
    let path = dir.join("tagged.dsf");
    std::fs::write(&path, f).unwrap();
    path
}

#[test]
fn dsf_tags_are_read() {
    let dir = tempfile::tempdir().unwrap();
    let mut tag = lofty::id3::v2::Id3v2Tag::new();
    tag.set_title("Slow Tide".into());
    tag.set_artist("The Harbour".into());
    tag.set_album("Night Ferry".into());
    let mut id3 = Vec::new();
    tag.dump_to(&mut id3, WriteOptions::default()).unwrap();
    let path = dsf_with_tag(dir.path(), &id3);
    let tags = read_tags(&path, &Limits::default());
    assert_eq!(tags.title.as_deref(), Some("Slow Tide"));
    assert_eq!(tags.artist.as_deref(), Some("The Harbour"));
    assert_eq!(tags.album.as_deref(), Some("Night Ferry"));

    // A metadata pointer past the end, or garbage there, is no tag.
    let path = dsf_with_tag(dir.path(), b"not a tag at all");
    assert_eq!(read_tags(&path, &Limits::default()).title, None);
}
