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
`STEP_COUNT_RANGE`, 1–99, `min <= max`):

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
cartwall's Main) record `wall_now`. Pre-listen never does. A record keeps
the later of the old and new time, so an airing logged after the clock
went back does not make the track look older. A track removed from the
library loses its time; a duplicate merged into another hands its time to
the survivor. There is one time per track, so the map never outgrows the
library.
