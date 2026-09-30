# Feedback Plan 6 — Per-Entry Repeat and Stop Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let the operator mark a playlist entry to repeat until they move on
(F4), or to stop the player after it every time it plays (F5), with icons in
the table and context-menu toggles.

**Architecture:**
- `fp-model`:
  - `PlaylistEntry` gains `repeat` and `stop_after`;
  - two new toggle commands;
  - `plan_for` and `reconcile` implement R26 and R27.
- **Repeat reuses the segue machinery.** While an entry repeats, the source
  the model preloads is the current entry itself at its cue-in, and the plan
  is the existing `StartNextAt { at_secs: cue_out, fade_current_until_secs:
  None }`. The engine therefore restarts the entry gaplessly and
  sample-accurately, with no overlap and no fade, exactly like a hard
  continuous transition. No new engine code and no new `TransitionPlan`
  variant are needed.
- A `TransitionStarted` for the entry that is already current clears the
  model's `preloaded` and `scheduled`, so `reconcile` preloads and plans the
  next pass.
- `fp-app`:
  - icons in the title cell;
  - checkable context-menu items.

**Tech Stack:** Rust 2024, serde, egui 0.36, the Offline backend for the
conductor test.

**Spec:** [`docs/superpowers/specs/2026-09-30-operator-feedback-design.md`](../specs/2026-09-30-operator-feedback-design.md) §2.1, §2.2 (R26, R27), §2.3 (entry flags). Roadmap: plan 6.

## Global Constraints

- The flags are serialised as `#[serde(default, skip_serializing_if =
  "is_false")]`, so files from before this plan load unchanged.
- Duplicating an entry copies both flags.
- R27 wins over R26. `stop_after_current` ends a repeating pass (the player
  stops).
- Pause keeps a repeat. Play (next), Previous, Stop and Fade stop end it.
- A repeating entry is not marked played and not pushed to the history while
  it repeats.
- UI strings, in both locales:
  - `menu-repeat` = "Repeat this track" / "Repetir esta pista";
  - `menu-stop-after` = "Stop after this track" / "Parar después de esta
    pista";
  - `flag-repeat` = "Repeats" / "Se repite";
  - `flag-stop-after` = "Stops after" / "Para al acabar".
- English. Gate before every commit. Branch: `feat/entry-repeat-stop`.

## Review Focus

- **A repeating entry with no known cue-out** (an unanalysed track). The plan
  uses the source end, as a hard transition does, and must not busy-loop
  (Task 3 test).
- **Removing or moving a repeating entry while it plays.** Nothing panics,
  and the next pass follows the model (Task 3 test).
- **The same entry repeating on two players.** Each player's pass is its own
  (Task 3 test).
- **Toggling repeat during the last second.** Re-planning never starts the
  next track and the repeat both (Task 4 conductor test).
- **An old `playlists.json`** loads with both flags off (Task 1 test).

---

### Task 1: Entry flags and toggles

- `PlaylistEntry { …, #[serde(default, skip_serializing_if = "is_false")] pub repeat: bool, … stop_after: bool }`; `Playlists::duplicate` copies them.
- `Command::ToggleEntryRepeat(EntryId)`, `Command::ToggleEntryStopAfter(EntryId)` (unknown entry → `ModelError::UnknownEntry` or the existing equivalent).
- Tests: toggles flip; duplicate copies; serde round trip; JSON without the fields loads false; a serialised entry without flags has no `repeat` key (fp-model tests and an fp-store round trip).

### Task 2: R27 stop after this entry

- `plan_for`: the current entry has `stop_after` → `StopAt { at_secs: end }` in any mode, before every other rule.
- Tests (`fp-model/tests/entry_flags.rs`): Continuous with a segue → StopAt; Single → StopAt; on `ReachedEnd` the player stops, the flag is still set, `next` is the following entry; `stop_after` wins over `repeat`.

### Task 3: R26 repeat in the model

- `fn repeating(state, player) -> bool`: transport not Stopped, no fade stop, not `stop_after_current`, current entry has `repeat` and not `stop_after`.
- `plan_for`: repeating → `StartNextAt { at_secs: end, fade_current_until_secs: None }` (Single mode too).
- `reconcile`: the preload target is the current entry (from cue-in) while repeating, else `next`.
- `on_event(TransitionStarted { entry })` with `entry == current` while repeating: clear `preloaded` and `scheduled` (reconcile re-issues both). Nothing marked played, no history push.
- Tests: Single and Continuous plan and preload; a pass keeps `current`, emits a new `Preload` of the same entry and a new `Schedule`; pause/resume keeps the plan; Play (next) moves on and the old entry is marked played; Previous and Stop end it; stop-after-current turns the plan into StopAt; toggling repeat while playing re-plans (Preload of next, StartNextAt with the segue); an entry without cue-out plans at the source end; the same entry repeating on two players; removing the repeating entry is refused (on air) as today.

### Task 4: Conductor test (engine)

- `fp-engine/tests/conductor.rs`: an Offline conductor plays a short repeating entry (generated WAV, 1 s, cue-out 1 s) and runs past three boundaries: three `TransitionStarted` for the same entry, the model's `current` unchanged, the position returns to cue-in after each boundary, and no `ReachedEnd`. Then `ToggleEntryRepeat` off: the next boundary advances to the next entry. Follow the existing conductor test harness (time driven explicitly).

### Task 5: UI

- `table.rs`: at the right of the title cell, `egui_phosphor::regular::REPEAT` (repeat) and `icons::stop_after` (stop after), in the row's text colour, each a small label with accessible name `flag-repeat` / `flag-stop-after`. Context menu: two checkable items (`ui.checkbox`-style or selectable with a check glyph) sending the toggles.
- Tests (kittest): the menu items send the toggles; a flagged entry shows its icon labels; an unflagged one does not.

### Task 6: Docs, review, PR

Main spec §3 (rules for R26, R27), §8.3 track table (icons, menu); `docs/user/playlists.md` (menu items), `players.md` (repeat and stop-after behaviour); `docs/technical/audio-engine.md` (repeat as a same-entry preload), `persistence.md` (entry flags). Review, PR `feat: per-track repeat and stop after`, CI, merge.
