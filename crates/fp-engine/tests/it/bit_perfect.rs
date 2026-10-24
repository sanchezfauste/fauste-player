#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Bit-perfect buses (Phase 4 spec B3–B5): the stream rate follows the
//! file while the bus is idle, and untouched sources reach the device
//! unchanged.

use crate::support;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use fp_backends::{AudioBackend, OfflineBackend, OfflineDevice};
use fp_engine::engine::{Engine, EngineSettings};
use fp_engine::worker::file_opener;
use fp_model::{
    AudioFormat, CartId, CartRequest, Config, EngineAction, EngineEvent, EntryId, Normalization,
    NormalizationStatus, OutputDevice, PlayerId, PlayerRoutes, Route, SourceRequest, TrackId,
    TransitionPlan,
};
use support::indexed_wav;

const P: PlayerId = PlayerId(1);
const BLOCK: usize = 480;

struct Rig {
    engine: Engine,
    dac: OfflineDevice,
    clock: Instant,
    dir: tempfile::TempDir,
    /// Every event the engine reported, in order.
    events: Vec<EngineEvent>,
}

fn dac_route(first_channel: u16) -> Route {
    Route {
        backend: "offline".into(),
        device: "dac".into(),
        first_channel,
    }
}

fn rig(bit_perfect: bool, exclusive_capable: bool) -> Rig {
    rig_routed(bit_perfect, exclusive_capable, dac_route(0), None)
}

/// As `rig`, with P's Main on `main` and its Cue on `cue`.
fn rig_routed(bit_perfect: bool, exclusive_capable: bool, main: Route, cue: Option<Route>) -> Rig {
    let backend = OfflineBackend::new();
    let dac = backend.add_device("dac", 2);
    dac.set_exclusive_capable(exclusive_capable);
    // An ordinary output to move a player to.
    backend.add_device("speakers", 2);
    let mut config = Config::default();
    config.outputs.backend = Some("offline".into());
    config.outputs.buffer_frames = BLOCK as u32;
    config.outputs.routes = vec![PlayerRoutes {
        player: P,
        main: Some(main),
        cue,
    }];
    if bit_perfect {
        config.outputs.bit_perfect = vec![OutputDevice {
            backend: "offline".into(),
            device: "dac".into(),
        }];
    }
    config.tuning.gain_smoothing_ms = 0.0;
    // The tests render far faster than real time, and the worker may get
    // little CPU (parallel tests): a source starts only once its whole file
    // is decoded (every file here lasts 3 s, under the 4 s ring), so the
    // render never outruns the worker into an underrun.
    config.tuning.prebuffer_secs = 4.0;
    config.tuning.ready_threshold_ms = 4_000.0;
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
    let mut engine = Engine::new(
        backends,
        EngineSettings::from_config(&config),
        file_opener(),
    );
    let clock = Instant::now();
    engine.execute(EngineAction::AddPlayer { player: P }, clock);
    Rig {
        engine,
        dac,
        clock,
        dir: tempfile::tempdir().unwrap(),
        events: Vec::new(),
    }
}

fn wav(dir: &Path, name: &str, rate: u32) -> PathBuf {
    indexed_wav(dir, name, rate, 2, rate as usize * 3)
}

fn request(n: u64, path: PathBuf, format: Option<AudioFormat>) -> SourceRequest {
    SourceRequest {
        entry: EntryId(n),
        track: TrackId(n),
        path,
        from_secs: 0.0,
        format,
        normalization: fp_model::Normalization::default(),
    }
}

fn pcm16(rate: u32) -> Option<AudioFormat> {
    Some(AudioFormat {
        sample_rate: rate,
        bits: Some(16),
        channels: 2,
        dsd_rate: None,
        bitrate: None,
    })
}

impl Rig {
    fn act(&mut self, action: EngineAction) {
        self.engine.execute(action, self.clock);
    }

    fn cue(&mut self, request: SourceRequest) {
        self.act(EngineAction::StartCue { player: P, request });
        self.settle();
    }

    fn start(&mut self, request: SourceRequest) {
        self.act(EngineAction::StartCurrent { player: P, request });
        self.settle();
    }

    /// Waits until no source waits for its worker. The deadline only bounds
    /// a hang: going on unsettled would test something else, so it fails.
    fn settle(&mut self) {
        let deadline = Instant::now() + Duration::from_secs(60);
        while self.engine.unsettled_sources() > 0 {
            assert!(
                Instant::now() < deadline,
                "the sources never settled: {}",
                self.engine.describe_player(P)
            );
            self.tick();
            std::thread::sleep(Duration::from_millis(1));
        }
        self.tick();
    }

    fn tick(&mut self) {
        let events = self.engine.tick(self.clock);
        self.events.extend(events);
    }

    /// The `StreamRate` reports so far, in order.
    fn stream_rates(&self) -> Vec<(u32, Vec<u32>)> {
        self.events
            .iter()
            .filter_map(|e| match e {
                EngineEvent::StreamRate { rate, refused, .. } => Some((*rate, refused.clone())),
                _ => None,
            })
            .collect()
    }

    fn transition_started(&self, entry: u64) -> bool {
        self.events.iter().any(|e| {
            matches!(e, EngineEvent::TransitionStarted { entry: EntryId(n), .. } if *n == entry)
        })
    }

    fn rate(&self) -> u32 {
        self.dac.config().unwrap().sample_rate
    }

    /// Renders `frames` frames in blocks, ticking after each; the left channel.
    fn run(&mut self, frames: usize) -> Vec<f32> {
        let rate = self.rate();
        let mut heard = Vec::new();
        for _ in 0..frames.div_ceil(BLOCK) {
            let out = self.dac.render(BLOCK).unwrap();
            heard.extend(out.chunks(2).map(|f| f[0]));
            self.clock += Duration::from_secs_f64(BLOCK as f64 / f64::from(rate));
            self.tick();
        }
        heard
    }
}

#[test]
fn an_idle_bit_perfect_bus_reopens_at_the_file_rate() {
    let mut r = rig(true, true);
    assert_eq!(r.rate(), 48_000);
    let path = wav(r.dir.path(), "a.wav", 44_100);
    r.start(request(1, path, pcm16(44_100)));
    assert_eq!(r.rate(), 44_100);
    assert!(r.dac.config().unwrap().exclusive);
    let heard = r.run(44_100);
    assert!(heard.iter().any(|v| *v != 0.0), "it plays");
    let position = r.engine.telemetry(P).position_secs.unwrap();
    assert!(
        (position - 1.0).abs() < 0.05,
        "one second at 44.1 kHz is one second: {position} (underruns {})",
        r.engine.telemetry(P).underruns
    );
}

#[test]
fn br1_a_bus_a_holder_comes_back_to_reports_its_rate_again() {
    // The model forgets a device's rate when its last holder leaves it; a
    // holder that comes back before the bus closes must bring it back.
    let mut r = rig(true, true);
    let dac = OutputDevice {
        backend: "offline".into(),
        device: "dac".into(),
    };
    let to = |device: &str| Route {
        backend: "offline".into(),
        device: device.into(),
        first_channel: 0,
    };
    r.tick();
    assert_eq!(r.stream_rates(), vec![(48_000, vec![])], "reported once");
    r.events.clear();
    for device in ["speakers", "dac"] {
        r.act(EngineAction::ApplyRoute {
            holder: fp_model::Holder::PlayerMain(P),
            route: Some(to(device)),
            force: false,
        });
    }
    r.tick();
    let back = r
        .events
        .iter()
        .rposition(|e| matches!(e, EngineEvent::Placed { device, .. } if *device == dac))
        .expect("placed on the dac again");
    let reported = r.events.iter().skip(back).any(
        |e| matches!(e, EngineEvent::StreamRate { device, rate: 48_000, .. } if *device == dac),
    );
    assert!(reported, "{:?}", r.events);
}

