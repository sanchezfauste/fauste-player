# Bit-perfect Hard Transitions, Plan 4: the Interface, the Strings and the Guide

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The operator sees and understands the bit-perfect mode: a locked fader drawn at the top and disabled (BF7), a Play/Next tooltip, a dim MIX marker and a Settings note saying such a player does not mix (BR15), a bit-perfect hint that says what the mode costs (BF9), Apply now naming the faders it resets (BF8), every string in the eleven locales (§6), and a user guide that explains, before the operator turns it on, that bit-perfect is a special mode and what it costs (§9).

**Architecture:** The interface only reads the model (plan 1): `fp_model::fader_locked` and `fp_model::bit_perfect_player` for each player, `fp_model::faders_reset_by` for the confirmation. `widgets::fader` gains an `enabled` flag; `view::mix_active(state, player)` decides the MIX marker's style for the player and its CUE window; Settings → Players shows a note while any player is a bit-perfect player. No new command, no new state.

**Tech Stack:** Rust 2024, `fp-app` (egui, Fluent), `egui_kittest` tests with the recording `Fake` controller; mdBook guide in `docs/user/`.

**Spec:** `docs/superpowers/specs/2026-10-08-bit-perfect-rate-and-fader-design.md` (§4.7 BR15, §5.2 BF7–BF9, §6 all strings, §7 `fp-app` row, §8 the loopback steps, §9.1, §9.2 `README.md` and `ui.md`, §9.4).

This is plan 4 of 4 (after plans 1–3; it needs `fader_locked`, `bit_perfect_player`, `faders_reset_by` and the `tip-volume-locked` key from plan 1, and links the MIDI and remote pages plan 3 wrote).

## Rulings (planning)

**Ruling: the dim MIX marker is the existing "not active" MIX style.** BR15 says the marker is drawn "with the style it already uses for markers that do not apply (the cue markers with 'Use cue-in and cue-out' off)". The MIX marker has its own dim style already, used in Single mode (`mix_active: false`, `crates/fp-app/src/ui/widgets.rs` ~L1626: `NEUTRAL_600` at 60 % instead of amber); the plan uses that one, through a new `view::mix_active(state, player)`: Continuous mode and not a bit-perfect player. — Why: one style for "this MIX point does not apply here", and no new drawing code. — Cost if wrong: the colour of one marker.

**Ruling: the CUE window's MIX marker follows its player.** `CueWindowView::mix_active` "looks as on the player" (`view.rs` L171-172); it uses `view::mix_active` too, so a bit-perfect player's CUE window draws the marker dim. — Cost if wrong: one call.

