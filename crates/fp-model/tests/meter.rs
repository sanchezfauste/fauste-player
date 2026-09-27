#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Level meter settings (meters spec M3).

use fp_model::{Config, LoudnessReadout, MeterBallistics};

#[test]
fn meter_defaults_follow_the_standards() {
    let m = Config::default().meter;
    assert_eq!(m.ballistics, MeterBallistics::DigitalPeak);
    assert!(!m.true_peak);
    assert_eq!(m.floor_db, -60.0);
    assert_eq!(m.peak_hold_secs, 2.0);
    assert_eq!(m.reference_dbfs, -18.0, "EBU R68 alignment");
    assert_eq!(m.warning_dbfs, -9.0, "EBU permitted maximum");
    assert_eq!(m.danger_dbfs, -3.0);
    assert_eq!(m.loudness, LoudnessReadout::ShortTerm);
    assert_eq!(m.loudness_target_lufs, -23.0, "EBU R128");
    assert!(Config::default().validate().is_empty());
}

#[test]
fn meter_zones_are_kept_in_order() {
    let mut c = Config::default();
    c.meter.warning_dbfs = -2.0;
    c.meter.danger_dbfs = -6.0;
    c.meter.floor_db = -10.0;
    c.meter.reference_dbfs = -18.0;
    c.meter.release_db_per_sec = 0.0;
    let warnings = c.validate();
    assert!(c.meter.warning_dbfs <= c.meter.danger_dbfs);
    assert!(c.meter.floor_db < c.meter.reference_dbfs);
    assert_eq!(c.meter.floor_db, -20.0, "clamped to its range");
    assert!(c.meter.release_db_per_sec >= 1.0);
    assert!(warnings.iter().any(|w| w.field.starts_with("meter.")));
}

#[test]
fn a_config_without_meter_loads_defaults() {
    let c: Config = serde_json::from_str(r#"{ "players": { "count": 2 } }"#).unwrap();
    assert_eq!(c.meter, Config::default().meter);
}
