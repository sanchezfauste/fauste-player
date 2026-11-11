//! The library window's saved state (track library spec §7, §9, §10): where
//! the window is, what it shows and how its table is arranged. Session
//! state, saved with the players; not part of `Config`. Pure rules: the
//! interface only displays it and sends `Command::SetLibraryView`.

use serde::{Deserialize, Serialize};

use crate::category::Categories;
use crate::columns::TableColumn;
use crate::ids::CategoryId;
use crate::layout::{lenient, lenient_items};

/// What the tree selects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LibrarySelection {
    /// Every track.
    #[default]
    All,
    /// The tracks without a category.
    Uncategorised,
    /// The tracks of one category (and its subcategories).
    Category(CategoryId),
    /// The tracks whose markers need review (external editor spec EE11).
    MarkersToReview,
}

/// A column of the library table: a tag column or one the library adds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LibraryColumn {
    Tag(TableColumn),
    Category,
    Kind,
}

/// The width of one column, in points.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LibraryColumnWidth {
    pub column: LibraryColumn,
    pub width: f32,
}

/// The table's sort order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LibrarySort {
    pub column: LibraryColumn,
    pub ascending: bool,
}

/// The library window's state. Every field loads leniently: a bad field
/// takes its default and the rest is kept.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LibraryView {
    #[serde(deserialize_with = "lenient")]
    pub open: bool,
    /// x, y, width, height in points; `None`: the default placement.
    #[serde(deserialize_with = "lenient")]
    pub rect: Option<[f32; 4]>,
    #[serde(deserialize_with = "lenient")]
    pub minimised: bool,
    #[serde(deserialize_with = "lenient")]
    pub maximised: bool,
    #[serde(deserialize_with = "lenient")]
    pub selection: LibrarySelection,
    /// The categories shown expanded in the tree.
    #[serde(deserialize_with = "lenient_items")]
    pub expanded: Vec<CategoryId>,
    /// Whether the table lists the whole library instead of the selection.
    #[serde(deserialize_with = "lenient")]
    pub whole_library: bool,
    #[serde(deserialize_with = "lenient_items")]
    pub columns: Vec<LibraryColumnWidth>,
    #[serde(deserialize_with = "lenient")]
    pub sort: Option<LibrarySort>,
    #[serde(deserialize_with = "lenient")]
    pub tree_width: Option<f32>,
    /// The library CUE's listening level, a linear gain 0.0–1.0 (track
    /// library spec LB59, LB60). Only `Command::SetLibraryCueVolume`
    /// changes it.
    #[serde(deserialize_with = "lenient_volume")]
    pub cue_volume: f32,
}

impl Default for LibraryView {
    fn default() -> Self {
        Self {
            open: false,
            rect: None,
            minimised: false,
            maximised: false,
            selection: LibrarySelection::default(),
            expanded: Vec::new(),
            whole_library: false,
            columns: Vec::new(),
            sort: None,
            tree_width: None,
            cue_volume: 1.0,
        }
    }
}

/// A level that is not a number loads as full (LB60).
fn lenient_volume<'de, D: serde::Deserializer<'de>>(d: D) -> Result<f32, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Lenient {
        Level(f32),
        Other(serde::de::IgnoredAny),
    }
    Ok(match Lenient::deserialize(d)? {
        Lenient::Level(v) => v,
        Lenient::Other(_) => 1.0,
    })
}

/// A listening level made usable: not finite is full, anything else is
/// clamped into 0.0–1.0.
pub fn usable_volume(v: f32) -> f32 {
    if v.is_finite() {
        v.clamp(0.0, 1.0)
    } else {
        1.0
    }
}

/// Whether `v` is a usable size in points: finite and above zero.
pub fn good_size(v: f32) -> bool {
    v.is_finite() && v > 0.0
}

/// Whether a saved rectangle is usable: a finite place and usable sizes.
pub fn good_rect([x, y, w, h]: [f32; 4]) -> bool {
    x.is_finite() && y.is_finite() && good_size(w) && good_size(h)
}

impl LibraryView {
    /// The view with what `categories` no longer holds or what is not a
    /// usable size removed: an unknown selection becomes `All`.
    pub fn repaired(mut self, categories: &Categories) -> LibraryView {
        if let LibrarySelection::Category(id) = self.selection
            && categories.get(id).is_none()
        {
            self.selection = LibrarySelection::All;
        }
        self.expanded.retain(|id| categories.get(*id).is_some());
        if let Some(rect) = self.rect
            && !good_rect(rect)
        {
            self.rect = None;
        }
        self.columns.retain(|c| good_size(c.width));
        self.tree_width = self.tree_width.filter(|w| good_size(*w));
        self.cue_volume = usable_volume(self.cue_volume);
        self
    }
}

/// The saved view read leniently: a value that is not a view is `None`
/// (the caller uses the default); inside a view each field falls back on
/// its own.
pub fn lenient_library_view(value: serde_json::Value) -> Option<LibraryView> {
    LibraryView::deserialize(value).ok()
}
