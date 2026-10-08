# Loudness Normalization, Plan 1: the `fp-dsp` Crate

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A new workspace crate `fp-dsp` holds the K-weighting and the true-peak interpolator (moved from `fp-engine`) and a new BS.1770 gated integrated-loudness meter (`LoudnessMeter`); the level meters keep working unchanged on top of it.

**Architecture:** `crates/fp-engine/src/kweight.rs` and `truepeak.rs` move verbatim (`git mv`) into `crates/fp-dsp/src/`; `fp-engine` depends on `fp-dsp` and its mixer imports from there. A new `crates/fp-dsp/src/loudness.rs` adds `LoudnessMeter`: per channel K-weighting plus true peak, energy per 100 ms hop, 400 ms gating blocks with 75 % overlap, absolute (−70 LUFS) and relative (−10 LU) gates. Nothing uses `LoudnessMeter` yet (plan 2 wires it into the analysis). The DSP tests of `fp-engine/tests/metering.rs` move to `fp-dsp/tests/filters.rs`; the meter-ballistics tests stay and import from `fp_dsp`.

**Tech Stack:** Rust 2024, no dependencies (pure signal processing), `hound` not needed (tests synthesise signals in memory).

**Spec:** `docs/superpowers/specs/2026-10-08-loudness-normalization-design.md` (read §1–§3, §5 N4–N6 and the `fp-dsp` paragraph of N6, §10 row `fp-dsp`, §11, §14 Q5, Q7).

## Global Constraints

