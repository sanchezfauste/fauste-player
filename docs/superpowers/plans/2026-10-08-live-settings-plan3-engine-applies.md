# Live Settings, Plan 3: The Engine Applies Output Changes

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The engine runs `ApplyDevice`, `ApplyRoute` and `ApplyAudioSystem` as soon as what they touch is quiet (or at once when forced), keeps the running configuration on a refusal, keeps the new one for an absent device, closes buses no holder uses, and reports `Applied` (L11–L17, L21). After this plan, with plans 1 and 2, a change made in Settings reaches the audio while the application runs.

**Architecture:** Every `Apply…` action goes into `Engine::kept` (one per `Target`; a newer one replaces it), and `run_kept` (at once and on every tick) runs those whose paths are quiet or that are forced. A device change is one `reopen_with` on the open bus (`Bus::apply_config` adds the absent-device rule); sources follow a new rate through `reopen_on_bus`, the body of today's `follow_forced_rate` made reusable. A route change moves the holder's sources to the new bus with the same reopen-at-position code, filtered to that holder. An audio system change re-places every holder on the default output. Buses holders leave are closed once quiet and unused.

**Tech Stack:** Rust 2024, `fp-engine`, `fp-backends` (`OfflineDevice::{refuse_rate, set_busy, unplug, replug, open_attempts, config, set_exclusive_capable}`).

**Spec:** `docs/superpowers/specs/2026-10-07-live-settings-design.md` (whole, including §14 and the "Planning notes"). Plans 1 and 2 must be merged first.

## Global Constraints

- All code, identifiers, comments, docs and commit messages in English; never mention other playout or radio-automation products.
- D1 and rule 10: nothing is cut on air by itself; after any apply, nothing that was paused, stopped or loaded starts. Only a forced apply (the operator's confirmed **Apply now**) interrupts, for one reopen.
- L11 quiet: no started, unpaused source (current, outgoing or CUE), no cart source, no test tone; paused and waiting sources are quiet. A DSD silence still running is not quiet.
- L14: busy budget `busy_budget(true)` on a quiet bus, none when forced on a sounding one.
- Real-time safety: nothing here runs on the device callback; buses are reopened on the conductor thread with the existing `Bus` methods; memory still leaves the mixer through `Retired`.
- `fp-engine` denies `clippy::indexing_slicing`; `unwrap`/`expect`/`panic` denied outside tests.
- Engine tests use the `Offline` backend and explicit time; bounded waits for decode workers only.
- Conventional Commits; every commit ends with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Gate for every task: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`.

## Review Focus

1. **A paused player's pause ramp not over yet** when a change arrives: the bus is not quiet for one more block, then the change runs; a paused track never starts. Test: Task 1, `l12_a_new_rate_reopens_the_device_and_a_paused_track_stays_paused_where_it_was` (the change arrives during the pause ramp and must wait for it).
2. **An operator's Apply now after the device refused that rate before**: the device is asked again (its refusal memory is cleared), not answered from the cache. Test: Task 3, `l14_apply_now_asks_a_device_again_for_a_rate_it_refused`.
3. **A device unplugged while its change waits**: nothing is interrupted; it opens with the new settings when it is back. Test: Task 3, `l15_an_absent_device_takes_the_new_settings_when_it_returns`.
4. **Moving one player off a device another player is using**: the other player keeps playing and the device stays open. Test: Task 4, `l16_a_route_change_waits_for_its_own_holder_only` (P plays on `main` while Q leaves it).
5. **A route to a channel pair the open device does not have yet**: the device is reopened with more channels (waiting until it is quiet), instead of a silent misroute. Test: Task 4, `l16_a_route_to_channels_the_open_device_lacks_reopens_it_with_more`.

---

## File Structure

| File | Responsibility |
|---|---|
| `crates/fp-engine/src/engine.rs` | `kept`, `run_kept`, quiet checks, `apply_device`, `reopen_device`, `apply_route` (players), `apply_audio_system`, `orphans`/`close_orphans`, `reopen_on_bus`/`reopen_player_sources` |
| `crates/fp-engine/src/engine/carts.rs` | Cartwall route moves, cart path quiet, `reopen_carts` |
| `crates/fp-engine/src/engine/dsd.rs` | `dsd_silence_running`, `leave_dsd_for_reopen` |
| `crates/fp-engine/src/bus.rs` | `apply_config`, `forget_refusals`, `replace_pcm_fallback`, absent-device handling in `reopen_with` |
| `crates/fp-engine/tests/live_settings.rs` | L11–L17, L21 tests |
| `crates/fp-engine/tests/dsd_output.rs` | Leaving DSD on a device change |
| `crates/fp-engine/tests/conductor.rs` | End to end through the model |
| `docs/technical/audio-engine.md`, `docs/technical/backends.md`, `docs/technical/architecture.md` | Applying changes, the audio system switch, the live settings flow |

---

### Task 1: Kept changes, the quiet guard and `ApplyDevice` (L11, L12)

**Files:**
- Modify: `crates/fp-engine/src/engine.rs` (imports L13-16; `Engine` fields; `Engine::new`; `execute` live arms; `follow_forced_rate` ~L778-911; `ensure_bus` fallback ~L1093-1103; `tick` ~L1981-2009; new methods)
- Modify: `crates/fp-engine/src/engine/dsd.rs` (new `dsd_silence_running`)
- Modify: `crates/fp-engine/src/bus.rs` (new `replace_pcm_fallback`)
- Test: `crates/fp-engine/tests/live_settings.rs`

**Interfaces:**
- Consumes: `fp_model::{Target, Wanted, DeviceSettings, target_and_wanted}` (plan 1); `Engine::{running, device_of, report_placement}` (plan 2).
- Produces (private to `Engine` unless stated):
  - field `kept: BTreeMap<Target, EngineAction>`
  - `fn keep(&mut self, action: EngineAction)`, `fn run_kept(&mut self)`
  - `fn bus_quiet(&self, bus: &BusKey) -> bool`, `fn player_quiet(&self, rt: &PlayerRuntime, on: impl Fn(&Playing) -> bool) -> bool`
  - `fn apply_device(&mut self, device: &OutputDevice, wanted: DeviceSettings, force: bool) -> bool` (`false`: kept)
  - `fn reopen_device(&mut self, key: &BusKey, wanted: DeviceSettings, quiet: bool, force: bool) -> Result<(), String>`
  - `fn report_applied(&mut self, target: Target, wanted: Wanted, outcome: Result<(), String>)`
  - `fn reopen_on_bus(&mut self, bus: &BusKey, old_rate: u32)`, `fn reopen_player_sources(&mut self, player: PlayerId, on: impl Fn(&Playing) -> bool, old_rate: u32)`
  - `fn pcm_fallback_for(global: (u32, u32), wanted: DeviceSettings, config: StreamConfig) -> Option<StreamConfig>` (free function)
  - `Bus::replace_pcm_fallback(&mut self, fallback: Option<StreamConfig>)` (pub)
  - `dsd.rs`: `pub(super) fn dsd_silence_running(&self, bus: &BusKey) -> bool`

- [ ] **Step 1: Write the failing tests**

In `crates/fp-engine/tests/live_settings.rs`, add `DeviceSettings, Target, Wanted` to the `fp_model` import and `const Q: PlayerId = PlayerId(2);` under `const P`, then append:

```rust
fn settings_of(r: &Rig, device: &str) -> DeviceSettings {
    r.engine.running_settings(&out(device)).unwrap()
}

/// The outcome the engine reported for `device`, if any.
fn applied<'a>(events: &'a [EngineEvent], device: &str) -> Option<&'a Result<(), String>> {
    events.iter().find_map(|e| match e {
        EngineEvent::Applied {
            target: Target::Device(d),
            outcome,
            ..
        } if *d == out(device) => Some(outcome),
        _ => None,
    })
}

fn apply_device(r: &mut Rig, device: &str, settings: DeviceSettings, force: bool) {
    r.act(EngineAction::ApplyDevice {
        device: out(device),
        settings,
        force,
    });
}

#[test]
fn l11_a_device_change_waits_for_a_fade_tail_then_runs() {
    let mut r = rig();
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, 0.0),
    });
    r.settle();
    r.run(3);
    r.act(EngineAction::FadeOutAndStop {
        player: P,
        fade_ms: 200,
    });
    let wanted = DeviceSettings {
        buffer_frames: 960,
        ..settings_of(&r, "main")
    };
    apply_device(&mut r, "main", wanted, false);
    r.run(2);
    assert_eq!(r.main.config().unwrap().buffer_frames, 480, "the fade sounds");
    assert!(applied(&r.events, "main").is_none());
    r.run(30);
    assert_eq!(r.main.config().unwrap().buffer_frames, 960);
    assert_eq!(applied(&r.events, "main"), Some(&Ok(())));
    assert_eq!(settings_of(&r, "main"), wanted);
}

#[test]
fn l11_a_device_change_waits_for_a_test_tone() {
    let mut r = rig();
    r.engine
        .play_test_tone(&route("main"), 440.0, 0.1, -18.0, r.clock);
    let wanted = DeviceSettings {
        buffer_frames: 960,
        ..settings_of(&r, "main")
    };
    apply_device(&mut r, "main", wanted, false);
    r.run(2);
    assert_eq!(r.main.config().unwrap().buffer_frames, 480);
    r.run(20);
    assert_eq!(r.main.config().unwrap().buffer_frames, 960);
}

#[test]
fn l12_a_new_rate_reopens_the_device_and_a_paused_track_stays_paused_where_it_was() {
    let mut r = rig();
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, 2.0),
    });
    r.settle();
    r.run(2);
    r.act(EngineAction::Pause { player: P });
    // The pause ramp is still running: the bus is not quiet yet.
    let wanted = DeviceSettings {
        sample_rate: 44_100,
        ..settings_of(&r, "main")
    };
    apply_device(&mut r, "main", wanted, false);
    assert_eq!(r.main.config().unwrap().sample_rate, 48_000);
    // The ramp (480 frames) and one block of margin.
    r.run(3);
    assert_eq!(r.main.config().unwrap().sample_rate, 44_100);
    let position = r.engine.telemetry(P).position_secs.unwrap();
    r.settle();
    let heard = r.run(5);
    assert!(heard.iter().all(|v| *v == 0.0), "nothing paused starts");
    let after = r.engine.telemetry(P).position_secs.unwrap();
    assert!((after - position).abs() < 0.001, "{position} then {after}");
    assert_eq!(applied(&r.events, "main"), Some(&Ok(())));
}

