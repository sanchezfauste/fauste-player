#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    // The rig serves the tests of live settings plans 2 and 3.
    dead_code
)]
//! Live settings spec (2026-10-07): what the engine reports about where
//! each holder plays (L3), the settings it takes while running (L10, §7),
//! and how it applies an output change (L11–L17, L21).

mod support;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use fp_backends::{AudioBackend, OfflineBackend, OfflineDevice};
use fp_engine::bus::BusKey;
use fp_engine::engine::{Engine, EngineSettings};
use fp_model::{
    AudioFormat, CartId, CartRequest, Config, DeviceSettings, DsdDevice, DsdMix, DsdOutput,
    EngineAction, EngineEvent, EntryId, Holder, OutputDevice, PlayerId, PlayerRoutes, Route,
    SOURCE_END, SourceRequest, Target, TrackId, device_settings,
};
use support::tagged_opener;

const BLOCK: usize = 480;
const P: PlayerId = PlayerId(1);
const Q: PlayerId = PlayerId(2);

fn route(device: &str) -> Route {
    Route {
        backend: "offline".into(),
        device: device.into(),
        first_channel: 0,
    }
}

fn out(device: &str) -> OutputDevice {
    OutputDevice {
        backend: "offline".into(),
        device: device.into(),
    }
}

/// P's Main on `main`, no Cue; the cartwall on the default output (`main`,
/// the first Offline device by name).
fn config() -> Config {
    let mut c = Config::default();
    c.outputs.backend = Some("offline".into());
    c.outputs.buffer_frames = BLOCK as u32;
    c.outputs.routes = vec![PlayerRoutes {
        player: P,
        main: Some(route("main")),
        cue: None,
    }];
    c.tuning.gain_smoothing_ms = 0.0;
    c
}

fn request(track: u64, from_secs: f64) -> SourceRequest {
    SourceRequest {
        entry: EntryId(track),
        track: TrackId(track),
        path: PathBuf::from(format!("track{track}")),
        from_secs,
        format: None,
    }
}

struct Rig {
    engine: Engine,
    main: OfflineDevice,
    other: OfflineDevice,
    clock: Instant,
    events: Vec<EngineEvent>,
}

/// Two Offline devices, `main` (2 channels) and `other` (4), and P added.
fn rig_with(config: &Config) -> Rig {
    let backend = OfflineBackend::new();
    let main = backend.add_device("main", 2);
    let other = backend.add_device("other", 4);
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
    let mut engine = Engine::new(
        backends,
        EngineSettings::from_config(config),
        tagged_opener(48_000 * 20),
    );
    let clock = Instant::now();
    engine.execute(EngineAction::AddPlayer { player: P }, clock);
    Rig {
        engine,
        main,
        other,
        clock,
        events: Vec::new(),
    }
}

fn rig() -> Rig {
    rig_with(&config())
}

impl Rig {
    fn act(&mut self, action: EngineAction) {
        self.engine.execute(action, self.clock);
    }

    fn tick(&mut self) {
        let events = self.engine.tick(self.clock);
        self.events.extend(events);
    }

