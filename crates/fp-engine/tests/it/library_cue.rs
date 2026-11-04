#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Library plan 4 (LB32, LB36): the library CUE is placed from the start,
//! without a stream or a worker, and plays a track on its own route.

use crate::support;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use fp_backends::{AudioBackend, OfflineBackend, OfflineDevice};
use fp_engine::engine::{Engine, EngineSettings};
use fp_model::{
    AudioFormat, Config, EngineAction, EngineEvent, Holder, LibraryCueRequest, Normalization,
    NormalizationStatus, Route, Target, TrackId,
};
use support::tagged_opener;

fn route() -> Route {
    Route {
        backend: "offline".into(),
        device: "card".into(),
        first_channel: 0,
    }
}

fn rig(library: Option<Route>) -> (Engine, Config, Vec<EngineEvent>) {
    let backend = OfflineBackend::new();
    backend.add_device("card", 2);
    let mut config = Config::default();
    config.outputs.backend = Some("offline".into());
    config.outputs.library.cue = library;
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
    let mut engine = Engine::new(
        backends,
        EngineSettings::from_config(&config),
        tagged_opener(48_000),
    );
    let events = engine.tick(Instant::now());
    (engine, config, events)
}

#[test]
fn lb32_the_library_cue_is_placed_from_the_start() {
    let (_, _, events) = rig(Some(route()));
    assert!(
        events.iter().any(|e| matches!(
            e,
            EngineEvent::Placed { holder: Holder::LibraryCue, route: Some(r), .. } if *r == route()
        )),
        "{events:?}"
    );
    let (_, _, events) = rig(None);
    assert!(
        events.iter().any(|e| matches!(
            e,
            EngineEvent::Unplaced {
                holder: Holder::LibraryCue,
                route: None
            }
        )),
        "{events:?}"
    );
}

#[test]
fn lb32_a_route_set_later_is_reported_without_opening_a_stream() {
    let (mut engine, mut config, _) = rig(None);
    let before = format!("{:?}", engine.bus_status());
    config.outputs.library.cue = Some(route());
    engine.execute(
        EngineAction::UpdateSettings(Box::new(config)),
        Instant::now(),
    );
    let events = engine.tick(Instant::now());
    assert!(
        events.iter().any(|e| matches!(
            e,
            EngineEvent::Placed { holder: Holder::LibraryCue, route: Some(r), .. } if *r == route()
        )),
        "{events:?}"
    );
    assert_eq!(format!("{:?}", engine.bus_status()), before);
}

const BLOCK: usize = 480;

fn on(device: &str, first_channel: u16) -> Route {
    Route {
        backend: "offline".into(),
        device: device.into(),
        first_channel,
    }
}

fn preview(n: u64) -> LibraryCueRequest {
    LibraryCueRequest {
        track: TrackId(n),
        path: PathBuf::from(format!("track{n}")),
        from_secs: 0.0,
        format: None,
        normalization: Normalization::default(),
    }
}

fn start(request: LibraryCueRequest) -> EngineAction {
    EngineAction::StartLibraryCue {
        request,
        paused: false,
    }
}

/// An engine playing on the Offline device `card`, with the library CUE on
/// `library`; every `track<N>` lasts `track_frames`.
struct Play {
    engine: Engine,
    device: OfflineDevice,
    channels: usize,
    clock: Instant,
    frames: Vec<Vec<f32>>,
    events: Vec<EngineEvent>,
}

fn play_rig(channels: u16, library: Option<Route>, track_frames: u64) -> Play {
    play_rig_with(channels, library, track_frames, |_| {})
}

/// As `play_rig`, with `adjust` applied to the configuration (and `card`
/// exclusive-capable, for a bit-perfect `card`).
fn play_rig_with(
    channels: u16,
    library: Option<Route>,
    track_frames: u64,
    adjust: impl FnOnce(&mut Config),
) -> Play {
    let backend = OfflineBackend::new();
    let device = backend.add_device("card", channels);
    device.set_exclusive_capable(true);
    backend.add_device("other", 2);
    let mut config = Config::default();
    config.outputs.buffer_frames = BLOCK as u32;
    config.outputs.backend = Some("offline".into());
    config.outputs.library.cue = library;
    config.tuning.gain_smoothing_ms = 0.0;
    adjust(&mut config);
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
    let engine = Engine::new(
        backends,
        EngineSettings::from_config(&config),
        tagged_opener(track_frames),
    );
    Play {
        engine,
        device,
        channels: usize::from(channels),
        clock: Instant::now(),
        frames: Vec::new(),
        events: Vec::new(),
    }
}

