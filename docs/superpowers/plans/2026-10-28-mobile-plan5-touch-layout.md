# Mobile Plan 5 — Touch Layout Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** On Android the interface fits the screen it runs on (mobile
spec §6): a phone in portrait shows all players as compact panels or one
player with its playlist, and the cartwall on its own screen; a phone in
landscape shows two players side by side; a tablet keeps the desktop
layout. Every target a finger uses is easier to hit.

**Architecture:** `fp-app` gains `ui::touch`: the form factor from the
screen size (a phone when the shortest side is under 600 points, egui
points being Android's dp), the phone's view state (which screen, which
player is open, which pair shows in landscape) as pure functions, and the
phone's middle area. `AppUi::with_touch()` turns it on (`fp-mobile` calls
it); without it nothing changes. A phone keeps the desktop's top bar,
with Players | Cartwall tiles in place of the panel tiles. The views reuse
the desktop's widgets: a compact panel is the player block without the
waveform and the playlist (`player::compact`), one player is the
desktop's player column (`player::column`), the landscape pair goes
through `panels::players_row` with two columns, and the cartwall screen is
`cartwall::strip` filling the area. Touch mode widens egui's interaction
radius so the desktop's small tiles are easier to hit.

**Tech Stack:** Rust (egui 0.37, egui_kittest).

**Spec:** `docs/superpowers/specs/2026-10-25-mobile-design.md` (§6, the
approved structure and its mockups).

## Global Constraints

- The desktop is unchanged: without `with_touch()` no code path differs,
  and every existing test passes as is.
- A tablet (shortest side ≥ 600) draws the desktop layout; only the touch
  style applies.
- Nothing the phone views hide stops or changes playout: the views only
  choose what is drawn.
