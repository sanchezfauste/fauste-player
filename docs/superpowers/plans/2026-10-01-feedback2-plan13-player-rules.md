# Player Rules (Feedback 2, Plan 13) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Two player rules from operator feedback: the entry on air can be set as next, and then plays once more from its cue-in (O37); and in Single mode the stop-after-current control ends the repeat of a repeating entry (O38).

**Architecture:**
- **No new state, no new engine action.** "Next is the current entry" is the existing pair `player.next == player.current` with `next_explicit == true`. A derived next (`next_playable_after`) never wraps, so it can never equal the current entry: a self-next is always an explicit operator choice. The engine already executes a gapless hard restart into the entry it is playing (the repeat, R26): `Preload { request for the current entry }` plus `Schedule { StartNextAt { at_secs: cue_out, fade_current_until_secs: None } }`, which the mixer carries out and reports as `TransitionStarted { player, entry }` for the same entry. Both come out of `reconcile` once `plan_for` and `preload_target` look at the right fields.
- **The difference between a repeat pass and a replay is decided in the model, in `on_event(TransitionStarted)`.** When the reported entry is still current and the entry is `repeating(..)`, it is a repeat pass (unchanged: not played, no history). Otherwise, if `next == entry`, it is the replay: the model runs `advance_to(i, entry, true)`, the same function every ordinary play uses. That marks the entry played, records it in the history, derives the next from the playlist (the self-next is consumed, because the derived next is not `Some(target)`), and clears the preload and plan so `reconcile` sends fresh ones. The pass after that is an ordinary one.
- **O38 is a guard change plus an invariant.** A pure helper `repeat_entry` (R26 without the stop-after-current test) is shared by `repeating`, `availability`, `ToggleStopAfterCurrent`, `SetMode` and the session restore. In Single mode the flag is meaningful only while the current entry repeats; `reconcile` drops it as soon as that stops being true.
- **Remote API and MIDI need no code of their own.** Both already go through `command_available`, so they follow `availability`; only their tests and the docs change.

**Tech Stack:** Rust, egui/eframe 0.36.2, egui_kittest 0.36.2. No new dependency, no new `Config` field (the rules are not operator values).

**Spec:** `docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md` §14 (plan 13: O37, O38; items table rows O37, O38) and §15 (global constraints). Main design spec `docs/superpowers/specs/2026-09-25-fauste-player-design.md` (rule 2: set next; rule 27: repeat, R26; rule 28: stop after this entry, R27). Roadmap: `docs/superpowers/plans/2026-10-01-operator-feedback-2-roadmap.md` (plan 13, branch `feat/player-rules`).

**Base.** Branch `feat/player-rules` from `master` at `0c931c2` (after plan 12). Line numbers below are from that tree and move; steps name functions and give the text to find. Find code with the CodeGraph index (`codegraph_explore`, `projectPath` = the repo) before grepping.

## Decisions taken

The spec leaves these open. Each is the most conservative reading.

- **Representation.** `next == current` with `next_explicit == true` (see Architecture). No `Replay` flag, no new `TransitionPlan` variant, no new `EngineAction`. `set_next` loses its `NextIsCurrent` refusal; the `ModelError::NextIsCurrent` variant, its UI message `error-next-is-current` (both locales) and its arm in `ui/app.rs` are removed (nothing can return it any more).
- **What the plan says for a self-next** (`plan_for`, after the existing order R27 entry stop-after, R26 repeat, then Single / stop-after-current / no next → `StopAt`): a self-next gets `StartNextAt { at_secs: cue_out, fade_current_until_secs: None }`, always hard. A segue into the same file would overlap the entry with itself, so the segue is skipped (spec: "the same gapless hard transition as a repeating entry"). A self-next on a file that can no longer be opened gets `StopAt` (the pass plays out, as a repeat does). `preload_target` needs no change: with no repeat active it returns `player.next`, which is the current entry, and `request_from_cue_in` gives its cue-in.
- **Repeat wins over a dormant self-next.** While the entry repeats, `repeating()` is true and `TransitionStarted` is a repeat pass; the self-next stays in the state, unconsumed. When the repeat ends by turning `repeat` off (not by stop-after-current, which also stops the player), the next boundary is a replay, then the player goes on. Rationale: the operator asked for it and nothing cancelled it; the same rule as "an explicit next survives".
- **What still wins over the next entry (O37):** an entry's `stop_after` (R27), Single mode, stop after current, a fade stop: all are earlier in `plan_for` and unchanged. The self-next stays explicit through the stop (`stop_player` keeps an explicit next), so after the stop Play starts that entry again from its cue-in. This is the spec's "In Single mode the player stops at the end and Play starts the entry again", and it holds in Continuous too after a stop.
- **Play / Next while the entry is on air with a self-next.** `play()` calls `advance()`, which runs `advance_to(current)`: a crossfade (the usual `fade_ms`) from the playing source into a fresh start of the same entry at its cue-in; the entry is marked played, recorded in the history and the next is derived. Previous skips a history entry equal to the one left, as today. Paused + Play resumes (no replay). Stop / Fade stop end it; the self-next survives, as any explicit next does.
- **Race at the boundary.** If a command changes the model while the mixer is already crossing the boundary, the event arrives for an entry that is still current. The decision is taken from the model at event time (`repeating` now, else `next == entry`): the same tolerance the repeat already has (`next_pressed_just_after_a_repeat_boundary_...` in `crates/fp-engine/tests/conductor.rs`).
- **Removing or moving.** The entry on air cannot be removed (`EntryOnAir`); a move keeps its id, so the self-next stays and the following entry is derived from the new position when the replay starts. A removal of other entries never touches a self-next. A file that becomes unreadable while it is on air with a self-next: the pending preload fails (`preload_failed`) or the source fails (`source_failed`); in both the self-next is replaced by the entry after it (`Successors::for_current`), so Continuous goes on with the following entry instead of stopping.
- **Session restore.** `session.rs` used to drop a restored next equal to the current entry. It now keeps an explicit one: the operator's choice survives a restart (the player restores paused; nothing goes on air by itself).
- **UI.** The playing row keeps its red on-air look and gets the green next arrow beside the speaker icon, so it is marked as next too. The context menu item "Set as next" is enabled on the playing row ("Play now" stays disabled there). **Double-click on the playing row is left as it is (does nothing)**: an accidental double click on the track on air must not schedule a replay; the menu item is the deliberate way. The CUE window's "Load as next" also accepts the entry on air (it uses `CueToNext`, which uses `set_next`), and is dimmed only when that entry already is the explicit next.
- **O38.** In Single mode `ToggleStopAfterCurrent` / `SetStopAfterCurrent` are allowed exactly while `repeat_entry` is true for the player (playing or paused, current entry has `repeat`, no entry `stop_after`, file playable, no fade stop). The stop-after-current badge and the entry notice rules are unchanged (`entry_notice` already hides the repeat notice while the flag is set, so the same notice as in Continuous mode shows). `SetMode(Single)` keeps the flag only while `repeat_entry` is true, and clears it otherwise. The flag then ends with the pass in progress at its cue-out (`plan_for` already gives `StopAt` when `stop_after_current`), and `stop_player` clears it. Without a repeating entry in Single mode the refusal is today's `StopAfterInSingle` (the remote API's `409`).
- **Invariant for the Single-mode flag.** `Single && stop_after_current` implies `repeat_entry`. `reconcile` enforces it after every command and event, so toggling repeat off, `stop_after` on the entry, Next into a non-repeating entry or a file turning unreadable all drop a stale flag by themselves.

## Global Constraints

