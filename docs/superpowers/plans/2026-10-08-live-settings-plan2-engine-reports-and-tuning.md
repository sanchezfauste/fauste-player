# Live Settings, Plan 2: Engine Reports and Live Tuning

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The engine tells the model what it runs with (audio system, where each holder plays, each open device's settings: L3), keeps those settings per open bus, takes a new configuration with `UpdateSettings` without touching an open bus (L10, L13), and reads every tuning value live (§7), mixer values through `BusCommand::Tune`.

**Architecture:** `Engine` gains `audio_system`/`backend_in_use` (the default-backend choice moves from `main.rs` into `Engine::new`) and `running: HashMap<BusKey, DeviceSettings>`; every read that decided something for an open bus from `EngineSettings` (bit-perfect, DSD mode, mix, silence, rate) reads the bus's running settings instead, so a new configuration reaches an open bus only through `ApplyDevice` (plan 3). Placements are pushed into `Engine::events`, which `tick` already hands to the model. Tuning: a new `BusCommand::Tune(MixerConfig)` (a `Copy` value: no allocation on the callback), `Bus::set_timing`, a ready threshold per load, and a conductor tick period read each loop.

**Tech Stack:** Rust 2024, `fp-engine`, `fp-backends` (`choose_default_backend`, `OfflineBackend`), `rtrb` command queues, `assert_no_alloc` in the mixer tests.

**Spec:** `docs/superpowers/specs/2026-10-07-live-settings-design.md` (read it whole, including §14 and the "Planning notes"). Plan 1 (`2026-10-08-live-settings-plan1-model-rules.md`) must be merged first: it defines `Holder`, `DeviceSettings`, the events and the actions used here.

## Global Constraints

- All code, identifiers, comments, docs and commit messages in English; never mention other playout or radio-automation products.
- Real-time safety (CLAUDE.md rule 5): the mixer and the render path never allocate, free, lock (only `try_lock` on the bus mixer), log, do I/O or panic. Memory enters through `BusCommand` and leaves through `Retired`. `MixerConfig` is `Copy`; `Tune` carries it by value.
- L10: `UpdateSettings` changes no open bus. L13: a holder or bus created after a change uses the current configuration.
- §7: no tuning field waits; "next use" means the value is read when the engine next does that thing; work already scheduled keeps the value it was given. Frame values in `Tune` are computed at each bus's running rate.
- `fp-engine` denies `clippy::indexing_slicing` (use `get`); `unwrap`/`expect`/`panic` denied outside tests; `unsafe_code` forbidden.
- Engine tests use the `Offline` backend and drive time explicitly (`now: Instant`); never sleep to wait for audio (a bounded wait for decode workers, as the existing rigs do, is fine).
- Conventional Commits; every commit ends with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Gate for every task: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`.

## Review Focus

1. **A new bit-perfect or DSD setting while a device is open and busy**: the open bus must keep following its *running* settings (no file-rate following, no DSD start) until `ApplyDevice`; a read left on `EngineSettings` would apply the change under the audio. Test: Task 3, `an_open_bus_keeps_its_running_bit_perfect_setting`.
2. **A `Tune` that does not fit a full command queue**: it is sent again next tick, not lost. Test: Task 3, `a_tune_waits_for_room_in_the_command_queue` (a unit test in `engine.rs` that fills the bus's command queue).
3. **The configured audio system is not usable on this machine** (a config copied from another OS): `Engine::new` falls back exactly as `main.rs` did, and reports both the configured and the chosen backend. Test: Task 1, `an_unusable_configured_audio_system_falls_back_and_says_so`.
4. **Engine tests that asserted "no events"** now see placement reports: only the placement events are filtered out; any other event still fails them. Task 1, Step 5.
5. **A ready threshold changed while a source is being filled**: the source keeps the threshold it was opened with; the next one takes the new one. Test: Task 3, `a_source_becomes_ready_at_the_threshold_it_was_opened_with`.

---

## File Structure

| File | Responsibility |
|---|---|
| `crates/fp-engine/src/bus.rs` | `BusKey` ⇄ `OutputDevice`; `Bus::set_timing` |
| `crates/fp-engine/src/engine.rs` | Default backend choice, running settings per bus, placement reports, `UpdateSettings`, `Tune` sending, `mixer_config`, `bus_timing` |
| `crates/fp-engine/src/engine/carts.rs` | Cartwall placement report; ready threshold per cart load |
| `crates/fp-engine/src/engine/dsd.rs` | DSD mode, mix, silence and PCM rate read from the bus's running settings |
| `crates/fp-engine/src/mixer.rs` | `BusCommand::Tune` |
| `crates/fp-engine/src/worker.rs` | `LoadOptions::ready_frames` |
| `crates/fp-engine/src/conductor.rs` | `Conductor::tick_period`, `spawn` reads it each loop |
| `crates/fp-app/src/main.rs` | No more backend choice; `spawn` without a period |
| `crates/fp-engine/tests/live_settings.rs` (new) | Engine-level live settings tests (extended in plan 3) |
| `crates/fp-engine/tests/{engine,cartwall,conductor,bus,mixer,worker}.rs` | Adjusted and new tests |
| `docs/technical/audio-engine.md` | "Live settings" section (placements, running settings, tuning) |

---

### Task 1: Placements, running settings per bus, and the audio system in the engine (L3, L4, L13)

**Files:**
- Modify: `crates/fp-engine/src/bus.rs:27-32` (`BusKey`)
- Modify: `crates/fp-engine/src/engine.rs:13-16` (imports), `76-158` (`EngineSettings`), `301-334` (`Engine` fields), `369-406` (`Engine::new`), `561-583` (`is_bit_perfect`), `694-719` (`prepare_start`), `947-960` (`default_backend`), `1058-1126` (`ensure_bus`), `1196-1261` (`add_player`, `remove_player`)
- Modify: `crates/fp-engine/src/engine/carts.rs:66-114` (`ensure_cartwall`)
- Modify: `crates/fp-engine/src/engine/dsd.rs:69-71` (`silence_frames`), `84-90` (`try_start_dsd` mode), `337-342` (`dsd_slot_started`), `362-375` (`before_start_on`), `567-570` (`reopen_native_as_pcm`)
- Modify: `crates/fp-app/src/main.rs:18-20` (imports), `152-196` (`run`)
- Modify: `crates/fp-engine/tests/engine.rs:1014-1034`, `crates/fp-engine/tests/cartwall.rs:151-161`
- Create: `crates/fp-engine/tests/live_settings.rs`
- Test: `crates/fp-engine/tests/conductor.rs`

**Interfaces:**
- Consumes (plan 1): `fp_model::{Holder, DeviceSettings, device_settings, EngineEvent::{Placed, Unplaced, Gone, AudioSystemInUse}}`.
- Produces:
  - `impl From<&fp_model::OutputDevice> for BusKey`; `BusKey::output_device(&self) -> fp_model::OutputDevice`
  - `EngineSettings::device_settings(&self, key: &BusKey) -> fp_model::DeviceSettings` (equal to `fp_model::device_settings` for the same configuration)
  - `Engine::backend_in_use(&self) -> &str`
  - `Engine::running_settings(&self, device: &OutputDevice) -> Option<DeviceSettings>` (an open bus's settings)
  - private: `Engine::device_of(&self, bus: &BusKey) -> DeviceSettings`, `Engine::report_placement(&mut self, holder, route: Option<Route>, bus: Option<&BusKey>)`, `fn choose_backend(backends, configured) -> String`
  - Engine fields used by plan 3: `audio_system: Option<String>`, `backend_in_use: String`, `running: HashMap<BusKey, DeviceSettings>`

- [ ] **Step 1: Write the failing tests**

Create `crates/fp-engine/tests/live_settings.rs`:

```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    // The rig serves the tests of live settings plans 2 and 3.
    dead_code
)]
//! Live settings spec (2026-10-07): what the engine reports about where
//! each holder plays (L3), the settings it takes while running (L10, §7),
//! and how it applies an output change (L11–L17, L21).

