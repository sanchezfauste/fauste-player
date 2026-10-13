# Bit-perfect Hard Transitions, Plan 3: MIDI, the Remote API and OSC

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Control surfaces respect the locked fader of a bit-perfect player: a MIDI fader binding is ignored and picks the volume up again at 100 % once unlocked (BF10), the remote API and OSC refuse any level but 100 % with the model's reason (BF12), clients see the lock (`PlayerDto::fader_locked`, OSC `volume-locked`, BF13), and Play/Next and Previous keep working, starting hard (BR16).

**Architecture:** Everything follows from the model (plan 1). `fp_control::router::Router::on_message` asks `fp_model::fader_locked` before a `Volume` binding acts and resets that binding's soft takeover while locked. The remote API needs no new refusal path: `api::plan` already applies `SetVolume` to a copy of the snapshot (`dry_run`), and the model's `ModelError::FaderLocked` becomes **409 `unavailable`** with its message through `ApiError::from_model`; OSC goes through the same `api::plan` and notes the code in the remote log. `PlayerDto` gains `fader_locked`, and `osc::values` publishes it.

**Tech Stack:** Rust 2024, `fp-control` (midir-free router logic), `fp-remote` (axum router driven with `tower::ServiceExt::oneshot`, `rosc`).

**Spec:** `docs/superpowers/specs/2026-10-08-bit-perfect-rate-and-fader-design.md` (§4.7 BR16, §5.3 BF10–BF11, §5.4 BF12–BF13, §7 `fp-control` and `fp-remote` rows, §9.1 `midi.md` and `remote-control.md`, §9.2 `remote-api.md`).

This is plan 3 of 4 (after plans 1 and 2; it needs `fp_model::fader_locked` and `ModelError::FaderLocked` from plan 1).

## Rulings (planning)

**Ruling: the router's function and the volume route are named as in the code.** The spec names `Router::handle` (~L110) and `POST /players/{id}/fader`; the code has `Router::on_message` (`crates/fp-control/src/router.rs` L58, the `Volume` arm at L112) and `PUT /players/{id}/volume` with `{"fader": f32}` (`crates/fp-remote/src/http/mod.rs` L123). An accepted action answers **202 Accepted**, not 200 (`tests/http.rs` `play_is_accepted_and_queued`). The plan follows the code; the docs keep the existing route. — Cost if wrong: none (names only).

**Ruling: while locked, the MIDI pickup keeps following the control.** BF10 says "the binding's soft takeover forgets its pickup (`picked = false`, `last_set = None`)". It also records the control's position (`last_input`) and the model's travel (`model_seen`, 100 %) while locked, so that after unlocking a control below the top never counts as "crossing" 100 % and the first unlocked message is not taken for a move of the model by other means. — Cost if wrong: one more message before pickup.

**Ruling: the OSC refusal is pinned by what is queued.** The remote log's throttle (`Throttle::note`) has no test hook; the OSC test checks that the volume message queues nothing while a following Play is queued, and the code path that notes the code (`Err(e) => log.note(source, e.code())`, `osc_server.rs` L91-93) is unchanged. — Cost if wrong: the log line is not asserted.

## Global Constraints

