//! Turns the counters of the real-time side into rate-limited log lines
//! (audit A8). The real-time code only increments atomics; the conductor
//! reads them once per tick and calls [`Watch::observe`] here, off the
//! device thread.

use std::time::{Duration, Instant};

/// The shortest time between two log lines about the same counter. A burst of
/// xruns (a device that stutters for a minute) is one line per window with
/// the number that came in, not one line per event. A constant, not a
/// `Config` field: no operator needs a different value, and the lines carry
/// the counts either way.
pub const REPORT_WINDOW: Duration = Duration::from_secs(10);

/// What came in since the last line about a counter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Increase {
    /// Events since the previous report.
    pub new: u64,
    /// The counter's value now.
    pub total: u64,
}

/// One counter watched between ticks.
#[derive(Debug, Clone, Default)]
pub struct Watch {
    seen: u64,
    /// Increases not reported yet because the window was still open.
    pending: u64,
    /// The window opened by the last report closes at this instant.
    open_until: Option<Instant>,
}

impl Watch {
    /// Notes the counter's `value` at `now`. Returns the increase to log when
    /// there is one and the window since the last line has passed. A value
    /// lower than the last one is a counter that restarted (a new source):
    /// all of it is new.
    pub fn observe(&mut self, value: u64, now: Instant) -> Option<Increase> {
        let new = if value >= self.seen {
            value - self.seen
        } else {
            value
        };
        self.seen = value;
        self.pending += new;
        if self.pending == 0 || self.open_until.is_some_and(|until| now < until) {
            return None;
        }
        let new = std::mem::take(&mut self.pending);
        self.open_until = Some(now + REPORT_WINDOW);
        Some(Increase { new, total: value })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_burst_is_one_line_per_window() {
        let t0 = Instant::now();
        let mut w = Watch::default();
        assert_eq!(w.observe(0, t0), None);
        assert_eq!(w.observe(2, t0), Some(Increase { new: 2, total: 2 }));
        assert_eq!(w.observe(5, t0 + Duration::from_secs(1)), None);
        assert_eq!(
            w.observe(5, t0 + REPORT_WINDOW),
            Some(Increase { new: 3, total: 5 })
        );
        assert_eq!(w.observe(5, t0 + REPORT_WINDOW * 3), None);
    }

    #[test]
    fn a_restarted_counter_counts_all_of_its_value_as_new() {
        let t0 = Instant::now();
        let mut w = Watch::default();
        assert!(w.observe(4, t0).is_some());
        let later = t0 + REPORT_WINDOW;
        assert_eq!(w.observe(1, later), Some(Increase { new: 1, total: 1 }));
    }
}
