#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback 2 spec O4: which changed settings wait for a restart.

use fp_model::{
    Config, OutputDevice, PlayerId, PlayerRoutes, RestartReason, Route, restart_pending,
};

fn route(device: &str) -> Route {
    Route {
        backend: "alsa".into(),
        device: device.into(),
        first_channel: 0,
    }
}

#[test]
fn the_same_configuration_needs_no_restart() {
    let c = Config::default();
    assert!(restart_pending(&c, &c.clone()).is_empty());
}

#[test]
fn each_start_up_setting_gives_its_reason() {
    type Edit = fn(&mut Config);
    let cases: [(Edit, RestartReason); 8] = [
        (
            |c| c.outputs.backend = Some("jack".into()),
            RestartReason::AudioSystem,
        ),
        (
            |c| c.outputs.sample_rate = 44_100,
            RestartReason::SampleRate,
        ),
        (
            |c| c.outputs.buffer_frames = 1024,
            RestartReason::BufferSize,
        ),
        (
            |c| {
                c.outputs.routes.push(PlayerRoutes {
                    player: PlayerId(1),
                    main: Some(route("dac")),
                    cue: None,
                })
            },
            RestartReason::Routes,
        ),
        (
            |c| c.outputs.cartwall.cue = Some(route("phones")),
            RestartReason::Routes,
        ),
        (
            |c| {
                c.outputs.bit_perfect.push(OutputDevice {
                    backend: "alsa".into(),
                    device: "dac".into(),
                })
            },
            RestartReason::BitPerfect,
        ),
        (|c| c.limits.max_players = 8, RestartReason::Limits),
        (|c| c.tuning.prebuffer_secs = 10.0, RestartReason::Tuning),
    ];
    for (edit, reason) in cases {
        let started = Config::default();
        let mut current = started.clone();
        edit(&mut current);
        assert_eq!(restart_pending(&started, &current), vec![reason]);
    }
}

#[test]
fn settings_applied_while_running_need_no_restart() {
    let started = Config::default();
    let mut c = started.clone();
    c.players.count = 8;
    c.players.fade_ms = 3000;
    c.meter.floor_db = -40.0;
    c.analysis.segue_drop_db = 20.0;
    c.ui.language = Some("es-ES".into());
    c.cartwall.default_rows = 4;
    c.shortcuts.clear();
    c.midi.enabled = true;
    c.remote.http.enabled = true;
    assert!(restart_pending(&started, &c).is_empty());
}

#[test]
fn changing_back_clears_the_reason() {
    let started = Config::default();
    let mut c = started.clone();
    c.outputs.sample_rate = 44_100;
    assert_eq!(
        restart_pending(&started, &c),
        vec![RestartReason::SampleRate]
    );
    c.outputs.sample_rate = started.outputs.sample_rate;
    assert!(restart_pending(&started, &c).is_empty());
}

#[test]
fn empty_and_reordered_routes_are_no_change() {
    let mut started = Config::default();
    started.outputs.routes = vec![
        PlayerRoutes {
            player: PlayerId(1),
            main: Some(route("dac")),
            cue: None,
        },
        PlayerRoutes {
            player: PlayerId(2),
            main: None,
            cue: Some(route("phones")),
        },
    ];
    started.outputs.bit_perfect = vec![
        OutputDevice {
            backend: "alsa".into(),
            device: "dac".into(),
        },
        OutputDevice {
            backend: "alsa".into(),
            device: "phones".into(),
        },
    ];
    let mut c = started.clone();
    c.outputs.routes.reverse();
    c.outputs.routes.push(PlayerRoutes {
        player: PlayerId(3),
        main: None,
        cue: None,
    });
    c.outputs.bit_perfect.reverse();
    assert!(restart_pending(&started, &c).is_empty());
}

#[test]
fn several_changes_are_listed_once_each_in_order() {
    let started = Config::default();
    let mut c = started.clone();
    c.tuning.prebuffer_secs = 10.0;
    c.outputs.sample_rate = 44_100;
    c.outputs.cartwall.main = Some(route("dac"));
    c.outputs.cartwall.cue = Some(route("phones"));
    assert_eq!(
        restart_pending(&started, &c),
        vec![
            RestartReason::SampleRate,
            RestartReason::Routes,
            RestartReason::Tuning
        ]
    );
}

