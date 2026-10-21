# Loudness Normalization — Design Spec

- **Date:** 2026-10-08
- **Status:** Approved for planning. The design in §2 and the answers in
  §14 are approved by the maintainer (2026-10-08).
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
| D2 | The analysis pass measures integrated loudness (EBU R128 / ITU-R BS.1770, gated) and true peak for every file, reusing the R128 code the meter already uses where possible. The result is stored in the analysis cache. A file that fails measurement degrades to "not measured" with a log line. A background loudness-only pass measures the tracks that have no measurement yet (D12). |
| D3 | The gain is a pure function in `fp-model`: `target − measured + trim`, limited so that a boost never takes the measured true peak above the ceiling, and gain ≤ maximum boost. A cut is never reduced by the ceiling. It is fixed when the track is loaded or started and never changes while it sounds. |
| D4 | The engine applies one more static per-source gain, multiplied with the fader. It arrives through `BusCommand`; nothing allocates on the device callback (CLAUDE.md rule 5). |
| D5 | A new configuration section with `normalize`, `target_lufs` (−16), `true_peak_ceiling_dbtp` (−1) and `max_boost_db` (12), each validated with a range. Changes apply to tracks loaded afterwards. Basis: the measurement is ITU-R BS.1770 integrated loudness and true peak; −16 LUFS follows the AES recommendations for streaming and online distribution (AES TD1008, 2021, which supersedes TD1004.1.15-10: −16 to −20 LUFS for streaming), not broadcast (EBU R128 −23 LUFS, ATSC A/85 −24 LKFS); −1 dBTP is the EBU R128 maximum true peak, also used by AES (a lower ceiling such as −2 dBTP for low-bit-rate encoders is common practice, not from the standards); the 12 dB maximum boost and maximum trim are product choices with no standard behind them. |
| D6 | Per track: exclude it from normalization, or a manual trim in dB, stored with the track's other edits (its markers) and edited where those are edited. |
| D7 | A track not measured yet (it can play before its analysis) plays at 0 dB, and the player shows a "not normalized" indicator. Nothing is adjusted mid-track. |
| D8 | Bit-perfect and native DSD win: no normalization on such a device, and the interface says so. |
| D9 | The player shows the applied gain; Settings has a Loudness section; the per-track controls. Remote API and MIDI follow the existing patterns (a state readout at least). Every new string in all 11 locales. |
| D10 | Rejected: writing normalized copies of files; live AGC. |
| D11 | `normalize` is on for new installations (§14, Q1, revised in itr06 I27); an existing configuration keeps its current setting (every saved config already stores the value). No migration. |
| D12 | A background loudness-only pass (N6b) measures tracks with no loudness measurement, with no full re-analysis; `ANALYSIS_VERSION` is not bumped (N6). |
| D13 | The shared K-weighting and true-peak code moves to a new crate `fp-dsp` (N6). |
| D14 | CUE and pre-listen (player and cart) are normalized exactly like on air (N9). |
| D15 | Files shorter than 400 ms are not measured: 0 dB and the "not normalized" badge (N5). |

## 3. Terms

| Term | Meaning |
|---|---|
| **Integrated loudness** | ITU-R BS.1770-5 programme loudness of the whole file, in LUFS: K-weighted mean square over 400 ms blocks with 75 % overlap, gated at −70 LUFS (absolute) and −10 LU below the ungated mean (relative). |
| **True peak** | The largest magnitude of the 4× oversampled signal (BS.1770-5 Annex 2), in dBTP. |
| **Measurement** | `LoudnessMeasurement { integrated_lufs: Option<f32>, true_peak_dbtp: f32 }`. `integrated_lufs` is `None` when no block passes the absolute gate (a silent file). |
| **Track loudness** | What the model knows of a track: `Track::loudness: Option<LoudnessMeasurement>`; `None` is **not measured** (not analysed yet, not reached by the loudness-only pass yet, shorter than 400 ms, or the measurement failed). |
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
| `normalize` | `bool` | `false` (D11) | — | Normalization on or off. |
| `target_lufs` | `f32` | −16.0 | −36.0 ..= −6.0 | Integrated loudness every track is brought to. |
| `true_peak_ceiling_dbtp` | `f32` | −1.0 | −12.0 ..= 0.0 | A boost may not take the measured true peak above this; it never cuts a track (N7). |
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
  carries it (N12).

