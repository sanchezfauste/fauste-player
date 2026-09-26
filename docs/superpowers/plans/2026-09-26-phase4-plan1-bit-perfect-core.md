# Phase 4 · Plan 1 — Bit-perfect core

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans (or superpowers:subagent-driven-development) to implement this plan task by task.

**Goal:** a device can be set to bit-perfect. On it:
- tracks play at their own rate, and the bus reopens at a new rate while it is idle;
- the samples reach the device unchanged at 100 % volume with no fade or overlap;
- the player's BP badge lights only while that is true.

This plan covers ALSA `hw:` devices and the Offline backend. WASAPI exclusive and Core Audio hog mode are Plan 2.

**Architecture:**
- Analysis records each file's rate and bits (`AudioFormat`). The model stores it on the track, and `SourceRequest`/`CartRequest` carry it, so the engine decides with no I/O.
- Each bus keeps its own rate. Every seconds↔frames conversion for a source uses the rate of that source's bus, and the worker opens each source at the rate the engine passes in `LoadOptions.rate`.
- A bit-perfect bus that is idle reopens its stream at the incoming file's rate, keeping the mixer.
- The mixer marks each source *unaltered* per block (gain exactly 1.0, alone on its channel pair). Telemetry combines that with the bus and format facts into `bit_perfect`.

**Tech stack:** existing crates; cpal ALSA `hw:` devices; no new dependencies.

**Spec:** [`docs/superpowers/specs/2026-09-26-phase4-bit-perfect-design.md`](../specs/2026-09-26-phase4-bit-perfect-design.md) (B1–B8), parent spec §4.8, §5.2, §13.

## Global Constraints

- **Rules carried over:**
  - English;
  - no third-party product names;
  - `forbid(unsafe_code)`;
  - no unwrap, expect or panic outside tests;
  - `indexing_slicing` denied in fp-engine, fp-backends, fp-decode and fp-analysis;
  - real-time rules on the mixer path: no allocation, lock, log or panic;
  - no hardcoded limits;
  - TDD;
  - every string in both `en-US` and `es-ES`;
  - docs kept in sync.
- **A non-bit-perfect bus behaves exactly as before:** it keeps the configured rate, and the whole existing suite stays green.
- **A bus's rate never changes while a source is attached to it** (B3.3).

## Review Focus

1. **A 44.1 kHz track segues into a 48 kHz one on a bit-perfect bus.** The second is resampled, with the badge off. There's no gap, no jump in timing, and the rate is unchanged until the bus is idle.
2. **A reopen at a rate the device refuses:** the bus falls back to its previous rate, and the track plays resampled. Nothing goes silent or stalls.
3. **A track not yet analysed** (no format) on a bit-perfect bus plays at the bus rate, and the badge is off.
4. **Volume moved to 99 % and back during playback:** the badge goes off, then on again once the volume ramp settles at exactly 1.0.
5. **A config written by Phase 3** (no `bit_perfect` key) and a library without formats both load unchanged.

---

### Task 1: The file format through analysis, model and requests

**Files:**
- `crates/fp-decode/src/lib.rs`: `FileDecoder::bits_per_sample() -> Option<u32>`, from the codec params, `None` for lossy codecs.
- `crates/fp-model/src/track.rs`:
  - `AudioFormat { sample_rate: u32, bits: Option<u32> }` (serde, Copy);
  - `TrackAnalysis.format: Option<AudioFormat>` (serde default);
  - `Track.format: Option<AudioFormat>` (serde default);
  - `apply_analysis` stores it.
- `crates/fp-model/src/command.rs`: `SourceRequest.format: Option<AudioFormat>`, `CartRequest.format: Option<AudioFormat>`.
- `crates/fp-model/src/state.rs`: requests take `track.format`.
- `crates/fp-analysis/src/analyze.rs`: fills `format`. `ANALYSIS_VERSION` = 4.
- Every other `SourceRequest {` and `CartRequest {` literal gets `format: None`.

