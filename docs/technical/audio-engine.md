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
The source is *ready* once `tuning.ready_threshold_ms` (500 ms) is buffered,
measured at the rate the source plays at (the worker scales the threshold).

A **seek** replaces the source: a new one is prepared at the target and the
old one is retired.

## Mixer

`Mixer` (`mixer.rs`) is a slot array. It is built off the RT thread with a
capacity derived from the routing and `tuning.mixer_headroom`, and grown
with `BusCommand::Grow` (the new storage is built by the conductor). For each
block it:

1. drains up to `tuning.max_commands_per_block` commands: `Attach`, `Start
   { at_frame }`, `Ramp { to, frames, curve, at_frame }`, `StopAt`, `Pause`,
   `RampOutBeforeCut`, `Resume`, `Cancel`, `Detach`, `Grow`;
2. mixes every active slot into its route's channel pair. Gain ramps are
   per-sample. Fades use an equal-power curve (`ramp.rs`), and de-click and
   pause ramps are linear. A source that starts inside the audio (`from_secs > 0`:
   a cue-in, a position, the next source of a transition, a CUE, a cart) ramps in
   over `tuning.declick_ms`; one that starts at the file's first frame stays
   hard. A hard cut, and a planned stop without a next source, fade the
   outgoing source out over `declick_ms` (`RampOutBeforeCut`, sent after its
   `StopAt`), except when the source's end of stream falls at the stop frame
   (a gapless join, or a stop at the file's end): the mixer skips the ramp
   when `eof` is set and the frames still buffered are at most the frames left
   to the stop frame plus 2 (`END_TOLERANCE_FRAMES`, for a frame of error in
   the analysed duration and resampling rounding), so the level stays
   constant. A failed source (decoder error) plays out its buffer; when what
   is left is at most `declick_ms` the mixer ramps it to zero over exactly
   those frames, so the end is not a step. A paused slot that has a stop
   frame (a stop while paused) finishes at that frame. The player volume is
   smoothed over `tuning.gain_smoothing_ms`;
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
    (current plus fading sources; the pre-listen's is dropped). The sums are
    taken one by one, inside `BusShared::whole_blocks`: a block rendered
    meanwhile is taken whole in the same reading, never split across ticks.
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
    average), which restarts on every start of an entry (a `StartCurrent`, a
    `Crossfade` or a `TransitionStarted`, so the same entry played again
    counts too) and on `EngineRequest::ResetMeterMax` (a click
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
  The layout is pure (`widgets::meter_layout`): both ends of the scale are
  always labelled (`scale_marks` includes the digital floor; the alignment
  label gives way to an end label), each label is centred on its line or, at
  the rect's edge, rests on or hangs from it (`MeterLine::label_align`), and
  `widgets::reference_segments` says which pieces of a reference line lie
  over the lit part of a bar. The line colours and opacities are the
  `METER_LINE_*` constants in `ui/theme.rs`.
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
The virtual clock keeps the timeline moving while the device opens, and
stops once it has: both may render one block meanwhile, so the timeline can
run one period fast, once per reconnection. A
reader that waits for a block to finish (`BusShared::consistent`,
`whole_blocks`) gives up after 50 ms, so a render thread gone mid-block
cannot hang the conductor.

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
- **Refusals:** a refused rate restores the previous one and is remembered
  (`Bus::refused_rates`), so later starts do not reopen the device to ask
  again; the list is cleared when the device comes back after a loss. On an
  exclusive stream the new rate is tried exclusive only (a refusal there is of
  the rate, not of exclusive access); a bus already shared may change rate
  shared. A double failure leaves the bus `Lost`, for the watchdog. The mixer's
  volume smoothing keeps its duration at the new rate (`Mixer::follow_rate`).
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

### DSD buses

A bit-perfect bus whose device has a DSD mode (`outputs.dsd_output`: `Dop` or
`Native`) can carry a DSD track unchanged (O25). The code is
`engine/dsd.rs` (the decision and the bookkeeping, on the conductor thread),
plus the mixer's DSD mode.

- **The word stream.** DSD travels as 16-bit words (two bytes per channel, the
  first byte first in time), one word per `f32` sample, at the word rate
  (DSD rate ÷ 16), which is also the bus rate. `source_pair_dsd` makes a
  source with two rings of the same length: the words, and the PCM conversion
  of the same file, resampled to the bus rate like any source. They fill and
  drain in lockstep
  (`push_pair`, `pop_pair`), so a seek or an underrun cannot misalign them.
  Each ring holds `tuning.prebuffer_secs` at the word rate, about 14 MB per
  DSD64 source and 56 MB per DSD256 source. The worker reads the raw bytes
  with `DsdRawReader` (`dsd_file_opener`) and converts the same file to PCM
  with `file_opener`.
- **The mixer's DSD mode** (`BusCommand::DsdMode { on, at_frame }`, up to four
  pending switches ordered by frame). In DSD mode:
  - a DSD source's words are copied to the output as they are (no gain, no
    ramp, no resampling); a fade, ramp or volume that falls due settles at
    its target without being applied, and a ramp in progress freezes and
    resumes when the mode ends;
  - every other slot is consumed without being written (muted);
  - the idle fill is the DSD silence word (`0x69 0x69`) instead of `0.0`, also
    when the device lock is missed;
  - the meters read the PCM ring at unity gain;
  - `mark_unaltered` is false, so BP is not lit.
  `BusCommand::HoldAll { from_frame, until_frame }` holds every slot (no
  consume, start or stop) for the blocks overlapping the range; with equal
  frames only the block straddling it. It lines a PCM start up with a mode
  switch, and holds the mixer while a native stream is reopened as PCM.
