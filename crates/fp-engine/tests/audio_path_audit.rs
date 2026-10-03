#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Probes for the audio path audit (feedback 2, O26):
//! `docs/technical/audio-path-audit.md`. Each test backs a claim of the
//! audit. A probe that shows a confirmed defect is `#[ignore]`d with the
//! finding's id until its fix task removes the attribute; the others pin
//! down behaviour the audit relies on.
//!
//! Level probes play constant (DC) sources at `LEVEL`: any step between two
//! consecutive output samples is then a step in gain, which is a click on
//! real audio. A 5 ms de-click ramp at 48 kHz moves `LEVEL` by about 0.002
//! per sample; `STEP_LIMIT` allows five times that.

mod support;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use assert_no_alloc::{AllocDisabler, assert_no_alloc};
use fp_backends::{AudioBackend, OfflineBackend, OfflineDevice, Renderer, SampleFormat};
use fp_engine::atomic::AtomicF32;
use fp_engine::engine::{Engine, EngineSettings};
use fp_engine::mixer::{BusCommand, Mixer, MixerConfig, MixerRenderer, SlotStorage};
use fp_engine::resample::StreamResampler;
use fp_engine::source::source_pair;
use fp_engine::worker::{SampleSource, SourceOpener, file_opener};
use fp_model::{
    CartId, CartRequest, Config, EngineAction, EngineEvent, EntryId, PlayerId, PlayerRoutes, Route,
    SourceRequest, TrackId, TransitionPlan,
};

#[cfg(debug_assertions)]
#[global_allocator]
static ALLOCATOR: AllocDisabler = AllocDisabler;

const RATE: f64 = 48_000.0;
const BLOCK: usize = 480;
const P: PlayerId = PlayerId(1);
const LEVEL: f32 = 0.5;
const STEP_LIMIT: f32 = 0.01;

// ---------------------------------------------------------------------------
// Helpers

/// A constant source at `LEVEL`. `fail` ends it with a read error instead of
/// a clean end (a decoder error in the middle of a file).
struct Dc {
    next: u64,
    total: u64,
    fail: bool,
}

impl SampleSource for Dc {
    fn next_block(&mut self, out: &mut Vec<f32>) -> Result<bool, String> {
        if self.next >= self.total {
            return if self.fail {
                Err("read error".to_owned())
            } else {
                Ok(false)
            };
        }
        let end = (self.next + 512).min(self.total);
        for _ in self.next..end {
            out.push(LEVEL);
            out.push(LEVEL);
        }
        self.next = end;
        Ok(true)
    }
}

/// Opens every path as a `Dc` of `frames` frames; paths starting with
/// `fail` end with an error.
fn dc_opener(frames: u64) -> SourceOpener {
    Arc::new(move |path, from_secs, rate| {
        let fail = path
            .file_name()
            .is_some_and(|n| n.to_string_lossy().starts_with("fail"));
        Ok(Box::new(Dc {
            next: (from_secs * f64::from(rate)).round() as u64,
            total: frames,
            fail,
        }) as Box<dyn SampleSource>)
    })
}

/// The largest step between consecutive samples: (index of the later
/// sample, size).
fn max_step(samples: &[f32]) -> (usize, f32) {
    samples
        .windows(2)
        .enumerate()
        .map(|(i, w)| (i + 1, (w[1] - w[0]).abs()))
        .fold((0, 0.0), |a, b| if b.1 > a.1 { b } else { a })
}

fn request(name: &str, entry: u64, from_secs: f64) -> SourceRequest {
    SourceRequest {
        entry: EntryId(entry),
        track: TrackId(entry),
        path: PathBuf::from(name),
        from_secs,
        format: None,
    }
}

fn config() -> Config {
    let mut config = Config::default();
    config.outputs.buffer_frames = BLOCK as u32;
    config.outputs.backend = Some("offline".into());
    // Start only with more buffered than a probe renders, so rendering
    // faster than real time never outruns the worker. The gain smoothing
    // keeps its default (fader moves are probed).
    config.tuning.ready_threshold_ms = 3_000.0;
    config
}

/// One player on an Offline device; `heard` is its left channel.
struct Rig {
    engine: Engine,
    main: OfflineDevice,
    clock: Instant,
    heard: Vec<f32>,
    events: Vec<EngineEvent>,
}

