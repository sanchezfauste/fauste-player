# Feedback Plan 7 — Track Table: Proportional Columns and Follow Current Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Track-table columns keep their proportions as the window resizes
and fill the table, giving Title the most room (F6). The table follows the
current entry unless the operator is using it (F18).

**Architecture:**
- `fp-model`: `ColumnWidths` becomes `{ fractions: Option<[f32; 4]> }`,
  normalised by the model. `None` is the default layout. Old pixel fields
  are ignored.
- `fp-app`:
  - a pure `view::column_px(fractions, width, number_min, duration_min) ->
    [f32; 4]`;
  - `table.rs` resets egui's table state whenever the width or the stored
    fractions change, and stores fractions when a resize handle is released;
  - `ViewState` records per player the last table interaction and a pending
    follow;
  - `track_table` scrolls to the current row with `scroll_to_row`, and
    `player.rs` switches the tab with `ShowPlaylist`.
- `ui.follow_current_grace_secs = 0` **disables following** (spec §2.3).
  This also changes the waveform follow from plan 5, where 0 followed at
  once, so both views read the field the same way.

**Tech Stack:** Rust 2024, egui_extras 0.36 `TableBuilder`, egui_kittest.

**Spec:** [`docs/superpowers/specs/2026-09-30-operator-feedback-design.md`](../specs/2026-09-30-operator-feedback-design.md) §2.3. Roadmap: plan 7.

## Global Constraints

- Fractions sum to 1. NaN, negative or all-zero fractions fall back to the
  default layout.
- `#` and Duration have minimum pixel widths from their content. Title and
  Artist default to 60:40 of the rest.
- Following waits while a drag or a context menu is active in that player,
  then applies the grace from the moment it ended. Interaction times are UI
  view state.
- English. Both locales if any string is added (none planned). Gate before
  every commit. Branch: `feat/table-follow`.

## Review Focus

- **A very narrow table** (a player at 380 px): the columns never go
  negative or overlap, and the minimums win (Task 1 test).
- **An old `session.json` with pixel widths**: it loads the default layout
  (Task 1 test).
- **A current entry removed or moved** before the follow runs: nothing
  panics, and no scroll goes to a stale index (Task 3 test).
- **Two players showing the same playlist**: each follows its own current
  (Task 3 test).
- **Grace 0**: nothing follows, neither the table nor the waveform (Task 3
  test and the updated waveform test).

---

### Task 1: Fractional `ColumnWidths` and `column_px`

- `fp_model::ColumnWidths { #[serde(default)] pub fractions: Option<[f32; 4]> }` with `pub fn normalized(self) -> Self` (sanitises; sums to 1); `Command::SetColumnWidths` stores the normalised value. Default: `None`.
- `fp_app::ui::view::column_px(fractions: Option<[f32; 4]>, width: f32, number_min: f32, duration_min: f32) -> [f32; 4]`: sums to `width`; minimums for `#` (and Duration) win; Title and Artist at least 60 px each when the width allows; default = minimums for `#`/Duration and 60:40 of the rest.
- Tests: normalisation (sum 1, NaN/negative/zero → None); JSON `{"number":40,"title":300,"duration":52}` loads as `None`; `column_px` default split, proportional scaling (same fractions at two widths give proportional Title/Artist), minimums at 380 px, a fraction set that sums ≠ 1 is normalised.

### Task 2: The table uses fractions

- `track_table`: `let px = column_px(p.columns.fractions, ui.available_width(), number_min, duration_min)`; columns `initial(px[i])` (Artist stays `remainder()` so the table always fills); keep `ViewState.table_layout: HashMap<PlayerId, (f32, Option<[f32; 4]>)>` and call `TableBuilder::reset()` when the width changed by more than 0.5 px or the stored fractions changed, unless a resize is in progress.
- `store_widths`: on release, send `SetColumnWidths(ColumnWidths { fractions: Some(widths / sum) })`.
- Test accessor `AppUi::column_widths(player) -> Option<[f32; 4]>` (reads `ViewState.widths`).
- Tests (kittest): at 1000 px the Title column is wider than Artist and the four widths fill the table; after `set_size` to 1400 px each column's share is within 1 % of before; a stored fraction set is applied; dragging a resize handle and releasing sends one `SetColumnWidths` whose fractions sum to 1.

### Task 3: Follow the current entry

- `ViewState`: `table_touched: HashMap<PlayerId, f64>` (last scroll over the table, entry drag, open row menu, tab click), `followed: HashMap<PlayerId, Option<EntryId>>` (the current the table last followed or saw), `follow_scroll: HashMap<PlayerId, EntryId>` (a scroll waiting for its playlist to be shown).
- In `player::show` (per player, every frame): when `current` differs from `followed`, record it; if grace is 0, do nothing more; if a drag or the row menu is active, set `table_touched = time` and keep waiting; else if `time − table_touched ≥ grace` (or never touched): send `ShowPlaylist` when the current's playlist is not shown, and put the entry in `follow_scroll`.
- `track_table`: when `follow_scroll` holds an entry of the shown playlist, `scroll_to_row(index, Some(Align::TOP))` once and remove it; an entry no longer in the playlist is dropped.
- Waveform follow: grace 0 means never follow (update the plan 5 check and its test to use a small positive grace).
- Tests (kittest, 1 player, 200 entries, the Fake applying commands): a new current at entry 150 with no interaction makes "Song 151" the top row; a wheel scroll over the table just before prevents it; a current in another playlist sends `ShowPlaylist` and then scrolls; the last entry scrolls as far as the list allows (its row visible); grace 0 never scrolls; a removed current entry drops the pending scroll; two players following their own currents.

### Task 4: Docs, review, PR

Main spec §8.3 (track table: proportional columns, follow), `docs/user/playlists.md`, `docs/technical/ui.md`, `persistence.md` (`columns.fractions`; grace 0 disables). Review, PR `feat(ui): proportional playlist columns and follow the current track`, CI, merge.
