# Shared Waveform Panel (Feedback 4, Plan 3) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The CUE window's waveform becomes the same panel as the player's: wheel zoom, pan, the Full view button, the intro and outro badges, the intro, outro and MIX markers, and marker editing (right-click menu and Alt-drag). Its zoom and drags are independent of the player's, and a marker edited in either place shows in both at once (Q7).

**Architecture:**
- **Extract with no behaviour change first.** Tasks 1–3 only move code. Task 1 pulls the pure `view::marker_fractions`, `view::intro_left` and `view::outro_left` out of `player_view`. Task 2 adds `WaveKey` and `WaveZooms` to `ui/wave_view.rs` and keys `ViewState::{wave_zoom, wave_menu, marker_drag}` by `WaveKey`; the player is the only caller and uses `WaveKey::Player(id)`. Task 3 moves `player.rs::wave` and `player.rs::edit_markers` into a new `ui/wave_panel.rs` behind `WavePanelInput`, and the pure wheel rules into `wave_view.rs`. After each of these tasks, every existing player waveform test (`tests/waveform_ui.rs`, `tests/markers_ui.rs`, `tests/waveform_view.rs`, `tests/view.rs`) passes unchanged.
- **Then adopt it in the CUE window.** Task 4 gives `CueWindowView` what the panel needs (markers against the whole file, the badges, the MIX look). Task 5 makes `cue_window.rs` a thin caller with `WaveKey::Cue(player)` and `&mut ViewState`, and a closed CUE window forgets its zoom, menu point and drag. Task 6 updates the docs.
- The panel only reads and writes the view state and sends `SetMarker` and `ResetMarkers` (they act on a track). It returns `WavePanelOutput { seek }`; the player maps it to `Command::Seek`, the CUE window to `Command::SeekCue`.

**Tech Stack:** Rust, egui/eframe 0.36.2, egui_kittest 0.36.2. No new dependency, so `cargo deny check` needs no new entry.

**Spec:** `docs/superpowers/specs/2026-10-06-operator-feedback-4-design.md` §4 (Q7, rules Q7.1–Q7.6) and §9 (global constraints). It depends on plan 2 (`2026-10-06-feedback4-plan2-play-before-analysis.md`), which changes the player's waveform (Q1, Q8) that this plan moves.

## Rulings

Decisions taken while writing the plan, where the spec is silent or this plan departs from it.

- Ruling: the line numbers below are those of `master` on 2026-10-06, before plan 2; the executor re-locates each by its function name after plan 2 is merged, and wherever plan 2 changed an expression this plan moves (in particular the player's `seekable:` and the stopped-player follow condition, Q8.6), the moved code keeps plan 2's expression, not the one quoted here — plan 3 runs after plan 2 (spec §1) but was written alongside it — cost if wrong: a moved line silently reverts a plan 2 rule; plan 2's own kittests (a click on a stopped player's waveform sends `Seek`) catch it.
- Ruling: "the player's existing waveform tests pass unchanged" means as plan 2 left them (plan 2 rewrites `a_stopped_players_waveform_does_not_seek` for Q8) — plan 2 changes that behaviour on purpose — cost if wrong: none; this plan does not edit `waveform_ui.rs` or `markers_ui.rs` at all.
- Ruling: the view state is keyed by `WaveKey` (Task 2) before the code moves (Task 3), the reverse of the order in spec §7 — the moved code then carries `WaveKey` from its first commit and is not edited twice; both steps change no behaviour, so the "extract first, then adopt in the CUE window" order the maintainer asked for holds — cost if wrong: none, the end state is the same.
- Ruling: `ViewState::wave_menu` is keyed by `WaveKey` too, although Q7.2 names only `wave_zoom` and `marker_drag` — a CUE window's menu and its player's menu on the same `PlayerId` would otherwise share one slot, and closing one would forget the other's point — cost if wrong: one more key type change in a private struct.
- Ruling: each key keeps today's egui id (`("waveform", player)` and `("cue-waveform", player)`, `WaveKey::id`) and today's accessible label ("Waveform: click to seek", "CUE waveform: click to seek") — the memoised columns, the pan-drag flag and every kittest that finds a waveform by its label are unchanged; no new string — cost if wrong: none.
- Ruling: the CUE's intro badge counts down to the intro end like the player's, rounded to tenths, and never blinks — the blink is the on-air talk-over warning (rule 18) and a CUE is not on air; the outro badge counts to the end of the file (Q7.5) — cost if wrong: one `intro_blink` expression in `cue_window.rs`.
- Ruling: the CUE window's MIX marker uses its player's mode (`mix_active`: solid amber in Continuous, dim in Single) — Q7.4 asks for "the same look as the player" — cost if wrong: one field of `CueWindowView`.
- Ruling: a zoomed CUE waveform follows the CUE position after the grace, paused or not (`follow: true`) — a paused player's zoom follows too (`status != Stopped`), and a paused position does not move — cost if wrong: one boolean in `cue_window.rs`. Superseded by the review: the zoom follows only while the CUE plays (`follow: !paused`), see the ledger.
- Ruling: when a player has no CUE, its CUE key's zoom, menu point and marker drag are forgotten (`cue_window::forget`) — otherwise a CUE reopened on the same entry restores an old zoom, and a drag cut off by Stop sends a `SetMarker` on the next CUE's first frame — cost if wrong: an operator who wanted the zoom kept across CUEs has to zoom again.
- Ruling: `WavePanelInput::editable_markers` is kept as Q7.1 lists it, and both callers pass `true` — the spec names it; it is the switch for a later caller that must not edit (a cart, a preview) — cost if wrong: one unused-in-practice field.
- Ruling: the panel returns the seek and the caller sends it after the panel's marker commands, where the player sent `Seek` before them — in one frame a seek needs a plain click without Alt, while `SetMarker` comes from a menu item (another widget) or an Alt-drag release, so they never share a frame — cost if wrong: the order of two commands in one frame.
- Ruling: `CueWindowView` keeps `position` next to the new `markers` (whose `position` is the same value) — existing view tests read it — cost if wrong: one redundant field.
- Ruling: Task 3 is a move, so its red step is the pure wheel rules it extracts into `wave_view.rs` (`wheel_notches`, `WaveView::wheel`); the rest of the move is guarded by the existing suites run before and after — a pure move has no new behaviour to fail first — cost if wrong: none.

## Global Constraints

- All code, identifiers, comments, docs, specs, plans and commit messages are in English. Never mention other playout or radio-automation products.
- Spec §9, verbatim: "`CLAUDE.md` rules 1–10 apply to every plan. In particular: English everywhere, and UI strings in both locales; no product names; operator values are `Config` fields with defaults, ranges and lenient loading (Q12's overrides and view); the real-time path never allocates, locks, logs or panics; behaviour lives in `fp-model` (`SetDuration`, `pending_start`, the DSD reasons); the UI never blocks (the header read and the file stat run on helper threads); bad data never crashes (a header without a duration, a file that changes while it is checked); nothing goes on air by itself (a pending start only changes where Play starts)."
- Q7.1: "A new `ui/wave_panel.rs` holds that logic behind an input struct: `key: WaveKey` (`Player(PlayerId)` or `Cue(PlayerId)`), the entry, the track, the media, the total, the markers, the position, `mix_active`, `seekable`, `follow`, the optional badges, `editable_markers` and the height. Its output is `{ seek: Option<f64> }`; the caller maps it to `Seek` or `SeekCue`."
- Q7.2: "`ViewState.wave_zoom` and `ViewState.marker_drag` are keyed by `WaveKey`, so the player's and its CUE's zoom and drags are independent."
- Q7.3: "The player and the CUE window are thin callers; the CUE window receives `&mut ViewState`."
- Q7.4: "The CUE window shows the intro, outro and MIX markers and edits them (right-click menu, Alt-drag) with the same look and commands as the player; it zooms, pans and has the Full view button."
- Q7.5: "A CUE plays the whole file, so its waveform shows cue-in and cue-out dimmed (`cue_edge_look`) without the head and tail shading, and its intro and outro badges count against the end of the file."
- Q7.6: "A marker edited in either place shows at once in both."
- Implementation note: "A pure `view::marker_fractions(..)` is extracted from `player_view` and unit-tested; both callers use it."
- Docs and locales: "Existing marker strings are reused. `docs/user/players.md` (CUE window), `docs/user/markers-and-mixing.md`, `docs/technical/ui.md`." No locale file changes in this plan; every task says so.
- `unwrap`, `expect` and `panic` are denied outside tests; test files start with the usual `#![allow(clippy::unwrap_used, ...)]` header, which the files touched here already have.
- The UI never blocks: the panel reads `MediaCache` (an in-memory map) and the snapshot only.
- Every commit is gated on `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace` and ends with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.

## Review Focus

Inputs the spec implies but no rule spells out, most likely to bite first; each has its test in the named task.

1. **A CUE stopped while zoomed, then started again on the same entry** — the operator expects the whole file, not the old zoom. Test: `a_new_cue_opens_on_the_whole_file` (Task 5).
2. **A CUE stopped in the middle of an Alt-drag** — no marker may move, now or when the next CUE opens. Test: `stopping_the_cue_mid_drag_moves_no_marker` (Task 5).
3. **A wheel over the CUE window while the player's waveform is under it, or the same track in both** — only the CUE's view zooms; the player's waveform keeps its own zoom. Tests: `the_wheel_zooms_the_cue_waveform_and_a_click_seeks_the_cue_in_the_zoomed_view` (exactly one Full view) and `the_player_and_its_cue_zoom_independently` (Task 5); `a_player_and_its_cue_keep_their_own_zoom` (Task 2).
4. **A CUE moved to another entry (a row click) while zoomed** — the new entry shows whole. Test: `a_cue_moved_to_another_entry_shows_it_whole` (Task 5).
5. **A CUE on a track of unknown length** — no zoom, no Full view, no seek, nothing to edit, no panic. Test: `a_cue_without_a_known_length_does_not_zoom` (Task 5), next to the existing `a_cue_without_a_known_length_shows_zero_and_cannot_seek`.

## File Structure

