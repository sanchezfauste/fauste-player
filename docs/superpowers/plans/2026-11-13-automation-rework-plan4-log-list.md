# Automation rework, plan 4: the Log's list is a playlist — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The Log column's list looks and works like a player's playlist (AR14–AR19, AR11, AR16): the playlists' table look, a Time column, block header rows, visible category tracks, the music fill row on air, live editing through an Add menu, drag and drop and the row menu, saved as date changes.

**Architecture:** The model gains the row data the list needs (`schedule_view`: planned start, chosen track, late next, the music fill row on air, block totals and overrun) and one live-edit command, `Command::EditLog { date, edit: LogEdit }`, whose rules (AR17–AR19) live in a new `fp-model/src/log_edit.rs` and store through `change_block_on_date` / `add_block_on_date`. The UI redraws `schedule::schedule_list` with `egui_extras::TableBuilder` and the playlists' row look (same row height, fonts, `theme::ON_AIR_ROW` / `NEXT_ROW`, header style), and gives the Log its own footer with the Add menu.

**Tech Stack:** Rust, egui / egui_extras, `egui_kittest`; `fp-model` tests with `automation_rig::Rig`.

**Spec:** `docs/superpowers/specs/2026-11-13-automation-rework-design.md` §6 (AR14, AR14b, AR15–AR19), AR11, AR16; mock-up `Automatización v2.html` (Claude Design project in CLAUDE.md).

## Global Constraints

- The Time column is on the Log only (AR14b); the Music fill keeps its table and rotation selector.
- No "Today's changes" button (AR14b).
- Rows of past dates, the row on air and played rows are not editable (AR19); a Music fill item stays last in its block.
- New strings in every locale (en-US, es-ES written; others translated).
- No new widget styles or colours: reuse `table.rs` / `track_cells.rs` / `theme.rs` (AR24).

## Review Focus

- An edit to the block on air reloads the Log player's queue after the row on air, never cutting it (Task 2 test).
- Dropping onto a row of yesterday is refused (Task 2 test).
- A block whose items exceed the time to the next block shows the overrun on its header (Task 1 test).
- A category item once loaded shows the chosen track (Task 1 test).
- The list stays fast with a day of 24 blocks (no per-frame selection or database work; `schedule_view` is pure and cheap).

---

### Task 1: The rows the list shows

**Files:** `crates/fp-model/src/automation.rs` (`ScheduleRow`, `schedule_view`, `AutomationState::fill_item`); Test `crates/fp-model/tests/it/automation_view.rs`.

**Interfaces — Produces:**
- `ScheduleRow::Block { date, block, start, origin, status, total_secs: f64, overrun_secs: Option<f64> }`.
- `ScheduleRow::Item { item, what, length_secs, time_critical, status, starts_at: NaiveDateTime, track: Option<TrackId>, late: bool }` — `track`: the track item's track, or the track chosen for a category item while its block is loaded (from `rows`); `late`: the row is next, automation is stopped and its block's start plus the late tolerance has passed (AR11); a rotation item's `length_secs` is the time from its start to the next block.
- `AutomationState::fill_item: Option<ItemRef>`: the rotation item whose music is on air (set when a block with a rotation item ends while running, or when a rotation item is played; cleared by `load_block` and `halt`); its row's status is `OnAir` while `fill_is_current`.

- [ ] **Step 1: Failing tests:** `rows_carry_their_planned_start`, `a_loaded_category_item_shows_its_track`, `the_music_fill_row_is_on_air_between_blocks`, `a_late_next_is_marked_late`, `a_block_header_carries_its_total_and_overrun`.
- [ ] **Step 2–4:** implement; `cargo test -p fp-model --test it automation` PASS.
- [ ] **Step 5: Commit** `feat(model): the log's rows carry what its list shows`

### Task 2: Live editing (AR17–AR19)

**Files:** Create `crates/fp-model/src/log_edit.rs`; modify `command.rs` (`Command::EditLog`), `reducer.rs`, `error.rs` (`ModelError::LogRowLocked`, `ModelError::PastDate`); Test create `crates/fp-model/tests/it/log_edit.rs` (+ `mod`).

