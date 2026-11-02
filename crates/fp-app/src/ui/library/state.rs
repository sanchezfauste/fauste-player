//! What the library window remembers between frames and never saves
//! (track library spec §7): the search text, the selected rows, the rows
//! cache and a move, resize or split drag in progress.

use std::sync::Arc;

use egui::{Pos2, Rect};
use fp_model::library_search::SearchIndex;
use fp_model::{Library, TrackId};

/// The library window's runtime state, in `ViewState::library`.
#[derive(Default)]
pub(crate) struct LibraryUi {
    /// The search field's text (not saved, plan 3 rulings).
    #[expect(dead_code, reason = "the table (plan 3, task 5) reads it")]
    pub search: String,
    /// The selected rows, in the order they were selected.
    #[expect(dead_code, reason = "the table (plan 3, task 5) reads it")]
    pub selected: Vec<TrackId>,
    /// The row a Shift+click extends the selection from.
    #[expect(dead_code, reason = "the table (plan 3, task 5) reads it")]
    pub anchor: Option<TrackId>,
    /// The rows the table lists, kept until what they depend on changes.
    #[expect(dead_code, reason = "the table (plan 3, task 5) reads it")]
    pub rows: Option<RowsCache>,
    /// The dialog the window has open, if any.
    #[expect(dead_code, reason = "the tree and the table (plan 3) read it")]
    pub dialog: Option<LibraryDialog>,
    /// A move or a resize of the window being dragged, or just released
    /// and waiting for the snapshot to hold its rectangle.
    pub(crate) live: Option<LiveRect>,
    /// The tree's width while its splitter is dragged or just released.
    pub(crate) live_tree: Option<LiveTree>,
    /// Where the window was drawn in the last frame; `None` while closed.
    pub(crate) drawn: Option<Rect>,
}

/// The table's rows and what they were computed from.
#[expect(dead_code, reason = "the table (plan 3, task 5) builds it")]
pub(crate) struct RowsCache {
    /// The library the index and the rows were built from.
    pub library: Arc<Library>,
    pub index: SearchIndex,
    pub rows: Vec<TrackId>,
}

/// The dialogs of the library window.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum LibraryDialog {}

/// What a drag on the window's frame does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Grip {
    /// The title bar: the window moves.
    Move,
    /// An edge or a corner: those edges follow the pointer.
    Resize {
        left: bool,
        right: bool,
        top: bool,
        bottom: bool,
    },
}

/// A move or resize of the window.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct LiveRect {
    pub grip: Grip,
    /// The window's rectangle when the drag started.
    pub start: Rect,
    /// Where the pointer was pressed.
    pub origin: Pos2,
    /// The rectangle drawn now.
    pub rect: Rect,
    /// The drag ended and its view was sent.
    pub released: bool,
}

/// A drag of the splitter between the tree and the table.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct LiveTree {
    /// The tree's width when the drag started.
    pub start: f32,
    /// Where the pointer was pressed, across.
    pub origin: f32,
    pub width: f32,
    pub released: bool,
}
