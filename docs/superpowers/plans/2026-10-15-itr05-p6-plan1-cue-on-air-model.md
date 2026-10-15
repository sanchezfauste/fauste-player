# Pre-listen on an On-air Output, Plan 1: the Model

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `fp-model` decides everything the on-air pre-listen amendment changes: a Cue route that overlaps a Main route is usable (CA1); a pre-listen on an on-air output starts at once while the output is quiet (CA2), records a confirmation request while it sounds (CA3), starts anyway when forced and is then confirmed (CA4), and is stopped with a notice when something starts there unless it is confirmed (CA5); remote and MIDI get a pure predicate to refuse what they cannot confirm (CA6); bit-perfect rate decisions already count the on-air pre-listen (CA7).

**Architecture:** A new pure module `fp_model::cue_on_air` holds the spec's terms (`holder_output`, `outputs_overlap`, `cue_is_on_air`, `sounding_on`), the predicate `needs_confirmation` / `cue_start`, and the runtime state `CueOnAir { request, notice, confirmed }` kept in `AppState::cue_on_air` (never saved). `Command` gains `ForceCue(CueStart)` and `CancelCueOnAir(u64)`. The reducer asks (CA3) in `cue_entry` and `cart_rules::cue` when a pre-listen starts from nothing; `apply` and `on_event` become thin wrappers that run `cue_on_air::before_step` / `after_step` around the old bodies: `after_step` stops unconfirmed pre-listens a Main holder started on (CA5, the `StopCue` first in the actions), keeps the confirmed list and the request up to date. The engine still drops a Cue route equal to Main until plan 2, and the interface shows nothing of the request until plan 3.

**Tech Stack:** Rust 2024, `fp-model` (pure, no I/O).

**Spec:** `docs/superpowers/specs/2026-10-15-cue-on-air-output-design.md` (§1–§3, §7 `fp-model`). It amends the main spec §4.6 and rule 26 (`docs/superpowers/specs/2026-09-25-fauste-player-design.md`), the cartwall spec C9 and the remote control spec: read those parts too.

This is plan 1 of 3, executed strictly in order on branch `feat/cue-on-air-itr05`:
1. **this plan** — the model;
2. `2026-10-15-itr05-p6-plan2-cue-on-air-engine.md` — CA8 and the engine side of CA7;
3. `2026-10-15-itr05-p6-plan3-cue-on-air-ui-remote-midi.md` — the popup, the notice, the strings, remote, MIDI, docs.

Between plan 1 and plan 2 a Cue route equal to Main is accepted by the model but the engine still ends such a CUE at once (`CueEnded`); nothing ever reaches the air. Between plan 1 and plan 3 a CA3 request is recorded but not shown: the CUE button does nothing on a sounding on-air output.

## Rulings (planning)

**Ruling: `force` is a command, `Command::ForceCue(CueStart)`, not a field on five variants.** `CueStart` is `Entry { player, entry }` or `Cart(CartId)`: what a pre-listen plays. The confirmation request stores it, and *Pre-listen anyway* sends it back. — Why: only a start can be forced (a stop needs no force), one variant replaces a `force` field on `ToggleCue`, `SetCue`, `CueEntry`, `CueCart` and `SetCartCue` and their ~140 call sites, and a request for `ToggleCue` must remember the entry it was about (the next may change before the answer). — Cost if wrong: a mechanical rename to `{ …, force: bool }` fields.

**Ruling: a holder's output uses the route the engine holds (`live.routes`), not the configured one.** §2 says "the channel pair of its configured route"; the device comes from `live.placement`, and pairing it with a configured route that a pending change has moved elsewhere would describe a place nothing plays. The configured route is used only when the engine has not reported one. A route whose backend or device is not the placement's (a missing backend falls back to the default output) plays on channels 1–2, as the engine does. — Cost if wrong: one `match` in `holder_output`.

**Ruling: CA5 counts Main holders only, and "starts" means "becomes audible" (`live::causes` goes from empty to non-empty) on the CUE's output.** The list in CA5 (play, next, automatic transition, cart fire, restart, a paused player resuming) is all Main holders, and the notice names "the holder that started". A paused player counts as *sounding* for CA3 (§2) but is not audible, so its resume is a start. Another pre-listen forced onto the same output does not stop an unconfirmed one. — Cost if wrong: one filter (`is_main`) in `audible_mains`.

