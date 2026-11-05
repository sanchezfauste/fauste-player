# Track library plan 9 — the phone's pre-listen card Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give the phone's Library tab the desktop preview bar's seek bar
and marker editing, in a card under the rows that the preview strip opens.

**Architecture:** UI only (`fp-app`): the model's library CUE already
seeks, sets where Play starts while stopped, and keeps its track after
Stop. The card reuses `wave_panel::show` (`WaveKey::Library`), which
already handles taps, long-press markers, pinch zoom, Full view and the
padlock on a phone.

**Tech Stack:** Rust, egui 0.37 / egui_kittest.

**Spec:** `docs/superpowers/specs/2026-11-01-track-library-design.md`
§11.3 (LB54–LB57).

## Global Constraints

- English in every file; never mention other playout or radio-automation
  products.
- No hardcoded product limits; no new config (the card's open state is UI
  state, not saved).
- `unwrap`/`expect`/`panic` denied outside tests.
- Every new string in every locale (en-US and es-ES written, the rest
  translated).

## Review Focus

- The rows never move when the card opens or closes or a preview starts
  (LB49 still holds for the strip; the card takes room from the bottom).
- A tap on the strip's Pause or Stop does not also toggle the card.
- Locked: a tap on the card's waveform neither seeks nor sets a start.
- The track leaving the library closes the card and empties the strip.
- Ticked rows: the bar stays at the bottom and the card above it.

---

### Task 1: The strip opens the card

**Files:** `crates/fp-app/src/ui/phone_library.rs`; tests in
`crates/fp-app/tests/it/phone_library_ui.rs`.

- [ ] `PhoneLibraryView` gets `pub card_open: bool`. `back()` on the
  Tracks page closes an open card first and returns true (LB57).
- [ ] `preview_strip` senses a click on its whole rect; a tap outside the
  buttons toggles `card_open` while `library_cue_view` is `Some`. With a
  track it draws a caret on its right (`icon::CARET_UP` closed,
  `CARET_DOWN` open). While the card is open and the preview is stopped,
  the strip shows Play (sends `PlayLibraryCue`, enabled per
  `command_available`) and the title.
- [ ] Tests: `lb54_a_tap_on_the_strip_opens_the_card`,
  `lb54_a_tap_on_stop_leaves_the_card_as_it_was`,
  `lb54_no_track_no_card`, `lb57_back_closes_the_card_first`.

### Task 2: The card

**Files:** new `crates/fp-app/src/ui/phone_prelisten.rs` (the card), the
`ui` module list, `phone_library.rs` (`tracks_page` layout); tests.

- [ ] `pub(crate) fn card(ui, scene, view_state, v: &LibraryCueView)`: the
  names; `wave_panel::show` with `WaveKey::Library`, `editable_markers:
  true`, `locked: touch::locked(..)`, `follow` while Playing, the seek sent
  as `SeekLibraryCue`; the time row (`view::time_text`, countdown); Play,
  Pause and Stop at `touch::TARGET` with `widgets::vu_compact` on the
  right. The waveform takes the height left, between the players' phone
  minimum and maximum.
- [ ] `tracks_page`: with the card open and a view, the rows get the room
  above the card (upright, the card at most half) or the left half (sideways);
  the ticks' bar stays under both.
- [ ] Tests: `lb56_a_tap_on_the_card_wave_seeks`,
  `lb56_locked_the_card_wave_does_not_seek`,
  `lb54_the_card_closes_when_its_track_leaves_the_library`,
  `lb55_the_rows_stay_put_when_the_card_opens`.
- [ ] Locales: the caret's accessible label (`phone-library-card-open`,
  `phone-library-card-close`) in every `main.ftl`.

### Task 3: Docs and screenshots

- [ ] `docs/user/android.md` (Pre-listen), `docs/technical/mobile.md`,
  `docs/technical/ui.md`.
- [ ] `scripts/package/play-screenshots.sh`: the 5-library shot shows the
  card open on a paused preview (mask the times and meter); retake the 11
  languages; `docs/images/android/5-library.png`; `play-listing.py check`.

## Not in this plan (rulings)

- The card on a tablet: a tablet has the desktop's library window.
- Volume for the library CUE: it has none on the desktop either.
