# Track library plan 3 — the library window Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A floating, non-modal library window on the desktop: a top-bar
tile, a category tree, a virtualised track table with search, menus and
drag and drop to playlists and carts, plus Settings → Library and the
playlists' *Show in library* and *Move to category ▸*.

**Architecture:** The window's saved state (`LibraryView`) lives in the
model and is saved in the store's key-value table like the layout. Search,
counts and sorting are pure functions in `fp-model` over an incremental
normalised index, so the UI only caches their results while the library
and the query stay the same. The UI is a new `ui/library/` module drawn
over the main window with `egui::Window`. It sends the plan-2 commands
(`CreateCategory`, `MoveTracksToCategory`, `RemoveTracks`, …) and the
existing playlist and cart ones.

**Tech Stack:** Rust, egui 0.37 / egui_kittest, rusqlite, `unicode-normalization` (new).

**Spec:** `docs/superpowers/specs/2026-11-01-track-library-design.md` (§7, §9, §10; LB24–LB31).

## Global Constraints

- Every user-visible string is a Fluent message in
  `crates/fp-app/locales/en-US/main.ftl`, with es-ES written and the other
  locales translated in the same PR.
- No hardcoded product limits (CLAUDE.md rule 4); no blocking on the UI
  thread (rule 8): file pickers and folder reads run on helper threads, as
  `AppUi::pick_files` does.
- `unwrap`/`expect`/`panic` are denied outside tests.
- The window is **desktop only**: on a phone (`view_state.phone_active`)
  there is no tile and no window (§11).
- LB28: only visible rows are laid out (`ScrollArea::show_rows`).
- LB30: matching is case- and accent-insensitive on title, artist, album,
  album artist, composer, genre, comment and file name; every word must
  match somewhere.
- LB31: with 20 000 tracks a search returns within a frame (16 ms) in
  release.
- New integration tests go into the crate's `tests/it/` with a `mod` line.
- Never mention other playout or radio-automation products.

## Review Focus

- A saved `LibraryView` that names a deleted category, a negative size or
  a position off the current screen: the window opens on *All tracks*,
  inside the window (Task 1 test, Task 3 test).
- Deleting the selected category from the tree: the selection falls back
  to *All tracks* (Task 1 test).
- The search text in another case and without accents, plus a word that
  only matches the file name (Task 2 tests).
- A removal or category deletion that touches a track on air is refused
  before any confirmation, with the holder named (Task 6 test).
- Dropping library rows on a cart with several rows selected: only the
  first track goes to that cart (Task 6 test).

## Rulings made while writing this plan

- **The library CUE route in Settings → Outputs (§9) moves to plan 4**,
  which creates `Holder::LibraryCue` and `outputs.library.cue`; a route
  control for a holder that does not exist yet would do nothing.
- **The preview bar (§7.2 item 1) is plan 4.** Plan 3 draws the window
  without it.
- **The ⟳ watched-folder mark and *Set/Clear watched folder* are plan 5**
  (plan 2 deferred `watched_folder`).
- **The library table has no `Number` column.** The playlists' first
  column is the entry's position; library rows have no position. The
  status icon (missing, unreadable, waiting for analysis) goes in a narrow
  leading cell that is always shown.
- **The search text is not saved;** only its scope is (LB26).
- **Accent folding uses `unicode-normalization`** (NFD, then combining
  marks dropped, then lower case), the usual way; `cargo deny check` must
  pass.
- **Show file location** opens the file's folder with
  `Platform::open_url` on its `file://` URL: no new dependency.
- **Dropping several rows on a cart** gives the cart the first one (a cart
  holds one file). *Add to cart ▸* with several rows fills the chosen cart
  and the next empty carts of the page, in grid order; the rest are left
  out with a notice.

---

### Task 1: The window's saved state in the model and the store

**Files:**
- Create: `crates/fp-model/src/library_view.rs`
- Modify: `crates/fp-model/src/lib.rs`, `state.rs`, `command.rs`, `reducer.rs`, `session.rs` (`RestoreParts`)
- Modify: `crates/fp-store/src/db/snapshot.rs` (KV `library_view`, next to `KV_LAYOUT`)
- Test: `crates/fp-model/tests/it/library_view.rs`, `crates/fp-store/tests/it/library_view_store.rs`

