# Settings Window (Feedback 2, Plan 2) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give the Settings window one fixed size and a shared row grid (O3), a "Restore defaults" button per section (O2), a "Restart now" action for settings that apply only at start-up (O4), and hide the Null backend from the audio system list (O5).

**Architecture:**
- **Rules in the model.** `fp_model::restore_defaults(&mut Config, SettingsSection)` resets one section, and `Command::RestoreDefaults` applies it through the reducer, so it is validated and keeps the player count. `fp_model::restart_pending(started, current) -> Vec<RestartReason>` compares the configuration the engine was built with against the model's current one.
- **The window.** `settings::window_size` gives one size for every section. The header, the section header and the footer have fixed sizes, and the body is a `ScrollArea::both`, so wide content scrolls instead of widening the window. Every row goes through one helper, `labelled_row`, with a fixed label column whose controls fill the rest.
- **Restart.** `AppUi` keeps the started configuration. While `restart_pending` is not empty, the Settings footer shows a notice and **Restart now**, and the top bar shows a "Restart pending" pill that runs the same action. When something is on air, both ask plan 1's on-air guard, with a new `ExitIntent::Restart`; when nothing is, they restart at once. Once confirmed (or straight away), the UI sets a shared restart flag and closes the window. `main` runs the normal shutdown, which saves the session, then drops the instance lock and starts the application again through `fp_app::restart::relaunch`. A start never plays anything, so rule 10 holds.

**Tech Stack:** Rust, egui/eframe 0.36.2, egui_kittest 0.36.2 (kittest 0.4), Fluent, std::process.

**Spec:** `docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md` §3 (and §12). Roadmap: `docs/superpowers/plans/2026-10-01-operator-feedback-2-roadmap.md`.

## Rulings made while planning

The spec leaves these points open, or the code answers them. Copy them into the ledger. Task 10 records the final set in the spec.

- **Restart-only fields.**
  - Ruling: the restart set is `outputs.backend`, `outputs.sample_rate`, `outputs.buffer_frames`, the player and cartwall routes, `outputs.bit_perfect`, `limits` and `tuning`. The player count is **not** in it — `SetPlayerCount` emits `AddPlayer`/`RemovePlayer` and the engine applies them at once. The spec's list named it. Every other section (players, meter, analysis, ui, cartwall, shortcuts, midi, remote) is read from the model snapshot while running. That was verified in `services.rs` (`follow_settings`), `fp-remote/src/server.rs`, `fp-control/src/service.rs` (`MidiCore::step`) and `conductor.rs` (meter).
  - Why: `EngineSettings::from_config` is built once in `main.rs`. The analysis cache, the store, the crash hook and the conductor tick take limits and tuning at start.
  - Cost if wrong: a stale notice, or a missing one.
- **How pending is detected.**
  - Ruling: compare the configuration given to the engine at start (`AppUi::with_started_config`) with `state.config` from the model snapshot. The model's config is what the store saves.
  - Why: the config file is never re-read while running, so the snapshot is the saved state.
  - Cost if wrong: none found.
- **Relaunch arguments.**
  - Ruling: relaunch with **no arguments**, in the same environment.
  - Why: the only run arguments are playlist paths. They were imported and saved at start, so passing them again would import them a second time. The spec says "the same arguments".
  - Cost if wrong: an operator who relied on a start argument loses it. None exists today.
- **Relaunch target.**
  - Ruling: use `$APPIMAGE` inside an AppImage, because the mounted image goes away with the old process. Inside a Flatpak (`FLATPAK_ID`), use `flatpak-spawn <exe>` through the Flatpak portal, which is reachable without extra permissions, because the old sandbox dies with its process. In every other case, use `current_exe()`, with Linux's `" (deleted)"` suffix stripped after a package upgrade.
  - Cost if wrong: on that packaging the restart fails. The app logs it and shows a message, and the operator starts it by hand.
