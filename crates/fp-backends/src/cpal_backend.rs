//! Backend built on cpal. In Phase 1 it provides the platform default host
//! (ALSA on Linux, WASAPI shared mode on Windows, Core Audio on macOS).

use std::str::FromStr;
use std::sync::Arc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::{
    AudioBackend, Availability, BackendError, BackendId, DeviceId, DeviceInfo, OutputStream,
    Renderer, StreamConfig, StreamErrorKind, StreamErrorSink,
};

#[derive(Debug, Clone, Copy)]
pub struct CpalBackend {
    host_id: cpal::HostId,
}

struct CpalStream {
    config: StreamConfig,
    _stream: cpal::Stream,
}

impl OutputStream for CpalStream {
    fn config(&self) -> StreamConfig {
        self.config
    }
}

fn backend_error(e: impl std::fmt::Display) -> BackendError {
    BackendError::Backend(e.to_string())
}

fn classify(kind: cpal::ErrorKind) -> StreamErrorKind {
    match kind {
        cpal::ErrorKind::DeviceNotAvailable
        | cpal::ErrorKind::StreamInvalidated
        | cpal::ErrorKind::HostUnavailable => StreamErrorKind::DeviceLost,
        cpal::ErrorKind::Xrun => StreamErrorKind::Xrun,
        cpal::ErrorKind::RealtimeDenied => StreamErrorKind::RealtimeDenied,
        _ => StreamErrorKind::Other,
    }
}

/// The requested buffer size if the device reports a range containing it;
/// `None` means "let the device choose" (a device that refuses a fixed size
/// must still play rather than leave the bus silent).
fn choose_buffer_frames(ranges: &[(u32, u32)], wanted: u32) -> Option<u32> {
    ranges
        .iter()
        .any(|&(min, max)| (min..=max).contains(&wanted))
        .then_some(wanted)
}

impl CpalBackend {
    /// The platform's default cpal host.
    pub fn default_host() -> Self {
        Self {
            host_id: cpal::default_host().id(),
        }
    }

    fn host(&self) -> Result<cpal::Host, BackendError> {
        cpal::host_from_id(self.host_id).map_err(|e| BackendError::Unavailable(e.to_string()))
    }

    fn find(&self, host: &cpal::Host, device: &DeviceId) -> Result<cpal::Device, BackendError> {
        let id = cpal::DeviceId::from_str(&device.0)
            .map_err(|_| BackendError::DeviceNotFound(device.clone()))?;
        host.device_by_id(&id)
            .ok_or_else(|| BackendError::DeviceNotFound(device.clone()))
    }
}

impl AudioBackend for CpalBackend {
    fn id(&self) -> BackendId {
        BackendId(self.host_id.name().to_lowercase())
    }

    fn availability(&self) -> Availability {
        match self.host() {
            Ok(_) => Availability::Available,
            Err(e) => Availability::Unavailable(e.to_string()),
        }
    }

    fn enumerate_devices(&self) -> Result<Vec<DeviceInfo>, BackendError> {
        let host = self.host()?;
        let mut list = Vec::new();
        for device in host.output_devices().map_err(backend_error)? {
            let Ok(id) = device.id() else { continue };
            let name = device
                .description()
                .map(|d| d.name().to_owned())
                .unwrap_or_else(|_| id.to_string());
            let mut channels = 0;
            let mut sample_rates = Vec::new();
            let mut buffer_frames: Option<(u32, u32)> = None;
            if let Ok(configs) = device.supported_output_configs() {
                for c in configs {
                    channels = channels.max(c.channels());
                    sample_rates.push((c.min_sample_rate(), c.max_sample_rate()));
                    if let cpal::SupportedBufferSize::Range { min, max } = *c.buffer_size() {
                        buffer_frames = Some(match buffer_frames {
                            Some((lo, hi)) => (lo.min(min), hi.max(max)),
                            None => (min, max),
                        });
                    }
                }
            }
            sample_rates.sort_unstable();
            sample_rates.dedup();
            list.push(DeviceInfo {
                id: DeviceId(id.to_string()),
                name,
                channels,
                sample_rates,
                buffer_frames,
                exclusive_capable: false,
                rate_switching: false,
            });
        }
        Ok(list)
    }

    fn default_device(&self) -> Option<DeviceId> {
        let host = self.host().ok()?;
        let device = host.default_output_device()?;
        device.id().ok().map(|id| DeviceId(id.to_string()))
    }

    fn open_output(
        &self,
        device: &DeviceId,
        config: StreamConfig,
        mut renderer: Box<dyn Renderer>,
        errors: Arc<dyn StreamErrorSink>,
    ) -> Result<Box<dyn OutputStream>, BackendError> {
        let host = self.host()?;
        let dev = self.find(&host, device)?;
        let channels = usize::from(config.channels);
        let ranges: Vec<(u32, u32)> = dev
            .supported_output_configs()
            .map(|configs| {
                configs
                    .filter(|c| {
                        c.channels() >= config.channels
                            && c.min_sample_rate() <= config.sample_rate
                            && config.sample_rate <= c.max_sample_rate()
                    })
                    .filter_map(|c| match *c.buffer_size() {
                        cpal::SupportedBufferSize::Range { min, max } => Some((min, max)),
                        cpal::SupportedBufferSize::Unknown => None,
                    })
                    .collect()
            })
            .unwrap_or_default();
        let buffer_size = match choose_buffer_frames(&ranges, config.buffer_frames) {
            Some(frames) => cpal::BufferSize::Fixed(frames),
            None => cpal::BufferSize::Default,
        };
        let cpal_config = cpal::StreamConfig {
            channels: config.channels,
            sample_rate: config.sample_rate,
            buffer_size,
        };
        let stream = dev
            .build_output_stream::<f32, _, _>(
                cpal_config,
                move |out: &mut [f32], _info: &cpal::OutputCallbackInfo| {
                    renderer.render(out, channels)
                },
                move |err: cpal::Error| errors.report(classify(err.kind())),
                None,
            )
            .map_err(|e| match e.kind() {
                cpal::ErrorKind::DeviceNotAvailable => BackendError::DeviceNotFound(device.clone()),
                cpal::ErrorKind::UnsupportedConfig => BackendError::Unsupported(e.to_string()),
                _ => backend_error(e),
            })?;
        stream.play().map_err(backend_error)?;
        Ok(Box::new(CpalStream {
            config,
            _stream: stream,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::choose_buffer_frames;

    #[test]
    fn a_buffer_size_outside_every_reported_range_falls_back_to_the_device_default() {
        assert_eq!(choose_buffer_frames(&[(64, 4096)], 512), Some(512));
        assert_eq!(choose_buffer_frames(&[(1024, 4096)], 512), None);
        assert_eq!(choose_buffer_frames(&[], 512), None);
    }
}
