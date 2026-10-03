#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Audit finding A8: every counted event is logged when it increases, at
//! most once per window, by the conductor and never by the real-time code.

mod support;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use fp_backends::{AudioBackend, OfflineBackend, OfflineDevice, StreamErrorKind};
use fp_engine::conductor::{Conductor, ConductorHandle, REPORT_WINDOW};
use fp_engine::engine::{Engine, EngineSettings};
use fp_model::{AppState, Config, Route};
use support::tagged_opener;

#[derive(Clone, Default)]
struct Lines(Arc<Mutex<Vec<u8>>>);

impl std::io::Write for Lines {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(b);
        Ok(b.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Lines {
    fn text(&self) -> String {
        String::from_utf8_lossy(&self.0.lock().unwrap()).into_owned()
    }

    fn count(&self, needle: &str) -> usize {
        self.text().lines().filter(|l| l.contains(needle)).count()
    }
}

/// Runs `f` with this thread's log lines captured (the conductor ticks on
/// the test thread).
fn capture<T>(f: impl FnOnce() -> T) -> (T, Lines) {
    let lines = Lines::default();
    let writer = lines.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_writer(move || writer.clone())
        .with_ansi(false)
        .finish();
    let out = tracing::subscriber::with_default(subscriber, f);
    (out, lines)
}

fn conductor() -> (Conductor, ConductorHandle, OfflineDevice, Instant) {
    let mut config = Config::default();
    config.players.count = 1;
    config.outputs.backend = Some("offline".into());
    config.outputs.buffer_frames = 480;
    let state = AppState::new(config, "Main");
    let backend = OfflineBackend::new();
    let device = backend.add_device("main", 2);
    let settings = EngineSettings::from_config(&state.config);
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
    let engine = Engine::new(backends, settings, tagged_opener(48_000));
    let now = Instant::now();
    let (conductor, handle) = Conductor::new(state, Vec::new(), engine, now);
    (conductor, handle, device, now)
}

/// A conductor whose bus is open (a test tone opens it).
fn open_conductor() -> (Conductor, ConductorHandle, OfflineDevice, Instant) {
    let (mut conductor, handle, device, now) = conductor();
    let route = Route {
        backend: "offline".into(),
        device: "main".into(),
        first_channel: 0,
    };
    assert!(handle.test_tone(route, 1_000.0));
    conductor.tick(now);
    assert!(device.is_open());
    (conductor, handle, device, now)
}

#[test]
fn xruns_are_logged_once_per_window_with_what_came_after() {
    let ((), lines) = capture(|| {
        let (mut c, handle, device, now) = open_conductor();
        // Quiet: nothing is logged.
        c.tick(now + Duration::from_millis(10));
        assert!(device.report_error(StreamErrorKind::Xrun));
        c.tick(now + Duration::from_millis(20));
        // More inside the window: held back.
        for _ in 0..3 {
            assert!(device.report_error(StreamErrorKind::Xrun));
        }
        c.tick(now + Duration::from_millis(30));
        c.tick(now + Duration::from_millis(40));
        let in_window = now + Duration::from_millis(20) + REPORT_WINDOW;
        // The window passes: the held-back ones are logged on the next tick.
        c.tick(in_window + Duration::from_millis(1));
        c.tick(in_window + Duration::from_millis(2));
        let t = handle.telemetry.load();
        assert_eq!(t.buses[0].counters.xruns, 4);
    });
    assert_eq!(lines.count("xrun"), 2, "{}", lines.text());
    assert!(lines.text().contains("new=1"), "{}", lines.text());
    assert!(lines.text().contains("new=3"), "{}", lines.text());
}

#[test]
fn an_unknown_stream_error_is_logged_and_not_an_xrun() {
    let (counters, lines) = capture(|| {
        let (mut c, handle, device, now) = open_conductor();
        assert!(device.report_error(StreamErrorKind::Other));
        c.tick(now + Duration::from_millis(10));
        let t = handle.telemetry.load();
        t.buses[0].counters
    });
    assert_eq!(counters.xruns, 0);
    assert_eq!(counters.stream_errors, 1);
    assert_eq!(lines.count("stream error"), 1, "{}", lines.text());
    assert_eq!(lines.count("xrun"), 0, "{}", lines.text());
}
