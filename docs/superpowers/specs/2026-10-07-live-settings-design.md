# Live Settings — Design Spec

- **Date:** 2026-10-07
- **Status:** Draft, for the maintainer's review.
- **Extends:** [the main design spec](2026-09-25-fauste-player-design.md)
  (§3 rules, §4 engine, §8.4 Settings), the
  [bit-perfect spec](2026-09-26-phase4-bit-perfect-design.md) (B3, B4, B6),
  the [second operator feedback spec](2026-10-01-operator-feedback-2-design.md)
  (**O4 is replaced** by this spec; O25 DSD settings are amended) and the
  [fourth operator feedback spec](2026-10-06-operator-feedback-4-design.md)
  (**Q12.6 is replaced**).
- **Numbering:** rules are `L<n>`. Each rule is testable and gets at least
  one test (§10).
- **Line numbers** are those of `master` at the date above; each reference
  also names the function or field.

---

## 1. Goal

Every setting that today waits for a restart (`fp_model::restart_pending`,
`crates/fp-model/src/restart.rs`: `AudioSystem`, `SampleRate`,
`BufferSize`, `Routes`, `BitPerfect`, `DsdOutput`, `Limits`, `Tuning`) takes
effect while the application runs, **as soon as it is safe**, without ever
cutting audio on air by itself. When a change must wait, the operator sees
what waits and on what, and may apply it at once with an explicit,
confirmed interruption.

The application restart (O4: the "Restart pending" pill, **Restart now**
and the relaunch) is no longer needed and is removed (§9.4, Q2).

## 2. Maintainer decisions (binding)

| # | Decision |
|---|---|
| D1 | A change is applied as soon as it is safe. Nothing is cut on air by itself (CLAUDE.md rule 10 in spirit). |
| D2 | **Busy** means a device is carrying audio: a source routed to it is playing, fading or in CUE (PFL), or a cart is playing on it. A paused player, or one with a track only loaded, is **not** busy. |
| D3 | An output-side change (sample rate, buffer size, route, bit-perfect, DSD mode, DSD mix, DSD silence) applies **per device**, to each affected device when that device is idle. A change of audio system affects every device and applies when **all** are idle. |
| D4 | The notice lists what waits and on what (which device, which player or cart keeps it busy) and offers **Apply now**, with a confirmation that says it briefly interrupts the audio on those devices. |
| D5 | Limits apply at once when safe; reductions wait until what they affect is idle or empty (§6). Tuning values are read live wherever possible (§7), reaching the device callback only through `BusCommand` (rule 5). |
| D6 | The rules live in `fp-model` as pure functions, one test per rule. The engine and the UI only execute and display. |

## 3. Terms

| Term | Meaning |
|---|---|
| **Holder** | One audio path that a route places on a device: `PlayerMain(PlayerId)`, `PlayerCue(PlayerId)`, `CartwallMain`, `CartwallCue`. |
| **Placement** | The device (`OutputDevice { backend, device }`) the engine really put a holder on: the routed device, or the default output it resolved (a missing Main route, a route to a backend this machine lacks). Only the engine knows it (`Engine::route_target`, `crates/fp-engine/src/engine.rs` ~L980), so it reports it (L3). |
| **Device in use** | A device at least one holder is placed on. |
| **Device settings** | `DeviceSettings { sample_rate, buffer_frames, bit_perfect, dsd: DsdOutput, dsd_mix: Option<DsdMix>, dsd_silence_ms: Option<f64> }`: everything a device's stream depends on (L4). |
| **Running** | The value the engine runs with now. **Wanted**: the value of the current `Config`. |
| **Item** | One pending change: a device, a holder's route, the audio system, the player limit or one cart page (§4.2). |
| **Cause** | Why an item waits: `BusyCause::{PlayerPlaying, PlayerFading, PlayerCue}(PlayerId)`, `CartPlaying(CartId)`, `CartCue(CartId)`; for limit reductions only, `PlayerPaused(PlayerId)` and `CartsBeyondGrid { page, count }`. |
| **Quiet** (engine) | A bus with no audible source: no started, unpaused source (current, outgoing or CUE), no cart source and no test tone. Paused and waiting sources are quiet. |