**Ruling: CA5 compares per pre-listen, before and after each step.** For each running unconfirmed pre-listen, `before_step` keeps the Main holders audible on its output; `after_step` stops it when a new one appears. A Main the engine places only after it starts (the cartwall's first cart: `Placed { CartwallMain }` arrives in a later `on_event`) is caught at that event. — Cost if wrong: none known; the cheaper "holders audible anywhere" snapshot would miss that case.

**Ruling: CA3 is decided when a pre-listen starts from nothing.** A running pre-listen that moves to another entry (`CueEntry` on a running CUE, rule O17's follow) or to another cart keeps running and keeps its confirmation: the operator already hears it on that output. — Cost if wrong: one more `ask` in `cue_entry` and `cart_rules::cue`, and a popup on every row click while a confirmed CUE runs.

**Ruling: `CancelCueOnAir` carries the request's id; `ForceCue` drops a request for the same holder.** A Cancel that arrives after a newer request replaced the old one must not drop the newer one. — Cost if wrong: none.

**Ruling: a remote or MIDI start planned from a quiet snapshot that reaches the conductor while the output sounds records a request (the console shows it).** The remote API and MIDI refuse up front with `needs_confirmation` on their snapshot (plan 3); the model has no notion of who sent a command. — Why: nothing goes on air; the operator at the console answers. — Cost if wrong: a `from_remote` flag on the commands.

## Global Constraints

- All code, identifiers, comments, docs and commit messages in English; never mention other playout or radio-automation products.
- Behaviour lives in `fp-model` as pure functions, **one test per spec rule** (CLAUDE.md rule 7, spec §7); test names start with the rule (`ca1_…`, `ca5_…`), the §2 terms with `ca_terms_…`.
- §2 verbatim: "Two outputs **overlap** when they are on the same device and their channel pairs share at least one channel." "A Cue holder is **on air** when its output overlaps the output of any Main holder (`PlayerMain` of any player, or `CartwallMain`)." "An on-air output **sounds** when any holder whose output overlaps it has a cause to be busy (`live::causes`, non-empty) or is a player paused with a current track (the definition BR3.5 already uses). The Cue holder being decided is left out."
- CA1: "a missing Cue route, or one to a backend this machine does not have, still means no CUE".
- CA3: "The request is runtime only and is dropped when the output stops sounding, the operator answers, or another CUE request replaces it."
- CA5: "the same step stops that CUE and records a **notice** naming the holder that started. A confirmed CUE keeps playing."
- Nothing goes on air by itself (CLAUDE.md rule 10); `unwrap`, `expect`, `panic` denied outside tests.
- Conventional Commits, scope `model`; every commit ends with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- `cargo` is at `$HOME/.cargo/bin` (`export PATH="$HOME/.cargo/bin:$PATH"`). Gate for every task: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`.
- Progress ledger: `.superpowers/sdd/itr05/progress.md` (section P6, plan 1). Record deviations as `Ruling: <decision> — <why> — <cost if wrong>`.

## Review Focus

1. **The cartwall's first cart** while a pre-listen plays unconfirmed on the device the cartwall's Main will open on: the fire itself cannot stop it (the cartwall is not placed yet); the `Placed { CartwallMain }` that follows must. Test: Task 4, `ca5_the_cartwalls_first_cart_stops_it_once_placed`.
2. **A Main route left empty** (the default output) that the engine resolves to the Cue's device: the Cue is on air on channels 1–2 and not on 3–4. Test: Task 2, `ca_terms_a_missing_main_route_plays_on_the_first_pair`.
3. **A stale Cancel**: the operator cancels while a newer request has replaced the one shown; the newer request must stay. Test: Task 3, `ca3_a_cancel_answers_only_its_own_request`.
4. **A request whose player is removed** (`SetPlayerCount` lower) before the operator answers: the request goes, nothing starts. Test: Task 3, `ca3_a_request_goes_with_its_player`.
5. **A confirmed pre-listen stopped and started again**: the second start is a new decision, not still confirmed (the confirmed list follows the CUE's life). Test: Task 4, `ca5_a_confirmation_ends_with_its_cue`.

---

## File Structure

| File | Responsibility |
|---|---|
| `crates/fp-model/src/cue_on_air.rs` (new) | §2 terms, CA3/CA6 predicates, `CueOnAir` state, `ask`, `forced`, `cancel`, `before_step`, `after_step` |
| `crates/fp-model/src/lib.rs` | `pub mod cue_on_air;`, re-exports |
| `crates/fp-model/src/command.rs` | `CueStart`; `Command::ForceCue`, `Command::CancelCueOnAir` |
| `crates/fp-model/src/state.rs`, `crates/fp-model/src/session.rs` (L173-181) | `AppState::cue_on_air` |
| `crates/fp-model/src/config.rs` (`cue_is_usable` L687, `player_has_cue` L499, `cartwall_has_cue` L511) | CA1; `routes_overlap`, `OutputsConfig::cue_shares_main` |
| `crates/fp-model/src/availability.rs` | comments only (CA1) |
| `crates/fp-model/src/reducer.rs` (`apply` L22, `on_event` L389, `toggle_cue` L982, `cue_entry` L995, `follow_cue` L1020) | wrappers, CA3, CA4 |
| `crates/fp-model/src/cart_rules.rs` (`request` L14, `cue` L99) | CA3/CA4 for carts, `cue_request` |
| `crates/fp-model/tests/cue_on_air.rs` (new) | CA1–CA7 |
| `crates/fp-model/tests/cue_output.rs` | rewritten for CA1 |
| `crates/fp-remote/tests/api.rs` (L359-381), `crates/fp-app/tests/cue_output_ui.rs`, `crates/fp-app/tests/cartwall_ui.rs` (L194-210), `crates/fp-app/tests/view.rs` (L600-613) | tests that used "Cue = Main" to mean "no CUE" now use "no Cue route" |
| `docs/technical/architecture.md`, `docs/superpowers/specs/2026-09-25-fauste-player-design.md`, `docs/superpowers/specs/2026-09-26-phase2-cartwall-settings-design.md`, `docs/superpowers/specs/2026-10-01-remote-control-design.md` | Task 5 |

---

### Task 1: A Cue route on the Main output is usable (CA1)

**Files:**
- Modify: `crates/fp-model/src/config.rs` (L497-519, L680-689)
- Modify: `crates/fp-model/src/availability.rs` (L49-51, L56-60)
- Modify: `crates/fp-model/tests/cue_output.rs` (whole file)
- Modify: `crates/fp-remote/tests/api.rs` (L358-381), `crates/fp-app/tests/cue_output_ui.rs`, `crates/fp-app/tests/cartwall_ui.rs` (L194-210), `crates/fp-app/tests/view.rs` (L600-613)

**Interfaces:**
- Produces: `fp_model::config::cue_is_usable(cue: Option<&Route>) -> bool` (one argument now); `fp_model::config::routes_overlap(a: &Route, b: &Route) -> bool`; `OutputsConfig::cue_shares_main(&self, cue: Holder) -> bool`. `player_has_cue` / `cartwall_has_cue` keep their signatures. `cue_equals_main`, `player_cue_equals_main`, `cartwall_cue_equals_main` stay until plan 3 removes them with the Settings note.

- [ ] **Step 1: Rewrite `crates/fp-model/tests/cue_output.rs` for CA1**

Replace the whole file with:

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Spec §4.6, rule 26 and C9 as amended by the on-air pre-listen spec CA1:
//! a CUE needs a Cue route; one that overlaps a Main route is usable.

mod common;

use std::path::PathBuf;

use common::{entries, fixture, p0};
use fp_model::{
    AppState, CartwallRoutes, Command, EngineAction, PlayerId, PlayerRoutes, Route, TrackAnalysis,
    apply, availability, command_available,
    config::{cue_equals_main, cue_is_usable, routes_overlap},
};
use fp_model::Holder;

fn route(device: &str, first_channel: u16) -> Route {
    Route {
        backend: "cpal".to_owned(),
        device: device.to_owned(),
        first_channel,
    }
}

fn set_routes(state: &mut AppState, player: PlayerId, main: Option<Route>, cue: Option<Route>) {
    state.config.outputs.routes = vec![PlayerRoutes { player, main, cue }];
}

/// A fixture whose first player has `main` and `cue` as its routes.
fn with_routes(main: Option<Route>, cue: Option<Route>) -> (AppState, PlayerId) {
    let mut s = fixture(3);
    let p = p0(&s);
    set_routes(&mut s, p, main, cue);
    (s, p)
}

fn no_cue_route() -> (AppState, PlayerId) {
    with_routes(Some(route("desk", 0)), None)
}

#[test]
fn ca1_any_cue_route_is_usable_even_the_main_one() {
    for main in [None, Some(route("desk", 0)), Some(route("phones", 0))] {
        assert!(cue_is_usable(Some(&route("desk", 0))), "{main:?}");
    }
    assert!(!cue_is_usable(None));
}

#[test]
fn equal_routes_are_still_recognised_until_settings_says_otherwise() {
    assert!(cue_equals_main(
        Some(&route("desk", 0)),
        Some(&route("desk", 0))
    ));
    assert!(!cue_equals_main(None, Some(&route("desk", 0))));
    assert!(!cue_equals_main(Some(&route("desk", 0)), None));
}

#[test]
fn ca_terms_routes_overlap_on_a_shared_channel_of_one_device() {
    assert!(routes_overlap(&route("desk", 0), &route("desk", 0)));
    assert!(routes_overlap(&route("desk", 0), &route("desk", 1)));
    assert!(routes_overlap(&route("desk", 3), &route("desk", 2)));
    assert!(!routes_overlap(&route("desk", 0), &route("desk", 2)));
    assert!(!routes_overlap(&route("desk", 0), &route("phones", 0)));
    let mut other = route("desk", 0);
    other.backend = "jack".to_owned();
    assert!(!routes_overlap(&route("desk", 0), &other));
}

#[test]
fn ca1_cue_shares_main_looks_at_every_main_route() {
    let mut s = fixture(3);
    let (p, q) = (s.players[0].id, s.players[1].id);
    s.config.outputs.routes = vec![
        PlayerRoutes {
            player: p,
            main: Some(route("speakers", 0)),
            cue: Some(route("desk", 0)),
        },
        PlayerRoutes {
            player: q,
            main: Some(route("desk", 1)),
            cue: Some(route("phones", 0)),
        },
    ];
    let outputs = &s.config.outputs;
    assert!(outputs.cue_shares_main(Holder::PlayerCue(p)), "Q's Main");
    assert!(!outputs.cue_shares_main(Holder::PlayerCue(q)));
    assert!(!outputs.cue_shares_main(Holder::PlayerMain(p)), "not a Cue");
    s.config.outputs.cartwall = CartwallRoutes {
        main: Some(route("phones", 0)),
        cue: Some(route("speakers", 1)),
    };
    let outputs = &s.config.outputs;
    assert!(outputs.cue_shares_main(Holder::PlayerCue(q)), "the cartwall's Main");
    assert!(outputs.cue_shares_main(Holder::CartwallCue), "P's Main");
    s.config.outputs.cartwall.cue = None;
    assert!(!s.config.outputs.cue_shares_main(Holder::CartwallCue));
}

#[test]
fn ca1_outputs_config_answers_per_player_and_for_the_cartwall() {
    let (mut s, p) = with_routes(Some(route("desk", 0)), Some(route("desk", 0)));
    let other = s.players[1].id;
    assert!(s.config.outputs.player_has_cue(p), "Cue on Main is usable");
    assert!(!s.config.outputs.player_has_cue(other), "no routes at all");
    set_routes(&mut s, p, Some(route("desk", 0)), None);
    assert!(!s.config.outputs.player_has_cue(p));

    s.config.outputs.cartwall = CartwallRoutes::default();
    assert!(!s.config.outputs.cartwall_has_cue());
    s.config.outputs.cartwall = CartwallRoutes {
        main: Some(route("desk", 2)),
        cue: Some(route("desk", 2)),
    };
    assert!(s.config.outputs.cartwall_has_cue());
}

#[test]
fn ca1_rule_26_cue_is_available_with_a_cue_route_on_main() {
    let (s, p) = with_routes(Some(route("desk", 0)), Some(route("desk", 0)));
    assert!(availability(&s, p).cue);
    assert!(command_available(&s, &Command::ToggleCue(p)));
}

#[test]
fn rule_26_cue_is_unavailable_without_a_cue_route() {
    let (s, p) = no_cue_route();
    assert!(!availability(&s, p).cue);
}

#[test]
fn ca1_a_cue_on_the_main_route_starts_while_nothing_is_placed() {
    let (mut s, p) = with_routes(Some(route("desk", 0)), Some(route("desk", 0)));
    let actions = apply(&mut s, Command::ToggleCue(p)).unwrap();
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EngineAction::StartCue { .. })),
        "{actions:?}"
    );
    assert!(s.player(p).unwrap().cue.is_some());
}

#[test]
fn toggle_set_and_cue_entry_do_nothing_without_a_cue_route() {
    let (mut s, p) = no_cue_route();
    let e = entries(&s)[1];
    for command in [
        Command::ToggleCue(p),
        Command::SetCue(p, true),
        Command::CueEntry(p, e),
    ] {
        let actions = apply(&mut s, command.clone()).unwrap();
        assert!(actions.is_empty(), "{command:?}: {actions:?}");
        assert!(s.player(p).unwrap().cue.is_none(), "{command:?}");
    }
}

#[test]
fn a_running_cue_still_stops_after_its_cue_route_was_removed() {
    let (mut s, p) = with_routes(Some(route("desk", 0)), Some(route("phones", 0)));
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    set_routes(&mut s, p, Some(route("desk", 0)), None);
    assert!(availability(&s, p).cue, "a running cue can be stopped");
    let actions = apply(&mut s, Command::ToggleCue(p)).unwrap();
    assert_eq!(actions, vec![EngineAction::StopCue { player: p }]);
}

fn cart_with_file(state: &mut AppState) -> fp_model::CartId {
    let page = state.cartwall.pages[0].id;
    apply(
        state,
        Command::AssignCartFile {
            page,
            index: 0,
            path: PathBuf::from("/carts/jingle.wav"),
        },
    )
    .unwrap();
    let track = state.cartwall.pages[0].carts[0].track.unwrap();
    apply(
        state,
        Command::ApplyAnalysis {
            track,
            analysis: Box::new(TrackAnalysis {
                duration_secs: 5.0,
                ..TrackAnalysis::default()
            }),
        },
    )
    .unwrap();
    state.cartwall.pages[0].carts[0].id
}

#[test]
fn a_cart_cue_does_nothing_without_a_cartwall_cue_route() {
    let mut s = fixture(0);
    let cart = cart_with_file(&mut s);
    s.config.outputs.cartwall = CartwallRoutes {
        main: Some(route("desk", 0)),
        cue: None,
    };
    for command in [Command::CueCart(cart), Command::SetCartCue(cart, true)] {
        let actions = apply(&mut s, command).unwrap();
        assert!(actions.is_empty(), "{actions:?}");
        assert_eq!(s.cartwall.cue, None);
    }
}

#[test]
fn ca1_a_cart_cue_starts_on_the_cartwalls_main_route() {
    let mut s = fixture(0);
    let cart = cart_with_file(&mut s);
    s.config.outputs.cartwall = CartwallRoutes {
        main: Some(route("desk", 0)),
        cue: Some(route("desk", 0)),
    };
    let actions = apply(&mut s, Command::CueCart(cart)).unwrap();
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EngineAction::StartCartCue(_))),
        "{actions:?}"
    );
    assert_eq!(s.cartwall.cue, Some(cart));
}

#[test]
fn cue_entry_and_cart_cue_commands_are_unavailable_without_a_cue_route() {
    let (mut s, p) = no_cue_route();
    let e = entries(&s)[1];
    assert!(!command_available(&s, &Command::CueEntry(p, e)));
    set_routes(&mut s, p, Some(route("desk", 0)), Some(route("desk", 0)));
    assert!(command_available(&s, &Command::CueEntry(p, e)));

    let cart = cart_with_file(&mut s);
    s.config.outputs.cartwall = CartwallRoutes {
        main: Some(route("phones", 0)),
        cue: None,
    };
    assert!(!command_available(&s, &Command::CueCart(cart)));
    s.config.outputs.cartwall.cue = Some(route("phones", 0));
    assert!(command_available(&s, &Command::CueCart(cart)));
    apply(&mut s, Command::CueCart(cart)).unwrap();
    s.config.outputs.cartwall.cue = None;
    assert!(
        command_available(&s, &Command::CueCart(cart)),
        "a running cart cue can be stopped"
    );
}
```

- [ ] **Step 2: Run it to see it fail**

Run: `cargo test -p fp-model --test cue_output`
Expected: compile errors — `cue_is_usable` takes two arguments, `routes_overlap` and `cue_shares_main` do not exist.

- [ ] **Step 3: CA1 in `config.rs`**

In `crates/fp-model/src/config.rs`, replace `player_has_cue` and `cartwall_has_cue` (L497-513) bodies:

```rust
    /// Whether `player` has a Cue output a CUE can be heard on (see
    /// [`cue_is_usable`]).
    pub fn player_has_cue(&self, player: PlayerId) -> bool {
        self.player_routes(player)
            .is_some_and(|r| cue_is_usable(r.cue.as_ref()))
    }
```

```rust
    /// Whether the cartwall has a Cue output a cart pre-listen can be heard on.
    pub fn cartwall_has_cue(&self) -> bool {
        cue_is_usable(self.cartwall.cue.as_ref())
    }

    /// On-air pre-listen spec §5: whether `cue`'s configured route overlaps
    /// a configured Main route (any player's or the cartwall's). A missing
    /// Main route goes to the default output, which only the engine
    /// resolves: it is not counted here (`cue_on_air::cue_is_on_air` sees
    /// it once placed). False for a holder that is not a Cue.
    pub fn cue_shares_main(&self, cue: crate::live::Holder) -> bool {
        use crate::live::Holder;
        let route = match cue {
            Holder::PlayerCue(p) => self.player_routes(p).and_then(|r| r.cue.as_ref()),
            Holder::CartwallCue => self.cartwall.cue.as_ref(),
            Holder::PlayerMain(_) | Holder::CartwallMain => None,
        };
        let Some(route) = route else {
            return false;
        };
        self.routes
            .iter()
            .filter_map(|r| r.main.as_ref())
            .chain(self.cartwall.main.as_ref())
            .any(|main| routes_overlap(main, route))
    }
```

Replace `cue_is_usable` (L680-689) with:

```rust
/// Whether a Cue route can be heard (spec §4.6 as amended by the on-air
/// pre-listen spec CA1): there is one. A Cue route on the Main output is
/// usable; the model asks before a pre-listen goes out on a sounding
/// on-air output (`cue_on_air`). A route to a backend this machine does not
/// have is only known to the running engine, which drops it.
pub fn cue_is_usable(cue: Option<&Route>) -> bool {
    cue.is_some()
}

/// Whether two routes share a channel of one device (on-air pre-listen
/// spec §2): same backend and device, and channel pairs that start less
/// than two channels apart.
pub fn routes_overlap(a: &Route, b: &Route) -> bool {
    a.backend == b.backend
        && a.device == b.device
        && a.first_channel.abs_diff(b.first_channel) < 2
}
```

- [ ] **Step 4: Comments in `availability.rs`**

In `crates/fp-model/src/availability.rs`, replace the comment above `cue:` (L49-50) with:

```rust
        // A running CUE can always be stopped; a new one needs a Cue route
        // (spec §4.6, on-air pre-listen spec CA1).
```

and in the doc comment of `command_available` replace "and for a CUE of an entry or a cart pre-listen that has no Cue output apart from Main (spec §4.6)" with "and for a CUE of an entry or a cart pre-listen without a Cue route (spec §4.6, CA1)".

- [ ] **Step 5: Run the model test**

Run: `cargo test -p fp-model --test cue_output`
Expected: PASS.

- [ ] **Step 6: The other crates' tests that meant "no CUE" by "Cue = Main"**

`crates/fp-remote/tests/api.rs` L358-381, replace the test with:

```rust
/// Spec §4.6 and CA1: a CUE needs a Cue route; without one every way to
/// start a CUE is unavailable, as the console dims it.
#[test]
fn a_cue_without_a_cue_route_is_a_conflict() {
    let mut s = demo_state();
    let p = s.players[0].id;
    let e = s.playlists.iter().next().unwrap().entries[2].id;
    let c = s.cartwall.pages[0].carts[0].id;
    for r in &mut s.config.outputs.routes {
        r.cue = None;
    }
    s.config.outputs.cartwall.cue = None;
    for op in [
        O::SetCue(p, true),
        O::CueEntry(p, e),
        O::SetCartCue(c, true),
    ] {
        let err = plan(&s, op).unwrap_err();
        assert_eq!(err.code(), "unavailable", "{op:?}");
    }
    assert_eq!(plan(&s, O::SetCue(p, false)).unwrap(), vec![]);
    assert_eq!(plan(&s, O::SetCartCue(c, false)).unwrap(), vec![]);
}
```

`crates/fp-app/tests/cue_output_ui.rs`: change the module doc to `//! Spec §4.6, rule 26 and CA1: a player without a Cue route has no CUE, and the interface says why.`, and replace `cue_on_main` and its three uses with:

```rust
/// One player without a Cue route.
fn no_cue_route() -> AppState {
    let mut s = state(1, 3);
    for r in &mut s.config.outputs.routes {
        r.cue = None;
    }
    s
}
```

(`harness(cue_on_main())` becomes `harness(no_cue_route())` in `the_cue_button_is_dimmed_and_says_why` and `the_row_menu_pre_listen_is_dimmed_and_says_why`.)

`crates/fp-app/tests/cartwall_ui.rs` L196-197: replace `s.config.outputs.cartwall.main = s.config.outputs.cartwall.cue.clone();` with `s.config.outputs.cartwall.cue = None;`.

`crates/fp-app/tests/view.rs` L600-613: replace the test with:

```rust
#[test]
fn a_click_leaves_a_cue_whose_cue_route_was_removed() {
    let (mut s, e, p) = state(3);
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    let routes = s
        .config
        .outputs
        .routes
        .iter_mut()
        .find(|r| r.player == p)
        .unwrap();
    routes.cue = None;
    assert_eq!(cue_follow_target(&s, p, e[2]), None);
}
```

- [ ] **Step 7: Gate**

Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
Expected: all green. If another test fails because it built "no CUE" from equal routes, change it the same way (a missing Cue route) and note it in the ledger.

- [ ] **Step 8: Commit**

```bash
git add crates/fp-model/src/config.rs crates/fp-model/src/availability.rs crates/fp-model/tests/cue_output.rs crates/fp-remote/tests/api.rs crates/fp-app/tests/cue_output_ui.rs crates/fp-app/tests/cartwall_ui.rs crates/fp-app/tests/view.rs
git commit -m "feat(model)!: a Cue route on the Main output is usable

On-air pre-listen spec CA1: one sound card means Main and Cue on the
same output, and today that leaves no CUE at all. A Cue route is now
usable whatever Main is; the next commits ask before a pre-listen goes
out on a sounding on-air output.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: Where a holder plays, on air, and what sounds there (§2)

**Files:**
- Create: `crates/fp-model/src/cue_on_air.rs`
- Modify: `crates/fp-model/src/lib.rs`, `crates/fp-model/src/command.rs`, `crates/fp-model/src/state.rs`, `crates/fp-model/src/session.rs` (L173-181)
- Test: `crates/fp-model/tests/cue_on_air.rs` (new)

**Interfaces:**
- Consumes: `fp_model::config::routes_overlap` is not used here (the engine-reported outputs are compared, not routes).
- Produces:
  - `fp_model::CueStart` (`command.rs`): `enum CueStart { Entry { player: PlayerId, entry: EntryId }, Cart(CartId) }`, `Copy + Eq`, `fn holder(self) -> Holder`.
  - `fp_model::cue_on_air::HolderOutput { pub device: OutputDevice, pub first_channel: u16 }`.
  - `cue_on_air::holder_output(state: &AppState, holder: Holder) -> Option<HolderOutput>`.
  - `cue_on_air::outputs_overlap(a: &HolderOutput, b: &HolderOutput) -> bool`.
  - `cue_on_air::cue_is_on_air(state: &AppState, cue: Holder) -> bool`.
  - `cue_on_air::sounding_on(state: &AppState, cue: Holder) -> Vec<Holder>` (display order; empty when not on air).
  - `cue_on_air::{CueOnAir, CueConfirmation, CueStopped}` and `AppState::cue_on_air: CueOnAir` (Task 3 adds its private id counter).

- [ ] **Step 1: The test file with the §2 terms**

Create `crates/fp-model/tests/cue_on_air.rs`:

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! On-air pre-listen spec (2026-10-15): the §2 terms and CA1–CA7, one test
//! per rule.

mod common;

use common::fixture;
use fp_model::cue_on_air::{
    HolderOutput, cue_is_on_air, holder_output, outputs_overlap, sounding_on,
};
use fp_model::{
    AppState, Command, EngineAction, EngineEvent, Holder, OutputDevice, PlayerId, Route, apply,
    device_settings, on_event,
};

fn route(device: &str, first_channel: u16) -> Route {
    Route {
        backend: "null".to_owned(),
        device: device.to_owned(),
        first_channel,
    }
}

fn device(name: &str) -> OutputDevice {
    OutputDevice {
        backend: "null".to_owned(),
        device: name.to_owned(),
    }
}

/// Configures `route` for `holder` (`None`: the default output) and reports
/// the engine placing `holder` on device `on` with it (live settings L3).
fn place(s: &mut AppState, holder: Holder, route: Option<Route>, on: &str) -> Vec<EngineAction> {
    let outputs = &mut s.config.outputs;
    match holder {
        Holder::PlayerMain(p) | Holder::PlayerCue(p) => {
            let routes = outputs.routes.iter_mut().find(|r| r.player == p).unwrap();
            if matches!(holder, Holder::PlayerMain(_)) {
                routes.main = route.clone();
            } else {
                routes.cue = route.clone();
            }
        }
        Holder::CartwallMain => outputs.cartwall.main = route.clone(),
        Holder::CartwallCue => outputs.cartwall.cue = route.clone(),
    }
    let running = device_settings(&s.config.outputs, &device(on));
    on_event(
        s,
        EngineEvent::Placed {
            holder,
            route,
            device: device(on),
            running,
        },
    )
}

/// Places `holder` on `desk`, on the pair from `first_channel`.
fn on_desk(s: &mut AppState, holder: Holder, first_channel: u16) -> Vec<EngineAction> {
    place(s, holder, Some(route("desk", first_channel)), "desk")
}

/// Four players over four tracks. Player 1's Main is on `speakers`, its Cue
/// and Player 2's Main share `desk` 1–2. Nothing plays.
fn shared() -> (AppState, PlayerId, PlayerId) {
    let mut s = fixture(4);
    let (p1, p2) = (s.players[0].id, s.players[1].id);
    place(&mut s, Holder::PlayerMain(p1), Some(route("speakers", 0)), "speakers");
    on_desk(&mut s, Holder::PlayerCue(p1), 0);
    on_desk(&mut s, Holder::PlayerMain(p2), 0);
    (s, p1, p2)
}

fn out(name: &str, first_channel: u16) -> HolderOutput {
    HolderOutput {
        device: device(name),
        first_channel,
    }
}

#[test]
fn ca_terms_outputs_overlap_on_a_shared_channel_of_one_device() {
    assert!(outputs_overlap(&out("desk", 0), &out("desk", 0)));
    assert!(outputs_overlap(&out("desk", 0), &out("desk", 1)));
    assert!(outputs_overlap(&out("desk", 3), &out("desk", 2)));
    assert!(!outputs_overlap(&out("desk", 0), &out("desk", 2)));
    assert!(!outputs_overlap(&out("desk", 0), &out("phones", 0)));
}

#[test]
fn ca_terms_a_holder_plays_where_the_engine_placed_it() {
    let (s, p1, _) = shared();
    assert_eq!(holder_output(&s, Holder::PlayerCue(p1)), Some(out("desk", 0)));
    assert_eq!(
        holder_output(&s, Holder::PlayerMain(p1)),
        Some(out("speakers", 0))
    );
    let p3 = s.players[2].id;
    assert_eq!(holder_output(&s, Holder::PlayerMain(p3)), None, "not placed");
}

/// Review Focus 2.
#[test]
fn ca_terms_a_missing_main_route_plays_on_the_first_pair() {
    let mut s = fixture(4);
    let (p1, p2) = (s.players[0].id, s.players[1].id);
    place(&mut s, Holder::PlayerMain(p2), None, "desk");
    assert_eq!(holder_output(&s, Holder::PlayerMain(p2)), Some(out("desk", 0)));
    on_desk(&mut s, Holder::PlayerCue(p1), 0);
    assert!(cue_is_on_air(&s, Holder::PlayerCue(p1)));
    on_desk(&mut s, Holder::PlayerCue(p1), 2);
    assert!(!cue_is_on_air(&s, Holder::PlayerCue(p1)));
}

#[test]
fn ca_terms_a_route_the_engine_moved_to_another_device_plays_on_its_first_pair() {
    let mut s = fixture(4);
    let p2 = s.players[1].id;
    // A route to a backend this machine lacks: the engine plays it on the
    // default output, on channels 1–2.
    place(&mut s, Holder::PlayerMain(p2), Some(route("gone", 4)), "desk");
    assert_eq!(holder_output(&s, Holder::PlayerMain(p2)), Some(out("desk", 0)));
}

#[test]
fn ca_terms_another_players_main_puts_a_cue_on_air() {
    let (s, p1, p2) = shared();
    assert!(cue_is_on_air(&s, Holder::PlayerCue(p1)));
    assert!(!cue_is_on_air(&s, Holder::PlayerCue(p2)), "not placed");
}

#[test]
fn ca_terms_unequal_pairs_overlap_and_separate_pairs_do_not() {
    let (mut s, p1, p2) = shared();
    on_desk(&mut s, Holder::PlayerMain(p2), 1);
    assert!(cue_is_on_air(&s, Holder::PlayerCue(p1)), "1–2 and 2–3");
    on_desk(&mut s, Holder::PlayerMain(p2), 2);
    assert!(!cue_is_on_air(&s, Holder::PlayerCue(p1)), "1–2 and 3–4");
}

#[test]
fn ca_terms_a_cue_next_to_another_cue_only_is_not_on_air() {
    let (mut s, p1, p2) = shared();
    place(&mut s, Holder::PlayerMain(p2), Some(route("speakers", 2)), "speakers");
    let p3 = s.players[2].id;
    on_desk(&mut s, Holder::PlayerCue(p3), 0);
    assert!(!cue_is_on_air(&s, Holder::PlayerCue(p1)));
    apply(&mut s, Command::ToggleCue(p3)).unwrap();
    assert!(sounding_on(&s, Holder::PlayerCue(p1)).is_empty(), "not on air");
}

#[test]
fn ca_terms_what_sounds_on_an_on_air_output() {
    let (mut s, p1, p2) = shared();
    let cue = Holder::PlayerCue(p1);
    assert!(sounding_on(&s, cue).is_empty(), "quiet");
    apply(&mut s, Command::Play(p2)).unwrap();
    assert_eq!(sounding_on(&s, cue), vec![Holder::PlayerMain(p2)]);
    apply(&mut s, Command::Pause(p2)).unwrap();
    assert_eq!(
        sounding_on(&s, cue),
        vec![Holder::PlayerMain(p2)],
        "a player paused with a track sounds (BR3.5)"
    );
    apply(&mut s, Command::Stop(p2)).unwrap();
    assert!(sounding_on(&s, cue).is_empty());
}

#[test]
fn ca_terms_another_pre_listen_sounds_and_the_cue_decided_is_left_out() {
    let (mut s, p1, _) = shared();
    let p3 = s.players[2].id;
    on_desk(&mut s, Holder::PlayerCue(p3), 0);
    apply(&mut s, Command::ToggleCue(p3)).unwrap();
    assert_eq!(
        sounding_on(&s, Holder::PlayerCue(p1)),
        vec![Holder::PlayerCue(p3)]
    );
    assert!(
        sounding_on(&s, Holder::PlayerCue(p3)).is_empty(),
        "its own pre-listen does not count"
    );
}

#[test]
fn ca_terms_the_runtime_state_starts_empty() {
    let s = fixture(1);
    assert_eq!(s.cue_on_air, fp_model::cue_on_air::CueOnAir::default());
    assert_eq!(common::roundtrip(&s).cue_on_air, s.cue_on_air);
}
```

- [ ] **Step 2: Run it to see it fail**

Run: `cargo test -p fp-model --test cue_on_air`
Expected: compile error, `fp_model::cue_on_air` does not exist.

- [ ] **Step 3: `CueStart` in `command.rs`**

In `crates/fp-model/src/command.rs`, after the `Command` enum (before `/// Something the audio engine observed.`), add:

```rust
/// What a pre-listen plays (on-air pre-listen spec CA3, CA4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CueStart {
    /// `entry` on `player`'s Cue.
    Entry { player: PlayerId, entry: EntryId },
    /// A cart on the cartwall's Cue.
    Cart(CartId),
}

impl CueStart {
    /// The Cue holder it plays on.
    pub fn holder(self) -> Holder {
        match self {
            Self::Entry { player, .. } => Holder::PlayerCue(player),
            Self::Cart(_) => Holder::CartwallCue,
        }
    }
}
```

and in `crates/fp-model/src/lib.rs` extend the `pub use command::{…}` list with `CueStart`.

- [ ] **Step 4: The module**

Create `crates/fp-model/src/cue_on_air.rs`:

```rust
//! Pre-listen on an on-air output (on-air pre-listen spec, 2026-10-15):
//! where a holder plays, whether a Cue holder is on air and what sounds
//! there (§2), the confirmation request (CA3) and the notice (CA5). Pure
//! functions of `AppState`; the outputs are the ones the engine reported
//! (`live.placement`, `live.routes`).

use crate::command::CueStart;
use crate::config::OutputDevice;
use crate::live::{Holder, causes, configured_route, holders};
use crate::player::Transport;
use crate::state::AppState;

/// Where a holder plays: the device and the first channel of its pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolderOutput {
    pub device: OutputDevice,
    pub first_channel: u16,
}

/// CA3: a pre-listen that did not start because its on-air output sounds.
/// Runtime only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CueConfirmation {
    /// Numbers the requests and notices, so an answer to an older request
    /// is ignored and the interface tells a new notice from a shown one.
    pub id: u64,
    pub holder: Holder,
    /// What was asked: *Pre-listen anyway* sends it as `Command::ForceCue`.
    pub start: CueStart,
    /// The holders that sound on that output, in display order.
    pub sounding: Vec<Holder>,
}

/// CA5: an unconfirmed pre-listen on `cue` stopped because `by` started
/// on its output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CueStopped {
    pub id: u64,
    pub cue: Holder,
    pub by: Holder,
}

/// The runtime state of CA3–CA5 (`AppState::cue_on_air`); never saved.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CueOnAir {
    /// The pending confirmation request, if any.
    pub request: Option<CueConfirmation>,
    /// The last pre-listen CA5 stopped.
    pub notice: Option<CueStopped>,
    /// Cue holders whose running pre-listen was forced (CA4).
    pub confirmed: Vec<Holder>,
}

/// §2: the device the engine placed `holder` on and the first channel of
/// the route it holds there (the configured one when the engine has not
/// reported one). A route to another device (a missing backend plays on
/// the default output) and a missing route play on channels 1–2. `None`
/// while `holder` is not placed.
pub fn holder_output(state: &AppState, holder: Holder) -> Option<HolderOutput> {
    let device = state.live.placement.get(&holder)?.clone();
    let route = match state.live.routes.get(&holder) {
        Some(route) => route.clone(),
        None => configured_route(&state.config, holder),
    };
    let first_channel = route
        .filter(|r| r.backend == device.backend && r.device == device.device)
        .map_or(0, |r| r.first_channel);
    Some(HolderOutput {
        device,
        first_channel,
    })
}

/// §2: on the same device, with channel pairs that share a channel.
pub fn outputs_overlap(a: &HolderOutput, b: &HolderOutput) -> bool {
    a.device == b.device && a.first_channel.abs_diff(b.first_channel) < 2
}

/// A Main holder: what goes on air.
pub(crate) fn is_main(holder: Holder) -> bool {
    matches!(holder, Holder::PlayerMain(_) | Holder::CartwallMain)
}

/// The holders other than `holder` whose output overlaps its own, in
/// display order.
fn sharing(state: &AppState, holder: Holder) -> Vec<Holder> {
    let Some(own) = holder_output(state, holder) else {
        return Vec::new();
    };
    holders(state)
        .into_iter()
        .filter(|h| *h != holder)
        .filter(|h| holder_output(state, *h).is_some_and(|o| outputs_overlap(&o, &own)))
        .collect()
}

/// §2: `cue`'s output overlaps the output of a Main holder.
pub fn cue_is_on_air(state: &AppState, cue: Holder) -> bool {
    sharing(state, cue).into_iter().any(is_main)
}

/// `holder` carries audio now (`live::causes`): a playing or fading
/// player, a running CUE, a playing cart, the cart pre-listen.
pub(crate) fn audible(state: &AppState, holder: Holder) -> bool {
    !causes(state, holder).is_empty()
}

/// A player's Main paused with a current track (BR3.5).
fn paused_with_track(state: &AppState, holder: Holder) -> bool {
    match holder {
        Holder::PlayerMain(id) => state
            .player(id)
            .is_ok_and(|p| p.transport == Transport::Paused && p.current.is_some()),
        _ => false,
    }
}

/// §2, CA3: the holders that sound on `cue`'s on-air output, in display
/// order, `cue` left out. Empty when `cue` is not on air, its player is
/// gone, or nothing sounds there.
pub fn sounding_on(state: &AppState, cue: Holder) -> Vec<Holder> {
    if let Holder::PlayerCue(p) = cue
        && state.player(p).is_err()
    {
        return Vec::new();
    }
    let shared = sharing(state, cue);
    if !shared.iter().copied().any(is_main) {
        return Vec::new();
    }
    shared
        .into_iter()
        .filter(|h| audible(state, *h) || paused_with_track(state, *h))
        .collect()
}
```

Register it in `crates/fp-model/src/lib.rs`: add `pub mod cue_on_air;` after `pub mod config;`.

- [ ] **Step 5: `AppState::cue_on_air`**

In `crates/fp-model/src/state.rs`, add after the `live` field:

```rust
    /// Pre-listen on an on-air output: the confirmation request, the
    /// notice and the confirmed pre-listens (on-air pre-listen spec CA3–CA5);
    /// runtime only.
    pub cue_on_air: crate::cue_on_air::CueOnAir,
```

and `cue_on_air: crate::cue_on_air::CueOnAir::default(),` after `live: …` in `AppState::new` (L52) and in `crates/fp-model/src/session.rs` (L180).

- [ ] **Step 6: Run the tests**

Run: `cargo test -p fp-model --test cue_on_air`
Expected: PASS (11 tests).

- [ ] **Step 7: Gate and commit**

Run the gate. Then:

```bash
git add crates/fp-model/src/cue_on_air.rs crates/fp-model/src/lib.rs crates/fp-model/src/command.rs crates/fp-model/src/state.rs crates/fp-model/src/session.rs crates/fp-model/tests/cue_on_air.rs
git commit -m "feat(model): where a holder plays and what sounds on an on-air output

On-air pre-listen spec §2: a holder's output is its placement and the
pair of the route the engine holds; two outputs overlap on a shared
channel of one device; a Cue is on air next to a Main; an on-air output
sounds while a holder there is busy or a player there is paused with a
track.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: Quiet starts, sounding asks, force confirms (CA2, CA3, CA4, CA6)

**Files:**
- Modify: `crates/fp-model/src/command.rs` (`Command`, after `CueToNext`)
- Modify: `crates/fp-model/src/cue_on_air.rs`
- Modify: `crates/fp-model/src/reducer.rs` (`apply` L22-24, the `CueEntry` arm L102, `on_event` L389-390, `toggle_cue` L982-993, `cue_entry` L995-1018, `follow_cue` L1031-1036, the cartwall arms L300-308)
- Modify: `crates/fp-model/src/cart_rules.rs` (`request` L14, `cue` L99-121)
- Test: `crates/fp-model/tests/cue_on_air.rs`

**Interfaces:**
- Consumes: Task 2's `sounding_on`, `audible`, `CueOnAir::next_id`, `CueStart`.
- Produces:
  - `Command::ForceCue(CueStart)`, `Command::CancelCueOnAir(u64)`.
  - `cue_on_air::cue_start(state: &AppState, command: &Command) -> Option<CueStart>` — the pre-listen a command would start from nothing (`ToggleCue`/`SetCue(_, true)` without a running CUE: the next; `CueEntry`; `CueCart`/`SetCartCue(_, true)` of a cart not pre-listened); `None` for anything else, `ForceCue` included.
  - `cue_on_air::needs_confirmation(state: &AppState, command: &Command) -> bool` — `apply` would record a request (CA6; plan 3's remote and MIDI use it).
  - `pub(crate) cue_on_air::{ask, forced, cancel, after_step}`; `after_step(state: &mut AppState, out: &mut Vec<EngineAction>)` (Task 4 adds a `before` argument).
  - `pub(crate) cart_rules::cue_request(state: &AppState, id: CartId) -> Option<CartRequest>`; `cart_rules::cue(state, id, force: bool, out)`.

- [ ] **Step 1: Tests**

In `crates/fp-model/tests/cue_on_air.rs`, add `use std::path::PathBuf;` above `use common::…`, make it `use common::{entries, fixture};`, extend the `use fp_model::cue_on_air::{…}` list with `needs_confirmation`, and the `use fp_model::{…}` list with `CartId, CueStart, EntryId, TrackAnalysis`. Append the helpers and the tests:

```rust
fn next(s: &AppState, p: PlayerId) -> EntryId {
    s.player(p).unwrap().next.unwrap()
}

fn starts_cue(actions: &[EngineAction]) -> bool {
    actions
        .iter()
        .any(|a| matches!(a, EngineAction::StartCue { .. }))
}

fn cart_with_file(s: &mut AppState) -> CartId {
    let page = s.cartwall.pages[0].id;
    apply(
        s,
        Command::AssignCartFile {
            page,
            index: 0,
            path: PathBuf::from("/carts/jingle.wav"),
        },
    )
    .unwrap();
    let track = s.cartwall.pages[0].carts[0].track.unwrap();
    apply(
        s,
        Command::ApplyAnalysis {
            track,
            analysis: Box::new(TrackAnalysis {
                duration_secs: 5.0,
                ..TrackAnalysis::default()
            }),
        },
    )
    .unwrap();
    s.cartwall.pages[0].carts[0].id
}

#[test]
fn ca2_a_quiet_on_air_output_starts_the_cue_at_once() {
    let (mut s, p1, _) = shared();
    assert!(cue_is_on_air(&s, Holder::PlayerCue(p1)));
    let actions = apply(&mut s, Command::ToggleCue(p1)).unwrap();
    assert!(starts_cue(&actions), "{actions:?}");
    assert!(s.player(p1).unwrap().cue.is_some());
    assert_eq!(s.cue_on_air.request, None);
    assert!(s.cue_on_air.confirmed.is_empty(), "started, not confirmed");
}

#[test]
fn ca3_another_players_main_sounding_asks_first() {
    let (mut s, p1, p2) = shared();
    apply(&mut s, Command::Play(p2)).unwrap();
    let entry = next(&s, p1);
    let actions = apply(&mut s, Command::ToggleCue(p1)).unwrap();
    assert!(!starts_cue(&actions), "{actions:?}");
    assert!(s.player(p1).unwrap().cue.is_none());
    let request = s.cue_on_air.request.clone().unwrap();
    assert_eq!(request.holder, Holder::PlayerCue(p1));
    assert_eq!(request.start, CueStart::Entry { player: p1, entry });
    assert_eq!(request.sounding, vec![Holder::PlayerMain(p2)]);
}

#[test]
fn ca3_every_way_to_start_a_player_cue_asks() {
    let (mut s, p1, p2) = shared();
    apply(&mut s, Command::Play(p2)).unwrap();
    let e = entries(&s)[2];
    for command in [
        Command::ToggleCue(p1),
        Command::SetCue(p1, true),
        Command::CueEntry(p1, e),
    ] {
        s.cue_on_air.request = None;
        let actions = apply(&mut s, command.clone()).unwrap();
        assert!(!starts_cue(&actions), "{command:?}");
        assert!(s.cue_on_air.request.is_some(), "{command:?}");
    }
}

#[test]
fn ca3_overlapping_unequal_pairs_ask_and_separate_pairs_start() {
    let (mut s, p1, p2) = shared();
    on_desk(&mut s, Holder::PlayerMain(p2), 1);
    apply(&mut s, Command::Play(p2)).unwrap();
    apply(&mut s, Command::ToggleCue(p1)).unwrap();
    assert!(s.cue_on_air.request.is_some(), "1–2 and 2–3 overlap");
    apply(&mut s, Command::Stop(p2)).unwrap();
    on_desk(&mut s, Holder::PlayerMain(p2), 2);
    apply(&mut s, Command::Play(p2)).unwrap();
    let actions = apply(&mut s, Command::ToggleCue(p1)).unwrap();
    assert!(starts_cue(&actions), "1–2 and 3–4: no rule applies");
    assert_eq!(s.cue_on_air.request, None);
}

#[test]
fn ca3_a_paused_player_counts_as_sounding() {
    let (mut s, p1, p2) = shared();
    apply(&mut s, Command::Play(p2)).unwrap();
    apply(&mut s, Command::Pause(p2)).unwrap();
    apply(&mut s, Command::ToggleCue(p1)).unwrap();
    assert_eq!(
        s.cue_on_air.request.as_ref().map(|r| r.sounding.clone()),
        Some(vec![Holder::PlayerMain(p2)])
    );
}

#[test]
fn ca3_the_request_goes_when_the_output_stops_sounding() {
    let (mut s, p1, p2) = shared();
    apply(&mut s, Command::Play(p2)).unwrap();
    apply(&mut s, Command::ToggleCue(p1)).unwrap();
    assert!(s.cue_on_air.request.is_some());
    let actions = apply(&mut s, Command::Stop(p2)).unwrap();
    assert_eq!(s.cue_on_air.request, None);
    assert!(!starts_cue(&actions), "nothing starts by itself");
    assert!(s.player(p1).unwrap().cue.is_none());
}

#[test]
fn ca3_the_request_follows_what_sounds() {
    let (mut s, p1, p2) = shared();
    apply(&mut s, Command::Play(p2)).unwrap();
    apply(&mut s, Command::ToggleCue(p1)).unwrap();
    let p3 = s.players[2].id;
    on_desk(&mut s, Holder::PlayerMain(p3), 0);
    apply(&mut s, Command::Play(p3)).unwrap();
    assert_eq!(
        s.cue_on_air.request.as_ref().unwrap().sounding,
        vec![Holder::PlayerMain(p2), Holder::PlayerMain(p3)]
    );
}

/// Review Focus 3.
#[test]
fn ca3_a_cancel_answers_only_its_own_request() {
    let (mut s, p1, p2) = shared();
    apply(&mut s, Command::Play(p2)).unwrap();
    apply(&mut s, Command::ToggleCue(p1)).unwrap();
    let old = s.cue_on_air.request.clone().unwrap().id;
    let e = entries(&s)[2];
    apply(&mut s, Command::CueEntry(p1, e)).unwrap();
    let new = s.cue_on_air.request.clone().unwrap();
    assert!(new.id > old, "another request replaces it");
    assert_eq!(new.start, CueStart::Entry { player: p1, entry: e });
    let actions = apply(&mut s, Command::CancelCueOnAir(old)).unwrap();
    assert!(actions.is_empty(), "{actions:?}");
    assert_eq!(s.cue_on_air.request.as_ref(), Some(&new), "a stale cancel");
    apply(&mut s, Command::CancelCueOnAir(new.id)).unwrap();
    assert_eq!(s.cue_on_air.request, None);
    assert!(s.player(p1).unwrap().cue.is_none());
}

/// Review Focus 4.
#[test]
fn ca3_a_request_goes_with_its_player() {
    let (mut s, _, p2) = shared();
    let p4 = s.players[3].id;
    on_desk(&mut s, Holder::PlayerCue(p4), 0);
    apply(&mut s, Command::Play(p2)).unwrap();
    apply(&mut s, Command::ToggleCue(p4)).unwrap();
    assert!(s.cue_on_air.request.is_some());
    apply(&mut s, Command::SetPlayerCount(3)).unwrap();
    assert_eq!(s.cue_on_air.request, None);
}

#[test]
fn ca3_a_cart_pre_listen_asks_first() {
    let (mut s, _, p2) = shared();
    let cart = cart_with_file(&mut s);
    on_desk(&mut s, Holder::CartwallCue, 0);
    apply(&mut s, Command::Play(p2)).unwrap();
    for command in [Command::CueCart(cart), Command::SetCartCue(cart, true)] {
        s.cue_on_air.request = None;
        let actions = apply(&mut s, command.clone()).unwrap();
        assert!(
            !actions
                .iter()
                .any(|a| matches!(a, EngineAction::StartCartCue(_))),
            "{command:?}: {actions:?}"
        );
        assert_eq!(s.cartwall.cue, None);
        assert_eq!(
            s.cue_on_air.request.as_ref().map(|r| r.start),
            Some(CueStart::Cart(cart))
        );
    }
}

#[test]
fn ca3_a_running_cue_that_moves_asks_nothing() {
    let (mut s, p1, p2) = shared();
    apply(&mut s, Command::Play(p2)).unwrap();
    let entry = next(&s, p1);
    apply(&mut s, Command::ForceCue(CueStart::Entry { player: p1, entry })).unwrap();
    let e = entries(&s)[3];
    let actions = apply(&mut s, Command::CueEntry(p1, e)).unwrap();
    assert!(starts_cue(&actions), "{actions:?}");
    assert_eq!(s.player(p1).unwrap().cue.map(|c| c.entry), Some(e));
    assert_eq!(s.cue_on_air.request, None);
    assert_eq!(s.cue_on_air.confirmed, vec![Holder::PlayerCue(p1)]);
}

#[test]
fn ca4_a_forced_start_plays_and_is_confirmed() {
    let (mut s, p1, p2) = shared();
    apply(&mut s, Command::Play(p2)).unwrap();
    apply(&mut s, Command::ToggleCue(p1)).unwrap();
    let start = s.cue_on_air.request.clone().unwrap().start;
    let actions = apply(&mut s, Command::ForceCue(start)).unwrap();
    assert!(starts_cue(&actions), "{actions:?}");
    assert!(s.player(p1).unwrap().cue.is_some());
    assert_eq!(s.cue_on_air.request, None, "answered");
    assert_eq!(s.cue_on_air.confirmed, vec![Holder::PlayerCue(p1)]);
}

#[test]
fn ca4_forcing_a_running_cue_only_confirms_it() {
    let (mut s, p1, _) = shared();
    apply(&mut s, Command::ToggleCue(p1)).unwrap();
    let entry = s.player(p1).unwrap().cue.unwrap().entry;
    let actions =
        apply(&mut s, Command::ForceCue(CueStart::Entry { player: p1, entry })).unwrap();
    assert!(!starts_cue(&actions), "no restart: {actions:?}");
    assert_eq!(s.cue_on_air.confirmed, vec![Holder::PlayerCue(p1)]);
}

#[test]
fn ca4_a_forced_cart_pre_listen_plays_and_is_confirmed() {
    let (mut s, _, p2) = shared();
    let cart = cart_with_file(&mut s);
    on_desk(&mut s, Holder::CartwallCue, 0);
    apply(&mut s, Command::Play(p2)).unwrap();
    let actions = apply(&mut s, Command::ForceCue(CueStart::Cart(cart))).unwrap();
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EngineAction::StartCartCue(_))),
        "{actions:?}"
    );
    assert_eq!(s.cartwall.cue, Some(cart));
    assert_eq!(s.cue_on_air.confirmed, vec![Holder::CartwallCue]);
}

