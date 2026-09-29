# Meters plan 3: standard scales, the K-System and a maximum readout

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** each meter type is drawn on its own standard's scale, the K-System (K-20, K-14, K-12) is added, and a maximum readout appears above the bars. The bars are continuous instead of segmented.

**Architecture:**
- **fp-engine:** `MeterState` (conductor thread) adds the K-System ballistics, the K-System's two-stage RMS average and the running maximum. The maximum restarts on a new entry and on an `EngineRequest`.
- **fp-app:** the widget maps levels through the scale of the chosen standard (`meter_position`, `scale_marks`) and paints rectangles.

**Tech Stack:** Rust, egui, egui_kittest.

**Spec:** `docs/superpowers/specs/2026-09-27-meters-design.md` (M0 sources, M2 K-System and "Maximum", M4 scales).

## Global Constraints

- Ballistics, scales and zones are the standards' own values; the spec cites the source of each.
- UI strings are in both `en-US` and `es-ES`.
- No `unwrap`, `expect` or indexing in fp-engine.
- Never crash on bad data: non-finite samples read as silence.

## Review Focus

- **A floor above −40 or −20 dBFS on the digital scale:** the scale stays monotonic and reaches 0 at the floor. Pinned by `every_scale_is_monotonic_and_ends_at_its_top`.
- **Levels above a scale's top (EBU above +12, VU above +3, overs):** they sit at the top. Same test.
- **A loud entry followed by a quiet one:** the new maximum ignores the bar still falling. Pinned by `a_restarted_maximum_ignores_the_bar_still_falling_from_before`.
- **NaN or infinite samples:** the meter recovers. Pinned by `a_non_finite_block_does_not_break_the_meter`.
- **Stop, then the next entry:** the maximum survives the stop and restarts with the next entry. Pinned by `the_meter_maximum_outlasts_a_stop_and_restarts_with_the_next_entry`.

---

### Task 1: K-System presets, ballistics, average and maximum (model and engine)

- [ ] **fp-model tests:** `MeterBallistics::K20`, `K14` and `K12` load from JSON; `k_reference_dbfs` gives −20, −14 and −12.
- [ ] **fp-engine tests** (`tests/metering.rs`):
  - a sine reads its level on the average;
  - 99 % of the RMS value at 600 ms, and the same fall;
  - the peak falls 26 dB in 3 s;
  - the maximum holds until it is reset, and ignores the previous entry's falling bar;
  - non-finite blocks.
- [ ] **fp-engine tests** (`tests/conductor.rs`): the maximum outlasts a stop, restarts with the next entry, and restarts on request.
- [ ] Implement, then commit `feat(engine): K-System meters and a maximum level per player`.

### Task 2: the display

- [ ] **Tests** (`tests/meter_view.rs`):
  - each scale's positions and marks;
  - every scale is monotonic and clips at its top;
  - the configured zones and the K-System's own zones;
  - the readout format;
  - a click on the readout restarts the maximum;
  - the K presets appear in Settings → Meters.
- [ ] Implement `meter_position`, `scale_marks`, `zone_of`, `max_readout`, the continuous bars and `Controller::reset_meter_max`. Commit `feat(ui): meters on their standard's scale, with a maximum readout`.

### Task 3: docs

- [ ] Update README, `docs/user/players.md`, `docs/user/settings.md`, `docs/technical/audio-engine.md`, `docs/technical/persistence.md` and the spec. Commit `docs: describe the standard meter scales and the K-System`.
