//! The bus mixer (spec §4.3). It runs on the device's real-time thread: it
//! sums the sources attached to its slots into the output, applying gain ramps
//! at exact frames. It never allocates, frees, blocks, logs or panics: memory
//! only arrives and leaves through lock-free queues.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

use fp_backends::{Renderer, StreamErrorKind, StreamErrorSink};

use crate::atomic::AtomicF32;
use crate::kweight::KWeighting;
use crate::ramp::{Curve, Ramp};
use crate::source::{SOURCE_CHANNELS, SourceConsumer};
use crate::truepeak::TruePeak;

/// Frames processed per inner step; bounds the stack scratch buffer.
const CHUNK_FRAMES: usize = 256;

/// Work for the mixer. All frame numbers are in bus frames
/// (`BusShared::frames_rendered` time).
pub enum BusCommand {
    /// Puts a source in `slot`, silent until `Start`.
    Attach {
        slot: usize,
        source: SourceConsumer,
        volume: Arc<AtomicF32>,
        first_channel: u16,
    },
    /// Starts playing at `at_frame` (or immediately if it has passed).
    Start { slot: usize, at_frame: u64 },
    /// From `at_frame`, moves the fade gain to `to` over `frames` frames.
    Ramp {
        slot: usize,
        to: f32,
        frames: u32,
        curve: Curve,
        at_frame: u64,
    },
    /// A linear de-click ramp to silence from `at_frame` for a cut at the
    /// slot's stop frame, skipped when the source's own end of stream falls
    /// at or before that stop frame (a gapless join: the audio already ends
    /// there, so the ramp would only dip the level). Send `StopAt` too.
    RampOutBeforeCut {
        slot: usize,
        frames: u32,
        at_frame: u64,
    },
    /// Stops at `at_frame` and reports `Finished`.
    StopAt { slot: usize, at_frame: u64 },
    /// Fades out over `ramp_frames`, then holds the position.
    Pause { slot: usize, ramp_frames: u32 },
    /// Fades back in over `ramp_frames` from where it paused.
    Resume { slot: usize, ramp_frames: u32 },
    /// Forgets a pending start, ramp or stop that has not happened yet.
    Cancel { slot: usize },
    /// Removes the source; it comes back through the retired queue.
    Detach { slot: usize },
    /// Replaces the slot storage with a larger one (built off the RT thread).
    Grow(SlotStorage),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusEvent {
    Started {
        slot: usize,
        frame: u64,
    },
    /// The source ended (stop frame reached, or ring drained after eof).
    Finished {
        slot: usize,
        frame: u64,
    },
}

/// Memory handed back to the conductor to be freed off the RT thread.
pub enum Retired {
    Source {
        /// The slot the source occupied, or `None` if it was refused
        /// (occupied or out-of-range slot) and never attached.
        slot: Option<usize>,
        source: SourceConsumer,
        volume: Arc<AtomicF32>,
    },
    Storage(SlotStorage),
}

/// Slot storage, allocated by the conductor.
pub struct SlotStorage(Vec<Option<Slot>>);

impl SlotStorage {
    pub fn with_capacity(slots: usize) -> Self {
        Self((0..slots).map(|_| None).collect())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Bus telemetry, readable from any thread. Also receives backend errors.
#[derive(Debug, Default)]
pub struct BusShared {
    pub frames_rendered: AtomicU64,
    /// Odd while a block is being rendered (a sequence lock for readers).
    pub render_seq: AtomicU64,
    pub heartbeat: AtomicU64,
    pub lost: AtomicBool,
    pub realtime_denied: AtomicBool,
    pub xruns: AtomicU64,
    /// Stream errors the backend could not classify (not xruns).
    pub stream_errors: AtomicU64,
    pub dropped_events: AtomicU64,
    pub lock_misses: AtomicU64,
    /// Items that could not be handed back and were leaked instead of being
    /// freed on the real-time thread (should always be zero).
    pub leaked: AtomicU64,
    /// Blocks in which a source's channel pair did not fit the stream.
    pub misrouted: AtomicU64,
    pub peak_l: AtomicF32,
    pub peak_r: AtomicF32,
    /// The stream's rate, set by the bus on every open (K-weighting).
    pub sample_rate: AtomicU32,
    /// Measure true peak instead of sample peak (meters spec M1).
    pub true_peak: AtomicBool,
    /// Programme-meter integrator time constants in ms (0: none) and its
    /// fall rate in dB/s, applied per sample (see `meter::mixer_integration`).
    pub ppm_tau1_ms: AtomicF32,
    pub ppm_tau2_ms: AtomicF32,
    pub fall_db_per_sec: AtomicF32,
}

/// How long a reader waits for a block to finish rendering.
const CONSISTENT_WAIT: std::time::Duration = std::time::Duration::from_millis(50);

impl BusShared {
    pub fn frames_rendered(&self) -> u64 {
        self.frames_rendered.load(Ordering::Acquire)
    }

    pub fn heartbeat(&self) -> u64 {
        self.heartbeat.load(Ordering::Acquire)
    }

    /// Runs `take` (which moves measurements out of the sources) until one
    /// pass runs with no block rendered meanwhile. A block that lands
    /// half-way through a pass is taken whole by the next one, so every
    /// block's measurements end up in the same reading. Bounded like
    /// `consistent`.
    pub fn whole_blocks(&self, mut take: impl FnMut()) {
        let started = std::time::Instant::now();
        loop {
            let before = self.render_seq.load(Ordering::Acquire);
            if before % 2 == 1 && started.elapsed() <= CONSISTENT_WAIT {
                std::thread::yield_now();
                continue;
            }
            take();
            if self.render_seq.load(Ordering::Acquire) == before
                || started.elapsed() > CONSISTENT_WAIT
            {
                return;
            }
        }
    }

    /// Runs `read` so that it observes a state between two blocks, never
    /// half-way through one (source positions and the bus clock agree).
    ///
    /// A block renders in well under `CONSISTENT_WAIT`; a render that never
    /// finishes (a device thread gone mid-block) gets a best-effort read
    /// instead of a hung conductor.
    pub fn consistent<T>(&self, read: impl Fn() -> T) -> T {
        let started = std::time::Instant::now();
        loop {
            let before = self.render_seq.load(Ordering::Acquire);
            if before % 2 == 1 {
                if started.elapsed() > CONSISTENT_WAIT {
                    return read();
                }
                std::thread::yield_now();
                continue;
            }
            let value = read();
            if self.render_seq.load(Ordering::Acquire) == before
                || started.elapsed() > CONSISTENT_WAIT
            {
                return value;
            }
        }
    }
}

impl StreamErrorSink for BusShared {
    fn report(&self, kind: StreamErrorKind) {
        match kind {
            StreamErrorKind::DeviceLost => self.lost.store(true, Ordering::Release),
            StreamErrorKind::RealtimeDenied => self.realtime_denied.store(true, Ordering::Release),
            StreamErrorKind::Xrun => {
                self.xruns.fetch_add(1, Ordering::Relaxed);
            }
            StreamErrorKind::Other => {
                self.stream_errors.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MixerConfig {
    /// Frames over which a volume change is smoothed.
    pub volume_smoothing_frames: u32,
    /// The de-click length: a failed source whose buffer holds no more than
    /// this many frames ramps to zero over what is left. 0 is no ramp.
    pub declick_frames: u32,
    /// Maximum commands applied per block.
    pub max_commands_per_block: usize,
}

/// Why a fade is pending.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FadeKind {
    Plain,
    /// A `RampOutBeforeCut`: skipped when the source ends by its stop frame.
    Cut,
}

/// Frames a source's end of stream may lie past its stop frame and still
/// count as ending there. The stop frame comes from the analysed duration,
/// which can be a frame short of the real audio, and a resampled source can
/// gain another frame to rounding. Cutting this few frames at full level is
/// inaudible.
const END_TOLERANCE_FRAMES: u64 = 2;

pub struct Slot {
    source: SourceConsumer,
    volume: Arc<AtomicF32>,
    volume_now: f32,
    first_channel: usize,
    start_at: Option<u64>,
    started: bool,
    fade: Ramp,
    pending_fade: Option<(u64, f32, u32, Curve, FadeKind)>,
    stop_at: Option<u64>,
    pause: Ramp,
    pausing: bool,
    paused: bool,
    /// The failed-source ramp to silence was started (once per source).
    fail_ramped: bool,
    finished: bool,
    /// This block: every frame rendered at gain exactly 1.0.
    block_unity: bool,
    /// This block: wrote a non-zero sample.
    block_audible: bool,
    /// Meter measurement state (meters spec M1).
    k_weighting: [KWeighting; 2],
    true_peak: [TruePeak; 2],
    /// Whether true peak was measured last block (its history restarts
    /// when it is switched on).
    true_peak_on: bool,
    /// Programme-meter integrator per channel: two stages, and the charge
    /// factors they were run with (a preset change starts them from rest).
    ppm: [[f32; 2]; 2],
    ppm_charge: [f32; 2],
}

/// The conductor's side of a mixer.
pub struct MixerHandle {
    pub commands: rtrb::Producer<BusCommand>,
    pub events: rtrb::Consumer<BusEvent>,
    pub retired: rtrb::Consumer<Retired>,
    pub shared: Arc<BusShared>,
}

pub struct Mixer {
    slots: SlotStorage,
    commands: rtrb::Consumer<BusCommand>,
    events: rtrb::Producer<BusEvent>,
    retired: rtrb::Producer<Retired>,
    /// Retired items the queue had no room for; retried every block.
    backlog: Option<Retired>,
    shared: Arc<BusShared>,
    config: MixerConfig,
    volume_step: f32,
    /// The volume smoothing as configured, and the rate it was sized for.
    smoothing_base: Option<(u32, u32)>,
    /// The de-click length at the same base rate as `smoothing_base`.
    declick_base: u32,
}

impl Mixer {
    /// Creates a mixer with `slots` slots and the queues to drive it.
    pub fn new(slots: usize, config: MixerConfig) -> (Mixer, MixerHandle) {
        let queue = 1024.max(slots * 8);
        let (cmd_tx, cmd_rx) = rtrb::RingBuffer::new(queue);
        let (ev_tx, ev_rx) = rtrb::RingBuffer::new(queue);
        let (ret_tx, ret_rx) = rtrb::RingBuffer::new(queue);
        let shared = Arc::new(BusShared::default());
        let volume_step = 1.0 / config.volume_smoothing_frames.max(1) as f32;
        let mixer = Mixer {
            slots: SlotStorage::with_capacity(slots),
            commands: cmd_rx,
            events: ev_tx,
            retired: ret_tx,
            backlog: None,
            shared: shared.clone(),
            config,
            volume_step,
            smoothing_base: None,
            declick_base: config.declick_frames,
        };
        (
            mixer,
            MixerHandle {
                commands: cmd_tx,
                events: ev_rx,
                retired: ret_rx,
                shared,
            },
        )
    }

    pub fn shared(&self) -> &Arc<BusShared> {
        &self.shared
    }

    fn emit(&mut self, event: BusEvent) {
        if self.events.push(event).is_err() {
            self.shared.dropped_events.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn retire(&mut self, item: Retired) {
        if self.backlog.is_some() {
            // Both the queue and the backlog are full (the conductor stopped
            // draining). Freeing here would break the real-time rules, so the
            // item is leaked and counted instead.
            std::mem::forget(item);
            self.shared.leaked.fetch_add(1, Ordering::Relaxed);
            return;
        }
        if let Err(rtrb::PushError::Full(item)) = self.retired.push(item) {
            self.backlog = Some(item);
        }
    }

    fn slot_mut(&mut self, slot: usize) -> Option<&mut Slot> {
        self.slots.0.get_mut(slot).and_then(Option::as_mut)
    }

    fn apply(&mut self, command: BusCommand, now: u64) {
        match command {
            BusCommand::Attach {
                slot,
                source,
                volume,
                first_channel,
            } => {
                let volume_now = volume.load();
                let rate = self.shared.sample_rate.load(Ordering::Acquire).max(1);
                if let Some(cell) = self.slots.0.get_mut(slot)
                    && cell.is_none()
                {
                    *cell = Some(Slot {
                        source,
                        volume,
                        volume_now,
                        first_channel: usize::from(first_channel),
                        start_at: None,
                        started: false,
                        fade: Ramp::hold(1.0),
                        pending_fade: None,
                        stop_at: None,
                        pause: Ramp::hold(1.0),
                        pausing: false,
                        paused: false,
                        fail_ramped: false,
                        finished: false,
                        block_unity: false,
                        block_audible: false,
                        k_weighting: [KWeighting::new(rate); 2],
                        true_peak: [TruePeak::default(); 2],
                        true_peak_on: false,
                        ppm: [[0.0; 2]; 2],
                        ppm_charge: [0.0; 2],
                    });
                } else {
                    // Occupied or out of range: hand the source straight back.
                    self.retire(Retired::Source {
                        slot: None,
                        source,
                        volume,
                    });
                }
            }
            BusCommand::Start { slot, at_frame } => {
                if let Some(s) = self.slot_mut(slot)
                    && !s.started
                {
                    s.start_at = Some(at_frame.max(now));
                }
            }
            BusCommand::Ramp {
                slot,
                to,
                frames,
                curve,
                at_frame,
            } => {
                if let Some(s) = self.slot_mut(slot) {
                    if at_frame <= now {
                        s.fade.retarget(to, frames, curve);
                        s.pending_fade = None;
                    } else {
                        s.pending_fade = Some((at_frame, to, frames, curve, FadeKind::Plain));
                    }
                }
            }
            BusCommand::RampOutBeforeCut {
                slot,
                frames,
                at_frame,
            } => {
                if let Some(s) = self.slot_mut(slot) {
                    if at_frame <= now {
                        if !ends_by_stop(s, now) {
                            s.fade.retarget(0.0, frames, Curve::Linear);
                        }
                        s.pending_fade = None;
                    } else {
                        s.pending_fade =
                            Some((at_frame, 0.0, frames, Curve::Linear, FadeKind::Cut));
                    }
                }
            }
            BusCommand::StopAt { slot, at_frame } => {
                if let Some(s) = self.slot_mut(slot) {
                    s.stop_at = Some(at_frame.max(now));
                }
            }
            BusCommand::Pause { slot, ramp_frames } => {
                if let Some(s) = self.slot_mut(slot)
                    && !s.paused
                {
                    s.pause.retarget(0.0, ramp_frames, Curve::Linear);
                    s.pausing = true;
                }
            }
            BusCommand::Resume { slot, ramp_frames } => {
                if let Some(s) = self.slot_mut(slot) {
                    s.paused = false;
                    s.pausing = false;
                    s.pause.retarget(1.0, ramp_frames, Curve::Linear);
                }
            }
            BusCommand::Cancel { slot } => {
                if let Some(s) = self.slot_mut(slot) {
                    if !s.started {
                        s.start_at = None;
                    }
                    s.pending_fade = None;
                    s.stop_at = None;
                }
            }
            BusCommand::Detach { slot } => {
                if let Some(cell) = self.slots.0.get_mut(slot)
                    && let Some(s) = cell.take()
                {
                    self.retire(Retired::Source {
                        slot: Some(slot),
                        source: s.source,
                        volume: s.volume,
                    });
                }
            }
            BusCommand::Grow(mut storage) => {
                if storage.len() >= self.slots.len() {
                    for (new, old) in storage.0.iter_mut().zip(self.slots.0.iter_mut()) {
                        *new = old.take();
                    }
                    std::mem::swap(&mut self.slots, &mut storage);
                }
                self.retire(Retired::Storage(storage));
            }
        }
    }

    /// The stream moved from `from_rate` to `to_rate` (a bit-perfect
    /// reopen): durations held in frames keep their length in time. Called
    /// while no stream renders.
    pub fn follow_rate(&mut self, from_rate: u32, to_rate: u32) {
        // Always from the first known size and rate, so chains of changes
        // between unrelated rates do not drift by rounding.
        let (base_frames, base_rate) = *self
            .smoothing_base
            .get_or_insert((self.config.volume_smoothing_frames, from_rate.max(1)));
        let frames = u64::from(base_frames) * u64::from(to_rate) / u64::from(base_rate);
        self.config.volume_smoothing_frames = u32::try_from(frames.max(1)).unwrap_or(u32::MAX);
        self.volume_step = 1.0 / self.config.volume_smoothing_frames as f32;
        let declick = u64::from(self.declick_base) * u64::from(to_rate) / u64::from(base_rate);
        self.config.declick_frames = u32::try_from(declick).unwrap_or(u32::MAX);
    }

    /// Mixes one block into `out` (interleaved, `channels` per frame).
    pub fn render(&mut self, out: &mut [f32], channels: usize) {
        self.shared.render_seq.fetch_add(1, Ordering::AcqRel);
        out.fill(0.0);
        let channels = channels.max(1);
        let frames = out.len() / channels;
        let now = self.shared.frames_rendered.load(Ordering::Relaxed);

        if let Some(item) = self.backlog.take()
            && let Err(rtrb::PushError::Full(item)) = self.retired.push(item)
        {
            self.backlog = Some(item);
        }
        for _ in 0..self.config.max_commands_per_block {
            match self.commands.pop() {
                Ok(command) => self.apply(command, now),
                Err(_) => break,
            }
        }

        let mut peak_l = 0.0f32;
        let mut peak_r = 0.0f32;
        let volume_step = self.volume_step;
        let meter = MeterMode::read(&self.shared);
        for index in 0..self.slots.len() {
            let Some(Some(slot)) = self.slots.0.get_mut(index) else {
                continue;
            };
            if slot.started && !slot.finished && slot.first_channel + 1 >= channels {
                self.shared.misrouted.fetch_add(1, Ordering::Relaxed);
            }
            let mut events: [Option<BusEvent>; 2] = [None, None];
            let (l, r) = render_slot(
                slot,
                index,
                out,
                channels,
                frames,
                now,
                volume_step,
                &mut events,
                meter,
                self.config.declick_frames,
            );
            peak_l = peak_l.max(l);
            peak_r = peak_r.max(r);
            for event in events.into_iter().flatten() {
                self.emit(event);
            }
        }
        mark_unaltered(&self.slots);
        self.shared.peak_l.fetch_max(peak_l);
        self.shared.peak_r.fetch_max(peak_r);
        self.shared
            .frames_rendered
            .store(now + frames as u64, Ordering::Release);
        self.shared.heartbeat.fetch_add(1, Ordering::Release);
        self.shared.render_seq.fetch_add(1, Ordering::AcqRel);
    }
}

/// How the meters measure this block, read once from `BusShared`.
#[derive(Debug, Clone, Copy)]
struct MeterMode {
    true_peak: bool,
    /// A programme meter: the two-stage integrator runs.
    programme: bool,
    /// Per-sample charge factors of the two stages, and the fall factor.
    charge: [f32; 2],
    decay: f32,
}

impl MeterMode {
    fn read(shared: &BusShared) -> Self {
        let rate = shared.sample_rate.load(Ordering::Relaxed).max(1) as f32;
        let factor = |tau_ms: f32| {
            if tau_ms > 0.0 {
                1.0 - (-1_000.0 / (tau_ms * rate)).exp()
            } else {
                1.0
            }
        };
        let (tau1, tau2) = (shared.ppm_tau1_ms.load(), shared.ppm_tau2_ms.load());
        let fall = shared.fall_db_per_sec.load().max(0.0);
        Self {
            true_peak: shared.true_peak.load(Ordering::Relaxed),
            programme: tau1 > 0.0 || tau2 > 0.0,
            charge: [factor(tau1), factor(tau2)],
            decay: 10f32.powf(-fall / (20.0 * rate)),
        }
    }

    /// A rectifier charging a first stage, which charges a second one; each
    /// only charges upwards and falls at the meter's fall rate.
    fn integrate(&self, stages: &mut [f32; 2], level: f32) -> f32 {
        let [a, b] = stages;
        let [k1, k2] = self.charge;
        *a = if level > *a {
            *a + (level - *a) * k1
        } else {
            *a * self.decay
        };
        *b = if *a > *b {
            *b + (*a - *b) * k2
        } else {
            *b * self.decay
        };
        // During silence the fall would end in subnormal numbers, slow on
        // many CPUs: flush them (far below any displayed level).
        for stage in [&mut *a, &mut *b] {
            if *stage < 1e-20 {
                *stage = 0.0;
            }
        }
        *b
    }
}

/// Marks each slot's source unaltered when it played at unity and no other
/// slot wrote into an overlapping channel pair. Compares slots pairwise: no
/// allocation, and the slot count is small.
fn mark_unaltered(slots: &SlotStorage) {
    for (i, slot) in slots.0.iter().enumerate() {
        let Some(slot) = slot else { continue };
        let alone = !slots.0.iter().enumerate().any(|(j, other)| {
            j != i
                && other.as_ref().is_some_and(|o| {
                    o.block_audible && o.first_channel.abs_diff(slot.first_channel) < 2
                })
        });
        slot.source
            .shared
            .unaltered
            .store(slot.block_unity && alone, Ordering::Release);
    }
}

/// True when the slot's source ends (end of stream, everything pushed is
/// already in the ring) at or before its stop frame, seen from frame `abs`,
/// give or take `END_TOLERANCE_FRAMES`.
/// Real-time safe: two atomic loads and a ring-occupancy read.
fn ends_by_stop(slot: &Slot, abs: u64) -> bool {
    let Some(stop) = slot.stop_at else {
        return false;
    };
    // Read eof before the ring: the producer pushes, then sets eof.
    slot.source.shared.is_eof()
        && slot.source.buffered_frames() as u64 <= stop.saturating_sub(abs) + END_TOLERANCE_FRAMES
}

/// Renders one slot into `out`. Returns the (left, right) peaks it produced.
#[allow(clippy::too_many_arguments)]
fn render_slot(
    slot: &mut Slot,
    index: usize,
    out: &mut [f32],
    channels: usize,
    frames: usize,
    block_start: u64,
    volume_step: f32,
    events: &mut [Option<BusEvent>; 2],
    meter: MeterMode,
    declick_frames: u32,
) -> (f32, f32) {
    if slot.ppm_charge != meter.charge {
        slot.ppm_charge = meter.charge;
        slot.ppm = [[0.0; 2]; 2];
    }
    if meter.true_peak && !slot.true_peak_on {
        slot.true_peak = [TruePeak::default(); 2];
    }
    slot.true_peak_on = meter.true_peak;
    slot.block_unity = false;
    slot.block_audible = false;
    if slot.finished {
        return (0.0, 0.0);
    }
    let block_end = block_start + frames as u64;
    let mut f = 0usize;
    if !slot.started {
        match slot.start_at {
            Some(at) if at < block_end => {
                slot.started = true;
                f = at.saturating_sub(block_start) as usize;
                if let Some(e) = events.get_mut(0) {
                    *e = Some(BusEvent::Started {
                        slot: index,
                        frame: block_start + f as u64,
                    });
                }
            }
            _ => return (0.0, 0.0),
        }
    }
    // A stop on a paused slot (the player was stopped while paused, and the
    // pause ramp ran out first): nothing is audible, end it at the stop frame.
    if slot.paused
        && let Some(stop) = slot.stop_at
        && stop < block_end
    {
        finish(slot, index, stop.max(block_start), events);
        return (0.0, 0.0);
    }
    let target_volume = slot.volume.load().clamp(0.0, 1.0);
    // A pair that does not fit the stream is consumed silently (never written
    // into another channel or frame); the caller counts it.
    let fits = slot.first_channel + 1 < channels;
    let mut chunk = [0.0f32; CHUNK_FRAMES * SOURCE_CHANNELS];
    let (mut peak_l, mut peak_r) = (0.0f32, 0.0f32);
    let mut unity = true;
    let mut rendered = false;
    // Sample peaks, for the unaltered-source check (not the meter's mode).
    let (mut audible_l, mut audible_r) = (0.0f32, 0.0f32);
    // Meter measurement, added to the source's accumulators once per block.
    let (mut sum_l, mut sum_r, mut k_l, mut k_r) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    let (mut abs_l, mut abs_r) = (0.0f64, 0.0f64);
    let mut measured = 0u64;
    let declick = u64::from(declick_frames);
    while f < frames && !slot.paused {
        let abs = block_start + f as u64;
        if slot.stop_at.is_some_and(|stop| abs >= stop) {
            finish(slot, index, abs, events);
            break;
        }
        if let Some((at, to, len, curve, kind)) = slot.pending_fade
            && abs >= at
        {
            if !(kind == FadeKind::Cut && ends_by_stop(slot, abs)) {
                slot.fade.retarget(to, len, curve);
            }
            slot.pending_fade = None;
        }
        // Process up to the next scheduled boundary so every change lands on its exact frame.
        let mut n = (frames - f).min(CHUNK_FRAMES);
        if let Some(stop) = slot.stop_at {
            n = n.min((stop - abs) as usize);
        }
        if let Some((at, ..)) = slot.pending_fade {
            n = n.min((at - abs) as usize);
        }
        if slot.pausing {
            n = n.min((slot.pause.remaining() as usize).max(1));
        }
        // A failed source stops pushing: when what is buffered fits the
        // de-click length, ramp it to zero over exactly those frames, so its
        // end is not a step. `n` stops where the ramp must begin.
        if declick > 0 && !slot.fail_ramped && slot.source.shared.is_failed() {
            let left = slot.source.buffered_frames() as u64;
            if left <= declick {
                slot.fade.retarget(
                    0.0,
                    u32::try_from(left.max(1)).unwrap_or(u32::MAX),
                    Curve::Linear,
                );
                slot.fail_ramped = true;
            } else {
                n = n.min(usize::try_from(left - declick).unwrap_or(usize::MAX));
            }
        }
        let Some(buf) = chunk.get_mut(..n * SOURCE_CHANNELS) else {
            break;
        };
        // Read "no more audio will come" *before* popping: a producer that
        // pushes its last samples and then sets eof cannot lose them.
        let ended = slot.source.shared.is_eof() || slot.source.shared.is_failed();
        let got = slot.source.pop_frames(buf);
        for (k, &[l, r]) in buf
            .as_chunks::<SOURCE_CHANNELS>()
            .0
            .iter()
            .take(got)
            .enumerate()
        {
            if slot.volume_now < target_volume {
                slot.volume_now = (slot.volume_now + volume_step).min(target_volume);
            } else if slot.volume_now > target_volume {
                slot.volume_now = (slot.volume_now - volume_step).max(target_volume);
            }
            let g = slot.fade.next_gain() * slot.pause.next_gain() * slot.volume_now;
            unity &= g == 1.0;
            rendered = true;
            let (l, r) = (l * g, r * g);
            audible_l = audible_l.max(l.abs());
            audible_r = audible_r.max(r.abs());
            let [tp_l, tp_r] = &mut slot.true_peak;
            let (mut level_l, mut level_r) = if meter.true_peak {
                (tp_l.push(l), tp_r.push(r))
            } else {
                (l.abs(), r.abs())
            };
            if meter.programme {
                let [env_l, env_r] = &mut slot.ppm;
                level_l = meter.integrate(env_l, level_l);
                level_r = meter.integrate(env_r, level_r);
            }
            peak_l = peak_l.max(level_l);
            peak_r = peak_r.max(level_r);
            let [kw_l, kw_r] = &mut slot.k_weighting;
            let (wl, wr) = (kw_l.process(f64::from(l)), kw_r.process(f64::from(r)));
            sum_l += f64::from(l) * f64::from(l);
            sum_r += f64::from(r) * f64::from(r);
            abs_l += f64::from(l.abs());
            abs_r += f64::from(r.abs());
            k_l += wl * wl;
            k_r += wr * wr;
            measured += 1;
            if fits {
                let base = (f + k) * channels + slot.first_channel;
                if let Some(o) = out.get_mut(base) {
                    *o += l;
                }
                if let Some(o) = out.get_mut(base + 1) {
                    *o += r;
                }
            }
        }
        slot.source
            .shared
            .frames_played
            .fetch_add(got as u64, Ordering::AcqRel);
        if slot.pausing && slot.pause.is_done() {
            slot.pausing = false;
            slot.paused = true;
        }
        if got < n {
            // End of stream, or a failed source whose buffer has drained.
            if ended {
                finish(slot, index, abs + got as u64, events);
                break;
            }
            slot.source.shared.underruns.fetch_add(1, Ordering::Relaxed);
            // The missing frames stay silent; the timeline keeps moving.
            f += n;
        } else {
            f += got;
        }
    }
    slot.source.shared.peak_l.fetch_max(peak_l);
    slot.source.shared.peak_r.fetch_max(peak_r);
    if measured > 0 {
        let shared = &slot.source.shared;
        shared.sum_sq_l.fetch_add(sum_l);
        shared.sum_sq_r.fetch_add(sum_r);
        shared.sum_abs_l.fetch_add(abs_l);
        shared.sum_abs_r.fetch_add(abs_r);
        shared.k_sum_l.fetch_add(k_l);
        shared.k_sum_r.fetch_add(k_r);
        shared.measured_frames.fetch_add(measured, Ordering::AcqRel);
    }
    slot.block_unity = unity && rendered;
    slot.block_audible = audible_l > 0.0 || audible_r > 0.0;
    (peak_l, peak_r)
}

fn finish(slot: &mut Slot, index: usize, frame: u64, events: &mut [Option<BusEvent>; 2]) {
    slot.finished = true;
    if let Some(e) = events.get_mut(1) {
        *e = Some(BusEvent::Finished { slot: index, frame });
    }
}

/// Adapts a shared mixer to a backend `Renderer`. The real-time thread only
/// ever `try_lock`s; on contention (a hand-over in progress) it outputs
/// silence for that block and counts the miss.
pub struct MixerRenderer {
    pub mixer: Arc<std::sync::Mutex<Mixer>>,
    pub shared: Arc<BusShared>,
}

impl Renderer for MixerRenderer {
    fn render(&mut self, out: &mut [f32], channels: usize) {
        match self.mixer.try_lock() {
            Ok(mut mixer) => mixer.render(out, channels),
            Err(std::sync::TryLockError::Poisoned(poisoned)) => {
                poisoned.into_inner().render(out, channels)
            }
            Err(std::sync::TryLockError::WouldBlock) => {
                out.fill(0.0);
                self.shared.lock_misses.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BusShared, MeterMode};
    use std::sync::atomic::Ordering;

    #[test]
    fn a_programme_meter_falls_to_exact_zero_in_silence() {
        let mode = MeterMode {
            true_peak: false,
            programme: true,
            charge: [0.01, 0.02],
            // 100 dB/s at 48 kHz.
            decay: 10f32.powf(-100.0 / (20.0 * 48_000.0)),
        };
        let mut stages = [0.0f32; 2];
        for _ in 0..4_800 {
            mode.integrate(&mut stages, 1.0);
        }
        for _ in 0..48_000 * 20 {
            mode.integrate(&mut stages, 0.0);
        }
        assert_eq!(stages, [0.0, 0.0], "no subnormal tail");
    }

    #[test]
    fn a_render_that_never_finishes_does_not_hang_the_reader() {
        let shared = BusShared::default();
        shared.render_seq.store(1, Ordering::Release); // a render left open
        assert_eq!(shared.consistent(|| 7), 7);
    }

    #[test]
    fn a_block_rendered_during_a_take_is_taken_whole() {
        let shared = BusShared::default();
        let mut passes = 0;
        shared.whole_blocks(|| {
            passes += 1;
            if passes == 1 {
                // A block renders between two of the take's reads.
                shared.render_seq.fetch_add(2, Ordering::AcqRel);
            }
        });
        assert_eq!(
            passes, 2,
            "the rest of that block is taken in the same tick"
        );
        let mut passes = 0;
        shared.whole_blocks(|| passes += 1);
        assert_eq!(passes, 1);
    }

    #[test]
    fn volume_smoothing_keeps_its_duration_when_the_rate_changes() {
        let config = super::MixerConfig {
            volume_smoothing_frames: 480, // 10 ms at 48 kHz
            declick_frames: 0,
            max_commands_per_block: 8,
        };
        let (mut mixer, _handle) = super::Mixer::new(1, config);
        mixer.follow_rate(48_000, 96_000);
        assert!(
            (mixer.volume_step - 1.0 / 960.0).abs() < 1e-9,
            "{}",
            mixer.volume_step
        );
        // Back and forth between unrelated rates: no drift.
        let config = super::MixerConfig {
            volume_smoothing_frames: 500,
            declick_frames: 0,
            max_commands_per_block: 8,
        };
        let (mut mixer, _handle) = super::Mixer::new(1, config);
        for _ in 0..3 {
            mixer.follow_rate(48_000, 44_100);
            mixer.follow_rate(44_100, 48_000);
        }
        assert_eq!(mixer.config.volume_smoothing_frames, 500);
    }

    #[test]
    fn declick_keeps_its_duration_when_the_rate_changes() {
        let config = super::MixerConfig {
            volume_smoothing_frames: 480,
            declick_frames: 240, // 5 ms at 48 kHz
            max_commands_per_block: 8,
        };
        let (mut mixer, _handle) = super::Mixer::new(1, config);
        mixer.follow_rate(48_000, 96_000);
        assert_eq!(mixer.config.declick_frames, 480);
        for _ in 0..3 {
            mixer.follow_rate(96_000, 44_100);
            mixer.follow_rate(44_100, 48_000);
        }
        assert_eq!(mixer.config.declick_frames, 240);
        mixer.follow_rate(48_000, 44_100);
        assert_eq!(mixer.config.declick_frames, 220);
    }
}
