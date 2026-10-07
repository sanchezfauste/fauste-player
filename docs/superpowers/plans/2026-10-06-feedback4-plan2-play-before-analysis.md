# Play Before Analysis, Start Point, CUE Follow (Feedback 4, Plan 2) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A track played before its analysis has its length from the file header, so it has a countdown, a position and click-to-seek (Q1). A click on a stopped player's waveform sets where Play starts the next track (Q8). A running CUE follows a click or a double-click on a row of its own player's playlist (Q6).

**Architecture:**
- **Q6 comes first and starts with a test.** Nobody knows the cause yet. Task 1 writes a kittest matrix that reproduces it with real pointer events. Task 2 finds the cause with `superpowers:systematic-debugging` and fixes it with that matrix as the gate.
- **Q1, layer by layer.**
  - `fp-decode` gets `FileDecoder::duration_hint_secs`: the length the container declares, with no length for an MPEG stream that only has an estimate.
  - `fp-model` gets `Command::SetDuration` and `Track::needs_header_duration`, both pure and tested.
  - `fp-app` gets a small `HeaderReader` thread (`src/header.rs`, built like the file probe), which `Services::header_pass` drives, tracks on a player first.
  - The view and the waveform already use `duration_secs`. Only the time row learns to show "—" for an unknown total.
- **Q8, model first.**
  - `PlayerState::pending_start` lives in a new `fp-model/src/pending_start.rs` with four functions: `pending_start_at`, `settle`, `start_request` and `preload_request`.
  - The reducer calls them from `Seek`, `play`, `set_next`, `MoveEntry`, `stop` and `reconcile`.
  - The session stores the pending start and loads it leniently.
  - The view shows it as the stopped player's position, and the stopped waveform becomes seekable.
  - The remote API's seek follows the same rule.
  - The engine needs no change, because it already matches a preload on `(entry, from_secs)`. A test pins that behaviour.

**Tech Stack:** Rust 2024, egui/eframe 0.36.2, egui_kittest 0.36.2, symphonia 0.6.1, crossbeam-channel, serde/serde_json. No new dependency.

**Spec:** `docs/superpowers/specs/2026-10-06-operator-feedback-4-design.md` §3 (Q1, Q8, Q6), §7 (plan 2) and §9 (global constraints). The spec commit has already applied the main spec changes (`docs/superpowers/specs/2026-09-25-fauste-player-design.md` §3.1 `pending_start`, rule 3, rule 3a, §6 "Playability before analysis", §8.3 Info row) and the feedback 2 O17 note. This plan adds only the "As built" notes.

## Rulings

Each ruling resolves a gap or an ambiguity in the spec. Record any further deviation in `.superpowers/sdd/feedback4-plan2/progress.md` in the same form.

- Ruling: the plan's file is `2026-10-06-feedback4-plan2-play-before-analysis.md`, as the coordinator asked, not `…-plan2-start-point.md` (spec §7, plan 3's header) — the coordinator named it — cost if wrong: one renamed link in plan 3 and the spec §7 list, which Task 11 fixes.
- Ruling: Q6 is Tasks 1 and 2, before the model work, although spec §7 says "model first" — the coordinator asked for the reproduction as the first task, and Q6 shares no code with Q1 and Q8 — cost if wrong: none; the tasks are independent.
- Ruling: Task 1 does not commit, and its failing test is committed with the fix in Task 2 — CLAUDE.md allows a commit only with the whole suite green — cost if wrong: none.
- Ruling: Task 2 cannot show the fix's code. It gives the debugging protocol, the hypotheses with their probe tests, a decision table and the gates — the cause is unknown (spec Q6 "Root cause. Not found yet") — cost if wrong: the executor must stop and ask when the evidence points at a layout or behaviour decision (the table says when).
- Ruling: the header read runs on a new `fp-header-reader` thread driven by `Services`, not on the analysis pool — a pool job waits behind a running full analysis (one worker by default), while a header read takes milliseconds — cost if wrong: one more idle thread.
- Ruling: at most `HEADER_READS_IN_FLIGHT = 8` reads wait on the reader at once, and tracks on a player go first — this is internal pacing, so a track loaded on a player overtakes a large folder just added. It is not an operator value (CLAUDE.md rule 4) — cost if wrong: it becomes a `tuning` field later.
- Ruling: `duration_hint_secs` is `None` for an MPEG audio stream (MP1/2/3) whose first 4 KiB after any ID3v2 tag hold no `Xing`, `Info` or `VBRI` frame. Without one, symphonia estimates the length from the first 17 frames' bitrate. On a VBR file that estimate can be short, and a short length would cut the track early on air, because the transition is planned at the end of the play range (Q1.5). This matches the spec's "some VBR files" in Q1.4 — cost if wrong: a CBR MP3 without an Info frame waits for its analysis for a total; the check is one function to relax.
- Ruling: an analysis that found no length (`duration_secs` 0) keeps the header's length (`Track::apply_analysis` only stores a positive one). "The analysis always replaces it" applies when the analysis has a length — cost if wrong: one line in `apply_analysis`.
- Ruling: `pending_start` is a field of `PlayerState`, which has no serde, and `#[serde(default, deserialize_with = "lenient_pending_start")]` goes on `PlayerSession` (spec Q8.1 says `#[serde(default)]` on the state) — only the session is serialised — cost if wrong: none.
- Ruling: a seek on a stopped player follows main spec rule 3a, "clamped to the entry's play range", rather than spec Q8.2's "clamped by `request_at`" (`request_at` clamps to the file):
  - a time before the cue-in becomes the cue-in;
  - a time at or past the known end of the range stores nothing and clears the pending start, because Play would end the track at once (the session restore rule for positions);
  - a broken `secs` (NaN, ±∞) changes nothing;
  - an entry whose file cannot be played gets no pending start.

  Why: the main spec is binding and its rule 3a says so — cost if wrong: one clamp in `pending_start_at`.
- Ruling: Q8.2's "it sends no engine action" means no playback action (no `StartCurrent`, `Seek`, `LoadPaused` or `Resume`). The `Preload` that Q8.7 asks for is still sent — Q8.7 requires it — cost if wrong: none.
- Ruling: every `SetNext` clears the pending start, even one that sets the same entry again. "Play now" is `SetNext` + `Play` (`table.rs` `context_menu`), and Q8.4 lists it among the starts that clear — cost if wrong: re-choosing the same next forgets the chosen start.
- Ruling: `settle` runs at the start of `reconcile`, after every command, every event and the restore. It drops a pending start whose player is not stopped or whose next is another entry, and clamps the rest again to the entry's current play range, because an analysis or a marker may have moved the cue-in. This covers "when `next` changes" and "its entry is removed" generally. `MoveEntry`, `SetNext` and `Stop` clear the pending start explicitly — after a `MoveEntry` the next can stay the same entry — cost if wrong: none; one test per path.
- Ruling: the engine test for Q8.7 is a characterisation test and passes at once. `take_or_open` already matches a preload on `(entry, start_secs)` (`engine.rs` ~L1476). The TDD "watch it fail" step is replaced by a control assertion that fails if reuse breaks — the spec asks for the test and the engine needs no change — cost if wrong: none.
- Ruling: the remote `POST /players/{id}/seek` on a stopped player checks `secs` against the next entry's cue range and sends `Command::Seek`. It is `409` only when there is nothing to seek (stopped with no next) — spec Q8 "The remote API's seek follows the same rule" — cost if wrong: one match arm.
- Ruling: the `seekable` field of `widgets::WaveInput` stays, and both callers pass `true` — plan 3 moves the waveform into `wave_panel.rs` with a `seekable` input (its Q7.1) — cost if wrong: one always-true field until plan 3.
- Ruling: the time row shows the total as the existing `placeholder-none` message ("—" in both locales) when it is unknown, through a pure `view::time_text`. No new Fluent key is needed — cost if wrong: none.

## Assumptions the executor must check first

1. **Branch base.** Plan 2 runs on `fix/operator-feedback-4` after plans that merge before it. Line numbers here are those of `master` at `613a3d7`, so re-locate every reference by its function or field name. Plan 1 changes `services.rs` (`Probe`, `Services` fields and `new`, `submit_new`), `view.rs`, `table.rs` and `cue_window.rs`. Keep its changes when you add Task 5's fields.
2. **The `player_has_cue` guard.** If `fix/operator-feedback-3` is merged by then, `cue_entry`, `cue_follow_target` and `availability` check `config.outputs.player_has_cue` (a Cue route apart from Main). Task 1's fixture sets such routes for every player, so it works with or without that branch. Run `git log --oneline master | grep -c "Cue route equals Main"` to know which code you are on.
3. **symphonia details** (Task 3), checked against the 0.6.1 sources in `~/.cargo/registry`:
   - the MP3 reader estimates `num_frames` for a stream without a Xing/Info/VBRI frame only when it reads 17 frames (`estimate_num_mpeg_frames`);
   - it reads `num_frames` from an `Info` frame with flag bit 0 set;
   - the FLAC reader takes `n_frames` from STREAMINFO without reading audio frames.

   If the workspace has moved off symphonia 0.6.1, run Task 3's tests before you trust the fixtures.
4. **kittest clicks** are pointer events at the node's centre (`egui_kittest-0.36.2/src/node.rs`, `click()` → `click_button`). Task 1 relies on this.
5. **The CUE window** is a non-modal `egui::Window` at `default_pos(80 + 28·index, 120 + 28·index)`, 380 px wide (`cue_window.rs::show`), and it can cover rows of the player's own table.

## Global Constraints

- Spec §9 (binding, verbatim): "`CLAUDE.md` rules 1–10 apply to every plan. In particular: English everywhere, and UI strings in both locales; no product names; operator values are `Config` fields with defaults, ranges and lenient loading (Q12's overrides and view); the real-time path never allocates, locks, logs or panics; behaviour lives in `fp-model` (`SetDuration`, `pending_start`, the DSD reasons); the UI never blocks (the header read and the file stat run on helper threads); bad data never crashes (a header without a duration, a file that changes while it is checked); nothing goes on air by itself (a pending start only changes where Play starts)."
- CLAUDE.md rule 6:
  - no `unwrap`, `expect` or `panic` outside tests;
  - `fp-decode` and `fp-engine` deny `clippy::indexing_slicing`, so use `get` there;
  - test files start with the `#![allow(...)]` line that their neighbours use.
- CLAUDE.md rule 7: each rule of Q1.2, Q1.5 and Q8.2–Q8.8 is a pure function in `fp-model` with at least one test named after the rule.
- CLAUDE.md rule 8: no file is opened on the UI thread. The header read runs on `fp-header-reader`.
- UI strings live in `crates/fp-app/locales/en-US/main.ftl` and `es-ES/main.ftl`, always both. This plan adds no key (see the rulings). `tests/i18n.rs` checks that both locales stay in step.
- Tests:
  - kittests use `Harness::builder().with_step_dt(0.02)` through `tests/support/mod.rs` (`harness`, `harness_sized`) and the recording `Fake` controller;
  - engine tests use the Offline backend and drive time explicitly;
  - never sleep to wait for audio.
- Commit gate for every commit step:

```sh
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
```

  Every commit message ends with the line `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. Never edit `CHANGELOG.md`.

## Review Focus

These are the failure modes the spec implies but does not test, most likely first. Each has a test in the named task.

1. **A short header length on a VBR MP3 without a Xing/Info frame** would plan the transition early and cut the track on air. Such a stream gets no header length. (Task 3 `an_mpeg_stream_without_a_length_frame_has_no_header_duration`.)
2. **A pending start chosen before the analysis moved the cue-in**, which would leave the start in the new trimmed head or past the new end. It is clamped again to the current play range, and the preload follows. (Task 7 `a_pending_start_is_clamped_again_when_the_analysis_moves_the_cue_in`.)
3. **A click in the trimmed tail of a stopped player** would make Play end the track at once. Nothing is stored, and Play starts at the cue-in. (Task 7 `q8_2_a_seek_at_or_past_the_end_of_the_play_range_stores_nothing`.)
4. **Broken or odd files on the header reader** (garbage bytes, a WAV with a broken header) never crash the reader, never send a length, and are not asked again every round. (Task 5 `a_file_whose_header_cannot_be_read_gets_no_duration_and_is_asked_once`; Task 3 `duration_from_frames_rejects_what_is_not_a_length`.)
5. **A late header answer, or a broken session file.**
   - A header answer that arrives after the analysis must not overwrite the analysed length. (Task 4 `q1_2_a_header_duration_after_the_analysis_is_ignored`.)
   - A `session.json` with a malformed `pending_start` must load the rest of the player. (Task 8 `a_broken_pending_start_in_the_session_file_loads_as_none`.)
6. **A large folder added while a track is on a player.** That track gets its length first. (Task 5 `a_track_on_a_player_gets_its_header_duration_ahead_of_the_library`.)

## File Structure

- **Q6.**
  - Create `crates/fp-app/tests/cue_follow_ui.rs` (Task 1).
  - The files of the fix depend on the cause (Task 2; likely `crates/fp-app/src/ui/table.rs`, `ui/cue_window.rs`, `ui/view.rs` or `crates/fp-model/src/reducer.rs`).
- **Q1, decode.** Modify `crates/fp-decode/src/lib.rs` (`duration_from_frames`, `FileDecoder::duration_hint_secs`, `id3v2_len` and `read_up_to` stay private and are reused) and `crates/fp-decode/src/symph.rs` (`length_declared`, `mpeg_length_frame`). Tests go in `crates/fp-decode/tests/info.rs` and `tests/dsd.rs`.
- **Q1, model.** Modify `crates/fp-model/src/command.rs` (`Command::SetDuration`), `reducer.rs` (its arm) and `track.rs` (`Track::needs_header_duration`). Create the test `crates/fp-model/tests/header_duration.rs`.
- **Q1, app.**
  - Create `crates/fp-app/src/header.rs` (`header_duration`, `HeaderReader`) and modify `crates/fp-app/src/lib.rs` (`pub mod header;`).
  - Modify `crates/fp-app/src/services.rs` (fields, `new`, `analysis_step`, `header_pass`, a test hook) and extend `crates/fp-app/tests/services.rs`.
  - Modify `crates/fp-app/src/ui/view.rs` (`time_text`) and `ui/player.rs` (`time_row`), and extend `crates/fp-app/tests/view.rs` and `tests/waveform_ui.rs`.
- **Q8, model.**
  - Create `crates/fp-model/src/pending_start.rs`.
  - Modify `crates/fp-model/src/lib.rs` (`mod pending_start;`), `player.rs` (`PlayerState::pending_start`), `reducer.rs` (`Seek`, `MoveEntry`, `play`, `stop`, `set_next`, `reconcile`) and `session.rs` (`PlayerSession::pending_start`, `lenient_pending_start`, `sessions`, `restore_player`).
  - Create the test `crates/fp-model/tests/pending_start.rs`, and update the `PlayerSession` literals in `crates/fp-model/tests/session.rs`.
- **Q8, engine.** Extend `crates/fp-engine/tests/engine.rs` (test only).
- **Q8, UI.** Modify `crates/fp-app/src/ui/view.rs` (`player_view`), `ui/player.rs` (`wave`: `seekable`, comments) and `ui/widgets.rs` (the `WaveInput::seekable` doc). Extend `tests/view.rs` and `tests/waveform_ui.rs`.
- **Q8, remote.** Modify `crates/fp-remote/src/api.rs` (`O::Seek`) and `crates/fp-remote/tests/api.rs`.
- **Docs.**
  - `docs/user/players.md`;
  - `docs/technical/decoding.md`, `analysis.md`, `ui.md`, `audio-engine.md`, `persistence.md` and `remote-api.md`;
  - the feedback 4 spec (status, the plan 2 file name in §7, "As built" under §3);
  - `docs/superpowers/plans/2026-10-06-feedback4-plan3-wave-panel.md` (the plan 2 link only);
  - `README.md`, only if it describes these behaviours.
- **Locales.** No change in any task: `placeholder-none` already exists in both.

---

### Task 1: Reproduce Q6 with a failing kittest matrix

**Suggested model:** `opus` (careful UI test design; the point is to reproduce an unknown defect).

**Files:**
- Create: `crates/fp-app/tests/cue_follow_ui.rs`
- Docs: none in this task.
- Locales (`en-US`/`es-ES`): none.

**Interfaces:**
- Consumes: `support::{Fake, harness_sized, state}` (`crates/fp-app/tests/support/mod.rs`), `Controller::send`, `Command::{ToggleCue, ShowPlaylist, CreatePlaylistFromPaths}`, `fp_model::{PlayerRoutes, Route}`.
- Produces: the test functions `q6_1_a_click_on_a_row_moves_the_cue_with_its_window_open`, `q6_2_a_double_click_on_a_row_moves_the_cue_with_its_window_open` and `q6_3_the_cue_follows_in_every_layout_tab_and_state`, plus the helper `run(Case) -> Outcome`. Task 2 adds its probe cases to this file.

- [x] **Step 1: Write the test file**

`crates/fp-app/tests/cue_follow_ui.rs`:

```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Operator feedback 4, Q6: with a CUE running on a player, a click or a
//! double-click on a row of a playlist that player shows moves the CUE to
//! that row's entry (Q6.1, Q6.2), with the CUE window open and on any of
//! the player's tabs (Q6.3). Real pointer events, as an operator's mouse.

