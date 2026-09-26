//! Crash reports (spec §9): a panic writes a report file to the log
//! directory, then the previous hook runs as usual. At most `max_reports`
//! files are written per run, so a panic that repeats (and is contained)
//! cannot fill the disk; later panics are only logged.

use std::backtrace::Backtrace;
use std::fmt::Write as _;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn install_panic_hook(log_dir: PathBuf, max_reports: usize) {
    let previous = std::panic::take_hook();
    let written = AtomicUsize::new(0);
    std::panic::set_hook(Box::new(move |info| {
        let message = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| (*s).to_owned())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "non-string panic payload".to_owned());
        let location = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_default();
        let thread = std::thread::current()
            .name()
            .unwrap_or("unnamed")
            .to_owned();
        let mut report = String::new();
        let _ = writeln!(report, "Fauste Player crash report");
        let _ = writeln!(report, "version: {}", env!("CARGO_PKG_VERSION"));
        let _ = writeln!(
            report,
            "os: {} {}",
            std::env::consts::OS,
            std::env::consts::ARCH
        );
        let _ = writeln!(report, "thread: {thread}");
        let _ = writeln!(report, "message: {message}");
        let _ = writeln!(report, "location: {location}");
        if written.fetch_add(1, Ordering::AcqRel) < max_reports {
            let _ = writeln!(report, "\n{}", Backtrace::force_capture());
            let stamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            let _ = std::fs::create_dir_all(&log_dir);
            let _ = std::fs::write(log_dir.join(format!("crash-{stamp}.txt")), report);
        }
        tracing::error!(%thread, %location, "panic: {message}");
        previous(info);
    }));
}
