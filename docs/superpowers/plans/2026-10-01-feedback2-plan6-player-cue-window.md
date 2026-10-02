# Player and CUE Window (Feedback 2, Plan 6) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The player says when the current entry repeats or stops after itself (O8); a click on the waveform seeks and a drag pans the zoomed view (O10); every CUE opens a floating window with its waveform, time, Pause/Resume, Stop and "Load as next" (O12); a running CUE follows the player's next and the selected row (O17).

**Architecture:**
- **Model first.** `CueState` gains `paused`. Three commands (`SeekCue`, `SetCuePaused`, `CueToNext`) and two engine actions (`SeekCue`, `SetCuePaused`) are pure rules with one test each. O17's "setting next moves the CUE" is a rule on `Command::SetNext`. O8's banner choice is a pure function, `fp_model::entry_notice`.
- **Engine.** The CUE source gains seek and pause through the same mechanisms the current source already uses (a replacement source for a seek, `BusCommand::Pause`/`Resume` for a hold), tested with the Offline backend. The CUE position already reaches the UI as `PlayerTelemetry::cue_position_secs`; the paused flag reaches it through the model snapshot (`PlayerState::cue`).
- **UI.** `widgets::waveform` stops previewing and seeking on drag and reports the sideways drag instead (`WaveOutput`); the player pans a zoomed view with it. A new `ui/cue_window.rs` draws one non-modal `egui::Window` per running CUE from a pure `view::cue_window_view`. All transport icons go through plan 4's `ui/glyphs.rs`. The table sends `CueEntry` on a single click while a CUE runs (`view::cue_follow_target`).

**Tech Stack:** Rust, egui/eframe 0.36.2, egui_kittest 0.36.2, the `Offline` audio backend for engine tests. No new dependency.