- New strings in all 11 locales; no new `Config` field (the breakpoint is
  the spec's).

## Review Focus

- A player removed (Settings → player count) while it is open on the
  phone: the view falls back to all players, no panic.
- Rotation between portrait and landscape keeps the open player.
- The landscape pair never points past the last player (odd counts, one
  player).
- The cartwall screen with the cartwall closed or hidden in the layout.
- Context menus on the phone open by long press (egui's secondary click).

---

### Task 1: Form factor and phone view state (pure)

**Files:**
- Create: `crates/fp-app/src/ui/touch.rs` (`pub mod touch;` in `ui.rs`)
- Test: `crates/fp-app/tests/it/touch.rs` (+ `mod` line)

**Interfaces:**
- `pub enum FormFactor { Phone { landscape: bool }, Tablet }`,
  `pub fn form_factor(size: egui::Vec2) -> FormFactor`.
- `#[derive(Default)] pub struct PhoneView { pub screen: PhoneScreen, pub open: Option<PlayerId>, pub pair: usize }`,
  `pub enum PhoneScreen { #[default] Players, Cartwall }`.
- `impl PhoneView { pub fn settle(&mut self, players: &[PlayerId]); pub fn pairs(count: usize) -> usize; pub fn pair_players(&self, players: &[PlayerId]) -> Vec<PlayerId>; pub fn swipe(&mut self, forward: bool, count: usize) }`.

- [ ] **Step 1: Write the failing tests:** `a_narrow_screen_is_a_phone`,
  `a_wide_phone_is_in_landscape`, `a_large_screen_is_a_tablet`,
  `a_removed_open_player_falls_back_to_all`,
  `pairs_cover_odd_counts`, `the_pair_never_points_past_the_last_player`,
  `swiping_stops_at_both_ends`.
- [ ] **Step 2:** Run them. Expected: FAIL.
- [ ] **Step 3:** Implement.
- [ ] **Step 4:** Run them. Expected: PASS.
- [ ] **Step 5:** Commit `feat(ui): form factors and the phone's view state`.

### Task 2: Touch mode and the phone's top bar

**Files:**
- Modify: `crates/fp-app/src/ui/app.rs` (`with_touch`, `touch: bool`,
  `phone: PhoneView` in the view state, the top bar on a phone)
- Modify: `crates/fp-app/src/ui/touch.rs` (`apply_style`, `screen_tiles`)
- Modify: every `locales/*/main.ftl`: `touch-players` "Players" /
  "Reproductores", `touch-cartwall` "Cartwall" / "Cartwall"
- Test: `crates/fp-app/tests/it/touch_ui.rs`

- [ ] **Step 1: Failing kittest tests** (412 × 892 and 892 × 412 and
  800 × 1280 harnesses with `with_touch()`):
  `the_phone_top_bar_switches_screens`, `a_tablet_keeps_the_panel_tiles`,
  `without_touch_nothing_changes` (the desktop panel tiles at phone size).
- [ ] **Step 2–4:** Implement: touch mode widens `interact_radius` and the
  scroll bars; on a phone the top bar's left part is two tiles.
- [ ] **Step 5:** Commit `feat(ui): touch mode and the phone's top bar`.

### Task 3: Phone, portrait: all players and one player

**Files:**
- Modify: `crates/fp-app/src/ui/player.rs` (`pub(crate) fn compact`)
- Modify: `crates/fp-app/src/ui/touch.rs` (`phone_middle`)
- Modify: `crates/fp-app/src/ui/app.rs` (the phone draws `phone_middle`
  in place of the panels)
- Strings: `touch-open-player` "Open { $player }" / "Abrir { $player }",
  `touch-all-players` "All" / "Todos"
- Test: `crates/fp-app/tests/it/touch_ui.rs`

- [ ] **Step 1: Failing tests:** `all_players_show_as_compact_panels`
  (each player's PLAY tile and no playlist table),
  `open_shows_one_player_with_its_playlist`,
  `chips_switch_the_open_player`, `all_goes_back_to_the_panels`,
  `play_in_a_compact_panel_sends_play` (the command reaches the fake).
- [ ] **Step 2–4:** Implement: the compact panel is the header, the top
  block (titles, countdown, transport, meters and fader) and a progress
  bar with the next track; an "Open P1" tile under it. One player: a chip
  row (All, P1…Pn) above `player::column` at the full width.
- [ ] **Step 5:** Commit `feat(ui): the phone's player views`.

### Task 4: Phone, landscape, and the cartwall screen

**Files:**
- Modify: `crates/fp-app/src/ui/touch.rs`
- Test: `crates/fp-app/tests/it/touch_ui.rs`

- [ ] **Step 1: Failing tests:** `landscape_shows_two_players_side_by_side`,
  `a_swipe_shows_the_next_pair`, `the_cartwall_screen_shows_the_carts`,
  `the_cartwall_screen_without_pages_says_so` (the strip's own empty state).
- [ ] **Step 2–4:** Implement: pair chips (P1–2, P3–4…) and a horizontal
  swipe over the area (no widget dragged, |dx| > 80 and twice |dy|)
  change the pair; two columns through `players_row`; a chip on a column
  opens it as in portrait. The cartwall screen draws `cartwall::strip`
  filling the area.
- [ ] **Step 5:** Commit `feat(ui): the phone's landscape and cartwall views`.

### Task 5: Android wiring, docs and screenshots

**Files:**
- Modify: `crates/fp-mobile/src/android.rs` (`.with_touch()`)
- Modify: `docs/technical/mobile.md` (a touch layout section; drop the
  interim layout line), `docs/technical/ui.md` if it lists the views

- [ ] **Step 1:** Wire, Android clippy, debug APK.
- [ ] **Step 2:** Emulator screenshots (phone portrait: all players, one
  player, cartwall; landscape; a tablet AVD) for the maintainer to
  compare with the mockups.
- [ ] **Step 3:** Commit `feat(mobile): the touch layout on Android`.

## Deferred (rulings in the ledger)

- The row menu as a bottom sheet: the existing context menu opens by long
  press; a sheet is a later polish.
- Long-press tooltips and long-press drag and drop of rows: egui's touch
  handling is kept as is until a device check says what is missing.
