#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Operator feedback 4, Q12.5: each device opens at its own rate and
//! buffer; a bit-perfect device starts at its own rate and still follows
//! the file (bit-perfect spec B3).

mod support;

use std::path::PathBuf;
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
    phones: OfflineDevice,
    clock: Instant,
    dir: tempfile::TempDir,
}

fn route(device: &str) -> Route {
    Route {
        backend: "offline".into(),
        device: device.into(),
        first_channel: 0,
    }
}

fn dev(device: &str) -> OutputDevice {
    OutputDevice {
        backend: "offline".into(),
        device: device.into(),
    }
}

/// Player 1 plays on `dac` (exclusive-capable) and pre-listens on `phones`;
/// `edit` sets the devices' own values.
fn rig(edit: impl FnOnce(&mut Config)) -> Rig {
    let backend = OfflineBackend::new();
    let dac = backend.add_device("dac", 2);
    dac.set_exclusive_capable(true);
    let phones = backend.add_device("phones", 2);
    let mut config = Config::default();
    config.outputs.backend = Some("offline".into());
    config.outputs.routes = vec![PlayerRoutes {
        player: P,
        main: Some(route("dac")),
        cue: Some(route("phones")),
    }];
    config.tuning.gain_smoothing_ms = 0.0;
    config.tuning.prebuffer_secs = 4.0;
    config.tuning.ready_threshold_ms = 1_500.0;
    edit(&mut config);
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
        phones,
        clock,
        dir: tempfile::tempdir().unwrap(),
    }
}

fn pcm16(rate: u32) -> Option<AudioFormat> {
    Some(AudioFormat {
        sample_rate: rate,
        bits: Some(16),
        channels: 2,
        dsd_rate: None,
    })
}

fn request(path: PathBuf, format: Option<AudioFormat>) -> SourceRequest {
    SourceRequest {
        entry: EntryId(1),
        track: TrackId(1),
        path,
        from_secs: 0.0,
        format,
    }
}

impl Rig {
    /// Starts `request` and waits for the decode worker (never for audio),
    /// as in `bit_perfect.rs`.
    fn start(&mut self, request: SourceRequest) {
        self.engine.execute(
            EngineAction::StartCurrent { player: P, request },
            self.clock,
        );
        let deadline = Instant::now() + Duration::from_secs(5);
        while self.engine.unsettled_sources() > 0 && Instant::now() < deadline {
            self.engine.tick(self.clock);
            std::thread::sleep(Duration::from_millis(1));
        }
        self.engine.tick(self.clock);
    }

    /// Renders `frames` frames of `dac` in blocks, ticking after each.
    fn run(&mut self, frames: usize) {
        let rate = self.dac.config().unwrap().sample_rate;
        for _ in 0..frames.div_ceil(BLOCK) {
            self.dac.render(BLOCK).unwrap();
            self.clock += Duration::from_secs_f64(BLOCK as f64 / f64::from(rate));
            self.engine.tick(self.clock);
        }
    }
}

#[test]
fn a_device_opens_at_its_own_rate_and_buffer() {
    let r = rig(|c| {
        c.outputs.set_device_rate(&dev("dac"), Some(96_000));
        c.outputs.set_device_buffer(&dev("dac"), Some(256));
    });
    let dac = r.dac.config().unwrap();
    assert_eq!((dac.sample_rate, dac.buffer_frames), (96_000, 256));
    let phones = r.phones.config().unwrap();
    assert_eq!(
        (phones.sample_rate, phones.buffer_frames),
        (48_000, 512),
        "a device without its own values opens with the global ones"
    );
}

#[test]
fn only_the_buffer_can_be_a_devices_own() {
    let r = rig(|c| c.outputs.set_device_buffer(&dev("phones"), Some(1024)));
    let phones = r.phones.config().unwrap();
    assert_eq!((phones.sample_rate, phones.buffer_frames), (48_000, 1024));
}

#[test]
fn a_source_on_a_device_with_its_own_rate_keeps_time() {
    let mut r = rig(|c| c.outputs.set_device_rate(&dev("dac"), Some(96_000)));
    let path = indexed_wav(r.dir.path(), "a.wav", 48_000, 2, 48_000 * 3);
    r.start(request(path, pcm16(48_000)));
    r.run(96_000);
    let position = r.engine.telemetry(P).position_secs.unwrap();
    assert!(
        (position - 1.0).abs() < 0.05,
        "96 000 frames at 96 kHz are one second: {position}"
    );
}

#[test]
fn a_bit_perfect_device_starts_at_its_own_rate_and_still_follows_the_file() {
    let mut r = rig(|c| {
        c.outputs.bit_perfect = vec![dev("dac")];
        c.outputs.set_device_rate(&dev("dac"), Some(96_000));
    });
    assert_eq!(r.dac.config().unwrap().sample_rate, 96_000);
    let path = indexed_wav(r.dir.path(), "a.wav", 44_100, 2, 44_100 * 3);
    r.start(request(path, pcm16(44_100)));
    assert_eq!(r.dac.config().unwrap().sample_rate, 44_100);
}
