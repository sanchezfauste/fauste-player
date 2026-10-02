//! The tag sheet (feedback 2 spec O23): every field the tag editor can show
//! for one file, as the file holds it. The sheet is a plain value: reading
//! and writing the file is `fp-analysis`' job, drawing it is the UI's. What
//! is decided here, as pure functions, is which fields exist, which are
//! shown, which values are invalid and which fields changed.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::track::{cut, parse_tag_date};

/// A field of the tag editor, in the order the editor shows them. Each one
/// maps to the format's own standard key (ID3v2 frame, Vorbis comment, MP4
/// atom, APE item, RIFF INFO chunk); `fp-analysis` knows how.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TagField {
    // Always shown.
    Title,
    Artist,
    Album,
    AlbumArtist,
    Date,
    TrackNumber,
    DiscNumber,
    Genre,
    Composer,
    Comment,
    // Shown when the file has them, offered by "Add field" otherwise.
    Subtitle,
    Grouping,
    Bpm,
    InitialKey,
    Mood,
    Isrc,
    Publisher,
    CatalogNumber,
    Copyright,
    OriginalArtist,
    OriginalAlbum,
    OriginalReleaseDate,
    Lyricist,
    Conductor,
    Remixer,
    Arranger,
    Performer,
    Language,
    EncodedBy,
    Lyrics,
    SortTitle,
    SortArtist,
    SortAlbum,
    SortAlbumArtist,
    SortComposer,
    ArtistWebsite,
}

/// How a field's value is typed and checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagFieldKind {
    /// Free text on one line.
    Text,
    /// Free text over several lines (comment, lyrics).
    LongText,
    /// ISO 8601 date (`parse_tag_date`).
    Date,
    /// A number and its total, both whole numbers (track, disc).
    Pair,
    /// A whole number (BPM).
    Whole,
}

impl TagField {
    /// Every field, in the order the editor shows them: the ten always-shown
    /// ones first, then the 26 optional ones.
    pub const ALL: [TagField; 36] = [
        TagField::Title,
        TagField::Artist,
        TagField::Album,
        TagField::AlbumArtist,
        TagField::Date,
        TagField::TrackNumber,
        TagField::DiscNumber,
        TagField::Genre,
        TagField::Composer,
        TagField::Comment,
        TagField::Subtitle,
        TagField::Grouping,
        TagField::Bpm,
        TagField::InitialKey,
        TagField::Mood,
        TagField::Isrc,
        TagField::Publisher,
        TagField::CatalogNumber,
        TagField::Copyright,
        TagField::OriginalArtist,
        TagField::OriginalAlbum,
        TagField::OriginalReleaseDate,
        TagField::Lyricist,
        TagField::Conductor,
        TagField::Remixer,
        TagField::Arranger,
        TagField::Performer,
        TagField::Language,
        TagField::EncodedBy,
        TagField::Lyrics,
        TagField::SortTitle,
        TagField::SortArtist,
        TagField::SortAlbum,
        TagField::SortAlbumArtist,
        TagField::SortComposer,
        TagField::ArtistWebsite,
    ];

    /// Shown for every file, whether it has the field or not.
    pub fn always_shown(self) -> bool {
        matches!(
            self,
            Self::Title
                | Self::Artist
                | Self::Album
                | Self::AlbumArtist
                | Self::Date
                | Self::TrackNumber
                | Self::DiscNumber
                | Self::Genre
                | Self::Composer
                | Self::Comment
        )
    }

    pub fn kind(self) -> TagFieldKind {
        match self {
            Self::Date | Self::OriginalReleaseDate => TagFieldKind::Date,
            Self::TrackNumber | Self::DiscNumber => TagFieldKind::Pair,
            Self::Bpm => TagFieldKind::Whole,
            Self::Comment | Self::Lyrics => TagFieldKind::LongText,
            _ => TagFieldKind::Text,
        }
    }

    /// The field can hold several values (two artists), shown one per line
    /// and written one per value through the format's own mechanism.
    pub fn is_multi_value(self) -> bool {
        matches!(
            self,
            Self::Artist
                | Self::AlbumArtist
                | Self::Genre
                | Self::Composer
                | Self::Mood
                | Self::OriginalArtist
                | Self::Lyricist
                | Self::Conductor
                | Self::Remixer
                | Self::Arranger
                | Self::Performer
                | Self::Language
        )
    }