#[test]
fn ca4_a_force_for_an_unknown_entry_or_cart_is_refused() {
    let (mut s, p1, _) = shared();
    let bad = EntryId(999_999);
    assert!(apply(&mut s, Command::ForceCue(CueStart::Entry { player: p1, entry: bad })).is_err());
    assert!(apply(&mut s, Command::ForceCue(CueStart::Cart(CartId(999_999)))).is_err());
}

#[test]
fn ca6_needs_confirmation_says_what_apply_would_do() {
    let (mut s, p1, p2) = shared();
    let e = entries(&s)[2];
    let commands = [
        Command::ToggleCue(p1),
        Command::SetCue(p1, true),
        Command::CueEntry(p1, e),
        Command::SetCue(p1, false),
        Command::Play(p1),
    ];
    let check = |s: &AppState| {
        for command in &commands {
            let mut copy = s.clone();
            apply(&mut copy, command.clone()).unwrap();
            assert_eq!(
                needs_confirmation(s, command),
                copy.cue_on_air.request.is_some(),
                "{command:?}"
            );
        }
    };
    check(&s);
    assert!(!needs_confirmation(&s, &Command::ToggleCue(p1)), "quiet");
    apply(&mut s, Command::Play(p2)).unwrap();
    check(&s);
    assert!(needs_confirmation(&s, &Command::ToggleCue(p1)), "sounding");
    let entry = next(&s, p1);
    assert!(!needs_confirmation(
        &s,
        &Command::ForceCue(CueStart::Entry { player: p1, entry })
    ));
    apply(&mut s, Command::ForceCue(CueStart::Entry { player: p1, entry })).unwrap();
    check(&s);
    assert!(!needs_confirmation(&s, &Command::ToggleCue(p1)), "a stop");
}

