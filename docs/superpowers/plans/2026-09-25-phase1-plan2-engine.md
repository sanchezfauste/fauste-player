# Phase 1 · Plan 2 — Audio engine and backends Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build `fp-backends` (the `AudioBackend` trait with Null, Offline and cpal backends) and `fp-engine` (sources, the real-time bus mixer, decoding and resampling, per-player worker threads, device-loss handling, the sample-accurate `Engine` and the `Conductor` thread), consuming the `EngineAction`/`EngineEvent` contract of `fp-model`.

**Architecture:** One real-time mixer per output device sums lock-free source rings and applies gain ramps at exact frames; it never allocates, blocks, logs or panics (memory only enters and leaves through `rtrb` queues). One worker thread per player decodes (symphonia), resamples (rubato) and fills its rings. The `Engine` translates model actions into mixer commands with frame timestamps dispatched `schedule_lead` ahead, and mixer/worker observations back into model events. The `Conductor` thread owns the `AppState`, runs the model reducer, drives the engine and publishes `arc-swap` snapshots. If a device disappears, a virtual-clock thread keeps the timeline running until the device returns.

**Tech Stack:** Rust 1.98.1 (edition 2024); cpal 0.18.2 (`realtime`, and `realtime-dbus` on Linux), symphonia 0.6.1 (`all`), rubato 5.0.0 + audioadapter-buffers 5.2.0, rtrb 0.4.0, crossbeam-channel 0.5.17, arc-swap 1.9.2, tracing 0.1.44; dev: assert_no_alloc 1.1.2, hound 3.5.1, tempfile 3.27.0.

**Spec:** `docs/superpowers/specs/2026-09-25-fauste-player-design.md`. Read §2.2–§2.4 (threads, communication rules, tuning), §4 (engine), §5 (backends), §9 (errors), §11 (testing). The model contract is in `crates/fp-model/src/command.rs`.

**Position in Phase 1:** this is plan 2 of 4.

| Plan | Scope |
|---|---|
| 1 | foundation (merged) |
| **2** | engine and backends (this document) |
| 3 | `fp-analysis` |
| 4 | `fp-app` (egui UI) and wiring |

**Provenance:** every code block in this plan was compiled and tested as a prototype before the plan was written: 163 workspace tests pass, and the 6-simulated-hour soak passes. The APIs of cpal 0.18, symphonia 0.6, rubato 5 and rtrb 0.4 were verified against their sources. Reproduce the code exactly; deviations need a ledger ruling.

## Global Constraints

- All code, identifiers, comments, docs and commit messages in **English**. Never mention third-party radio-automation products.
- Established naming, as in the spec glossary:
  - `Source` (not voice), `Bus`, `Mixer`, `Player`;
  - `segue_start`, `cue_in`, `cue_out`;
  - `SOURCE_END` means "at the natural end of the decoded file".
- No hardcoded product limits:
  - every timing comes from `fp_model::Tuning`;
  - mixer capacity is derived from routing × `tuning.mixer_headroom`;
  - the only constants are format facts (`SOURCE_CHANNELS = 2`) and internal work-chunk sizes.
- **Real-time rules (spec §2.2).** `Mixer::render` and everything it calls must never allocate, free, block, do I/O, log or panic.
  - Memory enters as `BusCommand` and leaves as `Retired` through `rtrb` queues.
  - The only lock an RT thread touches is the mixer hand-over mutex, and only via `try_lock`.
- `#![forbid(unsafe_code)]` (workspace lint) in both crates, and `#![deny(clippy::indexing_slicing)]` in both crates' `lib.rs` (spec §9). Tests may `#![allow(clippy::indexing_slicing)]`.
- No `unwrap`/`expect`/`panic!` in non-test code (workspace lints).
- `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` pass before every commit.
- Commit messages end with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- **Linux system packages:** `build-essential`, `pkg-config`, `libasound2-dev` (already installed), and `libdbus-1-dev` for cpal's `realtime-dbus` (rtkit real-time priority), added in Task 2.

## Review Focus

1. **A device that refuses the requested buffer size** (common on USB interfaces and some Windows drivers). Expect the stream to open with the device's default size rather than the bus staying silent. Pinned in Task 2 (`a_buffer_size_outside_every_reported_range_falls_back_to_the_device_default`).
2. **A file shorter than the model believes** (wrong tags, truncated download), so a planned stop lies beyond the end of the audio. Expect the engine to report the end when the audio runs out, and never to hang. Pinned in Task 8 (`a_file_shorter_than_its_planned_stop_reports_the_end_when_it_runs_out`).
3. **A route naming a backend that does not exist on this machine** (for example a config copied from Windows to Linux). Expect a fall-back to the default backend, and never a silent player. Pinned in Task 8 (`a_route_to_an_unknown_backend_falls_back_to_the_default_backend`).
4. **The UI flooding the conductor with commands.** Expect `send` to refuse without ever blocking the UI thread. Pinned in Task 9 (`a_flooded_command_queue_refuses_instead_of_blocking`).
5. **Pause racing a transition that the audio thread has already executed** (found by the stress test while prototyping). Expect the engine to follow what is audible and keep the player usable. Pinned in Task 8 (`a_transition_that_already_happened_when_pause_arrives_leaves_the_player_playing`).

---

## File Structure

```
Cargo.toml                                  + workspace deps for this plan
crates/fp-backends/
  Cargo.toml
  src/lib.rs                                AudioBackend, Renderer, OutputStream, DeviceInfo, StreamConfig, errors
  src/offline.rs                            OfflineBackend/OfflineDevice (render on demand; tests)
  src/null.rs                               NullBackend (discards at real-time pace)
  src/cpal_backend.rs                       CpalBackend (ALSA / WASAPI shared / Core Audio)
  tests/backends.rs                         Offline and Null behaviour
  tests/cpal.rs                             smoke test on the real platform stack
crates/fp-engine/
  Cargo.toml
  src/lib.rs                                module list
  src/atomic.rs                             AtomicF32
  src/ramp.rs                               Ramp, Curve (linear / equal-power)
  src/source.rs                             source_pair, SourceProducer/Consumer, SourceShared
  src/mixer.rs                              Mixer, BusCommand, BusEvent, Retired, SlotStorage, BusShared, MixerRenderer
  src/decode.rs                             FileDecoder (symphonia), downmix
  src/resample.rs                           StreamResampler (rubato, exact length)
  src/worker.rs                             PlayerWorker, SampleSource, SourceOpener, file_opener
  src/bus.rs                                Bus (device + mixer + watchdog + virtual clock + reconnect)
  src/engine.rs                             Engine, EngineSettings, PlayerTelemetry, BusStatus
  src/conductor.rs                          Conductor, ConductorHandle, Telemetry
  tests/support/mod.rs                      WAV fixtures and synthetic sources
  tests/mixer.rs  tests/decode.rs  tests/worker.rs  tests/bus.rs  tests/engine.rs  tests/conductor.rs
```

---

### Task 1: `fp-backends` — the backend trait, Offline and Null backends

**Files:**
- Modify: `Cargo.toml` (workspace dependencies)
- Create: `crates/fp-backends/Cargo.toml`, `crates/fp-backends/src/lib.rs`, `crates/fp-backends/src/offline.rs`, `crates/fp-backends/src/null.rs`
- Test: `crates/fp-backends/tests/backends.rs`

**Interfaces:**
- Produces (spec §5.1):
  - **Identifiers and descriptions:** `BackendId(pub String)`, `DeviceId(pub String)`, `Availability`, `DeviceInfo`, `StreamConfig { sample_rate, buffer_frames, channels }`.
  - **Stream errors:** `StreamErrorKind { DeviceLost, Xrun, RealtimeDenied, Other }`, reported through `trait StreamErrorSink: Send + Sync { fn report(&self, kind) }`.
  - **Rendering:** `trait Renderer: Send + 'static { fn render(&mut self, out: &mut [f32], channels: usize) }` and `trait OutputStream: Send { fn config(&self) -> StreamConfig }`. Dropping an `OutputStream` stops it.
  - **`BackendError`:** `Unavailable`, `DeviceNotFound(DeviceId)`, `Unsupported`, `Backend`.
  - **`trait AudioBackend: Send + Sync`:** `id()`, `availability()`, `enumerate_devices()`, `default_device() -> Option<DeviceId>`, and `open_output(&DeviceId, StreamConfig, Box<dyn Renderer>, Arc<dyn StreamErrorSink>) -> Result<Box<dyn OutputStream>, BackendError>`.
  - **`OfflineBackend`:** `new()`, `add_device(id, channels) -> OfflineDevice` and `device(&DeviceId)`.
  - **`OfflineDevice`:** `id()`, `render(frames) -> Option<Vec<f32>>`, `is_open()`, `unplug()` and `replug()`.
  - **`NullBackend`:** device `"null"`, which renders at real-time pace on its own thread.
- Deviation from spec §5.1, recorded as a ruling: `subscribe_device_changes` is not part of the Phase 1 trait. Device loss is detected by stream errors plus the engine's heartbeat watchdog (spec §4.7). Hot-plug notifications arrive with the native backends in Phase 3.

- [ ] **Step 1: Add the workspace dependencies**

In the root `Cargo.toml`:

1. Add `fp-backends = { path = "crates/fp-backends" }` after the `fp-model` line of `[workspace.dependencies]`.
2. Append these lines to the same section:
```toml
cpal = { version = "0.18.2", features = ["realtime"] }
symphonia = { version = "0.6.1", features = ["all"] }
rubato = "5.0.0"
audioadapter-buffers = "5.2.0"
rtrb = "0.4.0"
crossbeam-channel = "0.5.17"
arc-swap = "1.9.2"
tracing = "0.1.44"
assert_no_alloc = "1.1.2"
hound = "3.5.1"
```

- [ ] **Step 2: Write the crate manifest and the failing tests**

`crates/fp-backends/Cargo.toml` (cpal is added in Task 2):
```toml
[package]
name = "fp-backends"
description = "Audio output backends behind one trait (Null, Offline, cpal)"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
publish.workspace = true

[dependencies]
thiserror.workspace = true

[lints]
workspace = true
```

`crates/fp-backends/tests/backends.rs`:
```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use fp_backends::{
    AudioBackend, BackendError, DeviceId, NullBackend, OfflineBackend, Renderer, StreamConfig,
    StreamErrorKind, StreamErrorSink,
};

/// Writes an increasing counter into every sample and counts frames.
struct Counter {
    next: f32,
    frames: Arc<AtomicU64>,
}

impl Renderer for Counter {
    fn render(&mut self, out: &mut [f32], channels: usize) {
        for s in out.iter_mut() {
            *s = self.next;
            self.next += 1.0;
        }
        self.frames
            .fetch_add((out.len() / channels) as u64, Ordering::Relaxed);
    }
}

#[derive(Default)]
struct Errors {
    seen: Mutex<Vec<StreamErrorKind>>,
    count: AtomicUsize,
}

impl StreamErrorSink for Errors {
    fn report(&self, kind: StreamErrorKind) {
        self.count.fetch_add(1, Ordering::Relaxed);
        self.seen.lock().unwrap().push(kind);
    }
}

const STEREO: StreamConfig = StreamConfig {
    sample_rate: 48_000,
    buffer_frames: 256,
    channels: 2,
};

fn counter() -> (Box<Counter>, Arc<AtomicU64>) {
    let frames = Arc::new(AtomicU64::new(0));
    (
        Box::new(Counter {
            next: 0.0,
            frames: frames.clone(),
        }),
        frames,
    )
}

#[test]
fn offline_renders_on_demand_and_stops_when_dropped() {
    let backend = OfflineBackend::new();
    let device = backend.add_device("card", 2);
    assert_eq!(device.render(4), None, "nothing open yet");
    let (renderer, frames) = counter();
    let stream = backend
        .open_output(&device.id(), STEREO, renderer, Arc::new(Errors::default()))
        .unwrap();
    assert_eq!(device.render(2).unwrap(), vec![0.0, 1.0, 2.0, 3.0]);
    assert_eq!(frames.load(Ordering::Relaxed), 2);
    drop(stream);
    assert!(!device.is_open());
}

#[test]
fn offline_unplug_reports_loss_and_refuses_reopen_until_replugged() {
    let backend = OfflineBackend::new();
    let device = backend.add_device("usb", 2);
    let errors = Arc::new(Errors::default());
    let (renderer, _) = counter();
    let _stream = backend
        .open_output(&device.id(), STEREO, renderer, errors.clone())
        .unwrap();
    device.unplug();
    assert_eq!(
        *errors.seen.lock().unwrap(),
        vec![StreamErrorKind::DeviceLost]
    );
    assert_eq!(device.render(4), None);
    assert!(backend.enumerate_devices().unwrap().is_empty());
    let (renderer, _) = counter();
    assert!(matches!(
        backend.open_output(&device.id(), STEREO, renderer, errors.clone()),
        Err(BackendError::DeviceNotFound(_))
    ));
    device.replug();
    let (renderer, _) = counter();
    assert!(
        backend
            .open_output(&device.id(), STEREO, renderer, errors)
            .is_ok()
    );
}

#[test]
fn offline_rejects_more_channels_than_the_device_has() {
    let backend = OfflineBackend::new();
    let device = backend.add_device("mono", 1);
    let (renderer, _) = counter();
    assert!(matches!(
        backend.open_output(&device.id(), STEREO, renderer, Arc::new(Errors::default())),
        Err(BackendError::Unsupported(_))
    ));
}

#[test]
fn null_backend_consumes_audio_at_real_time_pace() {
    let backend = NullBackend;
    let device = backend.default_device().unwrap();
    let (renderer, frames) = counter();
    let stream = backend
        .open_output(&device, STEREO, renderer, Arc::new(Errors::default()))
        .unwrap();
    std::thread::sleep(Duration::from_millis(250));
    drop(stream);
    let rendered = frames.load(Ordering::Relaxed);
    // 250 ms at 48 kHz is 12 000 frames; allow for scheduling jitter.
    assert!((8_000..=16_000).contains(&rendered), "rendered {rendered}");
    let after = frames.load(Ordering::Relaxed);
    std::thread::sleep(Duration::from_millis(50));
    assert_eq!(
        frames.load(Ordering::Relaxed),
        after,
        "a dropped stream stops rendering"
    );
}

#[test]
fn null_backend_rejects_unknown_devices() {
    let (renderer, _) = counter();
    assert!(matches!(
        NullBackend.open_output(
            &DeviceId("nope".into()),
            STEREO,
            renderer,
            Arc::new(Errors::default())
        ),
        Err(BackendError::DeviceNotFound(_))
    ));
}
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cargo test -p fp-backends`
Expected: FAIL: the crate has no `src/lib.rs` yet (`can't find library`), or unresolved imports.

- [ ] **Step 4: Implement**

`crates/fp-backends/src/lib.rs`: use the file below, **but without the two cpal lines** (`mod cpal_backend;` and `pub use cpal_backend::CpalBackend;`). Task 2 adds them.

`crates/fp-backends/src/lib.rs`:
```rust
//! Audio output backends behind one trait (spec §5). The engine only talks to
//! `AudioBackend`; each backend turns the engine's `Renderer` into a device
//! stream. Phase 1 ships `Null`, `Offline` (tests) and a cpal-based backend.

#![deny(clippy::indexing_slicing)]

use std::sync::Arc;

use thiserror::Error;

mod cpal_backend;
mod null;
mod offline;

pub use cpal_backend::CpalBackend;
pub use null::NullBackend;
pub use offline::{OfflineBackend, OfflineDevice};

/// Stable identifier of a backend, e.g. `"alsa"`, `"wasapi"`, `"null"`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BackendId(pub String);

/// Backend-specific device identifier that survives restarts (stored in config).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeviceId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Availability {
    Available,
    Unavailable(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    pub id: DeviceId,
    pub name: String,
    pub channels: u16,
    /// Inclusive sample-rate ranges the device reports.
    pub sample_rates: Vec<(u32, u32)>,
    /// Inclusive buffer-size range in frames, when the device reports one.
    pub buffer_frames: Option<(u32, u32)>,
    pub exclusive_capable: bool,
    pub rate_switching: bool,
}

/// What the engine asks a backend to open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamConfig {
    pub sample_rate: u32,
    pub buffer_frames: u32,
    pub channels: u16,
}

/// Classified stream failures, reported from the backend's own threads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamErrorKind {
    /// The device disappeared or the stream can no longer run.
    DeviceLost,
    /// A buffer underrun/overrun happened in the backend.
    Xrun,
    /// The OS refused real-time scheduling; audio still plays but is more
    /// exposed to glitches under load (shown as a warning in Settings).
    RealtimeDenied,
    /// Anything else the backend reported.
    Other,
}

/// Receives stream errors. Implementations must be cheap and lock-free: they
/// may be called on a real-time thread.
pub trait StreamErrorSink: Send + Sync {
    fn report(&self, kind: StreamErrorKind);
}

/// Produces audio. Called on the backend's real-time thread: implementations
/// must not allocate, block, perform I/O or panic.
pub trait Renderer: Send + 'static {
    /// Fills `out` (interleaved f32, `channels` per frame) completely.
    fn render(&mut self, out: &mut [f32], channels: usize);
}

/// A running output stream. Dropping it stops the stream.
pub trait OutputStream: Send {
    /// The configuration actually in use.
    fn config(&self) -> StreamConfig;
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BackendError {
    #[error("backend unavailable: {0}")]
    Unavailable(String),
    #[error("device not found: {0:?}")]
    DeviceNotFound(DeviceId),
    #[error("unsupported configuration: {0}")]
    Unsupported(String),
    #[error("backend error: {0}")]
    Backend(String),
}

pub trait AudioBackend: Send + Sync {
    fn id(&self) -> BackendId;
    fn availability(&self) -> Availability;
    fn enumerate_devices(&self) -> Result<Vec<DeviceInfo>, BackendError>;
    fn default_device(&self) -> Option<DeviceId>;
    fn open_output(
        &self,
        device: &DeviceId,
        config: StreamConfig,
        renderer: Box<dyn Renderer>,
        errors: Arc<dyn StreamErrorSink>,
    ) -> Result<Box<dyn OutputStream>, BackendError>;
}
```

`crates/fp-backends/src/offline.rs`:
```rust
//! A backend for tests: devices are rendered on demand, on the caller's
//! thread, so engine behaviour can be checked sample by sample without a
//! sound card or real time passing.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::{
    AudioBackend, Availability, BackendError, BackendId, DeviceId, DeviceInfo, OutputStream,
    Renderer, StreamConfig, StreamErrorKind, StreamErrorSink,
};

struct Open {
    generation: u64,
    config: StreamConfig,
    renderer: Box<dyn Renderer>,
    errors: Arc<dyn StreamErrorSink>,
}

#[derive(Default)]
struct DeviceState {
    plugged: bool,
    generation: u64,
    open: Option<Open>,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Test handle to one offline device.
#[derive(Clone)]
pub struct OfflineDevice {
    id: DeviceId,
    channels: u16,
    state: Arc<Mutex<DeviceState>>,
}

impl OfflineDevice {
    pub fn id(&self) -> DeviceId {
        self.id.clone()
    }

    /// Renders `frames` frames through the open stream. Returns `None` when no
    /// stream is open (never opened, dropped, or the device was unplugged).
    pub fn render(&self, frames: usize) -> Option<Vec<f32>> {
        let mut state = lock(&self.state);
        let open = state.open.as_mut()?;
        let channels = usize::from(open.config.channels);
        let mut out = vec![0.0; frames * channels];
        open.renderer.render(&mut out, channels);
        Some(out)
    }

    pub fn is_open(&self) -> bool {
        lock(&self.state).open.is_some()
    }

    /// Simulates the device disappearing: reports `DeviceLost`, stops the
    /// stream and makes further opens fail until `replug`.
    pub fn unplug(&self) {
        let mut state = lock(&self.state);
        state.plugged = false;
        if let Some(open) = state.open.take() {
            open.errors.report(StreamErrorKind::DeviceLost);
        }
    }

    pub fn replug(&self) {
        lock(&self.state).plugged = true;
    }
}

struct OfflineStream {
    generation: u64,
    config: StreamConfig,
    state: Arc<Mutex<DeviceState>>,
}

impl OutputStream for OfflineStream {
    fn config(&self) -> StreamConfig {
        self.config
    }
}

impl Drop for OfflineStream {
    fn drop(&mut self) {
        let mut state = lock(&self.state);
        if state
            .open
            .as_ref()
            .is_some_and(|o| o.generation == self.generation)
        {
            state.open = None;
        }
    }
}

/// See the module documentation.
#[derive(Clone, Default)]
pub struct OfflineBackend {
    devices: Arc<Mutex<HashMap<DeviceId, OfflineDevice>>>,
}

impl OfflineBackend {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a plugged-in device with `channels` output channels.
    pub fn add_device(&self, id: &str, channels: u16) -> OfflineDevice {
        let device = OfflineDevice {
            id: DeviceId(id.to_owned()),
            channels,
            state: Arc::new(Mutex::new(DeviceState {
                plugged: true,
                ..DeviceState::default()
            })),
        };
        lock(&self.devices).insert(device.id.clone(), device.clone());
        device
    }

    pub fn device(&self, id: &DeviceId) -> Option<OfflineDevice> {
        lock(&self.devices).get(id).cloned()
    }
}

impl AudioBackend for OfflineBackend {
    fn id(&self) -> BackendId {
        BackendId("offline".to_owned())
    }

    fn availability(&self) -> Availability {
        Availability::Available
    }

    fn enumerate_devices(&self) -> Result<Vec<DeviceInfo>, BackendError> {
        let devices = lock(&self.devices);
        let mut list: Vec<DeviceInfo> = devices
            .values()
            .filter(|d| lock(&d.state).plugged)
            .map(|d| DeviceInfo {
                id: d.id.clone(),
                name: d.id.0.clone(),
                channels: d.channels,
                sample_rates: vec![(8_000, 768_000)],
                buffer_frames: Some((16, 16_384)),
                exclusive_capable: false,
                rate_switching: false,
            })
            .collect();
        list.sort_by(|a, b| a.id.0.cmp(&b.id.0));
        Ok(list)
    }

    fn default_device(&self) -> Option<DeviceId> {
        self.enumerate_devices()
            .ok()?
            .into_iter()
            .next()
            .map(|d| d.id)
    }

    fn open_output(
        &self,
        device: &DeviceId,
        config: StreamConfig,
        renderer: Box<dyn Renderer>,
        errors: Arc<dyn StreamErrorSink>,
    ) -> Result<Box<dyn OutputStream>, BackendError> {
        let dev = self
            .device(device)
            .ok_or_else(|| BackendError::DeviceNotFound(device.clone()))?;
        if config.channels > dev.channels {
            return Err(BackendError::Unsupported(format!(
                "{} channels requested, device has {}",
                config.channels, dev.channels
            )));
        }
        let mut state = lock(&dev.state);
        if !state.plugged {
            return Err(BackendError::DeviceNotFound(device.clone()));
        }
        state.generation += 1;
        let generation = state.generation;
        state.open = Some(Open {
            generation,
            config,
            renderer,
            errors,
        });
        Ok(Box::new(OfflineStream {
            generation,
            config,
            state: dev.state.clone(),
        }))
    }
}
```

`crates/fp-backends/src/null.rs`:
```rust
//! A backend that discards audio at real-time pace. It keeps the engine's
//! timeline running when no sound card is wanted (and in soak tests).

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crate::{
    AudioBackend, Availability, BackendError, BackendId, DeviceId, DeviceInfo, OutputStream,
    Renderer, StreamConfig, StreamErrorSink,
};

pub const NULL_DEVICE: &str = "null";

#[derive(Debug, Clone, Copy, Default)]
pub struct NullBackend;

struct NullStream {
    config: StreamConfig,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl OutputStream for NullStream {
    fn config(&self) -> StreamConfig {
        self.config
    }
}

impl Drop for NullStream {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl AudioBackend for NullBackend {
    fn id(&self) -> BackendId {
        BackendId("null".to_owned())
    }

    fn availability(&self) -> Availability {
        Availability::Available
    }

    fn enumerate_devices(&self) -> Result<Vec<DeviceInfo>, BackendError> {
        Ok(vec![DeviceInfo {
            id: DeviceId(NULL_DEVICE.to_owned()),
            name: "Null output".to_owned(),
            channels: 64,
            sample_rates: vec![(8_000, 768_000)],
            buffer_frames: Some((16, 16_384)),
            exclusive_capable: false,
            rate_switching: false,
        }])
    }

    fn default_device(&self) -> Option<DeviceId> {
        Some(DeviceId(NULL_DEVICE.to_owned()))
    }

    fn open_output(
        &self,
        device: &DeviceId,
        config: StreamConfig,
        mut renderer: Box<dyn Renderer>,
        _errors: Arc<dyn StreamErrorSink>,
    ) -> Result<Box<dyn OutputStream>, BackendError> {
        if device.0 != NULL_DEVICE {
            return Err(BackendError::DeviceNotFound(device.clone()));
        }
        if config.sample_rate == 0 || config.buffer_frames == 0 || config.channels == 0 {
            return Err(BackendError::Unsupported(format!("{config:?}")));
        }
        let stop = Arc::new(AtomicBool::new(false));
        let stop_flag = stop.clone();
        let channels = usize::from(config.channels);
        let frames = config.buffer_frames as usize;
        let period = Duration::from_secs_f64(
            f64::from(config.buffer_frames) / f64::from(config.sample_rate),
        );
        let thread = std::thread::Builder::new()
            .name("fp-null-output".to_owned())
            .spawn(move || {
                let mut buffer = vec![0.0f32; frames * channels];
                let mut deadline = Instant::now();
                while !stop_flag.load(Ordering::Acquire) {
                    renderer.render(&mut buffer, channels);
                    deadline += period;
                    let now = Instant::now();
                    if deadline > now {
                        std::thread::sleep(deadline - now);
                    } else {
                        // Fell behind (machine suspended, debugger…): do not try to catch up.
                        deadline = now;
                    }
                }
            })
            .map_err(|e| BackendError::Backend(e.to_string()))?;
        Ok(Box::new(NullStream {
            config,
            stop,
            thread: Some(thread),
        }))
    }
}
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p fp-backends`
Expected: 5 tests PASS.

