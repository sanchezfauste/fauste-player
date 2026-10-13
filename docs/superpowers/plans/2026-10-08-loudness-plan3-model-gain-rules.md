# Loudness Normalization, Plan 3: Gain Rules, Configuration and the Per-Track Edit

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The model decides one fixed normalization gain per source: a `loudness` configuration section, the per-track exclude/trim edit stored in the library, the pure gain rule (N7) for the device each holder plays on (N8), every `SourceRequest` and `CartRequest` carrying its `Normalization` (N9), fixed while it sounds (N11), re-preloaded when the next entry's gain changes (N12) and recomputed on crash recovery (N13).

**Architecture:** `fp_model::loudness` (created by plan 2) gains `LoudnessEdit`, `Normalization`, `NormalizationStatus`, `GainLimit`, `normalization` (N7), `holder_bit_perfect` (N8) and `normalization_for`. `Config::loudness: LoudnessConfig` lives in `config.rs` with the other sections. The request builders of `AppState` take the `Holder` that will play the source and fill `normalization`. `PlayerState` records `on_air` and `cue_normalization`, the cartwall records each `PlayingCart`'s and its CUE's, runtime only; `reconcile` clears the ones whose source is gone and compares `(entry, from_secs, normalization)` to decide a new `Preload`. The engine receives the field but ignores it until plan 4 (nothing changes on air yet: `normalize` defaults to off, and the engine does not apply the gain).

**Tech Stack:** Rust 2024, `fp-model` (pure, no I/O), `serde`.

**Spec:** `docs/superpowers/specs/2026-10-08-loudness-normalization-design.md` (§3, §4 N1–N3, §6 N7–N13, §10 `fp-model` rows, §14 Q1, Q2, Q6). Read also `crates/fp-model/src/live.rs` (holders, placement): loudness settings follow its "apply to future loads, never pending" model (N3).

**Ruling (planning): a stored trim beyond `max_trim_db`.** N10 refuses such a trim when it is set, but `max_trim_db` can be lowered afterwards (or the library edited by hand). N7 uses the trim clamped to ±`max_trim_db`, and a trim that is not finite as 0. — Why: an operator who lowers the maximum trim expects it to bound every track. — Cost if wrong: one `clamp` in `normalization`; the stored value is kept, so raising the maximum again restores it.

**Ruling (planning): where the runtime records live.** `PlayerState::on_air`, `PlayerState::cue_normalization`, `Cartwall::cue_normalization` and `PlayingCart::normalization` are set where the request is built and cleared in `reconcile` once their source is gone (`current`, `cue`, `cartwall.cue` is `None`), so no code path that ends a source has to remember them. — Cost if wrong: none for the audio (the engine keeps its own copy, plan 4); a stale badge for at most one `apply`.

## Global Constraints

