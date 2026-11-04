# Track library plan 5 — watched folders Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A category can watch a folder on the desktop: audio files in it
(and its subfolders) join the category, files that disappear are marked
missing, the folder is followed with file-system notifications while the
application runs and rescanned at start, and the operator can clean up with
*Remove missing tracks…*.

**Architecture:** The rules stay in `fp-model`: `Category::watched_folder`,
`Command::SetWatchedFolder` (LB19, exact duplicates refused),
`watching_category` (innermost wins), `missing_tracks` and
`Command::RemoveMissingTracks` (LB21). The file-system side lives in a new
`fp-app` module `watch/`: a `FolderWatcher` trait (the `notify`
implementation is desktop only; tests use a fake), a per-path trailing
`Debouncer` driven by `now: Instant`, and a `fp-folder-scan` helper thread
that walks folders and `stat`s paths. A `FolderWatch` owned by the services
thread joins them: it follows the categories' folders, turns debounced
events and scan answers into the existing `AddToLibrary` and `SetFileState`
commands, and publishes each folder's trouble in a `WatchCell` that the
library tree reads. New tracks reach the analysis through the services'
normal queue, behind the urgent one (LB23).

**Tech Stack:** Rust, `notify` 8.2 (new, desktop only), crossbeam-channel,
arc-swap, egui 0.37 / egui_kittest, tempfile.

**Spec:** `docs/superpowers/specs/2026-11-01-track-library-design.md` (§6
LB19–LB23; §7.3 the ⟳ mark, set/clear the watched folder, *Remove missing
tracks…*; §13 `library.watch_debounce_ms`; §14; §16 item 5; §17).

## Global Constraints

- English in every file; UI strings are Fluent messages in
  `crates/fp-app/locales/en-US/main.ftl`, with es-ES written and the other
  nine locales (ca-ES, de-DE, eu-ES, fr-FR, gl-ES, it-IT, nl-NL, pl-PL,
  pt-PT) translated in the same PR (`tests/it/i18n.rs` checks parity).
- Never mention other playout or radio-automation products. Describe only
  conventional folder watching: file-system notifications, a debounce, a
  rescan at start.
- `library.watch_debounce_ms`: default **1 000**, range **100–60 000**,
  lenient loading (rule 4).
- Rule 8: no file-system call on the UI thread (the folder picker runs on
  `fp-folder-dialog`, its result is canonicalised there). Scans never run on
  the services thread either: a share that is offline must not stall
  autosave (as `Probe` does for missing files).
- Rule 9: a missing or unreadable folder, a folder the system refuses to
  watch, an unreadable subfolder or a bad stored path degrade with a log
  line; nothing panics, no track is removed (LB22).
- Rule 10: nothing the watcher does loads or plays anything.
- `unwrap`/`expect`/`panic` denied outside tests; prefer `get`.
- Desktop only (§11): `notify` is a
  `[target.'cfg(not(target_os = "android"))'.dependencies]` entry of
  `fp-app`; the `notify` watcher module is `#[cfg(not(target_os =
  "android"))]`; the watch menu items are not offered on Android. Android
  must still build:
  `cargo ndk -P 29 -t arm64-v8a clippy --locked -p fp-mobile -p fp-app --lib -- -D warnings`
  and `scripts/check-mobile-isolation.sh` (CI runs both).
- LB12b: a services step with no folder event, no scan answer and unchanged
  categories does no work proportional to the library.
- Tests: services tests drive time with `now` and a fake watcher; the
  debounce is never waited for in wall time. New integration test files go
  into `crates/<crate>/tests/it/` with a `mod` line in `tests/it/main.rs`.
- Commit only with `cargo fmt --all --check`, `cargo clippy --workspace
  --all-targets -- -D warnings` and `cargo test --workspace` green; Task 3
  also runs `cargo deny check`.

## Review Focus

- **A watched folder on a drive that is not mounted at start:** no track
  under it is marked missing or removed, the category shows the trouble, the
  log says it once, and the folder is scanned as soon as it appears (Task 5
  test `a_missing_folder_is_reported_retried_and_scanned_when_it_appears`).
- **A folder with other files:** a video (`.mp4`), a text file, an audio
  file with an upper-case extension and an audio file two subfolders down:
  only the audio files join, all in the watching category, and no
  subcategory is created (Task 4 test `only_audio_files_join_and_subfolders_make_no_categories`).
- **The category deleted, or its folder cleared, while a scan is in
  flight:** the answer adds nothing to it, no error notice loops, the
  folder is no longer watched (Task 4 test
  `a_scan_answer_for_a_category_deleted_meanwhile_adds_nothing`).
- **A file still being copied:** a burst of events on one path adds it
  once, only after `watch_debounce_ms` of quiet in `now` (Task 3 unit test
  `a_path_is_due_only_after_a_quiet_period` and Task 5 test
  `a_burst_of_events_adds_the_file_once_after_the_debounce`).
- **A file renamed inside the folder:** the old track is marked missing
  (it keeps its markers; LB15), the new name joins as a new track (Task 5
  test `a_rename_marks_the_old_track_missing_and_adds_the_new_name`).

## Rulings made while writing this plan

- **LB19, nested folders.** Ruling: nested watched folders are allowed;
  only the *same* folder on two categories is refused
  (`ModelError::FolderAlreadyWatched`, with the category that holds it). A
  file under two watched folders goes to the category of the innermost one
  (`watching_category`, longest matching ancestor by path components).
  Folders compare as stored (canonicalised by the picker's helper thread,
  spec §5.1b). — Why: the spec's "innermost wins" only makes sense if
  nesting exists; a station often watches `Music/` and `Music/Gold/`
  separately. — Cost if wrong: one extra check in `SetWatchedFolder`.