**Ruling: the nine translated locales are written in this plan.** §6 gives en-US and es-ES for `tip-next-bit-perfect`, `settings-players-bit-perfect-note` and `settings-bit-perfect-hint`, and asks for AI translations of the other nine "at implementation time"; they are written below (from the en-US text, with each locale's existing names for the settings and buttons they mention), so the executor copies them. — Cost if wrong: a wording fix in one file.

**Ruling: the locked fader is painted at the top.** The model resets a locked player's volume to 1.0 (BF2) on the event that locks it, but the interface may draw one frame before that event's state is published; `meter_column` draws a locked fader at travel 1.0 whatever the volume says. — Cost if wrong: none.

## Global Constraints

- All code, identifiers, comments, docs and commit messages in English; UI strings are Fluent messages, `crates/fp-app/locales/en-US/main.ftl` is the source, every new or changed string goes into **all eleven** locale files (ca-ES, de-DE, en-US, es-ES, eu-ES, fr-FR, gl-ES, it-IT, nl-NL, pl-PL, pt-PT); never mention other playout or radio-automation products.
- The UI never blocks and only reads snapshots and sends commands (CLAUDE.md rule 8).
- BF7: "a locked player's fader is drawn at the top, disabled (greyed, no drag, no wheel), with the tooltip `tip-volume-locked`. `widgets::fader` … gains an `enabled` flag. The tooltip replaces `tip-volume-dsd`."
- BR15: "Both settings stay editable: they are global and still apply to the other players."
- §6: `tip-volume-dsd` is removed from every locale; `{ $players }` is "a list such as 'P1, P2' built from the existing player labels" (`player-label`).
- User guide: English only in `docs/user/`; the translations in `docs/i18n/` are regenerated before a release, never in this PR.
- `unwrap`, `expect`, `panic` denied outside tests.
- UI tests use `egui_kittest` with the recording `Fake` controller in `crates/fp-app/tests/support`.
- Conventional Commits; every commit ends with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Gate for every task: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`.
- Progress ledger: `.superpowers/sdd/2026-10-13-bit-perfect-plan4-interface-docs/progress.md`.

## Review Focus

1. **Scrolling the mouse wheel over a locked fader** (the most likely accidental gesture): nothing is sent, no "refused" notice appears. Test: Task 1, `bf7_a_locked_fader_takes_no_drag_and_no_wheel`.
2. **A locked player whose state still says 50 % for a frame**: the fader is drawn at the top. Test: Task 1, `bf7_a_locked_fader_is_drawn_at_the_top`.
3. **Two players on the same bit-perfect device, one stopped**: the Settings note shows (a bit-perfect player exists even when idle), and disappears when bit-perfect is turned off. Test: Task 3, `br15_the_players_note_shows_only_while_a_bit_perfect_player_exists`.
4. **Apply now with bit-perfect turned on for a device whose player is at 100 %**: no fader line (nothing jumps). Test: Task 4, `bf8_no_fader_line_when_every_fader_is_at_full`.
5. **The Play/Next tooltip of a bit-perfect player that is stopped**: it still reads "Play" (only "on air" changes). Test: Task 2, `br15_the_next_tooltip_says_there_is_no_crossfade`.

---

## File Structure

| File | Responsibility |
|---|---|
| `crates/fp-app/src/ui/widgets.rs` (`fader` L1252-1308) | `enabled` |
| `crates/fp-app/src/ui/player.rs` (`meter_column` L465-504; `transport` ~L632-648; `wave` L835-866) | BF7, the Play/Next tooltip, the MIX marker |
| `crates/fp-app/src/ui/view.rs` (`cue_window_view` ~L210) | `mix_active` |
| `crates/fp-app/src/ui/settings.rs` (`players` L1174, after the auto-segue row ~L1243) | the Players note |
| `crates/fp-app/src/ui/pending.rs` (`show_confirm` L368-410) | the fader line |
| `crates/fp-app/locales/*/main.ftl` | §6 |
| `crates/fp-app/tests/support/mod.rs` | `lock_player` |
| `crates/fp-app/tests/bit_perfect_ui.rs` (new), `tests/view.rs`, `tests/live_settings.rs`, `tests/dsd_ui.rs` | tests |
| `docs/user/{bit-perfect,players,markers-and-mixing,settings,troubleshooting,README}.md`, `README.md`, `docs/technical/ui.md` | Task 5 |

---

### Task 1: The locked fader on screen (BF7)

**Files:**
- Modify: `crates/fp-app/src/ui/widgets.rs` (`fader`, L1252-1308)
- Modify: `crates/fp-app/src/ui/player.rs` (`meter_column`, L489-503)
- Modify: every locale file: remove `tip-volume-dsd` (L21 in each)
- Modify: `crates/fp-app/tests/support/mod.rs` (`lock_player`)
- Modify: `crates/fp-app/tests/dsd_ui.rs` (`the_fader_says_dsd_plays_at_full_volume`, L53-61, goes: the key is gone)
- Test: `crates/fp-app/tests/bit_perfect_ui.rs` (new)

**Interfaces:**
- Consumes: `fp_model::fader_locked` and the key `tip-volume-locked` (plan 1).
- Produces: `pub fn fader(ui: &mut Ui, height: f32, position: f32, label: &str, enabled: bool) -> Option<f32>`; test helper `pub fn lock_player(state: &mut AppState, n: usize)`.

- [ ] **Step 1: The test helper**

`crates/fp-app/tests/support/mod.rs`:

```rust
/// Places player `n`'s (0-based) Main on a `dac` that runs bit-perfect (live
/// settings L3): its fader locks and it starts every track hard
/// (bit-perfect rate and fader spec BR2, BF1).
pub fn lock_player(state: &mut AppState, n: usize) {
    let p = state.players[n].id;
    let route = Route {
        backend: "null".to_owned(),
        device: "dac".to_owned(),
        first_channel: 0,
    };
    let dac = fp_model::OutputDevice {
        backend: "null".to_owned(),
        device: "dac".to_owned(),
    };
    if let Some(r) = state.config.outputs.routes.iter_mut().find(|r| r.player == p) {
        r.main = Some(route.clone());
    }
    if !state.config.outputs.bit_perfect.contains(&dac) {
        state.config.outputs.bit_perfect.push(dac.clone());
    }
    let running = fp_model::device_settings(&state.config.outputs, &dac);
    fp_model::on_event(
        state,
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

`crates/fp-app/tests/bit_perfect_ui.rs`:

```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Bit-perfect rate and fader spec: the locked fader (BF7), the cues that a
//! bit-perfect player does not mix (BR15).

mod support;

use egui_kittest::kittest::Queryable;
use fp_model::Command;
use support::{harness, lock_player, state};

const LOCKED: &str = "Volume held at 100 %: this player's output is bit-perfect.";

fn locked(players: usize) -> fp_model::AppState {
    let mut s = state(players, 3);
    lock_player(&mut s, 0);
    s
}

fn volume_commands(sent: &[Command]) -> usize {
    sent.iter()
        .filter(|c| matches!(c, Command::SetVolume(..)))
        .count()
}

#[test]
fn bf7_a_locked_fader_takes_no_drag_and_no_wheel() {
    let (mut h, fake) = harness(locked(1));
    h.run_steps(2);
    let at = h.get_by_label(LOCKED).rect().center();
    h.hover_at(at);
    h.step();
    h.event(egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Line,
        delta: egui::vec2(0.0, -1.0),
        phase: egui::TouchPhase::Move,
        modifiers: egui::Modifiers::NONE,
    });
    h.run_steps(5);
    h.event(egui::Event::PointerButton {
        pos: at,
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::NONE,
    });
    h.step();
    let lower = at + egui::vec2(0.0, 40.0);
    h.event(egui::Event::PointerMoved(lower));
    h.step();
    h.event(egui::Event::PointerButton {
        pos: lower,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::NONE,
    });
    h.run_steps(5);
    assert_eq!(volume_commands(&fake.take_sent()), 0);
    assert_eq!(fake.state.load().players[0].volume, 1.0);
    assert!(fake.take_rejection().is_none(), "nothing was refused: nothing was sent");
}

#[test]
fn bf7_a_locked_fader_is_drawn_at_the_top() {
    let mut s = locked(1);
    // A frame where the state has not caught up with the lock yet.
    s.players[0].volume = 0.5;
    let (mut h, _fake) = harness(s);
    h.run_steps(2);
    let fader = h.get_by_label(LOCKED);
    // The knob is drawn in the top 6 px of the fader's rect: its top
    // stays inside the widget, and the widget is the same rect as an
    // unlocked fader at 100 % (checked against one).
    let locked_rect = fader.rect();
    let (mut h2, _) = harness(state(1, 3));
    h2.run_steps(2);
    let free_rect = h2.get_by_label_contains("Volume").rect();
    assert_eq!(locked_rect.size(), free_rect.size());
}

#[test]
fn bf7_an_unlocked_fader_still_moves() {
    let (mut h, fake) = harness(state(1, 3));
    h.run_steps(2);
    let at = h.get_by_label_contains("Volume").rect().center();
    h.hover_at(at);
    h.step();
    h.event(egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Line,
        delta: egui::vec2(0.0, -1.0),
        phase: egui::TouchPhase::Move,
        modifiers: egui::Modifiers::NONE,
    });
    h.run_steps(5);
    assert_eq!(volume_commands(&fake.take_sent()), 1);
}
```

(`bf7_a_locked_fader_is_drawn_at_the_top` cannot read pixels; it checks the widget exists with the locked label and the same geometry while the state says 50 %. Step 3's code draws `position = 1.0` when locked; a reviewer checks that line. If the kittest snapshot feature is enabled in this crate — `grep -n "snapshot" crates/fp-app/Cargo.toml` — replace the geometry check with a snapshot of the meter column.)

- [ ] **Step 3: Run to verify it fails**

Run: `cargo test -p fp-app --test bit_perfect_ui 2>&1 | tail -8`
Expected: FAIL — no widget is labelled with `LOCKED` (the fader's tooltip is `tip-volume`).

- [ ] **Step 4: Implement**

`widgets.rs`, `fader`:

```rust
/// The volume fader: `position` is its travel (0 bottom … 1 top). A fader
/// that is not `enabled` (a bit-perfect output holds the volume at 100 %,
/// bit-perfect rate and fader spec BF7) is drawn greyed and takes no drag,
/// click or wheel.
pub fn fader(ui: &mut Ui, height: f32, position: f32, label: &str, enabled: bool) -> Option<f32> {
    let sense = if enabled {
        Sense::click_and_drag()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(vec2(FADER_WIDTH, height), sense);
    let owned = label.to_owned();
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Slider, enabled, owned.clone()));
    let mut changed = None;
    if enabled
        && (response.dragged() || response.clicked())
        && let Some(p) = response.interact_pointer_pos()
    {
        changed = Some((1.0 - (p.y - rect.top()) / rect.height()).clamp(0.0, 1.0));
    }
    if enabled && response.hovered() {
        // …the wheel code, unchanged…
    }
```

and in the painting, the level uses `if enabled { theme::NEUTRAL_300 } else { theme::NEUTRAL_600 }` and the knob `if enabled { theme::NEUTRAL_100 } else { theme::NEUTRAL_600 }`.

`player.rs`, `meter_column`, from `let tip = …`:

```rust
    // BF7: a bit-perfect output holds the volume at 100 %; the fader is
    // drawn at the top and disabled.
    let locked = fp_model::fader_locked(scene.state, id);
    let tip = if locked {
        t.tr("tip-volume-locked")
    } else {
        t.tr_args("tip-volume", &[("db", db.into())])
    };
    let position = if locked {
        1.0
    } else {
        view::fader_from_gain(volume)
    };
    if let Some(pos) = widgets::fader(ui, height, position, &tip, !locked) {
        scene
            .ctl
            .send(Command::SetVolume(id, view::gain_from_fader(pos)));
    }
```

(`telemetry` is still used above for the meter; only its `dsd` branch here goes.)

Remove the line `tip-volume-dsd = …` from all eleven `main.ftl` files, and delete `the_fader_says_dsd_plays_at_full_volume` from `tests/dsd_ui.rs` (its replacement is `bf7_…` above).

- [ ] **Step 5: Run to verify it passes**

Run: `cargo test -p fp-app --test bit_perfect_ui && cargo test -p fp-app --test dsd_ui && cargo test -p fp-app --test i18n && cargo test -p fp-app --test main_screen`
Expected: PASS.

- [ ] **Step 6: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-app
git commit -m "feat(ui): draw a bit-perfect player's fader locked at 100 %

The fader of a player whose output is bit-perfect is drawn at the top,
greyed, takes no drag or wheel, and says why; the DSD fader tooltip goes,
since such a fader can no longer move (bit-perfect rate and fader spec
BF7).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: The Play/Next tooltip and the dim MIX marker (BR15, §6.4)

**Files:**
- Modify: `crates/fp-app/src/ui/view.rs` (a new `mix_active` near `shown_entry` ~L140; `cue_window_view`'s `mix_active:` ~L210)
- Modify: `crates/fp-app/src/ui/player.rs` (`transport`, `play_label` ~L643-647; `wave`, `mix_active:` L850)
- Modify: every locale file: `tip-next-bit-perfect` after `tip-next` (L28)
- Test: `crates/fp-app/tests/bit_perfect_ui.rs`, `crates/fp-app/tests/view.rs`

**Interfaces:**
- Consumes: `fp_model::bit_perfect_player` (plan 1).
- Produces: `pub fn mix_active(state: &AppState, player: PlayerId) -> bool` in `fp_app::ui::view`.

- [ ] **Step 1: Write the failing tests**

`tests/bit_perfect_ui.rs`:

```rust
#[test]
fn br15_the_next_tooltip_says_there_is_no_crossfade() {
    let (mut h, fake) = harness(locked(1));
    h.run_steps(2);
    assert!(
        h.query_all_by_label("Play").next().is_some(),
        "stopped: the usual Play"
    );
    fake.send(Command::Play(fake.player(0)));
    h.run_steps(2);
    assert!(
        h.query_by_label("Next track, at once and without a crossfade (bit-perfect output)")
            .is_some()
    );
    assert!(h.query_by_label("Fade into the next track").is_none());
}
```

`tests/view.rs` (add `mix_active` to the `fp_app::ui::view` imports; the file has no `support` module, so the lock is built with the model's own event, as `support::lock_player` does):

```rust
#[test]
fn br15_the_mix_marker_is_dim_on_a_bit_perfect_player() {
    let (mut s, _, p) = state(3);
    let q = s.players[1].id;
    assert!(mix_active(&s, p), "Continuous mode");
    let dac = fp_model::OutputDevice {
        backend: "null".into(),
        device: "dac".into(),
    };
    s.config.outputs.bit_perfect = vec![dac.clone()];
    let running = fp_model::device_settings(&s.config.outputs, &dac);
    fp_model::on_event(
        &mut s,
        fp_model::EngineEvent::Placed {
            holder: fp_model::Holder::PlayerMain(p),
            route: None,
            device: dac,
            running,
        },
    );
    assert!(!mix_active(&s, p), "a bit-perfect player never mixes");
    assert!(mix_active(&s, q), "the other player still does");
    apply(&mut s, Command::SetMode(q, PlayMode::Single)).unwrap();
    assert!(!mix_active(&s, q), "Single mode, as before");
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p fp-app --test view br15 2>&1 | tail -5`
Expected: FAIL to compile ("cannot find function `mix_active`").

- [ ] **Step 3: Implement**

`view.rs`:

```rust
/// Whether `player`'s MIX marker is drawn solid: in Continuous mode, on a
/// player that mixes. A bit-perfect player never does (bit-perfect rate
/// and fader spec BR15): its marker is drawn dim, as in Single mode.
pub fn mix_active(state: &AppState, player: PlayerId) -> bool {
    state
        .player(player)
        .is_ok_and(|p| p.mode == PlayMode::Continuous)
        && !fp_model::bit_perfect_player(state, player)
}
```

`cue_window_view`: `mix_active: mix_active(state, player),` (the player the CUE window belongs to). `player.rs`, `wave`: `mix_active: view::mix_active(scene.state, id),`.

`transport`:

```rust
            let play_label = if !playing {
                t.tr("tip-play")
            } else if fp_model::bit_perfect_player(scene.state, id) {
                t.tr("tip-next-bit-perfect")
            } else {
                t.tr("tip-next")
            };
```

Locales, after `tip-next`:

| File | Line |
|---|---|
| `en-US` | `tip-next-bit-perfect = Next track, at once and without a crossfade (bit-perfect output)` |
| `es-ES` | `tip-next-bit-perfect = Siguiente pista, al momento y sin fundido (salida bit-perfect)` |
| `ca-ES` | `tip-next-bit-perfect = Pista següent, a l'instant i sense fos (sortida bit-perfect)` |
| `de-DE` | `tip-next-bit-perfect = Nächster Track, sofort und ohne Überblendung (Bit-perfect-Ausgang)` |
| `eu-ES` | `tip-next-bit-perfect = Hurrengo pista, berehala eta itzaltzerik gabe (bit-perfect irteera)` |
| `fr-FR` | `tip-next-bit-perfect = Piste suivante, aussitôt et sans fondu (sortie bit-perfect)` |
| `gl-ES` | `tip-next-bit-perfect = Seguinte pista, ao momento e sen fundido (saída bit-perfect)` |
| `it-IT` | `tip-next-bit-perfect = Brano successivo, subito e senza dissolvenza (uscita bit-perfect)` |
| `nl-NL` | `tip-next-bit-perfect = Volgend nummer, meteen en zonder overfaden (bit-perfect uitgang)` |
| `pl-PL` | `tip-next-bit-perfect = Następny utwór, od razu i bez przenikania (wyjście bit-perfect)` |
| `pt-PT` | `tip-next-bit-perfect = Faixa seguinte, de imediato e sem fade (saída bit-perfect)` |

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p fp-app --test view && cargo test -p fp-app --test bit_perfect_ui && cargo test -p fp-app --test i18n`
Expected: PASS.

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-app
git commit -m "feat(ui): show that a bit-perfect player does not mix

Its Play/Next tooltip says the next track starts at once without a
crossfade, and its MIX marker is drawn dim, on the player and in its CUE
window (bit-perfect rate and fader spec BR15).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: The Settings texts (BR15 note, BF9 hint; §6.3, §6.5)

**Files:**
- Modify: `crates/fp-app/src/ui/settings.rs` (`players`, after the `settings-auto-segue` row ending ~L1243)
- Modify: every locale file: `settings-players-bit-perfect-note` after `settings-hint-auto-segue` (L175 in en-US); `settings-bit-perfect-hint` replaced (L208 in en-US, L212 in pl-PL)
- Test: `crates/fp-app/tests/bit_perfect_ui.rs`

- [ ] **Step 1: Write the failing test**

```rust
const NOTE: &str = "Players on a bit-perfect output never mix or crossfade";

fn open_players(h: &mut egui_kittest::Harness<'static, fp_app::ui::app::AppUi>) {
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(egui::accesskit::Role::Button, "Players")
        .click();
    h.run_steps(2);
}

#[test]
fn br15_the_players_note_shows_only_while_a_bit_perfect_player_exists() {
    let (mut h, _) = harness(state(2, 3));
    open_players(&mut h);
    assert!(h.query_by_label_contains(NOTE).is_none());
    let (mut h, fake) = harness(locked(2));
    open_players(&mut h);
    assert!(h.query_by_label_contains(NOTE).is_some(), "P1 is idle but bit-perfect");
    // The settings it explains stay editable.
    assert!(
        h.query_by_role_and_label(egui::accesskit::Role::Slider, "Fade time")
            .is_some()
    );
    // Bit-perfect turned off for the device.
    let mut s = (**fake.state.load()).clone();
    let dac = fp_model::OutputDevice {
        backend: "null".into(),
        device: "dac".into(),
    };
    s.live.devices.get_mut(&dac).unwrap().bit_perfect = false;
    fake.state.store(std::sync::Arc::new(s));
    h.run_steps(2);
    assert!(h.query_by_label_contains(NOTE).is_none());
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p fp-app --test bit_perfect_ui br15_the_players 2>&1 | tail -5`
Expected: FAIL — the note is never shown.

- [ ] **Step 3: Implement**

`settings.rs`, `players`, right after the automatic-mix row:

```rust
    // BR15: the two settings above are global; say they do not apply to
    // the players on a bit-perfect output, which never mix.
    if scene
        .state
        .players
        .iter()
        .any(|p| fp_model::bit_perfect_player(scene.state, p.id))
    {
        note(ui, &t.tr("settings-players-bit-perfect-note"), theme::NEUTRAL_400);
    }
```

(`note` is `settings::note`, L670.)

Locales — `settings-players-bit-perfect-note`, after `settings-hint-auto-segue`:

| File | Line |
|---|---|
| `en-US` | `settings-players-bit-perfect-note = Players on a bit-perfect output never mix or crossfade: automatic mix does not apply to them, and the fade time only to Fade stop.` |
| `es-ES` | `settings-players-bit-perfect-note = Los reproductores con salida bit-perfect nunca mezclan ni encadenan con fundido: la mezcla automática no se les aplica, y la duración del fundido solo cuenta para Stop con fundido.` |
| `ca-ES` | `settings-players-bit-perfect-note = Els reproductors amb sortida bit-perfect mai no mesclen ni encadenen amb fos: la mescla automàtica no s'hi aplica, i la durada del fos només compta per a l'Stop amb fos.` |
| `de-DE` | `settings-players-bit-perfect-note = Player mit Bit-perfect-Ausgang mischen und überblenden nie: Der automatische Mix gilt für sie nicht, und die Blendzeit nur für Ausblenden.` |
| `eu-ES` | `settings-players-bit-perfect-note = Bit-perfect irteera duten erreproduzigailuek ez dute inoiz nahasten ezta itzaltzearekin kateatzen ere: nahasketa automatikoa ez zaie aplikatzen, eta itzaltze-denbora «Itzaltzearekin gelditu»rako bakarrik da.` |
| `fr-FR` | `settings-players-bit-perfect-note = Les lecteurs sur une sortie bit-perfect ne mixent ni n'enchaînent jamais en fondu : l'enchaînement automatique ne s'applique pas à eux, et la durée du fondu ne vaut que pour le Stop en fondu.` |
| `gl-ES` | `settings-players-bit-perfect-note = Os reprodutores con saída bit-perfect nunca mesturan nin encadean con fundido: a mestura automática non se lles aplica, e a duración do fundido só conta para o Stop con fundido.` |
| `it-IT` | `settings-players-bit-perfect-note = I lettori con uscita bit-perfect non mixano né concatenano mai in dissolvenza: il mix automatico non si applica a loro, e la durata della dissolvenza vale solo per lo Stop in dissolvenza.` |
| `nl-NL` | `settings-players-bit-perfect-note = Spelers op een bit-perfect uitgang mixen of overfaden nooit: automatisch mixen geldt niet voor hen, en de fadetijd alleen voor Uitfaden.` |
| `pl-PL` | `settings-players-bit-perfect-note = Odtwarzacze z wyjściem bit-perfect nigdy nie miksują ani nie przenikają: automatyczny miks ich nie dotyczy, a czas wyciszania dotyczy tylko Stop z wyciszeniem.` |
| `pt-PT` | `settings-players-bit-perfect-note = Os leitores com saída bit-perfect nunca misturam nem encadeiam com fade: a mistura automática não se lhes aplica, e a duração do fade só conta para o Stop com fade.` |

`settings-bit-perfect-hint`, replaced:

| File | New text |
|---|---|
| `en-US` | A special mode for audiophile playback, not for mixed programmes. The device is opened with exclusive access and follows each track's sample rate. The players that play on it never crossfade or mix: each track plays to its end, then the next one starts. When the sample rate changes and nothing else plays on the device, there is a short silence while it reopens. Their volume is held at 100 %. With no fade and nothing else on the same outputs, the samples reach the device unchanged; the BP badge lights while that is true. |
| `es-ES` | Un modo especial para la escucha audiófila, no para programas con mezclas. El dispositivo se abre en acceso exclusivo y sigue la frecuencia de muestreo de cada pista. Los reproductores que salen por él nunca encadenan ni mezclan pistas: cada pista suena hasta el final y luego empieza la siguiente. Si cambia la frecuencia de muestreo y no suena nada más en el dispositivo, hay un breve silencio mientras se vuelve a abrir. Su volumen queda fijo al 100 %. Sin fundidos y sin nada más en las mismas salidas, las muestras llegan al dispositivo sin cambios; el distintivo BP se enciende mientras sea así. |
| `ca-ES` | Un mode especial per a l'escolta audiòfila, no per a programes amb mescles. El dispositiu s'obre amb accés exclusiu i segueix la freqüència de mostreig de cada pista. Els reproductors que hi sonen mai no encadenen ni mesclen pistes: cada pista sona fins al final i després comença la següent. Si canvia la freqüència de mostreig i no hi sona res més, hi ha un breu silenci mentre es torna a obrir. El seu volum queda fix al 100 %. Sense fos i sense res més a les mateixes sortides, les mostres arriben al dispositiu sense canvis; el distintiu BP s'encén mentre és així. |
| `de-DE` | Ein Sondermodus für audiophile Wiedergabe, nicht für gemischte Programme. Das Gerät wird mit exklusivem Zugriff geöffnet und folgt der Abtastrate jedes Tracks. Die Player, die darauf spielen, überblenden und mischen nie: Jeder Track läuft bis zu seinem Ende, dann beginnt der nächste. Ändert sich die Abtastrate und spielt sonst nichts auf dem Gerät, entsteht eine kurze Stille, während es neu geöffnet wird. Ihre Lautstärke ist fest auf 100 %. Ohne Blende und ohne andere Quellen auf denselben Ausgängen erreichen die Samples das Gerät unverändert; solange das gilt, leuchtet das BP-Symbol. |
| `eu-ES` | Entzute audiofiloarentzako modu berezi bat, ez nahasketak dituzten programetarako. Gailua sarbide esklusiboarekin irekitzen da eta pista bakoitzaren lagintze-maiztasuna jarraitzen du. Bertan entzuten diren erreproduzigailuek ez dute inoiz pistarik kateatzen edo nahasten: pista bakoitza amaierara arte entzuten da, eta gero hurrengoa hasten da. Lagintze-maiztasuna aldatzen bada eta gailuan beste ezer entzuten ez bada, isilune labur bat dago berriro irekitzen den bitartean. Haien bolumena % 100ean finkatuta geratzen da. Itzaltzerik gabe eta irteera berberetan beste ezer gabe, laginak aldaketarik gabe iristen dira gailura; hala den bitartean, BP ikurra pizten da. |
| `fr-FR` | Un mode spécial pour l'écoute audiophile, pas pour les programmes mixés. Le périphérique est ouvert en accès exclusif et suit la fréquence d'échantillonnage de chaque piste. Les lecteurs qui y sont diffusés n'enchaînent ni ne mixent jamais : chaque piste est jouée jusqu'à sa fin, puis la suivante commence. Quand la fréquence d'échantillonnage change et que rien d'autre n'est diffusé sur le périphérique, un bref silence se produit pendant sa réouverture. Leur volume est bloqué à 100 %. Sans fondu et sans rien d'autre sur les mêmes sorties, les échantillons parviennent intacts au périphérique ; le badge BP s'allume tant que c'est le cas. |
| `gl-ES` | Un modo especial para a escoita audiófila, non para programas con mesturas. O dispositivo ábrese con acceso exclusivo e segue a frecuencia de mostraxe de cada pista. Os reprodutores que soan nel nunca encadean nin mesturan pistas: cada pista soa ata o final e despois comeza a seguinte. Se cambia a frecuencia de mostraxe e non soa nada máis no dispositivo, hai un breve silencio mentres se volve abrir. O seu volume queda fixo ao 100 %. Sen fundido e sen nada máis nas mesmas saídas, as mostras chegan ao dispositivo sen cambios; o distintivo BP acéndese mentres sexa así. |
| `it-IT` | Una modalità speciale per l'ascolto audiofilo, non per programmi mixati. Il dispositivo viene aperto in accesso esclusivo e segue la frequenza di campionamento di ogni brano. I lettori che vi suonano non concatenano né mixano mai i brani: ogni brano suona fino alla fine, poi inizia il successivo. Quando la frequenza di campionamento cambia e sul dispositivo non suona nient'altro, c'è un breve silenzio mentre viene riaperto. Il loro volume è bloccato al 100 %. Senza dissolvenza e senza altro sulle stesse uscite, i campioni arrivano al dispositivo inalterati; il badge BP si accende finché è così. |
| `nl-NL` | Een speciale modus voor audiofiel afspelen, niet voor gemixte programma's. Het apparaat wordt met exclusieve toegang geopend en volgt de samplefrequentie van elk nummer. De spelers die erop spelen, overfaden of mixen nooit: elk nummer speelt tot het einde, daarna begint het volgende. Als de samplefrequentie verandert en er verder niets op het apparaat speelt, valt er een korte stilte terwijl het opnieuw wordt geopend. Hun volume staat vast op 100 %. Zonder fade en zonder iets anders op dezelfde uitgangen bereiken de samples het apparaat ongewijzigd; de BP-badge brandt zolang dat zo is. |
| `pl-PL` | Specjalny tryb do odsłuchu audiofilskiego, nie do programów z miksami. Urządzenie jest otwierane z dostępem na wyłączność i dostosowuje się do częstotliwości próbkowania każdego utworu. Odtwarzacze, które na nim grają, nigdy nie przenikają ani nie miksują utworów: każdy utwór gra do końca, a potem zaczyna się następny. Gdy zmienia się częstotliwość próbkowania i nic innego nie gra na urządzeniu, następuje krótka cisza, gdy jest ono ponownie otwierane. Ich głośność jest zablokowana na 100 %. Bez wyciszania i bez niczego innego na tych samych wyjściach próbki docierają do urządzenia bez zmian; wskaźnik BP świeci, dopóki tak jest. |
| `pt-PT` | Um modo especial para audição audiófila, não para programas com misturas. O dispositivo é aberto com acesso exclusivo e segue a frequência de amostragem de cada faixa. Os leitores que tocam nele nunca encadeiam nem misturam faixas: cada faixa toca até ao fim e depois começa a seguinte. Quando a frequência de amostragem muda e nada mais toca no dispositivo, há um breve silêncio enquanto é reaberto. O seu volume fica fixo a 100 %. Sem fade e sem mais nada nas mesmas saídas, as amostras chegam ao dispositivo sem alterações; o indicador BP acende enquanto assim for. |

Each is one line `settings-bit-perfect-hint = <text>` (Fluent: a leading space after `=` is dropped; no line breaks).

`dsd_ui.rs` and `settings_layout.rs` may match parts of the old hint ("Exclusive access, and the device follows…"); `grep -rn "Exclusive access, and the device" crates/fp-app/tests` and update any match to a phrase of the new text.

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p fp-app --test bit_perfect_ui && cargo test -p fp-app --test i18n && cargo test -p fp-app --test settings_layout`
Expected: PASS.

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-app
git commit -m "feat(ui): say in Settings what bit-perfect output costs

The bit-perfect hint says it is a special mode, that its players never
crossfade, that a change of rate leaves a short silence and that their
volume is held at 100 %; Settings > Players notes that the fade time and
automatic mix do not apply to them (bit-perfect rate and fader spec BF9,
BR15).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: Apply now names the faders it resets (BF8, §6.2)

**Files:**
- Modify: `crates/fp-app/src/ui/pending.rs` (`show_confirm`, after the `interruptions` loop, ~L394)
- Modify: every locale file: `apply-now-fader-reset` after `apply-now-line` (L581 in en-US, L593 in pl-PL)
- Test: `crates/fp-app/tests/live_settings.rs`

**Interfaces:**
- Consumes: `fp_model::faders_reset_by(&AppState) -> Vec<PlayerId>` (plan 1); `player_number` (`pending.rs` L20).

- [ ] **Step 1: Write the failing tests**

`tests/live_settings.rs` (it already has `playing`, `null`, `open_settings_and_apply`, `CONFIRM`):

```rust
/// P1 playing on `null` at `volume`, and bit-perfect wanted for `null`.
fn bit_perfect_wanted(volume: f32) -> (Harness<'static, AppUi>, std::sync::Arc<Fake>) {
    let mut s = playing(2);
    let p = s.players[0].id;
    fp_model::apply(&mut s, Command::SetVolume(p, volume)).unwrap();
    let (mut h, fake) = harness(s);
    let mut config = fake.state.load().config.clone();
    config.outputs.bit_perfect = vec![null("null")];
    fake.send(Command::UpdateConfig(Box::new(config)));
    fake.take_sent();
    h.run_steps(2);
    (h, fake)
}

#[test]
fn bf8_apply_now_names_the_faders_it_resets() {
    let (mut h, _fake) = bit_perfect_wanted(0.5);
    open_settings_and_apply(&mut h);
    assert!(h.query_by_label(CONFIRM).is_some());
    assert!(
        h.query_by_label("The volume of P1 goes to 100 % (bit-perfect output).")
            .is_some()
    );
}

#[test]
fn bf8_no_fader_line_when_every_fader_is_at_full() {
    let (mut h, _fake) = bit_perfect_wanted(1.0);
    open_settings_and_apply(&mut h);
    assert!(h.query_by_label(CONFIRM).is_some());
    assert!(h.query_by_label_contains("goes to 100 %").is_none());
}
```

(Both players are on `null` in `reported`; P2 is at 100 %, so only P1 is named.)

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p fp-app --test live_settings bf8 2>&1 | tail -5`
Expected: FAIL — `bf8_apply_now_names_the_faders_it_resets` finds no such line.

- [ ] **Step 3: Implement**

`show_confirm`, after the `for (target, causes) in &interruptions { … }` loop:

```rust
            // BF8: the faders Apply now would lock while below 100 %.
            let reset = fp_model::faders_reset_by(state);
            if !reset.is_empty() {
                let players = reset
                    .iter()
                    .map(|p| t.tr_args("player-label", &[("n", player_number(state, *p).into())]))
                    .collect::<Vec<_>>()
                    .join(", ");
                text(
                    ui,
                    t.tr_args("apply-now-fader-reset", &[("players", players.into())]),
                    12.0,
                    theme::AMBER,
                );
            }
```

Locales, after `apply-now-line` (spec §6.2, verbatim):

| File | Line |
|---|---|
| `en-US` | `apply-now-fader-reset = The volume of { $players } goes to 100 % (bit-perfect output).` |
| `es-ES` | `apply-now-fader-reset = El volumen de { $players } pasa al 100 % (salida bit-perfect).` |
| `ca-ES` | `apply-now-fader-reset = El volum de { $players } passa al 100 % (sortida bit-perfect).` |
| `de-DE` | `apply-now-fader-reset = Die Lautstärke von { $players } geht auf 100 % (Bit-perfect-Ausgang).` |
| `eu-ES` | `apply-now-fader-reset = { $players }(r)en bolumena % 100era igotzen da (bit-perfect irteera).` |
| `fr-FR` | `apply-now-fader-reset = Le volume de { $players } passe à 100 % (sortie bit-perfect).` |
| `gl-ES` | `apply-now-fader-reset = O volume de { $players } pasa ao 100 % (saída bit-perfect).` |
| `it-IT` | `apply-now-fader-reset = Il volume di { $players } passa al 100 % (uscita bit-perfect).` |
| `nl-NL` | `apply-now-fader-reset = Het volume van { $players } gaat naar 100 % (bit-perfect uitgang).` |
| `pl-PL` | `apply-now-fader-reset = Głośność { $players } zmienia się na 100 % (wyjście bit-perfect).` |
| `pt-PT` | `apply-now-fader-reset = O volume de { $players } passa para 100 % (saída bit-perfect).` |

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p fp-app --test live_settings && cargo test -p fp-app --test i18n`
Expected: PASS.

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-app
git commit -m "feat(ui): name the faders Apply now resets to 100 %

When forcing pending output changes would put a player below 100 % on a
bit-perfect output, the confirmation says its volume goes to 100 %
(bit-perfect rate and fader spec BF8).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: The user guide, the README and the UI page (§8, §9.1, §9.2)

**Files** (English only; `docs/i18n/` is not touched):
- Modify: `docs/user/bit-perfect.md`:
  - **Introduction** (L3-6): add that it is a special mode for audiophile playback (archive listening, a hi-fi or monitoring chain, checking a digital link), not the default and not meant for a mixed music programme.
  - **New section `## What it costs`** right after the introduction and before `## Setting a device bit-perfect` (L8), a plain list for every player whose Main output is on the device, with the five bullets of spec §9.1 in order: no crossfades and no segues (each track plays to its end — its cue-out — then the next starts; **Automatic mix at the MIX point** and the MIX point are not used; **Play / Next** and **Previous** while on air switch at once, with a click-free stop); a **short silence** between two tracks whose sample rates differ while the device reopens (usually a few tens of milliseconds, longer on some devices), none between tracks at the same rate, which follow each other with no gap; the **volume fader is held at 100 %** on screen, on MIDI and over the remote API and OSC, and goes to 100 % when bit-perfect is turned on; **what still works** (Fade stop over the fade time, Pause, Stop, Restart, repeat and stop-after marks, Single and Continuous modes, Cue — not locked —, carts — they play, and while one plays on the device the next track is not bit-perfect and no silence is inserted —, DSD output, and every player whose Main output is on another device, which mixes as usual); **when to use it**, and when not.
  - **"What happens on a bit-perfect device"** (L40-64): rewrite "The rate follows the file" for hard transitions and rate cuts — when the device reopens between two tracks, and when the next track is resampled instead (another player, a cart or a CUE on the device, a rate the device refused, a track not analysed yet); remove the example "a 48 kHz track mixed into after a 44.1 kHz one". In "No processing, when nothing asks for it", "the player's volume is at 100 %" becomes "the player's volume, held at 100 % on a bit-perfect output".
  - **"DSD"** (L66 onwards): remove the "volume must stay at 100 %: the fader's tooltip says so. Moving it switches the track to PCM" line and the two "moving the fader" remarks; "Pressing Play on another track while it plays cuts it the same way instead of crossfading" and "The player's own next track does not overlap" are said once, in "What it costs"; "An album does not stay DSD" loses "a segue or a crossfade".
  - **"The BP badge"** (L163 onwards): "volume is 100 %" becomes "the volume is held at 100 % on a bit-perfect output"; "no fade runs" names Fade stop and the pause ramp, the only fades left.
  - **"Checking it yourself"** (L188 onwards): the two loopback steps of spec §8 — a 44.1 kHz track followed by a 96 kHz one in continuous mode with automatic mix on and a MIX point (the first plays to its cue-out with no fade, the converter shows 96 kHz on the second after a short silence, the BP badge lights on both); then two tracks at the same rate (the second follows with no gap and no overlap).
- Modify: `docs/user/players.md`: the **Mixing** badge row (L14) "never on a player whose Main output is bit-perfect"; the **Volume fader** bullet (in "Info row" or the meter column section) "locked at 100 % on a bit-perfect output, with a tooltip saying so"; the Transport rows **Play / Next** (L96) and **Previous** (L101) "on a bit-perfect output they switch at once, without a fade"; the **Waveform** section (L126 onwards) "the MIX marker is drawn dim on a bit-perfect player, as in SINGLE mode". Link each to [What it costs](bit-perfect.md#what-it-costs).
- Modify: `docs/user/markers-and-mixing.md`, "What the player does with them" (L71 onwards): a bullet that a player on a bit-perfect output ignores the MIX point and the automatic mix, and that Play while on air does not fade there, with the link.
- Modify: `docs/user/settings.md`: in **Players** (L103 onwards) the **Fade time** and **Automatic mix at the MIX point** rows say they do not apply to players on a bit-perfect output (except Fade stop), and mention the note under them; in **Audio outputs** (L58 onwards) the **Bit-perfect: *device*** row says it is a special mode, the players on it never crossfade, a change of rate leaves a short silence and their volume is held at 100 %, linking to "What it costs".
- Modify: `docs/user/troubleshooting.md`, "Bit-perfect" (L21 onwards): "A short silence before a track" explains the rate change and that it happens only when the rate changes; new entries "Tracks do not mix, and Play does not fade" and "The volume fader does not move", both pointing to "What it costs".
- Modify: `docs/user/README.md` L19, the Bit-perfect output row: add "a special mode: no crossfades, volume at 100 %".
- Modify: `docs/user/midi.md` and `docs/user/remote-control.md`: if plan 3 linked `bit-perfect.md` without the anchor, add `#what-it-costs` now.
- Modify: `README.md` L91-94, the **Bit-perfect output** feature line: a special mode; the device follows each track's rate, with a short silence when it changes; tracks follow each other without crossfades; the volume is held at 100 %.
- Modify: `docs/technical/ui.md`: the `ui/widgets.rs` row (the fader's `enabled` flag), the `ui/player.rs` row (the locked fader, the Play/Next tooltip `tip-next-bit-perfect`, `view::mix_active` for the dim MIX marker), the `ui/settings.rs` row (the Players note), the `ui/pending.rs` row (the fader line of Apply now, `faders_reset_by`).

- [ ] **Step 1: Edit the documents** as listed. Keep the guide's voice (short sentences, the operator's words, bold UI names).
- [ ] **Step 2: Check the site**

Run: `scripts/site/build.sh /tmp/fp-site-check && scripts/site/check-links.sh /tmp/fp-site-check` (needs `gh` or `FAUSTE_RELEASE_JSON`; if neither is available, run `scripts/site/mdbook.sh build docs/user` if the script supports it, or check each new link and anchor by hand: the anchor of `## What it costs` is `#what-it-costs`).
Expected: no dead links.

- [ ] **Step 3: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add docs README.md
git commit -m "docs: explain what bit-perfect output costs

The guide says, before the operator turns it on, that bit-perfect output
is a special mode: no crossfades, a short silence when the rate changes,
the volume held at 100 %, and what still works; the players, markers,
settings and troubleshooting pages, the README and the UI page say the
same.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

## Self-review notes

- Spec coverage (this plan): BF7, `tip-volume-dsd` removal → Task 1; BR15 tooltip and MIX marker, §6.4 → Task 2; BR15 Settings note, BF9, §6.3, §6.5 → Task 3; BF8 interface, §6.2 → Task 4; §8 loopback steps, §9.1 (all pages but `midi.md` and `remote-control.md`, plan 3), §9.2 `README.md` and `ui.md` → Task 5. §6.1 `tip-volume-locked` came with plan 1.
- Spec coverage (all four plans): BR1 → P1 T1, P2 T1; BR2, BR3 → P1 T2; BR4, BR5 → P1 T3; BR6 → P2 T2–T3; BR7 → P1 T4; BR8 → P2 T4; BR9 → P1 T4; BR10 → P1 T3, P2 T4; BR11 → P2 T4–T5; BR12 → P1 T3; BR13 → P2 T4; BR14 → P1 T3, P2 T5; BR15 → P4 T2–T3; BR16 → P3 T1–T2; BF1 → P1 T2; BF2–BF6 → P1 T5; BF7 → P4 T1; BF8 → P1 T6, P4 T4; BF9 → P4 T3; BF10, BF11 → P3 T1; BF12, BF13 → P3 T2; §5.5 → P1 T5, T7; §6.1 → P1 T5; §6.2 → P4 T4; §6.3 → P4 T3; §6.4 → P4 T2; §6.5 → P4 T3; §8 loopback → P4 T5; §9.1 → P3 T3, P4 T5; §9.2 → P2 T5 (`audio-engine.md`, `audio-path-audit.md`), P3 T3 (`remote-api.md`), P4 T5 (`README.md`, `ui.md`); §9.3 → P1 T7; §9.4 → P1 T5, P4 T1–T4.