The meter's own `meter.loudness_target_lufs` (−23, the readout's green
band) stays a separate value, whatever the normalization target (Q4).

## 5. Measurement (analysis)

- **N4** `analyze_file_cancellable` (`crates/fp-analysis/src/analyze.rs`)
  feeds every decoded stereo block, besides the envelope, to a
  `LoudnessMeter` of `fp-dsp`: per channel the K-weighting of the meters
  (`KWeighting`, moved from `crates/fp-engine/src/kweight.rs`) and the
  true-peak interpolator (`TruePeak`, moved from
  `crates/fp-engine/src/truepeak.rs`), and the
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
    is not measured (Q7): `loudness = None`, no warning (it is expected),
    so the track plays at 0 dB with the "not normalized" badge (N7,
    `NotMeasured`). The loudness-only pass records that it has been
    tried (N6b) and does not try it again.
  - A non-finite result, or a panic inside the meter, gives no measurement
    (`None` in `TrackAnalysis::loudness`) and a `tracing::warn!` with the
    path; the rest of the analysis is kept (CLAUDE.md rule 9).
- **N6** `TrackAnalysis` gains `#[serde(skip)] loudness:
  Option<LoudnessMeasurement>` and `#[serde(skip)] loudness_pass:
  Option<u32>`, the version of the loudness pass that ran on the file
  (`LOUDNESS_VERSION`, 1). The cache entry keeps its own bytes unchanged
  (postcard is not self-describing, so a field added to the entry would
  break the entries already written); the two fields travel in a
  separate postcard record **appended** after the entry (the loudness
  tail). A missing, unreadable or non-finite tail decodes as
  `loudness = None`, `loudness_pass = None` and leaves the entry itself
  valid. **`ANALYSIS_VERSION` is
  not bumped** (it stays 8): the measurement is additive, it changes no
  marker or envelope, so no entry becomes outdated and the library is not
  offered a full re-analysis. A cache entry written before this work
  has no tail, and the loudness-only pass (N6b) appends one in place under
  the same key. An analysis loaded from such an entry clears the track's
  `loudness` and `loudness_pass` (the measurement the track held may
  belong to other content), so the pass measures the file again. A
  measurement that is not finite is dropped by the model rules
  (`apply_analysis`, `apply_loudness`): JSON cannot hold it. A later
  change of the measurement algorithm bumps `LOUDNESS_VERSION` only, which
  re-runs the loudness-only pass and nothing else. A full analysis (new
  file, or **Analyse outdated** for a real `ANALYSIS_VERSION` change) also
  measures loudness and sets `loudness_pass = Some(LOUDNESS_VERSION)`. The
  measurement does not depend on `AnalysisSettings` or on `LoudnessConfig`,
  so the cache key does not change and changing the target never makes
  anything be analysed again. `Track::apply_analysis` stores the
  measurement in `Track::loudness`; it never touches `loudness_edit`, and
  a loudness-only result never touches markers.

  The DSP shared by the engine and the analysis (`kweight.rs`,
  `truepeak.rs`) moves to a **new crate `fp-dsp`** (Q5): pure signal
  processing, no I/O, no dependency on other workspace crates, `unsafe`
  forbidden and `clippy::indexing_slicing` denied like `fp-engine`. It
  holds the K-weighting, the true-peak interpolator and the gating
  integrator (`LoudnessMeter`, new code; it allocates only off the device
  callback, in the analysis pool). `fp-engine` (the meters) and
  `fp-analysis` depend on it; `fp-decode` does not change. The workspace
  `Cargo.toml` members, `deny.toml` if needed and CLAUDE.md "Where things
  are" gain the crate.

