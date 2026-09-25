# Phase 1 · Plan 1 — Foundation (workspace, domain model, persistence) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Stand up the Rust workspace with CI, and implement `fp-model` (every player behaviour rule as pure, tested logic) and `fp-store` (crash-safe persistence), so later plans only have to plug audio and UI onto a proven core.

**Architecture:** `fp-model` is a pure state machine with no I/O and no threads. `apply(state, Command)` and `on_event(state, EngineEvent)` mutate `AppState` and return the `EngineAction`s the future audio engine must perform. A `reconcile` pass derives preload and transition-schedule actions from the state, so the engine never needs to know player rules. `fp-store` serialises model documents as versioned JSON with atomic writes, rotating backups and corrupt-file quarantine.

**Tech Stack:** Rust 1.98.1 (edition 2024), serde 1.0.229, serde_json 1.0.151, thiserror 2.0.21, directories 6.0.0, proptest 1.11.0 (dev), tempfile 3.27.0 (dev), GitHub Actions, cargo-deny.

**Spec:** `docs/superpowers/specs/2026-09-25-fauste-player-design.md`. Read §2 (architecture), §2.4 (configuration and limits), §3 (player behaviour rules) and §7 (persistence) before starting.

**Position in Phase 1:** this is plan 1 of 4.

| Plan | Scope |
|---|---|
| **1** | foundation (this document) |
| 2 | `fp-engine` + `fp-backends` (Null, Offline, cpal) |
| 3 | `fp-analysis` |
| 4 | `fp-app` (egui UI) and wiring |

Plans 2–4 are written after this one lands, against the real interfaces produced here.

## Global Constraints

- All code, identifiers, comments, docs and commit messages in **English**.
- Never mention third-party radio-automation products anywhere (code, comments, docs, commit messages).
- Use established broadcast / pro-audio terminology from the spec glossary:
  - `Player` (not deck), `Source` (not voice), `segue_start`, `cue_in`, `cue_out`, `intro_end`, `outro_start`;
  - `Cartwall` / `CartPage` / `Cart`, `Bus::Main` / `Bus::Cue`.
- No hardcoded product limits. Anything tunable lives in `Config` (`players`, `analysis`, `outputs`, `ui`, `limits`, `tuning`) with a documented `Default` and clamped validation.
- Rust toolchain pinned to `1.98.1`, edition `2024`, workspace resolver `3`.
- `#![forbid(unsafe_code)]` in every crate of this plan (workspace lint `unsafe_code = "forbid"`).
- No `unwrap`/`expect`/`panic!` in non-test code (workspace clippy lints `unwrap_used`, `expect_used`, `panic` = deny). Tests may use them: unit tests via `clippy.toml`, integration test files via a file-level `#![allow(...)]`.
- `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` must pass before every commit.
- Nested `if` / `if let` that clippy's `collapsible_if` would flag are written as let-chains (`if a && let Some(x) = b { … }`), which edition 2024 supports.
- Commit messages end with the line `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.

## Review Focus

1. **Hand-edited config with nonsense values** (0 players, NaN, huge numbers). Expect: values are clamped with a warning; the app never refuses to start. Pinned in Task 4 (`nan_and_out_of_range_values_are_clamped`) and Task 10 (`invalid_values_are_clamped_on_load`).
2. **`session.json` referring to entries or playlists that no longer exist** (e.g. `playlists.json` was restored from a backup). Expect: dangling references are dropped, and players come up stopped with a sensible next. Pinned in Task 8 (`dangling_session_references_are_dropped`).
3. **A playlist where every file is missing or unreadable.** Expect: Play does nothing, nothing loops forever, no panic. Pinned in Task 6 (`playlist_of_unreadable_entries_never_starts`).
4. **Crash in the middle of a save** (truncated primary file, stale `.tmp` left behind). Expect: the last good backup loads and the broken file is quarantined, not deleted. Pinned in Task 9 (`truncated_primary_falls_back_to_backup_and_is_quarantined`).
5. **A state file written by a newer version of the app** (downgrade). Expect: a warning and defaults, with the newer file **not** quarantined, so it survives as `.bak1` on the next save. Pinned in Task 10 (`newer_schema_is_not_quarantined`).

---

## File Structure

```
Cargo.toml                          workspace manifest, shared deps, lints, profiles
rust-toolchain.toml                 pins 1.98.1 + rustfmt + clippy
clippy.toml                         allow unwrap/expect/panic in #[cfg(test)]
deny.toml                           cargo-deny policy
.gitignore
.github/workflows/ci.yml            fmt, clippy, test on Linux/Windows/macOS + cargo-deny
crates/fp-model/
  Cargo.toml
  src/lib.rs                        module list + re-exports
  src/ids.rs                        PlayerId, PlaylistId, EntryId, TrackId, IdGen
  src/error.rs                      ModelError
  src/track.rs                      Track, TrackKind, FileState, Markers, Library
  src/playlist.rs                   PlaylistEntry, Playlist, Playlists (+ edit operations)
  src/config.rs                     Config and its groups, validation with clamping
  src/player.rs                     PlayerState, Transport, PlayMode, CueState, ColumnWidths
  src/command.rs                    Command, EngineEvent, EngineAction, SourceRequest, TransitionPlan
  src/state.rs                      AppState + lookups
  src/reducer.rs                    apply(), on_event(), reconcile(), plan_for()
  src/session.rs                    PlayerSession, RestoreParts, AppState::sessions/restore
  tests/common/mod.rs               shared fixtures for integration tests
  tests/playlist_props.rs           proptest: playlist edits vs reference model
  tests/transport.rs                rules 2–8
  tests/transitions.rs              rules 9–12, source failures
  tests/editing.rs                  rules 13–15, 21, playlists, volume, seek, config
  tests/session.rs                  crash-recovery restore
crates/fp-store/
  Cargo.toml
  src/lib.rs
  src/error.rs                      StoreError
  src/paths.rs                      AppPaths (OS dirs or a test root)
  src/atomic.rs                     write_atomic, load_with_fallback, backups, quarantine
  src/migrate.rs                    schema upgrade driver
  src/docs.rs                       ConfigDoc, PlaylistsDoc, SessionDoc + schema constants
  src/store.rs                      Store facade: load() / save_*()
  tests/store_roundtrip.rs
```

---

### Task 1: Toolchain, workspace, CI and typed ids

**Files:**
- Create: `Cargo.toml`, `rust-toolchain.toml`, `clippy.toml`, `deny.toml`, `.gitignore`, `.github/workflows/ci.yml`
- Create: `crates/fp-model/Cargo.toml`, `crates/fp-model/src/lib.rs`, `crates/fp-model/src/ids.rs`

**Interfaces:**
- Produces:
  - `fp_model::{PlayerId, PlaylistId, EntryId, TrackId}`: `#[serde(transparent)]` newtypes over `u64` (`pub .0`), `Copy + Eq + Ord + Hash`.
  - `fp_model::IdGen` with `player()`, `playlist()`, `entry()`, `track()`, `next_raw() -> u64` and `observe(raw: u64)`.

- [ ] **Step 1: Install the Rust toolchain manager** (the machine has no Rust yet)

Run: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none && . "$HOME/.cargo/env"`
Expected: `rustup --version` prints a version.

- [ ] **Step 2: Create the workspace files**

`rust-toolchain.toml`:
```toml
[toolchain]
channel = "1.98.1"
components = ["rustfmt", "clippy"]
```

`Cargo.toml`:
```toml
[workspace]
resolver = "3"
members = ["crates/*"]

[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.98"
publish = false

[workspace.dependencies]
fp-model = { path = "crates/fp-model" }
serde = { version = "1.0.229", features = ["derive"] }
serde_json = "1.0.151"
thiserror = "2.0.21"
directories = "6.0.0"
proptest = "1.11.0"
tempfile = "3.27.0"

[workspace.lints.rust]
unsafe_code = "forbid"

[workspace.lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"

# Panics must unwind so that panic isolation (spec §8.5, §4.5) works.
[profile.dev]
panic = "unwind"

[profile.release]
panic = "unwind"
```

`clippy.toml`:
```toml
allow-unwrap-in-tests = true
allow-expect-in-tests = true
allow-panic-in-tests = true
```

`.gitignore`:
```
/target
```

`deny.toml`:
```toml
[graph]
all-features = true

[advisories]
version = 2
yanked = "deny"

[licenses]
version = 2
allow = [
  "MIT", "Apache-2.0", "Apache-2.0 WITH LLVM-exception", "BSD-2-Clause", "BSD-3-Clause",
  "ISC", "Zlib", "MPL-2.0", "Unicode-3.0", "Unicode-DFS-2016", "OFL-1.1",
]
confidence-threshold = 0.9

[licenses.private]
ignore = true

[bans]
multiple-versions = "warn"

[sources]
unknown-registry = "deny"
unknown-git = "deny"
```

`.github/workflows/ci.yml`:
```yaml
name: CI
on:
  push:
    branches: [master, main]
  pull_request:

jobs:
  test:
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - name: Install pinned toolchain
        run: rustup show active-toolchain || rustup toolchain install
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --all --check
      - run: cargo clippy --workspace --all-targets -- -D warnings
      - run: cargo test --workspace

  deny:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: EmbarkStudios/cargo-deny-action@v2
```

`crates/fp-model/Cargo.toml`:
```toml
[package]
name = "fp-model"
description = "Domain model and player behaviour rules (pure logic, no I/O)"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
publish.workspace = true

[dependencies]
serde.workspace = true
thiserror.workspace = true

[dev-dependencies]
proptest.workspace = true
serde_json.workspace = true

[lints]
workspace = true
```

- [ ] **Step 3: Write the failing tests for ids**

`crates/fp-model/src/lib.rs`:
```rust
//! Domain model of Fauste Player: tracks, playlists, players, configuration
//! and the pure state machine that implements the player behaviour rules.
//! This crate performs no I/O and spawns no threads.

pub mod ids;

pub use ids::{EntryId, IdGen, PlayerId, PlaylistId, TrackId};
```

`crates/fp-model/src/ids.rs` (tests first; the types below them are added in Step 5):
```rust
//! Opaque identifiers. Nothing in the model indexes a fixed-size array by
//! player or playlist number; everything is looked up by id.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_and_increasing_across_kinds() {
        let mut ids = IdGen::default();
        let a = ids.entry();
        let b = ids.track();
        let c = ids.player();
        assert!(a.0 < b.0 && b.0 < c.0);
    }

    #[test]
    fn observe_moves_the_generator_past_existing_ids() {
        let mut ids = IdGen::default();
        ids.observe(41);
        ids.observe(7);
        assert_eq!(ids.player(), PlayerId(42));
    }

    #[test]
    fn ids_serialize_as_plain_numbers() {
        assert_eq!(serde_json::to_string(&EntryId(7)).unwrap(), "7");
        let back: EntryId = serde_json::from_str("7").unwrap();
        assert_eq!(back, EntryId(7));
    }
}
```

- [ ] **Step 4: Run the tests to verify they fail**

Run: `cargo test -p fp-model`
Expected: FAIL to compile: `cannot find type IdGen`, `cannot find type PlayerId`, …

- [ ] **Step 5: Implement the ids**

Insert above the `#[cfg(test)]` block in `crates/fp-model/src/ids.rs`:
```rust
use serde::{Deserialize, Serialize};

macro_rules! id_type {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub u64);
    };
}

id_type!(
    /// Identifies a player (one playout column with its own transport).
    PlayerId
);
id_type!(
    /// Identifies a playlist.
    PlaylistId
);
id_type!(
    /// Identifies one entry of a playlist. The same track can appear in
    /// several entries, and "played" is tracked per entry.
    EntryId
);
id_type!(
    /// Identifies a track (one audio file and its metadata).
    TrackId
);

/// Monotonic id generator shared by every id kind, so ids never collide
/// even across kinds. It is persisted with the playlists.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdGen {
    last: u64,
}

impl IdGen {
    /// Returns a fresh raw id.
    pub fn next_raw(&mut self) -> u64 {
        self.last += 1;
        self.last
    }

    /// Makes sure future ids are greater than `raw` (used after loading data).
    pub fn observe(&mut self, raw: u64) {
        self.last = self.last.max(raw);
    }

    pub fn player(&mut self) -> PlayerId {
        PlayerId(self.next_raw())
    }

    pub fn playlist(&mut self) -> PlaylistId {
        PlaylistId(self.next_raw())
    }

    pub fn entry(&mut self) -> EntryId {
        EntryId(self.next_raw())
    }

    pub fn track(&mut self) -> TrackId {
        TrackId(self.next_raw())
    }
}
```

- [ ] **Step 6: Run the full check suite**

Run: `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
Expected: 3 tests PASS, no clippy warnings.

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml Cargo.lock rust-toolchain.toml clippy.toml deny.toml .gitignore .github crates/fp-model
git commit -m "build: set up Rust workspace, CI and fp-model ids

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: Tracks, markers and the library

**Files:**
- Create: `crates/fp-model/src/track.rs`
- Modify: `crates/fp-model/src/lib.rs`

**Interfaces:**
- Consumes: `TrackId` (Task 1).
- Produces:
  - `TrackKind { Music (default), Jingle, Effect, Ad, Voice }`.
  - `FileState { Ok (default), Missing, Unreadable }` with `is_playable(self) -> bool`.
  - `MarkerSource { Auto, Manual }`; `Marker { secs: f64, source: MarkerSource }`.
  - `MarkerKind { CueIn, IntroEnd, OutroStart, SegueStart, CueOut }`.
  - `Markers` with pub `Option<Marker>` fields `cue_in`, `intro_end`, `outro_start`, `segue_start`, `cue_out`, and methods `get(kind)`, `set_auto(kind, Option<f64>)`, `set_manual(kind, Option<f64>)`.
  - `Track` with pub fields `id`, `path: PathBuf`, `title`, `artist`, `album: String`, `duration_secs: f64`, `kind`, `file_state`, `markers`, `analyzed: bool`. Methods: `Track::new(id, path)`, `cue_in_secs()`, `cue_out_secs()`, `segue_start_secs()`, `intro_end_secs()`, `outro_start_secs()`, `play_length_secs()`.
  - `Library` with `insert(Track)`, `get(TrackId) -> Option<&Track>`, `get_mut`, `remove(TrackId) -> Option<Track>`, `iter()`, `iter_mut()`, `len()`, `is_empty()`, `is_playable(TrackId) -> bool` and `max_raw_id() -> u64`. It serialises as a JSON array of tracks.

- [ ] **Step 1: Write the failing tests**

Create `crates/fp-model/src/track.rs` with only the test module:
```rust
//! Tracks (audio files with metadata and cue markers) and the library that owns them.

#[cfg(test)]
mod tests {
    use super::*;

    fn track() -> Track {
        let mut t = Track::new(TrackId(1), PathBuf::from("/music/Artist - Song.flac"));
        t.duration_secs = 200.0;
        t
    }

    #[test]
    fn new_track_takes_its_title_from_the_file_stem() {
        assert_eq!(track().title, "Artist - Song");
    }

    #[test]
    fn cue_points_default_to_the_whole_file() {
        let t = track();
        assert_eq!(t.cue_in_secs(), 0.0);
        assert_eq!(t.cue_out_secs(), 200.0);
        assert_eq!(t.segue_start_secs(), None);
        assert_eq!(t.play_length_secs(), 200.0);
    }

    #[test]
    fn auto_markers_never_overwrite_manual_ones() {
        let mut t = track();
        t.markers.set_manual(MarkerKind::SegueStart, Some(190.0));
        t.markers.set_auto(MarkerKind::SegueStart, Some(185.0));
        assert_eq!(t.segue_start_secs(), Some(190.0));
        t.markers.set_auto(MarkerKind::CueOut, Some(198.0));
        assert_eq!(t.cue_out_secs(), 198.0);
        assert_eq!(t.markers.get(MarkerKind::CueOut).map(|m| m.source), Some(MarkerSource::Auto));
    }

    #[test]
    fn manual_marker_can_be_cleared() {
        let mut t = track();
        t.markers.set_manual(MarkerKind::IntroEnd, Some(12.0));
        t.markers.set_manual(MarkerKind::IntroEnd, None);
        assert_eq!(t.intro_end_secs(), None);
    }

    #[test]
    fn only_ok_files_are_playable() {
        let mut lib = Library::default();
        let mut t = track();
        lib.insert(t.clone());
        assert!(lib.is_playable(TrackId(1)));
        t.file_state = FileState::Missing;
        lib.insert(t);
        assert!(!lib.is_playable(TrackId(1)));
        assert!(!lib.is_playable(TrackId(99)));
    }

    #[test]
    fn library_round_trips_through_json_as_an_array() {
        let mut lib = Library::default();
        lib.insert(track());
        let json = serde_json::to_string(&lib).unwrap();
        assert!(json.starts_with('['));
        let back: Library = serde_json::from_str(&json).unwrap();
        assert_eq!(back, lib);
        assert_eq!(back.max_raw_id(), 1);
    }
}
```

Add to `crates/fp-model/src/lib.rs`:
```rust
pub mod track;

pub use track::{FileState, Library, Marker, MarkerKind, MarkerSource, Markers, Track, TrackKind};
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-model track`
Expected: FAIL to compile: `cannot find type Track`.

- [ ] **Step 3: Implement**

Insert above the test module in `track.rs`:
```rust
use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::ids::TrackId;

/// What a track is used for on air.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TrackKind {
    #[default]
    Music,
    Jingle,
    Effect,
    Ad,
    Voice,
}

/// Whether the file behind a track can currently be played.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FileState {
    #[default]
    Ok,
    Missing,
    Unreadable,
}

impl FileState {
    pub fn is_playable(self) -> bool {
        matches!(self, FileState::Ok)
    }
}

/// Whether a marker was detected by analysis or placed by the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarkerSource {
    Auto,
    Manual,
}

/// A cue point, in seconds from the start of the file.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Marker {
    pub secs: f64,
    pub source: MarkerSource,
}

/// The standard broadcast cue points of a track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarkerKind {
    CueIn,
    IntroEnd,
    OutroStart,
    SegueStart,
    CueOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Markers {
    pub cue_in: Option<Marker>,
    pub intro_end: Option<Marker>,
    pub outro_start: Option<Marker>,
    pub segue_start: Option<Marker>,
    pub cue_out: Option<Marker>,
}

impl Markers {
    pub fn get(&self, kind: MarkerKind) -> Option<Marker> {
        match kind {
            MarkerKind::CueIn => self.cue_in,
            MarkerKind::IntroEnd => self.intro_end,
            MarkerKind::OutroStart => self.outro_start,
            MarkerKind::SegueStart => self.segue_start,
            MarkerKind::CueOut => self.cue_out,
        }
    }

    fn slot_mut(&mut self, kind: MarkerKind) -> &mut Option<Marker> {
        match kind {
            MarkerKind::CueIn => &mut self.cue_in,
            MarkerKind::IntroEnd => &mut self.intro_end,
            MarkerKind::OutroStart => &mut self.outro_start,
            MarkerKind::SegueStart => &mut self.segue_start,
            MarkerKind::CueOut => &mut self.cue_out,
        }
    }

