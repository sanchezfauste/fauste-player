# Track library, plan 2: categories in the model and the store — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give the library a tree of categories, keep tracks in the library when nothing uses them, hold one track per file, and add the commands that move, re-kind and remove tracks and categories — in `fp-model` and `fp-store`, with no window yet.

**Architecture:** A new `fp-model` module `category.rs` holds `Category`, `Categories` (a `Vec` in tree order) and the pure LB9/LB10 checks; `AppState` gains `categories`, `Track` gains `category`. `Library` gains a path index so every path that enters the model reuses its track (LB14). The reducer's `forget_unreferenced` becomes `release_tracks`, which only applies LB17c. `fp-store` schema 2 adds a `categories` table and a `tracks.category` column; the snapshot reads and writes them. Paths are canonicalised where they are produced (fp-app helper threads), never in the model.

**Tech Stack:** Rust, `imbl`, `rusqlite`, serde_json, Fluent.

**Spec:** `docs/superpowers/specs/2026-11-01-track-library-design.md` §5 (LB9–LB18), §10 (first bullet), §13 (`library.max_category_depth`), §14, §16 plan 2.

## Global Constraints

- All code, comments, docs and commit messages in English; Conventional Commits, scopes `model`, `store`, `app`, `docs`.
- No `unsafe`; no `unwrap`/`expect`/`panic` outside tests; prefer `get` over indexing.
- Behaviour lives in `fp-model` as pure functions; one test per rule (LB9–LB18) in `crates/fp-model/tests/it/`.
- `library.max_category_depth`: default 8, range 1–32, lenient loading; a root category is depth 1.
- Category names: trimmed, non-empty; siblings may not share a name compared with `str::to_lowercase`.
- A track's file is never touched by any command of this plan.
- Never crash on bad data: an unknown category id on a track loads as uncategorised, a category whose parent is unknown or that closes a cycle loads at the root, each with one warning.
- Every new or changed UI string goes into every `crates/fp-app/locales/*/main.ftl` (en-US and es-ES written, the rest translated).
- One integration-test binary per crate: new test files go in `crates/<crate>/tests/it/` with a `mod` line in `tests/it/main.rs`.
- Commit only with `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` green.
- Never mention other playout or radio-automation products anywhere.

## Review Focus

1. **An existing plan-1 database (schema 1)** opens with every track uncategorised and nothing lost, and is written back as schema 2. → Task 4 `a_schema_1_database_opens_with_every_track_uncategorised`.
2. **The same file added from a playlist and from a cart** is one track: a marker set on one shows on the other. → Task 2 `the_same_path_reuses_its_track_everywhere`.
3. **Deleting a category while one of its tracks is on air** (or a cart of it plays) is refused before anything changes, naming the holder. → Task 3 `a_deletion_touching_an_on_air_entry_is_refused_whole`.
4. **A database with two tracks for one path** (plan 1 created a new track on every add) loads as one track, the entries and carts re-pointed, the operator's edits kept. → Task 2 `restore_merges_tracks_that_share_a_path`, Task 4 `duplicate_paths_are_merged_on_reload`.
5. **A missing file**: an uncategorised unused track leaves with a log line; a categorised or used one stays. → Task 2 `missing_uncategorised_unused_tracks_leave`, Task 5 `services_log_a_track_that_left_the_library`.

## Rulings made while writing this plan

- `Category::order` from the spec is the position in `Categories`' `Vec` (siblings keep their relative order); it is stored as the table's `position`. — simpler, same behaviour — a reorder costs rewriting the rows after it.
- `watched_folder` is added in plan 5 (serde default), not now. — YAGNI.
- The model compares paths as given; canonicalisation (`std::fs::canonicalize`, falling back to the path as given) happens in fp-app's helper threads, which already read the file system. Paths stored before this plan are kept as written. — the model stays pure and the UI never blocks — a symlinked path saved earlier is not merged with its canonical twin.
- Uniqueness of paths is enforced by the model (and merged at restore), not by a unique SQL index: schema 2 adds a plain `tracks_by_path` index. The migrations stay SQL strings (no Rust migrations needed yet). — avoids a Rust migration that would need to rewrite cart page documents — a model bug could write a duplicate, which the next start merges anyway.
- *Add to library* with a category: a new track takes the category (and its kind); an existing uncategorised track joins it; a track already in another category keeps its own (as LB20 will for watched folders).
- When duplicates merge, the survivor is the lowest id among the tracks with an operator edit (a manual marker, a loudness edit other than the default, or a category), else the lowest id.
- The model cannot log: the services thread logs LB17c removals by noticing a track that left while missing and uncategorised.

