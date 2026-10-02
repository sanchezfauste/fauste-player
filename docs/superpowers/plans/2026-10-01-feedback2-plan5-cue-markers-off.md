# Cue Markers Off (Feedback 2, Plan 5) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** An operator can turn off cue-in and cue-out for every player (O15): with `players.use_cue_markers` off, each player entry plays from 0 to the end of the file, the markers are kept, the countdowns and the MIX rules follow the whole file, and the waveform shows the two marks dimmed.

**Architecture:**
- **One effective-range function in the model.** `Track::play_range(use_markers) -> PlayRange` is the single place that says where a player entry starts and ends. Every player consumer calls it with `state.config.players.use_cue_markers`: the source requests the engine receives (`request_from_cue_in`, `request_at`), `plan_for`, the session restore, MIX validation in `set_marker`, the countdowns in `ui/view.rs`, the remote DTOs and the remote seek bounds. Carts keep calling the marker-based methods (`cue_in_secs`, `known_cue_out_secs`, `play_length_secs`).
- **Config.** One new field, `PlayersConfig::use_cue_markers: bool` (default `true`). A bool has no range, so `Config::validate` needs no clamp; lenient loading is already generic (`fp-store/src/lenient.rs`) and a test pins it. "Restore defaults" in Players already resets the whole `players` group, so it is covered by a test only.
- **UI.** `MarkerFractions` gains `ignored: bool`; a pure `widgets::cue_edge_look(ignored)` says how the waveform paints the cue edges; Settings > Players gets a toggle.

**Tech Stack:** Rust, egui/eframe 0.36.2, egui_kittest 0.36.2. No new dependency.

**Spec:** `docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md` §6 (item O15). Roadmap: `docs/superpowers/plans/2026-10-01-operator-feedback-2-roadmap.md` (plan 5, branch `feat/cue-markers-off`). Markers are specified in `docs/superpowers/specs/` (Phase 2 spec P2.8, manual markers; spec §3 rules 9-12, 17-20 for transitions and countdowns).

## Global Constraints

