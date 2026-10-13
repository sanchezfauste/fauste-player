# Loudness Normalization, Plan 2: Measurement, Cache and the Loudness-Only Pass

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Every file gets an integrated-loudness and true-peak measurement: a full analysis measures it, a background loudness-only pass measures the tracks analysed before this feature, the result is kept in the analysis cache and in the library (`Track::loudness`), and nothing is re-analysed in full because of it.

**Architecture:** `fp-model` gains the data (`LoudnessMeasurement`, `LOUDNESS_VERSION`, `Track::loudness`, `Track::loudness_pass`, `TrackAnalysis::{loudness, loudness_pass}`, `Command::ApplyLoudness`). `fp-analysis` feeds each decoded block to `fp_dsp::loudness::LoudnessMeter` during the full analysis, offers `measure_loudness_cancellable` for the loudness-only job, stores the measurement in the cache entry as a postcard **tail record** after the unchanged entry, and runs loudness-only jobs on a third, lowest-priority lane of the pool. The services thread queues those jobs only while no normal analysis is in flight, at most `Analyzer::loudness_share()` at a time, and sends each answer as `Command::ApplyLoudness`. Nothing uses the measurement for gain yet (plan 3).

**Tech Stack:** Rust 2024; `fp-dsp` (plan 1), `postcard`, `crossbeam-channel`, `tracing`; `hound` and `tempfile` in tests.

**Spec:** `docs/superpowers/specs/2026-10-08-loudness-normalization-design.md` (§3 terms, §5 N4–N6b, §10 rows `fp-model (tests/analysis.rs)` and `fp-analysis`, §11, §13 main spec §6, §14 Q3, Q7).