- **Duplicates in stored data** (a hand-edited or old database): on restore
  `Categories::repaired` keeps the folder on the first category in stored
  order and clears it on the others, one warning each.
- **Setting, changing or clearing a folder never moves tracks.** Files that
  join follow LB20 (a track already in a category keeps it); tracks of a
  folder that is no longer watched stay in their category.
- **Retry interval.** A folder that is missing, unreadable or that the
  system refuses to watch is retried every `tuning.missing_recheck_ms`
  (existing, 30 s), the interval missing files are already looked for at:
  no new config field.
- **An unreadable or missing root marks nothing missing.** Only a scan of a
  readable root, or a debounced event on a path inside it, marks the
  tracks whose file is gone (LB21). A drive that is unmounted is one
  trouble on the category, not thousands of missing marks.
- **Directory symlinks are not followed** while scanning (no cycles); file
  symlinks are read and their canonical path joins (LB14). The category is
  decided from the path as found under the watched folder, not from its
  canonical target.
- **Lost events** (`notify`'s rescan flag, a watcher error) rescan every
  watched folder.
- **`library.watch_debounce_ms` is config-file only** (no Settings
  control), like the other tuning values; documented in the persistence
  reference under a new `library` section.
- **Remove missing tracks…** is offered on every category (not only
  watching ones) and covers the category and its subcategories, like the
  tree's counts. Its count is taken when the item is clicked (no walk of the
  library while the menu is open); with none, a notice says so instead of a
  question. The model recomputes the set when the command arrives, so a
  track that became used or came back in between is kept.
- **The ⟳ mark and the menu items show in the window and in Settings →
  Library** (same tree), on every desktop build, the narrow (phone) layout
  of a desktop window included: the gate is the Android build
  (`cfg!(target_os = "android")`), which has no file-system watching.
  *Remove missing tracks…* is offered everywhere the tree is.
- **`notify` is CC0-1.0**, a licence not yet in `deny.toml` and
  `about.toml`. CC0-1.0 is GPL-compatible; the plan adds it to both lists
  (Task 3). If the maintainer objects at review, the fallback is a
  `[[licenses.exceptions]]` entry for `notify` alone.

---

### Task 1: Watched folders in the model, the store and the config

**Files:**
- Modify: `crates/fp-model/src/category.rs` (`Category`, `Categories::repaired`, new `watching_category`)
- Modify: `crates/fp-model/src/command.rs`, `reducer.rs`, `error.rs`, `lib.rs` (re-export)
- Modify: `crates/fp-model/src/config.rs` (`LibraryConfig`, `Config::validate`)
- Modify: every `Category { .. }` literal (`reducer.rs:358`, `fp-store/src/db/snapshot.rs:936`, `fp-store/tests/it/categories_store.rs:198`, `fp-model/tests/it/categories.rs:167,197`, `fp-model/tests/it/library_search.rs:25`)
- Create test: `crates/fp-model/tests/it/watched_folders.rs` (+ `mod` line)
- Test: `crates/fp-model/tests/it/loudness_config.rs` (next to the `max_category_depth` test), `crates/fp-store/tests/it/categories_store.rs`

**Interfaces:**
- Produces:
  ```rust
  // Category gains, after `kind`:
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub watched_folder: Option<PathBuf>,
  // Command:
  /// LB19: the folder `category` watches (`None`: none). Refused when
  /// another category watches the same folder.
  SetWatchedFolder { category: CategoryId, folder: Option<PathBuf> },
  // ModelError:
  FolderAlreadyWatched(CategoryId), // the category that watches it
  // category.rs, re-exported from lib.rs:
  /// LB19: the category whose watched folder holds `path`, the innermost
  /// when several do; `None` when none does.
  pub fn watching_category(categories: &Categories, path: &Path) -> Option<CategoryId>;
  // config.rs:
  pub struct LibraryConfig { pub max_category_depth: usize, pub watch_debounce_ms: u64 }
  ```

- [ ] **Step 1: Write the failing model tests** in `watched_folders.rs`
  (one `AppState::default()` with categories made by `CreateCategory`):
  - `setting_a_watched_folder_stores_it_and_none_clears_it` (LB19):
    `SetWatchedFolder { a, Some("/m/ids") }` → `categories.get(a).watched_folder == Some("/m/ids")`; then `None` → `None`.
  - `the_same_folder_cannot_be_watched_twice` (LB19): `a` watches `/m`;
    `SetWatchedFolder { b, Some("/m") }` → `Err(ModelError::FolderAlreadyWatched(a))`, `b` unchanged; setting `/m` again on `a` is `Ok`.
  - `nested_folders_may_be_watched_and_the_innermost_wins` (LB19): `a`
    watches `/m`, `b` watches `/m/gold` → both `Ok`;
    `watching_category(&cats, "/m/gold/x/1.mp3") == Some(b)`,
    `"/m/new/2.mp3"` → `Some(a)`, `"/m/golden/3.mp3"` → `Some(a)` (components, not string prefix), `"/other/4.mp3"` → `None`.
  - `an_unknown_category_is_refused`: `Err(ModelError::UnknownCategory(_))`.
  - `repair_clears_a_folder_watched_twice`: `Categories::from(vec![a(/m), b(/m)]).repaired()` → `a` keeps `/m`, `b` has `None`, one warning naming `b`.
  - `deleting_a_category_drops_its_folder`: after `DeleteCategory(a)`, `watching_category(.., "/m/1.mp3") == None`.

