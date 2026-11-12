# Automation plan 6: the Schedule column (§8.1)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** the Schedule column shows the schedule grouped by block, with
the next item marked and missed items struck through, and holds
automation's Start, Stop after current and a clear running/stopped state;
the operator sets or clears next from the list.

**Architecture:** a pure `automation::schedule_view(state)` in `fp-model`
lists the rows (block headers and items, each with its status). The app
draws it in place of the schedule player's tabs and track table:
`ui/schedule.rs` holds the bar (Start, Stop after current, the state) and
the list. Every action is an existing command (`StartAutomation`,
`StopAutomationAfterCurrent`, `SetScheduleNext`, `ClearScheduleNext`).

**Tech Stack:** Rust, `fp-model`, `egui` / `egui_kittest`, Fluent.

**Spec:** `docs/superpowers/specs/2026-11-11-automation-design.md` (§3 terms,
§8.1, AU15–AU17, AU23).

## Global Constraints

- UI strings in every locale (en-US and es-ES written, the others
  translated); Spanish uses §3's terms (Parrilla, Bloque, Musicalizador,
  No emitido, Puntual…).
- The UI only reads the model and sends commands (rule 8); behaviour in
  `fp-model` (rule 7).
- No `unwrap`/`expect`/`panic` outside tests.
- Rule 10: nothing goes on air by itself; Start is the operator's.
- Nocturne theme tokens from `ui/theme.rs`; no new colours.

## Review Focus

- An empty schedule (no blocks today): the column says so, Start still
  works (the filler plays).
- A block with a deleted track or category: the row says "not available",
  never panics.
- Midnight: yesterday's last block and tomorrow's first show with their
  day, and statuses stay right when the date changes.
- A next set by hand on a block of tomorrow or a missed item.
- The list scrolls to the item on air (or next) once, not every frame.

---

### Task 1: The rows of the Schedule column

**Files:**
- Modify: `crates/fp-model/src/automation.rs`
- Test: `crates/fp-model/tests/it/automation_view.rs` (new, `mod` line)

**Produces:**
```rust
pub enum RowStatus { Aired, OnAir, Next, Missed, Coming }
pub enum ScheduleRow {
    Block { date: NaiveDate, block: BlockId, start: NaiveDateTime,
            origin: BlockOrigin, status: RowStatus },
    Item { item: ItemRef, what: Item, length_secs: Option<f64>,
           time_critical: bool, status: RowStatus },
}
pub fn schedule_view(state: &AppState) -> Vec<ScheduleRow>
```
- Range: yesterday's last block, every block of today, tomorrow's first
  block (the window's blocks in time order).
- Item status: `OnAir` for the item of the schedule player's current row
  (`rows`); `Missed` when in `missed`; `Next` when it is `next`; `Aired`
  for the items before the on-air one in the block on air, and for every
  non-missed item of a block that started and is no longer on air;
  `Coming` otherwise.
- Block status: `OnAir` when on air; `Next` when next is in it; `Missed`
  when it started never (passed) and every item is missed; `Aired` when it
  started and ended; `Coming` otherwise.
- `length_secs`: a track's play range length when known; `None` for a
  category, a rotation or an unknown length. `time_critical` from
  `Item::is_time_critical`.

- [ ] Tests: `the_view_lists_yesterdays_last_today_and_tomorrows_first`,
  `the_item_on_air_and_next_are_marked`,
  `missed_items_and_blocks_are_marked`,
  `aired_items_are_marked_after_their_block`,
  `a_next_set_by_hand_is_marked_where_it_is`.
- [ ] RED, implement, GREEN, full suite, commit
  `feat(model): the rows of the Schedule column`.

### Task 2: Start, Stop after current and the state

**Files:**
- Create: `crates/fp-app/src/ui/schedule.rs` (`pub(crate) fn schedule_bar`)
- Modify: `crates/fp-app/src/ui/player.rs` (`playlist_area`: the schedule's
  player draws `schedule_bar` instead of the tabs), `ui.rs` (mod line),
  every `locales/*/main.ftl`
- Test: `crates/fp-app/tests/it/schedule_ui.rs` (new, `mod` line)

Strings: `schedule-start` (Start / Iniciar), `schedule-stop-after-current`
(Stop after current / Detener tras la actual), `schedule-running`
(Running / En marcha), `schedule-stopped` (Stopped / Detenida),
tooltips `tip-schedule-start`, `tip-schedule-stop-after-current`.

- Start is enabled while stopped and sends `StartAutomation`; Stop after
  current is enabled while running and sends `StopAutomationAfterCurrent`.
- The state is a chip: green (`theme::GREEN`) "Running", neutral
  "Stopped".

- [ ] Tests: `start_sends_start_automation`,
  `stop_after_current_sends_its_command_while_running`,
  `the_state_says_stopped_after_a_start_of_the_app`.
- [ ] RED, implement, GREEN, full suite, commit
  `feat(ui): start and stop the schedule from its column`.

### Task 3: The schedule list

**Files:**
- Modify: `crates/fp-app/src/ui/schedule.rs` (`pub(crate) fn schedule_list`),
  `crates/fp-app/src/ui/player.rs` (the schedule's player draws
  `schedule_list` instead of `table::track_table`), locales
- Test: `crates/fp-app/tests/it/schedule_ui.rs`

- A block row: its start `HH:MM:SS`, "yesterday"/"tomorrow" when not
  today (`schedule-yesterday`, `schedule-tomorrow`), and
  `schedule-changed-for-date` for a changed or date-only block.
- An item row: the track's title and artist (`track_cells`' style), a
  category as `schedule-item-category` ("{ $name } (one of the
  category)"), a rotation as `schedule-item-rotation` ("Music: { $name }"),
  a missing one as `schedule-item-missing`; its length; the
  time-critical glyph with `schedule-time-critical` as tooltip.
- Status styles: on air and next as a player's rows (`theme` tokens of the
  track table); missed struck through and dimmed with tooltip
  `schedule-missed`; aired dimmed.
- Empty: `schedule-empty` ("No blocks today").
- Scrolls to the row on air, else next, once per change (egui temp memory
  of the last target).

- [ ] Tests: `the_list_shows_blocks_and_their_items`,
  `a_missed_item_is_struck_through` (its label carries the missed
  tooltip), `an_empty_schedule_says_so`,
  `a_deleted_track_shows_not_available`.
- [ ] RED, implement, GREEN, full suite, commit
  `feat(ui): the schedule list in its column`.

### Task 4: Setting next from the list

**Files:** `crates/fp-app/src/ui/schedule.rs`, locales; test in
`schedule_ui.rs`.

- Double-click on an item sends `SetScheduleNext(item)`.
- Its context menu: `schedule-set-next` ("Play next"), and on the item that
  is next by hand `schedule-clear-next` ("Back to the clock") sending
  `ClearScheduleNext`.

- [ ] Tests: `double_click_sets_the_item_as_next`,
  `the_menu_clears_a_next_set_by_hand`.
- [ ] RED, implement, GREEN, full suite, commit
  `feat(ui): set the schedule's next from its list`.

### Task 5: Docs

- `docs/user/automation.md`: a "The schedule" section (the column, Start,
  Stop after current, statuses, setting next; the editor comes later).
- `docs/technical/ui.md` (`ui/schedule.rs`), `docs/technical/automation.md`
  (`schedule_view`).
- README roadmap: "the schedule's column is built; the editor is next".
- Commit `docs: the Schedule column`.

Then the final review, fixes test-first, PR.
