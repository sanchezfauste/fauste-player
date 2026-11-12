# Automation plan 8: the as-run log and the long-run test (AU36, AU38, AU39)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** record every scheduled item and filler track that aired, and
every scheduled item that was missed, with its date, scheduled time,
actual start and outcome. Keep the record in the database for
`automation.as_run_keep_days`. Prove with a 30-day simulated run that
nothing grows without bound and every block starts on time.

**Architecture:**
- **The model keeps a window.** `AppState::as_run` (an `AsRunLog`) holds
  only the entries from yesterday and today, each with a sequence number,
  plus the next sequence number. Entries are recorded in
  `reducer::settle_all` by comparing what the two automation players have
  on air, and the `missed` set, with what the log saw last.
- **The store keeps the record.** It appends the entries whose sequence is
  above the last one it saved, in schema 5's `as_run` table. At start-up
  it deletes rows older than `as_run_keep_days` (AU38), and it loads only
  the next sequence number.
- **Nothing else.** Viewing the log beyond the Schedule column, and
  exporting it, are non-goals (AU36).

**Tech Stack:** Rust, `fp-model`, `fp-store` (rusqlite).

**Spec:** `docs/superpowers/specs/2026-11-11-automation-design.md` (AU36,
AU38, AU39, §10, §11).

## Global Constraints

- `automation.as_run_keep_days`: default 90, range 1–3 650; it needs a
  clamp and lenient loading.
- The schema migration is append-only: migration 5 is added and
  `SCHEMA_VERSION` becomes 5.
- Damaged rows degrade to a log line (rule 9), never a crash.
- No `unwrap`, `expect` or `panic` outside tests.

## Review Focus

- **A crash between an entry and the next save.** The entry is lost, and
  that is acceptable: the save follows every change. It must never be
  written twice.
- **The clock jumps back.** Entries keep their own times. Sequence
  numbers, not times, decide what is new.
- **The same item airs twice** (a missed item played by hand). The log
  holds two entries, one Missed and one Aired.
- **Long-run memory.** The in-memory window holds yesterday and today
  only, whatever the keep days are.
- **Pruning at start-up.** It deletes only rows older than the keep days,
  measured from the local date.

---

### Task 1: The setting and the entries

**Files:**
- Create: `crates/fp-model/src/as_run.rs`
- Modify:
  - `config.rs` (`as_run_keep_days`)
  - `state.rs` (`as_run: AsRunLog`)
  - `reducer.rs` (`settle_all` calls `as_run::record`)
  - `lib.rs`
- Test: `crates/fp-model/tests/it/as_run.rs` (new)

**Produces:**
```rust
pub enum AsRunSource { Schedule, Filler }
pub enum AsRunOutcome { Aired, Missed }
pub struct AsRunEntry {
    pub seq: u64,
    pub date: NaiveDate,                  // the item's date, or the day it aired
    pub scheduled: Option<NaiveDateTime>, // the block's start; None for the filler
    pub started: Option<NaiveDateTime>,   // local time; None when missed
    pub track: Option<TrackId>,
    pub item: Option<ItemRef>,
    pub source: AsRunSource,
    pub outcome: AsRunOutcome,
}
pub struct AsRunLog { entries: VecDeque<AsRunEntry>, next_seq: u64, seen: … }
impl AsRunLog {
    pub fn entries(&self) -> impl Iterator<Item = &AsRunEntry>;
    pub fn since(&self, seq: u64) -> impl Iterator<Item = &AsRunEntry>;
    pub fn next_seq(&self) -> u64;
    pub fn resume_at(&mut self, next_seq: u64);
}
pub(crate) fn record(state: &mut AppState);
```

**Rules:**
- A new current row on the schedule's player that maps to an `ItemRef`
  records Aired with the block's start.
- A new current row on the filler's player records Aired from the Filler,
  with no scheduled time.
- Each `ItemRef` newly in `missed` records Missed.
- Entries dated before yesterday leave the window.

**Tests:**
- `a_block_item_that_starts_is_recorded_aired`
- `a_filler_song_that_starts_is_recorded`
- `a_missed_item_is_recorded_missed_once`
- `a_missed_item_played_later_has_both_entries`
- `old_entries_leave_the_window`
- `sequence_numbers_only_grow`
- `as_run_keep_days_has_its_default_and_range`

Commit: `feat(model): record what aired and what was missed (AU36)`.

### Task 2: Saving, loading and pruning

**Files:**
- `fp-store` `db/schema.rs`: migration 5,
  `CREATE TABLE as_run (seq INTEGER PRIMARY KEY, date INTEGER NOT NULL, doc TEXT NOT NULL); CREATE INDEX as_run_by_date ON as_run(date);`
- `db/snapshot.rs`: write the entries above the last saved sequence, and
  load `MAX(seq) + 1`.
- `store.rs` / `fp-app/src/session.rs`: prune at start-up, with a count
  logged.
- Test: `crates/fp-store/tests/it/as_run_store.rs`.

**Tests:**
- `entries_are_appended_once`
- `the_next_sequence_survives_a_restart`
- `rows_older_than_the_keep_days_are_pruned`
- `a_damaged_row_is_skipped_with_a_warning`
- The schema-version tests are updated to 5.

Commit: `feat(store): keep the as-run log (AU36, AU38)`.

### Task 3: The long-run test (AU39)

**Files:**
- Test: `crates/fp-model/tests/it/automation_long_run.rs`, using the
  `automation_rig` with a walker that ends songs at their length.
- Test: `crates/fp-store/tests/it/automation_long_run_store.rs`, saving
  the state every simulated hour.

**The run:**
- 30 simulated days.
- A week plan with two templates: weekdays with blocks every hour, and a
  weekend template.
- An override on day 10.
- `as_run_keep_days = 3`.
- Automation runs throughout.

**The checks:**
- The filler list stays at or under `filler_played_kept` played rows plus
  the look-ahead.
- The schedule player's list stays at or under one block.
- `missed` and `started` hold only yesterday to tomorrow.
- The in-memory as-run window stays at or under two days.
- Every block of every day starts within the late tolerance. Every block
  of the run is checked from the as-run log.
- In the store test, the `as_run` row count stays at or under four days'
  worth.

Ruling: this runs at model and store level with explicit time, not
through the engine. Engine timing is covered by plan 4's conductor test,
and the engine's real-time loop at a 5 ms tick over 30 days cannot run in
CI time.

Commit: `test: thirty days of automation (AU39)`.

### Task 4: Docs

- `docs/technical/automation.md`: the as-run log.
- `docs/technical/persistence.md`: the `as_run` table and the setting.
- The spec's §12 ruling on AU39.
- The README roadmap: automation is complete for its first release.

Then the final review, test-first fixes, and the PR.

## Rulings made while building

- **Pruning happens on save, not at start-up.** The store deletes the old
  rows in the save whose cutoff day changes (the first save of a session
  included), so a session that runs for weeks prunes every day. It logs
  no count. `fp-app` is not touched. After review, the cutoff moves no
  faster than real time since the session's first save, so a clock that
  jumps ahead does not delete the log.
- **One long-run test.** AU39 is one test at store level,
  `fp-store/tests/it/automation_long_run.rs`, with a small fake engine:
  the store half needs the model half's run anyway.
- **The README roadmap stays "in progress".** Calling automation complete
  is the maintainer's decision after real use; the roadmap line lists
  the as-run log as built.