- **Restore defaults in Players** (maintainer's decision).
  - Ruling: reset `players.*` except `count`. The interface language, although shown in that section, is kept.
  - Why: the count adds or removes players, which changes the show's layout, not a preference; the language is the operator's, not a tuning value.
- **No Restore defaults in Cartwall** (maintainer's decision).
  - Ruling: Cartwall gets no button and no new row. The section edits cart pages, which are show data, and `config.cartwall` (the grid of new pages) has no field in Settings. This departs from spec §3 O2, which listed Cartwall; Task 10 updates the spec.
- **Shortcuts.**
  - Ruling: the header button replaces the section's old "Reset to defaults" button (`shortcut-reset`). `Command::ResetShortcuts` stays: a model test uses it.
- **The pill** (maintainer's decision).
  - Ruling: clicking "Restart pending" runs the same action as **Restart now**. When nothing is on air, it restarts at once, with no confirmation. When something is on air, the on-air guard asks first. Its tooltip lists the reasons.

## Global Constraints

- All code, comments and docs in English. UI strings go in both `crates/fp-app/locales/en-US/main.ftl` and `es-ES/main.ftl`.
- Never name other playout or radio-automation products.
- Operator values are `Config` fields with defaults, ranges in `Config::validate` and lenient loading.
- `unsafe_code` is forbidden. Outside tests, no `unwrap`, `expect`, `panic` or indexing: use `get`.
- Behaviour lives in `fp-model`; the UI only displays and sends commands. The UI never blocks: the relaunch runs in `main` after the event loop has ended, never on the UI thread.
- Nothing goes on air by itself after a start, restart or crash.
- TDD: every new test is seen failing before the code that makes it pass.
- Before each commit: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace` all pass.
- Conventional Commits; every commit ends with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Ledger: `.superpowers/sdd/feedback2-plan2/progress.md`; record deviations as `Ruling: <decision> — <why> — <cost if wrong>`.
- Branch: `feat/settings-window`, from an up-to-date `master`.

## Review Focus

- **A restart-only value changed and then changed back** (48 kHz → 44.1 kHz → 48 kHz). Expected: no notice, no pill. Pinned in Task 2 (`changing_back_clears_the_reason`) and Task 8 (`changing_back_hides_the_pill`).
- **Restore defaults in Players after the player count was changed** (2 players, default 4). Expected: the count and the players stay; only the other Players values go back. Pinned in Task 1 (`the_command_keeps_the_player_count`).
- **A small main window** (700 × 500). Expected: the Settings window shrinks to fit and keeps one size across sections. Pinned in Task 3 (`a_small_screen_keeps_one_size_inside_it` and the `window_size` unit test).
- **Restart now while a Remote text field still holds an unsaved draft.** Expected: the draft is applied before the window closes, as when switching sections. Pinned in Task 8 (`restart_now_applies_a_remote_draft_first`).
- **The executable was replaced by a package upgrade while running** (Linux reports `/usr/bin/fauste-player (deleted)`). Expected: the new binary at `/usr/bin/fauste-player` starts. Pinned in Task 9 (`a_replaced_executable_starts_from_its_path`).

---

### Task 1: `restore_defaults` and `Command::RestoreDefaults` in the model

**Files:**
- Create: `crates/fp-model/src/restore.rs`
- Modify: `crates/fp-model/src/lib.rs` (add `pub mod restore;` and `pub use restore::{SettingsSection, restore_defaults};`)
- Modify: `crates/fp-model/src/command.rs` (new variant after `ResetShortcuts`)
- Modify: `crates/fp-model/src/reducer.rs` (new arm next to `Command::ResetShortcuts`)
- Test: `crates/fp-model/tests/restore.rs`

**Interfaces:**
- Produces:
  - `pub enum SettingsSection { Players, Meters, Analysis, Shortcuts }`, deriving `Debug, Clone, Copy, PartialEq, Eq, Hash`. Only the sections that have the button exist.
  - `pub fn restore_defaults(config: &mut Config, section: SettingsSection)`.
  - `Command::RestoreDefaults(SettingsSection)`: restores, validates and keeps `players.count` (through `update_config`).

- [ ] **Step 1: Write the failing tests**

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback 2 spec O2: "Restore defaults" resets one Settings section.

mod common;

use common::fixture;
use fp_model::{
    AnalysisSettings, Command, Config, MeterBallistics, MeterConfig,
    PlayersConfig, SettingsSection, apply, default_shortcuts, restore_defaults,
};

/// A configuration with a value changed in every part.
fn altered() -> Config {
    let mut c = Config::default();
    c.players.count = 6;
    c.players.fade_ms = 3000;
    c.players.auto_segue = false;
    c.players.history_len = 10;
    c.ui.language = Some("es-ES".into());
    c.ui.wave_color = "sand".into();
    c.ui.follow_current_grace_secs = 30.0;
    c.meter.ballistics = MeterBallistics::Vu;
    c.meter.floor_db = -40.0;
    c.analysis.segue_drop_db = 20.0;
    c.analysis.cover_thumb_px = 256;
    c.cartwall.default_rows = 4;
    c.cartwall.default_cols = 4;
    c.shortcuts.clear();
    c.outputs.backend = Some("alsa".into());
    c.outputs.sample_rate = 44_100;
    c.limits.max_players = 8;
    c.tuning.prebuffer_secs = 10.0;
    c.midi.enabled = true;
    c.remote.http.enabled = true;
    c
}

#[test]
fn players_resets_the_player_settings_but_not_the_count_or_the_language() {
    let mut c = altered();
    restore_defaults(&mut c, SettingsSection::Players);
    let mut expected = altered();
    expected.players = PlayersConfig {
        count: 6,
        ..PlayersConfig::default()
    };
    assert_eq!(c, expected);
}

#[test]
fn meters_resets_only_the_meter() {
    let mut c = altered();
    restore_defaults(&mut c, SettingsSection::Meters);
    let mut expected = altered();
    expected.meter = MeterConfig::default();
    assert_eq!(c, expected);
}

#[test]
fn analysis_resets_only_the_analysis() {
    let mut c = altered();
    restore_defaults(&mut c, SettingsSection::Analysis);
    let mut expected = altered();
    expected.analysis = AnalysisSettings::default();
    assert_eq!(c, expected);
}

#[test]
fn shortcuts_resets_only_the_shortcuts() {
    let mut c = altered();
    restore_defaults(&mut c, SettingsSection::Shortcuts);
    let mut expected = altered();
    expected.shortcuts = default_shortcuts();
    assert_eq!(c, expected);
}

#[test]
fn the_command_keeps_the_player_count() {
    let mut s = fixture(1);
    apply(&mut s, Command::SetPlayerCount(2)).unwrap();
    let mut config = s.config.clone();
    config.players.fade_ms = 3000;
    config.ui.language = Some("es-ES".into());
    apply(&mut s, Command::UpdateConfig(Box::new(config))).unwrap();

    apply(&mut s, Command::RestoreDefaults(SettingsSection::Players)).unwrap();
    assert_eq!(s.config.players.fade_ms, 1000);
    assert_eq!(s.config.ui.language.as_deref(), Some("es-ES"));
    assert_eq!(s.players.len(), 2);
    assert_eq!(s.config.players.count, 2);
}
```

- [ ] **Step 2: Run the tests and check that they fail**

Run: `cargo test -p fp-model --test restore`
Expected: compile error, because `SettingsSection`, `restore_defaults` and `Command::RestoreDefaults` are unresolved.

- [ ] **Step 3: Implement**

`crates/fp-model/src/restore.rs`:

```rust
//! "Restore defaults" in Settings (feedback 2 spec O2): one section's
//! fields go back to `Config::default()`, and nothing else changes.

use crate::config::{Config, PlayersConfig};

/// The Settings sections that offer "Restore defaults". Outputs and MIDI
/// depend on the hardware, Remote holds security settings, and Playlists
/// and Cartwall hold show data: they have no button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettingsSection {
    Players,
    Meters,
    Analysis,
    Shortcuts,
}

/// Resets the fields `section` shows. In Players, the player count is
/// kept (it adds or removes players, which is the show's layout, not a
/// preference) and so is the interface language (the operator's own).
pub fn restore_defaults(config: &mut Config, section: SettingsSection) {
    let defaults = Config::default();
    match section {
        SettingsSection::Players => {
            config.players = PlayersConfig {
                count: config.players.count,
                ..defaults.players
            };
        }
        SettingsSection::Meters => config.meter = defaults.meter,
        SettingsSection::Analysis => config.analysis = defaults.analysis,
        SettingsSection::Shortcuts => config.shortcuts = defaults.shortcuts,
    }
}
```

In `command.rs`, after `ResetShortcuts,`:

```rust
    /// Resets one Settings section to its defaults (feedback 2 spec O2).
    RestoreDefaults(crate::restore::SettingsSection),
```

In `reducer.rs`, next to the `Command::ResetShortcuts` arm:

```rust
        Command::RestoreDefaults(section) => {
            let mut config = state.config.clone();
            crate::restore::restore_defaults(&mut config, section);
            update_config(state, config, &mut out)?;
        }
```

`update_config` already validates, keeps `players.count` and refuses when a busy player would be removed. If `Command` is matched exhaustively anywhere else (`grep -rn "Command::ResetShortcuts" crates`), add the new variant there in the same way.

- [ ] **Step 4: Run the tests and check that they pass**

Run: `cargo test -p fp-model --test restore && cargo test -p fp-model`
Expected: 5 passed, and the rest of the model suite stays green.

- [ ] **Step 5: Commit**

```bash
git add crates/fp-model/src/restore.rs crates/fp-model/src/lib.rs crates/fp-model/src/command.rs crates/fp-model/src/reducer.rs crates/fp-model/tests/restore.rs
git commit -m "feat(model): restore one settings section to its defaults

Operators who tried values in a Settings section had no way back to the
defaults short of editing the config file.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: `restart_pending` in the model

**Files:**
- Create: `crates/fp-model/src/restart.rs`
- Modify: `crates/fp-model/src/lib.rs` (add `pub mod restart;` and `pub use restart::{RestartReason, restart_pending};`)
- Test: `crates/fp-model/tests/restart.rs`

**Interfaces:**
- Produces:
  - `pub enum RestartReason { AudioSystem, SampleRate, BufferSize, Routes, BitPerfect, Limits, Tuning }`, deriving `Debug, Clone, Copy, PartialEq, Eq, Hash`, in that order;
  - `pub fn restart_pending(started: &Config, current: &Config) -> Vec<RestartReason>`, in enum order, each reason at most once.
- Route lists are compared by meaning:
  - per player, ignoring order and entries with neither Main nor Cue;
  - `bit_perfect` compared as a set.

- [ ] **Step 1: Write the failing tests**

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback 2 spec O4: which changed settings wait for a restart.

use fp_model::{
    Config, OutputDevice, PlayerId, PlayerRoutes, RestartReason, Route, restart_pending,
};

fn route(device: &str) -> Route {
    Route {
        backend: "alsa".into(),
        device: device.into(),
        first_channel: 0,
    }
}

#[test]
fn the_same_configuration_needs_no_restart() {
    let c = Config::default();
    assert!(restart_pending(&c, &c.clone()).is_empty());
}

#[test]
fn each_start_up_setting_gives_its_reason() {
    type Edit = fn(&mut Config);
    let cases: [(Edit, RestartReason); 8] = [
        (|c| c.outputs.backend = Some("jack".into()), RestartReason::AudioSystem),
        (|c| c.outputs.sample_rate = 44_100, RestartReason::SampleRate),
        (|c| c.outputs.buffer_frames = 1024, RestartReason::BufferSize),
        (
            |c| {
                c.outputs.routes.push(PlayerRoutes {
                    player: PlayerId(1),
                    main: Some(route("dac")),
                    cue: None,
                })
            },
            RestartReason::Routes,
        ),
        (|c| c.outputs.cartwall.cue = Some(route("phones")), RestartReason::Routes),
        (
            |c| {
                c.outputs.bit_perfect.push(OutputDevice {
                    backend: "alsa".into(),
                    device: "dac".into(),
                })
            },
            RestartReason::BitPerfect,
        ),
        (|c| c.limits.max_players = 8, RestartReason::Limits),
        (|c| c.tuning.prebuffer_secs = 10.0, RestartReason::Tuning),
    ];
    for (edit, reason) in cases {
        let started = Config::default();
        let mut current = started.clone();
        edit(&mut current);
        assert_eq!(restart_pending(&started, &current), vec![reason]);
    }
}

#[test]
fn settings_applied_while_running_need_no_restart() {
    let started = Config::default();
    let mut c = started.clone();
    c.players.count = 8;
    c.players.fade_ms = 3000;
    c.meter.floor_db = -40.0;
    c.analysis.segue_drop_db = 20.0;
    c.ui.language = Some("es-ES".into());
    c.cartwall.default_rows = 4;
    c.shortcuts.clear();
    c.midi.enabled = true;
    c.remote.http.enabled = true;
    assert!(restart_pending(&started, &c).is_empty());
}

#[test]
fn changing_back_clears_the_reason() {
    let started = Config::default();
    let mut c = started.clone();
    c.outputs.sample_rate = 44_100;
    assert_eq!(restart_pending(&started, &c), vec![RestartReason::SampleRate]);
    c.outputs.sample_rate = started.outputs.sample_rate;
    assert!(restart_pending(&started, &c).is_empty());
}

#[test]
fn empty_and_reordered_routes_are_no_change() {
    let mut started = Config::default();
    started.outputs.routes = vec![
        PlayerRoutes {
            player: PlayerId(1),
            main: Some(route("dac")),
            cue: None,
        },
        PlayerRoutes {
            player: PlayerId(2),
            main: None,
            cue: Some(route("phones")),
        },
    ];
    started.outputs.bit_perfect = vec![
        OutputDevice {
            backend: "alsa".into(),
            device: "dac".into(),
        },
        OutputDevice {
            backend: "alsa".into(),
            device: "phones".into(),
        },
    ];
    let mut c = started.clone();
    c.outputs.routes.reverse();
    c.outputs.routes.push(PlayerRoutes {
        player: PlayerId(3),
        main: None,
        cue: None,
    });
    c.outputs.bit_perfect.reverse();
    assert!(restart_pending(&started, &c).is_empty());
}

#[test]
fn several_changes_are_listed_once_each_in_order() {
    let started = Config::default();
    let mut c = started.clone();
    c.tuning.prebuffer_secs = 10.0;
    c.outputs.sample_rate = 44_100;
    c.outputs.cartwall.main = Some(route("dac"));
    c.outputs.cartwall.cue = Some(route("phones"));
    assert_eq!(
        restart_pending(&started, &c),
        vec![
            RestartReason::SampleRate,
            RestartReason::Routes,
            RestartReason::Tuning
        ]
    );
}
```

- [ ] **Step 2: Run the tests and check that they fail**

Run: `cargo test -p fp-model --test restart`
Expected: compile error, because `RestartReason` and `restart_pending` are unresolved.

- [ ] **Step 3: Implement**

```rust
//! Settings that apply only when the application starts (feedback 2 spec
//! O4). The audio engine, its devices, the resource limits and the engine
//! tuning are built once at start-up; every other setting is read while
//! running.

use std::collections::{BTreeMap, HashSet};

use crate::config::{Config, OutputsConfig, Route};
use crate::ids::PlayerId;

/// Why a restart is needed, in the order the notice lists them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RestartReason {
    AudioSystem,
    SampleRate,
    BufferSize,
    /// A player's or the cartwall's Main or Cue output.
    Routes,
    BitPerfect,
    Limits,
    Tuning,
}

type Pair<'a> = (Option<&'a Route>, Option<&'a Route>);

/// Each player's routes by player, without empty entries.
fn player_routes(outputs: &OutputsConfig) -> BTreeMap<PlayerId, Pair<'_>> {
    outputs
        .routes
        .iter()
        .filter(|r| r.main.is_some() || r.cue.is_some())
        .map(|r| (r.player, (r.main.as_ref(), r.cue.as_ref())))
        .collect()
}

/// What changed between the configuration the application started with
/// and the current one that only a restart applies.
pub fn restart_pending(started: &Config, current: &Config) -> Vec<RestartReason> {
    let (a, b) = (&started.outputs, &current.outputs);
    let mut reasons = Vec::new();
    if a.backend != b.backend {
        reasons.push(RestartReason::AudioSystem);
    }
    if a.sample_rate != b.sample_rate {
        reasons.push(RestartReason::SampleRate);
    }
    if a.buffer_frames != b.buffer_frames {
        reasons.push(RestartReason::BufferSize);
    }
    if player_routes(a) != player_routes(b) || a.cartwall != b.cartwall {
        reasons.push(RestartReason::Routes);
    }
    let set = |o: &OutputsConfig| o.bit_perfect.iter().cloned().collect::<HashSet<_>>();
    if set(a) != set(b) {
        reasons.push(RestartReason::BitPerfect);
    }
    if started.limits != current.limits {
        reasons.push(RestartReason::Limits);
    }
    if started.tuning != current.tuning {
        reasons.push(RestartReason::Tuning);
    }
    reasons
}
```

- [ ] **Step 4: Run the tests and check that they pass**

Run: `cargo test -p fp-model --test restart`
Expected: 6 passed.

- [ ] **Step 5: Commit**

```bash
git add crates/fp-model/src/restart.rs crates/fp-model/src/lib.rs crates/fp-model/tests/restart.rs
git commit -m "feat(model): list the settings that wait for a restart

Outputs, limits and tuning are read once at start-up; the interface
needs to know when a change to them is not in effect yet.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: One window size for every section

**Files:**
- Modify: `crates/fp-app/src/ui/settings.rs`:
  - the size constants and `window_size`;
  - `show`: the size, and `ScrollArea::vertical` becomes `ScrollArea::both`;
  - a `#[cfg(test)] mod tests` at the end.
- Test: `crates/fp-app/tests/settings_layout.rs` (new)

**Interfaces:**
- Produces:
  - `const WINDOW_SIZE: egui::Vec2 = vec2(900.0, 640.0)`;
  - `const SCREEN_MARGIN: egui::Vec2 = vec2(48.0, 82.0)`;
  - `const MIN_WINDOW_SIZE: egui::Vec2 = vec2(320.0, 300.0)`;
  - `pub(crate) fn window_size(screen: egui::Vec2) -> egui::Vec2`;
  - the shared test helpers in `tests/settings_layout.rs`: `SCREEN`, `SECTIONS`, `open_settings` and `wait_for_devices`. Tasks 4 and 5 add tests to this file.

**Why the size changes today:**
- The body is a `ScrollArea::vertical`, and egui grows a scroll area along an axis that does not scroll.
- Outputs with a long device name therefore widens the modal, and the footer stops spanning it.
- The size is also `(screen − margin).clamp(320, 980)`, not one constant.

- [ ] **Step 1: Write the failing tests**

`crates/fp-app/tests/settings_layout.rs`:

```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Feedback 2 spec O3: the Settings window's size and grid.

mod support;

use std::sync::Arc;

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::AppUi;
use fp_backends::{AudioBackend, OfflineBackend};
use fp_model::{AppState, PlayerRoutes, Route};
use support::{harness_sized, state};

const SCREEN: egui::Vec2 = egui::vec2(1400.0, 900.0);
/// The window size the spec asks for (`settings::WINDOW_SIZE`).
const WINDOW: egui::Vec2 = egui::vec2(900.0, 640.0);
const SECTIONS: [&str; 9] = [
    "Audio outputs",
    "Players",
    "Meters",
    "Analysis",
    "Playlists",
    "Cartwall",
    "Keyboard shortcuts",
    "MIDI",
    "Remote",
];
const LONG_DEVICE: &str = "USB Audio Interface With A Remarkably Long Product Name";

/// Two players; player 1 plays on a device with a long name.
fn routed() -> AppState {
    let mut s = state(2, 1);
    let route = |device: &str| Route {
        backend: "offline".into(),
        device: device.into(),
        first_channel: 0,
    };
    s.config.outputs.backend = Some("offline".into());
    s.config.outputs.routes = vec![PlayerRoutes {
        player: s.players[0].id,
        main: Some(route(LONG_DEVICE)),
        cue: Some(route("speakers")),
    }];
    s
}

fn backends() -> Vec<Arc<dyn AudioBackend>> {
    let backend = OfflineBackend::new();
    backend.add_device(LONG_DEVICE, 2);
    backend.add_device("speakers", 2);
    vec![Arc::new(backend)]
}

fn open_settings(size: egui::Vec2, s: AppState) -> Harness<'static, AppUi> {
    let backends = backends();
    let (mut h, _) = harness_sized(s, size, move |ui| ui.with_backends(backends));
    h.get_by_label("Settings").click();
    h.run_steps(3);
    h
}

fn open(h: &mut Harness<'static, AppUi>, section: &str) {
    h.get_by_role_and_label(Role::Button, section).click();
    h.run_steps(3);
}

/// Devices are listed by a helper thread.
fn wait_for_devices(h: &mut Harness<'static, AppUi>) {
    for _ in 0..200 {
        h.run_steps(1);
        if h.query_all_by_label("Test Main").next().is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    h.run_steps(3);
}

fn close_rect(h: &Harness<'static, AppUi>) -> egui::Rect {
    h.get_by_label("Close").rect()
}

#[test]
fn every_section_has_the_same_window() {
    let mut h = open_settings(SCREEN, routed());
    let mut seen = Vec::new();
    for section in SECTIONS {
        open(&mut h, section);
        if section == "Audio outputs" {
            wait_for_devices(&mut h);
        }
        seen.push((section, close_rect(&h)));
    }
    // The window is centred: Close sits 16 px inside its right edge.
    let right = (SCREEN.x + WINDOW.x) / 2.0 - 16.0;
    for (section, rect) in seen {
        assert!(
            (rect.right() - right).abs() < 1.5,
            "{section}: Close ends at {} instead of {right}",
            rect.right()
        );
        let bottom = (SCREEN.y + WINDOW.y) / 2.0;
        assert!(
            rect.bottom() < bottom && rect.bottom() > bottom - 52.0,
            "{section}: Close is not in the footer"
        );
    }
}

#[test]
fn the_footer_spans_the_window() {
    let mut h = open_settings(SCREEN, routed());
    open(&mut h, "Audio outputs");
    wait_for_devices(&mut h);
    let left = (SCREEN.x - WINDOW.x) / 2.0 + 16.0;
    let notice = h.get_by_label("Changes apply at once").rect();
    assert!((notice.left() - left).abs() < 1.5, "notice at {}", notice.left());
    let right = (SCREEN.x + WINDOW.x) / 2.0 - 16.0;
    assert!((close_rect(&h).right() - right).abs() < 1.5);
}

#[test]
fn a_small_screen_keeps_one_size_inside_it() {
    let small = egui::vec2(700.0, 500.0);
    let mut h = open_settings(small, routed());
    let mut rights = Vec::new();
    for section in SECTIONS {
        open(&mut h, section);
        if section == "Audio outputs" {
            wait_for_devices(&mut h);
        }
        rights.push(close_rect(&h).right());
    }
    assert!(rights.iter().all(|r| (r - rights[0]).abs() < 0.5), "{rights:?}");
    assert!(rights[0] <= small.x - 16.0);
}
```

At the end of `settings.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_window_has_one_size_clamped_to_the_screen() {
        assert_eq!(window_size(vec2(1600.0, 940.0)), WINDOW_SIZE);
        assert_eq!(window_size(vec2(700.0, 500.0)), vec2(652.0, 418.0));
        assert_eq!(window_size(vec2(100.0, 100.0)), MIN_WINDOW_SIZE);
    }
}
```

- [ ] **Step 2: Run the tests and check that they fail**

Run: `cargo test -p fp-app --test settings_layout && cargo test -p fp-app --lib settings`
Expected:
- the integration tests fail, because the window is 980 px wide at 1400 × 900, or grows in Audio outputs;
- the unit test fails to compile, because `window_size` is missing.

- [ ] **Step 3: Implement**

After the `BUFFER_SIZES` constant:

```rust
/// The Settings window has one size whatever the section (feedback 2 spec
/// O3); the section body scrolls inside it.
const WINDOW_SIZE: egui::Vec2 = vec2(900.0, 640.0);
/// Room kept around the window inside the main window.
const SCREEN_MARGIN: egui::Vec2 = vec2(48.0, 82.0);
const MIN_WINDOW_SIZE: egui::Vec2 = vec2(320.0, 300.0);

/// `WINDOW_SIZE`, shrunk to fit `screen`.
pub(crate) fn window_size(screen: egui::Vec2) -> egui::Vec2 {
    (screen - SCREEN_MARGIN).min(WINDOW_SIZE).max(MIN_WINDOW_SIZE)
}
```

In `show`, replace the `let screen = …; let size = vec2(…);` lines with:

```rust
    let size = window_size(ctx.content_rect().size());
```

In the body column, replace `egui::ScrollArea::vertical()` with `egui::ScrollArea::both()`, and keep `.auto_shrink([false, false])`. Wide content now scrolls instead of widening the window.

- [ ] **Step 4: Run the tests and check that they pass**

Run: `cargo test -p fp-app --test settings_layout && cargo test -p fp-app --lib settings && cargo test -p fp-app --test settings`
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add crates/fp-app/src/ui/settings.rs crates/fp-app/tests/settings_layout.rs
git commit -m "fix(ui): one size for the settings window

The window followed its widest section: Audio outputs with a long device
name made it grow and the footer stopped spanning it.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: The shared row grid, and Outputs in columns

**Files:**
- Modify: `crates/fp-app/src/ui/settings.rs`:
  - `row` becomes a wrapper of a new `labelled_row`;
  - `route_picker` is laid out in columns.
- Test: `crates/fp-app/tests/settings_layout.rs` (add)

**Interfaces:**
- Consumes: the helpers and constants of `tests/settings_layout.rs` (Task 3).
- Produces, in `settings.rs`:
  - `pub(super) const LABEL_WIDTH: f32 = 180.0`;
  - `pub(super) fn labelled_row<R>(ui: &mut Ui, label: &str, hint: Option<&str>, control: impl FnOnce(&mut Ui, egui::Id) -> R) -> R`, where the `Id` is the label's, for `labelled_by`;
  - `pub(super) fn row(ui: &mut Ui, label: &str, hint: Option<&str>, control: impl FnOnce(&mut Ui))`, kept with the same signature, so callers do not change;
  - `const CHANNELS_WIDTH: f32 = 96.0`.

**Why rows do not line up today:** `allocate_ui_with_layout(vec2(220, 40), …)`
advances by the child's *used* rect. A short label ("Trim margin") gives a
narrower column than a long one, so the controls start at different x
positions. Fixing the column with `set_width` lines them up.

- [ ] **Step 1: Write the failing tests** (append to `tests/settings_layout.rs`)

```rust
fn left_of(h: &Harness<'static, AppUi>, role: Role, label: &str) -> f32 {
    h.get_by_role_and_label(role, label).rect().left()
}

#[test]
fn rows_share_one_label_column() {
    let mut h = open_settings(SCREEN, state(1, 1));
    open(&mut h, "Analysis");
    let short = left_of(&h, Role::Slider, "Trim margin");
    let long = left_of(&h, Role::Slider, "Minimum length for mix and outro markers");
    assert!((short - long).abs() < 0.5, "{short} vs {long}");
}

#[test]
fn outputs_test_buttons_line_up_in_columns() {
    let mut h = open_settings(SCREEN, routed());
    open(&mut h, "Audio outputs");
    wait_for_devices(&mut h);
    for label in ["Test Main", "Test Cue"] {
        let lefts: Vec<f32> = h
            .get_all_by_label(label)
            .map(|n| n.rect().left())
            .collect();
        // Player 1 (routed), player 2 (not routed) and the cartwall.
        assert_eq!(lefts.len(), 3, "{label}");
        assert!(
            lefts.iter().all(|x| (x - lefts[0]).abs() < 0.5),
            "{label}: {lefts:?}"
        );
    }
    let main = h.get_all_by_label("Test Main").next().unwrap().rect();
    let cue = h.get_all_by_label("Test Cue").next().unwrap().rect();
    assert!((main.left() - cue.left()).abs() < 0.5);
    assert!((main.right() - cue.right()).abs() < 0.5);
}
```

- [ ] **Step 2: Run the tests and check that they fail**

Run: `cargo test -p fp-app --test settings_layout rows_share outputs_test`
Expected: both fail, on the x positions.

- [ ] **Step 3: Implement**

Replace `fn row` in `settings.rs` with:

```rust
/// The label column of every Settings row (feedback 2 spec O3).
pub(super) const LABEL_WIDTH: f32 = 180.0;

/// A settings row: the label (and hint) in a fixed column on the left, the
/// control filling the rest. `control` gets the label's id, for
/// `labelled_by`.
pub(super) fn labelled_row<R>(
    ui: &mut Ui,
    label: &str,
    hint: Option<&str>,
    control: impl FnOnce(&mut Ui, egui::Id) -> R,
) -> R {
    let out = ui
        .horizontal(|ui| {
            ui.set_min_height(44.0);
            let label_id = ui
                .allocate_ui_with_layout(
                    vec2(LABEL_WIDTH, 40.0),
                    Layout::top_down(Align::Min),
                    |ui| {
                        // The column keeps its width whatever the label's.
                        ui.set_width(LABEL_WIDTH);
                        ui.spacing_mut().item_spacing = vec2(0.0, 2.0);
                        ui.add_space(4.0);
                        let id = ui
                            .add(
                                egui::Label::new(
                                    RichText::new(label).font(font(13.0)).color(theme::TEXT),
                                )
                                .selectable(false),
                            )
                            .id;
                        if let Some(hint) = hint {
                            ui.add(
                                egui::Label::new(
                                    RichText::new(hint)
                                        .font(font(11.0))
                                        .color(theme::NEUTRAL_500),
                                )
                                .selectable(false)
                                .wrap(),
                            );
                        }
                        id
                    },
                )
                .inner;
            ui.add_space(16.0);
            let rest = ui.available_width();
            ui.allocate_ui_with_layout(
                vec2(rest, 40.0),
                Layout::left_to_right(Align::Center),
                |ui| {
                    ui.set_width(rest);
                    control(ui, label_id)
                },
            )
            .inner
        })
        .inner;
    let width = ui.available_width();
    let (r, _) = ui.allocate_exact_size(vec2(width, 1.0), Sense::hover());
    ui.painter()
        .rect_filled(r, 0.0, theme::TEXT.gamma_multiply(0.06));
    out
}

/// `labelled_row` for controls that name themselves.
pub(super) fn row(ui: &mut Ui, label: &str, hint: Option<&str>, control: impl FnOnce(&mut Ui)) {
    labelled_row(ui, label, hint, |ui, _| control(ui));
}
```

Rewrite `route_picker`'s body as below. The bus tag stays on the left, and a
right-to-left block places the test button at the right edge, then the
channel-pair slot (kept empty without a route), and lets the device box fill
what is left. The closures that build `shown`, `none_text`, `labels`, `pair`
and `target` stay as they are; only their placement changes.

```rust
/// Width of the channel-pair slot, kept empty when a bus has no device, so
/// that the test buttons line up (feedback 2 spec O3).
const CHANNELS_WIDTH: f32 = 96.0;

fn route_picker(
    ui: &mut Ui,
    scene: &Scene<'_>,
    owner: Owner,
    bus: Bus,
    route: Option<Route>,
    backend: Option<&str>,
    devices: &[DeviceInfo],
) {
    let t = scene.i18n;
    // Both test buttons get the wider label's width: one column each.
    let test_width = [t.tr("settings-test-main"), t.tr("settings-test-cue")]
        .into_iter()
        .map(|l| {
            ui.painter()
                .layout_no_wrap(l, font(12.0), theme::TEXT)
                .size()
                .x
        })
        .fold(0.0, f32::max)
        + 36.0;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(8.0, 0.0);
        let bus_label = match bus {
            Bus::Main => t.tr("settings-main"),
            Bus::Cue => t.tr("settings-cue"),
        };
        ui.add_sized(
            vec2(40.0, 20.0),
            egui::Label::new(
                RichText::new(bus_label.to_uppercase())
                    .font(widgets::font_semibold(10.0))
                    .color(if bus == Bus::Main {
                        theme::ACCENT_400
                    } else {
                        theme::CUE
                    }),
            )
            .selectable(false),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            // Right to left: the test button, the channel pair, the device.
            let (label, freq) = match bus {
                Bus::Main => (t.tr("settings-test-main"), MAIN_TONE_HZ),
                Bus::Cue => (t.tr("settings-test-cue"), CUE_TONE_HZ),
            };
            // Main without a route plays on the default output; Cue needs one.
            let target = route.clone().or_else(|| {
                (bus == Bus::Main).then(|| Route {
                    backend: backend.unwrap_or_default().to_owned(),
                    device: String::new(),
                    first_channel: 0,
                })
            });
            if widgets::tile(
                ui,
                vec2(test_width, 26.0),
                &label,
                target.is_some(),
                TileStyle::plain(),
                |p, r, c| {
                    p.text(
                        r.center(),
                        egui::Align2::CENTER_CENTER,
                        format!("{} {label}", icon::WAVEFORM),
                        font(12.0),
                        c,
                    );
                },
            )
            .clicked()
                && let Some(target) = target
            {
                scene.ctl.test_tone(target, freq);
            }
            ui.allocate_ui_with_layout(
                vec2(CHANNELS_WIDTH, 26.0),
                Layout::left_to_right(Align::Center),
                |ui| {
                    ui.set_width(CHANNELS_WIDTH);
                    if let Some(r) = &route {
                        // The existing channel-pair ComboBox, unchanged except
                        // for `.width(CHANNELS_WIDTH - 8.0)`.
                        channel_pair(ui, scene, owner, bus, r, devices);
                    }
                },
            );
            let none_text = match bus {
                Bus::Main => t.tr("settings-default-device"),
                Bus::Cue => t.tr("settings-none"),
            };
            // The existing device ComboBox, unchanged except for
            // `.width(ui.available_width())`.
            device_box(ui, scene, owner, bus, route.as_ref(), backend, devices, &none_text);
        });
    });
    ui.add_space(4.0);
}
```

Move the two ComboBoxes, verbatim, into these helpers:
- `fn channel_pair(ui: &mut Ui, scene: &Scene<'_>, owner: Owner, bus: Bus, r: &Route, devices: &[DeviceInfo])`: the `channels` ComboBox and its `pair` closure;
- `fn device_box(ui: &mut Ui, scene: &Scene<'_>, owner: Owner, bus: Bus, route: Option<&Route>, backend: Option<&str>, devices: &[DeviceInfo], none_text: &str)`: the `device` ComboBox, with `device`, `labels` and `shown` computed inside.

`device_box` has eight parameters. Add
`#[allow(clippy::too_many_arguments)]` with the comment `// The picker's
parts, split for layout only.`

- [ ] **Step 4: Run the tests and check that they pass**

Run: `cargo test -p fp-app --test settings_layout && cargo test -p fp-app --test settings`
Expected: all pass, including the bit-perfect tests in `settings.rs`.

- [ ] **Step 5: Commit**

```bash
git add crates/fp-app/src/ui/settings.rs crates/fp-app/tests/settings_layout.rs
git commit -m "fix(ui): settings rows share one label column

Short labels made narrower columns, so controls started at different
places, and the Outputs test buttons moved with the channel picker.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: Every section on the grid

**Files:**
- Modify: `crates/fp-app/src/ui/settings/carts.rs` (`text_row`)
- Modify: `crates/fp-app/src/ui/settings/midi.rs` (the action rows)
- Modify: `crates/fp-app/src/ui/settings/remote.rs` (`field` and `section`)
- Test: `crates/fp-app/tests/settings_layout.rs` (add)

**Interfaces:**
- Consumes: `labelled_row`, `row`, `LABEL_WIDTH` (Task 4).
- `remote::field` gains a `width: f32` parameter, placed after `current`.

Players, Meters, Analysis, Outputs and Playlists' folder row already use
`row`. The Shortcuts list draws full-width tiles that fill the width. The
playlist table is a table with its own columns. Neither of the last two
changes.

- [ ] **Step 1: Write the failing test** (append to `tests/settings_layout.rs`)

```rust
use arc_swap::ArcSwap;

#[test]
fn controls_start_at_the_same_x_in_every_section() {
    let mut h = open_settings(SCREEN, state(1, 1));
    open(&mut h, "Players");
    let players = left_of(&h, Role::Slider, "Fade time");

    let mut with_cart = state(1, 0);
    let page = with_cart.cartwall.pages[0].id;
    fp_model::apply(
        &mut with_cart,
        fp_model::Command::AssignCartFile {
            page,
            index: 0,
            path: std::path::PathBuf::from("/carts/id.wav"),
        },
    )
    .unwrap();
    let mut h = open_settings(SCREEN, with_cart);
    open(&mut h, "Cartwall");
    let cartwall = left_of(&h, Role::TextInput, "Cart name");

    let cell = Arc::new(ArcSwap::from_pointee(fp_remote::RemoteStatus::default()));
    let (mut h, _) = harness_sized(state(1, 0), SCREEN, move |ui| ui.with_remote_status(cell));
    h.get_by_label("Settings").click();
    h.run_steps(3);
    open(&mut h, "Remote");
    let remote = h
        .get_all_by_role_and_label(Role::TextInput, "Address")
        .next()
        .unwrap()
        .rect()
        .left();

    assert!((players - cartwall).abs() < 0.5, "{players} vs {cartwall}");
    assert!((players - remote).abs() < 0.5, "{players} vs {remote}");
}
```

Add `arc_swap` and `fp_remote` to the imports if they are not visible yet.
Both are already dev-dependencies of `fp-app`: `remote_settings.rs` uses
them.

- [ ] **Step 2: Run the test and check that it fails**

Run: `cargo test -p fp-app --test settings_layout controls_start`
Expected: FAIL. The cart name field starts after a 220 px column, and the Remote address after its inline label.

- [ ] **Step 3: Implement**

`carts.rs`, replace `text_row`. Import `labelled_row` from `super`, and drop the `vec2` or `theme` imports if they become unused.

```rust
/// A labelled single-line text field; the label names the field for
/// accessibility (and tests).
fn text_row(ui: &mut Ui, label: &str, text: &mut String) -> egui::Response {
    labelled_row(ui, label, None, |ui, label_id| {
        ui.add(egui::TextEdit::singleline(text).desired_width(ui.available_width()))
            .labelled_by(label_id)
    })
}
```

`midi.rs`, replace the body of `for (action, key) in actions(n) { … }`.
Import `row` from `super`, and `Align` and `Layout` from `egui`.

```rust
        for (action, key) in actions(n) {
            row(ui, &t.tr(key), None, |ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing = vec2(8.0, 0.0);
                    // Right to left: Clear, Learn, then the binding fills.
                    if config.binding(action).is_some() && button(ui, &t.tr("midi-clear")) {
                        update(scene, |c| c.midi.unbind(action));
                    }
                    let learning = st.midi_learning == Some(action);
                    let label = if learning {
                        t.tr("midi-learning")
                    } else {
                        t.tr("midi-learn")
                    };
                    if button(ui, &label) {
                        if learning {
                            st.midi_learning = None;
                            let _ = midi.requests.send(MidiRequest::CancelLearn);
                        } else {
                            st.midi_learning = Some(action);
                            let _ = midi.requests.send(MidiRequest::Learn(action));
                        }
                    }
                    let bound = config
                        .binding(action)
                        .map(|b| format!("{} · {}", b.device, trigger_text(scene, b.trigger)))
                        .unwrap_or_else(|| t.tr("midi-unbound"));
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        text(ui, bound, theme::NEUTRAL_400);
                    });
                });
            });
        }
