# Live Settings, Plan 1: Model Rules and Pending State

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Put the live-settings rules (L1–L10, L20, L22–L24; L18 and L19 are dropped, see the ruling below) in `fp-model` as pure functions over a runtime `AppState::live`, with the new engine vocabulary (`Placed`, `Unplaced`, `Gone`, `AudioSystemInUse`, `Applied`; `UpdateSettings`, `ApplyAudioSystem`, `ApplyRoute`, `ApplyDevice`; `Command::ApplySettingsNow`).

**Architecture:** A new module `fp_model::live` holds the types (`Holder`, `DeviceSettings`, `Target`, `Wanted`, `BusyCause`, `LiveSettings`, `Pending`) and the rules (`causes`, `device_causes`, `pending`, `due`, `interruptions`). The reducer feeds `live` from engine events and, at the end of every `apply` and `on_event`, emits the output changes that are due. The engine only gets the new action variants as no-ops here (plans 2 and 3 make it report and apply); since it reports nothing yet, nothing is ever due at run time after this plan, and the restart notice of feedback 2 O4 keeps working until plan 5.

**Tech Stack:** Rust 2024, `fp-model` (no I/O, no threads), `std::collections::BTreeMap`.

**Spec:** `docs/superpowers/specs/2026-10-07-live-settings-design.md` (read it whole, including §14 and the "Planning notes" added on 2026-10-08).

**Maintainer ruling (2026-10-08), binding:** rules L18 and L19 (the player and cart-grid limit reductions) are dropped: limits never change while the application runs, so they are unreachable. Task 6 is not built; `PendingItem::{Players, CartPage}`, `BusyCause::{PlayerPaused, CartsBeyondGrid}` and `enforce_limits` do not exist; `UpdateConfig` keeps its existing `PlayerBusy` refusal. Where a step below still mentions them, ignore that part (the ledger lists each deviation).

## Global Constraints

- All code, identifiers, comments, docs and commit messages in English; never mention other playout or radio-automation products.
- D1: a change is applied as soon as it is safe; nothing is cut on air by itself (CLAUDE.md rule 10).
- D2: **busy** = a device carries audio: a source routed to it is playing, fading or in CUE (PFL), or a cart is playing on it. A paused player, or one with a track only loaded, is **not** busy. A held (paused) CUE **is** busy (Q4).
- D6: the rules live in `fp-model` as pure functions, one test per rule; the engine and the UI only execute and display.
- L24: nothing pending is saved; `AppState::live` is runtime only (like `PlayerState::dsd`).
- `unsafe_code` forbidden; `unwrap`, `expect`, `panic` denied outside tests; prefer `get` over indexing.
- Conventional Commits; every commit ends with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Gate for every task: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`.

## Review Focus

1. **A player removed while the engine still knows its holders** (the `Gone` event arrives a tick after `RemovePlayer`): `pending` must not list a route or cause for a player that is no longer in `state.players`. Test: Task 4, `a_removed_player_is_never_pending`.
2. **An `Applied` for an older value after the operator changed the setting again**: the newer value stays in flight and the running value is the one the engine took. Test: Task 5, `l8_an_answer_for_an_older_value_keeps_the_newer_one_in_flight`.
3. **A device no holder uses any more while its refusal is recorded**: its running settings, in-flight action and failure are forgotten, so a later use starts clean. Test: Task 3, `a_device_no_holder_uses_any_more_is_forgotten_with_its_failure`.
4. *(dropped with L18.)*
5. **A setting changed back while it waits**: the item disappears and no action is sent for it. Test: Task 5, `changing_a_setting_back_while_it_waits_leaves_nothing_pending`.

---

## File Structure

| File | Responsibility |
|---|---|
| `crates/fp-model/src/live.rs` (new) | Types and pure rules of the live settings spec |
| `crates/fp-model/src/lib.rs` | `pub mod live;` and re-exports |
| `crates/fp-model/src/state.rs` | `AppState::live` |
| `crates/fp-model/src/session.rs:173` | `AppState { .. }` literal gets `live` |
| `crates/fp-model/src/config.rs:305` | `OutputDevice` derives `PartialOrd, Ord` |
| `crates/fp-model/src/command.rs` | New `EngineEvent`, `EngineAction`, `Command` variants |
| `crates/fp-model/src/reducer.rs` | Feeds `live` from events; due actions at the end of `apply`/`on_event`; `update_config` (L10) |
| `crates/fp-engine/src/engine.rs:589` (`execute`) | No-op arms for the new actions (until plans 2 and 3) |
| `crates/fp-model/tests/live.rs` (new) | One test per rule |

---

### Task 1: Live types, `AppState::live` and `device_settings` (L4)

**Files:**
- Create: `crates/fp-model/src/live.rs`
- Modify: `crates/fp-model/src/lib.rs` (module list ~L3-30, re-exports ~L33-90)
- Modify: `crates/fp-model/src/state.rs:10-48`
- Modify: `crates/fp-model/src/session.rs:173-180`
- Modify: `crates/fp-model/src/config.rs:305` (`OutputDevice` derive)
- Test: `crates/fp-model/tests/live.rs`

**Interfaces:**
- Produces (in `fp_model`, re-exported at the crate root):
  - `pub enum Holder { PlayerMain(PlayerId), PlayerCue(PlayerId), CartwallMain, CartwallCue }` — `Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord`
  - `pub struct DeviceSettings { sample_rate: u32, buffer_frames: u32, bit_perfect: bool, dsd: DsdOutput, dsd_mix: Option<DsdMix>, dsd_silence_ms: Option<f64> }` — `Debug, Clone, Copy, PartialEq`
  - `pub enum Target { AudioSystem, Route(Holder), Device(OutputDevice) }` — `Debug, Clone, PartialEq, Eq, PartialOrd, Ord` (variant order is the L9 order)
  - `pub enum Wanted { AudioSystem(Option<String>), Route(Option<Route>), Device(DeviceSettings) }` — `Debug, Clone, PartialEq`
  - `pub struct Failure { pub wanted: Wanted, pub reason: String }`
  - `pub enum BusyCause { PlayerPlaying(PlayerId), PlayerFading(PlayerId), PlayerCue(PlayerId), CartPlaying(CartId), CartCue(CartId), PlayerPaused(PlayerId), CartsBeyondGrid { page: CartPageId, count: usize } }` — `Debug, Clone, Copy, PartialEq, Eq`
  - `pub struct LiveSettings { audio_system: Option<Option<String>>, audio_system_in_use: Option<String>, placement: BTreeMap<Holder, OutputDevice>, routes: BTreeMap<Holder, Option<Route>>, devices: BTreeMap<OutputDevice, DeviceSettings>, in_flight: BTreeMap<Target, Wanted>, failures: BTreeMap<Target, Failure> }` — `Debug, Clone, Default, PartialEq`, all fields `pub`
  - `pub fn device_settings(outputs: &OutputsConfig, device: &OutputDevice) -> DeviceSettings`
  - `AppState::live: LiveSettings`

- [ ] **Step 1: Write the failing tests**

Create `crates/fp-model/tests/live.rs`:

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Live settings spec (2026-10-07): one test per rule L1–L10, L20,
//! L22–L24. The engine is stood in for by the events it reports.

mod common;

use fp_model::{
    AppState, Config, DeviceSettings, DsdDevice, DsdMix, DsdOutput, LiveSettings, OutputDevice,
    Route, device_settings,
};

fn dev(name: &str) -> OutputDevice {
    OutputDevice {
        backend: "null".into(),
        device: name.into(),
    }
}

fn route_to(name: &str) -> Route {
    Route {
        backend: "null".into(),
        device: name.into(),
        first_channel: 0,
    }
}

#[test]
fn the_live_state_starts_empty_and_unknown() {
    let s = AppState::new(Config::default(), "Main");
    assert_eq!(s.live, LiveSettings::default());
    assert_eq!(s.live.audio_system, None, "unknown until the engine says");
}

#[test]
fn l4_device_settings_follow_the_effective_rate_buffer_and_bit_perfect() {
    let mut c = Config::default();
    c.outputs.routes = vec![fp_model::PlayerRoutes {
        player: fp_model::PlayerId(1),
        main: Some(route_to("dac")),
        cue: None,
    }];
    c.outputs.set_device_rate(&dev("dac"), Some(96_000));
    c.outputs.set_device_buffer(&dev("dac"), Some(256));
    c.outputs.bit_perfect = vec![dev("dac")];
    assert_eq!(
        device_settings(&c.outputs, &dev("dac")),
        DeviceSettings {
            sample_rate: 96_000,
            buffer_frames: 256,
            bit_perfect: true,
            dsd: DsdOutput::Pcm,
            dsd_mix: None,
            dsd_silence_ms: None,
        }
    );
    // An unrouted device opens at the global values.
    let other = device_settings(&c.outputs, &dev("other"));
    assert_eq!((other.sample_rate, other.buffer_frames), (48_000, 512));
    assert!(!other.bit_perfect);
}

#[test]
fn l4_dsd_mix_and_silence_count_only_on_a_device_that_carries_dsd() {
    let mut c = Config::default();
    c.outputs.routes = vec![fp_model::PlayerRoutes {
        player: fp_model::PlayerId(1),
        main: Some(route_to("dac")),
        cue: Some(route_to("phones")),
    }];
    c.outputs.bit_perfect = vec![dev("dac")];
    c.outputs.dsd_output = vec![DsdDevice {
        backend: "null".into(),
        device: "dac".into(),
        mode: DsdOutput::Dop,
    }];
    let before_dac = device_settings(&c.outputs, &dev("dac"));
    let before_phones = device_settings(&c.outputs, &dev("phones"));
    c.outputs.dsd_mix = DsdMix::HoldOthers;
    c.outputs.dsd_silence_ms = 400.0;
    let dac = device_settings(&c.outputs, &dev("dac"));
    assert_eq!(dac.dsd, DsdOutput::Dop);
    assert_eq!(dac.dsd_mix, Some(DsdMix::HoldOthers));
    assert_eq!(dac.dsd_silence_ms, Some(400.0));
    assert_ne!(dac, before_dac);
    assert_eq!(
        device_settings(&c.outputs, &dev("phones")),
        before_phones,
        "a PCM device does not change"
    );
    // A DSD mode on a device that is not bit-perfect is PCM.
    c.outputs.bit_perfect.clear();
    assert_eq!(device_settings(&c.outputs, &dev("dac")).dsd, DsdOutput::Pcm);
}
```