#[test]
fn ca6_needs_confirmation_for_a_cart() {
    let (mut s, _, p2) = shared();
    let cart = cart_with_file(&mut s);
    on_desk(&mut s, Holder::CartwallCue, 0);
    assert!(!needs_confirmation(&s, &Command::CueCart(cart)));
    apply(&mut s, Command::Play(p2)).unwrap();
    assert!(needs_confirmation(&s, &Command::CueCart(cart)));
    assert!(needs_confirmation(&s, &Command::SetCartCue(cart, true)));
    assert!(!needs_confirmation(&s, &Command::SetCartCue(cart, false)));
}
```

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p fp-model --test cue_on_air`
Expected: compile errors: no `Command::ForceCue`, `Command::CancelCueOnAir`, `needs_confirmation`.

- [ ] **Step 3: The commands**

In `crates/fp-model/src/command.rs`, add to `Command` after `CueToNext(PlayerId),`:

```rust
    /// On-air pre-listen spec CA4: starts `start` even on a sounding
    /// on-air output; the pre-listen is confirmed (CA5 does not stop it).
    /// On a pre-listen already running for that entry or cart it only
    /// confirms it. Answers a confirmation request for the same holder.
    ForceCue(CueStart),
    /// CA3: the operator cancelled the confirmation request `id`; a newer
    /// request stays.
    CancelCueOnAir(u64),
```

