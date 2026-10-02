//! Reading and writing the text tags of a track (feedback 2 spec O23).
//!
//! Reading degrades to "nothing" like `metadata::read_tags`. Writing never
//! touches the original until the new file is complete: the tags go into a
//! copy next to it, which is synced and then renamed over the original.

use std::path::{Path, PathBuf};

use fp_model::{Limits, TrackTags};
use lofty::config::WriteOptions;
use lofty::file::{FileType, TaggedFileExt};
use lofty::prelude::*;
use lofty::tag::items::Timestamp;
use lofty::tag::{ItemKey, Tag};

use crate::metadata::{read_tags, title_from_file_name};

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
    /// Anything else, with the system's or lofty's own description.
    Other(String),
}

impl std::fmt::Display for TagWriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported => f.write_str("this format has no writable tags"),
            Self::NotFound => f.write_str("the file was not found"),
            Self::Denied => f.write_str("permission denied"),
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
        year: tags.year,
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
    // A symlink is written through, not replaced.
    let real = path.canonicalize()?;
    if !can_write_tags(&real) {
        return Err(TagWriteError::Unsupported);
    }
    let after = after.clone().clamped(limits.max_tag_chars);
    let temp = temp_path(&real).ok_or_else(|| lofty_error("the file has no name"))?;
    let result = rewrite(&real, &temp, before, &after, limits);
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

fn rewrite(
    real: &Path,
    temp: &Path,
    before: &TrackTags,
    after: &TrackTags,
    limits: &Limits,
) -> Result<(), TagWriteError> {
    std::fs::copy(real, temp)?;
    // Parsed with its covers: if that fails the write fails, because saving
    // a tag read without them would drop them.
    let limit = usize::try_from(limits.max_cover_bytes)
        .unwrap_or(usize::MAX)
        .saturating_add(1024 * 1024)
        .max(16 * 1024 * 1024);
    lofty::config::apply_global_options(
        lofty::config::GlobalOptions::new().allocation_limit(limit),
    );
    let mut file = std::panic::catch_unwind(|| lofty::read_from_path(temp))
        .map_err(|_| lofty_error("the tag parser failed"))?
        .map_err(lofty_error)?;
    // The tag `read_tags` shows: the primary one, else the first.
    if file.primary_tag().is_none() && file.first_tag().is_none() {
        let ty = file.primary_tag_type();
        file.insert_tag(Tag::new(ty));
    }
    let tag = match file.primary_tag_mut() {
        Some(tag) => tag,
        None => file
            .first_tag_mut()
            .ok_or_else(|| lofty_error("the file has no tag to edit"))?,
    };
    change_tag(tag, before, after);
    file.save_to_path(temp, WriteOptions::default())
        .map_err(lofty_error)?;
    std::fs::OpenOptions::new()
        .write(true)
        .open(temp)?
        .sync_all()?;
    std::fs::rename(temp, real)?;
    Ok(())
}

fn change_tag(tag: &mut Tag, before: &TrackTags, after: &TrackTags) {
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
    if before.year != after.year {
        match after.year {
            Some(year) => match u16::try_from(year) {
                Ok(year) => tag.set_date(Timestamp {
                    year,
                    month: None,
                    day: None,
                    hour: None,
                    minute: None,
                    second: None,
                }),
                Err(_) => tag.remove_date(),
            },
            None => tag.remove_date(),
        }
    }
}
