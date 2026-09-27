# Audio formats (design)

**Parent spec:** [`2026-09-25-fauste-player-design.md`](2026-09-25-fauste-player-design.md), §6 (analysis) and the decoding crate (`fp-decode`).

**Goal:** play every format a radio library is likely to hold, not only what symphonia decodes. That covers DSD (DSF and DFF), Opus, WavPack and Monkey's Audio (APE). Everything plays through the same path as today: stereo `f32` at the file's own rate, with seeking, analysis and tags.

## F1. What is supported

| Format | Extensions | Decoder | Notes |
|---|---|---|---|
| WAV, AIFF, CAF | `wav`, `aif`, `aiff`, `caf` | symphonia | unchanged |
| FLAC | `flac` | symphonia | unchanged |
| MP1/MP2/MP3 | `mp1`, `mp2`, `mp3` | symphonia | unchanged |
| AAC, ALAC | `m4a`, `mp4`, `aac` | symphonia | unchanged |
| Ogg Vorbis | `ogg`, `oga` | symphonia | unchanged |
| Matroska/WebM | `mka`, `mkv`, `webm` | symphonia | plus Opus inside |
| **Opus** | `opus` (and in Ogg or WebM) | `opus-decoder`, a pure-Rust decoder that forbids `unsafe`, registered as a symphonia codec | always decoded at 48 kHz |
| **WavPack** | `wv` | `wavicle` (pure Rust, `forbid(unsafe_code)`) | lossless and hybrid (lossy part); 8-bit files are refused |
| **Monkey's Audio** | `ape` | `ape-decoder` (pure Rust, `deny(unsafe_code)`) | frame-accurate seek |
| **DSD** | `dsf`, `dff` | our own reader and DSD-to-PCM converter | DSD64 to DSD512; DST-compressed DFF is refused |

Every added crate is MIT or Apache-2.0 licensed. A GPL WavPack codec was rejected, because the project has no licence yet and `cargo deny` allows none.

**Not supported yet:** Musepack, TTA, WMA and AC-3. The only pure-Rust decoders are at version 0.0.x and cannot be verified here (there are no encoders or reference files). They show as unreadable files, as today.

## F2. Architecture (`fp-decode`)

- **`FileDecoder` fronts several backends.** Each backend exposes the same interface: open, sample rate, bits per sample, channels, frames hint, `seek(secs)`, and `next_block` (interleaved stereo `f32`, downmixed as today).
- **The backend is chosen by the file's magic bytes,** with the extension as a hint:
  - `DSD ` means DSF;
  - `FRM8` … `DSD ` means DFF;
  - `wvpk` means WavPack;
  - `MAC ` means APE;
  - anything else goes to symphonia.
- **The engine, analysis and bit-perfect code do not change:**
  - they see a rate, a bit depth (`None` for lossy and for DSD, which is converted) and blocks;
  - DSD is never bit-perfect: it is converted to PCM;
  - WavPack and APE are lossless integer PCM, so they can be.

## F3. DSD to PCM

- **Input:** DSF (little-endian bit order, per-channel blocks of 4096 bytes) and uncompressed DFF (DSDIFF, MSB first, interleaved bytes). DSD rates are 64·44.1 kHz and higher.
- **Output:** PCM at the DSD rate ÷ 32. DSD64 gives 88.2 kHz, DSD128 176.4 kHz, and so on. The engine resamples as it does for any file.
- **Filter:** a linear-phase low-pass FIR at the DSD rate, applied to the ±1 bit stream and decimated by 32, computed through per-byte lookup tables (8 taps per table). This is the usual method, and its cost does not depend on the filter length.
  - Pass band to 20 kHz (±0.1 dB).
  - The stop band from 0.5 × the output rate is attenuated by at least 100 dB, which removes the DSD noise-shaping band before decimation.
- **Level:** the ±1 bit stream maps to ±1.0, with unity filter gain at DC. A 50 %-modulation DSD sine (the SACD reference level) reads −6 dBFS.
- **Seeking:** DSF to the containing block, DFF by byte offset. The filter history is primed from the bytes just before the target.
- **Tags:** the DSF ID3v2 chunk (pointed to from the header) is read. DFF has no standard tags, so the file name is used.

## F4. Everywhere else

- **File filters:** adding files, drag and drop, folder scans and playlist imports accept the new extensions.
- **Desktop entry:** the MIME types include the new formats.
- **Documentation:** the README format list, the user guide (getting started, playlists) and the technical docs (decoding, analysis).

## F5. Verification

- **WavPack:** encoded in the tests with `wavicle`'s own encoder (a dev-dependency). The decoded samples must equal the source exactly (lossless) for 16- and 24-bit.
- **Opus:** encoded in the tests with a pure-Rust Opus encoder (a dev-dependency). A 1 kHz tone must decode at the right frequency and level (±0.5 dB), and seeking must land within one frame.
- **APE:** a small reference file with known content, checked against its stored MD5 (the decoder's `verify_md5`) and against its samples.
- **DSD:** a DSD64 stream generated in the test by a 5th-order sigma-delta modulator from a 1 kHz sine at 50 % modulation. It must decode to 88.2 kHz, at −6 dBFS ±0.2 dB, with THD+N below −80 dB. The DSF and DFF readers are checked on synthetic files, including seeking and the refusal of DST.