| File | Change |
|---|---|
| `crates/fp-app/src/ui/view.rs` | `marker_fractions`, `intro_left`, `outro_left`, `tenths` (Task 1); `CueWindowView::{markers, intro, outro, mix_active}` (Task 4) |
| `crates/fp-app/src/ui/wave_view.rs` | `WaveKey`, `WaveZooms` (Task 2); `ZOOM_STEP`, `PAN_STEP`, `POINTS_PER_NOTCH`, `NOTCHES_PER_PAGE`, `wheel_notches`, `WaveView::wheel` (Task 3) |
| `crates/fp-app/src/ui/app.rs` | `ViewState::{wave_menu, wave_zoom, marker_drag}` keyed by `WaveKey` (Task 2); `cue_window::show_all` gets `&mut self.view` (Task 5) |
| `crates/fp-app/src/ui/wave_panel.rs` (new) | `WaveBadges`, `WavePanelInput`, `WavePanelOutput`, `show`, `edit_markers` (Task 3) |
| `crates/fp-app/src/ui.rs` | `mod wave_panel;` (Task 3) |
| `crates/fp-app/src/ui/player.rs` | `wave` becomes a thin caller; `edit_markers` and the wheel constants leave (Tasks 2–3) |
| `crates/fp-app/src/ui/cue_window.rs` | `wave` becomes a thin caller; `show_all`/`show` take `&mut ViewState`; `forget` (Task 5) |
| `crates/fp-app/tests/view.rs` | Tests of Tasks 1 and 4 |
| `crates/fp-app/tests/waveform_view.rs` | Tests of Tasks 2 and 3 (`mod keys`, `mod wheel`) |
| `crates/fp-app/tests/cue_window.rs` | Tests of Task 5 |
| `README.md`, `docs/user/players.md`, `docs/user/markers-and-mixing.md`, `docs/technical/ui.md`, the spec | Task 6 |

---

### Task 1: `marker_fractions`, `intro_left` and `outro_left` out of `player_view` (no behaviour change)

**Executor model:** `opus` (refactor of the player's view model under its existing rule tests).

**Files:**
- Modify: `crates/fp-app/src/ui/view.rs:79-83` (`fraction`), `:208-249` (the tail of `player_view`)
- Test: `crates/fp-app/tests/view.rs`
- Docs: none in this task (Task 6). Locales: none (no new string; `en-US/main.ftl` and `es-ES/main.ftl` unchanged).

**Interfaces:**
- Consumes: `Track::{markers, intro_end_secs, outro_start_secs, segue_start_secs}` (`fp-model/src/track.rs:405-415`); the private `fraction` in `view.rs`.
- Produces (all `pub` in `fp_app::ui::view`):
  - `pub fn marker_fractions(track: &Track, total: f64, position: f64, ignored: bool) -> MarkerFractions`
  - `pub fn intro_left(track: &Track, position: f64) -> Option<f64>` — raw seconds to the intro end while before it.
  - `pub fn outro_left(track: &Track, position: f64, end: f64) -> Option<f64>` — seconds to `end` once at or past the outro start, never negative.
  - private `fn tenths(secs: f64) -> f64` — rounded to a tenth, as the badges show it.

- [x] **Step 1: Write the failing tests**

In `crates/fp-app/tests/view.rs`, extend the `use fp_app::ui::view::{…}` list with `MarkerFractions, intro_left, marker_fractions, outro_left` (keep it sorted the way `cargo fmt` leaves it), then append after the `use_markers` helper (`tests/view.rs:290-295`):

```rust
/// Every marker of the track of `entry` set by hand on a 200 s track:
/// cue-in 10, intro end 30, outro 150, MIX 170, cue-out 190.
fn all_marked(s: &mut AppState, entry: EntryId) -> fp_model::TrackId {
    let t = s.playlists.entry(entry).unwrap().track;
    mark(s, t, MarkerKind::CueIn, 10.0);
    mark(s, t, MarkerKind::CueOut, 190.0);
    mark(s, t, MarkerKind::IntroEnd, 30.0);
    mark(s, t, MarkerKind::OutroStart, 150.0);
    mark(s, t, MarkerKind::SegueStart, 170.0);
    t
}

fn near(v: Option<f32>, want: f32) -> bool {
    v.is_some_and(|v| (v - want).abs() < 1e-4)
}

#[test]
fn marker_fractions_place_the_markers_and_the_position_against_the_total() {
    let (mut s, e, _) = state(1);
    let t = all_marked(&mut s, e[0]);
    let track = s.library.get(t).unwrap();
    let m = marker_fractions(track, 200.0, 50.0, false);
    assert!(near(m.position, 0.25), "{m:?}");
    assert!(near(m.cue_in, 0.05) && near(m.cue_out, 0.95), "{m:?}");
    assert!(near(m.intro_end, 0.15), "{m:?}");
    assert!(near(m.outro_start, 0.75) && near(m.segue_start, 0.85), "{m:?}");
    assert!(!m.ignored);
    assert!(marker_fractions(track, 200.0, 50.0, true).ignored);
}

#[test]
fn marker_fractions_clamp_the_position_and_need_a_length() {
    let (mut s, e, _) = state(1);
    let t = all_marked(&mut s, e[0]);
    let track = s.library.get(t).unwrap();
    assert_eq!(
        marker_fractions(track, 200.0, 900.0, false).position,
        Some(1.0)
    );
    assert_eq!(
        marker_fractions(track, 0.0, 50.0, true),
        MarkerFractions {
            ignored: true,
            ..MarkerFractions::default()
        },
        "no length, nothing to place"
    );
}

#[test]
fn the_intro_and_outro_countdowns_start_at_their_markers() {
    let (mut s, e, _) = state(1);
    let t = all_marked(&mut s, e[0]);
    let track = s.library.get(t).unwrap();
    assert_eq!(intro_left(track, 20.0), Some(10.0));
    assert_eq!(intro_left(track, 30.0), None, "the intro is over");
    assert_eq!(outro_left(track, 100.0, 190.0), None, "not in the outro yet");
    assert_eq!(outro_left(track, 160.0, 190.0), Some(30.0));
    assert_eq!(outro_left(track, 195.0, 190.0), Some(0.0), "never negative");
}

#[test]
fn the_player_view_places_its_markers_with_marker_fractions() {
    let (mut s, e, p) = state(2);
    let t = all_marked(&mut s, e[0]);
    apply(&mut s, Command::Play(p)).unwrap();
    let v = player_view(&s, p, Some(50.0), 0.0).unwrap();
    let track = s.library.get(t).unwrap();
    assert_eq!(v.markers, marker_fractions(track, 200.0, 50.0, false));
}
```

- [x] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-app --test view marker_fractions`
Expected: does not compile — "unresolved imports `fp_app::ui::view::intro_left`, `marker_fractions`, `outro_left`".

- [x] **Step 3: Write the implementation**

In `crates/fp-app/src/ui/view.rs`, after `fn fraction` (`:79-83`), add:

```rust
/// Seconds rounded to a tenth, as the intro badge shows them.
fn tenths(secs: f64) -> f64 {
    (secs * 10.0).round() / 10.0
}

/// Where the markers of `track` and `position` fall on its waveform, as
/// fractions of `total` (operator feedback 4, Q7); all `None` without a
/// length. `ignored` draws cue-in and cue-out as dimmed marks without
/// shading: a player that does not use them, and every CUE, which plays the
/// whole file.
pub fn marker_fractions(
    track: &Track,
    total: f64,
    position: f64,
    ignored: bool,
) -> MarkerFractions {
    MarkerFractions {
        position: fraction(Some(position), total),
        cue_in: fraction(track.markers.cue_in.map(|m| m.secs), total),
        intro_end: fraction(track.intro_end_secs(), total),
        outro_start: fraction(track.outro_start_secs(), total),
        segue_start: fraction(track.segue_start_secs(), total),
        cue_out: fraction(track.markers.cue_out.map(|m| m.secs), total),
        ignored,
    }
}

/// Seconds left of a marked intro at `position` (spec §3 rule 18); `None`
/// without an intro or once it is over.
pub fn intro_left(track: &Track, position: f64) -> Option<f64> {
    track
        .intro_end_secs()
        .filter(|end| position < *end)
        .map(|end| end - position)
}

/// Seconds from `position` to `end` once the outro has started (rule 19);
/// `None` before it or without one.
pub fn outro_left(track: &Track, position: f64, end: f64) -> Option<f64> {
    track
        .outro_start_secs()
        .filter(|outro| position >= *outro)
        .map(|_| (end - position).max(0.0))
}
```

Then replace the tail of `player_view` (`view.rs:223-247`, from `if let Some(intro_end) = track.intro_end_secs()` to the end of the `if let Some(total) = total { view.markers = … }` block) with:

```rust
    if let Some(left) = intro_left(track, pos) {
        view.intro = Some(tenths(left));
        // The talk-over warning blinks only for a track on its way.
        view.intro_blink =
            (left <= 3.0 && p.current.is_some()).then(|| blink_phase.rem_euclid(1.0) < 0.5);
    }
    view.outro = outro_left(track, pos, cue_out);
    if let Some(total) = total {
        view.markers = marker_fractions(
            track,
            total,
            pos,
            !state.config.players.use_cue_markers,
        );
    }
```

(After plan 2, `pos` may come from the pending start; keep plan 2's `pos`.)

- [x] **Step 4: Run the tests to verify they pass, and the old view rules with them**

Run: `cargo test -p fp-app --test view`
Expected: PASS, including the untouched `rule18_the_intro_badge_counts_down_only_for_a_marked_intro_and_blinks_at_the_end`, `rule19_the_outro_badge_counts_down_to_cue_out`, `the_countdown_and_the_outro_follow_the_play_range` and `the_view_says_when_the_cue_marks_are_ignored`.

- [x] **Step 5: Commit**

```bash
if cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings \
   && cargo test --workspace; then
  git add crates/fp-app/src/ui/view.rs crates/fp-app/tests/view.rs
  git commit -m "refactor(ui): extract marker_fractions from the player view

The CUE window will place the same markers on its waveform (operator
feedback 4, Q7). The intro and outro countdowns move out with them.
No behaviour changes.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
fi
```

---

### Task 2: `WaveKey` and the keyed waveform view state (no behaviour change)

**Executor model:** `opus` (view-state invariants: a zoom belongs to its entry, a drag to its track).

**Files:**
- Modify: `crates/fp-app/src/ui/wave_view.rs:1-16` (imports, new types after `WaveZoom`)
- Modify: `crates/fp-app/src/ui/app.rs:117-118` (`wave_menu`), `:132-135` (`wave_zoom`, `marker_drag`)
- Modify: `crates/fp-app/src/ui/player.rs:20` (import), `:849-1058` (`wave`), `:1394-1551` (`edit_markers`)
- Test: `crates/fp-app/tests/waveform_view.rs` (new `mod keys` at the end)
- Docs: none in this task (Task 6). Locales: none (`en-US/main.ftl`, `es-ES/main.ftl` unchanged).

**Interfaces:**
- Consumes: `WaveZoom { view, entry, moved_at }`, `WaveView` (`wave_view.rs:8-23`).
- Produces (in `fp_app::ui::wave_view`):
  - `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] pub enum WaveKey { Player(PlayerId), Cue(PlayerId) }` with `pub fn id(self) -> egui::Id`.
  - `#[derive(Debug, Clone, Default)] pub struct WaveZooms` with `pub fn get(&self, key: WaveKey, entry: Option<EntryId>) -> Option<WaveZoom>` and `pub fn set(&mut self, key: WaveKey, zoom: Option<WaveZoom>)`.
  - `ViewState::wave_menu: HashMap<WaveKey, f64>`, `ViewState::wave_zoom: WaveZooms`, `ViewState::marker_drag: Option<(WaveKey, fp_model::MarkerKind, TrackId)>`.

