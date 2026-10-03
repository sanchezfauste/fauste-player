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
    AppState, CartId, Command, EngineAction, EngineEvent, EntryId, ModelError, PlayerId, Route,
    apply, on_event,
};

use crate::bus::BusKey;
use crate::engine::{BusStatus, CartTelemetry, Engine, PlayerTelemetry};
use crate::meter::MeterState;
pub use crate::reporting::REPORT_WINDOW;
use crate::reporting::{Increase, Watch};

/// A counter of the real-time side that the conductor watches (audit A8).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Counted {
    Xruns(BusKey),
    StreamErrors(BusKey),
    LockMisses(BusKey),
    Leaked(BusKey),
    DroppedEvents(BusKey),
    Misrouted(BusKey),
    Underruns(PlayerId),
}

fn log_increase(counted: &Counted, Increase { new, total }: Increase) {
    match counted {
        Counted::Xruns(bus) => {
            tracing::warn!(
                ?bus,
                new,
                total,
                "output xruns: the device missed a deadline"
            );
        }
        Counted::StreamErrors(bus) => {
            tracing::warn!(?bus, new, total, "stream error reported by the backend");
        }
        Counted::LockMisses(bus) => {
            tracing::warn!(
                ?bus,
                new,
                total,
                "blocks output as silence: the mixer was busy"
            );
        }
        Counted::Leaked(bus) => {
            tracing::error!(?bus, new, total, "items leaked by the real-time thread");
        }
        Counted::DroppedEvents(bus) => {
            tracing::warn!(?bus, new, total, "events dropped by the real-time thread");
        }
        Counted::Misrouted(bus) => {
            tracing::warn!(
                ?bus,
                new,
                total,
                "blocks misrouted: the channels did not fit the stream"
            );
        }
        Counted::Underruns(player) => {
            tracing::warn!(
                ?player,
                new,
                total,
                "source underruns: decoding did not keep up"
            );
        }
    }
}

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
    TestTone {
        route: Route,
        frequency_hz: f32,
    },
    /// Restarts the maximum of a player's meter (meters spec M2).
    ResetMeterMax(PlayerId),
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
    /// The entry each player's meter maximum belongs to.
    metered_entries: HashMap<PlayerId, EntryId>,
    /// Players that started an entry since the last metering: each start
    /// restarts the maximum, even of the same entry.
    started: Vec<PlayerId>,
    /// The counters of the real-time side, for the log (audit A8).
    watches: HashMap<Counted, Watch>,
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

/// Characters of a command kept in a log line.
const LOGGED_COMMAND_CHARS: usize = 120;

/// The start of `command`'s debug form, so a dropped command with a huge
/// payload (thousands of paths, a whole configuration) logs one short line.
fn command_summary(command: &Command) -> String {
    let full = format!("{command:?}");
    match full.char_indices().nth(LOGGED_COMMAND_CHARS) {
        Some((cut, _)) => format!("{}…", full.get(..cut).unwrap_or_default()),
        None => full,
    }
}

