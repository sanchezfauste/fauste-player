# Operator Feedback — Roadmap of Plans

**Spec:** [`docs/superpowers/specs/2026-09-30-operator-feedback-design.md`](../specs/2026-09-30-operator-feedback-design.md)

This roadmap splits the spec into eight plans. Each plan is executed in its
own conversation and reaches `master` through its own pull request (see
`CLAUDE.md` → Pull requests).

Plan 1 is written in full in
[`2026-09-30-feedback-plan1-interface-polish.md`](2026-09-30-feedback-plan1-interface-polish.md).
Plans 2–8 are outlined below. Each is written in full with
`superpowers:writing-plans` at the start of its own conversation, against the
code as the earlier plans left it. Writing them now would describe code that
is about to change.

## How to run a plan in a new conversation

1. Start from an up-to-date `master`, with the previous plan merged.
2. Prompt: *"Write plan N of `docs/superpowers/plans/2026-09-30-operator-feedback-roadmap.md`
   with superpowers:writing-plans, then execute it."* (For plan 1, which is already
   written: *"Execute `docs/superpowers/plans/2026-09-30-feedback-plan1-interface-polish.md`."*)
3. The plan file is `docs/superpowers/plans/2026-09-30-feedback-planN-<topic>.md`
   (keep the date of this roadmap so the series sorts together).
4. The ledger is `.superpowers/sdd/feedback-planN/progress.md`.
5. At the end, follow `superpowers:requesting-code-review` and then
   `superpowers:finishing-a-development-branch`.

## Global constraints (every plan)

- `CLAUDE.md` rules 1–10 apply. In particular:
  - English everywhere, and UI strings in both locales;
  - no product names;
  - every operator value is a `Config` field with a default, a range and
    lenient loading;
  - real-time code never allocates, locks, logs or panics;
  - behaviour lives in `fp-model`;
  - the UI never blocks;
  - bad data never crashes;
  - nothing goes on air by itself.
- TDD: every new test is seen failing first.
- Commit only with `cargo fmt --all --check`, `cargo clippy --workspace
  --all-targets -- -D warnings` and `cargo test --workspace` green.
- Each plan's last task syncs the docs: README, `docs/user`,
  `docs/technical`, the main spec (§3, §6 or §8 as touched) and this
  feedback spec when a ruling changes it.

---

## Plan 1 — Interface polish (F1, F8, F10, F14, F19, F21)

Written in full: see the plan file. The tasks are:
1. fixed-width times;
2. output device labels;
3. segmented SINGLE|CONT control;
4. "slate" as the default waveform colour;
5. the "play then stop" icon;
6. the version in the top bar and the About window;
7. docs and verification.

---

## Plan 2 — Meter column (F7, F9, F11)

**Branch:** `feat/meter-column`. **Spec:** §3.2. Depends on plan 1 (layout of
the header and icons).

1. **Muted meter colours.**
   - What: replace `VU_GREEN`, `VU_YELLOW` and `VU_RED` in `ui/theme.rs` with
     `METER_NORMAL` #7fb08a, `METER_WARNING` #d9b45a and `METER_DANGER`
     #d8646a, and update every use in `ui/widgets.rs` (zones, peak hold, max
     readout, loudness on-target).
   - Test: `tests/theme.rs` checks the exact values.
2. **Meter geometry as a pure function.**
   - What: `widgets::meter_layout(rect: Rect, c: &MeterConfig) -> MeterLayout`,
     with the label column, two adjacent bar rects, a max-readout rect, an
     optional loudness rect, and one `(y, label, is_alignment)` per drawn
     mark. Labels come from `scale_marks` and are dropped when they would
     overlap (reuse `mark_rows`' spacing, with the label's text height as the
     gap).
   - Tests (`tests/meter_view.rs`):
     - no two labels overlap at 64 px and at 136 px;
     - every line spans both bars;
     - nothing is drawn between the bars;
     - the alignment mark is kept.
