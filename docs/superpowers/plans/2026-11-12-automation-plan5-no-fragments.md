# Automation plan 5: no fragments before a block (AU25)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** the filler chooses its last songs before a block so that the
song faded at the hand-over loses at most `max_fade_cut_secs` and no song
starts that would play less than `min_song_play_secs` before its fade.

**Architecture:** the filler knows when each queued row will start (the
current row's start, seen by the model, plus each row's lead: to the segue
point when `auto_segue` is on, else to the cue-out). When a block is
coming, each row the filler adds is chosen with a `Fit` (the gap from the
row's start to the fade's start); the selection prefers tracks that end
the gap within the cut or leave room for a later song. When a song starts,
the generated rows after the loaded next are checked against the exact
timeline and the first one that no longer fits is chosen again (once per
song, so nothing churns). When nothing fits, the track that cuts least
wins and the conductor logs a line.

**Tech Stack:** Rust, `fp-model` (pure rules), `fp-engine` (conductor log line).

**Spec:** `docs/superpowers/specs/2026-11-11-automation-design.md` (AU25, §11, §13).

## Global Constraints

- `automation.max_fade_cut_secs`: default 30, range 0–300 (§11).
- `automation.min_song_play_secs`: default 45, range 0–600 (§11).
- Config fields: default, clamp in `Config::validate`, lenient loading (rule 4).
- Only classic, publicly documented methods (§13 Patents): least recently
  played order, separation rules, and a greedy fit of song lengths.