```

`remote.rs`:
- `field` gains `width: f32` after `current`, and uses `edit.desired_width(width)` instead of `320.0`;
- import `labelled_row` from `super`;
- in `section`, replace each `ui.horizontal(|ui| { let label = text(…).id; … })` block with rows.

The HTTP blocks become:

```rust
    labelled_row(ui, &t.tr("remote-bind"), None, |ui, label| {
        let w = ui.available_width();
        if let Some(v) = field(ui, st, "http.bind", label, &config.http.bind, w, false, false) {
            update(scene, |c| commit(c, "http.bind", &v));
        }
    });
    labelled_row(ui, &t.tr("remote-port"), None, |ui, label| {
        let port = number(ui, st, "http.port", label, config.http.port.into(), 1024..=65535, "");
        if let Some(port) = port {
            update(scene, |c| commit_number(c, "http.port", port));
        }
    });
    labelled_row(ui, &t.tr("remote-token"), None, |ui, label| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Right to left: Generate, Copy, Show, then the field fills.
            if button(ui, &t.tr("remote-token-generate"))
                && let Some(token) = crate::remote::new_token()
            {
                st.drafts.remove("http.token");
                update(scene, |c| c.remote.http.token = token);
            }
            if button(ui, &t.tr("remote-token-copy")) {
                ui.ctx().copy_text(config.http.token.clone());
            }
            let shown = if st.show_token {
                t.tr("remote-token-hide")
            } else {
                t.tr("remote-token-show")
            };
            if button(ui, &shown) {
                st.show_token = !st.show_token;
            }
            let masked = !st.show_token;
            let w = ui.available_width();
            if let Some(v) = field(ui, st, "http.token", label, &config.http.token, w, false, masked) {
                update(scene, |c| commit(c, "http.token", &v));
            }
        });
    });
