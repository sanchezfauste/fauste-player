# Loudness Normalization — Design Spec

- **Date:** 2026-10-08
- **Status:** Draft. The design in §2 is approved by the maintainer
  (2026-10-08); the open questions in §14 wait for answers.
- **Extends:** [the main design spec](2026-09-25-fauste-player-design.md)
  (§4.3 bus mixer, §6 analysis, §7 persistence, §8.3 main screen, §8.4
  Settings), the [meters spec](2026-09-27-meters-design.md) (M1, M2: the
  K-weighting and true peak are shared), the
  [bit-perfect spec](2026-09-26-phase4-bit-perfect-design.md) (B1 unaltered
  source), the [remote control spec](2026-10-01-remote-control-design.md)
  and the [live settings spec](2026-10-07-live-settings-design.md) (§3
  terms `Holder`, `Placement`; §4.1 `live`; on branch `feat/live-settings`,
  not on `master` yet).
- **Numbering:** rules are `N<n>`. Each rule is testable and gets at least
  one test (§10).
- **Line numbers** are those of `master` at the date above; each reference
  also names the function or field.

---

## 1. Goal

Every track and every cart plays at the same integrated loudness. The
analysis pass measures each file once; the player applies one **fixed gain
per track**, decided when the track is loaded and never changed while it
sounds. Nothing in the audio path reacts to the signal: there is no
automatic gain control and no compressor or limiter.

## 2. Maintainer decisions (binding, 2026-10-08)

| # | Decision |
|---|---|
| D1 | A fixed gain per track, measured during analysis. Never an AGC or a compressor. |
| D2 | The analysis pass measures integrated loudness (EBU R128 / ITU-R BS.1770, gated) and true peak for every file, reusing the R128 code the meter already uses where possible. The result is stored in the analysis cache; `ANALYSIS_VERSION` is bumped. A file that fails measurement degrades to "not measured" with a log line. |
| D3 | The gain is a pure function in `fp-model`: `target − measured + trim`, limited so that measured true peak + gain ≤ ceiling, and gain ≤ maximum boost. It is fixed when the track is loaded or started and never changes while it sounds. |
| D4 | The engine applies one more static per-source gain, multiplied with the fader. It arrives through `BusCommand`; nothing allocates on the device callback (CLAUDE.md rule 5). |
| D5 | A new configuration section with `normalize`, `target_lufs` (−16), `true_peak_ceiling_dbtp` (−1) and `max_boost_db` (12), each validated with a range. Changes apply to tracks loaded afterwards. |
| D6 | Per track: exclude it from normalization, or a manual trim in dB, stored with the track's other edits (its markers) and edited where those are edited. |
| D7 | A track not measured yet (it can play before its analysis) plays at 0 dB, and the player shows a "not normalized" indicator. Nothing is adjusted mid-track. |
| D8 | Bit-perfect and native DSD win: no normalization on such a device, and the interface says so. |
| D9 | The player shows the applied gain; Settings has a Loudness section; the per-track controls. Remote API and MIDI follow the existing patterns (a state readout at least). Every new string in all 11 locales. |
| D10 | Rejected: writing normalized copies of files; live AGC. |

## 3. Terms

| Term | Meaning |
|---|---|
| **Integrated loudness** | ITU-R BS.1770-5 programme loudness of the whole file, in LUFS: K-weighted mean square over 400 ms blocks with 75 % overlap, gated at −70 LUFS (absolute) and −10 LU below the ungated mean (relative). |
| **True peak** | The largest magnitude of the 4× oversampled signal (BS.1770-5 Annex 2), in dBTP. |
| **Measurement** | `LoudnessMeasurement { integrated_lufs: Option<f32>, true_peak_dbtp: f32 }`. `integrated_lufs` is `None` when no block passes the absolute gate (a silent file). |
| **Track loudness** | What the model knows of a track: `Track::loudness: Option<LoudnessMeasurement>`; `None` is **not measured** (not analysed yet, analysed by an earlier version, or the measurement failed). |
| **Loudness edit** | `Track::loudness_edit: LoudnessEdit { excluded: bool, trim_db: f32 }`, the operator's per-track setting. Default: not excluded, 0 dB. |
| **Holder** | As in the live settings spec §3: `PlayerMain(PlayerId)`, `PlayerCue(PlayerId)`, `CartwallMain`, `CartwallCue`. |
| **Normalization** | What a source gets, decided by N7: `Normalization { gain_db: f32, status: NormalizationStatus }`. |
| **Status** | `NormalizationStatus::{Applied { limited: Option<GainLimit> }, Off, Excluded, NotMeasured, Silent, BitPerfect}`; `GainLimit::{TruePeak, MaxBoost}`. Every status but `Applied` has `gain_db = 0`. |

