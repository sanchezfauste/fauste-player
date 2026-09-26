# Audio engine (`fp-engine`)

## Sources

A `Source` (`source.rs`) is one decoded stream of one file, split in two
halves:

- the **producer**, owned by the player's worker thread. It decodes with
  `fp-decode`, resamples to the bus rate with rubato (windowed sinc, filter
  delay removed so timelines stay exact), and pushes interleaved stereo `f32`
  into an `rtrb` ring;
- the **consumer**, attached to a mixer slot.

They share a `SourceShared` block of atomics: position, peaks, underruns,
`ready`, `eof` and `failed`. The ring holds `tuning.prebuffer_secs` (5 s).
The source is *ready* once `tuning.ready_threshold_ms` (500 ms) is buffered.

A **seek** replaces the source: a new one is prepared at the target and the
old one is retired.

## Mixer

`Mixer` (`mixer.rs`) is a slot array. It is built off the RT thread with a
capacity derived from the routing and `tuning.mixer_headroom`, and grown
with `BusCommand::Grow` (the new storage is built by the conductor). For each
block it:

1. drains up to `tuning.max_commands_per_block` commands: `Attach`, `Start
   { at_frame }`, `Ramp { to, frames, curve, at_frame }`, `StopAt`, `Pause`,
   `Resume`, `Cancel`, `Detach`, `Grow`;
2. mixes every active slot into its route's channel pair. Gain ramps are
   per-sample. Fades use an equal-power curve (`ramp.rs`), and de-click and
   pause ramps are linear. The player volume is smoothed over
   `tuning.gain_smoothing_ms`;
3. counts underruns (silence is output for missing samples);
4. emits `Started`, `Finished` and `Failed` events and updates `BusShared`
   (heartbeat, frames rendered, peaks). A `render_seq` seqlock lets readers
   take a consistent snapshot.

A slot whose channel pair does not fit the device is consumed silently and
counted (`misrouted`), so it can never leak onto other channels.

## Buses and device loss

A `Bus` (`bus.rs`) is one open device, keyed by `(backend, device)`, with one
mixer shared by every route to that device. It has a watchdog. A backend
error, or no heartbeat for `tuning.watchdog_timeout_ms` (500 ms; startup grace
`tuning.watchdog_startup_grace_ms`, 5 s), marks it **Lost**. A virtual-clock
thread then renders the same mixer into a discard buffer at real-time pace,
so countdowns, segues and chaining continue. The device is reopened every
`tuning.reconnect_interval_ms` (2 s) and takes the mixer back when it opens.

## Engine

`Engine` (`engine.rs`) executes model `EngineAction`s:

| Action | Effect |
|---|---|
| `Preload` | ask the worker for a source of the next entry at its cue-in; attach it idle |
| `StartCurrent` | start the preloaded source, or open one, once ready |
| `Crossfade` | start the next source now and ramp the current one down over `fade_ms` |
| `Schedule(plan)` | dispatch a `TransitionPlan` (`StopAt` or `StartNextAt { at_secs, fade_current_until_secs }`) to the mixer as exact frames once it is within `schedule_lead_ms` |
| `FadeOutAndStop`, `StopNow`, `Pause`, `Resume`, `Seek`, `SetVolume` | ramps and commands on the current source |
| `StartCue`, `StopCue` | a separate source on the player's Cue route |
| `AddPlayer`, `RemovePlayer` | create or retire a worker and its bookkeeping |
| `LoadPaused` | restore a session: load the source at a position, paused |

A known limit: a transition already dispatched to the mixer (within
`schedule_lead_ms` of its frame) still starts the entry it was given, even if
the next changed meanwhile. The model follows the engine
(`TransitionStarted`). This is also why rule 22 can briefly be bypassed at
that exact moment.

It turns bus events back into model events: `TransitionStarted`,
`ReachedEnd`, `FadeCompleted`, `SourceFailed` and `CueEnded`. Stale events
(for an entry that is no longer current) are ignored by the model.

**Routing:** `route_target` maps a configured route to a bus and a channel
pair. A route to a backend this machine does not have falls back to the
default output, so a config copied from another OS still plays.

**Test tones** (`play_test_tone`) render a short sine directly into a source
and attach it to any route. The slot is only taken when both commands fit the
queue, and it is released on `Finished`.

## Conductor

`Conductor` (`conductor.rs`) owns `AppState` and the `Engine`. Each tick
(`tuning.conductor_tick_ms`) it:

1. applies queued `Command`s with `fp_model::apply`. Refusals go back to the
   UI on the `rejected` channel;
2. handles at most one engine request (test tone);
3. ticks the engine: it drains bus events, runs watchdogs, dispatches due
   plans and routes events through `fp_model::on_event`;
4. reconciles (preloads and schedules derived from the state) and publishes
   the new `AppState` and `Telemetry`. `model_version` increases on every
   change, which drives autosave.

`Conductor::tick(now)` can be driven by hand. The engine tests and the stress
tests run it on simulated time with the `Offline` backend.