#[test]
fn dsd_settings_apply_at_restart() {
    let started = Config::default();
    for change in [
        |c: &mut Config| c.outputs.dsd_mix = fp_model::DsdMix::HoldOthers,
        |c: &mut Config| c.outputs.dsd_silence_ms = 500.0,
        |c: &mut Config| {
            c.outputs.dsd_output.push(fp_model::DsdDevice {
                backend: "alsa".into(),
                device: "hw:0".into(),
                mode: fp_model::DsdOutput::Dop,
            })
        },
    ] {
        let mut current = started.clone();
        change(&mut current);
        assert_eq!(
            restart_pending(&started, &current),
            vec![RestartReason::DsdOutput]
        );
    }
}

fn routed_to(c: &mut Config, device: &str) {
    c.outputs.routes.push(PlayerRoutes {
        player: PlayerId(1),
        main: Some(route(device)),
        cue: None,
    });
}

#[test]
fn a_routed_devices_own_rate_or_buffer_waits_for_a_restart() {
    let dac = OutputDevice {
        backend: "alsa".into(),
        device: "dac".into(),
    };
    let mut started = Config::default();
    routed_to(&mut started, "dac");
    let mut c = started.clone();
    c.outputs.set_device_rate(&dac, Some(96_000));
    assert_eq!(
        restart_pending(&started, &c),
        vec![RestartReason::SampleRate]
    );
    let mut c = started.clone();
    c.outputs.set_device_buffer(&dac, Some(1024));
    assert_eq!(
        restart_pending(&started, &c),
        vec![RestartReason::BufferSize]
    );
}

#[test]
fn an_unrouted_devices_own_values_do_not_ask_for_a_restart() {
    // The engine applies an override only to a routed device, so an
    // override on an unrouted one changes nothing until it is routed.
    let dac = OutputDevice {
        backend: "alsa".into(),
        device: "dac".into(),
    };
    let started = Config::default();
    let mut c = started.clone();
    c.outputs.set_device_rate(&dac, Some(96_000));
    c.outputs.set_device_buffer(&dac, Some(1024));
    assert!(restart_pending(&started, &c).is_empty());
}

#[test]
fn routing_then_unrouting_a_device_with_its_own_values_needs_no_restart() {
    let dac = OutputDevice {
        backend: "alsa".into(),
        device: "dac".into(),
    };
    let started = Config::default();
    let mut c = started.clone();
    routed_to(&mut c, "dac");
    c.outputs.set_device_rate(&dac, Some(96_000));
    c.outputs.set_device_buffer(&dac, Some(1024));
    // A device routed in only one configuration is a change of routes.
    assert_eq!(restart_pending(&started, &c), vec![RestartReason::Routes]);
    c.outputs.routes.clear();
    assert!(
        restart_pending(&started, &c).is_empty(),
        "the routes equal the started ones; the override applies to no device"
    );
}

#[test]
fn reordered_overrides_and_the_outputs_view_need_no_restart() {
    let dev = |name: &str| OutputDevice {
        backend: "alsa".into(),
        device: name.into(),
    };
    let mut started = Config::default();
    started.outputs.set_device_rate(&dev("dac"), Some(96_000));
    started.outputs.set_device_buffer(&dev("phones"), Some(256));
    let mut c = started.clone();
    c.outputs.device_overrides.reverse();
    c.ui.outputs_view = fp_model::OutputsView::Advanced;
    assert!(restart_pending(&started, &c).is_empty());
}

#[test]
fn a_devices_own_value_equal_to_the_global_one_needs_no_restart() {
    let dac = OutputDevice {
        backend: "alsa".into(),
        device: "dac".into(),
    };
    let started = Config::default();
    let mut c = started.clone();
    c.outputs
        .set_device_rate(&dac, Some(started.outputs.sample_rate));
    c.outputs
        .set_device_buffer(&dac, Some(started.outputs.buffer_frames));
    assert!(restart_pending(&started, &c).is_empty());
    // Clearing an own value that equals the global one changes nothing either.
    assert!(restart_pending(&c, &started).is_empty());
    // An own value equal to a changed global rate does not need one when the
    // device already had it.
    let mut was = started.clone();
    was.outputs.set_device_rate(&dac, Some(96_000));
    let mut now = was.clone();
    now.outputs.sample_rate = 96_000;
    assert_eq!(
        restart_pending(&was, &now),
        vec![RestartReason::SampleRate],
        "the global rate itself changed"
    );
}
