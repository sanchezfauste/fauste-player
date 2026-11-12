# Automation plan 1 — Rotations and selection Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the music-selection foundation of automation: the Hourly ID
track kind, rotations with their steps, last-aired times, the separation
rules and fair choice, and a rotation cursor that yields the next track.
These are pure model rules, saved in the database. No UI and no new player
yet.

**Architecture:** New pure modules in `fp-model`:
- `wall_time.rs`: wall-clock time and a seeded PRNG.
- `air_times.rs`: last aired time per track.
- `rotation.rs`: rotations and their commands.
- `selection.rs`: eligibility and the pick.
- `rotation_cursor.rs`: walks a rotation's steps.

Times are explicit inputs: `AppState::wall_now` is set by the conductor
every tick and by tests directly. `fp-store` gets schema migration 3
(rotations, air times) and the snapshot fields.

**Tech Stack:** Rust 2024, serde, rusqlite (fp-store), chrono is not needed
(times are UTC Unix milliseconds).

**Spec:** `docs/superpowers/specs/2026-11-11-automation-design.md`.

**Scope:**
- This plan implements AU1–AU4, AU30–AU35 and the selection side of AU25.
  AU25 is only the `planned` overlay and the `min/max` length filter hook
  below; the fitting itself comes in plan 2.
- **Plan 2** (the filler player, its list rules AU27–AU29, the Music
  column, the rotation editor) builds on these interfaces.

## Global Constraints

- **CLAUDE.md rules apply to every task:**
  - English identifiers, docs and commits.
  - No `unwrap`/`expect`/`panic` outside tests.
  - Use `get` rather than indexing.
  - No hard-coded operator-facing values (rule 4).
  - Untrusted rows degrade with a warning (rule 9).
- **Config:** every new field lives in `AutomationConfig`
  (`crates/fp-model/src/config.rs`), with a default, a clamp in
  `Config::validate`, and `#[serde(default)]`:
  - `automation.track_separation_mins`: default 180, range 0–10 080.
  - `automation.track_separation_percent`: default 50, range 0–95.
  - `automation.artist_separation_mins`: default 30, range 0–1 440.
  - The value 0 turns its criterion off.
- **New integration tests** go in `crates/<crate>/tests/it/<file>.rs`, with
  a `mod <file>;` line in `tests/it/main.rs`. Run one with
  `cargo test -p <crate> --test it <file>::`.
