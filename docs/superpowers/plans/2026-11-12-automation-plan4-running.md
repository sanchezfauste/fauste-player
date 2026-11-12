# Automation plan 4 — Running the schedule Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Automation plays the schedule. It covers:
- the schedule's own hidden player (the future Schedule column);
- running and stopped, and next from the clock or set by hand;
- Start and Stop after current;
- blocks played in order, with music from the filler between them;
- missed blocks and items, and the time-critical items that arrive too late;
- the hand-over from the filler to a block on time;
- clock jumps.

This plan adds no screens; the editor plan adds the Schedule column.

**Architecture:**
- **Model:** a new pure module `fp-model/src/automation.rs` holds
  `AutomationState` (in `AppState::automation`) and the rules.
- **The schedule player:** a `PlayerState` in continuous mode whose
  playlist holds the rows of the block being played. A row is an item
  resolved to a track. The player's existing machinery plays the rows in
  order, with crossfades and segues (AU18).
- **The tick:** the conductor sets `AppState::local_now` (local
  date-time) next to `wall_now` every tick, and calls
  `automation::tick(state)`. `tick` compares local times with block starts,
  starts blocks, hands over from the filler and marks missed items.
- **Commands:** `StartAutomation`, `StopAutomationAfterCurrent`,
  `SetScheduleNext` and `ClearScheduleNext`.
- **Persistence:** the schedule player and its playlist ids are saved, so
  their outputs and layout survive a restart. The running state, next and
  missed marks are not saved (AU15, AU16).

**Tech Stack:** Rust 2024, chrono (`NaiveDateTime`), the existing reducer,
selection and filler.

**Spec:** `docs/superpowers/specs/2026-11-11-automation-design.md` (AU15–AU24,
AU26, AU27's "or the next block", AU29's hand-over, AU40, AU41).

**Scope and deferrals** (ledger rulings):
- **Timing:** the hand-over is exact to one conductor tick
  (`tuning.conductor_tick_ms`, 5 ms by default), not to the sample. A
  sample-exact start across two players needs a new engine feature (a start
  at a given instant). That goes to plan 4b, with AU25 (no fragments: fitting
  the last songs before a block), which needs a length-aware pick. The
  maintainer will be asked whether tick precision is enough.