fn rig(opener: SourceOpener) -> Rig {
    let backend = OfflineBackend::new();
    let main = backend.add_device("main", 2);
    let mut config = config();
    config.outputs.routes = vec![PlayerRoutes {
        player: P,
        main: Some(Route {
            backend: "offline".into(),
            device: "main".into(),
            first_channel: 0,
        }),
        cue: None,
    }];
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
    let mut engine = Engine::new(backends, EngineSettings::from_config(&config), opener);
    let clock = Instant::now();
    engine.execute(EngineAction::AddPlayer { player: P }, clock);
    Rig {
        engine,
        main,
        clock,
        heard: Vec::new(),
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

    /// Renders one block without letting the engine see what happened.
    fn render_only(&mut self) {
        let out = self.main.render(BLOCK).unwrap();
        self.heard.extend(out.chunks(2).map(|f| f[0]));
        self.clock += Duration::from_secs_f64(BLOCK as f64 / RATE);
    }

    fn run(&mut self, blocks: usize) {
        for _ in 0..blocks {
            self.render_only();
            let events = self.engine.tick(self.clock);
            self.events.extend(events);
        }
    }

    /// Starts `name` from `from_secs` and plays until it is audible.
    fn play(&mut self, name: &str, from_secs: f64) -> usize {
        self.act(EngineAction::StartCurrent {
            player: P,
            request: request(name, 1, from_secs),
        });
        self.settle();
        self.run(2);
        self.heard.iter().position(|v| *v != 0.0).unwrap()
    }

    /// The largest step after the first audible sample (the start itself
    /// is probed separately).
    fn step_after(&self, start: usize) -> (usize, f32) {
        let (i, s) = max_step(&self.heard[start..]);
        (start + i, s)
    }
}

// ---------------------------------------------------------------------------
// §3.2 Gain ramps: paths that ramp (pass)

#[test]
fn a_stop_ramps_down_without_a_step() {
    let mut r = rig(dc_opener(480_000));
    let start = r.play("dc", 0.0);
    r.run(3);
    r.act(EngineAction::StopNow { player: P });
    r.run(5);
    assert_eq!(*r.heard.last().unwrap(), 0.0, "stopped");
    let (at, step) = r.step_after(start);
    assert!(step < STEP_LIMIT, "step {step} at {at}");
}

#[test]
fn pause_and_resume_ramp_without_a_step() {
    let mut r = rig(dc_opener(480_000));
    let start = r.play("dc", 0.0);
    r.run(3);
    r.act(EngineAction::Pause { player: P });
    r.run(5);
    assert_eq!(*r.heard.last().unwrap(), 0.0, "paused");
    r.act(EngineAction::Resume { player: P });
    r.run(5);
    assert_eq!(*r.heard.last().unwrap(), LEVEL, "playing again");
    let (at, step) = r.step_after(start);
    assert!(step < STEP_LIMIT, "step {step} at {at}");
}

#[test]
fn a_seek_ramps_the_old_source_out_and_the_new_one_in() {
    let mut r = rig(dc_opener(480_000));
    let start = r.play("dc", 0.0);
    r.run(3);
    r.act(EngineAction::Seek {
        player: P,
        secs: 3.0,
    });
    r.run(2);
    r.settle();
    r.run(5);
    assert_eq!(
        *r.heard.last().unwrap(),
        LEVEL,
        "playing at the new position"
    );
    let (at, step) = r.step_after(start);
    assert!(step < STEP_LIMIT, "step {step} at {at}");
}

#[test]
fn a_fader_move_is_smoothed() {
    let mut r = rig(dc_opener(480_000));
    let start = r.play("dc", 0.0);
    r.act(EngineAction::SetVolume {
        player: P,
        volume: 0.0,
    });
    r.run(5);
    r.act(EngineAction::SetVolume {
        player: P,
        volume: 1.0,
    });
    r.run(5);
    let (at, step) = r.step_after(start);
    assert!(step < STEP_LIMIT, "step {step} at {at}");
}

#[test]
fn a_fade_stop_and_a_planned_stop_ramp_down() {
    let mut r = rig(dc_opener(480_000));
    let start = r.play("dc", 0.0);
    r.act(EngineAction::FadeOutAndStop {
        player: P,
        fade_ms: 300,
    });
    r.run(40);
    let (at, step) = r.step_after(start);
    assert!(step < STEP_LIMIT, "fade stop: step {step} at {at}");

    let mut r = rig(dc_opener(480_000));
    let start = r.play("dc", 0.0);
    r.act(EngineAction::Schedule {
        player: P,
        plan: Some(TransitionPlan::StopAt { at_secs: 0.5 }),
    });
    r.run(80);
    assert_eq!(*r.heard.last().unwrap(), 0.0, "stopped at the cue-out");
    let (at, step) = r.step_after(start);
    assert!(step < STEP_LIMIT, "planned stop: step {step} at {at}");
}

// ---------------------------------------------------------------------------
// §3.2 Gain ramps: paths that step (confirmed defects)

// A1: fails until the fix task. A start inside the audio (a cue-in or a
// position past the first frame) begins at full level, with no ramp.
#[test]
fn a1_a_start_inside_the_file_ramps_in() {
    let mut r = rig(dc_opener(480_000));
    let start = r.play("dc", 1.0);
    r.run(2);
    // The device was silent before its first block.
    let heard: Vec<f32> = std::iter::once(0.0)
        .chain(r.heard.iter().copied())
        .collect();
    let (at, step) = max_step(&heard);
    assert!(step < STEP_LIMIT, "step {step} at {at} (start at {start})");
}

// A1: the new source of a hard transition (and of a segue or crossfade: they
// start through the same path) that starts at a cue-in past the first frame
// ramps in; the old one ramps out, so the sum never steps.
#[test]
fn a1_the_next_source_at_a_cue_in_ramps_in() {
    let mut r = rig(dc_opener(480_000));
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request("a", 1, 0.0),
    });
    r.act(EngineAction::Preload {
        player: P,
        request: Some(request("b", 2, 2.0)),
    });
    r.settle();
    r.run(2);
    let start = r.heard.iter().position(|v| *v != 0.0).unwrap();
    r.act(EngineAction::Schedule {
        player: P,
        plan: Some(TransitionPlan::StartNextAt {
            at_secs: 0.5,
            fade_current_until_secs: None,
        }),
    });
    r.run(60);
    assert!(
        r.events.contains(&EngineEvent::TransitionStarted {
            player: P,
            entry: EntryId(2)
        }),
        "the next started"
    );
    let (at, step) = r.step_after(start);
    assert!(step < STEP_LIMIT, "step {step} at {at}");
}

