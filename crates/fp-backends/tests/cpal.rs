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
