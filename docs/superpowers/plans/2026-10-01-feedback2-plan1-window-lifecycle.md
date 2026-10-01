# Window and Lifecycle (Feedback 2, Plan 1) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Confirm before closing while audio is on air (O6), give Linux Wayland windows the standard title-bar buttons (O14), and add a visible About button (O20).

**Architecture:**
- **The rule.** A pure model function, `fp_model::on_air`, lists what is
  sounding.
- **The guard.** The UI intercepts the viewport's close request. When `on_air`
  is not empty, it cancels the close and shows a modal from a new
  `ui/exit_guard.rs`. "Stop and close" sends the stop commands, marks the
  close as confirmed and closes the viewport. The existing shutdown in
  `main.rs` then saves the session.
- **O14.** eframe is built with `default-features = false`, so winit's
  `wayland-csd-adwaita` feature is off, and winit draws its bare fallback
  frame. Enabling that feature through a Linux-only winit dependency gives the
  Adwaita-style buttons.

**Tech Stack:** Rust, egui/eframe 0.36.2, egui_kittest, winit 0.30 (sctk-adwaita), Fluent.

**Spec:** `docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md` §2 (and §12).

## Global Constraints

- All code, comments and docs in English. UI strings go in both `crates/fp-app/locales/en-US/main.ftl` and `es-ES/main.ftl`.
- Never name other playout or radio-automation products.
- `unsafe_code` is forbidden. Outside tests, no `unwrap`, `expect`, `panic` or indexing: use `get`.
- Behaviour lives in `fp-model`; the UI only displays and sends commands. The UI never blocks.
- Nothing goes on air by itself after a start, restart or crash.
- TDD: every new test is seen failing before the code that makes it pass.
- Before each commit: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace` all pass.
- Conventional Commits; every commit ends with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Ledger: `.superpowers/sdd/feedback2-plan1/progress.md`; record deviations as `Ruling: <decision> — <why> — <cost if wrong>`.
- Branch: `feat/window-lifecycle`, from an up-to-date `master`.

## Review Focus

- A close request arrives again while the dialog is already open (the operator clicks the window ✕ twice). Expected: one dialog, still cancelled, nothing stopped.
- A close request while the Settings or About modal is open and audio is on air. Expected: the guard still shows. It takes precedence over the other modals.
- A player that is Paused (not Playing). Expected: it counts as on air, and it is stopped on confirm.
- A player whose current entry was removed from the library (no title). Expected: the dialog lists it with the fallback "P1", with no crash.
- Only the cartwall CUE or a player CUE is running. Expected: no dialog; the app closes.

---

### Task 1: `on_air` rule in the model

**Files:**
- Create: `crates/fp-model/src/on_air.rs`
- Modify: `crates/fp-model/src/lib.rs` (add `pub mod on_air;` and `pub use on_air::{OnAir, on_air};`)
- Test: `crates/fp-model/tests/on_air.rs`

**Interfaces:**
- Produces:
  - `pub enum OnAir { Player { player: PlayerId, entry: Option<EntryId> }, Cart(CartId) }`, deriving `Debug, Clone, Copy, PartialEq, Eq`;
  - `pub fn on_air(state: &AppState) -> Vec<OnAir>`, listing players in display order first, then carts in firing order.

- [ ] **Step 1: Write the failing tests**

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback 2 spec O6: what the close guard treats as on air.

mod common;

use common::{fixture, p0};
use fp_model::{Command, OnAir, PlayingCart, apply, on_air};

#[test]
fn a_stopped_state_has_nothing_on_air() {
    let s = fixture(3);
    assert!(on_air(&s).is_empty());
}

#[test]
fn playing_and_paused_players_are_on_air_with_their_entry() {
    let mut s = fixture(3);
    let p = p0(&s);
    apply(&mut s, Command::Play(p)).unwrap();
    let entry = s.players[0].current;
    assert!(entry.is_some());
    assert_eq!(on_air(&s), vec![OnAir::Player { player: p, entry }]);
    apply(&mut s, Command::Pause(p)).unwrap();
    assert_eq!(on_air(&s), vec![OnAir::Player { player: p, entry }]);
}

#[test]
fn playing_carts_are_on_air_after_the_players() {
    let mut s = fixture(3);
    let p = p0(&s);
    let cart = s.cartwall.pages[0].carts[0].id;
    s.cartwall.playing.push(PlayingCart { cart, looped: false });
    apply(&mut s, Command::Play(p)).unwrap();
    let entry = s.players[0].current;
    assert_eq!(
        on_air(&s),
        vec![OnAir::Player { player: p, entry }, OnAir::Cart(cart)]
    );
}

#[test]
fn a_cue_alone_is_not_on_air() {
    let mut s = fixture(3);
    let p = p0(&s);
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    assert!(s.players[0].cue.is_some());
    let cart = s.cartwall.pages[0].carts[0].id;
    s.cartwall.cue = Some(cart);
    assert!(on_air(&s).is_empty());
}
```

