# Vendored `opus-decoder` 0.1.1

Source: crates.io `opus-decoder` 0.1.1 (MIT OR Apache-2.0,
<https://github.com/TadeuszWolfGang/Rusopus>), used through
`[patch.crates-io]` in the workspace `Cargo.toml`.

## Local change

- `src/celt/kiss_fft.rs`: `KissFft::ifft` and `KissFft::fft` computed the
  transform as a direct O(n²) DFT with a modulo per term, which made the
  decoder spend about 5.5 ms per 20 ms packet (4× real time). They now use a
  mixed-radix decimation-in-time FFT (radices 4, 2, 3, 5, then any other
  prime with a generic butterfly), the algorithm of KISS FFT, in safe Rust.
  The output matches the direct DFT to within float rounding; the decoder's
  output is checked against a reference decoder in
  `crates/fp-decode/tests/opus.rs`.

Drop this copy once a release upstream has a fast FFT.