```

Then:
- "Web pages allowed…" (`remote-origins`) becomes `labelled_row` with a multiline `field` of width `ui.available_width()`;
- the OSC address and port become two `labelled_row`s, as for HTTP;
- `remote-sources` becomes a multiline `labelled_row`;
- `remote-position-interval` becomes a `labelled_row` around `number`.

Keep the switches (`ui.checkbox`), the status lines and the "HTTP API"/"OSC"
sub-headings as they are. `field` now has eight parameters, so add
`#[allow(clippy::too_many_arguments)]` above it.

- [ ] **Step 4: Run the tests and check that they pass**

Run: `cargo test -p fp-app --test settings_layout && cargo test -p fp-app --test remote_settings --test midi_settings --test settings`
Expected: all pass. The remote tests still find each field by its label through `labelled_by`.

- [ ] **Step 5: Commit**

```bash
git add crates/fp-app/src/ui/settings crates/fp-app/tests/settings_layout.rs
git commit -m "fix(ui): every settings section on the shared grid

Cartwall, MIDI and Remote laid out their rows by hand, so their controls
did not line up with the other sections.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 6: Section header with "Restore defaults"

**Files:**
- Modify: `crates/fp-app/src/ui/settings.rs`:
  - a `SECTIONS` table shared by `nav` and the new header;
  - `section_header`, `restorable` and `confirm_restore`;
  - `SettingsState.confirm_restore`, and `capturing()`;
  - `heading` and its calls are removed.
- Modify: `crates/fp-app/src/ui/settings/{carts,keys,meters,midi,remote}.rs` (remove `heading` calls and imports; `keys.rs` loses its Reset button)
- Modify: both `main.ftl` (add the keys below; remove `shortcut-reset`)
- Test: `crates/fp-app/tests/settings_restore.rs` (new)

**Interfaces:**
- Consumes: `fp_model::{SettingsSection, Command::RestoreDefaults}` (Task 1).
- Produces:
  - `SettingsState.confirm_restore: Option<SettingsSection>`, `pub(super)`;
  - `fn restorable(section: Section) -> Option<SettingsSection>`;
  - `const SECTIONS: [(Section, &str, &str); 9]` (section, glyph, title key);
  - `const SECTION_HEADER_HEIGHT: f32 = 56.0`.

**Strings:**

```
settings-restore = Restore defaults
settings-restore-question = Restore the default values of this section?
settings-restore-cancel = Cancel
settings-restore-confirm = Restore
```

```
settings-restore = Restaurar valores por defecto
settings-restore-question = ¿Restaurar los valores por defecto de esta sección?
settings-restore-cancel = Cancelar
settings-restore-confirm = Restaurar
```

- [ ] **Step 1: Write the failing tests**

```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Feedback 2 spec O2: "Restore defaults" in the Settings sections.

