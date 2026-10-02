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
use fp_analysis::tags::{
    CoverError, TagWriteError, load_cover_file, read_tag_sheet, read_track_tags,
    with_cover_thumbnail, write_tag_sheet, write_tags,
};
use fp_model::{CoverArt, Limits, TagSheet, TrackId, TrackTags};

/// One piece of work for the worker.
#[derive(Debug, Clone)]
pub enum TagJob {
    /// Read the whole tag sheet of a file (the editor opening). The cover
    /// of the sheet comes with a thumbnail of at most `thumb_px` pixels.
    ReadSheet {
        track: TrackId,
        path: PathBuf,
        limits: Limits,
        thumb_px: u32,
    },
    /// Write the fields and the cover where `after` differs from `before`,
    /// then read the file again, both as a sheet (its cover with a
    /// thumbnail of at most `thumb_px` pixels) and as the summary the
    /// library keeps.
    WriteSheet {
        track: TrackId,
        path: PathBuf,
        before: Box<TagSheet>,
        after: Box<TagSheet>,
        limits: Limits,
        thumb_px: u32,
    },
    /// Read the image file `path` as a new front cover for the editor of
    /// `track`: it must be a JPEG or PNG that decodes within the cover
    /// limits.
    LoadCover {
        track: TrackId,
        /// The editor session that asked, returned with the answer.
        session: u64,
        path: PathBuf,
        limits: Limits,
        thumb_px: u32,
    },
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

/// What the file holds after a sheet was written.
#[derive(Debug, Clone, PartialEq)]
pub struct SheetSaved {
    /// For the library (`Command::ApplyTags`).
    pub tags: TrackTags,
    /// For comparing with what was written; `None` if the file could not be
    /// read again as a sheet.
    pub sheet: Option<TagSheet>,
}

/// What a job produced.
#[derive(Debug, Clone, PartialEq)]
pub enum TagOutcome {
    /// `None`: the format has no writable tags or the file cannot be read.
    SheetRead {
        track: TrackId,
        sheet: Option<Box<TagSheet>>,
    },
    SheetWritten {
        track: TrackId,
        result: Result<Box<SheetSaved>, TagWriteError>,
    },
    /// The answer to `LoadCover`.
    CoverLoaded {
        track: TrackId,
        /// The session of the `LoadCover` job.
        session: u64,
        result: Result<CoverArt, CoverError>,
    },
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

/// The sheet of `path` with the thumbnail of its cover decoded.
fn read_sheet(path: &std::path::Path, limits: &Limits, thumb_px: u32) -> Option<TagSheet> {
    read_tag_sheet(path, limits).map(|sheet| with_cover_thumbnail(sheet, limits, thumb_px))
}

fn run(job: TagJob) -> TagOutcome {
    match job {
        TagJob::ReadSheet {
            track,
            path,
            limits,
            thumb_px,
        } => TagOutcome::SheetRead {
            track,
            sheet: read_sheet(&path, &limits, thumb_px).map(Box::new),
        },
        TagJob::WriteSheet {
            track,
            path,
            before,
            after,
            limits,
            thumb_px,
        } => {
            let result = write_tag_sheet(&path, &before, &after, &limits).map(|()| {
                Box::new(SheetSaved {
                    tags: read_track_tags(&path, &limits),
                    sheet: read_sheet(&path, &limits, thumb_px),
                })
            });
            TagOutcome::SheetWritten { track, result }
        }
        TagJob::LoadCover {
            track,
            session,
            path,
            limits,
            thumb_px,
        } => TagOutcome::CoverLoaded {
            track,
            session,
            result: load_cover_file(&path, &limits, thumb_px),
        },
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

/// What a job is, for answering when it panics and for skipping it at
/// shutdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Read,
    ReadSheet,
    LoadCover,
    Write,
    WriteSheet,
}

impl TagJob {
    fn kind(&self) -> Kind {
        match self {
            Self::Read { .. } => Kind::Read,
            Self::ReadSheet { .. } => Kind::ReadSheet,
            Self::LoadCover { .. } => Kind::LoadCover,
            Self::Write { .. } => Kind::Write,
            Self::WriteSheet { .. } => Kind::WriteSheet,
        }
    }

    fn track(&self) -> TrackId {
        match self {
            Self::Read { track, .. }
            | Self::ReadSheet { track, .. }
            | Self::LoadCover { track, .. }
            | Self::Write { track, .. }
            | Self::WriteSheet { track, .. } => *track,
        }
    }

    /// A read nobody will see is not worth the disk once the worker is
    /// being dropped; a write is a save the operator asked for.
    fn is_read(&self) -> bool {
        matches!(self.kind(), Kind::Read | Kind::ReadSheet | Kind::LoadCover)
    }
}

/// A panic inside a job is a failed job, not a dead worker.
fn run_contained(job: TagJob) -> TagOutcome {
    let (track, kind) = (job.track(), job.kind());
    let session = match &job {
        TagJob::LoadCover { session, .. } => *session,
        _ => 0,
    };
    catch_unwind(AssertUnwindSafe(|| run(job))).unwrap_or_else(|_| {
        tracing::error!(?track, "a tag job panicked");
        let failed = || TagWriteError::Other("the tag code failed".to_owned());
        match kind {
            Kind::Read => TagOutcome::Read {
                track,
                tags: TrackTags::default(),
            },
            Kind::ReadSheet => TagOutcome::SheetRead { track, sheet: None },
            Kind::LoadCover => TagOutcome::CoverLoaded {
                track,
                session,
                result: Err(CoverError::Unreadable("the image code failed".to_owned())),
            },
            Kind::Write => TagOutcome::Written {
                track,
                result: Err(failed()),
            },
            Kind::WriteSheet => TagOutcome::SheetWritten {
                track,
                result: Err(failed()),
            },
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
                    if stopped.load(Ordering::Acquire) && job.is_read() {
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

    fn wav(dir: &std::path::Path, name: &str) -> PathBuf {
        let path = dir.join(name);
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
        path
    }

    fn next_outcome(worker: &TagWorker) -> TagOutcome {
        worker
            .results()
            .recv_timeout(std::time::Duration::from_secs(10))
            .expect("an outcome")
    }

    fn sheet_job(track: u64, path: &std::path::Path) -> TagJob {
        TagJob::ReadSheet {
            track: TrackId(track),
            path: path.to_path_buf(),
            limits: Limits::default(),
            thumb_px: 64,
        }
    }

    #[test]
    fn a_sheet_read_answers_with_the_sheet() {
        let dir = tempfile::tempdir().unwrap();
        let path = wav(dir.path(), "x.wav");
        let worker = TagWorker::spawn(Box::new(|| {})).unwrap();
        assert!(worker.submit(sheet_job(7, &path)));
        match next_outcome(&worker) {
            TagOutcome::SheetRead {
                track,
                sheet: Some(sheet),
            } => {
                assert_eq!(track, TrackId(7));
                assert!(sheet.can_store(fp_model::TagField::AlbumArtist));
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn a_sheet_read_of_a_file_without_tags_answers_none() {
        let worker = TagWorker::spawn(Box::new(|| {})).unwrap();
        assert!(worker.submit(sheet_job(1, std::path::Path::new("/nonexistent/x.wav"))));
        assert_eq!(
            next_outcome(&worker),
            TagOutcome::SheetRead {
                track: TrackId(1),
                sheet: None
            }
        );
    }

    #[test]
    fn a_sheet_write_answers_with_the_file_as_it_is_now() {
        let dir = tempfile::tempdir().unwrap();
        let path = wav(dir.path(), "x.wav");
        let limits = Limits::default();
        let before = read_tag_sheet(&path, &limits).unwrap();
        let mut after = before.clone();
        after.set_text(fp_model::TagField::Title, "Sheet title", false);
        after.set_text(fp_model::TagField::Mood, "Calm", false);
        let worker = TagWorker::spawn(Box::new(|| {})).unwrap();
        assert!(worker.submit(TagJob::WriteSheet {
            track: TrackId(3),
            path: path.clone(),
            before: Box::new(before),
            after: Box::new(after.clone()),
            limits,
            thumb_px: 64,
        }));
        match next_outcome(&worker) {
            TagOutcome::SheetWritten {
                track,
                result: Ok(saved),
            } => {
                assert_eq!(track, TrackId(3));
                assert_eq!(saved.tags.title, "Sheet title");
                assert_eq!(saved.sheet, Some(after), "the file kept all of it");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn a_sheet_write_that_fails_says_why() {
        let path = PathBuf::from("/nonexistent/folder/x.wav");
        let before = TagSheet::new([fp_model::TagField::Title], 0, false);
        let mut after = before.clone();
        after.set_text(fp_model::TagField::Title, "x", false);
        let worker = TagWorker::spawn(Box::new(|| {})).unwrap();
        assert!(worker.submit(TagJob::WriteSheet {
            track: TrackId(4),
            path,
            before: Box::new(before),
            after: Box::new(after),
            limits: Limits::default(),
            thumb_px: 64,
        }));
        assert_eq!(
            next_outcome(&worker),
            TagOutcome::SheetWritten {
                track: TrackId(4),
                result: Err(TagWriteError::NotFound)
            }
        );
    }

    #[test]
    fn dropping_the_worker_skips_queued_sheet_reads_and_runs_sheet_writes() {
        let dir = tempfile::tempdir().unwrap();
        let path = wav(dir.path(), "x.wav");
        let limits = Limits::default();
        let before = read_tag_sheet(&path, &limits).unwrap();
        let mut after = before.clone();
        after.set_text(fp_model::TagField::Title, "Saved at shutdown", false);
        let outcomes = outcomes_after_drop(|worker| {
            for n in 1..=QUEUED {
                assert!(worker.submit(sheet_job(n as u64, &path)));
            }
            assert!(worker.submit(TagJob::WriteSheet {
                track: TrackId(99),
                path: path.clone(),
                before: Box::new(before.clone()),
                after: Box::new(after.clone()),
                limits: limits.clone(),
                thumb_px: 64,
            }));
        });
        assert_eq!(outcomes, 2, "the read in progress and the write");
        let sheet = read_tag_sheet(&path, &limits).unwrap();
        assert_eq!(
            sheet.values(fp_model::TagField::Title),
            ["Saved at shutdown"]
        );
    }

    fn png(width: u32) -> Vec<u8> {
        let img = image::RgbImage::from_pixel(width, 30, image::Rgb([200, 30, 30]));
        let mut out = Vec::new();
        image::DynamicImage::ImageRgb8(img)
            .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
            .unwrap();
        out
    }

    fn load_job(track: u64, path: &std::path::Path, limits: Limits) -> TagJob {
        TagJob::LoadCover {
            track: TrackId(track),
            session: 7,
            path: path.to_path_buf(),
            limits,
            thumb_px: 16,
        }
    }

    #[test]
    fn a_cover_load_answers_with_a_front_cover_and_its_thumbnail() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cover.png");
        std::fs::write(&path, png(40)).unwrap();
        let worker = TagWorker::spawn(Box::new(|| {})).unwrap();
        assert!(worker.submit(load_job(5, &path, Limits::default())));
        match next_outcome(&worker) {
            TagOutcome::CoverLoaded {
                track,
                session,
                result: Ok(cover),
            } => {
                assert_eq!((track, session), (TrackId(5), 7));
                assert!(cover.is_front());
                assert_eq!(cover.data(), png(40).as_slice());
                assert!(cover.thumb_png().is_some());
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn a_cover_load_that_fails_says_why() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cover.png");
        std::fs::write(&path, png(40)).unwrap();
        let small = Limits {
            max_cover_bytes: 10,
            ..Limits::default()
        };
        let worker = TagWorker::spawn(Box::new(|| {})).unwrap();
        assert!(worker.submit(load_job(5, &path, small)));
        assert_eq!(
            next_outcome(&worker),
            TagOutcome::CoverLoaded {
                track: TrackId(5),
                session: 7,
                result: Err(CoverError::TooLarge)
            }
        );
    }

    #[test]
    fn a_sheet_write_with_a_new_cover_answers_with_the_cover_and_its_thumbnail() {
        let dir = tempfile::tempdir().unwrap();
        let path = wav(dir.path(), "x.wav");
        let limits = Limits::default();
        let before = read_tag_sheet(&path, &limits).unwrap();
        assert!(before.can_store_cover() && before.cover().is_none());
        let mut after = before.clone();
        after.set_cover(Some(CoverArt::new(png(40), true)));
        let worker = TagWorker::spawn(Box::new(|| {})).unwrap();
        assert!(worker.submit(TagJob::WriteSheet {
            track: TrackId(3),
            path: path.clone(),
            before: Box::new(before),
            after: Box::new(after.clone()),
            limits,
            thumb_px: 16,
        }));
        match next_outcome(&worker) {
            TagOutcome::SheetWritten {
                result: Ok(saved), ..
            } => {
                let sheet = saved.sheet.expect("a sheet");
                assert_eq!(sheet.cover(), after.cover(), "the file kept the cover");
                assert!(sheet.cover().unwrap().thumb_png().is_some());
            }
            other => panic!("{other:?}"),
        }
        // The next sheet read shows it, with a thumbnail.
        assert!(worker.submit(sheet_job(3, &path)));
        match next_outcome(&worker) {
            TagOutcome::SheetRead {
                sheet: Some(sheet), ..
            } => assert!(sheet.cover().unwrap().thumb_png().is_some()),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn dropping_the_worker_skips_a_queued_cover_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cover.png");
        std::fs::write(&path, png(40)).unwrap();
        let outcomes = outcomes_after_drop(|worker| {
            for n in 1..=QUEUED {
                assert!(worker.submit(load_job(n as u64, &path, Limits::default())));
            }
        });
        assert_eq!(outcomes, 1, "only the load in progress");
    }
}
