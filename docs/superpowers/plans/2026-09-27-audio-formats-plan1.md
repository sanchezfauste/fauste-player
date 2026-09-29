# Audio formats · Plan 1 — DSD, Opus, WavPack and Monkey's Audio

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans (or superpowers:subagent-driven-development) to implement this plan task by task.

**Goal:** decode DSD (DSF and DFF), Opus, WavPack and Monkey's Audio alongside everything symphonia already decodes, through the same `FileDecoder` interface. The engine, analysis and bit-perfect code need no changes.

**Spec:** [`docs/superpowers/specs/2026-09-27-audio-formats-design.md`](../specs/2026-09-27-audio-formats-design.md) (F1–F5).

## Global Constraints

- Our code keeps `forbid(unsafe_code)`, and every new dependency forbids or denies `unsafe` itself.
- New dependencies must be MIT, Apache-2.0 or BSD licensed, and `cargo deny` must pass.
- `indexing_slicing` is denied in fp-decode; there is no unwrap, expect or panic outside tests.
- An untrusted file never crashes anything: a malformed file becomes a decode error, which the app shows as "unreadable".
- Seeking lands within one decode block of the target, and `skip_frames` trims to the exact frame, as with symphonia.
- Documentation is kept in sync (CLAUDE.md), and the change goes in through a pull request.

## Review Focus

1. **Malformed headers:** truncated, lying about sizes, or zero channels or rate. There must be an error, never a panic or an allocation the size of a lie.
2. **A seek near the end** of a file of each format, and a seek to 0.
3. **DSD level and aliasing:** the stop band must hold at the output rate's Nyquist frequency.
4. **Memory:** WavPack and APE are read block by block or frame by frame, never whole files.
5. **Tags, duration and analysis** work for each new format, and extensions are accepted in every file entry point.

---

### Task 1: Backends behind `FileDecoder`

- `fp-decode` gets a `Backend` enum. The symphonia code moves to `symph.rs` unchanged.
- The backend is chosen by `probe(first bytes, extension)`.
- **Tests:**
  - `each_format_is_recognised_by_its_magic_bytes`: DSF, DFF, WavPack, APE, and otherwise symphonia;
  - the existing decoding tests stay green.

### Task 2: DSD

- `dsd/dsf.rs` and `dsd/dff.rs` readers. Header validation limits every size by the file length.
- `dsd/convert.rs`:
  - a byte-table FIR decimator (÷32), using a Kaiser-windowed sinc;
  - pass band to 20 kHz, stop band from half the output rate at ≥ 100 dB.
- **Tests:**
  - `dsd64_sine_decodes_at_the_sacd_reference_level`;
  - `dsd_conversion_keeps_distortion_below_minus_80_db`;
  - `the_decimation_filter_rejects_the_noise_shaping_band`;
  - `dsf_and_dff_readers_agree`;
  - `seeking_a_dsd_file_lands_on_the_frame`;
  - `dst_compressed_dff_is_refused`;
  - `a_truncated_dsd_header_is_an_error`.

### Task 3: WavPack

- `wavpack.rs` reads block by block (32-byte header, then the body) and decodes each block with `wavicle::decode_stream`, since WavPack blocks are independent.
- Seeking uses the block index in each header.
- **Tests (fixtures encoded with `wavicle`'s encoder):**
  - `wavpack_16_and_24_bit_decode_bit_exact`;
  - `seeking_wavpack_lands_on_the_frame`;
  - `a_corrupt_wavpack_block_is_an_error`.

### Task 4: Monkey's Audio

- `ape.rs` decodes frame by frame with `ape-decoder`, seeks with `ApeDecoder::seek`, and reads bits and rate from `ApeInfo`.
- **Tests:** a reference `.ape` fixture with a known tone:
  - `ape_decodes_and_verifies_its_md5`;
  - `seeking_ape_lands_on_the_frame`.

### Task 5: Opus

- An Opus decoder for symphonia's codec registry, wrapping `opus-decoder`. It is used for Ogg and Matroska Opus tracks, and is registered in the registry `FileDecoder` uses.
- **Tests (fixtures encoded with `opus-pure`):**
  - `an_ogg_opus_tone_decodes_at_48_khz_with_its_level`;
  - `seeking_opus_lands_within_a_frame`.

### Task 6: Entry points, tags and docs

- **Extensions** in `ui/files.rs` (additions, drops, folder scans, imports) and in the rfd filters.
- **MIME types** in the `.desktop` file.
- **DSF tags:** the ID3v2 chunk, read with lofty's ID3v2 reader on the chunk bytes.
- **Tests:**
  - `new_formats_are_accepted_as_audio_files`;
  - `dsf_tags_are_read`.
- **Docs:** README feature and format lists, the user guide (getting started, playlists) and the technical docs (a new decoding section).
- Full checks, a fresh review, fixes, then the pull request.