- [ ] **Step: Check and commit**

Run: `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
Expected: everything passes, no warnings.
```bash
git add Cargo.toml Cargo.lock crates/fp-backends
git commit -m "feat(backends): add backend trait with Offline and Null backends

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: `fp-backends` — the cpal backend (ALSA / WASAPI shared / Core Audio)

**Files:**
- Modify: `crates/fp-backends/Cargo.toml`, `crates/fp-backends/src/lib.rs`
- Create: `crates/fp-backends/src/cpal_backend.rs`
- Test: `crates/fp-backends/tests/cpal.rs`, plus the unit test inside `cpal_backend.rs`

**Interfaces:**
- Consumes: Task 1.
- Produces:
  - `CpalBackend::default_host()`, which implements `AudioBackend` with `id()` equal to the lowercase cpal host name (`"alsa"`, `"wasapi"`, `"coreaudio"`).
  - Device ids are cpal's persistent `DeviceId` strings (`"alsa:default"`, …).
  - `open_output` requests `BufferSize::Fixed(buffer_frames)` only when the device reports a range containing it, and `BufferSize::Default` otherwise.
  - `cpal::ErrorKind` is mapped to `StreamErrorKind` (`DeviceNotAvailable`/`StreamInvalidated`/`HostUnavailable` → `DeviceLost`).

- [ ] **Step 1: Install the D-Bus headers (Linux only)**

cpal's `realtime-dbus` feature asks rtkit for real-time scheduling, so a normal desktop user gets real-time audio threads (spec §2.2, §14). Ask the human partner to run:
```
sudo apt install -y libdbus-1-dev
```
Verify with `pkg-config --modversion dbus-1`: it prints a version.

- [ ] **Step 2: Add cpal to the crate**

In `crates/fp-backends/Cargo.toml`, add `cpal.workspace = true` under `[dependencies]` and append:
```toml
[target.'cfg(target_os = "linux")'.dependencies]
cpal = { workspace = true, features = ["realtime-dbus"] }
```

- [ ] **Step 3: Write the failing tests**

`crates/fp-backends/tests/cpal.rs`:
```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Smoke test against the real platform audio stack. It never fails on
//! machines without a usable output device (CI runners).

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use fp_backends::{
    AudioBackend, CpalBackend, Renderer, StreamConfig, StreamErrorKind, StreamErrorSink,
};

struct Silence {
    frames: Arc<AtomicU64>,
}

impl Renderer for Silence {
    fn render(&mut self, out: &mut [f32], channels: usize) {
        out.fill(0.0);
        self.frames
            .fetch_add((out.len() / channels) as u64, Ordering::Relaxed);
    }
}

struct Ignore;

impl StreamErrorSink for Ignore {
    fn report(&self, _kind: StreamErrorKind) {}
}

const STEREO: StreamConfig = StreamConfig {
    sample_rate: 48_000,
    buffer_frames: 256,
    channels: 2,
};

#[test]
fn cpal_backend_enumerates_without_panicking_and_opens_the_default_device_if_any() {
    let backend = CpalBackend::default_host();
    let Ok(devices) = backend.enumerate_devices() else {
        return;
    };
    let Some(default) = backend.default_device() else {
        return;
    };
    assert!(devices.iter().any(|d| d.id == default) || !devices.is_empty());
    let frames = Arc::new(AtomicU64::new(0));
    let renderer = Box::new(Silence {
        frames: frames.clone(),
    });
    let Ok(stream) = backend.open_output(&default, STEREO, renderer, Arc::new(Ignore)) else {
        return; // CI machines often have no usable output device.
    };
    std::thread::sleep(Duration::from_millis(200));
    drop(stream);
    assert!(frames.load(Ordering::Relaxed) > 0);
}
```

The unit test at the bottom of `cpal_backend.rs` (Step 5) pins the buffer-size fall-back.

- [ ] **Step 4: Run the tests to verify they fail**

Run: `cargo test -p fp-backends --test cpal`
Expected: FAIL: `no CpalBackend in the root`.

- [ ] **Step 5: Implement**

Add to `crates/fp-backends/src/lib.rs` the line `mod cpal_backend;` (before `mod null;`) and `pub use cpal_backend::CpalBackend;` (before the `NullBackend` re-export). The file then matches Task 1's listing exactly.

`crates/fp-backends/src/cpal_backend.rs`:
```rust
//! Backend built on cpal. In Phase 1 it provides the platform default host
//! (ALSA on Linux, WASAPI shared mode on Windows, Core Audio on macOS).

use std::str::FromStr;
use std::sync::Arc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::{
    AudioBackend, Availability, BackendError, BackendId, DeviceId, DeviceInfo, OutputStream,
    Renderer, StreamConfig, StreamErrorKind, StreamErrorSink,
};

#[derive(Debug, Clone, Copy)]
pub struct CpalBackend {
    host_id: cpal::HostId,
}

struct CpalStream {
    config: StreamConfig,
    _stream: cpal::Stream,
}

impl OutputStream for CpalStream {
    fn config(&self) -> StreamConfig {
        self.config
    }
}

fn backend_error(e: impl std::fmt::Display) -> BackendError {
    BackendError::Backend(e.to_string())
}

fn classify(kind: cpal::ErrorKind) -> StreamErrorKind {
    match kind {
        cpal::ErrorKind::DeviceNotAvailable
        | cpal::ErrorKind::StreamInvalidated
        | cpal::ErrorKind::HostUnavailable => StreamErrorKind::DeviceLost,
        cpal::ErrorKind::Xrun => StreamErrorKind::Xrun,
        cpal::ErrorKind::RealtimeDenied => StreamErrorKind::RealtimeDenied,
        _ => StreamErrorKind::Other,
    }
}

/// The requested buffer size if the device reports a range containing it;
/// `None` means "let the device choose" (a device that refuses a fixed size
/// must still play rather than leave the bus silent).
fn choose_buffer_frames(ranges: &[(u32, u32)], wanted: u32) -> Option<u32> {
    ranges
        .iter()
        .any(|&(min, max)| (min..=max).contains(&wanted))
        .then_some(wanted)
}

impl CpalBackend {
    /// The platform's default cpal host.
    pub fn default_host() -> Self {
        Self {
            host_id: cpal::default_host().id(),
        }
    }

    fn host(&self) -> Result<cpal::Host, BackendError> {
        cpal::host_from_id(self.host_id).map_err(|e| BackendError::Unavailable(e.to_string()))
    }

    fn find(&self, host: &cpal::Host, device: &DeviceId) -> Result<cpal::Device, BackendError> {
        let id = cpal::DeviceId::from_str(&device.0)
            .map_err(|_| BackendError::DeviceNotFound(device.clone()))?;
        host.device_by_id(&id)
            .ok_or_else(|| BackendError::DeviceNotFound(device.clone()))
    }
}

impl AudioBackend for CpalBackend {
    fn id(&self) -> BackendId {
        BackendId(self.host_id.name().to_lowercase())
    }

    fn availability(&self) -> Availability {
        match self.host() {
            Ok(_) => Availability::Available,
            Err(e) => Availability::Unavailable(e.to_string()),
        }
    }

    fn enumerate_devices(&self) -> Result<Vec<DeviceInfo>, BackendError> {
        let host = self.host()?;
        let mut list = Vec::new();
        for device in host.output_devices().map_err(backend_error)? {
            let Ok(id) = device.id() else { continue };
            let name = device
                .description()
                .map(|d| d.name().to_owned())
                .unwrap_or_else(|_| id.to_string());
            let mut channels = 0;
            let mut sample_rates = Vec::new();
            let mut buffer_frames: Option<(u32, u32)> = None;
            if let Ok(configs) = device.supported_output_configs() {
                for c in configs {
                    channels = channels.max(c.channels());
                    sample_rates.push((c.min_sample_rate(), c.max_sample_rate()));
                    if let cpal::SupportedBufferSize::Range { min, max } = *c.buffer_size() {
                        buffer_frames = Some(match buffer_frames {
                            Some((lo, hi)) => (lo.min(min), hi.max(max)),
                            None => (min, max),
                        });
                    }
                }
            }
            sample_rates.sort_unstable();
            sample_rates.dedup();
            list.push(DeviceInfo {
                id: DeviceId(id.to_string()),
                name,
                channels,
                sample_rates,
                buffer_frames,
                exclusive_capable: false,
                rate_switching: false,
            });
        }
        Ok(list)
    }

    fn default_device(&self) -> Option<DeviceId> {
        let host = self.host().ok()?;
        let device = host.default_output_device()?;
        device.id().ok().map(|id| DeviceId(id.to_string()))
    }

    fn open_output(
        &self,
        device: &DeviceId,
        config: StreamConfig,
        mut renderer: Box<dyn Renderer>,
        errors: Arc<dyn StreamErrorSink>,
    ) -> Result<Box<dyn OutputStream>, BackendError> {
        let host = self.host()?;
        let dev = self.find(&host, device)?;
        let channels = usize::from(config.channels);
        let ranges: Vec<(u32, u32)> = dev
            .supported_output_configs()
            .map(|configs| {
                configs
                    .filter(|c| {
                        c.channels() >= config.channels
                            && c.min_sample_rate() <= config.sample_rate
                            && config.sample_rate <= c.max_sample_rate()
                    })
                    .filter_map(|c| match *c.buffer_size() {
                        cpal::SupportedBufferSize::Range { min, max } => Some((min, max)),
                        cpal::SupportedBufferSize::Unknown => None,
                    })
                    .collect()
            })
            .unwrap_or_default();
        let buffer_size = match choose_buffer_frames(&ranges, config.buffer_frames) {
            Some(frames) => cpal::BufferSize::Fixed(frames),
            None => cpal::BufferSize::Default,
        };
        let cpal_config = cpal::StreamConfig {
            channels: config.channels,
            sample_rate: config.sample_rate,
            buffer_size,
        };
        let stream = dev
            .build_output_stream::<f32, _, _>(
                cpal_config,
                move |out: &mut [f32], _info: &cpal::OutputCallbackInfo| {
                    renderer.render(out, channels)
                },
                move |err: cpal::Error| errors.report(classify(err.kind())),
                None,
            )
            .map_err(|e| match e.kind() {
                cpal::ErrorKind::DeviceNotAvailable => BackendError::DeviceNotFound(device.clone()),
                cpal::ErrorKind::UnsupportedConfig => BackendError::Unsupported(e.to_string()),
                _ => backend_error(e),
            })?;
        stream.play().map_err(backend_error)?;
        Ok(Box::new(CpalStream {
            config,
            _stream: stream,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::choose_buffer_frames;

    #[test]
    fn a_buffer_size_outside_every_reported_range_falls_back_to_the_device_default() {
        assert_eq!(choose_buffer_frames(&[(64, 4096)], 512), Some(512));
        assert_eq!(choose_buffer_frames(&[(1024, 4096)], 512), None);
        assert_eq!(choose_buffer_frames(&[], 512), None);
    }
}
```

- [ ] **Step 6: Run the tests to verify they pass**

Run: `cargo test -p fp-backends`
Expected: 7 tests PASS. The cpal smoke test returns early on machines without an output device.

- [ ] **Step: Check and commit**

Run: `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
Expected: everything passes, no warnings.
```bash
git add crates/fp-backends Cargo.lock
git commit -m "feat(backends): add cpal backend with buffer-size fallback

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: `fp-engine` — crate, `AtomicF32` and gain ramps

**Files:**
- Create: `crates/fp-engine/Cargo.toml`, `crates/fp-engine/src/lib.rs`, `crates/fp-engine/src/atomic.rs`, `crates/fp-engine/src/ramp.rs` (unit tests inside)

**Interfaces:**
- Produces:
  - **`AtomicF32`:** `new`, `load`, `store`, `fetch_max`, and `take` (read and reset).
  - **`Curve`:** `Linear` or `EqualPower`.
  - **`Ramp`:**
    - construction and control: `hold(gain)`, `retarget(to, len, curve)` (starts from the current value);
    - reading and stepping: `value()`, `next_gain()`, `is_done()`, `target()` and `remaining()`.
  - The equal-power fade-out is `cos`-shaped and the fade-in `sin`-shaped, so `out² + in² = 1` throughout a crossfade.

- [ ] **Step 1: Write the manifest and the failing tests**

`crates/fp-engine/Cargo.toml`:
```toml
[package]
name = "fp-engine"
description = "Audio engine: sources, bus mixers, decoding and the conductor"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
publish.workspace = true

[dependencies]
fp-model.workspace = true
fp-backends.workspace = true
thiserror.workspace = true
symphonia.workspace = true
rubato.workspace = true
audioadapter-buffers.workspace = true
rtrb.workspace = true
crossbeam-channel.workspace = true
arc-swap.workspace = true
tracing.workspace = true

[dev-dependencies]
assert_no_alloc.workspace = true
hound.workspace = true
tempfile.workspace = true

[lints]
workspace = true
```

`crates/fp-engine/src/lib.rs` (whole file):
```rust
//! The audio engine (spec §4): decoded `Source`s feed per-device bus `Mixer`s
//! on real-time threads; per-player worker threads decode and resample; the
//! `Conductor` turns model `EngineAction`s into sample-accurate bus commands.

#![deny(clippy::indexing_slicing)]

pub mod atomic;
pub mod ramp;
```

Create `crates/fp-engine/src/atomic.rs` and `crates/fp-engine/src/ramp.rs`, each containing **only its `#[cfg(test)] mod tests` block** from the listings in Step 3.

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-engine`
Expected: FAIL to compile: `cannot find type AtomicF32` / `Ramp`.

- [ ] **Step 3: Implement (whole files)**

`crates/fp-engine/src/atomic.rs`:
```rust
//! Lock-free scalar cells shared between threads.

use std::sync::atomic::{AtomicU32, Ordering};

/// An `f32` readable and writable from any thread without locks.
#[derive(Debug, Default)]
pub struct AtomicF32(AtomicU32);

impl AtomicF32 {
    pub fn new(value: f32) -> Self {
        Self(AtomicU32::new(value.to_bits()))
    }

    pub fn load(&self) -> f32 {
        f32::from_bits(self.0.load(Ordering::Relaxed))
    }

    pub fn store(&self, value: f32) {
        self.0.store(value.to_bits(), Ordering::Relaxed);
    }

    /// Stores `max(current, value)`.
    pub fn fetch_max(&self, value: f32) {
        let mut current = self.0.load(Ordering::Relaxed);
        while value > f32::from_bits(current) {
            match self.0.compare_exchange_weak(
                current,
                value.to_bits(),
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => return,
                Err(actual) => current = actual,
            }
        }
    }

