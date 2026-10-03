//! Executes model `EngineAction`s on buses and worker threads, and turns what
//! the buses and workers observe into model `EngineEvent`s (spec §4.4). All
//! timing is in bus frames: transitions are dispatched ahead of time
//! (`schedule_lead`) with an exact frame, so they never depend on when this
//! thread happens to wake.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, Sender};
use fp_backends::{AudioBackend, Availability, NullBackend, StreamConfig};
use fp_model::{
    CartwallRoutes, Config, EngineAction, EngineEvent, EntryId, PlayerId, PlayerRoutes, Route,
    SourceRequest, TransitionPlan, Tuning,
};

use crate::atomic::AtomicF32;
use crate::bus::{Bus, BusHealth, BusKey, BusTiming};
use crate::mixer::{BusCommand, BusEvent, MixerConfig};
use crate::ramp::Curve;
use crate::source::{SourceShared, source_pair};
use crate::worker::{PlayerWorker, SourceKey, SourceOpener, WorkerFailure};

mod carts;
pub use carts::{CartTelemetry, CartwallTelemetry};

/// Stream settings and tuning, derived from the model `Config`.
#[derive(Debug, Clone)]
pub struct EngineSettings {
    pub sample_rate: u32,
    pub buffer_frames: u32,
    pub channels: u16,
    pub tuning: Tuning,
    pub routes: Vec<PlayerRoutes>,
    pub cartwall_routes: CartwallRoutes,
    /// Backend used when a player has no explicit route (`None`: the first registered).
    pub default_backend: Option<String>,
    /// Devices played bit-perfect (Phase 4 spec B1).
    pub bit_perfect: std::collections::HashSet<BusKey>,
}

impl EngineSettings {
    pub fn from_config(config: &Config) -> Self {
        Self {
            sample_rate: config.outputs.sample_rate,
            buffer_frames: config.outputs.buffer_frames,
            channels: 2,
            tuning: config.tuning.clone(),
            routes: config.outputs.routes.clone(),
            cartwall_routes: config.outputs.cartwall.clone(),
            default_backend: config.outputs.backend.clone(),
            bit_perfect: config
                .outputs
                .bit_perfect
                .iter()
                .map(|d| BusKey {
                    backend: d.backend.clone(),
                    device: d.device.clone(),
                })
                .collect(),
        }
    }

    fn frames(&self, ms: f64) -> u64 {
        frames_at(self.sample_rate, ms)
    }
}

/// What the UI shows for one player, refreshed every tick.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PlayerTelemetry {
    /// Position of the current source in track seconds.
    pub position_secs: Option<f64>,
    pub cue_position_secs: Option<f64>,
    /// The player's meter, filled in by the conductor (meters spec M2).
    pub meter: crate::meter::MeterReading,
    pub underruns: u64,
    /// The current source reaches its Main device unchanged: the BP badge
    /// (Phase 4 spec B5).
    pub bit_perfect: bool,
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
    /// The worker failed after this source started: let the buffered audio
    /// play out, then report `SourceFailed` (spec §4.5).
    failed: bool,
    /// A pre-listen source (never paused with the player).
    cue: bool,
    /// A CUE source held by the CUE window's Pause: the mixer never
    /// evaluates a stop on a paused slot, so it is released directly.
    held: bool,
}

/// Why a source could not be created. None of these is a problem with the
/// file, so none is ever reported to the model as `SourceFailed`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AttachError {
    NoPlayer,
    NoRoute,
    NoSlot,
    QueueFull,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Plan {
    None,
    Waiting(TransitionPlan),
    /// Sent to the mixer; the transition happens at this bus frame.
    Dispatched(TransitionPlan, u64),
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
    /// The CUE source is held (the CUE window's Pause). Cleared by a new
    /// or stopped CUE.
    cue_paused: bool,
    plan: Plan,
    paused: bool,
    /// Emit `FadeCompleted` once `outgoing` is empty.
    notify_fade: bool,
}

pub struct Engine {
    backends: Vec<Arc<dyn AudioBackend>>,
    /// Backends that were unavailable when the engine started (a JACK
    /// server not running, a missing library): routes to them fall back as
    /// if the backend did not exist. One that was available and is lost
    /// later keeps its routes, and the watchdog retries it.
    unusable: std::collections::HashSet<String>,
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
    slot_exhaustions: u64,
    /// Test tones playing, by (bus, slot); released when they finish.
    tones: Vec<(BusKey, usize)>,
    /// Created on the first cart action.
    cartwall: Option<carts::CartwallRuntime>,
    /// The latest time `execute` or `tick` was given, for bus reopens made
    /// deep inside an action.
    now: Instant,
    /// Whether meters measure true peak (new buses start with it).
    true_peak: bool,
    /// Programme-meter integrator (ms, ms, dB/s) for new buses.
    integration: (f32, f32, f32),
}

/// Which source of a player a bus slot belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Role {
    Preload,
    Current,
    Outgoing(usize),
    Cue,
}

fn store_integration(shared: &crate::mixer::BusShared, (tau1, tau2, fall): (f32, f32, f32)) {
    shared.ppm_tau1_ms.store(tau1);
    shared.ppm_tau2_ms.store(tau2);
    shared.fall_db_per_sec.store(fall);
}

/// Empties a source's meter accumulators.
fn take_measurement(s: &SourceShared) {
    s.peak_l.take();
    s.peak_r.take();
    s.sum_sq_l.take();
    s.sum_sq_r.take();
    s.sum_abs_l.take();
    s.sum_abs_r.take();
    s.k_sum_l.take();
    s.k_sum_r.take();
    s.measured_frames
        .swap(0, std::sync::atomic::Ordering::AcqRel);
}

