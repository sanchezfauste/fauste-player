# Audio path audit (feedback 2, O26)

Written against commit `84102c7`. Each claim cites a test or a line of code.
Line numbers are those of that commit. Third-party code is cited from the
locked versions (`Cargo.lock`): cpal 0.18.2, dasp_sample 0.11.0, rubato
5.0.0, symphonia 0.6.1, under `~/.cargo/registry/src/*/`.

The probes added for this audit are in
`crates/fp-engine/tests/audio_path_audit.rs` and
`crates/fp-backends/tests/audio_path_audit.rs`. A probe that demonstrates a
confirmed defect is `#[ignore]`d with the finding's id; run them with
`cargo test -p fp-engine --test audio_path_audit -- --include-ignored` (and
the same for `fp-backends`). The level probes play constant (DC) sources at
0.5: any step between two output samples is then a step in gain, which is a
click on real audio. A 5 ms de-click ramp at 48 kHz moves 0.5 by about 0.002
per sample; the probes allow 0.01.

The path, in order: `FileDecoder` (`crates/fp-decode/src/lib.rs`) decodes to
interleaved stereo `f32` at the file rate; the player's worker
(`crates/fp-engine/src/worker.rs`) resamples it with `StreamResampler`
(`resample.rs`) and pushes it into the source's ring (`source.rs`); the bus
mixer (`mixer.rs`) sums the sources on the device's real-time thread,
applying ramps (`ramp.rs`) and measuring the meters; the backend
(`crates/fp-backends`) converts the `f32` block to the device format.

## 1. Decoding

### 1.1 Errors in the middle of a file

What the code does:

- **symphonia formats.** A corrupt packet is skipped
  (`crates/fp-decode/src/symph.rs:211`, `Err(Error::DecodeError(_)) =>
  continue`): the audio jumps over the missing packet. A truncated file
  (`UnexpectedEof`) ends as a normal end of stream (`symph.rs:201-202`). Any
  other error is returned (`symph.rs:204`).
- **WavPack and Monkey's Audio** return the error of the block that fails
  (`wavpack.rs:162`, `ape.rs:116`).
- **The worker** contains errors and panics to the source
  (`worker.rs:307-321`): it marks the source `failed` and `ready` and sends
  one `WorkerFailure`.
- **The engine** lets a current source that has buffered audio play it out
  (`engine.rs:1563-1569`, `c.failed = true`); the mixer finishes the source
  when its ring drains (`mixer.rs:672`, `mixer.rs:732-737`) and the engine
  then reports `SourceFailed` once (`engine.rs:1716-1726`). A failure with
  nothing buffered is reported at once (`engine.rs:1571-1593`). The model
  skips to the next entry in Continuous mode
  (`crates/fp-model/tests/transitions.rs`
  `a_failing_current_source_skips_to_next_in_continuous_mode`).

Evidence: `tests/review_fixes.rs`
`a_failing_current_source_plays_its_buffer_before_reporting`; the probe
`a4_a_source_that_fails_mid_file_ends_without_a_step` (one `SourceFailed`);
`a_truncated_wav_ends_as_a_normal_end_not_a_failure` (an interrupted
download reports `ReachedEnd`, never `SourceFailed`); `tests/decode.rs`
`a_truncated_file_plays_what_it_has_and_ends_cleanly`.

The cut itself has no ramp: the last buffered sample is followed by silence.
The probe `a4_a_source_that_fails_mid_file_ends_without_a_step` measures a
step of 0.5 (the full level) at the cut. A truncated file ends the same way,
as does the jump over a skipped packet.

Verdict: the handling is right (play the buffer, report once, skip), but the
end of a failed source is a click: **A4**. Skipped packets and truncated
files are accepted degradations of damaged files (rule 9): **A16**.

### 1.2 Gapless starts and ends

- **Decoder delay and padding.** Lossy files decode to the length their
  container declares (`crates/fp-decode/tests/gapless.rs`
  `lossy_files_decode_to_their_declared_length`, Vorbis only; symphonia's
  decoders trim encoder delay and padding by default,
  `symphonia-core-0.6.1/src/codecs/audio.rs:225`, `gapless: true`). The Opus
  pre-skip is dropped in Ogg and Matroska (`tests/opus.rs`
  `the_pre_skip_is_dropped_so_audio_stays_in_place`,
  `opus_in_matroska_drops_its_codec_delay_like_ogg`). No test covers MP3 or
  AAC encoder padding (not confirmed; needs fixtures).
- **Resampler delay.** `StreamResampler::new` drops the filter's
  `output_delay()` (`resample.rs:27`) and `finish` trims the output to
  `input × ratio` frames (`resample.rs:58-83`): `tests/decode.rs`
  `the_file_opener_resamples_to_the_bus_rate_with_an_exact_length`.
- **Transitions.** A transition with a known end is dispatched ahead with an
  exact frame (`engine.rs:1860-1897`). The hard `StartNextAt` (no overlap)
  fades the current source out over `declick_ms` ending on that frame, and
  starts the next on it at full level (`engine.rs:1962-1981`). The join is
  sample-exact: no gap, no overlap.

  When the transition frame is the **end of the current file** (a gapless
  album, a track into the next with no trailing silence), that de-click ramp
  removes the last 5 ms of real audio: the level dips to silence and steps
  back to full when the next starts. The probe
  `a2_a_gapless_join_at_the_end_of_the_file_keeps_the_level` measures the
  level falling to 0.002 and a step of 0.498 on the join frame. At a cut
  inside the file (a repeat pass or a self-next at a cue-out,
  `docs/technical/audio-engine.md:186`), the ramp-out is wanted; the step
  comes from the next source starting at full level inside its audio, which
  is **A1**.