- All code, identifiers, comments, docs and commit messages are in English. Never mention other products.
- Spec O15: "A new field, `players.use_cue_markers: bool`, defaults to `true` and loads leniently. It is independent of `players.auto_segue`, which already turns the MIX off."
- Spec O15: "When it is `false`, every player entry plays from 0 to the end of the file: automatic and manual cue-in and cue-out are ignored, but kept."
- Spec O15: "The rule lives in the model's effective-range function, which the engine requests, the countdowns and the MIX validation already share. One test per consumer."
- Spec O15: "It applies to players only; carts are unchanged."
- Spec O15: "The waveform draws the cue-in and cue-out marks dimmed while they are ignored."
- Spec O15: "Settings > Players gets a toggle "Use cue-in and cue-out", next to "Automatic mix at the MIX point"."
- No hardcoded product limits: the flag is a `Config` field with a documented default. The dim opacity is a visual constant in `ui/theme.rs`.
- Behaviour lives in `fp-model` as pure functions; the engine and the UI only execute and display. The UI never blocks. No `unwrap`, `expect`, `panic` or indexing outside tests (use `get`). A track whose duration is unknown (0) must never panic or produce NaN.
- Real-time safety is untouched: nothing in this plan runs on the device callback.
- UI strings are Fluent messages in `crates/fp-app/locales/en-US/main.ftl` (source) and `es-ES/main.ftl`, always both.
- Commands: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p <crate> --test <file> <name>` (one filter per command). Build only into the repo's `target/` (never set `CARGO_TARGET_DIR`).
- Commit only when fmt, clippy and the whole suite pass (`cargo test --workspace`). Commits end with the co-author trailer the harness provides. `CHANGELOG.md` is never edited by hand.
- Documentation is part of the change (Task 5).

## Review Focus

Failure modes the spec implies and its tests do not name; each has a test in the task named.

1. A track whose duration is still unknown (0 s) with the markers off: there is no end to plan to, so the plan must use `SOURCE_END`, the range length is 0, and nothing is NaN or negative. (Task 1 `an_unknown_duration_has_no_known_end`; Task 2 `an_unknown_duration_plans_to_the_source_end`.)
2. Toggling the setting while a player is on air: the track keeps playing (no restart, seek or stop); only the scheduled transition moves to the new end. A stopped player's preloaded source is requested again from the new start. (Task 2 `toggling_while_on_air_replans_without_touching_the_sound`, `toggling_re_preloads_the_next_entry_from_the_new_start`.)
3. A stale automatic MIX point beyond the cue-out (`segue >= cue_out`): ignored while the markers are on, honoured while they are off (the whole file is the range), ignored again when they are turned back on. (Task 2 `a_mix_point_past_the_cue_out_is_used_only_while_the_markers_are_off`.)
4. A `config.json` from before this version, or one with `"use_cue_markers": "yes"`: it loads with the default (`true`), keeps the other players fields, and warns once. (Task 1 `a_missing_or_bad_use_cue_markers_loads_as_on`.)
5. Carts and CUE: carts must keep their cue-in and cue-out when the setting is off; a player's CUE (pre-listen) follows the player's range, so what the operator hears in CUE is what the player will play. (Task 2 `carts_keep_their_markers_when_players_ignore_them`, `the_player_cue_starts_at_the_play_range_start`.)

## Decisions

- **No existing "effective-range function".** The spec calls it shared already, but the code had only `Track::cue_in_secs`, `cue_out_secs` and `known_cue_out_secs`, each used directly. This plan creates the function (`Track::play_range`) and moves every player consumer onto it. The three marker-based methods stay, for carts and for marker editing.
- **Name and shape.** `PlayRange { cue_in, cue_out, .. }` with `known_end() -> Option<f64>` and `length() -> f64`. With the markers on it equals today's behaviour exactly (cue-out falls back to the duration, "known" means a marker or a duration above 0). Off, it is `0 .. duration`.
- **Editing cue points while they are off.** `SetMarker` for `CueIn`/`CueOut` still validates against the real (kept) markers, so what the operator saves stays coherent when the setting is turned on again. `SetMarker` for intro, outro and MIX clamps into the effective range, because that is where the track plays; stored manual MIX points past the real cue-out are tolerated by `plan_for`'s existing filter when markers are on again.
- **Intro, outro and MIX are not cue markers.** They stay on, so the intro badge, the outro badge (counting down to the effective end) and the MIX overlap still work, bounded by the effective range. `auto_segue` stays the independent switch for MIX.
- **CUE follows the range.** `cue_entry` already uses `request_from_cue_in`; it starts at 0 when the setting is off. No separate rule.
- **Session restore.** A saved position at or past the effective end comes back at the effective start, as today for the cue-out.
- **Waveform when ignored.** The trimmed head and tail are not shaded (they will play); the cue-in and cue-out lines stay, drawn in `NEUTRAL_500` at `theme::CUE_EDGE_IGNORED_ALPHA` (0.35). Alt-drag marker editing is unchanged.
- **Remote.** Player `remaining_secs`, `elapsed_secs` (idle fallback) and the seek bounds follow the range. Cart DTOs are unchanged. The setting is already readable and writable through the existing config endpoints (it is a `Config` field); no new endpoint.
- **No hot restart of the playing source.** The setting changes where a track will end; it never moves a track already sounding (rule 10 of the on-air safety stance).

## File Structure

- Modify `crates/fp-model/src/config.rs`: `PlayersConfig::use_cue_markers`.
- Modify `crates/fp-model/src/track.rs`: `PlayRange`, `Track::play_range`.
- Modify `crates/fp-model/src/lib.rs`: export `PlayRange`.
- Modify `crates/fp-model/src/state.rs`: `request_from_cue_in`, `request_at`.
- Modify `crates/fp-model/src/reducer.rs`: `plan_for`, `set_marker`.
- Modify `crates/fp-model/src/session.rs`: restore position.
- Modify `crates/fp-app/src/ui/view.rs`: `player_view`, `playlist_times`, `MarkerFractions::ignored`.
- Modify `crates/fp-app/src/ui/widgets.rs`: `cue_edge_look`, waveform painting.
- Modify `crates/fp-app/src/ui/theme.rs`: `CUE_EDGE_IGNORED_ALPHA`.
- Modify `crates/fp-app/src/ui/settings.rs`: the toggle.
- Modify `crates/fp-app/locales/en-US/main.ftl`, `es-ES/main.ftl`.
- Modify `crates/fp-remote/src/dto.rs`, `crates/fp-remote/src/api.rs`.
- Create `crates/fp-model/tests/cue_markers_off.rs`; modify `crates/fp-model/tests/restore.rs`, `crates/fp-model/tests/cartwall.rs`, `crates/fp-store/src/lenient.rs` (unit test), `crates/fp-app/tests/view.rs`, `waveform_view.rs`, `settings.rs`, `crates/fp-remote/tests/dto.rs`, `api.rs`.
- Modify docs: `docs/user/settings.md`, `docs/user/markers-and-mixing.md`, `docs/technical/persistence.md`, `docs/technical/audio-engine.md`, `docs/technical/ui.md`, the spec (status note and "As built" under §6).

---

### Task 1: The setting and the effective range (model)

**Files:**
- Modify: `crates/fp-model/src/config.rs` (`PlayersConfig` and its `Default`, near line 230)
- Modify: `crates/fp-model/src/track.rs` (after `play_length_secs`, near line 237)
- Modify: `crates/fp-model/src/lib.rs` (the `pub use track::{...}` list)
- Create: `crates/fp-model/tests/cue_markers_off.rs`
- Modify: `crates/fp-model/tests/restore.rs`
- Modify: `crates/fp-store/src/lenient.rs` (the `tests` module)

**Interfaces:**
- Consumes: `Track::cue_in_secs()`, `cue_out_secs()`, `known_cue_out_secs()`, `duration_secs`.
- Produces: `PlayersConfig::use_cue_markers: bool` (default `true`); `fp_model::PlayRange { pub cue_in: f64, pub cue_out: f64 }` with `pub fn known_end(&self) -> Option<f64>` and `pub fn length(&self) -> f64`; `Track::play_range(&self, use_markers: bool) -> PlayRange`.

- [ ] **Step 1: Write the failing tests**

Create `crates/fp-model/tests/cue_markers_off.rs`:

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback 2 spec O15: players may ignore cue-in and cue-out.

mod common;

use common::fixture;
use fp_model::{Command, Config, MarkerKind, TrackId, apply};

/// Gives every track of `state` a cue-in and a cue-out (manual).
fn mark_all(state: &mut fp_model::AppState, cue_in: f64, cue_out: f64) {
    let tracks: Vec<TrackId> = state.library.iter().map(|t| t.id).collect();
    for track in tracks {
        for (kind, secs) in [(MarkerKind::CueIn, cue_in), (MarkerKind::CueOut, cue_out)] {
            apply(
                state,
                Command::SetMarker {
                    track,
                    kind,
                    secs: Some(secs),
                },
            )
            .unwrap();
        }
    }
}

#[test]
fn the_setting_defaults_to_on() {
    assert!(Config::default().players.use_cue_markers);
}

#[test]
fn the_range_follows_the_markers_when_they_are_used() {
    let mut state = fixture(1);
    mark_all(&mut state, 5.0, 170.0);
    let t = state.library.iter().next().unwrap();
    let r = t.play_range(true);
    assert_eq!((r.cue_in, r.cue_out), (5.0, 170.0));
    assert_eq!(r.known_end(), Some(170.0));
    assert_eq!(r.length(), 165.0);
}

#[test]
fn the_range_is_the_whole_file_when_the_markers_are_ignored_and_they_are_kept() {
    let mut state = fixture(1);
    mark_all(&mut state, 5.0, 170.0);
    let t = state.library.iter().next().unwrap();
    let r = t.play_range(false);
    assert_eq!((r.cue_in, r.cue_out), (0.0, 180.0));
    assert_eq!(r.known_end(), Some(180.0));
    assert_eq!(r.length(), 180.0);
    assert_eq!(t.cue_in_secs(), 5.0, "kept");
    assert_eq!(t.cue_out_secs(), 170.0, "kept");
}

#[test]
fn an_unknown_duration_has_no_known_end() {
    let mut state = fixture(1);
    for t in state.library.iter_mut() {
        t.duration_secs = 0.0;
    }
    let t = state.library.iter().next().unwrap();
    for use_markers in [true, false] {
        let r = t.play_range(use_markers);
        assert_eq!(r.known_end(), None, "{use_markers}");
        assert_eq!(r.length(), 0.0);
        assert!(r.cue_in.is_finite() && r.cue_out.is_finite());
    }
}
```

Append to `crates/fp-model/tests/restore.rs` (and in `altered()` add `c.players.use_cue_markers = false;` after the `auto_segue` line):

```rust
#[test]
fn players_restores_the_cue_markers_switch() {
    let mut c = altered();
    assert!(!c.players.use_cue_markers);
    restore_defaults(&mut c, SettingsSection::Players);
    assert!(c.players.use_cue_markers);
}
```

Add to the `tests` module of `crates/fp-store/src/lenient.rs`:

