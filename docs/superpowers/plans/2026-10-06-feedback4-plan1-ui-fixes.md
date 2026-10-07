# UI Fixes (Feedback 4, Plan 1) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The CUE window's Pause blinks while paused (Q5); a track waiting for its analysis shows a muted hourglass (Q2); the row's track info popup starts with the file error reason, is the only hover information on a row and never jumps (Q9); the violet drop line is drawn at the row boundary under the pointer, only in the table under the pointer (Q4); and a file that failed once is looked at again by size and modification time, with a row menu **Re-analyse** (Q10).

**Architecture:**
- **Pure first.** `widgets::{blink, paused_style}`, `view::{analysis_pending, animating}`, `view::track_tooltip` with a reason line and `table_layout::{drop_index, boundary_y, on_column_edge}` are pure functions with their own tests. The UI draws what they decide.
- **Q4 replaces widget hit-testing by geometry.** The drop index comes from the pointer, the scroll area's visible rect (`ScrollAreaOutput::inner_rect`), its scroll offset and `ROW_HEIGHT`; `DropTarget` gains the player.
- **Q10 stays off the services and UI threads.** The probe thread (`fp-file-probe`) already looks for missing files; it also `stat`s unreadable ones and answers with what it saw. `Services` decides (pure `look`), clears the failure and queues the analysis.

**Tech Stack:** Rust, egui/eframe 0.36.2, egui_extras 0.36.2 (`TableBuilder`), egui_kittest 0.36.2, crossbeam-channel. No new dependency, so `cargo deny check` needs no new entry.

**Spec:** `docs/superpowers/specs/2026-10-06-operator-feedback-4-design.md` §2 (items Q5, Q2, Q9, Q4, Q10; the plan order of §7 is Q5, Q2, Q9, Q4, Q10) and §9 (global constraints). It extends the main spec (§6 analysis, §8.3 track table), `2026-10-01-operator-feedback-2-design.md` (O12, O23) and the meters/bit-perfect specs only by reference. Branch: `fix/feedback4-ui-fixes`, from an up-to-date `master`.

## Global Constraints

Copied from spec §9 (`CLAUDE.md` rules 1–10 apply to every task):

- English everywhere, and UI strings in both locales (`crates/fp-app/locales/en-US/main.ftl` is the source, `es-ES/main.ftl` always follows; `tests/i18n.rs::both_locales_define_the_same_keys` checks it).
- No product names (other playout or radio-automation products) anywhere.
- Operator values are `Config` fields with defaults, ranges and lenient loading. This plan adds none: the probe interval is the existing `tuning.missing_recheck_ms`.
- The real-time path never allocates, locks, logs or panics. This plan does not touch it.
- Behaviour lives in `fp-model`. This plan adds no behaviour rule to the model: every decision is a pure function in `fp-app` (`view`, `table_layout`, `services::look`) with its own tests, because each one is about what the screen shows or what a file's `stat` says.
- The UI never blocks: the `stat` of Q10 runs on the probe thread, one look at a time. No file system call is added to the UI thread.
- Bad data never crashes: a file that cannot be `stat`ed, a file that changes while it is looked at, a pointer outside the window, a list of zero entries.
- Nothing goes on air by itself: none of the five items starts, queues or skips a track.
- Safety lints: no `unwrap`, `expect`, `panic` outside tests; `fp-app` code prefers `get` to indexing.
- Commands: `cargo test -p <crate> --test <file> <name>` (one filter per command). Commits only when `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace` passes. `CHANGELOG.md` is never edited by hand. Commit messages end with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.

## Review Focus

Failure modes the spec implies and its tests do not name; each has a test in the task named.