- **Unknown length.** While a track has no known end (not analysed yet,
  `crates/fp-model/src/reducer.rs:675`), the plan is `SOURCE_END`, handled
  when the source finishes (`engine.rs:1876-1878`, `engine.rs:1737-1762`):
  the next starts at the bus frame the engine sees on its next tick. `tests/
  engine.rs` `source_end_plans_chain_when_the_file_runs_out` allows a gap of
  up to two blocks. **A14**.
- **Carts.** A cart's cue-out is applied by the worker, which truncates the
  stream at `until_secs` (`worker.rs:354-362`, set from
  `engine/carts.rs:162-166`); the mixer then finishes the drained source at
  full level (`mixer.rs:732-737`). The probe
  `a3_a_cart_ends_at_its_cue_out_without_a_step` measures a step of 0.5 at
  the cue-out. **A3**. A looped cart reopens the file at its cue-in in the
  same ring without a gap (`worker.rs:366-374`, `tests/worker.rs`
  `a_looped_source_repeats_without_gaps`); the splice is sample-adjacent,
  so whether it is smooth depends on the loop points. **A15**.

Verdict: decoding and resampling are gapless; the join between two sources
is not (**A2**, **A1**).

### 1.3 Seeking

`EngineAction::Seek` replaces the current source (`engine.rs:1394-1419`): the
old one gets `stop_quick_and_release`, a linear `declick_ms` ramp to zero and
a stop (`engine.rs:1063-1065`, `engine.rs:1022-1060`), and the new one starts
when it is buffered with a `declick_ms` ramp in (`StartState::WhenReady {
fade_in: true }`, `engine.rs:1414`, applied at `engine.rs:1825-1855`). The
first frame is exact: a resampled source pre-rolls by an aligned amount and
drops the warm-up (`worker.rs:45-66`, `resample.rs:121`); `tests/decode.rs`
`a_resampled_start_after_cue_in_matches_continuous_playback` and
`seeking_lands_on_the_exact_frame`, `tests/engine.rs`
`seek_replaces_the_source_at_the_new_position`. Between the two there is
silence while the new source buffers (`ready_threshold_ms`).

Evidence: probe `a_seek_ramps_the_old_source_out_and_the_new_one_in` (no
step). Verdict: **no change needed** (A12).

### 1.4 Decode-ahead margin against slow disks

Each source's ring holds `tuning.prebuffer_secs` (5 s by default,
`crates/fp-model/src/config.rs:459`) at the bus rate (`engine.rs:948-949`,
`engine/carts.rs:138-139`). A source becomes ready when
`ready_threshold_ms` (500 ms, `config.rs:460`) is buffered
(`worker.rs:380-382`, scaled to the source rate at `worker.rs:306`). One
worker per player serves the emptiest ring first (`worker.rs:299-304`) at
above-normal priority (`worker.rs:245`).

A disk that stalls longer than the buffered audio empties the ring: the
mixer outputs silence for the missing frames, counts an underrun and keeps
the timeline moving (`mixer.rs:738-740`; `tests/mixer.rs`
`an_underrun_is_silent_counted_and_the_timeline_keeps_moving`). The
underrun is counted in `SourceShared::underruns` and copied into
`PlayerTelemetry::underruns` (`engine.rs:513-517`), but nothing reads it: the
UI never shows it (the message `alert-underruns` exists in both locales,
`crates/fp-app/locales/en-US/main.ftl:85`, and is used nowhere; the status
bar alerts are built at `crates/fp-app/src/ui/app.rs:1578-1592`) and nothing
logs it. See **A8**.

Verdict: the margin is ample and configurable (**A13**, no change); the
missing report is **A8**.

## 2. Resampling

### 2.1 Quality and passband

`StreamResampler` uses rubato's asynchronous sinc resampler with
`SincInterpolationParameters::default()` (`resample.rs:23-24`): a sinc of 256
taps, a Blackman-Harris² window, oversampling 128 with cubic interpolation,
and an automatic cutoff
(`rubato-5.0.0/src/asynchro_sinc.rs:54-60`). The cutoff is 0.947 of the
lower Nyquist frequency (`rubato-5.0.0/src/windows.rs:88-130` with
`npoints = 256`), so 20.9 kHz for 44.1 ↔ 48 kHz
(`asynchro_sinc.rs:223-235`).

Measured by `resampling_44_1_to_48_khz_is_flat_to_20_khz_and_rejects_images`
and `resampling_48_to_44_1_khz_rejects_aliases` (single tones at −6 dBFS,
one-second Hann-windowed measurement on exact bins; `--nocapture` prints
them):

| 44.1 → 48 kHz | Gain |
|---|---|
| 20 Hz – 19 kHz | 0.0000 dB |
| 20 kHz | −0.0017 dB |
| 20.5 kHz | −0.66 dB |
| 21 kHz | −9.6 dB |
| image of 21 kHz at 23.1 kHz | −155 dB |

| 48 → 44.1 kHz | Level |
|---|---|
| 1 kHz | 0.0000 dB |
| 22.5 / 23 / 23.5 kHz aliasing to 21.6 / 21.1 / 20.6 kHz | −153 / −165 / −164 dB |

The image and alias figures are at the floor of the measurement. Verdict:
**no change needed** (A11).

### 2.2 Switching between resampled and bit-perfect playback

A source is opened at the bus rate it will play on (`engine.rs:969-973`) and
keeps that rate for its whole life. A bit-perfect bus follows the file's rate
only before a start and only when nothing on it is sounding
(`prepare_start`, `engine.rs:543-562`; `bus_sounding`, `engine.rs:526-537`);
the waiting sources are reopened at the new rate (`engine.rs:566-614`). A
source that starts while the bus is busy is resampled for its whole length.
So the switch happens only between tracks, never inside one.