impl ConductorHandle {
    /// Queues a command; never blocks. `false` if the queue is full.
    pub fn send(&self, command: Command) -> bool {
        match self.commands.try_send(command) {
            Ok(()) => true,
            Err(TrySendError::Full(c)) => {
                tracing::warn!(
                    command = %command_summary(&c),
                    "command queue full; command dropped"
                );
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

impl ConductorHandle {
    /// Restarts the maximum of `player`'s meter; never blocks. `false` if
    /// the queue is full.
    pub fn reset_meter_max(&self, player: PlayerId) -> bool {
        self.requests
            .try_send(EngineRequest::ResetMeterMax(player))
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
            metered_entries: HashMap::new(),
            started: Vec::new(),
            watches: HashMap::new(),
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

    /// Runs `action` on the engine, noting the players it starts.
    fn execute(&mut self, action: EngineAction, now: Instant) {
        if let EngineAction::StartCurrent { player, .. } | EngineAction::Crossfade { player, .. } =
            action
        {
            self.started.push(player);
        }
        self.engine.execute(action, now);
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
                        self.execute(action, now);
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
                EngineRequest::ResetMeterMax(player) => {
                    if let Some(meter) = self.meters.get_mut(&player) {
                        meter.reset_max();
                    }
                }
            }
        }
        for event in self.engine.tick(now) {
            changed = true;
            if let EngineEvent::TransitionStarted { player, .. } = event {
                self.started.push(player);
            }
            for action in on_event(&mut self.state, event) {
                self.execute(action, now);
            }
        }
        if changed {
            self.version += 1;
            self.model.store(Arc::new(self.state.clone()));
        }
        let true_peak = self.state.config.meter.true_peak_in_use();
        if self.engine.true_peak() != true_peak {
            self.engine.set_true_peak(true_peak);
        }
        let integration = crate::meter::mixer_integration(&self.state.config.meter);
        if self.engine.meter_integration() != integration {
            self.engine.set_meter_integration(integration);
        }
        let dt = self
            .metered_at
            .map_or(0.0, |at| now.saturating_duration_since(at).as_secs_f64());
        self.metered_at = Some(now);
        let ids: Vec<PlayerId> = self.state.players.iter().map(|p| p.id).collect();
        self.meters.retain(|id, _| ids.contains(id));
        self.metered_entries.retain(|id, _| ids.contains(id));
        // A new entry, or any start of one (the same entry played again,
        // or segued into itself), restarts the maximum; a stop keeps it on
        // show until then.
        let started = std::mem::take(&mut self.started);
        for player in &self.state.players {
            let Some(entry) = player.current else {
                self.metered_entries.remove(&player.id);
                continue;
            };
            let new_entry = self.metered_entries.insert(player.id, entry) != Some(entry);
            if (new_entry || started.contains(&player.id))
                && let Some(meter) = self.meters.get_mut(&player.id)
            {
                meter.reset_max();
            }
        }
        let players: Vec<(PlayerId, PlayerTelemetry)> = ids
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
        let buses = self.engine.bus_status();
        self.report_counters(&buses, &players, now);
        let (carts, cart_cue) = self.engine.cart_telemetry();
        self.telemetry.store(Arc::new(Telemetry {
            carts,
            cart_cue,
            players,
            buses,
            model_version: self.version,
            dropped_commands: self.engine.dropped_commands(),
            slot_exhaustions: self.engine.slot_exhaustions(),
        }));
    }

    /// Logs the counters that grew since the last tick, at most once per
    /// window each (audit A8).
    fn report_counters(
        &mut self,
        buses: &[BusStatus],
        players: &[(PlayerId, PlayerTelemetry)],
        now: Instant,
    ) {
        let mut readings: Vec<(Counted, u64)> = Vec::new();
        for b in buses {
            let c = b.counters;
            let key = || b.key.clone();
            readings.extend([
                (Counted::Xruns(key()), c.xruns),
                (Counted::StreamErrors(key()), c.stream_errors),
                (Counted::LockMisses(key()), c.lock_misses),
                (Counted::Leaked(key()), c.leaked),
                (Counted::DroppedEvents(key()), c.dropped_events),
                (Counted::Misrouted(key()), c.misrouted),
            ]);
        }
        readings.extend(
            players
                .iter()
                .map(|(id, t)| (Counted::Underruns(*id), t.underruns)),
        );
        self.watches
            .retain(|counted, _| readings.iter().any(|(c, _)| c == counted));
        for (counted, value) in readings {
            let increase = self
                .watches
                .entry(counted.clone())
                .or_default()
                .observe(value, now);
            if let Some(increase) = increase {
                log_increase(&counted, increase);
            }
        }
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

#[cfg(test)]
mod tests {
    use super::command_summary;
    use fp_model::Command;
    use std::path::PathBuf;

    #[test]
    fn a_dropped_command_is_logged_briefly() {
        let paths = (0..10_000)
            .map(|n| PathBuf::from(format!("/m/{n}.flac")))
            .collect();
        let command = Command::InsertPaths {
            playlist: fp_model::PlaylistId(1),
            index: 0,
            paths,
        };
        let summary = command_summary(&command);
        assert!(summary.starts_with("InsertPaths"), "{summary}");
        assert!(summary.chars().count() <= 121, "{} chars", summary.len());
    }
}
