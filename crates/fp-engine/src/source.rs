//! A `Source` is one decoded stream (spec §4.1). The worker thread owns the
//! producer half and fills a lock-free ring of interleaved stereo `f32` at the
//! bus rate; a bus mixer owns the consumer half. Everything else they share is
//! atomic.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use crate::atomic::{AtomicF32, AtomicF64};

/// Sources are always stereo; mono is duplicated and multichannel downmixed.
pub const SOURCE_CHANNELS: usize = 2;

/// State shared between the producer, the mixer and the conductor.
#[derive(Debug, Default)]
pub struct SourceShared {
    /// Bus frames the mixer has consumed from the ring.
    pub frames_played: AtomicU64,
    /// Frames the worker has pushed into the ring.
    pub frames_pushed: AtomicU64,
    /// Set by the worker once the whole stream has been pushed.
    pub eof: AtomicBool,
    /// Set by the worker when the stream could not be opened or decoded.
    pub failed: AtomicBool,
    /// Set by the worker once enough audio is buffered to start (or at eof).
    pub ready: AtomicBool,
    /// Times the mixer found the ring empty before eof.
    pub underruns: AtomicU64,
    /// Peak levels since the UI last read them (after gain).
    pub peak_l: AtomicF32,
    pub peak_r: AtomicF32,
    /// Length in frames of one pass of a looped source, once the worker has
    /// seen it end (0 until then).
    pub loop_frames: AtomicU64,
    /// Set by the mixer after each block: every gain applied was exactly
    /// 1.0 and no other source wrote into its channel pair, so its samples
    /// reached the output unchanged (Phase 4 spec B1).
    pub unaltered: AtomicBool,
    /// Meter measurement since the conductor last took it (meters spec M1),
    /// after gain: sums of squares, K-weighted sums of squares, and the
    /// frames they cover.
    pub sum_sq_l: AtomicF64,
    pub sum_sq_r: AtomicF64,
    /// Sums of magnitudes (rectified), for the VU.
    pub sum_abs_l: AtomicF64,
    pub sum_abs_r: AtomicF64,
    pub k_sum_l: AtomicF64,
    pub k_sum_r: AtomicF64,
    pub measured_frames: AtomicU64,
}

impl SourceShared {
    pub fn frames_played(&self) -> u64 {
        self.frames_played.load(Ordering::Acquire)
    }

    /// True when everything pushed so far has been played.
    pub fn is_drained(&self) -> bool {
        self.frames_pushed.load(Ordering::Acquire) <= self.frames_played()
    }

    pub fn is_ready(&self) -> bool {
        self.ready.load(Ordering::Acquire)
    }

    pub fn is_eof(&self) -> bool {
        self.eof.load(Ordering::Acquire)
    }

    pub fn is_failed(&self) -> bool {
        self.failed.load(Ordering::Acquire)
    }
}

pub struct SourceProducer {
    ring: rtrb::Producer<f32>,
    /// The ring of DSD words (one f32 per word), beside the PCM ring, for a
    /// DSD source; `None` for a plain one.
    dsd: Option<rtrb::Producer<f32>>,
    pub shared: Arc<SourceShared>,
}

pub struct SourceConsumer {
    ring: rtrb::Consumer<f32>,
    dsd: Option<rtrb::Consumer<f32>>,
    pub shared: Arc<SourceShared>,
}

/// Creates a source whose ring holds `capacity_frames` stereo frames.
pub fn source_pair(capacity_frames: usize) -> (SourceProducer, SourceConsumer) {
    let (producer, consumer) = rtrb::RingBuffer::new(capacity_frames.max(1) * SOURCE_CHANNELS);
    let shared = Arc::new(SourceShared::default());
    (
        SourceProducer {
            ring: producer,
            dsd: None,
            shared: shared.clone(),
        },
        SourceConsumer {
            ring: consumer,
            dsd: None,
            shared,
        },
    )
}

/// Creates a DSD source: a PCM ring and a ring of DSD words, each holding
/// `capacity_frames` stereo frames, filled and drained in lockstep.
pub fn source_pair_dsd(capacity_frames: usize) -> (SourceProducer, SourceConsumer) {
    let (mut producer, mut consumer) = source_pair(capacity_frames);
    let (words_in, words_out) = rtrb::RingBuffer::new(capacity_frames.max(1) * SOURCE_CHANNELS);
    producer.dsd = Some(words_in);
    consumer.dsd = Some(words_out);
    (producer, consumer)
}

