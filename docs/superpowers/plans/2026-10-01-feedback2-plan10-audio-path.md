# Audio Path (Feedback 2, Plan 10) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Audit the whole audio path and fix what the audit confirms (O26), explain digital peak readings above 0 dBFS and let the maintainer choose a remedy (O34), list "No output (silent)" again (O27), and send DSD to bit-perfect devices unchanged, as DoP or native DSD, instead of always converting it to PCM (O25).

**Architecture:**
- **The audit comes first and may add tasks.** Task 1 only writes `docs/technical/audio-path-audit.md` (plus cheap probe tests). The coordinator then inserts one test-first fix task per confirmed defect before Task 3, and asks the maintainer to choose the O34 remedy before any O34 code exists (see "Audit follow-ups").
- **DSD travels through the existing f32 path as 16-bit words.** A DSD-direct source pushes, per stereo frame, 16 DSD bits per channel encoded exactly as an `f32` (`i16` value / 32768), at the *word rate* = DSD rate ÷ 16 (176.4 kHz for DSD64, 705.6 kHz for DSD256). The bus is reopened at the word rate. The mixer *copies* those words (no gain, no ramps, no summing) while the bus is in DSD mode, and fills every gap with the DSD idle word `0x6969`.
  - **DoP**: the bus renderer (`MixerRenderer`) replaces each word by a DoP 1.1 sample (marker `0x05`/`0xFA` alternating frame by frame, above the 16 bits) as 24-bit PCM in `f32`, which the device's 24- or 32-bit integer format carries exactly (`exclusive::write_samples` scales by powers of two).
  - **Native** (Linux, ALSA `hw:` devices that report a DSD format): a new ALSA output stream in `fp-backends` packs the words into the device's `DSD_U32_BE`/`…`/`DSD_U8` bytes.
  - The same source also carries the **PCM conversion** of the track, in a second ring kept in lockstep: the mixer meters it while DSD goes out, and plays it (with gain and fader) once the track is switched to PCM.
- **Behaviour lives in `fp-model`.** `fp_model::dsd` holds the pure decision (`dsd_decision`: whether DSD can go out unchanged and why not), the badge (`bp_badge`), the notice (`dsd_holds_others`) and the HoldOthers rules in the reducer (`plan_for`, `on_event(ReachedEnd)`, Play/Next/Previous, fade stop, volume). The engine reports `EngineEvent::DsdStarted { hold_others }` / `DsdEnded`, and executes the new `EngineAction::LeaveDsd`.
- **O27** is a filter removed in `fp-app` plus a ranking rule in `fp-backends::preferred_backend`.

**Tech Stack:** Rust 2024, cpal 0.18.2, `alsa` 0.11 (already in `Cargo.lock` through cpal; becomes a direct Linux dependency of `fp-backends` in Task 10), rtrb, rubato, egui/eframe 0.36.2, egui_kittest 0.36.2, serde.

**Spec:** `docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md` §11 (plan 10: O25, O26, O27, O34), §3 O5 (reversed in part by O27) and §15 (global constraints). Roadmap: `docs/superpowers/plans/2026-10-01-operator-feedback-2-roadmap.md` (plan 10, branch `feat/audio-path`). Main design spec `docs/superpowers/specs/2026-09-25-fauste-player-design.md`; Phase 4 bit-perfect rules B1–B6 are described in `docs/user/bit-perfect.md` and `docs/technical/audio-engine.md` ("Rates and bit-perfect buses").

**Base.** Branch `feat/audio-path` from `master` at `15ffce0` (after plan 13). Line numbers below are from that tree and move; steps name functions and quote the text to find. Find code with the CodeGraph index (`codegraph_explore`, `projectPath` = the repo) before grepping.

## Decisions taken

The spec leaves these open. Each is the most conservative reading; the final review may overturn any of them.

