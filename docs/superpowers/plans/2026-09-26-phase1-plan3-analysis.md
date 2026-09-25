# Phase 1 · Plan 3 — Track analysis Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement `fp-analysis` (spec §6) and connect its results to the model. The crate reads tags and cover art, computes waveform peaks and the automatic cue markers (`cue_in`, `cue_out`, `segue_start`, `outro_start`), caches everything per file, and runs the jobs on a small background pool.

**Architecture:**
- **Shared decoder:** decoding moves from `fp-engine` into a new `fp-decode` crate, so analysis does not depend on the audio engine. `fp-engine` re-exports it, so existing paths keep working.
- **Pure signal analysis:** `fp-analysis::signal` works on a mono envelope and is fully testable with synthetic signals.
- **Metadata:** `fp-analysis::metadata` wraps lofty (tags) and image (a cover thumbnail with decode limits).
- **Cache:** `fp-analysis::cache` stores one postcard file per track. The key covers the path, size, mtime, analysis version and the `AnalysisSettings` fingerprint.
- **Worker pool:** `fp-analysis::Analyzer` runs N worker threads over a job queue. Jobs can be cancelled; results come out on a channel.
- **Model:** a new `Command::ApplyAnalysis` stores metadata and automatic markers. Manual markers win, and the reducer's `reconcile` reschedules any transition whose markers changed.

**Tech Stack:** lofty 0.25.4, image 0.25.10 (`jpeg`, `png`), postcard 1.1.3 (`alloc`), serde, crossbeam-channel, symphonia (via `fp-decode`), hound and tempfile (dev). All APIs were verified by a compiled probe on 2026-09-26.

**Spec:** `docs/superpowers/specs/2026-09-25-fauste-player-design.md` §6 (analysis), §2.4 (settings), §10 (input limits).

**Execution note:** the human partner delegated execution ("continue until all plans are finished"). The executor is also the plan author, so tasks list exact interfaces, behaviours and tests, and the code is written TDD during execution rather than embedded here. A fresh reviewer checks the whole branch at the end.

## Global Constraints

- **Plan 1–2 constraints carry over:**
  - English only, and no third-party radio-automation product names;
  - no `unwrap`/`expect`/`panic`, `forbid(unsafe_code)`, and `deny(clippy::indexing_slicing)` in library crates;
  - fmt, clippy `-D warnings` and tests must pass before every commit;
  - commits end with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- **No hardcoded product limits:** every threshold comes from `AnalysisSettings` and every size limit from `Limits` (`max_cover_bytes`, `max_cover_pixels`, `max_state_file_bytes`).
- **Intro:** `intro_end` is never automatic (spec §6).
- **Minimum duration:** tracks shorter than `markers_min_duration_secs` get no segue start and no outro.
- **Robustness:** analysis never panics on bad input.
  - An unreadable file becomes `FileState::Unreadable`, and a missing file becomes `FileState::Missing`.
  - A corrupt cache entry is recomputed.
  - A cover larger than the limits is ignored, and the rest of the analysis still succeeds.

## Review Focus

1. **A file with no tags at all** (a plain WAV or a stripped MP3). Expect the title and artist to come from `Artist - Title` in the file name, and the rest to stay empty. Test: `untagged_files_fall_back_to_the_file_name`.
2. **An enormous or malformed embedded cover.** Expect the cover to be skipped, never a crash or a huge allocation. Test: `oversized_covers_are_ignored`.
3. **Analysis settings changed by the operator.** Expect the cached results for the old settings to be ignored, and the markers recomputed. Test: `changing_the_settings_invalidates_the_cache`.
4. **Silence-only or very quiet files.** Expect the cue points to cover the whole file, no segue start and no outro. Test: `a_silent_file_keeps_the_whole_duration_and_gets_no_segue`.
5. **The user placed markers manually, then re-analysed.** Expect the manual markers to survive (spec §6). Test: `re_analysis_never_overwrites_manual_markers` (model).

---

### Task 1: Extract `fp-decode`

- **Create** `crates/fp-decode` (`Cargo.toml` and `src/lib.rs`) with the current contents of `fp-engine/src/decode.rs`: `FileDecoder` and `downmix`, plus their unit test.
- **`FileDecoder` gains two methods:**
  - `frames_hint() -> Option<u64>`, the track's `num_frames` when the container knows it;
  - `channels()`.
- **`fp-engine`:**
  - depends on `fp-decode`;
  - `src/decode.rs` becomes `pub use fp_decode::*;`;
  - the existing decode tests keep passing unchanged.
- **Tests:** the existing `crates/fp-engine/tests/decode.rs` suite still passes (a behaviour-preserving move).

### Task 2: Model — apply analysis results

- **`fp-model::track`:**
  - add `TrackAnalysis { title: Option<String>, artist: Option<String>, album: Option<String>, duration_secs: f64, cue_in: Option<f64>, cue_out: Option<f64>, segue_start: Option<f64>, outro_start: Option<f64> }`;
  - add `Track::apply_analysis(&mut self, &TrackAnalysis)`.
