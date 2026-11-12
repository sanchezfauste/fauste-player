# Automation plan 3 — Schedule model Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The schedule's data and rules: blocks and items, day templates,
the week plan, per-date templates and date overrides, the computed
schedule of a date with local time and daylight saving, the pruning of old
overrides, and their persistence. No UI and no playout yet.

**Architecture:** Two new pure modules in `fp-model`:
- `schedule.rs` holds the types, the edit commands and the computation of a date.
- `local_time.rs` holds the time of day and the local-time lookup.

`AppState::schedule` holds the data, and the reducer applies the new
commands. `fp-store` gets schema migration 4 and the snapshot fields.
`fp-app` makes sure the template "Base" exists and prunes old overrides at
start-up. Dates are `chrono::NaiveDate`. The local zone is passed in through
a small trait, so the model never reads the clock or the zone itself.

**Tech Stack:**
- Rust 2024, serde.
- chrono, already a workspace dependency, is added to `fp-model` with its
  `serde` feature.
- rusqlite in fp-store.

**Spec:** `docs/superpowers/specs/2026-11-11-automation-design.md` (AU5–AU14,
the overrides half of AU38, §10, §11 `automation.overrides_keep_days`).

**Scope:**
- This plan implements AU5–AU7 and AU9–AU14. AU8 (the default rotation
  fills a short block) is a running rule, so it belongs to plan 4.
- **Plan 4** runs the schedule (AU15–AU26) on `Schedule::day` and
  `block_start`.
- **Plan 5** is the editor (§8.2, AU37). It drives the commands added here.

## Global Constraints

- CLAUDE.md rules:
  - English identifiers, docs and commits.
  - No `unwrap`/`expect`/`panic` outside tests; prefer `get`.
  - Rule 4 (config).
  - Rule 9: damaged rows degrade with a warning.
- Config: `automation.overrides_keep_days` goes in `AutomationConfig`:
  - default 30;
  - clamped to 1–3 650 in `Config::validate`;
  - `#[serde(default)]` through the struct's existing default.
- Every new `ModelError` variant is mapped in `fp-app/src/ui/app.rs`'s
  error-to-text match. A new message goes into every `locales/*/main.ftl`:
  en-US and es-ES written, the others translated faithfully.
- New tests go in `crates/<crate>/tests/it/<file>.rs` with a `mod` line in
  `tests/it/main.rs`.
- Every commit passes fmt, clippy `-D warnings` and `cargo test --workspace`.
  Its message ends with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Terms (spec §3), in code and docs:
  - Block, Item, Day template, Week plan, Date override, Time-critical.
  - es-ES: Bloque, Elemento, Plantilla, Semana tipo, Cambio del día, Puntual.
- Never name other products (rule 2).

## Review Focus

1. **Collision on a date:** after a template edit, a template block and a
   block changed or added on one date share a start time. The day shows the
   date's block once and drops the template one, with no duplicate and no
   panic (Task 4, `a_date_block_wins_a_time_collision`).
2. **A date that switches template after changes:** a block changed on a
   date whose template is then swapped (AU10). The changed block stays as a
   block of that date at its own time (the AU12 deleted-block rule). Nothing
   is lost silently (Task 4, `changes_survive_a_template_switch`).
3. **Daylight saving:** a block at 02:30 on the spring change has no start
   (missed). A block at 02:30 on the autumn change starts once, at the first
   occurrence (Task 4, `spring_gap_has_no_start` and
   `autumn_overlap_starts_at_the_first`).
4. **Deleting a template still in use:** deleting a template the week plan
   uses is refused. Dates that name it fall back to their weekday (Task 2,
   `a_template_in_the_week_plan_cannot_be_deleted` and
   `dates_naming_a_deleted_template_fall_back`).
5. **Damaged rows:** a template block at 25:00, two blocks at one time, a
   week plan naming a missing template, a date row whose key and date
   disagree. Each is repaired or dropped with a warning, and the rest loads
   (Task 5, `damaged_schedule_rows_are_repaired_or_dropped`).

---

### Task 1: Time of day, local zone, schedule types, time-critical items

