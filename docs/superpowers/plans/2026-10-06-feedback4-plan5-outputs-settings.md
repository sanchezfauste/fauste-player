# Outputs Settings (Feedback 4, Plan 5) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Settings → Audio outputs gets a **Basic | Advanced** selector (Q12). Advanced gives each routed device its own sample rate and buffer size, which the engine opens the device with. Every routed device shows its DSD mode, with a line that says why DoP or native DSD is not offered (Q3). The DSD silence time gets a control.

**Architecture:**
- **Model first.** `fp-model` gets:
  - `OutputsView` (`ui.outputs_view`), `DeviceOverride` (`outputs.device_overrides`) and the `OutputsConfig` helpers `rate_for`, `buffer_for`, `device_override`, `set_device_rate`, `set_device_buffer`, `routed_devices` and `advanced_in_use`;
  - the override checks in `Config::validate`, and the override in `restart_pending`;
  - a new module `device_offer` with the pure rules for what Settings offers a device: `offered_dsd_modes`, moved there from the UI; `dsd_not_offered`, the Q3 reasons; and `offered_rates` and `offered_buffers`.
- **Engine.** `EngineSettings` carries `device_streams: HashMap<BusKey, DeviceStream>`:
  - `ensure_bus` opens each device at `rate_for(key)` and `buffer_for(key)`;
  - `rate_of` falls back to the device's own rate, and the DSD switch back to PCM uses it as its last candidate;
  - the one-block margins use the bus's own buffer.
- **UI.** `ui/settings.rs` draws the selector and the Basic rows. A new `ui/settings/devices.rs` draws the Advanced device rows: the bit-perfect switch, the rate, the buffer and the DSD mode with its reason. The shared DSD settings follow: the mix and the new silence slider.

**Tech Stack:** Rust (edition 2024), egui/eframe 0.36.2, egui_kittest 0.36.2, serde/serde_json. No new dependency, so `cargo deny check` needs no new entry.

**Spec:** `docs/superpowers/specs/2026-10-06-operator-feedback-4-design.md` §6 (Q12 and Q3), §7 (plan 5) and §9 (global constraints). This plan depends on no other plan. Work on a branch `feat/outputs-settings` from an up-to-date `master`, and reach `master` through its own pull request (CLAUDE.md, Pull requests).

The spec's "Spec lines that change" are already in the specs: the main spec §8.4 (line 469), bit-perfect spec B3 rule 1 (line 42) and B6 (line 85), and feedback 2 spec O25 (lines 516–518). Task 6 only checks them and adds the "As built" note.

## Rulings

Each one is the most conservative reading of the spec. Copy them to the ledger (`.superpowers/sdd/2026-10-06-feedback4-plan5-outputs-settings/progress.md`) when the plan starts.