- **N6b** The **loudness-only pass** (Q3). When the library is loaded and
  after each import, the services thread lists the playable tracks with
  `loudness_pass != Some(LOUDNESS_VERSION)` and queues them in the analysis
  pool's own **loudness lane**, a separate queue below the urgent and the
  normal ones:
  - Priority: below every normal analysis job (new files, the operator's
    **Analyse**, **Analyse outdated**), so it never delays what the
    operator asked for. It starts only when the normal queue is empty and
    runs on a bounded share of the analysis pool (at most all but one
    worker, at least one; the share is derived from the pool size, not a
    new constant), so the UI and the playback decode path keep their
    headroom. A track that is loaded in a player or started is not
    interrupted by it; a job in progress is cancellable like any analysis.
    Running loudness jobs are not preempted by a new normal job; the free
    worker takes it. Queuing or applying a full analysis of a track
    cancels that track's loudness job, and an answer for a track no longer
    waiting for one is dropped, so a stale answer never overwrites the
    analysis's measurement.
  - Work: decode the file once (the same `FileDecoder` and downmix as N4)
    and feed only the `LoudnessMeter`: no envelope, no marker detection,
    no cover or tag work. It never replaces markers or any operator edit.
  - Storage: the result (a `LoudnessMeasurement`, or none for a file too
    short or too quiet) and `loudness_pass = Some(LOUDNESS_VERSION)` are
    written as the tail of the existing cache entry (same key, atomic
    replace). A file with no cache entry (the cache was cleared, or is
    off) is measured all the same and the result is recorded only in the
    model; its next full analysis writes the entry. The services thread
    sends the result to the model as `Command::ApplyLoudness` (loudness
    fields only); the
    model stores `Track::loudness` and, by N12, re-preloads the entry
    waiting to play if its normalization changes. A source on air is never
    touched (N11).
  - A failure or a file shorter than 400 ms is recorded in the model with
    `loudness_pass = Some(..)` and `loudness = None`, so it is not retried
    on every start; a log line (warn for a failure) names the path. Only
    a measurement (or a short or silent file) is written into the cache:
    a failure may be transient (a share that dropped mid-read), so the
    cache never records it and a **Re-analyse** measures the file again.
  - The pass does not depend on `normalize`: it runs also while
    normalization is off, so that turning it on finds the library
    measured. It needs no setting of its own: it is cheap, low priority and
    always on.

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

  `wanted = target_lufs − integrated_lufs + trim_db`. The ceiling and the
  maximum boost only limit boosts (Q2): when `wanted ≤ 0`, `g = wanted`
  and `limited = None`. When `wanted > 0`,
  `g = min(wanted, max(0, true_peak_ceiling_dbtp − true_peak_dbtp),
  max_boost_db)`. `limited` names the bound that won when it is lower than
  `wanted` (`TruePeak` before `MaxBoost` on a tie), else `None`. So the
  ceiling never takes a track below `target − measured + trim`: a track
  whose own true peak is already above the ceiling gets no boost (0 dB at
  least when `wanted > 0`) and a negative `wanted` is never reduced
  further. `gain_db` is rounded to 0.01 dB so that equal inputs give equal
  requests; a boost, whatever limited it, steps down to the hundredth at
  or below the room the ceiling leaves when rounding to the nearest would
  take it above that room, so the boost never passes
  `true_peak_ceiling_dbtp`. Both steps allow 0.00001 dB of f32 noise, so
  a room exactly on a hundredth keeps it.
- **N8** `bit_perfect` for a holder is whether the device it plays on is
  bit-perfect: the running settings of its placement
  (`live.devices[live.placement[holder]].bit_perfect`) once the engine has
  reported it (live settings L3); before that, or without the live
  settings work, whether the holder's configured route names a device in
  `config.outputs.bit_perfect` (no route: the default output, not
  bit-perfect). For a player's Main this agrees with the bit-perfect
  rate and fader spec's `bit_perfect_player` (BR2) whenever the engine has
  placed it, so a player whose fader is locked always gets exactly 0 dB.
  A native DSD stream only reaches a bit-perfect device
  (`OutputsConfig::dsd_output_for`), so N7 never gives a track decided
  there a gain; the engine's DSD path (`BusCommand::DsdMode`: no gain
  touches the words) still refuses a request whose gain is not exactly 1.0
  (N17).
- **N9** `SourceRequest` and `CartRequest`
  (`crates/fp-model/src/command.rs`) gain `normalization: Normalization`.
  Every place that builds one computes it for the holder that will play
  it: `Preload`, `StartCurrent`, `Crossfade`, `LoadPaused` (holder
  `PlayerMain`), `StartCue` (`PlayerCue`; a CUE seek reopens the same
  source and keeps its gain, N15),
  `StartCart` (`CartwallMain`), `StartCartCue` (`CartwallCue`). The CUE and
  pre-listen sources are normalized exactly like on air (Q6): the same
  N7 with the same configuration and track edit; only `bit_perfect` of
  the CUE holder's own device can make the result differ.
  `AppState::request_from_cue_in` and `request_at` take the holder.