**Interfaces:**
- Produces:
  ```rust
  pub enum LibrarySelection { All, Uncategorised, Category(CategoryId) } // Default: All
  pub enum LibraryColumn { Tag(TableColumn), Category, Kind }
  pub struct LibraryColumnWidth { pub column: LibraryColumn, pub width: f32 }
  pub struct LibrarySort { pub column: LibraryColumn, pub ascending: bool }
  #[serde(default)] pub struct LibraryView {
      pub open: bool,                     // false
      pub rect: Option<[f32; 4]>,         // x, y, width, height in points; None: default placement
      pub minimised: bool,
      pub maximised: bool,
      pub selection: LibrarySelection,
      pub expanded: Vec<CategoryId>,      // LB27
      pub whole_library: bool,            // LB29 scope
      pub columns: Vec<LibraryColumnWidth>,
      pub sort: Option<LibrarySort>,
      pub tree_width: Option<f32>,
  }
  impl LibraryView { pub fn repaired(self, categories: &Categories) -> LibraryView }
  pub fn lenient_library_view(value: serde_json::Value) -> Option<LibraryView>
  ```
  `AppState::library_view: LibraryView`; `RestoreParts::library_view`;
  `Command::SetLibraryView(Box<LibraryView>)`.

- [ ] **Step 1: Write the failing model tests** in `library_view.rs`:
  `set_library_view_stores_it` (the state holds the sent view after
  `apply`); `a_deleted_selected_category_falls_back_to_all_tracks`
  (select a category, `DeleteCategory` it, `selection == All` and it is
  gone from `expanded`); `repaired_drops_unknown_categories_and_bad_sizes`
  (`selection: Category(unknown)` → `All`; `expanded` keeps only known ids;
  `rect` with a NaN or a width ≤ 0 → `None`; widths that are not finite or
  ≤ 0 are dropped; `tree_width` the same); `restore_repairs_the_saved_view`.
- [ ] **Step 2: Run** `cargo test -p fp-model --test it library_view` — FAIL (module missing).
- [ ] **Step 3: Implement** `library_view.rs`. `SetLibraryView` stores
  `view.repaired(&state.categories)`; `DeleteCategory` and `restore`
  repair the saved view too. `lenient_library_view` uses the same lenient
  field loading as `lenient_layout` (a bad field takes its default, the
  rest is kept).
- [ ] **Step 4: Write the failing store tests** in
  `library_view_store.rs`: `the_library_view_survives_a_reload` (save,
  reopen, equal) and `a_bad_library_view_loads_the_default_with_a_warning`
  (write `{"open": "yes", "rect": 3}` into the KV row; it loads with
  `open: false`, `rect: None` and a warning).
- [ ] **Step 5: Implement** the KV row `library_view`: written in the full
  save and in the change save when it differs, read with
  `lenient_library_view`, like `read_layout`.
- [ ] **Step 6: Run** `cargo test -p fp-model --test it library_view` and
  `cargo test -p fp-store --test it library_view_store` — PASS; then fmt,
  clippy and the whole suite.
- [ ] **Step 7: Commit** `feat(model): save the library window's state with the session`.

### Task 2: Search, counts and sorting

**Files:**
- Create: `crates/fp-model/src/library_search.rs`
- Modify: `crates/fp-model/src/lib.rs`, `crates/fp-model/Cargo.toml`, `Cargo.toml` (workspace dependency `unicode-normalization`)
- Test: `crates/fp-model/tests/it/library_search.rs`

**Interfaces:**
- Consumes: `LibrarySelection`, `LibraryColumn`, `LibrarySort` (Task 1).
- Produces:
  ```rust
  pub fn fold(text: &str) -> String; // NFD, combining marks dropped, lower case
  pub struct SearchIndex { /* TrackId → folded haystack */ }
  impl SearchIndex {
      pub fn build(library: &Library) -> Self;
      /// Re-folds only what `library.changes_since(old)` reports.
      pub fn update(&mut self, old: &Library, library: &Library);
      pub fn matches(&self, track: TrackId, words: &[String]) -> bool;
  }
  pub fn query_words(query: &str) -> Vec<String>; // folded, split on whitespace
  pub fn library_rows(library: &Library, categories: &Categories, index: &SearchIndex,
                      selection: LibrarySelection, whole_library: bool, query: &str) -> Vec<TrackId>;
  pub fn sort_rows(rows: &mut [TrackId], library: &Library, categories: &Categories, sort: LibrarySort);
  pub struct CategoryCounts { pub all: usize, pub uncategorised: usize, pub by_category: HashMap<CategoryId, usize> }
  pub fn category_counts(library: &Library, categories: &Categories) -> CategoryCounts;
  ```
  The haystack is the folded title, artist, album, album artist, composer,
  genre, comment and file name, separated by `\n`.

