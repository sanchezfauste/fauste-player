//! A `Bus` is one open output device and its mixer (spec §4.6–4.7). If the
//! device fails — an error from the backend, or no heartbeat for
//! `watchdog_timeout` — a virtual-clock thread keeps rendering the same mixer
//! at real-time pace, so player timelines never stall; the device is retried
//! every `reconnect_interval` and takes the mixer back when it returns.
//!
//! Every method takes the current `Instant` explicitly so tests can drive time.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use fp_backends::dsd::{DopEncoder, DsdStream};
use fp_backends::{
    AudioBackend, BackendError, DeviceId, OutputStream, StreamConfig, StreamErrorSink,
};

use crate::mixer::{
    BusCommand, BusEvent, Mixer, MixerConfig, MixerHandle, MixerRenderer, Retired, SlotStorage,
};

/// Identifies an output device across backends.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BusKey {
    pub backend: String,
    pub device: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusHealth {
    Ok,
    Lost,
}

#[derive(Debug, Clone, Copy)]
pub struct BusTiming {
    pub watchdog_timeout: Duration,
    pub reconnect_interval: Duration,
    /// Watchdog timeout before a newly opened stream's first block.
    pub startup_grace: Duration,
}

struct VirtualClock {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl VirtualClock {
    fn start(mixer: Arc<Mutex<Mixer>>, config: StreamConfig) -> Option<Self> {
        let stop = Arc::new(AtomicBool::new(false));
        let flag = stop.clone();
        let channels = usize::from(config.channels.max(1));
        let frames = config.buffer_frames.max(1) as usize;
        let period = Duration::from_secs_f64(
            f64::from(config.buffer_frames.max(1)) / f64::from(config.sample_rate.max(1)),
        );
        let thread = std::thread::Builder::new()
            .name("fp-virtual-clock".to_owned())
            .spawn(move || {
                let mut buffer = vec![0.0f32; frames * channels];
                let mut deadline = Instant::now();
                while !flag.load(Ordering::Acquire) {
                    mixer
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner)
                        .render(&mut buffer, channels);
                    deadline += period;
                    let now = Instant::now();
                    if deadline > now {
                        std::thread::sleep(deadline - now);
                    } else {
                        deadline = now;
                    }
                }
            })
            .ok()?;
        Some(Self {
            stop,
            thread: Some(thread),
        })
    }
}

impl Drop for VirtualClock {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

pub struct Bus {
    key: BusKey,
    backend: Arc<dyn AudioBackend>,
    device: DeviceId,
    config: StreamConfig,
    timing: BusTiming,
    mixer: Arc<Mutex<Mixer>>,
    handle: MixerHandle,
    stream: Option<Box<dyn OutputStream>>,
    virtual_clock: Option<VirtualClock>,
    health: BusHealth,
    last_heartbeat: u64,
    last_beat_at: Instant,
    last_retry: Instant,
    /// Conductor-side view of which slots are in use.
    used: Vec<bool>,
    /// Last error from the backend, for the UI.
    last_error: Option<String>,
    /// Heartbeat value when the current stream was opened.
    opened_beat: u64,
    /// Whether the open stream got the exclusive access it asked for.
    exclusive_granted: bool,
    /// Rates this device refused, not asked for again until it comes back
    /// from a loss (it may be another device by then).
    refused_rates: std::collections::HashSet<u32>,
}

impl Bus {
    /// Opens the device. Never fails: if the device cannot be opened the bus
    /// starts `Lost`, running on the virtual clock and retrying.
    pub fn open(
        key: BusKey,
        backend: Arc<dyn AudioBackend>,
        config: StreamConfig,
        slots: usize,
        mixer_config: MixerConfig,
        timing: BusTiming,
        now: Instant,
    ) -> Bus {
        let (mixer, handle) = Mixer::new(slots, mixer_config);
        let device = DeviceId(key.device.clone());
        let mut bus = Bus {
            key,
            backend,
            device,
            config,
            timing,
            mixer: Arc::new(Mutex::new(mixer)),
            handle,
            stream: None,
            virtual_clock: None,
            health: BusHealth::Lost,
            last_heartbeat: 0,
            last_beat_at: now,
            last_retry: now,
            used: vec![false; slots],
            last_error: None,
            opened_beat: 0,
            exclusive_granted: false,
            refused_rates: std::collections::HashSet::new(),
        };
        if !bus.try_open(now, true) {
            bus.virtual_clock = VirtualClock::start(bus.mixer.clone(), bus.config);
        }
        bus
    }

