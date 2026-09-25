//! Streaming sample-rate conversion of interleaved stereo with rubato's
//! windowed-sinc resampler. The filter delay is removed and the output length
//! is exactly `input_frames × ratio`, so timelines stay accurate.

use audioadapter_buffers::direct::InterleavedSlice;
use rubato::{Async, FixedAsync, Indexing, Resampler, SincInterpolationParameters};

const CHUNK_FRAMES: usize = 1024;

pub struct StreamResampler {
    inner: Async<f32>,
    pending: Vec<f32>,
    out: Vec<f32>,
    ratio: f64,
    frames_in: u64,
    frames_out: u64,
    skip: usize,
}

impl StreamResampler {
    pub fn new(from_rate: u32, to_rate: u32) -> Result<Self, String> {
        let ratio = f64::from(to_rate) / f64::from(from_rate.max(1));
        let params = SincInterpolationParameters::default();
        let inner = Async::<f32>::new_sinc(ratio, 1.0, &params, CHUNK_FRAMES, 2, FixedAsync::Input)
            .map_err(|e| e.to_string())?;
        let out = vec![0.0; inner.output_frames_max() * 2];
        let skip = inner.output_delay();
        Ok(Self {
            inner,
            pending: Vec::new(),
            out,
            ratio,
            frames_in: 0,
            frames_out: 0,
            skip,
        })
    }

    /// Feeds interleaved stereo input; appends every complete output chunk to `dst`.
    pub fn push(&mut self, input: &[f32], dst: &mut Vec<f32>) -> Result<(), String> {
        self.pending.extend_from_slice(input);
        self.frames_in += (input.len() / 2) as u64;
        loop {
            let need = self.inner.input_frames_next();
            if self.pending.len() < need * 2 {
                return Ok(());
            }
            self.run(need, None, dst)?;
        }
    }

    /// Flushes the tail at end of stream, trimming the output to its exact length.
    pub fn finish(&mut self, dst: &mut Vec<f32>) -> Result<(), String> {
        let start = dst.len();
        let expected = (self.frames_in as f64 * self.ratio).round() as u64;
        while self.frames_out < expected {
            let need = self.inner.input_frames_next();
            let have = self.pending.len() / 2;
            self.pending.resize(need * 2, 0.0);
            let partial = if have > 0 && have < need {
                Some(have)
            } else {
                None
            };
            let before = self.frames_out;
            self.run(need, partial, dst)?;
            if self.frames_out == before && have == 0 {
                break;
            }
        }
        if self.frames_out > expected {
            let extra = usize::try_from(self.frames_out - expected).unwrap_or(usize::MAX) * 2;
            let keep = dst.len().saturating_sub(extra).max(start);
            dst.truncate(keep);
            self.frames_out = expected;
        }
        Ok(())
    }

    fn run(
        &mut self,
        need: usize,
        partial: Option<usize>,
        dst: &mut Vec<f32>,
    ) -> Result<(), String> {
        let capacity = self.out.len() / 2;
        let input_samples = self
            .pending
            .get(..need * 2)
            .ok_or("resampler input underflow")?;
        let input = InterleavedSlice::new(input_samples, 2, need).map_err(|e| e.to_string())?;
        let mut output =
            InterleavedSlice::new_mut(&mut self.out, 2, capacity).map_err(|e| e.to_string())?;
        let indexing = match partial {
            Some(frames) => Indexing::new().partial_len(frames),
            None => Indexing::new(),
        };
        let (_read, written) = self
            .inner
            .process_into_buffer(&input, &mut output, Some(&indexing))
            .map_err(|e| e.to_string())?;
        let skip = self.skip.min(written);
        self.skip -= skip;
        dst.extend_from_slice(self.out.get(skip * 2..written * 2).unwrap_or_default());
        self.frames_out += (written - skip) as u64;
        self.pending.drain(..need * 2);
        Ok(())
    }
}
