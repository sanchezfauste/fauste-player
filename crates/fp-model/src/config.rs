//! Application configuration. Every tunable value lives here with a
//! documented default; nothing product-related is a hardcoded constant.

use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::columns::{TableColumn, default_columns, normalize_columns};
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
    /// Level meters (meters spec M3).
    pub meter: MeterConfig,
    /// MIDI control surfaces (feedback spec §6).
    pub midi: crate::midi::MidiConfig,
    /// Remote control over the network (remote control spec).
    pub remote: crate::remote::RemoteConfig,
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
            meter: MeterConfig::default(),
            midi: crate::midi::MidiConfig::default(),
            remote: crate::remote::RemoteConfig::default(),
        }
    }
}

/// How the meter bar moves (meters spec M2): the standard meter types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MeterBallistics {
    /// Digital peak meter, IEC 60268-18: instant rise, 20 dB fall in 1.7 s.
    #[default]
    DigitalPeak,
    /// Quasi-peak programme meter, IEC 60268-10 type IIb (EBU): 10 ms
    /// integration, 24 dB fall in 2.8 s.
    EbuPpm,
    /// Quasi-peak programme meter, IEC 60268-10 type I (DIN 45406): 5 ms
    /// integration, 20 dB fall in 1.5 s.
    DinPpm,
    /// Volume unit meter, IEC 60268-17: RMS, 300 ms rise and fall.
    Vu,
    /// K-System meters: a peak and an RMS average section on a scale
    /// whose 0 is 20, 14 or 12 dB below full scale.
    K20,
    K14,
    K12,
    /// `attack_ms` and `release_db_per_sec`.
    Custom,
}

/// Which `config.meter` fields a meter type uses (meters spec M3). The
/// others are kept, for when the operator switches back, but neither
/// shown nor applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeterSettings {
    /// `attack_ms` and `release_db_per_sec`.
    pub custom_ballistics: bool,
    /// `floor_db`: the other scales have the range their standard gives.
    pub floor: bool,
    /// `peak_hold_secs`: programme meters and the VU have no hold.
    pub peak_hold: bool,
    /// `reference_dbfs`: a K meter's alignment is its own 0.
    pub alignment: bool,
    /// `warning_dbfs` and `danger_dbfs`: the other scales have their own
    /// red region.
    pub zones: bool,
    /// `true_peak`: only meters that read peaks.
    pub true_peak: bool,
}

impl MeterBallistics {
    /// The settings this type uses (meters spec M3).
    pub fn settings(self) -> MeterSettings {
        let digital = MeterSettings {
            custom_ballistics: false,
            floor: true,
            peak_hold: true,
            alignment: true,
            zones: true,
            true_peak: true,
        };
        match self {
            MeterBallistics::DigitalPeak => digital,
            MeterBallistics::Custom => MeterSettings {
                custom_ballistics: true,
                ..digital
            },
            MeterBallistics::EbuPpm | MeterBallistics::DinPpm | MeterBallistics::Vu => {
                MeterSettings {
                    custom_ballistics: false,
                    floor: false,
                    peak_hold: false,
                    alignment: true,
                    zones: false,
                    true_peak: false,
                }
            }
            MeterBallistics::K20 | MeterBallistics::K14 | MeterBallistics::K12 => MeterSettings {
                custom_ballistics: false,
                floor: false,
                peak_hold: true,
                alignment: false,
                zones: false,
                true_peak: true,
            },
        }
    }

    /// The level of a K-System meter's 0 in dBFS; `None` for other meters.
    pub fn k_reference_dbfs(self) -> Option<f32> {
        match self {
            MeterBallistics::K20 => Some(-20.0),
            MeterBallistics::K14 => Some(-14.0),
            MeterBallistics::K12 => Some(-12.0),
            _ => None,
        }
    }
}