The **BP** badge requires the file's rate to equal the bus rate, a device
format that holds the file's bits, exclusive access and a source the mixer
reports unaltered (`engine.rs:413-436`). Evidence: `tests/bit_perfect.rs`
`an_idle_bit_perfect_bus_reopens_at_the_file_rate`,
`a_busy_bit_perfect_bus_keeps_its_rate`, `a_resampled_source_is_not_bit_perfect`,
`bit_perfect_playback_is_bit_exact`, `volume_below_full_is_not_bit_perfect`,
`an_overlapping_source_is_not_bit_perfect`.

Whether a DAC clicks when its rate changes is a hardware matter (not
confirmed; needs hardware). Verdict: **no change needed** (A12).

## 3. Mixer

### 3.1 Summing, headroom and clipping

The mixer zeroes the block (`mixer.rs:454`) and adds every source into it in
`f32` with no clamp (`mixer.rs:714-722`, `*o += l`). The per-source gain is
`fade × pause × volume` (`mixer.rs:686`). The fade and pause ramps run
between 0 and 1 (`ramp.rs`, every target sent by the engine is 0.0 or 1.0:
`engine.rs:1035-1041`, `engine.rs:1826-1854`, `engine.rs:1944-1999`,
`mixer.rs:394`, `mixer.rs:402`). The volume is clamped to 0..=1 per block
(`mixer.rs:630`), and the model clamps it too. So no gain in the mixer
exceeds unity: probe `no_gain_in_the_mixer_exceeds_unity` (a volume of 4.0
plays at unity). One latent gap: `Attach` copies the volume unclamped as the
smoothing start (`mixer.rs:326`), so a volume above 1 at attach time would
apply for the smoothing time; the model never sends one, so this is not a
defect.

Above unity in the path, outside the mixer gains:

- the samples of the file itself (float files, lossy decoders);
- inter-sample peaks made real by resampling;
- the downmix of 3.0 and 3.1 files, which adds the centre at −3 dB with no
  normalisation (`crates/fp-decode/src/lib.rs:215-216`; 5.x is normalised,
  `lib.rs:217-231`): probe `a_three_channel_file_downmixes_above_full_scale`
  (0.8 + 0.57 = 1.37; A18);
- several sources on one output (overlaps, carts, other players): probe
  `the_meter_is_per_player_and_never_sees_the_device_sum` (two sources at 0.7
  give 1.4 at the device).

`f32` has 24 bits of mantissa and 8 of exponent: a sum of any realistic
number of sources keeps full resolution far above full scale, so the mix
itself never clips. Clipping happens only at the device format conversion
(§4.1).

Verdict: **no change needed** (A17). The readings this causes are §7.

### 3.2 Gain ramps: start, stop, pause, resume, seek, fade, fader move

Ramps available: `declick_ms` 5 ms linear, `pause_ramp_ms` 10 ms linear,
`gain_smoothing_ms` 20 ms per-sample slew of the volume
(`config.rs:457-468`, `mixer.rs:681-686`), and equal-power fades
(`ramp.rs:50-56`).

| Path | What applies | Evidence | Step? |
|---|---|---|---|
| Play (`StartCurrent`), CUE start, cart start | `declick_ms` ramp in when `from_secs > 0`, hard at the file's first frame (`Engine::send_start`, A1 fixed) | probes `a1_a_start_inside_the_file_ramps_in`, `a1_a_cue_inside_the_file_ramps_in`, `a1_a_cart_inside_the_file_ramps_in_and_one_at_the_start_does_not` | no |
| Stop now | `declick_ms` linear to 0, then stop (`engine.rs:1372-1392`, `engine.rs:1022-1060`) | probe `a_stop_ramps_down_without_a_step` | no |
| Pause / resume | `pause_ramp_ms` linear (`engine.rs:1304-1370`, `mixer.rs:390-404`) | probe `pause_and_resume_ramp_without_a_step`; `tests/mixer.rs` `pause_fades_out_holds_the_position_and_resume_continues_from_it` | no |
| Resume of a track loaded paused | `declick_ms` ramp in (`engine.rs:1354-1356`) | `tests/engine.rs` `load_paused_waits_for_resume_and_starts_at_the_saved_position` | no |
| Seek | old: `declick_ms` out; new: `declick_ms` in (`engine.rs:1394-1419`) | probe `a_seek_ramps_the_old_source_out_and_the_new_one_in` | no |
| Fade stop | equal-power over the fade (`engine.rs:1278-1297`) | probe `a_fade_stop_and_a_planned_stop_ramp_down` | no |
| Planned stop at a cue-out | `declick_ms` ending on the frame (`engine.rs:1985-2004`) | same probe; `tests/engine.rs` `a_planned_stop_ends_on_the_exact_frame_and_reports_the_entry` | no |
| Crossfade | old: equal-power out (`engine.rs:1259`); new: `declick_ms` ramp in when it starts inside the file (A1 fixed) | `tests/engine.rs` `a_crossfade_starts_the_next_now_and_reports_when_the_old_one_is_gone` | no |
| Segue | old: equal-power from the segue to the cue-out (`engine.rs:1941-1960`); new: on the frame, `declick_ms` ramp in when it starts inside the file (A1 fixed) | `tests/engine.rs` `a_segue_starts_the_next_on_the_exact_frame_and_overlaps_the_fade`; probe `a1_the_next_source_at_a_cue_in_ramps_in` | no |
| Hard transition | old: `declick_ms` ending on the frame; new: full level at the first frame, ramp in inside the file (`engine.rs:1962-1981`) | probe `a2_a_gapless_join_at_the_end_of_the_file_keeps_the_level` | **yes**: A2 at a file end |
| Fader move | per-sample slew over `gain_smoothing_ms` (`mixer.rs:681-685`) | probe `a_fader_move_is_smoothed`; `tests/mixer.rs` `volume_changes_are_smoothed` | no |
| Cart stop | `declick_ms` (`engine/carts.rs:261-291`) | `tests/cartwall.rs` | no |
| Cart cue-out | linear fade over the last `declick_ms` before the cue-out, none on a looped cart (`worker.rs`, `fade_to_limit`) | probe `a3_a_cart_ends_at_its_cue_out_without_a_step` | **yes** (A3) |
| Failed source draining | ramp to zero over the last buffered frames when at most `declick_ms` remain (`mixer.rs`, `fail_ramped`) | probe `a4_a_source_that_fails_mid_file_ends_without_a_step` | **yes** (A4) |
| Stop during the pause ramp | `fade_out` ramps the source out unless the pause ramp is over (`pause_ramp_ends`, `engine.rs` `fade_out`); the mixer finishes a paused slot that has a stop frame (`mixer.rs`, `render_slot`) | probe `a5_a_stop_during_the_pause_ramp_does_not_step` | **yes** (A5) |
| Stop or seek right after a start | a source whose start was sent but whose `Started` event is not polled yet is `Requested`, and `fade_out` ramps it out like a started one when its start frame has been reached (`engine.rs` `fade_out`), as the cartwall does (`engine/carts.rs` `stop_cart_source`) | probe `a5_a_stop_right_after_the_start_does_not_step` | **yes** (A5) |
| `Cancel` after a start | `Cancel` only forgets a start that has not happened (`mixer.rs:405-413`); every engine `Cancel` of a started source is followed by a ramp and a stop | `engine.rs:1032-1049` | no |
| Underrun | silence, no ramp (`mixer.rs:738-740`) | `tests/mixer.rs` `an_underrun_is_silent_counted_and_the_timeline_keeps_moving` | inherent to missing data |
| Player removed | `Detach` (`engine.rs:910-925`); the model removes only stopped players (`crates/fp-model/src/reducer.rs:989-997`), so at most a 5 ms stop ramp can be cut | — | negligible |

