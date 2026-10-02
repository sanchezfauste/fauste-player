# Cartwall Stop (Feedback 2, Plan 4) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Every transport action has one icon, defined once in `ui/glyphs.rs` and used by the player, the cartwall and (plan 6) the CUE window (O18); and the cartwall bar gets a "Stop all (n)" button at its right end that stops every playing cart (O19).

**Architecture:**
- **One icon table.** A new module `ui/glyphs.rs` maps `TransportAction` (play, next, pause, stop, fade stop, stop after, restart, previous, cue) to either a Phosphor font glyph or a drawn icon from `ui/icons.rs`. It exposes `paint` (into a painter), `glyph_text` (a string for menu items and text labels), and the two lookups (`font_glyph`, `drawn`) that tests use. The player, the cartwall and the playlist context menu stop naming icon constants and call it. `ui/icons.rs` stays as the drawing primitives.
- **The rule already exists.** `Command::StopAllCarts` (`cart_rules::stop_all`) stops every playing cart through `stop_playing`, the same function the single stop uses, and also the cart cue. This plan only pins that with one more rule test. The UI sends the command and shows `cartwall.playing.len()`; it decides nothing.
- **The button.** The cartwall header becomes a right-to-left row: the "Stop all" tile first (so it is always at the right end and always visible, whatever the window width), then a nested left-to-right row with the title, the page tabs and the hint, which take what is left. Its label comes from two Fluent messages, with and without the count.

**Tech Stack:** Rust, egui/eframe 0.36.2, egui_kittest 0.36.2, egui_phosphor, Fluent. No new dependency.

**Spec:** `docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md` §5 (items O18 and O19). Roadmap: `docs/superpowers/plans/2026-10-01-operator-feedback-2-roadmap.md` (plan 4, branch `feat/cartwall-stop-all`; note that `Command::StopAllCarts` already exists). Earlier cartwall spec: `docs/superpowers/specs/2026-09-26-phase2-cartwall-settings-design.md` (C10 Stop all).

## Global Constraints

