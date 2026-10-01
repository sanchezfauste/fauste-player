# Fauste Player — Design Spec

- **Date:** 2026-09-25
- **Status:** Draft for review
- **Design source:** Claude Design project `ab44062f-ab67-4424-ae98-ed3e39f38ff8`, file `Reproductor v3.dc.html` (Nocturne design system), plus its design chat.
- **Scope of this document:** the global architecture for every phase, and the full detail of **Phase 1**. Later phases get their own spec → plan → implementation cycle and only their boundaries are fixed here.

---

## 1. Purpose

Fauste Player is a desktop **radio-automation playout application**: several independent players with playlists, a cartwall of instant-fire buttons, and per-player Main/Cue audio outputs. It is meant for **live on-air use**, where an audio dropout or a frozen player is a serious failure.

### Goals

- Linux, Windows and macOS from one Rust codebase.
- **Audio never stops or glitches because of the UI.** The UI can stall, repaint slowly or even panic without affecting playback.
- Every player has its own decoding thread; audio output runs on real-time threads that never allocate, block, perform I/O or panic.
- Support every relevant audio backend on each OS (delivered across phases, see §13).
- Faithful translation of the v3 design (layout, colours, density, behaviour) into a native Rust UI.

### Non-goals

- Android and iOS (not viable for this product; revisit later if ever).
- Network features, streaming, telemetry. (Local MIDI control surfaces are in scope: feedback spec §6. Remote control over the network is in scope: [remote control spec](2026-10-01-remote-control-design.md).)
- A music library browser or database (the design has none; files are added by drag & drop or a file dialog).
- Scheduling / clock-based automation (log import, time-fixed events) — not in the design.

### Language rule

All code, identifiers, comments, specs and plans are in **English**.

End-user UI strings are **English** by default and fully translatable:

- Every visible string lives in Fluent (`.ftl`) resources: `locales/en-US/` (source of truth) and `locales/es-ES/` (shipped translation).
- The language follows the OS locale (`sys-locale`) when a translation exists, can be overridden in Settings, and falls back per key to `en-US`.
- A test fails if any locale is missing a key that `en-US` defines, or uses an unknown one.
- Numbers, times and plurals go through Fluent, not string concatenation.

This spec quotes UI labels in English (the `en-US` values). The design's original Spanish labels ("Sonando", "SIG.", "Cartuchera", …) become the `es-ES` translation.

### Naming rule

Use the established terms of radio broadcasting and pro audio (broadcast practice, the Rust audio ecosystem) instead of inventing names.

### Glossary (design term → code term)

| Design (ES) | Code (EN) | Convention source |
|---|---|---|
| Player / reproductor | `Player` | broadcast playout practice; "deck" is DJ jargon |
| — (decoded stream feeding a mixer) | `Source` | `rodio::Source`, OpenAL source, Web Audio source node |
| Pista sonando (rojo) | current entry | |
| Siguiente (verde) | next entry | |
| Stop con fundido | fade stop | |
| Stop al final | stop after current (`stop_after_current`) | |
| Modo SINGLE / CONT | `PlayMode::Single` / `PlayMode::Continuous` | |
| Punto de mezcla (MIX) | `segue_start` marker (UI label "MIX") | broadcast "segue" cue point |
| Preescucha (CUE / PFL) | cue (pre-fade listen) | broadcast consoles |
| Cartuchera | `Cartwall`; one tab = `CartPage`; one button = `Cart` | broadcast "cart wall"; "cart" from broadcast tape cartridges |
| Salida Main / Cue | `Bus::Main` / `Bus::Cue` output route | mixing consoles |
| Bit perfect | bit-perfect mode | |
| Marcadores | `cue_in`, `intro_end`, `outro_start`, `segue_start`, `cue_out` | broadcast cue points |

---

## 2. Architecture overview

### 2.1 Cargo workspace

| Crate | Responsibility | `unsafe` |
|---|---|---|
| `fp-model` | Domain types and pure logic: tracks, playlists, player state machine, cart pages, config. No I/O, no threads. | `forbid` |
| `fp-engine` | Audio engine: decoding, resampling, sources, bus mixers, fades, the **conductor** (control thread), telemetry. | `forbid` |
| `fp-backends` | `AudioBackend` trait and one module per backend, plus `Null` and `Offline` backends. | allowed, isolated here only |
| `fp-analysis` | Tags, cover art, waveform peaks, automatic markers, analysis cache. | `forbid` |
| `fp-store` | Persistence (atomic JSON), schema migrations, M3U/M3U8/PLS, cart-page import/export. | `forbid` |
| `fp-app` | egui UI; produces the `fauste-player` binary. | `forbid` |

Dependency direction: `fp-app → fp-engine → {fp-backends, fp-model}`, `fp-app → {fp-store, fp-analysis} → fp-model`. `fp-model` depends on nothing internal. Rust stable, edition 2024.

### 2.2 Threads

In priority order:

1. **Bus output threads** (one per open device stream). Created by the backend with real-time priority (SCHED_FIFO/rtkit on Linux, MMCSS "Pro Audio" on Windows, time-constraint policy on macOS). They run the bus **mixer**: sum the sources routed to the bus, apply per-sample gain ramps, update atomic telemetry. They must never allocate, free, block, do I/O, log or panic.
2. **Player decoder threads** (one per player; later one for the cartwall). Each decodes all sources of its player (current, incoming, cue) with `symphonia`, resamples with `rubato` when the file rate ≠ bus rate, and fills each source's lock-free SPSC ring buffer (`rtrb`). Elevated (non-RT) priority.
3. **Conductor thread.** Single owner of the authoritative application state (`fp-model` state + engine source bookkeeping). It consumes UI commands, reacts to engine events, applies player logic, schedules sample-accurate actions on buses, publishes snapshots to the UI and requests persistence.
4. **Background worker pool** (low priority, 2 threads). Analysis jobs and persistence writes.
5. **UI thread** (egui main thread). Reads snapshots, sends commands. Holds no lock that any other thread waits on.

