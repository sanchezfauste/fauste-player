# Decoding

`fp-decode` turns any supported file into interleaved stereo `f32` at the
file's own rate. Mono is duplicated and more channels are downmixed with the
ITU-R BS.775 coefficients (`downmix`). The engine resamples to the bus rate;
analysis measures the same blocks. The design is the audio formats spec,
[`2026-09-27-audio-formats-design.md`](../superpowers/specs/2026-09-27-audio-formats-design.md).

## `FileDecoder` and its backends

`FileDecoder::open` reads the first 16 bytes and `probe` picks a backend from
the file's signature. The extension is only a hint for formats without one.

| Signature | `Kind` | Backend | Crate |
|---|---|---|---|
| `DSD ` | `Dsf` | `dsd::DsdDecoder` | our own |
| `FRM8` … `DSD ` | `Dff` | `dsd::DsdDecoder` | our own |
| `wvpk` | `WavPack` | `wavpack::WavPackDecoder` | `wavicle` |
| `MAC ` | `Ape` | `ape::ApeFileDecoder` | `ape-decoder` |
| anything else | `Symphonia` | `symph::SymphoniaDecoder` | `symphonia`, plus `opus-decoder` |

Every backend has the same interface: `sample_rate`, `bits_per_sample`
(`None` for lossy codecs and for DSD), `channels`, `frames_hint`,
`seek(secs)` and `next_block`. All added crates are pure Rust, MIT or
Apache-2.0, and forbid or deny `unsafe`.

## symphonia and Opus

symphonia decodes WAV, AIFF, CAF, FLAC, MP1/2/3, AAC, ALAC, Vorbis and
Matroska. Its codec registry (`opus::codecs`) is symphonia's own plus an Opus
decoder (`opus::OpusDecoder`) built on `opus-decoder`. It decodes at 48 kHz,
applies the header's output gain, and supports mono and stereo (mapping
family 0).

Decoders are opened with gapless decoding on (symphonia's default): each
decoder removes the encoder delay and end padding the container marks on its
packets (`trim_start`, `trim_end`), so the Opus decoder does it too (Opus
pre-skip). The backend must not trim again (`tests/gapless.rs`). A seek lands
on the packet before the target and `skip_frames` drops the rest.

## DSD

DSF (little-endian bit order, per-channel blocks of usually 4096 bytes) and
uncompressed DSDIFF (MSB first, interleaved bytes) are read by `dsd/dsf.rs`
and `dsd/dff.rs`. Every size in a header is clamped to the file length, whole
DSF block groups only; DST-compressed DSDIFF is refused.

`dsd/convert.rs` converts to PCM at the DSD rate ÷ 32:

- an 800-tap linear-phase low-pass (Kaiser windowed sinc, β 10.61) at the
  DSD rate, flat to 20 kHz within ±0.1 dB and at least 100 dB down from the
  output's Nyquist frequency (unit tests check both);
- evaluated through one 256-entry table per byte of the window, so each
  output sample costs 100 lookups;
- ±1 bits, unity gain at DC: a 50 % sine (the SACD reference level) reads
  −6 dBFS. The tests encode a 1 kHz sine with a 5th-order sigma-delta
  modulator and measure −6.02 dBFS and THD+N of about −100 dB.

Output frame `k` is the filter centred on DSD byte `4k`. Bytes outside the
stream read as the idle pattern `0x69`, which the filter turns into silence.
A seek re-reads from the start of the window (the containing DSF block), so
it yields exactly the samples of a decode from the start.

## WavPack

`wavpack.rs` indexes the block headers when it opens the file, and decodes one
block at a time with `wavicle::decode_stream`: every WavPack block carries its
own starting state. A seek starts at the block that holds the target frame.
A block size larger than the format allows is refused before `wavicle` sees it
(the crate adds to it unchecked). Hybrid, multichannel and 8-bit files are
refused, since `wavicle` does not decode them.

## Monkey's Audio

`ape.rs` decodes one frame at a time with `ape-decoder` and seeks with its
`seek`, which gives the frame and the samples to skip. Samples are 8-, 16-,
24- or 32-bit integer or 32-bit float, little- or big-endian as the file
says. Before this backend, symphonia could take an APE file for an MPEG
stream and play noise.

## Not supported

Musepack, TTA, WMA and AC-3: no pure-Rust decoder could be verified. These
files show as unreadable.