    /// Waits (bounded) for the decode workers, never for audio.
    fn settle(&mut self) {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            self.tick();
            if self.engine.unsettled_sources() == 0 || Instant::now() > deadline {
                return;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    /// Renders `blocks` blocks on every open device, ticking after each;
    /// returns the left channel `main` played.
    fn run(&mut self, blocks: usize) -> Vec<f32> {
        let mut heard = Vec::new();
        for _ in 0..blocks {
            if let Some(samples) = self.main.render(BLOCK) {
                heard.extend(samples.chunks(2).map(|f| f[0]));
            }
            let _ = self.other.render(BLOCK);
            self.clock += Duration::from_millis(10);
            self.tick();
            // Gives the decode workers time to keep the rings topped up.
            std::thread::sleep(Duration::from_millis(1));
        }
        heard
    }

    fn take_events(&mut self) -> Vec<EngineEvent> {
        std::mem::take(&mut self.events)
    }
}

#[test]
fn l3_the_engine_reports_its_audio_system_and_where_each_holder_plays() {
    let mut r = rig();
    r.tick();
    let events = r.take_events();
    let c = config();
    assert!(events.contains(&EngineEvent::AudioSystemInUse {
        configured: Some("offline".into()),
        in_use: "offline".into(),
    }));
    assert!(events.contains(&EngineEvent::Placed {
        holder: Holder::PlayerMain(P),
        route: Some(route("main")),
        device: out("main"),
        running: device_settings(&c.outputs, &out("main")),
    }));
    assert!(events.contains(&EngineEvent::Unplaced {
        holder: Holder::PlayerCue(P),
        route: None,
    }));
    assert!(events.contains(&EngineEvent::Unplaced {
        holder: Holder::CartwallMain,
        route: None,
    }));
    assert_eq!(
        r.engine.running_settings(&out("main")),
        Some(device_settings(&c.outputs, &out("main")))
    );
    assert_eq!(r.engine.running_settings(&out("other")), None, "not open");
}

#[test]
fn l3_the_first_cart_places_the_cartwall_and_a_removed_player_is_gone() {
    let mut r = rig();
    r.tick();
    r.take_events();
    r.act(EngineAction::StartCart(CartRequest {
        cart: CartId(1),
        track: TrackId(9),
        path: PathBuf::from("track9"),
        from_secs: 0.0,
        until_secs: SOURCE_END,
        looped: false,
        format: None,
    }));
    r.tick();
    let events = r.take_events();
    assert!(events.iter().any(|e| matches!(
        e,
        EngineEvent::Placed { holder: Holder::CartwallMain, route: None, device, .. }
            if *device == out("main")
    )));
    assert!(events.contains(&EngineEvent::Unplaced {
        holder: Holder::CartwallCue,
        route: None,
    }));
    r.act(EngineAction::RemovePlayer { player: P });
    r.tick();
    let events = r.take_events();
    assert!(events.contains(&EngineEvent::Gone {
        holder: Holder::PlayerMain(P)
    }));
    assert!(events.contains(&EngineEvent::Gone {
        holder: Holder::PlayerCue(P)
    }));
}

#[test]
fn the_engine_and_the_model_agree_on_a_devices_settings() {
    let edits: [fn(&mut Config); 4] = [
        |_| {},
        |c| c.outputs.set_device_rate(&out("main"), Some(96_000)),
        |c| {
            c.outputs.bit_perfect = vec![out("main")];
            c.outputs.dsd_output = vec![DsdDevice {
                backend: "offline".into(),
                device: "main".into(),
                mode: DsdOutput::Dop,
            }];
            c.outputs.dsd_mix = DsdMix::HoldOthers;
        },
        // A device no route names opens at the global values.
        |c| c.outputs.set_device_buffer(&out("other"), Some(256)),
    ];
    for edit in edits {
        let mut c = config();
        edit(&mut c);
        let settings = EngineSettings::from_config(&c);
        for d in [out("main"), out("other")] {
            assert_eq!(
                settings.device_settings(&BusKey::from(&d)),
                device_settings(&c.outputs, &d),
                "{d:?}"
            );
        }
    }
}

#[test]
fn an_unusable_configured_audio_system_falls_back_and_says_so() {
    let mut c = config();
    c.outputs.backend = Some("missing".into());
    let mut r = rig_with(&c);
    assert_eq!(r.engine.backend_in_use(), "offline");
    r.tick();
    assert!(r.take_events().contains(&EngineEvent::AudioSystemInUse {
        configured: Some("missing".into()),
        in_use: "offline".into(),
    }));
}

#[test]
fn l10_new_settings_change_no_open_bus_and_a_device_first_used_takes_them() {
    let mut r = rig();
    let q = PlayerId(2);
    let mut c = config();
    c.outputs.buffer_frames = 960;
    c.outputs.routes.push(PlayerRoutes {
        player: q,
        main: Some(route("other")),
        cue: None,
    });
    r.act(EngineAction::UpdateSettings(Box::new(c)));
    r.tick();
    assert_eq!(
        r.main.config().unwrap().buffer_frames,
        480,
        "open: unchanged"
    );
    r.act(EngineAction::AddPlayer { player: q });
    assert_eq!(r.other.config().unwrap().buffer_frames, 960, "L13");
}

#[test]
fn a_new_gain_smoothing_reaches_the_open_mixer() {
    let mut r = rig();
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, 0.0),
    });
    r.settle();
    r.run(5);
    let mut c = config();
    c.tuning.gain_smoothing_ms = 10.0; // 480 frames at 48 kHz
    r.act(EngineAction::UpdateSettings(Box::new(c)));
    r.run(1);
    r.act(EngineAction::SetVolume {
        player: P,
        volume: 0.0,
    });
    let heard = r.run(3);
    let audible = heard.iter().filter(|v| **v != 0.0).count();
    assert!(
        audible > 400,
        "the volume moved over 10 ms, not at once: {audible} frames"
    );
}