Later tasks extend the `use` lines at the top of this file; add exactly the names each task lists, so that no import is ever unused (clippy runs with `-D warnings` on the tests too).

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-model --test live`
Expected: FAIL to compile: `cannot find type DeviceSettings in crate fp_model`, `no field live on AppState`.

- [ ] **Step 3: Write the implementation**

In `crates/fp-model/src/config.rs:305`, change the derive of `OutputDevice` to:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct OutputDevice {
```

Create `crates/fp-model/src/live.rs`:

```rust
//! Live settings (live settings spec, 2026-10-07): the output settings the
//! engine still runs with an older value, what keeps each change waiting,
//! and the engine work that applies a change once it is safe. Everything
//! here is a pure function of `AppState`; the engine reports what it runs
//! with through `EngineEvent`s, and `AppState::live` keeps it (runtime
//! only, never saved: L24).

use std::collections::BTreeMap;

use crate::config::{OutputDevice, OutputsConfig, Route};
use crate::dsd::{DsdMix, DsdOutput};
use crate::ids::{CartId, CartPageId, PlayerId};

/// One audio path a route places on a device (spec §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Holder {
    PlayerMain(PlayerId),
    PlayerCue(PlayerId),
    CartwallMain,
    CartwallCue,
}

/// Everything a device's stream depends on (L4).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeviceSettings {
    pub sample_rate: u32,
    pub buffer_frames: u32,
    pub bit_perfect: bool,
    /// `Pcm` unless the device is bit-perfect.
    pub dsd: DsdOutput,
    /// Only on a device whose `dsd` is not `Pcm`: elsewhere it changes
    /// nothing, so it never makes a device pending.
    pub dsd_mix: Option<DsdMix>,
    pub dsd_silence_ms: Option<f64>,
}

/// What one engine action changes. The variant order is the order of L9.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Target {
    AudioSystem,
    Route(Holder),
    Device(OutputDevice),
}

/// The value an action asks for.
#[derive(Debug, Clone, PartialEq)]
pub enum Wanted {
    AudioSystem(Option<String>),
    Route(Option<Route>),
    Device(DeviceSettings),
}

/// The last value a target refused, and why (L14).
#[derive(Debug, Clone, PartialEq)]
pub struct Failure {
    pub wanted: Wanted,
    pub reason: String,
}

/// Why an item waits (spec §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusyCause {
    PlayerPlaying(PlayerId),
    PlayerFading(PlayerId),
    PlayerCue(PlayerId),
    CartPlaying(CartId),
    CartCue(CartId),
    /// Only for the player limit (L18): a paused player is not removed.
    PlayerPaused(PlayerId),
    /// Only for a cart page over the grid limit (L19): `count` carts with
    /// a file sit beyond the new grid.
    CartsBeyondGrid { page: CartPageId, count: usize },
}

/// What the engine runs with, as it reported it (spec §4.1).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LiveSettings {
    /// The `outputs.backend` the engine runs with; `None` until the engine
    /// reports it (`AudioSystemInUse`), and then no audio system change is
    /// pending.
    pub audio_system: Option<Option<String>>,
    /// The backend the engine really chose (the status bar's label).
    pub audio_system_in_use: Option<String>,
    /// Where each holder with a bus plays.
    pub placement: BTreeMap<Holder, OutputDevice>,
    /// The route the engine holds for each holder it knows.
    pub routes: BTreeMap<Holder, Option<Route>>,
    /// The running settings of each device in use.
    pub devices: BTreeMap<OutputDevice, DeviceSettings>,
    /// Actions sent and not answered yet, with the value asked for.
    pub in_flight: BTreeMap<Target, Wanted>,
    /// The last refused value per target.
    pub failures: BTreeMap<Target, Failure>,
}

/// L4: the settings `device` opens with under `outputs`.
pub fn device_settings(outputs: &OutputsConfig, device: &OutputDevice) -> DeviceSettings {
    let dsd = outputs.dsd_output_for(&device.backend, &device.device);
    let carries_dsd = dsd != DsdOutput::Pcm;
    DeviceSettings {
        sample_rate: outputs.effective_rate(device),
        buffer_frames: outputs.effective_buffer(device),
        bit_perfect: outputs.bit_perfect.contains(device),
        dsd,
        dsd_mix: carries_dsd.then_some(outputs.dsd_mix),
        dsd_silence_ms: carries_dsd.then_some(outputs.dsd_silence_ms),
    }
}
```

In `crates/fp-model/src/lib.rs`, add `pub mod live;` after `mod entry_notice;`… keeping the list alphabetical (between `pub mod ids;` and `pub mod midi;`), and add the re-export after `pub use ids::…`:

```rust
pub use live::{
    BusyCause, DeviceSettings, Failure, Holder, LiveSettings, Target, Wanted, device_settings,
};
```

In `crates/fp-model/src/state.rs`, add the field after `pub ids: IdGen,` (L19):

```rust
    /// What the engine runs with (live settings spec); runtime only.
    pub live: crate::live::LiveSettings,
```

and in `AppState::new` (the `Self { … }` literal, L38-47) add `live: crate::live::LiveSettings::default(),` after `ids,`.

In `crates/fp-model/src/session.rs:173-180`, add `live: crate::live::LiveSettings::default(),` after `ids,` in the `AppState { … }` literal.

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-model --test live`
Expected: PASS.

- [ ] **Step 5: Run the gate and commit**

Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
Expected: all green.

```bash
git add crates/fp-model/src/live.rs crates/fp-model/src/lib.rs crates/fp-model/src/state.rs \
  crates/fp-model/src/session.rs crates/fp-model/src/config.rs crates/fp-model/tests/live.rs
git commit -m "$(cat <<'EOF'
feat(model): add the live settings state and device settings

