# Library Listening Level Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A volume fader and a full meter with its dB scale on the desktop
library preview bar, setting the library CUE's listening level, saved with
the library window's state.

**Architecture:** The level is session state in `LibraryView`
(`cue_volume`), changed only by a new `Command::SetLibraryCueVolume`. The
reducer sends `EngineAction::SetLibraryCueVolume` for live changes, and
every `LibraryCueRequest` carries the level, so a preview started after a
restart plays at it. The engine already gives the library CUE's mixer slot
a volume `Arc<AtomicF32>` (always 1.0 today); it now holds the level and
stores it there. The mixer meters after the volume, so the meter follows the
fader with no engine change. The UI draws the desktop player's meter and
fader column, shared rather than copied.

**Tech Stack:** Rust, egui 0.37 / egui_kittest, serde.

**Spec:** `docs/superpowers/specs/2026-11-01-track-library-design.md` §8.5
(LB58–LB61), with §8.4 for the existing preview bar.

## Global Constraints

- English everywhere in code, docs and commits.
- `unwrap`, `expect` and `panic` are denied outside tests.
- Real-time safety: nothing new on the mixer path. The engine only stores into the existing `AtomicF32`.
- The UI never blocks. It reads the state and sends commands.
- Every new UI string goes into all 11 locale files. en-US and es-ES are written by hand; the others are translated.
- Lenient loading: a bad saved level takes the default (1.0) and the rest of the view is kept.
- Desktop only (LB61). The phone's pre-listen card (`ui/phone_prelisten.rs`) does not change.
- Never mention other playout products.
- Commit only with these green: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace`. Conventional Commits, ending with the trailer `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.

## Review Focus

1. **A `SetLibraryView` sent in the same frame as a fader move.** The tree, a resize or the window placement send the whole view, copied from the state *before* the move. That copy must not undo the level. Rule: `SetLibraryView` keeps the state's `cue_volume` and ignores the incoming one (Task 1 test).
2. **A NaN, infinite or out-of-range level,** from a corrupt session file or a stray drag. NaN and infinities in a saved view load as 1.0. Out-of-range values are clamped into 0.0–1.0. A NaN command is ignored (Task 1 tests).
3. **The level set while no preview has ever started.** The engine has no library runtime yet, so the level must still apply to the first preview (Task 2 test).
4. **The player's meter and fader after the refactor.** They must look and behave the same, including a bit-perfect fader that is locked. The existing player UI tests must stay green, and the shared code must not lose the locked state (Task 3).
5. **The bar without a route.** Neither the meter nor the fader shows, and the notes keep their place (Task 3 test).

---

### Task 1: The level in the model and the session

**Files:**
- Modify: `crates/fp-model/src/library_view.rs`: the field `cue_volume`, its default of 1.0, its repair
- Modify: `crates/fp-model/src/command.rs`: `Command::SetLibraryCueVolume(f32)`, `EngineAction::SetLibraryCueVolume { volume: f32 }`, and `LibraryCueRequest.volume: f32`
- Modify: `crates/fp-model/src/reducer.rs`: handle the new command, and make `SetLibraryView` keep `cue_volume`
- Modify: `crates/fp-model/src/library_cue.rs`: the request carries `state.library_view.cue_volume`
- Modify: every exhaustive `match` on `Command`/`EngineAction` that stops compiling. The engine arm can be a no-op until Task 2.
- Test: `crates/fp-model/tests/it/library_view.rs`, `crates/fp-model/tests/it/library_cue.rs`

**Interfaces:**
- Produces:
  - `LibraryView::cue_volume: f32`: linear gain 0.0–1.0, default 1.0, under `#[serde(deserialize_with = "lenient")]`. `LibraryView::default()` must now give 1.0, so write `Default` by hand or use a serde default function.
  - `Command::SetLibraryCueVolume(f32)`.
  - `EngineAction::SetLibraryCueVolume { volume: f32 }`.
  - `LibraryCueRequest { …, volume: f32 }`.