- Ruling: changing a device's own rate gives `RestartReason::SampleRate`, and changing its own buffer gives `RestartReason::BufferSize`. No new reason and no new locale key — Q12.6 says "like the global values", and the notice already names rate and buffer — the cost if wrong: the notice does not say which device changed.
- Ruling: `offered_dsd_modes` and the new reason function `dsd_not_offered` both live in `fp-model` (`device_offer.rs`). They take a plain `DsdCaps` instead of `fp_backends::DeviceInfo` — §9 says "behaviour lives in `fp-model` (… the DSD reasons)". Q3.4 says "next to `offered_dsd_modes`", so that function moves with them, and `fp-model` gains no dependency — the cost if wrong: one moved function and its test.
- Ruling: `dsd_not_offered` returns at most one reason, the most fundamental, in this order: not exclusive-capable, bit-perfect off, native DSD needs Linux, no native DSD — Q3.2 asks for "a line", and the first reason that applies is the one the operator must act on first — the cost if wrong: an operator who fixes the first reason then learns the next one.
- Ruling: an unplugged device (not in the device list) reads as not exclusive-capable — the UI has no other fact about it, and its bit-perfect switch is already disabled for the same reason — the cost if wrong: the reason line reads "cannot be opened exclusively" for a device that is only unplugged.
- Ruling: a device's own rate and buffer offer only the values in `SAMPLE_RATES` and `BUFFER_SIZES` that the device reports (`DeviceInfo::sample_rates`, `DeviceInfo::buffer_frames`). When the device reports nothing, or is unplugged, they offer all of them, and a value already chosen always stays listed — a device asked to open at a rate it refuses stays silent, which is a real on-air hazard — the cost if wrong: a device that under-reports cannot be given a rate it does take, except in the configuration file.
- Ruling: Q12.7 "switching to Basic never changes the configuration" is read as "it changes `ui.outputs_view` only". The view itself is the persisted value Q12.1 asks for, and `restart_pending` ignores `ui` — the cost if wrong: none for playout, because nothing in `outputs` changes.
- Ruling: Basic shows one line, "Some devices have advanced settings …", when a routed device is bit-perfect or has its own rate or buffer, or when the DSD mix or the DSD silence differs from its default (`OutputsConfig::advanced_in_use`). Otherwise an upgraded installation with bit-perfect devices would open in Basic, the new default, with every active setting out of sight. The spec does not forbid the line, and Q12.7 only forbids changing the configuration — the cost if wrong: one line the maintainer may not want.
- Ruling: a device's own value equal to the global value is kept as the device's own; only the **Global (…)** choice removes it — it is what the operator chose, and it must survive a later change of the global value — the cost if wrong: an entry that looks redundant in `config.json`.
- Ruling: an override entry with neither a rate nor a buffer is removed, silently, by the setters and by `Config::validate` — it means nothing — the cost if wrong: none.
- Ruling: `DeviceOverride` serialises as the spec types it: `{"device": {"backend", "device"}, "sample_rate"?, "buffer_frames"?}`, nested, unlike the flat `dsd_output` — the cost if wrong: one more JSON shape in the persistence docs.
- Ruling: the Advanced rows list the devices that routes name explicitly (`OutputsConfig::routed_devices`, which replaces the UI's `chosen_devices`), each once, in route order. A player with no route (the system default output) gets no device rows. This is how bit-perfect has always worked (bit-perfect spec B6, "the devices the routes use") — the cost if wrong: the default output cannot have its own rate.
- Ruling: the DSD mode box is disabled when PCM is the only mode offered. The reason line says why — the cost if wrong: none, because a box with one choice does nothing.
- Ruling: `ensure_bus` computes the mixer's gain smoothing and declick lengths at the device's own rate, not the global rate. They are lengths in the bus's frames — the cost if wrong: none, because they were equal before this plan.
- Ruling: the section header **Bit-perfect devices** becomes **Per-device settings** (new key `settings-devices`), because it now holds the rate and buffer too. `settings-bit-perfect` and `settings-bit-perfect-none` are removed from both locales, and `settings-devices-none` replaces the second — the cost if wrong: the guide's wording changes (Task 6 updates it).

## Global Constraints

- CLAUDE.md rules 1–10 apply to every task. In particular:
  - English everywhere. UI strings go in `crates/fp-app/locales/en-US/main.ftl` and `es-ES/main.ftl`, always both.
  - Never name other playout or radio-automation products.
- From spec §9, verbatim:
  - "operator values are `Config` fields with defaults, ranges and lenient loading (Q12's overrides and view);"
  - "the real-time path never allocates, locks, logs or panics;"
  - "behaviour lives in `fp-model` (`SetDuration`, `pending_start`, the DSD reasons);"
  - "the UI never blocks (the header read and the file stat run on helper threads);"
  - "bad data never crashes (a header without a duration, a file that changes while it is checked);"
  - "nothing goes on air by itself (a pending start only changes where Play starts)."
- Ranges: a device's own rate is in 8 000…768 000 Hz and its own buffer in 16…16 384 frames, the same as `outputs.sample_rate` and `outputs.buffer_frames`. An out-of-range value is dropped with a log line (a `ConfigWarning`), and the device falls back to the global value (Q12.4).
- Defaults: `ui.outputs_view` is `Basic`, `outputs.device_overrides` is empty, there is no bit-perfect device, and DSD is converted to PCM (Q12.1, Q12.7).
- `unwrap`, `expect`, `panic` and indexing are denied outside tests (`fp-engine` denies `clippy::indexing_slicing`).
- Every commit runs `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace` first, and commits only if all of it passes.

## Review Focus

These conditions are implied by the spec but no spec test covers them. Each line names the task that pins it.

1. **An installation that already has a bit-perfect device opens in Basic**, the new default. The operator must still be told that advanced settings are active. Task 1 tests `advanced_in_use`; Task 4 tests the Basic line (`the_basic_view_says_when_advanced_settings_are_in_use`).
2. **A device given a rate it cannot open goes silent.** A 44.1/48 kHz-only card set to 96 kHz must not be offered that rate. Task 2 tests `offered_rates` and `offered_buffers`, and Task 7 checks it on the real card.
3. **A device's own rate equal to the global rate, then the global rate changes.** The device must keep the value the operator chose. Task 1 tests `a_value_equal_to_the_global_one_stays_the_devices_own`.
4. **Two routes on one device** (player 1's Main and player 2's Cue on different channel pairs) must give one set of device rows and one override. Task 1 tests `routed_devices_lists_each_device_once_in_route_order`.
5. **A device with its own rate keeps time**, and a bit-perfect device that starts at its own rate still follows the file (B3). Task 3 tests `a_source_on_a_device_with_its_own_rate_keeps_time` and `a_bit_perfect_device_starts_at_its_own_rate_and_still_follows_the_file`.

---

## File Structure

| File | Responsibility | Task |
|---|---|---|
| `crates/fp-model/src/config.rs` | `OutputsView`, `DeviceOverride`, the range constants, the `OutputsConfig` helpers, and validation | 1 |
| `crates/fp-model/src/restart.rs` | the overrides in `restart_pending` | 1 |
| `crates/fp-model/src/device_offer.rs` (new) | `DsdCaps`, `DsdNotOffered`, `offered_dsd_modes`, `dsd_not_offered`, `offered_rates`, `offered_buffers` | 2 |
| `crates/fp-model/src/lib.rs` | re-exports | 1, 2 |
| `crates/fp-model/tests/device_overrides.rs` (new) | Q12 model tests | 1 |
| `crates/fp-model/tests/restart.rs` | Q12.6 test | 1 |
| `crates/fp-model/tests/device_offer.rs` (new) | Q3 reasons, offered modes, rates and buffers | 2 |
| `crates/fp-store/src/lenient.rs` | lenient-loading test | 1 |
| `crates/fp-engine/src/engine.rs`, `crates/fp-engine/src/engine/dsd.rs` | `DeviceStream`, `rate_for`, `buffer_for`, `ensure_bus`, `rate_of`, `buffer_of` | 3 |
| `crates/fp-engine/tests/device_overrides.rs` (new), `crates/fp-engine/tests/dsd_output.rs` | engine tests | 3 |
| `crates/fp-app/src/ui/settings.rs` | the selector, the Basic rows, the `caption` and `note` helpers | 2, 4 |
| `crates/fp-app/src/ui/settings/devices.rs` (new) | the Advanced device rows and the DSD settings | 4, 5 |
| `crates/fp-app/tests/settings.rs`, `crates/fp-app/tests/dsd_ui.rs` | kittests | 4, 5 |
| `crates/fp-app/locales/en-US/main.ftl`, `crates/fp-app/locales/es-ES/main.ftl` | strings | 4, 5 |
| `docs/…`, `README.md` | documentation | 6, 7 |

---

### Task 1: Device overrides and the outputs view (model and store)

**Suggested executor:** `opus`. This task adds config fields, validation and restart rules, and their invariants interlock.

**Files:**
- Modify: `crates/fp-model/src/config.rs`:
  - `OutputsConfig` (L319–345), its `Default` (L355–369) and `impl OutputsConfig` (L371–388);
  - `UiConfig` (L389–404) and its `Default` (L406–417);
  - `Config::validate`, at the outputs block (L721–757).
- Modify: `crates/fp-model/src/restart.rs` (`RestartReason` L13–24, `restart_pending` L40–76).
- Modify: `crates/fp-model/src/lib.rs` (`pub use config::{…}`, L44–47).
- Modify: `crates/fp-store/src/lenient.rs` (the test module, after `bad_dsd_values_fall_back_one_by_one`, L195–211).
- Test: `crates/fp-model/tests/device_overrides.rs` (create), `crates/fp-model/tests/restart.rs` (append).
- Docs and locales: none in this task (Task 6 writes the docs; no UI string yet).

**Interfaces:**
- Consumes: `OutputDevice`, `Route`, `PlayerRoutes`, `ConfigWarning` and `clamp_to` (`config.rs`).
- Produces (`fp_model::…`):
  - `pub const SAMPLE_RATE_RANGE: RangeInclusive<u32> = 8_000..=768_000;` and `pub const BUFFER_FRAMES_RANGE: RangeInclusive<u32> = 16..=16_384;`
  - `pub enum OutputsView { Basic /* default */, Advanced }`, which is `Copy`, `Eq` and serde.
  - `pub struct DeviceOverride { pub device: OutputDevice, pub sample_rate: Option<u32>, pub buffer_frames: Option<u32> }`.
  - `OutputsConfig::device_overrides: Vec<DeviceOverride>` and `UiConfig::outputs_view: OutputsView`.
  - `OutputsConfig::device_override(&self, device: &OutputDevice) -> Option<&DeviceOverride>`.
  - `OutputsConfig::rate_for(&self, backend: &str, device: &str) -> u32` and `buffer_for(&self, backend: &str, device: &str) -> u32`.
  - `OutputsConfig::set_device_rate(&mut self, device: &OutputDevice, rate: Option<u32>)` and `set_device_buffer(&mut self, device: &OutputDevice, frames: Option<u32>)`.
  - `OutputsConfig::routed_devices(&self) -> Vec<OutputDevice>` and `advanced_in_use(&self) -> bool`.

- [x] **Step 1: Write the failing model tests**

`crates/fp-model/tests/device_overrides.rs` (new file):

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Operator feedback 4, Q12: each device's own rate and buffer, and the
//! Basic | Advanced view of Settings → Audio outputs.

use fp_model::{
    Config, DeviceOverride, DsdMix, OutputDevice, OutputsView, PlayerId, PlayerRoutes, Route,
};

fn dev(name: &str) -> OutputDevice {
    OutputDevice {
        backend: "alsa".into(),
        device: name.into(),
    }
}

fn own(name: &str, rate: Option<u32>, buffer: Option<u32>) -> DeviceOverride {
    DeviceOverride {
        device: dev(name),
        sample_rate: rate,
        buffer_frames: buffer,
    }
}

fn route(name: &str, first_channel: u16) -> Route {
    Route {
        backend: "alsa".into(),
        device: name.into(),
        first_channel,
    }
}

/// Player 1 plays on `dac` and pre-listens on `phones`.
fn routed() -> Config {
    let mut c = Config::default();
    c.outputs.routes = vec![PlayerRoutes {
        player: PlayerId(1),
        main: Some(route("dac", 0)),
        cue: Some(route("phones", 0)),
    }];
    c
}

#[test]
fn the_outputs_view_is_basic_by_default_and_is_kept() {
    assert_eq!(Config::default().ui.outputs_view, OutputsView::Basic);
    let c: Config = serde_json::from_str(r#"{"ui":{}}"#).unwrap();
    assert_eq!(c.ui.outputs_view, OutputsView::Basic);
    let c: Config = serde_json::from_str(r#"{"ui":{"outputs_view":"Advanced"}}"#).unwrap();
    assert_eq!(c.ui.outputs_view, OutputsView::Advanced);
}

#[test]
fn by_default_no_device_has_its_own_values() {
    let mut c = Config::default();
    assert!(c.outputs.device_overrides.is_empty());
    assert_eq!(c.outputs.rate_for("alsa", "dac"), 48_000);
    assert_eq!(c.outputs.buffer_for("alsa", "dac"), 512);
    assert!(c.validate().is_empty());
    let c: Config = serde_json::from_str(r#"{"outputs":{"sample_rate":44100}}"#).unwrap();
    assert!(c.outputs.device_overrides.is_empty(), "an older file has none");
}

#[test]
fn a_device_uses_its_own_rate_and_buffer_and_the_others_the_global_ones() {
    let mut c = Config::default();
    c.outputs.device_overrides = vec![own("dac", Some(96_000), None), own("phones", None, Some(256))];
    assert_eq!(c.outputs.rate_for("alsa", "dac"), 96_000);
    assert_eq!(c.outputs.buffer_for("alsa", "dac"), 512);
    assert_eq!(c.outputs.rate_for("alsa", "phones"), 48_000);
    assert_eq!(c.outputs.buffer_for("alsa", "phones"), 256);
    assert_eq!(c.outputs.rate_for("jack", "dac"), 48_000, "keyed by backend too");
}

#[test]
fn setting_and_clearing_a_devices_own_values() {
    let mut c = Config::default();
    c.outputs.set_device_rate(&dev("dac"), Some(96_000));
    c.outputs.set_device_buffer(&dev("dac"), Some(256));
    assert_eq!(c.outputs.device_overrides, vec![own("dac", Some(96_000), Some(256))]);
    c.outputs.set_device_rate(&dev("dac"), None);
    assert_eq!(c.outputs.device_overrides, vec![own("dac", None, Some(256))]);
    c.outputs.set_device_buffer(&dev("dac"), None);
    assert!(
        c.outputs.device_overrides.is_empty(),
        "an entry with nothing left is removed"
    );
    assert!(c.outputs.device_override(&dev("dac")).is_none());
}

#[test]
fn a_value_equal_to_the_global_one_stays_the_devices_own() {
    let mut c = Config::default();
    c.outputs.set_device_rate(&dev("dac"), Some(48_000));
    c.outputs.sample_rate = 44_100;
    assert_eq!(c.outputs.rate_for("alsa", "dac"), 48_000);
    assert_eq!(c.outputs.rate_for("alsa", "phones"), 44_100);
}

#[test]
fn out_of_range_values_are_dropped_so_the_global_value_applies() {
    let mut c = Config::default();
    c.outputs.device_overrides = vec![
        own("dac", Some(4_000), Some(1024)),
        own("phones", Some(96_000), Some(100_000)),
        own("spdif", Some(1_000_000), None),
    ];
    let warnings = c.validate();
    assert_eq!(
        c.outputs.device_overrides,
        vec![own("dac", None, Some(1024)), own("phones", Some(96_000), None)],
        "spdif is left with nothing and removed"
    );
    assert_eq!(c.outputs.rate_for("alsa", "dac"), 48_000);
    assert_eq!(c.outputs.buffer_for("alsa", "phones"), 512);
    assert_eq!(
        warnings
            .iter()
            .filter(|w| w.field == "outputs.device_overrides")
            .count(),
        3,
        "{warnings:?}"
    );
}

#[test]
fn the_range_ends_are_valid() {
    let mut c = Config::default();
    c.outputs.device_overrides = vec![
        own("low", Some(8_000), Some(16)),
        own("high", Some(768_000), Some(16_384)),
    ];
    let kept = c.outputs.device_overrides.clone();
    assert!(c.validate().is_empty());
    assert_eq!(c.outputs.device_overrides, kept);
}

#[test]
fn a_device_listed_twice_keeps_its_first_values() {
    let mut c = Config::default();
    c.outputs.device_overrides = vec![own("dac", Some(96_000), None), own("dac", Some(44_100), None)];
    let warnings = c.validate();
    assert_eq!(c.outputs.device_overrides, vec![own("dac", Some(96_000), None)]);
    assert!(warnings.iter().any(|w| w.field == "outputs.device_overrides"));
}

#[test]
fn an_empty_entry_is_removed_without_a_warning() {
    let mut c = Config::default();
    c.outputs.device_overrides = vec![own("dac", None, None)];
    assert!(c.validate().is_empty());
    assert!(c.outputs.device_overrides.is_empty());
}

#[test]
fn routed_devices_lists_each_device_once_in_route_order() {
    let mut c = routed();
    c.outputs.routes.push(PlayerRoutes {
        player: PlayerId(2),
        main: Some(route("phones", 2)),
        cue: Some(route("dac", 2)),
    });
    c.outputs.cartwall.main = Some(route("spdif", 0));
    assert_eq!(
        c.outputs.routed_devices(),
        vec![dev("dac"), dev("phones"), dev("spdif")]
    );
}

#[test]
fn advanced_settings_are_in_use_only_where_they_apply() {
    let base = routed();
    assert!(!base.outputs.advanced_in_use(), "the defaults");

    let mut c = base.clone();
    c.outputs.bit_perfect.push(dev("dac"));
    assert!(c.outputs.advanced_in_use(), "a routed bit-perfect device");

    let mut c = base.clone();
    c.outputs.set_device_buffer(&dev("phones"), Some(256));
    assert!(c.outputs.advanced_in_use(), "a routed device's own buffer");

    let mut c = base.clone();
    c.outputs.set_device_rate(&dev("gone"), Some(96_000));
    c.outputs.bit_perfect.push(dev("gone"));
    assert!(
        !c.outputs.advanced_in_use(),
        "a device no route uses is not shown, so it does not count"
    );

    let mut c = base.clone();
    c.outputs.dsd_mix = DsdMix::HoldOthers;
    assert!(c.outputs.advanced_in_use(), "the DSD mix");

    let mut c = base;
    c.outputs.dsd_silence_ms = 500.0;
    assert!(c.outputs.advanced_in_use(), "the DSD silence");
}
```

Append to `crates/fp-model/tests/restart.rs`:

```rust
#[test]
fn a_devices_own_rate_or_buffer_waits_for_a_restart() {
    let dac = OutputDevice {
        backend: "alsa".into(),
        device: "dac".into(),
    };
    let started = Config::default();
    let mut c = started.clone();
    c.outputs.set_device_rate(&dac, Some(96_000));
    assert_eq!(restart_pending(&started, &c), vec![RestartReason::SampleRate]);
    let mut c = started.clone();
    c.outputs.set_device_buffer(&dac, Some(1024));
    assert_eq!(restart_pending(&started, &c), vec![RestartReason::BufferSize]);
}

#[test]
fn reordered_overrides_and_the_outputs_view_need_no_restart() {
    let dev = |name: &str| OutputDevice {
        backend: "alsa".into(),
        device: name.into(),
    };
    let mut started = Config::default();
    started.outputs.set_device_rate(&dev("dac"), Some(96_000));
    started.outputs.set_device_buffer(&dev("phones"), Some(256));
    let mut c = started.clone();
    c.outputs.device_overrides.reverse();
    c.ui.outputs_view = fp_model::OutputsView::Advanced;
    assert!(restart_pending(&started, &c).is_empty());
}
```

- [x] **Step 2: Write the failing lenient-loading test**

In `crates/fp-store/src/lenient.rs`, add inside `mod tests`, after `bad_dsd_values_fall_back_one_by_one`:

```rust
    #[test]
    fn device_overrides_and_the_outputs_view_load_leniently() {
        let user: serde_json::Value = serde_json::from_str(
            r#"{"ui":{"outputs_view":"Expert"},
                "outputs":{"device_overrides":[
                    {"device":{"backend":"alsa","device":"hw:0"},"sample_rate":96000},
                    {"device":{"backend":"alsa","device":"hw:1"},"sample_rate":"fast"},
                    {"device":{"backend":"alsa","device":"hw:2"},"buffer_frames":1024}]}}"#,
        )
        .unwrap();
        let mut warnings = Vec::new();
        let mut c = config_from_value(&user, &mut warnings);
        assert_eq!(c.ui.outputs_view, fp_model::OutputsView::Basic);
        assert_eq!(c.outputs.device_overrides.len(), 2, "the valid elements are kept");
        assert_eq!(c.outputs.rate_for("alsa", "hw:0"), 96_000);
        assert_eq!(c.outputs.buffer_for("alsa", "hw:0"), 512);
        assert_eq!(c.outputs.buffer_for("alsa", "hw:2"), 1024);
        assert_eq!(warnings.len(), 2, "{warnings:?}");
        assert!(c.validate().is_empty());
    }
```

- [x] **Step 3: Run the tests to verify they fail**

Run: `cargo test -p fp-model --test device_overrides`
Expected: FAIL to compile with "unresolved imports `fp_model::DeviceOverride`, `fp_model::OutputsView`".

Run: `cargo test -p fp-model --test restart a_devices_own_rate_or_buffer_waits_for_a_restart`
Expected: FAIL to compile with "no method named `set_device_rate` found for struct `OutputsConfig`".

Run: `cargo test -p fp-store --lib device_overrides_and_the_outputs_view_load_leniently`
Expected: FAIL to compile with "could not find `OutputsView` in `fp_model`".

- [x] **Step 4: Add the types and the fields**

In `crates/fp-model/src/config.rs`, add these right after `pub struct PlayerRoutes` (after L317):

```rust
/// Valid `outputs.sample_rate`, also for a device's own rate (Hz).
pub const SAMPLE_RATE_RANGE: std::ops::RangeInclusive<u32> = 8_000..=768_000;
/// Valid `outputs.buffer_frames`, also for a device's own buffer (frames).
pub const BUFFER_FRAMES_RANGE: std::ops::RangeInclusive<u32> = 16..=16_384;

/// One device's own stream settings (operator feedback 4, Q12.4); `None`
/// uses the global `outputs.sample_rate` or `outputs.buffer_frames`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceOverride {
    pub device: OutputDevice,
    #[serde(default)]
    pub sample_rate: Option<u32>,
    #[serde(default)]
    pub buffer_frames: Option<u32>,
}

/// What Settings → Audio outputs shows (operator feedback 4, Q12.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum OutputsView {
    /// The audio system, the sample rate, the buffer size and the routes.
    #[default]
    Basic,
    /// Also each routed device's own rate and buffer, bit-perfect switch and
    /// DSD mode, and the DSD settings.
    Advanced,
}
```

In `OutputsConfig`, after `dsd_silence_ms` (L343), add:

```rust
    /// Devices with their own rate or buffer (operator feedback 4, Q12.4),
    /// keyed by backend and device like `bit_perfect`.
    #[serde(default)]
    pub device_overrides: Vec<DeviceOverride>,
```

In `impl Default for OutputsConfig`, after `dsd_silence_ms: crate::dsd::DEFAULT_DSD_SILENCE_MS,`, add `device_overrides: Vec::new(),`.

In `UiConfig`, after `table_columns`, add:

```rust
    /// Basic or Advanced view of Settings → Audio outputs (operator
    /// feedback 4, Q12.1). It only hides rows; it changes no output.
    pub outputs_view: OutputsView,
```

In `impl Default for UiConfig`, add `outputs_view: OutputsView::Basic,`.

- [x] **Step 5: Add the helpers**

In `impl OutputsConfig` (after `dsd_output_for`), add:

```rust
    /// `device`'s own rate and buffer, if it has any.
    pub fn device_override(&self, device: &OutputDevice) -> Option<&DeviceOverride> {
        self.device_overrides.iter().find(|o| &o.device == device)
    }

    fn own(&self, backend: &str, device: &str) -> Option<&DeviceOverride> {
        self.device_overrides
            .iter()
            .find(|o| o.device.backend == backend && o.device.device == device)
    }

    /// The rate a device opens at: its own, else `sample_rate`.
    pub fn rate_for(&self, backend: &str, device: &str) -> u32 {
        self.own(backend, device)
            .and_then(|o| o.sample_rate)
            .unwrap_or(self.sample_rate)
    }

    /// The buffer a device opens with: its own, else `buffer_frames`.
    pub fn buffer_for(&self, backend: &str, device: &str) -> u32 {
        self.own(backend, device)
            .and_then(|o| o.buffer_frames)
            .unwrap_or(self.buffer_frames)
    }

    /// Sets (`Some`) or clears (`None`) `device`'s own rate.
    pub fn set_device_rate(&mut self, device: &OutputDevice, rate: Option<u32>) {
        self.edit_override(device, |o| o.sample_rate = rate);
    }

    /// Sets (`Some`) or clears (`None`) `device`'s own buffer.
    pub fn set_device_buffer(&mut self, device: &OutputDevice, frames: Option<u32>) {
        self.edit_override(device, |o| o.buffer_frames = frames);
    }

    fn edit_override(&mut self, device: &OutputDevice, edit: impl FnOnce(&mut DeviceOverride)) {
        if self.device_override(device).is_none() {
            self.device_overrides.push(DeviceOverride {
                device: device.clone(),
                sample_rate: None,
                buffer_frames: None,
            });
        }
        if let Some(o) = self.device_overrides.iter_mut().find(|o| &o.device == device) {
            edit(o);
        }
        self.device_overrides
            .retain(|o| o.sample_rate.is_some() || o.buffer_frames.is_some());
    }

    /// The devices routes name explicitly (players, then the cartwall), each
    /// once, in route order: the devices Settings gives their own rows.
    pub fn routed_devices(&self) -> Vec<OutputDevice> {
        let mut devices: Vec<OutputDevice> = Vec::new();
        let routes = self
            .routes
            .iter()
            .flat_map(|r| r.main.iter().chain(r.cue.iter()))
            .chain(self.cartwall.main.iter())
            .chain(self.cartwall.cue.iter());
        for route in routes {
            let device = OutputDevice {
                backend: route.backend.clone(),
                device: route.device.clone(),
            };
            if !devices.contains(&device) {
                devices.push(device);
            }
        }
        devices
    }

    /// Whether the Advanced view holds anything that applies (operator
    /// feedback 4, Q12): a routed device that is bit-perfect or has its own
    /// rate or buffer, or a DSD setting away from its default.
    pub fn advanced_in_use(&self) -> bool {
        let defaults = Self::default();
        self.dsd_mix != defaults.dsd_mix
            || self.dsd_silence_ms.to_bits() != defaults.dsd_silence_ms.to_bits()
            || self.routed_devices().iter().any(|d| {
                self.bit_perfect.contains(d) || self.device_override(d).is_some()
            })
    }
```

- [x] **Step 6: Validate the overrides**

In `Config::validate`, change the two global clamps (L723–736) to use the constants:

```rust
        clamp_to(
            &mut o.sample_rate,
            *SAMPLE_RATE_RANGE.start(),
            *SAMPLE_RATE_RANGE.end(),
            "outputs.sample_rate",
            &mut w,
        );
        clamp_to(
            &mut o.buffer_frames,
            *BUFFER_FRAMES_RANGE.start(),
            *BUFFER_FRAMES_RANGE.end(),
            "outputs.buffer_frames",
            &mut w,
        );
```

After the `dsd_output` duplicate check (after L757, before `let t = &mut self.tuning;`), add:

```rust
        let mut seen: Vec<OutputDevice> = Vec::new();
        let before = o.device_overrides.len();
        o.device_overrides.retain(|d| {
            let first = !seen.contains(&d.device);
            seen.push(d.device.clone());
            first
        });
        if o.device_overrides.len() != before {
            w.push(ConfigWarning {
                field: "outputs.device_overrides",
                message: "a device was listed more than once; keeping its first values".to_owned(),
            });
        }
        for d in &mut o.device_overrides {
            if let Some(rate) = d.sample_rate
                && !SAMPLE_RATE_RANGE.contains(&rate)
            {
                w.push(ConfigWarning {
                    field: "outputs.device_overrides",
                    message: format!(
                        "{rate} Hz for {} is outside {}..={}; using outputs.sample_rate",
                        d.device.device,
                        SAMPLE_RATE_RANGE.start(),
                        SAMPLE_RATE_RANGE.end()
                    ),
                });
                d.sample_rate = None;
            }
            if let Some(frames) = d.buffer_frames
                && !BUFFER_FRAMES_RANGE.contains(&frames)
            {
                w.push(ConfigWarning {
                    field: "outputs.device_overrides",
                    message: format!(
                        "{frames} frames for {} is outside {}..={}; using outputs.buffer_frames",
                        d.device.device,
                        BUFFER_FRAMES_RANGE.start(),
                        BUFFER_FRAMES_RANGE.end()
                    ),
                });
                d.buffer_frames = None;
            }
        }
        // An entry with nothing left means nothing.
        o.device_overrides
            .retain(|d| d.sample_rate.is_some() || d.buffer_frames.is_some());
```

- [x] **Step 7: The overrides in `restart_pending`**

In `crates/fp-model/src/restart.rs`:
- change the import to `use crate::config::{Config, DeviceOverride, OutputsConfig, Route};`;
- change the doc comments of `SampleRate` and `BufferSize`:

```rust
    /// The global sample rate, or a device's own (operator feedback 4, Q12.6).
    SampleRate,
    /// The global buffer size, or a device's own.
    BufferSize,
```

Add after `player_routes`:

```rust
/// One setting's per-device values, by device, without devices that use
/// the global value.
fn own_values(
    outputs: &OutputsConfig,
    value: fn(&DeviceOverride) -> Option<u32>,
) -> BTreeMap<(&str, &str), u32> {
    outputs
        .device_overrides
        .iter()
        .filter_map(|o| value(o).map(|v| ((o.device.backend.as_str(), o.device.device.as_str()), v)))
        .collect()
}
```

Replace the two checks at L46–51 with:

```rust
    if a.sample_rate != b.sample_rate
        || own_values(a, |o| o.sample_rate) != own_values(b, |o| o.sample_rate)
    {
        reasons.push(RestartReason::SampleRate);
    }
    if a.buffer_frames != b.buffer_frames
        || own_values(a, |o| o.buffer_frames) != own_values(b, |o| o.buffer_frames)
    {
        reasons.push(RestartReason::BufferSize);
    }
```

- [x] **Step 8: Re-export**

In `crates/fp-model/src/lib.rs`, replace the `pub use config::{…}` block (L44–47) with:

```rust
pub use config::{
    AnalysisSettings, BUFFER_FRAMES_RANGE, CartwallConfig, CartwallRoutes, Config, ConfigWarning,
    DeviceOverride, Limits, LoudnessReadout, MeterBallistics, MeterConfig, MeterSettings,
    OutputDevice, OutputsConfig, OutputsView, PlayerRoutes, PlayersConfig, Route,
    SAMPLE_RATE_RANGE, Tuning, UiConfig,
};
```

(`cargo fmt` fixes the order if rustfmt sorts it differently.)

- [x] **Step 9: Run the tests to verify they pass**

Run: `cargo test -p fp-model --test device_overrides && cargo test -p fp-model --test restart && cargo test -p fp-store --lib lenient && cargo test -p fp-model --lib config`
Expected: PASS, including the existing `defaults_are_valid` and the restart tests.

- [x] **Step 10: Commit**

```bash
git add crates/fp-model/src/config.rs crates/fp-model/src/restart.rs crates/fp-model/src/lib.rs \
  crates/fp-model/tests/device_overrides.rs crates/fp-model/tests/restart.rs crates/fp-store/src/lenient.rs
if cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings \
   && cargo test --workspace; then
git commit -m "feat(model): give each output device its own rate and buffer

Operator feedback 4, Q12: the rate and the buffer were global, while
different devices want different values. outputs.device_overrides holds
them per device, validated with the global ranges; ui.outputs_view keeps
the Basic | Advanced choice of Settings.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
fi
```

---

### Task 2: What Settings offers a device: DSD modes, reasons, rates and buffers (model)

**Suggested executor:** `opus`. This task adds pure model rules and moves a function out of the UI.

**Files:**
- Create: `crates/fp-model/src/device_offer.rs`
- Modify: `crates/fp-model/src/lib.rs` (add `pub mod device_offer;` after `pub mod config;`, and the re-export).
- Modify: `crates/fp-app/src/ui/settings.rs`:
  - remove `offered_dsd_modes` (L1013–1033) and its unit test `only_the_modes_the_device_can_take_are_offered` (L1895–1925), together with the `info` helper (L1881–1893) once nothing uses it;
  - change the one call in `bit_perfect()` (L957).
- Test: `crates/fp-model/tests/device_offer.rs` (create).
- Docs and locales: none in this task (the reason strings arrive with their UI in Task 5).

**Interfaces:**
- Consumes: `fp_model::DsdOutput`.
- Produces (`fp_model::…`):
  - `pub struct DsdCaps { pub bit_perfect: bool, pub exclusive_capable: bool, pub native_dsd: bool, pub linux: bool }`, which is `Copy`.
  - `pub enum DsdNotOffered { NotExclusive, BitPerfectOff, NativeNeedsLinux, NoNativeDsd }`, which is `Copy` and `Eq`.
  - `pub fn offered_dsd_modes(caps: DsdCaps, configured: DsdOutput) -> Vec<DsdOutput>`.
  - `pub fn dsd_not_offered(caps: DsdCaps) -> Option<DsdNotOffered>`.
  - `pub fn offered_rates(reported: &[(u32, u32)], candidates: &[u32], current: Option<u32>) -> Vec<u32>`.
  - `pub fn offered_buffers(reported: Option<(u32, u32)>, candidates: &[u32], current: Option<u32>) -> Vec<u32>`.

- [x] **Step 1: Write the failing tests**

`crates/fp-model/tests/device_offer.rs` (new file):

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Operator feedback 4, Q3 and Q12: what Settings offers a routed device.

use fp_model::DsdOutput::{Dop, Native, Pcm};
use fp_model::{
    DsdCaps, DsdNotOffered, dsd_not_offered, offered_buffers, offered_dsd_modes, offered_rates,
};

/// A bit-perfect, exclusive-capable Linux device that takes native DSD.
const FULL: DsdCaps = DsdCaps {
    bit_perfect: true,
    exclusive_capable: true,
    native_dsd: true,
    linux: true,
};

#[test]
fn only_the_modes_the_device_can_take_are_offered() {
    assert_eq!(offered_dsd_modes(FULL, Pcm), vec![Pcm, Dop, Native]);
    let windows = DsdCaps { linux: false, ..FULL };
    assert_eq!(offered_dsd_modes(windows, Pcm), vec![Pcm, Dop]);
    let no_native = DsdCaps { native_dsd: false, ..FULL };
    assert_eq!(offered_dsd_modes(no_native, Pcm), vec![Pcm, Dop]);
    let shared = DsdCaps {
        exclusive_capable: false,
        native_dsd: false,
        ..FULL
    };
    assert_eq!(offered_dsd_modes(shared, Pcm), vec![Pcm]);
    let off = DsdCaps { bit_perfect: false, ..FULL };
    assert_eq!(offered_dsd_modes(off, Pcm), vec![Pcm], "bit-perfect off");
    let unplugged = DsdCaps {
        exclusive_capable: false,
        native_dsd: false,
        ..FULL
    };
    assert_eq!(
        offered_dsd_modes(unplugged, Native),
        vec![Pcm, Native],
        "a configured mode stays listed so it can be changed back"
    );
}

#[test]
fn every_mode_offered_needs_no_reason() {
    assert_eq!(dsd_not_offered(FULL), None);
}

#[test]
fn a_device_that_cannot_be_exclusive_says_so() {
    let caps = DsdCaps {
        bit_perfect: false,
        exclusive_capable: false,
        native_dsd: false,
        ..FULL
    };
    assert_eq!(dsd_not_offered(caps), Some(DsdNotOffered::NotExclusive));
}

#[test]
fn a_device_with_bit_perfect_off_says_so() {
    let caps = DsdCaps { bit_perfect: false, ..FULL };
    assert_eq!(dsd_not_offered(caps), Some(DsdNotOffered::BitPerfectOff));
}

#[test]
fn native_dsd_off_linux_says_it_needs_linux() {
    let caps = DsdCaps { linux: false, ..FULL };
    assert_eq!(dsd_not_offered(caps), Some(DsdNotOffered::NativeNeedsLinux));
}

#[test]
fn a_device_without_native_dsd_says_so() {
    let caps = DsdCaps { native_dsd: false, ..FULL };
    assert_eq!(dsd_not_offered(caps), Some(DsdNotOffered::NoNativeDsd));
}

const RATES: [u32; 6] = [44_100, 48_000, 88_200, 96_000, 176_400, 192_000];
const BUFFERS: [u32; 7] = [64, 128, 256, 512, 1024, 2048, 4096];

#[test]
fn only_the_rates_the_device_reports_are_offered() {
    assert_eq!(offered_rates(&[(44_100, 48_000)], &RATES, None), vec![44_100, 48_000]);
    assert_eq!(
        offered_rates(&[(44_100, 44_100), (96_000, 192_000)], &RATES, None),
        vec![44_100, 96_000, 176_400, 192_000]
    );
    assert_eq!(offered_rates(&[], &RATES, None), RATES.to_vec(), "nothing reported");
    assert_eq!(
        offered_rates(&[(44_100, 48_000)], &RATES, Some(96_000)),
        vec![44_100, 48_000, 96_000],
        "the chosen rate stays listed"
    );
}

#[test]
fn only_the_buffers_the_device_reports_are_offered() {
    assert_eq!(
        offered_buffers(Some((128, 1024)), &BUFFERS, None),
        vec![128, 256, 512, 1024]
    );
    assert_eq!(offered_buffers(None, &BUFFERS, None), BUFFERS.to_vec());
    assert_eq!(
        offered_buffers(Some((128, 1024)), &BUFFERS, Some(4096)),
        vec![128, 256, 512, 1024, 4096]
    );
}
```

- [x] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-model --test device_offer`
Expected: FAIL to compile with "unresolved imports `fp_model::DsdCaps`, …".

- [x] **Step 3: Write the module**

`crates/fp-model/src/device_offer.rs` (new file):

```rust
//! What Settings → Audio outputs offers a routed device (operator feedback
//! 4, Q3 and Q12): the DSD modes it can take and why the others are not
//! offered, and the rates and buffers it reports.

use crate::dsd::DsdOutput;

/// What Settings knows about a routed device when it offers DSD modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DsdCaps {
    /// The device is in `outputs.bit_perfect`.
    pub bit_perfect: bool,
    /// The device can be opened exclusively; `false` when it is not plugged in.
    pub exclusive_capable: bool,
    /// The device reports a DSD sample format.
    pub native_dsd: bool,
    /// The application runs on Linux (native DSD goes through ALSA).
    pub linux: bool,
}

