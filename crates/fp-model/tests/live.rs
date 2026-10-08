#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Live settings spec (2026-10-07): one test per rule L1–L10, L20,
//! L22–L24. The engine is stood in for by the events it reports.

mod common;

use common::{entries, fixture, p0};
use fp_model::{
    AppState, BusyCause, CartId, Command, Config, DeviceSettings, DsdDevice, DsdMix, DsdOutput,
    EngineEvent, Failure, Holder, LiveSettings, OutputDevice, Route, Target, TrackAnalysis, Wanted,
    apply, causes, configured_route, device_causes, device_settings, on_event,
};

fn dev(name: &str) -> OutputDevice {
    OutputDevice {
        backend: "null".into(),
        device: name.into(),
    }
}

fn route_to(name: &str) -> Route {
    Route {
        backend: "null".into(),
        device: name.into(),
        first_channel: 0,
    }
}

#[test]
fn the_live_state_starts_empty_and_unknown() {
    let s = AppState::new(Config::default(), "Main");
    assert_eq!(s.live, LiveSettings::default());
    assert_eq!(s.live.audio_system, None, "unknown until the engine says");
}

#[test]
fn l4_device_settings_follow_the_effective_rate_buffer_and_bit_perfect() {
    let mut c = Config::default();
    c.outputs.routes = vec![fp_model::PlayerRoutes {
        player: fp_model::PlayerId(1),
        main: Some(route_to("dac")),
        cue: None,
    }];
    c.outputs.set_device_rate(&dev("dac"), Some(96_000));
    c.outputs.set_device_buffer(&dev("dac"), Some(256));
    c.outputs.bit_perfect = vec![dev("dac")];
    assert_eq!(
        device_settings(&c.outputs, &dev("dac")),
        DeviceSettings {
            sample_rate: 96_000,
            buffer_frames: 256,
            bit_perfect: true,
            dsd: DsdOutput::Pcm,
            dsd_mix: None,
            dsd_silence_ms: None,
        }
    );
    // An unrouted device opens at the global values.
    let other = device_settings(&c.outputs, &dev("other"));
    assert_eq!((other.sample_rate, other.buffer_frames), (48_000, 512));
    assert!(!other.bit_perfect);
}

#[test]
fn l4_dsd_mix_and_silence_count_only_on_a_device_that_carries_dsd() {
    let mut c = Config::default();
    c.outputs.routes = vec![fp_model::PlayerRoutes {
        player: fp_model::PlayerId(1),
        main: Some(route_to("dac")),
        cue: Some(route_to("phones")),
    }];
    c.outputs.bit_perfect = vec![dev("dac")];
    c.outputs.dsd_output = vec![DsdDevice {
        backend: "null".into(),
        device: "dac".into(),
        mode: DsdOutput::Dop,
    }];
    let before_dac = device_settings(&c.outputs, &dev("dac"));
    let before_phones = device_settings(&c.outputs, &dev("phones"));
    c.outputs.dsd_mix = DsdMix::HoldOthers;
    c.outputs.dsd_silence_ms = 400.0;
    let dac = device_settings(&c.outputs, &dev("dac"));
    assert_eq!(dac.dsd, DsdOutput::Dop);
    assert_eq!(dac.dsd_mix, Some(DsdMix::HoldOthers));
    assert_eq!(dac.dsd_silence_ms, Some(400.0));
    assert_ne!(dac, before_dac);
    assert_eq!(
        device_settings(&c.outputs, &dev("phones")),
        before_phones,
        "a PCM device does not change"
    );
    // A DSD mode on a device that is not bit-perfect is PCM.
    c.outputs.bit_perfect.clear();
    assert_eq!(device_settings(&c.outputs, &dev("dac")).dsd, DsdOutput::Pcm);
}

/// Gives cart `index` of the first page a 10 s file.
fn load_cart(state: &mut AppState, index: usize) -> CartId {
    let page = state.cartwall.pages[0].id;
    let path = std::path::PathBuf::from(format!("/carts/cart{index}.wav"));
    apply(state, Command::AssignCartFile { page, index, path }).unwrap();
    let track = state.cartwall.pages[0].carts[index].track.unwrap();
    let analysis = TrackAnalysis {
        duration_secs: 10.0,
        ..TrackAnalysis::default()
    };
    apply(
        state,
        Command::ApplyAnalysis {
            track,
            analysis: Box::new(analysis),
        },
    )
    .unwrap();
    state.cartwall.pages[0].carts[index].id
}