- [ ] **Step 4: `cue_on_air.rs`: the predicate and the request**

Add to the `use` lines of `crates/fp-model/src/cue_on_air.rs`: `use crate::command::{Command, CueStart, EngineAction};` (replacing the `CueStart`-only import). Add the id counter to `CueOnAir` (a last field `seq: u64,`, private) and:

```rust
impl CueOnAir {
    /// The id of the next request or notice.
    pub(crate) fn next_id(&mut self) -> u64 {
        self.seq += 1;
        self.seq
    }
}
```

Append:

```rust
/// CA6: the pre-listen `command` would start from nothing: the next of a
/// player without a running CUE (`ToggleCue`, `SetCue(_, true)`), an entry
/// (`CueEntry`), a cart not pre-listened (`CueCart`,
/// `SetCartCue(_, true)`). `None` for anything else, `ForceCue` included.
pub fn cue_start(state: &AppState, command: &Command) -> Option<CueStart> {
    match command {
        Command::ToggleCue(p) | Command::SetCue(p, true) => {
            let player = state.player(*p).ok()?;
            if player.cue.is_some() {
                return None;
            }
            player.next.map(|entry| CueStart::Entry { player: *p, entry })
        }
        Command::CueEntry(p, entry) => Some(CueStart::Entry {
            player: *p,
            entry: *entry,
        }),
        Command::CueCart(cart) | Command::SetCartCue(cart, true) => {
            (state.cartwall.cue != Some(*cart)).then_some(CueStart::Cart(*cart))
        }
        _ => None,
    }
}

/// `start` begins a pre-listen from nothing: its holder has no running
/// one, a Cue route, and something it can play.
fn starts_fresh(state: &AppState, start: CueStart) -> bool {
    match start {
        CueStart::Entry { player, entry } => {
            state.player(player).is_ok_and(|p| p.cue.is_none())
                && state.config.outputs.player_has_cue(player)
                && state
                    .request_from_cue_in(entry, Holder::PlayerCue(player))
                    .is_some()
        }
        CueStart::Cart(cart) => {
            state.cartwall.cue.is_none()
                && state.config.outputs.cartwall_has_cue()
                && crate::cart_rules::cue_request(state, cart).is_some()
        }
    }
}

/// CA6: `apply(command)` would record a confirmation request instead of
/// starting a pre-listen (its on-air output sounds). The remote API and
/// MIDI cannot answer one, so they refuse such a command.
pub fn needs_confirmation(state: &AppState, command: &Command) -> bool {
    cue_start(state, command)
        .is_some_and(|start| starts_fresh(state, start) && !sounding_on(state, start.holder()).is_empty())
}

/// CA3: records the request for `start`, replacing any other, and returns
/// true when its on-air output sounds; false (nothing recorded) otherwise.
pub(crate) fn ask(state: &mut AppState, start: CueStart) -> bool {
    let holder = start.holder();
    let sounding = sounding_on(state, holder);
    if sounding.is_empty() {
        return false;
    }
    let id = state.cue_on_air.next_id();
    state.cue_on_air.request = Some(CueConfirmation {
        id,
        holder,
        start,
        sounding,
    });
    true
}

/// CA4: a forced pre-listen on `holder` answers its request and, once it
/// runs, is confirmed.
pub(crate) fn forced(state: &mut AppState, holder: Holder) {
    let on_air = &mut state.cue_on_air;
    if on_air.request.as_ref().is_some_and(|r| r.holder == holder) {
        on_air.request = None;
    }
    if audible(state, holder) && !state.cue_on_air.confirmed.contains(&holder) {
        state.cue_on_air.confirmed.push(holder);
    }
}

/// CA3: the operator cancelled request `id`.
pub(crate) fn cancel(state: &mut AppState, id: u64) {
    if state.cue_on_air.request.as_ref().is_some_and(|r| r.id == id) {
        state.cue_on_air.request = None;
    }
}

/// Closes every `apply` and `on_event`: the confirmed list keeps only
/// running pre-listens, and the request goes once its output no longer
/// sounds (or its holder already runs a pre-listen) or else names what
/// sounds now (CA3).
pub(crate) fn after_step(state: &mut AppState, _out: &mut Vec<EngineAction>) {
    let confirmed: Vec<Holder> = state
        .cue_on_air
        .confirmed
        .iter()
        .copied()
        .filter(|h| audible(state, *h))
        .collect();
    state.cue_on_air.confirmed = confirmed;
    let Some(holder) = state.cue_on_air.request.as_ref().map(|r| r.holder) else {
        return;
    };
    let sounding = if audible(state, holder) {
        Vec::new()
    } else {
        sounding_on(state, holder)
    };
    match state.cue_on_air.request.as_mut() {
        Some(request) if !sounding.is_empty() => request.sounding = sounding,
        _ => state.cue_on_air.request = None,
    }
}
```