- **N10** `Command::SetTrackLoudness { track, excluded: bool, trim_db: f32
  }` stores the track's loudness edit. It is refused with
  `ModelError::InvalidTrim` when `trim_db` is not finite or
  `|trim_db| > max_trim_db`, and with `UnknownTrack` for an unknown track.
  Both N7 and N10 use the same maximum (`max_trim_in_use`): a
  `max_trim_db` that is not a finite number of at least 0 (a configuration
  not validated) allows no trim.
  It is allowed while the track plays (it changes what loads afterwards,
  N12).
- **N11** Fixed while it sounds. `PlayerState` gains, runtime only (never
  saved, like `dsd`): `on_air: Option<Normalization>` for the current
  entry and `cue_normalization: Option<Normalization>` for the CUE; the
  cartwall's `PlayingCart` and its CUE keep theirs the same way. They are
  set from the request the model sends (`StartCurrent`, `Crossfade`,
  `LoadPaused`, `StartCue`, `StartCart`, `StartCartCue`) or, for a
  scheduled transition, taken from `EngineEvent::TransitionStarted`, which
  carries the normalization the engine opened the started source with
  (a re-preload that raced a transition the engine had already committed
  may have asked for another gain: the engine plays the earlier one, and
  `on_air` says so; a preload the engine keeps for a request with the
  same gain but another status, such as not measured and then off, takes
  the newer status), and cleared when the source ends. Nothing else
  changes them: not an analysis or loudness-only result, a configuration change, a loudness
  edit, nor a pending device change applied while the track is paused.
- **N12** Loaded afterwards. `PlayerState::preloaded` becomes `Option<(
  EntryId, f64, Normalization)>`, so `reconcile` sends a new `Preload` when
  the normalization of the entry waiting to play changes (an analysis or loudness-only result that
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
  track plays at 0 dB. A DSD track in that situation (loaded paused with a
  gain, then its device turned bit-perfect with DoP or native DSD) plays
  converted to PCM keeping its gain, with no **BP** or **DSD** badge:
  `DsdFacts` carries the request's linear gain and `dsd_decision` falls
  back with `DsdFallback::NormalizationGain` when it is not exactly 1.0.

## 8. User interface

### 8.1 Player

- **N18** The player header shows a gain badge next to the **BP** badge,
  for the current entry's `on_air` normalization while the player is not
  stopped:

  | Status | Badge | Tooltip |
  |---|---|---|
  | `Applied { limited: None }` | "−4.2 dB" (signed, one decimal; "0.0 dB" for a gain under 0.05 dB) | "Gain {gain} dB: brings the measured {lufs} LUFS to the {target} LUFS target.", plus "Includes this track's trim of {trim} dB." only when the trim in use is not 0 |
  | `Applied { limited: Some(_) }` | as above | adds "Limited by the true-peak ceiling ({ceiling} dBTP): {wanted} dB were wanted." or "Limited by the maximum boost ({max} dB): {wanted} dB were wanted." |
  | `NotMeasured` | "0.0 dB" (amber) | the loudness editor's lines (itr06 I26) |
  | `BitPerfect` | "0.0 dB" (amber), only while `normalize` is on | the loudness editor's lines |
  | `Excluded` | "0.0 dB" (amber) | the loudness editor's lines |
  | `Off`, `Silent` | none | — |

  *Amendment 2026-10-21 (itr06 I12).* Every not-normalized status shows the
  same amber "0.0 dB" and states its reason only in the tooltip, so the badge
  is as short as a gain. The badge has a fixed width (that of "−00.0 dB") and
  a border in its tone. The header measures its items first (`header_layout`)
  and collapses the least important when the column is narrow, so nothing
  overlaps.

  `view::gain_badge(&Normalization, &LoudnessConfig, &Track) -> Option<GainBadge>`
  is a pure function in `crates/fp-app/src/ui/view.rs`; `player.rs` paints it
  with the existing badge helper, at full strength (never dimmed). With `normalize` off a bit-perfect player
  shows no badge: it plays at unity gain either way, and the model reports
  `BitPerfect` before `Off`. The tooltip names the current settings; when
  `normalization(config, track, false)` now differs from `on_air` (a change
  since the source started, N11) the tooltip is only "Gain set when the
  track was loaded. Its loudness settings or measurement have changed
  since; it keeps this gain until it stops.": the current target,
  measurement and trim did not give this gain, so it does not quote them
  beside it. The badge shows what the model
  asked the engine for, or for a scheduled transition what the engine
  reports having started (N11).
- **N19** The row tooltip of the track tables (`view::track_tooltip`,
  the hover text of a playlist, library or cart row) gains a loudness
  line after the format: "Loudness: {lufs} LUFS, true peak {tp} dBTP" when
  measured, "Loudness: not measured" otherwise, and "Excluded from
  normalization" or "Trim {trim} dB" when the edit is not the default.
  Every trim the interface shows (this line, the badge tooltip of N18,
  the Trim slider of N20 and N21) is the one in use,
  `fp_model::trim_in_use(config, edit)`: the saved trim within ±
  `max_trim_db`, as N7 computes the gain. When a lowered maximum limits a
  saved trim, this line reads "Trim {trim} dB (saved {saved} dB, limited by
  the maximum trim)", unless both read the same at the precision shown
  (0.1 dB).

