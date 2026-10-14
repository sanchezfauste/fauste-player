# Shared signal processing (`fp-dsp`)

`fp-dsp` holds the signal processing that both the audio engine and the
analysis use. It has no dependencies, does no I/O, forbids `unsafe` and
denies `clippy::indexing_slicing`.

| Module | What | Used by |
|---|---|---|
| `kweight` | The ITU-R BS.1770 K-weighting: the pre-filter (high shelf) and the RLB high-pass, as two biquads derived for any rate from the standard's analogue prototypes. Filter states are flushed below 1e-20. | The mixer (per slot, on the device callback) and `loudness` |
| `truepeak` | The BS.1770-5 Annex 2 true-peak interpolator: 4× oversampling with the published order-48 FIR, in floating point. | The mixer and `loudness` |
| `loudness` | `LoudnessMeter`: the integrated loudness of a whole programme, gated (absolute −70 LUFS, relative −10 LU) over 400 ms blocks with a 100 ms hop, both channels weighted 1.0, and the largest true peak of both channels. | The analysis (loudness normalization spec N4) |

`KWeighting::process` and `TruePeak::push` use fixed arrays only and are
real-time safe. `LoudnessMeter` keeps the energy of every 100 ms hop
(about 36 000 numbers per hour of audio) and is never used on the device
callback.

The hop is the file's rate divided by ten, rounded down to whole frames
(at least one), so rates such as 11 025 Hz work; a gating block is four
hops (`gating_block_frames`), never longer than 400 ms, so a file of
400 ms or more always holds one. The analysis uses the same function to
tell a file too short to measure.
A trailing hop shorter than 100 ms is not measured.

A programme shorter than one gating block (400 ms) gives no measurement;
one with no block above the absolute gate has no integrated loudness. The
true peak of digital silence is reported as `SILENCE_DBTP` (−200 dBTP)
so that it is always a finite number. Samples that are not finite count
as silence. A push may end mid-frame: the half frame is carried to the
next push, so the channels never swap, and one never completed is
ignored. A true peak above
+60 dBTP (`MAX_PLAUSIBLE_DBTP`) means a corrupt file, and gives no
measurement rather than one that would silence the track.

Tests: `crates/fp-dsp/tests/filters.rs` (the standard's coefficients, EBU
Tech 3341 true-peak cases 15–23) and `crates/fp-dsp/tests/loudness.rs`
(EBU Tech 3341 cases 1–5 for integrated loudness, rates from 8 to 96 kHz,
short and silent programmes, non-finite input).