    /// Stores an analysis result unless the user placed this marker manually.
    pub fn set_auto(&mut self, kind: MarkerKind, secs: Option<f64>) {
        let slot = self.slot_mut(kind);
        if matches!(slot, Some(Marker { source: MarkerSource::Manual, .. })) {
            return;
        }
        *slot = secs.map(|secs| Marker { secs, source: MarkerSource::Auto });
    }

    /// Places (or clears, with `None`) a user marker.
    pub fn set_manual(&mut self, kind: MarkerKind, secs: Option<f64>) {
        *self.slot_mut(kind) = secs.map(|secs| Marker { secs, source: MarkerSource::Manual });
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Track {
    pub id: TrackId,
    pub path: PathBuf,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_secs: f64,
    pub kind: TrackKind,
    pub file_state: FileState,
    pub markers: Markers,
    /// True once analysis (tags, peaks, automatic markers) has completed.
    pub analyzed: bool,
}

impl Track {
    /// A not-yet-analysed track; the title is the file stem until tags are read.
    pub fn new(id: TrackId, path: PathBuf) -> Self {
        let title = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        Self {
            id,
            path,
            title,
            artist: String::new(),
            album: String::new(),
            duration_secs: 0.0,
            kind: TrackKind::default(),
            file_state: FileState::default(),
            markers: Markers::default(),
            analyzed: false,
        }
    }

    pub fn cue_in_secs(&self) -> f64 {
        self.markers.cue_in.map_or(0.0, |m| m.secs)
    }

    pub fn cue_out_secs(&self) -> f64 {
        self.markers.cue_out.map_or(self.duration_secs, |m| m.secs)
    }

    pub fn segue_start_secs(&self) -> Option<f64> {
        self.markers.segue_start.map(|m| m.secs)
    }

    pub fn intro_end_secs(&self) -> Option<f64> {
        self.markers.intro_end.map(|m| m.secs)
    }

    pub fn outro_start_secs(&self) -> Option<f64> {
        self.markers.outro_start.map(|m| m.secs)
    }

    /// Audible length between cue-in and cue-out.
    pub fn play_length_secs(&self) -> f64 {
        (self.cue_out_secs() - self.cue_in_secs()).max(0.0)
    }
}

/// All known tracks, keyed by id. Serialised as a JSON array.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(from = "Vec<Track>", into = "Vec<Track>")]
pub struct Library {
    tracks: BTreeMap<TrackId, Track>,
}

impl From<Vec<Track>> for Library {
    fn from(tracks: Vec<Track>) -> Self {
        Self { tracks: tracks.into_iter().map(|t| (t.id, t)).collect() }
    }
}

impl From<Library> for Vec<Track> {
    fn from(library: Library) -> Self {
        library.tracks.into_values().collect()
    }
}

impl Library {
    pub fn insert(&mut self, track: Track) {
        self.tracks.insert(track.id, track);
    }

    pub fn get(&self, id: TrackId) -> Option<&Track> {
        self.tracks.get(&id)
    }

    pub fn get_mut(&mut self, id: TrackId) -> Option<&mut Track> {
        self.tracks.get_mut(&id)
    }

    pub fn remove(&mut self, id: TrackId) -> Option<Track> {
        self.tracks.remove(&id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Track> {
        self.tracks.values()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Track> {
        self.tracks.values_mut()
    }

    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    pub fn is_playable(&self, id: TrackId) -> bool {
        self.get(id).is_some_and(|t| t.file_state.is_playable())
    }

    pub fn max_raw_id(&self) -> u64 {
        self.tracks.keys().map(|id| id.0).max().unwrap_or(0)
    }
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-model track`
Expected: 6 tests PASS.

- [ ] **Step 5: Check and commit**

Run: `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
```bash
git add crates/fp-model
git commit -m "feat(model): add tracks, cue markers and library

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: Model errors and playlists

**Files:**
- Create: `crates/fp-model/src/error.rs`, `crates/fp-model/src/playlist.rs`, `crates/fp-model/tests/playlist_props.rs`
- Modify: `crates/fp-model/src/lib.rs`

**Interfaces:**
- Consumes: ids (Task 1), `Library` (Task 2).
- Produces:
  - `ModelError` (thiserror) with the variants:
    - `UnknownPlayer(PlayerId)`, `UnknownPlaylist(PlaylistId)`, `UnknownEntry(EntryId)`
    - `NoPlaylists`, `LastPlaylist`, `PlaylistOnAir(PlaylistId)`, `EntryOnAir(EntryId)`
    - `NextIsCurrent`, `StopAfterInSingle`
    - `PlayerCountOutOfRange { requested: usize, max: usize }`, `PlayerBusy(PlayerId)`
  - `PlaylistEntry { id: EntryId, track: TrackId, played: bool }` (`Copy`).
  - `Playlist { id, name: String, entries: Vec<PlaylistEntry> }` with `new(id, name: impl Into<String>)` and `position(EntryId) -> Option<usize>`.
  - `Playlists` (ordered), with these methods:
    - `iter()`, `len()`, `is_empty()`, `first_id()`, `get(id)`, `get_mut(id)`, `add(Playlist)`;
    - `rename(id, String)`, `remove(id) -> Result<Playlist>`;
    - `find(EntryId) -> Option<(PlaylistId, usize)>`, `entry(EntryId) -> Option<&PlaylistEntry>`, `mark_played(EntryId)`;
    - `next_playable_after(EntryId, &Library)`, `first_playable(PlaylistId, &Library)`;
    - `insert(PlaylistId, index, Vec<PlaylistEntry>)`, `remove_entry(EntryId) -> Result<PlaylistEntry>`, `move_entry(EntryId, to: PlaylistId, index)`, `duplicate(EntryId, new_id: EntryId) -> Result<EntryId>`;
    - `references_track(TrackId) -> bool`, `max_raw_id() -> u64`.
  - `move_entry` index semantics: `index` is a drop position in the target list **before** the entry is removed, exactly like a drag-and-drop insertion line. Moving down inside the same list therefore lands at `index - 1`.

- [ ] **Step 1: Write the error type** (no behaviour to test on its own)

`crates/fp-model/src/error.rs`:
```rust
use thiserror::Error;

use crate::ids::{EntryId, PlayerId, PlaylistId};

/// Why a command was refused. A refused command never changes the state.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ModelError {
    #[error("unknown player {0:?}")]
    UnknownPlayer(PlayerId),
    #[error("unknown playlist {0:?}")]
    UnknownPlaylist(PlaylistId),
    #[error("unknown playlist entry {0:?}")]
    UnknownEntry(EntryId),
    #[error("there are no playlists")]
    NoPlaylists,
    #[error("the last playlist cannot be deleted")]
    LastPlaylist,
    #[error("playlist {0:?} contains an entry that is on air")]
    PlaylistOnAir(PlaylistId),
    #[error("entry {0:?} is on air and cannot be removed")]
    EntryOnAir(EntryId),
    #[error("the current entry cannot be set as next")]
    NextIsCurrent,
    #[error("stop after current is only available in continuous mode")]
    StopAfterInSingle,
    #[error("player count {requested} is outside 1..={max}")]
    PlayerCountOutOfRange { requested: usize, max: usize },
    #[error("player {0:?} is busy and cannot be removed")]
    PlayerBusy(PlayerId),
}
```

- [ ] **Step 2: Write the failing unit tests for playlists**

Create `crates/fp-model/src/playlist.rs` with the test module:
```rust
//! Playlists: ordered lists of entries pointing at library tracks.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::track::{FileState, Track};
    use std::path::PathBuf;

    fn setup() -> (Playlists, Library) {
        let mut lib = Library::default();
        for t in 1..=4 {
            lib.insert(Track::new(TrackId(t), PathBuf::from(format!("/m/{t}.flac"))));
        }
        let mut lists = Playlists::default();
        let mut a = Playlist::new(PlaylistId(100), "A");
        for (e, t) in [(1, 1), (2, 2), (3, 3)] {
            a.entries.push(PlaylistEntry { id: EntryId(e), track: TrackId(t), played: false });
        }
        lists.add(a);
        lists.add(Playlist::new(PlaylistId(200), "B"));
        (lists, lib)
    }

    fn ids(lists: &Playlists, pl: u64) -> Vec<u64> {
        lists.get(PlaylistId(pl)).unwrap().entries.iter().map(|e| e.id.0).collect()
    }

    #[test]
    fn moving_down_in_the_same_list_uses_the_drop_line_before_removal() {
        let (mut lists, _) = setup();
        lists.move_entry(EntryId(1), PlaylistId(100), 2).unwrap();
        assert_eq!(ids(&lists, 100), vec![2, 1, 3]);
    }

    #[test]
    fn moving_to_another_list_clamps_the_index() {
        let (mut lists, _) = setup();
        lists.move_entry(EntryId(2), PlaylistId(200), 99).unwrap();
        assert_eq!(ids(&lists, 100), vec![1, 3]);
        assert_eq!(ids(&lists, 200), vec![2]);
    }

    #[test]
    fn next_playable_skips_missing_and_unreadable_tracks() {
        let (lists, mut lib) = setup();
        lib.get_mut(TrackId(2)).unwrap().file_state = FileState::Missing;
        assert_eq!(lists.next_playable_after(EntryId(1), &lib), Some(EntryId(3)));
        lib.get_mut(TrackId(3)).unwrap().file_state = FileState::Unreadable;
        assert_eq!(lists.next_playable_after(EntryId(1), &lib), None);
        assert_eq!(lists.first_playable(PlaylistId(100), &lib), Some(EntryId(1)));
    }

    #[test]
    fn duplicate_inserts_an_unplayed_copy_right_after() {
        let (mut lists, _) = setup();
        lists.mark_played(EntryId(1));
        let copy = lists.duplicate(EntryId(1), EntryId(9)).unwrap();
        assert_eq!(copy, EntryId(9));
        assert_eq!(ids(&lists, 100), vec![1, 9, 2, 3]);
        assert!(!lists.entry(EntryId(9)).unwrap().played);
        assert_eq!(lists.entry(EntryId(9)).unwrap().track, TrackId(1));
    }

    #[test]
    fn the_last_playlist_cannot_be_removed() {
        let (mut lists, _) = setup();
        lists.remove(PlaylistId(200)).unwrap();
        assert_eq!(lists.remove(PlaylistId(100)), Err(ModelError::LastPlaylist));
        assert_eq!(lists.remove(PlaylistId(5)), Err(ModelError::UnknownPlaylist(PlaylistId(5))));
    }

    #[test]
    fn unknown_entries_are_reported() {
        let (mut lists, _) = setup();
        assert_eq!(lists.remove_entry(EntryId(77)), Err(ModelError::UnknownEntry(EntryId(77))));
        assert_eq!(
            lists.move_entry(EntryId(1), PlaylistId(5), 0),
            Err(ModelError::UnknownPlaylist(PlaylistId(5)))
        );
        assert_eq!(ids(&lists, 100), vec![1, 2, 3]);
    }

    #[test]
    fn references_and_max_id() {
        let (lists, _) = setup();
        assert!(lists.references_track(TrackId(3)));
        assert!(!lists.references_track(TrackId(4)));
        assert_eq!(lists.max_raw_id(), 200);
    }
}
```

Add to `lib.rs`:
```rust
pub mod error;
pub mod playlist;

pub use error::ModelError;
pub use playlist::{Playlist, PlaylistEntry, Playlists};
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cargo test -p fp-model playlist`
Expected: FAIL to compile: `cannot find type Playlists`.

- [ ] **Step 4: Implement**

Insert above the test module in `playlist.rs`:
```rust
use serde::{Deserialize, Serialize};

use crate::error::ModelError;
use crate::ids::{EntryId, PlaylistId, TrackId};
use crate::track::Library;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaylistEntry {
    pub id: EntryId,
    pub track: TrackId,
    pub played: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Playlist {
    pub id: PlaylistId,
    pub name: String,
    pub entries: Vec<PlaylistEntry>,
}

impl Playlist {
    pub fn new(id: PlaylistId, name: impl Into<String>) -> Self {
        Self { id, name: name.into(), entries: Vec::new() }
    }

    pub fn position(&self, entry: EntryId) -> Option<usize> {
        self.entries.iter().position(|e| e.id == entry)
    }
}

/// All playlists in display (tab) order.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Playlists {
    lists: Vec<Playlist>,
}

impl Playlists {
    pub fn iter(&self) -> impl Iterator<Item = &Playlist> {
        self.lists.iter()
    }

    pub fn len(&self) -> usize {
        self.lists.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lists.is_empty()
    }

    pub fn first_id(&self) -> Option<PlaylistId> {
        self.lists.first().map(|p| p.id)
    }

    pub fn get(&self, id: PlaylistId) -> Option<&Playlist> {
        self.lists.iter().find(|p| p.id == id)
    }

    pub fn get_mut(&mut self, id: PlaylistId) -> Option<&mut Playlist> {
        self.lists.iter_mut().find(|p| p.id == id)
    }

    pub fn add(&mut self, playlist: Playlist) {
        self.lists.push(playlist);
    }

    pub fn rename(&mut self, id: PlaylistId, name: String) -> Result<(), ModelError> {
        self.get_mut(id).ok_or(ModelError::UnknownPlaylist(id))?.name = name;
        Ok(())
    }

    /// Removes a playlist. The last remaining playlist cannot be removed.
    pub fn remove(&mut self, id: PlaylistId) -> Result<Playlist, ModelError> {
        let pos = self
            .lists
            .iter()
            .position(|p| p.id == id)
            .ok_or(ModelError::UnknownPlaylist(id))?;
        if self.lists.len() == 1 {
            return Err(ModelError::LastPlaylist);
        }
        Ok(self.lists.remove(pos))
    }

    pub fn find(&self, entry: EntryId) -> Option<(PlaylistId, usize)> {
        self.lists.iter().find_map(|p| p.position(entry).map(|i| (p.id, i)))
    }

    pub fn entry(&self, entry: EntryId) -> Option<&PlaylistEntry> {
        self.lists.iter().find_map(|p| p.entries.iter().find(|e| e.id == entry))
    }

    pub fn mark_played(&mut self, entry: EntryId) {
        if let Some(e) = self
            .lists
            .iter_mut()
            .find_map(|p| p.entries.iter_mut().find(|e| e.id == entry))
        {
            e.played = true;
        }
    }

    /// The first entry after `entry`, in the same playlist, whose track is playable.
    pub fn next_playable_after(&self, entry: EntryId, library: &Library) -> Option<EntryId> {
        let (pl, pos) = self.find(entry)?;
        self.get(pl)?
            .entries
            .iter()
            .skip(pos + 1)
            .find(|e| library.is_playable(e.track))
            .map(|e| e.id)
    }

    pub fn first_playable(&self, playlist: PlaylistId, library: &Library) -> Option<EntryId> {
        self.get(playlist)?
            .entries
            .iter()
            .find(|e| library.is_playable(e.track))
            .map(|e| e.id)
    }

    /// Inserts entries at `index` (clamped to the list length).
    pub fn insert(
        &mut self,
        playlist: PlaylistId,
        index: usize,
        entries: Vec<PlaylistEntry>,
    ) -> Result<(), ModelError> {
        let list = self.get_mut(playlist).ok_or(ModelError::UnknownPlaylist(playlist))?;
        let at = index.min(list.entries.len());
        list.entries.splice(at..at, entries);
        Ok(())
    }

    pub fn remove_entry(&mut self, entry: EntryId) -> Result<PlaylistEntry, ModelError> {
        let (pl, pos) = self.find(entry).ok_or(ModelError::UnknownEntry(entry))?;
        let list = self.get_mut(pl).ok_or(ModelError::UnknownPlaylist(pl))?;
        Ok(list.entries.remove(pos))
    }

    /// Moves `entry` to drop position `index` of playlist `to`, where `index`
    /// is measured before the entry is taken out of its current place.
    pub fn move_entry(&mut self, entry: EntryId, to: PlaylistId, index: usize) -> Result<(), ModelError> {
        if self.get(to).is_none() {
            return Err(ModelError::UnknownPlaylist(to));
        }
        let (from, pos) = self.find(entry).ok_or(ModelError::UnknownEntry(entry))?;
        let moved = self.remove_entry(entry)?;
        let target = if from == to && pos < index { index - 1 } else { index };
        self.insert(to, target, vec![moved])
    }

    /// Inserts an unplayed copy of `entry` right after it.
    pub fn duplicate(&mut self, entry: EntryId, new_id: EntryId) -> Result<EntryId, ModelError> {
        let (pl, pos) = self.find(entry).ok_or(ModelError::UnknownEntry(entry))?;
        let original = *self.entry(entry).ok_or(ModelError::UnknownEntry(entry))?;
        let copy = PlaylistEntry { id: new_id, track: original.track, played: false };
        self.insert(pl, pos + 1, vec![copy])?;
        Ok(new_id)
    }

    pub fn references_track(&self, track: TrackId) -> bool {
        self.lists.iter().any(|p| p.entries.iter().any(|e| e.track == track))
    }

    pub fn max_raw_id(&self) -> u64 {
        self.lists
            .iter()
            .flat_map(|p| std::iter::once(p.id.0).chain(p.entries.iter().map(|e| e.id.0)))
            .max()
            .unwrap_or(0)
    }
}
```

- [ ] **Step 5: Run the unit tests to verify they pass**

Run: `cargo test -p fp-model playlist`
Expected: 7 tests PASS.

- [ ] **Step 6: Write the property test against a reference model**

`crates/fp-model/tests/playlist_props.rs`:
```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Random sequences of playlist edits must behave exactly like a trivially
//! correct reference implementation built on plain vectors, and must never
//! lose or duplicate entry ids.

use fp_model::{EntryId, Playlist, PlaylistEntry, PlaylistId, Playlists, TrackId};
use proptest::prelude::*;

#[derive(Debug, Clone)]
enum Op {
    Insert { list: usize, index: usize, track: u64 },
    Remove { pick: usize },
    Move { pick: usize, list: usize, index: usize },
    Duplicate { pick: usize },
}

fn op() -> impl Strategy<Value = Op> {
    prop_oneof![
        (0..3usize, 0..20usize, 0..50u64)
            .prop_map(|(list, index, track)| Op::Insert { list, index, track }),
        (0..100usize).prop_map(|pick| Op::Remove { pick }),
        (0..100usize, 0..3usize, 0..20usize)
            .prop_map(|(pick, list, index)| Op::Move { pick, list, index }),
        (0..100usize).prop_map(|pick| Op::Duplicate { pick }),
    ]
}

/// Reference model: one vector of (entry id, track id) per playlist.
type Reference = Vec<Vec<(u64, u64)>>;

fn all_entries(r: &Reference) -> Vec<u64> {
    r.iter().flatten().map(|(e, _)| *e).collect()
}

fn locate(r: &Reference, id: u64) -> (usize, usize) {
    r.iter()
        .enumerate()
        .find_map(|(li, l)| l.iter().position(|(e, _)| *e == id).map(|p| (li, p)))
        .unwrap()
}

proptest! {
    #[test]
    fn playlist_operations_match_reference(ops in proptest::collection::vec(op(), 1..60)) {
        let mut lists = Playlists::default();
        for i in 0..3u64 {
            lists.add(Playlist::new(PlaylistId(i), format!("L{i}")));
        }
        let mut reference: Reference = vec![Vec::new(); 3];
        let mut next_entry = 100u64;

        for op in ops {
            match op {
                Op::Insert { list, index, track } => {
                    next_entry += 1;
                    let entry = PlaylistEntry { id: EntryId(next_entry), track: TrackId(track), played: false };
                    lists.insert(PlaylistId(list as u64), index, vec![entry]).unwrap();
                    let at = index.min(reference[list].len());
                    reference[list].insert(at, (next_entry, track));
                }
                Op::Remove { pick } => {
                    let ids = all_entries(&reference);
                    if ids.is_empty() { continue; }
                    let id = ids[pick % ids.len()];
                    lists.remove_entry(EntryId(id)).unwrap();
                    let (li, pos) = locate(&reference, id);
                    reference[li].remove(pos);
                }
                Op::Move { pick, list, index } => {
                    let ids = all_entries(&reference);
                    if ids.is_empty() { continue; }
                    let id = ids[pick % ids.len()];
                    lists.move_entry(EntryId(id), PlaylistId(list as u64), index).unwrap();
                    let (from, pos) = locate(&reference, id);
                    let item = reference[from].remove(pos);
                    let target = if from == list && pos < index { index - 1 } else { index };
                    let at = target.min(reference[list].len());
                    reference[list].insert(at, item);
                }
                Op::Duplicate { pick } => {
                    let ids = all_entries(&reference);
                    if ids.is_empty() { continue; }
                    let id = ids[pick % ids.len()];
                    next_entry += 1;
                    lists.duplicate(EntryId(id), EntryId(next_entry)).unwrap();
                    let (li, pos) = locate(&reference, id);
                    let track = reference[li][pos].1;
                    reference[li].insert(pos + 1, (next_entry, track));
                }
            }
            let actual: Reference = lists
                .iter()
                .map(|p| p.entries.iter().map(|e| (e.id.0, e.track.0)).collect())
                .collect();
            prop_assert_eq!(&actual, &reference);
            let mut unique = all_entries(&reference);
            unique.sort_unstable();
            unique.dedup();
            prop_assert_eq!(unique.len(), all_entries(&reference).len());
        }
    }
}
```

- [ ] **Step 7: Run the property test**

Run: `cargo test -p fp-model --test playlist_props`
Expected: PASS (256 generated cases).

- [ ] **Step 8: Check and commit**

Run: `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
```bash
git add crates/fp-model
git commit -m "feat(model): add playlists with edit operations and property tests

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: Configuration with clamped validation

**Files:**
- Create: `crates/fp-model/src/config.rs`, `crates/fp-model/src/player.rs` (only `PlayMode` in this task; the rest of the file comes in Task 5)
- Modify: `crates/fp-model/src/lib.rs`

**Interfaces:**
- Consumes: `PlayerId` (Task 1).
- Produces:
  - `PlayMode { Single, Continuous (default) }` in `player.rs`.
  - `Config { players, analysis, outputs, ui, limits, tuning }`, with every group `#[serde(default)]` and a `Default` impl carrying the documented defaults.
  - `PlayersConfig { count: usize = 4, default_mode = Continuous, fade_ms: u32 = 1000, auto_segue: bool = true, end_warning_secs: f64 = 10 }`.
  - `AnalysisSettings`:

    | Field | Default |
    |---|---|
    | `silence_threshold_db: f64` | −40 |
    | `segue_threshold_db: f64` | −18 |
    | `segue_max_secs: f64` | 8 |
    | `outro_drop_db: f64` | 6 |
    | `outro_max_secs: f64` | 30 |
    | `markers_min_duration_secs: f64` | 60 |
    | `peak_bucket_ms: u32` | 10 |
    | `rms_window_ms: u32` | 50 |
    | `cover_thumb_px: u32` | 128 |

  - `OutputsConfig { backend: Option<String>, sample_rate: u32 = 48000, buffer_frames: u32 = 512, routes: Vec<PlayerRoutes> }`.
  - `PlayerRoutes { player: PlayerId, main: Option<Route>, cue: Option<Route> }` and `Route { backend: String, device: String, first_channel: u16 }`.
  - `UiConfig { wave_color: String = "sand", music_dir: Option<PathBuf>, language: Option<String> }`.
  - `Limits`:

    | Field | Default |
    |---|---|
    | `max_players: usize` | 16 |
    | `max_cover_bytes: u64` | 20 MiB |
    | `max_cover_pixels: u32` | 8000 |
    | `max_state_file_bytes: u64` | 50 MiB |
    | `max_playlist_file_bytes: u64` | 10 MiB |
    | `backup_count: usize` | 3 |

  - `Tuning`:

    | Field | Default |
    |---|---|
    | `declick_ms: f64` | 5 |
    | `pause_ramp_ms: f64` | 10 |
    | `prebuffer_secs: f64` | 5 |
    | `ready_threshold_ms: f64` | 500 |
    | `mixer_headroom: f64` | 2.0 |
    | `max_commands_per_block: usize` | 256 |
    | `schedule_lead_ms: f64` | 200 |
    | `conductor_tick_ms: f64` | 5 |
    | `watchdog_timeout_ms: f64` | 500 |
    | `reconnect_interval_ms: f64` | 2000 |
    | `gain_smoothing_ms: f64` | 20 |
    | `save_debounce_ms: f64` | 1000 |

  - `Config::validate(&mut self) -> Vec<ConfigWarning>` clamps every numeric field to its range; NaN goes to the minimum.
  - `ConfigWarning { field: &'static str, message: String }`, which implements `Display` as `"{field}: {message}"`.

- [ ] **Step 1: Write the failing tests**

`crates/fp-model/src/player.rs` (initial content):
```rust
//! Per-player state.

use serde::{Deserialize, Serialize};

/// `Single` stops after every track; `Continuous` chains tracks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PlayMode {
    Single,
    #[default]
    Continuous,
}
```

`crates/fp-model/src/config.rs` (test module first):
```rust
//! Application configuration. Every tunable value lives here with a
//! documented default; nothing product-related is a hardcoded constant.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_valid() {
        let mut c = Config::default();
        assert!(c.validate().is_empty());
        assert_eq!(c.players.count, 4);
        assert_eq!(c.players.fade_ms, 1000);
        assert_eq!(c.analysis.segue_threshold_db, -18.0);
        assert_eq!(c.analysis.segue_max_secs, 8.0);
        assert_eq!(c.limits.max_players, 16);
    }

    #[test]
    fn missing_fields_take_defaults() {
        let c: Config = serde_json::from_str(r#"{"players":{"fade_ms":2000}}"#).unwrap();
        assert_eq!(c.players.fade_ms, 2000);
        assert_eq!(c.players.count, 4);
        assert_eq!(c.tuning, Tuning::default());
    }

    #[test]
    fn nan_and_out_of_range_values_are_clamped() {
        let mut c = Config::default();
        c.players.count = 0;
        c.analysis.segue_threshold_db = f64::NAN;
        c.tuning.prebuffer_secs = 1e9;
        let warnings = c.validate();
        assert_eq!(c.players.count, 1);
        assert_eq!(c.analysis.segue_threshold_db, -60.0);
        assert_eq!(c.tuning.prebuffer_secs, 60.0);
        let fields: Vec<_> = warnings.iter().map(|w| w.field).collect();
        assert!(fields.contains(&"players.count"));
        assert!(fields.contains(&"analysis.segue_threshold_db"));
        assert!(fields.contains(&"tuning.prebuffer_secs"));
        assert!(warnings[0].to_string().contains("using"));
    }

    #[test]
    fn player_count_is_capped_by_the_resource_limit() {
        let mut c = Config::default();
        c.limits.max_players = 8;
        c.players.count = 12;
        c.validate();
        assert_eq!(c.players.count, 8);
    }
}
```

Add to `lib.rs`:
```rust
pub mod config;
pub mod player;

pub use config::{
    AnalysisSettings, Config, ConfigWarning, Limits, OutputsConfig, PlayerRoutes, PlayersConfig, Route,
    Tuning, UiConfig,
};
pub use player::PlayMode;
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-model config`
Expected: FAIL to compile: `cannot find type Config`.

- [ ] **Step 3: Implement**

Insert above the test module in `config.rs`:
```rust
use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::ids::PlayerId;
use crate::player::PlayMode;

const MIB: u64 = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub players: PlayersConfig,
    pub analysis: AnalysisSettings,
    pub outputs: OutputsConfig,
    pub ui: UiConfig,
    pub limits: Limits,
    pub tuning: Tuning,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PlayersConfig {
    pub count: usize,
    pub default_mode: PlayMode,
    pub fade_ms: u32,
    pub auto_segue: bool,
    pub end_warning_secs: f64,
}

impl Default for PlayersConfig {
    fn default() -> Self {
        Self { count: 4, default_mode: PlayMode::Continuous, fade_ms: 1000, auto_segue: true, end_warning_secs: 10.0 }
    }
}

/// Parameters of automatic marker detection (spec §6).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AnalysisSettings {
    pub silence_threshold_db: f64,
    pub segue_threshold_db: f64,
    pub segue_max_secs: f64,
    pub outro_drop_db: f64,
    pub outro_max_secs: f64,
    pub markers_min_duration_secs: f64,
    pub peak_bucket_ms: u32,
    pub rms_window_ms: u32,
    pub cover_thumb_px: u32,
}

impl Default for AnalysisSettings {
    fn default() -> Self {
        Self {
            silence_threshold_db: -40.0,
            segue_threshold_db: -18.0,
            segue_max_secs: 8.0,
            outro_drop_db: 6.0,
            outro_max_secs: 30.0,
            markers_min_duration_secs: 60.0,
            peak_bucket_ms: 10,
            rms_window_ms: 50,
            cover_thumb_px: 128,
        }
    }
}

/// Where a bus is sent: a backend, a device and the first channel of a stereo pair.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Route {
    pub backend: String,
    pub device: String,
    pub first_channel: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerRoutes {
    pub player: PlayerId,
    pub main: Option<Route>,
    pub cue: Option<Route>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct OutputsConfig {
    /// Backend id; `None` means the platform default.
    pub backend: Option<String>,
    pub sample_rate: u32,
    pub buffer_frames: u32,
    pub routes: Vec<PlayerRoutes>,
}

impl Default for OutputsConfig {
    fn default() -> Self {
        Self { backend: None, sample_rate: 48_000, buffer_frames: 512, routes: Vec::new() }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    /// Name of a waveform colour in the theme palette.
    pub wave_color: String,
    pub music_dir: Option<PathBuf>,
    /// BCP-47 tag; `None` follows the OS locale.
    pub language: Option<String>,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self { wave_color: "sand".to_owned(), music_dir: None, language: None }
    }
}

/// Resource guards. Edited only in the config file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Limits {
    pub max_players: usize,
    pub max_cover_bytes: u64,
    pub max_cover_pixels: u32,
    pub max_state_file_bytes: u64,
    pub max_playlist_file_bytes: u64,
    pub backup_count: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_players: 16,
            max_cover_bytes: 20 * MIB,
            max_cover_pixels: 8000,
            max_state_file_bytes: 50 * MIB,
            max_playlist_file_bytes: 10 * MIB,
            backup_count: 3,
        }
    }
}

/// Engine internals. Edited only in the "advanced" part of the config file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Tuning {
    pub declick_ms: f64,
    pub pause_ramp_ms: f64,
    pub prebuffer_secs: f64,
    pub ready_threshold_ms: f64,
    pub mixer_headroom: f64,
    pub max_commands_per_block: usize,
    pub schedule_lead_ms: f64,
    pub conductor_tick_ms: f64,
    pub watchdog_timeout_ms: f64,
    pub reconnect_interval_ms: f64,
    pub gain_smoothing_ms: f64,
    pub save_debounce_ms: f64,
}

impl Default for Tuning {
    fn default() -> Self {
        Self {
            declick_ms: 5.0,
            pause_ramp_ms: 10.0,
            prebuffer_secs: 5.0,
            ready_threshold_ms: 500.0,
            mixer_headroom: 2.0,
            max_commands_per_block: 256,
            schedule_lead_ms: 200.0,
            conductor_tick_ms: 5.0,
            watchdog_timeout_ms: 500.0,
            reconnect_interval_ms: 2000.0,
            gain_smoothing_ms: 20.0,
            save_debounce_ms: 1000.0,
        }
    }
}

/// A value that was out of range and has been replaced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigWarning {
    pub field: &'static str,
    pub message: String,
}

impl fmt::Display for ConfigWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

/// Clamps `value` into `min..=max`. NaN (which fails every comparison) becomes `min`.
#[allow(clippy::neg_cmp_op_on_partial_ord)] // negated comparisons are deliberate: they catch NaN
fn clamp_to<T: PartialOrd + Copy + fmt::Display>(
    value: &mut T,
    min: T,
    max: T,
    field: &'static str,
    out: &mut Vec<ConfigWarning>,
) {
    let original = *value;
    if !(original >= min) {
        *value = min;
    } else if !(original <= max) {
        *value = max;
    } else {
        return;
    }
    out.push(ConfigWarning { field, message: format!("{original} is outside {min}..={max}; using {}", *value) });
}

impl Config {
    /// Brings every value into its valid range and reports what was changed.
    pub fn validate(&mut self) -> Vec<ConfigWarning> {
        let mut w = Vec::new();

        let l = &mut self.limits;
        clamp_to(&mut l.max_players, 1, 256, "limits.max_players", &mut w);
        clamp_to(&mut l.max_cover_bytes, MIB, 500 * MIB, "limits.max_cover_bytes", &mut w);
        clamp_to(&mut l.max_cover_pixels, 256, 30_000, "limits.max_cover_pixels", &mut w);
        clamp_to(&mut l.max_state_file_bytes, MIB, 1024 * MIB, "limits.max_state_file_bytes", &mut w);
        clamp_to(&mut l.max_playlist_file_bytes, 64 * 1024, 1024 * MIB, "limits.max_playlist_file_bytes", &mut w);
        clamp_to(&mut l.backup_count, 0, 20, "limits.backup_count", &mut w);

        let max_players = self.limits.max_players;
        let p = &mut self.players;
        clamp_to(&mut p.count, 1, max_players, "players.count", &mut w);
        clamp_to(&mut p.fade_ms, 50, 10_000, "players.fade_ms", &mut w);
        clamp_to(&mut p.end_warning_secs, 0.0, 120.0, "players.end_warning_secs", &mut w);

        let a = &mut self.analysis;
        clamp_to(&mut a.silence_threshold_db, -96.0, -10.0, "analysis.silence_threshold_db", &mut w);
        clamp_to(&mut a.segue_threshold_db, -60.0, 0.0, "analysis.segue_threshold_db", &mut w);
        clamp_to(&mut a.segue_max_secs, 0.0, 60.0, "analysis.segue_max_secs", &mut w);
        clamp_to(&mut a.outro_drop_db, 0.0, 40.0, "analysis.outro_drop_db", &mut w);
        clamp_to(&mut a.outro_max_secs, 0.0, 300.0, "analysis.outro_max_secs", &mut w);
        clamp_to(&mut a.markers_min_duration_secs, 0.0, 3600.0, "analysis.markers_min_duration_secs", &mut w);
        clamp_to(&mut a.peak_bucket_ms, 1, 1000, "analysis.peak_bucket_ms", &mut w);
        clamp_to(&mut a.rms_window_ms, 5, 1000, "analysis.rms_window_ms", &mut w);
        clamp_to(&mut a.cover_thumb_px, 16, 1024, "analysis.cover_thumb_px", &mut w);

        let o = &mut self.outputs;
        clamp_to(&mut o.sample_rate, 8_000, 768_000, "outputs.sample_rate", &mut w);
        clamp_to(&mut o.buffer_frames, 16, 16_384, "outputs.buffer_frames", &mut w);

        let t = &mut self.tuning;
        clamp_to(&mut t.declick_ms, 0.5, 50.0, "tuning.declick_ms", &mut w);
        clamp_to(&mut t.pause_ramp_ms, 0.5, 200.0, "tuning.pause_ramp_ms", &mut w);
        clamp_to(&mut t.prebuffer_secs, 1.0, 60.0, "tuning.prebuffer_secs", &mut w);
        clamp_to(&mut t.ready_threshold_ms, 50.0, 10_000.0, "tuning.ready_threshold_ms", &mut w);
        clamp_to(&mut t.mixer_headroom, 1.0, 8.0, "tuning.mixer_headroom", &mut w);
        clamp_to(&mut t.max_commands_per_block, 16, 4096, "tuning.max_commands_per_block", &mut w);
        clamp_to(&mut t.schedule_lead_ms, 20.0, 5000.0, "tuning.schedule_lead_ms", &mut w);
        clamp_to(&mut t.conductor_tick_ms, 1.0, 50.0, "tuning.conductor_tick_ms", &mut w);
        clamp_to(&mut t.watchdog_timeout_ms, 100.0, 10_000.0, "tuning.watchdog_timeout_ms", &mut w);
        clamp_to(&mut t.reconnect_interval_ms, 250.0, 60_000.0, "tuning.reconnect_interval_ms", &mut w);
        clamp_to(&mut t.gain_smoothing_ms, 1.0, 500.0, "tuning.gain_smoothing_ms", &mut w);
        clamp_to(&mut t.save_debounce_ms, 100.0, 60_000.0, "tuning.save_debounce_ms", &mut w);

        w
    }
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-model config`
Expected: 4 tests PASS.

- [ ] **Step 5: Check and commit**

Run: `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
```bash
git add crates/fp-model
git commit -m "feat(model): add configuration groups with clamped validation

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: Player state, commands and transport rules (spec §3, rules 2–8)

**Files:**
- Modify: `crates/fp-model/src/player.rs`, `crates/fp-model/src/lib.rs`
- Create: `crates/fp-model/src/command.rs`, `crates/fp-model/src/state.rs`, `crates/fp-model/src/reducer.rs`
- Create: `crates/fp-model/tests/common/mod.rs`, `crates/fp-model/tests/transport.rs`

**Interfaces:**
- Consumes: everything from Tasks 1–4.
- Produces:
  - `Transport { Stopped (default), Playing, Paused }`, `CueState { entry: EntryId }`.
  - `ColumnWidths { number: Option<f32>, title: Option<f32>, duration: f32 = 52.0 }`.
  - `PlayerState` with pub fields `id`, `playlist`, `current`, `next`, `transport`, `fading`, `mode`, `stop_after_current`, `cue: Option<CueState>`, `volume: f32` and `columns`. It also has crate-private engine bookkeeping `preloaded: Option<EntryId>` and `scheduled: Option<TransitionPlan>`. Constructor: `PlayerState::new(id, playlist, mode)`.
  - `SourceRequest { entry, track: TrackId, path: PathBuf, from_secs: f64 }`.
  - `TransitionPlan::{StopAt { at_secs }, StartNextAt { at_secs, fade_current_until_secs: Option<f64> }}`. Times are seconds in the current track.
  - `Command`, which starts with `Play`, `Pause`, `Stop`, `FadeStop` (each `(PlayerId)`), `SetNext(PlayerId, EntryId)` and `InsertPaths { playlist, index, paths: Vec<PathBuf> }`. Tasks 6–7 add more variants.
  - `EngineEvent`, which starts with `FadeCompleted { player }` and `ReachedEnd { player }`.
  - `EngineAction`, which starts with:
    - `Preload { player, request: Option<SourceRequest> }`
    - `StartCurrent { player, request }`
    - `Crossfade { player, request, fade_ms: u32 }`
    - `FadeOutAndStop { player, fade_ms }`
    - `Pause { player }`, `Resume { player }`, `StopNow { player }`
  - `AppState { config, library, playlists, players: Vec<PlayerState>, ids }` with:
    - `AppState::new(config, default_playlist_name: &str)`;
    - `player_index(PlayerId) -> Result<usize, ModelError>`, `player(PlayerId) -> Result<&PlayerState, ModelError>`;
    - `track_for_entry(EntryId) -> Option<&Track>`, `request_from_cue_in(EntryId) -> Option<SourceRequest>`, `request_at(EntryId, secs: f64) -> Option<SourceRequest>`;
    - `is_on_air(EntryId) -> bool`.
  - Reducer: `fp_model::apply(&mut AppState, Command) -> Result<Vec<EngineAction>, ModelError>` and `fp_model::on_event(&mut AppState, EngineEvent) -> Vec<EngineAction>`. A refused command leaves the state unchanged.
  - Crate-private helpers used by later tasks: `advance`, `stop_player`, `fill_empty_next` and `reconcile` in `reducer.rs`.
  - Test fixture `tests/common/mod.rs`: `fixture(n) -> AppState`, where every track is 180 s and all 4 players show the first playlist. Also `entries(&AppState) -> Vec<EntryId>` and `p0(&AppState) -> PlayerId`.

- [ ] **Step 1: Write the fixture and failing tests**

`crates/fp-model/tests/common/mod.rs`:
```rust
#![allow(dead_code, clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;

use fp_model::{AppState, Command, Config, EntryId, PlayerId, apply};

/// Default config (4 players), one playlist named "Main" with `tracks`
/// entries of 180 s each. Every player shows that playlist.
pub fn fixture(tracks: usize) -> AppState {
    let mut state = AppState::new(Config::default(), "Main");
    let playlist = state.playlists.first_id().unwrap();
    let paths = (0..tracks).map(|i| PathBuf::from(format!("/music/track{i}.flac"))).collect();
    apply(&mut state, Command::InsertPaths { playlist, index: 0, paths }).unwrap();
    for track in state.library.iter_mut() {
        track.duration_secs = 180.0;
    }
    state
}

pub fn entries(state: &AppState) -> Vec<EntryId> {
    let playlist = state.playlists.first_id().unwrap();
    state.playlists.get(playlist).unwrap().entries.iter().map(|e| e.id).collect()
}

pub fn p0(state: &AppState) -> PlayerId {
    state.players[0].id
}
```

`crates/fp-model/tests/transport.rs`:
```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Spec §3 rules 2–8: next selection, play, pause, stop, fade stop.

mod common;

use common::{entries, fixture, p0};
use fp_model::{Command, EngineAction, EngineEvent, ModelError, Transport, apply, on_event};

#[test]
fn inserting_into_an_empty_playlist_marks_the_first_entry_as_next() {
    let state = fixture(3);
    let e = entries(&state);
    assert!(state.players.iter().all(|p| p.next == Some(e[0])));
}

#[test]
fn rule2_double_click_sets_next_but_never_the_current_entry() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::SetNext(p, e[2])).unwrap();
    assert_eq!(state.player(p).unwrap().next, Some(e[2]));
    apply(&mut state, Command::Play(p)).unwrap();
    assert_eq!(apply(&mut state, Command::SetNext(p, e[2])), Err(ModelError::NextIsCurrent));
}

#[test]
fn rule3_play_while_stopped_starts_next_and_advances() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    let actions = apply(&mut state, Command::Play(p)).unwrap();
    let player = state.player(p).unwrap();
    assert_eq!(player.current, Some(e[0]));
    assert_eq!(player.next, Some(e[1]));
    assert_eq!(player.transport, Transport::Playing);
    assert!(matches!(
        actions.first(),
        Some(EngineAction::StartCurrent { player, request }) if *player == p && request.entry == e[0] && request.from_secs == 0.0
    ));
    assert!(actions.iter().any(
        |a| matches!(a, EngineAction::Preload { player, request: Some(r) } if *player == p && r.entry == e[1])
    ));
}

#[test]
fn rule3_play_without_next_does_nothing() {
    let mut state = fixture(0);
    let p = p0(&state);
    assert!(apply(&mut state, Command::Play(p)).unwrap().is_empty());
    assert_eq!(state.player(p).unwrap().transport, Transport::Stopped);
}

#[test]
fn rule4_play_while_paused_resumes() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::Play(p)).unwrap();
    apply(&mut state, Command::Pause(p)).unwrap();
    let actions = apply(&mut state, Command::Play(p)).unwrap();
    assert_eq!(actions, vec![EngineAction::Resume { player: p }]);
    assert_eq!(state.player(p).unwrap().current, Some(e[0]));
    assert_eq!(state.player(p).unwrap().transport, Transport::Playing);
}

#[test]
fn rule5_play_while_playing_crossfades_into_next() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::Play(p)).unwrap();
    let actions = apply(&mut state, Command::Play(p)).unwrap();
    assert!(matches!(
        actions.first(),
        Some(EngineAction::Crossfade { player, request, fade_ms: 1000 }) if *player == p && request.entry == e[1]
    ));
    let player = state.player(p).unwrap();
    assert_eq!((player.current, player.next, player.fading), (Some(e[1]), Some(e[2]), true));
    assert!(state.playlists.entry(e[0]).unwrap().played);
}

#[test]
fn rule5_play_during_a_fade_is_ignored_until_the_fade_completes() {
    let mut state = fixture(3);
    let p = p0(&state);
    apply(&mut state, Command::Play(p)).unwrap();
    apply(&mut state, Command::Play(p)).unwrap();
    assert!(apply(&mut state, Command::Play(p)).unwrap().is_empty());
    on_event(&mut state, EngineEvent::FadeCompleted { player: p });
    assert!(!state.player(p).unwrap().fading);
}

#[test]
fn rule6_pause_toggles_and_is_ignored_during_a_fade() {
    let mut state = fixture(3);
    let p = p0(&state);
    apply(&mut state, Command::Play(p)).unwrap();
    assert_eq!(apply(&mut state, Command::Pause(p)).unwrap(), vec![EngineAction::Pause { player: p }]);
    assert_eq!(state.player(p).unwrap().transport, Transport::Paused);
    assert_eq!(apply(&mut state, Command::Pause(p)).unwrap(), vec![EngineAction::Resume { player: p }]);
    apply(&mut state, Command::Play(p)).unwrap(); // crossfade → fading
    assert!(apply(&mut state, Command::Pause(p)).unwrap().is_empty());
}

#[test]
fn rule7_stop_marks_played_and_keeps_the_green_next() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::Play(p)).unwrap();
    let actions = apply(&mut state, Command::Stop(p)).unwrap();
    assert_eq!(actions.first(), Some(&EngineAction::StopNow { player: p }));
    let player = state.player(p).unwrap();
    assert_eq!((player.current, player.next, player.transport), (None, Some(e[1]), Transport::Stopped));
    assert!(state.playlists.entry(e[0]).unwrap().played);
}

#[test]
fn rule7_stop_keeps_an_explicitly_chosen_next() {
    let mut state = fixture(4);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::Play(p)).unwrap();
    apply(&mut state, Command::SetNext(p, e[3])).unwrap();
    apply(&mut state, Command::Stop(p)).unwrap();
    assert_eq!(state.player(p).unwrap().next, Some(e[3]));
}

#[test]
fn rule7_stopping_an_idle_player_does_nothing() {
    let mut state = fixture(1);
    let p = p0(&state);
    assert!(apply(&mut state, Command::Stop(p)).unwrap().is_empty());
}

#[test]
fn rule8_fade_stop_fades_then_stops_when_the_engine_reports_the_end() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::Play(p)).unwrap();
    let actions = apply(&mut state, Command::FadeStop(p)).unwrap();
    assert_eq!(actions, vec![EngineAction::FadeOutAndStop { player: p, fade_ms: 1000 }]);
    assert!(state.player(p).unwrap().fading);
    on_event(&mut state, EngineEvent::ReachedEnd { player: p });
    let player = state.player(p).unwrap();
    assert_eq!((player.current, player.next, player.fading), (None, Some(e[1]), false));
}

#[test]
fn unknown_player_is_refused() {
    let mut state = fixture(1);
    let ghost = fp_model::PlayerId(9999);
    assert_eq!(apply(&mut state, Command::Play(ghost)), Err(ModelError::UnknownPlayer(ghost)));
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-model --test transport`
Expected: FAIL to compile: `cannot find type AppState` / `apply`.

- [ ] **Step 3: Implement the player state**

Replace `crates/fp-model/src/player.rs` with:
```rust
//! Per-player state.

use serde::{Deserialize, Serialize};

use crate::command::TransitionPlan;
use crate::ids::{EntryId, PlayerId, PlaylistId};

/// `Single` stops after every track; `Continuous` chains tracks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PlayMode {
    Single,
    #[default]
    Continuous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Transport {
    #[default]
    Stopped,
    Playing,
    Paused,
}

/// Pre-listen (cue) of one entry on the player's Cue bus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CueState {
    pub entry: EntryId,
}

/// User-resized track-table column widths, in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ColumnWidths {
    pub number: Option<f32>,
    pub title: Option<f32>,
    pub duration: f32,
}

impl Default for ColumnWidths {
    fn default() -> Self {
        Self { number: None, title: None, duration: 52.0 }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerState {
    pub id: PlayerId,
    /// Playlist shown in this player's tab strip.
    pub playlist: PlaylistId,
    pub current: Option<EntryId>,
    pub next: Option<EntryId>,
    pub transport: Transport,
    pub fading: bool,
    pub mode: PlayMode,
    pub stop_after_current: bool,
    pub cue: Option<CueState>,
    /// Linear gain 0.0–1.0.
    pub volume: f32,
    pub columns: ColumnWidths,
    /// Entry the engine was last asked to preload.
    pub(crate) preloaded: Option<EntryId>,
    /// Transition plan the engine was last given.
    pub(crate) scheduled: Option<TransitionPlan>,
}

impl PlayerState {
    pub fn new(id: PlayerId, playlist: PlaylistId, mode: PlayMode) -> Self {
        Self {
            id,
            playlist,
            current: None,
            next: None,
            transport: Transport::Stopped,
            fading: false,
            mode,
            stop_after_current: false,
            cue: None,
            volume: 1.0,
            columns: ColumnWidths::default(),
            preloaded: None,
            scheduled: None,
        }
    }
}
```

- [ ] **Step 4: Implement commands, events and actions**

`crates/fp-model/src/command.rs`:
```rust
//! The vocabulary between the UI, the model and the audio engine.

use std::path::PathBuf;

use crate::ids::{EntryId, PlayerId, PlaylistId, TrackId};

/// A user intent, sent by the UI.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Play(PlayerId),
    Pause(PlayerId),
    Stop(PlayerId),
    FadeStop(PlayerId),
    SetNext(PlayerId, EntryId),
    InsertPaths { playlist: PlaylistId, index: usize, paths: Vec<PathBuf> },
}

/// Something the audio engine observed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineEvent {
    /// A crossfade or segue fade-out finished.
    FadeCompleted { player: PlayerId },
    /// The current source stopped: a scheduled `StopAt` was reached or a fade stop completed.
    ReachedEnd { player: PlayerId },
}

/// Everything the engine needs to open and position one source.
#[derive(Debug, Clone, PartialEq)]
pub struct SourceRequest {
    pub entry: EntryId,
    pub track: TrackId,
    pub path: PathBuf,
    pub from_secs: f64,
}

/// What must happen at the end of the current track, in seconds of that track.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransitionPlan {
    StopAt { at_secs: f64 },
    /// Start the preloaded next at `at_secs`. When `fade_current_until_secs`
    /// is set, the current source keeps playing and fades out until then (overlap).
    StartNextAt { at_secs: f64, fade_current_until_secs: Option<f64> },
}

/// Work for the audio engine, produced by the model.
#[derive(Debug, Clone, PartialEq)]
pub enum EngineAction {
    /// Prepare (or with `None`, drop) the source that will play next.
    Preload { player: PlayerId, request: Option<SourceRequest> },
    /// Start playing now (normally the preloaded source).
    StartCurrent { player: PlayerId, request: SourceRequest },
    /// Fade the current source out over `fade_ms` while `request` starts now at full level.
    Crossfade { player: PlayerId, request: SourceRequest, fade_ms: u32 },
    /// Fade the current source out over `fade_ms`, then stop and report `ReachedEnd`.
    FadeOutAndStop { player: PlayerId, fade_ms: u32 },
    Pause { player: PlayerId },
    Resume { player: PlayerId },
    StopNow { player: PlayerId },
}
```

- [ ] **Step 5: Implement the application state**

`crates/fp-model/src/state.rs`:
```rust
use crate::command::SourceRequest;
use crate::config::Config;
use crate::error::ModelError;
use crate::ids::{EntryId, IdGen, PlayerId};
use crate::player::PlayerState;
use crate::playlist::{Playlist, Playlists};
use crate::track::{Library, Track};

/// The complete authoritative state, owned by the conductor.
#[derive(Debug, Clone, PartialEq)]
pub struct AppState {
    pub config: Config,
    pub library: Library,
    pub playlists: Playlists,
    /// Players in display order.
    pub players: Vec<PlayerState>,
    pub ids: IdGen,
}

impl AppState {
    /// A fresh state: one empty playlist and `config.players.count` stopped players.
    pub fn new(config: Config, default_playlist_name: &str) -> Self {
        let mut ids = IdGen::default();
        let mut playlists = Playlists::default();
        let first = ids.playlist();
        playlists.add(Playlist::new(first, default_playlist_name));
        let mode = config.players.default_mode;
        let players = (0..config.players.count).map(|_| PlayerState::new(ids.player(), first, mode)).collect();
        Self { config, library: Library::default(), playlists, players, ids }
    }

    pub fn player_index(&self, id: PlayerId) -> Result<usize, ModelError> {
        self.players.iter().position(|p| p.id == id).ok_or(ModelError::UnknownPlayer(id))
    }

    pub fn player(&self, id: PlayerId) -> Result<&PlayerState, ModelError> {
        self.players.iter().find(|p| p.id == id).ok_or(ModelError::UnknownPlayer(id))
    }

    pub fn track_for_entry(&self, entry: EntryId) -> Option<&Track> {
        let e = self.playlists.entry(entry)?;
        self.library.get(e.track)
    }

    /// A request that starts the entry at its cue-in point.
    pub fn request_from_cue_in(&self, entry: EntryId) -> Option<SourceRequest> {
        let track = self.track_for_entry(entry)?;
        Some(SourceRequest { entry, track: track.id, path: track.path.clone(), from_secs: track.cue_in_secs() })
    }

    /// A request that starts the entry at `secs`, clamped to the file; a
    /// non-finite `secs` falls back to cue-in.
    pub fn request_at(&self, entry: EntryId, secs: f64) -> Option<SourceRequest> {
        let track = self.track_for_entry(entry)?;
        let from_secs =
            if secs.is_finite() { secs.clamp(0.0, track.duration_secs.max(0.0)) } else { track.cue_in_secs() };
        Some(SourceRequest { entry, track: track.id, path: track.path.clone(), from_secs })
    }

    /// True if `entry` is the current entry of any player.
    pub fn is_on_air(&self, entry: EntryId) -> bool {
        self.players.iter().any(|p| p.current == Some(entry))
    }
}
```

- [ ] **Step 6: Implement the reducer**

`crates/fp-model/src/reducer.rs`:
```rust
//! The player behaviour rules of spec §3, as pure state transitions.

use std::path::PathBuf;

use crate::command::{Command, EngineAction, EngineEvent, SourceRequest};
use crate::error::ModelError;
use crate::ids::{EntryId, PlayerId, PlaylistId};
use crate::player::Transport;
use crate::playlist::PlaylistEntry;
use crate::state::AppState;
use crate::track::Track;

/// Applies a user command. On `Err` the state is unchanged.
pub fn apply(state: &mut AppState, command: Command) -> Result<Vec<EngineAction>, ModelError> {
    let mut out = Vec::new();
    match command {
        Command::Play(id) => play(state, id, &mut out)?,
        Command::Pause(id) => pause(state, id, &mut out)?,
        Command::Stop(id) => stop(state, id, &mut out)?,
        Command::FadeStop(id) => fade_stop(state, id, &mut out)?,
        Command::SetNext(id, entry) => set_next(state, id, entry)?,
        Command::InsertPaths { playlist, index, paths } => insert_paths(state, playlist, index, paths)?,
    }
    reconcile(state, &mut out);
    Ok(out)
}

/// Applies an engine observation. Events about unknown players are ignored
/// (the player may have been removed while the event was in flight).
pub fn on_event(state: &mut AppState, event: EngineEvent) -> Vec<EngineAction> {
    let mut out = Vec::new();
    match event {
        EngineEvent::FadeCompleted { player } => {
            if let Ok(i) = state.player_index(player) {
                state.players[i].fading = false;
            }
        }
        EngineEvent::ReachedEnd { player } => {
            if let Ok(i) = state.player_index(player) {
                stop_player(state, i);
            }
        }
    }
    reconcile(state, &mut out);
    out
}

fn play(state: &mut AppState, id: PlayerId, out: &mut Vec<EngineAction>) -> Result<(), ModelError> {
    let i = state.player_index(id)?;
    match state.players[i].transport {
        Transport::Paused if state.players[i].current.is_some() => {
            state.players[i].transport = Transport::Playing;
            out.push(EngineAction::Resume { player: id });
        }
        Transport::Paused | Transport::Stopped => {
            if let Some(request) = advance(state, i) {
                out.push(EngineAction::StartCurrent { player: id, request });
            }
        }
        Transport::Playing => {
            if state.players[i].fading {
                return Ok(());
            }
            let fade_ms = state.config.players.fade_ms;
            if let Some(request) = advance(state, i) {
                state.players[i].fading = true;
                out.push(EngineAction::Crossfade { player: id, request, fade_ms });
            }
        }
    }
    Ok(())
}

fn pause(state: &mut AppState, id: PlayerId, out: &mut Vec<EngineAction>) -> Result<(), ModelError> {
    let i = state.player_index(id)?;
    let player = &mut state.players[i];
    match player.transport {
        Transport::Playing if !player.fading => {
            player.transport = Transport::Paused;
            out.push(EngineAction::Pause { player: id });
        }
        Transport::Paused => {
            player.transport = Transport::Playing;
            out.push(EngineAction::Resume { player: id });
        }
        _ => {}
    }
    Ok(())
}

fn stop(state: &mut AppState, id: PlayerId, out: &mut Vec<EngineAction>) -> Result<(), ModelError> {
    let i = state.player_index(id)?;
    let player = &state.players[i];
    if player.transport == Transport::Stopped && player.current.is_none() {
        return Ok(());
    }
    stop_player(state, i);
    out.push(EngineAction::StopNow { player: id });
    Ok(())
}

fn fade_stop(state: &mut AppState, id: PlayerId, out: &mut Vec<EngineAction>) -> Result<(), ModelError> {
    let i = state.player_index(id)?;
    let fade_ms = state.config.players.fade_ms;
    let player = &mut state.players[i];
    if player.transport == Transport::Playing && !player.fading {
        player.fading = true;
        out.push(EngineAction::FadeOutAndStop { player: id, fade_ms });
    }
    Ok(())
}

fn set_next(state: &mut AppState, id: PlayerId, entry: EntryId) -> Result<(), ModelError> {
    let i = state.player_index(id)?;
    if state.playlists.entry(entry).is_none() {
        return Err(ModelError::UnknownEntry(entry));
    }
    if state.players[i].current == Some(entry) {
        return Err(ModelError::NextIsCurrent);
    }
    state.players[i].next = Some(entry);
    Ok(())
}

fn insert_paths(
    state: &mut AppState,
    playlist: PlaylistId,
    index: usize,
    paths: Vec<PathBuf>,
) -> Result<(), ModelError> {
    if state.playlists.get(playlist).is_none() {
        return Err(ModelError::UnknownPlaylist(playlist));
    }
    let mut entries = Vec::with_capacity(paths.len());
    for path in paths {
        let track = state.ids.track();
        state.library.insert(Track::new(track, path));
        entries.push(PlaylistEntry { id: state.ids.entry(), track, played: false });
    }
    state.playlists.insert(playlist, index, entries)?;
    fill_empty_next(state);
    Ok(())
}

/// Rule 12: the next entry becomes current. Returns the request for the new
/// current source, or `None` (state untouched) when there is nothing to play.
pub(crate) fn advance(state: &mut AppState, i: usize) -> Option<SourceRequest> {
    let next = state.players[i].next?;
    let request = state.request_from_cue_in(next)?;
    if let Some(current) = state.players[i].current {
        state.playlists.mark_played(current);
    }
    let following = state.playlists.next_playable_after(next, &state.library);
    let player = &mut state.players[i];
    player.current = Some(next);
    player.next = following;
    player.transport = Transport::Playing;
    // The preloaded source is now the current one, and the old plan no longer applies.
    player.preloaded = None;
    player.scheduled = None;
    Some(request)
}

/// Rule 7: stop semantics shared by Stop, fade stop, stop-after-current and Single mode.
pub(crate) fn stop_player(state: &mut AppState, i: usize) {
    if let Some(current) = state.players[i].current {
        state.playlists.mark_played(current);
        if state.players[i].next.is_none() {
            state.players[i].next = state.playlists.next_playable_after(current, &state.library);
        }
    }
    let player = &mut state.players[i];
    player.current = None;
    player.transport = Transport::Stopped;
    player.fading = false;
    player.stop_after_current = false;
}

/// An idle player (stopped, nothing current, no next) picks the first playable
/// entry of the playlist it shows. Used after entries appear or disappear.
pub(crate) fn fill_empty_next(state: &mut AppState) {
    let picks: Vec<(usize, Option<EntryId>)> = state
        .players
        .iter()
        .enumerate()
        .filter(|(_, p)| p.transport == Transport::Stopped && p.current.is_none() && p.next.is_none())
        .map(|(i, p)| (i, state.playlists.first_playable(p.playlist, &state.library)))
        .collect();
    for (i, pick) in picks {
        state.players[i].next = pick;
    }
}

/// Derives the engine work implied by the state: preload whatever is next.
pub(crate) fn reconcile(state: &mut AppState, out: &mut Vec<EngineAction>) {
    let preloads: Vec<(usize, Option<EntryId>)> = state
        .players
        .iter()
        .enumerate()
        .filter(|(_, p)| p.preloaded != p.next)
        .map(|(i, p)| (i, p.next))
        .collect();
    for (i, wanted) in preloads {
        let request = wanted.and_then(|e| state.request_from_cue_in(e));
        let player = &mut state.players[i];
        player.preloaded = wanted;
        out.push(EngineAction::Preload { player: player.id, request });
    }
}
```

Update `lib.rs` so the module list and re-exports read:
```rust
pub mod command;
pub mod config;
pub mod error;
pub mod ids;
pub mod player;
pub mod playlist;
pub mod reducer;
pub mod state;
pub mod track;

pub use command::{Command, EngineAction, EngineEvent, SourceRequest, TransitionPlan};
pub use config::{
    AnalysisSettings, Config, ConfigWarning, Limits, OutputsConfig, PlayerRoutes, PlayersConfig, Route,
    Tuning, UiConfig,
};
pub use error::ModelError;
pub use ids::{EntryId, IdGen, PlayerId, PlaylistId, TrackId};
pub use player::{ColumnWidths, CueState, PlayMode, PlayerState, Transport};
pub use playlist::{Playlist, PlaylistEntry, Playlists};
pub use reducer::{apply, on_event};
pub use state::AppState;
pub use track::{FileState, Library, Marker, MarkerKind, MarkerSource, Markers, Track, TrackKind};
```

- [ ] **Step 7: Run the tests to verify they pass**

Run: `cargo test -p fp-model --test transport`
Expected: 13 tests PASS.

- [ ] **Step 8: Check and commit**

Run: `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
```bash
git add crates/fp-model
git commit -m "feat(model): add player state and transport rules

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 6: Transition planning, modes and source failures (spec §3, rules 9–12)

**Files:**
- Modify: `crates/fp-model/src/command.rs`, `crates/fp-model/src/reducer.rs`, `crates/fp-model/src/lib.rs`
- Create: `crates/fp-model/tests/transitions.rs`

**Interfaces:**
- Consumes: Task 5.
- Produces:
  - New `Command` variants: `SetMode(PlayerId, PlayMode)` and `ToggleStopAfterCurrent(PlayerId)`.
  - New `EngineEvent` variants: `TransitionStarted { player }` (the engine started the next source at the scheduled time) and `SourceFailed { player, entry }`.
  - New `EngineAction` variant: `Schedule { player, plan: Option<TransitionPlan> }` (`None` cancels).
  - Public `fp_model::plan_for(&AppState, &PlayerState) -> Option<TransitionPlan>`.
  - `reconcile` now also emits `Schedule` whenever a player's plan changes.

- [ ] **Step 1: Write the failing tests**

`crates/fp-model/tests/transitions.rs`:
```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Spec §3 rules 9–12 and decode failures.

mod common;

use common::{entries, fixture, p0};
use fp_model::{
    AppState, Command, EngineAction, EngineEvent, FileState, MarkerKind, ModelError, PlayMode, PlayerId,
    TransitionPlan, Transport, apply, on_event,
};

fn with_segue(state: &mut AppState, secs: f64) {
    for t in state.library.iter_mut() {
        t.markers.set_auto(MarkerKind::SegueStart, Some(secs));
    }
}

/// The last plan scheduled for `p` in `actions`, if any.
fn scheduled(actions: &[EngineAction], p: PlayerId) -> Option<Option<TransitionPlan>> {
    actions.iter().rev().find_map(|a| match a {
        EngineAction::Schedule { player, plan } if *player == p => Some(*plan),
        _ => None,
    })
}

#[test]
fn rule11_continuous_with_segue_overlaps_at_segue_start() {
    let mut state = fixture(3);
    with_segue(&mut state, 172.0);
    let p = p0(&state);
    let actions = apply(&mut state, Command::Play(p)).unwrap();
    assert_eq!(
        scheduled(&actions, p),
        Some(Some(TransitionPlan::StartNextAt { at_secs: 172.0, fade_current_until_secs: Some(180.0) }))
    );
}

#[test]
fn rule11_without_segue_the_next_starts_at_cue_out() {
    let mut state = fixture(3);
    let p = p0(&state);
    let actions = apply(&mut state, Command::Play(p)).unwrap();
    assert_eq!(
        scheduled(&actions, p),
        Some(Some(TransitionPlan::StartNextAt { at_secs: 180.0, fade_current_until_secs: None }))
    );
}

#[test]
fn rule11_auto_segue_disabled_ignores_the_segue_marker() {
    let mut state = fixture(3);
    with_segue(&mut state, 172.0);
    state.config.players.auto_segue = false;
    let p = p0(&state);
    let actions = apply(&mut state, Command::Play(p)).unwrap();
    assert_eq!(
        scheduled(&actions, p),
        Some(Some(TransitionPlan::StartNextAt { at_secs: 180.0, fade_current_until_secs: None }))
    );
}

#[test]
fn rule10_single_mode_stops_at_cue_out() {
    let mut state = fixture(3);
    with_segue(&mut state, 172.0);
    let p = p0(&state);
    apply(&mut state, Command::SetMode(p, PlayMode::Single)).unwrap();
    let actions = apply(&mut state, Command::Play(p)).unwrap();
    assert_eq!(scheduled(&actions, p), Some(Some(TransitionPlan::StopAt { at_secs: 180.0 })));
}

#[test]
fn rule11_last_entry_schedules_a_stop() {
    let mut state = fixture(1);
    let p = p0(&state);
    let actions = apply(&mut state, Command::Play(p)).unwrap();
    assert_eq!(scheduled(&actions, p), Some(Some(TransitionPlan::StopAt { at_secs: 180.0 })));
}

#[test]
fn rule9_stop_after_current_is_refused_in_single_mode() {
    let mut state = fixture(3);
    let p = p0(&state);
    apply(&mut state, Command::SetMode(p, PlayMode::Single)).unwrap();
    assert_eq!(apply(&mut state, Command::ToggleStopAfterCurrent(p)), Err(ModelError::StopAfterInSingle));
}

#[test]
fn rule9_switching_to_single_clears_stop_after_current() {
    let mut state = fixture(3);
    let p = p0(&state);
    apply(&mut state, Command::ToggleStopAfterCurrent(p)).unwrap();
    assert!(state.player(p).unwrap().stop_after_current);
    apply(&mut state, Command::SetMode(p, PlayMode::Single)).unwrap();
    assert!(!state.player(p).unwrap().stop_after_current);
}

#[test]
fn rule9_stop_after_current_reschedules_a_stop_and_keeps_next_green() {
    let mut state = fixture(3);
    with_segue(&mut state, 172.0);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::Play(p)).unwrap();
    let actions = apply(&mut state, Command::ToggleStopAfterCurrent(p)).unwrap();
    assert_eq!(scheduled(&actions, p), Some(Some(TransitionPlan::StopAt { at_secs: 180.0 })));
    // Rule 2: choosing another next keeps the flag.
    apply(&mut state, Command::SetNext(p, e[2])).unwrap();
    assert!(state.player(p).unwrap().stop_after_current);
    on_event(&mut state, EngineEvent::ReachedEnd { player: p });
    let player = state.player(p).unwrap();
    assert_eq!((player.current, player.next, player.stop_after_current), (None, Some(e[2]), false));
}

#[test]
fn rule11_transition_started_advances_marks_played_and_tracks_the_fade() {
    let mut state = fixture(3);
    with_segue(&mut state, 172.0);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::Play(p)).unwrap();
    let actions = on_event(&mut state, EngineEvent::TransitionStarted { player: p });
    let player = state.player(p).unwrap();
    assert_eq!((player.current, player.next, player.fading), (Some(e[1]), Some(e[2]), true));
    assert!(state.playlists.entry(e[0]).unwrap().played);
    assert!(scheduled(&actions, p).is_some(), "a plan for the new current must be sent");
    assert!(!actions.iter().any(|a| matches!(a, EngineAction::StartCurrent { .. })));
}

#[test]
fn rule11_transition_without_overlap_does_not_mark_fading() {
    let mut state = fixture(3);
    let p = p0(&state);
    apply(&mut state, Command::Play(p)).unwrap();
    on_event(&mut state, EngineEvent::TransitionStarted { player: p });
    assert!(!state.player(p).unwrap().fading);
}

#[test]
fn rule12_advance_skips_missing_entries() {
    let mut state = fixture(4);
    let (e, p) = (entries(&state), p0(&state));
    let missing = state.playlists.entry(e[1]).unwrap().track;
    state.library.get_mut(missing).unwrap().file_state = FileState::Missing;
    apply(&mut state, Command::Play(p)).unwrap();
    assert_eq!(state.player(p).unwrap().next, Some(e[2]));
}

#[test]
fn a_failing_current_source_skips_to_next_in_continuous_mode() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::Play(p)).unwrap();
    let actions = on_event(&mut state, EngineEvent::SourceFailed { player: p, entry: e[0] });
    assert!(matches!(actions.first(), Some(EngineAction::StartCurrent { request, .. }) if request.entry == e[1]));
    let failed = state.playlists.entry(e[0]).unwrap().track;
    assert_eq!(state.library.get(failed).unwrap().file_state, FileState::Unreadable);
}

#[test]
fn a_failing_current_source_stops_in_single_mode() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::SetMode(p, PlayMode::Single)).unwrap();
    apply(&mut state, Command::Play(p)).unwrap();
    let actions = on_event(&mut state, EngineEvent::SourceFailed { player: p, entry: e[0] });
    assert_eq!(actions.first(), Some(&EngineAction::StopNow { player: p }));
    assert_eq!(state.player(p).unwrap().transport, Transport::Stopped);
}

#[test]
fn a_failing_next_source_moves_next_forward_on_every_player() {
    let mut state = fixture(3);
    let e = entries(&state);
    let p = p0(&state);
    on_event(&mut state, EngineEvent::SourceFailed { player: p, entry: e[0] });
    assert!(state.players.iter().all(|pl| pl.next == Some(e[1])));
}

#[test]
fn playlist_of_unreadable_entries_never_starts() {
    let mut state = fixture(3);
    let e = entries(&state);
    for t in state.library.iter_mut() {
        t.file_state = FileState::Unreadable;
    }
    for pl in &mut state.players {
        pl.next = None;
    }
    let p = p0(&state);
    let actions = apply(&mut state, Command::Play(p)).unwrap();
    assert!(!actions.iter().any(|a| matches!(a, EngineAction::StartCurrent { .. })));
    assert_eq!(state.player(p).unwrap().transport, Transport::Stopped);
    assert_eq!(state.playlists.next_playable_after(e[0], &state.library), None);
    // Repeated failures terminate and leave the player stopped.
    for entry in &e {
        on_event(&mut state, EngineEvent::SourceFailed { player: p, entry: *entry });
    }
    assert_eq!(state.player(p).unwrap().transport, Transport::Stopped);
}

#[test]
fn stopping_cancels_the_schedule() {
    let mut state = fixture(3);
    let p = p0(&state);
    apply(&mut state, Command::Play(p)).unwrap();
    let actions = apply(&mut state, Command::Stop(p)).unwrap();
    assert_eq!(scheduled(&actions, p), Some(None));
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-model --test transitions`
Expected: FAIL to compile: `no variant SetMode`, `no variant TransitionStarted`.

- [ ] **Step 3: Extend the vocabulary**

In `command.rs`:

1. Add `use crate::player::PlayMode;` to the imports.
2. Add these variants to `Command`:
```rust
    SetMode(PlayerId, PlayMode),
    ToggleStopAfterCurrent(PlayerId),
```
3. Add these variants to `EngineEvent`:
```rust
    /// The engine started the next source as scheduled by a `StartNextAt` plan.
    TransitionStarted { player: PlayerId },
    /// A source could not be decoded or read.
    SourceFailed { player: PlayerId, entry: EntryId },
```
4. Add this variant to `EngineAction`:
```rust
    /// Replace the transition plan of the current source (`None` cancels it).
    Schedule { player: PlayerId, plan: Option<TransitionPlan> },
```

- [ ] **Step 4: Implement the rules**

In `reducer.rs`:

1. Add the imports `use crate::command::TransitionPlan;`, `use crate::player::{PlayMode, PlayerState};` and `use crate::track::FileState;` (merged into the existing `use` lines).
2. Add these arms to the `match command` in `apply`:
```rust
        Command::SetMode(id, mode) => {
            let i = state.player_index(id)?;
            let player = &mut state.players[i];
            player.mode = mode;
            if mode == PlayMode::Single {
                player.stop_after_current = false;
            }
        }
        Command::ToggleStopAfterCurrent(id) => {
            let i = state.player_index(id)?;
            let player = &mut state.players[i];
            if player.mode == PlayMode::Single {
                return Err(ModelError::StopAfterInSingle);
            }
            player.stop_after_current = !player.stop_after_current;
        }
```
3. Add these arms to the `match event` in `on_event`:
```rust
        EngineEvent::TransitionStarted { player } => {
            if let Ok(i) = state.player_index(player) {
                let overlapping = matches!(
                    state.players[i].scheduled,
                    Some(TransitionPlan::StartNextAt { fade_current_until_secs: Some(_), .. })
                );
                if advance(state, i).is_some() {
                    state.players[i].fading = overlapping;
                } else {
                    stop_player(state, i);
                    out.push(EngineAction::StopNow { player });
                }
            }
        }
        EngineEvent::SourceFailed { player, entry } => source_failed(state, player, entry, &mut out),
```
4. Add these functions:
```rust
/// Spec §4.5: mark the file unreadable, then skip (Continuous) or stop.
fn source_failed(state: &mut AppState, player: PlayerId, entry: EntryId, out: &mut Vec<EngineAction>) {
    if let Some(t) = state.playlists.entry(entry).map(|e| e.track).and_then(|track| state.library.get_mut(track)) {
        t.file_state = FileState::Unreadable;
    }
    if let Ok(i) = state.player_index(player)
        && state.players[i].current == Some(entry)
    {
        let keep_going = state.players[i].mode == PlayMode::Continuous && !state.players[i].stop_after_current;
        let restarted = if keep_going { advance(state, i) } else { None };
        match restarted {
            Some(request) => out.push(EngineAction::StartCurrent { player, request }),
            None => {
                stop_player(state, i);
                out.push(EngineAction::StopNow { player });
            }
        }
    }
    let replacement = state.playlists.next_playable_after(entry, &state.library);
    for p in &mut state.players {
        if p.next == Some(entry) {
            p.next = replacement;
        }
    }
}

/// Rules 9–11: what the engine must do when the current track ends.
pub fn plan_for(state: &AppState, player: &PlayerState) -> Option<TransitionPlan> {
    if player.transport == Transport::Stopped {
        return None;
    }
    let track = state.track_for_entry(player.current?)?;
    let cue_out = track.cue_out_secs();
    if player.mode == PlayMode::Single || player.stop_after_current || player.next.is_none() {
        return Some(TransitionPlan::StopAt { at_secs: cue_out });
    }
    match track.segue_start_secs() {
        Some(segue) if state.config.players.auto_segue => {
            Some(TransitionPlan::StartNextAt { at_secs: segue, fade_current_until_secs: Some(cue_out) })
        }
        _ => Some(TransitionPlan::StartNextAt { at_secs: cue_out, fade_current_until_secs: None }),
    }
}
```
5. Append this schedule pass to the end of `reconcile`:
```rust
    let plans: Vec<(usize, Option<TransitionPlan>)> = state
        .players
        .iter()
        .enumerate()
        .map(|(i, p)| (i, plan_for(state, p)))
        .filter(|(i, plan)| state.players[*i].scheduled != *plan)
        .collect();
    for (i, plan) in plans {
        let player = &mut state.players[i];
        player.scheduled = plan;
        out.push(EngineAction::Schedule { player: player.id, plan });
    }
```
6. In `lib.rs`, change the reducer re-export to `pub use reducer::{apply, on_event, plan_for};`.

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p fp-model`
Expected: all tests PASS, including the 13 in `transport.rs` and the 16 in `transitions.rs`.

- [ ] **Step 6: Check and commit**

Run: `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
```bash
git add crates/fp-model
git commit -m "feat(model): add transition planning, play modes and failure handling

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 7: Editing, cue, player count and settings commands (spec §3, rules 13–15 and 21)

**Files:**
- Modify: `crates/fp-model/src/command.rs`, `crates/fp-model/src/reducer.rs`
- Create: `crates/fp-model/tests/editing.rs`

**Interfaces:**
- Consumes: Tasks 5–6.
- Produces:
  - New `Command` variants:
    - cue: `ToggleCue(PlayerId)`, `CueEntry(PlayerId, EntryId)`;
    - per-player settings: `SetVolume(PlayerId, f32)`, `Seek(PlayerId, f64)`, `ShowPlaylist(PlayerId, PlaylistId)`, `SetColumnWidths(PlayerId, ColumnWidths)`;
    - entry editing: `RemoveEntry(EntryId)`, `MoveEntry { entry, to: PlaylistId, index: usize }`, `DuplicateEntry(EntryId)`;
    - playlists: `CreatePlaylist { name: String }`, `RenamePlaylist { playlist, name: String }`, `DeletePlaylist(PlaylistId)`;
    - app-wide: `SetPlayerCount(usize)`, `UpdateConfig(Box<Config>)`.
  - New `EngineEvent` variant: `CueEnded { player }`.
  - New `EngineAction` variants:
    - `AddPlayer { player }`, `RemovePlayer { player }`;
    - `StartCue { player, request }`, `StopCue { player }`;
    - `SetVolume { player, volume: f32 }`, `Seek { player, secs: f64 }`.
  - `UpdateConfig` never changes the player count; only `SetPlayerCount` does. The caller validates the config before sending it.
  - Removing entries or playlists also drops tracks that no playlist references any more.

- [ ] **Step 1: Write the failing tests**

`crates/fp-model/tests/editing.rs`:
```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Spec §3 rules 13–15 and 21, playlist management, volume, seek and config.

mod common;

use common::{entries, fixture, p0};
use fp_model::{
    ColumnWidths, Command, Config, EngineAction, EngineEvent, ModelError, PlaylistId, Transport, apply, on_event,
};

#[test]
fn rule13_an_entry_on_air_cannot_be_removed() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::Play(p)).unwrap();
    assert_eq!(apply(&mut state, Command::RemoveEntry(e[0])), Err(ModelError::EntryOnAir(e[0])));
    assert_eq!(entries(&state).len(), 3);
}

#[test]
fn rule13_removing_the_next_entry_moves_next_to_the_following_one() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::Play(p)).unwrap();
    apply(&mut state, Command::RemoveEntry(e[1])).unwrap();
    assert_eq!(state.player(p).unwrap().next, Some(e[2]));
}

