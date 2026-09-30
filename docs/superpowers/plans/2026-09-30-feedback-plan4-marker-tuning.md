# Feedback Plan 4 — Real-Music Corpus and Marker Tuning Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:**
- Trimming never removes real audio.
- Automatic segues give short overlaps (F16).
- A git-ignored folder of real music backs local, opt-in tests and a report
  for tuning (F12).

**Architecture:**
- `fp-analysis`:
  - the envelope gains a per-bucket stereo peak;
  - `detect_markers` trims on it, with a margin;
  - the segue level becomes relative to the median RMS of the body;
  - the analysis version is bumped.
- `fp-model` (`AnalysisSettings`):
  - drops `silence_threshold_db` and `segue_threshold_db`;
  - gains `trim_threshold_db`, `trim_margin_ms` and `segue_drop_db`;
  - `segue_max_secs` defaults to 4.
- `fp-store` reports stored fields this version no longer knows.
- Settings → Analysis shows the new fields.
- A corpus helper, `#[ignore]` tests and a `marker_report` example read
  `test-music/` or `FAUSTE_TEST_MUSIC`.

**Tech Stack:** Rust 2024, symphonia (through `fp-decode`), serde_json.

**Spec:** [`docs/superpowers/specs/2026-09-30-operator-feedback-design.md`](../specs/2026-09-30-operator-feedback-design.md) §4. Roadmap: plan 4.

## Global Constraints

- New config fields have a default, a range in `Config::validate` and
  lenient loading:
  - `analysis.trim_threshold_db`: −60, range −120 … −20;
  - `analysis.trim_margin_ms`: 20, range 0 … 1000;
  - `analysis.segue_drop_db`: 15, range 3 … 40;
  - `analysis.segue_max_secs`: default 4, range unchanged (0 … 60).
- `fp-analysis` denies `clippy::indexing_slicing`: use `get`.
- Corpus tests are `#[ignore]` and never run in CI. With no corpus they
  print a note and pass.
- The corpus folder is `test-music/` at the repository root. It is
  git-ignored except `test-music/README.md`, and `FAUSTE_TEST_MUSIC`
  overrides it.
- English code and docs. UI strings go in both locales. Conventional
  Commits (`analysis`, `model`, `store`, `ui`, `docs`) with the trailer
  `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Branch: `feat/marker-tuning`.

## Review Focus

- **A signal on one channel only, or in antiphase.** The mono mix would
  miss it, so trimming uses the stereo peak (Task 1 test).
- **Clicks and soft fades at the edges.** Anything at or above the threshold
  stays inside `[cue_in, cue_out]` (Task 1 tests).
- **Loud and quiet masters.** The same shape of fade-out gives the same
  overlap (Task 2 test).
- **A `config.json` with the old fields.** They are ignored with a warning,
  and the rest of the config loads (Task 3 test).
- **Manual markers survive** the version bump (Task 3 test).

---

### Task 1: Trim on bucket peaks

**Files:** `crates/fp-analysis/src/signal.rs` (`EnvelopeBuilder`, `Envelope.peak_db`, `detect_markers`), `crates/fp-model/src/config.rs` (`AnalysisSettings`), `crates/fp-app/tests/services.rs:233`, `crates/fp-analysis/tests/analyze.rs:118`.

**Interfaces:**
- Produces:
  - `Envelope.peak_db: Vec<f32>`: per bucket, the largest absolute sample of
    either channel, in dBFS;
  - `AnalysisSettings.trim_threshold_db: f64`;
  - `AnalysisSettings.trim_margin_ms: u32`.

- [ ] **Step 1: Failing tests** (in `signal.rs`'s test module):
  - `trimming_keeps_every_bucket_at_or_above_the_threshold`: 1 s of
    silence, a −30 dBFS click of 2 ms at 0.5 s, 1 s of silence, 5 s of tone,
    a 3 s fade from −20 to −50 dBFS, then 1 s of silence. Assert that every
    bucket whose `peak_db` ≥ −60 lies inside `[cue_in, cue_out]`, and that
    `cue_in` ≤ 0.5 s.
  - `a_soft_fade_in_is_not_cut`: a 4 s fade-in from −70 to −20 dBFS. The
    point where the level crosses −60 dBFS must be at or after `cue_in`.
  - `one_channel_or_antiphase_audio_is_not_trimmed_away`: a tone on L only,
    then a tone with R = −L. `cue_in` and `cue_out` cover both parts.
  - `the_margin_moves_cue_in_back_and_cue_out_forward`, clamped at 0 and at
    the duration.
  - Update the existing silent-file test: the whole duration is kept.

- [ ] **Step 2: Run** `cargo test -p fp-analysis`. Expected: compile errors (`peak_db`, `trim_threshold_db`).

- [ ] **Step 3: Implement.**
  - `EnvelopeBuilder` tracks `abs_max = max(|l|, |r|)` per bucket and pushes
    `to_db(abs_max)` into `peak_db` in `close_bucket`.
  - `AnalysisSettings`: remove `silence_threshold_db`, and add
    `trim_threshold_db: -60.0` and `trim_margin_ms: 20`. In `validate`,
    clamp them to −120…−20 and 0…1000.
  - `detect_markers`:

```rust
    let threshold = s.trim_threshold_db as f32;
    let first = env.peak_db.iter().position(|db| *db >= threshold);
    let last = env.peak_db.iter().rposition(|db| *db >= threshold);
    let (Some(first), Some(last)) = (first, last) else { /* whole file, no markers */ };
    let margin = f64::from(s.trim_margin_ms) / 1000.0;
    let cue_in = (env.bucket_start(first) - margin).max(0.0);
    let cue_out = (env.bucket_end(last) + margin).min(env.duration_secs);