The level of a natural file end is the file's own: nothing is applied there
(`mixer.rs:732-736`).

## 4. Output

### 4.1 f32 to integer conversion

There are two conversion paths.

**Shared mode and ALSA `hw:` (cpal).** `choose_sample_format` prefers `F32`,
then `I32`, `I24`, `I16` (`crates/fp-backends/src/cpal_backend.rs:497-506`).
`render_converted` converts each sample with `T::from_sample(*s)`
(`cpal_backend.rs:510-530`, the conversion at line 527), which is dasp's
(`dasp_sample-0.11.0/src/conv.rs:530-536`; the comment above it says it
"assume[s] `-1.0 <= s < 1.0` … and will overflow otherwise"):

- `I16`: `(s * 32_768.0) as i16`. Rust's float-to-int `as` saturates and
  maps NaN to 0, so overs clip; but it **truncates toward zero** instead of
  rounding.
- `I32`: `(s * 2_147_483_648.0) as i32`: saturates, NaN is 0.
- `I24`: `I24::new_unchecked((s * 8_388_608.0) as i32)`: **no clamp to 24
  bits**. A sample of +1.0 becomes 8 388 608, outside the 24-bit range; cpal
  sends `I24` as S24 in a 32-bit word (`cpal-0.18.2/src/host/alsa/mod.rs:480`)
  or shifts it left by 8 on WASAPI (`host/wasapi/stream.rs:896-906`), and
  either way the device reads a value of the opposite sign: full scale wraps
  to −full scale.
- `F32`: the sample goes out as it is, overs and NaN included.

Evidence: exact for in-range integer PCM (`cpal_backend.rs` tests
`conversion_to_i16_is_exact_for_16_bit_pcm`,
`conversion_to_i32_is_exact_for_24_bit_pcm`,
`conversion_to_i24_is_exact_for_24_bit_pcm`); probes in
`crates/fp-backends/tests/audio_path_audit.rs`:
`cpal_conversions_saturate_and_silence_nan_except_24_bit` (passes; cpal's own conversion, no longer used),
`a7_the_24_bit_conversion_clips_instead_of_wrapping` (+1.0 becomes 8 388 608,
read as −8 388 608) and `a7_the_16_bit_conversion_rounds_to_the_nearest_step`
(100.75 steps become 100) both failed before the fix (they now live in
`src/cpal_backend.rs`, next to the conversion, which is private). **A7**,
fixed in 6609e74: `OutputSample` converts with `x * 2^(n-1)`, rounded and
clamped to the integer range, NaN to 0.

`I24` is chosen only for a device that offers neither `F32` nor `I32`
(`cpal_backend.rs:497-506`, test `devices_with_only_24_bit_integer_are_used`).
Integer PCM played at unity never reaches +1.0 (its largest value is
1 − 2⁻²³ at most), so a bit-perfect 24-bit file is safe; any over from §7 is
not.

**Exclusive mode (WASAPI).** `write_samples` (`exclusive.rs:21-45`) converts
through `i16::from_sample_` and `i32::from_sample_` (saturating; `I24` is the
`i32` value with its low byte cleared), so overs clip and NaN is 0; `F32`
goes out as it is. Probe
`what_an_exclusive_device_receives_above_full_scale_and_for_nan`;
`crates/fp-backends/tests/exclusive.rs`.

### 4.2 Dither when the device is not bit-perfect and narrower than the source

There is no dither anywhere in the path (`render_converted` and
`write_samples` above are the only conversions). A 16-bit device gets
truncated samples whenever they are not exact 16-bit values: a 24-bit or
float file, any gain below unity, a fade, a resampled source or a sum. The
error of truncation toward zero is up to one step (2⁻¹⁵, −90.3 dBFS) with an
RMS of about −95 dBFS, correlated with the signal; rounding (A7) halves it to
−96.3 dBFS peak, −101 dBFS RMS. TPDF dither would turn the remaining error
into constant noise at about −96 dBFS RMS.

A 16-bit output happens only on a device whose best format is 16-bit
(`cpal_backend.rs:497-506`, `exclusive.rs:58-79`). The error is audible only
in very quiet passages at high listening gain, far below the noise of a
broadcast chain. Dither must also never touch a bit-perfect block, which the
backend cannot tell today. Verdict: **too large** for this plan (A10).

## 5. Buffers and devices

### 5.1 Buffer sizes against underruns

The stream asks for `outputs.buffer_frames` (512, `config.rs:349`;
`engine.rs:759-764`). cpal is given a fixed size only if the device reports a
range containing it, else the device default (`cpal_backend.rs:154-159`,
`cpal_backend.rs:365-371`; test
`a_buffer_size_outside_every_reported_range_falls_back_to_the_device_default`),
and the size in use is reported back (`cpal_backend.rs:109-115`, test
`the_reported_buffer_size_is_the_one_in_use`). WASAPI exclusive aligns the
period to the device (`wasapi_exclusive.rs:103-131`). The callback's scratch
buffer is allocated with the stream, sized to at least 4096 frames, and a
larger callback is rendered in pieces (`cpal_backend.rs:372`,
`cpal_backend.rs:519-529`; test
`integer_devices_get_converted_audio_rendered_in_scratch_sized_pieces`). The
mixer's work per block is bounded (`CHUNK_FRAMES`, `mixer.rs:18`;
`max_commands_per_block`, `mixer.rs:464-469`). Verdict: **no change needed**
(A20).

