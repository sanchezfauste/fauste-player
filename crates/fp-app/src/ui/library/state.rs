//! What the library window remembers between frames and never saves
//! (track library spec §7): the search text, the selected rows, the rows
//! cache and a move, resize or split drag in progress.

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use egui::{Pos2, Rect};
use fp_model::library_search::{
    CategoryCounts, SearchIndex, SortKeys, filter_rows, library_rows, sort_rows_by, update_ordered,
};
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
    pub selected: Selection,
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
    /// The category being dragged in a tree, and which tree it started in:
    /// the window's tree and the one in Settings → Library are both laid
    /// out while Settings is open, and only the one the drag started in
    /// takes its release.
    pub(crate) tree_drag: Option<(super::tree::TreeMode, CategoryId)>,
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
    /// The window's layer in the last frame: only a pointer on it is over
    /// the window (another window or a dialog may cover part of it).
    pub(crate) drawn_layer: Option<egui::LayerId>,
    /// A track *Show in library* asked for: the table selects its row and
    /// scrolls to it once the snapshot holds the selection it sent and
    /// lists the track.
    pub(crate) reveal: Option<Reveal>,
    /// The selected track last reported to the preview with
    /// `SelectLibraryCue` (LB34); `None` until the open window reports one.
    pub(crate) preview_sent: Option<Option<TrackId>>,
    /// How many times the rows were computed: a [`RowsCache`]'s
    /// `revision`.
    rows_built: u64,
    /// The selection in the table's order, kept until the rows or the
    /// selection change (an open row menu asks for it every frame).
    ordered: RefCell<Option<Ordered>>,
}

/// The selection in the table's order, and what it was computed from.
struct Ordered {
    rows: u64,
    selection: u64,
    tracks: Rc<[TrackId]>,
}

/// The selected rows: in the order they were selected, with a set so a
/// row asks whether it is selected in constant time.
#[derive(Debug, Default)]
pub(crate) struct Selection {
    order: Vec<TrackId>,
    set: HashSet<TrackId>,
    /// Bumped by every change.
    revision: u64,
}

impl Selection {
    pub(crate) fn contains(&self, track: &TrackId) -> bool {
        self.set.contains(track)
    }

    /// In the order they were selected.
    #[cfg(test)]
    pub(crate) fn ids(&self) -> &[TrackId] {
        &self.order
    }

    pub(crate) fn last(&self) -> Option<&TrackId> {
        self.order.last()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.order.len()
    }

    pub(crate) fn revision(&self) -> u64 {
        self.revision
    }

    pub(crate) fn clear(&mut self) {
        if !self.order.is_empty() {
            self.order.clear();
            self.set.clear();
            self.revision += 1;
        }
    }

    /// `track` alone.
    pub(crate) fn set_one(&mut self, track: TrackId) {
        if self.order.as_slice() == [track] {
            return;
        }
        self.clear();
        self.push(track);
    }

    /// Adds `track` at the end unless it is selected already.
    pub(crate) fn push(&mut self, track: TrackId) {
        if self.set.insert(track) {
            self.order.push(track);
            self.revision += 1;
        }
    }

    /// Removes `track`; the others keep their order.
    pub(crate) fn remove(&mut self, track: TrackId) {
        if self.set.remove(&track) {
            self.order.retain(|t| *t != track);
            self.revision += 1;
        }
    }

    /// Keeps the tracks `keep` accepts, in their order.
    pub(crate) fn retain(&mut self, mut keep: impl FnMut(&TrackId) -> bool) {
        let before = self.order.len();
        self.order.retain(|t| keep(t));
        if self.order.len() != before {
            self.set = self.order.iter().copied().collect();
            self.revision += 1;
        }
    }
}

#[cfg(test)]
impl PartialEq<Vec<TrackId>> for Selection {
    fn eq(&self, other: &Vec<TrackId>) -> bool {
        self.order == *other
    }
}

/// A *Show in library* waiting for the snapshot (track library spec §9).
pub(crate) struct Reveal {
    pub track: TrackId,
    /// The selection the window was sent to.
    pub selection: LibrarySelection,
    /// The snapshot's view when it was sent: a snapshot that differs from
    /// it without holding the selection is the operator's own change.
    pub against: Box<LibraryView>,
}

