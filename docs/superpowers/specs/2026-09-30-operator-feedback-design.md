# Operator Feedback — Design Spec

- **Date:** 2026-09-30
- **Status:** Approved in brainstorming, pending written review
- **Extends:** [the main design spec](2026-09-25-fauste-player-design.md) (§3 rules,
  §6 analysis, §8 UI) and the [meters spec](2026-09-27-meters-design.md) (M4 display).
- **Scope:** 21 items of operator feedback (F1–F21), grouped into eight plans.
  Each plan is implemented in its own conversation and reaches `master` through
  its own pull request. Each plan updates the main spec, the user guide and the
  technical docs for what it changes.

---

## 1. Items and plans

| Item | Summary | Plan |
|---|---|---|
| F1 | Countdowns keep a fixed width | 1 |
| F2 | Drag on the waveform, seek on release | 5 |
| F3 | Restart and Previous buttons | 3 |
| F4 | Per-entry auto repeat | 6 |
| F5 | Per-entry stop after this entry | 6 |
| F6 | Proportional track-table columns | 7 |
| F7 | Stray marks between the meter channels | 2 |
| F8 | Duplicate output device names (ALSA) | 1 |
| F9 | Meter colours follow the theme | 2 |
| F10 | SINGLE/CONT as one segmented control | 1 |
| F11 | Meter and fader column on the right, with a dB scale | 2 |
| F12 | Local real-music test corpus | 4 |
| F13 | MIDI control with LED feedback | 8 |
| F14 | New default waveform colour | 1 |
| F15 | A realistic README screenshot | 8 (last task) |
| F16 | Shorter segues; trimming never removes audio | 4 |
| F17 | Mouse-wheel zoom on the waveform | 5 |
| F18 | The table follows the current entry | 7 |
| F19 | A clearer stop-after-current icon | 1 |
| F20 | The waveform shows the whole file | 5 |
| F21 | Version in the top bar and an About window | 1 |

| Plan | Title | Items | Depends on |
|---|---|---|---|
| 1 | Interface polish | F1, F8, F10, F14, F19, F21 | — |
| 2 | Meter column | F7, F9, F11 | 1 |
| 3 | Restart, Previous and button availability | F3 | 2 |
| 4 | Real-music corpus and marker tuning | F12, F16 | — |
| 5 | Waveform: drag, zoom and whole file | F2, F17, F20 | 4 |
| 6 | Per-entry repeat and stop | F4, F5 | 3 |
| 7 | Track table: columns and follow current | F6, F18 | 6 |
| 8 | MIDI control and README screenshot | F13, F15 | 3 (and 1–7 for the screenshot) |

Execution order: 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8.

---

## 2. Player behaviour (plans 3, 6 and 7)

These rules extend spec §3 (rules 1–22) and each gets an `fp-model` unit test.
The main spec's §3 is updated by the plan that implements them.

### 2.1 New commands and data

- `Command::Restart(PlayerId)`, `Command::Previous(PlayerId)` (plan 3).
- `Command::ToggleEntryRepeat(EntryId)`, `Command::ToggleEntryStopAfter(EntryId)` (plan 6).
- `ShortcutAction::RestartPlayer(n)` and `ShortcutAction::PreviousPlayer(n)`, with no
  default key. They appear in Settings > Shortcuts, and MIDI (§6) reuses them.
- `PlaylistEntry` gains `repeat: bool` and `stop_after: bool`. Both are
  `#[serde(default, skip_serializing_if = "is_false")]` in `playlists.json`, so no
  migration is needed. Duplicating an entry copies both flags.
- `PlayerState` gains `history: Vec<EntryId>`, persisted in `session.json`
  (lenient: missing or invalid means empty). Its depth is
  `players.history_len` (default 50, range 0–1000; 0 disables Previous).

### 2.2 Rules

**R23 Restart.** If the player has a current entry and is not Stopped, Restart
seeks to its `cue_in`, with the same anti-click replace-source path as a seek
(spec §4.1). A paused player stays paused. Otherwise, nothing happens.

**R24 Previous.** Only while Playing and not fading:
- entries are popped from the player's `history` until one still exists in a
  playlist and is playable (not `Missing` or `Unreadable`); the others are
  discarded. If none is left, nothing happens;
- the popped entry starts at full level and the current one fades out over
  `fade_ms`, exactly as Play-while-Playing (rule 5);