- **Where the settings live.** The spec names `audio.dsd_mix`, but `Config` has no `audio` group and every output setting is in `outputs`. The three fields go into `OutputsConfig`: `dsd_output: Vec<DsdDevice>` (one mode per device), `dsd_mix: DsdMix` and `dsd_silence_ms: f64`. A mode applies only to a device that is also in `outputs.bit_perfect` (`OutputsConfig::dsd_output_for`).
- **They apply at restart.** The engine is built from `EngineSettings::from_config` once, like the bit-perfect list. `restart_pending` gains `RestartReason::DsdOutput` (any change to the three fields). The model never reads `dsd_mix` itself: the engine reports the policy in force with each `DsdStarted { hold_others }`, so model and engine cannot disagree after a Settings change that waits for a restart.
- **Silence time.** `outputs.dsd_silence_ms`, default 200 ms, range 0–2000 ms, edited in the configuration file only (like the engine tuning). It is sent when a DSD stream starts (before the first DSD byte), when it ends (before the device leaves DSD) and on every switch to PCM. Pause, resume and seek inside a DSD stream need none: the stream never stops, the gap is DSD silence.
- **When DSD goes out unchanged** (`dsd_decision`, in this order): the device's mode is not `Pcm`; the track is DSD (`AudioFormat::dsd_rate` known); it has one or two channels (more are downmixed, so never direct); the player's volume is exactly 1.0 (the spec's "the fader and the gain stay at unity"; starting a DSD stream louder than the operator set would be a surprise); nothing sounds on the device (`bus_sounding`). The engine then reopens the bus at the word rate with `StreamConfig::dsd`; a refused rate, a refused DSD stream, or (DoP) a device format that is not 24- or 32-bit integer falls back to PCM. Every fallback is logged with its reason (`DsdFallback`'s `Display`) unless the mode is `Pcm` or the file is not DSD (today's behaviour, nothing to say). Native that fails falls back to PCM, not to DoP (the operator chose native; DoP is a different signal for the DAC).
- **Unity while direct.** A fader move to anything but 100 % while DSD goes out unchanged ends DSD direct: the model clears `player.dsd` and emits `EngineAction::LeaveDsd` before `SetVolume`; the engine switches the track to PCM (silence, then PCM at the new volume). No command is refused (MIDI faders send volume constantly).
- **Fade stop** of a DSD-direct track stops it at once (the model emits `StopNow` instead of `FadeOutAndStop`): a DSD stream cannot change level, and silence followed by a PCM fade would sound like a restart.
- **Operator transport under the two policies.** `ConvertToPcm`: Play/Next/Previous crossfade as usual; the incoming source is "another source needs the output", so the engine switches the DSD track to PCM first. `HoldOthers`: Play/Next/Previous start the new entry hard (`StartCurrent`, no crossfade): the DSD stream ends, then the new entry starts after the silence. An operator action always acts; HoldOthers only governs automatic overlaps and other sources.
- **HoldOthers next track.** `plan_for` turns every `StartNextAt` (segue or hard) of a held DSD track into `StopAt { at_secs: end }`. When the engine reports `ReachedEnd`, the model starts what the ordinary plan would have started (`StartCurrent`; for a repeating entry the same entry, not marked played), so the next entry starts only when the DSD track has ended, with no overlap, and its own DSD/PCM decision is taken with the device idle. Every stop rule (Single, stop after current, the entry's "stop after", a fade stop) still stops.
- **Switching to PCM delays what triggered it.** When another source is about to start on a device in DSD mode under `ConvertToPcm`, the engine sends `HoldAll` + DSD silence for the silence time, then flips the bus to PCM; every slot holds during the window, so the DSD track resumes as PCM exactly where it held and the triggering source starts at the end of the window. Native devices are reopened in PCM at the same rate inside the window.
- **HoldOthers muting.** While the bus is in DSD mode, every non-DSD slot on it is consumed silently (its timeline advances, its events and meters stay as usual), so carts end on time and other players keep counting down.
- **Detecting a DSD file.** `AudioFormat` gains `dsd_rate: Option<u32>` (serde default `None`), filled by analysis from `FileDecoder::dsd_rate()`. `ANALYSIS_VERSION` goes from 6 to 7 ("bump when the output format changes", `fp-analysis/src/cache.rs:18`), so existing libraries get the established "analysis is outdated" path; until a DSD track is re-analysed it plays converted, as today.
- **Native DSD capability** is detected on Linux only, for ALSA `hw:` devices, during enumeration (a helper thread): `alsa::PCM::new(name, Playback, nonblock)` + `HwParams::any` + `test_format` for each `NativeDsdFormat` in preference order (`DSD_U32_BE`, `DSD_U32_LE`, `DSD_U16_BE`, `DSD_U16_LE`, `DSD_U8`). A device that is busy (for example opened by this application) keeps the last successful probe (a per-backend cache) and otherwise reports no native DSD. Without hardware the detection is tested through the pure `choose_native_format(accepts)` and the Offline backend's `set_native_dsd`; the ALSA calls themselves are a manual check.
- **O27.** "No output (silent)" is always listed, after the real systems (`listed_backends`). `main.rs` stops filtering `null` out of the candidates, so a configured `null` is really used; `preferred_backend` ranks `null` after every other system, so it is chosen as a fallback only when nothing else is available (as today).
- **Device loss during DSD.** The watchdog reopens the bus with its last `StreamConfig` (DSD included); the renderer's DoP marker phase restarts with the new stream, which DACs accept after their lock time. If the reopen fails the bus stays `Lost` as today.

## Global Constraints

- All code, identifiers, comments, docs, specs, plans and commit messages are in English. Never mention other playout, radio-automation or tag-editor products anywhere.
- Spec §15 (binding; the spec's own words):

> `CLAUDE.md` rules 1–10 apply to every plan. In particular:
>
> - English everywhere, and UI strings in both locales;
> - no product names;
> - operator values are `Config` fields with defaults, ranges and lenient
>   loading;
> - the real-time path never allocates, locks, logs or panics;
> - behaviour lives in `fp-model`;
> - the UI never blocks (tag writing, restart and file work run on helper
>   threads);
> - bad data never crashes;
> - nothing goes on air by itself.
>
> TDD throughout. Each plan ends with a docs task and a review by a fresh
> reviewer.

- Spec §11 values copied verbatim: DoP is "DSD over PCM (DoP 1.1), 24-bit samples at the DSD rate ÷ 16, with the alternating 0x05/0xFA markers"; Native is "offered only on Linux, through ALSA on hardware devices that report a DSD sample format"; "DoP for DSD256 needs 705.6 kHz"; DSD silence is "(0x69)"; `ConvertToPcm` is the default of the mix setting and `Pcm` the default DSD mode; the O34 limiter, if chosen, is "off by default, never on a bit-perfect path".
- CLAUDE.md rule 5: the mixer and every backend render path (`Mixer::render`, `MixerRenderer::render`, `DopEncoder`, `pack_native`, the ALSA DSD write loop's per-period work) never allocate, free, lock (only `try_lock` on the bus mixer), log, do I/O beyond the device write, or panic. Buffers are preallocated when a stream opens. `assert_no_alloc` tests cover the new mixer paths.
- CLAUDE.md rule 6: `unsafe_code` is forbidden (the `alsa` crate is used through its safe API only); no `unwrap`, `expect` or `panic` outside tests (tests carry the `#![allow(...)]` header the neighbouring test files use); `fp-engine`, `fp-backends`, `fp-decode` and `fp-analysis` deny `clippy::indexing_slicing` (use `get`).
- CLAUDE.md rule 9: a truncated, corrupt or multichannel DSD file never crashes and never sends noise: it falls back to PCM or ends with DSD silence.
- CLAUDE.md rule 10: nothing goes on air by itself; a restored track loaded paused stays paused whatever its format.
- UI strings are Fluent messages in `crates/fp-app/locales/en-US/main.ftl` and `es-ES/main.ftl`, always both (`tests/i18n.rs` checks the keys match).
- Engine tests use the `Offline` backend and explicit `now: Instant`, as `crates/fp-engine/tests/bit_perfect.rs` does (its `settle` loop waits for the decode worker, never for audio). Never open the real sound system; never sleep to wait for audio.
- Builds go only into the repo's `target/`.
- Gate for every commit (run from the repo root):

```bash
export PATH=$HOME/.cargo/bin:$PATH; cargo fmt --all --check && cargo clippy --workspace --all-targets -q -- -D warnings && cargo test --workspace -q
scripts/check-commits.sh origin/master   # no output = pass
```

  Run `cargo deny check` after any dependency change (Task 10). Commit subjects are Conventional Commits (scopes `decode`, `backends`, `engine`, `model`, `store`, `analysis`, `app`, `ui`, `docs`) and every commit message ends with the trailer `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>` (the commit commands below show the subject only). `CHANGELOG.md` is never edited by hand.

## What is verified here, and what is the maintainer's manual check

- **Verified by tests:** every rule in `fp_model::dsd` and the reducer (one test per rule); the DoP encoder byte for byte (markers, bit order, sign, 24-bit exactness through `exclusive::write_samples`); native packing for all five ALSA formats byte for byte; the raw DSD reader against the existing DSF/DFF writers; DoP frames end to end through the engine on an Offline device set to `I24` (silence prefix, the file's exact bytes, markers alternating across blocks, silence tail, pause, device loss and replug, HoldOthers muting, the PCM switch); every fallback (busy device, refused rate, 16-bit format, multichannel, volume, native unsupported); the meters reading the PCM conversion; the Settings combo offering only supported modes; O27 listing and fallback ranking.
- **Manual check (the maintainer, on hardware):** a DoP-capable DAC locks without a pop at start, stop and switch (and the silence time is long enough); DSD64/128/256 through DoP on ALSA `hw:`, WASAPI exclusive and Core Audio hog mode; native DSD on a Linux DAC that reports `DSD_U32_BE` (and one with `DSD_U32_LE` if available); the native probe on a busy device; the cpal path choosing an integer format for DoP. Task 12 writes this list into `docs/user/bit-perfect.md` ("Checking it yourself").

## Review Focus

Failure modes the spec implies and no rule test names; each has a test in the task named.

1. **A truncated, corrupt or 5.1 DSD file on a DSD device**: no crash, no noise; 5.1 plays converted, a truncated file ends with DSD silence. (Task 6 `a_truncated_dsf_ends_early_without_error`; Task 9 `a_multichannel_dsd_file_plays_converted`.)
2. **The device refuses what DSD needs** (the 705.6 kHz word rate of DSD256, a 16-bit-only format, a native stream on a device without it): the track plays as PCM, BP off, a log line, and the refusal is remembered so the next DSD track does not reopen the device again. (Task 9 `a_refused_word_rate_plays_converted_and_is_not_asked_again`, `a_sixteen_bit_device_plays_dsd_converted`, `native_on_a_device_without_it_plays_converted`.)
3. **Pause, resume and seek during DSD direct**: no PCM zero ever reaches a DoP stream (every frame keeps a valid marker), so the DAC never drops out of DSD mid-track. (Task 9 `pause_and_seek_keep_every_frame_valid_dop`.)
4. **The device is lost and comes back while DSD plays**: the reopened stream is DoP again, never raw words or PCM. (Task 9 `a_device_lost_during_dop_comes_back_as_dop`.)
5. **Restart with a DSD track loaded paused**: nothing on air; Play then starts DSD direct with the silence prefix. (Task 9 `a_dsd_track_loaded_paused_starts_direct_on_resume`.)

## File Structure

- Create `docs/technical/audio-path-audit.md` (Task 1) and, if useful, `crates/fp-engine/tests/audio_path_audit.rs` (probe tests).
- Create `crates/fp-model/src/dsd.rs` (DSD config types, `dsd_decision`, `DsdFallback`, `bp_badge`, `dsd_holds_others`), `crates/fp-model/tests/dsd_config.rs`, `crates/fp-model/tests/dsd_rules.rs`.
- Create `crates/fp-backends/src/dsd.rs` (pure: `DsdStream`, `NativeDsdFormat`, word/DoP encoding, `DopEncoder`, `pack_native`, `choose_native_format`), `crates/fp-backends/src/alsa_dsd.rs` (Linux: probe and native stream), `crates/fp-backends/tests/dsd.rs`.
- Create `crates/fp-decode/src/dsd/raw.rs` (`DsdRawReader`).
- Create `crates/fp-engine/src/engine/dsd.rs` (engine orchestration of DSD buses), `crates/fp-engine/tests/dsd_mixer.rs`, `crates/fp-engine/tests/dsd_output.rs`.
- Modify `crates/fp-model/src/{config.rs, restart.rs, player.rs, command.rs, reducer.rs, track.rs, lib.rs}`; `crates/fp-store/src/lenient.rs` (tests only); `crates/fp-backends/src/{lib.rs, null.rs, offline.rs, cpal_backend.rs, hosts.rs}`, `crates/fp-backends/Cargo.toml`; `crates/fp-decode/src/{lib.rs, dsd/mod.rs}`; `crates/fp-analysis/src/{analyze.rs, cache.rs}`; `crates/fp-engine/src/{source.rs, worker.rs, mixer.rs, bus.rs, engine.rs, conductor.rs}`, `crates/fp-engine/tests/support/mod.rs`; `crates/fp-app/src/{main.rs, ui/settings.rs, ui/player.rs, ui/view.rs, ui/app.rs}`, both locales, `crates/fp-app/tests/settings.rs`.
- Docs: `docs/user/{bit-perfect.md, settings.md, troubleshooting.md, getting-started.md}`, `docs/technical/{backends.md, audio-engine.md, decoding.md, threading-and-realtime.md, persistence.md, testing.md}`, `README.md`, the spec (§11 "As built", status line, O5 note) and the roadmap (row 10).

---

### Task 1: O26 audit of the audio path (investigation, no product code)

**Files:**
- Create: `docs/technical/audio-path-audit.md`
- Create (optional): `crates/fp-engine/tests/audio_path_audit.rs` — probe tests only
- Modify: `docs/technical/README.md` (one line linking the audit)

**Interfaces:**
- Consumes: the code as it is on the base commit.
- Produces: the findings table the coordinator turns into tasks (see "Audit follow-ups" below). Each row has an id `A1`, `A2`, … that fix tasks and the spec's "As built" cite.

This task changes no product code. Every claim in the document is backed by a test (name and file) or by a line of code (`path:line` on the commit the audit is written against; write the commit hash at the top). Where neither exists and the claim matters, add a **probe test** to `crates/fp-engine/tests/audio_path_audit.rs` (Offline backend, explicit `now`, generated WAVs through `support::indexed_wav` or synthetic `source_pair` rings, never real audio). A probe that shows a defect is committed `#[ignore]`d with a comment `// A<n>: fails until the fix task` so the suite stays green; the fix task removes the `#[ignore]`.

- [ ] **Step 1: Read the path end to end**

Read, in order, with `codegraph_explore`: `fp_decode::FileDecoder` (`crates/fp-decode/src/lib.rs`), the DSD, WavPack, APE and symphonia backends; `crates/fp-engine/src/worker.rs` (`file_opener`, `FileSource`, `step`, `run`, the failure path); `crates/fp-engine/src/resample.rs`; `crates/fp-engine/src/source.rs`; `crates/fp-engine/src/mixer.rs` (`Mixer::render`, `render_slot`, `apply`, `MixerRenderer`); `crates/fp-engine/src/ramp.rs`; `crates/fp-engine/src/bus.rs` (`Bus::open`, `try_open`, `reopen_at`, the watchdog, `VirtualClock`); `crates/fp-engine/src/engine.rs` (`start_current`, `fade_out`, `stop_quick_and_release`, `dispatch`, `dispatch_plans`, `prepare_start`, the starts loop near "for (bus, slot, fade_in, earliest) in starts", `is_bit_perfect`, `take_meter_input`); `crates/fp-engine/src/meter.rs`; `crates/fp-backends/src/cpal_backend.rs` (`render_converted`, `choose_sample_format`, the stream error callback and `classify`), `exclusive.rs` (`write_samples`), `wasapi_exclusive.rs`, `coreaudio_hog.rs`, `null.rs`.

- [ ] **Step 2: Write one section per spec bullet**

`docs/technical/audio-path-audit.md` has this outline (each section: what the code does, the evidence, the verdict):

```markdown
# Audio path audit (feedback 2, O26)

Written against commit <hash>. Each claim cites a test or a line of code.

## 1. Decoding
### 1.1 Errors in the middle of a file
### 1.2 Gapless starts and ends
### 1.3 Seeking
### 1.4 Decode-ahead margin against slow disks
## 2. Resampling
### 2.1 Quality and passband
### 2.2 Switching between resampled and bit-perfect playback
## 3. Mixer
### 3.1 Summing, headroom and clipping
### 3.2 Gain ramps: start, stop, pause, resume, seek, fade, fader move
## 4. Output
### 4.1 f32 to integer conversion
### 4.2 Dither when the device is not bit-perfect and narrower than the source
## 5. Buffers and devices
### 5.1 Buffer sizes against underruns
### 5.2 Xrun counting and reporting
### 5.3 Device loss and recovery
### 5.4 Rate changes on bit-perfect devices
## 6. Real-time safety of the callback (CLAUDE.md rule 5)
## 7. Readings above 0 dBFS (O34)
## 8. Findings
```

What each section must answer (look for these; do not assume the answer):

- **1.1** What a decoder error mid-file does: does the worker mark the source `failed`, play out what is buffered (`SourceShared::failed`, `render_slot`'s `ended` check), report `SourceFailed` once, and does the model skip the entry? Is there a click at the cut (the last buffered sample to silence, with no ramp)? A probe: a WAV truncated in the middle of a block.
- **1.2** Gapless: `crates/fp-decode/tests/gapless.rs`, the Opus pre-skip (`tests/opus.rs`), the resampler's delay removal (`StreamResampler::new` `output_delay`), and the hard `StartNextAt` cut (a repeat pass: the old pass "gets the usual de-click ramp at the cut", `docs/technical/audio-engine.md`) — is there a sample-exact join with no gap and no overlap, and is the de-click ramp audible as a dip?
- **1.3** Seeking replaces the source (`EngineAction::Seek`): does the old source get a de-click ramp out and the new one a ramp in (`StartState::WhenReady { fade_in: true }`)? Is the first frame exact (`aligned_preroll` in `file_opener`)?
- **1.4** `tuning.prebuffer_secs` (5 s) and `ready_threshold_ms` (500 ms): what happens when the disk stalls longer than the ring (underrun counted, silence, timeline keeps moving)? Is an underrun reported to the operator, and how?
- **2.1** rubato parameters (`SincInterpolationParameters::default()`): sinc length, cutoff, window, oversampling; measure the passband ripple and stopband with a probe (sweep or multi-tone through `StreamResampler`, 44.1 → 48 kHz) and state the numbers.
- **2.2** A source that starts on a bit-perfect bus at the file rate versus one that starts while the bus is busy (resampled for its whole length): is the change audible only between tracks? Is the BP badge right in both (`tests/bit_perfect.rs`)?
- **3.1** The mixer sums in `f32` with no clamp (`render_slot` `*o += l`); the volume is clamped to 0..=1 (`slot.volume.load().clamp(0.0, 1.0)`; `Command::SetVolume` clamps too): can any gain exceed unity? Where does clipping happen (the integer conversion, saturating `from_sample`)? Is `f32` headroom enough for any realistic sum?
- **3.2** For every start, stop, pause, resume, seek, fade, fader move and transition: which ramp applies (`declick_ms`, `pause_ramp_ms`, `gain_smoothing_ms`, equal-power fades), and is there any path where the level steps (a source cut without ramp: the end of a `StopAt` at the cue-out, a failed source draining, a slot `Detach`ed while audible, a `Cancel` after a start)? A probe per suspicious path: render a full-scale DC or 1 kHz source and assert the largest sample-to-sample step stays below a ramp's step.
- **4.1** `render_converted` / `write_samples`: exact for integer PCM (cite `conversion_to_i16_is_exact_for_16_bit_pcm` etc.); what happens to values above 1.0 (saturation) and to `NaN` (does `from_sample` map it to 0?).
- **4.2** Is there any dither when a 24-bit or float source plays on a 16-bit device (or after gain, fades or resampling on any integer device)? If not, quantify the truncation/rounding error and say whether it is a defect for this product (it is audible only at very low levels; the spec asks to check).
- **5.1–5.4** Buffer size handling (`StreamConfig::buffer_frames`, cpal fixed vs default, WASAPI period alignment); `StreamErrorKind::Xrun` → `BusShared::xruns` → how the conductor logs and the UI shows it; `tuning.watchdog_timeout_ms`, the virtual clock and reconnection (`tests/bus.rs`); `reopen_at` and refused rates.
- **6** List every function on the callback (`MixerRenderer::render`, `Mixer::render`, `render_slot`, `mark_unaltered`, `finish`, the cpal callback closure and `render_converted`, `wasapi_exclusive`'s render loop, `coreaudio_hog`, `NullStream`'s thread) and check each against rule 5 (allocation, free, locks, logging, I/O, panics, indexing). Cite the `assert_no_alloc` test (`crates/fp-engine/tests/mixer.rs`) and say which paths it does not cover.
- **7 (O34)** Explain when the digital peak meter reads above 0 dBFS. Check, with code lines and a probe each: a file whose samples already exceed full scale (float WAV) or a lossy file decoding above 1.0; inter-sample peaks after resampling; true-peak mode (`truepeak.rs`) reading above the sample peak; several sources on one output (the meter is per player: `take_meter_input` measures each player's own sources after its gain, *not* the device sum — confirm or refute); and whether any gain or fader can be above unity (3.1). Then confirm what the device receives (the integer conversion saturates; quote `write_samples`/`render_converted`) and how the meter shows it (`widgets::max_readout`, the scale end, the `zone_of` colours). End with the proposal, for the maintainer to choose, none of which is built in this task:
  - **(a)** a clear over indicator: a latched "OVER" mark on the meter when any sample of the player's output exceeded 0 dBFS (or the device sum clipped), cleared with the max readout; or
  - **(b)** an optional output limiter per bus: a look-ahead true-peak limiter at the end of the mix, `Config` field off by default, never active on a bit-perfect bus or a DSD stream; or
  - **(c)** no change (the readings are correct and documented).
  Give for (a) and (b) the code it would touch and its cost on the real-time thread.
- **8 Findings** — a table, one row per finding:

```markdown
| Id | Area | Finding | Evidence | Verdict | Proposed action |
|---|---|---|---|---|---|
| A1 | 3.2 Mixer ramps | … | `crates/fp-engine/src/mixer.rs:…`, probe `…` | confirmed defect | fix in this plan: … |
| A2 | 4.2 Dither | … | … | too large | roadmap item: … |
| A3 | 1.2 Gapless | … | `crates/fp-decode/tests/gapless.rs` | no change needed | — |
```

  Verdicts are exactly one of **confirmed defect** (fixed in this plan, test-first), **no change needed**, or **too large** (becomes its own roadmap item; say what it would take). The O34 row's verdict is **maintainer's choice** with options (a)/(b)/(c).

- [ ] **Step 3: Run the probes and the gate**

Run: `export PATH=$HOME/.cargo/bin:$PATH; cargo test -p fp-engine --test audio_path_audit -- --include-ignored` (to see the defects fail) and then the full gate (ignored probes stay ignored).
Expected: the gate passes; each confirmed defect has a failing, ignored probe or an exact code citation.

- [ ] **Step 4: Commit**

```bash
git add docs/technical/audio-path-audit.md docs/technical/README.md crates/fp-engine/tests/audio_path_audit.rs
git commit -m "docs: audio path audit (O26)"
```

---

## Audit follow-ups (inserted by the coordinator after Task 1)

This plan does not guess the audit's results. After Task 1 is reviewed:

1. **For each "confirmed defect" row**, the coordinator writes a task `Task 1.<n>: fix A<n> — <title>` and inserts it here, before Task 2, in the same format as the other tasks (Files / Interfaces / failing test first / minimal fix / gate / commit `fix(<scope>): … (A<n>)`). The failing test is the audit's probe with its `#[ignore]` removed, or a new focused test. Fixes that change `Mixer::render` keep the `assert_no_alloc` coverage. Record each inserted task in the ledger as `Ruling: added Task 1.<n> for A<n> — confirmed by the audit — <cost if wrong>`.
2. **For each "too large" row**, the coordinator adds a line to the roadmap's "Notes for the later plans" (a new item, no plan number yet) and does nothing else.
3. **O34**: the coordinator puts the audit's §7 proposal to the maintainer and waits for the choice. Only then, if the choice is (a) or (b), a task `Task 1.O34` is written and inserted after the defect fixes; (b) adds a `Config` field (off by default, range, lenient loading) and must be bypassed on every bit-perfect bus and every DSD stream. If the choice is (c), the audit's text goes into the user guide in Task 12.
4. A defect that touches the same code as Tasks 7–9 (the mixer or the engine's start/stop paths) is fixed first; those tasks then build on the fixed code, and their implementers are told which `A<n>` changed what.

Tasks that may grow because of the audit: Task 8 (mixer) and Task 9 (engine) if the audit changes ramps, starts or stops; Task 12 (docs) for the O34 text.

### Inserted after the audit (coordinator, 2026-10-03)

The audit (`docs/technical/audio-path-audit.md`, commit 68657a9) confirmed A1–A8. The maintainer chose **(c) no change** for O34 (A9): no `Task 1.O34`; Task 12 puts the §7 explanation into the user guide's meters page. A10 (dither) is "too large" and goes to the roadmap notes. Each task below starts from the audit's probe for its finding in `crates/fp-engine/tests/audio_path_audit.rs` or `crates/fp-backends/tests/audio_path_audit.rs`: remove its `#[ignore]`, watch it fail, fix, and keep every other test green. The audit section named in each task gives the evidence (file:line) and the proposed action, which is binding unless the code proves it wrong (then say so in the report). Each task updates the audit's findings row (verdict "fixed in <commit>") and any doc the fix makes true or false.

#### Task 1.1: fix A1 — a start inside the audio ramps in

- **Files:** `crates/fp-engine/src/mixer.rs` (start of a slot; `Ramp`), `crates/fp-engine/src/engine.rs` (start paths: Play, crossfade, segue, hard transition, CUE), `crates/fp-engine/src/engine/carts.rs` if carts start elsewhere; test `crates/fp-engine/tests/audio_path_audit.rs` `a1_a_start_inside_the_file_ramps_in`, plus one test per start kind not covered by the probe (CUE, cart, the next source of a segue started at a cue-in > 0).
- **Rule (audit §3.2, row A1):** every source that starts at `from_secs > 0` ramps in over `declick_ms` (the existing config value); a start at the first frame of the file stays hard (the file's own start and gapless joins). Existing segue/transition tests that start at 0 stay unchanged. Real-time safe: the ramp is state in the slot, no allocation (keep the `assert_no_alloc` coverage).
- **Commit:** `fix(engine): ramp in a source that starts inside the audio (A1)`.

#### Task 1.2: fix A2 — a gapless join keeps the level

- **Files:** `crates/fp-engine/src/mixer.rs` and/or `crates/fp-engine/src/engine.rs` (hard transition), `crates/fp-engine/src/worker.rs` (eof and pushed-frame count if the mixer needs it); test `a2_a_gapless_join_at_the_end_of_the_file_keeps_the_level`, plus a test that a hard cut inside the file still ramps out.
- **Rule (audit §1.2, row A2):** no de-click ramp on the outgoing source when the transition frame is its end of stream (the worker has set `eof` and the frame is at or past the last pushed frame, tolerating a one-frame error in the analysed duration); keep the ramp for cuts inside the file. Builds on Task 1.1: a gapless join's incoming source starts at frame 0, so it stays hard.
- **Commit:** `fix(engine): no de-click dip at a gapless join (A2)`.

#### Task 1.3: fix A3, A4, A5 — sources end without a step

- **Files:** `crates/fp-engine/src/worker.rs` (cart pass ending at `until_secs`), `crates/fp-engine/src/sources.rs` / `LoadOptions` (frame count), `crates/fp-engine/src/mixer.rs` (failed source ramp, stop on a pausing slot), `crates/fp-engine/src/engine.rs` and `engine/carts.rs` (`fade_out` treats `Requested` like `Started`); tests `a3_a_cart_ends_at_its_cue_out_without_a_step`, `a4_a_source_that_fails_mid_file_ends_without_a_step`, `a5_a_stop_during_the_pause_ramp_does_not_step`, `a5_a_stop_right_after_the_start_does_not_step`.
- **Rules (audit rows A3–A5):** A3 — the worker fades the last `declick_ms` before `until_secs` linearly to zero on a pass that ends there and does not loop (looped carts keep their splice, A15). A4 — when a source is failed and its ring holds no more than the de-click length, the mixer ramps it to zero over the frames that remain (real-time safe); a truncated file ends as a normal end (A16, not covered). A5 — `fade_out` treats `Requested` like `Started`, and a stop on a slot whose pause ramp may still run is a ramped stop, not a detach.
- **Commit:** `fix(engine): end carts, failed sources and early stops without a step (A3, A4, A5)`.

#### Task 1.4: fix A6 — non-finite samples never reach the device

- **Files:** `crates/fp-decode/src/lib.rs` (`FileDecoder` output, off the real-time thread); test `a6_non_finite_samples_never_reach_the_device` plus a focused fp-decode test (a float WAV with NaN and ±inf decodes to 0 there, and the log line appears once per file).
- **Rule (audit §1.1, row A6):** `FileDecoder` replaces NaN and ±inf by 0 and logs once per file (CLAUDE.md rule 9). The loudness reading of such a file recovers (the K-weighting no longer sees NaN).
- **Commit:** `fix(decode): replace non-finite samples with silence (A6)`.

#### Task 1.5: fix A7 — cpal integer conversion clips and rounds

- **Files:** `crates/fp-backends/src/cpal_backend.rs` (`render_converted`); tests `a7_the_24_bit_conversion_clips_instead_of_wrapping`, `a7_the_16_bit_conversion_rounds_to_the_nearest_step`.
- **Rule (audit §4.1, row A7):** convert with our own function: clamp to full scale, NaN to 0, round to the nearest step; exact integer PCM stays exact (existing exactness and bit-perfect tests unchanged). Real-time safe (no allocation; `fp-backends` denies `indexing_slicing`).
- **Commit:** `fix(backends): clip and round the integer conversion (A7)`.

#### Task 1.6: fix A8 — xruns and underruns are reported

- **Files:** `crates/fp-engine/src/conductor.rs` (counter increases → rate-limited log lines, off the real-time thread), the telemetry the UI reads, `crates/fp-app/src/ui/app.rs` status bar (`alert-underruns` per player — the key exists in both locales — and an xrun alert per device; add its message in both locales), `crates/fp-backends` stream error mapping (`StreamErrorKind::Other` logged, not counted as an xrun), `docs/technical/threading-and-realtime.md:65-67` made true; tests: conductor test that an increase logs once per window, kittest that the alerts show, a backends test for the `Other` mapping.
- **Rule (audit §5.2, row A8):** every counted event (xrun, underrun, lock miss, leak, dropped event, misrouted block) is logged when it increases, rate-limited; underruns and xruns are shown to the operator. Rate limit window is a `Config` field only if an operator would change it; otherwise a documented constant in the conductor (state which in the report).
- **Commit:** `fix(engine): report xruns and underruns (A8)`.

---

### Task 2: O27 — "No output (silent)" listed after the real systems

**Files:**
- Modify: `crates/fp-app/src/ui/settings.rs` (`listed_backends` near line 663 and its unit test `null_is_listed_only_when_configured` near line 1802)
- Modify: `crates/fp-app/src/main.rs` (the `listed` filter near line 167)
- Modify: `crates/fp-backends/src/hosts.rs` (`preferred_backend`)
- Test: `crates/fp-app/tests/settings.rs` (replace `the_null_backend_is_not_offered`), `crates/fp-backends/tests/hosts.rs`
- Docs: `docs/user/settings.md` (Audio system row), `docs/technical/backends.md` (Null row), the spec's O5 bullet gets "(reversed by O27: always listed)" — no other spec edit until Task 13

**Interfaces:**
- Produces: `listed_backends(all, configured)` returns every backend, `null` last; `preferred_backend` never prefers `null` while another backend is available.

- [ ] **Step 1: Write the failing tests**

In `crates/fp-app/src/ui/settings.rs`, replace the test `null_is_listed_only_when_configured` with:

```rust
    #[test]
    fn null_is_always_listed_after_the_real_systems() {
        let all = [choice("null"), choice("alsa"), choice("jack")];
        let ids = |configured| -> Vec<String> {
            listed_backends(&all, configured)
                .into_iter()
                .map(|b| b.id.clone())
                .collect()
        };
        assert_eq!(ids(None), vec!["alsa", "jack", "null"]);
        assert_eq!(ids(Some("alsa")), vec!["alsa", "jack", "null"]);
        assert_eq!(ids(Some("null")), vec!["alsa", "jack", "null"]);
    }
```

In `crates/fp-app/tests/settings.rs`, replace `the_null_backend_is_not_offered` with:

```rust
#[test]
fn the_null_backend_is_offered_as_no_output() {
    let mut h = outputs_with_null(Some("offline"));
    h.get_by_value("Offline").click();
    h.run_steps(2);
    assert!(h.query_by_label("No output (silent)").is_some());
    assert!(h.query_by_label("Null").is_none(), "never by its internal name");
}
```

In `crates/fp-backends/tests/hosts.rs` add:

```rust
#[test]
fn null_is_chosen_only_when_nothing_else_is_available() {
    use fp_backends::{choose_default_backend, preferred_backend};
    let all = [("null", true), ("alsa", true)];
    assert_eq!(preferred_backend(&all, "linux"), Some("alsa"));
    assert_eq!(preferred_backend(&all, "plan9"), Some("alsa"), "unknown OS too");
    assert_eq!(choose_default_backend(Some("null"), &all, "linux"), Some("null"));
    assert_eq!(preferred_backend(&[("null", true), ("alsa", false)], "linux"), Some("null"));
}
```

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p fp-app --lib null_is_always_listed && cargo test -p fp-app --test settings the_null_backend_is_offered && cargo test -p fp-backends --test hosts null_is_chosen`
Expected: FAIL (null filtered out; order; "plan9" picks null because it comes first).

- [ ] **Step 3: Implement**

`listed_backends` in `settings.rs`:

```rust
/// The audio systems the list offers: the real systems in their order, then
/// `null` as "No output (silent)" (feedback 2 spec O27, which reverses the
/// hiding part of O5).
fn listed_backends<'a>(
    all: &'a [BackendChoice],
    _configured: Option<&str>,
) -> Vec<&'a BackendChoice> {
    all.iter()
        .filter(|b| b.id != "null")
        .chain(all.iter().filter(|b| b.id == "null"))
        .collect()
}
```

Keep the parameter (unused, prefixed `_`) so the call site and the unit test stay as they are.

`main.rs`: replace the `listed` block by passing every backend, so a configured `null` is honoured:

```rust
    let listed: Vec<(&str, bool)> = availability
        .iter()
        .map(|(id, ok)| (id.as_str(), *ok))
        .collect();
```

`hosts.rs` `preferred_backend`: rank `null` after everything, known or not:

```rust
    let rank = |id: &str| {
        if id == "null" {
            order.len() + 1
        } else {
            order.iter().position(|p| *p == id).unwrap_or(order.len())
        }
    };
```

- [ ] **Step 4: Run the tests and the gate**

Expected: PASS. Also run `cargo test -p fp-app --test settings a_configured_null_backend_shows_as_no_output` (still passes).

- [ ] **Step 5: Docs**

`docs/user/settings.md`, Audio system row: replace the first sentence with "The last choice, **No output (silent)**, plays nothing: timelines run at real-time pace with no sound card (for a machine without one, or to rehearse)." `docs/technical/backends.md`: under the Implemented table add "The Settings list shows `null` last, as "No output (silent)"; `preferred_backend` ranks it after every other system, so it is the default only when nothing else is available." Spec §3 O5: append "(O27 reverses the hiding: it is always listed, last.)".

- [ ] **Step 6: Commit**

```bash
git add crates/fp-app/src/ui/settings.rs crates/fp-app/src/main.rs crates/fp-backends/src/hosts.rs crates/fp-app/tests/settings.rs crates/fp-backends/tests/hosts.rs docs/user/settings.md docs/technical/backends.md docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md
git commit -m "feat(app): list No output (silent) after the real audio systems (O27)"
```

---

### Task 3: DSD configuration fields

**Files:**
- Create: `crates/fp-model/src/dsd.rs` (config types only in this task; Task 4 adds the rules)
- Modify: `crates/fp-model/src/config.rs` (`OutputsConfig`, its `Default`, `Config::validate`), `crates/fp-model/src/restart.rs` (`RestartReason::DsdOutput`), `crates/fp-model/src/lib.rs` (re-exports)
- Modify: `crates/fp-app/src/ui/app.rs` (`RestartReason::DsdOutput => "restart-reason-dsd"` next to `restart-reason-bit-perfect`), both locales (`restart-reason-dsd = DSD output` / `restart-reason-dsd = salida DSD`)
- Test: `crates/fp-model/tests/dsd_config.rs`, `crates/fp-model/tests/restart.rs` (one case), `crates/fp-store/src/lenient.rs` (unit test)

**Interfaces:**
- Produces (used by Tasks 4, 9, 11):

```rust
// crates/fp-model/src/dsd.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum DsdOutput { #[default] Pcm, Dop, Native }
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum DsdMix { #[default] ConvertToPcm, HoldOthers }
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DsdDevice { pub backend: String, pub device: String, pub mode: DsdOutput }
pub const DEFAULT_DSD_SILENCE_MS: f64 = 200.0;
// crates/fp-model/src/config.rs, OutputsConfig gains:
pub dsd_output: Vec<DsdDevice>,   // #[serde(default)]
pub dsd_mix: DsdMix,              // #[serde(default)]
pub dsd_silence_ms: f64,          // default DEFAULT_DSD_SILENCE_MS, 0..=2000
impl OutputsConfig { pub fn dsd_output_for(&self, backend: &str, device: &str) -> DsdOutput }
// crates/fp-model/src/restart.rs
RestartReason::DsdOutput   // after BitPerfect
```

- [ ] **Step 1: Write the failing tests**

Create `crates/fp-model/tests/dsd_config.rs`:

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback 2 spec O25: the DSD settings are `Config` fields with defaults,
//! ranges and lenient loading.

use fp_model::{Config, DsdDevice, DsdMix, DsdOutput, OutputDevice};

fn device(mode: DsdOutput) -> DsdDevice {
    DsdDevice {
        backend: "alsa".into(),
        device: "hw:CARD=D,DEV=0".into(),
        mode,
    }
}

fn bit_perfect(c: &mut Config) {
    c.outputs.bit_perfect.push(OutputDevice {
        backend: "alsa".into(),
        device: "hw:CARD=D,DEV=0".into(),
    });
}

#[test]
fn defaults_convert_dsd_to_pcm() {
    let c = Config::default();
    assert!(c.outputs.dsd_output.is_empty());
    assert_eq!(c.outputs.dsd_mix, DsdMix::ConvertToPcm);
    assert_eq!(c.outputs.dsd_silence_ms, 200.0);
    assert_eq!(c.outputs.dsd_output_for("alsa", "hw:CARD=D,DEV=0"), DsdOutput::Pcm);
}

#[test]
fn a_mode_applies_only_to_a_bit_perfect_device() {
    let mut c = Config::default();
    c.outputs.dsd_output.push(device(DsdOutput::Dop));
    assert_eq!(
        c.outputs.dsd_output_for("alsa", "hw:CARD=D,DEV=0"),
        DsdOutput::Pcm,
        "not bit-perfect: converted"
    );
    bit_perfect(&mut c);
    assert_eq!(c.outputs.dsd_output_for("alsa", "hw:CARD=D,DEV=0"), DsdOutput::Dop);
    assert_eq!(c.outputs.dsd_output_for("alsa", "hw:CARD=E,DEV=0"), DsdOutput::Pcm);
}

#[test]
fn the_silence_time_is_clamped() {
    for (given, kept) in [(-5.0, 0.0), (f64::NAN, 0.0), (90_000.0, 2000.0), (150.0, 150.0)] {
        let mut c = Config::default();
        c.outputs.dsd_silence_ms = given;
        let warnings = c.validate();
        assert_eq!(c.outputs.dsd_silence_ms, kept, "{given}");
        assert_eq!(warnings.is_empty(), given == kept, "{given}: {warnings:?}");
    }
}

#[test]
fn a_device_listed_twice_keeps_its_first_mode() {
    let mut c = Config::default();
    c.outputs.dsd_output = vec![device(DsdOutput::Native), device(DsdOutput::Dop)];
    let warnings = c.validate();
    assert_eq!(c.outputs.dsd_output, vec![device(DsdOutput::Native)]);
    assert_eq!(warnings.len(), 1, "{warnings:?}");
}

#[test]
fn the_modes_serialize_by_name() {
    let json = serde_json::to_string(&device(DsdOutput::Dop)).unwrap();
    assert!(json.contains("\"Dop\""), "{json}");
    assert_eq!(serde_json::to_string(&DsdMix::HoldOthers).unwrap(), "\"HoldOthers\"");
}
```

(If `fp-model` has no `serde_json` dev-dependency, check `crates/fp-model/Cargo.toml`; add it under `[dev-dependencies]` with `serde_json.workspace = true` — a dev-only addition, no `cargo deny` needed for a workspace crate already in the lock.)

In `crates/fp-model/tests/restart.rs`, add a case next to the bit-perfect one (follow the file's helper style; read it first):

```rust
#[test]
fn dsd_settings_apply_at_restart() {
    let started = Config::default();
    for change in [
        |c: &mut Config| c.outputs.dsd_mix = fp_model::DsdMix::HoldOthers,
        |c: &mut Config| c.outputs.dsd_silence_ms = 500.0,
        |c: &mut Config| {
            c.outputs.dsd_output.push(fp_model::DsdDevice {
                backend: "alsa".into(),
                device: "hw:0".into(),
                mode: fp_model::DsdOutput::Dop,
            })
        },
    ] {
        let mut current = started.clone();
        change(&mut current);
        assert_eq!(restart_pending(&started, &current), vec![RestartReason::DsdOutput]);
    }
}
```

In `crates/fp-store/src/lenient.rs` `mod tests`, add:

```rust
    #[test]
    fn bad_dsd_values_fall_back_one_by_one() {
        let user: serde_json::Value = serde_json::from_str(
            r#"{"outputs":{"dsd_mix":"Sometimes","dsd_silence_ms":"long","sample_rate":44100,
                "dsd_output":[{"backend":"alsa","device":"hw:0","mode":"Dop"},
                              {"backend":"alsa","device":"hw:1","mode":"Laser"}]}}"#,
        )
        .unwrap();
        let mut warnings = Vec::new();
        let c = config_from_value(&user, &mut warnings);
        assert_eq!(c.outputs.dsd_mix, fp_model::DsdMix::ConvertToPcm);
        assert_eq!(c.outputs.dsd_silence_ms, 200.0);
        assert_eq!(c.outputs.sample_rate, 44_100, "the other fields are kept");
        assert_eq!(c.outputs.dsd_output.len(), 1, "the valid element is kept");
        assert_eq!(warnings.len(), 3, "{warnings:?}");
    }
```

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p fp-model --test dsd_config && cargo test -p fp-model --test restart dsd_settings && cargo test -p fp-store --lib bad_dsd_values`
Expected: FAIL to compile (`DsdDevice`, `dsd_output` not defined).

- [ ] **Step 3: Implement**

`crates/fp-model/src/dsd.rs`:

```rust
//! DSD output (feedback 2 spec O25): the settings, and (Task 4) the pure
//! rules that decide when DSD reaches a device unchanged.

use serde::{Deserialize, Serialize};

/// What a bit-perfect device receives from a DSD track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum DsdOutput {
    /// Converted to PCM, as for any other device.
    #[default]
    Pcm,
    /// DSD over PCM (DoP 1.1): 24-bit samples at the DSD rate ÷ 16, with the
    /// alternating 0x05/0xFA markers.
    Dop,
    /// Raw DSD; only on Linux, through ALSA, on hardware devices that report
    /// a DSD sample format.
    Native,
}

/// What happens when another source needs an output that carries DSD.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum DsdMix {
    /// The DSD track goes on as PCM from that moment, mixed as usual.
    #[default]
    ConvertToPcm,
    /// Nothing interrupts the DSD stream: the player's next track waits for
    /// its end, and other sources are muted on that output meanwhile.
    HoldOthers,
}

/// The DSD mode of one output device.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DsdDevice {
    pub backend: String,
    pub device: String,
    pub mode: DsdOutput,
}

/// Default DSD silence sent at a DSD stream's start, end and switch to PCM.
pub const DEFAULT_DSD_SILENCE_MS: f64 = 200.0;
```

`config.rs` `OutputsConfig` (after `bit_perfect`):

```rust
    /// DSD mode per device (feedback 2 spec O25). Only devices also listed
    /// in `bit_perfect` use it; every other device converts DSD to PCM.
    #[serde(default)]
    pub dsd_output: Vec<crate::dsd::DsdDevice>,
    /// What happens when another source needs an output carrying DSD.
    #[serde(default)]
    pub dsd_mix: crate::dsd::DsdMix,
    /// DSD silence (0x69) sent at a DSD stream's start, at its end and on
    /// every switch to PCM, so that the converter locks without a pop
    /// (milliseconds, 0–2000).
    pub dsd_silence_ms: f64,
```

with `Default` values `Vec::new()`, `DsdMix::default()`, `crate::dsd::DEFAULT_DSD_SILENCE_MS`, and:

```rust
impl OutputsConfig {
    /// The DSD mode of a device: its configured mode when it is bit-perfect,
    /// `Pcm` otherwise.
    pub fn dsd_output_for(&self, backend: &str, device: &str) -> crate::dsd::DsdOutput {
        let bit_perfect = self
            .bit_perfect
            .iter()
            .any(|d| d.backend == backend && d.device == device);
        if !bit_perfect {
            return crate::dsd::DsdOutput::Pcm;
        }
        self.dsd_output
            .iter()
            .find(|d| d.backend == backend && d.device == device)
            .map_or(crate::dsd::DsdOutput::Pcm, |d| d.mode)
    }
}
```

In `Config::validate`, next to the other `outputs` checks (find where `outputs.sample_rate` is clamped):

```rust
        clamp_to(
            &mut self.outputs.dsd_silence_ms,
            0.0,
            2000.0,
            "outputs.dsd_silence_ms",
            &mut w,
        );
        let mut seen: Vec<(String, String)> = Vec::new();
        let before = self.outputs.dsd_output.len();
        self.outputs.dsd_output.retain(|d| {
            let key = (d.backend.clone(), d.device.clone());
            let first = !seen.contains(&key);
            seen.push(key);
            first
        });
        if self.outputs.dsd_output.len() != before {
            w.push(ConfigWarning {
                field: "outputs.dsd_output",
                message: "a device was listed more than once; keeping its first mode".to_owned(),
            });
        }
```

`restart.rs`: add `DsdOutput` after `BitPerfect` in `RestartReason` (doc comment "A device's DSD mode, the DSD mix or the DSD silence time."), and in `restart_pending` after the bit-perfect check:

```rust
    let dsd = |o: &OutputsConfig| {
        (
            o.dsd_output.iter().cloned().collect::<HashSet<_>>(),
            o.dsd_mix,
            o.dsd_silence_ms.to_bits(),
        )
    };
    if dsd(a) != dsd(b) {
        reasons.push(RestartReason::DsdOutput);
    }
```

`lib.rs`: `pub mod dsd;` (or `mod dsd;` matching the crate's style) and re-export `DsdDevice, DsdMix, DsdOutput, DEFAULT_DSD_SILENCE_MS`. `ui/app.rs`: add the `RestartReason::DsdOutput` arm; locales: the two strings above.

- [ ] **Step 4: Run the tests and the gate**

Expected: PASS. The store round-trip test (`crates/fp-store/tests/store_roundtrip.rs`) must still pass unchanged.

- [ ] **Step 5: Commit**

```bash
git add crates/fp-model crates/fp-store/src/lenient.rs crates/fp-app/src/ui/app.rs crates/fp-app/locales
git commit -m "feat(model): DSD output settings per bit-perfect device (O25)"
```

---

### Task 4: DSD rules in the model

**Files:**
- Modify: `crates/fp-model/src/dsd.rs` (rules), `crates/fp-model/src/player.rs` (`PlayerState::dsd`), `crates/fp-model/src/command.rs` (`EngineEvent::DsdStarted`/`DsdEnded`, `EngineAction::LeaveDsd`), `crates/fp-model/src/track.rs` (`AudioFormat::dsd_rate`), `crates/fp-model/src/reducer.rs` (`plan_for`, `on_event`, `play`, `previous`, every `Crossfade` push, fade stop, `SetVolume`, `advance_to`, `stop_player`), `crates/fp-model/src/lib.rs`
- Modify every `AudioFormat { … }` literal (16 sites; `grep -rn "AudioFormat {" crates`) to add `dsd_rate: None` (or `..` where it reads better)
- Modify `crates/fp-engine/src/engine.rs` minimally so it compiles: `EngineAction::LeaveDsd { .. } => {}` (Task 9 implements it)
- Test: `crates/fp-model/tests/dsd_rules.rs`

**Interfaces:**
- Consumes: Task 3's `DsdOutput`, `DsdMix`.
- Produces (used by Tasks 6, 9, 11):

```rust
// fp_model::dsd
pub struct DsdFacts { pub mode: DsdOutput, pub format: Option<AudioFormat>, pub volume: f32, pub device_idle: bool }
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum DsdStreamMode { Dop, Native }
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub struct DsdTarget { pub mode: DsdStreamMode, pub dsd_rate: u32, pub word_rate: u32 }
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum DsdFallback { ModeIsPcm, NotDsd, Multichannel, VolumeNotUnity, DeviceBusy, RateRefused(u32), FormatTooNarrow, StreamRefused }
impl std::fmt::Display for DsdFallback
impl DsdFallback { pub fn worth_logging(self) -> bool }   // false for ModeIsPcm and NotDsd
pub fn dsd_decision(facts: &DsdFacts) -> Result<DsdTarget, DsdFallback>
pub const DSD_WORD_BITS: u32 = 16;
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub struct DsdOnAir { pub entry: EntryId, pub hold_others: bool }
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum BpBadge { Off, Pcm, Dsd }
pub fn bp_badge(bit_perfect: bool, dsd: bool) -> BpBadge
pub fn dsd_holds_others(state: &AppState, player: PlayerId) -> bool
// PlayerState
pub dsd: Option<DsdOnAir>
// AudioFormat
#[serde(default)] pub dsd_rate: Option<u32>
// EngineEvent
DsdStarted { player: PlayerId, entry: EntryId, hold_others: bool },
DsdEnded { player: PlayerId, entry: EntryId },
// EngineAction
LeaveDsd { player: PlayerId },
```

The rules, one test each (names start with the rule id):

| Id | Rule |
|---|---|
| D1 | `dsd_decision`: mode `Pcm` → `ModeIsPcm` |
| D2 | not a DSD file (`format` `None` or `dsd_rate` `None`) → `NotDsd` |
| D3 | more than two channels → `Multichannel` (0 channels = unknown counts as stereo) |
| D4 | volume ≠ 1.0 → `VolumeNotUnity` |
| D5 | device not idle → `DeviceBusy` |
| D6 | otherwise `DsdTarget { mode, dsd_rate, word_rate: dsd_rate / 16 }` (DSD64 → 176 400, DSD256 → 705 600) |
| D7 | `bp_badge`: DSD wins over BP; off when neither |
| D8 | `DsdStarted` for the current, playing entry sets `player.dsd`; for another entry or a stopped player it is ignored |
| D9 | `DsdEnded` for the entry on air clears it; for another entry it is ignored; advancing or stopping clears it |
| D10 | HoldOthers: `plan_for` gives `StopAt { end }` where the ordinary plan gives `StartNextAt` (segue or hard) |
| D11 | HoldOthers: `ReachedEnd` starts the next entry (`StartCurrent`, played mark, history) instead of stopping |
| D12 | HoldOthers: `ReachedEnd` of a repeating entry starts the same entry again, not marked played |
| D13 | HoldOthers: Single mode, stop after current and the entry's "stop after" still stop at `ReachedEnd` |
| D14 | HoldOthers: Play while playing (the next entry) and Previous give `StartCurrent`, never `Crossfade`, and do not set `fading` |
| D15 | ConvertToPcm (`hold_others == false`): `plan_for` is unchanged (segue kept) |
| D16 | a fade stop while `player.dsd` is set gives `StopNow` and stops the player at once |
| D17 | `SetVolume` below 1.0 while `player.dsd` is set emits `LeaveDsd` before `SetVolume` and clears `player.dsd`; at 1.0 it does not |
| D18 | `dsd_holds_others` is true only while a held DSD entry is on air |

- [ ] **Step 1: Write the failing tests**

Create `crates/fp-model/tests/dsd_rules.rs`:

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback 2 spec O25: when DSD reaches a device unchanged, and what the
//! players do while it does.

mod common;

use common::{entries, fixture, p0};
use fp_model::{
    AppState, AudioFormat, BpBadge, Command, DsdFacts, DsdFallback, DsdOutput, DsdStreamMode,
    DsdTarget, EngineAction, EngineEvent, EntryId, MarkerKind, PlayMode, PlayerId, TransitionPlan,
    Transport,
    apply, bp_badge, dsd_decision, dsd_holds_others, on_event, plan_for,
};

const DSD64: u32 = 2_822_400;

fn dsd(rate: u32, channels: u32) -> Option<AudioFormat> {
    Some(AudioFormat {
        sample_rate: rate / 32,
        bits: None,
        channels,
        dsd_rate: Some(rate),
    })
}

fn facts() -> DsdFacts {
    DsdFacts {
        mode: DsdOutput::Dop,
        format: dsd(DSD64, 2),
        volume: 1.0,
        device_idle: true,
    }
}

#[test]
fn d1_pcm_mode_converts() {
    let f = DsdFacts { mode: DsdOutput::Pcm, ..facts() };
    assert_eq!(dsd_decision(&f), Err(DsdFallback::ModeIsPcm));
    assert!(!DsdFallback::ModeIsPcm.worth_logging());
}

#[test]
fn d2_a_pcm_file_is_not_dsd() {
    for format in [
        None,
        Some(AudioFormat { sample_rate: 88_200, bits: Some(24), channels: 2, dsd_rate: None }),
    ] {
        let f = DsdFacts { format, ..facts() };
        assert_eq!(dsd_decision(&f), Err(DsdFallback::NotDsd));
    }
}

#[test]
fn d3_more_than_two_channels_convert() {
    let f = DsdFacts { format: dsd(DSD64, 6), ..facts() };
    assert_eq!(dsd_decision(&f), Err(DsdFallback::Multichannel));
    for channels in [0, 1, 2] {
        let f = DsdFacts { format: dsd(DSD64, channels), ..facts() };
        assert!(dsd_decision(&f).is_ok(), "{channels}");
    }
}

#[test]
fn d4_a_volume_below_unity_converts() {
    let f = DsdFacts { volume: 0.99, ..facts() };
    assert_eq!(dsd_decision(&f), Err(DsdFallback::VolumeNotUnity));
    assert!(DsdFallback::VolumeNotUnity.worth_logging());
}

#[test]
fn d5_a_busy_device_converts() {
    let f = DsdFacts { device_idle: false, ..facts() };
    assert_eq!(dsd_decision(&f), Err(DsdFallback::DeviceBusy));
}

#[test]
fn d6_the_word_rate_is_the_dsd_rate_over_16() {
    assert_eq!(
        dsd_decision(&facts()),
        Ok(DsdTarget { mode: DsdStreamMode::Dop, dsd_rate: DSD64, word_rate: 176_400 })
    );
    let f = DsdFacts { mode: DsdOutput::Native, format: dsd(DSD64 * 4, 2), ..facts() };
    assert_eq!(
        dsd_decision(&f),
        Ok(DsdTarget { mode: DsdStreamMode::Native, dsd_rate: DSD64 * 4, word_rate: 705_600 })
    );
    assert!(DsdFallback::RateRefused(705_600).to_string().contains("705600"));
}

#[test]
fn d7_the_badge_reads_dsd_while_dsd_goes_out() {
    assert_eq!(bp_badge(false, false), BpBadge::Off);
    assert_eq!(bp_badge(true, false), BpBadge::Pcm);
    assert_eq!(bp_badge(false, true), BpBadge::Dsd);
    assert_eq!(bp_badge(true, true), BpBadge::Dsd);
}

// ---------------------------------------------------------------- players

fn three() -> (AppState, PlayerId, [EntryId; 3]) {
    let state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    (state, p, [e[0], e[1], e[2]])
}

/// Entry `a` plays and the engine reported it going out as DSD.
fn on_air(hold_others: bool) -> (AppState, PlayerId, [EntryId; 3]) {
    let (mut s, p, e) = three();
    apply(&mut s, Command::Play(p)).unwrap();
    on_event(&mut s, EngineEvent::DsdStarted { player: p, entry: e[0], hold_others });
    (s, p, e)
}

fn plan(s: &AppState, p: PlayerId) -> Option<TransitionPlan> {
    plan_for(s, s.player(p).unwrap())
}

fn with_segue(state: &mut AppState, secs: f64) {
    for t in state.library.iter_mut() {
        t.markers.set_auto(MarkerKind::SegueStart, Some(secs));
    }
}

fn started(out: &[EngineAction], p: PlayerId) -> Option<EntryId> {
    out.iter().find_map(|a| match a {
        EngineAction::StartCurrent { player, request } if *player == p => Some(request.entry),
        _ => None,
    })
}

#[test]
fn d8_dsd_started_marks_only_the_entry_on_air() {
    let (s, p, [a, b, _]) = on_air(true);
    assert_eq!(s.player(p).unwrap().dsd.map(|d| (d.entry, d.hold_others)), Some((a, true)));
    let (mut s2, p2, _) = three();
    on_event(&mut s2, EngineEvent::DsdStarted { player: p2, entry: b, hold_others: true });
    assert_eq!(s2.player(p2).unwrap().dsd, None, "stopped player: stale");
    let mut s3 = s.clone();
    on_event(&mut s3, EngineEvent::DsdStarted { player: p, entry: b, hold_others: false });
    assert_eq!(s3.player(p).unwrap().dsd.map(|d| d.entry), Some(a), "another entry: stale");
}

#[test]
fn d9_dsd_ended_and_leaving_the_entry_clear_it() {
    let (mut s, p, [a, b, _]) = on_air(false);
    on_event(&mut s, EngineEvent::DsdEnded { player: p, entry: b });
    assert!(s.player(p).unwrap().dsd.is_some(), "another entry: stale");
    on_event(&mut s, EngineEvent::DsdEnded { player: p, entry: a });
    assert!(s.player(p).unwrap().dsd.is_none());

    let (mut s, p, _) = on_air(false);
    apply(&mut s, Command::Play(p)).unwrap(); // Play while playing: the next entry
    assert!(s.player(p).unwrap().dsd.is_none(), "advancing clears it");
    let (mut s, p, _) = on_air(false);
    apply(&mut s, Command::Stop(p)).unwrap();
    assert!(s.player(p).unwrap().dsd.is_none(), "stopping clears it");
}

#[test]
fn d10_hold_others_never_overlaps_the_next_entry() {
    let (mut s, p, _) = on_air(true);
    // A segue inside the track would overlap; held, the track plays to its end.
    with_segue(&mut s, 170.0);
    assert_eq!(plan(&s, p), Some(TransitionPlan::StopAt { at_secs: 180.0 }));
}

#[test]
fn d11_hold_others_starts_the_next_entry_when_the_dsd_track_ends() {
    let (mut s, p, [a, b, _]) = on_air(true);
    let out = on_event(&mut s, EngineEvent::ReachedEnd { player: p, entry: a });
    assert_eq!(started(&out, p), Some(b));
    let player = s.player(p).unwrap();
    assert_eq!(player.current, Some(b));
    assert_eq!(player.transport, Transport::Playing);
    assert_eq!(player.history.last(), Some(&a));
    assert!(player.dsd.is_none());
}

#[test]
fn d12_hold_others_repeats_a_repeating_entry_without_a_new_play() {
    let (mut s, p, [a, _, _]) = on_air(true);
    apply(&mut s, Command::SetEntryRepeat(a, true)).unwrap();
    let out = on_event(&mut s, EngineEvent::ReachedEnd { player: p, entry: a });
    assert_eq!(started(&out, p), Some(a));
    let player = s.player(p).unwrap();
    assert_eq!(player.current, Some(a));
    assert!(player.history.is_empty(), "a repeat pass is not a new play");
}

#[test]
fn d13_hold_others_still_obeys_every_stop_rule() {
    for setup in [
        (|s: &mut AppState, p: PlayerId, _a: EntryId| {
            apply(s, Command::SetMode(p, PlayMode::Single)).unwrap();
        }) as fn(&mut AppState, PlayerId, EntryId),
        |s, p, _a| {
            apply(s, Command::ToggleStopAfterCurrent(p)).unwrap();
        },
        |s, _p, a| {
            apply(s, Command::SetEntryStopAfter(a, true)).unwrap();
        },
    ] {
        let (mut s, p, [a, _, _]) = on_air(true);
        setup(&mut s, p, a);
        let out = on_event(&mut s, EngineEvent::ReachedEnd { player: p, entry: a });
        assert_eq!(started(&out, p), None);
        assert_eq!(s.player(p).unwrap().transport, Transport::Stopped);
    }
}

#[test]
fn d14_hold_others_starts_play_and_next_hard() {
    let (mut s, p, [_, b, _]) = on_air(true);
    let out = apply(&mut s, Command::Play(p)).unwrap(); // Play while playing
    assert_eq!(started(&out, p), Some(b));
    assert!(!out.iter().any(|a| matches!(a, EngineAction::Crossfade { .. })));
    assert!(!s.player(p).unwrap().fading);
}

#[test]
fn d15_convert_to_pcm_keeps_the_ordinary_plan() {
    let (mut s, p, _) = on_air(false);
    with_segue(&mut s, 170.0);
    assert_eq!(
        plan(&s, p),
        Some(TransitionPlan::StartNextAt { at_secs: 170.0, fade_current_until_secs: Some(180.0) })
    );
}

#[test]
fn d16_a_fade_stop_of_a_dsd_track_stops_at_once() {
    let (mut s, p, _) = on_air(false);
    let out = apply(&mut s, Command::FadeStop(p)).unwrap();
    assert!(out.iter().any(|a| matches!(a, EngineAction::StopNow { player } if *player == p)));
    assert!(!out.iter().any(|a| matches!(a, EngineAction::FadeOutAndStop { .. })));
    assert_eq!(s.player(p).unwrap().transport, Transport::Stopped);
}

#[test]
fn d17_a_fader_move_below_unity_leaves_dsd() {
    let (mut s, p, _) = on_air(false);
    let out = apply(&mut s, Command::SetVolume(p, 1.0)).unwrap();
    assert!(!out.iter().any(|a| matches!(a, EngineAction::LeaveDsd { .. })));
    let out = apply(&mut s, Command::SetVolume(p, 0.5)).unwrap();
    let leave = out.iter().position(|a| matches!(a, EngineAction::LeaveDsd { player } if *player == p));
    let volume = out.iter().position(|a| matches!(a, EngineAction::SetVolume { .. }));
    assert!(leave.is_some() && leave < volume, "{out:?}");
    assert!(s.player(p).unwrap().dsd.is_none());
}

#[test]
fn d18_the_notice_shows_while_a_held_dsd_track_is_on_air() {
    let (s, p, _) = on_air(true);
    assert!(dsd_holds_others(&s, p));
    let (s, p, _) = on_air(false);
    assert!(!dsd_holds_others(&s, p));
    let (mut s, p, _) = on_air(true);
    apply(&mut s, Command::Stop(p)).unwrap();
    assert!(!dsd_holds_others(&s, p));
}
```

The command names (`Play`, `Stop`, `FadeStop`, `SetEntryRepeat`, `SetEntryStopAfter`, `ToggleStopAfterCurrent`, `SetMode`) and `markers.set_auto(MarkerKind::SegueStart, …)` are the real ones on the base commit (`crates/fp-model/tests/transitions.rs` uses the same `with_segue`); `MarkerKind` must be re-exported by `fp_model` (it is used by other tests; check `lib.rs`).

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p fp-model --test dsd_rules`
Expected: FAIL to compile (`DsdFacts`, `dsd_rate`, `DsdStarted` …).

- [ ] **Step 3: Implement the types and the pure functions**

In `crates/fp-model/src/track.rs`, `AudioFormat` gains:

```rust
    /// The DSD rate of a DSD file (2 822 400 for DSD64); `None` for PCM.
    /// `sample_rate` is then the rate of its PCM conversion (DSD rate ÷ 32).
    #[serde(default)]
    pub dsd_rate: Option<u32>,
```

Fix every `AudioFormat { … }` literal (`grep -rn "AudioFormat {" crates`), adding `dsd_rate: None`; `fp-analysis/src/analyze.rs` fills it from the decoder in Task 6, `None` for now.

Append to `crates/fp-model/src/dsd.rs`:

```rust
use std::fmt;

use crate::ids::{EntryId, PlayerId};
use crate::player::Transport;
use crate::state::AppState;
use crate::track::AudioFormat;

/// DSD bits per channel in one sample of the word stream the engine carries.
pub const DSD_WORD_BITS: u32 = 16;

/// What the engine knows when a DSD track is about to start on a device.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DsdFacts {
    /// The device's mode (`OutputsConfig::dsd_output_for`).
    pub mode: DsdOutput,
    pub format: Option<AudioFormat>,
    /// The player's volume (linear gain).
    pub volume: f32,
    /// Nothing sounds on the device.
    pub device_idle: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DsdStreamMode {
    Dop,
    Native,
}

/// How DSD goes out: the stream mode, and the rate of its 16-bit word
/// stream (the device rate for DoP).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DsdTarget {
    pub mode: DsdStreamMode,
    pub dsd_rate: u32,
    pub word_rate: u32,
}

/// Why a DSD track is converted to PCM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DsdFallback {
    ModeIsPcm,
    NotDsd,
    Multichannel,
    VolumeNotUnity,
    DeviceBusy,
    RateRefused(u32),
    /// DoP needs a 24- or 32-bit integer device format.
    FormatTooNarrow,
    /// The device or the system refused the DSD stream.
    StreamRefused,
}

impl DsdFallback {
    /// Whether the conversion deserves a log line (not for the default
    /// mode, nor for a file that is not DSD).
    pub fn worth_logging(self) -> bool {
        !matches!(self, DsdFallback::ModeIsPcm | DsdFallback::NotDsd)
    }
}

impl fmt::Display for DsdFallback {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DsdFallback::ModeIsPcm => write!(f, "the device converts DSD to PCM"),
            DsdFallback::NotDsd => write!(f, "the track is not DSD"),
            DsdFallback::Multichannel => write!(f, "the track has more than two channels"),
            DsdFallback::VolumeNotUnity => write!(f, "the player's volume is not 100 %"),
            DsdFallback::DeviceBusy => write!(f, "something else plays on the device"),
            DsdFallback::RateRefused(rate) => write!(f, "the device refused {rate} Hz"),
            DsdFallback::FormatTooNarrow => {
                write!(f, "DoP needs a 24- or 32-bit integer device format")
            }
            DsdFallback::StreamRefused => write!(f, "the device refused the DSD stream"),
        }
    }
}

/// Whether a DSD track can reach its device unchanged, before the device is
/// asked (spec O25). The engine then opens the stream; a refusal there is
/// `RateRefused`, `FormatTooNarrow` or `StreamRefused`.
pub fn dsd_decision(facts: &DsdFacts) -> Result<DsdTarget, DsdFallback> {
    let mode = match facts.mode {
        DsdOutput::Pcm => return Err(DsdFallback::ModeIsPcm),
        DsdOutput::Dop => DsdStreamMode::Dop,
        DsdOutput::Native => DsdStreamMode::Native,
    };
    let format = facts.format.ok_or(DsdFallback::NotDsd)?;
    let dsd_rate = format.dsd_rate.filter(|r| *r > 0).ok_or(DsdFallback::NotDsd)?;
    if format.channels > 2 {
        return Err(DsdFallback::Multichannel);
    }
    #[allow(clippy::float_cmp)] // exactly unity: any other gain changes the signal
    if facts.volume != 1.0 {
        return Err(DsdFallback::VolumeNotUnity);
    }
    if !facts.device_idle {
        return Err(DsdFallback::DeviceBusy);
    }
    Ok(DsdTarget {
        mode,
        dsd_rate,
        word_rate: dsd_rate / DSD_WORD_BITS,
    })
}

/// A DSD stream the engine reported going out for a player's entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DsdOnAir {
    pub entry: EntryId,
    /// The mix policy in force: the engine holds other sources off the
    /// output (`DsdMix::HoldOthers`).
    pub hold_others: bool,
}

/// The player header's badge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BpBadge {
    Off,
    /// "BP": the PCM samples reach the device unchanged.
    Pcm,
    /// "DSD": the DSD stream reaches the device unchanged.
    Dsd,
}

