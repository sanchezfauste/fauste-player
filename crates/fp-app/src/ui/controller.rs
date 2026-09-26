//! What the UI needs from the rest of the application. The production
//! implementation is the conductor's handle; tests use a recording fake.

use std::sync::Arc;

use fp_engine::conductor::{ConductorHandle, Telemetry};
use fp_model::{AppState, Command, ModelError, Route};

pub trait Controller: Send + Sync {
    /// The latest model snapshot.
    fn model(&self) -> Arc<AppState>;
    /// The latest engine telemetry.
    fn telemetry(&self) -> Arc<Telemetry>;
    /// Queues a command; never blocks. `false` if it was dropped.
    fn send(&self, command: Command) -> bool;
    /// Plays a test tone on `route`; never blocks.
    fn test_tone(&self, route: Route, frequency_hz: f32) -> bool;
    /// The next command the model refused, if any.
    fn take_rejection(&self) -> Option<ModelError>;
}

impl Controller for ConductorHandle {
    fn model(&self) -> Arc<AppState> {
        self.model.load_full()
    }

    fn telemetry(&self) -> Arc<Telemetry> {
        self.telemetry.load_full()
    }

    fn send(&self, command: Command) -> bool {
        ConductorHandle::send(self, command)
    }

    fn test_tone(&self, route: Route, frequency_hz: f32) -> bool {
        ConductorHandle::test_tone(self, route, frequency_hz)
    }

    fn take_rejection(&self) -> Option<ModelError> {
        self.rejected.try_recv().ok()
    }
}
