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

## Implemented (Phase 1)

| Backend | Id | Platforms | Notes |
|---|---|---|---|
| `CpalBackend` | the cpal host id (`alsa`, `wasapi`, `coreaudio`) | Linux, Windows, macOS | Sample format chosen as F32, then I32, then I16, converting through a preallocated scratch buffer. Tries the requested buffer size, then the device default. Real-time priority through cpal's `audio_thread_priority` (rtkit over D-Bus on Linux). |
| `NullBackend` | `null` | all | Discards audio at real-time pace; keeps timelines running with no sound card |
| `OfflineBackend` | `offline` | tests | Devices rendered on demand on the caller's thread, for sample-exact tests |

## Roadmap (spec §5.2)

| Phase | Backends |
|---|---|
| 3 | PipeWire, PulseAudio, JACK (dynamically loaded), WASAPI exclusive, ASIO (with the SDK at build time), DirectSound, Core Audio hog mode |
| 4 | Bit-perfect output on exclusive backends |

Each native backend will sit behind a Cargo feature. It reports
`Unavailable(reason)` when its system library is missing, so the application
still starts. A conformance test suite, shared by all backends, is part of
Phase 3.

## Adding a backend

1. Implement `AudioBackend` in a new module of `fp-backends`. FFI, if any, is
   the only place `unsafe` may appear. It needs an `#[allow(unsafe_code)]`
   scoped to the module, `// SAFETY:` comments and tests.
2. Never allocate or block inside the render callback. Convert formats
   through buffers allocated when the stream opens.
3. Register it in `fp-app/src/main.rs` (the list passed to `Engine::new`).
4. Document it in this file and in the user guide's Settings page.
