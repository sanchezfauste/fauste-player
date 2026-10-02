#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The engine executes model actions with sample accuracy (spec §4.3–4.4).

mod support;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use fp_backends::{AudioBackend, OfflineBackend, OfflineDevice};
use fp_engine::engine::{Engine, EngineSettings};
use fp_model::{
    Config, EngineAction, EngineEvent, EntryId, PlayerId, PlayerRoutes, Route, SOURCE_END,
    SourceRequest, TrackId, TransitionPlan,
};
use support::{gated_opener, tagged_opener};

const RATE: f64 = 48_000.0;
const BLOCK: usize = 480;
const P: PlayerId = PlayerId(1);

struct Rig {
    engine: Engine,
    main: OfflineDevice,
    cue: OfflineDevice,
    clock: Instant,
    /// Everything rendered on the main device so far (left channel).
    heard: Vec<f32>,
    events: Vec<EngineEvent>,
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

fn rig(track_frames: u64, with_cue_route: bool) -> Rig {
    rig_with(tagged_opener(track_frames), with_cue_route)
}

fn rig_with(opener: fp_engine::worker::SourceOpener, with_cue_route: bool) -> Rig {
    let backend = OfflineBackend::new();
    let main = backend.add_device("main", 2);
    let cue = backend.add_device("cue", 2);
    let mut config = Config::default();
    config.outputs.buffer_frames = BLOCK as u32;
    config.outputs.backend = Some("offline".into());
    config.outputs.routes = vec![PlayerRoutes {
        player: P,
        main: Some(Route {
            backend: "offline".into(),
            device: "main".into(),
            first_channel: 0,
        }),
        cue: with_cue_route.then(|| Route {
            backend: "offline".into(),
            device: "cue".into(),
            first_channel: 0,
        }),
    }];
    config.tuning.gain_smoothing_ms = 0.0;
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
    let mut engine = Engine::new(backends, EngineSettings::from_config(&config), opener);
    let clock = Instant::now();
    engine.execute(EngineAction::AddPlayer { player: P }, clock);
    Rig {
        engine,
        main,
        cue,
        clock,
        heard: Vec::new(),
        events: Vec::new(),
    }
}

impl Rig {
    fn act(&mut self, action: EngineAction) {
        self.engine.execute(action, self.clock);
    }

    /// Waits (bounded) until worker threads have filled every pending
    /// source, ticking meanwhile; no fixed sleeps, so slow CI machines pass.
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

    /// Renders `blocks` blocks on both devices, ticking after each.
    fn run(&mut self, blocks: usize) {
        for _ in 0..blocks {
            let out = self.main.render(BLOCK).unwrap();
            self.heard.extend(out.chunks(2).map(|f| f[0]));
            let _ = self.cue.render(BLOCK);
            self.clock += Duration::from_secs_f64(BLOCK as f64 / RATE);
            let events = self.engine.tick(self.clock);
            self.events.extend(events);
        }
    }

    fn position(&self) -> f64 {
        self.engine.telemetry(P).position_secs.unwrap()
    }
}

fn tag(v: f32) -> (u64, u64) {
    let v = v as u64;
    (v / 100_000, v % 100_000)
}

#[test]
fn start_current_plays_the_track_from_its_first_frame_once_ready() {
    let mut r = rig(96_000, false);
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, 0.0),
    });
    r.settle();
    r.run(2);
    let first = r.heard.iter().position(|v| *v != 0.0).unwrap();
    assert_eq!(tag(r.heard[first]), (1, 0));
    assert_eq!(tag(r.heard[first + 100]), (1, 100));
}

#[test]
fn a_planned_stop_ends_on_the_exact_frame_and_reports_the_entry() {
    let mut r = rig(96_000, false);
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, 0.0),
    });
    r.settle();
    r.run(1);
    let start = r.heard.iter().position(|v| *v != 0.0).unwrap();
    r.act(EngineAction::Schedule {
        player: P,
        plan: Some(TransitionPlan::StopAt { at_secs: 0.5 }),
    });
    r.run(80);
    let stop = start + 24_000;
    assert_eq!(
        tag(r.heard[stop - 241]),
        (1, 23_759),
        "full level before the de-click ramp"
    );
    assert!(
        r.heard[stop..].iter().all(|v| *v == 0.0),
        "silent from the stop frame on"
    );
    assert!(r.events.contains(&EngineEvent::ReachedEnd {
        player: P,
        entry: EntryId(1)
    }));
}

