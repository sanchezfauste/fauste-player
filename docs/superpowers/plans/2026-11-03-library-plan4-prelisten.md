# Track library plan 4 — pre-listen in the library Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The library window gets a preview bar that pre-listens the
selected track on a new output, the **library CUE**, with Play, Pause and
Stop, the big remaining time, title, artist and category, a small meter,
and the zoomable waveform with click-to-seek and marker editing; the route
is set in Settings → Outputs.

**Architecture:** A new holder, `Holder::LibraryCue`, with its route
`outputs.library.cue`, joins every holder enumeration (live settings,
on-air pre-listen CA1–CA6, pending output changes). Its rules are pure
functions in a new `fp_model::library_cue` module over a runtime-only
`AppState::library_cue`; the UI reports the selected row with one command
and the model decides what the preview does (LB34). The engine plays it
from its own runtime (`engine/library.rs`, its own worker, created on first
use and placed from the start, like the cartwall) with pause and seek like
a player's CUE; the conductor publishes its position and its own meter in
`Telemetry`. The bar reuses the players' wave panel (generalised from an
entry to an entry or a track) and the meter, countdown and transport
widgets.

**Tech Stack:** Rust, egui 0.37 / egui_kittest, the `Offline` backend for
engine tests.

**Spec:** `docs/superpowers/specs/2026-11-01-track-library-design.md`
(§8 LB32–LB37; §9 the Settings → Outputs line; §13 `outputs.library.cue`;
§14; §16 item 4; §17), with the on-air pre-listen spec
`docs/superpowers/specs/2026-10-15-cue-on-air-output-design.md` (CA1–CA10).

## Global Constraints

- Every user-visible string is a Fluent message in
  `crates/fp-app/locales/en-US/main.ftl`. The parity test
  (`tests/it/i18n.rs::every_locale_defines_exactly_the_english_messages_and_variables`)
  fails on a missing key, so **each task that adds a string adds it to all
  11 locale files** in that task: en-US and es-ES written, the other nine
  (ca-ES, de-DE, eu-ES, fr-FR, gl-ES, it-IT, nl-NL, pl-PL, pt-PT) translated.