- **`apply_analysis` behaviour:**
  - fills in title, artist and album only when the analysis found them (`Some`), so nothing previously known is erased;
  - sets the duration and stores every marker with `Markers::set_auto`, so manual markers win;
  - sets `analyzed = true` and `file_state = Ok`.
- **`Command`** gains:
  - `ApplyAnalysis { track: TrackId, analysis: Box<TrackAnalysis> }`;
  - `SetFileState { track: TrackId, state: FileState }`, used for Missing or Unreadable from analysis.
- **Reducer:**
  - both commands update the library, then call `refresh_next` (a Missing file is skipped as next) and `reconcile`, so a newly known segue or duration reschedules the current transition;
  - an unknown track id is ignored; it is not an error, because analysis may finish after the track was removed.
- **Tests (`crates/fp-model/tests/analysis.rs`):**
  - `analysis_fills_metadata_and_markers`;
  - `analysis_does_not_erase_known_metadata_with_nothing`;
  - `re_analysis_never_overwrites_manual_markers`;
  - `a_newly_known_segue_reschedules_the_playing_transition`, which expects a `Schedule` action with `StartNextAt` at the segue after analysis arrives for the current track;
  - `a_missing_file_is_skipped_as_next`;
  - `results_for_removed_tracks_are_ignored`.

### Task 3: Signal analysis (pure)

`crates/fp-analysis/src/signal.rs`:

- **`Envelope`:**
  - `from_stereo(samples: &[f32], sample_rate, window_ms, bucket_ms)` accumulates mono RMS windows (`rms_db` per window, with a floor of −120 dB) and peak buckets (min/max as `i16`) in one pass;
  - it is streaming: `Envelope::new(...)`, then `push(&[f32])`, then `finish() -> Envelope` holding `duration_secs`, `rms_db: Vec<f32>`, `window_secs` and `peaks: Vec<(i16, i16)>`.
- **`detect_markers(&Envelope, &AnalysisSettings) -> AutoMarkers { cue_in, cue_out, segue_start: Option, outro_start: Option }`:**

  | Marker | Rule |
  |---|---|
  | `cue_in` | start of the first window whose RMS ≥ `silence_threshold_db`, else 0 |
  | `cue_out` | end of the last such window, else the duration |
  | `segue_start` | scanning back from `cue_out`, the end of the first window with RMS ≥ `segue_threshold_db`, clamped to `[cue_out − segue_max_secs, cue_out]` and to ≥ `cue_in`; if none, `cue_out − segue_max_secs` (clamped) |
  | `outro_start` | median dB of the windows in `[cue_in, cue_out]` minus `outro_drop_db`; scanning back, the end of the first window ≥ that level, clamped to ≥ `cue_out − outro_max_secs` and ≥ `cue_in` |

  Tracks shorter than `markers_min_duration_secs` get no `segue_start` and no `outro_start`.
- **Tests** (unit, synthetic signals):
  - `silence_at_both_ends_is_trimmed` (1 s silence, 60 s tone, 2 s silence);
  - `segue_starts_where_the_tail_drops_below_the_threshold` (a tone ending in a linear fade to silence over 6 s, with the threshold at −18 dB);
  - `segue_is_limited_to_max_seconds_before_cue_out` (a 20 s slow fade with `segue_max` 8);
  - `a_silent_file_keeps_the_whole_duration_and_gets_no_segue`;
  - `short_tracks_get_no_segue_or_outro`;
  - `outro_starts_where_the_level_drops_below_the_median`;
  - `peaks_hold_min_and_max_per_bucket`;
  - `streaming_equals_one_shot`.

### Task 4: Tags and cover art

`crates/fp-analysis/src/metadata.rs`:

- **`read_tags(path) -> Tags { title, artist, album: Option<String>, cover: Option<Vec<u8>> }`** uses lofty's primary tag, falling back to the first tag. A lofty error means "no tags", not a failure.
- **`title_from_file_name(path) -> (Option<artist>, title)`** splits the stem on the first `" - "`.
- **`thumbnail_png(bytes, limits: &Limits, px) -> Option<Vec<u8>>`:**
  - rejects input larger than `max_cover_bytes`;
  - decodes with `image::Limits` set from `max_cover_pixels` and a memory cap;
  - produces an aspect-preserving thumbnail of at most `px`, encoded as PNG;
  - returns `None` on any error.
- **Tests** (fixtures made with hound and lofty in a temp dir):
  - `tags_and_cover_are_read`;
  - `untagged_files_fall_back_to_the_file_name`;
  - `oversized_covers_are_ignored` (a 9000×10 PNG with the pixel limit at 8000, and a byte limit exceeded);
  - `corrupt_cover_bytes_are_ignored`.

### Task 5: Analysis of one file, and the cache