- All code, identifiers, comments, docs and commit messages are in English. Never mention other products.
- UI strings are Fluent messages in `crates/fp-app/locales/en-US/main.ftl` (the source) and `crates/fp-app/locales/es-ES/main.ftl`, always both (`tests/i18n.rs::both_locales_define_the_same_keys` enforces parity). Existing naming: `cartwall-title`, `cartwall-hint`, `menu-cart-stop`, `shortcut-stop-all-carts`.
- The UI never blocks and never panics: no `unwrap`, `expect` or indexing in `src/` (use `get`). The button only sends `Command::StopAllCarts` and reads the model snapshot.
- Behaviour lives in `fp-model`: the stop rule is the existing pure `apply` rule; the UI only executes and displays.
- Nothing goes on air by itself: this plan only stops sound.
- No hardcoded product limits: nothing here is an operator setting. The shortcut is already a `Config` entry (`shortcuts`), rebindable in Settings.
- Spec §5 O18: "One module, `ui/glyphs.rs`, maps each transport action (play, pause, stop, fade stop, stop after, restart, previous, cue) to its icon. The player, the cartwall and the CUE window (plan 6) draw icons only through it. The cart stop becomes the player's stop icon."
- Spec §5 O19: "A button at the right end of the cartwall bar, after the page and collapse controls. Its label is "Stop all", followed by the number of playing carts in parentheses ("Stop all (2)"). With no cart playing it is disabled and has no count. `Command::StopAllCarts` stops every playing cart the same way the individual stop does, with one rule test. `ShortcutAction::StopAllCarts` is added with no default key."
- Commands: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p <crate> --test <file> <name>` (one filter per command). Build only into the repo's `target/` (never set `CARGO_TARGET_DIR`).
- Commit only when fmt, clippy and the whole suite pass (`cargo test --workspace`). Commits end with the co-author trailer the harness provides.
- Documentation is part of the change (Task 3). `CHANGELOG.md` is never edited by hand; do not touch the roadmap table.

## Review Focus

Failure modes the spec implies and no obvious test covers; each has a test in the task named.

1. Carts playing on a page that is not shown: the count is the whole cartwall's, not the page's (the model's `playing` list is global). (Task 2, `stop_all_counts_carts_on_every_page`.)
2. A narrow window (420 px) with many page tabs: the button must stay fully visible at the right end, and the tabs and the hint give way, never the button. (Task 2, `stop_all_stays_at_the_right_end_in_a_narrow_window`.)
3. The strip collapsed: the header is always drawn, so the button must still show and work. (Task 2, `stop_all_works_with_the_cartwall_collapsed`.)
4. Carts stop by themselves or by the individual stop while the button shows a count: the label must drop to "Stop all" and the button must dim, with no stale count. (Task 2, `stop_all_dims_again_when_the_last_cart_stops`.)
5. Only a cart CUE pre-listen is running (nothing on air): the button stays dimmed (the spec counts playing carts), yet the shortcut still stops the cue, and the model does nothing and does not fail with nothing at all playing. (Task 2, `a_cart_cue_alone_does_not_enable_stop_all`; Task 1 model test `c10_stop_all_with_nothing_on_air_is_harmless_and_still_stops_the_cue`; Task 3 shortcut pins.)
6. A rebound or unbound `StopAllCarts` shortcut: the new key stops the carts and the old one does not; unbound does nothing. (Task 3, `stop_all_follows_a_rebound_shortcut`, `an_unbound_stop_all_shortcut_does_nothing`.)

## Decisions

- **The shortcut already exists, with a default key.** `ShortcutAction::StopAllCarts` is already in `fp-model`, dispatched in `ui/app.rs`, listed in Settings → Keyboard shortcuts, and bound to `Ctrl+Space` by default (`default_shortcuts`, documented in `docs/user/keyboard.md`, pinned by `fp-model/tests/shortcuts.rs`). The spec's "added with no default key" predates that. Removing the default would silently change every operator's keyboard and break a documented, tested behaviour, so this plan keeps `Ctrl+Space` and adds tests that pin rebinding and unbinding instead. (Cost if wrong: one line in `default_shortcuts` and its tests.)
- **Shortcut and button differ on an idle wall.** The button is dimmed when no cart is playing (spec), but the shortcut is not gated: `Command::StopAllCarts` also stops the cart CUE pre-listen, which is useful with nothing on air, and a no-op is harmless.
- **The count is `cartwall.playing.len()`.** Carts on every page; a cart in its de-click ramp has already left `playing`, so it is not counted.
- **Messages.** `cartwall-stop-all = Stop all` and `cartwall-stop-all-count = Stop all ({ $count })`; Spanish `Parar todo` and `Parar todo ({ $count })` (the cart menu already says "Parar"). The button's accessible name and tooltip are the same text.
- **Icon in the button.** The bar button shows the stop glyph from `glyphs` before its text, so it reads as the same stop as the player's.
- **The module's names.** The enum is `TransportAction`, not `Transport`, because `fp_model::Transport` is already imported in `ui/table.rs`. `Next` is the "play" button while a player is on air (fast-forward glyph); the spec's list has no separate entry for it, but the player draws it, so it must live in the module too.
- **Drawn icons keep their natural size.** `paint` centres a drawn icon in the rect at the size the player used (previous 16×12, restart 14×12, fade stop 16×12, stop after 18×13). The table's 16×12 "stop after" flag moves to `paint`; the drawing function fits itself inside the rect, so the change is under half a pixel.
- **A guard test reads the sources.** `tests/glyphs.rs` asserts that `player.rs`, `cartwall.rs` and `table.rs` no longer name the transport icon constants (`fill::STOP`, `fill::PLAY`, `fill::FAST_FORWARD`, `HEADPHONES`, `icons::`), so plan 6 and later cannot drift. `fill::PAUSE` and `SPEAKER_HIGH` in the table's row-status marker are a playback status, not an action, and stay.

## File Structure

- Create `crates/fp-app/src/ui/glyphs.rs`: `TransportAction`, `font_glyph`, `drawn`, `glyph_text`, `paint`.
- Modify `crates/fp-app/src/ui.rs`: `pub mod glyphs;`.
- Modify `crates/fp-app/src/ui/player.rs`, `ui/cartwall.rs`, `ui/table.rs`: draw through `glyphs`; cartwall also gets the button.
- Modify `crates/fp-app/locales/en-US/main.ftl`, `crates/fp-app/locales/es-ES/main.ftl`: two messages.
- Create `crates/fp-app/tests/glyphs.rs`. Modify `crates/fp-app/tests/cartwall_ui.rs`, `tests/main_screen.rs`, `tests/i18n.rs`, `crates/fp-model/tests/cartwall.rs`.
- Modify `docs/user/cartwall.md`, `docs/user/keyboard.md`, `docs/technical/ui.md`, the spec (note under §5).

---

### Task 1: One icon table for the transport actions (O18)

**Files:**
- Create: `crates/fp-app/src/ui/glyphs.rs`
- Modify: `crates/fp-app/src/ui.rs` (add `pub mod glyphs;` after `pub mod format;`)
- Modify: `crates/fp-app/src/ui/player.rs` (play tile near line 595, cue tile near line 238, cue readout near line 506, transport grid near lines 671-720, `use super::icons;` at line 15)
- Modify: `crates/fp-app/src/ui/cartwall.rs` (the cart menu near line 362)
- Modify: `crates/fp-app/src/ui/table.rs` (the stop-after flag near line 269, the context menu near lines 510 and 536, `use super::icons;` at line 11)
- Test: `crates/fp-app/tests/glyphs.rs` (new)

**Interfaces:**
- Consumes: `widgets::glyph(&Painter, Rect, &str, f32, Color32, bool)`, `icons::{fade_stop, stop_after, restart, previous}` (each `fn(Rect, Color32) -> Vec<Shape>`).
- Produces (used by Tasks 2 and 3 and by plan 6):

```rust
pub enum TransportAction { Play, Next, Pause, Stop, FadeStop, StopAfter, Restart, Previous, Cue }
impl TransportAction { pub const ALL: [TransportAction; 9]; }
pub type DrawFn = fn(Rect, Color32) -> Vec<Shape>;
pub struct Drawn { pub draw: DrawFn, pub size: Vec2 }
pub fn font_glyph(action: TransportAction) -> Option<(&'static str, bool)>; // (glyph, filled variant)
pub fn drawn(action: TransportAction) -> Option<Drawn>;
pub fn glyph_text(action: TransportAction) -> &'static str; // "" for a drawn icon
pub fn paint(painter: &Painter, rect: Rect, action: TransportAction, size: f32, color: Color32);
```

- [ ] **Step 1: Write the failing tests**

Create `crates/fp-app/tests/glyphs.rs`:

```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Operator feedback 2, O18: one icon per transport action.

use egui::{Rect, pos2, vec2};
use fp_app::ui::glyphs::{self, TransportAction};
use fp_app::ui::theme;

#[test]
fn every_action_has_exactly_one_kind_of_icon_and_no_two_share_one() {
    let mut seen: Vec<String> = Vec::new();
    for action in TransportAction::ALL {
        let font = glyphs::font_glyph(action);
        let drawn = glyphs::drawn(action);
        assert!(
            font.is_some() != drawn.is_some(),
            "{action:?} must be a font glyph or a drawn icon, not both or neither"
        );
        let key = match (font, drawn) {
            (Some((g, fill)), _) => format!("font:{g}:{fill}"),
            (_, Some(d)) => {
                // Drawn icons are told apart by their shapes.
                let rect = Rect::from_min_size(pos2(0.0, 0.0), d.size);
                format!("drawn:{:?}", (d.draw)(rect, theme::TEXT))
            }
            _ => unreachable!(),
        };
        assert!(!seen.contains(&key), "{action:?} repeats another icon");
        seen.push(key);
    }
    assert_eq!(seen.len(), 9);
}

#[test]
fn stop_is_the_filled_stop_square_and_cue_the_headphones() {
    assert_eq!(
        glyphs::font_glyph(TransportAction::Stop),
        Some((egui_phosphor::fill::STOP, true))
    );
    assert_eq!(
        glyphs::font_glyph(TransportAction::Cue),
        Some((egui_phosphor::regular::HEADPHONES, false))
    );
    assert_eq!(glyphs::glyph_text(TransportAction::Stop), egui_phosphor::fill::STOP);
    assert_eq!(glyphs::glyph_text(TransportAction::Play), egui_phosphor::fill::PLAY);
    // A drawn icon has no text form.
    assert_eq!(glyphs::glyph_text(TransportAction::FadeStop), "");
}

#[test]
fn drawn_actions_stay_inside_their_natural_size() {
    for action in TransportAction::ALL {
        let Some(d) = glyphs::drawn(action) else {
            continue;
        };
        let rect = Rect::from_min_size(pos2(10.0, 20.0), d.size);
        let shapes = (d.draw)(rect, theme::TEXT);
        assert!(!shapes.is_empty(), "{action:?}");
        for shape in shapes {
            assert!(
                rect.expand(1.0).contains_rect(shape.visual_bounding_rect()),
                "{action:?} {shape:?}"
            );
        }
        assert!(d.size.x > 0.0 && d.size.y > 0.0 && d.size.x <= 24.0 && d.size.y <= 16.0);
    }
    let _ = vec2(0.0, 0.0);
}

/// The player, the cartwall and the playlist menu draw transport icons only
/// through `glyphs` (a status marker such as the table's pause icon is not
/// a transport action and may stay).
#[test]
fn the_screens_do_not_name_transport_icons_themselves() {
    let sources = [
        ("player.rs", include_str!("../src/ui/player.rs")),
        ("cartwall.rs", include_str!("../src/ui/cartwall.rs")),
        ("table.rs", include_str!("../src/ui/table.rs")),
    ];
    for (name, source) in sources {
        for forbidden in [
            "fill::STOP",
            "fill::PLAY",
            "fill::FAST_FORWARD",
            "HEADPHONES",
            "icons::",
        ] {
            assert!(
                !source.contains(forbidden),
                "{name} still names {forbidden}; use ui::glyphs"
            );
        }
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-app --test glyphs every_action_has_exactly_one_kind_of_icon_and_no_two_share_one`
Expected: FAIL to compile (`unresolved import fp_app::ui::glyphs`).

- [ ] **Step 3: Create the module**

Create `crates/fp-app/src/ui/glyphs.rs`:

```rust
//! One icon per transport action (operator feedback 2, O18).
//!
//! The player, the cartwall and the CUE window draw their transport icons
//! only through this module, so the same action looks the same everywhere.
//! An icon is either a Phosphor font glyph or a shape drawn by
//! [`super::icons`] (the actions Phosphor has no icon for).

use egui::{Color32, Painter, Rect, Shape, Vec2, vec2};

use super::{icons, widgets};

/// What a transport button does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportAction {
    Play,
    /// The play button while a player is on air: fade to the next entry.
    Next,
    Pause,
    Stop,
    FadeStop,
    StopAfter,
    Restart,
    Previous,
    Cue,
}

impl TransportAction {
    pub const ALL: [TransportAction; 9] = [
        Self::Play,
        Self::Next,
        Self::Pause,
        Self::Stop,
        Self::FadeStop,
        Self::StopAfter,
        Self::Restart,
        Self::Previous,
        Self::Cue,
    ];
}

/// A drawing function of [`icons`].
pub type DrawFn = fn(Rect, Color32) -> Vec<Shape>;

/// A drawn icon and the size the transport grid draws it at.
#[derive(Clone, Copy)]
pub struct Drawn {
    pub draw: DrawFn,
    pub size: Vec2,
}

/// The font glyph of an action and whether it is the filled variant, or
/// `None` when the action is a drawn icon.
pub fn font_glyph(action: TransportAction) -> Option<(&'static str, bool)> {
    use egui_phosphor::{fill, regular};
    match action {
        TransportAction::Play => Some((fill::PLAY, true)),
        TransportAction::Next => Some((fill::FAST_FORWARD, true)),
        TransportAction::Pause => Some((fill::PAUSE, true)),
        TransportAction::Stop => Some((fill::STOP, true)),
        TransportAction::Cue => Some((regular::HEADPHONES, false)),
        TransportAction::FadeStop
        | TransportAction::StopAfter
        | TransportAction::Restart
        | TransportAction::Previous => None,
    }
}

/// The drawn icon of an action, or `None` when it is a font glyph.
pub fn drawn(action: TransportAction) -> Option<Drawn> {
    let (draw, w, h): (DrawFn, f32, f32) = match action {
        TransportAction::FadeStop => (icons::fade_stop, 16.0, 12.0),
        TransportAction::StopAfter => (icons::stop_after, 18.0, 13.0),
        TransportAction::Restart => (icons::restart, 14.0, 12.0),
        TransportAction::Previous => (icons::previous, 16.0, 12.0),
        TransportAction::Play
        | TransportAction::Next
        | TransportAction::Pause
        | TransportAction::Stop
        | TransportAction::Cue => return None,
    };
    Some(Drawn {
        draw,
        size: vec2(w, h),
    })
}

/// The glyph as text, for menu items and labels; empty for a drawn icon.
pub fn glyph_text(action: TransportAction) -> &'static str {
    font_glyph(action).map_or("", |(glyph, _)| glyph)
}

/// Paints the icon of `action` centred in `rect`: a font glyph at `size`
/// points, or a drawn icon at its natural size.
pub fn paint(painter: &Painter, rect: Rect, action: TransportAction, size: f32, color: Color32) {
    if let Some((glyph, fill)) = font_glyph(action) {
        widgets::glyph(painter, rect, glyph, size, color, fill);
    } else if let Some(d) = drawn(action) {
        painter.extend((d.draw)(Rect::from_center_size(rect.center(), d.size), color));
    }
}
```

In `crates/fp-app/src/ui.rs`, add `pub mod glyphs;` after `pub mod format;`.

- [ ] **Step 4: Route the screens through the module**

`crates/fp-app/src/ui/player.rs`:

1. Replace `use super::icons;` with `use super::glyphs::{self, TransportAction};`. If `icon` (`egui_phosphor::regular as icon`) is then unused, clippy says so; remove only what clippy reports.
2. The play tile: replace

```rust
            let glyph = if playing {
                egui_phosphor::fill::FAST_FORWARD
            } else {
                egui_phosphor::fill::PLAY
            };
```

with

```rust
            let play_action = if playing {
                TransportAction::Next
            } else {
                TransportAction::Play
            };
```

and inside its paint closure replace `widgets::glyph(p, r.translate(vec2(0.0, -6.0)), glyph, 28.0, c, true);` with `glyphs::paint(p, r.translate(vec2(0.0, -6.0)), play_action, 28.0, c);`.
3. The CUE tile: replace `format!("{} {cue_label}", icon::HEADPHONES)` with `format!("{} {cue_label}", glyphs::glyph_text(TransportAction::Cue))`. The CUE readout (near line 506): replace `RichText::new(icon::HEADPHONES)` with `RichText::new(glyphs::glyph_text(TransportAction::Cue))`.
4. The transport grid: delete the two closures `glyph` and `drawn` and put in their place

```rust
            let button = |action: TransportAction| -> PaintFn {
                Box::new(move |p, r, c| glyphs::paint(p, r, action, 13.0, c))
            };
```

Then in `rows` replace `drawn(icons::previous, 16.0, 12.0)` with `button(TransportAction::Previous)`, `glyph(egui_phosphor::fill::STOP)` with `button(TransportAction::Stop)`, `glyph(egui_phosphor::fill::PAUSE)` with `button(TransportAction::Pause)`, `drawn(icons::restart, 14.0, 12.0)` with `button(TransportAction::Restart)`, `drawn(icons::fade_stop, 16.0, 12.0)` with `button(TransportAction::FadeStop)`, `drawn(icons::stop_after, 18.0, 13.0)` with `button(TransportAction::StopAfter)`.

`crates/fp-app/src/ui/cartwall.rs`: add `use super::glyphs::{self, TransportAction};` and in the cart menu replace `item(ui, icon::HEADPHONES, "menu-cue")` with `item(ui, glyphs::glyph_text(TransportAction::Cue), "menu-cue")` and `item(ui, egui_phosphor::fill::STOP, "menu-cart-stop")` with `item(ui, glyphs::glyph_text(TransportAction::Stop), "menu-cart-stop")` (this is O18's "the cart stop becomes the player's stop icon": one table now says it).

`crates/fp-app/src/ui/table.rs`: replace `use super::icons;` with `use super::glyphs::{self, TransportAction};`. Replace `p.extend(icons::stop_after(r, text));` with `glyphs::paint(p, r, TransportAction::StopAfter, 13.0, text);`. In the context menu replace `egui_phosphor::fill::PLAY` (the `menu-play-now` item) with `glyphs::glyph_text(TransportAction::Play)` and `icon::HEADPHONES` (the `menu-cue` item) with `glyphs::glyph_text(TransportAction::Cue)`.

- [ ] **Step 5: Run the new tests and the screens' tests**

Run each:

```sh
cargo test -p fp-app --test glyphs every_action_has_exactly_one_kind_of_icon_and_no_two_share_one
cargo test -p fp-app --test glyphs stop_is_the_filled_stop_square_and_cue_the_headphones
cargo test -p fp-app --test glyphs drawn_actions_stay_inside_their_natural_size
cargo test -p fp-app --test glyphs the_screens_do_not_name_transport_icons_themselves
cargo test -p fp-app --test main_screen
cargo test -p fp-app --test theme
cargo test -p fp-app --test cartwall_ui
```

Expected: PASS for all (`main_screen`, `theme` and `cartwall_ui` run whole, as regression for the moved icons: the buttons keep their labels).

- [ ] **Step 6: Commit**

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
git add crates/fp-app/src/ui/glyphs.rs crates/fp-app/src/ui.rs crates/fp-app/src/ui/player.rs crates/fp-app/src/ui/cartwall.rs crates/fp-app/src/ui/table.rs crates/fp-app/tests/glyphs.rs
git commit -m "refactor(ui): draw transport icons through one glyphs module

The player, the cartwall menu and the playlist menu each named their own
icon constants, so the same action could look different on each screen.
One table now maps every transport action to its icon; the CUE window
(plan 6) will use it too." -m "Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>"
```

(Run the whole suite, `cargo test --workspace`, before the commit as the Global Constraints say.)

---

### Task 2: "Stop all (n)" at the right end of the cartwall bar (O19)

**Files:**
- Modify: `crates/fp-app/src/ui/cartwall.rs` (`header`, near lines 54-107)
- Modify: `crates/fp-app/locales/en-US/main.ftl` (after `cartwall-hint`, line 171), `crates/fp-app/locales/es-ES/main.ftl` (same place)
- Test: `crates/fp-model/tests/cartwall.rs`, `crates/fp-app/tests/cartwall_ui.rs`, `crates/fp-app/tests/i18n.rs`

**Interfaces:**
- Consumes: `Command::StopAllCarts`; `glyphs::paint`, `TransportAction::Stop` (Task 1); `widgets::tile(ui, size, label, enabled, style, paint)`; `Cartwall::playing: Vec<PlayingCart>`; test helpers `harness`, `harness_sized`, `with_cart`.
- Produces: the messages `cartwall-stop-all` and `cartwall-stop-all-count` (arg `count`); a tile whose accessible name is "Stop all" or "Stop all (n)".

- [ ] **Step 1: Write the failing tests**

(a) `crates/fp-model/tests/cartwall.rs`, after `c10_stop_all_stops_carts_and_cue`. These two pin the existing rule (they are expected to pass at once; they are the spec's "one rule test" for stopping "the same way the individual stop does", plus the idle case):

```rust
#[test]
fn c10_stop_all_stops_every_cart_the_way_each_single_stop_does() {
    let mut state = fixture(0);
    let a = load(&mut state, 0, 10.0);
    let b = load(&mut state, 1, 10.0);
    let c = load(&mut state, 2, 10.0);
    edit(&mut state, 1, |e| e.looped = true);
    edit(&mut state, 2, |e| e.exclusive = false);
    for id in [a, b, c] {
        apply(&mut state, Command::FireCart(id)).unwrap();
    }
    let mut one_by_one = state.clone();
    let mut each = Vec::new();
    for id in [a, b, c] {
        each.extend(apply(&mut one_by_one, Command::StopCart(id)).unwrap());
    }
    let all = apply(&mut state, Command::StopAllCarts).unwrap();
    assert_eq!(stopped(&all), stopped(&each));
    assert_eq!(stopped(&all), vec![a, b, c], "in firing order");
    assert_eq!(state.cartwall.playing, one_by_one.cartwall.playing);
    assert!(state.cartwall.playing.is_empty());
}

#[test]
fn c10_stop_all_with_nothing_on_air_is_harmless_and_still_stops_the_cue() {
    let mut state = fixture(0);
    assert!(apply(&mut state, Command::StopAllCarts).unwrap().is_empty());
    let a = load(&mut state, 0, 10.0);
    apply(&mut state, Command::CueCart(a)).unwrap();
    let actions = apply(&mut state, Command::StopAllCarts).unwrap();
    assert!(stopped(&actions).is_empty());
    assert!(actions.contains(&EngineAction::StopCartCue));
    assert!(state.cartwall.cue.is_none());
}
```

(b) `crates/fp-app/tests/i18n.rs`, at the end:

```rust
#[test]
fn the_stop_all_label_carries_its_count_in_both_languages() {
    let en = I18n::new(Some("en-US"));
    assert_eq!(en.tr("cartwall-stop-all"), "Stop all");
    assert_eq!(
        en.tr_args("cartwall-stop-all-count", &[("count", 2.into())]),
        "Stop all (2)"
    );
    let es = I18n::new(Some("es-ES"));
    assert_eq!(es.tr("cartwall-stop-all"), "Parar todo");
    assert_eq!(
        es.tr_args("cartwall-stop-all-count", &[("count", 12.into())]),
        "Parar todo (12)"
    );
}
```

(c) `crates/fp-app/tests/cartwall_ui.rs`: add the helpers after `with_cart` and the tests at the end of the file:

```rust
/// Gives cart `index` of the first page a 10 s file called `name`.
fn give_file(s: &mut AppState, index: usize, name: &str) {
    let page = s.cartwall.pages[0].id;
    apply(
        s,
        Command::AssignCartFile {
            page,
            index,
            path: PathBuf::from(format!("/carts/{name}.wav")),
        },
    )
    .unwrap();
    let track = s.cartwall.pages[0].carts[index].track.unwrap();
    let analysis = TrackAnalysis {
        duration_secs: 10.0,
        cue_in: Some(0.5),
        cue_out: Some(9.5),
        ..TrackAnalysis::default()
    };
    apply(
        s,
        Command::ApplyAnalysis {
            track,
            analysis: Box::new(analysis),
        },
    )
    .unwrap();
}

/// A state with `n` carts (the first is "Station ID") all playing.
fn with_playing_carts(n: usize) -> AppState {
    let mut s = with_cart();
    for i in 1..n {
        give_file(&mut s, i, &format!("c{i}"));
    }
    for i in 0..n {
        let id = s.cartwall.pages[0].carts[i].id;
        apply(&mut s, Command::FireCart(id)).unwrap();
    }
    s
}

fn stop_all_sent(fake: &Fake) -> bool {
    fake.take_sent().contains(&Command::StopAllCarts)
}

#[test]
fn stop_all_is_dimmed_and_has_no_count_while_no_cart_plays() {
    let (mut h, fake) = harness(with_cart());
    let button = h.get_by_label("Stop all");
    assert!(button.accesskit_node().is_disabled());
    button.click();
    h.run_steps(2);
    assert!(!stop_all_sent(&fake), "a dimmed button sends nothing");
}

#[test]
fn stop_all_shows_the_number_of_playing_carts() {
    let (mut h, _fake) = harness(with_playing_carts(2));
    h.run_steps(2);
    let button = h.get_by_label("Stop all (2)");
    assert!(!button.accesskit_node().is_disabled());
    assert!(h.query_by_label("Stop all").is_none(), "no label without a count");
}

#[test]
fn stop_all_counts_carts_on_every_page() {
    let mut s = with_playing_carts(2);
    apply(
        &mut s,
        Command::CreateCartPage {
            name: "Sports".into(),
        },
    )
    .unwrap();
    let sports = s.cartwall.pages[1].id;
    apply(&mut s, Command::ShowCartPage(sports)).unwrap();
    let (mut h, _fake) = harness(s);
    assert!(
        h.query_by_label("Station ID").is_none(),
        "the carts playing are on the page not shown"
    );
    assert!(h.query_by_label("Stop all (2)").is_some());
}

#[test]
fn clicking_stop_all_stops_every_cart() {
    let (mut h, fake) = harness(with_playing_carts(2));
    h.get_by_label("Stop all (2)").click();
    h.run_steps(3);
    assert!(stop_all_sent(&fake));
    assert!(fake.state.load().cartwall.playing.is_empty());
}

#[test]
fn stop_all_dims_again_when_the_last_cart_stops() {
    let (mut h, fake) = harness(with_playing_carts(1));
    assert!(h.query_by_label("Stop all (1)").is_some());
    let id = fake.state.load().cartwall.pages[0].carts[0].id;
    fp_app::ui::controller::Controller::send(fake.as_ref(), Command::StopCart(id));
    h.run_steps(3);
    assert!(h.query_by_label("Stop all (1)").is_none(), "no stale count");
    assert!(h.get_by_label("Stop all").accesskit_node().is_disabled());
}

#[test]
fn stop_all_works_with_the_cartwall_collapsed() {
    let (mut h, fake) = harness(with_playing_carts(2));
    h.get_by_label("CARTWALL").click();
    h.run_steps(3);
    assert!(h.query_by_label("Station ID").is_none(), "collapsed");
    h.get_by_label("Stop all (2)").click();
    h.run_steps(3);
    assert!(stop_all_sent(&fake));
}

#[test]
fn a_cart_cue_alone_does_not_enable_stop_all() {
    let mut s = with_cart();
    let id = s.cartwall.pages[0].carts[0].id;
    apply(&mut s, Command::CueCart(id)).unwrap();
    assert!(s.cartwall.cue.is_some() && s.cartwall.playing.is_empty());
    let (h, _fake) = harness(s);
    assert!(h.get_by_label("Stop all").accesskit_node().is_disabled());
}

#[test]
fn stop_all_sits_at_the_right_end_of_the_bar() {
    let (h, _fake) = harness(with_playing_carts(2));
    let stop = h.get_by_label("Stop all (2)").rect();
    let title = h.get_by_label("CARTWALL").rect();
    assert!(stop.left() > title.right(), "after the collapse control");
    assert!(stop.right() > 1000.0 - 60.0, "at the right end: {stop:?}");
}

#[test]
fn stop_all_stays_at_the_right_end_in_a_narrow_window() {
    let mut s = with_playing_carts(2);
    for n in 0..8 {
        apply(
            &mut s,
            Command::CreateCartPage {
                name: format!("A rather long page name {n}"),
            },
        )
        .unwrap();
    }
    let (mut h, _fake) = support::harness_sized(s, egui::vec2(420.0, 480.0), |ui| ui);
    h.run_steps(3);
    let stop = h.get_by_label("Stop all (2)").rect();
    assert!(stop.right() <= 420.0 + 0.5, "inside the window: {stop:?}");
    assert!(stop.right() > 420.0 - 60.0, "at the right end: {stop:?}");
    assert!(stop.width() > 40.0, "not squeezed: {stop:?}");
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run each:

```sh
cargo test -p fp-model --test cartwall c10_stop_all_stops_every_cart_the_way_each_single_stop_does
cargo test -p fp-model --test cartwall c10_stop_all_with_nothing_on_air_is_harmless_and_still_stops_the_cue
cargo test -p fp-app --test i18n the_stop_all_label_carries_its_count_in_both_languages
cargo test -p fp-app --test cartwall_ui stop_all_shows_the_number_of_playing_carts
cargo test -p fp-app --test cartwall_ui stop_all_stays_at_the_right_end_in_a_narrow_window
```

Expected: the two model tests PASS at once (they pin the existing rule; if either fails, stop and report: the rule is not what the spec assumes). The i18n test fails (`cartwall-stop-all` is returned as its own key). The two UI tests fail (`No nodes found with label "Stop all (2)"`). The other new UI tests fail the same way, or on the missing "Stop all" node.

- [ ] **Step 3: Add the messages**

`crates/fp-app/locales/en-US/main.ftl`, after the `cartwall-hint` line:

```ftl
cartwall-stop-all = Stop all
cartwall-stop-all-count = Stop all ({ $count })
```

`crates/fp-app/locales/es-ES/main.ftl`, after its `cartwall-hint` line:

```ftl
cartwall-stop-all = Parar todo
cartwall-stop-all-count = Parar todo ({ $count })
```

- [ ] **Step 4: Draw the button**

In `crates/fp-app/src/ui/cartwall.rs`, replace the whole `header` function with the version below (the `use super::glyphs::{self, TransportAction};` import was added in Task 1; add `Align2` through the path `egui::Align2`, already used in the file). The left part is the old header body, unchanged, moved into a nested left-to-right row:

```rust
fn header(ui: &mut Ui, scene: &Scene<'_>) {
    let t = scene.i18n;
    let wall = &scene.state.cartwall;
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), HEADER_HEIGHT),
        Layout::right_to_left(Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing = vec2(10.0, 0.0);
            // First, so that it is always at the right end and always whole;
            // the title, tabs and hint take what is left of the row.
            stop_all(ui, scene);
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                ui.spacing_mut().item_spacing = vec2(10.0, 0.0);
                let title = t.tr("cartwall-title");
                let caret = if wall.open {
                    icon::CARET_DOWN
                } else {
                    icon::CARET_RIGHT
                };
                let width = ui
                    .painter()
                    .layout_no_wrap(title.clone(), font(10.0), theme::NEUTRAL_300)
                    .size()
                    .x
                    + 24.0;
                let style = TileStyle {
                    border: Color32::TRANSPARENT,
                    hover_fill: Color32::TRANSPARENT,
                    active_fill: Color32::TRANSPARENT,
                    ..TileStyle::plain()
                };
                if widgets::tile(ui, vec2(width, 20.0), &title, true, style, |p, r, c| {
                    p.text(
                        r.left_center(),
                        egui::Align2::LEFT_CENTER,
                        format!("{caret} {title}"),
                        font(10.0),
                        c,
                    );
                })
                .clicked()
                {
                    scene.ctl.send(Command::SetCartwallOpen(!wall.open));
                }
                tabs(ui, scene);
                ui.add(
                    egui::Label::new(
                        RichText::new(t.tr("cartwall-hint"))
                            .font(font(10.0))
                            .color(theme::NEUTRAL_500),
                    )
                    .selectable(false)
                    .truncate(),
                );
            });
        },
    );
}