#[test]
fn an_open_bus_keeps_its_running_bit_perfect_setting() {
    let mut r = rig();
    r.main.set_exclusive_capable(true);
    let mut c = config();
    c.outputs.bit_perfect = vec![out("main")];
    r.act(EngineAction::UpdateSettings(Box::new(c)));
    let mut request = request(1, 0.0);
    request.format = Some(AudioFormat {
        sample_rate: 44_100,
        bits: Some(16),
        channels: 2,
        dsd_rate: None,
    });
    r.act(EngineAction::StartCurrent { player: P, request });
    r.settle();
    assert_eq!(
        r.main.config().unwrap().sample_rate,
        48_000,
        "not bit-perfect until ApplyDevice: the file is resampled"
    );
    assert!(!r.engine.running_settings(&out("main")).unwrap().bit_perfect);
}

/// A tuning update and a rate change in the same tick: the `Tune` is
/// computed for the rate the bus has when it is sent (48 kHz) and may still
/// wait in the command queue when the bit-perfect bus reopens at the file's
/// rate (44.1 kHz). The lengths must end up in 44.1 kHz frames.
#[test]
fn a_tuning_update_and_a_rate_change_in_one_tick_size_the_mixer_for_the_new_rate() {
    let mut c = config();
    c.outputs.bit_perfect = vec![out("main")];
    let mut r = rig_with(&c);
    r.main.set_exclusive_capable(true);
    let mut tuned = c.clone();
    tuned.tuning.gain_smoothing_ms = 10.0; // 480 frames at 48 kHz, 441 at 44.1
    r.act(EngineAction::UpdateSettings(Box::new(tuned)));
    let mut request = request(1, 0.0);
    request.format = Some(AudioFormat {
        sample_rate: 44_100,
        bits: Some(16),
        channels: 2,
        dsd_rate: None,
    });
    r.act(EngineAction::StartCurrent { player: P, request });
    r.settle();
    assert_eq!(
        r.main.config().unwrap().sample_rate,
        44_100,
        "the bit-perfect bus followed the file"
    );
    r.run(5);
    r.act(EngineAction::SetVolume {
        player: P,
        volume: 0.0,
    });
    let heard = r.run(3);
    let audible = heard.iter().filter(|v| **v != 0.0).count();
    assert!(
        (430..=450).contains(&audible),
        "10 ms at 44.1 kHz is 441 frames: {audible}"
    );
}

fn settings_of(r: &Rig, device: &str) -> DeviceSettings {
    r.engine.running_settings(&out(device)).unwrap()
}

