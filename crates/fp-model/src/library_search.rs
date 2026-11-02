//! Search, counts and sorting of the library window (track library spec §7,
//! §9). Pure functions: the interface asks for the rows to show and draws
//! them. The index folds each track's text once, so a search is a substring
//! test per word.

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use unicode_normalization::UnicodeNormalization;

use crate::category::Categories;
use crate::columns::TableColumn;
use crate::ids::{CategoryId, TrackId};
use crate::library_view::{LibraryColumn, LibrarySelection, LibrarySort};
use crate::track::{Library, Track, TrackKind};

/// `text` in lower case with its accents dropped (NFD, combining marks
/// removed), so case and accents do not change a search or an order.
pub fn fold(text: &str) -> String {
    text.nfd()
        .filter(|c| !unicode_normalization::char::is_combining_mark(*c))
        .flat_map(char::to_lowercase)
        .collect()
}

/// The words of a query, folded.
pub fn query_words(query: &str) -> Vec<String> {
    fold(query).split_whitespace().map(str::to_owned).collect()
}

fn file_name(track: &Track) -> String {
    track
        .path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// The folded title, artist, album, album artist, composer, genre, comment
/// and file name of a track, one per line.
fn haystack(track: &Track) -> String {
    let file = file_name(track);
    let fields = [
        &track.title,
        &track.artist,
        &track.album,
        &track.album_artist,
        &track.composer,
        &track.genre,
        &track.comment,
        &file,
    ];
    let mut out = String::new();
    for (i, field) in fields.into_iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(&fold(field));
    }
    out
}

/// The folded text of every track, kept in step with the library.
#[derive(Debug, Clone, Default)]
pub struct SearchIndex {
    haystacks: HashMap<TrackId, String>,
}

impl SearchIndex {
    pub fn build(library: &Library) -> Self {
        Self {
            haystacks: library.iter().map(|t| (t.id, haystack(t))).collect(),
        }
    }

    /// Re-folds only what `library.changes_since(old)` reports.
    pub fn update(&mut self, old: &Library, library: &Library) {
        let changes = library.changes_since(old);
        for id in changes.removed {
            self.haystacks.remove(&id);
        }
        for track in changes.upserted {
            self.haystacks.insert(track.id, haystack(track));
        }
    }

    /// Whether every word (from [`query_words`]) is in the track's text.
    pub fn matches(&self, track: TrackId, words: &[String]) -> bool {
        match self.haystacks.get(&track) {
            Some(text) => words.iter().all(|w| text.contains(w.as_str())),
            None => words.is_empty(),
        }
    }
}

/// The tracks the table lists, in id order: those of the selection (or the
/// whole library when `whole_library`) whose text has every word of `query`.
pub fn library_rows(
    library: &Library,
    categories: &Categories,
    index: &SearchIndex,
    selection: LibrarySelection,
    whole_library: bool,
    query: &str,
) -> Vec<TrackId> {
    let words = query_words(query);
    let scope: Option<HashSet<CategoryId>> = match selection {
        LibrarySelection::Category(id) if !whole_library => {
            Some(categories.subtree(id).into_iter().collect())
        }
        _ => None,
    };
    library
        .iter()
        .filter(|t| {
            whole_library
                || match selection {
                    LibrarySelection::All => true,
                    LibrarySelection::Uncategorised => t.category.is_none(),
                    LibrarySelection::Category(_) => t
                        .category
                        .is_some_and(|c| scope.as_ref().is_some_and(|s| s.contains(&c))),
                }
        })
        .filter(|t| index.matches(t.id, &words))
        .map(|t| t.id)
        .collect()
}

/// What a column's order compares.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Key {
    Text(String),
    Number(u64),
}

fn millis(secs: f64) -> Key {
    Key::Number((secs.max(0.0) * 1000.0).round() as u64)
}

fn kind_rank(kind: TrackKind) -> u64 {
    match kind {
        TrackKind::Music => 0,
        TrackKind::Jingle => 1,
        TrackKind::Effect => 2,
        TrackKind::Ad => 3,
        TrackKind::Voice => 4,
    }
}

/// The value a column orders by; `None` (no value) goes last either way.
fn key(track: &Track, categories: &Categories, column: LibraryColumn) -> Option<Key> {
    let text = |s: &str| (!s.is_empty()).then(|| Key::Text(fold(s)));
    match column {
        LibraryColumn::Kind => Some(Key::Number(kind_rank(track.kind))),
        LibraryColumn::Category => track
            .category
            .and_then(|c| categories.get(c))
            .map(|c| Key::Text(fold(&c.name))),
        LibraryColumn::Tag(column) => match column {
            // The library has no positions: the id is the stable order.
            TableColumn::Number => Some(Key::Number(track.id.0)),
            TableColumn::Title => text(&track.title),
            TableColumn::Artist => text(&track.artist),
            TableColumn::Album => text(&track.album),
            TableColumn::Date => text(track.date.as_deref().unwrap_or_default()),
            TableColumn::Genre => text(&track.genre),
            TableColumn::Duration => {
                (track.duration_secs > 0.0).then(|| millis(track.play_range(true).length()))
            }
            TableColumn::Intro => track.intro_secs(true).map(millis),
            TableColumn::FileName => text(&file_name(track)),
            TableColumn::Comment => text(
                &track
                    .comment
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" "),
            ),
        },
    }
}

/// Orders `rows` by a column, text compared with [`fold`]. Ties go by track
/// id, and tracks without a value for the column come last in both
/// directions.
pub fn sort_rows(
    rows: &mut [TrackId],
    library: &Library,
    categories: &Categories,
    sort: LibrarySort,
) {
    let mut keyed: Vec<(Option<Key>, TrackId)> = rows
        .iter()
        .map(|&id| {
            let k = library
                .get(id)
                .and_then(|t| key(t, categories, sort.column));
            (k, id)
        })
        .collect();
    keyed.sort_by(|(a, ida), (b, idb)| {
        let order = match (a, b) {
            (Some(a), Some(b)) if sort.ascending => a.cmp(b),
            (Some(a), Some(b)) => b.cmp(a),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        };
        order.then(ida.cmp(idb))
    });
    for (slot, (_, id)) in rows.iter_mut().zip(keyed) {
        *slot = id;
    }
}

/// How many tracks each tree node lists.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CategoryCounts {
    pub all: usize,
    pub uncategorised: usize,
    /// Own tracks plus those of the subcategories.
    pub by_category: HashMap<CategoryId, usize>,
}

pub fn category_counts(library: &Library, categories: &Categories) -> CategoryCounts {
    let mut own: HashMap<CategoryId, usize> = HashMap::new();
    let mut uncategorised = 0;
    for track in library.iter() {
        match track.category {
            Some(c) => *own.entry(c).or_default() += 1,
            None => uncategorised += 1,
        }
    }
    let by_category = categories
        .iter()
        .map(|c| {
            let n = categories
                .subtree(c.id)
                .iter()
                .map(|s| own.get(s).copied().unwrap_or(0))
                .sum();
            (c.id, n)
        })
        .collect();
    CategoryCounts {
        all: library.len(),
        uncategorised,
        by_category,
    }
}
