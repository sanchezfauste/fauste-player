# Automation plan 7: the schedule editor (§8.2, AU37)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** a floating Schedule editor, opened from the Schedule column, edits
the day templates (blocks and their items), the week plan and single dates
(another template, or blocks changed for that date only), and shows on a
block when its items overrun the next block (AU37).

**Architecture:** every edit is an existing command (plan 3:
`CreateTemplate`, `DuplicateTemplate`, `RenameTemplate`, `DeleteTemplate`,
`AddBlock`, `UpdateBlock`, `RemoveBlock`, `SetWeekTemplate`,
`SetDateTemplate`, `ChangeBlockOnDate`, `AddBlockOnDate`,
`RemoveBlockOnDate`, `RevertBlockOnDate`). One pure model helper,
`schedule::overrun_secs`, gives AU37's figure. The window lives in
`ui/schedule_editor.rs` as an `egui::Window`. Its UI-only state (whether
it is open, the tab, the selected template and date) lives in `ViewState`.
A block is one shared widget for template and date editing; it takes a
closure that turns the edited `Block` into the right command.

**Tech Stack:** Rust, `egui` / `egui_kittest`, Fluent.

**Spec:** `docs/superpowers/specs/2026-11-11-automation-design.md` (§3, §4.2–4.3,
§8.2, AU5–AU14, AU37).

## Global Constraints

- UI strings go in all 11 locales, with Spanish in §3's terms (Plantilla,
  Semana tipo, Cambio del día, Bloque, Elemento, Puntual).
- The UI never blocks, and it only sends commands; refusals come back as
  the usual error notice (`app::error_text`).
- Nocturne tokens from `ui/theme.rs`.
- No `unwrap`/`expect`/`panic` outside tests.

## Review Focus

- A block start typed badly ("25:00", "abc") is not sent; the field shows
  the error and keeps the old value.
- Dropping tracks onto a block whose last item is a rotation inserts the
  tracks before the rotation (the model refuses a rotation that is not
  last).
- AU37 with the last block of the day: the next block is the next day's
  first, or 24:00 when the next day has none.
- Deleting the template the week plan uses: the model refuses, and the
  notice says why.
- A date in the past: it can still be edited (the overrides prune later).

## Rulings made in this plan (the builder's calls per §8.2)

- **Rotations.** Rotations stay in Settings → Automation (plan 2). The
  editor has a **Rotations…** button that opens that section, instead of
  a second rotation editor.
- **Categories.** A category item is added with an "Add from category"
  combo. Dragging a category from the library tree comes later, because
  the tree has no drag source today. Tracks are dragged from the library
  (`DragTracks`), as onto the cartwall.
- **Window.** The window's place is not saved (egui keeps it while the app
  runs). The library window's saved rectangle (LB26) is not copied here.
- **Dates.** A date is chosen with a month grid: Previous/Next month and
  Today.

---

### Task 1: AU37's figure

**Files:** `crates/fp-model/src/schedule.rs`, test
`crates/fp-model/tests/it/schedule_overrun.rs`.

**Produces:**
`pub fn overrun_secs(state: &AppState, items: &[Item], start: TimeOfDay, next_start: Option<TimeOfDay>) -> Option<f64>`
- It sums the known lengths (cue-in to cue-out) of the track items.
- A category item counts its category's shortest playable track; a rotation
  item counts nothing.
- It compares the sum with the time to `next_start`: 24:00 when `None`,
  plus a day when `next_start <= start`.
- It returns `Some(excess)` when positive.

Tests:
- `a_block_that_fits_has_no_overrun`
- `a_block_longer_than_its_slot_says_by_how_much`
- `the_last_block_runs_to_midnight`
- `a_category_counts_its_shortest_track`

### Task 2: The window, its tabs and the week plan

**Files:** create `crates/fp-app/src/ui/schedule_editor.rs`; modify
`ui.rs`, `app.rs` (`ViewState::schedule_editor`, drawn after the library
window), `schedule.rs` (an **Edit** button on the bar opens it), the
locales. Test: `crates/fp-app/tests/it/schedule_editor_ui.rs`.

- Tabs: **Templates**, **Week**, **Days**. The window also has a
  **Rotations…** button, which opens Settings → Automation.
- The Week tab has seven rows (Monday first), each with a template combo
  that sends `SetWeekTemplate`.

Tests:
- `the_edit_button_opens_the_editor`
- `the_week_tab_sets_a_weekday_template`
- `rotations_opens_the_automation_settings`

### Task 3: Templates and blocks

**Files:** `schedule_editor.rs`, locales; tests in `schedule_editor_ui.rs`.

- The template list has New (`CreateTemplate` with
  `schedule-template-new-name`), Duplicate, Rename (a text field, sent
  when it loses focus) and Delete.
- The selected template's blocks are listed in time order:
  - Each block has its start (an `HH:MM:SS` field, parsed with
    `TimeOfDay::parse`; an invalid value is not sent and the field shows
    `schedule-editor-bad-time`).
  - Each block has its items. An item shows its text and length, a
    **Time-critical** checkbox (for track and category items), and Up,
    Down and Remove. Each change sends `UpdateBlock`.
  - Each block has **Remove block**.
  - Below the blocks there is **Add block** (the next free hour).
- AU37: a block with `overrun_secs` shows `schedule-editor-overrun`
  ("Too long by { $time }") in amber on its header.

Tests:
- `new_template_sends_create_template`
- `a_valid_start_sends_update_block`
- `a_bad_start_is_not_sent`
- `the_overrun_shows_on_the_block`
- `remove_block_sends_remove_block`

### Task 4: Adding items

**Files:** `schedule_editor.rs`, locales; tests.

- Dropping `DragTracks` onto a block adds Track items. They go before a
  final rotation item, otherwise at the end. This sends `UpdateBlock`.
- The **Add from category** combo adds a Category item.
- The **Music after** combo sets or replaces the final Rotation item, and
  **None** removes it.

Tests:
- `dropping_tracks_adds_them_to_the_block`
- `tracks_go_before_the_rotation`
- `a_category_item_is_added_from_the_combo`
- `the_rotation_item_is_set_and_cleared`

### Task 5: Days

**Files:** `schedule_editor.rs`, locales; tests.

- A month grid: Previous and Next month, Today, and the days. The selected
  day is highlighted, and days with changes are marked.
- The selected date shows:
  - its template combo: "Its weekday's (Name)" sends `SetDateTemplate`
    with `None`, and any other template sends `Some`;
  - its computed blocks, using the Task 3 block widget, where an edit
    sends `ChangeBlockOnDate`;
  - each changed block's `schedule-changed-for-date` tag and **Back to the
    template** (`RevertBlockOnDate`);
  - **Not on this day** (`RemoveBlockOnDate`);
  - **Add block on this day** (`AddBlockOnDate`).

Tests:
- `a_date_can_use_another_template`
- `editing_a_block_on_a_date_sends_change_block_on_date`
- `back_to_the_template_reverts_it`
- `not_on_this_day_removes_it`

### Task 6: Docs

- `docs/user/automation.md`: an "Editing the schedule" section.
- `docs/technical/ui.md`: the editor.
- `docs/technical/automation.md`: `overrun_secs`.
- The spec's §8.2 rulings.
- The README roadmap line.

Then the final review, test-first fixes, and the PR.