**Spec:** `docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md` §7 (items O8, O10, O12, O17). Roadmap: `docs/superpowers/plans/2026-10-01-operator-feedback-2-roadmap.md` (plan 6, branch `feat/cue-window`; depends on plan 4's `ui/glyphs.rs`). Related rules: main spec §3 R26/R27 (entry repeat and stop-after), the first feedback spec F2 (replaced by O10) and F17 (zoom follow).

## Global Constraints

- All code, identifiers, comments, docs and commit messages are in English. Never mention other products.
- Spec O8: "When the current entry has per-entry repeat (F4) or stop after this entry (F5), the player header shows a notice in the style of stop after current: "This track will repeat" or "Stops after this track". Stop after current, when also set, takes precedence."
- Spec O10: "A press and release that stays within the drag threshold seeks there. Dragging pans the zoomed view and never seeks. Without zoom, a drag does nothing. The F2 seek preview on drag is removed. Alt-drag marker editing is unchanged."
- Spec O12: "When a player starts a CUE, a floating, non-modal window opens for it. Several CUE windows stack. It shows: the title and artist; the waveform with the CUE position, where a click seeks the CUE; the elapsed and remaining time; and the buttons Pause/Resume, Stop and Load as next. Load as next makes the cued entry the player's explicit next and keeps the CUE running. Closing the window stops the CUE."
- Spec O12 model: "`CueState` gains `paused: bool`; new commands `SeekCue(PlayerId, f64)`, `SetCuePaused(PlayerId, bool)` and `CueToNext(PlayerId)`; each command has rule tests." Engine: "the CUE source supports seek and pause, with Offline-backend tests. The CUE position reaches the UI through the snapshot."
- Spec O17: "While a player's CUE is running: setting that player's next (double-click) moves the CUE to the new next, from its cue-in. This is a model rule on `SetNext`. Selecting a row with a single click in that player's table moves the CUE to that entry. Selection is UI state, so the UI sends `CueEntry`."
- Roadmap note: "Plan 6 draws the CUE window's icons through plan 4's `ui/glyphs.rs`." `tests/glyphs.rs` guards that the screens never name `fill::STOP`, `fill::PLAY`, `fill::FAST_FORWARD`, `HEADPHONES` or `icons::`.
- Behaviour lives in `fp-model` as pure functions; the engine and the UI only execute and display. The UI never blocks (no waits on the CUE; everything is a command). No `unwrap`, `expect`, `panic` or indexing outside tests; `fp-engine` denies `clippy::indexing_slicing` (use `get`). NaN or infinite times from the UI never reach the engine.
- Real-time safety: nothing in this plan runs on the device callback. The engine additions run on the conductor thread and use only existing `BusCommand`s (`Pause`, `Resume`, `Attach`, `Detach`).
- Nothing goes on air by itself: the CUE plays on the Cue route only; "Load as next" changes the next pointer and starts nothing; a CUE that is moved never touches the Main sources.
- No hardcoded product limits: the window size and waveform height are visual constants in `ui/cue_window.rs`; there is no new operator setting, so no `Config` field is added.
- UI strings are Fluent messages in `crates/fp-app/locales/en-US/main.ftl` (source) and `es-ES/main.ftl`, always both (`tests/i18n.rs::both_locales_define_the_same_keys` checks it).
- Commands: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p <crate> --test <file> <name>` (one filter per command). Build only into the repo's `target/`.
- Commit only when fmt, clippy and the whole suite pass (`cargo test --workspace`). Commits end with the co-author trailer the harness provides. `CHANGELOG.md` is never edited by hand.
- Documentation is part of the change (Task 7).

## Review Focus

Failure modes the spec implies and its tests do not name; each has a test in the task named.

1. A CUE that is paused and then moved (double-click sets another next, or a row is clicked): the new CUE must play, not stay silently paused, and the model flag and the engine flag must agree. (Task 1 `moving_a_paused_cue_plays_the_new_one`; Task 3 `a_new_cue_after_a_pause_plays`.)
2. Seeking a paused CUE: it must stay paused at the new place, not start playing. (Task 1 `seeking_a_paused_cue_keeps_it_paused`; Task 3 `seeking_a_paused_cue_stays_paused_at_the_new_position`.)
3. Commands that arrive after the CUE has gone (it ended by itself, the entry was removed, the window's button was pressed twice): `SeekCue`, `SetCuePaused` and `CueToNext` without a CUE do nothing and report no error; the engine ignores a seek or pause with no CUE source; a NaN seek is ignored. (Task 1 `commands_without_a_cue_do_nothing`, `a_broken_seek_is_ignored`; Task 3 `cue_commands_without_a_cue_source_are_ignored`.)
4. Selecting a row while a CUE runs: the row already cued, a file that is missing or unreadable, and a player with no CUE must not send `CueEntry` (which would restart the CUE or start a doomed source). A double-click is two clicks and a `SetNext`: the CUE moves once. (Task 1 `setting_the_cued_entry_as_next_keeps_it_running`, `setting_next_to_an_unplayable_entry_leaves_the_cue`; Task 6 `clicking_the_cued_row_a_missing_file_or_without_a_cue_sends_nothing`.)
5. A track whose duration is still unknown (0) shown in the CUE window: no NaN, no seek, `00:00` times, and the window still has Stop and Pause. A drag on a stopped or unzoomed waveform does nothing; a drag that starts on the Full view button does not pan. (Task 5 `a_cue_without_a_known_length_shows_zero_and_cannot_seek`; Task 4 `a_drag_without_zoom_does_nothing`, `dragging_from_the_full_view_button_does_not_pan`.)

## Decisions

- **Banner wording.** The spec gives the sentences "This track will repeat" and "Stops after this track". The header is narrow, so the outlined badge (the style of the Stop after badge) carries a short text (`Repeat`, `Stop after track`) and the sentence is its tooltip and accessible name. Both badges use the amber of the Stop after badge. Rule (`fp_model::entry_notice`): none while the player is stopped, while a fade stop runs, or while stop after current is set (its own badge says it); otherwise the entry's `stop_after` gives `StopsAfter` (it wins over repeat, as R26/R27 do); otherwise `repeat` on a playable file gives `Repeats` (a file that can no longer be opened plays its pass out, so it does not repeat).
- **Drag threshold.** "Within the drag threshold" is egui's own: `Response::clicked()` (a press and release that did not move more than egui's click distance and did not outlast its click duration). No custom threshold. A long press without moving is not a click and seeks nothing.
- **Hover time stays.** The hover line with the time under the pointer is kept (only the drag preview goes). It is hidden while a drag is held.
- **Pan applies only to a zoomed view.** `waveform` reports `pan_dx` for any primary drag that starts on it, outside the Full view button, without Alt; the player applies it only when the view is zoomed. Alt-drag stays marker editing and never pans.
- **Follow grace.** A held drag still pins the zoomed view against playhead following (F17), as before (`widgets::pan_dragging` replaces `seek_dragging`).
- **CUE plays whole files.** The CUE source request has no end, so the CUE window's waveform covers the whole file, its seek range is `0..duration`, and "remaining" counts to the end of the file, not to the cue-out. It starts at the play range start, as today.
- **Moving a CUE resumes it.** `cue_entry` (used by `CueEntry` and by the O17 rules) creates `CueState { entry, paused: false }` and the engine's `StartCue` clears its pause flag: listening to the new entry is the point of moving the CUE.
- **O17 `SetNext` rule.** After a successful `set_next`, a running CUE moves to the new next unless it is already on that entry (no restart; this also makes the double-click case, click then `SetNext`, move the CUE once) or the entry cannot be played (`AppState::playable_request` is `None`; the CUE is left as it is). Other ways the next changes (the playlist advancing, entry removal) are not part of O17 and are unchanged.
- **Click rule lives next to the view code.** The single-click rule is a pure `view::cue_follow_target(state, player, clicked) -> Option<EntryId>` (UI selection is not model state, so it is not in `fp-model`); it is the only place that decides, and it has unit tests. Only a primary click sends it (not a right-click or a drag).
- **`CueToNext`.** Uses `set_next` on the cued entry, so the cued entry already being the derived next becomes explicit (it then stops following playlist edits), the current entry is refused with `ModelError::NextIsCurrent` (the UI disables the button for it), and the CUE keeps running (no action is produced).
- **Idempotent, silent no-ops.** `SetCuePaused(p, same)` and every CUE command without a CUE return no actions and no error (as `SetCue` does).
- **Window.** One `egui::Window` per running CUE, id `("cue-window", player)`, no title bar of egui's (a custom row with the title and a close button, so its accessible names are ours), not resizable, default position staggered by the player index so several stack. The window is not modal. It is drawn before Settings, About and the close guard, which stay on top of it. Pressing the close button or Stop sends `SetCue(player, false)`.
- **Icons.** Pause and Resume use `TransportAction::Pause` and `TransportAction::Play`, Stop uses `TransportAction::Stop`, the window title uses `TransportAction::Cue`, all through `glyphs::paint` / `glyph_text`. "Load as next" is not a transport action: like the table's "Set as next" menu item it uses the Phosphor `ARROW_BEND_DOWN_RIGHT` glyph, and the close button uses `X`. `tests/glyphs.rs` gains `cue_window.rs` in its guarded sources.
- **Remote and MIDI.** The remote API and MIDI keep `SetCue` and `CueEntry`; pausing and seeking the CUE are not exposed there. `CueDto` stays `{ entry }` (the remote events already notice a CUE change through `PlayerState::cue`, which now includes `paused`). No endpoint is added.
- **Not persisted.** A CUE is never saved with the session (as today), so `paused` needs no serde default work: `CueState` already derives `Serialize`/`Deserialize` and is only built by the reducer.

## File Structure

- Create `crates/fp-model/src/entry_notice.rs`: `EntryNotice`, `entry_notice`.
- Modify `crates/fp-model/src/player.rs` (`CueState::paused`), `command.rs` (three commands, two engine actions), `reducer.rs` (rules), `lib.rs` (exports).
- Create `crates/fp-model/tests/cue_window.rs`, `crates/fp-model/tests/entry_notice.rs`.
- Modify `crates/fp-engine/src/engine.rs` (`cue_paused`, `seek_cue`, `set_cue_paused`, `execute` arms); modify `crates/fp-engine/tests/engine.rs`.
- Modify `crates/fp-app/src/ui/widgets.rs` (`WaveOutput`, `pan_dragging`, no seek drag), `ui/player.rs` (pan, banner), `ui/view.rs` (`entry_notice` field, `CueWindowView`, `cue_window_view`, `cue_follow_target`), `ui/table.rs` (O17 click), `ui/app.rs` (draw the windows), `ui.rs` (module).
- Create `crates/fp-app/src/ui/cue_window.rs`.
- Modify `crates/fp-app/locales/en-US/main.ftl`, `es-ES/main.ftl`.
- Create `crates/fp-app/tests/cue_window.rs`, `crates/fp-app/tests/entry_notice_ui.rs`; modify `crates/fp-app/tests/waveform_ui.rs`, `waveform_view.rs`, `view.rs`, `glyphs.rs`, `main_screen.rs` (O17 click tests).
- Modify docs: `docs/user/players.md`, `docs/user/playlists.md` (menu text: the CUE follows), `docs/technical/audio-engine.md`, `docs/technical/ui.md`, the spec (status note and "As built" under §7), `docs/superpowers/plans/2026-10-01-operator-feedback-2-roadmap.md` (row 6 done), `README.md` (feature line).

---

### Task 1: CUE pause, seek, load-as-next and follow rules (model)

**Files:**
- Modify: `crates/fp-model/src/player.rs` (`CueState`, near line 26)
- Modify: `crates/fp-model/src/command.rs` (`Command`, near line 44; `EngineAction`, near line 276)
- Modify: `crates/fp-model/src/reducer.rs` (`apply` arms near lines 42 and 93; `cue_entry` near line 664; new helpers after it)
- Test: Create `crates/fp-model/tests/cue_window.rs`

**Interfaces:**
- Consumes: `AppState::request_at(entry, secs) -> Option<SourceRequest>`, `AppState::playable_request`, `set_next` (private in `reducer.rs`), `cue_entry` (private in `reducer.rs`).
- Produces: `CueState { pub entry: EntryId, pub paused: bool }`; `Command::SeekCue(PlayerId, f64)`, `Command::SetCuePaused(PlayerId, bool)`, `Command::CueToNext(PlayerId)`; `EngineAction::SeekCue { player: PlayerId, secs: f64 }`, `EngineAction::SetCuePaused { player: PlayerId, paused: bool }`.

- [ ] **Step 1: Write the failing tests**

Create `crates/fp-model/tests/cue_window.rs`:

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback 2 spec O12 (CUE window commands) and O17 (the CUE follows).

mod common;

use common::{entries, fixture, p0};
use fp_model::{AppState, Command, EngineAction, FileState, ModelError, PlayerId, apply};

/// A state whose player 0 is cueing its next entry (the first of three).
fn cueing() -> (AppState, PlayerId, Vec<fp_model::EntryId>) {
    let mut s = fixture(3);
    let p = p0(&s);
    let e = entries(&s);
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    (s, p, e)
}

fn cue(s: &AppState) -> Option<fp_model::CueState> {
    s.players[0].cue
}

#[test]
fn a_new_cue_starts_unpaused() {
    let (s, _, e) = cueing();
    let c = cue(&s).unwrap();
    assert_eq!(c.entry, e[0]);
    assert!(!c.paused);
}

#[test]
fn pausing_and_resuming_the_cue_tell_the_engine_once() {
    let (mut s, p, _) = cueing();
    let a = apply(&mut s, Command::SetCuePaused(p, true)).unwrap();
    assert_eq!(
        a,
        vec![EngineAction::SetCuePaused {
            player: p,
            paused: true
        }]
    );
    assert!(cue(&s).unwrap().paused);
    assert!(
        apply(&mut s, Command::SetCuePaused(p, true))
            .unwrap()
            .is_empty(),
        "idempotent"
    );
    let a = apply(&mut s, Command::SetCuePaused(p, false)).unwrap();
    assert_eq!(
        a,
        vec![EngineAction::SetCuePaused {
            player: p,
            paused: false
        }]
    );
    assert!(!cue(&s).unwrap().paused);
}

#[test]
fn commands_without_a_cue_do_nothing() {
    let mut s = fixture(3);
    let p = p0(&s);
    for command in [
        Command::SetCuePaused(p, true),
        Command::SeekCue(p, 30.0),
        Command::CueToNext(p),
    ] {
        assert_eq!(apply(&mut s, command).unwrap(), vec![]);
    }
    assert!(cue(&s).is_none());
    assert!(!s.players[0].next_explicit);
}

#[test]
fn commands_for_an_unknown_player_are_refused() {
    let mut s = fixture(1);
    let ghost = PlayerId(99);
    assert!(matches!(
        apply(&mut s, Command::SeekCue(ghost, 1.0)),
        Err(ModelError::UnknownPlayer(_))
    ));
    assert!(matches!(
        apply(&mut s, Command::SetCuePaused(ghost, true)),
        Err(ModelError::UnknownPlayer(_))
    ));
    assert!(matches!(
        apply(&mut s, Command::CueToNext(ghost)),
        Err(ModelError::UnknownPlayer(_))
    ));
}

#[test]
fn seeking_the_cue_asks_the_engine_for_the_position_clamped_to_the_file() {
    let (mut s, p, _) = cueing();
    for (asked, want) in [(42.5, 42.5), (500.0, 180.0), (-3.0, 0.0)] {
        let a = apply(&mut s, Command::SeekCue(p, asked)).unwrap();
        assert_eq!(
            a,
            vec![EngineAction::SeekCue {
                player: p,
                secs: want
            }],
            "asked {asked}"
        );
    }
}

#[test]
fn a_broken_seek_is_ignored() {
    let (mut s, p, _) = cueing();
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(apply(&mut s, Command::SeekCue(p, bad)).unwrap(), vec![]);
    }
}

#[test]
fn seeking_a_paused_cue_keeps_it_paused() {
    let (mut s, p, _) = cueing();
    apply(&mut s, Command::SetCuePaused(p, true)).unwrap();
    apply(&mut s, Command::SeekCue(p, 10.0)).unwrap();
    assert!(cue(&s).unwrap().paused, "the model does not resume it");
}

#[test]
fn load_as_next_makes_the_cued_entry_the_explicit_next_and_keeps_the_cue() {
    let (mut s, p, e) = cueing();
    // Cue the third entry, whose row is not the next.
    apply(&mut s, Command::CueEntry(p, e[2])).unwrap();
    assert_ne!(s.players[0].next, Some(e[2]));
    let a = apply(&mut s, Command::CueToNext(p)).unwrap();
    assert_eq!(s.players[0].next, Some(e[2]));
    assert!(s.players[0].next_explicit);
    assert_eq!(cue(&s).unwrap().entry, e[2], "the CUE keeps running");
    assert!(
        !a.iter().any(|a| matches!(
            a,
            EngineAction::StopCue { .. } | EngineAction::StartCue { .. }
        )),
        "{a:?}"
    );
}

#[test]
fn load_as_next_of_the_current_entry_is_refused() {
    let (mut s, p, e) = cueing();
    apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(s.players[0].current, Some(e[0]));
    apply(&mut s, Command::CueEntry(p, e[0])).unwrap();
    assert_eq!(
        apply(&mut s, Command::CueToNext(p)),
        Err(ModelError::NextIsCurrent)
    );
}

#[test]
fn setting_next_moves_a_running_cue_to_the_new_next_from_its_cue_in() {
    let (mut s, p, e) = cueing();
    let a = apply(&mut s, Command::SetNext(p, e[2])).unwrap();
    assert_eq!(cue(&s).unwrap().entry, e[2]);
    let start = a.iter().find_map(|a| match a {
        EngineAction::StartCue { player, request } if *player == p => Some(request),
        _ => None,
    });
    let request = start.expect("a StartCue");
    assert_eq!(request.entry, e[2]);
    assert_eq!(request.from_secs, 0.0, "the entry's cue-in (none set)");
}

#[test]
fn moving_a_paused_cue_plays_the_new_one() {
    let (mut s, p, e) = cueing();
    apply(&mut s, Command::SetCuePaused(p, true)).unwrap();
    apply(&mut s, Command::SetNext(p, e[1])).unwrap();
    let c = cue(&s).unwrap();
    assert_eq!(c.entry, e[1]);
    assert!(!c.paused);
}

#[test]
fn setting_the_cued_entry_as_next_keeps_it_running() {
    let (mut s, p, e) = cueing();
    let a = apply(&mut s, Command::SetNext(p, e[0])).unwrap();
    assert_eq!(cue(&s).unwrap().entry, e[0]);
    assert!(
        !a.iter().any(|a| matches!(a, EngineAction::StartCue { .. })),
        "no restart: {a:?}"
    );
}

#[test]
fn setting_next_without_a_cue_starts_none() {
    let mut s = fixture(3);
    let p = p0(&s);
    let e = entries(&s);
    let a = apply(&mut s, Command::SetNext(p, e[2])).unwrap();
    assert!(cue(&s).is_none());
    assert!(
        !a.iter().any(|a| matches!(a, EngineAction::StartCue { .. })),
        "{a:?}"
    );
}

#[test]
fn setting_next_to_an_unplayable_entry_leaves_the_cue() {
    let (mut s, p, e) = cueing();
    let track = s.playlists.entry(e[2]).unwrap().track;
    apply(
        &mut s,
        Command::SetFileState {
            track,
            state: FileState::Missing,
        },
    )
    .unwrap();
    let a = apply(&mut s, Command::SetNext(p, e[2])).unwrap();
    assert_eq!(s.players[0].next, Some(e[2]));
    assert_eq!(cue(&s).unwrap().entry, e[0], "the CUE stays where it was");
    assert!(
        !a.iter().any(|a| matches!(a, EngineAction::StartCue { .. })),
        "{a:?}"
    );
}
```

- [ ] **Step 2: Run them to verify they fail**

Run: `cargo test -p fp-model --test cue_window a_new_cue_starts_unpaused`
Expected: FAIL to compile (`no field paused`, `no variant SetCuePaused`, `SeekCue`, `CueToNext`).

- [ ] **Step 3: Implement**

`crates/fp-model/src/player.rs`:

```rust
/// Pre-listen (cue) of one entry on the player's Cue bus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CueState {
    pub entry: EntryId,
    /// The CUE window's Pause is on: the source is held on the Cue bus.
    pub paused: bool,
}
```

`crates/fp-model/src/command.rs`, in `Command` after `CueEntry(PlayerId, EntryId),`:

```rust
    /// Feedback 2 spec O12: moves the running CUE to `secs` of its entry
    /// (clamped to the file). A broken value, or no CUE, does nothing.
    SeekCue(PlayerId, f64),
    /// Spec O12: holds or releases the running CUE. Idempotent; without a
    /// CUE it does nothing.
    SetCuePaused(PlayerId, bool),
    /// Spec O12 ("Load as next"): the cued entry becomes the player's
    /// explicit next and the CUE keeps running. Without a CUE it does
    /// nothing.
    CueToNext(PlayerId),
```

In `EngineAction`, after `StopCue { player: PlayerId },`:

```rust
    /// Replace the CUE source by one at `secs`; it stays held if the CUE is.
    SeekCue {
        player: PlayerId,
        secs: f64,
    },
    /// Hold or release the CUE source.
    SetCuePaused {
        player: PlayerId,
        paused: bool,
    },
```

`crates/fp-model/src/reducer.rs`. Replace the `SetNext` arm and add three arms after `CueEntry`:

```rust
        Command::SetNext(id, entry) => {
            set_next(state, id, entry)?;
            follow_cue(state, id, entry, &mut out)?;
        }
```

```rust
        Command::SeekCue(id, secs) => seek_cue(state, id, secs, &mut out)?,
        Command::SetCuePaused(id, paused) => set_cue_paused(state, id, paused, &mut out)?,
        Command::CueToNext(id) => {
            let i = state.player_index(id)?;
            if let Some(cue) = state.players[i].cue {
                set_next(state, id, cue.entry)?;
            }
        }
```

In `cue_entry`, change the assignment to `state.players[i].cue = Some(CueState { entry, paused: false });` and add after `cue_entry`:

```rust
/// Spec O17: a running CUE follows the player's new next, from its cue-in.
/// The entry already cued is left running (no restart), and an entry that
/// cannot be played leaves the CUE where it is.
fn follow_cue(
    state: &mut AppState,
    id: PlayerId,
    entry: EntryId,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    let i = state.player_index(id)?;
    let moves = state.players[i].cue.is_some_and(|c| c.entry != entry);
    if moves && state.playable_request(entry).is_some() {
        cue_entry(state, id, entry, out)?;
    }
    Ok(())
}

/// Spec O12: seeks the running CUE inside its file. The model does not
/// touch `paused`: a held CUE stays held at the new position.
fn seek_cue(
    state: &mut AppState,
    id: PlayerId,
    secs: f64,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    let i = state.player_index(id)?;
    let Some(cue) = state.players[i].cue else {
        return Ok(());
    };
    if !secs.is_finite() {
        return Ok(());
    }
    if let Some(request) = state.request_at(cue.entry, secs) {
        out.push(EngineAction::SeekCue {
            player: id,
            secs: request.from_secs,
        });
    }
    Ok(())
}

/// Spec O12: holds or releases the running CUE; only a change is sent.
fn set_cue_paused(
    state: &mut AppState,
    id: PlayerId,
    paused: bool,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    let i = state.player_index(id)?;
    if let Some(cue) = state.players[i].cue.as_mut()
        && cue.paused != paused
    {
        cue.paused = paused;
        out.push(EngineAction::SetCuePaused { player: id, paused });
    }
    Ok(())
}
```

(`EntryId` is already imported in `reducer.rs`; if not, add it to the `use crate::ids::{...}` list.)

- [ ] **Step 4: Run the tests to verify they pass**

Run each, one filter per command:
- `cargo test -p fp-model --test cue_window a_new_cue_starts_unpaused`
- `cargo test -p fp-model --test cue_window pausing_and_resuming`
- `cargo test -p fp-model --test cue_window commands_without_a_cue_do_nothing`
- `cargo test -p fp-model --test cue_window commands_for_an_unknown_player`
- `cargo test -p fp-model --test cue_window seeking_the_cue`
- `cargo test -p fp-model --test cue_window a_broken_seek_is_ignored`
- `cargo test -p fp-model --test cue_window seeking_a_paused_cue`
- `cargo test -p fp-model --test cue_window load_as_next`
- `cargo test -p fp-model --test cue_window setting_next`
- `cargo test -p fp-model --test cue_window moving_a_paused_cue`
- `cargo test -p fp-model --test cue_window setting_the_cued_entry`

Expected: PASS. If `cueing()` panics because the fixture's next is `None`, check `refresh_next` ran after `InsertPaths` (it does in `insert_paths`) and fix the helper, not the rule. Then `cargo test -p fp-model` passes (the other crates still compile only after the engine arm of Task 3, so run `cargo test -p fp-model` here, not `--workspace`).

- [ ] **Step 5: Commit**

`Engine::execute` matches `EngineAction` exhaustively, so the workspace builds again only once Task 3's two arms exist. Do Task 3 right after this task's Step 4 and commit both together (message in Task 3); the per-task review still looks at each task's diff.

---

### Task 2: The per-entry banner rule and the header badge (O8)

**Files:**
- Create: `crates/fp-model/src/entry_notice.rs`
- Modify: `crates/fp-model/src/lib.rs` (`mod entry_notice;` and `pub use entry_notice::{EntryNotice, entry_notice};`)
- Modify: `crates/fp-app/src/ui/view.rs` (`PlayerView::entry_notice`, `player_view`)
- Modify: `crates/fp-app/src/ui/player.rs` (`outlined`, `header`)
- Modify: `crates/fp-app/locales/en-US/main.ftl`, `crates/fp-app/locales/es-ES/main.ftl`
- Test: Create `crates/fp-model/tests/entry_notice.rs`, `crates/fp-app/tests/entry_notice_ui.rs`; modify `crates/fp-app/tests/view.rs`

**Interfaces:**
- Consumes: `PlayerState::{transport, stop_after_current, current, fade_stopping()}`, `PlaylistEntry::{repeat, stop_after}`, `Track::file_state.is_playable()`, `AppState::track_for_entry`.
- Produces: `fp_model::EntryNotice { Repeats, StopsAfter }`, `fp_model::entry_notice(state: &AppState, player: PlayerId) -> Option<EntryNotice>`; `PlayerView::entry_notice: Option<fp_model::EntryNotice>`; message keys `badge-entry-repeat`, `badge-entry-stop`, `tip-entry-repeat`, `tip-entry-stop`.

- [ ] **Step 1: Write the failing tests**

Create `crates/fp-model/tests/entry_notice.rs`:

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback 2 spec O8: which notice the player header shows for the
//! current entry's own repeat and stop-after flags.

mod common;

use common::{fixture, p0};
use fp_model::{AppState, Command, EntryNotice, FileState, entry_notice};

/// Player 0 is playing its first entry.
fn playing() -> (AppState, fp_model::PlayerId, fp_model::EntryId) {
    let mut s = fixture(3);
    let p = p0(&s);
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    let current = s.players[0].current.unwrap();
    (s, p, current)
}

fn flag(s: &mut AppState, command: Command) {
    fp_model::apply(s, command).unwrap();
}

#[test]
fn an_entry_without_flags_shows_no_notice() {
    let (s, p, _) = playing();
    assert_eq!(entry_notice(&s, p), None);
}

#[test]
fn a_repeating_current_entry_says_it_repeats() {
    let (mut s, p, e) = playing();
    flag(&mut s, Command::ToggleEntryRepeat(e));
    assert_eq!(entry_notice(&s, p), Some(EntryNotice::Repeats));
}

#[test]
fn a_stop_after_current_entry_says_it_stops_after() {
    let (mut s, p, e) = playing();
    flag(&mut s, Command::ToggleEntryStopAfter(e));
    assert_eq!(entry_notice(&s, p), Some(EntryNotice::StopsAfter));
}

#[test]
fn stop_after_wins_over_repeat_on_the_entry() {
    let (mut s, p, e) = playing();
    flag(&mut s, Command::ToggleEntryRepeat(e));
    flag(&mut s, Command::ToggleEntryStopAfter(e));
    assert_eq!(entry_notice(&s, p), Some(EntryNotice::StopsAfter));
}

#[test]
fn stop_after_current_takes_precedence_over_both() {
    let (mut s, p, e) = playing();
    flag(&mut s, Command::ToggleEntryRepeat(e));
    flag(&mut s, Command::ToggleStopAfterCurrent(p));
    assert_eq!(entry_notice(&s, p), None, "its own badge says it");
    flag(&mut s, Command::ToggleEntryStopAfter(e));
    assert_eq!(entry_notice(&s, p), None);
}

#[test]
fn a_stopped_player_shows_none_even_if_its_entries_are_flagged() {
    let mut s = fixture(3);
    let p = p0(&s);
    let first = s.players[0].next.unwrap();
    flag(&mut s, Command::ToggleEntryRepeat(first));
    assert_eq!(entry_notice(&s, p), None);
}

#[test]
fn a_flag_on_the_next_entry_is_not_shown_for_the_current_one() {
    let (mut s, p, _) = playing();
    let next = s.players[0].next.unwrap();
    flag(&mut s, Command::ToggleEntryRepeat(next));
    assert_eq!(entry_notice(&s, p), None);
}

#[test]
fn a_running_fade_stop_hides_the_repeat_notice() {
    let (mut s, p, e) = playing();
    flag(&mut s, Command::ToggleEntryRepeat(e));
    flag(&mut s, Command::FadeStop(p));
    assert_eq!(entry_notice(&s, p), None);
}

#[test]
fn an_unreadable_file_does_not_repeat() {
    let (mut s, p, e) = playing();
    flag(&mut s, Command::ToggleEntryRepeat(e));
    let track = s.playlists.entry(e).unwrap().track;
    flag(
        &mut s,
        Command::SetFileState {
            track,
            state: FileState::Unreadable,
        },
    );
    assert_eq!(entry_notice(&s, p), None);
}
```

Append to `crates/fp-app/tests/view.rs` (extend the `use fp_app::ui::view::{...}` list with nothing new; `player_view` is already imported):

```rust
#[test]
fn the_player_view_carries_the_entry_notice() {
    let (mut s, e, p) = state(2);
    apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(player_view(&s, p, None, 0.0).unwrap().entry_notice, None);
    apply(&mut s, Command::ToggleEntryRepeat(e[0])).unwrap();
    assert_eq!(
        player_view(&s, p, None, 0.0).unwrap().entry_notice,
        Some(fp_model::EntryNotice::Repeats)
    );
}
```

Create `crates/fp-app/tests/entry_notice_ui.rs`:

```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Feedback 2 spec O8: the player header names the entry's own repeat and
//! stop-after.

mod support;

use fp_model::{Command, apply};
use support::{harness, state};

const REPEAT: &str = "This track will repeat";
const STOPS: &str = "Stops after this track";

fn playing_with(flag: impl Fn(fp_model::EntryId) -> Option<Command>) -> fp_model::AppState {
    let mut s = state(1, 3);
    let p = s.players[0].id;
    apply(&mut s, Command::Play(p)).unwrap();
    let e = s.players[0].current.unwrap();
    if let Some(c) = flag(e) {
        apply(&mut s, c).unwrap();
    }
    s
}

#[test]
fn a_repeating_entry_shows_the_repeat_notice() {
    let (h, _) = harness(playing_with(|e| Some(Command::ToggleEntryRepeat(e))));
    assert!(h.query_by_label(REPEAT).is_some());
    assert!(h.query_by_label(STOPS).is_none());
}

#[test]
fn a_stop_after_entry_shows_the_stop_notice() {
    let (h, _) = harness(playing_with(|e| Some(Command::ToggleEntryStopAfter(e))));
    assert!(h.query_by_label(STOPS).is_some());
    assert!(h.query_by_label(REPEAT).is_none());
}

#[test]
fn no_flag_no_notice_and_stop_after_current_hides_it() {
    let (h, _) = harness(playing_with(|_| None));
    assert!(h.query_by_label(REPEAT).is_none() && h.query_by_label(STOPS).is_none());

    let mut s = playing_with(|e| Some(Command::ToggleEntryRepeat(e)));
    let p = s.players[0].id;
    apply(&mut s, Command::ToggleStopAfterCurrent(p)).unwrap();
    let (h, _) = harness(s);
    assert!(h.query_by_label(REPEAT).is_none());
}
```

- [ ] **Step 2: Run them to verify they fail**

Run: `cargo test -p fp-model --test entry_notice an_entry_without_flags_shows_no_notice`
Expected: FAIL to compile (`unresolved import fp_model::EntryNotice`). Likewise `cargo test -p fp-app --test entry_notice_ui a_repeating_entry_shows_the_repeat_notice` fails to compile.

- [ ] **Step 3: Implement**

Create `crates/fp-model/src/entry_notice.rs`:

```rust
//! The notice a player's header shows for its current entry's own flags
//! (feedback 2 spec O8). A pure function of the state: the UI only draws it.

use crate::ids::PlayerId;
use crate::player::Transport;
use crate::state::AppState;

/// What the current entry will do when it ends, beyond what the player's own
/// "Stop after" badge already says.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryNotice {
    /// The entry repeats (R26).
    Repeats,
    /// The entry stops the player when it ends (R27).
    StopsAfter,
}

