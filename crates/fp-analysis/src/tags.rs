//! Reading and writing the text tags of a track (feedback 2 spec O23).
//!
//! Reading degrades to "nothing" like `metadata::read_tags`. Writing never
//! touches the original until the new file is complete: the tags go into a
//! copy next to it, which is synced and then renamed over the original.

use std::path::{Path, PathBuf};

use fp_model::{
    CoverArt, Limits, TagField, TagFieldKind, TagSheet, TrackTags, changed_fields, cover_blocked,
    cover_changed, invalid_fields,
};
use lofty::config::WriteOptions;
use lofty::file::{FileType, TaggedFile, TaggedFileExt};
use lofty::picture::{MimeType, Picture, PictureType};
use lofty::prelude::*;
use lofty::tag::items::Timestamp;
use lofty::tag::{ItemKey, ItemValue, Tag, TagItem, TagType};

use crate::metadata::{display_picture, read_tags, tag_date, thumbnail_png, title_from_file_name};

/// Why tags could not be written. The original file is untouched in every
/// case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagWriteError {
    /// The format has no writable tags.
    Unsupported,
    /// The file or its folder is gone.
    NotFound,
    /// No permission to write there.
    Denied,
    /// The date is not an ISO 8601 date (`fp_model::parse_tag_date`).
    InvalidDate,
    /// The value of this field is not valid (`fp_model::invalid_fields`).
    InvalidField(TagField),
    /// The file's format cannot store a picture, so the cover cannot change.
    CoverNotStorable,
    /// The new cover is not a JPEG or PNG image that decodes within the
    /// limits.
    InvalidCover(CoverError),
    /// Anything else, with the system's or lofty's own description.
    Other(String),
}

impl std::fmt::Display for TagWriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported => f.write_str("this format has no writable tags"),
            Self::NotFound => f.write_str("the file was not found"),
            Self::Denied => f.write_str("permission denied"),
            Self::InvalidDate => f.write_str("the date is not an ISO 8601 date"),
            Self::InvalidField(field) => write!(f, "the {} is not valid", field.slug()),
            Self::CoverNotStorable => f.write_str("this format cannot store a cover"),
            Self::InvalidCover(why) => write!(f, "the cover is not usable: {why}"),
            Self::Other(detail) => f.write_str(detail),
        }
    }
}

impl From<std::io::Error> for TagWriteError {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::NotFound => Self::NotFound,
            std::io::ErrorKind::PermissionDenied => Self::Denied,
            _ => Self::Other(e.to_string()),
        }
    }
}

fn lofty_error(e: impl std::fmt::Display) -> TagWriteError {
    TagWriteError::Other(e.to_string())
}

/// The tags of `path` as the player shows them: the file name stands in for
/// a missing title (and its `Artist - ` for a missing artist), the other
/// fields stay empty. Never fails; a panic in the tag parser costs only the
/// tags.
pub fn read_track_tags(path: &Path, limits: &Limits) -> TrackTags {
    let tags = std::panic::catch_unwind(|| read_tags(path, limits)).unwrap_or_default();
    let (file_artist, file_title) = title_from_file_name(path);
    TrackTags {
        title: tags.title.unwrap_or(file_title),
        artist: tags.artist.or(file_artist).unwrap_or_default(),
        album: tags.album.unwrap_or_default(),
        album_artist: tags.album_artist.unwrap_or_default(),
        date: tags.date,
        genre: tags.genre.unwrap_or_default(),
        composer: tags.composer.unwrap_or_default(),
        comment: tags.comment.unwrap_or_default(),
    }
    .clamped(limits.max_tag_chars)
}

/// Whether the format of `path` has writable tags in lofty, judged from the
/// extension only: no I/O, so the interface can ask on its own thread.
pub fn can_write_tags(path: &Path) -> bool {
    FileType::from_path(path).is_some_and(|ft| ft.tag_support(ft.primary_tag_type()).is_writable())
}

/// Writes the fields where `after` differs from `before` into the file's
/// tags: text is set, an empty field removes the tag. Other fields, other
/// tags, covers and unknown items are kept.
///
/// 1. the file is copied to a temporary file in the same folder;
/// 2. the copy gets the new tags;
/// 3. the copy is synced to disk;
/// 4. the copy is renamed over the original.
///
/// Any error removes the copy and leaves the original as it was.
pub fn write_tags(
    path: &Path,
    before: &TrackTags,
    after: &TrackTags,
    limits: &Limits,
) -> Result<(), TagWriteError> {
    let after = after.clone().clamped(limits.max_tag_chars);
    safe_edit(path, limits, |tag| change_tag(tag, before, &after))
}