### 2.3 Communication rules

- UI → conductor: bounded `crossbeam-channel` of `Command`. On a full channel the UI drops the command and logs it (never blocks the UI).
- Conductor → UI:
  - **Model snapshot:** an immutable `Arc<AppSnapshot>` replaced via `arc-swap` whenever state changes.
  - **Live telemetry:** positions, VU levels, bus health, read directly from `Arc<…Telemetry>` structs made of atomics.
- Conductor ↔ bus mixer: an `rtrb` SPSC queue of `BusCommand` towards the mixer and an `rtrb` queue of `BusEvent` back. Retired sources go back through a **garbage queue** so they are dropped (freed) on the conductor thread, never on the RT thread.
- Conductor ↔ decoder threads: `crossbeam-channel` (non-RT on both sides).
- **The only lock touched by an RT thread** is the bus-mixer handoff mutex (§4.7), and the RT thread only ever calls `try_lock` on it. On contention it outputs silence for that block and increments a counter. It never waits.

### 2.4 Configuration, limits and extensibility

**No hardcoded product limits.** Anything a user or operator might reasonably want to change is configuration. Values are grouped by audience:

| Group (in `config.json`) | Audience | Examples | Where edited |
|---|---|---|---|
| `players`, `analysis`, `outputs`, `ui` | operator | player count, `fade_ms`, auto-segue, trim threshold and margin, segue drop, routes, wave colour | Settings UI |
| `limits` | resource guards | `max_players` (16), `max_cover_bytes`, `max_state_file_bytes`, `backup_count` | config file only |
| `tuning` | engine internals | `declick_ms`, `pause_ramp_ms`, `prebuffer_secs`, `ready_threshold_ms`, `mixer_headroom`, `max_commands_per_block`, `schedule_lead_ms`, `conductor_tick_ms`, `watchdog_timeout_ms`, `reconnect_interval_ms`, `gain_smoothing_ms`, `save_debounce_ms` | config file only (an "advanced" section) |

- Every group is a typed struct with `Default` values documented in code. They are validated on load: out-of-range values are clamped to the valid range with a logged warning, never rejected with a crash.
- Engine capacities (mixer slots, scratch buffers) are **derived** from the configuration at the moment a mixer is built (§4.3), never fixed constants.
- The only true constants are physical or format facts (e.g. the number of channels in a stereo frame).

**Designed for growth:**

- Players, playlists, cart pages and routes are identified by opaque ids (`PlayerId`, `PlaylistId`, …) in growable collections. Nothing indexes a fixed-size array by player number.
- The mixer mixes `Source`s without knowing their origin (player, cue, cart, test tone). A future origin (line input, network stream, voice-track recorder) only needs a new producer that feeds a `Source`.
- `Command`, `EngineAction` and `BusCommand` are enums in one place each. A feature adds variants and handlers, and the compiler's exhaustiveness checks show every place that must react.
- Persisted formats carry `schema_version` with migrations (§7), so new fields never break old files.

---

## 3. Player behaviour (normative)

These rules come from the design chat. Each one becomes an `fp-model` unit test.

### 3.1 State

A player has:

- `playlist`: the playlist shown in the player's tab strip.
- `current: Option<EntryId>`.
- `next: Option<EntryId>`.
- `transport ∈ {Stopped, Playing, Paused}` and a `fading` flag.
- `mode ∈ {Single, Continuous}`.
- `stop_after_current: bool`.
- `cue: Option<CueState>`.
- `volume` (linear 0.0–1.0, **default 1.0**; values outside are clamped, and a NaN is ignored so it never silences what is on air).

### 3.2 Rules

1. **Entry colours.** In each player's table, that player's current entry is shown red and its next green; an entry on air on another player is marked with that player's number ("P2"), not highlighted. Entries that player has played are dimmed. "Played" is kept per entry and per player, and persists.
2. **Double-click** on an entry sets `next` to that entry. This is not allowed on that player's own current entry. It is allowed while `stop_after_current` is on, and it does **not** clear that flag.
3. **Play while Stopped.** If `next` exists, it becomes `current` and starts; `next` becomes the entry after it. If there is no `next`, nothing happens.
4. **Play while Paused** resumes.
5. **Play while Playing.** If `next` exists and no fade is running:
   - the current source fades out over `fade_ms` (default 1000 ms, configurable);
   - the next source starts immediately at full level (overlap);
   - the Play button shows the fast-forward icon and "SIG." while playing.
   If there is no `next`, nothing happens.
6. **Pause** toggles Playing ⇄ Paused. Pausing applies an anti-click ramp (`tuning.pause_ramp_ms`, default 10 ms) and keeps the position. While paused, the Pause button blinks amber every 500 ms.
7. **Stop:**
   - applies an anti-click ramp (`tuning.declick_ms`, default 5 ms) and stops;
   - marks the current entry played and sets `current = None`;
   - `next` keeps any explicitly set value, otherwise becomes the entry after the stopped one;
   - clears `stop_after_current`.
8. **Fade stop** ramps to 0 over `fade_ms`, then applies Stop (rule 7).
9. **Stop after current:**
   - only available in `Continuous`; disabled (dimmed, with an explanatory tooltip) in `Single`;
   - switching to `Single` clears it;
   - when the current track reaches its end, apply Stop (rule 7);
   - the green `next` stays marked.
