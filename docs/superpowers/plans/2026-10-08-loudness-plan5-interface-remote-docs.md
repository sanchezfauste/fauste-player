# Loudness Normalization, Plan 5: Interface, Remote API and Documentation

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The operator sees and controls normalization: a gain badge in each player header, a loudness line in the track tooltips, a **Loudness…** popup on the waveform and the same controls in the cart editor, a **Loudness** tab in Settings; remote clients read the measurement, the edit and the gain on air and can set the edit; the user guide, the technical docs, the README and the specs describe all of it, in every locale.

**Architecture:** Pure view functions in `crates/fp-app/src/ui/view.rs` (`gain_badge`, `loudness_lines`, `signed_db`) return Fluent message keys with preformatted arguments (`Message`), which the painting code translates; `PlayerView` gains `gain: Option<GainBadge>` from the model's `PlayerState::on_air` (plan 3). One small widget module, `crates/fp-app/src/ui/loudness_edit.rs`, draws the exclude toggle and the trim slider and sends `Command::SetTrackLoudness`; the waveform popup and the cart editor both use it. `crates/fp-app/src/ui/settings/loudness.rs` is the Settings tab, following `settings/meters.rs`. `fp-remote` gains DTO fields and `PUT /tracks/{id}/loudness`. MIDI and OSC gain nothing (N24, N25).

**Tech Stack:** Rust 2024, egui 0.36 / `egui_kittest`, Fluent (`crates/fp-app/locales/*/main.ftl`), axum (`fp-remote`), mdBook guide (`docs/user`).

**Spec:** `docs/superpowers/specs/2026-10-08-loudness-normalization-design.md` (§8 N18–N22, §8.4 strings, §9 N23–N25, §10 rows `fp-app` and `fp-remote`, §11, §13). Design source for colours and spacing: the "Reproductor v3" design (CLAUDE.md "Design source"); reuse `theme` colours (`ACCENT`, `AMBER`, `NEUTRAL_600`, `NEUTRAL_400`) and the existing badge tile.

**Ruling (planning): the tooltip's loudness lines.** The track tooltip is a grid of label/value rows (`TipField`); the spec's N19 strings are whole sentences ("Loudness: −14.2 LUFS, true peak −0.8 dBTP"). They are drawn as plain lines under the grid (after Path) by a separate pure function `loudness_lines`, rather than as a grid row "after the format". — Why: a sentence does not fit the label column, and the spec's keys stay as written. — Cost if wrong: moving two lines up in `track_tip`.

**Ruling (planning): the measurement of a silent track** (status `Silent`, no integrated value) shows `track-loudness` with `—` as the LUFS value; the spec defines the line only for "measured" and "not measured".

**Ruling (planning): the unmeasured count is remembered per snapshot** (`UnmeasuredCount`, the same shape as `services::OutdatedCount`), so the Settings tab does not scan the library every frame.

## Global Constraints

