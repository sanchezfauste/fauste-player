# Track library plan 7 — phone follow-ups and device folders Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The maintainer's answers after trying plan 6: the preview strip
keeps its room, Device music gets a back button, a new list starts with an
empty search, and on Android a category can take its files from a device
folder that a button re-reads on demand.

**Architecture:** UI only, in `crates/fp-app/src/ui/phone_library.rs`,
`ui/library/tree.rs` and `ui/app.rs`. The device folder reuses
`Category::watched_folder` and `Command::SetWatchedFolder` (the model's
LB19 uniqueness rule applies unchanged); Android never starts a watcher, so
nothing follows it. Re-reading sends the existing `Command::AddToLibrary`
with the device files under the folder, taken from the `MediaBrowser`
list that MediaStore's observer keeps current.

**Tech Stack:** Rust, egui 0.37 / egui_kittest, Fluent.

**Spec:** `docs/superpowers/specs/2026-11-01-track-library-design.md`
§11.2 (LB49–LB53), with §11.1, LB19 and §5.1b's *Add to library* rule; the
phone layout spec PL2 and PL19b.

## Global Constraints

- English in every file; UI strings are Fluent messages in
  `crates/fp-app/locales/en-US/main.ftl`, es-ES written, the other nine
  locales translated in the same task (`tests/it/i18n.rs` checks parity).
- Never mention other playout or radio-automation products.
- The desktop is unchanged. A tablet changes only by LB52/LB53's menu items
  on Android.
- Rule 8: no file-system call on the UI thread. On Android the chosen
  folder is not canonicalised (§5.1b).
- `unwrap`/`expect`/`panic` denied outside tests; prefer `get`.

## Review Focus

- A device folder whose path is a prefix of a sibling (`/sd/Rock` and
  `/sd/Rocks`): re-reading `/sd/Rock` must not take `/sd/Rocks/a.mp3`
  (`Path::starts_with`, not string prefixes).
- Re-reading with no media permission (empty list): a notice, no command.
- A category whose folder another category already uses: the model refuses
  (LB19), the UI shows the error as it does for watched folders.
- The strip's idle room must not count as a tick target or shift the rows
  when a preview starts (row tops equal before and after).
- Device music's back during a pick: the header is absent, Cancel stays.

---

### Task 1: The strip's room is kept (LB49)

**Files:** `crates/fp-app/src/ui/phone_library.rs` (`preview_strip`),
locales, `crates/fp-app/tests/it/phone_library_ui.rs`.

- [ ] Test `lb49_the_rows_do_not_move_when_a_preview_starts`: on Rock's list,
  record the first row's top; set the library CUE playing; the top is equal;
  the idle hint `Tap 🎧 to pre-listen` is shown while stopped and gone
  while playing.
- [ ] `preview_strip` always allocates `touch::TARGET` of height; stopped, it
  paints the hint (`phone-library-strip-idle`) in `theme::NEUTRAL_500`.
- [ ] Commit `feat(ui): the phone's preview strip keeps its room`.

### Task 2: Device music's header (LB50)

**Files:** `phone_library.rs` (`screen`, `header`), locales, tests.

- [ ] Tests: `lb50_device_music_from_plus_has_a_back_button` (the header
  reads `Add to Rock`; its back returns to Rock's list);
  `lb50_device_music_from_the_selector_has_a_back_button` (reads
  `Device music`, back returns to the selector);
  `lb50_a_pick_shows_no_header` (pick mode: no back button, Cancel shown).
- [ ] `header` takes the title and whether the ＋ shows; `screen` draws it
  above `music::screen` on `LibraryPage::Device` when no pick is pending.
  Its back calls the same path as the system Back (`AppUi::music_back`
  semantics: up a folder first, then `PhoneLibraryView::back`).
- [ ] Commit `feat(ui): a back button on the phone's Device music`.

### Task 3: A new list starts with an empty search (LB51)

**Files:** `phone_library.rs` (`selector`), tests.

- [ ] Test `lb51_opening_another_category_clears_the_search`: search
  `cancion` in Rock, Back, open Jazz: the search field is empty and Jazz's
  rows show; *Whole library* keeps its state.
- [ ] Clear `view_state.library.search` when the selector opens a list.
- [ ] Commit `feat(ui): a phone list opens with an empty search`.

### Task 4: Device folders on Android (LB52, LB53)

**Files:** `ui/library/tree.rs` (menu, mark, actions), `ui/app.rs`
(`pick_watched_folder` without canonicalisation on Android),
`phone_library.rs` (header ↻, `reread_folder`), locales, tests.

- [ ] `pub(crate) fn folder_files(files: &[MediaFile], folder: &Path) ->
  Vec<PathBuf>`: the files under `folder` (`Path::starts_with`), in list
  order. Unit tests in the test binary through a `#[doc(hidden)] pub`
  re-export: the `/sd/Rock` vs `/sd/Rocks` case and an empty list.
- [ ] `pub(crate) fn reread_folder(scene, view_state, category)`: sends
  `AddToLibrary { paths, category: Some(category) }` when `paths` is not
  empty, and sets the notice `library-reread-done` (`$count`) or
  `library-reread-none`.
- [ ] The category menu: `cfg!(target_os = "android")` (a `device_folders`
  flag on the scene so tests can turn it on: `with_device_folders()` on
  the test builder) shows `library-menu-device-folder` /
  `library-menu-change-device-folder`, `library-menu-reread-device-folder`
  and `library-menu-forget-device-folder` instead of the watch items; the
  mark reads `library-tree-device-folder` with no trouble.
- [ ] The phone list header shows ↻ (`phone-library-reread-folder`) for a
  category with a folder when device folders are on.
- [ ] Tests: `lb52_the_menu_offers_a_device_folder`,
  `lb53_reread_adds_the_files_under_the_folder` (two files under
  `/sd/Rock`, one under `/sd/Rocks`: only the two are sent, with the
  category), `lb53_reread_with_no_files_says_so` (no command, notice).
- [ ] Commit `feat(ui): device folders for categories on Android`.

### Task 5: Docs, Play listing, emulator

- [ ] `docs/user/android.md`, `docs/user/library.md`, `docs/technical/ui.md`
  and `mobile.md`; the spec's notes as built.
- [ ] `packaging/android/play/<lang>/full_description.txt` in every
  language: the library (categories, search, pre-listen, device folders);
  `scripts/package/play-listing.py check` passes.
- [ ] The release APK on the emulator: strip room, back buttons, search
  cleared, device folder chosen and re-read.