#[test]
fn a_busy_bit_perfect_bus_keeps_its_rate() {
    let mut r = rig(true, true);
    let first = wav(r.dir.path(), "a.wav", 48_000);
    let second = wav(r.dir.path(), "b.wav", 44_100);
    r.start(request(1, first, pcm16(48_000)));
    r.run(4_800);
    r.act(EngineAction::Preload {
        player: P,
        request: Some(request(2, second, pcm16(44_100))),
    });
    r.settle();
    assert_eq!(r.rate(), 48_000, "never while something is attached");
}

#[test]
fn a_normal_bus_never_changes_rate() {
    let mut r = rig(false, true);
    let path = wav(r.dir.path(), "a.wav", 44_100);
    r.start(request(1, path, pcm16(44_100)));
    assert_eq!(r.rate(), 48_000);
    assert!(!r.dac.config().unwrap().exclusive);
}

#[test]
fn a_rate_the_device_refuses_keeps_the_previous_rate() {
    let mut r = rig(true, true);
    r.dac.refuse_rate(44_100);
    let path = wav(r.dir.path(), "a.wav", 44_100);
    r.start(request(1, path, pcm16(44_100)));
    assert_eq!(r.rate(), 48_000);
    let heard = r.run(4_800);
    assert!(heard.iter().any(|v| *v != 0.0), "it still plays, resampled");
}

#[test]
fn an_unknown_format_plays_at_the_bus_rate() {
    let mut r = rig(true, true);
    let path = wav(r.dir.path(), "a.wav", 44_100);
    r.start(request(1, path, None));
    assert_eq!(r.rate(), 48_000);
}

#[test]
fn a_device_without_exclusive_access_still_plays() {
    let mut r = rig(true, false);
    let path = wav(r.dir.path(), "a.wav", 48_000);
    r.start(request(1, path, pcm16(48_000)));
    assert!(!r.dac.config().unwrap().exclusive, "shared instead");
    let heard = r.run(4_800);
    assert!(heard.iter().any(|v| *v != 0.0));
}

/// Runs until the badge settles, a few blocks at most.
fn bit_perfect_after(r: &mut Rig, frames: usize) -> bool {
    r.run(frames);
    r.engine.telemetry(P).bit_perfect
}

#[test]
fn bit_perfect_playback_is_bit_exact() {
    let mut r = rig(true, true);
    let path = wav(r.dir.path(), "a.wav", 44_100);
    r.start(request(1, path, pcm16(44_100)));
    let heard = r.run(30_000);
    // `indexed_wav` sample i is (i % 20 000) as i16: find index 1, then
    // every following sample must be the file's value, bit for bit.
    let first = heard
        .iter()
        .position(|v| *v == 1.0 / 32_768.0)
        .expect("the file starts playing");
    for (i, v) in heard[first - 1..].iter().enumerate() {
        let expected = (i % 20_000) as f32 / 32_768.0;
        assert!(
            v.to_bits() == expected.to_bits(),
            "sample {i}: {v} is not {expected}"
        );
    }
    assert!(r.engine.telemetry(P).bit_perfect, "the badge is lit");
}

#[test]
fn volume_below_full_is_not_bit_perfect() {
    let mut r = rig(true, true);
    let path = wav(r.dir.path(), "a.wav", 48_000);
    r.start(request(1, path, pcm16(48_000)));
    assert!(bit_perfect_after(&mut r, 2_400));
    r.act(EngineAction::SetVolume {
        player: P,
        volume: 0.99,
    });
    assert!(!bit_perfect_after(&mut r, 2_400));
    r.act(EngineAction::SetVolume {
        player: P,
        volume: 1.0,
    });
    assert!(bit_perfect_after(&mut r, 2_400), "back at exactly 100 %");
}

#[test]
fn an_overlapping_source_is_not_bit_perfect() {
    let mut r = rig(true, true);
    let path = wav(r.dir.path(), "a.wav", 48_000);
    let jingle = wav(r.dir.path(), "j.wav", 48_000);
    r.start(request(1, path, pcm16(48_000)));
    assert!(bit_perfect_after(&mut r, 2_400));
    r.act(EngineAction::StartCart(CartRequest {
        cart: CartId(1),
        track: TrackId(9),
        path: jingle,
        from_secs: 0.0,
        until_secs: f64::INFINITY,
        looped: false,
        format: pcm16(48_000),
        normalization: fp_model::Normalization::default(),
    }));
    r.settle();
    assert!(!bit_perfect_after(&mut r, 2_400), "a cart plays over it");
}

#[test]
fn a_resampled_source_is_not_bit_perfect() {
    let mut r = rig(true, true);
    r.dac.refuse_rate(44_100);
    let path = wav(r.dir.path(), "a.wav", 44_100);
    r.start(request(1, path, pcm16(44_100)));
    assert!(!bit_perfect_after(&mut r, 2_400));
}

#[test]
fn a_shared_device_is_not_bit_perfect() {
    let mut r = rig(true, false);
    let path = wav(r.dir.path(), "a.wav", 48_000);
    r.start(request(1, path, pcm16(48_000)));
    assert!(!bit_perfect_after(&mut r, 2_400));
}

#[test]
fn a_lossy_file_is_not_bit_perfect() {
    let mut r = rig(true, true);
    let path = wav(r.dir.path(), "a.wav", 48_000);
    let lossy = Some(AudioFormat {
        sample_rate: 48_000,
        bits: None,
        channels: 2,
        dsd_rate: None,
        bitrate: None,
    });
    r.start(request(1, path, lossy));
    assert!(!bit_perfect_after(&mut r, 2_400));
}

#[test]
fn a_multichannel_file_is_not_bit_perfect() {
    let mut r = rig(true, true);
    // Three channels are downmixed to stereo: the samples change.
    let path = indexed_wav(r.dir.path(), "c.wav", 48_000, 3, 48_000);
    let format = Some(AudioFormat {
        sample_rate: 48_000,
        bits: Some(16),
        channels: 3,
        dsd_rate: None,
        bitrate: None,
    });
    r.start(request(1, path, format));
    assert!(!bit_perfect_after(&mut r, 2_400));
}

#[test]
fn a_preload_does_not_decide_the_rate() {
    let mut r = rig(true, true);
    let path = wav(r.dir.path(), "a.wav", 44_100);
    r.act(EngineAction::Preload {
        player: P,
        request: Some(request(1, path.clone(), pcm16(44_100))),
    });
    r.settle();
    assert_eq!(r.rate(), 48_000, "waiting is not playing");
    r.start(request(1, path, pcm16(44_100)));
    assert_eq!(r.rate(), 44_100, "starting it is");
    assert!(bit_perfect_after(&mut r, 4_410));
}