- All code, identifiers, comments, docs and commit messages in English; never mention other playout or radio-automation products.
- The UI never blocks: it reads snapshots and sends commands (CLAUDE.md rule 8). "The popup reads only the snapshot" (N20).
- Every new string is a Fluent message in **all 11 locale files** (`crates/fp-app/locales/{ca-ES,de-DE,en-US,es-ES,eu-ES,fr-FR,gl-ES,it-IT,nl-NL,pl-PL,pt-PT}/main.ftl`): en-US is the source, es-ES written by hand, the other nine AI-translated. `crates/fp-app/tests/i18n.rs` fails if any locale misses a message or a variable — so **each task adds its own keys to all 11 files**.
- Keys and English text, verbatim from §8.4: `settings-tab-loudness` "Loudness"; `settings-normalize` "Normalize loudness"; `settings-hint-normalize` "Plays every track and cart at the same integrated loudness with a fixed gain per track."; `settings-normalize-target` "Target"; `settings-normalize-ceiling` "True-peak ceiling"; `settings-normalize-max-boost` "Maximum boost"; `settings-normalize-max-trim` "Maximum trim"; `settings-hint-normalize-apply` "Changes apply to tracks loaded afterwards; what is playing keeps its gain."; `settings-normalize-unmeasured` "{ $count } tracks not measured yet"; `badge-gain` "{ $gain } dB"; `badge-not-normalized` "Not normalized"; `tip-gain-applied` "Normalized to { $target } LUFS: measured { $lufs } LUFS, trim { $trim } dB."; `tip-gain-limited-peak` "Limited by the true-peak ceiling ({ $ceiling } dBTP)."; `tip-gain-limited-boost` "Limited by the maximum boost ({ $max } dB)."; `tip-not-measured` "Loudness not measured yet: this track plays unchanged."; `tip-not-normalized-bp` "Bit-perfect output: tracks play unaltered."; `tip-not-normalized-excluded` "This track is excluded from normalization."; `track-loudness` "Loudness: { $lufs } LUFS, true peak { $tp } dBTP"; `track-loudness-unmeasured` "Loudness: not measured"; `track-loudness-excluded` "Excluded from normalization"; `track-loudness-trim` "Trim { $trim } dB"; `wave-loudness` "Loudness…"; `loudness-exclude` "Exclude from normalization"; `loudness-trim` "Trim". The meter keys `settings-loudness` and `settings-loudness-target` keep their meaning (Q4).
- N18 badge table: `Applied` → "±x.x dB" (one decimal; "0.0 dB" under 0.05 dB); `NotMeasured` → "Not normalized" amber; `BitPerfect`, `Excluded` → "Not normalized" muted; `Off`, `Silent` → no badge; shown "while the player is not stopped".
- N20: trim slider −`max_trim_db` ..= +`max_trim_db`, **0.5 dB steps, double-click resets to 0**; each change sends `SetTrackLoudness`.
- N22: slider ranges are those of N1 (target −36..=−6, ceiling −12..=0, max boost 0..=24, max trim 0..=24).
- N23 statuses as lower-case names: `applied`, `limited_true_peak`, `limited_max_boost`, `off`, `excluded`, `not_measured`, `silent`, `bit_perfect`. `config.loudness` is not exposed (station configuration is a remote non-goal).
- N24: `PUT /tracks/{id}/loudness` `{ "excluded": bool, "trim_db": number }`, same authentication and error mapping as the marker endpoint; `InvalidTrim` is refused with the model's reason. OSC and MIDI gain nothing (N24, N25).
- Numbers in the interface use the true minus sign `−` (U+2212), as the existing meter strings do.
- `CHANGELOG.md` is never edited; the translated guides in `docs/i18n/` are not touched (regenerated before a release).
- Conventional Commits; every commit ends with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Gate for every task: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`.
- Progress ledger: `.superpowers/sdd/2026-10-08-loudness-plan5-interface-remote-docs/progress.md`.

## Review Focus

1. **The badge of a paused player** (Paused is "not stopped"): it stays, with the gain the source was started with, even after the operator changes the target. Test: Task 2, `n18_a_paused_player_keeps_its_badge_and_gain`.
2. **The Loudness… popup left open while the player moves to another track**: it keeps editing the track it was opened for (it holds the `TrackId`), and closes by itself when that track leaves the library. Test: Task 3, `the_popup_closes_when_its_track_is_gone`.
3. **A trim slider when `max_trim_db` is 0**: the range collapses to 0..=0; the slider must not panic and the exclude toggle still works. Test: Task 3, `a_zero_maximum_trim_still_allows_excluding`.
4. **A remote client sending `trim_db` as a huge number or a string**: a number out of range is refused by the model (409 with the reason), a wrong type is a 400 from the JSON layer, an unknown field is a 400 (`deny_unknown_fields`, as the marker body). Test: Task 5, `n24_the_loudness_endpoint_refuses_bad_bodies`.
5. **Locale parity**: every key added in this plan exists, with the same variables, in all 11 files — caught by `crates/fp-app/tests/i18n.rs` at every task's gate.

---

## File Structure

| File | Responsibility |
|---|---|
| `crates/fp-app/src/ui/view.rs` | `Message`, `BadgeTone`, `GainBadge`, `signed_db`, `gain_badge`, `loudness_lines`; `PlayerView::gain` |
| `crates/fp-app/src/ui/player.rs` (~L337-365, after the BP tile) | Paints the gain badge |
| `crates/fp-app/src/ui/table.rs` (`track_tip`, ~L1012-1080) | Paints the loudness lines |
| `crates/fp-app/src/ui/loudness_edit.rs` (new) | Exclude toggle + trim slider for one track |
| `crates/fp-app/src/ui.rs` (module list) | `pub mod loudness_edit;` |
| `crates/fp-app/src/ui/wave_panel.rs` (`edit_markers`) and `crates/fp-app/src/ui/app.rs` (`ViewState`) | **Loudness…** item and popup |
| `crates/fp-app/src/ui/settings/carts.rs` (cart editor, after Exclusive ~L540) | The same controls for the cart's track |
| `crates/fp-app/src/ui/settings/loudness.rs` (new), `crates/fp-app/src/ui/settings.rs` | The Loudness tab, its nav entry, Restore defaults |
| `crates/fp-app/src/services.rs` | `UnmeasuredCount` |
| `crates/fp-app/locales/*/main.ftl` (11 files) | The strings |
| `crates/fp-app/tests/view.rs`, `tests/loudness_ui.rs` (new), `tests/settings_layout.rs`, `tests/settings_restore.rs` | Tests |
| `crates/fp-remote/src/dto.rs`, `src/api.rs`, `src/http/{mod,handlers}.rs` | N23, N24 |
| `crates/fp-remote/tests/{dto,edit}.rs` | Tests |
| `README.md`, `docs/user/*.md`, `docs/technical/{ui,remote-api}.md`, specs | Task 6 |

---

### Task 1: The view: badge, tooltip lines, player view (N18, N19)

**Files:**
- Modify: `crates/fp-app/src/ui/view.rs` (new items near `track_tooltip` ~L490-560; `PlayerView` L45-77; `player_view` L241-…)
- Modify: the 11 `main.ftl` files (badge, tip and track keys)
- Test: `crates/fp-app/tests/view.rs`

**Interfaces:**
- Consumes: `fp_model::{Normalization, NormalizationStatus, GainLimit, LoudnessConfig, LoudnessMeasurement, LoudnessEdit}`, `PlayerState::on_air` (plan 3).
- Produces (in `fp_app::ui::view`):
  - `#[derive(Debug, Clone, PartialEq)] pub struct Message { pub key: &'static str, pub args: Vec<(&'static str, String)> }`
  - `#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum BadgeTone { Normal, Amber, Muted }`
  - `#[derive(Debug, Clone, PartialEq)] pub struct GainBadge { pub text: Message, pub tone: BadgeTone, pub tip: Vec<Message> }`
  - `pub fn signed_db(db: f32) -> String` ("+4.0", "−4.2", "0.0" when `|db| < 0.05`)
  - `pub fn gain_badge(n: &Normalization, config: &LoudnessConfig, track: &Track) -> Option<GainBadge>`
  - `pub fn loudness_lines(track: &Track) -> Vec<Message>`
  - `PlayerView::gain: Option<GainBadge>`

- [ ] **Step 1: Write the failing tests** (append to `crates/fp-app/tests/view.rs`; add the new names to its `use fp_app::ui::view::{…}` and `fp_model` imports)

```rust
fn measured_track(lufs: Option<f32>, tp: f32) -> fp_model::Track {
    let mut t = fp_model::Track::new(fp_model::TrackId(7), std::path::PathBuf::from("/m/a.flac"));
    t.loudness = Some(fp_model::LoudnessMeasurement { integrated_lufs: lufs, true_peak_dbtp: tp });
    t
}

fn msg(key: &'static str, args: &[(&'static str, &str)]) -> Message {
    Message {
        key,
        args: args.iter().map(|(k, v)| (*k, (*v).to_owned())).collect(),
    }
}

fn applied(gain_db: f32, limited: Option<fp_model::GainLimit>) -> fp_model::Normalization {
    fp_model::Normalization {
        gain_db,
        status: fp_model::NormalizationStatus::Applied { limited },
    }
}

#[test]
fn signed_db_has_one_decimal_a_sign_and_a_true_minus() {
    assert_eq!(signed_db(4.0), "+4.0");
    assert_eq!(signed_db(-4.24), "−4.2");
    assert_eq!(signed_db(0.04), "0.0");
    assert_eq!(signed_db(-0.04), "0.0");
}

#[test]
fn n18_an_applied_gain_shows_its_value_and_why() {
    let t = measured_track(Some(-20.0), -10.0);
    let config = fp_model::LoudnessConfig { normalize: true, ..Default::default() };
    let b = gain_badge(&applied(4.0, None), &config, &t).unwrap();
    assert_eq!(b.text, msg("badge-gain", &[("gain", "+4.0")]));
    assert_eq!(b.tone, BadgeTone::Normal);
    assert_eq!(
        b.tip,
        vec![msg(
            "tip-gain-applied",
            &[("target", "−16.0"), ("lufs", "−20.0"), ("trim", "0.0")]
        )]
    );
}

#[test]
fn n18_a_limited_gain_says_which_limit() {
    let t = measured_track(Some(-20.0), -3.0);
    let config = fp_model::LoudnessConfig { normalize: true, ..Default::default() };
    let peak = gain_badge(&applied(2.0, Some(fp_model::GainLimit::TruePeak)), &config, &t).unwrap();
    assert_eq!(peak.tip[1], msg("tip-gain-limited-peak", &[("ceiling", "−1.0")]));
    let boost = gain_badge(&applied(12.0, Some(fp_model::GainLimit::MaxBoost)), &config, &t).unwrap();
    assert_eq!(boost.tip[1], msg("tip-gain-limited-boost", &[("max", "12.0")]));
}

#[test]
fn n18_the_other_statuses() {
    let t = measured_track(Some(-20.0), -10.0);
    let c = fp_model::LoudnessConfig::default();
    let of = |status| fp_model::Normalization { gain_db: 0.0, status };
    use fp_model::NormalizationStatus as S;
    let not = gain_badge(&of(S::NotMeasured), &c, &t).unwrap();
    assert_eq!((not.text.key, not.tone), ("badge-not-normalized", BadgeTone::Amber));
    assert_eq!(not.tip, vec![msg("tip-not-measured", &[])]);
    let bp = gain_badge(&of(S::BitPerfect), &c, &t).unwrap();
    assert_eq!((bp.tone, bp.tip[0].key), (BadgeTone::Muted, "tip-not-normalized-bp"));
    let ex = gain_badge(&of(S::Excluded), &c, &t).unwrap();
    assert_eq!((ex.tone, ex.tip[0].key), (BadgeTone::Muted, "tip-not-normalized-excluded"));
    assert_eq!(gain_badge(&of(S::Off), &c, &t), None);
    assert_eq!(gain_badge(&of(S::Silent), &c, &t), None);
}

#[test]
fn n19_the_tooltip_has_a_loudness_line() {
    let mut t = measured_track(Some(-14.24), -0.81);
    assert_eq!(
        loudness_lines(&t),
        vec![msg("track-loudness", &[("lufs", "−14.2"), ("tp", "−0.8")])]
    );
    t.loudness_edit.trim_db = 1.5;
    assert_eq!(loudness_lines(&t)[1], msg("track-loudness-trim", &[("trim", "+1.5")]));
    t.loudness_edit.excluded = true;
    assert_eq!(loudness_lines(&t)[1], msg("track-loudness-excluded", &[]));
    t.loudness = None;
    assert_eq!(loudness_lines(&t)[0], msg("track-loudness-unmeasured", &[]));
    let silent = measured_track(None, -90.0);
    assert_eq!(loudness_lines(&silent)[0].args[0], ("lufs", "—".to_owned()));
}

#[test]
fn n18_the_player_view_shows_the_gain_on_air_only_while_not_stopped() {
    let (mut s, _entries, p) = state(2);
    s.config.loudness.normalize = true;
    for t in s.library.iter_mut() {
        t.loudness = Some(fp_model::LoudnessMeasurement { integrated_lufs: Some(-20.0), true_peak_dbtp: -10.0 });
    }
    assert_eq!(player_view(&s, p, None, 0.0).unwrap().gain, None, "stopped");
    fp_model::apply(&mut s, fp_model::Command::Play(p)).unwrap();
    let v = player_view(&s, p, Some(1.0), 0.0).unwrap();
    assert_eq!(v.gain.unwrap().text, msg("badge-gain", &[("gain", "+4.0")]));
}

#[test]
fn every_message_the_view_returns_exists_in_english() {
    let en = fp_app::i18n::I18n::new(Some("en-US"));
    for key in [
        "badge-gain", "badge-not-normalized", "tip-gain-applied", "tip-gain-limited-peak",
        "tip-gain-limited-boost", "tip-not-measured", "tip-not-normalized-bp",
        "tip-not-normalized-excluded", "track-loudness", "track-loudness-unmeasured",
        "track-loudness-excluded", "track-loudness-trim",
    ] {
        assert_ne!(en.tr(key), key, "{key}");
    }
}
```

(`state(tracks)` is `tests/view.rs`'s own helper: `(AppState, Vec<EntryId>, PlayerId)`; `I18n::new(Some("en-US"))` is how `tests/cartwall_ui.rs` builds one.)

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p fp-app --test view 2>&1 | tail -5`
Expected: FAIL to compile (`gain_badge`, `Message`, … not found).

- [ ] **Step 3: Implement**

`view.rs`:

```rust
/// A Fluent message and its arguments, already formatted; translated where
/// it is painted, so the view stays a pure function.
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub key: &'static str,
    pub args: Vec<(&'static str, String)>,
}

impl Message {
    fn new(key: &'static str, args: &[(&'static str, String)]) -> Self {
        Self { key, args: args.to_vec() }
    }
}

/// How the gain badge is coloured (loudness normalization spec N18).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeTone {
    Normal,
    Amber,
    Muted,
}

/// The gain badge of a player header and its tooltip lines (N18).
#[derive(Debug, Clone, PartialEq)]
pub struct GainBadge {
    pub text: Message,
    pub tone: BadgeTone,
    pub tip: Vec<Message>,
}

/// One decimal with the true minus sign; `−0.0` is shown as `0.0`.
fn decimal(value: f32) -> String {
    let text = format!("{:.1}", value.abs());
    if value < 0.0 && text != "0.0" {
        format!("−{text}")
    } else {
        text
    }
}

/// A gain in dB as the badge shows it: signed, one decimal, "0.0" under
/// 0.05 dB.
pub fn signed_db(db: f32) -> String {
    if db.abs() < 0.05 {
        "0.0".to_owned()
    } else if db > 0.0 {
        format!("+{}", decimal(db))
    } else {
        decimal(db)
    }
}

/// N18: the badge for the normalization of the source on air.
pub fn gain_badge(
    n: &fp_model::Normalization,
    config: &fp_model::LoudnessConfig,
    track: &Track,
) -> Option<GainBadge> {
    use fp_model::{GainLimit, NormalizationStatus as S};
    let not_normalized = |tone, tip: &'static str| GainBadge {
        text: Message::new("badge-not-normalized", &[]),
        tone,
        tip: vec![Message::new(tip, &[])],
    };
    match n.status {
        S::Off | S::Silent => None,
        S::NotMeasured => Some(not_normalized(BadgeTone::Amber, "tip-not-measured")),
        S::BitPerfect => Some(not_normalized(BadgeTone::Muted, "tip-not-normalized-bp")),
        S::Excluded => Some(not_normalized(BadgeTone::Muted, "tip-not-normalized-excluded")),
        S::Applied { limited } => {
            let lufs = track
                .loudness
                .and_then(|m| m.integrated_lufs)
                .map_or_else(|| "—".to_owned(), decimal);
            let mut tip = vec![Message::new(
                "tip-gain-applied",
                &[
                    ("target", decimal(config.target_lufs)),
                    ("lufs", lufs),
                    ("trim", signed_db(track.loudness_edit.trim_db)),
                ],
            )];
            match limited {
                Some(GainLimit::TruePeak) => tip.push(Message::new(
                    "tip-gain-limited-peak",
                    &[("ceiling", decimal(config.true_peak_ceiling_dbtp))],
                )),
                Some(GainLimit::MaxBoost) => tip.push(Message::new(
                    "tip-gain-limited-boost",
                    &[("max", decimal(config.max_boost_db))],
                )),
                None => {}
            }
            Some(GainBadge {
                text: Message::new("badge-gain", &[("gain", signed_db(n.gain_db))]),
                tone: BadgeTone::Normal,
                tip,
            })
        }
    }
}

/// N19: the loudness lines of the track tooltip, drawn under its fields.
pub fn loudness_lines(track: &Track) -> Vec<Message> {
    let mut lines = vec![match track.loudness {
        Some(m) => Message::new(
            "track-loudness",
            &[
                ("lufs", m.integrated_lufs.map_or_else(|| "—".to_owned(), decimal)),
                ("tp", decimal(m.true_peak_dbtp)),
            ],
        ),
        None => Message::new("track-loudness-unmeasured", &[]),
    }];
    let edit = track.loudness_edit;
    if edit.excluded {
        lines.push(Message::new("track-loudness-excluded", &[]));
    } else if edit.trim_db != 0.0 {
        lines.push(Message::new("track-loudness-trim", &[("trim", signed_db(edit.trim_db))]));
    }
    lines
}
```

`PlayerView` gains

```rust
    /// The gain badge of the source on air, while the player is not
    /// stopped (loudness normalization spec N18).
    pub gain: Option<GainBadge>,
```

initialised `gain: None` in `player_view`; after the `view` literal (before `let Some(track) = current else …`):

```rust
    if p.transport != Transport::Stopped
        && let (Some(n), Some(on_air)) = (p.on_air, p.current.and_then(|e| state.track_for_entry(e)))
    {
        view.gain = gain_badge(&n, &state.config.loudness, on_air);
    }
```

Locales — append to each `main.ftl`, next to the existing `badge-bp`/`tip-bp-*` and `tip-field-*` groups, the 12 keys `badge-gain` … `track-loudness-trim` (English text from Global Constraints). es-ES by hand, for example: `badge-not-normalized = Sin normalizar`, `tip-gain-applied = Normalizado a { $target } LUFS: medido { $lufs } LUFS, ajuste { $trim } dB.`, `tip-gain-limited-peak = Limitado por el techo de pico real ({ $ceiling } dBTP).`, `tip-gain-limited-boost = Limitado por la ganancia máxima ({ $max } dB).`, `tip-not-measured = Sonoridad aún no medida: esta pista suena sin cambios.`, `tip-not-normalized-bp = Salida bit-perfect: las pistas suenan inalteradas.`, `tip-not-normalized-excluded = Esta pista está excluida de la normalización.`, `track-loudness = Sonoridad: { $lufs } LUFS, pico real { $tp } dBTP`, `track-loudness-unmeasured = Sonoridad: sin medir`, `track-loudness-excluded = Excluida de la normalización`, `track-loudness-trim = Ajuste { $trim } dB`, `badge-gain = { $gain } dB`. The other nine: translate with the same variables (keep `LUFS`, `dBTP`, `dB` untranslated).

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p fp-app --test view --test i18n`
Expected: PASS.

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-app
git commit -m "feat(ui): describe the normalization of a player and a track

Pure view functions say which gain badge a player shows and which
loudness lines a track tooltip has (loudness normalization spec N18,
N19), with their strings in every locale.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: Paint the badge and the tooltip lines (N18, N19)

**Files:**
- Modify: `crates/fp-app/src/ui/player.rs` (after the BP tile, ~L346-365), `crates/fp-app/src/ui/table.rs` (`track_tip`, after the field rows)
- Test: `crates/fp-app/tests/loudness_ui.rs` (new)

**Interfaces:**
- Consumes: `PlayerView::gain`, `GainBadge`, `BadgeTone`, `Message`, `loudness_lines` (Task 1).
- Produces: `fn tr_message(t: &crate::i18n::I18n, m: &view::Message) -> String` in `crates/fp-app/src/ui/view.rs`'s caller side — put it in `crates/fp-app/src/ui/widgets.rs` as `pub fn tr_message(…)` so Tasks 3–4 can reuse it.

- [ ] **Step 1: Write the failing tests**

`crates/fp-app/tests/loudness_ui.rs`:

```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Loudness normalization in the interface (spec §8): the gain badge
//! (N18), the tooltip lines (N19), the Loudness… popup (N20), the cart
//! editor (N21) and the Settings tab (N22).

mod support;

use egui_kittest::kittest::Queryable;
use fp_model::{AppState, Command, LoudnessMeasurement, apply};
use support::{harness, state};

/// One player on air with a −20 LUFS track: +4.0 dB at the default target.
fn on_air() -> AppState {
    let mut s = state(1, 2);
    s.config.loudness.normalize = true;
    for t in s.library.iter_mut() {
        t.analyzed = true;
        t.duration_secs = 180.0;
        t.loudness = Some(LoudnessMeasurement { integrated_lufs: Some(-20.0), true_peak_dbtp: -10.0 });
    }
    let p = s.players[0].id;
    apply(&mut s, Command::Play(p)).unwrap();
    s
}

#[test]
fn n18_the_header_shows_the_gain_and_says_why() {
    let (mut h, _fake) = harness(on_air());
    h.run_steps(2);
    // The tile's accessible name is its tooltip, as for the BP badge.
    assert!(
        h.query_by_label("Normalized to −16.0 LUFS: measured −20.0 LUFS, trim 0.0 dB.")
            .is_some()
    );
}

#[test]
fn n18_a_track_not_measured_says_so() {
    let mut s = on_air();
    for t in s.library.iter_mut() {
        t.loudness = None;
    }
    // Start again so that the source on air is the unmeasured one.
    let p = s.players[0].id;
    apply(&mut s, Command::Stop(p)).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    let (mut h, _fake) = harness(s);
    h.run_steps(2);
    assert!(h.query_by_label("Loudness not measured yet: this track plays unchanged.").is_some());
}

#[test]
fn n18_no_badge_when_normalization_is_off_or_stopped() {
    let mut s = on_air();
    let p = s.players[0].id;
    apply(&mut s, Command::Stop(p)).unwrap();
    let (mut h, _fake) = harness(s);
    h.run_steps(2);
    assert!(h.query_by_label_contains("Normalized to").is_none());
}

#[test]
fn n18_a_paused_player_keeps_its_badge_and_gain() {
    let mut s = on_air();
    let p = s.players[0].id;
    apply(&mut s, Command::Pause(p)).unwrap();
    let mut config = s.config.clone();
    config.loudness.target_lufs = -10.0;
    apply(&mut s, Command::UpdateConfig(Box::new(config))).unwrap();
    let (mut h, _fake) = harness(s);
    h.run_steps(2);
    // The source on air keeps +4.0 dB; the tooltip names the new target,
    // which the next track will use.
    assert!(h.query_by_label_contains("Normalized to −10.0 LUFS").is_some());
}
```

(The badge tile's accessible name is its tooltip, as for the BP badge (`widgets::tile(ui, size, &tip, …)`); the tooltip lines are joined with a space. `query_by_label_contains` is used as in `tests/dsd_ui.rs`.)

For N19, a test in the same file that hovers a playlist row and reads the tooltip: copy the pattern of `crates/fp-app/tests/table_tooltip.rs` (it hovers a row and queries the tooltip's labels), with the state from `on_air()` and the assertions:

```rust
    assert!(h.query_by_label("Loudness: −20.0 LUFS, true peak −10.0 dBTP").is_some());
```

and, after `SetTrackLoudness { excluded: true, … }` applied to the state before the harness, `h.query_by_label("Excluded from normalization").is_some()`.

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p fp-app --test loudness_ui 2>&1 | tail -15`
Expected: FAIL (no element with those labels).

- [ ] **Step 3: Implement**

`widgets.rs`:

```rust
/// Translates a view `Message` (its arguments are already formatted text).
pub fn tr_message(t: &crate::i18n::I18n, m: &crate::ui::view::Message) -> String {
    let args: Vec<(&str, crate::i18n::Arg)> = m
        .args
        .iter()
        .map(|(k, v)| (*k, crate::i18n::Arg::Text(v.clone())))
        .collect();
    t.tr_args(m.key, &args)
}
```

`player.rs`, after the BP `widgets::tile(…)` call, inside the same `ui.horizontal`:

```rust
                if let Some(badge) = &pv.gain {
                    let text = widgets::tr_message(t, &badge.text);
                    let tip = badge
                        .tip
                        .iter()
                        .map(|m| widgets::tr_message(t, m))
                        .collect::<Vec<_>>()
                        .join(" ");
                    let content = match badge.tone {
                        view::BadgeTone::Normal => theme::ACCENT,
                        view::BadgeTone::Amber => theme::AMBER,
                        view::BadgeTone::Muted => theme::NEUTRAL_600,
                    };
                    let width = ui
                        .painter()
                        .layout_no_wrap(text.clone(), font_semibold(9.0), content)
                        .size()
                        .x
                        + 12.0;
                    widgets::tile(
                        ui,
                        vec2(width, 20.0),
                        &tip,
                        false,
                        TileStyle {
                            content,
                            ..TileStyle::plain()
                        },
                        |p, r, c| {
                            p.text(r.center(), egui::Align2::CENTER_CENTER, &text, font_semibold(9.0), c);
                        },
                    );
                }
```

(Check the header still fits at the narrowest player width in `tests/main_screen.rs`/`settings_layout.rs`-style layout tests; if a test pins the header's contents, extend it.)

`table.rs`, `track_tip`, after the loop that draws the field rows:

```rust
    // N19: the loudness lines, under the fields.
    ui.add_space(4.0);
    for line in view::loudness_lines(track) {
        ui.add(
            egui::Label::new(
                RichText::new(widgets::tr_message(t, &line))
                    .font(font(11.0))
                    .color(theme::NEUTRAL_400),
            )
            .wrap(),
        );
    }
```

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p fp-app --test loudness_ui --test table_tooltip --test main_screen`
Expected: PASS.

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-app
git commit -m "feat(ui): show the normalization gain and a track's loudness

Each player header shows the gain of the source on air, or that it is
not normalized and why; the track tooltip shows the measurement and the
track's edit (loudness normalization spec N18, N19).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: The per-track controls: waveform popup and cart editor (N20, N21)

**Files:**
- Create: `crates/fp-app/src/ui/loudness_edit.rs`
- Modify: `crates/fp-app/src/ui.rs` (module list), `crates/fp-app/src/ui/app.rs` (`ViewState` ~L120-130: `loudness_popup: Option<TrackId>`), `crates/fp-app/src/ui/wave_panel.rs` (`edit_markers` context menu ~L289-334), `crates/fp-app/src/ui/app.rs` (~L729, draws the popup), `crates/fp-app/src/ui/settings/carts.rs` (cart editor, after the Exclusive row ~L540-555)
- Modify: the 11 `main.ftl` files (`wave-loudness`, `loudness-exclude`, `loudness-trim`)
- Test: `crates/fp-app/tests/loudness_ui.rs`

**Interfaces:**
- Consumes: `Command::SetTrackLoudness` (plan 3), `loudness_lines`, `tr_message` (Tasks 1–2).
- Produces: `pub fn controls(ui: &mut egui::Ui, scene: &Scene<'_>, track: &fp_model::Track)` in `fp_app::ui::loudness_edit` (draws the measurement line, the exclude toggle and the trim slider; sends `SetTrackLoudness` on each change).
  - `pub fn popup(ctx: &egui::Context, scene: &Scene<'_>, view: &mut ViewState)` in the same module (the Loudness… window for `ViewState::loudness_popup`).

- [ ] **Step 1: Write the failing tests** (append to `tests/loudness_ui.rs`; import `egui::{Event, Modifiers, PointerButton}` and `egui::accesskit::Role`)

```rust
/// Opens the waveform's context menu of player 1 and picks Loudness….
fn open_popup(h: &mut egui_kittest::Harness<'static, fp_app::ui::app::AppUi>) {
    let wave = h.get_by_label("Waveform: click to seek").rect();
    h.event(Event::PointerMoved(wave.center()));
    for pressed in [true, false] {
        h.event(Event::PointerButton {
            pos: wave.center(),
            button: PointerButton::Secondary,
            pressed,
            modifiers: Modifiers::NONE,
        });
    }
    h.run_steps(2);
    h.get_by_label("Loudness…").click();
    h.run_steps(2);
}

fn loudness_edits(sent: &[Command]) -> Vec<(bool, f32)> {
    sent.iter()
        .filter_map(|c| match c {
            Command::SetTrackLoudness { excluded, trim_db, .. } => Some((*excluded, *trim_db)),
            _ => None,
        })
        .collect()
}

#[test]
fn n20_the_popup_shows_the_measurement_and_excludes_the_track() {
    let (mut h, fake) = harness(on_air());
    h.run_steps(2);
    open_popup(&mut h);
    assert!(h.query_by_label("Loudness: −20.0 LUFS, true peak −10.0 dBTP").is_some());
    fake.take_sent();
    h.get_by_role_and_label(Role::CheckBox, "Exclude from normalization").click();
    h.run_steps(2);
    assert_eq!(loudness_edits(&fake.take_sent()), vec![(true, 0.0)]);
}

#[test]
fn n20_the_trim_slider_sends_half_db_steps_and_resets_on_double_click() {
    let (mut h, fake) = harness(on_air());
    h.run_steps(2);
    open_popup(&mut h);
    fake.take_sent();
    let slider = h.get_by_role_and_label(Role::Slider, "Trim");
    slider.focus();
    h.run_steps(1);
    h.key_press(egui::Key::ArrowRight);
    h.run_steps(2);
    assert_eq!(loudness_edits(&fake.take_sent()), vec![(false, 0.5)]);
    let rect = h.get_by_role_and_label(Role::Slider, "Trim").rect();
    for _ in 0..2 {
        for pressed in [true, false] {
            h.event(Event::PointerButton {
                pos: rect.center(),
                button: PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            });
        }
    }
    h.run_steps(2);
    assert_eq!(loudness_edits(&fake.take_sent()).last(), Some(&(false, 0.0)));
}

#[test]
fn a_zero_maximum_trim_still_allows_excluding() {
    let mut s = on_air();
    s.config.loudness.max_trim_db = 0.0;
    let (mut h, fake) = harness(s);
    h.run_steps(2);
    open_popup(&mut h);
    fake.take_sent();
    h.get_by_role_and_label(Role::CheckBox, "Exclude from normalization").click();
    h.run_steps(2);
    assert_eq!(loudness_edits(&fake.take_sent()), vec![(true, 0.0)]);
}

#[test]
fn the_popup_closes_when_its_track_is_gone() {
    let (mut h, fake) = harness(on_air());
    h.run_steps(2);
    open_popup(&mut h);
    let track = fake.state.load().library.iter().next().unwrap().id;
    fake.state.rcu(|s| {
        let mut s = (**s).clone();
        s.library.remove(track);
        s
    });
    h.run_steps(2);
    assert!(h.query_by_role_and_label(Role::CheckBox, "Exclude from normalization").is_none());
}

#[test]
fn n21_the_cart_editor_has_the_same_controls() {
    let mut s = state(1, 1);
    let page = s.cartwall.pages[0].id;
    apply(&mut s, Command::AssignCartFile { page, index: 0, path: "/carts/a.flac".into() }).unwrap();
    let (mut h, fake) = harness(s);
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Cartwall").click();
    h.run_steps(2);
    // The first cart is selected by default in the editor.
    fake.take_sent();
    h.get_by_role_and_label(Role::CheckBox, "Exclude from normalization").click();
    h.run_steps(2);
    assert_eq!(loudness_edits(&fake.take_sent()), vec![(true, 0.0)]);
}
```

(Adapt `fake.state.rcu`/`store` to how `support::Fake` exposes its state — it holds an `ArcSwap<AppState>`; the existing tests mutate it the same way, e.g. `tests/cue_window.rs`. If the cart editor needs a click on the cart first, copy the selection step from `tests/cartwall_ui.rs`. The double-click step: if `egui_kittest` needs the two clicks inside the double-click time, keep `Harness::builder().with_step_dt(0.02)` as CLAUDE.md "Testing notes" says — `support::harness` already does.)

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p fp-app --test loudness_ui n2 2>&1 | tail -15`
Expected: FAIL (no "Loudness…" item).

- [ ] **Step 3: Implement**

`crates/fp-app/src/ui/loudness_edit.rs`:

```rust
//! The per-track loudness controls (loudness normalization spec N20, N21):
//! the measurement, Exclude from normalization and the trim. Used by the
//! waveform's Loudness… popup and by the cart editor. Reads only the
//! snapshot; every change is one `SetTrackLoudness`.

use egui::{RichText, Ui};
use fp_model::{Command, Track};

use super::app::Scene;
use super::theme;
use super::view;
use super::widgets::{self, font};

pub fn controls(ui: &mut Ui, scene: &Scene<'_>, track: &Track) {
    let t = scene.i18n;
    if let Some(first) = view::loudness_lines(track).first() {
        ui.add(
            egui::Label::new(
                RichText::new(widgets::tr_message(t, first))
                    .font(font(12.0))
                    .color(theme::NEUTRAL_400),
            )
            .selectable(false),
        );
    }
    let edit = track.loudness_edit;
    let send = |excluded: bool, trim_db: f32| {
        scene.ctl.send(Command::SetTrackLoudness {
            track: track.id,
            excluded,
            trim_db,
        });
    };
    let mut excluded = edit.excluded;
    let label = t.tr("loudness-exclude");
    let r = ui.checkbox(&mut excluded, &label);
    if r.changed() {
        send(excluded, edit.trim_db);
    }
    let max = scene.state.config.loudness.max_trim_db.max(0.0);
    let mut trim = edit.trim_db.clamp(-max, max);
    let label = t.tr("loudness-trim");
    ui.horizontal(|ui| {
        ui.label(RichText::new(&label).font(font(12.0)));
        let r = ui.add_enabled(
            !excluded,
            egui::Slider::new(&mut trim, -max..=max).step_by(0.5).suffix(" dB"),
        );
        let v = f64::from(trim);
        let owned = label.clone();
        r.widget_info(|| egui::WidgetInfo::slider(!excluded, v, owned.clone()));
        if r.double_clicked() {
            send(excluded, 0.0);
        } else if r.changed() {
            send(excluded, trim);
        }
    });
}
```

(If the design's toggles are preferred over `ui.checkbox`, reuse the drawing of `settings::toggle` by moving it to `widgets.rs` as `pub fn toggle`; keep the `Checkbox` accessible role either way — the tests query `Role::CheckBox`.)

`ui.rs`: `pub mod loudness_edit;`.

`app.rs` `ViewState`: `pub loudness_popup: Option<fp_model::TrackId>,` (default `None`).

`wave_panel.rs`, `edit_markers`: in the context menu, before `wave-reset`:

```rust
        if item(ui, "wave-loudness") {
            view_state.loudness_popup = Some(track);
            ui.close();
        }
```

`view_state` is borrowed mutably by the closure already for `wave_menu`; if the borrow checker objects, record the click in a local `let mut open_loudness = false;` and set `view_state.loudness_popup` after the menu.

The popup is drawn by `app.rs`, not by the wave panel, so that it stays open and keeps editing its own track whatever the players show (Review Focus 2). Next to `cue_window::show_all(&ctx, &scene, &mut self.view);` (~L729):

```rust
        crate::ui::loudness_edit::popup(&ctx, &scene, &mut self.view);
```

and in `loudness_edit.rs`:

```rust
/// N20: the Loudness… popup of the track the waveform menu chose; closed
/// with its window button, or by itself when the track leaves the library.
pub fn popup(ctx: &egui::Context, scene: &Scene<'_>, view: &mut super::app::ViewState) {
    let Some(id) = view.loudness_popup else {
        return;
    };
    let Some(track) = scene.state.library.get(id) else {
        view.loudness_popup = None;
        return;
    };
    let mut open = true;
    egui::Window::new(scene.i18n.tr("wave-loudness"))
        .id(egui::Id::new(("loudness-popup", id.0)))
        .collapsible(false)
        .resizable(false)
        .open(&mut open)
        .show(ctx, |ui| controls(ui, scene, track));
    if !open {
        view.loudness_popup = None;
    }
}
```

(Match the type and field name of `ViewState` in `app.rs`; `self.view` is what `cue_window::show_all` receives.)

`settings/carts.rs`: after the Exclusive row of the selected cart, when the cart has a track:

```rust
    if let Some(track) = cart.track.and_then(|id| scene.state.library.get(id)) {
        row(ui, &t.tr("wave-loudness").trim_end_matches('…').to_owned(), None, |ui| {
            ui.vertical(|ui| crate::ui::loudness_edit::controls(ui, scene, track));
        });
    }
```

(Use a dedicated label key if `trim_end_matches` reads badly in a locale; `settings-tab-loudness` "Loudness" is the natural row label and already exists after Task 4 — if Task 4 is not done yet, add `settings-tab-loudness` in this task instead.)

Locales: add `wave-loudness`, `loudness-exclude`, `loudness-trim` to all 11 files (es-ES: `Sonoridad…`, `Excluir de la normalización`, `Ajuste`).

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p fp-app --test loudness_ui --test markers_ui --test cartwall_ui --test i18n`
Expected: PASS.

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-app
git commit -m "feat(ui): exclude a track from normalization or trim it

The waveform menu opens a Loudness popup, and the cart editor shows the
same controls: the measurement, Exclude from normalization and a trim
in half-decibel steps (loudness normalization spec N20, N21).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: The Loudness tab in Settings (N22)

**Files:**
- Create: `crates/fp-app/src/ui/settings/loudness.rs`
- Modify: `crates/fp-app/src/ui/settings.rs` (`Section` enum ~L55, module list, the section dispatch ~L278, `SECTIONS` ~L456 → 10 entries, `restorable` ~L480, `SettingsDeps` gains `unmeasured: usize`), `crates/fp-app/src/ui/app.rs` (fills `unmeasured` like `outdated`), `crates/fp-app/src/services.rs` (`UnmeasuredCount`)
- Modify: the 11 `main.ftl` files (the `settings-*` keys)
- Test: `crates/fp-app/tests/loudness_ui.rs`, `crates/fp-app/tests/settings_layout.rs` (`SECTIONS` gains "Loudness"), `crates/fp-app/tests/settings_restore.rs`

**Interfaces:**
- Consumes: `LoudnessConfig` (plan 3), `fp_model::loudness_unmeasured` (plan 2).
- Produces: `pub struct UnmeasuredCount` in `fp_app::services` with `get(&mut self, state: &Arc<AppState>) -> usize` (scans once per snapshot, as `OutdatedCount`).

- [ ] **Step 1: Write the failing tests**

`tests/loudness_ui.rs`:

```rust
fn open_loudness_tab(s: AppState) -> (egui_kittest::Harness<'static, fp_app::ui::app::AppUi>, std::sync::Arc<support::Fake>) {
    let (mut h, fake) = harness(s);
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Loudness").click();
    h.run_steps(2);
    fake.take_sent();
    (h, fake)
}

#[test]
fn n22_the_toggle_turns_normalization_on() {
    let (mut h, fake) = open_loudness_tab(state(1, 1));
    h.get_by_label("Normalize loudness").click();
    h.run_steps(2);
    assert!(fake.state.load().config.loudness.normalize);
}

#[test]
fn n22_the_sliders_keep_their_ranges() {
    let mut s = state(1, 1);
    s.config.loudness.target_lufs = -6.0;
    let (mut h, fake) = open_loudness_tab(s);
    let target = h.get_by_role_and_label(Role::Slider, "Target");
    target.focus();
    h.run_steps(1);
    h.key_press(egui::Key::ArrowRight);
    h.run_steps(2);
    assert_eq!(fake.state.load().config.loudness.target_lufs, -6.0, "the top of the range");
    for label in ["True-peak ceiling", "Maximum boost", "Maximum trim"] {
        assert!(h.query_by_role_and_label(Role::Slider, label).is_some(), "{label}");
    }
    assert!(h.query_by_label("Changes apply to tracks loaded afterwards; what is playing keeps its gain.").is_some());
}

#[test]
fn n22_it_counts_the_tracks_still_waiting_for_a_measurement() {
    let mut s = state(1, 3);
    for t in s.library.iter_mut() {
        t.analyzed = true;
    }
    let (mut h, _fake) = open_loudness_tab(s);
    h.run_steps(2);
    assert!(h.query_by_label("3 tracks not measured yet").is_some());
}
```

`tests/settings_restore.rs`: add `("Loudness", SettingsSection::Loudness)` to the table at ~L77 (the test that every restorable section sends its own `RestoreDefaults`).

`tests/settings_layout.rs`: `const SECTIONS: [&str; 10]` with `"Loudness"` after `"Analysis"`.

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p fp-app --test loudness_ui n22 --test settings_restore --test settings_layout 2>&1 | tail -15`
Expected: FAIL (no "Loudness" section).

- [ ] **Step 3: Implement**

`settings/loudness.rs` (structure of `settings/meters.rs`):

```rust
//! Settings → Loudness (loudness normalization spec N22): normalization on
//! or off, the target, the true-peak ceiling and the maximum boost and
//! trim. Changes apply to tracks loaded afterwards (N3).

use egui::Ui;

use super::{note, row, slider, toggle, update};
use crate::ui::app::Scene;
use crate::ui::theme;

pub(super) fn section(ui: &mut Ui, scene: &Scene<'_>, unmeasured: usize) {
    let t = scene.i18n;
    let l = scene.state.config.loudness.clone();
    let mut on = l.normalize;
    let label = t.tr("settings-normalize");
    row(ui, &label, Some(&t.tr("settings-hint-normalize")), |ui| {
        if toggle(ui, &mut on, &label) {
            update(scene, |c| c.loudness.normalize = on);
        }
    });
    type Field = (&'static str, f32, std::ops::RangeInclusive<f32>, f64, &'static str, fn(&mut fp_model::Config, f32));
    let fields: [Field; 4] = [
        ("settings-normalize-target", l.target_lufs, -36.0..=-6.0, 0.5, " LUFS", |c, v| c.loudness.target_lufs = v),
        ("settings-normalize-ceiling", l.true_peak_ceiling_dbtp, -12.0..=0.0, 0.1, " dBTP", |c, v| c.loudness.true_peak_ceiling_dbtp = v),
        ("settings-normalize-max-boost", l.max_boost_db, 0.0..=24.0, 0.5, " dB", |c, v| c.loudness.max_boost_db = v),
        ("settings-normalize-max-trim", l.max_trim_db, 0.0..=24.0, 0.5, " dB", |c, v| c.loudness.max_trim_db = v),
    ];
    for (key, value, range, step, suffix, set) in fields {
        let mut v = value;
        let label = t.tr(key);
        row(ui, &label, None, |ui| {
            if slider(ui, &mut v, range, step, suffix, &label) {
                update(scene, |c| set(c, v));
            }
        });
    }
    note(ui, &t.tr("settings-hint-normalize-apply"), theme::NEUTRAL_400);
    if unmeasured > 0 {
        note(
            ui,
            &t.tr_args("settings-normalize-unmeasured", &[("count", unmeasured.into())]),
            theme::NEUTRAL_400,
        );
    }
}
```

(Check the exact signatures of `row`, `note`, `slider`, `toggle`, `update` in `settings.rs` — `note(ui, text, color)` and `row(ui, label, hint, control)` are `pub(super)`; `slider`, `toggle`, `update` are private to `settings` and visible to its child modules, as `meters.rs` uses them.)

`settings.rs`: `mod loudness;`; `Section::Loudness` after `Analysis`; dispatch `Section::Loudness => loudness::section(ui, scene, deps.unmeasured),`; `SECTIONS` becomes `[_; 10]` with `(Section::Loudness, icon::SPEAKER_SIMPLE_HIGH, "settings-tab-loudness")` after Analysis (any phosphor glyph not used by another section; check it renders in `tests/settings_layout.rs`); `restorable`: `Section::Loudness => Some(SettingsSection::Loudness)`; `SettingsDeps::unmeasured: usize` with doc "How many tracks wait for a loudness measurement (N22)".

`services.rs`: `UnmeasuredCount` — copy `OutdatedCount` (fields `snapshot`, `count`, `scans`; `get`, `scans`) with `fp_model::loudness_unmeasured(state)` in place of `outdated_tracks(state)` and the doc "`loudness_unmeasured`, remembered for the snapshot it was worked out on (Settings → Loudness reads it every frame)". `app.rs`: a field `unmeasured: crate::services::UnmeasuredCount` next to `outdated`, and `unmeasured: self.unmeasured.get(&state)` where `SettingsDeps` is built.

Locales: the nine `settings-*` keys of §8.4 in all 11 files (es-ES: `settings-tab-loudness = Sonoridad`, `settings-normalize = Normalizar la sonoridad`, `settings-hint-normalize = Reproduce cada pista y cada cart a la misma sonoridad integrada con una ganancia fija por pista.`, `settings-normalize-target = Objetivo`, `settings-normalize-ceiling = Techo de pico real`, `settings-normalize-max-boost = Ganancia máxima`, `settings-normalize-max-trim = Ajuste máximo`, `settings-hint-normalize-apply = Los cambios se aplican a las pistas que se carguen después; lo que suena conserva su ganancia.`, `settings-normalize-unmeasured = { $count } pistas aún sin medir`). Use Fluent plurals in the locales where `{ $count }` needs them (as `settings-analyse-outdated` does, if it does).

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p fp-app`
Expected: PASS.

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-app
git commit -m "feat(ui): add the Loudness tab to Settings

Normalization on or off, the target, the true-peak ceiling, the
maximum boost and trim, how many tracks still wait for a measurement,
and Restore defaults (loudness normalization spec N22).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: Remote API: read the loudness, set the edit (N23, N24)

**Files:**
- Modify: `crates/fp-remote/src/dto.rs` (`TrackDto` L83-95 + `track_dto` L323-345; `PlayerDto` L23-41 + `player_dto` L257-…; `PlayingCartDto` L148-152 + its builder ~L239), `crates/fp-remote/src/api.rs` (`Edit::SetTrackLoudness`, `edit_commands`), `crates/fp-remote/src/http/mod.rs:147` (route), `crates/fp-remote/src/http/handlers.rs` (body + handler next to `set_marker`)
- Test: `crates/fp-remote/tests/dto.rs`, `crates/fp-remote/tests/edit.rs`

**Interfaces:**
- Produces:
  - `pub struct LoudnessDto { pub integrated_lufs: Option<f32>, pub true_peak_dbtp: f32 }`, `pub struct LoudnessEditDto { pub excluded: bool, pub trim_db: f32 }`, `pub struct NormalizationDto { pub gain_db: f32, pub status: &'static str }` (all `Debug, Clone, PartialEq, Serialize`)
  - `TrackDto::{loudness: Option<LoudnessDto>, loudness_edit: LoudnessEditDto}`, `PlayerDto::normalization: Option<NormalizationDto>`, `PlayingCartDto::normalization: NormalizationDto`
  - `pub fn normalization_name(status: NormalizationStatus) -> &'static str`
  - `Edit::SetTrackLoudness { track: TrackId, excluded: bool, trim_db: f32 }`
  - `PUT /api/v1/tracks/{id}/loudness`

- [ ] **Step 1: Write the failing tests**

`tests/dto.rs`:

```rust
#[test]
fn n23_a_track_carries_its_loudness_and_edit() {
    let mut s = demo_state();
    let id = s.playlists.iter().next().unwrap().entries[0].track;
    s.library.get_mut(id).unwrap().loudness = Some(fp_model::LoudnessMeasurement {
        integrated_lufs: Some(-14.5),
        true_peak_dbtp: -0.5,
    });
    fp_model::apply(&mut s, Command::SetTrackLoudness { track: id, excluded: true, trim_db: 1.5 }).unwrap();
    let out = dto::track(&s, id).unwrap();
    assert_eq!(out.loudness, Some(dto::LoudnessDto { integrated_lufs: Some(-14.5), true_peak_dbtp: -0.5 }));
    assert_eq!(out.loudness_edit, dto::LoudnessEditDto { excluded: true, trim_db: 1.5 });
    let json = serde_json::to_value(&out).unwrap();
    assert_eq!(json["loudness_edit"]["excluded"], true);
}

#[test]
fn n23_status_names() {
    use fp_model::{GainLimit, NormalizationStatus as S};
    for (status, name) in [
        (S::Applied { limited: None }, "applied"),
        (S::Applied { limited: Some(GainLimit::TruePeak) }, "limited_true_peak"),
        (S::Applied { limited: Some(GainLimit::MaxBoost) }, "limited_max_boost"),
        (S::Off, "off"),
        (S::Excluded, "excluded"),
        (S::NotMeasured, "not_measured"),
        (S::Silent, "silent"),
        (S::BitPerfect, "bit_perfect"),
    ] {
        assert_eq!(dto::normalization_name(status), name);
    }
}

#[test]
fn n23_the_player_and_a_cart_report_the_gain_on_air() {
    let mut s = demo_state();
    s.config.loudness.normalize = true;
    for t in s.library.iter_mut() {
        t.loudness = Some(fp_model::LoudnessMeasurement { integrated_lufs: Some(-20.0), true_peak_dbtp: -10.0 });
    }
    let p = s.players[0].id;
    let stopped = dto::player(&s, &Playback::default(), p).unwrap();
    assert_eq!(stopped.normalization, None);
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    let out = dto::player(&s, &Playback::default(), p).unwrap();
    assert_eq!(out.normalization, Some(dto::NormalizationDto { gain_db: 4.0, status: "applied" }));
}
```

(For the cart, if `demo_state` has a cart with a track, fire it and assert `dto::cartwall(&s, &Playback::default()).playing[0].normalization`; otherwise assign one as plan 3's `with_cart` does.)

`tests/edit.rs`:

```rust
#[test]
fn n24_the_loudness_edit_is_planned_and_refused_out_of_range() {
    let s = demo_state();
    let t = s.playlists.iter().next().unwrap().entries[0].track;
    assert_eq!(
        plan_edit(&s, Edit::SetTrackLoudness { track: t, excluded: true, trim_db: -3.0 }).unwrap(),
        vec![Command::SetTrackLoudness { track: t, excluded: true, trim_db: -3.0 }]
    );
    let refused = plan_edit(&s, Edit::SetTrackLoudness { track: t, excluded: false, trim_db: 40.0 }).unwrap_err();
    assert_eq!(refused.status(), 409);
    assert!(refused.message().contains("trim"), "{}", refused.message());
    assert_eq!(
        plan_edit(&s, Edit::SetTrackLoudness { track: TrackId(9_999), excluded: false, trim_db: 0.0 }).unwrap_err(),
        ApiError::NotFound
    );
}
```

(Use the accessor the existing tests use for an `ApiError`'s status and text — `status()` exists; check the name of the message accessor in `api.rs` and adapt.) In the HTTP test that lists accepted edits (~L360-395), add:

```rust
        (
            "PUT",
            format!("/api/v1/tracks/{t}/loudness"),
            Some(json!({"excluded": true, "trim_db": 1.5})),
        ),
```

and a new async test:

```rust
#[tokio::test]
async fn n24_the_loudness_endpoint_refuses_bad_bodies() {
    let s = demo_state();
    let t = s.playlists.iter().next().unwrap().entries[0].track;
    let fake = support::FakeControl::new(s);
    for body in [
        json!({"excluded": true}),
        json!({"excluded": "yes", "trim_db": 0.0}),
        json!({"excluded": false, "trim_db": 0.0, "gain": 3}),
    ] {
        assert_eq!(
            call(&fake, "PUT", &format!("/api/v1/tracks/{t}/loudness"), Some(body.clone())).await,
            StatusCode::BAD_REQUEST,
            "{body}"
        );
    }
    assert_eq!(
        call(&fake, "PUT", &format!("/api/v1/tracks/{t}/loudness"), Some(json!({"excluded": false, "trim_db": 99.0}))).await,
        StatusCode::CONFLICT
    );
    assert_eq!(
        call(&fake, "PUT", "/api/v1/tracks/99999/loudness", Some(json!({"excluded": false, "trim_db": 0.0}))).await,
        StatusCode::NOT_FOUND
    );
}
```

(Match `FakeControl` construction and `call` to the existing async tests in this file; if `ApiError::Unavailable` maps to another status than 409, use that one — the point is "refused with the model's reason", as for other model refusals.)

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p fp-remote --test dto --test edit 2>&1 | tail -5`
Expected: FAIL to compile.

- [ ] **Step 3: Implement**

`dto.rs`: the three DTO structs; `TrackDto` gains `pub loudness: Option<LoudnessDto>, pub loudness_edit: LoudnessEditDto` filled in `track_dto` from `t.loudness` and `t.loudness_edit`; `normalization_name` (match as in the test table); `fn normalization_dto(n: &Normalization) -> NormalizationDto`; `PlayerDto::normalization` = `p.on_air.filter(|_| p.transport != Transport::Stopped).map(|n| normalization_dto(&n))`; `PlayingCartDto::normalization` = `normalization_dto(&pc.normalization)`.

`api.rs`: `Edit::SetTrackLoudness { track: TrackId, excluded: bool, trim_db: f32 }` and in `edit_commands`:

```rust
        Edit::SetTrackLoudness {
            track: t,
            excluded,
            trim_db,
        } => {
            track(t)?;
            // The model refuses a trim out of range with its reason (N24).
            vec![Command::SetTrackLoudness {
                track: t,
                excluded,
                trim_db,
            }]
        }
```

`handlers.rs`:

```rust
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoudnessBody {
    excluded: bool,
    trim_db: f32,
}

pub async fn set_loudness(
    State(ctx): State<Ctx>,
    Path(raw): Path<String>,
    JsonBody(b): JsonBody<LoudnessBody>,
) -> Reply {
    run_edit(
        &ctx,
        Edit::SetTrackLoudness {
            track: TrackId(id(&raw)?),
            excluded: b.excluded,
            trim_db: b.trim_db,
        },
    )
}
```

`http/mod.rs`, next to the marker routes: `.route("/tracks/{id}/loudness", put(h::set_loudness))`. SSE: `TrackDto` changes now include loudness, so the existing `track` event fires when a measurement arrives (no code; check `crates/fp-remote/src/events.rs` compares DTOs, and if it compares a narrower set of fields, add the loudness ones).

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p fp-remote`
Expected: PASS.

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-remote
git commit -m "feat(remote): read the loudness and set a track's loudness edit

Tracks carry their measurement and edit, players and carts the gain on
air with its status, and PUT /tracks/{id}/loudness excludes or trims a
track (loudness normalization spec N23, N24).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 6: User guide, technical docs, README and specs

**Files:**
- Modify: `README.md` (feature list: loudness normalization, one line in the style of the others)
- Modify: `docs/user/settings.md` (the Loudness tab: each control, its range, "applies to tracks loaded afterwards", the unmeasured count, Restore defaults), `docs/user/players.md` (the gain badge and its tooltip; "Not normalized" amber/muted and why), `docs/user/markers-and-mixing.md` (the Loudness… popup: measurement, exclude, trim, double-click reset), `docs/user/cartwall.md` (the cart editor's controls), `docs/user/bit-perfect.md` (no normalization on bit-perfect devices; the BP badge and the gain), the analysis section of `docs/user/settings.md` (the background loudness measurement; it runs after new files, and whether normalization is on or off), `docs/user/remote-control.md` (the new fields and the endpoint)
- Modify: `docs/technical/ui.md` (badge, tooltip lines, popup, Settings tab, `Message`), `docs/technical/remote-api.md` (the `loudness`, `loudness_edit`, `normalization` fields and statuses; the endpoint row next to the marker rows L152-153; the `track` event now also fires for a new measurement)
- Modify: `docs/superpowers/specs/2026-09-25-fauste-player-design.md` §8.3 (the gain badge next to BP) and §8.4 (the Loudness tab); `docs/superpowers/specs/2026-10-01-remote-control-design.md` (the endpoint row after the marker rows at L170-171, the DTO fields)
- Screenshots: if a guide page shows the player header or the Settings navigation, regenerate with `scripts/site/screenshots.sh` (Linux, Xvfb; see CLAUDE.md "Testing notes") and check every PNG; if that is not possible here, record it in the ledger as a follow-up for the maintainer.

- [ ] **Step 1: Write the user pages** (English only; `docs/i18n/` untouched). Describe behaviour in its own terms; no other products.
- [ ] **Step 2: Technical docs and specs** as listed.
- [ ] **Step 3: Check the guide builds and its links**

Run: `scripts/site/build.sh /tmp/fp-site && scripts/site/check-links.sh /tmp/fp-site` (needs `gh` or `FAUSTE_RELEASE_JSON`; if unavailable, run `scripts/site/mdbook.sh build docs/user` or skip with a ledger note).
Expected: no dead links.

- [ ] **Step 4: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add README.md docs
git commit -m "docs: describe loudness normalization in the guide and the API

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

## Self-review notes

- Spec coverage: N18 → Tasks 1–2; N19 → Tasks 1–2 (ruling: lines under the grid); N20, N21 → Task 3; N22 (and N2's button) → Task 4; §8.4 strings → Tasks 1, 3, 4 (each with its keys in 11 locales); N23, N24 → Task 5; N25 (MIDI nothing) and OSC nothing → no task, stated in the docs (Task 6); §11 docs and §13 spec lines → Task 6.
- Kittest details (exact `Fake` state mutation, slider keyboard steps, cart selection) are to be matched to the existing tests named in each step; the assertions are the contract.

## Execution notes (2026-10-14)

Executed after the bit-perfect work (PRs #66–#69) and loudness plans 3–4
(PRs #72–#73); every symbol was checked against that code. Deviations:

- `gain_badge` shows no badge for `BitPerfect` while `normalize` is off (a
  bit-perfect player plays at unity gain either way, and the model reports
  `BitPerfect` before `Off`, so every bit-perfect player would otherwise say
  "Not normalized" with normalization off by default).
- The badge tooltip adds `tip-gain-earlier-settings` when the settings, the
  measurement or the edit now give another gain than the one on air (Review
  Focus 1: the tooltip would otherwise name a target the gain was not worked
  out for).
- The exclude control is the design's toggle, moved to `widgets::toggle`;
  the popup is a title-less window with the track's title and a Close tile
  (`loudness-close`); `settings-tab-loudness` is added in Task 3.
- The cart editor has two rows (Exclude, with the measurement as hint; Trim).
- The trim's double-click is read from the pointer, triple clicks included
  (the slider's rail senses drags only; a double-click soon after the click
  that opened the popup is a triple click to egui).
- Settings → Loudness adds `settings-hint-normalize-bp`; the unmeasured count
  uses Fluent plurals; `OutdatedCount` and `UnmeasuredCount` share a private
  `SnapshotCount`.
- Task 6 adds a guide page, `docs/user/loudness.md`. Screenshots were not
  regenerated (Settings navigation and the player header changed).