    /// The suffix of the field's label key in the locale files
    /// (`tag-field-<slug>`).
    pub fn slug(self) -> &'static str {
        match self {
            Self::Title => "title",
            Self::Artist => "artist",
            Self::Album => "album",
            Self::AlbumArtist => "album-artist",
            Self::Date => "date",
            Self::TrackNumber => "track-number",
            Self::DiscNumber => "disc-number",
            Self::Genre => "genre",
            Self::Composer => "composer",
            Self::Comment => "comment",
            Self::Subtitle => "subtitle",
            Self::Grouping => "grouping",
            Self::Bpm => "bpm",
            Self::InitialKey => "initial-key",
            Self::Mood => "mood",
            Self::Isrc => "isrc",
            Self::Publisher => "publisher",
            Self::CatalogNumber => "catalog-number",
            Self::Copyright => "copyright",
            Self::OriginalArtist => "original-artist",
            Self::OriginalAlbum => "original-album",
            Self::OriginalReleaseDate => "original-release-date",
            Self::Lyricist => "lyricist",
            Self::Conductor => "conductor",
            Self::Remixer => "remixer",
            Self::Arranger => "arranger",
            Self::Performer => "performer",
            Self::Language => "language",
            Self::EncodedBy => "encoded-by",
            Self::Lyrics => "lyrics",
            Self::SortTitle => "sort-title",
            Self::SortArtist => "sort-artist",
            Self::SortAlbum => "sort-album",
            Self::SortAlbumArtist => "sort-album-artist",
            Self::SortComposer => "sort-composer",
            Self::ArtistWebsite => "artist-website",
        }
    }
}

/// Tidies the lines of one field: trimmed, empty lines dropped. A number and
/// total field always holds exactly two entries (number, total), numbers in
/// their plain form (`03` is `3`), or nothing when both are empty.
fn canonical(field: TagField, lines: Vec<String>) -> Vec<String> {
    if field.kind() == TagFieldKind::Pair {
        let mut parts = lines.into_iter().map(|l| plain_number(&l));
        let (number, total) = (
            parts.next().unwrap_or_default(),
            parts.next().unwrap_or_default(),
        );
        return if number.is_empty() && total.is_empty() {
            Vec::new()
        } else {
            vec![number, total]
        };
    }
    lines
        .into_iter()
        .map(|l| tidy(field, &l))
        .filter(|l| !l.is_empty())
        .collect()
}

/// `text` trimmed; a whole number loses its leading zeros.
fn plain_number(text: &str) -> String {
    let text = text.trim();
    match text.parse::<u32>() {
        Ok(n) if text.bytes().all(|b| b.is_ascii_digit()) => n.to_string(),
        _ => text.to_owned(),
    }
}

fn tidy(field: TagField, text: &str) -> String {
    if field.kind() == TagFieldKind::Whole {
        plain_number(text)
    } else {
        text.trim().to_owned()
    }
}

fn is_whole(text: &str) -> bool {
    text.is_empty() || (text.bytes().all(|b| b.is_ascii_digit()) && text.parse::<u32>().is_ok())
}

/// A picture of the sheet: the file's front cover or, when it has none, its
/// first picture (the same rule the library's cover thumbnail follows). The
/// bytes are shared (`Arc`), so a draft of the sheet is cheap to clone and
/// two covers are equal when their bytes are.
#[derive(Debug, Clone)]
pub struct CoverArt {
    data: Arc<[u8]>,
    thumb: Option<Arc<[u8]>>,
    front: bool,
}

impl CoverArt {
    /// A picture with no thumbnail yet; `front` says whether its type is
    /// front cover.
    pub fn new(data: Vec<u8>, front: bool) -> Self {
        Self {
            data: data.into(),
            thumb: None,
            front,
        }
    }

    /// The same picture with `thumb` (a small PNG, `None` when the image
    /// could not be decoded) as its thumbnail.
    #[must_use]
    pub fn with_thumb(self, thumb: Option<Vec<u8>>) -> Self {
        Self {
            thumb: thumb.map(Into::into),
            ..self
        }
    }

    /// The image file as stored (JPEG, PNG, ...).
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// The thumbnail the interface draws; `None` if the image cannot be
    /// decoded (it is still kept as it is).
    pub fn thumb_png(&self) -> Option<&[u8]> {
        self.thumb.as_deref()
    }