**Files:**
- Modify: `Cargo.toml` (workspace chrono stays); `crates/fp-model/Cargo.toml` (add `chrono = { workspace = true, features = ["serde"] }`)
- Create: `crates/fp-model/src/local_time.rs`, `crates/fp-model/src/schedule.rs`
- Modify: `crates/fp-model/src/ids.rs` (`TemplateId`, `BlockId`, `IdGen::template`, `IdGen::block`), `crates/fp-model/src/lib.rs` (modules + re-exports)
- Test: `crates/fp-model/tests/it/schedule_types.rs`

**Interfaces:**
- Produces, in `local_time.rs`:
  - `pub struct TimeOfDay(u32)`: seconds after midnight, `0..86_400`. It
    serialises transparent.
  - `TimeOfDay::new(secs: u32) -> Option<TimeOfDay>`.
  - `TimeOfDay::hms(h: u32, m: u32, s: u32) -> Option<TimeOfDay>`.
  - `TimeOfDay::secs(self) -> u32`.
  - `impl Display` (`HH:MM:SS`) and `TimeOfDay::parse(&str) -> Option<TimeOfDay>`
    (`H:MM` or `HH:MM:SS`).
  - `pub const SECS_PER_DAY: u32 = 86_400`.
  - `pub trait LocalZone { fn earliest(&self, local: chrono::NaiveDateTime) -> Option<WallTime>; }`,
    implemented for `chrono::Local` and `chrono::FixedOffset` through
    `from_local_datetime(..).earliest()`.
- Produces, in `schedule.rs`:
  - `pub enum Item { Track { track: TrackId, #[serde(default)] time_critical: bool }, Category { category: CategoryId, #[serde(default)] time_critical: bool }, Rotation { rotation: RotationId } }`.
  - `Item::is_time_critical(&self, state: &AppState) -> bool` (AU7).
  - `pub struct Block { pub id: BlockId, pub start: TimeOfDay, pub items: Vec<Item> }`.
  - `pub fn check_items(items: &[Item]) -> Result<(), ModelError>`: a
    `Rotation` item must be the last item. It fails with
    `ModelError::RotationNotLast`, a new variant.

- [ ] **Step 1: Write the failing tests.** In `schedule_types.rs`:
  - `time_of_day_bounds`: `TimeOfDay::new(86_399)` is `Some` and `new(86_400)` is `None`; `hms(24,0,0)` and `hms(7,60,0)` are `None`.
  - `time_of_day_text`: `hms(7,30,0)` shows `"07:30:00"`; `parse("7:30")` and `parse("07:30:00")` both equal `hms(7,30,0)`; `parse("25:00")` and `parse("x")` are `None`.
  - `fixed_offset_zone`: `FixedOffset::east_opt(3600)` maps local 2026-03-01 10:00:00 to `WallTime` of `2026-03-01T09:00:00Z` in milliseconds.
  - `hourly_id_items_are_time_critical`:
    - a Track item of kind HourlyId is time-critical, and a Music one is not;
    - a Category item over an HourlyId-kind category is time-critical;
    - a Music item with `time_critical: true` is time-critical;
    - a Rotation item is never time-critical;
    - an item naming an unknown track is not time-critical.
  - `a_rotation_item_must_be_last`: `check_items([Rotation, Track])` is `Err(RotationNotLast)`; `[Track, Rotation]` and `[]` are `Ok`.
- [ ] **Step 2: Run** `cargo test -p fp-model --test it schedule_types::`. Expected: does not compile (missing types).
- [ ] **Step 3: Implement** the types above, the two ids and the error variant. Map `RotationNotLast` in `ui/app.rs` to a new message `error-schedule-rotation-last` ("A rotation must be the last item of a block." / "Una rotación debe ser el último elemento del bloque."), in every locale.
- [ ] **Step 4: Run** the same command. Expected: PASS.
- [ ] **Step 5: Commit** `feat(model): schedule blocks, items and the time of day`.

### Task 2: Day templates and the week plan (AU5, AU9)

**Files:**
- Modify: `crates/fp-model/src/schedule.rs`, `state.rs` (`pub schedule: crate::schedule::Schedule`), `command.rs`, `reducer.rs`, `error.rs`, `crates/fp-app/src/ui/app.rs`, locales
- Test: `crates/fp-model/tests/it/schedule_templates.rs`