impl SourceProducer {
    /// Frames that fit in the ring right now.
    pub fn free_frames(&self) -> usize {
        let pcm = self.ring.slots() / SOURCE_CHANNELS;
        match &self.dsd {
            Some(words) => pcm.min(words.slots() / SOURCE_CHANNELS),
            None => pcm,
        }
    }

    /// True for a DSD source (a word ring beside the PCM ring).
    pub fn is_dsd(&self) -> bool {
        self.dsd.is_some()
    }

    /// Frames waiting to be played.
    pub fn buffered_frames(&self) -> usize {
        (self.ring.buffer().capacity() - self.ring.slots()) / SOURCE_CHANNELS
    }

    /// Pushes as many whole frames of `samples` as fit; returns the number of
    /// samples consumed from `samples`.
    /// A DSD producer pushes nothing here: use `push_pair`.
    pub fn push(&mut self, samples: &[f32]) -> usize {
        if self.dsd.is_some() {
            return 0;
        }
        let whole = samples.len() - samples.len() % SOURCE_CHANNELS;
        let fit = whole.min(self.free_frames() * SOURCE_CHANNELS);
        let (pushed, _) = self
            .ring
            .push_partial_slice(samples.get(..fit).unwrap_or_default());
        self.shared
            .frames_pushed
            .fetch_add((pushed.len() / SOURCE_CHANNELS) as u64, Ordering::AcqRel);
        pushed.len()
    }

    /// Pushes whole frames of `pcm` and `dsd` (stereo each) into the two
    /// rings in lockstep, as many as both hold and both slices have; returns
    /// the frames pushed. A plain source takes nothing.
    pub fn push_pair(&mut self, pcm: &[f32], dsd: &[f32]) -> usize {
        let free = self.free_frames();
        let Some(words) = self.dsd.as_mut() else {
            return 0;
        };
        let frames = free
            .min(pcm.len() / SOURCE_CHANNELS)
            .min(dsd.len() / SOURCE_CHANNELS);
        let n = frames * SOURCE_CHANNELS;
        let (Some(pcm), Some(dsd)) = (pcm.get(..n), dsd.get(..n)) else {
            return 0;
        };
        let (a, _) = self.ring.push_partial_slice(pcm);
        let (b, _) = words.push_partial_slice(dsd);
        let pushed = a.len().min(b.len()) / SOURCE_CHANNELS;
        self.shared
            .frames_pushed
            .fetch_add(pushed as u64, Ordering::AcqRel);
        pushed
    }

    /// True when the mixer side has been dropped.
    pub fn is_abandoned(&self) -> bool {
        self.ring.is_abandoned()
    }
}

impl SourceConsumer {
    /// True for a DSD source.
    pub fn is_dsd(&self) -> bool {
        self.dsd.is_some()
    }

    /// Frames the mixer can take now: for a DSD source those both rings
    /// hold, which is what `pop_pair` delivers.
    pub fn buffered_frames(&self) -> usize {
        let pcm = self.ring.slots() / SOURCE_CHANNELS;
        match &self.dsd {
            Some(words) => pcm.min(words.slots() / SOURCE_CHANNELS),
            None => pcm,
        }
    }

    /// Pops up to `dst.len() / 2` frames into `dst`; returns frames popped.
    /// Never allocates or blocks (real-time safe). On a DSD source it pops
    /// the PCM ring only and discards the same number of words, which keeps
    /// the two rings in lockstep.
    pub fn pop_frames(&mut self, dst: &mut [f32]) -> usize {
        let whole = dst.len() - dst.len() % SOURCE_CHANNELS;
        let n = whole.min(self.buffered_frames() * SOURCE_CHANNELS);
        let popped = match dst.get_mut(..n) {
            Some(target) => self.ring.pop_partial_slice(target).0.len(),
            None => 0,
        };
        if let Some(words) = self.dsd.as_mut() {
            // Discard exactly `popped` words, a slot at a time.
            for _ in 0..popped {
                if words.pop().is_err() {
                    break;
                }
            }
        }
        popped / SOURCE_CHANNELS
    }