- [ ] **Step 1: Write the failing tests**
  - `library_view.rs`:
    - `a_default_view_has_full_cue_volume`: `LibraryView::default().cue_volume == 1.0`.
    - `a_saved_view_without_cue_volume_loads_full`: `lenient_library_view(json!({"open": true}))` gives `cue_volume` 1.0 and `open` true.
    - `a_bad_cue_volume_loads_full_and_keeps_the_rest`: `{"open": true, "cue_volume": "loud"}` gives 1.0 and `open` true.
    - `repaired_clamps_the_cue_volume`:
      - `repaired` turns 1.7 into 1.0 and -0.2 into 0.0;
      - it turns `f32::NAN` and `f32::INFINITY` into 1.0.
  - `library_cue.rs`:
    - `set_library_cue_volume_stores_and_sends_it`: `apply(SetLibraryCueVolume(0.5))` sets `state.library_view.cue_volume == 0.5` and returns exactly `[EngineAction::SetLibraryCueVolume { volume: 0.5 }]`.
    - `set_library_cue_volume_clamps`: 2.0 is stored and sent as 1.0, -1.0 as 0.0.
    - `a_nan_library_cue_volume_is_ignored`: the level stays as it was and nothing is sent.
    - `set_library_view_keeps_the_cue_volume`:
      - set the level to 0.3;
      - apply a `SetLibraryView` whose view has `cue_volume` 1.0 and `open` changed;
      - `open` changes and the level stays 0.3.
    - `the_preview_starts_at_the_saved_level`: with `cue_volume` 0.25, `PlayLibraryCue` on a playable track sends a `StartLibraryCue` whose `request.volume == 0.25`.
    - `the_level_survives_a_track_switch`: a playing preview switched to another row (LB34) sends a new request with the same `volume`.

- [ ] **Step 2: Run the tests and watch them fail**

  Run: `cargo test -p fp-model --test it library_`

  Expected: the build fails because `cue_volume`, `SetLibraryCueVolume` and `volume` are unknown.

- [ ] **Step 3: Implement**
  - `repaired`: a non-finite `cue_volume` becomes 1.0; anything else is clamped into 0.0–1.0.
  - Reducer:
    - `SetLibraryCueVolume(v)`: ignore NaN; otherwise clamp, store and push the engine action. This happens with or without a route, and with or without a preview.
    - `SetLibraryView(view)`: after `repaired`, put back the state's previous `cue_volume`.
  - The session store saves `LibraryView` as JSON already. Check that `fp-store` round-trips the new field: if a library-view persistence test exists in `crates/fp-store/tests/it`, add `cue_volume` 0.4 to it and assert it survives.

- [ ] **Step 4: Run the tests and see them pass**

  Run: `cargo test -p fp-model --test it library_ && cargo test -p fp-store`

  Expected: PASS.

- [ ] **Step 5: Commit**

  `feat(model): library CUE listening level (LB59, LB60)`

### Task 2: The engine applies the level

**Files:**
- Modify: `crates/fp-engine/src/engine/library.rs`: keep the level and store it into the runtime's `volume`
- Modify: `crates/fp-engine/src/engine.rs`: the `EngineAction::SetLibraryCueVolume` arm, and the field that holds the level before a runtime exists
- Test: `crates/fp-engine/tests/it/library_cue.rs`
- Docs: `docs/technical/audio-engine.md`: the actions table and the "Library CUE" section

**Interfaces:**
- Consumes (from Task 1): `EngineAction::SetLibraryCueVolume { volume }` and `LibraryCueRequest.volume`.

- [ ] **Step 1: Write the failing tests**

  Use the `Offline` backend and explicit `now`, following the existing tests in `library_cue.rs`. Use a full-scale tone file as those tests do.
  - `the_library_cue_volume_scales_its_output`:
    - start a preview with `request.volume` 1.0, render past the volume smoothing, and record the peak on the Cue channels;
    - send `SetLibraryCueVolume { volume: 0.5 }`, render past the smoothing again;
    - the peak is half of the first one (±1 %);
    - the library CUE's meter reading (`Telemetry.library_cue`) falls by about 6 dB (±0.5).
  - `a_level_set_before_the_first_preview_applies_to_it`:
    - send `SetLibraryCueVolume { volume: 0.5 }` while the library runtime does not exist yet;
    - then start with `request.volume` 0.5;
    - the peak is half of full scale.
  - `a_new_preview_takes_its_requests_level`: a preview started with `volume` 0.25 renders at a quarter of full scale.

- [ ] **Step 2: Run the tests and watch them fail**

  Run: `cargo test -p fp-engine --test it library_cue::`

  Expected: FAIL. The output stays at full scale.

- [ ] **Step 3: Implement**
  - The engine keeps `library_volume: f32` (initially 1.0).
  - `SetLibraryCueVolume` sets it and, when the runtime exists, stores it into `LibraryCueRuntime::volume`.
  - `start_library_cue` sets both from `request.volume` before it attaches the source.
  - `ensure_library` creates the atomic with `library_volume` instead of 1.0.
  - Nothing else changes. The mixer already smooths volume changes.