    /// The thumbnail as the media cache holds it.
    pub fn thumb_shared(&self) -> Option<Arc<[u8]>> {
        self.thumb.clone()
    }

    /// The picture is a front cover; otherwise it is only the first picture
    /// of the file, shown for lack of one, and the editor never changes it.
    pub fn is_front(&self) -> bool {
        self.front
    }
}

impl PartialEq for CoverArt {
    fn eq(&self, other: &Self) -> bool {
        self.front == other.front
            && (Arc::ptr_eq(&self.data, &other.data) || self.data == other.data)
    }
}

impl Eq for CoverArt {}

/// What a file's tag holds, field by field, and what its format can store.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TagSheet {
    values: BTreeMap<TagField, Vec<String>>,
    storable: BTreeSet<TagField>,
    cover: Option<CoverArt>,
    cover_storable: bool,
    /// Tags the sheet does not show (other standard keys, custom keys,
    /// pictures). They are kept as they are.
    pub other_kept: usize,
    /// The format also holds items the reader cannot count, so `other_kept`
    /// is a lower bound.
    pub other_kept_more: bool,
}

impl TagSheet {
    /// An empty sheet for a format that can store `storable`.
    pub fn new(
        storable: impl IntoIterator<Item = TagField>,
        other_kept: usize,
        other_kept_more: bool,
    ) -> Self {
        Self {
            values: BTreeMap::new(),
            storable: storable.into_iter().collect(),
            cover: None,
            cover_storable: false,
            other_kept,
            other_kept_more,
        }
    }

    /// The same sheet for a format that can (or cannot) store pictures. A
    /// new sheet cannot.
    #[must_use]
    pub fn with_cover_support(mut self, storable: bool) -> Self {
        self.cover_storable = storable;
        self
    }

    /// The file's format can store a picture (ID3v2, Vorbis comments, MP4
    /// and APE can; RIFF INFO, AIFF text and ID3v1 cannot).
    pub fn can_store_cover(&self) -> bool {
        self.cover_storable
    }

    /// The picture the editor shows: the front cover, else the first
    /// picture of the file.
    pub fn cover(&self) -> Option<&CoverArt> {
        self.cover.as_ref()
    }

    /// Replaces the picture (the front cover when `cover.is_front()`).
    pub fn set_cover(&mut self, cover: Option<CoverArt>) {
        self.cover = cover;
    }

    /// Clears the front cover. A picture of another type that the sheet only
    /// shows for lack of a front cover (`original`'s) stays as it is.
    pub fn remove_front_cover(&mut self, original: &TagSheet) {
        self.cover = original.cover.clone().filter(|cover| !cover.is_front());
    }

    /// The values of `field`: empty when the file has none; two entries
    /// (number, total) for a number and total field.
    pub fn values(&self, field: TagField) -> &[String] {
        self.values.get(&field).map_or(&[], Vec::as_slice)
    }

    /// Replaces the values of `field` (tidied; none left removes it).
    pub fn set_values(&mut self, field: TagField, values: Vec<String>) {
        let values = canonical(field, values);
        if values.is_empty() {
            self.values.remove(&field);
        } else {
            self.values.insert(field, values);
        }
    }

    /// The file's format can store `field`.
    pub fn can_store(&self, field: TagField) -> bool {
        self.storable.contains(&field)
    }

    /// The file has a value for `field`.
    pub fn has(&self, field: TagField) -> bool {
        self.values.contains_key(&field)
    }

    /// The values as one text, one per line: what a text box holds.
    pub fn text(&self, field: TagField) -> String {
        self.values(field).join("\n")
    }

    /// Sets the values from a text box. With `as_lines` every non-empty line
    /// is a value; without, the whole text is one value.
    pub fn set_text(&mut self, field: TagField, text: &str, as_lines: bool) {
        let values = if as_lines {
            text.lines().map(str::to_owned).collect()
        } else {
            vec![text.to_owned()]
        };
        self.set_values(field, values);
    }

    /// The number and the total of a track or disc field (empty if unset).
    pub fn pair(&self, field: TagField) -> (String, String) {
        let values = self.values(field);
        (
            values.first().cloned().unwrap_or_default(),
            values.get(1).cloned().unwrap_or_default(),
        )
    }

    pub fn set_pair(&mut self, field: TagField, number: &str, total: &str) {
        self.set_values(field, vec![number.to_owned(), total.to_owned()]);
    }