/// The one safe way to change a file's tag: the four steps of `write_tags`
/// around `edit`, which receives the tag the editor works on.
fn safe_edit(
    path: &Path,
    limits: &Limits,
    edit: impl FnOnce(&mut Tag) -> Result<(), TagWriteError>,
) -> Result<(), TagWriteError> {
    // A symlink is written through, not replaced.
    let real = path.canonicalize()?;
    if !can_write_tags(&real) {
        return Err(TagWriteError::Unsupported);
    }
    let temp = temp_path(&real).ok_or_else(|| lofty_error("the file has no name"))?;
    let result = rewrite(&real, &temp, limits, edit);
    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    result
}

/// `name.fptag-<pid>.ext` next to `real`: the extension stays, so the format
/// is recognised the same way.
fn temp_path(real: &Path) -> Option<PathBuf> {
    let stem = real.file_stem()?.to_string_lossy();
    let name = match real.extension() {
        Some(ext) => format!(
            "{stem}.fptag-{}.{}",
            std::process::id(),
            ext.to_string_lossy()
        ),
        None => format!("{stem}.fptag-{}", std::process::id()),
    };
    Some(real.with_file_name(name))
}

/// Lets lofty parse tags up to the configured cover size: its own limit is
/// 16 MiB, which would drop every tag of a file with a large cover. Its
/// options are per thread, so every reader and writer calls this first.
fn allow_large_tags(limits: &Limits) {
    let limit = usize::try_from(limits.max_cover_bytes)
        .unwrap_or(usize::MAX)
        .saturating_add(1024 * 1024)
        .max(16 * 1024 * 1024);
    lofty::config::apply_global_options(
        lofty::config::GlobalOptions::new().allocation_limit(limit),
    );
}

/// The tag the editor shows and edits: the primary one of the format, else
/// the first (the tag `read_tags` reads).
fn edited_tag(file: &TaggedFile) -> Option<&Tag> {
    file.primary_tag().or_else(|| file.first_tag())
}

fn edited_tag_mut(file: &mut TaggedFile) -> Option<&mut Tag> {
    if file.primary_tag().is_some() {
        file.primary_tag_mut()
    } else {
        file.first_tag_mut()
    }
}

fn rewrite(
    real: &Path,
    temp: &Path,
    limits: &Limits,
    edit: impl FnOnce(&mut Tag) -> Result<(), TagWriteError>,
) -> Result<(), TagWriteError> {
    std::fs::copy(real, temp)?;
    // Parsed with its covers: if that fails the write fails, because saving
    // a tag read without them would drop them.
    allow_large_tags(limits);
    let mut file = std::panic::catch_unwind(|| lofty::read_from_path(temp))
        .map_err(|_| lofty_error("the tag parser failed"))?
        .map_err(lofty_error)?;
    if edited_tag(&file).is_none() {
        let ty = file.primary_tag_type();
        file.insert_tag(Tag::new(ty));
    }
    let tag =
        edited_tag_mut(&mut file).ok_or_else(|| lofty_error("the file has no tag to edit"))?;
    urls_as_text(tag);
    edit(tag)?;
    file.save_to_path(temp, WriteOptions::default())
        .map_err(lofty_error)?;
    std::fs::OpenOptions::new()
        .write(true)
        .open(temp)?
        .sync_all()?;
    std::fs::rename(temp, real)?;
    Ok(())
}

/// ID3v2 URL frames (`WOAR`, `WCOM`, ...) come out of a parsed file as
/// `ItemValue::Locator`, and lofty 0.25.4 drops a locator when it builds the
/// frames of a saved tag. Re-adding them as text makes it write them back as
/// URL frames, so a save never loses a URL the editor does not show.
fn urls_as_text(tag: &mut Tag) {
    if tag.tag_type() != TagType::Id3v2 {
        return;
    }
    let mut keys = Vec::new();
    for item in tag.items() {
        if item.value().locator().is_some() && !keys.contains(&item.key()) {
            keys.push(item.key());
        }
    }
    for key in keys {
        let urls: Vec<String> = tag
            .get_items(key)
            .filter_map(item_text)
            .map(str::to_owned)
            .collect();
        // Not `Tag::take`: it swaps items around, which would reorder the
        // values of the other fields.
        tag.remove_key(key);
        for url in urls {
            tag.push(TagItem::new(key, ItemValue::Text(url)));
        }
    }
}

