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
use fp_decode::priority::Priority;

use crate::decode::FileDecoder;
use crate::resample::{StreamResampler, aligned_preroll};
use crate::source::SourceProducer;
use fp_backends::dsd::{DSD_SILENCE, silence_sample, word_to_sample};

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

/// Produces the two streams of a DSD source in step.
pub trait DsdSampleSource: Send {
    /// Appends equal numbers of stereo frames of PCM (at the word rate) and
    /// of DSD words; `false` at the end.
    fn next_pair(&mut self, pcm: &mut Vec<f32>, dsd: &mut Vec<f32>) -> Result<bool, String>;
}

/// Opens a `DsdSampleSource` for `path`, positioned at `from_secs`, for a
/// word rate (the DSD rate over 16).
pub type DsdOpener =
    Arc<dyn Fn(&Path, f64, u32) -> Result<Box<dyn DsdSampleSource>, String> + Send + Sync>;

/// Samples (not frames) each queue of a `DsdFileSource` is refilled to.
const DSD_QUEUE_MIN: usize = 8_192;

/// A DSD file for output without conversion: its raw words, and its PCM
/// conversion at the same rate for the meters and for a switch to PCM.
struct DsdFileSource {
    pcm: Box<dyn SampleSource>,
    raw: fp_decode::DsdRawReader,
    /// Bytes read per channel and not yet made into words.
    bytes: Vec<Vec<u8>>,
    pcm_queue: Vec<f32>,
    word_queue: Vec<f32>,
    pcm_done: bool,
    raw_done: bool,
}

/// Opener for DSD files: `(path, from_secs, word_rate)`.
pub fn dsd_file_opener() -> DsdOpener {
    Arc::new(|path, from_secs, word_rate| {
        let mut raw = fp_decode::DsdRawReader::open(path)?;
        if !(1..=2).contains(&raw.channels()) {
            return Err("only mono or stereo DSD goes out unchanged".to_owned());
        }
        if raw.dsd_rate() / 16 != word_rate {
            return Err(format!("word rate {word_rate} does not match the file"));
        }
        raw.seek(from_secs)?;
        let pcm = file_opener()(path, from_secs, word_rate)?;
        Ok(Box::new(DsdFileSource {
            pcm,
            bytes: vec![Vec::new(); raw.channels()],
            raw,
            pcm_queue: Vec::new(),
            word_queue: Vec::new(),
            pcm_done: false,
            raw_done: false,
        }))
    })
}

impl DsdFileSource {
    /// Reads more bytes and turns every complete pair into a word frame.
    fn refill_words(&mut self) -> Result<(), String> {
        if !self.raw.next_bytes(&mut self.bytes)? {
            self.raw_done = true;
        }
        let have = self.bytes.iter().map(Vec::len).min().unwrap_or(0);
        // At the end an odd trailing byte is padded with the idle byte.
        let take = if self.raw_done {
            have.div_ceil(2)
        } else {
            have / 2
        };
        for k in 0..take {
            let mut words = [0.0f32; 2];
            for (c, word) in words.iter_mut().enumerate() {
                // Mono is duplicated to both channels.
                let channel = self.bytes.get(c).or_else(|| self.bytes.first());
                let byte = |i: usize| {
                    channel
                        .and_then(|b| b.get(2 * k + i))
                        .copied()
                        .unwrap_or(DSD_SILENCE)
                };
                *word = word_to_sample(byte(0), byte(1));
            }
            self.word_queue.extend_from_slice(&words);
        }
        for channel in &mut self.bytes {
            let used = (take * 2).min(channel.len());
            channel.drain(..used);
        }
        if self.raw_done {
            for channel in &mut self.bytes {
                channel.clear();
            }
        }
        Ok(())
    }
}