#[test]
fn l1_a_playing_or_fading_player_is_busy_a_paused_or_loaded_one_is_not() {
    let mut s = fixture(3);
    let p = p0(&s);
    let main = Holder::PlayerMain(p);
    assert!(causes(&s, main).is_empty(), "stopped with a track loaded");
    apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(causes(&s, main), vec![BusyCause::PlayerPlaying(p)]);
    apply(&mut s, Command::Pause(p)).unwrap();
    assert!(causes(&s, main).is_empty(), "paused is not busy (D2)");
    apply(&mut s, Command::Pause(p)).unwrap();
    apply(&mut s, Command::FadeStop(p)).unwrap();
    assert_eq!(causes(&s, main), vec![BusyCause::PlayerFading(p)]);
}

#[test]
fn l1_a_cue_playing_or_held_keeps_the_players_cue_busy() {
    let mut s = fixture(3);
    let (p, e) = (p0(&s), entries(&s));
    apply(&mut s, Command::CueEntry(p, e[1])).unwrap();
    assert_eq!(
        causes(&s, Holder::PlayerCue(p)),
        vec![BusyCause::PlayerCue(p)]
    );
    apply(&mut s, Command::SetCuePaused(p, true)).unwrap();
    assert_eq!(
        causes(&s, Holder::PlayerCue(p)),
        vec![BusyCause::PlayerCue(p)],
        "Q4: a held CUE counts"
    );
    assert!(causes(&s, Holder::PlayerMain(p)).is_empty());
}

#[test]
fn l1_playing_carts_keep_the_cartwall_main_busy_and_a_cart_cue_its_cue() {
    let mut s = fixture(0);
    let (a, b) = (load_cart(&mut s, 0), load_cart(&mut s, 1));
    apply(&mut s, Command::FireCart(a)).unwrap();
    apply(&mut s, Command::FireCart(b)).unwrap();
    assert_eq!(
        causes(&s, Holder::CartwallMain),
        vec![BusyCause::CartPlaying(a), BusyCause::CartPlaying(b)]
    );
    assert!(causes(&s, Holder::CartwallCue).is_empty());
    let c = load_cart(&mut s, 2);
    apply(&mut s, Command::CueCart(c)).unwrap();
    assert_eq!(causes(&s, Holder::CartwallCue), vec![BusyCause::CartCue(c)]);
}

#[test]
fn l2_a_device_collects_the_causes_of_every_holder_placed_on_it() {
    let mut s = fixture(3);
    let (p1, p2) = (s.players[0].id, s.players[1].id);
    // Placements as the engine reports them (Task 3 adds the events).
    for p in s.players.clone() {
        s.live
            .placement
            .insert(Holder::PlayerMain(p.id), dev("default"));
        s.live
            .placement
            .insert(Holder::PlayerCue(p.id), dev("phones"));
    }
    apply(&mut s, Command::Play(p1)).unwrap();
    apply(&mut s, Command::Play(p2)).unwrap();
    assert_eq!(
        device_causes(&s, &dev("default")),
        vec![BusyCause::PlayerPlaying(p1), BusyCause::PlayerPlaying(p2)]
    );
    assert!(device_causes(&s, &dev("phones")).is_empty(), "no CUE");
    assert!(device_causes(&s, &dev("elsewhere")).is_empty());
}

/// Reports a holder as the engine does (L3).
fn report(
    state: &mut AppState,
    holder: Holder,
    route: Option<Route>,
    target: Option<OutputDevice>,
) {
    let event = match target {
        Some(device) => EngineEvent::Placed {
            holder,
            route,
            running: device_settings(&state.config.outputs, &device),
            device,
        },
        None => EngineEvent::Unplaced { holder, route },
    };
    on_event(state, event);
}