AppState::live keeps what the engine runs with (runtime only), and
device_settings (L4) gives everything a device's stream depends on.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
EOF
)"
```

---

### Task 2: Busy causes (L1, L2)

**Files:**
- Modify: `crates/fp-model/src/live.rs`
- Modify: `crates/fp-model/src/lib.rs` (re-exports)
- Test: `crates/fp-model/tests/live.rs`

**Interfaces:**
- Consumes: `Holder`, `BusyCause`, `LiveSettings::placement` (Task 1).
- Produces:
  - `pub fn holders(state: &AppState) -> Vec<Holder>` — display order: each player's Main then Cue, in `state.players` order, then `CartwallMain`, `CartwallCue`.
  - `pub fn configured_route(config: &Config, holder: Holder) -> Option<Route>`
  - `pub fn causes(state: &AppState, holder: Holder) -> Vec<BusyCause>` (L1; one per playing cart for `CartwallMain`)
  - `pub fn device_causes(state: &AppState, device: &OutputDevice) -> Vec<BusyCause>` (L2)

- [ ] **Step 1: Write the failing tests**

In `crates/fp-model/tests/live.rs`, replace the two `use` lines with:

```rust
use common::{entries, fixture, p0};
use fp_model::{
    AppState, BusyCause, CartId, Command, Config, DeviceSettings, DsdDevice, DsdMix, DsdOutput,
    Holder, LiveSettings, OutputDevice, Route, TrackAnalysis, apply, causes, device_causes,
    device_settings,
};
```

and append:

```rust
/// Gives cart `index` of the first page a 10 s file.
fn load_cart(state: &mut AppState, index: usize) -> CartId {
    let page = state.cartwall.pages[0].id;
    let path = std::path::PathBuf::from(format!("/carts/cart{index}.wav"));
    apply(state, Command::AssignCartFile { page, index, path }).unwrap();
    let track = state.cartwall.pages[0].carts[index].track.unwrap();
    let analysis = TrackAnalysis {
        duration_secs: 10.0,
        ..TrackAnalysis::default()
    };
    apply(
        state,
        Command::ApplyAnalysis {
            track,
            analysis: Box::new(analysis),
        },
    )
    .unwrap();
    state.cartwall.pages[0].carts[index].id
}

#[test]
fn l1_a_playing_or_fading_player_is_busy_a_paused_or_loaded_one_is_not() {
    let mut s = fixture(3);
    let p = p0(&s);
    let main = Holder::PlayerMain(p);
    assert!(causes(&s, main).is_empty(), "stopped with a track loaded");
    apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(causes(&s, main), vec![BusyCause::PlayerPlaying(p)]);
    apply(&mut s, Command::Pause(p)).unwrap();
    assert!(causes(&s, main).is_empty(), "paused is not busy (D2)");
    apply(&mut s, Command::Pause(p)).unwrap();
    apply(&mut s, Command::FadeStop(p)).unwrap();
    assert_eq!(causes(&s, main), vec![BusyCause::PlayerFading(p)]);
}

#[test]
fn l1_a_cue_playing_or_held_keeps_the_players_cue_busy() {
    let mut s = fixture(3);
    let (p, e) = (p0(&s), entries(&s));
    apply(&mut s, Command::CueEntry(p, e[1])).unwrap();
    assert_eq!(causes(&s, Holder::PlayerCue(p)), vec![BusyCause::PlayerCue(p)]);
    apply(&mut s, Command::SetCuePaused(p, true)).unwrap();
    assert_eq!(
        causes(&s, Holder::PlayerCue(p)),
        vec![BusyCause::PlayerCue(p)],
        "Q4: a held CUE counts"
    );
    assert!(causes(&s, Holder::PlayerMain(p)).is_empty());
}

#[test]
fn l1_playing_carts_keep_the_cartwall_main_busy_and_a_cart_cue_its_cue() {
    let mut s = fixture(0);
    let (a, b) = (load_cart(&mut s, 0), load_cart(&mut s, 1));
    apply(&mut s, Command::FireCart(a)).unwrap();
    apply(&mut s, Command::FireCart(b)).unwrap();
    assert_eq!(
        causes(&s, Holder::CartwallMain),
        vec![BusyCause::CartPlaying(a), BusyCause::CartPlaying(b)]
    );
    assert!(causes(&s, Holder::CartwallCue).is_empty());
    let c = load_cart(&mut s, 2);
    apply(&mut s, Command::CueCart(c)).unwrap();
    assert_eq!(causes(&s, Holder::CartwallCue), vec![BusyCause::CartCue(c)]);
}

