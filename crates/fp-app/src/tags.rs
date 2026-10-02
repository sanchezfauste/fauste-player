//! The tag worker (feedback 2 spec O23): one thread that reads and writes
//! tags, so neither the interface nor the services thread waits on a disk.
//! It follows the file probe's pattern: jobs in, outcomes out, both on
//! unbounded channels, and the thread ends when its handle is dropped.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;
use std::thread::JoinHandle;

use crossbeam_channel::{Receiver, Sender};
use fp_analysis::tags::{TagWriteError, read_track_tags, write_tags};
use fp_model::{Limits, TrackId, TrackTags};

/// One piece of work for the worker.
#[derive(Debug, Clone)]
pub enum TagJob {
    /// Read the tags of a file (the tag-only pass).
    Read {
        track: TrackId,
        path: PathBuf,
        limits: Limits,
    },
    /// Write the fields where `after` differs from `before`, then read the
    /// file again.
    Write {
        track: TrackId,
        path: PathBuf,
        before: Box<TrackTags>,
        after: Box<TrackTags>,
        limits: Limits,
    },
}

/// What a job produced.
#[derive(Debug, Clone, PartialEq)]
pub enum TagOutcome {
    Read {
        track: TrackId,
        tags: TrackTags,
    },
    /// `Ok` carries the tags as the file holds them after the write.
    Written {
        track: TrackId,
        result: Result<TrackTags, TagWriteError>,
    },
}

fn run(job: TagJob) -> TagOutcome {
    match job {
        TagJob::Read {
            track,
            path,
            limits,
        } => TagOutcome::Read {
            track,
            tags: read_track_tags(&path, &limits),
        },
        TagJob::Write {
            track,
            path,
            before,
            after,
            limits,
        } => {
            let result = write_tags(&path, &before, &after, &limits)
                .map(|()| read_track_tags(&path, &limits));
            TagOutcome::Written { track, result }
        }
    }
}

/// A panic inside a job is a failed job, not a dead worker.
fn run_contained(job: TagJob) -> TagOutcome {
    let (track, write) = match &job {
        TagJob::Read { track, .. } => (*track, false),
        TagJob::Write { track, .. } => (*track, true),
    };
    catch_unwind(AssertUnwindSafe(|| run(job))).unwrap_or_else(|_| {
        tracing::error!(?track, "a tag job panicked");
        if write {
            TagOutcome::Written {
                track,
                result: Err(TagWriteError::Other("the tag code failed".to_owned())),
            }
        } else {
            TagOutcome::Read {
                track,
                tags: TrackTags::default(),
            }
        }
    })
}

pub struct TagWorker {
    jobs: Option<Sender<TagJob>>,
    results: Receiver<TagOutcome>,
    thread: Option<JoinHandle<()>>,
}

impl TagWorker {
    /// Starts the thread. `repaint` runs after every outcome (the interface
    /// passes its context's repaint request).
    pub fn spawn(repaint: Box<dyn Fn() + Send>) -> std::io::Result<Self> {
        let (jobs_tx, jobs_rx) = crossbeam_channel::unbounded::<TagJob>();
        let (results_tx, results) = crossbeam_channel::unbounded();
        let thread = std::thread::Builder::new()
            .name("fp-tags".to_owned())
            .spawn(move || {
                while let Ok(job) = jobs_rx.recv() {
                    if results_tx.send(run_contained(job)).is_err() {
                        break;
                    }
                    repaint();
                }
            })?;
        Ok(Self {
            jobs: Some(jobs_tx),
            results,
            thread: Some(thread),
        })
    }

    /// Queues a job; never blocks. `false` if the worker is gone.
    pub fn submit(&self, job: TagJob) -> bool {
        self.jobs.as_ref().is_some_and(|tx| tx.send(job).is_ok())
    }

    pub fn results(&self) -> &Receiver<TagOutcome> {
        &self.results
    }
}

impl Drop for TagWorker {
    fn drop(&mut self) {
        // Closing the channel ends the loop after the job in progress.
        self.jobs = None;
        if let Some(thread) = self.thread.take()
            && thread.join().is_err()
        {
            tracing::error!("the tag worker panicked");
        }
    }
}
