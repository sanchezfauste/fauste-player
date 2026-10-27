//! Backend built on cpal: one per audio system (cpal host) compiled in.

use std::str::FromStr;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::dsd::DsdStream;
use crate::hosts::host_availability;
use crate::{
    AudioBackend, Availability, BackendError, BackendId, DeviceId, DeviceInfo, OutputStream,
    Renderer, SampleFormat, StreamConfig, StreamErrorKind, StreamErrorSink,
};

pub struct CpalBackend {
    host_id: cpal::HostId,
    host: HostCache<cpal::Host>,
    /// The native DSD format each ALSA `hw:` device took when last probed.
    #[cfg(target_os = "linux")]
    native_probe: Mutex<std::collections::HashMap<String, crate::dsd::NativeDsdFormat>>,
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
    /// The fixed buffer size cpal was given, if any.
    fixed_frames: Option<u32>,
    /// The largest block the device has asked for so far.
    called_frames: Arc<AtomicU32>,
    /// The most frames held at once (`held_frames`), 0 before a callback.
    held_frames: Arc<AtomicU32>,
    format: SampleFormat,
    _stream: cpal::Stream,
    /// Exclusive access held for the stream's life (Core Audio hog mode),
    /// released when dropped after the stream.
    _exclusive: Option<Box<dyn Send>>,
}

impl OutputStream for CpalStream {
    fn config(&self) -> StreamConfig {
        StreamConfig {
            buffer_frames: frames_in_use(
                self.config.buffer_frames,
                self.fixed_frames,
                self.called_frames.load(Ordering::Relaxed),
            ),
            ..self.config
        }
    }

    fn sample_format(&self) -> SampleFormat {
        self.format
    }

    fn dsd(&self) -> Option<DsdStream> {
        self.config.dsd
    }

    fn latency_frames(&self) -> Option<u32> {
        Some(self.held_frames.load(Ordering::Relaxed)).filter(|f| *f > 0)
    }
}

/// The frames a stream holds once a callback has handed over its `block`:
/// those still queued before it (`queued`, cpal's playback instant minus
/// its callback instant: the driver's delay, transfers in flight included)
/// and the block itself, at `rate`. Rounded up; saturates. Real-time safe.
fn held_frames(queued: Duration, block: u32, rate: u32) -> u32 {
    let queued = (queued.as_nanos() * u128::from(rate)).div_ceil(1_000_000_000);
    u32::try_from(queued.saturating_add(u128::from(block))).unwrap_or(u32::MAX)
}

