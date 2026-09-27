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

## M1. Measurement (engine, real-time thread)

- **Where:** each mixer slot measures what it outputs, after gain.
  - **Sample peak** per channel: the existing `peak_l`/`peak_r`.
  - **True peak** (when enabled): peak of the signal 4× oversampled with a windowed-sinc interpolator, as ITU-R BS.1770 recommends for true-peak meters. It replaces the sample peak in the same atomics.
  - **Mean square** per channel, unweighted, for VU ballistics.
  - **K-weighted mean square** per channel, for loudness (ITU-R BS.1770): the pre-filter (high shelf) and the RLB high-pass, with coefficients derived for the bus rate from the standard's analogue prototypes.
- **Storage:** sums of squares and the frame count accumulate in `SourceShared` atomics. The conductor takes them every tick, so nothing is lost between ticks.
- **Real-time rules:** filter states live in the slot, coefficients are computed at attach, and nothing is allocated. The true-peak switch is an atomic flag in `BusShared`, set from the configuration.

## M2. Ballistics and loudness (conductor thread)

- **`MeterState` per player** runs every conductor tick (5 ms) on the peaks and energies taken from the player's sources (current and outgoing), with the live `config.meter`.
- **Ballistics presets** (`MeterBallistics`):

  | Preset | Standard | Input | Rise | Fall |
  |---|---|---|---|---|
  | `DigitalPeak` (default) | IEC 60268-18 | peak | instant | 20 dB in 1.7 s |
  | `EbuPpm` | IEC 60268-10 type IIb | peak | integration 10 ms | 24 dB in 2.8 s |
  | `DinPpm` | IEC 60268-10 type I | peak | integration 5 ms | 20 dB in 1.5 s |
  | `Vu` | IEC 60268-17 | RMS | 300 ms to 99 % | 300 ms (symmetric) |
  | `Custom` | — | peak | `attack_ms` | `release_db_per_sec` |

  Programme-meter integration (EBU, DIN and custom rise) runs per sample in the mixer (`BusShared::integration_ms`), as a first-order rise on |x| with a time constant `integration / 3`. A tone burst of the integration time reads within about 0.5 dB of steady state, and shorter peaks read lower (a 0.5 ms click reads about 17 dB low on the EBU meter). The standards' exact burst figures are not claimed.
- **Ticks without a block:** a device block can span several 5 ms ticks. A tick with no audio less than 50 ms after the last block leaves the level standing, and the next block moves it for the whole span. Longer without audio counts as silence: the bar falls, and loudness counts the elapsed time as silent frames, so it falls instead of freezing.
- **Peak hold:** the highest level is kept for `peak_hold_secs`, then falls at the ballistics' fall rate. 0 disables it.
- **Loudness (EBU R128 / ITU-R BS.1770):**
  - energies are binned in 100 ms blocks;
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
