# Phase 3 · Plan 1 — Native audio systems

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans (or superpowers:subagent-driven-development) to implement this plan task by task.

**Goal:** Offer every audio system each OS has, not just the platform default:
- Linux: PipeWire, PulseAudio, JACK and ALSA;
- Windows: WASAPI (shared), JACK and ASIO (when built with the SDK);
- macOS: Core Audio and JACK.

The user picks one in Settings; missing ones show as unavailable, and the app still starts. A conformance suite checks every backend the same way.

**Architecture:**
- **One backend per cpal host.** cpal 0.18 implements these systems with safe APIs:
  - `pipewire` (pipewire-rs);
  - `pulseaudio` (a pure-Rust client, no libpulse);
  - `jack` (dynamically loaded on macOS and Windows);
  - `asio`.
- **`CpalBackend::for_host(HostId)` generalises the existing backend.** `fp-backends` exposes `system_backends()`: one backend per host compiled in, each reporting its own availability (a host that fails to initialise is `Unavailable(reason)`).
- **Cargo features on `fp-backends` and `fp-app`:**
  - `pulseaudio` (default on Linux);
  - `pipewire`, which needs `libpipewire-0.3-dev`, `libspa-0.2-dev` and clang at build time;
  - `jack`, which needs `libjack-dev` at build time on Linux only;
  - `asio`, which needs the Steinberg ASIO SDK (`CPAL_ASIO_DIR`) and is off by default.

  CI and releases enable `pipewire` and `jack` on Linux and `jack` on macOS and Windows.

**Spec changes (recorded in the parent spec §5.2 and §13):**
- WASAPI exclusive and Core Audio hog mode move to Phase 4. Exclusive access only matters for bit-perfect output, and both need platform APIs beyond cpal.
- DirectSound is dropped: Microsoft deprecated it, WASAPI supersedes it, and cpal no longer ships it.

## Global Constraints

- Rules carried over: no `unsafe` (every backend comes through cpal's safe API); the real-time rules in the render path (the existing format conversion is reused); no hardcoded limits; English; TDD; the docs kept in sync.
- The app must start and play on a machine that lacks any optional system. A missing library is `Unavailable`, never a crash.

## Review Focus

1. **A configured backend that is missing at run time** (a config from a machine with JACK, run where JACK is not installed). The routes fall back to the default output (the Phase 1 rule), Settings shows the backend as unavailable, and nothing crashes.
2. **The order of the default backend on Linux.** With nothing configured, PipeWire is preferred, then PulseAudio, then ALSA. The choice is logged and visible in the status bar.
3. **A JACK server that is not running.** JACK is `Unavailable("server not running")` rather than hanging at start-up.
4. **Two backends open at once** (Main on PulseAudio, Cue on ALSA). Each bus uses its own backend. Both play, and the watchdog handles them independently.
5. **Device ids from one backend used with another.** A route stores the backend with its device, so an id is never looked up in the wrong backend.

---

### Task 1: One backend per host

- `CpalBackend::for_host(host: cpal::HostId) -> Self`. Its id is the host name in lower case (`alsa`, `pulseaudio`, `pipewire`, `jack`, `wasapi`, `asio`, `coreaudio`).
- `availability()` tries `cpal::host_from_id` once, lazily, and caches `Unavailable(reason)` when it fails.
- `fp_backends::system_backends() -> Vec<Arc<dyn AudioBackend>>` covers every host in `cpal::available_hosts()`, plus the compiled-in hosts that are not available (listed as unavailable, with a reason).
- `default_system_backend(backends)` picks the preferred order per OS:
  - Linux: PipeWire, PulseAudio, JACK, ALSA;
  - Windows: WASAPI, then ASIO;
  - macOS: Core Audio, then JACK.

  An unavailable backend is skipped.
- **Tests:**
  - `each_host_is_a_backend_with_its_own_id`;
  - `unavailable_hosts_report_a_reason`, with an injected host factory;
  - `the_default_backend_follows_the_preferred_order`, with a pure function over ids and availability.

### Task 2: Conformance suite

- A generic test module, `fp-backends/tests/conformance.rs`, over `&dyn AudioBackend`:
  - devices enumerate, and every id reopens;
  - a stream opens at 48 kHz stereo with the requested buffer or a fallback;
  - the renderer is called, and the sample format is converted;
  - dropping the stream stops callbacks within 1 s;
  - the error sink is called on device loss, where it can be simulated.
- Null and Offline run it in CI.
- Every real system runs it under `#[ignore]` (`cargo test -p fp-backends --test conformance -- --ignored`) for the manual on-device check that Phase 3's exit criterion requires.

### Task 3: Wiring and Settings

- `main.rs` uses `system_backends()` and `default_system_backend`. The status bar label shows the backend in use.
- Settings → Audio outputs already lists backends with their availability. Check the labels (pretty names: "PipeWire", "PulseAudio", "JACK", "ALSA", "WASAPI", "ASIO", "Core Audio").
- The engine's default backend when `outputs.backend` is `None` is `default_system_backend`, instead of "the first registered".
- **Test:** `the_engine_defaults_to_the_preferred_backend`, on Offline, with two fake backends.

### Task 4: Features, CI and packaging notes

- Features in `fp-backends` and `fp-app`:
  - `pulseaudio`, on by default;
  - `pipewire`, `jack` and `asio`, off by default.

  The release build enables `pipewire` and `jack` on Linux and `jack` on macOS and Windows.
- **CI:**
  - the Linux job installs `libpipewire-0.3-dev libspa-0.2-dev libjack-jackd2-dev clang` and builds and tests with `--features pipewire,jack`;
  - the other OSes use `--features jack`.
- **Docs:**
  - README platform table;
  - `docs/technical/backends.md`;
  - the user guide: troubleshooting (JACK server, PipeWire) and settings;
  - build instructions per OS for the optional systems.
- **Verification:** full checks, a local conformance run on the systems available here, a fresh review, fixes, then merge.
