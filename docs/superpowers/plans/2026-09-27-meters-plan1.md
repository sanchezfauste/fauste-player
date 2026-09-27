# Meters · Plan 1 — Standard level meters

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans (or superpowers:subagent-driven-development) to implement this plan task by task.

**Goal:** replace the flickering meter with standard, configurable meters:
- ballistics presets: IEC 60268-18, 60268-10 (EBU and DIN) and 60268-17, plus custom;
- sample or true peak;
- configurable scale zones;
- an EBU R128 loudness readout.

**Architecture:**
- **The mixer measures:** peaks, mean square and K-weighted mean square per slot, accumulated in atomics.
- **The conductor meters:** a pure `MeterState` per player applies ballistics and loudness windows every tick with the live configuration, and publishes readings in the telemetry.
- **The interface draws** those readings.

**Spec:** [`docs/superpowers/specs/2026-09-27-meters-design.md`](../specs/2026-09-27-meters-design.md) (M1–M5).

## Global Constraints

- Rules carried over:
  - English;
  - no third-party names;
  - real-time rules in the mixer (no allocation, lock, log or panic);
  - `forbid(unsafe_code)`;
  - `indexing_slicing` denied in fp-engine;
  - every operator-facing value in `Config` with a default, a range and lenient loading;
  - strings in both `en-US` and `es-ES`;
  - TDD;
  - docs kept in sync.
- The mixer's no-allocation test (`assert_no_alloc`) stays green with metering on.

## Review Focus

1. **A player stopping mid-song:** the bar falls at the preset's rate, the hold drops after its time, and the loudness readout clears. Nothing freezes at the last value.
2. **A crossfade:** both sources contribute to the player's meter, and loudness does not double-count one source.
3. **Settings changing mid-playback** (preset, true peak, zones): the meter follows at once, and nothing stalls.
4. **Bit-perfect playback** still passes the bit-exact test: measuring never alters the samples.
5. **Buses at 44.1 kHz or 96 kHz** (bit-perfect rate changes): the K-weighting stays correct at every rate.

---

### Task 1: Configuration

- **`fp-model`:**
  - `MeterConfig`, `MeterBallistics` and `LoudnessReadout`, in `Config.meter` with serde defaults;
  - validation as in spec M3.
- **Tests:**
  - `meter_defaults_follow_the_standards`;
  - `meter_zones_are_kept_in_order`;
  - store: `a_config_without_meter_loads_defaults`.

### Task 2: Ballistics and loudness (`fp-engine/src/meter.rs`)

- A pure `MeterState::update(input: MeterInput, dt_secs, &MeterConfig) -> MeterReading`.
  - `MeterInput` holds the peaks, the sums of squares, the K-weighted sums and the frames.
  - `MeterReading` holds the levels and holds in dBFS, and the momentary and short-term LUFS.
- `LoudnessWindow` holds 100 ms bins in a fixed ring.
- **Tests:**
  - `digital_peak_rises_at_once_and_falls_20_db_in_1_7_s`;
  - `ebu_ppm_falls_24_db_in_2_8_s`;
  - `a_short_burst_reads_lower_on_a_ppm`;
  - `vu_reaches_99_percent_in_300_ms`;
  - `the_hold_stays_then_falls`;
  - `custom_ballistics_use_the_configured_rates`;
  - `a_minus_23_dbfs_1_khz_stereo_sine_reads_minus_23_lufs` (EBU Tech 3341 case 1, momentary and short-term, ±0.1 LU; energies from the real K-weighting of Task 3);
  - `silence_clears_the_loudness`.

### Task 3: Measurement in the mixer

- **`fp-engine/src/kweight.rs`:**
  - biquad coefficients for the BS.1770 pre-filter and RLB high-pass at any rate;
  - `KWeighting` state per channel.
- **`fp-engine/src/truepeak.rs`:**
  - a 4× windowed-sinc interpolator (fixed taps, a state ring per channel);
  - `TruePeak::push(sample) -> f32`, the maximum of the 4 phases.
- **Slot:** it gets the filter states, and at attach the coefficients for the bus rate (`BusShared.sample_rate`, set by the bus on every open).
- **`SourceShared`:** it gets `sum_sq_l`/`sum_sq_r`, `k_sum_l`/`k_sum_r` (AtomicF64 accumulators) and `measured_frames`.
- **`BusShared.true_peak`:** an `AtomicBool`.
- **Tests:**
  - `k_weighting_at_48_khz_matches_the_standard_coefficients` (the BS.1770 table);
  - `k_weighting_gain_at_1_khz_is_the_same_at_every_rate` (44.1, 48 and 96 kHz, ±0.05 dB);
  - `true_peak_finds_the_intersample_peak` (a sine at fs/4 with a 45° phase: sample peak 0.707, true peak ≈ 1.0);
  - `mixer_accumulates_energy_without_allocating`;
  - the existing bit-exact test stays green.

### Task 4: Conductor wiring and telemetry

- `Engine::take_meter_input(player) -> MeterInput` sums the player's current and outgoing sources.
- The conductor keeps `MeterState` per player, updates it every tick with `dt` from `now` and `state.config.meter`, publishes `PlayerTelemetry.meter: MeterReading`, and sets each bus's true-peak flag when the configuration changes.
- **Tests (conductor with Offline):**
  - `the_meter_follows_a_playing_source_and_falls_after_stop`;
  - `true_peak_can_be_switched_while_playing`.

### Task 5: Interface

- **Meter widget:** the zones, the reference tick and the hold per M4, and the loudness line.
- **Settings → Meters** (a new section): the preset (with the standard named), custom attack and fall (shown for `Custom`), true peak, floor, hold, reference, warning, danger, the loudness readout and the target.
- Strings in `en-US` and `es-ES`.
- **Tests (kittest):**
  - `the_meter_draws_the_telemetry_levels` (through a test hook that reads the lit segments);
  - `choosing_a_vu_meter_updates_the_config`;
  - `the_loudness_line_shows_lufs`.

### Task 6: Documentation and verification

- **User guide:** `players.md` (the meter) and `settings.md` (Meters).
- **Technical docs:** `audio-engine.md` (measurement and metering) and `persistence.md` (`meter` fields).
- Full checks, a fresh review, fixes, then merge.