## 4. The pending-change model

### 4.1 What is stored

`AppState` gains `live: LiveSettings`, runtime only (never saved, like
`PlayerState::dsd`). The engine fills it through events; the reducer reads
it.

| Field | Type | Set by | Meaning |
|---|---|---|---|
| `audio_system` | `Option<String>` | `Applied { AudioSystem }`; at start, `config.outputs.backend` | The `outputs.backend` value the engine runs with. |
| `audio_system_in_use` | `String` | `EngineEvent::AudioSystemInUse` | The backend really chosen (`choose_default_backend`); feeds the top bar's platform label. |
| `placement` | `BTreeMap<Holder, OutputDevice>` | `EngineEvent::Placed`, `Unplaced` | Where each holder with a bus plays. |
| `routes` | `BTreeMap<Holder, Option<Route>>` | `Placed`, `Unplaced`, `Applied { Route }` | The route the engine holds for each holder it knows, with a bus or not. |
| `devices` | `BTreeMap<OutputDevice, DeviceSettings>` | `Placed`, `Applied { Device }` | Running settings of each device in use. |
| `limits` | `Limits` | reducer (§6); at start, `config.limits` | Running limits. |
| `in_flight` | `BTreeMap<Target, Wanted>` | reducer, cleared by `Applied` | Actions sent and not yet answered, with the value asked for. |
| `failures` | `BTreeMap<Target, Failure { wanted, reason }>` | `Applied` with an error | The last refused value per target (L14). |

`Target` is `AudioSystem | Route(Holder) | Device(OutputDevice)`, and
`Wanted` is `AudioSystem(Option<String>) | Route(Option<Route>) |
Device(DeviceSettings)`. `Holder` and `OutputDevice` derive `Ord` for the
maps. `Tuning` is not stored: it never waits (§7).

### 4.2 How it is computed

`fp_model::live::pending(state: &AppState) -> Vec<Pending>` replaces
`restart_pending`. `Pending { item, from, to, causes: Vec<BusyCause>,
failure: Option<String> }`, in this order:

| Item | Pending when | Causes |
|---|---|---|
| `AudioSystem` | `live.audio_system != config.outputs.backend` | every cause of every holder (L7) |
| `Route(h)` for each `h` in `live.routes` | `live.routes[h] != config route of h` | the cause of `h` only (L6) |
| `Device(d)` for each device in use | `live.devices[d] != device_settings(&config.outputs, d)` | the causes of every holder placed on `d` (L5) |
| `Players` | `players.len() > config.limits.max_players` | for each excess player that is not removable (L18): its L1 cause, else `PlayerPaused` |
| `CartPage(p)` for each page over the grid limit | `p.rows > max_cart_rows` or `p.cols > max_cart_cols` | `CartPlaying`/`CartCue` of carts on `p`, and `CartsBeyondGrid` (L19) |

A device not in use is never pending: it opens with the wanted values
when first used (L13). A holder the engine has not reported yet (a player
just added) is not pending either: it is created from the current
configuration (L13).

## 5. Rules

### 5.1 Busy