- **Local time:** the model compares local date-times (`local_now` against
  a block's date and `TimeOfDay`). A block in the spring gap is never
  reached, so it is missed. In the autumn overlap a block plays once, because
  the `started` set keeps it from playing again. This meets AU14 without a
  zone in the model. The fade lead and the tolerance are measured in local
  seconds; they can be off by an hour only inside the changing hour.
- **Missed marks** are kept in memory for yesterday, today and tomorrow. The
  as-run log (AU36) and its pruning are plan 6.

## Global Constraints

- CLAUDE.md rules apply:
  - English everywhere in code, docs and commits.
  - No unwrap/expect/panic outside tests; prefer `get`.
  - Rule 4: new values are `Config` fields.
  - Rule 7: behaviour lives in `fp-model` as pure functions, with time passed
    in.
  - Rule 8: the UI never blocks.
  - Rule 9: damaged data degrades with a warning.
  - **Rule 10: nothing goes on air by itself.** Automation is stopped after
    every start, and only `StartAutomation` makes it run.
- New config fields go in `AutomationConfig`, each with a default, a clamp
  and lenient loading:
  - `automation.late_tolerance_secs`: default 120, range 0–1 800.
  - `automation.block_fade_ms`: default 2 000, range 0–10 000.
- Every new `ModelError` is mapped in `ui/app.rs`, with a message in all
  11 locales (en-US and es-ES written, the others translated).
- Tests go in `crates/<crate>/tests/it/`. Commits pass fmt, clippy
  `-D warnings` and the whole suite, and end with the Co-Authored-By trailer.
- Never name other products.

## Review Focus

1. **Restart while running:** the state is saved while automation runs and
   a block plays, then loaded. Automation is stopped, nothing plays, and next
   follows the clock (Task 6,
   `after_a_restart_automation_is_stopped_and_silent`).
2. **A block with nothing playable:** all its tracks are missing, or its
   category is empty. The block is passed over at once as aired-empty, with
   its items marked missed. Automation never stalls and never loops (Task 3,
   `a_block_with_nothing_playable_moves_on`).
3. **The clock jumps back an hour while running:** blocks that already
   started do not play again, and next is recomputed (Task 5,
   `a_backward_jump_replays_nothing`).
4. **Midnight:** a block at 23:30 overruns past 00:00. The 00:00 block of the
   next date follows in order (AU21 across dates). The day lists cover
   yesterday, today and tomorrow (Task 4, `an_overrun_chains_across_midnight`).
5. **The operator edits the schedule while a block plays:** the item on air
   is not cut. Edits apply to rows not yet started the next time a block
   loads (Task 3, `editing_the_schedule_never_cuts_the_item_on_air`).

---

### Task 1: The schedule player and automation state

**Files:**
- Create: `crates/fp-model/src/automation.rs`
- Modify:
  - `crates/fp-model/src/state.rs`: `automation: Option<AutomationState>`,
    `local_now: chrono::NaiveDateTime`, and the player guards;
  - `crates/fp-model/src/lib.rs`, `crates/fp-model/src/reducer.rs`: the guards;
  - `crates/fp-remote/src/api.rs`: the guards;
  - `crates/fp-app/src/ui/*`: wherever `is_filler`/`is_filler_playlist`
    stand for "not an operator player/list".
- Test: `crates/fp-model/tests/it/automation_player.rs`

**Interfaces:**
- Produces, in `automation.rs`:
  - `pub struct ItemRef { pub date: NaiveDate, pub block: BlockId, pub index: usize }`,
    with `Ord` and `Hash`.
  - `pub struct BlockRun { pub date: NaiveDate, pub block: BlockId, pub start: NaiveDateTime, pub in_order: bool, pub rotation: Option<RotationId> }`.
  - `pub struct AutomationState`:
    - `player: PlayerId`, `playlist: PlaylistId`;
    - `running: bool`;
    - `next: Option<ItemRef>` and `next_by_hand: bool`;
    - `on_air: Option<BlockRun>`;
    - `rows: HashMap<EntryId, ItemRef>`;
    - `started: BTreeSet<(NaiveDate, BlockId)>`;
    - `missed: BTreeSet<ItemRef>`;
    - `handing_over: bool`.
  - `pub fn ensure(state: &mut AppState, playlist_name: &str) -> Vec<EngineAction>`,
    modelled on `filler::ensure`:
    - a continuous-mode player inserted just before the filler's (or last);
    - an empty playlist;
    - the player hidden in the layout.
- Produces, on `AppState`:
  - `is_automation(player)`: the schedule's or the filler's player;
  - `is_automation_playlist(list)`;
  - `aux_players()` now excludes both automation players;
  - `player_ids()` gives the schedule player first, then the filler, then
    P1….
- Rule: every guard that today refuses operator edits to the filler's
  list (`FillerPlaylist`) also refuses them for the schedule's list, with the
  same error. The remote API's 404 covers both.

- [ ] **Step 1: Write the failing tests:**
  - `ensure_adds_a_hidden_schedule_player_once`: called twice, it adds one player, which is hidden and not in `aux_players`. `player_ids()[0]` is the schedule player and `player_ids()[1]` the filler.
  - `the_schedule_list_is_not_an_operator_list`: RenamePlaylist and DeletePlaylist on it, and ShowPlaylist of it on P1, each give `Err(FillerPlaylist)`.
  - `automation_starts_stopped`: `running` is false and `next` is None.
- [ ] **Step 2: Run** `cargo test -p fp-model --test it automation_player::`. Expected: does not compile.
- [ ] **Step 3: Implement.** Add `local_now` (default `NaiveDateTime::default()`). Replace `is_filler`/`is_filler_playlist` with `is_automation`/`is_automation_playlist` at the guard sites. Keep `is_filler` where the code means the filler alone (`filler_bar`, the top-up).
- [ ] **Step 4: Run** the same command, then `cargo test --workspace`. Expected: PASS.
- [ ] **Step 5: Commit** `feat(model): the schedule's player`.

### Task 2: Next from the clock, set by hand (AU16, AU23)

**Files:** `crates/fp-model/src/automation.rs`, `command.rs`, `reducer.rs`, `config.rs`; Test: `crates/fp-model/tests/it/automation_next.rs`

**Interfaces:**
- Produces:
  - `pub fn clock_next(state: &AppState) -> Option<ItemRef>`:
    - Look at yesterday's, today's and tomorrow's blocks
      (`Schedule::day`), in time order.
    - The first block that is not in `started` and whose start is after
      `local_now`, or within `late_tolerance_secs` before it, gives its
      first item (index 0).
    - A block whose start is further in the past is not chosen.
  - `pub fn settle_next(state: &mut AppState)`: while `!next_by_hand`, sets
    `next = clock_next`. While stopped, the blocks passed over (started in
    the past and not in `started`) get all their items marked missed.
  - Commands:
    - `SetScheduleNext(ItemRef)` sets `next_by_hand`. It is refused with
      `UnknownBlock` when the date and block are not in yesterday, today or
      tomorrow, or with the new `ModelError::UnknownItem` when the index is
      past the block's items.
    - `ClearScheduleNext` gives next back to the clock.
  - `AutomationConfig::late_tolerance_secs: u32`: default 120, range 0–1 800.
  - `pub fn block_items(state, date, block) -> Option<Vec<Item>>`: a helper
    over `Schedule::day`.

- [ ] **Step 1: Write the failing tests.** Base has blocks at 10:00 (2 items) and 11:00 (1 item); `local_now` is set directly.
  - `next_is_the_coming_block`: at 10:58, next is (today, 11:00, 0).
  - `a_block_just_started_is_still_next`: at 11:01 with tolerance 120, next is the 11:00 block; at 11:03 it is tomorrow's 10:00.
  - `blocks_passed_while_stopped_are_missed`: at 11:30, `settle_next` marks both 10:00 items and the 11:00 item missed.
  - `a_hand_set_next_stays`: SetScheduleNext to (today, 10:00, 1) at 11:30, then `settle_next` at 11:45: next is unchanged. ClearScheduleNext gives the clock's.
  - `next_crosses_midnight`: at 23:59 with the last block at 11:00, next is tomorrow's 10:00.
  - `set_next_checks_the_item`: index 5 gives `Err(UnknownItem)`; a date 10 days away gives `Err(UnknownBlock)`.
  - `late_tolerance_is_clamped`: 5 000 becomes 1 800.
- [ ] **Step 2: Run** `cargo test -p fp-model --test it automation_next::`. Expected: does not compile.
- [ ] **Step 3: Implement.** `reconcile` calls `settle_next` after each command and event. Map `UnknownItem` to `error-not-found`.
- [ ] **Step 4: Run** the same command. Expected: PASS.
- [ ] **Step 5: Commit** `feat(model): the schedule's next item`.

### Task 3: Start, play a block, stop after current (AU15, AU17, AU18, AU26)

**Files:** `crates/fp-model/src/automation.rs`, `command.rs`, `reducer.rs`; Test: `crates/fp-model/tests/it/automation_run.rs`

**Interfaces:**
- Consumes: Task 2.
- Produces:
  - `pub fn load_block(state: &mut AppState, from: ItemRef, in_order: bool, out: &mut Vec<EngineAction>)`:
    1. Replaces the schedule playlist's rows that have not started (never
       the current row) with one row per item from `from.index`:
       - a Track item becomes its track;
       - a Category item becomes `selection::pick` over
         `StepSource::Category` at `wall_now`;
       - a Rotation item ends the rows and is kept as `BlockRun::rotation`.
    2. Skips items with no playable track, marking them missed.
    3. Fills `rows`, sets `on_air` and inserts into `started`.
    4. Sets the schedule player's next to the first new row.
  - `pub fn tick(state: &mut AppState) -> Vec<EngineAction>`: the
    per-tick rules (this task: start a block when it is due).
  - Commands:
    - `StartAutomation`: `running = true`. If `next_by_hand`, load that
      block and play it now. Else, if next's block start is due, play it now;
      if it is later, play the filler until then (Task 4's hand-over, or Play
      on the filler).
    - `StopAutomationAfterCurrent`: `running = false`. Sets
      `stop_after_current` on whichever of the schedule player and the filler
      is playing, and clears `next_by_hand`.