- [x] **Step 1: Write the failing tests**

Append to `crates/fp-app/tests/waveform_view.rs`:

```rust
mod keys {
    use fp_app::ui::wave_view::{WaveKey, WaveView, WaveZoom, WaveZooms};
    use fp_model::{EntryId, PlayerId};

    fn zoom(entry: u64, start: f64) -> WaveZoom {
        WaveZoom {
            view: WaveView {
                start_secs: start,
                span_secs: 10.0,
            },
            entry: EntryId(entry),
            moved_at: 0.0,
        }
    }

    #[test]
    fn a_player_and_its_cue_keep_their_own_zoom() {
        let p = PlayerId(1);
        let (player, cue) = (WaveKey::Player(p), WaveKey::Cue(p));
        let mut z = WaveZooms::default();
        z.set(player, Some(zoom(5, 20.0)));
        assert_eq!(z.get(cue, Some(EntryId(5))), None, "the CUE is not zoomed");
        z.set(cue, Some(zoom(5, 40.0)));
        assert_eq!(z.get(player, Some(EntryId(5))), Some(zoom(5, 20.0)));
        assert_eq!(z.get(cue, Some(EntryId(5))), Some(zoom(5, 40.0)));
        z.set(cue, None);
        assert_eq!(z.get(cue, Some(EntryId(5))), None);
        assert_eq!(
            z.get(player, Some(EntryId(5))),
            Some(zoom(5, 20.0)),
            "the player's zoom stays"
        );
    }

    #[test]
    fn players_keep_their_own_zoom() {
        let mut z = WaveZooms::default();
        z.set(WaveKey::Player(PlayerId(1)), Some(zoom(5, 20.0)));
        assert_eq!(z.get(WaveKey::Player(PlayerId(2)), Some(EntryId(5))), None);
    }

    #[test]
    fn a_zoom_belongs_to_the_entry_it_was_made_on() {
        let key = WaveKey::Cue(PlayerId(1));
        let mut z = WaveZooms::default();
        z.set(key, Some(zoom(5, 20.0)));
        assert_eq!(z.get(key, Some(EntryId(6))), None, "another entry");
        assert_eq!(z.get(key, None), None, "no entry");
        assert_eq!(z.get(key, Some(EntryId(5))), Some(zoom(5, 20.0)));
    }

    #[test]
    fn each_key_keeps_its_waveform_id() {
        let (p, q) = (PlayerId(1), PlayerId(2));
        assert_eq!(WaveKey::Player(p).id(), egui::Id::new(("waveform", p)));
        assert_eq!(WaveKey::Cue(p).id(), egui::Id::new(("cue-waveform", p)));
        let ids = [
            WaveKey::Player(p).id(),
            WaveKey::Cue(p).id(),
            WaveKey::Player(q).id(),
            WaveKey::Cue(q).id(),
        ];
        for (i, a) in ids.iter().enumerate() {
            for b in ids.iter().skip(i + 1) {
                assert_ne!(a, b);
            }
        }
    }
}
```

- [x] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-app --test waveform_view keys`
Expected: does not compile — "unresolved imports `fp_app::ui::wave_view::WaveKey`, `WaveZooms`".

- [x] **Step 3: Add the types**

In `crates/fp-app/src/ui/wave_view.rs`, change the imports (`:5-6`) to:

```rust
use std::collections::HashMap;

use egui::{Rect, pos2};
use fp_model::{EntryId, PlayerId};
```

and add after `struct WaveZoom` (`:16`):

```rust
/// Which waveform a view state belongs to (operator feedback 4, Q7.2): a
/// player's, or its CUE window's. Their zoom, menu and marker drag are
/// independent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WaveKey {
    Player(PlayerId),
    Cue(PlayerId),
}

impl WaveKey {
    /// The waveform's egui id: its memoised columns and its pan drag are
    /// keyed on it.
    pub fn id(self) -> egui::Id {
        match self {
            Self::Player(p) => egui::Id::new(("waveform", p)),
            Self::Cue(p) => egui::Id::new(("cue-waveform", p)),
        }
    }
}

/// The zoomed waveforms; one that is not here shows the whole track.
#[derive(Debug, Clone, Default)]
pub struct WaveZooms(HashMap<WaveKey, WaveZoom>);

impl WaveZooms {
    /// The zoom of `key`, when it was made on `entry`: another entry, or
    /// none, shows the whole track.
    pub fn get(&self, key: WaveKey, entry: Option<EntryId>) -> Option<WaveZoom> {
        self.0
            .get(&key)
            .copied()
            .filter(|z| Some(z.entry) == entry)
    }

    /// Keeps `zoom` for `key`; `None` returns it to the whole track.
    pub fn set(&mut self, key: WaveKey, zoom: Option<WaveZoom>) {
        match zoom {
            Some(z) => {
                self.0.insert(key, z);
            }
            None => {
                self.0.remove(&key);
            }
        }
    }
}
```

- [x] **Step 4: Key the view state**

In `crates/fp-app/src/ui/app.rs`, replace the three fields:

```rust
    /// Where each waveform's menu was opened, in seconds.
    pub wave_menu: HashMap<super::wave_view::WaveKey, f64>,
```

```rust
    /// Zoomed waveforms; one without a zoom shows the whole track.
    pub wave_zoom: super::wave_view::WaveZooms,
    /// A marker being dragged on a waveform, and the track it belongs to.
    pub marker_drag: Option<(super::wave_view::WaveKey, fp_model::MarkerKind, TrackId)>,
```

In `crates/fp-app/src/ui/player.rs`:

- `:20` becomes `use super::wave_view::{WaveKey, WaveView, WaveZoom, min_span};`
- In `wave`, replace `:858-868` (from `// A zoom belongs to the entry it was made on.` to the `dragging` binding) with:

```rust
    let key = WaveKey::Player(id);
    // A zoom belongs to the entry it was made on.
    let mut zoom = view_state
        .wave_zoom
        .get(key, current)
        .filter(|_| total.is_some());
    let wave_id = key.id();
    // While zoomed, follow the playhead once the operator's last move is
    // older than the grace (feedback spec F17), but never under a held drag.
    let dragging = widgets::pan_dragging(ui, wave_id)
        || view_state.marker_drag.is_some_and(|(k, _, _)| k == key);
```

- `:923` becomes `edit_markers(ui, scene, view_state, key, track, shown, pv, &response);`
- Replace the `match zoom { … }` at `:1017-1024` with `view_state.wave_zoom.set(key, zoom);`
- In `edit_markers` (`:1397-1551`): the parameter `id: PlayerId` becomes `key: WaveKey`, and:
  - `:1423` `view_state.wave_menu.insert(key, secs_at(p.x));`
  - `:1425` `let at = view_state.wave_menu.get(&key).copied();`
  - `:1471` `view_state.wave_menu.remove(&key);`
  - `:1486` `view_state.marker_drag = nearest.map(|(kind, _)| (key, kind, track));`
  - `:1490-1495`:

```rust
    if view_state
        .marker_drag
        .is_some_and(|(k, _, t)| k == key && t != track)
    {
        view_state.marker_drag = None;
    }
```

  - `:1496-1499`:

```rust
    let dragging = view_state
        .marker_drag
        .filter(|(k, _, _)| *k == key)
        .map(|(_, k, _)| k);
```

- [x] **Step 5: Run the new and the existing waveform tests**

Run: `cargo test -p fp-app --test waveform_view && cargo test -p fp-app --test waveform_ui && cargo test -p fp-app --test markers_ui`
Expected: PASS, every existing test unchanged.

- [x] **Step 6: Commit**

```bash
if cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings \
   && cargo test --workspace; then
  git add crates/fp-app/src/ui/wave_view.rs crates/fp-app/src/ui/app.rs \
    crates/fp-app/src/ui/player.rs crates/fp-app/tests/waveform_view.rs
  git commit -m "refactor(ui): key the waveform view state by WaveKey

A player's waveform and its CUE window's will keep their own zoom, menu
point and marker drag (operator feedback 4, Q7.2). The player uses
WaveKey::Player and the same egui id as before. No behaviour changes.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
fi
```

---

### Task 3: `ui/wave_panel.rs`, with the player as its only caller (no behaviour change)