/// Why DoP or native DSD is not offered (Q3.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DsdNotOffered {
    /// Neither: the device cannot be opened exclusively.
    NotExclusive,
    /// Neither: the device is not bit-perfect.
    BitPerfectOff,
    /// Native DSD: not on this system.
    NativeNeedsLinux,
    /// Native DSD: the device reports no DSD format.
    NoNativeDsd,
}

/// The DSD modes a device can be given: PCM always; DoP when it is
/// bit-perfect and exclusive-capable; native DSD when it is bit-perfect,
/// on Linux, and reports a DSD format. The configured mode stays listed so
/// it can be changed back.
pub fn offered_dsd_modes(caps: DsdCaps, configured: DsdOutput) -> Vec<DsdOutput> {
    let mut modes = vec![DsdOutput::Pcm];
    if caps.bit_perfect && caps.exclusive_capable {
        modes.push(DsdOutput::Dop);
    }
    if caps.bit_perfect && caps.linux && caps.native_dsd {
        modes.push(DsdOutput::Native);
    }
    if !modes.contains(&configured) {
        modes.push(configured);
        modes.sort_by_key(|m| match m {
            DsdOutput::Pcm => 0,
            DsdOutput::Dop => 1,
            DsdOutput::Native => 2,
        });
    }
    modes
}

