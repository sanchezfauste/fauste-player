# Automation plan 2 — The music filler Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship the music filler on its own (AU27–AU29). It is a player that
fills its own list from a rotation and trims the rows it has played. It
appears as the **Music** column, and the operator can start, stop, reorder
and add to it. Settings gain the rotation editor and the separation rules.

**Architecture:** The filler is an ordinary `PlayerState` kept **last** in
`AppState::players`. `AppState::filler: Option<FillerState>` names its player
and its playlist, so every playout rule, the engine and the routing reuse
the player machinery unchanged. The auxiliary players are "every player but
the filler" (`AppState::aux_players`). Every place that numbers players for
the operator, or counts them, uses that list.

The pure rule `filler::settle(state)` runs at the end of every `apply` and
`on_event`, just before `reconcile`:
- it tops the list up to the look-ahead, using plan 1's `next_track` with
  the planned start times;
- it trims played rows beyond `filler_played_kept`.

The filler's list is a `Playlist` that tab strips, menus and the remote API
never list.

**Tech Stack:** Rust 2024, egui 0.37 + egui_kittest, rusqlite.

**Spec:** `docs/superpowers/specs/2026-11-11-automation-design.md` (AU27–AU29,
§8.1 Music column, §8.2 the rotation part of the editor, §11). Plan 1:
`docs/superpowers/plans/2026-11-11-automation-plan1-rotation.md`.

## Global Constraints

- **Plan 1 constraints** hold: CLAUDE.md rules, a config field per
  operator value, tests in `tests/it`, a green fmt/clippy/suite before every
  commit, and the co-author trailer.
- **New `AutomationConfig` fields**, each with a clamp in `Config::validate`
  and a row in `docs/technical/persistence.md`:
  - `filler_lookahead_mins`: default 30, range 5–240 (AU27).
  - `filler_played_kept`: default 50, range 10–1 000 (AU27).
- **Rule 10:** the filler never starts by itself. After a restart it is
  stopped, as every player is.
- **Rule 5:** nothing here touches the device callback.
- **Rule 8:** the editor and the column only read snapshots and send
  commands.
- **Android is a non-goal.** The phone never shows the filler: phone views
  use `aux_players`.
- **Display order:** the Music column sits **left** of the auxiliary
  players. In `state.players` the filler is **last**, so positional code
  for P1…Pn is unchanged. The UI orders the columns.

## Review Focus

1. **Player count changes with the filler present.** `SetPlayerCount`
   adds and removes auxiliary players only. The filler stays last, keeps
   its id, and is never counted (Task 2,
   `player_count_changes_leave_the_filler_last`).
2. **Keys and MIDI by number.** Key `5` with four auxiliary players must
   not drive the filler (Task 2,
   `numbered_shortcuts_never_reach_the_filler`).
3. **A library with nothing to play.** Topping up yields nothing, the list
   stays as it is, and there is no loop and no panic. The column shows the
   alert (Task 3, `an_empty_library_adds_nothing_and_flags_it`).
4. **Tracks without a known duration** (not analysed yet, 0 s). Topping up
   must still end, after a bounded number of rows (Task 3,
   `unknown_durations_do_not_fill_without_end`).
5. **Restart in the middle of the list.** The filler comes back stopped
   with its list, its rotation and its generated rows. A lost `kv` value
   leaves its playlist visible as an ordinary playlist, not a crash
   (Task 4, `a_lost_filler_record_degrades_to_an_ordinary_playlist`).

---

## Part A — Model and store

### Task 1: Filler settings and the filler's existence

**Files:**
- Modify: `crates/fp-model/src/config.rs` (the two `AutomationConfig`
  fields and their clamps).
- Create: `crates/fp-model/src/filler.rs`.
- Modify: `crates/fp-model/src/state.rs`:
  - field `pub filler: Option<FillerState>`;
  - `aux_players()`;
  - `is_filler()`.
- Modify: `crates/fp-model/src/session.rs` (the restore literal:
  `filler: None`).
- Modify: `crates/fp-model/src/lib.rs`.
- Modify: `crates/fp-app/src/session.rs` (`filler::ensure` after
  `ensure_default`, before `Conductor::new`).
- Modify: `crates/fp-app/locales/*/main.ftl` (`filler-playlist-name`).
- Test: `crates/fp-model/tests/it/filler.rs` (new).