#[test]
fn removing_the_last_reference_drops_the_track_from_the_library() {
    let mut state = fixture(2);
    let e = entries(&state);
    apply(&mut state, Command::DuplicateEntry(e[1])).unwrap();
    assert_eq!(state.library.len(), 2);
    let copy = entries(&state)[2];
    apply(&mut state, Command::RemoveEntry(copy)).unwrap();
    assert_eq!(state.library.len(), 2, "the original still references the track");
    apply(&mut state, Command::RemoveEntry(e[1])).unwrap();
    assert_eq!(state.library.len(), 1);
}

#[test]
fn rule14_switching_tabs_keeps_current_and_next() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::CreatePlaylist { name: "Other".into() }).unwrap();
    let other = state.playlists.iter().nth(1).unwrap().id;
    apply(&mut state, Command::Play(p)).unwrap();
    apply(&mut state, Command::ShowPlaylist(p, other)).unwrap();
    let player = state.player(p).unwrap();
    assert_eq!((player.playlist, player.current, player.next), (other, Some(e[0]), Some(e[1])));
}

#[test]
fn rule15_cue_toggles_prelisten_of_next_and_ends_on_event() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    let actions = apply(&mut state, Command::ToggleCue(p)).unwrap();
    assert!(matches!(actions.first(), Some(EngineAction::StartCue { request, .. }) if request.entry == e[0]));
    assert_eq!(state.player(p).unwrap().cue.map(|c| c.entry), Some(e[0]));
    assert_eq!(apply(&mut state, Command::ToggleCue(p)).unwrap(), vec![EngineAction::StopCue { player: p }]);
    apply(&mut state, Command::CueEntry(p, e[2])).unwrap();
    on_event(&mut state, EngineEvent::CueEnded { player: p });
    assert_eq!(state.player(p).unwrap().cue, None);
}