/// Why a mode is missing from `offered_dsd_modes`, the most fundamental
/// reason first; `None` when every mode is offered.
pub fn dsd_not_offered(caps: DsdCaps) -> Option<DsdNotOffered> {
    if !caps.exclusive_capable {
        Some(DsdNotOffered::NotExclusive)
    } else if !caps.bit_perfect {
        Some(DsdNotOffered::BitPerfectOff)
    } else if !caps.linux {
        Some(DsdNotOffered::NativeNeedsLinux)
    } else if !caps.native_dsd {
        Some(DsdNotOffered::NoNativeDsd)
    } else {
        None
    }
}

/// The `candidates` within the device's reported rate ranges (all of them
/// when it reports none), with `current` kept, in ascending order.
pub fn offered_rates(reported: &[(u32, u32)], candidates: &[u32], current: Option<u32>) -> Vec<u32> {
    let fits = |r: u32| reported.is_empty() || reported.iter().any(|(lo, hi)| (*lo..=*hi).contains(&r));
    with_current(candidates.iter().copied().filter(|r| fits(*r)).collect(), current)
}

/// The `candidates` within the device's reported buffer range (all of them
/// when it reports none), with `current` kept, in ascending order.
pub fn offered_buffers(reported: Option<(u32, u32)>, candidates: &[u32], current: Option<u32>) -> Vec<u32> {
    let fits = |b: u32| reported.is_none_or(|(lo, hi)| (lo..=hi).contains(&b));
    with_current(candidates.iter().copied().filter(|b| fits(*b)).collect(), current)
}