/// The loudness line under the meter (EBU R128).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LoudnessReadout {
    Off,
    /// Over the last 400 ms.
    Momentary,
    /// Over the last 3 s.
    #[default]
    ShortTerm,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MeterConfig {
    pub ballistics: MeterBallistics,
    /// Rise integration time for `Custom`, in ms (0 = instant).
    pub attack_ms: f32,
    /// Fall rate for `Custom`, in dB per second.
    pub release_db_per_sec: f32,
    /// Measure the peak of the 4× oversampled signal (ITU-R BS.1770).
    pub true_peak: bool,
    /// Bottom of the scale, in dBFS.
    pub floor_db: f32,
    /// How long the highest level stays lit; 0 turns the hold off.
    pub peak_hold_secs: f32,
    /// Alignment level mark (EBU R68: −18 dBFS).
    pub reference_dbfs: f32,
    /// Yellow from this level (EBU permitted maximum: −9 dBFS).
    pub warning_dbfs: f32,
    /// Red from this level.
    pub danger_dbfs: f32,
    pub loudness: LoudnessReadout,
    /// The readout is green within ±1 LU of this (EBU R128: −23 LUFS).
    pub loudness_target_lufs: f32,
}

impl MeterConfig {
    /// True peak is measured only when it is on and the type reads peaks.
    pub fn true_peak_in_use(&self) -> bool {
        self.true_peak && self.ballistics.settings().true_peak
    }

    /// The peak hold is shown only when it is on and the type has one.
    pub fn peak_hold_in_use(&self) -> bool {
        self.peak_hold_secs > 0.0 && self.ballistics.settings().peak_hold
    }
}