// A1: a CUE that starts inside the file ramps in.
#[test]
fn a1_a_cue_inside_the_file_ramps_in() {
    let backend = OfflineBackend::new();
    let main = backend.add_device("main", 2);
    let cue = backend.add_device("cue", 2);
    let mut config = config();
    let route = |device: &str| Route {
        backend: "offline".into(),
        device: device.into(),
        first_channel: 0,
    };
    config.outputs.routes = vec![PlayerRoutes {
        player: P,
        main: Some(route("main")),
        cue: Some(route("cue")),
    }];
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
    let mut engine = Engine::new(
        backends,
        EngineSettings::from_config(&config),
        dc_opener(480_000),
    );
    let mut clock = Instant::now();
    engine.execute(EngineAction::AddPlayer { player: P }, clock);
    engine.execute(
        EngineAction::StartCue {
            player: P,
            request: request("cue", 3, 1.0),
        },
        clock,
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    while engine.unsettled_sources() > 0 && Instant::now() < deadline {
        engine.tick(clock);
        std::thread::sleep(Duration::from_millis(1));
    }
    let _ = &main;
    let mut heard = vec![0.0];
    for _ in 0..3 {
        heard.extend(cue.render(BLOCK).unwrap().chunks(2).map(|f| f[0]));
        clock += Duration::from_secs_f64(BLOCK as f64 / RATE);
        engine.tick(clock);
    }
    assert!(heard.iter().any(|v| *v != 0.0), "the cue was heard");
    let (at, step) = max_step(&heard);
    assert!(step < STEP_LIMIT, "step {step} at {at}");
}

// A1: a cart that starts at a cue-in past the first frame ramps in; one
// that starts at the first frame stays hard.
#[test]
fn a1_a_cart_inside_the_file_ramps_in_and_one_at_the_start_does_not() {
    for (from_secs, ramps) in [(0.5, true), (0.0, false)] {
        let backend = OfflineBackend::new();
        let card = backend.add_device("card", 2);
        let mut config = config();
        config.outputs.cartwall.main = Some(Route {
            backend: "offline".into(),
            device: "card".into(),
            first_channel: 0,
        });
        let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
        let mut engine = Engine::new(
            backends,
            EngineSettings::from_config(&config),
            dc_opener(480_000),
        );
        let mut clock = Instant::now();
        engine.execute(
            EngineAction::StartCart(CartRequest {
                cart: CartId(1),
                track: TrackId(1),
                path: PathBuf::from("cart"),
                from_secs,
                until_secs: 5.0,
                looped: false,
                format: None,
            }),
            clock,
        );
        let deadline = Instant::now() + Duration::from_secs(5);
        while engine.unsettled_sources() > 0 && Instant::now() < deadline {
            engine.tick(clock);
            std::thread::sleep(Duration::from_millis(1));
        }
        let mut heard = vec![0.0];
        for _ in 0..3 {
            heard.extend(card.render(BLOCK).unwrap().chunks(2).map(|f| f[0]));
            clock += Duration::from_secs_f64(BLOCK as f64 / RATE);
            engine.tick(clock);
        }
        let (at, step) = max_step(&heard);
        assert_eq!(
            step < STEP_LIMIT,
            ramps,
            "from {from_secs}: step {step} at {at}"
        );
    }
}

// A2: a hard transition at the very end of the
// current file (a gapless join) fades the current out for `declick_ms`
// before the next starts at full level: a dip to silence, then a step.
#[test]
fn a2_a_gapless_join_at_the_end_of_the_file_keeps_the_level() {
    let mut r = rig(dc_opener(48_000)); // every file is 1 s long
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request("a", 1, 0.0),
    });
    r.act(EngineAction::Preload {
        player: P,
        request: Some(request("b", 2, 0.0)),
    });
    r.settle();
    r.run(2);
    let start = r.heard.iter().position(|v| *v != 0.0).unwrap();
    r.act(EngineAction::Schedule {
        player: P,
        plan: Some(TransitionPlan::StartNextAt {
            at_secs: 1.0,
            fade_current_until_secs: None,
        }),
    });
    r.run(110);
    assert!(
        r.events.contains(&EngineEvent::TransitionStarted {
            player: P,
            entry: EntryId(2)
        }),
        "the next started"
    );
    let join = &r.heard[start..start + 48_000 + 4_800];
    let lowest = join.iter().copied().fold(f32::MAX, f32::min);
    let (at, step) = max_step(join);
    assert!(
        lowest > LEVEL * 0.99 && step < STEP_LIMIT,
        "lowest {lowest}, step {step} at {} (join at {})",
        start + at,
        start + 48_000
    );
}