Re-export in `crates/fp-model/src/lib.rs`, after the `pub use config::{…};` block:

```rust
pub use cue_on_air::{CueConfirmation, CueOnAir, CueStopped, needs_confirmation};
```

- [ ] **Step 5: `cart_rules.rs`**

In `crates/fp-model/src/cart_rules.rs`, add after `fn request` (L14-…):

```rust
/// The request a pre-listen of `id` would send, if the cart exists and can
/// play (on-air pre-listen spec CA6).
pub(crate) fn cue_request(state: &AppState, id: CartId) -> Option<CartRequest> {
    let cart = state.cartwall.cart(id)?;
    request(state, cart, false, Holder::CartwallCue)
}
```

Replace `cue` (L99-121) with:

```rust
/// C9, as amended by the on-air pre-listen spec: a pre-listen that starts
/// from nothing on a sounding on-air output asks first (CA3) unless it is
/// `force`d (CA4); one that replaces another cart's keeps going.
pub(crate) fn cue(
    state: &mut AppState,
    id: CartId,
    force: bool,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    let cart = state
        .cartwall
        .cart(id)
        .cloned()
        .ok_or(ModelError::UnknownCart(id))?;
    let was = state.cartwall.cue;
    stop_cue(state, out);
    // Spec §4.6, CA1: a pre-listen needs a cartwall Cue route.
    if was == Some(id) || !state.config.outputs.cartwall_has_cue() {
        return Ok(());
    }
    if let Some(request) = request(state, &cart, false, Holder::CartwallCue) {
        if was.is_none()
            && !force
            && crate::cue_on_air::ask(state, crate::command::CueStart::Cart(id))
        {
            return Ok(());
        }
        state.cartwall.cue = Some(id);
        state.cartwall.cue_normalization = Some(request.normalization);
        out.push(EngineAction::StartCartCue(request));
    }
    Ok(())
}
```

