# Automation rework, plan 5: templates and days in Settings — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Templates, the week plan and dates are managed in Settings → Automation, and a template or a date is edited in a non-modal popup that is the same list as the Log column (AR20–AR22); the floating schedule editor and the Log's Edit log button go away.

**Architecture:** `log_edit` gains a target, `EditTarget::{Date(NaiveDate), Template(TemplateId)}`, with `Command::EditPlan { target, edit: LogEdit }` replacing `EditLog` (template edits go through `UpdateBlock` / `AddBlock` / `RemoveBlock`; `Revert` only for dates; template edits are never locked). `automation::plan_view(state, target)` returns the same `ScheduleRow`s for a template (its blocks, item `ItemRef`s dated today, statuses `Coming`) or a date. The UI's table moves into `schedule::log_table(ui, scene, view_state, rows, target, playing)`, used by the Log column (`playing = true`) and by the new popup `ui/plan_window.rs` (`playing = false`: no next, no Time-based late, no set-next menu). Settings → Automation gets a "Templates and days" part (template list with Edit…, New, Duplicate, Rename, Delete; the week plan; a calendar with the date's template and Edit day…). Opening the popup closes Settings.

**Tech Stack:** Rust, egui, `egui_kittest`.

**Spec:** `docs/superpowers/specs/2026-11-13-automation-rework-design.md` §7 (AR20–AR22), mock-up section 2–3.

## Global Constraints

- The popup never plays; its rows have no next and no on-air.
- Template edits change only the template; date edits are date changes (past dates locked, as today).
- Strings in every locale; no new widget styles.
- `schedule_editor.rs`, its strings that no longer show, `ViewState::schedule_editor` and `schedule-edit` are removed; their tests are replaced.

## Review Focus

- A template edited while its block is on air today: the on-air block is a date's computed block; a template change reaches it only if the date has no change for that block (AU12) — the reload rule (AR19b) must not fire for template edits. *(Executed: AR22b, today keeps the block on air as loaded, as a date change.)*
- Deleting the template the popup shows closes the popup.
- The popup and Settings never both open (Settings is modal).
- Rotations still edited in Settings as today.

---

### Task 1: Edit targets (model)

`log_edit.rs`: `pub enum EditTarget { Date(NaiveDate), Template(TemplateId) }`; `Command::EditPlan { target, edit }` (rename `EditLog`; update the Log UI and tests); template branch: `Insert/Remove/Move/SetHardStart/MoveBlock` → `UpdateBlock`, `AddBlock` → `AddBlock`, `RemoveBlock` → `RemoveBlock`, `Revert` → `UnknownBlock`. `automation::plan_view(state, target) -> Vec<ScheduleRow>`.

Tests (`log_edit.rs`, `automation_view.rs`): `a_template_edit_changes_the_template_only`, `a_template_block_can_be_added_and_removed`, `template_edits_are_never_locked`, `plan_view_lists_a_templates_blocks`, `plan_view_of_a_date_has_its_changes`. Commit `feat(model): templates and dates are edited like the log`.

### Task 2: One table for the Log and the popup (UI refactor)

`schedule.rs`: extract `log_table(ui, scene, view_state, rows, target, playing)`; `schedule_list` calls it with `schedule_view` and `EditTarget::Date` per row's date. Existing schedule UI tests keep passing. Commit `refactor(ui): the log's table takes its rows and target`.

### Task 3: The popup (`ui/plan_window.rs`)

A non-modal `egui::Window` like the library (`ViewState::plan_window: Option<EditTarget>`), title "Template · name" or the date, the table (`playing = false`) and a footer with Add (as the Log's, targeting the popup's target: Add puts items at the end of the last block). Tests (`plan_window_ui.rs`): `the_popup_lists_a_templates_blocks`, `add_block_in_the_popup_edits_the_template`, `remove_in_the_popup_edits_the_template`, `the_popup_closes_when_its_template_is_deleted`. Commit `feat(ui): templates and days open in a playlist-like popup`.

### Task 4: Settings → Automation, Templates and days

`settings/automation.rs`: a "Templates and days" part: the template list (select; Edit… opens the popup and closes Settings; New, Duplicate, Rename, Delete), the week plan (a combo per weekday), and a month calendar (dates with changes in amber; the selected date's template combo and Edit day…). Remove `schedule_editor.rs`, `ViewState::schedule_editor`, the Log's Edit log button and the strings only they used; the Log's strip shows today's date. Tests (`settings_automation_ui.rs` or the existing rotation editor test file): `edit_opens_the_popup_and_closes_settings`, `the_week_plan_sets_a_weekday_template`, `a_date_can_use_another_template`, `new_template_sends_create_template`. Commit `feat(ui): templates and days live in settings`.

### Task 5: Docs and screenshots script

User guide (Editing the log → templates in Settings), technical docs (`plan_window.rs`, `EditPlan`), the screenshot script's automation part (the editor shots now come from Settings and the popup), the old 2026-11-11 automation spec rewritten to match the rework (§3, §5.1, §8 replaced by references to the rework spec's rules). Commit `docs: templates and days in settings`.