**Ruling (planning, 2026-10-08): the cache tail record.** Cache entries are postcard, which is not self-describing: a field added to `TrackAnalysis` with `#[serde(default)]` does **not** make old entries decode (postcard would read the new field out of the following bytes, fail, and the entry would be deleted and the whole library re-analysed — exactly what N6/Q3 forbid). So the two new `TrackAnalysis` fields are `#[serde(skip)]` (the entry's bytes do not change) and the cache writes them as a second postcard value, `LoudnessTail { loudness, pass }`, appended after the entry in the same file, under the same key, with the same atomic replace. `postcard::from_bytes` ignores trailing bytes and `postcard::take_from_bytes` returns them, so: an old entry has no tail (`loudness = None`, `loudness_pass = None`); a new entry read by an older build still decodes. — Why: it is the only layout that keeps every existing entry valid without bumping `ANALYSIS_VERSION`. — Cost if wrong: a later change to `CachedAnalysis` must keep the tail last; documented in `cache.rs` and `docs/technical/analysis.md`.

**Ruling (planning): `LOUDNESS_VERSION` lives in `fp-model`** (`fp_model::loudness::LOUDNESS_VERSION`), not in `fp-analysis::cache` next to `ANALYSIS_VERSION`: the model decides which tracks still need the pass (`Track::needs_loudness_pass`) and the Settings line counts them (`loudness_unmeasured`, N22), and `fp-model` cannot depend on `fp-analysis`. `fp-analysis` re-exports it from `cache` for readers looking there.

**Ruling (planning): the bounded share is enforced by the services thread**, not by the workers: the services thread never has more than `Analyzer::loudness_share()` (`threads − 1`, at least 1) loudness jobs in flight, and queues none while any normal analysis is in flight; the workers take the loudness lane last (`select_biased!`). — Why: the pool stays one simple loop, and the limit is testable in the pool and in the services thread. — Cost if wrong: with a one-thread pool, a running loudness job delays a new normal job by one file (seconds); the spec allows "at least one".

## Global Constraints

- All code, identifiers, comments, docs and commit messages in English; never mention other playout or radio-automation products.
- "`ANALYSIS_VERSION` is not bumped (it stays 8)" (N6). A cache entry written before this work must still load, with `loudness = None` and `loudness_pass = None`.
- `LOUDNESS_VERSION = 1`; `loudness_pass` records "the version of the loudness pass that ran on the file … whatever its outcome" (N6).
- "What is measured is what the players play: the stereo downmix `FileDecoder` gives, at the file's own rate, both channels weighted 1.0. A damaged stretch that ends decoding early is measured up to that point" (N4).
- N5: no block above the absolute gate → `integrated_lufs = None`; shorter than 400 ms → `loudness = None`, **no warning**; a non-finite result or a panic inside the meter → `loudness = None` and a `tracing::warn!` with the path, the rest of the analysis kept.
- "`Track::apply_analysis` stores the measurement in `Track::loudness`; it never touches `loudness_edit`, and a loudness-only result never touches markers" (N6). (`loudness_edit` arrives in plan 3.)
- N6b: the pass runs "below every normal analysis job", "only when the normal queue is empty", "at most all but one worker, at least one", never replaces markers or edits, writes "into the existing cache entry (same key, atomic replace)", records failures and short files so they are not retried, "does not depend on `normalize`".
- Untrusted input never crashes (CLAUDE.md rule 9); `unwrap`, `expect`, `panic` denied outside tests; `fp-analysis` denies `clippy::indexing_slicing`.
- The UI never blocks: all of this runs on the services thread and the analysis pool.
- Conventional Commits; every commit ends with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Gate for every task: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`.
- Progress ledger: `.superpowers/sdd/2026-10-08-loudness-plan2-measurement/progress.md`.

## Review Focus

1. **A cache entry written by the current release** (no tail) must load with markers and peaks intact and no loudness, and must not be deleted as corrupt. Test: Task 2, `an_entry_without_loudness_still_loads`.
2. **A corrupt or truncated tail** (a crash between the two writes is impossible thanks to the atomic rename, but a disk error is not): the entry still loads, without loudness, and is not deleted. Test: Task 2, `a_broken_tail_is_ignored`.
3. **A full analysis loaded from an old cache entry after the pass already measured the track** (Re-analyse, or markers reset): `apply_analysis` must not wipe the track's measurement with the entry's `None`. Test: Task 1, `an_analysis_without_a_loudness_pass_keeps_the_measurement`.
4. **A track removed from the library while its loudness job runs**: no `ApplyLoudness` for a gone track changes anything, and the pass forgets it. Test: Task 4, `a_track_removed_while_measured_is_forgotten`.
5. **A library of thousands of old tracks at start**: the pass never queues more than the share at once, and a new file added meanwhile is analysed first. Test: Task 4, `the_pass_waits_for_normal_analysis_and_keeps_its_share`.

---

## File Structure

| File | Responsibility |
|---|---|
| `crates/fp-model/src/loudness.rs` (new) | `LOUDNESS_VERSION`, `LoudnessMeasurement`, `loudness_unmeasured` (plan 3 adds the gain rules here) |
| `crates/fp-model/src/lib.rs` | `pub mod loudness;` and re-exports |
| `crates/fp-model/src/track.rs` | `Track::{loudness, loudness_pass}`, `needs_loudness_pass`, `TrackAnalysis::{loudness, loudness_pass}` (`serde(skip)`), `apply_analysis`, `apply_loudness` |
| `crates/fp-model/src/command.rs` | `Command::ApplyLoudness` |
| `crates/fp-model/src/reducer.rs:152` | The `ApplyLoudness` arm next to `ApplyAnalysis` |
| `crates/fp-model/tests/analysis.rs` | N6 model tests |
| `crates/fp-analysis/Cargo.toml` | Depends on `fp-dsp` |
| `crates/fp-analysis/src/analyze.rs` | The meter in the full analysis; `measure_loudness_cancellable` |
| `crates/fp-analysis/src/cache.rs` | The tail record; `update_loudness`; re-export `LOUDNESS_VERSION` |
| `crates/fp-analysis/src/analyzer.rs` | The loudness lane, `LoudnessFn`, `LoudnessResult`, `with_fns`, `submit_loudness`, `cancel_loudness`, `loudness_results`, `loudness_share` |
| `crates/fp-analysis/src/lib.rs` | Re-export `measure_loudness_cancellable` |
| `crates/fp-analysis/tests/analyze.rs` | N4–N6 measurement and cache tests |
| `crates/fp-analysis/tests/loudness_pass.rs` (new) | N6b pool tests |
| `crates/fp-app/src/services.rs` | `loudness_pass` step, `ApplyLoudness` |
| `crates/fp-app/tests/services.rs` | N6b services tests |
| `docs/technical/analysis.md`, `docs/technical/persistence.md`, main spec §6 | Task 5 |

Literal `TrackAnalysis { … }` constructions that list every field and must gain `loudness: None, loudness_pass: None` (or end with `..TrackAnalysis::default()`), found with `cargo check --workspace --all-targets`: `crates/fp-model/tests/{analysis,markers,track_tags,header_duration,pending_start,cartwall,cue_output,live}.rs`, `crates/fp-app/tests/{services,waveform_ui,cartwall_ui,markers_ui}.rs` (those already ending in `..TrackAnalysis::default()` need nothing).

---

### Task 1: The measurement in the model (N6)

**Files:**
- Create: `crates/fp-model/src/loudness.rs`
- Modify: `crates/fp-model/src/lib.rs`, `crates/fp-model/src/track.rs` (`Track` ~L273-310, `Track::new` ~L351-375, `TrackAnalysis` ~L460-482, `apply_analysis` ~L487-517), `crates/fp-model/src/command.rs` (after `ApplyAnalysis`, ~L64), `crates/fp-model/src/reducer.rs:152-158`
- Modify: the `TrackAnalysis` literals listed above (compile fixes only)
- Test: `crates/fp-model/tests/analysis.rs`

**Interfaces:**
- Produces (re-exported at the `fp_model` root):
  - `pub const LOUDNESS_VERSION: u32 = 1;`
  - `#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)] pub struct LoudnessMeasurement { pub integrated_lufs: Option<f32>, pub true_peak_dbtp: f32 }`
  - `Track::loudness: Option<LoudnessMeasurement>`, `Track::loudness_pass: Option<u32>` (both `#[serde(default)]`)
  - `Track::needs_loudness_pass(&self) -> bool`: analysed, playable, and `loudness_pass != Some(LOUDNESS_VERSION)`
  - `Track::apply_loudness(&mut self, loudness: Option<LoudnessMeasurement>, version: u32)`
  - `TrackAnalysis::loudness: Option<LoudnessMeasurement>`, `TrackAnalysis::loudness_pass: Option<u32>` (both `#[serde(skip)]`, see the ruling)
  - `Command::ApplyLoudness { track: TrackId, loudness: Option<LoudnessMeasurement>, version: u32 }`
  - `pub fn loudness_unmeasured(state: &AppState) -> usize` (N22's count; used by plan 5)

- [ ] **Step 1: Write the failing tests** (append to `crates/fp-model/tests/analysis.rs`; add `LoudnessMeasurement, LOUDNESS_VERSION, loudness_unmeasured` to its `use fp_model::{…}`)

```rust
fn measured(lufs: f32) -> LoudnessMeasurement {
    LoudnessMeasurement {
        integrated_lufs: Some(lufs),
        true_peak_dbtp: -1.5,
    }
}

fn with_loudness(duration: f64, lufs: Option<f32>) -> TrackAnalysis {
    TrackAnalysis {
        loudness: lufs.map(measured),
        loudness_pass: Some(LOUDNESS_VERSION),
        ..analysis(duration)
    }
}

#[test]
fn n6_an_analysis_stores_the_loudness_measurement() {
    let mut state = fixture(1);
    let t = track_of(&state, 0);
    apply(
        &mut state,
        Command::ApplyAnalysis {
            track: t,
            analysis: Box::new(with_loudness(200.0, Some(-11.0))),
        },
    )
    .unwrap();
    let track = state.library.get(t).unwrap();
    assert_eq!(track.loudness, Some(measured(-11.0)));
    assert_eq!(track.loudness_pass, Some(LOUDNESS_VERSION));
    assert!(!track.needs_loudness_pass());
}

#[test]
fn an_analysis_without_a_loudness_pass_keeps_the_measurement() {
    // An entry cached before this feature carries no measurement: the one
    // the loudness-only pass found stays.
    let mut state = fixture(1);
    let t = track_of(&state, 0);
    state
        .library
        .get_mut(t)
        .unwrap()
        .apply_loudness(Some(measured(-9.0)), LOUDNESS_VERSION);
    apply(
        &mut state,
        Command::ApplyAnalysis {
            track: t,
            analysis: Box::new(analysis(200.0)),
        },
    )
    .unwrap();
    let track = state.library.get(t).unwrap();
    assert_eq!(track.loudness, Some(measured(-9.0)));
    assert_eq!(track.loudness_pass, Some(LOUDNESS_VERSION));
}

#[test]
fn n6_a_track_analysed_before_this_feature_needs_the_loudness_pass() {
    let mut state = fixture(1);
    let t = track_of(&state, 0);
    apply(
        &mut state,
        Command::ApplyAnalysis {
            track: t,
            analysis: Box::new(analysis(200.0)),
        },
    )
    .unwrap();
    let track = state.library.get(t).unwrap();
    assert_eq!((track.loudness, track.loudness_pass), (None, None));
    assert!(track.needs_loudness_pass());
}

#[test]
fn n6b_a_loudness_only_result_changes_no_marker() {
    let mut state = fixture(1);
    let t = track_of(&state, 0);
    apply(
        &mut state,
        Command::ApplyAnalysis {
            track: t,
            analysis: Box::new(analysis(200.0)),
        },
    )
    .unwrap();
    apply(
        &mut state,
        Command::SetMarker {
            track: t,
            kind: MarkerKind::IntroEnd,
            secs: Some(12.0),
        },
    )
    .unwrap();
    let before = state.library.get(t).unwrap().clone();
    apply(
        &mut state,
        Command::ApplyLoudness {
            track: t,
            loudness: Some(measured(-14.0)),
            version: LOUDNESS_VERSION,
        },
    )
    .unwrap();
    let after = state.library.get(t).unwrap();
    assert_eq!(after.markers, before.markers);
    assert_eq!((after.title.as_str(), after.duration_secs), (before.title.as_str(), before.duration_secs));
    assert_eq!(after.loudness, Some(measured(-14.0)));
    assert!(!after.needs_loudness_pass());
}

#[test]
fn n6b_a_failed_or_short_measurement_is_recorded_and_not_asked_again() {
    let mut state = fixture(1);
    let t = track_of(&state, 0);
    apply(
        &mut state,
        Command::ApplyAnalysis {
            track: t,
            analysis: Box::new(analysis(200.0)),
        },
    )
    .unwrap();
    apply(
        &mut state,
        Command::ApplyLoudness {
            track: t,
            loudness: None,
            version: LOUDNESS_VERSION,
        },
    )
    .unwrap();
    let track = state.library.get(t).unwrap();
    assert_eq!(track.loudness, None);
    assert!(!track.needs_loudness_pass());
}

#[test]
fn n6b_a_newer_loudness_version_asks_for_the_pass_again() {
    let mut state = fixture(1);
    let t = track_of(&state, 0);
    let track = state.library.get_mut(t).unwrap();
    track.analyzed = true;
    track.apply_loudness(Some(measured(-14.0)), LOUDNESS_VERSION - 1);
    assert!(track.needs_loudness_pass());
}

#[test]
fn a_loudness_result_for_a_gone_track_is_ignored() {
    let mut state = fixture(1);
    let before = state.clone();
    apply(
        &mut state,
        Command::ApplyLoudness {
            track: TrackId(999),
            loudness: Some(measured(-14.0)),
            version: LOUDNESS_VERSION,
        },
    )
    .unwrap();
    assert_eq!(state.library, before.library);
}

#[test]
fn n22_tracks_waiting_for_a_measurement_are_counted() {
    let mut state = fixture(3);
    let ids: Vec<TrackId> = (0..3).map(|n| track_of(&state, n)).collect();
    // One measured, one tried and failed, one never tried.
    state.library.get_mut(ids[0]).unwrap().apply_loudness(Some(measured(-14.0)), LOUDNESS_VERSION);
    state.library.get_mut(ids[1]).unwrap().apply_loudness(None, LOUDNESS_VERSION);
    assert_eq!(loudness_unmeasured(&state), 1);
    // A missing file is not waiting for anything.
    state.library.get_mut(ids[2]).unwrap().file_state = FileState::Missing;
    assert_eq!(loudness_unmeasured(&state), 0);
}

#[test]
fn n6_a_library_saved_before_loudness_loads_without_it() {
    let json = r#"[{"id":1,"path":"/a.flac","title":"A","artist":"","album":"",
        "duration_secs":10.0,"kind":"Music","file_state":"Ok","markers":{},
        "analyzed":true}]"#;
    let lib: fp_model::Library = serde_json::from_str(json).unwrap();
    let t = lib.get(TrackId(1)).unwrap();
    assert_eq!((t.loudness, t.loudness_pass), (None, None));
}
```

If `serde_json` is not a dev-dependency of `fp-model`, it is (the unit tests in `track.rs` use it); otherwise add `serde_json.workspace = true` under `[dev-dependencies]`.

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p fp-model --test analysis 2>&1 | tail -5`
Expected: FAIL to compile (`LoudnessMeasurement`, `ApplyLoudness`, … not found).

- [ ] **Step 3: Implement**

`crates/fp-model/src/loudness.rs`:

```rust
//! Loudness normalization (loudness normalization spec, 2026-10-08): what
//! the analysis measured of a track, and the pure rules that turn it into
//! a fixed gain per source (N7–N13, added with the gain rules).

use serde::{Deserialize, Serialize};

use crate::state::AppState;

/// The version of the loudness measurement (N6). Bump it when the
/// measurement changes: the loudness-only pass then measures every track
/// again, and nothing else is analysed again.
pub const LOUDNESS_VERSION: u32 = 1;

/// The integrated loudness and true peak of a whole file (spec §3).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LoudnessMeasurement {
    /// ITU-R BS.1770 gated integrated loudness, LUFS; `None` when no block
    /// passes the absolute gate (a silent file).
    pub integrated_lufs: Option<f32>,
    /// The largest true peak of both channels, dBTP.
    pub true_peak_dbtp: f32,
}

/// N22: playable tracks with no measurement that the loudness-only pass
/// (or their first analysis) has not tried yet.
pub fn loudness_unmeasured(state: &AppState) -> usize {
    state
        .library
        .iter()
        .filter(|t| {
            t.file_state.is_playable()
                && t.loudness.is_none()
                && t.loudness_pass != Some(LOUDNESS_VERSION)
        })
        .count()
}
```

`lib.rs`: `pub mod loudness;` (alphabetical, after `live`), and `pub use loudness::{LOUDNESS_VERSION, LoudnessMeasurement, loudness_unmeasured};`.

`track.rs` — in `Track`, after `tags_read`:

```rust
    /// What the analysis measured (loudness normalization spec N6); `None`
    /// is "not measured". Libraries saved earlier have none until the
    /// loudness-only pass measures them.
    #[serde(default)]
    pub loudness: Option<LoudnessMeasurement>,
    /// The `LOUDNESS_VERSION` of the pass that last ran on the file,
    /// whatever it found; `None` before any.
    #[serde(default)]
    pub loudness_pass: Option<u32>,
```

`Track::new` sets both to `None`. Add (with `use crate::loudness::{LOUDNESS_VERSION, LoudnessMeasurement};` at the top):

```rust
    /// N6b: whether the loudness-only pass should measure this track:
    /// analysed, playable, and not measured by the current pass version.
    /// A track not analysed yet gets its measurement from the analysis.
    pub fn needs_loudness_pass(&self) -> bool {
        self.analyzed
            && self.file_state.is_playable()
            && self.loudness_pass != Some(LOUDNESS_VERSION)
    }

    /// N6b: a loudness-only result. Only the measurement changes.
    pub fn apply_loudness(&mut self, loudness: Option<LoudnessMeasurement>, version: u32) {
        self.loudness = loudness;
        self.loudness_pass = Some(version);
    }
```

In `TrackAnalysis`, after `version`:

```rust
    /// The loudness measurement (N6). Not part of the cache entry's own
    /// bytes: the cache stores it as a tail record (see `fp_analysis::cache`).
    #[serde(skip)]
    pub loudness: Option<LoudnessMeasurement>,
    /// The `LOUDNESS_VERSION` that measured it; `None` for a result cached
    /// before loudness was measured.
    #[serde(skip)]
    pub loudness_pass: Option<u32>,
```

In `apply_analysis`, before `self.analysis_version = analysis.version;`:

```rust
        // A result cached before loudness was measured keeps what the
        // loudness-only pass found (N6).
        if let Some(version) = analysis.loudness_pass {
            self.apply_loudness(analysis.loudness, version);
        }
```

`command.rs`, after `ApplyAnalysis`:

```rust
    /// Loudness normalization spec N6b: the loudness-only pass measured a
    /// track (`None`: nothing could be measured; it is not tried again
    /// until `version` changes). Ignored if the track is gone.
    ApplyLoudness {
        track: TrackId,
        loudness: Option<crate::loudness::LoudnessMeasurement>,
        version: u32,
    },
```

`reducer.rs`, after the `ApplyAnalysis` arm:

```rust
        Command::ApplyLoudness {
            track,
            loudness,
            version,
        } => {
            if let Some(t) = state.library.get_mut(track) {
                t.apply_loudness(loudness, version);
            }
        }
```

Then `cargo check --workspace --all-targets` and add `loudness: None, loudness_pass: None,` to each full `TrackAnalysis { … }` literal the compiler lists (files in **File Structure**). `Command` is matched exhaustively in a few places (`fp_model::availability::command_available`, `fp-remote`, `fp-app`): give `ApplyLoudness` the same treatment as `ApplyAnalysis` in each match the compiler reports.

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p fp-model`
Expected: PASS.

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add -A crates
git commit -m "feat(model): keep each track's loudness measurement

Tracks carry the integrated loudness and true peak the analysis measured,
and which loudness pass measured them (loudness normalization spec N6).
A result cached before loudness was measured keeps the track's value.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: Measure during the analysis; the cache tail (N4, N5, N6)

**Files:**
- Modify: `crates/fp-analysis/Cargo.toml` (`fp-dsp.workspace = true`)
- Modify: `crates/fp-analysis/src/analyze.rs` (decode loop L55-83, result L95-118; new fn)
- Modify: `crates/fp-analysis/src/cache.rs` (`CachedAnalysis`, `load_key`, `store_key`, new `update_loudness`)
- Modify: `crates/fp-analysis/src/lib.rs` (re-export)
- Test: `crates/fp-analysis/tests/analyze.rs`

**Interfaces:**
- Consumes: `fp_dsp::loudness::{LoudnessMeter, Loudness}` (plan 1), `fp_model::{LoudnessMeasurement, LOUDNESS_VERSION}` (Task 1).
- Produces:
  - `pub fn measure_loudness_cancellable(path: &Path, cancelled: &dyn Fn() -> bool) -> Result<Option<LoudnessMeasurement>, AnalysisError>` in `fp_analysis::analyze`, re-exported at the crate root.
  - `analyze_file_cancellable` results carry `analysis.loudness` and `analysis.loudness_pass = Some(LOUDNESS_VERSION)`.
  - `AnalysisCache::update_loudness(&self, key: &CacheKey, loudness: Option<LoudnessMeasurement>) -> std::io::Result<bool>` (`Ok(false)`: no entry under `key`).
  - `pub use fp_model::LOUDNESS_VERSION;` in `fp_analysis::cache`.

- [ ] **Step 1: Write the failing tests** (append to `crates/fp-analysis/tests/analyze.rs`; add `use fp_analysis::measure_loudness_cancellable; use fp_model::{LoudnessMeasurement, LOUDNESS_VERSION};`)

```rust
/// A stereo 1 kHz sine at `dbfs` (peak) for `secs`, 48 kHz, 16 bit.
fn stereo_tone(dir: &Path, name: &str, dbfs: f64, secs: f64) -> PathBuf {
    let path = dir.join(name);
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 48_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(&path, spec).unwrap();
    let amp = 10f64.powf(dbfs / 20.0);
    for i in 0..(secs * 48_000.0) as u64 {
        let x = amp * (std::f64::consts::TAU * 1_000.0 * i as f64 / 48_000.0).sin();
        let s = (x * f64::from(i16::MAX)) as i16;
        w.write_sample(s).unwrap();
        w.write_sample(s).unwrap();
    }
    w.finalize().unwrap();
    path
}

#[test]
fn n4_the_analysis_measures_the_loudness() {
    let dir = tempfile::tempdir().unwrap();
    let path = stereo_tone(dir.path(), "tone.wav", -23.0, 5.0);
    let a = analyze_file(&path, &settings(), &Limits::default()).unwrap().analysis;
    let l = a.loudness.unwrap();
    assert!((l.integrated_lufs.unwrap() + 23.0).abs() < 0.2, "{l:?}");
    assert!((l.true_peak_dbtp + 23.0).abs() < 0.5, "{l:?}");
    assert_eq!(a.loudness_pass, Some(LOUDNESS_VERSION));
    assert_eq!(ANALYSIS_VERSION, 8, "N6: the analysis version is not bumped");
}

#[test]
fn n4_the_loudness_only_measurement_matches_the_full_analysis() {
    let dir = tempfile::tempdir().unwrap();
    let path = stereo_tone(dir.path(), "tone.wav", -18.0, 3.0);
    let full = analyze_file(&path, &settings(), &Limits::default()).unwrap().analysis;
    let only = measure_loudness_cancellable(&path, &|| false).unwrap();
    assert_eq!(full.loudness, only);
}

#[test]
fn n5_silence_is_measured_without_integrated_loudness() {
    let dir = tempfile::tempdir().unwrap();
    let path = stereo_tone(dir.path(), "silence.wav", -200.0, 2.0);
    let l = measure_loudness_cancellable(&path, &|| false).unwrap().unwrap();
    assert_eq!(l.integrated_lufs, None);
}

#[test]
fn n5_a_file_shorter_than_400_ms_is_not_measured() {
    let dir = tempfile::tempdir().unwrap();
    let path = stereo_tone(dir.path(), "blip.wav", -10.0, 0.2);
    let a = analyze_file(&path, &settings(), &Limits::default()).unwrap().analysis;
    assert_eq!(a.loudness, None);
    assert_eq!(a.loudness_pass, Some(LOUDNESS_VERSION), "tried, so not retried");
    assert_eq!(measure_loudness_cancellable(&path, &|| false).unwrap(), None);
}

#[test]
fn n4_a_file_cut_short_is_measured_up_to_the_damage() {
    let dir = tempfile::tempdir().unwrap();
    let path = stereo_tone(dir.path(), "cut.wav", -20.0, 4.0);
    let bytes = std::fs::read(&path).unwrap();
    // Drop the last quarter of the data: the header still promises 4 s.
    std::fs::write(&path, &bytes[..bytes.len() * 3 / 4]).unwrap();
    let l = measure_loudness_cancellable(&path, &|| false).unwrap().unwrap();
    assert!((l.integrated_lufs.unwrap() + 20.0).abs() < 0.2, "{l:?}");
}

#[test]
fn the_loudness_measurement_can_be_cancelled() {
    let dir = tempfile::tempdir().unwrap();
    let path = stereo_tone(dir.path(), "tone.wav", -20.0, 2.0);
    assert_eq!(
        measure_loudness_cancellable(&path, &|| true),
        Err(AnalysisError::Cancelled)
    );
}

#[test]
fn n6_the_cache_round_trips_the_loudness() {
    let dir = tempfile::tempdir().unwrap();
    let path = stereo_tone(dir.path(), "tone.wav", -23.0, 2.0);
    let a = analyze_file(&path, &settings(), &Limits::default()).unwrap();
    let c = cache(dir.path());
    c.store(&path, &settings(), &a).unwrap();
    let back = c.load(&path, &settings()).unwrap();
    assert_eq!(back.analysis.loudness, a.analysis.loudness);
    assert_eq!(back.analysis.loudness_pass, Some(LOUDNESS_VERSION));
}

#[test]
fn an_entry_without_loudness_still_loads() {
    // What the previous release wrote: the entry with no tail record.
    let dir = tempfile::tempdir().unwrap();
    let path = fixture(dir.path(), "a.wav", 5);
    let mut a = analyze_file(&path, &settings(), &Limits::default()).unwrap();
    a.analysis.loudness = None;
    a.analysis.loudness_pass = None;
    let c = cache(dir.path());
    c.store(&path, &settings(), &a).unwrap();
    let back = c.load(&path, &settings()).unwrap();
    assert_eq!(back, a, "markers, peaks and cover intact");
    assert_eq!((back.analysis.loudness, back.analysis.loudness_pass), (None, None));
}

#[test]
fn n6b_update_loudness_fills_the_entry_in_place() {
    let dir = tempfile::tempdir().unwrap();
    let path = fixture(dir.path(), "a.wav", 5);
    let mut a = analyze_file(&path, &settings(), &Limits::default()).unwrap();
    a.analysis.loudness = None;
    a.analysis.loudness_pass = None;
    let c = cache(dir.path());
    c.store(&path, &settings(), &a).unwrap();
    let file = only_entry(dir.path());
    let before = std::fs::read(&file).unwrap();
    let key = c.key(&path, &settings()).unwrap();
    let m = LoudnessMeasurement {
        integrated_lufs: Some(-12.5),
        true_peak_dbtp: -0.7,
    };
    assert!(c.update_loudness(&key, Some(m)).unwrap());
    let after = std::fs::read(&file).unwrap();
    assert!(after.starts_with(&before), "the entry's own bytes do not change");
    let back = c.load_key(&key).unwrap();
    assert_eq!(back.analysis.loudness, Some(m));
    assert_eq!(back.analysis.loudness_pass, Some(LOUDNESS_VERSION));
    assert_eq!(back.peaks, a.peaks);
    assert_eq!(back.analysis.cue_in, a.analysis.cue_in);
}

#[test]
fn update_loudness_without_an_entry_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let path = fixture(dir.path(), "a.wav", 1);
    let c = cache(dir.path());
    let key = c.key(&path, &settings()).unwrap();
    assert!(!c.update_loudness(&key, None).unwrap());
    assert!(c.load_key(&key).is_none());
}

#[test]
fn a_broken_tail_is_ignored() {
    let dir = tempfile::tempdir().unwrap();
    let path = stereo_tone(dir.path(), "tone.wav", -23.0, 1.0);
    let a = analyze_file(&path, &settings(), &Limits::default()).unwrap();
    let c = cache(dir.path());
    c.store(&path, &settings(), &a).unwrap();
    let file = only_entry(dir.path());
    let mut bytes = std::fs::read(&file).unwrap();
    bytes.truncate(bytes.len() - 2); // cut into the tail
    bytes.push(0xff);
    std::fs::write(&file, &bytes).unwrap();
    let back = c.load(&path, &settings()).expect("the entry is kept");
    assert_eq!((back.analysis.loudness, back.analysis.loudness_pass), (None, None));
    assert_eq!(back.peaks, a.peaks);
}

/// The one `.bin` entry under the test cache.
fn only_entry(dir: &Path) -> PathBuf {
    let entries: Vec<PathBuf> = std::fs::read_dir(dir.join("cache"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "bin"))
        .collect();
    assert_eq!(entries.len(), 1);
    entries[0].clone()
}
```

`a_broken_tail_is_ignored` relies on the tail being the last bytes; if the measured tail is shorter than 3 bytes, cut 1 byte instead (it is about 11 bytes: two option tags, two `f32`, one varint).

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p fp-analysis --test analyze 2>&1 | tail -5`
Expected: FAIL to compile (`measure_loudness_cancellable`, `update_loudness` not found).

- [ ] **Step 3: Implement**

`crates/fp-analysis/Cargo.toml`, `[dependencies]`: `fp-dsp.workspace = true`.

`analyze.rs` — new imports: `use std::panic::{AssertUnwindSafe, catch_unwind};`, `use fp_dsp::loudness::LoudnessMeter;`, `use fp_model::{LOUDNESS_VERSION, LoudnessMeasurement};`. Add:

```rust
/// The loudness meter of one file, isolated from the rest of the analysis:
/// a panic inside it costs only the measurement (spec N5).
struct Meter {
    meter: Option<LoudnessMeter>,
    rate: u32,
    frames: u64,
}

impl Meter {
    fn new(rate: u32) -> Self {
        Self {
            meter: Some(LoudnessMeter::new(rate)),
            rate,
            frames: 0,
        }
    }

    fn push(&mut self, stereo: &[f32], path: &Path) {
        self.frames += (stereo.len() / 2) as u64;
        if let Some(m) = self.meter.as_mut()
            && catch_unwind(AssertUnwindSafe(|| m.push(stereo))).is_err()
        {
            tracing::warn!(path = %path.display(), "the loudness measurement crashed");
            self.meter = None;
        }
    }

    /// The measurement (N5): `None` for a file under one 400 ms block (no
    /// warning: short effects are expected), a crash or a value that is
    /// not finite (with a warning).
    fn finish(self, path: &Path) -> Option<LoudnessMeasurement> {
        let meter = self.meter?;
        let result = catch_unwind(AssertUnwindSafe(|| meter.finish())).ok().flatten();
        let measurement = result.and_then(|l| {
            let m = LoudnessMeasurement {
                integrated_lufs: l.integrated_lufs.map(|v| v as f32),
                true_peak_dbtp: l.true_peak_dbtp as f32,
            };
            let finite = m.integrated_lufs.is_none_or(f32::is_finite) && m.true_peak_dbtp.is_finite();
            finite.then_some(m)
        });
        if measurement.is_none() {
            let short = self.frames * 10 < u64::from(self.rate.max(1)) * 4;
            if short {
                tracing::debug!(path = %path.display(), "too short to measure its loudness");
            } else {
                tracing::warn!(path = %path.display(), "the loudness could not be measured");
            }
        }
        measurement
    }
}
```

In `analyze_file_cancellable`: after creating `builder`, `let mut meter = Meter::new(decoder.sample_rate());`; in `Ok(true)` also `meter.push(&block, path);`; after the loop `let loudness = meter.finish(path);`; in the `TrackAnalysis` literal add `loudness, loudness_pass: Some(LOUDNESS_VERSION),`.

New function (same decode loop and early-stop handling as the analysis, nothing else):

```rust
/// Loudness normalization spec N6b: decodes `path` once and measures only
/// its loudness (no envelope, markers, tags or cover). `Ok(None)` when
/// nothing could be measured (shorter than 400 ms, or a failure inside
/// the meter, logged).
pub fn measure_loudness_cancellable(
    path: &Path,
    cancelled: &dyn Fn() -> bool,
) -> Result<Option<LoudnessMeasurement>, AnalysisError> {
    match std::fs::metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Err(AnalysisError::Missing),
        _ => {}
    }
    let mut decoder = FileDecoder::open(path).map_err(AnalysisError::Unreadable)?;
    let mut meter = Meter::new(decoder.sample_rate());
    let mut block = Vec::new();
    let mut decoded_any = false;
    loop {
        if cancelled() {
            return Err(AnalysisError::Cancelled);
        }
        block.clear();
        match decoder.next_block(&mut block) {
            Ok(true) => {
                decoded_any = true;
                meter.push(&block, path);
            }
            Ok(false) => break,
            Err(e) if decoded_any => {
                tracing::warn!(path = %path.display(), "decoding stopped early: {e}");
                break;
            }
            Err(e) => return Err(AnalysisError::Unreadable(e)),
        }
    }
    Ok(meter.finish(path))
}
```

`lib.rs`: `pub use analyze::{Analysis, AnalysisError, analyze_file, analyze_file_cancellable, measure_loudness_cancellable};`.

`cache.rs` — module doc gains a paragraph: "Each file holds the entry (`CachedAnalysis`) and, after it, an optional `LoudnessTail`: the loudness measurement is not part of the entry's own bytes, so entries written before loudness was measured stay valid and are filled in place by the loudness-only pass (loudness normalization spec N6). Keep the tail last if `CachedAnalysis` ever changes." Then:

```rust
pub use fp_model::LOUDNESS_VERSION;
use fp_model::LoudnessMeasurement;