### 5.2 Xrun counting and reporting

cpal's error callback (`cpal_backend.rs:491`) maps `ErrorKind::Xrun` to
`StreamErrorKind::Xrun` (`cpal_backend.rs:140-149`), and the bus counts it in
`BusShared::xruns` (`mixer.rs:187-196`). Unknown errors
(`StreamErrorKind::Other`) are counted as xruns too (`mixer.rs:192`). cpal
reports xruns on ALSA, WASAPI shared, Core Audio, PipeWire and JACK
(`cpal-0.18.2/src/host/*`, `ErrorKind::Xrun`), not on PulseAudio; the WASAPI
exclusive loop reports none (`wasapi_exclusive.rs:145-205`).

Nothing reads `xruns`, `lock_misses`, `leaked`, `dropped_events` or
`misrouted` outside `mixer.rs` (a search of `crates/` finds no other use),
and nothing reads the per-source `underruns` beyond copying it into the
telemetry (§1.4). `docs/technical/threading-and-realtime.md:65-67` says the
conductor turns these counters "into log lines and UI alerts"; it does not.
Verdict: **confirmed defect** (A8).

### 5.3 Device loss and recovery

A backend error marks the bus lost (`mixer.rs:190`); the watchdog also
declares it lost after `watchdog_timeout_ms` (500 ms) without a block, with a
longer grace before the first block (`bus.rs:298-321`). A virtual clock then
renders the same mixer at real-time pace (`bus.rs:43-82`), the device is
retried every `reconnect_interval_ms` and takes the mixer back
(`bus.rs:322-334`). The status bar shows `alert-device-lost`
(`crates/fp-app/src/ui/app.rs:1578-1589`). Evidence: `tests/bus.rs`
`unplugging_switches_to_the_virtual_clock_and_the_timeline_keeps_moving`,
`a_device_that_stops_rendering_is_declared_lost_by_the_watchdog`,
`a_slow_starting_device_gets_a_grace_period_before_its_first_block`,
`a_returning_device_takes_the_mixer_back_without_losing_time`,
`a_failed_reconnection_keeps_the_virtual_clock_running`. The returning
device resumes mid-signal without a ramp, which is inherent to the loss.
Verdict: **no change needed** (A19).

### 5.4 Rate changes on bit-perfect devices

`reopen_at` reopens the stream at the file's rate keeping the mixer, its
clock and its slots; a refused rate restores the previous one, is remembered
until the device comes back from a loss, and is not asked for again
(`bus.rs:249-278`, `bus.rs:332`). Frame-based durations follow the rate
(`mixer.rs:440-449`, test
`volume_smoothing_keeps_its_duration_when_the_rate_changes`). Evidence:
`tests/bit_perfect.rs` `a_rate_the_device_refuses_keeps_the_previous_rate`,
`an_idle_bit_perfect_bus_reopens_at_the_file_rate`. Verdict: **no change
needed** (A21).

## 6. Real-time safety of the callback (CLAUDE.md rule 5)

| Function | Allocation / free | Locks | Logging, I/O | Panics, indexing |
|---|---|---|---|---|
| cpal callback closure (`cpal_backend.rs:480-490`) | none: the renderer arrives through an `rtrb` queue (`cpal_backend.rs:383`, `484`); the scratch is allocated in `build` (`cpal_backend.rs:476`) | none | none | none |
| `render_converted` (`cpal_backend.rs:510-530`) | none | none | none | `get_mut`, no indexing |
| `MixerRenderer::render` (`mixer.rs:777-790`) | none | `try_lock` only; on contention it outputs silence and counts a miss | none | a poisoned lock is recovered |
| `Mixer::render`, `apply` (`mixer.rs:318-508`) | none: memory leaves through `Retired` (`mixer.rs:300-312`), a full queue leaks and counts instead of freeing | none | none | `get`/`get_mut`; subtractions guarded (`mixer.rs:646`, `650-662`) |
| `render_slot`, `finish`, `mark_unaltered`, `MeterMode` (`mixer.rs:522-767`) | none (a stack chunk, `mixer.rs:634`) | none | none | `get`, `as_chunks` |
| `TruePeak::push`, `KWeighting::process` (`truepeak.rs:39-62`, `kweight.rs:58-96`) | fixed arrays | none | none | `get` |
| `BusShared::report` (error callback, `mixer.rs:187-196`) | none | none | none | none |
| WASAPI exclusive render loop (`wasapi_exclusive.rs:174-203`) | buffers allocated before the loop (`wasapi_exclusive.rs:153-155`) | none of ours | the device calls themselves | `get_mut` |
| `write_samples` (`exclusive.rs:21-45`) | none | none | none | `as_chunks_mut` |
| Core Audio hog (`coreaudio_hog.rs`) | not on the callback: taken and released around the stream | — | — | — |
| `NullStream` thread, virtual clock (`null.rs:96-112`, `bus.rs:57-75`) | buffers allocated before the loop | the virtual clock `lock`s the mixer: it is not a device callback | they sleep to pace themselves, by design | none |