10. **Single mode.** At the end of the current track, apply Stop (rule 7). There is no automatic mix.
11. **Continuous mode.** Let `cur` be the current track:
    - If auto-segue is enabled, `cur` has a segue start, `next` exists and `stop_after_current` is off: at `cur.segue_start` the next source starts (sample-accurate) and `cur` fades from its segue start to its `cue_out` (overlap).
    - Otherwise, at `cur.cue_out` the next starts immediately (no gap) if `next` exists, else Stop.
12. **Advance** happens whenever the next becomes current:
    - the old current is marked played by this player;
    - `current = next`;
    - `next` = the entry after the new current in its playlist, skipping entries whose file state is `Missing` or `Unreadable`.
13. **Entry removal.** An entry that is current on any player cannot be removed. If the removed entry was a player's `next`, that player's `next` becomes the entry after it.
14. **Next across tabs.** Switching the displayed playlist tab does not change `current` or `next`. Advancing follows the playlist that contains the new current entry.
15. **Cue:**
    - toggling CUE pre-listens the player's `next` (or a specific entry chosen from the context menu) on the player's Cue output, from its `cue_in`;
    - it stops at the end of the track or on toggle;
    - it shows elapsed cue time in blue;
    - it never affects the Main output.
16. **Keyboard.** Keys `1`–`9` press Play on players 1–9 (shortcuts become remappable in Phase 2). `Delete`/`Backspace` removes the selected entry, subject to rule 13. `Esc` closes menus and dialogs.
17. **Countdown** shows `-remaining` to `cue_out`, with tenths, and `elapsed / total` on the row under the waveform. During the last `end_warning_secs` (default 10) the countdown turns red.
18. **Intro indicator.**
    - Shown only if the track has a manual `intro_end`.
    - While `position < intro_end`, a blue "INTRO nn.n" badge counts down, and it blinks during the last 3 s.
    - The waveform shades the intro region blue.