## 4. Configuration

A new section `Config::loudness: LoudnessConfig` (`crates/fp-model/src/config.rs`),
`#[serde(default)]`, so a configuration file without it loads with the
defaults (lenient loading).

| Field | Type | Default | Range in `Config::validate` | Meaning |
|---|---|---|---|---|
| `normalize` | `bool` | `false` (Q1) | — | Normalization on or off. |
| `target_lufs` | `f32` | −16.0 | −36.0 ..= −6.0 | Integrated loudness every track is brought to. |
| `true_peak_ceiling_dbtp` | `f32` | −1.0 | −12.0 ..= 0.0 | No gain may take the measured true peak above this. |
| `max_boost_db` | `f32` | 12.0 | 0.0 ..= 24.0 | The largest gain N7 may give (cuts are not limited). |
| `max_trim_db` | `f32` | 12.0 | 0.0 ..= 24.0 | The per-track trim is limited to ±this (N10). |

- **N1** `Config::validate` clamps each value into its range with a
  `ConfigWarning` naming the field (`loudness.target_lufs`, …), as for
  every other section; a non-finite value takes the default.
- **N2** `restore_defaults` gains `SettingsSection::Loudness`, which resets
  `config.loudness`.
- **N3** A loudness change never waits and never touches a sounding
  source. In the terms of the live settings spec it produces no pending
  item and no `EngineAction::UpdateSettings`: the gain travels in each
  source request (§6), so the next request built after `UpdateConfig`
  carries it (N12). On `master` before the live settings work lands,
  `restart_pending` (`crates/fp-model/src/restart.rs`) gains no reason for
  it either.

The meter's own `meter.loudness_target_lufs` (−23, the readout's green
band) stays a separate value (Q4).

## 5. Measurement (analysis)

- **N4** `analyze_file_cancellable` (`crates/fp-analysis/src/analyze.rs`)
  feeds every decoded stereo block, besides the envelope, to a
  `LoudnessMeter`: per channel the K-weighting of the meters
  (`crates/fp-engine/src/kweight.rs`, `KWeighting`) and the true-peak
  interpolator (`crates/fp-engine/src/truepeak.rs`, `TruePeak`), and the
  BS.1770 gating over 400 ms blocks with a 100 ms hop. What is measured is
  what the players play: the stereo downmix `FileDecoder` gives, at the
  file's own rate, both channels weighted 1.0. A damaged stretch that ends
  decoding early is measured up to that point, as the envelope is.
- **N5** Results:
  - At least one gating block passes the absolute gate:
    `integrated_lufs = Some(..)`.
  - None does (digital silence or near it): `integrated_lufs = None`
    (status `Silent` in N7).
  - A file shorter than one gating block (400 ms; short effects on carts)
    is measured as one block of its own length, then gated the same way
    (Q7).
  - A non-finite result, or a panic inside the meter, gives no measurement
    (`None` in `TrackAnalysis::loudness`) and a `tracing::warn!` with the
    path; the rest of the analysis is kept (CLAUDE.md rule 9).