- [ ] **Step 4: Run the tests and see them pass**

  Run: `cargo test -p fp-engine --test it library_cue::`

  Expected: PASS.

- [ ] **Step 5: Commit**

  `feat(engine): apply the library CUE listening level (LB59)`

### Task 3: The meter and fader on the preview bar

**Files:**
- Modify: `crates/fp-app/src/ui/player.rs`: pull the desktop branch of `meter_column` (the full `widgets::vu`, the hairline, `widgets::fader` and `widgets::fader_readout`) out into a `pub(crate)` helper that the preview bar can call. Its inputs are the reading, the labels, the fader position, the tooltip, the readout, whether it is enabled and the height. It returns whether the meter's maximum was clicked and the new fader position.
- Modify: `crates/fp-app/src/ui/library/preview.rs`: use the helper instead of `meter_column` with `vu_compact`, and take its width (`player.rs` has `widgets::meter_width(c) + METER_FADER_GAP + widgets::FADER_WIDTH`; expose it as a `pub(crate)` function) instead of `compact_meter_width`
- Modify: every locale `crates/fp-app/locales/*/main.ftl`: the new tooltip
- Test: `crates/fp-app/tests/it/library_preview_ui.rs`
- Docs:
  - `docs/user/library.md` (the "Pre-listening" list: the meter with its scale and the fader; the level is kept and saved and never changes a track);
  - `docs/technical/ui.md` (the `ui/library/preview.rs` row);
  - the `README.md` feature line on the library pre-listen, if there is one.

**Interfaces:**
- Consumes (from Task 1): `Command::SetLibraryCueVolume(f32)` and `state.library_view.cue_volume`.
- Reuses: `view::fader_from_gain`, `view::gain_from_fader`, `view::volume_db`, `view::volume_readout`, and the `unit-db`, `volume-silent` and `volume-readout-silent` messages.

New strings:

```
# en-US
tip-library-volume = Pre-listen level: { $db }
# es-ES
tip-library-volume = Nivel de preescucha: { $db }
```

Translate the other nine with the pre-listen word each locale already uses (`library-preview-play`).

- [ ] **Step 1: Write the failing tests** (in `library_preview_ui.rs`, using its `open`/`bar` helpers)
  - `the_bar_has_a_listening_level_fader`: with a route and a selected track, a widget labelled `Pre-listen level: 0.0 dB` is inside the bar.
  - `dragging_the_fader_sends_set_library_cue_volume`:
    - drag the fader's knob down;
    - the last command is a `SetLibraryCueVolume(v)` with `v < 1.0`;
    - no `SetVolume` was sent.
  - `the_fader_shows_the_saved_level`:
    - with `cue_volume` 0.5, the label reads `Pre-listen level:` followed by the dB that `view::volume_db(0.5)` gives, formatted as the player's tooltip formats it;
    - the readout under the fader shows `view::volume_readout(0.5)`.
  - `the_bar_meter_has_its_scale`: the bar contains the full meter's scale, for example its `0` mark label (use the same lookup the player meter tests use).
  - `without_a_route_neither_meter_nor_fader_shows`: no `Pre-listen level` widget, and the existing "needs a library CUE output" line is still shown.

- [ ] **Step 2: Run the tests and watch them fail**

  Run: `cargo test -p fp-app --test it library_preview_ui::`

  Expected: FAIL. The bar has no fader yet.

- [ ] **Step 3: Implement**
  - Extract the helper. The player's own `meter_column` keeps its phone branch and calls the helper for the desktop one, with its locked fader and `reset_meter_max(id)` unchanged.
  - In `preview_bar`, the meter column becomes the helper. Its height is the bar's inner height. Its maximum click calls `reset_library_cue_meter_max`. Its fader sends `SetLibraryCueVolume(view::gain_from_fader(pos))`. It is always enabled.
  - Update the doc comment on `meter_column` in `preview.rs`, or remove it if the helper replaces it entirely.
  - Run the app (`FAUSTE_HOME=<scratch> cargo run -p fp-app`, or Xvfb) and look at the bar at 1920×1080. The meter's scale, loudness line and fader readout must not be clipped. If they are, record a ruling and fix it in this task.

- [ ] **Step 4: Run the tests and see them pass, including the player's**

  Run: `cargo test -p fp-app --test it`

  Expected: PASS. No player or main-screen test changes.

- [ ] **Step 5: Commit**

  `feat(ui): listening level fader and full meter on the library preview bar (LB58)`

## Not in this plan (rulings)

- The phone card, the remote API and MIDI (LB61).
- `docs/images/guide/library.png` is retaken by the screenshots workflow after the merge, not by hand.