- All code, identifiers, comments, docs, specs, plans and commit messages are in English. Never mention other playout, radio-automation or tag-editor products.
- Spec §15 (binding; the spec's own words):

> `CLAUDE.md` rules 1–10 apply to every plan. In particular:
>
> - English everywhere, and UI strings in both locales;
> - no product names;
> - operator values are `Config` fields with defaults, ranges and lenient
>   loading;
> - the real-time path never allocates, locks, logs or panics;
> - behaviour lives in `fp-model`;
> - the UI never blocks (tag writing, restart and file work run on helper
>   threads);
> - bad data never crashes;
> - nothing goes on air by itself.
>
> TDD throughout. Each plan ends with a docs task and a review by a fresh
> reviewer.

- Spec §14 (binding; the spec's own words):

> - **O37 Play the current entry once more.** "Set as next" is accepted for
>   the entry that is on air (the `NextIsCurrent` refusal goes). The player
>   plays that entry once more from its cue-in when the current pass ends,
>   with the same gapless hard transition as a repeating entry (main spec
>   rule 27), and then goes on with the entry that follows it. Unlike
>   `repeat`, it acts once: the second pass is an ordinary play (it becomes
>   current again, is marked played and enters the history), and the next
>   entry is then worked out as usual. Everything that already wins over the
>   next entry still wins: Single mode, stop after current, an entry's "stop
>   after", a fade stop. In Single mode the player stops at the end and Play
>   starts the entry again. The playlist marks the playing row as next too,
>   and its "Set as next" action is enabled on that row. The remote API and
>   MIDI follow the same rule.
> - **O38 Stop after current in Single mode.** In Single mode, while the
>   current entry has `repeat`, the stop-after-current control is available:
>   it ends the repeat at the cue-out of the pass that is playing (main spec
>   rule 27 already lets stop-after-current end a repeat) and shows the same
>   notice as in Continuous mode. Switching to Single mode keeps the flag while
>   the current entry repeats. In Single mode without a repeating entry it is
>   refused as today (`StopAfterInSingle`, the remote API's `409`), since the
>   player stops at the end anyway.

- CLAUDE.md rule 5: nothing here touches the device callback. The model changes are pure; the engine is not edited.
- CLAUDE.md rule 6: `unsafe_code` is forbidden; no `unwrap`, `expect` or `panic` outside tests (tests carry the `#![allow(...)]` header the neighbouring test files use); prefer `get` to indexing in `fp-engine`, `fp-backends`, `fp-decode`, `fp-analysis` (and everywhere new).
- CLAUDE.md rule 7: behaviour is `fp-model` pure functions, one test per rule; the engine and UI only execute and display.
- CLAUDE.md rule 10: nothing goes on air by itself. A restored self-next and a restored flag leave the player paused.
- UI strings are Fluent messages in `crates/fp-app/locales/en-US/main.ftl` and `es-ES/main.ftl`, always both (`tests/i18n.rs` checks the keys match).
- UI tests: `egui_kittest` with `Harness::builder().with_step_dt(0.02)` (through `support::harness` / `support::harness_sized`) and the recording `Fake` controller in `crates/fp-app/tests/support`. Remote tests: `tower::ServiceExt::oneshot` and `FakeControl` in `crates/fp-remote/tests/support`. Engine tests: the `Offline` backend and explicit `now: Instant`; never sleep to wait for audio, never open the real sound system.
- Builds go only into the repo's `target/`.
- Gate for every commit (run from the repo root):

```bash
export PATH=$HOME/.cargo/bin:$PATH; cargo fmt --all --check && cargo clippy --workspace --all-targets -q -- -D warnings && cargo test --workspace -q
scripts/check-commits.sh origin/master   # no output = pass
```

  Commit subjects are Conventional Commits and end with the trailer `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>` (the commit commands below show the subject only). `CHANGELOG.md` is never edited by hand.
- Documentation is part of each task (CLAUDE.md "Documentation is part of every change"); Task 6 does the spec "As built", the roadmap and a last consistency sweep.

## Review Focus

Failure modes the spec implies and its tests do not name; each has a test in the task named.

1. **The playing entry's file is unreadable or goes missing while a self-next is pending** (preload fails, or the source fails): the player must not stop dead nor loop; it goes on with the entry after it (Continuous) or stops cleanly (Single). (Task 2 `a_self_next_whose_preload_fails_is_replaced_by_the_following_entry`, `a_self_next_on_a_failing_source_continues_with_the_following_entry`, `plan_for_stops_at_the_end_for_a_self_next_on_an_unplayable_file`.)
2. **Self-next on the last entry of the playlist, or on a one-entry playlist**: after the replay there is no following entry; the player must stop at the end of the second pass, not replay forever, and not panic. (Task 1 `the_replay_of_the_last_entry_leaves_no_next_and_stops_after_the_second_pass`.)
3. **The operator reorders, edits or removes around the self-next** (moves the playing entry, deletes the entry after it, switches the shown playlist, Previous): the self-next survives a move, the following entry is derived at replay time, nothing refers to a removed entry. (Task 2 `moving_the_entry_keeps_the_self_next_and_the_following_entry_follows_the_new_position`, `removing_the_entry_after_the_current_one_does_not_touch_a_self_next`, `previous_overrides_a_self_next`.)
4. **Session restart with a self-next and, in Single mode, the stop-after-current flag**: both survive (player paused, nothing on air), and a flag without a repeating entry is dropped. (Task 2 `a_restored_self_next_is_kept_and_the_player_stays_paused`; Task 3 `restore_keeps_the_single_mode_flag_only_for_a_repeating_entry`.)
5. **The Single-mode flag outliving its repeat** (repeat toggled off, Next into a plain entry, the entry marked "stop after", Stop): the control must dim again and the flag must not silently stop a later entry. (Task 3 `the_single_mode_flag_is_dropped_when_the_repeat_stops_applying`, `the_flag_ends_with_stop_and_does_not_reach_the_next_play`.)

## File Structure

- Create `crates/fp-model/tests/replay_next.rs` (O37 rules), `crates/fp-model/tests/single_stop_after.rs` (O38 rules), `crates/fp-app/tests/next_row_ui.rs` (O37 and O38 drawing).
- Modify `crates/fp-model/src/reducer.rs` (`set_next`, `plan_for`, `repeating`/`repeat_entry`, `on_event`, `source_failed`, `preload_failed`, `ToggleStopAfterCurrent`, `SetMode`, `reconcile`), `crates/fp-model/src/error.rs` (remove `NextIsCurrent`, reword `StopAfterInSingle`), `crates/fp-model/src/availability.rs`, `crates/fp-model/src/session.rs`.
- Modify `crates/fp-app/src/ui/view.rs` (`row_is_next`, `can_load_next`), `crates/fp-app/src/ui/table.rs` (row marking, menu), `crates/fp-app/src/ui/player.rs` (stop-after tooltip), `crates/fp-app/src/ui/app.rs` (error arm), both locales.
- Modify existing tests that asserted the old refusals: `crates/fp-model/tests/transport.rs` (`rule2_double_click_sets_next_but_never_the_current_entry`), `crates/fp-model/tests/cue_window.rs` (`load_as_next_of_the_current_entry_is_refused`), `crates/fp-model/tests/transitions.rs` / `set_values.rs` / `availability.rs` (whatever asserts `StopAfterInSingle`; find with `grep -rn "StopAfterInSingle\|stop_after_current" crates/*/tests`), `crates/fp-app/tests/view.rs` (`can_load_next`), `crates/fp-remote/tests/api.rs`.
- Add to `crates/fp-engine/tests/conductor.rs` (end to end, Offline backend).
- Modify docs: `docs/user/players.md`, `docs/user/playlists.md`, `docs/technical/audio-engine.md`, `docs/technical/remote-api.md`, `docs/user/remote-control.md`, `docs/user/midi.md` (one sentence if it lists the rule), the main spec (rules 2, 27, 28), the feedback 2 spec (§14 "As built", status line) and the roadmap (row 13 done).

---

### Task 1: O37 model: set the playing entry as next, and the replay

**Files:**
- Modify: `crates/fp-model/src/reducer.rs` (`set_next`, `plan_for`, `on_event`), `crates/fp-model/src/error.rs` (remove `NextIsCurrent`), `crates/fp-app/src/ui/app.rs` (remove the `NextIsCurrent` arm near line 1363), `crates/fp-app/locales/en-US/main.ftl` and `es-ES/main.ftl` (remove `error-next-is-current`)
- Test: `crates/fp-model/tests/replay_next.rs` (new); update `crates/fp-model/tests/transport.rs`, `crates/fp-model/tests/cue_window.rs`, `crates/fp-remote/tests/api.rs`
- Docs: main spec rules 2, 27 (R26 text) in `docs/superpowers/specs/2026-09-25-fauste-player-design.md`

**Interfaces:**
- Produces (used by every later task): after `Command::SetNext(p, current)`, `player.next == player.current` and `player.next_explicit == true`; `plan_for` returns `StartNextAt { at_secs: end, fade_current_until_secs: None }` for it; `on_event(TransitionStarted { player, entry: current })` with a self-next and no active repeat runs the ordinary play bookkeeping. `ModelError::NextIsCurrent` no longer exists.

- [ ] **Step 1: Write the failing tests**

Create `crates/fp-model/tests/replay_next.rs`:

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback 2 spec O37: the entry on air can be set as next and then plays
//! once more from its cue-in (a hard transition like a repeat, R26), as an
//! ordinary play.

mod common;

use common::{entries, fixture, p0};
use fp_model::{
    AppState, Command, EngineAction, EngineEvent, EntryId, PlayMode, PlayerId, TransitionPlan,
    Transport, apply, on_event, plan_for,
};

fn three() -> (AppState, PlayerId, [EntryId; 3]) {
    let state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    (state, p, [e[0], e[1], e[2]])
}

fn plan(s: &AppState, p: PlayerId) -> Option<TransitionPlan> {
    plan_for(s, s.player(p).unwrap())
}

fn preload(out: &[EngineAction], p: PlayerId) -> Option<Option<EntryId>> {
    out.iter().rev().find_map(|a| match a {
        EngineAction::Preload { player, request } if *player == p => {
            Some(request.as_ref().map(|r| r.entry))
        }
        _ => None,
    })
}

const HARD_END: TransitionPlan = TransitionPlan::StartNextAt {
    at_secs: 180.0,
    fade_current_until_secs: None,
};

fn started(s: &mut AppState, p: PlayerId, e: EntryId) -> Vec<EngineAction> {
    on_event(s, EngineEvent::TransitionStarted { player: p, entry: e })
}

/// Entry `a` is on air with itself as next.
fn replaying() -> (AppState, PlayerId, [EntryId; 3]) {
    let (mut s, p, e) = three();
    apply(&mut s, Command::Play(p)).unwrap();
    apply(&mut s, Command::SetNext(p, e[0])).unwrap();
    (s, p, e)
}

#[test]
fn o37_the_entry_on_air_can_be_set_as_next() {
    let (mut s, p, [a, _, _]) = three();
    apply(&mut s, Command::Play(p)).unwrap();
    apply(&mut s, Command::SetNext(p, a)).unwrap();
    let player = s.player(p).unwrap();
    assert_eq!((player.current, player.next, player.next_explicit), (Some(a), Some(a), true));
}

#[test]
fn o37_a_self_next_preloads_its_own_cue_in_and_plans_a_hard_transition_at_cue_out() {
    let (mut s, p, [a, _, _]) = three();
    apply(&mut s, Command::Play(p)).unwrap();
    let out = apply(&mut s, Command::SetNext(p, a)).unwrap();
    assert_eq!(preload(&out, p), Some(Some(a)), "{out:?}");
    assert_eq!(plan(&s, p), Some(HARD_END));
}

#[test]
fn o37_a_self_next_never_plans_a_segue() {
    let (mut s, p, [a, _, _]) = three();
    let t = s.track_for_entry(a).unwrap().id;
    // An automatic segue strictly inside the range would overlap the entry
    // with itself: the plan must stay the hard one at cue-out.
    s.library.get_mut(t).unwrap().segue_start_secs_auto = Some(170.0);
    apply(&mut s, Command::Play(p)).unwrap();
    apply(&mut s, Command::SetNext(p, a)).unwrap();
    assert_eq!(plan(&s, p), Some(HARD_END));
}

#[test]
fn o37_the_second_pass_is_an_ordinary_play() {
    let (mut s, p, [a, b, _]) = replaying();
    let out = started(&mut s, p, a);
    let player = s.player(p).unwrap();
    assert_eq!(player.current, Some(a), "it is current again");
    assert!(s.playlists.entry(a).unwrap().played_by.contains(&p), "marked played");
    assert_eq!(player.history, vec![a], "entered the history");
    assert_eq!((player.next, player.next_explicit), (Some(b), false), "the next is worked out as usual");
    assert_eq!(player.transport, Transport::Playing);
    assert_eq!(preload(&out, p), Some(Some(b)), "{out:?}");
    assert_eq!(plan(&s, p), Some(HARD_END), "and goes on with the following entry");
}

#[test]
fn o37_it_acts_once_the_third_pass_never_comes() {
    let (mut s, p, [a, b, _]) = replaying();
    started(&mut s, p, a);
    started(&mut s, p, b);
    assert_eq!(s.player(p).unwrap().current, Some(b));
}

#[test]
fn the_replay_of_the_last_entry_leaves_no_next_and_stops_after_the_second_pass() {
    let (mut s, p, [_, _, c]) = three();
    apply(&mut s, Command::SetNext(p, c)).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    apply(&mut s, Command::SetNext(p, c)).unwrap();
    started(&mut s, p, c);
    assert_eq!(s.player(p).unwrap().next, None);
    assert_eq!(plan(&s, p), Some(TransitionPlan::StopAt { at_secs: 180.0 }));
    on_event(&mut s, EngineEvent::ReachedEnd { player: p, entry: c });
    assert_eq!(s.player(p).unwrap().transport, Transport::Stopped);
}

#[test]
fn o37_a_repeating_entry_keeps_repeating_and_the_self_next_waits() {
    let (mut s, p, [a, b, _]) = three();
    apply(&mut s, Command::ToggleEntryRepeat(a)).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    apply(&mut s, Command::SetNext(p, a)).unwrap();
    started(&mut s, p, a);
    let player = s.player(p).unwrap();
    assert_eq!((player.next, player.next_explicit), (Some(a), true), "still waiting");
    assert!(player.history.is_empty(), "a repeat pass is not recorded");
    // Repeat off: the next boundary is the replay, then the player goes on.
    apply(&mut s, Command::ToggleEntryRepeat(a)).unwrap();
    started(&mut s, p, a);
    let player = s.player(p).unwrap();
    assert_eq!((player.next, player.history.clone()), (Some(b), vec![a]));
}

#[test]
fn o37_next_while_on_air_restarts_the_entry_with_a_crossfade() {
    let (mut s, p, [a, b, _]) = replaying();
    let out = apply(&mut s, Command::Play(p)).unwrap();
    assert!(
        out.iter().any(|x| matches!(x, EngineAction::Crossfade { player, request, .. }
            if *player == p && request.entry == a && request.from_secs == 0.0)),
        "{out:?}"
    );
    let player = s.player(p).unwrap();
    assert_eq!((player.current, player.next, player.history.clone()), (Some(a), Some(b), vec![a]));
}

#[test]
fn o37_paused_with_a_self_next_resumes_without_a_replay() {
    let (mut s, p, [a, _, _]) = replaying();
    apply(&mut s, Command::Pause(p)).unwrap();
    let out = apply(&mut s, Command::Play(p)).unwrap();
    assert!(out.iter().any(|x| matches!(x, EngineAction::Resume { .. })), "{out:?}");
    assert!(s.player(p).unwrap().history.is_empty());
    assert_eq!(s.player(p).unwrap().next, Some(a));
}

// Everything that already wins over the next entry still wins.

#[test]
fn o37_single_mode_stops_at_the_end_and_play_starts_the_entry_again() {
    let (mut s, p, [a, _, _]) = three();
    apply(&mut s, Command::SetMode(p, PlayMode::Single)).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    apply(&mut s, Command::SetNext(p, a)).unwrap();
    assert_eq!(plan(&s, p), Some(TransitionPlan::StopAt { at_secs: 180.0 }));
    on_event(&mut s, EngineEvent::ReachedEnd { player: p, entry: a });
    let player = s.player(p).unwrap();
    assert_eq!((player.transport, player.next), (Transport::Stopped, Some(a)));
    let out = apply(&mut s, Command::Play(p)).unwrap();
    assert!(
        out.iter().any(|x| matches!(x, EngineAction::StartCurrent { request, .. } if request.entry == a)),
        "{out:?}"
    );
}

#[test]
fn o37_stop_after_current_wins() {
    let (mut s, p, [a, _, _]) = replaying();
    apply(&mut s, Command::ToggleStopAfterCurrent(p)).unwrap();
    assert_eq!(plan(&s, p), Some(TransitionPlan::StopAt { at_secs: 180.0 }));
    on_event(&mut s, EngineEvent::ReachedEnd { player: p, entry: a });
    assert_eq!(s.player(p).unwrap().transport, Transport::Stopped);
}

#[test]
fn o37_an_entry_stop_after_wins() {
    let (mut s, p, [a, _, _]) = replaying();
    apply(&mut s, Command::ToggleEntryStopAfter(a)).unwrap();
    assert_eq!(plan(&s, p), Some(TransitionPlan::StopAt { at_secs: 180.0 }));
}

#[test]
fn o37_a_fade_stop_wins() {
    let (mut s, p, _) = replaying();
    apply(&mut s, Command::FadeStop(p)).unwrap();
    assert_eq!(plan(&s, p), None);
}
```

(Check the real field name of the automatic segue in `crates/fp-model/src/track.rs` — `segue_start_secs()` is the accessor — and set whatever the library exposes, as `crates/fp-model/tests/markers.rs` does; adjust `segue_start_secs_auto` to that name. If setting it is awkward, set a manual segue marker with `Command::SetMarker` instead.)

Update the tests that asserted the old refusal:
- `crates/fp-model/tests/transport.rs`: rename `rule2_double_click_sets_next_but_never_the_current_entry` to `rule2_set_next_also_accepts_the_current_entry` and assert `apply(&mut state, Command::SetNext(p, e[2])).is_ok()` and `next == Some(e[2])`.
- `crates/fp-model/tests/cue_window.rs`: `load_as_next_of_the_current_entry_is_refused` becomes `load_as_next_of_the_current_entry_is_accepted` (assert `Ok` and `next == Some(e[0])`).
- `crates/fp-remote/tests/api.rs`: `the_dry_run_turns_a_model_refusal_into_a_conflict` used `SetNext(current)`; use another refusal that still reaches the model (find one with `grep -n "EntryOnAir\|PlaylistOnAir" crates/fp-remote/src/api.rs`: removing the entry on air through the edit plan gives a 409 whose message contains "on air"), and keep the `409` and message assertions for that.

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p fp-model --test replay_next -q`. Expected: FAIL (`NextIsCurrent` is returned by `SetNext`); `cargo test -p fp-model --test transport -q` fails on the renamed rule.

- [ ] **Step 3: Implement**

`reducer.rs`, `set_next`: delete the `if state.players[i].current == Some(entry) { return Err(ModelError::NextIsCurrent); }` block. Update its doc comment: "O37: the entry on air is accepted; it plays once more."

`plan_for`: after the `if player.mode == PlayMode::Single || player.stop_after_current || player.next.is_none() { ... }` block and before the segue lookup add:

```rust
    // O37: the entry on air was set as next. It plays once more from its
    // cue-in, as a hard transition into itself (like a repeat, so no segue:
    // it would overlap the file with itself). A file that can no longer be
    // opened plays its pass out and stops.
    if player.next == Some(current) {
        return Some(if track.file_state.is_playable() {
            TransitionPlan::StartNextAt {
                at_secs: end,
                fade_current_until_secs: None,
            }
        } else {
            TransitionPlan::StopAt { at_secs: end }
        });
    }
```

`preload_target` keeps its code; add to its doc comment that with a self-next the `else` arm returns the current entry too.

`on_event`, `TransitionStarted`, first block: replace by

```rust
            if let Ok(i) = state.player_index(player)
                && state.players[i].current == Some(entry)
                && state.players[i].transport != Transport::Stopped
            {
                // A repeat pass (R26) keeps the entry as it is. Otherwise a
                // self-next (O37) is the replay: an ordinary play of the same
                // entry (played mark, history, next derived from the playlist).
                let replay = !repeating(state, &state.players[i])
                    && state.players[i].next == Some(entry);
                if replay && advance_to(state, i, entry, true).is_none() {
                    // The file cannot be read any more: the pass already
                    // started plays out; the next is the entry after it.
                    let following = state.playlists.next_playable_after(entry, &state.library);
                    state.players[i].next = following;
                    state.players[i].next_explicit = false;
                }
                let p = &mut state.players[i];
                p.preloaded = None;
                p.scheduled = None;
                p.transport = Transport::Playing;
            }
```

Update the comment above it (it says only "a repeating entry's next pass").

`error.rs`: delete `NextIsCurrent`. `ui/app.rs`: delete its arm. Both `main.ftl`: delete `error-next-is-current`. Compile everything: `cargo build --workspace --all-targets -q` and fix any other `NextIsCurrent` reference (`grep -rn NextIsCurrent crates docs/technical docs/user`).

Main spec: rule 2 (double-click / set next: "never the current entry") now says that Set as next accepts the entry on air and plays it once more; add a rule line after 27 (R26) describing the replay in the same style ("a self-next: `StartNextAt` hard at cue-out, then an ordinary play").

- [ ] **Step 4: Run the tests**

Run: `cargo test -p fp-model -q` then `cargo test -p fp-remote -q` then `cargo build --workspace --all-targets -q`. Expected: PASS.

- [ ] **Step 5: Gate and commit**

```bash
export PATH=$HOME/.cargo/bin:$PATH; cargo fmt --all --check && cargo clippy --workspace --all-targets -q -- -D warnings && cargo test --workspace -q
git add crates docs
git commit -m "feat(model): play the entry on air once more when it is set as next"
scripts/check-commits.sh origin/master
```

---

### Task 2: O37 model: the edges (failures, edits, restore, CUE)

**Files:**
- Modify: `crates/fp-model/src/reducer.rs` (`source_failed`, `preload_failed`), `crates/fp-model/src/session.rs` (restore keeps an explicit self-next), `crates/fp-app/src/ui/view.rs` (`cue_window_view`: `can_load_next`)
- Test: `crates/fp-model/tests/replay_next.rs` (append), `crates/fp-model/tests/restore.rs` or the same file for the restore test, `crates/fp-app/tests/view.rs`

**Interfaces:**
- Consumes: Task 1 (`replay_next.rs` helpers `three`, `plan`, `preload`, `started`, `replaying`, `HARD_END`).
- Produces: a failing self-next is replaced by the entry after it; `CueWindowView::can_load_next` is true for the entry on air unless it already is the explicit next.

- [ ] **Step 1: Write the failing tests** (append to `replay_next.rs`)

```rust
#[test]
fn a_self_next_whose_preload_fails_is_replaced_by_the_following_entry() {
    let (mut s, p, [a, b, _]) = replaying();
    on_event(&mut s, EngineEvent::PreloadFailed { player: p, entry: a });
    let player = s.player(p).unwrap();
    assert_eq!((player.next, player.next_explicit), (Some(b), false));
    assert_eq!(player.current, Some(a), "the pass on air is not touched");
}

#[test]
fn a_self_next_on_a_failing_source_continues_with_the_following_entry() {
    let (mut s, p, [a, b, _]) = replaying();
    let out = on_event(&mut s, EngineEvent::SourceFailed { player: p, entry: a });
    let player = s.player(p).unwrap();
    assert_eq!(player.current, Some(b), "Continuous goes on: {out:?}");
}

#[test]
fn plan_for_stops_at_the_end_for_a_self_next_on_an_unplayable_file() {
    let (mut s, p, [a, _, _]) = replaying();
    let t = s.track_for_entry(a).unwrap().id;
    s.library.get_mut(t).unwrap().file_state = fp_model::FileState::Unreadable;
    assert_eq!(plan(&s, p), Some(TransitionPlan::StopAt { at_secs: 180.0 }));
}

#[test]
fn moving_the_entry_keeps_the_self_next_and_the_following_entry_follows_the_new_position() {
    let (mut s, p, [a, b, c]) = replaying();
    let playlist = s.playlists.first_id().unwrap();
    // Move `a` to the end (look at `Command::MoveEntry` for its fields).
    apply(&mut s, Command::MoveEntry { playlist, entry: a, to: 2 }).unwrap();
    assert_eq!(s.player(p).unwrap().next, Some(a));
    started(&mut s, p, a);
    assert_eq!(s.player(p).unwrap().next, None, "nothing follows the last position");
    let _ = (b, c);
}

#[test]
fn removing_the_entry_after_the_current_one_does_not_touch_a_self_next() {
    let (mut s, p, [a, b, c]) = replaying();
    apply(&mut s, Command::RemoveEntries(vec![b])).unwrap(); // use the real removal command
    assert_eq!(s.player(p).unwrap().next, Some(a));
    started(&mut s, p, a);
    assert_eq!(s.player(p).unwrap().next, Some(c));
}

#[test]
fn previous_overrides_a_self_next() {
    let (mut s, p, [a, b, _]) = three();
    apply(&mut s, Command::Play(p)).unwrap();
    apply(&mut s, Command::Play(p)).unwrap(); // a -> b, history [a]
    apply(&mut s, Command::SetNext(p, b)).unwrap();
    apply(&mut s, Command::Previous(p)).unwrap();
    let player = s.player(p).unwrap();
    assert_eq!((player.current, player.next), (Some(a), Some(b)), "the entry left is next");
}

#[test]
fn stop_keeps_the_self_next_so_play_starts_the_entry_again() {
    let (mut s, p, [a, _, _]) = replaying();
    apply(&mut s, Command::Stop(p)).unwrap();
    assert_eq!(s.player(p).unwrap().next, Some(a));
    let out = apply(&mut s, Command::Play(p)).unwrap();
    assert!(
        out.iter().any(|x| matches!(x, EngineAction::StartCurrent { request, .. } if request.entry == a)),
        "{out:?}"
    );
}

#[test]
fn a_restored_self_next_is_kept_and_the_player_stays_paused() {
    let (s, p, [a, _, _]) = replaying();
    // Use the same save/restore pair `crates/fp-model/tests/restore.rs` uses
    // (a `Session` built from the state, applied to a fresh state).
    let restored = common::roundtrip(&s); // add this helper to tests/common/mod.rs from restore.rs
    let player = restored.player(p).unwrap();
    assert_eq!((player.current, player.next, player.next_explicit), (Some(a), Some(a), true));
    assert_eq!(player.transport, Transport::Paused);
}
```

(Where the commands' real names/fields differ — `MoveEntry`, the entry removal command, the session round trip — read `crates/fp-model/src/command.rs` and `crates/fp-model/tests/restore.rs`/`editing.rs` and use them; keep the assertions.)

In `crates/fp-app/tests/view.rs`, in the test with `"the current entry cannot be the next"` (near line 434): the last assertion becomes `assert!(cue_window_view(&s, p, None).unwrap().can_load_next, "the entry on air can be loaded as next (O37)")`, rename the message accordingly, then `apply(Command::CueToNext(p))` and assert `!can_load_next` ("already explicit").

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p fp-model --test replay_next -q` and `cargo test -p fp-app --test view -q`. Expected: the failure, source and restore tests FAIL; the move/remove/previous/stop tests may already pass (they pin behaviour).

- [ ] **Step 3: Implement**

`preload_failed`, `on_air` branch: after marking the file unreadable and clearing `preloaded`, add

```rust
        // A self-next (O37) on a file that cannot be read again: the entry
        // after it is the next.
        if state.players[i].next == Some(entry) {
            state.players[i].next = state.playlists.next_playable_after(entry, &state.library);
            state.players[i].next_explicit = false;
        }
```

`source_failed`: at its top, right after the file is marked unreadable and before `keep_going`/`advance`, do the same replacement for every player whose `current == Some(entry) && next == Some(entry)` (`advance` would otherwise try the same unreadable entry and stop the player). The existing loop below (`p.next == Some(entry)` → `successors.for_current(p.current)`) keeps handling the other players.

`session.rs`, `restore` of a player: replace

```rust
    let kept_next = s
        .next
        .filter(|e| state.playlists.entry(*e).is_some() && Some(*e) != current);
```
by
```rust
    // An explicit next may be the current entry (O37: it plays once more).
    let kept_next = s.next.filter(|e| {
        state.playlists.entry(*e).is_some() && (Some(*e) != current || s.next_explicit)
    });
```

`view.rs`, `cue_window_view`: `can_load_next: !(p.next == Some(cue.entry) && p.next_explicit)`.

- [ ] **Step 4: Run the tests**

Run: `cargo test -p fp-model -q && cargo test -p fp-app --test view -q`. Expected: PASS.

- [ ] **Step 5: Documentation**

`docs/user/players.md` (CUE window list): "**Load as next** makes the cued track the player's next ... It is dimmed when the track already is the next, or is the current track." becomes "It is dimmed when the track already is the next. If the cued track is the one on air, it plays once more when the current pass ends."

- [ ] **Step 6: Gate and commit**

```bash
export PATH=$HOME/.cargo/bin:$PATH; cargo fmt --all --check && cargo clippy --workspace --all-targets -q -- -D warnings && cargo test --workspace -q
git add crates docs
git commit -m "fix(model): keep a self-next safe across failures, edits and restarts"
scripts/check-commits.sh origin/master
```

---

### Task 3: O38 model: stop after current in Single mode

**Files:**
- Modify: `crates/fp-model/src/reducer.rs` (`repeat_entry` new, `repeating`, `Command::SetMode`, `Command::ToggleStopAfterCurrent`, `reconcile`), `crates/fp-model/src/availability.rs`, `crates/fp-model/src/session.rs` (restore of the flag), `crates/fp-model/src/error.rs` (message of `StopAfterInSingle`), `crates/fp-app/src/ui/app.rs` and both locales (`error-stop-after-single` wording)
- Test: `crates/fp-model/tests/single_stop_after.rs` (new); update the tests that asserted a flat refusal in Single mode (`grep -rn "StopAfterInSingle\|stop_after_current" crates/*/tests`, expect `transitions.rs`, `set_values.rs`, `availability.rs`, `crates/fp-remote/tests/api.rs`)
- Docs: main spec rules 27/28 notes, `docs/user/players.md` (Stop after row and SINGLE paragraph)

**Interfaces:**
- Produces: `pub(crate) fn repeat_entry(state: &AppState, player: &PlayerState) -> bool` in `reducer.rs` (R26 conditions without `!stop_after_current`); `availability(..).stop_after_current` is `Continuous || repeat_entry`; the invariant `Single && stop_after_current ⇒ repeat_entry` after every `apply` / `on_event`.

- [ ] **Step 1: Write the failing tests**

Create `crates/fp-model/tests/single_stop_after.rs`:

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback 2 spec O38: stop after current in Single mode, while the current
//! entry repeats.

mod common;

use common::{entries, fixture, p0};
use fp_model::{
    AppState, Command, EngineEvent, EntryId, ModelError, PlayMode, PlayerId, TransitionPlan,
    Transport, apply, availability, command_available, on_event, plan_for,
};

fn single_repeating() -> (AppState, PlayerId, [EntryId; 2]) {
    let mut s = fixture(2);
    let (e, p) = (entries(&s), p0(&s));
    apply(&mut s, Command::SetMode(p, PlayMode::Single)).unwrap();
    apply(&mut s, Command::ToggleEntryRepeat(e[0])).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    (s, p, [e[0], e[1]])
}

fn plan(s: &AppState, p: PlayerId) -> Option<TransitionPlan> {
    plan_for(s, s.player(p).unwrap())
}

#[test]
fn o38_the_control_is_available_in_single_mode_while_the_entry_repeats() {
    let (s, p, _) = single_repeating();
    assert!(availability(&s, p).stop_after_current);
    assert!(command_available(&s, &Command::ToggleStopAfterCurrent(p)));
}

#[test]
fn o38_it_is_refused_in_single_mode_without_a_repeating_entry() {
    let mut s = fixture(2);
    let p = p0(&s);
    apply(&mut s, Command::SetMode(p, PlayMode::Single)).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    assert!(!availability(&s, p).stop_after_current);
    assert_eq!(
        apply(&mut s, Command::ToggleStopAfterCurrent(p)),
        Err(ModelError::StopAfterInSingle)
    );
}

#[test]
fn o38_it_ends_the_repeat_at_the_cue_out_of_the_pass_that_is_playing() {
    let (mut s, p, [a, _]) = single_repeating();
    assert!(matches!(plan(&s, p), Some(TransitionPlan::StartNextAt { .. })), "repeating");
    apply(&mut s, Command::ToggleStopAfterCurrent(p)).unwrap();
    assert_eq!(plan(&s, p), Some(TransitionPlan::StopAt { at_secs: 180.0 }));
    on_event(&mut s, EngineEvent::ReachedEnd { player: p, entry: a });
    let player = s.player(p).unwrap();
    assert_eq!(player.transport, Transport::Stopped);
    assert!(!player.stop_after_current, "the flag ends with the stop");
}

#[test]
fn o38_the_notice_is_the_same_as_in_continuous_mode() {
    let (mut s, p, _) = single_repeating();
    assert_eq!(fp_model::entry_notice(&s, p), Some(fp_model::EntryNotice::Repeats));
    apply(&mut s, Command::ToggleStopAfterCurrent(p)).unwrap();
    assert_eq!(fp_model::entry_notice(&s, p), None, "the stop-after badge says it");
    assert!(s.player(p).unwrap().stop_after_current);
}

#[test]
fn o38_switching_to_single_keeps_the_flag_while_the_entry_repeats() {
    let mut s = fixture(2);
    let (e, p) = (entries(&s), p0(&s));
    apply(&mut s, Command::ToggleEntryRepeat(e[0])).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    apply(&mut s, Command::ToggleStopAfterCurrent(p)).unwrap();
    apply(&mut s, Command::SetMode(p, PlayMode::Single)).unwrap();
    assert!(s.player(p).unwrap().stop_after_current);
}

#[test]
fn switching_to_single_clears_the_flag_without_a_repeating_entry() {
    let mut s = fixture(2);
    let p = p0(&s);
    apply(&mut s, Command::Play(p)).unwrap();
    apply(&mut s, Command::ToggleStopAfterCurrent(p)).unwrap();
    apply(&mut s, Command::SetMode(p, PlayMode::Single)).unwrap();
    assert!(!s.player(p).unwrap().stop_after_current);
}

#[test]
fn the_single_mode_flag_is_dropped_when_the_repeat_stops_applying() {
    let (mut s, p, [a, _]) = single_repeating();
    apply(&mut s, Command::ToggleStopAfterCurrent(p)).unwrap();
    apply(&mut s, Command::ToggleEntryRepeat(a)).unwrap();
    assert!(!s.player(p).unwrap().stop_after_current, "repeat off");
    apply(&mut s, Command::ToggleEntryRepeat(a)).unwrap();
    apply(&mut s, Command::ToggleStopAfterCurrent(p)).unwrap();
    apply(&mut s, Command::ToggleEntryStopAfter(a)).unwrap();
    assert!(!s.player(p).unwrap().stop_after_current, "entry stop-after wins");
}

#[test]
fn the_flag_ends_with_stop_and_does_not_reach_the_next_play() {
    let (mut s, p, _) = single_repeating();
    apply(&mut s, Command::ToggleStopAfterCurrent(p)).unwrap();
    apply(&mut s, Command::Stop(p)).unwrap();
    assert!(!s.player(p).unwrap().stop_after_current);
}

#[test]
fn next_into_a_plain_entry_drops_the_single_mode_flag() {
    let (mut s, p, _) = single_repeating();
    apply(&mut s, Command::ToggleStopAfterCurrent(p)).unwrap();
    apply(&mut s, Command::Play(p)).unwrap(); // Next: the repeating entry is left
    assert!(!s.player(p).unwrap().stop_after_current);
}

#[test]
fn set_stop_after_current_follows_the_same_guard() {
    let (mut s, p, _) = single_repeating();
    apply(&mut s, Command::SetStopAfterCurrent(p, true)).unwrap();
    assert!(s.player(p).unwrap().stop_after_current);
}

#[test]
fn restore_keeps_the_single_mode_flag_only_for_a_repeating_entry() {
    let (mut s, p, _) = single_repeating();
    apply(&mut s, Command::ToggleStopAfterCurrent(p)).unwrap();
    let restored = common::roundtrip(&s); // helper added in Task 2
    let player = restored.player(p).unwrap();
    assert!(player.stop_after_current);
    assert_eq!(player.transport, Transport::Paused, "nothing goes on air by itself");
    // Same state, but the repeat mark was removed from the file on disk:
    let mut s2 = s.clone();
    let a = s2.player(p).unwrap().current.unwrap();
    s2.playlists.entry_mut(a).unwrap().repeat = false; // use the real accessor
    assert!(!common::roundtrip(&s2).player(p).unwrap().stop_after_current);
}
```

(`entry_mut` and `roundtrip` are the real accessors/helpers: read `crates/fp-model/src/playlist.rs` and `tests/restore.rs`. If `AppState` is not `Clone`, rebuild the state in the test instead of cloning.)

Update old tests: in whichever existing test calls `ToggleStopAfterCurrent` in Single mode on a non-repeating entry, nothing changes (still refused). `SetMode(Single)` clearing the flag: unchanged for a non-repeating entry. The remote test `stop_after_current_in_single_mode_is_a_conflict` stays valid (non-repeating).

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p fp-model --test single_stop_after -q`. Expected: FAIL (compile error for `repeat_entry` is not involved: tests use public API; the failures are `StopAfterInSingle` being returned and the flag being cleared).

- [ ] **Step 3: Implement**

`reducer.rs`: split `repeating`:

```rust
/// R26 without the stop-after-current test: the current entry would repeat
/// (O38 uses it to allow stop-after-current in Single mode).
pub(crate) fn repeat_entry(state: &AppState, player: &PlayerState) -> bool {
    player.transport != Transport::Stopped
        && !player.fade_stop_pending
        && player
            .current
            .and_then(|c| state.playlists.entry(c))
            .is_some_and(|e| e.repeat && !e.stop_after)
        // A file that can no longer be opened plays its pass out, once.
        && player
            .current
            .and_then(|c| state.track_for_entry(c))
            .is_some_and(|t| t.file_state.is_playable())
}

/// R26: the player's current entry repeats (and R27, stop-after-current or
/// a fade stop do not end it first).
fn repeating(state: &AppState, player: &PlayerState) -> bool {
    !player.stop_after_current && repeat_entry(state, player)
}
```

`Command::SetMode`:

```rust
            let i = state.player_index(id)?;
            state.players[i].mode = mode;
            // O38: in Single mode the flag only makes sense while the current
            // entry repeats.
            if mode == PlayMode::Single && !repeat_entry(state, &state.players[i]) {
                state.players[i].stop_after_current = false;
            }
```

`Command::ToggleStopAfterCurrent`:

```rust
            let i = state.player_index(id)?;
            if state.players[i].mode == PlayMode::Single && !repeat_entry(state, &state.players[i]) {
                return Err(ModelError::StopAfterInSingle);
            }
            let player = &mut state.players[i];
            player.stop_after_current = !player.stop_after_current;
```

`reconcile` (first lines, before the preload computation):

```rust
    // O38: in Single mode the flag lives only while the current entry
    // repeats (it may have stopped applying: repeat off, a stop-after mark,
    // Next into another entry, a file that cannot be read).
    let stale: Vec<usize> = state
        .players
        .iter()
        .enumerate()
        .filter(|(_, p)| {
            p.mode == PlayMode::Single && p.stop_after_current && !repeat_entry(state, p)
        })
        .map(|(i, _)| i)
        .collect();
    for i in stale {
        state.players[i].stop_after_current = false;
    }
```

Check that `apply` runs `reconcile` at its end (line ~284) for every command; if a command path returns early without it (for example the `volume.is_nan()` early return), that is fine since the invariant cannot have changed.

`availability.rs`: `stop_after_current: p.mode == PlayMode::Continuous || crate::reducer::repeat_entry(state, p)` (update the module so `repeat_entry` is reachable: `pub(crate)`). Update the field's doc.

`session.rs` restore: after `player.transport = ...`, replace the flag line by

```rust
    player.stop_after_current = s.stop_after_current
        && (s.mode == PlayMode::Continuous || crate::reducer::repeat_entry(state, &player));
```

`error.rs`: message of `StopAfterInSingle` becomes "stop after current is only available in single mode while the current entry repeats"; `ui/app.rs` keeps its arm; locale `error-stop-after-single`: en-US "Stop after the current track is only available in SINGLE mode while the track repeats."; es-ES "Stop al final solo está disponible en modo SINGLE mientras la pista se repite."

- [ ] **Step 4: Run the tests**

Run: `cargo test -p fp-model -q && cargo test -p fp-app --test i18n -q`. Expected: PASS.

- [ ] **Step 5: Documentation**

`docs/user/players.md`: the **Stop after** row: "Stop when the current track ends (continuous mode only), once." becomes "... once. In SINGLE mode it is available only while the current track repeats: it ends the repeat when the pass that is playing ends." The SINGLE paragraph (`*Stop after* is not available in ...`) gets the same exception. Main spec: add to rule 27/28 notes that Single mode keeps stop-after-current while the entry repeats.

- [ ] **Step 6: Gate and commit**

```bash
export PATH=$HOME/.cargo/bin:$PATH; cargo fmt --all --check && cargo clippy --workspace --all-targets -q -- -D warnings && cargo test --workspace -q
git add crates docs
git commit -m "feat(model): allow stop after current in Single mode while the entry repeats"
scripts/check-commits.sh origin/master
```

---

### Task 4: Engine and conductor, end to end

**Files:**
- Test: `crates/fp-engine/tests/conductor.rs` (append; helpers `model`, `settle`, `offline_conductor`, `entries`, `BLOCK`, `TRACK_FRAMES` already there)
- Modify: `crates/fp-engine/src/*` only if a test shows a real defect (none expected: the engine is entry-agnostic and the repeat already runs this path; `engine.rs` around line 1587 says "even when it is the same entry (a repeat)"). Docs: `docs/technical/audio-engine.md`

**Interfaces:**
- Consumes: Tasks 1 to 3 (model rules). Produces: end-to-end evidence that the model's `Preload` + `Schedule` pair drives the Offline engine to a gapless second pass and that the pass after it moves on.

- [ ] **Step 1: Write the failing tests**

Append to `crates/fp-engine/tests/conductor.rs` (copy the `run` closure of `a_repeating_entry_restarts_gaplessly_until_repeat_is_turned_off`; the test tracks carry sample values `n * 100_000 + frame`, as that test relies on):

```rust
#[test]
fn the_entry_on_air_set_as_next_plays_once_more_gaplessly_and_then_moves_on() {
    let (mut conductor, handle, device, mut now) = offline_conductor(model(1, 3));
    let e = entries(conductor.state());
    let p = conductor.state().players[0].id;
    assert!(handle.send(Command::Play(p)));
    settle(&mut conductor, now);
    assert!(handle.send(Command::SetNext(p, e[0])));
    let mut heard: Vec<f32> = Vec::new();
    for _ in 0..450 {
        conductor.tick(now);
        heard.extend(device.render(BLOCK).unwrap().chunks(2).map(|f| f[0]));
        now += Duration::from_millis(10);
        std::thread::yield_now();
    }
    // Pass 1 and pass 2 of track 1 are back to back (same boundary rule as
    // the repeat test), then track 2 follows, and track 1 never returns.
    let first = heard.iter().position(|v| *v == 100_000.0).unwrap();
    let len = TRACK_FRAMES as usize;
    assert_eq!(heard[first + len], 100_000.0, "the second pass starts on the boundary");
    let after = first + 2 * len;
    assert!(heard.get(after + 100).is_some_and(|v| (*v as u64) / 100_000 == 2), "then track 2");
    assert!(!heard[after..].iter().any(|v| (*v as u64) / 100_000 == 1), "it acted once");
    let model = handle.model.load();
    let player = model.player(p).unwrap();
    assert_eq!(player.current, Some(e[1]));
    assert!(model.playlists.entry(e[0]).unwrap().is_played_by(p));
    assert_eq!(player.history, vec![e[0]]);
}

#[test]
fn in_single_mode_the_self_next_stops_the_player_and_play_starts_the_entry_again() {
    let (mut conductor, handle, device, mut now) = offline_conductor(model(1, 3));
    let e = entries(conductor.state());
    let p = conductor.state().players[0].id;
    assert!(handle.send(Command::SetMode(p, PlayMode::Single)));
    assert!(handle.send(Command::Play(p)));
    settle(&mut conductor, now);
    assert!(handle.send(Command::SetNext(p, e[0])));
    for _ in 0..250 {
        conductor.tick(now);
        device.render(BLOCK).unwrap();
        now += Duration::from_millis(10);
        std::thread::yield_now();
    }
    let player = handle.model.load().player(p).unwrap().clone();
    assert_eq!((player.transport, player.next), (Transport::Stopped, Some(e[0])));
    assert!(handle.send(Command::Play(p)));
    settle(&mut conductor, now);
    assert_eq!(handle.model.load().player(p).unwrap().current, Some(e[0]));
}

#[test]
fn in_single_mode_stop_after_current_ends_a_repeat_at_the_end_of_the_pass() {
    let (mut conductor, handle, device, mut now) = offline_conductor(model(1, 3));
    let e = entries(conductor.state());
    let p = conductor.state().players[0].id;
    assert!(handle.send(Command::SetMode(p, PlayMode::Single)));
    assert!(handle.send(Command::ToggleEntryRepeat(e[0])));
    assert!(handle.send(Command::Play(p)));
    settle(&mut conductor, now);
    assert!(handle.send(Command::ToggleStopAfterCurrent(p)));
    for _ in 0..250 {
        conductor.tick(now);
        device.render(BLOCK).unwrap();
        now += Duration::from_millis(10);
        std::thread::yield_now();
    }
    let player = handle.model.load().player(p).unwrap().clone();
    assert_eq!(player.transport, Transport::Stopped);
    assert!(!player.stop_after_current);
}

#[test]
fn next_pressed_just_after_a_replay_boundary_keeps_the_model_on_the_next_track() {
    // Same shape as `next_pressed_just_after_a_repeat_boundary_...`, with a
    // self-next instead of repeat: the Play that arrives in the tick of the
    // restart wins, and the entry is not marked played twice or recorded twice.
    let (mut conductor, handle, device, mut now) = offline_conductor(model(1, 3));
    let e = entries(conductor.state());
    let p = conductor.state().players[0].id;
    assert!(handle.send(Command::Play(p)));
    settle(&mut conductor, now);
    assert!(handle.send(Command::SetNext(p, e[0])));
    let mut heard = 0usize;
    while heard + BLOCK < TRACK_FRAMES as usize - BLOCK {
        conductor.tick(now);
        device.render(BLOCK).unwrap();
        heard += BLOCK;
        now += Duration::from_millis(10);
        std::thread::yield_now();
    }
    conductor.tick(now);
    for _ in 0..3 {
        device.render(BLOCK).unwrap();
    }
    assert!(handle.send(Command::Play(p)));
    now += Duration::from_millis(30);
    for _ in 0..20 {
        conductor.tick(now);
        device.render(BLOCK).unwrap();
        now += Duration::from_millis(10);
        std::thread::yield_now();
    }
    let model = handle.model.load();
    let player = model.player(p).unwrap();
    assert!(matches!(player.current, Some(c) if c == e[0] || c == e[1]), "{player:?}");
    assert_eq!(player.history.iter().filter(|h| **h == e[0]).count(), 1, "{player:?}");
}
```

(Add `PlayMode` and `Transport` to the imports as needed; they are already imported at the top. If the last test's exact end state differs, assert only what the model guarantees: no panic, a consistent `current`, and at most one history record for the entry.)

- [ ] **Step 2: Run them**

Run: `cargo test -p fp-engine --test conductor replay -q` (and the three others by name). Expected: they pass straight away if the model tasks are right (the engine is unchanged); if one fails, the failure is a real defect: apply `superpowers:systematic-debugging`, fix it test-first in `engine.rs` / `conductor.rs`, and record it as `Ruling:` in the PR notes.

- [ ] **Step 3: Documentation**

`docs/technical/audio-engine.md`: in the `Preload` row and the paragraph near line 227, add that the same pair serves O37: "A self-next (the entry on air set as next) is preloaded and restarted in the same way, once; the model tells the pass apart from a repeat pass in `on_event`." No engine code changed.

- [ ] **Step 4: Gate and commit**

```bash
export PATH=$HOME/.cargo/bin:$PATH; cargo fmt --all --check && cargo clippy --workspace --all-targets -q -- -D warnings && cargo test --workspace -q
git add crates docs
git commit -m "test(engine): replay of the entry on air and stop after current in Single mode"
scripts/check-commits.sh origin/master
```

---

### Task 5: UI, the playing row marked as next, Set as next, the Single-mode control

**Files:**
- Modify: `crates/fp-app/src/ui/view.rs` (`row_is_next`), `crates/fp-app/src/ui/table.rs` (number cell of the playing row, context menu), `crates/fp-app/src/ui/player.rs` (`sa_tip`), both locales (`tip-stop-after-single`)
- Test: `crates/fp-app/tests/view.rs` (pure), `crates/fp-app/tests/next_row_ui.rs` (new, egui_kittest)
- Docs: `docs/user/playlists.md`, `docs/user/players.md`, `docs/technical/ui.md` (if it lists row states)

**Interfaces:**
- Consumes: Tasks 1 to 3. Produces: `fp_app::ui::view::row_is_next(state: &AppState, player: PlayerId, entry: EntryId) -> bool` (`player.next == Some(entry)`, whatever the row's `RowStatus`).

- [ ] **Step 1: Write the failing tests**

`crates/fp-app/tests/view.rs`:

```rust
#[test]
fn the_playing_row_is_also_marked_as_next_when_it_is_the_explicit_next() {
    let (mut s, e, p) = state(3);
    apply(&mut s, Command::Play(p)).unwrap();
    let playing = e[0];
    assert!(!row_is_next(&s, p, playing));
    apply(&mut s, Command::SetNext(p, playing)).unwrap();
    assert_eq!(row_status(&s, p, &entry(&s, playing)), RowStatus::Current, "still on air");
    assert!(row_is_next(&s, p, playing));
    assert!(!row_is_next(&s, p, e[1]));
}
```

Create `crates/fp-app/tests/next_row_ui.rs` (header and imports like `entry_notice_ui.rs`; look at `table_icons.rs` for how a row is found and at `main_screen.rs` around line 42 and 1003 for how the context menu "Set as next" is opened and what `sent(&fake)` is):

```rust
// 1. the playing row shows the next arrow only when it is the next
#[test]
fn the_playing_row_shows_the_next_arrow_once_it_is_set_as_next() {
    let mut s = state(1, 3);
    let p = s.players[0].id;
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    let (h, _) = harness(s.clone());
    assert_eq!(arrows(&h), 1, "only the derived next row has it");
    let current = s.players[0].current.unwrap();
    fp_model::apply(&mut s, Command::SetNext(p, current)).unwrap();
    let (h, _) = harness(s);
    assert_eq!(arrows(&h), 1, "the playing row has it and the next row has lost it");
}
// 2. Set as next is enabled on the playing row; Play now is not
// 3. clicking it sends SetNext(player, current)
// 4. double-click on the playing row sends nothing
// 5. Single mode, repeating current: the stop-after button is enabled and clicking it sends ToggleStopAfterCurrent
// 6. Single mode, plain current: the button is disabled and its tooltip is the "Not available" text
```

where `arrows(h)` counts `h.query_all_by_label_contains(egui_phosphor::regular::ARROW_BEND_DOWN_RIGHT)`; in test 1 the table is the only place the glyph appears (check the player column does not draw it; if it does, count within the table's rect). Write tests 2 to 6 in full following the existing patterns (right-click is done as in `main_screen.rs` around line 990, the button is found by its tooltip label `Stop after the current track`, disabled state through `node.accesskit_node().is_disabled()` as `main_screen.rs` does for other buttons; read it and copy). Every test uses `harness` / `harness_sized` (`with_step_dt(0.02)`).

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p fp-app --test view --test next_row_ui -q`. Expected: FAIL (`row_is_next` missing; the menu item is disabled on the playing row; the Single-mode button is disabled and has the old tooltip logic).

- [ ] **Step 3: Implement**

`view.rs`:

```rust
/// O37: the row of `entry` is the player's next, whatever else it is: the
/// entry on air can be its own next (it plays once more).
pub fn row_is_next(state: &AppState, player: PlayerId, entry: EntryId) -> bool {
    state.player(player).is_ok_and(|p| p.next == Some(entry))
}
```

(`RowStatus` keeps its precedence: `Current` first; `RowStatus::Next` is unchanged for other rows.)

`table.rs`, number cell, `RowStatus::Current` arm: after the speaker/pause glyph label is added, when `view::row_is_next(scene.state, player, entry.id)` also add a second small label with `icon::ARROW_BEND_DOWN_RIGHT` in the regular family and `theme::NEUTRAL_100` (the speaker glyph uses the fill family, so two labels, not one string), with a hover tip from a new Fluent key `tip-next-again` ("Plays once more when this pass ends" / es-ES "Se vuelve a reproducir al acabar esta pasada"). Keep the row fill `ON_AIR_ROW`. Check the column is wide enough for both glyphs at the default width (`table_layout.rs` tests guard the Number column; widen the minimum only if a test fails, and say so in the commit body).

`table.rs`, context menu: the "menu-set-next" item's enabled flag becomes `true` (remove `!own_current`); keep `!own_current && !fading` on "menu-play-now". Leave the `response.double_clicked() && status != RowStatus::Current` rule as is, and add a comment: "O37: a double-click on the playing row does nothing; the menu item is the deliberate way to set it as next."

`player.rs`: `sa_tip` uses the "not available" text only when the control is unavailable: `if pv.mode == PlayMode::Single && !available.stop_after_current { t.tr("tip-stop-after-single") } else { t.tr("tip-stop-after") }` (move the `available` binding above if needed). Locale `tip-stop-after-single`: en-US "Not available in SINGLE mode unless the track repeats: every track already stops at its end"; es-ES "No disponible en modo SINGLE si la pista no se repite: cada pista ya se para al acabar". Add `tip-next-again` to both locales.

- [ ] **Step 4: Run the tests**

Run: `cargo test -p fp-app -q`. Expected: PASS (including `i18n.rs`, `table_layout.rs`, `main_screen.rs`).

- [ ] **Step 5: Documentation**

- `docs/user/playlists.md`: row-colour table: the red row "can also show the green arrow: the track on air is also the next, so it plays once more"; **Double-click** line: "(not the one on air)" stays; context menu: "Set as next | Same as double-click, and also available on the track on air: it then plays once more, from its start, when the current pass ends (mixing like Repeat, no gap), then the player goes on. It acts once. Stop after, SINGLE mode and a Stop after mark still end the player first. While a CUE is running it moves to the new next".
- `docs/user/players.md`: Stop-after row and SINGLE paragraph already edited in Task 3; add one sentence under **Play / Next** on a self-next ("with the track on air as next, Play restarts that track with the usual fade"). `docs/technical/ui.md`: one line for `row_is_next`.

- [ ] **Step 6: Gate and commit**

```bash
export PATH=$HOME/.cargo/bin:$PATH; cargo fmt --all --check && cargo clippy --workspace --all-targets -q -- -D warnings && cargo test --workspace -q
git add crates docs
git commit -m "feat(ui): mark the playing row as next and allow Set as next on it"
scripts/check-commits.sh origin/master
```

---

### Task 6: Remote API and MIDI, then spec, roadmap and a last check

**Files:**
- Test: `crates/fp-remote/tests/api.rs`, `crates/fp-remote/tests/http.rs` (one end-to-end oneshot), a router test in `crates/fp-control/src/router.rs`'s test module or `crates/fp-control/tests/router.rs` (whichever holds the existing `command_available` filtering test; find with `grep -rn "command_available\|StopAfter" crates/fp-control`)
- Modify (only if a test fails): `crates/fp-remote/src/api.rs` (none expected: `O::SetNext` never checked the current entry, `SetStopAfterCurrent` goes through `available`, which Task 3 changed)
- Docs: `docs/technical/remote-api.md`, `docs/user/remote-control.md`, `docs/user/midi.md`; `docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md` (§14 "As built", status line), `docs/superpowers/plans/2026-10-01-operator-feedback-2-roadmap.md` (row 13 done, status line), `CLAUDE.md` only if a command or layout fact changed (none expected)

**Interfaces:**
- Consumes: Tasks 1 to 5. Produces: nothing for later tasks.

- [ ] **Step 1: Write the tests**

`crates/fp-remote/tests/api.rs`:

```rust
#[test]
fn setting_the_entry_on_air_as_next_is_accepted() {
    let mut s = demo_state();
    let p = s.players[0].id;
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    let current = s.players[0].current.unwrap();
    assert_eq!(plan(&s, O::SetNext(p, current)).unwrap(), vec![Command::SetNext(p, current)]);
}

#[test]
fn stop_after_current_in_single_mode_is_accepted_while_the_entry_repeats() {
    let mut s = demo_state();
    let p = s.players[0].id;
    fp_model::apply(&mut s, Command::SetMode(p, PlayMode::Single)).unwrap();
    let first = s.playlists.iter().next().unwrap().entries[0].id;
    fp_model::apply(&mut s, Command::ToggleEntryRepeat(first)).unwrap();
    fp_model::apply(&mut s, Command::SetNext(p, first)).unwrap();
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(
        plan(&s, O::SetStopAfterCurrent(p, true)).unwrap(),
        vec![Command::SetStopAfterCurrent(p, true)]
    );
}
```

The existing `stop_after_current_in_single_mode_is_a_conflict` stays (non-repeating entry: `409`). In `tests/http.rs`, add one oneshot: `PUT /api/v1/players/{p}/next` with the entry on air returns `202` and the `FakeControl` recorded `Command::SetNext(p, current)`; and `PUT .../stop-after-current` `{"on":true}` in Single mode with a plain current entry returns `409`. Copy the request-building helpers from the neighbouring tests (`http.rs` lines near 170).

MIDI: in the `fp-control` router tests add (next to the existing availability test): a binding for stop-after-current produces no command in Single mode with a plain current entry, and produces `Command::ToggleStopAfterCurrent(p)` in Single mode while the current entry repeats; a "set next" style binding does not exist on MIDI (check `MidiAction`; if none sets a next, say so in the test comment and cover only stop-after).

- [ ] **Step 2: Run them**

Run: `cargo test -p fp-remote -q && cargo test -p fp-control -q`. Expected: PASS without source changes. If one fails, fix it test-first in `crates/fp-remote/src/api.rs` or `crates/fp-control/src/router.rs`.

- [ ] **Step 3: Documentation**

- `docs/technical/remote-api.md`: row `PUT /players/{id}/next`: "Choose the next entry. The entry on air is accepted: it plays once more from its cue-in when the current pass ends". Row `PUT /players/{id}/stop-after-current`: "`409` in single mode unless the current entry repeats". Check the `409` explanation paragraph has no mention of the removed `NextIsCurrent`.
- `docs/user/remote-control.md` line about choosing the next entry: add "(the entry on air too: it plays once more)".
- `docs/user/midi.md`: if the stop-after row says "continuous mode only", add the repeat exception.
- Feedback 2 spec: status line "Plans 1 to 9, 12 and 13 are built"; at the end of §14 add

```markdown
- **As built.**
  - **O37.** `set_next` accepts the entry on air; the self-next is `next == current` with `next_explicit` (a derived next never wraps, so it is always an operator choice). `plan_for` gives `StartNextAt` at cue-out, hard, never a segue (`StopAt` on an unplayable file); `preload_target` is unchanged. `on_event(TransitionStarted)` tells a repeat pass (`repeating`) from the replay (`next == entry`), and the replay runs `advance_to`, the ordinary play bookkeeping. `NextIsCurrent` and its message are gone. Failing preloads and sources replace a self-next by the following entry; the session restore keeps an explicit self-next. The engine is unchanged. UI: the playing row also shows the next arrow, the menu item is enabled on it, double-click on it still does nothing, the CUE window's "Load as next" accepts the entry on air.
  - **O38.** `repeat_entry` (R26 without the flag test) guards `ToggleStopAfterCurrent`, `SetMode`, `availability` and the restore; `reconcile` drops a Single-mode flag as soon as the entry stops repeating. The remote API and MIDI follow `availability`; no code of their own.
```

- Roadmap: row 13 status `done`; add a note under "Notes for the later plans" only if a later plan is affected (none expected).
- Last sweep: `grep -rn "NextIsCurrent\|next-is-current\|never the current" crates docs README.md CLAUDE.md` must show only historical plans/specs of earlier plans (leave those); `grep -rni "continuous mode only" docs/user docs/technical README.md` has no stale claim about stop-after-current. README feature list: add nothing unless it states the old rule.

- [ ] **Step 4: Gate and commit**

```bash
export PATH=$HOME/.cargo/bin:$PATH; cargo fmt --all --check && cargo clippy --workspace --all-targets -q -- -D warnings && cargo test --workspace -q
git add crates docs
git commit -m "docs: plan 13 as built, remote and MIDI follow the player rules"
scripts/check-commits.sh origin/master
```

---

## Self-Review

- **Spec coverage.** O37: acceptance and refusal removal (Task 1), same hard transition as a repeat (Task 1 plan/preload tests, Task 4 gapless audio), acts once, second pass ordinary: current again, played, history, next as usual (Task 1), the four things that still win and Single's Play (Task 1), the row marking and the menu action (Task 5), remote and MIDI (Task 6). O38: control available in Single while repeating, ends the repeat at the cue-out of the pass, same notice (Task 3, Task 4), flag kept when switching to Single (Task 3), refused otherwise with `StopAfterInSingle` and the remote `409` (Task 3, Task 6). §15: English, locales both (Tasks 1, 3, 5), no `Config` field (none needed), real-time untouched, behaviour in `fp-model`, UI never blocks (no new I/O), bad data (Task 2 failure cases), nothing on air by itself (restore tests keep Paused).
- **Placeholders.** The test code names a few commands and accessors to look up (`MoveEntry` fields, the removal command, `roundtrip`, `entry_mut`, the segue field): each is flagged with where to read the real one and the assertion to keep. No step says "TBD".
- **Type consistency.** `repeat_entry` (Task 3) is the only new function other tasks name; it is produced and consumed in Task 3 (`availability`, `session`, `reconcile`, `ToggleStopAfterCurrent`, `SetMode`). `row_is_next` is produced and consumed in Task 5. Helpers `three`, `plan`, `preload`, `started`, `replaying`, `HARD_END` live in `replay_next.rs` (Task 1) and are reused by Task 2; `roundtrip` is added to `tests/common/mod.rs` in Task 2 and reused in Task 3.
- **Review Focus.** Each of the five lines has its test in the task named.
