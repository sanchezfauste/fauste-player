#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Hardware check of the bit-perfect rate cut (Phase 4 spec BR6, BR8) and
//! of the native DSD stop: when the engine closes a device stream to reopen
//! it at another rate, how much audio is still queued in the driver and
//! the device, and does any of it carry sound?
//!
//! The engine runs on a real ALSA `hw:` device through the project's
//! backend, ticking like the conductor. A tap around the backend counts
//! the frames the mixer rendered after the last sounding one (the silence
//! at the end of the queue); when a stream is dropped, it first reads the
//! kernel's view of the substream (`/proc/asound/<card>/pcm0p/sub0`):
//! `delay` is every frame written but not yet played, including those in
//! flight to a USB device. If `delay` is larger than the silence at the
//! end, the difference is sound that the close discarded. It also prints
//! what the stream reported holding (`OutputStream::latency_frames`), which
//! the engine waits for before a close (spec BR16), and the bus's xruns.
//!
//! The wall-clock figures only mean something on bare metal: with the
//! converter passed through to a virtual machine (QEMU's USB host), the
//! guest's stream consumes many times faster than real time, so silence
//! piles up after the last sound far sooner than a real device plays it.
//!
//! Only very quiet tones are played (−36 dBFS PCM, −40 dB DSD), a second
//! or two at a time. Linux only; never run in CI.
//!
//! `cargo run -p fp-engine --example rate_cut_tail -- [card] [buffer frames]`
//! (`card` is the ALSA card id, `AUDIO` by default; `buffer` 512).

use std::f64::consts::PI;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use fp_backends::{
    AudioBackend, Availability, BackendError, BackendId, DeviceId, DeviceInfo, OutputStream,
    Renderer, SampleFormat, StreamConfig, StreamErrorSink,
};
use fp_engine::engine::{Engine, EngineSettings};
use fp_engine::worker::file_opener;
use fp_model::{
    AudioFormat, Config, DsdDevice, DsdOutput, EngineAction, EngineEvent, EntryId, OutputDevice,
    PlayerId, PlayerRoutes, Route, SourceRequest, TrackId, TransitionPlan,
};

const P: PlayerId = PlayerId(1);
/// −36 dBFS.
const PCM_AMPLITUDE: f64 = 0.015_85;
/// −40 dB of the DSD modulation range.
const DSD_AMPLITUDE: f64 = 0.01;
const DSD64: u32 = 2_822_400;

/// What a dropped stream left behind.
#[derive(Debug, Clone)]
struct CloseReport {
    label: String,
    config: StreamConfig,
    /// Frames written but not played when the drop began (kernel view).
    delay: Option<i64>,
    /// Frames the device consumed, and frames written, since the stream started.
    hw_ptr: Option<i64>,
    /// What the stream reported holding at most (`latency_frames`).
    reported: Option<u32>,
    appl_ptr: Option<i64>,
    hw_params: String,
    /// Frames the mixer rendered after the last sounding one.
    silence_after_sound: u64,
    /// Frames rendered on this stream in all.
    rendered: u64,
    /// Render calls on this stream.
    calls: u64,
    /// From opening the stream to its drop.
    open_for: Duration,
    /// From the render of the last sounding frame to the drop.
    since_sound: Duration,
}

impl CloseReport {
    fn cut_frames(&self) -> Option<i64> {
        let silence = i64::try_from(self.silence_after_sound).ok()?;
        self.delay.map(|d| (d - silence).max(0))
    }
}

#[derive(Default)]
struct Probe {
    label: Mutex<String>,
    reports: Mutex<Vec<CloseReport>>,
}

/// Per stream: frames rendered, and the frame count at the end of the last
/// sounding frame.
#[derive(Default)]
struct Counters {
    rendered: AtomicU64,
    sound_end: AtomicU64,
    /// Nanoseconds from `opened` to the callback that rendered the last
    /// sounding frame.
    sound_at_ns: AtomicU64,
    calls: AtomicU64,
    opened: OnceInstant,
}

/// The instant a stream was opened.
struct OnceInstant(Instant);

impl Default for OnceInstant {
    fn default() -> Self {
        Self(Instant::now())
    }
}

