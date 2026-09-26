//! Backend built on cpal: one per audio system (cpal host) compiled in.

use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::hosts::host_availability;
use crate::{
    AudioBackend, Availability, BackendError, BackendId, DeviceId, DeviceInfo, OutputStream,
    Renderer, SampleFormat, StreamConfig, StreamErrorKind, StreamErrorSink,
};

pub struct CpalBackend {
    host_id: cpal::HostId,
    host: HostCache<cpal::Host>,
}

/// A host created on first use and kept: some systems (PulseAudio, JACK)
/// open a server connection per host, which should not happen on every
/// call. A host that failed to open is not kept, and one whose connection
/// was lost (a server restart, a device gone) is rebuilt on the next use,
/// so the watchdog's reopen reaches a live server.
struct HostCache<H> {
    host: Mutex<Option<Arc<H>>>,
    /// Set from a stream's error callback (real-time safe) when the device
    /// or server is lost.
    lost: Arc<AtomicBool>,
}

impl<H> Default for HostCache<H> {
    fn default() -> Self {
        Self {
            host: Mutex::new(None),
            lost: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl<H> HostCache<H> {
    fn get(&self, open: impl FnOnce() -> Result<H, String>) -> Result<Arc<H>, String> {
        let mut host = self.host.lock().unwrap_or_else(PoisonError::into_inner);
        if self.lost.swap(false, Ordering::AcqRel) {
            *host = None;
        }
        if let Some(h) = host.as_ref() {
            return Ok(Arc::clone(h));
        }
        let fresh = Arc::new(open()?);
        *host = Some(Arc::clone(&fresh));
        Ok(fresh)
    }

    fn lost(&self) -> &Arc<AtomicBool> {
        &self.lost
    }
}

/// Passes stream errors on, and marks the host for rebuilding when the
/// device or server is lost.
struct MarkLost {
    inner: Arc<dyn StreamErrorSink>,
    lost: Arc<AtomicBool>,
}

impl StreamErrorSink for MarkLost {
    fn report(&self, kind: StreamErrorKind) {
        if kind == StreamErrorKind::DeviceLost {
            self.lost.store(true, Ordering::Release);
        }
        self.inner.report(kind);
    }
}

struct CpalStream {
    config: StreamConfig,
    format: SampleFormat,
    _stream: cpal::Stream,
}

impl OutputStream for CpalStream {
    fn config(&self) -> StreamConfig {
        self.config
    }

    fn sample_format(&self) -> SampleFormat {
        self.format
    }
}

/// Whether a device can give the application sole, unconverted access. On
/// ALSA a `hw:` device is the hardware itself; everything else there goes
/// through plugins (`plughw:`, `default`) or a sound server. WASAPI has
/// exclusive mode and Core Audio hog mode (Phase 4 plan 2).
fn exclusive_capable(host: &str, device: &str) -> bool {
    // cpal's persisted ids name the host first (`alsa:hw:CARD=PCH,DEV=0`).
    let inner = device
        .strip_prefix(host)
        .and_then(|rest| rest.strip_prefix(':'))
        .unwrap_or(device);
    match host {
        "alsa" => inner.starts_with("hw:"),
        // Exclusive mode and hog mode: every output device can be asked;
        // whether it is granted is known when the stream opens.
        "wasapi" | "coreaudio" => true,
        _ => false,
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
            host: HostCache::default(),
        }
    }

    fn open_host(&self) -> Result<Arc<cpal::Host>, String> {
        self.host
            .get(|| cpal::host_from_id(self.host_id).map_err(|e| e.to_string()))
    }

    fn host(&self) -> Result<Arc<cpal::Host>, BackendError> {
        self.open_host().map_err(BackendError::Unavailable)
    }

    /// Output channels to offer for `device`. A sound server (PulseAudio)
    /// lists every channel count it could remix to; the sink's own layout
    /// is its default configuration.
    fn channels(&self, device: &cpal::Device, supported: u16) -> u16 {
        if self.host_id.name().eq_ignore_ascii_case("pulseaudio")
            && let Ok(config) = device.default_output_config()
        {
            return config.channels();
        }
        supported
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
        let opened = self.open_host().map(|host| {
            host.default_output_device().is_some()
                || host
                    .output_devices()
                    .is_ok_and(|mut devices| devices.next().is_some())
        });
        if opened != Ok(true) {
            // Try again next time: the server may be started later.
            self.host.lost().store(true, Ordering::Release);
        }
        host_availability(opened)
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
            let channels = self.channels(&device, channels);
            let exclusive = exclusive_capable(&self.id().0, &id.to_string());
            list.push(DeviceInfo {
                id: DeviceId(id.to_string()),
                name,
                channels,
                sample_rates,
                buffer_frames,
                exclusive_capable: exclusive,
                rate_switching: exclusive,
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
        let result = self.open_on_host(device, config, renderer, errors);
        if result.is_err() {
            // A stale connection is the usual cause; the next attempt
            // starts from a fresh host.
            self.host.lost().store(true, Ordering::Release);
        }
        result
    }
}

impl CpalBackend {
    fn open_on_host(
        &self,
        device: &DeviceId,
        config: StreamConfig,
        renderer: Box<dyn Renderer>,
        errors: Arc<dyn StreamErrorSink>,
    ) -> Result<Box<dyn OutputStream>, BackendError> {
        let errors: Arc<dyn StreamErrorSink> = Arc::new(MarkLost {
            inner: errors,
            lost: Arc::clone(self.host.lost()),
        });
        if config.exclusive && !exclusive_capable(&self.id().0, &device.0) {
            return Err(BackendError::Unsupported(
                "exclusive access is not available on this device".to_owned(),
            ));
        }
        if config.exclusive {
            match self.id().0.as_str() {
                // A hw: device is exclusive by itself: the cpal path below.
                "alsa" => {}
                #[cfg(windows)]
                "wasapi" => return crate::wasapi_exclusive::open(device, config, renderer, errors),
                _ => {
                    return Err(BackendError::Unsupported(
                        "exclusive access is not available on this system".to_owned(),
                    ));
                }
            }
        }
        let host = self.host()?;
        let dev = self.find(&host, device)?;
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
                cpal::SampleFormat::I24 => {
                    build::<cpal::I24>(&dev, cpal_config, receiver, errors, scratch_frames)
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
                        format: match format {
                            cpal::SampleFormat::I16 => SampleFormat::I16,
                            cpal::SampleFormat::I32 => SampleFormat::I32,
                            cpal::SampleFormat::I24 => SampleFormat::I24,
                            _ => SampleFormat::F32,
                        },
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
        cpal::SampleFormat::I24,
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
    use super::{
        HostCache, choose_buffer_frames, choose_sample_format, exclusive_capable, render_converted,
    };
    use crate::Renderer;

    /// Plays back fixed samples.
    struct Samples(Vec<f32>, usize);

    impl Renderer for Samples {
        fn render(&mut self, out: &mut [f32], _channels: usize) {
            for s in out {
                *s = self.0.get(self.1).copied().unwrap_or(0.0);
                self.1 += 1;
            }
        }
    }

    #[test]
    fn conversion_to_i16_is_exact_for_16_bit_pcm() {
        let values: Vec<i16> = (i16::MIN..=i16::MAX).collect();
        let samples = values.iter().map(|v| f32::from(*v) / 32_768.0).collect();
        let mut out = vec![0i16; values.len()];
        let mut scratch = vec![0.0f32; 4096];
        render_converted(&mut Samples(samples, 0), &mut out, 2, &mut scratch);
        assert_eq!(out, values);
    }

    #[test]
    fn conversion_to_i32_is_exact_for_24_bit_pcm() {
        let values: Vec<i32> = (-(1 << 23)..(1 << 23))
            .step_by(97)
            .chain([(1 << 23) - 1])
            .collect();
        let samples = values.iter().map(|v| *v as f32 / 8_388_608.0).collect();
        let mut out = vec![0i32; values.len()];
        let mut scratch = vec![0.0f32; 4096];
        render_converted(&mut Samples(samples, 0), &mut out, 1, &mut scratch);
        let expected: Vec<i32> = values.iter().map(|v| v << 8).collect();
        assert_eq!(out, expected);
    }

    #[test]
    fn exclusive_capability_reads_cpal_device_ids() {
        // cpal's persisted ids carry the host: `alsa:hw:CARD=PCH,DEV=0`.
        assert!(exclusive_capable("alsa", "alsa:hw:CARD=PCH,DEV=0"));
        assert!(!exclusive_capable("alsa", "alsa:plughw:CARD=PCH,DEV=0"));
        assert!(!exclusive_capable("alsa", "alsa:default"));
        assert!(!exclusive_capable("pulseaudio", "pulseaudio:hw:0,0"));
    }

    #[test]
    fn devices_with_only_24_bit_integer_are_used() {
        use cpal::SampleFormat::{I16, I24};
        assert_eq!(choose_sample_format(&[I16, I24]), Some(I24));
    }

    #[test]
    fn conversion_to_i24_is_exact_for_24_bit_pcm() {
        let values: Vec<i32> = (-(1 << 23)..(1 << 23)).step_by(89).collect();
        let samples = values.iter().map(|v| *v as f32 / 8_388_608.0).collect();
        let mut out = vec![cpal::I24::new(0).unwrap(); values.len()];
        let mut scratch = vec![0.0f32; 4096];
        render_converted(&mut Samples(samples, 0), &mut out, 1, &mut scratch);
        let got: Vec<i32> = out.iter().map(|v| v.inner()).collect();
        assert_eq!(got, values);
    }

    #[test]
    fn wasapi_and_core_audio_devices_are_exclusive_capable() {
        assert!(exclusive_capable(
            "wasapi",
            "wasapi:{0.0.0.00000000}.{guid}"
        ));
        assert!(exclusive_capable(
            "coreaudio",
            "coreaudio:BuiltInSpeakerDevice"
        ));
        assert!(!exclusive_capable("jack", "jack:system"));
        assert!(!exclusive_capable("asio", "asio:Interface"));
    }

    #[test]
    fn alsa_hw_devices_are_exclusive_capable() {
        assert!(exclusive_capable("alsa", "hw:CARD=PCH,DEV=0"));
        assert!(exclusive_capable("alsa", "hw:0,0"));
        assert!(!exclusive_capable("alsa", "plughw:CARD=PCH,DEV=0"));
        assert!(!exclusive_capable("alsa", "default"));
        assert!(!exclusive_capable("pulseaudio", "hw:0,0"));
        assert!(!exclusive_capable("jack", "system"));
    }

    #[test]
    fn a_host_that_failed_to_open_is_tried_again() {
        let cache = HostCache::<u32>::default();
        assert!(cache.get(|| Err("server not running".to_owned())).is_err());
        assert_eq!(cache.get(|| Ok(7)).ok().as_deref(), Some(&7));
    }

    #[test]
    fn a_host_is_kept_until_it_is_lost_then_rebuilt() {
        let cache = HostCache::<u32>::default();
        assert_eq!(cache.get(|| Ok(1)).ok().as_deref(), Some(&1));
        assert_eq!(cache.get(|| Ok(2)).ok().as_deref(), Some(&1), "kept");
        cache
            .lost()
            .store(true, std::sync::atomic::Ordering::Release);
        assert_eq!(cache.get(|| Ok(3)).ok().as_deref(), Some(&3), "rebuilt");
    }

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
