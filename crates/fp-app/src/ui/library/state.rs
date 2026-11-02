//! What the library window remembers between frames and never saves
//! (track library spec §7): the search text, the selected rows, the rows
//! cache and a move, resize or split drag in progress.

use egui::{Pos2, Rect};
use fp_model::library_search::{CategoryCounts, SearchIndex, library_rows, sort_rows};
use fp_model::library_view::{
    LibraryColumn, LibraryColumnWidth, LibrarySelection, LibrarySort, LibraryView,
};
use fp_model::{AppState, CartColor, Categories, CategoryId, Library, TrackId, TrackKind};

use super::confirm::{Asked, Removal};

/// The library window's runtime state, in `ViewState::library`.
#[derive(Default)]
pub(crate) struct LibraryUi {
    /// The search field's text (not saved, plan 3 rulings).
    pub search: String,
    /// The selected rows, in the order they were selected.
    pub selected: Vec<TrackId>,
    /// The row a Shift+click extends the selection from.
    pub anchor: Option<TrackId>,
    /// The rows the table lists, kept until what they depend on changes.
    pub rows: Option<RowsCache>,
    /// A column edge of the table being dragged.
    pub(crate) live_columns: Option<LiveColumns>,
    /// The dialog the window has open, if any.
    pub dialog: Option<LibraryDialog>,
    /// The tree's counts, kept until the library or the categories change.
    pub(crate) counts: Option<CountsCache>,
    /// The category being dragged in the tree.
    pub(crate) tree_drag: Option<CategoryId>,
    /// A move or a resize of the window being dragged.
    pub(crate) live: Option<LiveRect>,
    /// The tree's splitter being dragged.
    pub(crate) live_tree: Option<LiveTree>,
    /// The rectangle a finished move or resize sent, until the snapshot
    /// holds it.
    pub(crate) pending_rect: Option<Pending<Rect>>,
    /// The tree width a finished splitter drag sent, until the snapshot
    /// holds it.
    pub(crate) pending_tree: Option<Pending<f32>>,
    /// Where the window was drawn in the last frame; `None` while closed.
    pub(crate) drawn: Option<Rect>,
}

/// The table's rows and what they were computed from.
pub(crate) struct RowsCache {
    /// The library the index and the rows were built from (a clone shares
    /// its storage, so keeping it costs nothing).
    pub library: Library,
    pub categories: Categories,
    pub selection: LibrarySelection,
    pub whole_library: bool,
    pub query: String,
    pub sort: Option<LibrarySort>,
    pub index: SearchIndex,
    pub rows: Vec<TrackId>,
}

impl RowsCache {
    /// Whether the rows still hold for `state` and `query`.
    fn holds(&self, state: &AppState, query: &str) -> bool {
        let view = &state.library_view;
        self.library.shares_storage(&state.library)
            && self.categories == state.categories
            && self.selection == view.selection
            && self.whole_library == view.whole_library
            && self.sort == view.sort
            && self.query == query
    }
}

impl LibraryUi {
    /// The tracks the table lists (track library spec §7.4, §7.5): those of
    /// the selected category or of the whole library, matching the search,
    /// in the saved order. Computed again only when the library, the
    /// categories, the selection, the scope, the search or the order
    /// changed; the search index follows the library's changes. Selected
    /// rows whose track left the library are dropped.
    pub(crate) fn rows(&mut self, state: &AppState) -> &[TrackId] {
        let fresh = self
            .rows
            .as_ref()
            .is_some_and(|c| c.holds(state, &self.search));
        if !fresh {
            let library_changed = self
                .rows
                .as_ref()
                .is_none_or(|c| !c.library.shares_storage(&state.library));
            let index = match self.rows.take() {
                Some(RowsCache {
                    library, mut index, ..
                }) => {
                    index.update(&library, &state.library);
                    index
                }
                None => SearchIndex::build(&state.library),
            };
            let view = &state.library_view;
            let mut rows = library_rows(
                &state.library,
                &state.categories,
                &index,
                view.selection,
                view.whole_library,
                &self.search,
            );
            if let Some(sort) = view.sort {
                sort_rows(&mut rows, &state.library, &state.categories, sort);
            }
            if library_changed {
                self.selected.retain(|id| state.library.get(*id).is_some());
                if self.anchor.is_some_and(|a| state.library.get(a).is_none()) {
                    self.anchor = None;
                }
            }
            self.rows = Some(RowsCache {
                library: state.library.clone(),
                categories: state.categories.clone(),
                selection: view.selection,
                whole_library: view.whole_library,
                query: self.search.clone(),
                sort: view.sort,
                index,
                rows,
            });
        }
        self.rows.as_ref().map_or(&[], |c| c.rows.as_slice())
    }