**Interfaces:**
- Consumes: Task 1 types.
- Produces:
  - `pub struct DayTemplate { pub id: TemplateId, pub name: String, pub blocks: Vec<Block> }`.
    Its blocks are kept sorted by `start`, and no two share a start.
  - `pub struct WeekPlan(pub [Option<TemplateId>; 7])`, Monday first. It has
    `get(chrono::Weekday) -> Option<TemplateId>`.
  - `pub struct Schedule` (fields private) with:
    - `templates() -> &[DayTemplate]`, `template(id) -> Option<&DayTemplate>`;
    - `week() -> &WeekPlan`;
    - `from_parts(templates, week, dates: Vec<DatePlan>) -> (Schedule, Vec<String>)`
      (repairs and their warnings, used by Task 5);
    - `into_parts()`;
    - `max_raw_id() -> u64`, the largest template or block id.
  - `pub fn ensure_default(state: &mut AppState, name: &str)`: with no
    template, adds `name` (no blocks) and gives it to all seven days.
  - Commands:
    - `CreateTemplate { name: String }`;
    - `DuplicateTemplate { template: TemplateId, name: String }`: the copy's
      blocks get fresh ids;
    - `RenameTemplate { template: TemplateId, name: String }`;
    - `DeleteTemplate(TemplateId)`;
    - `AddBlock { template: TemplateId, start: TimeOfDay }`: no items;
    - `UpdateBlock { template: TemplateId, block: Block }`: start and items;
    - `RemoveBlock { template: TemplateId, block: BlockId }`;
    - `SetWeekTemplate { weekday: chrono::Weekday, template: TemplateId }`.
  - New `ModelError` variants:
    - `UnknownTemplate(TemplateId)` and `UnknownBlock(BlockId)`: mapped to
      `error-not-found`;
    - `EmptyTemplateName`, `LastTemplate`, `TemplateInUse`, `BlockTimeTaken`:
      each gets its own message.

- [ ] **Step 1: Write the failing tests:**
  - `a_fresh_install_has_base_on_every_day`: `ensure_default(&mut s, "Base")` twice gives one template, named "Base", and all 7 weekdays name it.
  - `blocks_stay_sorted_and_unique`: AddBlock at 08:00, then 07:30. The template's starts are `[07:30, 08:00]`. AddBlock at 08:00 again gives `Err(BlockTimeTaken)`. UpdateBlock moving 07:30 to 08:00 gives `Err(BlockTimeTaken)`.
  - `update_block_checks_items`: UpdateBlock with `[Rotation, Track]` gives `Err(RotationNotLast)`, and the block is unchanged.
  - `duplicate_gives_fresh_block_ids`: the copy has the same starts and items. None of its block ids appears in the original.
  - `a_template_in_the_week_plan_cannot_be_deleted`: `Err(TemplateInUse)`. Once no weekday names it, the delete succeeds. Deleting the only template gives `Err(LastTemplate)`.
  - `template_names_are_trimmed_and_required`: Create with `"  "` gives `Err(EmptyTemplateName)`, and Rename to `" Weekend "` stores `"Weekend"`.
- [ ] **Step 2: Run** `cargo test -p fp-model --test it schedule_templates::`. Expected: does not compile.
- [ ] **Step 3: Implement.** Add the field to `AppState` (`AppState::new` gives `Schedule::default()`; `restore` takes it from `RestoreParts::schedule`, a new field, and observes `max_raw_id`). Add the commands, the reducer arms and the errors. Write the messages:

  | Message | en-US | es-ES |
  |---|---|---|
  | `error-template-name-empty` | "A template needs a name." | "La plantilla necesita un nombre." |
  | `error-last-template` | "The last template cannot be deleted." | "No se puede eliminar la última plantilla." |
  | `error-template-in-use` | "The week plan uses this template." | "La semana tipo usa esta plantilla." |
  | `error-block-time-taken` | "Another block starts at that time." | "Otro bloque empieza a esa hora." |
- [ ] **Step 4: Run** the same command. Expected: PASS.
- [ ] **Step 5: Commit** `feat(model): day templates and the week plan`.

### Task 3: Per-date templates and date overrides (AU10–AU12)

