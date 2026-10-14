//! Integrated loudness of a whole programme (ITU-R BS.1770-5, loudness
//! normalization spec N4): the K-weighted mean square of both channels,
//! weighted 1.0 each, over 400 ms blocks with 75 % overlap, gated at
//! −70 LUFS (absolute) and 10 LU below the mean of the blocks above that
//! (relative); and the true peak of both channels.
//!
//! The energy of every 100 ms hop is kept, so memory grows with the length
//! of the file (about 36 000 numbers an hour). This runs in the analysis
//! pool, never on the device callback.

use crate::kweight::KWeighting;
use crate::truepeak::TruePeak;

/// The true peak reported for complete digital silence, whose true value
/// (−∞) is not a number a measurement can store. Far below any audible
/// level, so no gain rule ever depends on it.
pub const SILENCE_DBTP: f64 = -200.0;

/// The BS.1770 gating thresholds (LUFS and LU), fixed by the
/// Recommendation.
const ABSOLUTE_GATE_LUFS: f64 = -70.0;
const RELATIVE_GATE_LU: f64 = -10.0;
/// −0.691: the offset that makes a 997 Hz sine read its level.
const LOUDNESS_OFFSET: f64 = -0.691;
/// The largest true peak a real programme can have, dBTP. Decoded float
/// PCM may go over full scale, but never by 60 dB (1000 times): above it
/// the file is corrupt, and a measurement would lead to a cut that
/// silences the track. A validity bound, not a setting.
const MAX_PLAUSIBLE_DBTP: f64 = 60.0;
/// Hops per gating block: 4 × 100 ms = 400 ms, 75 % overlap.
const HOPS_PER_BLOCK: usize = 4;

/// The measurement of one programme.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Loudness {
    /// `None` when no block passes the absolute gate (silence or near it).
    pub integrated_lufs: Option<f64>,
    /// The largest true peak of both channels, dBTP; `SILENCE_DBTP` for
    /// digital silence.
    pub true_peak_dbtp: f64,
}

/// Measures one programme: feed it with `push`, read it with `finish`.
#[derive(Debug, Clone)]
pub struct LoudnessMeter {
    k: [KWeighting; 2],
    tp: [TruePeak; 2],
    /// Frames per 100 ms hop (at least 1).
    hop_frames: u64,
    /// Frames in the hop being filled.
    in_hop: u64,
    /// Sum of the K-weighted squares of both channels in that hop.
    hop_energy: f64,
    /// The energy of every complete hop, in order.
    hops: Vec<f64>,
    /// Largest true-peak magnitude so far (linear).
    peak: f32,
    /// The left sample of a frame split between two pushes.
    half: Option<f32>,
}

/// Frames in one gating block at `sample_rate` (0 is taken as 1 Hz): four
/// hops. At rates of 10 Hz or more it is never longer than 400 ms, so a
/// programme of 400 ms or more always holds one block (spec N5).
pub fn gating_block_frames(sample_rate: u32) -> u64 {
    hop_frames(sample_rate) * HOPS_PER_BLOCK as u64
}

/// Frames per 100 ms hop: a tenth of the rate, rounded down to whole frames
/// (at least one). Rounding down keeps a block within 400 ms where a tenth
/// of the rate is not a whole number (11 025 Hz: 1 102 frames, 99.95 ms).
fn hop_frames(sample_rate: u32) -> u64 {
    (u64::from(sample_rate.max(1)) / 10).max(1)
}

fn lufs_of(mean_square: f64) -> f64 {
    LOUDNESS_OFFSET + 10.0 * mean_square.log10()
}

/// The inverse of `lufs_of`.
fn mean_square_of(lufs: f64) -> f64 {
    10f64.powf((lufs - LOUDNESS_OFFSET) / 10.0)
}

impl LoudnessMeter {
    /// A meter for interleaved stereo at `sample_rate` (0 is taken as 1 Hz).
    pub fn new(sample_rate: u32) -> Self {
        let rate = sample_rate.max(1);
        Self {
            k: [KWeighting::new(rate), KWeighting::new(rate)],
            tp: [TruePeak::default(), TruePeak::default()],
            hop_frames: hop_frames(rate),
            in_hop: 0,
            hop_energy: 0.0,
            hops: Vec::new(),
            peak: 0.0,
            half: None,
        }
    }

    /// Feeds interleaved stereo samples. A push may end mid-frame: the half
    /// frame is completed by the next push, and one never completed is not
    /// measured. A sample that is not finite counts as silence.
    pub fn push(&mut self, stereo: &[f32]) {
        let mut rest = stereo;
        if let Some(left) = self.half
            && let Some((right, tail)) = rest.split_first()
        {
            self.half = None;
            self.frame([left, *right]);
            rest = tail;
        }
        let (frames, tail) = rest.as_chunks::<2>();
        for frame in frames {
            self.frame(*frame);
        }
        if let Some(left) = tail.first() {
            self.half = Some(*left);
        }
    }

    fn frame(&mut self, frame: [f32; 2]) {
        let mut energy = 0.0;
        for ((x, k), tp) in frame.iter().zip(&mut self.k).zip(&mut self.tp) {
            let x = if x.is_finite() { *x } else { 0.0 };
            self.peak = self.peak.max(tp.push(x));
            let w = k.process(f64::from(x));
            energy += w * w;
        }
        self.hop_energy += energy;
        self.in_hop += 1;
        if self.in_hop == self.hop_frames {
            self.hops.push(self.hop_energy);
            self.hop_energy = 0.0;
            self.in_hop = 0;
        }
    }

    /// The result so far; `None` with fewer than 400 ms pushed (one gating
    /// block, spec N5), when a value is not finite, or when the true peak is
    /// beyond any real programme (`MAX_PLAUSIBLE_DBTP`, a corrupt file).
    pub fn finish(&self) -> Option<Loudness> {
        if self.hops.len() < HOPS_PER_BLOCK {
            return None;
        }
        let block_frames = (self.hop_frames * HOPS_PER_BLOCK as u64) as f64;
        // The mean square of the blocks louder than `gate` (LUFS), if any.
        // A block's mean square is summed over the two channels (weight
        // 1.0); the gate is turned into a mean square once, so no block
        // needs a logarithm.
        let mean_above = |gate: f64| {
            let threshold = mean_square_of(gate);
            let (sum, n) = self
                .hops
                .windows(HOPS_PER_BLOCK)
                .map(|w| w.iter().sum::<f64>() / block_frames)
                .filter(|z| *z > threshold)
                .fold((0.0, 0usize), |(s, n), z| (s + z, n + 1));
            (n > 0).then(|| sum / n as f64)
        };
        let integrated_lufs = mean_above(ABSOLUTE_GATE_LUFS).and_then(|ungated| {
            let gate = (lufs_of(ungated) + RELATIVE_GATE_LU).max(ABSOLUTE_GATE_LUFS);
            mean_above(gate).map(lufs_of)
        });
        let true_peak_dbtp = if self.peak > 0.0 {
            20.0 * f64::from(self.peak).log10()
        } else {
            SILENCE_DBTP
        };
        let valid = integrated_lufs.is_none_or(f64::is_finite)
            && true_peak_dbtp.is_finite()
            && true_peak_dbtp <= MAX_PLAUSIBLE_DBTP;
        valid.then_some(Loudness {
            integrated_lufs,
            true_peak_dbtp,
        })
    }
}