Evidence: `tests/mixer.rs` wraps every `Mixer::render` in `assert_no_alloc`
(`render` helper, `tests/mixer.rs:74-78`), including true peak
(`true_peak_can_be_switched_on_for_a_bus`) and the command flood. It does not
cover `MixerRenderer`, the programme-meter integrator together with true
peak, `Grow`/`Detach` in the same render, or `write_samples`; the probe
`the_renderer_the_device_calls_and_the_sample_writer_never_allocate` covers
those. `assert_no_alloc` is active in debug builds only
(`#[cfg(debug_assertions)]`). `render_converted` is private to the backend
and not wrapped (its body is a loop over `get_mut` and `from_sample`).

Not confirmed (needs Windows or macOS): whether the `wasapi` crate's
`write_to_device` and `get_available_space_in_frames` allocate. On Linux,
unlocking the mixer's `Mutex` after a `try_lock` may make a futex wake system
call when the conductor or the virtual clock waits on it; it never blocks.

Verdict: **no change needed** (A22).

## 7. Readings above 0 dBFS (O34)

**What the meter measures.** Each player's meter reads that player's own
sources on air, current and fading out, after their gain
(`take_meter_input`, `engine.rs:440-473`; the samples are measured after
`l * g` at `mixer.rs:689-704`). It reads the sample peak, or the true peak
when it is on (`mixer.rs:693-697`, `truepeak.rs:36-62`). It never measures
the device's sum: probe `the_meter_is_per_player_and_never_sees_the_device_sum`
(two sources at 0.7 on one output: each reads −3.1 dBFS, the device gets
+2.9 dBFS). The pre-listen is taken and dropped (`engine.rs:447-449`).

**When it reads above 0 dBFS.**

1. The file already exceeds full scale: a float file, or a lossy file whose
   decoded peaks exceed 1.0. Nothing clamps before the device: probe
   `a_float_file_above_full_scale_reaches_the_mix_and_the_meter_unchanged`
   (a float WAV at 1.5 reaches the device at 1.5 and the meter reads
   +3.5 dBFS).
2. Resampling makes inter-sample peaks real: probe
   `resampling_creates_inter_sample_overs` (a sine whose samples are all at
   0 dBFS reads +3.01 dBFS after 44.1 → 48 kHz).
3. True-peak mode reads the peaks between samples, up to about 3 dB above
   the sample peak: `tests/metering.rs`
   `true_peak_meets_ebu_tech_3341_cases_15_to_19` (case 19: samples at
   0 dBFS read +3 dBTP).
4. The downmix of a 3.0 or 3.1 file (§3.1): probe
   `a_three_channel_file_downmixes_above_full_scale`.
5. Several sources on one output sum above full scale at the device (§3.1),
   but the meter does not show it.
6. No gain or fader exceeds unity (§3.1): probe
   `no_gain_in_the_mixer_exceeds_unity`.

**What the device receives** (§4.1): integer formats clip at full scale
(`dasp_sample-0.11.0/src/conv.rs:534-536`; `exclusive.rs:24-37`), except
cpal's 24-bit format, which wrapped until A7 was fixed; a float device receives the over as
it is, and the sound server or driver clips it in its own conversion.

**How the meter shows it.** The digital peak scale ends at 0 dBFS: a level
above sits at the top (`crates/fp-app/src/ui/widgets.rs:343`,
`digital_deflection(db.min(0.0))`). The bar is red from `danger_dbfs`
(−3 dBFS by default, `crates/fp-model/src/config.rs:204`;
`widgets.rs:392-407`, `widgets.rs:879`), so an over is the same red as
−2 dBFS. Only the maximum readout tells it: it prints the highest level with
its sign, for example `+3.5` (`widgets.rs:410-421`; the maximum follows the
measured peak, `crates/fp-engine/src/meter.rs:447-449`). Non-finite samples
are read as silence by the meter (`meter.rs:68-87`) while they reach the
device (A6, fixed: `FileDecoder` now replaces them by 0).

**Proposal** (the maintainer chooses; none of this is built by this task):

- **(a) Over indicator.** A latched "OVER" mark on the player's meter when
  any sample of its output exceeded 0 dBFS, and one per device when the
  device's sum clipped; both clear with the maximum readout.
  Code it touches: `render_slot` sets a per-source `over` flag when
  `|l|` or `|r|` exceeds 1.0 (it already computes `audible_l/r`,
  `mixer.rs:690-691`); `Mixer::render` sets a per-bus `clipped` flag after
  summing; `SourceShared` and `BusShared` get one `AtomicBool` each;
  `MeterInput`/`MeterReading` carry it and `MeterState::reset_max` clears it
  (`meter.rs:374-376`); the meter widget draws the mark; one message in both
  locales; the device flag can join the status bar alerts. Cost on the
  real-time thread: one comparison per sample per source and one pass of
  comparisons over the output block; no allocation, no new state beyond the
  flags.