- Rule 4: `outputs.library.cue` has a default (`None`) and lenient loading;
  a route has no numeric range, so `Config::validate` gets no entry (as the
  cartwall's Cue).
- Rule 5: nothing new runs on the device callback; the engine only sends
  `BusCommand`s (`Attach`, `Start`, `Pause`, `Resume`, stop with the
  de-click ramp) as the cart and CUE paths do.
- Rule 7: every preview decision (what Play starts, LB34 switching, LB35
  stops, CA3–CA5) is in `fp-model` with one test per rule.
- Rule 8: the UI reads `AppState` and `Telemetry` only; peaks come from the
  services thread's `MediaCache`.
- Rule 10: `AppState::library_cue` is runtime only: never saved, stopped
  after every start, even with `library_view.open` saved as true.
- CA1: without a library CUE route nothing starts. CA7: an on-air preview
  never reopens a bit-perfect bus at its file's rate. Pre-listen never goes
  out as DSD (`LoadOptions::dsd: false`); over a DSD stream it follows the
  device's DSD mix policy like a cart (CA10).
- The preview bar and the window are desktop only (`view_state.phone_active`
  draws nothing, §11); the model and engine parts are shared.
- `unwrap`/`expect`/`panic` denied outside tests; `fp-engine` denies
  `clippy::indexing_slicing`.
- New integration tests go into the crate's `tests/it/` with a `mod` line in
  `tests/it/main.rs`.
- Never mention other playout or radio-automation products. Describe only
  conventional pre-listen behaviour (play, pause, stop, seek on a separate
  output) in this project's own terms.

## Review Focus

- **A selected track whose file is missing or unreadable:** Play starts
  nothing and the bar says the file is not available; a running preview
  that is switched to it stops (Task 4 tests
  `lb33_a_missing_file_does_not_start`,
  `lb34_selecting_a_track_that_cannot_play_stops_the_preview`; Task 8 test
  `a_missing_track_shows_not_available_and_play_is_dimmed`).
- **A track not analysed yet (length unknown, no peaks):** it plays to the
  source end, the bar shows `--:--` total and no remaining countdown, and
  the waveform draws empty without panicking (Task 7 test
  `an_unknown_length_has_no_total_and_no_remaining`; Task 2 test
  `without_a_known_length_the_preview_plays_to_the_source_end`).
- **The library CUE route removed or moved in Settings while the preview
  plays:** the preview keeps playing until it stops (L11), then the change
  applies; Play is refused once there is no route (Task 2 test
  `a_route_change_waits_while_the_preview_sounds`; Task 4 test
  `lb32_a_removed_route_refuses_play_but_keeps_a_running_preview`).
- **The selection emptied while the preview plays (a search that filters
  the row out, Ctrl+click):** the preview keeps playing and the bar keeps
  showing it (Task 4 test `lb34_selecting_nothing_leaves_a_running_preview`;
  Task 8 test `filtering_the_row_out_keeps_the_preview_playing`).
- **A seek to NaN, a negative time or past the end:** NaN is ignored, the
  rest is clamped into the file (Task 4 test
  `lb33_a_seek_outside_the_file_is_clamped_and_nan_is_ignored`).

## Rulings made while writing this plan

- **The selected track** is the most recently selected row
  (`LibraryUi::selected.last()`): with several rows selected, the last one
  clicked is previewed.
- **Play on a paused preview resumes it; Play on a playing preview does
  nothing** (no restart from the cue-in; Stop then Play does that).
- **A click on the waveform while stopped** sets where the next Play starts
  (`LibraryCue::start_at`), as a stopped player's seek does (rule 3a); it is
  dropped when the bar's track changes. While playing or paused it seeks.
- **Selecting nothing** (no row selected, or the selected row filtered
  out) leaves a running preview as it is; a stopped bar empties.
- **Selecting a track that cannot be played** (missing, unreadable) while
  the preview runs stops it: the bar always shows the selected track and
  never shows one track while another sounds.
- **A running preview that switches to another track moves without asking**
  (CA3 asks only for a pre-listen that starts from nothing, as for the
  players' CUE). A switch while paused starts the new track held.
- **LB17b:** the library CUE does not hold a track on air; removing its
  track stops it (LB35).
- **The engine runtime** is the cartwall pattern: its own worker
  (`fp-library-cue`), created on the first `StartLibraryCue`, its placement
  reported from the engine's start and after each settings update until
  then, so `pending` never shows a route change that needs no apply (L3,
  L13).
- **The small meter** is `widgets::vu_compact` without a ruler, fed by its
  own `MeterState` in the conductor; its maximum restarts when the bar's
  track changes and when clicked. It is not a programme meter (CA9: it
  never reaches a player's or the cartwall's reading).
- **Without a route** the transport is dimmed with the reason as its
  tooltip; the bar still shows the track, its waveform and marker editing
  (LB18 does not need an output), and a line says why with a link that
  opens Settings on Outputs.
- **The CA5 notice's `by`** is always a Main holder: `notice_text` folds
  `Holder::LibraryCue` into the cartwall arm with a comment instead of a
  string no one would see.

---

### Task 1: The holder, its route and its runtime state

**Files:**
- Create: `crates/fp-model/src/library_cue.rs` (state type only in this task), `crates/fp-model/tests/it/library_cue.rs`
- Modify: `crates/fp-model/src/config.rs` (`LibraryRoutes`, `OutputsConfig::library`, `library_has_cue`, `cue_shares_main`, `routed_devices`), `live.rs` (`Holder`, `BusyCause`, `holders`, `configured_route`, `causes`), `cue_on_air.rs` (`stop`), `state.rs` (`AppState::library_cue`), `lib.rs`, `playout.rs`
- Modify (exhaustive `Holder`/`BusyCause` sites outside fp-model): `crates/fp-app/src/ui/pending.rs` (`holder_label`, `describe_cause`), `crates/fp-app/src/ui/cue_on_air.rs` (`sounding_text`, `notice_text`), `crates/fp-engine/src/engine.rs` (`holder_quiet`, `apply_route`, `EngineSettings`), `crates/fp-engine/src/engine/carts.rs` (`move_cartwall`'s holder match), `crates/fp-mobile/src/shell_state.rs` (`sounding`)
- Modify (test match sites): `crates/fp-model/tests/it/cue_on_air.rs` (`place`), `crates/fp-model/tests/it/common/mod.rs` (`place_on_dac`), `crates/fp-app/tests/it/support/mod.rs` (`on_desk`)
- Modify: `crates/fp-store/src/lenient.rs` (unit test), every locale file (`holder-library-cue`, `cause-library-cue`, `cue-on-air-library-cue`)
- Test: `crates/fp-model/tests/it/library_cue.rs`, `crates/fp-engine/tests/it/library_cue.rs` (new, with its `mod` line)

**Interfaces:**
- Produces:
  ```rust
  // config.rs
  #[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)] #[serde(default)]
  pub struct LibraryRoutes { pub cue: Option<Route> }
  pub struct OutputsConfig { …, #[serde(default)] pub library: LibraryRoutes }
  impl OutputsConfig { pub fn library_has_cue(&self) -> bool } // cue_is_usable(self.library.cue.as_ref())
  // live.rs
  pub enum Holder { PlayerMain(PlayerId), PlayerCue(PlayerId), CartwallMain, CartwallCue, LibraryCue }
  pub enum BusyCause { …, LibraryCue }
  // library_cue.rs
  #[derive(Debug, Clone, Default, PartialEq)]
  pub struct LibraryCue {
      pub track: Option<TrackId>,   // the bar's track (LB34)
      pub transport: Transport,     // Stopped, Playing or Paused
      pub start_at: Option<f64>,    // where Play starts while stopped
  }
  // state.rs: pub library_cue: crate::library_cue::LibraryCue  (runtime only)
  // fp-engine EngineSettings: pub library_routes: LibraryRoutes (from config.outputs.library)
  ```
  `holders()` order: … `CartwallMain`, `CartwallCue`, `LibraryCue` (last).
  `causes(LibraryCue)` is `[BusyCause::LibraryCue]` while `transport != Stopped`
  (a held preview carries audio, Q4), empty otherwise.

- [ ] **Step 1: Write the failing model tests** in `library_cue.rs`:
  `lb32_the_library_cue_is_the_last_holder` (`holders(&s).last() == Some(&Holder::LibraryCue)`);
  `lb32_its_route_is_outputs_library_cue` (`configured_route(&config, Holder::LibraryCue)` equals the set route; `None` by default; `library_has_cue()` follows it);
  `lb32_a_running_preview_is_a_cause` (`transport = Playing` and `Paused` give `[BusyCause::LibraryCue]`, `Stopped` gives `[]`);
  `lb32_routed_devices_and_cue_shares_main_count_the_library_cue` (a library cue on a player's Main device and pair: `cue_shares_main(Holder::LibraryCue)` is true; its device is in `routed_devices()`);
  `lb32_the_config_round_trips` (serde JSON of a config with the route, back equal; JSON without `library` gives `None`);
  `rule10_a_fresh_state_has_the_library_cue_stopped` (`AppState::new(..).library_cue == LibraryCue::default()`).
  And the store test in `lenient.rs`'s `mod tests`:
  `a_bad_library_cue_route_falls_back_to_none` (`{"outputs": {"sample_rate": 44100, "library": {"cue": 5}}}` → `outputs.library.cue == None`, `sample_rate == 44100`, one warning naming `outputs/library/cue`).
- [ ] **Step 2: Run** `cargo test -p fp-model --test it library_cue` and
  `cargo test -p fp-store lenient` — FAIL (no `Holder::LibraryCue`; the
  store warns `outputs/library: not used by this version`).
- [ ] **Step 3: Implement** the model side above, plus `cue_on_air::stop`'s
  `Holder::LibraryCue` arm (sets `transport = Stopped`, `start_at = None`
  and pushes nothing yet; Task 4 adds the engine action) and
  `playout_active` / fp-mobile `sounding` counting a running preview.
  In `cue_shares_main` the library's route is matched like the cartwall's
  Cue; `routed_devices` chains `library.cue` after the cartwall.
- [ ] **Step 4: Write the failing engine test** in
  `crates/fp-engine/tests/it/library_cue.rs` (an `Offline` rig like
  `cartwall.rs::rig`, with a library route argument):
  `lb32_the_library_cue_is_placed_from_the_start` (the events after
  `Engine::new` contain `Placed { holder: Holder::LibraryCue, route: Some(r), .. }`
  with a route, `Unplaced { holder: Holder::LibraryCue, route: None }`
  without); `lb32_a_route_set_later_is_reported_without_opening_a_stream`
  (`UpdateSettings` with a route → a `Placed` for `LibraryCue`, no bus opened
  for it: `bus_status()` unchanged).
- [ ] **Step 5: Implement** the engine placement: `report_library_route(&mut self, route: Option<Route>)`
  (target `cue_target(route)`, as `report_cartwall_route` does for a Cue),
  called in `Engine::new` and in `update_settings` when
  `settings.library_routes` changed. Until Task 2: `holder_quiet(LibraryCue)`
  is `true` and `apply_route(LibraryCue, route, _)` reports the placement
  and `Applied` Ok. `move_cartwall`'s holder match puts `Holder::LibraryCue`
  in its `PlayerMain | PlayerCue => Some(Ok(()))` arm.
- [ ] **Step 6: Add the UI arms and strings:** `holder_label` →
  `holder-library-cue` ("Library CUE" / "CUE de la biblioteca");
  `describe_cause` → `cause-library-cue` ("library CUE" / "CUE de la
  biblioteca"); `sounding_text` → `cue-on-air-library-cue` ("the library is
  pre-listening" / "la biblioteca está en preescucha"); `notice_text` folds
  it into the cartwall arm (ruling). Extend the `pending.rs` unit test
  `items_and_causes_read_as_the_spec_says` with a `Route` item for
  `Holder::LibraryCue` and the `LibraryCue` cause. All 11 locale files.
- [ ] **Step 7: Run** `cargo test -p fp-model --test it library_cue`, `cargo test -p fp-store lenient`,
  `cargo test -p fp-engine --test it library_cue`, `cargo test -p fp-app pending`,
  `cargo test -p fp-app --test it i18n` — PASS; then fmt, clippy and the
  whole suite (Android: `cargo check -p fp-mobile`).
- [ ] **Step 8: Commit** `feat(model): add the library CUE holder and its route`.

### Task 2: The engine plays, stops and ends the library CUE

**Files:**
- Create: `crates/fp-engine/src/engine/library.rs`
- Modify: `crates/fp-model/src/command.rs` (`LibraryCueRequest`, `EngineAction`, `EngineEvent`), `crates/fp-model/src/reducer.rs` (the `LibraryCueEnded` arm), `crates/fp-model/src/lib.rs`
- Modify: `crates/fp-engine/src/engine.rs` (`mod library`, `Engine::library`, `execute`, `holder_quiet`, `apply_route`, `bus_in_use`, `channels_for`, `ensure_bus`, `sounding_besides`, `unsettled_sources`, `attached_sources`, `handle_failures`, `handle_bus_event`, `start_ready_sources`)
- Test: `crates/fp-engine/tests/it/library_cue.rs`, `crates/fp-model/tests/it/library_cue.rs`

**Interfaces:**
- Consumes: Task 1's `Holder::LibraryCue`, `LibraryRoutes`, `report_library_route`.
- Produces:
  ```rust
  // fp-model command.rs
  #[derive(Debug, Clone, PartialEq)]
  pub struct LibraryCueRequest {
      pub track: TrackId, pub path: PathBuf, pub from_secs: f64,
      pub format: Option<AudioFormat>, pub normalization: crate::loudness::Normalization,
  }
  EngineAction::StartLibraryCue { request: LibraryCueRequest, paused: bool }
  EngineAction::StopLibraryCue
  EngineAction::SeekLibraryCue { secs: f64 }          // handled in Task 3
  EngineAction::SetLibraryCuePaused { paused: bool }  // handled in Task 3
  EngineEvent::LibraryCueEnded { track: TrackId }
  // fp-engine engine/library.rs
  pub(super) struct LibraryCueRuntime { worker: PlayerWorker, cue: Option<(BusKey, u16)>,
      route: Option<Route>, source: Option<LibrarySource>, outgoing: Vec<LibrarySource>, paused: bool }
  impl Engine {
      pub(super) fn start_library_cue(&mut self, request: &LibraryCueRequest, paused: bool, now: Instant);
      pub(super) fn stop_library_cue(&mut self);
      pub(super) fn library_path_quiet(&self) -> bool;
      pub(super) fn library_uses(&self, key: &BusKey) -> bool;
      pub(super) fn library_sources_on(&self, key: &BusKey) -> usize;
      pub(super) fn library_sounding(&self, bus: &BusKey) -> bool;
      pub(super) fn move_library_cue(&mut self, route: Option<Route>, force: bool) -> Option<Result<(), String>>;
      pub(super) fn library_failure(&mut self, failure: &WorkerFailure) -> bool;
      pub(super) fn library_bus_event(&mut self, bus: &BusKey, event: &BusEvent) -> bool;
      pub(super) fn start_ready_library(&mut self);
      pub(super) fn unsettled_library(&self) -> usize;
      pub fn library_cue_position(&self) -> Option<(TrackId, f64)>;
  }
  ```
  `LibrarySource` mirrors `CartSource` (key, bus, slot, from_secs, shared,
  start, failed, request, held) without `cart`, `looped` and `cue`.

- [ ] **Step 1: Write the failing engine tests** (Offline, time driven by `now`):
  `lb32_a_preview_plays_only_on_the_library_cue_route` (Main pair silent, Cue pair audible);
  `lb32_without_a_route_a_preview_ends_at_once` (`LibraryCueEnded { track }` in the first tick's events);
  `lb36_the_preview_plays_to_the_end_of_the_file_and_reports_it` (a short file; `LibraryCueEnded` once, after the last frame);
  `without_a_known_length_the_preview_plays_to_the_source_end` (`format: None`, same end);
  `stop_ramps_down_and_releases_its_slot` (after `StopLibraryCue` and the de-click ramp, `used_slots()` back to the count before; no `LibraryCueEnded`);
  `a_new_start_replaces_the_preview_without_an_end_event`;
  `a_paused_start_stays_silent` (`paused: true`: nothing audible after the ready threshold);
  `a_missing_file_ends_the_preview` (`LibraryCueEnded` for that track);
  `n14_the_preview_plays_with_its_normalization_gain` (a −6 dB gain halves the peak, as `cartwall.rs::n14_…`);
  `a_route_change_waits_while_the_preview_sounds` (`ApplyRoute { holder: LibraryCue, force: false }` gives no `Applied` while playing; after `StopLibraryCue` and the ramp it applies and reports `Placed` on the new device);
  `ca7_an_on_air_preview_never_reopens_a_bit_perfect_bus` (as `cartwall.rs::ca8_…` with a bit-perfect Main on the same pair: the bus rate stays).
- [ ] **Step 2: Run** `cargo test -p fp-engine --test it library_cue` — FAIL (no actions).
- [ ] **Step 3: Implement** `engine/library.rs` on the cart pre-listen
  pattern (`ensure_cartwall`, `cart_source`, `start_cart_cue`,
  `stop_cart_source`, `cart_failure`, `cart_bus_event`, `start_ready_carts`):
  worker `"fp-library-cue"`; `LoadOptions { until_secs: None, looped: false, dsd: false, .. }`;
  the source starts `WhenReady { fade_in: false }`, or `Idle` when
  `paused`; `prepare_start` only when `!self.cue_on_air(&cue)` (CA7); no
  target → `LibraryCueEnded` at once. `library_sources_on` is 2 on the
  library CUE's bus (a source and one ramping down), 0 elsewhere.
  `move_library_cue` follows `place_cart_cue` (a running preview on a moved
  route ends with `LibraryCueEnded`), reports the placement, and before the
  first preview only reports it (Task 1's behaviour). Wire every hook listed
  under Files; `holder_quiet(LibraryCue)` becomes `library_path_quiet()`.
- [ ] **Step 4: Write the failing model test:** `lb36_the_engines_end_stops_the_preview`
  (a state with `track = Some(a)`, `transport = Playing`: `on_event(LibraryCueEnded { track: a })`
  gives `Stopped`; `LibraryCueEnded { track: b }` changes nothing).
- [ ] **Step 5: Implement** the reducer arm (`library_cue::ended(state, track)`).
- [ ] **Step 6: Run** both crates' tests — PASS; fmt, clippy, whole suite.
- [ ] **Step 7: Commit** `feat(engine): play the library CUE on its own route`.

### Task 3: Pause, seek, rate changes, telemetry and the meter

**Files:**
- Modify: `crates/fp-engine/src/engine/library.rs`, `crates/fp-engine/src/engine.rs` (`execute`, `reopen_on_bus`), `crates/fp-engine/src/conductor.rs` (`Telemetry`, `EngineRequest`, the meter state, `ConductorHandle`), `crates/fp-app/src/ui/controller.rs` (`Controller::reset_library_cue_meter_max`)
- Test: `crates/fp-engine/tests/it/library_cue.rs`, `crates/fp-engine/tests/it/conductor.rs`

**Interfaces:**
- Consumes: Task 2's runtime and actions.
- Produces:
  ```rust
  impl Engine {
      pub(super) fn seek_library_cue(&mut self, secs: f64);
      pub(super) fn set_library_cue_paused(&mut self, paused: bool);
      pub(super) fn follow_forced_rate_library(&mut self, bus: &BusKey, old_rate: u32);
      pub fn take_library_cue_meter_input(&self) -> crate::meter::MeterInput;
  }
  #[derive(Debug, Clone, Copy, PartialEq)]
  pub struct LibraryCueTelemetry { pub track: TrackId, pub position_secs: f64, pub meter: crate::meter::MeterReading }
  pub struct Telemetry { …, pub library_cue: Option<LibraryCueTelemetry> }
  pub enum EngineRequest { …, ResetLibraryCueMeterMax }
  impl ConductorHandle { pub fn reset_library_cue_meter_max(&self) -> bool }
  trait Controller { fn reset_library_cue_meter_max(&self) -> bool { false } } // default: Fake needs no change
  ```

- [ ] **Step 1: Write the failing engine tests:**
  `lb33_pause_holds_and_resume_continues_where_it_was` (position constant while held, within one block after release);
  `a_pause_before_the_start_holds_the_source_idle` (as the CUE's `set_cue_paused` on a `WhenReady` source);
  `lb33_a_seek_replaces_the_source_at_the_new_position` (`library_cue_position()` ≈ the target after the ready threshold);
  `a_seek_while_held_stays_held_at_the_new_position`;
  `a_forced_rate_change_reopens_the_preview_at_its_position` (a bit-perfect library CUE device that a player's track moves to another rate: after the reopen the preview's position continues from where it was, within one block);
  `the_preview_meter_reads_only_the_preview` (`take_library_cue_meter_input().frames > 0`, a player's `take_meter_input` on the same bus stays empty).
- [ ] **Step 2: Run** `cargo test -p fp-engine --test it library_cue` — FAIL.
- [ ] **Step 3: Implement** seek and pause on the CUE pattern (`seek_cue`,
  `set_cue_paused`: the old source ramps down in `outgoing`; `paused`
  survives a seek and is cleared by a new start or a stop), the forced-rate
  reopen called from `reopen_on_bus` next to `follow_forced_rate_carts`, and
  the meter input taken from `source` and `outgoing` like `take_meter_input`.
- [ ] **Step 4: Write the failing conductor test**
  `the_telemetry_carries_the_library_preview_and_its_meter` (after a
  `StartLibraryCue` and a few ticks: `library_cue == Some(t)` with the
  track, a position ≥ `from_secs` and `meter.level_db` above the floor on both channels;
  after `StopLibraryCue` and the ramp: `None`), and
  `the_library_meter_maximum_restarts_with_a_new_track`.
- [ ] **Step 5: Implement** the conductor: a `library_meter: MeterState`
  updated each tick from `take_library_cue_meter_input`, reset when the
  telemetry's track changes and on `ResetLibraryCueMeterMax`.
- [ ] **Step 6: Run** `cargo test -p fp-engine --test it library_cue conductor` — PASS; fmt, clippy, whole suite.
- [ ] **Step 7: Commit** `feat(engine): pause, seek and meter the library CUE`.

### Task 4: The preview's rules (LB32–LB36)

**Files:**
- Modify: `crates/fp-model/src/library_cue.rs` (rules), `command.rs` (`Command` variants), `reducer.rs` (`apply_command` arms; `SetLibraryView`; `reconcile`), `availability.rs` (`command_available`), `cue_on_air.rs` (`stop` pushes `StopLibraryCue`)
- Test: `crates/fp-model/tests/it/library_cue.rs`, `crates/fp-model/tests/it/availability.rs`

**Interfaces:**
- Consumes: Task 1's state, Task 2's actions and request.
- Produces:
  ```rust
  Command::SelectLibraryCue(Option<TrackId>)   // the selected row changed (LB34)
  Command::PlayLibraryCue                      // plays library_cue.track (LB33)
  Command::SetLibraryCuePaused(bool)
  Command::StopLibraryCue
  Command::SeekLibraryCue(f64)
  // library_cue.rs
  pub fn request(state: &AppState, track: TrackId, at: Option<f64>) -> Option<LibraryCueRequest>;
  pub(crate) fn select(state: &mut AppState, track: Option<TrackId>, out: &mut Vec<EngineAction>);
  pub(crate) fn play(state: &mut AppState, force: bool, out: &mut Vec<EngineAction>);
  pub(crate) fn set_paused(state: &mut AppState, paused: bool, out: &mut Vec<EngineAction>);
  pub(crate) fn stop(state: &mut AppState, out: &mut Vec<EngineAction>);
  pub(crate) fn seek(state: &mut AppState, secs: f64, out: &mut Vec<EngineAction>);
  pub(crate) fn ended(state: &mut AppState, track: TrackId);       // Task 2
  pub(crate) fn settle(state: &mut AppState, out: &mut Vec<EngineAction>); // LB35, from reconcile
  ```
  `request`: `None` for an unknown track or one whose `file_state` is not
  playable; `at: None` or non-finite → `play_range(config.players.use_cue_markers).cue_in`;
  finite → clamped to `[0, duration]` when the duration is known, `[0, ∞)`
  otherwise; normalization is `normalization_for(state, Holder::LibraryCue, track)`.
  `play(force: false)` is what `PlayLibraryCue` calls; Task 5 makes it ask
  (CA3) and adds the forced path.

- [ ] **Step 1: Write the failing tests** (a fixture with a library route
  and two analysed tracks `a`, `b` with cue-ins 1.5 s and 2.0 s):
  - LB32: `lb32_without_a_route_play_does_nothing` (no action, `Stopped`);
    `lb32_a_removed_route_refuses_play_but_keeps_a_running_preview`
    (`command_available(PlayLibraryCue)` false while stopped without a
    route, true while paused; Stop always available; an `UpdateConfig`
    removing the route leaves `Playing` and emits no stop).
  - LB33: `lb33_play_starts_the_selected_track_from_its_cue_in`
    (`[StartLibraryCue { request.track == a, request.from_secs == 1.5, paused: false }]`, `Playing`);
    `lb33_play_while_playing_does_nothing`;
    `lb33_pause_holds_and_play_releases` (`SetLibraryCuePaused(true)` → `[SetLibraryCuePaused { paused: true }]`, `Paused`; `PlayLibraryCue` → `[SetLibraryCuePaused { paused: false }]`, `Playing`; a repeated pause sends nothing);
    `lb33_stop_ends_it` (`[StopLibraryCue]`, `Stopped`; Stop while stopped sends nothing);
    `lb33_a_seek_moves_a_running_preview` (`SeekLibraryCue(30.0)` → `[SeekLibraryCue { secs: 30.0 }]`, transport unchanged);
    `lb33_a_seek_while_stopped_sets_where_play_starts` (no action; `start_at == Some(30.0)`; Play → `from_secs == 30.0`, then `start_at == None`);
    `lb33_a_seek_outside_the_file_is_clamped_and_nan_is_ignored` (−5 → 0, 1e9 → duration, NaN → no action and `start_at` unchanged);
    `lb33_a_missing_file_does_not_start` (`SetFileState(a, Missing)` then Play: no action).
  - LB34: `lb34_selecting_another_track_switches_a_playing_preview_from_its_cue_in`
    (`[StartLibraryCue { request.track == b, from_secs == 2.0, paused: false }]`, `track == Some(b)`);
    `lb34_a_paused_preview_switches_held` (`paused: true`, still `Paused`);
    `lb34_a_stopped_preview_stays_stopped` (no action, `track == Some(b)`, `start_at == None`);
    `lb34_selecting_the_same_track_changes_nothing`;
    `lb34_selecting_nothing_leaves_a_running_preview` (`SelectLibraryCue(None)` while playing: no action, `track == Some(a)`; while stopped: `track == None`);
    `lb34_selecting_a_track_that_cannot_play_stops_the_preview` (`[StopLibraryCue]`, `track == Some(b)`, `Stopped`).
  - LB35: `lb35_removing_the_track_stops_the_preview` (`RemoveTracks(vec![a])` → contains `StopLibraryCue`, `track == None`);
    `lb35_deleting_its_category_stops_the_preview`;
    `lb35_closing_the_window_stops_it_and_minimising_does_not` (`SetLibraryView` with `minimised: true` → still `Playing`; with `open: false` → `[StopLibraryCue]`).
  - LB36: `lb36_the_request_carries_the_library_cue_normalization` (equals `normalization_for(&s, Holder::LibraryCue, track)`; with normalization off it is 0 dB).
  - Rule 10: `rule10_a_restored_state_has_the_library_cue_stopped` (`AppState::restore` from `RestoreParts` built as `common::roundtrip` does but with `library_view.open = true`, from a state whose preview played: `library_cue == LibraryCue::default()` and no `StartLibraryCue` among the returned actions).
- [ ] **Step 2: Run** `cargo test -p fp-model --test it library_cue availability` — FAIL.
- [ ] **Step 3: Implement** the rules and the reducer arms. `settle`
  (called at the end of `reconcile`) stops a preview whose track left the
  library and clears a `track` that left it; `SetLibraryView` calls `stop`
  when the new view is closed. `command_available`: `PlayLibraryCue` needs
  `library_has_cue()` and a playable `track`, except to release a paused
  preview; `StopLibraryCue`, `SetLibraryCuePaused` and `SeekLibraryCue` are
  always available.
- [ ] **Step 4: Run** — PASS; fmt, clippy, whole suite.
- [ ] **Step 5: Commit** `feat(model): pre-listen rules of the library CUE`.

### Task 5: On-air pre-listen for the library CUE (LB37)

**Files:**
- Modify: `crates/fp-model/src/command.rs` (`CueStart::Library`), `cue_on_air.rs` (`cue_start`, `starts_fresh`, `before_step`), `reducer.rs` (`force_cue`), `library_cue.rs` (`play` asks)
- Modify: `crates/fp-app/tests/it/cue_on_air_ui.rs`
- Test: `crates/fp-model/tests/it/cue_on_air.rs`

**Interfaces:**
- Consumes: Task 4's `play(state, force, out)`.
- Produces: `CueStart::Library(TrackId)` with `holder() == Holder::LibraryCue`.
  Exhaustive `CueStart` sites: `CueStart::holder` (command.rs),
  `starts_fresh` (cue_on_air.rs), `force_cue` (reducer.rs). Non-exhaustive
  site that must name the holder: `before_step`'s
  `matches!(h, PlayerCue(_) | CartwallCue)` gains `LibraryCue`. The remote
  API (`fp-remote/src/api.rs`) and MIDI (`fp-control/src/router.rs`) call
  `needs_confirmation` and send no library commands: no change there.

- [ ] **Step 1: Write the failing tests** in `cue_on_air.rs` (the file's
  `place`/`on_desk` helpers; a library CUE on `desk` 1–2 with player 2's Main):
  `lb37_a_quiet_on_air_output_starts_the_library_preview_at_once`;
  `lb37_a_sounding_on_air_output_asks_first` (player 2 playing: Play gives no action, `request == Some { holder: LibraryCue, start: CueStart::Library(a), sounding: [PlayerMain(p2)] }`);
  `lb37_pre_listen_anyway_starts_it_and_confirms_it` (`ForceCue(CueStart::Library(a))` → `StartLibraryCue`, `confirmed == [LibraryCue]`);
  `lb37_an_unconfirmed_preview_stops_when_a_player_starts_there` (`StopLibraryCue` first in the actions, `notice == Some((LibraryCue, PlayerMain(p2)))`);
  `lb37_a_confirmed_preview_keeps_playing`;
  `lb37_needs_confirmation_for_play_library_cue` (true while it would ask; false while paused (a release) and while playing);
  `lb37_the_request_goes_when_another_track_is_selected` (`SelectLibraryCue(Some(b))` drops it);
  `lb37_a_running_preview_switching_tracks_asks_nothing`.
- [ ] **Step 2: Run** `cargo test -p fp-model --test it cue_on_air` — FAIL.
- [ ] **Step 3: Implement.** `cue_start(PlayLibraryCue)` is
  `Some(CueStart::Library(track))` only while stopped with a `track`;
  `starts_fresh(Library(t))`: stopped, `library_cue.track == Some(t)`,
  `library_has_cue()` and `request(state, t, start_at).is_some()`;
  `force_cue(Library(t))`: `UnknownTrack(t)` if not in the library, then
  `select(Some(t))` and `play(force: true)` unless it already runs on `t`.
- [ ] **Step 4: Write the failing UI test** in `cue_on_air_ui.rs`:
  `a_library_preview_asks_the_same_way` (a state with that request: the
  modal's body names "player 2 is playing"; *Pre-listen anyway* sends
  `ForceCue(CueStart::Library(a))`).
- [ ] **Step 5: Run** the model and UI tests — PASS; fmt, clippy, whole suite.
- [ ] **Step 6: Commit** `feat(model): the library CUE asks before pre-listening on air`.

### Task 6: Settings → Outputs: the library CUE route (§9)

**Files:**
- Modify: `crates/fp-app/src/ui/settings.rs` (`outputs`, `Owner::Library`, `set_route`, `SettingsState::show_outputs`), `crates/fp-app/src/ui/app.rs` (`ViewState::open_outputs`, handled next to `edit_cart`), `crates/fp-app/tests/it/support/mod.rs` (`give_library_cue_route`), every locale file (`settings-library-outputs`)
- Test: `crates/fp-app/tests/it/settings.rs`

**Interfaces:**
- Consumes: Task 1's `outputs.library.cue`, `Holder::LibraryCue`.
- Produces: `pub(crate) fn SettingsState::show_outputs(&mut self)` (section
  Outputs, `phone_section = true`); `ViewState::open_outputs: bool`, which
  `AppUi` turns into `settings.show_outputs()` and `settings_open = true`;
  `pub fn give_library_cue_route(state: &mut AppState)` (test support: the
  library CUE on `null`/`phones`).

- [ ] **Step 1: Write the failing tests:**
  `the_library_cue_route_is_in_both_outputs_views` (Settings → Outputs,
  Basic then Advanced: a row labelled "Library" with one CUE picker and
  its Test CUE button);
  `picking_a_library_cue_device_sends_the_route` (choose `phones` → an
  `UpdateConfig` whose `outputs.library.cue` is that route, every other
  route unchanged);
  `a_library_cue_on_an_on_air_output_shows_the_note` (library CUE on a
  player's Main pair: the `settings-cue-on-air` note shows under it);
  `open_outputs_opens_settings_on_outputs`.
- [ ] **Step 2: Run** `cargo test -p fp-app --test it settings` — FAIL.
- [ ] **Step 3: Implement** the row after the cartwall's with
  `route_picker(…, Owner::Library, Bus::Cue, …)` and the on-air note for
  `Holder::LibraryCue`; `set_route` ignores `Bus::Main` for
  `Owner::Library`. String `settings-library-outputs` ("Library" /
  "Biblioteca"), all 11 locales.
- [ ] **Step 4: Run** — PASS; fmt, clippy, whole suite.
- [ ] **Step 5: Commit** `feat(ui): set the library CUE route in Settings → Outputs`.

### Task 7: The waveform for a track, the preview's view model and its peaks

**Files:**
- Modify: `crates/fp-app/src/ui/wave_view.rs` (`WaveSubject`, `WaveZoom`, `WaveZooms::get`, `WaveKey::Library`), `crates/fp-app/src/ui/wave_panel.rs` (`WavePanelInput::subject`, `label_key`, the loudness popup holder), `crates/fp-app/src/ui/player.rs` and `cue_window.rs` (pass `WaveSubject::Entry`), `crates/fp-app/src/ui/view.rs` (`library_cue_view`), `crates/fp-app/src/services.rs` (`wanted`), every locale file (`tip-library-waveform`)
- Test: `crates/fp-app/tests/it/waveform_view.rs`, `crates/fp-app/tests/it/view.rs`, `crates/fp-app/tests/it/services.rs`

**Interfaces:**
- Consumes: Tasks 1–4 (`AppState::library_cue`, `LibraryCueTelemetry`).
- Produces:
  ```rust
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
  pub enum WaveSubject { Entry(EntryId), Track(TrackId) }   // a zoom belongs to one
  pub struct WaveZoom { pub view: WaveView, pub subject: WaveSubject, pub moved_at: f64 }
  impl WaveZooms { pub fn get(&self, key: WaveKey, subject: Option<WaveSubject>) -> Option<WaveZoom> }
  pub enum WaveKey { Player(PlayerId), Cue(PlayerId), Library }  // Library.id() = Id::unique("library-waveform")
  // WavePanelInput: `entry: Option<EntryId>` becomes `subject: Option<WaveSubject>`
  // view.rs
  #[derive(Debug, Clone, PartialEq)]
  pub struct LibraryCueView {
      pub track: TrackId, pub title: String, pub artist: Option<String>,
      pub category: Option<String>,        // None: uncategorised
      pub transport: Transport, pub elapsed: f64, pub total: Option<f64>,
      pub remaining: Option<f64>,          // None while the length is unknown
      pub markers: MarkerFractions,        // whole file, `ignored: true` as a CUE
      pub intro: Option<f64>, pub outro: Option<f64>,
      pub playable: bool, pub has_route: bool,
  }
  pub fn library_cue_view(state: &AppState, position: Option<f64>) -> Option<LibraryCueView>;
  ```
  `position` is the telemetry's when its track is the bar's; otherwise the
  elapsed time is `start_at`, else the cue-in. `None` without a `track`.

- [ ] **Step 1: Write the failing tests:**
  in `waveform_view.rs`, `a_zoom_belongs_to_its_entry_or_its_track`
  (`set(WaveKey::Library, zoom on Track(a))`: `get(Library, Some(Track(a)))` is it,
  `Some(Track(b))` and `Some(Entry(e))` are `None`) and `the_library_waveform_has_its_own_id`;
  in `view.rs`, `the_library_cue_view_shows_the_track_and_its_category`
  (title, artist, category name; `category: None` for an uncategorised
  track), `a_stopped_preview_shows_its_start_point_or_cue_in`,
  `a_playing_preview_follows_the_telemetry_and_counts_down_to_the_file_end`
  (remaining = duration − position), `an_unknown_length_has_no_total_and_no_remaining`,
  `without_a_route_or_a_playable_file_it_says_so` (`has_route`, `playable` false);
  in `services.rs`, `the_library_preview_track_gets_its_peaks_while_the_window_is_open`
  (as `peaks_are_kept_only_for_tracks_on_a_player`: `SelectLibraryCue(Some(t))`
  with the window open → `media.contains(t)`; closed → not kept).
- [ ] **Step 2: Run** `cargo test -p fp-app --test it waveform_view view services` — FAIL.
- [ ] **Step 3: Implement.** The refactor keeps the players' and the CUE
  window's behaviour (their existing tests must pass unchanged apart from
  the field rename). `label_key(Library)` is `tip-library-waveform`
  ("Library CUE waveform: click to seek" / "Forma de onda del CUE de la
  biblioteca: haz clic para buscar"), all 11 locales; the Loudness… item
  opens the popup with `Holder::LibraryCue`. `wanted` adds
  `state.library_cue.track` while `state.library_view.open`.
- [ ] **Step 4: Run** — PASS; fmt, clippy, whole suite.
- [ ] **Step 5: Commit** `refactor(ui): let the wave panel show a track; the library preview's view`.

### Task 8: The preview bar in the library window

**Files:**
- Create: `crates/fp-app/src/ui/library/preview.rs`
- Modify: `crates/fp-app/src/ui/library/mod.rs` (`draw`: the bar above the split; `PREVIEW_BAR_HEIGHT`), `crates/fp-app/src/ui/library/state.rs` (`LibraryUi::preview_sent`), `crates/fp-app/src/ui/player.rs` and `widgets.rs` (`countdown` moved to `widgets::countdown` and shared), every locale file
- Test: `crates/fp-app/tests/it/library_preview_ui.rs` (new, `mod` line)

**Interfaces:**
- Consumes: Task 7's `library_cue_view`, `WaveKey::Library`, `WaveSubject::Track`; Task 4's commands; Task 3's `Telemetry::library_cue`, `Controller::reset_library_cue_meter_max`; Task 6's `ViewState::open_outputs`.
- Produces:
  ```rust
  pub(crate) const PREVIEW_BAR_HEIGHT: f32 = 112.0;
  pub(crate) fn preview_bar(ui: &mut Ui, scene: &Scene<'_>, view_state: &mut ViewState);
  pub(crate) fn follow_selection(scene: &Scene<'_>, library: &mut LibraryUi); // LB34: sends SelectLibraryCue
  pub(crate) fn countdown(ui: &mut Ui, remaining: f64, colour: Color32, size: f32); // widgets.rs
  // LibraryUi: pub(crate) preview_sent: Option<Option<TrackId>>
  ```
  Layout, left to right on the top row: Play, Pause, Stop (40×28 tiles,
  icons through `glyphs::TransportAction`, Pause lit with
  `paused_play_style` while paused); the big remaining time
  (`widgets::countdown`, `--:--` while unknown); title (semibold), then
  artist and the category's name (or "Uncategorised",
  `library-tree-uncategorised`) in a column that elides; the small meter at
  the right (`vu_compact`, no ruler). Below, full width: the wave panel
  (`WaveKey::Library`, editable markers, `follow` while playing) and the
  `elapsed / total` line (`view::time_text`). Without a track: one line
  `library-preview-empty`. Without a route: the meter's place shows
  `library-preview-no-route` and a link `library-preview-open-settings`
  that sets `view_state.open_outputs`.

- [ ] **Step 1: Write the failing UI tests** (`harness_sized`, a state
  with `give_library_cue_route`, the window open, three library tracks):
  `selecting_a_row_sends_select_library_cue` (click row 2 →
  `SelectLibraryCue(Some(t2))` sent once, not again on the next frames);
  `the_bar_plays_pauses_and_stops_the_selected_track` (Play →
  `PlayLibraryCue`; Pause → `SetLibraryCuePaused(true)`; Stop →
  `StopLibraryCue`, each through the Fake's reducer);
  `the_bar_shows_title_artist_category_and_the_remaining_time`;
  `a_click_on_the_waveform_seeks_the_preview` (`SeekLibraryCue(secs)` with
  `0 < secs < duration`; with media from `harness_with_media`);
  `the_waveform_menu_sets_a_marker_on_the_selected_track`
  (right-click, *Set cue in here* → `SetMarker { track: t2, kind: CueIn, .. }`);
  `without_a_route_the_bar_says_why_and_opens_settings_on_outputs`
  (Play dimmed with the reason as its tooltip; the link opens Settings with
  the "Library" route row visible);
  `a_missing_track_shows_not_available_and_play_is_dimmed`;
  `filtering_the_row_out_keeps_the_preview_playing` (a playing preview,
  then a search that lists nothing: no `StopLibraryCue` or
  `SelectLibraryCue` sent);
  `a_minimised_window_draws_no_bar_and_sends_nothing`;
  `the_player_countdown_is_unchanged` (the players' existing countdown tests
  in `fade_countdown_ui.rs` pass; this one checks the player header's
  remaining-time label text after the move to `widgets::countdown`).
- [ ] **Step 2: Run** `cargo test -p fp-app --test it library_preview_ui` — FAIL.
- [ ] **Step 3: Implement** `preview.rs` and `follow_selection`, called
  after the table each frame: it sends `SelectLibraryCue(selected.last())`
  when that differs from `preview_sent`, and records it. The bar's body is
  drawn only when not minimised (LB35: minimising keeps the preview).
  Strings, all 11 locales: `library-preview-play` ("Pre-listen" /
  "Preescuchar"), `library-preview-pause` ("Pause pre-listen" / "Pausar la
  preescucha"), `library-preview-stop` ("Stop pre-listen" / "Detener la
  preescucha"), `library-preview-empty` ("Select a track to pre-listen to
  it" / "Selecciona una pista para preescucharla"),
  `library-preview-no-route` ("Pre-listen needs a library CUE output" /
  "La preescucha necesita una salida CUE de la biblioteca"),
  `library-preview-open-settings` ("Choose it in Settings → Outputs" /
  "Elígela en Ajustes → Salidas"), `library-preview-not-available` ("The
  file cannot be played" / "El archivo no se puede reproducir").
- [ ] **Step 4: Run** — PASS. Then `cargo run -p fp-app` with
  `FAUSTE_HOME=<scratch>` and the `null` backend, open the library, and
  check the bar at the default window size and at `MIN_SIZE` (nothing
  overlaps; the tree and table keep at least 120 px of height); fmt,
  clippy, whole suite.
- [ ] **Step 5: Commit** `feat(ui): pre-listen from the library's preview bar`.

### Task 9: Documentation

**Files:**
- Modify: `README.md` (the track library feature: pre-listen on its own CUE output), `docs/user/library.md` (a *Pre-listening* section: the bar, Play/Pause/Stop, selection follows, click to seek, markers and loudness as on a player, what stops it, the route and the on-air question), `docs/user/settings.md` (Outputs table: a *Library* row), `docs/user/players.md` (*CUE (pre-listen)*: the on-air list names the library pre-listen and the question comes from the library too), `docs/user/markers-and-mixing.md` (*Editing markers*: also on the library's preview bar)
- Modify: `docs/technical/audio-engine.md` (the library CUE runtime, its actions in the actions table, `LibraryCueEnded`, telemetry and meter, placement before first use), `docs/technical/ui.md` (`ui/library/preview.rs`, `WaveSubject`, `WaveKey::Library`, `ViewState::open_outputs`), `docs/technical/architecture.md` (the holder list)
- Modify: the spec, a new *§8.4 Implementation notes (plan 4)* listing this plan's rulings

- [ ] **Step 1:** Write the pages above in English (the `docs/i18n`
  translations are not touched in a feature PR).
- [ ] **Step 2:** Run `scripts/site/build.sh` and
  `scripts/site/check-links.sh target/site` — no dead link.
- [ ] **Step 3:** Run fmt, clippy, the whole suite and
  `scripts/check-mobile-isolation.sh`.
- [ ] **Step 4: Commit** `docs: pre-listen in the library`.
