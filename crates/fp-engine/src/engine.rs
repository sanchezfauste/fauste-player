//! Executes model `EngineAction`s on buses and worker threads, and turns what
//! the buses and workers observe into model `EngineEvent`s (spec §4.4). All
//! timing is in bus frames: transitions are dispatched ahead of time
//! (`schedule_lead`) with an exact frame, so they never depend on when this
//! thread happens to wake.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, Sender};
use fp_backends::{AudioBackend, NullBackend, StreamConfig};
use fp_model::{
    Config, EngineAction, EngineEvent, EntryId, PlayerId, PlayerRoutes, SourceRequest,
    TransitionPlan, Tuning,
};

use crate::atomic::AtomicF32;
use crate::bus::{Bus, BusHealth, BusKey, BusTiming};
use crate::mixer::{BusCommand, BusEvent, MixerConfig};
use crate::ramp::Curve;
use crate::source::{SourceShared, source_pair};
use crate::worker::{PlayerWorker, SourceKey, SourceOpener, WorkerFailure};

/// Stream settings and tuning, derived from the model `Config`.
#[derive(Debug, Clone)]
pub struct EngineSettings {
    pub sample_rate: u32,
    pub buffer_frames: u32,
    pub channels: u16,
    pub tuning: Tuning,
    pub routes: Vec<PlayerRoutes>,
    /// Backend used when a player has no explicit route (`None`: the first registered).
    pub default_backend: Option<String>,
}

impl EngineSettings {
    pub fn from_config(config: &Config) -> Self {
        Self {
            sample_rate: config.outputs.sample_rate,
            buffer_frames: config.outputs.buffer_frames,
            channels: 2,
            tuning: config.tuning.clone(),
            routes: config.outputs.routes.clone(),
            default_backend: config.outputs.backend.clone(),
        }
    }

    fn frames(&self, ms: f64) -> u64 {
        (ms.max(0.0) * f64::from(self.sample_rate) / 1000.0).round() as u64
    }
}

