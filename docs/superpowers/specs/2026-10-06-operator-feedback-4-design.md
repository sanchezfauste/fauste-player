# Operator Feedback 4 — Design Spec

- **Date:** 2026-10-06
- **Status:** Plans 1 to 5 implemented (Q6 closed without a product change, see plan 2's "As built"), except plan 5's Task 7, the check on real hardware.
- **Extends:** [the main design spec](2026-09-25-fauste-player-design.md) (§3 rules,
  §6 analysis, §8 UI), the [meters spec](2026-09-27-meters-design.md) (M4 display),
  the [bit-perfect spec](2026-09-26-phase4-bit-perfect-design.md) (B3 rate, B6
  settings) and the [second operator feedback spec](2026-10-01-operator-feedback-2-design.md)
  (O12, O17 and O23 are amended here).
- **Scope:** the fourth round of operator feedback, items Q1–Q14 (the
  numbers follow the operator's list; Q13 is not part of this spec, and Q14 is
  out of scope, §8). The items are grouped into five plans. Each plan is
  written in full just before it runs, against the code the previous plan
  left. Each reaches `master` through its own pull request and updates the
  main spec, the user guide and the technical docs for what it changes.
- **Numbering:** the items are `Q<n>` so that they never clash with the first
  round (`F`), the second round (`O`) or the main spec's rule numbers (`R`).
  The rules of an item are numbered `Q<n>.<k>`; each rule is testable and
  gets at least one test.
- **Line numbers** are those of `master` at the date above; they drift, so
  each reference also names the function or field.

---

## 1. Items and plans

| Item | Summary | Plan |
|---|---|---|
| Q1 | A track played before its analysis has no countdown, position or seek | 2 |
| Q2 | An indicator for a track whose analysis is pending | 1 |
| Q3 | The DSD playback mode is hard to find | 5 |
| Q4 | The drop indicator shows in the wrong place | 1 |
| Q5 | The CUE window's Pause does not blink | 1 |
| Q6 | The CUE does not follow a row selected in the same player's playlist | 2 |
| Q7 | The CUE window's waveform lacks zoom and markers | 3 |
| Q8 | Seek a stopped player to choose where Play starts | 2 |
| Q9 | The track tooltip hides the file error and jumps on its first frame | 1 |
| Q10 | A file error stays until *Re-analyse all* | 1 |
| Q11 | Meter: nothing drawn over the bars; a ruler on each side | 4 |
| Q12 | Outputs settings: Basic and Advanced, per-device rate and buffer | 5 |
| Q14 | Multichannel (5.1) playout | out of scope (§8) |

| Plan | Title | Items | Depends on |
|---|---|---|---|
| 1 | UI fixes | Q2, Q4, Q5, Q9, Q10 | — |
| 2 | Play before analysis, start point, CUE follow | Q1, Q8, Q6 | — |
| 3 | Shared waveform panel | Q7 | 2 (Q1 and Q8 change the player's waveform that plan 3 moves) |
| 4 | Meter rulers | Q11 | — |
| 5 | Outputs settings | Q12, Q3 | — |

---

## 2. Plan 1 — UI fixes

### Q2. Pending-analysis indicator

- **Problem.** A track that is still waiting for its analysis looks like any
  other row; the operator cannot tell why it has no waveform yet.
- **Root cause.** Nothing in the table reads `Track.analyzed`, although it is
  in the snapshot. The Title cell's right-to-left flag strip
  (`crates/fp-app/src/ui/table.rs` ~L389–401, the `flag(...)` helper at
  ~L859) only shows `flag-outdated`.
- **Rules.**
  - **Q2.1** A row whose track is not analysed (`!track.analyzed`) and whose
    file is playable (`file_state.is_playable()`) shows a muted hourglass
    glyph in the Title cell's flag strip, with the tooltip "Analysis pending"
    (`flag-analysis-pending`).
  - **Q2.2** The flag is not shown for a missing or unreadable file (those
    rows already have their own icon) nor for an analysed track.
  - **Q2.3** When the analysis completes, the flag disappears on the next
    snapshot. An outdated track (analysed by an earlier version) keeps the
    `flag-outdated` flag, never the hourglass.
- **Implementation notes.** A pure `view::analysis_pending(track) -> bool`
  decides; `table.rs` paints through the existing `flag`. The glyph is a
  Phosphor hourglass in `NEUTRAL_400`.
- **Spec lines that change.** Main spec §8.3, Track table, the icons
  bullet: after "with a tooltip pointing to Settings → Analysis;" add "tracks
  not analysed yet show a muted hourglass there, with the tooltip 'Analysis
  pending' (operator feedback 4, Q2);".
- **Tests.** `view` unit tests for Q2.1–Q2.3; a kittest in
  `crates/fp-app/tests/table_icons.rs` that finds the flag by its accessible
  name for a pending track and not for an analysed or unreadable one.
- **Docs and locales.** `flag-analysis-pending` in en-US and es-ES;
  `docs/user/playlists.md` (row icons); `docs/technical/ui.md`.

### Q4. Drop indicator in the wrong place

- **Problem.** While an entry is dragged, the violet drop line sometimes
  shows at the end of the list, in both players that show the same playlist,
  or not at all.
- **Root cause.** The bar is painted at `table.rs` ~L289–303 from
  `ViewState.drop: Option<DropTarget { playlist, index }>`
  (`crates/fp-app/src/ui/app.rs` L61, L99), filtered per table at ~L165 and
  written at ~L553–600.
  - (a) The target is keyed by the playlist only, so two players that show
    the same playlist both draw it.
  - (b) The hovered row comes from each row's `dnd_hover_payload` /
    `contains_pointer`, which is false over the column-resize handles
    (~L634–648), the header and the scroll bar, while `pointer_in` uses
    `ui.max_rect()`. Over those gaps the target falls to `entries.len()`, the
    end of the list.
  - (c) No bar is drawn when the target row is outside the virtualised rows
    painted this frame.
- **Rules.**
  - **Q4.1** The drop index is computed from the pointer's y against the
    table body's geometry: the body rect, the scroll offset and `ROW_HEIGHT`.
    The index is the row boundary nearest to the pointer, clamped to
    `0..=entries.len()`; it never depends on which widget is under the
    pointer.
  - **Q4.2** A target exists only while the pointer is inside the body rect.
    Over the header, the column-resize handles, the scroll bar, a floating
    window that covers the table (the CUE window) or outside the table there
    is no target, and a release there drops nothing.
  - **Q4.3** The target is keyed by `(PlayerId, PlaylistId)`; only the table
    of that player draws the bar. Two players that show the same playlist
    never both draw it.
  - **Q4.4** The drop itself stays playlist-based: releasing inserts at the
    index in that playlist, as today.
  - **Q4.5** The bar is drawn at the boundary's y in the body even when the
    rows next to it are not painted this frame; a boundary outside the
    visible body draws nothing.
  - **Q4.6** OS file drops follow the same rules (`ViewState.file_drop`):
    a pointer over a table but on no target inserts nothing. Only a drop
    whose position the system does not report, or that lies outside every
    table, goes to the end of the list shown.
- **Implementation notes.** A pure `table_layout::drop_index(body: Rect,
  scroll_y: f32, row_height: f32, len: usize, pointer: Pos2) ->
  Option<usize>`; `DropTarget` gains `player: PlayerId`. The bar is painted
  after the body, from the same geometry.
- **Spec lines that change.** Main spec §8.3, Interactions: "drag & drop
  within a list, to another player's list and onto tabs, with a violet drop
  indicator line;" becomes "drag & drop within a list, to another player's
  list and onto tabs, with a violet drop indicator line at the row boundary
  under the pointer, drawn only in the table under the pointer (operator
  feedback 4, Q4);".
- **Tests.** Unit tests of `drop_index` (top, middle, bottom, scrolled, empty
  list, pointer outside); kittest: with two players on one playlist, only the
  hovered table draws the bar; hovering a resize handle gives no target.
- **Docs and locales.** No strings. `docs/technical/ui.md` (drag and drop).

### Q5. The CUE window's Pause does not blink

- **Problem.** A paused CUE looks the same as a running one in its window,
  while a paused player's Pause button blinks amber.
- **Root cause.** The blink lives inside the player:
  `blink = (scene.time * 2.0).floor() as i64 % 2 == 0` and an amber
  `TileStyle` (`crates/fp-app/src/ui/player.rs` ~L637, L696–716). The CUE
  window's buttons (`crates/fp-app/src/ui/cue_window.rs` `buttons`, ~L155–176)
  do not use it.
- **Rules.**
  - **Q5.1** While a CUE is paused (`v.paused`), its window's Pause tile
    blinks amber every 500 ms, the same phase and look as the player's
    (main spec rule 6).
  - **Q5.2** A running CUE's tile does not blink.
  - **Q5.3** While any CUE is paused the UI requests a repaint, so the blink
    runs without pointer movement.
- **Implementation notes.** `widgets::blink(time: f64) -> bool` and
  `widgets::paused_style(blink: bool) -> TileStyle`, used by both the player
  and the CUE window.
- **Spec lines that change.** Feedback 2, O12, the list of what the window
  shows: add "While the CUE is paused, its Pause button blinks amber like the
  player's (operator feedback 4, Q5)."
- **Tests.** Unit tests of `blink` (phase at 0.0, 0.49, 0.5, 1.0) and
  `paused_style`; a kittest in `tests/cue_window.rs` that the paused tile's
  fill alternates between two frames half a second apart.
- **Docs and locales.** No strings. `docs/user/players.md` (CUE window).

### Q9. Track tooltip

- **Problem.** The reason a file is missing or unreadable is never shown on
  hover, and the track info popup first appears for one frame above the row
  before it jumps below.
- **Root cause.**
  - egui allows one tooltip per layer. The row's response (`Sense::click_and_drag`,
    `table.rs` ~L182, `on_hover_ui` at ~L504) wins over the label tooltips on
    the number and the title (`on_hover_text`, ~L344–368 and ~L444–448), so
    the file error reason (`Scene::file_tip`, `app.rs` ~L253) never shows.
  - The jump comes from egui's sizing pass: the popup's width depends on its
    content, so the first frame is laid out at a guessed size above the
    anchor, and the next one flips below it.
- **Rules.**
  - **Q9.1** For a missing or unreadable file, the track info popup starts
    with the error reason, in amber, after the file icon glyph, wrapped to
    the popup's width; the other fields follow as today.
  - **Q9.2** The label tooltips on the number and the title are removed; the
    row popup is the only hover information on a row.
  - **Q9.3** The popup has a fixed width (`set_width`), so the sizing pass
    gives the final size and it is never shown at another place first.
  - **Q9.4** The cartwall's carts show the same error line in their tooltip.
- **Implementation notes.** `view::track_tooltip` gains the reason (from
  `file_tip`), so the content stays a pure function; the width is a constant
  in `table.rs`.
- **Spec lines that change.**
  - Main spec §8.3, Track table: "unreadable rows with a warning icon, whose
    tooltip gives the reason and the file path (the cartwall's carts show the
    same icons and tooltip)" becomes "unreadable rows with a warning icon; the
    row's track info popup starts with the reason, in amber (operator
    feedback 4, Q9; the cartwall's carts show the same icons and tooltip)".
  - Feedback 2, O23 (Tooltip) and §10 (missing-file M2): add "Operator
    feedback 4, Q9, replaces the title and number label tooltips: the reason
    is the first line of the row's popup, which has a fixed width."
- **Tests.** `view::track_tooltip` unit tests for the reason line; a kittest
  that hovering an unreadable row shows the reason and that the popup's rect
  is the same on its first and second frame.
- **Docs and locales.** No new strings (the reasons exist).
  `docs/user/playlists.md` (tooltip).

### Q10. A file error persists until *Re-analyse all*

- **Problem.** A file that failed once (a partial copy, a file being
  written) stays unreadable after it is fixed, until the operator runs
  *Re-analyse all* on the whole library.
- **Root cause.** By design (main spec §6, ~L379) `Unreadable` files are never
  retried; `Services.failed` (`crates/fp-app/src/services.rs` ~L241) skips
  them. Playback failures (`EngineEvent::SourceFailed` / `PreloadFailed` in
  `crates/fp-model/src/reducer.rs` ~L400–403) also set `Unreadable`.
- **Rules.**
  - **Q10.1** For each track whose file is `Unreadable` (from the analysis or
    from playback), `Services` records the file's size and modification time
    when it first sees the failure.
  - **Q10.2** On the existing `tuning.missing_recheck_ms` probe timer, the
    probe thread `stat`s those files; it never opens or decodes them. If the
    size or the modification time changed, the failure is cleared and the
    track is analysed again. If neither changed, nothing happens (no
    analysis, no autosave).
  - **Q10.3** A file that disappeared becomes `Missing` and follows the
    existing missing-file recheck.
  - **Q10.4** The row context menu gains **Re-analyse**
    (`ServiceRequest::ReanalyseTrack(TrackId)`). It clears the failure and
    analyses that track at once, whatever its state; manual markers are kept,
    as with *Re-analyse all*.
  - **Q10.5** A successful analysis makes the track playable again, with its
    markers. A failed one leaves it `Unreadable` with the new stat recorded.
  - **Q10.6** The stat runs off the services thread and the UI thread, one
    look at a time, like the missing-file probe.
- **Implementation notes.** `Services.failed` becomes a map `TrackId →
  Option<(u64, SystemTime)>`; the probe job takes both the missing and the
  failed lists. `ServiceRequest::ReanalyseTrack` reuses the urgent queue.
- **Spec lines that change.** Main spec §6: "`Unreadable` files are not
  retried by themselves (that would decode them every time): *Re-analyse all*
  checks them again." becomes "`Unreadable` files are not decoded again by
  themselves: on the same timer the probe compares their size and
  modification time with those seen at the failure, and a change sends the
  file to the analysis pool again. The row menu's **Re-analyse** and
  *Re-analyse all* check them at once (operator feedback 4, Q10)." Main spec
  §8.3, the context menu list: add "Re-analyse" after "Pre-listen on CUE".
- **Tests.** `services` tests with a temporary file: unchanged stat → no
  request; a new mtime or size → one analysis request; a removed file →
  Missing; `ReanalyseTrack` → one urgent request and the failure cleared. A
  kittest that the menu item sends the request.
- **Docs and locales.** `menu-reanalyse` (and its tooltip if any) in en-US
  and es-ES; `docs/user/playlists.md`, `docs/user/troubleshooting.md`,
  `docs/technical/analysis.md`.

### As built (plan 1)

- **Q5.** `widgets::{blink, paused_style}` are shared by the player and the CUE
  window; `view::animating` pins the repaint rule (no new rule: a paused CUE
  still has `cue` set).
- **Q2.** `view::analysis_pending` and a muted hourglass (`flag-analysis-pending`).
- **Q9.** `view::track_tooltip` takes the reason; `TIP_WIDTH` fixes the width.
- **Q4.** `table_layout::{drop_index, boundary_y, on_column_edge}`; `DropTarget`
  gains the player.
- **Q10.** The probe thread `stat`s unreadable files; `Services::stamps`;
  `ServiceRequest::ReanalyseTrack`; the row menu's **Re-analyse**
  (`menu-reanalyse`).
- **Known limitation (Q4).** The wheel does not scroll the list while a row is
  being dragged: egui 0.36's `ScrollArea` ignores it while a widget is dragged.
  There is no edge auto-scroll either.
  - **Lifted after plan 5 (as built).** The maintainer asked for scrolling
    during a drag. While a drag a table accepts is held (a `DragEntry`, or
    files the system hovers over the window) and the pointer is in a table
    body, on the table's own layer:
    - within `table_layout::DRAG_SCROLL_EDGE` (36 points, or a third of a
      body shorter than 108 points, so its middle never scrolls) of the
      body's top or bottom, the table scrolls toward that edge at a speed
      that ramps linearly from 0 at the inner side of the zone to
      `DRAG_SCROLL_MAX_SPEED` (900 points per second, about 32 rows) at the
      edge (`drag_scroll_speed`, unit-tested), times the frame's
      `stable_dt` (at most 0.1 s, `table::MAX_DRAG_SCROLL_DT`), with a
      repaint requested while it moves; it stops at the list's ends, when
      the pointer leaves the zone or the drag ends, and not on the frame the
      button is released, so the entry lands where the line was drawn;
    - the wheel scrolls the table: egui ignores it while a widget is
      dragged, so the table applies `smooth_scroll_delta.y` itself exactly
      then and consumes it (no double scroll);
    - the drop index and the violet line come from the new offset, so they
      follow the pointer as the rows move; a header (`DragColumn`) drag
      scrolls nothing. Every player table, in every tab and layout, does it.
    Both values are UI metrics (named constants, not `Config` fields).
    Tests: `fp-app/tests/table_drag_scroll_ui.rs` and the
    `drag_scroll_*` cases of `fp-app/tests/table_layout.rs`.

Rulings made while implementing:

- Q5.3 needs no new repaint rule: `AppUi::ui` already repaints every frame while `p.cue.is_some()`, and a paused CUE still has `cue` set; it is extracted as `view::animating` so that a test pins it.
- The main spec lines of §6, §8.3 and the feedback 2 spec lines about Q5/Q9 were already edited with this design spec, so only these notes were added.
- Q9.4 (carts show the same error line) needs no code: the cartwall already shows `Scene::file_tip` on hover, and `hovering_an_unavailable_cart_says_why` pins it.
- Q9.1 reuses the existing `file-missing-tip` / `file-unreadable-tip` strings, which end with the path, so the path shows twice (in the reason and in the Path row). Cost if wrong: two new reason-only strings.
- Q9.2 removes the file-error tooltips of the number and the title only; the `P<n>` mark and the "next again" arrow keep their own tooltips because they explain a glyph, not the file.
- Q4.1 vs Q4.2: the plan follows Q4.2 for the column resize handles (a grab zone of `resize_grab_radius_side` on each side of an interior edge gives no target) and Q4.1 for everything else. Cost if wrong: the bar blinks off for about 8 points when a drag crosses an edge.
- `Services.failed` stays a `HashSet<TrackId>` and the stats go in a new `stamps: HashMap<TrackId, Seen>`: playback failures set `Unreadable` in the model without going through `failed`, and putting them in `failed` would change when they are analysed. This replaces the implementation note above (a map).
- After a failed analysis the recorded stat is the one the analysis worker took just before decoding (`AnalysisResult::stamp`), so a file that changed while it was analysed is analysed again at the next look. A playback failure (or an analysis whose stat failed) has no such stat: its first look is sent at once and alone, not waited for the timer. Later looks follow `tuning.missing_recheck_ms`.
- A `stat` error other than "not found" records `Seen::Unknown` and is otherwise ignored; "not found" sends `SetFileState(Missing)` (Q10.3). Cost if wrong: a permission fix is only noticed through Re-analyse.
- **Re-analyse** goes through the same analyzer and cache as *Re-analyse all*, so an unchanged file whose analysis is cached comes back playable and fails again on the next playback. Cost if wrong: a cache bypass flag on `submit_urgent`.
- The kittests read drawn shapes through `Harness::output().shapes` because the fill of a tile and the violet bar are painted shapes, not accessible nodes.
- The Q9 popup lays out its fields as horizontal rows with a measured label column rather than a `Grid` (the popup moved over two frames after its sizing pass); its test starts at the first frame that paints the label.
- The drop-geometry tests keep only what is below the table header; the empty-playlist test compares the bar with the header label's centre plus half the header height. No scroll-bar width is subtracted from `inner_rect`: egui's `inner_rect` already excludes the bar.
- The scrolled long-list test scrolls first, then drags: egui ignores the wheel while a widget is dragged. If the operator expects the wheel (or edge auto-scroll) during a drag, that is new behaviour to add.
- A `changed` answer for a track already being analysed is dropped and its old stamp kept, so the next timed look reports it changed again and it is analysed once more after the running analysis. Cost if wrong: one extra analysis per such race.

---

## 3. Plan 2 — Play before analysis, start point, CUE follow

### Q1. A track played before its analysis

- **Problem.** A track played before its analysis has no countdown, no
  position and cannot be seeked.
- **Root cause.** `Track.duration_secs` is 0 until `Track::apply_analysis`
  (`crates/fp-model/src/track.rs` ~L478). `view::player_view`
  (`crates/fp-app/src/ui/view.rs` ~L132) then has `total = None`, and
  `widgets::waveform` (`widgets.rs` ~L1155) returns before drawing the
  playhead or seeking. `FileDecoder::frames_hint()`
  (`crates/fp-decode/src/lib.rs` ~L195) gives the container's frame count
  without decoding, but nothing uses it.
- **Rules.**
  - **Q1.1** When a track is added to the library or shown in a player, a
    worker thread (never the UI thread) opens the file's header and reads its
    duration from `frames_hint()` and the sample rate.
  - **Q1.2** `Command::SetDuration { track, secs }` stores it only while the
    track is not analysed (`!analyzed`) and `secs` is finite and positive.
    Once analysed, it is ignored; the analysis always replaces it.
  - **Q1.3** With a known duration, a track not analysed yet has the
    countdown, `elapsed / total`, the position and click-to-seek, as an
    analysed one. The wave box shows the centre line and the playhead; there
    is no waveform until the analysis.
  - **Q1.4** Without a hint (some VBR files), the elapsed time is shown, the
    total reads "—", and seeking stays off.
  - **Q1.5** The play range of such a track is `0..duration` (main spec §6,
    "Playability before analysis").
- **Implementation notes.** The header read can live with the analysis pool
  (a cheap urgent job) or a small worker in `Services`; plan 2 chooses. The
  reducer rule is pure and tested. Progressive waveform drawing is out of
  scope (§8).
- **Spec lines that change.** Main spec §6, "Playability before analysis":
  "Until then it has no waveform or segue start, and `cue_in = 0`, `cue_out =
  duration`." becomes "Until then it has no waveform or segue start, and
  `cue_in = 0`, `cue_out = duration`, where the duration is read from the
  file's header off the UI thread when the container gives it (operator
  feedback 4, Q1); without it, the total is unknown and the track cannot be
  seeked."
- **Tests.** Reducer tests for Q1.2 (stored before, ignored after, rejected
  when not finite); `view::player_view` with a header duration; a decode
  test of `frames_hint` on WAV, FLAC and a VBR MP3 without a header; a
  kittest that a click on the wave box of such a track sends `Seek`.
- **Docs and locales.** `docs/user/players.md`, `docs/technical/analysis.md`.

### Q8. Seek a stopped player

- **Problem.** The operator wants to choose where the next track starts
  before pressing Play, by clicking the stopped player's waveform.
- **Root cause.** It is blocked in three places: the UI
  (`player.rs` ~L913, `seekable: pv.status != PlayerStatus::Stopped`), the
  reducer (`Command::Seek`, `reducer.rs` ~L122–133, which acts only when not
  Stopped and only on `current`), and the main spec (§8.3: "the waveform
  shows times but does not seek").
- **Rules.**
  - **Q8.1** `PlayerState` gains `pending_start: Option<(EntryId, f64)>`,
    `#[serde(default)]`.
  - **Q8.2** `Command::Seek` while Stopped, with a `next` entry, stores
    `pending_start = (next, secs)`, with `secs` clamped by `request_at` to the
    entry's play range. It sends no engine action. With no `next` it does
    nothing.
  - **Q8.3** Play while Stopped (`play`, `reducer.rs` ~L422 → `advance_to`
    ~L1126) starts the `next` entry at the pending start when the pending
    entry is that `next`, instead of its cue-in, for that one start; the
    pending start is then cleared.
  - **Q8.4** The pending start is cleared when `next` changes, when its entry
    is removed or moved, on Stop, when it is used, and by every other start
    path (Play now, a remote or MIDI start of another entry).
  - **Q8.5** Restart, the automatic advance, the segue and Previous keep
    the cue-in; none of them reads the pending start.
  - **Q8.6** The view shows the pending start as the stopped player's
    position: the playhead and the countdown. The stopped waveform is
    seekable; its zoom still does not follow the position.
  - **Q8.7** The engine's preload of a stopped player's next entry
    (`preload_target`, `reducer.rs` ~L1245) starts at the pending start, so
    Play starts without a gap; a new pending start re-preloads.
  - **Q8.8** A pending start is not on air: nothing plays by itself (rule 10),
    and the session restore keeps it only while its entry is still `next`.
- **Spec lines that change.**
  - Main spec §3.1: add "`pending_start: Option<(EntryId, f64)>`: where Play
    starts the `next` entry from Stopped, when the operator chose (rule 3a)."
  - Main spec rule 3, "If `next` exists, it becomes `current` and starts;"
    becomes "If `next` exists, it becomes `current` and starts, at its cue-in
    or at its pending start (rule 3a);".
  - Main spec, new rule **3a. Pending start** (operator feedback 4, Q8): the
    text of Q8.2–Q8.5.
  - Main spec §8.3, Info row: "the waveform shows times but does not seek, a
    zoom on it does not follow the pinned position" becomes "a click on the
    waveform sets where Play starts (rule 3a) and the playhead and countdown
    show it, a zoom on it does not follow the pinned position".
- **Tests.** One reducer test per rule Q8.2–Q8.8 in `fp-model`; a
  `view::player_view` test; a kittest that a click on a stopped player's
  waveform sends `Seek`; an engine test that the preload starts at the
  pending start (Offline backend).
- **Docs and locales.** `docs/user/players.md`; `docs/technical/ui.md`;
  `docs/technical/audio-engine.md` (preload). The remote API's seek follows
  the same rule; `docs/technical/remote-api.md` says so.

### Q6. The CUE does not follow a row in the same player's playlist

- **Problem.** With a CUE running on a player, clicking or double-clicking
  another row of a playlist that player shows does not move the CUE,
  although feedback 2, O17 says it should.
- **Code today.** A single click goes through `view::cue_follow_target`
  (`view.rs` ~L159) and `table.rs` (~L500–503, ~L546–548) sends
  `Command::CueEntry`. A double-click (`SetNext`) moves it through
  `follow_cue` (`reducer.rs` ~L855). The guards are `player_has_cue`,
  `playable_request` and "not already cued".
- **Root cause.** Not found yet. The maintainer confirms the failure in the
  same player's playlist, so it is a real defect. Plan 2 reproduces it with
  a failing UI test first, then finds and fixes the cause
  (`superpowers:systematic-debugging`).
- **Rules.**
  - **Q6.1** With a CUE running on player P, a single click on a playable
    row of a playlist P shows moves the CUE to that row's entry.
  - **Q6.2** In the same situation a double-click (which sets P's next) moves
    the CUE to that entry too.
  - **Q6.3** Both hold with the CUE window open and with the playlist on any
    of P's tabs; the existing guards (unplayable file, already cued, no CUE
    route) still apply.
- **Spec lines that change.** Feedback 2, O17: add "Operator feedback 4, Q6,
  fixes a defect where this did not happen in the player's own playlist; the
  rules Q6.1–Q6.3 restate it."
- **Tests.** kittest with the recording `Fake` controller
  (`crates/fp-app/tests/support`), `with_step_dt(0.02)`: a click and a
  double-click on a row, with the CUE window open, each send the command that
  moves the CUE; reducer tests for `follow_cue` if the cause is in the model.
- **Docs and locales.** `docs/user/players.md` if the described behaviour
  changes (it should not).

### As built (plan 2)

- **Q6.** Not reproduced. `cue_follow_ui.rs::q6_1_a_click_on_a_row_moves_the_cue_with_its_window_open`, `q6_2_a_double_click_on_a_row_moves_the_cue_with_its_window_open` and `q6_3_the_cue_follows_in_every_layout_tab_and_state` (a matrix of layouts, tabs, windows and gestures, with real pointer events) pass on the product as it was, so no product code changed and the tests pin the behaviour. The maintainer's exact steps are pending (players, window size, how the CUE was started, whether the CUE window covered the row, click or double-click, the file's state, the Cue route, and whether only the window title changed).
  - After plan 2 the maintainer gave the steps: two players, the CUE started with the player's own CUE button (it sends `ToggleCue`, which cues the player's next entry from its cue-in), then a click or a double-click on another row of that player's table. Still not reproduced. `q6_4_after_the_players_cue_button_a_row_moves_the_cue_in_two_players` clicks the CUE button itself with real pointer events, at 1000×700 and 1920×1080, on either player, with a click or a double-click, with and without resting on the row long enough for its tooltip; every case moves the CUE (a mutation that disables `cue_follow_target` makes its click cases fail). The release build, run in Xvfb on the null backend through `scripts/site/screenshots.sh --hold` and driven with `xdotool` clicks, also moved the CUE on a stopped player and on one on air, for a click and a double-click. One case does not move it, by design (feedback 2, O17 says "in that player's table"): a click or double-click in another player's table that shows the same playlist. Two new players both show the first playlist, so this is a candidate for the report; it is left for the maintainer.
  - **Closed by the maintainer.** A click or double-click in another player's table, even one that shows the same playlist, does not move this player's CUE, and that is the intended behaviour (feedback 2, O17: "in that player's table"). Q6 is closed with no product change; the Q6 tests stay as regression tests.