    /// Returns the value and resets it to zero (peak meters).
    pub fn take(&self) -> f32 {
        f32::from_bits(self.0.swap(0f32.to_bits(), Ordering::Relaxed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_loads_and_tracks_the_maximum() {
        let a = AtomicF32::new(0.25);
        assert_eq!(a.load(), 0.25);
        a.fetch_max(0.5);
        a.fetch_max(0.1);
        assert_eq!(a.take(), 0.5);
        assert_eq!(a.load(), 0.0);
    }
}
```

`crates/fp-engine/src/ramp.rs`:
```rust
//! Per-sample gain ramps. Fades use an equal-power (sine/cosine) curve so a
//! crossfade keeps constant loudness; de-click ramps are linear.

use std::f32::consts::FRAC_PI_2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Curve {
    Linear,
    EqualPower,
}

/// Moves a gain from `from` to `to` over `len` samples, then holds `to`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ramp {
    from: f32,
    to: f32,
    len: u32,
    pos: u32,
    curve: Curve,
}

impl Ramp {
    /// A constant gain.
    pub fn hold(gain: f32) -> Self {
        Self {
            from: gain,
            to: gain,
            len: 0,
            pos: 0,
            curve: Curve::Linear,
        }
    }

    /// Starts a ramp from the current value of `self` to `to`.
    pub fn retarget(&mut self, to: f32, len: u32, curve: Curve) {
        *self = Self {
            from: self.value(),
            to,
            len,
            pos: 0,
            curve,
        };
    }

    /// Current gain without advancing.
    pub fn value(&self) -> f32 {
        if self.pos >= self.len {
            return self.to;
        }
        let t = self.pos as f32 / self.len as f32;
        let shaped = match self.curve {
            Curve::Linear => t,
            Curve::EqualPower if self.to >= self.from => (t * FRAC_PI_2).sin(),
            Curve::EqualPower => 1.0 - (t * FRAC_PI_2).cos(),
        };
        self.from + (self.to - self.from) * shaped
    }

    /// Returns the gain for the next sample and advances.
    pub fn next_gain(&mut self) -> f32 {
        let g = self.value();
        if self.pos < self.len {
            self.pos += 1;
        }
        g
    }

    pub fn is_done(&self) -> bool {
        self.pos >= self.len
    }

    pub fn target(&self) -> f32 {
        self.to
    }

    /// Samples left before the ramp reaches its target.
    pub fn remaining(&self) -> u32 {
        self.len - self.pos.min(self.len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_ramp_hits_exact_values_and_holds() {
        let mut r = Ramp::hold(0.0);
        r.retarget(1.0, 4, Curve::Linear);
        let g: Vec<f32> = (0..6).map(|_| r.next_gain()).collect();
        assert_eq!(g, vec![0.0, 0.25, 0.5, 0.75, 1.0, 1.0]);
        assert!(r.is_done());
    }

    #[test]
    fn equal_power_fade_out_keeps_constant_power_against_a_fade_in() {
        let mut out = Ramp::hold(1.0);
        out.retarget(0.0, 100, Curve::EqualPower);
        let mut inn = Ramp::hold(0.0);
        inn.retarget(1.0, 100, Curve::EqualPower);
        for _ in 0..100 {
            let (a, b) = (out.next_gain(), inn.next_gain());
            assert!((a * a + b * b - 1.0).abs() < 1e-5, "{a} {b}");
        }
        assert_eq!(out.next_gain(), 0.0);
    }

    #[test]
    fn retarget_starts_from_the_current_value() {
        let mut r = Ramp::hold(0.0);
        r.retarget(1.0, 2, Curve::Linear);
        r.next_gain();
        r.retarget(0.0, 2, Curve::Linear);
        assert_eq!(r.next_gain(), 0.5);
        assert_eq!(r.next_gain(), 0.25);
        assert_eq!(r.next_gain(), 0.0);
    }
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-engine`
Expected: 4 tests PASS.

- [ ] **Step: Check and commit**

Run: `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
Expected: everything passes, no warnings.
```bash
git add crates/fp-engine Cargo.lock
git commit -m "feat(engine): add AtomicF32 and equal-power gain ramps

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: Sources — the lock-free ring between worker and mixer

**Files:**
- Create: `crates/fp-engine/src/source.rs` (unit tests inside)
- Modify: `crates/fp-engine/src/lib.rs`

**Interfaces:**
- Produces:
  - `SOURCE_CHANNELS = 2` and `source_pair(capacity_frames) -> (SourceProducer, SourceConsumer)`.
  - `SourceShared` holds the atomics `frames_played`, `eof`, `failed`, `ready`, `underruns`, `peak_l` and `peak_r`, with the accessors `frames_played()`, `is_ready()`, `is_eof()` and `is_failed()`.
  - `SourceProducer`: `free_frames()`, `buffered_frames()`, `push(&[f32]) -> usize` (samples consumed; whole frames only) and `is_abandoned()`.
  - `SourceConsumer`: `pop_frames(&mut [f32]) -> usize` (frames; RT-safe) and `is_abandoned()`.
  - Both halves expose `pub shared: Arc<SourceShared>`.

- [ ] **Step 1: Write the failing tests**

Create `crates/fp-engine/src/source.rs` with only its `#[cfg(test)] mod tests` block from Step 3.

`crates/fp-engine/src/lib.rs` (whole file):
```rust
//! The audio engine (spec §4): decoded `Source`s feed per-device bus `Mixer`s
//! on real-time threads; per-player worker threads decode and resample; the
//! `Conductor` turns model `EngineAction`s into sample-accurate bus commands.

#![deny(clippy::indexing_slicing)]

pub mod atomic;
pub mod ramp;
pub mod source;
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-engine source`
Expected: FAIL to compile: `cannot find function source_pair`.

- [ ] **Step 3: Implement (whole file)**

`crates/fp-engine/src/source.rs`:
```rust
//! A `Source` is one decoded stream (spec §4.1). The worker thread owns the
//! producer half and fills a lock-free ring of interleaved stereo `f32` at the
//! bus rate; a bus mixer owns the consumer half. Everything else they share is
//! atomic.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use crate::atomic::AtomicF32;

/// Sources are always stereo; mono is duplicated and multichannel downmixed.
pub const SOURCE_CHANNELS: usize = 2;

/// State shared between the producer, the mixer and the conductor.
#[derive(Debug, Default)]
pub struct SourceShared {
    /// Bus frames the mixer has consumed from the ring.
    pub frames_played: AtomicU64,
    /// Set by the worker once the whole stream has been pushed.
    pub eof: AtomicBool,
    /// Set by the worker when the stream could not be opened or decoded.
    pub failed: AtomicBool,
    /// Set by the worker once enough audio is buffered to start (or at eof).
    pub ready: AtomicBool,
    /// Times the mixer found the ring empty before eof.
    pub underruns: AtomicU64,
    /// Peak levels since the UI last read them (after gain).
    pub peak_l: AtomicF32,
    pub peak_r: AtomicF32,
}

impl SourceShared {
    pub fn frames_played(&self) -> u64 {
        self.frames_played.load(Ordering::Acquire)
    }

    pub fn is_ready(&self) -> bool {
        self.ready.load(Ordering::Acquire)
    }

    pub fn is_eof(&self) -> bool {
        self.eof.load(Ordering::Acquire)
    }

    pub fn is_failed(&self) -> bool {
        self.failed.load(Ordering::Acquire)
    }
}

pub struct SourceProducer {
    ring: rtrb::Producer<f32>,
    pub shared: Arc<SourceShared>,
}

pub struct SourceConsumer {
    ring: rtrb::Consumer<f32>,
    pub shared: Arc<SourceShared>,
}

/// Creates a source whose ring holds `capacity_frames` stereo frames.
pub fn source_pair(capacity_frames: usize) -> (SourceProducer, SourceConsumer) {
    let (producer, consumer) = rtrb::RingBuffer::new(capacity_frames.max(1) * SOURCE_CHANNELS);
    let shared = Arc::new(SourceShared::default());
    (
        SourceProducer {
            ring: producer,
            shared: shared.clone(),
        },
        SourceConsumer {
            ring: consumer,
            shared,
        },
    )
}

impl SourceProducer {
    /// Frames that fit in the ring right now.
    pub fn free_frames(&self) -> usize {
        self.ring.slots() / SOURCE_CHANNELS
    }

    /// Frames waiting to be played.
    pub fn buffered_frames(&self) -> usize {
        (self.ring.buffer().capacity() - self.ring.slots()) / SOURCE_CHANNELS
    }

    /// Pushes as many whole frames of `samples` as fit; returns the number of
    /// samples consumed from `samples`.
    pub fn push(&mut self, samples: &[f32]) -> usize {
        let whole = samples.len() - samples.len() % SOURCE_CHANNELS;
        let fit = whole.min(self.free_frames() * SOURCE_CHANNELS);
        let (pushed, _) = self
            .ring
            .push_partial_slice(samples.get(..fit).unwrap_or_default());
        pushed.len()
    }

    /// True when the mixer side has been dropped.
    pub fn is_abandoned(&self) -> bool {
        self.ring.is_abandoned()
    }
}

impl SourceConsumer {
    /// Pops up to `dst.len() / 2` frames into `dst`; returns frames popped.
    /// Never allocates or blocks (real-time safe).
    pub fn pop_frames(&mut self, dst: &mut [f32]) -> usize {
        let whole = dst.len() - dst.len() % SOURCE_CHANNELS;
        let available = self.ring.slots() - self.ring.slots() % SOURCE_CHANNELS;
        let n = whole.min(available);
        match dst.get_mut(..n) {
            Some(target) => self.ring.pop_partial_slice(target).0.len() / SOURCE_CHANNELS,
            None => 0,
        }
    }

    /// True when the producer side has been dropped.
    pub fn is_abandoned(&self) -> bool {
        self.ring.is_abandoned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pushes_and_pops_whole_frames_only() {
        let (mut p, mut c) = source_pair(2);
        assert_eq!(
            p.push(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]),
            4,
            "ring holds 2 frames"
        );
        assert_eq!(p.buffered_frames(), 2);
        let mut dst = [0.0; 3];
        assert_eq!(c.pop_frames(&mut dst), 1);
        assert_eq!(dst, [1.0, 2.0, 0.0]);
        assert_eq!(p.free_frames(), 1);
    }

    #[test]
    fn abandoned_when_the_consumer_is_dropped() {
        let (p, c) = source_pair(4);
        drop(c);
        assert!(p.is_abandoned());
    }
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-engine source`
Expected: 2 tests PASS.

- [ ] **Step: Check and commit**

Run: `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
Expected: everything passes, no warnings.
```bash
git add crates/fp-engine
git commit -m "feat(engine): add lock-free source rings with shared telemetry

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: The real-time bus mixer

**Files:**
- Create: `crates/fp-engine/src/mixer.rs`
- Modify: `crates/fp-engine/src/lib.rs`
- Test: `crates/fp-engine/tests/mixer.rs`

**Interfaces:**
- Consumes: `fp_backends::{Renderer, StreamErrorKind, StreamErrorSink}`, Tasks 3–4.
- Produces (spec §4.3):
  - **`BusCommand`** (all frames are bus frames):
    - `Attach { slot, source: SourceConsumer, volume: Arc<AtomicF32>, first_channel: u16 }`
    - `Start { slot, at_frame }`
    - `Ramp { slot, to, frames, curve, at_frame }`
    - `StopAt { slot, at_frame }`
    - `Pause { slot, ramp_frames }`, `Resume { slot, ramp_frames }`
    - `Cancel { slot }`, `Detach { slot }`
    - `Grow(SlotStorage)`
  - **`BusEvent`:** `Started { slot, frame }` and `Finished { slot, frame }`.
  - **`Retired`:** `Source { slot, source, volume }` or `Storage(SlotStorage)`. Everything that must be freed comes back to the conductor this way.
  - **`SlotStorage::with_capacity(n)`**, with `len()` and `is_empty()`.
  - **`BusShared`** holds these atomics:

    | Atomic | Purpose |
    |---|---|
    | `frames_rendered` | the bus clock |
    | `render_seq` | sequence lock |
    | `heartbeat` | watchdog input |
    | `lost` | device lost |
    | `realtime_denied` | real-time priority refused |
    | `xruns` | backend xruns |
    | `dropped_events` | events that did not fit the queue |
    | `lock_misses` | blocks skipped during a hand-over |
    | `peak_l`, `peak_r` | bus peaks |

    It also has `frames_rendered()`, `heartbeat()` and `consistent(read)`, which reads between two blocks, never mid-block. `BusShared` implements `StreamErrorSink`.
  - **`MixerConfig { volume_smoothing_frames, max_commands_per_block }`**.
  - **`Mixer::new(slots, config) -> (Mixer, MixerHandle)`**, where `MixerHandle` holds the pub fields `commands`, `events`, `retired` and `shared`. `Mixer::render(&mut self, &mut [f32], channels)` is the only RT entry point.
  - **`MixerRenderer { mixer: Arc<Mutex<Mixer>>, shared }`** is the `Renderer` given to backends: it `try_lock`s, and on contention outputs silence and counts a `lock_miss`.
- Gain per sample is `fade ramp × pause ramp × smoothed volume`. Every scheduled change lands on its exact frame because rendering splits chunks at `stop_at` and pending-ramp boundaries.

- [ ] **Step 1: Write the failing tests**

`crates/fp-engine/tests/mixer.rs`:
```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Sample-accurate behaviour of the bus mixer, driven block by block.

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use assert_no_alloc::{AllocDisabler, assert_no_alloc};
use fp_backends::Renderer;
use fp_engine::atomic::AtomicF32;
use fp_engine::mixer::{
    BusCommand, BusEvent, Mixer, MixerConfig, MixerHandle, MixerRenderer, Retired, SlotStorage,
};
use fp_engine::ramp::Curve;
use fp_engine::source::{SourceProducer, source_pair};

#[cfg(debug_assertions)]
#[global_allocator]
static ALLOCATOR: AllocDisabler = AllocDisabler;

const CONFIG: MixerConfig = MixerConfig {
    volume_smoothing_frames: 1,
    max_commands_per_block: 64,
};

fn mixer(slots: usize) -> (Mixer, MixerHandle) {
    Mixer::new(slots, CONFIG)
}

/// A source whose left samples are 1, 2, 3… and right samples are negative.
fn counting_source(
    frames: usize,
    eof: bool,
) -> (SourceProducer, fp_engine::source::SourceConsumer) {
    let (mut p, c) = source_pair(frames.max(1));
    let samples: Vec<f32> = (1..=frames).flat_map(|i| [i as f32, -(i as f32)]).collect();
    assert_eq!(p.push(&samples), samples.len());
    if eof {
        p.shared.eof.store(true, Ordering::Release);
    }
    (p, c)
}

fn full_volume() -> Arc<AtomicF32> {
    Arc::new(AtomicF32::new(1.0))
}

fn attach(
    h: &mut MixerHandle,
    slot: usize,
    source: fp_engine::source::SourceConsumer,
    first_channel: u16,
) {
    assert!(
        h.commands
            .push(BusCommand::Attach {
                slot,
                source,
                volume: full_volume(),
                first_channel
            })
            .is_ok()
    );
}

fn send(h: &mut MixerHandle, c: BusCommand) {
    assert!(h.commands.push(c).is_ok());
}

fn render(m: &mut Mixer, frames: usize, channels: usize) -> Vec<f32> {
    let mut out = vec![0.0; frames * channels];
    assert_no_alloc(|| m.render(&mut out, channels));
    out
}

fn events(h: &mut MixerHandle) -> Vec<BusEvent> {
    std::iter::from_fn(|| h.events.pop().ok()).collect()
}

fn left(out: &[f32]) -> Vec<f32> {
    out.chunks(2).map(|f| f[0]).collect()
}

#[test]
fn an_attached_source_is_silent_until_started_then_starts_on_the_exact_frame() {
    let (mut m, mut h) = mixer(4);
    let (_p, c) = counting_source(16, false);
    attach(&mut h, 0, c, 0);
    assert!(render(&mut m, 4, 2).iter().all(|s| *s == 0.0));
    send(
        &mut h,
        BusCommand::Start {
            slot: 0,
            at_frame: 6,
        },
    );
    let out = render(&mut m, 4, 2); // frames 4..8
    assert_eq!(left(&out), vec![0.0, 0.0, 1.0, 2.0]);
    assert_eq!(out[5], -1.0, "right channel carries the right samples");
    assert_eq!(
        events(&mut h),
        vec![BusEvent::Started { slot: 0, frame: 6 }]
    );
}

#[test]
fn sources_are_summed_and_routed_to_their_channel_pair() {
    let (mut m, mut h) = mixer(4);
    let (_p1, a) = counting_source(4, false);
    let (_p2, b) = counting_source(4, false);
    attach(&mut h, 0, a, 0);
    attach(&mut h, 1, b, 2);
    send(
        &mut h,
        BusCommand::Start {
            slot: 0,
            at_frame: 0,
        },
    );
    send(
        &mut h,
        BusCommand::Start {
            slot: 1,
            at_frame: 0,
        },
    );
    let out = render(&mut m, 2, 4);
    assert_eq!(out, vec![1.0, -1.0, 1.0, -1.0, 2.0, -2.0, 2.0, -2.0]);
}

#[test]
fn stop_at_ends_on_the_exact_frame_and_reports_finished() {
    let (mut m, mut h) = mixer(2);
    let (_p, c) = counting_source(16, false);
    attach(&mut h, 0, c, 0);
    send(
        &mut h,
        BusCommand::Start {
            slot: 0,
            at_frame: 0,
        },
    );
    send(
        &mut h,
        BusCommand::StopAt {
            slot: 0,
            at_frame: 3,
        },
    );
    let out = render(&mut m, 5, 2);
    assert_eq!(left(&out), vec![1.0, 2.0, 3.0, 0.0, 0.0]);
    assert_eq!(
        events(&mut h),
        vec![
            BusEvent::Started { slot: 0, frame: 0 },
            BusEvent::Finished { slot: 0, frame: 3 }
        ]
    );
}

#[test]
fn a_drained_source_at_eof_finishes_on_its_last_frame() {
    let (mut m, mut h) = mixer(2);
    let (_p, c) = counting_source(3, true);
    attach(&mut h, 0, c, 0);
    send(
        &mut h,
        BusCommand::Start {
            slot: 0,
            at_frame: 0,
        },
    );
    render(&mut m, 8, 2);
    assert!(events(&mut h).contains(&BusEvent::Finished { slot: 0, frame: 3 }));
}

#[test]
fn an_underrun_is_silent_counted_and_the_timeline_keeps_moving() {
    let (mut m, mut h) = mixer(2);
    let (p, c) = counting_source(2, false);
    let shared = p.shared.clone();
    attach(&mut h, 0, c, 0);
    send(
        &mut h,
        BusCommand::Start {
            slot: 0,
            at_frame: 0,
        },
    );
    let out = render(&mut m, 4, 2);
    assert_eq!(left(&out), vec![1.0, 2.0, 0.0, 0.0]);
    assert_eq!(shared.underruns.load(Ordering::Relaxed), 1);
    assert_eq!(h.shared.frames_rendered(), 4);
    assert!(
        events(&mut h)
            .iter()
            .all(|e| !matches!(e, BusEvent::Finished { .. }))
    );
}

#[test]
fn a_ramp_scheduled_at_a_frame_starts_exactly_there() {
    let (mut m, mut h) = mixer(2);
    let (mut p, c) = source_pair(16);
    p.push(&[1.0; 32]);
    attach(&mut h, 0, c, 0);
    send(
        &mut h,
        BusCommand::Start {
            slot: 0,
            at_frame: 0,
        },
    );
    send(
        &mut h,
        BusCommand::Ramp {
            slot: 0,
            to: 0.0,
            frames: 4,
            curve: Curve::Linear,
            at_frame: 2,
        },
    );
    let out = render(&mut m, 8, 2);
    assert_eq!(left(&out), vec![1.0, 1.0, 1.0, 0.75, 0.5, 0.25, 0.0, 0.0]);
}

#[test]
fn pause_fades_out_holds_the_position_and_resume_continues_from_it() {
    let (mut m, mut h) = mixer(2);
    let (p, c) = counting_source(32, false);
    let shared = p.shared.clone();
    attach(&mut h, 0, c, 0);
    send(
        &mut h,
        BusCommand::Start {
            slot: 0,
            at_frame: 0,
        },
    );
    render(&mut m, 2, 2);
    send(
        &mut h,
        BusCommand::Pause {
            slot: 0,
            ramp_frames: 2,
        },
    );
    let out = render(&mut m, 6, 2);
    assert_eq!(left(&out), vec![3.0, 2.0, 0.0, 0.0, 0.0, 0.0]);
    let held = shared.frames_played();
    render(&mut m, 8, 2);
    assert_eq!(
        shared.frames_played(),
        held,
        "a paused source consumes nothing"
    );
    send(
        &mut h,
        BusCommand::Resume {
            slot: 0,
            ramp_frames: 1,
        },
    );
    let out = render(&mut m, 2, 2);
    assert_eq!(left(&out), vec![0.0, 6.0]);
}

#[test]
fn volume_changes_are_smoothed() {
    let config = MixerConfig {
        volume_smoothing_frames: 4,
        max_commands_per_block: 64,
    };
    let (mut m, mut h) = Mixer::new(2, config);
    let (mut p, c) = source_pair(16);
    p.push(&[1.0; 32]);
    let volume = full_volume();
    assert!(
        h.commands
            .push(BusCommand::Attach {
                slot: 0,
                source: c,
                volume: volume.clone(),
                first_channel: 0
            })
            .is_ok()
    );
    send(
        &mut h,
        BusCommand::Start {
            slot: 0,
            at_frame: 0,
        },
    );
    render(&mut m, 1, 2);
    volume.store(0.0);
    let out = render(&mut m, 5, 2);
    assert_eq!(left(&out), vec![0.75, 0.5, 0.25, 0.0, 0.0]);
}

#[test]
fn detached_sources_and_old_storage_come_back_for_freeing_off_the_rt_thread() {
    let (mut m, mut h) = mixer(1);
    let (_p, c) = counting_source(8, false);
    attach(&mut h, 0, c, 0);
    send(
        &mut h,
        BusCommand::Start {
            slot: 0,
            at_frame: 0,
        },
    );
    render(&mut m, 1, 2);
    send(&mut h, BusCommand::Grow(SlotStorage::with_capacity(3)));
    let out = render(&mut m, 1, 2);
    assert_eq!(
        left(&out),
        vec![2.0],
        "growing keeps playing sources in place"
    );
    assert!(matches!(h.retired.pop(), Ok(Retired::Storage(s)) if s.len() == 1));
    let (_p2, extra) = counting_source(8, false);
    attach(&mut h, 2, extra, 0);
    send(&mut h, BusCommand::Detach { slot: 0 });
    render(&mut m, 1, 2);
    assert!(matches!(
        h.retired.pop(),
        Ok(Retired::Source { slot: 0, .. })
    ));
}

#[test]
fn attaching_to_an_occupied_slot_hands_the_source_back() {
    let (mut m, mut h) = mixer(1);
    let (_p1, a) = counting_source(4, false);
    let (_p2, b) = counting_source(4, false);
    attach(&mut h, 0, a, 0);
    attach(&mut h, 0, b, 0);
    render(&mut m, 1, 2);
    assert!(matches!(
        h.retired.pop(),
        Ok(Retired::Source { slot: 0, .. })
    ));
}

#[test]
fn a_command_flood_is_spread_over_several_blocks() {
    let config = MixerConfig {
        volume_smoothing_frames: 1,
        max_commands_per_block: 4,
    };
    let (mut m, mut h) = Mixer::new(2, config);
    for _ in 0..10 {
        send(&mut h, BusCommand::Cancel { slot: 0 });
    }
    render(&mut m, 1, 2);
    assert_eq!(h.commands.slots(), h.commands.buffer().capacity() - 6);
}

#[test]
fn the_renderer_outputs_silence_instead_of_waiting_for_a_held_lock() {
    let (m, h) = mixer(1);
    let mixer = Arc::new(Mutex::new(m));
    let mut renderer = MixerRenderer {
        mixer: mixer.clone(),
        shared: h.shared.clone(),
    };
    let _guard = mixer.lock().unwrap();
    let mut out = vec![1.0; 8];
    renderer.render(&mut out, 2);
    assert!(out.iter().all(|s| *s == 0.0));
    assert_eq!(h.shared.lock_misses.load(Ordering::Relaxed), 1);
}
```

The `AllocDisabler` global allocator aborts the test process if anything allocates inside `assert_no_alloc(…)`. Every render in these tests runs under it, including `Attach`, `Detach` and `Grow` handling.

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-engine --test mixer`
Expected: FAIL to compile: `could not find mixer in fp_engine`.

- [ ] **Step 3: Implement**

`crates/fp-engine/src/lib.rs` (whole file):
```rust
//! The audio engine (spec §4): decoded `Source`s feed per-device bus `Mixer`s
//! on real-time threads; per-player worker threads decode and resample; the
//! `Conductor` turns model `EngineAction`s into sample-accurate bus commands.

#![deny(clippy::indexing_slicing)]

pub mod atomic;
pub mod mixer;
pub mod ramp;
pub mod source;
```

`crates/fp-engine/src/mixer.rs`:
```rust
//! The bus mixer (spec §4.3). It runs on the device's real-time thread: it
//! sums the sources attached to its slots into the output, applying gain ramps
//! at exact frames. It never allocates, frees, blocks, logs or panics: memory
//! only arrives and leaves through lock-free queues.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use fp_backends::{Renderer, StreamErrorKind, StreamErrorSink};

use crate::atomic::AtomicF32;
use crate::ramp::{Curve, Ramp};
use crate::source::{SOURCE_CHANNELS, SourceConsumer};

/// Frames processed per inner step; bounds the stack scratch buffer.
const CHUNK_FRAMES: usize = 256;

/// Work for the mixer. All frame numbers are in bus frames
/// (`BusShared::frames_rendered` time).
pub enum BusCommand {
    /// Puts a source in `slot`, silent until `Start`.
    Attach {
        slot: usize,
        source: SourceConsumer,
        volume: Arc<AtomicF32>,
        first_channel: u16,
    },
    /// Starts playing at `at_frame` (or immediately if it has passed).
    Start { slot: usize, at_frame: u64 },
    /// From `at_frame`, moves the fade gain to `to` over `frames` frames.
    Ramp {
        slot: usize,
        to: f32,
        frames: u32,
        curve: Curve,
        at_frame: u64,
    },
    /// Stops at `at_frame` and reports `Finished`.
    StopAt { slot: usize, at_frame: u64 },
    /// Fades out over `ramp_frames`, then holds the position.
    Pause { slot: usize, ramp_frames: u32 },
    /// Fades back in over `ramp_frames` from where it paused.
    Resume { slot: usize, ramp_frames: u32 },
    /// Forgets a pending start, ramp or stop that has not happened yet.
    Cancel { slot: usize },
    /// Removes the source; it comes back through the retired queue.
    Detach { slot: usize },
    /// Replaces the slot storage with a larger one (built off the RT thread).
    Grow(SlotStorage),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusEvent {
    Started {
        slot: usize,
        frame: u64,
    },
    /// The source ended (stop frame reached, or ring drained after eof).
    Finished {
        slot: usize,
        frame: u64,
    },
}

/// Memory handed back to the conductor to be freed off the RT thread.
pub enum Retired {
    Source {
        slot: usize,
        source: SourceConsumer,
        volume: Arc<AtomicF32>,
    },
    Storage(SlotStorage),
}

/// Slot storage, allocated by the conductor.
pub struct SlotStorage(Vec<Option<Slot>>);

impl SlotStorage {
    pub fn with_capacity(slots: usize) -> Self {
        Self((0..slots).map(|_| None).collect())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Bus telemetry, readable from any thread. Also receives backend errors.
#[derive(Debug, Default)]
pub struct BusShared {
    pub frames_rendered: AtomicU64,
    /// Odd while a block is being rendered (a sequence lock for readers).
    pub render_seq: AtomicU64,
    pub heartbeat: AtomicU64,
    pub lost: AtomicBool,
    pub realtime_denied: AtomicBool,
    pub xruns: AtomicU64,
    pub dropped_events: AtomicU64,
    pub lock_misses: AtomicU64,
    pub peak_l: AtomicF32,
    pub peak_r: AtomicF32,
}

impl BusShared {
    pub fn frames_rendered(&self) -> u64 {
        self.frames_rendered.load(Ordering::Acquire)
    }

    pub fn heartbeat(&self) -> u64 {
        self.heartbeat.load(Ordering::Acquire)
    }

    /// Runs `read` so that it observes a state between two blocks, never
    /// half-way through one (source positions and the bus clock agree).
    pub fn consistent<T>(&self, read: impl Fn() -> T) -> T {
        loop {
            let before = self.render_seq.load(Ordering::Acquire);
            if before % 2 == 1 {
                std::hint::spin_loop();
                continue;
            }
            let value = read();
            if self.render_seq.load(Ordering::Acquire) == before {
                return value;
            }
        }
    }
}

impl StreamErrorSink for BusShared {
    fn report(&self, kind: StreamErrorKind) {
        match kind {
            StreamErrorKind::DeviceLost => self.lost.store(true, Ordering::Release),
            StreamErrorKind::RealtimeDenied => self.realtime_denied.store(true, Ordering::Release),
            StreamErrorKind::Xrun | StreamErrorKind::Other => {
                self.xruns.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MixerConfig {
    /// Frames over which a volume change is smoothed.
    pub volume_smoothing_frames: u32,
    /// Maximum commands applied per block.
    pub max_commands_per_block: usize,
}

pub struct Slot {
    source: SourceConsumer,
    volume: Arc<AtomicF32>,
    volume_now: f32,
    first_channel: usize,
    start_at: Option<u64>,
    started: bool,
    fade: Ramp,
    pending_fade: Option<(u64, f32, u32, Curve)>,
    stop_at: Option<u64>,
    pause: Ramp,
    pausing: bool,
    paused: bool,
    finished: bool,
}

/// The conductor's side of a mixer.
pub struct MixerHandle {
    pub commands: rtrb::Producer<BusCommand>,
    pub events: rtrb::Consumer<BusEvent>,
    pub retired: rtrb::Consumer<Retired>,
    pub shared: Arc<BusShared>,
}

pub struct Mixer {
    slots: SlotStorage,
    commands: rtrb::Consumer<BusCommand>,
    events: rtrb::Producer<BusEvent>,
    retired: rtrb::Producer<Retired>,
    /// Retired items the queue had no room for; retried every block.
    backlog: Option<Retired>,
    shared: Arc<BusShared>,
    config: MixerConfig,
    volume_step: f32,
}

impl Mixer {
    /// Creates a mixer with `slots` slots and the queues to drive it.
    pub fn new(slots: usize, config: MixerConfig) -> (Mixer, MixerHandle) {
        let queue = 1024.max(slots * 8);
        let (cmd_tx, cmd_rx) = rtrb::RingBuffer::new(queue);
        let (ev_tx, ev_rx) = rtrb::RingBuffer::new(queue);
        let (ret_tx, ret_rx) = rtrb::RingBuffer::new(queue);
        let shared = Arc::new(BusShared::default());
        let volume_step = 1.0 / config.volume_smoothing_frames.max(1) as f32;
        let mixer = Mixer {
            slots: SlotStorage::with_capacity(slots),
            commands: cmd_rx,
            events: ev_tx,
            retired: ret_tx,
            backlog: None,
            shared: shared.clone(),
            config,
            volume_step,
        };
        (
            mixer,
            MixerHandle {
                commands: cmd_tx,
                events: ev_rx,
                retired: ret_rx,
                shared,
            },
        )
    }

    pub fn shared(&self) -> &Arc<BusShared> {
        &self.shared
    }

    fn emit(&mut self, event: BusEvent) {
        if self.events.push(event).is_err() {
            self.shared.dropped_events.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn retire(&mut self, item: Retired) {
        if self.backlog.is_some() {
            // Keep the oldest; this one waits in its slot (see `detach`).
            return;
        }
        if let Err(rtrb::PushError::Full(item)) = self.retired.push(item) {
            self.backlog = Some(item);
        }
    }

    fn slot_mut(&mut self, slot: usize) -> Option<&mut Slot> {
        self.slots.0.get_mut(slot).and_then(Option::as_mut)
    }

    fn apply(&mut self, command: BusCommand, now: u64) {
        match command {
            BusCommand::Attach {
                slot,
                source,
                volume,
                first_channel,
            } => {
                let volume_now = volume.load();
                if let Some(cell) = self.slots.0.get_mut(slot)
                    && cell.is_none()
                {
                    *cell = Some(Slot {
                        source,
                        volume,
                        volume_now,
                        first_channel: usize::from(first_channel),
                        start_at: None,
                        started: false,
                        fade: Ramp::hold(1.0),
                        pending_fade: None,
                        stop_at: None,
                        pause: Ramp::hold(1.0),
                        pausing: false,
                        paused: false,
                        finished: false,
                    });
                } else {
                    // Occupied or out of range: hand the source straight back.
                    self.retire(Retired::Source {
                        slot,
                        source,
                        volume,
                    });
                }
            }
            BusCommand::Start { slot, at_frame } => {
                if let Some(s) = self.slot_mut(slot)
                    && !s.started
                {
                    s.start_at = Some(at_frame.max(now));
                }
            }
            BusCommand::Ramp {
                slot,
                to,
                frames,
                curve,
                at_frame,
            } => {
                if let Some(s) = self.slot_mut(slot) {
                    if at_frame <= now {
                        s.fade.retarget(to, frames, curve);
                        s.pending_fade = None;
                    } else {
                        s.pending_fade = Some((at_frame, to, frames, curve));
                    }
                }
            }
            BusCommand::StopAt { slot, at_frame } => {
                if let Some(s) = self.slot_mut(slot) {
                    s.stop_at = Some(at_frame.max(now));
                }
            }
            BusCommand::Pause { slot, ramp_frames } => {
                if let Some(s) = self.slot_mut(slot)
                    && !s.paused
                {
                    s.pause.retarget(0.0, ramp_frames, Curve::Linear);
                    s.pausing = true;
                }
            }
            BusCommand::Resume { slot, ramp_frames } => {
                if let Some(s) = self.slot_mut(slot) {
                    s.paused = false;
                    s.pausing = false;
                    s.pause.retarget(1.0, ramp_frames, Curve::Linear);
                }
            }
            BusCommand::Cancel { slot } => {
                if let Some(s) = self.slot_mut(slot) {
                    if !s.started {
                        s.start_at = None;
                    }
                    s.pending_fade = None;
                    s.stop_at = None;
                }
            }
            BusCommand::Detach { slot } => {
                if let Some(cell) = self.slots.0.get_mut(slot)
                    && let Some(s) = cell.take()
                {
                    self.retire(Retired::Source {
                        slot,
                        source: s.source,
                        volume: s.volume,
                    });
                }
            }
            BusCommand::Grow(mut storage) => {
                if storage.len() >= self.slots.len() {
                    for (new, old) in storage.0.iter_mut().zip(self.slots.0.iter_mut()) {
                        *new = old.take();
                    }
                    std::mem::swap(&mut self.slots, &mut storage);
                }
                self.retire(Retired::Storage(storage));
            }
        }
    }

    /// Mixes one block into `out` (interleaved, `channels` per frame).
    pub fn render(&mut self, out: &mut [f32], channels: usize) {
        self.shared.render_seq.fetch_add(1, Ordering::AcqRel);
        out.fill(0.0);
        let channels = channels.max(1);
        let frames = out.len() / channels;
        let now = self.shared.frames_rendered.load(Ordering::Relaxed);

        if let Some(item) = self.backlog.take()
            && let Err(rtrb::PushError::Full(item)) = self.retired.push(item)
        {
            self.backlog = Some(item);
        }
        for _ in 0..self.config.max_commands_per_block {
            match self.commands.pop() {
                Ok(command) => self.apply(command, now),
                Err(_) => break,
            }
        }

        let mut peak_l = 0.0f32;
        let mut peak_r = 0.0f32;
        let volume_step = self.volume_step;
        for index in 0..self.slots.len() {
            let Some(Some(slot)) = self.slots.0.get_mut(index) else {
                continue;
            };
            let mut events: [Option<BusEvent>; 2] = [None, None];
            let (l, r) = render_slot(
                slot,
                index,
                out,
                channels,
                frames,
                now,
                volume_step,
                &mut events,
            );
            peak_l = peak_l.max(l);
            peak_r = peak_r.max(r);
            for event in events.into_iter().flatten() {
                self.emit(event);
            }
        }
        self.shared.peak_l.fetch_max(peak_l);
        self.shared.peak_r.fetch_max(peak_r);
        self.shared
            .frames_rendered
            .store(now + frames as u64, Ordering::Release);
        self.shared.heartbeat.fetch_add(1, Ordering::Release);
        self.shared.render_seq.fetch_add(1, Ordering::AcqRel);
    }
}

/// Renders one slot into `out`. Returns the (left, right) peaks it produced.
#[allow(clippy::too_many_arguments)]
fn render_slot(
    slot: &mut Slot,
    index: usize,
    out: &mut [f32],
    channels: usize,
    frames: usize,
    block_start: u64,
    volume_step: f32,
    events: &mut [Option<BusEvent>; 2],
) -> (f32, f32) {
    if slot.finished {
        return (0.0, 0.0);
    }
    let block_end = block_start + frames as u64;
    let mut f = 0usize;
    if !slot.started {
        match slot.start_at {
            Some(at) if at < block_end => {
                slot.started = true;
                f = at.saturating_sub(block_start) as usize;
                if let Some(e) = events.get_mut(0) {
                    *e = Some(BusEvent::Started {
                        slot: index,
                        frame: block_start + f as u64,
                    });
                }
            }
            _ => return (0.0, 0.0),
        }
    }
    let target_volume = slot.volume.load().clamp(0.0, 1.0);
    let mut chunk = [0.0f32; CHUNK_FRAMES * SOURCE_CHANNELS];
    let (mut peak_l, mut peak_r) = (0.0f32, 0.0f32);
    while f < frames && !slot.paused {
        let abs = block_start + f as u64;
        if slot.stop_at.is_some_and(|stop| abs >= stop) {
            finish(slot, index, abs, events);
            break;
        }
        if let Some((at, to, len, curve)) = slot.pending_fade
            && abs >= at
        {
            slot.fade.retarget(to, len, curve);
            slot.pending_fade = None;
        }
        // Process up to the next scheduled boundary so every change lands on its exact frame.
        let mut n = (frames - f).min(CHUNK_FRAMES);
        if let Some(stop) = slot.stop_at {
            n = n.min((stop - abs) as usize);
        }
        if let Some((at, ..)) = slot.pending_fade {
            n = n.min((at - abs) as usize);
        }
        if slot.pausing {
            n = n.min((slot.pause.remaining() as usize).max(1));
        }
        let Some(buf) = chunk.get_mut(..n * SOURCE_CHANNELS) else {
            break;
        };
        let got = slot.source.pop_frames(buf);
        for (k, &[l, r]) in buf
            .as_chunks::<SOURCE_CHANNELS>()
            .0
            .iter()
            .take(got)
            .enumerate()
        {
            if slot.volume_now < target_volume {
                slot.volume_now = (slot.volume_now + volume_step).min(target_volume);
            } else if slot.volume_now > target_volume {
                slot.volume_now = (slot.volume_now - volume_step).max(target_volume);
            }
            let g = slot.fade.next_gain() * slot.pause.next_gain() * slot.volume_now;
            let (l, r) = (l * g, r * g);
            peak_l = peak_l.max(l.abs());
            peak_r = peak_r.max(r.abs());
            let base = (f + k) * channels + slot.first_channel;
            if let Some(o) = out.get_mut(base) {
                *o += l;
            }
            if slot.first_channel + 1 < channels
                && let Some(o) = out.get_mut(base + 1)
            {
                *o += r;
            }
        }
        slot.source
            .shared
            .frames_played
            .fetch_add(got as u64, Ordering::AcqRel);
        if slot.pausing && slot.pause.is_done() {
            slot.pausing = false;
            slot.paused = true;
        }
        if got < n {
            if slot.source.shared.is_eof() {
                finish(slot, index, abs + got as u64, events);
                break;
            }
            slot.source.shared.underruns.fetch_add(1, Ordering::Relaxed);
            // The missing frames stay silent; the timeline keeps moving.
            f += n;
        } else {
            f += got;
        }
    }
    slot.source.shared.peak_l.fetch_max(peak_l);
    slot.source.shared.peak_r.fetch_max(peak_r);
    (peak_l, peak_r)
}

fn finish(slot: &mut Slot, index: usize, frame: u64, events: &mut [Option<BusEvent>; 2]) {
    slot.finished = true;
    if let Some(e) = events.get_mut(1) {
        *e = Some(BusEvent::Finished { slot: index, frame });
    }
}

/// Adapts a shared mixer to a backend `Renderer`. The real-time thread only
/// ever `try_lock`s; on contention (a hand-over in progress) it outputs
/// silence for that block and counts the miss.
pub struct MixerRenderer {
    pub mixer: Arc<std::sync::Mutex<Mixer>>,
    pub shared: Arc<BusShared>,
}

impl Renderer for MixerRenderer {
    fn render(&mut self, out: &mut [f32], channels: usize) {
        match self.mixer.try_lock() {
            Ok(mut mixer) => mixer.render(out, channels),
            Err(std::sync::TryLockError::Poisoned(poisoned)) => {
                poisoned.into_inner().render(out, channels)
            }
            Err(std::sync::TryLockError::WouldBlock) => {
                out.fill(0.0);
                self.shared.lock_misses.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-engine --test mixer`
Expected: 12 tests PASS.

- [ ] **Step 5: Prove the no-allocation guard works**

Create a temporary `crates/fp-engine/tests/zz_alloc_sanity.rs`:
```rust
use assert_no_alloc::{AllocDisabler, assert_no_alloc};
#[global_allocator]
static A: AllocDisabler = AllocDisabler;
#[test]
fn detects() { assert_no_alloc(|| { let v: Vec<u8> = Vec::with_capacity(10); std::hint::black_box(v); }); }
```
Run: `cargo test -p fp-engine --test zz_alloc_sanity`
Expected: the process aborts (`SIGABRT`). This proves the mixer tests would catch an allocation. Then delete the file.

- [ ] **Step: Check and commit**

Run: `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
Expected: everything passes, no warnings.
```bash
git add crates/fp-engine
git commit -m "feat(engine): add real-time bus mixer with sample-accurate scheduling

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 6: Decoding, resampling and the per-player worker thread

**Files:**
- Create: `crates/fp-engine/src/decode.rs`, `crates/fp-engine/src/resample.rs`, `crates/fp-engine/src/worker.rs`
- Modify: `crates/fp-engine/src/lib.rs`
- Test: `crates/fp-engine/tests/support/mod.rs`, `crates/fp-engine/tests/decode.rs`, `crates/fp-engine/tests/worker.rs`

**Interfaces:**
- Produces:
  - **`FileDecoder`:** `open(&Path) -> Result<Self, String>`, `sample_rate()`, `seek(secs)` (sample-exact: it drops the frames between where the demuxer landed and the target) and `next_block(&mut Vec<f32>) -> Result<bool, String>`.
    - Output is interleaved stereo at the file rate.
    - Mono is duplicated; more channels are downmixed with ITU-R BS.775 weights and the LFE is dropped.
    - Corrupt packets are skipped.
  - **`StreamResampler`:** `new(from, to)`, `push(&[f32], &mut Vec<f32>)` and `finish(&mut Vec<f32>)`. The filter delay is removed, so the output length is exactly `input × ratio`.
  - **Source abstraction:** `trait SampleSource: Send { fn next_block(&mut self, &mut Vec<f32>) -> Result<bool, String> }` and `type SourceOpener = Arc<dyn Fn(&Path, f64, u32) -> Result<Box<dyn SampleSource>, String> + Send + Sync>`. `file_opener()` combines decoding, seeking and resampling to the bus rate.
  - **Worker plumbing:** `SourceKey(pub u64)`, `WorkerCommand`, `WorkerFailure { key, error }`.
  - **`PlayerWorker`:** `spawn(name, opener, bus_rate, ready_frames, Sender<WorkerFailure>)`, `load(key, path, from_secs, producer)` and `drop_source(key)`. Dropping it joins the thread.
- Worker rules (spec §4.5):
  - serve the emptiest ring first;
  - set `ready` at `ready_frames`, on a full ring, or at eof;
  - set `eof` only after the last sample is pushed;
  - any `Err` or panic inside a source marks it `failed`, reports `WorkerFailure` and leaves the thread serving other sources;
  - an abandoned producer (the mixer dropped the consumer) is forgotten.

- [ ] **Step 1: Write the test helpers and the failing tests**

`crates/fp-engine/tests/support/mod.rs`:
```rust
#![allow(
    dead_code,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Shared test helpers: WAV fixtures and synthetic sources.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use fp_engine::worker::{SampleSource, SourceOpener};

/// Writes a 16-bit WAV whose sample `i` (per channel) is `(i % 20_000) as i16`,
/// so any position can be identified from its value.
pub fn indexed_wav(dir: &Path, name: &str, rate: u32, channels: u16, frames: usize) -> PathBuf {
    let path = dir.join(name);
    let spec = hound::WavSpec {
        channels,
        sample_rate: rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(&path, spec).unwrap();
    for i in 0..frames {
        for _ in 0..channels {
            w.write_sample((i % 20_000) as i16).unwrap();
        }
    }
    w.finalize().unwrap();
    path
}

/// The sample index encoded by `indexed_wav`, recovered from a decoded value.
pub fn index_of(sample: f32) -> i64 {
    (f64::from(sample) * 32_768.0).round() as i64
}

/// Emits `total` frames whose left sample is the frame number and right is its negation.
pub struct Counting {
    pub next: u64,
    pub total: u64,
    pub block: u64,
}

impl SampleSource for Counting {
    fn next_block(&mut self, out: &mut Vec<f32>) -> Result<bool, String> {
        if self.next >= self.total {
            return Ok(false);
        }
        let end = (self.next + self.block).min(self.total);
        for i in self.next..end {
            out.push(i as f32);
            out.push(-(i as f32));
        }
        self.next = end;
        Ok(true)
    }
}

/// Opener producing `Counting` sources of `total` frames. The path is ignored.
pub fn counting_opener(total: u64) -> SourceOpener {
    Arc::new(move |_path, from_secs, rate| {
        let start = (from_secs * f64::from(rate)).round() as u64;
        Ok(Box::new(Counting {
            next: start,
            total,
            block: 64,
        }) as Box<dyn SampleSource>)
    })
}

/// A source that panics on its first block.
pub struct Exploding;

impl SampleSource for Exploding {
    fn next_block(&mut self, _out: &mut Vec<f32>) -> Result<bool, String> {
        panic!("boom");
    }
}

/// Opener for paths named `track<N>`: sample `i` (from the start of the file)
/// of track N is `N * 100_000 + i` on both channels, and every track is
/// `frames` long. Any other path fails to open.
pub fn tagged_opener(frames: u64) -> SourceOpener {
    Arc::new(move |path, from_secs, rate| {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let n: u64 = name
            .strip_prefix("track")
            .and_then(|d| d.parse().ok())
            .ok_or_else(|| format!("cannot open {name}"))?;
        let start = (from_secs * f64::from(rate)).round() as u64;
        Ok(Box::new(Tagged {
            base: n * 100_000,
            next: start,
            total: frames,
        }) as Box<dyn SampleSource>)
    })
}

pub struct Tagged {
    base: u64,
    next: u64,
    total: u64,
}

impl SampleSource for Tagged {
    fn next_block(&mut self, out: &mut Vec<f32>) -> Result<bool, String> {
        if self.next >= self.total {
            return Ok(false);
        }
        let end = (self.next + 512).min(self.total);
        for i in self.next..end {
            let v = (self.base + i) as f32;
            out.push(v);
            out.push(v);
        }
        self.next = end;
        Ok(true)
    }
}
```

`crates/fp-engine/tests/decode.rs`:
```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

mod support;

use fp_engine::decode::FileDecoder;
use fp_engine::worker::file_opener;
use support::{index_of, indexed_wav};

fn decode_all(d: &mut FileDecoder) -> Vec<f32> {
    let mut out = Vec::new();
    while d.next_block(&mut out).unwrap() {}
    out
}

#[test]
fn mono_files_decode_to_duplicated_stereo_with_every_frame() {
    let dir = tempfile::tempdir().unwrap();
    let path = indexed_wav(dir.path(), "mono.wav", 44_100, 1, 88_200);
    let mut d = FileDecoder::open(&path).unwrap();
    assert_eq!(d.sample_rate(), 44_100);
    let out = decode_all(&mut d);
    assert_eq!(out.len(), 88_200 * 2);
    assert_eq!((index_of(out[20]), index_of(out[21])), (10, 10));
}

#[test]
fn seeking_lands_on_the_exact_frame() {
    let dir = tempfile::tempdir().unwrap();
    let path = indexed_wav(dir.path(), "seek.wav", 48_000, 2, 96_000);
    let mut d = FileDecoder::open(&path).unwrap();
    d.seek(1.5).unwrap();
    let out = decode_all(&mut d);
    assert_eq!(index_of(out[0]), 72_000 % 20_000);
    assert_eq!(out.len(), (96_000 - 72_000) * 2);
}

#[test]
fn the_file_opener_resamples_to_the_bus_rate_with_an_exact_length() {
    let dir = tempfile::tempdir().unwrap();
    let path = indexed_wav(dir.path(), "cd.wav", 44_100, 2, 44_100);
    let mut source = file_opener()(&path, 0.0, 48_000).unwrap();
    let mut out = Vec::new();
    while source.next_block(&mut out).unwrap() {}
    assert_eq!(out.len(), 48_000 * 2);
}

#[test]
fn a_file_that_is_not_audio_is_reported_with_its_path() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.mp3");
    std::fs::write(&path, b"definitely not an mp3").unwrap();
    let error = FileDecoder::open(&path).err().unwrap();
    assert!(error.contains("notes.mp3"), "{error}");
    assert!(FileDecoder::open(&dir.path().join("missing.flac")).is_err());
}
```

`crates/fp-engine/tests/worker.rs`:
```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

mod support;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use fp_engine::source::{SourceConsumer, source_pair};
use fp_engine::worker::{PlayerWorker, SampleSource, SourceKey, SourceOpener, WorkerFailure};
use support::{Exploding, counting_opener};

fn wait_until(what: &str, mut ok: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !ok() {
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn drain(c: &mut SourceConsumer, into: &mut Vec<f32>) {
    let mut buf = [0.0f32; 512];
    loop {
        let n = c.pop_frames(&mut buf);
        if n == 0 {
            return;
        }
        into.extend_from_slice(&buf[..n * 2]);
    }
}

fn worker(opener: SourceOpener) -> (PlayerWorker, crossbeam_channel::Receiver<WorkerFailure>) {
    let (tx, rx) = crossbeam_channel::unbounded();
    (
        PlayerWorker::spawn("test-worker", opener, 48_000, 100, tx).unwrap(),
        rx,
    )
}

#[test]
fn fills_the_ring_marks_ready_and_eof_and_keeps_order() {
    let (w, _failures) = worker(counting_opener(1_000));
    let (p, mut c) = source_pair(4_000);
    let shared = p.shared.clone();
    w.load(SourceKey(1), PathBuf::from("x"), 0.0, p);
    wait_until("eof", || shared.is_eof());
    assert!(shared.is_ready());
    let mut got = Vec::new();
    drain(&mut c, &mut got);
    let lefts: Vec<f32> = got.chunks(2).map(|f| f[0]).collect();
    assert_eq!(lefts, (0..1_000).map(|i| i as f32).collect::<Vec<_>>());
}

#[test]
fn refills_a_small_ring_as_it_is_consumed() {
    let (w, _failures) = worker(counting_opener(10_000));
    let (p, mut c) = source_pair(256);
    let shared = p.shared.clone();
    w.load(SourceKey(1), PathBuf::from("x"), 0.0, p);
    let mut got = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !(shared.is_eof() && got.len() >= 20_000) {
        assert!(Instant::now() < deadline);
        drain(&mut c, &mut got);
        std::thread::sleep(Duration::from_micros(200));
    }
    drain(&mut c, &mut got);
    assert_eq!(got.len(), 20_000);
    assert!(got.chunks(2).enumerate().all(|(i, f)| f[0] == i as f32));
}

#[test]
fn starts_at_the_requested_position() {
    let (w, _failures) = worker(counting_opener(96_000));
    let (p, mut c) = source_pair(128);
    w.load(SourceKey(1), PathBuf::from("x"), 1.0, p);
    let mut first = [0.0f32; 2];
    wait_until("first frame", || c.pop_frames(&mut first) == 1);
    assert_eq!(first[0], 48_000.0);
}

#[test]
fn an_open_failure_is_reported_and_marks_the_source_failed() {
    let opener: SourceOpener = Arc::new(|_, _, _| Err("cannot open".to_owned()));
    let (w, failures) = worker(opener);
    let (p, _c) = source_pair(64);
    let shared = p.shared.clone();
    w.load(SourceKey(7), PathBuf::from("x"), 0.0, p);
    let failure = failures.recv_timeout(Duration::from_secs(5)).unwrap();
    assert_eq!(
        failure,
        WorkerFailure {
            key: SourceKey(7),
            error: "cannot open".to_owned()
        }
    );
    assert!(shared.is_failed());
}

#[test]
fn a_panicking_decoder_is_contained_and_the_worker_keeps_serving() {
    let opener: SourceOpener = Arc::new(|path, _, _| {
        if path.to_string_lossy() == "bomb" {
            Ok(Box::new(Exploding) as Box<dyn SampleSource>)
        } else {
            counting_opener(10)(path, 0.0, 48_000)
        }
    });
    let (w, failures) = worker(opener);
    let (p1, _c1) = source_pair(64);
    w.load(SourceKey(1), PathBuf::from("bomb"), 0.0, p1);
    let failure = failures.recv_timeout(Duration::from_secs(5)).unwrap();
    assert_eq!(failure.key, SourceKey(1));
    assert!(failure.error.contains("panicked"));
    let (p2, _c2) = source_pair(64);
    let shared = p2.shared.clone();
    w.load(SourceKey(2), PathBuf::from("fine"), 0.0, p2);
    wait_until("second source eof", || shared.is_eof());
}

#[test]
fn dropping_a_source_releases_its_producer() {
    let (w, _failures) = worker(counting_opener(1_000_000));
    let (p, c) = source_pair(64);
    w.load(SourceKey(3), PathBuf::from("x"), 0.0, p);
    wait_until("ready", || c.shared.is_ready());
    w.drop_source(SourceKey(3));
    wait_until("producer dropped", || c.is_abandoned());
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-engine --test decode --test worker`
Expected: FAIL to compile: `could not find decode` / `worker in fp_engine`.

- [ ] **Step 3: Implement**

`crates/fp-engine/src/lib.rs` (whole file):
```rust
//! The audio engine (spec §4): decoded `Source`s feed per-device bus `Mixer`s
//! on real-time threads; per-player worker threads decode and resample; the
//! `Conductor` turns model `EngineAction`s into sample-accurate bus commands.

#![deny(clippy::indexing_slicing)]

pub mod atomic;
pub mod decode;
pub mod mixer;
pub mod ramp;
pub mod resample;
pub mod source;
pub mod worker;
```

`crates/fp-engine/src/decode.rs`:
```rust
//! File decoding with symphonia (pure-Rust codecs). Output is interleaved
//! stereo `f32` at the file's own rate: mono is duplicated, more channels are
//! downmixed with ITU-R BS.775 coefficients.

use std::fs::File;
use std::path::Path;

use symphonia::core::codecs::audio::{AudioDecoder, AudioDecoderOptions};
use symphonia::core::errors::Error;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo, TrackType};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::units::Time;

/// -3 dB, the ITU-R BS.775 weight for centre and surround channels.
const MINUS_3DB: f32 = std::f32::consts::FRAC_1_SQRT_2;

pub struct FileDecoder {
    format: Box<dyn FormatReader>,
    decoder: Box<dyn AudioDecoder>,
    track_id: u32,
    sample_rate: u32,
    scratch: Vec<f32>,
    /// Frames still to drop after a seek (the decoder lands before the target).
    skip_frames: u64,
}

impl FileDecoder {
    pub fn open(path: &Path) -> Result<Self, String> {
        let file = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());
        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }
        let format = symphonia::default::get_probe()
            .probe(
                &hint,
                mss,
                FormatOptions::default(),
                MetadataOptions::default(),
            )
            .map_err(|e| format!("{}: {e}", path.display()))?;
        let track = format
            .default_track(TrackType::Audio)
            .ok_or_else(|| format!("{}: no audio track", path.display()))?;
        let params = track
            .codec_params
            .as_ref()
            .and_then(|p| p.audio())
            .ok_or_else(|| format!("{}: no audio parameters", path.display()))?;
        let sample_rate = params
            .sample_rate
            .ok_or_else(|| format!("{}: unknown sample rate", path.display()))?;
        let decoder = symphonia::default::get_codecs()
            .make_audio_decoder(params, &AudioDecoderOptions::default())
            .map_err(|e| format!("{}: {e}", path.display()))?;
        let track_id = track.id;
        Ok(Self {
            format,
            decoder,
            track_id,
            sample_rate,
            scratch: Vec::new(),
            skip_frames: 0,
        })
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Positions the stream so that the next frame produced is at `secs`.
    pub fn seek(&mut self, secs: f64) -> Result<(), String> {
        if secs <= 0.0 {
            return Ok(());
        }
        let time = Time::try_from_secs_f64(secs).ok_or("seek position out of range")?;
        let seeked = self
            .format
            .seek(
                SeekMode::Accurate,
                SeekTo::Time {
                    time,
                    track_id: Some(self.track_id),
                },
            )
            .map_err(|e| e.to_string())?;
        self.decoder.reset();
        let time_base = self
            .format
            .tracks()
            .iter()
            .find(|t| t.id == self.track_id)
            .and_then(|t| t.time_base);
        let landed = time_base.map_or(secs, |tb| {
            tb.calc_time_saturating(seeked.actual_ts).as_secs_f64()
        });
        let behind = (secs - landed).max(0.0);
        self.skip_frames = (behind * f64::from(self.sample_rate)).round() as u64;
        Ok(())
    }

    /// Appends the next decoded packet as interleaved stereo. Returns `false`
    /// at the end of the stream. Corrupt packets are skipped.
    pub fn next_block(&mut self, out: &mut Vec<f32>) -> Result<bool, String> {
        loop {
            let packet = match self.format.next_packet() {
                Ok(Some(packet)) => packet,
                Ok(None) => return Ok(false),
                Err(e) => return Err(e.to_string()),
            };
            if packet.track_id != self.track_id {
                continue;
            }
            let buf = match self.decoder.decode(&packet) {
                Ok(buf) => buf,
                Err(Error::DecodeError(_)) => continue,
                Err(e) => return Err(e.to_string()),
            };
            let channels = buf.spec().channels().count().max(1);
            let frames = buf.frames();
            self.scratch.resize(frames * channels, 0.0);
            buf.copy_to_slice_interleaved(self.scratch.as_mut_slice());
            let skip = usize::try_from(self.skip_frames)
                .unwrap_or(usize::MAX)
                .min(frames);
            self.skip_frames -= skip as u64;
            for frame in self.scratch.chunks_exact(channels).skip(skip) {
                let (l, r) = downmix(frame);
                out.push(l);
                out.push(r);
            }
            return Ok(true);
        }
    }
}

/// Stereo from any channel count. Order follows the WAV/SMPTE convention
/// (L, R, C, LFE, Ls, Rs, …); the LFE channel is dropped.
fn downmix(frame: &[f32]) -> (f32, f32) {
    match *frame {
        [m] => (m, m),
        [l, r] => (l, r),
        [l, r, c] => (l + MINUS_3DB * c, r + MINUS_3DB * c),
        [l, r, c, _lfe] => (l + MINUS_3DB * c, r + MINUS_3DB * c),
        [l, r, c, _lfe, ls, rs, ..] => {
            let norm = 1.0 / (1.0 + 2.0 * MINUS_3DB);
            (
                (l + MINUS_3DB * (c + ls)) * norm,
                (r + MINUS_3DB * (c + rs)) * norm,
            )
        }
        [l, r, ..] => (l, r),
        [] => (0.0, 0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::downmix;

    #[test]
    fn downmix_duplicates_mono_and_folds_centre_into_both_sides() {
        assert_eq!(downmix(&[0.5]), (0.5, 0.5));
        let (l, r) = downmix(&[0.0, 0.0, 1.0]);
        assert!((l - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-6 && (r - l).abs() < 1e-6);
    }
}
```

`crates/fp-engine/src/resample.rs`:
```rust
//! Streaming sample-rate conversion of interleaved stereo with rubato's
//! windowed-sinc resampler. The filter delay is removed and the output length
//! is exactly `input_frames × ratio`, so timelines stay accurate.

use audioadapter_buffers::direct::InterleavedSlice;
use rubato::{Async, FixedAsync, Indexing, Resampler, SincInterpolationParameters};

const CHUNK_FRAMES: usize = 1024;

pub struct StreamResampler {
    inner: Async<f32>,
    pending: Vec<f32>,
    out: Vec<f32>,
    ratio: f64,
    frames_in: u64,
    frames_out: u64,
    skip: usize,
}

impl StreamResampler {
    pub fn new(from_rate: u32, to_rate: u32) -> Result<Self, String> {
        let ratio = f64::from(to_rate) / f64::from(from_rate.max(1));
        let params = SincInterpolationParameters::default();
        let inner = Async::<f32>::new_sinc(ratio, 1.0, &params, CHUNK_FRAMES, 2, FixedAsync::Input)
            .map_err(|e| e.to_string())?;
        let out = vec![0.0; inner.output_frames_max() * 2];
        let skip = inner.output_delay();
        Ok(Self {
            inner,
            pending: Vec::new(),
            out,
            ratio,
            frames_in: 0,
            frames_out: 0,
            skip,
        })
    }

    /// Feeds interleaved stereo input; appends every complete output chunk to `dst`.
    pub fn push(&mut self, input: &[f32], dst: &mut Vec<f32>) -> Result<(), String> {
        self.pending.extend_from_slice(input);
        self.frames_in += (input.len() / 2) as u64;
        loop {
            let need = self.inner.input_frames_next();
            if self.pending.len() < need * 2 {
                return Ok(());
            }
            self.run(need, None, dst)?;
        }
    }

    /// Flushes the tail at end of stream, trimming the output to its exact length.
    pub fn finish(&mut self, dst: &mut Vec<f32>) -> Result<(), String> {
        let start = dst.len();
        let expected = (self.frames_in as f64 * self.ratio).round() as u64;
        while self.frames_out < expected {
            let need = self.inner.input_frames_next();
            let have = self.pending.len() / 2;
            self.pending.resize(need * 2, 0.0);
            let partial = if have > 0 && have < need {
                Some(have)
            } else {
                None
            };
            let before = self.frames_out;
            self.run(need, partial, dst)?;
            if self.frames_out == before && have == 0 {
                break;
            }
        }
        if self.frames_out > expected {
            let extra = usize::try_from(self.frames_out - expected).unwrap_or(usize::MAX) * 2;
            let keep = dst.len().saturating_sub(extra).max(start);
            dst.truncate(keep);
            self.frames_out = expected;
        }
        Ok(())
    }

    fn run(
        &mut self,
        need: usize,
        partial: Option<usize>,
        dst: &mut Vec<f32>,
    ) -> Result<(), String> {
        let capacity = self.out.len() / 2;
        let input_samples = self
            .pending
            .get(..need * 2)
            .ok_or("resampler input underflow")?;
        let input = InterleavedSlice::new(input_samples, 2, need).map_err(|e| e.to_string())?;
        let mut output =
            InterleavedSlice::new_mut(&mut self.out, 2, capacity).map_err(|e| e.to_string())?;
        let indexing = match partial {
            Some(frames) => Indexing::new().partial_len(frames),
            None => Indexing::new(),
        };
        let (_read, written) = self
            .inner
            .process_into_buffer(&input, &mut output, Some(&indexing))
            .map_err(|e| e.to_string())?;
        let skip = self.skip.min(written);
        self.skip -= skip;
        dst.extend_from_slice(self.out.get(skip * 2..written * 2).unwrap_or_default());
        self.frames_out += (written - skip) as u64;
        self.pending.drain(..need * 2);
        Ok(())
    }
}
```

`crates/fp-engine/src/worker.rs`:
```rust
//! One decoding thread per player (spec §4.5). It opens, decodes and
//! resamples every source of its player and keeps their rings topped up,
//! serving the emptiest ring first. Failures — including panics inside a
//! decoder — are contained to the source that caused them.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::thread::JoinHandle;
use std::time::Duration;

use crossbeam_channel::{Receiver, Sender, TryRecvError};

use crate::decode::FileDecoder;
use crate::resample::StreamResampler;
use crate::source::SourceProducer;

/// Produces interleaved stereo at the bus rate.
pub trait SampleSource: Send {
    /// Appends the next block to `out`; `Ok(false)` at end of stream.
    fn next_block(&mut self, out: &mut Vec<f32>) -> Result<bool, String>;
}

/// Opens a `SampleSource` for `path`, positioned at `from_secs`, at `bus_rate`.
pub type SourceOpener =
    Arc<dyn Fn(&Path, f64, u32) -> Result<Box<dyn SampleSource>, String> + Send + Sync>;

/// Opener for real files: symphonia decoding plus rubato resampling.
pub fn file_opener() -> SourceOpener {
    Arc::new(|path, from_secs, bus_rate| {
        let mut decoder = FileDecoder::open(path)?;
        decoder.seek(from_secs)?;
        let resampler = if decoder.sample_rate() == bus_rate {
            None
        } else {
            Some(StreamResampler::new(decoder.sample_rate(), bus_rate)?)
        };
        Ok(Box::new(FileSource {
            decoder,
            resampler,
            block: Vec::new(),
            finished: false,
        }))
    })
}

struct FileSource {
    decoder: FileDecoder,
    resampler: Option<StreamResampler>,
    block: Vec<f32>,
    finished: bool,
}

impl SampleSource for FileSource {
    fn next_block(&mut self, out: &mut Vec<f32>) -> Result<bool, String> {
        if self.finished {
            return Ok(false);
        }
        let Some(resampler) = self.resampler.as_mut() else {
            return self.decoder.next_block(out);
        };
        self.block.clear();
        if self.decoder.next_block(&mut self.block)? {
            resampler.push(&self.block, out)?;
        } else {
            resampler.finish(out)?;
            self.finished = true;
        }
        Ok(true)
    }
}

/// Identifies a source within its worker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceKey(pub u64);

pub enum WorkerCommand {
    Load {
        key: SourceKey,
        path: PathBuf,
        from_secs: f64,
        producer: SourceProducer,
    },
    Drop {
        key: SourceKey,
    },
    Shutdown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerFailure {
    pub key: SourceKey,
    pub error: String,
}

struct Job {
    key: SourceKey,
    path: PathBuf,
    from_secs: f64,
    producer: SourceProducer,
    source: Option<Box<dyn SampleSource>>,
    pending: Vec<f32>,
    done: bool,
}

/// Handle to a running worker thread; dropping it stops the thread.
pub struct PlayerWorker {
    commands: Sender<WorkerCommand>,
    thread: Option<JoinHandle<()>>,
}

impl PlayerWorker {
    /// Spawns the worker. A source becomes ready once `ready_frames` are
    /// buffered; failures are sent on `failures`.
    pub fn spawn(
        name: &str,
        opener: SourceOpener,
        bus_rate: u32,
        ready_frames: usize,
        failures: Sender<WorkerFailure>,
    ) -> std::io::Result<Self> {
        let (tx, rx) = crossbeam_channel::unbounded();
        let thread = std::thread::Builder::new()
            .name(name.to_owned())
            .spawn(move || run(&rx, &opener, bus_rate, ready_frames, &failures))?;
        Ok(Self {
            commands: tx,
            thread: Some(thread),
        })
    }

    pub fn load(&self, key: SourceKey, path: PathBuf, from_secs: f64, producer: SourceProducer) {
        let _ = self.commands.send(WorkerCommand::Load {
            key,
            path,
            from_secs,
            producer,
        });
    }

    pub fn drop_source(&self, key: SourceKey) {
        let _ = self.commands.send(WorkerCommand::Drop { key });
    }
}

impl Drop for PlayerWorker {
    fn drop(&mut self) {
        let _ = self.commands.send(WorkerCommand::Shutdown);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn run(
    commands: &Receiver<WorkerCommand>,
    opener: &SourceOpener,
    bus_rate: u32,
    ready_frames: usize,
    failures: &Sender<WorkerFailure>,
) {
    let mut jobs: Vec<Job> = Vec::new();
    loop {
        let busy = jobs.iter().any(|j| !j.done && j.producer.free_frames() > 0);
        let first = if busy {
            match commands.try_recv() {
                Ok(c) => Some(c),
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => return,
            }
        } else {
            match commands.recv_timeout(Duration::from_millis(10)) {
                Ok(c) => Some(c),
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => None,
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => return,
            }
        };
        for command in first
            .into_iter()
            .chain(std::iter::from_fn(|| commands.try_recv().ok()))
        {
            match command {
                WorkerCommand::Load {
                    key,
                    path,
                    from_secs,
                    producer,
                } => {
                    jobs.retain(|j| j.key != key);
                    jobs.push(Job {
                        key,
                        path,
                        from_secs,
                        producer,
                        source: None,
                        pending: Vec::new(),
                        done: false,
                    });
                }
                WorkerCommand::Drop { key } => jobs.retain(|j| j.key != key),
                WorkerCommand::Shutdown => return,
            }
        }
        jobs.retain(|j| !j.producer.is_abandoned());
        // Serve the job with the least audio buffered first.
        if let Some(job) = jobs
            .iter_mut()
            .filter(|j| !j.done && j.producer.free_frames() > 0)
            .min_by_key(|j| j.producer.buffered_frames())
        {
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                step(job, opener, bus_rate, ready_frames)
            }));
            let error = match outcome {
                Ok(Ok(())) => None,
                Ok(Err(e)) => Some(e),
                Err(_) => Some("decoder panicked".to_owned()),
            };
            if let Some(error) = error {
                job.done = true;
                job.producer.shared.failed.store(true, Ordering::Release);
                job.producer.shared.ready.store(true, Ordering::Release);
                let _ = failures.send(WorkerFailure {
                    key: job.key,
                    error,
                });
            }
        }
    }
}

/// Opens the job if needed and moves at most one decoded block into its ring.
fn step(
    job: &mut Job,
    opener: &SourceOpener,
    bus_rate: u32,
    ready_frames: usize,
) -> Result<(), String> {
    if job.source.is_none() {
        job.source = Some(opener(&job.path, job.from_secs, bus_rate)?);
    }
    if job.pending.is_empty() {
        let more = match job.source.as_mut() {
            Some(source) => source.next_block(&mut job.pending)?,
            None => false,
        };
        if !more && job.pending.is_empty() {
            job.done = true;
            job.producer.shared.eof.store(true, Ordering::Release);
            job.producer.shared.ready.store(true, Ordering::Release);
            return Ok(());
        }
    }
    let pushed = job.producer.push(&job.pending);
    job.pending.drain(..pushed);
    if job.producer.buffered_frames() >= ready_frames || job.producer.free_frames() == 0 {
        job.producer.shared.ready.store(true, Ordering::Release);
    }
    Ok(())
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-engine --test decode --test worker && cargo test -p fp-engine --lib`
Expected: 4 decode tests, 6 worker tests and the unit tests PASS. The panic test prints a `boom` panic message; that is expected.

- [ ] **Step: Check and commit**

Run: `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
Expected: everything passes, no warnings.
```bash
git add crates/fp-engine
git commit -m "feat(engine): add decoding, resampling and per-player worker threads

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 7: The bus — device, watchdog, virtual clock and reconnection

**Files:**
- Create: `crates/fp-engine/src/bus.rs`
- Modify: `crates/fp-engine/src/lib.rs`
- Test: `crates/fp-engine/tests/bus.rs`

**Interfaces:**
- Consumes: `fp_backends::{AudioBackend, DeviceId, OutputStream, StreamConfig, StreamErrorSink}`, Task 5.
- Produces (spec §4.6–4.7):
  - `BusKey { backend: String, device: String }`, `BusHealth { Ok, Lost }`, `BusTiming { watchdog_timeout, reconnect_interval }`.
  - `Bus::open(key, backend, config, slots, mixer_config, timing, now) -> Bus` **never fails**: an unopenable device starts `Lost` on the virtual clock.
  - Other `Bus` methods:
    - state: `key()`, `health()`, `last_error()`, `sample_rate()`, `now_frame()`, `shared()`;
    - supervision: `supervise(now)`, which runs the watchdog and the reconnect every `reconnect_interval`;
    - slots: `alloc_slot()`, `send(BusCommand) -> bool`, `ensure_capacity(slots)` (a `Grow` with no audio interruption), `capacity()`, and `poll() -> Vec<BusEvent>`, which also frees slots handed back through `Retired`.
- Every method that depends on time takes `now: Instant`, so tests drive the watchdog without sleeping.

- [ ] **Step 1: Write the failing tests**

`crates/fp-engine/tests/bus.rs`:
```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Device loss, the virtual clock and reconnection (spec §4.7).

use std::sync::Arc;
use std::time::{Duration, Instant};

use fp_backends::{OfflineBackend, OfflineDevice, StreamConfig};
use fp_engine::atomic::AtomicF32;
use fp_engine::bus::{Bus, BusHealth, BusKey, BusTiming};
use fp_engine::mixer::{BusCommand, MixerConfig};
use fp_engine::source::source_pair;

const CONFIG: StreamConfig = StreamConfig {
    sample_rate: 48_000,
    buffer_frames: 480,
    channels: 2,
};
const MIXER: MixerConfig = MixerConfig {
    volume_smoothing_frames: 1,
    max_commands_per_block: 64,
};
const TIMING: BusTiming = BusTiming {
    watchdog_timeout: Duration::from_millis(500),
    reconnect_interval: Duration::from_secs(2),
};

fn setup(plugged: bool) -> (OfflineBackend, OfflineDevice, Bus, Instant) {
    let backend = OfflineBackend::new();
    let device = backend.add_device("card", 2);
    if !plugged {
        device.unplug();
    }
    let key = BusKey {
        backend: "offline".into(),
        device: "card".into(),
    };
    let t0 = Instant::now();
    let bus = Bus::open(key, Arc::new(backend.clone()), CONFIG, 4, MIXER, TIMING, t0);
    (backend, device, bus, t0)
}

#[test]
fn a_healthy_device_drives_the_bus_clock() {
    let (_b, device, mut bus, t0) = setup(true);
    assert_eq!(bus.health(), BusHealth::Ok);
    device.render(480).unwrap();
    device.render(480).unwrap();
    bus.supervise(t0 + Duration::from_millis(20));
    assert_eq!(bus.now_frame(), 960);
    assert_eq!(bus.health(), BusHealth::Ok);
}

#[test]
fn unplugging_switches_to_the_virtual_clock_and_the_timeline_keeps_moving() {
    let (_b, device, mut bus, t0) = setup(true);
    device.render(480).unwrap();
    device.unplug();
    bus.supervise(t0 + Duration::from_millis(10));
    assert_eq!(bus.health(), BusHealth::Lost);
    let before = bus.now_frame();
    std::thread::sleep(Duration::from_millis(150));
    let advanced = bus.now_frame() - before;
    assert!(
        advanced >= 2_400,
        "virtual clock rendered only {advanced} frames"
    );
}

#[test]
fn a_silent_device_is_declared_lost_by_the_watchdog() {
    let (_b, _device, mut bus, t0) = setup(true);
    bus.supervise(t0 + Duration::from_millis(400));
    assert_eq!(bus.health(), BusHealth::Ok);
    bus.supervise(t0 + Duration::from_millis(600));
    assert_eq!(bus.health(), BusHealth::Lost);
}

#[test]
fn a_returning_device_takes_the_mixer_back_without_losing_time() {
    let (_b, device, mut bus, t0) = setup(true);
    device.unplug();
    bus.supervise(t0);
    std::thread::sleep(Duration::from_millis(60));
    device.replug();
    bus.supervise(t0 + Duration::from_secs(1));
    assert_eq!(
        bus.health(),
        BusHealth::Lost,
        "retries wait for the reconnect interval"
    );
    bus.supervise(t0 + Duration::from_secs(3));
    assert_eq!(bus.health(), BusHealth::Ok);
    let at_handover = bus.now_frame();
    assert!(at_handover > 0);
    std::thread::sleep(Duration::from_millis(40));
    assert_eq!(
        bus.now_frame(),
        at_handover,
        "the virtual clock has stopped"
    );
    device.render(480).unwrap();
    assert_eq!(
        bus.now_frame(),
        at_handover + 480,
        "the device continues the same timeline"
    );
}

#[test]
fn a_device_missing_at_startup_starts_lost_on_the_virtual_clock() {
    let (_b, _device, bus, _t0) = setup(false);
    assert_eq!(bus.health(), BusHealth::Lost);
    assert!(bus.last_error().is_some());
    std::thread::sleep(Duration::from_millis(60));
    assert!(bus.now_frame() > 0);
}

#[test]
fn slots_are_reused_only_after_the_mixer_hands_them_back() {
    let (_b, device, mut bus, _t0) = setup(true);
    let slot = bus.alloc_slot().unwrap();
    let (_p, c) = source_pair(16);
    assert!(bus.send(BusCommand::Attach {
        slot,
        source: c,
        volume: Arc::new(AtomicF32::new(1.0)),
        first_channel: 0
    }));
    assert!(bus.send(BusCommand::Detach { slot }));
    for _ in 0..3 {
        bus.alloc_slot().unwrap();
    }
    assert_eq!(bus.alloc_slot(), None, "all four slots are taken");
    device.render(16).unwrap();
    bus.poll();
    assert_eq!(bus.alloc_slot(), Some(slot));
    bus.ensure_capacity(6);
    assert_eq!(bus.capacity(), 6);
    device.render(16).unwrap();
    assert!(bus.alloc_slot().is_some());
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-engine --test bus`
Expected: FAIL to compile: `could not find bus in fp_engine`.

- [ ] **Step 3: Implement**

`crates/fp-engine/src/lib.rs` (whole file):
```rust
//! The audio engine (spec §4): decoded `Source`s feed per-device bus `Mixer`s
//! on real-time threads; per-player worker threads decode and resample; the
//! `Conductor` turns model `EngineAction`s into sample-accurate bus commands.

#![deny(clippy::indexing_slicing)]

pub mod atomic;
pub mod bus;
pub mod decode;
pub mod mixer;
pub mod ramp;
pub mod resample;
pub mod source;
pub mod worker;
```

`crates/fp-engine/src/bus.rs`:
```rust
//! A `Bus` is one open output device and its mixer (spec §4.6–4.7). If the
//! device fails — an error from the backend, or no heartbeat for
//! `watchdog_timeout` — a virtual-clock thread keeps rendering the same mixer
//! at real-time pace, so player timelines never stall; the device is retried
//! every `reconnect_interval` and takes the mixer back when it returns.
//!
//! Every method takes the current `Instant` explicitly so tests can drive time.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use fp_backends::{AudioBackend, DeviceId, OutputStream, StreamConfig, StreamErrorSink};

use crate::mixer::{
    BusCommand, BusEvent, Mixer, MixerConfig, MixerHandle, MixerRenderer, Retired, SlotStorage,
};

/// Identifies an output device across backends.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BusKey {
    pub backend: String,
    pub device: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusHealth {
    Ok,
    Lost,
}

#[derive(Debug, Clone, Copy)]
pub struct BusTiming {
    pub watchdog_timeout: Duration,
    pub reconnect_interval: Duration,
}

struct VirtualClock {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl VirtualClock {
    fn start(mixer: Arc<Mutex<Mixer>>, config: StreamConfig) -> Option<Self> {
        let stop = Arc::new(AtomicBool::new(false));
        let flag = stop.clone();
        let channels = usize::from(config.channels.max(1));
        let frames = config.buffer_frames.max(1) as usize;
        let period = Duration::from_secs_f64(
            f64::from(config.buffer_frames.max(1)) / f64::from(config.sample_rate.max(1)),
        );
        let thread = std::thread::Builder::new()
            .name("fp-virtual-clock".to_owned())
            .spawn(move || {
                let mut buffer = vec![0.0f32; frames * channels];
                let mut deadline = Instant::now();
                while !flag.load(Ordering::Acquire) {
                    mixer
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner)
                        .render(&mut buffer, channels);
                    deadline += period;
                    let now = Instant::now();
                    if deadline > now {
                        std::thread::sleep(deadline - now);
                    } else {
                        deadline = now;
                    }
                }
            })
            .ok()?;
        Some(Self {
            stop,
            thread: Some(thread),
        })
    }
}

impl Drop for VirtualClock {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

pub struct Bus {
    key: BusKey,
    backend: Arc<dyn AudioBackend>,
    device: DeviceId,
    config: StreamConfig,
    timing: BusTiming,
    mixer: Arc<Mutex<Mixer>>,
    handle: MixerHandle,
    stream: Option<Box<dyn OutputStream>>,
    virtual_clock: Option<VirtualClock>,
    health: BusHealth,
    last_heartbeat: u64,
    last_beat_at: Instant,
    last_retry: Instant,
    /// Conductor-side view of which slots are in use.
    used: Vec<bool>,
    /// Last error from the backend, for the UI.
    last_error: Option<String>,
}

impl Bus {
    /// Opens the device. Never fails: if the device cannot be opened the bus
    /// starts `Lost`, running on the virtual clock and retrying.
    pub fn open(
        key: BusKey,
        backend: Arc<dyn AudioBackend>,
        config: StreamConfig,
        slots: usize,
        mixer_config: MixerConfig,
        timing: BusTiming,
        now: Instant,
    ) -> Bus {
        let (mixer, handle) = Mixer::new(slots, mixer_config);
        let device = DeviceId(key.device.clone());
        let mut bus = Bus {
            key,
            backend,
            device,
            config,
            timing,
            mixer: Arc::new(Mutex::new(mixer)),
            handle,
            stream: None,
            virtual_clock: None,
            health: BusHealth::Lost,
            last_heartbeat: 0,
            last_beat_at: now,
            last_retry: now,
            used: vec![false; slots],
            last_error: None,
        };
        if !bus.try_open(now) {
            bus.virtual_clock = VirtualClock::start(bus.mixer.clone(), bus.config);
        }
        bus
    }

    fn try_open(&mut self, now: Instant) -> bool {
        self.last_retry = now;
        self.handle.shared.lost.store(false, Ordering::Release);
        let renderer = Box::new(MixerRenderer {
            mixer: self.mixer.clone(),
            shared: self.handle.shared.clone(),
        });
        let errors: Arc<dyn StreamErrorSink> = self.handle.shared.clone();
        match self
            .backend
            .open_output(&self.device, self.config, renderer, errors)
        {
            Ok(stream) => {
                self.stream = Some(stream);
                self.health = BusHealth::Ok;
                self.last_heartbeat = self.handle.shared.heartbeat();
                self.last_beat_at = now;
                self.last_error = None;
                true
            }
            Err(e) => {
                self.last_error = Some(e.to_string());
                false
            }
        }
    }

    pub fn key(&self) -> &BusKey {
        &self.key
    }

    pub fn health(&self) -> BusHealth {
        self.health
    }

    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    pub fn sample_rate(&self) -> u32 {
        self.config.sample_rate
    }

    /// Current bus time in frames.
    pub fn now_frame(&self) -> u64 {
        self.handle.shared.frames_rendered()
    }

    pub fn shared(&self) -> &Arc<crate::mixer::BusShared> {
        &self.handle.shared
    }

    /// Watchdog and reconnection. Call regularly from the conductor.
    pub fn supervise(&mut self, now: Instant) {
        let beat = self.handle.shared.heartbeat();
        if beat != self.last_heartbeat {
            self.last_heartbeat = beat;
            self.last_beat_at = now;
        }
        match self.health {
            BusHealth::Ok => {
                let silent =
                    now.saturating_duration_since(self.last_beat_at) > self.timing.watchdog_timeout;
                if self.handle.shared.lost.load(Ordering::Acquire) || silent {
                    tracing::warn!(bus = ?self.key, silent, "output device lost; switching to the virtual clock");
                    self.stream = None;
                    self.health = BusHealth::Lost;
                    self.last_retry = now;
                    self.virtual_clock = VirtualClock::start(self.mixer.clone(), self.config);
                }
            }
            BusHealth::Lost => {
                if now.saturating_duration_since(self.last_retry) >= self.timing.reconnect_interval
                    && self.try_open(now)
                {
                    tracing::info!(bus = ?self.key, "output device back");
                    // The device renders from now on; stop the stand-in.
                    self.virtual_clock = None;
                }
            }
        }
    }

    /// Reserves a free slot.
    pub fn alloc_slot(&mut self) -> Option<usize> {
        let slot = self.used.iter().position(|u| !u)?;
        if let Some(u) = self.used.get_mut(slot) {
            *u = true;
        }
        Some(slot)
    }

    /// Sends a command; `false` if the queue is full (the caller logs it).
    pub fn send(&mut self, command: BusCommand) -> bool {
        self.handle.commands.push(command).is_ok()
    }

    /// Grows the mixer to at least `slots` slots without interrupting audio.
    pub fn ensure_capacity(&mut self, slots: usize) {
        if slots <= self.used.len() {
            return;
        }
        if self.send(BusCommand::Grow(SlotStorage::with_capacity(slots))) {
            self.used.resize(slots, false);
        }
    }

    pub fn capacity(&self) -> usize {
        self.used.len()
    }

    /// Drains events, and frees memory and slots the mixer handed back.
    pub fn poll(&mut self) -> Vec<BusEvent> {
        while let Ok(retired) = self.handle.retired.pop() {
            if let Retired::Source { slot, .. } = retired
                && let Some(u) = self.used.get_mut(slot)
            {
                *u = false;
            }
        }
        std::iter::from_fn(|| self.handle.events.pop().ok()).collect()
    }
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-engine --test bus`
Expected: 6 tests PASS (≈0.2 s; two of them watch the virtual clock in real time).

- [ ] **Step: Check and commit**

Run: `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
Expected: everything passes, no warnings.
```bash
git add crates/fp-engine
git commit -m "feat(engine): add bus with watchdog, virtual clock and reconnection

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 8: The engine — model actions to sample-accurate mixer commands

**Files:**
- Create: `crates/fp-engine/src/engine.rs`
- Modify: `crates/fp-engine/src/lib.rs`
- Test: `crates/fp-engine/tests/engine.rs`

**Interfaces:**
- Consumes:
  - from `fp_model`: `Config`, `EngineAction`, `EngineEvent`, `EntryId`, `PlayerId`, `PlayerRoutes`, `SourceRequest`, `TransitionPlan`, `Tuning`;
  - Tasks 1 and 3–7.
- Produces:
  - `EngineSettings { sample_rate, buffer_frames, channels, tuning, routes, default_backend }` with `from_config(&Config)`.
  - `PlayerTelemetry { position_secs, cue_position_secs, peak_l, peak_r, underruns }` and `BusStatus { key, health, error }`.
  - `Engine::new(backends, settings, opener)` always appends a `NullBackend` as the last resort.
  - Other `Engine` methods: `execute(EngineAction, now)`, `tick(now) -> Vec<EngineEvent>`, `telemetry(player)`, `bus_status()`, `dropped_commands()`, `describe_player(player) -> String` (diagnostics) and `settings()`.
- Behaviour (spec §4.3–4.4):
  - **Routing:** a route's backend id selects the backend. An unknown id falls back to the configured default, then the first backend. A player without a Main route uses the default device, or Null. A Cue without a route ends immediately with `CueEnded`.
  - **Starting sources:** new sources start only once `ready`, and a resumed or seeked source fades in over `declick_ms`.
  - **Transitions:** a finite plan is dispatched when it is within `schedule_lead_ms`, with an exact `at_frame`. For `StartNextAt`, the preload starts on that frame and the current fades out (equal power until `fade_current_until_secs`, or a de-click cut). `SOURCE_END` plans act when the current source's stream ends.
  - **Transitions taken back:** pause, stop, seek, preload or schedule changes cancel a dispatched but not-yet-executed transition (`undispatch`).
  - **Events:**
    - `TransitionStarted { entry }` when the preload starts; the engine then follows the audio and clears its paused flag;
    - `FadeCompleted` when every outgoing source has finished;
    - `ReachedEnd { entry }` for planned stops, fade stops and a stream that runs out;
    - `SourceFailed { entry }` from worker failures;
    - `CueEnded` when the cue source finishes.
  - **Slots:** a source that never started is freed immediately when stopped, so it never leaks a slot.

- [ ] **Step 1: Write the failing tests**

The synthetic "tagged" sources used here are already in `tests/support/mod.rs` (Task 6): sample *i* of `track<N>` is `N·100000 + i`, so the test can tell exactly which track and frame is audible.

`crates/fp-engine/tests/engine.rs`:
```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The engine executes model actions with sample accuracy (spec §4.3–4.4).

mod support;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use fp_backends::{AudioBackend, OfflineBackend, OfflineDevice};
use fp_engine::engine::{Engine, EngineSettings};
use fp_model::{
    Config, EngineAction, EngineEvent, EntryId, PlayerId, PlayerRoutes, Route, SOURCE_END,
    SourceRequest, TrackId, TransitionPlan,
};
use support::tagged_opener;

const RATE: f64 = 48_000.0;
const BLOCK: usize = 480;
const P: PlayerId = PlayerId(1);

struct Rig {
    engine: Engine,
    main: OfflineDevice,
    cue: OfflineDevice,
    clock: Instant,
    /// Everything rendered on the main device so far (left channel).
    heard: Vec<f32>,
    events: Vec<EngineEvent>,
}

fn request(track: u64, from_secs: f64) -> SourceRequest {
    SourceRequest {
        entry: EntryId(track),
        track: TrackId(track),
        path: PathBuf::from(format!("track{track}")),
        from_secs,
    }
}

fn rig(track_frames: u64, with_cue_route: bool) -> Rig {
    let backend = OfflineBackend::new();
    let main = backend.add_device("main", 2);
    let cue = backend.add_device("cue", 2);
    let mut config = Config::default();
    config.outputs.buffer_frames = BLOCK as u32;
    config.outputs.backend = Some("offline".into());
    config.outputs.routes = vec![PlayerRoutes {
        player: P,
        main: Some(Route {
            backend: "offline".into(),
            device: "main".into(),
            first_channel: 0,
        }),
        cue: with_cue_route.then(|| Route {
            backend: "offline".into(),
            device: "cue".into(),
            first_channel: 0,
        }),
    }];
    config.tuning.gain_smoothing_ms = 0.0;
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
    let mut engine = Engine::new(
        backends,
        EngineSettings::from_config(&config),
        tagged_opener(track_frames),
    );
    let clock = Instant::now();
    engine.execute(EngineAction::AddPlayer { player: P }, clock);
    Rig {
        engine,
        main,
        cue,
        clock,
        heard: Vec::new(),
        events: Vec::new(),
    }
}

impl Rig {
    fn act(&mut self, action: EngineAction) {
        self.engine.execute(action, self.clock);
    }

    /// Lets worker threads fill their rings, then ticks.
    fn settle(&mut self) {
        std::thread::sleep(Duration::from_millis(20));
        let events = self.engine.tick(self.clock);
        self.events.extend(events);
    }

    /// Renders `blocks` blocks on both devices, ticking after each.
    fn run(&mut self, blocks: usize) {
        for _ in 0..blocks {
            let out = self.main.render(BLOCK).unwrap();
            self.heard.extend(out.chunks(2).map(|f| f[0]));
            let _ = self.cue.render(BLOCK);
            self.clock += Duration::from_secs_f64(BLOCK as f64 / RATE);
            let events = self.engine.tick(self.clock);
            self.events.extend(events);
        }
    }

    fn position(&self) -> f64 {
        self.engine.telemetry(P).position_secs.unwrap()
    }
}

fn tag(v: f32) -> (u64, u64) {
    let v = v as u64;
    (v / 100_000, v % 100_000)
}

#[test]
fn start_current_plays_the_track_from_its_first_frame_once_ready() {
    let mut r = rig(96_000, false);
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, 0.0),
    });
    r.settle();
    r.run(2);
    let first = r.heard.iter().position(|v| *v != 0.0).unwrap();
    assert_eq!(tag(r.heard[first]), (1, 0));
    assert_eq!(tag(r.heard[first + 100]), (1, 100));
}

#[test]
fn a_planned_stop_ends_on_the_exact_frame_and_reports_the_entry() {
    let mut r = rig(96_000, false);
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, 0.0),
    });
    r.settle();
    r.run(1);
    let start = r.heard.iter().position(|v| *v != 0.0).unwrap();
    r.act(EngineAction::Schedule {
        player: P,
        plan: Some(TransitionPlan::StopAt { at_secs: 0.5 }),
    });
    r.run(80);
    let stop = start + 24_000;
    assert_eq!(
        tag(r.heard[stop - 241]),
        (1, 23_759),
        "full level before the de-click ramp"
    );
    assert!(
        r.heard[stop..].iter().all(|v| *v == 0.0),
        "silent from the stop frame on"
    );
    assert!(r.events.contains(&EngineEvent::ReachedEnd {
        player: P,
        entry: EntryId(1)
    }));
}

#[test]
fn a_segue_starts_the_next_on_the_exact_frame_and_overlaps_the_fade() {
    let mut r = rig(96_000, false);
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, 0.0),
    });
    r.act(EngineAction::Preload {
        player: P,
        request: Some(request(2, 0.0)),
    });
    r.settle();
    r.run(1);
    let start = r.heard.iter().position(|v| *v != 0.0).unwrap();
    r.act(EngineAction::Schedule {
        player: P,
        plan: Some(TransitionPlan::StartNextAt {
            at_secs: 0.5,
            fade_current_until_secs: Some(0.6),
        }),
    });
    r.run(80);
    let at = start + 24_000;
    assert_eq!(
        r.heard[at - 1],
        123_999.0,
        "only the current before the segue"
    );
    assert_eq!(
        r.heard[at],
        124_000.0 + 200_000.0,
        "the next joins at full level on the exact frame"
    );
    assert_eq!(
        tag(r.heard[at + 4_800 + 10]),
        (2, 4_810),
        "after the fade only the next remains"
    );
    let started = r.events.iter().position(|e| {
        *e == EngineEvent::TransitionStarted {
            player: P,
            entry: EntryId(2),
        }
    });
    let faded = r
        .events
        .iter()
        .position(|e| *e == EngineEvent::FadeCompleted { player: P });
    assert!(started.unwrap() < faded.unwrap());
}

#[test]
fn source_end_plans_chain_when_the_file_runs_out() {
    let mut r = rig(12_000, false);
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, 0.0),
    });
    r.act(EngineAction::Preload {
        player: P,
        request: Some(request(2, 0.0)),
    });
    r.act(EngineAction::Schedule {
        player: P,
        plan: Some(TransitionPlan::StartNextAt {
            at_secs: SOURCE_END,
            fade_current_until_secs: None,
        }),
    });
    r.settle();
    r.run(40);
    assert!(r.events.contains(&EngineEvent::TransitionStarted {
        player: P,
        entry: EntryId(2)
    }));
    let last_of_1 = r.heard.iter().rposition(|v| tag(*v).0 == 1).unwrap();
    let first_of_2 = r.heard.iter().position(|v| tag(*v) == (2, 0)).unwrap();
    assert!(first_of_2 > last_of_1);
    assert!(
        first_of_2 - last_of_1 <= 2 * BLOCK,
        "gap of {} frames",
        first_of_2 - last_of_1
    );
}

#[test]
fn a_crossfade_starts_the_next_now_and_reports_when_the_old_one_is_gone() {
    let mut r = rig(96_000, false);
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, 0.0),
    });
    r.act(EngineAction::Preload {
        player: P,
        request: Some(request(2, 0.0)),
    });
    r.settle();
    r.run(5);
    r.act(EngineAction::Crossfade {
        player: P,
        request: request(2, 0.0),
        fade_ms: 100,
    });
    r.settle();
    r.run(20);
    assert!(r.events.contains(&EngineEvent::FadeCompleted { player: P }));
    assert_eq!(tag(*r.heard.last().unwrap()).0, 2);
}

#[test]
fn fade_out_and_stop_reports_the_end_after_the_fade() {
    let mut r = rig(96_000, false);
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, 0.0),
    });
    r.settle();
    r.run(5);
    r.act(EngineAction::FadeOutAndStop {
        player: P,
        fade_ms: 100,
    });
    r.run(8);
    assert!(
        !r.events
            .iter()
            .any(|e| matches!(e, EngineEvent::ReachedEnd { .. })),
        "not before the fade ends"
    );
    r.run(4);
    assert!(r.events.contains(&EngineEvent::ReachedEnd {
        player: P,
        entry: EntryId(1)
    }));
    assert!(r.heard[r.heard.len() - BLOCK..].iter().all(|v| *v == 0.0));
}

#[test]
fn pause_holds_the_position_and_resume_continues_it() {
    let mut r = rig(96_000, false);
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, 0.0),
    });
    r.settle();
    r.run(10);
    r.act(EngineAction::Pause { player: P });
    r.run(2);
    let held = r.position();
    r.run(10);
    assert_eq!(r.position(), held);
    r.act(EngineAction::Resume { player: P });
    r.run(2);
    assert!(r.position() > held);
}

#[test]
fn load_paused_waits_for_resume_and_starts_at_the_saved_position() {
    let mut r = rig(96_000, false);
    r.act(EngineAction::LoadPaused {
        player: P,
        request: request(1, 1.0),
    });
    r.settle();
    r.run(4);
    assert!(
        r.heard.iter().all(|v| *v == 0.0),
        "nothing goes on air by itself"
    );
    assert_eq!(r.position(), 1.0);
    r.act(EngineAction::Resume { player: P });
    r.settle();
    r.run(4);
    // Resuming fades in from gain 0, so the start frame itself is silent.
    let start = r.heard.iter().position(|v| *v != 0.0).unwrap() - 1;
    assert_eq!(tag(r.heard[start + 300]).1, 48_000 + 300);
}

#[test]
fn seek_replaces_the_source_at_the_new_position() {
    let mut r = rig(96_000, false);
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, 0.0),
    });
    r.settle();
    r.run(2);
    r.act(EngineAction::Seek {
        player: P,
        secs: 1.5,
    });
    r.settle();
    r.run(4);
    assert!(r.position() >= 1.5);
    assert!((tag(*r.heard.last().unwrap()).1 as f64 / RATE) >= 1.5);
}

#[test]
fn an_unreadable_file_is_reported_with_its_entry() {
    let mut r = rig(96_000, false);
    let mut bad = request(1, 0.0);
    bad.path = PathBuf::from("corrupt.mp3");
    r.act(EngineAction::StartCurrent {
        player: P,
        request: bad,
    });
    r.settle();
    assert!(r.events.contains(&EngineEvent::SourceFailed {
        player: P,
        entry: EntryId(1)
    }));
}

#[test]
fn cue_plays_only_on_the_cue_output_and_ends_by_itself() {
    let mut r = rig(4_800, true);
    r.act(EngineAction::StartCue {
        player: P,
        request: request(3, 0.0),
    });
    r.settle();
    let mut cue_heard = Vec::new();
    for _ in 0..20 {
        let out = r.cue.render(BLOCK).unwrap();
        cue_heard.extend(out.chunks(2).map(|f| f[0]));
        r.run(1);
    }
    assert!(cue_heard.iter().any(|v| tag(*v).0 == 3));
    assert!(
        r.heard.iter().all(|v| *v == 0.0),
        "the main output never hears the cue"
    );
    assert!(r.events.contains(&EngineEvent::CueEnded { player: P }));
}

#[test]
fn cue_without_a_cue_output_ends_immediately() {
    let mut r = rig(4_800, false);
    r.act(EngineAction::StartCue {
        player: P,
        request: request(3, 0.0),
    });
    r.settle();
    assert!(r.events.contains(&EngineEvent::CueEnded { player: P }));
}

#[test]
fn pausing_before_a_dispatched_transition_takes_it_back() {
    let mut r = rig(96_000, false);
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, 0.0),
    });
    r.act(EngineAction::Preload {
        player: P,
        request: Some(request(2, 0.0)),
    });
    r.settle();
    r.run(1);
    r.act(EngineAction::Schedule {
        player: P,
        plan: Some(TransitionPlan::StartNextAt {
            at_secs: 0.2,
            fade_current_until_secs: None,
        }),
    });
    r.run(12); // ~0.12 s: the transition is now dispatched (within the 200 ms lead).
    r.act(EngineAction::Pause { player: P });
    r.run(40);
    assert!(
        !r.heard.iter().any(|v| tag(*v).0 == 2),
        "the next must not start while paused"
    );
    r.act(EngineAction::Resume { player: P });
    r.run(40);
    assert!(r.events.contains(&EngineEvent::TransitionStarted {
        player: P,
        entry: EntryId(2)
    }));
}

