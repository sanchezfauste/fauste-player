# Automation

The design is the [automation spec](../superpowers/specs/2026-11-11-automation-design.md)
(rules `AU<n>`). This page describes what is implemented so far: the music
filler's selection rules, pure functions in `fp-model`. The filler player,
the schedule and their screens come in later plans.

## Time and randomness (`wall_time.rs`)

The model never reads the clock. `AppState::wall_now` (a `WallTime`: UTC
Unix milliseconds) is set by the conductor at the start of every tick, and
by tests directly, so every rule that depends on the time is a pure
function of the state. `AppState::rng` is a `SplitMix64` the conductor seeds
from the clock when it starts; tests seed it with a fixed value. Both are
runtime only.

## Rotations (`rotation.rs`)

A `Rotation` has a name and an ordered list of `RotationStep`s. A step
names its `StepSource` and a count range `min..=max` (both in
`STEP_COUNT_RANGE`, 1–99, `min <= max`); a rotation needs at least one
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

## The music filler (`filler.rs`)

The filler (AU27–AU29) is an ordinary `PlayerState`, kept **last** in
`AppState::players`, with a playlist of its own; `AppState::filler`
(`FillerState`) names both. Fades, segues, routes, preloading and the engine
are the players' own. `AppState::aux_players()` is every player but the
filler: the player count, the numbered shortcuts and MIDI controls, the
saved player sessions, the remote API and the phone count and number only
those. Rules about sound (on air, playout, pre-listen) see the filler too.

`filler::settle` runs at the end of every command and engine event, before
`reconcile`:

- **trim:** the rows before the current one (or before the next, when
  stopped) beyond `automation.filler_played_kept` leave the list, through
  `take_out_entries`, so the list never grows;
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
