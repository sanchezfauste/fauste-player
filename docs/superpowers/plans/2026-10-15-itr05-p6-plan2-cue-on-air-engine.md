# Pre-listen on an On-air Output, Plan 2: the Engine

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The engine places a Cue holder on the Main output like any other holder and reports it (CA8), so the model's on-air rules of plan 1 see it; and a pre-listen on an on-air output of a bit-perfect device plays resampled at the running rate and never moves it (CA7).

**Architecture:** `Engine::cue_target` stops refusing a Cue target equal to the resolved Main (it still refuses a route to a backend this machine does not have); every caller drops the Main argument. `start_cue` and `start_cart_cue` skip `prepare_start` (the B3.2 reopen of an idle bit-perfect bus at the file's rate) when the pre-listen's (bus, first channel) overlaps a player's or the cartwall's Main. Nothing changes on the device callback: a Cue source on the Main bus is an ordinary mixer slot with its own `first_channel` (it always was, for a Cue on another pair of the Main device), and CA5's stop is the model's `StopCue` / `StopCartCue`, which the engine already executes in order before the `StartCurrent` that follows it.

**Tech Stack:** Rust 2024, `fp-engine`, `fp-backends` (`OfflineBackend` in tests).

**Spec:** `docs/superpowers/specs/2026-10-15-cue-on-air-output-design.md` (§4 CA8, CA7, §7 `fp-engine`). Plan 1 (`2026-10-15-itr05-p6-plan1-cue-on-air-model.md`) must be merged on the branch first.

This is plan 2 of 3, executed strictly in order on branch `feat/cue-on-air-itr05`.

## Rulings (planning)

**Ruling: the "is this output sounding" signal and the auto-stop are not engine work.** The spec defines *sounds* on the model's state (§2: `live::causes`, a paused player) and makes CA5's stop "an ordinary plan step; nothing new runs on the device callback" (§4). Plan 1 implements both in `fp_model::cue_on_air`; this plan only checks that the engine executes the stop and the start in one batch on a shared bus. — Cost if wrong: none for the rules; an engine-side signal would duplicate them.

**Ruling: CA7 on the engine side is "a pre-listen that overlaps a Main never reopens its bus".** `prepare_start` is skipped for such a pre-listen (players and carts). The check uses the engine's own (bus, first channel) of each Main (`PlayerRuntime::main`, `CartwallRuntime::main`) with §2's overlap (same bus, pairs less than two channels apart). A pre-listen on another pair of a bit-perfect Main device, or on a bit-perfect device of its own, still follows its file's rate while the bus is idle, as today. — Why: a reopen moves the whole device, and the next track on air would then start under a CUE tail at the CUE's rate. — Cost if wrong: one condition.

**Ruling: a track that starts while CA5's stopped pre-listen still ramps down on a bit-perfect device plays at the running rate for that pass.** The CUE's declick tail (`cue_outgoing`) sounds on the bus, so B3.2 does not reopen under it; the track plays resampled until the next hard transition (BR3 then moves the rate). — Why: the spec adds nothing new to the callback, and reopening under a tail clicks on air. — Cost if wrong: one pass resampled; reported to the maintainer.

## Global Constraints

- All code, identifiers, comments, docs and commit messages in English; never mention other playout or radio-automation products.
- CLAUDE.md rule 5 (real-time safety): nothing here runs on the device callback; no allocation, lock, log or panic is added to the mixer or the render path.
- `fp-engine` denies `clippy::indexing_slicing`: use `get`.
- CA8 verbatim: "The engine no longer drops a Cue route that resolves to the Main output (the default-output case of §4.6). It places the Cue holder on its device like any other holder and reports the placement in `live`."
- CA7 verbatim: "An on-air CUE on a bit-perfect device is a source like any other: it is resampled to the running rate and never moves the rate."
- CA1 (kept): "a missing Cue route, or one to a backend this machine does not have, still means no CUE".
- Engine tests use the `Offline` backend and drive time explicitly (`Rig::run`, `Rig::settle`); never sleep to wait for audio (the 1 ms sleeps in `settle`/`run` only give the decode workers CPU).
- Conventional Commits, scope `engine`; every commit ends with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- `cargo` is at `$HOME/.cargo/bin`. Gate for every task: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`.
- Progress ledger: `.superpowers/sdd/itr05/progress.md` (section P6, plan 2).

## Review Focus

1. **A Main route moved onto the Cue's pair while a pre-listen plays** (live settings L16): the Cue no longer changes with Main, so the move must apply as soon as Main's own sources are quiet and must not cut the pre-listen. Test: Task 1, `l16_a_main_moved_onto_the_cue_pair_keeps_the_pre_listen`.
2. **CA5 in the engine**: `StopCue` and `StartCurrent` in one batch on a bit-perfect on-air bus — the track must start and play at real-time speed, whatever rate the bus keeps. Test: Task 2, `ca5_a_track_started_as_its_pre_listen_stops_plays`.
3. **A pre-listen alone on a bit-perfect device** must still follow its file's rate (no regression of B3.2 for the Cue). Test: Task 2, `a_cue_alone_on_a_bit_perfect_device_still_follows_its_file`.
4. **A cart pre-listen on the cartwall's Main pair**, before and after the cartwall's first cart: it plays there, and is reported placed. Test: Task 1, `ca8_a_cart_cue_on_the_main_pair_plays_there`.
5. **A Cue route to a missing backend** next to a Main on the default output: still no CUE (`CueEnded` at once). Covered by the existing `a_cue_route_to_a_missing_backend_never_plays_on_main` (`tests/cartwall.rs`); Task 1 keeps it green.

---

## File Structure

| File | Responsibility |
|---|---|
| `crates/fp-engine/src/engine.rs` (`move_player_main` L1156-1205, `move_player_cue` L1210-1227, `cue_target` L2007-2027, `resolve_routes` L2030-2040, `start_cue` L3062-3093) | CA8, CA7 for players, `cue_on_air` |
| `crates/fp-engine/src/engine/carts.rs` (`ensure_cartwall` L83-86, `move_cartwall` L344 and L372, `start_cart_cue` L472-498) | CA8, CA7 for carts |
| `crates/fp-engine/tests/live_settings.rs` (L1055-1135) | CA8 placement; the four L16 tests whose Cue used to drop |
| `crates/fp-engine/tests/cartwall.rs` (L299-309) | CA8 for the cart pre-listen |
| `crates/fp-engine/tests/bit_perfect.rs` (`rig` L39-85) | CA7, CA5 sequence |
| `docs/technical/audio-engine.md` (L536, L603-606), `docs/technical/backends.md` (L234-236) | Task 3 |

---

### Task 1: A Cue on the Main output is placed there (CA8)

**Files:**
- Modify: `crates/fp-engine/src/engine.rs`, `crates/fp-engine/src/engine/carts.rs`
- Test: `crates/fp-engine/tests/live_settings.rs`, `crates/fp-engine/tests/cartwall.rs`

**Interfaces:**
- Produces: `fn cue_target(&self, route: &Route) -> Option<(BusKey, u16)>` (private; the `main` argument is gone). `EngineEvent::Placed { holder: Holder::PlayerCue(_) | Holder::CartwallCue, .. }` is now reported for a Cue on the Main output.

- [ ] **Step 1: Placement tests**

Append to `crates/fp-engine/tests/live_settings.rs`:

```rust
/// CA8: a Cue route equal to Main is placed on the Main device and reported.
#[test]
fn ca8_a_cue_route_equal_to_main_is_placed_there() {
    let mut c = config();
    c.outputs.routes[0].cue = Some(route("main"));
    let mut r = rig_with(&c);
    r.tick();
    let events = r.take_events();
    assert!(events.contains(&EngineEvent::Placed {
        holder: Holder::PlayerCue(P),
        route: Some(route("main")),
        device: out("main"),
        running: device_settings(&c.outputs, &out("main")),
    }));
}

/// CA8, the default-output case of §4.6: a missing Main route resolves to
/// the device the Cue route names; both are placed there.
#[test]
fn ca8_a_cue_on_the_default_output_main_resolves_to_is_placed_there() {
    let mut c = config();
    c.outputs.routes[0].main = None;
    c.outputs.routes[0].cue = Some(route("main"));
    let mut r = rig_with(&c);
    r.tick();
    let events = r.take_events();
    let placed_on_main = |holder: Holder| {
        events.iter().any(|e| {
            matches!(e, EngineEvent::Placed { holder: h, device, .. }
                if *h == holder && *device == out("main"))
        })
    };
    assert!(placed_on_main(Holder::PlayerMain(P)), "{events:?}");
    assert!(placed_on_main(Holder::PlayerCue(P)), "{events:?}");
}

/// CA8: the pre-listen on the Main pair is heard there.
#[test]
fn ca8_a_pre_listen_on_the_main_pair_is_heard_there() {
    let mut c = config();
    c.outputs.routes[0].cue = Some(route("main"));
    let mut r = rig_with(&c);
    r.act(EngineAction::StartCue {
        player: P,
        request: request(2, 0.0),
    });
    r.settle();
    let heard = r.run(5);
    assert!(!cue_ended(&r.events), "{:?}", r.events);
    assert!(heard.iter().any(|v| *v != 0.0), "nothing heard on main");
}
```

In `crates/fp-engine/tests/cartwall.rs`, replace `a_cue_route_equal_to_main_is_refused` (L299-309) with:

```rust
/// CA8 (on-air pre-listen spec): a cart pre-listen on the cartwall's Main
/// pair plays there; the model decides whether it may (CA3).
#[test]
fn ca8_a_cart_cue_on_the_main_pair_plays_there() {
    let mut r = rig(2, Some(route(0)), Some(route(0)), 48_000);
    r.act(EngineAction::StartCartCue(cart(1, 0.0, SOURCE_END, false)));
    r.settle();
    r.run(5);
    assert!(
        !r.events
            .contains(&EngineEvent::CartCueEnded { cart: CartId(1) }),
        "{:?}",
        r.events
    );
    assert!(audible(&r.channel(0)) > 0);
    assert!(r.events.iter().any(|e| matches!(
        e,
        EngineEvent::Placed {
            holder: fp_model::Holder::CartwallCue,
            ..
        }
    )));
}
```

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p fp-engine --test live_settings ca8 && cargo test -p fp-engine --test cartwall ca8`
Expected: FAIL — `Unplaced { PlayerCue }` instead of `Placed`, `CueEnded` / `CartCueEnded` at once.

- [ ] **Step 3: `cue_target` no longer refuses the Main target**

In `crates/fp-engine/src/engine.rs`, replace `cue_target` (L2007-2027) with:

```rust
    /// Where a Cue route really goes. Unlike Main, a Cue never falls back to
    /// the default output: a route to a backend this machine does not have
    /// means no cue. A Cue on the Main output is placed there like any
    /// holder (on-air pre-listen spec CA8); the model decides when it may
    /// sound (CA3–CA5).
    fn cue_target(&self, route: &Route) -> Option<(BusKey, u16)> {
        let Some(backend) = self.route_backend(&route.backend) else {
            tracing::warn!(backend = %route.backend, "cue route to an unavailable backend; no cue");
            return None;
        };
        Some((
            BusKey {
                backend: backend.id().0,
                device: route.device.clone(),
            },
            route.first_channel,
        ))
    }
```

Callers:
- `resolve_routes` (L2036-2038): `.and_then(|route| self.cue_target(route));`
- `move_player_main` (L1177): `let cue = cue_route.as_ref().and_then(|r| self.cue_target(r));`, and its doc comment (L1156-1159) loses "A Cue route equal to the new Main is dropped; one that differed again comes back." — replace with "Its Cue stays where its route goes (CA8)."
- `move_player_cue` (L1210-1227):

```rust
    /// L16: `player`'s Cue goes to `route` (no Cue without one, CA1).
    fn move_player_cue(
        &mut self,
        player: PlayerId,
        route: Option<Route>,
        force: bool,
    ) -> Option<Result<(), String>> {
        if !self.players.contains_key(&player) {
            return Some(Ok(()));
        }
        let cue = route.as_ref().and_then(|r| self.cue_target(r));
        let outcome = match &cue {
            Some(target) => self.make_room(target, force)?,
            None => Ok(()),
        };
        self.place_player_cue(player, cue, route);
        Some(outcome)
    }
```

In `crates/fp-engine/src/engine/carts.rs`:
- `ensure_cartwall` (L83-86): `.and_then(|route| self.cue_target(route));`
- `move_cartwall`, the `CartwallMain` arm (L344): `let cue = cue_route.as_ref().and_then(|r| self.cue_target(r));`
- the `CartwallCue` arm (L372): `let cue = route.as_ref().and_then(|r| self.cue_target(r));` — `main` is still used by the `CartwallMain` arm (`main != target`), so the tuple binding stays.

- [ ] **Step 4: Run the new tests**

Run: `cargo test -p fp-engine --test live_settings ca8 && cargo test -p fp-engine --test cartwall ca8`
Expected: PASS.

- [ ] **Step 5: The L16 tests whose Cue used to drop**

Run: `cargo test -p fp-engine --test live_settings l16`
Expected: FAIL in `l16_a_main_moved_onto_the_cue_pair_waits_for_the_pre_listen`, `l16_a_forced_main_move_onto_the_cue_pair_applies_at_once`, `l16_a_cartwall_main_moved_onto_the_cue_pair_waits_for_the_pre_listen`, `l16_a_forced_cartwall_main_move_onto_the_cue_pair_applies_at_once` (they assumed the Cue drops when Main lands on its pair).

Replace those four tests (L1055-1124) with:

```rust
/// L11, L16 with CA8: a Main moved onto the Cue's pair leaves the Cue where
/// it is, so the move waits only for Main's own sources (quiet here) and
/// the pre-listen goes on.
#[test]
fn l16_a_main_moved_onto_the_cue_pair_keeps_the_pre_listen() {
    for force in [false, true] {
        let mut r = pfl_rig();
        r.act(EngineAction::ApplyRoute {
            holder: Holder::PlayerMain(P),
            route: Some(route("other")),
            force,
        });
        r.run(3);
        assert!(route_applied(&r.events, Holder::PlayerMain(P)), "force {force}");
        assert!(!cue_ended(&r.events), "force {force}: the pre-listen is not cut");
    }
}

#[test]
fn l16_a_cartwall_main_moved_onto_the_cue_pair_keeps_the_pre_listen() {
    for force in [false, true] {
        let mut c = config();
        c.outputs.cartwall.cue = Some(route("other"));
        let mut r = rig_with(&c);
        r.act(EngineAction::StartCartCue(cart_request(1)));
        r.settle();
        r.run(2);
        r.take_events();
        r.act(EngineAction::ApplyRoute {
            holder: Holder::CartwallMain,
            route: Some(route("other")),
            force,
        });
        r.run(3);
        assert!(route_applied(&r.events, Holder::CartwallMain), "force {force}");
        assert!(!cue_ended(&r.events), "force {force}: the pre-listen is not cut");
    }
}
```

- [ ] **Step 6: Run the engine suite**

Run: `cargo test -p fp-engine`
Expected: PASS. A test that still expects `CueEnded`/`CartCueEnded` or `Unplaced` for a Cue equal to Main asserts the old §4.6; change it to CA8's placement and note it in the ledger.

- [ ] **Step 7: Gate and commit**

Run the gate. Then:

```bash
git add crates/fp-engine
git commit -m "feat(engine): place a Cue route on the Main output like any holder

On-air pre-listen spec CA8: a Cue route that resolves to the Main
output (equal routes, or a missing Main route on the default output) is
placed there and reported, instead of being dropped, so the model sees
the on-air pre-listen and decides when it may sound. A route to a
missing backend still means no cue.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: An on-air pre-listen never moves a bit-perfect rate (CA7)

**Files:**
- Modify: `crates/fp-engine/src/engine.rs` (`start_cue` L3062-3093, a new `cue_on_air` next to `bus_sounding` L1446)
- Modify: `crates/fp-engine/src/engine/carts.rs` (`start_cart_cue` L472-498)
- Test: `crates/fp-engine/tests/bit_perfect.rs`

**Interfaces:**
- Consumes: Task 1 (a Cue on the Main bus is placed).
- Produces: `fn cue_on_air(&self, cue: &(BusKey, u16)) -> bool` (private; `pub(super)` so `carts.rs` can call it).

- [ ] **Step 1: A rig with routes**

In `crates/fp-engine/tests/bit_perfect.rs`, turn `rig` into a wrapper:

```rust
fn dac_route(first_channel: u16) -> Route {
    Route {
        backend: "offline".into(),
        device: "dac".into(),
        first_channel,
    }
}

fn rig(bit_perfect: bool, exclusive_capable: bool) -> Rig {
    rig_routed(bit_perfect, exclusive_capable, dac_route(0), None)
}

/// As `rig`, with P's Main on `main` and its Cue on `cue`.
fn rig_routed(
    bit_perfect: bool,
    exclusive_capable: bool,
    main: Route,
    cue: Option<Route>,
) -> Rig {
```

and in its body replace the `config.outputs.routes = vec![PlayerRoutes { … }];` literal with:

```rust
    config.outputs.routes = vec![PlayerRoutes {
        player: P,
        main: Some(main),
        cue,
    }];
```

(the rest of the old `rig` body is unchanged). Add a helper next to `Rig::run`:

```rust
    fn cue(&mut self, request: SourceRequest) {
        self.act(EngineAction::StartCue { player: P, request });
        self.settle();
    }
```

- [ ] **Step 2: Tests**

Append to `crates/fp-engine/tests/bit_perfect.rs`:

```rust
/// CA7: a pre-listen on the on-air pair of an idle bit-perfect device is
/// resampled to the running rate; it never moves the rate.
#[test]
fn ca7_an_on_air_cue_plays_at_the_running_rate() {
    let mut r = rig_routed(true, true, dac_route(0), Some(dac_route(0)));
    assert_eq!(r.rate(), 48_000);
    let path = wav(r.dir.path(), "cue.wav", 44_100);
    r.cue(request(1, path, pcm16(44_100)));
    assert_eq!(r.rate(), 48_000, "the pre-listen never moves the rate");
    let heard = r.run(4_800);
    assert!(heard.iter().any(|v| *v != 0.0), "it is heard, resampled");
}

/// Review Focus 3: alone on a bit-perfect device (Main elsewhere), a
/// pre-listen still follows its file's rate (B3.2).
#[test]
fn a_cue_alone_on_a_bit_perfect_device_still_follows_its_file() {
    let speakers = Route {
        backend: "offline".into(),
        device: "speakers".into(),
        first_channel: 0,
    };
    let mut r = rig_routed(true, true, speakers, Some(dac_route(0)));
    let path = wav(r.dir.path(), "cue.wav", 44_100);
    r.cue(request(1, path, pcm16(44_100)));
    assert_eq!(r.rate(), 44_100);
}

/// Review Focus 2, CA5 in the engine: the model sends the pre-listen's
/// stop and the track's start in one batch; the track starts and plays at
/// real-time speed, whatever rate the bus keeps under the CUE's tail.
#[test]
fn ca5_a_track_started_as_its_pre_listen_stops_plays() {
    let mut r = rig_routed(true, true, dac_route(0), Some(dac_route(0)));
    let cue = wav(r.dir.path(), "cue.wav", 48_000);
    r.cue(request(1, cue, pcm16(48_000)));
    r.run(4_800);
    let track = wav(r.dir.path(), "track.wav", 44_100);
    r.act(EngineAction::StopCue { player: P });
    r.start(request(2, track, pcm16(44_100)));
    let rate = r.rate();
    let heard = r.run(rate as usize);
    assert!(heard.iter().any(|v| *v != 0.0), "the track is heard");
    let position = r.engine.telemetry(P).position_secs.unwrap();
    assert!(
        (position - 1.0).abs() < 0.05,
        "one second plays as one second at {rate} Hz: {position}"
    );
}
```

- [ ] **Step 3: Run them to see the CA7 one fail**

Run: `cargo test -p fp-engine --test bit_perfect ca7 a_cue_alone ca5_a_track`
Expected: `ca7_an_on_air_cue_plays_at_the_running_rate` FAILS (the rate becomes 44 100); the other two pass already (they pin behaviour this task must keep).

- [ ] **Step 4: The check and its use**

In `crates/fp-engine/src/engine.rs`, after `bus_sounding` (L1446-1448), add:

```rust
    /// On-air pre-listen spec §2, CA7: a pre-listen on `cue` (bus, first
    /// channel) shares a channel with a player's or the cartwall's Main.
    /// Such a pre-listen never reopens its bus at its file's rate.
    pub(super) fn cue_on_air(&self, cue: &(BusKey, u16)) -> bool {
        let overlaps = |main: &(BusKey, u16)| main.0 == cue.0 && main.1.abs_diff(cue.1) < 2;
        self.players.values().any(|rt| overlaps(&rt.main))
            || self.cartwall.as_ref().is_some_and(|c| overlaps(&c.main))
    }
```

In `start_cue` (L3073-3080) replace the `prepare_start` block with:

```rust
        // B3.2 follows the file's rate on an idle bit-perfect bus, but an
        // on-air pre-listen never moves the rate (CA7).
        if let Some(cue) = self
            .players
            .get(&player)
            .and_then(|rt| rt.cue.clone())
            .filter(|cue| !self.cue_on_air(cue))
        {
            self.prepare_start(&cue.0, request.format);
        }
```

In `crates/fp-engine/src/engine/carts.rs`, `start_cart_cue` (L479-486):

```rust
        if let Some(cue) = self
            .cartwall
            .as_ref()
            .and_then(|c| c.cue.clone())
            .filter(|cue| !self.cue_on_air(cue))
        {
            self.prepare_start(&cue.0, request.format);
        }
```

(`self.cartwall.as_ref()` borrows immutably inside the `filter` closure as well; both borrows are shared, and the `if let` binding owns its clone before `prepare_start` takes `&mut self`.)

- [ ] **Step 5: Run the tests**

Run: `cargo test -p fp-engine --test bit_perfect && cargo test -p fp-engine`
Expected: PASS.

- [ ] **Step 6: Gate and commit**

Run the gate. Then:

```bash
git add crates/fp-engine
git commit -m "feat(engine): an on-air pre-listen never moves a bit-perfect rate

On-air pre-listen spec CA7: a pre-listen whose output shares a channel
with a player's or the cartwall's Main is a source like any other on a
bit-perfect device: it plays resampled at the running rate and never
reopens the device at its file's rate. A pre-listen alone on a
bit-perfect device still follows its file (B3.2).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: Technical docs

**Files:**
- Modify: `docs/technical/audio-engine.md` (L536, L603-606), `docs/technical/backends.md` (L234-236)

- [ ] **Step 1: `audio-engine.md`**

Replace the `StartCue`, `StopCue` row (L536) with:

```markdown
| `StartCue`, `StopCue` | a separate source on the player's Cue route. The model sends `StartCue` only for a player with a Cue route (`OutputsConfig::player_has_cue`); a Cue route on the Main output is placed there like any holder (on-air pre-listen spec CA8), and the model decides when such a pre-listen may sound (CA3–CA5). A Cue route to a backend this machine does not have (`cue_target`) ends the CUE at once with `CueEnded`. A pre-listen that shares a channel with a Main (`Engine::cue_on_air`) never reopens a bit-perfect bus at its file's rate (CA7) |
```

and the cartwall routes paragraph (L603-606) with:

```markdown
  Main falls back to the default output. The model starts no cart
  pre-listen without a Cue route (`OutputsConfig::cartwall_has_cue`); a Cue
  route on the Main output plays there (CA8), and one the engine cannot
  route (a missing backend) ends at once (`CartCueEnded { cart }`; the
  model ignores the end of a pre-listen that is no longer the current one).
```

- [ ] **Step 2: `backends.md` L234-236**

The sentence "A Main route to one plays on the default output, and a Cue route to one means no cue, as for a backend this build does not have." stays; append: "A Cue route on the Main output is placed there (on-air pre-listen spec CA8)."

- [ ] **Step 3: Gate and commit**

Run the gate. Then:

```bash
git add docs/technical/audio-engine.md docs/technical/backends.md
git commit -m "docs(engine): a Cue on the Main output and the on-air pre-listen rate

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

## Self-review (against the spec)

- CA8: Task 1 (equal routes, the default-output case, audible on Main, cart pre-listen, L16 moves).
- CA7 (engine side): Task 2; the model side is plan 1 Task 5.
- §4 "Stopping a CUE under CA5 is an ordinary plan step": Task 2's `ca5_a_track_started_as_its_pre_listen_stops_plays`; no callback code changes.
- CA1's "a backend this machine does not have still means no CUE": `cue_target` keeps that refusal; existing `a_cue_route_to_a_missing_backend_never_plays_on_main`.