- **Every commit** passes `cargo fmt --all --check`,
  `cargo clippy --workspace --all-targets -- -D warnings` and
  `cargo test --workspace`, and ends with the trailer
  `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- **UI strings:** new strings go in every
  `crates/fp-app/locales/*/main.ftl`. Write en-US and es-ES by hand; the
  other locales get a faithful translation.
- **Patents:** use only the classic methods named in the spec §6.2. Do not
  copy any other product's behaviour, and do not name one anywhere (rule 2).

## Review Focus

1. **Deleted category:** a rotation step names a category that was
   deleted. The step is skipped and reported, with no panic and no endless
   loop (Task 6, `a_step_on_a_deleted_category_is_skipped`).
2. **Nothing playable:** the library has no playable track at all (empty,
   or every file missing). `pick` returns `None` and the cursor returns
   `None` after one pass over the steps (Task 5
   `nothing_playable_gives_none`, Task 6 `an_empty_library_ends_the_pass`).
3. **Clock jumped back:** a last-aired time lies in the future. It counts
   as "just aired", with no overflow and no panic (Task 5,
   `an_air_time_in_the_future_counts_as_just_aired`).
4. **Tiny or one-artist category:** a category of one track, or one where
   every track has the same artist, still yields a track through relaxing
   (Task 5, `relaxing_order_artist_then_track`).
5. **Damaged rows:** a rotation row with `min > max`, an unknown step
   source, or an air time for a track that no longer exists. Each is
   repaired or dropped with a warning, and the rest loads (Task 7,
   `damaged_rotation_and_air_time_rows_are_repaired_or_dropped`).

---

### Task 1: The Hourly ID track kind (AU30)

**Files:**
- Modify: `crates/fp-model/src/track.rs:15-22` (enum `TrackKind`).
- Modify: every exhaustive list of kinds. Find them with
  `grep -rn "TrackKind::Voice" crates --include=*.rs`. Today that gives
  `fp-model/src/library_search.rs` (`kind_rank`),
  `fp-remote/src/dto.rs` (kind names),
  `fp-app/src/ui/library/category_dialog.rs` (`KINDS`, `kind_key`) and the
  library menu that uses them.
- Modify: `crates/fp-app/locales/*/main.ftl` (`kind-hourly-id`).
- Modify: `docs/user/library.md:185` and `docs/technical/remote-api.md`
  (if it lists the kinds).
- Test: `crates/fp-model/tests/it/categories.rs`.
- Test: `crates/fp-remote/tests/it` (the dto kind test, if one exists).

**Interfaces:**
- Produces: `TrackKind::HourlyId`. It is placed last, after `Voice`, so
  stored values keep their meaning. It serializes as `"HourlyId"`. The
  remote API name is `"hourly_id"`. Its sort rank is 5.

- [ ] **Step 1: Write the failing test** `an_hourly_id_category_files_its_tracks_as_hourly_ids`
  in `categories.rs`:
  - Create a category with `kind: TrackKind::HourlyId`.
  - Add a track to it with `Command::AddToLibrary`.
  - `assert_eq!(track.kind, TrackKind::HourlyId)`.
  - Also assert the round trip
    `serde_json::to_string(&TrackKind::HourlyId) == "\"HourlyId\""`.
- [ ] **Step 2: Run it and watch it fail.**
  Run `cargo test -p fp-model --test it categories::an_hourly_id`.
  Expected: it does not compile, because there is no variant `HourlyId`.
- [ ] **Step 3: Add the variant** and fix every exhaustive match the
  compiler reports.
  - Add `kind-hourly-id` to every locale: en-US `Hourly ID`, es-ES
    `Indicativo horario`.
  - Add it to the kind lists of the category dialog and the library menu.
  - Update the user guide's kind list to "Music, Jingle, Effect, Ad, Voice
    or Hourly ID", with one sentence saying automation never plays an
    Hourly ID late.
- [ ] **Step 4: Run the test,** then the workspace suite. Expected: PASS.
- [ ] **Step 5: Commit** with the message
  `feat(model): add the Hourly ID track kind`.

---

### Task 2: Automation settings

**Files:**
- Modify: `crates/fp-model/src/config.rs` (new `AutomationConfig`, the
  `Config.automation` field, the clamps in `Config::validate` beside
  `library.*`).
- Modify: `crates/fp-model/src/lib.rs` (re-export `AutomationConfig`).
- Test: `crates/fp-model/tests/it/automation_config.rs` (new; add the
  `mod` line).

**Interfaces:**
- Produces:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AutomationConfig {
    pub track_separation_mins: u32,    // 180, 0..=10_080, 0 = off
    pub track_separation_percent: u8,  // 50, 0..=95, 0 = off
    pub artist_separation_mins: u32,   // 30, 0..=1_440, 0 = off
}
```

- [ ] **Step 1: Write the failing tests:**
  - `automation_defaults_match_the_spec`: asserts 180, 50 and 30.
  - `out_of_range_automation_values_are_clamped_with_a_warning`: sets
    `track_separation_mins = 20_000`, `track_separation_percent = 99` and
    `artist_separation_mins = 5_000`, calls `validate()`, and expects
    10 080, 95 and 1 440 plus three warnings naming the fields.
  - `a_config_without_automation_loads_the_defaults`: parses `{}` through
    the store's lenient config parse, or `serde_json::from_str::<Config>`,
    and gets the defaults.
- [ ] **Step 2: Run and watch them fail.**
  Run `cargo test -p fp-model --test it automation_config::`.
- [ ] **Step 3: Implement** the struct, its `Default`, the field on
  `Config` and in `Config::default`, and three `clamp_to` calls named
  `"automation.track_separation_mins"` and so on.