mod support;

use std::path::PathBuf;

use egui::{Event, Modifiers, PointerButton, Pos2, Rect, Vec2, pos2, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::AppUi;
use fp_app::ui::controller::Controller;
use fp_model::{AppState, Command, PlayerRoutes, PlaylistId, Route};
use support::{harness_sized, state};

const PAUSE: &str = "Pause CUE";
const RESUME: &str = "Resume CUE";
const STOP: &str = "Stop CUE";
const LOAD_NEXT: &str = "Load as next";
const CLOSE: &str = "Close and stop CUE";
const CUE_WAVE: &str = "CUE waveform: click to seek";
const MENU_CUE: &str = "Pre-listen on CUE";

/// How the CUE was started.
#[derive(Debug, Clone, Copy)]
enum Start {
    /// The player's CUE: it plays the next entry (Song 1).
    Button,
    /// "Pre-listen on CUE" in a row's menu: it plays Song 2.
    RowMenu,
}

#[derive(Debug, Clone, Copy)]
enum Gesture {
    Click,
    DoubleClick,
}

#[derive(Debug, Clone, Copy)]
enum Tab {
    /// The player shows "Main", where the cued entry is.
    Same,
    /// The player shows "Other": the cued entry is not on screen.
    Other,
}

#[derive(Debug, Clone, Copy)]
struct Case {
    size: Vec2,
    players: usize,
    /// The player whose CUE runs and whose table is clicked (0-based).
    player: usize,
    start: Start,
    /// The CUE was paused from its window before the gesture.
    paused: bool,
    tab: Tab,
    gesture: Gesture,
}

enum Outcome {
    Moved,
    /// The row is under the CUE window: there is nothing to click.
    Covered,
    Failed(String),
}

fn route(device: &str) -> Route {
    Route {
        backend: "cpal".to_owned(),
        device: device.to_owned(),
        first_channel: 0,
    }
}

/// `players` players showing "Main" (Song 1–4) and a playlist "Other"
/// (Other 1–3). Every track is 180 s long, and every player has a Cue
/// output apart from its Main output (so a CUE can run whether or not
/// the `player_has_cue` guard is in the code).
fn setup(players: usize) -> (AppState, PlaylistId) {
    let mut s = state(players, 4);
    fp_model::apply(
        &mut s,
        Command::CreatePlaylistFromPaths {
            name: "Other".to_owned(),
            paths: (1..=3)
                .map(|n| PathBuf::from(format!("/music/Other {n}.mp3")))
                .collect(),
        },
    )
    .unwrap();
    let other = s.playlists.iter().find(|l| l.name == "Other").unwrap().id;
    for t in s.library.iter_mut() {
        t.duration_secs = 180.0;
    }
    s.config.outputs.routes = s
        .players
        .iter()
        .map(|p| PlayerRoutes {
            player: p.id,
            main: Some(route("desk")),
            cue: Some(route("phones")),
        })
        .collect();
    (s, other)
}

/// Moves the pointer to `at` and presses and releases `button` there.
fn press(h: &mut Harness<'_, AppUi>, at: Pos2, button: PointerButton) {
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    for pressed in [true, false] {
        h.event(Event::PointerButton {
            pos: at,
            button,
            pressed,
            modifiers: Modifiers::NONE,
        });
    }
    h.step();
}

/// The rect of the row titled `title` in the `column`-th table (from the
/// left) that shows it. Only table rows carry these labels in these
/// cases. A playlist shown by one player alone has one such row.
fn row(h: &Harness<'_, AppUi>, title: &str, column: usize) -> Rect {
    let mut rects: Vec<Rect> = h.get_all_by_label(title).map(|n| n.rect()).collect();
    rects.sort_by(|a, b| a.min.x.total_cmp(&b.min.x));
    rects[column.min(rects.len() - 1)]
}

/// The CUE window's area: its controls, grown by the window's frame.
fn cue_window(h: &Harness<'_, AppUi>) -> Option<Rect> {
    [PAUSE, RESUME, STOP, LOAD_NEXT, CLOSE, CUE_WAVE]
        .into_iter()
        .filter_map(|label| h.query_by_label(label))
        .map(|n| n.rect())
        .reduce(|a, b| a.union(b))
        .map(|r| r.expand(13.0))
}

/// A point of `row` that the CUE window does not cover, if any.
fn visible_point(row: Rect, window: Option<Rect>) -> Option<Pos2> {
    let y = row.center().y;
    [row.center().x, row.left() + 2.0, row.right() - 2.0]
        .into_iter()
        .map(|x| pos2(x, y))
        .find(|p| window.is_none_or(|w| !w.contains(*p)))
}

fn run(case: Case) -> Outcome {
    let (s, other) = setup(case.players);
    let (mut h, fake) = harness_sized(s, case.size, |ui| ui);
    let p = fake.player(case.player);
    let main = fake.entries();
    let cued = match case.start {
        Start::Button => {
            fake.send(Command::ToggleCue(p));
            h.run_steps(2);
            main[0]
        }
        Start::RowMenu => {
            let r = row(&h, "Song 2", case.player);
            press(&mut h, r.center(), PointerButton::Secondary);
            h.run_steps(2);
            h.get_by_label(MENU_CUE).click();
            h.run_steps(2);
            main[1]
        }
    };
    if fake.state.load().players[case.player].cue.map(|c| c.entry) != Some(cued) {
        return Outcome::Failed(format!("{case:?}: the CUE did not start on {cued:?}"));
    }
    if case.paused {
        h.get_by_label(PAUSE).click();
        h.run_steps(2);
    }
    let (title, target) = match case.tab {
        Tab::Same => ("Song 3", main[2]),
        Tab::Other => {
            fake.send(Command::ShowPlaylist(p, other));
            h.run_steps(2);
            let entry = fake.state.load().playlists.get(other).unwrap().entries[1].id;
            ("Other 2", entry)
        }
    };
    if h.query_by_label(STOP).is_none() {
        return Outcome::Failed(format!("{case:?}: the CUE window is not open"));
    }
    let Some(at) = visible_point(row(&h, title, case.player), cue_window(&h)) else {
        return Outcome::Covered;
    };
    fake.take_sent();
    press(&mut h, at, PointerButton::Primary);
    if matches!(case.gesture, Gesture::DoubleClick) {
        press(&mut h, at, PointerButton::Primary);
    }
    h.run_steps(2);
    let sent = fake.take_sent();
    if matches!(case.gesture, Gesture::DoubleClick) && !sent.contains(&Command::SetNext(p, target))
    {
        return Outcome::Failed(format!("{case:?}: the double-click did not set the next; sent {sent:?}"));
    }
    let now = fake.state.load().players[case.player].cue.map(|c| c.entry);
    if now == Some(target) {
        Outcome::Moved
    } else {
        Outcome::Failed(format!(
            "{case:?}: the CUE is on {now:?}, not {target:?}; sent {sent:?}"
        ))
    }
}

fn expect_moved(case: Case) {
    match run(case) {
        Outcome::Moved => {}
        Outcome::Covered => panic!("{case:?}: the row is under the CUE window"),
        Outcome::Failed(why) => panic!("{why}"),
    }
}

fn simple(gesture: Gesture) -> Case {
    Case {
        size: vec2(1000.0, 700.0),
        players: 1,
        player: 0,
        start: Start::Button,
        paused: false,
        tab: Tab::Same,
        gesture,
    }
}

#[test]
fn q6_1_a_click_on_a_row_moves_the_cue_with_its_window_open() {
    expect_moved(simple(Gesture::Click));
}

#[test]
fn q6_2_a_double_click_on_a_row_moves_the_cue_with_its_window_open() {
    expect_moved(simple(Gesture::DoubleClick));
}

#[test]
fn q6_3_the_cue_follows_in_every_layout_tab_and_state() {
    // (window size, players, the player whose CUE runs)
    let layouts = [
        (vec2(1000.0, 700.0), 1, 0),
        (vec2(1920.0, 1080.0), 4, 0),
        (vec2(1920.0, 1080.0), 4, 3),
    ];
    let mut failed = Vec::new();
    let mut covered = Vec::new();
    for (size, players, player) in layouts {
        for start in [Start::Button, Start::RowMenu] {
            for paused in [false, true] {
                for tab in [Tab::Same, Tab::Other] {
                    for gesture in [Gesture::Click, Gesture::DoubleClick] {
                        let case = Case {
                            size,
                            players,
                            player,
                            start,
                            paused,
                            tab,
                            gesture,
                        };
                        match run(case) {
                            Outcome::Moved => {}
                            Outcome::Covered => covered.push(case),
                            Outcome::Failed(why) => failed.push(why),
                        }
                    }
                }
            }
        }
    }
    // A row under the CUE window cannot be clicked; Task 2 looks at these.
    eprintln!("rows under the CUE window: {covered:#?}");
    assert!(
        failed.is_empty(),
        "{} cases did not move the CUE:\n{}",
        failed.len(),
        failed.join("\n")
    );
}
```

- [x] **Step 2: Run it and record what fails**

Run: `cargo test -p fp-app --test cue_follow_ui -- --nocapture`

Expected: at least one test FAILS, which reproduces the operator's report. Copy the full list of failing cases and the "rows under the CUE window" list into `.superpowers/sdd/feedback4-plan2/progress.md`.

If every case passes, the defect is not reproduced yet. Do not write any product code. Go on with Task 2 Step 1, which adds probe cases until one fails.

If the file does not compile (a label or an API differs on this branch), fix the test only. Re-locate labels with `grep -n "cue-window-\|menu-cue" crates/fp-app/locales/en-US/main.ftl`.

- [ ] **Step 3: No commit**

Nothing is committed in this task, because the suite is red. Task 2 commits this file together with the fix.

---

### Task 2: Find and fix the cause of Q6

**Suggested model:** `opus` (debugging with no known cause; the fix may touch the model or the UI).

**Files:**
- Modify: `crates/fp-app/tests/cue_follow_ui.rs` (the probe cases below).
- Modify: the files the evidence points at. The candidates are `crates/fp-app/src/ui/table.rs` (`track_table`: the row response, `clicked`, `cue_follow` and `dragged`, ~L497–548), `crates/fp-app/src/ui/view.rs` (`cue_follow_target`, ~L159), `crates/fp-app/src/ui/cue_window.rs` (`show`, the window placement), `crates/fp-model/src/reducer.rs` (`follow_cue` ~L855, `cue_entry` ~L833, `set_next`).
- Test: `crates/fp-app/tests/cue_follow_ui.rs`. Also `crates/fp-model/tests/cue_window.rs` if the cause is in the model (spec Q6 "reducer tests for `follow_cue` if the cause is in the model").
- Docs: `docs/user/players.md` (the CUE section, ~L165–168), only if the visible behaviour changes. Spec "As built" is in Task 11.
- Locales (`en-US`/`es-ES`): none, unless the fix adds a visible string; then add it to both files in the same commit.

**Interfaces:**
- Consumes: Task 1's `run`, `Case`, `press`, `row`, `cue_window` and `visible_point`.
- Produces: a green `cue_follow_ui.rs`. No new public API is expected.

- [x] **Step 1: Load the skill and gather evidence (Phase 1)**

Invoke `superpowers:systematic-debugging`. Do not change product code until Phase 3 names one cause.

Group Task 1's failing cases by dimension (layout, player, start, paused, tab, gesture). The shared dimension is the first lead. For each failing case, add a temporary `eprintln!` in `table.rs` after `let response = row.response();` that prints `entry.id`, `response.clicked()`, `response.double_clicked()`, `response.drag_started()` and `cue_follow`. Then run:

`cargo test -p fp-app --test cue_follow_ui q6_3 -- --nocapture`

Record the output in the ledger, and remove the `eprintln!` before you commit.

- [x] **Step 2: Add the probe cases (Phase 2)**

Append these tests to `cue_follow_ui.rs`. Each one pins one hypothesis, and keeping them as regression tests is fine.

```rust
/// H2: a press that moves a few pixels before its release. egui counts it
/// as a click below its drag threshold (6 px). Above it, the row starts a
/// drag and becomes the selection, but the CUE does not follow.
#[test]
fn h2_a_click_with_a_small_movement_still_moves_the_cue() {
    let (s, _) = setup(1);
    let (mut h, fake) = harness_sized(s, vec2(1000.0, 700.0), |ui| ui);
    let p = fake.player(0);
    let e = fake.entries();
    fake.send(Command::ToggleCue(p));
    h.run_steps(2);
    let at = visible_point(row(&h, "Song 3", 0), cue_window(&h)).unwrap();
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    h.event(Event::PointerButton {
        pos: at,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    h.step();
    let to = pos2(at.x + 3.0, at.y + 1.0);
    h.event(Event::PointerMoved(to));
    h.step();
    h.event(Event::PointerButton {
        pos: to,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(2);
    assert_eq!(fake.state.load().players[0].cue.map(|c| c.entry), Some(e[2]));
}

/// H3: a click held as long as a slow operator's (0.5 s) before release.
#[test]
fn h3_a_slow_click_still_moves_the_cue() {
    let (s, _) = setup(1);
    let (mut h, fake) = harness_sized(s, vec2(1000.0, 700.0), |ui| ui);
    let p = fake.player(0);
    let e = fake.entries();
    fake.send(Command::ToggleCue(p));
    h.run_steps(2);
    let at = visible_point(row(&h, "Song 3", 0), cue_window(&h)).unwrap();
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    h.event(Event::PointerButton {
        pos: at,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(25);
    h.event(Event::PointerButton {
        pos: at,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(2);
    assert_eq!(fake.state.load().players[0].cue.map(|c| c.entry), Some(e[2]));
}

/// H4: the window was used last (it holds the focus) before the row is
/// clicked: Pause, then Resume, then the row.
#[test]
fn h4_a_click_after_using_the_cue_window_moves_the_cue() {
    let (s, _) = setup(1);
    let (mut h, fake) = harness_sized(s, vec2(1000.0, 700.0), |ui| ui);
    let p = fake.player(0);
    let e = fake.entries();
    fake.send(Command::ToggleCue(p));
    h.run_steps(2);
    h.get_by_label(PAUSE).click();
    h.run_steps(2);
    h.get_by_label(RESUME).click();
    h.run_steps(2);
    let at = visible_point(row(&h, "Song 3", 0), cue_window(&h)).unwrap();
    press(&mut h, at, PointerButton::Primary);
    h.run_steps(2);
    assert_eq!(fake.state.load().players[0].cue.map(|c| c.entry), Some(e[2]));
}

/// H5: the current entry is on air (the player plays Song 1) and the CUE
/// pre-listens the next (Song 2); then a click on Song 3.
#[test]
fn h5_a_click_while_the_player_is_on_air_moves_the_cue() {
    let (s, _) = setup(1);
    let (mut h, fake) = harness_sized(s, vec2(1000.0, 700.0), |ui| ui);
    let p = fake.player(0);
    let e = fake.entries();
    fake.send(Command::Play(p));
    fake.send(Command::ToggleCue(p));
    h.run_steps(2);
    assert_eq!(fake.state.load().players[0].cue.map(|c| c.entry), Some(e[1]));
    let at = visible_point(row(&h, "Song 3", 0), cue_window(&h)).unwrap();
    press(&mut h, at, PointerButton::Primary);
    h.run_steps(2);
    assert_eq!(fake.state.load().players[0].cue.map(|c| c.entry), Some(e[2]));
}
```

Run: `cargo test -p fp-app --test cue_follow_ui -- --nocapture`

Record which probes fail. If nothing fails at all (Task 1 and every probe pass), the defect cannot be reproduced from the description. In that case, stop and ask the maintainer for the exact steps: the number of players, the window size, how the CUE was started, and whether the window had been moved. Record a `Ruling:` line in the ledger and do not ship a speculative fix.

- [ ] **Step 3: Name the cause (Phase 3) and choose where the fix lives**

Use the evidence and this table. Write the chosen line in the ledger as `Ruling: cause is <…> — <evidence> — <cost if wrong>`.

| Evidence | Cause | Where the fix lives | Before you fix |
|---|---|---|---|
| Failures only in cases whose row is `Covered`, or the operator's rows sit under the window in the 1920×1080 layouts | The CUE window covers its own player's rows | `cue_window.rs::show` (`default_pos`) | **Stop and ask the maintainer**: where the window opens is a layout decision (feedback 2 O12). Record their answer as a ruling. |
| `clicked()` false and `drag_started()` true for the clicked row (H2 or H3 fails) | The click becomes a row drag, so `cue_follow` is never computed | `table.rs` (`track_table`): compute the follow on the gesture that selects the row, but never on a drag that moves an entry | Write a probe asserting that a real drag still moves the entry and does not move the CUE. |
| `clicked()` true, `cue_follow` `None` | A guard in `view::cue_follow_target` (or `player_has_cue`) refuses | `view.rs` | A `tests/view.rs` test for the guard, with the failing inputs. |
| `CueEntry`/`SetNext` sent, but the model's `cue` does not change | `follow_cue`/`cue_entry` refuse or return early | `reducer.rs` | One test per broken rule in `crates/fp-model/tests/cue_window.rs`. |
| Anything else | — | — | Back to Phase 1 with a new probe. |

- [ ] **Step 4: Write the failing model or view test the table asks for**

Write it in the file named in the table, with the exact inputs from the failing case. Run it and watch it fail:

`cargo test -p fp-model --test cue_window <name>` or `cargo test -p fp-app --test view <name>`

- [ ] **Step 5: Fix the cause (Phase 4)**

Make the smallest change in the file the table names. Do not touch other guards (unplayable file, already cued, no CUE output; Q6.3 keeps them). Then run:

```sh
cargo test -p fp-app --test cue_follow_ui
cargo test -p fp-app --test main_screen cue
cargo test -p fp-app --test cue_window
cargo test -p fp-model --test cue_window
```

Expected: all PASS, including the guard tests `clicking_the_cued_row_a_missing_file_or_without_a_cue_sends_nothing` and `a_double_click_moves_the_cue_once` in `main_screen.rs`.

- [ ] **Step 6: Docs**

If the visible behaviour changed (for example, where the CUE window opens), update `docs/user/players.md`, the CUE section (~L140–168), and `docs/technical/ui.md` (the CUE window paragraph, ~L220–230) in the same words as the change. Otherwise leave them as they are; players.md already describes the rule.

- [ ] **Step 7: Commit**

```sh
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace \
&& git add crates/fp-app/tests/cue_follow_ui.rs <the files the fix changed> \
&& git commit -m "fix(<ui|model>): a CUE follows a row of its own player's playlist" -m "<the cause, in one or two sentences, from the ledger ruling>" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

Use `ui` as the scope when the fix is in `crates/fp-app`, and `model` when it is in `crates/fp-model`.

---

### Task 3: The length a file's header declares (`fp-decode`)

**Suggested model:** `sonnet` (contained decoder change with exact fixtures).

**Files:**
- Modify: `crates/fp-decode/src/lib.rs` (after `FileDecoder::frames_hint`, ~L195–202; a free function after `read_up_to`, ~L89–100)
- Modify: `crates/fp-decode/src/symph.rs` (struct `SymphoniaDecoder` fields ~L17–48, `open` ~L56–125, a new private function at the end)
- Test: `crates/fp-decode/tests/info.rs` (append), `crates/fp-decode/tests/dsd.rs` (append one test)
- Docs: `docs/technical/decoding.md` (the backend interface sentence, ~L22–25)
- Locales (`en-US`/`es-ES`): none.

**Interfaces:**
- Produces:
  - `pub fn fp_decode::duration_from_frames(frames: Option<u64>, rate: u32) -> Option<f64>`;
  - `pub fn FileDecoder::duration_hint_secs(&self) -> Option<f64>`;
  - `pub(crate) fn SymphoniaDecoder::length_declared(&self) -> bool`.

- [x] **Step 1: Write the failing tests**

Append to `crates/fp-decode/tests/info.rs` (it already has `use fp_decode::FileDecoder;`; add the imports below at the top):

```rust
use std::path::{Path, PathBuf};

use fp_decode::duration_from_frames;

fn write(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, bytes).unwrap();
    path
}

/// A FLAC file of `samples` stereo 16-bit frames at 44.1 kHz with its
/// STREAMINFO block only: the length is in the header, no frame is needed.
fn flac_header_only(samples: u64) -> Vec<u8> {
    let mut b = b"fLaC".to_vec();
    // Last metadata block, type 0 (STREAMINFO), 34 bytes.
    b.extend_from_slice(&[0x80, 0x00, 0x00, 34]);
    b.extend_from_slice(&4096u16.to_be_bytes()); // min block size
    b.extend_from_slice(&4096u16.to_be_bytes()); // max block size
    b.extend_from_slice(&[0, 0, 0, 0, 0, 0]); // min and max frame size: unknown
    // Sample rate (20 bits), channels − 1 (3), bits − 1 (5), samples (36).
    let packed: u64 = (44_100u64 << 44) | (1 << 41) | (15 << 36) | samples;
    b.extend_from_slice(&packed.to_be_bytes());
    b.extend_from_slice(&[0u8; 16]); // MD5: not computed
    b
}

/// The length of one MPEG-1 Layer III frame at 128 kbit/s and 44.1 kHz.
const MP3_FRAME: usize = 417;

/// `frames` silent mono MP3 frames (128 kbit/s, 44.1 kHz). With `info`,
/// the first one is an Info frame that declares the other frames.
fn mp3(frames: usize, info: bool) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(frames * MP3_FRAME);
    for k in 0..frames {
        let mut frame = vec![0u8; MP3_FRAME];
        frame[..4].copy_from_slice(&[0xFF, 0xFB, 0x90, 0xC0]);
        if info && k == 0 {
            // After the header and 17 bytes of mono side information:
            // "Info", flags (the frame count is present), the frame count.
            frame[21..25].copy_from_slice(b"Info");
            frame[25..29].copy_from_slice(&1u32.to_be_bytes());
            frame[29..33].copy_from_slice(&((frames - 1) as u32).to_be_bytes());
        }
        bytes.extend_from_slice(&frame);
    }
    bytes
}

#[test]
fn duration_from_frames_rejects_what_is_not_a_length() {
    assert_eq!(duration_from_frames(Some(44_100), 44_100), Some(1.0));
    assert_eq!(duration_from_frames(None, 44_100), None);
    assert_eq!(duration_from_frames(Some(0), 44_100), None);
    assert_eq!(duration_from_frames(Some(44_100), 0), None);
}

#[test]
fn a_wav_header_gives_its_length() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("two.wav");
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 48_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(&path, spec).unwrap();
    for i in 0..96_000 {
        w.write_sample((i % 100) as i16).unwrap();
        w.write_sample((i % 100) as i16).unwrap();
    }
    w.finalize().unwrap();
    let d = FileDecoder::open(&path).unwrap();
    assert_eq!(d.duration_hint_secs(), Some(2.0));
}

#[test]
fn a_flac_streaminfo_gives_its_length_without_decoding() {
    let dir = tempfile::tempdir().unwrap();
    let path = write(dir.path(), "two.flac", &flac_header_only(88_200));
    let d = FileDecoder::open(&path).unwrap();
    assert_eq!(d.duration_hint_secs(), Some(2.0));
}

#[test]
fn an_mpeg_stream_without_a_length_frame_has_no_header_duration() {
    let dir = tempfile::tempdir().unwrap();
    // 40 frames: symphonia estimates a length from the bitrate...
    let long = write(dir.path(), "vbr.mp3", &mp3(40, false));
    let d = FileDecoder::open(&long).unwrap();
    assert!(d.frames_hint().is_some(), "symphonia estimates it");
    // ...which a VBR file can get wrong: it is not used.
    assert_eq!(d.duration_hint_secs(), None);
    // 8 frames: too short even to estimate.
    let short = write(dir.path(), "short.mp3", &mp3(8, false));
    assert_eq!(FileDecoder::open(&short).unwrap().duration_hint_secs(), None);
}

#[test]
fn an_mpeg_stream_with_an_info_frame_gives_its_length() {
    let dir = tempfile::tempdir().unwrap();
    let path = write(dir.path(), "cbr.mp3", &mp3(40, true));
    let secs = FileDecoder::open(&path).unwrap().duration_hint_secs().unwrap();
    let expected = 39.0 * 1152.0 / 44_100.0;
    assert!((secs - expected).abs() < 0.05, "{secs} vs {expected}");
}
```

Append to `crates/fp-decode/tests/dsd.rs`, after `dsd64_sine_decodes_at_the_sacd_reference_level`:

```rust
#[test]
fn a_dsf_header_gives_its_length_in_seconds() {
    let dir = tempfile::tempdir().unwrap();
    let samples = DSD64 as usize / 2;
    let bits = modulate(sine(1000.0, 0.5), DSD64, samples);
    let path = write(dir.path(), "half.dsf", &dsf(&[pack(&bits)], samples as u64));
    let secs = FileDecoder::open(&path).unwrap().duration_hint_secs().unwrap();
    assert!((secs - 0.5).abs() < 1e-9, "{secs}");
}
```

- [x] **Step 2: Run them to make sure they fail**

Run: `cargo test -p fp-decode --test info` and `cargo test -p fp-decode --test dsd a_dsf_header_gives_its_length_in_seconds`

Expected: compile errors, because `duration_from_frames` and `duration_hint_secs` are not defined.

- [x] **Step 3: Implement**

In `crates/fp-decode/src/lib.rs`, after `read_up_to`:

```rust
/// Seconds of `frames` frames at `rate`: `None` when the count is unknown
/// or zero, or the rate is zero (operator feedback 4, Q1).
pub fn duration_from_frames(frames: Option<u64>, rate: u32) -> Option<f64> {
    let frames = frames.filter(|f| *f > 0)?;
    if rate == 0 {
        return None;
    }
    let secs = frames as f64 / f64::from(rate);
    (secs.is_finite() && secs > 0.0).then_some(secs)
}
```

In `impl FileDecoder`, after `frames_hint`:

```rust
    /// The length the container declares, in seconds, without decoding
    /// (operator feedback 4, Q1): what a track shows until its analysis.
    /// `None` when the container does not say, and for an MPEG stream
    /// whose length symphonia would only estimate (no Xing, Info or VBRI
    /// frame): on a VBR file that estimate can end a track early.
    pub fn duration_hint_secs(&self) -> Option<f64> {
        let declared = match &self.backend {
            Backend::Symphonia(d) => d.length_declared(),
            Backend::WavPack(_) | Backend::Ape(_) | Backend::Dsd(_) => true,
        };
        if !declared {
            return None;
        }
        duration_from_frames(self.frames_hint(), self.sample_rate())
    }
```

In `crates/fp-decode/src/symph.rs`:
- add the field `length_declared: bool` to `SymphoniaDecoder`, after `frames_hint`, with the doc comment `/// False for an MPEG stream whose length is only symphonia's estimate.`;
- in `open`, after `let is_opus = …;`, add the lines below;
- set `length_declared,` in the `Self { … }` literal.

```rust
        let mpeg = [
            well_known::CODEC_ID_MP1,
            well_known::CODEC_ID_MP2,
            well_known::CODEC_ID_MP3,
        ]
        .contains(&params.codec);
        let length_declared = !mpeg || mpeg_length_frame(path);
```

After `frames_hint` in `impl SymphoniaDecoder`:

```rust
    /// Whether `frames_hint` is the container's own figure.
    pub(crate) fn length_declared(&self) -> bool {
        self.length_declared
    }
```

At the end of `symph.rs`:

```rust
/// How far into an MPEG stream (after any ID3v2 tag) a length frame is
/// looked for: its first frame, with room for the largest.
const MPEG_LENGTH_FRAME_SPAN: usize = 4096;

/// Whether an MPEG audio file starts with a frame that declares the
/// stream's length (Xing, Info or VBRI). Without one symphonia estimates
/// the length from the first frames' bitrate.
fn mpeg_length_frame(path: &Path) -> bool {
    use std::io::{Seek, SeekFrom};
    let Ok(mut file) = File::open(path) else {
        return false;
    };
    let mut head = [0u8; 10];
    let Ok(read) = crate::read_up_to(&mut file, &mut head) else {
        return false;
    };
    let start = crate::id3v2_len(head.get(..read).unwrap_or_default()).unwrap_or(0);
    if file.seek(SeekFrom::Start(start)).is_err() {
        return false;
    }
    let mut span = vec![0u8; MPEG_LENGTH_FRAME_SPAN];
    let Ok(read) = crate::read_up_to(&mut file, &mut span) else {
        return false;
    };
    span.get(..read)
        .unwrap_or_default()
        .windows(4)
        .any(|w| w == b"Xing" || w == b"Info" || w == b"VBRI")
}
```

`read_up_to` and `id3v2_len` are private functions of the crate root, and a child module can use them. If `read_up_to` returns `std::io::Result<usize>`, the `let Ok(..) else` above matches it as it is.

- [x] **Step 4: Run the tests to make sure they pass**

Run: `cargo test -p fp-decode --test info`, then `cargo test -p fp-decode --test dsd a_dsf_header_gives_its_length_in_seconds`, then `cargo test -p fp-decode`

Expected: PASS, and the decoder's other tests stay green.

- [x] **Step 5: Docs**

In `docs/technical/decoding.md`, after the sentence that lists the backend interface (`sample_rate`, …, `frames_hint`, …), add:

```markdown
`FileDecoder::duration_hint_secs` turns `frames_hint` into seconds
(`duration_from_frames`) for the length a track shows before its analysis
(operator feedback 4, Q1). An MPEG stream (MP1/2/3) has one only when its
first frame is a Xing, Info or VBRI frame: without it symphonia estimates
the length from the first frames' bitrate, which a VBR file can get short,
and a short length would end the track early.
```

- [x] **Step 6: Commit**

```sh
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace \
&& git add crates/fp-decode docs/technical/decoding.md \
&& git commit -m "feat(decode): the length a file's header declares" -m "A track played before its analysis needs a length for its countdown and seek. MPEG streams without a length frame get none: symphonia's bitrate estimate can be short on VBR files and would cut the track." -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: `Command::SetDuration` (model, Q1.2 and Q1.5)

**Suggested model:** `opus` (reducer rule; scheduling follows the new length).

**Files:**
- Modify: `crates/fp-model/src/command.rs` (after `ApplyTags`, ~L66–69)
- Modify: `crates/fp-model/src/reducer.rs` (`apply`, after the `Command::ApplyTags` arm, ~L152–156)
- Modify: `crates/fp-model/src/track.rs` (after `needs_tag_read`, ~L382–384)
- Test: `crates/fp-model/tests/header_duration.rs` (create)
- Docs: `docs/technical/analysis.md` (§"How the app uses it", the paragraph on what is submitted, ~L202–230) for the model rule
- Locales (`en-US`/`es-ES`): none.

**Interfaces:**
- Produces:
  - `Command::SetDuration { track: TrackId, secs: f64 }`;
  - `pub fn Track::needs_header_duration(&self) -> bool`, which is `!analyzed`, no positive length, and a playable file.

- [x] **Step 1: Write the failing tests**

`crates/fp-model/tests/header_duration.rs`:

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::float_cmp)]
//! Operator feedback 4, Q1: a track's length from its file header, until
//! its analysis replaces it.

use std::path::PathBuf;

use fp_model::{
    AppState, Command, Config, FileState, Track, TrackAnalysis, TrackId, TransitionPlan, apply,
    plan_for,
};

/// One playlist of `n` tracks, none analysed, none with a length.
fn unanalysed(n: usize) -> (AppState, Vec<TrackId>) {
    let mut s = AppState::new(Config::default(), "Main");
    let playlist = s.playlists.first_id().unwrap();
    let paths = (0..n)
        .map(|i| PathBuf::from(format!("/music/t{i}.mp3")))
        .collect();
    apply(
        &mut s,
        Command::InsertPaths {
            playlist,
            index: 0,
            paths,
        },
    )
    .unwrap();
    let tracks = s
        .playlists
        .get(playlist)
        .unwrap()
        .entries
        .iter()
        .map(|e| e.track)
        .collect();
    (s, tracks)
}

fn length(s: &AppState, t: TrackId) -> f64 {
    s.library.get(t).unwrap().duration_secs
}

fn analysis(duration_secs: f64) -> Box<TrackAnalysis> {
    Box::new(TrackAnalysis {
        duration_secs,
        ..TrackAnalysis::default()
    })
}

#[test]
fn q1_2_a_header_duration_is_stored_before_the_analysis() {
    let (mut s, t) = unanalysed(1);
    apply(&mut s, Command::SetDuration { track: t[0], secs: 200.0 }).unwrap();
    assert_eq!(length(&s, t[0]), 200.0);
    assert!(!s.library.get(t[0]).unwrap().analyzed);
}

#[test]
fn q1_2_the_analysis_replaces_the_header_duration() {
    let (mut s, t) = unanalysed(1);
    apply(&mut s, Command::SetDuration { track: t[0], secs: 200.0 }).unwrap();
    apply(
        &mut s,
        Command::ApplyAnalysis {
            track: t[0],
            analysis: analysis(198.5),
        },
    )
    .unwrap();
    assert_eq!(length(&s, t[0]), 198.5);
}

#[test]
fn an_analysis_without_a_length_keeps_the_header_duration() {
    let (mut s, t) = unanalysed(1);
    apply(&mut s, Command::SetDuration { track: t[0], secs: 200.0 }).unwrap();
    apply(
        &mut s,
        Command::ApplyAnalysis {
            track: t[0],
            analysis: analysis(0.0),
        },
    )
    .unwrap();
    assert_eq!(length(&s, t[0]), 200.0);
}

#[test]
fn q1_2_a_header_duration_after_the_analysis_is_ignored() {
    let (mut s, t) = unanalysed(1);
    apply(
        &mut s,
        Command::ApplyAnalysis {
            track: t[0],
            analysis: analysis(198.5),
        },
    )
    .unwrap();
    apply(&mut s, Command::SetDuration { track: t[0], secs: 250.0 }).unwrap();
    assert_eq!(length(&s, t[0]), 198.5);
}

#[test]
fn q1_2_a_duration_that_is_not_finite_and_positive_is_rejected() {
    let (mut s, t) = unanalysed(1);
    for bad in [0.0, -3.0, f64::NAN, f64::INFINITY] {
        apply(&mut s, Command::SetDuration { track: t[0], secs: bad }).unwrap();
        assert_eq!(length(&s, t[0]), 0.0, "{bad}");
    }
}

#[test]
fn q1_2_an_unknown_track_is_ignored() {
    let (mut s, _) = unanalysed(1);
    let before = s.clone();
    apply(
        &mut s,
        Command::SetDuration {
            track: TrackId(999_999),
            secs: 10.0,
        },
    )
    .unwrap();
    assert_eq!(s.library, before.library);
}

#[test]
fn q1_5_the_play_range_of_such_a_track_is_zero_to_its_duration() {
    let (mut s, t) = unanalysed(1);
    apply(&mut s, Command::SetDuration { track: t[0], secs: 200.0 }).unwrap();
    for use_markers in [true, false] {
        let range = s.library.get(t[0]).unwrap().play_range(use_markers);
        assert_eq!(range.cue_in, 0.0);
        assert_eq!(range.known_end(), Some(200.0), "{use_markers}");
    }
}

#[test]
fn q1_5_the_transition_is_planned_at_the_header_duration() {
    let (mut s, t) = unanalysed(2);
    let p = s.players[0].id;
    apply(&mut s, Command::Play(p)).unwrap();
    apply(&mut s, Command::SetDuration { track: t[0], secs: 200.0 }).unwrap();
    let at = match plan_for(&s, s.player(p).unwrap()) {
        Some(TransitionPlan::StartNextAt { at_secs, .. } | TransitionPlan::StopAt { at_secs }) => {
            at_secs
        }
        other => panic!("{other:?}"),
    };
    assert_eq!(at, 200.0);
}

#[test]
fn a_track_needs_a_header_read_only_while_it_has_no_length_and_no_analysis() {
    let mut t = Track::new(TrackId(1), PathBuf::from("/music/a.mp3"));
    assert!(t.needs_header_duration());
    t.duration_secs = 10.0;
    assert!(!t.needs_header_duration(), "it has a length");
    t.duration_secs = 0.0;
    t.file_state = FileState::Missing;
    assert!(!t.needs_header_duration(), "no file to read");
    t.file_state = FileState::Ok;
    t.analyzed = true;
    assert!(!t.needs_header_duration(), "analysed");
}
```

If `Library` does not implement `PartialEq` (check with `grep -n "derive" crates/fp-model/src/track.rs | head`), compare `s.library.iter().map(|t| t.duration_secs).collect::<Vec<_>>()` instead in `q1_2_an_unknown_track_is_ignored`.

- [x] **Step 2: Run them to make sure they fail**

Run: `cargo test -p fp-model --test header_duration`

Expected: a compile error, because there is no variant `SetDuration` and no method `needs_header_duration`.

- [x] **Step 3: Implement**

`crates/fp-model/src/command.rs`, after `ApplyTags { … },`:

```rust
    /// Operator feedback 4, Q1.2: the length the file's header declares,
    /// stored only while the track is not analysed and `secs` is finite
    /// and positive (ignored if the track is gone). The analysis replaces
    /// it.
    SetDuration {
        track: TrackId,
        secs: f64,
    },
```

`crates/fp-model/src/reducer.rs`, after the `Command::ApplyTags { … }` arm:

```rust
        Command::SetDuration { track, secs } => {
            // Q1.2: the header's length stands in until the analysis.
            if secs.is_finite()
                && secs > 0.0
                && let Some(t) = state.library.get_mut(track)
                && !t.analyzed
            {
                t.duration_secs = secs;
            }
        }
```

`crates/fp-model/src/track.rs`, after `needs_tag_read`:

```rust
    /// Operator feedback 4, Q1.1: whether the header reader should read
    /// this track's length: not analysed, no length yet, and a file that
    /// can be opened.
    pub fn needs_header_duration(&self) -> bool {
        !self.analyzed && !(self.duration_secs > 0.0) && self.file_state.is_playable()
    }
```

The `!(… > 0.0)` form also treats a NaN length as "no length". If clippy flags `neg_cmp_op_on_partial_ord`, write `self.duration_secs.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)` instead.

- [x] **Step 4: Run the tests to make sure they pass**

Run: `cargo test -p fp-model --test header_duration`, then `cargo test -p fp-model`

Expected: PASS.

- [x] **Step 5: Docs**

In `docs/technical/analysis.md`, at the end of §"How the app uses it (`fp-app/src/services.rs`)", add:

```markdown
Before its analysis a track has no length. `Command::SetDuration` stores
the length its file's header declares while the track is not analysed
(operator feedback 4, Q1.2): a finite, positive value only, ignored once
the track is analysed. The analysis replaces it when it finds a length.
With it the track has a countdown, a position and click-to-seek, and its
play range is `0..duration` (Q1.5).
```

- [x] **Step 6: Commit**

```sh
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace \
&& git add crates/fp-model docs/technical/analysis.md \
&& git commit -m "feat(model): a track's length from its file header until its analysis" -m "Q1.2: SetDuration stores the header's length while the track is not analysed; the analysis replaces it." -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: The header reader thread and its pass in `Services` (Q1.1)

**Suggested model:** `opus` (a new thread and services bookkeeping; ordering and retries).

**Files:**
- Create: `crates/fp-app/src/header.rs`
- Modify: `crates/fp-app/src/lib.rs` (add `pub mod header;` after `pub mod crash;`)
- Modify: `crates/fp-app/src/services.rs`:
  - the imports;
  - the `Services` fields (~L229–276) and `new` (~L278–319);
  - `analysis_step` (~L375–415): call `self.header_pass(state)` after `self.submit_new(state)`;
  - a new `header_pass` after `tag_pass` (~L553–594);
  - a test-hook accessor after `tag_pass_on` (~L347–352).
- Test: `crates/fp-app/tests/services.rs` (append)
- Docs: `docs/technical/analysis.md` (the list of threads and what is submitted, ~L202–230)
- Locales (`en-US`/`es-ES`): none.

**Interfaces:**
- Consumes: `fp_decode::FileDecoder::duration_hint_secs` (Task 3), `Command::SetDuration` and `Track::needs_header_duration` (Task 4), `Services::wanted` (existing, private associated function).
- Produces:
  - `pub fn fp_app::header::header_duration(path: &Path) -> Option<f64>`;
  - `pub struct fp_app::header::HeaderReader` with `spawn() -> std::io::Result<Self>`, `submit(&self, TrackId, PathBuf) -> bool` and `answers(&self) -> &Receiver<(TrackId, Option<f64>)>`;
  - the test hook `#[cfg(feature = "test-hooks")] pub fn Services::header_reads_sent(&self) -> u64`.

- [x] **Step 1: Write the failing tests**

Append to `crates/fp-app/tests/services.rs`:

```rust
/// Operator feedback 4, Q1.1: a track has the length its file's header
/// declares long before its analysis ends.
#[test]
fn a_track_gets_its_header_duration_before_its_analysis() {
    let dir = tempfile::tempdir().unwrap();
    let file = wav(dir.path(), "a.wav", 2);
    let mut r = rig_slow(&[file], dir, Duration::from_secs(30));
    r.run_until("the header duration", |r| only_track(r).duration_secs > 0.0);
    let t = only_track(&r);
    assert!(!t.analyzed, "the analysis is still running");
    assert!((t.duration_secs - 2.0).abs() < 1e-6, "{}", t.duration_secs);
}

/// A file whose header cannot be read gets no length, and is asked once.
#[test]
fn a_file_whose_header_cannot_be_read_gets_no_duration_and_is_asked_once() {
    let dir = tempfile::tempdir().unwrap();
    let broken = dir.path().join("broken.wav");
    std::fs::write(&broken, b"RIFF\x00\x00\x00\x00WAVEjunkjunkjunkjunk").unwrap();
    let mut r = rig_slow(&[broken], dir, Duration::from_secs(30));
    let mut rounds = 0;
    r.run_until("fifty rounds", |_| {
        rounds += 1;
        rounds > 50
    });
    assert_eq!(only_track(&r).duration_secs, 0.0);
    assert_eq!(r.services.header_reads_sent(), 1);
}

/// Tracks on a player get their length ahead of a large library.
#[test]
fn a_track_on_a_player_gets_its_header_duration_ahead_of_the_library() {
    let dir = tempfile::tempdir().unwrap();
    let files: Vec<PathBuf> = (1..=30)
        .map(|n| wav(dir.path(), &format!("{n}.wav"), 1))
        .collect();
    let mut r = rig_with(&files, dir, Duration::from_secs(30), |s| {
        let p = s.players[0].id;
        let last = s.playlists.iter().next().unwrap().entries[29].id;
        fp_model::apply(s, Command::SetNext(p, last)).unwrap();
    });
    let model = r.handle.model.load_full();
    let next = model.players[0].next.unwrap();
    let track = model.playlists.entry(next).unwrap().track;
    r.run_until("the loaded track's length", |r| {
        r.handle
            .model
            .load()
            .library
            .get(track)
            .is_some_and(|t| t.duration_secs > 0.0)
    });
    let known = r
        .handle
        .model
        .load()
        .library
        .iter()
        .filter(|t| t.duration_secs > 0.0)
        .count();
    // At most two rounds of eight reads were answered by then.
    assert!(known <= 16, "{known} lengths were read first");
}
```

- [x] **Step 2: Run them to make sure they fail**

Run: `cargo test -p fp-app --test services header`

Expected: a compile error, because there is no method `header_reads_sent`.

- [x] **Step 3: Implement the reader**

`crates/fp-app/src/header.rs`:

```rust
//! The header reader (operator feedback 4, Q1.1): one thread that opens a
//! file's header and reads how long it is, so a track has a length, a
//! countdown and click-to-seek before its analysis. It follows the file
//! probe's pattern: jobs in, answers out, both on unbounded channels, and
//! the thread ends when its handle is dropped.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};

use crossbeam_channel::{Receiver, Sender};
use fp_decode::FileDecoder;
use fp_model::TrackId;

/// The length `path`'s header declares, in seconds. `None` when the file
/// cannot be opened or its container does not say.
pub fn header_duration(path: &Path) -> Option<f64> {
    FileDecoder::open(path).ok()?.duration_hint_secs()
}

/// The reader thread's handle.
pub struct HeaderReader {
    jobs: Sender<(TrackId, PathBuf)>,
    answers: Receiver<(TrackId, Option<f64>)>,
}

impl HeaderReader {
    pub fn spawn() -> std::io::Result<Self> {
        let (jobs, inbox) = crossbeam_channel::unbounded::<(TrackId, PathBuf)>();
        let (outbox, answers) = crossbeam_channel::unbounded();
        std::thread::Builder::new()
            .name("fp-header-reader".to_owned())
            .spawn(move || {
                // Ends when the services drop their side.
                while let Ok((track, path)) = inbox.recv() {
                    // A decoder that panics on a broken file costs that
                    // file's length, never the thread.
                    let secs = catch_unwind(AssertUnwindSafe(|| header_duration(&path)))
                        .unwrap_or_else(|_| {
                            tracing::warn!(path = %path.display(), "reading the file header panicked");
                            None
                        });
                    if outbox.send((track, secs)).is_err() {
                        break;
                    }
                }
            })?;
        Ok(Self { jobs, answers })
    }

    /// Queues a read; `false` once the thread has stopped.
    pub fn submit(&self, track: TrackId, path: PathBuf) -> bool {
        self.jobs.send((track, path)).is_ok()
    }

    /// One answer per read: the length, or `None` when there is none.
    pub fn answers(&self) -> &Receiver<(TrackId, Option<f64>)> {
        &self.answers
    }
}
```

- [x] **Step 4: Wire it into `Services`**

In `crates/fp-app/src/services.rs`:
- add `use crate::header::HeaderReader;` next to `use crate::tags::…`;
- add the constant after `PERIOD`;
- add the fields to `struct Services`, after `tags_answered`;
- set them in `new`.

```rust
/// How many header reads wait on the reader at once: few enough that a
/// track loaded on a player goes ahead of a large folder just added.
const HEADER_READS_IN_FLIGHT: usize = 8;
```

```rust
    /// Reads the length of tracks not analysed yet (operator feedback 4,
    /// Q1.1); `None` if its thread could not start.
    header_reader: Option<HeaderReader>,
    /// Tracks whose header was asked for (answered or not): once each.
    headers_asked: HashSet<TrackId>,
    /// Reads sent and not answered yet.
    headers_in_flight: usize,
    /// Reads sent since the start (a test hook reads it).
    header_reads_sent: u64,
```

```rust
            header_reader: HeaderReader::spawn()
                .map_err(|e| {
                    tracing::error!(error = %e, "cannot start the header reader; lengths wait for the analysis");
                })
                .ok(),
            headers_asked: HashSet::new(),
            headers_in_flight: 0,
            header_reads_sent: 0,
```

In `analysis_step`, after `self.submit_new(state);`, add `self.header_pass(state);`.

After `tag_pass`:

```rust
    /// Operator feedback 4, Q1.1: tracks with no length and no analysis get
    /// the length their file's header declares, read on the header reader,
    /// the ones on a player first. Each track is asked once per session,
    /// unless its command could not be queued.
    fn header_pass(&mut self, state: &AppState) {
        let Some(reader) = self.header_reader.as_ref() else {
            return;
        };
        let answers: Vec<(TrackId, Option<f64>)> = reader.answers().try_iter().collect();
        for (track, secs) in answers {
            self.headers_in_flight = self.headers_in_flight.saturating_sub(1);
            if let Some(secs) = secs
                && !self.conductor.send(Command::SetDuration { track, secs })
            {
                // The queue is full: ask again on a later round.
                self.headers_asked.remove(&track);
            }
        }
        self.headers_asked
            .retain(|id| state.library.get(*id).is_some());
        let wanted = Self::wanted(state);
        let mut todo: Vec<&fp_model::Track> = state
            .library
            .iter()
            .filter(|t| t.needs_header_duration() && !self.headers_asked.contains(&t.id))
            .collect();
        // The tracks on a player first; the sort is stable.
        todo.sort_by_key(|t| !wanted.contains(&t.id));
        let room = HEADER_READS_IN_FLIGHT.saturating_sub(self.headers_in_flight);
        for track in todo.into_iter().take(room) {
            if !reader.submit(track.id, track.path.clone()) {
                tracing::error!("the header reader stopped; lengths wait for the analysis");
                self.header_reader = None;
                return;
            }
            self.headers_asked.insert(track.id);
            self.headers_in_flight += 1;
            self.header_reads_sent += 1;
        }
    }
```

If the borrow checker refuses `self.header_reader = None` while `reader` is borrowed, take the reader out first and put it back at the end. Use `let Some(reader) = self.header_reader.take() else { return; };`, then `self.header_reader = Some(reader);` on every path that keeps it.

After `tag_pass_on`:

```rust
    /// How many header reads were sent. Used to test that a track is
    /// asked once.
    #[cfg(feature = "test-hooks")]
    pub fn header_reads_sent(&self) -> u64 {
        self.header_reads_sent
    }
```

`fp-app` already depends on `fp-decode` (`crates/fp-app/Cargo.toml`), so there is no dependency change.

- [x] **Step 5: Run the tests to make sure they pass**

Run: `cargo test -p fp-app --test services`

Expected: PASS, the three new tests and the existing ones.

- [x] **Step 6: Docs**

In `docs/technical/analysis.md`, §"How the app uses it", after the paragraph about the probe thread (`fp-file-probe`), add:

```markdown
The **header reader** (`fp-header-reader`, `fp-app/src/header.rs`, driven
by `Services::header_pass`) gives a track a length before its analysis
(operator feedback 4, Q1.1). Each track with no length, no analysis and a
playable file (`Track::needs_header_duration`) is read once: the file's
header only (`FileDecoder::duration_hint_secs`), the tracks on a player
first, at most eight reads waiting at a time. A length goes to the model as
`Command::SetDuration`; a file that has none (or cannot be opened) gets
nothing and waits for its analysis. A panic while reading is caught and
logged.
```

- [x] **Step 7: Commit**

```sh
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace \
&& git add crates/fp-app/src/header.rs crates/fp-app/src/lib.rs crates/fp-app/src/services.rs crates/fp-app/tests/services.rs docs/technical/analysis.md \
&& git commit -m "feat(app): read each new track's length from its file header" -m "Q1.1: a header reader thread, off the UI and services threads, gives tracks a length before their analysis, the ones on a player first." -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 6: Countdown, time row and seek for a track not analysed yet (view and UI, Q1.3 and Q1.4)

**Suggested model:** `sonnet` (a small pure helper and kittests).

**Files:**
- Modify: `crates/fp-app/src/ui/view.rs` (a new `time_text` after `cue_follow_target`, ~L159–163)
- Modify: `crates/fp-app/src/ui/player.rs`: `time_row` (~L503–518), which gains a `scene` parameter, and its call in the column body (~L85), which becomes `time_row(ui, scene, &pv);`
- Test: `crates/fp-app/tests/view.rs` (append, and add `time_text` to its `use fp_app::ui::view::{…}`), `crates/fp-app/tests/waveform_ui.rs` (append)
- Docs: `docs/user/players.md` (the Info row "Title and artist" bullet, ~L22–26, and the Waveform click bullet, ~L127–129)
- Locales (`en-US`/`es-ES`): none (`placeholder-none` is reused).

**Interfaces:**
- Consumes: `Command::SetDuration` (Task 4).
- Produces: `pub fn fp_app::ui::view::time_text(elapsed: f64, total: Option<f64>, unknown: &str) -> String`.

- [x] **Step 1: Write the failing tests**

Append to `crates/fp-app/tests/view.rs`:

```rust
/// Operator feedback 4, Q1.3: with a length from its header, a track not
/// analysed yet has its countdown, total and position.
#[test]
fn q1_3_a_track_with_a_header_duration_has_its_countdown_and_total() {
    let (mut s, e, p) = state(2);
    let t0 = s.playlists.entry(e[0]).unwrap().track;
    s.library.get_mut(t0).unwrap().duration_secs = 0.0;
    apply(&mut s, Command::SetDuration { track: t0, secs: 120.0 }).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    let v = player_view(&s, p, Some(30.0), 0.0).unwrap();
    assert_eq!((v.elapsed, v.total, v.remaining), (30.0, Some(120.0), 90.0));
    assert_eq!(v.markers.position, Some(0.25));
}

/// Q1.4: without a header length, the elapsed time shows and the total
/// is unknown.
#[test]
fn q1_4_without_a_header_duration_the_total_is_unknown() {
    let (mut s, e, p) = state(2);
    let t0 = s.playlists.entry(e[0]).unwrap().track;
    s.library.get_mut(t0).unwrap().duration_secs = 0.0;
    apply(&mut s, Command::Play(p)).unwrap();
    let v = player_view(&s, p, Some(30.0), 0.0).unwrap();
    assert_eq!((v.elapsed, v.total), (30.0, None));
    assert_eq!(v.markers.position, None);
}

#[test]
fn the_time_row_shows_a_dash_for_an_unknown_total() {
    assert_eq!(time_text(30.0, None, "—"), "00:30 / —");
    assert_eq!(time_text(30.0, Some(f64::NAN), "—"), "00:30 / —");
    assert_eq!(time_text(30.0, Some(120.0), "—"), "00:30 / 02:00");
}
```

Append to `crates/fp-app/tests/waveform_ui.rs`:

```rust
/// P1 plays a track with no analysis; `header` is the length its file's
/// header declared, if any (operator feedback 4, Q1).
fn playing_before_analysis(header: Option<f64>) -> AppState {
    let mut s = state(1, 2);
    let e = s.playlists.iter().next().unwrap().entries[0].clone();
    if let Some(secs) = header {
        apply(&mut s, Command::SetDuration { track: e.track, secs }).unwrap();
    }
    let p = s.players[0].id;
    apply(&mut s, Command::Play(p)).unwrap();
    s
}

#[test]
fn q1_3_a_click_on_the_wave_box_of_a_track_not_analysed_yet_seeks() {
    let (mut h, fake) = harness(playing_before_analysis(Some(180.0)));
    assert!(h.query_by_label("00:00 / 03:00").is_some());
    let w = wave(&h);
    click(&mut h, pos2(x_of(w, 90.0), w.center().y));
    let s = seeks(&fake);
    assert_eq!(s.len(), 1, "{s:?}");
    assert!((s[0] - 90.0).abs() < 1.5, "{s:?}");
}

#[test]
fn q1_4_without_a_length_the_wave_box_does_not_seek_and_the_total_is_a_dash() {
    let (mut h, fake) = harness(playing_before_analysis(None));
    assert!(h.query_by_label("00:00 / —").is_some());
    let w = wave(&h);
    click(&mut h, w.center());
    assert!(seeks(&fake).is_empty());
}
```

- [x] **Step 2: Run them to make sure they fail**

Run: `cargo test -p fp-app --test view q1_` and `cargo test -p fp-app --test waveform_ui q1_`

Expected:
- `view`: a compile error, because `time_text` is not defined;
- `waveform_ui`: `q1_4_…` fails on the "—" label (it reads "00:00 / 00:00").

The two `q1_3` tests may already pass: the view and the widget use `duration_secs`, which Task 4 lets `SetDuration` fill. That is expected, and the tests pin it.

- [x] **Step 3: Implement**

`crates/fp-app/src/ui/view.rs`, after `cue_follow_target`:

```rust
/// The `elapsed / total` line under the waveform. `unknown` stands for a
/// total that is not known (operator feedback 4, Q1.4: a track played
/// before its analysis whose header gave no length).
pub fn time_text(elapsed: f64, total: Option<f64>, unknown: &str) -> String {
    let total = total
        .filter(|t| t.is_finite() && *t > 0.0)
        .map_or_else(|| unknown.to_owned(), super::format::clock);
    format!("{} / {}", super::format::clock(elapsed), total)
}
```

`crates/fp-app/src/ui/player.rs`, replace `time_row`:

```rust
/// `elapsed / total` under the waveform, right-aligned, close to it.
fn time_row(ui: &mut Ui, scene: &Scene<'_>, pv: &PlayerView) {
    ui.add_space(-4.0);
    let text = view::time_text(pv.elapsed, pv.total, &scene.i18n.tr("placeholder-none"));
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), 14.0),
        Layout::right_to_left(Align::Center),
        |ui| {
            widgets::tabular_label(ui, &text, &font(12.0), theme::NEUTRAL_400);
        },
    );
}
```

Change its call to `time_row(ui, scene, &pv);`.

- [x] **Step 4: Run the tests to make sure they pass**

Run: `cargo test -p fp-app --test view`, then `cargo test -p fp-app --test waveform_ui`, then `cargo test -p fp-app --test main_screen`

Expected: PASS. If a test of `main_screen.rs` looked for "00:00 / 00:00", it now finds "00:00 / —" and must be changed to that text. Find any such test with `grep -rn '00:00 / 00:00' crates/fp-app/tests`.

- [x] **Step 5: Docs**

In `docs/user/players.md`, in the Info row, after the "Title and artist" bullet, add:

```markdown
- A track played or loaded before its analysis has finished already has its
  length when its file's header gives it: the countdown, `elapsed / total`
  and click to jump work at once, over a flat line until the waveform is
  ready. When the header does not give it (some MP3 files), the total reads
  "—" and the waveform cannot be clicked until the analysis ends.
```

- [x] **Step 6: Commit**

```sh
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace \
&& git add crates/fp-app/src/ui/view.rs crates/fp-app/src/ui/player.rs crates/fp-app/tests/view.rs crates/fp-app/tests/waveform_ui.rs docs/user/players.md \
&& git commit -m "feat(ui): countdown and seek before the analysis; an unknown total reads a dash" -m "Q1.3 and Q1.4." -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 7: Pending start (model, rule 3a, Q8.2–Q8.5 and Q8.7) and its engine test

**Suggested model:** `opus` (reducer invariants across commands, events and reconcile).

**Files:**
- Create: `crates/fp-model/src/pending_start.rs`
- Modify: `crates/fp-model/src/lib.rs` (add `mod pending_start;` after `mod on_air`)
- Modify: `crates/fp-model/src/player.rs` (`PlayerState` ~L106–138 and `PlayerState::new` ~L146–166)
- Modify: `crates/fp-model/src/reducer.rs`:
  - the `Command::Seek` arm (~L122–133);
  - the `Command::MoveEntry` arm (~L183–186);
  - `play` (~L422–447);
  - `stop` (~L503–512);
  - `set_next` (~L540–548);
  - `reconcile` (~L1255–1307).
- Test: `crates/fp-model/tests/pending_start.rs` (create), `crates/fp-engine/tests/engine.rs` (append)
- Docs: `docs/technical/audio-engine.md` (the `Preload` row, ~L325); the user docs are in Task 9
- Locales (`en-US`/`es-ES`): none.

**Interfaces:**
- Consumes: `AppState::{playable_request, track_for_entry, request_at, request_from_cue_in}`, `Track::play_range`.
- Produces:
  - `pub pending_start: Option<(EntryId, f64)>` on `PlayerState`;
  - in `crate::pending_start`:
    - `pub(crate) fn pending_start_at(state: &AppState, entry: EntryId, secs: f64) -> Option<(EntryId, f64)>`;
    - `pub(crate) fn settle(state: &mut AppState)`;
    - `pub(crate) fn start_request(state: &AppState, pending: Option<(EntryId, f64)>, request: SourceRequest) -> SourceRequest`;
    - `pub(crate) fn preload_request(state: &AppState, player: &PlayerState, entry: EntryId) -> Option<SourceRequest>`.

- [ ] **Step 1: Write the failing model tests**

`crates/fp-model/tests/pending_start.rs`:

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::float_cmp)]
//! Rule 3a, pending start (operator feedback 4, Q8): a seek on a stopped
//! player chooses where Play starts its next entry.

mod common;

use common::{entries, fixture, p0};
use fp_model::{
    AppState, Command, EngineAction, EngineEvent, EntryId, MarkerKind, PlayerId, TrackAnalysis,
    Transport, apply, on_event,
};

fn set_cue_in(s: &mut AppState, entry: EntryId, secs: f64) {
    let t = s.playlists.entry(entry).unwrap().track;
    s.library
        .get_mut(t)
        .unwrap()
        .markers
        .set_auto(MarkerKind::CueIn, Some(secs));
}

fn starts(actions: &[EngineAction]) -> Vec<(EntryId, f64)> {
    actions
        .iter()
        .filter_map(|a| match a {
            EngineAction::StartCurrent { request, .. } | EngineAction::Crossfade { request, .. } => {
                Some((request.entry, request.from_secs))
            }
            _ => None,
        })
        .collect()
}

fn preloads(actions: &[EngineAction], p: PlayerId) -> Vec<(EntryId, f64)> {
    actions
        .iter()
        .filter_map(|a| match a {
            EngineAction::Preload {
                player,
                request: Some(r),
            } if *player == p => Some((r.entry, r.from_secs)),
            _ => None,
        })
        .collect()
}

fn pending(s: &AppState, p: PlayerId) -> Option<(EntryId, f64)> {
    s.player(p).unwrap().pending_start
}

#[test]
fn q8_2_a_seek_while_stopped_stores_where_play_starts_the_next_and_starts_nothing() {
    let mut s = fixture(3);
    let (e, p) = (entries(&s), p0(&s));
    let actions = apply(&mut s, Command::Seek(p, 30.0)).unwrap();
    assert_eq!(pending(&s, p), Some((e[0], 30.0)));
    let player = s.player(p).unwrap();
    assert_eq!((player.transport, player.current), (Transport::Stopped, None));
    assert!(
        actions.iter().all(|a| !matches!(
            a,
            EngineAction::StartCurrent { .. }
                | EngineAction::Seek { .. }
                | EngineAction::LoadPaused { .. }
                | EngineAction::Resume { .. }
        )),
        "{actions:?}"
    );
}

#[test]
fn q8_2_the_pending_start_is_raised_to_the_cue_in() {
    let mut s = fixture(3);
    let (e, p) = (entries(&s), p0(&s));
    set_cue_in(&mut s, e[0], 2.0);
    apply(&mut s, Command::Seek(p, 1.0)).unwrap();
    assert_eq!(pending(&s, p), Some((e[0], 2.0)));
}

#[test]
fn q8_2_a_seek_at_or_past_the_end_of_the_play_range_stores_nothing() {
    let mut s = fixture(3);
    let p = p0(&s);
    apply(&mut s, Command::Seek(p, 30.0)).unwrap();
    apply(&mut s, Command::Seek(p, 180.0)).unwrap();
    assert_eq!(pending(&s, p), None, "Play would end the track at once");
    apply(&mut s, Command::Seek(p, 500.0)).unwrap();
    assert_eq!(pending(&s, p), None);
}

#[test]
fn q8_2_a_broken_seek_changes_nothing() {
    let mut s = fixture(3);
    let (e, p) = (entries(&s), p0(&s));
    apply(&mut s, Command::Seek(p, 30.0)).unwrap();
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        apply(&mut s, Command::Seek(p, bad)).unwrap();
        assert_eq!(pending(&s, p), Some((e[0], 30.0)), "{bad}");
    }
}

#[test]
fn q8_2_without_a_next_a_seek_does_nothing() {
    let mut s = fixture(1);
    let (e, p) = (entries(&s), p0(&s));
    apply(&mut s, Command::RemoveEntry(e[0])).unwrap();
    assert_eq!(s.player(p).unwrap().next, None);
    apply(&mut s, Command::Seek(p, 10.0)).unwrap();
    assert_eq!(pending(&s, p), None);
}

#[test]
fn q8_2_a_next_whose_file_cannot_be_played_gets_no_pending_start() {
    let mut s = fixture(3);
    let (e, p) = (entries(&s), p0(&s));
    let track = s.playlists.entry(e[0]).unwrap().track;
    apply(
        &mut s,
        Command::SetFileState {
            track,
            state: fp_model::FileState::Missing,
        },
    )
    .unwrap();
    apply(&mut s, Command::Seek(p, 10.0)).unwrap();
    assert_eq!(pending(&s, p), None);
}

#[test]
fn q8_3_play_starts_the_next_at_its_pending_start_once() {
    let mut s = fixture(3);
    let (e, p) = (entries(&s), p0(&s));
    apply(&mut s, Command::Seek(p, 30.0)).unwrap();
    let actions = apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(starts(&actions), vec![(e[0], 30.0)]);
    assert_eq!(pending(&s, p), None, "used");
    apply(&mut s, Command::Stop(p)).unwrap();
    apply(&mut s, Command::SetNext(p, e[0])).unwrap();
    let actions = apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(starts(&actions), vec![(e[0], 0.0)], "only once");
}

#[test]
fn q8_4_changing_the_next_clears_the_pending_start() {
    let mut s = fixture(3);
    let (e, p) = (entries(&s), p0(&s));
    apply(&mut s, Command::Seek(p, 30.0)).unwrap();
    apply(&mut s, Command::SetNext(p, e[1])).unwrap();
    assert_eq!(pending(&s, p), None);
}

#[test]
fn q8_4_setting_the_same_next_again_clears_it_too() {
    let mut s = fixture(3);
    let (e, p) = (entries(&s), p0(&s));
    apply(&mut s, Command::Seek(p, 30.0)).unwrap();
    apply(&mut s, Command::SetNext(p, e[0])).unwrap();
    assert_eq!(pending(&s, p), None);
}

#[test]
fn q8_4_removing_its_entry_clears_it() {
    let mut s = fixture(3);
    let (e, p) = (entries(&s), p0(&s));
    apply(&mut s, Command::Seek(p, 30.0)).unwrap();
    apply(&mut s, Command::RemoveEntry(e[0])).unwrap();
    assert_eq!(pending(&s, p), None);
}

#[test]
fn q8_4_moving_its_entry_clears_it() {
    let mut s = fixture(3);
    let (e, p) = (entries(&s), p0(&s));
    let playlist = s.playlists.first_id().unwrap();
    apply(&mut s, Command::Seek(p, 30.0)).unwrap();
    apply(
        &mut s,
        Command::MoveEntry {
            entry: e[0],
            to: playlist,
            index: 2,
        },
    )
    .unwrap();
    assert_eq!(pending(&s, p), None);
}

#[test]
fn q8_4_stop_clears_it() {
    let mut s = fixture(3);
    let p = p0(&s);
    apply(&mut s, Command::Seek(p, 30.0)).unwrap();
    apply(&mut s, Command::Stop(p)).unwrap();
    assert_eq!(pending(&s, p), None);
}

#[test]
fn q8_4_play_now_of_another_entry_starts_it_at_its_cue_in() {
    let mut s = fixture(3);
    let (e, p) = (entries(&s), p0(&s));
    apply(&mut s, Command::Seek(p, 30.0)).unwrap();
    apply(&mut s, Command::SetNext(p, e[2])).unwrap();
    let actions = apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(starts(&actions), vec![(e[2], 0.0)]);
}

#[test]
fn q8_5_restart_the_automatic_advance_and_previous_keep_the_cue_in() {
    let mut s = fixture(3);
    let (e, p) = (entries(&s), p0(&s));
    set_cue_in(&mut s, e[0], 2.0);
    apply(&mut s, Command::Seek(p, 30.0)).unwrap();
    assert_eq!(starts(&apply(&mut s, Command::Play(p)).unwrap()), vec![(e[0], 30.0)]);
    // Restart goes back to the cue-in, not to the pending start.
    let actions = apply(&mut s, Command::Restart(p)).unwrap();
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EngineAction::Seek { secs, .. } if *secs == 2.0)),
        "{actions:?}"
    );
    // The automatic advance: the engine starts e1; e2 is preloaded at its cue-in.
    let actions = on_event(
        &mut s,
        EngineEvent::TransitionStarted {
            player: p,
            entry: e[1],
        },
    );
    assert_eq!(s.player(p).unwrap().current, Some(e[1]));
    assert_eq!(preloads(&actions, p), vec![(e[2], 0.0)]);
    // Previous goes back to e0 from its cue-in.
    let actions = apply(&mut s, Command::Previous(p)).unwrap();
    assert_eq!(starts(&actions), vec![(e[0], 2.0)]);
}