/// What the UI shows for one player, refreshed every tick.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PlayerTelemetry {
    /// Position of the current source in track seconds.
    pub position_secs: Option<f64>,
    pub cue_position_secs: Option<f64>,
    pub peak_l: f32,
    pub peak_r: f32,
    pub underruns: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BusStatus {
    pub key: BusKey,
    pub health: BusHealth,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StartState {
    /// Attached, waiting for a command (preload, or loaded paused).
    Idle,
    /// Start as soon as enough audio is buffered.
    WhenReady {
        fade_in: bool,
    },
    /// A `Start` has been sent for a future frame.
    Requested,
    Started,
}

struct Playing {
    key: SourceKey,
    bus: BusKey,
    slot: usize,
    request: SourceRequest,
    entry: EntryId,
    start_secs: f64,
    shared: Arc<SourceShared>,
    start: StartState,
    /// Report `ReachedEnd` when this source finishes (fade stop).
    report_end: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Plan {
    None,
    Waiting(TransitionPlan),
    Dispatched(TransitionPlan),
}

struct PlayerRuntime {
    worker: PlayerWorker,
    volume: Arc<AtomicF32>,
    cue_volume: Arc<AtomicF32>,
    main: (BusKey, u16),
    cue: Option<(BusKey, u16)>,
    preload: Option<Playing>,
    current: Option<Playing>,
    /// Sources fading out or stopping (crossfades, overlaps, seeks, stops).
    outgoing: Vec<Playing>,
    cue_src: Option<Playing>,
    plan: Plan,
    paused: bool,
    /// Emit `FadeCompleted` once `outgoing` is empty.
    notify_fade: bool,
}

pub struct Engine {
    backends: Vec<Arc<dyn AudioBackend>>,
    settings: EngineSettings,
    opener: SourceOpener,
    buses: BTreeMap<BusKey, Bus>,
    players: HashMap<PlayerId, PlayerRuntime>,
    failures_tx: Sender<WorkerFailure>,
    failures_rx: Receiver<WorkerFailure>,
    owners: HashMap<SourceKey, PlayerId>,
    next_key: u64,
    events: Vec<EngineEvent>,
    dropped_commands: u64,
}

/// Which source of a player a bus slot belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Role {
    Preload,
    Current,
    Outgoing(usize),
    Cue,
}

impl Engine {
    /// `backends` are tried by id for routes; a `Null` backend is always
    /// available as the last resort so a player can never stall.
    pub fn new(
        mut backends: Vec<Arc<dyn AudioBackend>>,
        settings: EngineSettings,
        opener: SourceOpener,
    ) -> Self {
        backends.push(Arc::new(NullBackend));
        let (failures_tx, failures_rx) = crossbeam_channel::unbounded();
        Self {
            backends,
            settings,
            opener,
            buses: BTreeMap::new(),
            players: HashMap::new(),
            failures_tx,
            failures_rx,
            owners: HashMap::new(),
            next_key: 0,
            events: Vec::new(),
            dropped_commands: 0,
        }
    }

    /// Bus commands that could not be queued (should always be zero).
    pub fn dropped_commands(&self) -> u64 {
        self.dropped_commands
    }

    pub fn settings(&self) -> &EngineSettings {
        &self.settings
    }

    pub fn bus_status(&self) -> Vec<BusStatus> {
        self.buses
            .values()
            .map(|b| BusStatus {
                key: b.key().clone(),
                health: b.health(),
                error: b.last_error().map(str::to_owned),
            })
            .collect()
    }

    pub fn telemetry(&self, player: PlayerId) -> PlayerTelemetry {
        let Some(rt) = self.players.get(&player) else {
            return PlayerTelemetry::default();
        };
        let rate = f64::from(self.settings.sample_rate);
        let position = |p: &Playing| p.start_secs + p.shared.frames_played() as f64 / rate;
        let (peak_l, peak_r) = rt
            .current
            .iter()
            .chain(rt.outgoing.iter())
            .fold((0.0f32, 0.0f32), |(l, r), p| {
                (l.max(p.shared.peak_l.take()), r.max(p.shared.peak_r.take()))
            });
        PlayerTelemetry {
            position_secs: rt.current.as_ref().map(position),
            cue_position_secs: rt.cue_src.as_ref().map(position),
            peak_l,
            peak_r,
            underruns: rt.current.as_ref().map_or(0, |p| {
                p.shared
                    .underruns
                    .load(std::sync::atomic::Ordering::Relaxed)
            }),
        }
    }

    fn backend(&self, id: Option<&str>) -> Arc<dyn AudioBackend> {
        let wanted = id.or(self.settings.default_backend.as_deref());
        wanted
            .and_then(|w| self.backends.iter().find(|b| b.id().0 == w))
            .or_else(|| self.backends.first())
            .cloned()
            .unwrap_or_else(|| Arc::new(NullBackend))
    }

    /// Resolves the (bus, first channel) of a player's Main and Cue outputs.
    fn resolve_routes(&self, player: PlayerId) -> ((BusKey, u16), Option<(BusKey, u16)>) {
        let routes = self.settings.routes.iter().find(|r| r.player == player);
        let key_for = |backend: &Arc<dyn AudioBackend>, device: String| BusKey {
            backend: backend.id().0,
            device,
        };
        let main = match routes.and_then(|r| r.main.as_ref()) {
            Some(route) => (
                key_for(&self.backend(Some(&route.backend)), route.device.clone()),
                route.first_channel,
            ),
            None => {
                let backend = self.backend(None);
                let device = backend
                    .default_device()
                    .map_or_else(|| "null".to_owned(), |d| d.0);
                let backend = if device == "null" {
                    Arc::new(NullBackend) as Arc<dyn AudioBackend>
                } else {
                    backend
                };
                (key_for(&backend, device), 0)
            }
        };
        let cue = routes.and_then(|r| r.cue.as_ref()).map(|route| {
            (
                key_for(&self.backend(Some(&route.backend)), route.device.clone()),
                route.first_channel,
            )
        });
        (main, cue)
    }

    fn ensure_bus(&mut self, key: &BusKey, now: Instant) {
        if !self.buses.contains_key(key) {
            let backend = self.backend(Some(&key.backend));
            let t = &self.settings.tuning;
            let config = StreamConfig {
                sample_rate: self.settings.sample_rate,
                buffer_frames: self.settings.buffer_frames,
                channels: self.settings.channels,
            };
            let mixer = MixerConfig {
                volume_smoothing_frames: self.settings.frames(t.gain_smoothing_ms).max(1) as u32,
                max_commands_per_block: t.max_commands_per_block,
            };
            let timing = BusTiming {
                watchdog_timeout: Duration::from_secs_f64(t.watchdog_timeout_ms / 1000.0),
                reconnect_interval: Duration::from_secs_f64(t.reconnect_interval_ms / 1000.0),
            };
            let bus = Bus::open(key.clone(), backend, config, 8, mixer, timing, now);
            self.buses.insert(key.clone(), bus);
        }
        // Capacity derived from routing (spec §4.3): per player on Main, a
        // current, a preload and up to three outgoing; per Cue, two.
        let mains = self.players.values().filter(|p| &p.main.0 == key).count();
        let cues = self
            .players
            .values()
            .filter(|p| p.cue.as_ref().is_some_and(|c| &c.0 == key))
            .count();
        let wanted = ((mains * 5 + cues * 2 + 2) as f64 * self.settings.tuning.mixer_headroom)
            .ceil() as usize;
        if let Some(bus) = self.buses.get_mut(key) {
            bus.ensure_capacity(wanted.max(8));
        }
    }

    /// Executes one action at time `now`.
    pub fn execute(&mut self, action: EngineAction, now: Instant) {
        match action {
            EngineAction::AddPlayer { player } => self.add_player(player, now),
            EngineAction::RemovePlayer { player } => self.remove_player(player),
            EngineAction::Preload { player, request } => self.preload(player, request),
            EngineAction::StartCurrent { player, request } => {
                self.start_current(player, &request, None)
            }
            EngineAction::Crossfade {
                player,
                request,
                fade_ms,
            } => {
                self.start_current(player, &request, Some(fade_ms));
            }
            EngineAction::FadeOutAndStop { player, fade_ms } => {
                self.fade_out_and_stop(player, fade_ms)
            }
            EngineAction::Pause { player } => self.pause(player),
            EngineAction::Resume { player } => self.resume(player),
            EngineAction::StopNow { player } => self.stop_now(player),
            EngineAction::Schedule { player, plan } => {
                self.undispatch(player);
                if let Some(rt) = self.players.get_mut(&player) {
                    rt.plan = plan.map_or(Plan::None, Plan::Waiting);
                }
            }
            EngineAction::Seek { player, secs } => self.seek(player, secs),
            EngineAction::SetVolume { player, volume } => {
                if let Some(rt) = self.players.get(&player) {
                    rt.volume.store(volume);
                }
            }
            EngineAction::StartCue { player, request } => self.start_cue(player, &request),
            EngineAction::StopCue { player } => {
                if let Some(cue) = self
                    .players
                    .get_mut(&player)
                    .and_then(|rt| rt.cue_src.take())
                {
                    self.stop_quick_and_release(cue);
                }
            }
            EngineAction::LoadPaused { player, request } => {
                if let Some(p) = self.new_source(player, false, &request) {
                    let old = self.players.get_mut(&player).and_then(|rt| {
                        rt.paused = true;
                        rt.current.replace(p)
                    });
                    if let Some(old) = old {
                        self.release(old);
                    }
                }
            }
        }
    }

    fn add_player(&mut self, player: PlayerId, now: Instant) {
        if self.players.contains_key(&player) {
            return;
        }
        let (main, cue) = self.resolve_routes(player);
        let t = &self.settings.tuning;
        let ready = self.settings.frames(t.ready_threshold_ms) as usize;
        let worker = match PlayerWorker::spawn(
            &format!("fp-player-{}", player.0),
            self.opener.clone(),
            self.settings.sample_rate,
            ready,
            self.failures_tx.clone(),
        ) {
            Ok(w) => w,
            Err(e) => {
                tracing::error!(?player, "cannot spawn the player worker: {e}");
                return;
            }
        };
        self.players.insert(
            player,
            PlayerRuntime {
                worker,
                volume: Arc::new(AtomicF32::new(1.0)),
                cue_volume: Arc::new(AtomicF32::new(1.0)),
                main: main.clone(),
                cue: cue.clone(),
                preload: None,
                current: None,
                outgoing: Vec::new(),
                cue_src: None,
                plan: Plan::None,
                paused: false,
                notify_fade: false,
            },
        );
        self.ensure_bus(&main.0, now);
        if let Some((key, _)) = cue {
            self.ensure_bus(&key, now);
        }
    }

    fn remove_player(&mut self, player: PlayerId) {
        if let Some(mut rt) = self.players.remove(&player) {
            let all: Vec<Playing> = rt
                .preload
                .take()
                .into_iter()
                .chain(rt.current.take())
                .chain(rt.cue_src.take())
                .chain(rt.outgoing.drain(..))
                .collect();
            for p in all {
                self.detach(&p);
                self.owners.remove(&p.key);
            }
        }
    }

    /// Attaches a new source to the player's Main (or Cue) bus and asks its
    /// worker to fill it.
    fn new_source(
        &mut self,
        player: PlayerId,
        cue: bool,
        request: &SourceRequest,
    ) -> Option<Playing> {
        let rt = self.players.get(&player)?;
        let (bus_key, channel) = if cue {
            rt.cue.clone()?
        } else {
            rt.main.clone()
        };
        let volume = if cue {
            rt.cue_volume.clone()
        } else {
            rt.volume.clone()
        };
        let ring = ((self.settings.tuning.prebuffer_secs * f64::from(self.settings.sample_rate))
            as usize)
            .max(1024);
        let bus = self.buses.get_mut(&bus_key)?;
        let Some(slot) = bus.alloc_slot() else {
            tracing::error!(?player, bus = ?bus_key, "no free mixer slot");
            return None;
        };
        let (producer, consumer) = source_pair(ring);
        let shared = consumer.shared.clone();
        if !bus.send(BusCommand::Attach {
            slot,
            source: consumer,
            volume,
            first_channel: channel,
        }) {
            tracing::error!(?player, "bus command queue full");
            return None;
        }
        self.next_key += 1;
        let key = SourceKey(self.next_key);
        rt.worker
            .load(key, request.path.clone(), request.from_secs, producer);
        self.owners.insert(key, player);
        Some(Playing {
            key,
            bus: bus_key,
            slot,
            request: request.clone(),
            entry: request.entry,
            start_secs: request.from_secs,
            shared,
            start: StartState::Idle,
            report_end: false,
        })
    }

    fn send(&mut self, bus: &BusKey, command: BusCommand) {
        if let Some(b) = self.buses.get_mut(bus)
            && !b.send(command)
        {
            self.dropped_commands += 1;
            tracing::error!(?bus, "bus command queue full; command dropped");
        }
    }

    fn now_frame(&self, bus: &BusKey) -> u64 {
        self.buses.get(bus).map_or(0, Bus::now_frame)
    }

    fn detach(&mut self, p: &Playing) {
        self.send(&p.bus, BusCommand::Detach { slot: p.slot });
        if let Some(owner) = self.owners.get(&p.key)
            && let Some(rt) = self.players.get(owner)
        {
            rt.worker.drop_source(p.key);
        }
    }

    fn release(&mut self, p: Playing) {
        self.detach(&p);
        self.owners.remove(&p.key);
    }

    /// Fades a source out over `frames` from now and stops it; it is released
    /// when the mixer reports it finished.
    fn fade_out(&mut self, player: PlayerId, p: Playing, frames: u64, curve: Curve) {
        if p.start != StartState::Started {
            self.release(p);
            return;
        }
        let now = self.now_frame(&p.bus);
        let len = u32::try_from(frames).unwrap_or(u32::MAX);
        self.send(&p.bus, BusCommand::Cancel { slot: p.slot });
        self.send(
            &p.bus,
            BusCommand::Ramp {
                slot: p.slot,
                to: 0.0,
                frames: len,
                curve,
                at_frame: now,
            },
        );
        self.send(
            &p.bus,
            BusCommand::StopAt {
                slot: p.slot,
                at_frame: now + frames,
            },
        );
        if let Some(rt) = self.players.get_mut(&player) {
            rt.outgoing.push(p);
        }
    }

    fn stop_quick_and_release(&mut self, p: Playing) {
        if p.start != StartState::Started {
            // Never reached the mixer's clock: there is nothing to ramp down.
            self.send(&p.bus, BusCommand::Cancel { slot: p.slot });
            self.release(p);
            return;
        }
        // A de-click ramp then detach; nothing is reported for it.
        let frames = self.settings.frames(self.settings.tuning.declick_ms);
        let now = self.now_frame(&p.bus);
        let len = u32::try_from(frames).unwrap_or(u32::MAX);
        self.send(&p.bus, BusCommand::Cancel { slot: p.slot });
        self.send(
            &p.bus,
            BusCommand::Ramp {
                slot: p.slot,
                to: 0.0,
                frames: len,
                curve: Curve::Linear,
                at_frame: now,
            },
        );
        self.send(
            &p.bus,
            BusCommand::StopAt {
                slot: p.slot,
                at_frame: now + frames,
            },
        );
        // Keep it until it finishes so the ramp is heard; then it is released.
        let owner = self.owners.get(&p.key).copied();
        match owner.and_then(|o| self.players.get_mut(&o)) {
            Some(rt) => rt.outgoing.push(p),
            None => self.release(p),
        }
    }

    /// Takes back a transition that was sent to the mixer but has not happened.
    fn undispatch(&mut self, player: PlayerId) {
        let Some(rt) = self.players.get_mut(&player) else {
            return;
        };
        let Plan::Dispatched(plan) = rt.plan else {
            return;
        };
        rt.plan = Plan::Waiting(plan);
        let mut cancels = Vec::new();
        if let Some(c) = rt.current.as_ref() {
            cancels.push((c.bus.clone(), c.slot));
        }
        if let Some(p) = rt.preload.as_mut()
            && p.start == StartState::Requested
        {
            p.start = StartState::Idle;
            cancels.push((p.bus.clone(), p.slot));
        }
        for (bus, slot) in cancels {
            self.send(&bus, BusCommand::Cancel { slot });
        }
    }

    fn preload(&mut self, player: PlayerId, request: Option<SourceRequest>) {
        self.undispatch(player);
        let Some(rt) = self.players.get_mut(&player) else {
            return;
        };
        if let (Some(old), Some(req)) = (rt.preload.as_ref(), request.as_ref())
            && old.entry == req.entry
            && old.start_secs == req.from_secs
        {
            return;
        }
        if let Some(old) = rt.preload.take() {
            self.release(old);
        }
        if let Some(req) = request {
            let p = self.new_source(player, false, &req);
            if let Some(rt) = self.players.get_mut(&player) {
                rt.preload = p;
            }
        }
    }

    /// Takes the preload if it matches `request`, otherwise opens a new source.
    fn take_or_open(&mut self, player: PlayerId, request: &SourceRequest) -> Option<Playing> {
        let rt = self.players.get_mut(&player)?;
        let matches = rt
            .preload
            .as_ref()
            .is_some_and(|p| p.entry == request.entry && p.start_secs == request.from_secs);
        if matches {
            return rt.preload.take();
        }
        self.new_source(player, false, request)
    }

    fn start_current(
        &mut self,
        player: PlayerId,
        request: &SourceRequest,
        crossfade_ms: Option<u32>,
    ) {
        self.undispatch(player);
        let Some(mut next) = self.take_or_open(player, request) else {
            self.events.push(EngineEvent::SourceFailed {
                player,
                entry: request.entry,
            });
            return;
        };
        if next.start == StartState::Requested {
            self.send(&next.bus, BusCommand::Cancel { slot: next.slot });
        }
        next.start = StartState::WhenReady { fade_in: false };
        let Some(rt) = self.players.get_mut(&player) else {
            return;
        };
        rt.paused = false;
        rt.plan = Plan::None;
        let old = rt.current.replace(next);
        if let Some(old) = old {
            match crossfade_ms {
                Some(ms) => {
                    let frames = self.settings.frames(f64::from(ms));
                    if let Some(rt) = self.players.get_mut(&player) {
                        rt.notify_fade = true;
                    }
                    self.fade_out(player, old, frames, Curve::EqualPower);
                }
                None => self.stop_quick_and_release(old),
            }
        } else if crossfade_ms.is_some() {
            // Nothing to fade: complete immediately.
            self.events.push(EngineEvent::FadeCompleted { player });
        }
    }

    fn fade_out_and_stop(&mut self, player: PlayerId, fade_ms: u32) {
        self.undispatch(player);
        let frames = self.settings.frames(f64::from(fade_ms));
        let Some(rt) = self.players.get_mut(&player) else {
            return;
        };
        rt.plan = Plan::None;
        let current = rt.current.take();
        let outgoing: Vec<Playing> = std::mem::take(&mut rt.outgoing);
        for p in outgoing {
            self.fade_out(player, p, frames, Curve::EqualPower);
        }
        match current {
            Some(mut p) if p.start == StartState::Started => {
                p.report_end = true;
                self.fade_out(player, p, frames, Curve::EqualPower);
            }
            Some(p) => {
                let entry = p.entry;
                self.release(p);
                self.events.push(EngineEvent::ReachedEnd { player, entry });
            }
            None => {}
        }
    }

    fn pause(&mut self, player: PlayerId) {
        self.undispatch(player);
        let ramp = u32::try_from(self.settings.frames(self.settings.tuning.pause_ramp_ms))
            .unwrap_or(u32::MAX);
        let Some(rt) = self.players.get_mut(&player) else {
            return;
        };
        rt.paused = true;
        let targets: Vec<(BusKey, usize)> = rt
            .current
            .iter()
            .chain(rt.outgoing.iter())
            .map(|p| (p.bus.clone(), p.slot))
            .collect();
        for (bus, slot) in targets {
            self.send(
                &bus,
                BusCommand::Pause {
                    slot,
                    ramp_frames: ramp,
                },
            );
        }
    }

    fn resume(&mut self, player: PlayerId) {
        let ramp = u32::try_from(self.settings.frames(self.settings.tuning.pause_ramp_ms))
            .unwrap_or(u32::MAX);
        let Some(rt) = self.players.get_mut(&player) else {
            return;
        };
        rt.paused = false;
        let mut targets: Vec<(BusKey, usize)> = rt
            .outgoing
            .iter()
            .map(|p| (p.bus.clone(), p.slot))
            .collect();
        if let Some(c) = rt.current.as_mut() {
            if c.start == StartState::Idle {
                c.start = StartState::WhenReady { fade_in: true };
            } else {
                targets.push((c.bus.clone(), c.slot));
            }
        }
        for (bus, slot) in targets {
            self.send(
                &bus,
                BusCommand::Resume {
                    slot,
                    ramp_frames: ramp,
                },
            );
        }
    }

    fn stop_now(&mut self, player: PlayerId) {
        self.undispatch(player);
        let Some(rt) = self.players.get_mut(&player) else {
            return;
        };
        rt.plan = Plan::None;
        rt.paused = false;
        rt.notify_fade = false;
        let all: Vec<Playing> = rt
            .current
            .take()
            .into_iter()
            .chain(rt.outgoing.drain(..))
            .collect();
        for p in all {
            self.stop_quick_and_release(p);
        }
    }

    fn seek(&mut self, player: PlayerId, secs: f64) {
        self.undispatch(player);
        let Some(mut request) = self
            .players
            .get(&player)
            .and_then(|rt| rt.current.as_ref())
            .map(|c| c.request.clone())
        else {
            return;
        };
        request.from_secs = secs;
        let Some(mut next) = self.new_source(player, false, &request) else {
            return;
        };
        let Some(rt) = self.players.get_mut(&player) else {
            return;
        };
        next.start = if rt.paused {
            StartState::Idle
        } else {
            StartState::WhenReady { fade_in: true }
        };
        if let Some(old) = rt.current.replace(next) {
            self.stop_quick_and_release(old);
        }
    }

    fn start_cue(&mut self, player: PlayerId, request: &SourceRequest) {
        if let Some(old) = self
            .players
            .get_mut(&player)
            .and_then(|rt| rt.cue_src.take())
        {
            self.stop_quick_and_release(old);
        }
        match self.new_source(player, true, request) {
            Some(mut cue) => {
                cue.start = StartState::WhenReady { fade_in: false };
                if let Some(rt) = self.players.get_mut(&player) {
                    rt.cue_src = Some(cue);
                }
            }
            // No Cue output configured: nothing can be heard, end it at once.
            None => self.events.push(EngineEvent::CueEnded { player }),
        }
    }
}

impl Engine {
    /// Advances the engine: supervises devices, turns bus and worker
    /// observations into model events, starts sources that became ready and
    /// dispatches transitions that are due. Returns the events for the model.
    pub fn tick(&mut self, now: Instant) -> Vec<EngineEvent> {
        self.handle_failures();
        let keys: Vec<BusKey> = self.buses.keys().cloned().collect();
        for key in keys {
            let events = match self.buses.get_mut(&key) {
                Some(bus) => {
                    bus.supervise(now);
                    bus.poll()
                }
                None => Vec::new(),
            };
            for event in events {
                self.handle_bus_event(&key, event);
            }
        }
        self.start_ready_sources();
        self.dispatch_plans();
        std::mem::take(&mut self.events)
    }

    fn handle_failures(&mut self) {
        let failures: Vec<WorkerFailure> = self.failures_rx.try_iter().collect();
        for failure in failures {
            let Some(player) = self.owners.get(&failure.key).copied() else {
                continue;
            };
            tracing::warn!(?player, error = %failure.error, "source failed");
            let Some(rt) = self.players.get_mut(&player) else {
                continue;
            };
            let taken = if rt.current.as_ref().is_some_and(|p| p.key == failure.key) {
                rt.plan = Plan::None;
                rt.current.take()
            } else if rt.preload.as_ref().is_some_and(|p| p.key == failure.key) {
                rt.preload.take()
            } else if rt.cue_src.as_ref().is_some_and(|p| p.key == failure.key) {
                rt.cue_src.take()
            } else {
                None
            };
            if let Some(p) = taken {
                let entry = p.entry;
                self.release(p);
                self.events
                    .push(EngineEvent::SourceFailed { player, entry });
            }
        }
    }

    fn find(&self, bus: &BusKey, slot: usize) -> Option<(PlayerId, Role)> {
        self.players.iter().find_map(|(id, rt)| {
            let is = |p: &Playing| &p.bus == bus && p.slot == slot;
            if rt.current.as_ref().is_some_and(is) {
                Some((*id, Role::Current))
            } else if rt.preload.as_ref().is_some_and(is) {
                Some((*id, Role::Preload))
            } else if rt.cue_src.as_ref().is_some_and(is) {
                Some((*id, Role::Cue))
            } else {
                rt.outgoing
                    .iter()
                    .position(is)
                    .map(|i| (*id, Role::Outgoing(i)))
            }
        })
    }

    fn handle_bus_event(&mut self, bus: &BusKey, event: BusEvent) {
        match event {
            BusEvent::Started { slot, .. } => {
                let Some((player, role)) = self.find(bus, slot) else {
                    return;
                };
                let Some(rt) = self.players.get_mut(&player) else {
                    return;
                };
                match role {
                    Role::Preload => {
                        // A dispatched transition happened: the next is now on air.
                        let Some(mut next) = rt.preload.take() else {
                            return;
                        };
                        next.start = StartState::Started;
                        let entry = next.entry;
                        let overlapping = matches!(
                            rt.plan,
                            Plan::Dispatched(TransitionPlan::StartNextAt {
                                fade_current_until_secs: Some(_),
                                ..
                            })
                        );
                        rt.plan = Plan::None;
                        // The audio moved on: whatever was requested meanwhile, the
                        // next is playing, and the model follows the engine.
                        rt.paused = false;
                        if let Some(old) = rt.current.replace(next) {
                            rt.outgoing.push(old);
                        }
                        rt.notify_fade |= overlapping;
                        self.events
                            .push(EngineEvent::TransitionStarted { player, entry });
                    }
                    Role::Current => {
                        if let Some(c) = rt.current.as_mut() {
                            c.start = StartState::Started;
                        }
                    }
                    Role::Cue => {
                        if let Some(c) = rt.cue_src.as_mut() {
                            c.start = StartState::Started;
                        }
                    }
                    Role::Outgoing(i) => {
                        if let Some(o) = rt.outgoing.get_mut(i) {
                            o.start = StartState::Started;
                        }
                    }
                }
            }
            BusEvent::Finished { slot, .. } => {
                let Some((player, role)) = self.find(bus, slot) else {
                    return;
                };
                match role {
                    Role::Current => self.current_finished(player),
                    Role::Outgoing(i) => {
                        let Some(rt) = self.players.get_mut(&player) else {
                            return;
                        };
                        if i < rt.outgoing.len() {
                            let p = rt.outgoing.remove(i);
                            let report = p.report_end.then_some(p.entry);
                            let done = rt.outgoing.is_empty() && rt.notify_fade;
                            if done {
                                rt.notify_fade = false;
                            }
                            self.release(p);
                            if let Some(entry) = report {
                                self.events.push(EngineEvent::ReachedEnd { player, entry });
                            } else if done {
                                self.events.push(EngineEvent::FadeCompleted { player });
                            }
                        }
                    }
                    Role::Cue => {
                        if let Some(p) = self
                            .players
                            .get_mut(&player)
                            .and_then(|rt| rt.cue_src.take())
                        {
                            self.release(p);
                            self.events.push(EngineEvent::CueEnded { player });
                        }
                    }
                    Role::Preload => {
                        if let Some(p) = self
                            .players
                            .get_mut(&player)
                            .and_then(|rt| rt.preload.take())
                        {
                            self.release(p);
                        }
                    }
                }
            }
        }
    }

    /// The current source stopped by itself (end of stream or a planned stop).
    fn current_finished(&mut self, player: PlayerId) {
        let Some(rt) = self.players.get_mut(&player) else {
            return;
        };
        let Some(finished) = rt.current.take() else {
            return;
        };
        let plan = match rt.plan {
            Plan::Waiting(p) | Plan::Dispatched(p) => Some(p),
            Plan::None => None,
        };
        let entry = finished.entry;
        self.release(finished);
        match plan {
            // End of stream before (or instead of) the scheduled transition:
            // start the next right now.
            Some(TransitionPlan::StartNextAt { .. }) => {
                let Some(rt) = self.players.get_mut(&player) else {
                    return;
                };
                rt.plan = Plan::Dispatched(TransitionPlan::StartNextAt {
                    at_secs: 0.0,
                    fade_current_until_secs: None,
                });
                match rt.preload.as_mut() {
                    Some(next) if next.start != StartState::Started => {
                        next.start = StartState::Requested;
                        let (bus, slot) = (next.bus.clone(), next.slot);
                        let at = self.now_frame(&bus);
                        self.send(&bus, BusCommand::Start { slot, at_frame: at });
                    }
                    _ => {
                        rt.plan = Plan::None;
                        self.events.push(EngineEvent::ReachedEnd { player, entry });
                    }
                }
            }
            _ => {
                if let Some(rt) = self.players.get_mut(&player) {
                    rt.plan = Plan::None;
                }
                self.events.push(EngineEvent::ReachedEnd { player, entry });
            }
        }
    }

    fn start_ready_sources(&mut self) {
        let declick = u32::try_from(self.settings.frames(self.settings.tuning.declick_ms))
            .unwrap_or(u32::MAX);
        let mut starts: Vec<(BusKey, usize, bool)> = Vec::new();
        for rt in self.players.values_mut() {
            let current = if rt.paused { None } else { rt.current.as_mut() };
            for p in current.into_iter().chain(rt.cue_src.iter_mut()) {
                if let StartState::WhenReady { fade_in } = p.start
                    && p.shared.is_ready()
                    && !p.shared.is_failed()
                {
                    p.start = StartState::Requested;
                    starts.push((p.bus.clone(), p.slot, fade_in));
                }
            }
        }
        for (bus, slot, fade_in) in starts {
            let now = self.now_frame(&bus);
            if fade_in {
                self.send(
                    &bus,
                    BusCommand::Ramp {
                        slot,
                        to: 0.0,
                        frames: 0,
                        curve: Curve::Linear,
                        at_frame: 0,
                    },
                );
            }
            self.send(
                &bus,
                BusCommand::Start {
                    slot,
                    at_frame: now,
                },
            );
            if fade_in {
                self.send(
                    &bus,
                    BusCommand::Ramp {
                        slot,
                        to: 1.0,
                        frames: declick,
                        curve: Curve::Linear,
                        at_frame: now,
                    },
                );
            }
        }
    }

    /// Sends due transitions to the mixer with their exact frame.
    fn dispatch_plans(&mut self) {
        let rate = f64::from(self.settings.sample_rate);
        let lead = self.settings.frames(self.settings.tuning.schedule_lead_ms);
        let declick = self.settings.frames(self.settings.tuning.declick_ms);
        let mut due: Vec<(PlayerId, TransitionPlan, u64)> = Vec::new();
        for (id, rt) in &self.players {
            let Plan::Waiting(plan) = rt.plan else {
                continue;
            };
            let Some(current) = rt.current.as_ref() else {
                continue;
            };
            if rt.paused || current.start != StartState::Started {
                continue;
            }
            let target = match plan {
                TransitionPlan::StopAt { at_secs }
                | TransitionPlan::StartNextAt { at_secs, .. } => at_secs,
            };
            if !target.is_finite() {
                continue; // `SOURCE_END`: handled when the source finishes.
            }
            let Some(bus) = self.buses.get(&current.bus) else {
                continue;
            };
            let (now, played) = bus
                .shared()
                .consistent(|| (bus.now_frame(), current.shared.frames_played()));
            let position = current.start_secs + played as f64 / rate;
            let until = ((target - position) * rate).round().max(0.0) as u64;
            if until <= lead {
                due.push((*id, plan, now + until));
            }
        }
        for (player, plan, at_frame) in due {
            self.dispatch(player, plan, at_frame, declick);
        }
    }

    fn dispatch(&mut self, player: PlayerId, plan: TransitionPlan, at_frame: u64, declick: u64) {
        let rate = f64::from(self.settings.sample_rate);
        let Some(rt) = self.players.get_mut(&player) else {
            return;
        };
        rt.plan = Plan::Dispatched(plan);
        let Some(current) = rt.current.as_ref() else {
            return;
        };
        let (cur_bus, cur_slot) = (current.bus.clone(), current.slot);
        let next = match plan {
            TransitionPlan::StartNextAt { .. } => rt.preload.as_mut().map(|p| {
                p.start = StartState::Requested;
                (p.bus.clone(), p.slot)
            }),
            TransitionPlan::StopAt { .. } => None,
        };
        let declick32 = u32::try_from(declick).unwrap_or(u32::MAX);
        match (plan, next) {
            (
                TransitionPlan::StartNextAt {
                    at_secs,
                    fade_current_until_secs,
                },
                Some((bus, slot)),
            ) => {
                self.send(&bus, BusCommand::Start { slot, at_frame });
                match fade_current_until_secs {
                    Some(until) => {
                        let len = (((until - at_secs) * rate).round() as u64).max(declick);
                        let len32 = u32::try_from(len).unwrap_or(u32::MAX);
                        self.send(
                            &cur_bus,
                            BusCommand::Ramp {
                                slot: cur_slot,
                                to: 0.0,
                                frames: len32,
                                curve: Curve::EqualPower,
                                at_frame,
                            },
                        );
                        self.send(
                            &cur_bus,
                            BusCommand::StopAt {
                                slot: cur_slot,
                                at_frame: at_frame + len,
                            },
                        );
                    }
                    None => {
                        let fade_at = at_frame.saturating_sub(declick);
                        self.send(
                            &cur_bus,
                            BusCommand::Ramp {
                                slot: cur_slot,
                                to: 0.0,
                                frames: declick32,
                                curve: Curve::Linear,
                                at_frame: fade_at,
                            },
                        );
                        self.send(
                            &cur_bus,
                            BusCommand::StopAt {
                                slot: cur_slot,
                                at_frame,
                            },
                        );
                    }
                }
            }
            // A stop, or a start with nothing preloaded: end the current track there.
            _ => {
                let fade_at = at_frame.saturating_sub(declick);
                self.send(
                    &cur_bus,
                    BusCommand::Ramp {
                        slot: cur_slot,
                        to: 0.0,
                        frames: declick32,
                        curve: Curve::Linear,
                        at_frame: fade_at,
                    },
                );
                self.send(
                    &cur_bus,
                    BusCommand::StopAt {
                        slot: cur_slot,
                        at_frame,
                    },
                );
            }
        }
    }
}

impl Engine {
    /// One-line description of a player's engine state, for logs and test failures.
    pub fn describe_player(&self, player: PlayerId) -> String {
        let Some(rt) = self.players.get(&player) else {
            return "none".into();
        };
        let d = |p: &Playing| {
            format!(
                "key{} slot{} start {:?} ready {} failed {} eof {} played {} under {}",
                p.key.0,
                p.slot,
                p.start,
                p.shared.is_ready(),
                p.shared.is_failed(),
                p.shared.is_eof(),
                p.shared.frames_played(),
                p.shared
                    .underruns
                    .load(std::sync::atomic::Ordering::Relaxed)
            )
        };
        format!(
            "paused {} plan {:?} cur [{}] pre [{}] outgoing {} cap {}",
            rt.paused,
            rt.plan,
            rt.current.as_ref().map(d).unwrap_or_default(),
            rt.preload.as_ref().map(d).unwrap_or_default(),
            rt.outgoing.len(),
            self.buses.values().map(|b| b.capacity()).sum::<usize>()
        )
    }
}