If `fixture`'s cart pages are empty (`carts` has no element 0), build a page
the way `crates/fp-model/tests/cartwall.rs` does, and note it in the ledger.

- [ ] **Step 2: Run the tests and check that they fail**

Run: `cargo test -p fp-model --test on_air`
Expected: compile error, because `on_air` and `OnAir` are unresolved imports.

- [ ] **Step 3: Implement**

```rust
//! What is on air (feedback 2 spec O6): what closing or restarting the
//! application would cut. A CUE is pre-listening, not on air.

use crate::{AppState, CartId, EntryId, PlayerId, Transport};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnAir {
    /// A player that is playing or paused, with its current entry.
    Player {
        player: PlayerId,
        entry: Option<EntryId>,
    },
    /// A playing cart.
    Cart(CartId),
}

/// Players in display order, then carts in firing order.
pub fn on_air(state: &AppState) -> Vec<OnAir> {
    let players = state
        .players
        .iter()
        .filter(|p| p.transport != Transport::Stopped)
        .map(|p| OnAir::Player {
            player: p.id,
            entry: p.current,
        });
    let carts = state.cartwall.playing.iter().map(|c| OnAir::Cart(c.cart));
    players.chain(carts).collect()
}
```

Check the transport field's name in `PlayerState` (`crates/fp-model/src/player.rs`) and use it.

- [ ] **Step 4: Run the tests and check that they pass**

Run: `cargo test -p fp-model --test on_air`
Expected: 4 passed.

- [ ] **Step 5: Commit**