```rust
    #[test]
    fn a_missing_or_bad_use_cue_markers_loads_as_on() {
        let old: serde_json::Value =
            serde_json::from_str(r#"{"players":{"fade_ms":2000}}"#).unwrap();
        let mut warnings = Vec::new();
        let c = config_from_value(&old, &mut warnings);
        assert!(c.players.use_cue_markers);
        assert!(warnings.is_empty(), "{warnings:?}");

        let bad: serde_json::Value =
            serde_json::from_str(r#"{"players":{"fade_ms":2000,"use_cue_markers":"yes"}}"#)
                .unwrap();
        let mut warnings = Vec::new();
        let c = config_from_value(&bad, &mut warnings);
        assert!(c.players.use_cue_markers);
        assert_eq!(c.players.fade_ms, 2000, "the other fields are kept");
        assert_eq!(warnings.len(), 1, "{warnings:?}");

        let off: serde_json::Value =
            serde_json::from_str(r#"{"players":{"use_cue_markers":false}}"#).unwrap();
        let c = config_from_value(&off, &mut Vec::new());
        assert!(!c.players.use_cue_markers);
    }
```

- [ ] **Step 2: Run them to verify they fail**

Run each: `cargo test -p fp-model --test cue_markers_off the_setting_defaults_to_on`
Expected: FAIL to compile (`no field use_cue_markers`, `no method play_range`). Same for `cargo test -p fp-model --test restore players_restores_the_cue_markers_switch` and `cargo test -p fp-store a_missing_or_bad_use_cue_markers_loads_as_on`.

- [ ] **Step 3: Implement**

In `crates/fp-model/src/config.rs`, `PlayersConfig` (after `auto_segue`) and `Default`:

```rust
    pub auto_segue: bool,
    /// Players honour each track's cue-in and cue-out. Off, every player
    /// entry plays from 0 to the end of the file; the markers are kept.
    /// Carts always use theirs. Independent of `auto_segue`.
    pub use_cue_markers: bool,
```

```rust
            auto_segue: true,
            use_cue_markers: true,
```

In `crates/fp-model/src/track.rs`, before `impl Track` add:

```rust
/// Where a player plays a track: from `cue_in` to `cue_out`. See
/// `Track::play_range`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayRange {
    pub cue_in: f64,
    /// The cue-out, or the duration; 0 while the duration is unknown.
    pub cue_out: f64,
    end_known: bool,
}

impl PlayRange {
    /// The end when it is actually known: a marker, or a duration from
    /// analysis. `None` while the duration is unknown.
    pub fn known_end(&self) -> Option<f64> {
        self.end_known.then_some(self.cue_out)
    }

    /// Audible length between the start and the end; never negative.
    pub fn length(&self) -> f64 {
        (self.cue_out - self.cue_in).max(0.0)
    }
}
```

and inside `impl Track`, after `play_length_secs`:

```rust
    /// The range a player plays (feedback 2 spec O15). With `use_markers`
    /// it is cue-in to cue-out; without, the whole file. The markers are
    /// never changed. Carts do not use this: they always follow their markers.
    pub fn play_range(&self, use_markers: bool) -> PlayRange {
        if use_markers {
            PlayRange {
                cue_in: self.cue_in_secs(),
                cue_out: self.cue_out_secs(),
                end_known: self.known_cue_out_secs().is_some(),
            }
        } else {
            PlayRange {
                cue_in: 0.0,
                cue_out: self.duration_secs.max(0.0),
                end_known: self.duration_secs > 0.0,
            }
        }
    }
```

In `crates/fp-model/src/lib.rs` add `PlayRange` to the `pub use track::{ ... }` list.

- [ ] **Step 4: Run the tests to verify they pass**

Run, one per command:
- `cargo test -p fp-model --test cue_markers_off the_setting_defaults_to_on`
- `cargo test -p fp-model --test cue_markers_off the_range_follows`
- `cargo test -p fp-model --test cue_markers_off the_range_is_the_whole_file`
- `cargo test -p fp-model --test cue_markers_off an_unknown_duration_has_no_known_end`
- `cargo test -p fp-model --test restore players_restores_the_cue_markers_switch`
- `cargo test -p fp-store a_missing_or_bad_use_cue_markers_loads_as_on`

Expected: PASS. Then `cargo test -p fp-model --lib` (config round-trip tests) passes.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-model crates/fp-store
git commit -m "feat(model): add the use_cue_markers setting and Track::play_range"
```

---

### Task 2: Players follow the range: engine requests, transitions, restore, MIX validation

**Files:**
- Modify: `crates/fp-model/src/state.rs` (`request_from_cue_in`, `request_at`, near lines 82-110)
- Modify: `crates/fp-model/src/reducer.rs` (`plan_for` near line 605, `set_marker` near line 700)
- Modify: `crates/fp-model/src/session.rs` (near line 190)
- Test: `crates/fp-model/tests/cue_markers_off.rs`, `crates/fp-model/tests/cartwall.rs`

**Interfaces:**
- Consumes: `Track::play_range(bool) -> PlayRange`, `PlayRange::{known_end, length, cue_in, cue_out}`, `config.players.use_cue_markers`.
- Produces: player source requests starting at `range.cue_in`; `plan_for` ending at `range.known_end()` (else `SOURCE_END`); MIX validated against the range. No signature changes.

- [ ] **Step 1: Write the failing tests**

Add to `crates/fp-model/tests/cue_markers_off.rs` (extend the imports to `use common::{entries, fixture, p0};` and `use fp_model::{AppState, Command, Config, EngineAction, MarkerKind, PlayMode, PlayerId, PlayerSession, RestoreParts, SOURCE_END, TrackId, TransitionPlan, Transport, apply};`):

```rust
fn set_use(state: &mut AppState, on: bool) -> Vec<EngineAction> {
    let mut config = state.config.clone();
    config.players.use_cue_markers = on;
    apply(state, Command::UpdateConfig(Box::new(config))).unwrap()
}

fn start_secs(actions: &[EngineAction]) -> Option<f64> {
    actions.iter().find_map(|a| match a {
        EngineAction::StartCurrent { request, .. } => Some(request.from_secs),
        _ => None,
    })
}

/// The last plan scheduled for `p` in `actions`, if any.
fn scheduled(actions: &[EngineAction], p: PlayerId) -> Option<Option<TransitionPlan>> {
    actions.iter().rev().find_map(|a| match a {
        EngineAction::Schedule { player, plan } if *player == p => Some(*plan),
        _ => None,
    })
}

fn preload_secs(actions: &[EngineAction], p: PlayerId) -> Option<f64> {
    actions.iter().find_map(|a| match a {
        EngineAction::Preload {
            player,
            request: Some(r),
        } if *player == p => Some(r.from_secs),
        _ => None,
    })
}

#[test]
fn the_engine_starts_a_player_entry_at_0_when_the_markers_are_ignored() {
    for (on, want) in [(true, 5.0), (false, 0.0)] {
        let mut state = fixture(2);
        mark_all(&mut state, 5.0, 170.0);
        set_use(&mut state, on);
        let p = p0(&state);
        let actions = apply(&mut state, Command::Play(p)).unwrap();
        assert_eq!(start_secs(&actions), Some(want), "use markers {on}");
    }
}