#[test]
fn a_transition_that_already_happened_when_pause_arrives_leaves_the_player_playing() {
    let mut r = rig(96_000, false);
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, 0.0),
    });
    r.act(EngineAction::Preload {
        player: P,
        request: Some(request(2, 0.0)),
    });
    r.settle();
    r.run(1);
    r.act(EngineAction::Schedule {
        player: P,
        plan: Some(TransitionPlan::StartNextAt {
            at_secs: 0.2,
            fade_current_until_secs: None,
        }),
    });
    r.run(12); // dispatched
    // The audio thread passes the transition frame before the conductor ticks again…
    for _ in 0..12 {
        r.main.render(BLOCK).unwrap();
    }
    // …and the operator's pause is executed before the Started event is seen.
    r.act(EngineAction::Pause { player: P });
    r.run(1);
    assert!(r.events.contains(&EngineEvent::TransitionStarted {
        player: P,
        entry: EntryId(2)
    }));
    // The model now says "playing track 2": the engine must agree.
    r.act(EngineAction::Seek {
        player: P,
        secs: 0.5,
    });
    r.settle();
    r.run(10);
    assert!(
        r.position() > 0.5,
        "the seeked source must play, position {}",
        r.position()
    );
}

#[test]
fn stopping_a_source_that_never_started_frees_its_slot() {
    let mut r = rig(96_000, false);
    for n in 0..40 {
        // Started and stopped before its worker filled it: it never reaches the mixer's clock.
        r.act(EngineAction::StartCurrent {
            player: P,
            request: request(1 + (n % 3), 0.0),
        });
        r.act(EngineAction::StopNow { player: P });
        r.run(1);
    }
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, 0.0),
    });
    r.settle();
    r.run(4);
    assert!(
        r.heard.iter().rev().take(BLOCK).any(|v| *v != 0.0),
        "a slot must still be available"
    );
}

