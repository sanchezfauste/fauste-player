//! The conductor thread (spec §2.2, §4.4): the single owner of the model
//! state. It applies UI commands and engine events through the pure model
//! reducer, executes the resulting actions on the engine, and publishes
//! immutable snapshots. The UI never locks anything the audio path waits on.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use arc_swap::ArcSwap;
use crossbeam_channel::{Receiver, Sender, TrySendError};
use fp_model::{
    AppState, CartId, Command, EngineAction, ModelError, PlayerId, Route, apply, on_event,
};

use crate::engine::{BusStatus, CartTelemetry, Engine, PlayerTelemetry};
use crate::meter::MeterState;

/// Live values for the UI, refreshed every tick.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Telemetry {
    pub players: Vec<(PlayerId, PlayerTelemetry)>,
    pub buses: Vec<BusStatus>,
    /// Increases every time the model snapshot changes (drives saving).
    pub model_version: u64,
    pub dropped_commands: u64,
    /// Sources refused for lack of a mixer slot (should always be zero).
    pub slot_exhaustions: u64,
    /// Carts on air, in firing order.
    pub carts: Vec<(CartId, CartTelemetry)>,
    /// The cart pre-listening and its position.
    pub cart_cue: Option<(CartId, f64)>,
}

/// Test tone parameters (spec §8.4): 1.5 s at −18 dBFS.
const TEST_TONE_SECS: f32 = 1.5;
const TEST_TONE_DB: f32 = -18.0;

/// Requests for the engine that are not model commands.
#[derive(Debug, Clone, PartialEq)]
pub enum EngineRequest {
    TestTone { route: Route, frequency_hz: f32 },
}

pub struct Conductor {
    state: AppState,
    engine: Engine,
    commands: Receiver<Command>,
    requests: Receiver<EngineRequest>,
    rejected: Sender<ModelError>,
    model: Arc<ArcSwap<AppState>>,
    telemetry: Arc<ArcSwap<Telemetry>>,
    version: u64,
    /// One meter per player (meters spec M2), and when they last moved.
    meters: HashMap<PlayerId, MeterState>,
    metered_at: Option<Instant>,
}

/// The UI's side of the conductor.
pub struct ConductorHandle {
    commands: Sender<Command>,
    requests: Sender<EngineRequest>,
    pub model: Arc<ArcSwap<AppState>>,
    pub telemetry: Arc<ArcSwap<Telemetry>>,
    /// Commands the model refused, with the reason (for a UI notice).
    pub rejected: Receiver<ModelError>,
    stop: Option<Arc<AtomicBool>>,
    thread: Option<JoinHandle<()>>,
}

impl ConductorHandle {
    /// Queues a command; never blocks. `false` if the queue is full.
    pub fn send(&self, command: Command) -> bool {
        match self.commands.try_send(command) {
            Ok(()) => true,
            Err(TrySendError::Full(c)) => {
                tracing::warn!(command = ?c, "command queue full; command dropped");
                false
            }
            Err(TrySendError::Disconnected(_)) => false,
        }
    }

    /// Plays a test tone on `route`; never blocks. `false` if the queue is full.
    pub fn test_tone(&self, route: Route, frequency_hz: f32) -> bool {
        self.requests
            .try_send(EngineRequest::TestTone {
                route,
                frequency_hz,
            })
            .is_ok()
    }
}