#[test]
fn l12_a_new_buffer_reopens_at_the_running_rate() {
    let mut r = rig();
    let wanted = DeviceSettings {
        buffer_frames: 960,
        ..settings_of(&r, "main")
    };
    apply_device(&mut r, "main", wanted, false);
    let config = r.main.config().unwrap();
    assert_eq!((config.sample_rate, config.buffer_frames), (48_000, 960));
}

#[test]
fn l12_a_bit_perfect_device_takes_a_new_rate_without_reopening() {
    let mut c = config();
    c.outputs.bit_perfect = vec![out("main")];
    let mut r = rig_with(&c);
    let attempts = r.main.open_attempts();
    let wanted = DeviceSettings {
        sample_rate: 96_000,
        ..settings_of(&r, "main")
    };
    apply_device(&mut r, "main", wanted, false);
    assert_eq!(r.main.open_attempts(), attempts, "the next file sets the rate");
    assert_eq!(settings_of(&r, "main").sample_rate, 96_000);
    assert_eq!(applied(&r.events, "main"), Some(&Ok(())));
}

#[test]
fn l12_bit_perfect_on_reopens_exclusive_and_a_refusal_plays_shared() {
    let mut r = rig();
    r.main.set_exclusive_capable(true);
    let shared = settings_of(&r, "main");
    let exclusive = DeviceSettings {
        bit_perfect: true,
        ..shared
    };
    apply_device(&mut r, "main", exclusive, false);
    assert!(r.main.config().unwrap().exclusive);
    apply_device(&mut r, "main", shared, false);
    assert!(!r.main.config().unwrap().exclusive);
    r.main.set_exclusive_capable(false);
    apply_device(&mut r, "main", exclusive, false);
    assert!(!r.main.config().unwrap().exclusive, "B4: plays shared");
    assert!(settings_of(&r, "main").bit_perfect);
}

#[test]
fn l12_dsd_mix_and_silence_are_stored_without_a_reopen() {
    let mut c = config();
    c.outputs.bit_perfect = vec![out("main")];
    c.outputs.dsd_output = vec![DsdDevice {
        backend: "offline".into(),
        device: "main".into(),
        mode: DsdOutput::Dop,
    }];
    let mut r = rig_with(&c);
    let attempts = r.main.open_attempts();
    let wanted = DeviceSettings {
        dsd_mix: Some(DsdMix::HoldOthers),
        dsd_silence_ms: Some(400.0),
        ..settings_of(&r, "main")
    };
    apply_device(&mut r, "main", wanted, false);
    assert_eq!(r.main.open_attempts(), attempts);
    assert_eq!(settings_of(&r, "main"), wanted);
}