#[test]
fn removing_a_cued_entry_stops_the_cue() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::CueEntry(p, e[2])).unwrap();
    let actions = apply(&mut state, Command::RemoveEntry(e[2])).unwrap();
    assert!(actions.contains(&EngineAction::StopCue { player: p }));
}

#[test]
fn move_and_duplicate_entries() {
    let mut state = fixture(3);
    let e = entries(&state);
    apply(&mut state, Command::MoveEntry { entry: e[0], to: state.playlists.first_id().unwrap(), index: 3 })
        .unwrap();
    assert_eq!(entries(&state), vec![e[1], e[2], e[0]]);
    apply(&mut state, Command::DuplicateEntry(e[1])).unwrap();
    assert_eq!(entries(&state).len(), 4);
}

#[test]
fn playlists_can_be_created_renamed_and_deleted() {
    let mut state = fixture(2);
    apply(&mut state, Command::CreatePlaylist { name: "Night".into() }).unwrap();
    let night = state.playlists.iter().nth(1).unwrap().id;
    apply(&mut state, Command::RenamePlaylist { playlist: night, name: "Late".into() }).unwrap();
    assert_eq!(state.playlists.get(night).unwrap().name, "Late");
    let p = p0(&state);
    apply(&mut state, Command::ShowPlaylist(p, night)).unwrap();
    apply(&mut state, Command::DeletePlaylist(night)).unwrap();
    assert_eq!(state.player(p).unwrap().playlist, state.playlists.first_id().unwrap());
}

