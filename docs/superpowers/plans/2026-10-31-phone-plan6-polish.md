# Phone layout plan 6: polish after the first trials — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The phone layout spec §15 (PL25–PL34): what the maintainer found
on a Samsung A56 — the row drag, the tabs, the landscape Playlists card,
the transport and times, the meters, battery use and every screen fitting.

**Architecture:** UI-only changes in `fp-app/src/ui` (phones only, behind
`view_state.phone_active`), one repaint decision in `app.rs`, no model or
engine rules. Each task ends with emulator screenshots at 1080×2340,
450 dpi (`adb shell wm size 1080x2340; wm density 450`).

**Tech Stack:** Rust, egui 0.37, egui_kittest, adb.

**Spec:** `docs/superpowers/specs/2026-10-28-phone-layout-design.md` §15

## Global Constraints

- Phones only; tablets and the desktop look and behave as before (their
  existing tests stay green unchanged).
- New strings in all 11 locales; none expected (icons reuse the tab names).
- No platform code outside `fp-mobile` and the `Platform` seam (iOS later).
- The UI never blocks; nothing new on the audio thread.

## Review Focus

1. A long locale (de-DE) on an upright phone: the top bar fits, nothing
   cut with "…" (PL26).
2. Eight players in landscape: the chips in the top bar scroll, the tabs
   and the lock stay reachable (PL27).
3. A card squeezed to its minimum: transport and times still drawn, no
   widget overlapping another (PL28).
4. Nothing playing and every meter at the floor: no frame is drawn for
   a second at a time except the clock (PL32).
5. A cart grid of 8 columns in Settings on an upright phone: no sideways
   scroll (PL33).

---

### Task 1: row drag on a device (PL25) — done in `074f4b7`

### Task 2: tabs as icons, chips in the top bar, legend (PL26, PL27, PL28 legend)

**Files:** `touch.rs` (`screen_tiles`, `playlists`), the top bar in
`app.rs`, the legend bar; tests in `touch_ui.rs`.

- [ ] Tests: portrait tabs are found by their names (`Role::Button`
  labels) and draw no "…"; the chosen tab's name is drawn when there is
  room; in landscape on Playlists the chips are in the top bar (their
  rect above the top bar's bottom) and the screen has no chip row; the
  legend is absent in landscape on a phone, present on a tablet.
- [ ] Implement; commit.

### Task 3: the card fits its space; bigger transport; both times (PL28, PL29)

Ruling (in the ledger): PL34 is not built — with 48 dp buttons a card is
~300 dp; a one-row mini card per player in landscape Live is proposed to
the maintainer instead.

**Files:** `player.rs` (the card), `touch.rs` (`live` landscape heights);
tests in `touch_ui.rs`.

- [ ] Tests: on a phone the transport buttons are ≥ 48 high; the card
  shows elapsed and total (`00:00` and `00:40` for a 40 s track); in
  landscape Playlists the Play button is inside the screen; in landscape
  Live two cards' Play buttons are visible at once.
- [ ] Implement a compact card (height below natural: shorter waveform,
  no cover, one-line title); commit.

### Task 4: compact meters, smooth fall (PL30, PL31)

**Files:** the meter and fader widgets, `view.rs` (`animating`), `app.rs`.

- [ ] Tests: `animating` is true while a stopped player's meter reading
  is above the floor and false once it is at the floor; on a phone the
  meter draws no scale numbers and is narrower than on a tablet.
- [ ] Implement; screenshots before/after for the maintainer; commit.

### Task 5: battery (PL32)

**Files:** `app.rs` (repaint decision), the helper threads' senders.

- [ ] Tests: idle (nothing moving) asks for the next repaint at the next
  clock second, never at 100 ms; a picker answer, a tag sheet and a
  notice each wake the UI (`request_repaint` from the sender's side).
- [ ] Measure `dumpsys gfxinfo` frames over 30 s idle before and after;
  record in the ledger; commit.

### Task 6: every screen fits (PL33)

**Files:** `settings.rs` (cartwall editor and the other sections),
windows and menus as found.

- [ ] Walk every Settings section, the About window, the tag editor, the
  row and track menus on the emulator in both orientations; list what
  overflows in the ledger.
- [ ] Test and fix each (first: the cart grid wraps to the width; the
  rows × columns controls stacked); commit.

### Task 7: docs, screenshots, review

- [ ] `docs/user/android.md`, `docs/technical/mobile.md`; screenshots of
  every screen to the maintainer; fresh reviewer; PR.