fn with_current(mut values: Vec<u32>, current: Option<u32>) -> Vec<u32> {
    if let Some(c) = current
        && !values.contains(&c)
    {
        values.push(c);
        values.sort_unstable();
    }
    values
}
```

In `crates/fp-model/src/lib.rs`, add `pub mod device_offer;` after `pub mod config;`, and add:

```rust
pub use device_offer::{
    DsdCaps, DsdNotOffered, dsd_not_offered, offered_buffers, offered_dsd_modes, offered_rates,
};
```

- [x] **Step 4: Use it in the UI and drop the UI copy**

In `crates/fp-app/src/ui/settings.rs`, inside `bit_perfect()`, replace `for mode in offered_dsd_modes(info, std::env::consts::OS, current) {` (L957) with:

```rust
                    let caps = fp_model::DsdCaps {
                        // Only bit-perfect devices reach this row until Q3.
                        bit_perfect: true,
                        exclusive_capable: info.is_some_and(|d| d.exclusive_capable),
                        native_dsd: info.is_some_and(|d| d.native_dsd),
                        linux: std::env::consts::OS == "linux",
                    };
                    for mode in fp_model::offered_dsd_modes(caps, current) {
```

Then:
- delete the UI's `fn offered_dsd_modes` and its doc comment (L1013–1033);
- delete the unit test `only_the_modes_the_device_can_take_are_offered`;
- delete the `info` helper of the test module, which only that test used.

The test moved to `fp-model` (Step 1).

- [x] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p fp-model --test device_offer && cargo test -p fp-app --lib settings && cargo test -p fp-app --test settings dsd`
Expected: PASS (the existing DSD kittests still pass: the bit-perfect `dac` offers Convert to PCM and DoP).

- [x] **Step 6: Commit**

```bash
git add crates/fp-model/src/device_offer.rs crates/fp-model/src/lib.rs crates/fp-model/tests/device_offer.rs \
  crates/fp-app/src/ui/settings.rs
if cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings \
   && cargo test --workspace; then
git commit -m "feat(model): say why a DSD mode is not offered

Operator feedback 4, Q3: the DSD mode was hard to find, with no word on
why DoP or native DSD were missing. The offered modes and the reason move
to the model as pure rules, with the rates and buffers a device reports.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
fi
```

---

### Task 3: The engine opens each device at its own rate and buffer

**Suggested executor:** `opus`. Bus rates, the DSD switch back to PCM and the real-time margins are involved.

**Files:**
- Modify: `crates/fp-engine/src/engine.rs`:
  - `EngineSettings` (L50–110), and a new `DeviceStream`;
  - `rate_of` (L849–856), a new `buffer_of` after it, and `ensure_bus` (L981–1027);
  - the margin at L1297–1299 (`let margin = u64::from(self.settings.buffer_frames);`).
- Modify: `crates/fp-engine/src/engine/dsd.rs`, at L291–294 (`dsd_source_gone`) and L561–564 (`configured`).
- Test: `crates/fp-engine/tests/device_overrides.rs` (create), `crates/fp-engine/tests/dsd_output.rs` (one test and one rig helper).
- Docs and locales: none in this task (Task 6 writes `docs/technical/audio-engine.md` and `backends.md`).

**Interfaces:**
- Consumes: `OutputsConfig::device_overrides` and `DeviceOverride` (Task 1).
- Produces (`fp_engine::engine::…`):
  - `pub struct DeviceStream { pub sample_rate: Option<u32>, pub buffer_frames: Option<u32> }`, which is `Copy` and `Default`;
  - `EngineSettings::device_streams: HashMap<BusKey, DeviceStream>`;
  - `EngineSettings::rate_for(&self, key: &BusKey) -> u32` and `buffer_for(&self, key: &BusKey) -> u32`.

- [x] **Step 1: Write the failing engine tests**

`crates/fp-engine/tests/device_overrides.rs` (new file):

```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Operator feedback 4, Q12.5: each device opens at its own rate and
//! buffer; a bit-perfect device starts at its own rate and still follows
//! the file (bit-perfect spec B3).

mod support;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use fp_backends::{AudioBackend, OfflineBackend, OfflineDevice};
use fp_engine::engine::{Engine, EngineSettings};
use fp_engine::worker::file_opener;
use fp_model::{
    AudioFormat, Config, EngineAction, EntryId, OutputDevice, PlayerId, PlayerRoutes, Route,
    SourceRequest, TrackId,
};
use support::indexed_wav;

const P: PlayerId = PlayerId(1);
const BLOCK: usize = 480;

struct Rig {
    engine: Engine,
    dac: OfflineDevice,
    phones: OfflineDevice,
    clock: Instant,
    dir: tempfile::TempDir,
}

fn route(device: &str) -> Route {
    Route {
        backend: "offline".into(),
        device: device.into(),
        first_channel: 0,
    }
}

fn dev(device: &str) -> OutputDevice {
    OutputDevice {
        backend: "offline".into(),
        device: device.into(),
    }
}

/// Player 1 plays on `dac` (exclusive-capable) and pre-listens on `phones`;
/// `edit` sets the devices' own values.
fn rig(edit: impl FnOnce(&mut Config)) -> Rig {
    let backend = OfflineBackend::new();
    let dac = backend.add_device("dac", 2);
    dac.set_exclusive_capable(true);
    let phones = backend.add_device("phones", 2);
    let mut config = Config::default();
    config.outputs.backend = Some("offline".into());
    config.outputs.routes = vec![PlayerRoutes {
        player: P,
        main: Some(route("dac")),
        cue: Some(route("phones")),
    }];
    config.tuning.gain_smoothing_ms = 0.0;
    config.tuning.prebuffer_secs = 4.0;
    config.tuning.ready_threshold_ms = 1_500.0;
    edit(&mut config);
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
    let mut engine = Engine::new(backends, EngineSettings::from_config(&config), file_opener());
    let clock = Instant::now();
    engine.execute(EngineAction::AddPlayer { player: P }, clock);
    Rig {
        engine,
        dac,
        phones,
        clock,
        dir: tempfile::tempdir().unwrap(),
    }
}

fn pcm16(rate: u32) -> Option<AudioFormat> {
    Some(AudioFormat {
        sample_rate: rate,
        bits: Some(16),
        channels: 2,
        dsd_rate: None,
    })
}

fn request(path: PathBuf, format: Option<AudioFormat>) -> SourceRequest {
    SourceRequest {
        entry: EntryId(1),
        track: TrackId(1),
        path,
        from_secs: 0.0,
        format,
    }
}

impl Rig {
    fn start(&mut self, request: SourceRequest) {
        self.engine
            .execute(EngineAction::StartCurrent { player: P, request }, self.clock);
        let deadline = Instant::now() + Duration::from_secs(5);
        while self.engine.unsettled_sources() > 0 && Instant::now() < deadline {
            self.engine.tick(self.clock);
            std::thread::sleep(Duration::from_millis(1));
        }
        self.engine.tick(self.clock);
    }

    /// Renders `frames` frames of `dac` in blocks, ticking after each.
    fn run(&mut self, frames: usize) {
        let rate = self.dac.config().unwrap().sample_rate;
        for _ in 0..frames.div_ceil(BLOCK) {
            self.dac.render(BLOCK).unwrap();
            self.clock += Duration::from_secs_f64(BLOCK as f64 / f64::from(rate));
            self.engine.tick(self.clock);
        }
    }
}

#[test]
fn a_device_opens_at_its_own_rate_and_buffer() {
    let r = rig(|c| {
        c.outputs.set_device_rate(&dev("dac"), Some(96_000));
        c.outputs.set_device_buffer(&dev("dac"), Some(256));
    });
    let dac = r.dac.config().unwrap();
    assert_eq!((dac.sample_rate, dac.buffer_frames), (96_000, 256));
    let phones = r.phones.config().unwrap();
    assert_eq!(
        (phones.sample_rate, phones.buffer_frames),
        (48_000, 512),
        "a device without its own values opens with the global ones"
    );
}

#[test]
fn only_the_buffer_can_be_a_devices_own() {
    let r = rig(|c| c.outputs.set_device_buffer(&dev("phones"), Some(1024)));
    let phones = r.phones.config().unwrap();
    assert_eq!((phones.sample_rate, phones.buffer_frames), (48_000, 1024));
}

#[test]
fn a_source_on_a_device_with_its_own_rate_keeps_time() {
    let mut r = rig(|c| c.outputs.set_device_rate(&dev("dac"), Some(96_000)));
    let path = indexed_wav(r.dir.path(), "a.wav", 48_000, 2, 48_000 * 3);
    r.start(request(path, pcm16(48_000)));
    r.run(96_000);
    let position = r.engine.telemetry(P).position_secs.unwrap();
    assert!(
        (position - 1.0).abs() < 0.05,
        "96 000 frames at 96 kHz are one second: {position}"
    );
}

#[test]
fn a_bit_perfect_device_starts_at_its_own_rate_and_still_follows_the_file() {
    let mut r = rig(|c| {
        c.outputs.bit_perfect = vec![dev("dac")];
        c.outputs.set_device_rate(&dev("dac"), Some(96_000));
    });
    assert_eq!(r.dac.config().unwrap().sample_rate, 96_000);
    let path = indexed_wav(r.dir.path(), "a.wav", 44_100, 2, 44_100 * 3);
    r.start(request(path, pcm16(44_100)));
    assert_eq!(r.dac.config().unwrap().sample_rate, 44_100);
}
```

In `crates/fp-engine/tests/dsd_output.rs`:
- make `rig_with` (L50) delegate to a new `rig_edited`, which takes a final `edit: impl FnOnce(&mut Config)` applied just before `EngineSettings::from_config(&config)`, and give `rig_with`'s body to `rig_edited`;

```rust
fn rig_with(
    mode: DsdOutput,
    mix: DsdMix,
    format: SampleFormat,
    device: impl FnOnce(&OfflineDevice),
) -> Rig {
    rig_edited(mode, mix, format, device, |_| {})
}
```

- in `rig_edited`, call `edit(&mut config);` right after `config.tuning.ready_threshold_ms = 1_500.0;`;
- add this test after `native_dsd_ends_on_a_pcm_rate_the_device_takes`:

```rust
#[test]
fn native_dsd_ends_on_the_devices_own_rate() {
    let mut r = rig_edited(
        DsdOutput::Native,
        DsdMix::ConvertToPcm,
        SampleFormat::I24,
        |dac| {
            dac.set_native_dsd(true);
            dac.set_max_pcm_rate(MAX_PCM);
        },
        |c| {
            c.outputs.set_device_rate(
                &OutputDevice {
                    backend: "offline".into(),
                    device: "dac".into(),
                },
                Some(96_000),
            )
        },
    );
    assert_eq!(r.rate(), 96_000, "it opens at its own rate");
    let path = dsd128_file(&r, "short.dsf", DSD128 as usize / 8 / 10);
    r.start(P, request(1, path, dsd128()));
    assert_eq!(r.rate(), WORD_RATE_128);
    r.run_raw(WORD_RATE_128 as usize / 2);
    let config = r.dac.config().expect("the device is never left closed");
    assert_eq!(config.dsd, None, "reopened as PCM");
    assert_eq!(config.sample_rate, 96_000, "its own rate, not the global one");
}
```

- [x] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-engine --test device_overrides`
Expected: FAIL. `a_device_opens_at_its_own_rate_and_buffer` panics with ``left: (48000, 512)`, `right: (96000, 256)``, and `a_bit_perfect_device_starts_at_its_own_rate_and_still_follows_the_file` panics with `left: 48000`, `right: 96000`.

Run: `cargo test -p fp-engine --test dsd_output native_dsd_ends_on_the_devices_own_rate`
Expected: FAIL with ``left: 48000`, `right: 96000`` ("it opens at its own rate").

- [x] **Step 3: Carry the overrides in `EngineSettings`**

In `crates/fp-engine/src/engine.rs`, add before `pub struct EngineSettings`:

```rust
/// A device's own stream settings (operator feedback 4, Q12.5); `None`
/// uses the global value.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DeviceStream {
    pub sample_rate: Option<u32>,
    pub buffer_frames: Option<u32>,
}
```

Add a field to `EngineSettings`, after `bit_perfect`:

```rust
    /// Devices with their own rate or buffer.
    pub device_streams: HashMap<BusKey, DeviceStream>,
```

In `from_config`, after the `bit_perfect` field, add:

```rust
            device_streams: config
                .outputs
                .device_overrides
                .iter()
                .map(|o| {
                    let key = BusKey {
                        backend: o.device.backend.clone(),
                        device: o.device.device.clone(),
                    };
                    let stream = DeviceStream {
                        sample_rate: o.sample_rate,
                        buffer_frames: o.buffer_frames,
                    };
                    (key, stream)
                })
                .collect(),
```

In `impl EngineSettings`, after `from_config`, add:

```rust
    /// The rate `key` opens at: its own, else `sample_rate`.
    pub fn rate_for(&self, key: &BusKey) -> u32 {
        self.device_streams
            .get(key)
            .and_then(|d| d.sample_rate)
            .unwrap_or(self.sample_rate)
    }

    /// The buffer `key` opens with: its own, else `buffer_frames`.
    pub fn buffer_for(&self, key: &BusKey) -> u32 {
        self.device_streams
            .get(key)
            .and_then(|d| d.buffer_frames)
            .unwrap_or(self.buffer_frames)
    }
```

- [x] **Step 4: Open each bus with its own values**

In `ensure_bus` (L981), replace the `StreamConfig` and `MixerConfig` literals with:

```rust
            let rate = self.settings.rate_for(key);
            let config = StreamConfig {
                sample_rate: rate,
                buffer_frames: self.settings.buffer_for(key),
                channels,
                exclusive: self.settings.bit_perfect.contains(key),
                dsd: None,
            };
            // Lengths in this bus's frames.
            let mixer = MixerConfig {
                volume_smoothing_frames: frames_at(rate, t.gain_smoothing_ms).max(1) as u32,
                declick_frames: frames_at(rate, t.declick_ms) as u32,
                max_commands_per_block: t.max_commands_per_block,
            };
```

Replace `rate_of` (L849–856) and add `buffer_of` after it:

```rust
    /// The rate `bus` runs at. Every source on a bus shares it, and it only
    /// changes while nothing is attached (Phase 4 spec B3). A bus not open
    /// yet runs at its device's own rate, else the global one.
    fn rate_of(&self, bus: &BusKey) -> u32 {
        self.buses
            .get(bus)
            .map_or_else(|| self.settings.rate_for(bus), Bus::sample_rate)
            .max(1)
    }

    /// The block size `bus` asks its device for.
    fn buffer_of(&self, bus: &BusKey) -> u32 {
        self.buses
            .get(bus)
            .map_or_else(|| self.settings.buffer_for(bus), |b| b.config().buffer_frames)
    }
```

At L1297–1299, replace `let margin = u64::from(self.settings.buffer_frames);` with `let margin = u64::from(self.buffer_of(&p.bus));`.

In `crates/fp-engine/src/engine/dsd.rs`:
- in `dsd_source_gone` (L291–294), replace `+ u64::from(self.settings.buffer_frames)` with `+ u64::from(self.buffer_of(&p.bus))`;
- at L561–564, replace `sample_rate: self.settings.sample_rate,` with `sample_rate: self.settings.rate_for(bus),`, and update its doc line in `docs/technical/audio-engine.md` in Task 6.

- [x] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p fp-engine --test device_overrides && cargo test -p fp-engine --test dsd_output && cargo test -p fp-engine --test bit_perfect`
Expected: PASS.

- [x] **Step 6: Commit**

```bash
git add crates/fp-engine/src/engine.rs crates/fp-engine/src/engine/dsd.rs \
  crates/fp-engine/tests/device_overrides.rs crates/fp-engine/tests/dsd_output.rs
if cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings \
   && cargo test --workspace; then
git commit -m "feat(engine): open each device at its own rate and buffer

Operator feedback 4, Q12.5: buses are per device, so a device's own rate
and buffer apply to its stream. A bit-perfect device starts at that rate
and still follows the file; leaving DSD falls back to it.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
fi
```

---

### Task 4: The Basic | Advanced selector

**Suggested executor:** `sonnet`. This task adds UI wiring and moves code mechanically.

**Files:**
- Create: `crates/fp-app/src/ui/settings/devices.rs`.
- Modify: `crates/fp-app/src/ui/settings.rs`:
  - add `mod devices;` after `mod columns;` (L21);
  - in `outputs()` (L679–842), the selector and the view switch;
  - remove `chosen_devices` (L843–865) and `bit_perfect()` (L866–1011), which move;
  - add the `caption` and `note` helpers after `row` (L641–644).
- Modify: `crates/fp-app/locales/en-US/main.ftl` and `crates/fp-app/locales/es-ES/main.ftl` (near `settings-bit-perfect`, L211–213).
- Test: `crates/fp-app/tests/settings.rs`:
  - the helpers `outputs` and `outputs_with` (L294–342);
  - three new tests.

**Interfaces:**
- Consumes: `OutputsView`, `OutputsConfig::routed_devices` and `advanced_in_use` (Task 1); `widgets::segmented` and `widgets::Segment` (`ui/widgets.rs` L104–180).
- Produces:
  - `devices::section(ui: &mut Ui, scene: &Scene<'_>, backends: &[BackendChoice])`, the Advanced device rows that Task 5 extends;
  - in `settings.rs`, `pub(super) fn caption(ui: &mut Ui, text: &str)` and `pub(super) fn note(ui: &mut Ui, text: &str, color: Color32)`;
  - in the tests, `outputs_in(view: OutputsView, bit_perfect: Vec<OutputDevice>)`.

- [ ] **Step 1: Write the failing kittests**

In `crates/fp-app/tests/settings.rs`, replace `outputs_with` (L303–342) with:

```rust
fn outputs_with(
    bit_perfect: Vec<fp_model::OutputDevice>,
) -> (
    egui_kittest::Harness<'static, fp_app::ui::app::AppUi>,
    Arc<support::Fake>,
) {
    outputs_in(fp_model::OutputsView::Advanced, bit_perfect)
}

/// Player 1 plays on `dac` (exclusive-capable) and pre-listens on
/// `speakers` (shared); Settings is open on Audio outputs in `view`.
fn outputs_in(
    view: fp_model::OutputsView,
    bit_perfect: Vec<fp_model::OutputDevice>,
) -> (
    egui_kittest::Harness<'static, fp_app::ui::app::AppUi>,
    Arc<support::Fake>,
) {
    let backend = OfflineBackend::new();
    backend.add_device("dac", 2).set_exclusive_capable(true);
    backend.add_device("speakers", 2);
    let mut s = state(1, 1);
    let route = |device: &str| Route {
        backend: "offline".into(),
        device: device.into(),
        first_channel: 0,
    };
    s.config.ui.outputs_view = view;
    s.config.outputs.backend = Some("offline".into());
    s.config.outputs.bit_perfect = bit_perfect;
    s.config.outputs.routes = vec![PlayerRoutes {
        player: s.players[0].id,
        main: Some(route("dac")),
        cue: Some(route("speakers")),
    }];
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
    let (mut h, fake) = harness_with_backends(s, backends);
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Audio outputs")
        .click();
    // Devices are listed by a helper thread; the routes show once they are.
    for _ in 0..200 {
        h.run_steps(1);
        if h.query_all_by_label("Test Main").next().is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    h.run_steps(2);
    (h, fake)
}
```

Move `fn dac()` (L495–500) up, just before `outputs()`, so that the new tests can use it. Then add after `a_listed_device_can_always_be_turned_off`:

```rust
#[test]
fn the_basic_view_hides_the_device_rows() {
    let (h, _) = outputs_in(fp_model::OutputsView::Basic, vec![dac()]);
    assert!(
        h.query_by_role_and_label(Role::RadioButton, "Basic")
            .is_some()
    );
    assert!(h.query_by_value("48000 Hz").is_some(), "the global rate stays");
    assert!(h.query_by_value("512").is_some(), "the global buffer stays");
    assert!(
        h.query_all_by_label("Test Main").next().is_some(),
        "the routes stay"
    );
    assert!(
        h.query_by_role_and_label(Role::CheckBox, "Bit-perfect: dac")
            .is_none()
    );
    assert!(h.query_by_value("Continue the DSD track as PCM").is_none());
}

#[test]
fn switching_views_changes_only_the_view() {
    let (mut h, fake) = outputs_with(vec![dac()]);
    let before = fake.state.load().config.clone();
    assert!(
        h.query_by_role_and_label(Role::CheckBox, "Bit-perfect: dac")
            .is_some()
    );
    h.get_by_role_and_label(Role::RadioButton, "Basic").click();
    h.run_steps(2);
    let basic = fake.state.load().config.clone();
    assert_eq!(basic.ui.outputs_view, fp_model::OutputsView::Basic);
    assert_eq!(basic.outputs, before.outputs, "no output setting changes");
    assert!(fp_model::restart_pending(&before, &basic).is_empty());
    assert!(
        h.query_by_role_and_label(Role::CheckBox, "Bit-perfect: dac")
            .is_none()
    );
    h.get_by_role_and_label(Role::RadioButton, "Advanced")
        .click();
    h.run_steps(2);
    assert_eq!(fake.state.load().config, before);
    assert!(
        h.query_by_role_and_label(Role::CheckBox, "Bit-perfect: dac")
            .is_some()
    );
}

#[test]
fn the_basic_view_says_when_advanced_settings_are_in_use() {
    let note = "Some devices have advanced settings";
    let (h, _) = outputs_in(fp_model::OutputsView::Basic, Vec::new());
    assert!(h.query_by_label_contains(note).is_none());
    let (h, _) = outputs_in(fp_model::OutputsView::Basic, vec![dac()]);
    assert!(h.query_by_label_contains(note).is_some());
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-app --test settings view`
Expected: FAIL to compile with "no field `outputs_view` on type `UiConfig`" only if Task 1 is missing. With Task 1 in place, the three tests fail at run time: `the_basic_view_hides_the_device_rows` with "assertion failed: h.query_by_role_and_label(Role::RadioButton, \"Basic\").is_some()".

- [ ] **Step 3: Add the strings**

In `crates/fp-app/locales/en-US/main.ftl`, replace the two lines `settings-bit-perfect = …` and `settings-bit-perfect-none = …` (L211 and L213) with:

```ftl
settings-outputs-view = Show
settings-outputs-basic = Basic
settings-outputs-advanced = Advanced
settings-outputs-advanced-in-use = Some devices have advanced settings (bit-perfect, DSD, or their own rate or buffer). Choose Advanced to see them.
settings-devices = Per-device settings
settings-devices-none = Choose a device for a player or the cartwall above to give it its own settings.
```

In `crates/fp-app/locales/es-ES/main.ftl`, replace the same two keys (L211 and L213) with:

```ftl
settings-outputs-view = Mostrar
settings-outputs-basic = Básico
settings-outputs-advanced = Avanzado
settings-outputs-advanced-in-use = Algunos dispositivos tienen ajustes avanzados (bit perfect, DSD, o su propia frecuencia o búfer). Elige Avanzado para verlos.
settings-devices = Ajustes por dispositivo
settings-devices-none = Elige arriba un dispositivo para un reproductor o la cartuchera para darle sus propios ajustes.
```

`settings-bit-perfect-hint` stays.

- [ ] **Step 4: Add the helpers and the selector**

In `crates/fp-app/src/ui/settings.rs`:
- add `OutputsView` to the `use fp_model::{…}` list;
- remove `DsdDevice`, `DsdMix` and `DsdOutput` from that list once the code moves (clippy reports what is unused);
- after `row` (L641–644), add:

```rust
/// A small upper-case heading over a group of rows.
pub(super) fn caption(ui: &mut Ui, text: &str) {
    ui.add(
        egui::Label::new(
            RichText::new(text.to_uppercase())
                .font(font(10.0))
                .color(theme::NEUTRAL_500),
        )
        .selectable(false),
    );
}

/// A line of explanation, wrapped to the section's width.
pub(super) fn note(ui: &mut Ui, text: &str, color: Color32) {
    ui.add(
        egui::Label::new(RichText::new(text).font(font(12.0)).color(color))
            .selectable(false)
            .wrap(),
    );
}

/// Basic | Advanced (operator feedback 4, Q12.1). Only `ui.outputs_view`
/// changes: the advanced settings are hidden, never reset (Q12.7).
fn view_selector(ui: &mut Ui, scene: &Scene<'_>, view: OutputsView) {
    let t = scene.i18n;
    let basic = t.tr("settings-outputs-basic");
    let advanced = t.tr("settings-outputs-advanced");
    let views = [OutputsView::Basic, OutputsView::Advanced];
    row(ui, &t.tr("settings-outputs-view"), None, |ui| {
        let selected = usize::from(view == OutputsView::Advanced);
        if let Some(chosen) = widgets::segmented(
            ui,
            egui::Id::new("outputs-view"),
            &[
                widgets::Segment {
                    text: &basic,
                    label: &basic,
                },
                widgets::Segment {
                    text: &advanced,
                    label: &advanced,
                },
            ],
            selected,
        )
        .and_then(|i| views.get(i).copied())
        {
            update(scene, |c| c.ui.outputs_view = chosen);
        }
    });
}
```

In `outputs()`, insert as the first lines after `let config = &scene.state.config;` (L681):

```rust
    let view = config.ui.outputs_view;
    view_selector(ui, scene, view);
```

At the end of `outputs()`, replace `let chosen = chosen_devices(config);` (L817) by nothing (delete the line), and replace `bit_perfect(ui, scene, &backends, &chosen);` (L841) with:

```rust
    match view {
        OutputsView::Advanced => devices::section(ui, scene, &backends),
        OutputsView::Basic => {
            if config.outputs.advanced_in_use() {
                ui.add_space(8.0);
                note(ui, &t.tr("settings-outputs-advanced-in-use"), theme::NEUTRAL_400);
            }
        }
    }
```

Replace the routes caption at L764–772 (`ui.add(egui::Label::new(RichText::new(t.tr("settings-player-routes").to_uppercase()) …))`) with `caption(ui, &t.tr("settings-player-routes"));`.

- [ ] **Step 5: Move the device rows to `devices.rs`**

Create `crates/fp-app/src/ui/settings/devices.rs`:

```rust
//! Settings → Audio outputs, Advanced view (operator feedback 4, Q12, and
//! Phase 4 spec B6): each routed device's bit-perfect switch and DSD mode,
//! then the DSD settings every device shares.

use egui::Ui;
use fp_backends::DeviceInfo;
use fp_model::{DsdDevice, DsdMix, DsdOutput, OutputDevice};

use super::{BackendChoice, caption, note, row, toggle, update};
use crate::ui::app::Scene;
use crate::ui::theme;

/// The device as its backend lists it, and the label the pickers give it
/// (its id when it is not listed: unplugged, or another system).
fn find<'a>(backends: &'a [BackendChoice], device: &OutputDevice) -> (Option<&'a DeviceInfo>, String) {
    let list = backends
        .iter()
        .find(|b| b.id == device.backend)
        .map(|b| b.devices.as_slice())
        .unwrap_or_default();
    let labels = fp_backends::device_labels(list);
    list.iter()
        .zip(labels)
        .find(|(d, _)| d.id.0 == device.device)
        .map_or((None, device.device.clone()), |(d, label)| (Some(d), label))
}

pub(super) fn section(ui: &mut Ui, scene: &Scene<'_>, backends: &[BackendChoice]) {
    let t = scene.i18n;
    ui.add_space(12.0);
    caption(ui, &t.tr("settings-devices"));
    note(ui, &t.tr("settings-bit-perfect-hint"), theme::NEUTRAL_400);
    ui.add_space(4.0);
    let devices = scene.state.config.outputs.routed_devices();
    if devices.is_empty() {
        note(ui, &t.tr("settings-devices-none"), theme::NEUTRAL_500);
    }
    for device in &devices {
        let (info, name) = find(backends, device);
        device_rows(ui, scene, device, info, &name);
    }
    dsd_settings(ui, scene);
}
```

Then move the body of the old `bit_perfect()` loop into `device_rows`. Copy it verbatim from the deleted `settings.rs` L904–978, from `let listed = …` down to the end of the DSD `row(...)`, with these changes:
- the function header is

```rust
fn device_rows(ui: &mut Ui, scene: &Scene<'_>, device: &OutputDevice, info: Option<&DeviceInfo>, name: &str) {
    let t = scene.i18n;
```

- the `backend_devices`/`labels`/`found`/`info`/`name` lookups (old L906–916) are gone, because `find` supplies `info` and `name`;
- `if !listed.contains(device) { continue; }` becomes `if !listed.contains(device) { return; }`.

The shared DSD part becomes `dsd_settings`. Copy it verbatim from the deleted L979–1011, from `ui.add_space(4.0);` through the DSD mix `row(...)`:

```rust
/// The DSD settings every device shares.
fn dsd_settings(ui: &mut Ui, scene: &Scene<'_>) {
    let t = scene.i18n;
    ui.add_space(4.0);
    note(ui, &t.tr("settings-dsd-hint"), theme::NEUTRAL_400);
    // … the DSD mix row, verbatim from the old bit_perfect() …
}
```

Also move `dsd_mode_label` (old L1035–1041) into `devices.rs` unchanged; nothing else uses it. Delete `chosen_devices` (`OutputsConfig::routed_devices` replaces it) and `bit_perfect()` from `settings.rs`.

- [ ] **Step 6: Run the tests to verify they pass**

Run: `cargo test -p fp-app --test settings && cargo test -p fp-app --test i18n`
Expected: PASS. That includes the existing bit-perfect and DSD kittests, which now open in Advanced through `outputs_with`, and `both_locales_define_the_same_keys`.

- [ ] **Step 7: Commit**

```bash
git add crates/fp-app/src/ui/settings.rs crates/fp-app/src/ui/settings/devices.rs \
  crates/fp-app/locales/en-US/main.ftl crates/fp-app/locales/es-ES/main.ftl crates/fp-app/tests/settings.rs
if cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings \
   && cargo test --workspace; then
git commit -m "feat(ui): split Audio outputs into Basic and Advanced

Operator feedback 4, Q12.1-Q12.2, Q12.7: every output setting showed at
once. Basic keeps the audio system, the rate, the buffer and the routes;
Advanced adds the per-device rows. Switching only hides rows, and Basic
says when advanced settings are in use.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
fi
```

---

### Task 5: Each device's rate, buffer and DSD row, and the DSD silence

**Suggested executor:** `sonnet`. This task adds UI rows on top of rules that already exist.

**Files:**
- Modify: `crates/fp-app/src/ui/settings/devices.rs` (`device_rows`, `dsd_settings`).
- Modify: `crates/fp-app/locales/en-US/main.ftl` and `crates/fp-app/locales/es-ES/main.ftl` (after `settings-dsd-mode`).
- Test: `crates/fp-app/tests/settings.rs`:
  - the new tests;
  - `a_bit_perfect_device_offers_its_dsd_modes_and_the_choice_updates_the_config`, updated;
  - `a_device_that_is_not_bit_perfect_has_no_dsd_row`, deleted because Q3.1 reverses it.
- Test: `crates/fp-app/tests/dsd_ui.rs` (new tests and a helper).

**Interfaces:**
- Consumes:
  - from Task 1: `OutputsConfig::{device_override, rate_for, buffer_for, set_device_rate, set_device_buffer}`;
  - from Task 2: `DsdCaps`, `DsdNotOffered`, `offered_dsd_modes`, `dsd_not_offered`, `offered_rates` and `offered_buffers`;
  - in `settings.rs`: `SAMPLE_RATES`, `BUFFER_SIZES`, `labelled_row`, `slider`, `note`.
- Produces: in Advanced, each routed device has:
  - the combo boxes labelled "Sample rate: {device}", "Buffer size: {device}" and "DSD: {device}" (labelled with `labelled_by`, so `get_by_role_and_label(Role::ComboBox, …)` finds them);
  - the "By default, DSD is converted to PCM." hint, and a reason line when `dsd_not_offered` is `Some`.

  Once for all devices, there is a "DSD silence" slider.

- [ ] **Step 1: Write the failing kittests**

In `crates/fp-app/tests/settings.rs`:
- delete `a_device_that_is_not_bit_perfect_has_no_dsd_row`;
- replace the first three lines of `a_bit_perfect_device_offers_its_dsd_modes_and_the_choice_updates_the_config` (the `get_by_value("Convert to PCM")` scroll and click) with:

```rust
    let (mut h, fake) = outputs_with(vec![dac()]);
    h.get_by_role_and_label(Role::ComboBox, "DSD: dac")
        .scroll_to_me();
    h.run_steps(5);
    h.get_by_role_and_label(Role::ComboBox, "DSD: dac").click();
    h.run_steps(2);
```

Then add:

```rust
fn rate_choices(fake: &support::Fake) -> Vec<Vec<fp_model::DeviceOverride>> {
    fake.take_sent()
        .into_iter()
        .filter_map(|c| match c {
            Command::UpdateConfig(config) => Some(config.outputs.device_overrides),
            _ => None,
        })
        .collect()
}

#[test]
fn a_device_can_be_given_its_own_rate_and_back_the_global_one() {
    let (mut h, fake) = outputs_with(Vec::new());
    h.get_by_role_and_label(Role::ComboBox, "Sample rate: dac")
        .scroll_to_me();
    h.run_steps(5);
    // dac and speakers both show it, so query_by_value would find two nodes.
    assert_eq!(
        h.query_all_by_value("Global (48000 Hz)").count(),
        2,
        "dac and speakers both use the global rate"
    );
    h.get_by_role_and_label(Role::ComboBox, "Sample rate: dac")
        .click();
    h.run_steps(2);
    h.get_by_label("96000 Hz").click();
    h.run_steps(2);
    let c = fake.state.load().config.clone();
    assert_eq!(c.outputs.rate_for("offline", "dac"), 96_000);
    assert_eq!(c.outputs.rate_for("offline", "speakers"), 48_000);
    assert_eq!(c.outputs.sample_rate, 48_000, "the global rate is untouched");
    assert_eq!(rate_choices(&fake).len(), 1, "one UpdateConfig");
    h.get_by_role_and_label(Role::ComboBox, "Sample rate: dac")
        .click();
    h.run_steps(2);
    h.get_by_label("Global (48000 Hz)").click();
    h.run_steps(2);
    assert!(fake.state.load().config.outputs.device_overrides.is_empty());
}

#[test]
fn a_device_can_be_given_its_own_buffer() {
    let (mut h, fake) = outputs_with(Vec::new());
    h.get_by_role_and_label(Role::ComboBox, "Buffer size: dac")
        .scroll_to_me();
    h.run_steps(5);
    h.get_by_role_and_label(Role::ComboBox, "Buffer size: dac")
        .click();
    h.run_steps(2);
    h.get_by_label("1024").click();
    h.run_steps(2);
    let c = fake.state.load().config.clone();
    assert_eq!(c.outputs.buffer_for("offline", "dac"), 1024);
    assert_eq!(c.outputs.buffer_for("offline", "speakers"), 512);
    assert!(
        h.query_by_label_contains("Latency: 21.3 ms").is_some(),
        "1024 frames at 48 kHz"
    );
}

#[test]
fn the_dsd_silence_slider_updates_the_config() {
    let (mut h, fake) = outputs_with(Vec::new());
    h.get_by_role_and_label(Role::Slider, "DSD silence")
        .scroll_to_me();
    h.run_steps(5);
    h.get_by_role_and_label(Role::Slider, "DSD silence")
        .focus();
    h.run_steps(1);
    h.key_press(Key::ArrowRight);
    h.run_steps(2);
    assert_eq!(fake.state.load().config.outputs.dsd_silence_ms, 210.0);
}
```

In `crates/fp-app/tests/dsd_ui.rs`, change the imports to:

```rust
use std::sync::Arc;

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable;
use fp_backends::{AudioBackend, OfflineBackend};
use fp_model::{DsdOnAir, OutputDevice, OutputsView, PlayerRoutes, Route, Transport};
use support::{Fake, harness, harness_with_backends, state};
```

and add:

```rust
/// Player 1 plays on `dac` (exclusive-capable, no DSD format); Settings is
/// open on Audio outputs, Advanced.
fn advanced_outputs(bit_perfect: bool) -> egui_kittest::Harness<'static, fp_app::ui::app::AppUi> {
    let backend = OfflineBackend::new();
    backend.add_device("dac", 2).set_exclusive_capable(true);
    let mut s = state(1, 1);
    s.config.ui.outputs_view = OutputsView::Advanced;
    s.config.outputs.backend = Some("offline".into());
    s.config.outputs.routes = vec![PlayerRoutes {
        player: s.players[0].id,
        main: Some(Route {
            backend: "offline".into(),
            device: "dac".into(),
            first_channel: 0,
        }),
        cue: None,
    }];
    if bit_perfect {
        s.config.outputs.bit_perfect = vec![OutputDevice {
            backend: "offline".into(),
            device: "dac".into(),
        }];
    }
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
    let (mut h, _) = harness_with_backends(s, backends);
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Audio outputs")
        .click();
    for _ in 0..200 {
        h.run_steps(1);
        if h.query_by_role_and_label(Role::ComboBox, "DSD: dac")
            .is_some()
        {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    h
}

#[test]
fn a_device_that_is_not_bit_perfect_shows_its_dsd_row_and_why() {
    let h = advanced_outputs(false);
    assert!(
        h.query_by_role_and_label(Role::ComboBox, "DSD: dac")
            .is_some()
    );
    assert!(
        h.query_by_label("DoP and native DSD need this device to be bit-perfect.")
            .is_some()
    );
    assert!(
        h.query_by_label("By default, DSD is converted to PCM.")
            .is_some()
    );
}

#[test]
fn a_bit_perfect_device_without_native_dsd_says_why() {
    let h = advanced_outputs(true);
    let why = if cfg!(target_os = "linux") {
        "This device does not take native DSD: its driver reports no DSD format."
    } else {
        "Native DSD needs Linux."
    };
    assert!(h.query_by_label(why).is_some(), "{why}");
    assert!(
        h.query_by_label("DoP and native DSD need this device to be bit-perfect.")
            .is_none()
    );
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-app --test settings own_ && cargo test -p fp-app --test dsd_ui why`
Expected: FAIL. `a_device_can_be_given_its_own_rate_and_back_the_global_one` panics with "Did not find a node with role ComboBox and label 'Sample rate: dac'"; the `dsd_ui` tests time out the wait and fail their first assertion.

- [ ] **Step 3: Add the strings**

In `crates/fp-app/locales/en-US/main.ftl`, after `settings-dsd-mode = DSD: { $device }`, add:

```ftl
settings-device-rate = Sample rate: { $device }
settings-device-buffer = Buffer size: { $device }
settings-device-global-rate = Global ({ $value } Hz)
settings-device-global-buffer = Global ({ $value })
settings-dsd-default-pcm = By default, DSD is converted to PCM.
settings-dsd-why-not-exclusive = DoP and native DSD need a device that can be opened exclusively; this one cannot.
settings-dsd-why-bit-perfect-off = DoP and native DSD need this device to be bit-perfect.
settings-dsd-why-native-linux = Native DSD needs Linux.
settings-dsd-why-no-native = This device does not take native DSD: its driver reports no DSD format.
settings-dsd-silence = DSD silence
settings-dsd-silence-hint = Sent before a DSD stream starts, after it ends and on a switch to PCM, so that the converter locks without a click.
```

In `crates/fp-app/locales/es-ES/main.ftl`, after `settings-dsd-mode = DSD: { $device }`, add:

```ftl
settings-device-rate = Frecuencia de muestreo: { $device }
settings-device-buffer = Tamaño del búfer: { $device }
settings-device-global-rate = Global ({ $value } Hz)
settings-device-global-buffer = Global ({ $value })
settings-dsd-default-pcm = Por defecto, el DSD se convierte a PCM.
settings-dsd-why-not-exclusive = DoP y DSD nativo necesitan un dispositivo que se pueda abrir en exclusiva, y este no puede.
settings-dsd-why-bit-perfect-off = DoP y DSD nativo necesitan que este dispositivo sea bit perfect.
settings-dsd-why-native-linux = El DSD nativo necesita Linux.
settings-dsd-why-no-native = Este dispositivo no acepta DSD nativo: su controlador no declara ningún formato DSD.
settings-dsd-silence = Silencio DSD
settings-dsd-silence-hint = Se envía antes de que empiece un flujo DSD, al acabar y al pasar a PCM, para que el convertidor se enganche sin chasquido.
```

- [ ] **Step 4: Draw the rows**

In `crates/fp-app/src/ui/settings/devices.rs`:
- change the imports to

```rust
use egui::Ui;
use fp_backends::DeviceInfo;
use fp_model::{DsdCaps, DsdDevice, DsdMix, DsdNotOffered, DsdOutput, OutputDevice};

use super::{
    BUFFER_SIZES, BackendChoice, SAMPLE_RATES, caption, labelled_row, note, row, slider, toggle,
    update,
};
use crate::ui::app::Scene;
use crate::ui::theme;
```

- replace `device_rows` with:

```rust
/// One routed device: its bit-perfect switch, its own rate and buffer, and
/// its DSD mode with why the other modes are not offered (Q3).
fn device_rows(
    ui: &mut Ui,
    scene: &Scene<'_>,
    device: &OutputDevice,
    info: Option<&DeviceInfo>,
    name: &str,
) {
    let t = scene.i18n;
    let outputs = &scene.state.config.outputs;
    let listed = outputs.bit_perfect.contains(device);
    let capable = info.is_some_and(|d| d.exclusive_capable);

    // Bit-perfect (Phase 4 spec B6). A listed device can always be turned
    // off, even when it is not plugged in or cannot be exclusive any more.
    let mut on = listed;
    let enabled = capable || on;
    let label = t.tr_args("settings-bit-perfect-device", &[("device", name.into())]);
    row(ui, name, None, |ui| {
        let response = ui
            .add_enabled_ui(enabled, |ui| toggle(ui, &mut on, &label))
            .response;
        if !enabled {
            response.on_disabled_hover_text(t.tr("bp-not-capable"));
            return;
        }
        if on != listed {
            let device = device.clone();
            update(scene, move |c| {
                c.outputs.bit_perfect.retain(|d| d != &device);
                if on {
                    c.outputs.bit_perfect.push(device);
                }
            });
        }
    });

    // Its own rate (Q12.3).
    let own = outputs.device_override(device);
    let own_rate = own.and_then(|o| o.sample_rate);
    let own_buffer = own.and_then(|o| o.buffer_frames);
    let hz = |r: u32| t.tr_args("unit-hz", &[("value", r.into())]);
    let global_rate = t.tr_args(
        "settings-device-global-rate",
        &[("value", outputs.sample_rate.into())],
    );
    let reported = info.map_or(&[][..], |d| d.sample_rates.as_slice());
    let rates = fp_model::offered_rates(reported, &SAMPLE_RATES, own_rate);
    let rate_label = t.tr_args("settings-device-rate", &[("device", name.into())]);
    labelled_row(ui, &rate_label, None, |ui, label| {
        let shown = own_rate.map_or_else(|| global_rate.clone(), hz);
        let response = egui::ComboBox::from_id_salt(("device-rate", &device.backend, &device.device))
            .selected_text(shown)
            .show_ui(ui, |ui| {
                if ui.selectable_label(own_rate.is_none(), &global_rate).clicked() {
                    set_rate(scene, device, None);
                }
                for r in rates {
                    if ui.selectable_label(own_rate == Some(r), hz(r)).clicked() {
                        set_rate(scene, device, Some(r));
                    }
                }
            })
            .response;
        let _ = response.labelled_by(label);
    });

    // Its own buffer, with the latency at the rate it runs.
    let rate = outputs.rate_for(&device.backend, &device.device);
    let buffer = outputs.buffer_for(&device.backend, &device.device);
    let latency_ms = f64::from(buffer) / f64::from(rate.max(1)) * 1000.0;
    let latency = t.tr_args(
        "settings-latency",
        &[("ms", format!("{latency_ms:.1}").into())],
    );
    let global_buffer = t.tr_args(
        "settings-device-global-buffer",
        &[("value", outputs.buffer_frames.into())],
    );
    let buffers = fp_model::offered_buffers(info.and_then(|d| d.buffer_frames), &BUFFER_SIZES, own_buffer);
    let buffer_label = t.tr_args("settings-device-buffer", &[("device", name.into())]);
    labelled_row(ui, &buffer_label, Some(&latency), |ui, label| {
        let shown = own_buffer.map_or_else(|| global_buffer.clone(), |b| b.to_string());
        let response = egui::ComboBox::from_id_salt(("device-buffer", &device.backend, &device.device))
            .selected_text(shown)
            .show_ui(ui, |ui| {
                if ui.selectable_label(own_buffer.is_none(), &global_buffer).clicked() {
                    set_buffer(scene, device, None);
                }
                for b in buffers {
                    if ui.selectable_label(own_buffer == Some(b), b.to_string()).clicked() {
                        set_buffer(scene, device, Some(b));
                    }
                }
            })
            .response;
        let _ = response.labelled_by(label);
    });

    // Its DSD mode, on every routed device (Q3.1), with the default (Q3.3)
    // and why a mode is missing (Q3.2).
    let caps = DsdCaps {
        bit_perfect: listed,
        exclusive_capable: capable,
        native_dsd: info.is_some_and(|d| d.native_dsd),
        linux: std::env::consts::OS == "linux",
    };
    let current = outputs.dsd_output_for(&device.backend, &device.device);
    let modes = fp_model::offered_dsd_modes(caps, current);
    let default_hint = t.tr("settings-dsd-default-pcm");
    let dsd_label = t.tr_args("settings-dsd-mode", &[("device", name.into())]);
    labelled_row(ui, &dsd_label, Some(&default_hint), |ui, label| {
        ui.add_enabled_ui(modes.len() > 1, |ui| {
            let response = egui::ComboBox::from_id_salt(("dsd-mode", &device.backend, &device.device))
                .selected_text(dsd_mode_label(t, current))
                .show_ui(ui, |ui| {
                    for mode in modes {
                        if ui
                            .selectable_label(mode == current, dsd_mode_label(t, mode))
                            .clicked()
                        {
                            set_dsd_mode(scene, device, mode);
                        }
                    }
                })
                .response;
            let _ = response.labelled_by(label);
        });
    });
    if let Some(why) = fp_model::dsd_not_offered(caps) {
        note(ui, &t.tr(why_key(why)), theme::NEUTRAL_400);
        ui.add_space(4.0);
    }
}

fn why_key(why: DsdNotOffered) -> &'static str {
    match why {
        DsdNotOffered::NotExclusive => "settings-dsd-why-not-exclusive",
        DsdNotOffered::BitPerfectOff => "settings-dsd-why-bit-perfect-off",
        DsdNotOffered::NativeNeedsLinux => "settings-dsd-why-native-linux",
        DsdNotOffered::NoNativeDsd => "settings-dsd-why-no-native",
    }
}

fn set_rate(scene: &Scene<'_>, device: &OutputDevice, rate: Option<u32>) {
    let device = device.clone();
    update(scene, move |c| c.outputs.set_device_rate(&device, rate));
}

fn set_buffer(scene: &Scene<'_>, device: &OutputDevice, frames: Option<u32>) {
    let device = device.clone();
    update(scene, move |c| c.outputs.set_device_buffer(&device, frames));
}

fn set_dsd_mode(scene: &Scene<'_>, device: &OutputDevice, mode: DsdOutput) {
    let device = device.clone();
    update(scene, move |c| {
        c.outputs.dsd_output.retain(|d| {
            (d.backend.as_str(), d.device.as_str()) != (device.backend.as_str(), device.device.as_str())
        });
        if mode != DsdOutput::Pcm {
            c.outputs.dsd_output.push(DsdDevice {
                backend: device.backend,
                device: device.device,
                mode,
            });
        }
    });
}
```

- in `dsd_settings`, after the DSD mix row, add:

```rust
    let mut silence = scene.state.config.outputs.dsd_silence_ms;
    let label = t.tr("settings-dsd-silence");
    row(ui, &label, Some(&t.tr("settings-dsd-silence-hint")), |ui| {
        if slider(ui, &mut silence, 0.0..=2000.0, 10.0, " ms", &label) {
            update(scene, |c| c.outputs.dsd_silence_ms = silence);
        }
    });
```

The slider range is the validated range of `outputs.dsd_silence_ms` (0–2000, `Config::validate`). If `Response::labelled_by` is `#[must_use]`, the `let _ =` keeps clippy quiet. If it is not, clippy accepts the binding as well.

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p fp-app --test settings && cargo test -p fp-app --test dsd_ui && cargo test -p fp-app --test i18n`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/fp-app/src/ui/settings/devices.rs crates/fp-app/locales/en-US/main.ftl \
  crates/fp-app/locales/es-ES/main.ftl crates/fp-app/tests/settings.rs crates/fp-app/tests/dsd_ui.rs
if cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings \
   && cargo test --workspace; then
git commit -m "feat(ui): give each device its rate, buffer and DSD row

Operator feedback 4, Q12.3 and Q3: in Advanced every routed device can
have its own rate and buffer, and shows its DSD mode with the default and
why DoP or native DSD is not offered. The DSD silence gets a slider.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
fi
```

---

### Task 6: Documentation

**Suggested executor:** `sonnet`. This task is documentation only.

**Files:**
- Modify:
  - `docs/user/settings.md`: Restart pending (L18–21), and the Audio outputs table (L44–60);
  - `docs/user/bit-perfect.md`: "Setting a device bit-perfect" (L8–15), the DSD modes paragraph (L64–73) and "Silence at the edges" (L112–119);
  - `docs/user/troubleshooting.md` (L58–60);
  - `docs/technical/persistence.md`: the `outputs` table (L146–158) and the `ui` table (L176–184);
  - `docs/technical/audio-engine.md`: "Rates and bit-perfect buses" (L150) and "DSD buses" (L265);
  - `docs/technical/backends.md`, after the `OutputStream::config()` paragraph (L29–34);
  - `docs/technical/ui.md`, only if it lists the Settings modules (search it for `settings/meters.rs` and add `settings/devices.rs` next to it);
  - `README.md`, the feature list (after the bit-perfect bullet, L87–90);
  - `docs/superpowers/specs/2026-10-06-operator-feedback-4-design.md` §6, an "As built" note.
- Check only: the spec lines Q12 and Q3 change are already in the main spec (L469), the bit-perfect spec (L42, L85) and the feedback 2 spec (L516–518).
- Locales: none (Tasks 4 and 5 added every string to both).

- [ ] **Step 1: Check the binding specs**

Run: `grep -n "Basic | Advanced" docs/superpowers/specs/2026-09-25-fauste-player-design.md; grep -n "rate override\|Advanced view" docs/superpowers/specs/2026-09-26-phase4-bit-perfect-design.md; grep -n "Advanced outputs view" docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md`
Expected: one match in each file. If one is missing, apply the replacement text the feedback 4 spec gives under Q12 or Q3, "Spec lines that change".

- [ ] **Step 2: User guide, Settings**

In `docs/user/settings.md`, Restart pending, replace "system, sample rate, buffer size, the Main and Cue outputs (players and cartwall) and the bit-perfect devices." with "system, sample rate, buffer size (also a device's own), the Main and Cue outputs (players and cartwall), the bit-perfect devices and the DSD settings."

Replace the paragraph after the screenshot and the table rows from "Sample rate" to "When another source needs a DSD output" with:

```markdown
At the top, **Show** chooses **Basic** or **Advanced**. Basic shows the
audio system, the sample rate, the buffer size and the outputs. Advanced
adds, for each device an output uses, its own rate and buffer, the
bit-perfect switch and the DSD mode, and then the DSD settings. Switching
views only shows or hides rows: nothing is changed or reset. When Basic
hides a setting that is in use, a line says so.

| Setting | Meaning |
|---|---|
| Audio system | (unchanged row) |
| Sample rate | The rate every output runs at unless a device has its own (Advanced); files are converted to it with high-quality resampling. Bit-perfect devices start at their rate and then follow the files. |
| Buffer size | Frames per audio block, unless a device has its own; the resulting latency is shown below it |
| Outputs per player | (unchanged row) |
| Test Main / Test Cue | (unchanged row) |
| Cartwall | (unchanged row) |
| Sample rate: *device* (Advanced) | **Global (…)** uses the sample rate above; a value gives this device its own rate. Only the rates the device reports are offered. |
| Buffer size: *device* (Advanced) | **Global (…)** uses the buffer size above; a value gives this device its own, with its latency below. |
| Bit-perfect: *device* (Advanced) | A bit-perfect device is opened with exclusive access and follows each file's sample rate while nothing plays on it. The switch is disabled where the device cannot give exclusive access. See [Bit-perfect output](bit-perfect.md). |
| DSD: *device* (Advanced) | **Convert to PCM** (the default), **DoP** or, on Linux, **Native DSD**. Every device shows it; only the modes the device can take are offered, and a line under it says why the others are not. See [DSD](bit-perfect.md#dsd). |
| When another source needs a DSD output (Advanced) | **Continue the DSD track as PCM** (the default), or **Keep DSD and mute the other sources**. See [DSD](bit-perfect.md#dsd). |
| DSD silence (Advanced) | DSD silence sent before a DSD stream starts, after it ends and on a switch to PCM, so that the converter locks without a click; 200 ms by default, 0 to 2000. |
```

Copy the text of the rows marked "(unchanged row)" from the current table, not the marker.

- [ ] **Step 3: User guide, Bit-perfect and Troubleshooting**

In `docs/user/bit-perfect.md`, "Setting a device bit-perfect", step 2 becomes: "2. Choose **Advanced** at the top of the section. Under **Per-device settings**, turn on **Bit-perfect** next to the device." The device then starts at its own sample rate when it has one (Settings), else at the global rate.

In the DSD section, replace "Under each bit-perfect device, Settings → Audio outputs has a **DSD** choice, next to the device's bit-perfect switch:" with "In the Advanced view of Settings → Audio outputs, every device an output uses has a **DSD** choice under its bit-perfect switch:". Then replace "Only the modes the device can take are offered." with: "Only the modes the device can take are offered, and a line under the choice says why the others are not: bit-perfect is off, the device cannot be opened exclusively, native DSD needs Linux, or the device does not take native DSD."

In "Silence at the edges", replace "It is `outputs.dsd_silence_ms` in the configuration file (0 to 2000)." with "It is **DSD silence** in Settings → Audio outputs, Advanced (0 to 2000 ms)."

In "DSD on a real converter", step 2, replace "a longer `outputs.dsd_silence_ms`" with "a longer **DSD silence** (Settings → Audio outputs, Advanced)".

In `docs/user/troubleshooting.md`, replace "raise `outputs.dsd_silence_ms` (200 by default) in the configuration file." with "raise **DSD silence** (200 ms by default) in Settings → Audio outputs, Advanced."

- [ ] **Step 4: Technical docs**

In `docs/technical/persistence.md`, `outputs` table, add after `dsd_silence_ms`:

```markdown
| `device_overrides[]` | empty | `{ device: { backend, device }, sample_rate?, buffer_frames? }`: a device's own rate and buffer, used instead of `sample_rate` and `buffer_frames` when it opens (operator feedback 4, Q12). Same ranges as the global fields; an out-of-range value is dropped with a warning and the device uses the global one; a device listed twice keeps its first values (warning); an entry with neither is removed |
```

In the `ui` table, add:

```markdown
| `outputs_view` | `Basic` | `Basic` or `Advanced`: what Settings → Audio outputs shows. It changes no output setting |
```

In `docs/technical/audio-engine.md`, "Rates and bit-perfect buses", replace "It starts at `outputs.sample_rate`." with "It starts at its device's own rate (`outputs.device_overrides`, `EngineSettings::rate_for`) or `outputs.sample_rate`, and asks for the device's own buffer or `outputs.buffer_frames` (`buffer_for`). The mixer's smoothing and declick lengths are in that rate's frames, and the one-block margins use the bus's own buffer (`Engine::buffer_of`)." In "DSD buses", replace "then `outputs.sample_rate`" with "then the device's own rate or `outputs.sample_rate` (`rate_for`)".

In `docs/technical/backends.md`, after the `OutputStream::config()` paragraph, add:

```markdown
Each device is asked for its own rate and buffer when it has them
(`outputs.device_overrides`, operator feedback 4, Q12), else for the global
ones; a backend sees only the resulting `StreamConfig`. Settings offers a
device only the rates and buffer sizes its `DeviceInfo` reports
(`fp_model::offered_rates`, `offered_buffers`), all of them when it reports
none.
```

- [ ] **Step 5: README and the spec's "As built" note**

In `README.md`, after the "Bit-perfect output" bullet, add:

```markdown
- **Per-device outputs:** Settings → Audio outputs has a Basic and an
  Advanced view; in Advanced each device can have its own sample rate and
  buffer size, and shows its bit-perfect switch and DSD mode with why a mode
  is not offered.
```

In `docs/superpowers/specs/2026-10-06-operator-feedback-4-design.md`, at the end of §6 (before `## 7. Plans`), add:

```markdown
**As built (plan 5).** The rulings are in
`docs/superpowers/plans/2026-10-06-feedback4-plan5-outputs-settings.md`.
In short:
- an override change reuses the SampleRate and BufferSize restart reasons;
- the DSD reasons and the offered modes live in `fp_model::device_offer`,
  with at most one reason, the most fundamental;
- a device's own rate and buffer offer only what the device reports;
- Basic shows a line when advanced settings are in use;
- the section heading is "Per-device settings".
```

- [ ] **Step 6: Screenshots**

The Outputs page now has the selector, so the guide image changes. On Linux with `xvfb`, `xdotool`, ImageMagick, `ffmpeg`, `python3` and `curl` installed, run `scripts/site/screenshots.sh`. Check `docs/images/guide/settings-outputs.png`: the page must show the Basic view with the selector, and the crop must still fit (CLAUDE.md, Testing notes). If the tools are missing, record that in the ledger as `Ruling: settings-outputs.png not regenerated — tools missing — the guide image lacks the selector until it is`.

- [ ] **Step 7: Commit**

```bash
git add README.md docs/user/settings.md docs/user/bit-perfect.md docs/user/troubleshooting.md \
  docs/technical/persistence.md docs/technical/audio-engine.md docs/technical/backends.md \
  docs/technical/ui.md docs/superpowers/specs/2026-10-06-operator-feedback-4-design.md docs/images/guide
if cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings \
   && cargo test --workspace; then
git commit -m "docs: describe the Basic and Advanced outputs settings

Operator feedback 4, Q12 and Q3: the guide, the technical docs and the
README describe each device's own rate and buffer, the DSD row on every
device with its reason, and the DSD silence control.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
fi
```

(`git add` of a file that did not change is harmless; drop `docs/technical/ui.md` from the list if Step 4 left it untouched.)

---

### Task 7 (optional, manual): Check on real hardware

**Suggested executor:** `opus`, with the maintainer present. Skip this task, and record that it was skipped in the ledger, when the maintainer is not available. Nothing in the pull request depends on it.

**Hardware:** an ICUSBAUDIO7D card is connected; it only does 44.1/48 kHz at 16 bit. The maintainer can also connect an SMSL SU-1 (DoP and native DSD) or a Sound Blaster X4. Hardware names go in the ledger only, never in the docs, specs or commits; describe devices there by what they do ("a USB converter with DoP and native DSD").

**Safety rule for every step:** never play audio on the default desktop output without asking first. The steps below keep the sound servers out of reach, so that only the device under test can open. Even so:
- **stop and ask** the maintainer before the first sound on each device, so that they can lower the volume or take off headphones;
- **stop and ask** again before playing on any card that is the desktop's default output.

**Files:** none in the repository, unless the results change the docs (Step 9). The scratch state lives in the session scratchpad, `$SCRATCH/fp-hw` (call it `$HW`).

- [ ] **Step 1: Find the cards, and the desktop default**

Run: `cat /proc/asound/cards; aplay -l; wpctl status 2>/dev/null | sed -n '/Sinks:/,/Sources:/p'; pactl get-default-sink 2>/dev/null`

Record each card's ALSA id (the name in brackets, for example `ICUSBAUDIO7D`). The application's device id is `alsa:hw:CARD=<id>,DEV=0` on the `alsa` backend. Record which card the desktop's default sink is.

Ask the maintainer which extra device to connect (SU-1 or X4), wait until it is plugged in, and run the command again.

- [ ] **Step 2: What each card offers**

Run, for each card number `N`: `cat /proc/asound/card$N/stream0`.
Record the formats, rates and the DSD formats listed (a USB converter with native DSD lists `DSD_U32_BE` or similar).

Expected: the ICUSBAUDIO7D lists S16_LE at 44100 and 48000 only.

- [ ] **Step 3: Test files**

```bash
mkdir -p "$HW/music"
for spec in "44100 s16" "48000 s16" "96000 s32" "192000 s32"; do
  set -- $spec
  ffmpeg -loglevel error -f lavfi -i "sine=frequency=1000:sample_rate=$1:duration=20" \
    -af "volume=-20dB" -ac 2 -sample_fmt "$2" -ar "$1" "$HW/music/tone-$1.wav"
done
```

Copy any `.dsf` or `.dff` file from `test-music/` (or `$FAUSTE_TEST_MUSIC`) into `$HW/music`. With none, the DSD checks (Step 8) are skipped and the ledger says so.

- [ ] **Step 4: A scratch home routed only to the device under test**

```bash
cargo build --release -p fp-app --bin fauste-player --example demo_session
rm -rf "$HW/home" && mkdir -p "$HW/home"
FAUSTE_HOME="$HW/home" target/release/examples/demo_session "$HW/music"
```

Then patch `config.json` with Python, as `scripts/site/screenshots.sh` does in `write_home`. Set `DEV` to the device under test, for example `alsa:hw:CARD=ICUSBAUDIO7D,DEV=0`:

```bash
DEV='alsa:hw:CARD=ICUSBAUDIO7D,DEV=0' python3 - "$HW/home" <<'EOF'
import json, os, sys
home = sys.argv[1]
dev = os.environ["DEV"]
path = f"{home}/config/config.json"
doc = json.load(open(path))
players = [p["id"] for p in json.load(open(f"{home}/data/session.json"))["players"]]
c = doc["config"]
r = lambda ch: {"backend": "alsa", "device": dev, "first_channel": ch}
c["outputs"]["backend"] = "alsa"
# Every player and the cartwall on the device under test: nothing falls
# back to the system output.
c["outputs"]["routes"] = [{"player": p, "main": r(0), "cue": None} for p in players]
c["outputs"]["cartwall"] = {"main": r(0), "cue": None}
c["ui"]["language"] = "en-US"
c["ui"]["outputs_view"] = "Advanced"
c["remote"]["http"]["enabled"] = True
c["remote"]["http"]["bind"] = "127.0.0.1"
c["remote"]["http"]["port"] = 7391
json.dump(doc, open(path, "w"), indent=2)
EOF
```

- [ ] **Step 5: Start the app with the sound servers out of reach**

```bash
FAUSTE_HOME="$HW/home" PULSE_SERVER=unix:/nonexistent PIPEWIRE_REMOTE=fauste-none \
  JACK_NO_START_SERVER=1 target/release/fauste-player >"$HW/app.out" 2>&1 &
```

Run it in the background, with `run_in_background`. Then check: `grep -ho 'device: "[^"]*"' "$HW"/home/logs/fauste-player*.log | sort -u`.
Expected: only the `hw:CARD=…` device under test. If any other device appears, stop the app (`kill %1`) and fix the config before any sound.

- [ ] **Step 6: Per-device rate and buffer (ICUSBAUDIO7D)**

1. In Settings → Audio outputs, Advanced, open **Sample rate: ICUSBAUDIO7D…**.
   Expected: only **Global (48000 Hz)**, **44100 Hz** and **48000 Hz** are offered (Review Focus 2). Record what is listed.
2. Choose **44100 Hz** and, under **Buffer size**, **256**. Choose **Restart now**.
3. **Ask the maintainer** to confirm the volume. Then play the first entry of player 1 (`curl -s -X POST http://127.0.0.1:7391/api/v1/players/<id>/play`, with the id from `GET /api/v1/players`), or press Play in the window.
4. Run `cat /proc/asound/card<N>/pcm0p/sub0/hw_params`.
   Expected: `rate: 44100 (44100/1)`. Record `period_size` and `buffer_size`: the requested 256 frames should show as the period size. If they do not, record what the driver chose, and compare with a run at **Global**.
5. Stop. With the extra device connected, also route player 2's Main to it. Use Settings, or Step 4's script with a second device, and leave it at **Global**. Play both players.
   Expected: each card's `hw_params` shows its own rate (44100 and 48000).

- [ ] **Step 7: Bit-perfect**

1. On the SU-1 (or the X4), turn **Bit-perfect** on, give it its own rate of 96000, restart, and **ask before playing**.
   - While a stream is open and idle, `hw_params` shows `rate: 96000`.
   - Play `tone-44100.wav` at 100 %: the rate becomes 44100 and the player's **BP** badge lights.
   - Play `tone-192000.wav`: 192000.
2. On the ICUSBAUDIO7D, turn **Bit-perfect** on, restart, and play `tone-96000.wav`.
   Expected: the card refuses 96000, so the stream stays at the device's rate, the track plays resampled, and the **BP** badge stays off.

Record the results in the ledger.

- [ ] **Step 8: DSD modes (needs DSD files)**

1. On the ICUSBAUDIO7D and the X4, read the line under **DSD: …**. It should read "This device does not take native DSD: …", or "… need this device to be bit-perfect" while bit-perfect is off. Record each.
2. On the SU-1 with bit-perfect on:
   - choose **DoP**, restart, **ask**, and play a DSF file at 100 %. The header shows **DSD**, the converter's display shows DSD64 (or the file's rate), and `hw_params` shows a 176400 Hz (DSD64) 24- or 32-bit PCM stream.
   - choose **Native DSD** (offered only if the driver reports a DSD format), restart and play. `hw_params` shows a `DSD_U32_BE` (or similar) format.
   - listen with the maintainer for a click at start, Stop and the end. If there is one, raise **DSD silence** with the new slider, restart, and repeat.
