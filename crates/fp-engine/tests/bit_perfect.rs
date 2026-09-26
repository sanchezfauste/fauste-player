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
    AudioFormat, Config, EngineAction, EntryId, OutputDevice, PlayerId, PlayerRoutes, Route,
    SourceRequest, TrackId,
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
        "one second at 44.1 kHz is one second: {position}"
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