mod support;

use egui::Key;
use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::AppUi;
use fp_model::{Command, SettingsSection};
use support::{harness, state};

const QUESTION: &str = "Restore the default values of this section?";

fn open(section: &str, s: fp_model::AppState) -> (Harness<'static, AppUi>, std::sync::Arc<support::Fake>) {
    let (mut h, fake) = harness(s);
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, section).click();
    h.run_steps(2);
    fake.take_sent();
    (h, fake)
}

#[test]
fn players_restores_its_defaults_after_confirmation() {
    let mut s = state(1, 1);
    s.config.players.fade_ms = 3000;
    let (mut h, fake) = open("Players", s);
    h.get_by_label("Restore defaults").click();
    h.run_steps(2);
    assert!(h.query_by_label(QUESTION).is_some());
    assert!(fake.take_sent().is_empty());
    h.get_by_label("Restore").click();
    h.run_steps(2);
    assert_eq!(
        fake.take_sent(),
        vec![Command::RestoreDefaults(SettingsSection::Players)]
    );
    assert_eq!(fake.state.load().config.players.fade_ms, 1000);
    assert!(h.query_by_label(QUESTION).is_none());
}

#[test]
fn cancel_and_escape_change_nothing_and_keep_settings_open() {
    let mut s = state(1, 1);
    s.config.meter.floor_db = -40.0;
    let (mut h, fake) = open("Meters", s);
    h.get_by_label("Restore defaults").click();
    h.run_steps(2);
    h.get_by_label("Cancel").click();
    h.run_steps(2);
    assert!(h.query_by_label(QUESTION).is_none());
    h.get_by_label("Restore defaults").click();
    h.run_steps(2);
    h.key_press(Key::Escape);
    h.run_steps(3);
    assert!(h.query_by_label(QUESTION).is_none());
    // Settings is still open.
    assert!(h.query_by_role_and_label(Role::Button, "Meters").is_some());
    assert!(fake.take_sent().is_empty());
    assert_eq!(fake.state.load().config.meter.floor_db, -40.0);
}

#[test]
fn each_section_with_the_button_restores_itself() {
    for (section, expected) in [
        ("Meters", SettingsSection::Meters),
        ("Analysis", SettingsSection::Analysis),
        ("Keyboard shortcuts", SettingsSection::Shortcuts),
    ] {
        let (mut h, fake) = open(section, state(1, 1));
        h.get_by_label("Restore defaults").click();
        h.run_steps(2);
        h.get_by_label("Restore").click();
        h.run_steps(2);
        assert_eq!(fake.take_sent(), vec![Command::RestoreDefaults(expected)], "{section}");
    }
}

#[test]
fn hardware_security_and_show_data_sections_have_no_button() {
    for section in ["Audio outputs", "Playlists", "Cartwall", "MIDI", "Remote"] {
        let (h, _) = open(section, state(1, 1));
        assert!(h.query_by_label("Restore defaults").is_none(), "{section}");
    }
}
```

- [ ] **Step 2: Run the tests and check that they fail**

Run: `cargo test -p fp-app --test settings_restore`
Expected: FAIL. There is no "Restore defaults" label.

- [ ] **Step 3: Implement**

In `settings.rs`:

```rust
/// Every section: its nav glyph and its title.
const SECTIONS: [(Section, &str, &str); 9] = [
    (Section::Outputs, icon::SPEAKER_HIGH, "settings-tab-outputs"),
    (Section::Players, icon::SLIDERS_HORIZONTAL, "settings-tab-players"),
    (Section::Meters, icon::GAUGE, "settings-tab-meters"),
    (Section::Analysis, icon::WAVEFORM, "settings-tab-analysis"),
    (Section::Playlists, icon::PLAYLIST, "settings-tab-playlists"),
    (Section::Cartwall, icon::SQUARES_FOUR, "settings-tab-cartwall"),
    (Section::Shortcuts, icon::KEYBOARD, "settings-tab-shortcuts"),
    (Section::Midi, icon::PIANO_KEYS, "settings-tab-midi"),
    (Section::Remote, icon::BROADCAST, "settings-tab-remote"),
];

const SECTION_HEADER_HEIGHT: f32 = 56.0;

/// The model section a Settings section restores; `None` where the spec
/// gives no button (hardware, security, show data).
fn restorable(section: Section) -> Option<SettingsSection> {
    match section {
        Section::Players => Some(SettingsSection::Players),
        Section::Meters => Some(SettingsSection::Meters),
        Section::Analysis => Some(SettingsSection::Analysis),
        Section::Shortcuts => Some(SettingsSection::Shortcuts),
        Section::Outputs
        | Section::Playlists
        | Section::Cartwall
        | Section::Midi
        | Section::Remote => None,
    }
}

/// The section's title, and "Restore defaults" on the right where the
/// section has one. It stays put while the body scrolls.
fn section_header(ui: &mut Ui, scene: &Scene<'_>, st: &mut SettingsState) {
    let t = scene.i18n;
    let key = SECTIONS
        .iter()
        .find(|(s, _, _)| *s == st.section)
        .map_or("settings-title", |(_, _, key)| *key);
    let width = ui.available_width();
    ui.allocate_ui_with_layout(
        vec2(width, SECTION_HEADER_HEIGHT),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.set_width(width);
            ui.add_space(24.0);
            ui.add(
                egui::Label::new(
                    RichText::new(t.tr(key))
                        .font(font_medium(20.0))
                        .color(theme::TEXT),
                )
                .selectable(false),
            );
            if let Some(section) = restorable(st.section) {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.add_space(24.0);
                    if button(ui, &t.tr("settings-restore")) {
                        st.confirm_restore = Some(section);
                    }
                });
            }
        },
    );
}

/// Asks before restoring; drawn over Settings.
fn confirm_restore(ctx: &egui::Context, scene: &Scene<'_>, st: &mut SettingsState) {
    let Some(section) = st.confirm_restore else {
        return;
    };
    let t = scene.i18n;
    let mut answer = None;
    let modal = egui::Modal::new(egui::Id::new("settings-restore"))
        .frame(egui::Frame::new().fill(theme::SURFACE).inner_margin(20.0))
        .backdrop_color(theme::NEUTRAL_900.gamma_multiply(0.5))
        .show(ctx, |ui| {
            ui.set_width(360.0);
            ui.spacing_mut().item_spacing = vec2(8.0, 12.0);
            ui.add(
                egui::Label::new(
                    RichText::new(t.tr("settings-restore-question"))
                        .font(font(13.0))
                        .color(theme::TEXT),
                )
                .selectable(false)
                .wrap(),
            );
            ui.horizontal(|ui| {
                if button(ui, &t.tr("settings-restore-cancel")) {
                    answer = Some(false);
                }
                if button(ui, &t.tr("settings-restore-confirm")) {
                    answer = Some(true);
                }
            });
        });
    // Esc or a click on the backdrop is "cancel".
    if answer.is_none() && modal.should_close() {
        answer = Some(false);
    }
    match answer {
        Some(true) => {
            scene.ctl.send(Command::RestoreDefaults(section));
            st.confirm_restore = None;
        }
        Some(false) => st.confirm_restore = None,
        None => {}
    }
}
```

Then:
- `nav` iterates `SECTIONS` instead of its inline array;
- in `show`'s body column, call `section_header(ui, scene, st);` right after `ui.set_min_size(…)`, before the `ScrollArea`. Change the Frame's margin to `egui::Margin { left: 24, right: 24, top: 0, bottom: 20 }`;
- delete every `heading(ui, …)` call: in `outputs`, `players`, `analysis` and `playlists`, and in `carts`, `keys`, `meters`, `midi` and `remote::section`. Then delete `fn heading` and drop it from each module's `use super::{…}`;
- call `confirm_restore(ctx, scene, st);` at the end of `show`, after the settings modal and before `if modal.should_close()`, so that it is the top modal;
- `SettingsState` gains `pub(super) confirm_restore: Option<SettingsSection>`;
- `capturing()` returns `self.keys.capturing() || self.midi_learning.is_some() || self.confirm_restore.is_some()`, so that Esc closes the confirmation and not Settings (`app.rs` checks `capturing()`);
- import `fp_model::SettingsSection`.

`keys.rs`: delete the `small(ui, &t.tr("shortcut-reset"), …)` button. Keep
the help label in its `ui.horizontal`. Remove `shortcut-reset` from both
locales.

- [ ] **Step 4: Run the tests and check that they pass**

Run: `cargo test -p fp-app --test settings_restore && cargo test -p fp-app --test settings --test settings_layout --test midi_settings --test remote_settings --test i18n`
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add crates/fp-app/src/ui/settings.rs crates/fp-app/src/ui/settings crates/fp-app/locales crates/fp-app/tests/settings_restore.rs
git commit -m "feat(ui): restore defaults per settings section

Each section that holds preferences gets a Restore defaults button in a
fixed header, with a confirmation. Outputs, MIDI and Remote depend on
the hardware or hold security settings, and Playlists and Cartwall hold
show data: they have none.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 7: Hide the Null backend

**Files:**
- Modify: `crates/fp-app/src/ui/settings.rs` (`outputs`: the backend ComboBox; new `listed_backends` and `backend_name`; unit tests)
- Modify: both `main.ftl`
- Test: `crates/fp-app/tests/settings.rs` (add)

**Interfaces:**
- Produces:
  - `fn listed_backends<'a>(all: &'a [BackendChoice], configured: Option<&str>) -> Vec<&'a BackendChoice>`: every backend but `null`, plus `null` when it is the one configured;
  - `fn backend_name(t: &crate::i18n::I18n, id: &str) -> String`: `settings-backend-null` for `null`, `fp_backends::display_name` otherwise.