impl Drop for ConductorHandle {
    fn drop(&mut self) {
        if let Some(stop) = &self.stop {
            stop.store(true, Ordering::Release);
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Conductor {
    /// Takes over `state` (typically restored from disk) and runs the
    /// `initial` actions that came with it (e.g. `LoadPaused`).
    pub fn new(
        state: AppState,
        initial: Vec<EngineAction>,
        mut engine: Engine,
        now: Instant,
    ) -> (Conductor, ConductorHandle) {
        for player in &state.players {
            engine.execute(EngineAction::AddPlayer { player: player.id }, now);
        }
        for action in initial {
            engine.execute(action, now);
        }
        let (tx, rx) = crossbeam_channel::bounded(1024);
        let (req_tx, req_rx) = crossbeam_channel::bounded(16);
        let (rejected_tx, rejected_rx) = crossbeam_channel::bounded(64);
        let model = Arc::new(ArcSwap::from_pointee(state.clone()));
        let telemetry = Arc::new(ArcSwap::from_pointee(Telemetry::default()));
        let conductor = Conductor {
            state,
            engine,
            commands: rx,
            requests: req_rx,
            rejected: rejected_tx,
            model: model.clone(),
            telemetry: telemetry.clone(),
            version: 0,
            meters: HashMap::new(),
            metered_at: None,
        };
        let handle = ConductorHandle {
            commands: tx,
            requests: req_tx,
            model,
            telemetry,
            rejected: rejected_rx,
            stop: None,
            thread: None,
        };
        (conductor, handle)
    }

    pub fn state(&self) -> &AppState {
        &self.state
    }

    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    /// One iteration: commands, engine, events, snapshots.
    pub fn tick(&mut self, now: Instant) {
        let mut changed = false;
        let commands: Vec<Command> = self.commands.try_iter().collect();
        for command in commands {
            match apply(&mut self.state, command) {
                Ok(actions) => {
                    changed = true;
                    for action in actions {
                        self.engine.execute(action, now);
                    }
                }
                Err(e) => {
                    let _ = self.rejected.try_send(e);
                }
            }
        }
        // One request per tick keeps the tick short; the rest wait their turn.
        if let Ok(request) = self.requests.try_recv() {
            match request {
                EngineRequest::TestTone {
                    route,
                    frequency_hz,
                } => {
                    self.engine.play_test_tone(
                        &route,
                        frequency_hz,
                        TEST_TONE_SECS,
                        TEST_TONE_DB,
                        now,
                    );
                }
            }
        }
        for event in self.engine.tick(now) {
            changed = true;
            for action in on_event(&mut self.state, event) {
                self.engine.execute(action, now);
            }
        }
        if changed {
            self.version += 1;
            self.model.store(Arc::new(self.state.clone()));
        }
        if self.engine.true_peak() != self.state.config.meter.true_peak {
            self.engine.set_true_peak(self.state.config.meter.true_peak);
        }
        let dt = self
            .metered_at
            .map_or(0.0, |at| now.saturating_duration_since(at).as_secs_f64());
        self.metered_at = Some(now);
        let ids: Vec<PlayerId> = self.state.players.iter().map(|p| p.id).collect();
        self.meters.retain(|id, _| ids.contains(id));
        let players = ids
            .into_iter()
            .map(|id| {
                let mut t = self.engine.telemetry(id);
                let input = self.engine.take_meter_input(id);
                t.meter =
                    self.meters
                        .entry(id)
                        .or_default()
                        .update(input, dt, &self.state.config.meter);
                (id, t)
            })
            .collect();
        let (carts, cart_cue) = self.engine.cart_telemetry();
        self.telemetry.store(Arc::new(Telemetry {
            carts,
            cart_cue,
            players,
            buses: self.engine.bus_status(),
            model_version: self.version,
            dropped_commands: self.engine.dropped_commands(),
            slot_exhaustions: self.engine.slot_exhaustions(),
        }));
    }

    /// Runs the conductor on its own thread, ticking every `period`.
    pub fn spawn(
        mut self,
        mut handle: ConductorHandle,
        period: Duration,
    ) -> std::io::Result<ConductorHandle> {
        let stop = Arc::new(AtomicBool::new(false));
        let flag = stop.clone();
        let thread = std::thread::Builder::new()
            .name("fp-conductor".to_owned())
            .spawn(move || {
                while !flag.load(Ordering::Acquire) {
                    let started = Instant::now();
                    self.tick(started);
                    if let Some(rest) = period.checked_sub(started.elapsed()) {
                        std::thread::sleep(rest);
                    }
                }
            })?;
        handle.stop = Some(stop);
        handle.thread = Some(thread);
        Ok(handle)
    }
}