#[test]
fn after_a_stop_the_next_track_follows_its_rate() {
    let mut r = rig(true, true);
    let first = wav(r.dir.path(), "a.wav", 44_100);
    let second = wav(r.dir.path(), "b.wav", 48_000);
    r.start(request(1, first, pcm16(44_100)));
    r.run(4_410);
    // Preloaded while the bus plays at 44.1 kHz: resampled for now.
    r.act(EngineAction::Preload {
        player: P,
        request: Some(request(2, second.clone(), pcm16(48_000))),
    });
    r.settle();
    assert_eq!(r.rate(), 44_100);
    r.act(EngineAction::StopNow { player: P });
    r.run(4_410);
    r.start(request(2, second, pcm16(48_000)));
    assert_eq!(r.rate(), 48_000);
    let heard = r.run(9_600);
    assert!(heard.iter().any(|v| *v != 0.0), "it plays");
    assert!(
        r.engine.telemetry(P).bit_perfect,
        "the preload was reopened at the new rate, not played resampled"
    );
}

#[test]
fn a_track_resumed_after_a_restart_follows_its_rate() {
    let mut r = rig(true, true);
    let path = wav(r.dir.path(), "a.wav", 44_100);
    r.act(EngineAction::LoadPaused {
        player: P,
        request: request(1, path, pcm16(44_100)),
    });
    r.settle();
    assert_eq!(r.rate(), 48_000);
    r.act(EngineAction::Resume { player: P });
    r.settle();
    assert_eq!(r.rate(), 44_100);
    let heard = r.run(4_410);
    assert!(heard.iter().any(|v| *v != 0.0), "it plays");
}

#[test]
fn a_refused_rate_is_not_tried_again_on_every_start() {
    let mut r = rig(true, true);
    r.dac.refuse_rate(44_100);
    let path = wav(r.dir.path(), "a.wav", 44_100);
    r.start(request(1, path.clone(), pcm16(44_100)));
    r.run(2_400);
    r.act(EngineAction::StopNow { player: P });
    r.run(2_400);
    let attempts = r.dac.open_attempts();
    r.start(request(2, path, pcm16(44_100)));
    assert_eq!(r.dac.open_attempts(), attempts, "the device said no once");
    assert_eq!(r.rate(), 48_000);
}

#[test]
fn br1_a_bus_reports_its_rate_when_it_opens_and_when_it_follows_a_file() {
    let mut r = rig(true, true);
    r.settle();
    assert_eq!(
        r.stream_rates(),
        vec![(48_000, vec![])],
        "opened at the global rate"
    );
    let path = wav(r.dir.path(), "a.wav", 44_100);
    r.start(request(1, path, pcm16(44_100)));
    assert_eq!(
        r.stream_rates().last(),
        Some(&(44_100, vec![])),
        "a B3.2 reopen"
    );
    let before = r.stream_rates().len();
    r.run(4_410);
    assert_eq!(
        r.stream_rates().len(),
        before,
        "nothing changed, nothing reported"
    );
}

#[test]
fn br1_a_refused_rate_is_reported() {
    let mut r = rig(true, true);
    r.dac.refuse_rate(44_100);
    let path = wav(r.dir.path(), "a.wav", 44_100);
    r.start(request(1, path, pcm16(44_100)));
    assert_eq!(r.stream_rates().last(), Some(&(48_000, vec![44_100])));
}

fn cut_plan(at_secs: f64, rate_cut: Option<u32>) -> EngineAction {
    EngineAction::Schedule {
        player: P,
        plan: Some(TransitionPlan::StartNextAt {
            at_secs,
            fade_current_until_secs: None,
            rate_cut,
        }),
    }
}

/// A 44.1 kHz track on air on the bit-perfect bus, and a 48 kHz next preloaded.
fn on_air_with_next(r: &mut Rig) {
    let a = wav(r.dir.path(), "a.wav", 44_100);
    let b = wav(r.dir.path(), "b.wav", 48_000);
    r.start(request(1, a, pcm16(44_100)));
    assert_eq!(r.rate(), 44_100);
    r.act(EngineAction::Preload {
        player: P,
        request: Some(request(2, b, pcm16(48_000))),
    });
}

/// The decode rate of P's source in `part` (`"cur ["` or `"pre ["`), read
/// from the engine's one-line description.
fn decode_rate(r: &Rig, part: &str) -> Option<u32> {
    let d = r.engine.describe_player(P);
    let source = d.split(part).nth(1)?.split(']').next()?;
    source
        .split("rate ")
        .nth(1)?
        .split(' ')
        .next()?
        .parse()
        .ok()
}

#[test]
fn br6_the_next_of_a_planned_cut_is_prepared_at_the_new_rate() {
    let mut r = rig(true, true);
    on_air_with_next(&mut r);
    assert_eq!(decode_rate(&r, "pre ["), Some(44_100), "no cut planned yet");
    r.act(cut_plan(0.5, Some(48_000)));
    r.settle();
    assert_eq!(r.rate(), 44_100, "nothing reopens before the transition");
    assert_eq!(decode_rate(&r, "pre ["), Some(48_000));
}

#[test]
fn br6_a_dropped_cut_opens_the_next_again_at_the_stream_rate() {
    let mut r = rig(true, true);
    on_air_with_next(&mut r);
    r.act(cut_plan(0.5, Some(48_000)));
    r.settle();
    r.act(cut_plan(0.5, None));
    r.settle();
    assert_eq!(decode_rate(&r, "pre ["), Some(44_100));
}

/// Renders block by block until the dac runs at `rate`, at most `frames`.
fn run_until_rate(r: &mut Rig, rate: u32, frames: usize) -> bool {
    for _ in 0..frames.div_ceil(BLOCK) {
        if r.rate() == rate {
            return true;
        }
        r.run(BLOCK);
    }
    r.rate() == rate
}

fn jingle(r: &Rig) -> EngineAction {
    EngineAction::StartCart(CartRequest {
        cart: CartId(1),
        track: TrackId(9),
        path: wav(r.dir.path(), "j.wav", 44_100),
        from_secs: 0.0,
        until_secs: f64::INFINITY,
        looped: false,
        format: pcm16(44_100),
        normalization: fp_model::Normalization::default(),
    })
}

#[test]
fn br6_a_scheduled_cut_reopens_the_device_between_the_tracks() {
    let mut r = rig(true, true);
    on_air_with_next(&mut r);
    r.act(cut_plan(0.5, Some(48_000)));
    r.settle();
    assert!(
        run_until_rate(&mut r, 48_000, 44_100),
        "past 0.5 s the device runs at 48 kHz"
    );
    assert_eq!(
        r.engine.unsettled_sources(),
        0,
        "the next was ready at the new rate: nothing to buffer after the reopen"
    );
    r.run(4_800);
    assert!(r.transition_started(2));
    assert!(
        bit_perfect_after(&mut r, 4_800),
        "the next track is bit-perfect"
    );
    assert_eq!(r.stream_rates().last(), Some(&(48_000, vec![])));
}

#[test]
fn br6_without_a_cut_the_hard_join_keeps_the_rate() {
    let mut r = rig(true, true);
    let a = wav(r.dir.path(), "a.wav", 44_100);
    let b = wav(r.dir.path(), "b.wav", 44_100);
    r.start(request(1, a, pcm16(44_100)));
    r.act(EngineAction::Preload {
        player: P,
        request: Some(request(2, b, pcm16(44_100))),
    });
    r.act(cut_plan(0.5, None));
    r.settle();
    let attempts = r.dac.open_attempts();
    r.run(44_100);
    assert!(r.transition_started(2));
    assert_eq!(r.rate(), 44_100);
    assert_eq!(r.dac.open_attempts(), attempts, "no reopen");
    assert!(bit_perfect_after(&mut r, 4_410));
}

