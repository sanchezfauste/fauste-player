# Monkey's Audio fixtures

`multiframe_16s_c2000.ape` (16-bit stereo, 3 s, several frames) and
`sine_24s_c2000.ape` (24-bit stereo, 1 s) are synthetic test signals from the
test fixtures of the `ape-decoder` crate
(<https://github.com/OMBS-IO/ape-decoder>, `tests/fixtures/ape`), licensed MIT
OR Apache-2.0.

The tests compare the decoded samples with an FNV-1a hash of the matching
reference WAV files from the same repository (`tests/fixtures/ref`), which
are not copied here.
