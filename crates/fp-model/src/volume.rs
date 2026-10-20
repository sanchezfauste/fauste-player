//! The volume fader's law, shared by the on-screen fader, MIDI faders and
//! the remote API's `fader` travel.
//!
//! A mixing-desk fader law: `dB = 192 * (travel^(1/8) - 1)`. It is fine near
//! 0 dB (about 24 dB per unit of travel at the top) and steepens towards the
//! bottom, so lowering the fader a little gives a small drop (90 % is about
//! -2.5 dB, 50 % about -15.9 dB, 10 % about -48 dB). Zero travel is silence.

/// The curve's scale: `dB = DB_SCALE * (travel^(1/DB_ROOT) - 1)`.
const DB_SCALE: f32 = 192.0;

/// The curve's root, see [`DB_SCALE`].
const DB_ROOT: f32 = 8.0;

/// Fader travel (0 bottom … 1 top) to decibels, 0 dB at the top. Travel at
/// or below 0 (or NaN) is `-inf`; travel above 1 counts as 1.
pub fn travel_to_db(pos: f32) -> f32 {
    if pos <= 0.0 || pos.is_nan() {
        return f32::NEG_INFINITY;
    }
    DB_SCALE * (pos.min(1.0).powf(1.0 / DB_ROOT) - 1.0)
}

/// Decibels to fader travel (the inverse of [`travel_to_db`]), clamped to
/// `0..=1`; a level of `-inf` dB, NaN or at or below the curve's bottom
/// (-192 dB) is 0 and 0 dB or more is 1.
pub fn db_to_travel(db: f32) -> f32 {
    if db.is_nan() {
        return 0.0;
    }
    (1.0 + db / DB_SCALE).clamp(0.0, 1.0).powf(DB_ROOT)
}

/// Fader travel (0 bottom … 1 top) to linear gain: 0 dB at the top, silence
/// at 0 (or NaN).
pub fn gain_from_fader(pos: f32) -> f32 {
    if pos <= 0.0 || pos.is_nan() {
        return 0.0;
    }
    10f32.powf(travel_to_db(pos) / 20.0)
}

/// Linear gain to fader travel (the inverse of [`gain_from_fader`]).
pub fn fader_from_gain(gain: f32) -> f32 {
    if gain <= 0.0 || gain.is_nan() {
        return 0.0;
    }
    db_to_travel(20.0 * gain.min(1.0).log10())
}