- **N6** `TrackAnalysis` gains `#[serde(default)] loudness:
  Option<LoudnessMeasurement>`; `ANALYSIS_VERSION` goes from 8 to 9 ("9:
  integrated loudness and true peak"). The measurement does not depend on
  `AnalysisSettings` or on `LoudnessConfig`, so the cache key does not
  change apart from the version, and changing the target never makes the
  library be analysed again. `Track::apply_analysis` stores the
  measurement in `Track::loudness`; it never touches `loudness_edit`.

  The DSP shared by the engine and the analysis (`kweight.rs`,
  `truepeak.rs`) moves to `fp-decode` (module `fp_decode::loudness`; both
  crates already depend on it and it denies `clippy::indexing_slicing`),
  and `fp-engine` uses it from there (Q5). The gating integrator is new
  code in the same module; it allocates only off the device callback (it
  runs in the analysis pool).

  Tracks analysed by version 8 or earlier become outdated
  (`services::outdated`) and keep `loudness = None` until the operator
  analyses them again (the start-up notice and **Settings → Analysis →
  Analyse outdated**), as for every earlier version bump (Q3).

## 6. The gain (model)

- **N7** `fp_model::loudness::normalization(config: &LoudnessConfig,
  track: &Track, bit_perfect: bool) -> Normalization`, in this order:

  | When | Status | `gain_db` |
  |---|---|---|
  | `bit_perfect` | `BitPerfect` | 0 |
  | `!config.normalize` | `Off` | 0 |
  | `track.loudness_edit.excluded` | `Excluded` | 0 |
  | `track.loudness == None` | `NotMeasured` | 0 |
  | `integrated_lufs == None` | `Silent` | 0 |
  | otherwise | `Applied { limited }` | `g` below |

  `wanted = target_lufs − integrated_lufs + trim_db`;
  `g = min(wanted, true_peak_ceiling_dbtp − true_peak_dbtp, max_boost_db)`.
  `limited` names the bound that won when it is lower than `wanted`
  (`TruePeak` before `MaxBoost` on a tie), else `None`. The ceiling also
  cuts a track whose own true peak is above it (Q2). `gain_db` is rounded
  to 0.01 dB so that equal inputs give equal requests.
- **N8** `bit_perfect` for a holder is whether the device it plays on is
  bit-perfect: the running settings of its placement
  (`live.devices[live.placement[holder]].bit_perfect`) once the engine has
  reported it (live settings L3); before that, or without the live
  settings work, whether the holder's configured route names a device in
  `config.outputs.bit_perfect` (no route: the default output, not
  bit-perfect). A native DSD stream only reaches a bit-perfect device
  (`OutputsConfig::dsd_output_for`), so N7 never gives it a gain and the
  engine's DSD path (`BusCommand::DsdMode`: no gain touches the words)
  needs no change.
- **N9** `SourceRequest` and `CartRequest`
  (`crates/fp-model/src/command.rs`) gain `normalization: Normalization`.
  Every place that builds one computes it for the holder that will play
  it: `Preload`, `StartCurrent`, `Crossfade`, `LoadPaused` (holder
  `PlayerMain`), `StartCue` (`PlayerCue`; a CUE seek reopens the same
  source and keeps its gain, N15),
  `StartCart` (`CartwallMain`), `StartCartCue` (`CartwallCue`).
  `AppState::request_from_cue_in` and `request_at` take the holder.
- **N10** `Command::SetTrackLoudness { track, excluded: bool, trim_db: f32
  }` stores the track's loudness edit. It is refused with
  `ModelError::InvalidTrim` when `trim_db` is not finite or
  `|trim_db| > max_trim_db`, and with `UnknownTrack` for an unknown track.
  It is allowed while the track plays (it changes what loads afterwards,
  N12).
- **N11** Fixed while it sounds. `PlayerState` gains, runtime only (never
  saved, like `dsd`): `on_air: Option<Normalization>` for the current
  entry and `cue_normalization: Option<Normalization>` for the CUE; the
  cartwall's `PlayingCart` and its CUE keep theirs the same way. They are
  set from the request the model sends (`StartCurrent`, `Crossfade`,
  `LoadPaused`, `StartCue`, `StartCart`, `StartCartCue`) or, for a
  scheduled transition, copied from the preloaded request on
  `TransitionStarted`, and cleared when the source ends. Nothing else
  changes them: not an analysis result, a configuration change, a loudness
  edit, nor a pending device change applied while the track is paused.
- **N12** Loaded afterwards. `PlayerState::preloaded` becomes `Option<(
  EntryId, f64, Normalization)>`, so `reconcile` sends a new `Preload` when
  the normalization of the entry waiting to play changes (an analysis that
  measured it, `UpdateConfig`, `SetTrackLoudness`, a placement change of
  `PlayerMain`). The preloaded source is not audible, so this is "loaded
  afterwards". A source already started keeps its gain (N11).
- **N13** Crash recovery (`LoadPaused`) and a session restore compute the
  normalization again from the current configuration and track: they load
  the track. Nothing starts by itself (CLAUDE.md rule 10).

## 7. Engine

- **N14** `BusCommand::Attach` (`crates/fp-engine/src/mixer.rs`) gains
  `gain: f32`, the linear value of `normalization.gain_db`
  (`10^(dB/20)`), computed by the engine thread. The slot keeps it as a
  plain field; the per-sample gain becomes
  `fade × pause × volume_now × gain`. No new command, no allocation, no
  lock: the value is copied with the rest of `Attach`, which is already
  sent off the callback. The fader stays clamped to 0–1; the
  normalization gain is not (a boost is above 1).
- **N15** The engine keeps each source's normalization with the source:
  a source opened again (a seek, a rate change, a route move in the live
  settings spec L12/L16, a device reconnect) is attached with the same
  gain. Only a new request brings a new gain.
- **N16** Meters measure after gain (meters spec M1), so the meter and its
  loudness readout show the normalized level. The unaltered-source check
  (bit-perfect spec B1) sees `gain ≠ 1` as any other gain; by N7 a source
  on a bit-perfect device has `gain = 1`.
- **N17** A source paused on a device that becomes bit-perfect meanwhile
  (live settings L12, "Bit-perfect on") keeps its gain (N11): the **BP**
  badge stays off for it, as the unaltered check reports, and the next
  track plays at 0 dB.

## 8. User interface

### 8.1 Player

- **N18** The player header shows a gain badge next to the **BP** badge,
  for the current entry's `on_air` normalization while the player is not
  stopped:

  | Status | Badge | Tooltip |
  |---|---|---|
  | `Applied { limited: None }` | "−4.2 dB" (signed, one decimal; "0.0 dB" for a gain under 0.05 dB) | "Normalized to {target} LUFS: measured {lufs} LUFS, trim {trim} dB." |
  | `Applied { limited: Some(_) }` | as above | adds "Limited by the true-peak ceiling ({ceiling} dBTP)." or "Limited by the maximum boost ({max} dB)." |
  | `NotMeasured` | "Not normalized" (amber) | "Loudness not measured yet: this track plays unchanged." |
  | `BitPerfect` | "Not normalized" (muted) | "Bit-perfect output: tracks play unaltered." |
  | `Excluded` | "Not normalized" (muted) | "This track is excluded from normalization." |
  | `Off`, `Silent` | none | — |

  `view::gain_badge(&Normalization, &LoudnessConfig, &Track) -> Option<GainBadge>`
  is a pure function in `crates/fp-app/src/ui/view.rs`; `player.rs` paints it
  with the existing badge helper.
- **N19** The track tooltip (operator feedback 4, Q9) adds a line
  "Loudness: {lufs} LUFS, true peak {tp} dBTP" when measured, "Loudness:
  not measured" otherwise, and "Excluded from normalization" or "Trim
  {trim} dB" when the edit is not the default.

### 8.2 Per-track controls

- **N20** The waveform panel's context menu (player column and CUE window,
  `crates/fp-app/src/ui/wave_panel.rs`, `edit_markers`), where the markers
  are edited, gains a **Loudness…** item. It opens a small popup with the
  measurement (or "not measured"), an **Exclude from normalization**
  checkbox and a **Trim** slider (−`max_trim_db` ..= +`max_trim_db`, 0.5 dB
  steps, double-click resets to 0). Each change sends `SetTrackLoudness`.
  The popup reads only the snapshot (rule 8).
