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

After each block the mixer sets `SourceShared::unaltered` for every slot:
every gain it got in the block was exactly 1.0, and no other slot wrote a
non-zero sample into an overlapping channel pair. In `f32`, `x * 1.0` and
`x + 0.0` are exact, so such a source reached the output bit for bit with no
special path. The check compares slots pairwise and allocates nothing.

## Level meters

Metering is split across the threads (spec [`2026-09-27-meters-design.md`](../superpowers/specs/2026-09-27-meters-design.md), whose section M0 lists the documents each part is verified against).

- **The mixer measures** each slot after its gain, on the real-time thread,
  without allocating.
  - The peak: the sample peak, or with `BusShared::true_peak` the largest
    of the four phases of the ITU-R BS.1770-5 Annex 2 interpolator
    (`truepeak.rs`, the published coefficients).
  - For programme meters, the peak goes through a two-stage rectifier
    integrator (`BusShared::ppm_tau1_ms`/`ppm_tau2_ms`, fall
    `fall_db_per_sec`), fitted so that 5 kHz bursts meet EBU Tech 3205
    table 2.
  - Sums of squares, rectified sums (for the VU), and K-weighted sums of
    squares (`kweight.rs`, the ITU-R BS.1770 filters derived for the bus
    rate, `BusShared::sample_rate`). Filter states are flushed below
    1e-20, so silence never runs on subnormal numbers.
  - All of it accumulates in `SourceShared` atomics.