struct TapRenderer {
    inner: Box<dyn Renderer>,
    counters: Arc<Counters>,
    dsd: bool,
}

impl Renderer for TapRenderer {
    fn render(&mut self, out: &mut [f32], channels: usize) {
        self.inner.render(out, channels);
        let channels = channels.max(1);
        let base = self.counters.rendered.load(Ordering::Relaxed);
        let silence = fp_backends::dsd::silence_sample();
        let quiet = |s: f32| s == 0.0 || (self.dsd && s == silence);
        let mut last = None;
        for (i, frame) in out.chunks(channels).enumerate() {
            if frame.iter().any(|s| !quiet(*s)) {
                last = Some(i as u64 + 1);
            }
        }
        let frames = (out.len() / channels) as u64;
        self.counters.calls.fetch_add(1, Ordering::Relaxed);
        if let Some(end) = last {
            self.counters.sound_end.store(base + end, Ordering::Relaxed);
            let ns = self.counters.opened.0.elapsed().as_nanos() as u64;
            self.counters.sound_at_ns.store(ns, Ordering::Relaxed);
        }
        self.counters
            .rendered
            .store(base + frames, Ordering::Relaxed);
    }
}

struct TapStream {
    inner: Option<Box<dyn OutputStream>>,
    counters: Arc<Counters>,
    probe: Arc<Probe>,
    proc_dir: PathBuf,
}

impl OutputStream for TapStream {
    fn config(&self) -> StreamConfig {
        self.inner.as_ref().unwrap().config()
    }
    fn sample_format(&self) -> SampleFormat {
        self.inner.as_ref().unwrap().sample_format()
    }
    fn dsd(&self) -> Option<fp_backends::dsd::DsdStream> {
        self.inner.as_ref().unwrap().dsd()
    }
}

fn status_field(status: &str, name: &str) -> Option<i64> {
    status.lines().find_map(|l| {
        let (k, v) = l.split_once(':')?;
        (k.trim() == name).then(|| v.trim().parse().ok())?
    })
}

impl Drop for TapStream {
    fn drop(&mut self) {
        // The kernel's view first, before anything else can move.
        let status = std::fs::read_to_string(self.proc_dir.join("status")).unwrap_or_default();
        let rendered = self.counters.rendered.load(Ordering::Relaxed);
        let sound_end = self.counters.sound_end.load(Ordering::Relaxed);
        let open_for = self.counters.opened.0.elapsed();
        let since_sound = open_for.saturating_sub(Duration::from_nanos(
            self.counters.sound_at_ns.load(Ordering::Relaxed),
        ));
        let calls = self.counters.calls.load(Ordering::Relaxed);
        let hw_params = std::fs::read_to_string(self.proc_dir.join("hw_params"))
            .unwrap_or_default()
            .lines()
            .filter(|l| {
                l.starts_with("rate")
                    || l.starts_with("period_size")
                    || l.starts_with("buffer_size")
            })
            .map(str::trim)
            .collect::<Vec<_>>()
            .join(", ");
        let config = self.config();
        let reported = self.inner.as_ref().and_then(|s| s.latency_frames());
        self.inner = None;
        let report = CloseReport {
            label: self.probe.label.lock().unwrap().clone(),
            config,
            delay: status_field(&status, "delay"),
            hw_ptr: status_field(&status, "hw_ptr"),
            reported,
            appl_ptr: status_field(&status, "appl_ptr"),
            hw_params,
            silence_after_sound: rendered.saturating_sub(sound_end),
            rendered,
            calls,
            open_for,
            since_sound,
        };
        self.probe.reports.lock().unwrap().push(report);
    }
}

struct TapBackend {
    inner: Arc<dyn AudioBackend>,
    probe: Arc<Probe>,
    proc_dir: PathBuf,
}