- **(b) Optional output limiter per bus.** A look-ahead true-peak limiter at
  the end of `Mixer::render`, a `Config` field per output device, off by
  default (ceiling in dBTP and release time with ranges), never active on a
  bit-perfect bus or a DSD stream (Tasks 3–9). Code it touches: a limiter
  module in `fp-engine` (delay line, a 4× true-peak detector like
  `truepeak.rs`, gain smoothing); its buffers sized from the configuration
  and the rate and replaced through a `BusCommand` when either changes; the
  bit-perfect check (`engine.rs:413-436`) must see it bypassed; settings UI
  and both locales. Cost on the real-time thread: about 50 multiply-adds
  per sample and channel for the detector plus the gain computer, and
  latency equal to the look-ahead (1–5 ms) on everything that bus plays.
- **(c) No change.** The readings are correct measurements of what each
  player sends; document them in the user guide (meters page).

Recommendation: **(a)**, together with the fix of A7. The readings are right
and an over is already shown by the `+` of the maximum readout, but easy to
miss; a latched mark makes it unmistakable at nearly no cost, and the device
flag covers the one case the per-player meter cannot see (several sources
summing). A limiter changes the programme, adds latency and duplicates the
broadcast processor that normally follows a playout system; it is worth
building only if operators report clipping on air. Whatever the choice,
24-bit devices must clip instead of wrapping (A7).

## 8. Findings

