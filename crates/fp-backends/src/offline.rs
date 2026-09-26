//! A backend for tests: devices are rendered on demand, on the caller's
//! thread, so engine behaviour can be checked sample by sample without a
//! sound card or real time passing.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::{
    AudioBackend, Availability, BackendError, BackendId, DeviceId, DeviceInfo, OutputStream,
    Renderer, SampleFormat, StreamConfig, StreamErrorKind, StreamErrorSink,
};

struct Open {
    generation: u64,
    config: StreamConfig,
    renderer: Box<dyn Renderer>,
    errors: Arc<dyn StreamErrorSink>,
}

#[derive(Default)]
struct DeviceState {
    plugged: bool,
    generation: u64,
    open: Option<Open>,
    exclusive_capable: bool,
    refused_rates: HashSet<u32>,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Test handle to one offline device.
#[derive(Clone)]
pub struct OfflineDevice {
    id: DeviceId,
    channels: u16,
    state: Arc<Mutex<DeviceState>>,
}

impl OfflineDevice {
    pub fn id(&self) -> DeviceId {
        self.id.clone()
    }

    /// Renders `frames` frames through the open stream. Returns `None` when no
    /// stream is open (never opened, dropped, or the device was unplugged).
    pub fn render(&self, frames: usize) -> Option<Vec<f32>> {
        let mut state = lock(&self.state);
        let open = state.open.as_mut()?;
        let channels = usize::from(open.config.channels);
        let mut out = vec![0.0; frames * channels];
        open.renderer.render(&mut out, channels);
        Some(out)
    }

    pub fn is_open(&self) -> bool {
        lock(&self.state).open.is_some()
    }

    /// Simulates the device disappearing: reports `DeviceLost`, stops the
    /// stream and makes further opens fail until `replug`.
    pub fn unplug(&self) {
        let mut state = lock(&self.state);
        state.plugged = false;
        if let Some(open) = state.open.take() {
            open.errors.report(StreamErrorKind::DeviceLost);
        }
    }

    pub fn replug(&self) {
        lock(&self.state).plugged = true;
    }

    /// Lets streams on this device be opened with `exclusive`.
    pub fn set_exclusive_capable(&self, capable: bool) {
        lock(&self.state).exclusive_capable = capable;
    }

    /// Makes opens at `rate` fail with `Unsupported`.
    pub fn refuse_rate(&self, rate: u32) {
        lock(&self.state).refused_rates.insert(rate);
    }

    /// The configuration of the open stream, if any.
    pub fn config(&self) -> Option<StreamConfig> {
        lock(&self.state).open.as_ref().map(|o| o.config)
    }
}

struct OfflineStream {
    generation: u64,
    config: StreamConfig,
    state: Arc<Mutex<DeviceState>>,
}

impl OutputStream for OfflineStream {
    fn config(&self) -> StreamConfig {
        self.config
    }

    fn sample_format(&self) -> SampleFormat {
        SampleFormat::F32
    }
}

impl Drop for OfflineStream {
    fn drop(&mut self) {
        let mut state = lock(&self.state);
        if state
            .open
            .as_ref()
            .is_some_and(|o| o.generation == self.generation)
        {
            state.open = None;
        }
    }
}

/// See the module documentation.
#[derive(Clone, Default)]
pub struct OfflineBackend {
    devices: Arc<Mutex<HashMap<DeviceId, OfflineDevice>>>,
}

impl OfflineBackend {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a plugged-in device with `channels` output channels.
    pub fn add_device(&self, id: &str, channels: u16) -> OfflineDevice {
        let device = OfflineDevice {
            id: DeviceId(id.to_owned()),
            channels,
            state: Arc::new(Mutex::new(DeviceState {
                plugged: true,
                ..DeviceState::default()
            })),
        };
        lock(&self.devices).insert(device.id.clone(), device.clone());
        device
    }

    pub fn device(&self, id: &DeviceId) -> Option<OfflineDevice> {
        lock(&self.devices).get(id).cloned()
    }
}

impl AudioBackend for OfflineBackend {
    fn id(&self) -> BackendId {
        BackendId("offline".to_owned())
    }

    fn availability(&self) -> Availability {
        Availability::Available
    }

    fn enumerate_devices(&self) -> Result<Vec<DeviceInfo>, BackendError> {
        let devices = lock(&self.devices);
        let mut list: Vec<DeviceInfo> = devices
            .values()
            .filter(|d| lock(&d.state).plugged)
            .map(|d| {
                let exclusive_capable = lock(&d.state).exclusive_capable;
                DeviceInfo {
                    id: d.id.clone(),
                    name: d.id.0.clone(),
                    channels: d.channels,
                    sample_rates: vec![(8_000, 768_000)],
                    buffer_frames: Some((16, 16_384)),
                    exclusive_capable,
                    rate_switching: exclusive_capable,
                }
            })
            .collect();
        list.sort_by(|a, b| a.id.0.cmp(&b.id.0));
        Ok(list)
    }

    fn default_device(&self) -> Option<DeviceId> {
        self.enumerate_devices()
            .ok()?
            .into_iter()
            .next()
            .map(|d| d.id)
    }

    fn open_output(
        &self,
        device: &DeviceId,
        config: StreamConfig,
        renderer: Box<dyn Renderer>,
        errors: Arc<dyn StreamErrorSink>,
    ) -> Result<Box<dyn OutputStream>, BackendError> {
        let dev = self
            .device(device)
            .ok_or_else(|| BackendError::DeviceNotFound(device.clone()))?;
        if config.channels > dev.channels {
            return Err(BackendError::Unsupported(format!(
                "{} channels requested, device has {}",
                config.channels, dev.channels
            )));
        }
        let mut state = lock(&dev.state);
        if !state.plugged {
            return Err(BackendError::DeviceNotFound(device.clone()));
        }
        if config.exclusive && !state.exclusive_capable {
            return Err(BackendError::Unsupported(
                "exclusive access is not available on this device".to_owned(),
            ));
        }
        if state.refused_rates.contains(&config.sample_rate) {
            return Err(BackendError::Unsupported(format!(
                "{} Hz is not supported",
                config.sample_rate
            )));
        }
        state.generation += 1;
        let generation = state.generation;
        state.open = Some(Open {
            generation,
            config,
            renderer,
            errors,
        });
        Ok(Box::new(OfflineStream {
            generation,
            config,
            state: dev.state.clone(),
        }))
    }
}