impl AudioBackend for TapBackend {
    fn id(&self) -> BackendId {
        self.inner.id()
    }
    fn availability(&self) -> Availability {
        self.inner.availability()
    }
    fn enumerate_devices(&self) -> Result<Vec<DeviceInfo>, BackendError> {
        self.inner.enumerate_devices()
    }
    fn default_device(&self) -> Option<DeviceId> {
        self.inner.default_device()
    }
    fn open_output(
        &self,
        device: &DeviceId,
        config: StreamConfig,
        renderer: Box<dyn Renderer>,
        errors: Arc<dyn StreamErrorSink>,
    ) -> Result<Box<dyn OutputStream>, BackendError> {
        let counters = Arc::new(Counters::default());
        let tap = TapRenderer {
            inner: renderer,
            counters: counters.clone(),
            dsd: config.dsd.is_some(),
        };
        let inner = self
            .inner
            .open_output(device, config, Box::new(tap), errors)?;
        Ok(Box::new(TapStream {
            inner: Some(inner),
            counters,
            probe: self.probe.clone(),
            proc_dir: self.proc_dir.clone(),
        }))
    }
}

fn sine_wav(dir: &Path, name: &str, rate: u32, secs: f64) -> PathBuf {
    let path = dir.join(name);
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: rate,
        bits_per_sample: 24,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(&path, spec).unwrap();
    let frames = (secs * f64::from(rate)) as usize;
    for i in 0..frames {
        let v = PCM_AMPLITUDE * (2.0 * PI * 1000.0 * i as f64 / f64::from(rate)).sin();
        let s = (v * 8_388_608.0).round() as i32;
        w.write_sample(s).unwrap();
        w.write_sample(s).unwrap();
    }
    w.finalize().unwrap();
    path
}

/// A stereo DSD64 DSF of a quiet 1 kHz tone (second-order modulator).
fn sine_dsf(dir: &Path, name: &str, secs: f64) -> PathBuf {
    let bits = (secs * f64::from(DSD64)) as usize / 8 * 8;
    let (mut i1, mut i2, mut y) = (0.0f64, 0.0f64, 0.0f64);
    let mut bytes = Vec::with_capacity(bits / 8);
    let mut byte = 0u8;
    for n in 0..bits {
        let x = DSD_AMPLITUDE * (2.0 * PI * 1000.0 * n as f64 / f64::from(DSD64)).sin();
        i1 += x - y;
        i2 += i1 - y;
        y = if i2 >= 0.0 { 1.0 } else { -1.0 };
        byte = (byte << 1) | u8::from(y > 0.0);
        if n % 8 == 7 {
            bytes.push(byte);
            byte = 0;
        }
    }
    // DSF: 4096-byte blocks per channel, least significant bit first.
    const BLOCK: usize = 4096;
    let blocks = bytes.len().div_ceil(BLOCK);
    let mut data = Vec::new();
    for g in 0..blocks {
        for _ in 0..2 {
            let mut block = vec![0x69u8.reverse_bits(); BLOCK];
            for (i, b) in bytes.iter().skip(g * BLOCK).take(BLOCK).enumerate() {
                block[i] = b.reverse_bits();
            }
            data.extend(block);
        }
    }
    let mut f = Vec::new();
    f.extend(b"DSD ");
    f.extend(28u64.to_le_bytes());
    f.extend((28 + 52 + 12 + data.len() as u64).to_le_bytes());
    f.extend(0u64.to_le_bytes());
    f.extend(b"fmt ");
    f.extend(52u64.to_le_bytes());
    f.extend(1u32.to_le_bytes());
    f.extend(0u32.to_le_bytes());
    f.extend(2u32.to_le_bytes());
    f.extend(2u32.to_le_bytes());
    f.extend(DSD64.to_le_bytes());
    f.extend(1u32.to_le_bytes());
    f.extend((bytes.len() as u64 * 8).to_le_bytes());
    f.extend((BLOCK as u32).to_le_bytes());
    f.extend(0u32.to_le_bytes());
    f.extend(b"data");
    f.extend((12 + data.len() as u64).to_le_bytes());
    f.extend(data);
    let path = dir.join(name);
    std::fs::write(&path, f).unwrap();
    path
}

fn pcm(rate: u32) -> Option<AudioFormat> {
    Some(AudioFormat {
        sample_rate: rate,
        bits: Some(24),
        channels: 2,
        dsd_rate: None,
    })
}