#[test]
fn a_playlist_on_air_or_the_last_one_cannot_be_deleted() {
    let mut state = fixture(2);
    let first = state.playlists.first_id().unwrap();
    assert_eq!(apply(&mut state, Command::DeletePlaylist(first)), Err(ModelError::LastPlaylist));
    apply(&mut state, Command::CreatePlaylist { name: "Spare".into() }).unwrap();
    let p = p0(&state);
    apply(&mut state, Command::Play(p)).unwrap();
    assert_eq!(apply(&mut state, Command::DeletePlaylist(first)), Err(ModelError::PlaylistOnAir(first)));
    assert_eq!(
        apply(&mut state, Command::DeletePlaylist(PlaylistId(424242))),
        Err(ModelError::UnknownPlaylist(PlaylistId(424242)))
    );
}

#[test]
fn rule21_player_count_grows_without_a_fixed_maximum() {
    let mut state = fixture(2);
    let e = entries(&state);
    let actions = apply(&mut state, Command::SetPlayerCount(8)).unwrap();
    assert_eq!(state.players.len(), 8);
    assert_eq!(state.config.players.count, 8);
    assert_eq!(actions.iter().filter(|a| matches!(a, EngineAction::AddPlayer { .. })).count(), 4);
    assert!(state.players.iter().all(|p| p.next == Some(e[0])));
}

