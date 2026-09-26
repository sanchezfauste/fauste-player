//! Application configuration. Every tunable value lives here with a
//! documented default; nothing product-related is a hardcoded constant.

use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::ids::PlayerId;
use crate::player::PlayMode;
use crate::shortcuts::{Shortcut, default_shortcuts};

const MIB: u64 = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub players: PlayersConfig,
    pub analysis: AnalysisSettings,
    pub outputs: OutputsConfig,
    pub ui: UiConfig,
    pub limits: Limits,
    pub tuning: Tuning,
    pub cartwall: CartwallConfig,
    pub shortcuts: Vec<Shortcut>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            players: PlayersConfig::default(),
            analysis: AnalysisSettings::default(),
            outputs: OutputsConfig::default(),
            ui: UiConfig::default(),
            limits: Limits::default(),
            tuning: Tuning::default(),
            cartwall: CartwallConfig::default(),
            shortcuts: default_shortcuts(),
        }
    }
}

/// Cartwall defaults (Phase 2 spec P2.3).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CartwallConfig {
    /// Grid of a new cart page.
    pub default_rows: u16,
    pub default_cols: u16,
}

impl Default for CartwallConfig {
    fn default() -> Self {
        Self {
            default_rows: 2,
            default_cols: 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PlayersConfig {
    pub count: usize,
    pub default_mode: PlayMode,
    pub fade_ms: u32,
    pub auto_segue: bool,
    pub end_warning_secs: f64,
}

impl Default for PlayersConfig {
    fn default() -> Self {
        Self {
            count: 4,
            default_mode: PlayMode::Continuous,
            fade_ms: 1000,
            auto_segue: true,
            end_warning_secs: 10.0,
        }
    }
}

/// Parameters of automatic marker detection (spec §6).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AnalysisSettings {
    pub silence_threshold_db: f64,
    pub segue_threshold_db: f64,
    pub segue_max_secs: f64,
    pub outro_drop_db: f64,
    pub outro_max_secs: f64,
    pub markers_min_duration_secs: f64,
    pub peak_bucket_ms: u32,
    pub rms_window_ms: u32,
    pub cover_thumb_px: u32,
}

impl Default for AnalysisSettings {
    fn default() -> Self {
        Self {
            silence_threshold_db: -40.0,
            segue_threshold_db: -18.0,
            segue_max_secs: 8.0,
            outro_drop_db: 6.0,
            outro_max_secs: 30.0,
            markers_min_duration_secs: 60.0,
            peak_bucket_ms: 10,
            rms_window_ms: 50,
            cover_thumb_px: 128,
        }
    }
}

/// Where a bus is sent: a backend, a device and the first channel of a stereo pair.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Route {
    pub backend: String,
    pub device: String,
    pub first_channel: u16,
}

/// One output device, as a bit-perfect setting names it (Phase 4 spec B1).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OutputDevice {
    pub backend: String,
    pub device: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerRoutes {
    pub player: PlayerId,
    pub main: Option<Route>,
    pub cue: Option<Route>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct OutputsConfig {
    /// Backend id; `None` means the platform default.
    pub backend: Option<String>,
    pub sample_rate: u32,
    pub buffer_frames: u32,
    pub routes: Vec<PlayerRoutes>,
    /// Where the cartwall plays (Phase 2 spec P2.4).
    #[serde(default)]
    pub cartwall: CartwallRoutes,
    /// Devices played bit-perfect: exclusive access, and the stream rate
    /// follows the files (Phase 4 spec B1, B3).
    #[serde(default)]
    pub bit_perfect: Vec<OutputDevice>,
}

/// The cartwall's outputs. Main falls back to the default output; without
/// a Cue route carts cannot be pre-listened.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct CartwallRoutes {
    pub main: Option<Route>,
    pub cue: Option<Route>,
}

impl Default for OutputsConfig {
    fn default() -> Self {
        Self {
            backend: None,
            sample_rate: 48_000,
            buffer_frames: 512,
            routes: Vec::new(),
            cartwall: CartwallRoutes::default(),
            bit_perfect: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    /// Name of a waveform colour in the theme palette.
    pub wave_color: String,
    pub music_dir: Option<PathBuf>,
    /// BCP-47 tag; `None` follows the OS locale.
    pub language: Option<String>,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            wave_color: "sand".to_owned(),
            music_dir: None,
            language: None,
        }
    }
}

/// Resource guards. Edited only in the config file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Limits {
    pub max_players: usize,
    pub max_cover_bytes: u64,
    pub max_cover_pixels: u32,
    pub max_state_file_bytes: u64,
    pub max_playlist_file_bytes: u64,
    pub backup_count: usize,
    /// Crash reports written per run (a contained, repeating panic is logged
    /// but does not fill the disk).
    pub max_crash_reports: usize,
    /// Largest cart page grid.
    pub max_cart_rows: u16,
    pub max_cart_cols: u16,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_players: 16,
            max_cover_bytes: 20 * MIB,
            max_cover_pixels: 8000,
            max_state_file_bytes: 50 * MIB,
            max_playlist_file_bytes: 10 * MIB,
            backup_count: 3,
            max_crash_reports: 20,
            max_cart_rows: 8,
            max_cart_cols: 16,
        }
    }
}