#[test]
fn a_segue_starts_the_next_on_the_exact_frame_and_overlaps_the_fade() {
    let mut r = rig(96_000, false);
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, 0.0),
    });
    r.act(EngineAction::Preload {
        player: P,
        request: Some(request(2, 0.0)),
    });
    r.settle();
    r.run(1);
    let start = r.heard.iter().position(|v| *v != 0.0).unwrap();
    r.act(EngineAction::Schedule {
        player: P,
        plan: Some(TransitionPlan::StartNextAt {
            at_secs: 0.5,
            fade_current_until_secs: Some(0.6),
        }),
    });
    r.run(80);
    let at = start + 24_000;
    assert_eq!(
        r.heard[at - 1],
        123_999.0,
        "only the current before the segue"
    );
    assert_eq!(
        r.heard[at],
        124_000.0 + 200_000.0,
        "the next joins at full level on the exact frame"
    );
    assert_eq!(
        tag(r.heard[at + 4_800 + 10]),
        (2, 4_810),
        "after the fade only the next remains"
    );
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
    assert!(started.unwrap() < faded.unwrap());
}

#[test]
fn source_end_plans_chain_when_the_file_runs_out() {
    let mut r = rig(12_000, false);
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, 0.0),
    });
    r.act(EngineAction::Preload {
        player: P,
        request: Some(request(2, 0.0)),
    });
    r.act(EngineAction::Schedule {
        player: P,
        plan: Some(TransitionPlan::StartNextAt {
            at_secs: SOURCE_END,
            fade_current_until_secs: None,
        }),
    });
    r.settle();
    r.run(40);
    assert!(r.events.contains(&EngineEvent::TransitionStarted {
        player: P,
        entry: EntryId(2)
    }));
    let last_of_1 = r.heard.iter().rposition(|v| tag(*v).0 == 1).unwrap();
    let first_of_2 = r.heard.iter().position(|v| tag(*v) == (2, 0)).unwrap();
    assert!(first_of_2 > last_of_1);
    assert!(
        first_of_2 - last_of_1 <= 2 * BLOCK,
        "gap of {} frames",
        first_of_2 - last_of_1
    );
}

#[test]
fn a_crossfade_starts_the_next_now_and_reports_when_the_old_one_is_gone() {
    let mut r = rig(96_000, false);
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, 0.0),
    });
    r.act(EngineAction::Preload {
        player: P,
        request: Some(request(2, 0.0)),
    });
    r.settle();
    r.run(5);
    r.act(EngineAction::Crossfade {
        player: P,
        request: request(2, 0.0),
        fade_ms: 100,
    });
    r.settle();
    r.run(20);
    assert!(r.events.contains(&EngineEvent::FadeCompleted { player: P }));
    assert_eq!(tag(*r.heard.last().unwrap()).0, 2);
}

#[test]
fn fade_out_and_stop_reports_the_end_after_the_fade() {
    let mut r = rig(96_000, false);
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, 0.0),
    });
    r.settle();
    r.run(5);
    r.act(EngineAction::FadeOutAndStop {
        player: P,
        fade_ms: 100,
    });
    r.run(8);
    assert!(
        !r.events
            .iter()
            .any(|e| matches!(e, EngineEvent::ReachedEnd { .. })),
        "not before the fade ends"
    );
    r.run(4);
    assert!(r.events.contains(&EngineEvent::ReachedEnd {
        player: P,
        entry: EntryId(1)
    }));
    assert!(r.heard[r.heard.len() - BLOCK..].iter().all(|v| *v == 0.0));
}

#[test]
fn pause_holds_the_position_and_resume_continues_it() {
    let mut r = rig(96_000, false);
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, 0.0),
    });
    r.settle();
    r.run(10);
    r.act(EngineAction::Pause { player: P });
    r.run(2);
    let held = r.position();
    r.run(10);
    assert_eq!(r.position(), held);
    r.act(EngineAction::Resume { player: P });
    r.run(2);
    assert!(r.position() > held);
}