- **Q1.**
  - `FileDecoder::duration_hint_secs` and `duration_from_frames` (`fp-decode`) give the length the file stores, and never an estimate, which can be short on VBR. An allowlist of symphonia readers (`symph.rs::stored_length`) is trusted: FLAC, WAV, AIFF, CAF, MP4/M4A and Ogg; MP1/2/3 only when the first Layer III frame after any ID3v2 tag has `Xing`/`Info` after its side information or `VBRI` 32 bytes after its header (checked when the file is opened). Without one, the stream is probed so that symphonia makes no estimate at all: its MPEG reader would otherwise take the estimate as the end of the stream, decode nothing past it and refuse seeks past it, cutting a VBR track short in playback and in the analysis (`ANALYSIS_VERSION` 8). Raw AAC (ADTS), Matroska/WebM and any other reader get no hint; WavPack, Monkey's Audio and DSD use their own headers.
  - `fp-app/src/header.rs` (`HeaderReader`, the `fp-header-reader` thread) and `Services::header_pass` read the headers, with eight reads in flight (`HEADER_READS_IN_FLIGHT`) and tracks on a player first.
  - `Command::SetDuration` and `Track::needs_header_duration` (`fp-model`): stored only for a track that is not analysed, finite and positive; the analysis replaces it, and an analysis with no length keeps it.
  - `view::time_text` shows the total as `placeholder-none` ("—") while it is unknown.