- All code, identifiers, comments, docs and commit messages in English; never mention other playout or radio-automation products.
- BF10: "the message gives no command when `fader_locked`, and the binding's soft takeover forgets its pickup … once the fader unlocks, the surface's control must pick up the volume at 100 % (by reaching the top or crossing it) before it moves anything."
- BF11: feedback covers buttons only; buttons are unchanged; no fader position is sent (§10 out of scope).
- BF12: "a value below 1 on a locked player is refused with **409 `unavailable`** and the model's message … A value of exactly 1 is accepted and changes nothing."
- BF13: "`PlayerDto` gains `fader_locked: bool`, and OSC publishes `/fauste/player/{n}/volume-locked` (int 0/1) beside `/volume` … Both are additive (no breaking change for clients)."
- BR16: Play/Next and Previous "are accepted, never refused"; "No 'crossfade' operation exists in either protocol."
- `unwrap`, `expect`, `panic` denied outside tests.
- Remote API tests drive the axum router with `tower::ServiceExt::oneshot` and the recording `FakeControl`; OSC tests use UDP sockets on `127.0.0.1:0`.
- Conventional Commits; every commit ends with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Gate for every task: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`.
- Progress ledger: `.superpowers/sdd/2026-10-13-bit-perfect-plan3-midi-remote/progress.md`.

## Review Focus

1. **A motorless MIDI fader left at 30 % while the player was locked**, then bit-perfect turned off: the first moves below the top must do nothing, so the level never jumps from 100 % to 30 %. Test: Task 1, `bf10_after_unlocking_a_control_below_the_top_waits_for_the_top`.
2. **A fader binding for a locked player that is already at the top** (127): ignored while locked, and picked up at once after unlocking. Test: Task 1, `bf10_a_control_at_the_top_picks_up_at_once_after_unlocking`.
3. **A remote surface restoring a saved level of exactly 1.0 on a locked player** (`PUT …/volume {"fader": 1.0}`): 202, queued, no error. Test: Task 2, `bf12_full_volume_is_accepted_over_http`.
4. **OSC `/volume 0.5` immediately followed by `/play`** from the same client on a locked player: the volume is refused and the play still goes through. Test: Task 2, `bf12_an_osc_volume_on_a_locked_player_is_refused_and_play_still_works`.
5. **A subscriber's dump when a player becomes locked**: `volume-locked` changes to 1 and is pushed as a change, with `/volume` at 1.0. Test: Task 2, `bf13_osc_values_follow_the_lock`.

---

## File Structure

| File | Responsibility |
|---|---|
| `crates/fp-control/src/router.rs` (`on_message` L58-151, `Volume` arm L112-149) | BF10 |
| `crates/fp-control/tests/router.rs`, `tests/feedback.rs` | BF10, BF11, BR16 |
| `crates/fp-remote/src/dto.rs` (`PlayerDto` L23-40, `player_dto` L257-282) | `fader_locked` |
| `crates/fp-remote/src/osc.rs` (`values` ~L237-262) | `/fauste/player/{n}/volume-locked` |
| `crates/fp-remote/tests/{api,dto,osc,osc_server,http}.rs`, `tests/support/mod.rs` | BF12, BF13, BR16 |
| `docs/user/midi.md`, `docs/user/remote-control.md`, `docs/technical/remote-api.md` | Task 3 |

---

### Task 1: A locked fader on a MIDI surface (BF10, BF11, BR16)

**Files:**
- Modify: `crates/fp-control/src/router.rs` (the `MidiAction::Volume(n)` arm, L112-149)
- Test: `crates/fp-control/tests/router.rs`, `crates/fp-control/tests/feedback.rs`

**Interfaces:**
- Consumes: `fp_model::fader_locked(&AppState, PlayerId) -> bool` (plan 1).

- [ ] **Step 1: Write the failing tests**

`crates/fp-control/tests/router.rs` (extend the `fp_model` imports with `EngineEvent, Holder, OutputDevice, device_settings, on_event`):

```rust
// Bit-perfect rate and fader spec BF10, BR16.

fn dac() -> OutputDevice {
    OutputDevice {
        backend: "null".into(),
        device: "dac".into(),
    }
}

/// P1's Main placed on `dac`, which runs bit-perfect: its fader is locked.
fn locked(mut s: AppState) -> AppState {
    let p = s.players[0].id;
    let route = fp_model::Route {
        backend: "null".into(),
        device: "dac".into(),
        first_channel: 0,
    };
    s.config
        .outputs
        .routes
        .iter_mut()
        .find(|r| r.player == p)
        .unwrap()
        .main = Some(route.clone());
    s.config.outputs.bit_perfect = vec![dac()];
    let running = device_settings(&s.config.outputs, &dac());
    on_event(
        &mut s,
        EngineEvent::Placed {
            holder: Holder::PlayerMain(p),
            route: Some(route),
            device: dac(),
            running,
        },
    );
    assert!(fp_model::fader_locked(&s, p));
    s
}