- No `unwrap`/`expect`/`panic` outside tests; tests in `crates/<crate>/tests/it/`.
- AU26: schedule items are never chosen or cut by this; only the filler's
  generated rows (AU28: the operator's rows stay).

## Review Focus

- Tracks of unknown length (not analysed yet): never counted as fitting;
  the fit must not prefer them nor divide by them.
- A block within less than `min_song_play_secs`: no song can satisfy both
  rules; the fallback must still fill (no dead air) and log once per pick,
  not once per tick.
- Segue drift: with `auto_segue` the next row starts at the segue point;
  the timeline must use it, or the fit is off by seconds per song.
- Re-fitting must never touch the current row, the loaded next of a
  playing filler, or an operator row.
- Rotation steps with short tracks (station IDs): the fit applies to
  whatever the step's pool holds.

---

### Task 1: The two config fields

**Files:**
- Modify: `crates/fp-model/src/config.rs` (the `automation` section, its
  `Default`, `Config::validate`)
- Test: `crates/fp-model/src/config.rs` unit tests (as the other automation fields)

- [ ] Write `automation_fit_settings_have_their_defaults_and_ranges`:
  defaults 30 and 45; 999 clamps to 300 and 600.
- [ ] Run, watch it fail (no field), add `max_fade_cut_secs: u32` and
  `min_song_play_secs: u32` with doc comments naming AU25, run green.
- [ ] Commit `feat(model): the fit settings of the filler (AU25)`.

### Task 2: The filler's timeline

**Files:**
- Modify: `crates/fp-model/src/filler.rs`, `crates/fp-model/src/session.rs`
  (the runtime field starts empty on restore)
- Test: `crates/fp-model/tests/it/automation_fit.rs` (new, `mod` line in `main.rs`)

**Produces:**
- `FillerState::current_since: Option<(EntryId, NaiveDateTime)>` (runtime,
  not saved): set in `settle` when the filler's current row changes, to
  `state.local_now`; cleared when it has no current row.
- `fn lead_secs(state: &AppState, entry: EntryId) -> f64`: from the play
  range's start to where the next row starts: the segue point when
  `players.auto_segue` is on and the track has one strictly inside its
  range (as `reducer::ordinary_plan`), else the length; unknown length
  gives `UNKNOWN_LENGTH_SECS`.
- `fn starts_ahead(state: &AppState, f: &FillerState) -> Vec<(EntryId, NaiveDateTime)>`:
  the expected start of each row from the anchor on: the first at
  `since + lead(current)` (not before now), or now without a current row.

- [ ] Test `the_timeline_uses_the_segue_point`: a filler song with a segue
  marker at 170 s of 180 s, `auto_segue` on; the second ahead row starts
  170 s after the current row started. With `auto_segue` off, 180 s.
- [ ] Run, watch it fail, implement, run green. Commit
  `feat(model): when each filler row will start`.

### Task 3: Choosing a track that fits

**Files:**
- Modify: `crates/fp-model/src/selection.rs`, `crates/fp-model/src/rotation_cursor.rs`
- Test: `crates/fp-model/tests/it/automation_fit.rs`

**Produces:**
- `pub struct Fit { pub gap_secs: f64, pub max_cut_secs: f64, pub min_play_secs: f64 }`
  in `selection.rs`.
- `pub fn pick_fitted(state, source, at, planned, rng, fit: Option<&Fit>) -> Option<Picked>`;
  `pick` calls it with `None` (unchanged behaviour).
- `Picked::unfitted: bool` (true when the fallback chose it).
- `pub fn next_track_fitted(…, fit: Option<&Fit>) -> CursorPick`;
  `next_track` calls it with `None`.

Classes for a track of known length `L` (cue range) and lead `D` with gap `G`:
- **Ends:** `L >= G`, `L - G <= max_cut`, `G >= min_play`.
- **Room:** `D < G` and `G - D >= min_play`, and the rest `R = G - D` can be
  closed: some pool track has length in `[R, R + max_cut]`, or `R` is at
  least the pool's longest length plus `min_play`.
- **Loose room:** `D < G` and `G - D >= min_play`.
- Unknown length: no class.

Choice: for each class in that order, the separation tiers as today
(both rules, then track only), each the fair choice among the first
`PICK_SPREAD` in least-recently-aired order; then the class with
separation relaxed. Nothing in a class: the least cut `max(L - G, 0)`,
ties to the smaller leftover `max(G - D, 0)`, then the order; `unfitted`.

- [ ] Tests, each with a one-step rotation over tracks 901–9xx of chosen
  lengths and a gap:
  - `a_song_that_ends_the_gap_is_preferred` (G 200, lengths 120, 210, 300:
    picks 210).
  - `a_song_that_leaves_room_for_another_is_fine` (G 600, all lengths 180:
    picks any, not unfitted).
  - `no_song_starts_that_would_play_too_little` (G 30, lengths 180 and 25:
    picks 25: cut 0).
  - `the_least_cut_wins_when_nothing_fits` (G 100, lengths 180 and 240:
    picks 180, `unfitted`).
  - `a_track_of_unknown_length_is_never_a_fit` (G 200, lengths 0 and 210:
    picks 210).
  - `without_a_fit_the_choice_is_unchanged` (`pick` vs `pick_fitted(None)`
    same pick with the same seed).
- [ ] Run, watch them fail, implement, run green. Commit
  `feat(model): choose a filler track that fits the gap (AU25)`.

### Task 4: Fitting the last songs before a block

**Files:**
- Modify: `crates/fp-model/src/filler.rs` (`top_up`, `settle`, a `refit`),
  `crates/fp-engine/src/conductor.rs` (the log line)
- Test: `crates/fp-model/tests/it/automation_fit.rs`, `crates/fp-engine/tests/it/conductor.rs`

**Produces:**
- `FillerState::unfitted: u64` (runtime): how many picks fell back.
- `FillerState::fitted_for: Option<EntryId>`: the current row the tail was
  last checked for.

Rules:
- `top_up`: when `automation::coming_start` is `Some(start)`, each added
  row gets `Fit { gap_secs: (start - block_fade - row_start) }` from
  `starts_ahead`; otherwise no fit.
- `refit` (in `settle`, before `top_up`): when a block is coming and the
  filler's current row differs from `fitted_for`, walk the ahead rows
  after the loaded next (the next of a stopped filler is not loaded) with
  their starts; the first generated row whose class (Task 3) is none, at
  a gap > 0, is dropped with every generated row after it; `fitted_for` is
  set. `top_up` refills with the fit.
- The conductor keeps the last `unfitted` it saw and logs
  `tracing::info!("filler: no song fits before the block; the one that cuts least plays")`
  when it grows.

- [ ] Tests:
  - `the_filler_ends_on_a_song_that_fits_the_block`: block at 11:00, start
    automation at 10:40, music of lengths 150–330 in steps; walk the rig
    to the hand-over: the song faded at the hand-over had at most 30 s
    left and played at least 45 s.
  - `a_drifting_queue_is_fitted_again_when_a_song_starts`: lengths with a
    segue 10 s before the end; the last song still ends the gap.
  - `the_operator_rows_are_never_refitted`.
  - `nothing_fits_counts_once_per_pick` (`unfitted` grows by the number of
    fallback picks, not per tick).
  - Engine: `the_conductor_logs_a_fallback_fit` is not needed if the
    count is the contract; the line is checked by reading the code
    (`tracing` capture is process-wide, rule in Testing notes).
- [ ] Run, watch them fail, implement, run green; the whole suite. Commit
  `feat(model): fit the filler's last songs before a block (AU25)`.

### Task 5: Docs

- `docs/technical/automation.md`: the fit (timeline, classes, refit once
  per song, fallback and its log line).
- `docs/user/` automation page: the two settings in plain words.
- `docs/technical/configuration.md` (or where the automation fields are
  listed): the two fields.
- Spec AU25: the method chosen, as the builder's call.
- README roadmap line when it names AU25's state.
- Commit `docs: fitting the last songs before a block`.

Then the final review (`superpowers:requesting-code-review`), fixes
test-first, PR.