#[test]
fn br6_a_cart_fired_just_before_the_cut_keeps_the_rate_and_the_next_plays() {
    let mut r = rig(true, true);
    on_air_with_next(&mut r);
    r.act(cut_plan(0.5, Some(48_000)));
    r.settle();
    let cart = jingle(&r);
    r.act(cart);
    r.settle();
    r.run(44_100);
    r.settle();
    r.run(4_410);
    assert!(r.transition_started(2), "the next plays");
    assert_eq!(r.rate(), 44_100, "the cart sounds: no reopen");
    assert_eq!(decode_rate(&r, "cur ["), Some(44_100), "at the stream rate");
    assert!(!r.engine.telemetry(P).bit_perfect, "resampled");
}

#[test]
fn br6_a_refused_cut_falls_back_and_the_next_plays() {
    let mut r = rig(true, true);
    r.dac.refuse_rate(48_000);
    on_air_with_next(&mut r);
    r.act(cut_plan(0.5, Some(48_000)));
    r.settle();
    r.run(44_100);
    r.settle();
    r.run(4_410);
    assert!(r.transition_started(2));
    assert_eq!(r.rate(), 44_100, "B3.4: back to the previous rate");
    assert_eq!(r.stream_rates().last(), Some(&(44_100, vec![48_000])));
    assert_eq!(decode_rate(&r, "cur ["), Some(44_100), "at the stream rate");
    let heard = r.run(4_410);
    assert!(heard.iter().any(|v| *v != 0.0), "it plays, resampled");
}

/// P plays a 44.1 kHz track; `b` (48 kHz) is started over it with `action`.
fn started_over(action: fn(SourceRequest) -> EngineAction) -> Rig {
    let mut r = rig(true, true);
    let a = wav(r.dir.path(), "a.wav", 44_100);
    let b = wav(r.dir.path(), "b.wav", 48_000);
    r.start(request(1, a, pcm16(44_100)));
    r.run(4_410);
    r.act(action(request(2, b, pcm16(48_000))));
    r
}

fn hard_start(request: SourceRequest) -> EngineAction {
    EngineAction::StartCurrent { player: P, request }
}

fn crossfade(request: SourceRequest) -> EngineAction {
    EngineAction::Crossfade {
        player: P,
        request,
        fade_ms: 200,
    }
}

fn dac() -> OutputDevice {
    OutputDevice {
        backend: "offline".into(),
        device: "dac".into(),
    }
}

#[test]
fn br8_a_hard_start_at_another_rate_reopens_after_the_declick() {
    let mut r = started_over(hard_start);
    assert_eq!(r.rate(), 44_100, "not while the old track's declick sounds");
    r.settle();
    r.run(BLOCK);
    r.settle();
    assert_eq!(r.rate(), 48_000);
    assert!(bit_perfect_after(&mut r, 4_800));
    assert_eq!(r.stream_rates().last(), Some(&(48_000, vec![])));
}

/// What the Offline dac holds in the play-out tests (`latency_frames`):
/// three blocks, as a device whose driver and transfers hold a few periods.
const HELD: usize = BLOCK * 3;

/// Renders block by block until the dac runs at `rate`, at most `frames`;
/// returns the silent frames the old stream rendered after its last sound
/// before it closed. A close discards what the stream holds, so fewer than
/// `HELD` would have cut the end of the sound short.
fn silence_before_rate(r: &mut Rig, rate: u32, frames: usize) -> usize {
    let mut trailing = 0;
    for _ in 0..frames.div_ceil(BLOCK) {
        if r.rate() == rate {
            return trailing;
        }
        for v in r.run(BLOCK) {
            trailing = if v == 0.0 { trailing + 1 } else { 0 };
        }
    }
    panic!("the dac never reopened at {rate}");
}

#[test]
fn br6_a_scheduled_cut_reopens_once_the_device_has_played_the_end() {
    let mut r = rig(true, true);
    r.dac.set_latency_frames(HELD as u32);
    on_air_with_next(&mut r);
    r.act(cut_plan(0.5, Some(48_000)));
    r.settle();
    let silence = silence_before_rate(&mut r, 48_000, 44_100);
    assert!(silence >= HELD, "{silence} frames: the end was cut");
    assert!(silence < HELD + 2 * BLOCK, "{silence} frames: no longer");
    r.run(4_800);
    assert!(r.transition_started(2));
    assert!(bit_perfect_after(&mut r, 4_800));
}

#[test]
fn br6_a_stop_while_the_device_plays_out_the_end_starts_nothing() {
    let mut r = rig(true, true);
    r.dac.set_latency_frames(HELD as u32);
    on_air_with_next(&mut r);
    r.act(cut_plan(0.5, Some(48_000)));
    r.settle();
    // Until the current has stopped: a block of silence, the end held.
    let mut trailing = 0;
    while trailing < BLOCK {
        for v in r.run(BLOCK) {
            trailing = if v == 0.0 { trailing + 1 } else { 0 };
        }
    }
    assert_eq!(r.rate(), 44_100, "still playing out");
    r.act(EngineAction::StopNow { player: P });
    let before = r.events.len();
    let heard = r.run(9_600);
    assert!(
        !r.events[before..]
            .iter()
            .any(|e| matches!(e, EngineEvent::ReachedEnd { .. })),
        "a stopped track does not reach its end afterwards"
    );
    assert!(heard.iter().all(|v| *v == 0.0), "silence");
    assert!(!r.transition_started(2), "the next never starts");
    assert_eq!(r.rate(), 44_100, "no reopen for a track that will not play");
}

#[test]
fn br6_a_pause_while_the_device_plays_out_the_end_holds_the_next_until_resume() {
    let mut r = rig(true, true);
    r.dac.set_latency_frames(HELD as u32);
    on_air_with_next(&mut r);
    r.act(cut_plan(0.5, Some(48_000)));
    r.settle();
    // Until the current has stopped: a block of silence, the end held.
    let mut trailing = 0;
    while trailing < BLOCK {
        for v in r.run(BLOCK) {
            trailing = if v == 0.0 { trailing + 1 } else { 0 };
        }
    }
    assert_eq!(r.rate(), 44_100, "still playing out");
    r.act(EngineAction::Pause { player: P });
    assert!(
        run_until_rate(&mut r, 48_000, 9_600),
        "the cut happens once the end has played"
    );
    let heard = r.run(9_600);
    assert!(heard.iter().all(|v| *v == 0.0), "paused: silence");
    assert!(!r.transition_started(2), "the next waits for the resume");
    r.act(EngineAction::Resume { player: P });
    r.settle();
    let heard = r.run(9_600);
    assert!(r.transition_started(2), "the next starts on resume");
    assert!(heard.iter().any(|v| *v != 0.0), "and plays");
    assert!(r.engine.telemetry(P).bit_perfect, "at its own rate");
}

