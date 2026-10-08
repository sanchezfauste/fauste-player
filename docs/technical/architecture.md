# Architecture

## Crates

| Crate | Responsibility | Depends on |
|---|---|---|
| `fp-model` | Domain types and the **pure** state machine: player rules (spec §3), cartwall rules (Phase 2 spec P2.3), markers, shortcuts. Commands in, new state and `EngineAction`s out. No I/O, no threads. | — |
| `fp-store` | Crash-safe persistence: atomic writes, rotating backups, quarantine of corrupt files, versioned JSON documents and migrations, lenient config loading; M3U/M3U8/PLS and cart page files | `fp-model` |
| `fp-decode` | File decoding to interleaved stereo `f32` (downmix per ITU-R BS.775): symphonia, plus DSD, WavPack, Monkey's Audio and Opus backends ([Decoding](decoding.md)) | — |
| `fp-backends` | The `AudioBackend` trait; `CpalBackend` (ALSA, WASAPI shared, Core Audio), `NullBackend`, `OfflineBackend` | — |
| `fp-engine` | Real-time mixer, buses with watchdog and virtual clock, per-player decode workers, resampling, the `Engine` and the `Conductor` thread | `fp-model`, `fp-backends`, `fp-decode` |
| `fp-analysis` | Tags and covers (lofty, image), waveform peaks, automatic markers, the analysis cache and the background pool | `fp-model`, `fp-decode` |
| `fp-control` | Control surfaces: MIDI parsing, bindings to commands (edge detection, soft takeover, availability), LED feedback, learn, and the MIDI service thread over `midir` (ALSA, CoreMIDI, WinMM) | `fp-model` |
| `fp-remote` | Remote control over the network: the HTTP/JSON API with SSE events (axum on a tokio current-thread runtime), OSC over UDP (rosc), the security guard and the remote thread; reaches the application through the `RemoteControl` trait ([Remote control API](remote-api.md)) | `fp-model` |
| `fp-app` | The egui application and the `fauste-player` binary: bootstrap, logging, crash reports, services thread, UI | all of the above |

```
fp-app ──► fp-engine ──► fp-backends
   │            │  └────► fp-decode
   │            └───────► fp-model
   ├──► fp-analysis ──► fp-decode, fp-model
   ├──► fp-control ───► fp-model
   ├──► fp-remote ────► fp-model
   └──► fp-store ─────► fp-model
```

All crates forbid `unsafe_code` through the workspace lints. `fp-engine`,
`fp-backends`, `fp-decode` and `fp-analysis` also deny
`clippy::indexing_slicing`. No crate may `unwrap`, `expect`
or `panic` outside tests.

## Data flow

```
 UI (egui) ──Command──► Conductor ──apply()──► fp-model reducer
    ▲                      │  ▲                     │
    │  Arc<AppState>       │  │ EngineEvent         │ EngineAction
    │  Arc<Telemetry>      ▼  │                     ▼
    └────── arc-swap ◄── Engine ──BusCommand──► Mixer (RT thread per device)
                           │  ◄──BusEvent───
                           └──► Worker threads (decode + resample per player)

 Services thread: Analyzer results ──► Command::ApplyAnalysis / SetFileState
                  model_version changes ──► Store (debounced saves)

 MIDI thread (fp-midi): port callbacks ──bytes──► Router ──Command──► Conductor
                        Arc<AppState> ──► LED diffs ──► output ports
```

1. The UI sends a `Command` over a bounded channel and never waits.
2. The conductor applies it with `fp_model::apply`. That gives the new state
   and a list of `EngineAction`s, which the `Engine` turns into bus commands
   and worker requests.
3. Buses report `BusEvent`s (started, finished). The engine turns them into
   model `EngineEvent`s, which go through `fp_model::on_event`.
4. After each change, the conductor publishes an immutable `Arc<AppState>`
   through `arc-swap`, plus a `Telemetry` snapshot (positions, peaks, bus
   health, model version). The UI and the services thread read them.
5. Output settings apply while running (live settings spec): the engine
   reports where each holder plays and what each device runs with
   (`Placed`, `AudioSystemInUse`); `fp_model::live::pending` compares that
   with the configuration; the reducer sends each change once nothing it
   affects is playing, and the engine applies it once its buses are quiet
   and answers with `Applied`.

## Shutdown

When the window closes, `run` stops everything that can send commands first
(the remote server and the MIDI service, `MidiService::shutdown`), then the
services thread makes the final save, and `run` releases the last
`Arc<ConductorHandle>`, which stops the conductor and the engine and closes
the output streams (a warning is logged if another holder remains). Nothing
goes on air by itself at the next start.

## Design principles

- **Behaviour lives in a pure reducer.** Every rule in spec §3 is a
  function of `(state, command | event) → (state, actions)`. It is unit-tested
  without audio, and property-tested for playlist operations.
- **Sample-accurate timing.** Transitions are dispatched to the mixer ahead of
  time (`tuning.schedule_lead_ms`) with an exact frame number. They do not
  depend on when the conductor thread wakes.
- **The audio thread is sacred.** It does not allocate, free, lock, log or
  panic. See [Threading and real time](threading-and-realtime.md).
- **No hardcoded product limits.** Counts and thresholds live in `Config`,
  and engine capacities are derived from it. Players, playlists and routes
  are addressed by opaque ids in growable collections.
- **Never crash on bad data.** Corrupt files fall back to backups, hostile
  covers are rejected by size, a broken track is marked and skipped, and a
  panicking UI frame shows a banner.
- **Growth by enum variants.** `Command`, `EngineAction` and `BusCommand` are
  single enums. A feature adds variants, and the exhaustiveness checks show
  every place that must react.

## Differences from the spec (Phase 1)

| Spec | Implemented | Why |
|---|---|---|
| wgpu renderer with glow fallback | glow only | smaller dependency set; enough for the UI; revisit if a platform needs wgpu |
| Fuzzing of the store loader | property and corruption tests; fuzz targets arrive with the Phase 2 parsers | see the roadmap |
| `anyhow` at the binary edge | plain `Box<dyn Error>` in `main` | a single call site |