#[test]
fn load_paused_waits_for_resume_and_starts_at_the_saved_position() {
    let mut r = rig(96_000, false);
    r.act(EngineAction::LoadPaused {
        player: P,
        request: request(1, 1.0),
    });
    r.settle();
    r.run(4);
    assert!(
        r.heard.iter().all(|v| *v == 0.0),
        "nothing goes on air by itself"
    );
    assert_eq!(r.position(), 1.0);
    r.act(EngineAction::Resume { player: P });
    r.settle();
    r.run(4);
    // Resuming fades in from gain 0, so the start frame itself is silent.
    let start = r.heard.iter().position(|v| *v != 0.0).unwrap() - 1;
    assert_eq!(tag(r.heard[start + 300]).1, 48_000 + 300);
}

#[test]
fn seek_replaces_the_source_at_the_new_position() {
    let mut r = rig(96_000, false);
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, 0.0),
    });
    r.settle();
    r.run(2);
    r.act(EngineAction::Seek {
        player: P,
        secs: 1.5,
    });
    r.settle();
    r.run(4);
    assert!(r.position() >= 1.5);
    assert!((tag(*r.heard.last().unwrap()).1 as f64 / RATE) >= 1.5);
}

#[test]
fn an_unreadable_preload_is_reported_as_a_preload_failure() {
    let mut r = rig(96_000, false);
    let mut bad = request(2, 0.0);
    bad.path = PathBuf::from("corrupt.mp3");
    r.act(EngineAction::Preload {
        player: P,
        request: Some(bad),
    });
    // The worker reports the failure on its own thread: wait for a report.
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while !r.events.iter().any(|e| {
        matches!(
            e,
            EngineEvent::PreloadFailed { .. } | EngineEvent::SourceFailed { .. }
        )
    }) && std::time::Instant::now() < deadline
    {
        r.settle();
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(
        r.events.contains(&EngineEvent::PreloadFailed {
            player: P,
            entry: EntryId(2)
        }),
        "{:?}",
        r.events
    );
    assert!(
        !r.events
            .iter()
            .any(|e| matches!(e, EngineEvent::SourceFailed { .. }))
    );
}

#[test]
fn an_unreadable_file_is_reported_with_its_entry() {
    let mut r = rig(96_000, false);
    let mut bad = request(1, 0.0);
    bad.path = PathBuf::from("corrupt.mp3");
    r.act(EngineAction::StartCurrent {
        player: P,
        request: bad,
    });
    r.settle();
    assert!(r.events.contains(&EngineEvent::SourceFailed {
        player: P,
        entry: EntryId(1)
    }));
}

#[test]
fn cue_plays_only_on_the_cue_output_and_ends_by_itself() {
    let mut r = rig(4_800, true);
    r.act(EngineAction::StartCue {
        player: P,
        request: request(3, 0.0),
    });
    r.settle();
    let mut cue_heard = Vec::new();
    for _ in 0..20 {
        let out = r.cue.render(BLOCK).unwrap();
        cue_heard.extend(out.chunks(2).map(|f| f[0]));
        r.run(1);
    }
    assert!(cue_heard.iter().any(|v| tag(*v).0 == 3));
    assert!(
        r.heard.iter().all(|v| *v == 0.0),
        "the main output never hears the cue"
    );
    assert!(r.events.contains(&EngineEvent::CueEnded {
        player: P,
        entry: EntryId(3)
    }));
}

#[test]
fn cue_without_a_cue_output_ends_immediately() {
    let mut r = rig(4_800, false);
    r.act(EngineAction::StartCue {
        player: P,
        request: request(3, 0.0),
    });
    r.settle();
    assert!(r.events.contains(&EngineEvent::CueEnded {
        player: P,
        entry: EntryId(3)
    }));
}

#[test]
fn pausing_before_a_dispatched_transition_takes_it_back() {
    let mut r = rig(96_000, false);
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, 0.0),
    });
    r.act(EngineAction::Preload {
        player: P,
        request: Some(request(2, 0.0)),
    });
    r.settle();
    r.run(1);
    r.act(EngineAction::Schedule {
        player: P,
        plan: Some(TransitionPlan::StartNextAt {
            at_secs: 0.2,
            fade_current_until_secs: None,
        }),
    });
    r.run(12); // ~0.12 s: the transition is now dispatched (within the 200 ms lead).
    r.act(EngineAction::Pause { player: P });
    r.run(40);
    assert!(
        !r.heard.iter().any(|v| tag(*v).0 == 2),
        "the next must not start while paused"
    );
    r.act(EngineAction::Resume { player: P });
    r.run(40);
    assert!(r.events.contains(&EngineEvent::TransitionStarted {
        player: P,
        entry: EntryId(2)
    }));
}