**Files:**
- Modify: `crates/fp-model/src/schedule.rs`, `command.rs`, `reducer.rs`
- Test: `crates/fp-model/tests/it/schedule_dates.rs`

**Interfaces:**
- Consumes: Task 2.
- Produces:
  - `pub enum DateChange { Changed { block: Block, own_time: bool }, Removed(BlockId), Added(Block) }`.
    In `Changed`, `block.id` is a template block's id. `own_time` says
    whether its start was set on this date (the start is kept either way,
    for the AU12 deleted-block case).
  - `pub struct DatePlan { pub date: NaiveDate, pub template: Option<TemplateId>, pub changes: Vec<DateChange> }`.
    The schedule keeps them in a `BTreeMap<NaiveDate, DatePlan>`. A plan
    with no template and no change is removed.
  - `Schedule::date_plan(date) -> Option<&DatePlan>`.
  - `Schedule::template_for(date) -> Option<TemplateId>`: the date's own
    template if it still exists, else its weekday's.
  - Commands, where `block.id` is a block of the date's computed day:
    - `SetDateTemplate { date: NaiveDate, template: Option<TemplateId> }`;
    - `ChangeBlockOnDate { date: NaiveDate, block: Block }`;
    - `AddBlockOnDate { date: NaiveDate, start: TimeOfDay }`;
    - `RemoveBlockOnDate { date: NaiveDate, block: BlockId }`;
    - `RevertBlockOnDate { date: NaiveDate, block: BlockId }`.
- Rules:
  - **ChangeBlockOnDate on a template block:** stores
    `Changed { own_time: block.start != template block's start }` and
    replaces an earlier change of the same block.
  - **ChangeBlockOnDate on a date-only block:** updates its `Added` entry.
  - **RemoveBlockOnDate on a template block:** stores `Removed(id)`, which
    replaces a `Changed` of the same block.
  - **RemoveBlockOnDate on a date-only block:** deletes its `Added` entry.
  - **RevertBlockOnDate:** drops the `Changed` or `Removed` of that template
    block. On a block that has none it gives `Err(UnknownBlock)`.
  - **Time check:** a start already used by another block of the computed
    day gives `Err(BlockTimeTaken)`.
  - **Items:** items go through `check_items`.
  - **DeleteTemplate:** sets `template: None` on every date plan that named
    the deleted template, and removes plans left empty.

- [ ] **Step 1: Write the failing tests** (a helper builds a "Base" template with blocks at 07:00, 08:00 and 09:00, and a date on a Tuesday):
  - `a_date_can_use_another_template`: SetDateTemplate to "Weekend" makes `template_for(date)` "Weekend" while the next Tuesday stays "Base". Setting `None` removes the date plan.
  - `a_change_stores_only_that_block`: ChangeBlockOnDate on the 08:00 block with new items stores exactly one `Changed` (`own_time: false`).
  - `moving_a_block_on_a_date_sets_own_time`: ChangeBlockOnDate moving 08:00 to 08:30 stores `own_time: true`.
  - `removing_and_reverting_a_template_block`:
    - RemoveBlockOnDate on 09:00 stores `Removed`;
    - RevertBlockOnDate removes it and the plan;
    - a second Revert gives `Err(UnknownBlock)`.
  - `added_blocks_belong_to_the_date`:
    - AddBlockOnDate at 10:00 stores `Added`;
    - RemoveBlockOnDate on it removes the entry;
    - AddBlockOnDate at 08:00 gives `Err(BlockTimeTaken)`.
  - `dates_naming_a_deleted_template_fall_back`:
    - a date plan names "Weekend";
    - after the week plan stops using "Weekend" and it is deleted, the
      date's plan is gone;
    - `template_for` gives the weekday's template.
- [ ] **Step 2: Run** `cargo test -p fp-model --test it schedule_dates::`. Expected: does not compile.
- [ ] **Step 3: Implement** the types, the commands and the rules above.
- [ ] **Step 4: Run** the same command. Expected: PASS.
- [ ] **Step 5: Commit** `feat(model): per-date templates and date overrides`.

### Task 4: The computed schedule of a date, daylight saving, pruning (AU13, AU14, AU38)