/// P1 back on a device that is not bit-perfect: unlocked at 100 % (BF4).
fn unlocked(mut s: AppState) -> AppState {
    let p = s.players[0].id;
    let speakers = OutputDevice {
        backend: "null".into(),
        device: "speakers".into(),
    };
    let running = device_settings(&s.config.outputs, &speakers);
    on_event(
        &mut s,
        EngineEvent::Placed {
            holder: Holder::PlayerMain(p),
            route: None,
            device: speakers,
            running,
        },
    );
    assert!(!fp_model::fader_locked(&s, p));
    s
}

#[test]
fn bf10_a_fader_binding_of_a_locked_player_gives_nothing() {
    let s = locked(at_travel(state(), travel_of(50)));
    let mut r = volume_router();
    for v in [0, 50, 100, 127] {
        assert_eq!(r.on_message("APC", cc_value(7, v), &s), None, "{v}");
    }
}

#[test]
fn bf10_after_unlocking_a_control_below_the_top_waits_for_the_top() {
    let s = locked(state());
    let mut r = volume_router();
    assert_eq!(r.on_message("APC", cc_value(7, 40), &s), None);
    let s = unlocked(s);
    for v in [40, 50, 64, 100, 126] {
        assert_eq!(r.on_message("APC", cc_value(7, v), &s), None, "{v}: not picked up");
    }
    let got = volume_of(r.on_message("APC", cc_value(7, 127), &s)).unwrap();
    assert!((got - 1.0).abs() < 1e-3, "picked up at the top");
    assert!(r.on_message("APC", cc_value(7, 100), &s).is_some(), "then it moves the volume");
}

#[test]
fn bf10_a_control_at_the_top_picks_up_at_once_after_unlocking() {
    let s = locked(state());
    let mut r = volume_router();
    assert_eq!(r.on_message("APC", cc_value(7, 127), &s), None);
    let s = unlocked(s);
    assert!(r.on_message("APC", cc_value(7, 127), &s).is_some());
}

#[test]
fn br16_play_and_previous_bindings_work_on_a_bit_perfect_player() {
    let s = locked(state());
    let p1 = s.players[0].id;
    let mut r = router(&[
        (note(1), MidiAction::Button(ShortcutAction::PlayPlayer(1))),
        (note(2), MidiAction::Button(ShortcutAction::PreviousPlayer(1))),
    ]);
    assert_eq!(r.on_message("APC", on(1), &s), Some(Command::Play(p1)), "accepted");
    // After a hard Next there is a history entry to go back to.
    let mut s = s;
    apply(&mut s, Command::Play(p1)).unwrap();
    assert!(!s.player(p1).unwrap().fading, "a hard start (BR7)");
    assert_eq!(r.on_message("APC", on(2), &s), Some(Command::Previous(p1)));
}
```

`crates/fp-control/tests/feedback.rs` (same imports; copy the `dac` and `locked` helpers above):

```rust
#[test]
fn bf11_a_locked_players_buttons_light_as_before() {
    let mut s = state();
    let p = s.players[0].id;
    apply(&mut s, Command::Play(p)).unwrap();
    let before = desired(&config(), &s, true);
    let s = locked(s);
    assert_eq!(desired(&config(), &s, true), before);
}
```

(`state()` in `feedback.rs` builds four players with Cue routes (L12-31); `config()` binds P1's buttons (L41-60).)

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p fp-control --test router bf10 2>&1 | tail -8`
Expected: FAIL — `bf10_a_fader_binding_of_a_locked_player_gives_nothing` gets `Some(SetVolume …)` for 127 (the control lands on 100 %). `br16_…` and `bf11_…` pass already: record in the ledger that they pin behaviour.