#[test]
fn br6_a_pause_during_the_held_end_lets_a_device_change_apply_once_the_end_has_played() {
    let mut r = rig(true, true);
    r.dac.set_latency_frames(HELD as u32);
    on_air_with_next(&mut r);
    r.act(cut_plan(0.5, Some(48_000)));
    r.settle();
    // Until the current has stopped: a block of silence, the end held.
    let mut trailing = 0;
    while trailing < BLOCK {
        for v in r.run(BLOCK) {
            trailing = if v == 0.0 { trailing + 1 } else { 0 };
        }
    }
    r.act(EngineAction::Pause { player: P });
    let running = r.engine.running_settings(&dac()).unwrap();
    r.act(EngineAction::ApplyDevice {
        device: dac(),
        settings: fp_model::DeviceSettings {
            buffer_frames: 960,
            ..running
        },
        force: false,
    });
    let applied = |r: &Rig| {
        r.events
            .iter()
            .any(|e| matches!(e, EngineEvent::Applied { .. }))
    };
    assert!(!applied(&r), "not while the end plays out");
    assert!(run_until_rate(&mut r, 48_000, 9_600), "the cut happens");
    r.run(BLOCK * 4);
    assert!(
        applied(&r),
        "paused and played out: quiet, without waiting for the resume"
    );
    assert!(!r.transition_started(2), "still paused");
    r.act(EngineAction::Resume { player: P });
    r.settle();
    let heard = r.run(9_600);
    assert!(r.transition_started(2), "the next starts on resume");
    assert!(heard.iter().any(|v| *v != 0.0), "and plays");
    assert!(r.engine.telemetry(P).bit_perfect, "at its own rate");
}

#[test]
fn br8_a_hard_start_reopens_once_the_device_has_played_the_declick() {
    let mut r = rig(true, true);
    r.dac.set_latency_frames(HELD as u32);
    let a = wav(r.dir.path(), "a.wav", 44_100);
    let b = wav(r.dir.path(), "b.wav", 48_000);
    r.start(request(1, a, pcm16(44_100)));
    r.run(4_410);
    r.act(hard_start(request(2, b, pcm16(48_000))));
    r.settle();
    let silence = silence_before_rate(&mut r, 48_000, 44_100);
    assert!(silence >= HELD, "{silence} frames: the declick was cut");
    assert!(silence < HELD + 2 * BLOCK, "{silence} frames: no longer");
    r.settle();
    assert!(bit_perfect_after(&mut r, 4_800));
}

#[test]
fn br17_a_start_right_after_a_paused_track_stopped_reopens_once_the_pause_ramp_has_played() {
    let mut r = rig(true, true);
    r.dac.set_latency_frames(HELD as u32);
    let a = wav(r.dir.path(), "a.wav", 44_100);
    let b = wav(r.dir.path(), "b.wav", 48_000);
    r.start(request(1, a, pcm16(44_100)));
    r.run(4_410);
    r.act(EngineAction::Pause { player: P });
    // Past the pause ramp (and the engine's block of margin), well within
    // what the dac holds.
    let mut before = 0;
    while before < BLOCK * 2 {
        for v in r.run(BLOCK) {
            before = if v == 0.0 { before + 1 } else { 0 };
        }
    }
    assert!(before < HELD, "{before} frames: the test needs a held ramp");
    // The paused track is released at once, with no `Finished` event.
    r.act(EngineAction::StopNow { player: P });
    r.start(request(2, b, pcm16(48_000)));
    let silence = before + silence_before_rate(&mut r, 48_000, 44_100);
    assert!(silence >= HELD, "{silence} frames: the pause ramp was cut");
    r.settle();
    let heard = r.run(9_600);
    assert!(heard.iter().any(|v| *v != 0.0), "it plays");
    assert!(r.engine.telemetry(P).bit_perfect, "at its own rate");
}

#[test]
fn a_start_right_after_a_stop_reopens_once_the_declick_has_played() {
    let mut r = rig(true, true);
    r.dac.set_latency_frames(HELD as u32);
    let a = wav(r.dir.path(), "a.wav", 44_100);
    let b = wav(r.dir.path(), "b.wav", 48_000);
    r.start(request(1, a, pcm16(44_100)));
    r.run(4_410);
    r.act(EngineAction::StopNow { player: P });
    r.run(BLOCK);
    r.start(request(2, b, pcm16(48_000)));
    let silence = silence_before_rate(&mut r, 48_000, 44_100);
    assert!(silence >= HELD, "{silence} frames: the declick was cut");
    r.settle();
    let heard = r.run(9_600);
    assert!(heard.iter().any(|v| *v != 0.0), "it plays");
    assert!(r.engine.telemetry(P).bit_perfect, "at its own rate");
}

#[test]
fn br17_a_bus_opened_again_does_not_wait_for_the_end_on_the_closed_one() {
    let mut r = rig(true, true);
    r.dac.set_latency_frames(HELD as u32);
    let a = wav(r.dir.path(), "a.wav", 44_100);
    r.start(request(1, a.clone(), pcm16(44_100)));
    // The 3 s track plays to its end, far into the old bus's frame clock.
    r.run(4 * 44_100);
    assert!(
        r.events
            .iter()
            .any(|e| matches!(e, EngineEvent::ReachedEnd { .. }))
    );
    // The output closes with its last holder, and opens again from frame 0.
    r.act(EngineAction::RemovePlayer { player: P });
    for _ in 0..4 {
        r.clock += Duration::from_millis(10);
        r.tick();
    }
    assert!(r.dac.config().is_none(), "the output closed");
    r.act(EngineAction::AddPlayer { player: P });
    r.settle();
    assert_eq!(r.rate(), 48_000, "opened again at the global rate");
    r.start(request(2, a, pcm16(44_100)));
    assert_eq!(
        r.rate(),
        44_100,
        "nothing sounded on this stream: the start reopens at once"
    );
}

#[test]
fn br17_a_bogus_hold_waits_at_most_the_configured_play_out() {
    let mut r = rig(true, true);
    // A saturated measurement: hours at 44.1 kHz.
    r.dac.set_latency_frames(u32::MAX);
    let a = wav(r.dir.path(), "a.wav", 44_100);
    let b = wav(r.dir.path(), "b.wav", 48_000);
    r.start(request(1, a, pcm16(44_100)));
    r.run(4_410);
    r.act(EngineAction::StopNow { player: P });
    r.run(BLOCK);
    r.start(request(2, b, pcm16(48_000)));
    // `tuning.play_out_max_ms` is 500 ms by default: 22 050 frames.
    let silence = silence_before_rate(&mut r, 48_000, 2 * 44_100);
    assert!(silence >= 22_050, "{silence} frames");
    assert!(silence < 22_050 + 3 * BLOCK, "{silence} frames: no longer");
    r.settle();
    let heard = r.run(9_600);
    assert!(heard.iter().any(|v| *v != 0.0), "it plays");
}

fn cart_at(r: &Rig, rate: u32) -> EngineAction {
    EngineAction::StartCart(CartRequest {
        cart: CartId(1),
        track: TrackId(9),
        path: wav(r.dir.path(), "j48.wav", rate),
        from_secs: 0.0,
        until_secs: f64::INFINITY,
        looped: false,
        format: pcm16(rate),
        normalization: fp_model::Normalization::default(),
    })
}

/// Plays a 44.1 kHz track to its end and, as soon as the engine reports
/// it, does `then`; renders until the dac runs at `rate` (at most 2 s
/// more). Returns the silent frames rendered after the track's last sound
/// before the old stream closed.
fn silence_after_the_end_before_rate(
    r: &mut Rig,
    rate: u32,
    then: impl FnOnce(&mut Rig),
) -> Option<usize> {
    let a = wav(r.dir.path(), "a.wav", 44_100);
    r.start(request(1, a, pcm16(44_100)));
    let mut then = Some(then);
    let mut trailing = 0;
    for _ in 0..(5 * 44_100) / BLOCK {
        if r.rate() == rate {
            return Some(trailing);
        }
        for v in r.run(BLOCK) {
            trailing = if v == 0.0 { trailing + 1 } else { 0 };
        }
        let ended = r
            .events
            .iter()
            .any(|e| matches!(e, EngineEvent::ReachedEnd { .. }));
        if ended && let Some(then) = then.take() {
            then(r);
        }
    }
    None
}