/// The outcome the engine reported for `device`, if any.
fn applied<'a>(events: &'a [EngineEvent], device: &str) -> Option<&'a Result<(), String>> {
    events.iter().find_map(|e| match e {
        EngineEvent::Applied {
            target: Target::Device(d),
            outcome,
            ..
        } if *d == out(device) => Some(outcome),
        _ => None,
    })
}

fn apply_device(r: &mut Rig, device: &str, settings: DeviceSettings, force: bool) {
    r.act(EngineAction::ApplyDevice {
        device: out(device),
        settings,
        force,
    });
    // Events reach the test with the next tick.
    r.tick();
}

#[test]
fn l11_a_device_change_waits_for_a_fade_tail_then_runs() {
    let mut r = rig();
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, 0.0),
    });
    r.settle();
    r.run(3);
    r.act(EngineAction::FadeOutAndStop {
        player: P,
        fade_ms: 200,
    });
    let wanted = DeviceSettings {
        buffer_frames: 960,
        ..settings_of(&r, "main")
    };
    apply_device(&mut r, "main", wanted, false);
    r.run(2);
    assert_eq!(
        r.main.config().unwrap().buffer_frames,
        480,
        "the fade sounds"
    );
    assert!(applied(&r.events, "main").is_none());
    r.run(30);
    assert_eq!(r.main.config().unwrap().buffer_frames, 960);
    assert_eq!(applied(&r.events, "main"), Some(&Ok(())));
    assert_eq!(settings_of(&r, "main"), wanted);
}

#[test]
fn l11_a_device_change_waits_for_a_test_tone() {
    let mut r = rig();
    r.engine
        .play_test_tone(&route("main"), 440.0, 0.1, -18.0, r.clock);
    let wanted = DeviceSettings {
        buffer_frames: 960,
        ..settings_of(&r, "main")
    };
    apply_device(&mut r, "main", wanted, false);
    r.run(2);
    assert_eq!(r.main.config().unwrap().buffer_frames, 480);
    r.run(20);
    assert_eq!(r.main.config().unwrap().buffer_frames, 960);
}

#[test]
fn l12_a_new_rate_reopens_the_device_and_a_paused_track_stays_paused_where_it_was() {
    let mut r = rig();
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, 2.0),
    });
    r.settle();
    r.run(2);
    r.act(EngineAction::Pause { player: P });
    // The pause ramp is still running: the bus is not quiet yet.
    let wanted = DeviceSettings {
        sample_rate: 44_100,
        ..settings_of(&r, "main")
    };
    apply_device(&mut r, "main", wanted, false);
    assert_eq!(r.main.config().unwrap().sample_rate, 48_000);
    // The ramp (480 frames) and one block of margin.
    r.run(3);
    assert_eq!(r.main.config().unwrap().sample_rate, 44_100);
    let position = r.engine.telemetry(P).position_secs.unwrap();
    r.settle();
    let heard = r.run(5);
    assert!(heard.iter().all(|v| *v == 0.0), "nothing paused starts");
    let after = r.engine.telemetry(P).position_secs.unwrap();
    assert!((after - position).abs() < 0.001, "{position} then {after}");
    assert_eq!(applied(&r.events, "main"), Some(&Ok(())));
}

#[test]
fn l12_a_new_buffer_reopens_at_the_running_rate() {
    let mut r = rig();
    let wanted = DeviceSettings {
        buffer_frames: 960,
        ..settings_of(&r, "main")
    };
    apply_device(&mut r, "main", wanted, false);
    let config = r.main.config().unwrap();
    assert_eq!((config.sample_rate, config.buffer_frames), (48_000, 960));
}

