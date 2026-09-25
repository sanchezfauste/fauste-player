//! Logging (spec §9): a daily-rotated file in the log directory (14 files
//! kept), written by a background thread so logging never blocks callers,
//! plus stderr in debug builds. `RUST_LOG` overrides the default level.

use std::path::Path;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Starts logging. Keep the returned guard alive for the whole run: dropping
/// it flushes the file. Returns `None` if the log file cannot be created
/// (the app still runs, logging to stderr only).
pub fn init(log_dir: &Path) -> Option<WorkerGuard> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let file = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .max_log_files(14)
        .filename_prefix("fauste-player")
        .filename_suffix("log")
        .build(log_dir);
    let (file_layer, guard, error) = match file {
        Ok(appender) => {
            let (writer, guard) = tracing_appender::non_blocking(appender);
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