#[test]
fn a_change_for_a_device_not_open_is_done_at_once() {
    let mut r = rig();
    let wanted = DeviceSettings {
        buffer_frames: 960,
        ..device_settings(&config().outputs, &out("other"))
    };
    apply_device(&mut r, "other", wanted, false);
    assert_eq!(applied(&r.events, "other"), Some(&Ok(())));
    assert!(!r.other.is_open(), "L13: it opens when first used");
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-engine --test live_settings`
Expected: FAIL: the `l11_`/`l12_` tests time out on the asserted configuration (the engine ignores `ApplyDevice`), `a_change_for_a_device_not_open…` finds no `Applied`.

- [ ] **Step 3: Write the implementation**

`crates/fp-engine/src/bus.rs`, after `set_pcm_fallback` (L400-403):

```rust
    /// Replaces the PCM fallback after a device change (live settings spec
    /// L12), as `Engine::ensure_bus` sets it for a new bus.
    pub fn replace_pcm_fallback(&mut self, fallback: Option<StreamConfig>) {
        self.pcm_fallback = fallback;
    }
```

`crates/fp-engine/src/engine/dsd.rs`, after `is_dsd_direct` (L54-57):

```rust
    /// Whether `bus` runs a DSD stream's silence (its tail, or a switch to
    /// PCM): not quiet, so a device change waits for it (live settings L11).
    pub(super) fn dsd_silence_running(&self, bus: &BusKey) -> bool {
        self.dsd_buses
            .get(bus)
            .is_some_and(|d| !matches!(d.state, DsdState::Playing { .. }))
    }
```

`crates/fp-engine/src/engine.rs`:

1. Imports: add `Target, Wanted` to the `fp_model::{…}` list.
2. `Engine` field (after `running`):

```rust
    /// Output changes kept until what they touch is quiet (live settings
    /// spec L11), one per target: a newer action for a target replaces it.
    kept: BTreeMap<Target, EngineAction>,
```

and `kept: BTreeMap::new(),` in `Engine::new`.

3. Free function after `bus_timing`:

```rust
/// The configuration the watchdog falls back to when a device's own rate
/// or buffer does not open any more (operator feedback 4, Q12): the global
/// rate and buffer. `None` for a device that runs at the global values.
fn pcm_fallback_for(
    (rate, buffer): (u32, u32),
    wanted: DeviceSettings,
    config: StreamConfig,
) -> Option<StreamConfig> {
    (wanted.sample_rate != rate || wanted.buffer_frames != buffer).then_some(StreamConfig {
        sample_rate: rate,
        buffer_frames: buffer,
        exact_buffer: false,
        ..config
    })
}
```

and in `ensure_bus`, replace the `if rate != self.settings.sample_rate || buffer != self.settings.buffer_frames { bus.set_pcm_fallback(…); }` block with:

```rust
            // A device's own rate or buffer it no longer takes (another
            // DAC, a changed driver) must not leave it silent for good:
            // the watchdog falls back to the global values.
            let global = (self.settings.sample_rate, self.settings.buffer_frames);
            if let Some(fallback) = pcm_fallback_for(global, device, config) {
                bus.set_pcm_fallback(fallback);
            }
```

4. Split `follow_forced_rate` (L778-911 at the time of writing). Keep its doc comment and make it:

```rust
    fn follow_forced_rate(&mut self, bus: &BusKey, old_rate: u32) {
        let new_rate = self.rate_of(bus);
        if new_rate == old_rate {
            return;
        }
        tracing::warn!(
            ?bus,
            old_rate,
            new_rate,
            "the device changed rate; reopening its sources"
        );
        self.reopen_on_bus(bus, old_rate);
    }

    /// Every player and cart source on `bus` is opened again at its
    /// position, measured at `old_rate`, as a seek does: one that was
    /// playing starts again as soon as it is ready, a waiting or paused one
    /// stays waiting. Fades in progress and test tones are cut; a cut fade
    /// ends as it would have (`ReachedEnd`, `FadeCompleted`). Used when the
    /// device changed rate under its sources, when a DSD stream was left,
    /// and by a forced device change (live settings spec L12, L21).
    fn reopen_on_bus(&mut self, bus: &BusKey, old_rate: u32) {
        let ids: Vec<PlayerId> = self.players.keys().copied().collect();
        for player in ids {
            self.reopen_player_sources(player, |p| &p.bus == bus, old_rate);
        }
        self.follow_forced_rate_carts(bus, old_rate);
        let (gone, kept): (Vec<_>, Vec<_>) = std::mem::take(&mut self.tones)
            .into_iter()
            .partition(|(b, _)| b == bus);
        self.tones = kept;
        for (bus, slot) in gone {
            self.send(&bus, BusCommand::Detach { slot });
        }
    }
```

   and move the per-player body of the old loop (from `let on_bus = |p: &Playing| &p.bus == bus;` through the end of the `for role in [Role::Current, Role::Preload, Role::Cue] { … }` loop) into:

```rust
    /// The sources of `player` that `on` selects, opened again where the
    /// player's routes point now (`rt.main`, `rt.cue`), at their position
    /// measured at `old_rate` (see `reopen_on_bus`). A route change (L16)
    /// moves a holder with it.
    fn reopen_player_sources(
        &mut self,
        player: PlayerId,
        on: impl Fn(&Playing) -> bool,
        old_rate: u32,
    ) {
        let old = f64::from(old_rate.max(1));
        let position = |p: &Playing| p.start_secs + p.shared.frames_played() as f64 / old;
        let touches = self.players.get(&player).is_some_and(|rt| {
            rt.current
                .iter()
                .chain(rt.preload.iter())
                .chain(rt.cue_src.iter())
                .chain(rt.outgoing.iter())
                .chain(rt.cue_outgoing.iter())
                .any(|p| on(p))
        });
        if !touches {
            return;
        }
        // … the old body from `self.undispatch(player);` to the end of the
        // `for role in …` loop, unchanged except: `.partition(on_bus)`
        // becomes `.partition(|p| on(p))` (twice), and
        // `field.take_if(|p| on_bus(p))` becomes `field.take_if(|p| on(p))`;
        // `continue` inside it (when the runtime is gone) becomes `return`
        // where it left the outer per-player loop, and stays `continue`
        // inside the `for role` loop.
    }
```

   Concretely, the old outer loop's `let Some(rt) = self.players.get_mut(&player) else { continue; };` becomes `else { return; };`, and inside `for role in [Role::Current, Role::Preload, Role::Cue]` the `let Some(rt) = … else { break; };` and `continue` stay as they are. The moved code is otherwise byte-for-byte the same; `cargo test -p fp-engine --test bit_perfect --test dsd_output` (which exercise `follow_forced_rate`) must stay green.

5. Quiet checks and device apply, added to `impl Engine` after `report_placement`:

```rust
    /// L11: whether nothing on `bus` is audible: no started source of a
    /// player that is not paused (current, outgoing, a preload a
    /// transition started, a CUE not held), no cart, no test tone and no
    /// DSD silence running. Paused and waiting sources are quiet.
    fn bus_quiet(&self, bus: &BusKey) -> bool {
        self.players
            .values()
            .all(|rt| self.player_quiet(rt, |p| &p.bus == bus))
            && !self.carts_sounding(bus)
            && !self.tones.iter().any(|(b, _)| b == bus)
            && !self.dsd_silence_running(bus)
    }

    /// Whether every source of `rt` that `on` selects is silent: never
    /// started, a held CUE, or paused with its pause ramp over.
    fn player_quiet(&self, rt: &PlayerRuntime, on: impl Fn(&Playing) -> bool) -> bool {
        let main = &rt.main.0;
        // One block of margin: the engine's frame lags the mixer's.
        let paused = rt.paused
            && self.now_frame(main)
                >= rt
                    .pause_ramp_ends
                    .saturating_add(u64::from(self.buffer_of(main)));
        let silent = |p: &Playing, held: bool| !on(p) || p.start == StartState::Idle || held;
        rt.current.iter().all(|p| silent(p, paused))
            && rt.outgoing.iter().all(|p| silent(p, paused))
            && rt.preload.iter().all(|p| silent(p, false))
            && rt.cue_src.iter().all(|p| silent(p, rt.cue_paused))
            && rt.cue_outgoing.iter().all(|p| silent(p, false))
    }

    /// L11: keeps `action` (replacing an older one for its target) and runs
    /// whatever kept action can run now.
    fn keep(&mut self, action: EngineAction) {
        if let Some((target, _)) = fp_model::target_and_wanted(&action) {
            self.kept.insert(target, action);
        }
        self.run_kept();
    }

    /// Runs the kept output changes that are forced or whose paths are
    /// quiet, in L9 order (`Target` sorts the audio system, the routes,
    /// then the devices). Called by `keep` and on every tick.
    fn run_kept(&mut self) {
        let targets: Vec<Target> = self.kept.keys().cloned().collect();
        for target in targets {
            let Some(action) = self.kept.get(&target).cloned() else {
                continue;
            };
            let done = match action {
                EngineAction::ApplyDevice {
                    device,
                    settings,
                    force,
                } => self.apply_device(&device, settings, force),
                _ => true,
            };
            if done {
                self.kept.remove(&target);
            }
        }
    }

    /// L8: tells the model an output change ran.
    fn report_applied(&mut self, target: Target, wanted: Wanted, outcome: Result<(), String>) {
        match &outcome {
            Ok(()) => tracing::info!(?target, "output change applied"),
            Err(reason) => {
                tracing::warn!(?target, %reason, "output change refused; the running settings stay");
            }
        }
        self.events.push(EngineEvent::Applied {
            target,
            wanted,
            outcome,
        });
    }

    /// L12: `device` runs with `wanted`. Returns `false` while it must wait
    /// (L11): its bus sounds and the change is not forced.
    fn apply_device(&mut self, device: &OutputDevice, wanted: DeviceSettings, force: bool) -> bool {
        let key = BusKey::from(device);
        let target = Target::Device(device.clone());
        let Some(old) = self.running.get(&key).copied() else {
            // Not open: it opens with the current configuration (L13).
            self.report_applied(target, Wanted::Device(wanted), Ok(()));
            return true;
        };
        let quiet = self.bus_quiet(&key);
        if !quiet && !force {
            return false;
        }
        let carries_dsd = self.dsd_buses.contains_key(&key);
        // A bit-perfect rate is only where the bus starts: the next file
        // sets it (B3). DSD mix and silence are read at the next DSD
        // decision. Neither reopens.
        let reopen = old.buffer_frames != wanted.buffer_frames
            || old.bit_perfect != wanted.bit_perfect
            || (!wanted.bit_perfect && old.sample_rate != wanted.sample_rate)
            || (carries_dsd && old.dsd != wanted.dsd);
        let outcome = if reopen {
            self.reopen_device(&key, wanted, quiet, force)
        } else {
            Ok(())
        };
        if outcome.is_ok() {
            self.running.insert(key, wanted);
        }
        self.report_applied(target, Wanted::Device(wanted), outcome);
        true
    }

    /// Reopens `key` with `wanted` in one reopen (L12), keeping the mixer,
    /// its clock and its slots; the sources on it follow a new rate. A
    /// refusal brings the running configuration back (L14).
    fn reopen_device(
        &mut self,
        key: &BusKey,
        wanted: DeviceSettings,
        quiet: bool,
        _force: bool,
    ) -> Result<(), String> {
        let now = self.now;
        let before_rate = self.rate_of(key);
        let global = (self.settings.sample_rate, self.settings.buffer_frames);
        let Some(bus) = self.buses.get_mut(key) else {
            return Ok(());
        };
        let base = bus.config();
        let config = StreamConfig {
            sample_rate: if wanted.bit_perfect {
                base.sample_rate
            } else {
                wanted.sample_rate
            },
            buffer_frames: wanted.buffer_frames,
            exclusive: wanted.bit_perfect,
            dsd: None,
            exact_buffer: wanted.buffer_frames != global.1,
            ..base
        };
        // A sounding bus (a forced change) never waits for a busy device:
        // its timeline would stall (L14, L21).
        let mut budget = bus.busy_budget(quiet);
        let outcome = bus.reopen_with(config, now, &mut budget);
        if outcome.is_ok() {
            bus.replace_pcm_fallback(pcm_fallback_for(global, wanted, config));
        }
        if self.rate_of(key) != before_rate {
            self.reopen_on_bus(key, before_rate);
        }
        outcome
    }
```

   (`_force` is used from Task 3 on; rename it to `force` there.)

6. In `execute`, replace the live arms with:

```rust
            EngineAction::UpdateSettings(config) => self.update_settings(&config, now),
            apply @ EngineAction::ApplyDevice { .. } => self.keep(apply),
            // Live settings: routes and the audio system come with tasks 4
            // and 5 of plan 3.
            EngineAction::ApplyAudioSystem { .. } | EngineAction::ApplyRoute { .. } => {}
```

7. In `tick`, after `self.end_dsd_streams();`, add `self.run_kept();`.

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-engine`
Expected: PASS (all engine tests, including `bit_perfect` and `dsd_output`).

- [ ] **Step 5: Run the gate and commit**

Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`

```bash
git add -A crates/
git commit -m "$(cat <<'EOF'
feat(engine): apply a device change once nothing sounds on it

ApplyDevice (live settings spec L11, L12) waits for a quiet bus, then
reopens it once: a new rate opens its sources again where they were,
paused ones stay paused; a bit-perfect rate, the DSD mix and the DSD
silence are only stored.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
EOF
)"
```

---

### Task 2: Leaving DSD on a device change (L12, §8)

**Files:**
- Modify: `crates/fp-engine/src/engine/dsd.rs` (new `leave_dsd_for_reopen`)
- Modify: `crates/fp-engine/src/engine.rs` (`reopen_device`)
- Test: `crates/fp-engine/tests/dsd_output.rs`

**Interfaces:**
- Consumes: `reopen_device`, `reopen_on_bus` (Task 1).
- Produces: `dsd.rs`: `pub(super) fn leave_dsd_for_reopen(&mut self, bus: &BusKey, quiet: bool) -> bool` (whether the bus carried DSD).

- [ ] **Step 1: Write the failing tests**

In `crates/fp-engine/tests/dsd_output.rs`, add `DeviceSettings, Target, Wanted` to the `fp_model` import and append:

```rust
fn dac() -> OutputDevice {
    OutputDevice {
        backend: "offline".into(),
        device: "dac".into(),
    }
}

/// Plays a DSD track to DoP on the dac, then pauses it.
fn paused_dsd(r: &mut Rig) {
    let (left, right) = pattern();
    let path = dsf_file(r.dir.path(), "a.dsf", &left, &right);
    r.start(P, request(1, path, dsd64()));
    r.run_raw(SILENCE_FRAMES + BLOCK * 4);
    assert!(
        r.seen
            .iter()
            .any(|e| matches!(e, EngineEvent::DsdStarted { player, .. } if *player == P))
    );
    r.act(EngineAction::Pause { player: P });
    // Past the pause ramp: the bus is quiet.
    r.run_raw(BLOCK * 8);
}

/// Live settings spec L12, §8: the DSD mode changes while a DSD track is
/// paused on the device.
#[test]
fn a_dsd_mode_change_leaves_dsd_and_keeps_the_paused_track_paused() {
    let mut r = rig(DsdOutput::Dop, DsdMix::ConvertToPcm, SampleFormat::I24);
    paused_dsd(&mut r);
    let settings = DeviceSettings {
        dsd: DsdOutput::Pcm,
        dsd_mix: None,
        dsd_silence_ms: None,
        ..r.engine.running_settings(&dac()).unwrap()
    };
    r.act(EngineAction::ApplyDevice {
        device: dac(),
        settings,
        force: false,
    });
    r.settle();
    assert_eq!(r.dac.config().unwrap().dsd, None, "back to PCM");
    assert!(r.seen.contains(&EngineEvent::DsdEnded {
        player: P,
        entry: EntryId(1)
    }));
    assert!(r.seen.contains(&EngineEvent::Applied {
        target: Target::Device(dac()),
        wanted: Wanted::Device(settings),
        outcome: Ok(()),
    }));
    let out = r.run_raw(BLOCK * 10);
    assert!(
        out.iter().all(|f| f[0] == 0.0 && f[1] == 0.0),
        "the paused track stays paused"
    );
}