- All code, identifiers, comments, docs and commit messages in English; never mention other playout or radio-automation products.
- `fp-dsp`: "pure signal processing, no I/O, no dependency on other workspace crates, `unsafe` forbidden and `clippy::indexing_slicing` denied like `fp-engine`" (spec N6).
- `KWeighting::process` and `TruePeak::push` stay real-time safe (no allocation, no lock, no panic): the mixer calls them on the device callback (CLAUDE.md rule 5). `LoudnessMeter` "allocates only off the device callback, in the analysis pool" (N6): it may grow a `Vec`, it is never used by the mixer.
- `unwrap`, `expect`, `panic` denied outside tests; use `get`, iterators and `windows`, never `[]` indexing in `src/`.
- Integrated loudness: "ITU-R BS.1770-5 programme loudness of the whole file, in LUFS: K-weighted mean square over 400 ms blocks with 75 % overlap, gated at −70 LUFS (absolute) and −10 LU below the ungated mean (relative)" (§3). Both channels weighted 1.0 (N4).
- "A file shorter than one gating block (400 ms) … is not measured" (N5, Q7): `finish` returns `None`.
- Silence: "`integrated_lufs` is `None` when no block passes the absolute gate (a silent file)" (§3).
- "Non-finite input does not panic" (§10 `fp-dsp` row).
- Conventional Commits; every commit ends with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Gate for every task: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`.
- Progress ledger: `.superpowers/sdd/2026-10-08-loudness-plan1-fp-dsp/progress.md`; record deviations as `Ruling: <decision> — <why> — <cost if wrong>`.

## Review Focus

1. **A rate whose 100 ms hop is not a whole number of frames** (44 100 Hz → 4 410 frames is whole, but 22 050 Hz, 11 025 Hz and 8 000 Hz must also work): the hop is `round(rate / 10)`, at least 1 frame, and a −23 dBFS sine still reads −23.0 LUFS. Test: Task 2, `a_sine_reads_its_level_at_every_rate`.
2. **Digital silence followed by a short tone shorter than the relative gate's influence** (a jingle with a long silent tail): the silent blocks never pass the absolute gate, so they do not pull the integrated value down. Test: Task 2, `silent_parts_do_not_lower_the_integrated_loudness` (Tech 3341 case 4).
3. **NaN or infinite samples in the input** (a damaged float file reaching the meter directly): they count as silence and the result stays finite. Test: Task 2, `non_finite_samples_count_as_silence`.
4. **An odd-length slice** (a block that ends in the middle of a frame): the trailing half frame is ignored, no panic. Test: Task 2, `a_trailing_half_frame_is_ignored`.
5. **The true peak of complete silence**: `20·log10(0)` is −∞; the meter reports the documented floor (`SILENCE_DBTP`, −200 dBTP) so the value is always finite. Test: Task 2, `silence_has_no_integrated_loudness_and_a_finite_peak`.

---

## File Structure

| File | Responsibility |
|---|---|
| `Cargo.toml` (workspace) | `fp-dsp` in `[workspace.dependencies]` (members are `crates/*`, so it is a member already) |
| `crates/fp-dsp/Cargo.toml` (new) | The crate manifest: no dependencies, workspace lints |
| `crates/fp-dsp/src/lib.rs` (new) | Crate doc, `#![deny(clippy::indexing_slicing)]`, `pub mod kweight; pub mod loudness; pub mod truepeak;` |
| `crates/fp-dsp/src/kweight.rs` (moved from `crates/fp-engine/src/kweight.rs`) | K-weighting biquads, unchanged |
| `crates/fp-dsp/src/truepeak.rs` (moved from `crates/fp-engine/src/truepeak.rs`) | True-peak interpolator, unchanged |
| `crates/fp-dsp/src/loudness.rs` (new) | `LoudnessMeter`, `Loudness`, `SILENCE_DBTP` |
| `crates/fp-dsp/tests/filters.rs` (new) | The K-weighting and true-peak tests moved from `fp-engine/tests/metering.rs` |
| `crates/fp-dsp/tests/loudness.rs` (new) | `LoudnessMeter` tests (EBU Tech 3341 cases, N5) |
| `crates/fp-engine/Cargo.toml` | Depends on `fp-dsp` |
| `crates/fp-engine/src/lib.rs` | Drops `pub mod kweight;` and `pub mod truepeak;` |
| `crates/fp-engine/src/mixer.rs:14,17` | `use fp_dsp::kweight::KWeighting;`, `use fp_dsp::truepeak::TruePeak;` |
| `crates/fp-engine/tests/metering.rs` | Keeps the meter-ballistics tests; imports from `fp_dsp` |
| `CLAUDE.md`, `README.md`, `docs/technical/*`, meters spec | The new crate in every crate list (Task 3) |

---

### Task 1: Create `fp-dsp` and move the K-weighting and the true peak

**Files:**
- Create: `crates/fp-dsp/Cargo.toml`, `crates/fp-dsp/src/lib.rs`, `crates/fp-dsp/tests/filters.rs`
- Move: `crates/fp-engine/src/kweight.rs` → `crates/fp-dsp/src/kweight.rs`; `crates/fp-engine/src/truepeak.rs` → `crates/fp-dsp/src/truepeak.rs`
- Modify: `Cargo.toml` (`[workspace.dependencies]`), `crates/fp-engine/Cargo.toml`, `crates/fp-engine/src/lib.rs:12,19`, `crates/fp-engine/src/mixer.rs:14,17`, `crates/fp-engine/tests/metering.rs`

**Interfaces:**
- Produces: `fp_dsp::kweight::{Biquad, coefficients, KWeighting}` and `fp_dsp::truepeak::{PHASES, TruePeak}`, with exactly the signatures they have today in `fp-engine` (`KWeighting::new(rate: u32) -> Self`, `KWeighting::process(&mut self, x: f64) -> f64`, `TruePeak::push(&mut self, x: f32) -> f32`, `coefficients(rate: u32) -> (Biquad, Biquad)`, `pub const PHASES: [[f32; 12]; 4]`).
- `fp_engine::kweight` and `fp_engine::truepeak` no longer exist.

- [ ] **Step 1: Move the DSP tests first (they will fail to compile: the crate does not exist yet)**

Create `crates/fp-dsp/tests/filters.rs` with the header below, then **move** (cut from `crates/fp-engine/tests/metering.rs`, paste here, unchanged) these items: `k_weighting_at_48_khz_matches_the_standard_coefficients`, `gain_db`, `k_weighting_gain_at_1_khz_is_the_same_at_every_rate`, `dbtp`, `tp_sine`, `within`, `true_peak_meets_ebu_tech_3341_cases_15_to_19`, `tp_burst`, `true_peak_meets_ebu_tech_3341_cases_20_to_23`, `silence_brings_the_k_weighting_to_exact_zero`, `the_true_peak_filter_is_the_one_bs_1770_publishes` (today lines 13–163 and 444–475 of `metering.rs`). In the moved `the_true_peak_filter_is_the_one_bs_1770_publishes`, replace `fp_engine::truepeak::PHASES` with `fp_dsp::truepeak::PHASES`.

```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The shared signal processing (loudness normalization spec N6): the
//! K-weighting and the true-peak interpolator of ITU-R BS.1770, which the
//! level meters (meters spec M1) and the analysis use.

use fp_dsp::kweight::{KWeighting, coefficients};
use fp_dsp::truepeak::TruePeak;
```

In `crates/fp-engine/tests/metering.rs`, replace the two imports at lines 10–11 with:

```rust
use fp_dsp::kweight::KWeighting;
```

(`programme` and `sine_ticks` still use `KWeighting`; `TruePeak` and `coefficients` are no longer used there.) Update the module doc of `metering.rs` to "Level meter ballistics and loudness readout (meters spec M1–M3); the filters themselves are tested in `fp-dsp`."

- [ ] **Step 2: Run to verify it fails**

Run: `export PATH="$HOME/.cargo/bin:$PATH"; cargo test -p fp-engine --test metering 2>&1 | tail -5`
Expected: FAIL to compile, "use of unresolved module or unlinked crate `fp_dsp`".

- [ ] **Step 3: Create the crate and move the modules**

```bash
mkdir -p crates/fp-dsp/src
git mv crates/fp-engine/src/kweight.rs crates/fp-dsp/src/kweight.rs
git mv crates/fp-engine/src/truepeak.rs crates/fp-dsp/src/truepeak.rs
```

`crates/fp-dsp/Cargo.toml`:

```toml
[package]
name = "fp-dsp"
description = "Signal processing shared by the engine and the analysis: K-weighting, true peak, BS.1770 loudness"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
publish.workspace = true

[dependencies]

[lints]
workspace = true
```

`crates/fp-dsp/src/lib.rs`:

```rust
//! Signal processing shared by the audio engine (the level meters, meters
//! spec M1) and the analysis (loudness normalization spec N4): the ITU-R
//! BS.1770 K-weighting, the true-peak interpolator and the gated
//! integrated-loudness meter. Pure computation: no I/O, no threads, no
//! dependency on other workspace crates.

#![deny(clippy::indexing_slicing)]

pub mod kweight;
pub mod truepeak;
```

Workspace `Cargo.toml`, `[workspace.dependencies]`, after `fp-model = …`:

```toml
fp-dsp = { path = "crates/fp-dsp" }
```

`crates/fp-engine/Cargo.toml`, `[dependencies]`, after `fp-model.workspace = true`:

```toml
fp-dsp.workspace = true
```

`crates/fp-engine/src/lib.rs`: delete `pub mod kweight;` and `pub mod truepeak;`.

`crates/fp-engine/src/mixer.rs`: replace `use crate::kweight::KWeighting;` with `use fp_dsp::kweight::KWeighting;` and `use crate::truepeak::TruePeak;` with `use fp_dsp::truepeak::TruePeak;` (keep `cargo fmt` import grouping).

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p fp-dsp --test filters && cargo test -p fp-engine --test metering && cargo test -p fp-engine --test audio_path_audit`
Expected: PASS (the same test count as before the move, split across the two files).

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add -A crates/fp-dsp crates/fp-engine Cargo.toml Cargo.lock
git commit -m "refactor(dsp): move the K-weighting and the true peak to a new fp-dsp crate

The analysis will measure loudness with the same filters the level meters
use (loudness normalization spec N6, Q5). A crate of its own keeps the
analysis from depending on the engine.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: `LoudnessMeter`: gated integrated loudness and true peak (N4, N5)

**Files:**
- Create: `crates/fp-dsp/src/loudness.rs`, `crates/fp-dsp/tests/loudness.rs`
- Modify: `crates/fp-dsp/src/lib.rs` (`pub mod loudness;`)

**Interfaces:**
- Consumes: `fp_dsp::kweight::KWeighting`, `fp_dsp::truepeak::TruePeak` (Task 1).
- Produces (plan 2 relies on these exact names):
  - `pub const SILENCE_DBTP: f64 = -200.0;`
  - `#[derive(Debug, Clone, Copy, PartialEq)] pub struct Loudness { pub integrated_lufs: Option<f64>, pub true_peak_dbtp: f64 }`
  - `pub struct LoudnessMeter` with `pub fn new(sample_rate: u32) -> Self`, `pub fn push(&mut self, stereo: &[f32])` (interleaved L, R) and `pub fn finish(&self) -> Option<Loudness>` (`None` when fewer than 400 ms were pushed, or when a result is not finite).

- [ ] **Step 1: Write the failing tests**

`crates/fp-dsp/tests/loudness.rs`:

```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Integrated loudness of a whole file (loudness normalization spec N4, N5),
//! against the EBU Tech 3341 cases that apply to a file measurement.

use fp_dsp::loudness::{Loudness, LoudnessMeter, SILENCE_DBTP};

/// Stereo 1 kHz sine programme: `(Some(dBFS peak), secs)` or `(None, secs)`
/// for silence, both channels identical, at `rate`.
fn programme(rate: u32, parts: &[(Option<f64>, f64)]) -> Vec<f32> {
    let mut out = Vec::new();
    let mut n = 0u64;
    for (level, secs) in parts {
        let amp = level.map_or(0.0, |db| 10f64.powf(db / 20.0));
        for _ in 0..(secs * f64::from(rate)).round() as usize {
            let x = (amp * (std::f64::consts::TAU * 1_000.0 * n as f64 / f64::from(rate)).sin())
                as f32;
            out.extend_from_slice(&[x, x]);
            n += 1;
        }
    }
    out
}

/// Measures `samples` pushed in blocks of `block` frames (as a decoder does).
fn measure(rate: u32, samples: &[f32], block: usize) -> Option<Loudness> {
    let mut m = LoudnessMeter::new(rate);
    for chunk in samples.chunks(block * 2) {
        m.push(chunk);
    }
    m.finish()
}

fn lufs(rate: u32, parts: &[(Option<f64>, f64)]) -> f64 {
    measure(rate, &programme(rate, parts), 4_096)
        .unwrap()
        .integrated_lufs
        .unwrap()
}

#[test]
fn ebu_tech_3341_case_1_reads_minus_23_lufs_at_48_and_44_1_khz() {
    for rate in [48_000, 44_100] {
        let v = lufs(rate, &[(Some(-23.0), 20.0)]);
        assert!((v + 23.0).abs() <= 0.1, "{rate} Hz: {v}");
    }
}

#[test]
fn ebu_tech_3341_case_2_reads_minus_33_lufs() {
    let v = lufs(48_000, &[(Some(-33.0), 20.0)]);
    assert!((v + 33.0).abs() <= 0.1, "{v}");
}

#[test]
fn the_relative_gate_drops_the_quiet_parts() {
    // Tech 3341 case 3: −36, −23, −36 dBFS for 10, 60, 10 s.
    let v = lufs(
        48_000,
        &[(Some(-36.0), 10.0), (Some(-23.0), 60.0), (Some(-36.0), 10.0)],
    );
    assert!((v + 23.0).abs() <= 0.1, "{v}");
}

#[test]
fn silent_parts_do_not_lower_the_integrated_loudness() {
    // Tech 3341 case 4: −72, −36, −23, −36, −72 dBFS for 10, 10, 60, 10, 10 s.
    let v = lufs(
        48_000,
        &[
            (Some(-72.0), 10.0),
            (Some(-36.0), 10.0),
            (Some(-23.0), 60.0),
            (Some(-36.0), 10.0),
            (Some(-72.0), 10.0),
        ],
    );
    assert!((v + 23.0).abs() <= 0.1, "{v}");
}

#[test]
fn loud_and_quiet_parts_average_by_energy() {
    // Tech 3341 case 5: −26, −20, −26 dBFS for 20, 20.1, 20 s.
    let v = lufs(
        48_000,
        &[(Some(-26.0), 20.0), (Some(-20.0), 20.1), (Some(-26.0), 20.0)],
    );
    assert!((v + 23.0).abs() <= 0.1, "{v}");
}

#[test]
fn a_sine_reads_its_level_at_every_rate() {
    for rate in [8_000, 11_025, 22_050, 32_000, 88_200, 96_000] {
        let v = lufs(rate, &[(Some(-23.0), 5.0)]);
        // 1 kHz is well inside every band; the K-weighting is derived per rate.
        assert!((v + 23.0).abs() <= 0.15, "{rate} Hz: {v}");
    }
}

#[test]
fn block_size_does_not_change_the_result() {
    let p = programme(48_000, &[(Some(-30.0), 3.0), (Some(-18.0), 3.0)]);
    let a = measure(48_000, &p, 1).unwrap();
    let b = measure(48_000, &p, 4_800).unwrap();
    let c = measure(48_000, &p, 100_000).unwrap();
    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn the_true_peak_is_measured_on_both_channels() {
    // Tech 3341 case 19: fs/4 at 1.41 FFS, 45°, reads +3.0 dBTP (+0.2/−0.4).
    let rate = 48_000u32;
    let mut s = Vec::new();
    for i in 0..rate as usize {
        let x = (1.41 * (std::f64::consts::TAU * i as f64 / 4.0 + 45f64.to_radians()).sin()) as f32;
        // Left silent, right carries the signal: the peak is per channel.
        s.extend_from_slice(&[0.0, x]);
    }
    let tp = measure(rate, &s, 1_024).unwrap().true_peak_dbtp;
    assert!(tp <= 3.2 && tp >= 2.6, "{tp}");
}

#[test]
fn silence_has_no_integrated_loudness_and_a_finite_peak() {
    let l = measure(48_000, &vec![0.0; 48_000 * 2 * 2], 1_024).unwrap();
    assert_eq!(l.integrated_lufs, None);
    assert_eq!(l.true_peak_dbtp, SILENCE_DBTP);
}

#[test]
fn near_silence_under_the_absolute_gate_has_no_integrated_loudness() {
    let l = measure(48_000, &programme(48_000, &[(Some(-75.0), 3.0)]), 1_024).unwrap();
    assert_eq!(l.integrated_lufs, None);
    assert!(l.true_peak_dbtp.is_finite() && l.true_peak_dbtp < -70.0);
}

#[test]
fn a_file_shorter_than_one_gating_block_is_not_measured() {
    // 200 ms (spec N5, Q7) and one frame short of 400 ms.
    assert_eq!(measure(48_000, &programme(48_000, &[(Some(-20.0), 0.2)]), 512), None);
    let almost = programme(48_000, &[(Some(-20.0), 0.4)]);
    assert_eq!(measure(48_000, &almost[..almost.len() - 2], 512), None);
    // Exactly 400 ms is one block: measured.
    assert!(measure(48_000, &almost, 512).is_some());
}

#[test]
fn non_finite_samples_count_as_silence() {
    let mut p = programme(48_000, &[(Some(-23.0), 5.0)]);
    for s in p.iter_mut().step_by(97) {
        *s = f32::NAN;
    }
    p[1_000] = f32::INFINITY;
    p[1_001] = f32::NEG_INFINITY;
    let l = measure(48_000, &p, 1_024).unwrap();
    assert!(l.integrated_lufs.unwrap().is_finite());
    assert!(l.true_peak_dbtp.is_finite());
}

#[test]
fn a_trailing_half_frame_is_ignored() {
    let mut m = LoudnessMeter::new(48_000);
    let p = programme(48_000, &[(Some(-23.0), 1.0)]);
    m.push(&p[..p.len() - 1]);
    m.push(&[0.5]);
    assert!(m.finish().is_some());
}

#[test]
fn a_zero_rate_is_treated_as_one_hertz_without_panicking() {
    let mut m = LoudnessMeter::new(0);
    m.push(&[0.1, 0.1, 0.2, 0.2, 0.3, 0.3, 0.4, 0.4]);
    // Four frames at 1 Hz are four hops of one frame: one block.
    let _ = m.finish();
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p fp-dsp --test loudness 2>&1 | tail -5`
Expected: FAIL to compile, "unresolved import `fp_dsp::loudness`".

- [ ] **Step 3: Implement**

Add `pub mod loudness;` to `crates/fp-dsp/src/lib.rs` (between `kweight` and `truepeak`).

`crates/fp-dsp/src/loudness.rs`:

```rust
//! Integrated loudness of a whole programme (ITU-R BS.1770-5, loudness
//! normalization spec N4): the K-weighted mean square of both channels,
//! weighted 1.0 each, over 400 ms blocks with 75 % overlap, gated at
//! −70 LUFS (absolute) and 10 LU below the mean of the blocks above that
//! (relative); and the true peak of both channels.
//!
//! The energy of every 100 ms hop is kept, so memory grows with the length
//! of the file (about 36 000 numbers an hour). This runs in the analysis
//! pool, never on the device callback.

use crate::kweight::KWeighting;
use crate::truepeak::TruePeak;

/// The true peak reported for complete digital silence, whose true value
/// (−∞) is not a number a measurement can store. Far below any audible
/// level, so no gain rule ever depends on it.
pub const SILENCE_DBTP: f64 = -200.0;

/// The BS.1770 gating thresholds (LUFS and LU), fixed by the
/// Recommendation.
const ABSOLUTE_GATE_LUFS: f64 = -70.0;
const RELATIVE_GATE_LU: f64 = -10.0;
/// −0.691: the offset that makes a 997 Hz sine read its level.
const LOUDNESS_OFFSET: f64 = -0.691;
/// Hops per gating block: 4 × 100 ms = 400 ms, 75 % overlap.
const HOPS_PER_BLOCK: usize = 4;

/// The measurement of one programme.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Loudness {
    /// `None` when no block passes the absolute gate (silence or near it).
    pub integrated_lufs: Option<f64>,
    /// The largest true peak of both channels, dBTP; `SILENCE_DBTP` for
    /// digital silence.
    pub true_peak_dbtp: f64,
}

pub struct LoudnessMeter {
    k: [KWeighting; 2],
    tp: [TruePeak; 2],
    /// Frames per 100 ms hop (at least 1).
    hop_frames: u64,
    /// Frames in the hop being filled.
    in_hop: u64,
    /// Sum of the K-weighted squares of both channels in that hop.
    hop_energy: f64,
    /// The energy of every complete hop, in order.
    hops: Vec<f64>,
    /// Largest true-peak magnitude so far (linear).
    peak: f32,
}

fn lufs_of(mean_square: f64) -> f64 {
    LOUDNESS_OFFSET + 10.0 * mean_square.log10()
}

impl LoudnessMeter {
    pub fn new(sample_rate: u32) -> Self {
        let rate = sample_rate.max(1);
        Self {
            k: [KWeighting::new(rate), KWeighting::new(rate)],
            tp: [TruePeak::default(), TruePeak::default()],
            hop_frames: ((f64::from(rate) / 10.0).round() as u64).max(1),
            in_hop: 0,
            hop_energy: 0.0,
            hops: Vec::new(),
            peak: 0.0,
        }
    }

    /// Feeds interleaved stereo samples. A trailing half frame is ignored;
    /// a sample that is not finite counts as silence.
    pub fn push(&mut self, stereo: &[f32]) {
        for frame in stereo.as_chunks::<2>().0 {
            let mut energy = 0.0;
            for ((x, k), tp) in frame.iter().zip(&mut self.k).zip(&mut self.tp) {
                let x = if x.is_finite() { *x } else { 0.0 };
                self.peak = self.peak.max(tp.push(x));
                let w = k.process(f64::from(x));
                energy += w * w;
            }
            self.hop_energy += energy;
            self.in_hop += 1;
            if self.in_hop == self.hop_frames {
                self.hops.push(self.hop_energy);
                self.hop_energy = 0.0;
                self.in_hop = 0;
            }
        }
    }

    /// The result so far; `None` with fewer than 400 ms pushed (one gating
    /// block, spec N5) or when a value is not finite.
    pub fn finish(&self) -> Option<Loudness> {
        if self.hops.len() < HOPS_PER_BLOCK {
            return None;
        }
        let block_frames = (self.hop_frames * HOPS_PER_BLOCK as u64) as f64;
        // Mean square per block, summed over the two channels (weight 1.0).
        let blocks: Vec<f64> = self
            .hops
            .windows(HOPS_PER_BLOCK)
            .map(|w| w.iter().sum::<f64>() / block_frames)
            .collect();
        let above = |gate: f64| blocks.iter().copied().filter(move |z| lufs_of(*z) > gate);
        let mean = |it: &mut dyn Iterator<Item = f64>| {
            let (sum, n) = it.fold((0.0, 0usize), |(s, n), z| (s + z, n + 1));
            (n > 0).then(|| sum / n as f64)
        };
        let integrated_lufs = match mean(&mut above(ABSOLUTE_GATE_LUFS)) {
            None => None,
            Some(ungated) => {
                let relative = lufs_of(ungated) + RELATIVE_GATE_LU;
                let gate = relative.max(ABSOLUTE_GATE_LUFS);
                mean(&mut above(gate)).map(lufs_of)
            }
        };
        let true_peak_dbtp = if self.peak > 0.0 {
            20.0 * f64::from(self.peak).log10()
        } else {
            SILENCE_DBTP
        };
        let finite = integrated_lufs.is_none_or(f64::is_finite) && true_peak_dbtp.is_finite();
        finite.then_some(Loudness {
            integrated_lufs,
            true_peak_dbtp,
        })
    }
}
```

Note: the relative gate applies on top of the absolute one (`max`), which is what BS.1770 means by "blocks above both gates". If clippy objects to `as` casts, the existing crates allow them (no `cast_*` lint is denied).

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p fp-dsp`
Expected: PASS. If `a_sine_reads_its_level_at_every_rate` misses at 8 kHz by more than 0.15 dB, check first that the K-weighting gain at 1 kHz is the same as at 48 kHz (`k_weighting_gain_at_1_khz_is_the_same_at_every_rate` covers 44.1–192 kHz only); if the filter itself deviates there, record a Ruling widening the tolerance for rates under 16 kHz and say why — do not change the filter.

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-dsp
git commit -m "feat(dsp): measure the gated integrated loudness of a programme

The analysis will measure each file once with the BS.1770 gating, the
same K-weighting and true peak as the meters (loudness normalization
spec N4, N5). Files under one 400 ms block give no measurement.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: Documentation for the new crate

**Files:**
- Modify: `CLAUDE.md` (rule 6 list of crates that deny `clippy::indexing_slicing`; table "Where things are")
- Modify: `README.md` ("Workspace" crate table, ~line 231)
- Create: `docs/technical/dsp.md`
- Modify: `docs/technical/README.md` (link to `dsp.md` next to the other technical pages)
- Modify: `docs/technical/audio-engine.md:71,77` (the meters section names `truepeak.rs` and `kweight.rs`: say they live in `fp-dsp`)
- Modify: `docs/technical/audio-path-audit.md:424,454` (paths `truepeak.rs`, `kweight.rs` → `crates/fp-dsp/src/…`; do not touch historical findings rows such as A6 except the path)
- Modify: `docs/superpowers/specs/2026-09-27-meters-design.md` §M1: one sentence "The K-weighting and the true-peak interpolator live in the `fp-dsp` crate, shared with the analysis (loudness normalization spec N6)."

No code test: the gate is the full suite plus a link check of the edited Markdown (`scripts/site/build.sh` is not needed; check relative links by hand).

- [ ] **Step 1: CLAUDE.md**

Rule 6, second bullet becomes:

```markdown
   - `fp-engine`, `fp-backends`, `fp-decode`, `fp-analysis` and `fp-dsp` deny
     `clippy::indexing_slicing` (use `get`); prefer `get` everywhere.
```

"Where things are", a row after `crates/fp-decode`:

```markdown
| `crates/fp-dsp` | K-weighting, true peak, BS.1770 gating: pure signal processing shared by the engine and the analysis |
```

`AGENTS.md` stays a pointer (do not edit it).

- [ ] **Step 2: README.md**

Workspace table, a row after `fp-decode`:

```markdown
| [`fp-dsp`](crates/fp-dsp) | Signal processing shared by the meters and the analysis: K-weighting, true peak, BS.1770 loudness |
```

- [ ] **Step 3: `docs/technical/dsp.md`**

```markdown
# Shared signal processing (`fp-dsp`)

`fp-dsp` holds the signal processing that both the audio engine and the
analysis use. It has no dependencies, does no I/O, forbids `unsafe` and
denies `clippy::indexing_slicing`.

| Module | What | Used by |
|---|---|---|
| `kweight` | The ITU-R BS.1770 K-weighting: the pre-filter (high shelf) and the RLB high-pass, as two biquads derived for any rate from the standard's analogue prototypes. Filter states are flushed below 1e-20. | The mixer (per slot, on the device callback) and `loudness` |
| `truepeak` | The BS.1770-5 Annex 2 true-peak interpolator: 4× oversampling with the published order-48 FIR, in floating point. | The mixer and `loudness` |
| `loudness` | `LoudnessMeter`: the integrated loudness of a whole programme, gated (absolute −70 LUFS, relative −10 LU) over 400 ms blocks with a 100 ms hop, both channels weighted 1.0, and the largest true peak of both channels. | The analysis (loudness normalization spec N4) |

`KWeighting::process` and `TruePeak::push` use fixed arrays only and are
real-time safe. `LoudnessMeter` keeps the energy of every 100 ms hop
(about 36 000 numbers per hour of audio) and is never used on the device
callback.

A programme shorter than one gating block (400 ms) gives no measurement;
one with no block above the absolute gate has no integrated loudness. The
true peak of digital silence is reported as `SILENCE_DBTP` (−200 dBTP)
so that it is always a finite number. Samples that are not finite count
as silence.

Tests: `crates/fp-dsp/tests/filters.rs` (the standard's coefficients, EBU
Tech 3341 true-peak cases 15–23) and `crates/fp-dsp/tests/loudness.rs`
(EBU Tech 3341 cases 1–5 for integrated loudness, short and silent
programmes, non-finite input).
```

Add it to `docs/technical/README.md` in the list of pages, next to `audio-engine.md`, as "[Shared signal processing](dsp.md) — `fp-dsp`: K-weighting, true peak, integrated loudness".

- [ ] **Step 4: The other references** (audio-engine.md, audio-path-audit.md, meters spec), as listed under **Files**.

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add CLAUDE.md README.md docs
git commit -m "docs: describe the fp-dsp crate

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

## Self-review notes

- Spec coverage: N6 (`fp-dsp` paragraph) → Tasks 1 and 3; N4 measurement algorithm → Task 2; N5 short file and silence → Task 2; §10 `fp-dsp` row → Task 2 tests; §11 CLAUDE.md, README, `dsp.md`, meters spec M1 → Task 3. `deny.toml` needs no change (workspace crates are path dependencies with `publish = false`).
- Out of this plan: wiring into the analysis (plan 2).