impl Reveal {
    pub(crate) fn new(track: TrackId, selection: LibrarySelection, against: &LibraryView) -> Self {
        Reveal {
            track,
            selection,
            against: Box::new(against.clone()),
        }
    }
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
    /// The folded text of every track, built when a search first needs it
    /// (a large library's first fold takes tens of milliseconds).
    pub index: Option<SearchIndex>,
    /// The folded keys of the sort column, for every track of `library`.
    pub sort_keys: Option<SortKeys>,
    /// Every track of the scope (the selection or the whole library) in the
    /// saved order, before the search: a key typed filters it instead of
    /// sorting again.
    pub ordered: Vec<TrackId>,
    pub rows: Vec<TrackId>,
    /// Which computation of the rows this is (`LibraryUi::rows_built`).
    pub revision: u64,
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

    /// Whether `ordered` still holds for `state`: all but the search.
    fn scope_holds(&self, state: &AppState) -> bool {
        let view = &state.library_view;
        self.library.shares_storage(&state.library)
            && self.categories == state.categories
            && self.selection == view.selection
            && self.whole_library == view.whole_library
            && self.sort == view.sort
    }
}

impl LibraryUi {
    /// The tracks the table lists (track library spec §7.4, §7.5): those of
    /// the selected category or of the whole library, matching the search,
    /// in the saved order. Computed again only when the library, the
    /// categories, the selection, the scope, the search or the order
    /// changed; the search index follows the library's changes. Selected
    /// rows no longer listed are dropped from the selection.
    pub(crate) fn rows(&mut self, state: &AppState) -> &[TrackId] {
        let fresh = self
            .rows
            .as_ref()
            .is_some_and(|c| c.holds(state, &self.search));
        if !fresh {
            let view = &state.library_view;
            let column = view.sort.map(|sort| sort.column);
            // Only the search changed: the scope's rows in order still hold.
            let kept = self
                .rows
                .as_ref()
                .filter(|c| c.scope_holds(state))
                .is_some();
            // The index and the sort keys follow the library's changes:
            // only the tracks a change touched are folded again.
            let (mut index, sort_keys, ordered) = match self.rows.take() {
                Some(RowsCache {
                    index,
                    sort_keys,
                    ordered,
                    ..
                }) if kept => (index, sort_keys, Some(ordered)),
                Some(RowsCache {
                    library,
                    categories,
                    selection,
                    whole_library,
                    sort,
                    mut index,
                    sort_keys,
                    ordered,
                    ..
                }) => {
                    if let Some(index) = index.as_mut() {
                        index.update(&library, &state.library);
                    }
                    let sort_keys = match (sort_keys, column) {
                        (Some(mut keys), Some(column)) if keys.column() == column => {
                            keys.update(&library, &state.library, &state.categories);
                            Some(keys)
                        }
                        (_, column) => {
                            column.map(|c| SortKeys::build(&state.library, &state.categories, c))
                        }
                    };
                    // Only the library changed (a scan analyses tracks while
                    // the window is open): the rows in order are patched.
                    let patched = (categories == state.categories
                        && selection == view.selection
                        && whole_library == view.whole_library
                        && sort == view.sort)
                        .then(|| {
                            update_ordered(
                                &ordered,
                                &library,
                                &state.library,
                                &state.categories,
                                view.selection,
                                view.whole_library,
                                view.sort
                                    .zip(sort_keys.as_ref())
                                    .map(|(sort, keys)| (keys, sort.ascending)),
                            )
                        })
                        .flatten();
                    (index, sort_keys, patched)
                }
                None => (
                    None,
                    column.map(|c| SortKeys::build(&state.library, &state.categories, c)),
                    None,
                ),
            };
            // No words, no index needed: every track matches.
            if index.is_none() && !fp_model::library_search::query_words(&self.search).is_empty() {
                index = Some(SearchIndex::build(&state.library));
            }
            let empty = SearchIndex::default();
            let ordered = ordered.unwrap_or_else(|| {
                // Without a search, no track needs the index.
                let mut ordered = library_rows(
                    &state.library,
                    &state.categories,
                    &empty,
                    view.selection,
                    view.whole_library,
                    "",
                );
                if let (Some(sort), Some(keys)) = (view.sort, sort_keys.as_ref()) {
                    sort_rows_by(&mut ordered, keys, sort.ascending);
                }
                ordered
            });
            let rows = filter_rows(&ordered, index.as_ref().unwrap_or(&empty), &self.search);
            // The selection keeps only the rows listed: what a category,
            // scope or search change hides, or the library lost, leaves it
            // (a menu or a drag acts on what the operator sees).
            if !self.selected.is_empty() {
                let listed: HashSet<TrackId> = rows.iter().copied().collect();
                self.selected.retain(|id| listed.contains(id));
                if self.anchor.is_some_and(|a| !listed.contains(&a)) {
                    self.anchor = None;
                }
            } else if self.anchor.is_some_and(|a| !rows.contains(&a)) {
                self.anchor = None;
            }
            self.rows_built += 1;
            self.rows = Some(RowsCache {
                library: state.library.clone(),
                categories: state.categories.clone(),
                selection: view.selection,
                whole_library: view.whole_library,
                query: self.search.clone(),
                sort: view.sort,
                index,
                sort_keys,
                ordered,
                rows,
                revision: self.rows_built,
            });
        }
        self.rows.as_ref().map_or(&[], |c| c.rows.as_slice())
    }

