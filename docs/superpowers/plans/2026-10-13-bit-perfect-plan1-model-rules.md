# Bit-perfect Hard Transitions, Plan 1: the Model Rules

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The model decides everything the bit-perfect amendment changes: it learns each device's stream rate (BR1), knows which players are bit-perfect players (BR2), plans every transition of such a player hard, with the rate the device reopens at (BR3–BR5, BR10), starts Play-while-playing and Previous hard (BR7), and holds their fader at 100 % (BF1–BF6), with the list of faders Apply now would reset (BF8).

**Architecture:** A new pure module `fp_model::bit_perfect` holds `bit_perfect_player`, `fader_locked`, `rate_cut` and `faders_reset_by`. `EngineEvent::StreamRate` fills `LiveSettings::{stream_rates, refused_rates}` (runtime only, forgotten with the device). `TransitionPlan::StartNextAt` gains `rate_cut: Option<u32>`; `plan_for` turns a bit-perfect player's `StartNextAt` into a hard plan at the end of the play range; `start_or_crossfade` emits `StartCurrent` for a bit-perfect player; `reconcile` holds a locked fader at 1.0; `Command::SetVolume` refuses a locked fader with `ModelError::FaderLocked`. The engine receives `rate_cut` but ignores it until plan 2 (a hard `StartNextAt` already works in the engine); the UI, MIDI and remote control only gain what keeps them compiling (the `error_text` arm and its string) until plans 3 and 4.

**Tech Stack:** Rust 2024, `fp-model` (pure, no I/O), `thiserror`.