- [ ] **Step 2: Config and store tests.** In `loudness_config.rs`:
  `watch_debounce_is_clamped_with_a_warning`: 50 → 100 and 99 000 → 60 000,
  each with a warning on field `"library.watch_debounce_ms"`; the default is
  1 000; a `library` section without the field loads 1 000. In
  `categories_store.rs`: `a_watched_folder_round_trips` (save with
  `/m/ids`, reload, equal) and `a_category_document_without_a_folder_loads_none`
  (a stored doc without the key, as plan 2 wrote it, loads `None`; no schema
  change: categories are JSON documents in `categories.doc`).

- [ ] **Step 3: Run them to see them fail**

  Run: `cargo test -p fp-model --test it watched_folders:: && cargo test -p fp-store --test it categories_store::`
  Expected: compile errors (`watched_folder`, `SetWatchedFolder`, `watching_category` missing).

- [ ] **Step 4: Implement.** The field and its serde attributes; the
  command in the reducer next to `SetCategoryKind` (unknown category →
  `UnknownCategory`; another category with an equal `watched_folder` →
  `FolderAlreadyWatched(that id)`); `watching_category` keeps the
  candidate with the most path components among those where
  `path.starts_with(folder)`; `repaired` gets one more pass after the
  existing ones; `watch_debounce_ms: 1_000` and a `clamp_to(.., 100,
  60_000, "library.watch_debounce_ms", ..)` after the depth clamp.
  `error_text` in `crates/fp-app/src/ui/app.rs` gets
  `FolderAlreadyWatched(_) => i18n.tr("error-folder-watched")`, with the
  message in all 11 locale files (en-US: `error-folder-watched = Another
  category already watches this folder.`; es-ES: `error-folder-watched =
  Otra categoría ya vigila esta carpeta.`).

- [ ] **Step 5: Run the tests** (Step 3 command plus `cargo test -p fp-app --test it i18n::`). Expected: PASS.

- [ ] **Step 6: Commit** `feat(model): a category can watch a folder`

### Task 2: Remove missing tracks in the model

**Files:**
- Modify: `crates/fp-model/src/library_rules.rs`, `command.rs`, `reducer.rs`, `lib.rs`
- Test: `crates/fp-model/tests/it/library_removal.rs`

**Interfaces:**
- Consumes: `Categories::subtree`, `state.playlists.references_track(id)` and `state.cartwall.references_track(id)` (as `release_tracks` uses them).
- Produces:
  ```rust
  /// LB21: the tracks of `category` and its subcategories whose file is
  /// missing and that no playlist entry or cart uses, in id order.
  pub fn missing_tracks(state: &AppState, category: CategoryId) -> Vec<TrackId>;
  // Command:
  /// LB21: removes `missing_tracks(category)` as found when it is applied.
  RemoveMissingTracks(CategoryId),
  ```

- [ ] **Step 1: Write the failing tests** in `library_removal.rs` (a state
  with category `a` and subcategory `b`, four tracks):
  - `missing_tracks_lists_only_unused_missing_tracks_of_the_subtree`: a
    missing unused track in `a` and one in `b` are listed; a missing track
    used by a playlist entry, a missing track used by a cart, an `Ok`
    track in `a` and a missing track of another category are not.
  - `remove_missing_tracks_removes_them_and_keeps_tracks_in_use`: after
    `RemoveMissingTracks(a)` the two listed are gone from the library, the
    used ones and their entries and carts are untouched, no
    `EngineAction::TrackLeftLibrary` is returned (an explicit removal).
  - `remove_missing_tracks_recomputes_when_applied`: list, then the file
    of one comes back (`SetFileState Ok`) before the command → it stays.
  - `remove_missing_tracks_of_an_unknown_category_is_refused`:
    `Err(UnknownCategory)`.

- [ ] **Step 2: Run them** — `cargo test -p fp-model --test it library_removal::` — Expected: FAIL (not defined).

- [ ] **Step 3: Implement** `missing_tracks` with `tracks_in(state,
  &subtree)` filtered by `FileState::Missing` and the two `references_track`
  checks; the command calls `remove_tracks` with that list (nothing used,
  so nothing on air and no entry goes).

- [ ] **Step 4: Run the tests** — Expected: PASS.

- [ ] **Step 5: Commit** `feat(model): remove a category's missing tracks`

### Task 3: The watch building blocks in fp-app

**Files:**
- Create: `crates/fp-app/src/watch/mod.rs` (trait, events, status cell), `watch/debounce.rs`, `watch/scan.rs`, `watch/notify_watcher.rs` (`#[cfg(not(target_os = "android"))]`)
- Modify: `crates/fp-app/src/lib.rs` (`pub mod watch;`), `crates/fp-app/Cargo.toml`, root `Cargo.toml` (`notify = "8.2"` in `[workspace.dependencies]`), `deny.toml`, `about.toml`
- Create test: `crates/fp-app/tests/it/watch_parts.rs` (+ `mod` line)