- [ ] **Step 1: Write the failing tests:**
  `fold_drops_accents_and_case` (`fold("Canción ÀÉÎ") == "cancion aei"`);
  `a_search_is_case_and_accent_insensitive` ("cancion" finds "Canción");
  `every_word_must_match_somewhere` ("cancion lopez" finds title "Canción"
  with artist "López", not a track with only one of them);
  `the_file_name_is_searched` ("demo_take3" finds `/m/demo_take3.wav`);
  `the_search_covers_the_selected_category_and_its_subcategories`
  (parent selected: its tracks and a child's, not a sibling's);
  `whole_library_searches_everything`;
  `uncategorised_shows_tracks_without_a_category`;
  `an_empty_query_lists_the_whole_scope`;
  `update_follows_an_edited_title` (index built, a title changed, `update`,
  the new word matches and the old does not);
  `counts_include_subcategories` (parent count = own + children's;
  `all` = every track; `uncategorised` = no category);
  `sort_rows_orders_by_a_column_and_direction` (title ascending and
  descending; `Category` by category name, uncategorised last; ties by
  track id); `searching_20000_tracks_takes_less_than_a_frame`
  (`#[cfg_attr(debug_assertions, ignore)]`: build the index outside the
  timer, then `library_rows` with a two-word query in under 16 ms).
- [ ] **Step 2: Run** `cargo test -p fp-model --test it library_search` — FAIL.
- [ ] **Step 3: Implement** `library_search.rs`; add
  `unicode-normalization` to the workspace dependencies and fp-model;
  `cargo deny check` passes.
- [ ] **Step 4: Run** the tests — PASS, and the timing test in release:
  `cargo test --release -p fp-model --test it library_search -- --include-ignored`.
- [ ] **Step 5: Commit** `feat(model): search, count and sort the library`.

### Task 3: The Library tile and the floating window

**Files:**
- Create: `crates/fp-app/src/ui/library/mod.rs` (window frame and state sync), `crates/fp-app/src/ui/library/state.rs` (`LibraryUi`, the runtime state in `ViewState`)
- Modify: `crates/fp-app/src/ui/panels.rs` (`layout_controls`: the tile after the Cartwall tile, before the layout menu), `crates/fp-app/src/ui/app.rs` (`ViewState::library`, the draw call), `crates/fp-app/src/ui.rs` (the `library` module), both locale files
- Test: `crates/fp-app/tests/it/library_window.rs`

**Interfaces:**
- Consumes: `LibraryView`, `Command::SetLibraryView` (Task 1).
- Produces: `pub(crate) struct LibraryUi` (search text, row selection
  `Vec<TrackId>` with an anchor, the rows cache, the open dialog);
  `pub(crate) fn draw(ctx: &egui::Context, scene: &Scene<'_>, view_state: &mut ViewState)`;
  `pub(crate) fn body_rect(...)` is up to the implementer. The window's
  content area is handed to Tasks 4 and 5 as two closures, the left
  (tree) and right (table) halves of a resizable split whose width is
  `LibraryView::tree_width` (default 220 points).

Behaviour: the tile reads "Library" (`top-library`), is lit while the window is open (the shown style of
`panel_tile`) and toggles `LibraryView::open`. The window is an
`egui::Window` without egui's title bar, with our own 28-point title bar:
the title, then **minimise**, **maximise/restore** and **close** buttons
(phosphor `MINUS`, `CORNERS_OUT`/`CORNERS_IN`, `X`). It is moved by
dragging its title bar and resized from its edges (minimum 480 × 320).
Default placement: centred in the area between the top bar and the status
bar, 70 % of its width and height. Minimised shows only the title bar.
Maximised fills that area exactly. The saved `rect` is clamped into the
area every frame, so the window never sits off screen. A changed view is
sent as one `SetLibraryView` when the move or resize ends, not every
frame.