    /// Acts on a pending *Show in library* (`rows` must be current): once
    /// the snapshot shows the selection asked for and the track is listed,
    /// the row alone is selected and its index is returned, to scroll to.
    /// The request lapses when the library no longer holds the track.
    pub(crate) fn take_reveal(&mut self, state: &AppState) -> Option<usize> {
        let reveal = self.reveal.as_ref()?;
        let view = &state.library_view;
        let (track, selection) = (reveal.track, reveal.selection);
        // It lapses when the track is gone, or when the operator has
        // moved on before it landed: typed a search, changed the scope, or
        // closed the window or picked another category.
        let moved_on = !self.search.is_empty()
            || view.whole_library != reveal.against.whole_library
            || if view.selection == selection {
                !view.open
            } else {
                *view != *reveal.against
            };
        if moved_on || state.library.get(track).is_none() {
            self.reveal = None;
            return None;
        }
        if view.selection != selection {
            return None;
        }
        let at = self
            .rows
            .as_ref()
            .and_then(|c| c.rows.iter().position(|r| *r == track))?;
        self.reveal = None;
        self.selected.set_one(track);
        self.anchor = Some(track);
        Some(at)
    }

    /// Which computation of the rows the table shows (0: none yet).
    pub(crate) fn rows_revision(&self) -> u64 {
        self.rows.as_ref().map_or(0, |c| c.revision)
    }

    /// The selected tracks in the table's order; computed again only when
    /// the rows or the selection changed.
    pub(crate) fn selected_in_order(&self) -> Rc<[TrackId]> {
        let (rows, selection) = (self.rows_revision(), self.selected.revision());
        let listed = self.rows.as_ref().map_or(&[][..], |c| c.rows.as_slice());
        // Never borrowed elsewhere; if it were, the order is computed anew.
        let Ok(mut ordered) = self.ordered.try_borrow_mut() else {
            return selected_among(listed, &self.selected).into();
        };
        if let Some(o) = ordered.as_ref()
            && o.rows == rows
            && o.selection == selection
        {
            return o.tracks.clone();
        }
        let tracks: Rc<[TrackId]> = selected_among(listed, &self.selected).into();
        *ordered = Some(Ordered {
            rows,
            selection,
            tracks: tracks.clone(),
        });
        tracks
    }

    /// What a right-click or a drag on the row of `track` acts on: the
    /// selection when the row is in it; else the row alone, which becomes
    /// the selection.
    pub(crate) fn take_row(&mut self, track: TrackId) -> Rc<[TrackId]> {
        if !self.selected.contains(&track) {
            self.selected.set_one(track);
            self.anchor = Some(track);
        }
        self.selected_in_order()
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
                self.selected.push(*id);
            }
            return;
        }
        if modifiers.command {
            if self.selected.contains(&track) {
                self.selected.remove(track);
            } else {
                self.selected.push(track);
            }
        } else {
            self.selected.set_one(track);
        }
        self.anchor = Some(track);
    }
}