- [ ] **Step 3: Implement**

In the `MidiAction::Volume(n)` arm, right after `let pickup = self.pickups.entry(…).or_default();`:

```rust
                // BF10: a locked fader (bit-perfect output) moves nothing.
                // The pickup is forgotten, so once unlocked the control must
                // reach the volume (100 %) again before it moves anything;
                // its position and the model's travel are still followed.
                if fp_model::fader_locked(state, player.id) {
                    pickup.picked = false;
                    pickup.last_set = None;
                    pickup.last_input = Some(position);
                    pickup.model_seen = Some(travel);
                    return None;
                }
```

Update the doc of `on_message`: "…or a bound fader that has picked up the volume and whose player's fader is not locked (bit-perfect rate and fader spec BF10)."

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p fp-control`
Expected: PASS.

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-control
git commit -m "feat(control): ignore a MIDI fader bound to a locked player

A fader binding of a player whose output is bit-perfect gives no
command, and must pick the volume up again at 100 % once the player is
unlocked, so the level never jumps (bit-perfect rate and fader spec
BF10, BF11, BR16).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: The remote API and OSC (BF12, BF13, BR16)

**Files:**
- Modify: `crates/fp-remote/src/dto.rs` (`PlayerDto` after `fader` L31-32; `player_dto` after `fader:` ~L270)
- Modify: `crates/fp-remote/src/osc.rs` (`values`, after the `volume` push at L249)
- Modify: `crates/fp-remote/tests/support/mod.rs` (a `lock_first_player` helper)
- Test: `crates/fp-remote/tests/api.rs`, `tests/dto.rs`, `tests/osc.rs`, `tests/osc_server.rs`, `tests/http.rs`

**Interfaces:**
- Consumes: `fp_model::fader_locked`, `ModelError::FaderLocked` (plan 1).
- Produces: `PlayerDto::fader_locked: bool` (serialised as `"fader_locked"`); OSC address `/fauste/player/{n}/volume-locked` (`i` 0/1).

- [ ] **Step 1: The test helper**

`crates/fp-remote/tests/support/mod.rs`:

```rust
/// Places player 1's Main on a `dac` that runs bit-perfect (live settings
/// L3), which locks its fader (bit-perfect rate and fader spec BF1).
pub fn lock_first_player(s: &mut AppState) {
    let p = s.players[0].id;
    let dac = fp_model::OutputDevice {
        backend: "null".into(),
        device: "dac".into(),
    };
    let route = fp_model::Route {
        backend: "null".into(),
        device: "dac".into(),
        first_channel: 0,
    };
    if let Some(r) = s.config.outputs.routes.iter_mut().find(|r| r.player == p) {
        r.main = Some(route.clone());
    }
    s.config.outputs.bit_perfect = vec![dac.clone()];
    let running = fp_model::device_settings(&s.config.outputs, &dac);
    fp_model::on_event(
        s,
        fp_model::EngineEvent::Placed {
            holder: fp_model::Holder::PlayerMain(p),
            route: Some(route),
            device: dac,
            running,
        },
    );
}
```

- [ ] **Step 2: Write the failing tests**

`tests/api.rs` (import `support::lock_first_player`, `fp_model::ModelError`):

```rust
#[test]
fn bf12_a_fader_below_full_on_a_locked_player_is_a_conflict_with_the_reason() {
    let mut s = demo_state();
    let p = s.players[0].id;
    lock_first_player(&mut s);
    let e = plan(&s, O::SetFader(p, 0.5)).unwrap_err();
    assert_eq!(e.status(), 409);
    assert_eq!(e.code(), "unavailable");
    assert_eq!(e.message(), ModelError::FaderLocked(p).to_string());
    assert_eq!(
        plan(&s, O::SetFader(p, 1.0)).unwrap(),
        vec![Command::SetVolume(p, gain_from_fader(1.0))],
        "exactly 100 % is accepted"
    );
}