#[test]
fn a_transition_that_already_happened_when_pause_arrives_leaves_the_player_playing() {
    let mut r = rig(96_000, false);
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, 0.0),
    });
    r.act(EngineAction::Preload {
        player: P,
        request: Some(request(2, 0.0)),
    });
    r.settle();
    r.run(1);
    r.act(EngineAction::Schedule {
        player: P,
        plan: Some(TransitionPlan::StartNextAt {
            at_secs: 0.2,
            fade_current_until_secs: None,
        }),
    });
    r.run(12); // dispatched
    // The audio thread passes the transition frame before the conductor ticks again…
    for _ in 0..12 {
        r.main.render(BLOCK).unwrap();
    }
    // …and the operator's pause is executed before the Started event is seen.
    r.act(EngineAction::Pause { player: P });
    r.run(1);
    assert!(r.events.contains(&EngineEvent::TransitionStarted {
        player: P,
        entry: EntryId(2)
    }));
    // The model now says "playing track 2": the engine must agree.
    r.act(EngineAction::Seek {
        player: P,
        secs: 0.5,
    });
    r.settle();
    r.run(10);
    assert!(
        r.position() > 0.5,
        "the seeked source must play, position {}",
        r.position()
    );
}

#[test]
fn stopping_a_source_that_never_started_frees_its_slot() {
    let mut r = rig(96_000, false);
    for n in 0..40 {
        // Started and stopped before its worker filled it: it never reaches the mixer's clock.
        r.act(EngineAction::StartCurrent {
            player: P,
            request: request(1 + (n % 3), 0.0),
        });
        r.act(EngineAction::StopNow { player: P });
        r.run(1);
    }
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, 0.0),
    });
    r.settle();
    r.run(4);
    assert!(
        r.heard.iter().rev().take(BLOCK).any(|v| *v != 0.0),
        "a slot must still be available"
    );
}

#[test]
fn a_file_shorter_than_its_planned_stop_reports_the_end_when_it_runs_out() {
    let mut r = rig(12_000, false);
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, 0.0),
    });
    r.act(EngineAction::Schedule {
        player: P,
        plan: Some(TransitionPlan::StopAt { at_secs: 10.0 }),
    });
    r.settle();
    r.run(40);
    assert!(r.events.contains(&EngineEvent::ReachedEnd {
        player: P,
        entry: EntryId(1)
    }));
}

#[test]
fn a_route_to_an_unknown_backend_falls_back_to_the_default_backend() {
    let backend = OfflineBackend::new();
    let main = backend.add_device("main", 2);
    let mut config = Config::default();
    config.outputs.buffer_frames = BLOCK as u32;
    config.outputs.routes = vec![PlayerRoutes {
        player: P,
        main: Some(Route {
            backend: "asio".into(),
            device: "main".into(),
            first_channel: 0,
        }),
        cue: None,
    }];
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
    let mut engine = Engine::new(
        backends,
        EngineSettings::from_config(&config),
        tagged_opener(96_000),
    );
    let now = Instant::now();
    engine.execute(EngineAction::AddPlayer { player: P }, now);
    engine.execute(
        EngineAction::StartCurrent {
            player: P,
            request: request(1, 0.0),
        },
        now,
    );
    std::thread::sleep(Duration::from_millis(20));
    engine.tick(now);
    let out = main.render(BLOCK).unwrap();
    assert!(
        out.iter().any(|v| *v != 0.0),
        "the player must not go silent"
    );
}

/// An audio system compiled in but not usable here (a JACK server that is
/// not running, a library that is missing).
struct Unavailable;