### 8.2 Per-track controls

- **N20** The waveform panel's context menu (player column and CUE window,
  `crates/fp-app/src/ui/wave_panel.rs`, `edit_markers`), where the markers
  are edited, gains a **Loudness…** item. It opens a small popup with the
  measurement (or "not measured"), an **Exclude from normalization**
  checkbox and a **Trim** slider (−`max_trim_db` ..= +`max_trim_db`, 0.5 dB
  steps, double-click resets to 0). Each change sends `SetTrackLoudness`.
  A playlist row's context menu has the same **Loudness…** item, for that
  row's track.
  The popup reads only the snapshot (rule 8). It names the track it edits,
  keeps editing it when the player moves on, has a Close button, and closes
  by itself when the track leaves the library.
  The slider is a relative trim, 0 for every unedited track, so the popup
  also shows the gain lines (`view::gain_lines`, from
  `fp_model::normalization`): "Automatic gain: X dB" (the gain with trim 0,
  or why there is none: normalization off, excluded, not measured, silent),
  "Gain with trim: Y dB" when the trim is not 0, the limit that held the
  gain back with the gain wanted, and a note when normalization is off. A bit-perfect player reads "No automatic gain: bit-perfect output plays tracks unaltered" (the popup keeps the holder it was opened from; the cart editor uses the cartwall Main). A short hint says the trim is added to the automatic gain, and a track that is sounding gets a note that a change applies from its next load. The cart editor draws the gain lines under the Trim row at full width.
- **N21** The cart edit dialog (where a cart's file and properties are
  edited) shows the same two controls for the cart's track, as two rows
  ("Exclude from normalization", with the measurement as its hint, and
  "Trim"). The Trim row's hint holds the same gain lines as N20.

### 8.3 Settings