// A2 (guard): a hard transition inside the file still fades the current out
// for `declick_ms` before it is cut, so the cut itself has no step.
#[test]
fn a2_a_hard_cut_inside_the_file_still_ramps_the_outgoing_source_out() {
    let mut r = rig(dc_opener(96_000)); // every file is 2 s long
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request("a", 1, 0.0),
    });
    r.act(EngineAction::Preload {
        player: P,
        request: Some(request("b", 2, 0.0)),
    });
    r.settle();
    r.run(2);
    let start = r.heard.iter().position(|v| *v != 0.0).unwrap();
    r.act(EngineAction::Schedule {
        player: P,
        plan: Some(TransitionPlan::StartNextAt {
            at_secs: 1.0,
            fade_current_until_secs: None,
        }),
    });
    r.run(110);
    assert!(
        r.events.contains(&EngineEvent::TransitionStarted {
            player: P,
            entry: EntryId(2)
        }),
        "the next started"
    );
    let cut = start + 48_000;
    let before = &r.heard[cut - 480..cut];
    let lowest = before.iter().copied().fold(f32::MAX, f32::min);
    assert!(
        lowest < LEVEL * 0.05,
        "the outgoing source ramps to silence before the cut (lowest {lowest})"
    );
}

// A2 (stop arm): a planned stop at the file's own end keeps the level to the
// last frame too; a stop inside the file still ramps (the test above).
#[test]
fn a2_a_planned_stop_at_the_end_of_the_file_keeps_the_level_to_the_end() {
    let mut r = rig(dc_opener(48_000)); // 1 s long
    let start = r.play("dc", 0.0);
    r.act(EngineAction::Schedule {
        player: P,
        plan: Some(TransitionPlan::StopAt { at_secs: 1.0 }),
    });
    r.run(110);
    let end = start + 48_000;
    let tail = &r.heard[end - 480..end];
    let lowest = tail.iter().copied().fold(f32::MAX, f32::min);
    assert!(lowest > LEVEL * 0.99, "lowest {lowest} before the end");
}