**Interfaces:**
- Produces:

```rust
// filler.rs
#[derive(Debug, Clone, PartialEq)]
pub struct FillerState {
    pub player: PlayerId,
    pub playlist: PlaylistId,
    /// The rotation it plays; `None` plays the default (AU4).
    pub rotation: Option<RotationId>,
    pub cursor: RotationCursor,
    /// Rows the filler added and may replace (AU28); every other row of
    /// the playlist was added or moved by the operator.
    pub generated: HashSet<EntryId>,
    /// Set when the last top-up found nothing to play (AU34 alert); runtime only.
    pub starved: bool,
}
/// Creates the filler (a stopped player last in `players`, an empty
/// hidden playlist named `playlist_name`) when there is none; returns the
/// engine actions (`AddPlayer`).
pub fn ensure(state: &mut AppState, playlist_name: &str) -> Vec<EngineAction>;
// state.rs
impl AppState {
    pub fn aux_players(&self) -> impl Iterator<Item = &PlayerState>;
    pub fn is_filler(&self, player: PlayerId) -> bool;
    pub fn is_filler_playlist(&self, playlist: PlaylistId) -> bool;
}
```

- The filler's player is in Continuous mode.
- `ensure` uses the rotation cursor of `rotations.default_id()`, or
  `RotationCursor::new(RotationId(0))` when there is none: `next_track`
  falls back on the default either way.

- [ ] **Step 1: Write the failing tests** in `filler.rs`:
  - `ensure_creates_a_stopped_filler_last_once`:
    - after `ensure` twice there is exactly one filler;
    - `players.last().id == filler.player`;
    - its transport is `Stopped`;
    - the returned actions hold one `AddPlayer` (the second call returns
      none);
    - `aux_players().count() == config.players.count`.
  - `filler_settings_defaults_and_ranges`: 30 and 50; `4` clamps to 5 and
    `2000` to 1 000.
- [ ] **Step 2: Run and watch them fail.**
  Run `cargo test -p fp-model --test it filler::`.
- [ ] **Step 3: Implement.** Locale strings: en-US `Music filler`, es-ES
  `Musicalizador`.
- [ ] **Step 4: Run the tests and the workspace suite.** Expected: PASS.
- [ ] **Step 5: Commit** with the message
  `feat(model): give the music filler a player of its own`.

### Task 2: The filler is not an auxiliary player

**Files:**
- Modify: `crates/fp-model/src/reducer.rs:1514` (`set_player_count`
  works on auxiliary players; new players go before the filler).
- Modify: `crates/fp-model/src/shortcuts.rs:139` (numbered player
  shortcuts index `aux_players()`).
- Modify: `crates/fp-model/src/midi.rs` (the same, if it indexes players).
- Modify: `crates/fp-model/src/session.rs`:
  - `sessions()` leaves the filler out;
  - `restore` builds auxiliary players only (the filler is restored in
    Task 4).
- Modify: `crates/fp-model/src/layout.rs:326` and `settle_layout` callers
  (the filler has a share like any column).
- Modify: `crates/fp-model/src/reset_played.rs`, `on_air.rs`,
  `sounding.rs` and `playout.rs`: review each `players` iteration and keep
  the filler where the rule is about sound (on air, playout active, exit
  guard) and drop it where it is about the operator's numbered players.
- Modify: `crates/fp-remote/src/dto.rs` and `events.rs`: players and
  playlists leave out the filler and its playlist.
- Modify: `crates/fp-app/src/ui/settings/keys.rs:54`, `midi.rs:127` and
  `settings.rs:1668`: counts use `aux_players()`.
- Test: `crates/fp-model/tests/it/filler.rs`.
- Test: `crates/fp-remote/tests/it` (the players and playlists list).

**Interfaces:**
- Consumes: `aux_players` and `is_filler` from Task 1.

- [ ] **Step 1: Write the failing tests:**
  - `player_count_changes_leave_the_filler_last`: with `ensure` done,
    `SetPlayerCount(2)` then `SetPlayerCount(5)`; the filler is last each
    time, its id is unchanged, and `aux_players().count()` is 2 then 5.
  - `numbered_shortcuts_never_reach_the_filler`: with 4 auxiliary players,
    `player_command(state, 4, …)` (the fifth) is `None`.
  - `sessions_leave_the_filler_out`.
  - In fp-remote, `the_api_lists_neither_the_filler_nor_its_playlist`.