3. **Draw the new meter.**
   - What: `widgets::vu` draws from `MeterLayout`: labels in monospace
     `NEUTRAL_400`, 1 px lines across both bars in `NEUTRAL_400` at 35 %, and
     no marks in the gap (F7).
   - Tests: existing meter tests updated, and a kittest that the max readout
     still resets on click.
4. **Right-hand column in the player.**
   - What: in `ui/player.rs`, the info and transport rows sit left of a right
     column that holds the meter and fader and spans both rows (mockup C).
     `elapsed / total` moves under the transport, right-aligned, using the
     plan 1 tabular helper. The 2×2 transport grid is unchanged (plan 3
     changes it). The column keeps working at the minimum player width
     (380 px).
   - Tests: kittest geometry. The meter's rect spans the Play button's
     vertical range and the cover's, and lies right of the countdown. The
     fader is right of the meter. The waveform's width is unchanged.
5. **Docs.**
   - What: update the meters spec M4 (layout, colours, lines), main spec §8.3
     (info row), `docs/user/players.md` and `docs/technical/ui.md`.

---

## Plan 3 — Restart, Previous and button availability (F3)

**Branch:** `feat/restart-previous`. **Spec:** §2.1, §2.2 (R23–R25, R28), and
the transport grid part of §3.2.

1. **History.**
   - What: `PlayerState.history`, and `players.history_len` (default 50, range
     0–1000) in `config.rs` with validation. Persist it in `session.json`
     (`fp-model/src/session.rs`, `fp-store`), leniently. R25: every rule-12
     advance pushes the outgoing current, capped at the configured length.
   - Tests (`fp-model/tests/transport.rs`):
     - one per advance path;
     - the cap;
     - a session round trip;
     - a corrupt history loads empty.
2. **R23 Restart.**
   - What: `Command::Restart` emits `EngineAction::Seek` to `cue_in` when not
     Stopped.
   - Tests: Playing, Paused (stays paused), Stopped (no action), no current.
3. **R24 Previous.**
   - What: `Command::Previous` pops the history (skipping gone or unplayable
     entries), crossfades like rule 5, marks the old current played without
     pushing it, and sets `next` to the old current (explicit).
   - Tests:
     - happy path;
     - repeated Previous goes further back;
     - stale entries skipped;
     - empty history does nothing;
     - fading, paused or stopped does nothing.
4. **R28 Availability.**
   - What: `fp_model::availability(&AppState, PlayerId) -> Availability`, a
     `Copy` struct with one `bool` per action (`play`, `pause`, `stop`,
     `fade_stop`, `restart`, `previous`, `stop_after_current`, `cue`).
   - Tests: one per row of the spec's table, with both the true and the false
     case.
5. **Shortcuts.**
   - What: `ShortcutAction::RestartPlayer(u16)` and `PreviousPlayer(u16)`, with
     no default key. They are listed in Settings > Shortcuts and mapped in
     `app.rs::shortcut_command`. Unavailable actions are ignored.
   - Tests: `fp-model/tests/shortcuts.rs`, and a kittest binding one.
6. **Engine.**
   - What: an Offline test that Previous crossfades exactly like Next (the same
     `Crossfade` path) and that Restart seeks without a click (the click
     detector of the engine tests).
7. **UI.**
   - What: `icons::restart` (bar + one triangle) and `icons::previous` (bar +
     two triangles). The transport grid becomes 3×2 (Previous and Restart
     first), with tooltips in both locales. Every transport button, including
     CUE and Stop after current, gets `enabled` from `availability`.
   - Tests: kittest that each new button sends its command, and that the
     buttons are dimmed and inert when unavailable (stopped player).
8. **Docs.**
   - What: main spec §3 (R23–R25, R28), `docs/user/players.md` and
     `keyboard.md`, `docs/technical/ui.md`.

---

## Plan 4 — Real-music corpus and marker tuning (F12, F16)

**Branch:** `feat/marker-tuning`. **Spec:** §4.

1. **Corpus folder.**
   - What:
     - `.gitignore` gets `/test-music/*` and `!/test-music/README.md`;
     - `test-music/README.md` explains what to put there;
     - a helper `fp-analysis/tests/support/corpus.rs` lists supported files
       from `FAUSTE_TEST_MUSIC`, else `<repo>/test-music`, and returns an
       empty list with a printed note when there is none;
     - `CLAUDE.md` (Commands and Testing notes) and
       `docs/technical/testing.md` document it.
