//! WASAPI exclusive mode (Phase 4 plan 2), through the `wasapi` crate's
//! safe API. The stream runs on its own thread: it opens the device,
//! negotiates the format and initialises the client there (COM objects stay
//! on the thread that created them), reports the outcome to `open`, then
//! renders on every buffer event with buffers allocated beforehand.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

use wasapi::{DeviceEnumerator, Direction, SampleType, StreamMode, WaveFormat};

use crate::exclusive::{Candidate, bytes_per_sample, negotiate, write_samples};
use crate::{
    BackendError, DeviceId, OutputStream, Renderer, SampleFormat, StreamConfig, StreamErrorKind,
    StreamErrorSink,
};

/// How long `open` waits for the render thread to report.
const OPEN_TIMEOUT: Duration = Duration::from_secs(5);
/// A buffer event later than this means the device stopped.
const EVENT_TIMEOUT_MS: u32 = 2_000;

struct ExclusiveStream {
    config: StreamConfig,
    format: SampleFormat,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl OutputStream for ExclusiveStream {
    fn config(&self) -> StreamConfig {
        self.config
    }

    fn sample_format(&self) -> SampleFormat {
        self.format
    }
}

impl Drop for ExclusiveStream {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn wave_format(candidate: Candidate, config: StreamConfig) -> WaveFormat {
    let sample_type = if candidate.format == SampleFormat::F32 {
        SampleType::Float
    } else {
        SampleType::Int
    };
    WaveFormat::new(
        usize::from(candidate.container_bits),
        usize::from(candidate.valid_bits),
        &sample_type,
        config.sample_rate as usize,
        usize::from(config.channels),
        None,
    )
}

fn unsupported(e: impl std::fmt::Display) -> BackendError {
    BackendError::Unsupported(e.to_string())
}

/// Everything the render loop needs, created on the render thread.
struct Opened {
    client: wasapi::AudioClient,
    render: wasapi::AudioRenderClient,
    event: wasapi::Handle,
    format: SampleFormat,
    buffer_frames: usize,
}

fn open_on_thread(endpoint: &str, config: StreamConfig) -> Result<Opened, BackendError> {
    let _ = wasapi::initialize_mta();
    let enumerator = DeviceEnumerator::new().map_err(unsupported)?;
    let device = enumerator
        .get_device(endpoint)
        .map_err(|_| BackendError::DeviceNotFound(DeviceId(endpoint.to_owned())))?;
    let mut client = device.get_iaudioclient().map_err(unsupported)?;
    let mut accepted: Option<WaveFormat> = None;
    let candidate =
        negotiate(
            |c| match client.is_supported_exclusive_with_quirks(&wave_format(c, config)) {
                Ok(format) => {
                    accepted = Some(format);
                    true
                }
                Err(_) => false,
            },
        )
        .ok_or_else(|| unsupported("the device takes none of the exclusive formats"))?;
    let format = accepted.ok_or_else(|| unsupported("no format"))?;
    let wanted = wasapi::calculate_period_100ns(
        i64::from(config.buffer_frames.max(1)),
        i64::from(config.sample_rate.max(1)),
    );
    let period = client
        .calculate_aligned_period_near(wanted, Some(128), &format)
        .map_err(unsupported)?;
    let mode = StreamMode::EventsExclusive { period_hns: period };
    if client
        .initialize_client(&format, &Direction::Render, &mode)
        .is_err()
    {
        // Some drivers want the period aligned to their own buffer size:
        // the documented recovery is a new client with that size.
        let frames = client.get_buffer_size().map_err(unsupported)?;
        let aligned =
            wasapi::calculate_period_100ns(i64::from(frames), i64::from(config.sample_rate));
        client = device.get_iaudioclient().map_err(unsupported)?;
        client
            .initialize_client(
                &format,
                &Direction::Render,
                &StreamMode::EventsExclusive {
                    period_hns: aligned,
                },
            )
            .map_err(unsupported)?;
    }
    let event = client.set_get_eventhandle().map_err(unsupported)?;
    let render = client.get_audiorenderclient().map_err(unsupported)?;
    let buffer_frames = client.get_buffer_size().map_err(unsupported)? as usize;
    Ok(Opened {
        client,
        render,
        event,
        format: candidate.format,
        buffer_frames,
    })
}

/// The render loop: preallocated buffers, no allocation or logging inside.
fn render_loop(
    opened: &Opened,
    channels: usize,
    renderer: &mut dyn Renderer,
    errors: &dyn StreamErrorSink,
    stop: &AtomicBool,
) {
    let samples = opened.buffer_frames.max(1) * channels;
    let mut scratch = vec![0.0f32; samples];
    let mut bytes = vec![0u8; samples * bytes_per_sample(opened.format)];
    if opened.client.start_stream().is_err() {
        errors.report(StreamErrorKind::DeviceLost);
        return;
    }
    while !stop.load(Ordering::Acquire) {
        if opened.event.wait_for_event(EVENT_TIMEOUT_MS).is_err() {
            errors.report(StreamErrorKind::DeviceLost);
            break;
        }
        let Ok(frames) = opened.client.get_available_space_in_frames() else {
            errors.report(StreamErrorKind::DeviceLost);
            break;
        };
        let frames = (frames as usize).min(opened.buffer_frames);
        let (Some(out), Some(raw)) = (
            scratch.get_mut(..frames * channels),
            bytes.get_mut(..frames * channels * bytes_per_sample(opened.format)),
        ) else {
            continue;
        };
        renderer.render(out, channels);
        write_samples(opened.format, out, raw);
        if opened.render.write_to_device(frames, raw, None).is_err() {
            errors.report(StreamErrorKind::DeviceLost);
            break;
        }
    }
    let _ = opened.client.stop_stream();
}

/// Opens `device` (cpal's `wasapi:<endpoint id>`) in exclusive mode.
pub(crate) fn open(
    device: &DeviceId,
    config: StreamConfig,
    mut renderer: Box<dyn Renderer>,
    errors: Arc<dyn StreamErrorSink>,
) -> Result<Box<dyn OutputStream>, BackendError> {
    let endpoint = device
        .0
        .strip_prefix("wasapi:")
        .unwrap_or(&device.0)
        .to_owned();
    let stop = Arc::new(AtomicBool::new(false));
    let flag = stop.clone();
    let (tx, rx) = crossbeam_channel::bounded(1);
    let channels = usize::from(config.channels.max(1));
    let thread = std::thread::Builder::new()
        .name("fp-wasapi-render".to_owned())
        .spawn(move || match open_on_thread(&endpoint, config) {
            Ok(opened) => {
                let _ = tx.send(Ok(opened.format));
                render_loop(&opened, channels, renderer.as_mut(), errors.as_ref(), &flag);
            }
            Err(e) => {
                let _ = tx.send(Err(e));
            }
        })
        .map_err(|e| BackendError::Backend(e.to_string()))?;
    let format = match rx.recv_timeout(OPEN_TIMEOUT) {
        Ok(Ok(format)) => format,
        Ok(Err(e)) => {
            let _ = thread.join();
            return Err(e);
        }
        Err(_) => {
            stop.store(true, Ordering::Release);
            return Err(BackendError::Backend(
                "the exclusive stream did not open in time".to_owned(),
            ));
        }
    };
    Ok(Box::new(ExclusiveStream {
        config,
        format,
        stop,
        thread: Some(thread),
    }))
}