**Interfaces — Produces:**
```rust
pub enum LogEdit {
    /// Items inserted before `index` of the block (at its end, before a
    /// Music fill item, when `index` is past it); a Rotation item replaces
    /// the block's.
    Insert { block: BlockId, index: usize, items: Vec<Item> },
    Remove { block: BlockId, index: usize },
    Move { from: (BlockId, usize), to: (BlockId, usize) },
    SetHardStart { block: BlockId, index: usize, on: bool },
    AddBlock { start: TimeOfDay },
    MoveBlock { block: BlockId, start: TimeOfDay },
    RemoveBlock { block: BlockId },
    Revert { block: BlockId },
}
Command::EditLog { date: NaiveDate, edit: LogEdit }
```
Rules: `date` before today → `PastDate`; an index that is the row on air or a played row of the block on air → `LogRowLocked`; otherwise the block's new items go through `change_block_on_date` (a date change, AU11–AU12). An edit to the block on air reloads the Log's queue after the row on air (`load_block` from the row after the current one, keeping the current row). `AddBlock`, `RemoveBlock`, `Revert` map to the existing date commands.

- [ ] **Step 1: Failing tests:** `inserting_a_track_changes_only_that_date`, `a_drop_past_a_music_fill_goes_before_it`, `a_new_rotation_replaces_the_blocks`, `the_row_on_air_cannot_be_removed`, `yesterday_cannot_be_edited`, `editing_the_block_on_air_reloads_after_the_row_on_air`, `a_block_can_be_added_moved_and_removed_today`, `hard_start_toggles_on_one_item`.
- [ ] **Steps 2–4**; **Step 5: Commit** `feat(model): the log is edited live as changes of its date`

### Task 3: The list looks like a playlist (UI)

**Files:** `crates/fp-app/src/ui/schedule.rs` (rewrite `schedule_list` with `TableBuilder`; reuse `table.rs` row height and header painting, `track_cells.rs` text cells, `theme` row colours); Test `crates/fp-app/tests/it/schedule_ui.rs`.

Columns: Time (`HH:MM:SS`, amber when `late`), Title (hard-start clock icon, the track title or category text, the category as a dim secondary text after a chosen track, the music fill icon and "Music fill · name"), Artist (artist, or "until HH:MM" for a music fill row), Duration. Block header rows: start `HH:MM`, "Block", item count and total, "changed for this day", overrun in amber. Missed rows struck through; double-click and the existing menu set next.

- [ ] **Step 1: Failing tests:** `the_log_list_has_the_playlist_columns` (labels Time, Title, Artist, Duration), `a_category_row_shows_its_track`, `a_late_next_shows_its_time_in_amber` (accessible label carries `schedule-late`), existing schedule_ui tests keep passing.
- [ ] **Steps 2–4**; **Step 5: Commit** `feat(ui): the log's list is drawn as a playlist`

### Task 4: Add menu, row menu, drag and drop (UI)

**Files:** `schedule.rs`, `player.rs` (the Log's footer: `log_footer` instead of the playlist footer), `app.rs` (`DragTracks` dropped on the Log); strings `schedule-add`, `schedule-add-track`, `schedule-add-category-track`, `schedule-add-music-fill`, `schedule-add-block`, `schedule-block-time`, `schedule-row-remove`, `schedule-block-move`, `schedule-block-remove`, `schedule-block-revert`, `schedule-late`, `schedule-time`, `schedule-until`; Test `schedule_ui.rs`.

- Footer: **Add** (menu: Track… opens the library window; Category track… and Music fill… a small popup with a combo, inserting before the selected row or at the end of today's next block; Block… a popup with the time field, prefilled with `schedule::free_start` of that date), the next block's start and the time to it.
- Row menu adds Hard start (toggle), Remove; a block header's menu: Change time…, Remove block, Back to the template (changed blocks).
- Library tracks dropped on a row insert before it (`LogEdit::Insert`), with the playlists' drop line; rows dragged within the Log move (`LogEdit::Move`).
- Locked rows (AR19) show no edit entries.

- [ ] **Step 1: Failing tests:** `add_block_sends_add_block_with_a_free_time`, `add_category_track_sends_insert`, `remove_sends_remove`, `the_row_on_air_has_no_remove`, `dropping_tracks_on_a_row_inserts_them_before_it`.
- [ ] **Steps 2–4**; **Step 5: Commit** `feat(ui): the log is edited from its list`

### Task 5: Docs

`docs/user/automation.md` (the Log column section: the list, editing live), `docs/technical/automation.md` (`log_edit.rs`, the new row fields), `docs/technical/ui.md` (`schedule.rs`), the rework spec's AR14 wording if the table reuse differs. **Commit** `docs: the log's list and live editing`.