mod support;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use fp_backends::{AudioBackend, OfflineBackend, OfflineDevice};
use fp_engine::bus::BusKey;
use fp_engine::engine::{Engine, EngineSettings};
use fp_model::{
    CartId, CartRequest, Config, DsdDevice, DsdMix, DsdOutput, EngineAction, EngineEvent, EntryId,
    Holder, OutputDevice, PlayerId, PlayerRoutes, Route, SOURCE_END, SourceRequest, TrackId,
    device_settings,
};
use support::tagged_opener;

const BLOCK: usize = 480;
const P: PlayerId = PlayerId(1);

fn route(device: &str) -> Route {
    Route {
        backend: "offline".into(),
        device: device.into(),
        first_channel: 0,
    }
}

fn out(device: &str) -> OutputDevice {
    OutputDevice {
        backend: "offline".into(),
        device: device.into(),
    }
}

/// P's Main on `main`, no Cue; the cartwall on the default output (`main`,
/// the first Offline device by name).
fn config() -> Config {
    let mut c = Config::default();
    c.outputs.backend = Some("offline".into());
    c.outputs.buffer_frames = BLOCK as u32;
    c.outputs.routes = vec![PlayerRoutes {
        player: P,
        main: Some(route("main")),
        cue: None,
    }];
    c.tuning.gain_smoothing_ms = 0.0;
    c
}

fn request(track: u64, from_secs: f64) -> SourceRequest {
    SourceRequest {
        entry: EntryId(track),
        track: TrackId(track),
        path: PathBuf::from(format!("track{track}")),
        from_secs,
        format: None,
    }
}

struct Rig {
    engine: Engine,
    main: OfflineDevice,
    other: OfflineDevice,
    clock: Instant,
    events: Vec<EngineEvent>,
}

/// Two Offline devices, `main` (2 channels) and `other` (4), and P added.
fn rig_with(config: &Config) -> Rig {
    let backend = OfflineBackend::new();
    let main = backend.add_device("main", 2);
    let other = backend.add_device("other", 4);
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
    let mut engine = Engine::new(
        backends,
        EngineSettings::from_config(config),
        tagged_opener(48_000 * 20),
    );
    let clock = Instant::now();
    engine.execute(EngineAction::AddPlayer { player: P }, clock);
    Rig {
        engine,
        main,
        other,
        clock,
        events: Vec::new(),
    }
}

fn rig() -> Rig {
    rig_with(&config())
}

impl Rig {
    fn act(&mut self, action: EngineAction) {
        self.engine.execute(action, self.clock);
    }

    fn tick(&mut self) {
        let events = self.engine.tick(self.clock);
        self.events.extend(events);
    }

