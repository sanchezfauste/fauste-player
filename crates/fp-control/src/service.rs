//! The MIDI service (feedback spec §6): connects the surfaces' ports,
//! rescans them for hot-plug, routes their messages to commands, keeps
//! their LEDs up to date and answers MIDI learn. It runs on its own thread
//! and talks to the rest only through channels and snapshots.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use arc_swap::ArcSwap;
use crossbeam_channel::{Receiver, Sender};
use fp_model::{AppState, Command, MidiAction, MidiTrigger};

use crate::feedback::{Feedback, desired};
use crate::learn::trigger_for;
use crate::message::parse;
use crate::router::Router;

/// What the service needs from the application: the latest model and a way
/// to send commands (the same channel as the interface).
pub trait MidiControl: Send + Sync {
    fn model(&self) -> Arc<AppState>;
    /// Queues a command; never blocks.
    fn send(&self, command: Command) -> bool;
}

/// A connected output port.
pub trait MidiOutput: Send {
    fn send(&mut self, bytes: &[u8]) -> Result<(), String>;
}

/// The system's MIDI ports.
pub trait MidiPorts: Send {
    fn inputs(&mut self) -> Vec<String>;
    fn outputs(&mut self) -> Vec<String>;
    /// Connects an input: its messages arrive on `sink` with the port's
    /// name. Dropping the returned value disconnects it.
    fn connect_input(
        &mut self,
        name: &str,
        sink: Sender<(String, Vec<u8>)>,
    ) -> Result<Box<dyn Send>, String>;
    fn connect_output(&mut self, name: &str) -> Result<Box<dyn MidiOutput>, String>;
}

/// Requests from the interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MidiRequest {
    /// Bind the next suitable message to this action.
    Learn(MidiAction),
    CancelLearn,
}

/// The answer to a learn request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Learned {
    pub action: MidiAction,
    pub device: String,
    pub trigger: MidiTrigger,
}

/// The input ports last seen, and whether each is connected.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MidiStatus {
    pub inputs: Vec<(String, bool)>,
}

/// The interface's side of the service.
#[derive(Clone)]
pub struct MidiHandle {
    pub status: Arc<ArcSwap<MidiStatus>>,
    pub requests: Sender<MidiRequest>,
    pub learned: Receiver<Learned>,
}

/// Raw bytes from an input port, with the port's name.
type Raw = (String, Vec<u8>);

/// Half the UI's blink period (Pause blinks at 500 ms).
const BLINK: Duration = Duration::from_millis(500);

/// The service's state, stepped by its thread (or by tests).
pub struct MidiCore {
    ports: Box<dyn MidiPorts>,
    control: Arc<dyn MidiControl>,
    router: Router,
    feedback: Feedback,
    inputs: HashMap<String, Box<dyn Send>>,
    outputs: HashMap<String, Box<dyn MidiOutput>>,
    messages: (Sender<Raw>, Receiver<Raw>),
    requests: Receiver<MidiRequest>,
    learned: Sender<Learned>,
    status: Arc<ArcSwap<MidiStatus>>,
    learning: Option<MidiAction>,
    next_scan: Option<Instant>,
    started: Option<Instant>,
}

impl MidiCore {
    pub fn new(ports: Box<dyn MidiPorts>, control: Arc<dyn MidiControl>) -> (Self, MidiHandle) {
        let (req_tx, req_rx) = crossbeam_channel::unbounded();
        let (learned_tx, learned_rx) = crossbeam_channel::unbounded();
        let status = Arc::new(ArcSwap::from_pointee(MidiStatus::default()));
        let config = control.model().config.midi.clone();
        let core = Self {
            ports,
            control,
            router: Router::new(&config),
            feedback: Feedback::default(),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            messages: crossbeam_channel::unbounded(),
            requests: req_rx,
            learned: learned_tx,
            status: status.clone(),
            learning: None,
            next_scan: None,
            started: None,
        };
        let handle = MidiHandle {
            status,
            requests: req_tx,
            learned: learned_rx,
        };
        (core, handle)
    }

    /// Waits up to `timeout` until a message or a request is ready,
    /// without taking it (`step` handles it, in order).
    pub fn wait(&self, timeout: Duration) {
        let mut ready = crossbeam_channel::Select::new();
        ready.recv(&self.messages.1);
        ready.recv(&self.requests);
        let _ = ready.ready_timeout(timeout);
    }

