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
| K-System | B. Katz, "An integrated approach to metering, monitoring and levelling" (AES), as specified in "Level Practices (Part 2)" (digido.com) | the average: a sine reads its level, 99 % of a step in 600 ms and the same fall; the peak falls 26 dB in 3 s |
| Scales | IEC 60268-10 (types I and IIb), 60268-17 and 60268-18 scale marks as published (for example in ITU-T J.15 and the standards' public summaries); the K-System's scale as above | the position of every mark, per scale |

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
  | `K20`, `K14`, `K12` | K-System | peak and RMS average | peak: one sample; average: 99 % in 600 ms | peak: 26 dB in 3 s; average: as it rises |
  | `Custom` | — | peak | `attack_ms` | `release_db_per_sec` |

  Programme-meter integration runs per sample in the mixer (M1). The conductor applies the fall. The VU needle is a second-order system (ζ = 0.8127, ω₀ = 13.51 rad/s) driven by the rectified average, calibrated so that a sine reads its peak level.
- **Ticks without a block:** a device block can span several 5 ms ticks. A tick with no audio less than 50 ms after the last block leaves the level standing, and the next block moves it for the whole span. Longer without audio counts as silence: the bar falls, and loudness counts the elapsed time as silent frames, so it falls instead of freezing.
- **Peak hold:** the highest level is kept for `peak_hold_secs`, then falls at the ballistics' fall rate. 0 disables it.
- **Average (RMS) level, for the K-System:** per channel, the mean square of the tick's samples goes through two equal first-order stages (τ = 600 ms / 5.84 = 102.7 ms, solved exactly per tick), so a step reads 99 % of its RMS value (−0.09 dB) after 600 ms and falls the same way, the K-System's 600 ms integration and fall. Ticks without audio count as silence, as for the bar. It is read by the AES17 convention (a full-scale sine reads 0 dBFS), so "peak and average sections are calibrated with sine wave to ride on the same numeric scale", as the K-System requires.
- **Maximum:** the highest level the bar reached (either channel), shown as a number. Only what each tick measured counts, on the meter's own terms (the VU's calibrated average, the peak for the others), so after a restart the bar still falling from the previous entry is ignored. It restarts when a different entry becomes current (a segue's overlap counts for the new entry: it is what the player puts on air), when an entry starts again (after a stop, even within one tick, or segued into itself), and when the operator clicks the readout (an `EngineRequest`, since the meter is not model state). Non-finite samples read as silence and never stick in the meter.
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
| `floor_db` | −60 | −96 … −20 | bottom of the digital scale (digital peak and custom); the other scales have the range their standard gives |
| `peak_hold_secs` | 2.0 | 0 … 10 | M2 |
| `reference_dbfs` | −18 | −30 … 0 | alignment level (EBU R68): the mark, EBU TEST, DIN −9 and 0 VU |
| `warning_dbfs` | −9 | −30 … 0 | yellow from here (EBU permitted maximum), on the digital scale |
| `danger_dbfs` | −3 | −30 … 0 | red from here, on the digital scale |
| `loudness` | `ShortTerm` | `Off`, `Momentary`, `ShortTerm` | readout under the meter |
| `loudness_target_lufs` | −23 | −36 … −10 | readout green within ±1 LU |

**Used by** (`MeterBallistics::settings`): Settings shows, and the meter applies, only the fields its standard leaves open; the others keep their value for when the type is chosen again.

| Type | `attack_ms`, `release_db_per_sec` | `floor_db` | `peak_hold_secs` | `reference_dbfs` | `warning_dbfs`, `danger_dbfs` | `true_peak` |
|---|---|---|---|---|---|---|
| `DigitalPeak` | – | ✓ | ✓ (IEC 60268-18 allows a hold) | ✓ | ✓ | ✓ |
| `Custom` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `EbuPpm`, `DinPpm` | – | – (fixed scale) | – (no hold in IEC 60268-10) | ✓ (TEST, −9) | – (red from the permitted maximum) | – (quasi-peak of the sampled signal) |
| `Vu` | – | – | – (a needle) | ✓ (0 VU) | – (red from 0 VU) | – (reads the average) |
| `K20`, `K14`, `K12` | – | – | ✓ (peak section) | – (its own 0) | – | ✓ (peak section) |

`Config::validate` keeps the zones ordered: floor < reference, and warning ≤ danger. Changes apply at once; `true_peak` reaches running buses through their flag.

## M4. Display

- The meter and the volume fader form a column at the right of the player, spanning the info and transport rows (feedback spec §3.2): the scale's labels, then the L and R bars side by side, then the fader.
- **One continuous bar per channel**, drawn to the pixel, not in segments (segments of 3 dB made the bar move in visible steps).
- **Scale: the one of the chosen meter's standard**, labelled on rulers on both sides of the bars in the meter's own units (monospace, `NEUTRAL_400`): a tick for every label and minor ticks between them at a spacing per scale. Nothing is drawn over the bars or in the gap between them (operator feedback 4, Q11). Levels above the top of a scale sit at the top.

  | Meter | Scale (marks) | Where the alignment level is | Law |
  |---|---|---|---|
  | Digital peak, custom | −60, −50, −40, −35, −30, −25, −20, −15, −10, −5, 0 dBFS (IEC 60268-18), from `floor_db` | the mark | the deflection commonly used for this scale, in % of 0 dBFS: 2.5 %/dB from −20 to 0, 2 from −30, 1.5 from −40, 0.75 from −50, 0.5 from −60, 0.25 from −70 and below it (so −20 is halfway); the bottom is `floor_db` |
  | EBU PPM | −12, −8, −4, TEST, +4, +8, +12 dB (IEC 60268-10 IIb) | TEST | linear in dB from −12 to +12; below −12 the bar rests at the bottom, like the needle |
  | DIN PPM | −50, −40, −30, −20, −10, −5, 0, +5 dB (IEC 60268-10 type I); 0 dB is the permitted maximum, 9 dB above alignment | −9 | semi-logarithmic: height ∝ the fourth root of the voltage, from −50 (bottom) to +5 (top) |
  | VU | −20, −10, −7, −5, −3, −2, −1, 0, +1, +2, +3 VU (IEC 60268-17) | 0 VU | proportional to the voltage, +3 VU at the top (0 VU at 70.8 %) |
  | K-20, K-14, K-12 | +N (0 dBFS, the top), +4, 0, −4 … −24 every 4 dB, −30, −40, −50, −60 | 0 (−N dBFS) | linear in dB from the top to −24 over 80 % of the height; −24 to −60 over the rest. The K-System asks for 1 dB marks down to −24: at this meter's size, every 4 dB |

- **Zones:** muted traffic-light colours (normal #7fb08a, warning #d9b45a, danger #d8646a). On the digital scale (digital peak, custom), green, yellow from `warning_dbfs`, red from `danger_dbfs`. The other scales use their own red region, which the configured levels could miss (a VU scale ends at −15 dBFS): VU red from 0 VU (its red arc); EBU and DIN red from the permitted maximum, 9 dB above alignment (EBU +9, DIN 0). The K-System: green below 0, amber from 0 to +4, red above.
- **Alignment level:** a thicker white tick on both rulers (operator feedback 4, Q11) at `reference_dbfs` (the K-System's 0 for K meters), whether or not it is one of the scale's marks. A bright line across the bars read as a fault in the signal. It is labelled only where the scale names it (EBU TEST, K 0, 0 VU); the digital meter's −18 dBFS keeps the round labels around it.
- **Legibility:** nothing is drawn over the bars. Both ends of the scale are labelled first; then labels are kept top down while they are at least 10 px from every kept label, so they never overlap at the available height. A mark without a label gets a minor tick.
- **K-System bars show both sections:** the average (RMS) level is the solid body; the part up to the peak level is the same colour, dimmed. The other meters show one level, as their standard defines.
- **Peak hold:** a two-pixel line at the hold level, in the colour of its zone.
- **Maximum readout** above the bars: the maximum (M2) in dBFS with one decimal, red in the danger zone, a dash before any audio. A click on it restarts it.
- With `loudness` on, one line under the meter shows the value in LUFS, coloured against the target.

## M5. Out of scope

- Integrated (gated) programme loudness and LRA.
- A master or bus meter.
- Metering of the cartwall.