#[test]
fn l12_a_bit_perfect_device_takes_a_new_rate_without_reopening() {
    let mut c = config();
    c.outputs.bit_perfect = vec![out("main")];
    let mut r = rig_with(&c);
    let attempts = r.main.open_attempts();
    let wanted = DeviceSettings {
        sample_rate: 96_000,
        ..settings_of(&r, "main")
    };
    apply_device(&mut r, "main", wanted, false);
    assert_eq!(
        r.main.open_attempts(),
        attempts,
        "the next file sets the rate"
    );
    assert_eq!(settings_of(&r, "main").sample_rate, 96_000);
    assert_eq!(applied(&r.events, "main"), Some(&Ok(())));
}

#[test]
fn l12_bit_perfect_on_reopens_exclusive_and_a_refusal_plays_shared() {
    let mut r = rig();
    r.main.set_exclusive_capable(true);
    let shared = settings_of(&r, "main");
    let exclusive = DeviceSettings {
        bit_perfect: true,
        ..shared
    };
    apply_device(&mut r, "main", exclusive, false);
    assert!(r.main.config().unwrap().exclusive);
    apply_device(&mut r, "main", shared, false);
    assert!(!r.main.config().unwrap().exclusive);
    r.main.set_exclusive_capable(false);
    apply_device(&mut r, "main", exclusive, false);
    assert!(!r.main.config().unwrap().exclusive, "B4: plays shared");
    assert!(settings_of(&r, "main").bit_perfect);
}

#[test]
fn l12_dsd_mix_and_silence_are_stored_without_a_reopen() {
    let mut c = config();
    c.outputs.bit_perfect = vec![out("main")];
    c.outputs.dsd_output = vec![DsdDevice {
        backend: "offline".into(),
        device: "main".into(),
        mode: DsdOutput::Dop,
    }];
    let mut r = rig_with(&c);
    let attempts = r.main.open_attempts();
    let wanted = DeviceSettings {
        dsd_mix: Some(DsdMix::HoldOthers),
        dsd_silence_ms: Some(400.0),
        ..settings_of(&r, "main")
    };
    apply_device(&mut r, "main", wanted, false);
    assert_eq!(r.main.open_attempts(), attempts);
    assert_eq!(settings_of(&r, "main"), wanted);
}

#[test]
fn a_change_for_a_device_not_open_is_done_at_once() {
    let mut r = rig();
    let wanted = DeviceSettings {
        buffer_frames: 960,
        ..device_settings(&config().outputs, &out("other"))
    };
    apply_device(&mut r, "other", wanted, false);
    assert_eq!(applied(&r.events, "other"), Some(&Ok(())));
    assert!(!r.other.is_open(), "L13: it opens when first used");
}

#[test]
fn l14_a_refused_rate_keeps_the_running_one_and_says_why() {
    let mut r = rig();
    r.main.refuse_rate(44_100);
    let running = settings_of(&r, "main");
    let wanted = DeviceSettings {
        sample_rate: 44_100,
        ..running
    };
    apply_device(&mut r, "main", wanted, false);
    assert_eq!(r.main.config().unwrap().sample_rate, 48_000);
    assert!(matches!(applied(&r.events, "main"), Some(Err(_))));
    assert_eq!(settings_of(&r, "main"), running);
}

#[test]
fn l14_a_device_busy_beyond_the_budget_keeps_the_running_settings() {
    let mut c = config();
    c.tuning.device_busy_retry_ms = 0.0;
    let mut r = rig_with(&c);
    // The first open and the three retries meet a busy device; the
    // restore of the running settings opens.
    r.main.set_busy(4);
    let wanted = DeviceSettings {
        buffer_frames: 960,
        ..settings_of(&r, "main")
    };
    apply_device(&mut r, "main", wanted, false);
    assert_eq!(r.main.config().unwrap().buffer_frames, 480);
    assert!(matches!(applied(&r.events, "main"), Some(Err(_))));
}