- the old current is marked played (rule 12) but is **not** pushed onto the
  history, so that Previous followed by Previous keeps going back instead of
  bouncing between two entries;
- `next` becomes the old current, with `next_explicit = true`.

**R25 History.** Every advance of rule 12 (a transition, Play-while-Playing,
Play-while-Stopped after a track) pushes the outgoing current onto `history`,
dropping the oldest entry beyond `players.history_len`. Previous (R24) does not
push. Removing an entry from its playlist does not edit histories; stale entries
are skipped when popped.

**R26 Auto repeat.** When the current entry has `repeat` and `plan_for` would
otherwise end, stop or chain it, the plan is instead a gapless restart of the
same entry at `cue_in` when it reaches `cue_out`:
- it is sample-accurate and uses the same preload and scheduled start as a
  segue, with no overlap and no fade (no segue with itself);
- it applies in Single and in Continuous mode;
- Pause does not end it: after Resume the entry keeps repeating;
- Play (next), Previous, Stop and Fade stop behave as usual and end it;
- with `stop_after_current` on, the current pass ends and the player stops
  (rule 9);
- toggling `repeat` while the entry plays re-plans through `reconcile`;
- every pass reports a `TransitionStarted` for the same entry. The model keeps
  `current` unchanged and does not mark the entry played or push it to history
  until the player leaves it.

**R27 Stop after this entry.** When the current entry has `stop_after`, the plan
is `StopAt { cue_out }` in any mode. At the end, Stop (rule 7) applies, but the
entry keeps its flag (it acts every time the entry plays) and `next` stays
marked. If the entry also has `repeat`, `stop_after` wins (R26 does not apply).
Unlike `stop_after_current` (rule 9), this flag belongs to the entry, persists,
and is not cleared by Stop.

**R28 Availability.** A pure function
`fp_model::availability(&AppState, PlayerId) -> Availability` says which
transport actions make sense now:

| Action | Available when |
|---|---|
| Play / Next | Paused with a current entry, or `next` exists and no fade is running |
| Pause | Playing and not fading, or Paused |
| Stop | there is a current entry |
| Fade stop | Playing and no fade stop running |
| Restart | there is a current entry and the player is not Stopped |
| Previous | Playing, not fading, and the history holds a playable entry |
| Stop after current | Continuous mode |
| Cue | `next` exists, or a cue is running |

The UI disables (dims) the buttons that are not available. Shortcuts and MIDI
ignore unavailable actions. `apply` keeps its own guards: availability is
presentation, not a second source of rules.

### 2.3 Track table (plans 6 and 7)

- **Entry flags.** An entry with `repeat` shows Phosphor's repeat icon, and one
  with `stop_after` shows the stop-after icon of §3.1 (F19). Both sit at the right
  of the title cell, in the row's text colour. The row context menu gains two
  checkable items: "Repeat this track" and "Stop after this track".
- **Follow the current entry (F18).** When a player's `current` changes, and the
  operator has not scrolled, dragged, opened a menu in, or clicked a tab of that
  player's table within the last `ui.follow_current_grace_secs` (default 10 s,
  range 0–600; 0 disables following):
  - the column shows the playlist that contains the new current (a
    `ShowPlaylist` command);
  - the table scrolls so the current row is the first visible row, or as far as
    the list allows near its end.

  If a drag or a context menu is active when the current changes, following
  waits until it ends, and then applies the grace rule from that moment. The
  interaction timestamps are UI view state, not model state.
- **Columns (F6).** `ColumnWidths` stores fractions of the table width for `#`,
  Title, Artist and Duration, summing to 1. `#` and Duration have minimum pixel
  widths that fit their content, and Title:Artist default to 60:40 of the rest.
  Resizing the window scales every column. Resizing a column by hand stores the
  new fractions when the handle is released. Pixel widths stored by earlier
  versions are discarded on load (lenient) and the defaults apply.

---

## 3. Interface (plans 1, 2 and 5)

### 3.1 Plan 1: polish

- **F1 Fixed-width times.** The UI font has no tabular figures in egui, so a helper
  lays out digits in cells as wide as the widest digit of that font and size.
  Separators keep their own widths. It is used by the big countdown, its tenths,
  `elapsed / total`, the playlist footer and the cue time. A test checks that
  `-00:00.0` and `-11:11.1` measure the same.