- **The conductor meters.**
  - Every tick, `Engine::take_meter_input` takes a player's measurement
    (current plus fading sources; the pre-listen's is dropped).
  - `meter::MeterState` applies the fall of each preset: 20 dB / 1.7 s,
    24 dB / 2.8 s, 20 dB / 1.5 s, or 26 dB / 3 s (K-System peak).
    Non-finite measurements read as silence.
  - The VU is a second-order needle (99 % in 300 ms, 1.25 % overshoot) on
    the rectified average, calibrated so a sine reads its peak level.
  - It keeps the peak hold; the K-System average, two equal first-order
    stages on the mean square (τ = 102.7 ms, so a step reads 99 % of its
    RMS value in 600 ms) solved exactly per tick and read by the AES17
    convention (a sine reads its peak level); and the maximum, counted
    only from what each tick measured (for the VU, its calibrated
    average), which restarts when the player's current entry changes or
    plays again after a stop, and on `EngineRequest::ResetMeterMax` (a click
    on the readout). It also computes momentary (400 ms) and short-term
    (3 s) loudness from 5 ms blocks: L = −0.691 + 10·log10(z_L + z_R).
  - A tick without a device block within 50 ms leaves the level standing,
    and the next block moves it for the whole span. Longer, the meter
    counts silence (the bar falls, and loudness counts silent frames).
  - `meter::mixer_integration` gives the buses their integrator for the
    configured preset. Settings apply at the next tick: the integrator
    constants and the true-peak flag reach every bus, and a preset change
    starts the needles and integrators from rest. The reading goes into
    `PlayerTelemetry::meter`.
  - The programme-meter stages and the K-weighting flush values below
    1e-20, so silence never runs on subnormal numbers. The VU's work per
    tick is bounded (at most one second of needle movement).
- **The interface only draws** the reading: continuous bars on the scale
  of the chosen standard (`widgets::meter_position`, marks from
  `widgets::scale_marks`), coloured by `widgets::zone_of`, with
  `widgets::max_readout` and `widgets::loudness_line`.
- **Tests** (`tests/metering.rs`, `tests/mixer.rs`):
  - the BS.1770 coefficients and the 1 kHz gain at every rate;
  - EBU Tech 3341 cases 1, 2, 9, 11, 12 and 14 (loudness) and 15–23
    (true peak);
  - Tech 3205 table 2 (EBU PPM);
  - the DIN integration time;
  - the VU's 99 % time, overshoot and average response;
  - every fall rate, and the hold.

## Buses and device loss

A `Bus` (`bus.rs`) is one open device, keyed by `(backend, device)`, with one
mixer shared by every route to that device. It has a watchdog. A backend
error, or no heartbeat for `tuning.watchdog_timeout_ms` (500 ms; startup grace
`tuning.watchdog_startup_grace_ms`, 5 s), marks it **Lost**. A virtual-clock
thread then renders the same mixer into a discard buffer at real-time pace,
so countdowns, segues and chaining continue. The device is reopened every
`tuning.reconnect_interval_ms` (2 s) and takes the mixer back when it opens.

### Rates and bit-perfect buses

- **Each bus has its own rate.** It starts at `outputs.sample_rate`. Every
  seconds↔frames conversion for a source uses the rate of that source's bus:
  fades, declicks, pause ramps, planned transitions, positions, cart loop
  points and test tones (`Engine::rate_of`, `frames_on`). Workers open each
  source at the rate the engine passes in `LoadOptions::rate`.
- **Bit-perfect buses** are named in `outputs.bit_perfect` and opened with
  `StreamConfig::exclusive`. If exclusive access is refused (`Unsupported`),
  the bus reopens shared and `exclusive_granted` stays false.
- **The rate follows the file.** When a source starts on a bit-perfect bus
  (`prepare_start`), the bus is reopened at the file's rate
  (`Bus::reopen_at`) if all of these hold:
  - the file's format is known (`SourceRequest::format`, from analysis);
  - the rate differs;
  - nothing on the bus sounds (`bus_sounding`: a source that is started,
    requested or waiting to be ready, or a test tone).

  Sources start in `start_current`, in `resume` of a track loaded paused, in
  `start_cue`, and when carts are fired or pre-listened.
- **What survives a reopen:** the mixer, its frame counter and its slots.
  Only the stream is replaced, and `reopen_waiting` re-creates the idle
  sources on the bus (preloads, tracks loaded paused) at the new rate.
- **Refusals:** a refused rate restores the previous one. A double failure
  leaves the bus `Lost`, for the watchdog.
- While anything on the bus sounds, the rate never changes, since every
  timeline on the bus is in its frames. Preloads never decide the rate.
- **`PlayerTelemetry::bit_perfect`** is set when all of these hold:
  - the current source is on its Main bus, which is bit-perfect, `Ok`, and
    exclusive;
  - the file rate equals the bus rate;
  - the file's bits are known and `SampleFormat::holds_bits` accepts them
    for the stream's format;
  - the mixer reports the source unaltered.

  This drives the BP badge.
- **Tests:** `tests/bit_perfect.rs` plays WAV files through `file_opener` on
  an Offline device. It covers the rate changes and refusals, and a
  bit-exact comparison of every rendered sample with the file.

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
(`TransitionStarted`). A transition counts as executed only once the bus has
rendered past its frame (`now_frame > at_frame`): at exactly that frame it
is still pending, so a fade stop or pause arriving then takes it back
instead of fading a source that never started.

It turns bus events back into model events: `TransitionStarted`,
`ReachedEnd`, `FadeCompleted`, `SourceFailed` and `CueEnded`. Stale events
(for an entry that is no longer current) are ignored by the model.

**Routing:** `route_target` maps a configured route to a bus and a channel
pair. A route to a backend this machine does not have falls back to the
default output, so a config copied from another OS still plays.

**Test tones** (`play_test_tone`) render a short sine directly into a source
and attach it to any route. The slot is only taken when both commands fit the
queue, and it is released on `Finished`.

## Cartwall

Carts play as ordinary sources (`engine/carts.rs`). No real-time code is
added.

- A dedicated worker (`fp-cartwall`) decodes them.
- **Routes:** they play on the cartwall routes (`config.outputs.cartwall`).
  Main falls back to the default output. Without a Cue route, a cart
  pre-listen ends at once (`CartCueEnded`).
- **Exact ends and loops:** each load carries `until_secs` (the cue-out) and
  `looped`. The worker cuts the frames past the end, so a cart finishes
  exactly at its cue-out. A looped cart reopens at its cue-in and keeps
  filling the same ring, so the loop point has no gap.
- **Stopping:** a started cart gets a de-click ramp; one that never started is
  released at once. Neither is reported as ended.
- **Events:** mixer `Finished` becomes `CartEnded` (or `CartCueEnded`). A
  worker failure becomes `CartFailed` once any buffered audio has played.
- **Mixer capacity:** it counts the carts on air, plus room for more.
- **Telemetry:** `Telemetry.carts` and `cart_cue` carry the positions
  (wrapped within the loop for looped carts).
- **Known limit:** a marker change on a cart's track applies from the next
  fire. A cart already playing keeps the end it started with.

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