/// §8: bit-perfect turned off on a device carrying DSD (a paused DSD track).
#[test]
fn bit_perfect_off_leaves_dsd_and_reopens_shared() {
    let mut r = rig(DsdOutput::Dop, DsdMix::ConvertToPcm, SampleFormat::I24);
    paused_dsd(&mut r);
    let settings = DeviceSettings {
        sample_rate: 48_000,
        buffer_frames: BLOCK as u32,
        bit_perfect: false,
        dsd: DsdOutput::Pcm,
        dsd_mix: None,
        dsd_silence_ms: None,
    };
    r.act(EngineAction::ApplyDevice {
        device: dac(),
        settings,
        force: false,
    });
    r.settle();
    let config = r.dac.config().unwrap();
    assert_eq!((config.dsd, config.exclusive, config.sample_rate), (None, false, 48_000));
    let out = r.run_raw(BLOCK * 10);
    assert!(out.iter().all(|f| f[0] == 0.0 && f[1] == 0.0));
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-engine --test dsd_output a_dsd_mode_change bit_perfect_off_leaves`
Expected: FAIL: `dsd` is still `Some(Dop)` (Task 1 reopens a DSD config without leaving DSD).

- [ ] **Step 3: Write the implementation**

`crates/fp-engine/src/engine/dsd.rs`, after `dsd_silence_running`:

```rust
    /// Before a device change on `bus` (live settings spec L12): a bus that
    /// carries DSD goes back at once to the PCM stream it had before, so the
    /// change starts from PCM; the DSD sources on it are opened again as
    /// PCM by the caller. Returns whether the bus carried DSD. Only for a
    /// quiet bus or a forced change.
    pub(super) fn leave_dsd_for_reopen(&mut self, bus: &BusKey, quiet: bool) -> bool {
        let Some(d) = self.dsd_buses.remove(bus) else {
            return false;
        };
        if matches!(d.state, DsdState::Playing { .. } | DsdState::Tail { .. }) {
            self.events.push(EngineEvent::DsdEnded {
                player: d.player,
                entry: d.entry,
            });
        }
        let now_frame = self.now_frame(bus);
        self.send(
            bus,
            BusCommand::DsdMode {
                on: false,
                at_frame: 0,
            },
        );
        // Ends a native switch's hold.
        self.send(
            bus,
            BusCommand::HoldAll {
                from_frame: 0,
                until_frame: now_frame,
            },
        );
        let now = self.now;
        if let Some(b) = self.buses.get_mut(bus) {
            let mut budget = b.busy_budget(quiet);
            if let Err(error) = b.reopen_with(d.pcm, now, &mut budget) {
                tracing::warn!(?bus, %error, "cannot reopen the DSD device as PCM before the change");
            }
        }
        true
    }
```

`crates/fp-engine/src/engine.rs`, `reopen_device`: right after `let before_rate = self.rate_of(key);` add

```rust
        let left_dsd = self.leave_dsd_for_reopen(key, quiet);
```

and replace the final `if self.rate_of(key) != before_rate { … }` with

```rust
        // DSD sources become PCM ones; any source follows a new rate.
        if left_dsd || self.rate_of(key) != before_rate {
            self.reopen_on_bus(key, before_rate);
        }
```

Update the doc comment of `reopen_device`: "…in one reopen (L12; a bus carrying DSD first goes back to its PCM stream, then takes the change)…".

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-engine --test dsd_output`
Expected: PASS.

- [ ] **Step 5: Run the gate and commit**

Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`

```bash
git add crates/fp-engine/src/engine.rs crates/fp-engine/src/engine/dsd.rs crates/fp-engine/tests/dsd_output.rs
git commit -m "$(cat <<'EOF'
feat(engine): leave DSD before a device change

A device carrying DSD (a paused or loaded DSD track) goes back to PCM
before its new settings apply; the DSD track is opened again where it
was and stays paused (live settings spec L12, §8).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
EOF
)"
```

---

### Task 3: Refusals and absent devices (L14, L15)

**Files:**
- Modify: `crates/fp-engine/src/bus.rs` (`Bus` field `missing`; `Bus::open`; `try_open` L233-297; `reopen_with` L527-600; new `apply_config`, `forget_refusals`)
- Modify: `crates/fp-engine/src/engine.rs` (`reopen_device`)
- Test: `crates/fp-engine/tests/live_settings.rs`

**Interfaces:**
- Consumes: Task 1.
- Produces:
  - `Bus::apply_config(&mut self, config: StreamConfig, now: Instant, budget: &mut BusyBudget) -> Result<(), String>`
  - `Bus::forget_refusals(&mut self)`
  - `reopen_with` no longer remembers a configuration as refused when the device was not there (`DeviceNotFound`), as for `Busy`.

- [ ] **Step 1: Write the failing tests**

Append to `crates/fp-engine/tests/live_settings.rs`:

```rust
#[test]
fn l14_a_refused_rate_keeps_the_running_one_and_says_why() {
    let mut r = rig();
    r.main.refuse_rate(44_100);
    let running = settings_of(&r, "main");
    let wanted = DeviceSettings {
        sample_rate: 44_100,
        ..running
    };
    apply_device(&mut r, "main", wanted, false);
    assert_eq!(r.main.config().unwrap().sample_rate, 48_000);
    assert!(matches!(applied(&r.events, "main"), Some(Err(_))));
    assert_eq!(settings_of(&r, "main"), running);
}

#[test]
fn l14_a_device_busy_beyond_the_budget_keeps_the_running_settings() {
    let mut c = config();
    c.tuning.device_busy_retry_ms = 0.0;
    let mut r = rig_with(&c);
    // The first open and the three retries meet a busy device; the
    // restore of the running settings opens.
    r.main.set_busy(4);
    let wanted = DeviceSettings {
        buffer_frames: 960,
        ..settings_of(&r, "main")
    };
    apply_device(&mut r, "main", wanted, false);
    assert_eq!(r.main.config().unwrap().buffer_frames, 480);
    assert!(matches!(applied(&r.events, "main"), Some(Err(_))));
}

#[test]
fn l14_apply_now_asks_a_device_again_for_a_rate_it_refused() {
    let mut r = rig();
    r.main.refuse_rate(44_100);
    let wanted = DeviceSettings {
        sample_rate: 44_100,
        ..settings_of(&r, "main")
    };
    apply_device(&mut r, "main", wanted, false);
    let attempts = r.main.open_attempts();
    apply_device(&mut r, "main", wanted, true);
    assert!(r.main.open_attempts() > attempts, "asked again, not from memory");
}

#[test]
fn l15_an_absent_device_takes_the_new_settings_when_it_returns() {
    let mut r = rig();
    r.main.unplug();
    r.run(1);
    let wanted = DeviceSettings {
        buffer_frames: 960,
        ..settings_of(&r, "main")
    };
    apply_device(&mut r, "main", wanted, false);
    assert_eq!(applied(&r.events, "main"), Some(&Ok(())));
    assert_eq!(settings_of(&r, "main"), wanted);
    r.main.replug();
    r.clock += Duration::from_secs(3);
    r.tick();
    assert_eq!(r.main.config().unwrap().buffer_frames, 960);
}

