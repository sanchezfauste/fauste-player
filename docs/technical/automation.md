# Automation

The design is the [automation spec](../superpowers/specs/2026-11-11-automation-design.md)
(rules `AU<n>`). This page describes what is implemented so far: the music
filler's selection rules and player, the schedule's data model, running
the schedule and its as-run log, pure functions in `fp-model`. The
Log column and the log editor are described in [ui](ui.md).

## Time and randomness (`wall_time.rs`)

The model never reads the clock. `AppState::wall_now` (a `WallTime`: UTC
Unix milliseconds) is set by the conductor at the start of every tick, and
by tests directly, so every rule that depends on the time is a pure
function of the state. `AppState::rng` is a `SplitMix64` the conductor seeds
from the clock when it starts; tests seed it with a fixed value. Both are
runtime only.

## Rotations (`rotation.rs`)

A `Rotation` has a name and an ordered list of `RotationStep`s. A step
names its `StepSource` and a count range `min..=max` (`min` in
`STEP_MIN_RANGE`, 0–99; `max` in `STEP_MAX_RANGE`, 1–99; `min <= max`;
the database loader clamps into them with a warning); a rotation needs at least one
step (a damaged database may still hold one without, and the walk then
falls back on the default):

- `StepSource::Category(id)`: the category with all its subcategories;
- `StepSource::Kind(kind)`: every track of that kind.

`Rotations` keeps them in display order with the default rotation (AU4).
The first rotation created becomes the default; deleting the default makes
the first remaining one the default; the last rotation cannot be deleted.
At every start the app calls `rotation::ensure_default`, which creates
"Music" (one step, all music, 1–1) when there is no rotation at all. A step
on a deleted category stays stored; selection skips it.

The walk (`rotation_cursor.rs`) draws a step's count each time it comes
up; a draw of 0 passes the step over. `walk` goes over the steps at most
twice from the cursor: in the second pass every draw is at least 1, so a
pass where all steps drew 0 still plays the rotation instead of falling
back on the default.

Commands: `CreateRotation`, `UpdateRotation`, `DeleteRotation`,
`SetDefaultRotation`.

## Air times (`air_times.rs`)