#[test]
fn rule21_player_count_respects_the_resource_limit_and_busy_players() {
    let mut state = fixture(2);
    let max = state.config.limits.max_players;
    assert_eq!(
        apply(&mut state, Command::SetPlayerCount(max + 1)),
        Err(ModelError::PlayerCountOutOfRange { requested: max + 1, max })
    );
    assert!(matches!(apply(&mut state, Command::SetPlayerCount(0)), Err(ModelError::PlayerCountOutOfRange { .. })));
    let last = state.players[3].id;
    apply(&mut state, Command::Play(last)).unwrap();
    assert_eq!(apply(&mut state, Command::SetPlayerCount(2)), Err(ModelError::PlayerBusy(last)));
    apply(&mut state, Command::Stop(last)).unwrap();
    let actions = apply(&mut state, Command::SetPlayerCount(2)).unwrap();
    assert!(actions.contains(&EngineAction::RemovePlayer { player: last }));
    assert_eq!(state.players.len(), 2);
}

#[test]
fn volume_is_clamped_and_nan_is_silence() {
    let mut state = fixture(1);
    let p = p0(&state);
    apply(&mut state, Command::SetVolume(p, 1.7)).unwrap();
    assert_eq!(state.player(p).unwrap().volume, 1.0);
    let actions = apply(&mut state, Command::SetVolume(p, f32::NAN)).unwrap();
    assert_eq!(actions, vec![EngineAction::SetVolume { player: p, volume: 0.0 }]);
}

#[test]
fn seek_is_clamped_and_ignored_when_idle() {
    let mut state = fixture(1);
    let p = p0(&state);
    assert!(apply(&mut state, Command::Seek(p, 10.0)).unwrap().is_empty());
    apply(&mut state, Command::Play(p)).unwrap();
    let actions = apply(&mut state, Command::Seek(p, 999.0)).unwrap();
    assert_eq!(actions, vec![EngineAction::Seek { player: p, secs: 180.0 }]);
}

#[test]
fn update_config_keeps_the_player_count() {
    let mut state = fixture(1);
    let mut config = Config::default();
    config.players.count = 9;
    config.players.fade_ms = 2500;
    apply(&mut state, Command::UpdateConfig(Box::new(config))).unwrap();
    assert_eq!(state.config.players.fade_ms, 2500);
    assert_eq!(state.config.players.count, 4);
    assert_eq!(state.players.len(), 4);
}