// A3: fails until the fix task. A cart's cue-out is cut by the worker
// (`LoadOptions::until_secs`), and the mixer ends the drained source at
// full level.
#[test]
fn a3_a_cart_ends_at_its_cue_out_without_a_step() {
    let backend = OfflineBackend::new();
    let card = backend.add_device("card", 2);
    let mut config = config();
    config.outputs.cartwall.main = Some(Route {
        backend: "offline".into(),
        device: "card".into(),
        first_channel: 0,
    });
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
    let mut engine = Engine::new(
        backends,
        EngineSettings::from_config(&config),
        dc_opener(96_000),
    );
    let mut clock = Instant::now();
    engine.execute(
        EngineAction::StartCart(CartRequest {
            cart: CartId(1),
            track: TrackId(1),
            path: PathBuf::from("cart"),
            from_secs: 0.0,
            until_secs: 0.5,
            looped: false,
            format: None,
        }),
        clock,
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    while engine.unsettled_sources() > 0 && Instant::now() < deadline {
        engine.tick(clock);
        std::thread::sleep(Duration::from_millis(1));
    }
    let mut heard = Vec::new();
    let mut events = Vec::new();
    for _ in 0..80 {
        heard.extend(card.render(BLOCK).unwrap().chunks(2).map(|f| f[0]));
        clock += Duration::from_secs_f64(BLOCK as f64 / RATE);
        events.extend(engine.tick(clock));
    }
    assert!(events.contains(&EngineEvent::CartEnded { cart: CartId(1) }));
    let start = heard.iter().position(|v| *v != 0.0).unwrap();
    let (at, step) = max_step(&heard[start..]);
    assert!(step < STEP_LIMIT, "step {step} at {}", start + at);
}

// A4: fails until the fix task. A decoder error in the middle of a file:
// the buffered audio plays out (as spec §4.5 asks) and the source is
// reported failed once, but its end is cut at full level.
#[test]
fn a4_a_source_that_fails_mid_file_ends_without_a_step() {
    let mut r = rig(dc_opener(48_000));
    let start = r.play("fail", 0.0);
    r.run(120);
    let failures = r
        .events
        .iter()
        .filter(|e| matches!(e, EngineEvent::SourceFailed { .. }))
        .count();
    assert_eq!(failures, 1, "reported once, after the buffer drained");
    let (at, step) = r.step_after(start);
    assert!(step < STEP_LIMIT, "step {step} at {at}");
}

// A5: fails until the fix task. A source released while it may still be
// audible is detached at once: a stop while the pause ramp runs.
#[test]
fn a5_a_stop_during_the_pause_ramp_does_not_step() {
    let mut r = rig(dc_opener(480_000));
    let start = r.play("dc", 0.0);
    r.run(3);
    r.act(EngineAction::Pause { player: P });
    r.act(EngineAction::StopNow { player: P });
    r.run(5);
    let (at, step) = r.step_after(start);
    assert!(step < STEP_LIMIT, "step {step} at {at}");
}

// A5: fails until the fix task. The same with a start the mixer already
// executed but whose `Started` event the engine has not polled yet.
#[test]
fn a5_a_stop_right_after_the_start_does_not_step() {
    let mut r = rig(dc_opener(480_000));
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request("dc", 1, 0.0),
    });
    r.settle(); // the start is sent, its frame not rendered yet
    r.render_only();
    r.render_only();
    let start = r.heard.iter().position(|v| *v != 0.0).unwrap();
    r.act(EngineAction::StopNow { player: P });
    r.run(5);
    let (at, step) = r.step_after(start);
    assert!(step < STEP_LIMIT, "step {step} at {at}");
}

// ---------------------------------------------------------------------------
// §1.1 / rule 9: untrusted samples

fn float_wav(dir: &Path, name: &str, samples: &[f32]) -> PathBuf {
    let path = dir.join(name);
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 48_000,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut w = hound::WavWriter::create(&path, spec).unwrap();
    for s in samples {
        w.write_sample(*s).unwrap();
    }
    w.finalize().unwrap();
    path
}

/// Plays `path` through the real decoder and returns the left channel.
fn play_file(path: &Path, blocks: usize) -> (Vec<f32>, Engine) {
    let mut r = rig(file_opener());
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(path.to_str().unwrap(), 1, 0.0),
    });
    r.settle();
    r.run(blocks);
    (r.heard, r.engine)
}

// A6: fails until the fix task. NaN and infinite samples in a float file
// pass the decoder, the mixer and the backend conversion unchanged: a
// float device (the format cpal prefers) receives them.
#[test]
fn a6_non_finite_samples_never_reach_the_device() {
    let dir = tempfile::tempdir().unwrap();
    let mut samples = vec![0.25f32; 9_600];
    samples[4_000] = f32::NAN;
    samples[4_001] = f32::INFINITY;
    samples[4_002] = f32::NEG_INFINITY;
    let path = float_wav(dir.path(), "nan.wav", &samples);
    let (heard, _) = play_file(&path, 30);
    let bad = heard.iter().filter(|v| !v.is_finite()).count();
    assert_eq!(bad, 0, "{bad} non-finite samples reached the device");
}

