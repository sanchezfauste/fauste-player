# Automation rework, plan 3: the Log and Music fill transport — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The Log and Music fill columns are driven by the player transport (Play, Pause, Stop, Fade stop, Stop at end, SINGLE/CONT) with the rules AR3–AR12; the Log column's own Start / Stop after current / Running controls go away.

**Architecture:** The model keeps `AutomationState::running` and the commands `StartAutomation` / `StopAutomationAfterCurrent` / `SetScheduleNext` / `ClearScheduleNext` as its internal API. The reducer routes the player transport commands aimed at the Log player (`AppState::is_schedule`) and the filler (`is_filler`) to new functions in `automation.rs` before the plain player handling. A new `AutomationState::stop_at_end` is the Log's Stop at end; stopping pins next to the row below (AR10). The Log header's view takes the filler's song, status and the automation's Stop at end while a music fill row is current.

**Tech Stack:** Rust; `fp-model` tests with `automation_rig::Rig` (explicit time, fake engine); `egui_kittest` UI tests.

**Spec:** `docs/superpowers/specs/2026-11-13-automation-rework-design.md` §4–§5 (AR3–AR13, AR6b, AR7, AR7b, AR11b, AR12).

## Global Constraints

- Rule 10: nothing goes on air by itself after a start, restart or crash; only Play on the Log (or the filler) starts anything.
- AU19–AU26 timing unchanged; the existing `automation_*` tests keep passing except where a rule here replaces theirs (each such test is rewritten, named in the task, and its change recorded as a ruling).
- No new config fields; new strings only if a label is needed (en-US, es-ES written, the others translated).
- No `unwrap`/`expect`/`panic` outside tests.

## Review Focus

- Play on the Log while the filler is **paused** for the Log: resumes the filler, `running` again (Task 3 test).
- Stop at end on the Log set while the fill plays and **unset** before the song ends: the block starts normally, nothing stops (Task 2 test).
- A Log in SINGLE whose block has a rotation item: after the last item, nothing else plays (Task 5 test).
- A clock jump while stopped with a pinned next: next stays (Task 1 test).
- The remote API and MIDI still cannot address the Log or the filler (existing tests; run `fp-remote` and `fp-control`).

---

### Task 1: Stopping pins next to the row below (AR10, AR12)

**Files:** Modify `crates/fp-model/src/automation.rs`; Test `crates/fp-model/tests/it/automation_next.rs`.

**Interfaces — Produces:**
- `pub fn row_below(state: &AppState, item: ItemRef) -> Option<ItemRef>`: the item at `index + 1` of the same block (a rotation item included), else the first item of the next block of the window in time order (any block, started or not).
- `fn pin_next_after_stop(state: &mut AppState)`: called whenever automation goes from running to stopped; sets `next` to the Log player's next row's item (via `rows`) when there is one, else `row_below(last item on air)`, else keeps the clock's next; sets `next_by_hand = true`.
- `AutomationState::last_on_air: Option<ItemRef>` (runtime only): the item of the Log's current row, kept by `settle_next` while a row is on air.

