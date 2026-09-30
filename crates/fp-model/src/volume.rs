//! The volume fader's curve, shared by the on-screen fader and MIDI faders.

/// The fader's range below 0 dB; the bottom of the travel is silence.
const FADER_RANGE_DB: f32 = 60.0;

/// Fader travel (0 bottom … 1 top) to linear gain: linear in dB, 0 dB at the top.
pub fn gain_from_fader(pos: f32) -> f32 {
    if pos <= 0.0 || pos.is_nan() {
        return 0.0;
    }
    let db = FADER_RANGE_DB * (pos.min(1.0) - 1.0);
    10f32.powf(db / 20.0)
}

/// Linear gain to fader travel (the inverse of [`gain_from_fader`]).
pub fn fader_from_gain(gain: f32) -> f32 {
    if gain <= 0.0 || gain.is_nan() {
        return 0.0;
    }
    (1.0 + 20.0 * gain.min(1.0).log10() / FADER_RANGE_DB).clamp(0.0, 1.0)
}