| Id | Area | Finding | Evidence | Verdict | Proposed action |
|---|---|---|---|---|---|
| A1 | 3.2 Ramps / 1.2 Transitions | A start inside the audio (Play from a cue-in or position, the new source of a crossfade, segue or hard transition, a CUE, a cart) begins at full level with no ramp: a step from silence. Impact: audible click whenever the start point is not silent. | `engine.rs:1251`, `engine.rs:1442`, `engine.rs:1922`, `engine.rs:1758`, `engine/carts.rs:184`, `engine.rs:1825-1855`; probe `a1_a_start_inside_the_file_ramps_in` (step 0.5) | fixed in 29ef66b | ramp in over `declick_ms` every source that starts at `from_secs > 0`; a start at the first frame of the file stays hard (the file's own start, and gapless joins). Keep `tests/engine.rs` segue/transition tests (they start at 0) passing. |
| A2 | 1.2 Gapless | A hard transition on the last frame of the current file fades the file's last `declick_ms` out, then the next starts at full level: a 5 ms dip to silence and a step at every gapless join. Impact: audible dip/click between continuous tracks. | `engine.rs:1962-1981`; probe `a2_a_gapless_join_at_the_end_of_the_file_keeps_the_level` (falls to 0.002, step 0.498) | fixed in 9cc3fa4 | no de-click ramp on the outgoing source when the transition frame is its end of stream (the mixer's `RampOutBeforeCut` is skipped when eof is set and the ring ends by the stop frame); the ramp stays for cuts inside the file. |
| A3 | 1.2 Carts | A cart's cue-out is a truncation by the worker; the mixer ends the drained source at full level. Impact: audible click at a cart's cue-out inside audio. | `worker.rs:354-362`, `engine/carts.rs:162-166`, `mixer.rs:732-737`; probe `a3_a_cart_ends_at_its_cue_out_without_a_step` (step 0.5) | fixed in 3ba3de2 | fix in this plan: the worker fades the last `declick_ms` before `until_secs` linearly to zero on a pass that ends there and does not loop (it knows `limit_frames`); the frame count is passed in `LoadOptions`. |
| A4 | 1.1 Errors | A source whose decoder fails mid-file plays its buffer and reports `SourceFailed` once (correct), but its end is cut at full level. Impact: audible click after a decode error (rare: damaged files). | `worker.rs:307-321`, `engine.rs:1563-1569`, `mixer.rs:732-737`; probe `a4_a_source_that_fails_mid_file_ends_without_a_step` (step 0.5) | fixed in 3ba3de2 | fix in this plan: when a source is `failed` and its ring holds no more than the de-click length, the mixer ramps it to zero over the frames that remain (one more `Ramp` per slot, real-time safe). A truncated file ends as a normal end (A16) and is not covered. |
| A5 | 3.2 Ramps | A source that may still be audible is detached at once: a stop while the pause ramp runs (`fade_out` releases a paused player's source), and a stop or seek between the mixer starting a source and the engine polling its `Started` event (`Requested`). Impact: audible click, only when the two commands come within about one block (a few ms to 10 ms). | `engine.rs:1024-1028`; contrast `engine/carts.rs:263-265`; probes `a5_a_stop_during_the_pause_ramp_does_not_step`, `a5_a_stop_right_after_the_start_does_not_step` (step 0.5) | fixed in 3ba3de2 | fix in this plan: `fade_out` treats `Requested` like `Started` (as the cartwall does), and a paused source whose pause ramp may still run is stopped after a ramp instead of detached (the mixer honours a stop on a pausing slot). |
| A6 | 1.1 / rule 9 | NaN and infinite samples from a file (float WAV, WavPack float) pass the decoder, resampler, mixer and conversion: a float device receives them; an integer device gets 0 for NaN and full scale for infinity. They also stick in the source's K-weighting state (`kweight.rs:58-69`), so its loudness reads silence afterwards (`meter.rs:68-87`). Impact: noise burst or full-scale click on bad data. | `symph.rs:217`, `wavpack.rs:174`, `mixer.rs:714-722`, `cpal_backend.rs:527`; probe `a6_non_finite_samples_never_reach_the_device` (3 of 3 reach the device) | fixed in de28367 | fix in this plan: `FileDecoder` replaces non-finite samples by 0 (off the real-time thread) and logs once per file. |
| A7 | 4.1 Conversion | cpal's `I24` conversion does not clamp: +1.0 and above wrap to the opposite sign on the device. cpal's `I16` conversion truncates toward zero instead of rounding. Impact: loud crackle instead of clipping on any over on a device whose best format is 24-bit integer; the `I16` error is not audible (≤ 1 step, −90 dBFS). | `cpal_backend.rs:527`, `dasp_sample-0.11.0/src/conv.rs:530-536`, `cpal-0.18.2/src/host/alsa/mod.rs:480`, `host/wasapi/stream.rs:896-906`; probes `a7_the_24_bit_conversion_clips_instead_of_wrapping`, `a7_the_16_bit_conversion_rounds_to_the_nearest_step` | fixed in 6609e74 | fix in this plan: `render_converted` converts with its own function: clamp to full scale, NaN to 0, round to the nearest step (exact integer PCM stays exact, so the existing exactness tests and bit-perfect playback are unchanged). |
| A8 | 5.2 / 1.4 Reporting | Xruns, source underruns, lock misses, leaks, dropped events and misrouted blocks are counted but never logged or shown, contrary to `threading-and-realtime.md:65-67`; `alert-underruns` is unused; unknown stream errors count as xruns. Impact: none audible; dropouts go unnoticed by the operator. | `mixer.rs:105-112`, `mixer.rs:187-196`, `mixer.rs:738`, `engine.rs:513-517`, `crates/fp-app/src/ui/app.rs:1578-1592`, `locales/en-US/main.ftl:85` | fixed in 4dd22a8 | fix in this plan: the conductor logs each increase (rate-limited) and the status bar shows `alert-underruns` per player and an xrun alert per device; `StreamErrorKind::Other` is logged, not counted as an xrun. |
| A9 | 7 Overs (O34) | Readings above 0 dBFS are correct: the file's own overs, resampled inter-sample peaks, true peak, the 3.x downmix; the device sum is not metered; integer devices clip (24-bit cpal wraps, A7). | §7 probes; `widgets.rs:343`, `widgets.rs:410-421` | resolved: option (c), documented in the player guide (readings above 0 dBFS) | (a) latched OVER mark per player and per device (recommended), (b) optional per-bus true-peak limiter, off by default, bypassed on bit-perfect and DSD, or (c) no change, documented. |
| A10 | 4.2 Dither | No dither: a 16-bit device gets truncated (A7: rounded) samples whenever they are not exact. Inaudible in practice (−96 dBFS). | `cpal_backend.rs:510-530`, `exclusive.rs:21-45` | too large | roadmap item: TPDF dither on integer outputs narrower than 24 bits, bypassed on blocks the mixer reports unaltered (needs that knowledge to reach the backend). |
| A11 | 2.1 Resampling | Flat to 20 kHz (−0.0017 dB), −0.66 dB at 20.5 kHz, −9.6 dB at 21 kHz; images and aliases below −150 dB. | `resample.rs:23-24`; probes `resampling_44_1_to_48_khz_is_flat_to_20_khz_and_rejects_images`, `resampling_48_to_44_1_khz_rejects_aliases` | no change needed | — |
| A12 | 1.3 Seek / 2.2 BP switch | Seek ramps out and in with an exact first frame; the rate switch happens only between tracks, and the BP badge follows it. | probe `a_seek_ramps_the_old_source_out_and_the_new_one_in`; `tests/decode.rs`, `tests/bit_perfect.rs` | no change needed | — |
| A13 | 1.4 Decode-ahead | 5 s ring, 500 ms to start, emptiest ring first, above-normal priority; a longer stall plays silence and keeps time. | `engine.rs:948-949`, `worker.rs:299-307`, `mixer.rs:738-740`; `tests/mixer.rs` | no change needed | (reporting: A8) |
| A14 | 1.2 Unknown length | A transition planned at `SOURCE_END` (track not analysed yet) starts the next on the engine's next tick: a gap of up to two blocks. | `engine.rs:1876-1878`, `engine.rs:1742-1744`; `tests/engine.rs` `source_end_plans_chain_when_the_file_runs_out` | no change needed | — |
| A15 | 1.2 Cart loops | A looped cart splices its cue-out to its cue-in sample-adjacent with no crossfade; smoothness depends on the loop points. | `worker.rs:366-374`; `tests/worker.rs` `a_looped_source_repeats_without_gaps` | no change needed | — |
| A16 | 1.1 Damaged files | symphonia skips corrupt packets (the audio jumps) and ends a truncated file as a normal end; both are the degradation rule 9 asks for. | `symph.rs:201-202`, `symph.rs:211`; probe `a_truncated_wav_ends_as_a_normal_end_not_a_failure` | no change needed | — |
| A17 | 3.1 Summing | `f32` sum with no clamp; no gain above unity (volume clamped per block, `mixer.rs:630`); `Attach` copies the volume unclamped as the smoothing start (`mixer.rs:326`), unreachable since the model clamps. | probes `no_gain_in_the_mixer_exceeds_unity`, `the_meter_is_per_player_and_never_sees_the_device_sum` | no change needed | — |
| A18 | 3.1 Downmix | 3.0 and 3.1 files fold the centre in at −3 dB without normalisation (up to +4.6 dB), while 5.x is normalised. | `lib.rs:215-231`; probe `a_three_channel_file_downmixes_above_full_scale` | no change needed | documented in §7 |
| A19 | 5.3 Device loss | Virtual clock, watchdog, reconnection and the lost-device alert work. | `bus.rs:298-336`; `tests/bus.rs` | no change needed | — |
| A20 | 5.1 Buffers | Requested size used when the device allows it, else its default; the size in use is reported; oversize callbacks render in pieces. | `cpal_backend.rs:109-159`, `cpal_backend.rs:365-372` and its tests | no change needed | — |
| A21 | 5.4 Rate changes | Reopen at the file rate keeping the clock; refused rates restored and remembered. | `bus.rs:249-278`; `tests/bit_perfect.rs` | no change needed | — |
| A22 | 6 Real-time safety | No allocation, lock (beyond `try_lock`), logging, I/O or panic on the callback paths; the WASAPI crate's calls are not verifiable on Linux. | §6 table; `tests/mixer.rs`; probe `the_renderer_the_device_calls_and_the_sample_writer_never_allocate` | no change needed | — |