- **Q8.**
  - `PlayerState::pending_start` and `fp-model/src/pending_start.rs` (`pending_start_at`, `settle`, `start_request`, `preload_request`). A start at or past the end of the play range is not stored; one before the cue-in becomes the cue-in.
  - Every `SetNext` clears it, `MoveEntry` and `Stop` clear it, and `settle` runs in `reconcile`.
  - `PlayerSession::pending_start` is saved and loaded leniently: a broken or stale value is dropped without a log line.
  - `view::player_view` shows it as the stopped player's position, and `player.rs::wave` is always seekable; the zoom of a stopped player still does not follow.
  - The remote `seek` on a stopped player (`api.rs`, `O::Seek`) targets the next entry's cue range; `400` for a start at or past its cue-out and `409` when there is no next or its file is missing or unreadable, the cases where rule 3a stores nothing.
  - The engine is unchanged: `take_or_open` already matches a preload on `(entry, start_secs)`, and an engine test pins it.

Rulings made while implementing:

- Q6: the matrix lets 0.7 s pass (`settle`) before each gesture. Without it egui reads a double-click as a triple click after a click elsewhere within 0.6 s, a test-time artefact unrelated to the CUE. Cost if wrong: hides a real fast-operator case.
- Q6 [ASK]: not reproduced; no product change shipped. Cost if wrong: Q6 stays open until the maintainer's steps arrive.
- Q6 [ASK], after the maintainer's steps: still not reproduced, so only the regression test is committed. Whether a click in another player's table that shows the same playlist should move this player's CUE is a design question for the maintainer. Cost if wrong: Q6 stays open.
  - Answered: no. Only a click in the player's own table moves its CUE; another player's table, even on the same playlist, does not (feedback 2, O17). Q6 is closed.