- **The DoP stage.** `MixerRenderer` (the device-side renderer) runs a
  `DopEncoder` over the block after the mixer, only in DoP streams, keeping
  the marker alternation across blocks; native streams pack the words in the
  backend. On a native stream, `MixerRenderer` turns every block the mixer
  renders out of DSD mode into the DSD silence word: the stream starts before
  `DsdMode` on reaches the mixer, and its first periods must not carry packed
  PCM (`0.0` packs as `0x00` bytes, which the converter would play as a DC
  step).
- **The decision.** `try_start_dsd` runs in `start_current` and `resume` (a
  track loaded paused), the two places that open the source on air. It builds
  `DsdFacts` (the device's mode, the track's format, the player's volume, and
  whether the device is idle) and asks `fp_model::dsd_decision`, which
  returns a `DsdTarget` or a `DsdFallback`: `ModeIsPcm`, `NotDsd`,
  `Multichannel`, `VolumeNotUnity`, `DeviceBusy`, `RateRefused`,
  `FormatTooNarrow` or `StreamRefused`. Every fallback but the first two is
  logged ("DSD converted to PCM"). A track that falls back plays through the
  PCM path. A preload is never turned into a DSD source (it is PCM); a DSD
  start releases a PCM preload of the same entry.
  - `open_dsd_stream` reopens the bus at the word rate with `dsd` set
    (`Bus::reopen_with`, exclusive only) and checks `Bus::dsd_fits`. A refusal
    restores the previous configuration and is remembered per
    `(word rate, kind)` (`refused_dsd`, cleared when the device comes back).
    The refusal is classified as `RateRefused` when the device's listed rates
    lack the word rate, else `StreamRefused`; `FormatTooNarrow` is the
    engine's own DoP format check.
  - Pre-listen and carts never go out as DSD.
- **Silence and tails** (`DsdState`, per bus: `Playing`, `Tail`,
  `Switching`). The DSD silence (`outputs.dsd_silence_ms`) goes out before a
  DSD source starts (`Playing::not_before`), after it ends (`Tail`) and across
  a switch to PCM. A DSD start on a bus in `Tail` at the same kind and word
  rate takes the stream over with no extra silence (the converter is still
  locked); `DsdEnded` is reported for the old entry. At the end of a tail
  (`end_dsd_streams`, from `tick`) a DoP bus leaves DSD mode and a native bus
  is reopened as PCM at the same rate (its stream closes first, so no PCM
  block reaches it). A PCM start over a tail or a switch waits until it ends
  (`before_start_on`).