#[test]
fn a_truncated_wav_ends_as_a_normal_end_not_a_failure() {
    // An interrupted download plays what it has and reports the end, not
    // a failure (`symph.rs` maps UnexpectedEof to the end of the stream).
    let dir = tempfile::tempdir().unwrap();
    let path = support::indexed_wav(dir.path(), "cut.wav", 48_000, 2, 48_000);
    let bytes = std::fs::read(&path).unwrap();
    std::fs::write(&path, &bytes[..bytes.len() / 2 + 3]).unwrap();
    let mut r = rig(file_opener());
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(path.to_str().unwrap(), 1, 0.0),
    });
    r.settle();
    r.run(80);
    assert!(r.events.contains(&EngineEvent::ReachedEnd {
        player: P,
        entry: EntryId(1)
    }));
    assert!(
        !r.events
            .iter()
            .any(|e| matches!(e, EngineEvent::SourceFailed { .. }))
    );
}

// ---------------------------------------------------------------------------
// §2.1 Resampling quality

fn sine(rate: f64, freq: f64, amp: f64, phase: f64, secs: f64) -> Vec<f32> {
    let n = (rate * secs) as usize;
    (0..n)
        .flat_map(|i| {
            let v = (amp * (std::f64::consts::TAU * freq * i as f64 / rate + phase).sin()) as f32;
            [v, v]
        })
        .collect()
}

fn resample(input: &[f32], from: u32, to: u32) -> Vec<f32> {
    let mut r = StreamResampler::new(from, to).unwrap();
    let mut out = Vec::new();
    r.push(input, &mut out).unwrap();
    r.finish(&mut out).unwrap();
    out
}

/// Amplitude of `freq` in the left channel over exactly one second
/// starting at `from` (an integer frequency sits on a bin: no scalloping),
/// Hann-windowed so the other tones do not leak in.
fn amplitude(stereo: &[f32], rate: usize, from: usize, freq: f64) -> f64 {
    let n = rate;
    let w = |i: usize| 0.5 - 0.5 * (std::f64::consts::TAU * i as f64 / n as f64).cos();
    let (mut re, mut im, mut sum) = (0.0f64, 0.0f64, 0.0f64);
    for i in 0..n {
        let x = f64::from(stereo[2 * (from + i)]) * w(i);
        let a = std::f64::consts::TAU * freq * i as f64 / rate as f64;
        re += x * a.cos();
        im -= x * a.sin();
        sum += w(i);
    }
    2.0 * (re * re + im * im).sqrt() / sum
}

fn db(x: f64) -> f64 {
    20.0 * x.max(1e-12).log10()
}

#[test]
fn resampling_44_1_to_48_khz_is_flat_to_20_khz_and_rejects_images() {
    // Run with `--nocapture` to read the figures the audit quotes.
    let mut gains = Vec::new();
    for f in [
        20.0, 1_000.0, 10_000.0, 16_000.0, 18_000.0, 19_000.0, 20_000.0, 20_500.0, 21_000.0,
    ] {
        let out = resample(&sine(44_100.0, f, 0.5, 0.0, 2.0), 44_100, 48_000);
        let g = db(amplitude(&out, 48_000, 24_000, f) / 0.5);
        println!("44.1→48 kHz passband: {f:>7.0} Hz {g:+.4} dB");
        gains.push((f, g));
    }
    // The image of 21 kHz (44.1 − 21 = 23.1 kHz) lands inside the 48 kHz band.
    let out = resample(&sine(44_100.0, 21_000.0, 0.5, 0.0, 2.0), 44_100, 48_000);
    let image = db(amplitude(&out, 48_000, 24_000, 23_100.0) / 0.5);
    println!("44.1→48 kHz image of 21 kHz at 23.1 kHz: {image:.1} dB");
    for (f, g) in &gains {
        if *f <= 19_000.0 {
            assert!(g.abs() < 0.05, "{f} Hz: {g} dB");
        }
    }
    assert!(image < -60.0, "image {image} dB");
}

#[test]
fn resampling_48_to_44_1_khz_rejects_aliases() {
    let mut worst = f64::MIN;
    for f in [22_500.0, 23_000.0, 23_500.0] {
        let out = resample(&sine(48_000.0, f, 0.5, 0.0, 2.0), 48_000, 44_100);
        let alias = 44_100.0 - f;
        let a = db(amplitude(&out, 44_100, 22_050, alias) / 0.5);
        println!("48→44.1 kHz: {f:.0} Hz aliases to {alias:.0} Hz at {a:.1} dB");
        worst = worst.max(a);
    }
    let out = resample(&sine(48_000.0, 1_000.0, 0.5, 0.0, 2.0), 48_000, 44_100);
    let g = db(amplitude(&out, 44_100, 22_050, 1_000.0) / 0.5);
    println!("48→44.1 kHz passband: 1000 Hz {g:+.4} dB");
    assert!(g.abs() < 0.05);
    assert!(worst < -60.0, "alias {worst} dB");
}

