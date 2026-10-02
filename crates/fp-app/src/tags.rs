//! The tag worker (feedback 2 spec O23): one thread that reads and writes
//! tags, so neither the interface nor the services thread waits on a disk.
//! It follows the file probe's pattern: jobs in, outcomes out, both on
//! unbounded channels, and the thread ends when its handle is dropped.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
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
    /// Set when the handle is dropped: queued reads are then skipped.
    stop: Arc<AtomicBool>,
}

impl TagWorker {
    /// Starts the thread. `repaint` runs after every outcome (the interface
    /// passes its context's repaint request).
    pub fn spawn(repaint: Box<dyn Fn() + Send>) -> std::io::Result<Self> {
        let (jobs_tx, jobs_rx) = crossbeam_channel::unbounded::<TagJob>();
        let (results_tx, results) = crossbeam_channel::unbounded();
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = Arc::clone(&stop);
        let thread = std::thread::Builder::new()
            .name("fp-tags".to_owned())
            .spawn(move || {
                while let Ok(job) = jobs_rx.recv() {
                    // A read that nobody will see is not worth the disk; a
                    // write is a save the operator asked for and still runs.
                    if stopped.load(Ordering::Acquire) && matches!(job, TagJob::Read { .. }) {
                        continue;
                    }
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
            stop,
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
        // Queued reads are skipped; the job in progress and the queued
        // writes still finish. Closing the channel ends the loop once the
        // queue is empty. The flag is set first so no read slips through.
        self.stop.store(true, Ordering::Release);
        self.jobs = None;
        if let Some(thread) = self.thread.take()
            && thread.join().is_err()
        {
            tracing::error!("the tag worker panicked");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    const QUEUED: usize = 50;

    fn read_job(n: usize) -> TagJob {
        TagJob::Read {
            track: TrackId(n as u64),
            path: PathBuf::from("/nonexistent/fp-tags-test.mp3"),
            limits: Limits::default(),
        }
    }

    /// Runs `queue` while the worker is held inside its first outcome, drops
    /// the worker, and returns how many outcomes it produced in all. The
    /// hold ends only once the drop has set the stop flag, so the result
    /// does not depend on timing.
    fn outcomes_after_drop(queue: impl FnOnce(&TagWorker)) -> usize {
        let count = Arc::new(AtomicUsize::new(0));
        let (started_tx, started_rx) = crossbeam_channel::bounded::<()>(1);
        let (gate_tx, gate_rx) = crossbeam_channel::bounded::<()>(1);
        let counted = Arc::clone(&count);
        let worker = TagWorker::spawn(Box::new(move || {
            if counted.fetch_add(1, Ordering::SeqCst) == 0 {
                let _ = started_tx.send(());
                let _ = gate_rx.recv();
            }
        }))
        .unwrap();
        assert!(worker.submit(read_job(0)));
        started_rx.recv().unwrap();
        queue(&worker);
        let stop = Arc::clone(&worker.stop);
        let releaser = std::thread::spawn(move || {
            while !stop.load(Ordering::Acquire) {
                std::thread::yield_now();
            }
            let _ = gate_tx.send(());
        });
        drop(worker);
        releaser.join().unwrap();
        count.load(Ordering::SeqCst)
    }

    #[test]
    fn dropping_the_worker_skips_the_queued_reads() {
        let outcomes = outcomes_after_drop(|worker| {
            for n in 1..=QUEUED {
                assert!(worker.submit(read_job(n)));
            }
        });
        assert_eq!(outcomes, 1, "only the read in progress finished");
    }

    #[test]
    fn dropping_the_worker_still_runs_the_queued_writes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("x.wav");
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 44_100,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut w = hound::WavWriter::create(&path, spec).unwrap();
        for i in 0..4_410 {
            w.write_sample((i % 100) as i16).unwrap();
        }
        w.finalize().unwrap();
        let limits = Limits::default();
        let before = read_track_tags(&path, &limits);
        let after = TrackTags {
            title: "Saved at shutdown".into(),
            ..before.clone()
        };
        let outcomes = outcomes_after_drop(|worker| {
            for n in 1..=QUEUED {
                assert!(worker.submit(read_job(n)));
            }
            assert!(worker.submit(TagJob::Write {
                track: TrackId(99),
                path: path.clone(),
                before: Box::new(before.clone()),
                after: Box::new(after.clone()),
                limits: limits.clone(),
            }));
        });
        assert_eq!(outcomes, 2, "the read in progress and the write");
        assert_eq!(read_track_tags(&path, &limits).title, "Saved at shutdown");
    }
}