- [ ] **Step 1: Write the failing kittest tests:**
  `the_library_tile_opens_and_closes_the_window` (LB24: the tile sends
  `SetLibraryView` with `open` toggled; the window's title is shown when
  `open`); `the_tile_is_not_on_a_phone`;
  `players_keep_working_while_the_library_is_open` (LB25: with the window
  open, a player's Play button still sends `Play`);
  `minimise_keeps_only_the_title_bar` and `maximise_fills_the_area_and_restores`
  (the sent view's flags); `close_closes_the_window`;
  `a_saved_rect_off_screen_is_drawn_inside_the_window` (LB26: `rect` at
  x = 5000; the window's rect lies inside the screen).
- [ ] **Step 2: Run** `cargo test -p fp-app --test it library_window` — FAIL.
- [ ] **Step 3: Implement** the tile and the window.
- [ ] **Step 4: Run** — PASS; fmt, clippy, whole suite.
- [ ] **Step 5: Commit** `feat(ui): a Library tile and its floating window`.

### Task 4: The category tree

**Files:**
- Create: `crates/fp-app/src/ui/library/tree.rs`, `crates/fp-app/src/ui/library/category_dialog.rs` (create, rename, colour, kind), `crates/fp-app/src/ui/library/confirm.rs` (the two removal confirmations, shared with Task 6)
- Modify: `crates/fp-app/src/ui/library/mod.rs`, both locale files
- Test: `crates/fp-app/tests/it/library_tree.rs`

**Interfaces:**
- Consumes: `category_counts`, `CategoryCounts` (Task 2); `category_deletion`,
  `removal_impact`, `RemovalImpact`, `OnAirHolds` (plan 2,
  `fp_model::library_rules`); the category commands of plan 2.
- Produces:
  ```rust
  pub(crate) enum TreeMode { Window, Settings } // Settings: no All / Uncategorised items, no selection
  pub(crate) fn category_tree(ui: &mut Ui, scene: &Scene<'_>, view_state: &mut ViewState, mode: TreeMode);
  pub(crate) enum Removal { Tracks(Vec<TrackId>), Category(CategoryId) }
  pub(crate) fn ask_removal(scene: &Scene<'_>, view_state: &mut ViewState, removal: Removal);
  pub(crate) fn draw_confirmations(ctx: &egui::Context, scene: &Scene<'_>, view_state: &mut ViewState);
  ```

Behaviour: items are *All tracks*, the categories in tree order, and
*Uncategorised*. Each has its colour swatch (the cart palette,
`CartColor::ALL`) and its count (own and subcategories'). A category with
children has a disclosure arrow; expanded ids are `LibraryView::expanded`
(LB27). A click selects (`LibraryView::selection`). **＋** creates a
category under the selected one (top level when *All tracks* or
*Uncategorised* is selected). The right-click menu has **New
subcategory…**, **Rename…**, **Colour ▸**, **Kind ▸** (the `TrackKind`
list), **Delete…**. A category dragged onto another becomes its last
child; dragged into the gap between two siblings, it goes there
(`MoveCategory`). A rejected command (LB9, LB10) shows its error as a
notice, as other model errors do. **Delete…** calls `ask_removal`:
refused at once when `on_air` is not empty (the notice names the entries
and carts, `error-library-on-air`); else the first confirmation gives the
number of subcategories and tracks; the second, only when entries or
carts are affected, gives their numbers; confirmed, `DeleteCategory` is
sent.

- [ ] **Step 1: Write the failing kittest tests:** `the_tree_lists_all_categories_and_uncategorised_with_counts`;
  `selecting_a_category_sends_the_selection`;
  `collapsing_a_category_hides_its_children_and_saves_it` (LB27);
  `plus_creates_a_category_under_the_selected_one`;
  `rename_sends_the_trimmed_name` and `a_taken_name_shows_the_error`;
  `colour_and_kind_menus_send_their_commands`;
  `dropping_a_category_on_another_moves_it_under_it`;
  `deleting_an_unused_category_asks_once`;
  `deleting_a_used_category_asks_twice_with_the_counts`;
  `deleting_a_category_on_air_is_refused_before_asking`.
- [ ] **Step 2: Run** `cargo test -p fp-app --test it library_tree` — FAIL.
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run** — PASS; fmt, clippy, whole suite.
- [ ] **Step 5: Commit** `feat(ui): the library's category tree`.

### Task 5: The track table and the search

**Files:**
- Create: `crates/fp-app/src/ui/library/table.rs`
- Modify: `crates/fp-app/src/ui/library/mod.rs`, `crates/fp-app/src/ui/app.rs` (`Picked` gains a target: a playlist position or a library category; file drops on the window go to the library), both locale files
- Test: `crates/fp-app/tests/it/library_table.rs`

**Interfaces:**
- Consumes: `SearchIndex`, `library_rows`, `sort_rows` (Task 2);
  `LibraryView::{columns, sort, whole_library, selection}` (Task 1); the
  playlists' cell rendering and status flags in `ui/table.rs`, reused
  rather than copied (move a shared helper into `ui/table_layout.rs` or a
  new `ui/track_cells.rs` when needed).
- Produces: `LibraryUi::rows(&mut self, state: &AppState) -> &[TrackId]`
  (cached; recomputed only when the library storage
  (`Library::shares_storage`), the categories, the selection, the scope,
  the query or the sort changed; the index is kept up to date with
  `SearchIndex::update`); `pub(crate) enum PickTarget { Playlist { playlist: PlaylistId, index: usize }, Library(Option<CategoryId>) }`.

Behaviour: above the table, the search field (filters as you type, LB29)
and the **Whole library** toggle. Columns: the leading status cell, then
`ui.table_columns` without `Number`, then **Category** and **Kind**; the
widths are `LibraryView::columns`; a header click sorts by that column,
again reverses (`LibraryView::sort`). Rows are drawn with
`ScrollArea::show_rows` (LB28). Click selects, Ctrl/Cmd-click adds, and
Shift-click extends from the anchor. **Add files** (button by the search
field) picks files on a helper thread and sends `AddToLibrary` with the
selected category (`None` for *All tracks* and *Uncategorised*). Files
dropped on the window do the same, and folders are read on the helper
thread with `audio_paths`, as for playlists.

- [ ] **Step 1: Write the failing tests:**
  `the_table_shows_the_selected_categorys_tracks_and_its_subcategories`;
  `typing_filters_as_you_type` (LB29);
  `whole_library_searches_beyond_the_category` and the toggle sends the
  scope in `SetLibraryView`;
  `only_visible_rows_are_laid_out` (LB28: 20 000 tracks, the harness lays
  out fewer than 100 row labels);
  `clicking_a_header_sorts_and_again_reverses`;
  `shift_click_selects_a_range`;
  `a_missing_track_shows_the_missing_flag`;
  `files_dropped_on_the_window_join_the_selected_category`
  (`AddToLibrary { category: Some(c) }`);
  `files_dropped_with_all_tracks_selected_are_uncategorised`.
- [ ] **Step 2: Run** `cargo test -p fp-app --test it library_table` — FAIL.
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run** — PASS; fmt, clippy, whole suite.
- [ ] **Step 5: Commit** `feat(ui): the library's track table and search`.

### Task 6: Row menu and drag and drop to playlists and carts

**Files:**
- Create: `crates/fp-app/src/ui/library/menu.rs`
- Modify: `crates/fp-app/src/ui/library/table.rs`, `crates/fp-app/src/ui/app.rs` (`DragTracks`), `crates/fp-app/src/ui/table.rs` and `crates/fp-app/src/ui/player.rs` (the playlist table and the playlist tabs accept `DragTracks`), `crates/fp-app/src/ui/cartwall.rs` (a cart accepts `DragTracks`), both locale files
- Test: `crates/fp-app/tests/it/library_menu.rs`

**Interfaces:**
- Consumes: `ask_removal`, `Removal` (Task 4); `LibraryUi` selection (Task 5); `RowMenuOutcome::{EditTags, Loudness}` of `ui/table.rs`, reused.
- Produces: `pub(crate) struct DragTracks { pub tracks: Vec<TrackId> }`.

Behaviour: the menu acts on the selected rows (the right-clicked row joins
the selection, or replaces it when it was not selected):
**Add to ▸** every playlist (`InsertTracks` at its end), **Add to cart ▸**
the shown page's empty carts (ruling above), **Move to category ▸** the
categories in tree order and **Uncategorised** (`MoveTracksToCategory`),
**Kind ▸** (`SetTracksKind`), **Edit tags…** and **Loudness…** (one row
only, as on a playlist), **Re-analyse**, **Reset markers to automatic**
(`ResetMarkers` per track), **Remove from library…** (`ask_removal`,
LB17), **Show file location** (one row only). Dragging rows starts a
`DragTracks` with the selection. Released on a playlist row it sends
`InsertTracks` at the drop line, on a playlist tab it sends `InsertTracks`
at that playlist's end, and on a cart `AssignCartTrack` with the first
track.

- [ ] **Step 1: Write the failing tests:** `add_to_sends_insert_tracks_at_the_end`;
  `add_to_cart_fills_the_chosen_and_next_empty_carts`;
  `move_to_category_sends_the_selected_tracks`;
  `edit_tags_is_offered_for_one_row_only`;
  `remove_from_library_asks_twice_when_tracks_are_used`;
  `remove_from_library_is_refused_while_on_air`;
  `dragging_rows_onto_a_playlist_inserts_them_at_the_drop_line`;
  `dragging_rows_onto_a_cart_assigns_the_first`.
- [ ] **Step 2: Run** `cargo test -p fp-app --test it library_menu` — FAIL.
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run** — PASS; fmt, clippy, whole suite.
- [ ] **Step 5: Commit** `feat(ui): send library tracks to playlists and carts`.

### Task 7: Playlist menu entries and Settings → Library

**Files:**
- Create: `crates/fp-app/src/ui/settings/library.rs`
- Modify: `crates/fp-app/src/ui/table.rs` (`context_menu`), `crates/fp-app/src/ui/settings.rs` (`Section::Library`, its tab), `crates/fp-app/src/ui/library/mod.rs` (a *show track* request), both locale files
- Test: `crates/fp-app/tests/it/library_links.rs`

**Interfaces:**
- Consumes: `category_tree(.., TreeMode::Settings)` (Task 4).
- Produces: `pub(crate) fn show_track(view_state: &mut ViewState, state: &AppState, track: TrackId)`. It opens the window and selects the track's category (or *Uncategorised*). It clears the search, selects the row and scrolls to it.

- [ ] **Step 1: Write the failing tests:**
  `show_in_library_opens_the_window_on_the_tracks_category`;
  `move_to_category_from_a_playlist_sends_the_entrys_track`;
  `settings_library_lists_the_categories_and_creates_one`.
- [ ] **Step 2: Run** `cargo test -p fp-app --test it library_links` — FAIL.
- [ ] **Step 3: Implement.** The playlist menu gains **Show in library**
  and **Move to category ▸** after **Loudness…**, on the desktop only.
  Settings → Library comes after Cartwall in the tab list (phosphor
  `BOOKS`) and shows the tree in `TreeMode::Settings` with its ＋.
- [ ] **Step 4: Run** — PASS; fmt, clippy, whole suite.
- [ ] **Step 5: Commit** `feat(ui): reach the library from playlists and Settings`.

### Task 8: Documentation and translations

**Files:**
- Create: `docs/user/library.md` (and its line in `docs/user/SUMMARY.md`)
- Modify: `README.md` (features), `docs/user/settings.md`, `docs/user/playlists.md`, `docs/user/layout.md` (or the page that lists the top bar), `docs/technical/ui.md`, `docs/technical/persistence.md` (the `library_view` row), every locale file under `crates/fp-app/locales/` (the strings of Tasks 3–7 translated), the spec's §16 note that the route and the preview bar are plan 4

- [ ] **Step 1:** Write the user page: opening the window, the tree, the
  search and its scope, the menus, dragging to playlists and carts, removal
  and its confirmations. Describe only what plan 3 ships.
- [ ] **Step 2:** Update the technical docs and the README.
- [ ] **Step 3:** Translate the new strings in every locale; run the
  locale tests (`cargo test -p fp-app --test it i18n`) and
  `scripts/site/check-translations.sh` if it covers the guide.
- [ ] **Step 4:** fmt, clippy, whole suite; commit `docs: the library window`.
