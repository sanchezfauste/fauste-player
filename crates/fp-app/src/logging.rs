//! Logging (spec §9): a daily-rotated file in the log directory, written by
//! a background thread so logging never blocks callers, plus stderr in
//! debug builds. `RUST_LOG` overrides the default filter.
//!
//! The disk use is bounded three ways: the default filter keeps the
//! PulseAudio client quiet (it logs every connection it makes); identical
//! consecutive lines are written once and followed by a "repeated N times"
//! line; and each daily file stops at `limits.max_log_file_mb`, with at most
//! `limits.max_log_files` files kept.

use std::io::{self, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

const FILE_PREFIX: &str = "fauste-player";
const FILE_SUFFIX: &str = "log";
const MIB: u64 = 1024 * 1024;

/// What `RUST_LOG` falls back to. The `pulseaudio` client logs a line at
/// info for every server connection, which a lost-and-retried output makes
/// every few seconds.
const DEFAULT_FILTER: &str = "info,pulseaudio=warn";

/// Writes a line once when it repeats: the lines that follow it with the
/// same text (everything after the leading timestamp) are counted, and a
/// single "previous message repeated N times" line replaces them when a
/// different line arrives or the writer is dropped (the end of the run).
/// It runs on the log thread, behind the non-blocking queue.
struct Collapse<W: Write> {
    inner: W,
    last_key: Vec<u8>,
    last_stamp: Vec<u8>,
    repeats: u64,
}

impl<W: Write> Collapse<W> {
    fn new(inner: W) -> Self {
        Self {
            inner,
            last_key: Vec::new(),
            last_stamp: Vec::new(),
            repeats: 0,
        }
    }

    fn summary(&mut self) -> io::Result<()> {
        if self.repeats == 0 {
            return Ok(());
        }
        let n = self.repeats;
        self.repeats = 0;
        let stamp = String::from_utf8_lossy(&self.last_stamp).into_owned();
        let times = if n == 1 { "time" } else { "times" };
        writeln!(
            self.inner,
            "{stamp}  INFO previous message repeated {n} {times}"
        )
    }
}

/// Splits a formatted line into its timestamp and the rest.
fn split_stamp(line: &[u8]) -> (&[u8], &[u8]) {
    let end = line
        .iter()
        .position(u8::is_ascii_whitespace)
        .unwrap_or(line.len());
    (
        line.get(..end).unwrap_or_default(),
        line.get(end..).unwrap_or_default(),
    )
}

impl<W: Write> Write for Collapse<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let (stamp, key) = split_stamp(buf);
        if !self.last_key.is_empty() && key == self.last_key.as_slice() {
            self.repeats += 1;
            self.last_stamp = stamp.to_vec();
            return Ok(buf.len());
        }
        self.summary()?;
        self.last_key = key.to_vec();
        self.last_stamp = stamp.to_vec();
        self.inner.write_all(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<W: Write> Drop for Collapse<W> {
    fn drop(&mut self) {
        let _ = self.summary();
        let _ = self.inner.flush();
    }
}

/// The current UTC day number (the rolling appender starts a new file at
/// UTC midnight).
fn utc_day() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() / 86_400)
}

/// Stops writing once the day's file holds `cap` bytes: one closing line
/// says so and everything after it is dropped until the next UTC day.
struct Capped<W: Write> {
    inner: W,
    cap: u64,
    written: u64,
    day: u64,
    clock: Box<dyn Fn() -> u64 + Send>,
    full: bool,
}

impl<W: Write> Capped<W> {
    /// `already` is what today's file holds from an earlier run.
    fn new(inner: W, cap: u64, already: u64) -> Self {
        Self::with_clock(inner, cap, already, utc_day)
    }

    fn with_clock(
        inner: W,
        cap: u64,
        already: u64,
        clock: impl Fn() -> u64 + Send + 'static,
    ) -> Self {
        Self {
            inner,
            cap,
            written: already,
            day: clock(),
            clock: Box::new(clock),
            full: false,
        }
    }
}

impl<W: Write> Write for Capped<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let today = (self.clock)();
        if today != self.day {
            self.day = today;
            self.written = 0;
            self.full = false;
        }
        if self.full {
            return Ok(buf.len());
        }
        if self.written.saturating_add(buf.len() as u64) > self.cap {
            self.full = true;
            let (stamp, _) = split_stamp(buf);
            let stamp = String::from_utf8_lossy(stamp).into_owned();
            writeln!(
                self.inner,
                "{stamp}  WARN log file size cap reached ({} MiB); no more lines are written to this file today",
                self.cap / MIB
            )?;
            return Ok(buf.len());
        }
        self.inner.write_all(buf)?;
        self.written += buf.len() as u64;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