```

    Here `bucket_start`/`bucket_end` mirror `window_start`/`window_end` with
    `bucket_secs`. The RMS body used by the segue and the outro is the RMS
    windows that overlap `[cue_in, cue_out]`: `first_w = (cue_in /
    window_secs) as usize` and `last_w = ((cue_out / window_secs).ceil() as
    usize).saturating_sub(1)`, clamped to the last window.

- [ ] **Step 4: Run** `cargo test -p fp-analysis -p fp-model`. Expected: PASS. Fix the two test files that set `silence_threshold_db`/`segue_threshold_db` (use the new fields).

- [ ] **Step 5: Commit:** `fix(analysis): never trim audio above the trim threshold`.

---

### Task 2: Relative segue

**Files:** `signal.rs`, `config.rs` (`segue_drop_db`, `segue_max_secs` default 4, remove `segue_threshold_db`).

**Interfaces:** Produces `AnalysisSettings.segue_drop_db: f64`.

- [ ] **Step 1: Failing tests:**
  - `the_same_fade_gives_the_same_overlap_on_loud_and_quiet_masters`: one
    body at −8 dBFS RMS and one at −20 dBFS RMS, each followed by the same
    6 s linear fade in dB (−0 → −40 relative to the body). Assert that the
    overlaps (`cue_out − segue_start`) are within 0.1 s of each other.
  - `the_overlap_never_exceeds_segue_max`: a 20 s fade. The overlap is ≤ 4 s
    with the default settings.
  - `a_track_that_ends_at_full_level_has_a_tiny_overlap`: a hard ending. The
    overlap is ≤ 0.1 s.

- [ ] **Step 2: Run** them. Expected: FAIL (the absolute −18 dBFS threshold gives different overlaps).

- [ ] **Step 3: Implement:**
  - `let segue_level = median - s.segue_drop_db as f32;`;
  - `segue_start` is the end of the last body window with RMS at or above
    `segue_level`, clamped with `segue_max_secs`, which defaults to 4.0;
  - remove `segue_threshold_db` from `AnalysisSettings`, and add
    `segue_drop_db: 15.0` with a range of 3…40.

- [ ] **Step 4: Run** `cargo test -p fp-analysis -p fp-model`. Expected: PASS.

- [ ] **Step 5: Commit:** `feat(analysis): start segues relative to the track's own level`.

---

### Task 3: Store, cache and Settings

**Files:** `crates/fp-store/src/lenient.rs`, `crates/fp-analysis/src/cache.rs` (`ANALYSIS_VERSION` 5 → 6), `crates/fp-app/src/ui/settings.rs` (analysis page), both locales; tests `crates/fp-store/tests/store_roundtrip.rs`, `crates/fp-model/src/config.rs` tests, `crates/fp-analysis/tests` (cache).

- [ ] **Step 1: Failing tests:**
  - store: a `config.json` with `analysis.silence_threshold_db = -45` and
    `analysis.segue_threshold_db = -20` loads with the defaults for the new
    fields, keeps a valid `analysis.outro_drop_db`, and reports one warning
    per old field that names it.
  - config: the defaults and ranges of the three new fields and of
    `segue_max_secs` (4).
  - cache: an entry written under version 5 is not read back under version 6
    (reuse the existing version test pattern). The model keeps a track's
    manual markers when a new analysis arrives (existing rule; add a test if
    none exists).