    /// Waits (bounded) for the decode workers, never for audio.
    fn settle(&mut self) {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            self.tick();
            if self.engine.unsettled_sources() == 0 || Instant::now() > deadline {
                return;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    /// Renders `blocks` blocks on every open device, ticking after each;
    /// returns the left channel `main` played.
    fn run(&mut self, blocks: usize) -> Vec<f32> {
        let mut heard = Vec::new();
        for _ in 0..blocks {
            if let Some(samples) = self.main.render(BLOCK) {
                heard.extend(samples.chunks(2).map(|f| f[0]));
            }
            let _ = self.other.render(BLOCK);
            self.clock += Duration::from_millis(10);
            self.tick();
            // Gives the decode workers time to keep the rings topped up.
            std::thread::sleep(Duration::from_millis(1));
        }
        heard
    }

    fn take_events(&mut self) -> Vec<EngineEvent> {
        std::mem::take(&mut self.events)
    }
}

#[test]
fn l3_the_engine_reports_its_audio_system_and_where_each_holder_plays() {
    let mut r = rig();
    r.tick();
    let events = r.take_events();
    let c = config();
    assert!(events.contains(&EngineEvent::AudioSystemInUse {
        configured: Some("offline".into()),
        in_use: "offline".into(),
    }));
    assert!(events.contains(&EngineEvent::Placed {
        holder: Holder::PlayerMain(P),
        route: Some(route("main")),
        device: out("main"),
        running: device_settings(&c.outputs, &out("main")),
    }));
    assert!(events.contains(&EngineEvent::Unplaced {
        holder: Holder::PlayerCue(P),
        route: None,
    }));
    assert!(events.contains(&EngineEvent::Unplaced {
        holder: Holder::CartwallMain,
        route: None,
    }));
    assert_eq!(
        r.engine.running_settings(&out("main")),
        Some(device_settings(&c.outputs, &out("main")))
    );
    assert_eq!(r.engine.running_settings(&out("other")), None, "not open");
}

#[test]
fn l3_the_first_cart_places_the_cartwall_and_a_removed_player_is_gone() {
    let mut r = rig();
    r.tick();
    r.take_events();
    r.act(EngineAction::StartCart(CartRequest {
        cart: CartId(1),
        track: TrackId(9),
        path: PathBuf::from("track9"),
        from_secs: 0.0,
        until_secs: SOURCE_END,
        looped: false,
        format: None,
    }));
    r.tick();
    let events = r.take_events();
    assert!(events.iter().any(|e| matches!(
        e,
        EngineEvent::Placed { holder: Holder::CartwallMain, route: None, device, .. }
            if *device == out("main")
    )));
    assert!(events.contains(&EngineEvent::Unplaced {
        holder: Holder::CartwallCue,
        route: None,
    }));
    r.act(EngineAction::RemovePlayer { player: P });
    r.tick();
    let events = r.take_events();
    assert!(events.contains(&EngineEvent::Gone {
        holder: Holder::PlayerMain(P)
    }));
    assert!(events.contains(&EngineEvent::Gone {
        holder: Holder::PlayerCue(P)
    }));
}

#[test]
fn the_engine_and_the_model_agree_on_a_devices_settings() {
    let edits: [fn(&mut Config); 4] = [
        |_| {},
        |c| c.outputs.set_device_rate(&out("main"), Some(96_000)),
        |c| {
            c.outputs.bit_perfect = vec![out("main")];
            c.outputs.dsd_output = vec![DsdDevice {
                backend: "offline".into(),
                device: "main".into(),
                mode: DsdOutput::Dop,
            }];
            c.outputs.dsd_mix = DsdMix::HoldOthers;
        },
        // A device no route names opens at the global values.
        |c| c.outputs.set_device_buffer(&out("other"), Some(256)),
    ];
    for edit in edits {
        let mut c = config();
        edit(&mut c);
        let settings = EngineSettings::from_config(&c);
        for d in [out("main"), out("other")] {
            assert_eq!(
                settings.device_settings(&BusKey::from(&d)),
                device_settings(&c.outputs, &d),
                "{d:?}"
            );
        }
    }
}

#[test]
fn an_unusable_configured_audio_system_falls_back_and_says_so() {
    let mut c = config();
    c.outputs.backend = Some("missing".into());
    let mut r = rig_with(&c);
    assert_eq!(r.engine.backend_in_use(), "offline");
    r.tick();
    assert!(r.take_events().contains(&EngineEvent::AudioSystemInUse {
        configured: Some("missing".into()),
        in_use: "offline".into(),
    }));
}
```

Append to `crates/fp-engine/tests/conductor.rs` (add `Holder` to the `fp_model` import):

```rust
#[test]
fn the_model_learns_where_each_holder_plays() {
    let (mut conductor, _handle, _device, now) = offline_conductor(model(1, 1));
    let p = conductor.state().players[0].id;
    conductor.tick(now);
    let live = &conductor.state().live;
    assert_eq!(live.audio_system_in_use.as_deref(), Some("offline"));
    assert_eq!(
        live.placement
            .get(&Holder::PlayerMain(p))
            .map(|d| d.device.as_str()),
        Some("main")
    );
    assert!(fp_model::pending(conductor.state()).is_empty());
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-engine --test live_settings --test conductor`
Expected: FAIL to compile: `no method named running_settings`, `no function or associated item named from found for struct BusKey`.

- [ ] **Step 3: Write the implementation**

In `crates/fp-engine/src/bus.rs`, after `pub struct BusKey { … }` (L29-32):

```rust
impl From<&fp_model::OutputDevice> for BusKey {
    fn from(device: &fp_model::OutputDevice) -> Self {
        Self {
            backend: device.backend.clone(),
            device: device.device.clone(),
        }
    }
}

impl BusKey {
    /// The model's name for this device.
    pub fn output_device(&self) -> fp_model::OutputDevice {
        fp_model::OutputDevice {
            backend: self.backend.clone(),
            device: self.device.clone(),
        }
    }
}
```

In `crates/fp-engine/src/engine.rs`:

1. Imports (L12-16): `use fp_backends::{AudioBackend, Availability, NullBackend, StreamConfig, choose_default_backend};` and

```rust
use fp_model::{
    CartwallRoutes, Config, DeviceSettings, EngineAction, EngineEvent, EntryId, Holder,
    OutputDevice, PlayerId, PlayerRoutes, Route, SourceRequest, TransitionPlan, Tuning,
};
```

2. In `impl EngineSettings`, after `buffer_for` (L153):

```rust
    /// What a bus for `key` opens with under these settings (live settings
    /// spec L4; the same as `fp_model::device_settings` for the
    /// configuration they came from).
    pub fn device_settings(&self, key: &BusKey) -> DeviceSettings {
        let bit_perfect = self.bit_perfect.contains(key);
        let dsd = if bit_perfect {
            self.dsd
                .modes
                .get(key)
                .copied()
                .unwrap_or(fp_model::DsdOutput::Pcm)
        } else {
            fp_model::DsdOutput::Pcm
        };
        let carries_dsd = dsd != fp_model::DsdOutput::Pcm;
        DeviceSettings {
            sample_rate: self.rate_for(key),
            buffer_frames: self.buffer_for(key),
            bit_perfect,
            dsd,
            dsd_mix: carries_dsd.then_some(self.dsd.mix),
            dsd_silence_ms: carries_dsd.then_some(self.dsd.silence_ms),
        }
    }
```

Update the doc comment of `EngineSettings::default_backend` (L66-67) to: `/// The configured audio system (\`outputs.backend\`); the engine chooses the one it uses (\`Engine::backend_in_use\`).`

3. Add to `pub struct Engine` (after `dsd_buses`, L333):

```rust
    /// The configured audio system the default output runs with
    /// (`outputs.backend`), and the backend really chosen for it (live
    /// settings spec L17). Only `ApplyAudioSystem` changes them.
    audio_system: Option<String>,
    backend_in_use: String,
    /// What each open bus runs with (L4). A new configuration reaches an
    /// open bus only through `ApplyDevice`.
    running: HashMap<BusKey, DeviceSettings>,
```

4. Add this free function after `frames_at` (L365-367):

```rust
/// The backend the default output uses (L17): the configured one when it
/// is available here, else the preferred available one, else Null.
fn choose_backend(backends: &[Arc<dyn AudioBackend>], configured: Option<&str>) -> String {
    let listed: Vec<(String, bool)> = backends
        .iter()
        .map(|b| (b.id().0, b.availability() == Availability::Available))
        .collect();
    let refs: Vec<(&str, bool)> = listed.iter().map(|(id, ok)| (id.as_str(), *ok)).collect();
    let chosen = choose_default_backend(configured, &refs, std::env::consts::OS)
        .unwrap_or("null")
        .to_owned();
    if configured.is_some_and(|c| c != chosen) {
        tracing::warn!(backend = ?configured, fallback = %chosen, "configured audio system unavailable");
    }
    chosen
}
```

5. Replace the body of `Engine::new` (L372-406) with:

```rust
        backends.push(Arc::new(NullBackend));
        let unusable = backends
            .iter()
            .filter(|b| b.availability() != Availability::Available)
            .map(|b| b.id().0)
            .collect();
        let audio_system = settings.default_backend.clone();
        let backend_in_use = choose_backend(&backends, audio_system.as_deref());
        let cartwall_routes = settings.cartwall_routes.clone();
        let (failures_tx, failures_rx) = crossbeam_channel::unbounded();
        let mut engine = Self {
            backends,
            unusable,
            settings,
            opener,
            buses: BTreeMap::new(),
            players: HashMap::new(),
            failures_tx,
            failures_rx,
            owners: HashMap::new(),
            next_key: 0,
            events: Vec::new(),
            dropped_commands: 0,
            slot_exhaustions: 0,
            tones: Vec::new(),
            cartwall: None,
            now: Instant::now(),
            true_peak: false,
            integration: (0.0, 0.0, 0.0),
            dsd_buses: HashMap::new(),
            audio_system,
            backend_in_use,
            running: HashMap::new(),
        };
        // L3: what the engine runs with, before anything is placed. The
        // cartwall has no bus until its first cart.
        engine.events.push(EngineEvent::AudioSystemInUse {
            configured: engine.audio_system.clone(),
            in_use: engine.backend_in_use.clone(),
        });
        engine.events.push(EngineEvent::Unplaced {
            holder: Holder::CartwallMain,
            route: cartwall_routes.main,
        });
        engine.events.push(EngineEvent::Unplaced {
            holder: Holder::CartwallCue,
            route: cartwall_routes.cue,
        });
        engine
```

6. Add public accessors after `pub fn settings(&self)` (L541-543):

```rust
    /// The backend the default output uses (L17).
    pub fn backend_in_use(&self) -> &str {
        &self.backend_in_use
    }

    /// What `device`'s bus runs with, while it is open (L4).
    pub fn running_settings(&self, device: &OutputDevice) -> Option<DeviceSettings> {
        self.running.get(&BusKey::from(device)).copied()
    }

    /// What `bus` runs with: its running settings when open, else what it
    /// would open with now (L13).
    fn device_of(&self, bus: &BusKey) -> DeviceSettings {
        self.running
            .get(bus)
            .copied()
            .unwrap_or_else(|| self.settings.device_settings(bus))
    }

    /// L3: `holder` holds `route` and plays on `bus`, or has no bus.
    fn report_placement(&mut self, holder: Holder, route: Option<Route>, bus: Option<&BusKey>) {
        let event = match bus {
            Some(bus) => EngineEvent::Placed {
                holder,
                route,
                device: bus.output_device(),
                running: self.device_of(bus),
            },
            None => EngineEvent::Unplaced { holder, route },
        };
        self.events.push(event);
    }
```

7. Reads that decide for an open bus use its running settings:
   - `is_bit_perfect` (L578): `&& self.settings.bit_perfect.contains(&p.bus)` → `&& self.device_of(&p.bus).bit_perfect`
   - `prepare_start` (L697): `if !self.settings.bit_perfect.contains(bus) || …` → `if !self.device_of(bus).bit_perfect || …`
   - `crates/fp-engine/src/engine/dsd.rs`, `silence_frames` (L69-71):

```rust
    fn silence_frames(&self, bus: &BusKey) -> u64 {
        let ms = self
            .device_of(bus)
            .dsd_silence_ms
            .unwrap_or(self.settings.dsd.silence_ms);
        self.frames_on(bus, ms)
    }
```

   - `try_start_dsd` (L84-90): `let mode = self.device_of(bus).dsd;`
   - `dsd_slot_started` (L340): `hold_others: self.device_of(bus).dsd_mix == Some(DsdMix::HoldOthers),`
   - `before_start_on` (L367): `DsdState::Playing { .. } => match self.device_of(bus).dsd_mix.unwrap_or(self.settings.dsd.mix) {`
   - `reopen_native_as_pcm` (L569): `sample_rate: self.device_of(bus).sample_rate,`
   - Remove the now unused `DsdOutput` import from `dsd.rs` only if the compiler says so.

8. `default_backend` (L947-960):

```rust
    /// The backend the default output uses (`backend_in_use`), else the
    /// first usable one.
    fn default_backend(&self) -> Arc<dyn AudioBackend> {
        self.route_backend(&self.backend_in_use)
            .or_else(|| {
                self.backends
                    .iter()
                    .find(|b| !self.unusable.contains(&b.id().0))
                    .cloned()
            })
            .unwrap_or_else(|| Arc::new(NullBackend))
    }
```

9. `ensure_bus` (L1058-1126): inside `if !self.buses.contains_key(key) { … }`, replace the lines computing `rate`, `buffer` and the `StreamConfig` with:

```rust
            let device = self.settings.device_settings(key);
            let (rate, buffer) = (device.sample_rate, device.buffer_frames);
            let config = StreamConfig {
                sample_rate: rate,
                buffer_frames: buffer,
                channels,
                exclusive: device.bit_perfect,
                dsd: None,
                // A device's own buffer it does not take falls back to the
                // global one (`pcm_fallback`), not to the device's default.
                exact_buffer: buffer != self.settings.buffer_frames,
            };
```

and before `self.buses.insert(key.clone(), bus);` add `self.running.insert(key.clone(), device);`.

10. `add_player` (L1196-1241): after `let (main, cue) = self.resolve_routes(player);` add

```rust
        let routes = self.settings.routes.iter().find(|r| r.player == player);
        let main_route = routes.and_then(|r| r.main.clone());
        let cue_route = routes.and_then(|r| r.cue.clone());
```

and replace the end of the function (from `self.ensure_bus(&main.0, now);`) with:

```rust
        self.ensure_bus(&main.0, now);
        if let Some((key, _)) = &cue {
            self.ensure_bus(key, now);
        }
        self.report_placement(Holder::PlayerMain(player), main_route, Some(&main.0));
        self.report_placement(Holder::PlayerCue(player), cue_route, cue.as_ref().map(|c| &c.0));
```

11. `remove_player` (L1242-1261): inside `if let Some(mut rt) = …`, after the `for p in all { … }` loop, add

```rust
            self.events.push(EngineEvent::Gone {
                holder: Holder::PlayerMain(player),
            });
            self.events.push(EngineEvent::Gone {
                holder: Holder::PlayerCue(player),
            });
```

12. `crates/fp-engine/src/engine/carts.rs`, `ensure_cartwall` (L66-114): declare `let mut created = None;` at the top; inside `if self.cartwall.is_none() { … }`, after `self.cartwall = Some(CartwallRuntime { … });` add `created = Some((main_key, cue_key, routes));` where, before building the runtime, `let main_key = main.0.clone(); let cue_key = cue.as_ref().map(|c| c.0.clone());` (taken before `main` and `cue` move into the runtime). After the `for key in keys { self.ensure_bus(&key, now); }` loop add:

```rust
        if let Some((main_key, cue_key, routes)) = created {
            self.report_placement(Holder::CartwallMain, routes.main, Some(&main_key));
            self.report_placement(Holder::CartwallCue, routes.cue, cue_key.as_ref());
        }
```

In `crates/fp-app/src/main.rs`:
- Imports (L18-20): `use fp_backends::{AudioBackend, Availability, NullBackend, display_name, system_backends};` (drop `choose_default_backend`).
- In `run`, replace L168-187 (from `let mut settings = EngineSettings::from_config(&config);` through `tracing::info!(backend = %in_use, ?availability, "audio systems");`) with:

```rust
    let settings = EngineSettings::from_config(&config);
    let engine = Engine::new(backends.clone(), settings, file_opener());
    let in_use = engine.backend_in_use().to_owned();
    tracing::info!(backend = %in_use, ?availability, "audio systems");
```

  remove the now unused `listed` binding (L163-167) and the later `let engine = Engine::new(…)` line (L194). Keep the `output`/`platform` lines (plan 5 moves the label into the interface).

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-engine --test live_settings --test conductor`
Expected: PASS.

- [ ] **Step 5: Keep the "no events" tests meaningful, then run everything**

Two engine tests asserted that nothing at all was reported; placements are reported now. In `crates/fp-engine/tests/engine.rs` and `crates/fp-engine/tests/cartwall.rs`, add this helper near the top of each file:

```rust
/// What the engine reported besides where holders play (live settings
/// spec L3).
fn reported(events: &[EngineEvent]) -> Vec<&EngineEvent> {
    events
        .iter()
        .filter(|e| {
            !matches!(
                e,
                EngineEvent::Placed { .. }
                    | EngineEvent::Unplaced { .. }
                    | EngineEvent::Gone { .. }
                    | EngineEvent::AudioSystemInUse { .. }
            )
        })
        .collect()
}
```

and change `assert!(r.events.is_empty(), "{:?}", r.events);` to `assert!(reported(&r.events).is_empty(), "{:?}", r.events);` in `cue_commands_without_a_cue_source_are_ignored` (`engine.rs` ~L1033) and `stopping_a_cart_before_it_starts_releases_its_slot` (`cartwall.rs` ~L160).

Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
Expected: all green.

- [ ] **Step 6: Commit**

```bash
git add -A crates/
git commit -m "$(cat <<'EOF'
feat(engine): report placements and keep each bus's running settings

The engine chooses the default audio system itself, tells the model
what it runs with and where each holder plays (live settings spec L3),
and decides for an open bus from that bus's own settings (L4), so a
new configuration reaches it only through a device change.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
EOF
)"
```

---

### Task 2: `BusCommand::Tune`

**Files:**
- Modify: `crates/fp-engine/src/mixer.rs:27-91` (`BusCommand`), `444-597` (`Mixer::apply`)
- Test: `crates/fp-engine/tests/mixer.rs`

**Interfaces:**
- Produces: `BusCommand::Tune(MixerConfig)` — applied at the next block start; replaces the mixer's config; a ramp already running keeps its length; `follow_rate` works from the new values afterwards.

- [ ] **Step 1: Write the failing test**

Append to `crates/fp-engine/tests/mixer.rs`:

```rust
#[test]
fn a_tune_changes_the_volume_smoothing_from_the_next_block() {
    let config = MixerConfig {
        volume_smoothing_frames: 4,
        declick_frames: 0,
        max_commands_per_block: 64,
    };
    let (mut m, mut h) = Mixer::new(2, config);
    let (mut p, c) = source_pair(16);
    p.push(&[1.0; 32]);
    let volume = full_volume();
    assert!(
        h.commands
            .push(BusCommand::Attach {
                slot: 0,
                source: c,
                volume: volume.clone(),
                first_channel: 0
            })
            .is_ok()
    );
    send(
        &mut h,
        BusCommand::Start {
            slot: 0,
            at_frame: 0,
        },
    );
    send(
        &mut h,
        BusCommand::Tune(MixerConfig {
            volume_smoothing_frames: 2,
            ..config
        }),
    );
    render(&mut m, 1, 2);
    volume.store(0.0);
    // `render` asserts that nothing allocates on the real-time path.
    let out = render(&mut m, 3, 2);
    assert_eq!(left(&out), vec![0.5, 0.0, 0.0]);
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p fp-engine --test mixer a_tune_changes`
Expected: FAIL to compile: `no variant named Tune found for enum BusCommand`.

- [ ] **Step 3: Write the implementation**

In `crates/fp-engine/src/mixer.rs`, add to `pub enum BusCommand` (after `HoldAll { … }`):

```rust
    /// Replaces the mixer's tuning (live settings spec §7) at the start of
    /// the next block: volume smoothing, de-click length and commands per
    /// block, in this bus's frames. A ramp already running keeps its
    /// length. `MixerConfig` is `Copy`: nothing is allocated.
    Tune(MixerConfig),
```

In `Mixer::apply`'s `match command { … }`, add after the `HoldAll` arm:

```rust
            BusCommand::Tune(config) => {
                self.config = config;
                self.volume_step = 1.0 / config.volume_smoothing_frames.max(1) as f32;
                // `follow_rate` sizes from these values from now on.
                self.smoothing_base = None;
                self.declick_base = config.declick_frames;
            }
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `cargo test -p fp-engine --test mixer`
Expected: PASS (all mixer tests).

- [ ] **Step 5: Run the gate and commit**

Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`

```bash
git add crates/fp-engine/src/mixer.rs crates/fp-engine/tests/mixer.rs
git commit -m "$(cat <<'EOF'
feat(engine): let the mixer take new tuning while it runs

BusCommand::Tune carries a MixerConfig by value and applies it at the
next block (live settings spec §7), with no allocation on the device
callback.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
EOF
)"
```

---

### Task 3: `UpdateSettings` and live tuning (L10, L13, §7)

**Files:**
- Modify: `crates/fp-engine/src/bus.rs` (add `set_timing`)
- Modify: `crates/fp-engine/src/worker.rs:233-250` (`LoadOptions`), `276-297` (`Job`), `418-470` (`run`)
- Modify: `crates/fp-engine/src/engine.rs` (`execute` arm, `ensure_bus` L1064-1092, `open_source` L1321-1326, `tick` L1981-2009, new `update_settings`, `send_tunes`, `mixer_config`, `bus_timing`, field `tune_due`)
- Modify: `crates/fp-engine/src/engine/carts.rs:164-170` (`cart_source` options)
- Modify: `crates/fp-engine/src/conductor.rs:450-470` (`spawn`), new `tick_period`
- Modify: `crates/fp-app/src/main.rs:198-200`
- Modify: `crates/fp-engine/tests/worker.rs` (six `LoadOptions { … }` literals: add `ready_frames: None,`), `crates/fp-engine/tests/conductor.rs:139`
- Modify: `docs/technical/audio-engine.md` (new "Live settings" section before "## Conductor"; "## Conductor" first sentence)
- Test: `crates/fp-engine/tests/{live_settings,bus,worker,conductor}.rs`

**Interfaces:**
- Consumes: `BusCommand::Tune` (Task 2), `EngineAction::UpdateSettings` (plan 1).
- Produces:
  - `Bus::set_timing(&mut self, timing: BusTiming)`
  - `LoadOptions::ready_frames: Option<usize>` (frames at the produced rate; `None`: the worker's own threshold)
  - `Conductor::tick_period(&self) -> Duration`; `Conductor::spawn(self, handle: ConductorHandle) -> std::io::Result<ConductorHandle>` (no period argument)
  - private: `fn mixer_config(t: &Tuning, rate: u32) -> MixerConfig`, `fn bus_timing(t: &Tuning) -> BusTiming`, `Engine::update_settings(&mut self, config: &Config, now: Instant)`, `Engine::send_tunes(&mut self)`, field `tune_due: std::collections::BTreeSet<BusKey>`

- [ ] **Step 1: Write the failing tests**

Append to `crates/fp-engine/tests/live_settings.rs`:

```rust
#[test]
fn l10_new_settings_change_no_open_bus_and_a_device_first_used_takes_them() {
    let mut r = rig();
    let q = PlayerId(2);
    let mut c = config();
    c.outputs.buffer_frames = 960;
    c.outputs.routes.push(PlayerRoutes {
        player: q,
        main: Some(route("other")),
        cue: None,
    });
    r.act(EngineAction::UpdateSettings(Box::new(c)));
    r.tick();
    assert_eq!(r.main.config().unwrap().buffer_frames, 480, "open: unchanged");
    r.act(EngineAction::AddPlayer { player: q });
    assert_eq!(r.other.config().unwrap().buffer_frames, 960, "L13");
}

#[test]
fn a_new_gain_smoothing_reaches_the_open_mixer() {
    let mut r = rig();
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, 0.0),
    });
    r.settle();
    r.run(5);
    let mut c = config();
    c.tuning.gain_smoothing_ms = 10.0; // 480 frames at 48 kHz
    r.act(EngineAction::UpdateSettings(Box::new(c)));
    r.run(1);
    r.act(EngineAction::SetVolume {
        player: P,
        volume: 0.0,
    });
    let heard = r.run(3);
    let audible = heard.iter().filter(|v| **v != 0.0).count();
    assert!(
        audible > 400,
        "the volume moved over 10 ms, not at once: {audible} frames"
    );
}
```

Review Focus 1: an open bus keeps deciding from its running settings after `UpdateSettings` (add `AudioFormat` to the `fp_model` import of `live_settings.rs`):

```rust
#[test]
fn an_open_bus_keeps_its_running_bit_perfect_setting() {
    let mut r = rig();
    r.main.set_exclusive_capable(true);
    let mut c = config();
    c.outputs.bit_perfect = vec![out("main")];
    r.act(EngineAction::UpdateSettings(Box::new(c)));
    let mut request = request(1, 0.0);
    request.format = Some(AudioFormat {
        sample_rate: 44_100,
        bits: Some(16),
        channels: 2,
        dsd_rate: None,
    });
    r.act(EngineAction::StartCurrent { player: P, request });
    r.settle();
    assert_eq!(
        r.main.config().unwrap().sample_rate,
        48_000,
        "not bit-perfect until ApplyDevice: the file is resampled"
    );
    assert!(!r.engine.running_settings(&out("main")).unwrap().bit_perfect);
}
```

The queue-full case needs the engine's private buses, so it is a unit
test: append to the end of `crates/fp-engine/src/engine.rs`:

```rust
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use fp_backends::OfflineBackend;

    #[test]
    fn a_tune_waits_for_room_in_the_command_queue() {
        let backend = OfflineBackend::new();
        let device = backend.add_device("main", 2);
        let mut config = Config::default();
        config.outputs.backend = Some("offline".into());
        let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
        let mut engine = Engine::new(
            backends,
            EngineSettings::from_config(&config),
            crate::worker::file_opener(),
        );
        let now = Instant::now();
        engine.execute(EngineAction::AddPlayer { player: PlayerId(1) }, now);
        let key = BusKey {
            backend: "offline".into(),
            device: "main".into(),
        };
        let bus = engine.buses.get_mut(&key).unwrap();
        // Nothing renders, so nothing drains the queue.
        while bus.send(BusCommand::Cancel { slot: 0 }) {}
        config.tuning.gain_smoothing_ms = 10.0;
        engine.execute(EngineAction::UpdateSettings(Box::new(config)), now);
        assert_eq!(engine.tune_due.len(), 1, "kept: the queue is full");
        // A block takes `max_commands_per_block` commands off the queue.
        device.render(480).unwrap();
        engine.tick(now);
        assert!(engine.tune_due.is_empty(), "sent once the mixer made room");
    }
}
```

Append to `crates/fp-engine/tests/bus.rs`:

```rust
#[test]
fn a_new_timing_applies_at_the_next_supervision() {
    let (_b, _device, mut bus, t0) = setup(true);
    bus.set_timing(BusTiming {
        startup_grace: Duration::from_millis(10),
        ..TIMING
    });
    bus.supervise(t0 + Duration::from_millis(50));
    assert_eq!(bus.health(), BusHealth::Lost, "the shorter grace applies");
}
```

Append to `crates/fp-engine/tests/worker.rs` (add `SampleSource` is already imported):

```rust
/// Emits `left` frames, then nothing more however often it is asked.
struct Stalling {
    left: u64,
}

