#![allow(clippy::unwrap_used)]
//! A busy port is logged once, not on every retry (plan 4, item 2). Its
//! own test binary: it installs the global log subscriber.

mod support;

use std::io::Write;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use fp_remote::{ServerError, ServerStatus, Timing, spawn_with};
use support::{FakeControl, demo_state};

#[derive(Clone, Default)]
struct Lines(Arc<Mutex<Vec<u8>>>);

impl Write for Lines {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(b);
        Ok(b.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn a_busy_port_is_logged_once_across_retries() {
    let lines = Lines::default();
    let writer = lines.clone();
    tracing_subscriber::fmt()
        .with_writer(move || writer.clone())
        .with_ansi(false)
        .init();
    let busy = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = busy.local_addr().unwrap().port();
    let fake = FakeControl::new(demo_state());
    fake.edit(|s| {
        s.config.remote.http.enabled = true;
        s.config.remote.http.port = port;
    });
    let timing = Timing {
        config_poll: Duration::from_millis(5),
        bind_retry: Duration::from_millis(10),
    };
    let handle = spawn_with(fake.clone(), timing).unwrap();
    // Every round publishes the status again as a new snapshot, so the
    // number of distinct snapshots with the bind error is the number of
    // times the port was tried. Wait for four of them (no fixed sleep).
    let cell = handle.status_cell();
    let mut seen = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(10);
    while seen.len() < 4 {
        assert!(
            Instant::now() < deadline,
            "only {} attempts seen",
            seen.len()
        );
        let status = cell.load_full();
        let failed = matches!(status.http, ServerStatus::Error(ServerError::Bind(_)));
        if failed && !seen.iter().any(|s| Arc::ptr_eq(s, &status)) {
            seen.push(status);
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    drop(handle);
    let text = String::from_utf8(lines.0.lock().unwrap().clone()).unwrap();
    assert_eq!(
        text.matches("remote HTTP could not listen").count(),
        1,
        "{text}"
    );
}
