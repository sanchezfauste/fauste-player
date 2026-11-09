# External editor and changed files Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Open a library track in an operator-chosen external editor, and have Fauste Player notice changed library files by itself, analyse them again (never while they sound) and flag tracks whose audio changed under manual markers.

**Architecture:** `fp-model` holds the new track data (file stamp, audio fingerprint, review flag) and the pure rules (EE5 sounding, EE10 audio change, EE12 clearing). `fp-analysis` computes the fingerprint while decoding. A new `fp-app::changes` module owns a `fp-change-check` thread that compares stamps; the services thread feeds it (start, point of use, folder events, on demand) and sends changed tracks to the analysis, deferred while they sound. The launch goes through the `Platform` seam; the UI adds menu entries, a row flag, a library filter and two settings.

**Tech Stack:** Rust, egui/egui_kittest, notify, SQLite (tracks as JSON documents: no migration needed).

**Spec:** `docs/superpowers/specs/2026-11-09-external-editor-design.md`

## Global Constraints

- Rule ids EE1–EE13 refer to the spec; cite them in doc comments.
- `library.external_editor: Option<PathBuf>`, default `None`; a non-absolute path loads as `None` with a `ConfigWarning` on `library.external_editor`.
- No periodic polling anywhere (EE7). No new system watch except the non-recursive folders of files opened in the editor.
- Never analyse again, nor apply a change-triggered result, while `track_sounding` is true (EE5, EE9).
- The fingerprint is FNV-1a 64 over the little-endian bytes of every decoded `f32` sample, in decode order. Never `std::hash::DefaultHasher` (not stable across Rust releases).
- `ANALYSIS_VERSION` 10 → 11; `analysis_is_current` keeps 10 current (no library-wide analysis on update).
- Desktop only for the editor: absent on Android and when `FLATPAK_ID` or `SNAP` is set.
- No `stat`, process launch or hashing on the UI thread (rule 8); nothing on the audio callback changes.
- Strings: en-US and es-ES written with each change; the other locales in bulk with AI in Task 10.
- Commits: Conventional Commits with the harness co-author trailer; fmt, clippy `-D warnings` and the whole suite green before each.

## Review Focus

- A track that starts sounding while its change-triggered analysis runs: the result must wait until it stops, then apply. Test in Task 6 (`a_result_for_a_track_that_started_sounding_waits`).
- An editor that saves by deleting and recreating the file inside the watch debounce: the track must not be marked missing. Test in Task 7 (`a_file_replaced_within_the_debounce_is_not_gone`).
- A track flagged, then edited again with tag-only changes: the flag stays (only review or a marker change clears it). Test in Task 2 (`a_tag_only_change_keeps_an_existing_flag`).
- Editor path pointing to a program that no longer exists: a status-bar notice, no panic, the menu still works next time. Test in Task 8 (`a_failed_launch_shows_a_notice`).
- A file opened in the editor whose folder is already inside a watched folder: no second watch, the change is still noticed once. Test in Task 7 (`an_edited_file_in_a_watched_folder_adds_no_watch`).

---

### Task 1: Track data and the audio-change rule (EE6, EE10)

**Files:**
- Modify: `crates/fp-model/src/track.rs` (types, `Track` fields, `TrackAnalysis` fields, `apply_analysis`)
- Create: `crates/fp-model/src/audio_change.rs` (pure rule), export in `crates/fp-model/src/lib.rs`
- Test: `crates/fp-model/tests/it/audio_change.rs` (+ `mod` line in `tests/it/main.rs`)

