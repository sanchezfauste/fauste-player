# Waveform with peak and RMS Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** a waveform that shows a loud track's dynamics: peaks dimmed, RMS solid, drawn continuously.

**Architecture:** analysis stores the RMS of each peak bucket next to its min/max (`WavePeak`); the cache version is bumped so tracks are re-analysed once. The UI reduces the buckets to one column per pixel with a pure function and paints the peak dimmed and the RMS solid.

**Tech Stack:** Rust, fp-analysis, egui (fp-app).

**Spec:** `docs/superpowers/specs/2026-09-25-fauste-player-design.md` (§6 Peaks, §8 Waveform).

## Global Constraints

- `clippy::indexing_slicing` denied in fp-analysis; no unwrap/expect/panic outside tests.
- Linear amplitude, symmetric about the centre line; full scale = `i16::MAX`.
- Manual markers survive the re-analysis (they are model state, not cache).

## Review Focus

- A column wider than one bucket: RMS must be the root of the mean square, not the mean of RMS.
- A track shorter than the widget, or more pixels than buckets: no gaps between columns.
- Silence: RMS 0 draws nothing but the centre line.
- Full-scale square wave: RMS equals the peak and saturates without overflow.
- An old cache entry: re-analysed, never mis-decoded.

---

### Task 1: RMS per bucket in analysis

**Files:** `crates/fp-analysis/src/signal.rs`, `analyze.rs`, `cache.rs`, `crates/fp-app/src/services.rs`.

- [ ] Test `peaks_hold_min_max_and_rms_per_bucket` (a 0.5 constant bucket has rms 0.5·MAX; a bucket with one −1 sample of 80 has rms MAX/√80); watch it fail.
- [ ] Add `pub struct WavePeak { min, max, rms: i16 }`, accumulate the bucket's sum of squares, change `peaks` to `Vec<WavePeak>` in `Envelope`, `Analysis`, `TrackMedia`; bump `ANALYSIS_VERSION` to 5.
- [ ] Run `cargo test -p fp-analysis`; commit `feat(analysis): keep the RMS level of each waveform bucket`.

### Task 2: continuous waveform with a peak outline and RMS body

**Files:** `crates/fp-app/src/ui/widgets.rs`, `crates/fp-app/tests/waveform_view.rs`, docs.

- [ ] Tests for `pub fn wave_columns(peaks, bucket_secs, span_secs, columns) -> Vec<(f32, f32)>`: one column per pixel; a column's peak is its largest bucket and its RMS the root of the mean square; columns past the audio are zero; more columns than buckets leave no gaps. Watch them fail.
- [ ] Implement it; draw each 1 px column: peak dimmed (0.45), RMS solid, played/unplayed colours.
- [ ] Docs (spec, analysis.md, players.md); fmt, clippy, suite; commit `feat(ui): draw the waveform continuously with its RMS body`.
