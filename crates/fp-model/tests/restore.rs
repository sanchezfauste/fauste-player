#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback 2 spec O2: "Restore defaults" resets one Settings section.

mod common;

use common::fixture;
use fp_model::{
    AnalysisSettings, Command, Config, MeterBallistics, MeterConfig, PlayersConfig,
    SettingsSection, apply, default_shortcuts, restore_defaults,
};

/// A configuration with a value changed in every part.
fn altered() -> Config {
    let mut c = Config::default();
    c.players.count = 6;
    c.players.fade_ms = 3000;
    c.players.auto_segue = false;
    c.players.history_len = 10;
    c.ui.language = Some("es-ES".into());
    c.ui.wave_color = "sand".into();
    c.ui.follow_current_grace_secs = 30.0;
    c.meter.ballistics = MeterBallistics::Vu;
    c.meter.floor_db = -40.0;
    c.analysis.segue_drop_db = 20.0;
    c.analysis.cover_thumb_px = 256;
    c.cartwall.default_rows = 4;
    c.cartwall.default_cols = 4;
    c.shortcuts.clear();
    c.outputs.backend = Some("alsa".into());
    c.outputs.sample_rate = 44_100;
    c.limits.max_players = 8;
    c.tuning.prebuffer_secs = 10.0;
    c.midi.enabled = true;
    c.remote.http.enabled = true;
    c
}

#[test]
fn players_resets_the_player_settings_but_not_the_count_or_the_language() {
    let mut c = altered();
    restore_defaults(&mut c, SettingsSection::Players);
    let mut expected = altered();
    expected.players = PlayersConfig {
        count: 6,
        ..PlayersConfig::default()
    };
    assert_eq!(c, expected);
}

#[test]
fn meters_resets_only_the_meter() {
    let mut c = altered();
    restore_defaults(&mut c, SettingsSection::Meters);
    let mut expected = altered();
    expected.meter = MeterConfig::default();
    assert_eq!(c, expected);
}

#[test]
fn analysis_resets_only_the_analysis() {
    let mut c = altered();
    restore_defaults(&mut c, SettingsSection::Analysis);
    let mut expected = altered();
    expected.analysis = AnalysisSettings::default();
    assert_eq!(c, expected);
}

#[test]
fn shortcuts_resets_only_the_shortcuts() {
    let mut c = altered();
    restore_defaults(&mut c, SettingsSection::Shortcuts);
    let mut expected = altered();
    expected.shortcuts = default_shortcuts();
    assert_eq!(c, expected);
}

#[test]
fn the_command_keeps_the_player_count() {
    let mut s = fixture(1);
    apply(&mut s, Command::SetPlayerCount(2)).unwrap();
    let mut config = s.config.clone();
    config.players.fade_ms = 3000;
    config.ui.language = Some("es-ES".into());
    apply(&mut s, Command::UpdateConfig(Box::new(config))).unwrap();

    apply(&mut s, Command::RestoreDefaults(SettingsSection::Players)).unwrap();
    assert_eq!(s.config.players.fade_ms, 1000);
    assert_eq!(s.config.ui.language.as_deref(), Some("es-ES"));
    assert_eq!(s.players.len(), 2);
    assert_eq!(s.config.players.count, 2);
}