    /// A fresh renderer for a stream opening: a DoP stream gets its own
    /// encoder, so the marker alternation starts over with the stream.
    fn renderer(&self) -> Box<MixerRenderer> {
        Box::new(MixerRenderer {
            mixer: self.mixer.clone(),
            shared: self.handle.shared.clone(),
            dop: (self.config.dsd == Some(DsdStream::Dop)).then(DopEncoder::new),
        })
    }

    /// Opens the stream. With `shared_fallback`, a device that refuses
    /// exclusive access opens shared; a rate change passes `false`, since
    /// there the refusal is of the rate, not of exclusive access.
    fn try_open(&mut self, now: Instant, shared_fallback: bool) -> bool {
        self.last_retry = now;
        self.handle.shared.lost.store(false, Ordering::Release);
        // Sources attached from now on measure at this rate (K-weighting).
        self.handle
            .shared
            .sample_rate
            .store(self.config.sample_rate, Ordering::Release);
        let renderer = self.renderer();
        let errors: Arc<dyn StreamErrorSink> = self.handle.shared.clone();
        let mut opened =
            self.backend
                .open_output(&self.device, self.config, renderer, errors.clone());
        self.exclusive_granted = self.config.exclusive && opened.is_ok();
        if shared_fallback
            && self.config.exclusive
            && matches!(opened, Err(BackendError::Unsupported(_)))
        {
            // Bit-perfect needs exclusive access; without it the device must
            // still play (Phase 4 spec B4).
            if let Err(e) = &opened {
                tracing::warn!(bus = ?self.key, error = %e, "exclusive access refused; opening shared");
            }
            let renderer = self.renderer();
            let shared = StreamConfig {
                exclusive: false,
                ..self.config
            };
            opened = self
                .backend
                .open_output(&self.device, shared, renderer, errors);
        }
        match opened {
            Ok(stream) => {
                self.stream = Some(stream);
                self.health = BusHealth::Ok;
                self.last_heartbeat = self.handle.shared.heartbeat();
                self.opened_beat = self.last_heartbeat;
                self.last_beat_at = now;
                self.last_error = None;
                true
            }
            Err(e) => {
                self.last_error = Some(e.to_string());
                false
            }
        }
    }

    pub fn key(&self) -> &BusKey {
        &self.key
    }

    pub fn health(&self) -> BusHealth {
        self.health
    }

    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    pub fn sample_rate(&self) -> u32 {
        self.config.sample_rate
    }

    /// Whether the device is open with the exclusive access it asked for.
    pub fn exclusive_granted(&self) -> bool {
        self.health == BusHealth::Ok && self.exclusive_granted
    }

    /// The open stream's sample format, if a device stream is open.
    pub fn sample_format(&self) -> Option<fp_backends::SampleFormat> {
        self.stream.as_ref().map(|s| s.sample_format())
    }

    /// Reopens the device at `rate`, keeping the mixer, its clock and its
    /// slots. Only for a bus with nothing attached: every timeline on a bus
    /// is in its frames (Phase 4 spec B3). If the device refuses the rate,
    /// the previous one is restored; if that fails too the bus is `Lost`
    /// and the watchdog takes over. Returns whether `rate` took.
    pub fn reopen_at(&mut self, rate: u32, now: Instant) -> bool {
        let previous = self.config.sample_rate;
        if rate == previous {
            return true;
        }
        if self.refused_rates.contains(&rate) {
            return false;
        }
        self.stream = None;
        self.virtual_clock = None;
        self.config.sample_rate = rate;
        self.follow_rate(previous, rate);
        // On an exclusive stream, a refusal is of the rate: keep exclusive
        // access at the previous rate. A device that is shared anyway may
        // change rate shared.
        let shared_fallback = !self.exclusive_granted;
        if self.try_open(now, shared_fallback) {
            tracing::info!(bus = ?self.key, rate, "stream rate follows the file");
            return true;
        }
        tracing::warn!(bus = ?self.key, rate, error = ?self.last_error, "rate refused; keeping the previous one");
        self.refused_rates.insert(rate);
        self.config.sample_rate = previous;
        self.follow_rate(rate, previous);
        if !self.try_open(now, true) {
            self.health = BusHealth::Lost;
            self.virtual_clock = VirtualClock::start(self.mixer.clone(), self.config);
        }
        false
    }