#[test]
fn br17_a_cart_fired_right_after_a_track_ended_reopens_once_the_end_has_played() {
    let mut r = rig(true, true);
    r.dac.set_latency_frames(HELD as u32);
    let silence = silence_after_the_end_before_rate(&mut r, 48_000, |r| {
        let cart = cart_at(r, 48_000);
        r.act(cart);
    })
    .expect("the cart's rate is taken");
    assert!(silence >= HELD, "{silence} frames: the end was cut");
    assert!(silence < HELD + 3 * BLOCK, "{silence} frames: no longer");
    r.settle();
    let heard = r.run(9_600);
    assert!(heard.iter().any(|v| *v != 0.0), "the cart plays");
    let (carts, _) = r.engine.cart_telemetry();
    assert_eq!(carts.len(), 1, "on air");
}

#[test]
fn br17_a_cart_stopped_while_the_end_plays_out_never_starts() {
    let mut r = rig(true, true);
    r.dac.set_latency_frames(HELD as u32);
    let reopened = silence_after_the_end_before_rate(&mut r, 48_000, |r| {
        let cart = cart_at(r, 48_000);
        r.act(cart);
        r.act(EngineAction::StopCart { cart: CartId(1) });
    });
    assert_eq!(reopened, None, "no reopen for a cart that will not play");
    let heard = r.run(9_600);
    assert!(heard.iter().all(|v| *v == 0.0), "silence");
    assert!(r.engine.cart_telemetry().0.is_empty());
}

#[test]
fn br17_a_track_loaded_paused_and_resumed_after_a_stop_reopens_once_the_declick_has_played() {
    let mut r = rig(true, true);
    r.dac.set_latency_frames(HELD as u32);
    let a = wav(r.dir.path(), "a.wav", 44_100);
    let b = wav(r.dir.path(), "b.wav", 48_000);
    r.start(request(1, a, pcm16(44_100)));
    r.run(4_410);
    r.act(EngineAction::StopNow { player: P });
    r.act(EngineAction::LoadPaused {
        player: P,
        request: request(2, b, pcm16(48_000)),
    });
    r.settle();
    r.run(BLOCK);
    r.act(EngineAction::Resume { player: P });
    let silence = silence_before_rate(&mut r, 48_000, 44_100);
    assert!(silence >= HELD, "{silence} frames: the declick was cut");
    assert!(silence < HELD + 3 * BLOCK, "{silence} frames: no longer");
    r.settle();
    let heard = r.run(9_600);
    assert!(heard.iter().any(|v| *v != 0.0), "it plays");
    assert!(r.engine.telemetry(P).bit_perfect, "at its own rate");
}

#[test]
fn br8_at_the_same_rate_the_hard_start_does_not_reopen() {
    let mut r = rig(true, true);
    let a = wav(r.dir.path(), "a.wav", 44_100);
    let b = wav(r.dir.path(), "b.wav", 44_100);
    r.start(request(1, a, pcm16(44_100)));
    r.run(4_410);
    let attempts = r.dac.open_attempts();
    r.start(request(2, b, pcm16(44_100)));
    r.run(4_410);
    assert_eq!(r.dac.open_attempts(), attempts);
    assert!(bit_perfect_after(&mut r, 4_410));
}

#[test]
fn br8_a_crossfade_at_another_rate_does_not_reopen() {
    let mut r = started_over(crossfade);
    r.settle();
    r.run(9_600);
    assert_eq!(r.rate(), 44_100, "BR11: a sounding bus keeps its rate");
    assert!(!r.engine.telemetry(P).bit_perfect, "resampled");
}

#[test]
fn br8_stop_in_the_gap_starts_nothing() {
    let mut r = started_over(hard_start);
    r.act(EngineAction::StopNow { player: P });
    r.settle();
    let heard = r.run(9_600);
    assert_eq!(r.rate(), 44_100, "no reopen for a track that will not play");
    assert!(heard.iter().skip(BLOCK * 2).all(|v| *v == 0.0), "silence");
}

#[test]
fn br8_a_cart_fired_in_the_gap_keeps_the_rate_and_the_track_plays_at_it() {
    let mut r = started_over(hard_start);
    assert_eq!(
        decode_rate(&r, "cur ["),
        Some(48_000),
        "prepared for the cut"
    );
    let cart = jingle(&r);
    r.act(cart);
    r.settle();
    r.run(BLOCK);
    r.settle();
    r.run(4_410);
    assert_eq!(r.rate(), 44_100, "the cart sounds: no reopen");
    assert_eq!(
        decode_rate(&r, "cur ["),
        Some(44_100),
        "opened again at the stream rate, never played at the wrong speed"
    );
    let position = r.engine.telemetry(P).position_secs.unwrap();
    assert!(position > 0.05, "it plays: {position}");
}

#[test]
fn br8_a_rate_refused_in_the_gap_falls_back_and_the_track_plays() {
    let mut r = started_over(hard_start);
    r.dac.refuse_rate(48_000);
    r.settle();
    r.run(BLOCK);
    r.settle();
    assert_eq!(r.rate(), 44_100, "B3.4: back to the previous rate");
    assert_eq!(r.stream_rates().last(), Some(&(44_100, vec![48_000])));
    assert_eq!(decode_rate(&r, "cur ["), Some(44_100), "at the stream rate");
    let heard = r.run(4_410);
    assert!(heard.iter().any(|v| *v != 0.0), "it plays, resampled");
}

fn seek(secs: f64) -> EngineAction {
    EngineAction::Seek { player: P, secs }
}

/// Renders until the dac reopens at `rate` (at most `frames`), then
/// `after` more frames; the left channel heard after the reopen.
fn heard_after_reopen(r: &mut Rig, rate: u32, frames: usize, after: usize) -> Vec<f32> {
    for _ in 0..frames.div_ceil(BLOCK) {
        if r.rate() == rate {
            break;
        }
        r.run(BLOCK);
        r.settle();
    }
    assert_eq!(r.rate(), rate, "the dac reopened");
    r.run(after)
}

/// Asserts that `heard` plays an `indexed_wav` file from frame `from` on
/// (after leading silence): a start inside the audio ramps in over the
/// declick (`ramp` frames, the first at gain 0), then every sample is the
/// file's value, bit for bit.
fn plays_bit_exact_from(heard: &[f32], from: usize, ramp: usize) {
    let first = heard
        .iter()
        .position(|v| *v != 0.0)
        .expect("the track plays");
    assert!(
        first > 0,
        "the ramp starts at gain 0, so the first heard sample cannot be the first one"
    );
    let start = first - 1;
    let exact = &heard[start + ramp..];
    assert!(exact.len() > 1_000, "enough heard after the ramp");
    for (i, v) in exact.iter().enumerate() {
        let index = (from + ramp + i) % 20_000;
        let expected = index as f32 / 32_768.0;
        assert!(
            v.to_bits() == expected.to_bits(),
            "sample {i}: {v} (index {}) is not index {index}",
            support::index_of(*v),
        );
    }
}