fn change_tag(tag: &mut Tag, before: &TrackTags, after: &TrackTags) -> Result<(), TagWriteError> {
    fn text(tag: &mut Tag, old: &str, new: &str, set: fn(&mut Tag, String), remove: fn(&mut Tag)) {
        if old == new {
            return;
        }
        if new.is_empty() {
            remove(tag);
        } else {
            set(tag, new.to_owned());
        }
    }
    fn item(tag: &mut Tag, key: ItemKey, old: &str, new: &str) {
        if old == new {
            return;
        }
        if new.is_empty() {
            tag.remove_key(key);
        } else {
            tag.insert_text(key, new.to_owned());
        }
    }
    text(
        tag,
        &before.title,
        &after.title,
        |t, v| t.set_title(v),
        |t| t.remove_title(),
    );
    text(
        tag,
        &before.artist,
        &after.artist,
        |t, v| t.set_artist(v),
        |t| t.remove_artist(),
    );
    text(
        tag,
        &before.album,
        &after.album,
        |t, v| t.set_album(v),
        |t| t.remove_album(),
    );
    text(
        tag,
        &before.genre,
        &after.genre,
        |t, v| t.set_genre(v),
        |t| t.remove_genre(),
    );
    text(
        tag,
        &before.comment,
        &after.comment,
        |t, v| t.set_comment(v),
        |t| t.remove_comment(),
    );
    item(
        tag,
        ItemKey::AlbumArtist,
        &before.album_artist,
        &after.album_artist,
    );
    item(tag, ItemKey::Composer, &before.composer, &after.composer);
    if before.date != after.date {
        match &after.date {
            Some(text) => {
                let timestamp = text
                    .parse::<Timestamp>()
                    .map_err(|_| TagWriteError::InvalidDate)?;
                tag.set_date(timestamp);
            }
            None => tag.remove_date(),
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// The tag sheet: every field of the editor, read from and written to the tag.
// ---------------------------------------------------------------------------

/// The item keys a field may be stored under, best first. A tag type stores
/// the field under the first of them its format maps (BPM is `TBPM` in
/// ID3v2, which lofty calls `IntegerBpm`; lyrics are `USLT` there, which it
/// calls `UnsyncLyrics`).
fn candidates(field: TagField) -> &'static [ItemKey] {
    use TagField as F;
    match field {
        F::Title => &[ItemKey::TrackTitle],
        F::Artist => &[ItemKey::TrackArtist],
        F::Album => &[ItemKey::AlbumTitle],
        F::AlbumArtist => &[ItemKey::AlbumArtist],
        F::Date => &[ItemKey::RecordingDate],
        F::TrackNumber => &[ItemKey::TrackNumber],
        F::DiscNumber => &[ItemKey::DiscNumber],
        F::Genre => &[ItemKey::Genre],
        F::Composer => &[ItemKey::Composer],
        F::Comment => &[ItemKey::Comment],
        F::Subtitle => &[ItemKey::TrackSubtitle],
        F::Grouping => &[ItemKey::ContentGroup],
        F::Bpm => &[ItemKey::IntegerBpm, ItemKey::Bpm],
        F::InitialKey => &[ItemKey::InitialKey],
        F::Mood => &[ItemKey::Mood],
        F::Isrc => &[ItemKey::Isrc],
        F::Publisher => &[ItemKey::Publisher],
        F::CatalogNumber => &[ItemKey::CatalogNumber],
        F::Copyright => &[ItemKey::CopyrightMessage],
        F::OriginalArtist => &[ItemKey::OriginalArtist],
        F::OriginalAlbum => &[ItemKey::OriginalAlbumTitle],
        F::OriginalReleaseDate => &[ItemKey::OriginalReleaseDate],
        F::Lyricist => &[ItemKey::Lyricist],
        F::Conductor => &[ItemKey::Conductor],
        F::Remixer => &[ItemKey::Remixer],
        F::Arranger => &[ItemKey::Arranger],
        F::Performer => &[ItemKey::Performer],
        F::Language => &[ItemKey::Language],
        F::EncodedBy => &[ItemKey::EncodedBy],
        F::Lyrics => &[ItemKey::Lyrics, ItemKey::UnsyncLyrics],
        F::SortTitle => &[ItemKey::TrackTitleSortOrder],
        F::SortArtist => &[ItemKey::TrackArtistSortOrder],
        F::SortAlbum => &[ItemKey::AlbumTitleSortOrder],
        F::SortAlbumArtist => &[ItemKey::AlbumArtistSortOrder],
        F::SortComposer => &[ItemKey::ComposerSortOrder],
        F::ArtistWebsite => &[ItemKey::TrackArtistUrl],
    }
}

/// Where one field lives in one tag type.
#[derive(Debug, Clone, Copy)]
struct Slot {
    field: TagField,
    key: ItemKey,
    /// The total's key of a number and total field, if the type has one.
    total: Option<ItemKey>,
}

impl Slot {
    /// The slot of `field` in tags of `tag_type`, if the format can store it.
    fn of(field: TagField, tag_type: TagType) -> Option<Self> {
        let stores = |key: &ItemKey| ItemKey::supported_keys(tag_type).contains(key);
        let key = candidates(field).iter().copied().find(stores)?;
        let total = match field {
            TagField::TrackNumber => Some(ItemKey::TrackTotal),
            TagField::DiscNumber => Some(ItemKey::DiscTotal),
            _ => None,
        }
        .filter(stores);
        Some(Self { field, key, total })
    }

    /// Items of these keys, with no description, are the field's.
    fn owns(&self, item: &TagItem) -> bool {
        if !item.description().is_empty() {
            return false;
        }
        let key = item.key();
        key == self.key
            || self.total == Some(key)
            || (self.field == TagField::Date && key == ItemKey::Year)
    }
}

/// Every field `tag_type` can store, in editor order.
pub fn storable_fields(tag_type: TagType) -> Vec<TagField> {
    slots(tag_type).into_iter().map(|s| s.field).collect()
}

fn slots(tag_type: TagType) -> Vec<Slot> {
    TagField::ALL
        .into_iter()
        .filter_map(|field| Slot::of(field, tag_type))
        .collect()
}

/// The text of an item; an ID3v2 URL is a locator, not text.
fn item_text(item: &TagItem) -> Option<&str> {
    item.value().text().or_else(|| item.value().locator())
}

fn texts(tag: &Tag, slot: &Slot, key: ItemKey) -> Vec<String> {
    tag.get_items(key)
        .filter(|item| slot.owns(item))
        .filter_map(item_text)
        .map(str::to_owned)
        .collect()
}

fn read_slot(tag: &Tag, slot: &Slot) -> Vec<String> {
    match slot.field.kind() {
        TagFieldKind::Date if slot.field == TagField::Date => tag_date(tag).into_iter().collect(),
        TagFieldKind::Pair => {
            let number = texts(tag, slot, slot.key).into_iter().next();
            let total = slot
                .total
                .and_then(|key| texts(tag, slot, key).into_iter().next());
            // Some taggers write "3/12" into a number key of their own.
            match (number, total) {
                (Some(n), None) => match n.split_once('/') {
                    Some((n, t)) => vec![n.to_owned(), t.to_owned()],
                    None => vec![n, String::new()],
                },
                (n, t) => vec![n.unwrap_or_default(), t.unwrap_or_default()],
            }
        }
        _ => texts(tag, slot, slot.key),
    }
}

/// The sheet of `tag`: the values of the fields its format can store, its
/// front cover (or first picture), and how many other tags (other keys,
/// items with a description, pictures that are not a front cover) it keeps.
fn sheet_of(tag: &Tag, limits: &Limits) -> TagSheet {
    let slots = slots(tag.tag_type());
    let other_items = tag
        .items()
        .filter(|item| !slots.iter().any(|slot| slot.owns(item)))
        .count();
    // The front cover is the sheet's own; every other picture is kept.
    let other_pictures = tag
        .pictures()
        .iter()
        .filter(|p| p.pic_type() != PictureType::CoverFront)
        .count();
    let mut sheet = TagSheet::new(
        slots.iter().map(|s| s.field),
        other_items + other_pictures,
        tag.has_format_specific_items(),
    )
    .with_cover_support(can_store_pictures(tag.tag_type()));
    sheet.set_cover(
        display_picture(tag)
            .map(|p| CoverArt::new(p.data().to_vec(), p.pic_type() == PictureType::CoverFront)),
    );
    for slot in &slots {
        sheet.set_values(slot.field, read_slot(tag, slot));
    }
    sheet.clamped(limits.max_tag_chars, limits.max_tag_values)
}

/// The tag sheet of `path`, or `None` when the format has no writable tags
/// or the file cannot be parsed (a panic in the parser included). A file
/// with no tag yet gives an empty sheet of its format's primary tag type.
pub fn read_tag_sheet(path: &Path, limits: &Limits) -> Option<TagSheet> {
    if !can_write_tags(path) {
        return None;
    }
    allow_large_tags(limits);
    let file = std::panic::catch_unwind(|| lofty::read_from_path(path))
        .map_err(|_| tracing::warn!(path = %path.display(), "the tag parser failed"))
        .ok()?
        .map_err(|e| tracing::debug!(path = %path.display(), "cannot read the tags: {e}"))
        .ok()?;
    Some(match edited_tag(&file) {
        Some(tag) => sheet_of(tag, limits),
        None => sheet_of(&Tag::new(file.primary_tag_type()), limits),
    })
}

/// Writes the fields where `after` differs from `before` into the file,
/// through the same safe copy as `write_tags`. A changed field is replaced
/// by its new values, one item per value, or removed when it is empty. A
/// changed front cover is replaced or removed (every front-cover picture,
/// then the new one is added). Items the sheet does not own (other keys,
/// items with a description such as an ID3v2 comment with a description,
/// custom frames, pictures that are not a front cover) are not touched.
///
/// Refuses, before touching the disk, a sheet with a field that
/// `fp_model::invalid_fields` reports, a cover change in a format that has
/// no pictures, and a new cover that is not a usable JPEG or PNG.
pub fn write_tag_sheet(
    path: &Path,
    before: &TagSheet,
    after: &TagSheet,
    limits: &Limits,
) -> Result<(), TagWriteError> {
    let after = after
        .clone()
        .clamped(limits.max_tag_chars, limits.max_tag_values);
    if let Some(field) = invalid_fields(before, &after).first() {
        return Err(TagWriteError::InvalidField(*field));
    }
    if cover_blocked(before, &after) {
        return Err(TagWriteError::CoverNotStorable);
    }
    // The new cover is judged before the disk is touched.
    let new_cover = match after.cover().filter(|c| c.is_front()) {
        Some(cover) if cover_changed(before, &after) => {
            let mime = cover_mime(cover.data(), limits).map_err(TagWriteError::InvalidCover)?;
            Some((cover.clone(), mime))
        }
        _ => None,
    };
    safe_edit(path, limits, |tag| {
        change_sheet(tag, before, &after, new_cover.as_ref())
    })
}

fn change_sheet(
    tag: &mut Tag,
    before: &TagSheet,
    after: &TagSheet,
    new_cover: Option<&(CoverArt, MimeType)>,
) -> Result<(), TagWriteError> {
    if cover_changed(before, after) {
        change_cover(tag, new_cover)?;
    }
    for field in changed_fields(before, after) {
        let slot = Slot::of(field, tag.tag_type())
            .ok_or_else(|| lofty_error("the tag cannot store this field"))?;
        let values = after.values(field);
        match field.kind() {
            TagFieldKind::Date if field == TagField::Date => match values.first() {
                Some(text) => {
                    let timestamp = text
                        .parse::<Timestamp>()
                        .map_err(|_| TagWriteError::InvalidDate)?;
                    tag.set_date(timestamp);
                }
                None => tag.remove_date(),
            },
            TagFieldKind::Pair => {
                let (number, total) = after.pair(field);
                replace(tag, &slot, slot.key, &[number])?;
                if let Some(key) = slot.total {
                    replace(tag, &slot, key, &[total])?;
                }
            }
            _ => replace(tag, &slot, slot.key, values)?,
        }
    }
    Ok(())
}

/// Replaces the field's items under `key` by one item per non-empty value.
fn replace(
    tag: &mut Tag,
    slot: &Slot,
    key: ItemKey,
    values: &[String],
) -> Result<(), TagWriteError> {
    // Not `Tag::take_filter`: it swaps items around, which would reorder the
    // values of the fields that did not change.
    tag.retain(|item| !(item.key() == key && slot.owns(item)));
    for value in values.iter().filter(|v| !v.is_empty()) {
        if !tag.push(TagItem::new(key, ItemValue::Text(value.clone()))) {
            return Err(lofty_error("the tag cannot store this field"));
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// The front cover.
// ---------------------------------------------------------------------------

/// Whether tags of `tag_type` can hold a picture. (RIFF INFO, AIFF text and
/// ID3v1 have no place for one: lofty drops it silently on save, so the
/// editor must not offer it.)
pub fn can_store_pictures(tag_type: TagType) -> bool {
    matches!(
        tag_type,
        TagType::Id3v2 | TagType::VorbisComments | TagType::Mp4Ilst | TagType::Ape
    )
}

/// Why an image cannot be used as a cover.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoverError {
    /// The file is larger than `limits.max_cover_bytes`.
    TooLarge,
    /// Not a JPEG or PNG image.
    UnsupportedFormat,
    /// It looks like JPEG or PNG but does not decode within
    /// `limits.max_cover_pixels` (broken, truncated or too big).
    Undecodable,
    /// The file could not be read, with the system's description.
    Unreadable(String),
}

impl std::fmt::Display for CoverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooLarge => f.write_str("the image is too large"),
            Self::UnsupportedFormat => f.write_str("the image is not a JPEG or PNG"),
            Self::Undecodable => f.write_str("the image cannot be decoded"),
            Self::Unreadable(detail) => f.write_str(detail),
        }
    }
}

impl std::error::Error for CoverError {}

/// Checks that `bytes` is a JPEG or PNG within the cover limits that
/// decodes, with the safe decode path of the library's thumbnails, and
/// returns its picture type and its thumbnail (at most `px` pixels wide).
fn check_cover(bytes: &[u8], limits: &Limits, px: u32) -> Result<(MimeType, Vec<u8>), CoverError> {
    if u64::try_from(bytes.len()).map_err(|_| CoverError::TooLarge)? > limits.max_cover_bytes {
        return Err(CoverError::TooLarge);
    }
    let mime = match image::guess_format(bytes) {
        Ok(image::ImageFormat::Jpeg) => MimeType::Jpeg,
        Ok(image::ImageFormat::Png) => MimeType::Png,
        _ => return Err(CoverError::UnsupportedFormat),
    };
    let thumb = std::panic::catch_unwind(|| thumbnail_png(bytes, limits, px))
        .map_err(|_| CoverError::Undecodable)?
        .ok_or(CoverError::Undecodable)?;
    Ok((mime, thumb))
}

/// The MIME type of a cover that may be written (`check_cover` without the
/// thumbnail the caller does not need).
fn cover_mime(bytes: &[u8], limits: &Limits) -> Result<MimeType, CoverError> {
    check_cover(bytes, limits, 1).map(|(mime, _)| mime)
}

/// Reads the image at `path` as a new front cover, with a thumbnail of at
/// most `thumb_px` pixels for the editor. It must be a JPEG or PNG of at
/// most `limits.max_cover_bytes` that decodes within `limits.max_cover_pixels`.
pub fn load_cover_file(
    path: &Path,
    limits: &Limits,
    thumb_px: u32,
) -> Result<CoverArt, CoverError> {
    use std::io::Read;

    let unreadable = |e: std::io::Error| CoverError::Unreadable(e.to_string());
    let meta = std::fs::metadata(path).map_err(unreadable)?;
    // A pipe or a device would block or never end.
    if !meta.is_file() {
        return Err(CoverError::Unreadable("it is not a file".to_owned()));
    }
    if meta.len() > limits.max_cover_bytes {
        return Err(CoverError::TooLarge);
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(unreadable)?
        .take(limits.max_cover_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(unreadable)?;
    let (_, thumb) = check_cover(&bytes, limits, thumb_px)?;
    Ok(CoverArt::new(bytes, true).with_thumb(Some(thumb)))
}

/// `sheet` with the thumbnail of its cover decoded (at most `px` pixels),
/// under the cover limits and with a parser panic contained. A cover that
/// does not decode keeps no thumbnail and stays as it is in the file.
#[must_use]
pub fn with_cover_thumbnail(mut sheet: TagSheet, limits: &Limits, px: u32) -> TagSheet {
    if let Some(cover) = sheet.cover().cloned() {
        let thumb = std::panic::catch_unwind(|| thumbnail_png(cover.data(), limits, px))
            .ok()
            .flatten();
        if thumb.is_none() {
            tracing::debug!("the cover in the tag cannot be shown");
        }
        sheet.set_cover(Some(cover.with_thumb(thumb)));
    }
    sheet
}

/// Replaces every front-cover picture of `tag` by `cover` (none removes
/// them). Pictures of other types are not touched.
fn change_cover(tag: &mut Tag, cover: Option<&(CoverArt, MimeType)>) -> Result<(), TagWriteError> {
    if !can_store_pictures(tag.tag_type()) {
        return Err(TagWriteError::CoverNotStorable);
    }
    tag.remove_picture_type(PictureType::CoverFront);
    if let Some((cover, mime)) = cover {
        tag.push_picture(
            Picture::unchecked(cover.data().to_vec())
                .pic_type(PictureType::CoverFront)
                .mime_type(mime.clone())
                .build(),
        );
    }
    Ok(())
}