1. **A drag that ends where there is no target** (released over the header, a column edge, the scroll bar, below the table, in the other player's table, or in an empty list): nothing moves, no bar is left behind, and an empty playlist still accepts a drop at index 0. (Task 5 `releasing_over_the_header_drops_nothing`, `releasing_on_a_column_edge_drops_nothing`, `an_empty_playlist_accepts_a_drop_at_zero`; Task 4 `outside_the_body_there_is_no_target`, `an_empty_list_has_the_one_boundary_zero`.)
2. **A long, scrolled playlist** (thousands of entries, the target row not built this frame, the wheel turned during the drag): the bar follows the pointer and a boundary outside the view draws nothing. (Task 4 `a_scrolled_body_shifts_the_boundaries`, `a_boundary_outside_the_view_has_no_y`; Task 5 `the_bar_follows_the_pointer_in_a_scrolled_long_list`.)
3. **A file that cannot be `stat`ed** (permission denied, a share that went offline, a path that became a folder) or **keeps changing** (a copy in progress): nothing happens for the first, no tight loop of looks, and a changed file is analysed once per look, never in a loop. (Task 6 `look_keeps_what_it_knows_when_the_stat_fails`, `a_file_that_fails_again_is_looked_at_with_its_new_stat`, `an_unreadable_file_that_did_not_change_is_not_analysed_again`.)
4. **A long reason or path in a narrow window** (a 380-point window, an 80-character path): the popup wraps, stays inside the window and is at the same place on its first and second frame. (Task 3 `the_popup_stays_inside_a_narrow_window`, `the_popup_never_shows_at_another_place_first`.)
5. **A pending track that stops being pending in other ways** (it turns missing or unreadable while waiting; it is analysed while the table is visible; an outdated one): the hourglass goes and the other flag, if any, stays. (Task 2 `a_pending_flag_goes_when_the_file_turns_unreadable`, `the_flag_disappears_when_the_analysis_arrives`, `an_outdated_track_never_shows_the_hourglass`.)

## Rulings

Each is a spec gap or ambiguity resolved while reading the code.

- Ruling: Q5.3 needs no new repaint rule — `AppUi::ui` already requests a repaint every frame while `p.cue.is_some()` (`app.rs` ~L863–871), and a paused CUE still has `cue` set — it is extracted as `view::animating` so that a test pins it — a regression would freeze the blink until the pointer moves.
- Ruling: the main spec lines of §6 (L381), §8.3 (L453, L457, L458) and the feedback 2 spec lines about Q5/Q9 (L283, L319, L462) were already edited with the design spec, so the docs task verifies them and adds the "As built" notes only — nothing to rewrite — a second edit would duplicate the sentences.
- Ruling: Q9.4 (the cartwall's carts show the same error line) needs no code: `cartwall.rs` ~L374 already shows `Scene::file_tip` on hover, and `hovering_an_unavailable_cart_says_why` pins it — kept as the guard — cost if wrong: one more `on_hover_text` line.
- Ruling: Q9.1 reuses the existing `file-missing-tip` / `file-unreadable-tip` strings, which end with the path, so the path shows twice (in the reason and in the Path row) — the spec says "no new strings (the reasons exist)" — cost if wrong: two new reason-only strings and a lookup change in `file_tip`.
- Ruling: Q9.2 removes the file-error tooltips of the number and the title only; the `P<n>` mark (`tip-on-air-elsewhere`) and the "next again" arrow (`tip-next-again`) keep their own tooltips because they explain a glyph, not the file — the spec names the file error tooltips as the conflict — cost if wrong: delete two `on_hover_text` calls.
- Ruling: Q4.1 says the index "never depends on which widget is under the pointer" while Q4.2 says there is no target over the column-resize handles; the plan follows Q4.2 for the handles (a grab zone of `resize_grab_radius_side` on each side of an interior edge gives no target) and Q4.1 for everything else — both are literal spec rules and the spec's own kittest asks for "hovering a resize handle gives no target" — cost if wrong: the bar blinks off for about 8 points when a drag crosses an edge; drop `on_column_edge` from the filter.
- Ruling: `Services.failed` stays a `HashSet<TrackId>` and the stats go in a new `stamps: HashMap<TrackId, Seen>` — playback failures (`SourceFailed`, `PreloadFailed`) set `Unreadable` in the model without going through `failed`, and putting them in `failed` would also change when they are analysed (`submit_new` gates on it) — cost if wrong: rename the set to a map and thread `Seen` through its 12 uses.
- Ruling: the first look at a newly unreadable file is not waited for the timer: it is sent at once (and alone) so that the recorded size and modification time are close to the failure, later looks follow `tuning.missing_recheck_ms` — Q10.1 says "when it first sees the failure" — cost if wrong: a fix made in the first 30 s after the failure is recorded as the failure's stat and is missed until the operator uses Re-analyse.
- Ruling: a `stat` error other than "not found" records `Seen::Unknown` and is otherwise ignored (no analysis, no model change); "not found" sends `SetFileState(Missing)` (Q10.3) — the spec says nothing about permission errors — cost if wrong: a permission fix is only noticed through Re-analyse.
- Ruling: **Re-analyse** goes through the same analyzer and cache as *Re-analyse all*, so an unchanged file whose analysis is cached comes back playable and fails again on the next playback — Q10.4 says "whatever its state" and Q10.5 says a failed analysis leaves it `Unreadable` with the new stat — cost if wrong: add a cache bypass flag to `submit_urgent`.
- Ruling: the kittests read drawn shapes through `Harness::output().shapes` (the way `exit_guard.rs` reads `viewport_output`) because the fill of a tile and the violet bar are painted shapes, not accessible nodes — cost if wrong: replace the helper in two test files with a snapshot test.

## File Structure

- Modify `crates/fp-app/src/ui/widgets.rs` (`blink`, `paused_style`), `ui/player.rs` (use them), `ui/cue_window.rs` (Pause tile style), `ui/view.rs` (`analysis_pending`, `animating`, `TipField::Problem`, `track_tooltip`), `ui/app.rs` (`animating`, `DropTarget.player`, `Scene.services`, `Scene::request`, the file-drop fallback), `ui/table.rs` (hourglass flag, popup, drop geometry, menu item), `ui/table_layout.rs` (`drop_index`, `boundary_y`, `on_column_edge`), `services.rs` (probe, `Seen`, `look`, `ReanalyseTrack`).
- Create tests `crates/fp-app/tests/blink.rs`, `table_tooltip.rs`, `table_drop_ui.rs`, `reanalyse_ui.rs`; extend `tests/cue_window.rs`, `tests/view.rs`, `tests/table_icons.rs`, `tests/table_layout.rs`, `tests/services.rs`.
- Modify `crates/fp-app/locales/en-US/main.ftl` and `es-ES/main.ftl` (`flag-analysis-pending`, `menu-reanalyse`).
- Modify docs (Task 8): `docs/user/players.md`, `docs/user/playlists.md`, `docs/user/troubleshooting.md`, `docs/technical/ui.md`, `docs/technical/analysis.md`, the spec (status and "As built" under §2), `README.md` only if it lists these behaviours.

---

### Task 1: The CUE window's Pause blinks (Q5)

Suggested executor: `sonnet`.

**Files:**
- Modify: `crates/fp-app/src/ui/widgets.rs` (after `TileStyle::plain`, ~L58), `crates/fp-app/src/ui/player.rs` (`transport`, L637 and L696–716), `crates/fp-app/src/ui/cue_window.rs` (`buttons`, L155–176), `crates/fp-app/src/ui/view.rs`, `crates/fp-app/src/ui/app.rs` (L863–871)
- Test: `crates/fp-app/tests/blink.rs` (create), `crates/fp-app/tests/cue_window.rs`
- Locales: none. Docs: Task 8 (`docs/user/players.md`).

**Interfaces:**
- Consumes: `TileStyle`, `theme::{AMBER_BG, AMBER_TEXT, AMBER_DIM}`, `Scene::time` (seconds since start).
- Produces: `pub fn widgets::blink(time: f64) -> bool` (true for the first half of each second), `pub fn widgets::paused_style(blink: bool) -> TileStyle`, `pub fn view::animating(state: &AppState) -> bool` (a player is playing, fading or has a CUE, paused or not).

- [x] **Step 1: Write the failing tests**

Create `crates/fp-app/tests/blink.rs`:

```rust
#![allow(clippy::unwrap_used)]
//! Operator feedback 4, Q5: the blink shared by the player and the CUE window.

use egui::Color32;
use fp_app::ui::theme;
use fp_app::ui::widgets::{blink, paused_style};

#[test]
fn the_blink_is_on_for_half_a_second_then_off() {
    assert!(blink(0.0));
    assert!(blink(0.49));
    assert!(!blink(0.5));
    assert!(!blink(0.99));
    assert!(blink(1.0));
    assert!(!blink(1.5));
}

#[test]
fn a_broken_clock_never_panics() {
    let _ = blink(f64::NAN);
    let _ = blink(f64::INFINITY);
    let _ = blink(-3.2);
}

#[test]
fn the_paused_style_is_amber_when_lit_and_transparent_when_not() {
    let on = paused_style(true);
    assert_eq!(on.fill, theme::AMBER_BG);
    assert_eq!(on.content, theme::AMBER_TEXT);
    let off = paused_style(false);
    assert_eq!(off.fill, Color32::TRANSPARENT);
    assert_eq!(off.content, theme::AMBER_DIM);
}
```

Append to `crates/fp-app/tests/cue_window.rs` (add `use egui::Color32;` and `use fp_app::ui::theme;` to the imports):

```rust
/// The fill the tile named `label` is drawn with in the last frame: the
/// first rectangle painted at its rectangle is its background.
fn tile_fill(h: &Harness<'_, AppUi>, label: &str) -> Color32 {
    let rect = h.get_by_label(label).rect();
    h.output()
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            egui::Shape::Rect(r)
                if r.rect.expand(0.5).contains_rect(rect)
                    && rect.expand(0.5).contains_rect(r.rect) =>
            {
                Some(r.fill)
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("no tile drawn at {rect:?}"))
}

/// The fills `label` takes over 1.2 s of frames (60 steps of 20 ms).
fn fills_over_a_second(h: &mut Harness<'_, AppUi>, label: &str) -> std::collections::HashSet<Color32> {
    let mut fills = std::collections::HashSet::new();
    for _ in 0..60 {
        h.step();
        fills.insert(tile_fill(h, label));
    }
    fills
}

#[test]
fn a_paused_cue_blinks_its_resume_button_amber() {
    let (mut h, fake) = cueing(1);
    // Sent to the model directly: a click would leave the pointer on the
    // tile and its hover fill would hide the blink.
    fake.send(Command::SetCuePaused(fake.player(0), true));
    h.run_steps(2);
    let fills = fills_over_a_second(&mut h, RESUME);
    assert!(
        fills.contains(&theme::AMBER_BG) && fills.contains(&Color32::TRANSPARENT),
        "{fills:?}"
    );
}

#[test]
fn a_running_cue_does_not_blink() {
    let (mut h, _fake) = cueing(1);
    let fills = fills_over_a_second(&mut h, PAUSE);
    assert_eq!(fills.len(), 1, "{fills:?}");
}

#[test]
fn a_paused_cue_keeps_the_interface_repainting() {
    let (_h, fake) = cueing(1);
    fake.send(Command::SetCuePaused(fake.player(0), true));
    assert!(fp_app::ui::view::animating(&fake.state.load()));
    assert!(!fp_app::ui::view::animating(&state(1, 3)));
}
```

- [x] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-app --test blink`
Expected: FAIL to compile (`blink` and `paused_style` not found).

Run: `cargo test -p fp-app --test cue_window a_paused_cue`
Expected: FAIL to compile (`animating` not found).

- [x] **Step 3: Implement**

In `widgets.rs`, after `impl TileStyle`:

```rust
/// The blink phase of a paused button: lit for the first half of each
/// second, dark for the second half (main spec rule 6).
pub fn blink(time: f64) -> bool {
    (time * 2.0).floor() as i64 % 2 == 0
}

/// The look of the Pause button of something that is paused: amber, blinking
/// with `blink`.
pub fn paused_style(blink: bool) -> TileStyle {
    TileStyle {
        fill: if blink {
            theme::AMBER_BG
        } else {
            Color32::TRANSPARENT
        },
        content: if blink {
            theme::AMBER_TEXT
        } else {
            theme::AMBER_DIM
        },
        ..TileStyle::plain()
    }
}
```

In `player.rs::transport`, replace the line `let blink = (scene.time * 2.0).floor() as i64 % 2 == 0;` with `let blink = widgets::blink(scene.time);` and the whole `let pause_style = if paused { TileStyle { fill: if blink {...} ... } } else { TileStyle::plain() };` with:

```rust
let pause_style = if paused {
    widgets::paused_style(blink)
} else {
    TileStyle::plain()
};
```

In `cue_window.rs::buttons`, replace the Pause tile's `TileStyle::plain()` (L165) by `style`, defined before the `widgets::tile` call:

```rust
let style = if v.paused {
    widgets::paused_style(widgets::blink(scene.time))
} else {
    TileStyle::plain()
};
```

In `view.rs` (after `volume_db`):

```rust
/// Something on screen moves on its own (a meter, a countdown, a blinking
/// Pause button): the interface repaints every frame instead of at the idle
/// rate. A paused CUE still has its `cue`, so its window keeps blinking.
pub fn animating(state: &AppState) -> bool {
    state
        .players
        .iter()
        .any(|p| p.transport == Transport::Playing || p.fading || p.cue.is_some())
}
```

In `app.rs`, replace the `let busy = state.players.iter().any(...)` block (L863–866) with `let busy = view::animating(&state);` (add `view` to the `super::` imports if it is not in scope).

- [x] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-app --test blink`, `cargo test -p fp-app --test cue_window`
Expected: PASS (all, including the ones that were there).

- [x] **Step 5: Commit**

```bash
cargo fmt --all
if cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace; then
  git add crates/fp-app/src/ui crates/fp-app/tests
  git commit -m "fix(ui): the CUE window's Pause blinks while the CUE is paused" -m "A paused CUE looked the same as a running one in its window, while a paused player's Pause button blinks amber. The blink and its style are now shared by both." -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
fi
```

---

### Task 2: Pending-analysis flag (Q2)

Suggested executor: `sonnet`.

**Files:**
- Modify: `crates/fp-app/src/ui/view.rs` (after `file_icon`, ~L270), `crates/fp-app/src/ui/table.rs` (Title cell flag strip, L390–407), `crates/fp-app/locales/en-US/main.ftl` and `es-ES/main.ftl` (after `flag-outdated`, L78)
- Test: `crates/fp-app/tests/view.rs`, `crates/fp-app/tests/table_icons.rs`
- Docs: Task 8 (`docs/user/playlists.md`, `docs/technical/ui.md`).

**Interfaces:**
- Consumes: `Track::{analyzed, file_state}`, `FileState::is_playable`, `services::outdated`, the table's `flag(ui, label, paint)` helper (`table.rs` L859), `widgets::glyph`.
- Produces: `pub fn view::analysis_pending(track: &Track) -> bool`; Fluent key `flag-analysis-pending`.

- [x] **Step 1: Write the failing tests**

In `tests/view.rs` add `analysis_pending` to the `fp_app::ui::view::{...}` import list and, if missing, `FileState` to the `fp_model` imports. Append:

```rust
fn pending_track() -> Track {
    Track::new(fp_model::TrackId(7), PathBuf::from("/m/New.flac"))
}

#[test]
fn q2_1_an_unanalysed_playable_track_is_pending() {
    assert!(analysis_pending(&pending_track()));
}

#[test]
fn q2_2_a_missing_or_unreadable_file_is_not_pending() {
    for state in [FileState::Missing, FileState::Unreadable] {
        let mut t = pending_track();
        t.file_state = state;
        assert!(!analysis_pending(&t), "{state:?}");
    }
}

#[test]
fn q2_3_an_analysed_track_is_not_pending_and_an_outdated_one_keeps_its_own_flag() {
    let mut t = pending_track();
    t.analyzed = true;
    assert!(!analysis_pending(&t));
    // Analysed by an earlier version: no format and version 0.
    t.format = None;
    t.analysis_version = 0;
    assert!(fp_app::services::outdated(&t));
    assert!(!analysis_pending(&t));
}
```

Append to `tests/table_icons.rs` (add `use fp_model::TrackAnalysis;`):

```rust
fn pending_flags(h: &Harness<'_, AppUi>) -> usize {
    h.query_all_by_label("Analysis pending").count()
}

#[test]
fn every_row_waiting_for_its_analysis_shows_the_hourglass_flag() {
    let (h, _) = harness(state(1, 3));
    assert_eq!(pending_flags(&h), 3);
}

#[test]
fn the_flag_disappears_when_the_analysis_arrives() {
    let (mut h, fake) = harness(state(1, 3));
    let track = fake.state.load().playlists.iter().next().unwrap().entries[1].track;
    fake.send(Command::ApplyAnalysis {
        track,
        analysis: Box::new(TrackAnalysis::default()),
    });
    h.run_steps(2);
    assert_eq!(pending_flags(&h), 2);
}

#[test]
fn a_pending_flag_goes_when_the_file_turns_unreadable() {
    let (mut h, fake) = harness(state(1, 3));
    let track = fake.state.load().playlists.iter().next().unwrap().entries[0].track;
    fake.send(Command::SetFileState {
        track,
        state: fp_model::FileState::Unreadable,
    });
    h.run_steps(2);
    assert_eq!(pending_flags(&h), 2, "the unreadable row has its own icon");
}

#[test]
fn an_outdated_track_never_shows_the_hourglass() {
    let mut s = state(1, 1);
    let track = s.playlists.entry(entries(&s)[0]).unwrap().track;
    let t = s.library.get_mut(track).unwrap();
    t.analyzed = true;
    t.analysis_version = 0;
    let (h, _) = harness(s);
    assert_eq!(pending_flags(&h), 0);
    assert!(
        h.query_by_label_contains("Analysed by an earlier version")
            .is_some()
    );
}
```

- [x] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-app --test view q2_`
Expected: FAIL to compile (`analysis_pending` not found).

- [x] **Step 3: Implement**

`view.rs`:

```rust
/// Q2: the track waits for its analysis and its file can be played, so the
/// table says why it has no waveform yet. A missing or unreadable file has
/// its own icon; an analysed one (outdated or not) has nothing pending.
pub fn analysis_pending(track: &Track) -> bool {
    !track.analyzed && track.file_state.is_playable()
}
```

`table.rs`, inside the Title cell's `right_to_left` strip, right after the `if crate::services::outdated(track) { ... }` block:

```rust
if view::analysis_pending(track) {
    flag(ui, &t.tr("flag-analysis-pending"), |p, r| {
        widgets::glyph(p, r, icon::HOURGLASS, 13.0, theme::NEUTRAL_400, false);
    });
}
```

Locales, after `flag-outdated`: en-US `flag-analysis-pending = Analysis pending`; es-ES `flag-analysis-pending = Análisis pendiente`.

- [x] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-app --test view q2_`, `cargo test -p fp-app --test table_icons`, `cargo test -p fp-app --test i18n`
Expected: PASS. If an older test now sees "Analysis pending" labels it did not expect, fix the older test's query (rows built with `state(..)` are unanalysed on purpose).

- [x] **Step 5: Commit**

```bash
cargo fmt --all
if cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace; then
  git add crates/fp-app
  git commit -m "feat(ui): a muted hourglass marks a track whose analysis is pending" -m "A row waiting for its analysis looked like any other, so the operator could not tell why it had no waveform yet." -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
fi
```

---

### Task 3: Track tooltip: the reason first, one popup, a fixed width (Q9)

Suggested executor: `sonnet`.

**Files:**
- Modify: `crates/fp-app/src/ui/view.rs` (`TipField` and `track_tooltip`, L404–440), `crates/fp-app/src/ui/table.rs` (`track_tip` L871–901, the number label tooltip L365–369, the title tooltip L433–448, the call at L504–506)
- Test: `crates/fp-app/tests/view.rs` (L490–540 call the old signature), `crates/fp-app/tests/table_tooltip.rs` (create)
- Locales: none (the reasons exist). Docs: Task 8 (`docs/user/playlists.md`).

**Interfaces:**
- Consumes: `Scene::file_tip(TrackId) -> Option<String>` (`app.rs` L253), `view::file_icon`.
- Produces: `view::TipField::Problem` (first variant after the existing ones is not required; it is the first line returned), `pub fn view::track_tooltip(track: &Track, problem: Option<&str>) -> Vec<(TipField, String)>`, `const TIP_WIDTH: f32` in `table.rs`.

- [x] **Step 1: Write the failing tests**

In `tests/view.rs`: change the two existing calls `track_tooltip(&tip_track())` and `track_tooltip(&t)` to pass `None` as the second argument, and append:

```rust
#[test]
fn q9_1_the_reason_is_the_first_line_of_the_tooltip() {
    let reason = "Cannot read the file: /m/Artist - Song.flac";
    let tip = track_tooltip(&tip_track(), Some(reason));
    assert_eq!(tip[0], (TipField::Problem, reason.to_owned()));
    assert_eq!(tip[1].0, TipField::Title);
    assert_eq!(tip.len(), 9, "the other fields follow as before");
}

#[test]
fn q9_1_a_blank_reason_is_left_out() {
    let tip = track_tooltip(&tip_track(), Some(""));
    assert_eq!(tip[0].0, TipField::Title);
}
```

Create `crates/fp-app/tests/table_tooltip.rs`:

```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Operator feedback 4, Q9: the row's track info popup.

mod support;

use egui_kittest::kittest::Queryable;
use fp_model::{AppState, Command, FileState};
use support::{harness, harness_sized, state};

/// Three tracks; "Song 2" cannot be read.
fn unreadable() -> AppState {
    let mut s = state(1, 3);
    let playlist = s.playlists.first_id().unwrap();
    let track = s.playlists.get(playlist).unwrap().entries[1].track;
    fp_model::apply(
        &mut s,
        Command::SetFileState {
            track,
            state: FileState::Unreadable,
        },
    )
    .unwrap();
    s
}

#[test]
fn q9_1_the_popup_starts_with_the_reason_above_the_fields() {
    let (mut h, _) = harness(unreadable());
    h.get_by_label("Song 2").hover();
    h.run_steps(40);
    let reason = h
        .get_by_label_contains("Cannot read the file: /music/Song 2.mp3")
        .rect();
    let title_field = h.get_by_label("Title").rect();
    assert!(reason.bottom() <= title_field.top() + 0.5, "{reason:?} {title_field:?}");
}

#[test]
fn q9_1_a_readable_track_has_no_reason_line() {
    let (mut h, _) = harness(state(1, 3));
    h.get_by_label("Song 2").hover();
    h.run_steps(40);
    assert!(h.query_by_label("Title").is_some(), "the popup is there");
    assert!(h.query_by_label_contains("Cannot read").is_none());
    assert!(h.query_by_label_contains("File not found").is_none());
}

#[test]
fn q9_2_the_number_and_the_title_have_no_tooltip_of_their_own() {
    // With one tooltip per layer the row's popup is the only one: hovering
    // the icon of the number and the title give the same popup content.
    let (mut h, _) = harness(unreadable());
    h.get_by_label_contains(egui_phosphor::regular::WARNING).hover();
    h.run_steps(40);
    assert_eq!(
        h.query_all_by_label_contains("Cannot read the file").count(),
        1
    );
}

#[test]
fn q9_3_the_popup_never_shows_at_another_place_first() {
    let (mut h, _) = harness(unreadable());
    h.hover_at(h.get_by_label("Song 2").rect().center());
    let mut first = None;
    for _ in 0..80 {
        h.step();
        if let Some(n) = h.query_by_label("Path") {
            first = Some(n.rect());
            break;
        }
    }
    let first = first.expect("the popup appears");
    h.run_steps(5);
    let later = h.get_by_label("Path").rect();
    assert!(
        (first.min - later.min).length() < 0.5 && (first.size() - later.size()).length() < 0.5,
        "{first:?} then {later:?}"
    );
}

#[test]
fn the_popup_stays_inside_a_narrow_window() {
    let (mut h, _) = harness_sized(unreadable(), egui::vec2(380.0, 700.0), |ui| ui);
    h.get_by_label("Song 2").hover();
    h.run_steps(40);
    let reason = h.get_by_label_contains("Cannot read the file").rect();
    assert!(
        reason.left() >= 0.0 && reason.right() <= 380.5,
        "{reason:?}"
    );
}
```

- [x] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-app --test view q9_`
Expected: FAIL to compile (`TipField::Problem`, second argument).

Run (after fixing compilation in Step 3 with the table still unchanged, to see the real failures): `cargo test -p fp-app --test table_tooltip`
Expected: `q9_1_the_popup_starts_with_the_reason_above_the_fields` FAILS (no reason line in the popup, or the reason is a separate tooltip). If `q9_3` already passes before the fix (egui hides the sizing pass), keep it as a guard and say so in the commit body.

- [x] **Step 3: Implement**

`view.rs`: add `Problem` as the first variant of `TipField`, and

```rust
/// What the row tooltip shows for `track`: the reason the file cannot be
/// played first (`problem`, when there is one and it is not blank), then the
/// fields it has, in this order. Album artist, composer and comment are left
/// to the editor.
pub fn track_tooltip(track: &Track, problem: Option<&str>) -> Vec<(TipField, String)> {
    let mut lines = Vec::new();
    if let Some(reason) = problem.filter(|r| !r.trim().is_empty()) {
        lines.push((TipField::Problem, reason.to_owned()));
    }
    // ... the rest of the function is unchanged (the `text` closure borrows `lines`)
```

(Declare `lines` before the closure as it is today; only the `if let` push above the closure is new.)

`table.rs`: add `/// Width of the track info popup: fixed, so egui's sizing pass gives the final size and the popup is never shown elsewhere first (Q9.3).` `const TIP_WIDTH: f32 = 380.0;` next to `ROW_HEIGHT`. Replace `track_tip`:

```rust
fn track_tip(ui: &mut Ui, scene: &Scene<'_>, track: &fp_model::Track) {
    let t = scene.i18n;
    // Never wider than the window (a narrow player in a small window).
    ui.set_width(TIP_WIDTH.min((ui.ctx().content_rect().width() - 16.0).max(120.0)));
    let lines = view::track_tooltip(track, scene.file_tip(track.id).as_deref());
    let mut rest = lines.as_slice();
    if let Some(((view::TipField::Problem, reason), tail)) = rest.split_first() {
        ui.add(
            egui::Label::new(
                RichText::new(format!("{}  {reason}", view::file_icon(track)))
                    .font(font(12.0))
                    .color(theme::AMBER),
            )
            .wrap(),
        );
        ui.add_space(4.0);
        rest = tail;
    }
    egui::Grid::new("track-tip")
        .num_columns(2)
        .spacing(vec2(10.0, 3.0))
        .show(ui, |ui| {
            for (field, value) in rest {
                let key = match field {
                    view::TipField::Problem => continue,
                    view::TipField::Title => "tip-field-title",
                    // ... the other arms unchanged
                };
                // ... label and value unchanged
            }
        });
}
```

Remove the file-error tooltips: in the number cell replace `let number = ui.add(...);` followed by the `if status == RowStatus::Unavailable && let Some(tip) = scene.file_tip(entry.track) { number.on_hover_text(tip); }` block by the bare `ui.add(...);`; in the Title cell do the same for `let title = ui.add(...)` and its `title.on_hover_text(tip)` block (and the comment above it). Keep the `tip-on-air-elsewhere` and `tip-next-again` tooltips (see Rulings).

- [x] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-app --test view`, `cargo test -p fp-app --test table_tooltip`, `cargo test -p fp-app --test main_screen hovering`, `cargo test -p fp-app --test cartwall_ui hovering_an_unavailable_cart_says_why`
Expected: PASS. The existing `hovering_*` tests of `main_screen.rs` must still pass unchanged: they hover the icon or the title and look for the reason, now served by the popup.

- [x] **Step 5: Commit**

```bash
cargo fmt --all
if cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace; then
  git add crates/fp-app
  git commit -m "fix(ui): the row popup shows the file error first and has a fixed width" -m "egui allows one tooltip per layer, so the row's popup hid the label tooltips that carried the reason. The reason is now the popup's first line, and a fixed width gives the sizing pass its final size, so the popup no longer jumps on its first frame." -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
fi
```

---

### Task 4: Drop geometry as pure functions (Q4, part 1)

Suggested executor: `sonnet`.

**Files:**
- Modify: `crates/fp-app/src/ui/table_layout.rs` (append; the `use` line becomes `use egui::{Pos2, Rect};`)
- Test: `crates/fp-app/tests/table_layout.rs`
- Locales: none. Docs: Task 8.

**Interfaces:**
- Consumes: nothing from earlier tasks.
- Produces:
  - `pub fn drop_index(body: Rect, scroll_y: f32, row_height: f32, len: usize, pointer: Pos2) -> Option<usize>`: the row boundary nearest to the pointer, `0..=len`; `None` when the pointer is not inside `body` or `row_height` is not a positive finite number.
  - `pub fn boundary_y(body: Rect, scroll_y: f32, row_height: f32, index: usize) -> Option<f32>`: the y of boundary `index` (the top edge of row `index`) in screen space, `None` when it is outside the visible `body`.
  - `pub fn on_column_edge(px: &[f32], left: f32, grab: f32, x: f32) -> bool`: `x` is within `grab` of an edge between two columns (the right edge of the last column is not one).

- [x] **Step 1: Write the failing tests**

Add `boundary_y, drop_index, on_column_edge` to the `use fp_app::ui::table_layout::{...}` line and `use egui::{Rect, pos2, vec2};`; append:

```rust
// --- drop_index, boundary_y, on_column_edge (operator feedback 4, Q4)

/// A body 500 x 280 points (ten rows of 28) starting at (100, 50).
fn body() -> Rect {
    Rect::from_min_size(pos2(100.0, 50.0), vec2(500.0, 280.0))
}

#[test]
fn the_top_of_the_body_is_boundary_zero() {
    assert_eq!(drop_index(body(), 0.0, 28.0, 5, pos2(200.0, 50.0)), Some(0));
    assert_eq!(drop_index(body(), 0.0, 28.0, 5, pos2(200.0, 60.0)), Some(0));
}

#[test]
fn the_nearest_boundary_wins_in_the_middle_of_the_list() {
    // Row 3 spans y = 134..162: its upper part is boundary 3, its lower 4.
    assert_eq!(drop_index(body(), 0.0, 28.0, 9, pos2(200.0, 139.0)), Some(3));
    assert_eq!(drop_index(body(), 0.0, 28.0, 9, pos2(200.0, 154.0)), Some(4));
}

#[test]
fn below_the_last_row_the_index_is_the_length() {
    assert_eq!(drop_index(body(), 0.0, 28.0, 5, pos2(200.0, 320.0)), Some(5));
}

#[test]
fn a_scrolled_body_shifts_the_boundaries() {
    // Scrolled by two rows: the top of the view is boundary 2.
    assert_eq!(drop_index(body(), 56.0, 28.0, 100, pos2(200.0, 51.0)), Some(2));
    // Scrolled by ten and a half rows: the top of the view is half way
    // between boundaries 10 and 11; the tie goes down.
    assert_eq!(drop_index(body(), 294.0, 28.0, 100, pos2(200.0, 50.0)), Some(11));
}

#[test]
fn an_empty_list_has_the_one_boundary_zero() {
    assert_eq!(drop_index(body(), 0.0, 28.0, 0, pos2(200.0, 200.0)), Some(0));
}

#[test]
fn outside_the_body_there_is_no_target() {
    for pointer in [
        pos2(200.0, 40.0),  // the header
        pos2(90.0, 100.0),  // left of the table
        pos2(601.0, 100.0), // the scroll bar and beyond
        pos2(200.0, 331.0), // below the table
        pos2(f32::NAN, 100.0),
    ] {
        assert_eq!(drop_index(body(), 0.0, 28.0, 5, pointer), None, "{pointer:?}");
    }
}

#[test]
fn a_broken_row_height_gives_no_target() {
    for height in [0.0, -28.0, f32::NAN, f32::INFINITY] {
        assert_eq!(drop_index(body(), 0.0, height, 5, pos2(200.0, 100.0)), None);
    }
}

#[test]
fn a_boundary_is_drawn_at_the_top_of_its_row() {
    assert_eq!(boundary_y(body(), 0.0, 28.0, 0), Some(50.0));
    assert_eq!(boundary_y(body(), 0.0, 28.0, 3), Some(134.0));
    assert_eq!(boundary_y(body(), 56.0, 28.0, 2), Some(50.0));
}

#[test]
fn a_boundary_outside_the_view_has_no_y() {
    assert_eq!(boundary_y(body(), 56.0, 28.0, 1), None, "above the view");
    assert_eq!(boundary_y(body(), 56.0, 28.0, 12), Some(330.0), "the bottom edge");
    assert_eq!(boundary_y(body(), 56.0, 28.0, 13), None, "below the view");
}

#[test]
fn the_interior_column_edges_are_grab_zones_and_the_last_edge_is_not() {
    let px = [60.0, 200.0, 100.0, 140.0];
    // Edges at 160, 360 and 460 for a table starting at x = 100.
    assert!(on_column_edge(&px, 100.0, 4.0, 160.0));
    assert!(on_column_edge(&px, 100.0, 4.0, 163.9));
    assert!(on_column_edge(&px, 100.0, 4.0, 456.5));
    assert!(!on_column_edge(&px, 100.0, 4.0, 165.0));
    assert!(!on_column_edge(&px, 100.0, 4.0, 100.0), "the table's own edge");
    assert!(!on_column_edge(&px, 100.0, 4.0, 600.0), "the last column's right edge");
    assert!(!on_column_edge(&[], 100.0, 4.0, 100.0));
}
```

- [x] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-app --test table_layout the_top_of_the_body`
Expected: FAIL to compile (`drop_index` not found).

- [x] **Step 3: Implement**

Append to `table_layout.rs`:

```rust
/// The row boundary a dragged entry would be dropped at (operator feedback 4,
/// Q4): the one nearest to the pointer, from `0` (before the first row) to
/// `len` (after the last). It comes from the pointer and the table's
/// geometry alone, never from the widget under the pointer. `body` is the
/// visible part of the table body (without the header and the scroll bar)
/// and `scroll_y` how far it is scrolled. `None` when the pointer is not
/// inside `body`.
pub fn drop_index(
    body: Rect,
    scroll_y: f32,
    row_height: f32,
    len: usize,
    pointer: Pos2,
) -> Option<usize> {
    if !body.contains(pointer) || !(row_height.is_finite() && row_height > 0.0) {
        return None;
    }
    let y = pointer.y - body.top() + if scroll_y.is_finite() { scroll_y } else { 0.0 };
    let boundary = (y / row_height).round().max(0.0);
    Some((boundary as usize).min(len))
}

/// The screen y of boundary `index` (the top edge of row `index`), when it is
/// inside the visible `body`; the bar is drawn there even if the rows next
/// to it were not built this frame.
pub fn boundary_y(body: Rect, scroll_y: f32, row_height: f32, index: usize) -> Option<f32> {
    let y = body.top() + index as f32 * row_height - scroll_y;
    (y.is_finite() && y >= body.top() - 0.5 && y <= body.bottom() + 0.5).then_some(y)
}

/// `x` is within `grab` points of an edge between two columns of a table
/// whose left side is at `left` (the column resize grab zones: a drop there
/// has no target, Q4.2). The right edge of the last column is not one.
pub fn on_column_edge(px: &[f32], left: f32, grab: f32, x: f32) -> bool {
    let mut edge = left;
    px.iter()
        .take(px.len().saturating_sub(1))
        .any(|w| {
            edge += w;
            (x - edge).abs() <= grab
        })
}
```

- [x] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-app --test table_layout`
Expected: PASS (new and old).

- [x] **Step 5: Commit**

```bash
cargo fmt --all
if cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace; then
  git add crates/fp-app
  git commit -m "feat(ui): pure geometry for the drop index of a dragged entry" -m "Prepares the fix of the drop line: the index will come from the pointer and the table body's geometry, not from the widget under the pointer." -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
fi
```

---

### Task 5: The drop line follows the geometry, per player (Q4, part 2)

Suggested executor: `opus` (a rewrite of the drag and drop block with several interacting frames; the end-of-drag and two-table cases are subtle).

**Files:**
- Modify: `crates/fp-app/src/ui/app.rs` (`DropTarget` L60–64, the file-drop fallback ~L1304), `crates/fp-app/src/ui/table.rs` (`track_table`: the `drop` variable L165, the `line` closure L289–304, the per-row drag code L512–530, `hovered_row`/`pointer_row`/`released` declarations L160–163, the drop block L552–605, the `.body(...)` call to keep its output)
- Test: `crates/fp-app/tests/table_drop_ui.rs` (create)
- Locales: none. Docs: Task 8 (`docs/technical/ui.md`, `docs/user/playlists.md`).

**Interfaces:**
- Consumes: `table_layout::{drop_index, boundary_y, on_column_edge}` (Task 4), `ScrollAreaOutput::{inner_rect, state.offset}` returned by `TableBuilder::body`, `px` (the column widths of the frame), `ROW_HEIGHT`.
- Produces: `DropTarget { player: PlayerId, playlist: PlaylistId, index: usize }` (`pub(crate)`); `ViewState::drop` and `ViewState::file_drop` hold it keyed by player and playlist.

- [x] **Step 1: Write the failing tests**

Create `crates/fp-app/tests/table_drop_ui.rs`:

```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Operator feedback 4, Q4: the violet drop line.

mod support;

use egui::{Event, Modifiers, PointerButton, Pos2, Rect, pos2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::AppUi;
use fp_app::ui::theme;
use fp_model::{Command, PlayerId};
use support::{Fake, harness, state};

fn press(h: &mut Harness<'_, AppUi>, at: Pos2, pressed: bool) {
    h.event(Event::PointerButton {
        pos: at,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    });
}

/// Presses at `from` and moves to `to` in steps, button still down.
fn begin_drag(h: &mut Harness<'_, AppUi>, from: Pos2, to: Pos2) {
    h.event(Event::PointerMoved(from));
    h.run_steps(1);
    press(h, from, true);
    h.run_steps(1);
    for step in 1..=10 {
        let t = step as f32 / 10.0;
        h.event(Event::PointerMoved(pos2(
            from.x + (to.x - from.x) * t,
            from.y + (to.y - from.y) * t,
        )));
        h.run_steps(1);
    }
    h.run_steps(2);
}

fn release(h: &mut Harness<'_, AppUi>, at: Pos2) {
    press(h, at, false);
    h.run_steps(3);
}

/// The violet drop lines drawn in the last frame: wide, 2 points high.
fn bars(h: &Harness<'_, AppUi>) -> Vec<Rect> {
    h.output()
        .shapes
        .iter()
        .filter_map(|s| match &s.shape {
            egui::Shape::Rect(r)
                if r.fill == theme::ACCENT && r.rect.width() > 100.0 && r.rect.height() < 3.0 =>
            {
                Some(r.rect)
            }
            _ => None,
        })
        .collect()
}

fn moves(fake: &Fake) -> Vec<Command> {
    fake.take_sent()
        .into_iter()
        .filter(|c| matches!(c, Command::MoveEntry { .. }))
        .collect()
}

#[test]
fn the_bar_is_at_the_boundary_under_the_pointer() {
    let (mut h, _) = harness(state(1, 5));
    let from = h.get_by_label("Song 1").rect().center();
    let row3 = h.get_by_label("Song 3").rect().center();
    // The lower part of row 3 is the boundary between rows 3 and 4.
    let to = pos2(row3.x, row3.y + 10.0);
    begin_drag(&mut h, from, to);
    let found = bars(&h);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!((found[0].top() - (row3.y + 14.0)).abs() < 1.5, "{found:?} {row3:?}");
}

#[test]
fn releasing_inserts_at_that_boundary_in_the_playlist() {
    let (mut h, fake) = harness(state(1, 5));
    let (e, playlist) = (fake.entries(), fake.state.load().playlists.first_id().unwrap());
    let from = h.get_by_label("Song 1").rect().center();
    let row3 = h.get_by_label("Song 3").rect().center();
    let to = pos2(row3.x, row3.y + 10.0);
    begin_drag(&mut h, from, to);
    fake.take_sent();
    release(&mut h, to);
    assert_eq!(
        moves(&fake),
        vec![Command::MoveEntry { entry: e[0], to: playlist, index: 3 }]
    );
    assert!(bars(&h).is_empty(), "no bar is left after the drop");
}

#[test]
fn only_the_table_under_the_pointer_draws_the_bar() {
    let mut s = state(2, 5);
    let playlist = s.playlists.first_id().unwrap();
    let p2: PlayerId = s.players[1].id;
    fp_model::apply(&mut s, Command::ShowPlaylist(p2, playlist)).unwrap();
    let (mut h, _) = harness(s);
    let mut songs: Vec<Rect> = h.query_all_by_label("Song 1").map(|n| n.rect()).collect();
    songs.sort_by(|a, b| a.left().total_cmp(&b.left()));
    let (left, right) = (songs[0], songs[1]);
    let to = pos2(left.center().x, left.center().y + 3.0 * 28.0);
    begin_drag(&mut h, left.center(), to);
    let found = bars(&h);
    assert_eq!(found.len(), 1, "the other player shows the same playlist: {found:?}");
    assert!(found[0].right() <= right.left() + 1.0, "{found:?} {right:?}");
}

#[test]
fn releasing_over_the_header_drops_nothing() {
    let (mut h, fake) = harness(state(1, 5));
    let from = h.get_by_label("Song 2").rect().center();
    let header = h.get_by_label("TITLE").rect().center();
    begin_drag(&mut h, from, header);
    assert!(bars(&h).is_empty(), "{:?}", bars(&h));
    fake.take_sent();
    release(&mut h, header);
    assert!(moves(&fake).is_empty());
}

#[test]
fn releasing_on_a_column_edge_drops_nothing() {
    let (mut h, fake) = harness(state(1, 5));
    let from = h.get_by_label("Song 2").rect().center();
    // The Title column ends where the Artist header's cell begins.
    let artist = h.get_by_label("ARTIST").rect();
    let on_edge = pos2(artist.left() - 8.0, from.y + 28.0);
    begin_drag(&mut h, from, on_edge);
    assert!(bars(&h).is_empty(), "{:?}", bars(&h));
    fake.take_sent();
    release(&mut h, on_edge);
    assert!(moves(&fake).is_empty());
}

#[test]
fn an_empty_playlist_accepts_a_drop_at_zero() {
    // Player 1 shows the full list, player 2 an empty one.
    let mut s = state(2, 2);
    fp_model::apply(&mut s, Command::CreatePlaylist { name: "Other".into() }).unwrap();
    let first = s.playlists.first_id().unwrap();
    let other = s.playlists.iter().find(|p| p.id != first).unwrap().id;
    let p2: PlayerId = s.players[1].id;
    fp_model::apply(&mut s, Command::ShowPlaylist(p2, other)).unwrap();
    let (mut h, fake) = harness(s);
    let e = fake.entries();
    let from = h.get_by_label("Song 1").rect().center();
    let mut headers: Vec<Rect> = h.query_all_by_label("TITLE").map(|n| n.rect()).collect();
    headers.sort_by(|a, b| a.left().total_cmp(&b.left()));
    let empty = headers[1];
    let to = pos2(empty.left() + 40.0, empty.bottom() + 60.0);
    begin_drag(&mut h, from, to);
    let found = bars(&h);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!((found[0].top() - empty.bottom()).abs() < 3.0, "{found:?} {empty:?}");
    fake.take_sent();
    release(&mut h, to);
    assert_eq!(
        moves(&fake),
        vec![Command::MoveEntry { entry: e[0], to: other, index: 0 }]
    );
}

#[test]
fn the_bar_follows_the_pointer_in_a_scrolled_long_list() {
    let (mut h, _) = harness(state(1, 200));
    let from = h.get_by_label("Song 2").rect().center();
    let to = pos2(from.x, from.y + 5.0 * 28.0);
    begin_drag(&mut h, from, to);
    h.event(Event::MouseWheel {
        unit: egui::MouseWheelUnit::Line,
        delta: egui::vec2(0.0, -5.0),
        phase: egui::TouchPhase::Move,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(4);
    let found = bars(&h);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!((found[0].center().y - to.y).abs() <= 15.0, "{found:?} {to:?}");
}
```

- [x] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-app --test table_drop_ui`
Expected: `the_bar_is_at_the_boundary_under_the_pointer` or `only_the_table_under_the_pointer_draws_the_bar` FAIL (two bars, or one at the wrong place); `releasing_on_a_column_edge_drops_nothing` and `releasing_over_the_header_drops_nothing` FAIL (the release below or on the handle still moves the entry).

- [x] **Step 3: Implement**

`app.rs`: `DropTarget` gains `pub player: PlayerId`. In `file_drops`'s `fallback` closure add `player: player.id`. Fix any other constructor the compiler reports.

`table.rs::track_table`:
1. Delete `hovered_row`, `pointer_row` and `released` (L160–163), the `let drop = view_state.drop.filter(...)` line (L165) and the whole `if let Some(d) = drop { ... }` block inside the `line` closure (L289–304).
2. Delete the per-row code that sets `pointer_row`, `hovered_row` and `released` (L516–530: the `if let Some(p) = pointer.filter(...)`, the `dnd_hover_payload` block and the `dnd_release_payload` block). Keep `response.drag_started()` and `dragged`.
3. Keep the builder's output: `let output = builder.header(...).body(...);` (the `.body(|body| { ... })` call returns `ScrollAreaOutput<()>`).
4. Replace everything from `// Drop target for entries dragged inside the app.` (L552) to the end of the `// OS file drops ...` block (L605) by:

```rust
// Q4: the drop target comes from the pointer and the body's geometry, and
// belongs to this table alone: it is keyed by player and playlist.
let body_rect = output.inner_rect;
let scroll_y = output.state.offset.y;
let grab = ui.style().interaction.resize_grab_radius_side;
let target = pointer
    .filter(|p| !table_layout::on_column_edge(&px, area.left(), grab, p.x))
    .and_then(|p| {
        table_layout::drop_index(body_rect, scroll_y, ROW_HEIGHT, entries.len(), p)
    });
let me = |d: &DropTarget| d.player == player && d.playlist == playlist;
if egui::DragAndDrop::has_payload_of_type::<DragEntry>(ui.ctx()) {
    match target {
        Some(index) => {
            view_state.drop = Some(DropTarget { player, playlist, index });
            // The line is drawn after the body, from the same geometry, at
            // the boundary even when the rows next to it are not built.
            if let Some(y) = table_layout::boundary_y(body_rect, scroll_y, ROW_HEIGHT, index) {
                let top = if index > 0 && index == entries.len() { y - 2.0 } else { y };
                ui.painter().with_clip_rect(body_rect).rect_filled(
                    Rect::from_min_size(pos2(body_rect.left(), top), vec2(body_rect.width(), 2.0)),
                    0.0,
                    theme::ACCENT,
                );
            }
        }
        None if view_state.drop.as_ref().is_some_and(me) => view_state.drop = None,
        None => {}
    }
    if ui.input(|i| i.pointer.any_released())
        && let Some(index) = target
        && let Some(payload) = egui::DragAndDrop::payload::<DragEntry>(ui.ctx())
    {
        scene.ctl.send(Command::MoveEntry {
            entry: payload.entry,
            to: playlist,
            index,
        });
        view_state.drop = None;
    }
}
// OS file drops follow the same rules (Q4.6).
if let Some(index) = target {
    view_state.file_drop = Some(DropTarget { player, playlist, index });
}
```

Remove the now-unused `pointer_in` and any unused imports the compiler reports. `area` is still used by the wheel test, the resize handles and the column-move line.

- [x] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-app --test table_drop_ui`, `cargo test -p fp-app --test main_screen files_dropped`, `cargo test -p fp-app --test table_header_ui`, `cargo test -p fp-app --test table_columns_ui`, `cargo test -p fp-app --test table_follow`
Expected: PASS. If `inner_rect` turns out to include the scroll bar (the `outside_the_body` pointer test over the bar fails), subtract `ui.spacing().scroll.allocated_width()` from its right side when the content is taller than the view, and note it in the commit body.

- [x] **Step 5: Commit**

```bash
cargo fmt --all
if cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace; then
  git add crates/fp-app
  git commit -m "fix(ui): the drop line is drawn at the row boundary under the pointer" -m "The target was keyed by the playlist only (two players showing it both drew the line) and came from per-row hover, which is false over the column handles, the header and the scroll bar, so it fell to the end of the list. It now comes from the pointer and the body's geometry, is keyed by player and playlist, and is drawn after the body." -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
fi
```

---

### Task 6: Unreadable files are looked at again by size and modification time (Q10, services)

Suggested executor: `opus` (a protocol between two threads and a state machine with several races).

**Files:**
- Modify: `crates/fp-app/src/services.rs`: the `Probe` struct and `present` (L22–90), `ServiceRequest` (L168–173), the `Services` fields (L225–280) and `new` (L283–320), `analysis_step`'s request loop (L383–388), `restart_analysis` (L456–466), `recheck_missing` (L468–525), `submit_new` (the urgent branch, ~L632–658), a `#[cfg(test)] mod tests` at the end
- Test: `crates/fp-app/tests/services.rs`
- Locales: none. Docs: Task 8 (`docs/technical/analysis.md`, `docs/user/troubleshooting.md`).

**Interfaces:**
- Consumes: `tuning.missing_recheck_ms`, `Command::SetFileState`, `ConductorHandle::send`, the `forced`, `failed`, `in_flight`, `done`, `retried` sets.
- Produces:
  - `ServiceRequest::ReanalyseTrack(TrackId)` (still `Copy + Eq`).
  - `type Stamp = (u64, SystemTime)`, `enum Seen { New, Unknown, At(Stamp) }`, `enum Look { Record(Seen), Keep, Changed, Gone }` and `fn look(seen: Seen, now: std::io::Result<Stamp>) -> Look` (private; unit-tested in the file).
  - `Services::stamps: HashMap<TrackId, Seen>` (one entry per track whose file is `Unreadable` in the model), `Services::urgent: HashSet<TrackId>`.
  - Test hook `#[cfg(feature = "test-hooks")] pub fn looked_at(&self) -> usize`: the number of unreadable tracks whose first look has been answered.

- [ ] **Step 1: Write the failing tests**

Inline unit tests, at the end of `services.rs`:

```rust
#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use std::io::{Error, ErrorKind};

    fn at(secs: u64) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(secs)
    }

    #[test]
    fn the_first_look_records_what_it_sees() {
        assert_eq!(look(Seen::New, Ok((10, at(5)))), Look::Record(Seen::At((10, at(5)))));
        assert_eq!(look(Seen::Unknown, Ok((10, at(5)))), Look::Record(Seen::At((10, at(5)))));
    }

    #[test]
    fn the_same_size_and_time_is_unchanged_and_a_new_one_is_a_change() {
        let seen = Seen::At((10, at(5)));
        assert_eq!(look(seen, Ok((10, at(5)))), Look::Keep);
        assert_eq!(look(seen, Ok((11, at(5)))), Look::Changed);
        assert_eq!(look(seen, Ok((10, at(6)))), Look::Changed);
    }

    #[test]
    fn a_file_that_is_gone_is_gone_whatever_was_seen() {
        for seen in [Seen::New, Seen::Unknown, Seen::At((1, at(1)))] {
            let gone = Err(Error::from(ErrorKind::NotFound));
            assert_eq!(look(seen, gone), Look::Gone);
        }
    }

    #[test]
    fn look_keeps_what_it_knows_when_the_stat_fails() {
        let denied = || Err(Error::from(ErrorKind::PermissionDenied));
        assert_eq!(look(Seen::New, denied()), Look::Record(Seen::Unknown));
        assert_eq!(look(Seen::Unknown, denied()), Look::Keep);
        assert_eq!(look(Seen::At((1, at(1))), denied()), Look::Keep);
    }
}
```

Integration tests appended to `tests/services.rs` (add `use std::time::SystemTime;`):

```rust
/// A file with an audio extension that cannot be decoded.
fn garbage(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, b"not audio at all").unwrap();
    path
}

fn all_in(r: &Rig, state: FileState) -> bool {
    r.handle.model.load().library.iter().all(|t| t.file_state == state)
}

/// The track is unreadable and its first look has been answered, so its
/// size and modification time are recorded.
fn unreadable_and_stamped(r: &Rig) -> bool {
    all_in(r, FileState::Unreadable) && r.services.looked_at() == 1
}

fn quiet_for(r: &mut Rig, ms: u64) {
    let until = r.now + Duration::from_millis(ms);
    r.run_until("a quiet period", |r| r.now >= until);
}

#[test]
fn an_unreadable_file_that_did_not_change_is_not_analysed_again() {
    let dir = tempfile::tempdir().unwrap();
    let bad = garbage(dir.path(), "bad.wav");
    let mut r = recheck_rig(&[bad], dir);
    r.run_until("unreadable and stamped", unreadable_and_stamped);
    let analyses = r.analyses.load(Ordering::SeqCst);
    let version = r.handle.telemetry.load().model_version;
    quiet_for(&mut r, 3_500);
    assert_eq!(r.analyses.load(Ordering::SeqCst), analyses);
    assert_eq!(r.handle.telemetry.load().model_version, version, "nothing is saved");
}

#[test]
fn an_unreadable_file_that_changed_size_is_analysed_again() {
    let dir = tempfile::tempdir().unwrap();
    let bad = garbage(dir.path(), "bad.wav");
    let staging = tempfile::tempdir().unwrap();
    let mut r = recheck_rig(std::slice::from_ref(&bad), dir);
    r.run_until("unreadable and stamped", unreadable_and_stamped);
    // Written elsewhere, then moved: never seen half-written.
    std::fs::rename(wav(staging.path(), "bad.wav", 1), &bad).unwrap();
    r.run_until("playable again", |r| {
        r.handle.model.load().library.iter().all(|t| t.file_state == FileState::Ok && t.analyzed)
    });
    assert_eq!(r.analyses.load(Ordering::SeqCst), 2, "one failure and one success");
}

/// The same size with a new modification time is a change too, and a file
/// that fails again is looked at with its new stat: no loop.
#[test]
fn a_file_that_fails_again_is_looked_at_with_its_new_stat() {
    let dir = tempfile::tempdir().unwrap();
    let bad = garbage(dir.path(), "bad.wav");
    let mut r = recheck_rig(std::slice::from_ref(&bad), dir);
    r.run_until("unreadable and stamped", unreadable_and_stamped);
    let file = std::fs::OpenOptions::new().write(true).open(&bad).unwrap();
    file.set_modified(SystemTime::now() + Duration::from_secs(60)).unwrap();
    drop(file);
    r.run_until("a second analysis", |r| r.analyses.load(Ordering::SeqCst) == 2);
    r.run_until("stamped again", unreadable_and_stamped);
    quiet_for(&mut r, 3_500);
    assert_eq!(r.analyses.load(Ordering::SeqCst), 2, "no third analysis");
    assert!(all_in(&r, FileState::Unreadable));
}

#[test]
fn an_unreadable_file_that_disappeared_becomes_missing() {
    let dir = tempfile::tempdir().unwrap();
    let bad = garbage(dir.path(), "bad.wav");
    let mut r = recheck_rig(std::slice::from_ref(&bad), dir);
    r.run_until("unreadable and stamped", unreadable_and_stamped);
    std::fs::remove_file(&bad).unwrap();
    r.run_until("missing", |r| all_in(r, FileState::Missing));
}

#[test]
fn reanalyse_track_analyses_a_failed_file_at_once_whatever_its_state() {
    let dir = tempfile::tempdir().unwrap();
    let bad = garbage(dir.path(), "bad.wav");
    let mut r = recheck_rig(&[bad], dir);
    r.run_until("unreadable and stamped", unreadable_and_stamped);
    let track = r.handle.model.load().library.iter().next().unwrap().id;
    r.services.requests().send(ServiceRequest::ReanalyseTrack(track)).unwrap();
    r.run_until("a second analysis", |r| r.analyses.load(Ordering::SeqCst) == 2);
}

#[test]
fn reanalyse_track_brings_back_a_track_that_playback_marked_unreadable() {
    let dir = tempfile::tempdir().unwrap();
    let good = wav(dir.path(), "good.wav", 1);
    let mut r = recheck_rig(&[good], dir);
    r.run_until("analysed", |r| r.handle.model.load().library.iter().all(|t| t.analyzed));
    let track = r.handle.model.load().library.iter().next().unwrap().id;
    // What a playback failure does (the reducer's `SourceFailed`).
    assert!(r.handle.send(Command::SetFileState { track, state: FileState::Unreadable }));
    r.run_until("unreadable and stamped", unreadable_and_stamped);
    r.services.requests().send(ServiceRequest::ReanalyseTrack(track)).unwrap();
    r.run_until("playable again", |r| all_in(r, FileState::Ok));
    assert_eq!(r.analyses.load(Ordering::SeqCst), 2);
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-app --lib services::tests`
Expected: FAIL to compile (`look`, `Seen`, `Look` not found).

Run: `cargo test -p fp-app --test services an_unreadable_file`
Expected: FAIL to compile (`looked_at`, `ReanalyseTrack`).

- [ ] **Step 3: Implement**

`services.rs`, add `use std::path::Path;` and `use std::time::SystemTime;` (merge with the existing imports), then:

1. **Types and the pure rule**, next to `Probe`:

```rust
/// A file's size and modification time: what tells that it changed.
type Stamp = (u64, SystemTime);

/// What the probe has seen of an unreadable file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Seen {
    /// Not looked at yet.
    New,
    /// Looked at, and the `stat` failed.
    Unknown,
    At(Stamp),
}

/// What one look at an unreadable file means (operator feedback 4, Q10).
#[derive(Debug, PartialEq)]
enum Look {
    /// First stat (or the first that worked): remember it.
    Record(Seen),
    /// Nothing to do.
    Keep,
    /// The size or the modification time changed: analyse it again.
    Changed,
    /// The file is not there any more.
    Gone,
}

fn look(seen: Seen, now: std::io::Result<Stamp>) -> Look {
    match (now, seen) {
        (Ok(now), Seen::New | Seen::Unknown) => Look::Record(Seen::At(now)),
        (Ok(now), Seen::At(before)) if now == before => Look::Keep,
        (Ok(_), Seen::At(_)) => Look::Changed,
        (Err(e), _) if e.kind() == std::io::ErrorKind::NotFound => Look::Gone,
        (Err(_), Seen::New) => Look::Record(Seen::Unknown),
        (Err(_), _) => Look::Keep,
    }
}

/// The `stat` of `path`. It never opens or decodes the file.
fn stamp_of(path: &Path) -> std::io::Result<Stamp> {
    let meta = std::fs::metadata(path)?;
    Ok((meta.len(), meta.modified()?))
}
```

2. **The probe** carries both lists. Replace `Probe`'s channel types: `paths: Sender<Job>`, `answers: Receiver<Answer>` (rename `found` to `answers` everywhere):

```rust
/// One look of the probe thread: files not found, and unreadable files with
/// what was seen of them.
struct Job {
    missing: Vec<(TrackId, PathBuf)>,
    unreadable: Vec<(TrackId, PathBuf, Seen)>,
}

#[derive(Default)]
struct Answer {
    /// Missing files that exist now.
    found: Vec<TrackId>,
    recorded: Vec<(TrackId, Seen)>,
    changed: Vec<TrackId>,
    gone: Vec<TrackId>,
}

fn run(job: Job) -> Answer {
    let mut answer = Answer {
        found: present(job.missing),
        ..Answer::default()
    };
    for (id, path, seen) in job.unreadable {
        match look(seen, stamp_of(&path)) {
            Look::Record(s) => answer.recorded.push((id, s)),
            Look::Keep => {}
            Look::Changed => answer.changed.push(id),
            Look::Gone => answer.gone.push(id),
        }
    }
    answer
}
```

`Probe::spawn` becomes `crossbeam_channel::unbounded::<Job>()` / `unbounded::<Answer>()` and the thread does `answers.send(run(batch))`.

3. **Requests and state.** `ServiceRequest` gains `/// Analyse this track at once, whatever its state (the row menu's Re-analyse). ReanalyseTrack(TrackId),`. `Services` gains `stamps: HashMap<TrackId, Seen>` and `urgent: HashSet<TrackId>` (initialised empty in `new`). In `analysis_step`'s request loop add `ServiceRequest::ReanalyseTrack(id) => self.reanalyse_track(state, id),` and add:

```rust
/// The row menu's Re-analyse: forget the failure and analyse this track
/// now, ahead of the library. Manual markers are kept by the model.
fn reanalyse_track(&mut self, state: &AppState, id: TrackId) {
    if state.library.get(id).is_none() {
        return;
    }
    if self.in_flight.remove(&id) {
        self.analyzer.cancel(id);
    }
    self.done.remove(&id);
    self.failed.remove(&id);
    self.retried.remove(&id);
    self.stamps.remove(&id);
    self.forced.insert(id);
    self.urgent.insert(id);
}
```

In `restart_analysis` add `self.stamps.clear();`. In `submit_new`'s pruning block add `self.stamps.retain(|id, _| known.contains(id)); self.urgent.retain(|id| known.contains(id));` and change `if wanted.contains(&id) { self.analyzer.submit_urgent(...) }` to `if wanted.contains(&id) || self.urgent.remove(&id) {`.

4. **`recheck_missing`** becomes (the doc comment mentions both kinds of files and no longer says unreadable files wait for Re-analyse all):

```rust
fn recheck_missing(&mut self, state: &AppState, now: Instant) {
    let Some(probe) = &mut self.probe else {
        return;
    };
    loop {
        match probe.answers.try_recv() {
            Ok(answer) => {
                probe.busy = false;
                // One in flight already (Re-analyse all) needs no second analysis.
                for id in answer.found.into_iter().filter(|id| !self.in_flight.contains(id)) {
                    self.failed.remove(&id);
                    self.forced.insert(id);
                }
                for (id, seen) in answer.recorded {
                    if let Some(known) = self.stamps.get_mut(&id) {
                        *known = seen;
                    }
                }
                for id in answer.changed.into_iter().filter(|id| !self.in_flight.contains(id)) {
                    self.stamps.remove(&id);
                    self.failed.remove(&id);
                    self.forced.insert(id);
                }
                for id in answer.gone {
                    // Q10.3: it follows the missing-file recheck from here.
                    self.stamps.remove(&id);
                    self.conductor.send(Command::SetFileState {
                        track: id,
                        state: FileState::Missing,
                    });
                }
            }
            Err(crossbeam_channel::TryRecvError::Empty) => break,
            Err(crossbeam_channel::TryRecvError::Disconnected) => {
                tracing::error!("the file probe stopped; missing files stay missing");
                self.probe = None;
                return;
            }
        }
    }
    // Every unreadable track, from the analysis or from playback, has an
    // entry; one that is not unreadable any more loses it.
    self.stamps.retain(|id, _| {
        state.library.get(*id).is_some_and(|t| t.file_state == FileState::Unreadable)
    });
    for t in state.library.iter().filter(|t| t.file_state == FileState::Unreadable) {
        self.stamps.entry(t.id).or_insert(Seen::New);
    }
    let interval =
        Duration::from_secs_f64(state.config.tuning.missing_recheck_ms.max(0.0) / 1000.0);
    let due = match self.last_recheck {
        Some(last) => now.saturating_duration_since(last) >= interval,
        None => {
            self.last_recheck = Some(now);
            false
        }
    };
    // A new failure is looked at once, without waiting for the timer, so
    // that the recorded stat is the one of the failure.
    let first_look = self
        .stamps
        .iter()
        .any(|(id, s)| *s == Seen::New && !self.in_flight.contains(id));
    // One look at a time: a hung mount only delays the next one.
    if probe.busy || !(due || first_look) {
        return;
    }
    if due {
        self.last_recheck = Some(now);
    }
    let missing: Vec<(TrackId, PathBuf)> = if due {
        state
            .library
            .iter()
            .filter(|t| t.file_state == FileState::Missing && !self.in_flight.contains(&t.id))
            .map(|t| (t.id, t.path.clone()))
            .collect()
    } else {
        Vec::new()
    };
    let unreadable: Vec<(TrackId, PathBuf, Seen)> = self
        .stamps
        .iter()
        .filter(|(id, seen)| !self.in_flight.contains(id) && (due || **seen == Seen::New))
        .filter_map(|(id, seen)| state.library.get(*id).map(|t| (*id, t.path.clone(), *seen)))
        .collect();
    if missing.is_empty() && unreadable.is_empty() {
        return;
    }
    if probe.paths.send(Job { missing, unreadable }).is_ok() {
        probe.busy = true;
    } else {
        tracing::error!("the file probe stopped; missing files stay missing");
        self.probe = None;
    }
}
```

5. **Test hook**, next to `tag_pass_on`:

```rust
/// How many unreadable tracks have had their first look answered.
#[cfg(feature = "test-hooks")]
pub fn looked_at(&self) -> usize {
    self.stamps.values().filter(|s| **s != Seen::New).count()
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-app --lib services::tests`, then `cargo test -p fp-app --test services` (each new test, and the older `a_missing_file_*` ones, which exercise the reworked loop).
Expected: PASS. If the garbage `.wav` is not reported as `Unreadable` (for example it is read as an empty file), change `garbage` to bytes that still fail the decoder and re-run `run_until("unreadable and stamped")` first: every test begins with it.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all
if cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace; then
  git add crates/fp-app
  git commit -m "fix(app): an unreadable file is analysed again when its size or time changes" -m "A file that failed once (a partial copy, a file being written) stayed unreadable after it was fixed until Re-analyse all. The probe thread now stats unreadable files on the missing-file timer, never opening them, and a change sends the track to the analysis pool. ReanalyseTrack analyses one track at once." -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
fi
```

---

### Task 7: The row menu's Re-analyse (Q10, UI)

Suggested executor: `sonnet`.

**Files:**
- Modify: `crates/fp-app/src/ui/app.rs` (`Scene` struct L236–248, its construction L614–624, a `request` method next to `file_tip`), `crates/fp-app/src/ui/table.rs` (`context_menu`, after the "Pre-listen on CUE" item L774–784), `crates/fp-app/locales/en-US/main.ftl` and `es-ES/main.ftl` (next to `menu-cue`, L71)
- Test: `crates/fp-app/tests/reanalyse_ui.rs` (create)
- Docs: Task 8 (`docs/user/playlists.md`).

**Interfaces:**
- Consumes: `ServiceRequest::ReanalyseTrack(TrackId)` (Task 6), `AppUi::with_services(Sender<ServiceRequest>)`, the row menu's `labelled` helper.
- Produces: `Scene::services: Option<&Sender<ServiceRequest>>`, `Scene::request(&self, ServiceRequest)` (a full queue or no services is ignored), Fluent key `menu-reanalyse`.

- [ ] **Step 1: Write the failing test**

Create `crates/fp-app/tests/reanalyse_ui.rs`:

```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Operator feedback 4, Q10.4: the row menu's Re-analyse.

mod support;

use egui_kittest::kittest::Queryable;
use fp_app::services::ServiceRequest;
use support::{harness, harness_from, state};

#[test]
fn the_row_menu_re_analyses_that_track() {
    let (tx, rx) = crossbeam_channel::bounded(4);
    let (mut h, fake) = harness_from(state(1, 3), move |ui| ui.with_services(tx));
    let track = fake.state.load().playlists.iter().next().unwrap().entries[1].track;
    h.get_all_by_label("Song 2").last().unwrap().click_secondary();
    h.run_steps(2);
    h.get_by_label("Re-analyse").click();
    h.run_steps(2);
    assert_eq!(rx.try_recv(), Ok(ServiceRequest::ReanalyseTrack(track)));
    assert!(rx.try_recv().is_err(), "one request");
}

#[test]
fn the_item_is_in_the_menu_even_without_services() {
    let (mut h, _) = harness(state(1, 3));
    h.get_all_by_label("Song 2").last().unwrap().click_secondary();
    h.run_steps(2);
    h.get_by_label("Re-analyse").click();
    h.run_steps(2);
    // Nothing to send to, and nothing panics.
    assert!(h.query_by_label("Re-analyse").is_none(), "the menu closed");
}
```

(`crossbeam_channel` is already a dev-usable dependency: `tests/outdated_notice.rs` uses it.)

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p fp-app --test reanalyse_ui`
Expected: FAIL (`no node with label "Re-analyse"`).

- [ ] **Step 3: Implement**

`app.rs`: add to `Scene` `pub services: Option<&'a Sender<ServiceRequest>>,`, set `services: self.services.as_ref(),` where `Scene` is built, and:

```rust
/// Asks the services thread for something. Without services (tests) or with
/// a full queue the request is dropped: it is only ever a convenience.
pub fn request(&self, request: ServiceRequest) {
    if let Some(services) = self.services {
        let _ = services.try_send(request);
    }
}
```

`table.rs::context_menu`, directly after the "Pre-listen on CUE" `if labelled(...).clicked() { ... }` block:

```rust
if labelled(ui, icon::ARROWS_CLOCKWISE, "menu-reanalyse", true).clicked() {
    scene.request(crate::services::ServiceRequest::ReanalyseTrack(track.id));
    ui.close();
}
```

Locales, after `menu-cue`: en-US `menu-reanalyse = Re-analyse`; es-ES `menu-reanalyse = Volver a analizar`.

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-app --test reanalyse_ui`, `cargo test -p fp-app --test i18n`, `cargo test -p fp-app --test next_row_ui`
Expected: PASS. Menu-order tests of other files that count menu items, if any, are updated here.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all
if cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace; then
  git add crates/fp-app
  git commit -m "feat(ui): a Re-analyse item in the track row menu" -m "It clears a file's failure and analyses that one track at once, so an operator no longer needs Re-analyse all for a single fixed file." -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
fi
```

---

### Task 8: Docs, spec "As built" and the self-check

Suggested executor: `sonnet`.

**Files:**
- Modify: `docs/user/players.md` (the CUE window list, ~L140–160), `docs/user/playlists.md` (row table L93–101, context menu L119–130, Drag and drop L191–200), `docs/user/troubleshooting.md` (L70–82), `docs/technical/ui.md` (`ui/table.rs` row L15, the view-state sentence ~L55, drag and drop, the CUE window), `docs/technical/analysis.md` (L218–228), `docs/superpowers/specs/2026-10-06-operator-feedback-4-design.md` (status line and an "As built" subsection at the end of §2), `README.md` only if `grep -n -i "re-analy\|tooltip\|drop indicator" README.md` finds a sentence these changes make wrong.
- Test: `cargo test -p fp-app --test i18n` and the whole suite (docs have no tests; the check is a read-through).

**Interfaces:** none.

- [ ] **Step 1: Verify the spec lines that the design spec already changed**

Run: `grep -n "operator feedback 4, Q\(2\|4\|5\|9\|10\)\|feedback 4, Q10" docs/superpowers/specs/2026-09-25-fauste-player-design.md docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md`
Expected: the lines listed in the Rulings (main spec L381, L453, L457, L458; feedback 2 L283, L319, L462). If one is missing, add it with the sentence given in spec §2 under "Spec lines that change".

- [ ] **Step 2: Write the user guide**

- `players.md`, CUE window: add "While the CUE is paused, its Pause button (shown as Resume) blinks amber, like the player's."
- `playlists.md`: in the row table add "Hourglass at the right of the title | The track is waiting for its analysis (hover: *Analysis pending*); it plays anyway, and the hourglass goes when the analysis finishes"; change the file-icon row to "hover the row: the popup starts with the reason, in amber, then the usual fields"; change the **Track tooltip** paragraph to say the popup is the only hover information on a row and starts with the reason for a missing or unreadable file; add **Re-analyse** to the context menu table ("Analyse this track again now, whatever its state. A fixed file that was unreadable is also picked up by itself: see Troubleshooting. Manual markers are kept"); in **Drag and drop** say the violet line is at the row boundary nearest to the pointer, appears only in the list under the pointer, and a drop over the header, the column edges or the scroll bar does nothing.
- `troubleshooting.md`: replace "A file that cannot be decoded is checked again only with **Settings → Analysis → Re-analyse all tracks**." by: it is checked again by itself on the same timer, by size and modification time (the file is not decoded again unless one of them changed, for example after a copy finishes); to check it at once use **Re-analyse** in its row menu or **Settings → Analysis → Re-analyse all tracks**.

- [ ] **Step 3: Write the technical docs**

- `ui.md`: describe `table_layout::{drop_index, boundary_y, on_column_edge}` and that `DropTarget` is keyed by player and playlist; the popup constant `TIP_WIDTH` and that the file error is the popup's first line; `widgets::{blink, paused_style}` shared by the player and the CUE window; `view::{analysis_pending, animating}`; `Scene::request`.
- `analysis.md`: replace "`Unreadable` files are not retried by themselves." with the Q10 rule: the probe `stat`s (never opens) unreadable files, `Services::stamps` records `Seen`, a change clears the failure and queues an analysis, "not found" becomes `Missing`, the first look is not timer-gated, `ReanalyseTrack` uses the urgent queue.

- [ ] **Step 4: Spec "As built" and status**

In the design spec set the status line to "Plan 1 implemented (branch `fix/feedback4-ui-fixes`); plans 2 to 5 not written." and add under §2 an "As built (plan 1)" list with one bullet per item and the Rulings of this plan, verbatim (they are the deviations).

- [ ] **Step 5: Self-check and commit**

Run `grep -rn -i "unreadable files are not retried\|waits for .Re-analyse all" docs README.md` (expected: no hits left). Then:

```bash
if cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace; then
  git add docs README.md
  git commit -m "docs: the pending flag, the popup, the drop line, the blinking CUE Pause and Re-analyse" -m "User guide, technical docs and the design spec's as-built notes for operator feedback 4, plan 1." -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
fi
```

Then follow CLAUDE.md: `superpowers:requesting-code-review` with a fresh reviewer on the most capable model (fix Critical and Important findings test-first, defer Minor ones in the ledger), push `fix/feedback4-ui-fixes`, open the pull request with `gh pr create` following `.github/pull_request_template.md` (title: `fix(ui): operator feedback 4, plan 1 (Q2, Q4, Q5, Q9, Q10)`), wait for CI with `gh pr checks --watch`, and merge with `gh pr merge --merge --delete-branch`.

---

## Self-review (against spec §2)

- **Q5** (Q5.1 blink in the CUE window: Task 1 `a_paused_cue_blinks_its_resume_button_amber`; Q5.2 no blink: `a_running_cue_does_not_blink`; Q5.3 repaint: `a_paused_cue_keeps_the_interface_repainting`; `blink` phases at 0.0, 0.49, 0.5, 1.0 and `paused_style`: `tests/blink.rs`). Locales none, docs `players.md`.
- **Q2** (Q2.1 pending flag: Task 2 `q2_1_*`, `every_row_waiting_*`; Q2.2 unplayable or analysed: `q2_2_*`, `a_pending_flag_goes_when_the_file_turns_unreadable`; Q2.3 goes on the next snapshot and outdated keeps its flag: `the_flag_disappears_when_the_analysis_arrives`, `q2_3_*`, `an_outdated_track_never_shows_the_hourglass`). `flag-analysis-pending` in both locales, `playlists.md`, `ui.md`.
- **Q9** (Q9.1 reason first, amber, wrapped: Task 3 `q9_1_*`; Q9.2 label tooltips removed: `q9_2_*`; Q9.3 fixed width, same place on frames 1 and 2: `q9_3_*`; Q9.4 carts: already true, guarded by `hovering_an_unavailable_cart_says_why`, see Rulings). Constant `TIP_WIDTH`, `view::track_tooltip` takes the reason.
- **Q4** (Q4.1 geometry: Task 4 `drop_index` tests and Task 5 `the_bar_is_at_the_boundary_under_the_pointer`; Q4.2 no target over header, handles, scroll bar, outside: `outside_the_body_there_is_no_target`, `releasing_over_the_header_drops_nothing`, `releasing_on_a_column_edge_drops_nothing`; Q4.3 per player: `only_the_table_under_the_pointer_draws_the_bar`; Q4.4 drop is playlist based: `releasing_inserts_at_that_boundary_in_the_playlist`; Q4.5 unbuilt rows and outside the view: `a_boundary_outside_the_view_has_no_y`, `the_bar_follows_the_pointer_in_a_scrolled_long_list`; Q4.6 OS file drops: `DropTarget` with player and `files_dropped_on_a_row_are_inserted_there` kept green).
- **Q10** (Q10.1 and Q10.2 record and compare stat on the probe timer without opening files: Task 6 `look` tests, `an_unreadable_file_that_did_not_change_is_not_analysed_again`, `an_unreadable_file_that_changed_size_is_analysed_again`; Q10.3 disappeared: `an_unreadable_file_that_disappeared_becomes_missing`; Q10.4 `ReanalyseTrack` and the menu: `reanalyse_track_*`, Task 7 `the_row_menu_re_analyses_that_track`; Q10.5 a failed analysis records the new stat: `a_file_that_fails_again_is_looked_at_with_its_new_stat`; Q10.6 off the services and UI threads: the stat is in `run` on the probe thread, one look at a time through `probe.busy`). `menu-reanalyse` in both locales, `playlists.md`, `troubleshooting.md`, `analysis.md`.
- **Placeholders:** none; every step has its code or its exact command.
- **Type consistency:** `blink`/`paused_style`/`animating` (Task 1), `analysis_pending` (Task 2), `track_tooltip(track, Option<&str>)` and `TipField::Problem` (Task 3), `drop_index`/`boundary_y`/`on_column_edge` and `DropTarget { player, playlist, index }` (Tasks 4 and 5), `Seen`/`Look`/`look`/`ServiceRequest::ReanalyseTrack(TrackId)`/`Scene::request` (Tasks 6 and 7) are named the same everywhere they appear.
- **Task order and dependencies:** 1, 2, 3 are independent; 5 needs 4; 7 needs 6; 8 last. Q5 first as the spec's order requires.
