//! Loudness normalization (loudness normalization spec, 2026-10-08): what
//! the analysis measured of a track, and the pure rules that turn it into
//! a fixed gain per source (N7–N13, added with the gain rules).

use serde::{Deserialize, Serialize};

use crate::config::{LoudnessConfig, OutputDevice};
use crate::live::{Holder, configured_route};
use crate::state::AppState;
use crate::track::Track;

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

/// The operator's per-track loudness setting (spec §3).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct LoudnessEdit {
    /// Plays the track unchanged, whatever its measurement.
    pub excluded: bool,
    /// Added to the gain, dB; limited to ± `max_trim_db` (N10).
    pub trim_db: f32,
}

/// The bound that limited a boost (N7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GainLimit {
    TruePeak,
    MaxBoost,
}

/// Why a source has the gain it has (spec §3). Every status but `Applied`
/// has a gain of 0 dB.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalizationStatus {
    Applied { limited: Option<GainLimit> },
    Off,
    Excluded,
    NotMeasured,
    Silent,
    BitPerfect,
}

/// The fixed gain one source gets (spec §3, N7).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Normalization {
    pub gain_db: f32,
    pub status: NormalizationStatus,
}

impl Default for Normalization {
    /// No gain: what every source had before normalization existed.
    fn default() -> Self {
        Self::none(NormalizationStatus::Off)
    }
}

impl Normalization {
    fn none(status: NormalizationStatus) -> Self {
        Self {
            gain_db: 0.0,
            status,
        }
    }

    /// The linear factor (`10^(dB/20)`); exactly 1.0 for 0 dB, so an
    /// unnormalized source stays bit-exact, and 1.0 for anything that is
    /// not a usable number.
    pub fn linear_gain(&self) -> f32 {
        if self.gain_db == 0.0 {
            return 1.0;
        }
        let g = 10f32.powf(self.gain_db / 20.0);
        if g.is_finite() && g > 0.0 { g } else { 1.0 }
    }
}

/// N7: the gain `track` gets under `config` on a device that is
/// `bit_perfect` or not.
pub fn normalization(config: &LoudnessConfig, track: &Track, bit_perfect: bool) -> Normalization {
    use NormalizationStatus as S;
    if bit_perfect {
        return Normalization::none(S::BitPerfect);
    }
    if !config.normalize {
        return Normalization::none(S::Off);
    }
    if track.loudness_edit.excluded {
        return Normalization::none(S::Excluded);
    }
    let Some(measured) = track.loudness else {
        return Normalization::none(S::NotMeasured);
    };
    let Some(lufs) = measured.integrated_lufs else {
        return Normalization::none(S::Silent);
    };
    let trim = trim_in_use(config, &track.loudness_edit);
    let wanted = config.target_lufs - lufs + trim;
    // The ceiling and the maximum boost only limit boosts (Q2).
    let peak_room = (config.true_peak_ceiling_dbtp - measured.true_peak_dbtp).max(0.0);
    let (gain, limited) = if wanted <= 0.0 {
        (wanted, None)
    } else {
        // Not a number or below 0 (a configuration not validated): no boost.
        let boost = if config.max_boost_db.is_finite() {
            config.max_boost_db.max(0.0)
        } else {
            0.0
        };
        if peak_room < wanted && peak_room <= boost {
            (peak_room, Some(GainLimit::TruePeak))
        } else if boost < wanted {
            (boost, Some(GainLimit::MaxBoost))
        } else {
            (wanted, None)
        }
    };
    let mut gain_db = (gain * 100.0).round() / 100.0;
    // Rounding must not take a boost above the ceiling, whatever limited
    // it: such a boost steps down to the hundredth at or below the room.
    // Both steps allow for f32 noise, so a room exactly on a hundredth
    // keeps it.
    if gain_db > 0.0 && gain_db > peak_room + ROOM_TOLERANCE_DB {
        gain_db = ((peak_room + ROOM_TOLERANCE_DB) * 100.0).floor() / 100.0;
    }
    if !gain_db.is_finite() {
        return Normalization::none(S::NotMeasured);
    }
    Normalization {
        gain_db,
        status: S::Applied { limited },
    }
}

/// How far, in dB, f32 arithmetic may put a rounded gain above the room
/// the true-peak ceiling leaves without that counting as passing it
/// (N7): far below the hundredth the gain is rounded to.
const ROOM_TOLERANCE_DB: f32 = 1e-5;

/// The largest trim N7 uses and N10 allows, dB: `max_trim_db`, or none
/// when it is not a finite number of at least 0 (a configuration not
/// validated).
pub fn max_trim_in_use(config: &LoudnessConfig) -> f32 {
    if config.max_trim_db.is_finite() {
        config.max_trim_db.max(0.0)
    } else {
        0.0
    }
}

/// The trim N7 uses for `edit`: the stored one within ± `max_trim_db`
/// (a maximum lowered since limits it, `max_trim_in_use`); one that is
/// not a number counts as none.
pub fn trim_in_use(config: &LoudnessConfig, edit: &LoudnessEdit) -> f32 {
    let max_trim = max_trim_in_use(config);
    if edit.trim_db.is_finite() {
        edit.trim_db.clamp(-max_trim, max_trim)
    } else {
        0.0
    }
}

/// N8: whether `holder` plays on a bit-perfect device: the running
/// settings of its placement once the engine reported them (live settings
/// L3), otherwise its configured route (no route: the default output, not
/// bit-perfect). For a player's Main this agrees with
/// `bit_perfect_player` whenever the engine has placed it.
pub fn holder_bit_perfect(state: &AppState, holder: Holder) -> bool {
    if let Some(running) = state
        .live
        .placement
        .get(&holder)
        .and_then(|device| state.live.devices.get(device))
    {
        return running.bit_perfect;
    }
    configured_route(&state.config, holder).is_some_and(|r| {
        state.config.outputs.bit_perfect.contains(&OutputDevice {
            backend: r.backend,
            device: r.device,
        })
    })
}

/// N7 for `track` played by `holder`.
pub fn normalization_for(state: &AppState, holder: Holder, track: &Track) -> Normalization {
    normalization(
        &state.config.loudness,
        track,
        holder_bit_perfect(state, holder),
    )
}