fn dsd64() -> Option<AudioFormat> {
    Some(AudioFormat {
        sample_rate: DSD64 / 32,
        bits: None,
        channels: 2,
        dsd_rate: Some(DSD64),
    })
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

struct Rig {
    engine: Engine,
    probe: Arc<Probe>,
    events: Vec<EngineEvent>,
    tick: Duration,
}

impl Rig {
    fn run_for(&mut self, secs: f64) {
        let end = Instant::now() + Duration::from_secs_f64(secs);
        while Instant::now() < end {
            let events = self.engine.tick(Instant::now());
            self.events.extend(events);
            std::thread::sleep(self.tick);
        }
    }

    fn act(&mut self, action: EngineAction) {
        self.engine.execute(action, Instant::now());
    }

    fn label(&self, label: &str) {
        *self.probe.label.lock().unwrap() = label.to_owned();
    }
}

fn rig(backend: Arc<dyn AudioBackend>, device: &str, buffer: u32, dsd: Option<f64>) -> Rig {
    let probe = Arc::new(Probe::default());
    let card = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "AUDIO".to_owned());
    let tap = TapBackend {
        inner: backend,
        probe: probe.clone(),
        proc_dir: PathBuf::from(format!("/proc/asound/{card}/pcm0p/sub0")),
    };
    let out = OutputDevice {
        backend: "alsa".into(),
        device: device.into(),
    };
    let mut config = Config::default();
    config.outputs.backend = Some("alsa".into());
    config.outputs.buffer_frames = buffer;
    config.outputs.routes = vec![PlayerRoutes {
        player: P,
        main: Some(Route {
            backend: "alsa".into(),
            device: device.into(),
            first_channel: 0,
        }),
        cue: None,
    }];
    config.outputs.bit_perfect = vec![out];
    if let Some(silence) = dsd {
        config.outputs.dsd_output = vec![DsdDevice {
            backend: "alsa".into(),
            device: device.into(),
            mode: DsdOutput::Native,
        }];
        config.outputs.dsd_silence_ms = silence;
    }
    let settings = EngineSettings::from_config(&config);
    let tick = Duration::from_secs_f64(config.tuning.conductor_tick_ms / 1000.0);
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(tap)];
    let mut engine = Engine::new(backends, settings, file_opener());
    engine.execute(EngineAction::AddPlayer { player: P }, Instant::now());
    Rig {
        engine,
        probe,
        events: Vec::new(),
        tick,
    }
}

fn plan(at_secs: f64, rate_cut: Option<u32>) -> EngineAction {
    EngineAction::Schedule {
        player: P,
        plan: Some(TransitionPlan::StartNextAt {
            at_secs,
            fade_current_until_secs: None,
            rate_cut,
        }),
    }
}

fn print(reports: &[CloseReport]) {
    for r in reports {
        let rate = f64::from(r.config.sample_rate.max(1));
        let ms = |f: i64| f as f64 * 1000.0 / rate;
        let silence = r.silence_after_sound as i64;
        match (r.delay, r.cut_frames()) {
            (Some(delay), Some(cut)) => println!(
                "  [{}] close at {} Hz{}: delay {delay} frames ({:.1} ms), silence after the last sound {silence} frames ({:.1} ms), {:.1} ms after it was rendered → sound discarded {cut} frames ({:.2} ms) [{}; {} frames in {} calls over {:.2} s; hw_ptr {:?} appl_ptr {:?}; reported held {:?}]",
                r.label,
                r.config.sample_rate,
                if r.config.dsd.is_some() {
                    " (DSD words)"
                } else {
                    ""
                },
                ms(delay),
                ms(silence),
                r.since_sound.as_secs_f64() * 1000.0,
                ms(cut),
                r.hw_params,
                r.rendered,
                r.calls,
                r.open_for.as_secs_f64(),
                r.hw_ptr,
                r.appl_ptr,
                r.reported,
            ),
            _ => println!(
                "  [{}] close at {} Hz: no kernel status (stream not running?)",
                r.label, r.config.sample_rate
            ),
        }
    }
}