#[test]
fn restart_goes_back_to_the_start_of_the_range() {
    for (on, want) in [(true, 5.0), (false, 0.0)] {
        let mut state = fixture(2);
        mark_all(&mut state, 5.0, 170.0);
        set_use(&mut state, on);
        let p = p0(&state);
        apply(&mut state, Command::Play(p)).unwrap();
        let actions = apply(&mut state, Command::Restart(p)).unwrap();
        assert!(
            actions.iter().any(|a| matches!(
                a,
                EngineAction::Seek { player, secs } if *player == p && *secs == want
            )),
            "use markers {on}: {actions:?}"
        );
    }
}

#[test]
fn the_player_cue_starts_at_the_play_range_start() {
    for (on, want) in [(true, 5.0), (false, 0.0)] {
        let mut state = fixture(2);
        mark_all(&mut state, 5.0, 170.0);
        set_use(&mut state, on);
        let p = p0(&state);
        let actions = apply(&mut state, Command::ToggleCue(p)).unwrap();
        let from = actions.iter().find_map(|a| match a {
            EngineAction::StartCue { request, .. } => Some(request.from_secs),
            _ => None,
        });
        assert_eq!(from, Some(want), "use markers {on}");
    }
}

#[test]
fn a_transition_is_planned_at_the_end_of_the_range() {
    for (on, want) in [(true, 170.0), (false, 180.0)] {
        let mut state = fixture(3);
        mark_all(&mut state, 5.0, 170.0);
        set_use(&mut state, on);
        let p = p0(&state);
        let actions = apply(&mut state, Command::Play(p)).unwrap();
        assert_eq!(
            scheduled(&actions, p),
            Some(Some(TransitionPlan::StartNextAt {
                at_secs: want,
                fade_current_until_secs: None
            })),
            "use markers {on}"
        );
        // Single mode stops at the same end.
        apply(&mut state, Command::SetMode(p, PlayMode::Single)).unwrap();
        let again = apply(&mut state, Command::Restart(p)).unwrap();
        let _ = again;
        let plan = fp_model::plan_for(&state, state.player(p).unwrap());
        assert_eq!(plan, Some(TransitionPlan::StopAt { at_secs: want }));
    }
}

#[test]
fn a_mix_point_past_the_cue_out_is_used_only_while_the_markers_are_off() {
    let mut state = fixture(3);
    mark_all(&mut state, 5.0, 170.0);
    for t in state.library.iter_mut() {
        t.markers.set_auto(MarkerKind::SegueStart, Some(175.0));
    }
    let p = p0(&state);
    apply(&mut state, Command::Play(p)).unwrap();
    let plan = |s: &AppState| fp_model::plan_for(s, s.player(p).unwrap());
    assert_eq!(
        plan(&state),
        Some(TransitionPlan::StartNextAt {
            at_secs: 170.0,
            fade_current_until_secs: None
        }),
        "stale against the cue-out"
    );
    set_use(&mut state, false);
    assert_eq!(
        plan(&state),
        Some(TransitionPlan::StartNextAt {
            at_secs: 175.0,
            fade_current_until_secs: Some(180.0)
        })
    );
    set_use(&mut state, true);
    assert_eq!(
        plan(&state),
        Some(TransitionPlan::StartNextAt {
            at_secs: 170.0,
            fade_current_until_secs: None
        })
    );
}

#[test]
fn the_mix_point_is_validated_against_the_effective_range() {
    for (on, want) in [(true, 170.0), (false, 178.0)] {
        let mut state = fixture(2);
        mark_all(&mut state, 5.0, 170.0);
        set_use(&mut state, on);
        let t = state.library.iter().next().unwrap().id;
        apply(
            &mut state,
            Command::SetMarker {
                track: t,
                kind: MarkerKind::SegueStart,
                secs: Some(178.0),
            },
        )
        .unwrap();
        let m = state.library.get(t).unwrap().markers.segue_start.unwrap();
        assert_eq!(m.secs, want, "use markers {on}");
    }
}

#[test]
fn the_cue_points_themselves_are_still_validated_against_the_kept_markers() {
    let mut state = fixture(2);
    mark_all(&mut state, 5.0, 170.0);
    set_use(&mut state, false);
    let t = state.library.iter().next().unwrap().id;
    let refused = apply(
        &mut state,
        Command::SetMarker {
            track: t,
            kind: MarkerKind::CueIn,
            secs: Some(171.0),
        },
    );
    assert!(refused.is_err(), "cue-in past the kept cue-out");
}

#[test]
fn a_restored_position_past_the_end_comes_back_at_the_start_of_the_range() {
    for (saved, want) in [(179.0, 179.0), (240.0, 0.0)] {
        let mut state = fixture(3);
        mark_all(&mut state, 0.5, 100.0);
        state.config.players.use_cue_markers = false;
        let p = p0(&state);
        apply(&mut state, Command::Play(p)).unwrap();
        let sessions: Vec<PlayerSession> = state.sessions(|_| saved);
        let parts = RestoreParts {
            config: state.config.clone(),
            library: state.library.clone(),
            playlists: state.playlists.clone(),
            cart_pages: state.cartwall.pages.clone(),
            cartwall_session: state.cartwall.session(),
            ids: state.ids.clone(),
        };
        let (restored, actions) = AppState::restore(parts, &sessions, "Main");
        let at = actions.iter().find_map(|a| match a {
            EngineAction::LoadPaused { player, request } if *player == p => {
                Some(request.from_secs)
            }
            _ => None,
        });
        assert_eq!(at, Some(want), "saved at {saved}");
        assert_eq!(restored.player(p).unwrap().transport, Transport::Paused);
    }
}

#[test]
fn an_unknown_duration_plans_to_the_source_end() {
    let mut state = fixture(3);
    for t in state.library.iter_mut() {
        t.duration_secs = 0.0;
    }
    set_use(&mut state, false);
    let p = p0(&state);
    let actions = apply(&mut state, Command::Play(p)).unwrap();
    assert_eq!(
        scheduled(&actions, p),
        Some(Some(TransitionPlan::StartNextAt {
            at_secs: SOURCE_END,
            fade_current_until_secs: None
        }))
    );
}

#[test]
fn toggling_re_preloads_the_next_entry_from_the_new_start() {
    let mut state = fixture(3);
    mark_all(&mut state, 5.0, 170.0);
    let p = p0(&state);
    let off = set_use(&mut state, false);
    assert_eq!(preload_secs(&off, p), Some(0.0));
    let on = set_use(&mut state, true);
    assert_eq!(preload_secs(&on, p), Some(5.0));
}