impl AudioBackend for Unavailable {
    fn id(&self) -> fp_backends::BackendId {
        fp_backends::BackendId("jack".into())
    }
    fn availability(&self) -> fp_backends::Availability {
        fp_backends::Availability::Unavailable("server not running".into())
    }
    fn enumerate_devices(&self) -> Result<Vec<fp_backends::DeviceInfo>, fp_backends::BackendError> {
        Err(fp_backends::BackendError::Unavailable(
            "server not running".into(),
        ))
    }
    fn default_device(&self) -> Option<fp_backends::DeviceId> {
        None
    }
    fn open_output(
        &self,
        _: &fp_backends::DeviceId,
        _: fp_backends::StreamConfig,
        _: Box<dyn fp_backends::Renderer>,
        _: Arc<dyn fp_backends::StreamErrorSink>,
    ) -> Result<Box<dyn fp_backends::OutputStream>, fp_backends::BackendError> {
        Err(fp_backends::BackendError::Unavailable(
            "server not running".into(),
        ))
    }
}

#[test]
fn a_route_to_an_unavailable_backend_falls_back_to_the_default_output() {
    let backend = OfflineBackend::new();
    let main = backend.add_device("main", 2);
    let mut config = Config::default();
    config.outputs.buffer_frames = BLOCK as u32;
    config.outputs.backend = Some("offline".into());
    config.outputs.routes = vec![PlayerRoutes {
        player: P,
        main: Some(Route {
            backend: "jack".into(),
            device: "system".into(),
            first_channel: 0,
        }),
        cue: None,
    }];
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(Unavailable), Arc::new(backend)];
    let mut engine = Engine::new(
        backends,
        EngineSettings::from_config(&config),
        tagged_opener(96_000),
    );
    let now = Instant::now();
    engine.execute(EngineAction::AddPlayer { player: P }, now);
    engine.execute(
        EngineAction::StartCurrent {
            player: P,
            request: request(1, 0.0),
        },
        now,
    );
    std::thread::sleep(Duration::from_millis(20));
    engine.tick(now);
    let out = main.render(BLOCK).unwrap();
    assert!(
        out.iter().any(|v| *v != 0.0),
        "a route to a system that is unavailable here must play on the default output"
    );
}

#[test]
fn a_fade_stop_on_the_exact_frame_of_a_transition_still_ends() {
    let mut r = rig(96_000, false);
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, 0.0),
    });
    r.act(EngineAction::Preload {
        player: P,
        request: Some(request(2, 0.0)),
    });
    r.settle();
    r.run(1);
    let start = r.heard.iter().position(|v| *v != 0.0).unwrap();
    r.act(EngineAction::Schedule {
        player: P,
        plan: Some(TransitionPlan::StartNextAt {
            at_secs: 0.5,
            fade_current_until_secs: None,
        }),
    });
    // Render up to the transition's frame exactly: it is the next frame the
    // device will play, not one already played.
    let at = start + 24_000;
    assert_eq!(at % BLOCK, 0, "the transition falls on a block boundary");
    r.run((at - r.heard.len()) / BLOCK);
    assert_eq!(r.heard.len(), at);
    r.act(EngineAction::FadeOutAndStop {
        player: P,
        fade_ms: 100,
    });
    r.run(100);
    assert!(
        r.events
            .iter()
            .any(|e| matches!(e, EngineEvent::ReachedEnd { player, .. } if *player == P)),
        "the fade stop ends: {:?}",
        r.events
    );
    assert!(
        r.heard[r.heard.len() - BLOCK..].iter().all(|v| *v == 0.0),
        "silent after the fade"
    );
    assert_eq!(
        r.engine.attached_sources(),
        1,
        "only the preloaded next is left, waiting"
    );
}

#[test]
fn a_transition_to_a_next_not_ready_yet_starts_it_when_ready_without_underruns() {
    let gate = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let mut r = rig_with(gated_opener(96_000, Arc::clone(&gate)), false);
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, 0.0),
    });
    r.settle();
    r.act(EngineAction::Preload {
        player: P,
        request: Some(SourceRequest {
            path: PathBuf::from("slow2"),
            ..request(2, 0.0)
        }),
    });
    r.run(1);
    r.act(EngineAction::Schedule {
        player: P,
        plan: Some(TransitionPlan::StartNextAt {
            at_secs: 0.1,
            fade_current_until_secs: None,
        }),
    });
    r.run(30); // past the transition, the next still opening
    gate.store(true, std::sync::atomic::Ordering::Release);
    r.settle();
    r.run(10);
    assert!(
        r.events.contains(&EngineEvent::TransitionStarted {
            player: P,
            entry: EntryId(2),
        }),
        "{:?}",
        r.events
    );
    let first = r.heard.iter().position(|v| tag(*v).0 == 2).unwrap();
    assert_eq!(
        tag(r.heard[first]),
        (2, 0),
        "the next starts from its first frame"
    );
    assert_eq!(
        r.engine.telemetry(P).underruns,
        0,
        "no silence played as audio"
    );
}

