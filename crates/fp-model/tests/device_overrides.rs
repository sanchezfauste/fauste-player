#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Operator feedback 4, Q12: each device's own rate and buffer, and the
//! Basic | Advanced view of Settings → Audio outputs.

use fp_model::{
    AppState, Command, Config, DeviceOverride, DsdDevice, DsdMix, DsdOutput, OutputDevice,
    OutputsView, PlayerId, PlayerRoutes, Route, apply,
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

#[test]
fn the_dsd_settings_count_only_with_a_routed_device() {
    // Advanced hides the DSD rows when no route names a device.
    let mut c = Config::default();
    c.outputs.dsd_mix = DsdMix::HoldOthers;
    c.outputs.dsd_silence_ms = 500.0;
    assert!(c.outputs.routed_devices().is_empty());
    assert!(!c.outputs.advanced_in_use());
}

fn dsd(name: &str) -> DsdDevice {
    DsdDevice {
        backend: "alsa".into(),
        device: name.into(),
        mode: DsdOutput::Dop,
    }
}

/// `routed()` with own values, bit-perfect and DSD on `dac`, `phones` and
/// `gone`, a device no route names.
fn with_settings_on(mut c: Config) -> Config {
    for name in ["dac", "phones", "gone"] {
        c.outputs.set_device_rate(&dev(name), Some(96_000));
        c.outputs.bit_perfect.push(dev(name));
        c.outputs.dsd_output.push(dsd(name));
    }
    c
}

#[test]
fn a_device_no_route_names_loses_its_own_settings() {
    let mut c = with_settings_on(routed());
    c.outputs.forget_unrouted_devices();
    assert_eq!(
        c.outputs.device_overrides,
        vec![
            own("dac", Some(96_000), None),
            own("phones", Some(96_000), None)
        ]
    );
    assert_eq!(c.outputs.bit_perfect, vec![dev("dac"), dev("phones")]);
    assert_eq!(c.outputs.dsd_output, vec![dsd("dac"), dsd("phones")]);
}

#[test]
fn a_cartwall_route_keeps_its_devices_settings() {
    let mut c = with_settings_on(routed());
    c.outputs.cartwall.cue = Some(route("gone", 2));
    c.outputs.forget_unrouted_devices();
    assert!(c.outputs.device_override(&dev("gone")).is_some());
    assert!(c.outputs.bit_perfect.contains(&dev("gone")));
}

#[test]
fn without_routes_every_device_setting_goes_and_the_globals_stay() {
    let mut c = with_settings_on(Config::default());
    c.outputs.sample_rate = 44_100;
    c.outputs.buffer_frames = 1024;
    c.outputs.forget_unrouted_devices();
    assert!(c.outputs.device_overrides.is_empty());
    assert!(c.outputs.bit_perfect.is_empty());
    assert!(c.outputs.dsd_output.is_empty());
    assert_eq!(
        (c.outputs.sample_rate, c.outputs.buffer_frames),
        (44_100, 1024),
        "the system default output keeps the global values"
    );
}

#[test]
fn a_config_update_that_unroutes_a_device_drops_its_settings() {
    let mut state = AppState::new(with_settings_on(routed()), "Main");
    let mut config = state.config.clone();
    // Player 1's Main goes back to the system default output.
    if let Some(r) = config.outputs.routes.first_mut() {
        r.main = None;
    }
    apply(&mut state, Command::UpdateConfig(Box::new(config))).unwrap();
    let o = &state.config.outputs;
    assert!(o.device_override(&dev("dac")).is_none());
    assert!(!o.bit_perfect.contains(&dev("dac")));
    assert!(o.dsd_output.iter().all(|d| d.device != "dac"));
    assert_eq!(o.device_overrides, vec![own("phones", Some(96_000), None)]);

    // Routed again, the device starts from the global values.
    let mut config = state.config.clone();
    if let Some(r) = config.outputs.routes.first_mut() {
        r.main = Some(route("dac", 0));
    }
    apply(&mut state, Command::UpdateConfig(Box::new(config))).unwrap();
    assert_eq!(state.config.outputs.effective_rate(&dev("dac")), 48_000);
    assert!(!state.config.outputs.bit_perfect.contains(&dev("dac")));
}
