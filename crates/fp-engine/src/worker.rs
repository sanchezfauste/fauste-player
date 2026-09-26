//! One decoding thread per player (spec §4.5). It opens, decodes and
//! resamples every source of its player and keeps their rings topped up,
//! serving the emptiest ring first. Failures — including panics inside a
//! decoder — are contained to the source that caused them.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::thread::JoinHandle;
use std::time::Duration;

use crossbeam_channel::{Receiver, Sender, TryRecvError};

use crate::decode::FileDecoder;
use crate::resample::{StreamResampler, aligned_preroll};
use crate::source::SourceProducer;

/// Produces interleaved stereo at the bus rate.
pub trait SampleSource: Send {
    /// Appends the next block to `out`; `Ok(false)` at end of stream.
    fn next_block(&mut self, out: &mut Vec<f32>) -> Result<bool, String>;
}

/// Opens a `SampleSource` for `path`, positioned at `from_secs`, at `bus_rate`.
pub type SourceOpener =
    Arc<dyn Fn(&Path, f64, u32) -> Result<Box<dyn SampleSource>, String> + Send + Sync>;

/// Opener for real files: symphonia decoding plus rubato resampling.
pub fn file_opener() -> SourceOpener {
    Arc::new(|path, from_secs, bus_rate| {
        let mut decoder = FileDecoder::open(path)?;
        let file_rate = decoder.sample_rate();
        if file_rate == bus_rate {
            decoder.seek(from_secs)?;
            return Ok(Box::new(FileSource {
                decoder,
                resampler: None,
                block: Vec::new(),
                finished: false,
                skip_frames: 0,
            }));
        }
        let resampler = StreamResampler::new(file_rate, bus_rate)?;
        // Start a little earlier and drop the warm-up, so the first frame is
        // what continuous playback would give at `from_secs`.
        let available = (from_secs.max(0.0) * f64::from(file_rate)).floor() as usize;
        let (mut pre_in, mut pre_out) =
            aligned_preroll(file_rate, bus_rate, resampler.warmup_input_frames());
        if pre_in > available {
            // Not enough audio before the start: use the largest aligned
            // pre-roll that fits (possibly none, at the very start).
            let (step_in, step_out) = aligned_preroll(file_rate, bus_rate, 1);
            let steps = available / step_in.max(1);
            pre_in = steps * step_in;
            pre_out = steps * step_out;
        }
        decoder.seek(from_secs - pre_in as f64 / f64::from(file_rate))?;
        Ok(Box::new(FileSource {
            decoder,
            resampler: Some(resampler),
            block: Vec::new(),
            finished: false,
            skip_frames: pre_out,
        }))
    })
}

struct FileSource {
    decoder: FileDecoder,
    resampler: Option<StreamResampler>,
    block: Vec<f32>,
    finished: bool,
    /// Output frames of pre-roll still to drop.
    skip_frames: usize,
}

impl SampleSource for FileSource {
    fn next_block(&mut self, out: &mut Vec<f32>) -> Result<bool, String> {
        if self.finished {
            return Ok(false);
        }
        let Some(resampler) = self.resampler.as_mut() else {
            return self.decoder.next_block(out);
        };
        self.block.clear();
        let start = out.len();
        if self.decoder.next_block(&mut self.block)? {
            resampler.push(&self.block, out)?;
        } else {
            resampler.finish(out)?;
            self.finished = true;
        }
        if self.skip_frames > 0 {
            let produced = (out.len() - start) / 2;
            let drop = self.skip_frames.min(produced);
            out.drain(start..start + drop * 2);
            self.skip_frames -= drop;
        }
        Ok(true)
    }
}

/// Identifies a source within its worker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceKey(pub u64);

/// How a source is bounded (Phase 2 spec P2.4).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LoadOptions {
    /// The source ends here (seconds in the file); `None` is the end of the file.
    pub until_secs: Option<f64>,
    /// At the end, start again at `from_secs` in the same ring, without a gap.
    pub looped: bool,
}

pub enum WorkerCommand {
    Load {
        key: SourceKey,
        path: PathBuf,
        from_secs: f64,
        producer: SourceProducer,
        options: LoadOptions,
    },
    Drop {
        key: SourceKey,
    },
    Shutdown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerFailure {
    pub key: SourceKey,
    pub error: String,
}

struct Job {
    key: SourceKey,
    path: PathBuf,
    from_secs: f64,
    producer: SourceProducer,
    source: Option<Box<dyn SampleSource>>,
    pending: Vec<f32>,
    done: bool,
    /// Frames per pass when bounded by `until`.
    limit_frames: Option<u64>,
    looped: bool,
    /// Frames taken from the source in the current pass.
    pass_frames: u64,
}

/// Handle to a running worker thread; dropping it stops the thread.
pub struct PlayerWorker {
    commands: Sender<WorkerCommand>,
    thread: Option<JoinHandle<()>>,
}

impl PlayerWorker {
    /// Spawns the worker. A source becomes ready once `ready_frames` are
    /// buffered; failures are sent on `failures`.
    pub fn spawn(
        name: &str,
        opener: SourceOpener,
        bus_rate: u32,
        ready_frames: usize,
        failures: Sender<WorkerFailure>,
    ) -> std::io::Result<Self> {
        let (tx, rx) = crossbeam_channel::unbounded();
        let thread = std::thread::Builder::new()
            .name(name.to_owned())
            .spawn(move || run(&rx, &opener, bus_rate, ready_frames, &failures))?;
        Ok(Self {
            commands: tx,
            thread: Some(thread),
        })
    }