- [ ] **Step 1: Failing tests**
  - `stopping_after_an_item_leaves_next_on_the_row_below`: two blocks (10:00 `[song 1, song 2]`, 11:00 `[song 3]`); Start at 9:59, at 10:00 song 1 on air; `StopAutomationAfterCurrent`; `end_current(log)`; at 10:30: `next == item(10:00, 1)` and `next_by_hand`; at 10:45 still the same.
  - `a_pinned_next_survives_a_clock_jump`: as above, then `jump_hms(12, 0, 0)`: next unchanged.
  - `at_start_up_next_follows_the_clock` (AR12): a fresh rig at 10:30: next is `item(11:00, 0)`; at 11:30 next follows the clock (no block left today → tomorrow's first or `None`, as `clock_next` says).
- [ ] **Step 2:** `cargo test -p fp-model --test it automation_next` — the first two FAIL.
- [ ] **Step 3:** Implement `row_below`, `last_on_air`, `pin_next_after_stop`; call the latter where `running` turns false (Task 2–5 add more callers).
- [ ] **Step 4:** `cargo test -p fp-model --test it automation` — PASS (rewrite `stop_after_current_ends_the_item_and_stops` if it asserts `next` follows the clock; ruling).
- [ ] **Step 5: Commit** `feat(model): stopping automation keeps next on the row below`

### Task 2: The Log's Stop at end (AR6)

**Files:** Modify `automation.rs`, `reducer.rs` (`ToggleStopAfterCurrent` / `SetStopAfterCurrent` on the Log player); Test `crates/fp-model/tests/it/automation_run.rs`.

**Interfaces — Produces:** `AutomationState::stop_at_end: bool` (runtime only); `pub(crate) fn toggle_stop_at_end(state: &mut AppState)`; `StopAutomationAfterCurrent` sets it on (no longer clears `running` at once).

Rules: while set, a Log row on air has `stop_after_current`; a filler song on air for the Log has `stop_after_current` too. When the Log's row ends, or the filler's song ends before a hand-over, `running = false`, `stop_at_end = false`, `pin_next_after_stop`. When a hand-over to a block comes first, the filler's mark is cleared, the block starts on time and its first row gets `stop_after_current`. Unsetting clears the marks it set. Toggling while not running does nothing (`Ok`).

- [ ] **Step 1: Failing tests**
  - `stop_at_end_on_an_item_stops_after_it`: as Task 1's first test but via `Command::ToggleStopAfterCurrent(log)`; `running` stays true until `end_current`, then false; filler silent at 10:03.
  - `stop_at_end_during_music_lets_the_song_end_then_stops`: Start at 10:30 (fill plays); toggle on the Log; `end_current(filler)`: filler silent, `!running`, next == `item(11:00, 0)`.
  - `stop_at_end_during_music_moves_to_the_block_when_it_comes_first`: Start at 10:58:30 (songs are 180 s); toggle on the Log; at 11:00:00 song 3 on air on the Log with `stop_after_current`; `end_current(log)` → `!running`.
  - `unsetting_stop_at_end_lets_automation_go_on`: as the previous, toggle twice; at 11:00 song 3 plays without `stop_after_current`, `running`.
  - Rewrite `stop_after_current_ends_the_item_and_stops` and `stop_after_current_lets_the_filler_song_end` for the new timing (running until the end); ruling.
- [ ] **Step 2:** run `automation_run` — new tests FAIL.
- [ ] **Step 3:** Implement.
- [ ] **Step 4:** `cargo test -p fp-model --test it automation` — PASS.
- [ ] **Step 5: Commit** `feat(model): the log's stop at end is the player's`

### Task 3: Play, Pause, Stop and Fade stop on the Log (AR3–AR5, AR7b)

**Files:** Modify `automation.rs`, `reducer.rs` (`Play`, `Pause`, `Stop`, `FadeStop`, `Previous` when `is_schedule(id)`); Test new `crates/fp-model/tests/it/automation_transport.rs` (+ `mod` line).

**Interfaces — Produces:** `pub(crate) fn log_play(state, out) -> Result<(), ModelError>`, `log_pause`, `log_stop(state, fade: bool, out)`; `fn fill_is_current(state) -> bool` (running, no block on air, filler playing or paused).

Rules:
- Play: a paused Log row resumes (plain player play) and `running = true`; else a filler paused while the fill is current resumes and `running = true`; else while running a Log row on air → plain player Play (next with crossfade); else `start` (existing: a pinned next plays at once with the filler faded over `block_fade_ms`; a clock next with a later start plays music until it, AU17).
- Pause: pauses the Log row on air, else the filler when the fill is current; `running = false` (no pin: the row stays current).
- Stop / Fade stop: stop or fade-stop the Log row on air, else the filler when the fill is current; `running = false`; `pin_next_after_stop`.
- Previous on the Log: `Ok`, no change (the header disables it, Task 6).
- When automation is stopped and the filler plays on its own, the Log's Pause/Stop/Fade stop do nothing to it.

- [ ] **Step 1: Failing tests** (Rig, two blocks as Task 1):
  - `play_on_the_log_starts_automation` (= `start_with_a_coming_block_plays_music_until_it` via `Command::Play(log)`).
  - `play_on_the_log_plays_a_pinned_next_at_once`.
  - `stop_on_the_log_stops_the_item_and_automation`, `fade_stop_on_the_log_…` (FadeOutAndStop action for the Log), `stop_on_the_log_during_music_stops_the_music`.
  - `pause_and_play_on_the_log_resume_the_item`, `pause_on_the_log_during_music_pauses_the_filler_and_play_resumes_it` (Review Focus).
  - `a_block_start_passing_while_paused_is_missed` (AR3).
  - `log_stop_leaves_a_filler_playing_on_its_own_alone`.
- [ ] **Step 2:** run `automation_transport` — FAIL.
- [ ] **Step 3:** Implement and route in `apply_command`.
- [ ] **Step 4:** `cargo test -p fp-model --test it` — PASS.
- [ ] **Step 5: Commit** `feat(model): the log's transport runs automation`

### Task 4: The Music fill transport (AR7)

**Files:** `automation.rs`, `reducer.rs` (commands on `is_filler(id)`); Test `automation_transport.rs`.

Rules: Pause / Stop / Fade stop on the filler while the fill is current for a running automation also set `running = false` (+ pin). The filler's own Stop at end, while the fill is current: if its song ends first, the filler stops and automation stops (+ pin); if a hand-over comes first, its mark is cleared and the block plays. Implemented by remembering, in the hand-over, to clear `stop_after_current` on the filler unless `stop_at_end` (Task 2) or `filler_held` set it, and by a `fill_stopped_by_operator` check in `drive`: the filler went from playing to stopped while the fill was current, no hand-over or hold was pending → stop automation.

- [ ] **Step 1: Failing tests:** `stopping_the_filler_during_music_stops_automation`, `the_fillers_stop_at_end_stops_automation_when_its_song_ends_first`, `the_fillers_stop_at_end_is_cleared_when_a_block_comes_first`, `play_on_the_filler_while_a_block_plays_mixes` (both playing, `running` unchanged).
- [ ] **Steps 2–4** as above; **Step 5: Commit** `feat(model): the music fill's transport follows the log`

### Task 5: SINGLE on the Log (AR6b)

**Files:** `automation.rs` (`drive`, `restore`), `reducer.rs` (`SetMode` allowed on the Log); Test `automation_transport.rs`.

Rules: Log in `PlayMode::Single`: Play plays next (pinned or clock, at once, a block item loaded by hand); when the row ends the Log player stops by its own Single rule, then `running = false` + pin; no block starts by itself; Play on a music fill row starts the filler with that rotation, and automation stays stopped. `restore` keeps the saved mode (no forced Continuous). Stop at end on a Single Log follows the players' rule (`StopAfterInSingle`).

- [ ] **Step 1: Failing tests:** `single_plays_one_item_and_stops`, `single_never_starts_a_block_by_itself`, `single_play_on_a_music_fill_row_starts_the_filler_only`, `the_log_mode_survives_a_restart`.
- [ ] **Steps 2–4**; **Step 5: Commit** `feat(model): the log has single and continuous modes`

### Task 6: The Log header is a player's (UI)

**Files:** `crates/fp-app/src/ui/schedule.rs` (`schedule_bar`: Start, Stop after current and the running label go; Edit log stays until plan 5), `crates/fp-app/src/ui/view.rs` (`player_view` for the Log while the fill is current: the filler's title/artist/times/status and `stop_after = stop_at_end`), `crates/fp-app/src/ui/player.rs` (Previous disabled on the Log; Stop at end enabled on the Log while running); locales (remove `schedule-start`, `tip-schedule-start`, `schedule-stop-after-current`, `tip-schedule-stop-after-current`, `schedule-running`, `schedule-stopped` from all eleven); docs `docs/user/automation.md` (the transport section), `docs/technical/automation.md`, `docs/technical/ui.md`; Test `crates/fp-app/tests/it/schedule_ui.rs`.

- [ ] **Step 1: Failing tests:** `the_log_has_no_start_button` (no "Start"/"Stop after current" labels), `play_on_the_log_sends_play` (header Play click → `Command::Play(log)` in the Fake), `during_music_the_log_header_shows_the_song` (rig state with the fill current → the header shows the filler's title and "On air").
- [ ] **Steps 2–4:** `cargo test -p fp-app --test it schedule` — PASS; whole suite green.
- [ ] **Step 5: Commit** `feat(ui): the log column is driven by its player header`