---

## File Structure

| File | Responsibility |
|---|---|
| `crates/fp-model/src/category.rs` (new) | `Category`, `Categories`, LB9/LB10 checks, subtree and depth helpers |
| `crates/fp-model/src/library_rules.rs` (new) | reducer helpers for LB11–LB17c: add paths, release, move, re-kind, removal impact, remove, delete category, merge duplicates |
| `crates/fp-model/src/ids.rs` | `CategoryId`, `IdGen::category` |
| `crates/fp-model/src/track.rs` | `Track::category`; `Library` path index |
| `crates/fp-model/src/state.rs`, `session.rs` | `AppState::categories`; restore takes categories, merges duplicates, keeps unreferenced tracks |
| `crates/fp-model/src/command.rs`, `reducer.rs`, `cart_rules.rs`, `error.rs` | new commands and errors; every track creation goes through the library |
| `crates/fp-model/src/config.rs` | `LibraryConfig { max_category_depth }` |
| `crates/fp-store/src/db/schema.rs`, `db/snapshot.rs`, `store.rs` | schema 2, categories rows, `tracks.category` |
| `crates/fp-app/src/ui/files.rs`, `ui/app.rs`, `ui/settings/carts.rs`, `services.rs` | canonical paths at the edges; LB17c log line; error strings |
| `crates/fp-app/locales/*/main.ftl` | error strings |
| `docs/technical/persistence.md`, `docs/technical/architecture.md`, `docs/user/data-and-backups.md`, `docs/user/playlists.md` | documentation |

---

### Task 1: Categories in the model (LB9, LB10, config)

**Files:**
- Create: `crates/fp-model/src/category.rs`, `crates/fp-model/tests/it/categories.rs`
- Modify: `crates/fp-model/src/ids.rs`, `lib.rs`, `track.rs` (field only), `state.rs`, `session.rs` (`RestoreParts`), `command.rs`, `reducer.rs`, `error.rs`, `config.rs`; `crates/fp-app/src/ui/app.rs` (error text), `crates/fp-app/locales/*/main.ftl`; every compile site of `AppState`/`RestoreParts`/`Track` literals in other crates.

**Interfaces:**
- Produces:
  - `pub struct CategoryId(pub u64)` via `id_type!`; `IdGen::category(&mut self) -> CategoryId`.
  - `pub struct Category { pub id: CategoryId, pub parent: Option<CategoryId>, pub name: String, pub color: Option<CartColor>, pub kind: TrackKind }` (serde, `#[serde(default)]` on optional fields).
  - `pub struct Categories { list: Vec<Category> }` with `get(id) -> Option<&Category>`, `iter()`, `children(parent: Option<CategoryId>) -> impl Iterator<Item=&Category>` (in order), `depth(id) -> usize` (root = 1), `subtree(id) -> Vec<CategoryId>` (id first, then descendants), `is_in_subtree(id, root) -> bool`, `len()`, `max_raw_id()`, `From<Vec<Category>>`, `into_vec()`, and `repaired(self) -> (Categories, Vec<String>)` (unknown parent or cycle → root, one warning each).
  - `pub fn check_name(cats: &Categories, parent: Option<CategoryId>, name: &str, except: Option<CategoryId>) -> Result<String, ModelError>` returning the trimmed name.
  - `Track::category: Option<CategoryId>` (`#[serde(default)]`); `Track::new` sets `None`.
  - `AppState::categories: Categories`; `RestoreParts::categories: Categories`.
  - `Config::library: LibraryConfig { max_category_depth: usize }` (default 8, range 1–32).
  - Commands: `CreateCategory { parent: Option<CategoryId>, name: String, color: Option<CartColor>, kind: TrackKind }`, `RenameCategory { category: CategoryId, name: String }`, `SetCategoryColor { category, color: Option<CartColor> }`, `SetCategoryKind { category, kind: TrackKind }`, `MoveCategory { category, parent: Option<CategoryId>, index: usize }` (index among the new siblings, clamped).
  - Errors: `UnknownCategory(CategoryId)`, `CategoryNameEmpty`, `CategoryNameTaken`, `CategoryTooDeep`, `CategoryIntoItself`.