**Executor model:** `opus` (moving the zoom, follow, pan and marker-drag logic without changing a frame's outcome).

**Files:**
- Create: `crates/fp-app/src/ui/wave_panel.rs`
- Modify: `crates/fp-app/src/ui.rs` (add `mod wave_panel;` after `pub mod view;`)
- Modify: `crates/fp-app/src/ui/wave_view.rs` (wheel constants, `wheel_notches`, `WaveView::wheel`)
- Modify: `crates/fp-app/src/ui/player.rs:20-32` (imports and wheel constants), `:849-1058` (`wave`), `:1394-1551` (`edit_markers`, deleted)
- Test: `crates/fp-app/tests/waveform_view.rs` (new `mod wheel`); guard: `tests/waveform_ui.rs`, `tests/markers_ui.rs` unchanged
- Docs: none in this task (Task 6). Locales: none (`en-US/main.ftl`, `es-ES/main.ftl` unchanged; the panel reuses `wave-full-view`, `wave-intro`, `wave-outro`, `mix-marker`, `tip-waveform`, `tip-cue-waveform`, `wave-set-*`, `wave-reset`).

**Interfaces:**
- Consumes: `WaveKey`, `WaveZooms` (Task 2); `widgets::{waveform, WaveInput, WaveOutput, pan_dragging, tile, time_badge, tabular_size, paint_tabular, TileStyle, font}`; `view::MarkerFractions`; `Scene`, `ViewState`.
- Produces:
  - In `fp_app::ui::wave_view`: `pub const ZOOM_STEP: f64 = 0.8`, `pub const PAN_STEP: f32 = 0.1`, `pub const POINTS_PER_NOTCH: f32 = 50.0`, `pub const NOTCHES_PER_PAGE: f32 = 3.0`, `pub fn wheel_notches(unit: egui::MouseWheelUnit, delta: egui::Vec2) -> egui::Vec2`, `impl WaveView { pub fn wheel(&self, notches: egui::Vec2, shift: bool, x: f32, rect: Rect, total: f64, min_span: f64) -> Self }`.
  - In `crate::ui::wave_panel` (crate-private):
    - `pub(crate) struct WaveBadges { pub intro: Option<f64>, pub intro_blink: Option<bool>, pub outro: Option<f64> }` (`Default`)
    - `pub(crate) struct WavePanelInput<'a> { pub key: WaveKey, pub entry: Option<EntryId>, pub track: Option<TrackId>, pub media: Option<&'a Arc<TrackMedia>>, pub total: Option<f64>, pub markers: MarkerFractions, pub mix_active: bool, pub seekable: bool, pub follow: bool, pub badges: WaveBadges, pub editable_markers: bool, pub height: f32 }` (the position is `markers.position`)
    - `pub(crate) struct WavePanelOutput { pub seek: Option<f64> }`
    - `pub(crate) fn show(ui: &mut Ui, scene: &Scene<'_>, view_state: &mut ViewState, input: &WavePanelInput<'_>) -> WavePanelOutput`

- [x] **Step 1: Write the failing tests for the wheel rules**

Append to `crates/fp-app/tests/waveform_view.rs`:

```rust
mod wheel {
    use egui::{MouseWheelUnit, Rect, pos2, vec2};
    use fp_app::ui::wave_view::{
        NOTCHES_PER_PAGE, PAN_STEP, POINTS_PER_NOTCH, WaveView, ZOOM_STEP, wheel_notches,
    };

    fn rect() -> Rect {
        Rect::from_min_size(pos2(100.0, 10.0), vec2(400.0, 60.0))
    }

    #[test]
    fn lines_are_notches_and_points_and_pages_are_converted() {
        assert_eq!(
            wheel_notches(MouseWheelUnit::Line, vec2(0.0, 2.0)),
            vec2(0.0, 2.0)
        );
        assert_eq!(
            wheel_notches(MouseWheelUnit::Point, vec2(0.0, POINTS_PER_NOTCH)),
            vec2(0.0, 1.0)
        );
        assert_eq!(
            wheel_notches(MouseWheelUnit::Page, vec2(0.0, 1.0)),
            vec2(0.0, NOTCHES_PER_PAGE)
        );
    }

    #[test]
    fn an_upward_notch_zooms_in_around_the_pointer() {
        let full = WaveView::full(180.0);
        let x = rect().center().x;
        let v = full.wheel(vec2(0.0, 1.0), false, x, rect(), 180.0, 0.0);
        assert_eq!(v, full.zoom_at(x, rect(), ZOOM_STEP, 180.0, 0.0));
        assert!((v.span_secs - 144.0).abs() < 1e-9, "{v:?}");
    }

    #[test]
    fn shift_or_a_sideways_wheel_pans() {
        let zoomed = WaveView::full(180.0).zoom_at(300.0, rect(), 0.5, 180.0, 0.0);
        let by_shift = zoomed.wheel(vec2(0.0, -1.0), true, 300.0, rect(), 180.0, 0.0);
        assert_eq!(
            by_shift,
            zoomed.pan(-1.0 * rect().width() * PAN_STEP, rect(), 180.0)
        );
        let sideways = zoomed.wheel(vec2(1.0, 0.0), false, 300.0, rect(), 180.0, 0.0);
        assert_eq!(
            sideways,
            zoomed.pan(rect().width() * PAN_STEP, rect(), 180.0)
        );
        assert_ne!(sideways, zoomed, "the view moved");
    }
}
```

- [x] **Step 2: Run them to verify they fail**

Run: `cargo test -p fp-app --test waveform_view wheel`
Expected: does not compile — "unresolved imports `fp_app::ui::wave_view::NOTCHES_PER_PAGE`, `PAN_STEP`, `POINTS_PER_NOTCH`, `ZOOM_STEP`, `wheel_notches`".

- [x] **Step 3: Move the wheel rules into `wave_view.rs`**

In `crates/fp-app/src/ui/wave_view.rs`, change `use egui::{Rect, pos2};` to `use egui::{MouseWheelUnit, Rect, Vec2, pos2};` and add after `fn min_span`:

```rust
/// One wheel notch zooms the waveform in to this share of its span.
pub const ZOOM_STEP: f64 = 0.8;
/// One sideways notch pans the waveform by this share of its width.
pub const PAN_STEP: f32 = 0.1;
/// Smooth-scrolling wheels and trackpads report points: this many make a
/// notch.
pub const POINTS_PER_NOTCH: f32 = 50.0;
/// A wheel that reports pages: one page is this many notches.
pub const NOTCHES_PER_PAGE: f32 = 3.0;

/// A wheel event in notches: a line is one, points and pages are
/// converted, so a trackpad zooms in proportion instead of one step per
/// event.
pub fn wheel_notches(unit: MouseWheelUnit, delta: Vec2) -> Vec2 {
    match unit {
        MouseWheelUnit::Line => delta,
        MouseWheelUnit::Point => delta / POINTS_PER_NOTCH,
        MouseWheelUnit::Page => delta * NOTCHES_PER_PAGE,
    }
}
```

and inside `impl WaveView`, after `pan`:

```rust
    /// One wheel event of `notches`: Shift or a mostly sideways wheel pans,
    /// otherwise it zooms around `x`, no closer than `min_span`.
    pub fn wheel(
        &self,
        notches: Vec2,
        shift: bool,
        x: f32,
        rect: Rect,
        total: f64,
        min_span: f64,
    ) -> Self {
        let sideways = notches.x.abs() > notches.y.abs();
        if shift || sideways {
            let step = if sideways { notches.x } else { notches.y };
            self.pan(step * rect.width() * PAN_STEP, rect, total)
        } else {
            self.zoom_at(
                x,
                rect,
                ZOOM_STEP.powf(f64::from(notches.y)),
                total,
                min_span,
            )
        }
    }
```

Run: `cargo test -p fp-app --test waveform_view wheel` — expected: PASS.

- [x] **Step 4: Create `crates/fp-app/src/ui/wave_panel.rs`**

The body of `show` is `player.rs::wave` (`:849-1058`) with `PlayerId`/`PlayerView` replaced by the input, and `edit_markers` is `player.rs::edit_markers` (`:1394-1551`) with `pv` replaced by `total` and `markers`. Whatever plan 2 changed inside those two functions moves along unchanged.

```rust
//! The waveform panel (operator feedback 4, Q7): the waveform with its
//! zoom, pan, Full view button, intro and outro badges and marker editing,
//! shared by the player column and the CUE window. It reads and writes the
//! view state of its `WaveKey` and sends only marker commands, which act on
//! a track; the caller turns the seek it returns into `Seek` or `SeekCue`.

use std::sync::Arc;

use egui::{Rect, RichText, Stroke, Ui, UiBuilder, pos2, vec2};
use fp_model::{Command, EntryId, MarkerKind, TrackId};

use super::app::{Scene, ViewState};
use super::format;
use super::theme;
use super::view::MarkerFractions;
use super::wave_view::{WaveKey, WaveView, WaveZoom, min_span, wheel_notches};
use super::widgets::{self, TileStyle, font};
use crate::services::TrackMedia;

/// The intro and outro countdowns drawn over the waveform (spec §3 rules
/// 18 and 19).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct WaveBadges {
    /// Seconds left of the intro.
    pub intro: Option<f64>,
    /// `Some(visible)` while the intro badge blinks.
    pub intro_blink: Option<bool>,
    /// Seconds from the outro to the end.
    pub outro: Option<f64>,
}

/// What the panel shows and allows (Q7.1).
pub(crate) struct WavePanelInput<'a> {
    pub key: WaveKey,
    /// The entry shown: a zoom belongs to it.
    pub entry: Option<EntryId>,
    /// Its track: edited markers belong to it.
    pub track: Option<TrackId>,
    pub media: Option<&'a Arc<TrackMedia>>,
    pub total: Option<f64>,
    /// The markers and the position, as fractions of `total`.
    pub markers: MarkerFractions,
    /// The MIX marker is drawn solid amber (Continuous mode).
    pub mix_active: bool,
    /// A click seeks.
    pub seekable: bool,
    /// While zoomed, the view follows the position after the grace.
    pub follow: bool,
    pub badges: WaveBadges,
    /// The right-click menu and Alt-drag edit the markers.
    pub editable_markers: bool,
    pub height: f32,
}

/// What a frame of the panel reports.
pub(crate) struct WavePanelOutput {
    /// A click's time; the caller sends `Seek` or `SeekCue`.
    pub seek: Option<f64>,
}

fn label_key(key: WaveKey) -> &'static str {
    match key {
        WaveKey::Player(_) => "tip-waveform",
        WaveKey::Cue(_) => "tip-cue-waveform",
    }
}

/// Draws the panel and handles its zoom, pan and marker editing.
pub(crate) fn show(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    input: &WavePanelInput<'_>,
) -> WavePanelOutput {
    let t = scene.i18n;
    let key = input.key;
    let total = input.total.filter(|t| *t > 0.0);
    // A zoom belongs to the entry it was made on.
    let mut zoom = view_state
        .wave_zoom
        .get(key, input.entry)
        .filter(|_| total.is_some());
    let wave_id = key.id();
    // While zoomed, follow the position once the operator's last move is
    // older than the grace (feedback spec F17), but never under a held drag.
    let dragging = widgets::pan_dragging(ui, wave_id)
        || view_state.marker_drag.is_some_and(|(k, _, _)| k == key);
    if let (Some(z), Some(total), Some(f)) = (zoom.as_mut(), total, input.markers.position) {
        let grace = scene.state.config.ui.follow_current_grace_secs;
        if dragging {
            z.moved_at = scene.time;
        } else if grace > 0.0 && scene.time - z.moved_at >= grace && input.follow {
            z.view = z.view.follow(f64::from(f) * total, total);
        }
    }
    let view = zoom.map(|z| z.view);
    // Where the Full view button goes while zoomed: no seek starts under it.
    let wave_rect = Rect::from_min_size(
        ui.cursor().min,
        vec2(ui.available_width(), input.height),
    );
    let full_view_text = t.tr("wave-full-view");
    let full_view_width = ui
        .painter()
        .layout_no_wrap(full_view_text.clone(), font(10.0), theme::TEXT)
        .size()
        .x
        + 12.0;
    let button_rect = Rect::from_min_size(
        pos2(
            wave_rect.right() - 4.0 - full_view_width,
            wave_rect.top() + 4.0,
        ),
        vec2(full_view_width, 18.0),
    );
    let full_view_button = zoom.map(|_| button_rect);
    let mix_label = t.tr("mix-marker");
    let label = t.tr(label_key(key));
    let wave_input = widgets::WaveInput {
        id: wave_id,
        media: input.media,
        total: input.total,
        markers: input.markers,
        colors: theme::wave_colors(&scene.state.config.ui.wave_color),
        mix_active: input.mix_active,
        mix_label: &mix_label,
        accessible_label: &label,
        view,
        shield: full_view_button,
        seekable: input.seekable,
    };
    let output = widgets::waveform(ui, input.height, &wave_input);
    let (response, seek, pan_dx) = (output.response, output.seek, output.pan_dx);
    let rect = response.rect;
    if input.editable_markers
        && let (Some(track), Some(total)) = (input.track, total)
    {
        let shown = view.unwrap_or_else(|| WaveView::full(total));
        edit_markers(
            ui,
            scene,
            view_state,
            key,
            track,
            shown,
            total,
            input.markers,
            &response,
        );
    }
    // The wheel zooms around the pointer; Shift or a sideways wheel pans.
    if let (Some(entry), Some(total), Some(p)) = (input.entry, total, response.hover_pos()) {
        let inner = rect.shrink(1.0);
        let bucket = input.media.map_or(
            f64::from(scene.state.config.analysis.peak_bucket_ms) / 1000.0,
            |m| m.peak_bucket_secs,
        );
        let min = min_span(bucket, inner.width());
        let wheels: Vec<(egui::Vec2, bool)> = ui.input(|i| {
            i.events
                .iter()
                .filter_map(|e| match e {
                    egui::Event::MouseWheel {
                        unit,
                        delta,
                        modifiers,
                        ..
                    } if *delta != egui::Vec2::ZERO && !modifiers.command => {
                        Some((wheel_notches(*unit, *delta), modifiers.shift))
                    }
                    _ => None,
                })
                .collect()
        });
        if !wheels.is_empty() {
            let mut v = view.unwrap_or_else(|| WaveView::full(total));
            for (notches, shift) in wheels {
                v = v.wheel(notches, shift, p.x, inner, total, min);
            }
            zoom = (!v.is_full(total)).then_some(WaveZoom {
                view: v,
                entry,
                moved_at: scene.time,
            });
            // The wheel was for the waveform, not for a scroll area around it.
            ui.ctx()
                .input_mut(|i| i.smooth_scroll_delta = egui::Vec2::ZERO);
            ui.ctx().request_repaint();
        }
    }
    // Dragging pans a zoomed view; without zoom a drag does nothing (O10).
    if pan_dx != 0.0
        && let (Some(z), Some(total)) = (zoom.as_mut(), total)
    {
        z.view = z.view.pan(pan_dx, rect.shrink(1.0), total);
        z.moved_at = scene.time;
    }
    let painter = ui.painter_at(rect);
    let mut badge_right = rect.right() - 4.0;
    if zoom.is_some() {
        let button = button_rect;
        let text = full_view_text;
        let mut child = ui.new_child(UiBuilder::new().max_rect(button));
        if widgets::tile(
            &mut child,
            button.size(),
            &text,
            true,
            TileStyle::plain(),
            |p, r, c| {
                p.text(
                    r.center(),
                    egui::Align2::CENTER_CENTER,
                    &text,
                    font(10.0),
                    c,
                );
            },
        )
        .clicked()
        {
            zoom = None;
        }
        badge_right = button.left() - 4.0;
    }
    view_state.wave_zoom.set(key, zoom);
    if let Some(left) = input.badges.intro {
        let fill = if input.badges.intro_blink == Some(true) {
            theme::INTRO_BADGE_BLINK
        } else {
            theme::INTRO_BADGE_BG
        };
        widgets::time_badge(
            &painter,
            rect.left_top() + vec2(4.0, 4.0),
            false,
            &t.tr("wave-intro"),
            &format!("{left:.1}"),
            (
                fill.gamma_multiply(0.95),
                theme::INTRO,
                theme::INTRO_BADGE_TEXT,
            ),
        );
    }
    if let Some(left) = input.badges.outro {
        widgets::time_badge(
            &painter,
            pos2(badge_right, rect.top() + 4.0),
            true,
            &t.tr("wave-outro"),
            &format!("{left:.1}"),
            (
                theme::OUTRO_BADGE_BG.gamma_multiply(0.9),
                theme::OUTRO_LINE,
                theme::OUTRO_BADGE_TEXT,
            ),
        );
    }
    WavePanelOutput { seek }
}
```

Then append `edit_markers`: copy `player.rs:1394-1551` verbatim as Task 2 left it (parameter `key: WaveKey`), with these changes only:

- the signature becomes

```rust
/// Marker editing on the waveform (Phase 2 spec P2.8): a context menu that
/// places a marker where it was opened, and Alt-drag on marker handles.
#[allow(clippy::too_many_arguments)]
fn edit_markers(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    key: WaveKey,
    track: TrackId,
    view: WaveView,
    total: f64,
    markers: MarkerFractions,
    response: &egui::Response,
) {
```

- delete the line `let total = pv.total.unwrap_or(0.0);` (`:1409`; `total` is now the parameter, always positive here);
- `let m = pv.markers;` (`:1412`) becomes `let m = markers;`.

In `crates/fp-app/src/ui.rs`, add `mod wave_panel;` between `pub mod view;` and `pub mod wave_view;`.

- [x] **Step 5: Make the player a thin caller**

In `crates/fp-app/src/ui/player.rs`:

- delete the four wheel constants and their comments (`:24-32`, `WAVE_ZOOM_STEP` to `WAVE_NOTCHES_PER_PAGE`); keep `WAVE_HEIGHT`;
- `:20` becomes:

```rust
use super::wave_panel::{self, WaveBadges, WavePanelInput};
use super::wave_view::WaveKey;
```

- replace the whole of `fn wave` (`:849-1058`) with:

```rust
fn wave(ui: &mut Ui, scene: &Scene<'_>, view_state: &mut ViewState, id: PlayerId, pv: &PlayerView) {
    // The current track, or the next one waiting while stopped.
    let entry = view::shown_entry(scene.state, id);
    let track = entry
        .and_then(|e| scene.state.playlists.entry(e))
        .map(|e| e.track);
    let media = track.and_then(|t| scene.media.get(t));
    let input = WavePanelInput {
        key: WaveKey::Player(id),
        entry,
        track,
        media: media.as_ref(),
        total: pv.total,
        markers: pv.markers,
        mix_active: pv.mode == PlayMode::Continuous,
        seekable: pv.status != PlayerStatus::Stopped,
        // A stopped player's position is pinned at the cue-in: following
        // it would undo a zoom made to prepare the next track.
        follow: pv.status != PlayerStatus::Stopped,
        badges: WaveBadges {
            intro: pv.intro,
            intro_blink: pv.intro_blink,
            outro: pv.outro,
        },
        editable_markers: true,
        height: WAVE_HEIGHT,
    };
    if let Some(secs) = wave_panel::show(ui, scene, view_state, &input).seek {
        scene.ctl.send(Command::Seek(id, secs));
    }
}
```

  (After plan 2, `seekable:` and the follow condition are whatever plan 2 left in the old `wave`.)
- delete `fn edit_markers` and its doc comment (`:1394-1551`);
- remove the imports clippy now reports unused (expected: `MarkerKind` from `fp_model`; check `RichText`, `UiBuilder` and `Rect`, which other functions of `player.rs` still use).

- [x] **Step 6: Run the whole fp-app suite: the guard of the move**

Run: `cargo test -p fp-app --test waveform_ui && cargo test -p fp-app --test markers_ui && cargo test -p fp-app --test waveform_view && cargo test -p fp-app --test cue_window && cargo test -p fp-app --test glyphs`
Expected: PASS with no test edited. If any player waveform test fails, the move changed behaviour: compare the moved code with `git show HEAD:crates/fp-app/src/ui/player.rs` line by line before touching a test.

- [x] **Step 7: Commit**

```bash
if cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings \
   && cargo test --workspace; then
  git add crates/fp-app/src/ui/wave_panel.rs crates/fp-app/src/ui.rs \
    crates/fp-app/src/ui/wave_view.rs crates/fp-app/src/ui/player.rs \
    crates/fp-app/tests/waveform_view.rs
  git commit -m "refactor(ui): move the player's waveform into a shared panel

ui/wave_panel.rs holds the zoom, pan, Full view, badges and marker
editing behind WavePanelInput; the player is its only caller and maps
the returned seek to Seek. The wheel rules move to wave_view.rs as pure
functions. The CUE window adopts the panel next (operator feedback 4,
Q7.1, Q7.3). No behaviour changes.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
fi
```

---

### Task 4: What the CUE window's panel shows (`CueWindowView`)

**Executor model:** `sonnet` (pure view fields with exact tests).

**Files:**
- Modify: `crates/fp-app/src/ui/view.rs:100-152` (`CueWindowView`, `cue_window_view`)
- Test: `crates/fp-app/tests/view.rs`
- Docs: none in this task (Task 6). Locales: none (`en-US/main.ftl`, `es-ES/main.ftl` unchanged).

**Interfaces:**
- Consumes: `marker_fractions`, `intro_left`, `outro_left`, `tenths` (Task 1); the test helpers `all_marked`, `near`, `mark`, `use_markers` in `tests/view.rs` (Task 1 and existing).
- Produces: `CueWindowView::{markers: MarkerFractions, intro: Option<f64>, outro: Option<f64>, mix_active: bool}`; `position` stays.

- [x] **Step 1: Write the failing tests**

In `crates/fp-app/tests/view.rs`, add `PlayMode` to the `use fp_model::{…}` list, then append after `load_as_next_is_offered_only_when_it_changes_something`:

```rust
#[test]
fn the_cue_window_places_the_markers_against_the_whole_file_without_shading() {
    let (mut s, e, p) = state(2);
    let t = all_marked(&mut s, e[0]);
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    let v = cue_window_view(&s, p, Some(50.0)).unwrap();
    let track = s.library.get(t).unwrap();
    assert_eq!(v.markers, marker_fractions(track, 200.0, 50.0, true));
    assert!(v.markers.ignored, "a CUE plays the whole file (Q7.5)");
    assert_eq!(v.markers.position, v.position);
    use_markers(&mut s, false);
    assert!(cue_window_view(&s, p, Some(50.0)).unwrap().markers.ignored);
}

#[test]
fn the_cue_window_badges_count_to_the_intro_and_to_the_end_of_the_file() {
    let (mut s, e, p) = state(2);
    all_marked(&mut s, e[0]);
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    let v = cue_window_view(&s, p, Some(18.76)).unwrap();
    assert_eq!((v.intro, v.outro), (Some(11.2), None));
    let v = cue_window_view(&s, p, Some(160.0)).unwrap();
    assert_eq!(
        (v.intro, v.outro),
        (None, Some(40.0)),
        "to the end of the file, not to the cue-out at 190 s"
    );
}

#[test]
fn a_cue_window_of_unknown_length_has_no_markers_or_outro() {
    let (mut s, e, p) = state(2);
    let t = s.playlists.entry(e[0]).unwrap().track;
    mark(&mut s, t, MarkerKind::OutroStart, 5.0);
    s.library.get_mut(t).unwrap().duration_secs = 0.0;
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    let v = cue_window_view(&s, p, Some(10.0)).unwrap();
    assert_eq!(v.markers.outro_start, None);
    assert_eq!(v.outro, None);
}

#[test]
fn the_cue_windows_mix_marker_follows_the_players_mode() {
    let (mut s, _, p) = state(2);
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    assert!(
        cue_window_view(&s, p, None).unwrap().mix_active,
        "Continuous is the default mode"
    );
    apply(&mut s, Command::SetMode(p, PlayMode::Single)).unwrap();
    assert!(!cue_window_view(&s, p, None).unwrap().mix_active);
}

/// Q7.6: both waveforms draw from the same track, so a marker set once
/// shows in both on the next frame.
#[test]
fn a_marker_set_once_shows_in_the_player_and_in_its_cue() {
    let (mut s, e, p) = state(2);
    // A stopped player shows its next entry, the one its CUE plays.
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    let t = s.playlists.entry(e[0]).unwrap().track;
    mark(&mut s, t, MarkerKind::IntroEnd, 40.0);
    let player = player_view(&s, p, None, 0.0).unwrap().markers.intro_end;
    let cue = cue_window_view(&s, p, None).unwrap().markers.intro_end;
    assert!(near(player, 0.2) && near(cue, 0.2), "{player:?} {cue:?}");
}
```

(`a_cue_window_of_unknown_length_has_no_markers_or_outro` sets the outro while the length is known, since `SetMarker` places intro, outro and MIX inside the play range, then forgets the length.)

- [x] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-app --test view cue_window`
Expected: does not compile — "no field `markers` on type `CueWindowView`" (and `intro`, `outro`, `mix_active`).

- [x] **Step 3: Write the implementation**

In `crates/fp-app/src/ui/view.rs`, add to `struct CueWindowView` after `position`:

```rust
    /// The markers and the position on the whole file. Cue-in and cue-out
    /// are dimmed marks without shading: a CUE plays the whole file
    /// (operator feedback 4, Q7.5).
    pub markers: MarkerFractions,
    /// Seconds left of a marked intro (spec §3 rule 18); it never blinks,
    /// since a CUE is not on air.
    pub intro: Option<f64>,
    /// Seconds from the outro to the end of the file (Q7.5).
    pub outro: Option<f64>,
    /// The MIX marker looks as on the player: solid in Continuous mode.
    pub mix_active: bool,
```

and in `cue_window_view`, add to the `CueWindowView { … }` literal after `position: …,`:

```rust
        markers: marker_fractions(track, total.unwrap_or(0.0), elapsed, true),
        intro: intro_left(track, elapsed).map(tenths),
        outro: total.and_then(|end| outro_left(track, elapsed, end)),
        mix_active: p.mode == PlayMode::Continuous,
```

Update the doc comment of `CueWindowView` (`:100-101`) to: "What a CUE window shows (feedback 2 spec O12; operator feedback 4, Q7). A CUE plays the whole file, so its times, markers and badges run to the end of the file, not to the cue-out."

- [x] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-app --test view`
Expected: PASS (the existing `cue_window_view` tests unchanged).

- [x] **Step 5: Commit**

```bash
if cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings \
   && cargo test --workspace; then
  git add crates/fp-app/src/ui/view.rs crates/fp-app/tests/view.rs
  git commit -m "feat(ui): give the CUE window view its markers and badges

A CUE plays the whole file, so its markers are placed on the whole file
with cue-in and cue-out dimmed, and its outro counts to the end of the
file (operator feedback 4, Q7.5). The MIX marker follows the player's
mode, as on the player.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
fi
```

---

### Task 5: The CUE window uses the panel

**Executor model:** `opus` (view state across window open and close, and the kittests that pin it).

**Files:**
- Modify: `crates/fp-app/src/ui/cue_window.rs:1-20` (doc, imports), `:22-38` (`show_all`), `:40-82` (`show`), `:117-143` (`wave`)
- Modify: `crates/fp-app/src/ui/app.rs:679` (`cue_window::show_all(&ctx, &scene)`)
- Test: `crates/fp-app/tests/cue_window.rs`
- Docs: none in this task (Task 6). Locales: none (`en-US/main.ftl`, `es-ES/main.ftl` unchanged; the window's waveform keeps `tip-cue-waveform`).

**Interfaces:**
- Consumes: `wave_panel::{show, WavePanelInput, WaveBadges}` (Task 3); `WaveKey::Cue`, `WaveZooms::set` (Task 2); `CueWindowView::{markers, intro, outro, mix_active}` (Task 4).
- Produces: `pub(crate) fn show_all(ctx: &egui::Context, scene: &Scene<'_>, view_state: &mut ViewState)`; private `fn forget(view_state: &mut ViewState, key: WaveKey)`.

Plan 1 (Q5) changes `buttons` in the same file and plan 2 (Q6) may add tests to `tests/cue_window.rs`; keep their changes when rebasing.

- [x] **Step 1: Write the failing tests**

In `crates/fp-app/tests/cue_window.rs`, change the imports to:

```rust
use egui::{Event, Modifiers, PointerButton, Pos2, Rect, pos2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use fp_app::ui::app::AppUi;
use fp_app::ui::controller::Controller;
use fp_model::{Command, MarkerKind, TrackId};
use support::{Fake, harness, state};
```

add after `const WAVE`:

```rust
const PLAYER_WAVE: &str = "Waveform: click to seek";
const FULL_VIEW: &str = "Full view";
```

and append:

```rust
/// P1 cues its next entry, a 180 s track whose intro ends at 30 s.
fn cueing_with_intro() -> (Harness<'static, AppUi>, Arc<Fake>, TrackId) {
    let mut s = state(1, 3);
    for t in s.library.iter_mut() {
        t.duration_secs = 180.0;
    }
    let track = s.playlists.iter().next().unwrap().entries[0].track;
    fp_model::apply(
        &mut s,
        Command::SetMarker {
            track,
            kind: MarkerKind::IntroEnd,
            secs: Some(30.0),
        },
    )
    .unwrap();
    let (mut h, fake) = harness(s);
    fake.send(Command::ToggleCue(fake.player(0)));
    fake.take_sent();
    h.run_steps(2);
    (h, fake, track)
}

/// The x of `secs` on a whole-file waveform of 180 s.
fn x_of(wave: Rect, secs: f32) -> f32 {
    wave.left() + 1.0 + (wave.width() - 2.0) * secs / 180.0
}

fn wheel(h: &mut Harness<'_, AppUi>, at: Pos2, dy: f32) {
    h.event(Event::PointerMoved(at));
    h.event(Event::MouseWheel {
        unit: egui::MouseWheelUnit::Line,
        delta: egui::vec2(0.0, dy),
        phase: egui::TouchPhase::Move,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(1);
}

fn button(h: &mut Harness<'_, AppUi>, at: Pos2, button: PointerButton, pressed: bool, modifiers: Modifiers) {
    h.event(Event::PointerButton {
        pos: at,
        button,
        pressed,
        modifiers,
    });
}

fn click(h: &mut Harness<'_, AppUi>, at: Pos2) {
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    button(h, at, PointerButton::Primary, true, Modifiers::NONE);
    button(h, at, PointerButton::Primary, false, Modifiers::NONE);
    h.run_steps(2);
}

/// Holds Alt, presses at `from` and moves to `to` in steps; the button
/// stays down.
fn alt_drag_to(h: &mut Harness<'_, AppUi>, from: Pos2, to: Pos2) {
    h.event(Event::ModifiersChanged(Modifiers::ALT));
    h.event(Event::PointerMoved(from));
    h.run_steps(1);
    button(h, from, PointerButton::Primary, true, Modifiers::ALT);
    h.run_steps(1);
    for step in 1..=10 {
        let x = from.x + (to.x - from.x) * step as f32 / 10.0;
        h.event(Event::PointerMoved(pos2(x, from.y)));
        h.run_steps(1);
    }
}

fn set_markers(sent: &[Command]) -> Vec<(TrackId, MarkerKind, Option<f64>)> {
    sent.iter()
        .filter_map(|c| match c {
            Command::SetMarker { track, kind, secs } => Some((*track, *kind, *secs)),
            _ => None,
        })
        .collect()
}

fn cue_seeks(sent: &[Command], p: fp_model::PlayerId) -> Vec<f64> {
    sent.iter()
        .filter_map(|c| match c {
            Command::SeekCue(id, secs) if *id == p => Some(*secs),
            _ => None,
        })
        .collect()
}

#[test]
fn the_wheel_zooms_the_cue_waveform_and_a_click_seeks_the_cue_in_the_zoomed_view() {
    let (mut h, fake) = cueing(1);
    let p = fake.player(0);
    let w = h.get_by_label(WAVE).rect();
    let centre = pos2(x_of(w, 90.0), w.center().y);
    for _ in 0..3 {
        wheel(&mut h, centre, 1.0);
    }
    assert_eq!(
        h.get_all_by_label(FULL_VIEW).count(),
        1,
        "only the CUE's waveform zoomed"
    );
    // 0.8³ of 180 s = 92.16 s around 90 s: from 43.92 s. A click three
    // quarters across lands at 43.92 + 0.75 · 92.16 ≈ 113 s.
    click(&mut h, pos2(x_of(w, 135.0), w.center().y));
    let sent = sent(&fake);
    assert!(
        !sent.iter().any(|c| matches!(c, Command::Seek(..))),
        "the CUE seeks, not the player: {sent:?}"
    );
    let seeks = cue_seeks(&sent, p);
    assert_eq!(seeks.len(), 1, "{sent:?}");
    assert!((seeks[0] - 113.0).abs() < 2.0, "{seeks:?}");
}

#[test]
fn the_cue_full_view_button_shows_the_whole_file_again() {
    let (mut h, fake) = cueing(1);
    let w = h.get_by_label(WAVE).rect();
    wheel(&mut h, pos2(x_of(w, 90.0), w.center().y), 1.0);
    h.get_by_label(FULL_VIEW).click();
    h.run_steps(2);
    assert!(h.query_by_label(FULL_VIEW).is_none());
    sent(&fake);
    click(&mut h, pos2(x_of(w, 135.0), w.center().y));
    let seeks = cue_seeks(&sent(&fake), fake.player(0));
    assert!((seeks[0] - 135.0).abs() < 1.5, "{seeks:?}");
}

#[test]
fn the_player_and_its_cue_zoom_independently() {
    let (mut h, _) = cueing(1);
    let cue = h.get_by_label(WAVE).rect();
    wheel(&mut h, pos2(x_of(cue, 90.0), cue.center().y), 1.0);
    // The stopped player shows the same track; its right end is clear of
    // the CUE window.
    let player = h.get_by_label(PLAYER_WAVE).rect();
    wheel(&mut h, pos2(x_of(player, 170.0), player.center().y), 1.0);
    assert_eq!(h.get_all_by_label(FULL_VIEW).count(), 2);
}

#[test]
fn a_new_cue_opens_on_the_whole_file() {
    let (mut h, fake) = cueing(1);
    let w = h.get_by_label(WAVE).rect();
    wheel(&mut h, pos2(x_of(w, 90.0), w.center().y), 1.0);
    assert!(h.query_by_label(FULL_VIEW).is_some());
    h.get_by_label(STOP).click();
    h.run_steps(2);
    fake.send(Command::ToggleCue(fake.player(0)));
    h.run_steps(2);
    assert!(h.query_by_label(WAVE).is_some(), "the CUE runs again");
    assert!(h.query_by_label(FULL_VIEW).is_none(), "on the same entry");
}

#[test]
fn a_cue_moved_to_another_entry_shows_it_whole() {
    let (mut h, fake) = cueing(1);
    let w = h.get_by_label(WAVE).rect();
    wheel(&mut h, pos2(x_of(w, 90.0), w.center().y), 1.0);
    let second = fake.entries()[1];
    fake.send(Command::CueEntry(fake.player(0), second));
    h.run_steps(2);
    assert!(h.query_by_label(FULL_VIEW).is_none());
}

#[test]
fn a_cue_without_a_known_length_does_not_zoom() {
    let mut s = state(1, 3);
    let first = s.playlists.iter().next().unwrap().entries[0].track;
    s.library.get_mut(first).unwrap().duration_secs = 0.0;
    let (mut h, fake) = harness(s);
    fake.send(Command::ToggleCue(fake.player(0)));
    h.run_steps(2);
    let w = h.get_by_label(WAVE).rect();
    wheel(&mut h, w.center(), 1.0);
    assert!(h.query_by_label(FULL_VIEW).is_none());
}

#[test]
fn the_cue_waveform_menu_sets_the_intro_here() {
    let (mut h, fake, track) = cueing_with_intro();
    let w = h.get_by_label(WAVE).rect();
    let at = pos2(x_of(w, 45.0), w.center().y);
    h.event(Event::PointerMoved(at));
    button(&mut h, at, PointerButton::Secondary, true, Modifiers::NONE);
    button(&mut h, at, PointerButton::Secondary, false, Modifiers::NONE);
    h.run_steps(2);
    h.get_by_label("Set intro end here").click();
    h.run_steps(2);
    let set = set_markers(&fake.take_sent());
    assert_eq!(set.len(), 1, "{set:?}");
    let (t, kind, secs) = set[0];
    assert_eq!((t, kind), (track, MarkerKind::IntroEnd));
    assert!((secs.unwrap() - 45.0).abs() < 2.0, "{secs:?}");
}

#[test]
fn alt_dragging_the_intro_on_the_cue_waveform_moves_it() {
    let (mut h, fake, track) = cueing_with_intro();
    let w = h.get_by_label(WAVE).rect();
    let from = pos2(x_of(w, 30.0), w.center().y);
    let to = pos2(x_of(w, 60.0), w.center().y);
    alt_drag_to(&mut h, from, to);
    button(&mut h, to, PointerButton::Primary, false, Modifiers::ALT);
    h.run_steps(2);
    let sent = fake.take_sent();
    let set = set_markers(&sent);
    assert_eq!(set.len(), 1, "one command on release: {sent:?}");
    let (t, kind, secs) = set[0];
    assert_eq!((t, kind), (track, MarkerKind::IntroEnd));
    assert!((secs.unwrap() - 60.0).abs() < 2.0, "{secs:?}");
    assert!(
        !sent
            .iter()
            .any(|c| matches!(c, Command::Seek(..) | Command::SeekCue(..))),
        "a marker drag does not seek"
    );
}

#[test]
fn stopping_the_cue_mid_drag_moves_no_marker() {
    let (mut h, fake, _) = cueing_with_intro();
    let p = fake.player(0);
    let w = h.get_by_label(WAVE).rect();
    let from = pos2(x_of(w, 30.0), w.center().y);
    let to = pos2(x_of(w, 60.0), w.center().y);
    alt_drag_to(&mut h, from, to);
    fake.send(Command::SetCue(p, false));
    h.run_steps(2);
    button(&mut h, to, PointerButton::Primary, false, Modifiers::ALT);
    h.event(Event::ModifiersChanged(Modifiers::NONE));
    h.run_steps(2);
    fake.send(Command::ToggleCue(p));
    h.run_steps(2);
    assert!(h.query_by_label(WAVE).is_some(), "the CUE runs again");
    assert!(set_markers(&fake.take_sent()).is_empty());
}
```

- [x] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-app --test cue_window`
Expected: FAIL. The zoom tests find no "Full view" (`the_wheel_zooms_…`: "only the CUE's waveform zoomed" left 0, right 1; `the_cue_full_view_button_…`, `the_player_and_its_cue_zoom_independently` and `a_new_cue_opens_on_the_whole_file` fail the same way), the menu test finds no "Set intro end here", the Alt-drag test sends no `SetMarker`. `a_cue_moved_to_another_entry_shows_it_whole`, `a_cue_without_a_known_length_does_not_zoom` and `stopping_the_cue_mid_drag_moves_no_marker` pass already (nothing zooms or drags yet); they turn into guards once the panel is in, and Step 4 shows the last one failing without `forget`.

- [x] **Step 3: Make the CUE window a thin caller**

In `crates/fp-app/src/ui/cue_window.rs`:

- the module doc (`:1-4`) becomes:

```rust
//! The CUE window (feedback 2 spec O12): one floating, non-modal window per
//! running CUE, with the waveform panel the player uses (zoom, pan, intro,
//! outro and MIX markers and their editing; operator feedback 4, Q7) and
//! the position, the elapsed and remaining time, and Pause/Resume, Stop and
//! Load as next. Closing it stops the CUE. Everything it does is a command;
//! nothing here waits for the engine.
```

- the imports (`:8-15`) become:

```rust
use fp_model::Command;

use super::app::{Scene, ViewState};
use super::format;
use super::glyphs::{self, TransportAction};
use super::theme;
use super::view::{self, CueWindowView};
use super::wave_panel::{self, WaveBadges, WavePanelInput};
use super::wave_view::WaveKey;
use super::widgets::{self, TileStyle, font, font_medium, font_semibold};
```

- `show_all` (`:22-38`) becomes:

```rust
/// Draws the window of every player that has a CUE running.
pub(crate) fn show_all(ctx: &egui::Context, scene: &Scene<'_>, view_state: &mut ViewState) {
    for (index, player) in scene.state.players.iter().enumerate() {
        if player.cue.is_none() {
            forget(view_state, WaveKey::Cue(player.id));
            continue;
        }
        let position = scene
            .telemetry
            .players
            .iter()
            .find(|(id, _)| *id == player.id)
            .and_then(|(_, t)| t.cue_position_secs);
        if let Some(v) = view::cue_window_view(scene.state, player.id, position) {
            show(ctx, scene, view_state, index, &v);
        }
    }
}

/// A player without a CUE forgets its CUE waveform's zoom, menu point and
/// marker drag, so the next CUE opens on the whole file and a drag cut off
/// by Stop never lands on it.
fn forget(view_state: &mut ViewState, key: WaveKey) {
    view_state.wave_zoom.set(key, None);
    view_state.wave_menu.remove(&key);
    if view_state.marker_drag.is_some_and(|(k, _, _)| k == key) {
        view_state.marker_drag = None;
    }
}
```

- `show` (`:40`) takes `view_state: &mut ViewState` after `scene`, and its `wave(ui, scene, v);` (`:78`) becomes `wave(ui, scene, view_state, v);`:

```rust
fn show(
    ctx: &egui::Context,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    index: usize,
    v: &CueWindowView,
) {
```

- `wave` (`:117-143`) becomes:

```rust
fn wave(ui: &mut egui::Ui, scene: &Scene<'_>, view_state: &mut ViewState, v: &CueWindowView) {
    let track = scene.state.playlists.entry(v.entry).map(|e| e.track);
    let media = track.and_then(|track| scene.media.get(track));
    let input = WavePanelInput {
        key: WaveKey::Cue(v.player),
        entry: Some(v.entry),
        track,
        media: media.as_ref(),
        total: v.total,
        markers: v.markers,
        mix_active: v.mix_active,
        // A CUE's zoom follows its position, as a playing player's does.
        follow: true,
        badges: WaveBadges {
            intro: v.intro,
            // The talk-over warning is for audio on air; a CUE is not.
            intro_blink: None,
            outro: v.outro,
        },
        editable_markers: true,
        height: WAVE_HEIGHT,
    };
    if let Some(secs) = wave_panel::show(ui, scene, view_state, &input).seek {
        scene.ctl.send(Command::SeekCue(v.player, secs));
    }
}
```

In `crates/fp-app/src/ui/app.rs:679`: `cue_window::show_all(&ctx, &scene, &mut self.view);` (`scene` borrows other fields of `self`, as `players_row` already shows).

- [x] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-app --test cue_window && cargo test -p fp-app --test waveform_ui && cargo test -p fp-app --test markers_ui && cargo test -p fp-app --test glyphs`
Expected: PASS — the new tests, the existing CUE window tests (`clicking_the_waveform_seeks_the_cue`, `a_cue_without_a_known_length_shows_zero_and_cannot_seek`, `two_cues_stack_two_windows`, …) and the player's waveform tests, unchanged.

To confirm `forget` is what makes `stopping_the_cue_mid_drag_moves_no_marker` and `a_new_cue_opens_on_the_whole_file` pass, comment out the `forget(…)` call, run `cargo test -p fp-app --test cue_window a_new_cue stopping_the_cue`, see both fail, and restore it.

- [x] **Step 5: Commit**

```bash
if cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings \
   && cargo test --workspace; then
  git add crates/fp-app/src/ui/cue_window.rs crates/fp-app/src/ui/app.rs \
    crates/fp-app/tests/cue_window.rs
  git commit -m "feat(ui): the CUE window gets the player's waveform panel

The CUE window zooms, pans, has Full view, shows the intro, outro and
MIX markers with their badges and edits them like the player, with its
own zoom and drag (operator feedback 4, Q7.2-Q7.6). A click still seeks
the CUE. A closed CUE window forgets its zoom and any drag.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
fi
```

---

### Task 6: Docs, spec and the guide image

**Executor model:** `sonnet` (documentation).

**Files:**
- Modify: `README.md:57-58`
- Modify: `docs/user/players.md` (section "CUE (pre-listen)", the waveform bullet)
- Modify: `docs/user/markers-and-mixing.md` (section "Editing markers", first line)
- Modify: `docs/technical/ui.md` (module table rows `ui/player.rs` and `ui/cue_window.rs`; sections "Marker editing", "Waveform view", "CUE window")
- Modify: `docs/superpowers/specs/2026-10-06-operator-feedback-4-design.md` (§4, As built)
- Check: `docs/superpowers/specs/2026-09-25-fauste-player-design.md` (§8.3 Waveform) and `docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md` (O12)
- Image: `docs/images/guide/cue-window.png`
- Locales: none (`crates/fp-app/locales/en-US/main.ftl` and `es-ES/main.ftl` unchanged; the panel reuses existing strings).

**Interfaces:**
- Consumes: the names of Tasks 1–5 (`wave_panel.rs`, `WavePanelInput`, `WaveKey`, `WaveZooms`, `marker_fractions`, `cue_window::forget`).
- Produces: nothing for code.

- [x] **Step 1: Check the spec lines Q7 changes are in place**

Run: `grep -n "same panel as the player's" docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md; grep -n "the CUE window uses the same waveform panel" docs/superpowers/specs/2026-09-25-fauste-player-design.md`
Expected: one line each (the spec author applied them with the spec). If either is missing, apply the text from §4 "Spec lines that change" of the feedback 4 spec verbatim.

- [x] **Step 2: User guide**

`README.md:57-58`:

```markdown
- **CUE pre-listen** on a separate device or channel pair, with a window per
  CUE to seek, zoom, edit markers, pause, stop and load the track as next.
```

`docs/user/players.md`, the CUE window's waveform bullet ("- the waveform of the whole file with the CUE position; click it to jump there;") becomes:

```markdown
- the waveform of the whole file with the CUE position. It works like the
  player's: click to jump there, zoom with the wheel, drag to move along,
  **Full view**, the intro and outro countdowns, and the intro, outro and MIX
  markers, which you edit here as on the player (see
  [Markers and mixing](markers-and-mixing.md)). A CUE plays the whole file,
  so nothing is drawn darker, the cue-in and cue-out are dimmed lines, and
  the outro counts down to the end of the file. Its zoom is its own: the
  player's waveform does not move, and a new CUE shows the whole file;
```

`docs/user/markers-and-mixing.md`, "On a player's waveform:" becomes:

```markdown
On a player's waveform, or on the waveform of its CUE window (the same
menu and handles; a change shows in both at once):
```

- [x] **Step 3: Technical docs (`docs/technical/ui.md`)**

- Module table: after the `ui/player.rs` row add

```markdown
| `ui/wave_panel.rs` | The waveform panel the player column and the CUE window share (operator feedback 4, Q7): `show(ui, scene, view_state, &WavePanelInput)` draws `widgets::waveform` with the zoom, the follow after the grace, wheel and drag pan, the Full view button, the intro and outro badges and marker editing, keyed by `WaveKey`; it returns `WavePanelOutput { seek }`, which the player sends as `Seek` and the CUE window as `SeekCue` |
```

  and the `ui/cue_window.rs` row becomes: "One non-modal `egui::Window` per running CUE, drawn from the pure `view::cue_window_view`; its waveform is `wave_panel` with `WaveKey::Cue`; icons through `ui/glyphs.rs`".
- "Marker editing": `player.rs::edit_markers` becomes `wave_panel.rs::edit_markers`; `ViewState::wave_menu` and `ViewState::marker_drag` are "keyed by `WaveKey` (`Player(id)` or `Cue(id)`), so a player and its CUE window keep their own menu point and drag". Add: "Both waveforms place the markers with the pure `view::marker_fractions`; the CUE passes `ignored = true`, since a CUE plays the whole file."
- "Waveform view": "`ViewState::wave_zoom` keeps a `WaveZoom` per zoomed player" becomes "`ViewState::wave_zoom` (`WaveZooms`) keeps a `WaveZoom` per zoomed waveform, keyed by `WaveKey`; `WaveZooms::get` returns it only for the entry it was made on". Add: "The wheel rules are pure: `wave_view::wheel_notches` converts lines, points and pages to notches and `WaveView::wheel` zooms or pans by them. Each `WaveKey` has its own egui id (`WaveKey::id`), so the memoised columns and the pan-drag flag of a player and its CUE never mix."
- "CUE window": after the first sentence add: "Its waveform is the shared panel (`WaveKey::Cue(player)`): `cue_window_view` gives it `markers` (`marker_fractions` on the whole file, cue edges dimmed), `intro` (never blinking), `outro` (to the end of the file) and `mix_active` (the player's mode). `show_all` receives `&mut ViewState`; for a player without a CUE, `forget` drops its CUE key's zoom, menu point and marker drag."

- [x] **Step 4: Spec "As built"**

At the end of §4 Q7 in `docs/superpowers/specs/2026-10-06-operator-feedback-4-design.md`, add:

```markdown
- **As built (plan 3).** `ui/wave_panel.rs` (`WavePanelInput`, `WaveBadges`,
  `WavePanelOutput`); `wave_view::{WaveKey, WaveZooms, wheel_notches,
  WaveView::wheel}`; `view::{marker_fractions, intro_left, outro_left}`;
  `CueWindowView::{markers, intro, outro, mix_active}`. `ViewState::wave_menu`
  is keyed by `WaveKey` too. The CUE's intro badge never blinks, its MIX
  marker follows the player's mode, its zoom follows its position, and a
  closed CUE window forgets its zoom, menu point and marker drag.
```

- [x] **Step 5: The guide image**

Run: `scripts/site/screenshots.sh`
Expected: `docs/images/guide/cue-window.png` is rewritten; open it and check that the window has the same size (the crop `406x222+164+204` in the script) and that its waveform shows the scene's markers. If the window grew, adjust the crop in `scripts/site/screenshots.sh` and run again. If Xvfb, xdotool, ImageMagick or ffmpeg is missing here, leave the image and record `Ruling: cue-window.png not re-taken — <tool> missing — the guide image lacks the markers until the next screenshot run` in the ledger and the PR description.

- [x] **Step 6: Build the site's links and commit**

Run: `scripts/site/build.sh /tmp/fp-site && scripts/site/check-links.sh /tmp/fp-site` (needs `gh` or `FAUSTE_RELEASE_JSON`; if neither is available, skip and say so in the PR).
Expected: no dead link.

```bash
if cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings \
   && cargo test --workspace; then
  git add README.md docs/user/players.md docs/user/markers-and-mixing.md \
    docs/technical/ui.md docs/superpowers/specs/2026-10-06-operator-feedback-4-design.md \
    docs/images/guide/cue-window.png scripts/site/screenshots.sh
  git commit -m "docs: the CUE window's waveform panel

The guide, the technical notes and the spec describe the shared panel:
zoom, pan and marker editing in the CUE window, keyed apart from the
player's (operator feedback 4, Q7).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
fi
```

---

## Self-review against spec §4 (Q7)

| Rule | Task |
|---|---|
| Q7.1 `wave_panel.rs`, input struct with key, entry, track, media, total, markers (with the position), `mix_active`, `follow`, badges, `editable_markers`, height (`seekable` dropped in Task 5: every waveform seeks); output `{ seek }` | 3 |
| Q7.2 `wave_zoom` and `marker_drag` keyed by `WaveKey` (and `wave_menu`, ruling) | 2; kittest `the_player_and_its_cue_zoom_independently` (5) |
| Q7.3 thin callers; the CUE window receives `&mut ViewState` | 3 (player), 5 (CUE) |
| Q7.4 CUE: markers, menu, Alt-drag, zoom, pan, Full view | 4, 5 (`the_cue_waveform_menu_sets_the_intro_here`, `alt_dragging_the_intro_on_the_cue_waveform_moves_it`, `the_wheel_zooms_…`, `the_cue_full_view_button_…`); pan is the panel's code under `waveform_ui.rs` |
| Q7.5 cue edges dimmed without shading; badges against the end of the file | 4 (`the_cue_window_places_the_markers_…`, `the_cue_window_badges_…`) |
| Q7.6 a marker edited in one place shows in both | 4 (`a_marker_set_once_shows_in_the_player_and_in_its_cue`) |
| `marker_fractions` extracted and unit-tested, used by both | 1, 4 |
| Tests: `marker_fractions`, `WaveKey`-keyed zoom state, CUE wheel zoom, `SeekCue` click, Alt-drag of the intro; player tests unchanged | 1, 2, 5; Tasks 2, 3 and 5 rerun `waveform_ui.rs` and `markers_ui.rs` unedited |
| Docs: `players.md`, `markers-and-mixing.md`, `ui.md`; strings reused | 6 |