**Strings:** `settings-backend-null = No output (silent)` / `settings-backend-null = Sin salida (silencio)`.

- [ ] **Step 1: Write the failing tests**

Unit tests, in `settings.rs`'s `mod tests`:

```rust
    fn choice(id: &str) -> BackendChoice {
        BackendChoice {
            id: id.to_owned(),
            unavailable: None,
            devices: Vec::new(),
        }
    }

    #[test]
    fn null_is_listed_only_when_configured() {
        let all = [choice("alsa"), choice("null")];
        let ids = |configured| -> Vec<String> {
            listed_backends(&all, configured)
                .into_iter()
                .map(|b| b.id.clone())
                .collect()
        };
        assert_eq!(ids(None), vec!["alsa"]);
        assert_eq!(ids(Some("alsa")), vec!["alsa"]);
        assert_eq!(ids(Some("null")), vec!["alsa", "null"]);
    }
```

In `tests/settings.rs`:

```rust
fn outputs_with_null(configured: Option<&str>) -> egui_kittest::Harness<'static, fp_app::ui::app::AppUi> {
    let mut s = state(1, 1);
    s.config.outputs.backend = configured.map(str::to_owned);
    let backends: Vec<Arc<dyn AudioBackend>> = vec![
        Arc::new(OfflineBackend::new()),
        Arc::new(fp_backends::NullBackend),
    ];
    let (mut h, _) = harness_with_backends(s, backends);
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Audio outputs").click();
    for _ in 0..200 {
        h.run_steps(1);
        if h.query_all_by_label("Test Main").next().is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    h
}

#[test]
fn the_null_backend_is_not_offered() {
    let mut h = outputs_with_null(Some("offline"));
    h.get_by_value("Offline").click();
    h.run_steps(2);
    assert!(h.query_by_label("System default").is_some());
    assert!(h.query_by_label("Null").is_none());
    assert!(h.query_by_label("No output (silent)").is_none());
}

#[test]
fn a_configured_null_backend_shows_as_no_output() {
    let h = outputs_with_null(Some("null"));
    assert!(h.query_by_value("No output (silent)").is_some());
}
```

- [ ] **Step 2: Run the tests and check that they fail**

Run: `cargo test -p fp-app --lib settings::tests::null && cargo test -p fp-app --test settings null`
Expected:
- the unit test fails to compile, because `listed_backends` is missing;
- `the_null_backend_is_not_offered` fails, because "Null" is listed;
- `a_configured_null_backend_shows_as_no_output` fails, because the box shows "Null".

- [ ] **Step 3: Implement**

```rust
/// The audio systems the list offers. `null` (silence) is for tests and
/// headless use: it shows only when the configuration names it
/// (feedback 2 spec O5).
fn listed_backends<'a>(all: &'a [BackendChoice], configured: Option<&str>) -> Vec<&'a BackendChoice> {
    all.iter()
        .filter(|b| b.id != "null" || configured == Some("null"))
        .collect()
}

fn backend_name(t: &crate::i18n::I18n, id: &str) -> String {
    if id == "null" {
        t.tr("settings-backend-null")
    } else {
        fp_backends::display_name(id).to_owned()
    }
}
```

In `outputs`:
- the selected text uses `backend_name(t, id)` instead of `fp_backends::display_name(id).to_owned()`;
- the loop is `for b in listed_backends(&backends, current.as_deref())`;
- the unavailable label's `name` argument and the plain label use `backend_name(t, &b.id)`.

Check the type of `scene.i18n` (`crate::i18n::I18n`) and adjust the
parameter type if it differs.

- [ ] **Step 4: Run the tests and check that they pass**

Run: `cargo test -p fp-app --lib settings && cargo test -p fp-app --test settings`
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add crates/fp-app/src/ui/settings.rs crates/fp-app/locales crates/fp-app/tests/settings.rs
git commit -m "feat(ui): hide the silent backend from the audio systems

Null is a test and headless backend; operators took it for a real
system. It shows, as No output, only when the configuration names it.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 8: Restart pending: notice, pill and guard

**Files:**
- Modify: `crates/fp-app/src/ui/exit_guard.rs` (`ExitIntent::Restart` and its strings)
- Modify: `crates/fp-app/src/ui/app.rs`:
  - `AppUi` fields `started` and `restart`, with `with_started_config` and `restart_flag`;
  - `ViewState.restart_requested`;
  - `begin_restart`, `restart_reason_key`;
  - the `top_bar` pill;
  - the `SettingsDeps` field and the use of `settings::Outcome`.
- Modify: `crates/fp-app/src/ui/settings.rs`:
  - `SettingsDeps.restart_pending`;
  - `show` returns `Outcome`;
  - the footer notice and **Restart now**;
  - the Outputs restart note is removed.
- Modify: both `main.ftl` (add the keys below; remove `settings-restart-note`)
- Test: `crates/fp-app/tests/restart_ui.rs` (new)

**Interfaces:**
- Consumes:
  - `fp_model::{restart_pending, RestartReason, on_air}` (Task 2 and plan 1);
  - `exit_guard::{show, stop_commands}` (plan 1);
  - `remote::flush` (exists).
- Produces:
  - `ExitIntent::Restart`;
  - `pub fn with_started_config(self, config: fp_model::Config) -> Self` (`AppUi::new` defaults it to `ctl.model().config.clone()`);
  - `pub fn restart_flag(&self) -> Arc<AtomicBool>`, which reads true once a restart was confirmed;
  - `pub(crate) struct Outcome { pub open: bool, pub restart: bool }`, returned by `settings::show`;
  - `SettingsDeps.restart_pending: bool`.

**Strings:**

```
settings-restart-pending = Some changes take effect after a restart.
settings-restart-now = Restart now
top-restart-pending = Restart pending
tip-restart-pending = Restart to apply: { $reasons }
restart-reason-audio-system = audio system
restart-reason-sample-rate = sample rate
restart-reason-buffer-size = buffer size
restart-reason-routes = outputs
restart-reason-bit-perfect = bit-perfect devices
restart-reason-limits = limits
restart-reason-tuning = engine tuning
exit-guard-restart-body = Restarting stops everything that is playing:
exit-guard-restart-confirm = Stop and restart
```

```
settings-restart-pending = Algunos cambios se aplican tras reiniciar.
settings-restart-now = Reiniciar ahora
top-restart-pending = Reinicio pendiente
tip-restart-pending = Reinicia para aplicar: { $reasons }
restart-reason-audio-system = sistema de audio
restart-reason-sample-rate = frecuencia de muestreo
restart-reason-buffer-size = tamaño del búfer
restart-reason-routes = salidas
restart-reason-bit-perfect = dispositivos bit-perfect
restart-reason-limits = límites
restart-reason-tuning = ajustes del motor
exit-guard-restart-body = Al reiniciar se detendrá todo lo que está sonando:
exit-guard-restart-confirm = Detener y reiniciar
```

- [ ] **Step 1: Write the failing tests**

```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Feedback 2 spec O4: settings that wait for a restart, and Restart now.

mod support;

use std::sync::Arc;
use std::sync::atomic::Ordering;

use arc_swap::ArcSwap;
use egui::accesskit::Role;
use egui::{ViewportCommand, ViewportId};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::AppUi;
use fp_model::Command;
use support::{Fake, harness, harness_from, state};

const NOTICE: &str = "Some changes take effect after a restart.";
const PILL: &str = "Restart pending";

fn sent_viewport_command(h: &Harness<'static, AppUi>, wanted: &ViewportCommand) -> bool {
    h.output()
        .viewport_output
        .get(&ViewportId::ROOT)
        .is_some_and(|v| v.commands.contains(wanted))
}

fn set_rate(h: &mut Harness<'static, AppUi>, fake: &Fake, rate: u32) {
    let mut s = (**fake.state.load()).clone();
    s.config.outputs.sample_rate = rate;
    fake.state.store(Arc::new(s));
    h.run_steps(2);
}

fn playing(players: usize) -> fp_model::AppState {
    let mut s = state(players, 3);
    let p = s.players[0].id;
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    s
}

#[test]
fn nothing_changed_shows_no_notice_and_no_pill() {
    let (mut h, _) = harness(state(1, 3));
    assert!(h.query_by_label(PILL).is_none());
    h.get_by_label("Settings").click();
    h.run_steps(2);
    assert!(h.query_by_label(NOTICE).is_none());
    assert!(h.query_by_label("Restart now").is_none());
}

#[test]
fn a_live_setting_needs_no_restart() {
    let (mut h, fake) = harness(state(1, 3));
    let mut s = (**fake.state.load()).clone();
    s.config.players.fade_ms = 3000;
    fake.state.store(Arc::new(s));
    h.run_steps(2);
    assert!(h.query_by_label(PILL).is_none());
}

#[test]
fn a_restart_only_change_shows_the_pill_and_the_notice() {
    let (mut h, fake) = harness(state(1, 3));
    set_rate(&mut h, &fake, 44_100);
    assert!(h.query_by_role_and_label(Role::Button, PILL).is_some());
    h.get_by_label("Settings").click();
    h.run_steps(2);
    assert!(h.query_by_label(NOTICE).is_some());
    assert!(h.query_by_label("Restart now").is_some());
}

#[test]
fn changing_back_hides_the_pill() {
    let (mut h, fake) = harness(state(1, 3));
    set_rate(&mut h, &fake, 44_100);
    set_rate(&mut h, &fake, 48_000);
    assert!(h.query_by_label(PILL).is_none());
}

#[test]
fn restart_now_with_nothing_on_air_closes_for_a_restart() {
    let (mut h, fake) = harness(state(1, 3));
    set_rate(&mut h, &fake, 44_100);
    h.get_by_label("Settings").click();
    h.run_steps(2);
    fake.take_sent();
    h.get_by_label("Restart now").click();
    h.run_steps(1);
    assert!(h.query_by_label("Audio is on air").is_none());
    assert!(sent_viewport_command(&h, &ViewportCommand::Close));
    assert!(h.state().restart_flag().load(Ordering::Acquire));
    assert!(fake.take_sent().is_empty());
}

#[test]
fn the_pill_restarts_at_once_when_nothing_is_on_air() {
    let (mut h, fake) = harness(state(1, 3));
    set_rate(&mut h, &fake, 44_100);
    fake.take_sent();
    h.get_by_label(PILL).click();
    h.run_steps(1);
    // No confirmation: nothing would be cut.
    assert!(h.query_by_label("Audio is on air").is_none());
    assert!(sent_viewport_command(&h, &ViewportCommand::Close));
    assert!(h.state().restart_flag().load(Ordering::Acquire));
    assert!(fake.take_sent().is_empty());
}

#[test]
fn the_pill_asks_first_while_audio_is_on_air() {
    let (mut h, fake) = harness(playing(1));
    let p = fake.player(0);
    set_rate(&mut h, &fake, 44_100);
    h.get_by_label(PILL).click();
    h.run_steps(2);
    assert!(h.query_by_label("Audio is on air").is_some());
    assert!(h.query_by_label("Restarting stops everything that is playing:").is_some());
    assert!(!h.state().restart_flag().load(Ordering::Acquire));
    h.get_by_label("Stop and restart").click();
    h.run_steps(1);
    assert_eq!(fake.take_sent(), vec![Command::Stop(p)]);
    assert!(sent_viewport_command(&h, &ViewportCommand::Close));
    assert!(h.state().restart_flag().load(Ordering::Acquire));
}

#[test]
fn cancelling_the_guard_keeps_running() {
    let (mut h, fake) = harness(playing(1));
    set_rate(&mut h, &fake, 44_100);
    h.get_by_label(PILL).click();
    h.run_steps(2);
    h.get_by_label("Cancel").click();
    h.run_steps(2);
    assert!(h.query_by_label("Audio is on air").is_none());
    assert!(!h.state().restart_flag().load(Ordering::Acquire));
    assert!(!sent_viewport_command(&h, &ViewportCommand::Close));
    assert!(fake.take_sent().is_empty());
}

#[test]
fn restart_now_applies_a_remote_draft_first() {
    let cell = Arc::new(ArcSwap::from_pointee(fp_remote::RemoteStatus::default()));
    let (mut h, fake) = harness_from(state(1, 0), move |ui| ui.with_remote_status(cell));
    set_rate(&mut h, &fake, 44_100);
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Remote").click();
    h.run_steps(2);
    h.get_all_by_role_and_label(Role::TextInput, "Address")
        .next()
        .unwrap()
        .focus();
    h.run_steps(2);
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    h.run_steps(1);
    for c in "10.0.0.9".chars() {
        h.get_all_by_role_and_label(Role::TextInput, "Address")
            .next()
            .unwrap()
            .type_text(&c.to_string());
        h.run_steps(1);
    }
    // The field still has focus: the address is only a draft.
    assert_ne!(fake.state.load().config.remote.http.bind, "10.0.0.9");
    h.get_by_label("Restart now").click();
    h.run_steps(1);
    assert_eq!(fake.state.load().config.remote.http.bind, "10.0.0.9");
}
```