    /// Keeps the mixer's frame-based durations at their length in time.
    fn follow_rate(&self, from_rate: u32, to_rate: u32) {
        self.mixer
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .follow_rate(from_rate, to_rate);
    }

    /// Current bus time in frames.
    pub fn now_frame(&self) -> u64 {
        self.handle.shared.frames_rendered()
    }

    pub fn shared(&self) -> &Arc<crate::mixer::BusShared> {
        &self.handle.shared
    }

    /// Watchdog and reconnection. Call regularly from the conductor.
    pub fn supervise(&mut self, now: Instant) {
        let beat = self.handle.shared.heartbeat();
        if beat != self.last_heartbeat {
            self.last_heartbeat = beat;
            self.last_beat_at = now;
        }
        match self.health {
            BusHealth::Ok => {
                // Until a newly opened stream delivers its first block it gets a
                // longer grace (Bluetooth and bridged devices can be slow to start).
                let timeout = if beat == self.opened_beat {
                    self.timing.startup_grace
                } else {
                    self.timing.watchdog_timeout
                };
                let silent = now.saturating_duration_since(self.last_beat_at) > timeout;
                if self.handle.shared.lost.load(Ordering::Acquire) || silent {
                    tracing::warn!(bus = ?self.key, silent, "output device lost; switching to the virtual clock");
                    self.stream = None;
                    self.health = BusHealth::Lost;
                    self.last_retry = now;
                    self.virtual_clock = VirtualClock::start(self.mixer.clone(), self.config);
                }
            }
            BusHealth::Lost => {
                if now.saturating_duration_since(self.last_retry) >= self.timing.reconnect_interval
                    && self.try_open(now, true)
                {
                    // The stand-in keeps the timeline moving while the device
                    // opens (which can take a while); it stops once the device
                    // renders. Both may render one block meanwhile: the
                    // timeline runs at most one period fast, once.
                    tracing::info!(bus = ?self.key, "output device back");
                    self.virtual_clock = None;
                    self.refused_rates.clear();
                }
            }
        }
    }

    /// Reserves a free slot.
    pub fn alloc_slot(&mut self) -> Option<usize> {
        let slot = self.used.iter().position(|u| !u)?;
        if let Some(u) = self.used.get_mut(slot) {
            *u = true;
        }
        Some(slot)
    }

    /// Free places in the command queue.
    pub fn command_room(&self) -> usize {
        self.handle.commands.slots()
    }

    pub fn used_slots(&self) -> usize {
        self.used.iter().filter(|u| **u).count()
    }

    /// Sends a command; `false` if the queue is full (the caller logs it).
    pub fn send(&mut self, command: BusCommand) -> bool {
        self.handle.commands.push(command).is_ok()
    }

    /// Grows the mixer to at least `slots` slots without interrupting audio.
    pub fn ensure_capacity(&mut self, slots: usize) {
        if slots <= self.used.len() {
            return;
        }
        if self.send(BusCommand::Grow(SlotStorage::with_capacity(slots))) {
            self.used.resize(slots, false);
        }
    }

    pub fn capacity(&self) -> usize {
        self.used.len()
    }

    /// Drains events, and frees memory and slots the mixer handed back.
    pub fn poll(&mut self) -> Vec<BusEvent> {
        while let Ok(retired) = self.handle.retired.pop() {
            // A refused attach (`slot: None`) never occupied a slot: nothing to free.
            if let Retired::Source {
                slot: Some(slot), ..
            } = retired
                && let Some(u) = self.used.get_mut(slot)
            {
                *u = false;
            }
        }
        std::iter::from_fn(|| self.handle.events.pop().ok()).collect()
    }
}