2. **`marker_report` example.**
   - What: `crates/fp-analysis/examples/marker_report.rs` analyses each corpus
     file with the current `AnalysisSettings` and prints a table: file,
     duration, cue_in, cue_out, segue, overlap, outro. It accepts
     `--set key=value` overrides of analysis settings for tuning.
3. **Trim by peak.**
   - What: `analysis.trim_threshold_db` (default −60, range −120…−20) and
     `analysis.trim_margin_ms` (default 20, range 0–1000) in `config.rs`.
     `silence_threshold_db` is removed, and a stored value is ignored with a
     log line. `detect_markers` computes `cue_in` and `cue_out` from bucket
     peaks with the margin.
   - Tests (`signal.rs` unit tests and `fp-analysis/tests`):
     - the invariant "no bucket with peak ≥ threshold outside `[cue_in,
       cue_out]`" on a soft fade-in at −50 dBFS, a −45 dBFS tail and a click
       at −30 dBFS in the head;
     - a silent file keeps the whole duration.
4. **Relative segue.**
   - What: `analysis.segue_drop_db` (default 15, range 3–40) replaces
     `segue_threshold_db`. `segue_max_secs` defaults to 4.
   - Tests: the same fade-out tail on a loud master (−8 dBFS RMS body) and a
     quiet one (−20 dBFS RMS body) gives the same overlap. The overlap never
     exceeds `segue_max_secs`.
5. **Cache and Settings.**
   - What: bump the analysis version (the cache key) so automatic markers are
     recomputed, and keep manual markers (test). Settings > Analysis shows the
     new fields, in both locales. `Config::validate` tests cover the ranges.
6. **Corpus tests and tuning.**
   - What: `fp-analysis/tests/real_music.rs` holds `#[ignore]` tests with the
     trim invariant and "overlap ≤ segue_max" for every corpus file.
   - **Checkpoint with the maintainer:** run `marker_report`, listen to a few
     transitions with the maintainer, adjust the defaults, and record
     `Ruling: defaults … — …` in the ledger.
7. **Docs.**
   - What: main spec §6, `docs/user/markers-and-mixing.md`, `settings.md`,
     `docs/technical/analysis.md`, `persistence.md`.

---

## Plan 5 — Waveform: drag, zoom and whole file (F2, F17, F20)

**Branch:** `feat/waveform-zoom`. **Spec:** §3.3. Depends on plan 4 (corpus).

1. **Investigate F20** (`superpowers:systematic-debugging`).
   - What: with a corpus file that has silent head and tail, compare
     `peaks.len() × bucket_secs`, the decoded frame count and `duration_secs`.
     Record the finding as a ruling. If a defect is found (the span misses
     part of the file), write the failing test in
     `fp-analysis/tests/analyze.rs` or `fp-app/tests/waveform_view.rs` first,
     then fix it.
2. **`WaveView` mapping.**
   - What: `widgets::WaveView { start_secs, span_secs }` with `x_of(secs)`,
     `secs_at(x)`, `zoom_at(x, factor, limits)`, `pan(dx)` and
     `follow(position)`. `wave_columns` gains a start offset and reduces only
     the visible range.
   - Tests (`tests/waveform_view.rs`):
     - the zoom keeps the time under the pointer fixed;
     - the view is clamped to `[0, total]`;
     - the deepest zoom is one bucket per pixel;
     - a full view equals today's drawing.
3. **Trimmed regions.**
   - What: draw the regions before `cue_in` and after `cue_out` dimmed, with
     1 px lines.
   - Tests: geometry test of the shaded rects from marker fractions.
4. **Drag to seek.**
   - What: press and move shows a preview line and time. Release inside sends
     `Command::Seek`; release outside or Esc cancels. A plain click still
     seeks, and Alt-drag still edits markers.
   - Tests (kittest): drag and release inside sends one `Seek` at the release
     point; release outside sends nothing; Esc cancels.
