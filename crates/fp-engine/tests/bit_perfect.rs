#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Bit-perfect buses (Phase 4 spec B3–B5): the stream rate follows the
//! file while the bus is idle, and untouched sources reach the device
//! unchanged.

mod support;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use fp_backends::{AudioBackend, OfflineBackend, OfflineDevice};
use fp_engine::engine::{Engine, EngineSettings};
use fp_engine::worker::file_opener;
use fp_model::{
    AudioFormat, CartId, CartRequest, Config, EngineAction, EntryId, OutputDevice, PlayerId,
    PlayerRoutes, Route, SourceRequest, TrackId,
};
use support::indexed_wav;

const P: PlayerId = PlayerId(1);
const BLOCK: usize = 480;

struct Rig {
    engine: Engine,
    dac: OfflineDevice,
    clock: Instant,
    dir: tempfile::TempDir,
}

fn rig(bit_perfect: bool, exclusive_capable: bool) -> Rig {
    let backend = OfflineBackend::new();
    let dac = backend.add_device("dac", 2);
    dac.set_exclusive_capable(exclusive_capable);
    let mut config = Config::default();
    config.outputs.backend = Some("offline".into());
    config.outputs.buffer_frames = BLOCK as u32;
    config.outputs.routes = vec![PlayerRoutes {
        player: P,
        main: Some(Route {
            backend: "offline".into(),
            device: "dac".into(),
            first_channel: 0,
        }),
        cue: None,
    }];
    if bit_perfect {
        config.outputs.bit_perfect = vec![OutputDevice {
            backend: "offline".into(),
            device: "dac".into(),
        }];
    }
    config.tuning.gain_smoothing_ms = 0.0;
    // Start only with more buffered than a test renders at once, so a test
    // that renders faster than real time never outruns the worker.
    config.tuning.prebuffer_secs = 4.0;
    config.tuning.ready_threshold_ms = 1_500.0;
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
    }
}

fn pcm16(rate: u32) -> Option<AudioFormat> {
    Some(AudioFormat {
        sample_rate: rate,
        bits: Some(16),
        channels: 2,
    })
}

impl Rig {
    fn act(&mut self, action: EngineAction) {
        self.engine.execute(action, self.clock);
    }

    fn start(&mut self, request: SourceRequest) {
        self.act(EngineAction::StartCurrent { player: P, request });
        self.settle();
    }

    fn settle(&mut self) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while self.engine.unsettled_sources() > 0 && Instant::now() < deadline {
            self.engine.tick(self.clock);
            std::thread::sleep(Duration::from_millis(1));
        }
        self.engine.tick(self.clock);
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
            self.engine.tick(self.clock);
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
