# Phase 4 · Plan 2 — Platform exclusive modes

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans (or superpowers:subagent-driven-development) to implement this plan task by task.

**Goal:** bit-perfect devices on Windows and macOS.
- **WASAPI exclusive mode:** an event-driven render thread through the `wasapi` crate.
- **Core Audio hog mode:** it sets the device's nominal rate and physical format through coreaudio-rs's safe helpers.

Both plug into Plan 1's contract, so the engine does not change:
- `StreamConfig::exclusive`;
- `DeviceInfo::exclusive_capable`;
- `OutputStream::sample_format`.

**Architecture:**
- **WASAPI** (`crates/fp-backends/src/wasapi_exclusive.rs`, `cfg(windows)`). When the `wasapi` cpal backend is asked for `exclusive`, it opens the device through the `wasapi` crate instead of cpal:
  - **Lookup:** `DeviceEnumerator::get_device(id)`. cpal's WASAPI device id is the endpoint id string (`IMMDevice::GetId`), the same one the crate takes.
  - **Format:** tried in order with `is_supported_exclusive_with_quirks`: 32-bit integer container with 24 valid bits (reported `I32`), 16-bit integer (reported `I16`), then 32-bit float (reported `F32`).
  - **Period:** `calculate_aligned_period_near(requested frames)`, with the documented `AUDCLNT_E_BUFFER_SIZE_NOT_ALIGNED` retry.
  - **Render thread** (`fp-wasapi-render`). COM objects are not `Send`, so the thread itself opens the device, negotiates the format and initialises the client, then reports success or the error back to `open_output` through a channel (bounded wait). Its steps:
    1. `initialize_mta`, device lookup and format negotiation;
    2. `set_get_eventhandle`;
    3. `start_stream`;
    4. loop: `wait_for_event(timeout)`, then `get_available_space_in_frames`, then `Renderer::render` into a preallocated f32 buffer, then convert to the device bytes (little-endian, exact for integer PCM, the same scaling as Plan 1), then `write_to_device`.
  - **Errors:** a timeout or error is reported through the error sink (`DeviceLost` for device invalidation, `Other` otherwise), and the thread ends.
  - **Stop:** dropping the stream sets a stop flag, stops the client and joins the thread.
- **Core Audio** (`crates/fp-backends/src/coreaudio_hog.rs`, `cfg(target_os = "macos")`). When the `coreaudio` backend is asked for `exclusive`:
  1. **Find the device:** from the cpal device's name, `get_device_id_from_name(name, false)`. If two devices share the name, hog mode is refused (`Unsupported`), and the bus falls back to shared as in Plan 1.
  2. **Take hog mode:** `toggle_hog_mode` unless `get_hogging_pid` is already this process. If another process holds it, refuse.
  3. **Rate and format:**
     - `set_device_sample_rate(id, rate)`;
     - `find_matching_physical_format` or `set_device_physical_stream_format`, choosing the widest integer physical format at that rate;
     - the stream reports `I32` when the physical format has ≥ 24 bits and `I16` when it has 16. The HAL converts cpal's f32 exactly for those sizes.
  4. **Open** the cpal stream as today.

  A `HogGuard` inside the stream releases hog mode (toggles back) on drop.
- `exclusive_capable` becomes true for WASAPI and Core Audio output devices. Whether exclusive access is really granted is decided at open time; a refusal falls back to shared, with the BP badge off (Plan 1).

**Spec:** `docs/superpowers/specs/2026-09-26-phase4-bit-perfect-design.md` B4, B8.

## Global Constraints

- **Rules carried over:**
  - `forbid(unsafe_code)`: only the safe APIs of `wasapi` and `coreaudio-rs`;
  - the real-time rules on the render thread: no allocation, lock, log or panic;
  - no unwrap, expect or panic;
  - `indexing_slicing` denied;
  - English;
  - docs kept in sync.
- **Nothing changes on Linux.** The new code is `cfg`'d per OS.
- **Verification:**
  - CI builds and tests on `windows-latest` and `macos-latest`;
  - locally, `cargo check --target x86_64-pc-windows-msvc` and `--target aarch64-apple-darwin` for `fp-backends`;
  - the manual loopback check from the user guide.

## Review Focus

1. **Another application holds the device exclusively,** or hog mode is held by another process. The open is refused and the bus plays shared, with the badge off. Nothing hangs.
2. **The device is unplugged during exclusive playback.** The render thread reports `DeviceLost` and ends. The watchdog runs the virtual clock and reopens, and hog mode is released.
3. **The app exits or the bus is dropped:** hog mode is released, the thread is joined, and no zombie thread or held device remains.
4. **A format the device lacks** (e.g. a 16-bit-only device, and a 24-bit file). The stream reports `I16`, and the badge is off for 24-bit files.
5. **Period alignment errors** on Intel HDA-like devices: the aligned retry succeeds.

---

### Task 1: Pure pieces, tested on every OS

- **`fp-backends::exclusive`:**
  - `write_samples(format, &[f32], &mut [u8])`: little-endian bytes for `I16`, `I24` (24 valid bits, MSB-aligned in 32) and `F32`, with the same exact scaling as the cpal path;
  - `negotiate(supported: impl FnMut(Candidate) -> bool) -> Option<Candidate>`: the WASAPI format order;
  - `choose_physical_format(&[PhysicalFormat], rate) -> Option<PhysicalFormat>`: the widest signed-integer format whose rate range contains `rate`, over a plain struct mirroring Core Audio's ranged description;
  - `exclusive_capable(host, id)` extended to `wasapi` and `coreaudio`.
- **Tests:** conversions exact for 16- and 24-bit PCM and saturating at ±1.0; negotiation order; physical-format choice (widest integer, rate range, none); capability per host.

### Task 2: WASAPI exclusive stream (`cfg(windows)`)

- The `wasapi = "0.24"` dependency, for the Windows target only.
- The `ExclusiveStream` type and the render thread as above; format negotiation as a pure function over the supported answers (tested everywhere).
- `CpalBackend::open_output` dispatches to it for the `wasapi` host when `exclusive` is set. The id's `wasapi:` prefix is stripped for `DeviceEnumerator::get_device`.
- **Verification:**
  - `cargo clippy --target x86_64-pc-windows-msvc -p fp-backends -- -D warnings` locally;
  - CI on windows-latest;
  - a `#[ignore]` real-device test.

### Task 3: Core Audio hog mode (`cfg(target_os = "macos")`)

- `HogGuard`, the rate and physical-format setup, and a pure `choose_physical_format` (tested everywhere).
- `CpalBackend::open_output` wraps the cpal stream with the guard for the `coreaudio` host when `exclusive` is set.
- **Verification:**
  - `cargo clippy --target aarch64-apple-darwin -p fp-backends -- -D warnings` locally;
  - CI on macos-latest;
  - a `#[ignore]` real-device test.

### Task 4: CI, docs, verification

- **CI:** the matrix already builds Windows and macOS. Add `cargo check` of the other OS targets to the Linux job, so a Linux-only change cannot break them unnoticed.
- **Docs:**
  - user guide `bit-perfect.md`: Windows and macOS sections, and the Windows setting "Allow applications to take exclusive control";
  - `backends.md`;
  - README table and roadmap: Phase 4 done.
- A fresh review, fixes, then merge.
