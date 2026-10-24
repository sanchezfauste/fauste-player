#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Probes for the audio path audit (feedback 2, O26):
//! `docs/technical/audio-path-audit.md` §4.1. They record what cpal's own
//! `T::from_sample` does. The shared-mode path no longer uses it: since A7,
//! `render_converted` in `src/cpal_backend.rs` converts with the private
//! `OutputSample`, whose tests live next to it.

use cpal::Sample;

const I24_MAX: i32 = (1 << 23) - 1;

// A7 (cpal's `I24` conversion wraps at full scale and `i16` truncates) is
// fixed by `OutputSample` in `src/cpal_backend.rs`; its probes
// `a7_the_24_bit_conversion_clips_instead_of_wrapping` and
// `a7_the_16_bit_conversion_rounds_to_the_nearest_step` live there, next
// to the private conversion. The tests below document what cpal itself does.

#[test]
fn cpal_conversions_saturate_and_silence_nan_except_24_bit() {
    assert_eq!(i16::from_sample(1.5f32), i16::MAX);
    assert_eq!(i16::from_sample(-1.5f32), i16::MIN);
    assert_eq!(i16::from_sample(f32::NAN), 0);
    assert_eq!(i32::from_sample(1.5f32), i32::MAX);
    assert_eq!(i32::from_sample(f32::NAN), 0);
    assert_eq!(cpal::I24::from_sample(f32::NAN).inner(), 0);
    // The largest value integer PCM can hold converts exactly.
    assert_eq!(
        cpal::I24::from_sample(I24_MAX as f32 / 8_388_608.0).inner(),
        I24_MAX
    );
    // Floats go out as they are, overs and NaN included.
    assert_eq!(f32::from_sample(1.5f32), 1.5);
    assert!(f32::from_sample(f32::NAN).is_nan());
}
