//! Wall-clock time for automation, and a small seeded random generator.
//! The model never reads the clock itself: the conductor sets
//! `AppState::wall_now` every tick, and tests set it directly.

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// A moment in UTC, in milliseconds since the Unix epoch.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct WallTime(pub i64);

impl WallTime {
    /// The system clock now; a clock before 1970 gives `WallTime(0)`.
    pub fn now() -> Self {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
            .unwrap_or(0);
        Self(millis)
    }

    /// `secs` later (earlier when negative), saturating; a non-finite
    /// `secs` changes nothing.
    pub fn plus_secs(self, secs: f64) -> Self {
        if !secs.is_finite() {
            return self;
        }
        let millis = (secs * 1000.0).round();
        // `as` saturates at the i64 bounds.
        Self(self.0.saturating_add(millis as i64))
    }

    /// Milliseconds from `earlier` to `self`, saturating; negative when
    /// `earlier` is in fact later.
    pub fn millis_since(self, earlier: Self) -> i64 {
        self.0.saturating_sub(earlier.0)
    }
}

/// SplitMix64: small, seedable and deterministic, enough for the fair
/// choice and the step counts of a rotation.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SplitMix64(pub u64);

impl SplitMix64 {
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A number in `0..n`; 0 when `n` is 0.
    pub fn below(&mut self, n: u64) -> u64 {
        if n == 0 { 0 } else { self.next_u64() % n }
    }

    /// A number in `min..=max`, the bounds swapped when `min > max`.
    pub fn in_range(&mut self, min: u8, max: u8) -> u8 {
        let (lo, hi) = if min <= max { (min, max) } else { (max, min) };
        let span = u64::from(hi - lo) + 1;
        // `below(span)` is below 256, so the sum fits.
        lo.saturating_add(u8::try_from(self.below(span)).unwrap_or(0))
    }
}