#[test]
fn l2_a_device_collects_the_causes_of_every_holder_placed_on_it() {
    let mut s = fixture(3);
    let (p1, p2) = (s.players[0].id, s.players[1].id);
    // Placements as the engine reports them (Task 3 adds the events).
    for p in s.players.clone() {
        s.live.placement.insert(Holder::PlayerMain(p.id), dev("default"));
        s.live.placement.insert(Holder::PlayerCue(p.id), dev("phones"));
    }
    apply(&mut s, Command::Play(p1)).unwrap();
    apply(&mut s, Command::Play(p2)).unwrap();
    assert_eq!(
        device_causes(&s, &dev("default")),
        vec![BusyCause::PlayerPlaying(p1), BusyCause::PlayerPlaying(p2)]
    );
    assert!(device_causes(&s, &dev("phones")).is_empty(), "no CUE");
    assert!(device_causes(&s, &dev("elsewhere")).is_empty());
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-model --test live`
Expected: FAIL to compile: `cannot find function causes in crate fp_model`.

- [ ] **Step 3: Write the implementation**

Append to `crates/fp-model/src/live.rs` (add `use crate::config::Config; use crate::player::Transport; use crate::state::AppState;` to the imports):

```rust
/// Every holder, in display order: each player's Main then Cue, then the
/// cartwall's Main and Cue (L9).
pub fn holders(state: &AppState) -> Vec<Holder> {
    state
        .players
        .iter()
        .flat_map(|p| [Holder::PlayerMain(p.id), Holder::PlayerCue(p.id)])
        .chain([Holder::CartwallMain, Holder::CartwallCue])
        .collect()
}

/// The route `config` gives `holder`.
pub fn configured_route(config: &Config, holder: Holder) -> Option<Route> {
    let outputs = &config.outputs;
    match holder {
        Holder::PlayerMain(p) => outputs.player_routes(p).and_then(|r| r.main.clone()),
        Holder::PlayerCue(p) => outputs.player_routes(p).and_then(|r| r.cue.clone()),
        Holder::CartwallMain => outputs.cartwall.main.clone(),
        Holder::CartwallCue => outputs.cartwall.cue.clone(),
    }
}

/// L1: why `holder` carries audio now; empty when it does not. A paused
/// player, a stopped one with a track loaded and a preload carry nothing
/// (D2); a held CUE does (Q4).
pub fn causes(state: &AppState, holder: Holder) -> Vec<BusyCause> {
    match holder {
        Holder::PlayerMain(id) => state
            .player(id)
            .ok()
            .and_then(|p| {
                if p.fading {
                    Some(BusyCause::PlayerFading(id))
                } else if p.transport == Transport::Playing {
                    Some(BusyCause::PlayerPlaying(id))
                } else {
                    None
                }
            })
            .into_iter()
            .collect(),
        Holder::PlayerCue(id) => state
            .player(id)
            .ok()
            .filter(|p| p.cue.is_some())
            .map(|_| BusyCause::PlayerCue(id))
            .into_iter()
            .collect(),
        Holder::CartwallMain => state
            .cartwall
            .playing
            .iter()
            .map(|c| BusyCause::CartPlaying(c.cart))
            .collect(),
        Holder::CartwallCue => state.cartwall.cue.map(BusyCause::CartCue).into_iter().collect(),
    }
}

/// Appends the causes of `holders` to `out`, each once.
fn collect_causes(state: &AppState, holders: impl Iterator<Item = Holder>, out: &mut Vec<BusyCause>) {
    for holder in holders {
        for cause in causes(state, holder) {
            if !out.contains(&cause) {
                out.push(cause);
            }
        }
    }
}

/// L2: the causes of every holder placed on `device`. A device is idle
/// when this is empty.
pub fn device_causes(state: &AppState, device: &OutputDevice) -> Vec<BusyCause> {
    let mut out = Vec::new();
    let on_device = holders(state)
        .into_iter()
        .filter(|h| state.live.placement.get(h) == Some(device));
    collect_causes(state, on_device, &mut out);
    out
}
```

In `crates/fp-model/src/lib.rs`, extend the `live` re-export to:

```rust
pub use live::{
    BusyCause, DeviceSettings, Failure, Holder, LiveSettings, Target, Wanted, causes,
    configured_route, device_causes, device_settings, holders,
};
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-model --test live`
Expected: PASS.

- [ ] **Step 5: Run the gate and commit**

Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`

```bash
git add crates/fp-model/src/live.rs crates/fp-model/src/lib.rs crates/fp-model/tests/live.rs
git commit -m "$(cat <<'EOF'
feat(model): say why a holder or a device is busy

L1 and L2 of the live settings spec: playing, fading and CUE (held or
not) are busy; paused and loaded are not.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
EOF
)"
```

---

### Task 3: Engine reports reach the model (L3, the answer half of L8, L14)

**Files:**
- Modify: `crates/fp-model/src/command.rs:191-236` (`EngineEvent`)
- Modify: `crates/fp-model/src/live.rs`
- Modify: `crates/fp-model/src/reducer.rs:335-443` (`on_event`)
- Modify (compile fixes only, if any): files that relied on `EngineEvent: Copy`
- Test: `crates/fp-model/tests/live.rs`

**Interfaces:**
- Consumes: Task 1 types.
- Produces (new `EngineEvent` variants; `EngineEvent` now derives only `Debug, Clone, PartialEq` — it carries `String`s and an `f64`):
  - `Placed { holder: Holder, route: Option<Route>, device: OutputDevice, running: DeviceSettings }`
  - `Unplaced { holder: Holder, route: Option<Route> }`
  - `Gone { holder: Holder }`
  - `AudioSystemInUse { configured: Option<String>, in_use: String }`
  - `Applied { target: Target, wanted: Wanted, outcome: Result<(), String> }`
  - `pub(crate) fn placed(state, holder, route, device, running)`, `unplaced(state, holder, route)`, `gone(state, holder)`, `audio_system_in_use(state, configured, in_use)`, `applied(state, target, wanted, outcome)` in `live.rs`.

- [ ] **Step 1: Write the failing tests**

In `crates/fp-model/tests/live.rs`, replace the `use fp_model::{…};` line with:

```rust
use fp_model::{
    AppState, BusyCause, CartId, Command, Config, DeviceSettings, DsdDevice, DsdMix, DsdOutput,
    EngineEvent, Failure, Holder, LiveSettings, OutputDevice, Route, Target, TrackAnalysis, Wanted,
    apply, causes, configured_route, device_causes, device_settings, on_event,
};
```

and append these helpers, used from here on:

```rust
/// Reports a holder as the engine does (L3).
fn report(
    state: &mut AppState,
    holder: Holder,
    route: Option<Route>,
    target: Option<OutputDevice>,
) {
    let event = match target {
        Some(device) => EngineEvent::Placed {
            holder,
            route,
            running: device_settings(&state.config.outputs, &device),
            device,
        },
        None => EngineEvent::Unplaced { holder, route },
    };
    on_event(state, event);
}

/// What the engine reports at start (L3): the audio system, each
/// player's Main on its route's device (`default` without one), its Cue
/// on its route's device, and the cartwall unplaced.
fn report_start(state: &mut AppState) {
    let configured = state.config.outputs.backend.clone();
    on_event(
        state,
        EngineEvent::AudioSystemInUse {
            configured,
            in_use: "null".into(),
        },
    );
    let ids: Vec<_> = state.players.iter().map(|p| p.id).collect();
    for id in ids {
        for holder in [Holder::PlayerMain(id), Holder::PlayerCue(id)] {
            let route = configured_route(&state.config, holder);
            let target = match (&route, holder) {
                (Some(r), _) => Some(dev(&r.device)),
                (None, Holder::PlayerMain(_)) => Some(dev("default")),
                (None, _) => None,
            };
            report(state, holder, route, target);
        }
    }
    for holder in [Holder::CartwallMain, Holder::CartwallCue] {
        let route = configured_route(&state.config, holder);
        report(state, holder, route, None);
    }
}
```

Then append the tests:


```rust
#[test]
fn l3_the_engine_reports_where_each_holder_plays() {
    let mut s = fixture(1);
    let p = p0(&s);
    report_start(&mut s);
    assert_eq!(s.live.audio_system, Some(None));
    assert_eq!(s.live.audio_system_in_use.as_deref(), Some("null"));
    assert_eq!(s.live.placement.get(&Holder::PlayerMain(p)), Some(&dev("default")));
    assert_eq!(s.live.placement.get(&Holder::PlayerCue(p)), Some(&dev("phones")));
    assert_eq!(s.live.placement.get(&Holder::CartwallMain), None);
    assert_eq!(
        s.live.routes.get(&Holder::CartwallCue),
        Some(&Some(route_to("phones")))
    );
    assert_eq!(
        s.live.devices.get(&dev("default")),
        Some(&device_settings(&s.config.outputs, &dev("default")))
    );
    on_event(&mut s, EngineEvent::Gone { holder: Holder::PlayerMain(p) });
    assert!(!s.live.placement.contains_key(&Holder::PlayerMain(p)));
    assert!(!s.live.routes.contains_key(&Holder::PlayerMain(p)));
}

#[test]
fn a_device_no_holder_uses_any_more_is_forgotten_with_its_failure() {
    let mut s = fixture(1);
    let p = p0(&s);
    s.config.outputs.routes[0].main = Some(route_to("dac"));
    report_start(&mut s);
    let target = Target::Device(dev("dac"));
    let wanted = Wanted::Device(DeviceSettings {
        buffer_frames: 1024,
        ..device_settings(&s.config.outputs, &dev("dac"))
    });
    on_event(
        &mut s,
        EngineEvent::Applied {
            target: target.clone(),
            wanted: wanted.clone(),
            outcome: Err("refused".into()),
        },
    );
    assert_eq!(
        s.live.failures.get(&target),
        Some(&Failure {
            wanted,
            reason: "refused".into()
        })
    );
    // The holder moves to the default output: nothing uses dac any more.
    report(&mut s, Holder::PlayerMain(p), None, Some(dev("default")));
    assert!(!s.live.devices.contains_key(&dev("dac")));
    assert!(!s.live.failures.contains_key(&target));
}

#[test]
fn l14_an_accepted_value_becomes_the_running_one_and_clears_the_failure() {
    let mut s = fixture(1);
    report_start(&mut s);
    let d = dev("default");
    let target = Target::Device(d.clone());
    let settings = DeviceSettings {
        sample_rate: 44_100,
        ..device_settings(&s.config.outputs, &d)
    };
    on_event(
        &mut s,
        EngineEvent::Applied {
            target: target.clone(),
            wanted: Wanted::Device(settings),
            outcome: Err("refused".into()),
        },
    );
    assert_eq!(
        s.live.devices.get(&d).map(|r| r.sample_rate),
        Some(48_000),
        "a refusal keeps the running value"
    );
    on_event(
        &mut s,
        EngineEvent::Applied {
            target: target.clone(),
            wanted: Wanted::Device(settings),
            outcome: Ok(()),
        },
    );
    assert_eq!(s.live.devices.get(&d), Some(&settings));
    assert!(!s.live.failures.contains_key(&target));
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-model --test live`
Expected: FAIL to compile: `no variant named Placed found for enum EngineEvent`.

- [ ] **Step 3: Write the implementation**

In `crates/fp-model/src/command.rs`, add the imports `use crate::config::{Config, OutputDevice, Route};` (replace the existing `use crate::config::Config;`) and `use crate::live::{DeviceSettings, Holder, Target, Wanted};`. Change the derive above `pub enum EngineEvent` (L191) to `#[derive(Debug, Clone, PartialEq)]` and add these variants at the end of the enum (after `DsdEnded`):

```rust
    /// Live settings spec L3: the engine bound `holder` to the bus of
    /// `device` (a player added, a route or the audio system applied, the
    /// cartwall's first cart), holding `route`; the device runs with
    /// `running`.
    Placed {
        holder: Holder,
        route: Option<Route>,
        device: OutputDevice,
        running: DeviceSettings,
    },
    /// L3: the engine holds `route` for `holder`, which has no bus (a
    /// player without a usable Cue route, the cartwall before its first
    /// cart).
    Unplaced {
        holder: Holder,
        route: Option<Route>,
    },
    /// L3: a player was removed; its holders are forgotten.
    Gone { holder: Holder },
    /// The audio system the engine runs with (`outputs.backend`) and the
    /// backend it really chose; reported at start and after
    /// `ApplyAudioSystem`.
    AudioSystemInUse {
        configured: Option<String>,
        in_use: String,
    },
    /// L8, L14: the engine ran an `Apply…` action for `target`.
    Applied {
        target: Target,
        wanted: Wanted,
        outcome: Result<(), String>,
    },
```

Append to `crates/fp-model/src/live.rs`:

```rust
/// Forgets the running settings, in-flight actions and failures of every
/// device no holder is placed on any more.
fn forget_unused_devices(live: &mut LiveSettings) {
    let used: Vec<OutputDevice> = live.placement.values().cloned().collect();
    live.devices.retain(|d, _| used.contains(d));
    let unused = |t: &Target| matches!(t, Target::Device(d) if !used.contains(d));
    live.in_flight.retain(|t, _| !unused(t));
    live.failures.retain(|t, _| !unused(t));
}

/// L3: `EngineEvent::Placed`.
pub(crate) fn placed(
    state: &mut AppState,
    holder: Holder,
    route: Option<Route>,
    device: OutputDevice,
    running: DeviceSettings,
) {
    let live = &mut state.live;
    live.placement.insert(holder, device.clone());
    live.routes.insert(holder, route);
    live.devices.insert(device, running);
    forget_unused_devices(live);
}

/// L3: `EngineEvent::Unplaced`.
pub(crate) fn unplaced(state: &mut AppState, holder: Holder, route: Option<Route>) {
    let live = &mut state.live;
    live.placement.remove(&holder);
    live.routes.insert(holder, route);
    forget_unused_devices(live);
}

/// L3: `EngineEvent::Gone`.
pub(crate) fn gone(state: &mut AppState, holder: Holder) {
    let live = &mut state.live;
    live.placement.remove(&holder);
    live.routes.remove(&holder);
    live.in_flight.remove(&Target::Route(holder));
    live.failures.remove(&Target::Route(holder));
    forget_unused_devices(live);
}

/// `EngineEvent::AudioSystemInUse`.
pub(crate) fn audio_system_in_use(
    state: &mut AppState,
    configured: Option<String>,
    in_use: String,
) {
    state.live.audio_system = Some(configured);
    state.live.audio_system_in_use = Some(in_use);
}

/// L8, L14: `EngineEvent::Applied`. The action is no longer in flight when
/// it asked for `wanted` (a newer one stays). On `Ok` the value is the
/// running one; on `Err` it is recorded as refused and not tried again
/// until the wanted value changes or the operator presses Apply now.
pub(crate) fn applied(
    state: &mut AppState,
    target: Target,
    wanted: Wanted,
    outcome: Result<(), String>,
) {
    let live = &mut state.live;
    if live.in_flight.get(&target) == Some(&wanted) {
        live.in_flight.remove(&target);
    }
    match outcome {
        Ok(()) => {
            match (&target, &wanted) {
                (Target::AudioSystem, Wanted::AudioSystem(backend)) => {
                    live.audio_system = Some(backend.clone());
                }
                (Target::Route(holder), Wanted::Route(route)) => {
                    if let Some(running) = live.routes.get_mut(holder) {
                        *running = route.clone();
                    }
                }
                (Target::Device(device), Wanted::Device(settings)) => {
                    if let Some(running) = live.devices.get_mut(device) {
                        *running = *settings;
                    }
                }
                _ => {}
            }
            live.failures.remove(&target);
        }
        Err(reason) => {
            live.failures.insert(target, Failure { wanted, reason });
        }
    }
}
```

In `crates/fp-model/src/reducer.rs`, `on_event` (L335), add these arms at the end of the `match event { … }` (after the `CueEnded` arm, L433-439):

```rust
        EngineEvent::Placed {
            holder,
            route,
            device,
            running,
        } => crate::live::placed(state, holder, route, device, running),
        EngineEvent::Unplaced { holder, route } => crate::live::unplaced(state, holder, route),
        EngineEvent::Gone { holder } => crate::live::gone(state, holder),
        EngineEvent::AudioSystemInUse { configured, in_use } => {
            crate::live::audio_system_in_use(state, configured, in_use)
        }
        EngineEvent::Applied {
            target,
            wanted,
            outcome,
        } => crate::live::applied(state, target, wanted, outcome),
```

`EngineEvent` is no longer `Copy`. Build with `cargo build --workspace --all-targets`; where a moved event is used again, add `.clone()` at the first use (expected sites: none in `src/`; possibly test code that reuses one event value). Do not change behaviour.

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-model --test live`
Expected: PASS.

- [ ] **Step 5: Run the gate and commit**

Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`

```bash
git add -A crates/
git commit -m "$(cat <<'EOF'
feat(model): keep what the engine reports about its outputs

Placed, Unplaced, Gone, AudioSystemInUse and Applied (live settings
spec L3, L8, L14) fill AppState::live; a device no holder uses is
forgotten with its failure.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
EOF
)"
```

---

### Task 4: Pending output changes (spec §4.2, L5, L6, L7)

**Files:**
- Modify: `crates/fp-model/src/live.rs`
- Modify: `crates/fp-model/src/lib.rs`
- Test: `crates/fp-model/tests/live.rs`

**Interfaces:**
- Consumes: Tasks 1-3.
- Produces:
  - `pub enum PendingItem { AudioSystem { from: Option<String>, to: Option<String> }, Route { holder: Holder, from: Option<Route>, to: Option<Route> }, Device { device: OutputDevice, from: DeviceSettings, to: DeviceSettings }, Players { from: usize, to: usize }, CartPage { page: CartPageId, from: (u16, u16), to: (u16, u16) } }` — `Debug, Clone, PartialEq`
  - `impl PendingItem { pub fn target(&self) -> Option<Target>; pub fn wanted(&self) -> Option<Wanted> }` (`None` for `Players`, `CartPage`)
  - `pub struct Pending { pub item: PendingItem, pub causes: Vec<BusyCause>, pub failure: Option<String> }` — `Debug, Clone, PartialEq`
  - `pub fn pending(state: &AppState) -> Vec<Pending>` — audio system, then routes (holders in display order), then devices in use (in the order their first holder is listed); the limit items are added in Task 6.

- [ ] **Step 1: Write the failing tests**

Add `PendingItem, pending` to the `fp_model` import of `crates/fp-model/tests/live.rs`, add the helper and tests:

```rust
fn update(state: &mut AppState, edit: impl FnOnce(&mut Config)) -> Vec<fp_model::EngineAction> {
    let mut config = state.config.clone();
    edit(&mut config);
    apply(state, Command::UpdateConfig(Box::new(config))).unwrap()
}

fn device_item(state: &AppState, name: &str) -> Option<fp_model::Pending> {
    pending(state)
        .into_iter()
        .find(|p| matches!(&p.item, PendingItem::Device { device, .. } if *device == dev(name)))
}

#[test]
fn nothing_is_pending_while_the_engine_runs_the_configuration() {
    let mut s = fixture(1);
    assert!(pending(&s).is_empty(), "nothing reported");
    report_start(&mut s);
    assert!(pending(&s).is_empty());
}

#[test]
fn l5_a_busy_device_delays_only_its_own_change() {
    let mut s = fixture(3);
    let p1 = p0(&s);
    s.config.outputs.routes[0].main = Some(route_to("dac"));
    report_start(&mut s);
    apply(&mut s, Command::Play(p1)).unwrap();
    update(&mut s, |c| c.outputs.buffer_frames = 1024);
    let dac = device_item(&s, "dac").unwrap();
    assert_eq!(dac.causes, vec![BusyCause::PlayerPlaying(p1)]);
    let PendingItem::Device { from, to, .. } = dac.item else {
        panic!("a device item")
    };
    assert_eq!((from.buffer_frames, to.buffer_frames), (512, 1024));
    assert!(device_item(&s, "default").unwrap().causes.is_empty());
}

#[test]
fn l6_a_route_change_waits_for_its_own_holder_only() {
    let mut s = fixture(3);
    let (p1, p2) = (s.players[0].id, s.players[1].id);
    report_start(&mut s);
    apply(&mut s, Command::Play(p1)).unwrap();
    update(&mut s, |c| {
        c.outputs.routes[0].main = Some(route_to("dac"));
        c.outputs.routes[1].main = Some(route_to("dac"));
    });
    let route_of = |h: Holder| {
        pending(&s)
            .into_iter()
            .find(|p| matches!(p.item, PendingItem::Route { holder, .. } if holder == h))
            .unwrap()
    };
    assert_eq!(
        route_of(Holder::PlayerMain(p1)).causes,
        vec![BusyCause::PlayerPlaying(p1)]
    );
    assert!(
        route_of(Holder::PlayerMain(p2)).causes.is_empty(),
        "P2 is idle: moving it interrupts nobody (Q1)"
    );
}

#[test]
fn l7_the_audio_system_waits_until_nothing_plays_anywhere() {
    let mut s = fixture(3);
    let p1 = p0(&s);
    report_start(&mut s);
    let cart = load_cart(&mut s, 0);
    apply(&mut s, Command::Play(p1)).unwrap();
    apply(&mut s, Command::FireCart(cart)).unwrap();
    update(&mut s, |c| c.outputs.backend = Some("other".into()));
    let item = pending(&s)
        .into_iter()
        .find(|p| matches!(p.item, PendingItem::AudioSystem { .. }))
        .unwrap();
    assert_eq!(
        item.item,
        PendingItem::AudioSystem {
            from: None,
            to: Some("other".into())
        }
    );
    assert_eq!(
        item.causes,
        vec![BusyCause::PlayerPlaying(p1), BusyCause::CartPlaying(cart)]
    );
}

#[test]
fn a_device_no_holder_is_placed_on_is_never_pending() {
    let mut s = fixture(1);
    report_start(&mut s);
    update(&mut s, |c| c.outputs.set_device_buffer(&dev("unused"), Some(1024)));
    assert!(device_item(&s, "unused").is_none());
}

#[test]
fn a_removed_player_is_never_pending() {
    let mut s = fixture(1);
    let last = s.players[3].id;
    report_start(&mut s);
    update(&mut s, |c| c.outputs.routes[3].main = Some(route_to("dac")));
    apply(&mut s, Command::SetPlayerCount(3)).unwrap();
    assert!(
        !pending(&s)
            .iter()
            .any(|p| matches!(p.item, PendingItem::Route { holder: Holder::PlayerMain(h), .. } if h == last)),
        "the engine has not said Gone yet, but the player is gone"
    );
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-model --test live`
Expected: FAIL to compile: `cannot find function pending in crate fp_model`.

- [ ] **Step 3: Write the implementation**

Append to `crates/fp-model/src/live.rs`:

```rust
/// One pending change (spec §4.2), with the value it changes from and to.
#[derive(Debug, Clone, PartialEq)]
pub enum PendingItem {
    AudioSystem {
        from: Option<String>,
        to: Option<String>,
    },
    Route {
        holder: Holder,
        from: Option<Route>,
        to: Option<Route>,
    },
    Device {
        device: OutputDevice,
        from: DeviceSettings,
        to: DeviceSettings,
    },
    /// L18: more players than `limits.max_players`.
    Players { from: usize, to: usize },
    /// L19: a cart page over the grid limit, as (rows, columns).
    CartPage {
        page: CartPageId,
        from: (u16, u16),
        to: (u16, u16),
    },
}

impl PendingItem {
    /// The engine target of an output change; `None` for a limit.
    pub fn target(&self) -> Option<Target> {
        match self {
            Self::AudioSystem { .. } => Some(Target::AudioSystem),
            Self::Route { holder, .. } => Some(Target::Route(*holder)),
            Self::Device { device, .. } => Some(Target::Device(device.clone())),
            Self::Players { .. } | Self::CartPage { .. } => None,
        }
    }

    /// The value an output change asks for; `None` for a limit.
    pub fn wanted(&self) -> Option<Wanted> {
        match self {
            Self::AudioSystem { to, .. } => Some(Wanted::AudioSystem(to.clone())),
            Self::Route { to, .. } => Some(Wanted::Route(to.clone())),
            Self::Device { to, .. } => Some(Wanted::Device(*to)),
            Self::Players { .. } | Self::CartPage { .. } => None,
        }
    }
}

/// A pending item, what it waits for, and why the engine refused its
/// value last time (only while the same value is wanted).
#[derive(Debug, Clone, PartialEq)]
pub struct Pending {
    pub item: PendingItem,
    pub causes: Vec<BusyCause>,
    pub failure: Option<String>,
}

/// L7: the causes of every holder, wherever it plays.
fn all_causes(state: &AppState) -> Vec<BusyCause> {
    let mut out = Vec::new();
    collect_causes(state, holders(state).into_iter(), &mut out);
    out
}

/// The devices holders are placed on, each once, in the order of their
/// first holder in display order.
fn devices_in_use(state: &AppState) -> Vec<OutputDevice> {
    let mut out: Vec<OutputDevice> = Vec::new();
    for holder in holders(state) {
        if let Some(device) = state.live.placement.get(&holder)
            && !out.contains(device)
        {
            out.push(device.clone());
        }
    }
    out
}

/// A pending output item, with its failure when the same value was
/// refused.
fn output_item(state: &AppState, item: PendingItem, causes: Vec<BusyCause>) -> Pending {
    let failure = item.target().zip(item.wanted()).and_then(|(target, wanted)| {
        state
            .live
            .failures
            .get(&target)
            .filter(|f| f.wanted == wanted)
            .map(|f| f.reason.clone())
    });
    Pending {
        item,
        causes,
        failure,
    }
}

/// Spec §4.2: every change the engine does not run yet, in L9 order (the
/// audio system, the routes in display order, the devices), then the limit
/// reductions. A device no holder uses is never pending, nor is a holder
/// the engine has not reported (both open with the current configuration,
/// L13).
pub fn pending(state: &AppState) -> Vec<Pending> {
    let mut out = Vec::new();
    let live = &state.live;
    let configured = &state.config.outputs.backend;
    if let Some(running) = &live.audio_system
        && running != configured
    {
        let item = PendingItem::AudioSystem {
            from: running.clone(),
            to: configured.clone(),
        };
        out.push(output_item(state, item, all_causes(state)));
    }
    for holder in holders(state) {
        let Some(running) = live.routes.get(&holder) else {
            continue;
        };
        let wanted = configured_route(&state.config, holder);
        if *running != wanted {
            let item = PendingItem::Route {
                holder,
                from: running.clone(),
                to: wanted,
            };
            out.push(output_item(state, item, causes(state, holder)));
        }
    }
    for device in devices_in_use(state) {
        let Some(running) = live.devices.get(&device) else {
            continue;
        };
        let wanted = device_settings(&state.config.outputs, &device);
        if *running != wanted {
            let causes = device_causes(state, &device);
            let item = PendingItem::Device {
                from: *running,
                to: wanted,
                device,
            };
            out.push(output_item(state, item, causes));
        }
    }
    out
}
```

Extend the `live` re-export in `crates/fp-model/src/lib.rs` with `Pending, PendingItem, pending`.

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-model --test live`
Expected: PASS.

- [ ] **Step 5: Run the gate and commit**

Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`

```bash
git add crates/fp-model/src/live.rs crates/fp-model/src/lib.rs crates/fp-model/tests/live.rs
git commit -m "$(cat <<'EOF'
feat(model): list the output changes the engine does not run yet

pending() (live settings spec §4.2): a device waits on its own holders
(L5), a route on its holder (L6), the audio system on every holder
(L7).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
EOF
)"
```

---

### Task 5: Emitting the due changes (L8, L9, L10)

**Files:**
- Modify: `crates/fp-model/src/command.rs:239-327` (`EngineAction`)
- Modify: `crates/fp-model/src/live.rs`
- Modify: `crates/fp-model/src/reducer.rs:20-331` (`apply`), `335-443` (`on_event`), `1080-1106` (`update_config`)
- Modify: `crates/fp-engine/src/engine.rs:589-655` (`execute`)
- Modify: `crates/fp-model/src/lib.rs`
- Test: `crates/fp-model/tests/live.rs`

**Interfaces:**
- Consumes: `pending`, `PendingItem::{target, wanted}` (Task 4).
- Produces (new `EngineAction` variants):
  - `UpdateSettings(Box<Config>)` — L10
  - `ApplyAudioSystem { backend: Option<String>, force: bool }`
  - `ApplyRoute { holder: Holder, route: Option<Route>, force: bool }`
  - `ApplyDevice { device: OutputDevice, settings: DeviceSettings, force: bool }`
  - `pub fn due(state: &AppState) -> Vec<EngineAction>` (L8)
  - `pub fn target_and_wanted(action: &EngineAction) -> Option<(Target, Wanted)>` (used by the engine in plan 3)
  - `pub(crate) fn dispatch_due(state: &mut AppState, out: &mut Vec<EngineAction>)`
  - `fn action_for(item: &PendingItem, force: bool) -> Option<EngineAction>` (private, used again in Task 7)

- [ ] **Step 1: Write the failing tests**

Add `EngineAction, due` to the `fp_model` import of `crates/fp-model/tests/live.rs` and append:

```rust
fn apply_device(state: &AppState, name: &str, force: bool) -> EngineAction {
    EngineAction::ApplyDevice {
        device: dev(name),
        settings: device_settings(&state.config.outputs, &dev(name)),
        force,
    }
}

#[test]
fn l8_an_idle_change_is_sent_once_and_completes_with_applied() {
    let mut s = fixture(1);
    report_start(&mut s);
    let actions = update(&mut s, |c| c.outputs.sample_rate = 44_100);
    let wanted = apply_device(&s, "default", false);
    assert!(actions.contains(&wanted));
    assert!(actions.contains(&apply_device(&s, "phones", false)));
    let again = apply(&mut s, Command::SetCartwallOpen(true)).unwrap();
    assert!(!again.contains(&wanted), "in flight: not sent twice");
    assert!(due(&s).is_empty());
    let settings = device_settings(&s.config.outputs, &dev("default"));
    on_event(
        &mut s,
        EngineEvent::Applied {
            target: Target::Device(dev("default")),
            wanted: Wanted::Device(settings),
            outcome: Ok(()),
        },
    );
    assert!(device_item(&s, "default").is_none());
    assert!(!s.live.in_flight.contains_key(&Target::Device(dev("default"))));
}

#[test]
fn l8_a_refused_value_is_not_tried_again_until_the_setting_changes() {
    let mut s = fixture(1);
    report_start(&mut s);
    update(&mut s, |c| c.outputs.sample_rate = 44_100);
    let settings = device_settings(&s.config.outputs, &dev("default"));
    on_event(
        &mut s,
        EngineEvent::Applied {
            target: Target::Device(dev("default")),
            wanted: Wanted::Device(settings),
            outcome: Err("44100 Hz refused".into()),
        },
    );
    let item = device_item(&s, "default").unwrap();
    assert_eq!(item.failure.as_deref(), Some("44100 Hz refused"));
    let again = apply(&mut s, Command::SetCartwallOpen(true)).unwrap();
    assert!(!again.contains(&apply_device(&s, "default", false)));
    let actions = update(&mut s, |c| c.outputs.sample_rate = 96_000);
    assert!(actions.contains(&apply_device(&s, "default", false)));
    assert_eq!(device_item(&s, "default").unwrap().failure, None);
}

#[test]
fn l8_an_answer_for_an_older_value_keeps_the_newer_one_in_flight() {
    let mut s = fixture(1);
    report_start(&mut s);
    update(&mut s, |c| c.outputs.sample_rate = 44_100);
    let older = device_settings(&s.config.outputs, &dev("default"));
    update(&mut s, |c| c.outputs.sample_rate = 96_000);
    let newer = device_settings(&s.config.outputs, &dev("default"));
    on_event(
        &mut s,
        EngineEvent::Applied {
            target: Target::Device(dev("default")),
            wanted: Wanted::Device(older),
            outcome: Ok(()),
        },
    );
    assert_eq!(s.live.devices.get(&dev("default")), Some(&older));
    assert_eq!(
        s.live.in_flight.get(&Target::Device(dev("default"))),
        Some(&Wanted::Device(newer))
    );
}

#[test]
fn l9_the_audio_system_goes_first_then_routes_then_devices() {
    let mut s = fixture(1);
    report_start(&mut s);
    let actions = update(&mut s, |c| {
        c.outputs.backend = Some("other".into());
        c.outputs.routes[0].main = Some(route_to("dac"));
        c.outputs.buffer_frames = 1024;
    });
    let kinds: Vec<u8> = actions
        .iter()
        .filter_map(|a| match a {
            EngineAction::ApplyAudioSystem { .. } => Some(0),
            EngineAction::ApplyRoute { .. } => Some(1),
            EngineAction::ApplyDevice { .. } => Some(2),
            _ => None,
        })
        .collect();
    let mut sorted = kinds.clone();
    sorted.sort_unstable();
    assert_eq!(kinds, sorted);
    assert_eq!(kinds.first(), Some(&0));
    assert!(kinds.contains(&1) && kinds.contains(&2));
}

#[test]
fn l10_update_config_tells_the_engine_only_when_outputs_or_tuning_change() {
    let mut s = fixture(1);
    let actions = update(&mut s, |c| c.players.fade_ms = 2500);
    assert!(
        !actions
            .iter()
            .any(|a| matches!(a, EngineAction::UpdateSettings(_)))
    );
    let actions = update(&mut s, |c| c.outputs.buffer_frames = 1024);
    assert!(actions.contains(&EngineAction::UpdateSettings(Box::new(s.config.clone()))));
    let actions = update(&mut s, |c| c.tuning.prebuffer_secs = 8.0);
    assert!(actions.iter().any(
        |a| matches!(a, EngineAction::UpdateSettings(c) if c.tuning.prebuffer_secs == 8.0)
    ));
}

#[test]
fn changing_a_setting_back_while_it_waits_leaves_nothing_pending() {
    let mut s = fixture(3);
    let p1 = p0(&s);
    report_start(&mut s);
    apply(&mut s, Command::Play(p1)).unwrap();
    update(&mut s, |c| c.outputs.buffer_frames = 1024);
    assert!(!device_item(&s, "default").unwrap().causes.is_empty());
    let actions = update(&mut s, |c| c.outputs.buffer_frames = 512);
    assert!(device_item(&s, "default").is_none());
    assert!(
        !actions
            .iter()
            .any(|a| matches!(a, EngineAction::ApplyDevice { device, .. } if *device == dev("default")))
    );
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-model --test live`
Expected: FAIL to compile: `no variant named ApplyDevice found for enum EngineAction`.

- [ ] **Step 3: Write the implementation**

In `crates/fp-model/src/command.rs`, add `use crate::live::{DeviceSettings, Holder, Target, Wanted};` if not already there, and append to `pub enum EngineAction` (after `StopCartCue,`):

```rust
    /// L10: the configuration the engine takes for new buses and new
    /// holders, and the tuning it reads live. It changes no open bus.
    UpdateSettings(Box<Config>),
    /// L17: change the audio system the default output uses. Not forced,
    /// it waits until every bus is quiet (L11).
    ApplyAudioSystem {
        backend: Option<String>,
        force: bool,
    },
    /// L16: move `holder` to `route`. Not forced, it waits until the
    /// holder's own sources are quiet (L6, L11).
    ApplyRoute {
        holder: Holder,
        route: Option<Route>,
        force: bool,
    },
    /// L12: run `device` with `settings`. Not forced, it waits until the
    /// device's bus is quiet (L11).
    ApplyDevice {
        device: OutputDevice,
        settings: DeviceSettings,
        force: bool,
    },
```

Append to `crates/fp-model/src/live.rs` (add `use crate::command::EngineAction;`):

```rust
/// The engine action that applies an output item.
fn action_for(item: &PendingItem, force: bool) -> Option<EngineAction> {
    match item {
        PendingItem::AudioSystem { to, .. } => Some(EngineAction::ApplyAudioSystem {
            backend: to.clone(),
            force,
        }),
        PendingItem::Route { holder, to, .. } => Some(EngineAction::ApplyRoute {
            holder: *holder,
            route: to.clone(),
            force,
        }),
        PendingItem::Device { device, to, .. } => Some(EngineAction::ApplyDevice {
            device: device.clone(),
            settings: *to,
            force,
        }),
        PendingItem::Players { .. } | PendingItem::CartPage { .. } => None,
    }
}

/// What an `Apply…` action changes and the value it asks for.
pub fn target_and_wanted(action: &EngineAction) -> Option<(Target, Wanted)> {
    match action {
        EngineAction::ApplyAudioSystem { backend, .. } => {
            Some((Target::AudioSystem, Wanted::AudioSystem(backend.clone())))
        }
        EngineAction::ApplyRoute { holder, route, .. } => {
            Some((Target::Route(*holder), Wanted::Route(route.clone())))
        }
        EngineAction::ApplyDevice {
            device, settings, ..
        } => Some((Target::Device(device.clone()), Wanted::Device(*settings))),
        _ => None,
    }
}

/// L8: the output items with no cause whose value is neither in flight nor
/// refused already, in L9 order.
pub fn due(state: &AppState) -> Vec<EngineAction> {
    let live = &state.live;
    pending(state)
        .iter()
        .filter(|p| p.causes.is_empty())
        .filter_map(|p| action_for(&p.item, false))
        .filter(|action| {
            target_and_wanted(action).is_some_and(|(target, wanted)| {
                live.in_flight.get(&target) != Some(&wanted)
                    && live.failures.get(&target).map(|f| &f.wanted) != Some(&wanted)
            })
        })
        .collect()
}

/// Sends `action`, recording it in flight.
fn send(state: &mut AppState, action: EngineAction, out: &mut Vec<EngineAction>) {
    if let Some((target, wanted)) = target_and_wanted(&action) {
        state.live.in_flight.insert(target, wanted);
    }
    out.push(action);
}

/// L8: called at the end of every `apply` and `on_event`.
pub(crate) fn dispatch_due(state: &mut AppState, out: &mut Vec<EngineAction>) {
    for action in due(state) {
        send(state, action, out);
    }
}
```

Extend the `live` re-export in `crates/fp-model/src/lib.rs` with `due, target_and_wanted`.

In `crates/fp-model/src/reducer.rs`:
- At the end of `apply` (L328-330), replace

```rust
    fill_empty_next(state);
    reconcile(state, &mut out);
    Ok(out)
```

with

```rust
    fill_empty_next(state);
    reconcile(state, &mut out);
    crate::live::dispatch_due(state, &mut out);
    Ok(out)
```

- At the end of `on_event` (L440-442), replace

```rust
    fill_empty_next(state);
    reconcile(state, &mut out);
    out
```

with

```rust
    fill_empty_next(state);
    reconcile(state, &mut out);
    crate::live::dispatch_due(state, &mut out);
    out
```

- In `update_config` (L1087-1106), insert before `state.config = config;`:

```rust
    if config.outputs != state.config.outputs || config.tuning != state.config.tuning {
        out.push(EngineAction::UpdateSettings(Box::new(config.clone())));
    }
```

In `crates/fp-engine/src/engine.rs`, `execute` (L589), add this arm at the end of the `match action { … }` (after `LoadPaused`). Plans 2 and 3 replace it:

```rust
            // Live settings: the engine takes them from plan 2 (settings)
            // and plan 3 (applies) of the live settings work. Until then it
            // reports no placement, so the model never sends an apply.
            EngineAction::UpdateSettings(_)
            | EngineAction::ApplyAudioSystem { .. }
            | EngineAction::ApplyRoute { .. }
            | EngineAction::ApplyDevice { .. } => {}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-model --test live`
Expected: PASS.

- [ ] **Step 5: Run the gate and commit**

Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`

```bash
git add -A crates/
git commit -m "$(cat <<'EOF'
feat(model): send output changes as soon as nothing they affect plays

due() and the Apply actions (live settings spec L8, L9), and
UpdateSettings when the outputs or the tuning change (L10). The engine
takes them as no-ops until it reports its placements.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
EOF
)"
```

---

### Task 6: Dropped (L18, L19)

Dropped by the maintainer ruling above. Only L23 (every other limit applies at once) keeps a test, added in Task 5: `l23_limits_never_make_anything_pending`.

---

### Task 7: Apply now and what it interrupts (L20, L22, L24)

**Files:**
- Modify: `crates/fp-model/src/command.rs:13-189` (`Command`)
- Modify: `crates/fp-model/src/live.rs`
- Modify: `crates/fp-model/src/reducer.rs:20-326` (`apply` match)
- Modify: `crates/fp-model/src/lib.rs`
- Test: `crates/fp-model/tests/live.rs`

**Interfaces:**
- Consumes: `action_for`, `send` (Task 5), `pending` (Task 4).
- Produces:
  - `Command::ApplySettingsNow`
  - `pub(crate) fn apply_now(state: &mut AppState, out: &mut Vec<EngineAction>)` (L20)
  - `pub fn interruptions(state: &AppState) -> Vec<(Target, Vec<BusyCause>)>` (L22)
  - `pub fn has_output_items(state: &AppState) -> bool` — whether Apply now can do anything (used by the UI in plan 5).

- [ ] **Step 1: Write the failing tests**

Add `has_output_items, interruptions` to the `fp_model` import and `roundtrip` to the `common` import (`use common::{entries, fixture, p0, roundtrip};`), and append:

```rust
#[test]
fn l20_apply_now_forces_every_output_change_but_never_a_limit() {
    let mut s = fixture(3);
    let last = s.players[3].id;
    report_start(&mut s);
    apply(&mut s, Command::Play(last)).unwrap();
    update(&mut s, |c| {
        c.outputs.buffer_frames = 1024;
        c.limits.max_players = 3;
    });
    let actions = apply(&mut s, Command::ApplySettingsNow).unwrap();
    assert!(actions.contains(&apply_device(&s, "default", true)));
    assert!(
        actions.contains(&apply_device(&s, "phones", true)),
        "in flight already: sent again, forced"
    );
    assert!(
        !actions
            .iter()
            .any(|a| matches!(a, EngineAction::RemovePlayer { .. }))
    );
    assert_eq!(s.players.len(), 4, "L18 is never forced");
}