#[test]
fn column_widths_are_stored_per_player() {
    let mut state = fixture(1);
    let p = p0(&state);
    let widths = ColumnWidths { number: Some(40.0), title: Some(200.0), duration: 60.0 };
    apply(&mut state, Command::SetColumnWidths(p, widths)).unwrap();
    assert_eq!(state.player(p).unwrap().columns, widths);
    assert_eq!(state.players[1].columns, ColumnWidths::default());
    assert_eq!(state.player(p).unwrap().transport, Transport::Stopped);
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-model --test editing`
Expected: FAIL to compile: `no variant RemoveEntry`, …

- [ ] **Step 3: Extend the vocabulary**

In `command.rs`:

1. Add the imports `use crate::config::Config;` and `use crate::player::ColumnWidths;`.
2. Add these variants to `Command`:
```rust
    ToggleCue(PlayerId),
    CueEntry(PlayerId, EntryId),
    SetVolume(PlayerId, f32),
    Seek(PlayerId, f64),
    ShowPlaylist(PlayerId, PlaylistId),
    SetColumnWidths(PlayerId, ColumnWidths),
    RemoveEntry(EntryId),
    MoveEntry { entry: EntryId, to: PlaylistId, index: usize },
    DuplicateEntry(EntryId),
    CreatePlaylist { name: String },
    RenamePlaylist { playlist: PlaylistId, name: String },
    DeletePlaylist(PlaylistId),
    SetPlayerCount(usize),
    /// Replaces the configuration (already validated by the caller); the
    /// player count is kept, use `SetPlayerCount` to change it.
    UpdateConfig(Box<Config>),
```
3. Add this variant to `EngineEvent`:
```rust
    /// The cue source reached its end.
    CueEnded { player: PlayerId },
```
4. Add these variants to `EngineAction`:
```rust
    AddPlayer { player: PlayerId },
    RemovePlayer { player: PlayerId },
    StartCue { player: PlayerId, request: SourceRequest },
    StopCue { player: PlayerId },
    SetVolume { player: PlayerId, volume: f32 },
    Seek { player: PlayerId, secs: f64 },
```

- [ ] **Step 4: Implement**

In `reducer.rs`:

1. Add `use std::collections::HashSet;`, `use crate::config::Config;` and `use crate::player::CueState;`, plus `PlayerState` if it is not already imported. Also add `use crate::playlist::Playlist;` and `use crate::ids::TrackId;` (merged into the existing lines).
2. Add these arms to the `match command` in `apply`:
```rust
        Command::ToggleCue(id) => toggle_cue(state, id, &mut out)?,
        Command::CueEntry(id, entry) => cue_entry(state, id, entry, &mut out)?,
        Command::SetVolume(id, volume) => {
            let i = state.player_index(id)?;
            let volume = if volume.is_nan() { 0.0 } else { volume.clamp(0.0, 1.0) };
            state.players[i].volume = volume;
            out.push(EngineAction::SetVolume { player: id, volume });
        }
        Command::Seek(id, secs) => {
            let i = state.player_index(id)?;
            let player = &state.players[i];
            if player.transport != Transport::Stopped
                && let Some(request) = player.current.and_then(|c| state.request_at(c, secs))
            {
                out.push(EngineAction::Seek { player: id, secs: request.from_secs });
            }
        }
        Command::ShowPlaylist(id, playlist) => {
            let i = state.player_index(id)?;
            if state.playlists.get(playlist).is_none() {
                return Err(ModelError::UnknownPlaylist(playlist));
            }
            state.players[i].playlist = playlist;
        }
        Command::SetColumnWidths(id, widths) => {
            let i = state.player_index(id)?;
            state.players[i].columns = widths;
        }
        Command::RemoveEntry(entry) => remove_entry(state, entry, &mut out)?,
        Command::MoveEntry { entry, to, index } => state.playlists.move_entry(entry, to, index)?,
        Command::DuplicateEntry(entry) => {
            let new_id = state.ids.entry();
            state.playlists.duplicate(entry, new_id)?;
        }
        Command::CreatePlaylist { name } => {
            let id = state.ids.playlist();
            state.playlists.add(Playlist::new(id, name));
        }
        Command::RenamePlaylist { playlist, name } => state.playlists.rename(playlist, name)?,
        Command::DeletePlaylist(playlist) => delete_playlist(state, playlist, &mut out)?,
        Command::SetPlayerCount(count) => set_player_count(state, count, &mut out)?,
        Command::UpdateConfig(config) => {
            let mut config: Config = *config;
            config.players.count = state.config.players.count;
            state.config = config;
        }
```
3. Add this arm to the `match event` in `on_event`:
```rust
        EngineEvent::CueEnded { player } => {
            if let Ok(i) = state.player_index(player) {
                state.players[i].cue = None;
            }
        }
```
4. Add these functions:
```rust
fn toggle_cue(state: &mut AppState, id: PlayerId, out: &mut Vec<EngineAction>) -> Result<(), ModelError> {
    let i = state.player_index(id)?;
    if state.players[i].cue.take().is_some() {
        out.push(EngineAction::StopCue { player: id });
    } else if let Some(entry) = state.players[i].next {
        cue_entry(state, id, entry, out)?;
    }
    Ok(())
}

fn cue_entry(
    state: &mut AppState,
    id: PlayerId,
    entry: EntryId,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    let i = state.player_index(id)?;
    let request = state.request_from_cue_in(entry).ok_or(ModelError::UnknownEntry(entry))?;
    state.players[i].cue = Some(CueState { entry });
    out.push(EngineAction::StartCue { player: id, request });
    Ok(())
}

/// Stops cues that point at any of `entries`, and clears next pointers into them
/// (replacing them with `replacement(entry)`).
fn detach_entries(
    state: &mut AppState,
    entries: &HashSet<EntryId>,
    replacement: impl Fn(EntryId) -> Option<EntryId>,
    out: &mut Vec<EngineAction>,
) {
    for p in &mut state.players {
        if let Some(next) = p.next.filter(|n| entries.contains(n)) {
            p.next = replacement(next);
        }
        if p.cue.is_some_and(|c| entries.contains(&c.entry)) {
            p.cue = None;
            out.push(EngineAction::StopCue { player: p.id });
        }
    }
}

/// Drops library tracks that no playlist entry references any more.
fn forget_unreferenced(state: &mut AppState, tracks: impl IntoIterator<Item = TrackId>) {
    for track in tracks {
        if !state.playlists.references_track(track) {
            state.library.remove(track);
        }
    }
}

fn remove_entry(state: &mut AppState, entry: EntryId, out: &mut Vec<EngineAction>) -> Result<(), ModelError> {
    if state.is_on_air(entry) {
        return Err(ModelError::EntryOnAir(entry));
    }
    let after = state.playlists.next_playable_after(entry, &state.library);
    let removed = state.playlists.remove_entry(entry)?;
    detach_entries(state, &HashSet::from([entry]), |_| after, out);
    forget_unreferenced(state, [removed.track]);
    fill_empty_next(state);
    Ok(())
}

fn delete_playlist(
    state: &mut AppState,
    playlist: PlaylistId,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    let list = state.playlists.get(playlist).ok_or(ModelError::UnknownPlaylist(playlist))?;
    if list.entries.iter().any(|e| state.is_on_air(e.id)) {
        return Err(ModelError::PlaylistOnAir(playlist));
    }
    let removed = state.playlists.remove(playlist)?;
    let first = state.playlists.first_id().ok_or(ModelError::NoPlaylists)?;
    for p in &mut state.players {
        if p.playlist == playlist {
            p.playlist = first;
        }
    }
    let ids: HashSet<EntryId> = removed.entries.iter().map(|e| e.id).collect();
    detach_entries(state, &ids, |_| None, out);
    forget_unreferenced(state, removed.entries.iter().map(|e| e.track));
    fill_empty_next(state);
    Ok(())
}

fn set_player_count(state: &mut AppState, count: usize, out: &mut Vec<EngineAction>) -> Result<(), ModelError> {
    let max = state.config.limits.max_players;
    if count == 0 || count > max {
        return Err(ModelError::PlayerCountOutOfRange { requested: count, max });
    }
    if count < state.players.len() {
        if let Some(busy) = state.players[count..].iter().find(|p| p.transport != Transport::Stopped || p.cue.is_some()) {
            return Err(ModelError::PlayerBusy(busy.id));
        }
        for p in state.players.drain(count..) {
            out.push(EngineAction::RemovePlayer { player: p.id });
        }
    } else {
        let first = state.playlists.first_id().ok_or(ModelError::NoPlaylists)?;
        let mode = state.config.players.default_mode;
        while state.players.len() < count {
            let id = state.ids.player();
            state.players.push(PlayerState::new(id, first, mode));
            out.push(EngineAction::AddPlayer { player: id });
        }
        fill_empty_next(state);
    }
    state.config.players.count = count;
    Ok(())
}
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p fp-model`
Expected: all tests PASS (15 new in `editing.rs`).

- [ ] **Step 6: Check and commit**

Run: `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
```bash
git add crates/fp-model
git commit -m "feat(model): add editing, cue, player count and config commands

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 8: Session snapshot and crash-recovery restore (spec §7)

**Files:**
- Create: `crates/fp-model/src/session.rs`, `crates/fp-model/tests/session.rs`
- Modify: `crates/fp-model/src/command.rs`, `crates/fp-model/src/lib.rs`

**Interfaces:**
- Consumes: Tasks 1–7.
- Produces:
  - `PlayerSession { id, playlist, current: Option<EntryId>, next: Option<EntryId>, mode, stop_after_current: bool, position_secs: f64, volume: f32, columns: ColumnWidths }`, with serde and `#[serde(default)]` on optional-feeling fields.
  - `RestoreParts { config: Config, library: Library, playlists: Playlists, ids: IdGen }`.
  - `AppState::sessions(&self, position_of: impl Fn(PlayerId) -> f64) -> Vec<PlayerSession>`.
  - `AppState::restore(parts: RestoreParts, sessions: &[PlayerSession], default_playlist_name: &str) -> (AppState, Vec<EngineAction>)`, with this behaviour:
    - players come up `Paused` if they had a current entry and `Stopped` otherwise, and never `Playing`;
    - dangling references are dropped;
    - ids are made unique;
    - the id generator is moved past every id it sees.
  - New `EngineAction` variant: `LoadPaused { player, request }` (open the source at a position, paused).

- [ ] **Step 1: Write the failing tests**

`crates/fp-model/tests/session.rs`:
```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Crash recovery: restoring players from the last session snapshot.

mod common;

use common::{entries, fixture, p0};
use fp_model::{
    AppState, Command, Config, EngineAction, EntryId, IdGen, Library, PlayMode, PlayerSession, PlaylistId,
    Playlists, RestoreParts, Transport, apply,
};

fn parts(state: &AppState) -> RestoreParts {
    RestoreParts {
        config: state.config.clone(),
        library: state.library.clone(),
        playlists: state.playlists.clone(),
        ids: state.ids.clone(),
    }
}

#[test]
fn a_playing_player_comes_back_paused_at_its_position() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::Play(p)).unwrap();
    let sessions = state.sessions(|_| 42.0);
    let (restored, actions) = AppState::restore(parts(&state), &sessions, "Main");
    let player = restored.player(p).unwrap();
    assert_eq!((player.current, player.next, player.transport), (Some(e[0]), Some(e[1]), Transport::Paused));
    assert!(actions.iter().any(|a| matches!(
        a,
        EngineAction::LoadPaused { player, request } if *player == p && request.entry == e[0] && request.from_secs == 42.0
    )));
    assert!(restored.players.iter().all(|pl| pl.transport != Transport::Playing));
}

#[test]
fn dangling_session_references_are_dropped() {
    let state = fixture(3);
    let e = entries(&state);
    let session = PlayerSession {
        id: state.players[0].id,
        playlist: PlaylistId(999_999),
        current: Some(EntryId(888_888)),
        next: Some(EntryId(777_777)),
        mode: PlayMode::Single,
        stop_after_current: true,
        position_secs: f64::NAN,
        volume: 7.0,
        columns: Default::default(),
    };
    let (restored, actions) = AppState::restore(parts(&state), &[session], "Main");
    let player = &restored.players[0];
    assert_eq!(player.playlist, restored.playlists.first_id().unwrap());
    assert_eq!(player.current, None);
    assert_eq!(player.next, Some(e[0]), "an idle player picks the first playable entry");
    assert_eq!(player.transport, Transport::Stopped);
    assert!(!player.stop_after_current, "stop-after is meaningless in Single mode");
    assert_eq!(player.volume, 1.0);
    assert!(!actions.iter().any(|a| matches!(a, EngineAction::LoadPaused { .. })));
}

#[test]
fn restoring_with_no_playlists_creates_a_default_one_and_moves_ids_forward() {
    let parts = RestoreParts {
        config: Config::default(),
        library: Library::default(),
        playlists: Playlists::default(),
        ids: IdGen::default(),
    };
    let duplicate = PlayerSession {
        id: fp_model::PlayerId(500),
        playlist: PlaylistId(1),
        current: None,
        next: None,
        mode: PlayMode::Continuous,
        stop_after_current: false,
        position_secs: 0.0,
        volume: 0.5,
        columns: Default::default(),
    };
    let (mut restored, _) = AppState::restore(parts, &[duplicate.clone(), duplicate], "Main");
    assert_eq!(restored.playlists.len(), 1);
    assert_eq!(restored.players.len(), 4);
    assert_eq!(restored.players[0].id, fp_model::PlayerId(500));
    assert_ne!(restored.players[1].id, fp_model::PlayerId(500), "duplicate ids get a fresh id");
    assert_eq!(restored.players[0].volume, 0.5);
    assert!(restored.ids.next_raw() > 500);
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-model --test session`
Expected: FAIL to compile: `cannot find type PlayerSession`.

- [ ] **Step 3: Implement**

Add to `EngineAction` in `command.rs`:
```rust
    /// Open `request` at its position and hold it paused (crash recovery).
    LoadPaused { player: PlayerId, request: SourceRequest },
```

`crates/fp-model/src/session.rs`:
```rust
//! Per-player session snapshot (for `session.json`) and crash-recovery restore.

use serde::{Deserialize, Serialize};

use crate::command::EngineAction;
use crate::config::Config;
use crate::ids::{EntryId, IdGen, PlayerId, PlaylistId};
use crate::player::{ColumnWidths, PlayMode, PlayerState, Transport};
use crate::playlist::{Playlist, Playlists};
use crate::reducer::{fill_empty_next, reconcile};
use crate::state::AppState;
use crate::track::Library;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerSession {
    pub id: PlayerId,
    pub playlist: PlaylistId,
    #[serde(default)]
    pub current: Option<EntryId>,
    #[serde(default)]
    pub next: Option<EntryId>,
    #[serde(default)]
    pub mode: PlayMode,
    #[serde(default)]
    pub stop_after_current: bool,
    #[serde(default)]
    pub position_secs: f64,
    #[serde(default = "full_volume")]
    pub volume: f32,
    #[serde(default)]
    pub columns: ColumnWidths,
}

fn full_volume() -> f32 {
    1.0
}

/// Everything loaded from disk except the per-player sessions.
#[derive(Debug, Clone, Default)]
pub struct RestoreParts {
    pub config: Config,
    pub library: Library,
    pub playlists: Playlists,
    pub ids: IdGen,
}

impl AppState {
    pub fn sessions(&self, position_of: impl Fn(PlayerId) -> f64) -> Vec<PlayerSession> {
        self.players
            .iter()
            .map(|p| PlayerSession {
                id: p.id,
                playlist: p.playlist,
                current: p.current,
                next: p.next,
                mode: p.mode,
                stop_after_current: p.stop_after_current,
                position_secs: if p.current.is_some() { position_of(p.id) } else { 0.0 },
                volume: p.volume,
                columns: p.columns,
            })
            .collect()
    }

    /// Rebuilds the state after a start or a crash. Nothing is ever restored
    /// as playing: nothing goes on air by itself (spec §7).
    pub fn restore(
        parts: RestoreParts,
        sessions: &[PlayerSession],
        default_playlist_name: &str,
    ) -> (AppState, Vec<EngineAction>) {
        let RestoreParts { config, library, mut playlists, mut ids } = parts;
        ids.observe(library.max_raw_id());
        ids.observe(playlists.max_raw_id());
        for s in sessions {
            ids.observe(s.id.0);
        }
        if playlists.is_empty() {
            playlists.add(Playlist::new(ids.playlist(), default_playlist_name));
        }
        let mut state = AppState { config, library, playlists, players: Vec::new(), ids };
        let mut out = Vec::new();
        for k in 0..state.config.players.count {
            let player = match sessions.get(k) {
                Some(session) => restore_player(&mut state, session),
                None => new_player(&mut state),
            };
            state.players.push(player);
        }
        for p in &state.players {
            if let Some(current) = p.current {
                let position = sessions.iter().find(|s| s.id == p.id).map_or(0.0, |s| s.position_secs);
                if let Some(request) = state.request_at(current, position) {
                    out.push(EngineAction::LoadPaused { player: p.id, request });
                }
            }
        }
        fill_empty_next(&mut state);
        reconcile(&mut state, &mut out);
        (state, out)
    }
}

fn first_playlist(state: &AppState) -> PlaylistId {
    // `restore` guarantees at least one playlist.
    state.playlists.first_id().unwrap_or(PlaylistId(0))
}

fn new_player(state: &mut AppState) -> PlayerState {
    let id = state.ids.player();
    PlayerState::new(id, first_playlist(state), state.config.players.default_mode)
}

fn restore_player(state: &mut AppState, s: &PlayerSession) -> PlayerState {
    let id = if state.players.iter().any(|p| p.id == s.id) { state.ids.player() } else { s.id };
    let playlist = if state.playlists.get(s.playlist).is_some() { s.playlist } else { first_playlist(state) };
    let current = s.current.filter(|e| state.playlists.entry(*e).is_some());
    let next = s
        .next
        .filter(|e| state.playlists.entry(*e).is_some() && Some(*e) != current)
        .or_else(|| current.and_then(|c| state.playlists.next_playable_after(c, &state.library)));
    let volume = if s.volume.is_finite() && (0.0..=1.0).contains(&s.volume) { s.volume } else { 1.0 };
    let mut player = PlayerState::new(id, playlist, s.mode);
    player.current = current;
    player.next = next;
    player.transport = if current.is_some() { Transport::Paused } else { Transport::Stopped };
    player.stop_after_current = s.mode == PlayMode::Continuous && s.stop_after_current;
    player.volume = volume;
    player.columns = s.columns;
    player
}
```

Add to `lib.rs`:
```rust
pub mod session;

pub use session::{PlayerSession, RestoreParts};
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-model`
Expected: all tests PASS (3 new).

- [ ] **Step 5: Check and commit**

Run: `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
```bash
git add crates/fp-model
git commit -m "feat(model): add session snapshot and crash-recovery restore

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 9: `fp-store` paths and crash-safe file I/O

**Files:**
- Create: `crates/fp-store/Cargo.toml`, `crates/fp-store/src/lib.rs`, `crates/fp-store/src/error.rs`, `crates/fp-store/src/paths.rs`, `crates/fp-store/src/atomic.rs`

**Interfaces:**
- Consumes: nothing from `fp-model` yet.
- Produces:
  - `AppPaths { config_dir, data_dir, cache_dir, log_dir: PathBuf }`:
    - constructors `AppPaths::system() -> Option<AppPaths>` and `AppPaths::under(root: &Path) -> AppPaths`;
    - `config_file()`, `playlists_file()`, `session_file()`.
  - `StoreError { Io(#[from] io::Error), Serialize(String) }`.
  - `write_atomic(path: &Path, bytes: &[u8], backups: usize) -> io::Result<()>`.
  - `backup_path(path: &Path, n: usize) -> PathBuf`, which gives `file.json.bakN`.
  - `ParseError { Corrupt(String), TooNew(String) }`.
  - `LoadSource { Primary, Backup(usize), Defaults }`.
  - `Loaded<T> { value: Option<T>, source: LoadSource, warnings: Vec<String> }`.
  - `load_with_fallback<T>(path, backups, max_bytes: u64, parse: impl Fn(&[u8]) -> Result<T, ParseError>) -> Loaded<T>`, which behaves like this:
    - a corrupt candidate is renamed `*.corrupt-<unix secs>`;
    - a `TooNew` candidate is left in place;
    - a missing primary on first run gives `Defaults` with no warnings.

- [ ] **Step 1: Create the crate manifest**

`crates/fp-store/Cargo.toml`:
```toml
[package]
name = "fp-store"
description = "Crash-safe persistence of configuration, playlists and session"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
publish.workspace = true

[dependencies]
fp-model.workspace = true
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
directories.workspace = true

[dev-dependencies]
tempfile.workspace = true

[lints]
workspace = true
```

`crates/fp-store/src/error.rs`:
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    #[error("could not serialise: {0}")]
    Serialize(String),
}
```

`crates/fp-store/src/paths.rs`:
```rust
use std::path::{Path, PathBuf};

use directories::ProjectDirs;

/// Where the application keeps its files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub log_dir: PathBuf,
}

impl AppPaths {
    /// The OS-standard locations (XDG, %APPDATA%, ~/Library/Application Support).
    pub fn system() -> Option<Self> {
        let dirs = ProjectDirs::from("org", "Fauste", "Fauste Player")?;
        Some(Self {
            config_dir: dirs.config_dir().to_path_buf(),
            data_dir: dirs.data_dir().to_path_buf(),
            cache_dir: dirs.cache_dir().to_path_buf(),
            log_dir: dirs.data_local_dir().join("logs"),
        })
    }

    /// Everything under one root directory (tests, portable installs).
    pub fn under(root: &Path) -> Self {
        Self {
            config_dir: root.join("config"),
            data_dir: root.join("data"),
            cache_dir: root.join("cache"),
            log_dir: root.join("logs"),
        }
    }

    pub fn config_file(&self) -> PathBuf {
        self.config_dir.join("config.json")
    }

    pub fn playlists_file(&self) -> PathBuf {
        self.data_dir.join("playlists.json")
    }

    pub fn session_file(&self) -> PathBuf {
        self.data_dir.join("session.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn under_places_files_in_separate_subdirectories() {
        let root = Path::new("/tmp/root");
        let p = AppPaths::under(root);
        assert_eq!(p.config_file(), root.join("config").join("config.json"));
        assert_eq!(p.session_file(), root.join("data").join("session.json"));
    }
}
```

`crates/fp-store/src/lib.rs`:
```rust
//! Crash-safe persistence: atomic writes, rotating backups, corrupt-file
//! quarantine and versioned JSON documents.

pub mod atomic;
pub mod error;
pub mod paths;

pub use atomic::{LoadSource, Loaded, ParseError, backup_path, load_with_fallback, write_atomic};
pub use error::StoreError;
pub use paths::AppPaths;
```

- [ ] **Step 2: Write the failing tests for atomic I/O**

`crates/fp-store/src/atomic.rs` (test module first):
```rust
//! Atomic file replacement with rotating backups, and loading with fallback.

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_text(bytes: &[u8]) -> Result<String, ParseError> {
        let s = std::str::from_utf8(bytes).map_err(|e| ParseError::Corrupt(e.to_string()))?;
        if s.starts_with("ok:") {
            Ok(s.to_owned())
        } else if s.starts_with("future:") {
            Err(ParseError::TooNew("written by a newer version".into()))
        } else {
            Err(ParseError::Corrupt("bad content".into()))
        }
    }

    fn names(dir: &Path) -> Vec<String> {
        let mut v: Vec<String> =
            fs::read_dir(dir).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().into_owned()).collect();
        v.sort();
        v
    }

    #[test]
    fn writes_rotate_backups_and_leave_no_temp_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("sub/state.json");
        for n in 1..=4 {
            write_atomic(&file, format!("ok:{n}").as_bytes(), 2).unwrap();
        }
        assert_eq!(fs::read_to_string(&file).unwrap(), "ok:4");
        assert_eq!(fs::read_to_string(backup_path(&file, 1)).unwrap(), "ok:3");
        assert_eq!(fs::read_to_string(backup_path(&file, 2)).unwrap(), "ok:2");
        assert_eq!(names(&dir.path().join("sub")), vec!["state.json", "state.json.bak1", "state.json.bak2"]);
    }

    #[test]
    fn zero_backups_keeps_only_the_primary() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("s.json");
        write_atomic(&file, b"ok:1", 0).unwrap();
        write_atomic(&file, b"ok:2", 0).unwrap();
        assert_eq!(names(dir.path()), vec!["s.json"]);
    }

    #[test]
    fn first_run_loads_defaults_silently() {
        let dir = tempfile::tempdir().unwrap();
        let loaded = load_with_fallback(&dir.path().join("none.json"), 3, 1024, parse_text);
        assert_eq!((loaded.value, loaded.source), (None, LoadSource::Defaults));
        assert!(loaded.warnings.is_empty());
    }

    #[test]
    fn truncated_primary_falls_back_to_backup_and_is_quarantined() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("s.json");
        write_atomic(&file, b"ok:1", 3).unwrap();
        write_atomic(&file, b"ok:2", 3).unwrap();
        fs::write(&file, b"{trunc").unwrap(); // crash mid-write of a non-atomic writer
        fs::write(dir.path().join("s.json.tmp"), b"stale").unwrap(); // leftover temp file
        let loaded = load_with_fallback(&file, 3, 1024, parse_text);
        assert_eq!(loaded.value.as_deref(), Some("ok:1"));
        assert_eq!(loaded.source, LoadSource::Backup(1));
        assert_eq!(loaded.warnings.len(), 1);
        assert!(!file.exists());
        assert!(names(dir.path()).iter().any(|n| n.starts_with("s.json.corrupt-")));
    }

    #[test]
    fn too_new_files_are_left_in_place() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("s.json");
        fs::write(&file, b"future:9").unwrap();
        let loaded = load_with_fallback(&file, 3, 1024, parse_text);
        assert_eq!(loaded.source, LoadSource::Defaults);
        assert!(loaded.warnings[0].contains("newer"));
        assert!(file.exists());
    }

    #[test]
    fn oversized_files_are_refused_without_reading_them() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("s.json");
        fs::write(&file, vec![b'x'; 2048]).unwrap();
        let loaded = load_with_fallback(&file, 0, 1024, parse_text);
        assert_eq!(loaded.source, LoadSource::Defaults);
        assert!(loaded.warnings[0].contains("limit"));
        assert!(file.exists());
    }
}
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cargo test -p fp-store`
Expected: FAIL to compile: `cannot find function write_atomic`.

- [ ] **Step 4: Implement**

Insert above the test module in `atomic.rs`:
```rust
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Why a candidate file could not be used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Damaged or invalid: the file is quarantined.
    Corrupt(String),
    /// Written by a newer version of the app: the file is left untouched.
    TooNew(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadSource {
    Primary,
    Backup(usize),
    Defaults,
}

#[derive(Debug)]
pub struct Loaded<T> {
    /// `None` means "use defaults".
    pub value: Option<T>,
    pub source: LoadSource,
    pub warnings: Vec<String>,
}

fn sibling(path: &Path, suffix: &str) -> PathBuf {
    let mut name = OsString::from(path.as_os_str());
    name.push(".");
    name.push(suffix);
    PathBuf::from(name)
}

pub fn backup_path(path: &Path, n: usize) -> PathBuf {
    sibling(path, &format!("bak{n}"))
}

/// Replaces `path` with `bytes` so that a crash at any point leaves either the
/// old or the new content in place, never a mix. It then keeps up to `backups`
/// previous versions.
pub fn write_atomic(path: &Path, bytes: &[u8], backups: usize) -> io::Result<()> {
    let dir = path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "path has no parent directory"))?;
    fs::create_dir_all(dir)?;
    let tmp = sibling(path, "tmp");
    {
        let mut file = File::create(&tmp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }
    rotate_backups(path, backups)?;
    fs::rename(&tmp, path)?;
    sync_dir(dir)
}

fn rotate_backups(path: &Path, backups: usize) -> io::Result<()> {
    if backups == 0 || !path.exists() {
        return Ok(());
    }
    for n in (1..backups).rev() {
        let from = backup_path(path, n);
        if from.exists() {
            fs::rename(&from, backup_path(path, n + 1))?;
        }
    }
    // Copy (not rename) so the primary stays in place until the new one replaces it.
    fs::copy(path, backup_path(path, 1))?;
    Ok(())
}

