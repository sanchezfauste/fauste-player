//! Tags and cover art. Every failure here degrades to "not available": a
//! file with broken tags or a hostile cover image still plays and analyses.

use std::io::Cursor;
use std::path::Path;

use fp_model::Limits;
use lofty::file::TaggedFileExt;
use lofty::tag::Accessor;

/// What the file's tags say. `None` means the tag is absent.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Tags {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    /// Raw bytes of the front cover (or the first picture).
    pub cover: Option<Vec<u8>>,
}

fn non_empty(s: Option<std::borrow::Cow<'_, str>>) -> Option<String> {
    s.map(|v| v.trim().to_owned()).filter(|v| !v.is_empty())
}

/// Reads tags with lofty. Unreadable or untagged files yield empty `Tags`.
pub fn read_tags(path: &Path) -> Tags {
    let Ok(file) = lofty::read_from_path(path) else {
        return Tags::default();
    };
    let Some(tag) = file.primary_tag().or_else(|| file.first_tag()) else {
        return Tags::default();
    };
    let pictures = tag.pictures();
    let cover = pictures
        .iter()
        .find(|p| p.pic_type() == lofty::picture::PictureType::CoverFront)
        .or_else(|| pictures.first())
        .map(|p| p.data().to_vec());
    Tags {
        title: non_empty(tag.title()),
        artist: non_empty(tag.artist()),
        album: non_empty(tag.album()),
        cover,
    }
}

/// `Artist - Title` from the file name; without a separator the whole stem
/// is the title.
pub fn title_from_file_name(path: &Path) -> (Option<String>, String) {
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    match stem.split_once(" - ") {
        Some((artist, title)) if !artist.trim().is_empty() && !title.trim().is_empty() => {
            (Some(artist.trim().to_owned()), title.trim().to_owned())
        }
        _ => (None, stem.trim().to_owned()),
    }
}

/// A PNG thumbnail (at most `px`×`px`, aspect preserved) of an embedded
/// cover, decoded under the configured limits. `None` on any problem.
pub fn thumbnail_png(bytes: &[u8], limits: &Limits, px: u32) -> Option<Vec<u8>> {
    if u64::try_from(bytes.len()).ok()? > limits.max_cover_bytes {
        return None;
    }
    let mut reader = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .ok()?;
    let side = limits.max_cover_pixels;
    let mut decode_limits = image::Limits::default();
    decode_limits.max_image_width = Some(side);
    decode_limits.max_image_height = Some(side);
    // RGBA at the maximum size: the most a legitimate cover can need.
    decode_limits.max_alloc = Some(u64::from(side) * u64::from(side) * 4);
    reader.limits(decode_limits);
    let image = reader.decode().ok()?;
    let thumb = image.thumbnail(px, px);
    let mut out = Vec::new();
    thumb
        .write_to(&mut Cursor::new(&mut out), image::ImageFormat::Png)
        .ok()?;
    Some(out)
}