pub fn bp_badge(bit_perfect: bool, dsd: bool) -> BpBadge {
    match (bit_perfect, dsd) {
        (_, true) => BpBadge::Dsd,
        (true, false) => BpBadge::Pcm,
        (false, false) => BpBadge::Off,
    }
}

/// Whether `player` plays DSD that holds the other sources off its output,
/// for the notice that says they are muted.
pub fn dsd_holds_others(state: &AppState, player: PlayerId) -> bool {
    state.player(player).is_ok_and(|p| {
        p.transport != Transport::Stopped
            && p.dsd.is_some_and(|d| d.hold_others && p.current == Some(d.entry))
    })
}
```

(Adjust the `use` paths to the crate's real module names; `state.player(id)` returns `Result<&PlayerState, ModelError>` per `entry_notice.rs`.)

`player.rs`: `PlayerState` gains `pub dsd: Option<crate::dsd::DsdOnAir>,` (doc: "The DSD stream the engine reported for the entry on air (spec O25); runtime only, never saved."), `None` in `PlayerState::new`. Check that no `Serialize` derive or session snapshot copies `PlayerState` wholesale (`session.rs`); if one does, leave `dsd` out of it.

`command.rs`: add to `EngineEvent`:

```rust
    /// `entry` reaches its Main device as DSD, unchanged (spec O25).
    /// `hold_others` is the mix policy in force (`DsdMix::HoldOthers`).
    /// Ignored unless `entry` is the player's current.
    DsdStarted { player: PlayerId, entry: EntryId, hold_others: bool },
    /// The DSD stream of `entry` ended or was switched to PCM.
    DsdEnded { player: PlayerId, entry: EntryId },