- **L1** A holder's cause (`fp_model::live::cause(state, holder) ->
  Option<BusyCause>`):

  | Holder | Busy when | Cause |
  |---|---|---|
  | `PlayerMain(p)` | `transport == Playing` and not fading | `PlayerPlaying(p)` |
  | `PlayerMain(p)` | `fading` (a fade stop or a transition) | `PlayerFading(p)` |
  | `PlayerCue(p)` | `cue.is_some()` (playing or held) | `PlayerCue(p)` |
  | `CartwallMain` | `cartwall.playing` not empty | `CartPlaying(c)` for each |
  | `CartwallCue` | `cartwall.cue == Some(c)` | `CartCue(c)` |

  `Paused`, `Stopped` with a loaded entry, and a preload are not busy (D2).
- **L2** A device's causes are the causes of every holder placed on it
  (`fp_model::live::device_causes`). A device is idle when it has none.
- **L3** The engine reports every holder it knows. `EngineEvent::Placed
  { holder, route, device, running: DeviceSettings }` when it binds a
  holder to a bus (player added, route applied, audio system applied, the
  cartwall's first cart or cart CUE). `Unplaced { holder, route }` for a
  holder without a bus: a player with no Cue route, a Cue route the
  engine drops (equal to the resolved Main), the cartwall before its
  first cart (reported at engine start). `Gone { holder }` when a player
  is removed; the model then forgets the holder.

### 5.2 Pending and due

- **L4** `fp_model::live::device_settings(&OutputsConfig, &OutputDevice)
  -> DeviceSettings`: `sample_rate = effective_rate`, `buffer_frames =
  effective_buffer`, `bit_perfect` = listed in `bit_perfect`, `dsd` =
  `dsd_output_for` when bit-perfect else `Pcm`; `dsd_mix` and
  `dsd_silence_ms` are `Some` only when `dsd != Pcm` (on any other device
  they change nothing, so they never make it pending).
- **L5** A device item is pending as in §4.2 and waits on its L2 causes
  only: a busy device never delays another device (D3).
- **L6** A route item waits on its own holder only: moving an idle player
  or cart path off a device, or onto one, interrupts nobody else on
  either device. (See Q1.)
- **L7** The audio system item waits until no holder anywhere is busy (D3).
- **L8** `fp_model::live::due(state) -> Vec<EngineAction>`: every pending
  output-side item with no cause, not in `in_flight` with the same wanted
  value, and not in `failures` with the same wanted value. The reducer
  calls it at the end of every `apply` and `on_event` (like
  `fill_empty_next`), records the actions in `in_flight`, and emits them.
  `EngineEvent::Applied { target, wanted, outcome: Result<(), String> }`
  removes `in_flight[target]` when its wanted value matches; on `Ok` it
  sets the running value (`live.audio_system`, `live.routes`,
  `live.devices`) and removes `failures[target]`.
- **L9** In one step, actions go in this order: `ApplyAudioSystem`, then
  `ApplyRoute` (holders in display order, then the cartwall), then
  `ApplyDevice`. Limit reductions (L18, L19) run in the same step, in the
  reducer itself.
- **L10** `UpdateConfig` also emits `EngineAction::UpdateSettings(Box<Config>)`
  when `outputs` or `tuning` changed: the engine's target for new buses and
  new holders (L13) and the live tuning (§7). It changes no open bus.

### 5.3 Applying (engine)

- **L11** Engine quiet guard. A non-forced `ApplyAudioSystem`,
  `ApplyRoute` or `ApplyDevice` runs only when every bus it touches is
  quiet. Otherwise the engine keeps it (a newer action for the same
  target replaces it) and tries again each tick. This covers what the
  model does not see: a de-click or fade tail after Stop, and a test tone
  (at most 1.5 s). It reports nothing until it has run.
- **L12** `ApplyDevice { device, settings, force }` on an open bus, by
  what changed:

  | Change | Engine does |
  |---|---|
  | Rate, not bit-perfect | `Bus::reopen_with` at the new rate; then every source on the bus is opened again at its position at the new rate (as `follow_forced_rate` does): a paused one stays paused, a waiting one stays waiting. |
  | Rate, bit-perfect device | Nothing is reopened: the rate is only where the bus starts, and the next file sets it (B3). The value is stored. |
  | Buffer size | `reopen_with` with the new buffer at the running rate; sources stay as they are. |
  | Bit-perfect on | `reopen_with` with `exclusive: true` at the running rate; a refusal of exclusive access is handled as at start-up (B4). The next file sets the rate. |
  | Bit-perfect off | Leaves DSD if the bus carries it, then `reopen_with` shared at `effective_rate`; sources follow as for a rate change. The device's DSD mode stops applying (L4). |
  | DSD mode | If the bus has a DSD stream (`dsd_buses`: a paused or loaded DSD source), it leaves DSD and the DSD sources are opened again at their position under the new mode; otherwise the mode is stored for the next DSD source. |
  | DSD mix, DSD silence | Stored in the bus's DSD settings; read at the next DSD decision. No reopen. |

  Several changes in one action take a single `reopen_with`.
  `pcm_fallback` is set again from the new settings as `ensure_bus` does.
  The bus's mixer, clock and slots are kept. On success the engine
  reports `Applied { target: Device(device), wanted: settings, outcome:
  Ok }`.
- **L13** A holder or bus created after a change uses the current
  configuration: a player added, a first cart, or a device first used.
  A bus that no holder uses any more, and is quiet, is closed (releasing
  the device, which matters for exclusive access).
- **L14** Refusal. When the new stream does not open with `Unsupported`,
  or with `Busy` after the busy budget (`busy_budget(true)` on a quiet
  bus, none when forced on a sounding one), the engine reopens the
  running configuration and reports `outcome: Err(reason)`. The model
  keeps the running value and records `failures[target] = { wanted,
  reason }`. L8 does not try that wanted value again; it is tried again
  when the wanted value changes, or on **Apply now** (L20).
- **L15** Absent device. When the device is missing (`DeviceNotFound`,
  or `Lost` during the reopen), there is nothing to interrupt: the bus
  stays `Lost` with the **new** configuration, the watchdog opens it when
  it comes back (falling back to `pcm_fallback` if refused, as today), and
  the change is reported `Ok`. A device that disappears while an item
  waits changes nothing in L5: the item still waits on the holders'
  state, not on the device's health.
- **L16** `ApplyRoute { holder, route, force }` moves the holder's
  sources to the new target bus (opening it if needed): each source is
  opened again there at its position, paused stays paused, waiting stays
  waiting. The old bus is closed if unused (L13). A holder without a bus
  (the cartwall before its first cart, a player gaining a Cue route) only
  takes the route. A Cue route removed while that CUE is open (only when
  forced, L20) closes the CUE with the usual `CueEnded` or `CartCueEnded`.
  Reports `Placed` or `Unplaced`, then `Applied`.
- **L17** `ApplyAudioSystem { backend, force }` resolves the new default
  backend with `choose_default_backend` (moved from `main.rs` into the
  engine, with the same OS preference and availability list), re-places
  every holder whose route does not name a usable backend (the default
  output) as L16 does, closes unused buses, and reports
  `AudioSystemInUse` and `Applied`. Routes that name a backend keep it.

### 5.4 Limit reductions

- **L18** *(limits, players)* When `max_players` drops below
  `players.len()`, `players.count` is clamped at once (`Config::validate`)
  and the excess players are removed **from the end only**: while the last
  player is beyond the limit and removable (`transport == Stopped` and
  `cue.is_none()`), it is removed (`EngineAction::RemovePlayer`). A
  removable player before a non-removable one waits, so the players kept
  are always the first `max_players`. `update_config` no longer refuses with
  `PlayerBusy` for this. (Paused counts as not removable: removing it would
  lose what is paused on air, O6.)
- **L19** *(limits, cart grid)* For each page with `rows > max_cart_rows`
  or `cols > max_cart_cols`, the target is `(min(rows, max_rows),
  min(cols, max_cols))`. The page is resized (`resize_page`) when no cart
  of the page is playing or in CUE and no cart at an index ≥ target rows ×
  cols has a file. Otherwise it waits; `CartsBeyondGrid { page, count }`
  names how many carts with files must be cleared or moved first. Nothing
  drops a cart with a file by itself.

### 5.5 Apply now

- **L20** `Command::ApplySettingsNow` emits every pending output-side item
  (audio system, routes, devices, including failed ones) with `force:
  true`, ignoring causes. It never forces L18 or L19: those would stop
  audio for good or lose carts, not interrupt it briefly.
- **L21** Forced apply on a sounding bus: the busy budget is none, the
  stream gap lasts one reopen; sources that were playing are opened again
  at their position and resume as soon as they are ready (as after a
  seek); fades in progress and test tones are cut, and the model gets the
  same events as when they end (`FadeCompleted`, `ReachedEnd`). Nothing
  that was paused, stopped or loaded starts (rule 10).
- **L22** `fp_model::live::interruptions(state) -> Vec<(Target,
  Vec<BusyCause>)>`: the items `ApplySettingsNow` would force that have
  causes. The UI asks for confirmation if and only if this is not empty.

### 5.6 Other limits and persistence

- **L23** Every other limit applies at once (§6), and every increase
  applies at once.
- **L24** Nothing pending is saved. The configuration is saved as today;
  the next start opens everything with it, so nothing is pending after a
  start. A page still over the grid limit at the next start is repaired by
  the existing load path (`Cartwall::normalize`, which may drop carts
  beyond the largest grid with a warning); the notice says so (§9.2).

## 6. Limits

| Field | Today | Live behaviour |
|---|---|---|
| `max_players` | read by the reducer; reduction refused if a player beyond is busy | increase at once; reduction by L18 |
| `max_cart_rows`, `max_cart_cols` | read live by the reducer, Settings and the remote API; pages repaired only on load | increase at once (new grids may be larger); reduction by L19; `cartwall.default_rows/cols` clamped at once |
| `max_cover_bytes`, `max_cover_pixels` | copied into the analyzer, its cache and the remote's cache reader at start | services forward the new `Limits` to the analyzer (`Analyzer::update_limits`, like `update_settings`) and its cache; the remote reads them from the model snapshot per request. Applies to analyses started after; the library is not analysed again (cache entries keyed by the old limits simply miss). |
| `max_tag_chars`, `max_tag_values` | analyzer copy; tag editor reads live | as above for the analyzer; existing tags are not trimmed again |
| `max_state_file_bytes`, `backup_count` | read from `state.config` at each save | already live; no change |
| `max_playlist_file_bytes` | read at each import | already live; no change |
| `max_crash_reports` | the crash hook is armed before the configuration loads, with the default | the cap becomes an atomic set from the configuration after load and on each change |

## 7. Tuning

No tuning field waits. "Next use" means the value is read when the engine
next does that thing; work already scheduled keeps the value it was given.
Mixer values reach the callback only through a new `BusCommand::Tune(
MixerConfig)` (`MixerConfig` is `Copy`: no allocation; applied at the next
block start, replacing the mixer's config; a ramp in progress keeps its
length). A `Tune` that does not fit the command queue is sent again next
tick (only the latest matters).

| Field | Where it is read | Takes effect |
|---|---|---|
| `declick_ms` | engine fades (`frames_on`), and the mixer's `declick_frames` | engine: next use; mixer: `Tune` to every bus |
| `pause_ramp_ms` | engine, each Pause/Resume | next pause or resume |
| `prebuffer_secs` | ring size when a source is created | sources created after |
| `ready_threshold_ms` | carts at creation; players at `PlayerWorker::spawn` | sources created after; the worker gets it with each open request instead of at spawn |
| `mixer_headroom` | `ensure_bus` slot capacity | at once: capacity is recomputed for every bus and grown (`BusCommand::Grow`, allocated off the callback); a smaller value never shrinks slots |
| `max_commands_per_block` | mixer | `Tune` to every bus |
| `schedule_lead_ms` | engine when it schedules a transition | next schedule; a `Start` already sent keeps its frame |
| `conductor_tick_ms` | conductor loop (today fixed at spawn, `main.rs` ~L199) | next tick: the loop reads it from its state |
| `watchdog_timeout_ms`, `watchdog_startup_grace_ms`, `reconnect_interval_ms`, `device_busy_retries`, `device_busy_retry_ms` | `BusTiming` (conductor thread, not the callback) | at once: `Bus::set_timing` on every bus; the next `supervise` uses them |
| `gain_smoothing_ms` | mixer `volume_smoothing_frames` | `Tune` to every bus; next volume change |
| `save_debounce_ms`, `missing_recheck_ms` | services, from `state.config` | already live |
| `restart_handoff_ms` | the relaunch | removed with the relaunch (§9.4); lenient loading ignores the key |

Frame values in `Tune` are computed at each bus's running rate; a later
rate change converts them with the existing `Mixer::follow_rate`.

## 8. Bit-perfect and DSD

| Situation | Behaviour |
|---|---|
| Rate change on a bit-perfect device | Waits for the device to be idle like any device change (D3); when applied, nothing is reopened (L12). |
| Bit-perfect turned on while a paused track is loaded | Applied when idle (paused is idle): reopened exclusive at the running rate; the paused source stays paused; the next start follows its file's rate (B3) as usual. |
| Bit-perfect turned off on a device carrying DSD (paused DSD track) | Leaves DSD, reopens shared, reopens the DSD source as PCM at its position, still paused. |
| DSD mode change while a DSD track plays | Waits (the device is busy) or, on Apply now, leaves DSD and continues the track under the new mode after the gap. |
| `dsd_mix` / `dsd_silence_ms` change | Pending only for devices whose DSD mode is not PCM (L4); applied per device when idle; no reopen. |
| `HoldOthers` holding a source back | The held source is not audible; the DSD track is: the device is busy because of the DSD player. |

## 9. User interface

### 9.1 Where

- The top bar's "Restart pending" pill becomes **Settings pending**
  (amber), shown while `pending` is not empty. Its tooltip lists the items
  (one line each); clicking it opens the pending panel.
- The Settings footer shows the same notice and **Apply now**.
- The pending panel lists each item, its causes, and any failure.

### 9.2 Strings (en-US source; es-ES too)

| Key (indicative) | Text (described) |
|---|---|
| `top-settings-pending` | "Settings pending" |
| `settings-pending` | "Some changes wait until the outputs they affect are free." |
| `pending-device` | "{device}: {changes}" — changes as "sample rate 48 → 96 kHz", "buffer 512 → 256", "bit-perfect on", "DSD: DoP", "DSD mix", "DSD silence 200 → 400 ms" |
| `pending-route` | "{holder} → {device}" (holder: "P{n} Main", "P{n} CUE", "Cartwall Main", "Cartwall CUE") |
| `pending-audio-system` | "Audio system: {from} → {to}" |
| `pending-players` | "Players: {from} → {to}" |
| `pending-cart-page` | "Cart page “{name}”: {rows}×{cols} → {r}×{c}" |
| `pending-waiting` | "waiting for {causes}" — causes as "P{n} playing", "P{n} fading", "P{n} CUE", "P{n} paused", "cart “{name}” playing", "cart “{name}” CUE", "{count} carts beyond the new grid" |
| `pending-failed` | "{device} did not take {what}: {reason}. Still {running}." |
| `pending-cart-grid-on-load` | "If the application starts before this is applied, carts beyond the new grid are dropped." |
| `settings-apply-now` | "Apply now" |
| `apply-now-title` | "Apply now?" |
| `apply-now-body` | "Applying now briefly interrupts the audio on:" then one line per device: "{device}: {causes}" |
| `apply-now-limits` | "Player and cart page limits keep waiting until what they affect is free." (shown when such items exist) |
| `apply-now-confirm` | "Interrupt and apply" |

Device names come from the device list Settings already enumerates on a
helper thread (rule 8); an unknown id is shown as is.

### 9.3 Apply now

**Apply now** sends `Command::ApplySettingsNow` at once when
`interruptions` is empty (only failures to retry, or causes that ended
meanwhile). Otherwise it opens the confirmation modal: the list from
`interruptions`, **Cancel** (the default, also Esc and closing the modal)
and **Interrupt and apply**. If the list becomes empty while the modal is
open, the modal closes and the command is sent, as the exit guard does:
it then interrupts nothing (the operator's request still retries any
failed item).

### 9.4 Removed

`RestartReason`, `restart_pending`, `AppUi::with_started_config`, the
`ExitIntent::Restart` path, `begin_restart`, `restart_flag`,
`crates/fp-app/src/restart.rs`, `tuning.restart_handoff_ms`, and the keys
`top-restart-pending`, `tip-restart-pending`, `settings-restart-pending`,
`settings-restart-now`, `restart-reason-*`, `exit-guard-restart-*`,
`restart-failed`. The close guard (O6) is unchanged. The platform label
follows `live.audio_system_in_use`.

## 10. Remote control and MIDI

Station configuration stays a non-goal of the remote API (remote spec §1):
no endpoint, SSE event or OSC address is added, and neither the API nor
MIDI can apply settings. What a live change does is published by the
existing paths: a player removed by L18 as any change of the player
count, a page resized by L19 as a `cartwall` event. See Q3.

## 11. Testing

All engine tests use the `Offline` backend and explicit `now: Instant`;
nothing sleeps.

| Layer | Tests |
|---|---|
| `fp-model` (`tests/live.rs`) | One test per rule L1–L10, L18–L20, L22–L24: causes per holder state (paused not busy), device causes, route waits on its holder only, audio system waits on all, `device_settings` ignores DSD mix on a PCM device, `due` skips in-flight and failed values, order of actions, `ApplySettingsNow` forces only output items, player removal from the end only, page shrink blocked by a file beyond the grid, other limits at once. Placement is fed with synthetic `Placed` events. |
| `fp-engine` (`tests/live_settings.rs`) | L11: an apply waits for a fade tail and a test tone, then runs. L12: `OfflineDevice::config()` after each change kind; a paused source keeps its position and stays paused at the new rate; a bit-perfect rate change opens nothing (`open_attempts` unchanged). L13: an unused bus closes (`is_open` false). L14: `refuse_rate` → running config back, `Err` reported; `set_busy` beyond the budget → same. L15: `unplug` → bus `Lost` with the new config, `replug` → opens it. L16/L17: a holder moved keeps its paused source; an audio system switch moves only default-output holders. L21: forced apply on a playing source resumes it; nothing paused starts. |
| `fp-engine` (`tests/mixer.rs`, `audio_path_audit.rs`) | `BusCommand::Tune` changes the declick and smoothing lengths from the next block; the audit covers it (no allocation on the callback). `Bus::set_timing` changes the watchdog timeout. |
| `fp-engine` (`tests/conductor.rs`) | End to end: a rate change while P1 plays waits; Stop P1 → applied on the next tick; `conductor_tick_ms` change honoured. |
| `fp-app` kittests (`tests/settings.rs`, new `tests/live_settings.rs`) | The pill and footer list items with causes; Apply now with causes opens the modal (Cancel is the default and sends nothing); without causes it sends `ApplySettingsNow` directly; the modal closes when the causes end. |
| `fp-app` (`services`) | New limits reach the analyzer (`update_limits`). |

## 12. Documentation to update

- `docs/user/settings.md`: "Restart pending" becomes "Settings pending"
  (what waits, Apply now); Audio outputs ("changes wait for a restart"
  goes).
- `docs/user/getting-started.md` (~L95), `docs/user/bit-perfect.md` (~L84,
  ~L205: no restart needed).
- `docs/technical/architecture.md` (the restart section goes; live
  settings flow), `docs/technical/audio-engine.md` (`UpdateSettings`,
  `Apply*`, `Placed`, `Tune`, quiet guard, bus closing),
  `docs/technical/backends.md` (audio system switch).
- Specs: feedback-2 O4 marked "Replaced by the live settings spec";
  feedback-4 Q12.6 replaced; main spec §8.4 restart note; bit-perfect B6
  note.
- Both locales (§9.2, §9.4). `README.md` if its feature list mentions
  restarts.

## 13. Out of scope

- Editing limits or tuning in the UI, and reloading the configuration file
  while running: today nothing changes them while the application runs,
  so §6–§7 make the engine ready for such a path without adding one.
- Changing the player count from Settings (`set_player_count`): it keeps
  refusing a busy player, as today.
- The load-time repair of cart pages (L24).
- Multichannel layouts, and the bit-perfect rate change at transitions.
- Any remote or MIDI control of settings (§10).

## 14. Maintainer answers (2026-10-07)

- **Q1** Confirmed: a route change waits for its holder (the player or the
  cartwall it moves), not for the whole device (L6).
- **Q2** Confirmed: the relaunch is removed and no manual restart action is
  kept (§9.4); `restart_handoff_ms` goes with it.
- **Q3** Confirmed: the remote API does not expose pending items (§10).
- **Q4** Confirmed: a held (paused) CUE counts as busy (L1).