/// The default declick (5 ms) in frames at 48 kHz.
const RAMP_48K: usize = 240;

#[test]
fn br8_a_seek_in_the_gap_keeps_the_cut_and_plays_bit_perfect_from_there() {
    let mut r = started_over(hard_start);
    r.act(seek(1.0));
    let position = r.engine.telemetry(P).position_secs.unwrap();
    assert!((position - 1.0).abs() < 1e-9, "the seek point: {position}");
    r.settle();
    assert_eq!(
        decode_rate(&r, "cur ["),
        Some(48_000),
        "still prepared for the cut"
    );
    let heard = heard_after_reopen(&mut r, 48_000, 9_600, 4_800);
    assert_eq!(r.stream_rates().last(), Some(&(48_000, vec![])));
    // Frame 48 000 of the file (1 s at 48 kHz) first, ramped in over the
    // declick, then bit for bit.
    plays_bit_exact_from(&heard, 48_000, RAMP_48K);
    assert!(r.engine.telemetry(P).bit_perfect, "the badge is lit");
    let position = r.engine.telemetry(P).position_secs.unwrap();
    assert!(position > 1.05, "it plays on from the seek: {position}");
}

#[test]
fn br17_a_seek_in_the_resume_gap_waits_at_the_files_rate_and_plays_bit_perfect_from_there() {
    let mut r = rig(true, true);
    r.dac.set_latency_frames(HELD as u32);
    let a = wav(r.dir.path(), "a.wav", 44_100);
    let b = wav(r.dir.path(), "b.wav", 48_000);
    r.start(request(1, a, pcm16(44_100)));
    r.run(4_410);
    r.act(EngineAction::StopNow { player: P });
    r.act(EngineAction::LoadPaused {
        player: P,
        request: request(2, b, pcm16(48_000)),
    });
    r.settle();
    r.run(BLOCK);
    r.act(EngineAction::Resume { player: P });
    // Loaded paused at the bus's old rate; the resume waits for the
    // declick to play out before the dac reopens at the file's rate.
    r.act(seek(1.0));
    r.settle();
    assert_eq!(r.rate(), 44_100, "still in the gap");
    assert_eq!(
        decode_rate(&r, "cur ["),
        Some(48_000),
        "the source from the seek waits at the rate of the cut"
    );
    let heard = heard_after_reopen(&mut r, 48_000, 44_100, 4_800);
    assert_eq!(r.stream_rates().last(), Some(&(48_000, vec![])));
    plays_bit_exact_from(&heard, 48_000, RAMP_48K);
    assert!(r.engine.telemetry(P).bit_perfect, "the badge is lit");
}

/// `settle` waits for a source that waits idle in the gap as for any
/// other: after a seek there, its decode is done before the test goes on.
#[test]
fn br8_settle_waits_for_the_source_sought_in_the_gap() {
    let mut r = started_over(hard_start);
    r.act(seek(1.0));
    r.settle();
    let d = r.engine.describe_player(P);
    let current = d.split("cur [").nth(1).unwrap().split(']').next().unwrap();
    assert!(current.contains("start Idle"), "still in the gap: {d}");
    assert!(
        current.contains("ready true"),
        "settled before its decode: {d}"
    );
}

#[test]
fn br8_two_seeks_in_the_gap_the_last_one_wins() {
    let mut r = started_over(hard_start);
    r.act(seek(1.0));
    r.settle();
    r.act(seek(2.0));
    r.settle();
    let heard = heard_after_reopen(&mut r, 48_000, 9_600, 4_800);
    plays_bit_exact_from(&heard, 96_000, RAMP_48K);
    assert!(r.engine.telemetry(P).bit_perfect);
}

#[test]
fn br8_a_seek_in_the_gap_then_a_refused_rate_falls_back_and_plays_from_the_seek() {
    let mut r = started_over(hard_start);
    r.act(seek(1.0));
    r.dac.refuse_rate(48_000);
    r.settle();
    r.run(BLOCK);
    r.settle();
    assert_eq!(r.rate(), 44_100, "B3.4: back to the previous rate");
    assert_eq!(r.stream_rates().last(), Some(&(44_100, vec![48_000])));
    assert_eq!(decode_rate(&r, "cur ["), Some(44_100), "at the stream rate");
    let heard = r.run(4_410);
    assert!(heard.iter().any(|v| *v != 0.0), "it plays, resampled");
    let position = r.engine.telemetry(P).position_secs.unwrap();
    assert!(
        (1.05..1.5).contains(&position),
        "it plays from the seek: {position}"
    );
}

#[test]
fn br8_a_cart_fired_after_a_seek_in_the_gap_keeps_the_rate_and_plays_from_the_seek() {
    let mut r = started_over(hard_start);
    r.act(seek(1.0));
    r.settle();
    let cart = jingle(&r);
    r.act(cart);
    r.settle();
    r.run(BLOCK);
    r.settle();
    r.run(4_410);
    assert_eq!(r.rate(), 44_100, "the cart sounds: no reopen");
    assert_eq!(decode_rate(&r, "cur ["), Some(44_100));
    let position = r.engine.telemetry(P).position_secs.unwrap();
    assert!(
        (1.05..1.5).contains(&position),
        "it plays from the seek: {position}"
    );
}

#[test]
fn br8_a_stop_after_a_seek_in_the_gap_starts_nothing() {
    let mut r = started_over(hard_start);
    r.act(seek(1.0));
    r.act(EngineAction::StopNow { player: P });
    r.settle();
    let heard = r.run(9_600);
    assert_eq!(r.rate(), 44_100, "no reopen for a track that will not play");
    assert!(heard.iter().skip(BLOCK * 2).all(|v| *v == 0.0), "silence");
}

#[test]
fn br8_a_second_start_in_the_gap_replaces_the_first() {
    // At the stream rate it starts at once; at another rate it waits for a
    // cut of its own. Either way it plays at its own rate.
    for rate in [44_100, 96_000] {
        let mut r = started_over(hard_start);
        let c = wav(r.dir.path(), "c.wav", rate);
        r.act(hard_start(request(3, c, pcm16(rate))));
        r.settle();
        r.run(BLOCK);
        r.settle();
        assert_eq!(r.rate(), rate, "the second start's own rate");
        assert_eq!(decode_rate(&r, "cur ["), Some(rate));
        assert!(bit_perfect_after(&mut r, 4_800), "{rate}");
    }
}

#[test]
fn br8_a_pause_in_the_gap_resumes_at_the_stream_rate_when_a_cart_sounds() {
    let mut r = started_over(hard_start);
    r.act(EngineAction::Pause { player: P });
    r.run(BLOCK);
    let cart = jingle(&r);
    r.act(cart);
    r.settle();
    r.run(BLOCK);
    r.act(EngineAction::Resume { player: P });
    r.settle();
    assert_eq!(r.rate(), 44_100, "the cart sounds: no reopen");
    assert_eq!(
        decode_rate(&r, "cur ["),
        Some(44_100),
        "opened again at the stream rate, never played at the wrong speed"
    );
}