/// `ms` milliseconds in frames at `rate`.
fn frames_at(rate: u32, ms: f64) -> u64 {
    (ms.max(0.0) * f64::from(rate) / 1000.0).round() as u64
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
        let unusable = backends
            .iter()
            .filter(|b| b.availability() != Availability::Available)
            .map(|b| b.id().0)
            .collect();
        let (failures_tx, failures_rx) = crossbeam_channel::unbounded();
        Self {
            backends,
            unusable,
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
            slot_exhaustions: 0,
            tones: Vec::new(),
            cartwall: None,
            now: Instant::now(),
            true_peak: false,
            integration: (0.0, 0.0, 0.0),
        }
    }

    /// Sources that could not be attached for lack of a mixer slot.
    pub fn slot_exhaustions(&self) -> u64 {
        self.slot_exhaustions
    }

    /// Sources still waiting for their worker: pending starts, and preloads
    /// not yet ready. Tests poll this instead of sleeping.
    pub fn unsettled_sources(&self) -> usize {
        self.players
            .values()
            .map(|rt| {
                let waiting = rt
                    .current
                    .iter()
                    .chain(rt.cue_src.iter())
                    // A failed source counts until its failure has been handled
                    // (the worker flags it just before it reports).
                    .filter(|p| matches!(p.start, StartState::WhenReady { .. }))
                    .count();
                let preload = rt
                    .preload
                    .iter()
                    .filter(|p| !p.shared.is_ready() && !p.shared.is_failed())
                    .count();
                // A held CUE source (Idle) still being filled, like a preload.
                let held_cue = rt
                    .cue_src
                    .iter()
                    .filter(|p| {
                        p.start == StartState::Idle && !p.shared.is_ready() && !p.shared.is_failed()
                    })
                    .count();
                waiting + preload + held_cue
            })
            .sum::<usize>()
            + self.unsettled_carts()
    }

    /// Mixer slots in use across all buses (a leak shows up here).
    pub fn used_slots(&self) -> usize {
        self.buses.values().map(|b| b.used_slots()).sum()
    }

    /// Test tones still playing.
    pub fn active_tones(&self) -> usize {
        self.tones.len()
    }

    /// Plays a sine test tone on `route` (Settings: "Test Main"/"Test Cue").
    /// The whole tone is rendered into the ring up front, so no worker is
    /// involved; short de-click ramps avoid clicks at both ends.
    pub fn play_test_tone(
        &mut self,
        route: &Route,
        frequency_hz: f32,
        secs: f32,
        level_db: f32,
        now: Instant,
    ) {
        let (key, channel) = self.route_target(route);
        self.ensure_bus(&key, now);
        let rate = self.rate_of(&key) as f32;
        let frames = (secs.max(0.0) * rate) as usize;
        let ramp = (self.frames_on(&key, self.settings.tuning.declick_ms) as usize).max(1);
        let amplitude = 10f32.powf(level_db / 20.0);
        let step = std::f32::consts::TAU * frequency_hz / rate;
        let samples: Vec<f32> = (0..frames)
            .flat_map(|i| {
                let envelope = (i.min(frames - 1 - i).min(ramp) as f32 / ramp as f32).min(1.0);
                let v = amplitude * envelope * (step * i as f32).sin();
                [v, v]
            })
            .collect();
        let Some(bus) = self.buses.get_mut(&key) else {
            return;
        };
        // Attach and Start must both fit, or the slot would never be freed.
        if bus.command_room() < 2 {
            self.dropped_commands += 1;
            return;
        }
        let Some(slot) = bus.alloc_slot() else {
            self.slot_exhaustions += 1;
            return;
        };
        let (mut producer, consumer) = source_pair(frames.max(1));
        producer.push(&samples);
        producer
            .shared
            .eof
            .store(true, std::sync::atomic::Ordering::Release);
        producer
            .shared
            .ready
            .store(true, std::sync::atomic::Ordering::Release);
        let volume = Arc::new(AtomicF32::new(1.0));
        let at = bus.now_frame();
        if bus.send(BusCommand::Attach {
            slot,
            source: consumer,
            volume,
            first_channel: channel,
        }) && bus.send(BusCommand::Start { slot, at_frame: at })
        {
            self.tones.push((key, slot));
        } else {
            self.dropped_commands += 1;
        }
    }

    /// Sources attached to mixers across all players (a leak shows up here).
    pub fn attached_sources(&self) -> usize {
        self.players
            .values()
            .map(|rt| {
                rt.preload.iter().count()
                    + rt.current.iter().count()
                    + rt.cue_src.iter().count()
                    + rt.outgoing.len()
            })
            .sum()
    }

    /// Bus commands that could not be queued, plus test tones refused because
    /// the queue had no room for them.
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

    /// Whether `p` reaches its device unchanged (Phase 4 spec B5): a
    /// bit-perfect bus open with exclusive access, the file at the bus rate
    /// with a sample size the device format holds, and the mixer reporting
    /// the source unaltered.
    fn is_bit_perfect(&self, p: &Playing) -> bool {
        let Some(bus) = self.buses.get(&p.bus) else {
            return false;
        };
        let Some(format) = p.request.format else {
            return false;
        };
        let Some(bits) = format.bits else {
            return false;
        };
        // Sources are stereo: more channels are downmixed (altered).
        if !(1..=2).contains(&format.channels) {
            return false;
        }
        !p.cue
            && p.start == StartState::Started
            && self.settings.bit_perfect.contains(&p.bus)
            && bus.exclusive_granted()
            && format.sample_rate == bus.sample_rate()
            && bus.sample_format().is_some_and(|f| f.holds_bits(bits))
            && p.shared
                .unaltered
                .load(std::sync::atomic::Ordering::Acquire)
    }

    /// What `player`'s sources on air (current and fading out) measured
    /// since the last call; the measurement is taken, not copied.
    pub fn take_meter_input(&self, player: PlayerId) -> crate::meter::MeterInput {
        let mut input = crate::meter::MeterInput::default();
        let Some(rt) = self.players.get(&player) else {
            return input;
        };
        // The pre-listen is not on air: its measurement is taken and dropped
        // so it does not pile up.
        for p in rt.cue_src.iter() {
            take_measurement(&p.shared);
        }
        for p in rt.current.iter().chain(rt.outgoing.iter()) {
            let s = &p.shared;
            let mut source = crate::meter::MeterInput::default();
            let mut take = || {
                source.extend(crate::meter::MeterInput {
                    peak: [s.peak_l.take(), s.peak_r.take()],
                    sum_sq: [s.sum_sq_l.take(), s.sum_sq_r.take()],
                    sum_abs: [s.sum_abs_l.take(), s.sum_abs_r.take()],
                    k_sum: [s.k_sum_l.take(), s.k_sum_r.take()],
                    frames: s
                        .measured_frames
                        .swap(0, std::sync::atomic::Ordering::AcqRel),
                });
            };
            // The sums are taken one by one: a block rendered meanwhile is
            // taken whole in the same reading (a tick never gets half a block).
            match self.buses.get(&p.bus) {
                Some(bus) => bus.shared().whole_blocks(take),
                None => take(),
            }
            input.merge(source);
        }
        input
    }

    /// Meters measure true peak on every bus (meters spec M1).
    pub fn set_true_peak(&mut self, on: bool) {
        self.true_peak = on;
        for bus in self.buses.values() {
            bus.shared()
                .true_peak
                .store(on, std::sync::atomic::Ordering::Release);
        }
    }

    pub fn true_peak(&self) -> bool {
        self.true_peak
    }

    /// The programme-meter integrator every bus runs per sample (two time
    /// constants in ms, and the fall in dB/s; meters spec M1).
    pub fn set_meter_integration(&mut self, integration: (f32, f32, f32)) {
        self.integration = integration;
        for bus in self.buses.values() {
            store_integration(bus.shared(), integration);
        }
    }

    pub fn meter_integration(&self) -> (f32, f32, f32) {
        self.integration
    }

    pub fn telemetry(&self, player: PlayerId) -> PlayerTelemetry {
        let Some(rt) = self.players.get(&player) else {
            return PlayerTelemetry::default();
        };
        let position = |p: &Playing| {
            p.start_secs + p.shared.frames_played() as f64 / f64::from(self.rate_of(&p.bus))
        };
        PlayerTelemetry {
            position_secs: rt.current.as_ref().map(position),
            cue_position_secs: rt.cue_src.as_ref().map(position),
            meter: crate::meter::MeterReading::default(),
            underruns: rt.current.as_ref().map_or(0, |p| {
                p.shared
                    .underruns
                    .load(std::sync::atomic::Ordering::Relaxed)
            }),
            bit_perfect: rt.current.as_ref().is_some_and(|p| self.is_bit_perfect(p)),
        }
    }

    /// Whether anything on `bus` is audible or about to be: a source
    /// started, requested or waiting only to be ready, or a test tone.
    /// Sources that merely wait (preloads, a track loaded paused) do not
    /// count: they are reopened when the rate changes.
    fn bus_sounding(&self, bus: &BusKey) -> bool {
        let sounding = |start: StartState| start != StartState::Idle;
        let players = self.players.values().any(|rt| {
            rt.current
                .iter()
                .chain(rt.preload.iter())
                .chain(rt.cue_src.iter())
                .chain(rt.outgoing.iter())
                .any(|p| &p.bus == bus && sounding(p.start))
        });
        players || self.carts_sounding(bus) || self.tones.iter().any(|(b, _)| b == bus)
    }

    /// Before a source with `format` starts on `bus`: a bit-perfect bus
    /// with nothing sounding follows the file's rate (Phase 4 spec B3), and
    /// the sources waiting on it are reopened at the new rate. A sounding
    /// bus, or an unknown format, keeps the rate: the source is resampled.
    fn prepare_start(&mut self, bus: &BusKey, format: Option<fp_model::AudioFormat>) {
        if !self.settings.bit_perfect.contains(bus) {
            return;
        }
        let Some(rate) = format.map(|f| f.sample_rate).filter(|r| *r > 0) else {
            return;
        };
        if self.rate_of(bus) == rate || self.bus_sounding(bus) {
            return;
        }
        let now = self.now;
        let changed = self.buses.get_mut(bus).is_some_and(|b| {
            let before = b.sample_rate();
            b.reopen_at(rate, now);
            b.sample_rate() != before
        });
        if changed {
            self.reopen_waiting(bus);
        }
    }

    /// Re-creates every waiting (never started) player source on `bus` at
    /// the bus's new rate; their old ones were opened at the old rate.
    fn reopen_waiting(&mut self, bus: &BusKey) {
        let ids: Vec<PlayerId> = self.players.keys().copied().collect();
        for player in ids {
            let waiting = |p: &Option<Playing>| {
                p.as_ref()
                    .is_some_and(|p| &p.bus == bus && p.start == StartState::Idle)
            };
            let Some(rt) = self.players.get_mut(&player) else {
                continue;
            };
            let old_preload = if waiting(&rt.preload) {
                rt.preload.take()
            } else {
                None
            };
            let old_current = if waiting(&rt.current) {
                rt.current.take()
            } else {
                None
            };
            if let Some(old) = old_preload {
                let request = old.request.clone();
                self.release(old);
                let fresh = self.new_source(player, false, &request).ok();
                if let Some(rt) = self.players.get_mut(&player) {
                    rt.preload = fresh;
                }
            }
            if let Some(old) = old_current {
                let request = old.request.clone();
                self.release(old);
                match self.new_source(player, false, &request) {
                    Ok(fresh) => {
                        if let Some(rt) = self.players.get_mut(&player) {
                            rt.current = Some(fresh);
                        }
                    }
                    Err(e) => {
                        // An engine limitation, not a bad file: end the entry.
                        tracing::error!(?player, error = ?e, "cannot reopen a waiting source");
                        self.events.push(EngineEvent::ReachedEnd {
                            player,
                            entry: request.entry,
                        });
                    }
                }
            }
        }
    }

    fn find_backend(&self, id: &str) -> Option<Arc<dyn AudioBackend>> {
        self.backends.iter().find(|b| b.id().0 == id).cloned()
    }

    /// The rate `bus` runs at. Every source on a bus shares it, and it only
    /// changes while nothing is attached (Phase 4 spec B3).
    fn rate_of(&self, bus: &BusKey) -> u32 {
        self.buses
            .get(bus)
            .map_or(self.settings.sample_rate, Bus::sample_rate)
            .max(1)
    }

    /// `ms` milliseconds in frames of `bus`.
    fn frames_on(&self, bus: &BusKey, ms: f64) -> u64 {
        frames_at(self.rate_of(bus), ms)
    }

    /// A backend routes may use: registered and available at start-up.
    fn route_backend(&self, id: &str) -> Option<Arc<dyn AudioBackend>> {
        self.find_backend(id)
            .filter(|b| !self.unusable.contains(&b.id().0))
    }

    /// The configured default backend, else the first usable one.
    fn default_backend(&self) -> Arc<dyn AudioBackend> {
        self.settings
            .default_backend
            .as_deref()
            .and_then(|id| self.route_backend(id))
            .or_else(|| {
                self.backends
                    .iter()
                    .find(|b| !self.unusable.contains(&b.id().0))
                    .cloned()
            })
            .unwrap_or_else(|| Arc::new(NullBackend))
    }

    /// The default backend's default device, or the Null device.
    fn default_output(&self) -> BusKey {
        let backend = self.default_backend();
        match backend.default_device() {
            Some(device) => BusKey {
                backend: backend.id().0,
                device: device.0,
            },
            None => BusKey {
                backend: "null".to_owned(),
                device: "null".to_owned(),
            },
        }
    }

    /// Where a configured route really goes. A route naming a backend this
    /// machine does not have (e.g. a config copied from another OS) falls
    /// back to the default output: its device id belongs to the missing
    /// backend and would never open.
    fn route_target(&self, route: &Route) -> (BusKey, u16) {
        match self.route_backend(&route.backend) {
            Some(backend) => (
                BusKey {
                    backend: backend.id().0,
                    device: route.device.clone(),
                },
                route.first_channel,
            ),
            None => {
                tracing::warn!(backend = %route.backend, "route to an unavailable backend; using the default output");
                (self.default_output(), 0)
            }
        }
    }

    /// Where a Cue route really goes. Unlike Main, a Cue never falls back to
    /// the default output (that is where Main plays): a route to a backend
    /// this machine does not have, or one identical to Main, means no cue.
    fn cue_target(&self, route: &Route, main: &(BusKey, u16)) -> Option<(BusKey, u16)> {
        let Some(backend) = self.route_backend(&route.backend) else {
            tracing::warn!(backend = %route.backend, "cue route to an unavailable backend; no cue");
            return None;
        };
        let target = (
            BusKey {
                backend: backend.id().0,
                device: route.device.clone(),
            },
            route.first_channel,
        );
        if &target == main {
            tracing::warn!("cue route equals the main route; no cue");
            return None;
        }
        Some(target)
    }

    /// Resolves the (bus, first channel) of a player's Main and Cue outputs.
    fn resolve_routes(&self, player: PlayerId) -> ((BusKey, u16), Option<(BusKey, u16)>) {
        let routes = self.settings.routes.iter().find(|r| r.player == player);
        let main = match routes.and_then(|r| r.main.as_ref()) {
            Some(route) => self.route_target(route),
            None => (self.default_output(), 0),
        };
        let cue = routes
            .and_then(|r| r.cue.as_ref())
            .and_then(|route| self.cue_target(route, &main));
        (main, cue)
    }

    /// Channels a bus must open with so that every route to it fits (e.g.
    /// Main on 1/2 and Cue on 3/4 of one interface needs 4), capped by what
    /// the device reports.
    fn channels_for(&self, key: &BusKey) -> u16 {
        let wanted = self
            .settings
            .routes
            .iter()
            .flat_map(|r| r.main.iter().chain(r.cue.iter()))
            .chain(self.settings.cartwall_routes.main.iter())
            .chain(self.settings.cartwall_routes.cue.iter())
            .map(|route| self.route_target(route))
            .filter(|(k, _)| k == key)
            .map(|(_, first)| first.saturating_add(2))
            .fold(self.settings.channels.max(2), u16::max);
        let device_channels = self
            .find_backend(&key.backend)
            .and_then(|b| b.enumerate_devices().ok())
            .and_then(|devices| devices.into_iter().find(|d| d.id.0 == key.device))
            .map(|d| d.channels)
            .filter(|c| *c > 0);
        match device_channels {
            Some(available) => wanted.min(available).max(2.min(available)),
            None => wanted,
        }
    }

    fn ensure_bus(&mut self, key: &BusKey, now: Instant) {
        if !self.buses.contains_key(key) {
            let backend = self
                .find_backend(&key.backend)
                .unwrap_or_else(|| Arc::new(NullBackend));
            let channels = self.channels_for(key);
            let t = &self.settings.tuning;
            let config = StreamConfig {
                sample_rate: self.settings.sample_rate,
                buffer_frames: self.settings.buffer_frames,
                channels,
                exclusive: self.settings.bit_perfect.contains(key),
            };
            let mixer = MixerConfig {
                volume_smoothing_frames: self.settings.frames(t.gain_smoothing_ms).max(1) as u32,
                max_commands_per_block: t.max_commands_per_block,
            };
            let timing = BusTiming {
                watchdog_timeout: Duration::from_secs_f64(t.watchdog_timeout_ms / 1000.0),
                reconnect_interval: Duration::from_secs_f64(t.reconnect_interval_ms / 1000.0),
                startup_grace: Duration::from_secs_f64(t.watchdog_startup_grace_ms / 1000.0),
            };
            let bus = Bus::open(key.clone(), backend, config, 8, mixer, timing, now);
            bus.shared()
                .true_peak
                .store(self.true_peak, std::sync::atomic::Ordering::Release);
            store_integration(bus.shared(), self.integration);
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
        let carts = self.cart_sources_on(key);
        let wanted = ((mains * 5 + cues * 2 + carts + 2) as f64
            * self.settings.tuning.mixer_headroom)
            .ceil() as usize;
        if let Some(bus) = self.buses.get_mut(key) {
            bus.ensure_capacity(wanted.max(8));
        }
    }

    /// Executes one action at time `now`.
    pub fn execute(&mut self, action: EngineAction, now: Instant) {
        self.now = now;
        match action {
            EngineAction::StartCart(request) => self.start_cart(&request, now),
            EngineAction::StopCart { cart } => self.stop_cart(cart),
            EngineAction::StartCartCue(request) => self.start_cart_cue(&request, now),
            EngineAction::StopCartCue => self.stop_cart_cue(),
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
                if let Some(rt) = self.players.get_mut(&player) {
                    rt.cue_paused = false;
                }
                if let Some(cue) = self
                    .players
                    .get_mut(&player)
                    .and_then(|rt| rt.cue_src.take())
                {
                    self.stop_quick_and_release(player, cue);
                }
            }
            EngineAction::SeekCue { player, secs } => self.seek_cue(player, secs),
            EngineAction::SetCuePaused { player, paused } => self.set_cue_paused(player, paused),
            EngineAction::LoadPaused { player, request } => {
                if let Ok(p) = self.new_source(player, false, &request) {
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
                cue_paused: false,
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
    ) -> Result<Playing, AttachError> {
        let rt = self.players.get(&player).ok_or(AttachError::NoPlayer)?;
        let (bus_key, channel) = if cue {
            rt.cue.clone().ok_or(AttachError::NoRoute)?
        } else {
            rt.main.clone()
        };
        let volume = if cue {
            rt.cue_volume.clone()
        } else {
            rt.volume.clone()
        };
        // A bus running faster than configured needs more frames for the
        // same seconds of buffer.
        let rate = self.rate_of(&bus_key).max(self.settings.sample_rate);
        let ring = ((self.settings.tuning.prebuffer_secs * f64::from(rate)) as usize).max(1024);
        let bus = self.buses.get_mut(&bus_key).ok_or(AttachError::NoRoute)?;
        let Some(slot) = bus.alloc_slot() else {
            self.slot_exhaustions += 1;
            tracing::error!(?player, bus = ?bus_key, "no free mixer slot");
            return Err(AttachError::NoSlot);
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
            return Err(AttachError::QueueFull);
        }
        self.next_key += 1;
        let key = SourceKey(self.next_key);
        let rate = self.rate_of(&bus_key);
        if let Some(rt) = self.players.get(&player) {
            rt.worker
                .load_at(key, request.path.clone(), request.from_secs, producer, rate);
        }
        self.owners.insert(key, player);
        Ok(Playing {
            key,
            bus: bus_key,
            slot,
            request: request.clone(),
            entry: request.entry,
            start_secs: request.from_secs,
            shared,
            start: StartState::Idle,
            report_end: false,
            failed: false,
            cue,
            held: false,
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

    /// Fades a playing source out over `frames` from now and stops it; it is
    /// released when the mixer reports it finished. A source that is not
    /// audible (never started, or paused) is released at once. Returns
    /// whether the source was queued to fade (and will finish later).
    fn fade_out(&mut self, player: PlayerId, p: Playing, ms: f64, curve: Curve) -> bool {
        let frames = self.frames_on(&p.bus, ms);
        let paused = !p.cue && self.players.get(&player).is_some_and(|rt| rt.paused);
        if p.start != StartState::Started || paused || p.held {
            self.send(&p.bus, BusCommand::Cancel { slot: p.slot });
            self.release(p);
            return false;
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
        match self.players.get_mut(&player) {
            Some(rt) => {
                rt.outgoing.push(p);
                true
            }
            None => {
                self.release(p);
                false
            }
        }
    }

    /// Stops a source with a de-click ramp; nothing is reported for it.
    fn stop_quick_and_release(&mut self, player: PlayerId, p: Playing) {
        self.fade_out(player, p, self.settings.tuning.declick_ms, Curve::Linear);
    }

    /// Takes back a transition that was sent to the mixer but has not
    /// happened yet. If the mixer already executed it (its frame has passed,
    /// but this thread has not seen the `Started` event), it is committed
    /// instead: the engine follows what is audible. Returns `true` if a
    /// transition was committed.
    fn undispatch(&mut self, player: PlayerId) -> bool {
        let Some(rt) = self.players.get(&player) else {
            return false;
        };
        let Plan::Dispatched(plan, at_frame) = rt.plan else {
            return false;
        };
        // `now_frame` is the next frame the device will play: a transition
        // at exactly that frame has not happened yet.
        // The transition happened only if the next was sent its `Start`: a
        // next still waiting for its file has not started, whatever the frame.
        let sent = rt
            .preload
            .as_ref()
            .is_some_and(|p| matches!(p.start, StartState::Requested | StartState::Started));
        let executed = sent
            && rt
                .current
                .as_ref()
                .is_some_and(|c| self.now_frame(&c.bus) > at_frame);
        if executed && matches!(plan, TransitionPlan::StartNextAt { .. }) && rt.preload.is_some() {
            self.promote(player);
            return true;
        }
        let Some(rt) = self.players.get_mut(&player) else {
            return false;
        };
        rt.plan = Plan::Waiting(plan);
        let mut cancels = Vec::new();
        if let Some(c) = rt.current.as_ref() {
            cancels.push((c.bus.clone(), c.slot));
        }
        if let Some(p) = rt.preload.as_mut() {
            match p.start {
                StartState::Requested => {
                    p.start = StartState::Idle;
                    cancels.push((p.bus.clone(), p.slot));
                }
                // Waiting for its file: it was sent nothing, and must not
                // start by itself once the file is read.
                StartState::WhenReady { .. } => p.start = StartState::Idle,
                StartState::Idle | StartState::Started => {}
            }
        }
        for (bus, slot) in cancels {
            self.send(&bus, BusCommand::Cancel { slot });
        }
        false
    }

    /// The preload started as the dispatched transition said: it becomes the
    /// current source and the old one is outgoing. Reports `TransitionStarted`.
    fn promote(&mut self, player: PlayerId) {
        let Some(rt) = self.players.get_mut(&player) else {
            return;
        };
        let Some(mut next) = rt.preload.take() else {
            return;
        };
        next.start = StartState::Started;
        let entry = next.entry;
        let overlapping = matches!(
            rt.plan,
            Plan::Dispatched(
                TransitionPlan::StartNextAt {
                    fade_current_until_secs: Some(_),
                    ..
                },
                _
            )
        );
        rt.plan = Plan::None;
        // The audio moved on: whatever was requested meanwhile, the next is
        // playing, and the model follows the engine.
        rt.paused = false;
        if let Some(old) = rt.current.replace(next) {
            rt.outgoing.push(old);
        }
        rt.notify_fade |= overlapping;
        self.events
            .push(EngineEvent::TransitionStarted { player, entry });
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
            // A preload that cannot attach is simply absent; the transition
            // then ends the current track instead (see `dispatch`).
            let p = self.new_source(player, false, &req).ok();
            if let Some(rt) = self.players.get_mut(&player) {
                rt.preload = p;
            }
        }
    }

    /// Takes the preload if it matches `request`, otherwise opens a new source.
    fn take_or_open(
        &mut self,
        player: PlayerId,
        request: &SourceRequest,
    ) -> Result<Playing, AttachError> {
        let rt = self.players.get_mut(&player).ok_or(AttachError::NoPlayer)?;
        let matches = rt
            .preload
            .as_ref()
            .is_some_and(|p| p.entry == request.entry && p.start_secs == request.from_secs);
        if matches && let Some(p) = rt.preload.take() {
            return Ok(p);
        }
        self.new_source(player, false, request)
    }

    fn start_current(
        &mut self,
        player: PlayerId,
        request: &SourceRequest,
        crossfade_ms: Option<u32>,
    ) {
        let reported = self.events.len();
        let committed = self.undispatch(player);
        let already = self
            .players
            .get(&player)
            .and_then(|rt| rt.current.as_ref())
            .is_some_and(|c| c.entry == request.entry && c.start == StartState::Started);
        if committed && already {
            // The mixer already started exactly this entry.
            return;
        }
        if committed {
            // The mixer started something else (a repeating entry's next
            // pass) just before this command, which the model gave after
            // choosing `request`: that start is replaced right now, so the
            // model must not follow it.
            let mut i = reported;
            while i < self.events.len() {
                if matches!(
                    self.events.get(i),
                    Some(EngineEvent::TransitionStarted { player: p, .. }) if *p == player
                ) {
                    self.events.remove(i);
                } else {
                    i += 1;
                }
            }
        }
        if let Some(main) = self.players.get(&player).map(|rt| rt.main.0.clone()) {
            self.prepare_start(&main, request.format);
        }
        let mut next = match self.take_or_open(player, request) {
            Ok(next) => next,
            Err(e) => {
                // An engine limitation, not a bad file: end the entry cleanly.
                tracing::error!(?player, error = ?e, "cannot start a source");
                self.events.push(EngineEvent::ReachedEnd {
                    player,
                    entry: request.entry,
                });
                if crossfade_ms.is_some() {
                    self.events.push(EngineEvent::FadeCompleted { player });
                }
                return;
            }
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
        let queued = match (old, crossfade_ms) {
            (Some(old), Some(ms)) => self.fade_out(player, old, f64::from(ms), Curve::EqualPower),
            (Some(old), None) => {
                self.stop_quick_and_release(player, old);
                false
            }
            (None, _) => false,
        };
        if crossfade_ms.is_some() {
            if queued {
                if let Some(rt) = self.players.get_mut(&player) {
                    rt.notify_fade = true;
                }
            } else {
                // Nothing audible to fade: the fade is complete right away.
                self.events.push(EngineEvent::FadeCompleted { player });
            }
        }
    }

    fn fade_out_and_stop(&mut self, player: PlayerId, fade_ms: u32) {
        self.undispatch(player);
        let fade = f64::from(fade_ms);
        let Some(rt) = self.players.get_mut(&player) else {
            return;
        };
        rt.plan = Plan::None;
        let current = rt.current.take();
        let outgoing: Vec<Playing> = std::mem::take(&mut rt.outgoing);
        for p in outgoing {
            self.fade_out(player, p, fade, Curve::EqualPower);
        }
        if let Some(mut p) = current {
            let entry = p.entry;
            p.report_end = true;
            if !self.fade_out(player, p, fade, Curve::EqualPower) {
                self.events.push(EngineEvent::ReachedEnd { player, entry });
            }
        }
    }

    /// The pause/resume ramp in frames of `bus`.
    fn ramp_frames(&self, bus: &BusKey) -> u32 {
        u32::try_from(self.frames_on(bus, self.settings.tuning.pause_ramp_ms)).unwrap_or(u32::MAX)
    }

    fn pause(&mut self, player: PlayerId) {
        if self.undispatch(player) {
            // The transition already happened in the audio: the next is on
            // air and the model will follow it (`TransitionStarted`).
            return;
        }
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
            let ramp = self.ramp_frames(&bus);
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
        // A track loaded paused (after a restart) starts now: its rate may
        // be followed like any other start.
        let waiting = self
            .players
            .get(&player)
            .and_then(|rt| rt.current.as_ref())
            .filter(|c| c.start == StartState::Idle)
            .map(|c| (c.bus.clone(), c.request.format));
        if let Some((bus, format)) = waiting {
            self.prepare_start(&bus, format);
        }
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
            let ramp = self.ramp_frames(&bus);
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
        rt.notify_fade = false;
        let all: Vec<Playing> = rt
            .current
            .take()
            .into_iter()
            .chain(rt.outgoing.drain(..))
            .collect();
        // `fade_out` releases paused sources at once (they are silent).
        for p in all {
            self.stop_quick_and_release(player, p);
        }
        if let Some(rt) = self.players.get_mut(&player) {
            rt.paused = false;
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
        let Ok(mut next) = self.new_source(player, false, &request) else {
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
            self.stop_quick_and_release(player, old);
        }
    }

    fn start_cue(&mut self, player: PlayerId, request: &SourceRequest) {
        if let Some(rt) = self.players.get_mut(&player) {
            rt.cue_paused = false;
        }
        if let Some(old) = self
            .players
            .get_mut(&player)
            .and_then(|rt| rt.cue_src.take())
        {
            self.stop_quick_and_release(player, old);
        }
        if let Some(cue) = self
            .players
            .get(&player)
            .and_then(|rt| rt.cue.as_ref())
            .map(|c| c.0.clone())
        {
            self.prepare_start(&cue, request.format);
        }
        match self.new_source(player, true, request) {
            Ok(mut cue) => {
                cue.start = StartState::WhenReady { fade_in: false };
                if let Some(rt) = self.players.get_mut(&player) {
                    rt.cue_src = Some(cue);
                }
            }
            // No Cue output (or no slot): nothing can be heard, end it at once.
            Err(_) => self.events.push(EngineEvent::CueEnded {
                player,
                entry: request.entry,
            }),
        }
    }

    /// Replaces the CUE source by one at `secs` (spec O12). A held CUE stays
    /// held: the new source waits idle until it is released. Without a CUE
    /// source nothing happens.
    fn seek_cue(&mut self, player: PlayerId, secs: f64) {
        let Some(mut request) = self
            .players
            .get(&player)
            .and_then(|rt| rt.cue_src.as_ref())
            .map(|c| c.request.clone())
        else {
            return;
        };
        request.from_secs = secs;
        let Ok(mut next) = self.new_source(player, true, &request) else {
            return;
        };
        let Some(rt) = self.players.get_mut(&player) else {
            return;
        };
        next.start = if rt.cue_paused {
            StartState::Idle
        } else {
            StartState::WhenReady { fade_in: false }
        };
        if let Some(old) = rt.cue_src.replace(next) {
            self.stop_quick_and_release(player, old);
        }
    }

    /// Holds or releases the CUE source (spec O12). A source that has not
    /// started yet is held back idle; one that is audible ramps down with
    /// the pause ramp. Without a CUE source only the flag changes.
    fn set_cue_paused(&mut self, player: PlayerId, paused: bool) {
        let Some(rt) = self.players.get_mut(&player) else {
            return;
        };
        rt.cue_paused = paused;
        let Some(cue) = rt.cue_src.as_mut() else {
            return;
        };
        cue.held = paused;
        let (bus, slot) = (cue.bus.clone(), cue.slot);
        let send = match (paused, cue.start) {
            (true, StartState::WhenReady { .. }) => {
                cue.start = StartState::Idle;
                None
            }
            (false, StartState::Idle) => {
                cue.start = StartState::WhenReady { fade_in: false };
                None
            }
            (pause, _) => Some(pause),
        };
        let Some(pause) = send else {
            return;
        };
        let ramp_frames = self.ramp_frames(&bus);
        let command = if pause {
            BusCommand::Pause { slot, ramp_frames }
        } else {
            BusCommand::Resume { slot, ramp_frames }
        };
        self.send(&bus, command);
    }
}

impl Engine {
    /// Advances the engine: supervises devices, turns bus and worker
    /// observations into model events, starts sources that became ready and
    /// dispatches transitions that are due. Returns the events for the model.
    pub fn tick(&mut self, now: Instant) -> Vec<EngineEvent> {
        self.now = now;
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
            if self.cart_failure(&failure) {
                continue;
            }
            let Some(player) = self.owners.get(&failure.key).copied() else {
                continue;
            };
            tracing::warn!(?player, error = %failure.error, "source failed");
            let Some(rt) = self.players.get_mut(&player) else {
                continue;
            };
            // A current source that is playing (or about to) keeps playing what
            // is buffered; the mixer finishes it when the ring drains and the
            // failure is reported then (spec §4.5). A failure before anything
            // was buffered finishes at once the same way.
            if let Some(c) = rt.current.as_mut()
                && c.key == failure.key
                && c.start != StartState::Idle
                && !c.shared.is_drained()
            {
                c.failed = true;
                continue;
            }
            let mut preload = false;
            let taken = if rt.current.as_ref().is_some_and(|p| p.key == failure.key) {
                rt.plan = Plan::None;
                rt.current.take()
            } else if rt.preload.as_ref().is_some_and(|p| p.key == failure.key) {
                preload = true;
                rt.preload.take()
            } else if rt.cue_src.as_ref().is_some_and(|p| p.key == failure.key) {
                rt.cue_src.take()
            } else {
                None
            };
            if let Some(p) = taken {
                let entry = p.entry;
                self.release(p);
                // A preload that fails says nothing about what is on air,
                // even when it is the same entry (a repeat).
                self.events.push(if preload {
                    EngineEvent::PreloadFailed { player, entry }
                } else {
                    EngineEvent::SourceFailed { player, entry }
                });
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
        if let BusEvent::Finished { slot, .. } = event
            && let Some(i) = self.tones.iter().position(|(b, s)| b == bus && *s == slot)
        {
            let (bus, slot) = self.tones.remove(i);
            self.send(&bus, BusCommand::Detach { slot });
            return;
        }
        if self.cart_bus_event(bus, &event) {
            return;
        }
        match event {
            BusEvent::Started { slot, .. } => {
                let Some((player, role)) = self.find(bus, slot) else {
                    return;
                };
                let Some(rt) = self.players.get_mut(&player) else {
                    return;
                };
                match role {
                    Role::Preload => self.promote(player),
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
                            let entry = p.entry;
                            self.release(p);
                            self.events.push(EngineEvent::CueEnded { player, entry });
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

    /// The current source stopped by itself (end of stream, a planned stop,
    /// or the drained end of a source whose worker failed).
    fn current_finished(&mut self, player: PlayerId) {
        let Some(rt) = self.players.get_mut(&player) else {
            return;
        };
        let Some(finished) = rt.current.take() else {
            return;
        };
        let plan = match rt.plan {
            Plan::Waiting(p) | Plan::Dispatched(p, _) => Some(p),
            Plan::None => None,
        };
        let (entry, failed) = (finished.entry, finished.failed);
        self.release(finished);
        if failed {
            // Everything buffered has been heard; now the model decides (skip
            // in Continuous mode, stop otherwise).
            if let Some(rt) = self.players.get_mut(&player) {
                rt.plan = Plan::None;
            }
            self.events
                .push(EngineEvent::SourceFailed { player, entry });
            return;
        }
        let next = match plan {
            Some(TransitionPlan::StartNextAt { .. }) => self
                .players
                .get(&player)
                .and_then(|rt| rt.preload.as_ref())
                .filter(|p| p.start != StartState::Started)
                .map(|p| {
                    // A `Requested` start was sent with its ramp already; the
                    // mixer ignores a second `Start` of a started slot, so a
                    // second ramp would only step the gain.
                    let ramp = p.start_secs > 0.0 && p.start != StartState::Requested;
                    (p.bus.clone(), p.slot, p.shared.is_ready(), ramp)
                }),
            _ => None,
        };
        match next {
            // End of stream before (or instead of) the scheduled transition:
            // start the next right now, or as soon as it is buffered;
            // `promote` runs on its `Started` event.
            Some((bus, slot, ready, inside)) => {
                let at = self.now_frame(&bus);
                if ready {
                    self.send_start(&bus, slot, at, inside);
                }
                if let Some(rt) = self.players.get_mut(&player) {
                    rt.plan = Plan::Dispatched(
                        TransitionPlan::StartNextAt {
                            at_secs: 0.0,
                            fade_current_until_secs: None,
                        },
                        at,
                    );
                    if let Some(p) = rt.preload.as_mut() {
                        p.start = if ready {
                            StartState::Requested
                        } else {
                            StartState::WhenReady { fade_in: false }
                        };
                    }
                }
            }
            None => {
                if let Some(rt) = self.players.get_mut(&player) {
                    rt.plan = Plan::None;
                }
                self.events.push(EngineEvent::ReachedEnd { player, entry });
            }
        }
    }

    fn start_ready_sources(&mut self) {
        /// Queues the start of `p` if it waits to be ready and now is.
        fn take_ready(
            p: &mut Playing,
            earliest: u64,
            starts: &mut Vec<(BusKey, usize, bool, u64)>,
        ) {
            if let StartState::WhenReady { fade_in } = p.start
                // Also set when the worker failed: whatever it buffered still plays.
                && p.shared.is_ready()
                // A failed source with nothing buffered has nothing to play:
                // it waits for its failure to be handled instead.
                && !(p.shared.is_failed() && p.shared.is_drained())
            {
                p.start = StartState::Requested;
                // A start inside the audio ramps in; the file's first frame is hard.
                starts.push((
                    p.bus.clone(),
                    p.slot,
                    fade_in || p.start_secs > 0.0,
                    earliest,
                ));
            }
        }
        self.start_ready_carts();
        // (bus, slot, fade in, earliest frame).
        let mut starts: Vec<(BusKey, usize, bool, u64)> = Vec::new();
        for rt in self.players.values_mut() {
            if rt.paused {
                for p in rt.cue_src.iter_mut() {
                    take_ready(p, 0, &mut starts);
                }
                continue;
            }
            // A preload only waits to start after a dispatched transition,
            // and never before the frame it was dispatched for.
            let dispatched = match rt.plan {
                Plan::Dispatched(TransitionPlan::StartNextAt { .. }, at_frame) => Some(at_frame),
                _ => None,
            };
            for p in rt.current.iter_mut().chain(rt.cue_src.iter_mut()) {
                take_ready(p, 0, &mut starts);
            }
            if let Some(p) = rt.preload.as_mut() {
                match dispatched {
                    Some(frame) => take_ready(p, frame, &mut starts),
                    // Its transition is gone (taken back, or the current
                    // source failed): it goes back to waiting idle.
                    None if matches!(p.start, StartState::WhenReady { .. }) => {
                        p.start = StartState::Idle;
                    }
                    None => {}
                }
            }
        }
        for (bus, slot, fade_in, earliest) in starts {
            let now = self.now_frame(&bus).max(earliest);
            self.send_start(&bus, slot, now, fade_in);
        }
    }

    /// Sends the `Start` of `slot` at frame `now`, with a de-click ramp in
    /// from silence when `fade_in`.
    pub(super) fn send_start(&mut self, bus: &BusKey, slot: usize, now: u64, fade_in: bool) {
        let declick =
            u32::try_from(self.frames_on(bus, self.settings.tuning.declick_ms)).unwrap_or(u32::MAX);
        if fade_in {
            self.send(
                bus,
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
            bus,
            BusCommand::Start {
                slot,
                at_frame: now,
            },
        );
        if fade_in {
            self.send(
                bus,
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

    /// Sends due transitions to the mixer with their exact frame.
    fn dispatch_plans(&mut self) {
        let mut due: Vec<(PlayerId, TransitionPlan, u64, u64)> = Vec::new();
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
            let rate = f64::from(self.rate_of(&current.bus));
            let lead = self.frames_on(&current.bus, self.settings.tuning.schedule_lead_ms);
            let declick = self.frames_on(&current.bus, self.settings.tuning.declick_ms);
            let position = current.start_secs + played as f64 / rate;
            let until = ((target - position) * rate).round().max(0.0) as u64;
            if until <= lead {
                due.push((*id, plan, now + until, declick));
            }
        }
        for (player, plan, at_frame, declick) in due {
            self.dispatch(player, plan, at_frame, declick);
        }
    }

    fn dispatch(&mut self, player: PlayerId, plan: TransitionPlan, at_frame: u64, declick: u64) {
        let rate = self
            .players
            .get(&player)
            .and_then(|rt| rt.current.as_ref())
            .map_or(self.settings.sample_rate, |c| self.rate_of(&c.bus));
        let rate = f64::from(rate);
        let Some(rt) = self.players.get_mut(&player) else {
            return;
        };
        rt.plan = Plan::Dispatched(plan, at_frame);
        let Some(current) = rt.current.as_ref() else {
            return;
        };
        let (cur_bus, cur_slot) = (current.bus.clone(), current.slot);
        let next = match plan {
            TransitionPlan::StartNextAt { .. } => rt.preload.as_mut().map(|p| {
                // A next that is not buffered yet starts as soon as it is,
                // instead of on the frame as silence counted as underruns.
                let ready = p.shared.is_ready();
                p.start = if ready {
                    StartState::Requested
                } else {
                    StartState::WhenReady { fade_in: false }
                };
                (p.bus.clone(), p.slot, ready, p.start_secs > 0.0)
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
                Some((bus, slot, ready, inside)),
            ) => {
                if ready {
                    self.send_start(&bus, slot, at_frame, inside);
                }
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