```

and to `EngineAction`:

```rust
    /// Switch the player's DSD stream to PCM now (spec O25: the fader left
    /// unity).
    LeaveDsd { player: PlayerId },
```

- [ ] **Step 4: Implement the reducer rules**

In `crates/fp-model/src/reducer.rs`:

1. Rename the current body of `plan_for` to `fn ordinary_plan(state: &AppState, player: &PlayerState) -> Option<TransitionPlan>` (unchanged) and make `plan_for`:

```rust
/// Rules 9–11 (see `ordinary_plan`), then O25: a DSD stream that holds its
/// output is never overlapped; it plays to its end and the next entry
/// starts from `ReachedEnd` (`dsd_hold_continue`).
pub fn plan_for(state: &AppState, player: &PlayerState) -> Option<TransitionPlan> {
    let plan = ordinary_plan(state, player)?;
    if dsd_holds(player) && matches!(plan, TransitionPlan::StartNextAt { .. }) {
        let end = current_end(state, player)?;
        return Some(TransitionPlan::StopAt { at_secs: end });
    }
    Some(plan)
}

/// The end of the play range of `player`'s current entry.
fn current_end(state: &AppState, player: &PlayerState) -> Option<f64> {
    let track = state.track_for_entry(player.current?)?;
    let range = track.play_range(state.config.players.use_cue_markers);
    Some(range.known_end().unwrap_or(SOURCE_END))
}

/// The player's DSD stream holds its output (`DsdMix::HoldOthers`).
pub(crate) fn dsd_holds(player: &PlayerState) -> bool {
    player
        .dsd
        .is_some_and(|d| d.hold_others && player.current == Some(d.entry))
}

/// O25 HoldOthers: at the end of a held DSD track, what the ordinary plan
/// would have started at its end, started now with no overlap. `None` when
/// the ordinary plan stops the player.
fn dsd_hold_continue(state: &mut AppState, i: usize) -> Option<SourceRequest> {
    let plan = ordinary_plan(state, &state.players[i])?;
    if !matches!(plan, TransitionPlan::StartNextAt { .. }) {
        return None;
    }
    state.players[i].dsd = None;
    let current = state.players[i].current?;
    if repeating(state, &state.players[i]) {
        // R26: another pass of the same entry, not a new play.
        let request = state.request_from_cue_in(current)?;
        let p = &mut state.players[i];
        p.preloaded = None;
        p.scheduled = None;
        return Some(request);
    }
    let next = state.players[i].next?;
    advance_to(state, i, next, true)
}
```

2. `on_event`: replace the `ReachedEnd` arm and add the two DSD arms:

```rust
        EngineEvent::ReachedEnd { player, entry } => {
            if let Ok(i) = state.player_index(player)
                && state.players[i].current == Some(entry)
            {
                let held = dsd_holds(&state.players[i]);
                match held.then(|| dsd_hold_continue(state, i)).flatten() {
                    Some(request) => out.push(EngineAction::StartCurrent { player, request }),
                    None => stop_player(state, i),
                }
            }
        }
        EngineEvent::DsdStarted { player, entry, hold_others } => {
            if let Ok(i) = state.player_index(player)
                && state.players[i].current == Some(entry)
                && state.players[i].transport != Transport::Stopped
            {
                state.players[i].dsd = Some(DsdOnAir { entry, hold_others });
            }
        }
        EngineEvent::DsdEnded { player, entry } => {
            if let Ok(i) = state.player_index(player)
                && state.players[i].dsd.is_some_and(|d| d.entry == entry)
            {
                state.players[i].dsd = None;
            }
        }
```

3. `advance_to` and `stop_player`: set `player.dsd = None;` next to `player.scheduled = None;` / `player.stop_after_current = false;`.

4. Every `EngineAction::Crossfade` push (`grep -n "EngineAction::Crossfade" crates/fp-model/src/reducer.rs`: `play`, `previous`, and any other): when `dsd_holds(&state.players[i])`, push `EngineAction::StartCurrent { player: id, request }` instead and do not set `fading`. Use one helper:

```rust
/// Play-while-playing starts: a crossfade, or (O25 HoldOthers) a hard start
/// that ends the DSD stream first.
fn start_or_crossfade(
    state: &mut AppState,
    i: usize,
    request: SourceRequest,
    out: &mut Vec<EngineAction>,
) {
    let id = state.players[i].id;
    // `advance_to` already cleared `dsd`; the hold is read before it.
    out.push(EngineAction::Crossfade {
        player: id,
        request,
        fade_ms: state.config.players.fade_ms,
    });
    state.players[i].fading = true;
}
```

   Because `advance_to` clears `dsd`, read `let held = dsd_holds(&state.players[i]);` *before* calling `advance(state, i)` in each caller, then `if held { out.push(StartCurrent{..}) } else { start_or_crossfade(..) }`.

5. Fade stop (find the `Command::FadeStop` handler, the one that pushes `FadeOutAndStop`): when `state.players[i].dsd.is_some()`, do exactly what the plain Stop command does (call the same code path: `stop_player` + `EngineAction::StopNow`) and return.

6. `Command::SetVolume`: after clamping and before pushing `SetVolume`:

```rust
            #[allow(clippy::float_cmp)] // only exactly unity keeps DSD unchanged
            if volume != 1.0 && state.players[i].dsd.take().is_some() {
                out.push(EngineAction::LeaveDsd { player: id });
            }
```

7. `lib.rs`: re-export `BpBadge, DsdFacts, DsdFallback, DsdOnAir, DsdStreamMode, DsdTarget, DSD_WORD_BITS, bp_badge, dsd_decision, dsd_holds_others`.

8. `crates/fp-engine/src/engine.rs` `execute`: add `EngineAction::LeaveDsd { .. } => {}` with a comment `// Task 9 (feedback 2 plan 10)` so the workspace compiles. Any exhaustive `match` on `EngineEvent` elsewhere (`grep -rn "EngineEvent::CartCueEnded" crates`) gets the two new arms (ignored where irrelevant).

- [ ] **Step 5: Run the tests and the gate**

Run: `cargo test -p fp-model` then the full gate.
Expected: PASS; every existing reducer test still passes (the rules change nothing while `player.dsd` is `None`).

- [ ] **Step 6: Commit**

```bash
git add crates/fp-model crates/fp-engine/src/engine.rs crates/fp-app crates/fp-analysis crates/fp-engine/tests crates/fp-remote
git commit -m "feat(model): rules for DSD reaching a device unchanged (O25)"
```

---

### Task 5: DSD primitives in the backends (pure encoders, stream flag, Offline hooks)

**Files:**
- Create: `crates/fp-backends/src/dsd.rs`
- Modify: `crates/fp-backends/src/lib.rs` (`pub mod dsd;`, `StreamConfig::dsd`, `DeviceInfo::native_dsd`, `OutputStream::dsd`), `null.rs`, `offline.rs`, `cpal_backend.rs` (`choose_sample_format` for DoP; refuse `Native` for now), every `StreamConfig { … }` literal (17 sites: `grep -rn "StreamConfig {" crates`) and every `DeviceInfo { … }` literal (6 sites)
- Test: `crates/fp-backends/tests/dsd.rs`, `crates/fp-backends/tests/backends.rs` (Offline and Null cases)

**Interfaces:**
- Produces (used by Tasks 7–11):

```rust
// fp_backends::dsd
pub const DSD_SILENCE: u8 = 0x69;
pub const DOP_MARKERS: [u8; 2] = [0x05, 0xFA];
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] pub enum DsdStream { Dop, Native }
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] pub enum NativeDsdFormat { U32Be, U32Le, U16Be, U16Le, U8 }
pub const NATIVE_PREFERENCE: [NativeDsdFormat; 5];
impl NativeDsdFormat { pub fn bytes(self) -> usize; pub fn device_rate(self, word_rate: u32) -> u32 }
pub fn choose_native_format(accepts: impl FnMut(NativeDsdFormat) -> bool) -> Option<NativeDsdFormat>
pub fn word_to_sample(first: u8, second: u8) -> f32
pub fn sample_to_word(sample: f32) -> [u8; 2]
pub fn silence_sample() -> f32
pub fn dop_sample(marker: u8, word: f32) -> f32
pub struct DopEncoder { /* next marker */ }
impl DopEncoder { pub fn new() -> Self; pub fn encode_in_place(&mut self, buf: &mut [f32], channels: usize) }
pub fn pack_native(format: NativeDsdFormat, words: &[f32], channels: usize, out: &mut [u8]) -> usize
// fp_backends
StreamConfig { …, pub dsd: Option<dsd::DsdStream> }
DeviceInfo { …, pub native_dsd: bool }
trait OutputStream { …; fn dsd(&self) -> Option<dsd::DsdStream> { None } }
OfflineDevice::set_sample_format(&self, SampleFormat)
OfflineDevice::set_native_dsd(&self, bool)
```

- [ ] **Step 1: Write the failing tests**

Create `crates/fp-backends/tests/dsd.rs`:

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing)]
//! DSD carried through the f32 path (feedback 2 spec O25): words, DoP 1.1
//! and the native ALSA formats, byte for byte.

use fp_backends::SampleFormat;
use fp_backends::dsd::{
    DOP_MARKERS, DSD_SILENCE, DopEncoder, NativeDsdFormat, choose_native_format, dop_sample,
    pack_native, sample_to_word, silence_sample, word_to_sample,
};
use fp_backends::exclusive::write_samples;

#[test]
fn every_word_survives_the_f32_path() {
    for first in 0..=255u8 {
        for second in [0x00, 0x01, 0x69, 0x7F, 0x80, 0xFE, 0xFF] {
            assert_eq!(sample_to_word(word_to_sample(first, second)), [first, second]);
        }
    }
    assert_eq!(sample_to_word(silence_sample()), [DSD_SILENCE, DSD_SILENCE]);
}

/// The 24-bit DoP word the device receives for `sample`, through the same
/// integer conversion as a real stream.
fn device_word(format: SampleFormat, sample: f32) -> u32 {
    let mut bytes = [0u8; 4];
    write_samples(format, &[sample], &mut bytes);
    u32::from_le_bytes(bytes) >> 8
}

#[test]
fn a_dop_sample_is_the_marker_above_the_two_bytes_exactly() {
    for format in [SampleFormat::I24, SampleFormat::I32] {
        for marker in DOP_MARKERS {
            for (a, b) in [(0x00, 0x00), (0x69, 0x69), (0xAB, 0xCD), (0xFF, 0xFF), (0x80, 0x01)] {
                let s = dop_sample(marker, word_to_sample(a, b));
                let expected = (u32::from(marker) << 16) | (u32::from(a) << 8) | u32::from(b);
                assert_eq!(device_word(format, s), expected, "{format:?} {marker:#x} {a:#x}{b:#x}");
            }
        }
    }
}

#[test]
fn markers_alternate_frame_by_frame_across_calls_and_channels() {
    let mut enc = DopEncoder::new();
    let w = word_to_sample(0x12, 0x34);
    let mut first = vec![w; 3 * 2]; // 3 stereo frames
    let mut second = vec![w; 2 * 2];
    enc.encode_in_place(&mut first, 2);
    enc.encode_in_place(&mut second, 2);
    let markers: Vec<u32> = first
        .iter()
        .chain(&second)
        .map(|s| device_word(SampleFormat::I24, *s) >> 16)
        .collect();
    assert_eq!(markers, vec![0x05, 0x05, 0xFA, 0xFA, 0x05, 0x05, 0xFA, 0xFA, 0x05, 0x05]);
}

#[test]
fn native_formats_pack_bytes_in_time_order() {
    // Two stereo word frames: left 0x0102 then 0x0304, right 0xA1A2 then 0xA3A4.
    let words = [
        word_to_sample(0x01, 0x02),
        word_to_sample(0xA1, 0xA2),
        word_to_sample(0x03, 0x04),
        word_to_sample(0xA3, 0xA4),
    ];
    let cases: [(NativeDsdFormat, &[u8]); 5] = [
        (NativeDsdFormat::U32Be, &[1, 2, 3, 4, 0xA1, 0xA2, 0xA3, 0xA4]),
        (NativeDsdFormat::U32Le, &[4, 3, 2, 1, 0xA4, 0xA3, 0xA2, 0xA1]),
        (NativeDsdFormat::U16Be, &[1, 2, 0xA1, 0xA2, 3, 4, 0xA3, 0xA4]),
        (NativeDsdFormat::U16Le, &[2, 1, 0xA2, 0xA1, 4, 3, 0xA4, 0xA3]),
        (NativeDsdFormat::U8, &[1, 0xA1, 2, 0xA2, 3, 0xA3, 4, 0xA4]),
    ];
    for (format, expected) in cases {
        let mut out = [0u8; 8];
        assert_eq!(pack_native(format, &words, 2, &mut out), 8, "{format:?}");
        assert_eq!(&out, expected, "{format:?}");
    }
}

#[test]
fn native_device_rates_follow_the_container() {
    let word_rate = 176_400; // DSD64
    assert_eq!(NativeDsdFormat::U32Be.device_rate(word_rate), 88_200);
    assert_eq!(NativeDsdFormat::U16Le.device_rate(word_rate), 176_400);
    assert_eq!(NativeDsdFormat::U8.device_rate(word_rate), 352_800);
}

#[test]
fn the_preferred_native_format_is_32_bit_big_endian() {
    assert_eq!(choose_native_format(|_| true), Some(NativeDsdFormat::U32Be));
    assert_eq!(
        choose_native_format(|f| matches!(f, NativeDsdFormat::U8 | NativeDsdFormat::U16Le)),
        Some(NativeDsdFormat::U16Le)
    );
    assert_eq!(choose_native_format(|_| false), None);
}
```

In `crates/fp-backends/tests/backends.rs` add (follow the file's helpers for a no-op renderer and error sink):

```rust
#[test]
fn offline_dop_needs_exclusive_access_and_a_24_bit_integer_format() {
    let backend = OfflineBackend::new();
    let dac = backend.add_device("dac", 2);
    dac.set_exclusive_capable(true);
    let config = StreamConfig {
        sample_rate: 176_400,
        buffer_frames: 512,
        channels: 2,
        exclusive: true,
        dsd: Some(DsdStream::Dop),
    };
    assert!(matches!(open(&backend, &dac, config), Err(BackendError::Unsupported(_))), "F32 device");
    dac.set_sample_format(SampleFormat::I24);
    let stream = open(&backend, &dac, config).unwrap();
    assert_eq!(stream.sample_format(), SampleFormat::I24);
    assert_eq!(stream.dsd(), Some(DsdStream::Dop));
}