#[test]
fn br16_play_and_previous_on_a_bit_perfect_player_are_accepted() {
    let mut s = demo_state();
    let p = s.players[0].id;
    lock_first_player(&mut s);
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(plan(&s, O::Play(p)).unwrap(), vec![Command::Play(p)]);
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(plan(&s, O::Previous(p)).unwrap(), vec![Command::Previous(p)]);
}
```

`tests/dto.rs`:

```rust
#[test]
fn bf13_a_player_says_whether_its_fader_is_locked() {
    let mut s = demo_state();
    let p = s.players[0].id;
    let playback = Playback::default();
    assert!(!dto::player(&s, &playback, p).unwrap().fader_locked);
    support::lock_first_player(&mut s);
    let out = dto::player(&s, &playback, p).unwrap();
    assert!(out.fader_locked);
    assert_eq!(out.fader, 1.0);
    let json = serde_json::to_value(&out).unwrap();
    assert_eq!(json["fader_locked"], serde_json::Value::Bool(true));
}
```

(Add `serde_json` to the test's imports if `tests/dto.rs` does not use it yet; it is a dev-dependency of `fp-remote`, `tests/http.rs` uses it.)

`tests/osc.rs`:

```rust
#[test]
fn bf13_osc_values_follow_the_lock() {
    let mut s = demo_state();
    let get = |s: &fp_model::AppState, a: &str| {
        values(s, &Playback::default())
            .into_iter()
            .find(|(addr, _)| addr == a)
            .map(|(_, v)| v)
            .unwrap()
    };
    assert_eq!(get(&s, "/fauste/player/1/volume-locked"), OscType::Int(0));
    support::lock_first_player(&mut s);
    assert_eq!(get(&s, "/fauste/player/1/volume-locked"), OscType::Int(1));
    assert_eq!(get(&s, "/fauste/player/1/volume"), OscType::Float(1.0));
    assert_eq!(get(&s, "/fauste/player/2/volume-locked"), OscType::Int(0));
}
```

`tests/osc_server.rs`:

```rust
#[test]
fn bf12_an_osc_volume_on_a_locked_player_is_refused_and_play_still_works() {
    let mut s = demo_state();
    support::lock_first_player(&mut s);
    let fake = FakeControl::new(s);
    enabled(&fake);
    let handle = spawn(fake.clone()).unwrap();
    let to = osc_addr(&handle);
    let sock = UdpSocket::bind("127.0.0.1:0").unwrap();
    send(&sock, to, "/fauste/player/1/volume", vec![OscType::Float(0.5)]);
    send(&sock, to, "/fauste/player/1/play", vec![]);
    let p = fake.state.load().players[0].id;
    assert_eq!(wait_sent(&fake, 1), vec![Command::Play(p)], "the volume queued nothing");
}
```

`tests/http.rs`:

```rust
#[tokio::test]
async fn bf12_full_volume_is_accepted_over_http_and_less_is_a_conflict() {
    let mut s = demo_state();
    support::lock_first_player(&mut s);
    let p = s.players[0].id;
    let fake = FakeControl::new(s);
    let uri = format!("/api/v1/players/{}/volume", p.0);
    let (status, out, _) = call(ctx(&fake), "PUT", &uri, Some(json!({"fader": 0.5}))).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(out.to_string().contains("held at 100 %"), "{out}");
    assert!(fake.take_sent().is_empty());
    let (status, _, _) = call(ctx(&fake), "PUT", &uri, Some(json!({"fader": 1.0}))).await;
    assert_eq!(status, StatusCode::ACCEPTED);
}
```

(The Review Focus calls this `bf12_full_volume_is_accepted_over_http`; one test covers both halves.)

- [ ] **Step 3: Run to verify it fails**

Run: `cargo test -p fp-remote 2>&1 | tail -8`
Expected: FAIL to compile ("no field `fader_locked`"). `bf12_…` in `api.rs`, `osc_server.rs` and `http.rs` would pass already once compiling (the model refuses through `dry_run`); record that in the ledger.

- [ ] **Step 4: Implement**

`dto.rs`, `PlayerDto` after `fader`:

```rust
    /// The fader is held at 100 % because the player's output is
    /// bit-perfect; such a player also starts every track without a
    /// crossfade (bit-perfect rate and fader spec BF13, BR16).
    pub fader_locked: bool,
