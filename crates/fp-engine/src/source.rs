//! A `Source` is one decoded stream (spec §4.1). The worker thread owns the
//! producer half and fills a lock-free ring of interleaved stereo `f32` at the
//! bus rate; a bus mixer owns the consumer half. Everything else they share is
//! atomic.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use crate::atomic::AtomicF32;

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
    pub shared: Arc<SourceShared>,
}

pub struct SourceConsumer {
    ring: rtrb::Consumer<f32>,
    pub shared: Arc<SourceShared>,
}

/// Creates a source whose ring holds `capacity_frames` stereo frames.
pub fn source_pair(capacity_frames: usize) -> (SourceProducer, SourceConsumer) {
    let (producer, consumer) = rtrb::RingBuffer::new(capacity_frames.max(1) * SOURCE_CHANNELS);
    let shared = Arc::new(SourceShared::default());
    (
        SourceProducer {
            ring: producer,
            shared: shared.clone(),
        },
        SourceConsumer {
            ring: consumer,
            shared,
        },
    )
}

impl SourceProducer {
    /// Frames that fit in the ring right now.
    pub fn free_frames(&self) -> usize {
        self.ring.slots() / SOURCE_CHANNELS
    }

    /// Frames waiting to be played.
    pub fn buffered_frames(&self) -> usize {
        (self.ring.buffer().capacity() - self.ring.slots()) / SOURCE_CHANNELS
    }

    /// Pushes as many whole frames of `samples` as fit; returns the number of
    /// samples consumed from `samples`.
    pub fn push(&mut self, samples: &[f32]) -> usize {
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

    /// True when the mixer side has been dropped.
    pub fn is_abandoned(&self) -> bool {
        self.ring.is_abandoned()
    }
}

impl SourceConsumer {
    /// Pops up to `dst.len() / 2` frames into `dst`; returns frames popped.
    /// Never allocates or blocks (real-time safe).
    pub fn pop_frames(&mut self, dst: &mut [f32]) -> usize {
        let whole = dst.len() - dst.len() % SOURCE_CHANNELS;
        let available = self.ring.slots() - self.ring.slots() % SOURCE_CHANNELS;
        let n = whole.min(available);
        match dst.get_mut(..n) {
            Some(target) => self.ring.pop_partial_slice(target).0.len() / SOURCE_CHANNELS,
            None => 0,
        }
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
    fn abandoned_when_the_consumer_is_dropped() {
        let (p, c) = source_pair(4);
        drop(c);
        assert!(p.is_abandoned());
    }
}
