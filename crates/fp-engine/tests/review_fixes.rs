#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Regressions found by the whole-branch review of Phase 1 plan 2.

mod support;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use fp_backends::{AudioBackend, OfflineBackend, OfflineDevice};
use fp_engine::engine::{Engine, EngineSettings};
use fp_model::{
    Config, EngineAction, EngineEvent, EntryId, PlayerId, PlayerRoutes, Route, SourceRequest,
    TrackId, TransitionPlan,
};
use support::tagged_opener;

const BLOCK: usize = 480;
const P: PlayerId = PlayerId(1);

fn request(name: &str, n: u64, from_secs: f64) -> SourceRequest {
    SourceRequest {
        entry: EntryId(n),
        track: TrackId(n),
        path: PathBuf::from(format!("{name}{n}")),
        from_secs,
    }
}

fn track(n: u64) -> SourceRequest {
    request("track", n, 0.0)
}

fn route(device: &str, first_channel: u16) -> Route {
    Route {
        backend: "offline".into(),
        device: device.into(),
        first_channel,
    }
}

struct Rig {
    engine: Engine,
    device: OfflineDevice,
    channels: usize,
    clock: Instant,
    frames: Vec<Vec<f32>>,
    events: Vec<EngineEvent>,
}

fn rig(channels: u16, main: Option<Route>, cue: Option<Route>, track_frames: u64) -> Rig {
    let backend = OfflineBackend::new();
    let device = backend.add_device("card", channels);
    let mut config = Config::default();
    config.outputs.buffer_frames = BLOCK as u32;
    config.outputs.backend = Some("offline".into());
    config.outputs.routes = vec![PlayerRoutes {
        player: P,
        main,
        cue,
    }];
    config.tuning.gain_smoothing_ms = 0.0;
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
    let mut engine = Engine::new(
        backends,
        EngineSettings::from_config(&config),
        tagged_opener(track_frames),
    );
    let clock = Instant::now();
    engine.execute(EngineAction::AddPlayer { player: P }, clock);
    Rig {
        engine,
        device,
        channels: usize::from(channels),
        clock,
        frames: Vec::new(),
        events: Vec::new(),
    }
}

impl Rig {
    fn act(&mut self, action: EngineAction) {
        self.engine.execute(action, self.clock);
    }