impl SampleSource for Stalling {
    fn next_block(&mut self, out: &mut Vec<f32>) -> Result<bool, String> {
        if self.left == 0 {
            std::thread::sleep(Duration::from_millis(1));
            return Ok(true);
        }
        let n = self.left.min(64);
        for _ in 0..n {
            out.extend_from_slice(&[0.5, 0.5]);
        }
        self.left -= n;
        Ok(true)
    }
}

#[test]
fn a_source_becomes_ready_at_the_threshold_it_was_opened_with() {
    let opener: SourceOpener =
        Arc::new(|_, _, _| Ok(Box::new(Stalling { left: 2_000 }) as Box<dyn SampleSource>));
    let (w, _f) = worker(opener); // spawned with a threshold of 100 frames
    let (p, c) = source_pair(40_000);
    let shared = p.shared.clone();
    let options = LoadOptions {
        ready_frames: Some(5_000),
        ..LoadOptions::default()
    };
    w.load_with(SourceKey(1), PathBuf::from("x"), 0.0, p, options);
    wait_until("2000 frames buffered", || c.buffered_frames() >= 2_000);
    assert!(!shared.is_ready(), "5000 frames asked for, 2000 there");
    let (p, _c2) = source_pair(40_000);
    let shared = p.shared.clone();
    w.load(SourceKey(2), PathBuf::from("x"), 0.0, p);
    wait_until("ready at the spawn threshold", || shared.is_ready());
}
```

Append to `crates/fp-engine/tests/conductor.rs`:

```rust
#[test]
fn the_conductor_follows_a_new_tick_period() {
    let (mut conductor, handle, _device, now) = offline_conductor(model(1, 1));
    assert_eq!(conductor.tick_period(), Duration::from_millis(5));
    let mut config = conductor.state().config.clone();
    config.tuning.conductor_tick_ms = 20.0;
    handle.send(Command::UpdateConfig(Box::new(config)));
    conductor.tick(now);
    assert_eq!(conductor.tick_period(), Duration::from_millis(20));
}
```

and change L139 to `let handle = conductor.spawn(handle).unwrap();`.

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-engine --test live_settings --test bus --test worker --test conductor`
Expected: FAIL to compile: `no method named set_timing`, `no field ready_frames`, `no method named tick_period`.

