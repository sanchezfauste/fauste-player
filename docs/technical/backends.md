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

`DeviceInfo::detail` is what sets a device apart from others with the same
name: cpal's ALSA host lists every output profile of a card (front,
surround, direct hardware…) under the card's name, and the first extended
description line that is not the name names the profile (else the device's
address, when it differs from the name). Pickers show
`device_labels(&devices)`: the name, then ` — detail`, then ` (id)` for any
label still shared.

`OutputStream::config()` is the configuration in use, not the one asked for:
cpal reports the fixed buffer size it was given or, with the device's
default, the largest block called back so far; WASAPI exclusive reports the
buffer size the driver settled on. When WASAPI refuses a period and the
aligned retry fails too, the error keeps the first refusal (the reason), and
the bus logs it when it falls back to shared.

**Exclusive access (Phase 4):**
- `StreamConfig::exclusive` asks for sole, unconverted access. A backend that
  cannot give it for the device returns `BackendError::Unsupported`.
- `DeviceInfo::exclusive_capable` says which devices can give it, and
  `rate_switching` says which can be reopened at another rate.
- `OutputStream::sample_format()` gives the format the device really runs in
  (`F32`, `I32`, `I24` (24 bits in 32), `I16`). cpal has no packed 3-byte
  24-bit format, so devices that accept only `S24_3LE` cannot be opened.
- `SampleFormat::holds_bits` says whether integer PCM of a given size passes
  unchanged through the f32 mixer and the format: 24 bits for F32, I32 and
  I24, and 16 for I16.
- The f32 → I16, I24 and I32 conversions are exact for integer PCM (they
  scale by powers of two), and tests pin that.

**Exclusive-capable devices:**
- **Exclusive-capable devices** (`exclusive_capable(host, id)`, which reads
  cpal's persisted ids such as `alsa:hw:CARD=…,DEV=…`):
  - ALSA `hw:` devices. They are the hardware itself and open through cpal.
  - Every WASAPI device (`wasapi_exclusive.rs`, Windows).
    - The `wasapi` crate's safe API runs on a render thread of its own, since
      COM objects stay on the thread that created them.
    - That thread negotiates the format in this order: 24-in-32 integer,
      32-bit integer, 16-bit integer, then float (`exclusive::negotiate`).
    - It aligns the period, with the documented retry for unaligned buffers.
    - It reports the outcome to `open`. An open that takes longer than 5 s
      is refused as `Unsupported`, so the bus plays shared.
    - Then it raises itself to real-time priority (MMCSS through
      `audio_thread_priority`) and primes one silent buffer.
    - On each buffer event it renders into preallocated buffers
      (`exclusive::write_samples`).
    - Events are awaited in 20 ms slices, so a stop never waits for a stuck
      driver.
    - Two seconds without an event, or any failure, is reported as
      `DeviceLost`.
  - Every Core Audio device (`coreaudio_hog.rs`, macOS).
    - Hog mode is taken before the cpal stream is built.
    - After the build (which sets a float physical format of its own) and
      before play, `HogGuard::prepare` sets the nominal rate and the widest
      signed-integer linear-PCM physical format at that rate, with at least
      the stream's channels (`exclusive::choose_physical_format`). Encoded
      formats (AC-3, IEC 60958) are never chosen.
    - The stream reports that hardware format.
    - The device is found by name: a name shared by several devices, or a
      device another process holds, is refused.
    - `HogGuard` releases hog mode after the stream stops.
  - Any other exclusive request is refused with `Unsupported`, so the bus
    plays shared and never claims bit-perfect output.

  Only safe APIs are used, and `forbid(unsafe_code)` holds.
- Offline devices can be marked capable (`set_exclusive_capable`) or made to
  refuse a rate (`refuse_rate`), for tests.
- Null refuses `exclusive`, so a bus on it plays shared and never claims to
  be bit-perfect.

## Implemented

| Backend | Id | Platforms | Build |
|---|---|---|---|
| `CpalBackend` (one per cpal host) | `alsa`, `pulseaudio`, `pipewire`, `jack`, `wasapi`, `asio`, `coreaudio` | per OS | see below |
| `NullBackend` | `null` | all | Discards audio at real-time pace; keeps timelines running with no sound card |
| `OfflineBackend` | `offline` | tests | Devices rendered on demand on the caller's thread, for sample-exact tests |

`system_backends()` returns one `CpalBackend::for_host` for every host
compiled into the build (`cpal::ALL_HOSTS`), in the OS preference order.
Nothing connects to a system until it is first used.

- **Hosts:** each backend creates its cpal host on first use and keeps it,
  because some systems open a server connection per host. A host that
  failed to open is not kept. One whose stream reported the device lost, or
  whose open failed, is rebuilt on the next use, so the watchdog's reopen
  reaches a restarted server instead of a dead connection.
- **Availability:** `host_availability` gives `Available` only when the host
  opens *and* has an output device. A missing library makes it
  `Unavailable(reason)`, and so does a JACK host with no server (cpal opens
  it anyway, with no ports): "no output device (is its server running?)".
- **Channels:** a device reports the most channels any of its
  configurations offers, except on PulseAudio, where every sink offers every
  count it could remix to; there the sink's own layout (its default
  configuration) is used.
- **Streams:** they use the same code for every host: the sample format is
  chosen as F32, then I32, then I16 and converted through a preallocated
  buffer; the requested buffer size is tried first, then the device default.
  Real-time priority comes through cpal's `audio_thread_priority` (rtkit
  over D-Bus on Linux).
- **Default backend:** `choose_default_backend` keeps the configured
  backend when it is available, otherwise `preferred_backend` picks the
  first available one in the OS order (Linux: PipeWire, PulseAudio, JACK,
  ALSA; Windows: WASAPI, ASIO, JACK; macOS: Core Audio, JACK), with a
  warning.
- **Routes:** the engine notes which backends were unavailable when it
  started. A Main route to one plays on the default output, and a Cue route
  to one means no cue, as for a backend this build does not have. A backend
  that was available and is lost later keeps its routes, and the watchdog
  retries it.

| Cargo feature (`fp-backends`, forwarded by `fp-app`) | System | Build needs |
|---|---|---|
| `pulseaudio` (default) | PulseAudio (a pure-Rust client, also works with PipeWire's PulseAudio server) | nothing |
| `pipewire` | PipeWire, linked (the binary needs libpipewire 0.3.53 or later to start) | `libpipewire-0.3-dev`, `libspa-0.2-dev`, clang |
| `jack` | JACK, loaded at run time (the binary starts without it) | Linux: `libjack-jackd2-dev`; Windows, macOS: nothing |
| `asio` | ASIO (Windows) | the Steinberg ASIO SDK (`CPAL_ASIO_DIR`) |

CI builds and tests with `pipewire,jack` on Linux and `jack` elsewhere. The
release archives use `jack` only: `pipewire` would make libpipewire a
start-up dependency, and the release image (Ubuntu 22.04, chosen for its
older glibc) has a libpipewire older than the bindings need. Distribution
packages can enable it. CI's `release-baseline` job builds on that image,
and `scripts/check-runtime-deps.sh` fails any Linux binary that links a
library beyond ALSA, D-Bus and the C runtime.

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
