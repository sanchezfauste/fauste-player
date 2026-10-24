#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Loudness normalization spec §4: the `loudness` configuration section.

use crate::common;

use common::{fixture, p0};
use fp_model::{Command, Config, EngineAction, LoudnessConfig, apply, pending};

#[test]
fn the_defaults_are_those_of_the_spec() {
    let c = LoudnessConfig::default();
    assert!(c.normalize, "D11: on by default");
    assert_eq!(
        (
            c.target_lufs,
            c.true_peak_ceiling_dbtp,
            c.max_boost_db,
            c.max_trim_db
        ),
        (-16.0, -1.0, 12.0, 12.0)
    );
    assert_eq!(Config::default().loudness, c);
}

/// Sets one field of a default configuration, validates it and returns
/// the warnings and the resulting section.
fn validated(set: impl Fn(&mut LoudnessConfig)) -> (Vec<&'static str>, LoudnessConfig) {
    let mut config = Config::default();
    set(&mut config.loudness);
    let warnings = config.validate().into_iter().map(|w| w.field).collect();
    (warnings, config.loudness)
}

#[test]
fn n1_each_value_is_clamped_into_its_range_with_a_warning() {
    let (w, l) = validated(|c| c.target_lufs = -50.0);
    assert!(w.contains(&"loudness.target_lufs"), "{w:?}");
    assert_eq!(l.target_lufs, -36.0);
    let (w, l) = validated(|c| c.target_lufs = 0.0);
    assert!(w.contains(&"loudness.target_lufs"), "{w:?}");
    assert_eq!(l.target_lufs, -6.0);

    let (w, l) = validated(|c| c.true_peak_ceiling_dbtp = 3.0);
    assert!(w.contains(&"loudness.true_peak_ceiling_dbtp"), "{w:?}");
    assert_eq!(l.true_peak_ceiling_dbtp, 0.0);
    let (w, l) = validated(|c| c.true_peak_ceiling_dbtp = -20.0);
    assert!(w.contains(&"loudness.true_peak_ceiling_dbtp"), "{w:?}");
    assert_eq!(l.true_peak_ceiling_dbtp, -12.0);

    let (w, l) = validated(|c| c.max_boost_db = 30.0);
    assert!(w.contains(&"loudness.max_boost_db"), "{w:?}");
    assert_eq!(l.max_boost_db, 24.0);
    let (w, l) = validated(|c| c.max_boost_db = -1.0);
    assert!(w.contains(&"loudness.max_boost_db"), "{w:?}");
    assert_eq!(l.max_boost_db, 0.0);

    let (w, l) = validated(|c| c.max_trim_db = 99.0);
    assert!(w.contains(&"loudness.max_trim_db"), "{w:?}");
    assert_eq!(l.max_trim_db, 24.0);
    let (w, l) = validated(|c| c.max_trim_db = -3.0);
    assert!(w.contains(&"loudness.max_trim_db"), "{w:?}");
    assert_eq!(l.max_trim_db, 0.0);
}

#[test]
fn n1_a_value_that_is_not_a_number_takes_the_default() {
    let mut config = Config::default();
    config.loudness.target_lufs = f32::NAN;
    config.loudness.true_peak_ceiling_dbtp = f32::INFINITY;
    config.loudness.max_boost_db = f32::NEG_INFINITY;
    config.loudness.max_trim_db = f32::NAN;
    let warnings = config.validate();
    assert_eq!(config.loudness, LoudnessConfig::default());
    assert_eq!(
        warnings
            .iter()
            .filter(|w| w.field.starts_with("loudness."))
            .count(),
        4
    );
}

#[test]
fn n1_values_in_range_give_no_warning() {
    let mut config = Config {
        loudness: LoudnessConfig {
            normalize: true,
            target_lufs: -23.0,
            true_peak_ceiling_dbtp: -2.0,
            max_boost_db: 6.0,
            max_trim_db: 3.0,
        },
        ..Config::default()
    };
    let before = config.loudness.clone();
    assert!(
        config
            .validate()
            .iter()
            .all(|w| !w.field.starts_with("loudness."))
    );
    assert_eq!(config.loudness, before);
}

#[test]
fn n1_a_file_without_the_section_loads_the_defaults() {
    let config: Config = serde_json::from_str(r#"{"players":{"count":2}}"#).unwrap();
    assert_eq!(config.loudness, LoudnessConfig::default());
    let partial: Config = serde_json::from_str(r#"{"loudness":{"normalize":true}}"#).unwrap();
    assert!(partial.loudness.normalize);
    assert_eq!(partial.loudness.target_lufs, -16.0);
}

#[test]
fn n3_a_loudness_change_only_preloads_and_never_waits() {
    let mut state = fixture(3);
    // Start from off so that turning it on is part of the change.
    state.config.loudness.normalize = false;
    let p = p0(&state);
    apply(&mut state, Command::Play(p)).unwrap();
    let mut config = state.config.clone();
    config.loudness.normalize = true;
    config.loudness.target_lufs = -20.0;
    let actions = apply(&mut state, Command::UpdateConfig(Box::new(config))).unwrap();
    assert!(
        actions
            .iter()
            .all(|a| matches!(a, EngineAction::Preload { .. })),
        "{actions:?}"
    );
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EngineAction::Preload { .. })),
        "the change reaches the next track: {actions:?}"
    );
    assert!(pending(&state).is_empty());
    assert!(state.config.loudness.normalize, "applied at once");
}

#[test]
fn a_saved_false_stays_off_and_a_missing_field_takes_the_default() {
    let off: Config = serde_json::from_str(r#"{"loudness":{"normalize":false}}"#).unwrap();
    assert!(
        !off.loudness.normalize,
        "an operator who chose off stays off"
    );
    let missing: Config = serde_json::from_str(r#"{"loudness":{"target_lufs":-18.0}}"#).unwrap();
    assert!(missing.loudness.normalize);
    let none: Config = serde_json::from_str("{}").unwrap();
    assert!(none.loudness.normalize);
}