5. **Zoom.**
   - What:
     - the wheel zooms and Shift+wheel or a horizontal wheel pans;
     - the view follows the playhead unless the operator panned within
       `ui.follow_current_grace_secs` (new `Config` field, default 10, range
       0–600, validated);
     - a "Full view" button appears in the top-right corner while zoomed;
     - the view resets on track change;
     - markers, Alt handles, the hover tooltip and seeking all go through
       `WaveView`.
   - Tests (kittest):
     - wheel zoom then click seeks to the zoomed time;
     - "Full view" resets;
     - a track change resets;
     - marker drag in a zoomed view sets the right time.
6. **Docs.**
   - What: main spec §8.3 (waveform), `docs/user/players.md` and
     `markers-and-mixing.md`, `docs/technical/ui.md`, `persistence.md` (new
     field).

---

## Plan 6 — Per-entry repeat and stop (F4, F5)

**Branch:** `feat/entry-repeat-stop`. **Spec:** §2.1, §2.2 (R26, R27), §2.3
(entry flags). Depends on plan 3 (availability, history).

1. **Entry flags.**
   - What: `PlaylistEntry.repeat` and `.stop_after` (`serde(default,
     skip_serializing_if)`), `Command::ToggleEntryRepeat` and
     `ToggleEntryStopAfter`. Duplicate copies both flags.
   - Tests: a `fp-store` round trip, old files load without them, and a
     proptest that flags survive move and duplicate.
2. **R27 Stop after this entry.**
   - What: `plan_for` returns `StopAt { cue_out }` for a `stop_after` entry in
     any mode. The flag persists after Stop, and `next` stays.
   - Tests: Continuous with auto-segue, Single, flag kept, precedence over
     repeat.
3. **R26 Repeat in the model.**
   - What: a new `TransitionPlan::RepeatAt { at_secs }` (restart the current
     entry at `cue_in` when it reaches `cue_out`). `plan_for` returns it for a
     `repeat` entry unless `stop_after_current` or `stop_after` is set. A
     `TransitionStarted` for the same entry keeps `current` and neither marks
     it played nor pushes it to the history. The preload of the repeat source
     is part of the plan.
   - Tests:
     - Single and Continuous;
     - pause/resume keeps it;
     - Play (next), Previous, Stop and Fade stop end it;
     - stop-after-current ends the pass;
     - toggling while playing re-plans.
4. **Engine.**
   - What: execute `RepeatAt` with the same preload and scheduled start as a
     segue, with no overlap and no fade.
   - Tests (Offline): the restarted source starts at the exact frame after
     `cue_out`, with no gap and no click; three passes; pause across a
     boundary.
5. **UI.**
   - What: Phosphor `REPEAT` and the plan 1 stop-after icon at the right of
     the title cell, in the row's text colour. The context menu gains
     checkable "Repeat this track" and "Stop after this track", in both
     locales.
   - Tests (kittest): the menu sends the toggles, and the icons show for
     flagged entries (by accessible label).
6. **Docs.**
   - What: main spec §3 (R26, R27), `docs/user/playlists.md` and `players.md`,
     `docs/technical/audio-engine.md` and `persistence.md`.

---

## Plan 7 — Track table: columns and follow current (F6, F18)

**Branch:** `feat/table-follow`. **Spec:** §2.3. Depends on plan 6 (table
cells).

1. **Fractional columns.**
   - What: `ColumnWidths { number, title, artist, duration }` as fractions
     summing to 1, with pixel minimums for `#` and Duration derived from their
     content. Old pixel widths are discarded on load (lenient, with a log
     line).
   - Tests: model normalisation (sum to 1, NaN or negative → defaults) and a
     session round trip.
2. **Table uses fractions.**
   - What: `ui/table.rs` computes pixel widths from fractions every frame and
     stores the new fractions on handle release (`SetColumnWidths`).
   - Tests (kittest): after resizing the harness window wider, each column's
     share is unchanged and the table fills the width; a manual drag stores
     fractions.