- [ ] **Step 4: Run the tests.** Expected: PASS.
- [ ] **Step 5: Commit** with the message
  `feat(model): add the automation separation settings`.

---

### Task 3: Wall time, the PRNG and last-aired times (AU35)

**Files:**
- Create: `crates/fp-model/src/wall_time.rs`.
- Create: `crates/fp-model/src/air_times.rs`.
- Modify: `crates/fp-model/src/state.rs` (new fields).
- Modify: `crates/fp-model/src/reducer.rs:1575` (`advance_to`).
- Modify: `crates/fp-model/src/cart_rules.rs:46` (`fire`).
- Modify: `crates/fp-model/src/library_rules.rs:288` (`remove_tracks`)
  and `forget_unreferenced`, so the air times of removed tracks go.
- Modify: `crates/fp-engine/src/conductor.rs`:
  - `tick`: set `state.wall_now` first.
  - `Conductor::new`: seed `state.rng`.
- Test: `crates/fp-model/tests/it/air_times.rs` (new).

**Interfaces:**
- Produces:

```rust
// wall_time.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WallTime(pub i64); // UTC Unix milliseconds
impl WallTime {
    pub fn now() -> Self;                         // SystemTime; before 1970 → WallTime(0)
    pub fn plus_secs(self, secs: f64) -> Self;    // saturating, non-finite → self
    pub fn millis_since(self, earlier: Self) -> i64; // saturating; negative when `earlier` is later
}
/// SplitMix64: small, seedable, deterministic in tests.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SplitMix64(pub u64);
impl SplitMix64 {
    pub fn next_u64(&mut self) -> u64;
    pub fn below(&mut self, n: u64) -> u64;       // 0 when n == 0
    pub fn in_range(&mut self, min: u8, max: u8) -> u8; // inclusive; swaps if min > max
}

// air_times.rs
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AirTimes { /* HashMap<TrackId, WallTime> */ }
impl AirTimes {
    pub fn get(&self, track: TrackId) -> Option<WallTime>;
    pub fn record(&mut self, track: TrackId, at: WallTime); // keeps the later time
    pub fn retain_tracks(&mut self, keep: impl Fn(TrackId) -> bool);
    pub fn iter(&self) -> impl Iterator<Item = (TrackId, WallTime)> + '_;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
impl FromIterator<(TrackId, WallTime)> for AirTimes;

// state.rs, new AppState fields
pub air_times: AirTimes,      // saved
pub wall_now: WallTime,       // runtime: the conductor sets it every tick
pub rng: SplitMix64,          // runtime: seeded by the conductor
```

- `AirTimes::record` keeps the later of the two times. Without that, an
  airing logged after a backward clock jump would make the track look
  older than it is.

- [ ] **Step 1: Write the failing tests** in `air_times.rs`:
  - `a_player_start_records_the_track_at_wall_now`: set
    `s.wall_now = WallTime(1_000)`, play an entry, and expect
    `s.air_times.get(track) == Some(WallTime(1_000))`.
  - `a_cart_fire_records_its_track`.
  - `a_cue_does_not_record`: a `CueEntry` and the library pre-listen leave
    `air_times` empty.
  - `removing_a_track_drops_its_air_time`.
  - `record_keeps_the_later_time`.
  - `splitmix_is_deterministic_and_in_range`: the same seed gives the same
    sequence; `in_range(2, 3)` gives only 2 or 3 over 1 000 draws, and
    both values appear.
- [ ] **Step 2: Run and watch them fail.**
  Run `cargo test -p fp-model --test it air_times::`.
- [ ] **Step 3: Implement.**
  - Call `crate::air_times::record_track(state, track)` in `advance_to`
    after the request is built, and in `cart_rules::fire` after the
    request.
  - `record_track` is `pub(crate)`. It reads `state.wall_now`.
  - Re-export `WallTime`, `SplitMix64` and `AirTimes` from `lib.rs`.
  - In the conductor:
    - `tick` starts with `self.state.wall_now = WallTime::now();`.
    - `new` seeds `state.rng = SplitMix64(seed)`, where `seed` comes from
      `SystemTime` nanoseconds.