- [x] **Step 1: Write the failing tests** in `crates/fp-model/tests/it/categories.rs`, using `AppState::new(Config::default(), "P")` and `apply`:
  - `names_are_trimmed_and_must_not_be_empty` (LB9): create `"  Gold "` → name `"Gold"`; create `"   "` → `Err(CategoryNameEmpty)`.
  - `siblings_cannot_share_a_name_ignoring_case` (LB9): root `"Gold"` then root `"gold"` → `Err(CategoryNameTaken)`; `"gold"` under another parent → Ok; renaming `"Gold"` to `"GOLD"` → Ok (itself excepted).
  - `depth_is_limited_by_config` (LB9): with `max_category_depth = 2`, a chain of 2 is Ok, a third level `Err(CategoryTooDeep)`; moving a 2-deep subtree under a root category → `Err(CategoryTooDeep)`.
  - `a_category_cannot_move_under_itself` (LB10): A ⊃ B; `MoveCategory { category: A, parent: Some(B), .. }` → `Err(CategoryIntoItself)`; also `parent: Some(A)`.
  - `move_category_sets_parent_and_sibling_order`: roots A, B, C; move C to index 0 at the root → children order C, A, B.
  - `changing_a_category_kind_does_not_change_its_tracks` (LB13, second half): a track with `category = Some(A)`, kind Music; `SetCategoryKind { A, Ad }` → track still Music.
  - `repaired_moves_orphans_and_cycles_to_the_root`: `Categories::from(vec![…])` with an unknown parent and a two-node cycle → all at the root, 2 or 3 warnings, no panic.
  - A config test in the existing config test file: `library.max_category_depth` 0 and 99 are clamped into 1–32 with a warning; a missing `library` section loads the default.