**Tests:**
- `the_analysis_records_the_rate_and_bits`: a 16-bit, 44.1 kHz WAV gives `Some(AudioFormat { sample_rate: 44_100, bits: Some(16) })`.
- `requests_carry_the_track_format`: model; `StartCurrent`'s request has the track's format.
- `a_library_without_formats_loads`: fp-store; old JSON has `format: None`.

### Task 2: Backend API — exclusive access, sample format, exclusive-capable devices

**Files:**
- `crates/fp-backends/src/lib.rs`:
  - `StreamConfig.exclusive: bool`;
  - `pub enum SampleFormat { F32, I32, I16 }` with `holds_bits(bits: u32) -> bool` (F32 ≤ 24, I32 ≤ 32, I16 ≤ 16);
  - `OutputStream::sample_format(&self) -> SampleFormat`.
- **cpal backend:**
  - `exclusive_capable` = ALSA host and a device id starting with `hw:`;
  - `open_output` with `exclusive` on a device that is not capable returns `BackendError::Unsupported("exclusive access is not available on this device")`;
  - the stream reports the chosen format.
- **Offline:**
  - devices can be marked exclusive-capable and can refuse rates (`OfflineDevice::refuse_rate(u32)`, for the fallback test);
  - `OfflineDevice::config() -> Option<StreamConfig>` gives the open stream's config;
  - Offline streams report `F32`.
- **Null:** `F32`, not exclusive-capable. It accepts `exclusive` and ignores it: it is the last resort.
- **Tests:**
  - `conversion_to_i16_is_exact_for_16_bit_pcm`: every i16 value `v` as `v / 32768.0` converts back to `v`;
  - `conversion_to_i32_is_exact_for_24_bit_pcm`: a sweep of 24-bit values `v`, as `v / 8388608.0`, converts back to `v << 8`;
  - `alsa_hw_devices_are_exclusive_capable`: a pure function `exclusive_capable(host, id)`;
  - `sample_formats_hold_their_bits`;
  - `exclusive_is_refused_where_unavailable`: pure function.

  The conversion tests pin existing, correct behaviour. They are guards: they pass on the first run, and that is recorded in the ledger.

### Task 3: Rates per bus

A refactor with no change in behaviour.

- `EngineSettings::frames(ms)` becomes `frames_at(rate, ms)`. Every call site uses the rate of the bus concerned: `Bus::sample_rate()`, or `Playing.rate` for a source.
- `Playing` gains `rate: u32`, the bus rate when it was attached. Positions and plans use it.
- `LoadOptions.rate: u32`. The worker opens the source at `options.rate`. `PlayerWorker::spawn` no longer takes a rate.
- Carts (`engine/carts.rs`) and test tones do the same.
- **Tests:** the whole suite stays green. The behaviour this enables, a bus at a rate other than the configured one, is tested in Task 4: `an_idle_bit_perfect_bus_reopens_at_the_file_rate` also checks that the position after one second of rendering at 44.1 kHz is 1.0 s.

### Task 4: Bit-perfect buses follow the file rate

**Files:**
- `crates/fp-model/src/config.rs`:
  - `OutputsConfig.bit_perfect: Vec<OutputDevice>` (serde default);
  - `OutputDevice { backend: String, device: String }`;
  - lenient loading as for routes.
- `crates/fp-engine/src/engine.rs`:
  - `EngineSettings.bit_perfect: HashSet<BusKey>`;
  - a bus is created with `exclusive = true` when bit-perfect;
  - `fn rate_for(&mut self, bus: &BusKey, format: Option<AudioFormat>, now) -> u32` runs before each attach (players and carts). If the bus is bit-perfect, idle (no used slots), and the format's rate differs, it calls `Bus::reopen_at(rate, now)`. It returns the bus's rate afterwards.
