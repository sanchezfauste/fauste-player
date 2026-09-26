#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use fp_backends::{
    AudioBackend, BackendError, DeviceId, NullBackend, OfflineBackend, Renderer, SampleFormat,
    StreamConfig, StreamErrorKind, StreamErrorSink,
};

/// Writes an increasing counter into every sample and counts frames.
struct Counter {
    next: f32,
    frames: Arc<AtomicU64>,
}

impl Renderer for Counter {
    fn render(&mut self, out: &mut [f32], channels: usize) {
        for s in out.iter_mut() {
            *s = self.next;
            self.next += 1.0;
        }
        self.frames
            .fetch_add((out.len() / channels) as u64, Ordering::Relaxed);
    }
}

#[derive(Default)]
struct Errors {
    seen: Mutex<Vec<StreamErrorKind>>,
    count: AtomicUsize,
}

impl StreamErrorSink for Errors {
    fn report(&self, kind: StreamErrorKind) {
        self.count.fetch_add(1, Ordering::Relaxed);
        self.seen.lock().unwrap().push(kind);
    }
}

const STEREO: StreamConfig = StreamConfig {
    sample_rate: 48_000,
    buffer_frames: 256,
    channels: 2,
    exclusive: false,
};

fn counter() -> (Box<Counter>, Arc<AtomicU64>) {
    let frames = Arc::new(AtomicU64::new(0));
    (
        Box::new(Counter {
            next: 0.0,
            frames: frames.clone(),
        }),
        frames,
    )
}

#[test]
fn offline_renders_on_demand_and_stops_when_dropped() {
    let backend = OfflineBackend::new();
    let device = backend.add_device("card", 2);
    assert_eq!(device.render(4), None, "nothing open yet");
    let (renderer, frames) = counter();
    let stream = backend
        .open_output(&device.id(), STEREO, renderer, Arc::new(Errors::default()))
        .unwrap();
    assert_eq!(device.render(2).unwrap(), vec![0.0, 1.0, 2.0, 3.0]);
    assert_eq!(frames.load(Ordering::Relaxed), 2);
    drop(stream);
    assert!(!device.is_open());
}

#[test]
fn offline_unplug_reports_loss_and_refuses_reopen_until_replugged() {
    let backend = OfflineBackend::new();
    let device = backend.add_device("usb", 2);
    let errors = Arc::new(Errors::default());
    let (renderer, _) = counter();
    let _stream = backend
        .open_output(&device.id(), STEREO, renderer, errors.clone())
        .unwrap();
    device.unplug();
    assert_eq!(
        *errors.seen.lock().unwrap(),
        vec![StreamErrorKind::DeviceLost]
    );
    assert_eq!(device.render(4), None);
    assert!(backend.enumerate_devices().unwrap().is_empty());
    let (renderer, _) = counter();
    assert!(matches!(
        backend.open_output(&device.id(), STEREO, renderer, errors.clone()),
        Err(BackendError::DeviceNotFound(_))
    ));
    device.replug();
    let (renderer, _) = counter();
    assert!(
        backend
            .open_output(&device.id(), STEREO, renderer, errors)
            .is_ok()
    );
}

#[test]
fn offline_rejects_more_channels_than_the_device_has() {
    let backend = OfflineBackend::new();
    let device = backend.add_device("mono", 1);
    let (renderer, _) = counter();
    assert!(matches!(
        backend.open_output(&device.id(), STEREO, renderer, Arc::new(Errors::default())),
        Err(BackendError::Unsupported(_))
    ));
}

#[test]
fn null_backend_consumes_audio_at_real_time_pace() {
    let backend = NullBackend;
    let device = backend.default_device().unwrap();
    let (renderer, frames) = counter();
    let stream = backend
        .open_output(&device, STEREO, renderer, Arc::new(Errors::default()))
        .unwrap();
    std::thread::sleep(Duration::from_millis(250));
    drop(stream);
    let rendered = frames.load(Ordering::Relaxed);
    // 250 ms at 48 kHz is 12 000 frames; allow for scheduling jitter.
    assert!((8_000..=16_000).contains(&rendered), "rendered {rendered}");
    let after = frames.load(Ordering::Relaxed);
    std::thread::sleep(Duration::from_millis(50));
    assert_eq!(
        frames.load(Ordering::Relaxed),
        after,
        "a dropped stream stops rendering"
    );
}

#[test]
fn null_backend_rejects_unknown_devices() {
    let (renderer, _) = counter();
    assert!(matches!(
        NullBackend.open_output(
            &DeviceId("nope".into()),
            STEREO,
            renderer,
            Arc::new(Errors::default())
        ),
        Err(BackendError::DeviceNotFound(_))
    ));
}

#[test]
fn sample_formats_hold_their_bits() {
    assert!(SampleFormat::I16.holds_bits(16));
    assert!(!SampleFormat::I16.holds_bits(24));
    assert!(SampleFormat::F32.holds_bits(24));
    assert!(!SampleFormat::F32.holds_bits(32));
    assert!(SampleFormat::I32.holds_bits(24));
    // The mixer works in f32: 32-bit files lose their low bits on any device.
    assert!(!SampleFormat::I32.holds_bits(32));
    assert!(SampleFormat::I24.holds_bits(24));
    assert!(!SampleFormat::I24.holds_bits(32));
}

#[test]
fn offline_exclusive_access_needs_a_capable_device() {
    let backend = OfflineBackend::new();
    let shared = backend.add_device("shared", 2);
    let direct = backend.add_device("direct", 2);
    direct.set_exclusive_capable(true);
    let exclusive = StreamConfig {
        exclusive: true,
        ..STEREO
    };
    let refused = backend.open_output(
        &shared.id(),
        exclusive,
        counter().0,
        Arc::new(Errors::default()),
    );
    assert!(matches!(refused, Err(BackendError::Unsupported(_))));
    let stream = backend
        .open_output(
            &direct.id(),
            exclusive,
            counter().0,
            Arc::new(Errors::default()),
        )
        .unwrap();
    assert_eq!(stream.sample_format(), SampleFormat::F32);
    assert_eq!(direct.config(), Some(exclusive));
    let listed = backend.enumerate_devices().unwrap();
    assert!(
        listed
            .iter()
            .any(|d| d.id == direct.id() && d.exclusive_capable && d.rate_switching)
    );
}

#[test]
fn offline_devices_can_refuse_a_rate() {
    let backend = OfflineBackend::new();
    let device = backend.add_device("card", 2);
    device.refuse_rate(44_100);
    let at_44k = StreamConfig {
        sample_rate: 44_100,
        ..STEREO
    };
    assert!(matches!(
        backend.open_output(
            &device.id(),
            at_44k,
            counter().0,
            Arc::new(Errors::default())
        ),
        Err(BackendError::Unsupported(_))
    ));
    assert!(
        backend
            .open_output(
                &device.id(),
                STEREO,
                counter().0,
                Arc::new(Errors::default())
            )
            .is_ok()
    );
}

#[test]
fn null_refuses_exclusive_access() {
    let backend = NullBackend;
    let device = backend.default_device().unwrap();
    let exclusive = StreamConfig {
        exclusive: true,
        ..STEREO
    };
    assert!(matches!(
        backend.open_output(&device, exclusive, counter().0, Arc::new(Errors::default())),
        Err(BackendError::Unsupported(_))
    ));
}