(Add `CartRequest` to the file's `use crate::command::{…}` import if it is not there.)

- [ ] **Step 6: The reducer**

In `crates/fp-model/src/reducer.rs`:

1. Add `CueStart` to the `use crate::command::{…}` import.
2. Replace the head of `apply` (L21-24):

```rust
/// Applies a user command. On `Err` the state is unchanged.
pub fn apply(state: &mut AppState, command: Command) -> Result<Vec<EngineAction>, ModelError> {
    let mut out = apply_command(state, command)?;
    crate::cue_on_air::after_step(state, &mut out);
    Ok(out)
}

/// `apply`, without the on-air pre-listen upkeep that closes every step.
fn apply_command(state: &mut AppState, command: Command) -> Result<Vec<EngineAction>, ModelError> {
    let mut out = Vec::new();
```

3. Replace the head of `on_event` (L387-390):

```rust
/// Applies an engine observation. Events about unknown players are ignored
/// (the player may have been removed while the event was in flight).
pub fn on_event(state: &mut AppState, event: EngineEvent) -> Vec<EngineAction> {
    let mut out = on_engine_event(state, event);
    crate::cue_on_air::after_step(state, &mut out);
    out
}

/// `on_event`, without the on-air pre-listen upkeep that closes every step.
fn on_engine_event(state: &mut AppState, event: EngineEvent) -> Vec<EngineAction> {
    let mut out = Vec::new();
```

4. The arms: `Command::CueEntry(id, entry) => cue_entry(state, id, entry, false, &mut out)?,`; after `Command::CueToNext(id) => { … }` add

```rust
        Command::ForceCue(start) => force_cue(state, start, &mut out)?,
        Command::CancelCueOnAir(id) => crate::cue_on_air::cancel(state, id),
```

and in the cartwall arms `cart_rules::cue(state, cart, &mut out)?` becomes `cart_rules::cue(state, cart, false, &mut out)?` (twice: `CueCart` and `SetCartCue`).

5. Replace `toggle_cue` and `cue_entry` (L982-1018) with:

```rust
fn toggle_cue(
    state: &mut AppState,
    id: PlayerId,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    let i = state.player_index(id)?;
    if state.players[i].cue.take().is_some() {
        out.push(EngineAction::StopCue { player: id });
    } else if let Some(entry) = state.players[i].next {
        cue_entry(state, id, entry, false, out)?;
    }
    Ok(())
}

/// Starts (or moves) `id`'s CUE on `entry`. A CUE that starts from nothing
/// on a sounding on-air output records a confirmation request instead,
/// unless it is `force`d (on-air pre-listen spec CA3, CA4); a running one
/// moves without asking.
fn cue_entry(
    state: &mut AppState,
    id: PlayerId,
    entry: EntryId,
    force: bool,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    let i = state.player_index(id)?;
    // Spec §4.6, CA1: without a Cue route nothing could be heard.
    if !state.config.outputs.player_has_cue(id) {
        return Ok(());
    }
    let request = state
        .request_from_cue_in(entry, Holder::PlayerCue(id))
        .ok_or(ModelError::UnknownEntry(entry))?;
    if state.players[i].cue.is_none()
        && !force
        && crate::cue_on_air::ask(state, CueStart::Entry { player: id, entry })
    {
        return Ok(());
    }
    state.players[i].cue = Some(CueState {
        entry,
        paused: false,
    });
    state.players[i].cue_normalization = Some(request.normalization);
    out.push(EngineAction::StartCue {
        player: id,
        request,
    });
    Ok(())
}

/// On-air pre-listen spec CA4: starts `start` even on a sounding on-air
/// output, or only confirms it when it already runs.
fn force_cue(
    state: &mut AppState,
    start: CueStart,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    match start {
        CueStart::Entry { player, entry } => {
            let i = state.player_index(player)?;
            if state.players[i].cue.is_none_or(|c| c.entry != entry) {
                cue_entry(state, player, entry, true, out)?;
            }
        }
        CueStart::Cart(cart) => {
            state
                .cartwall
                .cart(cart)
                .ok_or(ModelError::UnknownCart(cart))?;
            if state.cartwall.cue != Some(cart) {
                cart_rules::cue(state, cart, true, out)?;
            }
        }
    }
    crate::cue_on_air::forced(state, start.holder());
    Ok(())
}
```

6. In `follow_cue`, `cue_entry(state, id, entry, out)?;` becomes `cue_entry(state, id, entry, false, out)?;`.

- [ ] **Step 7: Run the tests**

Run: `cargo test -p fp-model --test cue_on_air && cargo test -p fp-model`
Expected: PASS. If `forced` does not compile because `on_air` borrows `state.cue_on_air` mutably across the `audible(state, …)` call, end the borrow first (the code above takes `on_air` only for the request check; keep that order).

- [ ] **Step 8: Gate and commit**

Run the gate (the workspace's exhaustive `match`es on `Command`, if any, get an arm; `command_available` already has `_ => true`). Then:

```bash
git add crates/fp-model
git commit -m "feat(model): ask before a pre-listen goes out on a sounding on-air output

On-air pre-listen spec CA2-CA4 and CA6: a pre-listen that starts from
nothing on an on-air output plays at once while the output is quiet and
otherwise records a confirmation request (runtime only, dropped when the
output goes quiet, when answered or replaced). ForceCue starts it anyway
and confirms it. needs_confirmation lets remote control and MIDI refuse
what they cannot confirm.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: Something starts: the unconfirmed pre-listen stops (CA5)

**Files:**
- Modify: `crates/fp-model/src/cue_on_air.rs` (`after_step`, new `Before`, `before_step`, `audible_mains`, `stop`)
- Modify: `crates/fp-model/src/reducer.rs` (the `apply` and `on_event` wrappers of Task 3)
- Test: `crates/fp-model/tests/cue_on_air.rs`

**Interfaces:**
- Consumes: Task 3's wrappers and `after_step`.
- Produces: `pub(crate) struct Before`; `pub(crate) fn before_step(state: &AppState) -> Before`; `pub(crate) fn after_step(state: &mut AppState, before: &Before, out: &mut Vec<EngineAction>)`; `AppState::cue_on_air.notice: Option<CueStopped>` filled by CA5 (plan 3 shows it).

- [ ] **Step 1: Tests**

Append to `crates/fp-model/tests/cue_on_air.rs` (add `CueStopped` to the `fp_model::cue_on_air` import):

```rust
fn notice(s: &AppState) -> Option<(Holder, Holder)> {
    s.cue_on_air.notice.map(|n| (n.cue, n.by))
}

#[test]
fn ca5_play_stops_an_unconfirmed_cue_first() {
    let (mut s, p1, p2) = shared();
    apply(&mut s, Command::ToggleCue(p1)).unwrap();
    let actions = apply(&mut s, Command::Play(p2)).unwrap();
    assert_eq!(
        actions.first(),
        Some(&EngineAction::StopCue { player: p1 }),
        "the pre-listen stops before the track starts: {actions:?}"
    );
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EngineAction::StartCurrent { player, .. } if *player == p2))
    );
    assert!(s.player(p1).unwrap().cue.is_none());
    assert_eq!(
        notice(&s),
        Some((Holder::PlayerCue(p1), Holder::PlayerMain(p2)))
    );
}

#[test]
fn ca5_a_confirmed_cue_keeps_playing() {
    let (mut s, p1, p2) = shared();
    apply(&mut s, Command::Play(p2)).unwrap();
    let entry = next(&s, p1);
    apply(&mut s, Command::ForceCue(CueStart::Entry { player: p1, entry })).unwrap();
    apply(&mut s, Command::Stop(p2)).unwrap();
    let actions = apply(&mut s, Command::Play(p2)).unwrap();
    assert!(
        !actions.contains(&EngineAction::StopCue { player: p1 }),
        "{actions:?}"
    );
    assert!(s.player(p1).unwrap().cue.is_some());
    assert_eq!(s.cue_on_air.notice, None);
}

/// Review Focus 5.
#[test]
fn ca5_a_confirmation_ends_with_its_cue() {
    let (mut s, p1, p2) = shared();
    apply(&mut s, Command::Play(p2)).unwrap();
    let entry = next(&s, p1);
    apply(&mut s, Command::ForceCue(CueStart::Entry { player: p1, entry })).unwrap();
    apply(&mut s, Command::ToggleCue(p1)).unwrap();
    assert!(s.cue_on_air.confirmed.is_empty());
    apply(&mut s, Command::Stop(p2)).unwrap();
    apply(&mut s, Command::ToggleCue(p1)).unwrap();
    apply(&mut s, Command::Play(p2)).unwrap();
    assert!(s.player(p1).unwrap().cue.is_none(), "unconfirmed again");
}

#[test]
fn ca5_the_players_own_play_stops_its_cue_on_one_card() {
    let mut s = fixture(4);
    let p1 = s.players[0].id;
    on_desk(&mut s, Holder::PlayerMain(p1), 0);
    on_desk(&mut s, Holder::PlayerCue(p1), 0);
    apply(&mut s, Command::ToggleCue(p1)).unwrap();
    let actions = apply(&mut s, Command::Play(p1)).unwrap();
    assert_eq!(actions.first(), Some(&EngineAction::StopCue { player: p1 }));
    assert_eq!(
        notice(&s),
        Some((Holder::PlayerCue(p1), Holder::PlayerMain(p1)))
    );
}

#[test]
fn ca5_a_paused_player_resuming_stops_it() {
    let (mut s, p1, p2) = shared();
    place(&mut s, Holder::PlayerMain(p2), Some(route("speakers", 2)), "speakers");
    apply(&mut s, Command::Play(p2)).unwrap();
    apply(&mut s, Command::Pause(p2)).unwrap();
    apply(&mut s, Command::ToggleCue(p1)).unwrap();
    assert!(s.player(p1).unwrap().cue.is_some(), "desk is quiet");
    // The paused player moves onto the desk (live settings L16): paused, it
    // sounds but starts nothing.
    on_desk(&mut s, Holder::PlayerMain(p2), 0);
    assert!(s.player(p1).unwrap().cue.is_some());
    let actions = apply(&mut s, Command::Play(p2)).unwrap();
    assert_eq!(actions.first(), Some(&EngineAction::StopCue { player: p1 }));
    assert_eq!(
        notice(&s),
        Some((Holder::PlayerCue(p1), Holder::PlayerMain(p2)))
    );
}

/// Review Focus 1.
#[test]
fn ca5_the_cartwalls_first_cart_stops_it_once_placed() {
    let (mut s, p1, _) = shared();
    let cart = cart_with_file(&mut s);
    s.config.outputs.cartwall.main = None;
    apply(&mut s, Command::ToggleCue(p1)).unwrap();
    apply(&mut s, Command::FireCart(cart)).unwrap();
    assert!(
        s.player(p1).unwrap().cue.is_some(),
        "the cartwall is not placed yet"
    );
    let actions = place(&mut s, Holder::CartwallMain, None, "desk");
    assert_eq!(actions.first(), Some(&EngineAction::StopCue { player: p1 }));
    assert_eq!(
        notice(&s),
        Some((Holder::PlayerCue(p1), Holder::CartwallMain))
    );
}

#[test]
fn ca5_a_cart_pre_listen_stops_when_a_player_starts() {
    let (mut s, _, p2) = shared();
    let cart = cart_with_file(&mut s);
    on_desk(&mut s, Holder::CartwallCue, 0);
    apply(&mut s, Command::CueCart(cart)).unwrap();
    assert_eq!(s.cartwall.cue, Some(cart));
    let actions = apply(&mut s, Command::Play(p2)).unwrap();
    assert_eq!(actions.first(), Some(&EngineAction::StopCartCue));
    assert_eq!(s.cartwall.cue, None);
    assert_eq!(
        notice(&s),
        Some((Holder::CartwallCue, Holder::PlayerMain(p2)))
    );
}

#[test]
fn ca5_another_pre_listen_does_not_stop_it() {
    let (mut s, p1, _) = shared();
    let p3 = s.players[2].id;
    on_desk(&mut s, Holder::PlayerCue(p3), 0);
    apply(&mut s, Command::ToggleCue(p1)).unwrap();
    let entry = next(&s, p3);
    apply(&mut s, Command::ForceCue(CueStart::Entry { player: p3, entry })).unwrap();
    assert!(s.player(p1).unwrap().cue.is_some());
    assert!(s.player(p3).unwrap().cue.is_some());
}

#[test]
fn ca5_a_cue_off_air_is_never_stopped() {
    let mut s = fixture(4);
    let (p1, p2) = (s.players[0].id, s.players[1].id);
    on_desk(&mut s, Holder::PlayerMain(p2), 0);
    place(&mut s, Holder::PlayerCue(p1), Some(route("phones", 0)), "phones");
    apply(&mut s, Command::ToggleCue(p1)).unwrap();
    let actions = apply(&mut s, Command::Play(p2)).unwrap();
    assert!(!actions.contains(&EngineAction::StopCue { player: p1 }));
    assert_eq!(s.cue_on_air.notice, None);
}

