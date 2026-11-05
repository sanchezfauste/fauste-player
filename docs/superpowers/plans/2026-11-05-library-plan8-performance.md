# Track library plan 8 — performance carry-overs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Before the 0.13.0 release, remove the costs that grow with the
library or the selection which plans 1–7 left behind, and close two plan 7
follow-ups.

**Architecture:** No behaviour changes except Task 6's LB21 on a device
folder re-read (the spec already says it). Work in `fp-app`'s library UI
and folder watch, `fp-model`'s `category_counts`, and `fp-engine`'s
`ConductorHandle`.

**Tech Stack:** Rust, egui 0.37 / egui_kittest.

**Spec:** `docs/superpowers/specs/2026-11-01-track-library-design.md`
(§7.4 selection, LB21, LB42, LB53).

## Global Constraints

- English in every file; never mention other playout or radio-automation
  products.
- Rule 8: nothing on the UI thread touches the file system.
- `unwrap`/`expect`/`panic` denied outside tests; prefer `get`.
- Visible behaviour stays as the spec says; the existing tests stay green
  unchanged except where a field's type changes.

## Review Focus

- Ctrl-click removing a row from a large selection keeps the order of the
  rest (the menu and drags act in table order).
- Ticks on the phone are still dropped when the search hides them (LB42),
  now when the rows change instead of every frame.
- A re-read with an empty device list (no permission) marks nothing
  missing.
- Nested watched folders: a track under both a root and an event path is
  listed once in `known`.
- A command the conductor refuses is re-queued intact, not lost.

---

### Task 1: The library window's selection

**Files:** `crates/fp-app/src/ui/library/state.rs`, `table.rs`,
`preview.rs`; their tests.

- [ ] `pub(crate) struct Selection { order: Vec<TrackId>, set: HashSet<TrackId>, revision: u64 }`
  with `contains`, `ids() -> &[TrackId]`, `last`, `is_empty`, `len`,
  `clear`, `set_one(id)`, `push(id)` (no duplicate), `remove(id)`,
  `retain(f)`; every change bumps `revision`. `LibraryUi::selected` becomes
  a `Selection`. Shift-click becomes linear.
- [ ] `RowsCache` gets a `revision` (a counter bumped on every rebuild).
  `LibraryUi::selected_in_order` keeps a cache keyed by
  `(rows revision, selection revision)` (a `RefCell`), so an open row menu
  no longer walks every row each frame.
- [ ] Test: `a_shift_click_over_many_rows_is_linear` (20 000 rows, shift-click
  from the first to the last, then ctrl-click one away: order kept, < 1 s
  in debug).
- [ ] Commit `perf(ui): the library selection is a set in table order`.

### Task 2: The phone's ticks

**Files:** `crates/fp-app/src/ui/phone_library.rs`, the phone tab state.

- [ ] `PhoneLibraryView` stores the rows revision its ticks were checked
  against; the hidden ticks are dropped only when it changes.
  `LibraryUi::rows_revision() -> u64`.
- [ ] The LB42 tests stay green.
- [ ] Commit `perf(ui): the phone checks its ticks when the rows change`.

### Task 3: `category_counts` in one pass

**Files:** `crates/fp-model/src/library_search.rs`.

- [ ] Each category's own count is added to it and to each ancestor
  (walking `parent`, bounded by the number of categories against a broken
  tree), instead of a subtree walk per category.
- [ ] Test `category_counts_add_up_the_subtree` (three levels) stays and a
  deep chain of 200 categories counts right.
- [ ] Commit `perf(model): category counts in one pass`.

### Task 4: The watch's scan job

**Files:** `crates/fp-app/src/watch/folders.rs`.

- [ ] `known` and `holding`: a `HashSet` of the roots, the event paths
  (and the checks, for `holding`) and one pass over the library that looks
  up each track's ancestors (`Path::ancestors`), instead of library ×
  paths. Troubled folders likewise.
- [ ] The watch tests stay green.
- [ ] Commit `perf(app): the folder watch finds its tracks in one pass`.

### Task 5: Commands to the conductor are not cloned

**Files:** `crates/fp-engine/src/conductor.rs`, `crates/fp-app/src/services.rs`.

- [ ] `ConductorHandle::offer(&self, command: Command) -> Result<(), Command>`:
  the command comes back when the queue is full (logged like `send`).
  `send` calls it. `watch_step` re-queues what comes back, without a clone.
- [ ] Test: a full queue gives the command back.
- [ ] Commit `perf(engine): a refused command comes back to its sender`.

### Task 6: Plan 7 follow-ups

**Files:** `phone_library.rs` (`reread_folder`), the French locale, tests.

- [ ] LB21 on re-read: when the device lists files, a track under the
  folder that the list lacks is set missing (`SetFileState`), and a missing
  one the list has again is re-analysed (`ServiceRequest::ReanalyseTrack`),
  as the folder watch does. With an empty list nothing changes.
- [ ] Tests `lb53_reread_marks_the_gone_files_missing`,
  `lb53_reread_with_no_permission_marks_nothing`.
- [ ] fr-FR: *Médiathèque* → *Bibliothèque* (three messages).
- [ ] Spec LB53 note; ledger.
- [ ] Commit `fix(ui): a device folder re-read marks gone files missing`.

## Not in this plan (rulings)

- The first index build (~29 ms on the UI thread, once per window open)
  and `Snapshot::of` cloning the playlists per autosave: measured small;
  moving them needs a model change out of proportion before a release.
- The not-followed folder's retry walk: it is how such a folder sees
  changes (LB22); a backoff would need a config field and a spec change.
- `layer_id_at` at the screen edge: not reproduced on the emulator.