/// Engine internals. Edited only in the "advanced" part of the config file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Tuning {
    pub declick_ms: f64,
    pub pause_ramp_ms: f64,
    pub prebuffer_secs: f64,
    pub ready_threshold_ms: f64,
    pub mixer_headroom: f64,
    pub max_commands_per_block: usize,
    pub schedule_lead_ms: f64,
    pub conductor_tick_ms: f64,
    pub watchdog_timeout_ms: f64,
    /// Watchdog timeout before a newly opened stream delivers its first block.
    pub watchdog_startup_grace_ms: f64,
    pub reconnect_interval_ms: f64,
    pub gain_smoothing_ms: f64,
    pub save_debounce_ms: f64,
}

impl Default for Tuning {
    fn default() -> Self {
        Self {
            declick_ms: 5.0,
            pause_ramp_ms: 10.0,
            prebuffer_secs: 5.0,
            ready_threshold_ms: 500.0,
            mixer_headroom: 2.0,
            max_commands_per_block: 256,
            schedule_lead_ms: 200.0,
            conductor_tick_ms: 5.0,
            watchdog_timeout_ms: 500.0,
            watchdog_startup_grace_ms: 5000.0,
            reconnect_interval_ms: 2000.0,
            gain_smoothing_ms: 20.0,
            save_debounce_ms: 1000.0,
        }
    }
}

/// A value that was out of range and has been replaced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigWarning {
    pub field: &'static str,
    pub message: String,
}

impl fmt::Display for ConfigWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

/// Clamps `value` into `min..=max`. NaN (which fails every comparison) becomes `min`.
#[allow(clippy::neg_cmp_op_on_partial_ord)] // negated comparisons are deliberate: they catch NaN
fn clamp_to<T: PartialOrd + Copy + fmt::Display>(
    value: &mut T,
    min: T,
    max: T,
    field: &'static str,
    out: &mut Vec<ConfigWarning>,
) {
    let original = *value;
    if !(original >= min) {
        *value = min;
    } else if !(original <= max) {
        *value = max;
    } else {
        return;
    }
    out.push(ConfigWarning {
        field,
        message: format!("{original} is outside {min}..={max}; using {}", *value),
    });
}