    fn settle(&mut self) {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let events = self.engine.tick(self.clock);
            self.events.extend(events);
            if self.engine.unsettled_sources() == 0 || Instant::now() > deadline {
                return;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    fn render_only(&mut self, blocks: usize) {
        for _ in 0..blocks {
            let out = self.device.render(BLOCK).unwrap();
            self.frames
                .extend(out.chunks(self.channels).map(<[f32]>::to_vec));
        }
    }

    fn run(&mut self, blocks: usize) {
        for _ in 0..blocks {
            self.render_only(1);
            self.clock += Duration::from_millis(10);
            let events = self.engine.tick(self.clock);
            self.events.extend(events);
        }
    }

    fn channel(&self, c: usize) -> Vec<f32> {
        self.frames.iter().map(|f| f[c]).collect()
    }
}

fn is_failure(e: &EngineEvent) -> bool {
    matches!(e, EngineEvent::SourceFailed { .. })
}

// C1 — Cue on the second channel pair of the Main device.
#[test]
fn cue_on_the_second_pair_of_the_main_device_never_reaches_main() {
    let mut r = rig(4, Some(route("card", 0)), Some(route("card", 2)), 48_000);
    r.act(EngineAction::StartCue {
        player: P,
        request: track(3),
    });
    r.settle();
    r.run(10);
    assert!(
        r.channel(0)
            .iter()
            .chain(r.channel(1).iter())
            .all(|v| *v == 0.0),
        "cue leaked to Main"
    );
    assert!(
        r.channel(2).iter().any(|v| *v != 0.0),
        "cue must play on channels 3/4"
    );
}

// C2 — Pause then Stop, repeatedly.
#[test]
fn pause_then_stop_never_leaks_mixer_slots() {
    let mut r = rig(2, Some(route("card", 0)), None, 48_000);
    for _ in 0..60 {
        r.act(EngineAction::StartCurrent {
            player: P,
            request: track(1),
        });
        r.settle();
        r.run(1);
        r.act(EngineAction::Pause { player: P });
        r.run(2);
        r.act(EngineAction::StopNow { player: P });
        r.run(2);
    }
    r.act(EngineAction::StartCurrent {
        player: P,
        request: track(2),
    });
    r.settle();
    r.run(3);
    assert!(
        !r.events.iter().any(is_failure),
        "good files must never be reported as failed"
    );
    assert!(r.channel(0).iter().rev().take(BLOCK).any(|v| *v != 0.0));
}

// C3 — a route to a backend this machine does not have.
#[test]
fn a_route_to_an_absent_backend_uses_the_default_backend_and_device() {
    let backend = OfflineBackend::new();
    let main = backend.add_device("main", 2);
    let mut config = Config::default();
    config.outputs.buffer_frames = BLOCK as u32;
    config.outputs.routes = vec![PlayerRoutes {
        player: P,
        main: Some(Route {
            backend: "asio".into(),
            device: "ASIO4ALL v2".into(),
            first_channel: 0,
        }),
        cue: None,
    }];
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
    let mut engine = Engine::new(
        backends,
        EngineSettings::from_config(&config),
        tagged_opener(48_000),
    );
    let now = Instant::now();
    engine.execute(EngineAction::AddPlayer { player: P }, now);
    engine.execute(
        EngineAction::StartCurrent {
            player: P,
            request: track(1),
        },
        now,
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    while engine.unsettled_sources() > 0 && Instant::now() < deadline {
        engine.tick(now);
        std::thread::sleep(Duration::from_millis(1));
    }
    engine.tick(now);
    let out = main.render(BLOCK).unwrap();
    assert!(
        out.iter().any(|v| *v != 0.0),
        "the player must play on the default device"
    );
}

// I1 — crossfade from a current that never started.
#[test]
fn a_crossfade_from_a_source_that_never_started_completes_the_fade() {
    let mut r = rig(2, Some(route("card", 0)), None, 48_000);
    r.act(EngineAction::StartCurrent {
        player: P,
        request: track(1),
    });
    r.act(EngineAction::Crossfade {
        player: P,
        request: track(2),
        fade_ms: 100,
    });
    r.settle();
    r.run(30);
    assert!(r.events.contains(&EngineEvent::FadeCompleted { player: P }));
}

// I2 — a pause arriving after the mixer already executed an overlapping segue.
#[test]
fn a_pause_after_an_executed_overlap_keeps_playing_and_completes_the_fade() {
    let mut r = rig(2, Some(route("card", 0)), None, 96_000);
    r.act(EngineAction::StartCurrent {
        player: P,
        request: track(1),
    });
    r.act(EngineAction::Preload {
        player: P,
        request: Some(track(2)),
    });
    r.settle();
    r.run(1);
    r.act(EngineAction::Schedule {
        player: P,
        plan: Some(TransitionPlan::StartNextAt {
            at_secs: 0.2,
            fade_current_until_secs: Some(0.4),
        }),
    });
    r.run(12); // dispatched
    r.render_only(12); // the mixer passes the segue frame before the conductor ticks
    r.act(EngineAction::Pause { player: P });
    r.run(40);
    let started = r.events.iter().position(|e| {
        *e == EngineEvent::TransitionStarted {
            player: P,
            entry: EntryId(2),
        }
    });
    let faded = r
        .events
        .iter()
        .position(|e| *e == EngineEvent::FadeCompleted { player: P });
    assert!(
        started.is_some() && faded.is_some() && started < faded,
        "events {:?}",
        r.events
    );
    assert!(
        r.channel(0)
            .iter()
            .rev()
            .take(BLOCK)
            .all(|v| (*v as u64) / 100_000 == 2),
        "the next keeps playing"
    );
}

// I3 — a source failing mid-play drains what is buffered first.
#[test]
fn a_failing_current_source_plays_its_buffer_before_reporting() {
    let mut r = rig(2, Some(route("card", 0)), None, 9_600);
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request("broken", 1, 0.0),
    });
    r.settle();
    r.run(30);
    let heard = r.channel(0);
    assert!(
        heard.contains(&109_599.0),
        "the last buffered frame must be heard"
    );
    assert!(r.events.contains(&EngineEvent::SourceFailed {
        player: P,
        entry: EntryId(1)
    }));
}

// I5 — running out of mixer slots is not a file failure.
#[test]
fn running_out_of_mixer_slots_is_never_reported_as_a_file_failure() {
    let mut r = rig(2, Some(route("card", 0)), None, 480_000);
    r.act(EngineAction::StartCurrent {
        player: P,
        request: track(1),
    });
    r.settle();
    r.run(1);
    for n in 2..40 {
        r.act(EngineAction::Crossfade {
            player: P,
            request: track(n % 4 + 1),
            fade_ms: 10_000,
        });
        r.settle();
        r.run(1);
    }
    assert!(
        !r.events.iter().any(is_failure),
        "slot exhaustion must not mark files unreadable"
    );
    assert!(r.engine.slot_exhaustions() > 0);
}