#[test]
fn ca5_each_notice_is_new() {
    let (mut s, p1, p2) = shared();
    apply(&mut s, Command::ToggleCue(p1)).unwrap();
    apply(&mut s, Command::Play(p2)).unwrap();
    let first: CueStopped = s.cue_on_air.notice.unwrap();
    apply(&mut s, Command::Stop(p2)).unwrap();
    apply(&mut s, Command::ToggleCue(p1)).unwrap();
    apply(&mut s, Command::Play(p2)).unwrap();
    assert!(s.cue_on_air.notice.unwrap().id > first.id);
}
```

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p fp-model --test cue_on_air ca5`
Expected: FAIL — no `StopCue`, no notice (`ca5_a_cue_off_air_is_never_stopped` and `ca5_another_pre_listen_does_not_stop_it` already pass).

- [ ] **Step 3: `cue_on_air.rs`**

Replace `after_step` and add, in `crates/fp-model/src/cue_on_air.rs`:

```rust
/// What CA5 compares a step against: each running pre-listen that is not
/// confirmed, with the Main holders audible on its output before the step.
pub(crate) struct Before(Vec<(Holder, Vec<Holder>)>);

/// The Main holders other than `cue` audible on `cue`'s output.
fn audible_mains(state: &AppState, cue: Holder) -> Vec<Holder> {
    sharing(state, cue)
        .into_iter()
        .filter(|h| is_main(*h) && audible(state, *h))
        .collect()
}

/// Opens every `apply` and `on_event` (CA5).
pub(crate) fn before_step(state: &AppState) -> Before {
    Before(
        holders(state)
            .into_iter()
            .filter(|h| matches!(h, Holder::PlayerCue(_) | Holder::CartwallCue))
            .filter(|h| audible(state, *h) && !state.cue_on_air.confirmed.contains(h))
            .map(|h| (h, audible_mains(state, h)))
            .collect(),
    )
}

/// Stops the pre-listen on `cue`.
fn stop(state: &mut AppState, cue: Holder, out: &mut Vec<EngineAction>) {
    match cue {
        Holder::PlayerCue(player) => {
            if let Ok(i) = state.player_index(player)
                && state.players[i].cue.take().is_some()
            {
                out.push(EngineAction::StopCue { player });
            }
        }
        Holder::CartwallCue => {
            if state.cartwall.cue.take().is_some() {
                out.push(EngineAction::StopCartCue);
            }
        }
        Holder::PlayerMain(_) | Holder::CartwallMain => {}
    }
}

/// Closes every `apply` and `on_event`. CA5: a pre-listen that was running
/// unconfirmed before the step stops when a Main holder became audible on
/// its output during it; the stop goes first in `out`, and a notice names
/// that holder. Then the confirmed list keeps only running pre-listens, and
/// the request goes once its output no longer sounds (or its holder already
/// runs a pre-listen) or else names what sounds now (CA3).
pub(crate) fn after_step(state: &mut AppState, before: &Before, out: &mut Vec<EngineAction>) {
    let mut stops = Vec::new();
    for (cue, was) in &before.0 {
        if !audible(state, *cue) || state.cue_on_air.confirmed.contains(cue) {
            continue;
        }
        let Some(by) = audible_mains(state, *cue)
            .into_iter()
            .find(|h| !was.contains(h))
        else {
            continue;
        };
        stop(state, *cue, &mut stops);
        let id = state.cue_on_air.next_id();
        state.cue_on_air.notice = Some(CueStopped { id, cue: *cue, by });
    }
    let confirmed: Vec<Holder> = state
        .cue_on_air
        .confirmed
        .iter()
        .copied()
        .filter(|h| audible(state, *h))
        .collect();
    state.cue_on_air.confirmed = confirmed;
    if let Some(holder) = state.cue_on_air.request.as_ref().map(|r| r.holder) {
        let sounding = if audible(state, holder) {
            Vec::new()
        } else {
            sounding_on(state, holder)
        };
        match state.cue_on_air.request.as_mut() {
            Some(request) if !sounding.is_empty() => request.sounding = sounding,
            _ => state.cue_on_air.request = None,
        }
    }
    if !stops.is_empty() {
        out.splice(0..0, stops);
        // A stopped pre-listen may free an output change (live settings L8).
        crate::live::dispatch_due(state, out);
    }
}
```

- [ ] **Step 4: The wrappers take the snapshot**

In `crates/fp-model/src/reducer.rs`, `apply` becomes:

```rust
pub fn apply(state: &mut AppState, command: Command) -> Result<Vec<EngineAction>, ModelError> {
    let before = crate::cue_on_air::before_step(state);
    let mut out = apply_command(state, command)?;
    crate::cue_on_air::after_step(state, &before, &mut out);
    Ok(out)
}
```

and `on_event`:

```rust
pub fn on_event(state: &mut AppState, event: EngineEvent) -> Vec<EngineAction> {
    let before = crate::cue_on_air::before_step(state);
    let mut out = on_engine_event(state, event);
    crate::cue_on_air::after_step(state, &before, &mut out);
    out
}
```

- [ ] **Step 5: Run the tests**

Run: `cargo test -p fp-model --test cue_on_air && cargo test -p fp-model`
Expected: PASS.

- [ ] **Step 6: Gate and commit**

Run the gate. Then:

```bash
git add crates/fp-model
git commit -m "feat(model): stop an unconfirmed pre-listen when something starts on its output

On-air pre-listen spec CA5: when a Main holder becomes audible on the
output of a pre-listen that was not confirmed, the same step stops the
pre-listen (its StopCue goes first) and records a notice naming that
holder. A confirmed pre-listen keeps playing. The comparison is per
pre-listen, so a Main the engine places after it starts (the cartwall's
first cart) is caught when its placement arrives.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: Bit-perfect players count the on-air pre-listen (CA7); docs

**Files:**
- Test: `crates/fp-model/tests/cue_on_air.rs`
- Modify: `docs/technical/architecture.md` (L7)
- Modify: `docs/superpowers/specs/2026-09-25-fauste-player-design.md` (rule 26 L198, §4.6 L285), `docs/superpowers/specs/2026-09-26-phase2-cartwall-settings-design.md` (C9), `docs/superpowers/specs/2026-10-01-remote-control-design.md` (its CUE operations)

**Interfaces:**
- Consumes: `fp_model::rate_cut`, the bit-perfect test helpers of `tests/common/mod.rs` (`bp_route`, `place_on_dac`, `report_stream_rate`, `set_track_rates`).

- [ ] **Step 1: The test**

Append to `crates/fp-model/tests/cue_on_air.rs`:

```rust
/// CA7: an on-air pre-listen on a bit-perfect device is a source like any
/// other: BR3.5 already keeps the device's rate while it sounds.
#[test]
fn ca7_an_on_air_cue_on_a_bit_perfect_device_keeps_the_rate() {
    let mut s = fixture(4);
    let (p1, p2) = (s.players[0].id, s.players[1].id);
    common::set_track_rates(&mut s, &[48_000, 44_100, 44_100, 44_100]);
    common::place_on_dac(&mut s, Holder::PlayerMain(p2));
    common::place_on_dac(&mut s, Holder::PlayerCue(p1));
    common::report_stream_rate(&mut s, 48_000, Vec::new());
    apply(&mut s, Command::Play(p2)).unwrap();
    let following = entries(&s)[1];
    let request = s
        .request_from_cue_in(following, Holder::PlayerMain(p2))
        .unwrap();
    assert_eq!(fp_model::rate_cut(&s, p2, &request), Some(44_100), "quiet");
    let entry = next(&s, p1);
    apply(&mut s, Command::ForceCue(CueStart::Entry { player: p1, entry })).unwrap();
    assert!(s.player(p1).unwrap().cue.is_some());
    assert_eq!(
        fp_model::rate_cut(&s, p2, &request),
        None,
        "the on-air pre-listen sounds on the device"
    );
}
```

- [ ] **Step 2: Run it**

Run: `cargo test -p fp-model --test cue_on_air ca7`
Expected: PASS at once (BR3.5 already counts it). If it fails, stop and record a ruling: the failure is a BR3.5 gap, not CA7's.

- [ ] **Step 3: Docs**

`docs/technical/architecture.md` L7: after "locked fader in `bit_perfect`)" insert ", pre-listen on an on-air output in `cue_on_air`: what sounds there, the confirmation request, the notice". 

Main spec rule 26 (L198): replace "Cue: a cue runs, or a next exists and the player has a usable Cue output (§4.6)." with "Cue: a cue runs, or a next exists and the player has a Cue route (§4.6 as amended by the on-air pre-listen spec CA1)." and "without a usable Cue output `ToggleCue`, `SetCue(true)` and `CueEntry` start nothing (a running CUE can still be stopped), and the dimmed CUE button's tooltip says to choose a Cue output apart from Main." with "without a Cue route `ToggleCue`, `SetCue(true)` and `CueEntry` start nothing (a running CUE can still be stopped), and the dimmed CUE button's tooltip says to choose a Cue output."

Main spec §4.6 (L285): append after the paragraph:

```markdown
**Amended (2026-10-15, [on-air pre-listen spec](2026-10-15-cue-on-air-output-design.md)):**
a Cue route that overlaps a Main route is usable (CA1). A pre-listen on an
on-air output plays at once while that output is quiet, asks the operator
while something sounds there, and stops by itself when something starts
there unless the operator confirmed it (CA2–CA5).
```

Cartwall spec C9 and the remote control spec's CUE operations: append one line each, "Amended by the [on-air pre-listen spec](2026-10-15-cue-on-air-output-design.md): …" — C9: "a cartwall Cue route on its Main output is usable, and a pre-listen on a sounding on-air output asks first (CA1, CA3)"; remote: "a CUE that would need a confirmation is refused with `409 cue_on_air`; `\"force\": true` starts it anyway (CA6, CA4)".

- [ ] **Step 4: Gate and commit**

Run the gate. Then:

```bash
git add crates/fp-model/tests/cue_on_air.rs docs/technical/architecture.md docs/superpowers/specs
git commit -m "docs(model): on-air pre-listen in the model and the amended specs

CA7's test pins that an on-air pre-listen on a bit-perfect device keeps
the running rate (BR3.5 already counts it). The main spec's rule 26 and
§4.6, cartwall C9 and the remote control spec point at the amendment.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

## Self-review (against the spec)

- §2 terms: Task 2 (`holder_output`, `outputs_overlap`, `cue_is_on_air`, `sounding_on`), with a missing Main route, overlapping unequal pairs, separate pairs, a paused player, another pre-listen, the decided holder left out.
- CA1: Task 1 (config, availability, C9) — the interface strings and the Settings note are plan 3.
- CA2, CA3, CA4: Task 3 (player CUE three ways, cart pre-listen two ways, request lifecycle: quiet, answered, replaced, stale cancel, player gone; force on running/new, cart force).
- CA5: Task 4 (play, own player, resume, cartwall's first cart, cart pre-listen, confirmed survives, off-air untouched, another pre-listen ignored).
- CA6: Task 3 (`needs_confirmation` agrees with `apply`); its use is plan 3.
- CA7: Task 5 (model side); the engine side is plan 2.
- CA8: plan 2. §5: plan 3.
