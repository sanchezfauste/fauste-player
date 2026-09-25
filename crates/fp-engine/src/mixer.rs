//! The bus mixer (spec §4.3). It runs on the device's real-time thread: it
//! sums the sources attached to its slots into the output, applying gain ramps
//! at exact frames. It never allocates, frees, blocks, logs or panics: memory
//! only arrives and leaves through lock-free queues.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use fp_backends::{Renderer, StreamErrorKind, StreamErrorSink};

use crate::atomic::AtomicF32;
use crate::ramp::{Curve, Ramp};
use crate::source::{SOURCE_CHANNELS, SourceConsumer};

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
        slot: usize,
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
    pub dropped_events: AtomicU64,
    pub lock_misses: AtomicU64,
    pub peak_l: AtomicF32,
    pub peak_r: AtomicF32,
}

impl BusShared {
    pub fn frames_rendered(&self) -> u64 {
        self.frames_rendered.load(Ordering::Acquire)
    }

    pub fn heartbeat(&self) -> u64 {
        self.heartbeat.load(Ordering::Acquire)
    }

    /// Runs `read` so that it observes a state between two blocks, never
    /// half-way through one (source positions and the bus clock agree).
    pub fn consistent<T>(&self, read: impl Fn() -> T) -> T {
        loop {
            let before = self.render_seq.load(Ordering::Acquire);
            if before % 2 == 1 {
                std::hint::spin_loop();
                continue;
            }
            let value = read();
            if self.render_seq.load(Ordering::Acquire) == before {
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
            StreamErrorKind::Xrun | StreamErrorKind::Other => {
                self.xruns.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MixerConfig {
    /// Frames over which a volume change is smoothed.
    pub volume_smoothing_frames: u32,
    /// Maximum commands applied per block.
    pub max_commands_per_block: usize,
}

pub struct Slot {
    source: SourceConsumer,
    volume: Arc<AtomicF32>,
    volume_now: f32,
    first_channel: usize,
    start_at: Option<u64>,
    started: bool,
    fade: Ramp,
    pending_fade: Option<(u64, f32, u32, Curve)>,
    stop_at: Option<u64>,
    pause: Ramp,
    pausing: bool,
    paused: bool,
    finished: bool,
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
            // Keep the oldest; this one waits in its slot (see `detach`).
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
                        finished: false,
                    });
                } else {
                    // Occupied or out of range: hand the source straight back.
                    self.retire(Retired::Source {
                        slot,
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
                        s.pending_fade = Some((at_frame, to, frames, curve));
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
                        slot,
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
        for index in 0..self.slots.len() {
            let Some(Some(slot)) = self.slots.0.get_mut(index) else {
                continue;
            };
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
            );
            peak_l = peak_l.max(l);
            peak_r = peak_r.max(r);
            for event in events.into_iter().flatten() {
                self.emit(event);
            }
        }
        self.shared.peak_l.fetch_max(peak_l);
        self.shared.peak_r.fetch_max(peak_r);
        self.shared
            .frames_rendered
            .store(now + frames as u64, Ordering::Release);
        self.shared.heartbeat.fetch_add(1, Ordering::Release);
        self.shared.render_seq.fetch_add(1, Ordering::AcqRel);
    }
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
) -> (f32, f32) {
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
    let target_volume = slot.volume.load().clamp(0.0, 1.0);
    let mut chunk = [0.0f32; CHUNK_FRAMES * SOURCE_CHANNELS];
    let (mut peak_l, mut peak_r) = (0.0f32, 0.0f32);
    while f < frames && !slot.paused {
        let abs = block_start + f as u64;
        if slot.stop_at.is_some_and(|stop| abs >= stop) {
            finish(slot, index, abs, events);
            break;
        }
        if let Some((at, to, len, curve)) = slot.pending_fade
            && abs >= at
        {
            slot.fade.retarget(to, len, curve);
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
        let Some(buf) = chunk.get_mut(..n * SOURCE_CHANNELS) else {
            break;
        };
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
            let (l, r) = (l * g, r * g);
            peak_l = peak_l.max(l.abs());
            peak_r = peak_r.max(r.abs());
            let base = (f + k) * channels + slot.first_channel;
            if let Some(o) = out.get_mut(base) {
                *o += l;
            }
            if slot.first_channel + 1 < channels
                && let Some(o) = out.get_mut(base + 1)
            {
                *o += r;
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
            if slot.source.shared.is_eof() {
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
