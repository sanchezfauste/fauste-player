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
    pub year: Option<u32>,
    pub genre: Option<String>,
    pub album_artist: Option<String>,
    pub composer: Option<String>,
    pub comment: Option<String>,
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
    let parsed = lofty::read_from_path(path)
        .or_else(|_| {
            lofty::probe::Probe::open(path)?
                .options(lofty::config::ParseOptions::new().read_cover_art(false))
                .guess_file_type()?
                .read()
        })
        .or_else(|e| match dsf_tag_as_wav(path, limit as u64) {
            Some(wav) => lofty::probe::Probe::new(std::io::Cursor::new(wav))
                // The wrapper has no audio to describe.
                .options(lofty::config::ParseOptions::new().read_properties(false))
                .guess_file_type()?
                .read(),
            None => Err(e),
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
        year: tag
            .date()
            .map(|d| u32::from(d.year))
            .filter(|y| (1..=9999).contains(y)),
        genre: non_empty(tag.genre()),
        album_artist: non_empty(
            tag.get_string(lofty::tag::ItemKey::AlbumArtist)
                .map(Into::into),
        ),
        composer: non_empty(
            tag.get_string(lofty::tag::ItemKey::Composer)
                .map(Into::into),
        ),
        comment: non_empty(tag.comment()),
        cover,
    }
}

/// The ID3v2 chunk of a DSF file, which the DSF header points to, wrapped
/// in a WAV container so that lofty reads it like any other ID3v2 tag
/// (lofty does not read DSF itself). At most `max_bytes` are read.
fn dsf_tag_as_wav(path: &Path, max_bytes: u64) -> Option<Vec<u8>> {
    use std::io::{Read, Seek, SeekFrom};

    let mut file = std::fs::File::open(path).ok()?;
    let mut header = [0u8; 28];
    file.read_exact(&mut header).ok()?;
    if header.get(0..4)? != b"DSD " {
        return None;
    }
    let at = u64::from_le_bytes(header.get(20..28)?.try_into().ok()?);
    let len = file.metadata().ok()?.len();
    if at == 0 || at >= len {
        return None;
    }
    file.seek(SeekFrom::Start(at)).ok()?;
    let mut id3 = Vec::new();
    file.take((len - at).min(max_bytes))
        .read_to_end(&mut id3)
        .ok()?;
    if !id3.starts_with(b"ID3") {
        return None;
    }
    // A 16-bit mono PCM format chunk, no audio, and the tag.
    let mut fmt = Vec::new();
    fmt.extend(1u16.to_le_bytes());
    fmt.extend(1u16.to_le_bytes());
    fmt.extend(44_100u32.to_le_bytes());
    fmt.extend(88_200u32.to_le_bytes());
    fmt.extend(2u16.to_le_bytes());
    fmt.extend(16u16.to_le_bytes());
    let pad = id3.len() % 2;
    let tag_len = u32::try_from(id3.len()).ok()?;
    let riff_len = 4 + (8 + 16) + 8 + 8 + tag_len + u32::try_from(pad).ok()?;
    let mut wav = Vec::with_capacity(riff_len as usize + 8);
    wav.extend(b"RIFF");
    wav.extend(riff_len.to_le_bytes());
    wav.extend(b"WAVEfmt ");
    wav.extend(16u32.to_le_bytes());
    wav.extend(fmt);
    wav.extend(b"data");
    wav.extend(0u32.to_le_bytes());
    wav.extend(b"ID3 ");
    wav.extend(tag_len.to_le_bytes());
    wav.extend(id3);
    wav.extend(std::iter::repeat_n(0u8, pad));
    Some(wav)
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
    // Tools in some locales write a decimal comma.
    let text = text.trim().replace(',', ".");
    if text.is_empty() {
        return None;
    }
    let mut total = 0.0;
    let parts: Vec<&str> = text.split(':').collect();
    if parts.len() > 3 {
        return None;
    }
    let last = parts.len() - 1;
    for (i, part) in parts.iter().enumerate() {
        // Only the seconds field may have decimals.
        let allowed = |c: char| c.is_ascii_digit() || (i == last && c == '.');
        if part.is_empty() || !part.chars().all(allowed) {
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

/// Most bytes read from a DSF file's tag when looking for `INTRO`.
const DSF_INTRO_TAG_BYTES: u64 = 16 * 1024 * 1024;

/// The intro time from an `INTRO` tag: ID3v2 `TXXX:INTRO`, a Vorbis, Opus or
/// FLAC comment, an APE item (WavPack, Monkey's Audio, MP3), or an MP4
/// freeform `----:com.apple.iTunes:INTRO`. Anything unreadable is `None`.
pub fn read_intro(path: &Path) -> Option<f64> {
    use lofty::ape::ApeTag;
    use lofty::file::{AudioFile, FileType};
    use lofty::id3::v2::Id3v2Tag;
    use lofty::mp4::{AtomData, AtomIdent};
    use lofty::tag::ItemValue;

    // TXXX descriptions are matched without regard to case, like Vorbis
    // comment and APE keys: tools write "Intro" as often as "INTRO".
    let from_id3 = |t: Option<&Id3v2Tag>| {
        t.into_iter().flatten().find_map(|frame| match frame {
            lofty::id3::v2::Frame::UserText(f) if f.description.eq_ignore_ascii_case(INTRO_KEY) => {
                Some(f.content.to_string())
            }
            _ => None,
        })
    };
    let from_ape = |t: Option<&ApeTag>| {
        t.and_then(|t| t.get(INTRO_KEY))
            .and_then(|i| match i.value() {
                ItemValue::Text(s) => Some(s.clone()),
                _ => None,
            })
    };
    let result = std::panic::catch_unwind(|| -> Option<String> {
        // Covers are not needed here, and a huge one must not hide the tag.
        let options = lofty::config::ParseOptions::new()
            .read_properties(false)
            .read_cover_art(false);
        // DSF: the ID3v2 chunk, which lofty reads once it is in a WAV.
        if let Some(wav) = dsf_tag_as_wav(path, DSF_INTRO_TAG_BYTES) {
            let f = lofty::iff::wav::WavFile::read_from(&mut std::io::Cursor::new(wav), options)
                .ok()?;
            return from_id3(f.id3v2());
        }
        let probe = lofty::probe::Probe::open(path)
            .ok()?
            .guess_file_type()
            .ok()?;
        let file_type = probe.file_type()?;
        let mut reader = std::fs::File::open(path).ok()?;
        match file_type {
            FileType::Opus => {
                let f = lofty::ogg::OpusFile::read_from(&mut reader, options).ok()?;
                f.vorbis_comments().get(INTRO_KEY).map(str::to_owned)
            }
            FileType::WavPack => {
                let f = lofty::wavpack::WavPackFile::read_from(&mut reader, options).ok()?;
                from_ape(f.ape())
            }
            FileType::Ape => {
                let f = lofty::ape::ApeFile::read_from(&mut reader, options).ok()?;
                from_ape(f.ape()).or_else(|| from_id3(f.id3v2()))
            }
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