- [ ] **Step 3: Write the implementation**

`crates/fp-engine/src/bus.rs`, after `pub fn busy_budget` (L690-699):

```rust
    /// New watchdog, reconnection and busy-retry timing (live settings
    /// spec §7): the next `supervise` and the next reopen use it.
    pub fn set_timing(&mut self, timing: BusTiming) {
        self.timing = timing;
    }
```

`crates/fp-engine/src/worker.rs`:
- `LoadOptions` (after `dsd: bool,`):

```rust
    /// Frames (at the produced rate) buffered before the source is ready;
    /// `None` is the threshold the worker was spawned with, scaled to the
    /// rate. Lets a new `tuning.ready_threshold_ms` reach the next source
    /// (live settings spec §7).
    pub ready_frames: Option<usize>,
```

- `Job` (after `rate: u32,`): `/// The ready threshold this source was opened with (see \`LoadOptions\`).\n    ready_frames: Option<usize>,` and in `run`'s `jobs.push(Job { … })` add `ready_frames: options.ready_frames,`.
- In `run`, replace `let ready = scaled_frames(ready_frames, job.rate, bus_rate);` with

```rust
            let ready = job
                .ready_frames
                .unwrap_or_else(|| scaled_frames(ready_frames, job.rate, bus_rate));
```