- All code, identifiers, comments, docs and commit messages in English; never mention other playout or radio-automation products.
- Behaviour lives in `fp-model` as pure functions, **one test per spec rule** (CLAUDE.md rule 7); test names start with the rule (`n7_…`).
- §4 configuration, verbatim: `normalize` `bool` default `false` (D11); `target_lufs` `f32` −16.0, range −36.0 ..= −6.0; `true_peak_ceiling_dbtp` `f32` −1.0, range −12.0 ..= 0.0; `max_boost_db` `f32` 12.0, range 0.0 ..= 24.0; `max_trim_db` `f32` 12.0, range 0.0 ..= 24.0. `#[serde(default)]` on the section (lenient loading).
- N1: clamp into the range "with a `ConfigWarning` naming the field (`loudness.target_lufs`, …) … a non-finite value takes the default".
- N3: "A loudness change never waits and never touches a sounding source … it produces no pending item and no `EngineAction::UpdateSettings` … `restart_pending` gains no reason for it."
- N7 (verbatim table order): `bit_perfect` → `BitPerfect`; `!normalize` → `Off`; `excluded` → `Excluded`; `loudness == None` → `NotMeasured`; `integrated_lufs == None` → `Silent`; otherwise `Applied { limited }`. `wanted = target_lufs − integrated_lufs + trim_db`; `wanted ≤ 0` → `g = wanted`, `limited = None`; else `g = min(wanted, max(0, ceiling − true_peak), max_boost)`, `limited` the bound that won when lower than `wanted`, **`TruePeak` before `MaxBoost` on a tie**; rounded to 0.01 dB. Every status but `Applied` has `gain_db = 0`.
- N9: CUE and pre-listen "normalized exactly like on air (Q6) … only `bit_perfect` of the CUE holder's own device can make the result differ."
- N11: on-air and CUE normalizations change only with a new request or a scheduled transition copying the preloaded one; "not an analysis or loudness-only result, a configuration change, a loudness edit, nor a pending device change applied while the track is paused".
- Nothing goes on air by itself (CLAUDE.md rule 10); `unwrap`, `expect`, `panic` denied outside tests.
- Conventional Commits; every commit ends with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Gate for every task: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`.
- Progress ledger: `.superpowers/sdd/2026-10-08-loudness-plan3-model-gain-rules/progress.md`.

## Review Focus

1. **A Seek or a CUE seek while normalization settings changed meanwhile**: the reopened source must keep its gain; the model sends only `Seek`/`SeekCue` (no new request), so nothing new is computed. Test: Task 5, `n11_a_seek_sends_no_new_gain`.
2. **A repeating entry (R26) whose gain changes while it plays**: its next pass is the preload of the same entry, re-preloaded with the new gain; the pass on air keeps its own. Test: Task 5, `n12_a_repeating_entry_takes_the_new_gain_on_its_next_pass`.
3. **A trim stored above a `max_trim_db` that was lowered later**: clamped by N7. Test: Task 2, `n7_a_stored_trim_is_limited_by_the_current_maximum`.
4. **A `TransitionStarted` for an entry other than the one preloaded** (the engine follows its own preload; the model may have changed its next meanwhile): `on_air` is recomputed for the entry that really started, not copied from a preload of another entry. Test: Task 5, `n11_a_transition_to_another_entry_recomputes_its_gain`.
5. **A non-finite configuration value from code** (a UI slider bug sending `NaN` through `UpdateConfig`): it becomes the default, never a NaN gain. Test: Task 1, `n1_a_value_that_is_not_a_number_takes_the_default`.

---

## File Structure

| File | Responsibility |
|---|---|
| `crates/fp-model/src/config.rs` | `LoudnessConfig`, `Config::loudness`, its validation (N1) |
| `crates/fp-model/src/restore.rs` | `SettingsSection::Loudness` (N2) |
| `crates/fp-model/src/loudness.rs` | `LoudnessEdit`, `Normalization`, `NormalizationStatus`, `GainLimit`, `normalization`, `holder_bit_perfect`, `normalization_for` |
| `crates/fp-model/src/track.rs` | `Track::loudness_edit` |
| `crates/fp-model/src/command.rs` | `SourceRequest::normalization`, `CartRequest::normalization`, `Command::SetTrackLoudness` |
| `crates/fp-model/src/error.rs` | `ModelError::InvalidTrim` |
| `crates/fp-model/src/state.rs` | `request_from_cue_in`, `request_at`, `playable_request` take a `Holder` |
| `crates/fp-model/src/player.rs` | `PlayerState::{on_air, cue_normalization}`, `preloaded` gains the normalization |
| `crates/fp-model/src/cartwall.rs` | `PlayingCart::normalization`, `Cartwall::cue_normalization` |
| `crates/fp-model/src/reducer.rs`, `pending_start.rs`, `availability.rs`, `cart_rules.rs`, `session.rs` | Callers pass their holder; on-air records; `reconcile` |
| `crates/fp-model/tests/loudness_config.rs` (new), `tests/restore.rs`, `tests/loudness.rs` (new) | The rule tests |
| `crates/fp-engine/tests/*.rs`, `crates/fp-app/src/ui/view.rs:225`, `crates/fp-app/tests/{track_tags_ui,exit_guard}.rs`, `crates/fp-model/tests/{on_air,history}.rs` | Compile fixes for the new fields and parameters |
| `docs/technical/persistence.md`, `docs/technical/architecture.md` or the model page, bit-perfect spec B1 | Task 6 |

---

### Task 1: `LoudnessConfig` (N1, N2, N3)

**Files:**
- Modify: `crates/fp-model/src/config.rs` (`Config` L16-50, a new struct after `MeterConfig` ~L211, `Config::validate` ~L1240 before `w`)
- Modify: `crates/fp-model/src/restore.rs`, `crates/fp-model/src/lib.rs` (re-export `LoudnessConfig`)
- Test: `crates/fp-model/tests/loudness_config.rs` (new), `crates/fp-model/tests/restore.rs`

**Interfaces:**
- Produces: `pub struct LoudnessConfig { pub normalize: bool, pub target_lufs: f32, pub true_peak_ceiling_dbtp: f32, pub max_boost_db: f32, pub max_trim_db: f32 }` (`Debug, Clone, PartialEq, Serialize, Deserialize`, `#[serde(default)]`, `Default` per §4); `Config::loudness: LoudnessConfig`; `SettingsSection::Loudness`.

- [ ] **Step 1: Write the failing tests**

`crates/fp-model/tests/loudness_config.rs`:

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Loudness normalization spec §4: the `loudness` configuration section.

mod common;

use common::{fixture, p0};
use fp_model::{Command, Config, EngineAction, LoudnessConfig, apply, pending, restart_pending};

#[test]
fn the_defaults_are_those_of_the_spec() {
    let c = LoudnessConfig::default();
    assert!(!c.normalize, "D11: off by default");
    assert_eq!(
        (c.target_lufs, c.true_peak_ceiling_dbtp, c.max_boost_db, c.max_trim_db),
        (-16.0, -1.0, 12.0, 12.0)
    );
    assert_eq!(Config::default().loudness, c);
}

#[test]
fn n1_each_value_is_clamped_into_its_range_with_a_warning() {
    for (set, field, expected) in [
        ((|c: &mut LoudnessConfig| c.target_lufs = -50.0) as fn(&mut LoudnessConfig), "loudness.target_lufs", -36.0),
        (|c| c.target_lufs = 0.0, "loudness.target_lufs", -6.0),
        (|c| c.true_peak_ceiling_dbtp = 3.0, "loudness.true_peak_ceiling_dbtp", 0.0),
        (|c| c.true_peak_ceiling_dbtp = -20.0, "loudness.true_peak_ceiling_dbtp", -12.0),
        (|c| c.max_boost_db = 30.0, "loudness.max_boost_db", 24.0),
        (|c| c.max_boost_db = -1.0, "loudness.max_boost_db", 0.0),
        (|c| c.max_trim_db = 99.0, "loudness.max_trim_db", 24.0),
        (|c| c.max_trim_db = -3.0, "loudness.max_trim_db", 0.0),
    ] {
        let mut config = Config::default();
        set(&mut config.loudness);
        let warnings = config.validate();
        assert!(warnings.iter().any(|w| w.field == field), "{field}: {warnings:?}");
        let l = &config.loudness;
        let value = match field {
            "loudness.target_lufs" => l.target_lufs,
            "loudness.true_peak_ceiling_dbtp" => l.true_peak_ceiling_dbtp,
            "loudness.max_boost_db" => l.max_boost_db,
            _ => l.max_trim_db,
        };
        assert_eq!(value, expected, "{field}");
    }
}

#[test]
fn n1_a_value_that_is_not_a_number_takes_the_default() {
    let mut config = Config::default();
    config.loudness.target_lufs = f32::NAN;
    config.loudness.true_peak_ceiling_dbtp = f32::INFINITY;
    config.loudness.max_boost_db = f32::NEG_INFINITY;
    config.loudness.max_trim_db = f32::NAN;
    let warnings = config.validate();
    assert_eq!(config.loudness, LoudnessConfig::default());
    assert_eq!(warnings.iter().filter(|w| w.field.starts_with("loudness.")).count(), 4);
}

#[test]
fn n1_values_in_range_give_no_warning() {
    let mut config = Config::default();
    config.loudness = LoudnessConfig {
        normalize: true,
        target_lufs: -23.0,
        true_peak_ceiling_dbtp: -2.0,
        max_boost_db: 6.0,
        max_trim_db: 3.0,
    };
    let before = config.loudness.clone();
    assert!(config.validate().iter().all(|w| !w.field.starts_with("loudness.")));
    assert_eq!(config.loudness, before);
}

#[test]
fn n1_a_file_without_the_section_loads_the_defaults() {
    let config: Config = serde_json::from_str(r#"{"players":{"count":2}}"#).unwrap();
    assert_eq!(config.loudness, LoudnessConfig::default());
    let partial: Config =
        serde_json::from_str(r#"{"loudness":{"normalize":true}}"#).unwrap();
    assert!(partial.loudness.normalize);
    assert_eq!(partial.loudness.target_lufs, -16.0);
}

#[test]
fn n3_a_loudness_change_only_preloads_and_never_waits() {
    let mut state = fixture(3);
    let p = p0(&state);
    apply(&mut state, Command::Play(p)).unwrap();
    let before = state.config.clone();
    let mut config = state.config.clone();
    config.loudness.normalize = true;
    config.loudness.target_lufs = -20.0;
    let actions = apply(&mut state, Command::UpdateConfig(Box::new(config))).unwrap();
    assert!(
        actions.iter().all(|a| matches!(a, EngineAction::Preload { .. })),
        "{actions:?}"
    );
    assert!(restart_pending(&before, &state.config).is_empty());
    assert!(pending(&state).is_empty());
}
```

(If clippy rejects the function-pointer table, rewrite it as one block per field; keep every case.)

`crates/fp-model/tests/restore.rs`: in `altered()` add `c.loudness.normalize = true; c.loudness.target_lufs = -20.0;`, add `LoudnessConfig` to the imports, and:

```rust
#[test]
fn n2_loudness_resets_the_loudness_section_only() {
    let mut c = altered();
    restore_defaults(&mut c, SettingsSection::Loudness);
    assert_eq!(c.loudness, LoudnessConfig::default());
    let mut expected = altered();
    expected.loudness = LoudnessConfig::default();
    assert_eq!(c, expected);
}
```

Also check that the existing restore tests for the other sections still assert "everything else unchanged" with the altered loudness (they compare against `altered()` with only their section replaced, so they keep passing).

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p fp-model --test loudness_config --test restore 2>&1 | tail -5`
Expected: FAIL to compile (`LoudnessConfig`, `SettingsSection::Loudness` not found).

- [ ] **Step 3: Implement**

`config.rs`, in `Config` after `remote`:

```rust
    /// Loudness normalization (loudness normalization spec §4). Changes
    /// apply to sources loaded afterwards (N3).
    pub loudness: LoudnessConfig,
```

and `loudness: LoudnessConfig::default(),` in `Default`. After `MeterConfig`'s `impl Default`:

```rust
/// Loudness normalization (loudness normalization spec §4): one fixed gain
/// per track, decided when it is loaded.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LoudnessConfig {
    /// Normalization on or off. Off by default: an upgrade changes nothing
    /// on air until the operator turns it on (D11).
    pub normalize: bool,
    /// Integrated loudness every track is brought to, LUFS (−36 ..= −6).
    pub target_lufs: f32,
    /// A boost may not take the measured true peak above this, dBTP
    /// (−12 ..= 0); it never cuts a track (N7).
    pub true_peak_ceiling_dbtp: f32,
    /// The largest gain normalization may give, dB (0 ..= 24); cuts are
    /// not limited.
    pub max_boost_db: f32,
    /// The per-track trim is limited to ± this, dB (0 ..= 24).
    pub max_trim_db: f32,
}

impl Default for LoudnessConfig {
    fn default() -> Self {
        Self {
            normalize: false,
            target_lufs: -16.0,
            true_peak_ceiling_dbtp: -1.0,
            max_boost_db: 12.0,
            max_trim_db: 12.0,
        }
    }
}

/// `clamp_to`, except that a value that is not finite takes `default`.
fn clamp_finite(
    value: &mut f32,
    min: f32,
    max: f32,
    default: f32,
    field: &'static str,
    out: &mut Vec<ConfigWarning>,
) {
    if value.is_finite() {
        clamp_to(value, min, max, field, out);
    } else {
        out.push(ConfigWarning {
            field,
            message: format!("{value} is not a number; using {default}"),
        });
        *value = default;
    }
}

impl LoudnessConfig {
    /// N1: every value into its range.
    fn validate(&mut self, w: &mut Vec<ConfigWarning>) {
        let d = Self::default();
        clamp_finite(&mut self.target_lufs, -36.0, -6.0, d.target_lufs, "loudness.target_lufs", w);
        clamp_finite(
            &mut self.true_peak_ceiling_dbtp,
            -12.0,
            0.0,
            d.true_peak_ceiling_dbtp,
            "loudness.true_peak_ceiling_dbtp",
            w,
        );
        clamp_finite(&mut self.max_boost_db, 0.0, 24.0, d.max_boost_db, "loudness.max_boost_db", w);
        clamp_finite(&mut self.max_trim_db, 0.0, 24.0, d.max_trim_db, "loudness.max_trim_db", w);
    }
}
```

In `Config::validate`, before the final `w`: `self.loudness.validate(&mut w);`.

`restore.rs`: add `Loudness` to `SettingsSection` (doc comment list: "`players`, `meter`, `analysis`, `loudness` or `shortcuts`") and the arm `SettingsSection::Loudness => config.loudness = defaults.loudness,`. `lib.rs`: add `LoudnessConfig` to the `pub use config::{…}` list.

`update_config` needs no change: `UpdateSettings` is only sent when `outputs` or `tuning` change (live settings L10), and `restart_pending` compares only outputs, limits and tuning.

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p fp-model`
Expected: PASS.

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-model
git commit -m "feat(model): add the loudness normalization settings

A loudness section with the target, the true-peak ceiling and the
maximum boost and trim, validated with ranges and off by default
(loudness normalization spec §4, N1-N3).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: The gain rule and the device rule (N7, N8)

**Files:**
- Modify: `crates/fp-model/src/loudness.rs`, `crates/fp-model/src/track.rs` (`Track::loudness_edit`), `crates/fp-model/src/lib.rs`
- Test: `crates/fp-model/tests/loudness.rs` (new)

**Interfaces:**
- Consumes: `LoudnessMeasurement` (plan 2), `LoudnessConfig` (Task 1), `fp_model::live::{Holder, configured_route}`, `AppState::live`.
- Produces (re-exported at the crate root):
  - `#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)] #[serde(default)] pub struct LoudnessEdit { pub excluded: bool, pub trim_db: f32 }`
  - `Track::loudness_edit: LoudnessEdit` (`#[serde(default)]`)
  - `#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum GainLimit { TruePeak, MaxBoost }`
  - `#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum NormalizationStatus { Applied { limited: Option<GainLimit> }, Off, Excluded, NotMeasured, Silent, BitPerfect }`
  - `#[derive(Debug, Clone, Copy, PartialEq)] pub struct Normalization { pub gain_db: f32, pub status: NormalizationStatus }`, `impl Default` (`0.0`, `Off`), `Normalization::linear_gain(&self) -> f32` (exactly `1.0` for 0 dB; `1.0` for any non-finite result)
  - `pub fn normalization(config: &LoudnessConfig, track: &Track, bit_perfect: bool) -> Normalization`
  - `pub fn holder_bit_perfect(state: &AppState, holder: Holder) -> bool`
  - `pub fn normalization_for(state: &AppState, holder: Holder, track: &Track) -> Normalization`

- [ ] **Step 1: Write the failing tests**

`crates/fp-model/tests/loudness.rs`:

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Loudness normalization spec §6: the gain rule (N7), the device rule
//! (N8), requests (N9), edits (N10), fixed while sounding (N11), loaded
//! afterwards (N12) and crash recovery (N13).

mod common;

use std::path::PathBuf;

use common::{entries, fixture, p0};
use fp_model::{
    AppState, Command, DeviceSettings, EngineAction, EngineEvent, GainLimit, Holder, LoudnessConfig,
    LoudnessEdit, LoudnessMeasurement, Normalization, NormalizationStatus as S, OutputDevice,
    Route, Track, TrackId, apply, device_settings, holder_bit_perfect, normalization, on_event,
};

fn on() -> LoudnessConfig {
    LoudnessConfig {
        normalize: true,
        ..LoudnessConfig::default()
    }
}

fn track(lufs: Option<f32>, tp: f32) -> Track {
    let mut t = Track::new(TrackId(1), PathBuf::from("/a.flac"));
    t.loudness = Some(LoudnessMeasurement {
        integrated_lufs: lufs,
        true_peak_dbtp: tp,
    });
    t
}

fn gain(n: Normalization) -> (f32, S) {
    (n.gain_db, n.status)
}

#[test]
fn n7_bit_perfect_wins() {
    let t = track(Some(-20.0), -10.0);
    assert_eq!(gain(normalization(&on(), &t, true)), (0.0, S::BitPerfect));
}

#[test]
fn n7_off_gives_no_gain() {
    let t = track(Some(-20.0), -10.0);
    assert_eq!(gain(normalization(&LoudnessConfig::default(), &t, false)), (0.0, S::Off));
}

#[test]
fn n7_an_excluded_track_gets_no_gain() {
    let mut t = track(Some(-20.0), -10.0);
    t.loudness_edit.excluded = true;
    assert_eq!(gain(normalization(&on(), &t, false)), (0.0, S::Excluded));
}

#[test]
fn n7_a_track_not_measured_plays_at_0_db() {
    let mut t = track(Some(-20.0), -10.0);
    t.loudness = None;
    assert_eq!(gain(normalization(&on(), &t, false)), (0.0, S::NotMeasured));
}

#[test]
fn n7_a_silent_track_gets_no_gain() {
    let t = track(None, -80.0);
    assert_eq!(gain(normalization(&on(), &t, false)), (0.0, S::Silent));
}

#[test]
fn n7_a_measured_track_is_brought_to_the_target() {
    let t = track(Some(-20.0), -10.0);
    assert_eq!(gain(normalization(&on(), &t, false)), (4.0, S::Applied { limited: None }));
}

#[test]
fn n7_a_boost_is_limited_by_the_true_peak_ceiling() {
    // Wants +4; the peak may rise only from −3 to −1 dBTP.
    let t = track(Some(-20.0), -3.0);
    assert_eq!(
        gain(normalization(&on(), &t, false)),
        (2.0, S::Applied { limited: Some(GainLimit::TruePeak) })
    );
}

#[test]
fn n7_a_boost_is_limited_by_the_maximum_boost() {
    let t = track(Some(-40.0), -30.0);
    assert_eq!(
        gain(normalization(&on(), &t, false)),
        (12.0, S::Applied { limited: Some(GainLimit::MaxBoost) })
    );
}

#[test]
fn n7_on_a_tie_the_true_peak_is_named() {
    // Wants +24; the ceiling allows +12 and so does the maximum boost.
    let t = track(Some(-40.0), -13.0);
    assert_eq!(
        gain(normalization(&on(), &t, false)),
        (12.0, S::Applied { limited: Some(GainLimit::TruePeak) })
    );
}

#[test]
fn n7_a_boost_with_the_peak_already_above_the_ceiling_gets_0_db() {
    // Spec §10: −18 LUFS, +0.5 dBTP, target −16 → +0 dB.
    let t = track(Some(-18.0), 0.5);
    assert_eq!(
        gain(normalization(&on(), &t, false)),
        (0.0, S::Applied { limited: Some(GainLimit::TruePeak) })
    );
}

#[test]
fn n7_a_cut_is_never_limited() {
    // Spec §10: −10 LUFS, +0.5 dBTP keeps its −6 dB.
    let t = track(Some(-10.0), 0.5);
    assert_eq!(gain(normalization(&on(), &t, false)), (-6.0, S::Applied { limited: None }));
    // Nor by the maximum boost.
    let mut c = on();
    c.max_boost_db = 0.0;
    let t = track(Some(0.0), -0.1);
    assert_eq!(gain(normalization(&c, &t, false)), (-16.0, S::Applied { limited: None }));
}

#[test]
fn n7_the_trim_is_added_before_the_limits() {
    let mut t = track(Some(-20.0), -10.0);
    t.loudness_edit.trim_db = 3.0; // wants +7, ceiling allows +9
    assert_eq!(gain(normalization(&on(), &t, false)), (7.0, S::Applied { limited: None }));
    t.loudness_edit.trim_db = 6.0; // wants +10, ceiling allows +9
    assert_eq!(
        gain(normalization(&on(), &t, false)),
        (9.0, S::Applied { limited: Some(GainLimit::TruePeak) })
    );
    t.loudness_edit.trim_db = -6.0; // wants −2: a cut
    assert_eq!(gain(normalization(&on(), &t, false)), (-2.0, S::Applied { limited: None }));
}

#[test]
fn n7_a_stored_trim_is_limited_by_the_current_maximum() {
    let mut t = track(Some(-20.0), -20.0);
    t.loudness_edit.trim_db = 10.0;
    let mut c = on();
    c.max_trim_db = 2.0;
    assert_eq!(gain(normalization(&c, &t, false)).0, 6.0);
    t.loudness_edit.trim_db = f32::NAN;
    assert_eq!(gain(normalization(&c, &t, false)).0, 4.0);
}

#[test]
fn n7_the_gain_is_rounded_to_a_hundredth_of_a_db() {
    let t = track(Some(-20.004_7), -10.0);
    assert_eq!(normalization(&on(), &t, false).gain_db, 4.0);
    let t = track(Some(-20.016), -10.0);
    assert_eq!(normalization(&on(), &t, false).gain_db, 4.02);
}

#[test]
fn unity_is_exactly_one() {
    assert_eq!(Normalization::default().linear_gain(), 1.0);
    let six = Normalization {
        gain_db: 6.02,
        status: S::Applied { limited: None },
    };
    assert!((six.linear_gain() - 2.0).abs() < 1e-3);
}

/// A state normalizing 3 tracks of −20 LUFS / −10 dBTP: +4 dB each.
fn normalizing(tracks: usize) -> AppState {
    let mut s = fixture(tracks);
    s.config.loudness.normalize = true;
    for t in s.library.iter_mut() {
        t.analyzed = true;
        t.loudness = Some(LoudnessMeasurement {
            integrated_lufs: Some(-20.0),
            true_peak_dbtp: -10.0,
        });
    }
    // The fixture preloaded the first entry at 0 dB; one `apply` preloads
    // it again with its gain, so each test sees only its own actions.
    let p = p0(&s);
    apply(&mut s, Command::SetVolume(p, 1.0)).unwrap();
    s
}

fn route(device: &str) -> Route {
    Route {
        backend: "null".into(),
        device: device.into(),
        first_channel: 0,
    }
}

fn device(name: &str) -> OutputDevice {
    OutputDevice {
        backend: "null".into(),
        device: name.into(),
    }
}

#[test]
fn n8_the_default_output_is_not_bit_perfect() {
    let s = normalizing(1);
    assert!(!holder_bit_perfect(&s, Holder::PlayerMain(p0(&s))));
}

#[test]
fn n8_before_the_engine_reports_the_configured_route_decides() {
    let mut s = normalizing(1);
    let p = p0(&s);
    s.config.outputs.routes[0].main = Some(route("dac"));
    s.config.outputs.bit_perfect.push(device("dac"));
    assert!(holder_bit_perfect(&s, Holder::PlayerMain(p)));
    assert!(!holder_bit_perfect(&s, Holder::PlayerCue(p)), "phones are not");
}

#[test]
fn n8_once_placed_the_running_device_decides() {
    let mut s = normalizing(1);
    let p = p0(&s);
    // Configured bit-perfect, but the engine runs the device without it.
    s.config.outputs.routes[0].main = Some(route("dac"));
    s.config.outputs.bit_perfect.push(device("dac"));
    let mut running: DeviceSettings = device_settings(&s.config.outputs, &device("dac"));
    running.bit_perfect = false;
    on_event(
        &mut s,
        EngineEvent::Placed {
            holder: Holder::PlayerMain(p),
            route: Some(route("dac")),
            device: device("dac"),
            running,
        },
    );
    assert!(!holder_bit_perfect(&s, Holder::PlayerMain(p)));
}
```

The `use` list above is the one the whole file ends with; in this task import only what its tests use (clippy denies unused imports), and let each later task add its own.

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p fp-model --test loudness 2>&1 | tail -5`
Expected: FAIL to compile (`normalization`, `Normalization`, … not found).

- [ ] **Step 3: Implement**

`track.rs`: in `Track`, after `loudness_pass`:

```rust
    /// The operator's loudness edit (N10): exclude from normalization, or a
    /// trim in dB. Stored with the markers; libraries saved earlier have
    /// the default (not excluded, 0 dB).
    #[serde(default)]
    pub loudness_edit: LoudnessEdit,
```

`Track::new`: `loudness_edit: LoudnessEdit::default(),`. Import `LoudnessEdit` from `crate::loudness`.

`loudness.rs`, add (imports: `crate::config::{LoudnessConfig, OutputDevice}`, `crate::live::{Holder, configured_route}`, `crate::track::Track`):

```rust
/// The operator's per-track loudness setting (spec §3).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct LoudnessEdit {
    pub excluded: bool,
    /// Added to the gain, dB; limited to ± `max_trim_db` (N10).
    pub trim_db: f32,
}

/// The bound that limited a boost (N7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GainLimit {
    TruePeak,
    MaxBoost,
}

/// Why a source has the gain it has (spec §3). Every status but `Applied`
/// has a gain of 0 dB.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalizationStatus {
    Applied { limited: Option<GainLimit> },
    Off,
    Excluded,
    NotMeasured,
    Silent,
    BitPerfect,
}

/// The fixed gain one source gets (spec §3, N7).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Normalization {
    pub gain_db: f32,
    pub status: NormalizationStatus,
}

impl Default for Normalization {
    /// No gain: what every source had before normalization existed.
    fn default() -> Self {
        Self {
            gain_db: 0.0,
            status: NormalizationStatus::Off,
        }
    }
}

impl Normalization {
    fn none(status: NormalizationStatus) -> Self {
        Self { gain_db: 0.0, status }
    }

    /// The linear factor (`10^(dB/20)`); exactly 1.0 for 0 dB, so an
    /// unnormalized source stays bit-exact, and 1.0 for anything that is
    /// not a usable number.
    pub fn linear_gain(&self) -> f32 {
        if self.gain_db == 0.0 {
            return 1.0;
        }
        let g = 10f32.powf(self.gain_db / 20.0);
        if g.is_finite() && g > 0.0 { g } else { 1.0 }
    }
}

/// N7: the gain `track` gets under `config` on a device that is
/// `bit_perfect` or not.
pub fn normalization(config: &LoudnessConfig, track: &Track, bit_perfect: bool) -> Normalization {
    use NormalizationStatus as S;
    if bit_perfect {
        return Normalization::none(S::BitPerfect);
    }
    if !config.normalize {
        return Normalization::none(S::Off);
    }
    if track.loudness_edit.excluded {
        return Normalization::none(S::Excluded);
    }
    let Some(measured) = track.loudness else {
        return Normalization::none(S::NotMeasured);
    };
    let Some(lufs) = measured.integrated_lufs else {
        return Normalization::none(S::Silent);
    };
    let trim = track.loudness_edit.trim_db;
    let trim = if trim.is_finite() {
        trim.clamp(-config.max_trim_db, config.max_trim_db)
    } else {
        0.0
    };
    let wanted = config.target_lufs - lufs + trim;
    // The ceiling and the maximum boost only limit boosts (Q2).
    let (gain, limited) = if wanted <= 0.0 {
        (wanted, None)
    } else {
        let peak_room = (config.true_peak_ceiling_dbtp - measured.true_peak_dbtp).max(0.0);
        let boost = config.max_boost_db;
        if peak_room < wanted && peak_room <= boost {
            (peak_room, Some(GainLimit::TruePeak))
        } else if boost < wanted {
            (boost, Some(GainLimit::MaxBoost))
        } else {
            (wanted, None)
        }
    };
    let gain_db = (gain * 100.0).round() / 100.0;
    if !gain_db.is_finite() {
        return Normalization::none(S::NotMeasured);
    }
    Normalization {
        gain_db,
        status: S::Applied { limited },
    }
}

/// N8: whether `holder` plays on a bit-perfect device: the running
/// settings of its placement once the engine reported it (live settings
/// L3), otherwise its configured route (no route: the default output,
/// not bit-perfect).
pub fn holder_bit_perfect(state: &AppState, holder: Holder) -> bool {
    if let Some(running) = state
        .live
        .placement
        .get(&holder)
        .and_then(|device| state.live.devices.get(device))
    {
        return running.bit_perfect;
    }
    configured_route(&state.config, holder).is_some_and(|r| {
        state.config.outputs.bit_perfect.contains(&OutputDevice {
            backend: r.backend,
            device: r.device,
        })
    })
}

/// N7 for `track` played by `holder`.
pub fn normalization_for(state: &AppState, holder: Holder, track: &Track) -> Normalization {
    normalization(&state.config.loudness, track, holder_bit_perfect(state, holder))
}
```

`lib.rs`: `pub use loudness::{GainLimit, LOUDNESS_VERSION, LoudnessEdit, LoudnessMeasurement, Normalization, NormalizationStatus, holder_bit_perfect, loudness_unmeasured, normalization, normalization_for};`.

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p fp-model --test loudness`
Expected: PASS.

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-model
git commit -m "feat(model): decide the normalization gain of a track

A pure rule brings each measured track to the target, limits boosts by
the true-peak ceiling and the maximum boost, never limits a cut, and
gives no gain on a bit-perfect device (loudness normalization spec N7,
N8).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: Every request carries its normalization (N9)

**Files:**
- Modify: `crates/fp-model/src/command.rs` (`SourceRequest` ~L304, `CartRequest` ~L420)
- Modify: `crates/fp-model/src/state.rs` (`playable_request`, `request_from_cue_in`, `request_at`)
- Modify: every caller: `crates/fp-model/src/reducer.rs` (L34 Restart, L134 Seek, L513 previous, L776 `dsd_hold_continue`, L894 `cue_entry`, L918 `follow_cue`, L939 `seek_cue`, L1193 `advance_to`), `crates/fp-model/src/pending_start.rs` (L22, L62, L77, L79 — `pending_start_at`, `start_request`, `preload_request` gain a `holder`/use the player's), `crates/fp-model/src/availability.rs:34`, `crates/fp-model/src/session.rs:227`, `crates/fp-model/src/cart_rules.rs` (`request`), `crates/fp-app/src/ui/view.rs:225`, `crates/fp-model/tests/history.rs:78`
- Modify (compile fixes): the `SourceRequest { … }` / `CartRequest { … }` literals in `crates/fp-engine/tests/{engine,review_fixes,audio_path_audit,bit_perfect,device_overrides,dsd_output,dsd_real_music,cartwall}.rs` gain `normalization: Normalization::default(),`
- Test: `crates/fp-model/tests/loudness.rs`

**Interfaces:**
- Produces:
  - `SourceRequest::normalization: Normalization`, `CartRequest::normalization: Normalization`
  - `AppState::playable_request(&self, entry: EntryId, holder: Holder) -> Option<SourceRequest>`
  - `AppState::request_from_cue_in(&self, entry: EntryId, holder: Holder) -> Option<SourceRequest>`
  - `AppState::request_at(&self, entry: EntryId, secs: f64, holder: Holder) -> Option<SourceRequest>`

- [ ] **Step 1: Write the failing tests** (append to `tests/loudness.rs`)

```rust
fn request_gain(actions: &[EngineAction]) -> Vec<(&'static str, f32)> {
    actions
        .iter()
        .filter_map(|a| match a {
            EngineAction::Preload { request: Some(r), .. } => Some(("preload", r.normalization.gain_db)),
            EngineAction::StartCurrent { request, .. } => Some(("start", request.normalization.gain_db)),
            EngineAction::Crossfade { request, .. } => Some(("crossfade", request.normalization.gain_db)),
            EngineAction::StartCue { request, .. } => Some(("cue", request.normalization.gain_db)),
            EngineAction::LoadPaused { request, .. } => Some(("load", request.normalization.gain_db)),
            EngineAction::StartCart(r) => Some(("cart", r.normalization.gain_db)),
            EngineAction::StartCartCue(r) => Some(("cart-cue", r.normalization.gain_db)),
            _ => None,
        })
        .collect()
}

#[test]
fn n9_player_requests_carry_the_main_gain() {
    let mut s = normalizing(3);
    let p = p0(&s);
    let start = apply(&mut s, Command::Play(p)).unwrap();
    assert!(request_gain(&start).contains(&("start", 4.0)), "{start:?}");
    assert!(request_gain(&start).contains(&("preload", 4.0)), "{start:?}");
    let fade = apply(&mut s, Command::Play(p)).unwrap();
    assert!(request_gain(&fade).contains(&("crossfade", 4.0)), "{fade:?}");
}

#[test]
fn n9_the_cue_is_normalized_like_on_air() {
    let mut s = normalizing(3);
    let p = p0(&s);
    let cue = apply(&mut s, Command::ToggleCue(p)).unwrap();
    assert_eq!(request_gain(&cue), vec![("cue", 4.0)]);
}

#[test]
fn n9_a_bit_perfect_cue_device_gets_no_gain_while_main_does() {
    let mut s = normalizing(3);
    let p = p0(&s);
    s.config.outputs.bit_perfect.push(device("phones"));
    let cue = apply(&mut s, Command::ToggleCue(p)).unwrap();
    assert_eq!(request_gain(&cue), vec![("cue", 0.0)]);
    let start = apply(&mut s, Command::Play(p)).unwrap();
    assert!(request_gain(&start).contains(&("start", 4.0)));
}

/// Gives the first cart of the first page the file `path`, measured like
/// the others (+4 dB).
fn with_cart(s: &mut AppState) -> fp_model::CartId {
    let page = s.cartwall.pages[0].id;
    apply(
        s,
        Command::AssignCartFile {
            page,
            index: 0,
            path: PathBuf::from("/carts/jingle.flac"),
        },
    )
    .unwrap();
    let cart = s.cartwall.pages[0].carts[0].clone();
    let t = s.library.get_mut(cart.track.unwrap()).unwrap();
    t.analyzed = true;
    t.loudness = Some(LoudnessMeasurement {
        integrated_lufs: Some(-20.0),
        true_peak_dbtp: -10.0,
    });
    cart.id
}

#[test]
fn n9_carts_and_their_pre_listen_carry_the_gain() {
    let mut s = normalizing(1);
    let cart = with_cart(&mut s);
    let fire = apply(&mut s, Command::FireCart(cart)).unwrap();
    assert_eq!(request_gain(&fire), vec![("cart", 4.0)]);
    let cue = apply(&mut s, Command::CueCart(cart)).unwrap();
    assert_eq!(request_gain(&cue), vec![("cart-cue", 4.0)]);
}

#[test]
fn n9_with_normalization_off_every_request_is_at_0_db() {
    let mut s = normalizing(3);
    s.config.loudness.normalize = false;
    let p = p0(&s);
    let start = apply(&mut s, Command::Play(p)).unwrap();
    assert!(request_gain(&start).iter().all(|(_, g)| *g == 0.0));
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p fp-model --test loudness n9 2>&1 | tail -5`
Expected: FAIL to compile (`no field normalization on SourceRequest`).

- [ ] **Step 3: Implement**

`command.rs`: `SourceRequest` and `CartRequest` gain, last:

```rust
    /// The fixed gain of this source (loudness normalization spec N9),
    /// decided for the holder that plays it.
    pub normalization: crate::loudness::Normalization,
```

`state.rs` (`use crate::live::Holder;`): each of the three builders takes `holder: Holder` last; `request_from_cue_in` and `request_at` fill `normalization: crate::loudness::normalization_for(self, holder, track),`; `playable_request` passes `holder` through. Doc comments: "`holder` is who plays it: the request carries its normalization (N9)".

`cart_rules.rs`: `fn request(state: &AppState, cart: &Cart, looped: bool, holder: Holder) -> Option<CartRequest>` fills `normalization: crate::loudness::normalization_for(state, holder, track)`; `fire` passes `Holder::CartwallMain`, `cue` passes `Holder::CartwallCue`.

Callers in `reducer.rs` — the holder is `Holder::PlayerMain(id)` for everything that plays on the player's Main output (`Restart`, `Seek`, `previous`, `dsd_hold_continue`, `advance_to`; there use `let holder = Holder::PlayerMain(state.players[i].id);`) and `Holder::PlayerCue(id)` for `cue_entry`, `follow_cue` and `seek_cue`. `pending_start.rs`: `pending_start_at(state, entry, secs, holder)` (callers: `Command::Seek` passes `PlayerMain(id)`, `settle` passes `PlayerMain(p.id)`), `start_request(state, pending, request, holder)` (caller `play` passes `PlayerMain(id)`), `preload_request` uses `Holder::PlayerMain(player.id)`. `availability.rs:34`: `state.playable_request(*e, Holder::PlayerMain(p.id))`. `session.rs:227`: `state.request_at(current, position, Holder::PlayerMain(p.id))`. `fp-app/src/ui/view.rs:225` (`cue_follow_target`): `Holder::PlayerCue(player)`. `tests/history.rs:78`: `Holder::PlayerMain(p)` for its player.

Engine test literals: add `normalization: fp_model::Normalization::default(),` (or import `Normalization`) to each `SourceRequest {` / `CartRequest {` the compiler reports in `crates/fp-engine/tests`.

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test --workspace`
Expected: PASS (the engine ignores the new field until plan 4).

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add -A crates
git commit -m "feat(model): send each source's normalization with its request

Players, CUE, carts and their pre-listen get the gain of the device
their holder plays on, so a pre-listen sounds like on air unless its
own device is bit-perfect (loudness normalization spec N9, Q6).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: The per-track edit (N10)

**Files:**
- Modify: `crates/fp-model/src/command.rs` (`Command::SetTrackLoudness` after `ResetMarkers`), `crates/fp-model/src/error.rs` (`InvalidTrim`), `crates/fp-model/src/reducer.rs` (arm after `ResetMarkers`)
- Test: `crates/fp-model/tests/loudness.rs`

**Interfaces:**
- Produces: `Command::SetTrackLoudness { track: TrackId, excluded: bool, trim_db: f32 }`; `ModelError::InvalidTrim`.

- [ ] **Step 1: Write the failing tests** (append to `tests/loudness.rs`; add `fp_model::ModelError` to the imports)

```rust
#[test]
fn n10_the_edit_is_stored_with_the_track() {
    let mut s = normalizing(1);
    let track = s.library.iter().next().unwrap().id;
    apply(&mut s, Command::SetTrackLoudness { track, excluded: true, trim_db: -2.5 }).unwrap();
    assert_eq!(
        s.library.get(track).unwrap().loudness_edit,
        LoudnessEdit { excluded: true, trim_db: -2.5 }
    );
}

#[test]
fn n10_a_trim_out_of_range_or_not_a_number_is_refused() {
    let mut s = normalizing(1);
    let track = s.library.iter().next().unwrap().id;
    for trim_db in [12.5, -12.5, f32::NAN, f32::INFINITY] {
        let before = s.clone();
        assert_eq!(
            apply(&mut s, Command::SetTrackLoudness { track, excluded: false, trim_db }),
            Err(ModelError::InvalidTrim),
            "{trim_db}"
        );
        assert_eq!(s, before, "a refused command changes nothing");
    }
    // The limits themselves are allowed.
    apply(&mut s, Command::SetTrackLoudness { track, excluded: false, trim_db: 12.0 }).unwrap();
}

#[test]
fn n10_an_unknown_track_is_refused() {
    let mut s = normalizing(1);
    assert_eq!(
        apply(&mut s, Command::SetTrackLoudness { track: TrackId(999), excluded: true, trim_db: 0.0 }),
        Err(ModelError::UnknownTrack(TrackId(999)))
    );
}

#[test]
fn n10_the_track_on_air_can_be_edited() {
    let mut s = normalizing(2);
    let p = p0(&s);
    apply(&mut s, Command::Play(p)).unwrap();
    let track = s.playlists.entry(entries(&s)[0]).unwrap().track;
    assert!(apply(&mut s, Command::SetTrackLoudness { track, excluded: true, trim_db: 0.0 }).is_ok());
}

#[test]
fn n10_the_edit_survives_a_library_round_trip() {
    let mut s = normalizing(1);
    let track = s.library.iter().next().unwrap().id;
    apply(&mut s, Command::SetTrackLoudness { track, excluded: true, trim_db: 1.5 }).unwrap();
    let json = serde_json::to_string(&s.library).unwrap();
    let back: fp_model::Library = serde_json::from_str(&json).unwrap();
    assert_eq!(back.get(track).unwrap().loudness_edit, s.library.get(track).unwrap().loudness_edit);
}
```

(`AppState: PartialEq` exists; `serde_json` is a dev-dependency of `fp-model`.)

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p fp-model --test loudness n10 2>&1 | tail -5`
Expected: FAIL to compile (`SetTrackLoudness`, `InvalidTrim` not found).

- [ ] **Step 3: Implement**

`error.rs`:

```rust
    #[error("the loudness trim must be a number within the maximum trim")]
    InvalidTrim,
```

`command.rs`, after `ResetMarkers`:

```rust
    /// Loudness normalization spec N10: excludes a track from
    /// normalization or trims its gain. Allowed while it plays: it changes
    /// what loads afterwards.
    SetTrackLoudness {
        track: TrackId,
        excluded: bool,
        trim_db: f32,
    },
```

`reducer.rs`, after the `ResetMarkers` arm:

```rust
        Command::SetTrackLoudness {
            track,
            excluded,
            trim_db,
        } => {
            let max = state.config.loudness.max_trim_db;
            let t = state
                .library
                .get_mut(track)
                .ok_or(ModelError::UnknownTrack(track))?;
            if !trim_db.is_finite() || trim_db.abs() > max {
                return Err(ModelError::InvalidTrim);
            }
            t.loudness_edit = crate::loudness::LoudnessEdit { excluded, trim_db };
        }
```

Add `SetTrackLoudness` wherever the compiler reports a non-exhaustive `match` on `Command` (for example in `fp-remote` or `fp-app` command descriptions), treating it like `SetMarker`.

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p fp-model --test loudness n10`
Expected: PASS.

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add -A crates
git commit -m "feat(model): exclude a track from normalization or trim it

The operator's per-track loudness edit is stored with the track, refused
outside the maximum trim, and allowed while the track plays (loudness
normalization spec N10).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: Fixed while it sounds, loaded afterwards, crash recovery (N11, N12, N13)

**Files:**
- Modify: `crates/fp-model/src/player.rs` (`PlayerState` L106-140, `new` L151-171)
- Modify: `crates/fp-model/src/cartwall.rs` (`PlayingCart` L99-103, `Cartwall` L115-140 and its `Default`)
- Modify: `crates/fp-model/src/reducer.rs` (`advance_to`, `dsd_hold_continue`, `TransitionStarted` arm L375-420, `cue_entry`, `reconcile` L1315-1373), `crates/fp-model/src/cart_rules.rs` (`fire`, `cue`), `crates/fp-model/src/session.rs:207-235`
- Modify (compile fixes): `PlayingCart { … }` literals in `crates/fp-app/tests/{track_tags_ui,exit_guard}.rs`, `crates/fp-model/tests/on_air.rs` gain `normalization: Normalization::default()`
- Test: `crates/fp-model/tests/loudness.rs`

**Interfaces:**
- Produces:
  - `PlayerState::on_air: Option<Normalization>` (the current entry's; runtime only), `PlayerState::cue_normalization: Option<Normalization>`
  - `PlayerState::preloaded: Option<(EntryId, f64, Normalization)>` (still `pub(crate)`)
  - `PlayingCart::normalization: Normalization`, `Cartwall::cue_normalization: Option<Normalization>`

- [ ] **Step 1: Write the failing tests** (append to `tests/loudness.rs`)

```rust
fn measure(s: &mut AppState, entry: fp_model::EntryId, lufs: f32) {
    let track = s.playlists.entry(entry).unwrap().track;
    s.library.get_mut(track).unwrap().loudness = Some(LoudnessMeasurement {
        integrated_lufs: Some(lufs),
        true_peak_dbtp: -20.0,
    });
}

fn preloads(actions: &[EngineAction]) -> Vec<f32> {
    actions
        .iter()
        .filter_map(|a| match a {
            EngineAction::Preload { request: Some(r), .. } => Some(r.normalization.gain_db),
            _ => None,
        })
        .collect()
}

/// Only `Preload`s: nothing reaches what is sounding.
fn only_preloads(actions: &[EngineAction]) -> bool {
    actions.iter().all(|a| matches!(a, EngineAction::Preload { .. }))
}

#[test]
fn n11_the_gain_on_air_is_recorded_and_kept() {
    let mut s = normalizing(3);
    let (p, e) = (p0(&s), entries(&s));
    apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(s.player(p).unwrap().on_air.map(|n| n.gain_db), Some(4.0));
    // An analysis result, a loudness-only result, a configuration change
    // and an edit of the track on air: nothing on air changes.
    let track = s.playlists.entry(e[0]).unwrap().track;
    let a = apply(
        &mut s,
        Command::ApplyLoudness {
            track,
            loudness: Some(LoudnessMeasurement { integrated_lufs: Some(-30.0), true_peak_dbtp: -20.0 }),
            version: fp_model::LOUDNESS_VERSION,
        },
    )
    .unwrap();
    let mut config = s.config.clone();
    config.loudness.target_lufs = -12.0;
    let b = apply(&mut s, Command::UpdateConfig(Box::new(config))).unwrap();
    let c = apply(
        &mut s,
        Command::SetTrackLoudness { track, excluded: true, trim_db: 0.0 },
    )
    .unwrap();
    for actions in [&a, &b, &c] {
        assert!(only_preloads(actions), "{actions:?}");
    }
    assert_eq!(s.player(p).unwrap().on_air.map(|n| n.gain_db), Some(4.0));
}

#[test]
fn n11_a_seek_sends_no_new_gain() {
    let mut s = normalizing(3);
    let p = p0(&s);
    apply(&mut s, Command::Play(p)).unwrap();
    s.config.loudness.target_lufs = -10.0;
    let seek = apply(&mut s, Command::Seek(p, 30.0)).unwrap();
    assert!(seek.iter().any(|a| matches!(a, EngineAction::Seek { .. })));
    assert!(request_gain(&seek).iter().all(|(kind, _)| *kind == "preload"));
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    let cue_seek = apply(&mut s, Command::SeekCue(p, 10.0)).unwrap();
    assert!(request_gain(&cue_seek).iter().all(|(kind, _)| *kind == "preload"));
}

#[test]
fn n11_a_scheduled_transition_takes_the_preloaded_gain() {
    let mut s = normalizing(3);
    let (p, e) = (p0(&s), entries(&s));
    apply(&mut s, Command::Play(p)).unwrap();
    measure(&mut s, e[1], -22.0); // the next now wants +6
    let pre = apply(&mut s, Command::SetVolume(p, 1.0)).unwrap();
    assert_eq!(preloads(&pre), vec![6.0]);
    // The model changes its mind after the engine already started it.
    s.library.get_mut(s.playlists.entry(e[1]).unwrap().track).unwrap().loudness = None;
    on_event(&mut s, EngineEvent::TransitionStarted { player: p, entry: e[1] });
    assert_eq!(s.player(p).unwrap().on_air.map(|n| n.gain_db), Some(6.0));
}

#[test]
fn n11_a_transition_to_another_entry_recomputes_its_gain() {
    let mut s = normalizing(3);
    let (p, e) = (p0(&s), entries(&s));
    apply(&mut s, Command::Play(p)).unwrap();
    measure(&mut s, e[2], -24.0);
    // The engine reports an entry that was not the one preloaded.
    on_event(&mut s, EngineEvent::TransitionStarted { player: p, entry: e[2] });
    assert_eq!(s.player(p).unwrap().on_air.map(|n| n.gain_db), Some(8.0));
}

#[test]
fn n11_the_record_is_cleared_when_the_source_ends() {
    let mut s = normalizing(3);
    let p = p0(&s);
    apply(&mut s, Command::Play(p)).unwrap();
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    assert!(s.player(p).unwrap().cue_normalization.is_some());
    apply(&mut s, Command::Stop(p)).unwrap();
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    let player = s.player(p).unwrap();
    assert_eq!((player.on_air, player.cue_normalization), (None, None));
}

#[test]
fn n11_carts_keep_their_gain() {
    let mut s = normalizing(1);
    let cart = with_cart(&mut s);
    apply(&mut s, Command::FireCart(cart)).unwrap();
    apply(&mut s, Command::CueCart(cart)).unwrap();
    assert_eq!(s.cartwall.playing[0].normalization.gain_db, 4.0);
    assert_eq!(s.cartwall.cue_normalization.map(|n| n.gain_db), Some(4.0));
    s.config.loudness.target_lufs = -10.0;
    apply(&mut s, Command::SetVolume(p0(&s), 1.0)).unwrap();
    assert_eq!(s.cartwall.playing[0].normalization.gain_db, 4.0);
    apply(&mut s, Command::CueCart(cart)).unwrap();
    assert_eq!(s.cartwall.cue_normalization, None);
}

#[test]
fn n12_a_measurement_of_the_next_preloads_it_again() {
    let mut s = normalizing(3);
    let (p, e) = (p0(&s), entries(&s));
    apply(&mut s, Command::Play(p)).unwrap();
    let track = s.playlists.entry(e[1]).unwrap().track;
    let a = apply(
        &mut s,
        Command::ApplyLoudness {
            track,
            loudness: Some(LoudnessMeasurement { integrated_lufs: Some(-18.0), true_peak_dbtp: -20.0 }),
            version: fp_model::LOUDNESS_VERSION,
        },
    )
    .unwrap();
    assert_eq!(preloads(&a), vec![2.0]);
}

#[test]
fn n12_a_configuration_change_preloads_the_next_again() {
    let mut s = normalizing(3);
    let p = p0(&s);
    apply(&mut s, Command::Play(p)).unwrap();
    let mut config = s.config.clone();
    config.loudness.target_lufs = -18.0;
    let a = apply(&mut s, Command::UpdateConfig(Box::new(config))).unwrap();
    assert_eq!(preloads(&a), vec![2.0]);
    // Unchanged settings preload nothing.
    let same = s.config.clone();
    assert!(preloads(&apply(&mut s, Command::UpdateConfig(Box::new(same))).unwrap()).is_empty());
}

#[test]
fn n12_an_edit_of_the_next_preloads_it_again() {
    let mut s = normalizing(3);
    let (p, e) = (p0(&s), entries(&s));
    apply(&mut s, Command::Play(p)).unwrap();
    let track = s.playlists.entry(e[1]).unwrap().track;
    let a = apply(
        &mut s,
        Command::SetTrackLoudness { track, excluded: false, trim_db: -1.5 },
    )
    .unwrap();
    assert_eq!(preloads(&a), vec![2.5]);
}

#[test]
fn n12_a_placement_on_a_bit_perfect_device_preloads_the_next_again() {
    let mut s = normalizing(3);
    let p = p0(&s);
    apply(&mut s, Command::Play(p)).unwrap();
    let mut running = device_settings(&s.config.outputs, &device("dac"));
    running.bit_perfect = true;
    let a = on_event(
        &mut s,
        EngineEvent::Placed {
            holder: Holder::PlayerMain(p),
            route: Some(route("dac")),
            device: device("dac"),
            running,
        },
    );
    assert_eq!(preloads(&a), vec![0.0]);
    assert_eq!(s.player(p).unwrap().on_air.map(|n| n.gain_db), Some(4.0), "N17");
}

#[test]
fn n12_a_repeating_entry_takes_the_new_gain_on_its_next_pass() {
    let mut s = normalizing(3);
    let (p, e) = (p0(&s), entries(&s));
    apply(&mut s, Command::SetEntryRepeat(e[0], true)).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    let mut config = s.config.clone();
    config.loudness.target_lufs = -14.0;
    let a = apply(&mut s, Command::UpdateConfig(Box::new(config))).unwrap();
    assert_eq!(preloads(&a), vec![6.0], "the repeat pass is preloaded again");
    assert_eq!(s.player(p).unwrap().on_air.map(|n| n.gain_db), Some(4.0));
}

#[test]
fn n13_crash_recovery_computes_the_gain_again() {
    let mut s = normalizing(3);
    let p = p0(&s);
    apply(&mut s, Command::Play(p)).unwrap();
    s.config.loudness.target_lufs = -14.0;
    let parts = fp_model::RestoreParts {
        config: s.config.clone(),
        library: s.library.clone(),
        playlists: s.playlists.clone(),
        cart_pages: s.cartwall.pages.clone(),
        cartwall_session: s.cartwall.session(),
        ids: s.ids.clone(),
    };
    let (restored, actions) = AppState::restore(parts, &s.sessions(|_| 20.0), "Main");
    assert!(request_gain(&actions).contains(&("load", 6.0)), "{actions:?}");
    assert_eq!(restored.player(p).unwrap().on_air.map(|n| n.gain_db), Some(6.0));
    assert!(
        !actions.iter().any(|a| matches!(a, EngineAction::StartCurrent { .. } | EngineAction::Resume { .. })),
        "nothing goes on air by itself"
    );
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p fp-model --test loudness 2>&1 | tail -5`
Expected: FAIL to compile (`on_air`, `cue_normalization` not found).

- [ ] **Step 3: Implement**

`player.rs`: `PlayerState` gains

```rust
    /// The normalization of the current entry's source, fixed while it
    /// sounds (loudness normalization spec N11); runtime only, never saved.
    pub on_air: Option<crate::loudness::Normalization>,
    /// The same for the CUE source.
    pub cue_normalization: Option<crate::loudness::Normalization>,
```

(`None` in `new`), and `preloaded: Option<(EntryId, f64, crate::loudness::Normalization)>` with its doc "…and the normalization it was preloaded with: a new one is a new preload (N12)".

`cartwall.rs`: `PlayingCart` gains `pub normalization: crate::loudness::Normalization,` (it loses `Eq`: keep `Copy, PartialEq`); `Cartwall` gains `pub cue_normalization: Option<crate::loudness::Normalization>,` (doc "the pre-listen's normalization (N11); runtime only") with `None` in `Default`.

`reducer.rs`:

1. `advance_to`: after `player.dsd = None;` add `player.on_air = Some(request.normalization);`.
2. `dsd_hold_continue`, repeat branch: `p.on_air = Some(request.normalization);` next to `p.preloaded = None;`.
3. `TransitionStarted` arm: at its top, `let preloaded = state.player_index(player).ok().and_then(|i| state.players[i].preloaded);` and a helper used at the end of both successful branches:

```rust
/// N11: a scheduled transition plays what was preloaded, with the gain it
/// was preloaded with; an entry that was not the one preloaded gets its
/// own gain now.
fn started_normalization(
    state: &AppState,
    i: usize,
    entry: EntryId,
    preloaded: Option<(EntryId, f64, crate::loudness::Normalization)>,
) -> Option<crate::loudness::Normalization> {
    match preloaded {
        Some((e, _, n)) if e == entry => Some(n),
        _ => {
            let holder = crate::live::Holder::PlayerMain(state.players[i].id);
            state.request_from_cue_in(entry, holder).map(|r| r.normalization)
        }
    }
}
```

In the first branch (current == entry), after `p.transport = Transport::Playing;` set `state.players[i].on_air = started_normalization(state, i, entry, preloaded);` (compute before taking `p`). In the second branch, after `advance_to(…).is_some()` succeeds, the same.
4. `cue_entry`: after `state.players[i].cue = Some(CueState { … });` add `state.players[i].cue_normalization = Some(request.normalization);` (take the value before `request` moves into the action).
5. `stop_player`: `player.on_air = None;`.
6. `reconcile`, at the top (after `settle`):

```rust
    // N11: a record lives as long as its source.
    for p in &mut state.players {
        if p.current.is_none() {
            p.on_air = None;
        }
        if p.cue.is_none() {
            p.cue_normalization = None;
        }
    }
    if state.cartwall.cue.is_none() {
        state.cartwall.cue_normalization = None;
    }
```

and the preload comparison and record use the triple: `let wanted = request.as_ref().map(|r| (r.entry, r.from_secs, r.normalization));` and `player.preloaded = request.as_ref().map(|r| (r.entry, r.from_secs, r.normalization));`.

`cart_rules.rs`: `fire` pushes `PlayingCart { cart: id, looped: cart.looped, normalization: request.normalization }`; `cue` sets `state.cartwall.cue_normalization = Some(request.normalization);` with `state.cartwall.cue = Some(id);`.

`session.rs` (restore, ~L207-235): collect the `LoadPaused` requests in the existing loop into `let mut loaded: Vec<(PlayerId, Normalization)>` and, after the loop, `for (id, n) in loaded { if let Ok(i) = state.player_index(id) { state.players[i].on_air = Some(n); } }` (the loop borrows `state.players` immutably).

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test --workspace`
Expected: PASS.

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add -A crates
git commit -m "feat(model): keep each source's gain while it sounds

The gain on air, in CUE and of each cart is recorded when its source
starts and never changes until it ends; the next entry is preloaded
again when its gain changes, and crash recovery computes it anew
(loudness normalization spec N11-N13).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 6: The pass runs with normalization off; documentation

**Files:**
- Test: `crates/fp-app/tests/services.rs` (N6b's "runs with `normalize` off", now that the field exists)
- Modify: `docs/technical/persistence.md` (`Track::loudness_edit` in `library.json`; `config.json` `loudness` section with defaults and ranges)
- Modify: the technical page that lists the model's rules (`docs/technical/architecture.md`, section on `fp-model`): `fp_model::loudness` (N7–N13), "settings apply to sources loaded afterwards, never pending"
- Modify: `docs/superpowers/specs/2026-09-26-phase4-bit-perfect-design.md` B1: add "Loudness normalization is off on a bit-perfect device (loudness normalization spec N7, N8): such a source gets no gain."

- [ ] **Step 1: The test**

```rust
#[test]
fn n6b_the_pass_runs_with_normalization_off() {
    let dir = tempfile::tempdir().unwrap();
    let a = wav(dir.path(), "a.wav", 1);
    let mut r = rig_with(&[a], dir, Duration::ZERO, |s| {
        analysed_before_loudness(s);
        s.config.loudness.normalize = false;
    });
    r.run_until("the loudness", |r| only_track(r).loudness_pass.is_some());
    assert!(!r.handle.model.load().config.loudness.normalize);
}
```

Run: `cargo test -p fp-app --features test-hooks --test services n6b_the_pass_runs` — expected PASS at once (plan 2 already does this; the test pins it now that the setting exists). Note in the ledger that it passed on first run and why.

- [ ] **Step 2: The docs** as listed above (persistence: field names, types, defaults, ranges, "absent in older files: defaults").

- [ ] **Step 3: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-app/tests/services.rs docs
git commit -m "docs: describe the normalization settings and per-track edit

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

## Self-review notes

- Spec coverage: N1–N3 → Task 1; N7, N8 → Task 2; N9 → Task 3; N10 → Task 4; N11–N13 → Task 5 (N17's model half: `on_air` kept on a bit-perfect placement, `n12_a_placement_on_a_bit_perfect_device…`); N6b with `normalize` off → Task 6; §11 persistence and bit-perfect spec B1 → Task 6.
- The engine still ignores `normalization` after this plan (plan 4), so on air nothing changes even with `normalize` on; the UI shows nothing yet (plan 5).
