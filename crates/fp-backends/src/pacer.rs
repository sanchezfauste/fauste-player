//! Real-time pacing for a render thread that has no device clock: the Null
//! output and the engine's virtual clock.
//!
//! The schedule is absolute: block `n` is due at `start + n * period`. A
//! thread that wakes up late (coarse timers, a busy machine) renders the
//! blocks it owes, so the audio clock keeps real time on average instead of
//! losing every wake-up's lateness. Making up is bounded per wake-up, and a
//! gap too long to make up (a suspended machine, a debugger stop) restarts the
//! schedule instead of rendering a burst.

use std::time::{Duration, Instant};

/// An absolute block schedule for a thread that sleeps between blocks.
#[derive(Debug, Clone, Copy)]
pub struct Pacer {
    period: Duration,
    deadline: Instant,
    max_blocks: u32,
    resync_after: Duration,
}

impl Pacer {
    /// The most audio one wake-up renders to make up for lost time. Four
    /// times a coarse 23 ms wake-up, so steady lateness is always made up,
    /// yet short enough that the thread checks its stop flag often.
    pub const MAX_CATCH_UP: Duration = Duration::from_millis(100);
    /// Further behind than this (or than one block, if longer), the schedule
    /// restarts from now: the time is lost rather than rendered in a burst.
    pub const RESYNC_AFTER: Duration = Duration::from_secs(1);

    /// A schedule of blocks of `period`, the first one due at `start`.
    pub fn new(period: Duration, start: Instant) -> Self {
        let per_block = period.as_nanos().max(1);
        let max_blocks = u32::try_from(Self::MAX_CATCH_UP.as_nanos() / per_block)
            .unwrap_or(u32::MAX)
            .max(1);
        Self {
            period,
            deadline: start,
            max_blocks,
            resync_after: Self::RESYNC_AFTER.max(period),
        }
    }

    /// When the next block is due.
    pub fn deadline(&self) -> Instant {
        self.deadline
    }

    /// The most blocks one wake-up renders.
    pub fn max_blocks_per_wake_up(&self) -> u32 {
        self.max_blocks
    }

    /// How many blocks to render at `now`: none before the deadline, the
    /// blocks owed (at most [`Self::max_blocks_per_wake_up`]) after it, one
    /// after a gap beyond [`Self::RESYNC_AFTER`]. Advances the schedule by
    /// the blocks returned. Never allocates.
    pub fn blocks_due(&mut self, now: Instant) -> u32 {
        let Some(late) = now.checked_duration_since(self.deadline) else {
            return 0;
        };
        if late > self.resync_after {
            self.deadline = now + self.period;
            return 1;
        }
        let owed = late.as_nanos() / self.period.as_nanos().max(1) + 1;
        let blocks = u32::try_from(owed).unwrap_or(u32::MAX).min(self.max_blocks);
        self.deadline += self.period * blocks;
        blocks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 256 frames at 48 kHz: about 5.3 ms.
    fn period() -> Duration {
        Duration::from_secs_f64(256.0 / 48_000.0)
    }

    #[test]
    fn on_time_renders_one_block() {
        let start = Instant::now();
        let mut pacer = Pacer::new(period(), start);
        assert_eq!(pacer.blocks_due(start), 1);
        assert_eq!(pacer.deadline(), start + period());
    }

    #[test]
    fn early_renders_nothing() {
        let start = Instant::now();
        let mut pacer = Pacer::new(period(), start);
        pacer.blocks_due(start);
        assert_eq!(pacer.blocks_due(start + period() / 2), 0);
        assert_eq!(pacer.deadline(), start + period());
    }

    #[test]
    fn late_wake_up_renders_the_blocks_owed() {
        let start = Instant::now();
        let mut pacer = Pacer::new(period(), start);
        pacer.blocks_due(start);
        // A wake-up 23 ms after the second block was due owes it plus the
        // four whole periods that passed since.
        let now = start + period() + Duration::from_millis(23);
        assert_eq!(pacer.blocks_due(now), 5);
        assert_eq!(pacer.deadline(), start + period() * 6);
        assert!(pacer.deadline() > now);
    }

    #[test]
    fn coarse_wake_ups_keep_real_time() {
        let start = Instant::now();
        let mut pacer = Pacer::new(period(), start);
        let mut blocks = 0u64;
        let mut now = start;
        while now < start + Duration::from_secs(10) {
            blocks += u64::from(pacer.blocks_due(now));
            now += Duration::from_millis(23);
        }
        let rendered = period().as_secs_f64() * blocks as f64;
        let elapsed = (now - start).as_secs_f64();
        assert!(
            (elapsed - rendered).abs() <= 0.023 + period().as_secs_f64(),
            "rendered {rendered} s in {elapsed} s"
        );
    }

    #[test]
    fn catching_up_is_bounded_per_wake_up() {
        let start = Instant::now();
        let mut pacer = Pacer::new(period(), start);
        pacer.blocks_due(start);
        let now = start + period() + Duration::from_millis(600);
        let bound = pacer.max_blocks_per_wake_up();
        assert_eq!(
            u128::from(bound),
            Pacer::MAX_CATCH_UP.as_nanos() / period().as_nanos()
        );
        assert_eq!(pacer.blocks_due(now), bound);
        // The rest is still owed: the next wake-up goes on catching up.
        assert_eq!(pacer.blocks_due(now), bound);
    }

    #[test]
    fn a_period_longer_than_the_bound_still_renders_its_block() {
        let long = Duration::from_millis(500);
        let start = Instant::now();
        let mut pacer = Pacer::new(long, start);
        assert_eq!(pacer.max_blocks_per_wake_up(), 1);
        assert_eq!(pacer.blocks_due(start), 1);
    }

    #[test]
    fn far_behind_resyncs_without_a_burst() {
        let start = Instant::now();
        let mut pacer = Pacer::new(period(), start);
        pacer.blocks_due(start);
        let now = start + period() + Pacer::RESYNC_AFTER + Duration::from_millis(1);
        assert_eq!(pacer.blocks_due(now), 1);
        assert_eq!(pacer.deadline(), now + period());
    }

    #[test]
    fn a_long_period_is_not_resynced_within_its_own_length() {
        // 16384 frames at 8 kHz: a block longer than the resync threshold.
        let long = Duration::from_secs_f64(16_384.0 / 8_000.0);
        let start = Instant::now();
        let mut pacer = Pacer::new(long, start);
        pacer.blocks_due(start);
        let now = start + long + Duration::from_millis(1_500);
        assert_eq!(pacer.blocks_due(now), 1);
        assert_eq!(pacer.deadline(), start + long * 2);
    }
}