- Q6 side finding [ASK]: a double-click less than 0.6 s after a click elsewhere is a triple click to egui, so `SetNext` is not sent (the CUE still follows via the single click). Left as is; treating `triple_clicked()` like `double_clicked()` would send `SetNext` twice on a real triple click.
  - Answered after plan 5 (as built): the maintainer wants a triple click on a playlist row to act like a double-click. `table.rs` sends `SetNext` for a click egui counts as a double or a triple click, once per burst of clicks: `set_next_on_click` remembers the row it set (`ViewState.set_next_sent`), a single click or a click on another row starts a new burst, so a real triple click (double, then triple to egui) sends it once. The track table is the only place in the interface where a double-click is an action (no other widget checks `double_clicked`), so nothing else changes. Tests: `fp-app/tests/row_double_click.rs` (a click on one row, then within 0.6 s a double-click on another, sets that row as next; a real triple click and a plain double-click each send one `SetNext`). The `settle` in the Q6 matrix stays: it tests the gestures apart, as before. A unit test in `table.rs` pins `set_next_on_click` (once per burst, a new row in the same burst, a reset on a single click).
  - Ruling (accepted): egui counts a click as a triple when it lands within 6 px of the previous click and the click before that was within 0.6 s, so a slow second click right after a quick earlier click can set a row as next — two clicks on one row in quick succession are close to a double-click — cost if wrong: an occasional unintended `SetNext`, undone by setting another row.