- [ ] **Step 4: Run the tests and the workspace suite.** Expected: PASS.
- [ ] **Step 5: Commit** with the message
  `feat(model): record when each track last aired`.

---

### Task 4: Rotations and their commands (AU1, AU3, AU4)

**Files:**
- Create: `crates/fp-model/src/rotation.rs`.
- Modify: `crates/fp-model/src/ids.rs` (`RotationId` and
  `IdGen::rotation()`).
- Modify: `crates/fp-model/src/command.rs`, `reducer.rs` (dispatch),
  `error.rs`, `state.rs` (`pub rotations: Rotations`) and `lib.rs`.
- Modify: `crates/fp-app/src/session.rs:56` (ensure the default rotation
  after `store.load`).
- Modify: `crates/fp-app/locales/*/main.ftl` (`rotation-default-name`).
- Create: `docs/technical/automation.md`, and link it from
  `docs/technical/README.md`.
- Test: `crates/fp-model/tests/it/rotations.rs` (new).

**Interfaces:**
- Produces:

```rust
pub enum StepSource { Category(CategoryId), Kind(TrackKind) } // serde: externally tagged
pub struct RotationStep { pub source: StepSource, pub min: u8, pub max: u8 }
pub struct Rotation { pub id: RotationId, pub name: String, pub steps: Vec<RotationStep> }
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Rotations { list: Vec<Rotation>, default: Option<RotationId> }
impl Rotations {
    pub fn get(&self, id: RotationId) -> Option<&Rotation>;
    pub fn iter(&self) -> impl Iterator<Item = &Rotation>;
    pub fn default_id(&self) -> Option<RotationId>;
    pub fn default_rotation(&self) -> Option<&Rotation>;
    pub fn from_parts(list: Vec<Rotation>, default: Option<RotationId>) -> Self; // repairs: default must exist, else first
    pub fn into_parts(self) -> (Vec<Rotation>, Option<RotationId>);
}
pub const STEP_COUNT_RANGE: RangeInclusive<u8> = 1..=99;
pub fn check_step(step: &RotationStep) -> Result<(), ModelError>;
/// AU4: with no rotation at all, adds `name` with one step `Kind(Music) 1–1` as the default.
pub fn ensure_default(state: &mut AppState, name: &str);

// Command variants
CreateRotation { name: String, steps: Vec<RotationStep> },  // the new id is the largest
UpdateRotation(Rotation),                                   // replaces name and steps by id
DeleteRotation(RotationId),
SetDefaultRotation(RotationId),

// ModelError variants
UnknownRotation(RotationId),
InvalidRotationStep,   // min or max outside STEP_COUNT_RANGE, or min > max
EmptyRotationName,     // the name is empty after trimming
LastRotation,          // DeleteRotation of the only rotation
```

- `DeleteRotation` of the default rotation makes the first remaining
  rotation the default.
- `STEP_COUNT_RANGE` validates data; it is not an operator setting, so it
  is no `Config` field.
- A step on a deleted category stays stored. Selection skips it (Task 6),
  and the plan 2 editor shows it as broken.

- [ ] **Step 1: Write the failing tests** in `rotations.rs`, one per rule:
  - `create_validates_the_name_and_steps`: an empty name gives
    `EmptyRotationName`; `min 3 max 2` and `min 0` give
    `InvalidRotationStep`.
  - `create_update_delete_round_trip`.
  - `the_last_rotation_cannot_be_deleted`.
  - `deleting_the_default_promotes_the_first`.
  - `ensure_default_creates_music_once`: called twice, it leaves one
    rotation, named as given, with step `Kind(TrackKind::Music)` 1–1, and
    that rotation is the default.
  - `from_parts_repairs_a_dangling_default`.
- [ ] **Step 2: Run and watch them fail.**
  Run `cargo test -p fp-model --test it rotations::`.