/// "Stop all (n)": stops every playing cart (spec O19). Dimmed, and without
/// a count, while no cart is playing. The count is the whole cartwall's.
fn stop_all(ui: &mut Ui, scene: &Scene<'_>) {
    const ICON_WIDTH: f32 = 14.0;
    let t = scene.i18n;
    let playing = scene.state.cartwall.playing.len();
    let label = if playing == 0 {
        t.tr("cartwall-stop-all")
    } else {
        t.tr_args("cartwall-stop-all-count", &[("count", playing.into())])
    };
    let text_width = ui
        .painter()
        .layout_no_wrap(label.clone(), font(10.0), theme::TEXT)
        .size()
        .x;
    let width = 6.0 + ICON_WIDTH + 4.0 + text_width + 8.0;
    let clicked = widgets::tile(
        ui,
        vec2(width, 20.0),
        &label,
        playing > 0,
        TileStyle::plain(),
        |p, r, c| {
            let icon_rect = Rect::from_min_size(
                pos2(r.left() + 6.0, r.top()),
                vec2(ICON_WIDTH, r.height()),
            );
            glyphs::paint(p, icon_rect, TransportAction::Stop, 11.0, c);
            p.text(
                pos2(icon_rect.right() + 4.0, r.center().y),
                egui::Align2::LEFT_CENTER,
                &label,
                font(10.0),
                c,
            );
        },
    )
    .clicked();
    if clicked {
        scene.ctl.send(Command::StopAllCarts);
    }
}
```

(`tr_args` takes `&[(&str, Arg)]`; `playing.into()` works as in `(index + 1).into()` elsewhere in the file because `Arg: From<usize>`. If it does not, use `(playing as i64).into()` as the nearest existing call does; check `crates/fp-app/src/i18n.rs`.)

- [ ] **Step 5: Run the tests to verify they pass**

Run each:

```sh
cargo test -p fp-app --test i18n the_stop_all_label_carries_its_count_in_both_languages
cargo test -p fp-app --test i18n both_locales_define_the_same_keys
cargo test -p fp-app --test cartwall_ui stop_all_is_dimmed_and_has_no_count_while_no_cart_plays
cargo test -p fp-app --test cartwall_ui stop_all_shows_the_number_of_playing_carts
cargo test -p fp-app --test cartwall_ui stop_all_counts_carts_on_every_page
cargo test -p fp-app --test cartwall_ui clicking_stop_all_stops_every_cart
cargo test -p fp-app --test cartwall_ui stop_all_dims_again_when_the_last_cart_stops
cargo test -p fp-app --test cartwall_ui stop_all_works_with_the_cartwall_collapsed
cargo test -p fp-app --test cartwall_ui a_cart_cue_alone_does_not_enable_stop_all
cargo test -p fp-app --test cartwall_ui stop_all_sits_at_the_right_end_of_the_bar
cargo test -p fp-app --test cartwall_ui stop_all_stays_at_the_right_end_in_a_narrow_window
cargo test -p fp-app --test cartwall_ui
cargo test -p fp-app --test main_screen
```

Expected: PASS. If `stop_all_stays_at_the_right_end_in_a_narrow_window` fails because the tabs push the button off the row, the nested `with_layout` is not bounded by the remaining width: wrap the tabs call's `max_width` in `tabs` (it uses 60 % of `available_rect_before_wrap`, which is now the remaining width) and check the hint's `truncate()`; do not move the button back to the end of the row.

- [ ] **Step 6: Commit**

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
git add crates/fp-app/src/ui/cartwall.rs crates/fp-app/locales crates/fp-app/tests/cartwall_ui.rs crates/fp-app/tests/i18n.rs crates/fp-model/tests/cartwall.rs
git commit -m "feat(ui): add a Stop all button with a count to the cartwall bar

Stopping several carts took one right-click each. The button at the right
end of the bar sends StopAllCarts and shows how many carts are playing; it
is dimmed with none. The model rule is unchanged and now has a test that it
stops carts exactly as the single stop does." -m "Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>"
```

