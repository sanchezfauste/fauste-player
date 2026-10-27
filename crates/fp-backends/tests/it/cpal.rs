#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Smoke test against the real platform audio stack. It never fails on
//! machines without a usable output device (CI runners).

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use fp_backends::{
    AudioBackend, CpalBackend, Renderer, StreamConfig, StreamErrorKind, StreamErrorSink,
};

struct Silence {
    frames: Arc<AtomicU64>,
}

impl Renderer for Silence {
    fn render(&mut self, out: &mut [f32], channels: usize) {
        out.fill(0.0);
        self.frames
            .fetch_add((out.len() / channels) as u64, Ordering::Relaxed);
    }
}

struct Ignore;

impl StreamErrorSink for Ignore {
    fn report(&self, _kind: StreamErrorKind) {}
}

const STEREO: StreamConfig = StreamConfig {
    sample_rate: 48_000,
    buffer_frames: 256,
    channels: 2,
    exclusive: false,
    dsd: None,
    exact_buffer: false,
    shared_buffer_us: 0,
};

#[test]
fn cpal_backend_enumerates_without_panicking_and_opens_the_default_device_if_any() {
    let backend = CpalBackend::default_host();
    let Ok(devices) = backend.enumerate_devices() else {
        return;
    };
    let Some(default) = backend.default_device() else {
        return;
    };
    assert!(devices.iter().any(|d| d.id == default) || !devices.is_empty());
    let frames = Arc::new(AtomicU64::new(0));
    let renderer = Box::new(Silence {
        frames: frames.clone(),
    });
    let Ok(stream) = backend.open_output(&default, STEREO, renderer, Arc::new(Ignore)) else {
        return; // CI machines often have no usable output device.
    };
    std::thread::sleep(Duration::from_millis(200));
    drop(stream);
    assert!(frames.load(Ordering::Relaxed) > 0);
}

/// The scheduling policy of the calling thread (field 41 of its `stat`):
/// 1 is `SCHED_FIFO`, 2 `SCHED_RR`.
#[cfg(target_os = "linux")]
fn policy() -> Option<u32> {
    let stat = std::fs::read_to_string("/proc/thread-self/stat").ok()?;
    let after = stat.rsplit_once(')')?.1;
    after.split_whitespace().nth(38)?.parse().ok()
}

/// What the callback thread ran with once it had settled.
#[cfg(target_os = "linux")]
struct Probe {
    calls: u32,
    policy: Arc<std::sync::Mutex<Option<u32>>>,
}

#[cfg(target_os = "linux")]
impl Renderer for Probe {
    fn render(&mut self, out: &mut [f32], _channels: usize) {
        out.fill(0.0);
        self.calls += 1;
        // Until the stream is dropped: rtkit may answer late.
        if self.calls >= 8 {
            *self.policy.lock().unwrap() = policy();
        }
    }
}

#[cfg(target_os = "linux")]
struct Denied(Arc<std::sync::atomic::AtomicBool>);

#[cfg(target_os = "linux")]
impl StreamErrorSink for Denied {
    fn report(&self, kind: StreamErrorKind) {
        if kind == StreamErrorKind::RealtimeDenied {
            self.0.store(true, Ordering::Relaxed);
        }
    }
}

/// A PulseAudio output (a sound server: shared) takes the shared buffer,
/// and its callback thread, which cpal leaves at normal priority, is
/// promoted or the refusal is reported. Passes without a server.
#[cfg(target_os = "linux")]
#[test]
fn a_pulseaudio_output_is_shared_and_its_callback_runs_in_real_time() {
    let Some(backend) = fp_backends::system_backends()
        .into_iter()
        .find(|b| b.id().0 == "pulseaudio")
    else {
        return;
    };
    let Some(default) = backend.default_device() else {
        return;
    };
    let seen = Arc::new(std::sync::Mutex::new(None));
    let denied = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let config = StreamConfig {
        shared_buffer_us: 50_000,
        ..STEREO
    };
    let renderer = Box::new(Probe {
        calls: 0,
        policy: seen.clone(),
    });
    let Ok(stream) =
        backend.open_output(&default, config, renderer, Arc::new(Denied(denied.clone())))
    else {
        return;
    };
    std::thread::sleep(Duration::from_millis(1500));
    assert_eq!(stream.config().buffer_frames, 2_400, "50 ms at 48 kHz");
    let policy = *seen.lock().unwrap();
    drop(stream);
    assert!(
        matches!(policy, Some(1 | 2)) || denied.load(Ordering::Relaxed),
        "policy {policy:?}, no refusal reported"
    );
}
