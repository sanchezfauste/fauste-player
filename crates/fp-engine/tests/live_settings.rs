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
    CartId, CartRequest, Config, DsdDevice, DsdMix, DsdOutput, EngineAction, EngineEvent, EntryId,
    Holder, OutputDevice, PlayerId, PlayerRoutes, Route, SOURCE_END, SourceRequest, TrackId,
    device_settings,
};
use support::tagged_opener;

const BLOCK: usize = 480;
const P: PlayerId = PlayerId(1);

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