impl DsdSampleSource for DsdFileSource {
    fn next_pair(&mut self, pcm: &mut Vec<f32>, dsd: &mut Vec<f32>) -> Result<bool, String> {
        while !self.raw_done && self.word_queue.len() < DSD_QUEUE_MIN {
            self.refill_words()?;
        }
        while !self.pcm_done && self.pcm_queue.len() < DSD_QUEUE_MIN {
            if !self.pcm.next_block(&mut self.pcm_queue)? {
                self.pcm_done = true;
            }
        }
        // Keep the two equal: pad the side that ended first.
        if self.raw_done && self.word_queue.len() < self.pcm_queue.len() {
            self.word_queue
                .resize(self.pcm_queue.len(), silence_sample());
        }
        if self.pcm_done && self.pcm_queue.len() < self.word_queue.len() {
            self.pcm_queue.resize(self.word_queue.len(), 0.0);
        }
        let n = self.pcm_queue.len().min(self.word_queue.len());
        let n = n - n % 2;
        if n == 0 {
            return Ok(false);
        }
        pcm.extend(self.pcm_queue.drain(..n));
        dsd.extend(self.word_queue.drain(..n));
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
    /// The rate to produce, that of the bus the source plays on; `None` is
    /// the rate the worker was spawned with.
    pub rate: Option<u32>,
    /// Frames (at the produced rate) before `until_secs` over which a
    /// pass that ends there, and does not loop, fades linearly to zero, so
    /// the cut is not a step. 0 is no fade. A looped source keeps its
    /// splice.
    pub fade_out_frames: u64,
    /// Open the file as a DSD source (raw words beside the PCM conversion).
    /// `looped` is refused for it. Its fade only touches the PCM ring.
    pub dsd: bool,
    /// Frames (at the produced rate) buffered before the source is ready;
    /// `None` is the threshold the worker was spawned with, scaled to the
    /// rate. Lets a new `tuning.ready_threshold_ms` reach the next source
    /// (live settings spec §7).
    pub ready_frames: Option<usize>,
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

enum JobSource {
    Pcm(Box<dyn SampleSource>),
    Dsd(Box<dyn DsdSampleSource>),
}

struct Job {
    key: SourceKey,
    path: PathBuf,
    from_secs: f64,
    producer: SourceProducer,
    source: Option<JobSource>,
    pending: Vec<f32>,
    /// DSD words matching `pending` frame for frame (empty for PCM).
    pending_dsd: Vec<f32>,
    dsd: bool,
    done: bool,
    /// Frames per pass when bounded by `until`.
    limit_frames: Option<u64>,
    looped: bool,
    /// Frames of the de-click fade before `limit_frames` (see `LoadOptions`).
    fade_frames: u64,
    /// Frames taken from the source in the current pass.
    pass_frames: u64,
    /// Output rate of this source.
    rate: u32,
    /// The ready threshold this source was opened with (see `LoadOptions`).
    ready_frames: Option<usize>,
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
        Self::spawn_with_dsd(
            name,
            opener,
            dsd_file_opener(),
            bus_rate,
            ready_frames,
            failures,
        )
    }

    /// Like `spawn`, with the opener of DSD sources given (tests).
    pub fn spawn_with_dsd(
        name: &str,
        opener: SourceOpener,
        dsd_opener: DsdOpener,
        bus_rate: u32,
        ready_frames: usize,
        failures: Sender<WorkerFailure>,
    ) -> std::io::Result<Self> {
        let (tx, rx) = crossbeam_channel::unbounded();
        let thread = std::thread::Builder::new()
            .name(name.to_owned())
            .spawn(move || run(&rx, &opener, &dsd_opener, bus_rate, ready_frames, &failures))?;
        Ok(Self {
            commands: tx,
            thread: Some(thread),
        })
    }

    pub fn load(&self, key: SourceKey, path: PathBuf, from_secs: f64, producer: SourceProducer) {
        self.load_with(key, path, from_secs, producer, LoadOptions::default());
    }

    /// Loads a whole file at `rate`.
    pub fn load_at(
        &self,
        key: SourceKey,
        path: PathBuf,
        from_secs: f64,
        producer: SourceProducer,
        rate: u32,
    ) {
        let options = LoadOptions {
            rate: Some(rate),
            ..LoadOptions::default()
        };
        self.load_with(key, path, from_secs, producer, options);
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
    dsd_opener: &DsdOpener,
    bus_rate: u32,
    ready_frames: usize,
    failures: &Sender<WorkerFailure>,
) {
    // Above normal, never real time (main spec §2.2): the ring buffers must
    // not run dry because the interface or the analysis pool took the
    // processor. A system that refuses leaves the thread at normal priority.
    fp_decode::priority::set_current(Priority::AboveNormal, "decoder");
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
                    let rate = options.rate.unwrap_or(bus_rate);
                    let limit_frames = options
                        .until_secs
                        .filter(|u| u.is_finite())
                        .map(|u| ((u - from_secs).max(0.0) * f64::from(rate)).round() as u64);
                    jobs.push(Job {
                        key,
                        path,
                        from_secs,
                        producer,
                        source: None,
                        pending: Vec::new(),
                        pending_dsd: Vec::new(),
                        dsd: options.dsd,
                        done: false,
                        limit_frames,
                        looped: options.looped,
                        fade_frames: if options.looped {
                            0
                        } else {
                            options.fade_out_frames
                        },
                        pass_frames: 0,
                        rate,
                        ready_frames: options.ready_frames,
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
            // The threshold is a time: scale it to the rate the source plays at.
            let ready = job
                .ready_frames
                .unwrap_or_else(|| scaled_frames(ready_frames, job.rate, bus_rate));
            let outcome = catch_unwind(AssertUnwindSafe(|| step(job, opener, dsd_opener, ready)));
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

/// Fades the frames of `job.pending` that lie in the last `fade_frames`
/// before `limit` linearly to zero (the pass ends at `limit`).
fn fade_to_limit(job: &mut Job, limit: u64) {
    let fade = job.fade_frames;
    let first = job.pass_frames;
    if fade == 0
        || first.saturating_add((job.pending.len() / 2) as u64) <= limit.saturating_sub(fade)
    {
        return;
    }
    for (i, pair) in job.pending.as_chunks_mut::<2>().0.iter_mut().enumerate() {
        let left = limit.saturating_sub(first + i as u64);
        if left < fade {
            let gain = left as f32 / fade as f32;
            for sample in pair {
                *sample *= gain;
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

/// `frames` at `from_rate`, as the same duration at `to_rate`.
fn scaled_frames(frames: usize, to_rate: u32, from_rate: u32) -> usize {
    let scaled = frames as u64 * u64::from(to_rate) / u64::from(from_rate.max(1));
    usize::try_from(scaled).unwrap_or(usize::MAX)
}

/// Opens the job if needed and moves at most one decoded block into its ring.
fn step(
    job: &mut Job,
    opener: &SourceOpener,
    dsd_opener: &DsdOpener,
    ready_frames: usize,
) -> Result<(), String> {
    if job.limit_frames == Some(0) {
        return finish(job);
    }
    if job.source.is_none() {
        job.source = Some(if job.dsd {
            if job.looped {
                return Err("a DSD source cannot loop".to_owned());
            }
            JobSource::Dsd(dsd_opener(&job.path, job.from_secs, job.rate)?)
        } else {
            JobSource::Pcm(opener(&job.path, job.from_secs, job.rate)?)
        });
    }
    if job.pending.is_empty() {
        let at_limit = job.limit_frames.is_some_and(|l| job.pass_frames >= l);
        let more = !at_limit
            && match job.source.as_mut() {
                Some(JobSource::Pcm(source)) => source.next_block(&mut job.pending)?,
                Some(JobSource::Dsd(source)) => {
                    source.next_pair(&mut job.pending, &mut job.pending_dsd)?
                }
                None => false,
            };
        if let Some(limit) = job.limit_frames {
            // Cut what goes past `until`.
            let room = limit.saturating_sub(job.pass_frames);
            let frames = (job.pending.len() / 2) as u64;
            if frames > room {
                let keep = usize::try_from(room).unwrap_or(usize::MAX) * 2;
                job.pending.truncate(keep);
                job.pending_dsd.truncate(keep);
            }
            // Only PCM is faded: a gain on DSD words would corrupt them.
            fade_to_limit(job, limit);
        }
        job.pass_frames += (job.pending.len() / 2) as u64;
        if job.pending.is_empty() && (at_limit || !more) {
            // End of a pass: loop (unless the pass was empty), or finish.
            if job.looped && job.pass_frames > 0 {
                job.producer
                    .shared
                    .loop_frames
                    .store(job.pass_frames, Ordering::Release);
                job.source = Some(JobSource::Pcm(opener(&job.path, job.from_secs, job.rate)?));
                job.pass_frames = 0;
                return Ok(());
            }
            return finish(job);
        }
    }
    let pushed = if job.dsd {
        let frames = job.producer.push_pair(&job.pending, &job.pending_dsd);
        job.pending_dsd.drain(..frames * 2);
        frames * 2
    } else {
        job.producer.push(&job.pending)
    };
    job.pending.drain(..pushed);
    if job.producer.buffered_frames() >= ready_frames || job.producer.free_frames() == 0 {
        job.producer.shared.ready.store(true, Ordering::Release);
    }
    Ok(())
}