/// Player P playing track 1, a next (track 2) that opens only once `gate`
/// is set, and a transition to it at `at_secs` already scheduled.
fn slow_next(at_secs: f64, fade_until: Option<f64>) -> (Rig, Arc<std::sync::atomic::AtomicBool>) {
    let gate = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let mut r = rig_with(gated_opener(96_000, Arc::clone(&gate)), false);
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, 0.0),
    });
    r.settle();
    r.act(EngineAction::Preload {
        player: P,
        request: Some(SourceRequest {
            path: PathBuf::from("slow2"),
            ..request(2, 0.0)
        }),
    });
    r.run(1);
    r.act(EngineAction::Schedule {
        player: P,
        plan: Some(TransitionPlan::StartNextAt {
            at_secs,
            fade_current_until_secs: fade_until,
        }),
    });
    (r, gate)
}

fn open(gate: &std::sync::atomic::AtomicBool) {
    gate.store(true, std::sync::atomic::Ordering::Release);
}

#[test]
fn a_next_waiting_for_its_file_never_starts_after_a_stop() {
    // Rule 10: nothing goes on air by itself.
    for blocks in [15, 40] {
        let (mut r, gate) = slow_next(0.3, None);
        r.run(blocks); // inside the transition window, or past it
        r.act(EngineAction::StopNow { player: P });
        open(&gate);
        r.settle();
        r.run(60);
        assert!(
            !r.heard.iter().any(|v| tag(*v).0 == 2),
            "after {blocks} blocks"
        );
        assert!(
            !r.events
                .iter()
                .any(|e| matches!(e, EngineEvent::TransitionStarted { .. })),
            "{:?}",
            r.events
        );
    }
}

#[test]
fn a_next_ready_before_its_frame_still_starts_on_it() {
    let (mut r, gate) = slow_next(0.15, None);
    r.run(1);
    open(&gate);
    r.settle();
    r.run(40);
    let first = r.heard.iter().position(|v| tag(*v).0 == 2).unwrap();
    let start = r.heard.iter().position(|v| tag(*v).0 == 1).unwrap();
    assert_eq!(first - start, (0.15 * RATE) as usize, "not early, not late");
}

#[test]
fn a_pause_while_the_next_waits_takes_the_transition_back() {
    let (mut r, gate) = slow_next(0.1, Some(1.5));
    r.run(30); // past the frame, the current fading, the next not open yet
    r.act(EngineAction::Pause { player: P });
    open(&gate);
    r.settle();
    r.run(10);
    assert!(
        !r.events
            .iter()
            .any(|e| matches!(e, EngineEvent::TransitionStarted { .. })),
        "nothing starts while paused: {:?}",
        r.events
    );
    r.act(EngineAction::Resume { player: P });
    r.settle();
    r.run(200);
    assert!(
        r.heard.iter().any(|v| tag(*v).0 == 2),
        "the next plays after the resume"
    );
}
impl Rig {
    fn cue_position(&self) -> Option<f64> {
        self.engine.telemetry(P).cue_position_secs
    }

    /// Renders `blocks` blocks and returns what the Cue device played
    /// (left channel); the Main device is rendered too, as `run` does.
    fn run_hearing_cue(&mut self, blocks: usize) -> Vec<f32> {
        let mut heard = Vec::new();
        for _ in 0..blocks {
            let out = self.cue.render(BLOCK).unwrap();
            heard.extend(out.chunks(2).map(|f| f[0]));
            self.run(1);
        }
        heard
    }
}

fn start_cue(r: &mut Rig, from_secs: f64) {
    r.act(EngineAction::StartCue {
        player: P,
        request: request(3, from_secs),
    });
    r.settle();
}