/// Bytes in the log files modified today (a run earlier in the day wrote
/// them), so that a restart does not get a fresh allowance.
fn written_today(dir: &Path) -> u64 {
    let today = utc_day();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    entries
        .flatten()
        .filter(|e| {
            let name = e.file_name();
            let name = name.to_string_lossy();
            name.starts_with(FILE_PREFIX) && name.ends_with(FILE_SUFFIX)
        })
        .filter_map(|e| e.metadata().ok())
        .filter(|m| {
            m.modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .is_some_and(|d| d.as_secs() / 86_400 == today)
        })
        .map(|m| m.len())
        .max()
        .unwrap_or(0)
}

/// Starts logging. Keep the returned guard alive for the whole run: dropping
/// it flushes the file. Returns `None` if the log file cannot be created
/// (the app still runs, logging to stderr only). `max_file_mb` and
/// `max_files` are `limits.max_log_file_mb` and `limits.max_log_files`.
pub fn init(log_dir: &Path, max_file_mb: u64, max_files: usize) -> Option<WorkerGuard> {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER));
    let file = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .max_log_files(max_files)
        .filename_prefix(FILE_PREFIX)
        .filename_suffix(FILE_SUFFIX)
        .build(log_dir);
    let (file_layer, guard, error) = match file {
        Ok(appender) => {
            let capped = Capped::new(
                appender,
                max_file_mb.saturating_mul(MIB),
                written_today(log_dir),
            );
            let (writer, guard) = tracing_appender::non_blocking(Collapse::new(capped));
            (
                Some(
                    tracing_subscriber::fmt::layer()
                        .with_ansi(false)
                        .with_writer(writer),
                ),
                Some(guard),
                None,
            )
        }
        Err(e) => (None, None, Some(e)),
    };
    // stderr in debug builds, or whenever the file cannot be written.
    let stderr = (cfg!(debug_assertions) || file_layer.is_none())
        .then(|| tracing_subscriber::fmt::layer().with_writer(std::io::stderr));
    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .with(stderr)
        .try_init();
    if let Some(e) = error {
        tracing::warn!("cannot open the log file: {e}");
    }
    guard
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Default)]
    struct Sink(Arc<Mutex<Vec<u8>>>);

    impl Write for Sink {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl Sink {
        fn text(&self) -> String {
            String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
        }
    }

    fn line(ts: &str, text: &str) -> Vec<u8> {
        format!("{ts}  INFO app: {text}\n").into_bytes()
    }

    #[test]
    fn identical_consecutive_lines_are_written_once_then_summarised() {
        let sink = Sink::default();
        let mut w = Collapse::new(sink.clone());
        for t in ["t1", "t2", "t3", "t4"] {
            w.write_all(&line(t, "connecting")).unwrap();
        }
        assert_eq!(sink.text(), "t1  INFO app: connecting\n");
        w.write_all(&line("t5", "other")).unwrap();
        assert_eq!(
            sink.text(),
            "t1  INFO app: connecting\n\
             t4  INFO previous message repeated 3 times\n\
             t5  INFO app: other\n"
        );
    }

    #[test]
    fn a_pending_summary_is_written_when_the_writer_is_dropped() {
        let sink = Sink::default();
        let mut w = Collapse::new(sink.clone());
        w.write_all(&line("t1", "a")).unwrap();
        w.write_all(&line("t2", "b")).unwrap();
        w.write_all(&line("t3", "b")).unwrap();
        drop(w);
        assert!(
            sink.text()
                .ends_with("t3  INFO previous message repeated 1 time\n")
        );
    }

    #[test]
    fn the_cap_stops_a_file_with_one_closing_line() {
        let sink = Sink::default();
        let mut w = Capped::new(sink.clone(), 100, 0);
        let first = line("t1", "0123456789012345678901234567890123456789");
        w.write_all(&first).unwrap();
        w.write_all(&line(
            "t2",
            "second line that does not fit in the cap at all",
        ))
        .unwrap();
        let after_cap = sink.text();
        assert!(after_cap.contains("t1"));
        assert!(!after_cap.contains("second line"));
        assert!(after_cap.contains("log file size cap reached"));
        w.write_all(&line("t3", "more")).unwrap();
        assert_eq!(sink.text(), after_cap, "nothing more is written");
    }

    #[test]
    fn the_cap_counts_what_the_file_already_holds_and_resets_on_a_new_day() {
        let sink = Sink::default();
        let day = Arc::new(std::sync::atomic::AtomicU64::new(1));
        let mut w = Capped::with_clock(sink.clone(), 50, 49, {
            let day = Arc::clone(&day);
            move || day.load(std::sync::atomic::Ordering::Relaxed)
        });
        w.write_all(&line("t1", "x")).unwrap();
        assert!(sink.text().contains("cap reached"));
        let before = sink.text().len();
        day.store(2, std::sync::atomic::Ordering::Relaxed);
        w.write_all(&line("t2", "fresh")).unwrap();
        assert!(sink.text().len() > before);
        assert!(sink.text().contains("t2"));
    }

    #[test]
    fn the_default_filter_quiets_the_pulseaudio_client() {
        assert_eq!(DEFAULT_FILTER, "info,pulseaudio=warn");
        let f = EnvFilter::new(DEFAULT_FILTER);
        assert!(f.to_string().contains("pulseaudio=warn"));
    }
}