/// The notice for `player`'s current entry, if any.
///
/// None while the player is stopped or a fade stop runs, and while stop after
/// current is set (its own badge says it). Otherwise the entry's stop-after
/// wins over its repeat, as in the rules; a file that can no longer be opened
/// plays its pass out and does not repeat.
pub fn entry_notice(state: &AppState, player: PlayerId) -> Option<EntryNotice> {
    let p = state.player(player).ok()?;
    if p.transport == Transport::Stopped || p.fade_stopping() || p.stop_after_current {
        return None;
    }
    let entry = state.playlists.entry(p.current?)?;
    if entry.stop_after {
        return Some(EntryNotice::StopsAfter);
    }
    let playable = state
        .track_for_entry(entry.id)
        .is_some_and(|t| t.file_state.is_playable());
    (entry.repeat && playable).then_some(EntryNotice::Repeats)
}
```

`crates/fp-model/src/lib.rs`: add `mod entry_notice;` with the other `mod` lines and `pub use entry_notice::{EntryNotice, entry_notice};` with the other `pub use` lines.

`crates/fp-app/src/ui/view.rs`: add to `PlayerView` (after `stop_after_current`):

```rust
    /// The current entry repeats or stops after itself (spec O8).
    pub entry_notice: Option<fp_model::EntryNotice>,
```

and in `player_view`'s initial `PlayerView { ... }` literal, after `stop_after_current: p.stop_after_current,`:

```rust
        entry_notice: fp_model::entry_notice(state, player),
```

`crates/fp-app/src/ui/player.rs`: replace `outlined` with a version that can carry a tooltip, keeping the old call sites working:

```rust
fn outlined(ui: &mut Ui, text: &str, border: Color32, color: Color32) {
    outlined_with_tip(ui, text, None, border, color);
}

/// As `outlined`, with a sentence that is the badge's tooltip and
/// accessible name.
fn outlined_with_tip(ui: &mut Ui, text: &str, tip: Option<&str>, border: Color32, color: Color32) {
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_uppercase(), font(10.0), color);
    let size = vec2(galley.size().x + 12.0, 18.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter()
        .rect_stroke(rect, 0.0, Stroke::new(1.0, border), StrokeKind::Inside);
    ui.painter().galley(
        pos2(rect.left() + 6.0, rect.center().y - galley.size().y / 2.0),
        galley,
        color,
    );
    if let Some(tip) = tip {
        let owned = tip.to_owned();
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Label, true, owned.clone())
        });
        response.on_hover_text(tip);
    }
}
```

In `header`, right after the `if pv.stop_after_current { ... }` block, add:

```rust
            if let Some(notice) = pv.entry_notice {
                let (text, tip) = match notice {
                    fp_model::EntryNotice::Repeats => ("badge-entry-repeat", "tip-entry-repeat"),
                    fp_model::EntryNotice::StopsAfter => ("badge-entry-stop", "tip-entry-stop"),
                };
                outlined_with_tip(
                    ui,
                    &t.tr(text),
                    Some(&t.tr(tip)),
                    theme::AMBER,
                    theme::AMBER_TEXT,
                );
            }
```

`crates/fp-app/locales/en-US/main.ftl`, after `badge-stop-after = Stop after`:

```
badge-entry-repeat = Repeat
badge-entry-stop = Stop after track
tip-entry-repeat = This track will repeat
tip-entry-stop = Stops after this track
```

`crates/fp-app/locales/es-ES/main.ftl`, after `badge-stop-after = Stop al final`:

```
badge-entry-repeat = Repetir
badge-entry-stop = Parar tras la pista
tip-entry-repeat = Esta pista se repetirá
tip-entry-stop = Se para al acabar esta pista
```

- [ ] **Step 4: Run the tests to verify they pass**

Run each:
- `cargo test -p fp-model --test entry_notice an_entry_without_flags_shows_no_notice`
- `cargo test -p fp-model --test entry_notice a_repeating_current_entry`
- `cargo test -p fp-model --test entry_notice a_stop_after_current_entry`
- `cargo test -p fp-model --test entry_notice stop_after_wins_over_repeat`
- `cargo test -p fp-model --test entry_notice stop_after_current_takes_precedence`
- `cargo test -p fp-model --test entry_notice a_stopped_player_shows_none`
- `cargo test -p fp-model --test entry_notice a_flag_on_the_next_entry`
- `cargo test -p fp-model --test entry_notice a_running_fade_stop`
- `cargo test -p fp-model --test entry_notice an_unreadable_file_does_not_repeat`
- `cargo test -p fp-app --test view the_player_view_carries_the_entry_notice`
- `cargo test -p fp-app --test entry_notice_ui a_repeating_entry_shows_the_repeat_notice`
- `cargo test -p fp-app --test entry_notice_ui a_stop_after_entry_shows_the_stop_notice`
- `cargo test -p fp-app --test entry_notice_ui no_flag_no_notice`
- `cargo test -p fp-app --test i18n both_locales_define_the_same_keys`

Expected: PASS. (Task 1 and Task 3 are already committed together by now, so the workspace builds; see Task 3's order note.)

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-model crates/fp-app
git commit -m "feat(ui): show a notice when the current entry repeats or stops after itself"
```

---

### Task 3: The engine's CUE seeks and pauses (engine)

> **Order note.** `EngineAction` is matched exhaustively in `Engine::execute`, so Task 1 does not compile the workspace until this task's arms exist. Execute Task 3 straight after Task 1 and commit Tasks 1 and 3 with one green gate (message below); then Task 2. The execution order is therefore 1, 3, 2, 4, 5, 6, 7. The reviewer gate per task still applies to each task's diff.