- [ ] **Step 2: Run and watch them fail.**
- [ ] **Step 3: Implement.** Add a short comment at each iteration
  reviewed, saying why the filler is in or out.
- [ ] **Step 4: Run the tests and the workspace suite.** Expected: PASS.
- [ ] **Step 5: Commit** with the message
  `feat(model): keep the music filler apart from the numbered players`.

### Task 3: Topping up and trimming the list (AU27, AU28)

**Files:**
- Modify: `crates/fp-model/src/filler.rs` (`settle`, `set_rotation`,
  `on_entries_changed`).
- Modify: `crates/fp-model/src/command.rs` and `reducer.rs`:
  - `Command::SetFillerRotation(Option<RotationId>)`;
  - call `filler::settle(state)` in `apply` and `on_event` right before
    `reconcile`.
- Modify: `crates/fp-model/src/reducer.rs`. Moving or inserting rows into
  the filler's playlist by any command makes them operator rows: they are
  never in `generated`. That is automatic for new entries; a row moved
  within the list is removed from `generated`.
- Modify: `docs/technical/automation.md` (section "The music filler").
- Test: `crates/fp-model/tests/it/filler.rs`.

**Interfaces:**
- Consumes: `next_track`, `CursorPick` (plan 1); `Track::play_length_secs`.
- Produces:

```rust
/// AU27, AU28: tops the filler's list up and trims what it played.
pub(crate) fn settle(state: &mut AppState);
/// The fallback length of a row whose track has no known duration.
pub const UNKNOWN_LENGTH_SECS: f64 = 180.0;
/// At most this many rows are added by one `settle`, whatever the lengths.
pub const MAX_ADDED_PER_SETTLE: usize = 50;
```

`settle` follows this algorithm (not fixed by its signature):

```text
f = state.filler else return
list = playlist f.playlist; player = f.player
// Trim (AU27): rows before the current one (or, when stopped with nothing
// current, before the next) that are played by the filler player, keeping
// the last filler_played_kept of them; also drop them from `generated`.
// Uses take_out_entries so players' current/next stay consistent.
// Top up: ahead = rows after current (or from next when stopped)
remaining = Σ play_length(ahead) (UNKNOWN_LENGTH_SECS when 0)
planned  = [(track, wall_now + offset)] for every ahead row, offset accumulating
added = 0
while remaining < lookahead_mins*60 && added < MAX_ADDED_PER_SETTLE:
    pick = next_track(state, &mut cursor, at = wall_now + remaining, planned, rng)
    None → f.starved = true; break
    append entry; generated.insert(entry); planned.push(...); remaining += length; added += 1
starved = false when something was added or ahead is non-empty
```

- `SetFillerRotation(r)`:
  - sets `rotation` and `cursor = RotationCursor::new(r or default)`;
  - removes the **generated** rows after the current row (and the next,
    unless the player is fading into it);
  - then `settle` refills them (AU28).
  - An unknown rotation id is `UnknownRotation`.
- `settle` runs whether the filler is playing or stopped. A stopped filler
  shows its next tracks, so Play has something to start (AU29).

- [ ] **Step 1: Write the failing tests** (rules off unless a test needs
  them; 20 music tracks of 180 s):
  - `settle_fills_up_to_the_lookahead`: a stopped filler, look-ahead 30 →
    10 rows (1 800 s), all generated.
  - `played_rows_beyond_the_kept_count_are_trimmed`: with kept = 10, play
    through 25 rows (drive `on_event` with `TransitionStarted`, or
    `Command::Play` plus `advance`). The list keeps 10 played rows before
    the current one, and `generated` holds no id that is no longer in the
    list.
  - `an_operator_row_is_never_replaced`: insert a track by
    `InsertTracks` into the filler's playlist after the next, then
    `SetFillerRotation`; the operator row is still there, in place, and is
    not in `generated`.
  - `a_removed_row_does_not_count_as_aired`: `RemoveEntry` of a generated
    row leaves `air_times` empty.
  - `an_empty_library_adds_nothing_and_flags_it`: `starved == true`, no
    rows; then add tracks and run any command → rows, `starved == false`.
  - `unknown_durations_do_not_fill_without_end`: tracks with
    `duration_secs = 0` → rows added in one settle ≤ `MAX_ADDED_PER_SETTLE`,
    and the list ends at 10 rows (UNKNOWN_LENGTH_SECS × 10 = 1 800).
  - `the_list_stays_bounded_over_a_day`: drive 500 transitions; the
    playlist never exceeds `kept + rows for the look-ahead + 1`. This is
    AU38's filler part.
  - `set_filler_rotation_regenerates_only_generated_rows`.