19. **Outro indicator.** When `position ≥ outro_start`, an amber badge counts down to `cue_out`. The waveform shades the outro region warm.
20. **Playlist footer:** `-remaining | elapsed / total` for the whole playlist, in the same format as the current track, from this player's point of view (its played entries and its current position).
21. **Player count** is configurable at runtime (default 4, minimum 1). There is no architectural maximum: players are identified by `PlayerId` and stored in growable collections. Config validation caps the count at `limits.max_players` (default 16) only as a resource guard. Reducing the count is refused while a player that would be removed is playing. A configuration update is validated like a loaded one; a lower `limits.max_players` removes the idle players above it, and is refused while one of them is busy.
22. **Players are independent.** A player's `current`, `next` and played marks change only through that player's own transport commands and engine events. Several players may show the same playlist, each at its own position, and the same entry may be on air on several players at once. Two things still look across players: an entry on air on any player cannot be removed (rule 13), and edits to a playlist (insert, move, remove) or a file becoming unreadable re-derive the next of every player that shows it (rules 12 and 13). (This replaces the Phase 1 rule "no duplicate on air by default", which moved every other player's next whenever one player started an entry.)
23. **Restart** (feedback spec R23): with a current entry, not Stopped and no fade stop running, seek to its `cue_in` through the anti-click seek path; a paused player stays paused. Otherwise nothing happens.
24. **Previous** (R24): only while Playing and not fading. Entries are popped from the player's `history` until one still exists, is playable (not Missing or Unreadable) and is not the current entry (the others are discarded); if none is left, nothing happens. The popped entry starts at full level while the current one fades out over `fade_ms`, exactly as Play-while-Playing (rule 5). The entry left is marked played but not recorded in the history, and becomes the explicit `next`.
25. **History** (R25): every advance (a transition, Play-while-Playing) and every stop records the entry left in the player's `history`, dropping the oldest beyond `players.history_len` (default 50, 0–1000; 0 disables Previous). It is saved in `session.json`; one that does not parse loads empty, and entries that no longer exist are dropped on restore. Removing an entry does not edit histories; stale entries are skipped by Previous.
26. **Availability** (R28): `fp_model::availability` says which transport actions make sense now — Play/Next: paused with a current entry, or a next exists and no fade runs; Pause: playing and not fading, or paused; Stop: a current entry; Fade stop: playing and no fade stop running; Restart: a current entry, not Stopped and no fade stop running; Previous: playing, not fading, and a playable history entry; Stop after current: continuous mode; Cue: a next exists or a cue runs. The UI dims the others, and shortcuts ignore them; `apply` keeps its own guards.
27. **Repeat this entry** (feedback spec R26): while the current entry has `repeat` (and not `stop_after`), and no stop-after-current or fade stop applies, the player preloads that same entry at its cue-in and plans a hard transition into it at cue-out (`StartNextAt { at_secs: cue_out, fade_current_until_secs: None }`): a gapless, sample-accurate restart, in Single and Continuous mode. Each pass reports `TransitionStarted` for the same entry; it stays current, is not marked played and is not recorded in the history. Pause keeps the repeat; Play (next), Previous, Stop, Fade stop and stop-after-current end it; toggling it while it plays re-plans.
28. **Stop after this entry** (R27): while the current entry has `stop_after`, the plan is `StopAt { cue_out }` in any mode; at the end Stop (rule 7) applies. The flag belongs to the entry, is saved with the playlist, acts every time the entry plays, and wins over `repeat`.

---

## 4. Audio engine (`fp-engine`)

### 4.1 Source

A `Source` is one decoded stream of one file. It has two halves:

- a **producer half** owned by the player decoder thread;
- an **RT half** owned by a bus mixer slot, containing:
  - the `rtrb::Consumer<f32>` of an interleaved stereo ring at the bus rate;
  - a preallocated gain ramp state;
  - a `SourceShared` telemetry block of atomics: state, position in source frames, peak L/R, underrun count.

Source states: `Prebuffering → Ready → Playing ⇄ Paused → Finished`, plus `Failed` for decode errors.

The ring capacity is `tuning.prebuffer_secs` (default 5 s) of audio. The decoder keeps the ring full, and a source becomes `Ready` once it holds `tuning.ready_threshold_ms` (default 500 ms).

**Seek** is implemented as *replace source*. A new source is prepared at the target position, the mixer crossfades old → new over `tuning.declick_ms`, and the old source is retired. This avoids stale-sample problems in the ring.

### 4.2 Mono and multichannel sources

Mono files are duplicated to L/R. Files with more than 2 channels are downmixed to stereo with standard ITU-R BS.775 coefficients. In Phase 1 all buses are stereo. The bus channel count is a field of the bus config, not a constant, so multichannel buses can be added later without changing the mixer interface.

### 4.3 Bus mixer

One `Mixer` per open output stream:

- a slot array preallocated when the mixer is built, with **capacity derived from the routing** rather than a constant. The capacity is the sum, over everything routed to the bus, of the sources it can hold at once (2 per player on Main, 1 per player on Cue, the cart count of each routed cart page, plus test tones), multiplied by `tuning.mixer_headroom` (default 2×);
- a preallocated scratch buffer sized for the backend's maximum block, as reported when the stream opens;
- a `BusShared` telemetry block: heartbeat counter, frames rendered, xruns, peak.

When a configuration change needs more capacity (more players, a bigger cart page, a new route), the conductor builds a new, larger `Mixer` off the RT thread, moves the live sources into it, and swaps it in through the handoff (§4.7). No allocation ever happens on the RT thread. If a source cannot be attached because every slot is busy, which should be impossible with the derived capacity, the attach is refused, logged and surfaced in the UI. It is never silently dropped.

For each block the mixer:

1. Drains `BusCommand`s, up to `tuning.max_commands_per_block` (default 256) so a flood cannot overrun a block; the rest wait for the next block.
2. For each active slot:
   - pops samples;
   - applies the per-sample gain as a linear ramp in dB-space for fades (equal-power curves for crossfades), multiplied by the player volume (smoothed over `tuning.gain_smoothing_ms`, default 20 ms);
   - accumulates into the output.
3. On ring underrun, outputs silence for the missing samples and increments the source's underrun counter.
4. Writes the output to the route's channel pair, adds the heartbeat and emits `BusEvent`s.

`BusCommand` variants (all timestamps in bus frames):

- `Attach { slot, source }`
- `Start { slot, at_frame }`
- `Ramp { slot, to_gain, over_frames, at_frame }`
- `StopAt { slot, at_frame }`
- `Pause { slot }`
- `Resume { slot }`
- `Detach { slot }`

`BusEvent` variants:

- `Started { slot, frame }`
- `Finished { slot, frame }` — ring drained after the decoder signalled EOF
- `Detached { slot, source }` — the source is returned for dropping

**Sample-accurate scheduling.** The conductor tracks each bus's `frames_rendered`. It schedules the segue start (`Start { at_frame }`) and fades ahead of time, at least `tuning.schedule_lead_ms` (default 200 ms) before they are due. The result does not depend on when the conductor thread happens to wake.

### 4.4 Conductor

- Event loop: `crossbeam::select!` over the UI command channel and decoder events, with a `tuning.conductor_tick_ms` (default 5 ms) tick that drains `BusEvent` queues and updates deadlines.
- It owns an `fp-model::AppState` and applies §3 through pure functions of `fp-model` that return the new state plus a list of `EngineAction`s (prepare source, start at, ramp, stop, cue…). This keeps all behaviour testable without audio.
- **Preload.** Whenever a player's `next` changes, the conductor asks the player decoder to prepare a source for it, positioned at `cue_in`. Play therefore starts within one block.
- It publishes an `AppSnapshot` after each state change and requests a debounced save (§7).

### 4.5 Player decoder thread

- It owns the producer halves of every source of its player, currently current, incoming (overlap or next-preload) and cue. It holds them in a growable collection, not fixed fields, so future per-player sources (e.g. a second pre-listen or a jingle overlay) need no restructuring.
- It fills rings round-robin, prioritising the source with the least buffered audio.
- It is supervised: its loop runs inside `catch_unwind`. After a panic or a decode error, the affected source goes to `Failed` and the conductor is notified. The conductor marks the entry `Unreadable`, skips to the following entry per §3 (rule 12), and restarts the thread if it died.
- File I/O errors, including a file that disappears mid-play, map to `Failed` the same way. Audio already buffered in the ring keeps playing.

### 4.6 Output routes

A route is `(backend_id, device_id, channel_pair)`. Every player has a Main route and a Cue route; the cartwall has its own Main and Cue routes (Phase 2).

A multichannel device is opened **once**. All routes to it share one stream and one mixer, and each source slot writes to its route's channel pair. Routes that share a device and channel pair are summed.

### 4.7 Device loss and the virtual clock

- A backend stream error, or a missing heartbeat for `tuning.watchdog_timeout_ms` (default 500 ms, watchdog in the conductor), marks the bus `Lost`.
- The mixer lives in `Arc<Mutex<Mixer>>`. The RT callback only `try_lock`s it (§2.3). On loss, a **virtual-clock thread** takes over: it calls the same `Mixer::render` into a discard buffer at real-time pace. Player timelines (countdowns, segue points, chaining) therefore keep progressing, and the automation never stalls.
- The conductor retries opening the device every `tuning.reconnect_interval_ms` (default 2000 ms). On success the new RT stream takes the mixer back, the sources continue from their current positions, and the bus returns to `Ok`.
- The UI shows a red alert on affected players and in the status bar while a bus is `Lost`.

### 4.8 Sample rate

In Phase 1 (shared mode) each device stream runs at the configured rate (default 48 kHz; Settings offers the rates the device reports as supported) and every source is resampled to it with `rubato` (sinc, high quality). The buffer size (default 512 frames; Settings offers the range the device reports) is requested from the backend. The reported latency is shown in Settings.

Bit-perfect mode (no resampling, no processing, file rate) is Phase 4.

---

## 5. Audio backends (`fp-backends`)

### 5.1 Trait

```rust
pub trait AudioBackend: Send + Sync {
    fn id(&self) -> BackendId;
    fn availability(&self) -> Availability; // Available | Unavailable(reason)
    fn enumerate_devices(&self) -> Result<Vec<DeviceInfo>, BackendError>;
    fn open_output(
        &self,
        device: &DeviceId,
        config: StreamConfig,          // sample rate, buffer frames, channels, share mode
        renderer: Box<dyn Renderer>,   // called on the RT thread
    ) -> Result<Box<dyn OutputStream>, BackendError>;
    fn subscribe_device_changes(&self, tx: Sender<DeviceChange>) -> Result<(), BackendError>;
}

pub trait Renderer: Send + 'static {
    /// Interleaved f32; the backend converts to the device format.
    fn render(&mut self, out: &mut [f32], channels: usize);
    fn on_error(&mut self, err: StreamErrorKind); // must be RT-safe: sets an atomic flag
}
```

`DeviceInfo` carries the name, channel count, supported rates and sample formats, `exclusive_capable` and `rate_switching` flags.

### 5.2 Backend matrix

| OS | Backend | Phase | Implementation |
|---|---|---|---|
| all | `Null` (real-time pacing, discards) | 1 | own |
| all | `Offline` (renders to memory on a simulated clock; tests) | 1 | own |
| Linux | ALSA | 1 | `cpal` |
| Linux | PipeWire | 3 | `cpal` `pipewire` host (pipewire-rs) |
| Linux | PulseAudio | 3 | `cpal` `pulseaudio` host (pure-Rust client) |
| Linux, macOS, Windows | JACK | 3 | `cpal` `jack` host (dynamically loaded on macOS and Windows) |
| Windows | WASAPI shared | 1 | `cpal` |
| Windows | WASAPI exclusive | 4 | with bit-perfect output (exclusive access only matters there) |
| Windows | ASIO | 3 | `cpal` `asio` host (needs the Steinberg SDK at build time via `CPAL_ASIO_DIR`) |
| macOS | Core Audio | 1 | `cpal` |
| macOS | Core Audio hog mode (exclusive) | 4 | with bit-perfect output |

DirectSound is not supported. Microsoft deprecated it, WASAPI supersedes it,
and it would need unsafe FFI for no benefit (ruling taken when Phase 3 was
planned). Every Phase 3 backend comes through cpal's safe API, so the
workspace keeps `forbid(unsafe_code)`.

### 5.3 Availability and linking

Each backend is behind a Cargo feature and is registered at startup. A backend whose system library is missing reports `Unavailable(reason)`. Settings shows it as unavailable, and the app still starts.

- JACK is loaded with `dlopen`.
- ALSA is always linked on Linux.
- PipeWire and PulseAudio are linked, and the Linux packages declare them as dependencies (Phase 5).
- If the ASIO SDK is absent at build time, the Windows binary is built without ASIO. This is logged at build time and visible in Settings.

---

## 6. Analysis (`fp-analysis`)

Jobs run on the background pool at low priority, one file at a time per worker, and can be cancelled.

- **Tags** via `lofty`: title, artist, album, duration. Fallback: parse `Artist - Title` from the file name; otherwise the artist is empty and the UI shows the localised "Unknown artist".
- **Cover art** via `lofty` + `image`:
  - rejects images larger than `limits.max_cover_bytes` (default 20 MB) or `limits.max_cover_pixels` (default 8000×8000);
  - decodes with explicit `image::Limits`;
  - stores a thumbnail (default 128×128, `analysis.cover_thumb_px`) in the cache.
- **Peaks:** full decode, mono sum; per bucket (`analysis.peak_bucket_ms`, default 10 ms) the minimum, the maximum and the RMS level, stored as `i16` (full scale = `i16::MAX`).
- **Loudness envelope:** RMS over `analysis.rms_window_ms` windows (default 50 ms; in-memory only during analysis).
- **Automatic markers** (seconds, `f64`). Every threshold below is a field of `AnalysisSettings` (in `config.json`, editable in Settings), not a constant:
  - `cue_in`: the start of the first `peak_bucket_ms` bucket whose **stereo peak** (the largest absolute sample of either channel) is ≥ `trim_threshold_db` (default −60 dBFS), moved back by `trim_margin_ms` (default 20 ms, clamped at 0); 0 when nothing reaches it. Trimming never removes audio: no bucket at or above the threshold lies outside `[cue_in, cue_out]` (feedback spec §4.2).
  - `cue_out`: the end of the last such bucket plus `trim_margin_ms` (clamped at the duration), or the duration.
  - `segue_start`: the end of the last RMS window of the body (the windows between `cue_in` and `cue_out`) at or above the body's median RMS − `segue_drop_db` (default 15 dB), clamped to `[cue_out − segue_max_secs, cue_out]` (default `segue_max_secs` = 4) and to `≥ cue_in`. Relative to the track, so loud and quiet masters with the same fade get the same overlap. Tracks shorter than `markers_min_duration_secs` (default 60 s, as in the design) get no segue start.
  - `outro_start`: scanning backwards from `cue_out`, the end of the first window whose RMS ≥ (median track RMS − `outro_drop_db`, default 6 dB), clamped to `≥ cue_out − outro_max_secs` (default 30 s). Tracks shorter than `markers_min_duration_secs` get no outro.
  - `intro_end`: **never automatic.** It is manual only (Phase 2 marker editing, plus the Phase 2 tag convention `INTRO=<seconds>` as TXXX/Vorbis comment). If unset, no intro is shown.
- **Marker provenance:** every marker is `Auto` or `Manual`. Manual markers are never overwritten by re-analysis.
- **Cache:**
  - `postcard`-encoded file per track in the OS cache dir;
  - the key is a hash of (canonical path, size, mtime) plus the analysis version;
  - a corrupt cache entry is discarded and recomputed;
  - entries of older analysis versions are removed by the analysis pool, off the start-up path;
  - tracks an earlier version analysed (an older `analysis_version`, or no format) keep that analysis until the operator asks: at start a notice gives their number with **Analyse now** and **Later**, and Settings → Analysis offers the same. Only tracks on screen, which need their waveform, are analysed at once. Re-analysing a library costs the processor for a while on an on-air machine.
- **Playability before analysis.** A track can be played before its analysis finishes. Until then it has no waveform or segue start, and `cue_in = 0`, `cue_out = duration`. If analysis finishes while the track is current or next, its markers apply to scheduling that has not happened yet.

Supported formats: WAV, AIFF, CAF, FLAC, MP1/2/3, AAC/M4A, ALAC, Ogg Vorbis, Opus, Matroska/WebM audio, WavPack, Monkey's Audio and DSD (DSF, DSDIFF). The extensions and decoders are in the [audio formats spec](2026-09-27-audio-formats-design.md) F1 (Phase 1 had symphonia's formats only).