**Files:**
- Modify: `crates/fp-model/src/schedule.rs`, `crates/fp-model/src/config.rs`
- Test: `crates/fp-model/tests/it/schedule_day.rs`

**Interfaces:**
- Consumes: Tasks 1–3.
- Produces:
  - `pub enum BlockOrigin { Template, Changed, DateOnly }`.
  - `pub struct DayBlock { pub id: BlockId, pub start: TimeOfDay, pub items: Vec<Item>, pub origin: BlockOrigin }`.
  - `Schedule::day(&self, date: NaiveDate) -> Vec<DayBlock>`, sorted by
    start with unique starts (AU13).
  - `pub fn block_start(zone: &impl LocalZone, date: NaiveDate, start: TimeOfDay) -> Option<WallTime>`
    (AU14: `None` when the local time does not exist; the earliest one when
    it occurs twice).
  - `pub fn prune_dates(state: &mut AppState, today: NaiveDate) -> usize`
    removes the date plans dated before
    `today - automation.overrides_keep_days` and returns how many it removed.
  - `AutomationConfig::overrides_keep_days: u32`: default 30, clamped to 1–3 650.
- How `day` builds a date:
  1. Start from the template's blocks (`template_for`).
  2. For each `Changed`: if the template has that block, take the change's
     items, and its start when `own_time` is set (else the template block's
     start), as `Changed`. If the template has no such block, use the
     change's own block as `DateOnly`.
  3. Drop the blocks a `Removed` names.
  4. Add each `Added` as `DateOnly`.
  5. Sort by `(start, id)`. When two blocks share a start, keep the
     `Changed` or `DateOnly` one and drop the `Template` one. Between two
     blocks of the same origin, keep the lower id.

- [ ] **Step 1: Write the failing tests:**
  - `a_plain_day_is_its_template`: the Tuesday with no plan equals Base's blocks, all `Template`.
  - `template_edits_reach_unchanged_blocks`: after UpdateBlock on Base's 07:00 items, the date shows the new items for 07:00 and keeps its own `Changed` 08:00.
  - `a_changed_block_follows_its_template_time`: with an 08:00 change (`own_time: false`), moving the template block to 08:15 shows the change at 08:15.
  - `a_changed_block_outlives_its_template_block`: with an 08:00 change, RemoveBlock on Base's 08:00 still shows the date's block at 08:00 as `DateOnly`.
  - `changes_survive_a_template_switch`: a date changes its 08:00 block, then SetDateTemplate to an empty "Weekend". The day has exactly that one block, `DateOnly`.
  - `a_date_block_wins_a_time_collision`: a block added on the date at 10:00, then AddBlock 10:00 on Base. The day has one 10:00 block, `DateOnly`.
  - `spring_gap_has_no_start` and `autumn_overlap_starts_at_the_first`:
    - a test zone in the test file implements `LocalZone`: UTC+1, with
      UTC+2 from local 2026-03-29 02:00 to 2026-10-25 03:00;
    - `block_start(zone, 2026-03-29, 02:30)` is `None`;
    - `block_start(zone, 2026-10-25, 02:30)` is `00:30Z` (UTC+2), not
      `01:30Z`.
  - `old_overrides_are_pruned`:
    - plans on 2026-01-01, 2026-02-10 and 2026-02-20; keep_days 30; today
      2026-03-12;
    - prune removes only 2026-01-01 and returns 1;
    - a second prune returns 0.
  - `overrides_keep_days_is_clamped`: 0 becomes 1 and 99 999 becomes 3 650 after `Config::validate`.
- [ ] **Step 2: Run** `cargo test -p fp-model --test it schedule_day::`. Expected: does not compile.
- [ ] **Step 3: Implement** `day`, `block_start`, `prune_dates` and the config field.
- [ ] **Step 4: Run** the same command. Expected: PASS.
- [ ] **Step 5: Commit** `feat(model): compute a date's schedule with its overrides`.

### Task 5: Persistence (§10, schema 4) and start-up

