#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Automation spec §11: the `automation` configuration section.

use fp_model::{AutomationConfig, Config};

#[test]
fn automation_defaults_match_the_spec() {
    let c = AutomationConfig::default();
    assert_eq!(
        (
            c.track_separation_mins,
            c.track_separation_percent,
            c.artist_separation_mins
        ),
        (180, 50, 30)
    );
    assert_eq!(Config::default().automation, c);
}

#[test]
fn out_of_range_automation_values_are_clamped_with_a_warning() {
    let mut config = Config::default();
    config.automation.track_separation_mins = 20_000;
    config.automation.track_separation_percent = 99;
    config.automation.artist_separation_mins = 5_000;
    let warnings = config.validate();
    assert_eq!(config.automation.track_separation_mins, 10_080);
    assert_eq!(config.automation.track_separation_percent, 95);
    assert_eq!(config.automation.artist_separation_mins, 1_440);
    let fields: Vec<&str> = warnings.iter().map(|w| w.field).collect();
    for field in [
        "automation.track_separation_mins",
        "automation.track_separation_percent",
        "automation.artist_separation_mins",
    ] {
        assert!(fields.contains(&field), "{field} in {fields:?}");
    }
}

#[test]
fn a_config_without_automation_loads_the_defaults() {
    let config: Config = serde_json::from_str("{}").unwrap();
    assert_eq!(config.automation, AutomationConfig::default());
    let partial: Config =
        serde_json::from_str(r#"{"automation":{"artist_separation_mins":10}}"#).unwrap();
    assert_eq!(partial.automation.artist_separation_mins, 10);
    assert_eq!(partial.automation.track_separation_mins, 180);
}
