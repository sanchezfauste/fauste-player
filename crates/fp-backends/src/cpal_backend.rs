//! Backend built on cpal. In Phase 1 it provides the platform default host
//! (ALSA on Linux, WASAPI shared mode on Windows, Core Audio on macOS).

use std::str::FromStr;
use std::sync::Arc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::{
    AudioBackend, Availability, BackendError, BackendId, DeviceId, DeviceInfo, OutputStream,
    Renderer, StreamConfig, StreamErrorKind, StreamErrorSink,
};

pub struct CpalBackend {
    host_id: cpal::HostId,
    /// Created on first use and kept: some systems (PulseAudio, JACK) open
    /// a server connection per host, which should not happen on every call.
    host: std::sync::OnceLock<Result<cpal::Host, String>>,
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

impl std::fmt::Debug for CpalBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CpalBackend")
            .field("host", &self.host_id)
            .finish()
    }
}

impl CpalBackend {
    /// The platform's default cpal host.
    pub fn default_host() -> Self {
        Self::for_host(cpal::default_host().id())
    }

    /// One audio system (ALSA, PulseAudio, PipeWire, JACK, WASAPI, ASIO,
    /// Core Audio), as compiled into this build.
    pub fn for_host(host_id: cpal::HostId) -> Self {
        Self {
            host_id,
            host: std::sync::OnceLock::new(),
        }
    }

    fn host(&self) -> Result<&cpal::Host, BackendError> {
        self.host
            .get_or_init(|| cpal::host_from_id(self.host_id).map_err(|e| e.to_string()))
            .as_ref()
            .map_err(|e| BackendError::Unavailable(e.clone()))
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
        renderer: Box<dyn Renderer>,
        errors: Arc<dyn StreamErrorSink>,
    ) -> Result<Box<dyn OutputStream>, BackendError> {
        let host = self.host()?;
        let dev = self.find(host, device)?;
        let matching: Vec<cpal::SupportedStreamConfigRange> = dev
            .supported_output_configs()
            .map(|configs| {
                configs
                    .filter(|c| {
                        c.channels() >= config.channels
                            && c.min_sample_rate() <= config.sample_rate
                            && config.sample_rate <= c.max_sample_rate()
                    })
                    .collect()
            })
            .unwrap_or_default();
        let formats: Vec<cpal::SampleFormat> = matching.iter().map(|c| c.sample_format()).collect();
        // Devices that report nothing usable are still tried with f32.
        let format = choose_sample_format(&formats).unwrap_or(cpal::SampleFormat::F32);
        let ranges: Vec<(u32, u32)> = matching
            .iter()
            .filter(|c| c.sample_format() == format)
            .filter_map(|c| match *c.buffer_size() {
                cpal::SupportedBufferSize::Range { min, max } => Some((min, max)),
                cpal::SupportedBufferSize::Unknown => None,
            })
            .collect();
        let mut attempts = Vec::with_capacity(2);
        if let Some(frames) = choose_buffer_frames(&ranges, config.buffer_frames) {
            attempts.push(cpal::BufferSize::Fixed(frames));
        }
        // A device that rejects a fixed size must still play rather than
        // leave the bus silent: fall back to its own default.
        attempts.push(cpal::BufferSize::Default);
        let scratch_frames = (config.buffer_frames as usize).max(4096);
        let mut renderer = Some(renderer);
        let mut last = BackendError::Unsupported("no configuration accepted".to_owned());
        for buffer_size in attempts {
            let cpal_config = cpal::StreamConfig {
                channels: config.channels,
                sample_rate: config.sample_rate,
                buffer_size,
            };
            // The renderer is handed to the callback only once the stream
            // exists, so a failed attempt never loses it.
            let (mut handoff, receiver) = rtrb::RingBuffer::<Box<dyn Renderer>>::new(1);
            let errors = errors.clone();
            let built = match format {
                cpal::SampleFormat::I16 => {
                    build::<i16>(&dev, cpal_config, receiver, errors, scratch_frames)
                }
                cpal::SampleFormat::I32 => {
                    build::<i32>(&dev, cpal_config, receiver, errors, scratch_frames)
                }
                _ => build::<f32>(&dev, cpal_config, receiver, errors, scratch_frames),
            };
            match built {
                Ok(stream) => {
                    if let Some(r) = renderer.take() {
                        let _ = handoff.push(r);
                    }
                    stream.play().map_err(backend_error)?;
                    return Ok(Box::new(CpalStream {
                        config,
                        _stream: stream,
                    }));
                }
                Err(e) => match e.kind() {
                    cpal::ErrorKind::DeviceNotAvailable => {
                        return Err(BackendError::DeviceNotFound(device.clone()));
                    }
                    cpal::ErrorKind::UnsupportedConfig => {
                        last = BackendError::Unsupported(e.to_string())
                    }
                    _ => return Err(backend_error(e)),
                },
            }
        }
        Err(last)
    }
}