impl Play {
    fn act(&mut self, action: EngineAction) {
        self.engine.execute(action, self.clock);
    }

    fn settle(&mut self) {
        let deadline = Instant::now() + Duration::from_secs(60);
        loop {
            let events = self.engine.tick(self.clock);
            self.events.extend(events);
            if self.engine.unsettled_sources() == 0 {
                return;
            }
            assert!(Instant::now() < deadline, "the sources never settled");
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    fn run(&mut self, blocks: usize) {
        for _ in 0..blocks {
            // A device whose bus was closed (a moved route) renders nothing.
            if let Some(out) = self.device.render(BLOCK) {
                self.frames
                    .extend(out.chunks(self.channels).map(<[f32]>::to_vec));
            }
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

    fn ends(&self) -> Vec<TrackId> {
        self.events
            .iter()
            .filter_map(|e| match e {
                EngineEvent::LibraryCueEnded { track } => Some(*track),
                _ => None,
            })
            .collect()
    }
}

fn audible(samples: &[f32]) -> usize {
    samples.iter().filter(|v| **v != 0.0).count()
}

#[test]
fn lb32_a_preview_plays_only_on_the_library_cue_route() {
    let mut r = play_rig(4, Some(on("card", 2)), 48_000);
    r.act(start(preview(1)));
    r.settle();
    r.run(10);
    assert_eq!(audible(&r.channel(0)) + audible(&r.channel(1)), 0);
    assert!(audible(&r.channel(2)) > 0);
    assert!(audible(&r.channel(3)) > 0);
    let (track, position) = r.engine.library_cue_position().unwrap();
    assert_eq!(track, TrackId(1));
    assert!(position > 0.0, "{position}");
    assert!(r.ends().is_empty(), "{:?}", r.events);
}

#[test]
fn lb32_without_a_route_a_preview_ends_at_once() {
    let mut r = play_rig(2, None, 48_000);
    r.act(start(preview(1)));
    let events = r.engine.tick(r.clock);
    assert!(
        events.contains(&EngineEvent::LibraryCueEnded { track: TrackId(1) }),
        "{events:?}"
    );
    assert_eq!(r.engine.used_slots(), 0);
}

/// Plays a 0.1 s `track1` to its end; the whole file is heard and the end
/// is reported once, after its last frame.
fn plays_to_the_end(format: Option<AudioFormat>) {
    let mut r = play_rig(2, Some(on("card", 0)), 4_800);
    r.act(start(LibraryCueRequest {
        format,
        ..preview(1)
    }));
    r.settle();
    let mut ended_at = None;
    for block in 0..30 {
        r.run(1);
        if ended_at.is_none() && !r.ends().is_empty() {
            ended_at = Some(block);
        }
    }
    assert_eq!(r.ends(), vec![TrackId(1)], "{:?}", r.events);
    let left = r.channel(0);
    let last = left.iter().rposition(|v| *v != 0.0).unwrap();
    assert_eq!(left[last], 104_799.0, "the last frame of the file");
    assert_eq!(audible(&left), 4_800);
    let ended_at = ended_at.unwrap();
    assert!(
        (ended_at + 1) * BLOCK > last,
        "reported in block {ended_at}, before frame {last}"
    );
    assert_eq!(r.engine.library_cue_position(), None);
    assert_eq!(r.engine.used_slots(), 0);
}

#[test]
fn lb36_the_preview_plays_to_the_end_of_the_file_and_reports_it() {
    plays_to_the_end(Some(AudioFormat {
        sample_rate: 48_000,
        bits: Some(16),
        channels: 2,
        dsd_rate: None,
        bitrate: None,
    }));
}

#[test]
fn without_a_known_length_the_preview_plays_to_the_source_end() {
    plays_to_the_end(None);
}

#[test]
fn stop_ramps_down_and_releases_its_slot() {
    let mut r = play_rig(2, Some(on("card", 0)), 48_000);
    let before = r.engine.used_slots();
    r.act(start(preview(1)));
    r.settle();
    r.run(5);
    r.act(EngineAction::StopLibraryCue);
    r.run(10);
    let left = r.channel(0);
    let last = left.iter().rposition(|v| *v != 0.0).unwrap();
    let peak = left.iter().fold(0.0f32, |m, v| m.max(v.abs()));
    assert!(
        left[last].abs() < peak * 0.1,
        "cut at full level: {} of {peak}",
        left[last]
    );
    assert_eq!(r.engine.used_slots(), before);
    assert_eq!(r.engine.attached_sources(), 0);
    assert_eq!(r.engine.library_cue_position(), None);
    assert!(r.ends().is_empty(), "{:?}", r.events);
}

#[test]
fn a_new_start_replaces_the_preview_without_an_end_event() {
    let mut r = play_rig(2, Some(on("card", 0)), 48_000);
    r.act(start(preview(1)));
    r.settle();
    r.run(5);
    r.act(start(preview(2)));
    r.settle();
    r.run(10);
    assert!(r.ends().is_empty(), "{:?}", r.events);
    let left = r.channel(0);
    assert!(
        *left.last().unwrap() >= 200_000.0,
        "{}",
        left.last().unwrap()
    );
    assert_eq!(
        r.engine.library_cue_position().map(|p| p.0),
        Some(TrackId(2))
    );
    assert_eq!(r.engine.used_slots(), 1);
}

#[test]
fn a_paused_start_stays_silent() {
    let mut r = play_rig(2, Some(on("card", 0)), 48_000);
    r.act(EngineAction::StartLibraryCue {
        request: preview(1),
        paused: true,
    });
    r.settle();
    r.run(10);
    assert_eq!(audible(&r.channel(0)), 0);
    assert_eq!(r.engine.library_cue_position(), Some((TrackId(1), 0.0)));
    assert!(r.ends().is_empty(), "{:?}", r.events);
    r.act(EngineAction::StopLibraryCue);
    r.run(1);
    assert_eq!(r.engine.used_slots(), 0);
}

#[test]
fn a_missing_file_ends_the_preview() {
    let mut r = play_rig(2, Some(on("card", 0)), 48_000);
    r.act(start(LibraryCueRequest {
        path: PathBuf::from("missing1"),
        ..preview(1)
    }));
    r.settle();
    r.run(5);
    assert_eq!(r.ends(), vec![TrackId(1)], "{:?}", r.events);
    assert_eq!(r.engine.used_slots(), 0);
}

#[test]
fn n14_the_preview_plays_with_its_normalization_gain() {
    let mut r = play_rig(2, Some(on("card", 0)), 48_000);
    r.act(start(LibraryCueRequest {
        normalization: Normalization {
            gain_db: -6.0206,
            status: NormalizationStatus::Applied { limited: None },
        },
        ..preview(1)
    }));
    r.settle();
    r.run(10);
    let left = r.channel(0);
    let first = left.iter().position(|v| *v != 0.0).unwrap();
    // Track 1, sample 100 is 100 100; at -6.02 dB it is half of that.
    assert!(
        (left[first + 100] - 50_050.0).abs() < 50.0,
        "{}",
        left[first + 100]
    );
}

#[test]
fn a_route_change_waits_while_the_preview_sounds() {
    let mut r = play_rig(2, Some(on("card", 0)), 48_000);
    r.act(start(preview(1)));
    r.settle();
    r.run(5);
    r.events.clear();
    r.act(EngineAction::ApplyRoute {
        holder: Holder::LibraryCue,
        route: Some(on("other", 0)),
        force: false,
    });
    r.run(3);
    let applied = |events: &[EngineEvent]| {
        events.iter().any(|e| {
            matches!(
                e,
                EngineEvent::Applied {
                    target: Target::Route(Holder::LibraryCue),
                    ..
                }
            )
        })
    };
    assert!(!applied(&r.events), "{:?}", r.events);
    r.act(EngineAction::StopLibraryCue);
    r.run(10);
    assert!(applied(&r.events), "{:?}", r.events);
    assert!(
        r.events.iter().any(|e| matches!(
            e,
            EngineEvent::Placed { holder: Holder::LibraryCue, route: Some(r), .. }
                if *r == on("other", 0)
        )),
        "{:?}",
        r.events
    );
    assert!(r.ends().is_empty(), "{:?}", r.events);
}

/// CA7: a preview on the on-air pair of an idle bit-perfect device is
/// resampled to the running rate; it never moves the rate.
#[test]
fn ca7_an_on_air_preview_never_reopens_a_bit_perfect_bus() {
    let player = fp_model::PlayerId(1);
    let backend = OfflineBackend::new();
    let dac = backend.add_device("dac", 2);
    dac.set_exclusive_capable(true);
    let mut config = Config::default();
    config.outputs.backend = Some("offline".into());
    config.outputs.buffer_frames = BLOCK as u32;
    config.outputs.routes = vec![fp_model::PlayerRoutes {
        player,
        main: Some(on("dac", 0)),
        cue: None,
    }];
    config.outputs.library.cue = Some(on("dac", 0));
    config.outputs.bit_perfect = vec![fp_model::OutputDevice {
        backend: "offline".into(),
        device: "dac".into(),
    }];
    config.tuning.prebuffer_secs = 4.0;
    config.tuning.ready_threshold_ms = 4_000.0;
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
    let mut engine = Engine::new(
        backends,
        EngineSettings::from_config(&config),
        fp_engine::worker::file_opener(),
    );
    let mut clock = Instant::now();
    engine.execute(EngineAction::AddPlayer { player }, clock);
    engine.tick(clock);
    assert_eq!(dac.config().unwrap().sample_rate, 48_000);
    let dir = tempfile::tempdir().unwrap();
    let path = support::indexed_wav(dir.path(), "preview.wav", 44_100, 2, 44_100 * 3);
    engine.execute(
        start(LibraryCueRequest {
            path,
            format: Some(AudioFormat {
                sample_rate: 44_100,
                bits: Some(16),
                channels: 2,
                dsd_rate: None,
                bitrate: None,
            }),
            ..preview(1)
        }),
        clock,
    );
    let deadline = Instant::now() + Duration::from_secs(60);
    while engine.unsettled_sources() > 0 {
        assert!(Instant::now() < deadline, "the preview never settled");
        engine.tick(clock);
        std::thread::sleep(Duration::from_millis(1));
    }
    engine.tick(clock);
    assert_eq!(
        dac.config().unwrap().sample_rate,
        48_000,
        "the preview never moves the rate"
    );
    let mut heard = Vec::new();
    for _ in 0..10 {
        heard.extend(dac.render(BLOCK).unwrap());
        clock += Duration::from_millis(10);
        engine.tick(clock);
    }
    assert!(heard.iter().any(|v| *v != 0.0), "it is heard, resampled");
}

// Task 3: pause, seek, rate changes and the meter (LB33).

/// One block at 48 kHz, in seconds.
const BLOCK_SECS: f64 = BLOCK as f64 / 48_000.0;

fn pause(paused: bool) -> EngineAction {
    EngineAction::SetLibraryCuePaused { paused }
}

impl Play {
    fn position(&self) -> f64 {
        self.engine.library_cue_position().unwrap().1
    }

    /// The left samples of the last `blocks` blocks rendered.
    fn last_blocks(&self, blocks: usize) -> Vec<f32> {
        let left = self.channel(0);
        left[left.len().saturating_sub(blocks * BLOCK)..].to_vec()
    }
}

/// The position (in seconds at `rate`) in track `n` of a tagged sample.
fn tagged_secs(n: u64, sample: f32, rate: u32) -> f64 {
    (f64::from(sample) - (n * 100_000) as f64) / f64::from(rate)
}

#[test]
fn lb33_pause_holds_and_resume_continues_where_it_was() {
    let mut r = play_rig(2, Some(on("card", 0)), 48_000 * 10);
    r.act(start(preview(1)));
    r.settle();
    r.run(10);
    r.act(pause(true));
    r.run(5);
    let held = r.position();
    r.run(10);
    assert_eq!(r.position(), held, "a held preview does not move");
    assert_eq!(audible(&r.last_blocks(9)), 0, "and is silent");
    r.act(pause(false));
    r.run(1);
    let resumed = r.position();
    assert!(
        (resumed - held - BLOCK_SECS).abs() < 1e-9,
        "one block after {held}: {resumed}"
    );
    r.run(5);
    assert!(audible(&r.last_blocks(4)) > 0);
    assert!(r.ends().is_empty(), "{:?}", r.events);
}

#[test]
fn a_pause_before_the_start_holds_the_source_idle() {
    let mut r = play_rig(2, Some(on("card", 0)), 48_000 * 10);
    r.act(start(preview(1)));
    r.act(pause(true));
    r.settle();
    r.run(10);
    assert_eq!(audible(&r.channel(0)), 0);
    assert_eq!(r.engine.library_cue_position(), Some((TrackId(1), 0.0)));
    r.act(pause(false));
    r.settle();
    r.run(5);
    assert!(audible(&r.channel(0)) > 0);
    assert!(r.position() > 0.0);
}

#[test]
fn lb33_a_seek_replaces_the_source_at_the_new_position() {
    let mut r = play_rig(2, Some(on("card", 0)), 48_000 * 10);
    r.act(start(preview(1)));
    r.settle();
    r.run(5);
    r.act(EngineAction::SeekLibraryCue { secs: 5.0 });
    assert_eq!(r.engine.library_cue_position(), Some((TrackId(1), 5.0)));
    r.settle();
    r.run(10);
    let position = r.position();
    assert!(
        (5.0..5.0 + 10.0 * BLOCK_SECS + 1e-9).contains(&position),
        "{position}"
    );
    let last = *r.channel(0).last().unwrap();
    assert!(
        (tagged_secs(1, last, 48_000) - position).abs() <= BLOCK_SECS,
        "heard {last} at {position}"
    );
    assert_eq!(r.engine.used_slots(), 1, "the old source is released");
    assert!(r.ends().is_empty(), "{:?}", r.events);
}

#[test]
fn a_seek_while_held_stays_held_at_the_new_position() {
    let mut r = play_rig(2, Some(on("card", 0)), 48_000 * 10);
    r.act(start(preview(1)));
    r.settle();
    r.run(5);
    r.act(pause(true));
    r.run(5);
    r.act(EngineAction::SeekLibraryCue { secs: 5.0 });
    r.settle();
    r.run(10);
    assert_eq!(audible(&r.last_blocks(10)), 0, "still held");
    assert_eq!(r.engine.library_cue_position(), Some((TrackId(1), 5.0)));
    assert_eq!(r.engine.used_slots(), 1);
    r.act(pause(false));
    r.settle();
    r.run(5);
    assert!(audible(&r.last_blocks(4)) > 0);
    assert!(r.position() > 5.0);
}

fn device_settings(r: &Play) -> fp_model::DeviceSettings {
    r.engine
        .running_settings(&fp_model::OutputDevice {
            backend: "offline".into(),
            device: "card".into(),
        })
        .unwrap()
}

#[test]
fn a_forced_rate_change_reopens_the_preview_at_its_position() {
    let mut r = play_rig(2, Some(on("card", 0)), 48_000 * 10);
    r.act(start(preview(1)));
    r.settle();
    r.run(100);
    let before = r.position();
    let wanted = fp_model::DeviceSettings {
        sample_rate: 44_100,
        ..device_settings(&r)
    };
    r.act(EngineAction::ApplyDevice {
        device: fp_model::OutputDevice {
            backend: "offline".into(),
            device: "card".into(),
        },
        settings: wanted,
        force: true,
    });
    assert_eq!(r.device.config().unwrap().sample_rate, 44_100);
    let reopened = r.position();
    assert!(
        (reopened - before).abs() <= BLOCK_SECS,
        "{before} then {reopened}"
    );
    r.settle();
    r.run(10);
    let position = r.position();
    let last = *r.channel(0).last().unwrap();
    assert!(
        (tagged_secs(1, last, 44_100) - position).abs() <= BLOCK_SECS,
        "heard {last} at {position}: decoded at the new rate"
    );
    assert!(position > reopened);
    assert!(r.ends().is_empty(), "{:?}", r.events);
}

const P: fp_model::PlayerId = fp_model::PlayerId(1);

/// A 4-channel `card`, bit-perfect when `bit_perfect`, with P's Main on
/// its first pair and the library CUE on the second.
fn shared_rig(bit_perfect: bool) -> Play {
    let mut r = play_rig_with(4, Some(on("card", 2)), 48_000 * 10, |c| {
        c.outputs.routes = vec![fp_model::PlayerRoutes {
            player: P,
            main: Some(on("card", 0)),
            cue: None,
        }];
        if bit_perfect {
            c.outputs.bit_perfect = vec![fp_model::OutputDevice {
                backend: "offline".into(),
                device: "card".into(),
            }];
        }
    });
    r.act(EngineAction::AddPlayer { player: P });
    r
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

#[test]
fn a_held_preview_follows_a_rate_change_made_while_it_waits() {
    let mut r = shared_rig(true);
    r.act(EngineAction::StartLibraryCue {
        request: LibraryCueRequest {
            from_secs: 1.0,
            ..preview(1)
        },
        paused: true,
    });
    r.settle();
    r.run(2);
    assert_eq!(r.device.config().unwrap().sample_rate, 48_000);
    r.act(EngineAction::StartCurrent {
        player: P,
        request: fp_model::SourceRequest {
            entry: fp_model::EntryId(2),
            track: TrackId(2),
            path: PathBuf::from("track2"),
            from_secs: 0.0,
            format: pcm16(44_100),
            normalization: Normalization::default(),
        },
    });
    assert_eq!(
        r.device.config().unwrap().sample_rate,
        44_100,
        "the player's track moves the idle bit-perfect bus"
    );
    r.settle();
    r.run(5);
    assert_eq!(r.engine.library_cue_position(), Some((TrackId(1), 1.0)));
    r.act(pause(false));
    r.settle();
    r.run(10);
    let position = r.position();
    let last = r.frames.last().unwrap()[2];
    assert!(
        (tagged_secs(1, last, 44_100) - position).abs() <= BLOCK_SECS,
        "heard {last} at {position}: from where it was held, at the new rate"
    );
    assert!(position > 1.0, "{position}");
}

#[test]
fn the_preview_meter_reads_only_the_preview() {
    let mut r = shared_rig(false);
    r.act(start(preview(1)));
    r.settle();
    r.run(2);
    r.engine.take_library_cue_meter_input();
    r.run(5);
    let preview = r.engine.take_library_cue_meter_input();
    assert!(preview.frames > 0 && preview.peak[0] > 0.0, "{preview:?}");
    assert_eq!(r.engine.take_meter_input(P).frames, 0);
    let again = r.engine.take_library_cue_meter_input();
    assert_eq!(again.frames, 0, "taken, not copied");
}

/// Two seeks before the first tail has ramped out: every tail still ramps
/// down; none is cut (the preview may share a channel with a Main, CA7).
#[test]
fn two_quick_seeks_ramp_every_tail_out() {
    let mut r = play_rig(2, Some(on("card", 0)), 48_000 * 10);
    r.act(start(preview(1)));
    r.settle();
    r.run(5);
    let level = *r.channel(0).last().unwrap();
    r.act(EngineAction::SeekLibraryCue { secs: 3.0 });
    r.settle();
    r.act(EngineAction::SeekLibraryCue { secs: 6.0 });
    let before = r.frames.len();
    r.run(1);
    let left = r.channel(0);
    let mut previous = level;
    for (i, v) in left[before..].iter().enumerate() {
        assert!(
            (v - previous).abs() < level * 0.05,
            "a jump at frame {i} after the seeks: {previous} to {v}"
        );
        previous = *v;
    }
    r.settle();
    r.run(10);
    assert_eq!(r.engine.used_slots(), 1, "every tail is released");
    assert!(r.position() > 6.0);
}