- Task 1's tests are committed alone, as regression tests, because there is no fix to commit with them.
- Q1: the FLAC test fixture carries one valid frame header after STREAMINFO, because symphonia 0.6.1's FLAC reader resyncs to the first frame when it opens a file.
- Q1: `Track::needs_header_duration` tests `is_finite() && > 0.0` instead of `!(x > 0.0)` (clippy's `neg_cmp_op_on_partial_ord`); an infinite length counts as no length.
- Q1: `header_reads_sent` is always counted; only its accessor is behind `test-hooks`.
- Q1: the header pass reads only tracks whose file state is `Ok`; a missing or unreadable track is read when it is found again.
- Q8: the engine test's opener is 10 s long (480 000 frames), not 2 s, so the control pass has something to play.
- Q8: `settle` zips the kept values back with `iter_mut` instead of indexing.
- Q8: `editing.rs::seek_is_clamped_and_ignored_when_idle` became `seek_is_clamped_and_starts_nothing_when_stopped`; it now asserts only that no playback action is sent, because rule 3a sends the preload at the pending start (Q8.7).
- Q8.8 [ASK]: a broken or stale `pending_start` in `session.json` is dropped silently, with no log line, like the other lenient session fields (fp-model is pure and `restore` has no warning channel). Cost if wrong: a small follow-up in fp-store.
- Q8: the remote seek error for no next is "nothing to seek" (was "nothing is playing"), since it now also covers a stopped player.

---

## 4. Plan 3 — Shared waveform panel

### Q7. The CUE window's waveform lacks zoom and markers

- **Problem.** The CUE window's seek bar cannot zoom, does not show intro,
  outro and MIX, and cannot edit markers, while the player's waveform can.
- **Root cause.** `widgets::waveform` (`widgets.rs` ~L1129–1348) and
  `WaveView` (`ui/wave_view.rs`) are shared, but the rest is player-only in
  `player.rs`:
  - `wave()` (~L849–1065): the zoom kept in `ViewState.wave_zoom:
    HashMap<PlayerId, WaveZoom>` (`app.rs` ~L133), following after a grace,
    wheel zoom and pan, the Full view button, the intro and outro badges;
  - `edit_markers()` (~L1397–1557): the right-click menu, Alt-drag and
    `ViewState.marker_drag` (`app.rs` ~L135).

  The marker commands `SetMarker { track, .. }` and `ResetMarkers { track }`
  act on a track, not on a player; only the seek differs (`Seek` against
  `SeekCue`).
- **Rules.**
  - **Q7.1** A new `ui/wave_panel.rs` holds that logic behind an input struct:
    `key: WaveKey` (`Player(PlayerId)` or `Cue(PlayerId)`), the entry, the
    track, the media, the total, the markers, the position, `mix_active`,
    `follow`, the optional badges, `editable_markers` and the
    height. Its output is `{ seek: Option<f64> }`; the caller maps it to
    `Seek` or `SeekCue`.
  - **Q7.2** `ViewState.wave_zoom` and `ViewState.marker_drag` are keyed by
    `WaveKey`, so the player's and its CUE's zoom and drags are independent.
  - **Q7.3** The player and the CUE window are thin callers; the CUE window
    receives `&mut ViewState`.
  - **Q7.4** The CUE window shows the intro, outro and MIX markers and edits
    them (right-click menu, Alt-drag) with the same look and commands as the
    player; it zooms, pans and has the Full view button.
  - **Q7.5** A CUE plays the whole file, so its waveform shows cue-in and
    cue-out dimmed (`cue_edge_look`) without the head and tail shading, and
    its intro and outro badges count against the end of the file.
  - **Q7.6** A marker edited in either place shows at once in both.
- **Implementation notes.** A pure `view::marker_fractions(..)` is extracted
  from `player_view` and unit-tested; both callers use it.
- **Spec lines that change.**
  - Feedback 2, O12: "the waveform with the CUE position, where a click
    seeks the CUE;" becomes "the waveform with the CUE position, the same
    panel as the player's (zoom, pan, intro, outro and MIX markers and their
    editing; operator feedback 4, Q7), where a click seeks the CUE;".
  - Main spec §8.3, Waveform: add the bullet "the CUE window uses the same
    waveform panel (operator feedback 4, Q7)."