    /// A click on the row of `track` among `rows` (track library spec §7.4):
    /// alone it selects that row; with Ctrl (Cmd on macOS) it adds or
    /// removes it; with Shift it selects from the anchor to it (added to
    /// the selection with Ctrl too).
    pub(crate) fn click(&mut self, rows: &[TrackId], track: TrackId, modifiers: egui::Modifiers) {
        let range = |anchor: TrackId| {
            let from = rows.iter().position(|r| *r == anchor)?;
            let to = rows.iter().position(|r| *r == track)?;
            let (a, b) = if from <= to { (from, to) } else { (to, from) };
            rows.get(a..=b)
        };
        if modifiers.shift
            && let Some(span) = self.anchor.and_then(range)
        {
            if !modifiers.command {
                self.selected.clear();
            }
            for id in span {
                if !self.selected.contains(id) {
                    self.selected.push(*id);
                }
            }
            return;
        }
        if modifiers.command {
            match self.selected.iter().position(|s| *s == track) {
                Some(at) => {
                    self.selected.remove(at);
                }
                None => self.selected.push(track),
            }
        } else {
            self.selected = vec![track];
        }
        self.anchor = Some(track);
    }
}

/// A drag of an edge of the table's columns.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LiveColumns {
    /// The column whose right edge is dragged.
    pub edge: usize,
    pub columns: Vec<LibraryColumn>,
    /// The widths when the drag began, and where the pointer was.
    pub start_px: Vec<f32>,
    pub start_x: f32,
    /// The widths now.
    pub px: Vec<f32>,
    /// Set when the drag ends and the widths are sent: the saved widths at
    /// that moment. The drag's widths are drawn until the snapshot's differ.
    pub released: Option<Vec<LibraryColumnWidth>>,
}

/// The tree's counts and what they were computed from
/// (`category_counts` walks every category's subtree: not every frame).
pub(crate) struct CountsCache {
    pub library: Library,
    pub categories: Categories,
    pub counts: CategoryCounts,
}

/// The dialogs of the library window.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum LibraryDialog {
    /// A new category under `parent` (`None`: the top level).
    NewCategory {
        parent: Option<CategoryId>,
        name: String,
        color: Option<CartColor>,
        kind: TrackKind,
        /// The dialog has just opened: its name field takes the focus.
        fresh: bool,
    },
    /// A new name for `category`.
    RenameCategory {
        category: CategoryId,
        name: String,
        /// The dialog has just opened: the name is selected.
        fresh: bool,
    },
    /// A removal waiting for the operator's answer (LB11, LB17).
    Removal {
        removal: Removal,
        step: RemovalStep,
        /// What the question shows, and the library it was counted in.
        asked: Box<Asked>,
    },
}

/// The two questions of a removal (LB11).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RemovalStep {
    /// The subcategories and the tracks that go.
    First,
    /// The playlist entries and the carts that lose their tracks.
    Second,
}

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
}

/// A drag of the splitter between the tree and the table.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct LiveTree {
    /// The tree's width when the drag started.
    pub start: f32,
    /// Where the pointer was pressed, across.
    pub origin: f32,
    pub width: f32,
}

/// A value a finished drag sent, drawn and built on until the snapshot
/// holds it.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Pending<T> {
    pub value: T,
    /// The snapshot's view when the value was sent: once the snapshot
    /// differs from it, the snapshot has caught up.
    pub sent_against: Box<LibraryView>,
}