#[test]
fn br8_next_takes_a_preload_opened_for_a_cut_only_at_its_rate() {
    let mut r = rig(true, true);
    on_air_with_next(&mut r);
    r.act(cut_plan(2.5, Some(48_000)));
    r.settle();
    // The operator presses Next before the planned cut: the same entry.
    let b = r.dir.path().join("b.wav");
    r.act(hard_start(request(2, b, pcm16(48_000))));
    r.settle();
    r.run(BLOCK);
    r.settle();
    assert_eq!(r.rate(), 48_000);
    assert!(
        bit_perfect_after(&mut r, 4_800),
        "played at its own rate, at the right speed"
    );
}

#[test]
fn br8_without_a_cut_next_does_not_take_a_preload_at_another_rate() {
    let mut r = rig(true, true);
    on_air_with_next(&mut r);
    r.act(cut_plan(2.5, Some(48_000)));
    r.settle();
    assert_eq!(decode_rate(&r, "pre ["), Some(48_000));
    // A cart sounds on the device: the hard start keeps the rate (D2).
    let cart = jingle(&r);
    r.act(cart);
    r.settle();
    let b = r.dir.path().join("b.wav");
    r.act(hard_start(request(2, b, pcm16(48_000))));
    assert_eq!(
        decode_rate(&r, "cur ["),
        Some(44_100),
        "opened again at the stream rate, never played at the wrong speed"
    );
    r.settle();
    r.run(4_410);
    assert_eq!(r.rate(), 44_100);
}

#[test]
fn br13_a_kept_device_change_waits_through_the_gap() {
    let mut r = started_over(hard_start);
    let running = r.engine.running_settings(&dac()).unwrap();
    r.act(EngineAction::ApplyDevice {
        device: dac(),
        settings: fp_model::DeviceSettings {
            buffer_frames: 960,
            ..running
        },
        force: false,
    });
    r.settle();
    r.run(BLOCK);
    let applied = |r: &Rig| {
        r.events
            .iter()
            .any(|e| matches!(e, EngineEvent::Applied { .. }))
    };
    assert!(
        !applied(&r),
        "not in the gap, nor while the new track plays"
    );
    r.act(EngineAction::StopNow { player: P });
    r.run(BLOCK * 4);
    assert!(applied(&r), "once the device is quiet");
}

#[test]
fn br8_next_over_a_paused_track_reopens_and_a_device_change_waits() {
    let mut r = rig(true, true);
    let a = wav(r.dir.path(), "a.wav", 44_100);
    let b = wav(r.dir.path(), "b.wav", 48_000);
    r.start(request(1, a, pcm16(44_100)));
    r.run(4_410);
    r.act(EngineAction::Pause { player: P });
    r.run(BLOCK * 4);
    // The paused track is the one Next replaces: it does not keep the rate.
    r.act(hard_start(request(2, b, pcm16(48_000))));
    let running = r.engine.running_settings(&dac()).unwrap();
    r.act(EngineAction::ApplyDevice {
        device: dac(),
        settings: fp_model::DeviceSettings {
            buffer_frames: 960,
            ..running
        },
        force: false,
    });
    let applied = |r: &Rig| {
        r.events
            .iter()
            .any(|e| matches!(e, EngineEvent::Applied { .. }))
    };
    r.run(BLOCK);
    r.settle();
    assert!(
        !applied(&r),
        "not in the gap, nor while the new track plays"
    );
    assert_eq!(r.rate(), 48_000, "the cut reopened the device");
    assert!(bit_perfect_after(&mut r, 4_800));
    r.act(EngineAction::StopNow { player: P });
    r.run(BLOCK * 4);
    assert!(applied(&r), "once the device is quiet");
}

#[test]
fn br14_bit_perfect_turned_off_reopens_shared_at_the_configured_rate_and_reports_it() {
    let mut r = rig(true, true);
    let a = wav(r.dir.path(), "a.wav", 44_100);
    r.start(request(1, a, pcm16(44_100)));
    r.run(4_410);
    r.act(EngineAction::StopNow { player: P });
    r.run(BLOCK * 4);
    let running = r.engine.running_settings(&dac()).unwrap();
    r.act(EngineAction::ApplyDevice {
        device: dac(),
        settings: fp_model::DeviceSettings {
            bit_perfect: false,
            ..running
        },
        force: false,
    });
    r.run(BLOCK);
    assert_eq!(r.rate(), 48_000, "effective_rate");
    assert!(!r.dac.config().unwrap().exclusive);
    assert_eq!(r.stream_rates().last(), Some(&(48_000, vec![])));
}

#[test]
fn n16_a_normalization_gain_is_not_bit_perfect() {
    let mut r = rig(true, true);
    let path = wav(r.dir.path(), "a.wav", 48_000);
    let mut req = request(1, path, pcm16(48_000));
    req.normalization = Normalization {
        gain_db: -3.0,
        status: NormalizationStatus::Applied { limited: None },
    };
    r.start(req);
    assert!(!bit_perfect_after(&mut r, 2_400));
}

#[test]
fn n16_no_gain_stays_bit_perfect() {
    // `request` carries `Normalization::default()`: 0 dB, a factor of 1.0.
    let mut r = rig(true, true);
    let path = wav(r.dir.path(), "a.wav", 48_000);
    r.start(request(1, path, pcm16(48_000)));
    assert!(bit_perfect_after(&mut r, 2_400));
}

/// CA7: a pre-listen on the on-air pair of an idle bit-perfect device is
/// resampled to the running rate; it never moves the rate.
#[test]
fn ca7_an_on_air_cue_plays_at_the_running_rate() {
    let mut r = rig_routed(true, true, dac_route(0), Some(dac_route(0)));
    assert_eq!(r.rate(), 48_000);
    let path = wav(r.dir.path(), "cue.wav", 44_100);
    r.cue(request(1, path, pcm16(44_100)));
    assert_eq!(r.rate(), 48_000, "the pre-listen never moves the rate");
    let heard = r.run(4_800);
    assert!(heard.iter().any(|v| *v != 0.0), "it is heard, resampled");
}

/// A pre-listen alone on a bit-perfect device (Main elsewhere) still
/// follows its file's rate (B3.2).
#[test]
fn a_cue_alone_on_a_bit_perfect_device_still_follows_its_file() {
    let speakers = Route {
        backend: "offline".into(),
        device: "speakers".into(),
        first_channel: 0,
    };
    let mut r = rig_routed(true, true, speakers, Some(dac_route(0)));
    let path = wav(r.dir.path(), "cue.wav", 44_100);
    r.cue(request(1, path, pcm16(44_100)));
    assert_eq!(r.rate(), 44_100);
}

/// CA5 in the engine: the model sends the pre-listen's stop and the track's
/// start in one batch; the track starts and plays at real-time speed,
/// whatever rate the bus keeps under the CUE's tail.
#[test]
fn ca5_a_track_started_as_its_pre_listen_stops_plays() {
    let mut r = rig_routed(true, true, dac_route(0), Some(dac_route(0)));
    let cue = wav(r.dir.path(), "cue.wav", 48_000);
    r.cue(request(1, cue, pcm16(48_000)));
    r.run(4_800);
    let track = wav(r.dir.path(), "track.wav", 44_100);
    r.act(EngineAction::StopCue { player: P });
    r.start(request(2, track, pcm16(44_100)));
    let rate = r.rate();
    let heard = r.run(rate as usize);
    assert!(heard.iter().any(|v| *v != 0.0), "the track is heard");
    let position = r.engine.telemetry(P).position_secs.unwrap();
    assert!(
        (position - 1.0).abs() < 0.05,
        "one second plays as one second at {rate} Hz: {position}"
    );
}
