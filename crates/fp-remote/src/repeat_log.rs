//! Decides when a failure that repeats (a busy port, retried every
//! `BIND_RETRY`) is worth a log line: the first time, when the error
//! changes, and every `every` while it lasts.

use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct RepeatLog {
    every: Duration,
    last: Option<(String, Instant)>,
}

impl RepeatLog {
    pub fn new(every: Duration) -> Self {
        Self { every, last: None }
    }

    /// True when `error`, seen at `now`, should be logged.
    pub fn should_log(&mut self, error: &str, now: Instant) -> bool {
        let due = match &self.last {
            Some((e, at)) => e != error || now.saturating_duration_since(*at) >= self.every,
            None => true,
        };
        if due {
            self.last = Some((error.to_owned(), now));
        }
        due
    }

    /// The failure is over: the next one is logged at once.
    pub fn clear(&mut self) {
        self.last = None;
    }
}