- **N22** A **Loudness** tab in Settings
  (`crates/fp-app/src/ui/settings/loudness.rs`) with: **Normalize
  loudness** (toggle), **Target** (slider, LUFS), **True-peak ceiling**
  (slider, dBTP), **Maximum boost** (slider, dB), **Maximum trim**
  (slider, dB), each with a short one-clause hint (the loudness every track is brought to; a boost never takes a true peak above the ceiling; the most a quiet track is turned up; how far a track's trim may go), a hint "Changes apply to tracks loaded afterwards; what is
  playing keeps its gain.", a line counting tracks not measured
  (`loudness_unmeasured(state)`: playable with `loudness == None` and
  `loudness_pass != Some(LOUDNESS_VERSION)`, i.e. still waiting for the
  pass) saying that the background pass is measuring them, and
  **Restore defaults** (N2). Slider ranges are those of N1. A note says that
  a bit-perfect output is never normalized.

### 8.4 Strings (en-US source; es-ES written with the change; the other nine locales AI-translated)

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
| `settings-normalize-unmeasured` | "{ $count } tracks not measured yet" (Fluent plural: "1 track not measured yet") |
| `settings-hint-normalize-bp` | "A player or cart on a bit-perfect output always plays at unity gain: it is never normalized." |
| `tip-gain-earlier-settings` | "Gain set when the track was loaded. Its loudness settings or measurement have changed since; it keeps this gain until it stops." |
| `loudness-close` | "Close" |
| `badge-gain` | "{ $gain } dB" |
| `tip-gain-applied` | (removed in itr06 I26: the tooltip of an applied gain lists `view::loudness_facts`, the lines of the loudness editor: the measurement with its true peak, then the gain lines) |
| `tip-gain-limited-peak` | "Limited by the true-peak ceiling ({ $ceiling } dBTP)." |
| `tip-gain-limited-boost` | "Limited by the maximum boost ({ $max } dB)." |
| `track-loudness` | "Loudness: { $lufs } LUFS, true peak { $tp } dBTP" |
| `track-loudness-unmeasured` | "Loudness: not measured" |
| `track-loudness-excluded` | "Excluded from normalization" |
| `track-loudness-trim` | "Trim { $trim } dB" |
| `track-loudness-trim-limited` | "Trim { $trim } dB (saved { $saved } dB, limited by the maximum trim)" |
| `wave-loudness` | "Loudness…" |
| `loudness-exclude` | "Exclude from normalization" |
| `loudness-trim` | "Trim" |
| `error-invalid-trim` | "The trim must be a number within the maximum trim." (N10: also for a value that is not finite) |

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
| `fp-model` (`tests/restore.rs`, new `tests/loudness_config.rs`) | N1: each field clamped with a warning, non-finite → default, a file without `loudness` loads the defaults. N2: Restore defaults resets the section only. N3: `UpdateConfig` with loudness changes only emits `Preload`s (no pending item, no `UpdateSettings`). |
| `fp-model` (new `tests/loudness.rs`) | N7: one test per row of the table, the true-peak and max-boost limits (and the tie), a cut not limited by `max_boost_db` nor by the ceiling (a −18 LUFS track with a +0.5 dBTP peak and target −16 gets +0 dB, a −10 LUFS track with a +0.5 dBTP peak keeps its −6 dB), a boost with a peak above the ceiling gets 0 dB, trim added before the limits, rounding, a boost never rounded above the ceiling whatever limited it, and a room on a hundredth kept exactly. N8: bit-perfect from placement, from the configured route, default output not bit-perfect. N9: every request kind carries the holder's normalization; CUE and cart pre-listen get the same gain as on air (Q6), except on a bit-perfect Cue device (0 dB while Main gets a gain). N10: refused out of range and non-finite, and any trim refused under a maximum that is not finite or below 0; allowed while playing. N11: an analysis result, a config change and an edit leave `on_air` and the engine untouched while playing; `TransitionStarted` takes the normalization the engine reports, also when a re-preload of the same entry with another gain raced it (fp-engine `tests/engine.rs`: the report carries the started source's gain, and the latest status when a preload was kept for the same gain). N12: each of the four causes re-preloads the next. N13: `LoadPaused` recomputes. |
| `fp-model` (`tests/analysis.rs`) | N6: `apply_analysis` stores the measurement and keeps the edit and the markers; a track without a measurement is `NotMeasured`; a loudness-only result changes no marker; `ANALYSIS_VERSION` stays 8 and a v8 entry is not outdated. |
| `fp-dsp` (`tests/loudness.rs`) | N4: EBU Tech 3341 / 3342 cases that apply to a file measurement (1 kHz sine at −23 dBFS reads −23.0 LUFS ±0.1 at 44.1 and 48 kHz; the gating cases); true peak of the BS.1770 test signals; the moved K-weighting keeps its existing tests. N5: silence → `None`; a 200 ms file is not measured (`None`, no warning); non-finite input does not panic. |
| `fp-analysis` (`tests/analyze.rs`, new `tests/loudness_pass.rs`) | N4–N6: a generated WAV gets a measurement; a cache entry round-trips `loudness` and `loudness_pass`; an old entry without them decodes; a file truncated mid-stream is measured up to the damage. N6b: the pass fills the existing entry in place and leaves markers byte-identical; it runs only when the normal queue is empty and yields to a new normal job; a short file or a failure is recorded and not retried; a `LOUDNESS_VERSION` bump re-queues; it runs with `normalize` off. |
| `fp-engine` (`tests/mixer.rs`, `audio_path_audit.rs`) | N14: a slot attached with +6 dB outputs twice the amplitude and the meter reads it; a fader move keeps the gain; the audit covers `Attach` with a gain (no allocation on the callback). N15: a seek and a forced rate change keep the gain. N16: gain 1 on a bit-perfect bus stays bit-exact; gain ≠ 1 clears the unaltered flag. N17 (`tests/dsd_output.rs`, `fp-model/tests/dsd_rules.rs`): a DSD request with a gain on a DoP or native device plays converted, also when loaded paused and resumed; `dsd_decision` falls back with `NormalizationGain`. |
| `fp-app` kittests (new `tests/loudness_ui.rs`) | N18: each badge and its accessible name; N19: tooltip lines, the trim in use and the saved one when the maximum limits it; N20: the Loudness… popup sends `SetTrackLoudness`; N21: the cart dialog does too; N22: the Settings tab updates the config and its sliders keep the ranges. |
| `fp-remote` (`tests/api.rs`) | N23: the DTO fields for each status; N24: the endpoint, a refused trim and an unknown track. |

