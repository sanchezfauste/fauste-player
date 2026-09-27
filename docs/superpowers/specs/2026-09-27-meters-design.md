# Level meters (design)

**Parent spec:** [`2026-09-25-fauste-player-design.md`](2026-09-25-fauste-player-design.md) §8 (player column meter).

**Problem:** the Phase 1 meter jumps and flickers, for three reasons:
- the engine resets each source's peak every conductor tick (5 ms), while the interface reads one snapshot per frame (about 16 ms), so it shows a random 5 ms slice;
- the bar has no ballistics;
- the scale follows no standard.

**Goal:** a player meter that behaves like broadcast meters:
- standard ballistics, chosen in Settings;
- sample or true peak;
- scale marks and colour zones at configurable levels;
- an optional EBU R128 loudness readout.

## M0. Sources and what is verified against them

| Part | Document | Verified by tests |
|---|---|---|
| K-weighting | ITU-R BS.1770-5, tables 1 and 2 | coefficients at 48 kHz; the same 1 kHz gain at 44.1, 96 and 192 kHz |
| True peak | ITU-R BS.1770-5, Annex 2 (the published order-48, 4-phase filter) | the coefficients; EBU Tech 3341 cases 15–23 (+0.2 / −0.4 dB) |
| Loudness (M, S) | ITU-R BS.1770-5; EBU Tech 3341 (2023) table 1 | cases 1, 2, 9, 11, 12, 14 (±0.1 LU) |
| EBU PPM | EBU Tech 3205-E: table 2 (burst response), 3.3 (integration time), 3.10 (return time) | every burst of table 2 within tolerance; 24 dB fall in 2.8 s |
| DIN PPM | IEC 60268-10 type I (not available: paid). The integration-time definition quoted in Tech 3205 is used | a 5 ms burst reads 2 dB low; 20 dB fall in 1.5 s |
| VU | IEC 60268-17 (not available). Its publicly documented figures are used: average response, 99 % in 300 ms ± 10 %, 1–1.5 % overshoot | all three |
| Digital peak | IEC 60268-18 (not available): instant rise, 20 dB fall in 1.7 s | rise and fall |

## M1. Measurement (engine, real-time thread)

- **Where:** each mixer slot measures what it outputs, after gain.
  - **Sample peak** per channel: the existing `peak_l`/`peak_r`.
  - **True peak** (when enabled): the largest magnitude of the four phases of the ITU-R BS.1770-5 Annex 2 interpolator. It replaces the sample peak in the same atomics.
  - **Programme-meter integration** (EBU, DIN, custom): a two-stage rectifier integrator per sample. Each stage charges upwards only and falls at the meter's fall rate. The EBU time constants (2.25 and 0.95 ms) are fitted to Tech 3205 table 2; DIN and custom scale them to their integration time by the IEC definition.
  - **Rectified sum** per channel (Σ|x|), for the VU; sums of squares are kept too.
  - **K-weighted mean square** per channel, for loudness (ITU-R BS.1770): the pre-filter (high shelf) and the RLB high-pass, with coefficients derived for the bus rate from the standard's analogue prototypes.
- **Storage:** sums of squares, rectified sums, K-weighted sums and the frame count accumulate in `SourceShared` atomics. The conductor takes them every tick, so nothing is lost between ticks.
- **Real-time rules:** filter states live in the slot, coefficients are computed at attach, and nothing is allocated. The true-peak switch is an atomic flag in `BusShared`, set from the configuration.

## M2. Ballistics and loudness (conductor thread)

- **`MeterState` per player** runs every conductor tick (5 ms) on the peaks and energies taken from the player's sources (current and outgoing), with the live `config.meter`.
- **Ballistics presets** (`MeterBallistics`):

  | Preset | Standard | Input | Rise | Fall |
  |---|---|---|---|---|
  | `DigitalPeak` (default) | IEC 60268-18 | peak | instant | 20 dB in 1.7 s |
  | `EbuPpm` | IEC 60268-10 type IIb | peak | integration 10 ± 2 ms (the fit: 8.37 ms by the IEC definition) | 24 dB in 2.8 s |
  | `DinPpm` | IEC 60268-10 type I | peak | integration 5 ms | 20 dB in 1.5 s |
  | `Vu` | IEC 60268-17 | rectified average | second order: 99 % in 300 ms, 1.25 % overshoot | the same movement |
  | `Custom` | — | peak | `attack_ms` | `release_db_per_sec` |

  Programme-meter integration runs per sample in the mixer (M1). The conductor applies the fall. The VU needle is a second-order system (ζ = 0.8127, ω₀ = 13.51 rad/s) driven by the rectified average, calibrated so that a sine reads its peak level.
- **Ticks without a block:** a device block can span several 5 ms ticks. A tick with no audio less than 50 ms after the last block leaves the level standing, and the next block moves it for the whole span. Longer without audio counts as silence: the bar falls, and loudness counts the elapsed time as silent frames, so it falls instead of freezing.
- **Peak hold:** the highest level is kept for `peak_hold_secs`, then falls at the ballistics' fall rate. 0 disables it.
- **Loudness (EBU R128 / ITU-R BS.1770):**
  - energies are binned in 5 ms blocks (the tick), so the windows slide finely enough for Tech 3341's live-meter cases;
  - momentary loudness covers the last 400 ms, short-term the last 3 s;
  - L = −0.691 + 10·log10(z_L + z_R) LUFS, with no gating (M and S are ungated);
  - stereo only (the sources are stereo);
  - with no source playing, the readout clears.

## M3. Configuration (`config.meter`, lenient, validated)

| Field | Default | Range | Meaning |
|---|---|---|---|
| `ballistics` | `DigitalPeak` | preset | M2 |
| `attack_ms` | 5.0 | 0 … 1000 | `Custom` rise |
| `release_db_per_sec` | 11.8 | 1 … 100 | `Custom` fall |
| `true_peak` | false | — | M1 |
| `floor_db` | −60 | −96 … −20 | bottom of the scale |
| `peak_hold_secs` | 2.0 | 0 … 10 | M2 |
| `reference_dbfs` | −18 | −30 … 0 | alignment mark (EBU R68) |
| `warning_dbfs` | −9 | −30 … 0 | yellow from here (EBU permitted maximum) |
| `danger_dbfs` | −3 | −30 … 0 | red from here |
| `loudness` | `ShortTerm` | `Off`, `Momentary`, `ShortTerm` | readout under the meter |
| `loudness_target_lufs` | −23 | −36 … −10 | readout green within ±1 LU |

`Config::validate` keeps the zones ordered: floor < reference, and warning ≤ danger. Changes apply at once; `true_peak` reaches running buses through their flag.

## M4. Display

- The meter keeps its size in the player column.
- Segments are drawn from the floor to 0 dBFS in dB-linear steps, green, yellow from `warning_dbfs`, red from `danger_dbfs`.
- A tick marks `reference_dbfs`. The hold segment is drawn in the colour of its zone.
- With `loudness` on, one line under the meter shows the value in LUFS, coloured against the target.

## M5. Out of scope

- Integrated (gated) programme loudness and LRA.
- A master or bus meter.
- Metering of the cartwall.