/// The tracks of `rows` that are in `selected`, in the order of `rows`.
pub(crate) fn selected_among(rows: &[TrackId], selected: &Selection) -> Vec<TrackId> {
    if selected.is_empty() {
        return Vec::new();
    }
    rows.iter()
        .copied()
        .filter(|r| selected.contains(r))
        .collect()
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
/// (`category_counts` walks the whole library: not every frame).
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
    /// *Remove missing tracks…* waiting for the answer (LB21): how many
    /// tracks of `category` it would remove, counted when it was asked.
    RemoveMissing { category: CategoryId, count: usize },
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

#[cfg(test)]
mod tests {
    use super::*;
    use fp_model::Command;
    use fp_model::library_view::LibrarySelection;
    use std::path::PathBuf;

    fn state_with(names: &[&str]) -> AppState {
        let mut s = AppState::new(fp_model::Config::default(), "Main");
        s.library_view.open = true;
        fp_model::apply(
            &mut s,
            Command::AddToLibrary {
                paths: names
                    .iter()
                    .map(|n| PathBuf::from(format!("/lib/{n}.mp3")))
                    .collect(),
                category: None,
            },
        )
        .unwrap();
        s
    }

    fn id(s: &AppState, name: &str) -> TrackId {
        s.library
            .track_at(&PathBuf::from(format!("/lib/{name}.mp3")))
            .unwrap()
    }

    /// The search index is folded only once a search needs it: opening the
    /// window on a large library costs no index build.
    #[test]
    fn the_search_index_is_built_when_a_search_needs_it() {
        let s = state_with(&["Alpha", "Beta"]);
        let mut library = LibraryUi::default();
        assert_eq!(library.rows(&s).len(), 2);
        assert!(library.rows.as_ref().unwrap().index.is_none(), "no search");
        library.search = "beta".into();
        assert_eq!(library.rows(&s), [id(&s, "Beta")]);
        assert!(library.rows.as_ref().unwrap().index.is_some());
        library.search.clear();
        assert_eq!(library.rows(&s).len(), 2);
        assert!(library.rows.as_ref().unwrap().index.is_some(), "kept");
    }

    #[test]
    fn a_reveal_waits_for_the_selection_then_selects_the_row_once() {
        let mut s = state_with(&["a", "b", "c"]);
        let b = id(&s, "b");
        let mut ui = LibraryUi {
            reveal: Some(Reveal::new(
                b,
                LibrarySelection::Uncategorised,
                &s.library_view,
            )),
            ..LibraryUi::default()
        };
        // The snapshot still shows All tracks.
        ui.rows(&s);
        assert_eq!(ui.take_reveal(&s), None, "not yet");
        assert!(ui.selected.is_empty());
        s.library_view.selection = LibrarySelection::Uncategorised;
        ui.rows(&s);
        let at = ui.take_reveal(&s).expect("listed now");
        assert_eq!(ui.rows(&s).get(at), Some(&b));
        assert_eq!(ui.selected, vec![b]);
        assert_eq!(ui.anchor, Some(b));
        assert_eq!(ui.take_reveal(&s), None, "only once");
    }

    #[test]
    fn a_reveal_lapses_when_the_track_leaves_the_library() {
        let s = state_with(&["a"]);
        let gone = TrackId(999_999);
        let mut ui = LibraryUi {
            reveal: Some(Reveal::new(gone, LibrarySelection::All, &s.library_view)),
            ..LibraryUi::default()
        };
        ui.rows(&s);
        assert_eq!(ui.take_reveal(&s), None);
        assert!(ui.reveal.is_none());
    }

    fn pending(s: &AppState, track: TrackId) -> LibraryUi {
        LibraryUi {
            reveal: Some(Reveal::new(
                track,
                LibrarySelection::Uncategorised,
                &s.library_view,
            )),
            ..LibraryUi::default()
        }
    }

    #[test]
    fn a_reveal_lapses_when_the_operator_picks_another_category_first() {
        let mut s = state_with(&["a", "b"]);
        let b = id(&s, "b");
        let mut ui = pending(&s, b);
        // The snapshot changes, to something else than the target.
        s.library_view.selection = LibrarySelection::Category(fp_model::CategoryId(7));
        ui.rows(&s);
        assert_eq!(ui.take_reveal(&s), None);
        assert!(ui.reveal.is_none(), "dropped");
    }

    #[test]
    fn a_reveal_lapses_when_the_operator_searches_or_closes_the_window() {
        let mut s = state_with(&["a", "b"]);
        let b = id(&s, "b");
        let mut ui = pending(&s, b);
        ui.search = "a".into();
        ui.rows(&s);
        assert_eq!(ui.take_reveal(&s), None);
        assert!(ui.reveal.is_none(), "a search drops it");

        let mut ui = pending(&s, b);
        s.library_view.selection = LibrarySelection::Uncategorised;
        s.library_view.open = false;
        ui.rows(&s);
        assert_eq!(ui.take_reveal(&s), None);
        assert!(ui.reveal.is_none(), "closing drops it");
    }

    #[test]
    fn a_reveal_lapses_when_the_scope_changes() {
        let mut s = state_with(&["a", "b"]);
        let b = id(&s, "b");
        let mut ui = pending(&s, b);
        s.library_view.whole_library = true;
        ui.rows(&s);
        assert_eq!(ui.take_reveal(&s), None);
        assert!(ui.reveal.is_none());
    }
}