#[test]
fn offline_native_dsd_needs_the_capability() {
    let backend = OfflineBackend::new();
    let dac = backend.add_device("dac", 2);
    dac.set_exclusive_capable(true);
    let config = StreamConfig {
        sample_rate: 176_400,
        buffer_frames: 512,
        channels: 2,
        exclusive: true,
        dsd: Some(DsdStream::Native),
    };
    assert!(open(&backend, &dac, config).is_err());
    dac.set_native_dsd(true);
    assert!(backend.enumerate_devices().unwrap()[0].native_dsd);
    assert_eq!(open(&backend, &dac, config).unwrap().dsd(), Some(DsdStream::Native));
}

#[test]
fn null_refuses_any_dsd_stream() {
    let config = StreamConfig {
        sample_rate: 176_400,
        buffer_frames: 512,
        channels: 2,
        exclusive: false,
        dsd: Some(DsdStream::Dop),
    };
    assert!(open_null(config).is_err());
}
```

(`open`/`open_null` are thin helpers over `AudioBackend::open_output` with a silent renderer and a no-op sink; reuse what the file already has.)

In `crates/fp-backends/src/cpal_backend.rs` `mod tests` add:

```rust
    #[test]
    fn dop_uses_only_24_or_32_bit_integer_formats() {
        use cpal::SampleFormat::{F32, I16, I24, I32};
        assert_eq!(choose_dop_sample_format(&[F32, I32, I16]), Some(I32));
        assert_eq!(choose_dop_sample_format(&[F32, I24]), Some(I24));
        assert_eq!(choose_dop_sample_format(&[F32, I16]), None);
    }
```

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p fp-backends`
Expected: FAIL to compile.

- [ ] **Step 3: Implement `dsd.rs`**

```rust
//! DSD carried through the engine's f32 path (feedback 2 spec O25). A DSD
//! stream is a stream of 16-bit words (16 DSD bits per channel, the first
//! byte first in time, each byte most significant bit first) at the word
//! rate, DSD rate ÷ 16. Each word travels exactly as one f32 sample. Pure
//! and real-time safe: no allocation, no panics.

/// The DSD idle pattern: a decoder turns it into silence.
pub const DSD_SILENCE: u8 = 0x69;
/// DoP 1.1 markers, alternating frame by frame, the same in every channel
/// of one frame.
pub const DOP_MARKERS: [u8; 2] = [0x05, 0xFA];

/// How a stream carries DSD (`StreamConfig::dsd`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DsdStream {
    /// DSD over PCM (DoP 1.1) in a 24- or 32-bit integer PCM stream.
    Dop,
    /// Raw DSD in one of the device's DSD formats.
    Native,
}

/// Native DSD sample formats (ALSA `DSD_U32_BE` … `DSD_U8`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeDsdFormat {
    U32Be,
    U32Le,
    U16Be,
    U16Le,
    U8,
}

/// The order in which native formats are tried.
pub const NATIVE_PREFERENCE: [NativeDsdFormat; 5] = [
    NativeDsdFormat::U32Be,
    NativeDsdFormat::U32Le,
    NativeDsdFormat::U16Be,
    NativeDsdFormat::U16Le,
    NativeDsdFormat::U8,
];

impl NativeDsdFormat {
    /// DSD bytes per channel in one device frame.
    pub fn bytes(self) -> usize {
        match self {
            NativeDsdFormat::U32Be | NativeDsdFormat::U32Le => 4,
            NativeDsdFormat::U16Be | NativeDsdFormat::U16Le => 2,
            NativeDsdFormat::U8 => 1,
        }
    }

    /// The device's frame rate for a word stream at `word_rate`.
    pub fn device_rate(self, word_rate: u32) -> u32 {
        word_rate.saturating_mul(2) / self.bytes() as u32
    }
}

/// The first native format, in preference order, that `accepts` takes.
pub fn choose_native_format(
    mut accepts: impl FnMut(NativeDsdFormat) -> bool,
) -> Option<NativeDsdFormat> {
    NATIVE_PREFERENCE.into_iter().find(|f| accepts(*f))
}

/// One word as an f32 sample, exactly (an `i16` over 32768).
pub fn word_to_sample(first: u8, second: u8) -> f32 {
    f32::from(i16::from_be_bytes([first, second])) / 32768.0
}

/// The word an f32 sample carries (the inverse of `word_to_sample`).
pub fn sample_to_word(sample: f32) -> [u8; 2] {
    // Saturating float-to-int conversion; NaN reads as 0.
    let v = (sample * 32768.0).round().clamp(-32768.0, 32767.0) as i16;
    v.to_be_bytes()
}

/// The DSD idle word as a sample.
pub fn silence_sample() -> f32 {
    word_to_sample(DSD_SILENCE, DSD_SILENCE)
}

/// One DoP sample: `marker` above the word's two bytes, as 24-bit PCM in an
/// f32 (exact: 24 bits fit the mantissa, and the integer conversions scale
/// by powers of two).
pub fn dop_sample(marker: u8, word: f32) -> f32 {
    let [a, b] = sample_to_word(word);
    let v = (u32::from(marker) << 16) | (u32::from(a) << 8) | u32::from(b);
    // Sign-extend the 24-bit word.
    let signed = ((v << 8) as i32) >> 8;
    signed as f32 / 8_388_608.0
}

/// Turns word frames into DoP frames, keeping the marker alternation across
/// calls (one encoder per open stream).
#[derive(Debug, Clone, Copy, Default)]
pub struct DopEncoder {
    odd: bool,
}

impl DopEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Replaces each frame of words in `buf` (`channels` per frame) by its
    /// DoP samples. Real-time safe.
    pub fn encode_in_place(&mut self, buf: &mut [f32], channels: usize) {
        for frame in buf.chunks_exact_mut(channels.max(1)) {
            let marker = if self.odd { DOP_MARKERS[1] } else { DOP_MARKERS[0] };
            for s in frame.iter_mut() {
                *s = dop_sample(marker, *s);
            }
            self.odd = !self.odd;
        }
    }
}

/// Packs word frames (`channels` per frame) into device bytes for `format`;
/// returns the bytes written. 32-bit formats take two word frames per
/// device frame (an odd last word frame is left out: streams render an
/// even number of frames). Stops where `out` ends. Real-time safe.
pub fn pack_native(format: NativeDsdFormat, words: &[f32], channels: usize, out: &mut [u8]) -> usize {
    let channels = channels.max(1);
    let mut written = 0;
    let mut put = |byte: u8, written: &mut usize| {
        if let Some(o) = out.get_mut(*written) {
            *o = byte;
            *written += 1;
        }
    };
    match format {
        NativeDsdFormat::U16Be | NativeDsdFormat::U16Le => {
            for s in words {
                let [a, b] = sample_to_word(*s);
                let (x, y) = if format == NativeDsdFormat::U16Be { (a, b) } else { (b, a) };
                put(x, &mut written);
                put(y, &mut written);
            }
        }
        NativeDsdFormat::U8 => {
            for frame in words.chunks_exact(channels) {
                for half in 0..2 {
                    for s in frame {
                        let [a, b] = sample_to_word(*s);
                        put(if half == 0 { a } else { b }, &mut written);
                    }
                }
            }
        }
        NativeDsdFormat::U32Be | NativeDsdFormat::U32Le => {
            for pair in words.chunks_exact(channels * 2) {
                let (first, second) = pair.split_at(channels);
                for (s1, s2) in first.iter().zip(second) {
                    let [a, b] = sample_to_word(*s1);
                    let [c, d] = sample_to_word(*s2);
                    let bytes = if format == NativeDsdFormat::U32Be { [a, b, c, d] } else { [d, c, b, a] };
                    for byte in bytes {
                        put(byte, &mut written);
                    }
                }
            }
        }
    }
    written
}
```

`DOP_MARKERS[0]`/`[1]` are indexing on a const array; `clippy::indexing_slicing` flags them in this crate — write `let [even, odd] = DOP_MARKERS;` once and pick between them.

- [ ] **Step 4: Thread the flag through the backends**

- `lib.rs`: `pub mod dsd;`; `StreamConfig` gains `/// DSD carried by this stream (feedback 2 spec O25); `None` for PCM. pub dsd: Option<dsd::DsdStream>,`; `DeviceInfo` gains `/// The device takes raw DSD (Linux, ALSA hw: devices reporting a DSD format). pub native_dsd: bool,`; `OutputStream` gains `fn dsd(&self) -> Option<dsd::DsdStream> { None }`. Add `dsd: None` / `native_dsd: false` to every literal (`grep -rn "StreamConfig {\|DeviceInfo {" crates`).
- `null.rs`: refuse `config.dsd.is_some()` with `Unsupported("the null output carries no DSD")` (before the exclusive check).
- `offline.rs`: `DeviceState` gains `sample_format: Option<SampleFormat>` (None = F32) and `native_dsd: bool`; `set_sample_format`, `set_native_dsd`; `enumerate_devices` reports `native_dsd`; `open_output` refuses `Some(Dop)` unless the format is `I24` or `I32`, refuses `Some(Native)` unless `native_dsd`; `OfflineStream` stores the format and the dsd flag and returns them from `sample_format()` / `dsd()`. A DSD stream also requires `exclusive` (refuse otherwise).
- `cpal_backend.rs`: add

```rust
/// DoP needs every bit of a 24-bit word: only 24- or 32-bit integer.
fn choose_dop_sample_format(formats: &[cpal::SampleFormat]) -> Option<cpal::SampleFormat> {
    [cpal::SampleFormat::I32, cpal::SampleFormat::I24]
        .into_iter()
        .find(|f| formats.contains(f))
}
```

  and in `open_on_host`, where the sample format is chosen (`choose_sample_format`), use `choose_dop_sample_format` when `config.dsd == Some(DsdStream::Dop)` and return `Unsupported("DoP needs a 24- or 32-bit integer format")` when it gives `None`. `Some(DsdStream::Native)` returns `Unsupported("native DSD is not available on this system")` for now (Task 10). The WASAPI exclusive and Core Audio paths: refuse `Some(Native)`; for `Some(Dop)` they open as today and the engine checks the format they report (Task 9).

- [ ] **Step 5: Run the tests and the gate**

Expected: PASS (`cargo test -p fp-backends`, then the gate).

- [ ] **Step 6: Commit**

```bash
git add crates
git commit -m "feat(backends): DoP and native DSD encoders and the DSD stream flag (O25)"
```

---

### Task 6: Raw DSD reading and the DSD rate in the analysis

**Files:**
- Create: `crates/fp-decode/src/dsd/raw.rs`
- Modify: `crates/fp-decode/src/dsd/mod.rs` (share the chunk reader; `DsdDecoder::dsd_rate`; `mod raw; pub use raw::DsdRawReader;`), `crates/fp-decode/src/lib.rs` (`pub use dsd::DsdRawReader;`, `FileDecoder::dsd_rate`)
- Modify: `crates/fp-analysis/src/analyze.rs` (`dsd_rate: decoder.dsd_rate()`), `crates/fp-analysis/src/cache.rs` (`ANALYSIS_VERSION` 6 → 7, with a comment "7: AudioFormat::dsd_rate (feedback 2 O25)")
- Test: `crates/fp-decode/tests/dsd.rs` (append), `crates/fp-analysis/tests/analyze.rs` (one case)

**Interfaces:**
- Produces (used by Task 7):

```rust
// fp_decode
pub struct DsdRawReader { … }
impl DsdRawReader {
    /// Opens a DSF or uncompressed DSDIFF file for its raw DSD bytes.
    pub fn open(path: &Path) -> Result<Self, String>;
    pub fn dsd_rate(&self) -> u32;
    pub fn channels(&self) -> usize;
    /// Bytes per channel of audio in the file.
    pub fn len_bytes(&self) -> u64;
    /// Positions at byte round(secs × rate ÷ 8) of each channel, rounded down
    /// to an even byte (one 16-bit word), clamped to the end.
    pub fn seek(&mut self, secs: f64) -> Result<(), String>;
    /// Appends the next bytes of every channel to `out[c]` (most significant
    /// bit first in time, whatever the container). `out.len()` must equal
    /// `channels()`. Returns `false` at the end of the audio.
    pub fn next_bytes(&mut self, out: &mut [Vec<u8>]) -> Result<bool, String>;
}
impl FileDecoder { pub fn dsd_rate(&self) -> Option<u32> }
```

- [ ] **Step 1: Write the failing tests**

Append to `crates/fp-decode/tests/dsd.rs` (it already has the `dsf`, `dsf_blocks`, `dff` writers and `write`; `DSD64` is its rate constant):

```rust
// ------------------------------------------------------------------ raw bytes

fn ramp_bytes(len: usize, seed: u8) -> Vec<u8> {
    (0..len).map(|i| (i as u8).wrapping_mul(7).wrapping_add(seed)).collect()
}

fn read_raw(path: &Path, from_secs: f64) -> Vec<Vec<u8>> {
    let mut r = fp_decode::DsdRawReader::open(path).unwrap();
    r.seek(from_secs).unwrap();
    let mut out = vec![Vec::new(); r.channels()];
    while r.next_bytes(&mut out).unwrap() {}
    out
}

#[test]
fn raw_bytes_of_dsf_and_dff_are_the_written_bytes_msb_first() {
    let dir = tempfile::tempdir().unwrap();
    let (l, r) = (ramp_bytes(10_000, 1), ramp_bytes(10_000, 100));
    let channels = vec![l.clone(), r.clone()];
    for path in [
        write(dir.path(), "a.dsf", &dsf(&channels, 10_000 * 8)),
        write(dir.path(), "a.dff", &dff(&channels, b"DSD ")),
    ] {
        let got = read_raw(&path, 0.0);
        assert_eq!(got, channels, "{}", path.display());
        let reader = fp_decode::DsdRawReader::open(&path).unwrap();
        assert_eq!(reader.dsd_rate(), DSD64);
        assert_eq!(reader.len_bytes(), 10_000);
    }
}

#[test]
fn a_raw_seek_lands_on_an_even_byte() {
    let dir = tempfile::tempdir().unwrap();
    let l = ramp_bytes(20_000, 3);
    let path = write(dir.path(), "a.dsf", &dsf(&[l.clone(), l.clone()], 20_000 * 8));
    // 0.01 s at DSD64 is byte 3528; a tiny offset rounds down to it.
    let got = read_raw(&path, 0.010_000_3);
    assert_eq!(got[0], l[3528..].to_vec());
    let got = read_raw(&path, 0.010_000_3 + 1.0 / f64::from(DSD64) * 8.0); // byte 3529 → 3528
    assert_eq!(got[0], l[3528..].to_vec());
    assert!(read_raw(&path, 99.0)[0].is_empty(), "past the end");
}

#[test]
fn a_truncated_dsf_ends_early_without_error() {
    let dir = tempfile::tempdir().unwrap();
    let l = ramp_bytes(BLOCK * 3, 5);
    let mut bytes = dsf(&[l.clone(), l.clone()], (BLOCK * 3 * 8) as u64);
    bytes.truncate(bytes.len() - BLOCK * 2 - 100);
    let path = write(dir.path(), "cut.dsf", &bytes);
    let got = read_raw(&path, 0.0);
    assert!(got[0].len() < l.len());
    assert_eq!(got[0], l[..got[0].len()].to_vec(), "what is there is exact");
}

#[test]
fn the_decoder_reports_the_dsd_rate_and_pcm_files_none() {
    let dir = tempfile::tempdir().unwrap();
    let path = write(dir.path(), "a.dsf", &dsf(&[ramp_bytes(4096, 0)], 4096 * 8));
    assert_eq!(fp_decode::FileDecoder::open(&path).unwrap().dsd_rate(), Some(DSD64));
    let ogg = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/lossy/tone.ogg");
    assert_eq!(fp_decode::FileDecoder::open(&ogg).unwrap().dsd_rate(), None);
}
```