**Files:**
- Modify: `crates/fp-engine/src/engine.rs` (`PlayerRuntime` near line 139; `add_player` literal near line 866; `execute` near lines 823-831; `start_cue` near line 1400; new `seek_cue`, `set_cue_paused` after it)
- Test: `crates/fp-engine/tests/engine.rs`

**Interfaces:**
- Consumes: `EngineAction::SeekCue { player, secs }`, `EngineAction::SetCuePaused { player, paused }` (Task 1); `Engine::new_source(player, cue: bool, &SourceRequest) -> Result<Playing, AttachError>`, `stop_quick_and_release`, `ramp_frames(&BusKey) -> u32`, `BusCommand::{Pause, Resume}`, `StartState`.
- Produces: `PlayerRuntime::cue_paused: bool` (engine-internal); `Engine::telemetry(player).cue_position_secs` already reports the position, and a held CUE reports a constant one.

- [ ] **Step 1: Write the failing tests**

Append to `crates/fp-engine/tests/engine.rs` (the helpers `rig`, `request`, `Rig::settle`, `Rig::run`, `tag`, `P`, `BLOCK`, `RATE` exist in that file):

```rust
impl Rig {
    fn cue_position(&self) -> Option<f64> {
        self.engine.telemetry(P).cue_position_secs
    }

    /// Renders `blocks` blocks and returns what the Cue device played
    /// (left channel); the Main device is rendered too, as `run` does.
    fn run_hearing_cue(&mut self, blocks: usize) -> Vec<f32> {
        let mut heard = Vec::new();
        for _ in 0..blocks {
            let out = self.cue.render(BLOCK).unwrap();
            heard.extend(out.chunks(2).map(|f| f[0]));
            self.run(1);
        }
        heard
    }
}

fn start_cue(r: &mut Rig, from_secs: f64) {
    r.act(EngineAction::StartCue {
        player: P,
        request: request(3, from_secs),
    });
    r.settle();
}

#[test]
fn pausing_the_cue_holds_its_position_and_resuming_continues() {
    let mut r = rig(480_000, true);
    start_cue(&mut r, 0.0);
    r.run_hearing_cue(10);
    let before = r.cue_position().unwrap();
    assert!(before > 0.0, "{before}");

    r.act(EngineAction::SetCuePaused {
        player: P,
        paused: true,
    });
    // The pause ramps down; once it has, nothing more is played.
    r.run_hearing_cue(10);
    let held = r.cue_position().unwrap();
    let silent = r.run_hearing_cue(10);
    assert!(silent.iter().all(|v| *v == 0.0), "a held CUE is silent");
    assert_eq!(r.cue_position().unwrap(), held, "the position holds");

    r.act(EngineAction::SetCuePaused {
        player: P,
        paused: false,
    });
    let resumed = r.run_hearing_cue(10);
    assert!(resumed.iter().any(|v| *v != 0.0), "it plays again");
    assert!(r.cue_position().unwrap() > held + 0.05);
    assert!(
        !r.events
            .iter()
            .any(|e| matches!(e, EngineEvent::CueEnded { .. })),
        "pausing never ends the CUE"
    );
}

#[test]
fn seeking_the_cue_replaces_the_source_at_the_new_position() {
    let mut r = rig(480_000, true);
    start_cue(&mut r, 0.0);
    r.run_hearing_cue(2);
    r.act(EngineAction::SeekCue {
        player: P,
        secs: 3.0,
    });
    r.settle();
    let heard = r.run_hearing_cue(6);
    assert!(r.cue_position().unwrap() >= 3.0);
    let last = heard.iter().rev().find(|v| **v != 0.0).unwrap();
    assert!(tag(*last).1 as f64 / RATE >= 3.0, "the Cue device plays from 3 s");
    assert!(
        r.heard.iter().all(|v| *v == 0.0),
        "the main output never hears the CUE"
    );
}

#[test]
fn seeking_a_paused_cue_stays_paused_at_the_new_position() {
    let mut r = rig(480_000, true);
    start_cue(&mut r, 0.0);
    r.run_hearing_cue(4);
    r.act(EngineAction::SetCuePaused {
        player: P,
        paused: true,
    });
    r.run_hearing_cue(10);
    r.act(EngineAction::SeekCue {
        player: P,
        secs: 2.0,
    });
    r.settle();
    let silent = r.run_hearing_cue(10);
    assert!(silent.iter().all(|v| *v == 0.0));
    let at = r.cue_position().unwrap();
    assert!((2.0..2.05).contains(&at), "held at the seek target: {at}");
    r.act(EngineAction::SetCuePaused {
        player: P,
        paused: false,
    });
    let heard = r.run_hearing_cue(10);
    assert!(heard.iter().any(|v| *v != 0.0));
    assert!(r.cue_position().unwrap() > at + 0.05);
}

#[test]
fn a_new_cue_after_a_pause_plays() {
    let mut r = rig(480_000, true);
    start_cue(&mut r, 0.0);
    r.run_hearing_cue(4);
    r.act(EngineAction::SetCuePaused {
        player: P,
        paused: true,
    });
    r.run_hearing_cue(10);
    // The model starts the new CUE unpaused (it moved to another entry).
    start_cue(&mut r, 0.0);
    let heard = r.run_hearing_cue(10);
    assert!(heard.iter().any(|v| *v != 0.0), "the new CUE is audible");
}

#[test]
fn cue_commands_without_a_cue_source_are_ignored() {
    let mut r = rig(48_000, true);
    r.act(EngineAction::SeekCue {
        player: P,
        secs: 1.0,
    });
    r.act(EngineAction::SetCuePaused {
        player: P,
        paused: true,
    });
    r.act(EngineAction::SetCuePaused {
        player: P,
        paused: false,
    });
    r.settle();
    r.run(2);
    assert_eq!(r.cue_position(), None);
    assert!(r.events.is_empty(), "{:?}", r.events);
}

#[test]
fn a_seek_past_the_end_ends_the_cue() {
    let mut r = rig(48_000, true);
    start_cue(&mut r, 0.0);
    r.act(EngineAction::SeekCue {
        player: P,
        secs: 5.0,
    });
    r.settle();
    r.run_hearing_cue(10);
    assert!(r.events.contains(&EngineEvent::CueEnded {
        player: P,
        entry: EntryId(3)
    }));
}
```

- [ ] **Step 2: Run them to verify they fail**