#[test]
fn l14_apply_now_asks_a_device_again_for_a_rate_it_refused() {
    let mut r = rig();
    r.main.refuse_rate(44_100);
    let wanted = DeviceSettings {
        sample_rate: 44_100,
        ..settings_of(&r, "main")
    };
    apply_device(&mut r, "main", wanted, false);
    let attempts = r.main.open_attempts();
    apply_device(&mut r, "main", wanted, true);
    assert!(
        r.main.open_attempts() > attempts,
        "asked again, not from memory"
    );
}

#[test]
fn l15_an_absent_device_takes_the_new_settings_when_it_returns() {
    let mut r = rig();
    r.main.unplug();
    r.run(1);
    let wanted = DeviceSettings {
        buffer_frames: 960,
        ..settings_of(&r, "main")
    };
    apply_device(&mut r, "main", wanted, false);
    assert_eq!(applied(&r.events, "main"), Some(&Ok(())));
    assert_eq!(settings_of(&r, "main"), wanted);
    r.main.replug();
    r.clock += Duration::from_secs(3);
    r.tick();
    assert_eq!(r.main.config().unwrap().buffer_frames, 960);
}

#[test]
fn l15_a_device_gone_during_the_change_keeps_the_new_settings() {
    let mut r = rig();
    r.main.unplug();
    let wanted = DeviceSettings {
        buffer_frames: 960,
        ..settings_of(&r, "main")
    };
    apply_device(&mut r, "main", wanted, false);
    assert_eq!(applied(&r.events, "main"), Some(&Ok(())));
    r.main.replug();
    r.clock += Duration::from_secs(3);
    r.tick();
    assert_eq!(r.main.config().unwrap().buffer_frames, 960);
}

fn route_applied(events: &[EngineEvent], holder: Holder) -> bool {
    events.iter().any(|e| {
        matches!(e, EngineEvent::Applied { target: Target::Route(h), outcome: Ok(()), .. } if *h == holder)
    })
}

#[test]
fn l16_a_moved_player_keeps_its_paused_track_paused_on_the_new_device() {
    let mut r = rig();
    r.act(EngineAction::LoadPaused {
        player: P,
        request: request(1, 2.0),
    });
    r.settle();
    r.take_events();
    r.act(EngineAction::ApplyRoute {
        holder: Holder::PlayerMain(P),
        route: Some(route("other")),
        force: false,
    });
    r.settle();
    let events = r.take_events();
    assert!(events.iter().any(|e| matches!(
        e,
        EngineEvent::Placed { holder: Holder::PlayerMain(p), device, .. }
            if *p == P && *device == out("other")
    )));
    assert!(route_applied(&events, Holder::PlayerMain(P)));
    r.run(3);
    let position = r.engine.telemetry(P).position_secs.unwrap();
    assert!((position - 2.0).abs() < 0.01, "{position}");
    assert!(r.other.is_open());
    assert!(!r.main.is_open(), "L13: closed once nothing uses it");
}

#[test]
fn l16_a_route_change_waits_for_its_own_holder_only() {
    let mut c = config();
    c.outputs.routes.push(PlayerRoutes {
        player: Q,
        main: Some(route("main")),
        cue: None,
    });
    let mut r = rig_with(&c);
    r.act(EngineAction::AddPlayer { player: Q });
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, 0.0),
    });
    r.settle();
    r.run(2);
    r.act(EngineAction::ApplyRoute {
        holder: Holder::PlayerMain(Q),
        route: Some(route("other")),
        force: false,
    });
    r.tick();
    assert!(route_applied(&r.events, Holder::PlayerMain(Q)), "Q is idle");
    let heard = r.run(3);
    assert!(heard.iter().all(|v| *v != 0.0), "P plays on, uninterrupted");
    assert!(r.main.is_open(), "P still uses it");
    r.act(EngineAction::ApplyRoute {
        holder: Holder::PlayerMain(P),
        route: Some(route("other")),
        force: false,
    });
    r.run(2);
    assert!(!route_applied(&r.events, Holder::PlayerMain(P)), "P sounds");
    r.act(EngineAction::StopNow { player: P });
    r.run(5);
    assert!(route_applied(&r.events, Holder::PlayerMain(P)));
}

