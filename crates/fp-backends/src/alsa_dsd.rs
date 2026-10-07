//! Native DSD through ALSA (feedback 2 spec O25, Linux). cpal cannot open a
//! PCM in a DSD format, so a `hw:` device that takes one is driven here
//! with the `alsa` crate's safe API. The stream runs on its own thread: it
//! opens the PCM and sets the hardware parameters there, reports the
//! outcome to `open`, then renders and writes one period at a time with
//! buffers allocated beforehand.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread::JoinHandle;
use std::time::Duration;

use alsa::pcm::{Access, Format, Frames, HwParams, PCM};
use alsa::{Direction, ValueOr};

use crate::dsd::{DsdStream, NativeDsdFormat, choose_native_format, pack_native};
use crate::{
    BackendError, DeviceId, OutputStream, Renderer, SampleFormat, StreamConfig, StreamErrorKind,
    StreamErrorSink,
};

/// How long `open` waits for the stream thread to report.
const OPEN_TIMEOUT: Duration = Duration::from_secs(5);
/// Device periods in the ring buffer: enough slack that one late period is
/// not an underrun.
const PERIODS_PER_BUFFER: u32 = 4;
/// `EPIPE`: an underrun.
const EPIPE: i32 = 32;
/// `EAGAIN`: a non-blocking write found no room.
const EAGAIN: i32 = 11;
/// The shortest wait for room in the device buffer.
const MIN_WAIT_MS: u32 = 20;
/// A wait lasts this many device periods (and at least `MIN_WAIT_MS`).
const WAIT_PERIODS: u64 = 4;

/// The ALSA PCM name of a cpal ALSA device id (`alsa:hw:CARD=D,DEV=0` →
/// `hw:CARD=D,DEV=0`); `None` for anything but `hw:`.
pub(crate) fn pcm_name(device_id: &str) -> Option<&str> {
    let inner = device_id.strip_prefix("alsa:").unwrap_or(device_id);
    // A NUL cannot be in a PCM name (and would make the alsa crate panic).
    (inner.starts_with("hw:") && !inner.contains('\0')).then_some(inner)
}

pub(crate) fn alsa_format(format: NativeDsdFormat) -> Format {
    match format {
        NativeDsdFormat::U32Be => Format::DSDU32BE,
        NativeDsdFormat::U32Le => Format::DSDU32LE,
        NativeDsdFormat::U16Be => Format::DSDU16BE,
        NativeDsdFormat::U16Le => Format::DSDU16LE,
        NativeDsdFormat::U8 => Format::DSDU8,
    }
}

/// Word frames to render per device period: even, and at least 2.
pub(crate) fn words_per_period(format: NativeDsdFormat, device_period: usize) -> usize {
    let words = device_period.saturating_mul(format.bytes()) / 2;
    (words - words % 2).max(2)
}

/// The first DSD format the device accepts, tested on its hardware
/// parameters only. The PCM is opened non-blocking and dropped at once: a
/// busy device (including our own open stream) fails here and is never
/// disturbed. Nothing is prepared or written.
pub(crate) fn probe(pcm_name: &str) -> Result<Option<NativeDsdFormat>, String> {
    let pcm = PCM::new(pcm_name, Direction::Playback, true).map_err(|e| e.to_string())?;
    let hwp = HwParams::any(&pcm).map_err(|e| e.to_string())?;
    Ok(choose_native_format(|f| {
        hwp.test_format(alsa_format(f)).is_ok()
    }))
}

/// How long to wait for room in the device buffer before looking at the
/// stop flag again: a few periods, never less than `MIN_WAIT_MS`. This
/// bounds how long dropping the stream can take.
pub(crate) fn wait_timeout_ms(device_period: usize, device_rate: u32) -> u32 {
    let ms =
        (device_period as u64).saturating_mul(WAIT_PERIODS * 1000) / u64::from(device_rate.max(1));
    u32::try_from(ms).unwrap_or(u32::MAX).max(MIN_WAIT_MS)
}

fn unsupported(e: impl std::fmt::Display) -> BackendError {
    BackendError::Unsupported(e.to_string())
}