**Interfaces:**
- Produces:
  - `pub struct FileStamp { pub size: u64, pub modified_ns: u64 }` (Copy, Eq, Serialize, Deserialize; nanoseconds since the Unix epoch), `impl FileStamp { pub fn from_metadata(size: u64, modified: SystemTime) -> Option<Self> }`.
  - `pub struct AudioHash { pub value: u64, pub version: u32 }` (Copy, Eq, serde).
  - `pub struct MarkersReview { pub dropped: Vec<MarkerKind> }` (Clone, Eq, serde).
  - `Track` gains `#[serde(default)] pub file_stamp: Option<FileStamp>`, `pub audio_hash: Option<AudioHash>`, `pub markers_review: Option<MarkersReview>`.
  - `TrackAnalysis` gains `#[serde(default)] pub audio_hash: Option<u64>` (cached with the analysis) and `#[serde(skip)] pub file_stamp: Option<FileStamp>` (set by the services from the job's stamp, never from the cache).
  - `pub enum AudioChange { Same, Changed, Unknown }` and `pub fn audio_change(old: Option<AudioHash>, new: Option<u64>, new_version: u32) -> AudioChange` (`Unknown` when either side is missing or versions differ).
  - `apply_analysis` applies EE10 after the existing updates: on `Changed` with at least one manual marker before the update, drops each manual marker whose `secs > new duration` (its slot then takes the analysis value through `set_auto`), and sets `markers_review = Some(MarkersReview { dropped })` (a new flag replaces an old one, keeping the union of `dropped`). On `Same` or `Unknown` the flag is left as it is. Always stores `file_stamp` (when `Some`) and `audio_hash = new.map(|value| AudioHash { value, version })`.

- [ ] **Step 1: Failing tests** in `audio_change.rs`: `same_fingerprint_raises_no_flag`, `changed_audio_with_a_manual_marker_flags_it`, `changed_audio_without_manual_markers_raises_no_flag`, `no_stored_fingerprint_raises_no_flag_and_stores_one`, `another_version_raises_no_flag`, `a_manual_marker_past_the_new_end_is_dropped_and_replaced_by_the_automatic_one` (manual segue 220 s, new duration 200 s, analysis segue 190 s → segue is Auto 190, `dropped == [SegueStart]`), `a_manual_marker_inside_the_new_audio_stays`, `a_tag_only_change_keeps_an_existing_flag`, and `an_old_track_document_loads_without_the_new_fields` (JSON without them → all `None`).
- [ ] **Step 2:** `cargo test -p fp-model --test it audio_change::` → FAIL (types missing).
- [ ] **Step 3:** Add the types, fields and `audio_change`; extend `apply_analysis` as above. Fix every `TrackAnalysis { .. }` literal in the workspace (`..Default::default()` where they list fields).
- [ ] **Step 4:** Tests pass; `cargo test -p fp-store` still green (documents round-trip).
- [ ] **Step 5:** Commit `feat(model): file stamp, audio fingerprint and markers review flag`.

### Task 2: Clearing the flag, recording stamps, and "sounding" (EE5, EE12)

**Files:**
- Modify: `crates/fp-model/src/command.rs`, `crates/fp-model/src/reducer.rs` (`set_marker`, `ResetMarkers`, new arms)
- Create: `crates/fp-model/src/sounding.rs`, export in `lib.rs`
- Test: `crates/fp-model/tests/it/markers_review.rs`, `crates/fp-model/tests/it/sounding.rs`

**Interfaces:**
- Consumes: Task 1 types.
- Produces:
  - `Command::MarkReviewed { tracks: Vec<TrackId> }`: clears `markers_review` on each (unknown ids ignored).
  - `Command::SetFileStamp { track: TrackId, stamp: FileStamp }`: stores the stamp of a track that had none (ignored if the track has one or is gone).
  - Any accepted `SetMarker` and `ResetMarkers` clears `markers_review` of that track.
  - `pub fn track_sounding(state: &AppState, track: TrackId) -> bool`: true when the track is a player's `current` (any transport), `pending_start`, or CUE entry; the last `history` entry of a player that is `fading`; on a `cartwall.playing` cart or the `cartwall.cue` cart; or `library_cue.track` (any transport).

- [ ] **Step 1: Failing tests**: `mark_reviewed_clears_the_flag_of_each_track`, `setting_a_marker_clears_the_flag`, `clearing_a_marker_clears_the_flag`, `reset_markers_clears_the_flag`, `a_refused_marker_keeps_the_flag` (non-finite secs), `set_file_stamp_fills_only_a_missing_stamp`; in `sounding.rs` one test per source listed above plus `a_track_nowhere_is_not_sounding`.
- [ ] **Step 2:** Run both test files → FAIL.
- [ ] **Step 3:** Implement. Doc comments cite EE5 and EE12.
- [ ] **Step 4:** Tests pass; `cargo test -p fp-remote` green (exhaustive `Command` matches, if any, updated).
- [ ] **Step 5:** Commit `feat(model): clear the markers review flag and tell when a track sounds`.

### Task 3: Configuration (EE1)

**Files:**
- Modify: `crates/fp-model/src/config.rs` (`LibraryConfig`, `Default`, `Config::validate`, inline tests)

**Interfaces:**
- Produces: `LibraryConfig::external_editor: Option<PathBuf>`.

- [ ] **Step 1: Failing test** `external_editor_defaults_to_none_and_drops_a_relative_path`: default `None`; `"audacity"` → `None` with a warning whose field is `library.external_editor`; `/usr/bin/audacity` kept; JSON without the field loads `None`.
- [ ] **Step 2:** `cargo test -p fp-model external_editor` → FAIL.
- [ ] **Step 3:** Add the field (doc: what, default none, absolute path, desktop only) and the check in `validate`, following the existing `ConfigWarning` pushes.
- [ ] **Step 4:** Passes.
- [ ] **Step 5:** Commit `feat(model): external editor setting`.

### Task 4: The audio fingerprint (EE8)

**Files:**
- Create: `crates/fp-analysis/src/fingerprint.rs` (`pub(crate) struct Fingerprint`)
- Modify: `crates/fp-analysis/src/analyze.rs` (feed every decoded block, set `analysis.audio_hash`), `crates/fp-analysis/src/cache.rs` (`ANALYSIS_VERSION = 11`; `analysis_is_current`: `version >= 10 || (version == 9 && …)`; update its doc and test)
- Test: `crates/fp-analysis/tests/it/fingerprint.rs`

**Interfaces:**
- Produces: `Fingerprint::new() -> Self`, `fn push(&mut self, samples: &[f32])`, `fn finish(&self) -> u64` (FNV-1a 64, offset `0xcbf2_9ce4_8422_2325`, prime `0x0100_0000_01b3`, over `s.to_bits().to_le_bytes()`). `TrackAnalysis::audio_hash` is `Some` for every successful analysis.

- [ ] **Step 1: Failing tests** (WAV files written in the test, as the existing ones): `the_same_audio_with_other_tags_has_the_same_fingerprint` (two WAVs, same samples, different `LIST/INFO` `INAM`), `one_changed_sample_changes_the_fingerprint`, `a_trimmed_file_changes_the_fingerprint`, and the updated `version_10_stays_current`.
- [ ] **Step 2:** `cargo test -p fp-analysis --test it fingerprint::` → FAIL.
- [ ] **Step 3:** Implement; the cancelled and early-stop paths still return as today (a damaged stretch hashes what decoded).
- [ ] **Step 4:** Passes; the whole `fp-analysis` suite green.
- [ ] **Step 5:** Commit `feat(analysis): audio fingerprint of the decoded samples`.

### Task 5: The change-check thread (EE7 core)

**Files:**
- Create: `crates/fp-app/src/changes.rs` (declared in `lib.rs`)
- Test: `crates/fp-app/tests/it/changes.rs`

**Interfaces:**
- Consumes: `FileStamp` (Task 1).
- Produces:
  - `pub enum Check { Same, Changed, Unstamped(FileStamp), Unknown }` and `pub fn compare(stored: Option<FileStamp>, now: io::Result<FileStamp>) -> Check` (`Unknown` on any error, NotFound included: missing files stay with the existing missing-file rules).
  - `pub struct ChangeCheck` with `pub fn spawn() -> Option<Self>` (thread `fp-change-check`; logged `None` if it cannot start), `pub fn send(&mut self, batch: Vec<(TrackId, PathBuf, Option<FileStamp>)>) -> bool`, `pub fn answers(&mut self) -> Vec<ChangeAnswer>` (never blocks), `pub fn busy(&self) -> bool`.
  - `pub struct ChangeAnswer { pub changed: Vec<TrackId>, pub unstamped: Vec<(TrackId, FileStamp)> }`.

- [ ] **Step 1: Failing tests**: `compare` table (same, size differs, time differs, unstamped, error → Unknown); `the_thread_reports_a_rewritten_file` (scratch file, stamp, rewrite with another size, batch → `changed == [id]`).
- [ ] **Step 2:** `cargo test -p fp-app --test it changes::` → FAIL.
- [ ] **Step 3:** Implement on the `Probe` pattern of `services.rs` (unbounded channels, one batch at a time).
- [ ] **Step 4:** Passes.
- [ ] **Step 5:** Commit `feat(app): change-check thread for library files`.

### Task 6: Services: start, point of use, on demand, deferral (EE7.1, EE7.4, EE7.5, EE9)

**Files:**
- Modify: `crates/fp-app/src/services.rs`
- Test: `crates/fp-app/tests/it/services.rs` (existing binary)

**Interfaces:**
- Consumes: `ChangeCheck`, `track_sounding`, `Command::SetFileStamp`.
- Produces:
  - `ServiceRequest::CheckTrack(TrackId)` (the marker editor opened; Task 9 sends it), `ServiceRequest::LookForChangedFiles`, `ServiceRequest::FollowEditedFile(TrackId)` (handled in Task 7).
  - The services set `analysis.file_stamp` from `AnalysisResult::stamp` (`FileStamp::from_metadata`) before sending `Command::ApplyAnalysis`.
  - A changed track goes through `reanalyse_track` (urgent, not `fresh`: its key changed anyway) once `!track_sounding`; until then it waits in `changed_waiting: HashSet<TrackId>`. A result of such an analysis for a track that sounds by the time it arrives is held (`held: HashMap<TrackId, AnalysisResult>`) and applied on the first step where it no longer sounds.
  - Point of use: each step, the tracks of every player's `current`, `next` and CUE entries, the carts of the shown page and `library_cue.track` that were not in that set on the previous snapshot are sent for a check.
  - Start: the first step sends every analysed, playable track once (in batches); `unstamped` answers become `Command::SetFileStamp`.
  - EE13 (builder's call, taken): the tag editor's own writes get no special case. They are noticed like any change, analysed once, and EE10 finds the same fingerprint, so no flag. Record it in the ledger as a Ruling.
  - `LookForChangedFiles` sends every analysed, playable track; when its last batch is answered, the count of changed tracks is published for the UI as `pub fn changed_found(&self) -> Option<usize>` through the existing activity cell (one-shot notice).

- [ ] **Step 1: Failing tests**: `a_file_changed_before_start_is_analysed_again`, `a_track_without_a_stamp_gets_one_at_start_and_no_analysis`, `loading_a_changed_track_on_a_player_analyses_it_again`, `a_changed_track_that_sounds_waits_until_it_stops`, `a_result_for_a_track_that_started_sounding_waits`, `look_for_changed_files_reports_the_count`, `an_unchanged_library_sends_no_analysis`.
- [ ] **Step 2:** `cargo test -p fp-app --test it services::` → new tests FAIL.
- [ ] **Step 3:** Implement in a `change_step(&mut self, state: &AppState)` called from `analysis_step`.
- [ ] **Step 4:** Passes; whole `fp-app` suite green.
- [ ] **Step 5:** Commit `feat(app): analyse changed library files again, never while they sound`.

### Task 7: Folder events on known tracks and edited-file folders (EE7.2, EE7.3)

**Files:**
- Modify: `crates/fp-app/src/watch/mod.rs` (`FolderWatcher::watch_flat`), `crates/fp-app/src/watch/notify_watcher.rs`, `crates/fp-app/src/watch/folders.rs` (`WatchOutcome`, `apply_answer`, `follow_edited`), `crates/fp-app/src/watch/scan.rs` (gone check), `crates/fp-app/src/services.rs` (`FollowEditedFile`, touched → check)
- Test: `crates/fp-app/tests/it/watch_parts.rs`, `crates/fp-app/tests/it/watched_folders.rs`

**Interfaces:**
- Produces:
  - `fn watch_flat(&mut self, folder: &Path) -> Result<(), String>` on `FolderWatcher`, default body `self.watch(folder)` (fakes unchanged); `NotifyWatcher` uses `RecursiveMode::NonRecursive`.
  - `WatchOutcome::touched: Vec<TrackId>`: known, non-missing tracks whose file a scan found after an event (today dropped in `apply_answer`). The services send them for a check, not straight to the analysis.
  - `FolderWatch::follow_edited(&mut self, folder: PathBuf)`: watches `folder` flat for the rest of the run unless a followed watched folder already contains it; events under it go through the same debounce and scan.
  - Gone check: a known track found gone by a scan that came from an event is looked at once more on the next free scan before `SetFileState::Missing` is sent.

- [ ] **Step 1: Failing tests**: `an_event_on_a_known_track_reports_it_touched`, `a_file_replaced_within_the_debounce_is_not_gone`, `an_edited_file_outside_watched_folders_is_followed_flat`, `an_edited_file_in_a_watched_folder_adds_no_watch`.
- [ ] **Step 2:** Run the two test files → FAIL.
- [ ] **Step 3:** Implement.
- [ ] **Step 4:** Passes; whole `fp-app` suite green.
- [ ] **Step 5:** Commit `feat(app): notice changes to known tracks and to files opened in the editor`.

### Task 8: Launching the editor (EE2–EE5)

**Files:**
- Modify: `crates/fp-app/src/platform.rs`, `crates/fp-mobile/src/platform.rs`, `crates/fp-app/src/ui/playlist_files.rs` (`FileOutcome`), the place that turns `FileOutcome` into notices in `crates/fp-app/src/ui/app.rs`
- Create: `crates/fp-model/src/external_edit.rs` (block rule)
- Test: `crates/fp-model/tests/it/external_edit.rs`, `crates/fp-app/tests/it/external_editor.rs` (+ the recording platform fake in `tests/it/support`)

**Interfaces:**
- Consumes: `track_sounding`, `LibraryConfig::external_editor`.
- Produces:
  - `pub enum ExternalEditBlock { NoEditor, FileUnavailable, Sounding }`, `pub fn external_edit_block(state: &AppState, track: TrackId) -> Option<ExternalEditBlock>` (in that order).
  - `Platform::can_launch_programs(&self) -> bool` (desktop: false when `FLATPAK_ID` or `SNAP` is set; Android and `NoPlatform`: false) and `Platform::launch_editor(&self, program: &Path, file: &Path, done: Box<dyn FnOnce(Result<(), String>) + Send>)`: on a `fp-editor` thread, `open -a <program> <file>` on macOS when `program` ends in `.app`, else `Command::new(program).arg(file)`, spawned and never waited for; `done` gets the spawn result.
  - `FileOutcome::EditorFailed { program: PathBuf, error: String }` → status-bar notice `notice-editor-failed` (`$program`, `$error`) and a `tracing::warn!`.

- [ ] **Step 1: Failing tests**: model `no_editor_blocks_first`, `a_missing_file_blocks`, `a_sounding_track_blocks`, `a_quiet_track_with_an_editor_is_allowed`; app `a_failed_launch_shows_a_notice` (fake returns `Err`).
- [ ] **Step 2:** Run → FAIL.
- [ ] **Step 3:** Implement.
- [ ] **Step 4:** Passes; `cargo build -p fp-mobile --target aarch64-linux-android` is not required locally, but `cargo clippy --workspace` must be green.
- [ ] **Step 5:** Commit `feat(app): launch the external editor through the platform`.

### Task 9: Menus, row flag, filter and settings (EE1, EE2, EE11, EE12, EE7.5)

**Files:**
- Modify: `crates/fp-app/src/ui/library/menu.rs` (`row_menu`), `crates/fp-app/src/ui/table.rs` (`context_menu`), `crates/fp-app/src/ui/track_cells.rs` (`title_cell`: review flag next to `file_flag`), `crates/fp-model/src/library_view.rs` (`LibrarySelection::MarkersToReview`), `crates/fp-model/src/library_search.rs` (`library_rows`, counts), `crates/fp-app/src/ui/library/tree.rs` (fixed item after *Uncategorised*, shown while its count > 0), `crates/fp-app/src/ui/settings/library.rs` (editor row), `crates/fp-app/src/ui/settings.rs` (`analysis`: Look for changed files), marker editors (send `ServiceRequest::CheckTrack` when opened), `crates/fp-app/locales/en-US/main.ftl`, `crates/fp-app/locales/es-ES/main.ftl`
- Test: `crates/fp-app/tests/it/external_editor.rs`, `crates/fp-model/tests/it/library_search.rs` (or the existing search test file)

**Interfaces:**
- Consumes: Tasks 2, 3, 6, 8.
- Produces (Fluent keys): `menu-open-in-editor`, `menu-open-in-editor-no-editor`, `menu-open-in-editor-unavailable`, `menu-open-in-editor-sounding`, `menu-mark-reviewed`, `track-markers-review-tip` (`$markers`, empty when none dropped), `library-tree-markers-review`, `settings-external-editor`, `settings-hint-external-editor`, `settings-choose`, `settings-clear`, `settings-look-for-changes`, `settings-hint-look-for-changes`, `notice-changed-files` (`$count`), `notice-editor-failed`.
- Behaviour: *Open in external editor* shown when `platform.can_launch_programs()` and exactly one track, dimmed with the block's tooltip; on click `launch_editor`, then `ServiceRequest::FollowEditedFile`. *Mark as reviewed* shown when any selected track is flagged; sends `MarkReviewed` for the flagged ones. The editor row (path, *Choose…* via `pick_file` on a helper thread with no extension filter, *Clear*) is hidden when `!can_launch_programs()`.

- [ ] **Step 1: Failing tests** (`egui_kittest`, `Fake` controller): `the_menu_offers_the_editor_for_one_track`, `the_editor_entry_is_dimmed_without_an_editor`, `the_editor_entry_is_absent_when_programs_cannot_be_launched`, `opening_asks_to_follow_the_file`, `mark_as_reviewed_sends_the_flagged_tracks`, `a_flagged_row_shows_the_review_flag_with_the_dropped_markers`, `the_review_filter_lists_only_flagged_tracks`, `the_settings_row_is_hidden_when_programs_cannot_be_launched`, `look_for_changed_files_sends_the_request`.
- [ ] **Step 2:** `cargo test -p fp-app --test it external_editor::` → FAIL.
- [ ] **Step 3:** Implement, following the existing `entry`/`submenu` helpers and `file_flag`'s `flag` helper (amber `egui_phosphor::regular::SEAL_WARNING`).
- [ ] **Step 4:** Passes; whole workspace suite green.
- [ ] **Step 5:** Commit `feat(ui): open in external editor, markers review flag and filter`.

### Task 10: Documentation, locales and the full check

**Files:**
- Modify: `docs/user/` (library and settings chapters; a "When a file changes" section), `docs/technical/` (change detection, fingerprint, analysis version), `README.md` (feature line), every other `crates/fp-app/locales/*/main.ftl` (AI translation of the Task 8–9 keys), the spec's Status line (implemented)

- [ ] **Step 1:** Write the docs in operator terms (EE7 sources, EE10 cases, EE12 clearing; Android, Flatpak and snap have no editor).
- [ ] **Step 2:** Add the keys to every locale; `cargo test -p fp-app` (locale completeness tests) green.
- [ ] **Step 3:** `scripts/site/build.sh` and `scripts/site/check-links.sh target/site` pass.
- [ ] **Step 4:** Full check: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`.
- [ ] **Step 5:** Run the app (`FAUSTE_HOME=/tmp/fp-ee cargo run -p fp-app`) with Audacity set as the editor: open a track, trim it, save; the track is analysed again and, with a manual marker, flagged. Note the result in the ledger.
- [ ] **Step 6:** Commit `docs: external editor and changed files`.
