# Waveform with peak and RMS Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

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

- [x] Test `peaks_hold_the_rms_level_of_each_bucket` (a 0.5 constant bucket has rms 0.5·MAX; a bucket with one −1 sample of 80 has rms MAX/√80); watch it fail.
- [x] Add `pub struct WavePeak { min, max, rms: i16 }`, accumulate the bucket's sum of squares, change `peaks` to `Vec<WavePeak>` in `Envelope`, `Analysis`, `TrackMedia`; bump `ANALYSIS_VERSION` to 5.
- [x] Run `cargo test -p fp-analysis`; commit `feat(analysis): keep the RMS level of each waveform bucket`.

### Task 2: continuous waveform with a peak outline and RMS body

**Files:** `crates/fp-app/src/ui/widgets.rs`, `crates/fp-app/tests/waveform_view.rs`, docs.

- [x] Tests for `pub fn wave_columns(peaks, bucket_secs, span_secs, columns) -> Vec<WaveColumn>` (`WaveColumn { peak, rms }`, fractions of full scale): one column per pixel; a column's peak is its largest bucket and its RMS the root of the mean square; columns past the audio are zero; more columns than buckets leave no gaps. Watch them fail.
- [x] Implement it; draw each 1 px column: peak dimmed (0.45), RMS solid, played/unplayed colours.
- [x] Docs (spec, analysis.md, players.md); fmt, clippy, suite; commit `feat(ui): draw the waveform continuously with its RMS body`.

## Rulings and deferred minors

- Ruling: `wave_columns` returns a `WaveColumn { peak, rms }` struct, not a tuple — named fields read better in the widget and the tests — cost if wrong: one rename.
- Deferred minors (final review):
  - float truncation in `bucket_at` can drop the last 10 ms bucket when the span equals the audio length;
  - `a0 + 1` can overflow for a corrupt duration of about 1e17 s, and the column count is not capped for an unbounded width;
  - about 2 rects per pixel per player every frame, and the bucket reduction runs every frame (cache it per track and width, or build one mesh);
  - cache entries grow from about 6 to about 8 bytes per bucket, and old-version entries are never pruned;
  - no tests for a silent bucket, a NaN bucket length, or an old-version cache key missing.

## Follow-up (fix/meter-waveform-minors)

- Fixed: column boundaries get a 1e-6 bucket nudge so the last bucket is kept; a non-finite bucket length or span draws nothing; `a0 + 1` saturates.
- Fixed: the reduction is memoised per track (`Arc` identity), span and width (`memo_columns`), and the columns are drawn as one mesh.
- Fixed: cache file names carry the analysis version, and opening the cache prunes other versions.
- Fixed: tests for a silent bucket, non-finite inputs, a huge span and old-version pruning.
- Ruling: no cap on the column count — it is the widget's pixel width, bounded by the screen — cost if wrong: one large allocation whenever the track, span or width changes at an absurd width.
- Ruling: the RMS byte pair per bucket stays (about 8 bytes per 10 ms, about 2.9 MB for a 1 h track) — it is what the solid body is drawn from — cost if wrong: a larger cache directory.
Deferred minors (follow-up review): settled in [`2026-09-30-deferred-minors.md`](2026-09-30-deferred-minors.md).