(`dsf`'s second argument is the sample count per channel, in bits: bytes × 8, as the existing tests pass it.)

In `crates/fp-analysis/tests/analyze.rs`, add a case that analyses a DSF (write one with the same byte layout as above; copy the minimal `dsf` writer into the test or into a shared helper) and asserts `format.dsd_rate == Some(2_822_400)` and `format.sample_rate == 88_200`, and one WAV asserting `dsd_rate == None`.

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p fp-decode --test dsd raw_ && cargo test -p fp-decode --test dsd truncated && cargo test -p fp-analysis --test analyze dsd`
Expected: FAIL to compile.

- [ ] **Step 3: Implement**

1. In `dsd/mod.rs`, move the file reading out of `DsdDecoder::read_chunk` into a struct both readers share, without changing what `DsdDecoder` produces:

```rust
/// Reads a DSD file's audio, chunk by chunk, into per-channel bytes (most
/// significant bit first).
pub(crate) struct ChunkReader {
    file: File,
    layout: Layout,
    /// The next byte (per channel) to read from the file.
    read_pos: u64,
    raw: Vec<u8>,
}

impl ChunkReader {
    /// Appends the next chunk of every channel to `out`. `false` at the end.
    pub(crate) fn read_chunk(&mut self, out: &mut [Vec<u8>]) -> Result<bool, String> { /* the body of today's read_chunk, writing into `out` */ }
    /// The first byte a read can start at for `byte`: DSF reads whole block
    /// groups, so the start of `byte`'s block.
    pub(crate) fn aligned(&self, byte: u64) -> u64 { /* block alignment as in DsdDecoder::seek */ }
    pub(crate) fn set_read_pos(&mut self, at: u64) { self.read_pos = at; }
}
```

   `DsdDecoder` keeps `bytes: Vec<Vec<u8>>`, `base`, `next` and uses `ChunkReader` for the I/O; its tests (`cargo test -p fp-decode --test dsd`) prove it unchanged.

2. `dsd/raw.rs`:

```rust
//! Raw DSD bytes for output without conversion (feedback 2 spec O25).

use std::path::Path;

use super::{ChunkReader, Layout};

pub struct DsdRawReader {
    reader: ChunkReader,
    /// Bytes per channel still to skip at the front of the next chunk (a
    /// seek inside a DSF block).
    skip: usize,
    /// Each output's length before a read (reused).
    lens: Vec<usize>,
}

impl DsdRawReader {
    pub fn open(path: &Path) -> Result<Self, String> { /* probe the first 16 bytes with crate::probe: Dsf → dsf::layout, Dff → dff::layout, else Err("not a DSD file"); layout.validate() */ }
    pub fn dsd_rate(&self) -> u32 { self.reader.layout.rate }
    pub fn channels(&self) -> usize { self.reader.layout.channels }
    pub fn len_bytes(&self) -> u64 { self.reader.layout.bytes }
    pub fn seek(&mut self, secs: f64) -> Result<(), String> {
        let rate = f64::from(self.dsd_rate());
        let byte = (secs.max(0.0) * rate / 8.0).round() as u64; // saturating
        let byte = (byte - byte % 2).min(self.len_bytes());
        let start = self.reader.aligned(byte);
        self.reader.set_read_pos(start);
        self.skip = usize::try_from(byte - start).map_err(|e| e.to_string())?;
        Ok(())
    }
    pub fn next_bytes(&mut self, out: &mut [Vec<u8>]) -> Result<bool, String> {
        if out.len() != self.channels() || out.is_empty() {
            return Err("one output per channel".to_owned());
        }
        self.lens.clear();
        self.lens.extend(out.iter().map(Vec::len));
        let more = self.reader.read_chunk(out)?;
        if self.skip > 0 {
            let fresh = out
                .first()
                .zip(self.lens.first())
                .map_or(0, |(v, before)| v.len() - before);
            let drop = self.skip.min(fresh);
            for (v, before) in out.iter_mut().zip(&self.lens) {
                v.drain(*before..*before + drop);
            }
            self.skip -= drop;
        }
        Ok(more)
    }
}
```

   (`lens: Vec<usize>` is a field reused across calls.) In words: it remembers each `out[c].len()` before the read (store the lengths in a reused `Vec<usize>` field), calls `read_chunk`, then, while `skip > 0`, drains `min(skip, new bytes)` from each channel's newly appended part and decreases `skip`; returns the `read_chunk` result. A layout with zero channels or `out.len() != channels()` returns `Err`. This runs on the decode worker, not the device callback: allocation is allowed, panics and indexing are not.

3. `DsdDecoder::dsd_rate(&self) -> u32 { self.layout.rate }`; `FileDecoder::dsd_rate(&self) -> Option<u32>` (`Backend::Dsd(d) => Some(d.dsd_rate())`, others `None`). `lib.rs`: `pub use dsd::DsdRawReader;`.

4. `analyze.rs`: `dsd_rate: decoder.dsd_rate(),` in the `AudioFormat` literal; `cache.rs`: `ANALYSIS_VERSION: u32 = 7`. Run the `fp-app` tests that pin the version (`tests/outdated_notice.rs`, `tests/services.rs`); they read the constant, so they need no change — confirm.

- [ ] **Step 4: Run the tests and the gate**

Expected: PASS, including every existing DSD conversion test.

- [ ] **Step 5: Commit**

```bash
git add crates/fp-decode crates/fp-analysis
git commit -m "feat(decode): raw DSD reading and the DSD rate of analysed files (O25)"
```

---

### Task 7: DSD sources — a word ring beside the PCM ring

**Files:**
- Modify: `crates/fp-engine/src/source.rs` (DSD pair), `crates/fp-engine/src/worker.rs` (`DsdSampleSource`, `DsdOpener`, `dsd_file_opener`, `LoadOptions::dsd`, the job's two pending buffers, `PlayerWorker::spawn_with_dsd`)
- Modify: `crates/fp-engine/tests/support/mod.rs` (a DSF writer for engine tests)
- Test: `crates/fp-engine/tests/worker.rs` (append), `crates/fp-engine/src/source.rs` unit tests

**Interfaces:**
- Consumes: Task 5 `fp_backends::dsd::{word_to_sample, silence_sample}`; Task 6 `fp_decode::DsdRawReader`.
- Produces (used by Tasks 8–9):

```rust
// fp_engine::source
pub fn source_pair_dsd(capacity_frames: usize) -> (SourceProducer, SourceConsumer);
impl SourceProducer {
    /// Pushes whole frames of `pcm` and `dsd` (stereo each, equal frame
    /// counts) into the two rings in lockstep; returns frames pushed.
    pub fn push_pair(&mut self, pcm: &[f32], dsd: &[f32]) -> usize;
    pub fn is_dsd(&self) -> bool;
}
impl SourceConsumer {
    pub fn is_dsd(&self) -> bool;
    /// Pops up to `pcm.len() / 2` frames from both rings in lockstep into
    /// `pcm` and `dsd` (same length); returns frames popped. Real-time safe.
    pub fn pop_pair(&mut self, pcm: &mut [f32], dsd: &mut [f32]) -> usize;
}
// fp_engine::worker
pub trait DsdSampleSource: Send {
    /// Appends equal numbers of stereo frames of PCM (at the word rate) and
    /// of DSD words; `false` at the end.
    fn next_pair(&mut self, pcm: &mut Vec<f32>, dsd: &mut Vec<f32>) -> Result<bool, String>;
}
pub type DsdOpener =
    Arc<dyn Fn(&Path, f64, u32) -> Result<Box<dyn DsdSampleSource>, String> + Send + Sync>;
pub fn dsd_file_opener() -> DsdOpener;   // (path, from_secs, word_rate)
LoadOptions { …, pub dsd: bool }
PlayerWorker::spawn_with_dsd(name, opener, dsd_opener: DsdOpener, bus_rate, ready_frames, failures)
// tests/support
pub fn dsf_file(dir: &Path, name: &str, left: &[u8], right: &[u8]) -> PathBuf  // DSD64, bytes MSB first
```

- [ ] **Step 1: Write the failing tests**

In `crates/fp-engine/src/source.rs` `mod tests`:

```rust
    #[test]
    fn a_dsd_pair_pushes_and_pops_both_rings_in_lockstep() {
        let (mut p, mut c) = source_pair_dsd(3);
        assert!(p.is_dsd() && c.is_dsd());
        let pcm = [1.0, -1.0, 2.0, -2.0, 3.0, -3.0, 4.0, -4.0];
        let dsd = [10.0, -10.0, 20.0, -20.0, 30.0, -30.0, 40.0, -40.0];
        assert_eq!(p.push_pair(&pcm, &dsd), 3, "the ring holds 3 frames");
        let (mut a, mut b) = ([0.0; 4], [0.0; 4]);
        assert_eq!(c.pop_pair(&mut a, &mut b), 2);
        assert_eq!(a, [1.0, -1.0, 2.0, -2.0]);
        assert_eq!(b, [10.0, -10.0, 20.0, -20.0]);
        assert_eq!(p.shared.frames_pushed.load(Ordering::Acquire), 3);
    }

    #[test]
    fn a_plain_source_is_not_dsd_and_pop_pair_gives_nothing() {
        let (p, mut c) = source_pair(2);
        assert!(!p.is_dsd() && !c.is_dsd());
        assert_eq!(c.pop_pair(&mut [0.0; 2], &mut [0.0; 2]), 0);
    }
```

In `crates/fp-engine/tests/support/mod.rs`, add the writer (DSD64, 4096-byte blocks, LSB first on disk as DSF requires):

```rust
pub const DSD64: u32 = 2_822_400;

/// A stereo DSF file at DSD64 whose channels hold `left` and `right`
/// (bytes most significant bit first in time, as the engine carries them).
pub fn dsf_file(dir: &Path, name: &str, left: &[u8], right: &[u8]) -> PathBuf {
    const BLOCK: usize = 4096;
    let len = left.len().min(right.len());
    let blocks = len.div_ceil(BLOCK);
    let mut data = Vec::new();
    for g in 0..blocks {
        for ch in [left, right] {
            let mut block = vec![0x69u8.reverse_bits(); BLOCK];
            for (i, byte) in ch.iter().skip(g * BLOCK).take(BLOCK.min(len - g * BLOCK)).enumerate() {
                block[i] = byte.reverse_bits();
            }
            data.extend(block);
        }
    }
    let mut f = Vec::new();
    let total = 28 + 52 + 12 + data.len() as u64;
    f.extend(b"DSD ");
    f.extend(28u64.to_le_bytes());
    f.extend(total.to_le_bytes());
    f.extend(0u64.to_le_bytes());
    f.extend(b"fmt ");
    f.extend(52u64.to_le_bytes());
    f.extend(1u32.to_le_bytes()); // version
    f.extend(0u32.to_le_bytes()); // DSD raw
    f.extend(2u32.to_le_bytes()); // stereo
    f.extend(2u32.to_le_bytes()); // channels
    f.extend(DSD64.to_le_bytes());
    f.extend(1u32.to_le_bytes()); // 1 bit per sample
    f.extend((len as u64 * 8).to_le_bytes()); // samples per channel
    f.extend((BLOCK as u32).to_le_bytes());
    f.extend(0u32.to_le_bytes());
    f.extend(b"data");
    f.extend((12 + data.len() as u64).to_le_bytes());
    f.extend(data);
    let path = dir.join(name);
    std::fs::write(&path, f).unwrap();
    path
}
```

(Compare the header field order with `crates/fp-decode/tests/dsd.rs` `dsf_blocks` before trusting it; that writer is known good. Fix this one to match if they differ.)

In `crates/fp-engine/tests/worker.rs`, append (reuse the file's helpers for spawning a worker and draining a consumer; read it first):

```rust
#[test]
fn a_dsd_load_fills_the_word_ring_with_the_files_bytes_and_the_pcm_ring_in_step() {
    let dir = tempfile::tempdir().unwrap();
    let left: Vec<u8> = (0..20_000u32).map(|i| (i % 251) as u8).collect();
    let right: Vec<u8> = left.iter().map(|b| !b).collect();
    let path = support::dsf_file(dir.path(), "a.dsf", &left, &right);
    let (failures, _rx) = crossbeam_channel::unbounded();
    let worker = PlayerWorker::spawn_with_dsd(
        "t", file_opener(), dsd_file_opener(), 176_400, 1_000, failures,
    )
    .unwrap();
    let (producer, mut consumer) = source_pair_dsd(20_000);
    worker.load_with(
        SourceKey(1),
        path,
        0.0,
        producer,
        LoadOptions { dsd: true, rate: Some(176_400), ..LoadOptions::default() },
    );
    wait_until(|| consumer.shared.is_eof());
    let (mut pcm, mut dsd) = (vec![0.0; 20_000], vec![0.0; 20_000]);
    let frames = consumer.pop_pair(&mut pcm, &mut dsd);
    assert_eq!(frames, 10_000, "one word frame per two bytes");
    for (k, frame) in dsd.chunks(2).take(frames).enumerate() {
        assert_eq!(sample_to_word(frame[0]), [left[2 * k], left[2 * k + 1]], "frame {k}");
        assert_eq!(sample_to_word(frame[1]), [right[2 * k], right[2 * k + 1]], "frame {k}");
    }
    assert!(pcm.iter().take(frames * 2).all(|s| s.is_finite() && s.abs() <= 1.5));
}
```

(`wait_until` polls with a deadline like `bit_perfect.rs`'s `settle`; if the file has no such helper add one there. The worker runs on its own thread, so waiting for it is not waiting for audio.)

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p fp-engine --lib dsd_pair && cargo test -p fp-engine --test worker a_dsd_load`
Expected: FAIL to compile.

- [ ] **Step 3: Implement the DSD pair**

`source.rs`: `SourceProducer` and `SourceConsumer` gain `dsd: Option<rtrb::Producer<f32>>` / `Option<rtrb::Consumer<f32>>` (`None` from `source_pair`). `source_pair_dsd` builds both rings with the same capacity. `push_pair` pushes `min(pcm frames, dsd frames, free frames of each ring)` whole frames to both, then adds them to `frames_pushed` once. `pop_pair` pops `min(dst frames, available in each)` whole frames from both (a ring whose producer pushed both in lockstep always has the same count; take the minimum anyway). `push` on a DSD producer pushes nothing and returns 0 (the worker never calls it); `pop_frames` on a DSD consumer pops the PCM ring only and discards the same number of words (keeps lockstep) — write it so, and document it.

- [ ] **Step 4: Implement the DSD source and the worker job**

`worker.rs`:

```rust
/// A DSD file for output without conversion: its raw words, and its PCM
/// conversion at the same rate for the meters and for a switch to PCM.
struct DsdFileSource {
    pcm: Box<dyn SampleSource>,
    raw: fp_decode::DsdRawReader,
    bytes: Vec<Vec<u8>>,
    pcm_queue: Vec<f32>,
    word_queue: Vec<f32>,
    pcm_done: bool,
    raw_done: bool,
}

pub fn dsd_file_opener() -> DsdOpener {
    Arc::new(|path, from_secs, word_rate| {
        let mut raw = fp_decode::DsdRawReader::open(path)?;
        if !(1..=2).contains(&raw.channels()) {
            return Err("only mono or stereo DSD goes out unchanged".to_owned());
        }
        if raw.dsd_rate() / 16 != word_rate {
            return Err(format!("word rate {word_rate} does not match the file"));
        }
        raw.seek(from_secs)?;
        let pcm = file_opener()(path, from_secs, word_rate)?;
        Ok(Box::new(DsdFileSource { pcm, bytes: vec![Vec::new(); raw.channels()], raw, … }))
    })
}
```

  `next_pair`: refill `word_queue` from `raw.next_bytes` (each pair of bytes per channel → one `word_to_sample`; mono duplicated to both channels; an odd trailing byte is padded with `DSD_SILENCE`) and `pcm_queue` from `pcm.next_block` until each holds at least one block or its reader ended; then move `n = min(len)` frames from both queues to the outputs. When one side has ended and the other has not, pad the ended side (`0.0` for PCM, `silence_sample()` for words) so the two stay equal; return `false` only when both ended and both queues are empty.

  The job: `LoadOptions` gains `/// Open the file as a DSD source (raw words beside the PCM conversion). pub dsd: bool,`. `Job` holds `source: Option<JobSource>` with `enum JobSource { Pcm(Box<dyn SampleSource>), Dsd(Box<dyn DsdSampleSource>) }`, and `pending_dsd: Vec<f32>` beside `pending`. `step` opens with `dsd_opener` when `options.dsd`, calls `next_pair`, applies the `limit_frames` cut to both buffers equally (`until_secs`), pushes with `push_pair`, and keeps the rest pending. `looped` is refused for DSD (the job fails: carts never request DSD). Failure reporting is unchanged (`WorkerFailure`). `PlayerWorker::spawn` keeps its signature and passes `dsd_file_opener()`; `spawn_with_dsd` takes it explicitly (tests). `run`/`step` get the extra opener by reference.

- [ ] **Step 5: Run the tests and the gate**

Expected: PASS; every existing worker test unchanged.

- [ ] **Step 6: Commit**

```bash
git add crates/fp-engine
git commit -m "feat(engine): DSD sources carry raw words beside their PCM conversion (O25)"
```

---

### Task 8: The mixer's DSD mode and the DoP stage

**Files:**
- Modify: `crates/fp-engine/src/mixer.rs` (`BusCommand::DsdMode`, `BusCommand::HoldAll`, `BusShared::dsd_on`, the block fill, `render_slot` for DSD slots, muting, holding, `MixerRenderer::dop`)
- Modify: `crates/fp-engine/src/bus.rs` (`MixerRenderer` construction passes `dop`)
- Test: `crates/fp-engine/tests/dsd_mixer.rs`

**Interfaces:**
- Consumes: Task 7 `source_pair_dsd`, `pop_pair`, `is_dsd`; Task 5 `silence_sample`, `DopEncoder`.
- Produces (used by Task 9):

```rust
BusCommand::DsdMode { on: bool, at_frame: u64 }
BusCommand::HoldAll { until_frame: u64 }
BusShared { …, pub dsd_on: AtomicBool }
pub struct MixerRenderer { pub mixer, pub shared, pub dop: Option<fp_backends::dsd::DopEncoder> }
```

Semantics (write them as the doc comments):
- `DsdMode` and `HoldAll` take effect at the start of the first block whose first frame is at or after `at_frame`/until `until_frame` (block granularity keeps every block wholly DSD or wholly PCM, which the DoP stage needs).
- **DSD mode on:** the block is pre-filled with `silence_sample()` instead of `0.0`. A started DSD slot *copies* its word ring into its channel pair (overwrite, no gain, no fade, no pause ramp, no volume smoothing; a pending fade or ramp is ignored; `Pause` holds at once, `Resume` continues at once, `StopAt` cuts on its frame); it meters its PCM ring at unity gain (peaks, sums, K-weighting, true peak, programme integrator: the same code as today, fed the PCM samples with `g = 1.0`); an underrun leaves the silence fill. Every non-DSD slot is consumed silently (its ring advances, it meters as usual, `misrouted` is not counted) — the HoldOthers mute and the switching window. `unaltered` is false for every slot (BP never lights during DSD; the DSD badge comes from the engine).
- **DSD mode off:** a DSD slot mixes its PCM ring with its gains like any slot (words popped and dropped).
- **HoldAll:** while a block starts before `until_frame`, no slot consumes, starts or stops: a `Start`/`StopAt` frame inside the hold takes effect at the first block after it. The fill applies (silence words in DSD mode).
- `BusShared::dsd_on` mirrors the mode for the block just rendered (set before the block is filled).
- `MixerRenderer::render`: after `mixer.render`, when `shared.dsd_on` and `dop` is `Some`, `dop.encode_in_place(out, channels)`. On a lock miss, the fill is `silence_sample()` when `dsd_on`, else `0.0`, then the same encoding. A `None` `dop` on a DSD bus is a native stream: the backend packs the words.

- [ ] **Step 1: Write the failing tests**

Create `crates/fp-engine/tests/dsd_mixer.rs` with the helpers of `tests/mixer.rs` (copy `CONFIG`, `mixer`, `full_volume`, `send`, `render` with `assert_no_alloc`, `events`, the `#[global_allocator]` block):

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing)]
//! The bus mixer's DSD mode (feedback 2 spec O25), block by block.

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use assert_no_alloc::{AllocDisabler, assert_no_alloc};
use fp_backends::Renderer;
use fp_backends::dsd::{DopEncoder, silence_sample, word_to_sample};
use fp_engine::atomic::AtomicF32;
use fp_engine::mixer::{BusCommand, Mixer, MixerConfig, MixerHandle, MixerRenderer};
use fp_engine::source::{SourceConsumer, SourceProducer, source_pair, source_pair_dsd};

#[cfg(debug_assertions)]
#[global_allocator]
static ALLOCATOR: AllocDisabler = AllocDisabler;

const CONFIG: MixerConfig = MixerConfig { volume_smoothing_frames: 1, max_commands_per_block: 64 };

fn send(h: &mut MixerHandle, c: BusCommand) {
    assert!(h.commands.push(c).is_ok());
}

fn render(m: &mut Mixer, frames: usize) -> Vec<f32> {
    let mut out = vec![0.0; frames * 2];
    assert_no_alloc(|| m.render(&mut out, 2));
    out
}

/// A DSD source of `frames` frames: words 1, 2, 3… (as samples) on the
/// left, their negatives on the right; its PCM ring holds 0.25 everywhere.
fn dsd_source(frames: usize, volume: f32) -> (SourceProducer, SourceConsumer, Arc<AtomicF32>) {
    let (mut p, c) = source_pair_dsd(frames);
    let words: Vec<f32> = (1..=frames)
        .flat_map(|i| {
            let w = word_to_sample((i >> 8) as u8, i as u8);
            [w, -w]
        })
        .collect();
    let pcm = vec![0.25; frames * 2];
    assert_eq!(p.push_pair(&pcm, &words), frames);
    (p, c, Arc::new(AtomicF32::new(volume)))
}

fn attach(h: &mut MixerHandle, slot: usize, source: SourceConsumer, volume: Arc<AtomicF32>) {
    send(h, BusCommand::Attach { slot, source, volume, first_channel: 0 });
}

#[test]
fn in_dsd_mode_the_words_are_copied_exactly_whatever_the_volume() {
    let (mut m, mut h) = Mixer::new(4, CONFIG);
    let (_p, c, v) = dsd_source(8, 0.3);
    attach(&mut h, 0, c, v);
    send(&mut h, BusCommand::DsdMode { on: true, at_frame: 0 });
    send(&mut h, BusCommand::Start { slot: 0, at_frame: 2 });
    let out = render(&mut m, 6);
    assert_eq!(out[0], silence_sample(), "before the start: DSD silence");
    assert_eq!(out[2], silence_sample());
    assert_eq!(out[4], word_to_sample(0, 1), "first word, unchanged by the 0.3 volume");
    assert_eq!(out[5], -word_to_sample(0, 1));
    assert_eq!(out[10], word_to_sample(0, 4));
    assert!(m.shared().dsd_on.load(Ordering::Acquire));
}

#[test]
fn in_dsd_mode_other_sources_are_muted_but_advance() {
    let (mut m, mut h) = Mixer::new(4, CONFIG);
    let (_p, c, v) = dsd_source(16, 1.0);
    attach(&mut h, 0, c, v);
    let (mut p2, c2) = source_pair(16);
    p2.push(&[0.5; 32]);
    send(&mut h, BusCommand::Attach { slot: 1, source: c2, volume: Arc::new(AtomicF32::new(1.0)), first_channel: 0 });
    send(&mut h, BusCommand::DsdMode { on: true, at_frame: 0 });
    send(&mut h, BusCommand::Start { slot: 0, at_frame: 0 });
    send(&mut h, BusCommand::Start { slot: 1, at_frame: 0 });
    let out = render(&mut m, 4);
    assert_eq!(out[0], word_to_sample(0, 1), "nothing summed into the DSD word");
    assert_eq!(p2.shared.frames_played(), 4, "the muted source advanced");
}

#[test]
fn the_dsd_slot_is_metered_from_its_pcm_ring() {
    let (mut m, mut h) = Mixer::new(4, CONFIG);
    let (p, c, v) = dsd_source(8, 1.0);
    attach(&mut h, 0, c, v);
    send(&mut h, BusCommand::DsdMode { on: true, at_frame: 0 });
    send(&mut h, BusCommand::Start { slot: 0, at_frame: 0 });
    render(&mut m, 8);
    assert_eq!(p.shared.peak_l.load(), 0.25, "the PCM conversion, not the words");
    assert_eq!(p.shared.measured_frames.load(Ordering::Acquire), 8);
}

#[test]
fn with_dsd_mode_off_the_dsd_slot_mixes_its_pcm_with_gain() {
    let (mut m, mut h) = Mixer::new(4, CONFIG);
    let (_p, c, v) = dsd_source(8, 0.5);
    attach(&mut h, 0, c, v);
    send(&mut h, BusCommand::Start { slot: 0, at_frame: 0 });
    let out = render(&mut m, 4);
    assert_eq!(out[0], 0.125);
    assert!(!m.shared().dsd_on.load(Ordering::Acquire));
}

#[test]
fn hold_all_freezes_every_slot_and_defers_starts_to_the_next_block() {
    let (mut m, mut h) = Mixer::new(4, CONFIG);
    let (p, c, v) = dsd_source(16, 1.0);
    attach(&mut h, 0, c, v);
    send(&mut h, BusCommand::DsdMode { on: true, at_frame: 0 });
    send(&mut h, BusCommand::Start { slot: 0, at_frame: 0 });
    render(&mut m, 4); // words 1..=4
    send(&mut h, BusCommand::HoldAll { until_frame: 8 });
    send(&mut h, BusCommand::DsdMode { on: false, at_frame: 8 });
    let held = render(&mut m, 4); // frames 4..8: held, silence words
    assert!(held.iter().all(|s| *s == silence_sample()));
    assert_eq!(p.shared.frames_played(), 4, "nothing consumed while held");
    let after = render(&mut m, 4); // frames 8..12: PCM now, from where it held
    assert_eq!(after[0], 0.25);
    assert_eq!(p.shared.frames_played(), 8);
}

#[test]
fn the_renderer_encodes_dop_only_while_the_bus_is_in_dsd_mode() {
    let (m, mut h) = Mixer::new(4, CONFIG);
    let shared = m.shared().clone();
    let mixer = Arc::new(Mutex::new(m));
    let mut r = MixerRenderer { mixer: mixer.clone(), shared: shared.clone(), dop: Some(DopEncoder::new()) };
    send(&mut h, BusCommand::DsdMode { on: true, at_frame: 0 });
    let mut out = vec![0.0; 4 * 2];
    r.render(&mut out, 2);
    let marker = |s: f32| ((s * 8_388_608.0) as i32 as u32 >> 16) & 0xFF;
    assert_eq!(out.iter().map(|s| marker(*s)).collect::<Vec<_>>(), vec![5, 5, 0xFA, 0xFA, 5, 5, 0xFA, 0xFA]);
    // A lock miss keeps the stream valid DoP: silence, markers continuing.
    let guard = mixer.lock().unwrap();
    r.render(&mut out, 2);
    drop(guard);
    assert_eq!(marker(out[0]), 5);
    assert_eq!(((out[0] * 8_388_608.0) as i32 as u32) & 0xFFFF, 0x6969);
    send(&mut h, BusCommand::DsdMode { on: false, at_frame: 0 });
    r.render(&mut out, 2);
    assert!(out.iter().all(|s| *s == 0.0), "PCM silence once DSD mode is off");
}
```

(Note: `marker` uses the sign-extended 24-bit value; `0xFA` words are negative, so the `& 0xFF` after the shift recovers the byte. If `AtomicF32::load` or `BusShared` field names differ, use the real ones.)

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p fp-engine --test dsd_mixer`
Expected: FAIL to compile.

- [ ] **Step 3: Implement in `mixer.rs`**

- `Mixer` gains `dsd_on: bool`, `pending_dsd: Option<(u64, bool)>`, `hold_until: u64`. `apply`: `DsdMode { on, at_frame }` stores `pending_dsd`; `HoldAll { until_frame }` sets `hold_until = until_frame` (the latest command wins, so a `HoldAll` at the current frame ends a hold early, as Task 9 does for native devices).
- `render`: after draining commands, if `pending_dsd` is due (`at_frame <= now`), apply it; store `self.shared.dsd_on`; fill `out` with `silence_sample()` or `0.0`; if `now < hold_until`, skip every slot (no `render_slot` call; no events), else render slots with an extra `dsd_on: bool` argument.
- `render_slot(…, dsd_on)`: for a `slot.source.is_dsd()` slot, pop with `pop_pair` into two stack chunks (`[f32; CHUNK_FRAMES * SOURCE_CHANNELS]` each — no allocation); when `dsd_on` write the words with `*o = w` and meter the PCM samples at `g = 1.0`, skipping the fade/pause/volume gain computation, finishing a pause at once; when not `dsd_on`, use the PCM samples as today. For a non-DSD slot when `dsd_on`: pop and meter as today but do not write into `out`. Keep `render_slot` readable: factor the per-sample metering into an `#[inline]` helper `measure(slot, meter, l, r, acc)` used by both paths, rather than duplicating it.
- `mark_unaltered`: when `dsd_on`, store `false` for every slot.
- `MixerRenderer` gains `pub dop: Option<fp_backends::dsd::DopEncoder>`; `render` as specified above. `bus.rs` builds it with `dop: (config.dsd == Some(DsdStream::Dop)).then(DopEncoder::new)`; every other construction (tests) adds `dop: None`.