// ---------------------------------------------------------------------------
// §7 Readings above 0 dBFS (O34)

fn meter_mixer(slots: usize) -> (Mixer, fp_engine::mixer::MixerHandle) {
    let (m, h) = Mixer::new(
        slots,
        MixerConfig {
            volume_smoothing_frames: 1,
            declick_frames: 0,
            max_commands_per_block: 64,
        },
    );
    h.shared.sample_rate.store(48_000, Ordering::Release);
    (m, h)
}

fn dc_source(level: f32, frames: usize) -> fp_engine::source::SourceConsumer {
    let (mut p, c) = source_pair(frames);
    p.push(&vec![level; frames * 2]);
    c
}

#[test]
fn a_float_file_above_full_scale_reaches_the_mix_and_the_meter_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    let path = float_wav(dir.path(), "over.wav", &vec![1.5f32; 48_000]);
    let (heard, engine) = play_file(&path, 20);
    let peak = heard.iter().copied().fold(0.0f32, f32::max);
    assert_eq!(peak, 1.5, "no clamp before the device");
    let input = engine.take_meter_input(P);
    assert_eq!(input.peak[0], 1.5, "the meter reads +3.5 dBFS");
}

#[test]
fn resampling_creates_inter_sample_overs() {
    // fs/4 at 45°: every sample is ±1.0 (0 dBFS) though the wave peaks at
    // +3 dB between them; at 48 kHz the samples land near those peaks.
    let input = sine(
        44_100.0,
        11_025.0,
        std::f64::consts::SQRT_2,
        0.25 * std::f64::consts::PI,
        1.0,
    );
    let in_peak = input.iter().fold(0.0f32, |m, v| m.max(v.abs()));
    let out = resample(&input, 44_100, 48_000);
    let out_peak = out[2_000..out.len() - 2_000]
        .iter()
        .fold(0.0f32, |m, v| m.max(v.abs()));
    println!(
        "inter-sample over: input sample peak {:+.2} dBFS, resampled {:+.2} dBFS",
        db(f64::from(in_peak)),
        db(f64::from(out_peak))
    );
    assert!(in_peak <= 1.0 + 1e-6);
    assert!(out_peak > 1.3, "{out_peak}");
}

#[test]
fn the_meter_is_per_player_and_never_sees_the_device_sum() {
    let (mut m, mut h) = meter_mixer(2);
    let a = dc_source(0.7, 4_096);
    let b = dc_source(0.7, 4_096);
    let (sa, sb) = (a.shared.clone(), b.shared.clone());
    for (slot, source) in [(0, a), (1, b)] {
        assert!(
            h.commands
                .push(BusCommand::Attach {
                    slot,
                    source,
                    volume: Arc::new(AtomicF32::new(1.0)),
                    first_channel: 0,
                })
                .is_ok()
        );
        assert!(
            h.commands
                .push(BusCommand::Start { slot, at_frame: 0 })
                .is_ok()
        );
    }
    let mut out = vec![0.0f32; 512 * 2];
    m.render(&mut out, 2);
    assert!((out[100] - 1.4).abs() < 1e-6, "the device gets +2.9 dBFS");
    assert_eq!(sa.peak_l.take(), 0.7, "each source reads −3.1 dBFS");
    assert_eq!(sb.peak_l.take(), 0.7);
}

#[test]
fn no_gain_in_the_mixer_exceeds_unity() {
    let (mut m, mut h) = meter_mixer(1);
    assert!(
        h.commands
            .push(BusCommand::Attach {
                slot: 0,
                source: dc_source(0.5, 4_096),
                volume: Arc::new(AtomicF32::new(1.0)),
                first_channel: 0,
            })
            .is_ok()
    );
    assert!(
        h.commands
            .push(BusCommand::Start {
                slot: 0,
                at_frame: 0
            })
            .is_ok()
    );
    let mut out = vec![0.0f32; 512 * 2];
    m.render(&mut out, 2);
    // A volume above 1 (the model never sends one) is clamped per block.
    // (The Attach copies the volume unclamped as the smoothing start, so
    // it is set only after the first block.)
    let volume = Arc::new(AtomicF32::new(1.0));
    let (mut m, mut h) = meter_mixer(1);
    assert!(
        h.commands
            .push(BusCommand::Attach {
                slot: 0,
                source: dc_source(0.5, 4_096),
                volume: volume.clone(),
                first_channel: 0,
            })
            .is_ok()
    );
    assert!(
        h.commands
            .push(BusCommand::Start {
                slot: 0,
                at_frame: 0
            })
            .is_ok()
    );
    m.render(&mut out, 2);
    volume.store(4.0);
    m.render(&mut out, 2);
    assert!(out.iter().all(|v| *v <= 0.5), "unity at most");
}