#[test]
fn q8_7_the_next_is_preloaded_at_the_pending_start_and_again_when_it_changes() {
    let mut s = fixture(3);
    let (e, p) = (entries(&s), p0(&s));
    let actions = apply(&mut s, Command::Seek(p, 30.0)).unwrap();
    assert_eq!(preloads(&actions, p), vec![(e[0], 30.0)]);
    let actions = apply(&mut s, Command::Seek(p, 45.0)).unwrap();
    assert_eq!(preloads(&actions, p), vec![(e[0], 45.0)]);
    let actions = apply(&mut s, Command::SetNext(p, e[1])).unwrap();
    assert_eq!(preloads(&actions, p), vec![(e[1], 0.0)]);
}

#[test]
fn q8_7_play_starts_exactly_where_the_preload_is() {
    let mut s = fixture(3);
    let p = p0(&s);
    let preloaded = preloads(&apply(&mut s, Command::Seek(p, 30.0)).unwrap(), p);
    let started = starts(&apply(&mut s, Command::Play(p)).unwrap());
    assert_eq!(preloaded, started, "the engine matches them exactly");
}

#[test]
fn q8_8_a_pending_start_never_goes_on_air_by_itself() {
    let mut s = fixture(3);
    let (e, p) = (entries(&s), p0(&s));
    apply(&mut s, Command::Seek(p, 30.0)).unwrap();
    let track = s.playlists.entry(e[0]).unwrap().track;
    let mut actions = apply(&mut s, Command::SetVolume(p, 0.5)).unwrap();
    actions.extend(
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
        .unwrap(),
    );
    actions.extend(on_event(&mut s, EngineEvent::FadeCompleted { player: p }));
    assert!(starts(&actions).is_empty(), "{actions:?}");
    assert_eq!(s.player(p).unwrap().transport, Transport::Stopped);
    assert_eq!(pending(&s, p), Some((e[0], 30.0)));
}