- In `crates/fp-engine/tests/worker.rs`, add `ready_frames: None,` to each of the six `LoadOptions { … }` literals.

`crates/fp-engine/src/engine.rs`:
- Add after `choose_backend`:

```rust
/// The mixer's lengths in frames of a bus running at `rate` (§7).
fn mixer_config(t: &Tuning, rate: u32) -> MixerConfig {
    MixerConfig {
        volume_smoothing_frames: frames_at(rate, t.gain_smoothing_ms).max(1) as u32,
        declick_frames: frames_at(rate, t.declick_ms) as u32,
        max_commands_per_block: t.max_commands_per_block,
    }
}

/// The watchdog, reconnection and busy-retry timing of every bus (§7).
fn bus_timing(t: &Tuning) -> BusTiming {
    BusTiming {
        watchdog_timeout: Duration::from_secs_f64(t.watchdog_timeout_ms / 1000.0),
        reconnect_interval: Duration::from_secs_f64(t.reconnect_interval_ms / 1000.0),
        startup_grace: Duration::from_secs_f64(t.watchdog_startup_grace_ms / 1000.0),
        busy_retries: t.device_busy_retries,
        busy_retry_interval: Duration::from_secs_f64(t.device_busy_retry_ms.max(0.0) / 1000.0),
    }
}
```