#[test]
fn l16_removing_a_cue_route_by_force_ends_the_open_cue() {
    let mut c = config();
    c.outputs.routes[0].cue = Some(route("other"));
    let mut r = rig_with(&c);
    r.act(EngineAction::StartCue {
        player: P,
        request: request(2, 0.0),
    });
    r.settle();
    r.run(2);
    r.act(EngineAction::ApplyRoute {
        holder: Holder::PlayerCue(P),
        route: None,
        force: false,
    });
    r.run(2);
    assert!(
        !route_applied(&r.events, Holder::PlayerCue(P)),
        "the CUE sounds"
    );
    r.act(EngineAction::ApplyRoute {
        holder: Holder::PlayerCue(P),
        route: None,
        force: true,
    });
    r.tick();
    assert!(r.events.contains(&EngineEvent::CueEnded {
        player: P,
        entry: EntryId(2)
    }));
    assert!(r.events.contains(&EngineEvent::Unplaced {
        holder: Holder::PlayerCue(P),
        route: None
    }));
}

#[test]
fn l16_the_cartwall_before_its_first_cart_only_takes_the_route() {
    let mut r = rig();
    r.take_events();
    r.act(EngineAction::ApplyRoute {
        holder: Holder::CartwallMain,
        route: Some(route("other")),
        force: false,
    });
    r.tick();
    let events = r.take_events();
    assert!(events.contains(&EngineEvent::Unplaced {
        holder: Holder::CartwallMain,
        route: Some(route("other"))
    }));
    assert!(route_applied(&events, Holder::CartwallMain));
    assert!(!r.other.is_open());
}

#[test]
fn l16_a_route_to_channels_the_open_device_lacks_reopens_it_with_more() {
    let mut c = config();
    c.outputs.routes[0].main = Some(route("other"));
    let mut r = rig_with(&c);
    assert_eq!(r.other.config().unwrap().channels, 2);
    let pair = Route {
        first_channel: 2,
        ..route("other")
    };
    c.outputs.routes[0].cue = Some(pair.clone());
    r.act(EngineAction::UpdateSettings(Box::new(c)));
    r.act(EngineAction::ApplyRoute {
        holder: Holder::PlayerCue(P),
        route: Some(pair),
        force: false,
    });
    r.tick();
    assert_eq!(r.other.config().unwrap().channels, 4);
    assert!(route_applied(&r.events, Holder::PlayerCue(P)));
}

/// L3, L13: the cartwall has no bus before its first cart, so its holder is
/// `Unplaced` with the route the engine holds. A new cartwall route must be
/// reported at once: the model would show a false pending change otherwise.
#[test]
fn l3_a_cartwall_route_changed_before_its_first_cart_is_reported_unplaced_with_it() {
    let mut r = rig();
    r.tick();
    r.take_events();
    let mut c = config();
    c.outputs.cartwall.main = Some(route("other"));
    r.act(EngineAction::UpdateSettings(Box::new(c)));
    r.tick();
    let events = r.take_events();
    assert!(events.contains(&EngineEvent::Unplaced {
        holder: Holder::CartwallMain,
        route: Some(route("other")),
    }));
    assert!(events.contains(&EngineEvent::Unplaced {
        holder: Holder::CartwallCue,
        route: None,
    }));
    assert!(!r.other.is_open(), "nothing opens before the first cart");
    // The same routes again, or any other setting: nothing to report.
    let mut c = config();
    c.outputs.cartwall.main = Some(route("other"));
    c.outputs.buffer_frames = 960;
    r.act(EngineAction::UpdateSettings(Box::new(c)));
    r.tick();
    assert!(
        !r.take_events()
            .iter()
            .any(|e| matches!(e, EngineEvent::Unplaced { .. })),
        "unchanged routes are not reported again"
    );
}
