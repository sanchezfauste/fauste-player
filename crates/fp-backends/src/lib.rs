//! Audio output backends behind one trait (spec §5). The engine only talks to
//! `AudioBackend`; each backend turns the engine's `Renderer` into a device
//! stream. Phase 1 ships `Null`, `Offline` (tests) and a cpal-based backend.

#![deny(clippy::indexing_slicing)]

use std::sync::Arc;

use thiserror::Error;

#[cfg(target_os = "linux")]
mod alsa_dsd;
#[cfg(target_os = "macos")]
mod coreaudio_hog;
mod cpal_backend;
pub mod dsd;
pub mod exclusive;
mod hosts;
mod null;
mod offline;
#[cfg(windows)]
mod wasapi_exclusive;

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
    /// What sets this device apart from others with the same name (the
    /// output profile of a sound card), when the backend says.
    pub detail: Option<String>,
    pub channels: u16,
    /// Inclusive sample-rate ranges the device reports.
    pub sample_rates: Vec<(u32, u32)>,
    /// Inclusive buffer-size range in frames, when the device reports one.
    pub buffer_frames: Option<(u32, u32)>,
    pub exclusive_capable: bool,
    pub rate_switching: bool,
    /// The device takes raw DSD (Linux, ALSA hw: devices reporting a DSD format).
    pub native_dsd: bool,
}

/// The labels an output picker shows for `devices`, in order: the name (the
/// id when it is blank), then ` — detail` when the backend gives one that adds something, then
/// ` (id)` for any label two devices still share, so that every choice can
/// be told apart.
pub fn device_labels(devices: &[DeviceInfo]) -> Vec<String> {
    let base: Vec<String> = devices
        .iter()
        .map(|d| {
            // A device with no name is known by its id.
            let name = match d.name.trim() {
                "" => d.id.0.as_str(),
                name => name,
            };
            match d
                .detail
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty() && *s != name)
            {
                Some(detail) => format!("{name} — {detail}"),
                None => name.to_owned(),
            }
        })
        .collect();
    base.iter()
        .zip(devices)
        .map(|(label, d)| {
            if base.iter().filter(|l| *l == label).count() > 1 {
                format!("{label} ({})", d.id.0)
            } else {
                label.clone()
            }
        })
        .collect()
}

/// What the engine asks a backend to open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamConfig {
    pub sample_rate: u32,
    pub buffer_frames: u32,
    pub channels: u16,
    /// Sole, unconverted access to the device (Phase 4 spec B4). A backend
    /// that cannot give it for this device refuses with `Unsupported`.
    pub exclusive: bool,
    /// DSD carried by this stream (feedback 2 spec O25); `None` for PCM.
    pub dsd: Option<dsd::DsdStream>,
}

/// The sample format a stream really runs in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleFormat {
    F32,
    I32,
    /// 24 bits in a 32-bit container.
    I24,
    I16,
}

impl SampleFormat {
    /// Whether integer PCM of `bits` reaches the device unchanged through
    /// the mixer, which works in f32 (a 24-bit mantissa): at most 24 bits,
    /// and no more than the format itself carries.
    pub fn holds_bits(self, bits: u32) -> bool {
        let capacity = match self {
            SampleFormat::F32 | SampleFormat::I32 | SampleFormat::I24 => 24,
            SampleFormat::I16 => 16,
        };
        bits <= capacity
    }
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
    /// The sample format the device runs in.
    fn sample_format(&self) -> SampleFormat;
    /// How the stream carries DSD, `None` for PCM.
    fn dsd(&self) -> Option<dsd::DsdStream> {
        None
    }
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