3. On the ICUSBAUDIO7D (16-bit) with **DoP** set, play the DSF file.
   Expected: it plays converted to PCM, and the header does not show **DSD**.

- [ ] **Step 9: Record and clean up**

1. Stop the app (`kill` its pid) and `rm -rf "$HW"`.
2. Write each result in the ledger: device, setting, expected, seen.
3. Fix any bug found test-first, under `superpowers:systematic-debugging`, in a commit on this branch, before the pull request.
4. If DoP or native DSD worked on the real converter, update the docs:
   - `docs/user/bit-perfect.md`, "DSD on a real converter": replace "DoP and native DSD have not been tried on a real converter by the project." with what was verified, for example "DoP and native DSD have been checked on one USB converter on Linux.";
   - `README.md`: in the DSD bullet, replace "DoP and native DSD are checked on simulated devices only; a check on a real converter is still to do." to match;
   - `README.md`, roadmap row 6: replace "(DoP and native DSD need a check on real hardware)" to match.

   Name no product. Then commit:

```bash
git add README.md docs/user/bit-perfect.md
if cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings \
   && cargo test --workspace; then
git commit -m "docs: record the DSD check on a real converter

DoP and native DSD were checked on a USB converter on Linux, with each
device's own rate and buffer and bit-perfect rate following.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
fi
```

