# Phone layout plan 4: the Music screen — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The phone's Music screen of the phone layout spec §9 (PL17–PL19b):
a fourth tab that browses the device's music by folder, artist or album,
selects tracks and adds them to a player's playlist, and that answers the
pickers in place of the media browser window.

**Architecture:** MediaStore's tags travel with each path as one string
record (`path\0title\0artist\0album\0duration_ms`), so the JNI call keeps
its signature and the parsing is pure Rust tested on the host. A pure
`music` module turns the file list and the screen's state into what to
show; the screen in `fp-app/src/ui/music.rs` draws it and sends the
existing `InsertPaths` command, or answers the `MediaBrowser` request.
Phones only: tablets keep the window.

**Tech Stack:** Rust, egui 0.37, egui_kittest, Kotlin (MediaStore).

**Spec:** `docs/superpowers/specs/2026-10-28-phone-layout-design.md`

## Global Constraints

- Phones only; tablets and the desktop keep the media browser window.
- Read-only: nothing writes to music files.
- No new player rules: adding is `Command::InsertPaths`.
- New strings in all 11 locales, literal text where English has no term.
- The UI never blocks: the list comes from `MediaBrowser::files()`.
- Ruling: the 🎧 pre-listen of PL18 is deferred — a file outside a
  playlist has no CUE source today (`CueStart` is an entry or a cart), so
  it needs model and engine work of its own.

## Review Focus

1. A file without tags (a bare path record, or empty fields): the row
   shows the file name, the Artists and Albums views group it under
   "Unknown artist" / "Unknown album".
2. *Add after the next* when the target player has no next, or its next is
   in another playlist: one button, *Add at the end*.
3. The selection keeps the order of the taps, and a file that left the
   list (a rescan) leaves the selection.
4. Back on the Music screen: a folder goes up to its parent first, then
   a picker's request is cancelled, then Music gives way to Live.
5. A request arriving while another screen shows: the Music screen opens
   in pick mode, and answering returns to the screen that was showing.

---

### Task 1: tags with the paths (PL18 data)

**Files:** `crates/fp-app/src/ui/media_browser.rs`,
`crates/fp-mobile/src/jni.rs`, `MediaFiles.kt`, tests in
`crates/fp-app/tests/it/media_browser.rs`.

**Interfaces:** Produces `pub struct MediaFile { pub path: PathBuf, pub
title: Option<String>, pub artist: Option<String>, pub album:
Option<String>, pub duration_secs: Option<f64> }` with `MediaFile::from_record(&str) -> Option<MediaFile>`
(empty fields → `None`, a record without `\0` is a bare path, an empty
path → `None`) and `MediaFile::title_or_name(&self) -> String`;
`MediaBrowser::set_files(Vec<MediaFile>)`, `files() -> Arc<Vec<MediaFile>>`.

- [ ] Tests: a full record; a bare path; empty fields; bad duration →
  `None`; `folders` still groups by parent. Watch them fail.
- [ ] Implement; Kotlin queries `TITLE, ARTIST, ALBUM, DURATION` with
  `DATA` and joins them with `\u0000` (MediaStore's `<unknown>` artist is
  dropped). Commit.

### Task 2: what the Music screen lists (PL17)

**Files:** create `crates/fp-app/src/ui/music.rs` (pure part), test
`crates/fp-app/tests/it/music.rs`.

**Interfaces:** `pub enum Browse { Folders, Artists, Albums }`;
`pub struct MusicView { pub browse: Browse, pub at: Option<String>, pub
folder: Option<PathBuf>, pub search: String, pub ticked: Vec<PathBuf> }`;
`pub fn listing(files: &[MediaFile], view: &MusicView) -> Listing` with
`Listing { groups: Vec<(String, Group)>, tracks: Vec<usize> }` (a group
is a sub-folder, an artist or an album; tracks index into `files`, sorted
by title without case); `MusicView::back(&mut self) -> bool` (folder to
parent, artist/album to the list; false at the top); `MusicView::toggle(path)`.

- [ ] Tests: Folders start at the deepest folder holding every file and
  list sub-folders then tracks; Artists and Albums list names, then the
  chosen one's tracks; a search lists matching tracks of every folder
  (title, artist, album, file name, any case); back steps up then reports
  false; toggle keeps tap order.
- [ ] Implement. Commit.

### Task 3: the Music tab (PL1, PL2, PL17, PL18)

**Files:** `touch.rs` (`PhoneScreen::Music`, a fourth chip, `back`),
`music.rs` (`screen`), `app.rs` (Back), tests
`crates/fp-app/tests/it/music_ui.rs`.

- [ ] Tests: the top bar has Music; the screen shows the search field, the
  Folders | Artists | Albums switch and a breadcrumb; a row shows title,
  artist and duration; a tap on a row ticks it; Back goes up a folder,
  then to Live.
- [ ] Implement; strings `touch-music`, `music-search`, `music-folders`,
  `music-artists`, `music-albums`, `music-unknown-artist`,
  `music-unknown-album`. Commit.

### Task 4: the selection bar (PL19)

**Files:** `music.rs`, tests in `music_ui.rs`.

- [ ] Tests: no bar without a selection; the bar shows the count and the
  target (the Playlists screen's player and its playlist); *Add at the
  end* sends `InsertPaths { index: len }` in tap order and clears the
  selection; *Add after the next* sends `after_next`'s index; with no
  next in the target only *Add at the end* shows; the target menu
  changes player and playlist.
- [ ] Implement; strings `music-selected`, `music-add-end`,
  `music-add-after-next`, `music-target`. Commit.

### Task 5: pick mode (PL19b)

**Files:** `music.rs`, `app.rs` (no modal on a phone), tests in
`music_ui.rs`.

- [ ] Tests: a request on a phone opens Music; one file: a tap answers it;
  a folder: *Use this folder* answers the folder shown; audio files: the
  bar's *Add* answers the ticked files in order; Back cancels; after an
  answer the earlier screen returns; a tablet still shows the window.
- [ ] Implement with the existing `media-browser-*` strings. Commit.

### Task 6: documentation and the emulator

- [ ] `docs/user/android.md`, `docs/technical/mobile.md`; release APK
  screenshots of Music (portrait, landscape, a selection). Commit.