    /// One pass: requests, a rescan when due, messages, then LEDs.
    pub fn step(&mut self, now: Instant) {
        let started = *self.started.get_or_insert(now);
        let state = self.control.model();
        let config = &state.config.midi;
        self.router.set_config(config);
        while let Ok(request) = self.requests.try_recv() {
            self.learning = match request {
                MidiRequest::Learn(action) => Some(action),
                MidiRequest::CancelLearn => None,
            };
            // Learning listens to every input: connect them now.
            self.next_scan = None;
        }
        if self.next_scan.is_none_or(|t| now >= t) {
            self.scan(config);
            self.next_scan =
                Some(now + Duration::from_millis(u64::from(config.rescan_interval_ms)));
        }
        while let Ok((device, bytes)) = self.messages.1.try_recv() {
            if !config.enabled {
                continue;
            }
            let Some(msg) = parse(&bytes) else {
                continue;
            };
            if let Some(action) = self.learning {
                if let Some(trigger) = trigger_for(action, msg) {
                    self.learning = None;
                    let _ = self.learned.send(Learned {
                        action,
                        device,
                        trigger,
                    });
                }
                continue;
            }
            if let Some(command) = self.router.on_message(&device, msg, &state)
                && !self.control.send(command)
            {
                tracing::warn!("a MIDI command was dropped: the queue is full");
            }
        }
        if config.enabled {
            let phase = now.duration_since(started).as_millis() / BLINK.as_millis();
            let blink_on = phase.is_multiple_of(2);
            for led in self.feedback.changes(desired(config, &state, blink_on)) {
                if let Some(out) = self.outputs.get_mut(&led.output)
                    && let Err(error) = out.send(&led.bytes)
                {
                    tracing::debug!(%error, port = %led.output, "a MIDI LED was not sent");
                }
            }
        }
    }

    /// Connects the ports that appeared (by name), drops the ones that are
    /// gone, and publishes the list.
    fn scan(&mut self, config: &fp_model::MidiConfig) {
        if !config.enabled {
            // Off: no port is opened or even listed.
            self.inputs.clear();
            self.outputs.clear();
            self.feedback.reset();
            if !self.status.load().inputs.is_empty() {
                self.status.store(Arc::new(MidiStatus::default()));
            }
            return;
        }
        // Never our own ports (each connection is a client of its own).
        let ours = |name: &String| !name.starts_with(crate::ports::CLIENT);
        let inputs: Vec<String> = self.ports.inputs().into_iter().filter(ours).collect();
        let outputs: Vec<String> = self.ports.outputs().into_iter().filter(ours).collect();
        // Only the devices that are bound are opened (ports can be
        // exclusive), every input while learning.
        let bound: Vec<&str> = config.bindings.iter().map(|b| b.device.as_str()).collect();
        let feedback_port = |device: &str| {
            config
                .devices
                .iter()
                .find(|d| d.input == device)
                .and_then(|d| d.output.clone())
                .unwrap_or_else(|| device.to_owned())
        };
        let wanted_outputs: Vec<String> = bound.iter().map(|d| feedback_port(d)).collect();
        let learning = self.learning.is_some();
        let wanted_input = |name: &String| learning || bound.contains(&name.as_str());
        self.inputs
            .retain(|name, _| inputs.contains(name) && wanted_input(name));
        self.outputs
            .retain(|name, _| outputs.contains(name) && wanted_outputs.contains(name));
        let mut reconnected = false;
        {
            for name in inputs.iter().filter(|n| wanted_input(n)) {
                if !self.inputs.contains_key(name) {
                    match self.ports.connect_input(name, self.messages.0.clone()) {
                        Ok(connection) => {
                            tracing::info!(port = %name, "MIDI input connected");
                            self.inputs.insert(name.clone(), connection);
                        }
                        Err(error) => {
                            tracing::warn!(%error, port = %name, "MIDI input not connected")
                        }
                    }
                }
            }
            for name in outputs.iter().filter(|n| wanted_outputs.contains(n)) {
                if !self.outputs.contains_key(name) {
                    match self.ports.connect_output(name) {
                        Ok(out) => {
                            self.outputs.insert(name.clone(), out);
                            reconnected = true;
                        }
                        Err(error) => {
                            tracing::debug!(%error, port = %name, "MIDI output not connected")
                        }
                    }
                }
            }
        }
        if reconnected {
            // Every LED again, now that the surface listens.
            self.feedback.reset();
        }
        let status = MidiStatus {
            inputs: inputs
                .into_iter()
                .map(|name| {
                    let connected = self.inputs.contains_key(&name);
                    (name, connected)
                })
                .collect(),
        };
        if **self.status.load() != status {
            self.status.store(Arc::new(status));
        }
    }
}

/// Starts the service on its own thread, named `fp-midi`.
pub fn spawn(
    ports: Box<dyn MidiPorts>,
    control: Arc<dyn MidiControl>,
) -> std::io::Result<MidiHandle> {
    let (mut core, handle) = MidiCore::new(ports, control);
    std::thread::Builder::new()
        .name("fp-midi".to_owned())
        .spawn(move || {
            loop {
                let step = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    core.step(Instant::now());
                }));
                if step.is_err() {
                    tracing::error!("the MIDI service recovered from a panic");
                }
                core.wait(Duration::from_millis(20));
            }
        })?;
    Ok(handle)
}
