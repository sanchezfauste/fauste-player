//! A `Bus` is one open output device and its mixer (spec §4.6–4.7). If the
//! device fails — an error from the backend, or no block asked for within
//! `watchdog_timeout` — a virtual-clock thread keeps rendering the same mixer
//! at real-time pace, so player timelines never stall; the device is retried
//! every `reconnect_interval` and takes the mixer back once the reopened
//! stream asks for its first block. Until then the stream plays silence
//! and the bus stays `Lost`: a stream that opens but never starts (its
//! card held by another output) is closed after `startup_grace` and
//! retried, reported once.
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

impl From<&fp_model::OutputDevice> for BusKey {
    fn from(device: &fp_model::OutputDevice) -> Self {
        Self {
            backend: device.backend.clone(),
            device: device.device.clone(),
        }
    }
}

impl BusKey {
    /// The model's name for this device.
    pub fn output_device(&self) -> fp_model::OutputDevice {
        fp_model::OutputDevice {
            backend: self.backend.clone(),
            device: self.device.clone(),
        }
    }
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
    /// How many more times a rate change tries a device that answered
    /// busy, and how long it waits before each try.
    pub busy_retries: u32,
    pub busy_retry_interval: Duration,
}

/// The busy retries one top-level reopen may still spend, each after
/// `BusTiming::busy_retry_interval`: the open at the new configuration and
/// the restore of the previous one share it, and so do the reopens an
/// engine operation chains. So the conductor waits at most
/// `busy_retries × busy_retry_interval` per operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BusyBudget {
    retries: u32,
}

impl BusyBudget {
    /// No retries: a busy open fails at once.
    pub const NONE: BusyBudget = BusyBudget { retries: 0 };