/// The buffer size a stream runs with: the fixed size it was given, or with
/// the device's default the largest block called back so far (the request
/// until the first callback).
fn frames_in_use(requested: u32, fixed: Option<u32>, called: u32) -> u32 {
    match fixed {
        Some(frames) => frames,
        None if called > 0 => called,
        None => requested,
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

/// Whether a sound server or a system mixer stands between the stream and
/// the hardware: then the callback only feeds a queue that the mixer reads
/// on its own clock, and a short one runs dry whenever a busy machine holds
/// the callback back. On ALSA the hardware is `hw:` and what converts on
/// its way to one card (`plughw:`, the channel layouts, `iec958:`,
/// `hdmi:`); anything else may be a server or `dmix` (`default`,
/// `pipewire`, `pulse`, and `sysdefault:`, usually `plug` into `dmix`).
/// PipeWire runs its clients on the graph's cycle, and a longer block asks
/// it for a longer quantum when nothing else needs a shorter one. Core
/// Audio, JACK and ASIO call back on the device's (or the JACK server's)
/// own cycle, where the period is the latency asked for.
fn shared_output(host: &str, device: &str, exclusive: bool) -> bool {
    const ALSA_HARDWARE: &[&str] = &[
        "hw:",
        "plughw:",
        "front:",
        "rear:",
        "center_lfe:",
        "side:",
        "surround",
        "iec958:",
        "hdmi:",
    ];
    if exclusive {
        return false;
    }
    let inner = device
        .strip_prefix(host)
        .and_then(|rest| rest.strip_prefix(':'))
        .unwrap_or(device);
    match host {
        "pulseaudio" | "pipewire" | "wasapi" => true,
        "alsa" => !ALSA_HARDWARE.iter().any(|p| inner.starts_with(p)),
        _ => false,
    }
}

/// The buffer to ask for: at least `shared_buffer_us` (rounded up to whole
/// frames) on a shared output, unless the device's own buffer was set
/// (`exact_buffer`). The floor goes no higher than the largest size the
/// device reports (`ranges`; a PipeWire quantum stops at 2048): a size
/// past it would be refused and leave the device to choose its own.
fn wanted_frames(config: &StreamConfig, shared: bool, ranges: &[(u32, u32)]) -> u32 {
    if !shared || config.exact_buffer {
        return config.buffer_frames;
    }
    let floor =
        (u64::from(config.shared_buffer_us) * u64::from(config.sample_rate)).div_ceil(1_000_000);
    let floor = u32::try_from(floor).unwrap_or(u32::MAX);
    let largest = ranges.iter().map(|&(_, max)| max).max();
    config
        .buffer_frames
        .max(largest.map_or(floor, |max| floor.min(max)))
}

fn backend_error(e: impl std::fmt::Display) -> BackendError {
    BackendError::Backend(e.to_string())
}

/// Passes a cpal stream error on, classified. On some hosts (ALSA, WASAPI)
/// cpal calls this on the real-time audio thread, so it only classifies and
/// counts: no logging, formatting or allocation. The conductor logs the
/// counts (an error that fits no class is `StreamErrorKind::Other`).
fn report_stream_error(errors: &dyn StreamErrorSink, err: &cpal::Error) {
    errors.report(classify(err.kind()));
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

/// The buffer sizes to try, in order. A device that rejects a fixed size
/// must still play rather than leave the bus silent, so the device's own
/// default comes last; with `exact` (`StreamConfig::exact_buffer`) only
/// the fixed size is tried, and none when the device reports ranges that
/// leave it out (cpal itself would not refuse it).
fn buffer_attempts(ranges: &[(u32, u32)], wanted: u32, exact: bool) -> Vec<cpal::BufferSize> {
    let mut attempts = Vec::with_capacity(2);
    if exact {
        if ranges.is_empty() || choose_buffer_frames(ranges, wanted).is_some() {
            attempts.push(cpal::BufferSize::Fixed(wanted));
        }
        return attempts;
    }
    if let Some(frames) = choose_buffer_frames(ranges, wanted) {
        attempts.push(cpal::BufferSize::Fixed(frames));
    }
    attempts.push(cpal::BufferSize::Default);
    attempts
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
            #[cfg(target_os = "linux")]
            native_probe: Mutex::default(),
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

    /// The host's outputs with the id, name and detail they are listed
    /// under. On Android the AAudio host's ids are the stable ones of
    /// `aaudio_ids` (mobile spec MB5); elsewhere they are cpal's.
    fn listed_outputs(&self, host: &cpal::Host) -> Result<Vec<Listed>, BackendError> {
        let devices = host.output_devices().map_err(backend_error)?;
        #[cfg(target_os = "android")]
        if self.host_id == cpal::HostId::AAudio {
            return Ok(aaudio_listed(devices.collect()));
        }
        Ok(devices
            .filter_map(|device| {
                let id = device.id().ok()?;
                let description = device.description().ok();
                let name = description
                    .as_ref()
                    .map_or_else(|| id.to_string(), |d| d.name().to_owned());
                let detail = description.as_ref().and_then(device_detail);
                Some((DeviceId(id.to_string()), name, detail, device))
            })
            .collect())
    }

    fn find(&self, host: &cpal::Host, device: &DeviceId) -> Result<cpal::Device, BackendError> {
        #[cfg(target_os = "android")]
        if self.host_id == cpal::HostId::AAudio
            && crate::aaudio_ids::is_stable(&device.0)
            && let Some(found) = pick(
                &device.0,
                self.listed_outputs(host)?
                    .into_iter()
                    .map(|(id, _, _, d)| (id, d))
                    .collect(),
            )
        {
            return Ok(found);
        }
        let id = cpal::DeviceId::from_str(&device.0)
            .map_err(|_| BackendError::DeviceNotFound(device.clone()))?;
        let default = || {
            host.default_output_device()
                .and_then(|d| d.id().ok().map(|id| (id.to_string(), d)))
        };
        resolve(&device.0, || host.device_by_id(&id), default)
            .ok_or_else(|| BackendError::DeviceNotFound(device.clone()))
    }
}

/// An output as `listed_outputs` gives it: id, name, detail, device.
type Listed = (DeviceId, String, Option<String>, cpal::Device);

/// The AAudio outputs under their stable ids and labels (mobile spec MB5).
#[cfg(target_os = "android")]
fn aaudio_listed(devices: Vec<cpal::Device>) -> Vec<Listed> {
    use crate::aaudio_ids::{ListedOutput, identities};
    let outputs: Vec<ListedOutput> = devices
        .iter()
        .map(|d| {
            let description = d.description().ok();
            ListedOutput {
                name: description
                    .as_ref()
                    .map(|x| x.name().to_owned())
                    .unwrap_or_default(),
                device_type: description
                    .as_ref()
                    .map_or(cpal::DeviceType::Unknown, |x| x.device_type()),
                interface: description
                    .as_ref()
                    .map_or(cpal::InterfaceType::Unknown, |x| x.interface_type()),
                address: description
                    .as_ref()
                    .and_then(|x| x.address().map(str::to_owned)),
            }
        })
        .collect();
    identities(&outputs)
        .into_iter()
        .zip(devices)
        .map(|((id, label), device)| (id, label, None, device))
        .collect()
}

/// The device listed under `wanted`, if any.
#[cfg(any(target_os = "android", test))]
fn pick<D>(wanted: &str, listed: Vec<(DeviceId, D)>) -> Option<D> {
    listed
        .into_iter()
        .find(|(id, _)| id.0 == wanted)
        .map(|(_, device)| device)
}

/// The device named `wanted`: from the device list, or the host's default
/// device when `wanted` is its id. cpal's AAudio host names its default
/// device `aaudio:-1`, which `device_by_id` cannot find; route-less players
/// went to Null on Android because of it (mobile spec §11). The default is
/// only asked for when the list has no such device.
fn resolve<D>(
    wanted: &str,
    by_id: impl FnOnce() -> Option<D>,
    default: impl FnOnce() -> Option<(String, D)>,
) -> Option<D> {
    by_id().or_else(|| {
        default()
            .filter(|(id, _)| id == wanted)
            .map(|(_, device)| device)
    })
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
        for (id, name, detail, device) in self.listed_outputs(&host)? {
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
            let exclusive = exclusive_capable(&self.id().0, &id.0);
            let native_dsd = self.native_dsd_capable(&id.0);
            list.push(DeviceInfo {
                id,
                name,
                detail,
                channels,
                sample_rates,
                buffer_frames,
                exclusive_capable: exclusive,
                rate_switching: exclusive,
                native_dsd,
                shared: shared_output(&self.id().0, &id.to_string(), false),
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

/// Refuses the DSD streams this backend cannot carry: native DSD, and DSD
/// over a shared stream, where the system mixer would alter the words.
fn check_dsd(config: &StreamConfig) -> Result<(), BackendError> {
    match config.dsd {
        None => Ok(()),
        Some(DsdStream::Native) => Err(BackendError::Unsupported(
            "native DSD is not available on this system".to_owned(),
        )),
        Some(DsdStream::Dop) if !config.exclusive => Err(BackendError::Unsupported(
            "DSD needs exclusive access".to_owned(),
        )),
        Some(DsdStream::Dop) => Ok(()),
    }
}

#[cfg(target_os = "linux")]
impl CpalBackend {
    /// Whether an ALSA `hw:` device takes a DSD format. A device that cannot
    /// be probed (busy, including by our own stream) keeps its last result.
    fn native_dsd_capable(&self, device_id: &str) -> bool {
        if self.host_id != cpal::HostId::Alsa {
            return false;
        }
        let Some(name) = crate::alsa_dsd::pcm_name(device_id) else {
            return false;
        };
        let mut cache = self
            .native_probe
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        match crate::alsa_dsd::probe(name) {
            Ok(Some(format)) => {
                cache.insert(device_id.to_owned(), format);
                true
            }
            Ok(None) => {
                cache.remove(device_id);
                false
            }
            Err(_) => cache.contains_key(device_id),
        }
    }

    fn open_native(
        &self,
        device: &DeviceId,
        config: StreamConfig,
        renderer: Box<dyn Renderer>,
        errors: Arc<dyn StreamErrorSink>,
    ) -> Result<Box<dyn OutputStream>, BackendError> {
        let name = match crate::alsa_dsd::pcm_name(&device.0) {
            Some(name) if self.host_id == cpal::HostId::Alsa && config.exclusive => name,
            _ => {
                return Err(BackendError::Unsupported(
                    "native DSD is not available on this device".to_owned(),
                ));
            }
        };
        let cached = self
            .native_probe
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&device.0)
            .copied();
        crate::alsa_dsd::open(device, name, cached, config, renderer, errors)
    }
}

#[cfg(target_os = "linux")]
impl CpalBackend {
    /// A promoter for the PulseAudio host, whose callback runs on the
    /// client's thread at normal priority (cpal promotes the others). A
    /// refusal is reported as `RealtimeDenied`: the stream still plays.
    fn realtime_promoter(
        &self,
        errors: &Arc<dyn StreamErrorSink>,
        frames: u32,
        rate: u32,
    ) -> Option<crate::rt_promote::Promoter<audio_thread_priority::RtPriorityThreadInfo>> {
        if self.id().0 != "pulseaudio" {
            return None;
        }
        let errors = Arc::clone(errors);
        let promoted = crate::rt_promote::Promoter::spawn(move |thread| {
            match audio_thread_priority::promote_thread_to_real_time(thread, frames, rate) {
                Ok(_) => tracing::info!("the PulseAudio client thread runs in real time"),
                Err(error) => {
                    tracing::warn!(%error, "the PulseAudio client thread stays at normal priority");
                    errors.report(StreamErrorKind::RealtimeDenied);
                }
            }
        });
        promoted
            .inspect_err(|error| tracing::warn!(%error, "cannot start the real-time helper"))
            .ok()
    }
}

#[cfg(not(target_os = "linux"))]
impl CpalBackend {
    fn native_dsd_capable(&self, _device_id: &str) -> bool {
        false
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
        if config.dsd == Some(DsdStream::Native) {
            #[cfg(target_os = "linux")]
            return self.open_native(device, config, renderer, errors);
            #[cfg(not(target_os = "linux"))]
            return Err(BackendError::Unsupported(
                "native DSD is not available on this system".to_owned(),
            ));
        }
        check_dsd(&config)?;
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
                #[cfg(target_os = "macos")]
                "coreaudio" => {}
                _ => {
                    return Err(BackendError::Unsupported(
                        "exclusive access is not available on this system".to_owned(),
                    ));
                }
            }
        }
        let host = self.host()?;
        let dev = self.find(&host, device)?;
        #[cfg(target_os = "macos")]
        let mut hog = if config.exclusive {
            let name = dev
                .description()
                .map(|d| d.name().to_owned())
                .map_err(backend_error)?;
            Some(crate::coreaudio_hog::take(&name)?)
        } else {
            None
        };
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
        let format = if config.dsd == Some(DsdStream::Dop) {
            choose_dop_sample_format(&formats).ok_or_else(|| {
                BackendError::Unsupported("DoP needs a 24- or 32-bit integer format".to_owned())
            })?
        } else {
            choose_sample_format(&formats).unwrap_or(cpal::SampleFormat::F32)
        };
        let ranges: Vec<(u32, u32)> = matching
            .iter()
            .filter(|c| c.sample_format() == format)
            .filter_map(|c| match *c.buffer_size() {
                cpal::SupportedBufferSize::Range { min, max } => Some((min, max)),
                cpal::SupportedBufferSize::Unknown => None,
            })
            .collect();
        let shared = shared_output(&self.id().0, &device.0, config.exclusive);
        let wanted = wanted_frames(&config, shared, &ranges);
        let attempts = buffer_attempts(&ranges, wanted, config.exact_buffer);
        let scratch_frames = (wanted as usize).max(4096);
        let mut renderer = Some(renderer);
        // Only reached without an attempt: an exact buffer out of range.
        let mut last = BackendError::Unsupported(format!(
            "a buffer of {} frames is outside the device's range",
            config.buffer_frames
        ));
        for buffer_size in attempts {
            let cpal_config = cpal::StreamConfig {
                channels: config.channels,
                sample_rate: config.sample_rate,
                buffer_size,
            };
            // The renderer is handed to the callback only once the stream
            // exists, so a failed attempt never loses it.
            let (mut handoff, receiver) = rtrb::RingBuffer::<Box<dyn Renderer>>::new(1);
            #[cfg(target_os = "linux")]
            let promoter = self.realtime_promoter(&errors, wanted, config.sample_rate);
            let errors = errors.clone();
            let fixed_frames = match buffer_size {
                cpal::BufferSize::Fixed(frames) => Some(frames),
                cpal::BufferSize::Default => None,
            };
            let called_frames = Arc::new(AtomicU32::new(0));
            let held_frames = Arc::new(AtomicU32::new(0));
            let io = Io {
                handoff: receiver,
                errors,
                scratch_frames,
                called_frames: called_frames.clone(),
                held_frames: held_frames.clone(),
                #[cfg(target_os = "linux")]
                promoter,
            };
            let built = match format {
                cpal::SampleFormat::I16 => build::<i16>(&dev, cpal_config, io),
                cpal::SampleFormat::I32 => build::<i32>(&dev, cpal_config, io),
                cpal::SampleFormat::I24 => build::<cpal::I24>(&dev, cpal_config, io),
                _ => build::<f32>(&dev, cpal_config, io),
            };
            match built {
                Ok(stream) => {
                    if let Some(r) = renderer.take() {
                        let _ = handoff.push(r);
                    }
                    #[allow(unused_mut)]
                    let mut format = match format {
                        cpal::SampleFormat::I16 => SampleFormat::I16,
                        cpal::SampleFormat::I32 => SampleFormat::I32,
                        cpal::SampleFormat::I24 => SampleFormat::I24,
                        _ => SampleFormat::F32,
                    };
                    #[allow(unused_mut)]
                    let mut held: Option<Box<dyn Send>> = None;
                    // In hog mode the hardware format is set after cpal built
                    // its stream (cpal sets a float physical format of its
                    // own), and what reaches the device is that format.
                    #[cfg(target_os = "macos")]
                    if let Some(guard) = hog.take() {
                        format = guard.prepare(config.sample_rate, u32::from(config.channels))?;
                        held = Some(Box::new(guard));
                    }
                    stream.play().map_err(backend_error)?;
                    return Ok(Box::new(CpalStream {
                        config,
                        fixed_frames,
                        called_frames,
                        held_frames,
                        format,
                        _stream: stream,
                        _exclusive: held,
                    }));
                }
                Err(e) => match e.kind() {
                    cpal::ErrorKind::DeviceNotAvailable => {
                        return Err(BackendError::DeviceNotFound(device.clone()));
                    }
                    cpal::ErrorKind::UnsupportedConfig => {
                        last = BackendError::Unsupported(e.to_string())
                    }
                    // Held by another client for now (ALSA's EBUSY or
                    // EAGAIN, WASAPI's device in use): not a refusal.
                    cpal::ErrorKind::DeviceBusy => {
                        return Err(BackendError::Busy(e.to_string()));
                    }
                    _ => return Err(backend_error(e)),
                },
            }
        }
        Err(last)
    }
}

/// What a stream's callback takes with it.
struct Io {
    /// Where the renderer arrives once the stream exists.
    handoff: rtrb::Consumer<Box<dyn Renderer>>,
    errors: Arc<dyn StreamErrorSink>,
    scratch_frames: usize,
    /// The largest block called back, for `OutputStream::config`.
    called_frames: Arc<AtomicU32>,
    /// The most frames held, for `OutputStream::latency_frames`.
    held_frames: Arc<AtomicU32>,
    /// Promotes the callback's thread when the host leaves it at normal
    /// priority (`realtime_promoter`).
    #[cfg(target_os = "linux")]
    promoter: Option<crate::rt_promote::Promoter<audio_thread_priority::RtPriorityThreadInfo>>,
}

/// Builds an output stream in sample type `T`. The callback renders f32 into
/// a scratch buffer allocated here (never in the callback) and converts.
fn build<T>(
    dev: &cpal::Device,
    config: cpal::StreamConfig,
    io: Io,
) -> Result<cpal::Stream, cpal::Error>
where
    T: OutputSample + Send + 'static,
{
    let Io {
        mut handoff,
        errors,
        scratch_frames,
        called_frames,
        held_frames: held,
        #[cfg(target_os = "linux")]
        mut promoter,
    } = io;
    let channels = usize::from(config.channels).max(1);
    let rate = config.sample_rate;
    let mut scratch = vec![0.0f32; scratch_frames.max(1) * channels];
    let mut renderer: Option<Box<dyn Renderer>> = None;
    dev.build_output_stream::<T, _, _>(
        config,
        move |out: &mut [T], info: &cpal::OutputCallbackInfo| {
            // Once: the thread's ids, read with plain system calls.
            #[cfg(target_os = "linux")]
            if let Some(p) = promoter.as_mut() {
                p.offer(|| audio_thread_priority::get_current_thread_info().ok());
            }
            let frames = u32::try_from(out.len() / channels).unwrap_or(u32::MAX);
            called_frames.fetch_max(frames, Ordering::Relaxed);
            let stamp = info.timestamp();
            let queued = stamp.playback.duration_since(stamp.callback);
            held.fetch_max(held_frames(queued, frames, rate), Ordering::Relaxed);
            if renderer.is_none() {
                renderer = handoff.pop().ok();
            }
            match renderer.as_mut() {
                Some(r) => render_converted(r.as_mut(), out, channels, &mut scratch),
                None => out.fill(T::EQUILIBRIUM),
            }
        },
        move |err: cpal::Error| report_stream_error(errors.as_ref(), &err),
        None,
    )
}

/// DoP needs every bit of a 24-bit word: only 24- or 32-bit integer.
fn choose_dop_sample_format(formats: &[cpal::SampleFormat]) -> Option<cpal::SampleFormat> {
    [cpal::SampleFormat::I32, cpal::SampleFormat::I24]
        .into_iter()
        .find(|f| formats.contains(f))
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

/// A device sample format and its conversion from the mixer's f32.
///
/// Integer formats use one convention: a float in [-1, 1) maps to the
/// integer of `n` bits by `x * 2^(n-1)`, rounded to the nearest step (half
/// away from zero) and clamped to `[-2^(n-1), 2^(n-1) - 1]`. So integer PCM
/// that entered as `v / 2^(n-1)` comes out as exactly `v`, +1.0 and above
/// clip to the positive maximum, -1.0 is the negative minimum and NaN is 0.
/// Float formats take the sample as it is. Real-time safe: no allocation,
/// no branches that can panic.
trait OutputSample: cpal::SizedSample {
    fn from_f32(s: f32) -> Self;
}

impl OutputSample for f32 {
    fn from_f32(s: f32) -> Self {
        s
    }
}

impl OutputSample for i16 {
    fn from_f32(s: f32) -> Self {
        // `as` saturates and maps NaN to 0.
        (s * 32_768.0).round() as i16
    }
}

impl OutputSample for i32 {
    fn from_f32(s: f32) -> Self {
        (s * 2_147_483_648.0).round() as i32
    }
}

impl OutputSample for cpal::I24 {
    fn from_f32(s: f32) -> Self {
        let v = ((s * 8_388_608.0).round() as i32).clamp(-(1 << 23), (1 << 23) - 1);
        cpal::I24::new(v).unwrap_or(<cpal::I24 as cpal::Sample>::EQUILIBRIUM)
    }
}

/// Renders `out` through `scratch` in whole-frame pieces and converts each
/// sample to the device format (`OutputSample`: clipping, rounding). Real-time safe: no allocation.
fn render_converted<T>(
    renderer: &mut dyn Renderer,
    out: &mut [T],
    channels: usize,
    scratch: &mut [f32],
) where
    T: OutputSample,
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
            *o = T::from_f32(*s);
        }
    }
}

/// The first description line beyond the name, else the device's address:
/// ALSA lists every output profile of a card under the card's name, and
/// this line names the profile.
fn device_detail(d: &cpal::DeviceDescription) -> Option<String> {
    let adds = |line: &&str| !line.is_empty() && *line != d.name();
    d.extended()
        .map(str::trim)
        .find(adds)
        .or_else(|| d.address().map(str::trim).filter(adds))
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::{
        HostCache, OutputSample, check_dsd, choose_buffer_frames, choose_dop_sample_format,
        choose_sample_format, classify, device_detail, exclusive_capable, pick, render_converted,
        resolve,
    };
    use crate::{DeviceId, Renderer, StreamErrorKind};
    use cpal::DeviceDescriptionBuilder;

    #[test]
    fn the_host_default_id_resolves_to_the_default_device() {
        let found = resolve("aaudio:-1", || None, || Some(("aaudio:-1".to_owned(), 7)));
        assert_eq!(found, Some(7));
    }

    #[test]
    fn another_id_goes_through_the_device_list() {
        let found = resolve("aaudio:5", || Some(5), || Some(("aaudio:-1".to_owned(), 7)));
        assert_eq!(found, Some(5));
    }

    #[test]
    fn a_stable_id_is_found_among_the_listed_devices() {
        let listed = vec![
            (DeviceId("aaudio:builtin/speaker/P".to_owned()), 1),
            (DeviceId("aaudio:bluetooth/unknown/JBL".to_owned()), 2),
        ];
        assert_eq!(pick("aaudio:bluetooth/unknown/JBL", listed), Some(2));
    }

    #[test]
    fn an_unknown_stable_id_is_not_found() {
        let listed = vec![(DeviceId("aaudio:builtin/speaker/P".to_owned()), 1)];
        assert_eq!(pick("aaudio:bluetooth/unknown/JBL", listed), None);
    }

    #[test]
    fn an_unknown_id_that_is_not_the_default_is_not_found() {
        let found = resolve("aaudio:9", || None, || Some(("aaudio:-1".to_owned(), 7)));
        assert_eq!(found, None);
    }

    #[test]
    fn the_detail_is_the_first_extended_line_beyond_the_name() {
        let d = DeviceDescriptionBuilder::new("HDA Intel PCH")
            .extended(["HDA Intel PCH", "", "5.1 Surround output"])
            .address("hw:0,0")
            .build();
        assert_eq!(device_detail(&d).as_deref(), Some("5.1 Surround output"));
    }

    #[test]
    fn without_extended_lines_the_detail_is_the_address() {
        let d = DeviceDescriptionBuilder::new("Speakers")
            .address("USB 2-1.4")
            .build();
        assert_eq!(device_detail(&d).as_deref(), Some("USB 2-1.4"));
        let same = DeviceDescriptionBuilder::new("Speakers")
            .address("Speakers")
            .build();
        assert_eq!(device_detail(&same), None);
    }

    /// Records what a sink was told.
    #[derive(Default)]
    struct Told(std::sync::Mutex<Vec<StreamErrorKind>>);

    impl crate::StreamErrorSink for Told {
        fn report(&self, kind: StreamErrorKind) {
            self.0.lock().unwrap().push(kind);
        }
    }

    // A8: an unknown error reaches the sink as `Other` (the bus counts it
    // apart from xruns and the conductor logs it).
    #[test]
    fn an_unclassified_error_is_counted_through_the_sink_as_other() {
        let told = Told::default();
        super::report_stream_error(&told, &cpal::Error::new(cpal::ErrorKind::BackendError));
        super::report_stream_error(&told, &cpal::Error::new(cpal::ErrorKind::Xrun));
        assert_eq!(
            *told.0.lock().unwrap(),
            vec![StreamErrorKind::Other, StreamErrorKind::Xrun]
        );
    }

    #[test]
    fn only_xrun_is_an_xrun_and_unknown_errors_are_other() {
        assert_eq!(classify(cpal::ErrorKind::Xrun), StreamErrorKind::Xrun);
        for kind in [
            cpal::ErrorKind::BackendError,
            cpal::ErrorKind::DeviceBusy,
            cpal::ErrorKind::ResourceExhausted,
            cpal::ErrorKind::Other,
        ] {
            assert_eq!(classify(kind), StreamErrorKind::Other, "{kind}");
        }
    }

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

    // A7: cpal's own conversions wrap at 24 bits and truncate at 16.
    #[test]
    fn a7_the_24_bit_conversion_clips_instead_of_wrapping() {
        for (s, want) in [
            (1.0f32, (1 << 23) - 1),
            (1.5, (1 << 23) - 1),
            (-1.0, -(1 << 23)),
            (-1.5, -(1 << 23)),
            (f32::INFINITY, (1 << 23) - 1),
            (f32::NAN, 0),
        ] {
            assert_eq!(cpal::I24::from_f32(s).inner(), want, "{s}");
        }
    }

    #[test]
    fn a7_the_16_bit_conversion_rounds_to_the_nearest_step() {
        for (s, nearest) in [(100.75f32, 101i16), (-100.75, -101), (0.6, 1), (-0.4, 0)] {
            assert_eq!(i16::from_f32(s / 32_768.0), nearest, "{s} steps");
        }
        assert_eq!(i16::from_f32(1.0), i16::MAX);
        assert_eq!(i16::from_f32(-1.0), i16::MIN);
        assert_eq!(i16::from_f32(f32::NAN), 0);
        assert_eq!(i32::from_f32(1.5), i32::MAX);
        assert_eq!(i32::from_f32(-1.5), i32::MIN);
        assert_eq!(i32::from_f32(f32::NAN), 0);
        assert_eq!(f32::from_f32(1.5), 1.5);
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
    fn a_dsd_stream_needs_exclusive_access_and_never_goes_native() {
        use crate::StreamConfig;
        use crate::dsd::DsdStream;
        let config = |dsd, exclusive| StreamConfig {
            sample_rate: 176_400,
            buffer_frames: 512,
            channels: 2,
            exclusive,
            dsd,
            exact_buffer: false,
            shared_buffer_us: 0,
        };
        assert!(check_dsd(&config(Some(DsdStream::Dop), false)).is_err());
        assert!(check_dsd(&config(Some(DsdStream::Dop), true)).is_ok());
        assert!(check_dsd(&config(Some(DsdStream::Native), true)).is_err());
        assert!(check_dsd(&config(None, false)).is_ok());
    }

    #[test]
    fn dop_uses_only_24_or_32_bit_integer_formats() {
        use cpal::SampleFormat::{F32, I16, I24, I32};
        assert_eq!(choose_dop_sample_format(&[F32, I32, I16]), Some(I32));
        assert_eq!(choose_dop_sample_format(&[F32, I24]), Some(I24));
        assert_eq!(choose_dop_sample_format(&[F32, I16]), None);
    }

    #[test]
    fn dop_samples_convert_to_the_exact_integer_words() {
        use crate::dsd::{DOP_MARKERS, DSD_SILENCE, dop_sample, word_to_sample};
        let words = [
            (0x00, 0x00),
            (0xFF, 0xFF),
            (DSD_SILENCE, DSD_SILENCE),
            (0x80, 0x00),
            (0x7F, 0xFF),
        ];
        for marker in DOP_MARKERS {
            for (a, b) in words {
                let s = dop_sample(marker, word_to_sample(a, b));
                let word = (i32::from(marker) << 16) | (i32::from(a) << 8) | i32::from(b);
                let signed = (word << 8) >> 8;
                assert_eq!(<cpal::I24 as OutputSample>::from_f32(s).inner(), signed);
                assert_eq!(<i32 as OutputSample>::from_f32(s), signed << 8);
            }
        }
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

    #[test]
    fn outputs_with_a_mixer_between_them_and_the_hardware_are_shared() {
        use super::shared_output;
        assert!(shared_output(
            "pulseaudio",
            "pulseaudio:alsa_output.pci",
            false
        ));
        assert!(shared_output("pipewire", "pipewire:42", false));
        assert!(shared_output("alsa", "alsa:default", false));
        assert!(shared_output("alsa", "alsa:pipewire", false));
        assert!(shared_output("alsa", "alsa:dmix:CARD=PCH,DEV=0", false));
        assert!(!shared_output("alsa", "alsa:hw:CARD=PCH,DEV=0", false));
        assert!(!shared_output("alsa", "alsa:plughw:CARD=PCH,DEV=0", false));
        // Usually the card's stock `default`: `plug` into `dmix`.
        assert!(shared_output("alsa", "alsa:sysdefault:CARD=PCH", false));
        assert!(!shared_output("alsa", "alsa:front:CARD=PCH,DEV=0", false));
        assert!(shared_output("wasapi", "wasapi:{guid}", false));
        assert!(
            !shared_output("wasapi", "wasapi:{guid}", true),
            "exclusive mode"
        );
        // The callback is the hardware's (or the JACK server's) own cycle.
        assert!(!shared_output("coreaudio", "coreaudio:BuiltIn", false));
        assert!(!shared_output("jack", "jack:system", false));
        assert!(!shared_output("asio", "asio:Driver", false));
    }

    #[test]
    fn a_shared_output_asks_for_at_least_the_shared_buffer() {
        use super::wanted_frames;
        use crate::StreamConfig;
        let config = |sample_rate, buffer_frames, exact_buffer| StreamConfig {
            sample_rate,
            buffer_frames,
            channels: 2,
            exclusive: false,
            dsd: None,
            exact_buffer,
            shared_buffer_us: 50_000,
        };
        let any = [(1, 262_144)];
        let wanted =
            |c: &StreamConfig, shared, ranges: &[(u32, u32)]| wanted_frames(c, shared, ranges);
        assert_eq!(wanted(&config(48_000, 512, false), true, &any), 2_400);
        assert_eq!(wanted(&config(44_100, 512, false), true, &any), 2_205);
        assert_eq!(wanted(&config(44_100, 4_096, false), true, &any), 4_096);
        assert_eq!(
            wanted(&config(48_000, 512, false), false, &any),
            512,
            "hardware"
        );
        assert_eq!(
            wanted(&config(48_000, 256, true), true, &any),
            256,
            "a device's own buffer is used as it is"
        );
        let off = StreamConfig {
            shared_buffer_us: 0,
            ..config(48_000, 512, false)
        };
        assert_eq!(wanted(&off, true, &any), 512, "no floor");
        // A floor past the largest size the device takes (a PipeWire
        // quantum) is that size, not a refused one that would leave the
        // device to choose.
        let quantum = [(32, 2_048)];
        assert_eq!(wanted(&config(48_000, 512, false), true, &quantum), 2_048);
        assert_eq!(wanted(&config(48_000, 4_096, false), true, &quantum), 4_096);
        assert_eq!(
            wanted(&config(48_000, 512, false), true, &[]),
            2_400,
            "no range reported"
        );
    }

    #[test]
    fn an_exact_buffer_is_never_replaced_by_the_device_default() {
        use super::buffer_attempts;
        use cpal::BufferSize::{Default, Fixed};
        let range = [(64, 4096)];
        assert_eq!(buffer_attempts(&range, 512, false), [Fixed(512), Default]);
        assert_eq!(buffer_attempts(&[(1024, 4096)], 512, false), [Default]);
        assert_eq!(buffer_attempts(&range, 512, true), [Fixed(512)]);
        assert!(
            buffer_attempts(&[(1024, 4096)], 512, true).is_empty(),
            "outside every reported range: refused"
        );
        assert_eq!(
            buffer_attempts(&[], 512, true),
            [Fixed(512)],
            "no range reported: the device decides"
        );
    }

    #[test]
    fn the_reported_buffer_size_is_the_one_in_use() {
        use super::frames_in_use;
        assert_eq!(frames_in_use(512, Some(256), 0), 256, "a fixed size");
        assert_eq!(
            frames_in_use(512, None, 0),
            512,
            "default, before a callback"
        );
        assert_eq!(
            frames_in_use(512, None, 1_024),
            1_024,
            "default, as called back"
        );
    }

    #[test]
    fn a_callback_holds_what_was_queued_before_it_and_its_own_block() {
        use super::held_frames;
        use std::time::Duration;
        // 1 151 frames queued at 44.1 kHz (26.1 ms), then a 512-frame block.
        let queued = Duration::from_nanos(1_151 * 1_000_000_000 / 44_100);
        assert_eq!(held_frames(queued, 512, 44_100), 1_151 + 512);
        assert_eq!(held_frames(Duration::ZERO, 512, 96_000), 512);
        assert_eq!(
            held_frames(Duration::from_secs(u64::MAX), 512, 96_000),
            u32::MAX,
            "saturates"
        );
    }
}