## 11. Documentation to update

- `README.md`: feature list (loudness normalization).
- `docs/user/settings.md` (the Loudness tab), `docs/user/players.md` (the
  gain badge), `docs/user/markers-and-mixing.md` (the Loudness… popup:
  exclude and trim), `docs/user/cartwall.md` (the cart dialog),
  `docs/user/bit-perfect.md` (no normalization on bit-perfect devices), `docs/user/analysis.md` or the page that describes analysis (the background loudness measurement),
  `docs/user/remote-control.md` (fields and endpoint).
- `docs/technical/analysis.md` (measurement, the loudness-only pass, `LOUDNESS_VERSION`, `ANALYSIS_VERSION` unchanged),
  `docs/technical/audio-engine.md` (`Attach` gain, gain kept on reopen),
  `docs/technical/persistence.md` (`Track::loudness`,
  `Track::loudness_edit`), `docs/technical/remote-api.md`,
  `docs/technical/ui.md` (badge, popup, Settings tab),
  `docs/technical/dsp.md` (new: the `fp-dsp` crate; or a section of
  `docs/technical/audio-engine.md`) and `docs/technical/README.md` (crate map).
- `CLAUDE.md` "Where things are": a row `crates/fp-dsp` (K-weighting,
  true peak, BS.1770 gating: pure signal processing shared by the engine
  and the analysis), and the safety-lint rule 6 lists `fp-dsp` among the
  crates that deny `clippy::indexing_slicing`; `AGENTS.md` stays a pointer.
  Also the workspace `Cargo.toml` and the `README.md` crate overview if it
  lists crates.
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
  and true peak (loudness normalization spec N4–N6); tracks without a
  measurement get a background loudness-only pass (N6b)."
- Main spec §8.4: add the Loudness tab.

## 14. Maintainer decisions (2026-10-08)

Answers to the questions raised while writing the spec; binding.

- **Q1** `normalize` defaulted to off so that an upgrade changed nothing on
  air (§4). Revised (itr06 I27): it is **on for new installations**; an
  existing configuration keeps its current setting (every saved config already
  stores the value). No migration.
- **Q2** The true-peak ceiling **only limits boosts**: it never cuts a
  track below `target − measured + trim`, and a negative gain is never
  reduced further by it (N7).
- **Q3** A **background loudness-only pass** measures tracks with no
  loudness measurement, with no full re-analysis. Decision on versions:
  `ANALYSIS_VERSION` is **not bumped**; a separate `LOUDNESS_VERSION`
  (with `loudness_pass` in the cache entry) governs the pass, so the
  library is never made "outdated" by this feature (N6, N6b).
- **Q4** The meter's −23 LUFS target (`meter.loudness_target_lufs`) stays
  **separate** from the normalization target (−16).
- **Q5** The shared K-weighting and true-peak code moves to a **new crate
  `fp-dsp`** (N6); the crate table in CLAUDE.md is updated (§11).
- **Q6** CUE and pre-listen (player and cart) are normalized **exactly
  like on air** (N9).
- **Q7** Files shorter than 400 ms are **not measured**: 0 dB and the "not
  normalized" badge (N5).
