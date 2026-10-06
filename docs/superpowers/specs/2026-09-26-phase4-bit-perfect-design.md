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

- **Analysis records the format:** `TrackAnalysis` gains `format: Option<AudioFormat>`, holding:
  - the sample rate;
  - the bits per sample, which are `None` for lossy codecs, whose decoded output is not integer PCM;
  - the channel count. Files with more than two channels are downmixed, so they are never bit-perfect.
- **The track keeps it:** `Track::format` is persisted with serde default `None`.
- **The request carries it:** `SourceRequest.format` and `CartRequest.format` pass it to the engine, so the engine decides rates with no file I/O.
- `ANALYSIS_VERSION` becomes 4. A track that is analysed but has no format (a library from an earlier version) is analysed again, once, when the operator asks or a player shows it (main spec, analysis cache).
- **A track not yet analysed** (format unknown) plays at the bus rate, as in Phase 1.

## B3. Stream rate follows the file

Rules for a bit-perfect bus:

1. **Rate:** each bus has its own rate. A normal bus runs at `outputs.sample_rate`. A bit-perfect bus starts at `outputs.sample_rate` and changes only as below.
2. **The rate is decided when a source starts, not when it is attached.** A source *starts* when:
   - a track is played (`StartCurrent`);
   - a track loaded paused is resumed for the first time;
   - a pre-listen starts;
   - a cart is fired or pre-listened.

   If the bus is not **sounding**, the file's format is known and its rate differs, the bus is reopened at that rate first. A bus is sounding while any source on it is started, has a start requested, or is waiting only to be buffered, or while a test tone plays.
   - The mixer, its frame counter and its slots survive the reopen; only the device stream is replaced.
   - Every source that is only **waiting** on that bus (preloads, a track loaded paused, of any player) is re-created at the new rate. It has not played a frame, so no timeline moves.
   - Reopening takes as long as the device needs to start (typically tens of milliseconds). This gap is the documented cost of changing rate.
   - Preloads never decide the rate: at start-up every player preloads, and the first preload would otherwise fix the rate for the session.
3. **A sounding bus keeps its rate:** a source that starts on a sounding bit-perfect bus at another rate is resampled to the bus rate, as in Phase 1, and is not bit-perfect. For example, a segue from a 44.1 kHz track into a 48 kHz one plays the 48 kHz track resampled for its whole length. A bus's rate never changes while anything on it sounds, because every frame-based timeline on the bus (planned stops, segues, positions) is in that bus's frames.
4. **A failed reopen:** if the reopen at the new rate fails (the device does not support the rate), the bus goes back to its previous rate, the source is resampled, and a warning is logged. If the previous rate also fails, the bus is `Lost` and follows the Phase 1 watchdog rules. A device that answers **busy** (another application, or the sound server, took it in the moment the stream was closed) has refused nothing: the open is tried again up to `tuning.device_busy_retries` times (default 3), `tuning.device_busy_retry_ms` apart (default 20 ms; the conductor waits meanwhile, so both are kept small). If it stays busy, the bus goes back to its previous rate as above, with a warning that the device is busy, but the rate is **not** remembered as refused: the next start asks for it again. Only a real refusal of the rate or format is remembered.
5. **Timelines use the bus rate:** everything that converts between seconds and frames for a source uses the rate of that source's bus: fades, declicks, planned stops, positions, loop points and test tones. The worker opens each source at the rate the engine gives it in `LoadOptions.rate`.

## B4. Exclusive access and sample format

- `StreamConfig` gains `exclusive: bool`. A bit-perfect bus asks for it.
- A backend that cannot honour it for a device returns `BackendError::Unsupported`. The bus then retries without `exclusive`, logs a warning, and the BP badge stays off.
- On ALSA, a `hw:` device is exclusive by itself; any other ALSA device refuses `exclusive`.
- `OutputStream` gains `sample_format() -> SampleFormat` (`F32`, `I32`, `I24` or `I16`), the format the device really runs in. `I24` is 24 bits in a 32-bit container.
- **The chosen format:** F32, then I32, then I24, then I16. A source is bit-perfect only if its bits pass unchanged:
  - The mixer works in f32 (a 24-bit mantissa), so no source above 24 bits is ever bit-perfect. That includes 32-bit integer files and 32-bit float files.
  - F32, I32 and I24 devices hold up to 24 bits.
  - I16 devices hold up to 16 bits.
- **The conversion to integer formats must be exact** for integer PCM: f32 → I16 for 16-bit sources, and f32 → I32 or I24 for 16- and 24-bit sources. This is covered by tests.
- **Packed 24-bit devices:** devices that take only packed 3-byte 24-bit samples (`S24_3LE`, common on USB DACs) are not supported by the audio library, and cannot be opened.
- **Null** refuses `exclusive`.

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