- [ ] **Step 4: Run the tests and the gate**

Expected: PASS, including all of `tests/mixer.rs` and `tests/metering.rs` unchanged.

- [ ] **Step 5: Commit**

```bash
git add crates/fp-engine
git commit -m "feat(engine): the mixer copies DSD words and the renderer encodes DoP (O25)"
```

---

### Task 9: The engine sends DSD to bit-perfect devices

**Files:**
- Create: `crates/fp-engine/src/engine/dsd.rs`
- Modify: `crates/fp-engine/src/engine.rs` (`EngineSettings::dsd`, `PlayerTelemetry::dsd`, `start_current`, `resume`, `seek`, `pause`, `fade_out`, `stop_quick_and_release`, the starts loop, `dispatch`, `execute` for `LeaveDsd`, event translation, `is_bit_perfect`, `new_source`), `crates/fp-engine/src/engine/carts.rs` (cart and test-tone starts go through `before_start_on`), `crates/fp-engine/src/bus.rs` (`Bus::reopen_with`, `Bus::stream_dsd`, refused DSD configurations)
- Test: `crates/fp-engine/tests/dsd_output.rs`

**Interfaces:**
- Consumes: Task 3 config; Task 4 `dsd_decision`, `DsdFacts`, `DsdFallback`, events and `LeaveDsd`; Tasks 5–8.
- Produces (used by Task 11):

```rust
pub struct DsdSettings { pub modes: HashMap<BusKey, fp_model::DsdOutput>, pub mix: fp_model::DsdMix, pub silence_ms: f64 }
EngineSettings { …, pub dsd: DsdSettings }       // from_config: modes of bit-perfect devices only
PlayerTelemetry { …, pub dsd: bool }             // the current source goes out as DSD
Bus::reopen_with(&mut self, config: StreamConfig, now: Instant) -> Result<(), String>
Bus::stream_dsd(&self) -> Option<DsdStream>
```

Behaviour (each line has a test below):

1. **Start** (`start_current`, a track loaded paused on `resume`, never a cue or a cart): before `prepare_start`, `try_start_dsd(player, bus, request)` builds `DsdFacts { mode: settings.dsd.modes[bus] or Pcm, format: request.format, volume: the player's volume atomic, device_idle: !bus_sounding(bus) && no DSD tail pending }` and calls `fp_model::dsd_decision`. On `Ok(target)`: `bus.reopen_with(StreamConfig { sample_rate: target.word_rate, dsd: Some(mode), exclusive: true, ..current })`; for DoP check `bus.sample_format()` is `I24` or `I32` (else `FormatTooNarrow`: reopen PCM at the previous config); for native check `bus.stream_dsd() == Some(Native)`. On success: `reopen_waiting(bus)` (preloads re-created at the word rate), send `DsdMode { on: true, at_frame: now_frame }`, open the source with `LoadOptions { dsd: true, rate: Some(word_rate) }` (never take a PCM preload), and start it at `max(the usual start frame, now_frame + silence_frames)` so at least the silence time of DSD silence precedes it. Record `DsdBus { stream, player, entry, slot }` in `Engine::dsd_buses`. When the mixer reports the slot `Started`, push `EngineEvent::DsdStarted { player, entry, hold_others: mix == HoldOthers }`. On `Err(fallback)`: `if fallback.worth_logging() { tracing::info!(bus = ?bus, %fallback, "DSD converted to PCM") }` and continue with today's PCM path (`prepare_start`, which reopens at the PCM rate if idle).
2. **Refusals are remembered.** `Bus::reopen_with` keeps `refused_dsd: HashSet<(u32, DsdStream)>`; a refused pair is not asked again (returns `Err` at once) until the device comes back after a loss (cleared with `refused_rates`). A refused rate maps to `RateRefused(word_rate)`, any other open error to `StreamRefused`.
3. **End.** When the DSD slot finishes (`Finished`, or its source fails), the bus stays in DSD mode; the engine sets `tail_until = finish_frame + silence_frames`. In `tick`, once `now_frame >= tail_until` and no new DSD start claimed the bus: DoP → send `DsdMode { on: false, at_frame: now_frame }`; native → drop the stream, send `DsdMode { on: false }`, reopen PCM at the same rate (`reopen_with` with `dsd: None`). Remove the `DsdBus`, push `DsdEnded`. A PCM start on that bus waits (stays `WhenReady`) until the tail is over; a DSD start may reuse the bus directly if the word rate matches.
4. **Another source needs the output** — every place that sends a `BusCommand::Start` for a non-DSD slot (the starts loop in `tick`, `dispatch` for `StartNextAt`, carts, test tones, cue sources) first calls `before_start_on(bus, at_frame) -> u64`:
   - no DSD on the bus → `at_frame`;
   - `HoldOthers` → `at_frame` (the mixer mutes it);
   - `ConvertToPcm` → `switch_to_pcm(bus)`: `HoldAll { until_frame: F + S }` and `DsdMode { on: false, at_frame: F + S }` with `F = max(at_frame, now_frame)`, `S` = silence frames; native also reopens PCM at the same rate at the end of the window (the engine keeps the hold until the reopen is done, then sends `HoldAll { until_frame: now_frame }`); push `DsdEnded`; return `F + S`.
5. **`EngineAction::LeaveDsd { player }`** → `switch_to_pcm` on that player's DSD bus, if any.
6. **Pause/resume/seek of the DSD source**: pause and resume are immediate (`Pause`/`Resume` with `ramp_frames: 0`; the mixer holds and the silence fill keeps the DoP stream valid); seek replaces the source with a new DSD source at the target (same bus, still in DSD mode, `device_idle` not required since it is the same player's stream) started with no extra silence; stop and `StopNow` cut it (no de-click ramp: `fade_out`/`stop_quick_and_release` use 0 frames for a DSD-direct source) and start the tail (3).
7. **Telemetry**: `PlayerTelemetry::dsd` is true while the current source is the slot of a `DsdBus` whose bus is in DSD mode (not switched); `is_bit_perfect` returns false for it.
8. **Device loss**: the watchdog's reopen uses the bus's stored `StreamConfig` (DSD included) — check `try_open` uses `self.config` and that `reopen_with` stores the DSD flag in it; `MixerRenderer` gets a fresh `DopEncoder`.

- [ ] **Step 1: Write the failing tests**

Create `crates/fp-engine/tests/dsd_output.rs`, built on the `Rig` of `tests/bit_perfect.rs` (copy `rig`, `request`, `Rig::{act, start, settle, rate, run}` and adapt):

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing)]
//! DSD reaching bit-perfect devices unchanged (feedback 2 spec O25), end to
//! end on an Offline device: DoP frames byte for byte, silence, mixing
//! policies and fallbacks.

mod support;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use fp_backends::dsd::DsdStream;
use fp_backends::exclusive::write_samples;
use fp_backends::{AudioBackend, OfflineBackend, OfflineDevice, SampleFormat};
use fp_engine::engine::{Engine, EngineSettings};
use fp_engine::worker::file_opener;
use fp_model::{
    AudioFormat, Config, DsdDevice, DsdMix, DsdOutput, EngineAction, EngineEvent, EntryId,
    OutputDevice, PlayerId, PlayerRoutes, Route, SourceRequest, TrackId,
};
use support::{DSD64, dsf_file};

const P: PlayerId = PlayerId(1);
const Q: PlayerId = PlayerId(2);
const BLOCK: usize = 480;
const WORD_RATE: u32 = DSD64 / 16; // 176 400
const SILENCE_MS: f64 = 10.0;
const SILENCE_FRAMES: usize = 1_764; // 10 ms at 176.4 kHz

struct Rig {
    engine: Engine,
    dac: OfflineDevice,
    clock: Instant,
    dir: tempfile::TempDir,
    /// Everything `Engine::tick` returned so far.
    seen: Vec<EngineEvent>,
}

fn rig(mode: DsdOutput, mix: DsdMix, format: SampleFormat) -> Rig {
    let backend = OfflineBackend::new();
    let dac = backend.add_device("dac", 2);
    dac.set_exclusive_capable(true);
    dac.set_sample_format(format);
    let route = |player| PlayerRoutes {
        player,
        main: Some(Route { backend: "offline".into(), device: "dac".into(), first_channel: 0 }),
        cue: None,
    };
    let mut config = Config::default();
    config.outputs.backend = Some("offline".into());
    config.outputs.buffer_frames = BLOCK as u32;
    config.outputs.routes = vec![route(P), route(Q)];
    config.outputs.bit_perfect = vec![OutputDevice { backend: "offline".into(), device: "dac".into() }];
    config.outputs.dsd_output = vec![DsdDevice { backend: "offline".into(), device: "dac".into(), mode }];
    config.outputs.dsd_mix = mix;
    config.outputs.dsd_silence_ms = SILENCE_MS;
    config.tuning.gain_smoothing_ms = 0.0;
    config.tuning.prebuffer_secs = 4.0;
    config.tuning.ready_threshold_ms = 1_500.0;
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
    let mut engine = Engine::new(backends, EngineSettings::from_config(&config), file_opener());
    let clock = Instant::now();
    engine.execute(EngineAction::AddPlayer { player: P }, clock);
    engine.execute(EngineAction::AddPlayer { player: Q }, clock);
    Rig { engine, dac, clock, dir: tempfile::tempdir().unwrap(), seen: Vec::new() }
}

fn dsd64() -> Option<AudioFormat> {
    Some(AudioFormat { sample_rate: DSD64 / 32, bits: None, channels: 2, dsd_rate: Some(DSD64) })
}

/// Two seconds of recognisable bytes that never contain the idle byte.
fn pattern() -> (Vec<u8>, Vec<u8>) {
    let len = DSD64 as usize / 8 * 2;
    let left: Vec<u8> = (0..len).map(|i| match (i % 253) as u8 { 0x69 => 0x6A, b => b }).collect();
    let right: Vec<u8> = left.iter().map(|b| match !b { 0x69 => 0x6B, b => b }).collect();
    (left, right)
}

fn request(n: u64, path: PathBuf, format: Option<AudioFormat>) -> SourceRequest {
    SourceRequest { entry: EntryId(n), track: TrackId(n), path, from_secs: 0.0, format }
}

/// One rendered stereo frame as the 24-bit words the device receives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Frame { marker: u8, left: [u8; 2], right: [u8; 2] }

impl Rig {
    fn act(&mut self, action: EngineAction) { self.engine.execute(action, self.clock); }
    fn start(&mut self, player: PlayerId, request: SourceRequest) {
        self.act(EngineAction::StartCurrent { player, request });
        self.settle();
    }
    /// As in `bit_perfect.rs` (waits for the decode worker, never for audio),
    /// keeping what `tick` returns.
    fn settle(&mut self) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while self.engine.unsettled_sources() > 0 && Instant::now() < deadline {
            let events = self.engine.tick(self.clock);
            self.seen.extend(events);
            std::thread::sleep(Duration::from_millis(1));
        }
        let events = self.engine.tick(self.clock);
        self.seen.extend(events);
    }
    fn rate(&self) -> u32 { self.dac.config().unwrap().sample_rate }
    /// Renders `frames` frames and returns them decoded as DoP 24-bit words.
    fn run_dop(&mut self, frames: usize) -> Vec<Frame> {
        let mut out = Vec::new();
        for _ in 0..frames.div_ceil(BLOCK) {
            let samples = self.dac.render(BLOCK).unwrap();
            let mut bytes = vec![0u8; samples.len() * 4];
            write_samples(SampleFormat::I24, &samples, &mut bytes);
            for f in bytes.chunks(8) {
                let w = |b: &[u8]| u32::from_le_bytes(b.try_into().unwrap()) >> 8;
                let (l, r) = (w(&f[0..4]), w(&f[4..8]));
                assert_eq!(l >> 16, r >> 16, "one marker per frame");
                out.push(Frame { marker: (l >> 16) as u8, left: [(l >> 8) as u8, l as u8], right: [(r >> 8) as u8, r as u8] });
            }
            self.clock += Duration::from_secs_f64(BLOCK as f64 / f64::from(self.rate()));
            let events = self.engine.tick(self.clock);
            self.seen.extend(events);
        }
        out
    }
    fn events(&mut self) -> Vec<EngineEvent> {
        std::mem::take(&mut self.seen)
    }
}

fn assert_valid_dop(frames: &[Frame]) {
    for pair in frames.windows(2) {
        assert!(matches!(pair[0].marker, 0x05 | 0xFA), "{:?}", pair[0]);
        assert_ne!(pair[0].marker, pair[1].marker, "markers alternate");
    }
}

const IDLE: [u8; 2] = [0x69, 0x69];

#[test]
fn a_dsd_track_goes_out_as_dop_byte_for_byte_after_the_silence() {
    let mut r = rig(DsdOutput::Dop, DsdMix::ConvertToPcm, SampleFormat::I24);
    let (left, right) = pattern();
    let path = dsf_file(r.dir.path(), "a.dsf", &left, &right);
    r.start(P, request(1, path, dsd64()));
    assert_eq!(r.rate(), WORD_RATE);
    assert_eq!(r.dac.config().unwrap().dsd, Some(DsdStream::Dop));
    let frames = r.run_dop(WORD_RATE as usize / 4);
    assert_valid_dop(&frames);
    let first = frames.iter().position(|f| f.left != IDLE).unwrap();
    assert!(first >= SILENCE_FRAMES, "at least the silence time first: {first}");
    assert!(frames[..first].iter().all(|f| f.left == IDLE && f.right == IDLE));
    for (k, f) in frames[first..].iter().enumerate() {
        assert_eq!(f.left, [left[2 * k], left[2 * k + 1]], "frame {k}");
        assert_eq!(f.right, [right[2 * k], right[2 * k + 1]], "frame {k}");
    }
    assert!(r.engine.telemetry(P).dsd);
    assert!(!r.engine.telemetry(P).bit_perfect);
    assert!(r.events().contains(&EngineEvent::DsdStarted { player: P, entry: EntryId(1), hold_others: false }));
}

#[test]
fn the_meter_reads_the_pcm_conversion_not_the_words() {
    // All-idle bytes: the words read -1.7 dBFS as samples, the conversion is silence.
    let mut r = rig(DsdOutput::Dop, DsdMix::ConvertToPcm, SampleFormat::I24);
    let idle = vec![0x69u8; DSD64 as usize / 8];
    let path = dsf_file(r.dir.path(), "idle.dsf", &idle, &idle);
    r.start(P, request(1, path, dsd64()));
    r.run_dop(WORD_RATE as usize / 4);
    let input = r.engine.take_meter_input(P);
    assert!(input.frames > 0);
    assert!(input.peak.iter().all(|p| *p < 1e-3), "{:?}", input.peak);
}

#[test]
fn the_end_sends_silence_then_leaves_dsd_mode() { /* render past the end of a 0.1 s file: idle frames ≥ SILENCE_FRAMES with valid markers, then samples are PCM zeros (marker 0); DsdEnded reported; telemetry.dsd false */ }

#[test]
fn pause_and_seek_keep_every_frame_valid_dop() { /* start, run 0.05 s, Pause → run: all idle words, markers valid; Resume → data continues from where it paused; Seek to 1.0 s → data resumes at byte round(1.0 × DSD64 / 8) even; assert_valid_dop over the whole capture */ }

#[test]
fn hold_others_mutes_a_second_player_on_the_same_device() { /* mix HoldOthers; P plays DSD; Q starts a WAV (support::indexed_wav at 176.4 kHz) on the same device; the DoP bytes stay exactly the file's; Q's telemetry position advances; DsdStarted had hold_others: true */ }

#[test]
fn convert_to_pcm_switches_after_the_silence_when_another_source_starts() { /* mix ConvertToPcm; P plays DSD; Q starts a WAV; capture: valid DoP data, then ≥ SILENCE_FRAMES of idle frames, then non-DoP samples (decode with write_samples: the marker byte is no longer 0x05/0xFA consistently; simplest: assert the raw f32 samples are not equal to dop_sample(...) and that telemetry(P).dsd is false); DsdEnded { P, 1 } reported; P's position kept counting from where it held (position after switch ≈ position before + rendered time − silence) */ }

#[test]
fn leave_dsd_switches_to_pcm() { /* P plays DSD; act(LeaveDsd { player: P }); after the silence the bus is PCM; telemetry(P).dsd false */ }

#[test]
fn a_busy_device_plays_dsd_converted() { /* Q plays a WAV first; then P starts the DSF → rate stays, dsd None, telemetry(P).dsd false, P audible */ }

#[test]
fn a_refused_word_rate_plays_converted_and_is_not_asked_again() {
    let mut r = rig(DsdOutput::Dop, DsdMix::ConvertToPcm, SampleFormat::I24);
    r.dac.refuse_rate(WORD_RATE);
    let (left, right) = pattern();
    let path = dsf_file(r.dir.path(), "a.dsf", &left, &right);
    r.start(P, request(1, path.clone(), dsd64()));
    assert_eq!(r.rate(), DSD64 / 32, "the PCM conversion's rate, as before O25");
    assert_eq!(r.dac.config().unwrap().dsd, None);
    let attempts = r.dac.open_attempts();
    r.act(EngineAction::StopNow { player: P });
    r.settle();
    r.start(P, request(2, path, dsd64()));
    assert_eq!(r.dac.open_attempts(), attempts, "the refused pair is not asked again");
}

#[test]
fn a_sixteen_bit_device_plays_dsd_converted() { /* rig with SampleFormat::I16 → FormatTooNarrow (the Offline open refuses DoP on I16) → PCM at 88.2 kHz, dsd None */ }

#[test]
fn native_on_a_device_without_it_plays_converted() { /* rig(DsdOutput::Native, …) without set_native_dsd → PCM; with set_native_dsd(true) → dac.config().dsd == Some(Native), rate == WORD_RATE, rendered samples are the raw words (sample_to_word gives the file's bytes) */ }

#[test]
fn a_multichannel_dsd_file_plays_converted() { /* format channels: 6 in the request → PCM; no DSD stream */ }

#[test]
fn a_volume_below_unity_plays_converted() { /* act(SetVolume { player: P, volume: 0.5 }) before the start → PCM */ }

#[test]
fn pcm_mode_plays_converted_as_before() { /* rig(DsdOutput::Pcm, …) → rate 88 200, dsd None, no DsdStarted */ }

#[test]
fn a_device_lost_during_dop_comes_back_as_dop() { /* start DSD, run, dac.unplug(), tick past the watchdog, dac.replug(), tick past reconnect_interval_ms; dac.config().dsd == Some(Dop); run_dop: assert_valid_dop */ }

#[test]
fn a_dsd_track_loaded_paused_starts_direct_on_resume() { /* act(LoadPaused { … DSF request at 0.5 s … }); nothing rendered is non-zero (no stream change yet, nothing on air); act(Resume { player: P }); settle; rate == WORD_RATE; run_dop: idle prefix ≥ SILENCE_FRAMES then the bytes from byte round(0.5 × DSD64 / 8) */ }
```

Write every test body fully before running; each comment says exactly what the test asserts, and the two complete tests above show the style. Events come from `Engine::tick`'s return value (kept in `Rig::seen`). `EngineAction::LoadPaused { player, request }` restores a paused track; the player's volume is `EngineAction::SetVolume { player, volume }`.

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p fp-engine --test dsd_output`
Expected: FAIL to compile (`EngineSettings::dsd`, `telemetry.dsd`…), then, once the types exist, the behaviour tests fail.

- [ ] **Step 3: Implement**

Follow "Behaviour" 1–8 above. Put the DSD orchestration in `crates/fp-engine/src/engine/dsd.rs` as `impl Engine` blocks (like `engine/carts.rs`): `try_start_dsd`, `before_start_on`, `switch_to_pcm`, `end_dsd_tails` (called from `tick`), `dsd_slot_started` (from the event translation), `silence_frames(bus)`. Keep `engine.rs` changes to the call sites. `Bus::reopen_with` generalises `reopen_at` (which becomes `reopen_with` with only the rate changed, keeping its tests green): same "stream = None, follow_rate, try_open, restore on failure" sequence, plus the DSD refusal set. Every new log line is on the conductor thread (never in the mixer).

- [ ] **Step 4: Run the tests and the gate**

Expected: PASS, and `tests/bit_perfect.rs`, `tests/engine.rs`, `tests/conductor.rs`, `tests/cartwall.rs` unchanged.

- [ ] **Step 5: Commit**

```bash
git add crates/fp-engine
git commit -m "feat(engine): DSD reaches bit-perfect devices as DoP or native DSD (O25)"
```

---

### Task 10: Native DSD through ALSA (Linux)

