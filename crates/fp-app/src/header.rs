//! The header reader (operator feedback 4, Q1.1): one thread that opens a
//! file's header and reads how long it is, so a track has a length, a
//! countdown and click-to-seek before its analysis. It follows the file
//! probe's pattern: jobs in, answers out, both on unbounded channels, and
//! the thread ends when its handle is dropped.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};

use crossbeam_channel::{Receiver, Sender};
use fp_decode::FileDecoder;
use fp_model::TrackId;

/// The length `path`'s header declares, in seconds. `None` when the file
/// cannot be opened or its container does not say.
pub fn header_duration(path: &Path) -> Option<f64> {
    FileDecoder::open(path).ok()?.duration_hint_secs()
}

/// The reader thread's handle.
pub struct HeaderReader {
    jobs: Sender<(TrackId, PathBuf)>,
    answers: Receiver<(TrackId, Option<f64>)>,
}

impl HeaderReader {
    pub fn spawn() -> std::io::Result<Self> {
        let (jobs, inbox) = crossbeam_channel::unbounded::<(TrackId, PathBuf)>();
        let (outbox, answers) = crossbeam_channel::unbounded();
        std::thread::Builder::new()
            .name("fp-header-reader".to_owned())
            .spawn(move || {
                // Ends when the services drop their side.
                while let Ok((track, path)) = inbox.recv() {
                    // A decoder that panics on a broken file costs that
                    // file's length, never the thread.
                    let secs = catch_unwind(AssertUnwindSafe(|| header_duration(&path)))
                        .unwrap_or_else(|_| {
                            tracing::warn!(path = %path.display(), "reading the file header panicked");
                            None
                        });
                    if outbox.send((track, secs)).is_err() {
                        break;
                    }
                }
            })?;
        Ok(Self { jobs, answers })
    }

    /// Queues a read; `false` once the thread has stopped.
    pub fn submit(&self, track: TrackId, path: PathBuf) -> bool {
        self.jobs.send((track, path)).is_ok()
    }

    /// One answer per read: the length, or `None` when there is none.
    pub fn answers(&self) -> &Receiver<(TrackId, Option<f64>)> {
        &self.answers
    }
}