**Interfaces:**
- Produces:
  ```rust
  // watch/mod.rs
  pub enum FolderEvent {
      /// Something changed at this path: created, written, removed, or
      /// renamed from or to it.
      Touched(PathBuf),
      /// Events were lost: every watched folder is scanned again.
      Rescan,
  }
  pub trait FolderWatcher: Send {
      /// Follows `folder` and its subfolders; Err: why the system refused.
      fn watch(&mut self, folder: &Path) -> Result<(), String>;
      fn unwatch(&mut self, folder: &Path);
      /// The events since the last call; never blocks.
      fn events(&mut self) -> Vec<FolderEvent>;
  }
  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum FolderTrouble { Missing, Unreadable, NotFollowed }
  #[derive(Debug, Clone, Default, PartialEq)]
  pub struct WatchStatus { pub troubles: BTreeMap<CategoryId, FolderTrouble> }
  pub type WatchCell = Arc<ArcSwap<WatchStatus>>;
  pub fn idle_watch_cell() -> WatchCell;
  // watch/debounce.rs
  #[derive(Debug, Default)]
  pub struct Debouncer { /* path -> last event */ }
  impl Debouncer {
      pub fn note(&mut self, path: PathBuf, now: Instant);
      /// The paths with no event for `quiet`, sorted, removed from the set.
      pub fn take_due(&mut self, now: Instant, quiet: Duration) -> Vec<PathBuf>;
      pub fn is_empty(&self) -> bool;
  }
  // watch/scan.rs
  pub struct ScanJob {
      /// Folders to walk whole, with their subfolders.
      pub roots: Vec<PathBuf>,
      /// Paths an event touched: a file is looked at, a folder walked.
      pub paths: Vec<PathBuf>,
      /// Tracks to check: each one whose file is not there is `gone`.
      pub known: Vec<(TrackId, PathBuf)>,
  }
  pub struct Found { pub seen: PathBuf, pub canonical: PathBuf }
  #[derive(Default)]
  pub struct ScanAnswer {
      pub found: Vec<Found>,
      pub gone: Vec<TrackId>,
      /// Roots that could not be walked, and why.
      pub troubles: Vec<(PathBuf, FolderTrouble)>,
      /// Roots walked.
      pub readable: Vec<PathBuf>,
  }
  /// Runs one job (file-system work: helper thread only).
  pub fn scan(job: &ScanJob) -> ScanAnswer;
  /// The `fp-folder-scan` thread: one job at a time, as `Probe`.
  pub struct Scanner { /* jobs tx, answers rx, busy */ }
  impl Scanner {
      pub fn spawn() -> Option<Scanner>;
      pub fn busy(&self) -> bool;
      pub fn send(&mut self, job: ScanJob) -> bool;
      pub fn answers(&mut self) -> Vec<ScanAnswer>;
  }
  // watch/notify_watcher.rs (desktop)
  pub struct NotifyWatcher { /* notify::RecommendedWatcher + channel */ }
  impl NotifyWatcher { pub fn new() -> Result<NotifyWatcher, String>; }
  impl FolderWatcher for NotifyWatcher { .. }
  ```
  `AUDIO_EXTENSIONS` and `canonical_path` stay in `ui/files.rs`; `scan`
  uses `ui::files::is_audio` and `canonical_path`.

- [ ] **Step 1: Write the failing tests** in `watch_parts.rs`:
  - `a_path_is_due_only_after_a_quiet_period`: `note(p, t0)`,
    `note(p, t0 + 600 ms)`; `take_due(t0 + 1 400 ms, 1 s)` is empty;
    `take_due(t0 + 1 600 ms, 1 s) == [p]`; then `is_empty()`.
  - `paths_come_due_independently_and_sorted`: two paths noted 500 ms
    apart come due one after the other; two due at once come sorted.
  - `scan_finds_audio_files_in_subfolders_only`: a temp folder with
    `a.mp3`, `B.FLAC`, `sub/deeper/c.wav`, `film.mp4`, `notes.txt` →
    `found` seen paths are exactly the three audio files; `readable ==
    [root]`.
  - `scan_reports_a_missing_root_without_gone_marks`: root
    `tmp/absent`, `known` holds a track under it → `troubles == [(root,
    Missing)]`, `gone` empty.
  - `scan_marks_known_tracks_whose_file_is_gone`: `known` = one present,
    one deleted file → `gone == [deleted id]`.
  - `scan_walks_a_touched_folder_and_looks_at_a_touched_file`: `paths` =
    a new subfolder with two audio files and a single audio file → all
    three found.
  - `#[cfg(unix)] scan_does_not_follow_folder_symlinks`: a symlink to the
    root inside the root → each file found once, the walk ends.
  - `#[cfg(not(target_os = "android"))] notify_watcher_refuses_a_missing_folder`:
    `NotifyWatcher::new()?.watch(absent)` is `Err`, `watch(tempdir)` is
    `Ok`. (No event is waited for.)

- [ ] **Step 2: Run** `cargo test -p fp-app --test it watch_parts::` — Expected: FAIL (module missing).