#[test]
fn toggling_while_on_air_replans_without_touching_the_sound() {
    let mut state = fixture(3);
    mark_all(&mut state, 5.0, 170.0);
    let p = p0(&state);
    apply(&mut state, Command::Play(p)).unwrap();
    let actions = set_use(&mut state, false);
    assert_eq!(
        scheduled(&actions, p),
        Some(Some(TransitionPlan::StartNextAt {
            at_secs: 180.0,
            fade_current_until_secs: None
        }))
    );
    assert!(
        !actions.iter().any(|a| matches!(
            a,
            EngineAction::StartCurrent { .. }
                | EngineAction::Seek { .. }
                | EngineAction::StopNow { .. }
                | EngineAction::Crossfade { .. }
        )),
        "{actions:?}"
    );
    assert_eq!(state.player(p).unwrap().transport, Transport::Playing);
}
```

(If `Command::ToggleCue` needs the player to have a next entry, `fixture(2)` provides it.)

Add to `crates/fp-model/tests/cartwall.rs`:

```rust
#[test]
fn carts_keep_their_markers_when_players_ignore_them() {
    let mut state = fixture(1);
    let c = load(&mut state, 0, 10.0);
    let mut config = state.config.clone();
    config.players.use_cue_markers = false;
    apply(&mut state, Command::UpdateConfig(Box::new(config))).unwrap();
    let actions = apply(&mut state, Command::FireCart(c)).unwrap();
    let requests = started(&actions);
    let request = requests.first().unwrap();
    assert_eq!(request.from_secs, 0.5, "the cart cue-in is kept");
    assert_eq!(request.until_secs, 9.5, "the cart cue-out is kept");
}
```

- [ ] **Step 2: Run them to verify they fail**

Run each, e.g. `cargo test -p fp-model --test cue_markers_off the_engine_starts_a_player_entry_at_0`, then `a_transition_is_planned_at_the_end_of_the_range`, `a_mix_point_past_the_cue_out`, `the_mix_point_is_validated`, `a_restored_position_past_the_end`, `toggling_re_preloads`, `toggling_while_on_air`, `the_player_cue_starts`, `restart_goes_back`, and `cargo test -p fp-model --test cartwall carts_keep_their_markers`.
Expected: the `false` cases FAIL (still 5.0 / 170.0); the cart test passes already (it pins that the cart is not changed), as do the `true` cases. `the_cue_points_themselves_are_still_validated_against_the_kept_markers` and `an_unknown_duration_plans_to_the_source_end` also already pass: they are guards for the change.

- [ ] **Step 3: Implement**

`crates/fp-model/src/state.rs`:

```rust
    /// A request that starts the entry at the start of its play range: its
    /// cue-in, or 0 when players ignore cue markers.
    pub fn request_from_cue_in(&self, entry: EntryId) -> Option<SourceRequest> {
        let track = self.track_for_entry(entry)?;
        let range = track.play_range(self.config.players.use_cue_markers);
        Some(SourceRequest {
            entry,
            track: track.id,
            path: track.path.clone(),
            from_secs: range.cue_in,
            format: track.format,
        })
    }
```

and in `request_at` replace the non-finite fallback:

```rust
        } else {
            track.play_range(self.config.players.use_cue_markers).cue_in
        };
```

`crates/fp-model/src/reducer.rs`, `plan_for`:

```rust
    let track = state.track_for_entry(current)?;
    let range = track.play_range(state.config.players.use_cue_markers);
    let end = range.known_end().unwrap_or(SOURCE_END);
```
and
```rust
    let segue = track
        .segue_start_secs()
        .filter(|s| *s >= range.cue_in && *s < end);
```

`set_marker` (keep `cue_in`, `cue_out` for the `CueIn`/`CueOut` arms, add the range for the inner markers):

```rust
    let use_markers = state.config.players.use_cue_markers;
    let t = state
        .library
        .get(track)
        .ok_or(ModelError::UnknownTrack(track))?;
    let cue_in = t.cue_in_secs();
    let cue_out = t.known_cue_out_secs();
    let range = t.play_range(use_markers);
```
and the last arm:
```rust
            _ => v
                .max(range.cue_in)
                .min(range.known_end().unwrap_or(f64::INFINITY)),
```
(Doc comment: cue points are validated against the kept markers; intro, outro and MIX against the effective range.) The post-edit `clamp_manual_inner` block stays as it is.

`crates/fp-model/src/session.rs`:

```rust
                let position = match state.track_for_entry(current) {
                    Some(t) => {
                        let range = t.play_range(state.config.players.use_cue_markers);
                        if range.known_end().is_some_and(|out| saved >= out) {
                            range.cue_in
                        } else {
                            saved
                        }
                    }
                    None => saved,
                };
```
Update the comment above it: "past the end of the play range".

- [ ] **Step 4: Run the tests to verify they pass**

Re-run every command from Step 2. Expected: PASS. Then `cargo test -p fp-model` (whole crate: existing transition, marker, session and cartwall tests unchanged).

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-model
git commit -m "feat(model): players play the whole file when cue markers are off"
```

---

### Task 3: Countdowns and remote follow the range

**Files:**
- Modify: `crates/fp-app/src/ui/view.rs` (`player_view` near line 130, `playlist_times` near line 229)
- Modify: `crates/fp-remote/src/dto.rs` (`player_dto`, near line 257)
- Modify: `crates/fp-remote/src/api.rs` (`O::Seek`, near line 166)
- Test: `crates/fp-app/tests/view.rs`, `crates/fp-remote/tests/dto.rs`, `crates/fp-remote/tests/api.rs`

**Interfaces:**
- Consumes: `Track::play_range(bool)`, `PlayRange::{cue_in, cue_out, length}`, `state.config.players.use_cue_markers`.
- Produces: `PlayerView.remaining`/`elapsed`/`outro`, `PlaylistTimes`, `PlayerDto.elapsed_secs`/`remaining_secs` and the `Seek` bounds computed from the effective range. Cart views and DTOs are not touched.

- [ ] **Step 1: Write the failing tests**

Add to `crates/fp-app/tests/view.rs`:

```rust
fn mark(s: &mut AppState, track: fp_model::TrackId, kind: MarkerKind, secs: f64) {
    apply(
        s,
        Command::SetMarker {
            track,
            kind,
            secs: Some(secs),
        },
    )
    .unwrap();
}

fn use_markers(s: &mut AppState, on: bool) {
    let mut config = s.config.clone();
    config.players.use_cue_markers = on;
    apply(s, Command::UpdateConfig(Box::new(config))).unwrap();
}

#[test]
fn the_countdown_and_the_outro_follow_the_play_range() {
    let (mut s, e, p) = state(2);
    let t = s.playlists.entry(e[0]).unwrap().track;
    mark(&mut s, t, MarkerKind::CueOut, 190.0);
    s.library
        .get_mut(t)
        .unwrap()
        .markers
        .set_auto(MarkerKind::OutroStart, Some(170.0));
    apply(&mut s, Command::Play(p)).unwrap();
    let v = player_view(&s, p, Some(185.0), 0.0).unwrap();
    assert_eq!((v.remaining, v.outro), (5.0, Some(5.0)));
    use_markers(&mut s, false);
    let v = player_view(&s, p, Some(185.0), 0.0).unwrap();
    assert_eq!((v.remaining, v.outro), (15.0, Some(15.0)));
    assert!(!v.end_warning, "15 s is outside the default 10 s warning");
}

#[test]
fn a_waiting_player_shows_the_start_of_the_range_as_elapsed() {
    let (mut s, e, p) = state(2);
    let t = s.playlists.entry(e[0]).unwrap().track;
    mark(&mut s, t, MarkerKind::CueIn, 12.0);
    assert_eq!(player_view(&s, p, None, 0.0).unwrap().elapsed, 12.0);
    use_markers(&mut s, false);
    assert_eq!(player_view(&s, p, None, 0.0).unwrap().elapsed, 0.0);
}

#[test]
fn playlist_times_follow_the_play_range() {
    let (mut s, e, p) = state(3);
    for entry in &e {
        let t = s.playlists.entry(*entry).unwrap().track;
        mark(&mut s, t, MarkerKind::CueIn, 10.0);
        mark(&mut s, t, MarkerKind::CueOut, 190.0);
    }
    let playlist = s.playlists.first_id().unwrap();
    assert_eq!(playlist_times(&s, p, playlist, &[]).total, 540.0);
    use_markers(&mut s, false);
    assert_eq!(playlist_times(&s, p, playlist, &[]).total, 600.0);
}
```

Add to `crates/fp-remote/tests/dto.rs`:

```rust
fn cut_the_current_track_at(s: &mut fp_model::AppState, p: PlayerId, secs: f64) {
    let entry = s.player(p).unwrap().current.unwrap();
    let track = s.playlists.entry(entry).unwrap().track;
    fp_model::apply(
        s,
        Command::SetMarker {
            track,
            kind: MarkerKind::CueOut,
            secs: Some(secs),
        },
    )
    .unwrap();
}

#[test]
fn a_player_reports_its_remaining_time_to_the_play_range() {
    let mut s = demo_state();
    let p = s.players[0].id;
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    cut_the_current_track_at(&mut s, p, 100.0);
    let playback = Playback {
        players: vec![(p, 30.0)],
        ..Default::default()
    };
    assert_eq!(
        dto::player(&s, &playback, p).unwrap().remaining_secs,
        Some(70.0)
    );
    s.config.players.use_cue_markers = false;
    assert_eq!(
        dto::player(&s, &playback, p).unwrap().remaining_secs,
        Some(150.0)
    );
}
```

Add to `crates/fp-remote/tests/api.rs`:

```rust
#[test]
fn seek_bounds_follow_the_play_range() {
    let mut s = demo_state();
    let p = s.players[0].id;
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    let entry = s.player(p).unwrap().current.unwrap();
    let track = s.playlists.entry(entry).unwrap().track;
    fp_model::apply(
        &mut s,
        Command::SetMarker {
            track,
            kind: fp_model::MarkerKind::CueOut,
            secs: Some(100.0),
        },
    )
    .unwrap();
    assert_eq!(plan(&s, O::Seek(p, 150.0)).unwrap_err().status(), 400);
    s.config.players.use_cue_markers = false;
    assert_eq!(
        plan(&s, O::Seek(p, 150.0)).unwrap(),
        vec![Command::Seek(p, 150.0)]
    );
}
```

- [ ] **Step 2: Run them to verify they fail**

Run each: `cargo test -p fp-app --test view the_countdown_and_the_outro_follow_the_play_range`, `... a_waiting_player_shows_the_start`, `... playlist_times_follow_the_play_range`, `cargo test -p fp-remote --test dto a_player_reports_its_remaining_time`, `cargo test -p fp-remote --test api seek_bounds_follow`.
Expected: FAIL on the `use markers off` assertions (values from the markers).

- [ ] **Step 3: Implement**

`crates/fp-app/src/ui/view.rs`, `player_view` (replace the `pos`/`cue_out` lines):

```rust
    let range = track.play_range(state.config.players.use_cue_markers);
    let pos = position
        .filter(|v| v.is_finite())
        .unwrap_or(range.cue_in);
    let cue_out = range.cue_out;
```
(the rest, `view.remaining = (cue_out - pos).max(0.0)` and the outro, already use `cue_out`.)

`playlist_times`:

```rust
    let use_markers = state.config.players.use_cue_markers;
    ...
        let range = track.play_range(use_markers);
        let len = range.length();
        total += len;
        ...
                .map_or(range.cue_in, |(_, s)| *s);
            elapsed += (pos - range.cue_in).clamp(0.0, len);
```

`crates/fp-remote/src/dto.rs`, `player_dto`:

```rust
    let use_markers = model.config.players.use_cue_markers;
    let current = p.current.and_then(|e| model.track_for_entry(e));
    let elapsed = current.map(|t| {
        playback
            .player_position(p.id)
            .filter(|v| v.is_finite())
            .unwrap_or_else(|| t.play_range(use_markers).cue_in)
    });
    ...
        remaining_secs: current
            .zip(elapsed)
            .map(|(t, e)| (t.play_range(use_markers).cue_out - e).max(0.0)),
```
(Leave `cartwall()` as is.)

`crates/fp-remote/src/api.rs`, `O::Seek`:

```rust
            let range = track.play_range(state.config.players.use_cue_markers);
            let (from, to) = (range.cue_in, range.cue_out);
```

- [ ] **Step 4: Run the tests to verify they pass**

Re-run the Step 2 commands: PASS. Then `cargo test -p fp-app --test view` and `cargo test -p fp-remote` (existing tests unchanged).

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-app crates/fp-remote
git commit -m "feat(ui): countdowns and the remote API follow the play range"
```

---

### Task 4: The waveform dims the ignored cue marks

**Files:**
- Modify: `crates/fp-app/src/ui/view.rs` (`MarkerFractions`, `player_view`)
- Modify: `crates/fp-app/src/ui/widgets.rs` (`cue_edge_look`, the trimmed-region block near line 1203)
- Modify: `crates/fp-app/src/ui/theme.rs` (constant)
- Test: `crates/fp-app/tests/view.rs`, `crates/fp-app/tests/waveform_view.rs`

**Interfaces:**
- Consumes: `state.config.players.use_cue_markers`; the existing `MarkerFractions { position, cue_in, intro_end, outro_start, segue_start, cue_out }`.
- Produces: `MarkerFractions.ignored: bool` (default `false`, set by `player_view` to `!use_cue_markers`); `widgets::CueEdgeLook { pub shade_trimmed: bool, pub line_alpha: f32 }`; `widgets::cue_edge_look(ignored: bool) -> CueEdgeLook`; `theme::CUE_EDGE_IGNORED_ALPHA: f32 = 0.35`.

- [ ] **Step 1: Write the failing tests**

Add to `crates/fp-app/tests/view.rs`:

```rust
#[test]
fn the_view_says_when_the_cue_marks_are_ignored() {
    let (mut s, e, p) = state(2);
    let t = s.playlists.entry(e[0]).unwrap().track;
    mark(&mut s, t, MarkerKind::CueIn, 12.0);
    let on = player_view(&s, p, None, 0.0).unwrap().markers;
    assert!(!on.ignored);
    use_markers(&mut s, false);
    let off = player_view(&s, p, None, 0.0).unwrap().markers;
    assert!(off.ignored);
    assert_eq!(off.cue_in, on.cue_in, "the marks are kept, only dimmed");
}
```

Add to `crates/fp-app/tests/waveform_view.rs` (extend the `use fp_app::ui::widgets::{...}` list with `CueEdgeLook, cue_edge_look`):

```rust
#[test]
fn ignored_cue_marks_are_dimmed_and_do_not_shade_the_file() {
    assert_eq!(
        cue_edge_look(false),
        CueEdgeLook {
            shade_trimmed: true,
            line_alpha: 1.0
        }
    );
    let ignored = cue_edge_look(true);
    assert!(!ignored.shade_trimmed, "the head and tail will play");
    assert_eq!(ignored.line_alpha, theme::CUE_EDGE_IGNORED_ALPHA);
    assert!(ignored.line_alpha > 0.0 && ignored.line_alpha < 1.0);
}