- `crates/fp-engine/src/bus.rs`:
  - `Bus::reopen_at(rate, now) -> bool`: drop the stream, set `config.sample_rate`, `try_open`. On failure, restore the old rate and `try_open` again (the virtual clock covers a double failure). It returns whether the new rate took.
  - If an exclusive open is refused with `Unsupported`, it is retried without `exclusive`, and the bus records `exclusive_granted = false`.
- **Tests (Offline, real WAV files through `file_opener`):**
  - `an_idle_bit_perfect_bus_reopens_at_the_file_rate`: 44.1 kHz file; the device's config becomes 44 100;
  - `a_busy_bit_perfect_bus_keeps_its_rate`: 48 kHz playing, then a 44.1 kHz preload; the rate stays 48 000;
  - `a_normal_bus_never_changes_rate`;
  - `a_rate_the_device_refuses_keeps_the_previous_rate`: `refuse_rate(44_100)`; it still plays;
  - `an_unknown_format_plays_at_the_bus_rate`;
  - `bit_perfect_config_round_trips_and_old_configs_load`: model/store.

### Task 5: Unaltered sources, telemetry and bit-exact output

**Files:**
- `crates/fp-engine/src/source.rs`: `SourceShared.unaltered: AtomicBool`.
- **`crates/fp-engine/src/mixer.rs`:** each slot's block records:
  - `unity`: every gain exactly 1.0;
  - its pair and whether it wrote a non-zero sample.

  After all slots are rendered, a slot is unaltered when `unity` holds and no other slot wrote non-zero samples on an overlapping pair. This uses a fixed-size pass over the slots with no allocation.
- **`crates/fp-engine/src/engine.rs`:** `PlayerTelemetry.bit_perfect: bool` per spec B5. It needs:
  - the bus is bit-perfect, `Ok`, and `exclusive_granted`;
  - `playing.rate == format.sample_rate`;
  - the bits are known and `stream.sample_format().holds_bits(bits)`;
  - `shared.unaltered`.
- **Tests:**
  - `bit_perfect_playback_is_bit_exact`: a 16-bit WAV of distinct values on a bit-perfect Offline device. Every rendered sample equals `v / 32768.0` exactly, and the telemetry's `bit_perfect` is true.
  - `volume_below_full_is_not_bit_perfect`: the badge is false at 0.99, and true again after returning to 1.0 and the smoothing.
  - `an_overlapping_source_is_not_bit_perfect`: a cart on the same pair.
  - `a_resampled_source_is_not_bit_perfect`.
  - `mixer_marks_unaltered_only_at_unity_and_alone`: a mixer unit test.

### Task 6: Interface — BP badge and Settings

- The player header's BP badge is active when `telemetry.bit_perfect` is set. The tooltip is `bp-on` or `bp-off`.
- **Settings → Audio outputs → Bit-perfect devices:**
  - one row per distinct device used by a route (players and cartwall), with a checkbox;
  - disabled with the `bp-not-capable` tooltip when `DeviceInfo.exclusive_capable` is false;
  - writes `config.outputs.bit_perfect`, with the restart note.
- Strings are in `en-US` and `es-ES`.
- **Tests (kittest):**
  - `the_bp_badge_follows_telemetry`;
  - `a_device_can_be_marked_bit_perfect`;
  - `a_shared_device_cannot_be_bit_perfect`.

### Task 7: Documentation and verification

- **User guide:**
  - `settings.md`: bit-perfect devices;
  - `players.md`: the BP badge;
  - `troubleshooting.md`: a gap when the rate changes, and `hw:` devices busy while a sound server holds the card;
  - a new section on bit-perfect listening, with the manual loopback check (record the device's output digitally and compare it with the file, e.g. with `sox` or a null test).
- **Technical docs:**
  - `audio-engine.md`: per-bus rates, the rate change, unaltered sources;
  - `backends.md`: `exclusive`, `sample_format`, exclusive-capable devices;
  - `analysis.md`: the format and version 4.
- README feature list and roadmap: Phase 4, plan 1 done.
- Full checks, a fresh review, fixes, then merge.