- [ ] **Step 2: Run and watch them fail.**
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run the tests and the workspace suite.** Expected: PASS.
- [ ] **Step 5: Commit** with the message
  `feat(model): top up and trim the music filler's list`.

### Task 4: Saving and restoring the filler

**Files:**
- Modify: `crates/fp-store/src/db/snapshot.rs`:
  - `Snapshot.filler: Option<FillerSession>`;
  - kv key `"filler"`;
  - `of`, `read_checked`, `write_full`, `write_changes`;
  - `crates/fp-store/src/import.rs` and the test snapshot literal get
    `None`.
- Modify: `crates/fp-model/src/filler.rs`, `FillerSession` (serde) with:
  - `player: PlayerSession`;
  - `playlist: PlaylistId`;
  - `rotation: Option<RotationId>`;
  - `cursor: RotationCursor`;
  - `generated: Vec<EntryId>`.
- Modify: `crates/fp-model/src/session.rs`:
  - `RestoreParts.filler: Option<FillerSession>`;
  - `restore` rebuilds the filler stopped and last;
  - an unknown playlist, or a duplicate id, drops the record with a note.
- Modify: `crates/fp-store/src/store.rs` (pass it through).
- Modify: `docs/technical/persistence.md` (the `kv` value `filler`).
- Test: `crates/fp-store/tests/it/automation_store.rs`.
- Test: `crates/fp-model/tests/it/filler.rs`.

- [ ] **Step 1: Write the failing tests:**
  - `the_filler_round_trips_stopped_with_its_list`: save while playing,
    load, then check:
    - the transport is `Stopped`;
    - the same player id and playlist;
    - the same rows and the same `generated`;
    - the same rotation and cursor.
  - `a_lost_filler_record_degrades_to_an_ordinary_playlist`: delete the
    kv row. The load has no error, the old playlist is listed as an
    ordinary playlist, and `ensure` makes a new filler.
- [ ] **Step 2: Run and watch them fail.**
- [ ] **Step 3: Implement** (the kv value is lenient; a bad one gives a
  warning and `None`).
- [ ] **Step 4: Run the tests and the workspace suite.** Expected: PASS.
- [ ] **Step 5: Commit** with the message
  `feat(store): save the music filler`.

## Part B — Screens

### Task 5: The Music column

**Files:**
- Modify: `crates/fp-app/src/ui/app.rs:2541` and `panel_layout.rs`. The
  column order is `[filler, aux…]`; the filler's column is shown or hidden
  like a player's (RL7). A layout saved before has no entry for it, so it
  is **hidden** by default: a station that never automates never sees it.
- Modify: `crates/fp-app/src/ui/panels.rs:268` (the top-bar tile: label
  `music-column`, "Music" / "Música"), `player.rs:429` and `662` (header
  name), `pending.rs`, `table.rs:1075` and `settings.rs:1235` (labels).
  Everything goes through one new helper,
  `ui::player_name(t, state, id) -> String`: `Music` for the filler,
  `P{n}` among the auxiliary players otherwise.