- **N21** The cart edit dialog (where a cart's file and properties are
  edited) shows the same two controls for the cart's track.

### 8.3 Settings

- **N22** A **Loudness** tab in Settings
  (`crates/fp-app/src/ui/settings/loudness.rs`) with: **Normalize
  loudness** (toggle), **Target** (slider, LUFS), **True-peak ceiling**
  (slider, dBTP), **Maximum boost** (slider, dB), **Maximum trim**
  (slider, dB), a hint "Changes apply to tracks loaded afterwards; what is
  playing keeps its gain.", a line counting tracks not measured
  (`loudness_unmeasured(state)`: analysed and playable with
  `loudness == None`) with a pointer to **Analyse outdated**, and
  **Restore defaults** (N2). Slider ranges are those of N1.

### 8.4 Strings (en-US source; es-ES by hand; the other nine locales AI-translated)

| Key (indicative) | Text |
|---|---|
| `settings-tab-loudness` | "Loudness" |
| `settings-normalize` | "Normalize loudness" |
| `settings-hint-normalize` | "Plays every track and cart at the same integrated loudness with a fixed gain per track." |
| `settings-normalize-target` | "Target" |
| `settings-normalize-ceiling` | "True-peak ceiling" |
| `settings-normalize-max-boost` | "Maximum boost" |
| `settings-normalize-max-trim` | "Maximum trim" |
| `settings-hint-normalize-apply` | "Changes apply to tracks loaded afterwards; what is playing keeps its gain." |
| `settings-normalize-unmeasured` | "{ $count } tracks not measured yet" |
| `badge-gain` | "{ $gain } dB" |
| `badge-not-normalized` | "Not normalized" |
| `tip-gain-applied` | "Normalized to { $target } LUFS: measured { $lufs } LUFS, trim { $trim } dB." |
| `tip-gain-limited-peak` | "Limited by the true-peak ceiling ({ $ceiling } dBTP)." |
| `tip-gain-limited-boost` | "Limited by the maximum boost ({ $max } dB)." |
| `tip-not-measured` | "Loudness not measured yet: this track plays unchanged." |
| `tip-not-normalized-bp` | "Bit-perfect output: tracks play unaltered." |
| `tip-not-normalized-excluded` | "This track is excluded from normalization." |
| `track-loudness` | "Loudness: { $lufs } LUFS, true peak { $tp } dBTP" |
| `track-loudness-unmeasured` | "Loudness: not measured" |
| `track-loudness-excluded` | "Excluded from normalization" |
| `track-loudness-trim` | "Trim { $trim } dB" |
| `wave-loudness` | "Loudness…" |
| `loudness-exclude` | "Exclude from normalization" |
| `loudness-trim` | "Trim" |