/// The loudness measurement stored after the entry.
#[derive(Serialize, Deserialize)]
struct LoudnessTail {
    loudness: Option<LoudnessMeasurement>,
    pass: u32,
}
```

`load_key`: replace the `postcard::from_bytes::<CachedAnalysis>(&bytes)` match with:

```rust
        match postcard::take_from_bytes::<CachedAnalysis>(&bytes) {
            Ok((cached, rest)) if cached.key == key.0 => {
                let mut analysis = cached.analysis;
                // A missing or unreadable tail is "not measured yet"; the
                // entry itself is fine.
                if let Ok(tail) = postcard::from_bytes::<LoudnessTail>(rest) {
                    analysis.analysis.loudness = tail.loudness;
                    analysis.analysis.loudness_pass = Some(tail.pass);
                }
                Some(analysis)
            }
            Ok(_) => None, // a hash collision: another file's entry
            Err(_) => {
                let _ = fs::remove_file(&file);
                None
            }
        }
```

`store_key`: after `let bytes = postcard::to_allocvec(&CachedAnalysis { … })…?;` make it `let mut bytes = …;` and append the tail:

```rust
        if let Some(pass) = analysis.analysis.loudness_pass {
            let tail = postcard::to_allocvec(&LoudnessTail {
                loudness: analysis.analysis.loudness,
                pass,
            })
            .map_err(io::Error::other)?;
            bytes.extend_from_slice(&tail);
        }