#[cfg(unix)]
fn sync_dir(dir: &Path) -> io::Result<()> {
    File::open(dir)?.sync_all()
}

#[cfg(not(unix))]
fn sync_dir(_dir: &Path) -> io::Result<()> {
    Ok(())
}

fn read_limited(path: &Path, max_bytes: u64) -> io::Result<Option<Vec<u8>>> {
    let meta = match fs::metadata(path) {
        Ok(meta) => meta,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };
    if meta.len() > max_bytes {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("file is {} bytes, the limit is {max_bytes}", meta.len()),
        ));
    }
    fs::read(path).map(Some)
}

fn quarantine(path: &Path, warnings: &mut Vec<String>) {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let target = sibling(path, &format!("corrupt-{stamp}"));
    if let Err(e) = fs::rename(path, &target) {
        warnings.push(format!("{}: could not quarantine: {e}", path.display()));
    }
}

/// Loads the primary file, falling back to `.bak1`…`.bakN`. Never fails:
/// when nothing is usable it returns `value: None` (use defaults).
pub fn load_with_fallback<T>(
    path: &Path,
    backups: usize,
    max_bytes: u64,
    parse: impl Fn(&[u8]) -> Result<T, ParseError>,
) -> Loaded<T> {
    let mut warnings = Vec::new();
    let candidates = std::iter::once((LoadSource::Primary, path.to_path_buf()))
        .chain((1..=backups).map(|n| (LoadSource::Backup(n), backup_path(path, n))));
    for (source, candidate) in candidates {
        match read_limited(&candidate, max_bytes) {
            Ok(None) => {}
            Ok(Some(bytes)) => match parse(&bytes) {
                Ok(value) => return Loaded { value: Some(value), source, warnings },
                Err(ParseError::TooNew(msg)) => warnings.push(format!("{}: {msg}", candidate.display())),
                Err(ParseError::Corrupt(msg)) => {
                    warnings.push(format!("{}: unreadable ({msg}); quarantined", candidate.display()));
                    quarantine(&candidate, &mut warnings);
                }
            },
            Err(e) => warnings.push(format!("{}: {e}", candidate.display())),
        }
    }
    Loaded { value: None, source: LoadSource::Defaults, warnings }
}
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p fp-store`
Expected: 7 tests PASS.

- [ ] **Step 6: Check and commit**

Run: `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
```bash
git add crates/fp-store Cargo.lock
git commit -m "feat(store): add app paths and crash-safe atomic file I/O

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 10: Versioned documents, migrations and the `Store` facade

**Files:**
- Create: `crates/fp-store/src/migrate.rs`, `crates/fp-store/src/docs.rs`, `crates/fp-store/src/store.rs`, `crates/fp-store/tests/store_roundtrip.rs`
- Modify: `crates/fp-store/src/lib.rs`

**Interfaces:**
- Consumes:
  - from `fp-model`: `AppState`, `Config`, `Limits`, `Library`, `Playlists`, `IdGen`, `PlayerSession`, `RestoreParts`, `EngineAction`, `PlayerId`;
  - from Task 9: `write_atomic`, `load_with_fallback`, `ParseError`, `AppPaths`, `StoreError`.
- Produces:
  - Schema versions `CONFIG_SCHEMA = 1`, `PLAYLISTS_SCHEMA = 1`, `SESSION_SCHEMA = 1`, and migration tables `CONFIG_MIGRATIONS`, `PLAYLISTS_MIGRATIONS`, `SESSION_MIGRATIONS: &[Migration]` (empty at v1).
  - `Migration = fn(serde_json::Value) -> Result<serde_json::Value, String>`, where entry *k* upgrades schema *k+1* to *k+2*.
  - `upgrade(doc: Value, current: u32, migrations: &[Migration]) -> Result<Value, ParseError>`.
  - Documents: `ConfigDoc { schema_version, config }`, `PlaylistsDoc { schema_version, library, playlists, ids }` and `SessionDoc { schema_version, players: Vec<PlayerSession> }`.
  - `Store::new(paths: AppPaths, limits: Limits)`, `Store::paths()`, and `Store::load(&self, default_playlist_name: &str) -> LoadedState`, where `LoadedState { state: AppState, actions: Vec<EngineAction>, warnings: Vec<String> }`.
  - `save_config(&AppState)`, `save_playlists(&AppState)` and `save_session(&AppState, position_of: impl Fn(PlayerId) -> f64)`, each returning `Result<(), StoreError>`.

- [ ] **Step 1: Write the failing tests**

`crates/fp-store/tests/store_roundtrip.rs`:
```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::PathBuf;

use fp_model::{AppState, Command, Config, EngineAction, Transport, apply};
use fp_store::{AppPaths, Store};

fn store(dir: &tempfile::TempDir) -> Store {
    Store::new(AppPaths::under(dir.path()), Config::default().limits)
}

#[test]
fn first_run_starts_with_defaults_and_no_warnings() {
    let dir = tempfile::tempdir().unwrap();
    let loaded = store(&dir).load("Main");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    assert_eq!(loaded.state.players.len(), 4);
    assert_eq!(loaded.state.playlists.len(), 1);
}

#[test]
fn saved_state_is_restored_paused_at_its_position() {
    let dir = tempfile::tempdir().unwrap();
    let s = store(&dir);
    let mut state = AppState::new(Config::default(), "Main");
    let playlist = state.playlists.first_id().unwrap();
    let paths = vec![PathBuf::from("/m/a.flac"), PathBuf::from("/m/b.flac")];
    apply(&mut state, Command::InsertPaths { playlist, index: 0, paths }).unwrap();
    for t in state.library.iter_mut() {
        t.duration_secs = 200.0;
    }
    let p = state.players[0].id;
    apply(&mut state, Command::Play(p)).unwrap();

    s.save_config(&state).unwrap();
    s.save_playlists(&state).unwrap();
    s.save_session(&state, |_| 42.5).unwrap();

    let loaded = s.load("Main");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    assert_eq!(loaded.state.library, state.library);
    assert_eq!(loaded.state.playlists, state.playlists);
    let restored = loaded.state.player(p).unwrap();
    assert_eq!(restored.transport, Transport::Paused);
    assert_eq!(restored.current, state.players[0].current);
    assert!(loaded.actions.iter().any(|a| matches!(
        a,
        EngineAction::LoadPaused { player, request } if *player == p && (request.from_secs - 42.5).abs() < 1e-9
    )));
}

#[test]
fn newer_schema_is_not_quarantined() {
    let dir = tempfile::tempdir().unwrap();
    let s = store(&dir);
    let path = s.paths().config_file();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, r#"{"schema_version":99,"config":{}}"#).unwrap();
    let loaded = s.load("Main");
    assert!(loaded.warnings.iter().any(|w| w.contains("newer")), "{:?}", loaded.warnings);
    assert!(path.exists());
    assert_eq!(loaded.state.config, Config::default());
}

#[test]
fn corrupt_config_falls_back_to_defaults_with_a_warning() {
    let dir = tempfile::tempdir().unwrap();
    let s = store(&dir);
    let path = s.paths().config_file();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "not json").unwrap();
    let loaded = s.load("Main");
    assert!(!loaded.warnings.is_empty());
    assert_eq!(loaded.state.config, Config::default());
    let quarantined = fs::read_dir(path.parent().unwrap())
        .unwrap()
        .any(|e| e.unwrap().file_name().to_string_lossy().contains(".corrupt-"));
    assert!(quarantined);
}

#[test]
fn invalid_values_are_clamped_on_load() {
    let dir = tempfile::tempdir().unwrap();
    let s = store(&dir);
    let path = s.paths().config_file();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, r#"{"schema_version":1,"config":{"players":{"count":0}}}"#).unwrap();
    let loaded = s.load("Main");
    assert_eq!(loaded.state.players.len(), 1);
    assert!(loaded.warnings.iter().any(|w| w.contains("players.count")), "{:?}", loaded.warnings);
}
```

Add a unit test module at the bottom of `crates/fp-store/src/migrate.rs` (the file is created in Step 3):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn add_field(mut v: Value) -> Result<Value, String> {
        v.as_object_mut().ok_or("not an object")?.insert("added".into(), json!(true));
        Ok(v)
    }

    #[test]
    fn upgrades_step_by_step_and_stamps_the_version() {
        let out = upgrade(json!({"schema_version": 1}), 2, &[add_field]).unwrap();
        assert_eq!(out, json!({"schema_version": 2, "added": true}));
    }

    #[test]
    fn current_version_passes_through() {
        let out = upgrade(json!({"schema_version": 1, "x": 1}), 1, &[]).unwrap();
        assert_eq!(out, json!({"schema_version": 1, "x": 1}));
    }

    #[test]
    fn newer_and_invalid_versions_are_rejected() {
        assert!(matches!(upgrade(json!({"schema_version": 5}), 1, &[]), Err(ParseError::TooNew(_))));
        assert!(matches!(upgrade(json!({"schema_version": 0}), 1, &[]), Err(ParseError::Corrupt(_))));
        assert!(matches!(upgrade(json!({}), 1, &[]), Err(ParseError::Corrupt(_))));
        assert!(matches!(upgrade(json!({"schema_version": 1}), 3, &[add_field]), Err(ParseError::Corrupt(_))));
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-store`
Expected: FAIL to compile: `cannot find type Store` / `upgrade`.

- [ ] **Step 3: Implement migrations**

Insert at the top of `crates/fp-store/src/migrate.rs`:
```rust
//! Schema upgrades. Each document carries `schema_version`; migrations turn
//! version k+1 into k+2 until the current version is reached.

use serde_json::{Value, json};

use crate::atomic::ParseError;

pub type Migration = fn(Value) -> Result<Value, String>;

pub fn upgrade(mut doc: Value, current: u32, migrations: &[Migration]) -> Result<Value, ParseError> {
    let version = doc
        .get("schema_version")
        .and_then(Value::as_u64)
        .and_then(|v| u32::try_from(v).ok())
        .ok_or_else(|| ParseError::Corrupt("missing or invalid schema_version".into()))?;
    if version == 0 {
        return Err(ParseError::Corrupt("schema_version 0 is invalid".into()));
    }
    if version > current {
        return Err(ParseError::TooNew(format!(
            "written by a newer version of the app (schema {version}, this build reads up to {current})"
        )));
    }
    for from in version..current {
        let step = usize::try_from(from - 1).map_err(|e| ParseError::Corrupt(e.to_string()))?;
        let migration = migrations
            .get(step)
            .ok_or_else(|| ParseError::Corrupt(format!("no migration from schema {from}")))?;
        doc = migration(doc).map_err(ParseError::Corrupt)?;
        if let Some(obj) = doc.as_object_mut() {
            obj.insert("schema_version".into(), json!(from + 1));
        }
    }
    Ok(doc)
}
```

- [ ] **Step 4: Implement the documents and the store**

`crates/fp-store/src/docs.rs`:
```rust
//! On-disk document shapes. Bump a `*_SCHEMA` constant and append a
//! migration whenever a document's shape changes.

use serde::{Deserialize, Serialize};

use fp_model::{Config, IdGen, Library, PlayerSession, Playlists};

use crate::migrate::Migration;

pub const CONFIG_SCHEMA: u32 = 1;
pub const PLAYLISTS_SCHEMA: u32 = 1;
pub const SESSION_SCHEMA: u32 = 1;

pub const CONFIG_MIGRATIONS: &[Migration] = &[];
pub const PLAYLISTS_MIGRATIONS: &[Migration] = &[];
pub const SESSION_MIGRATIONS: &[Migration] = &[];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigDoc {
    pub schema_version: u32,
    #[serde(default)]
    pub config: Config,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistsDoc {
    pub schema_version: u32,
    #[serde(default)]
    pub library: Library,
    #[serde(default)]
    pub playlists: Playlists,
    #[serde(default)]
    pub ids: IdGen,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDoc {
    pub schema_version: u32,
    #[serde(default)]
    pub players: Vec<PlayerSession>,
}
```

`crates/fp-store/src/store.rs`:
```rust
use std::path::Path;

use serde::Serialize;
use serde::de::DeserializeOwned;

use fp_model::{AppState, EngineAction, Limits, PlayerId, RestoreParts};

use crate::atomic::{Loaded, ParseError, load_with_fallback, write_atomic};
use crate::docs::{
    CONFIG_MIGRATIONS, CONFIG_SCHEMA, ConfigDoc, PLAYLISTS_MIGRATIONS, PLAYLISTS_SCHEMA, PlaylistsDoc,
    SESSION_MIGRATIONS, SESSION_SCHEMA, SessionDoc,
};
use crate::error::StoreError;
use crate::migrate::{Migration, upgrade};
use crate::paths::AppPaths;

/// Result of loading everything at startup. Loading never fails: problems
/// become `warnings` and defaults are used instead.
#[derive(Debug)]
pub struct LoadedState {
    pub state: AppState,
    pub actions: Vec<EngineAction>,
    pub warnings: Vec<String>,
}

pub struct Store {
    paths: AppPaths,
    /// Limits used to read `config.json` itself; the loaded config's limits apply afterwards.
    limits: Limits,
}

fn load_doc<T: DeserializeOwned>(
    path: &Path,
    backups: usize,
    max_bytes: u64,
    schema: u32,
    migrations: &[Migration],
) -> Loaded<T> {
    load_with_fallback(path, backups, max_bytes, |bytes| {
        let value: serde_json::Value =
            serde_json::from_slice(bytes).map_err(|e| ParseError::Corrupt(e.to_string()))?;
        let value = upgrade(value, schema, migrations)?;
        serde_json::from_value(value).map_err(|e| ParseError::Corrupt(e.to_string()))
    })
}

impl Store {
    pub fn new(paths: AppPaths, limits: Limits) -> Self {
        Self { paths, limits }
    }

    pub fn paths(&self) -> &AppPaths {
        &self.paths
    }

    pub fn load(&self, default_playlist_name: &str) -> LoadedState {
        let mut warnings = Vec::new();

        let config_doc: Loaded<ConfigDoc> = load_doc(
            &self.paths.config_file(),
            self.limits.backup_count,
            self.limits.max_state_file_bytes,
            CONFIG_SCHEMA,
            CONFIG_MIGRATIONS,
        );
        warnings.extend(config_doc.warnings);
        let mut config = config_doc.value.map(|d| d.config).unwrap_or_default();
        warnings.extend(config.validate().into_iter().map(|w| w.to_string()));
        let limits = config.limits.clone();

        let lists: Loaded<PlaylistsDoc> = load_doc(
            &self.paths.playlists_file(),
            limits.backup_count,
            limits.max_state_file_bytes,
            PLAYLISTS_SCHEMA,
            PLAYLISTS_MIGRATIONS,
        );
        warnings.extend(lists.warnings);
        let session: Loaded<SessionDoc> = load_doc(
            &self.paths.session_file(),
            limits.backup_count,
            limits.max_state_file_bytes,
            SESSION_SCHEMA,
            SESSION_MIGRATIONS,
        );
        warnings.extend(session.warnings);

        let (library, playlists, ids) =
            lists.value.map(|d| (d.library, d.playlists, d.ids)).unwrap_or_default();
        let sessions = session.value.map(|d| d.players).unwrap_or_default();
        let (state, actions) =
            AppState::restore(RestoreParts { config, library, playlists, ids }, &sessions, default_playlist_name);
        LoadedState { state, actions, warnings }
    }

    pub fn save_config(&self, state: &AppState) -> Result<(), StoreError> {
        let doc = ConfigDoc { schema_version: CONFIG_SCHEMA, config: state.config.clone() };
        write_doc(&self.paths.config_file(), &doc, state.config.limits.backup_count)
    }

    pub fn save_playlists(&self, state: &AppState) -> Result<(), StoreError> {
        let doc = PlaylistsDoc {
            schema_version: PLAYLISTS_SCHEMA,
            library: state.library.clone(),
            playlists: state.playlists.clone(),
            ids: state.ids.clone(),
        };
        write_doc(&self.paths.playlists_file(), &doc, state.config.limits.backup_count)
    }

    pub fn save_session(&self, state: &AppState, position_of: impl Fn(PlayerId) -> f64) -> Result<(), StoreError> {
        let doc = SessionDoc { schema_version: SESSION_SCHEMA, players: state.sessions(position_of) };
        write_doc(&self.paths.session_file(), &doc, state.config.limits.backup_count)
    }
}

fn write_doc<T: Serialize>(path: &Path, doc: &T, backups: usize) -> Result<(), StoreError> {
    let bytes = serde_json::to_vec_pretty(doc).map_err(|e| StoreError::Serialize(e.to_string()))?;
    write_atomic(path, &bytes, backups)?;
    Ok(())
}
```

Update `crates/fp-store/src/lib.rs`:
```rust
//! Crash-safe persistence: atomic writes, rotating backups, corrupt-file
//! quarantine and versioned JSON documents.

pub mod atomic;
pub mod docs;
pub mod error;
pub mod migrate;
pub mod paths;
pub mod store;

pub use atomic::{LoadSource, Loaded, ParseError, backup_path, load_with_fallback, write_atomic};
pub use docs::{CONFIG_SCHEMA, ConfigDoc, PLAYLISTS_SCHEMA, PlaylistsDoc, SESSION_SCHEMA, SessionDoc};
pub use error::StoreError;
pub use migrate::{Migration, upgrade};
pub use paths::AppPaths;
pub use store::{LoadedState, Store};
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p fp-store`
Expected: all tests PASS: 1 in paths, 6 in atomic, 3 in migrate and 5 in `store_roundtrip.rs`.

- [ ] **Step 6: Final full check**

Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
Expected: everything green.

- [ ] **Step 7: Commit**

```bash
git add crates/fp-store Cargo.lock
git commit -m "feat(store): add versioned documents, migrations and Store facade

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

## Spec coverage of this plan

| Spec section | Covered here | Deferred to |
|---|---|---|
| §2.1 workspace, crate boundaries, `forbid(unsafe)` | Task 1 (`fp-model`, `fp-store`) | Plans 2–4 add their crates |
| §2.4 config groups, clamping, no hardcoded limits, growable players | Tasks 4, 7 | — |
| §3 rules 2–15, 21 | Tasks 5–7 | — |
| §3 rules 1, 16–20 (colours, keys, countdown, badges, footer) | model data only (played flags, markers) | Plan 4 (UI view-model) |
| §4 engine | `EngineAction` / `EngineEvent` contract only | Plan 2 |
| §6 analysis | `AnalysisSettings`, marker model | Plan 3 |
| §7 persistence (atomic write, backups, fallback, quarantine, schema, crash recovery, limits) | Tasks 8–10 | M3U/PLS → Phase 2 |
| §9 no unwrap/expect/panic | Task 1 lints | `indexing_slicing` lint added in Plan 2 for `fp-engine`/`fp-backends` |
| §10 `cargo deny` in CI | Task 1 | fuzzing → Phase 2 (parsers) |
| §11 model unit and property tests, store tests, CI matrix | Tasks 1, 3, 5–10 | engine, analysis and UI tests → Plans 2–4 |