impl Default for MeterConfig {
    fn default() -> Self {
        Self {
            ballistics: MeterBallistics::DigitalPeak,
            attack_ms: 5.0,
            release_db_per_sec: 11.8,
            true_peak: false,
            floor_db: -60.0,
            peak_hold_secs: 2.0,
            reference_dbfs: -18.0,
            warning_dbfs: -9.0,
            danger_dbfs: -3.0,
            loudness: LoudnessReadout::ShortTerm,
            loudness_target_lufs: -23.0,
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
    /// Players honour each track's cue-in and cue-out. Off, every player
    /// entry plays from 0 to the end of the file; the markers are kept.
    /// Carts always use theirs. Independent of `auto_segue`.
    pub use_cue_markers: bool,
    pub end_warning_secs: f64,
    /// How many entries Previous can go back (R25); 0 disables Previous.
    pub history_len: usize,
}

impl Default for PlayersConfig {
    fn default() -> Self {
        Self {
            count: 4,
            default_mode: PlayMode::Continuous,
            fade_ms: 1000,
            auto_segue: true,
            use_cue_markers: true,
            end_warning_secs: 10.0,
            history_len: 50,
        }
    }
}

/// Parameters of automatic marker detection (spec §6).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AnalysisSettings {
    /// Buckets whose peak reaches this level (dBFS) are never trimmed.
    pub trim_threshold_db: f64,
    /// Kept around the first and last audible buckets.
    pub trim_margin_ms: u32,
    /// The segue starts where the level has fallen this far (dB) below the
    /// median RMS of the track's body.
    pub segue_drop_db: f64,
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
            trim_threshold_db: -60.0,
            trim_margin_ms: 20,
            segue_drop_db: 15.0,
            segue_max_secs: 4.0,
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
    /// After the operator moves a view (a zoomed waveform, a scrolled
    /// playlist), it stops following what plays for this long (seconds);
    /// 0 never follows.
    pub follow_current_grace_secs: f64,
    /// The columns of the track tables, in order; one list for every player
    /// and playlist (feedback 2 spec O24). Title and Duration are required.
    pub table_columns: Vec<TableColumn>,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            wave_color: "slate".to_owned(),
            music_dir: None,
            language: None,
            follow_current_grace_secs: 10.0,
            table_columns: default_columns(),
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
    /// Longest tag text kept per field, in characters (tags are trimmed and cut).
    pub max_tag_chars: usize,
    /// Most values kept per tag field (two artists are two values), so a
    /// hostile file cannot fill the tag editor with thousands of lines.
    pub max_tag_values: usize,
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
            max_tag_chars: 2000,
            max_tag_values: 32,
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
    /// How often files not found (a drive not mounted yet) are looked for
    /// again.
    pub missing_recheck_ms: f64,
    /// Inside a Flatpak, how long Restart now waits for the new instance to
    /// take the instance lock before this one ends (its sandbox, and the
    /// launcher in it, end with it).
    pub restart_handoff_ms: f64,
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
            missing_recheck_ms: 30_000.0,
            restart_handoff_ms: 5_000.0,
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
pub(crate) fn clamp_to<T: PartialOrd + Copy + fmt::Display>(
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
            &mut l.max_tag_chars,
            64,
            100_000,
            "limits.max_tag_chars",
            &mut w,
        );
        clamp_to(
            &mut l.max_tag_values,
            1,
            1000,
            "limits.max_tag_values",
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
        clamp_to(&mut p.history_len, 0, 1000, "players.history_len", &mut w);

        clamp_to(
            &mut self.ui.follow_current_grace_secs,
            0.0,
            600.0,
            "ui.follow_current_grace_secs",
            &mut w,
        );
        let columns = normalize_columns(&self.ui.table_columns);
        if columns != self.ui.table_columns {
            w.push(ConfigWarning {
                field: "ui.table_columns",
                message: format!("{:?} repaired to {columns:?}", self.ui.table_columns),
            });
            self.ui.table_columns = columns;
        }

        let a = &mut self.analysis;
        clamp_to(
            &mut a.trim_threshold_db,
            -120.0,
            -20.0,
            "analysis.trim_threshold_db",
            &mut w,
        );
        clamp_to(
            &mut a.trim_margin_ms,
            0,
            1000,
            "analysis.trim_margin_ms",
            &mut w,
        );
        clamp_to(
            &mut a.segue_drop_db,
            3.0,
            40.0,
            "analysis.segue_drop_db",
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
        clamp_to(
            &mut t.missing_recheck_ms,
            1_000.0,
            3_600_000.0,
            "tuning.missing_recheck_ms",
            &mut w,
        );
        clamp_to(
            &mut t.restart_handoff_ms,
            500.0,
            60_000.0,
            "tuning.restart_handoff_ms",
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

        let m = &mut self.meter;
        clamp_to(&mut m.attack_ms, 0.0, 1000.0, "meter.attack_ms", &mut w);
        clamp_to(
            &mut m.release_db_per_sec,
            1.0,
            100.0,
            "meter.release_db_per_sec",
            &mut w,
        );
        clamp_to(&mut m.floor_db, -96.0, -20.0, "meter.floor_db", &mut w);
        clamp_to(
            &mut m.peak_hold_secs,
            0.0,
            10.0,
            "meter.peak_hold_secs",
            &mut w,
        );
        clamp_to(
            &mut m.reference_dbfs,
            -30.0,
            0.0,
            "meter.reference_dbfs",
            &mut w,
        );
        clamp_to(
            &mut m.warning_dbfs,
            -30.0,
            0.0,
            "meter.warning_dbfs",
            &mut w,
        );
        clamp_to(&mut m.danger_dbfs, -30.0, 0.0, "meter.danger_dbfs", &mut w);
        clamp_to(
            &mut m.loudness_target_lufs,
            -36.0,
            -10.0,
            "meter.loudness_target_lufs",
            &mut w,
        );
        if m.reference_dbfs <= m.floor_db {
            m.reference_dbfs = m.floor_db + 1.0;
            w.push(ConfigWarning {
                field: "meter.reference_dbfs",
                message: "raised above the scale floor".to_owned(),
            });
        }
        if m.warning_dbfs > m.danger_dbfs {
            m.danger_dbfs = m.warning_dbfs;
            w.push(ConfigWarning {
                field: "meter.danger_dbfs",
                message: "raised to the warning level".to_owned(),
            });
        }

        clamp_to(
            &mut self.midi.rescan_interval_ms,
            250,
            60_000,
            "midi.rescan_interval_ms",
            &mut w,
        );
        self.midi.validate(&mut w);
        self.remote.validate(&mut w);

        w
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_follow_grace_is_validated() {
        let mut c = Config::default();
        assert_eq!(c.ui.follow_current_grace_secs, 10.0);
        c.ui.follow_current_grace_secs = 9999.0;
        c.validate();
        assert_eq!(c.ui.follow_current_grace_secs, 600.0);
    }

    #[test]
    fn the_history_length_is_validated() {
        let mut c = Config::default();
        assert_eq!(c.players.history_len, 50);
        c.players.history_len = 5000;
        assert!(!c.validate().is_empty());
        assert_eq!(c.players.history_len, 1000);
    }

    #[test]
    fn trim_and_segue_settings_have_their_defaults_and_ranges() {
        let a = AnalysisSettings::default();
        assert_eq!(a.trim_threshold_db, -60.0);
        assert_eq!(a.trim_margin_ms, 20);
        assert_eq!(a.segue_drop_db, 15.0);
        assert_eq!(a.segue_max_secs, 4.0);
        let mut c = Config::default();
        c.analysis.trim_threshold_db = -500.0;
        c.analysis.trim_margin_ms = 50_000;
        c.analysis.segue_drop_db = 99.0;
        c.validate();
        assert_eq!(c.analysis.trim_threshold_db, -120.0);
        assert_eq!(c.analysis.trim_margin_ms, 1000);
        assert_eq!(c.analysis.segue_drop_db, 40.0);
    }

    #[test]
    fn the_default_waveform_colour_is_slate_and_a_stored_one_is_kept() {
        assert_eq!(Config::default().ui.wave_color, "slate");
        let c: Config = serde_json::from_str(r#"{"ui":{"wave_color":"sand"}}"#).unwrap();
        assert_eq!(c.ui.wave_color, "sand");
        let c: Config = serde_json::from_str(r#"{"ui":{}}"#).unwrap();
        assert_eq!(c.ui.wave_color, "slate");
    }

    #[test]
    fn defaults_are_valid() {
        let mut c = Config::default();
        assert!(c.validate().is_empty());
        assert_eq!(c.players.count, 4);
        assert_eq!(c.players.fade_ms, 1000);
        assert_eq!(c.analysis.segue_drop_db, 15.0);
        assert_eq!(c.analysis.segue_max_secs, 4.0);
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
        c.analysis.segue_drop_db = f64::NAN;
        c.tuning.prebuffer_secs = 1e9;
        let warnings = c.validate();
        assert_eq!(c.players.count, 1);
        assert_eq!(c.analysis.segue_drop_db, 3.0);
        assert_eq!(c.tuning.prebuffer_secs, 60.0);
        let fields: Vec<_> = warnings.iter().map(|w| w.field).collect();
        assert!(fields.contains(&"players.count"));
        assert!(fields.contains(&"analysis.segue_drop_db"));
        assert!(fields.contains(&"tuning.prebuffer_secs"));
        assert!(warnings[0].to_string().contains("using"));
    }

    #[test]
    fn the_missing_file_recheck_has_its_default_and_range() {
        assert_eq!(Tuning::default().missing_recheck_ms, 30_000.0);
        let c: Config = serde_json::from_str(r#"{"tuning":{}}"#).unwrap();
        assert_eq!(c.tuning.missing_recheck_ms, 30_000.0);
        for (set, kept) in [(10.0, 1_000.0), (1e7, 3_600_000.0)] {
            let mut c = Config::default();
            c.tuning.missing_recheck_ms = set;
            let warnings = c.validate();
            assert_eq!(c.tuning.missing_recheck_ms, kept);
            assert!(
                warnings
                    .iter()
                    .any(|w| w.field == "tuning.missing_recheck_ms")
            );
        }
    }

    #[test]
    fn the_restart_handoff_has_its_default_and_range() {
        assert_eq!(Tuning::default().restart_handoff_ms, 5_000.0);
        let c: Config = serde_json::from_str(r#"{"tuning":{}}"#).unwrap();
        assert_eq!(c.tuning.restart_handoff_ms, 5_000.0);
        for (set, kept) in [(10.0, 500.0), (1e7, 60_000.0)] {
            let mut c = Config::default();
            c.tuning.restart_handoff_ms = set;
            let warnings = c.validate();
            assert_eq!(c.tuning.restart_handoff_ms, kept);
            assert!(
                warnings
                    .iter()
                    .any(|w| w.field == "tuning.restart_handoff_ms")
            );
        }
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