- [ ] **Step 2: Run** the tests. Expected: FAIL.

- [ ] **Step 3: Implement:**
  - `lenient.rs`: in `merge`, a key that has no default (unknown to this
    version) is not merged. It produces the warning `config{field}: not
    used by this version; ignored`.
  - `ANALYSIS_VERSION = 6`.
  - Settings: replace the silence and segue-threshold sliders with:
    - "Trim threshold": −120…−20 dB, step 1;
    - "Trim margin": 0…1000 ms, step 5;
    - "Segue drop": 3…40 dB, step 1.

    The keys are `settings-trim-threshold`, `settings-trim-margin` and
    `settings-segue-drop`. The Spanish strings are "Umbral de recorte",
    "Margen de recorte" and "Caída para el segue". Remove
    `settings-silence` and `settings-segue-threshold` from both locales.

- [ ] **Step 4: Run** `cargo test --workspace`. Expected: PASS.

- [ ] **Step 5: Commit:** `feat(ui): trim and segue settings; recompute automatic markers`.

---

### Task 4: Corpus folder, report and corpus tests

**Files:**
- `.gitignore`;
- `test-music/README.md`;
- `crates/fp-analysis/tests/support/corpus.rs`;
- `crates/fp-analysis/tests/real_music.rs`;
- `crates/fp-analysis/examples/marker_report.rs`;
- `CLAUDE.md`;
- `docs/technical/testing.md` (create it if it is missing, and link it from
  `docs/technical/README.md`).

- [ ] **Step 1:**
  - `.gitignore`: add `/test-music/*` and `!/test-music/README.md`.
  - `test-music/README.md`: what to put there (any supported audio; a few
    dozen tracks of varied genres and masters, with natural intros and
    endings) and how to run the tests.
- [ ] **Step 2: Corpus helper.**
  - `corpus::files() -> Vec<PathBuf>` reads `FAUSTE_TEST_MUSIC`, or else
    `<CARGO_MANIFEST_DIR>/../../test-music`.
  - It walks the folder recursively, sorts the files and keeps the
    extensions `fp-decode` supports. Read the list the app uses:
    `fp_app::ui::files::AUDIO_EXTENSIONS` is not reachable from
    `fp-analysis`, so write the list next to the helper and cross-check it
    with `fp-decode`'s docs.
  - It prints a note and returns an empty list when there is nothing.
- [ ] **Step 3: `real_music.rs`**, `#[ignore]`. For every file:
  - `analyze_file` succeeds;
  - every bucket with a peak at or above `trim_threshold_db` lies inside
    `[cue_in, cue_out]`: re-derive the envelope with `EnvelopeBuilder`
    through `fp-decode`, or expose the peaks from `Analysis.peaks` and use
    `max(|min|, |max|)`;
  - the overlap is ≤ `segue_max_secs`.

  Run it with `cargo test -p fp-analysis --test real_music -- --ignored`.
  Expected: it passes with an empty corpus, printing the note.
- [ ] **Step 4: `marker_report` example.**
  - Arguments: `[--set key=value]… [dir]`.
  - The overrides go through `serde_json` onto `AnalysisSettings`.
  - It prints one row per file: name, duration, cue_in, cue_out, segue,
    overlap and outro, plus the median and the percentiles of the overlap.
- [ ] **Step 5: Docs.**
  - `CLAUDE.md` Commands: the corpus test and `marker_report` lines.
  - `CLAUDE.md` Testing notes: the corpus.
  - `docs/technical/testing.md`: the same, in more detail.
- [ ] **Step 6:** gate. Commit `test(analysis): real-music corpus tests and a marker report`.

---

### Task 5: Docs, review, pull request

- Update:
  - main spec §6 (markers);
  - `docs/user/markers-and-mixing.md`;
  - `docs/user/settings.md` (Analysis);
  - `docs/technical/analysis.md`;
  - `docs/technical/persistence.md` (the `analysis` table).
- Tuning checkpoint (spec §4.2): with no corpus on this machine, the
  defaults stay at the spec's starting values (−60 dB, 20 ms, 15 dB, 4 s).
  Ledger it as a ruling. The maintainer runs `marker_report` on their music
  and adjusts later.
- Full verification. Final review (a fresh reviewer on the most capable
  model). PR `feat(analysis): trimming never removes audio; shorter segues`,
  CI, then merge.
