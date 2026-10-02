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