- **`analyze_file(path, &AnalysisSettings, &Limits) -> Result<Analysis, AnalysisError>`:**
  - `Analysis` = `{ analysis: TrackAnalysis, peaks: Vec<(i16, i16)>, peak_bucket_secs: f64, cover_png: Option<Vec<u8>> }`;
  - `AnalysisError` = `Missing` or `Unreadable(String)`;
  - it decodes with `fp-decode` in streaming fashion into the `Envelope`, detects the markers, and reads tags and cover (tag title and artist win over the file name).
- **`AnalysisCache::new(dir)`** with these methods:
  - `load(path, settings, limits) -> Option<Analysis>`;
  - `store(path, settings, &Analysis)`.
- **Cache format:**
  - the file name is the FNV-1a 64 hash (hex) of the key;
  - the key = canonical path, file size, mtime (ns), `ANALYSIS_VERSION` and the settings fingerprint (the `Debug` of `AnalysisSettings`);
  - the body is postcard of `CachedAnalysis { key, analysis }`;
  - `load` checks the key string, which protects against hash collisions, and the size limit; a corrupt file is removed;
  - `store` writes to a temp file and renames it into place.
- **Tests:**
  - `analyzing_a_wav_fixture_finds_duration_markers_and_peaks`;
  - `a_missing_file_is_reported_as_missing`;
  - `a_non_audio_file_is_reported_as_unreadable`;
  - `cache_round_trips`;
  - `a_modified_file_invalidates_the_cache`;
  - `changing_the_settings_invalidates_the_cache`;
  - `a_corrupt_cache_entry_is_recomputed`.

### Task 6: The analyzer pool

- **`Analyzer::spawn(threads, settings, limits, cache: Option<AnalysisCache>) -> Analyzer`.**
- **Methods:**
  - `submit(track: TrackId, path)`;
  - `cancel(track)`;
  - `update_settings(settings)`, which applies to jobs submitted later;
  - `results() -> &Receiver<AnalysisResult>`.
- **`AnalysisResult`** = `{ track, outcome: Result<Analysis, AnalysisError> }`.
- **Behaviour:**
  - it checks the cache first;
  - a job that has been cancelled, or is a duplicate of one already queued, is skipped;
  - it uses `threads` worker threads; the default comes from the app (2);
  - a panic while analysing is contained and reported as `Unreadable`;
  - dropping the `Analyzer` stops and joins the threads.
- **Tests:**
  - `submitted_files_are_analysed_in_the_background`;
  - `cancelled_jobs_produce_no_result`;
  - `cached_results_are_returned_without_analysing_again`: the analyse function is injected and counts its calls; a second submit of the same unchanged file is served from the cache and the count stays at 1;
  - `a_panicking_decoder_is_reported_as_unreadable`, using an injected analyse function.

## Spec coverage

| Spec §6 item | Task |
|---|---|
| tags via lofty, file-name fallback | 4, 5 |
| cover art with limits, thumbnail | 4 |
| peaks per bucket | 3, 5 |
| RMS envelope | 3 |
| cue_in / cue_out / segue_start / outro_start rules | 3 |
| intro never automatic | 2, 3 |
| marker provenance: manual wins | 2 |
| cache keyed by path/size/mtime/version; corrupt → recompute | 5 |
| playable before analysis; markers apply when analysis lands | 2 |
| background pool, cancellable | 6 |
| INTRO tag convention | Phase 2 (deferred per spec) |

## Review outcome and interface changes

The whole-branch review found 1 critical and 4 important issues; four minors were promoted by effect. All of them were fixed with tests that failed first:

| # | Issue | Fix |
|---|---|---|
| 1 | A stale preload after analysis found the real cue-in (dead air) | `PlayerState.preloaded` now tracks `(entry, from_secs)` |
| 2 | Dropping the analyzer ran the whole queue | a stop flag, plus a cancellation check between decoded blocks |
| 3 | A large cover dropped every tag | lofty's per-thread allocation limit follows `max_cover_bytes` |
| 4 | Very quiet files got a segue | no segue unless the level ever reaches `segue_threshold_db`; no zero-length outro |
| 5 | A stale automatic segue sat outside a manual cue-out | `plan_for` ignores a segue outside `[cue_in, cue_out)` |

The promoted minors:

- the cache key is taken before analysing;
- the whole job runs under `catch_unwind`;
- tag and cover panics only lose metadata;
- a mid-stream decode error keeps the audio decoded so far.

The interfaces differ from Task 5–6 as written:

- **Cache:**
  - `AnalysisCache::new(dir, &Limits)`, and the cover limits are part of the key;
  - an explicit `key` / `load_key` / `store_key` API.
- **Analyzer:**
  - `Analyzer::spawn` returns `io::Result`;
  - `AnalyzeFn` takes a fourth argument, `&dyn Fn() -> bool` ("cancelled?");
  - `AnalysisError::Cancelled`;
  - `analyze_file_cancellable`;
  - submissions carry generation numbers.