3. **Follow current.**
   - What: in `ViewState`, per player, keep the last interaction time with
     its table and tabs (scroll, drag, menu, tab click). When `current`
     changes and no interaction happened within
     `ui.follow_current_grace_secs` (added by plan 5), send `ShowPlaylist`
     for the current's playlist if needed and scroll so the current row is
     first (`TableBuilder::scroll_to_row(i, Some(Align::TOP))`). While a drag
     or a menu is active, it waits until it ends.
   - Tests (kittest):
     - advance with no interaction scrolls (the current row is the top
       visible row);
     - a recent scroll prevents it;
     - the current in another tab switches the tab;
     - near the end of the list the table scrolls as far as it can;
     - grace 0 disables following.
4. **Docs.**
   - What: main spec §8.3 (table), `docs/user/playlists.md`,
     `docs/technical/ui.md`.

---

## Plan 8 — MIDI control and README screenshot (F13, F15)

**Branch:** `feat/midi-control`. **Spec:** §5 and §6. Depends on plan 3
(availability, actions). The screenshot task runs after plans 1–7 have merged.

1. **Crate and parsing.**
   - What: a new crate `crates/fp-control` (`forbid(unsafe_code)`, the same
     lints as the others) with `midir` (check `cargo deny check`).
     `midi::parse(&[u8]) -> Option<MidiMessage>` handles Note On/Off
     (velocity 0 = off), CC and Pitch Bend (14-bit), and ignores anything
     else.
   - Tests: every kind, running status absent, truncated or malformed bytes.
2. **Config.**
   - What: `MidiConfig`, `MidiBinding`, `MidiTrigger` and `MidiAction` in
     `fp-model/src/config.rs` (or a new `midi.rs`), loaded leniently and
     validated. `rescan_interval_ms` is 250–60000.
   - Tests: defaults, a round trip, and invalid bindings dropped with a
     warning.
3. **Binding and commands.**
   - What: `Router::on_message(port, msg, &AppState) -> Option<Command>`.
     Buttons fire on Note On > 0 or on a rising CC edge at 64. Unavailable
     actions (plan 3 `availability`) are ignored.
   - Tests: each action, the edge detection, and availability.
4. **Soft takeover.**
   - What: `Pickup` per `Volume` binding, in fader travel. It re-arms when the
     model volume changes by another means.
   - Tests: no change before crossing; crossing from below and from above;
     re-arm.
5. **Ports and hot-plug.**
   - What: a `MidiPorts` trait (list inputs and outputs, connect, send) with a
     `midir` implementation, plus a rescan thread every `rescan_interval_ms`
     that reconnects by name.
   - Tests: with a fake `MidiPorts`, unplug and replug reconnect and refresh
     the LEDs.
6. **LED feedback.**
   - What: a feedback thread reads snapshots and computes the desired LED
     state per binding (Play lit while playing, Pause blinking at 500 ms while
     paused, Cue while cueing, Stop/Restart/Previous lit while available,
     unavailable unlit). It sends only the changes, to the same-named output
     or the override.
   - Tests: diffing, blinking cadence, override port.
7. **Learn.**
   - What: `LearnSession` binds the next matching message (buttons: Note or
     CC; Volume: CC or Pitch Bend). A binding used elsewhere moves. Esc
     cancels.
   - Tests: each case.
8. **Settings > MIDI.**
   - What: the enable switch, the ports and their state, and the per-player
     action rows with Learn and Clear, in both locales. Nothing blocks the UI
     (a helper thread and channels).
   - Tests: kittest learn flow with the fake ports.
9. **Wiring and docs.**
   - What: start `fp-control` from `main.rs` / `bootstrap.rs` with the
     controller channel. Add a `docs/user/midi.md` page, update
     `docs/technical/architecture.md` (crate, threads) and README
     (features). Clarify in the main spec §1 that the "remote control"
     non-goal means network control.
10. **README screenshot (F15).**
    - What: `examples/demo_session` sets up the realistic state of spec §5
      and plays real audio through the Null backend so the meters move. Take
      the screenshot with the `CLAUDE.md` procedure and replace
      `docs/images/main-screen.png`. The maintainer checks it before the PR.
