#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Probes for the audio path audit (feedback 2, O26):
//! `docs/technical/audio-path-audit.md` §4.1. The shared-mode cpal path
//! converts each sample with `T::from_sample` (`render_converted` in
//! `src/cpal_backend.rs`), which is the conversion probed here. A probe
//! that shows a confirmed defect is `#[ignore]`d with the finding's id until
//! its fix task removes the attribute.

use cpal::Sample;

const I24_MAX: i32 = (1 << 23) - 1;
const I24_MIN: i32 = -(1 << 23);

// A7: fails until the fix task. cpal's `I24::from_sample(f32)` is
// `I24::new_unchecked((s * 2^23) as i32)`: nothing clamps it to 24 bits, so
// full scale and above leave the 24-bit range and wrap to the opposite
// sign on the device (the low 24 bits are what an S24 device plays).
#[test]
#[ignore = "A7: fails until the fix task"]
fn a7_the_24_bit_conversion_clips_instead_of_wrapping() {
    for s in [1.0f32, 1.5, -1.5] {
        let v = cpal::I24::from_sample(s).inner();
        assert!(
            (I24_MIN..=I24_MAX).contains(&v),
            "{s} becomes {v} (low 24 bits read as {})",
            (v << 8) >> 8
        );
    }
}

// A7: fails until the fix task. cpal's `i16::from_sample(f32)` truncates
// toward zero instead of rounding: an error of up to one step, biased
// toward zero (twice the error of rounding).
#[test]
#[ignore = "A7: fails until the fix task"]
fn a7_the_16_bit_conversion_rounds_to_the_nearest_step() {
    for (s, nearest) in [(100.75f32, 101i16), (-100.75, -101), (0.6, 1)] {
        let v = i16::from_sample(s / 32_768.0);
        assert_eq!(v, nearest, "{s} steps");
    }
}

#[test]
fn integer_conversions_saturate_and_silence_nan_except_24_bit() {
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