In `restart_now_applies_a_remote_draft_first`, the single `run_steps(1)` after
the click matters. Without the flush, the field commits one frame later, after
the window was asked to close; the test checks the click's own frame. Watch
this test fail before adding `remote::flush`. If it passes without the flush,
record that in the ledger and check the address after `h.step()` instead of
`run_steps(1)`, so that it pins the flush.

- [ ] **Step 2: Run the tests and check that they fail**

Run: `cargo test -p fp-app --test restart_ui`
Expected: compile error, because `restart_flag` is missing. Once it exists as a stub that returns `Arc::new(AtomicBool::new(false))`, the tests fail on the missing pill and notice.

- [ ] **Step 3: Implement**

`exit_guard.rs`:

```rust
pub enum ExitIntent {
    /// Close the window and quit.
    Close,
    /// Restart the application to apply settings (feedback 2 spec O4).
    Restart,
}
```

In `show`:

```rust
    let (body, confirm) = match intent {
        ExitIntent::Close => ("exit-guard-close-body", "exit-guard-close-confirm"),
        ExitIntent::Restart => ("exit-guard-restart-body", "exit-guard-restart-confirm"),
    };
```

`settings.rs`:
- `SettingsDeps` gains `pub restart_pending: bool`.
- `show` returns `Outcome`:

```rust
/// What the modal asks of the application after a frame.
pub(crate) struct Outcome {
    /// `false` once the modal should close.
    pub open: bool,
    /// The operator pressed Restart now.
    pub restart: bool,
}
```

- Declare `let mut restart = false;` next to `let mut open = true;`, and return `Outcome { open, restart }`.
- In the footer, right after the Close tile's `if … { open = false; }`, while still in the right-to-left layout:

```rust
                    if deps.restart_pending {
                        let label = t.tr("settings-restart-now");
                        let width = ui
                            .painter()
                            .layout_no_wrap(label.clone(), font_medium(13.0), theme::TEXT)
                            .size()
                            .x
                            + 28.0;
                        let style = TileStyle {
                            border: theme::AMBER,
                            ..TileStyle::plain()
                        };
                        if widgets::tile(ui, vec2(width, 30.0), &label, true, style, |p, r, c| {
                            p.text(
                                r.center(),
                                egui::Align2::CENTER_CENTER,
                                &label,
                                font_medium(13.0),
                                c,
                            );
                        })
                        .clicked()
                        {
                            restart = true;
                        }
                    }
```

- The footer text becomes:

```rust
                        let (text, color) = match (&deps.notice, deps.restart_pending) {
                            (Some(n), _) => (n.clone(), theme::AMBER),
                            (None, true) => (t.tr("settings-restart-pending"), theme::AMBER),
                            (None, false) => (t.tr("settings-applies-now"), theme::NEUTRAL_500),
                        };
```

- After the settings modal, before `confirm_restore`: `if restart { remote::flush(scene, &mut st.remote); }`. A typed address or token is saved before the window closes.
- In `outputs`, delete the `settings-restart-note` label and the `ui.add_space(8.0)` after it. Remove the key from both locales, and `grep -rn "take effect the next time" crates` to update any test.

`app.rs`:
- Add `use std::sync::atomic::{AtomicBool, Ordering};` and `use fp_model::RestartReason;`.
- `ViewState` gains:

```rust
    /// Restart now was pressed (Settings footer or the top-bar pill).
    pub restart_requested: bool,
```

- `AppUi` gains fields, set in `new`:

```rust
    /// The configuration the engine was built with (feedback 2 spec O4).
    started: fp_model::Config,
    /// Set once a restart is confirmed; `main` reads it after the window
    /// closes.
    restart: Arc<AtomicBool>,
```

```rust
            started: ctl.model().config.clone(),
            restart: Arc::new(AtomicBool::new(false)),
```

`ctl` is moved into the struct in `new`, so read the config before the
`Self { … }` literal: `let started = ctl.model().config.clone();`.

- Methods:

```rust
    /// The configuration the audio engine was built with; a change to a
    /// start-up setting after it shows "Restart pending".
    pub fn with_started_config(mut self, config: fp_model::Config) -> Self {
        self.started = config;
        self
    }

    /// True once the operator confirmed a restart: `main` starts the
    /// application again after its normal shutdown.
    pub fn restart_flag(&self) -> Arc<AtomicBool> {
        self.restart.clone()
    }
```

- A free function:

```rust
/// Closes the window for a restart. The close guard lets it through: what
/// was on air has been stopped, or nothing was.
fn begin_restart(restart: &AtomicBool, view: &mut ViewState, ctx: &egui::Context) {
    restart.store(true, Ordering::Release);
    view.close_confirmed = true;
    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
}

fn restart_reason_key(reason: RestartReason) -> &'static str {
    match reason {
        RestartReason::AudioSystem => "restart-reason-audio-system",
        RestartReason::SampleRate => "restart-reason-sample-rate",
        RestartReason::BufferSize => "restart-reason-buffer-size",
        RestartReason::Routes => "restart-reason-routes",
        RestartReason::BitPerfect => "restart-reason-bit-perfect",
        RestartReason::Limits => "restart-reason-limits",
        RestartReason::Tuning => "restart-reason-tuning",
    }
}
```

- In `AppUi::ui`, before building the `Scene`:

```rust
        let pending = fp_model::restart_pending(&self.started, &state.config);
```

  Pass `&pending` to `top_bar`, whose signature becomes `fn top_bar(ui: &mut Ui, scene: &Scene<'_>, view_state: &mut ViewState, pending: &[RestartReason])`. In `SettingsDeps`, set `restart_pending: !pending.is_empty()`, and replace the call with:

```rust
            let outcome = settings::show(&ctx, &scene, &mut self.settings, &deps);
            self.view.settings_open = outcome.open;
            self.view.restart_requested |= outcome.restart;
```

- Right before `if let Some(intent) = self.view.exit_guard {`:

```rust
        // O4: Restart now asks the close guard first when audio is on air.
        if std::mem::take(&mut self.view.restart_requested) {
            if fp_model::on_air(&state).is_empty() {
                begin_restart(&self.restart, &mut self.view, &ctx);
            } else {
                self.view.exit_guard = Some(ExitIntent::Restart);
            }
        }
```

- In the guard's `Some(true)` arm, add `ExitIntent::Restart => begin_restart(&self.restart, &mut self.view, &ctx),` next to `ExitIntent::Close`.