`AppState::air_times` holds, per track, when it last started on air
(AU35): `advance_to` (any player's Main) and `cart_rules::fire` (the
cartwall's Main) record `wall_now`. Pre-listen never does. A record replaces the stored
time: the latest airing is the truth, even when the clock went back. The
conductor brings every time after the clock back to it when it starts and
whenever the clock goes back (`AirTimes::clamp_future`, with a log line),
so a stamp left by a clock that was ahead never holds a track or an artist
back for as long as the clock was wrong. Loading keeps the later of two
times for a track. A track removed from the
library loses its time; a duplicate merged into another hands its time to
the survivor. There is one time per track, so the map never outgrows the
library.

## Selection (`selection.rs`)

`pick(state, source, at, planned, rng)` chooses the next track of a step
(AU31–AU34). `pool` is the source's playable tracks. `planned` lists tracks
already queued but not yet aired, with their expected start: they count as
aired then, so a look-ahead never queues two tracks that break the rules
between themselves. `last(t)` is the later of the air time and any planned
time.

- **Track separation (AU31).** A track that never aired is eligible.
  Otherwise it is eligible when `track_separation_mins` have passed since
  `last(t)`, **or** when at least `ceil(track_separation_percent × (pool − 1)
  / 100)` other tracks of the pool aired after it. A criterion set to 0 is
  off; with both off every track is eligible. A time in the future (the
  clock went back) has not "passed". With no other track in the pool, the
  percentage is met trivially.
- **Artist separation (AU32).** No track of the library whose artist
  (trimmed, lower-cased) equals the candidate's aired within
  `artist_separation_mins` before `at`. An empty artist is exempt.
- **Fair choice (AU33).** The eligible tracks are ordered by `last`
  (never aired first, then by id), and the pick is uniform among the first
  `PICK_SPREAD` (5).
- **Relaxing (AU34).** With no track meeting both rules, artist separation
  is dropped (`Relaxed::Artist`); with none meeting track separation
  either, the whole pool is used (`Relaxed::Track`), least recently aired
  first. An empty pool gives `None`.

The counts behind the percentage come from the pool's sorted air times
(a binary search), so one choice is O(n log n) in the pool's size.
`eligible` returns the tracks that meet both rules, in that order, for
tests and for the editor.

## Walking a rotation (`rotation_cursor.rs`)

A `RotationCursor` (rotation, step, `left`, `fresh`) is where the filler
stands in its rotation; it is plain data, saved with the filler.
`next_track` moves it on and returns a `CursorPick`:

- a fresh cursor starts at step 0; when a step has played its count, the
  next step comes up (after the last, the first again), with a count drawn
  afresh in its `min..=max` (AU2);
- a step whose source yields nothing (a deleted or empty category, every
  file missing) is skipped and listed in `skipped_steps` for the caller's
  log; one call tries every step at most once;
- an unknown rotation, one without steps, or one whose steps all came up
  empty falls back on the default rotation through a cursor of its own, so
  the caller's cursor keeps its place (`used_default`, AU34). With nothing
  playable anywhere the result is `None`.

## The music fill (`filler.rs`)

The filler (AU27–AU29) is an ordinary `PlayerState`, kept **last** in
`AppState::players`, with a playlist of its own; `AppState::filler`
(`FillerState`) names both. Fades, segues, routes, preloading and the engine
are the players' own. `AppState::aux_players()` is every player but the
automation players (the filler and the schedule's): the player count, the numbered shortcuts and MIDI controls, the
saved player sessions, the remote API and the phone count and number only
those. Rules about sound (on air, playout, pre-listen) see the filler too.

`filler::settle` runs at the end of every command and engine event, before
`reconcile`:

- **trim:** the rows before the current one (or before the next, when
  stopped) beyond `automation.filler_played_kept` leave the list, through
  `take_out_entries`, so the list never grows;
- **ahead:** the rows from the player's next on, when next is after the
  row on air (the rows between will not play, and are not history
  either: they stay in the list); else the rows after the row on air
  (rework spec AR13). A filler with no next, playing or stopped, takes
  the first row the top-up adds as next;
- **top up:** while the rows ahead play for less than
  `automation.filler_lookahead_mins`, it asks the rotation cursor for the
  next track (`next_track`, with every row ahead passed as `planned` at its
  expected start) and appends it. A row whose track has no known length
  counts `UNKNOWN_LENGTH_SECS` (180 s), and one settle adds at most
  `MAX_ADDED_PER_SETTLE` (50) rows. When nothing can be added and nothing is
  ahead, `starved` is set for the column's alert.

`FillerState::generated` holds the rows the filler added. A row the
operator inserts or moves is not in it, and `SetFillerRotation` replaces
only generated rows after the current one (keeping the next of a playing
filler, already loaded for the hand-over). A removed row was never aired,
so it does not count for separation.

The filler's list is not an operator playlist. The reducer refuses with
`ModelError::FillerPlaylist`:
- renaming or deleting it;
- showing it on another player, or another list on the filler;
- a next for the filler from another list (top-up and trim count from
  where its next stands in its own list).

Deleting a playlist never leaves the operator without one of their own
(`LastPlaylist`) and never falls back to the filler's list. A lower
`limits.max_players` in `UpdateConfig` counts only the numbered players.
After `DeleteRotation` or `SetDefaultRotation`, `filler::retarget` moves a
filler whose rotation was deleted, or that follows the default, to the
default as `SetFillerRotation(None)` would. Settings → Playlists lists
the playlists without the filler's.

## The schedule model (`schedule.rs`, `local_time.rs`)

The schedule's data (AU5–AU14) lives in `AppState::schedule`, a `Schedule`
of three parts:

- **day templates** (`DayTemplate`): a name and `blocks`, kept in start
  order with no two at one start (`BlockTimeTaken`). A `Block` has a stable
  `BlockId`, a `TimeOfDay` (seconds after local midnight, `0..86 400`) and
  `items`;
- **the week plan** (`WeekPlan`): a template per weekday, Monday first. A
  template the week plan uses cannot be deleted (`TemplateInUse`), nor the
  last one (`LastTemplate`);
- **date plans** (`DatePlan`, one per `NaiveDate`): another template for the
  date (AU10) and `changes`, each a `DateChange`:
  - `Changed { block, own_time }`: a template block's content for that date.
    `own_time` is set when the date moved it; otherwise it follows the
    template block's start.
  - `Removed(id)`: the template block does not play that date.
  - `Added(block)`: a block of the date alone.

  A plan that no longer changes anything is removed.

An `Item` is a track, one track of a category, or a rotation, which must be
the block's last item (`check_items`, `RotationNotLast`).
`Item::is_time_critical` is AU7: an Hourly ID track, a category of Hourly ID
kind, or an item the operator marked.

The commands are:

- for templates: `CreateTemplate`, `DuplicateTemplate` (fresh block ids),
  `RenameTemplate`, `DeleteTemplate`, `AddBlock`, `UpdateBlock`,
  `RemoveBlock` and `SetWeekTemplate`;
- for dates: `SetDateTemplate`, `ChangeBlockOnDate`, `AddBlockOnDate`,
  `RemoveBlockOnDate` and `RevertBlockOnDate` ("back to the template").

The date commands name blocks of the date's computed day.

`Schedule::day(date)` computes a date (AU13) as a list of `DayBlock`s
(`origin`: `Template`, `Changed` or `DateOnly`):

1. It takes the template's blocks (`template_for`: the date's template if it
   still exists, else its weekday's).
2. It applies the changes. A `Changed` whose template block is gone (deleted,
   or the date switched template) becomes a `DateOnly` block at its own
   start.
3. It sorts by start. When two blocks share a start, the date's block wins
   over the template's; between two blocks of the same kind, the lower id
   wins.

Local time is passed in: `block_start(zone, date, start)` turns a block's
start into a `WallTime` through the `LocalZone` trait (implemented for
`chrono::Local` and `FixedOffset`; tests use their own zone). A time that
does not exist on the date (the spring change) gives `None`, and the block
is missed. A time that occurs twice (the autumn change) gives the first
occurrence (AU14).

`prune_dates(state, today)` deletes the date plans older than
`automation.overrides_keep_days` (AU38). The app calls it at start-up. The
store saves the schedule in schema 4 ([persistence](persistence.md)).

## Running the schedule (`automation.rs`)

`AppState::automation` (an `AutomationState`) runs the schedule (AU15–AU24,
AU40). The schedule plays through its own player and list, made by
`automation::ensure` like the filler's:

- **The players:** `players` holds the schedule's player just before the
  filler's, both last. `player_ids` shows the schedule first, then the
  Music fill column, then P1…. `is_automation` and `is_automation_playlist` keep
  both players and their lists out of the operator's lists, the remote API
  and the player count, and the reducer refuses edits to them with
  `FillerPlaylist`.
- **Local time:** the model never reads the clock. The conductor sets
  `local_now` (local date and time) next to `wall_now` on every tick, then
  calls `automation::tick`. Block starts are local date-times (a date and a
  `TimeOfDay`), compared with `local_now`. A block in the spring gap is
  never reached, so it is missed. In the autumn overlap a block plays once
  (AU14), because `started` keeps it from playing again.
- **Next** (`ItemRef`: a date, a block and an item index) follows the clock
  (`clock_next`): the first block of yesterday, today or tomorrow that has
  not started and starts after `local_now − automation.late_tolerance_secs`.
  The operator may set it by hand (`SetScheduleNext`), and it stays until
  Start or `ClearScheduleNext`. `settle_next` runs after every step.
- **Missed blocks:** a block whose start passes since the last look (`seen`)
  is marked missed in `missed` when automation is stopped, between blocks,
  or while a block chosen away from its time plays. A start that passes
  while a block plays in order waits for it (AU21).
- **Start** (`StartAutomation`): a next set by hand plays at once, with the
  filler fading out. A next from the clock plays at its block's start, with
  the filler until then.
- **Loading a block:** `load_block` puts the block's items into the
  schedule's list:
  - a track item becomes its track;
  - a category item becomes a track picked by the §6 rules;
  - a rotation item ends the rows, and the filler plays it after the block;
  - an item with nothing playable, or a hard-start item already past
    the late tolerance, is missed.

  The rows play in order with the player's crossfades and segues (AU18).
- **Each tick** (`drive`):
  - **A block ends** when its player is idle. The filler takes the block's
    rotation or the default (AU20). If the block played in order, the
    earliest block that came due meanwhile plays now (AU21). Otherwise next
    from the clock plays at its time.
  - **The hand-over (AU24):** `automation.block_fade_ms` before a block, the
    filler fades out (`reducer::fade_stop_over`), and `handing_over` keeps
    it from restarting. At the start the block plays.
  - **A late hard-start item (AU22):** a next row that is hard-start
    and would start beyond the block's start plus the tolerance is taken
    out and missed. When it would start is estimated from when the row on
    air started (`row_since`) and its length.
  - **A next set by hand between blocks** follows the filler song on air.
    Nothing is cut.
- **Clock jumps (AU40):** a step of `local_now` back, or forward by more
  than `CLOCK_JUMP_SECS` (60), recomputes next before anything plays.
  Blocks jumped over are missed, and blocks already started never play
  again.
- **The filler's look-ahead (AU27):** while automation runs between blocks,
  `coming_start` limits it to the start of the filler's fade before the
  coming block, and generated rows that would start after the fade leave
  the list.
- **No fragments (AU25):** see "Fitting the last songs" below.
- **Stop after current** (`StopAutomationAfterCurrent`) sets
  `stop_after_current` on whichever of the two players plays, and stops
  automation. Nothing then plays by itself.

The hand-over is exact to one conductor tick (`tuning.conductor_tick_ms`),
not to the sample, as the spec's AU24 accepts. `AutomationSession` (the `kv`
value `automation`) keeps the schedule's player and list ids across a
restart. Automation always comes back stopped.

## Editing helpers (`schedule.rs`)

`schedule::overrun_secs(state, items, start, next_start)` is AU37's figure:
the items' known lengths (a track's play range, a category's shortest
playable track, a rotation nothing) against the time to `next_start`
(midnight when `None`, the next day's when not after `start`); `Some` only
when positive. `schedule::with_tracks(items, tracks)` adds track items
before a final rotation item, else at the end. The editor (`ui/schedule_editor.rs`)
sends the plan 3 commands only.

## The Log column's rows (`schedule_view`)

`automation::schedule_view(state)` lists what the Log column draws
(§8.1): yesterday's last block, today's blocks and tomorrow's first, in
time order, widened to keep a block on air from yesterday (an overrun past
midnight) and the next item's block, with every block between. Each is a
`ScheduleRow::Block` (date, start, `BlockOrigin`, status) followed by its
`ScheduleRow::Item`s (`ItemRef`, the item, a track's length when known,
hard-start, status). The next shown is `AutomationState::next`, except
while a block plays in order: then the block that came due meanwhile
(`overdue_after`), which is what will play (AU21). A `RowStatus` is
`OnAir` for the item of the schedule player's current row, `Next`, then
`Missed` when in `missed`, `Aired` for the items before the one on air and
for a started block no longer on air, else `Coming`. A block is `OnAir`,
`Next` (next is in it), `Missed` (all its items), `Aired` (started) or
`Coming`.

`load_block` keeps `missed` true to AU23: the rows of the block on air
that a next set by hand drops, and the items before the first of a block
played from its middle that never started, are missed; an item it queues
is no longer missed (a missed item played by hand).

## Fitting the last songs (AU25)

Before a block, the filler chooses its last songs by length so that the
song faded at the hand-over loses at most `automation.max_fade_cut_secs`,
and no song starts that would play less than
`automation.min_song_play_secs` before the fade. It is a greedy fit of
known lengths, a classic method.

- **The timeline** (`filler::starts_ahead`): `FillerState::current_since`
  notes when the model first saw the filler's row on air. Each row's lead
  (`filler::lead_secs`, from `selection::timing`) runs to its segue point
  when `players.auto_segue` is on and the track has one inside its play
  range, as the player's own plan; otherwise to its end. The first row
  ahead starts when the row on air reaches its lead, and each next one a
  lead later. While a block is coming, the top-up and the drop of rows
  beyond the block both use this timeline, up to the start of the fade
  (`block_fade_ms` before the block). A row still audible when the fade
  starts is the last: nothing is queued after it (even a loaded next
  leaves when the row on air is that row), so the player plays it into
  the fade instead of segueing into a song that would be cut at once.
- **The choice** (`selection::pick_fitted` with a `Fit`: the gap from the
  row's start to the fade, the cut, the minimum). Each track of known
  length falls in a class, best first:
  - *ends*: it is at least as long as the gap, the cut is within the
    limit, and the gap is at least the minimum (a track as long as the
    gap that fails this has no class);
  - *room*: it is shorter than the gap, its lead leaves at least the
    minimum, and
    some track of the pool could close what is left (or enough is left
    for more);
  - *loose room*: it is shorter than the gap and its lead leaves at least
    the minimum.

  Each class is tried with both separation rules, then the track rule
  alone; then every class with the separation relaxed. When no track
  fits, the one that cuts least wins (then the one that leaves least, then
  the least recently aired), `Picked::unfitted` says so, and
  `FillerState::unfitted` counts it; the conductor logs a line when that
  count grows. Far from the block (a gap of at least twice the pool's
  longest track plus the minimum) every known track leaves room, and the
  choice is the ordinary `pick`, so tracks of unknown length still play
  then.
- **The refit** (`filler::refit`): segues and edits make the queue drift.
  Once per song on air (`FillerState::fitted_for`), the generated rows
  after the loaded next are checked against the timeline; the first that
  no longer fits (`selection::fits`) leaves with every generated row after
  it and the generated row just before it (that one often left the gap
  nothing closes), and the top-up chooses again. The current row, the loaded next and
  the operator's rows are never touched, so a next loaded before
  automation started may still be cut.

## The as-run log (`as_run.rs`)

`AppState::as_run` (an `AsRunLog`) records what aired and what was missed
(AU36). `reducer::settle_all` ends with `as_run::record`, which compares
what the two automation players have on air (playing or paused: a pause
and its resume are one airing) and the
`missed` set with what it saw last:

- a new row on air on the schedule's player that maps to an `ItemRef`
  is an `Aired` entry from the `Schedule`, with the block's start and the
  local time it started;
- a new row on air on the filler's player is an `Aired` entry from the
  `Filler`, with no scheduled time;
- each item newly in `missed` is a `Missed` entry, with no start.

A missed item played by hand later has both entries. Each entry takes the
next sequence number, which only grows (`resume_at` never goes back), so
the store appends exactly what it has not saved, whatever the clock did.
The model keeps only yesterday's and today's entries; the store keeps
`automation.as_run_keep_days` (AU38; see [persistence](persistence.md)).
Viewing the log beyond the Log column and exporting it are non-goals.

AU39's long-run test (`fp-store/tests/it/automation_long_run.rs`) runs
thirty simulated days with a week plan, a date override, the filler and a
save every hour, through the model and the store with a small fake engine,
and checks that the lists, marks, in-memory log and `as_run` table stay
bounded and that every block started on time. It also restarts the store
half way, checks that the database holds exactly the logged entries since
the cutoff (each sequence once), that the filler played every day, and
that the `entries` and `air_times` tables stay bounded.