#[test]
fn l15_a_device_gone_during_the_change_keeps_the_new_settings() {
    let mut r = rig();
    r.main.unplug();
    let wanted = DeviceSettings {
        buffer_frames: 960,
        ..settings_of(&r, "main")
    };
    apply_device(&mut r, "main", wanted, false);
    assert_eq!(applied(&r.events, "main"), Some(&Ok(())));
    r.main.replug();
    r.clock += Duration::from_secs(3);
    r.tick();
    assert_eq!(r.main.config().unwrap().buffer_frames, 960);
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-engine --test live_settings l14_ l15_`
Expected: FAIL: `l14_apply_now_asks…` (no new open attempt: the refusal is answered from memory), `l15_…` (`Err` reported, old buffer back).

- [ ] **Step 3: Write the implementation**

`crates/fp-engine/src/bus.rs`:
- Field after `rate_change`: `/// The last open failed because the device was not there.\n    missing: bool,` and `missing: false,` in `Bus::open`.
- In `try_open`, right after `self.exclusive_granted = self.config.exclusive && opened.is_ok();` (and again after the shared fallback reassigns `opened`, i.e. just before `match opened {`), set:

```rust
        self.missing = matches!(opened, Err(BackendError::DeviceNotFound(_)));
```

  (Put it once, immediately before `match opened {`.)
- In `reopen_with`, replace

```rust
        if matches!(opened, Err(BackendError::Busy(_))) {
            // Not a refusal of the configuration: the next start asks again.
            tracing::warn!(…busy…);
        } else {
            tracing::warn!(…refused…);
            match config.dsd { … }
        }
```

  with

```rust
        match &opened {
            // Not a refusal of the configuration: the next start asks again.
            Err(BackendError::Busy(_)) => {
                tracing::warn!(bus = ?self.key, rate = config.sample_rate, dsd = ?config.dsd, %error, "device busy (another application or the sound server may be using it); keeping the previous stream");
            }
            Err(BackendError::DeviceNotFound(_)) => {
                tracing::warn!(bus = ?self.key, %error, "device not there; keeping the previous configuration");
            }
            _ => {
                tracing::warn!(bus = ?self.key, rate = config.sample_rate, dsd = ?config.dsd, %error, "stream refused; keeping the previous one");
                match config.dsd {
                    Some(dsd) => {
                        self.refused_dsd.insert((config.sample_rate, dsd));
                    }
                    None => {
                        self.refused_rates.insert(config.sample_rate);
                    }
                }
            }
        }
```

  and update its doc comment's sentence "A device that stays busy through `busy_retries` refused nothing" to "A device that stays busy through `busy_retries`, or is not there, refused nothing".
- New methods after `reopen_with`:

```rust
    /// A device change (live settings spec L12, L14, L15): `reopen_with`,
    /// except that a device that is not there, missing now or lost
    /// already, keeps `config` for when it returns: nothing is
    /// interrupted, the bus stays `Lost` on the virtual clock, and the
    /// watchdog opens `config` (falling back to `pcm_fallback` if refused).
    pub fn apply_config(
        &mut self,
        config: StreamConfig,
        now: Instant,
        budget: &mut BusyBudget,
    ) -> Result<(), String> {
        let absent = self.health == BusHealth::Lost && self.stream.is_none();
        if !absent {
            match self.reopen_with(config, now, budget) {
                Ok(()) => return Ok(()),
                Err(error) if !self.missing => return Err(error),
                Err(_) => {}
            }
        }
        let previous = self.config;
        self.stream = None;
        self.config = config;
        self.follow_rate(previous.sample_rate, config.sample_rate);
        self.health = BusHealth::Lost;
        // The stand-in clock runs at the new rate.
        self.virtual_clock = None;
        self.start_virtual_clock();
        tracing::info!(bus = ?self.key, "device not there; it opens with the new settings when it returns");
        Ok(())
    }

    /// Forgets the rates and DSD streams this device refused, so that the
    /// operator's Apply now asks it again (L14).
    pub fn forget_refusals(&mut self) {
        self.refused_rates.clear();
        self.refused_dsd.clear();
    }
```

`crates/fp-engine/src/engine.rs`, `reopen_device`: rename `_force` to `force`; before `let mut budget = bus.busy_budget(quiet);` add

```rust
        if force {
            bus.forget_refusals();
        }
```

and replace `bus.reopen_with(config, now, &mut budget)` with `bus.apply_config(config, now, &mut budget)`. Extend its doc comment: "…A refusal brings the running configuration back (L14); a device that is not there keeps the new one for when it returns (L15). A forced change asks again for what the device refused before."

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-engine`
Expected: PASS (including `bus.rs`: `a_rate_the_device_refuses_is_still_remembered`, `a_device_busy_beyond_the_retries_keeps_the_rate_without_refusing_it`).

- [ ] **Step 5: Run the gate and commit**

Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`

```bash
git add crates/fp-engine/src/bus.rs crates/fp-engine/src/engine.rs crates/fp-engine/tests/live_settings.rs
git commit -m "$(cat <<'EOF'
feat(engine): keep the running settings on a refusal, the new ones for an absent device

A device that refuses a change goes back to what it ran and the model
hears why (live settings spec L14); Apply now asks it again. A device
that is not there takes the new settings when it returns (L15).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
EOF
)"
```

---

### Task 4: Route changes and closing unused buses (L6, L13, L16)

**Files:**
- Modify: `crates/fp-engine/src/engine.rs` (`PlayerRuntime` fields L273-299; `add_player`; `remove_player`; `Engine` field `orphans`; `execute`; `run_kept`; `tick`; new methods)
- Modify: `crates/fp-engine/src/engine/carts.rs` (`CartwallRuntime` fields L36-45; `ensure_cartwall`; `follow_forced_rate_carts` L197-243; new methods)
- Test: `crates/fp-engine/tests/live_settings.rs`

**Interfaces:**
- Consumes: `reopen_player_sources`, `bus_quiet`, `player_quiet`, `report_applied`, `keep`/`run_kept` (Task 1); `report_placement` (plan 2).
- Produces:
  - `PlayerRuntime::{main_route, cue_route}: Option<Route>`, `CartwallRuntime::{main_route, cue_route}: Option<Route>` (the routes the engine holds)
  - field `orphans: std::collections::BTreeSet<BusKey>`; `fn close_orphans(&mut self)`
  - `fn holder_quiet(&self, holder: Holder) -> bool`, `fn make_room(&mut self, target: &(BusKey, u16), force: bool) -> bool`
  - `fn apply_route(&mut self, holder: Holder, route: Option<Route>, force: bool) -> bool`
  - `fn move_player_main(&mut self, player: PlayerId, route: Option<Route>, force: bool) -> bool`, `fn move_player_cue(…) -> bool`, `fn place_player_cue(&mut self, player: PlayerId, cue: Option<(BusKey, u16)>, route: Option<Route>)`, `fn end_player_cue(&mut self, player: PlayerId)`
  - `carts.rs`: `pub(super) fn move_cartwall(&mut self, holder: Holder, route: Option<Route>, force: bool) -> bool`, `pub(super) fn cart_path_quiet(&self, cue: bool) -> bool`, `pub(super) fn cartwall_uses(&self, key: &BusKey) -> bool`, `fn reopen_carts(&mut self, bus: &BusKey, main: bool, cue: bool, old_rate: u32)`, `fn place_cart_cue(…)`, `fn end_cart_cue(&mut self)`

- [ ] **Step 1: Write the failing tests**

Append to `crates/fp-engine/tests/live_settings.rs`:

```rust
fn route_applied(events: &[EngineEvent], holder: Holder) -> bool {
    events.iter().any(|e| {
        matches!(e, EngineEvent::Applied { target: Target::Route(h), outcome: Ok(()), .. } if *h == holder)
    })
}

#[test]
fn l16_a_moved_player_keeps_its_paused_track_paused_on_the_new_device() {
    let mut r = rig();
    r.act(EngineAction::LoadPaused {
        player: P,
        request: request(1, 2.0),
    });
    r.settle();
    r.take_events();
    r.act(EngineAction::ApplyRoute {
        holder: Holder::PlayerMain(P),
        route: Some(route("other")),
        force: false,
    });
    r.settle();
    let events = r.take_events();
    assert!(events.iter().any(|e| matches!(
        e,
        EngineEvent::Placed { holder: Holder::PlayerMain(p), device, .. }
            if *p == P && *device == out("other")
    )));
    assert!(route_applied(&events, Holder::PlayerMain(P)));
    r.run(3);
    let position = r.engine.telemetry(P).position_secs.unwrap();
    assert!((position - 2.0).abs() < 0.01, "{position}");
    assert!(r.other.is_open());
    assert!(!r.main.is_open(), "L13: closed once nothing uses it");
}

#[test]
fn l16_a_route_change_waits_for_its_own_holder_only() {
    let mut c = config();
    c.outputs.routes.push(PlayerRoutes {
        player: Q,
        main: Some(route("main")),
        cue: None,
    });
    let mut r = rig_with(&c);
    r.act(EngineAction::AddPlayer { player: Q });
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, 0.0),
    });
    r.settle();
    r.run(2);
    r.act(EngineAction::ApplyRoute {
        holder: Holder::PlayerMain(Q),
        route: Some(route("other")),
        force: false,
    });
    r.tick();
    assert!(route_applied(&r.events, Holder::PlayerMain(Q)), "Q is idle");
    let heard = r.run(3);
    assert!(heard.iter().all(|v| *v != 0.0), "P plays on, uninterrupted");
    assert!(r.main.is_open(), "P still uses it");
    r.act(EngineAction::ApplyRoute {
        holder: Holder::PlayerMain(P),
        route: Some(route("other")),
        force: false,
    });
    r.run(2);
    assert!(!route_applied(&r.events, Holder::PlayerMain(P)), "P sounds");
    r.act(EngineAction::StopNow { player: P });
    r.run(5);
    assert!(route_applied(&r.events, Holder::PlayerMain(P)));
}

#[test]
fn l16_removing_a_cue_route_by_force_ends_the_open_cue() {
    let mut c = config();
    c.outputs.routes[0].cue = Some(route("other"));
    let mut r = rig_with(&c);
    r.act(EngineAction::StartCue {
        player: P,
        request: request(2, 0.0),
    });
    r.settle();
    r.run(2);
    r.act(EngineAction::ApplyRoute {
        holder: Holder::PlayerCue(P),
        route: None,
        force: false,
    });
    r.run(2);
    assert!(!route_applied(&r.events, Holder::PlayerCue(P)), "the CUE sounds");
    r.act(EngineAction::ApplyRoute {
        holder: Holder::PlayerCue(P),
        route: None,
        force: true,
    });
    r.tick();
    assert!(r.events.contains(&EngineEvent::CueEnded {
        player: P,
        entry: EntryId(2)
    }));
    assert!(r.events.contains(&EngineEvent::Unplaced {
        holder: Holder::PlayerCue(P),
        route: None
    }));
}

#[test]
fn l16_the_cartwall_before_its_first_cart_only_takes_the_route() {
    let mut r = rig();
    r.take_events();
    r.act(EngineAction::ApplyRoute {
        holder: Holder::CartwallMain,
        route: Some(route("other")),
        force: false,
    });
    let events = r.take_events();
    assert!(events.contains(&EngineEvent::Unplaced {
        holder: Holder::CartwallMain,
        route: Some(route("other"))
    }));
    assert!(route_applied(&events, Holder::CartwallMain));
    assert!(!r.other.is_open());
}

#[test]
fn l16_a_route_to_channels_the_open_device_lacks_reopens_it_with_more() {
    let mut c = config();
    c.outputs.routes[0].main = Some(route("other"));
    let mut r = rig_with(&c);
    assert_eq!(r.other.config().unwrap().channels, 2);
    let pair = Route {
        first_channel: 2,
        ..route("other")
    };
    c.outputs.routes[0].cue = Some(pair.clone());
    r.act(EngineAction::UpdateSettings(Box::new(c)));
    r.act(EngineAction::ApplyRoute {
        holder: Holder::PlayerCue(P),
        route: Some(pair),
        force: false,
    });
    assert_eq!(r.other.config().unwrap().channels, 4);
    assert!(route_applied(&r.events, Holder::PlayerCue(P)));
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-engine --test live_settings l16_`
Expected: FAIL: no `Applied` for a route (the engine ignores `ApplyRoute`).

- [ ] **Step 3: Write the implementation**

`crates/fp-engine/src/engine.rs`:

1. `PlayerRuntime` (after `cue: Option<(BusKey, u16)>,`):

```rust
    /// The routes the engine holds for this player's Main and Cue (live
    /// settings spec L3, L16).
    main_route: Option<Route>,
    cue_route: Option<Route>,
```

   In `add_player`, initialise them in the `PlayerRuntime { … }` literal with `main_route: main_route.clone(), cue_route: cue_route.clone(),` (the locals from plan 2).

2. `Engine` field after `kept`:

```rust
    /// Buses a holder left (a route or the audio system changed, a player
    /// was removed): closed once nothing uses them and they are quiet (L13).
    orphans: std::collections::BTreeSet<BusKey>,
```

   and `orphans: std::collections::BTreeSet::new(),` in `Engine::new`.

3. `remove_player`: inside `if let Some(mut rt) = …`, before the `Gone` events, add

```rust
            self.orphans.insert(rt.main.0.clone());
            if let Some((key, _)) = &rt.cue {
                self.orphans.insert(key.clone());
            }
```

4. New methods in `impl Engine`:

```rust
    /// L6, L11: whether `holder`'s own sources are silent; a route change
    /// waits for its holder only.
    fn holder_quiet(&self, holder: Holder) -> bool {
        match holder {
            Holder::PlayerMain(p) => self
                .players
                .get(&p)
                .is_none_or(|rt| self.player_quiet(rt, |s| !s.cue)),
            Holder::PlayerCue(p) => self
                .players
                .get(&p)
                .is_none_or(|rt| self.player_quiet(rt, |s| s.cue)),
            Holder::CartwallMain => self.cart_path_quiet(false),
            Holder::CartwallCue => self.cart_path_quiet(true),
        }
    }

    /// Whether `target` can carry a route at its channel pair (L16): an
    /// open device with too few channels is reopened with more while it is
    /// quiet (or the change is forced); otherwise the change waits. A
    /// device that has no more channels keeps what it has.
    fn make_room(&mut self, target: &(BusKey, u16), force: bool) -> bool {
        let Some(open) = self.buses.get(&target.0).map(|b| b.config().channels) else {
            return true;
        };
        let channels = self.channels_for(&target.0);
        if channels <= open {
            return true;
        }
        let quiet = self.bus_quiet(&target.0);
        if !quiet && !force {
            return false;
        }
        let now = self.now;
        if let Some(bus) = self.buses.get_mut(&target.0) {
            let config = StreamConfig {
                channels,
                ..bus.config()
            };
            let mut budget = bus.busy_budget(quiet);
            if let Err(error) = bus.reopen_with(config, now, &mut budget) {
                tracing::warn!(bus = ?target.0, %error, "cannot open more channels on the device");
            }
        }
        true
    }

    /// L16: `ApplyRoute`. Returns `false` while it must wait (L11): the
    /// holder still sounds, or its new device must open more channels and
    /// sounds, and the change is not forced.
    fn apply_route(&mut self, holder: Holder, route: Option<Route>, force: bool) -> bool {
        if !force && !self.holder_quiet(holder) {
            return false;
        }
        let done = match holder {
            Holder::PlayerMain(p) => self.move_player_main(p, route.clone(), force),
            Holder::PlayerCue(p) => self.move_player_cue(p, route.clone(), force),
            Holder::CartwallMain | Holder::CartwallCue => {
                self.move_cartwall(holder, route.clone(), force)
            }
        };
        if done {
            self.report_applied(Target::Route(holder), Wanted::Route(route), Ok(()));
        }
        done
    }

    /// L16: `player`'s Main goes to `route` (the default output without
    /// one). Its sources are opened again on the new bus at their position:
    /// paused stays paused, waiting stays waiting. A Cue route equal to the
    /// new Main is dropped; one that differed again comes back.
    fn move_player_main(&mut self, player: PlayerId, route: Option<Route>, force: bool) -> bool {
        let target = match &route {
            Some(r) => self.route_target(r),
            None => (self.default_output(), 0),
        };
        if !self.make_room(&target, force) {
            return false;
        }
        let now = self.now;
        let Some(rt) = self.players.get_mut(&player) else {
            return true;
        };
        let old = std::mem::replace(&mut rt.main, target.clone());
        rt.main_route = route.clone();
        // The pause ramp was in the old bus's frames.
        rt.pause_ramp_ends = 0;
        let cue_route = rt.cue_route.clone();
        let cue_now = rt.cue.clone();
        self.ensure_bus(&target.0, now);
        if old != target {
            let rate = self.rate_of(&old.0);
            self.reopen_player_sources(player, |p| !p.cue && p.bus == old.0, rate);
            self.orphans.insert(old.0);
        }
        self.report_placement(Holder::PlayerMain(player), route, Some(&target.0));
        let cue = cue_route
            .as_ref()
            .and_then(|r| self.cue_target(r, &target));
        if cue != cue_now {
            self.place_player_cue(player, cue, cue_route);
        }
        true
    }

    /// L16: `player`'s Cue goes to `route` (no Cue without one, or when it
    /// equals Main).
    fn move_player_cue(&mut self, player: PlayerId, route: Option<Route>, force: bool) -> bool {
        let Some(main) = self.players.get(&player).map(|rt| rt.main.clone()) else {
            return true;
        };
        let cue = route.as_ref().and_then(|r| self.cue_target(r, &main));
        if let Some(target) = &cue
            && !self.make_room(target, force)
        {
            return false;
        }
        self.place_player_cue(player, cue, route);
        true
    }

    /// Puts `player`'s CUE path on `cue`: its sources move there at their
    /// position; without a target an open CUE ends (`CueEnded`, only
    /// reachable when forced). Reports the placement.
    fn place_player_cue(
        &mut self,
        player: PlayerId,
        cue: Option<(BusKey, u16)>,
        route: Option<Route>,
    ) {
        let now = self.now;
        let Some(rt) = self.players.get_mut(&player) else {
            return;
        };
        let old = std::mem::replace(&mut rt.cue, cue.clone());
        rt.cue_route = route.clone();
        if let Some((key, _)) = &cue {
            self.ensure_bus(key, now);
        }
        if old != cue
            && let Some((old_key, _)) = &old
        {
            self.orphans.insert(old_key.clone());
            if cue.is_some() {
                let rate = self.rate_of(old_key);
                self.reopen_player_sources(player, |p| p.cue && &p.bus == old_key, rate);
            } else {
                self.end_player_cue(player);
            }
        }
        self.report_placement(Holder::PlayerCue(player), route, cue.as_ref().map(|c| &c.0));
    }

    /// The CUE of `player` ends because its output went away (L16).
    fn end_player_cue(&mut self, player: PlayerId) {
        let Some(rt) = self.players.get_mut(&player) else {
            return;
        };
        rt.cue_paused = false;
        let cue = rt.cue_src.take();
        let tails = std::mem::take(&mut rt.cue_outgoing);
        if let Some(cue) = cue {
            let entry = cue.entry;
            self.send(&cue.bus, BusCommand::Cancel { slot: cue.slot });
            self.release(cue);
            self.events.push(EngineEvent::CueEnded { player, entry });
        }
        for p in tails {
            self.send(&p.bus, BusCommand::Cancel { slot: p.slot });
            self.release(p);
        }
    }

    /// L13: closes the buses holders left, once no holder uses them,
    /// nothing on them sounds and the mixer has handed every slot back. The
    /// device is released (exclusive access matters).
    fn close_orphans(&mut self) {
        let orphans: Vec<BusKey> = self.orphans.iter().cloned().collect();
        for key in orphans {
            let used = self.players.values().any(|rt| {
                rt.main.0 == key || rt.cue.as_ref().is_some_and(|c| c.0 == key)
            }) || self.cartwall_uses(&key);
            if used {
                self.orphans.remove(&key);
                continue;
            }
            let idle = self.bus_quiet(&key)
                && self.buses.get(&key).is_none_or(|b| b.used_slots() == 0);
            if !idle {
                continue;
            }
            self.orphans.remove(&key);
            if self.buses.remove(&key).is_some() {
                self.running.remove(&key);
                self.dsd_buses.remove(&key);
                self.tune_due.remove(&key);
                tracing::info!(bus = ?key, "output closed: no output uses it any more");
            }
        }
    }
```

5. `execute`: replace the remaining no-op live arm so that routes are kept too:

```rust
            apply @ (EngineAction::ApplyDevice { .. } | EngineAction::ApplyRoute { .. }) => {
                self.keep(apply)
            }
            // Live settings: the audio system comes with task 5 of plan 3.
            EngineAction::ApplyAudioSystem { .. } => {}
```

6. `run_kept`: add the arm

```rust
                EngineAction::ApplyRoute {
                    holder,
                    route,
                    force,
                } => self.apply_route(holder, route, force),
```

7. `tick`: after `self.run_kept();` add `self.close_orphans();`.

`crates/fp-engine/src/engine/carts.rs`:

1. `CartwallRuntime` (after `cue: Option<(BusKey, u16)>,`): `/// The routes the engine holds for the cartwall (L3, L16).\n    main_route: Option<Route>,\n    cue_route: Option<Route>,` and in `ensure_cartwall`'s `CartwallRuntime { … }` add `main_route: routes.main.clone(), cue_route: routes.cue.clone(),` (before `routes` moves into `created`).

2. Replace `follow_forced_rate_carts` with:

```rust
    /// The cart part of `reopen_on_bus`.
    pub(super) fn follow_forced_rate_carts(&mut self, bus: &BusKey, old_rate: u32) {
        self.reopen_carts(bus, true, true, old_rate);
    }

    /// Cart sources on `bus` opened again where the cartwall's routes point
    /// now, at their position measured at `old_rate` (a looped one from its
    /// cue-in, since its loop restarts where it is opened): the playing
    /// carts when `main`, the pre-listen when `cue`. A cart de-clicking on
    /// `bus` is cut.
    fn reopen_carts(&mut self, bus: &BusKey, main: bool, cue: bool, old_rate: u32) {
        let Some(c) = self.cartwall.as_mut() else {
            return;
        };
        let on_bus = |s: &CartSource| &s.bus == bus;
        let (stopping, kept): (Vec<CartSource>, Vec<CartSource>) = std::mem::take(&mut c.stopping)
            .into_iter()
            .partition(on_bus);
        c.stopping = kept;
        let (playing, kept): (Vec<CartSource>, Vec<CartSource>) = if main {
            std::mem::take(&mut c.playing).into_iter().partition(on_bus)
        } else {
            (Vec::new(), std::mem::take(&mut c.playing))
        };
        c.playing = kept;
        let cue_src = if cue {
            c.cue_src.take_if(|s| on_bus(s))
        } else {
            None
        };
        // … from here on the old body, unchanged, with `cue` renamed to
        // `cue_src` where it named the taken pre-listen:
        for source in stopping {
            self.send(&source.bus, BusCommand::Cancel { slot: source.slot });
            self.release_cart(&source);
        }
        let old = f64::from(old_rate.max(1));
        let again = playing
            .into_iter()
            .map(|s| (s, false))
            .chain(cue_src.into_iter().map(|s| (s, true)));
        for (source, is_cue) in again {
            let mut request = source.request.clone();
            if !source.looped {
                request.from_secs = source.from_secs + source.shared.frames_played() as f64 / old;
            }
            self.send(&source.bus, BusCommand::Cancel { slot: source.slot });
            self.release_cart(&source);
            match self.cart_source(&request, is_cue) {
                Ok(fresh) => {
                    if let Some(c) = self.cartwall.as_mut() {
                        if is_cue {
                            c.cue_src = Some(fresh);
                        } else {
                            c.playing.push(fresh);
                        }
                    }
                }
                Err(_) => self.events.push(if is_cue {
                    EngineEvent::CartCueEnded { cart: request.cart }
                } else {
                    EngineEvent::CartEnded { cart: request.cart }
                }),
            }
        }
    }
```

3. New methods:

```rust
    /// Whether the cartwall's Main (`cue == false`) or Cue path is silent:
    /// no cart playing (or pre-listened) and none de-clicking on its bus.
    pub(super) fn cart_path_quiet(&self, cue: bool) -> bool {
        self.cartwall.as_ref().is_none_or(|c| {
            let bus = if cue {
                c.cue.as_ref().map(|k| &k.0)
            } else {
                Some(&c.main.0)
            };
            let sources_gone = if cue {
                c.cue_src.is_none()
            } else {
                c.playing.is_empty()
            };
            sources_gone && c.stopping.iter().all(|s| Some(&s.bus) != bus)
        })
    }

    /// Whether the cartwall plays or pre-listens on `key`.
    pub(super) fn cartwall_uses(&self, key: &BusKey) -> bool {
        self.cartwall.as_ref().is_some_and(|c| {
            c.main.0 == *key || c.cue.as_ref().is_some_and(|k| k.0 == *key)
        })
    }

    /// L16 for the cartwall. Before its first cart only the route changes:
    /// the cartwall opens with the current configuration (L13). Returns
    /// `false` while the new device must open more channels and sounds.
    pub(super) fn move_cartwall(&mut self, holder: Holder, route: Option<Route>, force: bool) -> bool {
        let Some(main) = self.cartwall.as_ref().map(|c| c.main.clone()) else {
            self.events.push(EngineEvent::Unplaced { holder, route });
            return true;
        };
        match holder {
            Holder::CartwallMain => {
                let target = match &route {
                    Some(r) => self.route_target(r),
                    None => (self.default_output(), 0),
                };
                if !self.make_room(&target, force) {
                    return false;
                }
                let now = self.now;
                let Some(c) = self.cartwall.as_mut() else {
                    return true;
                };
                c.main = target.clone();
                c.main_route = route.clone();
                let cue_route = c.cue_route.clone();
                let cue_now = c.cue.clone();
                self.ensure_bus(&target.0, now);
                if main != target {
                    let rate = self.rate_of(&main.0);
                    self.reopen_carts(&main.0, true, false, rate);
                    self.orphans.insert(main.0);
                }
                self.report_placement(Holder::CartwallMain, route, Some(&target.0));
                let cue = cue_route
                    .as_ref()
                    .and_then(|r| self.cue_target(r, &target));
                if cue != cue_now {
                    self.place_cart_cue(cue, cue_route);
                }
            }
            Holder::CartwallCue => {
                let cue = route.as_ref().and_then(|r| self.cue_target(r, &main));
                if let Some(target) = &cue
                    && !self.make_room(target, force)
                {
                    return false;
                }
                self.place_cart_cue(cue, route);
            }
            Holder::PlayerMain(_) | Holder::PlayerCue(_) => {}
        }
        true
    }

    /// Puts the cart pre-listen path on `cue`; without a target an open
    /// pre-listen ends (`CartCueEnded`).
    fn place_cart_cue(&mut self, cue: Option<(BusKey, u16)>, route: Option<Route>) {
        let now = self.now;
        let Some(c) = self.cartwall.as_mut() else {
            return;
        };
        let old = std::mem::replace(&mut c.cue, cue.clone());
        c.cue_route = route.clone();
        if let Some((key, _)) = &cue {
            self.ensure_bus(key, now);
        }
        if old != cue
            && let Some((old_key, _)) = &old
        {
            self.orphans.insert(old_key.clone());
            if cue.is_some() {
                let rate = self.rate_of(old_key);
                self.reopen_carts(old_key, false, true, rate);
            } else {
                self.end_cart_cue();
            }
        }
        self.report_placement(Holder::CartwallCue, route, cue.as_ref().map(|c| &c.0));
    }

    /// The pre-listen ends because its output went away (L16).
    fn end_cart_cue(&mut self) {
        if let Some(source) = self.cartwall.as_mut().and_then(|c| c.cue_src.take()) {
            let cart = source.cart;
            self.send(&source.bus, BusCommand::Cancel { slot: source.slot });
            self.release_cart(&source);
            self.events.push(EngineEvent::CartCueEnded { cart });
        }
    }
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-engine`
Expected: PASS. If an existing test expected a device to stay open after `RemovePlayer` (L13 now closes it once unused), check that the closure is what the spec asks for and update the test's expectation with a comment citing L13.

- [ ] **Step 5: Run the gate and commit**

Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`

```bash
git add -A crates/fp-engine
git commit -m "$(cat <<'EOF'
feat(engine): move a player or the cartwall to another output while running

ApplyRoute (live settings spec L16) waits only for the holder it moves
(L6), opens its sources again on the new device where they were, and
closes a device nothing uses any more (L13). A removed Cue output ends
an open CUE.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
EOF
)"
```

---

### Task 5: Switching the audio system (L7, L17)

**Files:**
- Modify: `crates/fp-engine/src/engine.rs` (`execute`, `run_kept`, new `apply_audio_system`)
- Test: `crates/fp-engine/tests/live_settings.rs`

**Interfaces:**
- Consumes: `choose_backend` (plan 2), `move_player_main`, `move_cartwall`, `bus_quiet`, `report_applied` (Tasks 1, 4).
- Produces: `fn apply_audio_system(&mut self, backend: Option<String>, force: bool) -> bool`; `cartwall_main_route(&self) -> Option<Option<Route>>` in `carts.rs` (the cartwall's held Main route, `None` before its first cart).

- [ ] **Step 1: Write the failing tests**

Append to `crates/fp-engine/tests/live_settings.rs`:

```rust
/// P on the default output (`main`), Q on `other` by name.
fn two_players() -> Rig {
    let mut c = config();
    c.outputs.routes = vec![
        PlayerRoutes {
            player: P,
            main: None,
            cue: None,
        },
        PlayerRoutes {
            player: Q,
            main: Some(route("other")),
            cue: None,
        },
    ];
    let mut r = rig_with(&c);
    r.act(EngineAction::AddPlayer { player: Q });
    r
}