    /// Pops up to `pcm.len() / 2` frames from both rings in lockstep into
    /// `pcm` and `dsd` (same length); returns frames popped. A plain source
    /// gives nothing. Never allocates or blocks (real-time safe).
    pub fn pop_pair(&mut self, pcm: &mut [f32], dsd: &mut [f32]) -> usize {
        let Some(words) = self.dsd.as_mut() else {
            return 0;
        };
        let frames = (pcm.len() / SOURCE_CHANNELS)
            .min(dsd.len() / SOURCE_CHANNELS)
            .min(self.ring.slots() / SOURCE_CHANNELS)
            .min(words.slots() / SOURCE_CHANNELS);
        let n = frames * SOURCE_CHANNELS;
        let (Some(pcm), Some(dsd)) = (pcm.get_mut(..n), dsd.get_mut(..n)) else {
            return 0;
        };
        let a = self.ring.pop_partial_slice(pcm).0.len();
        let b = words.pop_partial_slice(dsd).0.len();
        a.min(b) / SOURCE_CHANNELS
    }

    /// True when the producer side has been dropped.
    pub fn is_abandoned(&self) -> bool {
        self.ring.is_abandoned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pushes_and_pops_whole_frames_only() {
        let (mut p, mut c) = source_pair(2);
        assert_eq!(
            p.push(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]),
            4,
            "ring holds 2 frames"
        );
        assert_eq!(p.buffered_frames(), 2);
        let mut dst = [0.0; 3];
        assert_eq!(c.pop_frames(&mut dst), 1);
        assert_eq!(dst, [1.0, 2.0, 0.0]);
        assert_eq!(p.free_frames(), 1);
    }

    #[test]
    fn a_dsd_pair_pushes_and_pops_both_rings_in_lockstep() {
        let (mut p, mut c) = source_pair_dsd(3);
        assert!(p.is_dsd() && c.is_dsd());
        let pcm = [1.0, -1.0, 2.0, -2.0, 3.0, -3.0, 4.0, -4.0];
        let dsd = [10.0, -10.0, 20.0, -20.0, 30.0, -30.0, 40.0, -40.0];
        assert_eq!(p.push_pair(&pcm, &dsd), 3, "the ring holds 3 frames");
        assert_eq!(c.buffered_frames(), 3);
        let (mut a, mut b) = ([0.0; 4], [0.0; 4]);
        assert_eq!(c.pop_pair(&mut a, &mut b), 2);
        assert_eq!(a, [1.0, -1.0, 2.0, -2.0]);
        assert_eq!(b, [10.0, -10.0, 20.0, -20.0]);
        assert_eq!(p.shared.frames_pushed.load(Ordering::Acquire), 3);
    }

    #[test]
    fn a_plain_source_is_not_dsd_and_pop_pair_gives_nothing() {
        let (p, mut c) = source_pair(2);
        assert!(!p.is_dsd() && !c.is_dsd());
        assert_eq!(c.pop_pair(&mut [0.0; 2], &mut [0.0; 2]), 0);
    }

    #[test]
    fn a_dsd_consumer_reports_only_frames_both_rings_hold() {
        let (mut p, mut c) = source_pair_dsd(4);
        p.push_pair(&[1.0; 6], &[2.0; 6]);
        // Pop the PCM ring only, as a stray pop_frames would not: the
        // frames reported are those pop_pair can deliver.
        let mut one = [0.0; 2];
        assert_eq!(c.pop_frames(&mut one), 1);
        assert_eq!(c.buffered_frames(), 2);
        let (mut a, mut b) = ([0.0; 8], [0.0; 8]);
        assert_eq!(c.pop_pair(&mut a, &mut b), 2, "lockstep kept by pop_frames");
        assert_eq!(c.buffered_frames(), 0);
    }

    #[test]
    fn a_dsd_producer_ignores_plain_push() {
        let (mut p, c) = source_pair_dsd(2);
        assert_eq!(p.push(&[1.0, 2.0]), 0);
        assert_eq!(c.buffered_frames(), 0);
    }

    #[test]
    fn abandoned_when_the_consumer_is_dropped() {
        let (p, c) = source_pair(4);
        drop(c);
        assert!(p.is_abandoned());
    }
}