Run: `cargo test -p fp-engine --test engine pausing_the_cue_holds_its_position_and_resuming_continues`
Expected: FAIL to compile if Task 1 is in (the engine's `execute` has no arm for the new actions: "non-exhaustive patterns"), or the test fails at the first assertion once the arms are stubs. Either is the expected red.

- [ ] **Step 3: Implement**

In `crates/fp-engine/src/engine.rs`:

`PlayerRuntime`: add after `cue_src: Option<Playing>,`:

```rust
    /// The CUE source is held (the CUE window's Pause). Cleared by a new
    /// or stopped CUE.
    cue_paused: bool,
```

`add_player`'s `PlayerRuntime { ... }` literal: add `cue_paused: false,` after `cue_src: None,`.

`execute`: add the two arms next to `StartCue`/`StopCue`, and clear the flag when the CUE stops:

```rust
            EngineAction::StartCue { player, request } => self.start_cue(player, &request),
            EngineAction::StopCue { player } => {
                if let Some(rt) = self.players.get_mut(&player) {
                    rt.cue_paused = false;
                }
                if let Some(cue) = self
                    .players
                    .get_mut(&player)
                    .and_then(|rt| rt.cue_src.take())
                {
                    self.stop_quick_and_release(player, cue);
                }
            }
            EngineAction::SeekCue { player, secs } => self.seek_cue(player, secs),
            EngineAction::SetCuePaused { player, paused } => self.set_cue_paused(player, paused),
```

In `start_cue`, a new CUE is never held: at its very start (before the old source is taken) add

```rust
        if let Some(rt) = self.players.get_mut(&player) {
            rt.cue_paused = false;
        }
```

Add after `start_cue`:

```rust
    /// Replaces the CUE source by one at `secs` (spec O12). A held CUE stays
    /// held: the new source waits idle until it is released. Without a CUE
    /// source nothing happens.
    fn seek_cue(&mut self, player: PlayerId, secs: f64) {
        let Some(mut request) = self
            .players
            .get(&player)
            .and_then(|rt| rt.cue_src.as_ref())
            .map(|c| c.request.clone())
        else {
            return;
        };
        request.from_secs = secs;
        let Ok(mut next) = self.new_source(player, true, &request) else {
            return;
        };
        let Some(rt) = self.players.get_mut(&player) else {
            return;
        };
        next.start = if rt.cue_paused {
            StartState::Idle
        } else {
            StartState::WhenReady { fade_in: false }
        };
        if let Some(old) = rt.cue_src.replace(next) {
            self.stop_quick_and_release(player, old);
        }
    }

    /// Holds or releases the CUE source (spec O12). A source that has not
    /// started yet is held back idle; one that is audible ramps down with
    /// the pause ramp. Without a CUE source only the flag changes.
    fn set_cue_paused(&mut self, player: PlayerId, paused: bool) {
        let Some(rt) = self.players.get_mut(&player) else {
            return;
        };
        rt.cue_paused = paused;
        let Some(cue) = rt.cue_src.as_mut() else {
            return;
        };
        let (bus, slot) = (cue.bus.clone(), cue.slot);
        let send = match (paused, cue.start) {
            (true, StartState::WhenReady { .. }) => {
                cue.start = StartState::Idle;
                None
            }
            (false, StartState::Idle) => {
                cue.start = StartState::WhenReady { fade_in: false };
                None
            }
            (true, _) => Some(true),
            (false, _) => Some(false),
        };
        let Some(pause) = send else {
            return;
        };
        let ramp_frames = self.ramp_frames(&bus);
        let command = if pause {
            BusCommand::Pause { slot, ramp_frames }
        } else {
            BusCommand::Resume { slot, ramp_frames }
        };
        self.send(&bus, command);
    }
```

(`StartState` is `Copy`, so `match (paused, cue.start)` does not move. If the borrow checker rejects `cue` living across `self.ramp_frames`, it is because `rt`/`cue` are still borrowed: the code above ends their use before the call, which is why `(bus, slot)` and `send` are computed first.)

- [ ] **Step 4: Run the tests to verify they pass**

Run each:
- `cargo test -p fp-engine --test engine pausing_the_cue_holds_its_position_and_resuming_continues`
- `cargo test -p fp-engine --test engine seeking_the_cue_replaces_the_source_at_the_new_position`
- `cargo test -p fp-engine --test engine seeking_a_paused_cue_stays_paused_at_the_new_position`
- `cargo test -p fp-engine --test engine a_new_cue_after_a_pause_plays`
- `cargo test -p fp-engine --test engine cue_commands_without_a_cue_source_are_ignored`
- `cargo test -p fp-engine --test engine a_seek_past_the_end_ends_the_cue`
- `cargo test -p fp-engine --test engine cue_plays_only_on_the_cue_output_and_ends_by_itself`

Expected: PASS. If the held-position assertion flakes because the pause ramp is longer than 10 blocks, raise the run count of the first `r.run_hearing_cue(10)` after the pause (the tuning `pause_ramp_ms` default is the bound; do not loosen the equality).

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-model crates/fp-engine
git commit -m "feat(engine): seek and pause the player CUE"
```

(Tasks 1 and 3 go in this one commit if they were not green separately; otherwise two commits: `feat(model): ...` after Task 1 with the engine arms added, then `feat(engine): ...`.)

---

### Task 4: Click seeks, drag pans (O10)

**Files:**
- Modify: `crates/fp-app/src/ui/widgets.rs` (`WaveInput`, `SeekDrag`, `seek_dragging`, `waveform`, doc comments, near lines 1063-1345)
- Modify: `crates/fp-app/src/ui/player.rs` (`wave`, near lines 777-980)
- Test: `crates/fp-app/tests/waveform_ui.rs`, `crates/fp-app/tests/waveform_view.rs`

**Interfaces:**
- Consumes: `WaveView::pan(dx, rect, total) -> WaveView`, `WaveZoom { view, entry, moved_at }`.
- Produces: `widgets::WaveOutput { pub response: Response, pub seek: Option<f64>, pub pan_dx: f32 }`; `widgets::waveform(ui, height, &WaveInput) -> WaveOutput`; `widgets::pan_dragging(ui: &Ui, id: egui::Id) -> bool`; `WaveInput` loses its `entry` field. Task 5 calls `waveform` with the new return type.

- [ ] **Step 1: Write the failing tests**

In `crates/fp-app/tests/waveform_ui.rs`, update the header doc to "Waveform click to seek, drag to pan, zoom and trimmed regions (feedback 2 spec O10).", and make these changes.

Delete these tests, whose mechanism is gone: `releasing_a_drag_inside_seeks_to_the_release_point`, `releasing_a_drag_outside_the_waveform_cancels_it`, `escape_during_a_drag_cancels_it`, `a_drag_started_on_one_track_never_seeks_into_the_next`, `a_drag_left_over_while_a_track_had_no_length_never_fires_later`, and the helper `switch_track` that only they use. Remove the `Key` import if nothing else uses it (clippy reports unused imports).

Replace `dragging_from_the_full_view_button_does_not_seek` and `the_view_does_not_follow_while_a_drag_is_held` with the versions below, and add the new tests (they use the file's existing `playing`, `wave`, `x_of`, `press`, `drag_to`, `seeks`, `wheel`, `click`, `at_position` helpers):

```rust
/// Zooms three notches around 90 s: the view is then 43.92 s to 136.08 s.
fn zoom_in(h: &mut Harness<'_, AppUi>, w: Rect) {
    let centre = pos2(x_of(w, 90.0), w.center().y);
    for _ in 0..3 {
        wheel(h, centre, 0.0, 1.0, Modifiers::NONE);
    }
}

#[test]
fn dragging_pans_the_zoomed_view_and_never_seeks() {
    let (mut h, fake) = harness(playing());
    let w = wave(&h);
    zoom_in(&mut h, w);
    fake.take_sent();
    // 30 s of the full view to the left: the zoomed view moves later.
    let (from, to) = (
        pos2(x_of(w, 100.0), w.center().y),
        pos2(x_of(w, 70.0), w.center().y),
    );
    drag_to(&mut h, from, to);
    press(&mut h, to, false);
    h.run_steps(2);
    assert!(seeks(&fake).is_empty(), "a drag never seeks");
    // The centre showed 90 s; the view has moved about 15 s later (egui
    // drops the first few pixels of a drag), so it now shows about 105 s.
    click(&mut h, pos2(w.center().x, w.center().y));
    let s = seeks(&fake);
    assert_eq!(s.len(), 1, "{s:?}");
    assert!(s[0] > 98.0 && s[0] < 112.0, "{s:?}");
}

#[test]
fn a_drag_without_zoom_does_nothing() {
    let (mut h, fake) = harness(playing());
    let w = wave(&h);
    drag_to(
        &mut h,
        pos2(x_of(w, 40.0), w.center().y),
        pos2(x_of(w, 120.0), w.center().y),
    );
    press(&mut h, pos2(x_of(w, 120.0), w.center().y), false);
    h.run_steps(2);
    assert!(seeks(&fake).is_empty());
    assert!(h.query_by_label("Full view").is_none(), "still the whole track");
    click(&mut h, pos2(x_of(w, 90.0), w.center().y));
    let s = seeks(&fake);
    assert!((s[0] - 90.0).abs() < 1.5, "the view did not move: {s:?}");
}

#[test]
fn a_drag_released_outside_the_waveform_never_seeks() {
    let (mut h, fake) = harness(playing());
    let w = wave(&h);
    let outside = pos2(x_of(w, 120.0), w.bottom() + 60.0);
    drag_to(&mut h, pos2(x_of(w, 40.0), w.center().y), outside);
    press(&mut h, outside, false);
    h.run_steps(2);
    assert!(seeks(&fake).is_empty());
}

#[test]
fn a_click_that_moves_within_the_drag_threshold_still_seeks() {
    let (mut h, fake) = harness(playing());
    let w = wave(&h);
    let at = pos2(x_of(w, 90.0), w.center().y);
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    press(&mut h, at, true);
    let moved = pos2(at.x + 2.0, at.y);
    h.event(Event::PointerMoved(moved));
    h.run_steps(1);
    press(&mut h, moved, false);
    h.run_steps(2);
    let s = seeks(&fake);
    assert_eq!(s.len(), 1, "{s:?}");
    assert!((s[0] - 90.0).abs() < 2.0, "{s:?}");
}

#[test]
fn a_stopped_players_zoomed_waveform_still_pans() {
    let mut s = playing();
    let p = s.players[0].id;
    apply(&mut s, Command::Stop(p)).unwrap();
    let next = s.players[0].next.unwrap();
    let track = s.playlists.entry(next).unwrap().track;
    apply(
        &mut s,
        Command::ApplyAnalysis {
            track,
            analysis: Box::new(TrackAnalysis {
                duration_secs: 180.0,
                ..TrackAnalysis::default()
            }),
        },
    )
    .unwrap();
    let (mut h, fake) = harness(s);
    let w = wave(&h);
    zoom_in(&mut h, w);
    drag_to(
        &mut h,
        pos2(x_of(w, 100.0), w.center().y),
        pos2(x_of(w, 70.0), w.center().y),
    );
    press(&mut h, pos2(x_of(w, 70.0), w.center().y), false);
    h.run_steps(2);
    assert!(seeks(&fake).is_empty(), "it never seeks");
    // Panned later: Full view is still offered.
    assert!(h.query_by_label("Full view").is_some());
}

#[test]
fn dragging_from_the_full_view_button_does_not_pan() {
    let (mut h, fake) = harness(playing());
    let w = wave(&h);
    wheel(
        &mut h,
        pos2(x_of(w, 90.0), w.center().y),
        0.0,
        1.0,
        Modifiers::NONE,
    );
    let button = h.get_by_label("Full view").rect();
    fake.take_sent();
    let to = pos2(button.center().x - 40.0, button.center().y);
    drag_to(&mut h, button.center(), to);
    press(&mut h, to, false);
    h.run_steps(2);
    assert!(seeks(&fake).is_empty());
    // One notch around 90 s shows 18 s to 162 s: the centre is still 90 s.
    click(&mut h, pos2(w.center().x, w.center().y));
    let s = seeks(&fake);
    assert!((s[0] - 90.0).abs() < 1.5, "the view did not move: {s:?}");
}

#[test]
fn the_view_does_not_follow_while_a_drag_is_held() {
    let mut s = playing();
    s.config.ui.follow_current_grace_secs = 0.01;
    let (mut h, fake) = harness(s);
    at_position(&fake, 20.0);
    h.run_steps(1);
    let w = wave(&h);
    for _ in 0..6 {
        wheel(
            &mut h,
            pos2(x_of(w, 20.0), w.center().y),
            0.0,
            1.0,
            Modifiers::NONE,
        );
    }
    // The view is 14.8 s to 62 s. A small drag keeps the button held.
    let (from, to) = (
        pos2(x_of(w, 60.0), w.center().y),
        pos2(x_of(w, 66.0), w.center().y),
    );
    drag_to(&mut h, from, to);
    at_position(&fake, 150.0);
    h.run_steps(2);
    press(&mut h, to, false);
    h.run_steps(2);
    fake.take_sent();
    click(&mut h, pos2(w.center().x, w.center().y));
    let s = seeks(&fake);
    assert!(
        s[0] < 60.0,
        "the view stayed near the start while the drag was held: {s:?}"
    );
}
```

In `crates/fp-app/tests/waveform_view.rs`, remove the line `entry: None,` from the two `WaveInput { ... }` literals (they are at the places that build the input, near lines 171 and 344).

- [ ] **Step 2: Run them to verify they fail**

Run: `cargo test -p fp-app --test waveform_ui dragging_pans_the_zoomed_view_and_never_seeks`
Expected: FAIL (the drag still seeks on release, and nothing pans). `cargo test -p fp-app --test waveform_view a_player_that_unloads_lets_go_of_its_waveform` fails to compile only after `entry` is removed from the struct in Step 3; before that it compiles with the literal edit (a missing field error), which is also red.

- [ ] **Step 3: Implement**

`crates/fp-app/src/ui/widgets.rs`:

Remove the `entry` field from `WaveInput` and update the `seekable` doc to "Clicks seek. A stopped player's next track always starts at its cue-in, so its waveform only shows times." Delete `struct SeekDrag` and `seek_dragging`; add:

```rust
/// What the waveform reports for a frame (feedback 2 spec O10).
pub struct WaveOutput {
    pub response: Response,
    /// A click (a press and release within egui's drag threshold): the
    /// time under it. A drag never seeks.
    pub seek: Option<f64>,
    /// How far, in pixels, a primary drag that started on the waveform
    /// moved sideways this frame (positive: to the right). The caller pans
    /// a zoomed view by it; Alt-drag (marker editing) and a drag that
    /// starts under the shield (a button over the waveform) report 0.
    pub pan_dx: f32,
}

/// Whether a pan drag is held on the waveform `id` (the zoomed view then
/// does not follow the playhead).
pub fn pan_dragging(ui: &Ui, id: egui::Id) -> bool {
    ui.data(|d| d.get_temp::<bool>(id.with("pan-drag")).is_some())
}
```

Change `waveform` to return `WaveOutput`. At its start remove the `drag_id` block that drops stale `SeekDrag`s (the `let drag_id`, `pointer_down` and the `if ui.data(|d| d.get_temp::<SeekDrag>...` statement). The early return for a missing total becomes:

```rust
    let pan_id = input.id.with("pan-drag");
    let Some(total) = input.total.filter(|t| *t > 0.0) else {
        ui.data_mut(|d| d.remove_temp::<bool>(pan_id));
        return WaveOutput {
            response,
            seek: None,
            pan_dx: 0.0,
        };
    };
```

Replace everything from the line `// Alt (Option) is for marker editing: it never seeks.` to the end of the function with:

```rust
    // Alt (Option) is for marker editing: it never seeks or pans.
    let alt = ui.input(|i| i.modifiers.alt);
    let shielded = |p: Pos2| input.shield.is_some_and(|r| r.contains(p));
    let origin = ui.input(|i| i.pointer.press_origin());
    let panning = response.dragged_by(egui::PointerButton::Primary)
        && !alt
        && !origin.is_some_and(shielded);
    ui.data_mut(|d| {
        if panning {
            d.insert_temp(pan_id, true);
        } else {
            d.remove_temp::<bool>(pan_id);
        }
    });
    let pan_dx = if panning { response.drag_delta().x } else { 0.0 };
    let mut seek = None;
    // The hover line shows the time under the pointer; a drag pans instead.
    if !response.dragged()
        && let Some(p) = response.hover_pos().filter(|p| !shielded(*p))
    {
        let x = p.x;
        painter.rect_filled(
            Rect::from_min_size(pos2(x, inner.top()), vec2(1.0, inner.height())),
            0.0,
            theme::TEXT.gamma_multiply(0.6),
        );
        let text = format::clock(view.secs_at(x, inner));
        let galley = painter.layout_no_wrap(text, font(10.0), theme::TEXT);
        let tw = galley.size().x + 8.0;
        let lx = (x + 4.0).min(inner.right() - tw);
        let bg = Rect::from_min_size(pos2(lx, inner.top() + 13.0), vec2(tw, 14.0));
        painter.rect_filled(bg, 0.0, theme::NEUTRAL_800);
        painter.galley(pos2(lx + 4.0, bg.top() + 1.0), galley, theme::TEXT);
        if input.seekable && response.clicked() && !alt {
            seek = Some(view.secs_at(p.x, inner));
        }
    }
    WaveOutput {
        response,
        seek,
        pan_dx,
    }
}
```

(`response.clicked()` is only true on release without a drag, which is the "within the drag threshold" rule. The existing doc comment of `waveform` becomes "Draws the waveform; reports a click's seek target and a drag's sideways movement.")

`crates/fp-app/src/ui/player.rs`, `wave`:

- Replace `widgets::seek_dragging(ui, wave_id)` with `widgets::pan_dragging(ui, wave_id)`.
- Remove `entry: current,` from the `WaveInput` literal.
- Replace `let (response, seek) = widgets::waveform(ui, WAVE_HEIGHT, &input);` with:

```rust
    let output = widgets::waveform(ui, WAVE_HEIGHT, &input);
    let (response, seek, pan_dx) = (output.response, output.seek, output.pan_dx);
```

- Immediately before the `match zoom { Some(z) => { view_state.wave_zoom.insert(id, z); } ... }` block, add:

```rust
    // Dragging pans a zoomed view; without zoom a drag does nothing (O10).
    if pan_dx != 0.0
        && let (Some(z), Some(total)) = (zoom.as_mut(), total)
    {
        z.view = z.view.pan(pan_dx, rect.shrink(1.0), total);
        z.moved_at = scene.time;
    }
```

(`rect` is `response.rect`, bound earlier in `wave`.)

- [ ] **Step 4: Run the tests to verify they pass**

Run each:
- `cargo test -p fp-app --test waveform_ui dragging_pans_the_zoomed_view_and_never_seeks`
- `cargo test -p fp-app --test waveform_ui a_drag_without_zoom_does_nothing`
- `cargo test -p fp-app --test waveform_ui a_drag_released_outside`
- `cargo test -p fp-app --test waveform_ui a_click_that_moves_within_the_drag_threshold_still_seeks`
- `cargo test -p fp-app --test waveform_ui a_stopped_players_zoomed_waveform_still_pans`
- `cargo test -p fp-app --test waveform_ui dragging_from_the_full_view_button_does_not_pan`
- `cargo test -p fp-app --test waveform_ui the_view_does_not_follow_while_a_drag_is_held`
- `cargo test -p fp-app --test waveform_ui a_plain_click_still_seeks_at_once`
- `cargo test -p fp-app --test waveform_ui a_stopped_players_waveform_does_not_seek`
- `cargo test -p fp-app --test waveform_view`
- `cargo test -p fp-app --test markers_ui`

Expected: PASS. The numeric windows (98 to 112 s, below 60 s) are wide on purpose: egui drops the first pixels of a drag, so do not tighten them. If another test in `fp-app` asserted a drag seeking, `cargo test -p fp-app` names it: rewrite it to the new rule, never restore the seek.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-app
git commit -m "feat(ui): click seeks on the waveform and a drag pans the zoomed view"
```

---

### Task 5: The CUE window (O12)

**Files:**
- Create: `crates/fp-app/src/ui/cue_window.rs`
- Modify: `crates/fp-app/src/ui.rs` (`mod cue_window;`)
- Modify: `crates/fp-app/src/ui/view.rs` (`CueWindowView`, `cue_window_view`)
- Modify: `crates/fp-app/src/ui/app.rs` (draw the windows in `AppUi::ui`)
- Modify: `crates/fp-app/locales/en-US/main.ftl`, `es-ES/main.ftl`
- Test: Create `crates/fp-app/tests/cue_window.rs`; modify `crates/fp-app/tests/view.rs`, `crates/fp-app/tests/glyphs.rs`

**Interfaces:**
- Consumes: `Command::{SeekCue, SetCuePaused, CueToNext, SetCue}` (Task 1); `widgets::waveform -> WaveOutput`, `WaveInput` without `entry` (Task 4); `glyphs::{paint, glyph_text, TransportAction}`; `Scene { ctl, i18n, media, state, telemetry }`; `PlayerTelemetry::cue_position_secs`; `view::fraction` (private to `view.rs`, used inside `cue_window_view`); `widgets::{tile, TileStyle, tabular_label}`; `theme::{CUE, CUE_BG, SURFACE}`.
- Produces: `view::CueWindowView { player, entry, title, artist, elapsed, total, remaining, paused, position, can_load_next }`; `view::cue_window_view(state: &AppState, player: PlayerId, position: Option<f64>) -> Option<CueWindowView>`; `cue_window::show_all(ctx: &egui::Context, scene: &Scene<'_>)` (crate-private); message keys `cue-window-title`, `cue-window-pause`, `cue-window-resume`, `cue-window-stop`, `cue-window-load-next`, `cue-window-close`, `tip-cue-waveform`.

- [ ] **Step 1: Write the failing tests**

Append to `crates/fp-app/tests/view.rs` (add `cue_window_view` to the `use fp_app::ui::view::{...}` list):

```rust
#[test]
fn the_cue_window_view_needs_a_running_cue() {
    let (s, _, p) = state(2);
    assert_eq!(cue_window_view(&s, p, None), None);
}

#[test]
fn the_cue_window_view_shows_the_cued_entry_its_times_and_its_state() {
    let (mut s, e, p) = state(3);
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    let v = cue_window_view(&s, p, Some(50.0)).unwrap();
    assert_eq!(v.entry, e[0]);
    assert_eq!(v.title, "Song 0");
    assert_eq!(v.artist.as_deref(), Some("Artist"));
    assert_eq!(v.elapsed, 50.0);
    assert_eq!(v.total, Some(200.0));
    assert_eq!(v.remaining, 150.0, "to the end of the file");
    assert_eq!(v.position, Some(0.25));
    assert!(!v.paused);
    apply(&mut s, Command::SetCuePaused(p, true)).unwrap();
    assert!(cue_window_view(&s, p, Some(50.0)).unwrap().paused);
}

#[test]
fn a_cue_window_without_a_position_shows_the_start_of_the_range() {
    let (mut s, e, p) = state(2);
    let t = s.playlists.entry(e[0]).unwrap().track;
    mark(&mut s, t, MarkerKind::CueIn, 12.0);
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    let v = cue_window_view(&s, p, None).unwrap();
    assert_eq!(v.elapsed, 12.0);
    let broken = cue_window_view(&s, p, Some(f64::NAN)).unwrap();
    assert_eq!(broken.elapsed, 12.0);
}

#[test]
fn a_position_past_the_end_is_clamped() {
    let (mut s, _, p) = state(2);
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    let v = cue_window_view(&s, p, Some(900.0)).unwrap();
    assert_eq!((v.elapsed, v.remaining), (200.0, 0.0));
}

#[test]
fn an_unknown_length_gives_zero_times_and_no_position() {
    let (mut s, e, p) = state(2);
    let t = s.playlists.entry(e[0]).unwrap().track;
    s.library.get_mut(t).unwrap().duration_secs = 0.0;
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    let v = cue_window_view(&s, p, Some(5.0)).unwrap();
    assert_eq!((v.total, v.remaining, v.position), (None, 0.0, None));
}

#[test]
fn load_as_next_is_offered_only_when_it_changes_something() {
    let (mut s, e, p) = state(3);
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    // Cued entry is the derived next: loading it makes it explicit.
    assert!(cue_window_view(&s, p, None).unwrap().can_load_next);
    apply(&mut s, Command::CueToNext(p)).unwrap();
    assert!(!cue_window_view(&s, p, None).unwrap().can_load_next, "already explicit");
    apply(&mut s, Command::CueEntry(p, e[2])).unwrap();
    assert!(cue_window_view(&s, p, None).unwrap().can_load_next);
    apply(&mut s, Command::Play(p)).unwrap();
    apply(&mut s, Command::CueEntry(p, e[0])).unwrap();
    assert!(
        !cue_window_view(&s, p, None).unwrap().can_load_next,
        "the current entry cannot be the next"
    );
}
```

Create `crates/fp-app/tests/cue_window.rs`:

```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Feedback 2 spec O12: the CUE window.

mod support;

use std::sync::Arc;

use egui::{Event, Modifiers, PointerButton, pos2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::AppUi;
use fp_app::ui::controller::Controller;
use fp_model::Command;
use support::{Fake, harness, state};

const PAUSE: &str = "Pause CUE";
const RESUME: &str = "Resume CUE";
const STOP: &str = "Stop CUE";
const LOAD_NEXT: &str = "Load as next";
const CLOSE: &str = "Close and stop CUE";
const WAVE: &str = "CUE waveform: click to seek";

/// P1 is cueing its next entry; every track is 180 s long.
fn cueing(players: usize) -> (Harness<'static, AppUi>, Arc<Fake>) {
    let mut s = state(players, 3);
    for t in s.library.iter_mut() {
        t.duration_secs = 180.0;
    }
    let (mut h, fake) = harness(s);
    fake.send(Command::ToggleCue(fake.player(0)));
    fake.take_sent();
    h.run_steps(2);
    (h, fake)
}

fn sent(fake: &Fake) -> Vec<Command> {
    fake.take_sent()
}

#[test]
fn there_is_no_window_without_a_cue() {
    let (h, _) = harness(state(1, 3));
    for label in [PAUSE, STOP, LOAD_NEXT, CLOSE, WAVE] {
        assert!(h.query_by_label(label).is_none(), "{label}");
    }
}

#[test]
fn a_running_cue_opens_its_window_and_stopping_it_closes_it() {
    let (mut h, fake) = cueing(1);
    for label in [PAUSE, STOP, LOAD_NEXT, CLOSE, WAVE] {
        assert!(h.query_by_label(label).is_some(), "{label}");
    }
    h.get_by_label(STOP).click();
    h.run_steps(2);
    assert_eq!(sent(&fake), vec![Command::SetCue(fake.player(0), false)]);
    assert!(h.query_by_label(STOP).is_none(), "the window closed");
}

#[test]
fn the_close_button_stops_the_cue() {
    let (mut h, fake) = cueing(1);
    h.get_by_label(CLOSE).click();
    h.run_steps(2);
    assert_eq!(sent(&fake), vec![Command::SetCue(fake.player(0), false)]);
    assert!(h.query_by_label(CLOSE).is_none());
}

#[test]
fn pause_becomes_resume() {
    let (mut h, fake) = cueing(1);
    let p = fake.player(0);
    h.get_by_label(PAUSE).click();
    h.run_steps(2);
    assert_eq!(sent(&fake), vec![Command::SetCuePaused(p, true)]);
    assert!(h.query_by_label(PAUSE).is_none());
    h.get_by_label(RESUME).click();
    h.run_steps(2);
    assert_eq!(sent(&fake), vec![Command::SetCuePaused(p, false)]);
    assert!(h.query_by_label(PAUSE).is_some());
}

#[test]
fn load_as_next_sends_the_command_and_dims_once_it_is_explicit() {
    let (mut h, fake) = cueing(1);
    let p = fake.player(0);
    h.get_by_label(LOAD_NEXT).click();
    h.run_steps(2);
    assert_eq!(sent(&fake), vec![Command::CueToNext(p)]);
    assert!(h.get_by_label(LOAD_NEXT).accesskit_node().is_disabled());
    assert!(h.query_by_label(STOP).is_some(), "the CUE keeps running");
}

#[test]
fn clicking_the_waveform_seeks_the_cue() {
    let (mut h, fake) = cueing(1);
    let p = fake.player(0);
    let wave = h.get_by_label(WAVE).rect();
    let at = pos2(wave.left() + 1.0 + (wave.width() - 2.0) * 0.5, wave.center().y);
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    for pressed in [true, false] {
        h.event(Event::PointerButton {
            pos: at,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        });
    }
    h.run_steps(2);
    let seeks: Vec<f64> = sent(&fake)
        .into_iter()
        .filter_map(|c| match c {
            Command::SeekCue(id, secs) if id == p => Some(secs),
            _ => None,
        })
        .collect();
    assert_eq!(seeks.len(), 1, "{seeks:?}");
    assert!((seeks[0] - 90.0).abs() < 2.0, "{seeks:?}");
}

#[test]
fn the_window_shows_the_position_from_the_engine() {
    let (mut h, fake) = cueing(1);
    fake.telemetry
        .store(Arc::new(fp_engine::conductor::Telemetry {
            players: vec![(
                fake.player(0),
                fp_engine::engine::PlayerTelemetry {
                    cue_position_secs: Some(65.0),
                    ..Default::default()
                },
            )],
            ..Default::default()
        }));
    h.run_steps(2);
    assert!(h.query_by_label("01:05").is_some(), "elapsed");
    assert!(h.query_by_label("-01:55").is_some(), "remaining");
}

#[test]
fn two_cues_stack_two_windows() {
    let (mut h, fake) = cueing(2);
    fake.send(Command::ToggleCue(fake.player(1)));
    h.run_steps(2);
    assert_eq!(h.get_all_by_label(STOP).count(), 2);
    assert_eq!(h.get_all_by_label(WAVE).count(), 2);
    // Stopping the second leaves the first.
    h.get_all_by_label(STOP).nth(1).unwrap().click();
    h.run_steps(2);
    assert_eq!(h.get_all_by_label(STOP).count(), 1);
}

#[test]
fn a_cue_without_a_known_length_shows_zero_and_cannot_seek() {
    let mut s = state(1, 3);
    let first = s.playlists.iter().next().unwrap().entries[0].track;
    s.library.get_mut(first).unwrap().duration_secs = 0.0;
    let (mut h, fake) = harness(s);
    fake.send(Command::ToggleCue(fake.player(0)));
    fake.take_sent();
    h.run_steps(2);
    assert!(h.query_by_label(STOP).is_some());
    assert!(h.query_by_label(PAUSE).is_some());
    assert!(h.query_by_label("00:00").is_some());
    let wave = h.get_by_label(WAVE).rect();
    let at = wave.center();
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    for pressed in [true, false] {
        h.event(Event::PointerButton {
            pos: at,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        });
    }
    h.run_steps(2);
    assert!(sent(&fake).is_empty());
}

#[test]
fn a_cue_that_ends_by_itself_closes_its_window() {
    let (mut h, fake) = cueing(1);
    let p = fake.player(0);
    let entry = fake.state.load().players[0].cue.unwrap().entry;
    let mut s = (*fake.state.load_full()).clone();
    fp_model::on_event(&mut s, fp_model::EngineEvent::CueEnded { player: p, entry });
    fake.state.store(Arc::new(s));
    h.run_steps(2);
    assert!(h.query_by_label(STOP).is_none());
}
```

The file imports `fp_model::Command` only (no `apply`, no `TrackAnalysis`): change its `use fp_model::{...}` line to `use fp_model::Command;`.

In `crates/fp-app/tests/glyphs.rs`, add to the `sources` array of `the_screens_do_not_name_transport_icons_themselves`:

```rust
        ("cue_window.rs", include_str!("../src/ui/cue_window.rs")),
```

- [ ] **Step 2: Run them to verify they fail**

Run: `cargo test -p fp-app --test view the_cue_window_view_needs_a_running_cue`
Expected: FAIL to compile (`unresolved import cue_window_view`). `cargo test -p fp-app --test cue_window there_is_no_window_without_a_cue` fails to compile (the keys exist only after Step 3) or fails at the first assertion; `tests/glyphs.rs` fails to compile until `cue_window.rs` exists.

- [ ] **Step 3: Implement**

`crates/fp-app/src/ui/view.rs`, after `shown_entry`:

```rust
/// What a CUE window shows (feedback 2 spec O12). A CUE plays the whole
/// file, so its times run to the end of the file, not to the cue-out.
#[derive(Debug, Clone, PartialEq)]
pub struct CueWindowView {
    pub player: PlayerId,
    pub entry: EntryId,
    pub title: String,
    pub artist: Option<String>,
    pub elapsed: f64,
    /// File length, when known.
    pub total: Option<f64>,
    /// Seconds to the end of the file; 0 while the length is unknown.
    pub remaining: f64,
    pub paused: bool,
    /// The position as a fraction of the file, for the waveform.
    pub position: Option<f32>,
    /// "Load as next" has something to do: the cued entry is not the
    /// current one and not already the explicit next.
    pub can_load_next: bool,
}

/// The CUE window of `player`, or `None` while it has no CUE. `position`
/// is the engine's CUE position; without one (or a broken one) the CUE
/// shows the start of its play range, where it begins.
pub fn cue_window_view(
    state: &AppState,
    player: PlayerId,
    position: Option<f64>,
) -> Option<CueWindowView> {
    let p = state.player(player).ok()?;
    let cue = p.cue?;
    let track = state.track_for_entry(cue.entry)?;
    let total = (track.duration_secs > 0.0).then_some(track.duration_secs);
    let start = track.play_range(state.config.players.use_cue_markers).cue_in;
    let mut elapsed = position
        .filter(|v| v.is_finite())
        .unwrap_or(start)
        .max(0.0);
    if let Some(total) = total {
        elapsed = elapsed.min(total);
    }
    Some(CueWindowView {
        player,
        entry: cue.entry,
        title: track.title.clone(),
        artist: Some(track.artist.clone()).filter(|a| !a.is_empty()),
        elapsed,
        total,
        remaining: total.map_or(0.0, |t| (t - elapsed).max(0.0)),
        paused: cue.paused,
        position: fraction(Some(elapsed), total.unwrap_or(0.0)),
        can_load_next: p.current != Some(cue.entry)
            && !(p.next == Some(cue.entry) && p.next_explicit),
    })
}
```

`crates/fp-app/src/ui.rs`: add `mod cue_window;` after `pub mod controller;`.

Create `crates/fp-app/src/ui/cue_window.rs`:

```rust
//! The CUE window (feedback 2 spec O12): one floating, non-modal window per
//! running CUE, with the waveform and position, the elapsed and remaining
//! time, and Pause/Resume, Stop and Load as next. Closing it stops the CUE.
//! Everything it does is a command; nothing here waits for the engine.

use egui::{Align, Color32, Layout, RichText, Stroke, pos2, vec2};
use egui_phosphor::regular as icon;
use fp_model::Command;

use super::app::Scene;
use super::format;
use super::glyphs::{self, TransportAction};
use super::theme;
use super::view::{self, CueWindowView};
use super::widgets::{self, TileStyle, font, font_medium, font_semibold};

const WIDTH: f32 = 380.0;
const WAVE_HEIGHT: f32 = 56.0;
const BUTTON: egui::Vec2 = vec2(40.0, 28.0);
const LOAD_NEXT_WIDTH: f32 = 130.0;

/// Draws the window of every player that has a CUE running.
pub(crate) fn show_all(ctx: &egui::Context, scene: &Scene<'_>) {
    for (index, player) in scene.state.players.iter().enumerate() {
        if player.cue.is_none() {
            continue;
        }
        let position = scene
            .telemetry
            .players
            .iter()
            .find(|(id, _)| *id == player.id)
            .and_then(|(_, t)| t.cue_position_secs);
        if let Some(v) = view::cue_window_view(scene.state, player.id, position) {
            show(ctx, scene, index, &v);
        }
    }
}

fn show(ctx: &egui::Context, scene: &Scene<'_>, index: usize, v: &CueWindowView) {
    let t = scene.i18n;
    let stagger = 28.0 * index as f32;
    egui::Window::new(t.tr_args("cue-window-title", &[("n", (index + 1).into())]))
        .id(egui::Id::new(("cue-window", v.player)))
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .default_pos(pos2(80.0 + stagger, 120.0 + stagger))
        .frame(
            egui::Frame::new()
                .fill(theme::SURFACE)
                .stroke(Stroke::new(1.0, theme::CUE))
                .inner_margin(12.0),
        )
        .show(ctx, |ui| {
            ui.set_width(WIDTH);
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
            title_row(ui, scene, index, v);
            ui.add(
                egui::Label::new(
                    RichText::new(&v.title)
                        .font(font_medium(15.0))
                        .color(theme::TEXT),
                )
                .selectable(false)
                .truncate(),
            );
            let artist = v
                .artist
                .clone()
                .unwrap_or_else(|| t.tr("unknown-artist"));
            ui.add(
                egui::Label::new(
                    RichText::new(artist)
                        .font(font(13.0))
                        .color(theme::NEUTRAL_400),
                )
                .selectable(false)
                .truncate(),
            );
            wave(ui, scene, v);
            time_row(ui, v);
            buttons(ui, scene, v);
        });
}

fn title_row(ui: &mut egui::Ui, scene: &Scene<'_>, index: usize, v: &CueWindowView) {
    let t = scene.i18n;
    ui.horizontal(|ui| {
        ui.add(
            egui::Label::new(
                RichText::new(format!(
                    "{} {}",
                    glyphs::glyph_text(TransportAction::Cue),
                    t.tr_args("cue-window-title", &[("n", (index + 1).into())])
                ))
                .font(font_semibold(11.0))
                .color(theme::CUE),
            )
            .selectable(false),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let label = t.tr("cue-window-close");
            if widgets::tile(
                ui,
                vec2(24.0, 20.0),
                &label,
                true,
                TileStyle::plain(),
                |p, r, c| widgets::glyph(p, r, icon::X, 12.0, c, false),
            )
            .clicked()
            {
                scene.ctl.send(Command::SetCue(v.player, false));
            }
        });
    });
}

fn wave(ui: &mut egui::Ui, scene: &Scene<'_>, v: &CueWindowView) {
    let track = scene.state.playlists.entry(v.entry).map(|e| e.track);
    let media = track.and_then(|track| scene.media.get(track));
    let markers = view::MarkerFractions {
        position: v.position,
        ..view::MarkerFractions::default()
    };
    let label = scene.i18n.tr("tip-cue-waveform");
    let mix_label = String::new();
    let input = widgets::WaveInput {
        id: egui::Id::new(("cue-waveform", v.player)),
        media: media.as_ref(),
        total: v.total,
        markers,
        colors: theme::wave_colors(&scene.state.config.ui.wave_color),
        mix_active: false,
        mix_label: &mix_label,
        accessible_label: &label,
        view: None,
        shield: None,
        seekable: true,
    };
    let output = widgets::waveform(ui, WAVE_HEIGHT, &input);
    if let Some(secs) = output.seek {
        scene.ctl.send(Command::SeekCue(v.player, secs));
    }
}

fn time_row(ui: &mut egui::Ui, v: &CueWindowView) {
    ui.horizontal(|ui| {
        widgets::tabular_label(ui, &format::clock(v.elapsed), &font(12.0), theme::CUE);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let (minutes, _) = format::countdown(v.remaining);
            widgets::tabular_label(ui, &minutes, &font(12.0), theme::NEUTRAL_400);
        });
    });
}

fn buttons(ui: &mut egui::Ui, scene: &Scene<'_>, v: &CueWindowView) {
    let t = scene.i18n;
    ui.horizontal(|ui| {
        // Pause shows Pause while the CUE plays and Resume (a play
        // triangle) while it is held.
        let (action, key) = if v.paused {
            (TransportAction::Play, "cue-window-resume")
        } else {
            (TransportAction::Pause, "cue-window-pause")
        };
        let label = t.tr(key);
        if widgets::tile(ui, BUTTON, &label, true, TileStyle::plain(), move |p, r, c| {
            glyphs::paint(p, r, action, 14.0, c)
        })
        .clicked()
        {
            scene.ctl.send(Command::SetCuePaused(v.player, !v.paused));
        }
        let label = t.tr("cue-window-stop");
        if widgets::tile(ui, BUTTON, &label, true, TileStyle::plain(), |p, r, c| {
            glyphs::paint(p, r, TransportAction::Stop, 14.0, c)
        })
        .clicked()
        {
            scene.ctl.send(Command::SetCue(v.player, false));
        }
        let label = t.tr("cue-window-load-next");
        let text = format!("{}  {label}", icon::ARROW_BEND_DOWN_RIGHT);
        let style = TileStyle {
            content: if v.can_load_next {
                theme::CUE
            } else {
                Color32::from_gray(0x80)
            },
            ..TileStyle::plain()
        };
        if widgets::tile(
            ui,
            vec2(LOAD_NEXT_WIDTH, 28.0),
            &label,
            v.can_load_next,
            style,
            |p, r, c| {
                p.text(
                    r.center(),
                    egui::Align2::CENTER_CENTER,
                    &text,
                    font_semibold(11.0),
                    c,
                );
            },
        )
        .clicked()
        {
            scene.ctl.send(Command::CueToNext(v.player));
        }
    });
}
```

Notes for the implementer: `widgets::glyph(painter, rect, icon, size, color, fill)` is the real signature (`widgets.rs:182`); `theme::wave_colors(&str)` and `format::countdown` are used exactly as `player.rs` uses them; the disabled tile already dims its content, so replace the `Color32::from_gray(0x80)` branch with `theme::CUE` for both states if the dimming alone reads well in the screenshot check (the colour literal exists only to keep the first version honest; delete it with `Color32` from the imports if unused).

`crates/fp-app/src/ui/app.rs`: in `AppUi::ui`, right after `ui.allocate_rect(full, Sense::hover());` add:

```rust
        // The CUE windows float over the screen; the dialogs below stay on top.
        cue_window::show_all(&ctx, &scene);
```

and add `use super::cue_window;` with the other `use super::...` lines (the module is private to `ui`).

`crates/fp-app/locales/en-US/main.ftl`, after `tip-cue = ...`:

```
cue-window-title = CUE · P{ $n }
cue-window-pause = Pause CUE
cue-window-resume = Resume CUE
cue-window-stop = Stop CUE
cue-window-load-next = Load as next
cue-window-close = Close and stop CUE
tip-cue-waveform = CUE waveform: click to seek
```

`crates/fp-app/locales/es-ES/main.ftl`, after `tip-cue = ...`:

```
cue-window-title = CUE · P{ $n }
cue-window-pause = Pausar CUE
cue-window-resume = Reanudar CUE
cue-window-stop = Parar CUE
cue-window-load-next = Cargar como siguiente
cue-window-close = Cerrar y parar CUE
tip-cue-waveform = Forma de onda del CUE: clic para saltar
```

- [ ] **Step 4: Run the tests to verify they pass**

Run each:
- `cargo test -p fp-app --test view the_cue_window_view`
- `cargo test -p fp-app --test view a_cue_window_without_a_position`
- `cargo test -p fp-app --test view a_position_past_the_end_is_clamped`
- `cargo test -p fp-app --test view an_unknown_length_gives_zero_times`
- `cargo test -p fp-app --test view load_as_next_is_offered`
- `cargo test -p fp-app --test cue_window there_is_no_window_without_a_cue`
- `cargo test -p fp-app --test cue_window a_running_cue_opens_its_window`
- `cargo test -p fp-app --test cue_window the_close_button_stops_the_cue`
- `cargo test -p fp-app --test cue_window pause_becomes_resume`
- `cargo test -p fp-app --test cue_window load_as_next_sends`
- `cargo test -p fp-app --test cue_window clicking_the_waveform_seeks_the_cue`
- `cargo test -p fp-app --test cue_window the_window_shows_the_position`
- `cargo test -p fp-app --test cue_window two_cues_stack`
- `cargo test -p fp-app --test cue_window a_cue_without_a_known_length`
- `cargo test -p fp-app --test cue_window a_cue_that_ends_by_itself`
- `cargo test -p fp-app --test glyphs the_screens_do_not_name_transport_icons_themselves`
- `cargo test -p fp-app --test i18n both_locales_define_the_same_keys`
- `cargo test -p fp-app --test main_screen` (the new windows must not disturb the main-screen tests: none of them starts a CUE).

Expected: PASS. If a test cannot find a label because egui merges the tile into a parent node, query it by `get_all_by_label(...).next()` as `main_screen.rs` does for Play; do not rename the keys. The time labels (`01:05`, `-01:55`) are tabular labels: if they are painted rather than labelled, assert through `view::cue_window_view` only (already tested) and drop those two assertions, keeping the position/telemetry wiring covered by the clamp tests.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-app
git commit -m "feat(ui): add the CUE window with seek, pause, stop and load as next"
```

---

### Task 6: A single click moves a running CUE (O17, UI half)

**Files:**
- Modify: `crates/fp-app/src/ui/view.rs` (`cue_follow_target`)
- Modify: `crates/fp-app/src/ui/table.rs` (the row click, near lines 81 and 339)
- Test: `crates/fp-app/tests/view.rs`, `crates/fp-app/tests/main_screen.rs`

**Interfaces:**
- Consumes: `PlayerState::cue`, `AppState::playable_request`, `Command::CueEntry(PlayerId, EntryId)`.
- Produces: `view::cue_follow_target(state: &AppState, player: PlayerId, clicked: EntryId) -> Option<EntryId>`: `Some(clicked)` when the player has a CUE on another entry and `clicked` can be played.

- [ ] **Step 1: Write the failing tests**

Append to `crates/fp-app/tests/view.rs` (add `cue_follow_target` to the `use fp_app::ui::view::{...}` list):

```rust
#[test]
fn a_click_moves_a_running_cue_to_a_playable_other_row() {
    let (mut s, e, p) = state(3);
    assert_eq!(cue_follow_target(&s, p, e[2]), None, "no CUE running");
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    assert_eq!(cue_follow_target(&s, p, e[2]), Some(e[2]));
    assert_eq!(cue_follow_target(&s, p, e[0]), None, "already cued");
}

#[test]
fn a_click_on_a_missing_or_unreadable_file_leaves_the_cue() {
    let (mut s, e, p) = state(3);
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    for file in [FileState::Missing, FileState::Unreadable] {
        let track = s.playlists.entry(e[1]).unwrap().track;
        apply(&mut s, Command::SetFileState { track, state: file }).unwrap();
        assert_eq!(cue_follow_target(&s, p, e[1]), None, "{file:?}");
    }
}

#[test]
fn an_unknown_player_or_entry_gives_none() {
    let (mut s, e, p) = state(2);
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    assert_eq!(cue_follow_target(&s, PlayerId(99), e[1]), None);
    assert_eq!(cue_follow_target(&s, p, EntryId(9999)), None);
}
```

Append to `crates/fp-app/tests/main_screen.rs` (it already imports `Command`, `harness`, `state`, `Fake`; add `use fp_app::ui::controller::Controller;` if absent):

```rust
#[test]
fn clicking_a_row_moves_a_running_cue_to_it() {
    let (mut h, fake) = harness(state(1, 3));
    let p = fake.player(0);
    let e = fake.entries();
    fake.send(Command::ToggleCue(p));
    fake.take_sent();
    h.run_steps(2);
    h.get_by_label("Song 3").click();
    h.run_steps(2);
    assert!(sent(&fake).contains(&Command::CueEntry(p, e[2])));
}

#[test]
fn clicking_the_cued_row_a_missing_file_or_without_a_cue_sends_nothing() {
    let (mut h, fake) = harness(state(1, 3));
    let p = fake.player(0);
    let e = fake.entries();
    // No CUE: a click only selects.
    h.get_by_label("Song 2").click();
    h.run_steps(2);
    assert!(!sent(&fake).iter().any(|c| matches!(c, Command::CueEntry(..))));
    // The cued row (the next, Song 1): nothing to move.
    fake.send(Command::ToggleCue(p));
    fake.take_sent();
    h.run_steps(2);
    h.get_by_label("Song 1").click();
    h.run_steps(2);
    assert!(!sent(&fake).iter().any(|c| matches!(c, Command::CueEntry(..))));
    // A missing file.
    let track = fake.state.load().playlists.entry(e[2]).unwrap().track;
    fake.send(Command::SetFileState {
        track,
        state: fp_model::FileState::Missing,
    });
    fake.take_sent();
    h.run_steps(2);
    h.get_by_label("Song 3").click();
    h.run_steps(2);
    assert!(!sent(&fake).iter().any(|c| matches!(c, Command::CueEntry(..))));
}

#[test]
fn a_double_click_moves_the_cue_once() {
    let (mut h, fake) = harness(state(1, 3));
    let p = fake.player(0);
    let e = fake.entries();
    fake.send(Command::ToggleCue(p));
    fake.take_sent();
    h.run_steps(2);
    h.get_by_label("Song 3").click();
    h.step();
    h.get_by_label("Song 3").click();
    h.run_steps(2);
    let commands = sent(&fake);
    assert!(commands.contains(&Command::SetNext(p, e[2])));
    let moves = commands
        .iter()
        .filter(|c| matches!(c, Command::CueEntry(..)))
        .count();
    assert_eq!(moves, 1, "{commands:?}");
    assert_eq!(fake.state.load().players[0].cue.unwrap().entry, e[2]);
}
```

- [ ] **Step 2: Run them to verify they fail**

Run: `cargo test -p fp-app --test view a_click_moves_a_running_cue_to_a_playable_other_row`
Expected: FAIL to compile (`cue_follow_target` unresolved). `cargo test -p fp-app --test main_screen clicking_a_row_moves_a_running_cue_to_it` fails at the assertion (no `CueEntry` is sent).

- [ ] **Step 3: Implement**

`crates/fp-app/src/ui/view.rs`, after `cue_window_view`:

```rust
/// Feedback 2 spec O17: while `player`'s CUE runs, a single click on a row
/// moves it to that entry. `Some(entry)` when the CUE is on another entry
/// and `clicked` can be played; `None` for no CUE, the entry already cued,
/// or a file that is missing or unreadable. Selection is UI state, so this
/// decision is here and the move itself is the model's `CueEntry`.
pub fn cue_follow_target(state: &AppState, player: PlayerId, clicked: EntryId) -> Option<EntryId> {
    let cue = state.player(player).ok()?.cue?;
    (cue.entry != clicked && state.playable_request(clicked).is_some()).then_some(clicked)
}
```

`crates/fp-app/src/ui/table.rs`: next to `let mut clicked: Option<EntryId> = None;` (line 81) add

```rust
    // O17: the entry a running CUE moves to after a primary click.
    let mut cue_follow: Option<EntryId> = None;
```

and change the click handler (near line 339) to

```rust
                if response.clicked() {
                    clicked = Some(entry.id);
                    cue_follow = view::cue_follow_target(scene.state, player, entry.id);
                }
```

then, right after the `if let Some(entry) = clicked.or(dragged) { ... }` block that updates the selection, add

```rust
    if let Some(entry) = cue_follow {
        scene.ctl.send(Command::CueEntry(player, entry));
    }
```

(Only `response.clicked()` feeds `cue_follow`; the context-menu path that also sets `clicked` does not, so a right-click selects without moving the CUE.)

- [ ] **Step 4: Run the tests to verify they pass**

Run each:
- `cargo test -p fp-app --test view a_click_moves_a_running_cue`
- `cargo test -p fp-app --test view a_click_on_a_missing_or_unreadable_file_leaves_the_cue`
- `cargo test -p fp-app --test view an_unknown_player_or_entry_gives_none`
- `cargo test -p fp-app --test main_screen clicking_a_row_moves_a_running_cue_to_it`
- `cargo test -p fp-app --test main_screen clicking_the_cued_row_a_missing_file`
- `cargo test -p fp-app --test main_screen a_double_click_moves_the_cue_once`
- `cargo test -p fp-app --test main_screen double_clicking_a_row_sets_next`
- `cargo test -p fp-app --test main_screen the_context_menu_removes_an_entry`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-app
git commit -m "feat(ui): a single click moves a running CUE to the clicked row"
```

---

### Task 7: Documentation

**Files:**
- Modify: `docs/user/players.md` (header table row, Waveform bullets, CUE section)
- Modify: `docs/user/playlists.md` (the "Set as next" / "Pre-listen on CUE" menu lines)
- Modify: `docs/technical/audio-engine.md` (the action table row and the CUE paragraph)
- Modify: `docs/technical/ui.md` (module table, waveform view section, CUE window section)
- Modify: `docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md` (status line and "As built" under §7)
- Modify: `docs/superpowers/plans/2026-10-01-operator-feedback-2-roadmap.md` (row 6 status `done`)
- Modify: `README.md` (the CUE feature line)
- Modify: `CLAUDE.md` only if a command or layout line changed (none does: the new files sit in existing crates; leave it).

**Interfaces:** none (prose). Every claim must match the code as built: re-read `entry_notice.rs`, `reducer.rs` (`follow_cue`, `seek_cue`, `set_cue_paused`), `engine.rs` (`seek_cue`, `set_cue_paused`), `widgets.rs` (`WaveOutput`), `cue_window.rs` before writing.

- [ ] **Step 1: Update the user guide**

`docs/user/players.md`:

- In the header table, after the **Stop after** badge row, add: `| **Repeat** / **Stop after track** badge | The current track repeats, or stops the player when it ends, because of its own mark in the playlist menu. Hover for the full sentence. The player's own **Stop after** button wins: while it is on, only its badge shows |`.
- In the Waveform section replace the two bullets "Hover to see..." and "**Press and drag**..." by: "Hover to see the time under the pointer. **Click to jump** there (while playing or paused; a stopped player always starts at the cue-in). A click is a press and release without moving the pointer more than a few pixels." and "**Press and drag** moves the zoomed view along the track, like grabbing it. A drag never jumps, and without zoom it does nothing. Alt-drag still edits markers."
- Replace the CUE section body by the description of the window: pressing **CUE** (or **Pre-listen on CUE** in a track's menu) plays the track on the CUE output and opens a small **CUE window** for that player; several windows can be open, one per player. The window shows the title and artist, the whole-file waveform with the CUE position (click it to jump), the elapsed and remaining time (to the end of the file: CUE plays whole files), and the buttons **Pause**/**Resume**, **Stop** and **Load as next** (makes the cued track the player's next, as a double-click would, and keeps the CUE playing; dimmed when it already is the next, or is the current track). The close button of the window, or **Stop**, stops the CUE. While a CUE runs, setting a next (double-click) or a single click on a row moves it to that track from its cue-in, and playing there resumes it if it was paused; a track whose file is missing leaves the CUE where it is.

`docs/user/playlists.md`: in the "Set as next" row add "While a CUE is running it moves to the new next"; in the "Pre-listen on CUE" row add "(it opens the CUE window)". Keep the table pipes aligned with the surrounding rows.

- [ ] **Step 2: Update the technical docs**

`docs/technical/audio-engine.md`: change the `StartCue`, `StopCue` row to also list `SeekCue` (a replacement CUE source at the target; idle when the CUE is held, so a held CUE stays held) and `SetCuePaused` (`BusCommand::Pause`/`Resume` with the pause ramp for an audible source; a source that has not started is held idle and started on release; `PlayerRuntime::cue_paused` remembers the state and a new CUE clears it). One sentence that the CUE position is `PlayerTelemetry::cue_position_secs` and a held CUE reports a constant one.

`docs/technical/ui.md`: add a row for `ui/cue_window.rs` to the module table ("one non-modal `egui::Window` per running CUE, drawn from the pure `view::cue_window_view`; icons through `ui/glyphs.rs`"), and update the `ui/glyphs.rs` row's "and the CUE window will too" to "and the CUE window too". In "Waveform view" replace "drag-to-seek" and the `SeekDrag` sentence with: the waveform reports a click's seek target and a drag's sideways movement (`WaveOutput { response, seek, pan_dx }`); the player pans a zoomed view with `WaveView::pan`; `widgets::pan_dragging` pins the view against following while a drag is held; egui's click rule is the drag threshold. Add a short "CUE window" section: the window per player, the commands it sends (`SeekCue`, `SetCuePaused`, `CueToNext`, `SetCue`), the model rules (`follow_cue` on `SetNext`, `cue_entry` clears `paused`), and `view::cue_follow_target` for the table click (O17).

- [ ] **Step 3: Update the spec, roadmap and README**

Spec §7 (after the O17 bullet): mark status and add an "As built" list, as plans 5 did for §6:

```markdown
- **As built.**
  - O8: `fp_model::entry_notice(state, player) -> Option<EntryNotice>` (`Repeats`, `StopsAfter`) decides; the header draws an outlined amber badge (`badge-entry-repeat`, `badge-entry-stop`) whose tooltip and accessible name are the spec's sentences. None while stopped, during a fade stop, or while stop after current is set; the entry's stop-after wins over its repeat; an unreadable file does not repeat.
  - O10: `widgets::waveform` returns `WaveOutput { response, seek, pan_dx }`. Only `Response::clicked()` seeks. A primary drag that does not start with Alt or under the Full view button pans a zoomed view and does nothing otherwise. The hover time stays; the drag preview and Esc-cancel are gone.
  - O12: `CueState { entry, paused }`; `Command::{SeekCue, SetCuePaused, CueToNext}`; `EngineAction::{SeekCue, SetCuePaused}`. A CUE plays whole files, so the window's waveform and times cover the whole file. A moved CUE starts unpaused. `CueToNext` uses `set_next` (it becomes explicit; the current entry is refused). The window is `ui/cue_window.rs`, one per running CUE, with its own close button that stops the CUE. The remote API and MIDI do not expose pause or seek of the CUE.
  - O17: `SetNext` calls `follow_cue`: the CUE moves to a playable new next from its cue-in unless it is already there. The table sends `CueEntry` on a primary click through `view::cue_follow_target` (not for the cued row, a missing or unreadable file, or a player without a CUE).
```

Roadmap: set row 6's status to `done`. README: extend the line `- **CUE pre-listen** on a separate device or channel pair.` to `- **CUE pre-listen** on a separate device or channel pair, with a window per CUE to seek, pause, stop and load the track as next.`

- [ ] **Step 4: Check the docs**

Run: `grep -rn "SeekDrag\|seek_dragging\|release the button over the waveform\|CUE window will" docs README.md crates` and expect no stale hit (the old F2 spec text under `docs/superpowers/specs/2026-09-30-operator-feedback-design.md` and its plan may keep describing F2 as history: do not edit them; the O10 note in the new spec says it replaces F2). Run `scripts/check-commits.sh origin/master` after committing.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add docs README.md
git commit -m "docs: describe the CUE window, per-entry notice and waveform drag"
```

---

## Self-Review

**Spec coverage.**
- O8: Task 2 (rule, view, header badge, both locales, docs in Task 7). Precedence of stop after current: `stop_after_current_takes_precedence_over_both` and the UI test.
- O10: Task 4 (click seeks, drag pans, no zoom does nothing, preview on drag removed, Alt-drag untouched: `markers_ui` is re-run; the existing `alt_dragging_a_marker_in_a_zoomed_view_sets_the_zoomed_time` stays).
- O12: model (Task 1: `paused`, three commands, rule tests), engine (Task 3: seek and pause with Offline tests; position through `cue_position_secs`, paused through the snapshot), window (Task 5: title, artist, waveform with position and click-seek, elapsed and remaining, Pause/Resume, Stop, Load as next, close stops the CUE, several windows stack).
- O17: model rule on `SetNext` (Task 1) and the single-click `CueEntry` (Task 6).
- Docs, README, spec "As built", roadmap: Task 7. Locales in both languages inside the tasks that add them (2 and 5).

**Placeholder scan.** No TBD. Two places say "if X then Y" about the harness (label lookup of painted text in Task 5 Step 4; ordering note in Task 3): both give the concrete fallback. 

**Type consistency.** `CueState { entry, paused }` (Task 1) is read by `cue_window_view` and `cue_follow_target` (Tasks 5 and 6). `EngineAction::SeekCue { player, secs }` and `SetCuePaused { player, paused }` are named the same in Tasks 1 and 3. `Command::{SeekCue(PlayerId, f64), SetCuePaused(PlayerId, bool), CueToNext(PlayerId)}` are used with those shapes in Tasks 1, 5 and the test helpers. `WaveOutput { response, seek, pan_dx }` and `pan_dragging` (Task 4) are what Task 5's `wave()` consumes (`output.seek`). `WaveInput` loses `entry` in Task 4 and Task 5's literal has none. `EntryNotice::{Repeats, StopsAfter}` and `entry_notice` are the same in Task 2's model, view and header code. `view::cue_window_view` and `view::cue_follow_target` signatures match their tests.

**Review Focus.** Each of the five lines has its test named in the owning task.

**Known weak spot to watch in review.** The engine's held-position test depends on the pause ramp finishing within ten blocks (480 frames each); the step says to raise the run count, never to loosen the equality. The O10 numeric windows in the pan test are deliberately wide because egui drops the first pixels of a drag.