- In `top_bar`, inside the right-to-left block after the About tile (so it sits to About's left):

```rust
        if !pending.is_empty() {
            let i18n = scene.i18n;
            let label = i18n.tr("top-restart-pending");
            let reasons = pending
                .iter()
                .map(|r| i18n.tr(restart_reason_key(*r)))
                .collect::<Vec<_>>()
                .join(", ");
            let tip = i18n.tr_args("tip-restart-pending", &[("reasons", reasons.into())]);
            let width = ui
                .painter()
                .layout_no_wrap(label.clone(), font(12.0), theme::AMBER)
                .size()
                .x
                + 34.0;
            let style = TileStyle {
                border: theme::AMBER,
                content: theme::AMBER,
                hover_fill: theme::NEUTRAL_800,
                ..TileStyle::plain()
            };
            if widgets::tile(ui, vec2(width, 24.0), &label, true, style, |p, r, c| {
                p.text(
                    r.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("{} {label}", egui_phosphor::regular::ARROWS_CLOCKWISE),
                    font(12.0),
                    c,
                );
            })
            .on_hover_text(tip)
            .clicked()
            {
                view_state.restart_requested = true;
            }
        }
```

- [ ] **Step 4: Run the tests and check that they pass**

Run: `cargo test -p fp-app --test restart_ui && cargo test -p fp-app --test exit_guard --test settings --test about --test main_screen --test i18n`
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add crates/fp-app/src/ui crates/fp-app/locales crates/fp-app/tests/restart_ui.rs
git commit -m "feat(ui): show pending restarts and offer Restart now

Changes to outputs, limits and tuning silently waited for the next start.
Now the Settings footer and a top-bar pill say so, and Restart now goes
through the on-air guard.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 9: Relaunch after the shutdown

**Files:**
- Create: `crates/fp-app/src/restart.rs`
- Modify: `crates/fp-app/src/lib.rs` (`pub mod restart;`)
- Modify: `crates/fp-app/src/main.rs` (`run` returns `Exit`; `main` relaunches after dropping the lock; the started config goes to `AppUi`)
- Modify: both `main.ftl` (`restart-failed`)
- Test: `crates/fp-app/tests/relaunch.rs` (new)

**Interfaces:**
- Consumes: `AppUi::{with_started_config, restart_flag}` (Task 8), `bootstrap::HOME_VAR` (exists).
- Produces:
  - `pub struct Relaunch { pub program: OsString, pub args: Vec<OsString> }`, deriving `Debug, Clone, PartialEq, Eq`;
  - `pub struct Launcher { pub exe: Option<PathBuf>, pub appimage: Option<OsString>, pub flatpak: bool, pub home: Option<OsString> }`, deriving `Debug, Clone, Default`, with `pub fn from_env() -> Self` and `pub fn plan(&self) -> Option<Relaunch>`;
  - `pub fn spawn(plan: &Relaunch) -> std::io::Result<()>`;
  - `pub fn relaunch() -> std::io::Result<()>`.

**Strings:**
- `restart-failed = Fauste Player could not start again. Start it from your applications menu.`
- `restart-failed = Fauste Player no ha podido volver a arrancar. Ábrelo desde el menú de aplicaciones.`

- [ ] **Step 1: Write the failing tests**

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback 2 spec O4: how the application starts itself again.

use std::ffi::OsString;
use std::path::PathBuf;

use fp_app::restart::{Launcher, Relaunch, spawn};

fn plain(exe: &str) -> Launcher {
    Launcher {
        exe: Some(PathBuf::from(exe)),
        ..Launcher::default()
    }
}

#[test]
fn a_plain_install_starts_its_own_executable_without_arguments() {
    assert_eq!(
        plain("/usr/bin/fauste-player").plan(),
        Some(Relaunch {
            program: "/usr/bin/fauste-player".into(),
            args: Vec::new(),
        })
    );
}

#[test]
fn a_replaced_executable_starts_from_its_path() {
    assert_eq!(
        plain("/usr/bin/fauste-player (deleted)").plan().unwrap().program,
        OsString::from("/usr/bin/fauste-player")
    );
}

#[test]
fn an_appimage_starts_the_image_not_the_mounted_copy() {
    let launcher = Launcher {
        appimage: Some("/home/op/Apps/Fauste_Player.AppImage".into()),
        ..plain("/tmp/.mount_FausteX/usr/bin/fauste-player")
    };
    assert_eq!(
        launcher.plan().unwrap().program,
        OsString::from("/home/op/Apps/Fauste_Player.AppImage")
    );
}

#[test]
fn a_flatpak_starts_through_the_portal() {
    let launcher = Launcher {
        flatpak: true,
        ..plain("/app/bin/fauste-player")
    };
    assert_eq!(
        launcher.plan(),
        Some(Relaunch {
            program: "flatpak-spawn".into(),
            args: vec!["/app/bin/fauste-player".into()],
        })
    );
}

#[test]
fn a_flatpak_keeps_fauste_home() {
    let launcher = Launcher {
        flatpak: true,
        home: Some("/var/home/op/show".into()),
        ..plain("/app/bin/fauste-player")
    };
    assert_eq!(
        launcher.plan().unwrap().args,
        vec![
            OsString::from("--env=FAUSTE_HOME=/var/home/op/show"),
            OsString::from("/app/bin/fauste-player"),
        ]
    );
}

#[test]
fn without_an_executable_there_is_no_plan() {
    assert_eq!(Launcher::default().plan(), None);
}

#[test]
fn starting_a_missing_program_is_an_error() {
    let plan = Relaunch {
        program: "/nonexistent/fauste-player-test".into(),
        args: Vec::new(),
    };
    assert!(spawn(&plan).is_err());
}
```

- [ ] **Step 2: Run the tests and check that they fail**

Run: `cargo test -p fp-app --test relaunch`
Expected: compile error, because `fp_app::restart` is missing.

- [ ] **Step 3: Implement**

`crates/fp-app/src/restart.rs`:

```rust
//! Starting the application again (feedback 2 spec O4). `main` calls
//! `relaunch` after the normal shutdown (session saved, audio stopped,
//! instance lock released). The new process starts like any other start:
//! nothing goes on air by itself.

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::bootstrap::HOME_VAR;

/// The program to start, and its arguments. No playlist arguments are
/// passed: those of the first start were imported and saved already.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Relaunch {
    pub program: OsString,
    pub args: Vec<OsString>,
}

/// Where the running application came from.
#[derive(Debug, Clone, Default)]
pub struct Launcher {
    /// This executable.
    pub exe: Option<PathBuf>,
    /// The AppImage file (`$APPIMAGE`): its mounted copy goes away with
    /// this process.
    pub appimage: Option<OsString>,
    /// Inside a Flatpak sandbox, which ends with this process.
    pub flatpak: bool,
    /// `FAUSTE_HOME`, which a new Flatpak sandbox does not inherit.
    pub home: Option<OsString>,
}

/// Linux names an executable that a package upgrade replaced
/// `<path> (deleted)`; the new one is at `<path>`.
fn live_path(exe: PathBuf) -> PathBuf {
    match exe.to_str().and_then(|s| s.strip_suffix(" (deleted)")) {
        Some(path) => PathBuf::from(path),
        None => exe,
    }
}

impl Launcher {
    pub fn from_env() -> Self {
        let set = |name: &str| std::env::var_os(name).filter(|v| !v.is_empty());
        Self {
            exe: std::env::current_exe().ok(),
            appimage: set("APPIMAGE"),
            flatpak: set("FLATPAK_ID").is_some(),
            home: set(HOME_VAR),
        }
    }

    /// How to start the application again; `None` when this executable
    /// is unknown.
    pub fn plan(&self) -> Option<Relaunch> {
        if let Some(image) = &self.appimage {
            return Some(Relaunch {
                program: image.clone(),
                args: Vec::new(),
            });
        }
        let exe = live_path(self.exe.clone()?);
        if self.flatpak {
            // A new sandbox through the Flatpak portal, which every app
            // may use.
            let mut args = Vec::new();
            if let Some(home) = &self.home {
                let mut env = OsString::from(format!("--env={HOME_VAR}="));
                env.push(home);
                args.push(env);
            }
            args.push(exe.into_os_string());
            return Some(Relaunch {
                program: "flatpak-spawn".into(),
                args,
            });
        }
        Some(Relaunch {
            program: exe.into_os_string(),
            args: Vec::new(),
        })
    }
}

/// Starts `plan` and leaves it running: this process ends right after.
#[allow(clippy::zombie_processes)] // the parent exits; the child is not waited for
pub fn spawn(plan: &Relaunch) -> std::io::Result<()> {
    Command::new(&plan.program)
        .args(&plan.args)
        .stdin(Stdio::null())
        .spawn()
        .map(|_child| ())
}

/// Starts the application again, the same way it was started.
pub fn relaunch() -> std::io::Result<()> {
    let plan = Launcher::from_env().plan().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "the application's executable is unknown",
        )
    })?;
    tracing::info!(program = ?plan.program, args = ?plan.args, "starting again");
    spawn(&plan)
}
```

If clippy reports `clippy::zombie_processes` as an unknown lint, remove that
`allow`. If it reports another lint on `spawn`, allow that one instead, and
note it in the ledger.

`main.rs`:

```rust
/// How the interface ended.
enum Exit {
    Quit,
    /// The operator asked for a restart (feedback 2 spec O4).
    Restart,
}
```

Changes in `run`:
- the signature becomes `-> Result<Exit, Box<dyn std::error::Error>>`;
- add `.with_started_config(config.clone())` to the `AppUi::new(…)` builder chain;
- right after the builder chain, before `app` is moved, add `let restart = app.restart_flag();`;
- the tail becomes:

```rust
    // Final save with the current positions, then stop the audio.
    drop(remote);
    services.shutdown();
    drop(handle);
    result.map_err(|e| e.to_string())?;
    Ok(if restart.load(std::sync::atomic::Ordering::Acquire) {
        Exit::Restart
    } else {
        Exit::Quit
    })
```

In `main`, replace the `match result { … }` with:

```rust
    match result {
        Ok(Exit::Quit) => {
            tracing::info!("stopped");
            ExitCode::SUCCESS
        }
        Ok(Exit::Restart) => {
            // The lock is released above: the new process can take it.
            match fp_app::restart::relaunch() {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => {
                    tracing::error!(error = %e, "could not start again");
                    let i18n = I18n::new(None);
                    eprintln!("fauste-player: {}", i18n.tr("restart-failed"));
                    let _ = rfd::MessageDialog::new()
                        .set_title("Fauste Player")
                        .set_description(i18n.tr("restart-failed"))
                        .set_level(rfd::MessageLevel::Error)
                        .show();
                    ExitCode::FAILURE
                }
            }
        }
        Err(e) => {
            tracing::error!(error = %e, "could not start");
            eprintln!("fauste-player: {e}");
            ExitCode::FAILURE
        }
    }
```

`drop(lock)` stays where it is, before the `match`.

- [ ] **Step 4: Run the tests and check that they pass**

Run: `cargo test -p fp-app --test relaunch && cargo build -p fp-app`
Expected: 7 passed; the binary builds.

- [ ] **Step 5: Manual check (Linux, Xvfb as in CLAUDE.md)**

1. Start `target/debug/fauste-player` with a scratch `FAUSTE_HOME`.
2. In Settings → Audio outputs, change the buffer size.
3. Check that the pill shows, then click **Restart now**.
4. Check that a new window opens on the new buffer size, that nothing plays, and that `ps` shows one process.

Record the result in the ledger. Flatpak and AppImage cannot be checked in CI. List them for the maintainer in the PR.

- [ ] **Step 6: Commit**

```bash
git add crates/fp-app/src/restart.rs crates/fp-app/src/lib.rs crates/fp-app/src/main.rs crates/fp-app/locales crates/fp-app/tests/relaunch.rs
git commit -m "feat(app): start again after Restart now

After the normal shutdown (session saved, audio stopped, lock released)
the application starts a new copy of itself: the executable, the
AppImage, or a new Flatpak sandbox.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 10: Docs and verification

**Files:**
- Modify: `docs/user/settings.md`:
  - the window and the section header with Restore defaults: Players (the player count and the language are kept), Meters, Analysis and Keyboard shortcuts have it; Audio outputs, Playlists, Cartwall, MIDI and Remote do not;
  - the Restart pending notice, the pill and Restart now, with the final list of restart-only settings. Say plainly that the pill restarts at once when nothing is on air;
  - line 8 ("apply the next time the application starts") becomes a pointer to Restart now;
  - "No output (silent)" for the Null backend.
- Modify: `docs/user/troubleshooting.md` (lines 8 and 63: "restart" means **Restart now**)
- Modify: `docs/user/getting-started.md` (the top-bar pill, one line)
- Modify: `docs/technical/ui.md`:
  - a "Settings window" section on `window_size`, `labelled_row`/`LABEL_WIDTH`, the fixed section header and `ScrollArea::both`;
  - in "Exit guard", `ExitIntent::Restart` and `begin_restart`.
- Modify: `docs/technical/architecture.md` (the restart path: `restart_flag` → `run` returns `Exit::Restart` → shutdown → lock dropped → `restart::relaunch`; the AppImage and Flatpak cases, with one line saying that these two relaunch paths are not exercised in CI and need a manual check)
- Modify: `README.md` (features: restore defaults, restart pending; keep the wording short)
- Modify: `docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md` §3:
  - O4: the final restart set, without the player count, and why;
  - relaunch with no arguments, and why;
  - O2: Players keeps the player count and the interface language; Cartwall has no button (maintainer's decision), so the sections with one are Players, Meters, Analysis and Shortcuts;
  - the pill runs Restart now: at once when nothing is on air, through the on-air guard otherwise.
- Modify: `docs/superpowers/specs/2026-09-25-fauste-player-design.md` §8.4 (Restore defaults, Restart now, the fixed size)
- Modify: `docs/superpowers/plans/2026-10-01-operator-feedback-2-roadmap.md` (plan 2 status: done)

The PR description repeats that line: the AppImage and Flatpak relaunch need a manual check.

- [ ] **Step 1:** Update the docs listed above. Write plain English, and describe behaviour in its own terms.
- [ ] **Step 2:** Run the full gate: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`. Expected: green.
- [ ] **Step 3:** Commit `docs: describe restore defaults and restart now` with the co-author trailer.
