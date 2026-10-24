# Decoding

`fp-decode` turns any supported file into interleaved stereo `f32` at the
file's own rate. Mono is duplicated and more channels are downmixed by
their positions with the ITU-R BS.775 coefficients (`downmix.rs`, see
[Downmix](#downmix)). The engine resamples to the bus rate;
analysis measures the same blocks. The design is the audio formats spec,
[`2026-09-27-audio-formats-design.md`](../superpowers/specs/2026-09-27-audio-formats-design.md).

## Downmix

`Downmix` (`crates/fp-decode/src/downmix.rs`) turns one frame of any layout
into a stereo pair. The layout is the channel positions the container reports
(symphonia's `Channels`; the decoded order is the order of the position
bits, which for WAV masks is the WAV order). When it reports none, or a
count that does not match the frame, the layout is the usual one for the
channel count. So is a set equal to the first `n` positions with `n` of 4 or
more: plain `WAVE_FORMAT_PCM` files and masks of zero get that set invented
by symphonia, which says nothing of the speakers (the mask of a 3.1 file is
the same set, so four channels always read as quad). The backends that know
no positions (WavPack, Monkey's Audio, DSD) use the count; the mix is built
once when the file is opened (symphonia: when the channels change).

| Position | To left | To right |
|---|---|---|
| Front left (also front left of centre, wide, bottom) | 1 | 0 |
| Front right (likewise) | 0 | 1 |
| Front centre | -3 dB | -3 dB |
| Back centre, top centre | -3 dB | -3 dB |
| Rear left, side left, top left | -3 dB | 0 |
| Rear right, side right, top right | 0 | -3 dB |
| LFE | dropped | dropped |

A layout with at least one surround, side, rear, top or extra front channel
(front left/right of centre, wide) is scaled by
`1 / max(sum of the left weights, sum of the right weights)`, so that it
cannot clip (5.1: `1 / (1 + 2 * 0.707)`, as before). L R C (3.0) is not
scaled; an audit test pins that. A single channel, whatever its position, is duplicated.

By count: 1 mono; 2 L R; 3 L R C; 4 quad (L R Ls Rs); 5 L R C Ls Rs; 6 5.1
(L R C LFE Ls Rs); 7 6.1 (L R C LFE Bc Sl Sr); 8 7.1 (L R C LFE Bl Br Sl
Sr); beyond eight, the first eight. Vorbis and FLAC deliver their channels in
this order already (symphonia reorders Vorbis), so no reordering is needed
here. Opus is read only in mapping family 0 (mono and stereo): multichannel
Opus, like multichannel AAC, is not supported.

symphonia's ALAC reader maps its 7.1 layout with the Lc/Ls weights swapped
(upstream); the downmix trusts the positions it is given.

Loudness is measured on this downmix, so a layout's side channels count in it.
Changing the matrix for a layout changes the analysis of those files: bump
`ANALYSIS_VERSION` (10 did). A track analysed at 9 stays current when it has
1, 2 or 6 channels, whose downmix is unchanged
(`fp_analysis::cache::analysis_is_current`).

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
(operator feedback 4, Q1), but only when the file stores the count: a short
length would end a VBR track early, so an estimate is never used. The rule
is an allowlist of symphonia readers known to read a stored count
(`symph.rs::stored_length`): FLAC (STREAMINFO), WAV and AIFF (the data
chunk), CAF, MP4/M4A and Ogg (the last page's granule, so Vorbis and Opus).
An MPEG stream (MP1/2/3) has one only when its first frame (the first
Layer III sync after any ID3v2 tag) is a length frame: `Xing` or `Info`
right after the side information (9, 17 or 32 bytes by version and channel
mode), or `VBRI` 32 bytes after the header, where symphonia reads them;
without one symphonia estimates the length from the first frames' bitrate.
The check reads the first 4 KiB after any ID3v2 tag when the file is
opened. Every other reader gets no
length: raw AAC (ADTS), whose count symphonia estimates from a sample of
frame sizes, Matroska/WebM and any reader added later. WavPack sums its
block headers, and Monkey's Audio and DSD read their headers' sample
counts.

symphonia's MPEG reader does more with its estimate than report it: it
takes it as the end of the stream, so every packet past it is marked to be
trimmed whole (`trim_end`, decoded to nothing) and a seek past it is out of
range. A VBR stream whose first frames run at a higher bitrate than the
rest would stop early in playback, and its analysed length would be short.
The reader estimates only on a seekable stream, so an MPEG stream without a
length frame is probed a second time through a source that reports itself
unseekable until the probe is done (`symph.rs::probe`). It then has no
length and reads to the last frame; seeking works as usual. Without a LAME
tag the encoder delay and padding are not known, so they stay in the
decoded audio (a few tens of milliseconds), as in any decoder. The ADTS
reader keeps its estimate only as a length and neither trims nor bounds
seeks by it.

## symphonia and Opus

symphonia decodes WAV, AIFF, CAF, FLAC, MP1/2/3, AAC, ALAC, Vorbis and
Matroska. Its codec registry (`opus::codecs`) is symphonia's own plus an Opus
decoder (`opus::OpusDecoder`) built on `opus-decoder`. It decodes at 48 kHz,
applies the header's output gain, and supports mono and stereo (mapping
family 0).

Decoders are opened with gapless decoding on (symphonia's default): each
decoder removes the encoder delay and end padding the container marks on its
packets (`trim_start`, `trim_end`), so the Opus decoder does it too. The
backend must not trim again (`tests/it/gapless.rs`).

The Opus **pre-skip** (RFC 7845 §4.2) is not marked that way: symphonia
0.6 keeps it as the track's delay in Ogg, whose granule positions count it,
and Matroska shifts its timestamps by `CodecDelay` rounded to its 1 ms
resolution. `SymphoniaDecoder` therefore drops exactly the pre-skip of the
`OpusHead` from the start of the stream in both containers, and shifts Ogg
Opus timestamps back by it when seeking and placing packets
(`ts_offset_secs`). A click comes out where it went in (`tests/it/opus.rs`).

**Seeking** (`SymphoniaDecoder::seek`): decoding restarts a codec's pre-roll
before the target, 80 ms for Opus (RFC 7845 §4.6) and the longest block
(8192 frames) for Vorbis, whose first packet after a reset only primes the
overlap. Each decoded packet is then placed by its timestamp (its frames end
where `pts + dur` ends, or start at `pts` when the container gives no
duration, as Matroska blocks), and the frames before the target are dropped, so the
output starts on the target frame whatever the decoder yields first. A seek
past the end is the end of the stream, as for the other backends, also in a
Matroska file written to a pipe (no cues, segment size unknown), where the
reader looks for the target by reading on and runs into the end of the file.

A seek whose pre-roll reaches the start of the stream (a restart from 0, a
target within the first 80 ms of an Opus file or the first 8192 frames of a
Vorbis file, or exactly 0 for other codecs) drops the pre-skip and the frames
before the target by count, as on the first play, so a restart is
sample-identical to it. Timestamps would place them only to the container's
precision: Matroska's are whole milliseconds and its `CodecDelay` is rounded
to them, and a test file restarted that way came out 648 frames short. A
reader that has not moved since it was opened (nothing read, no seek) is used
as it is; otherwise the file is opened again (`tests/it/opus.rs`, the unit tests
in `symph.rs`).

symphonia seeks a Matroska file without cues (a live recording) only forward.
A seek to a timestamp below what the reader has already read or landed on
(the end of the last block read: its duration, or the frames it decoded to,
rounded up to the next tick, when it has none), which such a reader answers by landing past the target,
opens the file again and seeks forward from the start, so a target inside the
block read last is not missed. A forward seek never opens it again. Opening
the file again checks that it still holds the same track (id, codec and sample
rate); a file replaced on disk makes the seek fail instead of being read with
the old decoder.

symphonia's Matroska reader is vendored (`vendor/symphonia-format-mkv`,
through `[patch.crates-io]`) with two seeking fixes
(`vendor/symphonia-format-mkv/VENDORED.md`). Version 0.6.1 took a cue
point's time, divided once too often, as its cluster's timestamp, so a seek
through any cue but the first skipped the whole cluster and landed on the next
one (a FLAC file muxed with 5 s clusters came out about 4.7 s late at 10 s);
the cluster's timestamp is now read from the cluster. And its blocks carry no
duration (muxers write neither `BlockDuration` nor `DefaultDuration` for
audio), so the seek landed on the first block starting at or after the target,
up to a block late for a codec without pre-roll (FLAC, PCM); it now lands on
the block that holds the target. In a file with video, whose cues index the
video's frames (often several per cluster) and whose clusters start at its
keyframes, the audio block that holds the target can start in a cluster
before the cued one, several before when clusters are shorter than an audio
block. So the clusters of the cues at or before the target (from the audio
track's own last cue, if any) are tried from the last back, until one reaches
an audio block at or before the target: usually the cued cluster or the one
before it, and never a reopening of the file. The frames before the target are then dropped
by timestamp, to the container's precision: Matroska's timestamps are whole
milliseconds, so a seek in a 44.1 kHz FLAC or PCM file lands within about 22
frames of the target (`tests/it/matroska.rs`, with PCM files built in the test).

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

## Encoded bitstreams stored as PCM

A DTS-CD rip is a DTS stream written into the samples of a 16-bit stereo PCM
file (WAV, FLAC, AIFF); played as PCM it is loud noise. When symphonia opens
a lossless 16-bit stereo file, `FileDecoder::open` decodes the first 65536
frames after any leading digital silence (up to 262144 frames of it are
skipped; 256 KiB of samples), gives them to `encoded::detect` and seeks back
to the start. `detect` looks for the DTS core sync words (16- and 14-bit,
big- and little-endian) and for the IEC 61937 preamble (`0xF872 0x4E1F`, in
both byte orders) in the interleaved sample words, and validates the header
after each: for DTS, FTYPE = 1, SHORT = 31, at least 5 blocks and FSIZE (the
frame size, converted to words for the 14-bit packed forms); for IEC 61937, a
known Pc data type and a Pd length. It needs three valid syncs at one exact
spacing no shorter than the frame, so neither an accidental word nor a
periodic waveform that happens to contain the sync words matches.
`FileDecoder::open_header` skips the sniff for callers that only read the
length. A match fails the open
with `encoded bitstream (DTS) stored as PCM; not supported` (or `IEC 61937`)
and one warning in the log, so analysis and playback refuse the file; the
track shows as unreadable and only the log says why (`FileState::Unreadable`
carries no text). Decoding DTS or AC-3 is out of scope.

## Not supported

Musepack, TTA, WMA and AC-3: no pure-Rust decoder could be verified. These
files show as unreadable.
