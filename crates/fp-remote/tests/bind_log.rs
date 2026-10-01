#![allow(clippy::unwrap_used)]
//! A busy port is logged once, not on every retry (plan 4, item 2). Its
//! own test binary: it installs the global log subscriber.

mod support;

use std::io::Write;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use fp_remote::{ServerError, ServerStatus, spawn};
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
    let handle = spawn(fake.clone()).unwrap();
    // The bind is retried every 2 s: let it try three times.
    std::thread::sleep(Duration::from_millis(5_500));
    assert!(matches!(
        handle.status().http,
        ServerStatus::Error(ServerError::Bind(_))
    ));
    drop(handle);
    let text = String::from_utf8(lines.0.lock().unwrap().clone()).unwrap();
    assert_eq!(
        text.matches("remote HTTP could not listen").count(),
        1,
        "{text}"
    );
}