- **Tests.** Unit tests of `marker_fractions` and of the `WaveKey`-keyed zoom
  state; kittests in `tests/cue_window.rs`: zoom by wheel, a click sends
  `SeekCue`, Alt-drag of the intro sends `SetMarker`; the player's existing
  waveform tests (`waveform_ui.rs`, `markers_ui.rs`) pass unchanged.
- **Docs and locales.** Existing marker strings are reused.
  `docs/user/players.md` (CUE window), `docs/user/markers-and-mixing.md`,
  `docs/technical/ui.md`.
- **As built (plan 3).** `ui/wave_panel.rs` (`WavePanelInput`, `WaveBadges`,
  `WavePanelOutput`); `wave_view::{WaveKey, WaveZooms, wheel_notches,
  WaveView::wheel}`; `view::{marker_fractions, intro_left, outro_left}`;
  `CueWindowView::{markers, intro, outro, mix_active}`. `ViewState::wave_menu`
  is keyed by `WaveKey` too. The CUE's intro badge never blinks, its MIX
  marker follows the player's mode, its zoom follows its position while it plays, and a
  closed CUE window forgets its zoom, menu point and marker drag.
  - `seekable` is not in the panel's input: every waveform seeks since plan 2
    (a stopped player's click is rule 3a's pending start), so the switch was
    dead and `WaveInput::seekable` went with it.
  - The wheel rules are the pure `wheel_notches` and `WaveView::wheel`; each
    `WaveKey` has its own egui id, so the memoised columns and the pan-drag
    flag of a player and its CUE never mix.
  - The panel sends its seek after its marker commands, in the same frame.

---

## 5. Plan 4 — Meter rulers

### Q11. Meter: nothing over the bars, a ruler on each side

- **Problem.** The reference lines across the bars read as a signal fault
  and clutter the meter; the scale is only on the left.
- **Root cause.** In `widgets.rs`, `vu()` (~L475–600) and `meter_layout`
  (~L725–850) put the legend in a left label column only;
  `reference_segments` (~L667) paints thin lines across the bars and the gap;
  the alignment level is a 2 px line plus 3 px notches at the outer edges in
  `NEUTRAL_400`. The tests are in `crates/fp-app/tests/meter_view.rs`.
- **Rules.**
  - **Q11.1** Nothing is drawn inside or across the level bars or the gap
    between them; the bars show only the level, the peak hold and the zones.
  - **Q11.2** The scale is a ruler on each side of the bars, with the labels
    on both sides.
  - **Q11.3** Every labelled mark has a tick on both rulers.
  - **Q11.4** Minor ticks sit between the labelled marks, as on a measuring
    ruler, with a spacing chosen per scale (for example every 1 dB on EBU
    PPM, every 5 dB on the digital scale below −20), and only where they stay
    at least 3 px apart.
  - **Q11.5** The alignment (reference) level is a thicker tick, always
    white, on both rulers, never inside a bar.
  - **Q11.6** The label crowding rules of M4 (both ends first, then top down
    10 px apart) still hold on each ruler.
  - **Q11.7** The meter grows by one label column; the player column's
    layout makes room for it at the minimum width (380 px).
- **Spec lines that change** (meters spec M4).
  - "labelled on the left in the meter's own units (monospace, `NEUTRAL_400`),
    with a 1 px reference line at 35 % across both bars for every label.
    Nothing is drawn in the gap between the channels." becomes "labelled on
    rulers on both sides of the bars in the meter's own units (monospace,
    `NEUTRAL_400`): a tick for every label and minor ticks between them at a
    spacing per scale. Nothing is drawn over the bars or in the gap between
    them (operator feedback 4, Q11)."
  - "Alignment level: two short notches (3 × 2 px, `NEUTRAL_400`) at the
    outer edges of the bars, over a faint 1 px reference line, at
    `reference_dbfs`" becomes "Alignment level: a thicker white tick on both
    rulers at `reference_dbfs`".
  - "Legibility: lines stay inside the bars." becomes "Legibility: nothing is
    drawn over the bars."; "A mark without a label gets no line." becomes "A
    mark without a label gets a minor tick."
  - Main spec §8.3, Meter column: "labelled dB scale, stereo meter with
    reference lines and peak hold" becomes "stereo meter between two
    labelled rulers, with peak hold".
  - Feedback 2, O13 is superseded: add "Operator feedback 4, Q11, removes the
    reference lines over the bars."
- **Tests.** In `tests/meter_view.rs`, the line tests become "nothing is
  drawn over the bars" (no shape intersects the bar rects except the level,
  the peak hold and the zones), plus ticks on both sides, the white
  alignment tick, minor tick spacing per scale and the crowding rules at 64,
  136 and 300 px; `tests/theme.rs` drops `METER_LINE_*` and pins the new tick
  colours; a geometry test of the player column at 380 px.
- **Docs and locales.** No strings. `docs/user/players.md` (meters),
  `docs/technical/ui.md`; the guide screenshots are refreshed with
  `scripts/site/screenshots.sh`.
- **As built (plan 4).**
  - `meter_layout` returns two `Ruler`s and the `ticks`. Per side: a label column as wide as the scale's widest label (`label_column`: 15 px for three characters, 20 px for EBU's `TEST`, at the monospace font's advance), a 3 px clear gap, so a label's minus never reads as part of a tick, and a 4 px tick strip flush with its bar. `meter_width` is 74 px on every scale but EBU (84 px); it was 52 px with one label column. `vu` paints only the bars' own content plus the rulers; `reference_segments`, `LineShade`, the notches and the `METER_LINE_*` constants are gone.
  - Minor ticks are ruled like a measuring ruler. A scale mark that lost its label is a minor tick where it stays 3 px from the ticks above. Each segment between two of those ticks (labels, the alignment level, unlabelled marks) takes the finest step of a 0.5, 1, 2, 2.5, 5, 10 dB ladder that is not finer than the scale's own spacing and keeps every tick 3 px apart, and draws every multiple of that step in the scale's units strictly inside the segment, or nothing. The scale's own spacing (`minor_marks`, the finest ticks): digital and custom 5 dB below −20 dBFS and 1 dB above; EBU 1 dB; DIN 5 dB below −20 relative to its 0, then 1 dB; VU 5 dB below −10, 1 dB to −3, 0.5 dB above; K-System 5 dB below −24, then 1 dB. A custom floor of −57 dBFS gets its first minor tick at −55, not at −52.
  - Ticks: major `NEUTRAL_400`, minor the same at 60 %, alignment white and 2 px thick (`METER_TICK`, `METER_TICK_MINOR_ALPHA`, `METER_ALIGNMENT_TICK`).
  - The wider meter takes width from the transport row, so the countdown in a narrow player is smaller than before the rulers: measured with a 3:25 time in the harness, its height at windows 380, 420 and 460 px wide is 24, 26 and 39 px (30, 33 and 45 px with one label column). In the default layout (four players in 1920 × 1080) it keeps its full size, 46 px, which a test pins. `METER_COLUMN_GAP` (the gap between the transport grid and the meter column, `ui/player.rs`) went from 10 to 6 px, and `COUNTDOWN_MIN_SCALE` from 18/38 to 13/38, so a ten-hour countdown fits at the minimum width beside the widest meter, EBU's, at 13 px.

---

## 6. Plan 5 — Outputs settings

### Q12. Basic and Advanced outputs, per-device rate and buffer

- **Problem.** Every output setting shows at once, and the rate and the
  buffer are global, while different devices want different values.