#[test]
fn l17_a_new_audio_system_moves_only_the_holders_on_the_default_output() {
    let mut r = two_players();
    r.act(EngineAction::LoadPaused {
        player: P,
        request: request(1, 1.0),
    });
    r.settle();
    r.take_events();
    r.act(EngineAction::ApplyAudioSystem {
        backend: Some("null".into()),
        force: false,
    });
    r.settle();
    let events = r.take_events();
    assert!(events.contains(&EngineEvent::AudioSystemInUse {
        configured: Some("null".into()),
        in_use: "null".into(),
    }));
    assert!(events.iter().any(|e| matches!(
        e,
        EngineEvent::Placed { holder: Holder::PlayerMain(p), device, .. }
            if *p == P && device.backend == "null"
    )));
    assert!(
        !events.iter().any(|e| matches!(
            e,
            EngineEvent::Placed { holder: Holder::PlayerMain(q), .. } if *q == Q
        )),
        "a route that names a backend keeps it"
    );
    assert!(events.contains(&EngineEvent::Applied {
        target: Target::AudioSystem,
        wanted: Wanted::AudioSystem(Some("null".into())),
        outcome: Ok(()),
    }));
    assert_eq!(r.engine.backend_in_use(), "null");
    let position = r.engine.telemetry(P).position_secs.unwrap();
    assert!((position - 1.0).abs() < 0.01, "{position}");
    r.run(3);
    assert!(!r.main.is_open(), "nothing uses it any more");
}

