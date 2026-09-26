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
pub fn read_tags(path: &Path, limits: &Limits) -> Tags {
    // lofty refuses tag blocks larger than its own allocation limit (16 MiB),
    // which would drop every tag of a file with a large cover. Align it with
    // the configured cover limit (lofty's options are per thread).
    let default_limit = 16 * 1024 * 1024;
    let limit = usize::try_from(limits.max_cover_bytes)
        .unwrap_or(usize::MAX)
        .saturating_add(1024 * 1024);
    lofty::config::apply_global_options(
        lofty::config::GlobalOptions::new().allocation_limit(limit.max(default_limit)),
    );
    // If parsing still fails, retry without pictures so the text tags survive.
    let parsed = lofty::read_from_path(path).or_else(|_| {
        lofty::probe::Probe::open(path)?
            .options(lofty::config::ParseOptions::new().read_cover_art(false))
            .guess_file_type()?
            .read()
    });
    let Ok(file) = parsed else {
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

/// The description / key of the intro tag (Phase 2 spec P2.8).
const INTRO_KEY: &str = "INTRO";

/// Parses an intro time: seconds (`12.5`) or `m:ss(.f)` / `h:mm:ss(.f)`.
pub fn parse_intro_time(text: &str) -> Option<f64> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    let mut total = 0.0;
    let parts: Vec<&str> = text.split(':').collect();
    if parts.len() > 3 {
        return None;
    }
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() || !part.chars().all(|c| c.is_ascii_digit() || c == '.') {
            return None;
        }
        let value: f64 = part.parse().ok()?;
        // Minutes and seconds after the first field stay below 60.
        if i > 0 && value >= 60.0 {
            return None;
        }
        total = total * 60.0 + value;
    }
    total.is_finite().then_some(total)
}

/// The intro time from an `INTRO` tag: ID3v2 `TXXX:INTRO`, a Vorbis or FLAC
/// comment, an APE item, or an MP4 freeform `----:com.apple.iTunes:INTRO`.
/// Anything unreadable is `None`.
pub fn read_intro(path: &Path) -> Option<f64> {
    use lofty::ape::ApeTag;
    use lofty::file::{AudioFile, FileType};
    use lofty::id3::v2::Id3v2Tag;
    use lofty::mp4::{AtomData, AtomIdent};
    use lofty::tag::ItemValue;

    let from_id3 = |t: Option<&Id3v2Tag>| {
        t.and_then(|t| t.get_user_text(INTRO_KEY))
            .map(str::to_owned)
    };
    let from_ape = |t: Option<&ApeTag>| {
        t.and_then(|t| t.get(INTRO_KEY))
            .and_then(|i| match i.value() {
                ItemValue::Text(s) => Some(s.clone()),
                _ => None,
            })
    };
    let result = std::panic::catch_unwind(|| -> Option<String> {
        let probe = lofty::probe::Probe::open(path)
            .ok()?
            .guess_file_type()
            .ok()?;
        let file_type = probe.file_type()?;
        let mut reader = std::fs::File::open(path).ok()?;
        let options = lofty::config::ParseOptions::new().read_properties(false);
        match file_type {
            FileType::Flac => {
                let f = lofty::flac::FlacFile::read_from(&mut reader, options).ok()?;
                f.vorbis_comments()
                    .and_then(|v| v.get(INTRO_KEY))
                    .map(str::to_owned)
                    .or_else(|| from_id3(f.id3v2()))
            }
            FileType::Vorbis => {
                let f = lofty::ogg::VorbisFile::read_from(&mut reader, options).ok()?;
                f.vorbis_comments().get(INTRO_KEY).map(str::to_owned)
            }
            FileType::Mpeg => {
                let f = lofty::mpeg::MpegFile::read_from(&mut reader, options).ok()?;
                from_id3(f.id3v2()).or_else(|| from_ape(f.ape()))
            }
            FileType::Wav => {
                let f = lofty::iff::wav::WavFile::read_from(&mut reader, options).ok()?;
                from_id3(f.id3v2())
            }
            FileType::Aiff => {
                let f = lofty::iff::aiff::AiffFile::read_from(&mut reader, options).ok()?;
                from_id3(f.id3v2())
            }
            FileType::Mp4 => {
                let f = lofty::mp4::Mp4File::read_from(&mut reader, options).ok()?;
                let ident = AtomIdent::Freeform {
                    mean: "com.apple.iTunes".into(),
                    name: INTRO_KEY.into(),
                };
                f.ilst()
                    .and_then(|i| i.get(&ident))
                    .and_then(|a| a.data().next())
                    .and_then(|d| match d {
                        AtomData::UTF8(s) => Some(s.clone()),
                        _ => None,
                    })
            }
            _ => None,
        }
    });
    result.ok().flatten().as_deref().and_then(parse_intro_time)
}