- **Code today.** `OutputsConfig` (`crates/fp-model/src/config.rs` ~L320–345)
  has a global backend, `sample_rate` and `buffer_frames`, the routes per
  player and the cartwall, `bit_perfect: Vec<OutputDevice>`, `dsd_output:
  Vec<DsdDevice>` (`Pcm`, `Dop` or `Native` per device, used only when the
  device is bit-perfect), and the global `dsd_mix` and `dsd_silence_ms`.
  Buses are keyed per device (`BusKey { backend, device }`,
  `crates/fp-engine/src/bus.rs` ~L25; `ensure_bus`, `engine.rs` ~L981). Routes
  on one device share one stream, so a rate or buffer per route is
  impossible; per device it is natural. `dsd_silence_ms` has no UI.
- **Rules.**
  - **Q12.1** Settings → Outputs has a **Basic | Advanced** selector,
    persisted in `ui.outputs_view` (`Basic` by default, lenient loading).
  - **Q12.2** Basic shows the audio system, the sample rate, the buffer size
    and the routes.
  - **Q12.3** Advanced adds, per device used by a route:
    - an optional sample rate and an optional buffer size, which override the
      global ones for that device;
    - the bit-perfect toggle;
    - the DSD playback mode (Q3);
    - and, once for all devices, `dsd_mix` and `dsd_silence_ms`.
  - **Q12.4** The overrides are a new `outputs.device_overrides:
    Vec<DeviceOverride { device: OutputDevice, sample_rate: Option<u32>,
    buffer_frames: Option<u32> }>`, keyed by backend and device like
    `bit_perfect`, validated with the same ranges as the global fields in
    `Config::validate`; an out-of-range value is dropped with a log line, so
    the device falls back to the global value.
  - **Q12.5** `EngineSettings` carries the overrides; `ensure_bus` opens a
    device at its own rate and buffer when it has them. A bit-perfect bus
    starts at that rate and follows the file as before (bit-perfect spec B3).
  - **Q12.6** Changing an override needs a restart, like the global values
    (`RestartReason`).
  - **Q12.7** Switching to Basic never changes the configuration: it only
    hides the advanced rows. The defaults are no bit-perfect device and DSD
    converted to PCM.
- **Spec lines that change.**
  - Main spec §8.4, Audio outputs: "sample rate and buffer size, with the
    computed latency;" becomes "sample rate and buffer size, with the
    computed latency; a Basic | Advanced selector, where Advanced adds a
    rate and buffer per device, bit-perfect, the DSD mode and the DSD
    settings (operator feedback 4, Q12);".
  - Bit-perfect spec B3 rule 1: "A normal bus runs at `outputs.sample_rate`.
    A bit-perfect bus starts at `outputs.sample_rate`" becomes "A normal bus
    runs at its device's rate override, or `outputs.sample_rate`. A
    bit-perfect bus starts at that same rate".
  - Bit-perfect spec B6: "**Settings → Audio outputs** lists the devices the
    routes use, each with a **Bit-perfect** checkbox." becomes "**Settings →
    Audio outputs**, in its Advanced view (operator feedback 4, Q12), lists
    the devices the routes use, each with a **Bit-perfect** checkbox, its own
    rate and buffer, and its DSD mode."
- **Tests.** `fp-model`: validation and lenient loading of `device_overrides`
  and `ui.outputs_view`, `restart_pending` for an override change; `fp-engine`:
  `ensure_bus` opens at the override (Offline backend); kittests in
  `tests/settings.rs` and `tests/dsd_ui.rs`: Basic hides the rows, switching
  views leaves `Config` unchanged, an override edit sends `UpdateConfig`.
- **Docs and locales.** New keys in en-US and es-ES (`settings-outputs-basic`,
  `settings-outputs-advanced`, `settings-device-rate`,
  `settings-device-buffer`, `settings-dsd-silence`, the "use global" choice);
  `docs/user/settings.md`, `docs/user/bit-perfect.md`,
  `docs/technical/backends.md`, `docs/technical/audio-engine.md`.

### Q3. The DSD playback mode is hard to find

- **Problem.** The maintainer could not find where to choose native DSD or
  DoP.
- **Root cause.** The selector exists (`crates/fp-app/src/ui/settings.rs`
  `bit_perfect()` ~L867–1015, `offered_dsd_modes` ~L1016), but it appears only
  once the device is routed and bit-perfect is on, with no explanation.
- **Rules.**
  - **Q3.1** In the Advanced view, every routed device shows its DSD row,
    even when only PCM is offered.
  - **Q3.2** When DoP or Native is not offered, a line under the row says
    why: bit-perfect is off; the device is not exclusive-capable; native DSD
    needs Linux; the device does not take native DSD.
  - **Q3.3** The row states that the default is DSD converted to PCM.
  - **Q3.4** The reasons come from a pure function next to
    `offered_dsd_modes`, one test per reason.
- **Spec lines that change.** Feedback 2, O25: "The setting sits next to the
  device's bit-perfect switch, and only the modes the device can take are
  offered." becomes "The setting sits next to the device's bit-perfect
  switch in the Advanced outputs view; only the modes the device can take
  are offered, and a line says why the others are not (operator feedback 4,
  Q3)."
- **Tests.** Unit tests of the reason function; a kittest in
  `tests/dsd_ui.rs` that a non-bit-perfect device shows the row and its
  reason.
- **Docs and locales.** The reason strings and `settings-dsd-default-pcm` in
  en-US and es-ES; `docs/user/bit-perfect.md`.

**As built (plan 5).** The rulings are in
`docs/superpowers/plans/2026-10-06-feedback4-plan5-outputs-settings.md`.
In short:
- an override change reuses the SampleRate and BufferSize restart reasons
  (the notice does not say which device changed);
- the DSD reasons and the offered modes live in `fp_model::device_offer`,
  with at most one reason, the most fundamental first;
- an unplugged device has its own reason (`NotConnected`), and a DSD mode
  the device cannot take shows as PCM without rewriting the configuration
  (`effective_dsd_mode`);
- an override reaches the engine only for a device a route names; the
  system-default output opens at the global values. (Changed after plan 5,
  see "Maintainer decisions after plan 5" below: a stale override no
  longer stays in the configuration.)
- a saved own rate or buffer the device no longer reports stays listed with
  a note, and a bus whose own rate does not open falls back to the global
  rate and buffer through `pcm_fallback` (and, after plan 5, so does a
  bus whose own buffer does not open; see below);
- `advanced_in_use` counts the DSD mix and silence only with a routed device;
- the Advanced rows list the devices that routes name (`routed_devices`);
  a player on the system default output gets no rows;
- a device's own value equal to the global one stays the device's own, and
  only **Global (...)** removes it;
- switching views changes `ui.outputs_view` only;
- the DSD mode box is disabled when PCM is the only mode offered;
- the engine opens each device at its own rate and buffer, and a
  bit-perfect device starts at its own rate and still follows each file.

Three choices are pending the maintainer's confirmation (each is easy to
reverse):
- **[ASK] Rates and buffers offered.** A device's own rate and buffer list
  only the values the device reports (all when it reports nothing or is
  unplugged), and a chosen value always stays listed. A device that
  under-reports cannot be given a rate it does take, except in
  `config.json`. Reversing it is a change to `offered_rates` and
  `offered_buffers`.
- **[ASK] The Basic hint line.** Basic shows one line when a routed device is
  bit-perfect or has its own rate or buffer, or when the DSD mix or silence
  is not the default (`OutputsConfig::advanced_in_use`), so that an upgraded
  installation does not hide active settings. Reversing it is dropping the
  line.
- **[ASK] The heading.** "Bit-perfect devices" became "Per-device settings"
  (`settings-devices`; `settings-devices-none` replaces
  `settings-bit-perfect-none`), because the section now holds the rate and
  buffer too.