#[test]
fn a_three_channel_file_downmixes_above_full_scale() {
    // L R C at −1.9 dBFS each: BS.775 folds the centre in at −3 dB with no
    // normalisation for 3.0 and 3.1 (5.x is normalised), so L' = 0.8 + 0.57.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("lrc.wav");
    let spec = hound::WavSpec {
        channels: 3,
        sample_rate: 48_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(&path, spec).unwrap();
    for _ in 0..4_800 {
        for _ in 0..3 {
            w.write_sample(26_214i16).unwrap(); // 0.8
        }
    }
    w.finalize().unwrap();
    let mut d = fp_engine::decode::FileDecoder::open(&path).unwrap();
    let mut out = Vec::new();
    d.next_block(&mut out).unwrap();
    assert!(out[0] > 1.3, "{}", out[0]);
}

#[test]
fn what_an_exclusive_device_receives_above_full_scale_and_for_nan() {
    let samples = [1.5f32, -1.5, f32::NAN, 1.0];
    let mut bytes = [0u8; 16];
    fp_backends::exclusive::write_samples(SampleFormat::I24, &samples, &mut bytes);
    let words: Vec<i32> = bytes
        .chunks(4)
        .map(|b| i32::from_le_bytes(b.try_into().unwrap()))
        .collect();
    assert_eq!(
        words,
        vec![0x7fff_ff00, i32::MIN, 0, 0x7fff_ff00],
        "I24 clips (saturating), NaN is silence"
    );
    let mut bytes = [0u8; 8];
    fp_backends::exclusive::write_samples(SampleFormat::I16, &samples, &mut bytes);
    let words: Vec<i16> = bytes
        .chunks(2)
        .map(|b| i16::from_le_bytes(b.try_into().unwrap()))
        .collect();
    assert_eq!(words, vec![i16::MAX, i16::MIN, 0, i16::MAX]);
    let mut bytes = [0u8; 16];
    fp_backends::exclusive::write_samples(SampleFormat::F32, &samples, &mut bytes);
    let floats: Vec<f32> = bytes
        .chunks(4)
        .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
        .collect();
    assert_eq!(floats[0], 1.5, "float goes out unclipped");
    assert!(floats[2].is_nan(), "and NaN too");
}

// ---------------------------------------------------------------------------
// §6 Real-time safety: paths `tests/mixer.rs` does not wrap

#[test]
fn the_renderer_the_device_calls_and_the_sample_writer_never_allocate() {
    let (m, mut h) = meter_mixer(2);
    let shared = h.shared.clone();
    shared.true_peak.store(true, Ordering::Release);
    shared.ppm_tau1_ms.store(5.0);
    shared.fall_db_per_sec.store(20.0);
    let mut renderer = MixerRenderer {
        mixer: Arc::new(std::sync::Mutex::new(m)),
        shared,
        dop: None,
        native: false,
    };
    // Built off the real-time thread, applied on it.
    let grow = BusCommand::Grow(SlotStorage::with_capacity(4));
    assert!(h.commands.push(grow).is_ok());
    assert!(
        h.commands
            .push(BusCommand::Attach {
                slot: 0,
                source: dc_source(0.5, 4_096),
                volume: Arc::new(AtomicF32::new(1.0)),
                first_channel: 0,
            })
            .is_ok()
    );
    assert!(
        h.commands
            .push(BusCommand::Start {
                slot: 0,
                at_frame: 0
            })
            .is_ok()
    );
    let mut out = vec![0.0f32; 512 * 2];
    let mut bytes = vec![0u8; 512 * 2 * 4];
    assert_no_alloc(|| {
        renderer.render(&mut out, 2);
        for format in [
            SampleFormat::I16,
            SampleFormat::I24,
            SampleFormat::I32,
            SampleFormat::F32,
        ] {
            fp_backends::exclusive::write_samples(format, &out, &mut bytes);
        }
    });
    assert!(h.commands.push(BusCommand::Detach { slot: 0 }).is_ok());
    assert_no_alloc(|| renderer.render(&mut out, 2));
    assert_eq!(h.retired.slots(), 2, "storage and source handed back");
}