/// Builds an output stream in sample type `T`. The callback renders f32 into
/// a scratch buffer allocated here (never in the callback) and converts.
fn build<T>(
    dev: &cpal::Device,
    config: cpal::StreamConfig,
    mut handoff: rtrb::Consumer<Box<dyn Renderer>>,
    errors: Arc<dyn StreamErrorSink>,
    scratch_frames: usize,
) -> Result<cpal::Stream, cpal::Error>
where
    T: cpal::SizedSample + cpal::FromSample<f32> + Send + 'static,
{
    let channels = usize::from(config.channels).max(1);
    let mut scratch = vec![0.0f32; scratch_frames.max(1) * channels];
    let mut renderer: Option<Box<dyn Renderer>> = None;
    dev.build_output_stream::<T, _, _>(
        config,
        move |out: &mut [T], _info: &cpal::OutputCallbackInfo| {
            if renderer.is_none() {
                renderer = handoff.pop().ok();
            }
            match renderer.as_mut() {
                Some(r) => render_converted(r.as_mut(), out, channels, &mut scratch),
                None => out.fill(T::EQUILIBRIUM),
            }
        },
        move |err: cpal::Error| errors.report(classify(err.kind())),
        None,
    )
}

/// Prefers float output, then the widest integer format the device takes.
fn choose_sample_format(formats: &[cpal::SampleFormat]) -> Option<cpal::SampleFormat> {
    [
        cpal::SampleFormat::F32,
        cpal::SampleFormat::I32,
        cpal::SampleFormat::I16,
    ]
    .into_iter()
    .find(|f| formats.contains(f))
}

/// Renders `out` through `scratch` in whole-frame pieces and converts each
/// sample to the device format (saturating). Real-time safe: no allocation.
fn render_converted<T>(
    renderer: &mut dyn Renderer,
    out: &mut [T],
    channels: usize,
    scratch: &mut [f32],
) where
    T: cpal::SizedSample + cpal::FromSample<f32>,
{
    let channels = channels.max(1);
    let step = (scratch.len() / channels).max(1) * channels;
    for chunk in out.chunks_mut(step) {
        let Some(buf) = scratch.get_mut(..chunk.len()) else {
            chunk.fill(T::EQUILIBRIUM);
            continue;
        };
        renderer.render(buf, channels);
        for (o, s) in chunk.iter_mut().zip(buf.iter()) {
            *o = T::from_sample(*s);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{choose_buffer_frames, choose_sample_format, render_converted};
    use crate::Renderer;

    struct Ramp(f32);

    impl Renderer for Ramp {
        fn render(&mut self, out: &mut [f32], _channels: usize) {
            for s in out {
                *s = self.0;
                self.0 += 0.25;
            }
        }
    }

    #[test]
    fn float_output_is_preferred_then_the_widest_integer_format() {
        use cpal::SampleFormat::{F32, I16, I32, U8};
        assert_eq!(choose_sample_format(&[I16, F32, I32]), Some(F32));
        assert_eq!(choose_sample_format(&[I16, I32]), Some(I32));
        assert_eq!(choose_sample_format(&[I16]), Some(I16));
        assert_eq!(choose_sample_format(&[U8]), None);
    }

    #[test]
    fn integer_devices_get_converted_audio_rendered_in_scratch_sized_pieces() {
        let mut renderer = Ramp(0.0);
        let mut out = [0i16; 6];
        let mut scratch = [0.0f32; 4]; // smaller than the block: rendered in pieces
        render_converted(&mut renderer, &mut out, 2, &mut scratch);
        assert_eq!(out, [0, 8192, 16384, 24576, i16::MAX, i16::MAX]);
    }

    #[test]
    fn a_buffer_size_outside_every_reported_range_falls_back_to_the_device_default() {
        assert_eq!(choose_buffer_frames(&[(64, 4096)], 512), Some(512));
        assert_eq!(choose_buffer_frames(&[(1024, 4096)], 512), None);
        assert_eq!(choose_buffer_frames(&[], 512), None);
    }
}