**Spec:** `docs/superpowers/specs/2026-10-08-bit-perfect-rate-and-fader-design.md` (§3, §4.1–§4.6 BR1–BR5, BR7, BR9, BR10, BR12, BR14; §5.1 BF1–BF6; BF8's `faders_reset_by`; §5.5; §6.1; §9.3). It amends `docs/superpowers/specs/2026-09-26-phase4-bit-perfect-design.md` (B1, B3.3, B7), the main spec §3.2 rules 5, 11 and R24, the live settings spec §13 and the feedback-2 spec O25: read those parts too.

This is plan 1 of 4, executed strictly in order, one PR each:
1. **this plan** — the model rules;
2. `2026-10-13-bit-perfect-plan2-engine.md` — `StreamRate` reports, the scheduled rate cut, the reopen under a hard start;
3. `2026-10-13-bit-perfect-plan3-midi-remote.md` — MIDI, the remote API and OSC;
4. `2026-10-13-bit-perfect-plan4-interface-docs.md` — the interface, the strings of §6 and the user guide.

## Rulings (planning)

**Ruling: the interface's text for `ModelError::FaderLocked`.** `fp-app`'s `error_text` (`crates/fp-app/src/ui/app.rs` ~L1542) matches `ModelError` exhaustively, so the new variant needs a Fluent string, which §6 does not list. It shows `tip-volume-locked` (§6.1), added here to all eleven locales with the spec's own translations; plan 4 uses the same key for the fader's tooltip. — Why: one sentence says the same thing in both places; the fader is disabled in plan 4, so the notice only shows if a command races the lock. — Cost if wrong: one more key and one arm.

**Ruling: a repeat or a replay follows BR3 literally.** BR4 says an R26 repeat and an O37 replay "are the same file, so their `rate_cut` is always `None` (BR3.3)". That holds only while the pass on air plays at its own rate; a pass started while a cart sounded plays resampled, and BR3 then gives `Some(file rate)` for the next pass. `plan_for` applies BR3 to whatever `preload_target` preloads, with no special case. — Why: BR3 is the rule and D1 asks the device to follow the music; the next pass becomes bit-perfect. — Cost if wrong: one `filter` in `plan_for` (`repeating(state, player) || player.next == player.current` → `None`). Reported to the maintainer.

**Ruling: where `faders_reset_by` looks.** Apply now forces every pending output item, so afterwards every device in use runs its configured settings: a player's fader would be locked when its Main would be on a device that `outputs.bit_perfect` lists. Its Main would be on the `to` of its pending Route item, or else where it is placed now. A pending route to the default output (`to: None`) is not resolved (the engine chooses the device): such a player is not listed, and BF2 still resets its fader once it is placed. — Cost if wrong: a missing warning line for that rare case; the reset itself still happens.

**Ruling: the error message names the player id.** `FaderLocked(PlayerId)` prints `the volume of player {id} is held at 100 % on a bit-perfect output` with the id's number (the model has no 1-based position in an error). — Cost if wrong: the text of one API message.

## Global Constraints

- All code, identifiers, comments, docs and commit messages in English; never mention other playout or radio-automation products.
- Behaviour lives in `fp-model` as pure functions, **one test per spec rule** (CLAUDE.md rule 7, spec D6); test names start with the rule (`br4_…`, `bf3_…`).
- §3 "Bit-perfect device": "the value the device **runs** with counts: `live.devices[d].bit_perfect` … not the wanted value of `Config`. A pending 'bit-perfect on' or 'off' changes nothing until it is applied."
- BR2: "A player not placed yet is not a bit-perfect player." Its Cue route does not count.
- BR3, verbatim conditions 1–6: bit-perfect player on Main device `d`; stream rate of `d` known; next format known, PCM (`dsd_rate` is `None`), `sample_rate` `r` not zero and differs from the stream rate; `r` not among the refused rates of `d`; every holder placed on `d` other than `PlayerMain(player)` has no cause ("This player's own CUE placed on `d` counts as something else"); `player.dsd` is `None`.
- BR4: "`StopAt` plans … are unchanged … every other player's plan has `None`."
- BF3: "refused with `ModelError::FaderLocked(p)` … unless `v` is exactly 1.0, which is accepted and changes nothing. A NaN is ignored as today."
- `unwrap`, `expect`, `panic` denied outside tests; nothing goes on air by itself (CLAUDE.md rule 10).
- Conventional Commits; every commit ends with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Gate for every task: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`.
- Progress ledger: `.superpowers/sdd/2026-10-13-bit-perfect-plan1-model-rules/progress.md`.

## Review Focus

1. **A player whose route moved onto a bit-perfect device while it was paused at 50 % volume** (live settings apply a route move to an idle player): the first event after the move must reset the fader, and the engine must receive exactly one `SetVolume { 1.0 }`, not one per event. Test: Task 5, `bf2_the_reset_is_sent_once`.
2. **The next entry analysed while the current one plays** (its format goes from unknown to 96 kHz): `reconcile` must send a new `Schedule` whose `rate_cut` is `Some(96000)`, at the same `at_secs`. Test: Task 3, `br5_an_analysed_next_brings_its_rate_cut`.
3. **The device reported a rate refused while the plan already carries a cut to it**: the next `StreamRate` must turn the plan's `rate_cut` into `None`. Test: Task 3, `br5_a_refused_rate_drops_the_cut`.
4. **`SetVolume(p, 1.0)` on a locked player sent by a remote client restoring a level**: accepted, no state change and no engine action. Test: Task 5, `bf3_a_locked_fader_refuses_any_level_but_full`.
5. **A player removed while its device keeps running** (`Gone` for its holders): the device's stream rate stays while another holder uses it and goes with the last one. Test: Task 1, `br1_the_rates_are_forgotten_with_the_last_holder`.

---

## File Structure

| File | Responsibility |
|---|---|
| `crates/fp-model/src/bit_perfect.rs` (new) | `bit_perfect_player` (BR2), `fader_locked` (BF1), `rate_cut` (BR3), `faders_reset_by` (BF8) |
| `crates/fp-model/src/lib.rs` | `pub mod bit_perfect;` and its re-exports |
| `crates/fp-model/src/command.rs` (`EngineEvent` L206, `TransitionPlan` L293) | `EngineEvent::StreamRate`; `TransitionPlan::StartNextAt::rate_cut` |
| `crates/fp-model/src/live.rs` (`LiveSettings` L75, `forget_unused_devices` L199) | `stream_rates`, `refused_rates`, `stream_rate` |
| `crates/fp-model/src/reducer.rs` (`Command::SetVolume` L108, `on_event` L337, `plan_for` L741, `start_or_crossfade` L788, `reconcile` L1315) | BR4, BR7, BF2, BF3 |
| `crates/fp-model/src/error.rs` | `ModelError::FaderLocked` |
| `crates/fp-model/src/availability.rs` | `Availability::fader`; `command_available` for `SetVolume` |
| `crates/fp-model/tests/common/mod.rs` | bit-perfect helpers shared by the model tests |
| `crates/fp-model/tests/bit_perfect_rules.rs` (new) | one test per rule |
| `crates/fp-model/tests/live.rs` | BR12 |
| `crates/fp-model/tests/availability.rs` (L28, L43) | `fader: true` in two literals |
| `crates/fp-app/src/ui/app.rs` (`error_text` ~L1542), `crates/fp-app/locales/*/main.ftl` | the `FaderLocked` notice, `tip-volume-locked` |
| every `TransitionPlan::StartNextAt { … }` literal or pattern (Task 3 lists them) | `rate_cut: None` / `..` |
| `docs/superpowers/specs/*`, `docs/technical/architecture.md` | Task 7 |

---

### Task 1: Stream rates reach the model (BR1)

**Files:**
- Modify: `crates/fp-model/src/command.rs` (`EngineEvent`, after `Applied` at L267-271)
- Modify: `crates/fp-model/src/live.rs` (`LiveSettings` L75-92, `forget_unused_devices` L199-205, a new `stream_rate` after `audio_system_in_use` ~L240)
- Modify: `crates/fp-model/src/reducer.rs` (`on_event`, the `Applied` arm ~L452)
- Modify: `crates/fp-model/tests/common/mod.rs` (helpers)
- Test: `crates/fp-model/tests/bit_perfect_rules.rs` (new)

**Interfaces:**
- Produces: `EngineEvent::StreamRate { device: OutputDevice, rate: u32, refused: Vec<u32> }`; `LiveSettings::stream_rates: BTreeMap<OutputDevice, u32>`, `LiveSettings::refused_rates: BTreeMap<OutputDevice, Vec<u32>>`; test helpers `bp_device()`, `bp_route()`, `place_on_dac(&mut AppState, Holder) -> Vec<EngineAction>`, `report_stream_rate(&mut AppState, u32, Vec<u32>) -> Vec<EngineAction>`, `set_track_rates(&mut AppState, &[u32])`.

- [ ] **Step 1: The shared helpers**

Append to `crates/fp-model/tests/common/mod.rs` (extend its `use fp_model::{…}` with `AudioFormat, EngineAction, EngineEvent, Holder, OutputDevice, device_settings, on_event`):

```rust
/// The device the bit-perfect tests play on.
pub fn bp_device() -> OutputDevice {
    OutputDevice {
        backend: "null".to_owned(),
        device: "dac".to_owned(),
    }
}

/// A route to `bp_device`, on its first channel pair.
pub fn bp_route() -> Route {
    Route {
        backend: "null".to_owned(),
        device: "dac".to_owned(),
        first_channel: 0,
    }
}

/// Routes `holder` to `dac`, lists `dac` as bit-perfect, and reports the
/// engine placing `holder` there with `dac` running bit-perfect (live
/// settings L3): nothing is pending afterwards.
pub fn place_on_dac(state: &mut AppState, holder: Holder) -> Vec<EngineAction> {
    let route = bp_route();
    let outputs = &mut state.config.outputs;
    match holder {
        Holder::PlayerMain(p) | Holder::PlayerCue(p) => {
            let routes = outputs.routes.iter_mut().find(|r| r.player == p).unwrap();
            if matches!(holder, Holder::PlayerMain(_)) {
                routes.main = Some(route.clone());
            } else {
                routes.cue = Some(route.clone());
            }
        }
        Holder::CartwallMain => outputs.cartwall.main = Some(route.clone()),
        Holder::CartwallCue => outputs.cartwall.cue = Some(route.clone()),
    }
    if !outputs.bit_perfect.contains(&bp_device()) {
        outputs.bit_perfect.push(bp_device());
    }
    let running = device_settings(&state.config.outputs, &bp_device());
    on_event(
        state,
        EngineEvent::Placed {
            holder,
            route: Some(route),
            device: bp_device(),
            running,
        },
    )
}

/// The engine reports `dac` running at `rate`, having refused `refused` (BR1).
pub fn report_stream_rate(state: &mut AppState, rate: u32, refused: Vec<u32>) -> Vec<EngineAction> {
    on_event(
        state,
        EngineEvent::StreamRate {
            device: bp_device(),
            rate,
            refused,
        },
    )
}

/// Gives the tracks of the first playlist, in order, 16-bit stereo PCM at `rates`.
pub fn set_track_rates(state: &mut AppState, rates: &[u32]) {
    for (entry, rate) in entries(state).into_iter().zip(rates) {
        let track = state.playlists.entry(entry).unwrap().track;
        state.library.get_mut(track).unwrap().format = Some(AudioFormat {
            sample_rate: *rate,
            bits: Some(16),
            channels: 2,
            dsd_rate: None,
        });
    }
}
```

- [ ] **Step 2: Write the failing tests**

`crates/fp-model/tests/bit_perfect_rules.rs`:

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Bit-perfect rate and fader spec (2026-10-08): one test per rule BR1–BR14
//! and BF1–BF8 that the model decides. The engine is stood in for by the
//! events it reports.

mod common;

use common::{bp_device, fixture, p0, place_on_dac, report_stream_rate};
use fp_model::{EngineEvent, Holder};

#[test]
fn br1_a_stream_rate_report_is_kept_per_device() {
    let mut s = fixture(3);
    let p = p0(&s);
    place_on_dac(&mut s, Holder::PlayerMain(p));
    assert_eq!(s.live.stream_rates.get(&bp_device()), None, "unknown until reported");
    report_stream_rate(&mut s, 96_000, vec![]);
    assert_eq!(s.live.stream_rates.get(&bp_device()), Some(&96_000));
    assert_eq!(s.live.refused_rates.get(&bp_device()), Some(&vec![]));
    report_stream_rate(&mut s, 44_100, vec![192_000]);
    assert_eq!(s.live.stream_rates.get(&bp_device()), Some(&44_100));
    assert_eq!(s.live.refused_rates.get(&bp_device()), Some(&vec![192_000]));
}

#[test]
fn br1_the_rates_are_forgotten_with_the_last_holder() {
    let mut s = fixture(3);
    let (p, q) = (s.players[0].id, s.players[1].id);
    place_on_dac(&mut s, Holder::PlayerMain(p));
    place_on_dac(&mut s, Holder::PlayerMain(q));
    report_stream_rate(&mut s, 96_000, vec![44_100]);
    fp_model::on_event(&mut s, EngineEvent::Gone { holder: Holder::PlayerMain(p) });
    assert_eq!(s.live.stream_rates.get(&bp_device()), Some(&96_000), "q still plays there");
    fp_model::on_event(
        &mut s,
        EngineEvent::Unplaced {
            holder: Holder::PlayerMain(q),
            route: None,
        },
    );
    assert_eq!(s.live.stream_rates.get(&bp_device()), None);
    assert_eq!(s.live.refused_rates.get(&bp_device()), None);
}
```

- [ ] **Step 3: Run to verify it fails**

Run: `cargo test -p fp-model --test bit_perfect_rules 2>&1 | tail -5`
Expected: FAIL to compile ("no variant named `StreamRate`", "no field `stream_rates`").

- [ ] **Step 4: Implement**

`command.rs`, in `EngineEvent` after `Applied { … }`:

```rust
    /// Bit-perfect rate and fader spec BR1: `device`'s stream runs at
    /// `rate`, and it refused `refused` (ascending; B3.4). Reported when its
    /// bus opens and whenever either changes.
    StreamRate {
        device: OutputDevice,
        rate: u32,
        refused: Vec<u32>,
    },
```

`live.rs`, in `LiveSettings` after `failures`:

```rust
    /// The rate each device's stream runs at now (bit-perfect rate and
    /// fader spec BR1); a device missing here has an unknown stream rate.
    pub stream_rates: BTreeMap<OutputDevice, u32>,
    /// The rates each device refused (B3.4), ascending.
    pub refused_rates: BTreeMap<OutputDevice, Vec<u32>>,
```

`forget_unused_devices` gains, after `live.devices.retain(…)`:

```rust
    live.stream_rates.retain(|d, _| used.contains(d));
    live.refused_rates.retain(|d, _| used.contains(d));
```

After `audio_system_in_use`:

```rust
/// BR1: `EngineEvent::StreamRate`.
pub(crate) fn stream_rate(state: &mut AppState, device: OutputDevice, rate: u32, refused: Vec<u32>) {
    state.live.stream_rates.insert(device.clone(), rate);
    state.live.refused_rates.insert(device, refused);
}
```

`reducer.rs`, `on_event`, after the `Applied` arm:

```rust
        EngineEvent::StreamRate {
            device,
            rate,
            refused,
        } => crate::live::stream_rate(state, device, rate, refused),
```

- [ ] **Step 5: Run to verify it passes**

Run: `cargo test -p fp-model --test bit_perfect_rules br1 && cargo test -p fp-model --test live`
Expected: PASS (`the_live_state_starts_empty_and_unknown` still holds: both maps start empty).

- [ ] **Step 6: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-model
git commit -m "feat(model): keep each device's stream rate

The engine will report the rate a device's stream runs at and the rates
it refused; the model keeps them per device, runtime only, and forgets
them with the device (bit-perfect rate and fader spec BR1).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: Bit-perfect players and the rate a cut reopens at (BR2, BR3, BF1)

**Files:**
- Create: `crates/fp-model/src/bit_perfect.rs`
- Modify: `crates/fp-model/src/lib.rs` (module list L5-31, re-exports)
- Test: `crates/fp-model/tests/bit_perfect_rules.rs`

**Interfaces:**
- Consumes: `LiveSettings::{placement, devices, stream_rates, refused_rates}`; `fp_model::live::{causes, holders}`.
- Produces: `pub fn bit_perfect_player(state: &AppState, player: PlayerId) -> bool`; `pub fn fader_locked(state: &AppState, player: PlayerId) -> bool`; `pub fn rate_cut(state: &AppState, player: PlayerId, next: &SourceRequest) -> Option<u32>`; all re-exported from the crate root.

- [ ] **Step 1: Write the failing tests**

Add to `bit_perfect_rules.rs` (extend the `fp_model` imports with `AppState, Command, CueState, DsdOnAir, EntryId, PlayerId, SourceRequest, apply, bit_perfect_player, fader_locked, rate_cut`, and the `common` ones with `entries, set_track_rates`; each later task adds what its tests use, so no import is ever unused at a gate):

```rust
/// `fixture(3)`: P1's Main on `dac`, which runs bit-perfect at 44.1 kHz;
/// every track 44.1 kHz 16-bit; P1 plays its first entry.
fn on_air() -> (AppState, PlayerId, [EntryId; 3]) {
    let mut s = fixture(3);
    let p = p0(&s);
    let e = entries(&s);
    place_on_dac(&mut s, Holder::PlayerMain(p));
    report_stream_rate(&mut s, 44_100, vec![]);
    set_track_rates(&mut s, &[44_100, 44_100, 44_100]);
    apply(&mut s, Command::Play(p)).unwrap();
    (s, p, [e[0], e[1], e[2]])
}

/// The request P1 preloads for `entry`, at its cue-in.
fn next_request(s: &AppState, entry: EntryId) -> SourceRequest {
    s.request_from_cue_in(entry).unwrap()
}

fn rates(s: &mut AppState, next: u32) {
    set_track_rates(s, &[44_100, next, 44_100]);
}

#[test]
fn br2_a_main_placed_on_a_device_running_bit_perfect() {
    let (s, p, _) = on_air();
    assert!(bit_perfect_player(&s, p));
    assert!(!bit_perfect_player(&s, s.players[1].id), "not placed yet");
}

#[test]
fn br2_the_running_value_counts_not_the_wanted_one() {
    let mut s = fixture(3);
    let p = p0(&s);
    place_on_dac(&mut s, Holder::PlayerMain(p));
    // Wanted off, still running on: pending until applied.
    s.config.outputs.bit_perfect.clear();
    assert!(bit_perfect_player(&s, p));
    let mut s = fixture(3);
    let p = p0(&s);
    place_on_dac(&mut s, Holder::PlayerMain(p));
    let running = s.live.devices.get_mut(&bp_device()).unwrap();
    running.bit_perfect = false;
    // Wanted on (the configuration lists it), not running yet.
    assert!(!bit_perfect_player(&s, p));
}

#[test]
fn br2_a_cue_route_on_the_device_does_not_count() {
    let mut s = fixture(3);
    let p = p0(&s);
    place_on_dac(&mut s, Holder::PlayerCue(p));
    assert!(!bit_perfect_player(&s, p));
}

#[test]
fn bf1_the_fader_is_locked_exactly_for_a_bit_perfect_player() {
    let (s, p, _) = on_air();
    assert!(fader_locked(&s, p));
    let q = s.players[1].id;
    assert_eq!(fader_locked(&s, q), bit_perfect_player(&s, q));
    assert!(!fader_locked(&s, q), "unplaced is not locked");
}

#[test]
fn br3_all_conditions_hold_gives_the_next_rate() {
    let (mut s, p, [_, b, _]) = on_air();
    rates(&mut s, 96_000);
    assert_eq!(rate_cut(&s, p, &next_request(&s, b)), Some(96_000));
}

#[test]
fn br3_1_not_a_bit_perfect_player() {
    let (mut s, p, [_, b, _]) = on_air();
    rates(&mut s, 96_000);
    s.live.devices.get_mut(&bp_device()).unwrap().bit_perfect = false;
    assert_eq!(rate_cut(&s, p, &next_request(&s, b)), None);
}

#[test]
fn br3_2_the_stream_rate_is_unknown() {
    let (mut s, p, [_, b, _]) = on_air();
    rates(&mut s, 96_000);
    s.live.stream_rates.clear();
    assert_eq!(rate_cut(&s, p, &next_request(&s, b)), None);
}

#[test]
fn br3_3_the_next_format_is_unknown_dsd_zero_or_the_same_rate() {
    let (mut s, p, [_, b, _]) = on_air();
    let track = s.playlists.entry(b).unwrap().track;
    for format in [
        None,
        Some(fp_model::AudioFormat {
            sample_rate: 88_200,
            bits: None,
            channels: 2,
            dsd_rate: Some(2_822_400),
        }),
        Some(fp_model::AudioFormat {
            sample_rate: 0,
            bits: Some(16),
            channels: 2,
            dsd_rate: None,
        }),
        Some(fp_model::AudioFormat {
            sample_rate: 44_100,
            bits: Some(16),
            channels: 2,
            dsd_rate: None,
        }),
    ] {
        s.library.get_mut(track).unwrap().format = format;
        assert_eq!(rate_cut(&s, p, &next_request(&s, b)), None, "{format:?}");
    }
}

#[test]
fn br3_4_a_rate_the_device_refused() {
    let (mut s, p, [_, b, _]) = on_air();
    rates(&mut s, 96_000);
    report_stream_rate(&mut s, 44_100, vec![96_000]);
    assert_eq!(rate_cut(&s, p, &next_request(&s, b)), None);
}

#[test]
fn br3_5_something_else_sounds_on_the_device() {
    // Another player playing there.
    let (mut s, p, [_, b, _]) = on_air();
    rates(&mut s, 96_000);
    let q = s.players[1].id;
    place_on_dac(&mut s, Holder::PlayerMain(q));
    assert_eq!(rate_cut(&s, p, &next_request(&s, b)), Some(96_000), "q is stopped");
    apply(&mut s, Command::Play(q)).unwrap();
    assert_eq!(rate_cut(&s, p, &next_request(&s, b)), None);
    // This player's own CUE placed on the device.
    let (mut s, p, [_, b, c]) = on_air();
    rates(&mut s, 96_000);
    s.live.placement.insert(Holder::PlayerCue(p), bp_device());
    s.players[0].cue = Some(CueState {
        entry: c,
        paused: false,
    });
    assert_eq!(rate_cut(&s, p, &next_request(&s, b)), None);
}

#[test]
fn br3_6_a_dsd_stream_on_air() {
    let (mut s, p, [a, b, _]) = on_air();
    rates(&mut s, 96_000);
    s.players[0].dsd = Some(DsdOnAir {
        entry: a,
        hold_others: false,
    });
    assert_eq!(rate_cut(&s, p, &next_request(&s, b)), None);
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p fp-model --test bit_perfect_rules 2>&1 | tail -5`
Expected: FAIL to compile ("unresolved imports `fp_model::bit_perfect_player`…").

- [ ] **Step 3: Implement**

`crates/fp-model/src/bit_perfect.rs`:

```rust
//! Bit-perfect rate and fader spec (2026-10-08): which players are
//! bit-perfect players, the rate a hard transition reopens their device
//! at, and their locked fader. Pure functions of `AppState`; the device
//! values are the ones the engine reported it **runs** with (`live`).

use crate::command::SourceRequest;
use crate::config::OutputDevice;
use crate::ids::PlayerId;
use crate::live::{Holder, causes, holders};
use crate::state::AppState;

/// Where the engine placed `player`'s Main; `None` before it is placed.
fn main_device(state: &AppState, player: PlayerId) -> Option<&OutputDevice> {
    state.live.placement.get(&Holder::PlayerMain(player))
}

/// BR2: `player`'s Main is placed on a device that runs bit-perfect. Its
/// Cue route does not count; a player not placed yet is not one.
pub fn bit_perfect_player(state: &AppState, player: PlayerId) -> bool {
    main_device(state, player)
        .and_then(|d| state.live.devices.get(d))
        .is_some_and(|running| running.bit_perfect)
}

/// BF1: the player's volume is held at exactly 1.0.
pub fn fader_locked(state: &AppState, player: PlayerId) -> bool {
    bit_perfect_player(state, player)
}

/// BR3: the rate `player`'s Main device reopens at during the hard
/// transition into `next`, or `None` when it keeps its rate (the next
/// track then plays at the running rate, resampled if it differs).
pub fn rate_cut(state: &AppState, player: PlayerId, next: &SourceRequest) -> Option<u32> {
    // 1. A bit-perfect player, on Main device `d`.
    if !bit_perfect_player(state, player) {
        return None;
    }
    let device = main_device(state, player)?;
    // 2. The stream rate of `d` is known.
    let stream = *state.live.stream_rates.get(device)?;
    // 3. A known PCM format at a rate other than the stream's.
    let format = next.format.filter(|f| f.dsd_rate.is_none())?;
    let rate = format.sample_rate;
    if rate == 0 || rate == stream {
        return None;
    }
    // 4. Not a rate the device refused.
    if state
        .live
        .refused_rates
        .get(device)
        .is_some_and(|refused| refused.contains(&rate))
    {
        return None;
    }
    // 5. Nothing else sounds on `d` (this player's own CUE counts).
    let others_quiet = holders(state)
        .into_iter()
        .filter(|h| *h != Holder::PlayerMain(player))
        .filter(|h| state.live.placement.get(h) == Some(device))
        .all(|h| causes(state, h).is_empty());
    if !others_quiet {
        return None;
    }
    // 6. The current entry does not go out as DSD (O25 ends that stream).
    if state.player(player).ok()?.dsd.is_some() {
        return None;
    }
    Some(rate)
}
```

`lib.rs`: add `pub mod bit_perfect;` (alphabetically after `pub mod availability;`) and

```rust
pub use bit_perfect::{bit_perfect_player, fader_locked, rate_cut};
```

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p fp-model --test bit_perfect_rules`
Expected: PASS.

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-model
git commit -m "feat(model): tell bit-perfect players and the rate a cut reopens at

A player whose Main runs on a bit-perfect device is a bit-perfect player
and its fader is locked; the rate its device would reopen at for the next
track follows the six conditions of the spec (bit-perfect rate and fader
spec BR2, BR3, BF1).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: A bit-perfect player's plan is always hard (BR4, BR5, BR10, BR12, BR14)

**Files:**
- Modify: `crates/fp-model/src/command.rs` (`TransitionPlan` L293-303)
- Modify: `crates/fp-model/src/reducer.rs` (`plan_for` L741-748)
- Modify (compile fixes, `rate_cut: None` in literals, `..` in exhaustive patterns): `crates/fp-model/src/reducer.rs` (`ordinary_plan` L826-829, L840-843, L855-862), `crates/fp-engine/src/engine.rs` (`current_finished` ~L2928-2934, `dispatch` ~L3197-3203), and the tests `crates/fp-model/tests/{analysis,cue_markers_off,dsd_rules,entry_flags,header_duration,markers,replay_next,review_fixes,single_stop_after,transitions}.rs`, `crates/fp-engine/tests/{audio_path_audit,dsd_output,engine,review_fixes}.rs` — find them all with `grep -rn "StartNextAt {" crates --include=*.rs`
- Test: `crates/fp-model/tests/bit_perfect_rules.rs`, `crates/fp-model/tests/live.rs`

**Interfaces:**
- Consumes: `rate_cut`, `bit_perfect_player` (Task 2).
- Produces: `TransitionPlan::StartNextAt { at_secs: f64, fade_current_until_secs: Option<f64>, rate_cut: Option<u32> }` (still `Copy`). Plan 2 executes `rate_cut`.

- [ ] **Step 1: Write the failing tests**

Add to `bit_perfect_rules.rs` (extend the `fp_model` imports with `DeviceSettings, EngineAction, MarkerKind, PlayMode, Target, TrackAnalysis, TransitionPlan, Wanted, on_event, plan_for`):

```rust
fn plan(s: &AppState, p: PlayerId) -> Option<TransitionPlan> {
    plan_for(s, s.player(p).unwrap())
}

/// A MIX point at 170 s on every track (they are 180 s long).
fn with_segue(s: &mut AppState) {
    s.config.players.auto_segue = true;
    for t in s.library.iter_mut() {
        t.markers.set_auto(MarkerKind::SegueStart, Some(170.0));
    }
}

fn hard(rate_cut: Option<u32>) -> Option<TransitionPlan> {
    Some(TransitionPlan::StartNextAt {
        at_secs: 180.0,
        fade_current_until_secs: None,
        rate_cut,
    })
}

const SEGUE: Option<TransitionPlan> = Some(TransitionPlan::StartNextAt {
    at_secs: 170.0,
    fade_current_until_secs: Some(180.0),
    rate_cut: None,
});

/// The last `Schedule` sent for `p` in `out`.
fn scheduled(out: &[EngineAction], p: PlayerId) -> Option<Option<TransitionPlan>> {
    out.iter().rev().find_map(|a| match a {
        EngineAction::Schedule { player, plan } if *player == p => Some(*plan),
        _ => None,
    })
}

#[test]
fn br4_a_bit_perfect_player_plays_to_its_end_and_never_segues() {
    let (mut s, p, _) = on_air();
    with_segue(&mut s);
    assert_eq!(plan(&s, p), hard(None), "same rate: no reopen");
    rates(&mut s, 96_000);
    assert_eq!(plan(&s, p), hard(Some(96_000)));
}

#[test]
fn br4_stop_plans_are_unchanged() {
    let (mut s, p, _) = on_air();
    with_segue(&mut s);
    apply(&mut s, Command::SetMode(p, PlayMode::Single)).unwrap();
    assert_eq!(plan(&s, p), Some(TransitionPlan::StopAt { at_secs: 180.0 }));
}

#[test]
fn br4_off_bit_perfect_the_plan_is_the_segue() {
    let (mut s, p, _) = on_air();
    with_segue(&mut s);
    s.live.devices.get_mut(&bp_device()).unwrap().bit_perfect = false;
    assert_eq!(plan(&s, p), SEGUE);
}

#[test]
fn br5_a_cart_on_the_device_drops_the_reopen_and_its_end_brings_it_back() {
    let (mut s, p, _) = on_air();
    with_segue(&mut s);
    rates(&mut s, 96_000);
    // Any command reconciles: the plan now carries the cut.
    apply(&mut s, Command::SetMode(p, PlayMode::Continuous)).unwrap();
    assert_eq!(plan(&s, p), hard(Some(96_000)));
    place_on_dac(&mut s, Holder::CartwallMain);
    let cart = load_cart(&mut s);
    let out = apply(&mut s, Command::FireCart(cart)).unwrap();
    assert_eq!(scheduled(&out, p), Some(hard(None)), "{out:?}");
    let out = on_event(&mut s, EngineEvent::CartEnded { cart });
    assert_eq!(scheduled(&out, p), Some(hard(Some(96_000))), "{out:?}");
}

#[test]
fn br5_an_analysed_next_brings_its_rate_cut() {
    let (mut s, p, [_, b, _]) = on_air();
    let track = s.playlists.entry(b).unwrap().track;
    s.library.get_mut(track).unwrap().format = None;
    // Any command reconciles; the plan is hard with no reopen.
    apply(&mut s, Command::SetMode(p, PlayMode::Continuous)).unwrap();
    assert_eq!(plan(&s, p), hard(None));
    let analysis = TrackAnalysis {
        duration_secs: 180.0,
        format: Some(fp_model::AudioFormat {
            sample_rate: 96_000,
            bits: Some(24),
            channels: 2,
            dsd_rate: None,
        }),
        ..TrackAnalysis::default()
    };
    let out = apply(
        &mut s,
        Command::ApplyAnalysis {
            track,
            analysis: Box::new(analysis),
        },
    )
    .unwrap();
    assert_eq!(scheduled(&out, p), Some(hard(Some(96_000))), "{out:?}");
}

#[test]
fn br5_a_refused_rate_drops_the_cut() {
    let (mut s, p, _) = on_air();
    rates(&mut s, 96_000);
    apply(&mut s, Command::SetMode(p, PlayMode::Continuous)).unwrap();
    assert_eq!(plan(&s, p), hard(Some(96_000)));
    let out = report_stream_rate(&mut s, 44_100, vec![96_000]);
    assert_eq!(scheduled(&out, p), Some(hard(None)));
}

#[test]
fn br5_moved_off_the_device_gives_back_the_segue() {
    let (mut s, p, _) = on_air();
    with_segue(&mut s);
    apply(&mut s, Command::SetMode(p, PlayMode::Continuous)).unwrap();
    let speakers = fp_model::OutputDevice {
        backend: "null".into(),
        device: "speakers".into(),
    };
    let running = fp_model::device_settings(&s.config.outputs, &speakers);
    let out = on_event(
        &mut s,
        EngineEvent::Placed {
            holder: Holder::PlayerMain(p),
            route: None,
            device: speakers,
            running,
        },
    );
    assert_eq!(scheduled(&out, p), Some(SEGUE));
}

#[test]
fn br10_an_unanalysed_next_is_hard_without_a_reopen() {
    let (mut s, p, [_, b, _]) = on_air();
    let track = s.playlists.entry(b).unwrap().track;
    s.library.get_mut(track).unwrap().format = None;
    assert_eq!(plan(&s, p), hard(None));
}

#[test]
fn br10_dsd_next_or_dsd_on_air_takes_no_cut_and_the_hold_still_stops() {
    let (mut s, p, [a, b, _]) = on_air();
    let track = s.playlists.entry(b).unwrap().track;
    s.library.get_mut(track).unwrap().format = Some(fp_model::AudioFormat {
        sample_rate: 88_200,
        bits: None,
        channels: 2,
        dsd_rate: Some(2_822_400),
    });
    assert_eq!(plan(&s, p), hard(None), "the PCM of a DSD next");
    rates(&mut s, 96_000);
    s.players[0].dsd = Some(DsdOnAir {
        entry: a,
        hold_others: false,
    });
    assert_eq!(plan(&s, p), hard(None), "a DSD stream on air");
    s.players[0].dsd = Some(DsdOnAir {
        entry: a,
        hold_others: true,
    });
    assert_eq!(plan(&s, p), Some(TransitionPlan::StopAt { at_secs: 180.0 }));
}

#[test]
fn br10_another_player_on_an_ordinary_device_keeps_its_segue() {
    let (mut s, _, _) = on_air();
    with_segue(&mut s);
    let q = s.players[1].id;
    apply(&mut s, Command::Play(q)).unwrap();
    assert_eq!(plan(&s, q), SEGUE);
}

#[test]
fn br14_bit_perfect_turned_off_gives_back_the_ordinary_plan() {
    let (mut s, p, _) = on_air();
    with_segue(&mut s);
    apply(&mut s, Command::SetMode(p, PlayMode::Continuous)).unwrap();
    let off = DeviceSettings {
        bit_perfect: false,
        ..*s.live.devices.get(&bp_device()).unwrap()
    };
    let out = on_event(
        &mut s,
        EngineEvent::Applied {
            target: Target::Device(bp_device()),
            wanted: Wanted::Device(off),
            outcome: Ok(()),
        },
    );
    assert_eq!(scheduled(&out, p), Some(SEGUE));
}

/// Gives cart 0 of the first page a 10 s file.
fn load_cart(state: &mut AppState) -> fp_model::CartId {
    let page = state.cartwall.pages[0].id;
    let path = std::path::PathBuf::from("/carts/cart0.wav");
    apply(state, Command::AssignCartFile { page, index: 0, path }).unwrap();
    let track = state.cartwall.pages[0].carts[0].track.unwrap();
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
    state.cartwall.pages[0].carts[0].id
}
```

(`rates` changes the library directly, which sends nothing; the `SetMode` that follows it in these tests is only there to run `reconcile`, as any command does.)

`crates/fp-model/tests/live.rs` (extend `use common::{…}` with `place_on_dac, report_stream_rate, set_track_rates`):

```rust
#[test]
fn br12_a_rate_cut_keeps_a_pending_device_change_waiting_until_the_transition() {
    let mut s = fixture(3);
    let p = p0(&s);
    let e = entries(&s);
    place_on_dac(&mut s, Holder::PlayerMain(p));
    report_stream_rate(&mut s, 44_100, vec![]);
    set_track_rates(&mut s, &[44_100, 96_000, 44_100]);
    apply(&mut s, Command::Play(p)).unwrap();
    let mut config = s.config.clone();
    config.outputs.set_device_buffer(&common::bp_device(), Some(256));
    let out = apply(&mut s, Command::UpdateConfig(Box::new(config))).unwrap();
    assert!(
        !out.iter().any(|a| matches!(a, EngineAction::ApplyDevice { .. })),
        "P1 plays: the buffer waits"
    );
    assert!(due(&s).is_empty());
    // The engine cuts: the old track stopped, the device reopens, the next
    // starts. The model hears only `TransitionStarted`; P1 stays Playing.
    report_stream_rate(&mut s, 96_000, vec![]);
    assert!(due(&s).is_empty(), "during the gap");
    let out = on_event(
        &mut s,
        EngineEvent::TransitionStarted {
            player: p,
            entry: e[1],
        },
    );
    assert!(!out.iter().any(|a| matches!(a, EngineAction::ApplyDevice { .. })));
    assert!(due(&s).is_empty(), "still playing after the transition");
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p fp-model --test bit_perfect_rules br4 2>&1 | tail -5`
Expected: FAIL to compile ("struct variant `TransitionPlan::StartNextAt` has no field named `rate_cut`").

- [ ] **Step 3: Implement**

`command.rs`:

```rust
    /// Start the preloaded next at `at_secs`. When `fade_current_until_secs`
    /// is set, the current source keeps playing and fades out until then
    /// (overlap). With `rate_cut`, a hard transition on a bit-perfect
    /// device that reopens at that rate between the two tracks (bit-perfect
    /// rate and fader spec BR4, BR6); `None` for every other plan.
    StartNextAt {
        at_secs: f64,
        fade_current_until_secs: Option<f64>,
        rate_cut: Option<u32>,
    },
```

Add `rate_cut: None,` to every `StartNextAt` literal and `..` to every pattern that lists the fields without it (`grep -rn "StartNextAt {" crates --include=*.rs`; the compiler names each one: E0063 "missing field `rate_cut`", E0027 "pattern does not mention field `rate_cut`"). In `crates/fp-engine/src/engine.rs` `dispatch` the pattern `TransitionPlan::StartNextAt { at_secs, fade_current_until_secs }` becomes `TransitionPlan::StartNextAt { at_secs, fade_current_until_secs, .. }`; plan 2 gives `rate_cut` its meaning there.

`reducer.rs`, `plan_for`:

```rust
/// Rules 9–11 (see `ordinary_plan`), then O25: a DSD stream that holds its
/// output is never overlapped; it plays to its end and the next entry
/// starts from `ReachedEnd` (`dsd_hold_continue`). Then BR4: a bit-perfect
/// player never segues; it plays to the end of its play range, and the
/// device reopens at the next entry's rate when BR3 allows.
pub fn plan_for(state: &AppState, player: &PlayerState) -> Option<TransitionPlan> {
    let plan = ordinary_plan(state, player)?;
    if !matches!(plan, TransitionPlan::StartNextAt { .. }) {
        return Some(plan);
    }
    if dsd_holds(player) {
        let end = current_end(state, player)?;
        return Some(TransitionPlan::StopAt { at_secs: end });
    }
    if crate::bit_perfect::bit_perfect_player(state, player.id) {
        let end = current_end(state, player)?;
        let rate_cut = preload_target(state, player)
            .and_then(|e| crate::pending_start::preload_request(state, player, e))
            .and_then(|next| crate::bit_perfect::rate_cut(state, player.id, &next));
        return Some(TransitionPlan::StartNextAt {
            at_secs: end,
            fade_current_until_secs: None,
            rate_cut,
        });
    }
    Some(plan)
}
```

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p fp-model --test bit_perfect_rules && cargo test -p fp-model --test live br12 && cargo test -p fp-engine`
Expected: PASS (the engine compiles with `rate_cut` ignored).

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates
git commit -m "feat(model): plan every transition of a bit-perfect player hard

A player whose Main runs bit-perfect plays each track to the end of its
play range, never to its MIX point, and its plan carries the rate the
device reopens at when nothing else sounds there; the plan follows the
state until the transition (bit-perfect rate and fader spec BR4, BR5,
BR10, BR12, BR14).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: Manual starts over the current are hard (BR7, BR9)

**Files:**
- Modify: `crates/fp-model/src/reducer.rs` (`start_or_crossfade` L786-806; the doc of `previous` L495-497)
- Test: `crates/fp-model/tests/bit_perfect_rules.rs`

**Interfaces:**
- Consumes: `bit_perfect_player` (Task 2).

- [ ] **Step 1: Write the failing tests**

```rust
/// What a manual start sent: `Ok(entry)` for `StartCurrent`, `Err(entry)` for `Crossfade`.
fn start_of(out: &[EngineAction], p: PlayerId) -> Option<Result<EntryId, EntryId>> {
    out.iter().find_map(|a| match a {
        EngineAction::StartCurrent { player, request } if *player == p => Some(Ok(request.entry)),
        EngineAction::Crossfade { player, request, .. } if *player == p => Some(Err(request.entry)),
        _ => None,
    })
}

#[test]
fn br7_play_while_playing_and_previous_start_hard_at_any_rate() {
    for next_rate in [44_100, 96_000] {
        let (mut s, p, [a, b, _]) = on_air();
        rates(&mut s, next_rate);
        let out = apply(&mut s, Command::Play(p)).unwrap();
        assert_eq!(start_of(&out, p), Some(Ok(b)), "{next_rate}");
        assert!(!s.player(p).unwrap().fading, "no Mixing badge");
        let out = apply(&mut s, Command::Previous(p)).unwrap();
        assert_eq!(start_of(&out, p), Some(Ok(a)), "{next_rate}");
        assert!(!s.player(p).unwrap().fading);
    }
}

#[test]
fn br7_play_on_the_entry_on_air_set_as_next_starts_hard() {
    let (mut s, p, [a, _, _]) = on_air();
    apply(&mut s, Command::SetNext(p, a)).unwrap();
    let out = apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(start_of(&out, p), Some(Ok(a)));
}

#[test]
fn br7_another_player_still_crossfades() {
    let (mut s, _, _) = on_air();
    let q = s.players[1].id;
    apply(&mut s, Command::Play(q)).unwrap();
    let out = apply(&mut s, Command::Play(q)).unwrap();
    assert!(matches!(start_of(&out, q), Some(Err(_))), "{out:?}");
    assert!(s.player(q).unwrap().fading);
}

#[test]
fn br9_fade_stop_pause_and_restart_are_unchanged() {
    let (mut s, p, _) = on_air();
    let fade_ms = s.config.players.fade_ms;
    let out = apply(&mut s, Command::Pause(p)).unwrap();
    assert!(out.contains(&EngineAction::Pause { player: p }));
    apply(&mut s, Command::Play(p)).unwrap(); // resume
    let out = apply(&mut s, Command::Restart(p)).unwrap();
    assert!(out.iter().any(|a| matches!(a, EngineAction::Seek { player, .. } if *player == p)));
    let out = apply(&mut s, Command::FadeStop(p)).unwrap();
    assert!(out.contains(&EngineAction::FadeOutAndStop { player: p, fade_ms }));
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p fp-model --test bit_perfect_rules br7 2>&1 | tail -8`
Expected: FAIL — `br7_play_while_playing…` gets `Some(Err(b))` (a crossfade). `br7_another_player_still_crossfades` and `br9_…` pass already; record in the ledger that they pin behaviour that must not change.

- [ ] **Step 3: Implement**

`start_or_crossfade`:

```rust
/// Play-while-playing starts: a crossfade, or a hard start that cuts the
/// current track with the declick: a DSD stream holding its output (O25
/// HoldOthers, `held`) or any bit-perfect player (bit-perfect rate and
/// fader spec BR7), which never enters `fading`.
fn start_or_crossfade(
    state: &mut AppState,
    i: usize,
    held: bool,
    request: SourceRequest,
    out: &mut Vec<EngineAction>,
) {
    let player = state.players[i].id;
    if held || crate::bit_perfect::bit_perfect_player(state, player) {
        out.push(EngineAction::StartCurrent { player, request });
        return;
    }
    out.push(EngineAction::Crossfade {
        player,
        request,
        fade_ms: state.config.players.fade_ms,
    });
    state.players[i].fading = true;
}
```

In the doc of `previous`, "crossfades into it like Play-while-Playing" becomes "starts it like Play-while-Playing (a crossfade, or a hard start on a bit-perfect player)".

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p fp-model`
Expected: PASS.

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-model
git commit -m "feat(model): start Next and Previous hard on a bit-perfect player

Play while playing, Play on the entry on air set as next, and Previous
cut the current track with the declick instead of crossfading when the
player's Main is bit-perfect; Fade stop, pause and restart are unchanged
(bit-perfect rate and fader spec BR7, BR9).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: The locked fader (BF2–BF6)

**Files:**
- Modify: `crates/fp-model/src/error.rs` (after `CartGridOutOfRange` L42-43)
- Modify: `crates/fp-model/src/reducer.rs` (`Command::SetVolume` arm L108-121; `reconcile` L1315-1318)
- Modify: `crates/fp-model/src/availability.rs` (struct L11-22, `availability` L35-47, `command_available` L53-71)
- Modify: `crates/fp-model/tests/availability.rs` (the literals at L28 and L43 gain `fader: true,`)
- Modify: `crates/fp-app/src/ui/app.rs` (`error_text`, ~L1542-1565)
- Modify: `crates/fp-app/locales/{ca-ES,de-DE,en-US,es-ES,eu-ES,fr-FR,gl-ES,it-IT,nl-NL,pl-PL,pt-PT}/main.ftl` (after `tip-volume` at L48 in en-US; the same place in the others)
- Test: `crates/fp-model/tests/bit_perfect_rules.rs` (extend the `fp_model` imports with nothing new; use `common::roundtrip` by path)

**Interfaces:**
- Consumes: `fader_locked` (Task 2).
- Produces: `ModelError::FaderLocked(PlayerId)`; `Availability::fader: bool`; the Fluent key `tip-volume-locked` (plan 4 uses it for the fader's tooltip).

- [ ] **Step 1: Write the failing tests**

```rust
fn set_volumes(out: &[EngineAction], p: PlayerId) -> Vec<f32> {
    out.iter()
        .filter_map(|a| match a {
            EngineAction::SetVolume { player, volume } if *player == p => Some(*volume),
            _ => None,
        })
        .collect()
}

/// `fixture(3)` with P1 at half volume, not placed yet.
fn half() -> (AppState, PlayerId) {
    let mut s = fixture(3);
    let p = p0(&s);
    apply(&mut s, Command::SetVolume(p, 0.5)).unwrap();
    (s, p)
}

#[test]
fn bf2_placed_on_a_bit_perfect_device_the_fader_goes_to_full() {
    let (mut s, p) = half();
    let out = place_on_dac(&mut s, Holder::PlayerMain(p));
    assert_eq!(s.player(p).unwrap().volume, 1.0);
    assert_eq!(set_volumes(&out, p), vec![1.0]);
}

#[test]
fn bf2_bit_perfect_turned_on_for_its_device_resets_the_fader() {
    let (mut s, p) = half();
    // P1 on dac, which runs shared: not locked yet.
    let dac = bp_device();
    let shared = fp_model::device_settings(&s.config.outputs, &dac);
    on_event(
        &mut s,
        EngineEvent::Placed {
            holder: Holder::PlayerMain(p),
            route: None,
            device: dac.clone(),
            running: shared,
        },
    );
    assert_eq!(s.player(p).unwrap().volume, 0.5);
    let on = DeviceSettings {
        bit_perfect: true,
        ..shared
    };
    let out = on_event(
        &mut s,
        EngineEvent::Applied {
            target: Target::Device(dac),
            wanted: Wanted::Device(on),
            outcome: Ok(()),
        },
    );
    assert_eq!(s.player(p).unwrap().volume, 1.0);
    assert_eq!(set_volumes(&out, p), vec![1.0]);
}

#[test]
fn bf2_a_restored_lower_volume_goes_to_full_when_placed() {
    let (s, p) = half();
    let mut s = common::roundtrip(&s);
    assert_eq!(s.player(p).unwrap().volume, 0.5, "restored as saved");
    let out = place_on_dac(&mut s, Holder::PlayerMain(p));
    assert_eq!(set_volumes(&out, p), vec![1.0]);
}

#[test]
fn bf2_the_reset_is_sent_once() {
    let (mut s, p) = half();
    place_on_dac(&mut s, Holder::PlayerMain(p));
    let out = report_stream_rate(&mut s, 44_100, vec![]);
    assert!(set_volumes(&out, p).is_empty(), "already at 100 %");
}

#[test]
fn bf3_a_locked_fader_refuses_any_level_but_full() {
    let (mut s, p, _) = on_air();
    let before = s.clone();
    assert_eq!(
        apply(&mut s, Command::SetVolume(p, 0.5)),
        Err(fp_model::ModelError::FaderLocked(p))
    );
    assert_eq!(s, before, "a refused command changes nothing");
    let out = apply(&mut s, Command::SetVolume(p, 1.0)).unwrap();
    assert!(set_volumes(&out, p).is_empty(), "accepted, nothing to change");
    assert!(apply(&mut s, Command::SetVolume(p, f32::NAN)).is_ok(), "ignored as today");
    assert_eq!(
        fp_model::ModelError::FaderLocked(p).to_string(),
        format!("the volume of player {} is held at 100 % on a bit-perfect output", p.0)
    );
}

#[test]
fn bf3_availability_says_the_fader_is_locked() {
    let (s, p, _) = on_air();
    assert!(!fp_model::availability(&s, p).fader);
    assert!(!fp_model::command_available(&s, &Command::SetVolume(p, 0.5)));
    assert!(fp_model::command_available(&s, &Command::SetVolume(p, 1.0)));
    let q = s.players[1].id;
    assert!(fp_model::availability(&s, q).fader);
}

#[test]
fn bf4_unlocking_keeps_full_volume_and_frees_the_fader() {
    let (mut s, p, _) = on_air();
    let off = DeviceSettings {
        bit_perfect: false,
        ..*s.live.devices.get(&bp_device()).unwrap()
    };
    let out = on_event(
        &mut s,
        EngineEvent::Applied {
            target: Target::Device(bp_device()),
            wanted: Wanted::Device(off),
            outcome: Ok(()),
        },
    );
    assert!(set_volumes(&out, p).is_empty(), "nothing moves it");
    assert_eq!(s.player(p).unwrap().volume, 1.0);
    let out = apply(&mut s, Command::SetVolume(p, 0.5)).unwrap();
    assert_eq!(set_volumes(&out, p), vec![0.5]);
}

#[test]
fn bf5_fade_stop_does_not_touch_the_volume() {
    let (mut s, p, _) = on_air();
    let out = apply(&mut s, Command::FadeStop(p)).unwrap();
    assert!(out.iter().any(|a| matches!(a, EngineAction::FadeOutAndStop { .. })));
    assert!(set_volumes(&out, p).is_empty());
    assert_eq!(s.player(p).unwrap().volume, 1.0);
}

#[test]
fn bf6_a_cue_route_on_the_device_locks_nothing_and_cue_plays() {
    let (mut s, p) = half();
    let out = place_on_dac(&mut s, Holder::PlayerCue(p));
    assert!(set_volumes(&out, p).is_empty());
    assert!(!fader_locked(&s, p));
    let e = entries(&s);
    let out = apply(&mut s, Command::CueEntry(p, e[1])).unwrap();
    assert!(out.iter().any(|a| matches!(a, EngineAction::StartCue { .. })), "{out:?}");
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p fp-model --test bit_perfect_rules bf 2>&1 | tail -8`
Expected: FAIL to compile ("no variant `FaderLocked`", "no field `fader`").

- [ ] **Step 3: Implement**

`error.rs`:

```rust
    #[error("the volume of player {} is held at 100 % on a bit-perfect output", .0.0)]
    FaderLocked(PlayerId),
```

`reducer.rs`, the `SetVolume` arm:

```rust
        Command::SetVolume(id, volume) => {
            let i = state.player_index(id)?;
            // A broken value never silences what is on air: it is ignored.
            if volume.is_nan() {
                return Ok(out);
            }
            // BF3: a bit-perfect player's fader is held at exactly 100 %.
            if crate::bit_perfect::fader_locked(state, id) {
                #[allow(clippy::float_cmp)] // exactly unity is the only level allowed
                if volume != 1.0 {
                    return Err(ModelError::FaderLocked(id));
                }
                return Ok(out);
            }
            let volume = volume.clamp(0.0, 1.0);
            // …the rest unchanged (O25: unreachable from the fader now, kept
            // as a guard, bit-perfect rate and fader spec §5.5).
```

Keep the `LeaveDsd` lines as they are and add the §5.5 note to their comment.

`reconcile`, first thing after the doc comment:

```rust
    // BF2: a locked fader is at exactly 100 % after every command and event.
    let locked: Vec<usize> = state
        .players
        .iter()
        .enumerate()
        .filter(|(_, p)| p.volume != 1.0 && crate::bit_perfect::fader_locked(state, p.id))
        .map(|(i, _)| i)
        .collect();
    for i in locked {
        let player = &mut state.players[i];
        player.volume = 1.0;
        out.push(EngineAction::SetVolume {
            player: player.id,
            volume: 1.0,
        });
    }
```

(`p.volume != 1.0` needs `#[allow(clippy::float_cmp)]` on the statement if clippy flags it.)

`availability.rs`:

```rust
    /// The volume fader can move: false while it is locked at 100 % on a
    /// bit-perfect output (bit-perfect rate and fader spec BF3).
    pub fader: bool,
```

in `availability`: `fader: !crate::bit_perfect::fader_locked(state, player),`; in `command_available`:

```rust
        // BF3: a locked fader takes only exactly 100 % (a NaN is ignored).
        #[allow(clippy::float_cmp)]
        Command::SetVolume(id, v) => a(id).fader || *v == 1.0 || v.is_nan(),
```

`crates/fp-model/tests/availability.rs`: add `fader: true,` to the two `Availability { … }` literals (L28-33, L43-46).

`crates/fp-app/src/ui/app.rs`, `error_text`:

```rust
        ModelError::FaderLocked(_) => i18n.tr("tip-volume-locked"),
```

Locales — add after `tip-volume` in each file (text from spec §6.1, verbatim):

| File | Line |
|---|---|
| `en-US` | `tip-volume-locked = Volume held at 100 %: this player's output is bit-perfect.` |
| `es-ES` | `tip-volume-locked = Volumen fijo al 100 %: la salida de este reproductor es bit-perfect.` |
| `ca-ES` | `tip-volume-locked = Volum fix al 100 %: la sortida d'aquest reproductor és bit-perfect.` |
| `de-DE` | `tip-volume-locked = Lautstärke fest auf 100 %: Der Ausgang dieses Players ist bit-perfect.` |
| `eu-ES` | `tip-volume-locked = Bolumena % 100ean finkatuta: erreproduzigailu honen irteera bit-perfect da.` |
| `fr-FR` | `tip-volume-locked = Volume bloqué à 100 % : la sortie de ce lecteur est bit-perfect.` |
| `gl-ES` | `tip-volume-locked = Volume fixo ao 100 %: a saída deste reprodutor é bit-perfect.` |
| `it-IT` | `tip-volume-locked = Volume bloccato al 100 %: l'uscita di questo lettore è bit-perfect.` |
| `nl-NL` | `tip-volume-locked = Volume vast op 100 %: de uitgang van deze speler is bit-perfect.` |
| `pl-PL` | `tip-volume-locked = Głośność zablokowana na 100 %: wyjście tego odtwarzacza jest bit-perfect.` |
| `pt-PT` | `tip-volume-locked = Volume fixo a 100 %: a saída deste leitor é bit-perfect.` |

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p fp-model && cargo test -p fp-app --test i18n`
Expected: PASS (`d17_a_fader_move_below_unity_leaves_dsd` in `dsd_rules.rs` still passes: its player is not placed, so not locked — the §5.5 guard keeps its test).

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-model crates/fp-app
git commit -m "feat(model): hold a bit-perfect player's fader at 100 %

A locked fader goes to 100 % after any command or event that locks it,
refuses every other level with a reason, and is free again at 100 % once
unlocked; fades and CUE are not the fader (bit-perfect rate and fader
spec BF1-BF6).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 6: The faders Apply now would reset (BF8, model half)

**Files:**
- Modify: `crates/fp-model/src/bit_perfect.rs`, `crates/fp-model/src/lib.rs` (re-export)
- Test: `crates/fp-model/tests/bit_perfect_rules.rs`

**Interfaces:**
- Consumes: `fp_model::live::{pending, PendingItem, device_settings}`.
- Produces: `pub fn faders_reset_by(state: &AppState) -> Vec<PlayerId>` (plan 4 shows it in the Apply now confirmation).

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn bf8_apply_now_lists_the_faders_it_would_reset() {
    let mut s = fixture(3);
    let (p, q, r) = (s.players[0].id, s.players[1].id, s.players[2].id);
    // P1 at 50 % on dac, which runs shared and is wanted bit-perfect.
    let dac = bp_device();
    let shared = fp_model::device_settings(&s.config.outputs, &dac);
    for holder in [Holder::PlayerMain(p), Holder::PlayerMain(r)] {
        on_event(
            &mut s,
            EngineEvent::Placed {
                holder,
                route: Some(common::bp_route()),
                device: dac.clone(),
                running: shared,
            },
        );
    }
    for player in [p, r] {
        s.config
            .outputs
            .routes
            .iter_mut()
            .find(|x| x.player == player)
            .unwrap()
            .main = Some(common::bp_route());
    }
    apply(&mut s, Command::SetVolume(p, 0.5)).unwrap();
    s.config.outputs.bit_perfect = vec![dac.clone()];
    // Q at 70 % elsewhere, with a pending route to dac.
    apply(&mut s, Command::SetVolume(q, 0.7)).unwrap();
    s.config
        .outputs
        .routes
        .iter_mut()
        .find(|x| x.player == q)
        .unwrap()
        .main = Some(common::bp_route());
    let speakers = fp_model::OutputDevice {
        backend: "null".into(),
        device: "speakers".into(),
    };
    let running = fp_model::device_settings(&s.config.outputs, &speakers);
    on_event(
        &mut s,
        EngineEvent::Placed {
            holder: Holder::PlayerMain(q),
            route: None,
            device: speakers,
            running,
        },
    );
    // R is on dac at 100 %: nothing to reset.
    assert_eq!(fp_model::faders_reset_by(&s), vec![p, q]);
}

#[test]
fn bf8_a_locked_player_is_not_listed() {
    let (s, _, _) = on_air();
    assert!(fp_model::faders_reset_by(&s).is_empty());
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p fp-model --test bit_perfect_rules bf8 2>&1 | tail -5`
Expected: FAIL to compile ("cannot find function `faders_reset_by`").

- [ ] **Step 3: Implement**

`bit_perfect.rs` (extend the imports with `crate::live::{PendingItem, device_settings, pending}`):

```rust
/// BF8: the players whose fader Apply now (live settings L20) would lock
/// while it is below 100 %. Apply now forces every pending output item, so
/// afterwards each device runs its configured settings; a player's Main
/// is then on the `to` of its pending route, or where it is placed now. A
/// pending route to the default output is not resolved here: the engine
/// chooses that device (BF2 still resets the fader once it is placed).
pub fn faders_reset_by(state: &AppState) -> Vec<PlayerId> {
    let items = pending(state);
    state
        .players
        .iter()
        .filter(|p| p.volume < 1.0 && !fader_locked(state, p.id))
        .filter(|p| {
            let holder = Holder::PlayerMain(p.id);
            let moved = items.iter().find_map(|item| match &item.item {
                PendingItem::Route { holder: h, to, .. } if *h == holder => Some(to.clone()),
                _ => None,
            });
            let device = match moved {
                Some(Some(route)) => Some(OutputDevice {
                    backend: route.backend,
                    device: route.device,
                }),
                Some(None) => None,
                None => state.live.placement.get(&holder).cloned(),
            };
            device.is_some_and(|d| device_settings(&state.config.outputs, &d).bit_perfect)
        })
        .map(|p| p.id)
        .collect()
}
```

`lib.rs`: add `faders_reset_by` to the `bit_perfect` re-export.

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p fp-model --test bit_perfect_rules bf8`
Expected: PASS.

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-model
git commit -m "feat(model): list the faders Apply now would reset

The players whose Main would land on a bit-perfect device once every
pending output change is forced, with their volume below 100 %, so the
confirmation can name them (bit-perfect rate and fader spec BF8).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 7: The specs and the architecture page say the same (§9.3)

**Files:**
- Modify: `docs/superpowers/specs/2026-09-26-phase4-bit-perfect-design.md` — B3.3: a first line "*Replaced by the [bit-perfect rate and fader spec](2026-10-08-bit-perfect-rate-and-fader-design.md) (2026-10-08) §4: a bit-perfect player's transitions are hard, and the device reopens at the next track's rate between them when nothing else sounds; the rest of this rule stays as BR11.*"; B1 "Unaltered source": a note that the fader of a bit-perfect player is held at 100 % (BF1–BF3); B7: after "Automatic volume bypass (the operator sets 100 %)." add "*(Now in scope as the locked fader, bit-perfect rate and fader spec §5.)*" and after "Changing the rate while a bus is busy." add "*(Now in scope as the rate cut between two tracks of a hard transition, §4.)*".
- Modify: `docs/superpowers/specs/2026-09-25-fauste-player-design.md` §3.2 — at the end of rule 5 (L151 onwards), of rule 11 (L169 onwards) and of R24 (L194): "Except on a bit-perfect player: a hard transition ([bit-perfect rate and fader spec](2026-10-08-bit-perfect-rate-and-fader-design.md), D8)."
- Modify: `docs/superpowers/specs/2026-10-07-live-settings-design.md` L395: "the bit-perfect rate change at transitions" gains "(specified in the [bit-perfect rate and fader spec](2026-10-08-bit-perfect-rate-and-fader-design.md))".
- Modify: `docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md` O25 (the `LeaveDsd` bullet, ~L573): "*(Bit-perfect rate and fader spec §5.5: a DSD stream plays only on a bit-perfect device, whose fader is locked at 100 %, so the fader can no longer reach this rule; it stays as a guard with its test.)*"
- Modify: `docs/technical/architecture.md` L7, the `fp-model` row: "player rules (spec §3)" becomes "player rules (spec §3, with the bit-perfect players' hard transitions and locked fader in `bit_perfect`)".
- Modify: the spec under implementation, its header's **Status** line (L5, "Draft, for the maintainer's review (§12)."): "Approved (2026-10-13); implemented by the plans `2026-10-13-bit-perfect-plan1` to `plan4`."

- [ ] **Step 1: Edit the documents** as listed. Check every link with `scripts/site/check-links.sh` only if it runs without the site build; otherwise check each relative path by hand (`ls docs/superpowers/specs/<name>`).
- [ ] **Step 2: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add docs
git commit -m "docs(spec): point the amended rules to the bit-perfect rate and fader spec

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

## Self-review notes

- Spec coverage (this plan): BR1 model half → Task 1; BR2, BR3, BF1 → Task 2; BR4, BR5, BR10, BR12, BR14 (model half) → Task 3; BR7, BR9 → Task 4; BF2–BF6, §5.5 (the guard and its test stay), §6.1 `tip-volume-locked` → Task 5; BF8 `faders_reset_by` → Task 6; §9.3 → Task 7. Left to plan 2: BR1 engine half, BR6, BR8, BR11, BR13, BR14 engine half; plan 3: BF10–BF13, BR16; plan 4: BR15, BF7, BF8 interface, BF9, §6.2–§6.5, `tip-volume-dsd` removal, §8 loopback steps, §9.1, §9.2 (except `audio-engine.md` and `audio-path-audit.md`, plan 2).
- After this plan the engine still ignores `rate_cut`: a bit-perfect player's transitions are hard (the engine's existing hard join) and the device keeps its rate between tracks until plan 2. The fader is refused by the model but the on-screen fader is still enabled until plan 4: a drag shows the `tip-volume-locked` notice.
