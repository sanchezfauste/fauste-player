#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! One set of checks every audio backend must pass (Phase 3 exit
//! criterion). Null and Offline run in CI; every real system runs with
//! `cargo test -p fp-backends --test conformance -- --ignored` on a machine
//! that has it (the manual on-device check).

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use fp_backends::{
    AudioBackend, Availability, NullBackend, OfflineBackend, Renderer, StreamConfig,
    StreamErrorKind, StreamErrorSink, system_backends,
};

struct Counting(Arc<AtomicU64>);

impl Renderer for Counting {
    fn render(&mut self, out: &mut [f32], channels: usize) {
        // A quiet tone, so a listener can confirm the output on a device.
        let n = self
            .0
            .fetch_add((out.len() / channels.max(1)) as u64, Ordering::Relaxed);
        for (i, frame) in out.chunks_mut(channels.max(1)).enumerate() {
            let t = (n + i as u64) as f32 / 48_000.0;
            let v = (t * 440.0 * std::f32::consts::TAU).sin() * 0.05;
            frame.fill(v);
        }
    }
}

struct Ignore;

impl StreamErrorSink for Ignore {
    fn report(&self, _kind: StreamErrorKind) {}
}

const CONFIG: StreamConfig = StreamConfig {
    sample_rate: 48_000,
    buffer_frames: 512,
    channels: 2,
    exclusive: false,
    dsd: None,
};

/// Waits until `ok`, pumping `drive` (for backends rendered on demand).
fn wait(what: &str, drive: &dyn Fn(), ok: impl Fn() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while !ok() {
        assert!(Instant::now() < deadline, "timed out: {what}");
        drive();
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// The checks. `drive` renders one block for backends without a clock.
fn conform(backend: &dyn AudioBackend, drive: &dyn Fn()) {
    let id = backend.id().0;
    assert!(!id.is_empty());
    assert_eq!(
        backend.availability(),
        Availability::Available,
        "{id} must be available"
    );
    let devices = backend.enumerate_devices().expect("devices enumerate");
    assert!(!devices.is_empty(), "{id} lists at least one device");
    for d in &devices {
        assert!(!d.name.is_empty(), "{id}: every device has a name");
    }
    let device = backend
        .default_device()
        .or_else(|| devices.first().map(|d| d.id.clone()))
        .expect("a device to open");
    assert!(
        devices.iter().any(|d| d.id == device),
        "{id}: the default device is listed"
    );
    // Opens, renders, and stops rendering once dropped.
    let frames = Arc::new(AtomicU64::new(0));
    let stream = backend
        .open_output(
            &device,
            CONFIG,
            Box::new(Counting(frames.clone())),
            Arc::new(Ignore),
        )
        .expect("the default device opens");
    let config = stream.config();
    assert!(
        config.channels >= 2 && config.sample_rate > 0,
        "{id}: {config:?}"
    );
    wait("the renderer is called", drive, || {
        frames.load(Ordering::Relaxed) > 4_800
    });
    drop(stream);
    std::thread::sleep(Duration::from_millis(300));
    let after = frames.load(Ordering::Relaxed);
    std::thread::sleep(Duration::from_millis(500));
    assert_eq!(
        frames.load(Ordering::Relaxed),
        after,
        "{id}: no callbacks after drop"
    );
    // The same device opens again.
    let again = Arc::new(AtomicU64::new(0));
    let stream = backend
        .open_output(
            &device,
            CONFIG,
            Box::new(Counting(again.clone())),
            Arc::new(Ignore),
        )
        .expect("the device reopens");
    wait("the reopened renderer is called", drive, || {
        again.load(Ordering::Relaxed) > 0
    });
    drop(stream);
}

#[test]
fn the_null_backend_conforms() {
    conform(&NullBackend, &|| {});
}

#[test]
fn the_offline_backend_conforms() {
    let backend = OfflineBackend::new();
    let device = backend.add_device("card", 2);
    conform(&backend, &|| {
        let _ = device.render(512);
    });
}

#[test]
#[ignore = "needs the real audio systems of this machine"]
fn every_available_system_conforms() {
    let mut checked = Vec::new();
    for backend in system_backends() {
        if backend.availability() != Availability::Available {
            eprintln!("skipping {}: {:?}", backend.id().0, backend.availability());
            continue;
        }
        conform(backend.as_ref(), &|| {});
        checked.push(backend.id().0);
    }
    eprintln!("conformance passed for: {checked:?}");
    assert!(!checked.is_empty());
}