- [x] **Step 2: Run** `cargo test -p fp-model --test it categories::` — expect compile failure.
- [x] **Step 3: Implement** the types, the checks and the five commands (`MoveCategory` removes the category from the `Vec` and inserts it after the new parent's subtree position matching `index` among siblings; its whole subtree moves with it, so descendants stay after it). Add the error texts (`error-category-name-empty`, `error-category-name-taken`, `error-category-too-deep`, `error-category-into-itself`, `error-unknown-category`) to the `ModelError` match in `crates/fp-app/src/ui/app.rs` and to every locale file.
- [x] **Step 4: Run** the focused tests, then fmt, clippy and the workspace suite.
- [x] **Step 5: Commit** `feat(model): add a tree of track categories`.

---

### Task 2: Tracks stay in the library, one per file (LB14, LB15, LB16, LB17c)

**Files:**
- Create: `crates/fp-model/src/library_rules.rs`, `crates/fp-model/tests/it/library_lifetime.rs`
- Modify: `crates/fp-model/src/track.rs`, `reducer.rs` (`insert_paths`, `forget_unreferenced`, `SetFileState`, `CreatePlaylistFromPaths`), `cart_rules.rs` (`assign_file`, `forget_tracks`, `import_page`, `delete_page`, `set_track`), `session.rs` (restore), `command.rs`

**Interfaces:**
- Consumes: Task 1 (`Track::category`, `Categories`).
- Produces:
  - `Library::track_at(&self, path: &Path) -> Option<TrackId>`, backed by an `imbl::OrdMap<PathBuf, TrackId>` kept in step by `insert`, `remove` and `get_mut` callers (`get_mut` may change `path`: re-index through a new `Library::set_path(id, PathBuf)` instead, and make the `path` field change only there — check every writer of `track.path`).
  - `library_rules::track_for_path(state: &mut AppState, path: PathBuf) -> TrackId` — the existing track for that path, or a new uncategorised one.
  - `library_rules::release_tracks(state: &mut AppState, tracks: impl IntoIterator<Item = TrackId>)` — LB17c: removes a track only if it is uncategorised, `FileState::Missing`, and no entry or cart uses it.
  - `library_rules::merge_duplicate_paths(state: &mut AppState) -> usize` — merges tracks that share a path (survivor per the plan's ruling), re-points playlist entries and carts, returns how many tracks were merged away.
  - Command `AddToLibrary { paths: Vec<PathBuf>, category: Option<CategoryId> }` (ruling above; a new track in a category takes its kind, LB13).

- [x] **Step 1: Write the failing tests** in `library_lifetime.rs`:
  - `the_same_path_reuses_its_track_everywhere` (LB14): `InsertPaths` of `/m/a.wav` into a playlist twice and `AssignCartFile` with the same path → library `len() == 1`, both entries and the cart point at the same `TrackId`; a `SetMarker` on it is seen through the cart's track.
  - `a_track_stays_when_its_last_entry_goes` (LB15, LB16): insert, `RemoveEntry`, `DeletePlaylist` (another list), `ClearCartFile`, `DeleteCartPage` → the track is still in the library, uncategorised.
  - `missing_uncategorised_unused_tracks_leave` (LB17c): an unused uncategorised track → `SetFileState { Missing }` removes it; the same track with a category, or used by an entry, stays; and a used uncategorised missing track leaves when its last entry is removed.
  - `add_to_library_takes_the_category_and_its_kind` (LB16, LB13): `AddToLibrary { [a], Some(Ads) }` with Ads kind `Ad` → track category Ads, kind Ad; again with `Some(Gold)` → keeps Ads; an uncategorised existing track joins.
  - `restore_merges_tracks_that_share_a_path`: `RestoreParts` with tracks 1 and 2 on the same path, track 2 with a manual cue-in, an entry on 1 and a cart on 2 → one track (id 2), both pointing at it, manual cue-in kept.
  - `restore_keeps_unreferenced_tracks` (LB15): a track no entry or cart uses survives `AppState::restore`.
- [x] **Step 2: Run** `cargo test -p fp-model --test it library_lifetime::` — expect failures.
- [x] **Step 3: Implement.** Every `Track::new` + `library.insert` in the reducer and `cart_rules` goes through `track_for_path`. `forget_unreferenced`/`forget_tracks` become `release_tracks`. `import_page`'s "position given twice" and "outside the grid" branches use `release_tracks` (the track may be shared now). `SetFileState` with `Missing` calls `release_tracks([track])` after its existing work. Restore: drop the orphan sweep, call `merge_duplicate_paths`. Update the doc comment of `cart_rules::set_track` ("forgets the old one if unused").
- [x] **Step 4: Run** the focused tests, then fmt, clippy and the workspace suite (existing tests that expected forgetting change: update them to the new rule and say so in the commit body).
- [x] **Step 5: Commit** `feat(model)!: keep tracks in the library and hold one track per file`.

---

### Task 3: Move, re-kind and remove tracks; delete categories (LB11, LB12, LB13, LB17, LB17b)

**Files:**
- Modify: `crates/fp-model/src/library_rules.rs`, `command.rs`, `reducer.rs`, `error.rs`; `crates/fp-app/src/ui/app.rs`, locales
- Create: `crates/fp-model/tests/it/library_removal.rs`

**Interfaces:**
- Consumes: Tasks 1–2; the reducer's `is_on_air`, `successors_of`, `detach_entries` (as `remove_entry` uses them) and `cart_rules`' cart-playing checks.
- Produces:
  - `pub struct RemovalImpact { pub tracks: usize, pub entries: usize, pub carts: usize, pub on_air: OnAirHolds }` and `pub struct OnAirHolds { pub entries: Vec<EntryId>, pub carts: Vec<CartId> }` with `is_empty()`.
  - `pub fn removal_impact(state: &AppState, tracks: &[TrackId]) -> RemovalImpact` (on-air entries per `is_on_air`; carts playing or pre-listened).
  - `pub fn category_deletion(state: &AppState, category: CategoryId) -> Option<(usize /* subcategories */, RemovalImpact)>`.
  - Commands: `MoveTracksToCategory { tracks: Vec<TrackId>, category: Option<CategoryId> }`, `SetTracksKind { tracks: Vec<TrackId>, kind: TrackKind }`, `RemoveTracks(Vec<TrackId>)`, `DeleteCategory(CategoryId)`.
  - Error: `LibraryOnAir(OnAirHolds)`.

- [x] **Step 1: Write the failing tests** in `library_removal.rs`:
  - `moving_tracks_sets_their_category_and_kind` (LB12, LB13): two tracks into Ads (kind Ad) → both category Ads, kind Ad; to `None` → uncategorised, kind unchanged; an unknown category → `Err(UnknownCategory)` and nothing changed.
  - `the_kind_can_be_changed_on_the_track` (LB13): after joining Ads, `SetTracksKind { Jingle }` → Jingle.
  - `removal_impact_counts_entries_and_carts` (LB17): a track in 2 entries and 1 cart → `{tracks: 1, entries: 2, carts: 1, on_air empty}`.
  - `remove_tracks_removes_their_entries_and_empties_their_carts` (LB17): after `RemoveTracks`, no entry or cart references them; the carts keep their name and colour with `track: None`; the next of a player whose next entry went is the entry after it.
  - `a_deletion_touching_an_on_air_entry_is_refused_whole` (LB17b): one of two tracks is current on a player (`Play`) → `RemoveTracks` and `DeleteCategory` both `Err(LibraryOnAir { entries: [that entry], .. })`, and state is unchanged (`assert_eq!` with a clone taken before); likewise a playing cart (`FireCart`) and a cart pre-listen.
  - `deleting_a_category_removes_its_subtree_and_tracks` (LB11): A ⊃ B, tracks in A and B, one in C → after `DeleteCategory(A)` only C and its track remain; `category_deletion(A)` beforehand reports 1 subcategory and the right impact.
- [x] **Step 2: Run** `cargo test -p fp-model --test it library_removal::` — expect failures.
- [x] **Step 3: Implement.** Check holds first, then remove: entries via the same path as `remove_entry` (successors, `detach_entries`, `refresh_next`), carts emptied as `ClearCartFile` does, then the tracks, then (for `DeleteCategory`) the categories. Add `error-library-on-air` (it names how many entries and carts hold it) to the error match and every locale.
- [x] **Step 4: Run** the focused tests, fmt, clippy, the workspace suite.
- [x] **Step 5: Commit** `feat(model): move, re-kind and remove library tracks and categories`.

---

### Task 4: Categories in the database (schema 2)

**Files:**
- Modify: `crates/fp-store/src/db/schema.rs`, `db/snapshot.rs`, `store.rs`, `import.rs` (only if it builds `RestoreParts`)
- Test: `crates/fp-store/tests/it/categories_store.rs` (new), `crates/fp-store/src/db/snapshot.rs` unit tests

**Interfaces:**
- Consumes: Task 1 (`Categories`, `repaired`), Task 2 (`merge_duplicate_paths` runs inside restore).
- Produces: `SCHEMA_VERSION = 2`; `Snapshot::categories: Vec<Category>`.

Migration 2 (SQL):

```sql
CREATE TABLE categories (id INTEGER PRIMARY KEY, position INTEGER NOT NULL, doc TEXT NOT NULL);
ALTER TABLE tracks ADD COLUMN category INTEGER;
CREATE INDEX tracks_by_category ON tracks(category);
CREATE INDEX tracks_by_path ON tracks(path);
```

- [x] **Step 1: Write the failing tests:**
  - `categories_and_track_categories_round_trip`: save a state with A ⊃ B (colour, kind) and a track in B; reload → same categories in order, track in B.
  - `a_schema_1_database_opens_with_every_track_uncategorised`: build a schema-1 file with the plan-1 SQL (copy migration 1 into the test) holding one track and one playlist; `Store::load` → the track is there, uncategorised; after a save `PRAGMA user_version` is 2.
  - `an_unknown_category_on_a_track_loads_uncategorised`: write a track row with `category = 999` → loads `None` with one warning.
  - `duplicate_paths_are_merged_on_reload` (LB14 on reload): two track rows on one path, an entry on each → one track after load, both entries on it; after a save the database has one row.
  - A snapshot unit test: `write_changes` writes only the changed category row (count rows changed via `conn.changes()` or a `total_changes()` difference).
- [x] **Step 2: Run** `cargo test -p fp-store --test it categories_store::` — expect failures.
- [x] **Step 3: Implement.** The `tracks.category` column wins over the doc field (as the other key columns); `upsert_track` writes it. Categories are read in `position` order, then `repaired()` (its warnings join the read's). `write_full` and `write_changes` handle `categories` with `sync_docs`. `Snapshot::of` and the restore pass the categories through.
- [x] **Step 4: Run** focused tests, fmt, clippy, the workspace suite.
- [x] **Step 5: Commit** `feat(store): store categories and each track's category (schema 2)`.

---

### Task 5: Canonical paths at the edges and the LB17c log line

**Files:**
- Modify: `crates/fp-app/src/ui/files.rs`, `ui/app.rs` (picks), `ui/settings/carts.rs` (cart file and page import), `ui/playlist_files.rs` (playlist import), `services.rs`
- Test: `crates/fp-app/tests/it/playlist_files.rs` or a new `crates/fp-app/tests/it/library_paths.rs`; `crates/fp-app/tests/it/services.rs`

**Interfaces:**
- Produces: `pub fn canonical_path(path: &Path) -> PathBuf` in `crates/fp-app/src/ui/files.rs` (`std::fs::canonicalize`, else the path as given).

- [x] **Step 1: Write the failing tests:**
  - `audio_paths_are_canonical`: a temp dir with `a.wav` and a symlink `link.wav → a.wav` (Unix only, `#[cfg(unix)]`) → `audio_paths(&[link])` returns the canonical `a.wav` path; a path given relative through `..` comes back without it.
  - `services_log_a_track_that_left_the_library` (LB17c): with the services test harness, an uncategorised unused track whose file is removed from the temp dir; after the file check, the track is gone from the model and the captured log has one line naming its path. A categorised track in the same situation stays.
- [x] **Step 2: Run** them — expect failures.
- [x] **Step 3: Implement.** `audio_paths` maps through `canonical_path` (it already runs on helper threads); the file picks, the cart file pick, the cart page import and the M3U/PLS import map their paths through it on their helper threads. In `services.rs`, where the library is compared between snapshots (skip when `shares_storage`), log `track left the library (file missing, uncategorised, unused)` with the path for each removed track whose previous version was `Missing` and uncategorised.
- [x] **Step 4: Run** focused tests, fmt, clippy, the workspace suite.
- [x] **Step 5: Commit** `feat(app): canonical paths for new tracks; log tracks that leave the library`.

---

### Task 6: Store hardening carried over from plan 1

**Files:**
- Modify: `crates/fp-store/src/db/backup.rs`, `db/mod.rs`, `db/snapshot.rs`, `import.rs`, `store.rs`
- Test: `crates/fp-store/tests/it/store_roundtrip.rs`, `crates/fp-store/tests/it/` import tests

- [x] **Step 1: Write the failing tests:**
  - `unusable_backups_are_set_aside_not_rotated_away`: a corrupt main file and corrupt `.bak1`/`.bak2` → after the start, each is renamed `.corrupt-<stamp>` (none deleted), and the next backup rotation does not delete them.
  - `a_busy_database_at_start_is_reported`: hold an exclusive lock on `fauste.db` from a second connection (`BEGIN EXCLUSIVE`) → `Store::load` gives an in-memory session and its `warnings` contain a line saying the database is in use (today only the log has it).
  - `a_too_new_json_file_blocks_the_import` and `an_unreadable_bak_blocks_the_import`: the import leaves every JSON file in place and no database file.
  - `set_aside_rolls_back_when_a_side_file_cannot_move` (the rollback path of `set_aside`; on Unix make the target directory read-only for the side file rename, `#[cfg(unix)]`).
- [x] **Step 2: Run** them — expect failures where behaviour is missing (the import and rollback tests may already pass: then they are coverage, say so in the report).
- [x] **Step 3: Implement.** Backups judged unusable are set aside with the same `.corrupt-<stamp>` naming as the main file. The busy start adds a load warning. The `*.imported` renames go through `retry_locked`. `id_text` truncates a text or blob to 64 characters. The player document is parsed once in `read_players`.
- [x] **Step 4: Run** focused tests, fmt, clippy, the workspace suite.
- [x] **Step 5: Commit** `fix(store): keep unusable backups, report a busy database, retry locked renames`.

---

### Task 7: Documentation

**Files:**
- Modify: `docs/technical/persistence.md` (schema 2, categories table, the path rule, LB17c), `docs/technical/architecture.md` (categories in `AppState`), `docs/user/data-and-backups.md` (tracks now stay in the library; schema 2 cannot be opened by older builds — the existing set-aside text covers it), `docs/user/playlists.md` (removing an entry no longer forgets its track; the same file is one track everywhere), the spec §5 status note if behaviour differs from it (the rulings above), and the plan's checkboxes.

- [x] **Step 1:** Update the files; check every statement against the code.
- [x] **Step 2:** `scripts/site/build.sh /tmp/site-check && scripts/site/check-links.sh /tmp/site-check` (if `gh` is unavailable, `FAUSTE_RELEASE_JSON` per CLAUDE.md, or skip the build and note it).
- [x] **Step 3: Commit** `docs: describe categories and tracks that stay in the library`.
