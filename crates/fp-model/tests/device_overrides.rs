#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Operator feedback 4, Q12: each device's own rate and buffer, and the
//! Basic | Advanced view of Settings → Audio outputs.

use fp_model::{
    Config, DeviceOverride, DsdMix, OutputDevice, OutputsView, PlayerId, PlayerRoutes, Route,
};

fn dev(name: &str) -> OutputDevice {
    OutputDevice {
        backend: "alsa".into(),
        device: name.into(),
    }
}

fn own(name: &str, rate: Option<u32>, buffer: Option<u32>) -> DeviceOverride {
    DeviceOverride {
        device: dev(name),
        sample_rate: rate,
        buffer_frames: buffer,
    }
}

fn route(name: &str, first_channel: u16) -> Route {
    Route {
        backend: "alsa".into(),
        device: name.into(),
        first_channel,
    }
}

/// Player 1 plays on `dac` and pre-listens on `phones`.
fn routed() -> Config {
    let mut c = Config::default();
    c.outputs.routes = vec![PlayerRoutes {
        player: PlayerId(1),
        main: Some(route("dac", 0)),
        cue: Some(route("phones", 0)),
    }];
    c
}

#[test]
fn the_outputs_view_is_basic_by_default_and_is_kept() {
    assert_eq!(Config::default().ui.outputs_view, OutputsView::Basic);
    let c: Config = serde_json::from_str(r#"{"ui":{}}"#).unwrap();
    assert_eq!(c.ui.outputs_view, OutputsView::Basic);
    let c: Config = serde_json::from_str(r#"{"ui":{"outputs_view":"Advanced"}}"#).unwrap();
    assert_eq!(c.ui.outputs_view, OutputsView::Advanced);
}

#[test]
fn by_default_no_device_has_its_own_values() {
    let mut c = Config::default();
    assert!(c.outputs.device_overrides.is_empty());
    assert_eq!(c.outputs.rate_for("alsa", "dac"), 48_000);
    assert_eq!(c.outputs.buffer_for("alsa", "dac"), 512);
    assert!(c.validate().is_empty());
    let c: Config = serde_json::from_str(r#"{"outputs":{"sample_rate":44100}}"#).unwrap();
    assert!(
        c.outputs.device_overrides.is_empty(),
        "an older file has none"
    );
}

#[test]
fn a_device_uses_its_own_rate_and_buffer_and_the_others_the_global_ones() {
    let mut c = Config::default();
    c.outputs.device_overrides = vec![
        own("dac", Some(96_000), None),
        own("phones", None, Some(256)),
    ];
    assert_eq!(c.outputs.rate_for("alsa", "dac"), 96_000);
    assert_eq!(c.outputs.buffer_for("alsa", "dac"), 512);
    assert_eq!(c.outputs.rate_for("alsa", "phones"), 48_000);
    assert_eq!(c.outputs.buffer_for("alsa", "phones"), 256);
    assert_eq!(
        c.outputs.rate_for("jack", "dac"),
        48_000,
        "keyed by backend too"
    );
}

#[test]
fn setting_and_clearing_a_devices_own_values() {
    let mut c = Config::default();
    c.outputs.set_device_rate(&dev("dac"), Some(96_000));
    c.outputs.set_device_buffer(&dev("dac"), Some(256));
    assert_eq!(
        c.outputs.device_overrides,
        vec![own("dac", Some(96_000), Some(256))]
    );
    c.outputs.set_device_rate(&dev("dac"), None);
    assert_eq!(
        c.outputs.device_overrides,
        vec![own("dac", None, Some(256))]
    );
    c.outputs.set_device_buffer(&dev("dac"), None);
    assert!(
        c.outputs.device_overrides.is_empty(),
        "an entry with nothing left is removed"
    );
    assert!(c.outputs.device_override(&dev("dac")).is_none());
}

#[test]
fn a_value_equal_to_the_global_one_stays_the_devices_own() {
    let mut c = Config::default();
    c.outputs.set_device_rate(&dev("dac"), Some(48_000));
    c.outputs.sample_rate = 44_100;
    assert_eq!(c.outputs.rate_for("alsa", "dac"), 48_000);
    assert_eq!(c.outputs.rate_for("alsa", "phones"), 44_100);
}

#[test]
fn out_of_range_values_are_dropped_so_the_global_value_applies() {
    let mut c = Config::default();
    c.outputs.device_overrides = vec![
        own("dac", Some(4_000), Some(1024)),
        own("phones", Some(96_000), Some(100_000)),
        own("spdif", Some(1_000_000), None),
    ];
    let warnings = c.validate();
    assert_eq!(
        c.outputs.device_overrides,
        vec![
            own("dac", None, Some(1024)),
            own("phones", Some(96_000), None)
        ],
        "spdif is left with nothing and removed"
    );
    assert_eq!(c.outputs.rate_for("alsa", "dac"), 48_000);
    assert_eq!(c.outputs.buffer_for("alsa", "phones"), 512);
    assert_eq!(
        warnings
            .iter()
            .filter(|w| w.field == "outputs.device_overrides")
            .count(),
        3,
        "{warnings:?}"
    );
}

#[test]
fn the_range_ends_are_valid() {
    let mut c = Config::default();
    c.outputs.device_overrides = vec![
        own("low", Some(8_000), Some(16)),
        own("high", Some(768_000), Some(16_384)),
    ];
    let kept = c.outputs.device_overrides.clone();
    assert!(c.validate().is_empty());
    assert_eq!(c.outputs.device_overrides, kept);
}

#[test]
fn a_device_listed_twice_keeps_its_first_values() {
    let mut c = Config::default();
    c.outputs.device_overrides = vec![
        own("dac", Some(96_000), None),
        own("dac", Some(44_100), None),
    ];
    let warnings = c.validate();
    assert_eq!(
        c.outputs.device_overrides,
        vec![own("dac", Some(96_000), None)]
    );
    assert!(
        warnings
            .iter()
            .any(|w| w.field == "outputs.device_overrides")
    );
}

#[test]
fn an_empty_entry_is_removed_without_a_warning() {
    let mut c = Config::default();
    c.outputs.device_overrides = vec![own("dac", None, None)];
    assert!(c.validate().is_empty());
    assert!(c.outputs.device_overrides.is_empty());
}

#[test]
fn routed_devices_lists_each_device_once_in_route_order() {
    let mut c = routed();
    c.outputs.routes.push(PlayerRoutes {
        player: PlayerId(2),
        main: Some(route("phones", 2)),
        cue: Some(route("dac", 2)),
    });
    c.outputs.cartwall.main = Some(route("spdif", 0));
    assert_eq!(
        c.outputs.routed_devices(),
        vec![dev("dac"), dev("phones"), dev("spdif")]
    );
}

#[test]
fn advanced_settings_are_in_use_only_where_they_apply() {
    let base = routed();
    assert!(!base.outputs.advanced_in_use(), "the defaults");

    let mut c = base.clone();
    c.outputs.bit_perfect.push(dev("dac"));
    assert!(c.outputs.advanced_in_use(), "a routed bit-perfect device");

    let mut c = base.clone();
    c.outputs.set_device_buffer(&dev("phones"), Some(256));
    assert!(c.outputs.advanced_in_use(), "a routed device's own buffer");

    let mut c = base.clone();
    c.outputs.set_device_rate(&dev("gone"), Some(96_000));
    c.outputs.bit_perfect.push(dev("gone"));
    assert!(
        !c.outputs.advanced_in_use(),
        "a device no route uses is not shown, so it does not count"
    );

    let mut c = base.clone();
    c.outputs.dsd_mix = DsdMix::HoldOthers;
    assert!(c.outputs.advanced_in_use(), "the DSD mix");

    let mut c = base;
    c.outputs.dsd_silence_ms = 500.0;
    assert!(c.outputs.advanced_in_use(), "the DSD silence");
}
