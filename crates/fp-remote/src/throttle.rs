//! Log lines about refused input, at most one per source per second, so a
//! flood cannot flood the log.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

const LOG_EVERY: Duration = Duration::from_secs(1);
/// Sources remembered before old ones are forgotten.
const LOG_SOURCES: usize = 256;

#[derive(Default)]
pub struct Throttle {
    last: Mutex<HashMap<Option<IpAddr>, Instant>>,
}

impl Throttle {
    pub fn note(&self, source: Option<IpAddr>, what: &str) {
        let now = Instant::now();
        let mut last = self.last.lock().unwrap_or_else(PoisonError::into_inner);
        if last.len() >= LOG_SOURCES {
            last.retain(|_, at| now.duration_since(*at) < LOG_EVERY);
        }
        let due = last
            .get(&source)
            .is_none_or(|at| now.duration_since(*at) >= LOG_EVERY);
        if due {
            last.insert(source, now);
            tracing::warn!(?source, what, "remote input refused");
        }
    }
}
