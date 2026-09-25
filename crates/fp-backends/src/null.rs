//! A backend that discards audio at real-time pace. It keeps the engine's
//! timeline running when no sound card is wanted (and in soak tests).

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crate::{
    AudioBackend, Availability, BackendError, BackendId, DeviceId, DeviceInfo, OutputStream,
    Renderer, StreamConfig, StreamErrorSink,
};

pub const NULL_DEVICE: &str = "null";

#[derive(Debug, Clone, Copy, Default)]
pub struct NullBackend;

struct NullStream {
    config: StreamConfig,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl OutputStream for NullStream {
    fn config(&self) -> StreamConfig {
        self.config
    }
}

impl Drop for NullStream {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl AudioBackend for NullBackend {
    fn id(&self) -> BackendId {
        BackendId("null".to_owned())
    }

    fn availability(&self) -> Availability {
        Availability::Available
    }

    fn enumerate_devices(&self) -> Result<Vec<DeviceInfo>, BackendError> {
        Ok(vec![DeviceInfo {
            id: DeviceId(NULL_DEVICE.to_owned()),
            name: "Null output".to_owned(),
            channels: 64,
            sample_rates: vec![(8_000, 768_000)],
            buffer_frames: Some((16, 16_384)),
            exclusive_capable: false,
            rate_switching: false,
        }])
    }

    fn default_device(&self) -> Option<DeviceId> {
        Some(DeviceId(NULL_DEVICE.to_owned()))
    }

    fn open_output(
        &self,
        device: &DeviceId,
        config: StreamConfig,
        mut renderer: Box<dyn Renderer>,
        _errors: Arc<dyn StreamErrorSink>,
    ) -> Result<Box<dyn OutputStream>, BackendError> {
        if device.0 != NULL_DEVICE {
            return Err(BackendError::DeviceNotFound(device.clone()));
        }
        if config.sample_rate == 0 || config.buffer_frames == 0 || config.channels == 0 {
            return Err(BackendError::Unsupported(format!("{config:?}")));
        }
        let stop = Arc::new(AtomicBool::new(false));
        let stop_flag = stop.clone();
        let channels = usize::from(config.channels);
        let frames = config.buffer_frames as usize;
        let period = Duration::from_secs_f64(
            f64::from(config.buffer_frames) / f64::from(config.sample_rate),
        );
        let thread = std::thread::Builder::new()
            .name("fp-null-output".to_owned())
            .spawn(move || {
                let mut buffer = vec![0.0f32; frames * channels];
                let mut deadline = Instant::now();
                while !stop_flag.load(Ordering::Acquire) {
                    renderer.render(&mut buffer, channels);
                    deadline += period;
                    let now = Instant::now();
                    if deadline > now {
                        std::thread::sleep(deadline - now);
                    } else {
                        // Fell behind (machine suspended, debugger…): do not try to catch up.
                        deadline = now;
                    }
                }
            })
            .map_err(|e| BackendError::Backend(e.to_string()))?;
        Ok(Box::new(NullStream {
            config,
            stop,
            thread: Some(thread),
        }))
    }
}