```

`player_dto`: `fader_locked: fp_model::fader_locked(model, p.id),` after `fader:`.

`osc.rs`, `values`, after the `volume` push:

```rust
        out.push((at("volume-locked"), flag(p.fader_locked)));
```

- [ ] **Step 5: Run to verify it passes**

Run: `cargo test -p fp-remote`
Expected: PASS (the SSE and event tests that compare whole player objects gain the field on both sides; fix any literal `PlayerDto { … }` in tests with `fader_locked: false`).

- [ ] **Step 6: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-remote
git commit -m "feat(remote): show and enforce the locked fader of bit-perfect players

Players report fader_locked and OSC publishes volume-locked; a fader
below 100 % on such a player is refused with 409 and the model's reason,
over HTTP and OSC, while exactly 100 %, Play and Previous are accepted
(bit-perfect rate and fader spec BF12, BF13, BR16).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: Documentation (§9.1 `midi.md`, `remote-control.md`; §9.2 `remote-api.md`)

**Files:**
- Modify: `docs/user/midi.md`, "How the controls behave": a bullet "A volume binding of a player whose Main output is bit-perfect does nothing: its volume is held at 100 % (see [What it costs](bit-perfect.md#what-it-costs)). Once the output is no longer bit-perfect, move the control to the top to pick the volume up again." and one "Play/Next and Previous on such a player switch at once, without a crossfade."
- Modify: `docs/user/remote-control.md`, "What a client can do": `play` and `previous` on a player whose Main output is bit-perfect switch at once, without a crossfade; a volume below 100 % is refused (409, with the reason) and exactly 100 % is accepted; `fader_locked` says which players are locked. "OSC": `/fauste/player/{n}/volume-locked` (1 while locked) and the refused `/volume`. Link to [What it costs](bit-perfect.md#what-it-costs).
- Modify: `docs/technical/remote-api.md`: the player object table (L45, after `fader`): `fader_locked` | `bool` | "the fader is held at 100 % on a bit-perfect output; such a player also starts every track hard"; the `PUT /players/{id}/volume` row (L119): "409 `unavailable` below 1 on a locked player; exactly 1 accepted"; the OSC tables (L211, L229): `/fauste/player/{n}/volume-locked` | `i` 0/1; a sentence under the transport actions that `play` and `previous` on a bit-perfect player start without a crossfade (bit-perfect rate and fader spec BR16).

The anchor `bit-perfect.md#what-it-costs` is created by plan 4; until it lands the link points to the page's top (mdBook keeps the page link). If `scripts/site/check-links.sh` checks anchors and fails on it, link to `bit-perfect.md` without the anchor here and let plan 4 add the anchor in these two pages.

- [ ] **Step 1: Edit the three documents** as listed.
- [ ] **Step 2: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add docs
git commit -m "docs: describe the locked fader on MIDI and remote control

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

## Self-review notes

- Spec coverage (this plan): BF10, BF11, BR16 (MIDI) → Task 1; BF12, BF13, BR16 (remote, OSC) → Task 2; §9.1 `midi.md`, `remote-control.md` and §9.2 `remote-api.md` → Task 3.
- No refusal is added in `api.rs`: the model's (plan 1) is the one source, as BF12 asks.