- Rules:
  - While running, `tick` starts the next block at its start.
  - After a block ends, `tick` moves on per Task 4.
  - A row marked aired records its track's air time (AU35 already does it
    for any on-air start).

- [ ] **Step 1: Write the failing tests** (a helper drives `apply`, `on_event` with `TransitionStarted`/`ReachedEnd`, and `tick` at set times):
  - `start_with_a_coming_block_plays_music_until_it`: at 10:58, StartAutomation plays the filler and not the schedule player. At 11:00 `tick` starts the schedule player with the 11:00 block's first row.
  - `start_with_a_hand_set_next_plays_it_now`: SetScheduleNext(10:00, 0) at 11:30, then Start plays that row at once.
  - `a_block_plays_its_items_in_order`: the two rows of 10:00 play one after the other, as the player's next.
  - `stop_after_current_ends_the_item_and_stops`: during a schedule item, StopAutomationAfterCurrent; at ReachedEnd nothing plays and the filler is not started.
  - `a_category_item_picks_a_track_when_its_block_loads`: one row whose track is in the category.
  - `a_block_with_nothing_playable_moves_on` (Review Focus 2).
  - `editing_the_schedule_never_cuts_the_item_on_air` (Review Focus 5): an UpdateBlock during the first row sends no StopNow or Crossfade for the schedule player.
  - `nothing_plays_by_itself`: with `running` false, ticks past several block starts produce no StartCurrent.