#[test]
fn a_pending_start_is_clamped_again_when_the_analysis_moves_the_cue_in() {
    let mut s = fixture(3);
    let (e, p) = (entries(&s), p0(&s));
    apply(&mut s, Command::Seek(p, 1.0)).unwrap();
    let track = s.playlists.entry(e[0]).unwrap().track;
    let actions = apply(
        &mut s,
        Command::ApplyAnalysis {
            track,
            analysis: Box::new(TrackAnalysis {
                duration_secs: 180.0,
                cue_in: Some(4.0),
                ..TrackAnalysis::default()
            }),
        },
    )
    .unwrap();
    assert_eq!(pending(&s, p), Some((e[0], 4.0)));
    assert_eq!(preloads(&actions, p), vec![(e[0], 4.0)]);
}
```

If `TrackAnalysis` has no `Default` derive in this branch, build it the way `crates/fp-model/tests/analysis.rs` does.

- [ ] **Step 2: Write the engine test (it pins existing behaviour)**

Append to `crates/fp-engine/tests/engine.rs`:

```rust
/// An opener that counts the sources it opens.
fn counting_opener(opened: Arc<std::sync::atomic::AtomicUsize>) -> fp_engine::worker::SourceOpener {
    let tagged = support::tagged_opener(96_000);
    Arc::new(move |path, from_secs, rate| {
        opened.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        tagged(path, from_secs, rate)
    })
}