- **F8 Device names.** `DeviceInfo` gains `detail: Option<String>`, taken from
  cpal's `DeviceDescription` (`extended()` lines, for example "5.1 Surround
  output to Front…", else `address()`). A pure function builds the output
  picker's labels as `name — detail`. If two labels are still equal, it appends
  the device id. Tests cover ALSA-like duplicates.
- **F10 Segmented mode control.** SINGLE and CONT are one control, with a shared
  border and no gap. The selected half is filled `NEUTRAL_700`.
- **F14 Waveform colour.** A new palette entry `slate`: played `NEUTRAL_300`
  (#cfd3e5), unplayed #4a4e5c. It becomes the default `ui.wave_color`. An existing
  `config.json` that stores `sand` keeps it.
- **F19 Stop-after-current icon.** "Play then stop": a play triangle followed by
  a stop square, drawn as shapes in `icons.rs`. It replaces the return-arrow
  icon and is reused for entry flags (§2.3).
- **F21 Version and About.**
  - The top bar shows `Fauste Player` followed by the version
    (`CARGO_PKG_VERSION`, dimmed). Clicking the name or the version opens the
    About window.
  - The About window is modal, closes with Esc, and shows:
    - the name, the version and "Copyright © Marc Sánchez Fauste. All rights
      reserved.";
    - the notices that the bundled components' licences require: the Inter font
      (SIL Open Font License 1.1, with its copyright line and the OFL text,
      embedded from `assets/fonts/OFL.txt`) and Phosphor Icons (MIT, with its
      copyright and permission notice);
    - a statement that the program uses open-source Rust crates under their
      own licences, and a "Third-party licences" button that opens the
      installed `licenses/THIRD-PARTY.html` (produced by cargo-about in release
      builds). The file is looked up next to the executable and in each
      package's documented licence directory. If it is missing (a development
      build), the button is disabled and a tooltip says why.
  - All strings are in both locales. Opening the file never blocks the UI.

### 3.2 Plan 2: meter column (F7, F9, F11)

The layout follows mockup C, as approved:

```
┌ header: P1 ● ON AIR ……… BP [SINGLE|CONT] CUE ┐
│ [cover] Title / Artist / ■ next    │ max   │ ┃ │
│                                    │ 0 ─┃┃─ │ ┃ │
│ [PLAY][⏮][■][❚❚]   -02:19.3       │-20─┃┃─ │ ┃ │ fader
│       [|◀][◣][▶■]                  │-60─┃┃─ │ ┃ │
│                          01:18 / 03:40         │
│ [waveform ………………………………………………………………………………] │
```

- **Layout.** The meter and the volume fader leave the info row and form a
  column at the right of the player. It spans the info and transport rows (not
  the waveform). From left to right: the scale labels, the L and R bars side by
  side, then the fader. The maximum readout sits above the bars, and the
  loudness line below them when it is on.
- **Time.** `elapsed / total` moves under the transport, right-aligned.
- **Transport grid.** The grid becomes 3×2. Its first column holds Previous
  (top) and Restart (bottom), which plan 3 wires up. Stop, Pause, Fade stop and
  Stop after current keep their relative places.
- **Scale.** The dB labels (monospace, `NEUTRAL_400`) are chosen from
  `scale_marks` so that they never overlap at the available height. Every
  labelled mark gets a 1 px reference line across both bars, in `NEUTRAL_400` at
  35 % opacity. The alignment mark keeps its heavier weight.
- **F7.** The marks painted in the gap between the channels are removed, since the
  reference lines replace them.
- **F9 Colours.** New theme constants in "muted traffic-light" tones:
  - normal #7fb08a;
  - warning #d9b45a;
  - danger #d8646a.

  They replace `VU_GREEN`, `VU_YELLOW` and `VU_RED` everywhere the meter uses
  them, including the peak hold, the maximum readout and the loudness on-target
  colour.
- The meters spec M4 and `docs/user` are updated with the new layout.

### 3.3 Plan 5: waveform (F2, F17, F20)

- **F2 Drag to seek.**
  - Pressing and moving on the waveform shows a preview line with the target
    time.
  - Releasing inside the waveform seeks there. Releasing outside it, or
    pressing Esc, cancels, and nothing on air changes.
  - A plain click still seeks immediately. Alt-drag stays marker editing and
    never seeks.