#[test]
fn pausing_the_cue_holds_its_position_and_resuming_continues() {
    let mut r = rig(480_000, true);
    start_cue(&mut r, 0.0);
    r.run_hearing_cue(10);
    let before = r.cue_position().unwrap();
    assert!(before > 0.0, "{before}");

    r.act(EngineAction::SetCuePaused {
        player: P,
        paused: true,
    });
    // The pause ramps down; once it has, nothing more is played.
    r.run_hearing_cue(10);
    let held = r.cue_position().unwrap();
    let silent = r.run_hearing_cue(10);
    assert!(silent.iter().all(|v| *v == 0.0), "a held CUE is silent");
    assert_eq!(r.cue_position().unwrap(), held, "the position holds");

    r.act(EngineAction::SetCuePaused {
        player: P,
        paused: false,
    });
    let resumed = r.run_hearing_cue(10);
    assert!(resumed.iter().any(|v| *v != 0.0), "it plays again");
    assert!(r.cue_position().unwrap() > held + 0.05);
    assert!(
        !r.events
            .iter()
            .any(|e| matches!(e, EngineEvent::CueEnded { .. })),
        "pausing never ends the CUE"
    );
}

#[test]
fn seeking_the_cue_replaces_the_source_at_the_new_position() {
    let mut r = rig(480_000, true);
    start_cue(&mut r, 0.0);
    r.run_hearing_cue(2);
    r.act(EngineAction::SeekCue {
        player: P,
        secs: 3.0,
    });
    r.settle();
    let heard = r.run_hearing_cue(6);
    assert!(r.cue_position().unwrap() >= 3.0);
    let last = heard.iter().rev().find(|v| **v != 0.0).unwrap();
    // The tag's frame field overflows past 100 000 frames: undo the base.
    let frame = *last as u64 - 3 * 100_000;
    assert!(frame as f64 / RATE >= 3.0, "the Cue device plays from 3 s");
    assert!(
        r.heard.iter().all(|v| *v == 0.0),
        "the main output never hears the CUE"
    );
}

#[test]
fn seeking_a_paused_cue_stays_paused_at_the_new_position() {
    let mut r = rig(480_000, true);
    start_cue(&mut r, 0.0);
    r.run_hearing_cue(4);
    r.act(EngineAction::SetCuePaused {
        player: P,
        paused: true,
    });
    r.run_hearing_cue(10);
    r.act(EngineAction::SeekCue {
        player: P,
        secs: 2.0,
    });
    r.settle();
    let silent = r.run_hearing_cue(10);
    assert!(silent.iter().all(|v| *v == 0.0));
    let at = r.cue_position().unwrap();
    assert!((2.0..2.05).contains(&at), "held at the seek target: {at}");
    r.act(EngineAction::SetCuePaused {
        player: P,
        paused: false,
    });
    let heard = r.run_hearing_cue(10);
    assert!(heard.iter().any(|v| *v != 0.0));
    assert!(r.cue_position().unwrap() > at + 0.05);
}

#[test]
fn a_new_cue_after_a_pause_plays() {
    let mut r = rig(480_000, true);
    start_cue(&mut r, 0.0);
    r.run_hearing_cue(4);
    r.act(EngineAction::SetCuePaused {
        player: P,
        paused: true,
    });
    r.run_hearing_cue(10);
    // The model starts the new CUE unpaused (it moved to another entry).
    start_cue(&mut r, 0.0);
    let heard = r.run_hearing_cue(10);
    assert!(heard.iter().any(|v| *v != 0.0), "the new CUE is audible");
}

#[test]
fn cue_commands_without_a_cue_source_are_ignored() {
    let mut r = rig(48_000, true);
    r.act(EngineAction::SeekCue {
        player: P,
        secs: 1.0,
    });
    r.act(EngineAction::SetCuePaused {
        player: P,
        paused: true,
    });
    r.act(EngineAction::SetCuePaused {
        player: P,
        paused: false,
    });
    r.settle();
    r.run(2);
    assert_eq!(r.cue_position(), None);
    assert!(r.events.is_empty(), "{:?}", r.events);
}

#[test]
fn a_seek_past_the_end_ends_the_cue() {
    let mut r = rig(48_000, true);
    start_cue(&mut r, 0.0);
    r.act(EngineAction::SeekCue {
        player: P,
        secs: 5.0,
    });
    r.settle();
    r.run_hearing_cue(10);
    assert!(r.events.contains(&EngineEvent::CueEnded {
        player: P,
        entry: EntryId(3)
    }));
}
