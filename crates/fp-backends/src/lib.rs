//! Audio output backends behind one trait (spec §5). The engine only talks to
//! `AudioBackend`; each backend turns the engine's `Renderer` into a device
//! stream. Phase 1 ships `Null`, `Offline` (tests) and a cpal-based backend.

#![deny(clippy::indexing_slicing)]

use std::sync::Arc;

use thiserror::Error;

mod cpal_backend;
mod hosts;
mod null;
mod offline;

pub use cpal_backend::CpalBackend;
pub use hosts::{
    choose_default_backend, display_name, host_availability, preferred_backend, system_backends,
};
pub use null::NullBackend;
pub use offline::{OfflineBackend, OfflineDevice};

/// Stable identifier of a backend, e.g. `"alsa"`, `"wasapi"`, `"null"`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BackendId(pub String);

/// Backend-specific device identifier that survives restarts (stored in config).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeviceId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Availability {
    Available,
    Unavailable(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    pub id: DeviceId,
    pub name: String,
    pub channels: u16,
    /// Inclusive sample-rate ranges the device reports.
    pub sample_rates: Vec<(u32, u32)>,
    /// Inclusive buffer-size range in frames, when the device reports one.
    pub buffer_frames: Option<(u32, u32)>,
    pub exclusive_capable: bool,
    pub rate_switching: bool,
}

/// What the engine asks a backend to open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamConfig {
    pub sample_rate: u32,
    pub buffer_frames: u32,
    pub channels: u16,
}

/// Classified stream failures, reported from the backend's own threads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamErrorKind {
    /// The device disappeared or the stream can no longer run.
    DeviceLost,
    /// A buffer underrun/overrun happened in the backend.
    Xrun,
    /// The OS refused real-time scheduling; audio still plays but is more
    /// exposed to glitches under load (shown as a warning in Settings).
    RealtimeDenied,
    /// Anything else the backend reported.
    Other,
}

/// Receives stream errors. Implementations must be cheap and lock-free: they
/// may be called on a real-time thread.
pub trait StreamErrorSink: Send + Sync {
    fn report(&self, kind: StreamErrorKind);
}

/// Produces audio. Called on the backend's real-time thread: implementations
/// must not allocate, block, perform I/O or panic.
pub trait Renderer: Send + 'static {
    /// Fills `out` (interleaved f32, `channels` per frame) completely.
    fn render(&mut self, out: &mut [f32], channels: usize);
}

/// A running output stream. Dropping it stops the stream.
pub trait OutputStream: Send {
    /// The configuration actually in use.
    fn config(&self) -> StreamConfig;
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BackendError {
    #[error("backend unavailable: {0}")]
    Unavailable(String),
    #[error("device not found: {0:?}")]
    DeviceNotFound(DeviceId),
    #[error("unsupported configuration: {0}")]
    Unsupported(String),
    #[error("backend error: {0}")]
    Backend(String),
}

pub trait AudioBackend: Send + Sync {
    fn id(&self) -> BackendId;
    fn availability(&self) -> Availability;
    fn enumerate_devices(&self) -> Result<Vec<DeviceInfo>, BackendError>;
    fn default_device(&self) -> Option<DeviceId>;
    fn open_output(
        &self,
        device: &DeviceId,
        config: StreamConfig,
        renderer: Box<dyn Renderer>,
        errors: Arc<dyn StreamErrorSink>,
    ) -> Result<Box<dyn OutputStream>, BackendError>;
}