fn main() {
    let buffer: u32 = std::env::args()
        .nth(2)
        .and_then(|b| b.parse().ok())
        .unwrap_or(512);
    let card = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "AUDIO".to_owned());
    let alsa = fp_backends::system_backends()
        .into_iter()
        .find(|b| b.id().0 == "alsa")
        .expect("no ALSA host");
    let devices = alsa.enumerate_devices().unwrap();
    let wanted = format!("hw:CARD={card},DEV=0");
    let device = devices
        .iter()
        .find(|d| d.id.0.ends_with(&wanted))
        .unwrap_or_else(|| {
            for d in &devices {
                eprintln!("  {}", d.id.0);
            }
            panic!("no {wanted} device");
        })
        .id
        .0
        .clone();
    println!("device {device}, buffer {buffer} frames");
    let dir = tempfile::tempdir().unwrap();
    let a = sine_wav(dir.path(), "a.wav", 44_100, 1.2);
    let b = sine_wav(dir.path(), "b.wav", 96_000, 0.6);
    let rounds: usize = std::env::var("ROUNDS")
        .ok()
        .and_then(|r| r.parse().ok())
        .unwrap_or(5);

    println!("BR6 scheduled cut 44.1 kHz → 96 kHz, mid-file (declick then stop):");
    for round in 0..rounds {
        let mut r = rig(alsa.clone(), &device, buffer, None);
        r.label("open");
        r.act(EngineAction::StartCurrent {
            player: P,
            request: request(1, a.clone(), pcm(44_100)),
        });
        r.act(EngineAction::Preload {
            player: P,
            request: Some(request(2, b.clone(), pcm(96_000))),
        });
        r.run_for(0.2);
        r.act(plan(0.6 + 0.01 * round as f64, Some(96_000)));
        r.label("cut");
        r.run_for(0.9);
        r.label("end");
        for b in r.engine.bus_status() {
            println!("  xruns {}", b.counters.xruns);
        }
        drop(r.engine);
        print(&r.probe.reports.lock().unwrap());
    }

    println!("BR8 hard start 44.1 kHz → 96 kHz (declick, then the cut):");
    for round in 0..rounds {
        let mut r = rig(alsa.clone(), &device, buffer, None);
        r.label("open");
        r.act(EngineAction::StartCurrent {
            player: P,
            request: request(1, a.clone(), pcm(44_100)),
        });
        r.run_for(0.4 + 0.007 * round as f64);
        r.label("cut");
        r.act(EngineAction::StartCurrent {
            player: P,
            request: request(2, b.clone(), pcm(96_000)),
        });
        r.run_for(0.5);
        r.label("end");
        for b in r.engine.bus_status() {
            println!("  xruns {}", b.counters.xruns);
        }
        drop(r.engine);
        print(&r.probe.reports.lock().unwrap());
    }

    println!("BR6 scheduled cut at the end of the file (no declick):");
    let short = sine_wav(dir.path(), "short.wav", 44_100, 0.5);
    for _ in 0..rounds {
        let mut r = rig(alsa.clone(), &device, buffer, None);
        r.label("open");
        r.act(EngineAction::StartCurrent {
            player: P,
            request: request(1, short.clone(), pcm(44_100)),
        });
        r.act(EngineAction::Preload {
            player: P,
            request: Some(request(2, b.clone(), pcm(96_000))),
        });
        r.run_for(0.1);
        r.act(plan(0.5, Some(96_000)));
        r.label("cut");
        r.run_for(0.9);
        r.label("end");
        for b in r.engine.bus_status() {
            println!("  xruns {}", b.counters.xruns);
        }
        drop(r.engine);
        print(&r.probe.reports.lock().unwrap());
    }

    let tone = sine_dsf(dir.path(), "d.dsf", 0.5);
    for silence in [fp_model::dsd::DEFAULT_DSD_SILENCE_MS, 0.0] {
        println!("native DSD64 track ending, DSD silence {silence} ms, back to PCM:");
        for _ in 0..rounds.min(3) {
            let mut r = rig(alsa.clone(), &device, buffer, Some(silence));
            r.label("open");
            r.act(EngineAction::StartCurrent {
                player: P,
                request: request(1, tone.clone(), dsd64()),
            });
            r.label("dsd end");
            r.run_for(1.2);
            r.label("end");
            let dsd = r
                .events
                .iter()
                .any(|e| matches!(e, EngineEvent::DsdStarted { .. }));
            drop(r.engine);
            println!("  (DSD carried unchanged: {dsd})");
            print(&r.probe.reports.lock().unwrap());
        }
    }
}