struct AlsaDsdStream {
    config: StreamConfig,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl OutputStream for AlsaDsdStream {
    fn config(&self) -> StreamConfig {
        self.config
    }

    /// The container only; nothing reads it for native DSD.
    fn sample_format(&self) -> SampleFormat {
        SampleFormat::I32
    }

    fn dsd(&self) -> Option<DsdStream> {
        Some(DsdStream::Native)
    }
}

impl Drop for AlsaDsdStream {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// An open, prepared PCM and what the loop needs to feed it.
struct Opened {
    pcm: PCM,
    format: NativeDsdFormat,
    device_rate: u32,
    device_period: usize,
}

fn open_pcm(
    name: &str,
    config: StreamConfig,
    cached: Option<NativeDsdFormat>,
) -> Result<Opened, BackendError> {
    // Non-blocking, and kept so: a busy device fails at once instead of
    // parking this thread in the kernel, and writes wait with a timeout.
    let pcm = PCM::new(name, Direction::Playback, true).map_err(|e| {
        let message = format!("cannot open {name}: {e}");
        // EBUSY, or EAGAIN from a non-blocking open: held by another client.
        match std::io::Error::from_raw_os_error(e.errno().abs()).kind() {
            std::io::ErrorKind::ResourceBusy | std::io::ErrorKind::WouldBlock => {
                BackendError::Busy(message)
            }
            _ => BackendError::Unsupported(message),
        }
    })?;
    let (format, device_rate, device_period) = {
        let hwp = HwParams::any(&pcm).map_err(unsupported)?;
        let format = cached
            .filter(|f| hwp.test_format(alsa_format(*f)).is_ok())
            .or_else(|| choose_native_format(|f| hwp.test_format(alsa_format(f)).is_ok()))
            .ok_or_else(|| unsupported("the device takes no DSD format"))?;
        hwp.set_access(Access::RWInterleaved).map_err(unsupported)?;
        hwp.set_format(alsa_format(format)).map_err(unsupported)?;
        hwp.set_channels(u32::from(config.channels))
            .map_err(unsupported)?;
        let wanted = format.device_rate(config.sample_rate);
        hwp.set_rate(wanted, ValueOr::Nearest)
            .map_err(unsupported)?;
        let rate = hwp.get_rate().map_err(unsupported)?;
        if rate != wanted {
            return Err(unsupported(format!(
                "the device runs at {rate} Hz, not the {wanted} Hz native DSD needs"
            )));
        }
        // Word frames to device frames: a device frame carries `bytes / 2`
        // word frames.
        let near = (u64::from(config.buffer_frames.max(1)) * 2 / format.bytes() as u64).max(1);
        let period = hwp
            .set_period_size_near(
                Frames::try_from(near).unwrap_or(Frames::MAX),
                ValueOr::Nearest,
            )
            .map_err(unsupported)?;
        hwp.set_buffer_size_near(period.saturating_mul(Frames::from(PERIODS_PER_BUFFER)))
            .map_err(unsupported)?;
        pcm.hw_params(&hwp).map_err(unsupported)?;
        (format, rate, usize::try_from(period).unwrap_or(1).max(1))
    };
    pcm.prepare().map_err(unsupported)?;
    Ok(Opened {
        pcm,
        format,
        device_rate,
        device_period,
    })
}

/// Renders and writes periods until `stop`. Real-time: after the buffers
/// exist nothing here allocates, locks, logs or panics.
fn run(
    opened: &Opened,
    channels: usize,
    renderer: &mut dyn Renderer,
    errors: &dyn StreamErrorSink,
    stop: &AtomicBool,
) {
    let words_len = words_per_period(opened.format, opened.device_period);
    let mut words = vec![0.0f32; words_len * channels];
    let mut bytes = vec![0u8; words_len * channels * 2];
    // Bytes in one device frame.
    let frame_bytes = channels * opened.format.bytes();
    let io = opened.pcm.io_bytes();
    let wait_ms = wait_timeout_ms(opened.device_period, opened.device_rate);
    // An error the loop recovers from (and counts); `false` ends the stream.
    let recovered = |e: alsa::Error| {
        match e.errno() {
            EAGAIN => return true,
            EPIPE => errors.report(StreamErrorKind::Xrun),
            _ => {}
        }
        // EPIPE is prepared again, a suspend is resumed, EINTR is ignored.
        if opened.pcm.try_recover(e, true).is_ok() {
            true
        } else {
            errors.report(StreamErrorKind::DeviceLost);
            false
        }
    };
    while !stop.load(Ordering::Acquire) {
        renderer.render(&mut words, channels);
        let total = pack_native(opened.format, &words, channels, &mut bytes);
        let mut done = 0;
        while done < total && !stop.load(Ordering::Acquire) {
            // Wait for room, so the stop flag is seen within a few periods
            // even when the device stops consuming.
            match opened.pcm.wait(Some(wait_ms)) {
                Ok(true) => {}
                Ok(false) => continue,
                Err(e) => {
                    if recovered(e) {
                        continue;
                    }
                    return;
                }
            }
            let Some(rest) = bytes.get(done..total) else {
                break;
            };
            match io.writei(rest) {
                Ok(frames) => done += frames * frame_bytes,
                Err(e) => {
                    if recovered(e) {
                        continue;
                    }
                    return;
                }
            }
        }
    }
}

/// Opens `pcm_name` for native DSD. `cached` is the format a probe found.
pub(crate) fn open(
    device: &DeviceId,
    pcm_name: &str,
    cached: Option<NativeDsdFormat>,
    config: StreamConfig,
    mut renderer: Box<dyn Renderer>,
    errors: Arc<dyn StreamErrorSink>,
) -> Result<Box<dyn OutputStream>, BackendError> {
    let name = pcm_name.to_owned();
    let stop = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&stop);
    let (tx, rx) = mpsc::sync_channel::<Result<(), BackendError>>(1);
    let channels = usize::from(config.channels.max(1));
    let thread = std::thread::Builder::new()
        .name("fp-alsa-dsd".to_owned())
        .spawn(move || match open_pcm(&name, config, cached) {
            Ok(opened) => {
                // `open` gave up waiting: nobody is listening any more.
                if tx.send(Ok(())).is_err() || flag.load(Ordering::Acquire) {
                    return;
                }
                let _priority = audio_thread_priority::promote_current_thread_to_real_time(
                    u32::try_from(opened.device_period).unwrap_or(u32::MAX),
                    opened.device_rate,
                )
                .inspect_err(|_| errors.report(StreamErrorKind::RealtimeDenied))
                .ok();
                run(&opened, channels, renderer.as_mut(), errors.as_ref(), &flag);
                // Setup and exit are outside the loop: free to stop the PCM.
                let _ = opened.pcm.drop();
            }
            Err(e) => {
                let _ = tx.send(Err(e));
            }
        })
        .map_err(|e| BackendError::Backend(e.to_string()))?;
    match rx.recv_timeout(OPEN_TIMEOUT) {
        Ok(Ok(())) => Ok(Box::new(AlsaDsdStream {
            config,
            stop,
            thread: Some(thread),
        })),
        Ok(Err(e)) => {
            let _ = thread.join();
            Err(match e {
                BackendError::Unsupported(m) => {
                    BackendError::Unsupported(format!("{}: {m}", device.0))
                }
                other => other,
            })
        }
        Err(_) => {
            stop.store(true, Ordering::Release);
            Err(BackendError::Unsupported(
                "the device did not open in time".to_owned(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_hw_devices_have_a_pcm_name() {
        assert_eq!(pcm_name("alsa:hw:CARD=D,DEV=0"), Some("hw:CARD=D,DEV=0"));
        assert_eq!(pcm_name("hw:CARD=D,DEV=0"), Some("hw:CARD=D,DEV=0"));
        assert_eq!(pcm_name("alsa:plughw:CARD=D,DEV=0"), None);
        assert_eq!(pcm_name("alsa:default"), None);
    }

    #[test]
    fn a_name_with_a_nul_is_refused() {
        assert_eq!(pcm_name("alsa:hw:CARD=D\0,DEV=0"), None);
    }

    #[test]
    fn a_wait_lasts_a_few_periods_but_not_less_than_the_floor() {
        // 1024 frames at 88 200 Hz: 4 periods = 46 ms.
        assert_eq!(wait_timeout_ms(1024, 88_200), 46);
        assert_eq!(wait_timeout_ms(64, 176_400), MIN_WAIT_MS);
        assert_eq!(wait_timeout_ms(1024, 0), 4_096_000);
    }

    #[test]
    fn formats_map_to_alsa() {
        assert_eq!(alsa_format(NativeDsdFormat::U32Be), Format::DSDU32BE);
        assert_eq!(alsa_format(NativeDsdFormat::U8), Format::DSDU8);
    }

    #[test]
    fn a_period_renders_an_even_number_of_word_frames() {
        assert_eq!(words_per_period(NativeDsdFormat::U32Be, 512), 1024);
        assert_eq!(words_per_period(NativeDsdFormat::U16Le, 511), 510);
        assert_eq!(words_per_period(NativeDsdFormat::U8, 3), 2);
    }
}
