# itr06 plan D: multichannel downmix

Goal: mix every surround layout down to stereo by its channel positions (I10). Multichannel playout stays out of scope.

## I10 layout-aware downmix
- `crates/fp-decode/src/downmix.rs`: `Downmix` built from symphonia's `Channels` (count fallback), applied per frame in `symph.rs`; `downmix(frame)` keeps the count-based entry for WavPack, APE and DSD.
- Fixes 7.1 (side channels dropped), 6.1, quad (was L R C LFE), 2.1 (LFE folded in). 5.1 keeps its coefficients.
- Tests: unit tests per layout in `downmix.rs`; `crates/fp-decode/tests/surround.rs` (generated extensible WAVs); `crates/fp-analysis/tests/analyze.rs` (side channels count in loudness).
- `ANALYSIS_VERSION` 9 to 10: cached values of multichannel files other than 5.1 change.
- Docs: `docs/technical/decoding.md`, user guide formats paragraph, audio formats spec F2.
- No UI strings.