- [ ] **Step 3: Add the dependency.** `notify = "8.2"` in the workspace
  table; in `fp-app`'s desktop-only dependency table `notify.workspace =
  true`. Add `"CC0-1.0"` to `deny.toml` `[licenses] allow` and
  `about.toml` `accepted` (ruling above). Run `cargo deny check`;
  Expected: `advisories ok, bans ok, licenses ok, sources ok`.

- [ ] **Step 4: Implement.** `Debouncer` over a `HashMap<PathBuf,
  Instant>`. `scan` walks with an explicit stack (no recursion), skips
  entries whose `file_type()` is a symlink to a folder, keeps `is_audio`
  files, and classifies a root: `NotFound` → `Missing`, any other error
  (or not a folder) → `Unreadable`; an unreadable subfolder is skipped with
  one `tracing::warn!` per scan. `gone` is every `known` whose path is not
  `is_file()`, except those under a root in `troubles`. `Scanner` mirrors
  `Probe` in `services.rs` (unbounded channels, thread `fp-folder-scan`,
  logs and returns `None` when the thread cannot start). `NotifyWatcher`
  uses `notify::recommended_watcher` with a closure sending into a
  crossbeam channel and `RecursiveMode::Recursive`; `events()` turns
  `event.need_rescan()` and every `Err` (logged) into `Rescan`, and each
  other event's paths into `Touched`.

- [ ] **Step 5: Run** the Step 2 command and `cargo clippy --workspace
  --all-targets -- -D warnings`. Expected: PASS, no warnings.

- [ ] **Step 6: Commit** `feat(app): folder watcher, debounce and scan thread`

### Task 4: The services follow the watched folders and rescan at start

**Files:**
- Create: `crates/fp-app/src/watch/folders.rs` (`FolderWatch`)
- Modify: `crates/fp-app/src/services.rs` (field, setter, call in `analysis_step`)
- Create: `crates/fp-app/tests/it/support/services_rig.rs` (moved), `crates/fp-app/tests/it/watched_folders.rs` (+ `mod` lines)
- Modify: `crates/fp-app/tests/it/services.rs` (uses the moved rig), `support/mod.rs` (`pub mod services_rig;`, `FakeWatcher`)

**Interfaces:**
- Consumes: Task 1 `watching_category`, `Category::watched_folder`, `LibraryConfig::watch_debounce_ms`; Task 3 everything.
- Produces:
  ```rust
  // watch/folders.rs
  pub(crate) struct WatchOutcome {
      pub commands: Vec<Command>,
      /// Missing tracks whose file is there again: to analyse at once.
      pub returned: Vec<TrackId>,
  }
  pub(crate) struct FolderWatch { .. }
  impl FolderWatch {
      pub(crate) fn new(watcher: Box<dyn FolderWatcher>, status: WatchCell) -> Self;
      /// One round on the services thread; never blocks.
      pub(crate) fn step(&mut self, state: &AppState, now: Instant) -> WatchOutcome;
      /// Commands the conductor could not queue: sent first next round.
      pub(crate) fn requeue(&mut self, unsent: Vec<Command>);
  }
  // services.rs
  impl Services {
      /// Watches the categories' folders (desktop). Without it nothing is watched.
      pub fn set_watcher(&mut self, watcher: Box<dyn FolderWatcher>);
      pub fn watch_status(&self) -> WatchCell;
      #[cfg(feature = "test-hooks")] pub fn folder_scans(&self) -> u64;
  }
  // tests/it/support/mod.rs
  #[derive(Clone, Default)]
  pub struct FakeWatcher(Arc<Mutex<FakeWatch>>); // watched, refused, queued events
  impl FakeWatcher {
      pub fn push(&self, event: FolderEvent);
      pub fn watched(&self) -> Vec<PathBuf>;
      pub fn refuse(&self, folder: PathBuf, refused: bool);
  }
  ```

- [ ] **Step 1: Move the services rig.** Move `wav`, `Rig`, `rig`,
  `rig_slow`, `rig_with`, `rig_hooked`, `rig_pool`, `rig_cached`, `wait`,
  `AfterAnalysis` and `Rig::run_until` from `tests/it/services.rs` into
  `support/services_rig.rs` unchanged except `pub` visibility. Run
  `cargo test -p fp-app --test it services::`; Expected: PASS, same count
  as before. Commit `test(app): share the services test rig`.

- [ ] **Step 2: Write the failing tests** in `watched_folders.rs`. Each
  builds a rig on a temp folder, creates categories through `prepare`
  (`CreateCategory`, `SetWatchedFolder`), calls
  `r.services.set_watcher(Box::new(fake.clone()))`, and loops with
  `r.run_until`, advancing `r.now` (never sleeping for the debounce):
  - `the_start_rescan_files_the_folder_into_its_category` (LB20, LB22):
    `ids/` holds `a.mp3`, `b.wav`; category *IDs* watches it; after the
    first steps both tracks are in the library with `category ==
    Some(ids)` and *IDs*' kind; `fake.watched() == [ids_folder]`.
  - `only_audio_files_join_and_subfolders_make_no_categories` (LB20,
    Review Focus): `a.MP3`, `x/y/b.flac`, `film.mp4`, `notes.txt` → two
    tracks, both in *IDs*; `categories.len()` unchanged.
  - `a_known_file_keeps_its_category_and_an_uncategorised_one_joins`
    (LB14, LB20): before the start, `c.mp3` is in the library in
    *Gold* and `d.mp3` uncategorised (via `AddToLibrary`) → after the
    rescan `c` stays in *Gold*, `d` is in *IDs*; still one track per file.
  - `the_innermost_watched_folder_wins` (LB19): *Music* watches `m/`,
    *Gold* watches `m/gold/`; `m/1.mp3` → *Music*, `m/gold/2.mp3` →
    *Gold*.
  - `a_scan_answer_for_a_category_deleted_meanwhile_adds_nothing`
    (Review Focus): one `r.services.step` sends the start scan; then
    `r.handle.send(DeleteCategory(ids))` and `r.conductor.tick` before the
    next services step takes the answer → no track ends up in a deleted
    category, no new uncategorised track, `fake.watched()` empty after the
    next step.
  - `clearing_a_folder_stops_watching_and_keeps_the_tracks`: after the
    rescan, `SetWatchedFolder { ids, None }` → `fake.watched()` empty;
    the tracks stay in *IDs*.
  - `an_idle_watch_does_not_walk_the_library` (LB12b): after the rescan
    settles, 50 steps without events → `r.services.library_walks()` and
    `r.services.folder_scans()` unchanged.

- [ ] **Step 3: Run** `cargo test -p fp-app --test it watched_folders::` — Expected: FAIL (`set_watcher` missing).

- [ ] **Step 4: Implement `FolderWatch::step`** in this order: (1) when
  `state.categories` differs from the last seen copy, rebuild `folder →
  category` from `watched_folder`, `unwatch` dropped folders (and forget
  their troubles), `watch` new ones and queue each new folder as a root to
  scan (the first round queues every folder: the start rescan); a
  `watch` error sets `NotFollowed` but the folder is still scanned. (2)
  Drain `watcher.events()` into the `Debouncer` (`Rescan` queues every
  folder as a root). (3) Take `scanner.answers()`: group `found` by
  `watching_category(&state.categories, &found.seen)` into one
  `AddToLibrary { paths: canonical, category: Some(c) }` per category
  (dropping files no current category watches); a `found` whose canonical
  path is a `Missing` track goes to `returned`; each `gone` track not
  already `Missing` becomes `SetFileState { track, state: Missing }`;
  `troubles` and `readable` update the status (log a line when a trouble
  starts and when it clears, not on every retry). (4) When the scanner is
  not busy and roots or due paths wait, send one `ScanJob` with `known` =
  the library's tracks not `Missing` whose path is under a root or equal
  to / under a due path (the only library walk, and only then). Publish
  the status cell when it changed. In `Services::analysis_step`, right
  after `recheck_missing`: run the step, send each command
  (`self.conductor.send`), `requeue` the ones refused, and for
  `returned` do what a probe's `found` does (`failed.remove`,
  `forced.insert`, `walk_again = true`).

- [ ] **Step 5: Run** the Step 3 command and the whole `services::` module. Expected: PASS.

- [ ] **Step 6: Commit** `feat(app): watched folders are scanned at start`

### Task 5: Events, missing files, troubles and analysis priority

**Files:**
- Modify: `crates/fp-app/src/watch/folders.rs`
- Test: `crates/fp-app/tests/it/watched_folders.rs`

**Interfaces:**
- Consumes: Task 4 `FolderWatch`, `FakeWatcher`, the rig's `gate` and `started`.
- Produces: no new public names; `FolderWatch` gains the retry timer
  (`tuning.missing_recheck_ms`) and uses `library.watch_debounce_ms`.

- [ ] **Step 1: Write the failing tests** (same file, same rig):
  - `a_new_file_joins_after_the_debounce` (LB20, LB22): after the start
    rescan, write `new.mp3`, `fake.push(Touched(new))`; at `now +
    debounce − 10 ms` it is not in the library; at `now + debounce` (and
    the scan answered) it is, in *IDs*.
  - `a_burst_of_events_adds_the_file_once_after_the_debounce` (Review
    Focus): five `Touched(new)` 200 ms apart (`watch_debounce_ms =
    1 000`) → not in the library before 1 000 ms after the last event;
    then exactly one track for it, and `r.services.folder_scans()` grew by
    exactly 1 for the whole burst.
  - `a_removed_file_marks_its_track_missing_and_its_return_restores_it`
    (LB21): delete `a.mp3`, push `Touched(a)` → after the debounce its
    track is `FileState::Missing` and still in *IDs*; write it back, push
    `Touched(a)` → `FileState::Ok` and analysed, same `TrackId`.
  - `a_rename_marks_the_old_track_missing_and_adds_the_new_name` (Review
    Focus): rename `a.mp3` → `z.mp3`, push both paths → `a`'s track
    `Missing` (its manual cue-in kept), a new track for `z` in *IDs*.
  - `a_removed_subfolder_marks_its_tracks_missing`: delete `x/` holding two
    tracks, push `Touched(x)` → both `Missing`.
  - `a_missing_folder_is_reported_retried_and_scanned_when_it_appears`
    (LB22, Review Focus): the folder does not exist at start and
    `fake.refuse(folder, true)`; a track under it in *IDs* → status
    `troubles[ids] == Missing`, the track not `Missing` and not removed;
    create the folder with `b.mp3`, `fake.refuse(folder, false)`, advance
    `now` by `missing_recheck_ms` → trouble cleared, `b` in *IDs*,
    `fake.watched()` holds the folder.
  - `a_folder_the_system_refuses_is_still_scanned`: `fake.refuse` on an
    existing folder → status `NotFollowed`, its files still join at start.
  - `lost_events_rescan_every_folder`: write a file without an event,
    push `Rescan` → after the scan the file is in the library.
  - `watched_folder_files_are_analysed_behind_a_players_track` (LB23):
    with `r.gate` closed, a player's next entry on an unanalysed track
    outside the folder and twelve new files in the folder joining in the
    same round; open the gate → `started_before(&r, player_file) <= 1`
    (the pattern of `a_track_on_a_player_is_analysed_ahead_of_the_library`;
    move `started_before` to the rig file).

- [ ] **Step 2: Run** `cargo test -p fp-app --test it watched_folders::` — Expected: the new tests FAIL.

- [ ] **Step 3: Implement** what Task 4's step left out: due paths from
  `Debouncer::take_due(now, Duration::from_millis(watch_debounce_ms))`
  join the next `ScanJob`'s `paths`; every `tuning.missing_recheck_ms`,
  folders with a trouble get `watch` again (`NotFollowed`, `Missing`) and
  are queued as roots; a successful `watch` after a refusal also queues a
  root scan (events may have been missed). No change to the analysis
  queue: new tracks are unanalysed and `submit_new` gives them
  `Analyzer::submit` unless a player or the screen wants them (LB23).

- [ ] **Step 4: Run** the Step 2 command, then `cargo test -p fp-app --test it services::`. Expected: PASS.

- [ ] **Step 5: Commit** `feat(app): follow watched folders while running`

### Task 6: The library window — ⟳ mark, watched folder menu, remove missing tracks

**Files:**
- Modify: `crates/fp-app/src/session.rs` (create the watcher, expose the cell), `crates/fp-app/src/ui/app.rs` (`with_watch_status`, `Scene::pick_watched_folder`, `PickTarget::WatchedFolder`, the `Picked` handler, `error_text` done in Task 1)
- Modify: `crates/fp-app/src/ui/library/tree.rs` (mark, menu, actions), `state.rs` (`LibraryDialog::RemoveMissing`), `confirm.rs` (the question)
- Modify: all 11 `crates/fp-app/locales/*/main.ftl`
- Modify: `crates/fp-app/tests/it/support/mod.rs` (`FakePlatform::folder: Option<PathBuf>`, returned by `pick_folder` when there is no browser)
- Create test: `crates/fp-app/tests/it/library_watch_ui.rs` (+ `mod` line)

**Interfaces:**
- Consumes: Task 1 `SetWatchedFolder`; Task 2 `missing_tracks`, `RemoveMissingTracks`; Task 3 `WatchCell`, `FolderTrouble`, `NotifyWatcher`; Task 4 `Services::set_watcher`, `watch_status`.
- Produces:
  ```rust
  // session.rs
  pub struct AppSession { .., pub watch: WatchCell }
  // app.rs
  impl AppUi { pub fn with_watch_status(self, cell: WatchCell) -> Self; }
  pub(crate) enum PickTarget { .., WatchedFolder(CategoryId) }
  impl Scene<'_> {
      /// The folder dialog on `fp-folder-dialog`, starting in the current
      /// folder or `ui.music_dir`; the choice is canonicalised there.
      pub(crate) fn pick_watched_folder(&self, category: CategoryId);
  }
  // state.rs
  LibraryDialog::RemoveMissing { category: CategoryId, count: usize },
  // tree.rs Action gains
  WatchFolder(CategoryId), StopWatching(CategoryId), RemoveMissing(CategoryId),
  ```
  New messages (en-US; es-ES in brackets):
  `library-menu-watch-folder = Watch folder…` [Vigilar carpeta…],
  `library-menu-change-watched-folder = Change watched folder…` [Cambiar carpeta vigilada…],
  `library-menu-stop-watching = Stop watching folder` [Dejar de vigilar la carpeta],
  `library-menu-remove-missing = Remove missing tracks…` [Quitar pistas que faltan…],
  `library-tree-watching = Watches { $folder }` [Vigila { $folder }],
  `library-tree-folder-missing = Watched folder not found: { $folder }. It is looked for again.` [No se encuentra la carpeta vigilada: { $folder }. Se vuelve a buscar.],
  `library-tree-folder-unreadable = Watched folder cannot be read: { $folder }. It is tried again.` [No se puede leer la carpeta vigilada: { $folder }. Se vuelve a intentar.],
  `library-tree-folder-not-followed = Changes in { $folder } are not followed; it is scanned at start and tried again.` [Los cambios en { $folder } no se siguen; se revisa al iniciar y se vuelve a intentar.],
  `library-remove-missing-title = Remove missing tracks` [Quitar pistas que faltan],
  `library-remove-missing-question = { $count -> [one] One track of { $name } has a missing file and is not used. *[other] { $count } tracks of { $name } have a missing file and are not used. } Remove them from the library? Tracks in use are kept; no file is touched.`
  [`{ $count -> [one] Una pista de { $name } no tiene su archivo y no se usa. *[other] { $count } pistas de { $name } no tienen su archivo y no se usan. } ¿Quitarlas de la biblioteca? Las pistas en uso se conservan; no se toca ningún archivo.`],
  `library-remove-missing-confirm = Remove` [Quitar],
  `library-remove-missing-none = { $name } has no unused tracks with a missing file.` [{ $name } no tiene pistas sin uso a las que les falte el archivo.].

- [ ] **Step 1: Write the failing UI tests** in `library_watch_ui.rs`
  (the `tree_state`/`open`/`menu` helpers of `library_tree.rs`, copied or
  moved to `support`):
  - `a_watching_category_shows_the_mark_with_its_folder`: *Rock* watches
    `/m/rock` → `h.get_by_label("Watches /m/rock")` exists; *Jingles*
    has none (`query_by_label` for any "Watches" label under it is None).
  - `a_folder_in_trouble_shows_why`: the cell holds `troubles[rock] =
    Missing` → the label `Watched folder not found: /m/rock. It is looked
    for again.` exists.
  - `watch_folder_picks_on_a_helper_thread_and_sends_the_command`:
    `FakePlatform { folder: Some(tmp) }`; menu on *Jingles* → click
    *Watch folder…* → after a few steps (the dialog thread answered),
    `SetWatchedFolder { category: jingles, folder: Some(canonical(tmp)) }`
    was sent and `platform.asked == ["folder"]`.
  - `a_cancelled_picker_sends_nothing`: `folder: None` → no command.
  - `stop_watching_sends_none`: menu on *Rock* shows *Change watched
    folder…* and *Stop watching folder*; the latter sends
    `SetWatchedFolder { rock, None }`; *Jingles*' menu shows neither, only
    *Watch folder…*.
  - `remove_missing_asks_with_the_count_then_sends`: *Rock* has two
    missing unused tracks and a missing used one → the question names 2;
    *Remove* sends `RemoveMissingTracks(rock)`; *Cancel* sends nothing.
  - `remove_missing_with_none_is_a_notice`: no missing track → no dialog,
    the notice text `Rock has no unused tracks with a missing file.`.
  - `settings_library_offers_the_same_watch_items`: Settings → Library,
    menu on *Rock* → *Stop watching folder* present.

- [ ] **Step 2: Run** `cargo test -p fp-app --test it library_watch_ui::` — Expected: FAIL.

- [ ] **Step 3: Implement.** Session: on desktop
  (`#[cfg(not(target_os = "android"))]`) `NotifyWatcher::new()` and
  `services.set_watcher(..)` before `spawn`; an `Err` is logged
  (`watched folders are not followed`) and the session starts anyway;
  `watch: services.watch_status()` in every build (idle on Android);
  `app_ui()` adds `.with_watch_status(self.watch.clone())`. Tree row: after
  the name, an `icon::ARROWS_CLOCKWISE` mark in `theme::NEUTRAL_400`
  (`theme::AMBER` with a trouble) with its own response, `widget_info`
  label = `library-tree-watching` or the trouble message, the same text as
  hover tooltip. Menu: before the separator, the watch items unless
  `cfg!(target_os = "android")`, then *Remove missing tracks…*. `Action::RemoveMissing`
  computes `missing_tracks` once: zero → `set_notice`, else
  `LibraryDialog::RemoveMissing`; the question is drawn in `confirm.rs`
  with the existing exit-guard buttons; *Remove* sends
  `RemoveMissingTracks`. `Picked` with `WatchedFolder(c)` and one path
  sends `SetWatchedFolder { category: c, folder: Some(path) }`. Add the
  messages to all 11 locale files (other nine translated).

- [ ] **Step 4: Run** Step 2's command, `library_tree::`, `library_menu::`,
  `settings::`, `i18n::`. Expected: PASS.

- [ ] **Step 5: Commit** `feat(ui): watch a folder from the library tree`

### Task 7: Documentation, spec notes and the Android check

**Files:**
- Modify: `README.md` (features: the library bullet gains watched folders)
- Modify: `docs/user/library.md` (new section *Watched folders* after *Categories*; the menu list in *Categories* gains the new items; *Remove missing tracks…* under *Removing tracks*), `docs/user/settings.md` (*Library* lists *watch a folder*), `docs/user/troubleshooting.md` (a watched folder on a network share or with too many subfolders: the trouble messages, the start rescan; Linux `fs.inotify.max_user_watches`)
- Modify: `docs/technical/persistence.md` (`watched_folder` in the category document; new `### library (config file only)` table with `max_category_depth` and `watch_debounce_ms`; the duplicate repair), `docs/technical/architecture.md` (the `watch` module in the `fp-app` row; desktop only), `docs/technical/threading-and-realtime.md` (helpers table: `fp-folder-scan`, and the `notify` watcher's own thread), `docs/technical/ui.md` (the tree's mark and menu; the folder dialog on `fp-folder-dialog`), `docs/technical/mobile.md` (no watched folders on Android, `notify` not built there)
- Modify: `docs/superpowers/specs/2026-11-01-track-library-design.md` (new `### 6.1 Implementation notes (plan 5)` with this plan's rulings, as §5.1b does for plan 2)

- [ ] **Step 1: Write the docs.** The user guide says, in its own terms:
  what joins (audio files, subfolders included, no subcategories), that a
  track already in another category stays there, that a vanished file is
  marked missing and comes back by itself, that the folder is followed
  while the application runs (after `library.watch_debounce_ms`, 1 s by
  default) and rescanned at every start, the ⟳ mark and its warning, that
  nested folders go to the innermost category, and that the same folder
  cannot be watched twice. English only (no `docs/i18n` change).

- [ ] **Step 2: Check the site and Android.** Run
  `scripts/site/build.sh /tmp/fp-site && scripts/site/check-links.sh /tmp/fp-site`
  (Expected: no dead link); `scripts/check-mobile-isolation.sh` (Expected:
  OK); with the NDK installed,
  `cargo ndk -P 29 -t arm64-v8a clippy --locked -p fp-mobile -p fp-app --lib -- -D warnings`
  (Expected: no warnings), and
  `cargo tree -p fp-app --target aarch64-linux-android -i notify`
  (Expected: an error that `notify` matches no package: it is not built
  for Android). Without the NDK, record it in the ledger for CI.

- [ ] **Step 3: Full verification.** `cargo fmt --all --check && cargo
  clippy --workspace --all-targets -- -D warnings && cargo test
  --workspace && cargo deny check`. Expected: all green.

- [ ] **Step 4: Commit** `docs: watched folders in the guide and the technical docs`