Task 7, the check on real hardware, is not done.

**Maintainer decisions after plan 5 (as built).**
- **A device that leaves every route loses its own settings.** The
  maintainer: there is no need to keep the configuration of a device that
  may never be connected again. `OutputsConfig::forget_unrouted_devices`
  (`fp-model`) removes the `device_overrides`, `bit_perfect` and
  `dsd_output` entries of every device no route names (players' Main and
  Cue, the cartwall's Main and Cue). Only `fp-store` applies it, on load,
  so the settings are forgotten the next time the application starts (a
  stale entry written by an earlier version goes too). Not in the reducer:
  Settings applies every route click at once, so swapping two devices
  between players, or going to the system default and back, leaves a
  device unrouted for a moment, and its settings must survive that; a
  change of outputs needs a restart anyway. Before pruning, the load drops
  the routes of players that no longer exist
  (`OutputsConfig::forget_routes_of_other_players`; a removed player's id
  is never used again, and its routes would keep its devices routed), but
  only when the session file loaded, since without it the players' ids are
  not known. A device routed again after a start starts from the global
  values. Settings shows the bit-perfect switch and
  the DSD mode only next to a routed device, like the own rate and buffer,
  so they follow the same rule. The removal is silent, like the removal of
  an empty override. The system-default output is untouched: it has no
  entry, and opens at the global values as before.
  - Ruling: no warning when an unrouted device's entry is removed — the
    existing pattern removes an empty override silently, and the operator
    caused it by changing the route — cost if wrong: a one-line warning in
    `Store::load`.
  - Ruling: pruning only on load, not in `update_config` — a route edit
    in Settings (a swap, the system default and back) must not lose a
    device's settings, and the restart notice clears when the routes
    return to the started ones — cost if wrong: a device unrouted during
    the session keeps its hidden settings until the next start.
  - Ruling: a removed player's routes are dropped on load only when the
    session file loaded — without it the restored players get new ids and
    every route would look stale — cost if wrong: stale routes (and their
    devices' settings) stay until a start with a session.
- **A refused own buffer falls back to the global buffer**, like a refused
  own rate. `ensure_bus` gives a bus the global rate and buffer as its
  `pcm_fallback` when its device has its own rate or its own buffer (before,
  only its own rate), and `Bus::open_pcm_fallback` uses it when it differs
  in the rate or in the buffer. When the device does not take its own
  values, the watchdog reopens it with the global ones: the global buffer
  alone when only the buffer was its own, both when both were. The log is
  the same warn line as the rate fallback's (one for the buffer, which says
  so); a buffer change alone does not open the sources again, since their
  timelines are in the same rate. As before, there is no on-screen notice.
  Tests: `fp-engine/tests/device_overrides.rs`, with
  `OfflineDevice::refuse_buffer`.
  cpal never refuses a buffer size: a fixed size outside the device's
  reported range was skipped and the device opened with its default buffer,
  so the fallback never ran. A bus with its own buffer now asks for it
  exactly (`StreamConfig::exact_buffer`): cpal then tries only that fixed
  size, and refuses it with `Unsupported` when the device reports ranges
  that leave it out. The fallback (global rate and buffer) is not exact, so
  the global path keeps its behaviour (outside the range, the device's
  default). `Engine::buffer_of` uses the buffer the stream runs with
  (`Bus::buffer_frames`), not the one asked for. The Offline backend models
  cpal: with a reported range, a size outside it opens with the device's
  default (`set_default_buffer`) unless asked for exactly.
  - Ruling: the strictness is a field of the request
    (`StreamConfig::exact_buffer`), not a reopen after the engine compares
    buffers — cpal reports the requested size until the first callback, so
    a mismatch is not known at open, and a refusal reuses the existing
    `pcm_fallback` path — cost if wrong: one more field every backend sees.
  - Ruling: a DSD stream is never exact, as before — refusing a DSD stream
    for its buffer would lose the DSD — cost if wrong: DoP may run at the
    device's default buffer.
  - Ruling: a rate the bus follows (bit-perfect) keeps its own buffer
    exactly; a device that takes the rate only with another buffer refuses
    the rate (the engine resamples), an edge case since devices report the
    same buffer range at every rate — cost if wrong: a resampled file on
    such a device.
  The fallback runs only when the device refuses the configuration
  (`BackendError::Unsupported`, `Bus::open_or_fall_back`): a device missing
  or busy for a moment (a USB replug, another client) is asked for its own
  values again at the next retry rather than running at the global ones for
  the rest of the session. And a bit-perfect bus that follows a file's rate
  (`reopen_with`, PCM to PCM) keeps, for a device with its own buffer, a
  fallback at the new rate with the global buffer; a fallback to another
  rate is still forgotten there.

---

## 7. Plans

1. **Plan 1 — UI fixes** (Q2, Q4, Q5, Q9, Q10). Independent UI changes plus
   one service rule. Order: Q5 (the shared blink helper), Q2, Q9, Q4, Q10.
   `docs/superpowers/plans/2026-10-06-feedback4-plan1-ui-fixes.md`.
2. **Plan 2 — Play before analysis, start point, CUE follow** (Q1, Q8, Q6).
   Model first: `SetDuration` and `pending_start` with one reducer test per
   rule, then the view and the UI; Q6 starts with the failing kittest.
   `…-feedback4-plan2-play-before-analysis.md`.
3. **Plan 3 — Shared waveform panel** (Q7). After plan 2. Extract
   `wave_panel.rs` with the player as its only caller and its tests green,
   then key the view state by `WaveKey`, then move the CUE window onto it.
   `…-feedback4-plan3-wave-panel.md`.
4. **Plan 4 — Meter rulers** (Q11). Layout first (`meter_layout` with two
   rulers), then painting, then the player column.
   `…-feedback4-plan4-meter-rulers.md`.
5. **Plan 5 — Outputs settings** (Q12, Q3). Model (`device_overrides`,
   `ui.outputs_view`), engine (`EngineSettings`, `ensure_bus`), then the
   Settings view and the DSD reasons.
   `…-feedback4-plan5-outputs-settings.md`.

Each plan follows CLAUDE.md: test-driven, fmt, clippy and the whole suite
green before each commit, a docs task, a review by a fresh reviewer, and its
own pull request. Each plan adds its "As built" notes to its section here.

## 8. Out of scope

- **Q14, multichannel (5.1) playout.** Deferred to its own spec. The impact
  analysis found:
  - stereo is assumed in the decoder's downmix, the source ring
    (`SOURCE_CHANNELS = 2`), the resampler, the mixer, the meters and the
    loudness measurement, and the UI;
  - multichannel devices are used today as stereo pairs
    (`Route.first_channel`), so the choice cannot be automatic: it must be per
    route, a channel layout plus a channel map, because the ALSA hardware
    order differs from the WAV order;
  - the estimate is two to four weeks.
- A progressive waveform drawn while the analysis runs (Q1 shows the
  baseline and the playhead only).
- Bit-perfect rate changes at transitions (parked; the maintainer has not
  decided).
- Detecting, in Settings, a card used both through `hw:` and through the
  sound server.

## 9. Global constraints

`CLAUDE.md` rules 1–10 apply to every plan. In particular:

- English everywhere, and UI strings in both locales;
- no product names;
- operator values are `Config` fields with defaults, ranges and lenient
  loading (Q12's overrides and view);
- the real-time path never allocates, locks, logs or panics;
- behaviour lives in `fp-model` (`SetDuration`, `pending_start`, the DSD
  reasons);
- the UI never blocks (the header read and the file stat run on helper
  threads);
- bad data never crashes (a header without a duration, a file that changes
  while it is checked);
- nothing goes on air by itself (a pending start only changes where Play
  starts).