---

### Task 3: Shortcut pins, screenshot check and documentation (O19, O18)

**Files:**
- Modify: `crates/fp-app/tests/main_screen.rs` (after `ctrl_space_stops_all_carts`)
- Modify: `docs/user/cartwall.md`, `docs/user/keyboard.md`, `docs/technical/ui.md`, `docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md`
- Modify (only if the screenshot calls for it): `crates/fp-app/src/ui/cartwall.rs`

**Interfaces:** Consumes everything above; produces nothing new.

- [ ] **Step 1: Write the shortcut tests**

In `crates/fp-app/tests/main_screen.rs`, after `ctrl_space_stops_all_carts`:

```rust
#[test]
fn stop_all_follows_a_rebound_shortcut() {
    let mut s = state(1, 1);
    fp_model::apply(
        &mut s,
        Command::SetShortcut {
            action: fp_model::ShortcutAction::StopAllCarts,
            chord: Some(fp_model::KeyChord::key("Q")),
        },
    )
    .unwrap();
    let (mut h, fake) = harness(s);
    h.key_press(Key::Q);
    h.run_steps(2);
    assert_eq!(sent(&fake), vec![Command::StopAllCarts]);
    h.key_press_modifiers(egui::Modifiers::CTRL, Key::Space);
    h.run_steps(2);
    assert!(sent(&fake).is_empty(), "Ctrl+Space lost its binding");
}

#[test]
fn an_unbound_stop_all_shortcut_does_nothing() {
    let mut s = state(1, 1);
    fp_model::apply(
        &mut s,
        Command::SetShortcut {
            action: fp_model::ShortcutAction::StopAllCarts,
            chord: None,
        },
    )
    .unwrap();
    let (mut h, fake) = harness(s);
    h.key_press_modifiers(egui::Modifiers::CTRL, Key::Space);
    h.run_steps(2);
    assert!(sent(&fake).is_empty());
}
```