- [ ] **Step 2: Run** `cargo test -p fp-model --test it automation_run::`. Expected: does not compile.
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run** the same command. Expected: PASS.
- [ ] **Step 5: Commit** `feat(model): start and stop the schedule`.

### Task 4: After a block, missed blocks, late time-critical items, the hand-over (AU19–AU22, AU24, AU27)

**Files:** `crates/fp-model/src/automation.rs`, `filler.rs`, `config.rs`; Test: `crates/fp-model/tests/it/automation_blocks.rs`

**Interfaces:**
- Consumes: Task 3.
- Produces:
  - `AutomationConfig::block_fade_ms: u32`: default 2 000, range 0–10 000.
  - `pub fn coming_start(state: &AppState) -> Option<NaiveDateTime>`: the
    start of next's block. The filler's top-up uses it, so the look-ahead
    stops at the coming block (AU27).
- Rules in `tick`:
  - **A block ends** (the schedule player stopped, `on_air` set, running):
    - If `on_air.in_order` and some block started after `on_air.start` and
      at or before now, the earliest such block loads now with
      `in_order: true` (AU21). Its time-critical items whose start would be
      later than block start + tolerance are marked missed and left out
      (AU22).
    - Otherwise next = the first block not yet started by the clock (AU19),
      and the blocks between are marked missed. The filler plays
      `on_air.rotation`, else the default rotation (AU20), through
      `filler::set_rotation` and Play when stopped.
  - **The hand-over (AU24):** when `now >= start - block_fade_ms` and the
    filler is playing, send `FadeOutAndStop { filler, fade_ms }`, where
    `fade_ms` is the time left to the start, at most `block_fade_ms`. Set
    `handing_over`. When `now >= start`, load the block (`in_order: true`)
    and Play the schedule player.
  - **A late time-critical row** (AU22): before the schedule player moves
    on, a next row that is time-critical and would start after block start
    + tolerance is removed and marked missed. Nothing is cut.
  - **No cutting (AU26):** the schedule player is never faded by automation.