```

(the size check that follows then covers both). New method:

```rust
    /// N6b: records a loudness-only result in the entry under `key`, in
    /// place (the entry's own bytes are rewritten unchanged, the tail
    /// replaced) with the same atomic replace as `store_key`. `Ok(false)`
    /// when there is no entry: the file's full analysis will measure it.
    pub fn update_loudness(
        &self,
        key: &CacheKey,
        loudness: Option<LoudnessMeasurement>,
    ) -> io::Result<bool> {
        let Some(mut analysis) = self.load_key(key) else {
            return Ok(false);
        };
        analysis.analysis.loudness = loudness;
        analysis.analysis.loudness_pass = Some(LOUDNESS_VERSION);
        self.store_key(key, &analysis)?;
        Ok(true)
    }
```

Postcard encodes a struct deterministically from its values, so rewriting the same `CachedAnalysis` gives the same leading bytes (`n6b_update_loudness_fills_the_entry_in_place` checks it).

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p fp-analysis`
Expected: PASS (existing cache tests included: `cache_round_trips` now round-trips the measurement too).

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add -A crates/fp-analysis Cargo.lock
git commit -m "feat(analysis): measure each file's loudness and cache it

The analysis feeds the decoded audio to the BS.1770 loudness meter as
well (loudness normalization spec N4, N5). The cache stores the result
after the entry, so entries written before stay valid and the library
is not analysed again (N6).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: The loudness lane of the analysis pool (N6b)