- [ ] **Step 2: Run them**

Run each:

```sh
cargo test -p fp-app --test main_screen stop_all_follows_a_rebound_shortcut
cargo test -p fp-app --test main_screen an_unbound_stop_all_shortcut_does_nothing
```

Expected: PASS at once. They pin behaviour that exists (the Decisions section explains why the default key stays). If one fails, the dispatcher has a bug: fix it in `crates/fp-app/src/ui/app.rs` (the shortcut dispatcher near line 905) before going on, and say so in the commit message.

- [ ] **Step 3: Check the look in a screenshot**

Follow the CLAUDE.md screenshot recipe (`xvfb`, `xdotool` and ImageMagick `import` must be installed):

```sh
cargo build --release -p fp-app
Xvfb :77 -screen 0 1920x1080x24 -nolisten tcp &
mkdir -p /tmp/fp-stopall   # scratch state only; builds still go to target/
FAUSTE_HOME=/tmp/fp-stopall cargo run --release -p fp-app --example demo_session -- <music dir>
```

Start the app with `env -u WAYLAND_DISPLAY DISPLAY=:77 FAUSTE_HOME=/tmp/fp-stopall target/release/fauste-player &`, enable `remote.http` as the recipe says, and fire two or three carts through the API (`docs/user/remote-control.md`). Capture the window (`DISPLAY=:77 xwininfo -name "Fauste Player"`, then `DISPLAY=:77 import -window <id> shot.png`) in three states: no cart playing, two playing, and the window at about 700 px wide with the page tabs crowded. Confirm: the button is at the right end of the bar, after the tabs and the hint; its stop square looks like the player's Stop; dimmed with no cart playing and no count; "Stop all (2)" with two; the label is not cut off at 700 px; the page tabs and the hint give way, not the button. If the text or the icon looks cramped, adjust the paddings in `stop_all` (`ICON_WIDTH`, the 6/4/8 px paddings) and re-run `cargo test -p fp-app --test cartwall_ui`. Stop the app and the Xvfb server when done (`kill %1 %2` or by pid).