- **F17 Zoom.**
  - The mouse wheel zooms in and out around the pointer. Shift+wheel or a
    horizontal wheel pans.
  - The deepest zoom shows one analysis bucket per pixel. It is derived from
    `peak_bucket_ms` and the width, not a constant.
  - While zoomed:
    - the view follows the playhead when it leaves the view, unless the
      operator panned in the last `ui.follow_current_grace_secs`. This plan
      introduces that `Config` field (default 10 s, range 0–600), and plan 7
      reuses it (§2.3);
    - a small "Full view" button appears in the top-right corner of the
      waveform.
  - Zooming fully out or changing track returns to the full view.
  - One pure view mapping, `WaveView { start_secs, span_secs }`, serves drawing,
    markers, marker handles, the hover tooltip and seeking, and it is
    unit-tested. `wave_columns` reduces only the visible range.
- **F20 Whole file.**
  - First, the plan verifies with a corpus file (§4) that the peaks and the
    span cover the whole decoded file. If they do not, the defect is fixed
    test-first.
  - Then the regions before `cue_in` and after `cue_out` are drawn dimmed, with
    thin marker lines, so the trimmed silence is visible instead of looking
    like the track starts late.

---

## 4. Analysis (plan 4: F12, F16)

### 4.1 Real-music corpus (F12)

- `test-music/` at the repository root is git-ignored except for its
  `README.md`, which explains what to put there. `FAUSTE_TEST_MUSIC=<dir>`
  overrides the location.
- `crates/fp-analysis/tests/real_music.rs` holds `#[ignore]` tests (run with
  `cargo test -p fp-analysis -- --ignored`). They skip with a message when the
  folder is empty or missing, and never run in CI. For every supported file
  they check the invariants of §4.2.
- The example `fp-analysis/examples/marker_report.rs` prints, for every file in
  the corpus, the duration, `cue_in`, `cue_out`, `segue_start`, the resulting
  overlap and `outro_start`. It is used to review the markers by ear and to set
  the defaults.
- `CLAUDE.md` and `docs/technical/testing.md` document the corpus.

### 4.2 Markers (F16)

- **Trimming never removes audio.**
  - `cue_in` and `cue_out` use the **peak** of each 10 ms bucket, not the 50 ms
    RMS window.
  - `cue_in` is the start of the first bucket whose peak is ≥
    `analysis.trim_threshold_db` (default −60 dBFS), moved back by
    `analysis.trim_margin_ms` (default 20 ms, clamped at 0).
  - `cue_out` is the end of the last such bucket, plus the margin (clamped at
    the duration).
  - Invariant (tested on fixtures and on the corpus): no bucket with a peak at
    or above the threshold lies outside `[cue_in, cue_out]`.
- **Shorter segues.**
  - The segue threshold is relative to the track: `analysis.segue_drop_db`
    (default 15 dB) below the median RMS of the body. It replaces the absolute
    `segue_threshold_db`.
  - `segue_start` is the end of the last window at or above that level, clamped
    to `[cue_out − segue_max_secs, cue_out]`, where `segue_max_secs` now defaults
    to 4 s.
  - The target is an overlap of 1–3 s on typical music. The final defaults are
    set with the corpus and recorded in the plan's ledger.
- **Outro.** Outro detection is unchanged.
- **Settings and cache.**
  - `silence_threshold_db` and `segue_threshold_db` are dropped from
    `AnalysisSettings`. A stored value is ignored on load (lenient), because its
    meaning changed. Settings > Analysis shows the new fields, each with a range
    in `Config::validate`.
  - The analysis version is bumped, so automatic markers are recomputed.
    Manual markers are kept.
- Spec §6 and `docs/user/markers-and-mixing.md` are updated.

---

## 5. README screenshot (plan 8, last task: F15)

- `examples/demo_session` sets up a realistic state:
  - player 1 on air mid-track;
  - player 2 paused;
  - player 3 further down its playlist, with played rows dimmed;
  - player 4 stopped with a next.

  It also plays real audio through the Null backend so the meters show level.
- `docs/images/main-screen.png` is regenerated with the procedure in `CLAUDE.md`
  once plans 1–7 have merged.

---

## 6. MIDI control (plan 8: F13)

### 6.1 Structure

- A new crate `fp-control` (depends on `fp-model`) owns MIDI.
- It uses `midir` (MIT) for cross-platform MIDI input and output: ALSA on
  Linux, CoreMIDI on macOS, WinMM on Windows. `forbid(unsafe_code)` holds in
  our code. `cargo deny check` runs after adding it.
