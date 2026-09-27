# Meters · Plan 2 — Meters faithful to the standards

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans (or superpowers:subagent-driven-development) to implement this plan task by task.

**Goal:** replace the approximations of plan 1 with behaviour verified against the published documents:
- the ITU-R BS.1770-5 true-peak filter;
- programme meters meeting EBU Tech 3205-E;
- a VU with the IEC 60268-17 needle and average response;
- loudness meeting EBU Tech 3341's live-meter cases.

**Spec:** [`docs/superpowers/specs/2026-09-27-meters-design.md`](../specs/2026-09-27-meters-design.md), M0–M2 (updated).

**Sources:**
- ITU-R BS.1770-5 (2023), Annex 2;
- EBU Tech 3341 (2023), table 1;
- EBU Tech 3205-E, sections 3.1, 3.3 and 3.10.

The IEC 60268-10, -17 and -18 texts are not available (paid). Where they are needed, the definition quoted by the EBU or the publicly documented figures are used, and the spec says so.

## Global Constraints

As plan 1: real-time rules in the mixer, `forbid(unsafe_code)`, no hardcoded operator limits, TDD, docs in sync.

## Review Focus

1. **Every test value comes from a document**, with the document's tolerance, not a tolerance chosen to pass.
2. **The mixer stays real-time safe** with the two-stage integrator and the new true-peak filter (the no-allocation test).
3. **Presets switched mid-playback** reach every bus (integrator constants, true peak).
4. **The VU needle stays stable** for any tick span (explicit integration steps).
5. **The loudness window** (600 blocks) is allocated on the conductor thread only.

---

### Task 1: True peak — the published filter

- `truepeak::PHASES`, as published in Annex 2.
- The peak is the largest of the four phases.
- **Tests:**
  - `the_true_peak_filter_is_the_one_bs_1770_publishes`;
  - Tech 3341 cases 15–19 and 20–23 (the synthesized, low-passed and decimated signal).

### Task 2: Programme meters — Tech 3205

- A two-stage rectifier integrator in the mixer, with `BusShared::ppm_tau1_ms`/`ppm_tau2_ms`.
- EBU constants fitted to table 2; DIN and custom scaled by the integration-time definition.
- **Tests:**
  - `the_ebu_ppm_meets_tech_3205_table_2`;
  - `the_din_ppm_has_a_5_ms_integration_time`;
  - `a_digital_peak_meter_shows_even_the_shortest_burst`.

### Task 3: VU — IEC 60268-17

- The rectified sums from the mixer.
- A second-order needle (ζ from a 1.25 % overshoot, 99 % in 300 ms), calibrated so a sine reads its peak.
- **Tests:**
  - `vu_reaches_99_percent_in_300_ms`;
  - `vu_overshoots_between_1_and_1_5_percent`;
  - `vu_reads_the_rectified_average_like_a_real_vu`.

### Task 4: Loudness — Tech 3341 live cases

- 5 ms blocks: 80 for momentary, 600 for short-term.
- **Tests:** cases 1, 2, 9, 11, 12 and 14.

### Task 5: Docs and verification

- The spec's M0 table of sources, and the user and technical docs.
- Full checks, a fresh review, fixes, then merge.