The existing meter keys `settings-loudness` and `settings-loudness-target`
keep their meaning (the meter readout); the new keys avoid them.

## 9. Remote control and MIDI

Following the remote spec, the API reads state and edits tracks; station
configuration stays a non-goal, so `config.loudness` is not exposed.

- **N23** `TrackDto` (`crates/fp-remote/src/dto.rs`) gains `loudness:
  Option<LoudnessDto { integrated_lufs: Option<f32>, true_peak_dbtp: f32
  }>` and `loudness_edit: LoudnessEditDto { excluded, trim_db }`.
  `PlayerDto` gains `normalization: Option<NormalizationDto { gain_db,
  status }>` for the entry on air (`status` as a lower-case name:
  `applied`, `limited_true_peak`, `limited_max_boost`, `off`, `excluded`,
  `not_measured`, `silent`, `bit_perfect`); `PlayingCartDto` the same for
  its cart. They reach clients through the existing state and SSE events.
- **N24** `PUT /tracks/{id}/loudness` with `{ "excluded": bool, "trim_db":
  number }` sends `SetTrackLoudness`, beside `PUT
  /tracks/{id}/markers/{kind}`, with the same authentication, the same
  error mapping (`ApiError::from_model`: an unknown track is not found,
  `InvalidTrim` is refused with the model's reason) and an
  `Edit::SetTrackLoudness` in `crates/fp-remote/src/api.rs`. OSC gains
  nothing (it has no track editing).
- **N25** MIDI gains nothing: MIDI actions are per-player transport and
  faders (`MidiAction`), and the gain is fixed per track.

## 10. Testing

Engine tests use the `Offline` backend and explicit `now: Instant`; nothing
sleeps.

| Layer | Tests |
|---|---|
| `fp-model` (`tests/restore.rs`, new `tests/loudness_config.rs`) | N1: each field clamped with a warning, non-finite → default, a file without `loudness` loads the defaults. N2: Restore defaults resets the section only. N3: `UpdateConfig` with loudness changes only emits `Preload`s (no pending item, no restart reason). |
| `fp-model` (new `tests/loudness.rs`) | N7: one test per row of the table, the true-peak and max-boost limits (and the tie), a cut not limited by `max_boost_db`, trim added before the limits, rounding. N8: bit-perfect from placement, from the configured route, default output not bit-perfect. N9: every request kind carries the holder's normalization (CUE on a bit-perfect Cue device gets 0 dB while Main gets a gain). N10: refused out of range and non-finite; allowed while playing. N11: an analysis result, a config change and an edit leave `on_air` and the engine untouched while playing; `TransitionStarted` copies the preloaded value. N12: each of the four causes re-preloads the next. N13: `LoadPaused` recomputes. |
| `fp-model` (`tests/analysis.rs`) | N6: `apply_analysis` stores the measurement and keeps the edit; a version-8 track is not measured. |
| `fp-decode` (`tests/loudness.rs`) | N4: EBU Tech 3341 / 3342 cases that apply to a file measurement (1 kHz sine at −23 dBFS reads −23.0 LUFS ±0.1 at 44.1 and 48 kHz; the gating cases); true peak of the BS.1770 test signals; the moved K-weighting keeps its existing tests. N5: silence → `None`; a 200 ms file measured as one block; non-finite input does not panic. |
| `fp-analysis` (`tests/analyze.rs`) | N4–N6: a generated WAV gets a measurement; a cached v9 entry round-trips it; a file truncated mid-stream is measured up to the damage. |
| `fp-engine` (`tests/mixer.rs`, `audio_path_audit.rs`) | N14: a slot attached with +6 dB outputs twice the amplitude and the meter reads it; a fader move keeps the gain; the audit covers `Attach` with a gain (no allocation on the callback). N15: a seek and a forced rate change keep the gain. N16: gain 1 on a bit-perfect bus stays bit-exact; gain ≠ 1 clears the unaltered flag. |
| `fp-app` kittests (new `tests/loudness_ui.rs`) | N18: each badge and its accessible name; N19: tooltip lines; N20: the Loudness… popup sends `SetTrackLoudness`; N21: the cart dialog does too; N22: the Settings tab updates the config and its sliders keep the ranges. |
| `fp-remote` (`tests/api.rs`) | N23: the DTO fields for each status; N24: the endpoint, a refused trim and an unknown track. |

## 11. Documentation to update

- `README.md`: feature list (loudness normalization).
- `docs/user/settings.md` (the Loudness tab), `docs/user/players.md` (the
  gain badge), `docs/user/markers-and-mixing.md` (the Loudness… popup:
  exclude and trim), `docs/user/cartwall.md` (the cart dialog),
  `docs/user/bit-perfect.md` (no normalization on bit-perfect devices),
  `docs/user/remote-control.md` (fields and endpoint).
- `docs/technical/analysis.md` (measurement, version 9, outdated tracks),
  `docs/technical/audio-engine.md` (`Attach` gain, gain kept on reopen),
  `docs/technical/persistence.md` (`Track::loudness`,
  `Track::loudness_edit`), `docs/technical/remote-api.md`,
  `docs/technical/ui.md` (badge, popup, Settings tab),
  `docs/technical/decoding.md` (the `loudness` module).
- Main spec §4.3 (the per-source gain), §6 (the measurement), §8.3 (the
  badge), §8.4 (the Loudness tab); meters spec M1 (the shared module);
  bit-perfect spec B1 (normalization is off on such devices).
- Every locale file (§8.4).

## 12. Out of scope

- Writing normalized copies of files, and any live AGC, compressor or
  limiter (D10).
- Album (group) normalization, and loudness range (LRA).
- Normalizing over the play range only (cue-in to cue-out): the whole file
  is measured.
- Reading or writing ReplayGain or R128 gain tags.
- A track-table column for loudness or gain.
- Exporting per-track edits with playlists or cart pages.
- Normalization of the test tone.

## 13. Spec lines that change

- Main spec §4.3 (bus mixer): after the volume bullet add "Each source also
  has a fixed normalization gain from its request (loudness normalization
  spec N14), multiplied with the volume; it never changes while the source
  exists."
- Main spec §6 (analysis): add "Analysis also measures integrated loudness
  and true peak (loudness normalization spec N4–N6)."
- Main spec §8.4: add the Loudness tab.

## 14. Open questions for the maintainer

- **Q1** `normalize` defaults to **off**, so an upgrade changes nothing on
  air until the operator turns it on (and an existing library is not
  measured yet, Q3). On would normalize new installations at once. Off or
  on?
- **Q2** N7 follows D3 literally: the ceiling also **cuts** a track whose
  own true peak is above it (a −18 LUFS track with a +0.5 dBTP peak gets
  −1.5 dB with a −1 dBTP ceiling). The alternative is to let the ceiling
  only limit boosts (`g = min(wanted, max(0, ceiling − tp), max_boost)`
  when `wanted > 0`). Keep the literal rule?
- **Q3** Bumping `ANALYSIS_VERSION` makes the whole library outdated, and
  by the existing policy outdated tracks are analysed again only when the
  operator asks (start-up notice, Settings → Analysis). Until then they are
  not measured and play at 0 dB with the indicator. Keep this, or run a
  loudness-only pass automatically in the background (decode only, without
  replacing markers)?
- **Q4** The meter's loudness target (`meter.loudness_target_lufs`, −23)
  stays separate from `loudness.target_lufs` (−16). Should the meter's
  green band follow the normalization target while normalization is on?
- **Q5** The shared K-weighting and true-peak code moves to `fp-decode`
  (module `loudness`). A small new crate (`fp-dsp`) would be cleaner but
  adds one to the workspace. Which?
- **Q6** The CUE (player and cart pre-listen) is normalized too, for its
  own Cue device (N9), so pre-listening sounds as on air. Confirm.
- **Q7** A file shorter than 400 ms is measured as one block of its own
  length (N5), outside BS.1770 (which needs one full block). Confirm, or
  leave such files not measured (0 dB)?