- [ ] **Step 4: Update the user guide**

`docs/user/cartwall.md`: in "## Using it", after the bullet "**Click a cart** to fire it. **Click it again** to stop it.", add:

```markdown
- **Stop all** (right end of the bar) stops every cart that is playing, on
  every page. Its label shows how many are playing, as in **Stop all (2)**;
  with none playing it is dimmed and shows no count.
```

In the "## Keyboard" section, replace the sentence "**Ctrl+Space** stops every cart." with:

```markdown
**Ctrl+Space** stops every cart (the same as **Stop all**; it also stops a
cart you are pre-listening on CUE, even when no cart is playing).
```

`docs/user/keyboard.md`: replace the table row ``| `Ctrl+Space` | Stop every cart |`` with ``| `Ctrl+Space` | Stop every cart, and the cart CUE (the cartwall's **Stop all** button) |``. In the bullet "A shortcut whose button is dimmed does nothing (for example Stop on a stopped player)." append " The one exception is the stop-all-carts shortcut, which also stops the cart CUE." In the list of available actions, make sure "stop every cart" is mentioned next to "fire a cart of the page shown" (add the words if the list lacks them).

- [ ] **Step 5: Update the technical docs**

`docs/technical/ui.md`:

1. In the table row for `ui/widgets.rs`, `ui/icons.rs`, `ui/theme.rs` (line 24) add `ui/glyphs.rs` before `ui/icons.rs` in the first cell (`ui/widgets.rs`, `ui/glyphs.rs`, `ui/icons.rs`, `ui/theme.rs`) and end the second cell with: ` `ui/glyphs.rs` maps each transport action (`TransportAction`: play, next, pause, stop, fade stop, stop after, restart, previous, cue) to a Phosphor glyph or a drawn icon; the player, the cartwall and the playlist menu draw their transport icons only through it (`tests/glyphs.rs` guards that), and the CUE window will too.`
2. In the row for `ui/cartwall.rs`, `ui/cart_view.rs` (line 19) append: ` The bar's "Stop all (n)" button sends `Command::StopAllCarts` and shows `cartwall.playing.len()`; it is the first item of a right-to-left row so it never gives way to the tabs.`

- [ ] **Step 6: Update the spec**

In `docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md`, under "## 5. Plan 4 — Cartwall stop", at the end of the section add:

```markdown
- **As built.**
  - `ui/glyphs.rs` has `TransportAction` (the spec's list plus `Next`, the play
    button while a player is on air). The player, the cartwall menu and the
    playlist menu draw only through it; `tests/glyphs.rs` guards that.
  - The "Stop all (n)" button is the first item of a right-to-left row, so
    it is always whole at the right end; the title, tabs and hint take the
    rest. The count is `cartwall.playing.len()`, carts on every page. The
    messages are `cartwall-stop-all` and `cartwall-stop-all-count`.
  - `ShortcutAction::StopAllCarts` already existed, bound to `Ctrl+Space`
    by default. The default stays: removing it would change a documented
    and tested behaviour. The shortcut is not dimmed with the button: it
    also stops the cart CUE, so it works with no cart playing.
```

- [ ] **Step 7: Run the whole gate**

Run:

```sh
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
```

Expected: all green. Also `grep -rn -i "stop all" README.md` and update the README feature list if it describes the cartwall controls (docs are part of the change).

- [ ] **Step 8: Commit**

```bash
git add crates/fp-app/tests/main_screen.rs docs/user/cartwall.md docs/user/keyboard.md docs/technical/ui.md docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md
git commit -m "docs(ui): describe the Stop all button and the shared glyphs

Pins the rebinding and unbinding of the stop-all-carts shortcut, and
records in the spec that its Ctrl+Space default stays." -m "Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>"
```

(Stage `crates/fp-app/src/ui/cartwall.rs` too if Step 3 changed paddings, and `README.md` if Step 7 changed it.)

---

## Self-review

- **O18:** `ui/glyphs.rs` maps every transport action (Task 1: play, next, pause, stop, fade stop, stop after, restart, previous, cue); player, cartwall menu and playlist menu draw through it (Task 1 step 4, guarded by `the_screens_do_not_name_transport_icons_themselves`); the cart stop is the player's stop icon (`glyph_text(Stop)` in the menu, `Stop` in the button, one table); plan 6 can reuse it (documented in `docs/technical/ui.md`, Task 3).
- **O19:** button at the right end after the page and collapse controls (Task 2 step 4, tests `stop_all_sits_at_the_right_end_of_the_bar` and the narrow-window one); label "Stop all (n)" and dimmed with no count (messages in both locales, tests `stop_all_is_dimmed_and_has_no_count_while_no_cart_plays`, `stop_all_shows_the_number_of_playing_carts`); `Command::StopAllCarts` stops carts like the single stop, one rule test (Task 2 (a)); the shortcut exists, with its default kept and recorded in Decisions (Task 3 pins).
- **Types:** `TransportAction`, `font_glyph`, `drawn`, `Drawn { draw, size }`, `glyph_text`, `paint(&Painter, Rect, TransportAction, f32, Color32)` are named the same in every task. Messages `cartwall-stop-all`, `cartwall-stop-all-count` (arg `count`) are the same in the code, both locales and the tests.
- **Locales:** two new messages in `en-US` and `es-ES`; `both_locales_define_the_same_keys` keeps them in step.
- **Review Focus:** items 1-6 each map to a named test above.