- Modify: `crates/fp-app/src/ui/player.rs`:
  - The filler column has **no tab strip**.
  - Its header row shows a **rotation selector**: a combo of the
    rotations, "Default (<name>)" first, sending `SetFillerRotation`.
  - It shows the **starved** alert (`filler-starved`: "Nothing to play:
    check the rotation's categories").
- Modify: `crates/fp-app/src/ui/table.rs`. The filler's rows **added by
  hand** carry a small marker in the `#` column (`icon::HAND`, with the
  tooltip `filler-row-manual`); generated rows look like any row.
- Modify: `crates/fp-app/src/ui/music.rs` and `touch.rs` (phone): use
  `aux_players()`.
- Modify: `crates/fp-app/locales/*/main.ftl`: `music-column`,
  `filler-starved`, `filler-row-manual` and `filler-rotation-default`.
- Modify: `docs/user/` (a new page `automation.md` "The music filler",
  linked from `docs/user/SUMMARY.md`) and `README.md` (features: "music
  filler").
- Test: `crates/fp-app/tests/it/filler_ui.rs` (new, egui_kittest with the
  `Fake` controller).

- [ ] **Step 1: Write the failing UI tests:**
  - `the_music_column_is_hidden_until_shown_from_the_top_bar`: then click
    the "Music" tile; the column appears **left** of P1.
  - `the_rotation_selector_sends_set_filler_rotation`.
  - `a_starved_filler_shows_the_alert`.
  - `a_manual_row_carries_the_hand_marker`.
  - `the_auxiliary_players_keep_their_numbers`: P1–P4 labels are
    unchanged with the filler present.
- [ ] **Step 2: Run and watch them fail.**
  Run `cargo test -p fp-app --test it filler_ui::`.
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run the tests and the workspace suite.** Expected: PASS.
- [ ] **Step 5: Commit** with the message
  `feat(ui): show the music filler as the Music column`.

### Task 6: The rotation editor and the rules in Settings (§8.2, rotation part)

**Files:**
- Create: `crates/fp-app/src/ui/settings/automation.rs`, a new
  `Section::Automation` (icon `icon::SHUFFLE`, tab
  `settings-tab-automation`), placed after Library. It contains:
  - **Rotations:**
    - a list of rotations, the default marked;
    - buttons New, Rename (inline text edit), Delete (refused for the last
      one, with the model's error shown) and Make default.
  - **Steps of the selected rotation:** one row per step:
    - a source picker (a combo: every category as an indented tree path,
      then "All <kind>" for each kind);
    - `min` and `max` drag values in 1–99;
    - remove, up and down buttons;
    - an "Add step" button.
    - Every edit sends `UpdateRotation` with the whole rotation.
    - A step on a deleted category shows `rotation-step-missing` in the
      warning colour.
  - **Rules:** sliders for:
    - `track_separation_mins` (0–10 080, step 15, " min");
    - `track_separation_percent` (0–95, " %");
    - `artist_separation_mins` (0–1 440, step 5, " min");
    - `filler_lookahead_mins` (5–240);
    - `filler_played_kept` (10–1 000).
    - Each has a hint string.
  - The Automation section is restorable: a new `SettingsSection::Automation`
    in `restore.rs` resets `config.automation`.
- Modify: `crates/fp-app/src/ui/settings.rs` (the section list, `body`
  dispatch, `restorable`).
- Modify: `crates/fp-model/src/restore.rs`.
- Modify: `crates/fp-app/locales/*/main.ftl`: every string of the
  section. en-US and es-ES by hand, the rest translated.
- Modify: `docs/user/automation.md` (the rotation editor and the rules)
  and `docs/user/settings.md` (the new section).
- Test: `crates/fp-app/tests/it/rotation_editor_ui.rs` (new).
- Test: `crates/fp-model/tests/it/filler.rs` (restore defaults).

- [ ] **Step 1: Write the failing tests:**
  - `new_rotation_sends_create_with_one_music_step`.
  - `editing_a_step_count_sends_update_rotation`.
  - `deleting_the_last_rotation_shows_the_error`.
  - `a_step_on_a_deleted_category_is_flagged`.
  - `restore_defaults_resets_the_automation_rules`.
- [ ] **Step 2: Run and watch them fail.**
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run the tests and the workspace suite.** Expected: PASS.
- [ ] **Step 5: Commit** with the message
  `feat(ui): edit rotations and the music rules in Settings`.

### End of plan

- [ ] Screenshots: add a Music column to the `demo_session` scene only if it
  stays hidden by default. The README image must not change unless the
  maintainer wants it to; say so in the PR.
- [ ] Run the `docs-sync-checker` agent and the final review (CLAUDE.md "End
  of plan"). Fix Critical and Important findings test-first.
- [ ] Open the PR for plans 1 and 2 together: `feat: music filler and
  rotations`, following the template. Wait for CI on all three OSes, and
  merge when green (`gh pr merge --merge --delete-branch`), with a rebase
  on master first.
