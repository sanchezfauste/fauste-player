//! Connects the MIDI service (`fp-control`) to the conductor.

use std::sync::Arc;

use fp_control::service::{MidiControl, MidiService};
use fp_engine::conductor::ConductorHandle;
use fp_model::{AppState, Command};

/// The conductor as the MIDI service sees it.
struct Bridge(Arc<ConductorHandle>);

impl MidiControl for Bridge {
    fn model(&self) -> Arc<AppState> {
        self.0.model.load_full()
    }

    fn send(&self, command: Command) -> bool {
        self.0.send(command)
    }
}

/// Starts MIDI control on the system's ports. A failure is logged and the
/// application runs without it. The service holds a share of the conductor
/// until it is shut down.
pub fn start(handle: Arc<ConductorHandle>) -> Option<MidiService> {
    match fp_control::service::spawn(
        Box::new(fp_control::ports::MidirPorts),
        Arc::new(Bridge(handle)),
    ) {
        Ok(midi) => Some(midi),
        Err(error) => {
            tracing::warn!(%error, "MIDI control could not start");
            None
        }
    }
}