- In `ensure_bus`, replace the inline `let mixer = MixerConfig { … };` and `let timing = BusTiming { … };` (L1077-1091) with `let mixer = mixer_config(&self.settings.tuning, rate);` and `let timing = bus_timing(&self.settings.tuning);`, and drop the now unused `let t = &self.settings.tuning;` there.
- Add the field `/// Buses that still need the current mixer tuning (\`BusCommand::Tune\`).\n    tune_due: std::collections::BTreeSet<BusKey>,` to `Engine` and `tune_due: std::collections::BTreeSet::new(),` in `Engine::new`.
- In `execute`, replace the plan 1 no-op arm with:

```rust
            EngineAction::UpdateSettings(config) => self.update_settings(&config, now),
            // Live settings: the applies come with plan 3.
            EngineAction::ApplyAudioSystem { .. }
            | EngineAction::ApplyRoute { .. }
            | EngineAction::ApplyDevice { .. } => {}
```

- Add to `impl Engine` (after `report_placement`):

```rust
    /// L10: the configuration new buses and new holders take (L13), and
    /// the tuning, read live (§7). No open bus changes: a device changes
    /// only through `ApplyDevice`, a holder through `ApplyRoute`, the
    /// default output through `ApplyAudioSystem`.
    fn update_settings(&mut self, config: &Config, now: Instant) {
        let retune = self.settings.tuning != config.tuning;
        self.settings = EngineSettings::from_config(config);
        if !retune {
            return;
        }
        let timing = bus_timing(&self.settings.tuning);
        let keys: Vec<BusKey> = self.buses.keys().cloned().collect();
        for key in keys {
            if let Some(bus) = self.buses.get_mut(&key) {
                bus.set_timing(timing);
            }
            // `mixer_headroom`: the capacity is derived again; it only grows.
            self.ensure_bus(&key, now);
            self.tune_due.insert(key);
        }
        self.send_tunes();
    }

    /// Sends every bus that waits for it the mixer tuning at its running
    /// rate. A full command queue is tried again next tick; only the
    /// latest tuning matters.
    fn send_tunes(&mut self) {
        let tuning = &self.settings.tuning;
        let mut sent = Vec::new();
        for key in &self.tune_due {
            if let Some(bus) = self.buses.get_mut(key)
                && !bus.send(BusCommand::Tune(mixer_config(tuning, bus.sample_rate())))
            {
                continue;
            }
            sent.push(key.clone());
        }
        for key in sent {
            self.tune_due.remove(&key);
        }
    }
```