**Files:**
- Create: `crates/fp-backends/src/alsa_dsd.rs` (`#[cfg(target_os = "linux")]`)
- Modify: `crates/fp-backends/Cargo.toml` (Linux: `alsa = "0.11"`, and the real-time priority crate cpal's `realtime-dbus` already pulls — check with `cargo tree -p fp-backends -e features -i audio_thread_priority` and use the same version and features), `crates/fp-backends/src/lib.rs`, `crates/fp-backends/src/cpal_backend.rs` (probe in `enumerate_devices`, a probe cache, native open in `open_on_host`)
- Test: `crates/fp-backends/src/alsa_dsd.rs` unit tests (pure parts), `crates/fp-backends/tests/conformance.rs` (an `#[ignore]`d on-device native check)

**Interfaces:**
- Consumes: Task 5 `NativeDsdFormat`, `choose_native_format`, `pack_native`, `StreamConfig::dsd`, `OutputStream::dsd`.
- Produces: `DeviceInfo::native_dsd` true for ALSA `hw:` devices that accept a DSD format; `open_output` with `dsd: Some(Native)` on such a device.

Pure parts (unit-tested here):

```rust
/// The ALSA PCM name of a cpal ALSA device id (`alsa:hw:CARD=D,DEV=0` → `hw:CARD=D,DEV=0`); `None` for anything but `hw:`.
pub(crate) fn pcm_name(device_id: &str) -> Option<&str>;
pub(crate) fn alsa_format(format: NativeDsdFormat) -> alsa::pcm::Format;   // DSDU32BE …
/// Word frames to render per device period: even, and at least 2.
pub(crate) fn words_per_period(format: NativeDsdFormat, device_period: usize) -> usize;
```

- [ ] **Step 1: Write the failing tests**

In `alsa_dsd.rs` `mod tests`:

```rust
    #[test]
    fn only_hw_devices_have_a_pcm_name() {
        assert_eq!(pcm_name("alsa:hw:CARD=D,DEV=0"), Some("hw:CARD=D,DEV=0"));
        assert_eq!(pcm_name("hw:CARD=D,DEV=0"), Some("hw:CARD=D,DEV=0"));
        assert_eq!(pcm_name("alsa:plughw:CARD=D,DEV=0"), None);
        assert_eq!(pcm_name("alsa:default"), None);
    }

    #[test]
    fn formats_map_to_alsa() {
        use alsa::pcm::Format;
        assert_eq!(alsa_format(NativeDsdFormat::U32Be), Format::DSDU32BE);
        assert_eq!(alsa_format(NativeDsdFormat::U8), Format::DSDU8);
    }

    #[test]
    fn a_period_renders_an_even_number_of_word_frames() {
        assert_eq!(words_per_period(NativeDsdFormat::U32Be, 512), 1024);
        assert_eq!(words_per_period(NativeDsdFormat::U16Le, 511), 510);
        assert_eq!(words_per_period(NativeDsdFormat::U8, 3), 2);
    }
```

In `crates/fp-backends/tests/conformance.rs`, an `#[ignore]` test `native_dsd_on_a_real_device` that, when `FAUSTE_NATIVE_DSD_DEVICE` names a cpal ALSA id, opens it with `dsd: Some(Native)` at 176 400 and renders DSD silence for one second; it skips with a note when the variable is unset (the maintainer's on-device check; never run in CI).

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p fp-backends --lib alsa_dsd`
Expected: FAIL to compile.

- [ ] **Step 3: Implement**

- `probe(pcm_name) -> Result<Option<NativeDsdFormat>, String>`: `alsa::PCM::new(name, alsa::Direction::Playback, true)` (non-blocking, so a busy device fails at once), `alsa::pcm::HwParams::any(&pcm)`, then `choose_native_format(|f| hwp.test_format(alsa_format(f)).is_ok())`. Any error is `Err` (busy, not found).
- `CpalBackend` gains `native_probe: Mutex<HashMap<String, NativeDsdFormat>>` (Linux only). In `enumerate_devices`, for the `alsa` host and a `pcm_name`, call `probe`: `Ok(Some(f))` → store and `native_dsd = true`; `Ok(None)` → remove and false; `Err` → the cached value (a busy device keeps its last result). Enumeration already runs on helper threads (`fp-device-scan`), never on the UI or the callback; still bound the probe: it opens non-blocking.
- `open(pcm_name, config, renderer, errors) -> Result<Box<dyn OutputStream>, BackendError>`: spawns `fp-alsa-dsd`, which opens the PCM (blocking), sets `RWInterleaved`, the cached (or freshly probed) format, `config.channels`, `format.device_rate(config.sample_rate)` exactly (`set_rate(.., ValueOr::Nearest)` then check it is exact, else refuse), a period near `config.buffer_frames` scaled to device frames, and `prepare`. It reports success or the error to `open` through a bounded channel (as `wasapi_exclusive.rs` does, with the same 5 s timeout → `Unsupported`). Then it promotes itself to real-time priority (same crate as cpal), allocates its two buffers once (`words: Vec<f32>` of `words_per_period × channels`, `bytes: Vec<u8>` of the period in bytes) and loops until the stop flag: `renderer.render(&mut words, channels)`, `pack_native`, `pcm.io_bytes().writei(&bytes)`; an `EPIPE` underrun → `errors.report(StreamErrorKind::Xrun)` and `pcm.prepare()`; any other error → `DeviceLost` and exit. The stream struct returns `config()`, `sample_format() = SampleFormat::I32` (documented: the container; nothing reads it for native) and `dsd() = Some(DsdStream::Native)`; `Drop` sets the flag and joins.
- `open_on_host`: `Some(DsdStream::Native)` on the `alsa` host with a `pcm_name` and `config.exclusive` → `alsa_dsd::open`; anything else → `Unsupported`.
- `cargo deny check` (new direct dependency).

- [ ] **Step 4: Run the tests, `cargo deny check` and the gate**

Expected: PASS. On a machine without a DSD DAC nothing else can be verified; say so in the commit body.

- [ ] **Step 5: Commit**

```bash
git add crates/fp-backends Cargo.lock
git commit -m "feat(backends): native DSD output through ALSA on Linux (O25)"
```

---

### Task 11: Settings, badge, fader and notice

**Files:**
- Modify: `crates/fp-app/src/ui/settings.rs` (`bit_perfect`: a DSD mode combo per device that is on; a DSD mix row; `offered_dsd_modes`), `crates/fp-app/src/ui/player.rs` (badge text through `bp_badge`, the DSD notice badge, the fader's tooltip), `crates/fp-app/src/ui/view.rs` (`PlayerView::dsd`, `PlayerView::dsd_holds_others`), both locales
- Test: `crates/fp-app/src/ui/settings.rs` unit test, `crates/fp-app/tests/settings.rs`, `crates/fp-app/tests/view.rs` or a new `crates/fp-app/tests/dsd_ui.rs`

**Interfaces:**
- Consumes: Task 3 config; Task 4 `bp_badge`, `BpBadge`, `dsd_holds_others`; Task 5 `DeviceInfo::native_dsd`; Task 9 `PlayerTelemetry::dsd`.
- Produces: `fn offered_dsd_modes(info: Option<&DeviceInfo>, os: &str, configured: DsdOutput) -> Vec<DsdOutput>` (in `settings.rs`).

Strings (en-US / es-ES), added next to the bit-perfect ones:

```ftl
settings-dsd-mode = DSD: { $device }
settings-dsd-pcm = Convert to PCM
settings-dsd-dop = DoP
settings-dsd-native = Native DSD
settings-dsd-hint = DSD tracks reach the device unchanged when nothing else plays on it and the player's volume is 100 %. DoP works with most DSD-capable converters; native DSD needs a converter whose driver reports a DSD format (Linux).
settings-dsd-mix = When another source needs a DSD output
settings-dsd-mix-convert = Continue the DSD track as PCM
settings-dsd-mix-hold = Keep DSD and mute the other sources
badge-dsd = DSD
tip-dsd-on = DSD reaches the device unchanged
badge-dsd-hold = Others muted
tip-dsd-hold = DSD plays on this output: other sources routed to it are muted until the track ends.
tip-volume-dsd = DSD plays at 100 %. Moving the fader continues the track as PCM.
```

```ftl
settings-dsd-mode = DSD: { $device }
settings-dsd-pcm = Convertir a PCM
settings-dsd-dop = DoP
settings-dsd-native = DSD nativo
settings-dsd-hint = Las pistas DSD llegan al dispositivo sin cambios cuando no suena nada más en él y el volumen del reproductor está al 100 %. DoP funciona con la mayoría de convertidores con DSD; el DSD nativo necesita un convertidor cuyo controlador declare un formato DSD (Linux).
settings-dsd-mix = Cuando otra fuente necesita una salida DSD
settings-dsd-mix-convert = Seguir la pista DSD en PCM
settings-dsd-mix-hold = Mantener el DSD y silenciar las demás fuentes
badge-dsd = DSD
tip-dsd-on = El DSD llega al dispositivo sin cambios
badge-dsd-hold = Otras silenciadas
tip-dsd-hold = Suena DSD en esta salida: las demás fuentes dirigidas a ella están silenciadas hasta que acabe la pista.
tip-volume-dsd = El DSD suena al 100 %. Si mueves el fader, la pista sigue en PCM.
```

- [ ] **Step 1: Write the failing tests**

`settings.rs` `mod tests`:

```rust
    fn info(exclusive: bool, native: bool) -> DeviceInfo {
        DeviceInfo {
            id: fp_backends::DeviceId("hw:0".into()),
            name: "DAC".into(),
            detail: None,
            channels: 2,
            sample_rates: vec![(44_100, 768_000)],
            buffer_frames: None,
            exclusive_capable: exclusive,
            rate_switching: exclusive,
            native_dsd: native,
        }
    }

    #[test]
    fn only_the_modes_the_device_can_take_are_offered() {
        use fp_model::DsdOutput::{Dop, Native, Pcm};
        assert_eq!(offered_dsd_modes(Some(&info(true, true)), "linux", Pcm), vec![Pcm, Dop, Native]);
        assert_eq!(offered_dsd_modes(Some(&info(true, true)), "windows", Pcm), vec![Pcm, Dop]);
        assert_eq!(offered_dsd_modes(Some(&info(true, false)), "linux", Pcm), vec![Pcm, Dop]);
        assert_eq!(offered_dsd_modes(Some(&info(false, false)), "linux", Pcm), vec![Pcm]);
        assert_eq!(offered_dsd_modes(None, "linux", Pcm), vec![Pcm], "unplugged");
        assert_eq!(
            offered_dsd_modes(None, "linux", Native),
            vec![Pcm, Native],
            "a configured mode stays visible so it can be changed back"
        );
    }
```

`crates/fp-app/tests/settings.rs` (reuse `outputs_with`, which takes the bit-perfect list; the Offline device must be exclusive-capable — read how the existing bit-perfect UI test sets that up):

```rust
#[test]
fn a_bit_perfect_device_offers_its_dsd_modes_and_the_choice_updates_the_config() {
    // A bit-perfect Offline device that is exclusive-capable, without native DSD.
    let (mut h, fake) = outputs_with(vec![fp_model::OutputDevice { backend: "offline".into(), device: "dac".into() }]);
    h.get_by_label("Convert to PCM").click();
    h.run_steps(2);
    assert!(h.query_by_label("DoP").is_some());
    assert!(h.query_by_label("Native DSD").is_none(), "the device reports no DSD format");
    h.get_by_label("DoP").click();
    h.run_steps(2);
    let c = fake.last_config().unwrap(); // use the Fake's real accessor for SetConfig
    assert_eq!(c.outputs.dsd_output_for("offline", "dac"), fp_model::DsdOutput::Dop);
}

#[test]
fn the_dsd_mix_choice_updates_the_config() { /* click "Keep DSD and mute the other sources" → dsd_mix == HoldOthers */ }
```

UI badge tests (in `crates/fp-app/tests/dsd_ui.rs`, harness from `support`): with telemetry `dsd: true` the header shows "DSD" (not "BP") and its tooltip `tip-dsd-on`; with `player.dsd = Some(DsdOnAir { hold_others: true, .. })` on the current playing entry the "Others muted" badge shows; the fader's tooltip contains "DSD plays at 100 %" while `telemetry.dsd`. Read how existing tests feed telemetry to the harness (`grep -rn "bit_perfect: true" crates/fp-app/tests`) and do the same.

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p fp-app --lib only_the_modes && cargo test -p fp-app --test settings dsd && cargo test -p fp-app --test dsd_ui`
Expected: FAIL.

- [ ] **Step 3: Implement**

- `offered_dsd_modes`: `Pcm` always; `Dop` if `info.exclusive_capable`; `Native` if `os == "linux" && info.native_dsd`; plus `configured` if it is not already in the list (inserted in enum order).
- In `bit_perfect(…)`, after each device's switch row, when the device is on (`listed.contains(device)`), add a row `t.tr_args("settings-dsd-mode", device)` with a combo (the same combo widget the Settings window uses elsewhere; `grep -n "ComboBox" crates/fp-app/src/ui/settings.rs`) over `offered_dsd_modes(info, std::env::consts::OS, current)`, labelled with the three strings; a change sends `update(scene, |c| { c.outputs.dsd_output.retain(|d| (d.backend, d.device) != key); if mode != Pcm { c.outputs.dsd_output.push(DsdDevice { … mode }) } })`. After the device list, when `chosen` is not empty, the hint label and the DSD mix row (two-choice combo). Nothing here enumerates devices (it reads `st.backends`, filled by the helper thread).
- `view.rs`: `PlayerView` gains `dsd: bool` (from telemetry, like `bit_perfect`) and `dsd_holds_others: bool` (`fp_model::dsd_holds_others(state, player)`).
- `player.rs`: the badge tile uses `fp_model::bp_badge(pv.bit_perfect, pv.dsd)`: `Off` → "BP" dimmed with `tip-bp-off`, `Pcm` → "BP" accent with `tip-bp-on`, `Dsd` → `badge-dsd` accent with `tip-dsd-on` (widen the tile to fit "DSD" if needed; keep the layout's fixed widths from plan 12). Next to the entry notice, when `pv.dsd_holds_others`, an `outlined_with_tip` badge `badge-dsd-hold` / `tip-dsd-hold` (amber, like the entry notices). In `meter_column`, the fader tooltip is `tip-volume-dsd` while `telemetry.dsd`; the fader stays enabled (the model turns a move into a switch to PCM).

- [ ] **Step 4: Run the tests and the gate**

Expected: PASS, including `tests/i18n.rs`.

- [ ] **Step 5: Commit**

```bash
git add crates/fp-app
git commit -m "feat(ui): DSD mode per bit-perfect device, the DSD badge and the muted notice (O25)"
```

---

### Task 12: DSD documentation (user and technical)

**Files:**
- Modify: `docs/user/bit-perfect.md`, `docs/user/settings.md`, `docs/user/getting-started.md` (the DSD sentence), `docs/user/troubleshooting.md`, `docs/technical/backends.md`, `docs/technical/audio-engine.md`, `docs/technical/decoding.md`, `docs/technical/threading-and-realtime.md`, `docs/technical/persistence.md` (the three `outputs` fields and their defaults/ranges), `docs/technical/testing.md` (the new test files), `docs/technical/remote-api.md` (if the track `format` object now carries `dsd_rate`: check `crates/fp-remote` serialisation), `README.md` (formats line: "DSD plays converted to PCM, or unchanged on bit-perfect devices as DoP or native DSD")

**Interfaces:** none (docs only).

- [ ] **Step 1: User guide, `bit-perfect.md`**

Add a section "## DSD" after "What happens on a bit-perfect device", covering, in the guide's plain style: the three modes (Convert to PCM, the default; DoP; Native DSD, Linux only, for converters whose driver reports a DSD format) and where to set them (next to the device's bit-perfect switch; only the modes the device can take are offered; restart to apply); when DSD reaches the device unchanged (bit-perfect device, nothing else playing on it, volume 100 %, mono or stereo DSD, the device accepts the rate: DoP needs the DSD rate ÷ 16, 176.4 kHz for DSD64 and 705.6 kHz for DSD256, and a 24- or 32-bit format) and that otherwise the track is converted and the log says why; the badge reads **DSD**; the meters show the level of the PCM conversion; the fader at 100 % (moving it continues the track as PCM); a fade stop stops at once; the DSD silence at start, stop and switch (`outputs.dsd_silence_ms`, 200 ms by default, in the configuration file); the mixing setting with both choices and what each does to the next track, carts and other players, and the "Others muted" notice. Update the BP badge list ("DSD is converted, so it never is" → "DSD shows **DSD** instead when it goes out unchanged"). Extend "Checking it yourself" with the maintainer's hardware checks listed in this plan ("What is verified here…").

If the maintainer chose O34 option (c), add a short "Readings above 0 dBFS" paragraph to the meters part of `docs/user/players.md` from the audit's §7.

- [ ] **Step 2: Technical docs**

- `backends.md`: `StreamConfig::dsd`, `DeviceInfo::native_dsd`, `OutputStream::dsd`; the DoP rule (24- or 32-bit integer, `choose_dop_sample_format`); the ALSA native stream (`alsa_dsd.rs`: probe, cache, thread, formats, rates, xrun handling) and that it is the one place using the `alsa` crate directly; Null refuses DSD; Offline hooks `set_sample_format`, `set_native_dsd`.
- `audio-engine.md`: a section "### DSD buses" under "Rates and bit-perfect buses": the word stream, `source_pair_dsd`, the mixer's DSD mode (`DsdMode`, `HoldAll`, copy, fill, mute, metering from the PCM ring), the DoP stage in `MixerRenderer`, the engine's decision (`dsd_decision`, facts, fallbacks and logging), silence and tails, the two mix policies, `LeaveDsd`, telemetry, refusals, device loss.
- `decoding.md`: `DsdRawReader` and `FileDecoder::dsd_rate`, the shared chunk reader.
- `threading-and-realtime.md`: the `fp-alsa-dsd` thread row; the DoP encoder and native packing on the real-time path (no allocation).

- [ ] **Step 3: Check and commit**

Run the gate (docs-only commits still run it) and `scripts/check-commits.sh origin/master`.

```bash
git add docs README.md
git commit -m "docs: DSD output on bit-perfect devices (O25)"
```

---

### Task 13: Spec "As built", roadmap and README

**Files:**
- Modify: `docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md` (status line; §11 "As built"), `docs/superpowers/plans/2026-10-01-operator-feedback-2-roadmap.md` (row 10 → done; "Notes for the later plans" for any "too large" audit rows), `README.md` (roadmap/features, if it lists plan 10 or DSD as converted only), this plan (mark deviations)

- [ ] **Step 1: Spec**

Status line: "Plans 1 to 10, 12 and 13 are built". Append to §11 a "- **As built.**" list in the style of §14's: the audit (link to `docs/technical/audio-path-audit.md`, the ids of the defects fixed in this plan with one line each, the "too large" items moved to the roadmap); O34 (the maintainer's choice and what was built, or "no change; documented"); O27 (always listed, last; `preferred_backend` ranks it last; `main.rs` honours a configured `null`); O25 (the decisions of this plan as built: the settings in `outputs`, restart, the word stream, DoP and native, the decision facts, silence, the two policies, `LeaveDsd`, fade stop, analysis version 7, the manual hardware checks outstanding). Record every deviation from this plan the ledger holds.

- [ ] **Step 2: Roadmap and README**

Roadmap row 10: `done`. README: the formats sentence (if Task 12 did not already) and any roadmap entry for plan 10.

- [ ] **Step 3: Gate and commit**

```bash
git add docs README.md
git commit -m "docs: plan 10 as built, audio path"
```

### Deviations as built

Recorded from the ledger; the spec's section 11 "As built" has the same list in context.

- Ruling: O34 = (c) no change, documented — the maintainer's choice — none.
- Ruling: Tasks 1.1 to 1.6 added for A1 to A8, A3 to A5 grouped; A10 to the roadmap — the audit's probes confirmed them — a larger review surface for 1.3.
- Ruling: A1 ramps only starts at `from_secs > 0` — the audit's proposal — a rare click on a file whose first sample is non-zero.
- Ruling: A2 also covers a stop without a next source at a file's end, with a tolerance of two frames past the stop frame — same dip, and resampling rounds — a tiny click if the last two frames are loud.
- Ruling: A8 counts on the device threads and logs from the conductor; `fp-backends` does not log — CLAUDE.md rule 5.
- Task 2: the unused `configured` parameter of the backend list was dropped.
- Task 4: `start_or_crossfade` takes `held`, because the hold was read after `advance_to` cleared `dsd`; a fade stop on DSD delegates to `stop`.
- Task 8: the pending DSD mode switches are a fixed ordered queue of four (overflow drops the oldest and counts a dropped event), instead of one slot where the last wins.
- Task 9: `HoldAll` gains `from_frame`; a PCM start during a DSD tail starts at the tail's end after one block of hold; a dispatch shifts the whole transition by the switch delay; the Offline backend gains a `set_dop_any_format` knob.
- Ruling: after a device loss DSD continues only on an exclusive device that passes the I24/I32 check, else PCM; a bus with a pending switch to PCM is busy; a reconnect restores DSD mode only if it was on, and reports the device lost when it cannot open it.
- Task 10: ALSA opens non-blocking and waits with a timeout, instead of a blocking open — a busy device must not hold the output thread.
- Task 6: a DSF header with an absurd channel count is rejected (it used to abort on a huge allocation).
- Extra: a real-file end-to-end test (`crates/fp-engine/tests/dsd_real_music.rs`, `#[ignore]`d), byte-exact DoP and native on a DSD64 file.
- Outstanding: DoP on a real converter; native DSD on a real device (`FAUSTE_NATIVE_DSD_DEVICE`, `native_dsd_on_a_real_device`).

---

## Self-review notes (for the coordinator)

- Spec coverage: O26 → Task 1 + "Audit follow-ups"; O34 → Task 1 §7 + maintainer choice (+ Task 1.O34 if chosen, + Task 12 text); O27 → Task 2; O25 settings → Task 3, rules → Task 4, DoP → Tasks 5/8/9, native → Tasks 5/9/10, silence → Tasks 8/9, meters → Tasks 7/8/9, fader/gain at unity → Tasks 4/9/11, BP reads DSD → Tasks 4/9/11, mixing policies → Tasks 4/8/9, notice → Tasks 4/11, user guide → Task 12; global docs → Task 13.
- Type names used across tasks: `DsdOutput`, `DsdMix`, `DsdDevice`, `DsdFacts`, `DsdTarget`, `DsdStreamMode`, `DsdFallback`, `DsdOnAir`, `BpBadge` (fp-model); `DsdStream`, `NativeDsdFormat`, `DopEncoder` (fp-backends); `DsdRawReader` (fp-decode); `source_pair_dsd`, `push_pair`, `pop_pair`, `DsdSampleSource`, `DsdOpener`, `dsd_file_opener`, `LoadOptions::dsd`, `BusCommand::{DsdMode, HoldAll}`, `BusShared::dsd_on`, `MixerRenderer::dop`, `DsdSettings`, `PlayerTelemetry::dsd`, `Bus::reopen_with`, `Bus::stream_dsd` (fp-engine); `EngineEvent::{DsdStarted, DsdEnded}`, `EngineAction::LeaveDsd`, `PlayerState::dsd`, `AudioFormat::dsd_rate`, `RestartReason::DsdOutput`.