**Files:**
- Modify: `crates/fp-analysis/src/analyzer.rs`
- Test: `crates/fp-analysis/tests/loudness_pass.rs` (new)

**Interfaces:**
- Consumes: `measure_loudness_cancellable`, `AnalysisCache::update_loudness` (Task 2).
- Produces (in `fp_analysis::analyzer`):
  - `pub type LoudnessFn = Arc<dyn Fn(&Path, &dyn Fn() -> bool) -> Result<Option<LoudnessMeasurement>, AnalysisError> + Send + Sync>;`
  - `#[derive(Debug)] pub struct LoudnessResult { pub track: TrackId, pub path: PathBuf, pub outcome: Result<Option<LoudnessMeasurement>, AnalysisError> }`
  - `Analyzer::with_fns(threads: usize, settings: AnalysisSettings, limits: Limits, cache: Option<AnalysisCache>, analyze: AnalyzeFn, loudness: LoudnessFn) -> std::io::Result<Self>` (`spawn` and `with_analyze_fn` call it with `Arc::new(measure_loudness_cancellable)`)
  - `Analyzer::submit_loudness(&self, track: TrackId, path: PathBuf)` (ignored while one is pending for `track`)
  - `Analyzer::cancel_loudness(&self, track: TrackId)`
  - `Analyzer::loudness_results(&self) -> &Receiver<LoudnessResult>`
  - `Analyzer::loudness_share(&self) -> usize` = `threads.saturating_sub(1).max(1)` (the number of workers spawned)

- [ ] **Step 1: Write the failing tests**

`crates/fp-analysis/tests/loudness_pass.rs`:

```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The loudness-only pass in the analysis pool (loudness normalization
//! spec N6b).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use fp_analysis::analyzer::{AnalyzeFn, Analyzer, LoudnessFn};
use fp_analysis::cache::AnalysisCache;
use fp_analysis::{AnalysisError, analyze_file, measure_loudness_cancellable};
use fp_model::{AnalysisSettings, LOUDNESS_VERSION, Limits, TrackId};

const WAIT: Duration = Duration::from_secs(10);

fn wav(dir: &Path, name: &str, secs: u32) -> PathBuf {
    let path = dir.join(name);
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 8_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(&path, spec).unwrap();
    for i in 0..8_000 * secs {
        w.write_sample(((i as f32 * 0.3).sin() * 10_000.0) as i16).unwrap();
    }
    w.finalize().unwrap();
    path
}

fn real_analyze() -> AnalyzeFn {
    Arc::new(|path, settings, limits, cancelled| {
        fp_analysis::analyze_file_cancellable(path, settings, limits, cancelled)
    })
}

fn real_loudness() -> LoudnessFn {
    Arc::new(|path, cancelled| measure_loudness_cancellable(path, cancelled))
}

#[test]
fn a_loudness_job_measures_and_fills_the_cache_entry_in_place() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "a.wav", 2);
    let cache_dir = dir.path().join("cache");
    let settings = AnalysisSettings::default();
    // An entry as the previous release wrote it: no loudness.
    let old = AnalysisCache::new(cache_dir.clone(), &Limits::default());
    let mut a = analyze_file(&path, &settings, &Limits::default()).unwrap();
    a.analysis.loudness = None;
    a.analysis.loudness_pass = None;
    old.store(&path, &settings, &a).unwrap();
    let analyzer = Analyzer::with_fns(
        2,
        settings.clone(),
        Limits::default(),
        Some(AnalysisCache::new(cache_dir.clone(), &Limits::default())),
        real_analyze(),
        real_loudness(),
    )
    .unwrap();
    analyzer.submit_loudness(TrackId(1), path.clone());
    let r = analyzer.loudness_results().recv_timeout(WAIT).unwrap();
    assert_eq!(r.track, TrackId(1));
    let measured = r.outcome.unwrap();
    assert!(measured.is_some());
    let back = old.load(&path, &settings).unwrap();
    assert_eq!(back.analysis.loudness, measured);
    assert_eq!(back.analysis.loudness_pass, Some(LOUDNESS_VERSION));
    assert_eq!(back.peaks, a.peaks, "markers and peaks untouched");
    assert_eq!(back.analysis.segue_start, a.analysis.segue_start);
    assert!(analyzer.results().try_recv().is_err(), "no full result");
}

#[test]
fn a_new_normal_job_goes_ahead_of_waiting_loudness_jobs() {
    let dir = tempfile::tempdir().unwrap();
    let file = wav(dir.path(), "a.wav", 1);
    let gate = Arc::new(AtomicBool::new(false));
    let order = Arc::new(Mutex::new(Vec::<&'static str>::new()));
    let (g, o) = (gate.clone(), order.clone());
    let analyze: AnalyzeFn = Arc::new(move |path, settings, limits, _| {
        while !g.load(Ordering::Acquire) {
            std::thread::sleep(Duration::from_millis(1));
        }
        o.lock().unwrap().push("normal");
        analyze_file(path, settings, limits)
    });
    let o = order.clone();
    let loudness: LoudnessFn = Arc::new(move |path, cancelled| {
        o.lock().unwrap().push("loudness");
        measure_loudness_cancellable(path, cancelled)
    });
    let analyzer = Analyzer::with_fns(
        1,
        AnalysisSettings::default(),
        Limits::default(),
        None,
        analyze,
        loudness,
    )
    .unwrap();
    // The only worker is busy with a normal job…
    analyzer.submit(TrackId(1), file.clone());
    std::thread::sleep(Duration::from_millis(50));
    // …while two loudness jobs and then a normal one are queued.
    analyzer.submit_loudness(TrackId(2), file.clone());
    analyzer.submit_loudness(TrackId(3), file.clone());
    analyzer.submit(TrackId(4), file.clone());
    gate.store(true, Ordering::Release);
    for _ in 0..2 {
        analyzer.results().recv_timeout(WAIT).unwrap();
    }
    for _ in 0..2 {
        analyzer.loudness_results().recv_timeout(WAIT).unwrap();
    }
    assert_eq!(
        *order.lock().unwrap(),
        vec!["normal", "normal", "loudness", "loudness"]
    );
}

#[test]
fn a_normal_and_a_loudness_job_for_the_same_track_both_run() {
    let dir = tempfile::tempdir().unwrap();
    let file = wav(dir.path(), "a.wav", 1);
    let analyzer =
        Analyzer::spawn(2, AnalysisSettings::default(), Limits::default(), None).unwrap();
    analyzer.submit_loudness(TrackId(1), file.clone());
    analyzer.submit(TrackId(1), file);
    assert!(analyzer.results().recv_timeout(WAIT).is_ok());
    assert!(analyzer.loudness_results().recv_timeout(WAIT).is_ok());
}

#[test]
fn a_cancelled_loudness_job_delivers_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let file = wav(dir.path(), "a.wav", 1);
    let started = Arc::new(AtomicBool::new(false));
    let s = started.clone();
    let loudness: LoudnessFn = Arc::new(move |_, cancelled| {
        s.store(true, Ordering::Release);
        while !cancelled() {
            std::thread::sleep(Duration::from_millis(1));
        }
        Err(AnalysisError::Cancelled)
    });
    let analyzer = Analyzer::with_fns(
        1,
        AnalysisSettings::default(),
        Limits::default(),
        None,
        real_analyze(),
        loudness,
    )
    .unwrap();
    analyzer.submit_loudness(TrackId(1), file);
    while !started.load(Ordering::Acquire) {
        std::thread::sleep(Duration::from_millis(1));
    }
    analyzer.cancel_loudness(TrackId(1));
    assert!(analyzer
        .loudness_results()
        .recv_timeout(Duration::from_millis(300))
        .is_err());
}

#[test]
fn a_crash_in_the_loudness_job_is_reported_as_unreadable() {
    let dir = tempfile::tempdir().unwrap();
    let file = wav(dir.path(), "a.wav", 1);
    #[allow(clippy::panic)]
    let loudness: LoudnessFn = Arc::new(|_, _| panic!("injected"));
    let analyzer = Analyzer::with_fns(
        1,
        AnalysisSettings::default(),
        Limits::default(),
        None,
        real_analyze(),
        loudness,
    )
    .unwrap();
    analyzer.submit_loudness(TrackId(1), file);
    let r = analyzer.loudness_results().recv_timeout(WAIT).unwrap();
    assert!(matches!(r.outcome, Err(AnalysisError::Unreadable(_))));
}

#[test]
fn the_loudness_share_leaves_one_worker_free() {
    for (threads, share) in [(1, 1), (2, 1), (3, 2), (4, 3)] {
        let a = Analyzer::spawn(threads, AnalysisSettings::default(), Limits::default(), None)
            .unwrap();
        assert_eq!(a.loudness_share(), share, "{threads} threads");
    }
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p fp-analysis --test loudness_pass 2>&1 | tail -5`
Expected: FAIL to compile (`LoudnessFn`, `with_fns`, … not found).

- [ ] **Step 3: Implement** (all in `analyzer.rs`; extend the module doc with "Loudness-only jobs (loudness normalization spec N6b) have a third queue, taken last, with their own bookkeeping and results.")

1. Types: add `LoudnessFn` and `LoudnessResult` as in **Interfaces**; `use fp_model::LoudnessMeasurement;` and `use crate::analyze::measure_loudness_cancellable;`.
2. `Shared` gains `loudness_book: Mutex<Bookkeeping>` and `loudness: LoudnessFn`.
3. `Analyzer` gains `loudness_jobs: Option<Sender<Job>>`, `loudness_results: Receiver<LoudnessResult>`, `share: usize`.
4. `with_analyze_fn(…, analyze)` becomes a call to `with_fns(…, analyze, Arc::new(measure_loudness_cancellable))`; `with_fns` holds today's body plus a third unbounded channel `(loudness_tx, loudness_rx)`, a results channel `(lres_tx, lres_rx)`, `Lanes { urgent, normal, loudness: loudness_rx.clone() }`, and passes `lres_tx.clone()` to each `worker`. `share = threads.max(1).saturating_sub(1).max(1)`.
5. `Drop` also sets `self.loudness_jobs = None` before joining.
6. Lanes:

```rust
/// Which queue a job came from.
#[derive(Clone, Copy)]
enum Lane {
    Analysis,
    Loudness,
}

impl Lanes {
    /// The next job: urgent first, then normal, then loudness-only
    /// (N6b: below every normal job); `None` once the analyzer is gone.
    fn next(&self) -> Option<(Lane, Job)> {
        if let Ok(job) = self.urgent.try_recv() {
            return Some((Lane::Analysis, job));
        }
        if let Ok(job) = self.normal.try_recv() {
            return Some((Lane::Analysis, job));
        }
        crossbeam_channel::select_biased! {
            recv(self.urgent) -> job => job.ok().map(|j| (Lane::Analysis, j)),
            recv(self.normal) -> job => job.ok().map(|j| (Lane::Analysis, j)),
            recv(self.loudness) -> job => job.ok().map(|j| (Lane::Loudness, j)),
        }
    }
}
```

7. `worker` takes `loudness_results: &Sender<LoudnessResult>`; its loop becomes `while let Some((lane, job)) = lanes.next()`, checks `stop`, and dispatches: `Lane::Analysis` → today's body (moved into `fn analysis_job(job: Job, results: &Sender<AnalysisResult>, shared: &Shared)`), `Lane::Loudness` → `loudness_job`:

```rust
fn loudness_job(job: Job, results: &Sender<LoudnessResult>, shared: &Shared) {
    {
        let mut book = lock(&shared.loudness_book);
        if !book.is_current(&job) {
            return;
        }
        book.running.insert(job.track, job.generation);
    }
    let outcome = catch_unwind(AssertUnwindSafe(|| run_loudness(&job, shared))).unwrap_or_else(
        |_| Err(AnalysisError::Unreadable("the loudness measurement crashed".to_owned())),
    );
    let deliver = {
        let mut book = lock(&shared.loudness_book);
        if book.running.get(&job.track) == Some(&job.generation) {
            book.running.remove(&job.track);
        }
        let current = book.is_current(&job);
        if current {
            book.latest.remove(&job.track);
        }
        current
    };
    if deliver && !matches!(outcome, Err(AnalysisError::Cancelled)) {
        let _ = results.send(LoudnessResult {
            track: job.track,
            path: job.path,
            outcome,
        });
    }
}

fn run_loudness(
    job: &Job,
    shared: &Shared,
) -> Result<Option<LoudnessMeasurement>, AnalysisError> {
    let settings = lock(&shared.settings).clone();
    // The key is taken first: a file that changes meanwhile is not
    // recorded under its new key.
    let key = shared.cache.as_ref().and_then(|c| c.key(&job.path, &settings));
    let cancelled =
        || shared.stop.load(Ordering::Acquire) || !lock(&shared.loudness_book).is_current(job);
    let outcome = (shared.loudness)(&job.path, &cancelled);
    // A measurement, or a file that cannot be measured, is recorded in the
    // file's entry so it is not measured again (N6b).
    let recorded = match &outcome {
        Ok(m) => Some(*m),
        Err(AnalysisError::Unreadable(_)) => Some(None),
        Err(_) => None,
    };
    if let (Some(loudness), Some(cache), Some(key)) = (recorded, &shared.cache, &key)
        && cache.key(&job.path, &settings).as_ref() == Some(key)
        && let Err(e) = cache.update_loudness(key, loudness)
    {
        tracing::warn!(path = %job.path.display(), "cannot cache the loudness: {e}");
    }
    outcome
}
```

8. Public methods:

```rust
    /// N6b: queues a loudness-only measurement of `path` as `track`,
    /// behind every normal job (ignored while one is pending for `track`).
    pub fn submit_loudness(&self, track: TrackId, path: PathBuf) {
        let generation = {
            let mut book = lock(&self.shared.loudness_book);
            if book.latest.contains_key(&track) {
                return;
            }
            book.submit(track)
        };
        if let Some(jobs) = &self.loudness_jobs {
            let _ = jobs.send(Job { track, path, generation });
        }
    }

    /// Cancels a pending or running loudness-only job.
    pub fn cancel_loudness(&self, track: TrackId) {
        lock(&self.shared.loudness_book).latest.remove(&track);
    }

    pub fn loudness_results(&self) -> &Receiver<LoudnessResult> {
        &self.loudness_results
    }

    /// How many loudness-only jobs may run at once: all workers but one,
    /// at least one (N6b), so new files and the operator's requests keep
    /// a worker.
    pub fn loudness_share(&self) -> usize {
        self.share
    }
```

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p fp-analysis`
Expected: PASS (the existing `analyzer.rs` tests unchanged).

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-analysis
git commit -m "feat(analysis): run loudness-only jobs below every analysis

The pool measures the loudness of tracks analysed before loudness was
measured, on a queue it takes last, and records the result in the
file's cache entry in place (loudness normalization spec N6b).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: The loudness-only pass in the services thread (N6b)

**Files:**
- Modify: `crates/fp-app/src/services.rs` (fields ~L320-390, `new` ~L396-452, `analysis_step` ~L539-581, new `fn loudness_pass`)
- Test: `crates/fp-app/tests/services.rs` (the rig gets a counting loudness function)

**Interfaces:**
- Consumes: `Analyzer::{submit_loudness, cancel_loudness, loudness_results, loudness_share}`, `LoudnessResult` (Task 3); `Track::needs_loudness_pass`, `Command::ApplyLoudness`, `LOUDNESS_VERSION` (Task 1).
- Produces: nothing new for later plans (the model keeps `Track::loudness` up to date).

- [ ] **Step 1: Write the failing tests**

In `crates/fp-app/tests/services.rs`:

1. Add to `Rig`: `loudness_runs: Arc<AtomicUsize>` and `finished: Arc<AtomicUsize>` (analyses completed), and `seen_finished: Arc<AtomicUsize>` (the value of `finished` when the last loudness job started).
2. In `rig_hooked`, increment `finished` after `analyze_file_cancellable` returns (inside `counting`), build a loudness function and switch the pool to `with_fns`:

```rust
    let finished = Arc::new(AtomicUsize::new(0));
    let seen_finished = Arc::new(AtomicUsize::new(0));
    let loudness_runs = Arc::new(AtomicUsize::new(0));
    let (runs, seen, done) = (loudness_runs.clone(), seen_finished.clone(), finished.clone());
    let loudness: fp_analysis::analyzer::LoudnessFn = Arc::new(move |path, cancelled| {
        runs.fetch_add(1, Ordering::SeqCst);
        seen.store(done.load(Ordering::SeqCst), Ordering::SeqCst);
        fp_analysis::measure_loudness_cancellable(path, cancelled)
    });
    let analyzer = Analyzer::with_fns(
        1,
        Default::default(),
        Default::default(),
        None,
        counting,
        loudness,
    )
    .unwrap();
```

3. Helper and tests:

```rust
/// Points every player at a new, empty playlist, so that no track is on
/// screen: a track a player shows gets a full analysis for its waveform
/// (`Services::wanted`), which would measure it before the pass could.
fn off_screen(state: &mut fp_model::AppState) {
    fp_model::apply(state, Command::CreatePlaylist { name: "Other".into() }).unwrap();
    let other = state.playlists.iter().last().unwrap().id;
    for p in &mut state.players {
        p.playlist = other;
        p.next = None;
        p.next_explicit = false;
    }
}

/// Marks every track as analysed by the previous release: markers and
/// length known, no loudness measured yet; no track on screen.
fn analysed_before_loudness(state: &mut fp_model::AppState) {
    off_screen(state);
    for t in state.library.iter_mut() {
        t.analyzed = true;
        t.analysis_version = fp_analysis::cache::ANALYSIS_VERSION;
        t.tags_read = true;
        t.duration_secs = 2.0;
    }
}

#[test]
fn n6b_old_tracks_are_measured_once_without_a_full_analysis() {
    let dir = tempfile::tempdir().unwrap();
    let a = wav(dir.path(), "a.wav", 2);
    let mut r = rig_with(&[a], dir, Duration::ZERO, analysed_before_loudness);
    r.run_until("the loudness", |r| only_track(r).loudness_pass.is_some());
    let t = only_track(&r);
    assert!(t.loudness.and_then(|l| l.integrated_lufs).is_some());
    let until = r.now + Duration::from_millis(500);
    r.run_until("a quiet period", |r| r.now >= until);
    assert_eq!(r.analyses.load(Ordering::SeqCst), 0, "no full analysis");
    assert_eq!(r.loudness_runs.load(Ordering::SeqCst), 1, "measured once");
}

#[test]
fn n6b_a_new_file_is_measured_by_its_analysis_not_by_the_pass() {
    let dir = tempfile::tempdir().unwrap();
    let a = wav(dir.path(), "a.wav", 2);
    let mut r = rig(&[a], dir);
    r.run_until("the analysis", |r| only_track(r).analyzed);
    let until = r.now + Duration::from_millis(500);
    r.run_until("a quiet period", |r| r.now >= until);
    assert!(only_track(&r).loudness.is_some());
    assert_eq!(r.loudness_runs.load(Ordering::SeqCst), 0);
}

#[test]
fn the_pass_waits_for_normal_analysis_and_keeps_its_share() {
    let dir = tempfile::tempdir().unwrap();
    let old = wav(dir.path(), "old.wav", 1);
    let new = wav(dir.path(), "new.wav", 1);
    // `old` was analysed by the previous release, `new` was just added.
    let mut r = rig_with(&[old.clone(), new], dir, Duration::from_millis(300), |s| {
        off_screen(s);
        for t in s.library.iter_mut().filter(|t| t.path == old) {
            t.analyzed = true;
            t.analysis_version = fp_analysis::cache::ANALYSIS_VERSION;
            t.tags_read = true;
        }
    });
    r.run_until("both measured", |r| {
        r.handle.model.load().library.iter().all(|t| t.loudness_pass.is_some())
    });
    assert_eq!(
        r.seen_finished.load(Ordering::SeqCst),
        1,
        "the loudness job started after the new file's analysis"
    );
}