---

## Self-Review

**Spec coverage (§6):**

| Rule | Where |
|---|---|
| Q12.1 selector, persisted, `Basic` by default, lenient | Task 1 (`OutputsView`, lenient test), Task 4 (selector, kittests) |
| Q12.2 Basic shows the system, rate, buffer and routes | Task 4 `the_basic_view_hides_the_device_rows` |
| Q12.3 per-device rate and buffer, bit-perfect, DSD mode; `dsd_mix` and `dsd_silence_ms` once | Task 4 (move), Task 5 (rate, buffer, DSD row, silence slider) |
| Q12.4 `device_overrides`, ranges, drop with a log line | Task 1 (validate tests, lenient test) |
| Q12.5 `EngineSettings`, `ensure_bus`, bit-perfect starts at the device's rate | Task 3 |
| Q12.6 restart | Task 1 (`restart.rs`) |
| Q12.7 Basic never changes the configuration; the defaults | Task 4 `switching_views_changes_only_the_view`; Task 1 defaults test |
| Q3.1 every routed device shows its DSD row in Advanced | Task 5 `a_device_that_is_not_bit_perfect_shows_its_dsd_row_and_why` |
| Q3.2 a line says why | Tasks 2 and 5 |
| Q3.3 the default is PCM | Task 5 (row hint, kittest) |
| Q3.4 pure function, one test per reason | Task 2 (four reason tests and one "none" test) |
| Tests the spec lists (fp-model, fp-engine, kittests in `settings.rs` and `dsd_ui.rs`) | Tasks 1, 3, 4, 5 |
| Docs and locales the spec lists | Tasks 4, 5 (keys), Task 6 (`settings.md`, `bit-perfect.md`, `backends.md`, `audio-engine.md`) |
| Spec lines that change | already in the specs; Task 6 Step 1 checks them |

**Placeholder scan:** every code step has its code. The task text marks the only "copy verbatim" instructions with exact line ranges: in Task 4, moving `bit_perfect()` and the DSD mix row, and in Task 6, the "(unchanged row)" table rows. Task 5 then replaces the moved `device_rows` in full.

**Type consistency:**
- Every task uses the same calls: `set_device_rate`/`set_device_buffer(&OutputDevice, Option<u32>)`, `rate_for`/`buffer_for(&str, &str)` on `OutputsConfig`, and `rate_for`/`buffer_for(&BusKey)` on `EngineSettings`.
- `offered_dsd_modes(DsdCaps, DsdOutput)` has the same signature in Tasks 2 and 5.
- `DsdNotOffered`'s variant names match between Task 2 and `why_key`.
- `outputs_in(OutputsView, Vec<OutputDevice>)` is the same in Tasks 4 and 5.

**Review Focus:** each of the five lines names its test and its task.