    pub fn load(&self, key: SourceKey, path: PathBuf, from_secs: f64, producer: SourceProducer) {
        self.load_with(key, path, from_secs, producer, LoadOptions::default());
    }

    pub fn load_with(
        &self,
        key: SourceKey,
        path: PathBuf,
        from_secs: f64,
        producer: SourceProducer,
        options: LoadOptions,
    ) {
        let _ = self.commands.send(WorkerCommand::Load {
            key,
            path,
            from_secs,
            producer,
            options,
        });
    }

    pub fn drop_source(&self, key: SourceKey) {
        let _ = self.commands.send(WorkerCommand::Drop { key });
    }
}

impl Drop for PlayerWorker {
    fn drop(&mut self) {
        let _ = self.commands.send(WorkerCommand::Shutdown);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn run(
    commands: &Receiver<WorkerCommand>,
    opener: &SourceOpener,
    bus_rate: u32,
    ready_frames: usize,
    failures: &Sender<WorkerFailure>,
) {
    let mut jobs: Vec<Job> = Vec::new();
    loop {
        let busy = jobs.iter().any(|j| !j.done && j.producer.free_frames() > 0);
        let first = if busy {
            match commands.try_recv() {
                Ok(c) => Some(c),
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => return,
            }
        } else {
            match commands.recv_timeout(Duration::from_millis(10)) {
                Ok(c) => Some(c),
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => None,
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => return,
            }
        };
        for command in first
            .into_iter()
            .chain(std::iter::from_fn(|| commands.try_recv().ok()))
        {
            match command {
                WorkerCommand::Load {
                    key,
                    path,
                    from_secs,
                    producer,
                    options,
                } => {
                    jobs.retain(|j| j.key != key);
                    let limit_frames = options
                        .until_secs
                        .filter(|u| u.is_finite())
                        .map(|u| ((u - from_secs).max(0.0) * f64::from(bus_rate)).round() as u64);
                    jobs.push(Job {
                        key,
                        path,
                        from_secs,
                        producer,
                        source: None,
                        pending: Vec::new(),
                        done: false,
                        limit_frames,
                        looped: options.looped,
                        pass_frames: 0,
                    });
                }
                WorkerCommand::Drop { key } => jobs.retain(|j| j.key != key),
                WorkerCommand::Shutdown => return,
            }
        }
        jobs.retain(|j| !j.producer.is_abandoned());
        // Serve the job with the least audio buffered first.
        if let Some(job) = jobs
            .iter_mut()
            .filter(|j| !j.done && j.producer.free_frames() > 0)
            .min_by_key(|j| j.producer.buffered_frames())
        {
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                step(job, opener, bus_rate, ready_frames)
            }));
            let error = match outcome {
                Ok(Ok(())) => None,
                Ok(Err(e)) => Some(e),
                Err(_) => Some("decoder panicked".to_owned()),
            };
            if let Some(error) = error {
                job.done = true;
                job.producer.shared.failed.store(true, Ordering::Release);
                job.producer.shared.ready.store(true, Ordering::Release);
                let _ = failures.send(WorkerFailure {
                    key: job.key,
                    error,
                });
            }
        }
    }
}

fn finish(job: &mut Job) -> Result<(), String> {
    job.done = true;
    job.producer.shared.eof.store(true, Ordering::Release);
    job.producer.shared.ready.store(true, Ordering::Release);
    Ok(())
}

/// Opens the job if needed and moves at most one decoded block into its ring.
fn step(
    job: &mut Job,
    opener: &SourceOpener,
    bus_rate: u32,
    ready_frames: usize,
) -> Result<(), String> {
    if job.limit_frames == Some(0) {
        return finish(job);
    }
    if job.source.is_none() {
        job.source = Some(opener(&job.path, job.from_secs, bus_rate)?);
    }
    if job.pending.is_empty() {
        let at_limit = job.limit_frames.is_some_and(|l| job.pass_frames >= l);
        let more = !at_limit
            && match job.source.as_mut() {
                Some(source) => source.next_block(&mut job.pending)?,
                None => false,
            };
        if let Some(limit) = job.limit_frames {
            // Cut what goes past `until`.
            let room = limit.saturating_sub(job.pass_frames);
            let frames = (job.pending.len() / 2) as u64;
            if frames > room {
                job.pending
                    .truncate(usize::try_from(room).unwrap_or(usize::MAX) * 2);
            }
        }
        job.pass_frames += (job.pending.len() / 2) as u64;
        if job.pending.is_empty() && (at_limit || !more) {
            // End of a pass: loop (unless the pass was empty), or finish.
            if job.looped && job.pass_frames > 0 {
                job.producer
                    .shared
                    .loop_frames
                    .store(job.pass_frames, Ordering::Release);
                job.source = Some(opener(&job.path, job.from_secs, bus_rate)?);
                job.pass_frames = 0;
                return Ok(());
            }
            return finish(job);
        }
    }
    let pushed = job.producer.push(&job.pending);
    job.pending.drain(..pushed);
    if job.producer.buffered_frames() >= ready_frames || job.producer.free_frames() == 0 {
        job.producer.shared.ready.store(true, Ordering::Release);
    }
    Ok(())
}