    /// Whether the editor shows `field` one value per line: it can hold
    /// several, or the file holds several now.
    pub fn shows_lines(&self, field: TagField) -> bool {
        field.is_multi_value() || self.values(field).len() > 1
    }

    /// The fields to draw, in order: the always-shown ones (the editor greys
    /// out those the format cannot store), then each optional field the file
    /// has or the operator `added` (only if the format can store it).
    pub fn visible_fields(&self, added: &BTreeSet<TagField>) -> Vec<TagField> {
        TagField::ALL
            .into_iter()
            .filter(|f| {
                f.always_shown() || (self.can_store(*f) && (self.has(*f) || added.contains(f)))
            })
            .collect()
    }

    /// What **Add field** offers: optional fields the format can store that
    /// the file does not have and the operator has not added yet.
    pub fn addable_fields(&self, added: &BTreeSet<TagField>) -> Vec<TagField> {
        TagField::ALL
            .into_iter()
            .filter(|f| {
                !f.always_shown() && self.can_store(*f) && !self.has(*f) && !added.contains(f)
            })
            .collect()
    }

    /// Every line cut to `max_chars` characters and at most `max_values`
    /// values per field (a number and total field keeps its two). Tag text
    /// comes from files and keyboards and must not bloat memory or the
    /// window.
    #[must_use]
    pub fn clamped(self, max_chars: usize, max_values: usize) -> Self {
        let mut out = Self {
            values: BTreeMap::new(),
            ..self.clone()
        };
        for (field, lines) in self.values {
            let keep = if field.kind() == TagFieldKind::Pair {
                lines.len()
            } else {
                max_values
            };
            let lines = lines
                .into_iter()
                .take(keep)
                .map(|mut line| {
                    cut(&mut line, max_chars);
                    line
                })
                .collect();
            out.set_values(field, lines);
        }
        out
    }
}

/// The operator changed the front cover (replaced or removed it).
pub fn cover_changed(before: &TagSheet, after: &TagSheet) -> bool {
    before.cover != after.cover
}

/// The cover was changed but the file's format cannot store pictures: the
/// change cannot be written.
pub fn cover_blocked(before: &TagSheet, after: &TagSheet) -> bool {
    cover_changed(before, after) && !after.can_store_cover()
}

/// The cover was changed and the file does not hold it as written after the
/// save (`read_back`).
pub fn cover_unstored(before: &TagSheet, after: &TagSheet, read_back: &TagSheet) -> bool {
    cover_changed(before, after) && after.cover != read_back.cover
}

/// The fields whose values differ between two sheets, in editor order.
pub fn changed_fields(before: &TagSheet, after: &TagSheet) -> Vec<TagField> {
    TagField::ALL
        .into_iter()
        .filter(|f| before.values(*f) != after.values(*f))
        .collect()
}

/// The changed fields (`after` against `before`) whose value is not valid:
/// a date that is not ISO 8601, a number, total or BPM that is not a whole
/// number, a total without a number, several lines in a one-value field of
/// those kinds, or a field the file's format cannot store. A value the file
/// already held and the operator did not touch is never reported: it is kept
/// as it is.
pub fn invalid_fields(before: &TagSheet, after: &TagSheet) -> Vec<TagField> {
    changed_fields(before, after)
        .into_iter()
        .filter(|f| {
            let values = after.values(*f);
            let valid = match f.kind() {
                TagFieldKind::Text | TagFieldKind::LongText => true,
                TagFieldKind::Date => {
                    values.len() <= 1 && values.iter().all(|v| parse_tag_date(v).is_ok())
                }
                TagFieldKind::Whole => values.len() <= 1 && values.iter().all(|v| is_whole(v)),
                TagFieldKind::Pair => {
                    let (number, total) = after.pair(*f);
                    is_whole(&number)
                        && is_whole(&total)
                        && (total.is_empty() || !number.is_empty())
                }
            };
            !valid || !after.can_store(*f)
        })
        .collect()
}

/// The fields the operator changed that the file does not hold as written
/// after the save (`read_back`): the format dropped or reshaped them.
pub fn unstored_fields(before: &TagSheet, after: &TagSheet, read_back: &TagSheet) -> Vec<TagField> {
    changed_fields(before, after)
        .into_iter()
        .filter(|f| after.values(*f) != read_back.values(*f))
        .collect()
}