    pub fn retries(&self) -> u32 {
        self.retries
    }
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
    /// The device's block count (`BusShared::device_blocks`) last seen.
    last_heartbeat: u64,
    last_beat_at: Instant,
    last_retry: Instant,
    /// Conductor-side view of which slots are in use.
    used: Vec<bool>,
    /// Last error from the backend, for the UI.
    last_error: Option<String>,
    /// The device's block count when the current stream was opened.
    opened_beat: u64,
    /// A reopened stream that never started was reported in this loss.
    unstarted_reported: bool,
    /// Whether the open stream got the exclusive access it asked for.
    exclusive_granted: bool,
    /// Rates this device refused, not asked for again until it comes back
    /// from a loss (it may be another device by then).
    refused_rates: std::collections::HashSet<u32>,
    /// DSD streams (word rate, kind) this device refused, forgotten with
    /// `refused_rates` (feedback 2 spec O25).
    refused_dsd: std::collections::HashSet<(u32, DsdStream)>,
    /// The device came back from a loss unable to carry the DSD stream it
    /// had, and was reopened as PCM (read once by `take_dsd_lost`).
    dsd_lost: bool,
    /// The PCM configuration the watchdog falls back to when the stored
    /// one does not open: the one the device ran before it carried DSD (a
    /// device that takes native DSD at a word rate need not take that rate
    /// as PCM), or the global rate and buffer for a device with its own
    /// (operator feedback 4, Q12). Once the bus changes PCM rate by itself,
    /// only a device's own buffer keeps one: the new rate with the global
    /// buffer.
    pcm_fallback: Option<StreamConfig>,
    /// The watchdog reopened the device at `pcm_fallback`: the rate the
    /// bus ran before (read once by `take_rate_change`).
    rate_change: Option<u32>,
    /// The last open failed because the device was not there.
    missing: bool,
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
            unstarted_reported: false,
            exclusive_granted: false,
            refused_rates: std::collections::HashSet::new(),
            refused_dsd: std::collections::HashSet::new(),
            dsd_lost: false,
            pcm_fallback: None,
            rate_change: None,
            missing: false,
        };
        if bus.try_open(now, true).is_err() {
            // The stand-in renders the mixer without DoP encoding or native
            // DSD packing: nothing it renders reaches a converter.
            bus.start_virtual_clock();
        }
        bus
    }

    /// A fresh renderer for a stream opening: a DoP stream gets its own
    /// encoder, so the marker alternation starts over with the stream.
    fn renderer(&self) -> Box<MixerRenderer> {
        Box::new(MixerRenderer::new(
            self.mixer.clone(),
            self.handle.shared.clone(),
            (self.config.dsd == Some(DsdStream::Dop)).then(DopEncoder::new),
            self.config.dsd == Some(DsdStream::Native),
        ))
    }

    /// Opens the stream. With `shared_fallback`, a device that refuses
    /// exclusive access opens shared; a rate change passes `false`, since
    /// there the refusal is of the rate, not of exclusive access.
    ///
    /// While the virtual clock runs (a reopen after a loss) the new stream
    /// starts in standby and the bus stays `Lost`: `supervise` hands the
    /// mixer over at the stream's first block.
    fn try_open(&mut self, now: Instant, shared_fallback: bool) -> Result<(), BackendError> {
        self.last_retry = now;
        self.handle.shared.lost.store(false, Ordering::Release);
        // Never two renderers of one mixer: the device waits for the
        // virtual clock to stop.
        self.handle
            .shared
            .standby
            .store(self.virtual_clock.is_some(), Ordering::Release);
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
            && matches!(
                opened,
                Err(BackendError::Unsupported(_) | BackendError::Busy(_))
            )
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
        self.missing = matches!(opened, Err(BackendError::DeviceNotFound(_)));
        match opened {
            Ok(stream) => {
                self.stream = Some(stream);
                if self.virtual_clock.is_none() {
                    self.health = BusHealth::Ok;
                }
                self.last_heartbeat = self.handle.shared.device_blocks();
                self.opened_beat = self.last_heartbeat;
                self.last_beat_at = now;
                self.last_error = None;
                Ok(())
            }
            Err(e) => {
                self.last_error = Some(e.to_string());
                Err(e)
            }
        }
    }

    /// `try_open`, tried again while the device answers busy and `budget`
    /// has retries left, `busy_retry_interval` apart: closing a stream to
    /// change its rate leaves a gap in which another client (a sound server
    /// probing the device) can take it for a moment. Each retry spends one
    /// from `budget`, so one operation blocks the caller for at most
    /// `busy_retries × busy_retry_interval`, on top of the opens themselves.
    fn open_retrying_busy(
        &mut self,
        now: Instant,
        shared_fallback: bool,
        budget: &mut BusyBudget,
    ) -> Result<(), BackendError> {
        let started = Instant::now();
        let mut result = self.try_open(now, shared_fallback);
        while matches!(result, Err(BackendError::Busy(_))) && budget.retries > 0 {
            budget.retries -= 1;
            if !self.timing.busy_retry_interval.is_zero() {
                std::thread::sleep(self.timing.busy_retry_interval);
            }
            // The caller's `now`, moved on by the time spent waiting.
            result = self.try_open(now + started.elapsed(), shared_fallback);
        }
        result
    }

    /// Starts the virtual clock for a bus that has just lost its stream: a
    /// new loss, whose first stream that never starts is reported again.
    fn start_virtual_clock(&mut self) {
        self.unstarted_reported = false;
        self.virtual_clock = VirtualClock::start(self.mixer.clone(), self.config);
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

    /// The configuration the stream is asked to open with (and reopened
    /// with after a loss).
    pub fn config(&self) -> StreamConfig {
        self.config
    }

    /// The buffer the open stream runs with (a device may choose its own
    /// when it does not take the one asked for), else the one asked for.
    pub fn buffer_frames(&self) -> u32 {
        self.stream
            .as_ref()
            .map_or(self.config.buffer_frames, |s| s.config().buffer_frames)
    }

    /// How the open stream carries DSD, if a device stream is open.
    pub fn stream_dsd(&self) -> Option<DsdStream> {
        self.stream.as_ref().and_then(|s| s.dsd())
    }

    /// Closes the device stream (the mixer keeps its state) so that nothing
    /// renders until the next open; the caller reopens at once.
    pub fn close_stream(&mut self) {
        self.stream = None;
    }

    /// Whether the open stream can carry `stream` unchanged: exclusive
    /// access, and a 24- or 32-bit integer format for DoP (whatever the
    /// backend accepted) or a stream the backend packs as native DSD.
    pub fn dsd_fits(&self, stream: DsdStream) -> bool {
        // Asked of a stream just reopened too, before its first block.
        self.stream.is_some()
            && self.exclusive_granted
            && match stream {
                DsdStream::Dop => matches!(
                    self.sample_format(),
                    Some(fp_backends::SampleFormat::I24 | fp_backends::SampleFormat::I32)
                ),
                DsdStream::Native => self.stream_dsd() == Some(DsdStream::Native),
            }
    }

    /// Whether the device came back from a loss as PCM instead of the DSD
    /// stream it carried (see `reconnect`); cleared by the call.
    pub fn take_dsd_lost(&mut self) -> bool {
        std::mem::take(&mut self.dsd_lost)
    }

    /// Remembers the PCM configuration to fall back to after DSD (see
    /// `pcm_fallback`).
    pub fn set_pcm_fallback(&mut self, config: StreamConfig) {
        self.pcm_fallback = Some(config);
    }

    /// Replaces the PCM fallback after a device change (live settings spec
    /// L12), as `Engine::ensure_bus` sets it for a new bus.
    pub fn replace_pcm_fallback(&mut self, fallback: Option<StreamConfig>) {
        self.pcm_fallback = fallback;
    }

    /// The rate the bus ran before the watchdog reopened it at another one
    /// (`pcm_fallback`); the sources on it were opened for that rate.
    /// Cleared by the call.
    pub fn take_rate_change(&mut self) -> Option<u32> {
        self.rate_change.take()
    }

    /// The watchdog's reopen with the stored configuration. A DSD stream
    /// comes back only with exclusive access and a stream that fits it
    /// (`dsd_fits`); otherwise the present device is reopened as PCM at the
    /// same rate (never left closed), the mixer leaves DSD mode before the
    /// first block, and `dsd_lost` tells the engine. A PCM configuration
    /// the device refuses (`Unsupported`) falls back to `pcm_fallback`
    /// (`open_pcm_fallback`); a device missing or busy for a moment is
    /// asked for the same configuration again next time.
    fn reconnect(&mut self, now: Instant) -> bool {
        let Some(dsd) = self.config.dsd else {
            return self.open_or_fall_back(now);
        };
        // A DoP stream whose DSD has ended keeps its configuration while the
        // mixer is PCM: only a mixer in DSD mode is put back in it below.
        let was_on = self.handle.shared.dsd_on.load(Ordering::Acquire);
        if self.try_open(now, false).is_ok() {
            if self.dsd_fits(dsd) {
                return true;
            }
            self.stream = None;
        }
        // Out of DSD mode before any block reaches a PCM stream.
        self.send(BusCommand::DsdMode {
            on: false,
            at_frame: 0,
        });
        let dsd_config = self.config;
        self.config.dsd = None;
        if self.open_or_fall_back(now) {
            tracing::warn!(bus = ?self.key, ?dsd, "the device came back unable to carry DSD; playing PCM");
            self.dsd_lost = true;
            return true;
        }
        // Not there at all: try the DSD stream again next time. A stream
        // opened above and dropped as unfit leaves no stream: lost.
        self.health = BusHealth::Lost;
        self.config = dsd_config;
        if was_on {
            self.send(BusCommand::DsdMode {
                on: true,
                at_frame: 0,
            });
        }
        false
    }

    /// Opens the stored PCM configuration, or `pcm_fallback` when the
    /// device refuses it (operator feedback 4, Q12).
    fn open_or_fall_back(&mut self, now: Instant) -> bool {
        match self.try_open(now, true) {
            Ok(()) => true,
            Err(BackendError::Unsupported(_)) => self.open_pcm_fallback(now),
            Err(_) => false,
        }
    }

    /// After the stored PCM configuration did not open: opens the device
    /// with `pcm_fallback` when that has another rate or buffer, keeping
    /// the mixer's durations in time; on a rate change `rate_change` tells
    /// the engine, which opens the sources again at the new rate (a buffer
    /// change alone leaves them as they are). Otherwise the configuration
    /// stays.
    fn open_pcm_fallback(&mut self, now: Instant) -> bool {
        let Some(fallback) = self.pcm_fallback.filter(|f| {
            f.sample_rate != self.config.sample_rate || f.buffer_frames != self.config.buffer_frames
        }) else {
            return false;
        };
        let previous = self.config;
        self.config = fallback;
        self.follow_rate(previous.sample_rate, fallback.sample_rate);
        if self.try_open(now, true).is_ok() {
            if previous.sample_rate == fallback.sample_rate {
                tracing::warn!(bus = ?self.key, from = previous.buffer_frames, to = fallback.buffer_frames, "the device did not take its buffer size; playing with the fallback buffer");
            } else {
                tracing::warn!(bus = ?self.key, from = previous.sample_rate, to = fallback.sample_rate, buffer = fallback.buffer_frames, "the device did not take its rate; playing at the fallback rate");
                self.rate_change = Some(previous.sample_rate);
            }
            return true;
        }
        self.config = previous;
        self.follow_rate(fallback.sample_rate, previous.sample_rate);
        false
    }

    /// Remembers that the device cannot carry `stream` at `word_rate` (the
    /// engine found the opened stream unfit), until it comes back from a loss.
    pub fn refuse_dsd(&mut self, word_rate: u32, stream: DsdStream) {
        self.refused_dsd.insert((word_rate, stream));
    }

    /// Reopens the device at `rate` as PCM, keeping the mixer, its clock and
    /// its slots (see `reopen_with`). Returns whether `rate` took.
    pub fn reopen_at(&mut self, rate: u32, now: Instant, budget: &mut BusyBudget) -> bool {
        if rate == self.config.sample_rate {
            return true;
        }
        let config = StreamConfig {
            sample_rate: rate,
            dsd: None,
            ..self.config
        };
        self.reopen_with(config, now, budget).is_ok()
    }

    /// Reopens the device with `config`, keeping the mixer, its clock and
    /// its slots. Only for a bus whose timelines can follow the new rate:
    /// every timeline on a bus is in its frames (Phase 4 spec B3). If the
    /// device refuses, the refusal is remembered (a rate, or a DSD stream at
    /// a rate) and not asked again until the device comes back from a loss,
    /// and the previous configuration is restored. A device that stays busy
    /// through `busy_retries`, or is not there, refused nothing: the previous configuration is
    /// restored without remembering it. If the restore fails too the
    /// bus is `Lost` and the watchdog takes over. Leaving native DSD never
    /// restores it (PCM would reach a DSD stream): the bus stays `Lost` with
    /// the PCM configuration instead.
    pub fn reopen_with(
        &mut self,
        config: StreamConfig,
        now: Instant,
        budget: &mut BusyBudget,
    ) -> Result<(), String> {
        let previous = self.config;
        if config == previous && self.stream.is_some() {
            return Ok(());
        }
        let refused = match config.dsd {
            Some(dsd) => self.refused_dsd.contains(&(config.sample_rate, dsd)),
            None => self.refused_rates.contains(&config.sample_rate),
        };
        if refused {
            return Err(format!(
                "{} Hz {:?} was refused before",
                config.sample_rate, config.dsd
            ));
        }
        self.stream = None;
        self.virtual_clock = None;
        self.config = config;
        self.follow_rate(previous.sample_rate, config.sample_rate);
        // On an exclusive stream, a refusal is of the rate: keep exclusive
        // access at the previous rate. A device that is shared anyway may
        // change rate shared. DSD never falls back to shared access.
        let shared_fallback = !self.exclusive_granted && config.dsd.is_none();
        let opened = self.open_retrying_busy(now, shared_fallback, budget);
        if opened.is_ok() {
            tracing::info!(bus = ?self.key, rate = config.sample_rate, dsd = ?config.dsd, "stream reopened");
            if previous.dsd.is_none() && config.dsd.is_none() {
                // Back in PCM, at a rate of its own choosing: a fallback to
                // another rate is forgotten, while a device's own buffer
                // keeps the global one at the new rate (operator feedback
                // 4, Q12).
                self.pcm_fallback = self
                    .pcm_fallback
                    .filter(|f| config.exact_buffer && f.buffer_frames != config.buffer_frames)
                    .map(|f| StreamConfig {
                        buffer_frames: f.buffer_frames,
                        exact_buffer: false,
                        ..config
                    });
            }
            return Ok(());
        }
        let error = self.last_error.clone().unwrap_or_default();
        match &opened {
            // Not a refusal of the configuration: the next start asks again.
            Err(BackendError::Busy(_)) => {
                tracing::warn!(bus = ?self.key, rate = config.sample_rate, dsd = ?config.dsd, %error, "device busy (another application or the sound server may be using it); keeping the previous stream");
            }
            Err(BackendError::DeviceNotFound(_)) => {
                tracing::warn!(bus = ?self.key, %error, "device not there; keeping the previous configuration");
            }
            _ => {
                tracing::warn!(bus = ?self.key, rate = config.sample_rate, dsd = ?config.dsd, %error, "stream refused; keeping the previous one");
                match config.dsd {
                    Some(dsd) => {
                        self.refused_dsd.insert((config.sample_rate, dsd));
                    }
                    None => {
                        self.refused_rates.insert(config.sample_rate);
                    }
                }
            }
        }
        let leaving_native = previous.dsd == Some(DsdStream::Native) && config.dsd.is_none();
        if !leaving_native {
            self.config = previous;
            self.follow_rate(config.sample_rate, previous.sample_rate);
        }
        if leaving_native || self.open_retrying_busy(now, true, budget).is_err() {
            self.health = BusHealth::Lost;
            self.start_virtual_clock();
        }
        Err(error)
    }

    /// A device change (live settings spec L12, L14, L15): `reopen_with`,
    /// except that a device that is not there, missing now or lost
    /// already, keeps `config` for when it returns: nothing is
    /// interrupted, the bus stays `Lost` on the virtual clock, and the
    /// watchdog opens `config` (falling back to `pcm_fallback` if refused).
    pub fn apply_config(
        &mut self,
        config: StreamConfig,
        now: Instant,
        budget: &mut BusyBudget,
    ) -> Result<(), String> {
        let absent = self.health == BusHealth::Lost && self.stream.is_none();
        if !absent {
            match self.reopen_with(config, now, budget) {
                Ok(()) => return Ok(()),
                Err(error) if !self.missing => return Err(error),
                Err(_) => {}
            }
        }
        let previous = self.config;
        self.stream = None;
        self.config = config;
        self.follow_rate(previous.sample_rate, config.sample_rate);
        self.health = BusHealth::Lost;
        // The stand-in clock runs at the new rate.
        self.virtual_clock = None;
        self.start_virtual_clock();
        tracing::info!(bus = ?self.key, "device not there; it opens with the new settings when it returns");
        Ok(())
    }

    /// Forgets the rates and DSD streams this device refused, so that the
    /// operator's Apply now asks it again (L14).
    pub fn forget_refusals(&mut self) {
        self.refused_rates.clear();
        self.refused_dsd.clear();
    }

    /// Keeps the mixer's frame-based durations at their length in time.
    fn follow_rate(&self, from_rate: u32, to_rate: u32) {
        if from_rate == to_rate {
            return;
        }
        self.mixer
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .follow_rate(from_rate, to_rate);
    }

    /// Current bus time in frames.
    pub fn now_frame(&self) -> u64 {
        self.handle.shared.frames_rendered()
    }

    /// The volume smoothing the mixer runs with, for tests of `Tune`.
    #[cfg(test)]
    pub(crate) fn mixer_volume_smoothing_frames(&self) -> u32 {
        self.mixer
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .volume_smoothing_frames()
    }

    pub fn shared(&self) -> &Arc<crate::mixer::BusShared> {
        &self.handle.shared
    }

    /// Watchdog and reconnection. Call regularly from the conductor.
    pub fn supervise(&mut self, now: Instant) {
        let beat = self.handle.shared.device_blocks();
        if beat != self.last_heartbeat {
            self.last_heartbeat = beat;
            self.last_beat_at = now;
        }
        match self.health {
            BusHealth::Ok => {
                // Until a newly opened stream delivers its first block it gets a
                // longer grace (Bluetooth and bridged devices can be slow to start).
                let started = beat != self.opened_beat;
                let timeout = if started {
                    self.timing.watchdog_timeout
                } else {
                    self.timing.startup_grace
                };
                let silent = now.saturating_duration_since(self.last_beat_at) > timeout;
                if self.handle.shared.lost.load(Ordering::Acquire) || silent {
                    tracing::warn!(bus = ?self.key, silent, started, "output device lost; switching to the virtual clock");
                    self.stream = None;
                    self.health = BusHealth::Lost;
                    self.last_retry = now;
                    self.start_virtual_clock();
                }
            }
            BusHealth::Lost if self.stream.is_some() => self.await_first_block(beat, now),
            BusHealth::Lost => {
                if now.saturating_duration_since(self.last_retry) >= self.timing.reconnect_interval
                    && self.reconnect(now)
                {
                    tracing::debug!(bus = ?self.key, "output device reopened; waiting for its first block");
                }
            }
        }
    }

    /// A stream reopened after a loss, in standby while the virtual clock
    /// renders: its first block hands the mixer over (the clock stops
    /// before the stream leaves standby, so the two never render together).
    /// One that asks for nothing within `startup_grace`, or fails, is
    /// closed and retried after `reconnect_interval`; the first such stream
    /// of a loss is reported.
    fn await_first_block(&mut self, beat: u64, now: Instant) {
        if beat != self.opened_beat {
            self.virtual_clock = None;
            self.handle.shared.standby.store(false, Ordering::Release);
            self.health = BusHealth::Ok;
            self.last_beat_at = now;
            tracing::info!(bus = ?self.key, "output device back");
            self.refused_rates.clear();
            self.refused_dsd.clear();
            return;
        }
        let failed = self.handle.shared.lost.load(Ordering::Acquire);
        if !failed && now.saturating_duration_since(self.last_retry) <= self.timing.startup_grace {
            return;
        }
        self.stream = None;
        self.last_retry = now;
        if !failed && !self.unstarted_reported {
            self.unstarted_reported = true;
            tracing::warn!(bus = ?self.key, "output device opened but never started; is another output holding the same card (a direct hw: output)? Retrying quietly");
        }
    }

    /// The busy budget of one reopen operation: `busy_retries` on an idle
    /// bus; none on a sounding one, whose timeline would stall while the
    /// conductor waits (its stream is closed and no virtual clock runs).
    pub fn busy_budget(&self, idle: bool) -> BusyBudget {
        if idle {
            BusyBudget {
                retries: self.timing.busy_retries,
            }
        } else {
            BusyBudget::NONE
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

    /// New watchdog, reconnection and busy-retry timing (live settings
    /// spec §7): the next `supervise` and the next reopen use it.
    pub fn set_timing(&mut self, timing: BusTiming) {
        self.timing = timing;
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
