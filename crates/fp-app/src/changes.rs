//! The change-check thread (EE7): looks at the files of a batch of tracks off
//! the UI and services threads and tells which changed since their stored
//! stamp and which had none yet. There is no periodic polling: callers send
//! a batch when something asks for it.

use std::io;
use std::path::PathBuf;

use crossbeam_channel::{Receiver, Sender};
use fp_model::{FileStamp, TrackId};

/// What a look at one file says.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Check {
    /// The stamp is the stored one.
    Same,
    /// Size or modification time differ from the stored stamp.
    Changed,
    /// Nothing was stored yet: this is the stamp to store.
    Unstamped(FileStamp),
    /// The file could not be looked at (missing files included, which keep
    /// the existing missing-file rules).
    Unknown,
}

/// Compares the stored stamp of a track with the file's stamp now.
pub fn compare(stored: Option<FileStamp>, now: io::Result<FileStamp>) -> Check {
    match (stored, now) {
        (_, Err(_)) => Check::Unknown,
        (None, Ok(now)) => Check::Unstamped(now),
        (Some(stored), Ok(now)) if stored == now => Check::Same,
        (Some(_), Ok(_)) => Check::Changed,
    }
}

/// One track of a batch: its id, its file and the stamp stored for it.
pub type Item = (TrackId, PathBuf, Option<FileStamp>);

/// The result of one batch.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChangeAnswer {
    pub changed: Vec<TrackId>,
    pub unstamped: Vec<(TrackId, FileStamp)>,
}

/// The helper thread. One batch is in flight at a time.
pub struct ChangeCheck {
    jobs: Sender<Vec<Item>>,
    answers: Receiver<ChangeAnswer>,
    busy: bool,
    /// The thread ended (it can only end on its own by panicking).
    gone: bool,
}

impl ChangeCheck {
    /// Starts the thread; `None` (logged) when it cannot start.
    pub fn spawn() -> Option<Self> {
        Self::spawn_with(false)
    }

    /// A check whose thread takes one batch and ends without answering, as
    /// one that panicked (tests).
    #[cfg(feature = "test-hooks")]
    pub fn spawn_dying() -> Option<Self> {
        Self::spawn_with(true)
    }

    fn spawn_with(dies: bool) -> Option<Self> {
        let (jobs, job_rx) = crossbeam_channel::unbounded::<Vec<Item>>();
        let (answer_tx, answers) = crossbeam_channel::unbounded::<ChangeAnswer>();
        let spawned = std::thread::Builder::new()
            .name("fp-change-check".to_owned())
            .spawn(move || {
                // Ends when the owner drops its side.
                while let Ok(batch) = job_rx.recv() {
                    if dies {
                        break;
                    }
                    if answer_tx.send(run(batch)).is_err() {
                        break;
                    }
                }
            });
        match spawned {
            Ok(_) => Some(Self {
                jobs,
                answers,
                busy: false,
                gone: false,
            }),
            Err(e) => {
                tracing::error!(error = %e, "cannot start the change check; changed files go unnoticed");
                None
            }
        }
    }

    /// Hands a batch over. `false` when one is still in flight or the
    /// thread is gone.
    pub fn send(&mut self, batch: Vec<Item>) -> bool {
        if self.busy || self.jobs.send(batch).is_err() {
            return false;
        }
        self.busy = true;
        true
    }

    /// The answers that arrived, without blocking. A thread found gone
    /// (its side of the channel closed) clears `busy` and sets `gone`.
    pub fn answers(&mut self) -> Vec<ChangeAnswer> {
        let mut answers = Vec::new();
        loop {
            match self.answers.try_recv() {
                Ok(answer) => answers.push(answer),
                Err(crossbeam_channel::TryRecvError::Empty) => break,
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    self.gone = true;
                    self.busy = false;
                    break;
                }
            }
        }
        if !answers.is_empty() {
            self.busy = false;
        }
        answers
    }

    /// The thread has ended: no batch will be answered any more.
    pub fn gone(&self) -> bool {
        self.gone
    }

    /// A batch was sent and its answer has not been taken.
    pub fn busy(&self) -> bool {
        self.busy
    }
}

fn stamp_now(path: &std::path::Path) -> io::Result<FileStamp> {
    let meta = std::fs::metadata(path)?;
    FileStamp::from_metadata(meta.len(), meta.modified()?)
        .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidData))
}

fn run(batch: Vec<Item>) -> ChangeAnswer {
    let mut answer = ChangeAnswer::default();
    for (id, path, stored) in batch {
        match compare(stored, stamp_now(&path)) {
            Check::Changed => answer.changed.push(id),
            Check::Unstamped(stamp) => answer.unstamped.push((id, stamp)),
            Check::Same | Check::Unknown => {}
        }
    }
    answer
}
