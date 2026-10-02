#![allow(clippy::unwrap_used)]
//! Decoder threads ask for above-normal priority, and a system that refuses
//! is logged once, not once per thread (main spec §2.2). Its own test
//! binary: it installs the global log subscriber, and it reads the threads
//! from `/proc`.
#![cfg(target_os = "linux")]

use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use fp_engine::worker::{PlayerWorker, file_opener};

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

/// The nice value of the thread of this process called `name`.
fn nice_of(name: &str) -> Option<i32> {
    for task in std::fs::read_dir("/proc/self/task").unwrap().flatten() {
        let comm = std::fs::read_to_string(task.path().join("comm")).unwrap_or_default();
        if comm.trim_end() != name {
            continue;
        }
        let stat = std::fs::read_to_string(task.path().join("stat")).ok()?;
        let after = stat.rsplit_once(')')?.1;
        return after.split_whitespace().nth(16)?.parse().ok();
    }
    None
}

fn spawn(name: &str) -> PlayerWorker {
    let (failures, _rx) = crossbeam_channel::unbounded();
    PlayerWorker::spawn(name, file_opener(), 48_000, 4_800, failures).unwrap()
}

#[test]
fn decoder_threads_ask_for_a_higher_priority_and_a_refusal_is_logged_once() {
    let lines = Lines::default();
    let writer = lines.clone();
    tracing_subscriber::fmt()
        .with_writer(move || writer.clone())
        .with_ansi(false)
        .init();
    let a = spawn("fp-decode-a");
    let b = spawn("fp-decode-b");
    let refused = |lines: &Lines| {
        String::from_utf8(lines.0.lock().unwrap().clone())
            .unwrap()
            .matches("refused a thread priority change")
            .count()
    };
    let raised = |name: &str| nice_of(name).is_some_and(|n| n < 0);
    // Either the system accepted both (they are above normal), or it refused
    // and said so. Neither happening means the threads never asked.
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let said = refused(&lines) > 0;
        if (said || raised("fp-decode-a")) && (said || raised("fp-decode-b")) {
            break;
        }
        assert!(Instant::now() < deadline, "the decoder threads never asked");
        std::thread::sleep(Duration::from_millis(5));
    }
    // Dropping a worker joins its thread: both have asked by now.
    let both_raised = raised("fp-decode-a") && raised("fp-decode-b");
    drop(a);
    drop(b);
    let said = refused(&lines);
    assert!(said <= 1, "logged {said} times");
    assert!(said == 1 || both_raised);
}