#[test]
fn a_waveform_with_ignored_marks_draws_without_panicking() {
    let track = media(0.5);
    let mut harness = egui_kittest::Harness::new_ui(move |ui| {
        let input = WaveInput {
            id: egui::Id::new(("wave", 2)),
            media: Some(&track),
            total: Some(1.0),
            markers: MarkerFractions {
                cue_in: Some(0.1),
                cue_out: Some(0.9),
                ignored: true,
                ..MarkerFractions::default()
            },
            colors: theme::wave_colors("sand"),
            mix_active: false,
            mix_label: "MIX",
            accessible_label: "Waveform",
            view: None,
            entry: None,
            shield: None,
            seekable: true,
        };
        waveform(ui, 40.0, &input);
    });
    harness.run();
}
```

- [ ] **Step 2: Run them to verify they fail**

Run each: `cargo test -p fp-app --test view the_view_says_when_the_cue_marks_are_ignored`, `cargo test -p fp-app --test waveform_view ignored_cue_marks_are_dimmed`, `cargo test -p fp-app --test waveform_view a_waveform_with_ignored_marks`.
Expected: FAIL to compile (`no field ignored`, `cue_edge_look` not found).

- [ ] **Step 3: Implement**

`crates/fp-app/src/ui/theme.rs` (next to the other waveform constants):

```rust
/// Opacity of the cue-in and cue-out lines on the waveform while players
/// ignore cue markers (feedback 2 spec O15).
pub const CUE_EDGE_IGNORED_ALPHA: f32 = 0.35;
```

`crates/fp-app/src/ui/view.rs`, `MarkerFractions` gets the last field, and `player_view` sets it:

```rust
    pub cue_out: Option<f32>,
    /// Players ignore cue-in and cue-out: the marks are drawn dimmed and
    /// the head and tail are not shaded.
    pub ignored: bool,
}
```
```rust
            cue_out: fraction(track.markers.cue_out.map(|m| m.secs), total),
            ignored: !state.config.players.use_cue_markers,
        };
```

`crates/fp-app/src/ui/widgets.rs`, near `TRIMMED_DIM`:

```rust
/// How the waveform draws the cue-in and cue-out edges.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CueEdgeLook {
    /// Shade the trimmed head and tail (playback skips them).
    pub shade_trimmed: bool,
    /// Opacity of the edge lines.
    pub line_alpha: f32,
}

/// Cue edges are plain while players use them; while ignored the whole file
/// plays, so nothing is shaded and the two lines are only dimmed marks.
pub fn cue_edge_look(ignored: bool) -> CueEdgeLook {
    if ignored {
        CueEdgeLook {
            shade_trimmed: false,
            line_alpha: theme::CUE_EDGE_IGNORED_ALPHA,
        }
    } else {
        CueEdgeLook {
            shade_trimmed: true,
            line_alpha: 1.0,
        }
    }
}
```

In `waveform`, wrap the existing trimmed loop:

```rust
    let look = cue_edge_look(m.ignored);
    if look.shade_trimmed {
        let secs = |f: Option<f32>| f.map(|f| f64::from(f) * total);
        for (region, edge) in view
            .trimmed(inner, secs(m.cue_in), secs(m.cue_out), total)
            .into_iter()
            .zip([true, false])
        {
            // ... the existing body, unchanged ...
        }
    } else {
        for f in [m.cue_in, m.cue_out].into_iter().flatten() {
            let x = x_of(f);
            if x >= inner.left() && x <= inner.right() {
                painter.rect_filled(
                    Rect::from_min_size(pos2(x, inner.top()), vec2(1.0, inner.height())),
                    0.0,
                    theme::NEUTRAL_500.gamma_multiply(look.line_alpha),
                );
            }
        }
    }
```
(Keep the comment about F20 above the `if`. Move the `secs` closure into the `if` as shown.)

Then confirm the waveform receives the flag: `grep -n "markers:" crates/fp-app/src/ui/player.rs` must show the `WaveInput` built with `pv.markers` (the whole struct). If a place copies fields one by one, add `ignored`.

- [ ] **Step 4: Run the tests to verify they pass**

Re-run the Step 2 commands: PASS. Then `cargo test -p fp-app --test waveform_view`, `cargo test -p fp-app --test view`, `cargo test -p fp-app --test waveform_ui`, `cargo test -p fp-app --test markers_ui`.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-app
git commit -m "feat(ui): dim the cue marks on the waveform while they are ignored"
```

---

### Task 5: The Settings toggle, locales and documentation

**Files:**
- Modify: `crates/fp-app/src/ui/settings.rs` (Players section, after the `auto_segue` row near line 1246)
- Modify: `crates/fp-app/locales/en-US/main.ftl`, `crates/fp-app/locales/es-ES/main.ftl`
- Test: `crates/fp-app/tests/settings.rs`
- Modify: `docs/user/settings.md`, `docs/user/markers-and-mixing.md`, `docs/technical/persistence.md`, `docs/technical/audio-engine.md`, `docs/technical/ui.md`, `docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md`

**Interfaces:**
- Consumes: `config.players.use_cue_markers`; the Settings helpers `row`, `toggle`, `update` used by the `auto_segue` row; the `t.tr` key lookup.
- Produces: messages `settings-use-cue-markers` and `settings-hint-use-cue-markers`; a checkbox labelled "Use cue-in and cue-out" in Settings > Players.

- [ ] **Step 1: Write the failing test**

Add to `crates/fp-app/tests/settings.rs`:

```rust
#[test]
fn the_cue_markers_toggle_updates_the_config() {
    let (mut h, fake) = harness(state(1, 1));
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Players").click();
    h.run_steps(2);
    assert!(fake.state.load().config.players.use_cue_markers);
    h.get_by_role_and_label(Role::CheckBox, "Use cue-in and cue-out")
        .click();
    h.run_steps(2);
    let sent: Vec<bool> = fake
        .take_sent()
        .into_iter()
        .filter_map(|c| match c {
            Command::UpdateConfig(config) => Some(config.players.use_cue_markers),
            _ => None,
        })
        .collect();
    assert_eq!(sent.last(), Some(&false));
    assert!(!fake.state.load().config.players.use_cue_markers);
    assert!(
        h.query_by_role_and_label(Role::CheckBox, "Automatic mix at the MIX point")
            .is_some(),
        "next to the automatic mix switch"
    );
}
```

- [ ] **Step 2: Run it to verify it fails**

Run: `cargo test -p fp-app --test settings the_cue_markers_toggle_updates_the_config`
Expected: FAIL (no such checkbox).

- [ ] **Step 3: Implement**

`crates/fp-app/locales/en-US/main.ftl` (after the `settings-auto-segue` line, and after `settings-hint-auto-segue`):

```
settings-use-cue-markers = Use cue-in and cue-out
```
```
settings-hint-use-cue-markers = Off: players play every track from the start to the end of the file; the markers are kept. Carts always use theirs
```

`crates/fp-app/locales/es-ES/main.ftl` (same positions):

```
settings-use-cue-markers = Usar cue-in y cue-out
```
```
settings-hint-use-cue-markers = Desactivado: los players reproducen cada pista de principio a fin del archivo; los marcadores se conservan. Los carts siempre usan los suyos
```

`crates/fp-app/src/ui/settings.rs`, right after the `auto_segue` row:

```rust
    let mut cue_markers = config.players.use_cue_markers;
    row(
        ui,
        &t.tr("settings-use-cue-markers"),
        Some(&t.tr("settings-hint-use-cue-markers")),
        |ui| {
            if toggle(ui, &mut cue_markers, &t.tr("settings-use-cue-markers")) {
                update(scene, |c| c.players.use_cue_markers = cue_markers);
            }
        },
    );
```

Docs, in this step:
- `docs/user/settings.md`: in the Players table add the row `| Use cue-in and cue-out | On | Off: players play every track from the start to the end of the file; the markers are kept and carts still use theirs |` next to "Automatic mix at the MIX point".
- `docs/user/markers-and-mixing.md`, section "What the player does with them": add a bullet "**Use cue-in and cue-out off** (Settings → Players): every player plays each track from 0 to the end of the file. Cue-in and cue-out, automatic and manual, are kept, and the waveform draws them as dim lines. The MIX point, intro and outro still work, within the whole file; **Automatic mix** is a separate switch. Carts always use their own cue-in and cue-out." Add also that a track still being analysed already plays this way.
- `docs/technical/persistence.md`: in the `players` table add `| `use_cue_markers` | true | Players honour cue-in and cue-out; off plays 0 to the end of the file. No range; a wrong type loads as the default |`.
- `docs/technical/audio-engine.md`: near the section on transition planning (`grep -n "cue-out" docs/technical/audio-engine.md`) add a paragraph: the effective range is `Track::play_range(use_markers)` in `fp-model`; `request_from_cue_in`, `request_at`, `plan_for`, the session restore and `set_marker` (MIX, intro, outro) use it; toggling re-preloads the next entry and re-schedules the plan but never moves a source already sounding; carts use `cue_in_secs`/`known_cue_out_secs` and are unaffected.
- `docs/technical/ui.md`: under "Marker editing" add a bullet: when `MarkerFractions::ignored` is set (`!players.use_cue_markers`), `widgets::cue_edge_look` turns off the head and tail shading and the cue-in/cue-out lines are drawn at `theme::CUE_EDGE_IGNORED_ALPHA`; Alt-drag editing is unchanged.
- Spec `docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md`: under "## 6. Plan 5 — Cue markers off", after the O15 bullets, add:

```
- **As built.**
  - `Track::play_range(use_markers) -> PlayRange` is the effective-range
    function. There was none before: each consumer read the cue methods
    directly. Now `request_from_cue_in`, `request_at`, `plan_for`, the session
    restore, `set_marker` (MIX, intro, outro), `player_view`,
    `playlist_times`, the remote player DTO and the remote seek bounds use it.
    Carts keep the marker-based methods.
  - Cue-in and cue-out edits are validated against the kept markers even
    while they are ignored; intro, outro and MIX against the effective range.
  - The player's CUE follows the range. Toggling the setting re-preloads and
    re-plans; it never restarts a track on air.
  - The waveform does not shade the head and tail while the marks are
    ignored; the lines are drawn at `CUE_EDGE_IGNORED_ALPHA` (0.35).
  - The messages are `settings-use-cue-markers` and
    `settings-hint-use-cue-markers`.
```
Also, if the spec's header has a "Status" line, append "plan 5 built" in the same style as earlier plans (see how plan 3 and 4 did it; do not touch the roadmap table).

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-app --test settings the_cue_markers_toggle_updates_the_config` (PASS), `cargo test -p fp-app --test i18n` (both locales have every key), `cargo test -p fp-app --test settings_restore`, `cargo test -p fp-app --test settings_layout`.
Then the full gate: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`.

- [ ] **Step 5: Commit**

```bash
git add crates/fp-app docs
git commit -m "feat(ui): add the Use cue-in and cue-out switch to Settings

Documents the setting in the user guide, the technical docs and the spec."
```

---

## Self-Review

- **Spec coverage:** the field, default, lenient load (Task 1); independent of `auto_segue` (Task 2 tests never touch it; `plan_for` keeps its own `auto_segue` branch); 0..end for every player entry, markers kept (Tasks 1-2); one rule in the effective-range function with one test per consumer: engine requests (`the_engine_starts...`, `restart...`, `the_player_cue...`, preload), transitions (`a_transition_is_planned...`), countdowns (Task 3), MIX validation (`a_mix_point_past...`, `the_mix_point_is_validated...`), session restore, remote; carts unchanged (`carts_keep_their_markers...`); dimmed marks (Task 4); Settings toggle next to the automatic mix switch, both locales (Task 5); docs and "As built" (Task 5).
- **Placeholders:** none; the only "unchanged body" mention (Task 4, the trimmed-loop body) refers to code that stays as it is in the file.
- **Types:** `PlayRange { cue_in, cue_out }` + `known_end()` + `length()`, `Track::play_range(bool)`, `MarkerFractions::ignored`, `CueEdgeLook`, `cue_edge_look(bool)`, `theme::CUE_EDGE_IGNORED_ALPHA`, `settings-use-cue-markers` are used with the same names in every task.
- **Review Focus:** items 1-5 each have a named test in the task that owns the code.