#[test]
fn l20_apply_now_retries_a_refused_value() {
    let mut s = fixture(1);
    report_start(&mut s);
    update(&mut s, |c| c.outputs.sample_rate = 44_100);
    let settings = device_settings(&s.config.outputs, &dev("default"));
    on_event(
        &mut s,
        EngineEvent::Applied {
            target: Target::Device(dev("default")),
            wanted: Wanted::Device(settings),
            outcome: Err("refused".into()),
        },
    );
    let actions = apply(&mut s, Command::ApplySettingsNow).unwrap();
    assert!(actions.contains(&apply_device(&s, "default", true)));
}

#[test]
fn l22_interruptions_name_only_what_apply_now_would_cut() {
    let mut s = fixture(3);
    let (p1, last) = (p0(&s), s.players[3].id);
    report_start(&mut s);
    apply(&mut s, Command::Play(p1)).unwrap();
    apply(&mut s, Command::Play(last)).unwrap();
    update(&mut s, |c| c.limits.max_players = 3);
    assert!(interruptions(&s).is_empty(), "a limit is not forced");
    assert!(!has_output_items(&s));
    update(&mut s, |c| c.outputs.buffer_frames = 1024);
    assert_eq!(
        interruptions(&s),
        vec![(
            Target::Device(dev("default")),
            vec![BusyCause::PlayerPlaying(p1), BusyCause::PlayerPlaying(last)]
        )]
    );
    assert!(has_output_items(&s));
}

