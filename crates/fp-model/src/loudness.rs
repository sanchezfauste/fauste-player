//! Loudness normalization (loudness normalization spec, 2026-10-08): what
//! the analysis measured of a track, and the pure rules that turn it into
//! a fixed gain per source (N7–N13, added with the gain rules).

use serde::{Deserialize, Serialize};

use crate::state::AppState;

/// The version of the loudness measurement (N6). Bump it when the
/// measurement changes: the loudness-only pass then measures every track
/// again, and nothing else is analysed again.
pub const LOUDNESS_VERSION: u32 = 1;

/// The integrated loudness and true peak of a whole file (spec §3).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LoudnessMeasurement {
    /// ITU-R BS.1770 gated integrated loudness, LUFS; `None` when no block
    /// passes the absolute gate (a silent file).
    pub integrated_lufs: Option<f32>,
    /// The largest true peak of both channels, dBTP.
    pub true_peak_dbtp: f32,
}

impl LoudnessMeasurement {
    /// Whether every number is finite. One that is not (a corrupt cache
    /// entry) is no measurement: JSON cannot hold it.
    pub fn is_finite(&self) -> bool {
        self.integrated_lufs.is_none_or(f32::is_finite) && self.true_peak_dbtp.is_finite()
    }
}

/// N22: playable tracks with no measurement that the loudness-only pass
/// (or their first analysis) has not tried yet.
pub fn loudness_unmeasured(state: &AppState) -> usize {
    state
        .library
        .iter()
        .filter(|t| {
            t.file_state.is_playable()
                && t.loudness.is_none()
                && t.loudness_pass != Some(LOUDNESS_VERSION)
        })
        .count()
}
