# Phase 2 · Plan 3 — Interface: cartwall, full Settings, playlist files, marker editing

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans (or superpowers:subagent-driven-development) to implement this plan task by task.

**Goal:** Put the Phase 2 capabilities (plans 1 and 2) in the operator's hands, reaching parity with the v3 design:
- the cartwall strip;
- remappable shortcuts;
- Settings: Cartwall, Keyboard shortcuts, language, cartwall outputs, playlist import/export;
- marker editing on the waveform.

**Architecture:**
- **Everything goes through the `Controller`** (commands) and the snapshots. As before, file dialogs, playlist parsing and cart-page parsing run on helper threads, and their results come back through channels.
- **Shortcuts:** the keyboard handler maps the configured chords to commands. Players and carts are resolved by position.
- **Language:** a change rebuilds the `I18n` when `config.ui.language` changes.

**Spec:** [`docs/superpowers/specs/2026-09-26-phase2-cartwall-settings-design.md`](../specs/2026-09-26-phase2-cartwall-settings-design.md) (P2.5, P2.6, P2.7, P2.8, P2.9).

## Global Constraints

- Rules carried over: English; every string in both `en-US` and `es-ES`; no third-party names; the UI thread never blocks; no unwrap, expect or panic; TDD with kittest (`with_step_dt(0.02)`); the v3 design and Nocturne theme.

## Review Focus

1. **A shortcut pressed while a text field has focus**, or held down: ignored (the Phase 1 rules).
2. **Rebinding in Settings to a chord that another action uses:** the conflict is shown before saving, and saving moves the chord.
3. **Importing a very large or malformed playlist file:** the UI stays responsive (parsing happens off the UI thread), and a notice explains the result (entries added, streams skipped, or the error).
4. **Dragging a marker handle past cue-out, or past another marker:** the model clamps or refuses, the UI shows the refusal, and nothing jumps.
5. **The cartwall with many pages and a small window:** tabs scroll, buttons keep a minimum width, and nothing overlaps the players.

---

### Task 1: Cartwall strip

- A collapsible strip under the players (spec P2.9):
  - the header shows the caret, **CARTWALL** (click to collapse, `SetCartwallOpen`), the page tabs (`ShowCartPage`, with a red dot when a cart on that page plays) and the hint;
  - below it, a grid of buttons.
- **Button:**
  - kind dot, name, kind label, time: the length, or a `-mm:ss` countdown from telemetry;
  - a red border and a translucent red progress bar while playing;
  - a warning style for an unavailable file, and a dimmed style for an empty cart;
  - click fires or stops it (`FireCart`);
  - right-click opens: *Pre-listen on CUE* (`CueCart`), *Stop*, *Edit…* (opens Settings on that cart).
- An empty page name shows as the localised "Carts n".
- **Tests (kittest):**
  - `clicking_a_cart_fires_it`;
  - `the_cartwall_collapses_and_expands`;
  - `a_playing_page_shows_a_dot_on_its_tab`;
  - `an_empty_cart_does_nothing`;
  - `cart_countdown_uses_telemetry`.

### Task 2: Shortcut dispatch

- Replace the fixed 1–9 handling with `config.shortcuts`:
  - each binding's key name is mapped with `egui::Key::from_name`, and unknown names are ignored (logged once);
  - only first presses count, and nothing fires while a text field has focus;
  - `PlayPlayer(n)` → `Play`, `PausePlayer` → `Pause`, `StopPlayer` → `Stop`, `FadeStopPlayer` → `FadeStop`, `CuePlayer` → `ToggleCue`;
  - `FireCart(n)` → `FireCart` for cart *n* of the page shown;
  - `StopAllCarts`;
  - `ToggleCartwall`, `NextCartPage`, `PreviousCartPage`.
- `Delete`, `Backspace` and `Esc` stay fixed.
- **Tests:**
  - `number_keys_play_players` (still green through the defaults);
  - `function_keys_fire_carts_of_the_page_shown`;
  - `ctrl_space_stops_all_carts`;
  - `a_rebound_key_plays_the_new_target`;
  - `shortcuts_are_ignored_while_editing_text` (still green).

### Task 3: Settings — Cartwall, Keyboard shortcuts, language, outputs

- **Cartwall section:**
  - page tabs;
  - **New page**;
  - **Import…** and **Export…**: `parse_cart_page` / `write_cart_page` on helper threads, reached through rfd dialogs;
  - **Delete page**, with the refusal shown;
  - page name, and rows × cols (`ResizeCartPage`, with the refusal shown);
  - the grid, where a click selects a cart;
  - the cart editor: name, file (**Choose…** runs `AssignCartFile`, **Clear** runs `ClearCartFile`), kind, **Loop**, **Stop other carts when fired**.
- **Keyboard shortcuts section:**
  - one row per action, with the actions listed for the configured players and the carts of the largest page;
  - click a row, then press a chord to bind it (`SetShortcut`);
  - a conflict note names the action that loses the chord;
  - **Unbind**, and **Reset to defaults**.
- **Language:** System / English / Español in Players. `config.ui.language` changes, and the app rebuilds its `I18n` on the next frame.
- **Audio outputs:** a **Cartwall** row with Main and Cue pickers and test buttons (`config.outputs.cartwall`), plus the restart note.
- **Tests:**
  - `a_cart_can_be_renamed_and_made_exclusive`;
  - `deleting_the_last_cart_page_shows_the_refusal`;
  - `binding_a_shortcut_shows_and_resolves_the_conflict`;
  - `choosing_spanish_switches_the_interface`.

### Task 4: Playlist import and export

- **Settings → Playlists:**
  - **Import M3U / PLS…**: an rfd dialog, then read and `parse_playlist` on a helper thread, then `CreatePlaylistFromPaths` named after the file. A notice gives the entries added and the streams skipped, or the error;
  - **M3U** on each playlist row: a save dialog, then `write_m3u8` on a helper thread, with a notice.
- **Tests:**
  - `imported_playlists_become_new_playlists`: inject a picked path through a test seam, since rfd cannot run under kittest;
  - `exported_playlists_are_valid_m3u8`: a pure function, `export_entries(state, playlist)`.

### Task 5: Marker editing on the waveform

- **Right-click on the waveform:**
  - *Set cue in here*, *Set intro end here*, *Set outro start here*, *Set MIX point here*, *Set cue out here*;
  - *Reset markers to automatic*.
- **Alt/Option held:** handles appear on the markers. A drag shows a time label and commits a single `SetMarker` on release. Refusals appear as notices.
- The intro and outro badges and shading follow the new values.
- **Tests:**
  - `the_waveform_menu_sets_the_intro_here`;
  - `alt_dragging_the_mix_marker_moves_it`;
  - `reset_markers_is_sent`.

### Task 6: User documentation and verification

- **User guide:**
  - `cartwall.md` (new);
  - update `settings.md`, `keyboard.md`, `markers-and-mixing.md`, `playlists.md`, `README.md` (index).
- **README:** features, and the Roadmap marked "Phase 2 done".
- Screenshot of the main screen with the cartwall.
- Full checks, a fresh review, fixes, then merge.