#[test]
fn l17_the_audio_system_waits_until_every_device_is_quiet() {
    let mut r = two_players();
    r.act(EngineAction::StartCurrent {
        player: Q,
        request: request(2, 0.0),
    });
    r.settle();
    r.run(2);
    r.act(EngineAction::ApplyAudioSystem {
        backend: Some("null".into()),
        force: false,
    });
    r.run(2);
    assert_eq!(r.engine.backend_in_use(), "offline", "Q plays on other");
    r.act(EngineAction::StopNow { player: Q });
    r.run(5);
    assert_eq!(r.engine.backend_in_use(), "null");
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-engine --test live_settings l17_`
Expected: FAIL: `backend_in_use()` stays `"offline"`.

- [ ] **Step 3: Write the implementation**

`crates/fp-engine/src/engine/carts.rs`:

```rust
    /// The cartwall's held Main route; `None` before its first cart.
    pub(super) fn cartwall_main_route(&self) -> Option<Option<Route>> {
        self.cartwall.as_ref().map(|c| c.main_route.clone())
    }
```

`crates/fp-engine/src/engine.rs`, new method:

```rust
    /// L17: the default output moves to `backend`'s choice. Waits (L7,
    /// L11) until every bus is quiet unless forced. Every holder whose
    /// route does not name a usable backend (no route, or a route to a
    /// backend this machine lacks) is re-placed as a route change does;
    /// routes that name a backend keep it.
    fn apply_audio_system(&mut self, backend: Option<String>, force: bool) -> bool {
        if !force && !self.buses.keys().all(|k| self.bus_quiet(k)) {
            return false;
        }
        self.backend_in_use = choose_backend(&self.backends, backend.as_deref());
        self.audio_system = backend.clone();
        let on_default =
            |route: &Option<Route>| route.as_ref().is_none_or(|r| self.route_backend(&r.backend).is_none());
        let players: Vec<(PlayerId, Option<Route>)> = self
            .players
            .iter()
            .filter(|(_, rt)| on_default(&rt.main_route))
            .map(|(id, rt)| (*id, rt.main_route.clone()))
            .collect();
        let cartwall = self
            .cartwall_main_route()
            .filter(|route| on_default(route));
        for (player, route) in players {
            self.move_player_main(player, route, true);
        }
        if let Some(route) = cartwall {
            self.move_cartwall(Holder::CartwallMain, route, true);
        }
        self.events.push(EngineEvent::AudioSystemInUse {
            configured: backend.clone(),
            in_use: self.backend_in_use.clone(),
        });
        self.report_applied(Target::AudioSystem, Wanted::AudioSystem(backend), Ok(()));
        true
    }
```

  (`on_default` borrows `self` immutably; collect `players` and `cartwall` before the moves, as written.)

- `execute`: the live arm becomes

```rust
            apply @ (EngineAction::ApplyAudioSystem { .. }
            | EngineAction::ApplyRoute { .. }
            | EngineAction::ApplyDevice { .. }) => self.keep(apply),
```

- `run_kept`: add

```rust
                EngineAction::ApplyAudioSystem { backend, force } => {
                    self.apply_audio_system(backend, force)
                }
```

`docs/technical/backends.md`: at the end of the section that describes the default backend choice (search for `choose_default_backend`; add the paragraph after the paragraph that mentions it), add:

```markdown
The engine makes the choice (`Engine::new`, `choose_backend`) and reports
it to the model (`EngineEvent::AudioSystemInUse { configured, in_use }`);
the status bar shows `in_use`. Changing **Audio system** in Settings applies
while the application runs (live settings spec L17) once nothing plays on
any device: the engine chooses again with the same rule, re-places every
holder on the default output (no route, or a route to a backend this
machine lacks), closes the buses nothing uses any more, and reports the new
choice. Routes that name a backend keep it.
```

If `backends.md` never names `choose_default_backend`, put the paragraph at the end of its "## Implemented" section.

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-engine`
Expected: PASS.

- [ ] **Step 5: Run the gate and commit**

Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`

```bash
git add crates/fp-engine docs/technical/backends.md
git commit -m "$(cat <<'EOF'
feat(engine): switch the audio system while running

ApplyAudioSystem (live settings spec L17) waits until every device is
quiet, chooses the backend again and moves only what plays on the
default output.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
EOF
)"
```

---

### Task 6: Forced changes, end to end, and the technical docs (L21)

**Files:**
- Test: `crates/fp-engine/tests/live_settings.rs`, `crates/fp-engine/tests/conductor.rs`
- Modify: `docs/technical/audio-engine.md` ("## Live settings" from plan 2; the action table under "## Engine")
- Modify: `docs/technical/architecture.md` (the numbered data-flow list, ~L49-58)

**Interfaces:**
- Consumes: everything above. No new code is expected; if a test fails, fix the engine where the test points (TDD: the test comes first).

- [ ] **Step 1: Write the tests**

Append to `crates/fp-engine/tests/live_settings.rs`:

```rust
#[test]
fn l21_a_forced_change_resumes_what_played_and_starts_nothing_paused() {
    let mut c = config();
    c.outputs.routes.push(PlayerRoutes {
        player: Q,
        main: Some(route("main")),
        cue: None,
    });
    let mut r = rig_with(&c);
    r.act(EngineAction::AddPlayer { player: Q });
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, 0.0),
    });
    r.act(EngineAction::LoadPaused {
        player: Q,
        request: request(2, 3.0),
    });
    r.settle();
    r.run(5);
    let wanted = DeviceSettings {
        sample_rate: 44_100,
        ..settings_of(&r, "main")
    };
    apply_device(&mut r, "main", wanted, true);
    assert_eq!(r.main.config().unwrap().sample_rate, 44_100, "forced: at once");
    r.settle();
    let heard = r.run(10);
    assert!(
        heard.iter().any(|v| (*v as u64) / 100_000 == 1),
        "P plays again after the gap"
    );
    assert!(
        !heard.iter().any(|v| (*v as u64) / 100_000 == 2),
        "Q stays paused (rule 10)"
    );
    let q = r.engine.telemetry(Q).position_secs.unwrap();
    assert!((q - 3.0).abs() < 0.01, "{q}");
}