```bash
git add crates/fp-model/src/on_air.rs crates/fp-model/src/lib.rs crates/fp-model/tests/on_air.rs
git commit -m "feat(model): list what is on air

The close guard needs to know what closing would cut: players that are
playing or paused, and playing carts. A CUE is not on air.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: The close guard

**Files:**
- Create: `crates/fp-app/src/ui/exit_guard.rs`
- Modify: `crates/fp-app/src/ui/mod.rs` (add `pub mod exit_guard;`, following how `about` is declared)
- Modify: `crates/fp-app/src/ui/app.rs`:
  - `ViewState` gains `pub exit_guard: Option<ExitIntent>` and `pub close_confirmed: bool`;
  - `AppUi::ui` checks the close request each frame, right after `let state = self.ctl.model();`;
  - the guard modal is drawn before every other modal, and its Esc handling comes first in the keyboard block (the one near `if self.view.settings_open { if escape …`).
- Modify: both `main.ftl` files
- Test: `crates/fp-app/tests/exit_guard.rs`

**Interfaces:**
- Consumes: `fp_model::{on_air, OnAir}` (Task 1).
- Produces, in `ui/exit_guard.rs`:
  - `pub enum ExitIntent { Close }`, deriving `Debug, Clone, Copy, PartialEq, Eq`. Plan 2 adds `Restart`.
  - `pub(crate) fn stop_commands(items: &[OnAir]) -> Vec<Command>`: a `Command::Stop(player)` for each player item, then one `Command::StopAllCarts` if any cart item exists.
  - `pub(crate) fn show(ctx: &egui::Context, scene: &Scene<'_>, intent: ExitIntent, items: &[OnAir]) -> Option<bool>`: draws the modal. Returns `Some(true)` on confirm, `Some(false)` on cancel, and `None` while it stays open.

**Behaviour in `AppUi::ui`:**

```rust
// O6: a close request while something is on air waits for the operator.
let close_requested = ctx.input(|i| i.viewport().close_requested());
if close_requested && !self.view.close_confirmed {
    if fp_model::on_air(&state).is_empty() {
        // Nothing to cut: let the window close.
    } else {
        ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        self.view.exit_guard = Some(ExitIntent::Close);
    }
}
```

When the modal returns `Some(true)`:
- send every command from `stop_commands(&on_air(&state))`;
- set `close_confirmed = true` and clear `exit_guard`;
- call `ctx.send_viewport_cmd(egui::ViewportCommand::Close)`.

When it returns `Some(false)` or Esc is pressed, clear `exit_guard`. While
`exit_guard` is `Some`, keyboard shortcuts are ignored, as they are for About.

**Strings** (both locales; the Spanish ones are given):

```
exit-guard-title = Audio is on air
exit-guard-close-body = Closing stops everything that is playing:
exit-guard-cancel = Cancel
exit-guard-close-confirm = Stop and close
on-air-player = P{ $n } — { $title }
on-air-player-empty = P{ $n }
on-air-cart = Cart — { $title }
```

```
exit-guard-title = Hay audio en el aire
exit-guard-close-body = Al cerrar se detendrá todo lo que está sonando:
exit-guard-cancel = Cancelar
exit-guard-close-confirm = Detener y cerrar
on-air-player = P{ $n } — { $title }
on-air-player-empty = P{ $n }
on-air-cart = Cartucho — { $title }
```

The list item for a player:
- `$n` is the 1-based position of the player in `state.players`;
- `$title` is `state.track_for_entry(entry)?.title`;
- `on-air-player-empty` is used when there is no entry or no track.

The list item for a cart:
- `$title` is the cart's name, or the track's title when the name is empty.

Draw the modal the way `ui/about.rs::show` does: `egui::Modal`, `theme::SURFACE`, tiles drawn with `widgets::tile`. Cancel comes first, and the confirm tile uses `theme::ON_AIR_ROW` as its border colour.

- [ ] **Step 1: Write the failing tests**

```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Feedback 2 spec O6: closing while audio is on air asks first.

mod support;

use egui::{Key, ViewportCommand, ViewportEvent, ViewportId};
use egui_kittest::kittest::Queryable;
use fp_model::{Command, PlayingCart};
use support::{harness, state};

const GUARD: &str = "Audio is on air";

fn request_close(h: &mut egui_kittest::Harness<'static, fp_app::ui::app::AppUi>) {
    h.input_mut()
        .viewports
        .entry(ViewportId::ROOT)
        .or_default()
        .events
        .push(ViewportEvent::Close);
    h.run_steps(2);
}

fn sent_viewport_command(
    h: &egui_kittest::Harness<'static, fp_app::ui::app::AppUi>,
    wanted: &ViewportCommand,
) -> bool {
    h.output()
        .viewport_output
        .get(&ViewportId::ROOT)
        .is_some_and(|v| v.commands.contains(wanted))
}

fn playing(players: usize) -> fp_model::AppState {
    let mut s = state(players, 3);
    let p = s.players[0].id;
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    s
}

#[test]
fn closing_with_nothing_on_air_shows_no_dialog() {
    let (mut h, fake) = harness(state(1, 3));
    request_close(&mut h);
    assert!(h.query_by_label(GUARD).is_none());
    assert!(!sent_viewport_command(&h, &ViewportCommand::CancelClose));
    assert!(fake.take_sent().is_empty());
}

#[test]
fn closing_while_playing_cancels_and_asks() {
    let (mut h, fake) = harness(playing(1));
    request_close(&mut h);
    assert!(h.query_by_label(GUARD).is_some());
    assert!(h.query_by_label_contains("P1 — Song 1").is_some());
    assert!(fake.take_sent().is_empty());
}

#[test]
fn cancel_and_escape_keep_everything_playing() {
    let (mut h, fake) = harness(playing(1));
    request_close(&mut h);
    h.get_by_label("Cancel").click();
    h.run_steps(2);
    assert!(h.query_by_label(GUARD).is_none());
    request_close(&mut h);
    h.key_press(Key::Escape);
    h.run_steps(2);
    assert!(h.query_by_label(GUARD).is_none());
    assert!(fake.take_sent().is_empty());
}

#[test]
fn stop_and_close_stops_players_and_carts_then_closes() {
    let mut s = playing(2);
    let cart = s.cartwall.pages[0].carts[0].id;
    s.cartwall.playing.push(PlayingCart { cart, looped: false });
    let p = s.players[0].id;
    let (mut h, fake) = harness(s);
    request_close(&mut h);
    h.get_by_label("Stop and close").click();
    h.run_steps(1);
    assert_eq!(
        fake.take_sent(),
        vec![Command::Stop(p), Command::StopAllCarts]
    );
    assert!(sent_viewport_command(&h, &ViewportCommand::Close));
}

#[test]
fn a_second_close_request_keeps_one_dialog_and_stops_nothing() {
    let (mut h, fake) = harness(playing(1));
    request_close(&mut h);
    request_close(&mut h);
    assert_eq!(h.query_all_by_label(GUARD).count(), 1);
    assert!(fake.take_sent().is_empty());
}

#[test]
fn the_guard_shows_over_settings() {
    let (mut h, _) = harness(playing(1));
    h.get_by_label_contains("Settings").click();
    h.run_steps(2);
    request_close(&mut h);
    assert!(h.query_by_label(GUARD).is_some());
}

#[test]
fn shortcuts_do_nothing_while_the_guard_is_open() {
    let (mut h, fake) = harness(playing(1));
    request_close(&mut h);
    h.key_press(Key::Space);
    h.key_press(Key::Num1);
    h.run_steps(2);
    assert!(fake.take_sent().is_empty());
}
```

Also add a unit test of `stop_commands` with a paused player and two carts. It
expects `[Stop(p), StopAllCarts]`, with one `StopAllCarts` only. It can live
at the bottom of `exit_guard.rs` in `#[cfg(test)] mod tests`.

Two adjustments may be needed:
- The Fake does not run the reducer, so the state passed to `harness` must
  already be Playing. Check `Fake::model` in `tests/support/mod.rs`.
- If `h.output()` does not hold the viewport commands of the last step, read
  them after `h.step()` instead of `run_steps`, and note how in the ledger.

- [ ] **Step 2: Run the tests and check that they fail**

Run: `cargo test -p fp-app --test exit_guard`
Expected: the tests that expect a dialog fail, because no "Audio is on air" label exists.

- [ ] **Step 3: Implement `exit_guard.rs`, the strings and the wiring in `app.rs` as described above**

- [ ] **Step 4: Run the tests and check that they pass**

Run: `cargo test -p fp-app --test exit_guard && cargo test -p fp-app --test about --test main_screen`
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add crates/fp-app/src/ui/exit_guard.rs crates/fp-app/src/ui/mod.rs crates/fp-app/src/ui/app.rs crates/fp-app/locales crates/fp-app/tests/exit_guard.rs
git commit -m "feat(ui): ask before closing while audio is on air

Closing the window used to cut whatever was playing. Now the close is
held and the operator confirms, seeing what will stop.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: About button in the top bar

**Files:**
- Modify: `crates/fp-app/src/ui/app.rs` (`top_bar`, the right-to-left block: add the tile after the Settings tile, which places it to the Settings button's left)
- Modify: both `main.ftl` (no new key: reuse `tip-about`)
- Test: `crates/fp-app/tests/about.rs`

**Interfaces:**
- Consumes: `view_state.about_open` (exists).
- The button is a 24 × 24 tile showing `egui_phosphor::regular::INFO`. Its accessible label and hover text are `tip-about` ("About Fauste Player").
- The click area on the name and version stays. To keep two widgets from sharing one accessible label, rename the label of the name and version area to a new key, `tip-about-name`, with the same text in both locales. The `ABOUT` constant in the test then finds the button.

- [ ] **Step 1: Write the failing test** (add it to `tests/about.rs`)

```rust
#[test]
fn the_info_button_sits_next_to_settings_and_opens_about() {
    let (mut h, _) = harness(state(1, 1));
    let button = h.get_by_label(ABOUT);
    let settings = h.get_by_label_contains("Settings");
    let (b, s) = (button.rect(), settings.rect());
    assert!(b.right() <= s.left() && (b.center().y - s.center().y).abs() < 2.0);
    button.click();
    h.run_steps(2);
    assert!(h.query_by_label_contains("All rights reserved").is_some());
}
```

Update `clicking_the_name_opens_about_and_escape_closes_it` to click the
name's own label (`tip-about-name`).

- [ ] **Step 2: Run the test and check that it fails**

Run: `cargo test -p fp-app --test about`
Expected: the new test fails. `get_by_label(ABOUT)` finds the name area, which lies left of Settings, far from it, so the position assertion fails.

- [ ] **Step 3: Implement the button and the label rename**

- [ ] **Step 4: Run the tests and check that they pass**

Run: `cargo test -p fp-app --test about`
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git commit -am "feat(ui): an About button in the top bar

About opened only from the app name, which nobody found.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: Standard window buttons on Wayland

**Files:**
- Modify: `crates/fp-app/Cargo.toml`
- Modify: `Cargo.toml` (workspace) only if workspace dependencies hold winit; otherwise keep it local
- Modify: `deny.toml` only if `cargo deny check` asks for a licence that is not listed
- Modify: `docs/technical/ui.md`

**Root cause (verified while planning):**
- `eframe` is declared with `default-features = false`.
- winit's `wayland-csd-adwaita` feature comes only from `winit/default`, through eframe's `default`.
- On a compositor without server-side decorations (GNOME), winit therefore draws sctk's bare fallback frame. `sctk-adwaita` is not even in the lock file.

- [ ] **Step 1: Check that sctk-adwaita is missing**

Run: `cargo tree -p fp-app -i sctk-adwaita`
Expected: an error saying the package `sctk-adwaita` is not found.

- [ ] **Step 2: Add the dependency**

In `crates/fp-app/Cargo.toml`:

```toml
# Wayland compositors without server-side decorations (GNOME) need winit's
# Adwaita-style frame; eframe is built without its default features, which
# would otherwise bring it.
[target.'cfg(target_os = "linux")'.dependencies]
winit = { version = "0.30", default-features = false, features = ["wayland-csd-adwaita"] }
```

Use the same `0.30.x` that `Cargo.lock` already resolves (`cargo tree -p fp-app -i winit`).

- [ ] **Step 3: Verify**

Run:
- `cargo tree -p fp-app -i sctk-adwaita`: now found;
- `cargo deny check`: passes. If a licence fails, add only that licence to `deny.toml`'s `allow` list and record it in the ledger.

Then run the full gate (fmt, clippy, test).

- [ ] **Step 4: Visual check by the maintainer**

Xvfb cannot show Wayland client-side decorations. Build with `cargo build
--release -p fp-app` and ask the maintainer to run `target/release/fauste-player`
on their GNOME Wayland desktop. They confirm that the title bar shows the
standard minimise, maximise and close buttons, and that maximise toggles. If
it does not, stop. Record the result as a `Ruling` and fall back to the
second option of the spec: start under XWayland on GNOME, by unsetting
`WAYLAND_DISPLAY` before eframe starts when `XDG_CURRENT_DESKTOP` contains
`GNOME`.

- [ ] **Step 5: Docs and commit**

In `docs/technical/ui.md`, add a short "Window decorations" paragraph: why
winit's Adwaita frame is enabled on Linux, and that Windows and macOS use
their native frames.

```bash
git add crates/fp-app/Cargo.toml Cargo.lock deny.toml docs/technical/ui.md
git commit -m "fix(app): standard window buttons on Wayland

Without winit's Adwaita frame, compositors that leave decorations to the
client got a bare frame with non-standard buttons.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: Docs and verification

**Files:**
- Modify: `docs/user/getting-started.md` (About button; closing while on air)
- Modify: `docs/user/players.md` or `troubleshooting.md` (wherever closing or quitting is described; grep "close" or "quit")
- Modify: `docs/technical/ui.md` (exit guard: where it sits in the frame, and that it takes precedence over the other modals)
- Modify: `docs/superpowers/specs/2026-09-25-fauste-player-design.md` §8 (top bar: the About button) and §3 if quitting is described
- Modify: `docs/superpowers/plans/2026-10-01-operator-feedback-2-roadmap.md` (plan 1 status: done)

- [ ] **Step 1:** Update the docs listed above. Keep them in plain English and describe behaviour in its own terms.
- [ ] **Step 2:** Run the full gate: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`. Expected: green.
- [ ] **Step 3:** Commit `docs: describe the close guard and the About button` with the co-author trailer.
