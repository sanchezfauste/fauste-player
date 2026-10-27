# Mobile Plan 4 — Media Browser Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** On Android, wherever the desktop opens a file or folder dialog
(adding tracks to a playlist, a cart's file, the music folder), an in-app
browser lists the device's music grouped by folder, and the choice comes
back as real paths that the existing decoders open (MB7).

**Architecture:** `fp-app` gains `ui::media_browser`: a `MediaBrowser`
handle (the device's audio files and at most one pending request) and the
egui window that answers the request. The `Platform` trait does not
change: `AndroidPlatform`'s pickers call `MediaBrowser::browse` on the
helper thread they already run on, which waits until the operator
answers; `AppUi` draws the window while a request is pending. The file
list comes from `MediaStore`: the Kotlin shell queries it after the
permission is granted, on resume and when it changes, and passes the
paths through `Native.setMediaFiles`.

**Tech Stack:** Rust (egui 0.37, egui_kittest), Kotlin (`MediaStore`,
`ContentObserver`), `jni` 0.21.

**Spec:** `docs/superpowers/specs/2026-10-25-mobile-design.md` (MB7, §2
read-only music, §4).

## Global Constraints

- The UI never blocks (rule 8): `browse` is only called from helper
  threads; the window reads the handle with a short lock, never waiting.
- No new `Config` field. Every new string goes into all 11 locales.
- Paths stay `PathBuf`; nothing is copied into app storage; a listed file
  that has gone degrades as on the desktop (`audio_paths` drops it).
- The desktop is unchanged: it never builds a `MediaBrowser`.
- Tag writing, covers, imports and exports stay hidden on mobile (plan 2).

## Review Focus

- A second request while one is pending: the first one is answered
  "nothing chosen" and never hangs its thread.
- The activity is destroyed while a request is pending: the request stays
  and the next activity's interface shows it (the handle is process-wide).
- The file list changes while the window is open (a scan finishes): the
  selection keeps the paths still listed.
- No permission or no music: the window says so instead of an empty box.
- Thousands of files: folders start closed, so a frame draws one row per
  folder plus the open ones.

---

### Task 1: The `MediaBrowser` handle (pure)

**Files:**
- Create: `crates/fp-app/src/ui/media_browser.rs`
- Modify: `crates/fp-app/src/ui/mod.rs` or the `ui` module list (`pub mod media_browser;`)
- Test: `crates/fp-app/tests/it/media_browser.rs` (+ `mod` line)

**Interfaces:**
- Produces:
  - `pub enum BrowseMode { AudioFiles, OneFile { extensions: Vec<String> }, Folder }`
  - `#[derive(Clone, Default)] pub struct MediaBrowser`, with
    `set_files(&self, files: Vec<PathBuf>)`,
    `files(&self) -> Arc<Vec<PathBuf>>`,
    `browse(&self, mode: BrowseMode, start: Option<PathBuf>) -> Vec<PathBuf>` (blocks),
    `pending(&self) -> Option<(u64, BrowseMode, Option<PathBuf>)>` (the request id),
    `answer(&self, request: u64, chosen: Vec<PathBuf>)` (ignored for a stale id),
    `set_repaint(&self, ctx: egui::Context)` (asked to repaint when a request arrives).
  - `pub fn folders(files: &[PathBuf], mode: &BrowseMode) -> Vec<(PathBuf, Vec<PathBuf>)>`:
    the folders holding matching files, sorted case-insensitively, each with its files sorted the same way.

- [ ] **Step 1: Write the failing tests:**
  - `folders_group_and_sort_case_insensitively`
  - `one_file_mode_keeps_only_its_extensions`
  - `browse_returns_the_answer` (thread browses; test polls `pending`, answers)
  - `a_second_request_answers_the_first_with_nothing`
  - `a_stale_answer_is_ignored`
- [ ] **Step 2:** Run `cargo test -p fp-app --test it media_browser`. Expected: FAIL (module missing).
- [ ] **Step 3:** Implement with a `Mutex<Inner>` and a `Condvar` (or a channel per request).
- [ ] **Step 4:** Run the tests. Expected: PASS.
- [ ] **Step 5:** Commit `feat(ui): a media browser handle for platforms without file dialogs`.

### Task 2: The browser window

**Files:**
- Modify: `crates/fp-app/src/ui/media_browser.rs` (`pub fn show(ctx, browser, i18n, state)`)
- Modify: `crates/fp-app/src/ui/app.rs` (`AppUi::with_media_browser`, draw while pending)
- Modify: every `crates/fp-app/locales/*/main.ftl`
- Test: `crates/fp-app/tests/it/media_browser_ui.rs` (+ `mod` line)

**Interfaces:**
- Consumes: Task 1's handle.
- Produces: `AppUi::with_media_browser(self, MediaBrowser) -> Self`.

Strings (en-US / es-ES):
`media-browser-title` "Music on this device" / "Música del dispositivo";
`media-browser-files` "{ $count } files" / "{ $count } archivos" (plural forms);
`media-browser-add` "Add { $count }" / "Añadir { $count }";
`media-browser-use-folder` "Use this folder" / "Usar esta carpeta";
`media-browser-empty` "No music found. Copy audio files to the device's Music folder, and allow access to music." / "No se ha encontrado música. Copia archivos de audio a la carpeta Música del dispositivo y permite el acceso a la música.";
the existing cancel string for Cancel.

- [ ] **Step 1: Write the failing kittest tests** (a `MediaBrowser` with a fake list, a helper thread calling `browse`):
  - `audio_files_mode_adds_the_ticked_files`
  - `a_folder_tick_selects_its_files`
  - `one_file_mode_answers_on_the_first_tap`
  - `folder_mode_answers_the_folder`
  - `cancel_answers_nothing`
  - `an_empty_list_says_where_music_comes_from`
  - `no_window_without_a_request`
- [ ] **Step 2:** Run them. Expected: FAIL.
- [ ] **Step 3:** Implement: a modal window filling most of the screen; a scrolling list of folders (closed, with their file count; a tap opens one), files as checkable rows (AudioFiles), tappable rows (OneFile), a "Use this folder" button per folder (Folder); Cancel and Add at the bottom. Rows at least 48 px high.
- [ ] **Step 4:** Run the tests and the whole fp-app suite. Expected: PASS.
- [ ] **Step 5:** Commit `feat(ui): the media browser window`.

### Task 3: Android wiring

**Files:**
- Modify: `crates/fp-mobile/src/platform.rs` (pickers through the browser)
- Modify: `crates/fp-mobile/src/android.rs` (a process-wide `MediaBrowser`, `with_media_browser`)
- Modify: `crates/fp-mobile/src/jni.rs` (`Native.setMediaFiles(String[])`)
- Modify: `mobile/android/.../Native.kt`, `MainActivity.kt` (MediaStore query on a background thread after the permission, on resume, on `ContentObserver` change)

- [ ] **Step 1:** `AndroidPlatform`: `pick_audio_files` → `AudioFiles`; `pick_file` → `OneFile` with the given extensions (one path or none); `pick_folder` → `Folder`; `save_file` stays `None`.
- [ ] **Step 2:** Android clippy and `scripts/package/android.sh debug`. Expected: builds.
- [ ] **Step 3:** Commit `feat(mobile): pick music with the media browser`.

### Task 4: Docs and device check

- [ ] **Step 1:** `docs/technical/mobile.md`: a media access section; drop the interim picker line.
- [ ] **Step 2:** Emulator: Add opens the browser with `/sdcard/Music`; ticking two files adds them; a cart takes one file; Settings → music folder takes a folder.
- [ ] **Step 3:** Commit `docs: describe the Android media browser`.