- [ ] **Step 1: Write the failing tests:**
  - `after_a_block_music_until_the_next`: the 10:00 block ends at 10:20, the filler plays, and the 11:00 block starts at 11:00.
  - `the_filler_fades_out_before_the_block`: at 10:59:58 (`block_fade_ms` 2 000) the filler gets `FadeOutAndStop { fade_ms: 2000 }`. At 11:00:00 the schedule player starts.
  - `a_block_played_by_hand_late_skips_to_the_next_unstarted` (the maintainer's example): 10:00 is played by hand at 11:00 and ends at 11:55. Music runs until 12:00, then 12:00 plays. The 11:00 block is missed.
  - `an_overrun_plays_the_next_block_when_it_ends`: 11:00 (in order) runs to 12:01. The 12:00 block plays at 12:01, and is not missed.
  - `a_late_hourly_id_is_skipped`: an overrun to 12:03 with tolerance 120. The 12:00 block's Hourly ID item is missed and its second item plays.
  - `an_overrun_chains_across_midnight` (Review Focus 4).
  - `the_rotation_item_sets_the_filler_rotation`: a block ending in a Rotation item makes the filler play that rotation afterwards; with none, the default.
  - `block_fade_ms_is_clamped`.
- [ ] **Step 2: Run** `cargo test -p fp-model --test it automation_blocks::`. Expected: does not compile.
- [ ] **Step 3: Implement.** Change the filler's top-up to stop at `coming_start` while automation runs.
- [ ] **Step 4: Run** the same command, then `cargo test -p fp-model --test it filler::`. Expected: PASS.
- [ ] **Step 5: Commit** `feat(model): blocks in time order, with music between them`.

### Task 5: The filler on its own, clock jumps (AU29, AU40, AU41)

**Files:** `crates/fp-model/src/automation.rs`; Test: `crates/fp-model/tests/it/automation_clock.rs`

**Rules:**
- **Start while the filler plays (AU29):** StartAutomation while the filler
  already plays hands it to the schedule. It keeps playing until the coming
  block, and nothing restarts.
- **Clock jumps (AU40):** `tick` keeps the last `local_now` it saw. A
  backward step, or a forward step over more than 60 s of local time
  (`CLOCK_JUMP_SECS`, a detection threshold rather than an operator value),
  is a jump:
  - next is recomputed (unless set by hand);
  - after a forward jump, blocks jumped over are marked missed;
  - after a backward jump, `started` keeps aired blocks from playing again.
- **Other players (AU41):** automation does not watch the auxiliary
  players.

- [ ] **Step 1: Write the failing tests:**
  - `starting_automation_keeps_the_filler_song`: with the filler playing, StartAutomation sends nothing to the filler.
  - `a_backward_jump_replays_nothing` (Review Focus 3).
  - `a_forward_jump_misses_the_skipped_blocks`.
  - `aux_players_do_not_delay_a_block`: P1 playing does not stop the 11:00 block from starting at 11:00.
- [ ] **Step 2: Run** `cargo test -p fp-model --test it automation_clock::`. Expected: does not compile, then FAIL.
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run** the same command. Expected: PASS.
- [ ] **Step 5: Commit** `feat(model): automation and clock jumps`.

### Task 6: Persistence, start-up and the conductor

**Files:**
- Modify:
  - `crates/fp-model/src/automation.rs` and `session.rs`:
    `AutomationSession { player: PlayerSession, playlist: PlaylistId }` and
    the restore;
  - `crates/fp-store/src/db/snapshot.rs`: the `kv` value `automation`;
  - `crates/fp-store/src/store.rs`;
  - `crates/fp-engine/src/conductor.rs`: `local_now` and `tick`;
  - `crates/fp-app/src/session.rs`: `automation::ensure`;
  - the locales: `schedule-playlist-name`.
- Test:
  - `crates/fp-store/tests/it/automation_store.rs`: add a test;
  - `crates/fp-model/tests/it/automation_run.rs`: add a test;
  - `crates/fp-engine/tests/it/conductor.rs`: add a test.

**Interfaces:**
- `AppState::automation_session() -> Option<AutomationSession>`. The list is
  saved empty (its rows are rebuilt), and the player is restored stopped.
- `RestoreParts::automation: Option<AutomationSession>`.
- The conductor:
  - sets `state.local_now = chrono::Local::now().naive_local()` next to
    `wall_now`;
  - calls `fp_model::automation::tick(&mut self.state)` after the commands;
  - executes the actions, and marks `changed` when there are any or when
    `tick` reports a change. `tick` returns `(Vec<EngineAction>, bool)`.

- [ ] **Step 1: Write the failing tests:**
  - `the_schedule_player_round_trips_stopped`: after save and load, the same player and playlist ids, an empty list, not playing.
  - `after_a_restart_automation_is_stopped_and_silent` (Review Focus 1).
  - `the_conductor_ticks_automation`: an Offline-backend conductor with a schedule whose block is due and automation started. The schedule player is Playing within one tick.
- [ ] **Step 2: Run** each, and watch it fail.
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run** `cargo test --workspace`. Expected: PASS.
- [ ] **Step 5: Commit** `feat(engine): drive automation from the conductor`.

### Task 7: Documentation

- `docs/technical/automation.md`: a "Running the schedule" section with the
  state, the tick, the hand-over at tick precision and its rulings.
- `docs/technical/persistence.md`: the `kv` value `automation`, and the two
  new config fields.
- `docs/technical/ui.md`: `player_ids` order and the guards.
- `docs/technical/remote-api.md`: the schedule player is not in the API.
- Run `docs-sync-checker`. Commit `docs: running the schedule`.