- **The two mix policies** (`outputs.dsd_mix`), applied by `before_start_on`
  to any start on a bus carrying DSD (the starts loop, `current_finished`,
  `dispatch`, carts, test tones):
  - `ConvertToPcm`: `switch_to_pcm` holds the bus for the DSD silence, then
    leaves DSD mode, and the DSD source goes on from its PCM ring where it
    held. `dispatch` moves the whole transition (the next start, the
    current's stop or fade, the `Dispatched` frame) by that delay. The
    player's own next track comes from a PCM preload, so it always plays
    converted: only the first DSD track after an idle device, or one the
    operator starts (a new start goes through the decision again, and takes a
    tail over), goes out as DSD.
  - `HoldOthers`: nothing interrupts the stream, and the other sources are
    muted by the mixer's DSD mode. The model does not plan an overlapping
    start for a held player (`dsd_holds`: a crossfade becomes a hard start,
    and `StartNextAt` is replaced by a start from `ReachedEnd`, in
    `dsd_hold_continue`), so each track starts after the previous one's tail.
- **Leaving DSD.** The model sends `EngineAction::LeaveDsd` when the volume
  leaves exactly 1.0 (`SetVolume`); `leave_dsd` is `switch_to_pcm`. A fade
  stop of a DSD player is a `Stop` in the reducer (`fade_stop`), and
  `Engine::fade_out` cuts a DSD source with a `Cancel` and a release (no
  fade, no de-click), which also serves `Stop`, `StopNow`, a crossfade out
  of it and a seek (the new source takes its place). Pause and resume use no
  ramp.
- **Telemetry and events.** `PlayerTelemetry::dsd` is true while the current
  source is the `Playing` slot of a DSD bus, and `is_bit_perfect` is false then.
  `EngineEvent::DsdStarted { player, entry, hold_others }` is sent when the
  mixer starts the source (after a re-check of the volume, which switches to
  PCM instead if it left unity meanwhile), and `DsdEnded` when the stream
  ends, switches to PCM or is taken over. The model keeps it in
  `PlayerState::dsd`; `bp_badge` and `dsd_holds_others` drive the badge and the
  notice.
- **Device loss.** The watchdog reconnects through `Bus::reconnect`. A DSD
  configuration comes back only with exclusive access and a stream that fits
  (`dsd_fits`); otherwise the present device is reopened as PCM at the same
  rate, `dsd_lost` is set, and `dsd_stream_lost` drops the record and reports
  `DsdEnded`: the DSD track goes on from its PCM ring. If the PCM open fails
  too, the DSD configuration is restored for the next retry.
- **Tests:** `tests/dsd_mixer.rs` (the mixer's modes, under `assert_no_alloc`)
  and `tests/dsd_output.rs` (the engine on Offline devices, byte for byte);
  `tests/dsd_real_music.rs` is opt-in (see [Testing](testing.md)).

## Engine

`Engine` (`engine.rs`) executes model `EngineAction`s:

| Action | Effect |
|---|---|
| `Preload` | ask the worker for a source of the next entry at its cue-in; attach it idle. While an entry repeats (R26) the model preloads that same entry, and its `StartNextAt` at cue-out restarts it gaplessly (the old pass gets the usual de-click ramp at the cut). The same pair serves O37: a self-next (the entry on air set as next) is preloaded and restarted in the same way, once; the model tells the pass apart from a repeat pass in `on_event` |
| `StartCurrent` | start the preloaded source, or open one, once ready |
| `Crossfade` | start the next source now and ramp the current one down over `fade_ms` |
| `Schedule(plan)` | dispatch a `TransitionPlan` (`StopAt` or `StartNextAt { at_secs, fade_current_until_secs }`) to the mixer as exact frames once it is within `schedule_lead_ms` |
| `FadeOutAndStop`, `StopNow`, `Pause`, `Resume`, `Seek`, `SetVolume` | ramps and commands on the current source. A stop or seek ramps out a source whose start was sent (`Requested`) or whose pause ramp may still run (`pause_ramp_ends`, plus a block of margin); a source that is silent (never started, held, or paused with the ramp over) is released at once |
| `StartCue`, `StopCue` | a separate source on the player's Cue route |
| `SeekCue` | replace the CUE source by one at the target (the CUE plays whole files); it starts idle when the CUE is held, so a held CUE stays held |
| `SetCuePaused` | `BusCommand::Pause`/`Resume` with the pause ramp for an audible source; a source that has not started yet is held idle and started on release. `PlayerRuntime::cue_paused` remembers the state and a new CUE clears it |
| `AddPlayer`, `RemovePlayer` | create or retire a worker and its bookkeeping |
| `LoadPaused` | restore a session: load the source at a position, paused |

The CUE position is `PlayerTelemetry::cue_position_secs`; a held CUE reports a
constant one. A held CUE is released when it is replaced or stopped.

The effective play range is `Track::play_range(use_markers)` in `fp-model`
(`players.use_cue_markers`; off gives 0 to the end of the file, or the source
end while the duration is unknown). `request_from_cue_in`, `request_at`,
`plan_for`, the session restore and `set_marker` (MIX, intro, outro) use it.
Toggling the setting re-preloads the next entry and re-schedules the plan but
never moves a source already sounding. Carts use `cue_in_secs` and
`known_cue_out_secs` and are unaffected.

A known limit: a transition already dispatched to the mixer (within
`schedule_lead_ms` of its frame) still starts the entry it was given, even if
the next changed meanwhile. The model follows the engine
(`TransitionStarted`). A transition counts as executed only once the bus has
rendered past its frame (`now_frame > at_frame`): at exactly that frame it
is still pending, so a fade stop or pause arriving then takes it back
instead of fading a source that never started. A next that is not buffered
yet when its transition is dispatched (or when the current source ends)
starts as soon as it is ready, never before the dispatched frame, rather than
on the frame as silence counted as underruns. It starts only while that
transition stands: a stop, pause, seek or new start takes the transition
back and the waiting next goes back to idle, and a transition counts as
executed only once the next was sent its `Start`.

It turns bus events back into model events: `TransitionStarted`,
`ReachedEnd`, `FadeCompleted`, `SourceFailed`, `PreloadFailed` and
`CueEnded { entry }`. Stale events (for an entry that is no longer current,
or no longer the cue) are ignored by the model. A preload that cannot be
opened is `PreloadFailed`, never `SourceFailed`: the source on air plays on
even when it is the same entry (a repeat). When a command such as Next
arrives in the tick where the mixer has just started a scheduled transition
into something else (a repeating entry's next pass), `start_current`
replaces that start at once and drops its `TransitionStarted`, so the model
stays on the entry it chose. A self-next replay (O37) runs through this same path.

**Routing:** `route_target` maps a configured route to a bus and a channel
pair. A route to a backend this machine does not have falls back to the
default output, so a config copied from another OS still plays.

**Test tones** (`play_test_tone`) render a short sine directly into a source
and attach it to any route. The slot is only taken when both commands fit the
queue, and it is released on `Finished`. The synthesis (1.5 s, a few
milliseconds even at 768 kHz) runs on the conductor, well within
`schedule_lead_ms`.

## Cartwall

Carts play as ordinary sources (`engine/carts.rs`). No real-time code is
added.

- A dedicated worker (`fp-cartwall`) decodes them.
- **Routes:** they play on the cartwall routes (`config.outputs.cartwall`).
  Main falls back to the default output. Without a Cue route, a cart
  pre-listen ends at once (`CartCueEnded { cart }`; the model ignores the end
  of a pre-listen that is no longer the current one).
- **Exact ends and loops:** each load carries `until_secs` (the cue-out) and
  `looped`. The worker cuts the frames past the end, so a cart finishes
  exactly at its cue-out, and fades the last `declick_ms` before it to zero
  (`LoadOptions::fade_out_frames`) so the cut is not a step. A looped cart reopens at its cue-in and keeps
  filling the same ring, so the loop point has no gap (no fade).
- **Stopping:** a started cart, or one whose start was sent (`Requested`), gets
  a de-click ramp; one that never started is released at once. Neither is reported as ended.
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