**Files:**
- Modify:
  - `crates/fp-store/src/db/schema.rs`: migration 4, `SCHEMA_VERSION = 4`;
  - `crates/fp-store/src/db/snapshot.rs`: fields, read, write, sync;
  - `crates/fp-store/src/store.rs`: `restore`;
  - `crates/fp-store/src/import.rs` and `legacy.rs`: empty values;
  - `crates/fp-app/src/session.rs`: `ensure_default` and `prune_dates`;
  - `crates/fp-app/locales/*/main.ftl`: `schedule-template-default-name`.
- Test: `crates/fp-store/tests/it/schedule_store.rs`, and
  `crates/fp-store/tests/it/db_schema.rs` (the version is now 4).

**Interfaces:**
- Consumes: Tasks 2–4 (`Schedule::from_parts`, `into_parts`).
- Produces:
  - Migration 4:
    ```sql
    CREATE TABLE day_templates (id INTEGER PRIMARY KEY, position INTEGER NOT NULL, doc TEXT NOT NULL);
    CREATE TABLE date_plans (id INTEGER PRIMARY KEY, position INTEGER NOT NULL, doc TEXT NOT NULL);
    ```
  - Keys and positions:
    - a date plan's `id` is `date.num_days_from_ce()`, and its `position`
      is its index in date order;
    - the week plan is the `kv` value `week_plan`.
  - New `Snapshot` fields: `templates: Vec<DayTemplate>`,
    `week_plan: WeekPlan` and `date_plans: Vec<DatePlan>`.
  - `Schedule::from_parts` repairs, with one warning each:
    - a block whose start is out of range is dropped;
    - of two blocks sharing a start, the later one is dropped;
    - a weekday naming a missing template, or none, gets the first template;
    - a date plan naming a missing template gets `template: None`;
    - a `Changed`/`Added` block with an invalid item order keeps its items
      up to and including the first rotation.
  - The reader drops a date row whose doc's date disagrees with its key,
    with a warning.
- At start-up, in `session.rs`, after `rotation::ensure_default`:
  - `schedule::ensure_default(&mut state, &tr("schedule-template-default-name"))`;
  - `schedule::prune_dates(&mut state, chrono::Local::now().date_naive())`.
  - en-US "Base", es-ES "Base"; the other locales translated.

- [ ] **Step 1: Write the failing tests** in `schedule_store.rs`:
  - `the_schedule_round_trips`: two templates with blocks of all three item kinds, a week plan using both, and a date plan with `Changed`, `Removed` and `Added`. After save and load, `state.schedule` is equal and there are no warnings.
  - `a_schema_3_database_migrates_to_4_with_no_templates`: drop the two tables and the kv value, set `user_version = 3`, then load. There are no warnings and no templates, and `PRAGMA user_version` is 4.
  - `pruned_dates_leave_the_table`: save with three date plans, prune one, save. `date_plans` has 2 rows.
  - `damaged_schedule_rows_are_repaired_or_dropped`, with hand-edited rows:
    - a block with `start` 90000 is dropped;
    - two blocks at 08:00: one remains;
    - a `week_plan` naming id 999 gets the first template;
    - a date row whose key disagrees with its doc's date is dropped;
    - a template row with invalid JSON is dropped;
    - each of the five gives a warning, and the rest loads.
- [ ] **Step 2: Run** `cargo test -p fp-store --test it schedule_store::`. Expected: does not compile.
- [ ] **Step 3: Implement** the migration, the snapshot fields (read, full write, `sync_docs` for both tables, `write_kv` when the week plan changed), `restore` and the start-up calls.
- [ ] **Step 4: Run** `cargo test -p fp-store --test it` and `cargo test -p fp-app --test it`. Expected: PASS.
- [ ] **Step 5: Commit** `feat(store): save the schedule (schema 4)`.

### Task 6: Documentation

**Files:**
- Modify:
  - `docs/technical/automation.md`: a "Schedule model" section with the
    types, how a day is computed, the collision rule, local time and
    pruning;
  - `docs/technical/persistence.md`: schema 4;
  - `docs/user/settings.md`, if it lists config-file-only fields: add
    `automation.overrides_keep_days`.
- No user-guide page yet: nothing is visible until plans 4 and 5.

- [ ] **Step 1:** Write the sections. Run `docs-sync-checker` on the branch diff.
- [ ] **Step 2: Commit** `docs: the schedule model and schema 4`.