#[test]
fn l24_nothing_pending_is_saved() {
    let mut s = fixture(3);
    let p1 = p0(&s);
    report_start(&mut s);
    apply(&mut s, Command::Play(p1)).unwrap();
    update(&mut s, |c| c.outputs.buffer_frames = 1024);
    assert!(!pending(&s).is_empty());
    let restored = roundtrip(&s);
    assert_eq!(restored.live, LiveSettings::default());
    assert!(pending(&restored).is_empty());
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-model --test live`
Expected: FAIL to compile: `no variant named ApplySettingsNow found for enum Command`.

- [ ] **Step 3: Write the implementation**

In `crates/fp-model/src/command.rs`, add to `pub enum Command` after `RestoreDefaults(…)`:

```rust
    /// Live settings spec L20: every pending output change now, forced,
    /// even where it briefly interrupts the audio on a device. Never the
    /// limit reductions.
    ApplySettingsNow,
```

Append to `crates/fp-model/src/live.rs`:

```rust
/// L20: every pending output item, forced, including those in flight or
/// refused; never L18 or L19.
pub(crate) fn apply_now(state: &mut AppState, out: &mut Vec<EngineAction>) {
    let actions: Vec<EngineAction> = pending(state)
        .iter()
        .filter_map(|p| action_for(&p.item, true))
        .collect();
    for action in actions {
        send(state, action, out);
    }
}

/// L22: the output items Apply now would force that have causes. The UI
/// asks for confirmation if and only if this is not empty.
pub fn interruptions(state: &AppState) -> Vec<(Target, Vec<BusyCause>)> {
    pending(state)
        .into_iter()
        .filter(|p| !p.causes.is_empty())
        .filter_map(|p| p.item.target().map(|t| (t, p.causes)))
        .collect()
}

/// Whether Apply now has anything to apply: an output item is pending.
pub fn has_output_items(state: &AppState) -> bool {
    pending(state).iter().any(|p| p.item.target().is_some())
}
```

In `crates/fp-model/src/reducer.rs`, add to the `match command` of `apply`, after the `ResetMarkers` arm:

```rust
        Command::ApplySettingsNow => crate::live::apply_now(state, &mut out),
```

Extend the `live` re-export in `crates/fp-model/src/lib.rs` with `has_output_items, interruptions`.

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-model --test live`
Expected: PASS.

- [ ] **Step 5: Run the gate and commit**

Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`

```bash
git add crates/fp-model/src/command.rs crates/fp-model/src/live.rs crates/fp-model/src/reducer.rs \
  crates/fp-model/src/lib.rs crates/fp-model/tests/live.rs
git commit -m "$(cat <<'EOF'
feat(model): apply output changes now on request

ApplySettingsNow forces every pending output change, never a limit
(live settings spec L20); interruptions() lists what it would cut
(L22). Nothing pending survives a restore (L24).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
EOF
)"
```

---

## Self-Review (done while writing)

- **Spec coverage:** L1 (Task 2), L2 (Task 2), L3 model side (Task 3), L4 (Task 1), L5–L7 (Task 4), L8–L10 (Task 5), L14 model side (Task 3, Task 5), L18, L19, L23 (Task 6), L20, L22, L24 (Task 7). L11–L13, L15–L17, L21 are engine rules: plans 2 and 3. §6 limits forwarding: plan 4. §9: plan 5.
- **Placeholders:** none; the one compile-fix instruction (Task 3, Step 3) names what to do and why.
- **Type consistency:** `Holder`, `Target`, `Wanted`, `DeviceSettings`, `PendingItem`, `Pending`, `BusyCause` names and fields match across tasks; `target_and_wanted` is the name plan 3 consumes.
- **Review Focus:** every line has its test in the owning task.