/// Operator feedback 4, Q8.7: Play from a pending start uses the source
/// preloaded there; nothing is opened again. (Characterisation: the
/// engine already matches a preload on entry and start.)
#[test]
fn play_from_a_pending_start_uses_the_preload_made_there() {
    for (start_at, opens) in [(1.0, 1), (2.0, 2)] {
        let opened = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let mut r = rig_with(counting_opener(Arc::clone(&opened)), false);
        r.act(EngineAction::Preload {
            player: P,
            request: Some(request(1, 1.0)),
        });
        r.act(EngineAction::StartCurrent {
            player: P,
            request: request(1, start_at),
        });
        r.settle();
        r.run(4);
        assert_eq!(
            opened.load(std::sync::atomic::Ordering::SeqCst),
            opens,
            "start at {start_at}"
        );
        assert!(r.position() >= start_at, "{}", r.position());
        assert!((tag(*r.heard.last().unwrap()).1 as f64 / RATE) >= start_at);
    }
}
```

The second pass (start at 2.0 against a preload at 1.0) is the control: it opens a second source, which proves that the counter sees a fresh open.

- [ ] **Step 3: Run them to make sure the model tests fail**

Run: `cargo test -p fp-model --test pending_start`

Expected: a compile error, because there is no field `pending_start`.

Run: `cargo test -p fp-engine --test engine play_from_a_pending_start_uses_the_preload_made_there`

Expected: PASS. This is a characterisation test (see the rulings). If it fails, stop: the engine does not reuse the preload as `take_or_open` suggests, and Q8.7 needs an engine change. Record that in the ledger before you go on.

- [ ] **Step 4: Implement the module**

`crates/fp-model/src/pending_start.rs`:

```rust
//! Rule 3a, pending start (operator feedback 4, Q8): where Play starts the
//! next entry of a stopped player, when the operator chose it with a seek.
//! A pending start is not on air: it only changes where Play starts.