- [ ] **Step 3: Implement.**
  - In `session.rs`, after the load:
    `fp_model::rotation::ensure_default(&mut loaded.state, &I18n::new(None).tr("rotation-default-name"))`.
    This uses the same English-at-load pattern as the default playlist.
  - Locale strings: en-US `Music`, es-ES `Música`.
  - Write `docs/technical/automation.md`, sections "Rotations" and "Air
    times": what is stored, where it is saved, and who sets `wall_now`.
- [ ] **Step 4: Run the tests and the workspace suite.** Expected: PASS.
- [ ] **Step 5: Commit** with the message `feat(model): add rotations`.

---

### Task 5: Selection: separation, fair choice, relaxing (AU31–AU34)

**Files:**
- Create: `crates/fp-model/src/selection.rs`.
- Modify: `docs/technical/automation.md` (section "Selection").
- Test: `crates/fp-model/tests/it/selection.rs` (new).

**Interfaces:**
- Consumes: `AirTimes`, `WallTime`, `SplitMix64` (Task 3); `StepSource`
  (Task 4); `AutomationConfig` (Task 2).
- Produces:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relaxed { None, Artist, Track }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Picked { pub track: TrackId, pub relaxed: Relaxed }
/// Tracks queued but not yet aired, with their expected start: they count as aired then (plan 2's look-ahead).
pub type Planned<'a> = &'a [(TrackId, WallTime)];
/// The playable tracks of `source`: a category with all its descendants, or every track of a kind.
pub fn pool(state: &AppState, source: &StepSource) -> Vec<TrackId>;
/// The spread of the fair choice: picks uniformly among this many least recently aired eligible tracks.
pub const PICK_SPREAD: usize = 5;
pub fn pick(state: &AppState, source: &StepSource, at: WallTime, planned: Planned<'_>, rng: &mut SplitMix64) -> Option<Picked>;
```

`pick` is not fixed by its signature. Follow this algorithm exactly:

```text
last(t)  = latest of state.air_times.get(t) and any planned time of t (None = never aired)
pool     = pool(state, source); empty → None
norm(a)  = a.trim().to_lowercase(); empty artist → exempt from the artist rule
track_ok(t):
    last(t) is None → true
    let mins = cfg.track_separation_mins, pct = cfg.track_separation_percent
    if mins == 0 && pct == 0 → true
    by_time = mins > 0 && at.millis_since(last(t)) >= mins * 60_000   (negative → not elapsed)
    others  = pool.len() - 1
    needed  = ceil(pct * others / 100)
    since   = count of u in pool, u != t, last(u) is Some and last(u) > last(t)
    by_pct  = pct > 0 && since >= needed
    by_time || by_pct
artist_ok(t): cfg.artist_separation_mins == 0, or an empty artist, or no track in the
    whole library with the same norm(artist) has last() within the last artist_separation_mins before `at`
tiers: [ok both] → Relaxed::None; [track_ok] → Artist; [all of pool] → Track
first non-empty tier; sort by (last asc, None first; then TrackId asc);
choose uniformly among the first min(len, PICK_SPREAD) with rng.below
```

- `PICK_SPREAD` tunes the algorithm; it is not an operator limit
  (Ruling: a constant). Ledger it.

- [ ] **Step 1: Write the failing tests,** each with a library built in the
  test (helpers: `track(id, artist, category)` and
  `aired(state, id, minutes_ago)`). Seed `rng` at
  `SplitMix64(7)`.
  - `a_never_aired_track_is_preferred`.
  - `track_separation_by_time`: with 10 tracks and `pct = 0`, a track aired
    179 min ago is not eligible; at 180 min it is.
  - `track_separation_by_percent_of_the_category`: with 35 tracks and
    `mins = 0`, `pct = 50`, a track becomes eligible when 17 others have
    aired since it, and not at 16.
  - `either_criterion_is_enough`.
  - `artist_separation_across_the_library`: an artist aired 29 min ago from
    another category blocks the candidate; at 30 min it does not; case and
    spaces are ignored.
  - `relaxing_order_artist_then_track`:
    - a one-track category aired 1 min ago is still picked, with
      `Relaxed::Track`;
    - a category where every track has an artist aired 1 min ago is picked
      with `Relaxed::Artist`.
  - `nothing_playable_gives_none`: an empty pool, and a pool whose tracks
    are all `FileState::Missing`.
  - `fair_choice_stays_among_the_oldest`: over 200 picks with no air times
    recorded between picks, every result lies in the 5 oldest.
  - `planned_tracks_count_as_aired`.
  - `an_air_time_in_the_future_counts_as_just_aired`: a time 1 h after
    `at` → not eligible by time, no panic.
  - `a_category_includes_its_subcategories`.
- [ ] **Step 2: Run and watch them fail.**
  Run `cargo test -p fp-model --test it selection::`.
- [ ] **Step 3: Implement** `pool` and `pick` in `selection.rs`, and
  re-export them.
  - `pool` keeps only tracks with `file_state.is_playable()`.
  - For a category source it walks the `Categories` tree (reuse its
    descendant helper if there is one; otherwise add
    `Categories::descendants_of(id) -> Vec<CategoryId>` with its own unit
    test).
  - Document the algorithm in `docs/technical/automation.md`.
- [ ] **Step 4: Run the tests.** Expected: PASS.
- [ ] **Step 5: Commit** with the message
  `feat(model): choose tracks with separation rules and fair choice`.

---

### Task 6: The rotation cursor (AU2, AU34)

**Files:**
- Create: `crates/fp-model/src/rotation_cursor.rs`.
- Modify: `docs/technical/automation.md` (section "Walking a rotation").
- Test: `crates/fp-model/tests/it/rotation_cursor.rs` (new).

**Interfaces:**
- Consumes: `pick`, `Picked` and `Planned` (Task 5); `Rotations` and
  `RotationStep` (Task 4).
- Produces:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RotationCursor { pub rotation: RotationId, pub step: usize, pub left: u8, pub fresh: bool }
impl RotationCursor { pub fn new(rotation: RotationId) -> Self; } // step 0, left 0, fresh
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorPick {
    pub picked: Option<Picked>,
    pub skipped_steps: Vec<usize>,   // steps that yielded nothing on this call (for the caller's log)
    pub used_default: bool,          // AU34 fallback to the default rotation
}
pub fn next_track(state: &AppState, cursor: &mut RotationCursor, at: WallTime, planned: Planned<'_>, rng: &mut SplitMix64) -> CursorPick;
```

`next_track` is not fixed by its signature. Follow this algorithm:

```text
rot = rotations.get(cursor.rotation) or else the default rotation (used_default = true); none → picked None
repeat at most rot.steps.len() times:
    if cursor.fresh { cursor.fresh = false; cursor.step = 0; cursor.left = draw(step 0) }
    else if cursor.left == 0 { cursor.step = (cursor.step + 1) % len; cursor.left = draw(step) }
    match pick(step.source) { Some(p) → cursor.left -= 1; return p,
                              None → skipped_steps.push(step); cursor.left = 0 }
every step failed → if rot is not the default: try the default with a temporary
    RotationCursor::new(default) (the caller's cursor is unchanged), used_default = true
draw(step) = rng.in_range(step.min, step.max)
```

- A rotation with no steps behaves like one whose steps all failed.
- The cursor is plain data: plan 2 saves it with the filler.

- [ ] **Step 1: Write the failing tests:**
  - `steps_play_in_order_and_wrap`: steps A 1–1, B 1–1 → A B A B.
  - `counts_are_drawn_in_their_range_each_time`: step A 2–3, B 1–1 → over
    50 cycles every run of A has length 2 or 3, and both lengths occur.
  - `a_step_on_a_deleted_category_is_skipped`: the result comes from the
    next step, and `skipped_steps == [0]`.
  - `an_unknown_rotation_uses_the_default`.
  - `a_fruitless_rotation_falls_back_to_the_default_once`.
  - `an_empty_library_ends_the_pass`: `picked == None`, the call returns,
    and `skipped_steps` lists every step.
- [ ] **Step 2: Run and watch them fail.**
  Run `cargo test -p fp-model --test it rotation_cursor::`.
- [ ] **Step 3: Implement** `next_track`, and re-export it.
- [ ] **Step 4: Run the tests.** Expected: PASS.
- [ ] **Step 5: Commit** with the message
  `feat(model): walk a rotation's steps`.

---

### Task 7: Saving rotations and air times (schema 3)

**Files:**
- Modify: `crates/fp-store/src/db/schema.rs` (migration 3,
  `SCHEMA_VERSION = 3`).
- Modify: `crates/fp-store/src/db/snapshot.rs`:
  - `Snapshot`: `rotations: Vec<Rotation>`,
    `default_rotation: Option<RotationId>`, `air_times: AirTimes`.
  - `Snapshot::of`, `read_checked` (new `read_rotations`,
    `read_air_times`), `write_full`, `write_changes`.
- Modify: `crates/fp-model/src/session.rs` (`RestoreParts` gains
  `rotations: Rotations` and `air_times: AirTimes`; `restore` sets them,
  and drops air times of tracks not in the library).
- Modify: `crates/fp-store/src/store.rs` (wherever `RestoreParts` is
  built).
- Modify: `docs/technical/persistence.md` (the new tables).
- Test: `crates/fp-store/tests/it/automation_store.rs` (new).
- Test: extend `crates/fp-store/tests/it/db_schema.rs` if it pins the
  version.

**Interfaces:**
- Migration 3:

```sql
CREATE TABLE rotations (id INTEGER PRIMARY KEY, position INTEGER NOT NULL, doc TEXT NOT NULL);
CREATE TABLE air_times (track INTEGER PRIMARY KEY, at INTEGER NOT NULL) WITHOUT ROWID;
```

- `default_rotation` lives in `kv` under the key `"default_rotation"`.
- `rotations` is written with `sync_docs`.
- `air_times` is written as a diff: upsert the changed pairs, delete the
  missing tracks.
- Reading is lenient:
  - an unparsable rotation doc is dropped with a warning;
  - a step with `min > max`, or out of `STEP_COUNT_RANGE`, is clamped into
    range and swapped, with a warning;
  - an air time whose track is not in the library is dropped silently, as
    a removed track's would be.
- `IdGen::observe` sees rotation ids.

- [ ] **Step 1: Write the failing tests:**
  - `rotations_and_air_times_round_trip`: write a full snapshot, read it,
    and compare it equal.
  - `a_changed_air_time_writes_one_row`: write the changes, then check the
    row count and the value.
  - `damaged_rotation_and_air_time_rows_are_repaired_or_dropped`: insert
    bad rows with raw SQL, read, and expect the repair and the warnings.
  - `a_schema_2_database_migrates_to_3_with_no_rotations`: then
    `ensure_default` adds "Music".
- [ ] **Step 2: Run and watch them fail.**
  Run `cargo test -p fp-store --test it automation_store::`.
- [ ] **Step 3: Implement.**
  - Follow the `categories` pattern (migration 2, `read_categories`,
    `sync_docs`) for rotations.
  - Follow the `tracks` diff pattern for `air_times`.
- [ ] **Step 4: Run the whole suite,** with fmt and clippy. Expected:
  all green.
- [ ] **Step 5: Commit** with the message
  `feat(store): save rotations and air times`.

---

### End of plan

- [ ] Run the `docs-sync-checker` agent on the branch and fix what it
  reports. No user-guide page describes rotations yet: the Music column and
  the editor arrive in plan 2.
- [ ] Request the code review (CLAUDE.md "End of plan"). Fix Critical and
  Important findings test-first, and ledger the Minor ones in
  `.superpowers/sdd/automation-plan1/progress.md`.
- [ ] Do not open the PR yet. Plans 1 and 2 merge together, so the first
  merged automation work already has a visible feature. The branch stays
  `feat/automation`.
