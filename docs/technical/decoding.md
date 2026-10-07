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

`FileDecoder::duration_hint_secs` turns `frames_hint` into seconds
(`duration_from_frames`) for the length a track shows before its analysis
(operator feedback 4, Q1). An MPEG stream (MP1/2/3) has one only when its
first frame is a Xing, Info or VBRI frame: without it symphonia estimates
the length from the first frames' bitrate, which a VBR file can get short,
and a short length would end the track early.

## symphonia and Opus

symphonia decodes WAV, AIFF, CAF, FLAC, MP1/2/3, AAC, ALAC, Vorbis and
Matroska. Its codec registry (`opus::codecs`) is symphonia's own plus an Opus
decoder (`opus::OpusDecoder`) built on `opus-decoder`. It decodes at 48 kHz,
applies the header's output gain, and supports mono and stereo (mapping
family 0).

Decoders are opened with gapless decoding on (symphonia's default): each
decoder removes the encoder delay and end padding the container marks on its
packets (`trim_start`, `trim_end`), so the Opus decoder does it too. The
backend must not trim again (`tests/gapless.rs`).

The Opus **pre-skip** (RFC 7845 §4.2) is not marked that way: symphonia
0.6 keeps it as the track's delay in Ogg, whose granule positions count it,
and Matroska shifts its timestamps by `CodecDelay` rounded to its 1 ms
resolution. `SymphoniaDecoder` therefore drops exactly the pre-skip of the
`OpusHead` from the start of the stream in both containers, and shifts Ogg
Opus timestamps back by it when seeking and placing packets
(`ts_offset_secs`). A click comes out where it went in (`tests/opus.rs`).

**Seeking** (`SymphoniaDecoder::seek`): decoding restarts a codec's pre-roll
before the target, 80 ms for Opus (RFC 7845 §4.6) and the longest block
(8192 frames) for Vorbis, whose first packet after a reset only primes the
overlap. Each decoded packet is then placed by its timestamp (its frames end
where `pts + dur` ends, or start at `pts` when the container gives no
duration, as Matroska blocks), and the frames before the target are dropped, so the
output starts on the target frame whatever the decoder yields first. A seek
past the end is the end of the stream, as for the other backends.

The Opus decoder goes through 16-bit PCM inside `opus-decoder`: its noise
(about −96 dBFS) is far below Opus's own coding noise. A decoder without that
step exists (`opus-pure`), but it uses `unsafe` code, which the formats spec
rules out for decoding untrusted files.

`opus-decoder` is vendored (`vendor/opus-decoder`, through
`[patch.crates-io]`): version 0.1.1 computed its FFT as a direct O(n²) DFT,
5.5 ms per 20 ms packet, and the copy uses a mixed-radix FFT instead,
about 55 µs, still without `unsafe` (`vendor/opus-decoder/VENDORED.md`).

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
  output sample costs 100 lookups, straight over the buffered bytes except at
  the edges of the stream (about 16 % of a core for DSD512 5.1 in a release
  build; `dsd_surround_decoding_speed`, an ignored test, measures it);
- ±1 bits, unity gain at DC: a 50 % sine (the SACD reference level) reads
  −6 dBFS. The tests encode a 1 kHz sine with a 5th-order sigma-delta
  modulator and measure −6.02 dBFS and THD+N of about −100 dB.

Output frame `k` is the filter centred on DSD byte `4k`. Bytes outside the
stream read as the idle pattern `0x69`, which the filter turns into silence.
A seek re-reads from the start of the window (the containing DSF block), so
it yields exactly the samples of a decode from the start. DSF blocks are read
several at a time (about 4096 bytes per channel), so a file with tiny blocks
costs no more reads than the usual one.

`FileDecoder::dsd_rate()` gives the DSD rate of a DSD file (`None` for every
other format); analysis stores it in `AudioFormat::dsd_rate`, and
`AudioFormat::sample_rate` stays the rate of the PCM conversion.

**Raw bytes for DSD output** (O25). `DsdRawReader` (`dsd/raw.rs`) reads a DSF
or uncompressed DSDIFF file as per-channel DSD bytes, most significant bit
first in time whatever the container (`next_bytes`), with `dsd_rate`,
`channels`, `len_bytes` and `seek`. A seek lands on round(secs × rate ÷ 8)
bytes, rounded down to an even byte (one 16-bit word). It shares the
`ChunkReader` (header layout and block reading, the same code as the
converting decoder) with `DsdDecoder`, so both read a file identically.
The engine reads the same file twice, raw for the output and converted for the
meters and for the switch to PCM (`dsd_file_opener`).

## WavPack

`wavpack.rs` indexes the block headers when it opens the file, and decodes one
block at a time with `wavicle::decode_stream`: every WavPack block carries its
own starting state. A seek starts at the block that holds the target frame,
counting from the first block's index (which need not be 0 in a file cut out
of a longer stream).
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