use crate::command::SourceRequest;
use crate::ids::EntryId;
use crate::player::{PlayerState, Transport};
use crate::state::AppState;

/// Q8.2: the pending start that a seek to `secs` stores on a stopped player
/// whose next is `entry`, raised to the entry's cue-in. `None` for a
/// broken `secs`, an entry whose file cannot be played, or a start at or
/// past the end of the play range (Play would end the track at once).
pub(crate) fn pending_start_at(
    state: &AppState,
    entry: EntryId,
    secs: f64,
) -> Option<(EntryId, f64)> {
    if !secs.is_finite() {
        return None;
    }
    state.playable_request(entry)?;
    let track = state.track_for_entry(entry)?;
    let range = track.play_range(state.config.players.use_cue_markers);
    let at = secs.max(range.cue_in);
    if range.known_end().is_some_and(|end| at >= end) {
        return None;
    }
    Some((entry, at))
}

/// Q8.4: a pending start lives only while its player is stopped and its
/// entry is the player's next. It is clamped again to the entry's current
/// play range, which an analysis or a marker may have moved.
pub(crate) fn settle(state: &mut AppState) {
    let changes: Vec<(usize, Option<(EntryId, f64)>)> = state
        .players
        .iter()
        .enumerate()
        .filter_map(|(i, p)| {
            let (entry, secs) = p.pending_start?;
            let kept = (p.transport == Transport::Stopped && p.next == Some(entry))
                .then(|| pending_start_at(state, entry, secs))
                .flatten();
            (kept != p.pending_start).then_some((i, kept))
        })
        .collect();
    for (i, kept) in changes {
        state.players[i].pending_start = kept;
    }
}

