#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Live settings spec (2026-10-07): one test per rule L1–L10, L20,
//! L22–L24. The engine is stood in for by the events it reports.

mod common;

use common::{entries, fixture, p0, roundtrip};
use fp_model::{
    AppState, BusyCause, CartId, Command, Config, DeviceSettings, DsdDevice, DsdMix, DsdOutput,
    EngineAction, EngineEvent, Failure, Holder, LiveSettings, OutputDevice, PendingItem, Route,
    Target, TrackAnalysis, Wanted, apply, causes, configured_route, device_causes, device_settings,
    due, has_output_items, interruptions, on_event, pending,
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
fn a_refusal_for_a_device_no_holder_uses_is_not_recorded() {
    let mut s = fixture(1);
    report_start(&mut s);
    let target = Target::Device(dev("gone"));
    let wanted = Wanted::Device(device_settings(&s.config.outputs, &dev("gone")));
    on_event(
        &mut s,
        EngineEvent::Applied {
            target: target.clone(),
            wanted,
            outcome: Err("refused".into()),
        },
    );
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

fn update(state: &mut AppState, edit: impl FnOnce(&mut Config)) -> Vec<fp_model::EngineAction> {
    let mut config = state.config.clone();
    edit(&mut config);
    apply(state, Command::UpdateConfig(Box::new(config))).unwrap()
}

fn device_item(state: &AppState, name: &str) -> Option<fp_model::Pending> {
    pending(state)
        .into_iter()
        .find(|p| matches!(&p.item, PendingItem::Device { device, .. } if *device == dev(name)))
}

#[test]
fn nothing_is_pending_while_the_engine_runs_the_configuration() {
    let mut s = fixture(1);
    assert!(pending(&s).is_empty(), "nothing reported");
    report_start(&mut s);
    assert!(pending(&s).is_empty());
}

#[test]
fn l5_a_busy_device_delays_only_its_own_change() {
    let mut s = fixture(3);
    let p1 = p0(&s);
    s.config.outputs.routes[0].main = Some(route_to("dac"));
    report_start(&mut s);
    apply(&mut s, Command::Play(p1)).unwrap();
    update(&mut s, |c| c.outputs.buffer_frames = 1024);
    let dac = device_item(&s, "dac").unwrap();
    assert_eq!(dac.causes, vec![BusyCause::PlayerPlaying(p1)]);
    let PendingItem::Device { from, to, .. } = dac.item else {
        panic!("a device item")
    };
    assert_eq!((from.buffer_frames, to.buffer_frames), (512, 1024));
    assert!(device_item(&s, "default").unwrap().causes.is_empty());
}

fn apply_devices(actions: &[EngineAction]) -> Vec<OutputDevice> {
    actions
        .iter()
        .filter_map(|a| match a {
            EngineAction::ApplyDevice { device, .. } => Some(device.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn l8_waiting_change_is_sent_only_when_idle() {
    let mut s = fixture(3);
    let p1 = p0(&s);
    s.config.outputs.routes[0].main = Some(route_to("dac"));
    report_start(&mut s);
    apply(&mut s, Command::Play(p1)).unwrap();
    let entry = s.players[0].current.unwrap();
    let actions = update(&mut s, |c| c.outputs.buffer_frames = 1024);
    let sent = apply_devices(&actions);
    assert!(sent.contains(&dev("default")), "the idle device goes out");
    assert!(!sent.contains(&dev("dac")), "the busy device waits");
    // The track ends: the device is idle and the waiting change goes out.
    let actions = on_event(&mut s, EngineEvent::ReachedEnd { player: p1, entry });
    assert_eq!(apply_devices(&actions), vec![dev("dac")]);
}

#[test]
fn l6_a_route_change_waits_for_its_own_holder_only() {
    let mut s = fixture(3);
    let (p1, p2) = (s.players[0].id, s.players[1].id);
    report_start(&mut s);
    apply(&mut s, Command::Play(p1)).unwrap();
    update(&mut s, |c| {
        c.outputs.routes[0].main = Some(route_to("dac"));
        c.outputs.routes[1].main = Some(route_to("dac"));
    });
    let route_of = |h: Holder| {
        pending(&s)
            .into_iter()
            .find(|p| matches!(p.item, PendingItem::Route { holder, .. } if holder == h))
            .unwrap()
    };
    assert_eq!(
        route_of(Holder::PlayerMain(p1)).causes,
        vec![BusyCause::PlayerPlaying(p1)]
    );
    assert!(
        route_of(Holder::PlayerMain(p2)).causes.is_empty(),
        "P2 is idle: moving it interrupts nobody (Q1)"
    );
}

#[test]
fn l7_the_audio_system_waits_until_nothing_plays_anywhere() {
    let mut s = fixture(3);
    let p1 = p0(&s);
    report_start(&mut s);
    let cart = load_cart(&mut s, 0);
    apply(&mut s, Command::Play(p1)).unwrap();
    apply(&mut s, Command::FireCart(cart)).unwrap();
    update(&mut s, |c| c.outputs.backend = Some("other".into()));
    let item = pending(&s)
        .into_iter()
        .find(|p| matches!(p.item, PendingItem::AudioSystem { .. }))
        .unwrap();
    assert_eq!(
        item.item,
        PendingItem::AudioSystem {
            from: None,
            to: Some("other".into())
        }
    );
    assert_eq!(
        item.causes,
        vec![BusyCause::PlayerPlaying(p1), BusyCause::CartPlaying(cart)]
    );
}

#[test]
fn a_device_no_holder_is_placed_on_is_never_pending() {
    let mut s = fixture(1);
    report_start(&mut s);
    update(&mut s, |c| {
        c.outputs.set_device_buffer(&dev("unused"), Some(1024))
    });
    assert!(device_item(&s, "unused").is_none());
}

#[test]
fn a_removed_player_is_never_pending() {
    let mut s = fixture(1);
    let last = s.players[3].id;
    report_start(&mut s);
    update(&mut s, |c| c.outputs.routes[3].main = Some(route_to("dac")));
    apply(&mut s, Command::SetPlayerCount(3)).unwrap();
    assert!(
        !pending(&s)
            .iter()
            .any(|p| matches!(p.item, PendingItem::Route { holder: Holder::PlayerMain(h), .. } if h == last)),
        "the engine has not said Gone yet, but the player is gone"
    );
}

fn apply_device(state: &AppState, name: &str, force: bool) -> EngineAction {
    EngineAction::ApplyDevice {
        device: dev(name),
        settings: device_settings(&state.config.outputs, &dev(name)),
        force,
    }
}

#[test]
fn l8_an_idle_change_is_sent_once_and_completes_with_applied() {
    let mut s = fixture(1);
    report_start(&mut s);
    let actions = update(&mut s, |c| c.outputs.sample_rate = 44_100);
    let wanted = apply_device(&s, "default", false);
    assert!(actions.contains(&wanted));
    assert!(actions.contains(&apply_device(&s, "phones", false)));
    let again = apply(&mut s, Command::SetCartwallOpen(true)).unwrap();
    assert!(!again.contains(&wanted), "in flight: not sent twice");
    assert!(due(&s).is_empty());
    let settings = device_settings(&s.config.outputs, &dev("default"));
    on_event(
        &mut s,
        EngineEvent::Applied {
            target: Target::Device(dev("default")),
            wanted: Wanted::Device(settings),
            outcome: Ok(()),
        },
    );
    assert!(device_item(&s, "default").is_none());
    assert!(
        !s.live
            .in_flight
            .contains_key(&Target::Device(dev("default")))
    );
}

#[test]
fn l8_a_refused_value_is_not_tried_again_until_the_setting_changes() {
    let mut s = fixture(1);
    report_start(&mut s);
    update(&mut s, |c| c.outputs.sample_rate = 44_100);
    let settings = device_settings(&s.config.outputs, &dev("default"));
    on_event(
        &mut s,
        EngineEvent::Applied {
            target: Target::Device(dev("default")),
            wanted: Wanted::Device(settings),
            outcome: Err("44100 Hz refused".into()),
        },
    );
    let item = device_item(&s, "default").unwrap();
    assert_eq!(item.failure.as_deref(), Some("44100 Hz refused"));
    let again = apply(&mut s, Command::SetCartwallOpen(true)).unwrap();
    assert!(!again.contains(&apply_device(&s, "default", false)));
    let actions = update(&mut s, |c| c.outputs.sample_rate = 96_000);
    assert!(actions.contains(&apply_device(&s, "default", false)));
    assert_eq!(device_item(&s, "default").unwrap().failure, None);
}

#[test]
fn l8_an_answer_for_an_older_value_keeps_the_newer_one_in_flight() {
    let mut s = fixture(1);
    report_start(&mut s);
    update(&mut s, |c| c.outputs.sample_rate = 44_100);
    let older = device_settings(&s.config.outputs, &dev("default"));
    update(&mut s, |c| c.outputs.sample_rate = 96_000);
    let newer = device_settings(&s.config.outputs, &dev("default"));
    on_event(
        &mut s,
        EngineEvent::Applied {
            target: Target::Device(dev("default")),
            wanted: Wanted::Device(older),
            outcome: Ok(()),
        },
    );
    assert_eq!(s.live.devices.get(&dev("default")), Some(&older));
    assert_eq!(
        s.live.in_flight.get(&Target::Device(dev("default"))),
        Some(&Wanted::Device(newer))
    );
}

#[test]
fn l9_the_audio_system_goes_first_then_routes_then_devices() {
    let mut s = fixture(1);
    report_start(&mut s);
    let actions = update(&mut s, |c| {
        c.outputs.backend = Some("other".into());
        c.outputs.routes[0].main = Some(route_to("dac"));
        c.outputs.buffer_frames = 1024;
    });
    let kinds: Vec<u8> = actions
        .iter()
        .filter_map(|a| match a {
            EngineAction::ApplyAudioSystem { .. } => Some(0),
            EngineAction::ApplyRoute { .. } => Some(1),
            EngineAction::ApplyDevice { .. } => Some(2),
            _ => None,
        })
        .collect();
    let mut sorted = kinds.clone();
    sorted.sort_unstable();
    assert_eq!(kinds, sorted);
    assert_eq!(kinds.first(), Some(&0));
    assert!(kinds.contains(&1) && kinds.contains(&2));
}

#[test]
fn l10_update_config_tells_the_engine_only_when_outputs_or_tuning_change() {
    let mut s = fixture(1);
    let actions = update(&mut s, |c| c.players.fade_ms = 2500);
    assert!(
        !actions
            .iter()
            .any(|a| matches!(a, EngineAction::UpdateSettings(_)))
    );
    let actions = update(&mut s, |c| c.outputs.buffer_frames = 1024);
    assert!(actions.contains(&EngineAction::UpdateSettings(Box::new(s.config.clone()))));
    let actions = update(&mut s, |c| c.tuning.prebuffer_secs = 8.0);
    assert!(
        actions.iter().any(
            |a| matches!(a, EngineAction::UpdateSettings(c) if c.tuning.prebuffer_secs == 8.0)
        )
    );
}

#[test]
fn changing_a_setting_back_while_it_waits_leaves_nothing_pending() {
    let mut s = fixture(3);
    let p1 = p0(&s);
    report_start(&mut s);
    apply(&mut s, Command::Play(p1)).unwrap();
    update(&mut s, |c| c.outputs.buffer_frames = 1024);
    assert!(!device_item(&s, "default").unwrap().causes.is_empty());
    let actions = update(&mut s, |c| c.outputs.buffer_frames = 512);
    assert!(device_item(&s, "default").is_none());
    assert!(!actions.iter().any(
        |a| matches!(a, EngineAction::ApplyDevice { device, .. } if *device == dev("default"))
    ));
}

#[test]
fn l23_limits_never_make_anything_pending_nor_reach_the_engine() {
    let mut s = fixture(1);
    report_start(&mut s);
    let actions = update(&mut s, |c| {
        c.limits.max_cover_bytes *= 2;
        c.limits.max_tag_chars *= 2;
        c.limits.max_players = 32;
        c.limits.max_cart_rows = 16;
    });
    assert!(pending(&s).is_empty());
    assert!(
        !actions
            .iter()
            .any(|a| matches!(a, EngineAction::UpdateSettings(_))),
        "limits are not engine settings"
    );
    assert_eq!(s.config.limits.max_players, 32);
}

#[test]
fn l20_apply_now_forces_every_output_change_even_one_in_flight() {
    let mut s = fixture(3);
    let last = s.players[3].id;
    report_start(&mut s);
    apply(&mut s, Command::Play(last)).unwrap();
    update(&mut s, |c| c.outputs.buffer_frames = 1024);
    let actions = apply(&mut s, Command::ApplySettingsNow).unwrap();
    assert!(actions.contains(&apply_device(&s, "default", true)));
    assert!(
        actions.contains(&apply_device(&s, "phones", true)),
        "in flight already: sent again, forced"
    );
}

#[test]
fn l20_apply_now_retries_a_refused_value() {
    let mut s = fixture(1);
    report_start(&mut s);
    update(&mut s, |c| c.outputs.sample_rate = 44_100);
    let settings = device_settings(&s.config.outputs, &dev("default"));
    on_event(
        &mut s,
        EngineEvent::Applied {
            target: Target::Device(dev("default")),
            wanted: Wanted::Device(settings),
            outcome: Err("refused".into()),
        },
    );
    let actions = apply(&mut s, Command::ApplySettingsNow).unwrap();
    assert!(actions.contains(&apply_device(&s, "default", true)));
}

#[test]
fn l22_interruptions_name_only_what_apply_now_would_cut() {
    let mut s = fixture(3);
    let (p1, last) = (p0(&s), s.players[3].id);
    report_start(&mut s);
    assert!(interruptions(&s).is_empty());
    assert!(!has_output_items(&s));
    apply(&mut s, Command::Play(p1)).unwrap();
    apply(&mut s, Command::Play(last)).unwrap();
    update(&mut s, |c| c.outputs.buffer_frames = 1024);
    assert_eq!(
        interruptions(&s),
        vec![(
            Target::Device(dev("default")),
            vec![BusyCause::PlayerPlaying(p1), BusyCause::PlayerPlaying(last)]
        )]
    );
    assert!(has_output_items(&s));
}

#[test]
fn l24_nothing_pending_is_saved() {
    let mut s = fixture(3);
    let p1 = p0(&s);
    report_start(&mut s);
    apply(&mut s, Command::Play(p1)).unwrap();
    update(&mut s, |c| c.outputs.buffer_frames = 1024);
    assert!(!pending(&s).is_empty());
    let restored = roundtrip(&s);
    assert_eq!(restored.live, LiveSettings::default());
    assert!(pending(&restored).is_empty());
}

#[test]
fn l4_an_own_value_equal_to_the_global_one_changes_nothing() {
    let mut c = Config::default();
    c.outputs.routes = vec![fp_model::PlayerRoutes {
        player: fp_model::PlayerId(1),
        main: Some(route_to("dac")),
        cue: None,
    }];
    let before = device_settings(&c.outputs, &dev("dac"));
    c.outputs
        .set_device_rate(&dev("dac"), Some(c.outputs.sample_rate));
    c.outputs
        .set_device_buffer(&dev("dac"), Some(c.outputs.buffer_frames));
    assert_eq!(device_settings(&c.outputs, &dev("dac")), before);
}

#[test]
fn l4_an_own_value_on_a_device_no_route_names_applies_to_nothing() {
    let mut c = Config::default();
    let before = device_settings(&c.outputs, &dev("spare"));
    c.outputs.set_device_rate(&dev("spare"), Some(96_000));
    assert_eq!(device_settings(&c.outputs, &dev("spare")), before);
}
