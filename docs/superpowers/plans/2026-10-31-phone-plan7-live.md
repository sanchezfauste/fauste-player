# Phone layout plan 7: the Live screen's fold and mini cards — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The phone layout spec §16 (PL35, PL36): the Live screen's
cartwall folds, and landscape Live shows a one-row mini card per player.

**Architecture:** UI only, in `fp-app/src/ui` (`touch.rs` `live`,
`cartwall.rs` `strip`/`header`, a `mini_card` in `player.rs`). The fold
reuses `Cartwall::open` and `Command::SetCartwallOpen`; no model change.

**Tech Stack:** Rust, egui 0.37, egui_kittest.

**Spec:** `docs/superpowers/specs/2026-10-28-phone-layout-design.md` §16

## Global Constraints

- Phones only; tablets and the desktop unchanged.
- No new strings expected (the fold reuses `cartwall-title`; the mini
  card reuses the transport's `tip-*` labels and `player-label`).
- The lock (PL12) does not hold the mini card's buttons (it holds faders
  and waveforms only).

## Review Focus

1. Folded Live, then the Carts tab: the grid shows, unfolded.
2. Folded Live with carts playing: *Stop all* in the header stops them.
3. Eight players in landscape: the mini cards scroll, each row whole.
4. A long title in German: cut short, the buttons stay whole.
5. A paused player's mini card: Play blinks as on the card.

---

### Task 1: the Live cartwall folds (PL35)

**Files:** `cartwall.rs` (`strip`: the phone never folds on Carts, folds
on Live; `header`: caret on Live), `touch.rs` (`live`: folded, the
header at the bottom and the cards over the rest); tests in
`touch_ui.rs`.

- [ ] Tests: on upright Live the header's "Cartwall" title sends
  `SetCartwallOpen(false)`; with `cartwall.open = false` Live shows no
  cart button and the cards area reaches the header; the Carts screen
  with `open = false` still shows the carts; in landscape folded the
  cards take the whole width.
- [ ] Implement; commit.

### Task 2: the mini card (PL36)

**Files:** `player.rs` (`pub(crate) fn mini_card`), `touch.rs` (`live`
in landscape); tests in `touch_ui.rs`.

- [ ] Tests: landscape Live shows every player's Play, Pause and Stop
  (48 high) for three players at the A56 size; a tap on the row opens
  Playlists with that player; Play sends `Command::Play`; portrait keeps
  the full card (its Restart button shows).
- [ ] Implement; commit.

### Task 3: docs and review

- [ ] `docs/user/android.md`, `docs/technical/mobile.md`; release APK to
  `/tmp`; fresh reviewer; PR.