/// Q8.3: what Play while Stopped sends for `request`: the same entry at the
/// pending start taken from the player, when that start is for it.
pub(crate) fn start_request(
    state: &AppState,
    pending: Option<(EntryId, f64)>,
    request: SourceRequest,
) -> SourceRequest {
    match pending {
        Some((entry, secs)) if entry == request.entry => {
            state.request_at(entry, secs).unwrap_or(request)
        }
        _ => request,
    }
}

/// Q8.7: where the engine preloads `entry` for `player`: at the pending
/// start while the player is stopped with one for it, else at the cue-in.
pub(crate) fn preload_request(
    state: &AppState,
    player: &PlayerState,
    entry: EntryId,
) -> Option<SourceRequest> {
    match player.pending_start {
        Some((e, secs)) if e == entry && player.transport == Transport::Stopped => {
            state.request_at(e, secs)
        }
        _ => state.request_from_cue_in(entry),
    }
}
```

The preload and the start build their request with the same call, `state.request_at(entry, secs)` on the same stored `secs`. So their `from_secs` are equal, and the engine's `take_or_open` reuses the preload.

- [ ] **Step 5: Wire it into the state and the reducer**

`crates/fp-model/src/player.rs`: add the field to `PlayerState` after `history`:

```rust
    /// Rule 3a (operator feedback 4, Q8): where Play starts the `next`
    /// entry from Stopped, when the operator chose it with a seek. Only
    /// while the player is stopped and the entry is its next.
    pub pending_start: Option<(EntryId, f64)>,
```

Add `pending_start: None,` in `PlayerState::new`. Add `EntryId` to the imports if it is not there yet.

`crates/fp-model/src/lib.rs`: add `mod pending_start;`.

`crates/fp-model/src/reducer.rs`, replace the `Command::Seek` arm:

```rust
        Command::Seek(id, secs) => {
            let i = state.player_index(id)?;
            let player = &state.players[i];
            if player.transport == Transport::Stopped {
                // Rule 3a (Q8.2): where Play will start the next. Nothing
                // goes to the engine but the preload (`reconcile`).
                if let Some(next) = player.next
                    && secs.is_finite()
                {
                    let at = crate::pending_start::pending_start_at(state, next, secs);
                    state.players[i].pending_start = at;
                }
            } else if let Some(request) = player.current.and_then(|c| state.request_at(c, secs)) {
                out.push(EngineAction::Seek {
                    player: id,
                    secs: request.from_secs,
                });
            }
        }
```

`Command::MoveEntry` arm, before `refresh_next(state);`:

```rust
            // Q8.4: a moved entry loses the start chosen for it.
            for p in &mut state.players {
                if p.pending_start.is_some_and(|(e, _)| e == entry) {
                    p.pending_start = None;
                }
            }
```

`play`, the `Transport::Paused | Transport::Stopped` arm:

```rust
        Transport::Paused | Transport::Stopped => {
            // Q8.3: the pending start is used by this start, once.
            let pending = state.players[i].pending_start.take();
            if let Some(request) = advance(state, i) {
                let request = crate::pending_start::start_request(state, pending, request);
                out.push(EngineAction::StartCurrent {
                    player: id,
                    request,
                });
            }
        }
```

`stop`, as its first lines after `let i = state.player_index(id)?;`:

```rust
    // Q8.4: Stop forgets a pending start, even on a stopped player.
    state.players[i].pending_start = None;
```

`set_next`, before `Ok(())`:

```rust
    // Q8.4: a new next, or the same one chosen again (Play now), starts at
    // its cue-in.
    state.players[i].pending_start = None;
```

`reconcile`, as its first statement:

```rust
    crate::pending_start::settle(state);
```

In the `preloads` map, replace `preload_target(state, p).and_then(|e| state.request_from_cue_in(e))` with:

```rust
                preload_target(state, p)
                    .and_then(|e| crate::pending_start::preload_request(state, p, e)),
```

- [ ] **Step 6: Run the tests to make sure they pass**

Run: `cargo test -p fp-model --test pending_start`, then `cargo test -p fp-model`, then `cargo test -p fp-engine --test engine`

Expected: PASS. `crates/fp-model/tests/session.rs` does not compile until Task 8 adds `pending_start` to `PlayerSession`. That is fine: it only constructs `PlayerSession`, not `PlayerState`. If it does fail here, the cause is something else; investigate.

- [ ] **Step 7: Docs**

In `docs/technical/audio-engine.md`, in the `Preload` row of the action table, after "ask the worker for a source of the next entry at its cue-in;", add:

```markdown
at its pending start instead while the player is stopped and the operator chose one (rule 3a, operator feedback 4 Q8.7); `StartCurrent` then asks for the same entry and start, so `take_or_open` uses that preload and Play starts without a gap;
```

- [ ] **Step 8: Commit**

```sh
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace \
&& git add crates/fp-model crates/fp-engine/tests/engine.rs docs/technical/audio-engine.md \
&& git commit -m "feat(model): a seek on a stopped player sets where Play starts" -m "Rule 3a (operator feedback 4, Q8.2-Q8.5, Q8.7): the pending start is stored, preloaded, used once by Play and cleared when the next changes, its entry moves, on Stop or by another start." -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 8: The pending start in the session (Q8.8)

**Suggested model:** `opus` (restore invariants; nothing may go on air).

**Files:**
- Modify: `crates/fp-model/src/session.rs`:
  - `PlayerSession` (~L15–38);
  - a lenient deserializer next to `lenient_history` (~L54–66);
  - `sessions` (~L85–106);
  - `restore_player` (~L229–277).
- Modify: `crates/fp-model/tests/session.rs` (the two `PlayerSession { … }` literals, ~L51 and ~L96: add `pending_start: None,`)
- Test: `crates/fp-model/tests/pending_start.rs` (append)
- Docs: `docs/technical/persistence.md` (the `data/session.json` row, ~L10)
- Locales (`en-US`/`es-ES`): none.

**Interfaces:**
- Consumes: `PlayerState::pending_start` and `pending_start::settle` (through `reconcile`, which `restore` already calls), both from Task 7.
- Produces: `pub pending_start: Option<(EntryId, f64)>` on `PlayerSession`, which defaults to `None` and loads leniently.

- [ ] **Step 1: Write the failing tests**

Append to `crates/fp-model/tests/pending_start.rs` (add `PlayerSession` and `RestoreParts` to the `use fp_model::{…}` line):

```rust
fn restore(state: &AppState, sessions: &[PlayerSession]) -> (AppState, Vec<EngineAction>) {
    let parts = RestoreParts {
        config: state.config.clone(),
        library: state.library.clone(),
        playlists: state.playlists.clone(),
        cart_pages: state.cartwall.pages.clone(),
        cartwall_session: state.cartwall.session(),
        ids: state.ids.clone(),
    };
    AppState::restore(parts, sessions, "Main")
}

#[test]
fn q8_8_the_session_keeps_a_pending_start_while_its_entry_is_still_next() {
    let mut s = fixture(3);
    let (e, p) = (entries(&s), p0(&s));
    apply(&mut s, Command::Seek(p, 30.0)).unwrap();
    let (restored, actions) = restore(&s, &s.sessions(|_| 0.0));
    assert_eq!(pending(&restored, p), Some((e[0], 30.0)));
    assert_eq!(restored.player(p).unwrap().transport, Transport::Stopped);
    assert_eq!(preloads(&actions, p), vec![(e[0], 30.0)]);
    assert!(
        actions.iter().all(|a| !matches!(
            a,
            EngineAction::StartCurrent { .. } | EngineAction::LoadPaused { .. }
        )),
        "nothing goes on air by itself: {actions:?}"
    );
}

#[test]
fn q8_8_a_pending_start_whose_entry_is_no_longer_next_is_dropped_on_restore() {
    let mut s = fixture(3);
    let (e, p) = (entries(&s), p0(&s));
    apply(&mut s, Command::Seek(p, 30.0)).unwrap();
    let mut sessions = s.sessions(|_| 0.0);
    sessions[0].next = Some(e[1]);
    sessions[0].next_explicit = true;
    let (restored, _) = restore(&s, &sessions);
    assert_eq!(pending(&restored, p), None);
}

#[test]
fn a_broken_pending_start_in_the_session_file_loads_as_none() {
    for value in [
        serde_json::json!("garbage"),
        serde_json::json!([1, null]),
        serde_json::json!({"entry": 1}),
    ] {
        let json = serde_json::json!({
            "id": 1,
            "playlist": 1,
            "next": 5,
            "pending_start": value,
        });
        let session: PlayerSession = serde_json::from_value(json).unwrap();
        assert_eq!(session.pending_start, None, "{value}");
        assert_eq!(session.next, Some(EntryId(5)), "the rest still loads");
    }
    let old: PlayerSession = serde_json::from_value(serde_json::json!({"id": 1, "playlist": 1})).unwrap();
    assert_eq!(old.pending_start, None, "files written before Q8");
}
```

- [ ] **Step 2: Run them to make sure they fail**

Run: `cargo test -p fp-model --test pending_start q8_8` and `cargo test -p fp-model --test pending_start a_broken_pending_start`

Expected: a compile error, because there is no field `pending_start` on `PlayerSession`.

- [ ] **Step 3: Implement**

`crates/fp-model/src/session.rs`: add to `PlayerSession`, after `history`:

```rust
    /// Rule 3a (operator feedback 4, Q8.8): where Play starts the next,
    /// kept only while that entry is still the next.
    #[serde(default, deserialize_with = "lenient_pending_start")]
    pub pending_start: Option<(EntryId, f64)>,
```

After `lenient_history`:

```rust
/// A pending start that does not parse loads as none: it only changes
/// where Play starts.
fn lenient_pending_start<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<Option<(EntryId, f64)>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Lenient {
        Valid(Option<(EntryId, f64)>),
        Other(serde::de::IgnoredAny),
    }
    Ok(match Lenient::deserialize(d)? {
        Lenient::Valid(v) => v.filter(|(_, secs)| secs.is_finite()),
        Lenient::Other(_) => None,
    })
}
```

In `sessions`, add `pending_start: p.pending_start,` to the `PlayerSession { … }` literal.

In `restore_player`, after `player.next_explicit = next_explicit;`:

```rust
    // Q8.8: only on a stopped player, for its next; `reconcile` clamps it
    // to the play range (`pending_start::settle`).
    player.pending_start = s
        .pending_start
        .filter(|(e, secs)| current.is_none() && Some(*e) == next && secs.is_finite());
```

In `crates/fp-model/tests/session.rs`, add `pending_start: None,` after `history: Vec::new(),` in both `PlayerSession` literals.

Check that no other file builds a `PlayerSession` literal: `grep -rn "PlayerSession {" crates --include=*.rs`. Add the field wherever the grep finds one.

- [ ] **Step 4: Run the tests to make sure they pass**

Run: `cargo test -p fp-model --test pending_start`, then `cargo test -p fp-model`, then `cargo test -p fp-store`

Expected: PASS. `fp-store` loads `session.json` through this type.

- [ ] **Step 5: Docs**

In `docs/technical/persistence.md`, in the `data/session.json` row, after the `history: …` part of the `players[]` list, add:

```markdown
, pending_start: `[entry, secs]`, where Play starts the next entry of a stopped player (rule 3a); kept only while that entry is still the next, loaded leniently (anything that does not parse is none)
```

- [ ] **Step 6: Commit**

```sh
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace \
&& git add crates/fp-model docs/technical/persistence.md \
&& git commit -m "feat(model): keep a stopped player's pending start across a restart" -m "Q8.8: only while its entry is still next; never on air by itself; a broken value loads as none." -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 9: The stopped player's waveform seeks and shows the pending start (view and UI, Q8.6)

**Suggested model:** `sonnet` (view field, one widget flag, kittests).

**Files:**
- Modify: `crates/fp-app/src/ui/view.rs` (`player_view`, the `pos` line, ~L210–212)
- Modify: `crates/fp-app/src/ui/player.rs`: `wave` (~L849–920), `seekable:` (~L913) and the comment in the zoom follow (~L878–880)
- Modify: `crates/fp-app/src/ui/widgets.rs` (`WaveInput::seekable` doc, ~L1103–1105)
- Test: `crates/fp-app/tests/view.rs` (append), `crates/fp-app/tests/waveform_ui.rs` (replace `a_stopped_players_waveform_does_not_seek`, ~L101–134)
- Docs:
  - `docs/user/players.md`: the "Title and artist" bullet (~L22–26) and the Waveform "Click to jump" bullet (~L127–129);
  - `docs/technical/ui.md`: §"Waveform view", ~L200–215.
- Locales (`en-US`/`es-ES`): none (`tip-waveform` "Waveform: click to seek" still fits).

**Interfaces:**
- Consumes: `PlayerState::pending_start` (Task 7) and `Command::Seek` on a stopped player (Task 7).
- Produces: `PlayerView::elapsed`, `remaining` and `markers.position` for a stopped player with a pending start.

- [ ] **Step 1: Write the failing tests**

Append to `crates/fp-app/tests/view.rs`:

```rust
/// Rule 3a (operator feedback 4, Q8.6): a stopped player shows its pending
/// start as its position: the playhead and the countdown.
#[test]
fn q8_6_a_stopped_player_shows_its_pending_start() {
    let (mut s, e, p) = state(3);
    apply(&mut s, Command::Seek(p, 50.0)).unwrap();
    let v = player_view(&s, p, Some(150.0), 0.0).unwrap();
    assert_eq!(v.status, PlayerStatus::Stopped);
    assert_eq!((v.elapsed, v.remaining), (50.0, 150.0));
    assert_eq!(v.markers.position, Some(0.25));
    apply(&mut s, Command::SetNext(p, e[1])).unwrap();
    assert_eq!(
        player_view(&s, p, None, 0.0).unwrap().elapsed,
        0.0,
        "a new next starts at its cue-in"
    );
}
```

In `crates/fp-app/tests/waveform_ui.rs`, replace the whole of `a_stopped_players_waveform_does_not_seek`, with its doc comment, by:

```rust
/// A stopped player shows its next track. A click on its waveform sets
/// where Play starts it (rule 3a, operator feedback 4 Q8); a drag never
/// seeks.
#[test]
fn a_click_on_a_stopped_players_waveform_sets_where_play_starts() {
    let mut s = playing();
    let p = s.players[0].id;
    apply(&mut s, Command::Stop(p)).unwrap();
    let next = s.players[0].next.unwrap();
    let track = s.playlists.entry(next).unwrap().track;
    let analysis = TrackAnalysis {
        duration_secs: 180.0,
        ..TrackAnalysis::default()
    };
    apply(
        &mut s,
        Command::ApplyAnalysis {
            track,
            analysis: Box::new(analysis),
        },
    )
    .unwrap();
    let (mut h, fake) = harness(s);
    let w = wave(&h);
    click(&mut h, pos2(x_of(w, 90.0), w.center().y));
    let sent = seeks(&fake);
    assert_eq!(sent.len(), 1, "{sent:?}");
    assert!((sent[0] - 90.0).abs() < 1.5, "{sent:?}");
    let model = fake.state.load();
    let (entry, secs) = model.players[0].pending_start.unwrap();
    assert_eq!(Some(entry), model.players[0].next);
    assert!((secs - 90.0).abs() < 1.5, "{secs}");
    let at = pos2(w.center().x, w.center().y);
    drag_to(&mut h, at, pos2(at.x + 60.0, at.y));
    press(&mut h, pos2(at.x + 60.0, at.y), false);
    h.run_steps(2);
    assert!(seeks(&fake).is_empty(), "a drag never seeks");
}
```

- [ ] **Step 2: Run them to make sure they fail**

Run: `cargo test -p fp-app --test view q8_6` and `cargo test -p fp-app --test waveform_ui a_click_on_a_stopped`

Expected: FAIL.
- `view`: elapsed is 0.0 (the cue-in), not 50.0.
- `waveform_ui`: no `Seek` is sent, because the waveform is not seekable while stopped.

- [ ] **Step 3: Implement**

`crates/fp-app/src/ui/view.rs`, in `player_view`, replace:

```rust
    let pos = position.filter(|v| v.is_finite()).unwrap_or(range.cue_in);