/// What the engine reports at start (L3): the audio system, each
/// player's Main on its route's device (`default` without one), its Cue
/// on its route's device, and the cartwall unplaced.
fn report_start(state: &mut AppState) {
    let configured = state.config.outputs.backend.clone();
    on_event(
        state,
        EngineEvent::AudioSystemInUse {
            configured,
            in_use: "null".into(),
        },
    );
    let ids: Vec<_> = state.players.iter().map(|p| p.id).collect();
    for id in ids {
        for holder in [Holder::PlayerMain(id), Holder::PlayerCue(id)] {
            let route = configured_route(&state.config, holder);
            let target = match (&route, holder) {
                (Some(r), _) => Some(dev(&r.device)),
                (None, Holder::PlayerMain(_)) => Some(dev("default")),
                (None, _) => None,
            };
            report(state, holder, route, target);
        }
    }
    for holder in [Holder::CartwallMain, Holder::CartwallCue] {
        let route = configured_route(&state.config, holder);
        report(state, holder, route, None);
    }
}

#[test]
fn l3_the_engine_reports_where_each_holder_plays() {
    let mut s = fixture(1);
    let p = p0(&s);
    report_start(&mut s);
    assert_eq!(s.live.audio_system, Some(None));
    assert_eq!(s.live.audio_system_in_use.as_deref(), Some("null"));
    assert_eq!(
        s.live.placement.get(&Holder::PlayerMain(p)),
        Some(&dev("default"))
    );
    assert_eq!(
        s.live.placement.get(&Holder::PlayerCue(p)),
        Some(&dev("phones"))
    );
    assert_eq!(s.live.placement.get(&Holder::CartwallMain), None);
    assert_eq!(
        s.live.routes.get(&Holder::CartwallCue),
        Some(&Some(route_to("phones")))
    );
    assert_eq!(
        s.live.devices.get(&dev("default")),
        Some(&device_settings(&s.config.outputs, &dev("default")))
    );
    on_event(
        &mut s,
        EngineEvent::Gone {
            holder: Holder::PlayerMain(p),
        },
    );
    assert!(!s.live.placement.contains_key(&Holder::PlayerMain(p)));
    assert!(!s.live.routes.contains_key(&Holder::PlayerMain(p)));
}

#[test]
fn a_device_no_holder_uses_any_more_is_forgotten_with_its_failure() {
    let mut s = fixture(1);
    let p = p0(&s);
    s.config.outputs.routes[0].main = Some(route_to("dac"));
    report_start(&mut s);
    let target = Target::Device(dev("dac"));
    let wanted = Wanted::Device(DeviceSettings {
        buffer_frames: 1024,
        ..device_settings(&s.config.outputs, &dev("dac"))
    });
    on_event(
        &mut s,
        EngineEvent::Applied {
            target: target.clone(),
            wanted: wanted.clone(),
            outcome: Err("refused".into()),
        },
    );
    assert_eq!(
        s.live.failures.get(&target),
        Some(&Failure {
            wanted,
            reason: "refused".into()
        })
    );
    // The holder moves to the default output: nothing uses dac any more.
    report(&mut s, Holder::PlayerMain(p), None, Some(dev("default")));
    assert!(!s.live.devices.contains_key(&dev("dac")));
    assert!(!s.live.failures.contains_key(&target));
}

#[test]
fn l14_an_accepted_value_becomes_the_running_one_and_clears_the_failure() {
    let mut s = fixture(1);
    report_start(&mut s);
    let d = dev("default");
    let target = Target::Device(d.clone());
    let settings = DeviceSettings {
        sample_rate: 44_100,
        ..device_settings(&s.config.outputs, &d)
    };
    on_event(
        &mut s,
        EngineEvent::Applied {
            target: target.clone(),
            wanted: Wanted::Device(settings),
            outcome: Err("refused".into()),
        },
    );
    assert_eq!(
        s.live.devices.get(&d).map(|r| r.sample_rate),
        Some(48_000),
        "a refusal keeps the running value"
    );
    on_event(
        &mut s,
        EngineEvent::Applied {
            target: target.clone(),
            wanted: Wanted::Device(settings),
            outcome: Ok(()),
        },
    );
    assert_eq!(s.live.devices.get(&d), Some(&settings));
    assert!(!s.live.failures.contains_key(&target));
}