---

## 7. Persistence (`fp-store`)

- **Location:** from `directories::ProjectDirs` (XDG on Linux, `%APPDATA%` on Windows, `~/Library/Application Support` on macOS).
- **Files:**
  - `config.json`: settings and routes;
  - `playlists.json`: playlists, entries, played flags, track references and manual markers;
  - `session.json`: per-player playlist, current, next, mode, stop-after, position and column widths;
  - `carts.json`: Phase 2.
- **Format:** `serde_json` with a top-level `schema_version`. Migrations are explicit functions `vN → vN+1`, each with tests.
- **Atomic write:**
  1. write to `file.tmp`;
  2. `fsync` it;
  3. rotate `file.json → file.json.bak1 → … → .bakN` (`limits.backup_count`, default 3);
  4. `rename(tmp, file.json)`;
  5. `fsync` the directory (Unix).
- **Load:** on parse error or failed validation, try `.bak1`…`.bakN` in order, show a non-blocking warning and log it. If everything fails, start with defaults and keep the corrupt files renamed `*.corrupt-<timestamp>`. **Never crash on bad data.**
- **Autosave:** the conductor marks state dirty; saves are debounced by `tuning.save_debounce_ms` (default 1000 ms) and run on the background pool. `session.json` is refreshed at the same interval while any player plays.
- **Crash recovery.** On startup, players restore their playlist, current, next and position, but always come up **Stopped/Paused**. Nothing goes on air by itself. A saved position at or past the track's cue-out (known from analysis) is restored at its cue-in: resumed there, the track would end at once (Stop in Single mode, the next track in Continuous).
- **Limits:** input files larger than `limits.max_state_file_bytes` (default 50 MB, JSON) or `limits.max_playlist_file_bytes` (default 10 MB, M3U/PLS) are rejected with a clear error.
- **M3U/M3U8 and PLS:** Phase 2. A tolerant parser that ignores unknown lines, handles relative paths (resolved against the playlist's directory), non-UTF-8 paths and `#EXTINF`. It is fuzzed.

---

## 8. UI (`fp-app`)

### 8.1 Framework and theme

- `eframe`/`egui` with the `wgpu` renderer (falling back to `glow` if wgpu initialisation fails).
- **Native window decorations.** The in-app top bar keeps the app name with the version (a click opens the About window: copyright, bundled licence notices, third-party notices file), the "Settings" button and the clock.
- `theme` module: Nocturne tokens as constants. The design's `oklch` values are converted once to sRGB and stored as precomputed constants, with a unit test that checks the conversion. Rounding is 0 and spacing follows the design.
- **Fonts:** Inter embedded (OFL); Phosphor icons via `egui-phosphor`.
- **Player-specific icons** (fade stop, stop after current) are drawn as vector shapes, as in the design.
- **Waveform colours:** Violet, Amber, Cyan, White, Orange, Magenta, Ice, Sand, Slate (default **Slate**). They are a named palette in the theme, so more can be added without code changes elsewhere.
- **Repaint policy:** at display rate while anything plays or a fade runs; otherwise every 100 ms (clock).

### 8.2 Data flow

Each frame the UI loads `Arc<AppSnapshot>` (via `arc-swap`) and reads telemetry atomics. User actions become `Command`s. The UI keeps only view state locally: hover, drag in progress, open menu, selection and scroll.

### 8.3 Main screen (matches v3)

- **Player column**, one per configured player (min width 380 px; any number of players is laid out in a horizontally scrollable row):
  - **Header:** `P1`…`Pn`, state dot and label ("On air", "Stopped", "Paused"), fade and stop-after badges, BP badge (inactive until Phase 4), SINGLE|CONT segmented control (one border, the active mode filled), CUE button.
  - **Info row:** 64 px cover (placeholder vinyl icon if none), title, artist, and the next line with the green square (plus cue time in blue when cueing).
  - **Meter column** at the right, spanning the info row and the transport: labelled dB scale, stereo meter with reference lines and peak hold (meters spec M4), vertical volume fader (drag + wheel, dB tooltip).
  - **Transport:**
    - Play/NEXT button spanning 2 rows;
    - a 3×2 grid, 6 px gaps, all equal size: Previous, Stop, Pause on top; Restart, Fade stop, Stop-after-current below. Buttons whose action is unavailable (rule 26) are dimmed and inert, the Play button and the header's CUE included;
    - big countdown with tenths.
  - **Waveform:**
    - played/unplayed colours, intro/outro shading, dashed amber MIX marker, playhead;
    - drawn continuously, one column per pixel, as audio editors draw it: the peak envelope in the colour dimmed, and the RMS level of the same span as a solid body inside it. On a loud master the peaks fill the height but the body still shows the track's dynamics. Both are linear in amplitude and symmetric about the centre line; a column's peak is the largest of its buckets, its RMS the root of their mean square;
    - the trimmed head and tail (before `cue_in`, after `cue_out`) dimmed, with 1 px lines at the cue points;
    - hover time tooltip, click to seek; press-and-drag previews and seeks on release inside (outside or Esc cancels; Alt-drag edits markers);
    - wheel zoom around the pointer down to one bucket per pixel, Shift or sideways wheel pans, a "Full view" button while zoomed; the view follows the playhead unless moved within `ui.follow_current_grace_secs`, and resets on a new entry (feedback spec §3.3);
    - intro and outro badges per §3 (rules 18 and 19).
  - `elapsed / total` on a row under the waveform, right-aligned.
  - **Playlist tabs:** reordering and dropping entries on a tab appends them.
  - **Track table:**
    - `egui_extras::TableBuilder` with resizable `#`, Title, Artist and Duration columns (resize handles padded away from labels); widths are stored per player as fractions of the table and laid out every frame, so the columns fill the table and keep their proportions on resize (`#` and Duration have content minimums, Title:Artist default 60:40);
    - follows the current entry: when it changes and the table and tabs were not used within `ui.follow_current_grace_secs` (0 = never), the tab of its playlist is shown and its row scrolled to the top; a drag or open row menu makes it wait (feedback spec F18);
    - virtualised rows;
    - `#` zero-padded to the digit count of the playlist length (3 digits for ≥ 100 entries);
    - this player's current row red with a speaker icon, its next row green with an arrow icon, entries on air on another player marked "P<n>", rows this player has played dimmed, missing/unreadable rows with a warning icon; entries marked to repeat or to stop after show a repeat icon or the stop-after icon at the right of the title, in the row's text colour, and the row menu has checkable "Repeat this track" and "Stop after this track".
  - **Footer:** "+ Add" (native file dialog via `rfd`, defaulting to the music folder), entry count, playlist times (§3, rule 20).
- **Interactions:**
  - single click selects, double click sets next;
  - context menu: Play now, Set as next, Pre-listen on CUE, Add tracks below…, Duplicate, Move to ▸, Remove from playlist;
  - drag & drop within a list, to another player's list and onto tabs, with a violet drop indicator line;
  - OS file drops onto a list insert at the drop position.
- **Status bar:** shortcut hints, legend (On air, Next, Intro, Segue), backend/OS label, bus alerts.
- **Cartwall:** Phase 2 (hidden in Phase 1).

### 8.4 Settings (Phase 1 subset)

A modal window, closed with `Esc` or "Close", with these sections:

- **Audio outputs:**
  - backend (Phase 1: the single real backend of the OS, plus Null);
  - sample rate and buffer size, with the computed latency;
  - per player, Main and Cue device and channel pair;
  - "Test Main" / "Test Cue" buttons that play a test tone (defaults: 1 kHz / 440 Hz, 1.5 s, −18 dBFS) through the real engine route.
- **Players:** player count, default mode, `fade_ms`, auto-segue on/off, `end_warning_secs` (`history_len` is set in `config.json`).
- **Analysis:** every `AnalysisSettings` field (trim threshold and margin, segue drop and max seconds, outro drop and max seconds, minimum duration for markers), with a "Re-analyse all" action. Manual markers are kept.
- **Playlists:** music folder, and create/rename/delete playlists. The last playlist cannot be deleted, nor a playlist containing a current entry.

The Cartwall and Shortcuts sections, plus M3U import/export and the language selector, come in Phase 2. In Phase 1 the language follows the OS locale.

### 8.5 Panic isolation

Each `App::update` body runs inside `catch_unwind(AssertUnwindSafe(...))`. On panic:

- it logs the panic;
- it sets a `ui_degraded` flag that renders a minimal error banner with a "Restart interface" button, which rebuilds the UI view state from the current snapshot;
- it continues.

The engine and conductor are unaffected. `panic = "unwind"` is required in all profiles.

---

## 9. Errors and logging

- Libraries use `thiserror` enums; the binary uses `anyhow` at the edges.
- No `unwrap`/`expect` in production paths. Clippy enforces `unwrap_used`, `expect_used`, `panic`, `indexing_slicing` in `fp-engine` and `fp-backends`.
- `tracing` with `tracing-appender` (non-blocking, daily rotation, 14 files kept) in the OS log dir, plus stderr in debug builds.
- RT threads never log. They increment atomic counters, and the conductor turns counter deltas into log lines and UI alerts.
- A global panic hook writes a crash report (message, backtrace, version, OS) to the log dir.

---

## 10. Security

- No network access and no telemetry.
- `#![forbid(unsafe_code)]` in every crate except `fp-backends`. All FFI there is wrapped in safe types, documented with `// SAFETY:` comments and covered by tests.
- `cargo deny` checks licences (allow-list: MIT, Apache-2.0, BSD, ISC, Zlib, OFL for fonts, MPL-2.0), advisories and duplicate sources. It runs in CI.
- Size limits on every parsed input (§6, §7).
- Fuzz targets (`cargo-fuzz`, Linux CI nightly job): the M3U/PLS parser (Phase 2), the store loader, and the cart-page import (Phase 2).
- File paths from playlists are only ever opened for reading. Nothing is executed and nothing is written outside the app's own data, cache and log dirs, or paths the user explicitly picks in a save dialog.

---

## 11. Testing strategy

- **`fp-model`:** a unit test per rule in §3, plus `proptest` properties for playlist operations (move within/between lists, remove, duplicate, insert): entries are never lost or duplicated unintentionally, and `current` is never removed.
- **`fp-engine` with the `Offline` backend:**
  - fade curve shape and duration exact to the sample;
  - overlap starts exactly at `segue_start`;
  - no discontinuity above a threshold at pause, resume and seek (a click detector over the rendered buffer);
  - events are emitted at the right frame;
  - underrun accounting;
  - the virtual-clock takeover keeps the timeline continuous.
- **RT hygiene:** `assert_no_alloc` wraps `Mixer::render` in tests. Any allocation or free fails the test.
- **Stress:** 8 players (above the default, to prove there is no fixed limit) in continuous mode with random commands, plus cue, for 6 simulated hours on `Offline`. Invariants: no panics, no stuck player, bounded queues.
- **Soak (manual/CI optional):** 1 real-time hour on `Null`.
- **`fp-store`:** round-trip, migration, corrupt-file fallback, and atomic-write crash simulation (a truncated temp file must not replace a good file).
- **`fp-analysis`:** markers on generated fixtures (sine with silent head/tail, fade-out tails of known dB slope).
- **i18n:** every `es-ES` key exists in `en-US` and vice versa; no unknown Fluent variables.
- **Config:** defaults validate; out-of-range values clamp with a warning; derived mixer capacity grows with player count.
- **UI:** snapshot-level logic tests (view-model functions: formatting, row colouring, footer maths). Rendering is checked manually.
- **CI:** GitHub Actions on `ubuntu-latest`, `windows-latest` and `macos-latest`, running `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test` and `cargo deny check`.

---

## 12. Dependencies (initial set)

`symphonia` (all pure-Rust codecs), `rubato`, `rtrb`, `crossbeam-channel`, `arc-swap`, `cpal`, `lofty`, `image`, `postcard`, `serde`/`serde_json`, `directories`, `eframe`/`egui`/`egui_extras`, `egui-phosphor`, `rfd`, `fluent-bundle`, `unic-langid`, `sys-locale`, `tracing`/`tracing-subscriber`/`tracing-appender`, `thiserror`, `anyhow`, `audio_thread_priority`. Dev: `proptest`, `assert_no_alloc`.

Exact versions are pinned in the Phase 1 plan after checking crates.io at implementation time.

---

## 13. Phases

| Phase | Content | Exit criteria |
|---|---|---|
| **1. Usable core** | Workspace, `fp-model` (§3), engine (§4) with Null/Offline and cpal ALSA / WASAPI shared / Core Audio, analysis (§6 without intro tags), store (§7 without M3U/carts), UI main screen (§8.3 without the cartwall), Settings subset (§8.4), logging, CI. | On each OS: the default 4 players (and a configured 8) play real files through selectable Main/Cue devices; every rule in §3 works; auto-segue overlaps at detected segue points; state survives restart; unplugging the output device does not stop the timeline and it recovers on replug; all tests green in CI. |
| **2. Cartwall and full Settings** | Cart pages (configurable grid, default 2×8 = 16 carts as in the design; overlap by default, loop, exclusive; per-page tabs with the playing dot; collapsible), cart bus routing, Cartwall and Shortcuts settings (remappable shortcuts), language selector, M3U/M3U8/PLS import and M3U export, cart-page import/export, manual marker editing on the waveform (intro/outro/mix, modifier + drag), `INTRO` tag convention. | Parity with the v3 design feature set. |
| **3. Native backends** | PipeWire, PulseAudio, JACK, ASIO (all through cpal hosts). | Each backend passes the backend conformance test suite and a manual on-device check. |
| **4. Bit-perfect** | WASAPI exclusive and Core Audio hog mode; exclusive mode at the file's rate and format, no resampling or processing when volume = 100 % and no fade; real BP indicator; device reopen between tracks of different rates. | Bit-exact loopback verification on at least one device per OS. |
| **5. Packaging** | deb, rpm, Flatpak, AppImage; signed MSI; signed and notarised dmg. | Installable artefacts produced by CI. |

---

## 14. Open risks

- **`cpal` device-loss behaviour differs per OS.** The watchdog (§4.7) covers silent failures, and the Phase 1 exit criteria require a manual unplug test on each OS.
- **egui table and drag-and-drop polish** may need custom widgets to match the design's density. This is budgeted inside Phase 1.
- **Real-time priority on Linux** needs rtkit or `CAP_SYS_NICE`. If both are unavailable, the app runs at normal priority, logs a warning and shows it in Settings.