```

with:

```rust
    // Rule 3a (operator feedback 4, Q8.6): a stopped player shows where
    // Play will start its next.
    let pending = p
        .pending_start
        .filter(|(e, _)| p.transport == Transport::Stopped && p.next == Some(*e))
        .map(|(_, secs)| secs);
    let pos = position
        .filter(|v| v.is_finite())
        .or(pending)
        .unwrap_or(range.cue_in);
```

`crates/fp-app/src/ui/player.rs`, in `wave`:
- replace `seekable: pv.status != PlayerStatus::Stopped,` with `seekable: true,`;
- reword the comment above the zoom follow (`// A stopped player's position is pinned at the cue-in: …`) to: `// A stopped player's position is pinned (its cue-in or its pending start, rule 3a): following it would undo a zoom made to prepare the next track.`

Leave the follow condition (`pv.status != PlayerStatus::Stopped`) as it is: Q8.6 says the zoom still does not follow.

`crates/fp-app/src/ui/widgets.rs`, the doc of `WaveInput::seekable`:

```rust
    /// Clicks seek. On a stopped player a click sets where Play starts the
    /// next track (rule 3a); the CUE window seeks the CUE.
    pub seekable: bool,
```

If `PlayerStatus` is no longer used in `player.rs` after this change, remove it from the imports (clippy reports it).

- [ ] **Step 4: Run the tests to make sure they pass**

Run: `cargo test -p fp-app --test view`, then `cargo test -p fp-app --test waveform_ui`, then `cargo test -p fp-app --test markers_ui`, then `cargo test -p fp-app --test waveform_view`

Expected: PASS.

- [ ] **Step 5: Docs**

`docs/user/players.md`:
- In the Info row "Title and artist" bullet, change "ready at the cue-in." to "ready at the cue-in, or where you clicked its waveform."
- In the Waveform section, replace the bullet "Hover to see the time under the pointer. **Click to jump** there (while playing or paused; a stopped player always starts at the cue-in). …" with:

```markdown
- Hover to see the time under the pointer. **Click to jump** there. On a
  stopped player a click chooses where **Play** starts the next track: the
  playhead and the countdown move there, and nothing plays until you press
  Play. Choosing another next track, moving or removing it, or Stop goes back
  to the cue-in; so does any other way of starting a track, and Restart,
  Previous and the automatic advance always use the cue-in. A click after the
  end (in the darker tail) keeps the cue-in. A click is a press and release
  without moving the pointer more than a few pixels.
```

`docs/technical/ui.md`, §"Waveform view", after the sentence about `widgets::waveform` reporting a click's seek target, add:

```markdown
Every waveform is seekable. On a stopped player the click sends
`Command::Seek`, which the model turns into the pending start (rule 3a), and
`view::player_view` shows that start as the stopped player's position; the
zoom of a stopped player still does not follow it.
```

- [ ] **Step 6: Commit**

```sh
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace \
&& git add crates/fp-app/src/ui crates/fp-app/tests/view.rs crates/fp-app/tests/waveform_ui.rs docs/user/players.md docs/technical/ui.md \
&& git commit -m "feat(ui): click a stopped player's waveform to choose where Play starts" -m "Q8.6: the playhead and countdown show the pending start; the zoom still does not follow it." -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 10: The remote API's seek on a stopped player

**Suggested model:** `sonnet` (one match arm and its tests).

**Files:**
- Modify: `crates/fp-remote/src/api.rs` (`O::Seek`, ~L166–181)
- Test: `crates/fp-remote/tests/api.rs` (`seek_needs_a_running_entry_and_a_position_inside_its_cue_range`, ~L130–140, plus a new test)
- Docs: `docs/technical/remote-api.md` (the `POST /players/{id}/seek` row, ~L117)
- Locales (`en-US`/`es-ES`): none.

**Interfaces:**
- Consumes: `Command::Seek` on a stopped player (Task 7).
- Produces: `plan(state, Operation::Seek(p, secs))`. It is `Ok(vec![Command::Seek(p, secs)])` for a stopped player with a next when `secs` is within that entry's cue range. It is `409` when there is nothing to seek.

- [ ] **Step 1: Write the failing tests**

In `crates/fp-remote/tests/api.rs`:
- rename `seek_needs_a_running_entry_and_a_position_inside_its_cue_range` to `seek_while_playing_stays_inside_the_cue_range_of_what_plays`;
- delete its first assertion, `assert_eq!(plan(&s, O::Seek(p, 10.0)).unwrap_err().status(), 409);`;
- append:

```rust
/// Operator feedback 4, Q8 (rule 3a): a seek on a stopped player sets
/// where Play starts its next, inside that entry's cue range.
#[test]
fn seek_on_a_stopped_player_sets_where_play_starts_its_next() {
    let mut s = demo_state();
    let p = s.players[0].id;
    assert_eq!(
        plan(&s, O::Seek(p, 10.0)).unwrap(),
        vec![Command::Seek(p, 10.0)]
    );
    assert_eq!(plan(&s, O::Seek(p, 500.0)).unwrap_err().status(), 400);
    // With no next there is nothing to seek.
    let all: Vec<EntryId> = s
        .playlists
        .iter()
        .flat_map(|l| l.entries.iter().map(|e| e.id))
        .collect();
    for e in all {
        fp_model::apply(&mut s, Command::RemoveEntry(e)).unwrap();
    }
    assert_eq!(s.player(p).unwrap().next, None);
    assert_eq!(plan(&s, O::Seek(p, 10.0)).unwrap_err().status(), 409);
}
```

- [ ] **Step 2: Run them to make sure they fail**

Run: `cargo test -p fp-remote --test api seek`

Expected: `seek_on_a_stopped_player_sets_where_play_starts_its_next` FAILS, because the stopped player gets a `409`.

- [ ] **Step 3: Implement**

In `crates/fp-remote/src/api.rs`, replace the head of the `O::Seek` arm, up to `.ok_or_else(…)?;`, with:

```rust
        O::Seek(p, secs) => {
            let pl = player(p)?;
            // Operator feedback 4, Q8 (rule 3a): a stopped player's seek
            // sets where Play starts its next.
            let entry = if pl.transport == Transport::Stopped {
                pl.next
            } else {
                pl.current
            };
            let track = entry
                .and_then(|e| state.track_for_entry(e))
                .ok_or_else(|| ApiError::Unavailable("nothing to seek".to_owned()))?;
```

The cue-range check and `Ok(vec![Command::Seek(p, secs)])` after it stay as they are.

- [ ] **Step 4: Run the tests to make sure they pass**

Run: `cargo test -p fp-remote`

Expected: PASS. Also check that no OSC or HTTP test still expects a `409` for a stopped player: `grep -rn "Seek\|/seek" crates/fp-remote/tests`.

- [ ] **Step 5: Docs**

In `docs/technical/remote-api.md`, the seek row becomes:

```markdown
| `POST /players/{id}/seek` | `{"secs": f64}` | Seek within the cue range of what is playing; on a stopped player, set where Play starts the next entry (rule 3a), within that entry's cue range. `409` when there is nothing to seek |
```

- [ ] **Step 6: Commit**

```sh
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace \
&& git add crates/fp-remote docs/technical/remote-api.md \
&& git commit -m "feat(remote): seek a stopped player to choose where Play starts" -m "The remote seek follows rule 3a (operator feedback 4, Q8)." -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 11: Spec "As built", links and the closing docs check

**Suggested model:** `sonnet` (docs only).

**Files:**
- Modify: `docs/superpowers/specs/2026-10-06-operator-feedback-4-design.md`:
  - the status line;
  - §7: the plan 2 file name becomes `2026-10-06-feedback4-plan2-play-before-analysis.md`;
  - an "As built" subsection at the end of §3.
- Modify: `docs/superpowers/plans/2026-10-06-feedback4-plan3-wave-panel.md` (the plan 2 link in its **Spec** line and its rulings: `…-plan2-start-point.md` → `…-plan2-play-before-analysis.md`)
- Modify: `README.md`, only if `grep -n -i "waveform\|seek\|before.*analy" README.md` finds a sentence these changes make wrong.
- Locales (`en-US`/`es-ES`): none. Confirm with `cargo test -p fp-app --test i18n`.

- [ ] **Step 1: Write the "As built" notes**

At the end of §3 of the feedback 4 spec, add a `### As built (plan 2)` subsection. Fill it from the ledger and the code as merged; every bullet names the real function:

- **Q6.** The cause from Task 2's ruling, the file and function fixed, and the tests `cue_follow_ui.rs::q6_1_…`, `q6_2_…` and `q6_3_…`.
- **Q1.**
  - `FileDecoder::duration_hint_secs` and `duration_from_frames`, and the MPEG Xing/Info/VBRI rule;
  - `fp-app/src/header.rs` (`HeaderReader`, `fp-header-reader`) and `Services::header_pass`, with eight reads in flight and tracks on a player first;
  - `Command::SetDuration` and `Track::needs_header_duration`;
  - `view::time_text` with `placeholder-none`.
- **Q8.**
  - `PlayerState::pending_start` and `fp-model/src/pending_start.rs` (`pending_start_at`, `settle`, `start_request`, `preload_request`);
  - every `SetNext` clears the pending start, `MoveEntry` and `Stop` clear it, and `settle` runs in `reconcile`;
  - a start at or past the end of the play range is not stored;
  - `PlayerSession::pending_start` loads leniently;
  - the remote seek on a stopped player;
  - the engine is unchanged (`take_or_open`).
- Every `Ruling:` line of this plan's ledger, one line each.

Set the status line to say that plan 2 is implemented.

- [ ] **Step 2: Check that the docs agree**

Run:

```sh
grep -rn "stopped player always starts at the cue-in\|does not seek\|seekable: pv.status" docs crates/fp-app/src
grep -n "pending start\|pending_start" docs/user/players.md docs/technical/ui.md docs/technical/audio-engine.md docs/technical/persistence.md docs/technical/remote-api.md
grep -n "header" docs/technical/analysis.md docs/technical/decoding.md
```

Expected:
- the first grep finds nothing (or only the main spec's history);
- the second grep finds each file once or more;
- the third finds Tasks 3 and 5's paragraphs.

Fix any sentence that disagrees.

- [ ] **Step 3: Commit**

```sh
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace \
&& git add docs README.md \
&& git commit -m "docs: feedback 4 plan 2 as built" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

- [ ] **Step 4: Review and PR**

Follow CLAUDE.md:
1. Run `superpowers:requesting-code-review` with a fresh reviewer on the most capable model, over the whole branch range of this plan. Fix the Critical and Important findings test-first, and defer the Minor ones in the ledger.
2. Push, and open the PR with `gh pr create` following `.github/pull_request_template.md`. Title: `feat: play before analysis, start point and CUE follow (feedback 4, plan 2)`. The body ends with the line `🤖 Generated with [Claude Code](https://claude.com/claude-code)`.
3. Run `gh pr checks --watch`.
4. Merge with `gh pr merge --merge --delete-branch` once it is green and reviewed.

---

## Spec coverage (self-review)

| Spec rule | Task |
|---|---|
| Q1.1 worker thread reads `frames_hint` and the rate, on add or show | 3 (`duration_hint_secs`), 5 (`HeaderReader`, `header_pass`, tracks on a player first) |
| Q1.2 `SetDuration` only while not analysed, finite and positive; the analysis replaces it | 4 |
| Q1.3 countdown, `elapsed / total`, position and click-to-seek; centre line and playhead without a waveform | 6 (view and kittest); the widget already draws the line and playhead with `total` and no media (`widgets.rs` ~L1145–1160, L1315) |
| Q1.4 no hint: elapsed shown, total "—", no seek | 3 (MPEG estimate → none), 6 (`time_text`, kittest) |
| Q1.5 play range `0..duration` | 4 |
| Q1 tests: reducer, `player_view`, decode (WAV, FLAC, VBR MP3 without header), kittest seek | 4, 6, 3, 6 |
| Q8.1 `pending_start` field, serde default | 7 (state), 8 (session) |
| Q8.2 seek while Stopped stores it, clamped, no playback action; nothing without a next | 7 |
| Q8.3 Play uses it once | 7 |
| Q8.4 cleared by a next change, removal, move, Stop, use, other starts | 7 |
| Q8.5 Restart, advance, segue and Previous keep the cue-in | 7 (`q8_5_…`; the segue uses the same `preload_target` path as the advance) |
| Q8.6 view position and playhead; seekable; the zoom does not follow | 9 |
| Q8.7 preload at the pending start; re-preload on change | 7 (model and engine test) |
| Q8.8 not on air; session keeps it while its entry is next | 7 (`q8_8_a_pending_start_never_goes_on_air_by_itself`), 8 |
| Q8 remote API follows the rule | 10 |
| Q6.1–Q6.3 with the window open, any tab, guards kept | 1, 2 |
| Docs per item | each task, 11 |