impl Config {
    /// Brings every value into its valid range and reports what was changed.
    pub fn validate(&mut self) -> Vec<ConfigWarning> {
        let mut w = Vec::new();

        let l = &mut self.limits;
        clamp_to(&mut l.max_players, 1, 256, "limits.max_players", &mut w);
        clamp_to(
            &mut l.max_cover_bytes,
            MIB,
            500 * MIB,
            "limits.max_cover_bytes",
            &mut w,
        );
        clamp_to(
            &mut l.max_cover_pixels,
            256,
            30_000,
            "limits.max_cover_pixels",
            &mut w,
        );
        clamp_to(
            &mut l.max_state_file_bytes,
            MIB,
            1024 * MIB,
            "limits.max_state_file_bytes",
            &mut w,
        );
        clamp_to(
            &mut l.max_playlist_file_bytes,
            64 * 1024,
            1024 * MIB,
            "limits.max_playlist_file_bytes",
            &mut w,
        );
        clamp_to(&mut l.backup_count, 0, 20, "limits.backup_count", &mut w);
        clamp_to(
            &mut l.max_crash_reports,
            1,
            10_000,
            "limits.max_crash_reports",
            &mut w,
        );

        clamp_to(&mut l.max_cart_rows, 1, 64, "limits.max_cart_rows", &mut w);
        clamp_to(&mut l.max_cart_cols, 1, 64, "limits.max_cart_cols", &mut w);
        let (max_rows, max_cols) = (l.max_cart_rows, l.max_cart_cols);
        let c = &mut self.cartwall;
        clamp_to(
            &mut c.default_rows,
            1,
            max_rows,
            "cartwall.default_rows",
            &mut w,
        );
        clamp_to(
            &mut c.default_cols,
            1,
            max_cols,
            "cartwall.default_cols",
            &mut w,
        );

        let max_players = self.limits.max_players;
        let p = &mut self.players;
        clamp_to(&mut p.count, 1, max_players, "players.count", &mut w);
        clamp_to(&mut p.fade_ms, 50, 10_000, "players.fade_ms", &mut w);
        clamp_to(
            &mut p.end_warning_secs,
            0.0,
            120.0,
            "players.end_warning_secs",
            &mut w,
        );

        let a = &mut self.analysis;
        clamp_to(
            &mut a.silence_threshold_db,
            -96.0,
            -10.0,
            "analysis.silence_threshold_db",
            &mut w,
        );
        clamp_to(
            &mut a.segue_threshold_db,
            -60.0,
            0.0,
            "analysis.segue_threshold_db",
            &mut w,
        );
        clamp_to(
            &mut a.segue_max_secs,
            0.0,
            60.0,
            "analysis.segue_max_secs",
            &mut w,
        );
        clamp_to(
            &mut a.outro_drop_db,
            0.0,
            40.0,
            "analysis.outro_drop_db",
            &mut w,
        );
        clamp_to(
            &mut a.outro_max_secs,
            0.0,
            300.0,
            "analysis.outro_max_secs",
            &mut w,
        );
        clamp_to(
            &mut a.markers_min_duration_secs,
            0.0,
            3600.0,
            "analysis.markers_min_duration_secs",
            &mut w,
        );
        clamp_to(
            &mut a.peak_bucket_ms,
            1,
            1000,
            "analysis.peak_bucket_ms",
            &mut w,
        );
        clamp_to(
            &mut a.rms_window_ms,
            5,
            1000,
            "analysis.rms_window_ms",
            &mut w,
        );
        clamp_to(
            &mut a.cover_thumb_px,
            16,
            1024,
            "analysis.cover_thumb_px",
            &mut w,
        );

        let o = &mut self.outputs;
        clamp_to(
            &mut o.sample_rate,
            8_000,
            768_000,
            "outputs.sample_rate",
            &mut w,
        );
        clamp_to(
            &mut o.buffer_frames,
            16,
            16_384,
            "outputs.buffer_frames",
            &mut w,
        );

        let t = &mut self.tuning;
        clamp_to(&mut t.declick_ms, 0.5, 50.0, "tuning.declick_ms", &mut w);
        clamp_to(
            &mut t.pause_ramp_ms,
            0.5,
            200.0,
            "tuning.pause_ramp_ms",
            &mut w,
        );
        clamp_to(
            &mut t.prebuffer_secs,
            1.0,
            60.0,
            "tuning.prebuffer_secs",
            &mut w,
        );
        clamp_to(
            &mut t.ready_threshold_ms,
            50.0,
            10_000.0,
            "tuning.ready_threshold_ms",
            &mut w,
        );
        clamp_to(
            &mut t.mixer_headroom,
            1.0,
            8.0,
            "tuning.mixer_headroom",
            &mut w,
        );
        clamp_to(
            &mut t.max_commands_per_block,
            16,
            4096,
            "tuning.max_commands_per_block",
            &mut w,
        );
        clamp_to(
            &mut t.schedule_lead_ms,
            20.0,
            5000.0,
            "tuning.schedule_lead_ms",
            &mut w,
        );
        clamp_to(
            &mut t.conductor_tick_ms,
            1.0,
            50.0,
            "tuning.conductor_tick_ms",
            &mut w,
        );
        clamp_to(
            &mut t.watchdog_timeout_ms,
            100.0,
            10_000.0,
            "tuning.watchdog_timeout_ms",
            &mut w,
        );
        clamp_to(
            &mut t.watchdog_startup_grace_ms,
            100.0,
            60_000.0,
            "tuning.watchdog_startup_grace_ms",
            &mut w,
        );
        clamp_to(
            &mut t.reconnect_interval_ms,
            250.0,
            60_000.0,
            "tuning.reconnect_interval_ms",
            &mut w,
        );
        clamp_to(
            &mut t.gain_smoothing_ms,
            1.0,
            500.0,
            "tuning.gain_smoothing_ms",
            &mut w,
        );
        clamp_to(
            &mut t.save_debounce_ms,
            100.0,
            60_000.0,
            "tuning.save_debounce_ms",
            &mut w,
        );

        // One chord per action and one action per chord; the first wins.
        let mut chords = std::collections::HashSet::new();
        let mut actions = std::collections::HashSet::new();
        self.shortcuts.retain(|s| {
            if s.action.position() == Some(0) {
                w.push(ConfigWarning {
                    field: "shortcuts",
                    message: format!(
                        "{} for {:?} ignored: positions start at 1",
                        s.chord, s.action
                    ),
                });
                return false;
            }
            let fresh = !chords.contains(&s.chord) && !actions.contains(&s.action);
            if fresh {
                chords.insert(s.chord.clone());
                actions.insert(s.action);
            } else {
                w.push(ConfigWarning {
                    field: "shortcuts",
                    message: format!("{} for {:?} ignored: already bound", s.chord, s.action),
                });
            }
            fresh
        });

        w
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_valid() {
        let mut c = Config::default();
        assert!(c.validate().is_empty());
        assert_eq!(c.players.count, 4);
        assert_eq!(c.players.fade_ms, 1000);
        assert_eq!(c.analysis.segue_threshold_db, -18.0);
        assert_eq!(c.analysis.segue_max_secs, 8.0);
        assert_eq!(c.limits.max_players, 16);
    }

    #[test]
    fn missing_fields_take_defaults() {
        let c: Config = serde_json::from_str(r#"{"players":{"fade_ms":2000}}"#).unwrap();
        assert_eq!(c.players.fade_ms, 2000);
        assert_eq!(c.players.count, 4);
        assert_eq!(c.tuning, Tuning::default());
    }

    #[test]
    fn nan_and_out_of_range_values_are_clamped() {
        let mut c = Config::default();
        c.players.count = 0;
        c.analysis.segue_threshold_db = f64::NAN;
        c.tuning.prebuffer_secs = 1e9;
        let warnings = c.validate();
        assert_eq!(c.players.count, 1);
        assert_eq!(c.analysis.segue_threshold_db, -60.0);
        assert_eq!(c.tuning.prebuffer_secs, 60.0);
        let fields: Vec<_> = warnings.iter().map(|w| w.field).collect();
        assert!(fields.contains(&"players.count"));
        assert!(fields.contains(&"analysis.segue_threshold_db"));
        assert!(fields.contains(&"tuning.prebuffer_secs"));
        assert!(warnings[0].to_string().contains("using"));
    }

    #[test]
    fn player_count_is_capped_by_the_resource_limit() {
        let mut c = Config::default();
        c.limits.max_players = 8;
        c.players.count = 12;
        c.validate();
        assert_eq!(c.players.count, 8);
    }
}
