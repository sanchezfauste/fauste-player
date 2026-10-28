# Phone layout plan 3: the Playlists screen — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The phone's Playlists screen of the phone layout spec §6
(PL8–PL11b): the track list over the chosen player's full card, a row
menu with the moves a finger needs, a drag handle, a short footer and
markers by touch.

**Architecture:** UI over existing commands (`MoveEntry`, `SetMarker`,
`SetNext`, `RemoveEntry`). One pure helper joins `fp-model` (where an
entry goes "after the next" of a player), shared with plan 4's *Add after
the next*. Everything else is in `fp-app/src/ui` (`touch.rs`,
`player.rs`, `table.rs`, `wave_panel.rs`), on a phone only
(`view_state.phone_active`): tablets and the desktop do not change.

**Tech Stack:** Rust, egui 0.37, egui_kittest.

**Spec:** `docs/superpowers/specs/2026-10-28-phone-layout-design.md`

## Global Constraints

- Phones only; tablets (shortest side ≥ 600 dp) and the desktop unchanged.
- No new player rules: every action is an existing `Command`.
- New strings in all 11 locales, literal text where English has no term.
- The lock (PL12) holds the Playlists card's fader and waveform; the row
  menu, the handle and the footer are never locked.
- `unwrap`/`expect`/`panic` only in tests; the UI never blocks.

## Review Focus

1. Move up on the first row and Move down on the last: disabled, no command.
2. *Move to another player…* for the entry on air: disabled (PL10b).
3. *Move to another player…* when that player has no next, or its next is
   in another playlist than the one it shows: the end of the shown playlist.
4. A finger drag on a row away from its handle scrolls and moves nothing.
5. A long press on the Live card's waveform opens no marker menu (PL11b).

---

### Task 1: the Playlists screen (PL8, PL9)

**Files:** `crates/fp-app/src/ui/touch.rs` (`playlists`),
`crates/fp-app/src/ui/player.rs` (`playlists_screen`, `card` sizes),
tests in `crates/fp-app/tests/it/touch_ui.rs`.

**Interfaces:** Produces `player::playlists_screen(ui, scene, view_state,
covers, index, landscape)`; `card` gains the waveform height and whether
markers are editable (`CardWave { height, markers }`).

- [ ] Tests: `the_playlists_list_is_over_the_card` (portrait: the first
  row's bottom is above the card's Play), `in_landscape_the_list_is_left_of_the_card`,
  `the_playlists_card_has_a_larger_waveform` (taller than the Live card's
  44), `the_playlists_screen_has_tabs_and_a_new_playlist_button`. Watch them fail.
- [ ] Implement: chips, then tabs (with +), then the table and footer in
  what the card leaves (portrait: the card at the bottom, its height
  remembered from the last frame; landscape: the list on the left half,
  the card on the right); waveform 72 pt with markers editable.
- [ ] Run the touch_ui tests, commit.

### Task 2: the row menu and the handle (PL10, PL10b)

**Files:** `crates/fp-model/src/playlist_rules.rs` or the module that
holds `next` helpers (`after_next`), `crates/fp-app/src/ui/table.rs`,
tests `crates/fp-model/tests/it/after_next.rs`,
`crates/fp-app/tests/it/touch_ui.rs`.

**Interfaces:** `fp_model::after_next(state: &AppState, player: PlayerId)
-> Option<(PlaylistId, usize)>`: the playlist the player shows and the
index right after its next entry when that playlist holds it, else its
length; `None` for an unknown player.

- [ ] Model tests: next in the shown playlist → index after it; no next →
  the end; next in another playlist → the end.
- [ ] UI tests: *Move up* sends `MoveEntry { index: pos - 1 }`, *Move down*
  `pos + 2` (a boundary before removal); both disabled at the ends;
  *Move to another player…* → P2 sends `MoveEntry` to `after_next(P2)`;
  disabled on air; a drag from the ≡ handle to another row moves the
  entry, a drag elsewhere on the row moves nothing.
- [ ] Implement: on a phone the row menu adds *Move up*, *Move down* and a
  *Move to another player* submenu (players other than this one); the
  Number cell draws a ≡ handle sensing drags that starts the existing
  `DragEntry` drag-and-drop.
- [ ] Strings: `menu-move-up`, `menu-move-down`, `menu-move-to-player`,
  `tip-drag-handle` in 11 locales. Commit.

### Task 3: the footer (PL11)

**Files:** `crates/fp-app/src/ui/player.rs` (`footer`), tests in
`touch_ui.rs`.

- [ ] Tests: on a phone the footer shows *Add* and a ⋯ tile; ⋯ opens a menu
  with *Reset played*; the desktop footer is unchanged.
- [ ] Implement; string `tip-footer-more`. Commit.

### Task 4: markers by touch (PL11b)

**Files:** `crates/fp-app/src/ui/wave_panel.rs` (`edit_markers`), tests in
`crates/fp-app/tests/it/touch_ui.rs` (or `markers_ui.rs`).

- [ ] Tests: a long press on the Playlists waveform opens the menu with
  the time pressed and *Set cue-in here* sends `SetMarker` at that time;
  a manual marker offers *Remove* (`SetMarker { secs: None }`); a long
  press on a Live card's waveform opens nothing.
- [ ] Implement *Remove* entries for each manual marker (all form
  factors would show them; on a phone only, to keep the desktop as is);
  strings `wave-remove-cue-in`, `-intro`, `-outro`, `-mix`, `-cue-out`.
  Commit.

### Task 5: documentation and the emulator

- [ ] `docs/user/android.md`, `docs/technical/mobile.md`; screenshots of
  the Playlists screen (portrait, landscape, the row menu) on the
  emulator with the release APK. Commit.
