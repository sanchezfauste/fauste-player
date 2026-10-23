#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The MIDI service's log (J6): what a field log needs to tell a dead
//! connection from a silent device. Alone in its binary, so no other test's
//! thread caches the log callsites as uninteresting.

mod support;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use support::{FakePorts, control, core};

/// Log lines written while `f` runs, info level and up, without ANSI codes.
fn logged(f: impl FnOnce()) -> String {
    #[derive(Clone, Default)]
    struct Buf(Arc<Mutex<Vec<u8>>>);
    impl std::io::Write for Buf {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let buf = Buf::default();
    let writer = buf.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .with_max_level(tracing::Level::INFO)
        .with_writer(move || writer.clone())
        .finish();
    tracing::subscriber::with_default(subscriber, f);
    String::from_utf8(buf.0.lock().unwrap().clone()).unwrap()
}

#[test]
fn the_log_tells_connections_drops_and_first_messages_apart() {
    let ports = FakePorts::default();
    ports.plug("APC");
    let c = control(true);
    let (mut core, _handle) = core(&ports, &c);
    let mut t = Instant::now();
    let log = logged(|| {
        core.step(t);
        ports.press("APC", &[0x90, 36, 100]);
        ports.press("APC", &[0x90, 37, 100]);
        core.step(t);
        ports.unplug("APC");
        t += Duration::from_millis(2100);
        core.step(t);
        ports.plug("APC");
        t += Duration::from_millis(2100);
        core.step(t);
        ports.press("APC", &[0x90, 36, 100]);
        core.step(t);
    });
    let lines: Vec<&str> = log
        .lines()
        .filter(|l| l.contains("MIDI"))
        .map(|l| l.split_once("fp_control").map_or(l, |(_, rest)| rest))
        .collect();
    let has = |needle: &str| lines.iter().filter(|l| l.contains(needle)).count();
    assert_eq!(has("MIDI input connected"), 2, "{log}");
    assert_eq!(has("MIDI output connected"), 2, "{log}");
    assert_eq!(has("MIDI input disconnected"), 1, "{log}");
    assert_eq!(has("MIDI output disconnected"), 1, "{log}");
    assert_eq!(
        has("first MIDI message"),
        2,
        "once per connection, not per message: {log}"
    );
    assert!(has("MIDI ports") >= 3, "the lists as they change: {log}");
}
