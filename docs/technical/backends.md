# Audio backends (`fp-backends`)

## Trait

```rust
pub trait AudioBackend: Send + Sync {
    fn id(&self) -> BackendId;
    fn availability(&self) -> Availability;          // Available | Unavailable(reason)
    fn enumerate_devices(&self) -> Result<Vec<DeviceInfo>, BackendError>;
    fn default_device(&self) -> Option<DeviceId>;
    fn open_output(&self, device: &DeviceId, config: StreamConfig,
                   renderer: Box<dyn Renderer>, errors: Arc<dyn StreamErrorSink>)
        -> Result<Box<dyn OutputStream>, BackendError>;
}
```

The engine only knows this trait. A `Renderer` is called on the device's
real-time thread with an interleaved `f32` buffer. Stream errors are reported
through an RT-safe sink: it sets atomics, which the bus watchdog reads.

## Implemented

| Backend | Id | Platforms | Build |
|---|---|---|---|
| `CpalBackend` (one per cpal host) | `alsa`, `pulseaudio`, `pipewire`, `jack`, `wasapi`, `asio`, `coreaudio` | per OS | see below |
| `NullBackend` | `null` | all | Discards audio at real-time pace; keeps timelines running with no sound card |
| `OfflineBackend` | `offline` | tests | Devices rendered on demand on the caller's thread, for sample-exact tests |

`system_backends()` returns one `CpalBackend::for_host` for every host
compiled into the build (`cpal::ALL_HOSTS`), the platform default first.

- **Hosts:** each backend creates its cpal host once and keeps it. Some
  systems open a server connection per host. A host that cannot start (a
  missing library, a JACK server that is not running) makes the backend
  `Unavailable(reason)`.
- **Streams:** they use the same code for every host: the sample format is
  chosen as F32, then I32, then I16 and converted through a preallocated
  buffer; the requested buffer size is tried first, then the device default.
  Real-time priority comes through cpal's `audio_thread_priority` (rtkit
  over D-Bus on Linux).
- **Default backend:** `preferred_backend` picks the first available one in
  the OS order (Linux: PipeWire, PulseAudio, JACK, ALSA; Windows: WASAPI,
  ASIO, JACK; macOS: Core Audio, JACK). A configured backend that is not
  available at start-up is replaced by that choice for the default output,
  with a warning. Routes keep naming their own backend and retry while it is
  away.

| Cargo feature (`fp-backends`, forwarded by `fp-app`) | System | Build needs |
|---|---|---|
| `pulseaudio` (default) | PulseAudio (a pure-Rust client, also works with PipeWire's PulseAudio server) | nothing |
| `pipewire` | PipeWire | `libpipewire-0.3-dev`, `libspa-0.2-dev`, clang |
| `jack` | JACK | Linux: `libjack-dev` (the library is loaded at run time); Windows, macOS: nothing |
| `asio` | ASIO (Windows) | the Steinberg ASIO SDK (`CPAL_ASIO_DIR`) |

CI builds and tests with `pipewire,jack` on Linux and `jack` elsewhere; the
release archives use the same features.

## Conformance suite

`fp-backends/tests/conformance.rs` runs the same checks against any
backend:

- it is available and lists named devices, including the default device;
- the default device opens, and the stream reports a sane configuration;
- the renderer is called;
- callbacks stop within a second after the stream is dropped;
- the device opens again.

Null and Offline run in CI. On a machine with real systems, run
`cargo test -p fp-backends --test conformance -- --ignored` (with the
features you built): that is the on-device check each system needs before a
release.

## Not provided

- **WASAPI exclusive and Core Audio hog mode** belong to Phase 4
  (bit-perfect). Exclusive access only matters there, and both need
  platform APIs beyond cpal.
- **DirectSound** is not supported: it is deprecated and WASAPI supersedes
  it.

## Adding a backend

1. Implement `AudioBackend` in a new module of `fp-backends`. Prefer crates
   with safe APIs. The workspace forbids `unsafe_code`, and `forbid` cannot be
   overridden by `allow`, so a backend that truly needs FFI requires an
   explicit decision: change the lint level for `fp-backends` only (to
   `deny`, with a module-scoped `allow`), plus `// SAFETY:` comments and
   tests. Record that decision in the spec.
2. Never allocate or block inside the render callback. Convert formats
   through buffers allocated when the stream opens.
3. Register it in `fp-app/src/main.rs` (the list passed to `Engine::new`).
4. Document it in this file and in the user guide's Settings page.