#[test]
fn a_file_shorter_than_its_planned_stop_reports_the_end_when_it_runs_out() {
    let mut r = rig(12_000, false);
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, 0.0),
    });
    r.act(EngineAction::Schedule {
        player: P,
        plan: Some(TransitionPlan::StopAt { at_secs: 10.0 }),
    });
    r.settle();
    r.run(40);
    assert!(r.events.contains(&EngineEvent::ReachedEnd {
        player: P,
        entry: EntryId(1)
    }));
}

#[test]
fn a_route_to_an_unknown_backend_falls_back_to_the_default_backend() {
    let backend = OfflineBackend::new();
    let main = backend.add_device("main", 2);
    let mut config = Config::default();
    config.outputs.buffer_frames = BLOCK as u32;
    config.outputs.routes = vec![PlayerRoutes {
        player: P,
        main: Some(Route {
            backend: "asio".into(),
            device: "main".into(),
            first_channel: 0,
        }),
        cue: None,
    }];
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
    let mut engine = Engine::new(
        backends,
        EngineSettings::from_config(&config),
        tagged_opener(96_000),
    );
    let now = Instant::now();
    engine.execute(EngineAction::AddPlayer { player: P }, now);
    engine.execute(
        EngineAction::StartCurrent {
            player: P,
            request: request(1, 0.0),
        },
        now,
    );
    std::thread::sleep(Duration::from_millis(20));
    engine.tick(now);
    let out = main.render(BLOCK).unwrap();
    assert!(
        out.iter().any(|v| *v != 0.0),
        "the player must not go silent"
    );
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-engine --test engine`
Expected: FAIL to compile: `could not find engine in fp_engine`.

- [ ] **Step 3: Implement**

`crates/fp-engine/src/lib.rs` (whole file):
```rust
//! The audio engine (spec §4): decoded `Source`s feed per-device bus `Mixer`s
//! on real-time threads; per-player worker threads decode and resample; the
//! `Conductor` turns model `EngineAction`s into sample-accurate bus commands.

#![deny(clippy::indexing_slicing)]

pub mod atomic;
pub mod bus;
pub mod decode;
pub mod engine;
pub mod mixer;
pub mod ramp;
pub mod resample;
pub mod source;
pub mod worker;
```

`crates/fp-engine/src/engine.rs`:
```rust
//! Executes model `EngineAction`s on buses and worker threads, and turns what
//! the buses and workers observe into model `EngineEvent`s (spec §4.4). All
//! timing is in bus frames: transitions are dispatched ahead of time
//! (`schedule_lead`) with an exact frame, so they never depend on when this
//! thread happens to wake.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, Sender};
use fp_backends::{AudioBackend, NullBackend, StreamConfig};
use fp_model::{
    Config, EngineAction, EngineEvent, EntryId, PlayerId, PlayerRoutes, SourceRequest,
    TransitionPlan, Tuning,
};

use crate::atomic::AtomicF32;
use crate::bus::{Bus, BusHealth, BusKey, BusTiming};
use crate::mixer::{BusCommand, BusEvent, MixerConfig};
use crate::ramp::Curve;
use crate::source::{SourceShared, source_pair};
use crate::worker::{PlayerWorker, SourceKey, SourceOpener, WorkerFailure};

/// Stream settings and tuning, derived from the model `Config`.
#[derive(Debug, Clone)]
pub struct EngineSettings {
    pub sample_rate: u32,
    pub buffer_frames: u32,
    pub channels: u16,
    pub tuning: Tuning,
    pub routes: Vec<PlayerRoutes>,
    /// Backend used when a player has no explicit route (`None`: the first registered).
    pub default_backend: Option<String>,
}

impl EngineSettings {
    pub fn from_config(config: &Config) -> Self {
        Self {
            sample_rate: config.outputs.sample_rate,
            buffer_frames: config.outputs.buffer_frames,
            channels: 2,
            tuning: config.tuning.clone(),
            routes: config.outputs.routes.clone(),
            default_backend: config.outputs.backend.clone(),
        }
    }

    fn frames(&self, ms: f64) -> u64 {
        (ms.max(0.0) * f64::from(self.sample_rate) / 1000.0).round() as u64
    }
}

/// What the UI shows for one player, refreshed every tick.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PlayerTelemetry {
    /// Position of the current source in track seconds.
    pub position_secs: Option<f64>,
    pub cue_position_secs: Option<f64>,
    pub peak_l: f32,
    pub peak_r: f32,
    pub underruns: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BusStatus {
    pub key: BusKey,
    pub health: BusHealth,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StartState {
    /// Attached, waiting for a command (preload, or loaded paused).
    Idle,
    /// Start as soon as enough audio is buffered.
    WhenReady {
        fade_in: bool,
    },
    /// A `Start` has been sent for a future frame.
    Requested,
    Started,
}

struct Playing {
    key: SourceKey,
    bus: BusKey,
    slot: usize,
    request: SourceRequest,
    entry: EntryId,
    start_secs: f64,
    shared: Arc<SourceShared>,
    start: StartState,
    /// Report `ReachedEnd` when this source finishes (fade stop).
    report_end: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Plan {
    None,
    Waiting(TransitionPlan),
    Dispatched(TransitionPlan),
}

struct PlayerRuntime {
    worker: PlayerWorker,
    volume: Arc<AtomicF32>,
    cue_volume: Arc<AtomicF32>,
    main: (BusKey, u16),
    cue: Option<(BusKey, u16)>,
    preload: Option<Playing>,
    current: Option<Playing>,
    /// Sources fading out or stopping (crossfades, overlaps, seeks, stops).
    outgoing: Vec<Playing>,
    cue_src: Option<Playing>,
    plan: Plan,
    paused: bool,
    /// Emit `FadeCompleted` once `outgoing` is empty.
    notify_fade: bool,
}

pub struct Engine {
    backends: Vec<Arc<dyn AudioBackend>>,
    settings: EngineSettings,
    opener: SourceOpener,
    buses: BTreeMap<BusKey, Bus>,
    players: HashMap<PlayerId, PlayerRuntime>,
    failures_tx: Sender<WorkerFailure>,
    failures_rx: Receiver<WorkerFailure>,
    owners: HashMap<SourceKey, PlayerId>,
    next_key: u64,
    events: Vec<EngineEvent>,
    dropped_commands: u64,
}

/// Which source of a player a bus slot belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Role {
    Preload,
    Current,
    Outgoing(usize),
    Cue,
}

impl Engine {
    /// `backends` are tried by id for routes; a `Null` backend is always
    /// available as the last resort so a player can never stall.
    pub fn new(
        mut backends: Vec<Arc<dyn AudioBackend>>,
        settings: EngineSettings,
        opener: SourceOpener,
    ) -> Self {
        backends.push(Arc::new(NullBackend));
        let (failures_tx, failures_rx) = crossbeam_channel::unbounded();
        Self {
            backends,
            settings,
            opener,
            buses: BTreeMap::new(),
            players: HashMap::new(),
            failures_tx,
            failures_rx,
            owners: HashMap::new(),
            next_key: 0,
            events: Vec::new(),
            dropped_commands: 0,
        }
    }

    /// Bus commands that could not be queued (should always be zero).
    pub fn dropped_commands(&self) -> u64 {
        self.dropped_commands
    }

    pub fn settings(&self) -> &EngineSettings {
        &self.settings
    }

    pub fn bus_status(&self) -> Vec<BusStatus> {
        self.buses
            .values()
            .map(|b| BusStatus {
                key: b.key().clone(),
                health: b.health(),
                error: b.last_error().map(str::to_owned),
            })
            .collect()
    }

    pub fn telemetry(&self, player: PlayerId) -> PlayerTelemetry {
        let Some(rt) = self.players.get(&player) else {
            return PlayerTelemetry::default();
        };
        let rate = f64::from(self.settings.sample_rate);
        let position = |p: &Playing| p.start_secs + p.shared.frames_played() as f64 / rate;
        let (peak_l, peak_r) = rt
            .current
            .iter()
            .chain(rt.outgoing.iter())
            .fold((0.0f32, 0.0f32), |(l, r), p| {
                (l.max(p.shared.peak_l.take()), r.max(p.shared.peak_r.take()))
            });
        PlayerTelemetry {
            position_secs: rt.current.as_ref().map(position),
            cue_position_secs: rt.cue_src.as_ref().map(position),
            peak_l,
            peak_r,
            underruns: rt.current.as_ref().map_or(0, |p| {
                p.shared
                    .underruns
                    .load(std::sync::atomic::Ordering::Relaxed)
            }),
        }
    }

    fn backend(&self, id: Option<&str>) -> Arc<dyn AudioBackend> {
        let wanted = id.or(self.settings.default_backend.as_deref());
        wanted
            .and_then(|w| self.backends.iter().find(|b| b.id().0 == w))
            .or_else(|| self.backends.first())
            .cloned()
            .unwrap_or_else(|| Arc::new(NullBackend))
    }

    /// Resolves the (bus, first channel) of a player's Main and Cue outputs.
    fn resolve_routes(&self, player: PlayerId) -> ((BusKey, u16), Option<(BusKey, u16)>) {
        let routes = self.settings.routes.iter().find(|r| r.player == player);
        let key_for = |backend: &Arc<dyn AudioBackend>, device: String| BusKey {
            backend: backend.id().0,
            device,
        };
        let main = match routes.and_then(|r| r.main.as_ref()) {
            Some(route) => (
                key_for(&self.backend(Some(&route.backend)), route.device.clone()),
                route.first_channel,
            ),
            None => {
                let backend = self.backend(None);
                let device = backend
                    .default_device()
                    .map_or_else(|| "null".to_owned(), |d| d.0);
                let backend = if device == "null" {
                    Arc::new(NullBackend) as Arc<dyn AudioBackend>
                } else {
                    backend
                };
                (key_for(&backend, device), 0)
            }
        };
        let cue = routes.and_then(|r| r.cue.as_ref()).map(|route| {
            (
                key_for(&self.backend(Some(&route.backend)), route.device.clone()),
                route.first_channel,
            )
        });
        (main, cue)
    }

    fn ensure_bus(&mut self, key: &BusKey, now: Instant) {
        if !self.buses.contains_key(key) {
            let backend = self.backend(Some(&key.backend));
            let t = &self.settings.tuning;
            let config = StreamConfig {
                sample_rate: self.settings.sample_rate,
                buffer_frames: self.settings.buffer_frames,
                channels: self.settings.channels,
            };
            let mixer = MixerConfig {
                volume_smoothing_frames: self.settings.frames(t.gain_smoothing_ms).max(1) as u32,
                max_commands_per_block: t.max_commands_per_block,
            };
            let timing = BusTiming {
                watchdog_timeout: Duration::from_secs_f64(t.watchdog_timeout_ms / 1000.0),
                reconnect_interval: Duration::from_secs_f64(t.reconnect_interval_ms / 1000.0),
            };
            let bus = Bus::open(key.clone(), backend, config, 8, mixer, timing, now);
            self.buses.insert(key.clone(), bus);
        }
        // Capacity derived from routing (spec §4.3): per player on Main, a
        // current, a preload and up to three outgoing; per Cue, two.
        let mains = self.players.values().filter(|p| &p.main.0 == key).count();
        let cues = self
            .players
            .values()
            .filter(|p| p.cue.as_ref().is_some_and(|c| &c.0 == key))
            .count();
        let wanted = ((mains * 5 + cues * 2 + 2) as f64 * self.settings.tuning.mixer_headroom)
            .ceil() as usize;
        if let Some(bus) = self.buses.get_mut(key) {
            bus.ensure_capacity(wanted.max(8));
        }
    }

    /// Executes one action at time `now`.
    pub fn execute(&mut self, action: EngineAction, now: Instant) {
        match action {
            EngineAction::AddPlayer { player } => self.add_player(player, now),
            EngineAction::RemovePlayer { player } => self.remove_player(player),
            EngineAction::Preload { player, request } => self.preload(player, request),
            EngineAction::StartCurrent { player, request } => {
                self.start_current(player, &request, None)
            }
            EngineAction::Crossfade {
                player,
                request,
                fade_ms,
            } => {
                self.start_current(player, &request, Some(fade_ms));
            }
            EngineAction::FadeOutAndStop { player, fade_ms } => {
                self.fade_out_and_stop(player, fade_ms)
            }
            EngineAction::Pause { player } => self.pause(player),
            EngineAction::Resume { player } => self.resume(player),
            EngineAction::StopNow { player } => self.stop_now(player),
            EngineAction::Schedule { player, plan } => {
                self.undispatch(player);
                if let Some(rt) = self.players.get_mut(&player) {
                    rt.plan = plan.map_or(Plan::None, Plan::Waiting);
                }
            }
            EngineAction::Seek { player, secs } => self.seek(player, secs),
            EngineAction::SetVolume { player, volume } => {
                if let Some(rt) = self.players.get(&player) {
                    rt.volume.store(volume);
                }
            }
            EngineAction::StartCue { player, request } => self.start_cue(player, &request),
            EngineAction::StopCue { player } => {
                if let Some(cue) = self
                    .players
                    .get_mut(&player)
                    .and_then(|rt| rt.cue_src.take())
                {
                    self.stop_quick_and_release(cue);
                }
            }
            EngineAction::LoadPaused { player, request } => {
                if let Some(p) = self.new_source(player, false, &request) {
                    let old = self.players.get_mut(&player).and_then(|rt| {
                        rt.paused = true;
                        rt.current.replace(p)
                    });
                    if let Some(old) = old {
                        self.release(old);
                    }
                }
            }
        }
    }

    fn add_player(&mut self, player: PlayerId, now: Instant) {
        if self.players.contains_key(&player) {
            return;
        }
        let (main, cue) = self.resolve_routes(player);
        let t = &self.settings.tuning;
        let ready = self.settings.frames(t.ready_threshold_ms) as usize;
        let worker = match PlayerWorker::spawn(
            &format!("fp-player-{}", player.0),
            self.opener.clone(),
            self.settings.sample_rate,
            ready,
            self.failures_tx.clone(),
        ) {
            Ok(w) => w,
            Err(e) => {
                tracing::error!(?player, "cannot spawn the player worker: {e}");
                return;
            }
        };
        self.players.insert(
            player,
            PlayerRuntime {
                worker,
                volume: Arc::new(AtomicF32::new(1.0)),
                cue_volume: Arc::new(AtomicF32::new(1.0)),
                main: main.clone(),
                cue: cue.clone(),
                preload: None,
                current: None,
                outgoing: Vec::new(),
                cue_src: None,
                plan: Plan::None,
                paused: false,
                notify_fade: false,
            },
        );
        self.ensure_bus(&main.0, now);
        if let Some((key, _)) = cue {
            self.ensure_bus(&key, now);
        }
    }

    fn remove_player(&mut self, player: PlayerId) {
        if let Some(mut rt) = self.players.remove(&player) {
            let all: Vec<Playing> = rt
                .preload
                .take()
                .into_iter()
                .chain(rt.current.take())
                .chain(rt.cue_src.take())
                .chain(rt.outgoing.drain(..))
                .collect();
            for p in all {
                self.detach(&p);
                self.owners.remove(&p.key);
            }
        }
    }

    /// Attaches a new source to the player's Main (or Cue) bus and asks its
    /// worker to fill it.
    fn new_source(
        &mut self,
        player: PlayerId,
        cue: bool,
        request: &SourceRequest,
    ) -> Option<Playing> {
        let rt = self.players.get(&player)?;
        let (bus_key, channel) = if cue {
            rt.cue.clone()?
        } else {
            rt.main.clone()
        };
        let volume = if cue {
            rt.cue_volume.clone()
        } else {
            rt.volume.clone()
        };
        let ring = ((self.settings.tuning.prebuffer_secs * f64::from(self.settings.sample_rate))
            as usize)
            .max(1024);
        let bus = self.buses.get_mut(&bus_key)?;
        let Some(slot) = bus.alloc_slot() else {
            tracing::error!(?player, bus = ?bus_key, "no free mixer slot");
            return None;
        };
        let (producer, consumer) = source_pair(ring);
        let shared = consumer.shared.clone();
        if !bus.send(BusCommand::Attach {
            slot,
            source: consumer,
            volume,
            first_channel: channel,
        }) {
            tracing::error!(?player, "bus command queue full");
            return None;
        }
        self.next_key += 1;
        let key = SourceKey(self.next_key);
        rt.worker
            .load(key, request.path.clone(), request.from_secs, producer);
        self.owners.insert(key, player);
        Some(Playing {
            key,
            bus: bus_key,
            slot,
            request: request.clone(),
            entry: request.entry,
            start_secs: request.from_secs,
            shared,
            start: StartState::Idle,
            report_end: false,
        })
    }

    fn send(&mut self, bus: &BusKey, command: BusCommand) {
        if let Some(b) = self.buses.get_mut(bus)
            && !b.send(command)
        {
            self.dropped_commands += 1;
            tracing::error!(?bus, "bus command queue full; command dropped");
        }
    }

    fn now_frame(&self, bus: &BusKey) -> u64 {
        self.buses.get(bus).map_or(0, Bus::now_frame)
    }

    fn detach(&mut self, p: &Playing) {
        self.send(&p.bus, BusCommand::Detach { slot: p.slot });
        if let Some(owner) = self.owners.get(&p.key)
            && let Some(rt) = self.players.get(owner)
        {
            rt.worker.drop_source(p.key);
        }
    }

    fn release(&mut self, p: Playing) {
        self.detach(&p);
        self.owners.remove(&p.key);
    }

    /// Fades a source out over `frames` from now and stops it; it is released
    /// when the mixer reports it finished.
    fn fade_out(&mut self, player: PlayerId, p: Playing, frames: u64, curve: Curve) {
        if p.start != StartState::Started {
            self.release(p);
            return;
        }
        let now = self.now_frame(&p.bus);
        let len = u32::try_from(frames).unwrap_or(u32::MAX);
        self.send(&p.bus, BusCommand::Cancel { slot: p.slot });
        self.send(
            &p.bus,
            BusCommand::Ramp {
                slot: p.slot,
                to: 0.0,
                frames: len,
                curve,
                at_frame: now,
            },
        );
        self.send(
            &p.bus,
            BusCommand::StopAt {
                slot: p.slot,
                at_frame: now + frames,
            },
        );
        if let Some(rt) = self.players.get_mut(&player) {
            rt.outgoing.push(p);
        }
    }

    fn stop_quick_and_release(&mut self, p: Playing) {
        if p.start != StartState::Started {
            // Never reached the mixer's clock: there is nothing to ramp down.
            self.send(&p.bus, BusCommand::Cancel { slot: p.slot });
            self.release(p);
            return;
        }
        // A de-click ramp then detach; nothing is reported for it.
        let frames = self.settings.frames(self.settings.tuning.declick_ms);
        let now = self.now_frame(&p.bus);
        let len = u32::try_from(frames).unwrap_or(u32::MAX);
        self.send(&p.bus, BusCommand::Cancel { slot: p.slot });
        self.send(
            &p.bus,
            BusCommand::Ramp {
                slot: p.slot,
                to: 0.0,
                frames: len,
                curve: Curve::Linear,
                at_frame: now,
            },
        );
        self.send(
            &p.bus,
            BusCommand::StopAt {
                slot: p.slot,
                at_frame: now + frames,
            },
        );
        // Keep it until it finishes so the ramp is heard; then it is released.
        let owner = self.owners.get(&p.key).copied();
        match owner.and_then(|o| self.players.get_mut(&o)) {
            Some(rt) => rt.outgoing.push(p),
            None => self.release(p),
        }
    }

    /// Takes back a transition that was sent to the mixer but has not happened.
    fn undispatch(&mut self, player: PlayerId) {
        let Some(rt) = self.players.get_mut(&player) else {
            return;
        };
        let Plan::Dispatched(plan) = rt.plan else {
            return;
        };
        rt.plan = Plan::Waiting(plan);
        let mut cancels = Vec::new();
        if let Some(c) = rt.current.as_ref() {
            cancels.push((c.bus.clone(), c.slot));
        }
        if let Some(p) = rt.preload.as_mut()
            && p.start == StartState::Requested
        {
            p.start = StartState::Idle;
            cancels.push((p.bus.clone(), p.slot));
        }
        for (bus, slot) in cancels {
            self.send(&bus, BusCommand::Cancel { slot });
        }
    }

    fn preload(&mut self, player: PlayerId, request: Option<SourceRequest>) {
        self.undispatch(player);
        let Some(rt) = self.players.get_mut(&player) else {
            return;
        };
        if let (Some(old), Some(req)) = (rt.preload.as_ref(), request.as_ref())
            && old.entry == req.entry
            && old.start_secs == req.from_secs
        {
            return;
        }
        if let Some(old) = rt.preload.take() {
            self.release(old);
        }
        if let Some(req) = request {
            let p = self.new_source(player, false, &req);
            if let Some(rt) = self.players.get_mut(&player) {
                rt.preload = p;
            }
        }
    }

    /// Takes the preload if it matches `request`, otherwise opens a new source.
    fn take_or_open(&mut self, player: PlayerId, request: &SourceRequest) -> Option<Playing> {
        let rt = self.players.get_mut(&player)?;
        let matches = rt
            .preload
            .as_ref()
            .is_some_and(|p| p.entry == request.entry && p.start_secs == request.from_secs);
        if matches {
            return rt.preload.take();
        }
        self.new_source(player, false, request)
    }

    fn start_current(
        &mut self,
        player: PlayerId,
        request: &SourceRequest,
        crossfade_ms: Option<u32>,
    ) {
        self.undispatch(player);
        let Some(mut next) = self.take_or_open(player, request) else {
            self.events.push(EngineEvent::SourceFailed {
                player,
                entry: request.entry,
            });
            return;
        };
        if next.start == StartState::Requested {
            self.send(&next.bus, BusCommand::Cancel { slot: next.slot });
        }
        next.start = StartState::WhenReady { fade_in: false };
        let Some(rt) = self.players.get_mut(&player) else {
            return;
        };
        rt.paused = false;
        rt.plan = Plan::None;
        let old = rt.current.replace(next);
        if let Some(old) = old {
            match crossfade_ms {
                Some(ms) => {
                    let frames = self.settings.frames(f64::from(ms));
                    if let Some(rt) = self.players.get_mut(&player) {
                        rt.notify_fade = true;
                    }
                    self.fade_out(player, old, frames, Curve::EqualPower);
                }
                None => self.stop_quick_and_release(old),
            }
        } else if crossfade_ms.is_some() {
            // Nothing to fade: complete immediately.
            self.events.push(EngineEvent::FadeCompleted { player });
        }
    }

    fn fade_out_and_stop(&mut self, player: PlayerId, fade_ms: u32) {
        self.undispatch(player);
        let frames = self.settings.frames(f64::from(fade_ms));
        let Some(rt) = self.players.get_mut(&player) else {
            return;
        };
        rt.plan = Plan::None;
        let current = rt.current.take();
        let outgoing: Vec<Playing> = std::mem::take(&mut rt.outgoing);
        for p in outgoing {
            self.fade_out(player, p, frames, Curve::EqualPower);
        }
        match current {
            Some(mut p) if p.start == StartState::Started => {
                p.report_end = true;
                self.fade_out(player, p, frames, Curve::EqualPower);
            }
            Some(p) => {
                let entry = p.entry;
                self.release(p);
                self.events.push(EngineEvent::ReachedEnd { player, entry });
            }
            None => {}
        }
    }

    fn pause(&mut self, player: PlayerId) {
        self.undispatch(player);
        let ramp = u32::try_from(self.settings.frames(self.settings.tuning.pause_ramp_ms))
            .unwrap_or(u32::MAX);
        let Some(rt) = self.players.get_mut(&player) else {
            return;
        };
        rt.paused = true;
        let targets: Vec<(BusKey, usize)> = rt
            .current
            .iter()
            .chain(rt.outgoing.iter())
            .map(|p| (p.bus.clone(), p.slot))
            .collect();
        for (bus, slot) in targets {
            self.send(
                &bus,
                BusCommand::Pause {
                    slot,
                    ramp_frames: ramp,
                },
            );
        }
    }

    fn resume(&mut self, player: PlayerId) {
        let ramp = u32::try_from(self.settings.frames(self.settings.tuning.pause_ramp_ms))
            .unwrap_or(u32::MAX);
        let Some(rt) = self.players.get_mut(&player) else {
            return;
        };
        rt.paused = false;
        let mut targets: Vec<(BusKey, usize)> = rt
            .outgoing
            .iter()
            .map(|p| (p.bus.clone(), p.slot))
            .collect();
        if let Some(c) = rt.current.as_mut() {
            if c.start == StartState::Idle {
                c.start = StartState::WhenReady { fade_in: true };
            } else {
                targets.push((c.bus.clone(), c.slot));
            }
        }
        for (bus, slot) in targets {
            self.send(
                &bus,
                BusCommand::Resume {
                    slot,
                    ramp_frames: ramp,
                },
            );
        }
    }

    fn stop_now(&mut self, player: PlayerId) {
        self.undispatch(player);
        let Some(rt) = self.players.get_mut(&player) else {
            return;
        };
        rt.plan = Plan::None;
        rt.paused = false;
        rt.notify_fade = false;
        let all: Vec<Playing> = rt
            .current
            .take()
            .into_iter()
            .chain(rt.outgoing.drain(..))
            .collect();
        for p in all {
            self.stop_quick_and_release(p);
        }
    }

    fn seek(&mut self, player: PlayerId, secs: f64) {
        self.undispatch(player);
        let Some(mut request) = self
            .players
            .get(&player)
            .and_then(|rt| rt.current.as_ref())
            .map(|c| c.request.clone())
        else {
            return;
        };
        request.from_secs = secs;
        let Some(mut next) = self.new_source(player, false, &request) else {
            return;
        };
        let Some(rt) = self.players.get_mut(&player) else {
            return;
        };
        next.start = if rt.paused {
            StartState::Idle
        } else {
            StartState::WhenReady { fade_in: true }
        };
        if let Some(old) = rt.current.replace(next) {
            self.stop_quick_and_release(old);
        }
    }

    fn start_cue(&mut self, player: PlayerId, request: &SourceRequest) {
        if let Some(old) = self
            .players
            .get_mut(&player)
            .and_then(|rt| rt.cue_src.take())
        {
            self.stop_quick_and_release(old);
        }
        match self.new_source(player, true, request) {
            Some(mut cue) => {
                cue.start = StartState::WhenReady { fade_in: false };
                if let Some(rt) = self.players.get_mut(&player) {
                    rt.cue_src = Some(cue);
                }
            }
            // No Cue output configured: nothing can be heard, end it at once.
            None => self.events.push(EngineEvent::CueEnded { player }),
        }
    }
}

impl Engine {
    /// Advances the engine: supervises devices, turns bus and worker
    /// observations into model events, starts sources that became ready and
    /// dispatches transitions that are due. Returns the events for the model.
    pub fn tick(&mut self, now: Instant) -> Vec<EngineEvent> {
        self.handle_failures();
        let keys: Vec<BusKey> = self.buses.keys().cloned().collect();
        for key in keys {
            let events = match self.buses.get_mut(&key) {
                Some(bus) => {
                    bus.supervise(now);
                    bus.poll()
                }
                None => Vec::new(),
            };
            for event in events {
                self.handle_bus_event(&key, event);
            }
        }
        self.start_ready_sources();
        self.dispatch_plans();
        std::mem::take(&mut self.events)
    }

    fn handle_failures(&mut self) {
        let failures: Vec<WorkerFailure> = self.failures_rx.try_iter().collect();
        for failure in failures {
            let Some(player) = self.owners.get(&failure.key).copied() else {
                continue;
            };
            tracing::warn!(?player, error = %failure.error, "source failed");
            let Some(rt) = self.players.get_mut(&player) else {
                continue;
            };
            let taken = if rt.current.as_ref().is_some_and(|p| p.key == failure.key) {
                rt.plan = Plan::None;
                rt.current.take()
            } else if rt.preload.as_ref().is_some_and(|p| p.key == failure.key) {
                rt.preload.take()
            } else if rt.cue_src.as_ref().is_some_and(|p| p.key == failure.key) {
                rt.cue_src.take()
            } else {
                None
            };
            if let Some(p) = taken {
                let entry = p.entry;
                self.release(p);
                self.events
                    .push(EngineEvent::SourceFailed { player, entry });
            }
        }
    }

    fn find(&self, bus: &BusKey, slot: usize) -> Option<(PlayerId, Role)> {
        self.players.iter().find_map(|(id, rt)| {
            let is = |p: &Playing| &p.bus == bus && p.slot == slot;
            if rt.current.as_ref().is_some_and(is) {
                Some((*id, Role::Current))
            } else if rt.preload.as_ref().is_some_and(is) {
                Some((*id, Role::Preload))
            } else if rt.cue_src.as_ref().is_some_and(is) {
                Some((*id, Role::Cue))
            } else {
                rt.outgoing
                    .iter()
                    .position(is)
                    .map(|i| (*id, Role::Outgoing(i)))
            }
        })
    }

    fn handle_bus_event(&mut self, bus: &BusKey, event: BusEvent) {
        match event {
            BusEvent::Started { slot, .. } => {
                let Some((player, role)) = self.find(bus, slot) else {
                    return;
                };
                let Some(rt) = self.players.get_mut(&player) else {
                    return;
                };
                match role {
                    Role::Preload => {
                        // A dispatched transition happened: the next is now on air.
                        let Some(mut next) = rt.preload.take() else {
                            return;
                        };
                        next.start = StartState::Started;
                        let entry = next.entry;
                        let overlapping = matches!(
                            rt.plan,
                            Plan::Dispatched(TransitionPlan::StartNextAt {
                                fade_current_until_secs: Some(_),
                                ..
                            })
                        );
                        rt.plan = Plan::None;
                        // The audio moved on: whatever was requested meanwhile, the
                        // next is playing, and the model follows the engine.
                        rt.paused = false;
                        if let Some(old) = rt.current.replace(next) {
                            rt.outgoing.push(old);
                        }
                        rt.notify_fade |= overlapping;
                        self.events
                            .push(EngineEvent::TransitionStarted { player, entry });
                    }
                    Role::Current => {
                        if let Some(c) = rt.current.as_mut() {
                            c.start = StartState::Started;
                        }
                    }
                    Role::Cue => {
                        if let Some(c) = rt.cue_src.as_mut() {
                            c.start = StartState::Started;
                        }
                    }
                    Role::Outgoing(i) => {
                        if let Some(o) = rt.outgoing.get_mut(i) {
                            o.start = StartState::Started;
                        }
                    }
                }
            }
            BusEvent::Finished { slot, .. } => {
                let Some((player, role)) = self.find(bus, slot) else {
                    return;
                };
                match role {
                    Role::Current => self.current_finished(player),
                    Role::Outgoing(i) => {
                        let Some(rt) = self.players.get_mut(&player) else {
                            return;
                        };
                        if i < rt.outgoing.len() {
                            let p = rt.outgoing.remove(i);
                            let report = p.report_end.then_some(p.entry);
                            let done = rt.outgoing.is_empty() && rt.notify_fade;
                            if done {
                                rt.notify_fade = false;
                            }
                            self.release(p);
                            if let Some(entry) = report {
                                self.events.push(EngineEvent::ReachedEnd { player, entry });
                            } else if done {
                                self.events.push(EngineEvent::FadeCompleted { player });
                            }
                        }
                    }
                    Role::Cue => {
                        if let Some(p) = self
                            .players
                            .get_mut(&player)
                            .and_then(|rt| rt.cue_src.take())
                        {
                            self.release(p);
                            self.events.push(EngineEvent::CueEnded { player });
                        }
                    }
                    Role::Preload => {
                        if let Some(p) = self
                            .players
                            .get_mut(&player)
                            .and_then(|rt| rt.preload.take())
                        {
                            self.release(p);
                        }
                    }
                }
            }
        }
    }

    /// The current source stopped by itself (end of stream or a planned stop).
    fn current_finished(&mut self, player: PlayerId) {
        let Some(rt) = self.players.get_mut(&player) else {
            return;
        };
        let Some(finished) = rt.current.take() else {
            return;
        };
        let plan = match rt.plan {
            Plan::Waiting(p) | Plan::Dispatched(p) => Some(p),
            Plan::None => None,
        };
        let entry = finished.entry;
        self.release(finished);
        match plan {
            // End of stream before (or instead of) the scheduled transition:
            // start the next right now.
            Some(TransitionPlan::StartNextAt { .. }) => {
                let Some(rt) = self.players.get_mut(&player) else {
                    return;
                };
                rt.plan = Plan::Dispatched(TransitionPlan::StartNextAt {
                    at_secs: 0.0,
                    fade_current_until_secs: None,
                });
                match rt.preload.as_mut() {
                    Some(next) if next.start != StartState::Started => {
                        next.start = StartState::Requested;
                        let (bus, slot) = (next.bus.clone(), next.slot);
                        let at = self.now_frame(&bus);
                        self.send(&bus, BusCommand::Start { slot, at_frame: at });
                    }
                    _ => {
                        rt.plan = Plan::None;
                        self.events.push(EngineEvent::ReachedEnd { player, entry });
                    }
                }
            }
            _ => {
                if let Some(rt) = self.players.get_mut(&player) {
                    rt.plan = Plan::None;
                }
                self.events.push(EngineEvent::ReachedEnd { player, entry });
            }
        }
    }

    fn start_ready_sources(&mut self) {
        let declick = u32::try_from(self.settings.frames(self.settings.tuning.declick_ms))
            .unwrap_or(u32::MAX);
        let mut starts: Vec<(BusKey, usize, bool)> = Vec::new();
        for rt in self.players.values_mut() {
            let current = if rt.paused { None } else { rt.current.as_mut() };
            for p in current.into_iter().chain(rt.cue_src.iter_mut()) {
                if let StartState::WhenReady { fade_in } = p.start
                    && p.shared.is_ready()
                    && !p.shared.is_failed()
                {
                    p.start = StartState::Requested;
                    starts.push((p.bus.clone(), p.slot, fade_in));
                }
            }
        }
        for (bus, slot, fade_in) in starts {
            let now = self.now_frame(&bus);
            if fade_in {
                self.send(
                    &bus,
                    BusCommand::Ramp {
                        slot,
                        to: 0.0,
                        frames: 0,
                        curve: Curve::Linear,
                        at_frame: 0,
                    },
                );
            }
            self.send(
                &bus,
                BusCommand::Start {
                    slot,
                    at_frame: now,
                },
            );
            if fade_in {
                self.send(
                    &bus,
                    BusCommand::Ramp {
                        slot,
                        to: 1.0,
                        frames: declick,
                        curve: Curve::Linear,
                        at_frame: now,
                    },
                );
            }
        }
    }

    /// Sends due transitions to the mixer with their exact frame.
    fn dispatch_plans(&mut self) {
        let rate = f64::from(self.settings.sample_rate);
        let lead = self.settings.frames(self.settings.tuning.schedule_lead_ms);
        let declick = self.settings.frames(self.settings.tuning.declick_ms);
        let mut due: Vec<(PlayerId, TransitionPlan, u64)> = Vec::new();
        for (id, rt) in &self.players {
            let Plan::Waiting(plan) = rt.plan else {
                continue;
            };
            let Some(current) = rt.current.as_ref() else {
                continue;
            };
            if rt.paused || current.start != StartState::Started {
                continue;
            }
            let target = match plan {
                TransitionPlan::StopAt { at_secs }
                | TransitionPlan::StartNextAt { at_secs, .. } => at_secs,
            };
            if !target.is_finite() {
                continue; // `SOURCE_END`: handled when the source finishes.
            }
            let Some(bus) = self.buses.get(&current.bus) else {
                continue;
            };
            let (now, played) = bus
                .shared()
                .consistent(|| (bus.now_frame(), current.shared.frames_played()));
            let position = current.start_secs + played as f64 / rate;
            let until = ((target - position) * rate).round().max(0.0) as u64;
            if until <= lead {
                due.push((*id, plan, now + until));
            }
        }
        for (player, plan, at_frame) in due {
            self.dispatch(player, plan, at_frame, declick);
        }
    }

    fn dispatch(&mut self, player: PlayerId, plan: TransitionPlan, at_frame: u64, declick: u64) {
        let rate = f64::from(self.settings.sample_rate);
        let Some(rt) = self.players.get_mut(&player) else {
            return;
        };
        rt.plan = Plan::Dispatched(plan);
        let Some(current) = rt.current.as_ref() else {
            return;
        };
        let (cur_bus, cur_slot) = (current.bus.clone(), current.slot);
        let next = match plan {
            TransitionPlan::StartNextAt { .. } => rt.preload.as_mut().map(|p| {
                p.start = StartState::Requested;
                (p.bus.clone(), p.slot)
            }),
            TransitionPlan::StopAt { .. } => None,
        };
        let declick32 = u32::try_from(declick).unwrap_or(u32::MAX);
        match (plan, next) {
            (
                TransitionPlan::StartNextAt {
                    at_secs,
                    fade_current_until_secs,
                },
                Some((bus, slot)),
            ) => {
                self.send(&bus, BusCommand::Start { slot, at_frame });
                match fade_current_until_secs {
                    Some(until) => {
                        let len = (((until - at_secs) * rate).round() as u64).max(declick);
                        let len32 = u32::try_from(len).unwrap_or(u32::MAX);
                        self.send(
                            &cur_bus,
                            BusCommand::Ramp {
                                slot: cur_slot,
                                to: 0.0,
                                frames: len32,
                                curve: Curve::EqualPower,
                                at_frame,
                            },
                        );
                        self.send(
                            &cur_bus,
                            BusCommand::StopAt {
                                slot: cur_slot,
                                at_frame: at_frame + len,
                            },
                        );
                    }
                    None => {
                        let fade_at = at_frame.saturating_sub(declick);
                        self.send(
                            &cur_bus,
                            BusCommand::Ramp {
                                slot: cur_slot,
                                to: 0.0,
                                frames: declick32,
                                curve: Curve::Linear,
                                at_frame: fade_at,
                            },
                        );
                        self.send(
                            &cur_bus,
                            BusCommand::StopAt {
                                slot: cur_slot,
                                at_frame,
                            },
                        );
                    }
                }
            }
            // A stop, or a start with nothing preloaded: end the current track there.
            _ => {
                let fade_at = at_frame.saturating_sub(declick);
                self.send(
                    &cur_bus,
                    BusCommand::Ramp {
                        slot: cur_slot,
                        to: 0.0,
                        frames: declick32,
                        curve: Curve::Linear,
                        at_frame: fade_at,
                    },
                );
                self.send(
                    &cur_bus,
                    BusCommand::StopAt {
                        slot: cur_slot,
                        at_frame,
                    },
                );
            }
        }
    }
}

impl Engine {
    /// One-line description of a player's engine state, for logs and test failures.
    pub fn describe_player(&self, player: PlayerId) -> String {
        let Some(rt) = self.players.get(&player) else {
            return "none".into();
        };
        let d = |p: &Playing| {
            format!(
                "key{} slot{} start {:?} ready {} failed {} eof {} played {} under {}",
                p.key.0,
                p.slot,
                p.start,
                p.shared.is_ready(),
                p.shared.is_failed(),
                p.shared.is_eof(),
                p.shared.frames_played(),
                p.shared
                    .underruns
                    .load(std::sync::atomic::Ordering::Relaxed)
            )
        };
        format!(
            "paused {} plan {:?} cur [{}] pre [{}] outgoing {} cap {}",
            rt.paused,
            rt.plan,
            rt.current.as_ref().map(d).unwrap_or_default(),
            rt.preload.as_ref().map(d).unwrap_or_default(),
            rt.outgoing.len(),
            self.buses.values().map(|b| b.capacity()).sum::<usize>()
        )
    }
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `for i in 1 2 3; do cargo test -p fp-engine --test engine; done`
Expected: 17 tests PASS on every run. Running three times guards against timing flakiness with the worker threads.

- [ ] **Step: Check and commit**

Run: `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
Expected: everything passes, no warnings.
```bash
git add crates/fp-engine
git commit -m "feat(engine): add engine executing model actions with sample accuracy

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 9: The conductor thread, end-to-end and stress tests

**Files:**
- Create: `crates/fp-engine/src/conductor.rs`
- Modify: `crates/fp-engine/src/lib.rs`
- Test: `crates/fp-engine/tests/conductor.rs`

**Interfaces:**
- Consumes:
  - from `fp_model`: `AppState`, `Command`, `EngineAction`, `ModelError`, `apply`, `on_event`;
  - from Task 8: `Engine`, `PlayerTelemetry`, `BusStatus`.
- Produces (spec §2.2–2.3, §4.4):
  - `Telemetry { players: Vec<(PlayerId, PlayerTelemetry)>, buses: Vec<BusStatus>, model_version: u64, dropped_commands: u64 }`.
  - **`Conductor`:**
    - `Conductor::new(state, initial_actions, engine, now) -> (Conductor, ConductorHandle)`, which executes `AddPlayer` for every player and then the initial actions (e.g. restore's `LoadPaused`);
    - `tick(now)`, `state()`, `engine()`;
    - `spawn(self, handle, period) -> io::Result<ConductorHandle>`, which runs its own thread.
  - **`ConductorHandle`:**
    - `send(Command) -> bool`: a bounded queue of 1024 that never blocks;
    - `model: Arc<ArcSwap<AppState>>`, replaced on every change;
    - `telemetry: Arc<ArcSwap<Telemetry>>`, replaced every tick;
    - `rejected: Receiver<ModelError>`;
    - dropping it stops and joins the thread.
- `model_version` increases whenever the model snapshot changes. Plan 4 uses it to trigger debounced saves.

- [ ] **Step 1: Write the failing tests**

`crates/fp-engine/tests/conductor.rs`:
```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The conductor wires the model rules to the engine end to end.

mod support;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use fp_backends::{AudioBackend, NullBackend, OfflineBackend, OfflineDevice};
use fp_engine::conductor::{Conductor, ConductorHandle};
use fp_engine::engine::{Engine, EngineSettings};
use fp_model::{AppState, Command, Config, EntryId, ModelError, PlayMode, PlayerId, Transport};
use support::tagged_opener;

const BLOCK: usize = 480;
const TRACK_FRAMES: u64 = 48_000;

fn model(players: usize, tracks: u64) -> AppState {
    let mut config = Config::default();
    config.players.count = players;
    config.outputs.backend = Some("offline".into());
    config.outputs.buffer_frames = BLOCK as u32;
    let mut state = AppState::new(config, "Main");
    let playlist = state.playlists.first_id().unwrap();
    let paths = (1..=tracks)
        .map(|n| PathBuf::from(format!("track{n}")))
        .collect();
    fp_model::apply(
        &mut state,
        Command::InsertPaths {
            playlist,
            index: 0,
            paths,
        },
    )
    .unwrap();
    for t in state.library.iter_mut() {
        t.duration_secs = TRACK_FRAMES as f64 / 48_000.0;
    }
    state
}

fn offline_conductor(state: AppState) -> (Conductor, ConductorHandle, OfflineDevice, Instant) {
    let backend = OfflineBackend::new();
    let device = backend.add_device("main", 2);
    let settings = EngineSettings::from_config(&state.config);
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
    let engine = Engine::new(backends, settings, tagged_opener(TRACK_FRAMES));
    let now = Instant::now();
    let (conductor, handle) = Conductor::new(state, Vec::new(), engine, now);
    (conductor, handle, device, now)
}

fn entries(state: &AppState) -> Vec<EntryId> {
    let p = state.playlists.first_id().unwrap();
    state
        .playlists
        .get(p)
        .unwrap()
        .entries
        .iter()
        .map(|e| e.id)
        .collect()
}

#[test]
fn continuous_playback_chains_tracks_in_the_model_and_on_air() {
    let (mut conductor, handle, device, mut now) = offline_conductor(model(1, 3));
    let e = entries(conductor.state());
    let p = conductor.state().players[0].id;
    assert!(handle.send(Command::Play(p)));
    conductor.tick(now);
    std::thread::sleep(Duration::from_millis(30));
    let mut heard = Vec::new();
    for _ in 0..150 {
        conductor.tick(now);
        heard.extend(device.render(BLOCK).unwrap().chunks(2).map(|f| f[0]));
        now += Duration::from_millis(10);
        std::thread::yield_now();
    }
    let model = handle.model.load();
    let player = model.player(p).unwrap();
    assert_eq!(
        player.current,
        Some(e[1]),
        "the second track is on air after the first ended"
    );
    assert!(model.playlists.entry(e[0]).unwrap().played);
    assert!(heard.iter().any(|v| (*v as u64) / 100_000 == 2));
    assert!(handle.telemetry.load().model_version > 0);
}

#[test]
fn refused_commands_are_reported_to_the_ui() {
    let (mut conductor, handle, _device, now) = offline_conductor(model(1, 2));
    let p = conductor.state().players[0].id;
    handle.send(Command::SetMode(p, PlayMode::Single));
    handle.send(Command::ToggleStopAfterCurrent(p));
    conductor.tick(now);
    assert_eq!(
        handle.rejected.try_recv(),
        Ok(ModelError::StopAfterInSingle)
    );
}

#[test]
fn the_spawned_conductor_runs_in_real_time_on_the_null_backend() {
    let mut state = model(1, 2);
    state.config.outputs.backend = Some("null".into());
    let settings = EngineSettings::from_config(&state.config);
    let engine = Engine::new(
        vec![Arc::new(NullBackend)],
        settings,
        tagged_opener(TRACK_FRAMES),
    );
    let (conductor, handle) = Conductor::new(state, Vec::new(), engine, Instant::now());
    let p = conductor.state().players[0].id;
    let handle = conductor.spawn(handle, Duration::from_millis(5)).unwrap();
    handle.send(Command::Play(p));
    std::thread::sleep(Duration::from_millis(300));
    let t = handle.telemetry.load();
    let position = t
        .players
        .iter()
        .find(|(id, _)| *id == p)
        .unwrap()
        .1
        .position_secs
        .unwrap();
    assert!(position > 0.1, "position {position}");
    drop(handle);
}

#[test]
fn a_flooded_command_queue_refuses_instead_of_blocking() {
    let (conductor, handle, _device, _now) = offline_conductor(model(1, 2));
    let p = conductor.state().players[0].id;
    let started = Instant::now();
    let accepted = (0..5_000)
        .filter(|_| handle.send(Command::SetVolume(p, 0.5)))
        .count();
    assert!(accepted < 5_000, "the queue is bounded");
    assert!(
        started.elapsed() < Duration::from_secs(1),
        "sending never blocks the UI"
    );
}

/// xorshift, deterministic across runs.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

fn stress(simulated_secs: u64) {
    let (mut conductor, handle, device, mut now) = offline_conductor(model(8, 30));
    let players: Vec<PlayerId> = conductor.state().players.iter().map(|p| p.id).collect();
    let e = entries(conductor.state());
    let mut rng = Rng(0x5eed);
    let mut last_move: HashMap<PlayerId, (f64, u64, Instant)> = HashMap::new();
    let mut history: HashMap<PlayerId, Vec<(u64, String)>> = HashMap::new();
    let blocks = simulated_secs * 100;
    for block in 0..blocks {
        if block % 25 == 0 {
            let p = players[rng.below(players.len())];
            let command = match rng.below(10) {
                0..=2 => Command::Play(p),
                3 => Command::Pause(p),
                4 => Command::Stop(p),
                5 => Command::FadeStop(p),
                6 => Command::SetNext(p, e[rng.below(e.len())]),
                7 => Command::SetMode(
                    p,
                    if rng.below(2) == 0 {
                        PlayMode::Single
                    } else {
                        PlayMode::Continuous
                    },
                ),
                8 => Command::Seek(p, rng.below(900) as f64 / 1000.0),
                _ => Command::SetVolume(p, rng.below(100) as f32 / 100.0),
            };
            history
                .entry(p)
                .or_default()
                .push((block, format!("{command:?}")));
            handle.send(command);
        }
        conductor.tick(now);
        device.render(BLOCK).unwrap();
        now += Duration::from_millis(10);
        if block % 4 == 0 {
            std::thread::yield_now();
        }
        // A playing (not paused) player must never stand still for long.
        let model = handle.model.load();
        let telemetry = handle.telemetry.load();
        for (id, t) in &telemetry.players {
            let playing = model
                .player(*id)
                .is_ok_and(|p| p.transport == Transport::Playing);
            match (playing, t.position_secs) {
                (true, Some(pos)) => {
                    let entry = last_move.entry(*id).or_insert((pos, block, Instant::now()));
                    if (pos - entry.0).abs() > f64::EPSILON {
                        *entry = (pos, block, Instant::now());
                    }
                    // Simulated time runs far faster than the (real-time)
                    // worker threads, so "stuck" also requires real time to pass.
                    let stuck =
                        block - entry.1 >= 500 && entry.2.elapsed() > Duration::from_secs(2);
                    if stuck {
                        let h = history
                            .get(id)
                            .map(|v| v[v.len().saturating_sub(8)..].to_vec());
                        let pl = model.player(*id).unwrap();
                        panic!(
                            "ENGINE {} || player {id:?} stuck at {pos} since block {}; now {block}; state {:?} cur {:?} next {:?} fading {}; history {h:#?}",
                            conductor.engine().describe_player(*id),
                            entry.1,
                            pl.transport,
                            pl.current,
                            pl.next,
                            pl.fading
                        );
                    }
                }
                _ => {
                    last_move.remove(id);
                }
            }
        }
    }
    assert_eq!(handle.telemetry.load().dropped_commands, 0);
}

#[test]
fn eight_players_survive_ten_simulated_minutes_of_random_commands() {
    stress(600);
}

/// The spec's long soak (spec §11): run with `cargo test -- --ignored`.
#[test]
#[ignore = "long soak: 6 simulated hours"]
fn eight_players_survive_six_simulated_hours_of_random_commands() {
    stress(6 * 3600);
}
```

Notes on the stress test:
- **What it checks:** 8 players share one device and receive a random command every 250 ms for 10 simulated minutes. A playing, non-paused player must advance: "stuck" means 5 simulated seconds **and** 2 real seconds without movement, because simulated time runs far faster than the real-time worker threads. No bus command may ever be dropped. On failure it prints the player's last commands and the engine's view (`describe_player`).
- **The ignored variant** runs 6 simulated hours (spec §11). Run it with `cargo test --release -p fp-engine --test conductor -- --ignored`; it takes about 11 s in release.

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-engine --test conductor`
Expected: FAIL to compile: `could not find conductor in fp_engine`.

- [ ] **Step 3: Implement**

`crates/fp-engine/src/lib.rs` (whole file):
```rust
//! The audio engine (spec §4): decoded `Source`s feed per-device bus `Mixer`s
//! on real-time threads; per-player worker threads decode and resample; the
//! `Conductor` turns model `EngineAction`s into sample-accurate bus commands.

#![deny(clippy::indexing_slicing)]

pub mod atomic;
pub mod bus;
pub mod conductor;
pub mod decode;
pub mod engine;
pub mod mixer;
pub mod ramp;
pub mod resample;
pub mod source;
pub mod worker;
```

`crates/fp-engine/src/conductor.rs`:
```rust
//! The conductor thread (spec §2.2, §4.4): the single owner of the model
//! state. It applies UI commands and engine events through the pure model
//! reducer, executes the resulting actions on the engine, and publishes
//! immutable snapshots. The UI never locks anything the audio path waits on.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use arc_swap::ArcSwap;
use crossbeam_channel::{Receiver, Sender, TrySendError};
use fp_model::{AppState, Command, EngineAction, ModelError, PlayerId, apply, on_event};

use crate::engine::{BusStatus, Engine, PlayerTelemetry};

/// Live values for the UI, refreshed every tick.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Telemetry {
    pub players: Vec<(PlayerId, PlayerTelemetry)>,
    pub buses: Vec<BusStatus>,
    /// Increases every time the model snapshot changes (drives saving).
    pub model_version: u64,
    pub dropped_commands: u64,
}

pub struct Conductor {
    state: AppState,
    engine: Engine,
    commands: Receiver<Command>,
    rejected: Sender<ModelError>,
    model: Arc<ArcSwap<AppState>>,
    telemetry: Arc<ArcSwap<Telemetry>>,
    version: u64,
}

/// The UI's side of the conductor.
pub struct ConductorHandle {
    commands: Sender<Command>,
    pub model: Arc<ArcSwap<AppState>>,
    pub telemetry: Arc<ArcSwap<Telemetry>>,
    /// Commands the model refused, with the reason (for a UI notice).
    pub rejected: Receiver<ModelError>,
    stop: Option<Arc<AtomicBool>>,
    thread: Option<JoinHandle<()>>,
}

impl ConductorHandle {
    /// Queues a command; never blocks. `false` if the queue is full.
    pub fn send(&self, command: Command) -> bool {
        match self.commands.try_send(command) {
            Ok(()) => true,
            Err(TrySendError::Full(c)) => {
                tracing::warn!(command = ?c, "command queue full; command dropped");
                false
            }
            Err(TrySendError::Disconnected(_)) => false,
        }
    }
}

impl Drop for ConductorHandle {
    fn drop(&mut self) {
        if let Some(stop) = &self.stop {
            stop.store(true, Ordering::Release);
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Conductor {
    /// Takes over `state` (typically restored from disk) and runs the
    /// `initial` actions that came with it (e.g. `LoadPaused`).
    pub fn new(
        state: AppState,
        initial: Vec<EngineAction>,
        mut engine: Engine,
        now: Instant,
    ) -> (Conductor, ConductorHandle) {
        for player in &state.players {
            engine.execute(EngineAction::AddPlayer { player: player.id }, now);
        }
        for action in initial {
            engine.execute(action, now);
        }
        let (tx, rx) = crossbeam_channel::bounded(1024);
        let (rejected_tx, rejected_rx) = crossbeam_channel::bounded(64);
        let model = Arc::new(ArcSwap::from_pointee(state.clone()));
        let telemetry = Arc::new(ArcSwap::from_pointee(Telemetry::default()));
        let conductor = Conductor {
            state,
            engine,
            commands: rx,
            rejected: rejected_tx,
            model: model.clone(),
            telemetry: telemetry.clone(),
            version: 0,
        };
        let handle = ConductorHandle {
            commands: tx,
            model,
            telemetry,
            rejected: rejected_rx,
            stop: None,
            thread: None,
        };
        (conductor, handle)
    }

    pub fn state(&self) -> &AppState {
        &self.state
    }

    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    /// One iteration: commands, engine, events, snapshots.
    pub fn tick(&mut self, now: Instant) {
        let mut changed = false;
        let commands: Vec<Command> = self.commands.try_iter().collect();
        for command in commands {
            match apply(&mut self.state, command) {
                Ok(actions) => {
                    changed = true;
                    for action in actions {
                        self.engine.execute(action, now);
                    }
                }
                Err(e) => {
                    let _ = self.rejected.try_send(e);
                }
            }
        }
        for event in self.engine.tick(now) {
            changed = true;
            for action in on_event(&mut self.state, event) {
                self.engine.execute(action, now);
            }
        }
        if changed {
            self.version += 1;
            self.model.store(Arc::new(self.state.clone()));
        }
        let players = self
            .state
            .players
            .iter()
            .map(|p| (p.id, self.engine.telemetry(p.id)))
            .collect();
        self.telemetry.store(Arc::new(Telemetry {
            players,
            buses: self.engine.bus_status(),
            model_version: self.version,
            dropped_commands: self.engine.dropped_commands(),
        }));
    }

    /// Runs the conductor on its own thread, ticking every `period`.
    pub fn spawn(
        mut self,
        mut handle: ConductorHandle,
        period: Duration,
    ) -> std::io::Result<ConductorHandle> {
        let stop = Arc::new(AtomicBool::new(false));
        let flag = stop.clone();
        let thread = std::thread::Builder::new()
            .name("fp-conductor".to_owned())
            .spawn(move || {
                while !flag.load(Ordering::Acquire) {
                    let started = Instant::now();
                    self.tick(started);
                    if let Some(rest) = period.checked_sub(started.elapsed()) {
                        std::thread::sleep(rest);
                    }
                }
            })?;
        handle.stop = Some(stop);
        handle.thread = Some(thread);
        Ok(handle)
    }
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-engine --test conductor && cargo test --release -p fp-engine --test conductor -- --ignored`
Expected: 5 tests PASS (1 ignored in the first run), then the soak PASSES.

- [ ] **Step 5: Full workspace check**

Run: `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
Expected: 163 tests pass and 1 is ignored.

- [ ] **Step: Check and commit**

Run: `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
Expected: everything passes, no warnings.
```bash
git add crates/fp-engine
git commit -m "feat(engine): add conductor thread with snapshots and stress tests

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

## Spec coverage of this plan

| Spec | Covered | Deferred |
|---|---|---|
| §2.2 threads: RT bus threads, one worker per player, conductor | Tasks 5–9 | real-time priority for *worker* threads (not in std; OS-specific) → Phase 3 |
| §2.3 lock-free communication, retired queue, `try_lock` only | Tasks 4, 5, 7, 9 | — |
| §2.4 no hardcoded limits (tuning, derived capacity, growable slots) | Tasks 5, 7, 8 | — |
| §4.1 sources, ring size from `prebuffer_secs`, ready threshold, seek as replace-source | Tasks 4, 6, 8 | — |
| §4.2 mono/multichannel downmix | Task 6 | — |
| §4.3 mixer, sample-accurate commands/events, equal-power crossfades | Tasks 3, 5, 8 | — |
| §4.4 conductor, preload, sample-accurate scheduling | Tasks 8, 9 | — |
| §4.5 worker supervision, panic containment, failures → `SourceFailed` | Tasks 6, 8 | — |
| §4.6 routes, channel pairs, shared devices | Tasks 5, 8 | multichannel buses → later |
| §4.7 device loss, virtual clock, reconnection, watchdog | Task 7 | UI alert → Plan 4 |
| §4.8 shared-mode sample rate, resampling | Tasks 6, 8 | bit-perfect → Phase 4 |
| §5 backend trait, Null, Offline, cpal default host, availability | Tasks 1, 2 | `subscribe_device_changes` and native backends → Phase 3 |
| §9 no RT logging, `indexing_slicing` denied, counters | Tasks 1, 5, 8 | panic hook and log files → Plan 4 |
| §11 engine tests on Offline, no-alloc guard, stress (10 min) and soak (6 h) | Tasks 5–9 | — |