- midir delivers input on its own threads. They are never audio threads. A
  message is parsed, matched against the bindings, and turned into a `Command`,
  which goes through the same controller channel as the UI. Parsing never
  panics, and malformed or unknown messages are ignored.
- The main spec's non-goal "remote control" is clarified to mean network
  control. A local MIDI control surface is in scope.

### 6.2 Configuration

`config.midi`, loaded leniently and validated:

- `enabled: bool` (default false);
- `rescan_interval_ms` (default 2000, range 250–60000);
- `feedback: bool` (default true);
- `devices`: per input port name, an optional `output_port` override for
  feedback;
- `bindings: Vec<MidiBinding>`, where `MidiBinding { device: String, trigger:
  MidiTrigger, action: MidiAction }`:
  - `MidiTrigger` is one of `Note { channel, note }`, `ControlChange { channel,
    controller }` or `PitchBend { channel }`;
  - `MidiAction` is one of `Button(ShortcutAction)`, restricted to per-player
    transport actions (Play/Next, Pause, Stop, Fade stop, Restart, Previous,
    Cue), or `Volume(u16)` (1-based player).

### 6.3 Behaviour

- **Buttons.** A button binding fires on Note On with velocity > 0, or on a CC
  rising from below 64 to 64 or above. An action that R28 marks unavailable is
  ignored.
- **Faders.**
  - A `Volume` binding reads a 7-bit CC or a 14-bit Pitch Bend, maps it to fader
    travel, and applies the UI fader's curve (`gain_from_fader`).
  - Soft takeover: the physical control changes the volume only after it
    reaches or crosses the current fader position. The binding's pickup is
    re-armed whenever the volume changes by any other means.
- **LED feedback.**
  - When `feedback` is on, a feedback thread watches the snapshots.
  - For each bound button, it sends the same Note (velocity 127 lit, 0 unlit)
    or CC (127 / 0) to the output port with the input's name, or to the
    configured override.
  - What each LED shows:
    - Play: lit while Playing;
    - Pause: blinks at the UI's 500 ms rate while Paused;
    - Cue: lit while cueing;
    - Stop, Restart and Previous: lit while available;
    - any unavailable action: unlit.
  - The feedback thread only sends a message when a LED's state changes.
- **Hot-plug.** Ports are rescanned every `rescan_interval_ms`. A port that
  returns is reconnected by name, and the LEDs are refreshed.
- **Rule 10.** Nothing goes on air at start-up. Only new messages act. At
  connection, faders start "not picked up".

### 6.4 Settings > MIDI

- An enable switch, and the ports with their state: connected or missing.
- For every player, a row per action (Play/Next, Pause, Stop, Fade stop,
  Restart, Previous, Cue, Volume), each with its binding, a "Learn" button and
  a "Clear" button.
- Learn binds the next matching message from any connected port: a Note or CC
  for buttons, a CC or Pitch Bend for Volume. A binding already used elsewhere
  moves to the new action. Esc cancels.
- Device enumeration and learning run off the UI thread (non-negotiable rule 8).
- All strings are in both locales. `docs/user/` gains a MIDI page.

---

## 7. Testing summary

- **`fp-model`:**
  - one test per rule R23–R28;
  - history bounds and skipping;
  - repeat plus stop-after precedence;
  - lenient loading of the new fields;
  - proptest: flags survive move and duplicate.
- **`fp-engine` (Offline backend):**
  - a repeat restarts at `cue_in` sample-accurately without a gap;
  - Previous crossfades like Next;
  - Restart seeks without clicks.
- **`fp-analysis`:**
  - the trim invariant on fixtures with soft fade-ins and fade-outs;
  - the relative segue on masters loud and quiet;
  - the corpus tests (`#[ignore]`).
- **`fp-app` (egui_kittest):**
  - fixed-width times;
  - device labels;
  - segmented control;
  - the About window and its disabled licence button;
  - meter geometry (bars, labels without overlap, reference lines);
  - availability dimming;
  - drag-to-seek and cancel;
  - zoom mapping;
  - follow-current grace, tab switch and scroll;
  - proportional columns under resize;
  - entry flag icons and menu items.
- **`fp-control`:**
  - message parsing (including malformed input);
  - binding match;
  - rising-edge CC;
  - soft takeover;
  - LED state diffing;
  - learn;
  - all with no device, through an injectable port trait.
