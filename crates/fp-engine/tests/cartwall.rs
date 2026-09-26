#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Phase 2 spec P2.4: the cartwall on the Offline backend.

mod support;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use fp_backends::{AudioBackend, OfflineBackend, OfflineDevice};
use fp_engine::engine::{Engine, EngineSettings};
use fp_model::{
    CartId, CartRequest, Config, EngineAction, EngineEvent, Route, SOURCE_END, TrackId,
};
use support::tagged_opener;

const BLOCK: usize = 480;
const RATE: f64 = 48_000.0;

fn route(first_channel: u16) -> Route {
    Route {
        backend: "offline".into(),
        device: "card".into(),
        first_channel,
    }
}

fn cart(n: u64, from_secs: f64, until_secs: f64, looped: bool) -> CartRequest {
    CartRequest {
        cart: CartId(n),
        track: TrackId(n),
        path: PathBuf::from(format!("track{n}")),
        from_secs,
        until_secs,
        looped,
        format: None,
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
    config.outputs.cartwall.main = main;
    config.outputs.cartwall.cue = cue;
    config.tuning.gain_smoothing_ms = 0.0;
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
    let engine = Engine::new(
        backends,
        EngineSettings::from_config(&config),
        tagged_opener(track_frames),
    );
    Rig {
        engine,
        device,
        channels: usize::from(channels),
        clock: Instant::now(),
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

    fn run(&mut self, blocks: usize) {
        for _ in 0..blocks {
            let out = self.device.render(BLOCK).unwrap();
            self.frames
                .extend(out.chunks(self.channels).map(<[f32]>::to_vec));
            self.clock += Duration::from_millis(10);
            let events = self.engine.tick(self.clock);
            self.events.extend(events);
            // Give the worker time to keep rings topped up.
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    fn channel(&self, c: usize) -> Vec<f32> {
        self.frames.iter().map(|f| f[c]).collect()
    }
}

fn audible(samples: &[f32]) -> usize {
    samples.iter().filter(|v| **v != 0.0).count()
}

#[test]
fn a_fired_cart_plays_on_the_cartwall_route_and_ends_with_cart_ended() {
    let mut r = rig(2, Some(route(0)), None, 48_000);
    r.act(EngineAction::StartCart(cart(1, 0.0, 2_400.0 / RATE, false)));
    r.settle();
    r.run(30);
    assert!(
        r.events
            .contains(&EngineEvent::CartEnded { cart: CartId(1) })
    );
    assert_eq!(audible(&r.channel(0)), 2_400, "ends exactly at cue-out");
    assert_eq!(r.engine.used_slots(), 0);
}

#[test]
fn a_looped_cart_keeps_playing_across_the_loop_point() {
    let mut r = rig(2, Some(route(0)), None, 48_000);
    r.act(EngineAction::StartCart(cart(1, 0.0, 480.0 / RATE, true)));
    r.settle();
    r.run(40);
    let left = r.channel(0);
    let first = left.iter().position(|v| *v != 0.0).unwrap();
    let playing = &left[first..];
    assert_eq!(audible(playing), playing.len(), "no gap");
    for (i, v) in playing.iter().enumerate().skip(480) {
        assert_eq!(*v, playing[i - 480], "period of 480 frames at {i}");
    }
    assert!(
        !r.events
            .contains(&EngineEvent::CartEnded { cart: CartId(1) })
    );
}

#[test]
fn stopping_a_cart_before_it_starts_releases_its_slot() {
    let mut r = rig(2, Some(route(0)), None, 48_000);
    r.act(EngineAction::StartCart(cart(1, 0.0, SOURCE_END, false)));
    r.act(EngineAction::StopCart { cart: CartId(1) });
    r.settle();
    r.run(20);
    assert_eq!(audible(&r.channel(0)), 0);
    assert_eq!(r.engine.used_slots(), 0);
    assert!(r.events.is_empty(), "{:?}", r.events);
}

#[test]
fn stopping_a_playing_cart_fades_it_out_without_reporting_an_end() {
    let mut r = rig(2, Some(route(0)), None, 48_000);
    r.act(EngineAction::StartCart(cart(1, 0.0, SOURCE_END, false)));
    r.settle();
    r.run(5);
    r.act(EngineAction::StopCart { cart: CartId(1) });
    r.run(10);
    let tail = &r.channel(0)[r.frames.len() - BLOCK * 5..];
    assert_eq!(audible(tail), 0);
    assert_eq!(r.engine.used_slots(), 0);
    assert!(
        !r.events
            .contains(&EngineEvent::CartEnded { cart: CartId(1) })
    );
}

#[test]
fn sixteen_carts_fired_together_all_start() {
    let mut r = rig(2, Some(route(0)), None, 48_000);
    for n in 1..=16 {
        r.act(EngineAction::StartCart(cart(n, 0.0, SOURCE_END, false)));
    }
    r.settle();
    r.run(5);
    assert_eq!(r.engine.slot_exhaustions(), 0);
    let (carts, _) = r.engine.cart_telemetry();
    assert_eq!(carts.len(), 16);
    assert!(carts.iter().all(|(_, t)| t.position_secs > 0.0));
}

#[test]
fn a_missing_cart_file_reports_cart_failed() {
    let mut r = rig(2, Some(route(0)), None, 48_000);
    let mut request = cart(1, 0.0, SOURCE_END, false);
    request.path = PathBuf::from("missing1");
    r.act(EngineAction::StartCart(request));
    r.settle();
    r.run(5);
    assert!(
        r.events
            .contains(&EngineEvent::CartFailed { cart: CartId(1) })
    );
    assert_eq!(r.engine.used_slots(), 0);
}

#[test]
fn the_cart_cue_never_reaches_main() {
    let mut r = rig(4, Some(route(0)), Some(route(2)), 48_000);
    r.act(EngineAction::StartCartCue(cart(1, 0.0, SOURCE_END, false)));
    r.settle();
    r.run(10);
    assert_eq!(audible(&r.channel(0)) + audible(&r.channel(1)), 0);
    assert!(audible(&r.channel(2)) > 0);
    let (_, cue) = r.engine.cart_telemetry();
    assert_eq!(cue.map(|(c, _)| c), Some(CartId(1)));
    r.act(EngineAction::StopCartCue);
    r.run(10);
    assert_eq!(r.engine.used_slots(), 0);
}

#[test]
fn without_a_cue_route_the_cart_cue_ends_at_once() {
    let mut r = rig(2, Some(route(0)), None, 48_000);
    r.act(EngineAction::StartCartCue(cart(1, 0.0, SOURCE_END, false)));
    r.run(1);
    assert!(r.events.contains(&EngineEvent::CartCueEnded));
    assert_eq!(audible(&r.channel(0)), 0);
}

#[test]
fn cartwall_routes_default_to_the_default_output() {
    let mut r = rig(2, None, None, 48_000);
    r.act(EngineAction::StartCart(cart(1, 0.0, SOURCE_END, false)));
    r.settle();
    r.run(5);
    assert!(audible(&r.channel(0)) > 0);
}

#[test]
fn cart_positions_are_reported_and_wrap_when_looped() {
    let mut r = rig(2, Some(route(0)), None, 48_000);
    r.act(EngineAction::StartCart(cart(
        1,
        0.5,
        0.5 + 480.0 / RATE,
        true,
    )));
    r.settle();
    r.run(20);
    let (carts, _) = r.engine.cart_telemetry();
    let pos = carts[0].1.position_secs;
    assert!((0.5..0.5 + 480.0 / RATE).contains(&pos), "{pos}");
}

#[test]
fn a_cue_route_to_a_missing_backend_never_plays_on_main() {
    let foreign = Route {
        backend: "elsewhere".into(),
        device: "card".into(),
        first_channel: 0,
    };
    let mut r = rig(2, Some(route(0)), Some(foreign), 48_000);
    r.act(EngineAction::StartCartCue(cart(1, 0.0, SOURCE_END, false)));
    r.run(5);
    assert!(r.events.contains(&EngineEvent::CartCueEnded));
    assert_eq!(audible(&r.channel(0)), 0);
}

#[test]
fn a_cue_route_equal_to_main_is_refused() {
    let mut r = rig(2, Some(route(0)), Some(route(0)), 48_000);
    r.act(EngineAction::StartCartCue(cart(1, 0.0, SOURCE_END, false)));
    r.run(5);
    assert!(r.events.contains(&EngineEvent::CartCueEnded));
    assert_eq!(audible(&r.channel(0)), 0);
}

#[test]
fn stopping_a_cart_just_after_its_start_is_requested_still_fades() {
    let mut r = rig(2, Some(route(0)), None, 48_000);
    r.act(EngineAction::StartCart(cart(1, 0.0, SOURCE_END, false)));
    r.settle(); // Start requested; the mixer has not reported Started yet.
    let out = r.device.render(BLOCK).unwrap();
    r.frames.extend(out.chunks(r.channels).map(<[f32]>::to_vec));
    r.act(EngineAction::StopCart { cart: CartId(1) });
    r.run(10);
    let left = r.channel(0);
    let last = left.iter().rposition(|v| *v != 0.0).unwrap();
    let peak = left.iter().fold(0.0f32, |m, v| m.max(v.abs()));
    assert!(
        left[last].abs() < peak * 0.1,
        "cut at full level: {} of {peak}",
        left[last]
    );
    assert_eq!(r.engine.used_slots(), 0);
}

#[test]
fn a_looped_cart_without_a_known_end_reports_a_wrapped_position() {
    // 4 800-frame file (0.1 s), looped to its natural end.
    let mut r = rig(2, Some(route(0)), None, 4_800);
    r.act(EngineAction::StartCart(cart(1, 0.0, SOURCE_END, true)));
    r.settle();
    r.run(40); // 0.4 s: several passes
    let (carts, _) = r.engine.cart_telemetry();
    let pos = carts[0].1.position_secs;
    assert!((0.0..0.1).contains(&pos), "{pos}");
}