- In `tick`, before `std::mem::take(&mut self.events)`, add `self.send_tunes();`.
- In `open_source` (L1321-1326), the `LoadOptions` literal becomes:

```rust
            let options = crate::worker::LoadOptions {
                rate: Some(rate),
                dsd,
                ready_frames: Some(
                    frames_at(rate, self.settings.tuning.ready_threshold_ms) as usize
                ),
                ..crate::worker::LoadOptions::default()
            };
```

`crates/fp-engine/src/engine/carts.rs`, `cart_source` (L164-170):

```rust
        let rate = self.rate_of(&bus_key);
        let options = LoadOptions {
            until_secs: Some(request.until_secs).filter(|u| u.is_finite()),
            looped: request.looped,
            rate: Some(rate),
            fade_out_frames: self.frames_on(&bus_key, self.settings.tuning.declick_ms),
            dsd: false,
            ready_frames: Some(frames_at(rate, self.settings.tuning.ready_threshold_ms) as usize),
        };
```

(`frames_at` is a private free function of `engine.rs`; `carts.rs` sees it through `use super::*;`.)

`crates/fp-engine/src/conductor.rs`:
- Add to `impl Conductor` (after `engine()`):

```rust
    /// How long the conductor sleeps between ticks: `tuning.conductor_tick_ms`,
    /// read again before every sleep (live settings spec §7).
    pub fn tick_period(&self) -> Duration {
        Duration::from_secs_f64(self.state.config.tuning.conductor_tick_ms.max(1.0) / 1000.0)
    }
```

- `spawn` loses its `period` parameter (`pub fn spawn(mut self, mut handle: ConductorHandle) -> std::io::Result<ConductorHandle>`); inside the loop use `if let Some(rest) = self.tick_period().checked_sub(started.elapsed()) {`. Update its doc comment: "Runs the conductor on its own thread, ticking every `tick_period`."

`crates/fp-app/src/main.rs`: delete `let tick = Duration::from_secs_f64(…);` (L199) and call `conductor.spawn(handle)?`.

`docs/technical/audio-engine.md`: change the first sentence of "## Conductor" to "`Conductor` (`conductor.rs`) owns `AppState` and the `Engine`. Each tick (`tuning.conductor_tick_ms`, read again before every sleep: `Conductor::tick_period`) it:" and insert before "## Conductor":

```markdown
## Live settings

The engine tells the model what it runs with (live settings spec, L3). At
start, `Engine::new` reports `AudioSystemInUse { configured, in_use }`: the
configured `outputs.backend` and the backend it chose for the default output
(`choose_default_backend`, the same OS preference and availability list
`main` used before), and `Unplaced` for the cartwall's Main and Cue (no bus
before the first cart). Adding a player reports `Placed` for its Main and
`Placed` or `Unplaced` for its Cue; the cartwall's first cart reports its
two holders; removing a player reports `Gone` for both. `Placed` carries the
route the engine holds, the device and the device's running settings
(`DeviceSettings`).

Each open bus keeps the settings it was opened with in `Engine::running`
(`running_settings` for a device). Whatever decides something for an open
bus reads them: bit-perfect rate following (`prepare_start`, the BP badge),
the DSD mode, mix and silence, and the configured PCM rate a native DSD bus
returns to. `EngineSettings::device_settings` gives what a bus would open
with now and equals `fp_model::device_settings` for the same configuration.

`EngineAction::UpdateSettings(config)` replaces `EngineSettings`: new buses,
new players and the cartwall's first cart use it (L13); no open bus changes.
The tuning is read live (§7): fades, pauses, prebuffer, schedule lead and
ready threshold are read where they are used (the ready threshold travels
with each `LoadOptions`, so a source keeps the one it was opened with);
`mixer_headroom` grows each bus's capacity at once; every bus gets
`Bus::set_timing` and a `BusCommand::Tune(MixerConfig)` with the lengths at
its running rate (a full command queue is tried again next tick); the
conductor reads `conductor_tick_ms` before every sleep.
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-engine`
Expected: PASS.

- [ ] **Step 5: Run the gate and commit**

Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`

```bash
git add -A crates/ docs/technical/audio-engine.md
git commit -m "$(cat <<'EOF'
feat(engine): take new settings and tuning while running

UpdateSettings (live settings spec L10) is what new buses and holders
open with; open buses keep theirs. Every tuning value is read live
(§7): Tune and set_timing for open buses, the ready threshold per
load, the conductor's tick period before each sleep.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
EOF
)"
```

---

## Self-Review (done while writing)

- **Spec coverage:** L3 engine side (Task 1), L4 engine side (Task 1), L10 (Task 3), L13 for new buses and holders (Tasks 1, 3; closing unused buses is plan 3), §7 every field: `declick_ms`, `pause_ramp_ms`, `prebuffer_secs`, `schedule_lead_ms` (read at use; `settings` replaced), `ready_threshold_ms` (per load), `mixer_headroom` (`ensure_bus` on every bus), `max_commands_per_block`, `gain_smoothing_ms` (`Tune`), watchdog and busy timing (`set_timing`), `conductor_tick_ms` (`tick_period`), `save_debounce_ms`/`missing_recheck_ms` (already live), `restart_handoff_ms` (removed in plan 5). L17's backend choice moves here; the switch itself is plan 3.
- **Placeholders:** the queue-filling test step gives exact code and a self-checking assertion.
- **Type consistency:** `device_of`, `report_placement`, `running`, `backend_in_use`, `audio_system`, `mixer_config`, `bus_timing`, `tune_due` are the names plan 3 uses.