#[test]
fn n6b_a_short_file_is_recorded_and_not_measured_again() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("blip.wav");
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 48_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(&path, spec).unwrap();
    for i in 0..9_600 {
        w.write_sample(((i as f32 * 0.05).sin() * 8_000.0) as i16).unwrap();
    }
    w.finalize().unwrap();
    let mut r = rig_with(&[path], dir, Duration::ZERO, analysed_before_loudness);
    r.run_until("the attempt", |r| only_track(r).loudness_pass.is_some());
    assert_eq!(only_track(&r).loudness, None);
    let until = r.now + Duration::from_millis(500);
    r.run_until("a quiet period", |r| r.now >= until);
    assert_eq!(r.loudness_runs.load(Ordering::SeqCst), 1);
}

#[test]
fn n6b_a_newer_loudness_version_measures_again() {
    let dir = tempfile::tempdir().unwrap();
    let a = wav(dir.path(), "a.wav", 1);
    let mut r = rig_with(&[a], dir, Duration::ZERO, |s| {
        analysed_before_loudness(s);
        for t in s.library.iter_mut() {
            t.apply_loudness(None, fp_model::LOUDNESS_VERSION - 1);
        }
    });
    r.run_until("the new pass", |r| {
        only_track(r).loudness_pass == Some(fp_model::LOUDNESS_VERSION)
    });
    assert_eq!(r.loudness_runs.load(Ordering::SeqCst), 1);
}

#[test]
fn a_track_removed_while_measured_is_forgotten() {
    let dir = tempfile::tempdir().unwrap();
    let a = wav(dir.path(), "a.wav", 1);
    let mut r = rig_with(&[a], dir, Duration::ZERO, analysed_before_loudness);
    let entry = r.handle.model.load().playlists.iter().next().unwrap().entries[0].id;
    r.handle.send(Command::RemoveEntry(entry));
    r.run_until("the track gone", |r| r.handle.model.load().library.is_empty());
    let until = r.now + Duration::from_millis(300);
    r.run_until("a quiet period", |r| r.now >= until);
    assert!(r.handle.model.load().library.is_empty());
}
```

(N6b "it runs with `normalize` off" is covered by every test above: `normalize` is off by default and does not exist until plan 3; plan 3 adds `the_pass_runs_with_normalization_off` once the field exists.)

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p fp-app --features test-hooks --test services n6b 2>&1 | tail -5`
Expected: the new tests FAIL (time out waiting for "the loudness").

- [ ] **Step 3: Implement**

`services.rs`:

1. Imports: `use fp_analysis::analyzer::{AnalysisResult, Analyzer, LoudnessResult};`, `fp_model::LOUDNESS_VERSION`.
2. Fields (after `tags_answered`):

```rust
    /// Loudness-only jobs queued and not answered yet (loudness
    /// normalization spec N6b).
    loudness_in_flight: HashSet<TrackId>,
    /// Tracks whose loudness answer was sent to the model but whose
    /// snapshot does not show it yet: not queued twice meanwhile.
    loudness_answered: HashSet<TrackId>,
```

initialised empty in `new`.

3. In `analysis_step`, after `self.tag_pass(state);`: `self.loudness_pass(state);`.
4. The pass:

```rust
    /// N6b: tracks analysed before loudness was measured (or by an older
    /// `LOUDNESS_VERSION`) get a loudness-only measurement, below every
    /// normal analysis: nothing is queued while a normal job is in flight,
    /// and never more than the pool's share at once. Failures and files
    /// too short to measure are recorded too, so they are not retried.
    fn loudness_pass(&mut self, state: &AppState) {
        let answers: Vec<LoudnessResult> = self.analyzer.loudness_results().try_iter().collect();
        for answer in answers {
            self.loudness_in_flight.remove(&answer.track);
            let loudness = match answer.outcome {
                Ok(loudness) => loudness,
                Err(AnalysisError::Cancelled) => continue,
                Err(e) => {
                    tracing::warn!(path = %answer.path.display(), "cannot measure the loudness: {e}");
                    None
                }
            };
            if self.conductor.send(Command::ApplyLoudness {
                track: answer.track,
                loudness,
                version: LOUDNESS_VERSION,
            }) {
                self.loudness_answered.insert(answer.track);
            }
        }
        let gone: Vec<TrackId> = self
            .loudness_in_flight
            .iter()
            .copied()
            .filter(|id| state.library.get(*id).is_none_or(|t| !t.needs_loudness_pass()))
            .collect();
        for id in gone {
            self.analyzer.cancel_loudness(id);
            self.loudness_in_flight.remove(&id);
        }
        self.loudness_answered
            .retain(|id| state.library.get(*id).is_some_and(|t| t.needs_loudness_pass()));
        // Below every normal job: only once the normal queue is empty.
        if !self.in_flight.is_empty() {
            return;
        }
        let room = self
            .analyzer
            .loudness_share()
            .saturating_sub(self.loudness_in_flight.len());
        let todo: Vec<(TrackId, std::path::PathBuf)> = state
            .library
            .iter()
            .filter(|t| {
                t.needs_loudness_pass()
                    && !self.loudness_in_flight.contains(&t.id)
                    && !self.loudness_answered.contains(&t.id)
            })
            .take(room)
            .map(|t| (t.id, t.path.clone()))
            .collect();
        for (id, path) in todo {
            self.analyzer.submit_loudness(id, path);
            self.loudness_in_flight.insert(id);
        }
    }
```

A full analysis that is queued for a track (`in_flight`) also measures it, so `in_flight` non-empty is exactly "the normal queue is not empty".

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p fp-app --features test-hooks --test services`
Expected: PASS (old tests included: tracks analysed in a test get their measurement from the analysis, so no loudness job runs and the counts they assert do not change).

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-app
git commit -m "feat(app): measure the loudness of tracks analysed before

The services thread queues a loudness-only job for every track that has
no measurement from the current loudness pass, after every normal
analysis and within the pool's share (loudness normalization spec N6b).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: Technical documentation and the main spec

**Files:**
- Modify: `docs/technical/analysis.md` (the measurement, the loudness-only pass, `LOUDNESS_VERSION`, `ANALYSIS_VERSION` unchanged, the cache tail)
- Modify: `docs/technical/persistence.md` (`Track::loudness`, `Track::loudness_pass` in `library.json`; the cache file layout)
- Modify: `docs/superpowers/specs/2026-09-25-fauste-player-design.md` §6: add "Analysis also measures integrated loudness and true peak (loudness normalization spec N4–N6); tracks without a measurement get a background loudness-only pass (N6b)."

- [ ] **Step 1: `analysis.md`** — add a section "Loudness" with: what is measured (BS.1770 integrated loudness and true peak of the stereo downmix at the file's rate, via `fp_dsp::loudness::LoudnessMeter`, during the same decode as the envelope); the outcomes (measured; silent: no integrated value; under 400 ms or a failure: no measurement, recorded as tried); `LOUDNESS_VERSION` (in `fp-model`, 1) versus `ANALYSIS_VERSION` (8, unchanged by this feature: an entry is never outdated because of loudness); the loudness-only pass (third queue of the pool, taken last; the services thread queues jobs only while no normal analysis is in flight and at most `threads − 1` at once, at least one; each answer becomes `Command::ApplyLoudness`; the result is written in place into the file's cache entry; it runs whether normalization is on or off); the cache file layout (entry, then the optional `LoudnessTail`, why, and "keep it last").
- [ ] **Step 2: `persistence.md`** — in the `library.json` track fields: `loudness` (`{ integrated_lufs: number|null, true_peak_dbtp: number }` or absent) and `loudness_pass` (number or absent); both optional, older files load without them. In the analysis cache paragraph: the tail record.
- [ ] **Step 3: main spec §6** as above.
- [ ] **Step 4: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add docs
git commit -m "docs: describe the loudness measurement and its pass

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

## Self-review notes

- Spec coverage: N4 → Task 2; N5 (silence, short, non-finite, panic) → Tasks 2 (and plan 1 for the meter itself); N6 (fields, `ANALYSIS_VERSION` 8, old entries, `apply_analysis`) → Tasks 1–2; N6b (priority, share, work, storage, failure recording, independent of `normalize`) → Tasks 3–4; N22's count `loudness_unmeasured` → Task 1 (shown by plan 5); §11 analysis and persistence docs → Task 5; §13 main spec §6 → Task 5.
- Deferred to plan 3: `Track::loudness_edit`, the gain, and N12's re-preload when a measurement arrives (`reconcile` compares normalizations, so `ApplyLoudness` needs no extra code there).
