# Phase 4 — Bit-perfect output (design)

**Parent spec:** [`2026-09-25-fauste-player-design.md`](2026-09-25-fauste-player-design.md), §4.8, §5.2 and §13 (Phase 4).

**Goal:** a device can be set to *bit-perfect*. On such a device:
- a track plays at its own sample rate and format;
- when volume is 100 % and no fade or other source touches it, the samples reach the device unchanged;
- the player's **BP** badge lights only while that is actually true.

**Exit criterion (parent spec §13):** bit-exact loopback verification on at least one device per OS. That check is manual. The automated equivalent is a bit-exact test on the Offline backend, plus conversion tests for every device sample format.

---

## B1. Terms

- **Bit-perfect device:** a `(backend, device)` pair listed in `config.outputs.bit_perfect`. It opens in exclusive mode where the backend has one, and its stream rate follows the files it plays (B3).
- **Exclusive-capable:** `DeviceInfo::exclusive_capable`, meaning the backend can give the application sole, unconverted access to the device:
  - ALSA `hw:` devices (direct hardware access, exclusive by nature). This is Plan 1.
  - WASAPI exclusive mode and Core Audio hog mode. These are Plan 2.
  - PulseAudio, PipeWire, JACK, ALSA `plughw:`/`default`, Null and Offline are not exclusive-capable. Offline accepts the flag so that tests can exercise the path.
- **Unaltered source:** in a mixer block, a source is unaltered when both hold:
  - every gain applied to it was exactly 1.0 (volume 100 %, no fade or pause ramp in progress);
  - no other source put a non-zero sample on its channel pair.

  In f32, `x * 1.0` and `x + 0.0` are exact, so the mixer passes such a source through bit for bit with no special path.

## B2. File format

- **Analysis records the format:** `TrackAnalysis` gains `format: Option<AudioFormat>`, holding the sample rate and the bits per sample. Bits are `None` for lossy codecs, whose decoded output is not integer PCM.
- **The track keeps it:** `Track::format` is persisted with serde default `None`.
- **The request carries it:** `SourceRequest.format` and `CartRequest.format` pass it to the engine, so the engine decides rates with no file I/O.
- `ANALYSIS_VERSION` becomes 4, so existing libraries are re-analysed in the background and gain their formats.
- **A track not yet analysed** (format unknown) plays at the bus rate, as in Phase 1.

## B3. Stream rate follows the file

Rules for a bit-perfect bus:

1. **Rate:** each bus has its own rate. A normal bus runs at `outputs.sample_rate`. A bit-perfect bus starts at `outputs.sample_rate` and changes only as below.
2. **Rate change when idle:** when a source with a known format is about to attach to a bit-perfect bus that has **no attached sources** (idle), and the format's rate differs, the bus is reopened at that rate first.
   - The mixer, its frame counter and its slots survive the reopen; only the device stream is replaced.
   - Reopening takes as long as the device needs to start (typically tens of milliseconds). This gap is the documented cost of changing rate.
3. **A busy bus keeps its rate:** a source attaching to a busy bit-perfect bus at another rate is resampled to the bus rate, as in Phase 1, and is not bit-perfect. For example, a segue from a 44.1 kHz track into a 48 kHz one plays the 48 kHz track resampled for its whole length. A bus's rate never changes while anything is attached, because every frame-based timeline on the bus (planned stops, segues, positions) is in that bus's frames.
4. **A failed reopen:** if the reopen at the new rate fails (the device does not support the rate), the bus goes back to its previous rate, the source is resampled, and a warning is logged. If the previous rate also fails, the bus is `Lost` and follows the Phase 1 watchdog rules.
5. **Timelines use the bus rate:** everything that converts between seconds and frames for a source uses the rate of that source's bus: fades, declicks, planned stops, positions, loop points and test tones. The worker opens each source at the rate the engine gives it in `LoadOptions.rate`.

## B4. Exclusive access and sample format

- `StreamConfig` gains `exclusive: bool`. A bit-perfect bus asks for it.
- A backend that cannot honour it for a device returns `BackendError::Unsupported`. The bus then retries without `exclusive`, logs a warning, and the BP badge stays off.
- On ALSA, a `hw:` device is exclusive by itself; any other ALSA device refuses `exclusive`.
- `OutputStream` gains `sample_format() -> SampleFormat` (`F32`, `I32` or `I16`), the format the device really runs in.
- **The chosen format:** the existing order, F32 then I32 then I16, is kept. A source is bit-perfect only if the device format holds its bits:
  - F32 holds up to 24 bits;
  - I32 holds up to 32 bits;
  - I16 holds up to 16 bits.
- **The conversion to integer formats must be exact** for integer PCM: f32 → I16 for 16-bit sources, and f32 → I32 for 16- and 24-bit sources. This is covered by tests.

## B5. The BP badge

`PlayerTelemetry.bit_perfect` is true only when all of these hold:
- the player's current source plays on its **Main** bus;
- that bus is bit-perfect, its stream is open (`Ok`) and exclusive;
- the source's file rate equals the bus rate (no resampling);
- the file's bits are known and fit the device format;
- the mixer reports the source unaltered in its last block.

The badge in the player header uses it. It was inactive since Phase 1.

## B6. Settings

- **Settings → Audio outputs** lists the devices the routes use, each with a **Bit-perfect** checkbox.
  - The checkbox is disabled, with the reason in a tooltip ("This device is shared by a sound server"), when the device is not exclusive-capable.
  - Changes apply after a restart, like every output change.
- The cue devices can be bit-perfect as well, but the badge follows Main only.

## B7. Out of scope

- DSD and DoP.
- Integer mixing paths.
- Automatic volume bypass (the operator sets 100 %).
- Changing the rate while a bus is busy.
- Switching devices' clock sources.

## B8. Plans

- **Plan 1 (portable core):** everything above for ALSA `hw:` and Offline:
  - the format in analysis, the model and requests;
  - per-bus rates and the rate change;
  - `exclusive` and `sample_format`;
  - mixer unaltered tracking;
  - telemetry and the badge;
  - Settings;
  - docs.
- **Plan 2 (platform exclusive modes):**
  - WASAPI exclusive through the `wasapi` crate (a safe API; `forbid(unsafe_code)` holds);
  - Core Audio hog mode and the device nominal rate through `coreaudio-rs`'s safe helpers (`toggle_hog_mode`, `set_device_sample_rate`);
  - compile checks in CI, and a manual loopback procedure in the docs.