#[test]
fn l21_a_forced_change_cuts_a_fade_and_reports_its_end() {
    let mut r = rig();
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, 0.0),
    });
    r.settle();
    r.run(3);
    r.act(EngineAction::FadeOutAndStop {
        player: P,
        fade_ms: 2_000,
    });
    r.run(2);
    let wanted = DeviceSettings {
        sample_rate: 44_100,
        ..settings_of(&r, "main")
    };
    apply_device(&mut r, "main", wanted, true);
    r.tick();
    assert!(r.events.contains(&EngineEvent::ReachedEnd {
        player: P,
        entry: EntryId(1)
    }));
}
```

Append to `crates/fp-engine/tests/conductor.rs`:

```rust
#[test]
fn a_rate_change_waits_for_the_playing_player_and_applies_after_its_stop() {
    let (mut conductor, handle, device, mut now) = offline_conductor(model(1, 3));
    let p = conductor.state().players[0].id;
    conductor.tick(now);
    assert!(handle.send(Command::Play(p)));
    settle(&mut conductor, now);
    let mut config = conductor.state().config.clone();
    config.outputs.sample_rate = 44_100;
    assert!(handle.send(Command::UpdateConfig(Box::new(config))));
    for _ in 0..5 {
        conductor.tick(now);
        let _ = device.render(BLOCK);
        now += Duration::from_millis(10);
    }
    assert_eq!(device.config().unwrap().sample_rate, 48_000, "P1 is on air");
    assert!(!fp_model::pending(conductor.state()).is_empty());
    assert!(handle.send(Command::Stop(p)));
    for _ in 0..10 {
        conductor.tick(now);
        let _ = device.render(BLOCK);
        now += Duration::from_millis(10);
    }
    assert_eq!(device.config().unwrap().sample_rate, 44_100);
    assert!(fp_model::pending(conductor.state()).is_empty());
}
```

- [ ] **Step 2: Run the tests**

Run: `cargo test -p fp-engine --test live_settings l21_ && cargo test -p fp-engine --test conductor a_rate_change_waits`
Expected: PASS. If one fails, use superpowers:systematic-debugging, fix the engine code it points to (not the test's expectation, which is the spec's), and run again.

- [ ] **Step 3: Write the technical docs**

`docs/technical/audio-engine.md`, append to the "## Live settings" section (written in plan 2):

```markdown
### Applying a change

The model sends `ApplyAudioSystem`, `ApplyRoute` and `ApplyDevice` when
what they affect is idle in its view, and forced ones for **Apply now**.
The engine keeps each in `Engine::kept` (one per `Target`; a newer one
replaces it) and runs it, at once or on a later tick, when what it touches
is quiet (L11): for a device, its bus; for a route, the holder's own
sources; for the audio system, every bus. Quiet means no started, unpaused
source (current, outgoing, CUE), no cart, no test tone and no DSD silence
running; paused and waiting sources are quiet. A forced action runs at once.
Each one ends with `EngineEvent::Applied { target, wanted, outcome }`.

- `ApplyDevice` (L12): one `Bus::apply_config` on the open bus with the
  new rate (kept on a bit-perfect device, where the next file sets it),
  buffer and exclusive access; a bus carrying DSD goes back to PCM first.
  The DSD mix and silence are only stored. A new rate, or DSD left, opens
  every source on the bus again at its position (`reopen_on_bus`, also used
  when the watchdog had to change rate): playing sources resume when ready,
  paused and waiting ones stay so, fades and test tones are cut and end as
  they would have. A refusal reopens the running configuration and reports
  `Err` (L14); a forced change first forgets the device's remembered
  refusals. A device that is not there keeps the new configuration and the
  watchdog opens it when it returns (L15). `pcm_fallback` is set again as
  for a new bus.
- `ApplyRoute` (L16): the holder's sources are opened again on the new bus
  at their position (`reopen_player_sources`, `reopen_carts`); a device
  open with too few channels for the new pair is reopened with more when it
  is quiet; the cartwall before its first cart only takes the route; a Cue
  route removed while that CUE is open (forced) ends it with `CueEnded` or
  `CartCueEnded`. Reports `Placed` or `Unplaced`.
- `ApplyAudioSystem` (L17): see `backends.md`.

A bus a holder left is closed once nothing uses it, it is quiet and the
mixer has handed every slot back (`close_orphans`, L13), so its device is
released.
```

  In the action table under "## Engine", add these rows after `LoadPaused`:

```markdown
| `UpdateSettings` | take a new configuration for new buses and holders, and the tuning (see "Live settings") |
| `ApplyDevice`, `ApplyRoute`, `ApplyAudioSystem` | apply an output change when what it touches is quiet, or at once when forced (see "Live settings") |
```

`docs/technical/architecture.md`, after item 4 of the numbered list (~L57), add:

```markdown
5. Output settings apply while running (live settings spec): the engine
   reports where each holder plays and what each device runs with
   (`Placed`, `AudioSystemInUse`); `fp_model::live::pending` compares that
   with the configuration; the reducer sends each change once nothing it
   affects is playing, and the engine applies it once its buses are quiet
   and answers with `Applied`.
```

- [ ] **Step 4: Run the gate**

Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
Expected: all green.

- [ ] **Step 5: Commit**

```bash
git add crates/fp-engine/tests docs/technical/audio-engine.md docs/technical/architecture.md
git commit -m "$(cat <<'EOF'
test(engine): forced output changes and the live settings flow end to end

A forced change resumes what played and starts nothing paused (live
settings spec L21); a rate change waits for the player on air and
applies after its stop, through the model.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
EOF
)"
```

---

## Self-Review (done while writing)

- **Spec coverage:** L11 (Task 1), L12 every row (Tasks 1, 2), L13 closing (Task 4), L14 (Task 3), L15 (Task 3), L16 incl. cartwall and forced CUE removal (Task 4), L17 (Task 5), L21 (Task 6). §8 rows: rate on bit-perfect (Task 1), bit-perfect on with paused track (Task 1 `bit_perfect_on…`, paused case via `reopen_on_bus`), bit-perfect off with paused DSD (Task 2), DSD mode with DSD playing forced (Task 2 path + Task 6 forced semantics), mix/silence (Task 1).
- **Placeholders:** the `reopen_player_sources` move (Task 1, Step 3.4) describes a verbatim move of existing code with the exact three substitutions; everything new is written out.
- **Type consistency:** `kept`, `orphans`, `run_kept`, `apply_device`, `reopen_device(key, wanted, quiet, force)`, `apply_route`, `move_player_main`, `move_cartwall`, `apply_audio_system`, `cartwall_main_route` match across tasks and with plan 2's `running`, `device_of`, `report_placement`, `backend_in_use`, `choose_backend`.
