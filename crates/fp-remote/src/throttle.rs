//! Log lines about refused input, at most one per source per second, so a
//! flood cannot flood the log. The table of sources is bounded: once full,
//! further sources (for example spoofed UDP senders) share one summary line
//! per second instead of growing it.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

const LOG_EVERY: Duration = Duration::from_secs(1);
/// Sources remembered at once.
const LOG_SOURCES: usize = 256;

#[derive(Default)]
struct Inner {
    last: HashMap<Option<IpAddr>, Instant>,
    /// Refusals from sources that did not fit in the table.
    others: u64,
    others_logged: Option<Instant>,
}

#[derive(Default)]
pub struct Throttle {
    inner: Mutex<Inner>,
}

impl Throttle {
    pub fn note(&self, source: Option<IpAddr>, what: &str) {
        let now = Instant::now();
        let mut inner = self.inner.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(at) = inner.last.get_mut(&source) {
            if now.duration_since(*at) >= LOG_EVERY {
                *at = now;
                tracing::warn!(?source, what, "remote input refused");
            }
            return;
        }
        if inner.last.len() >= LOG_SOURCES {
            inner
                .last
                .retain(|_, at| now.duration_since(*at) < LOG_EVERY);
        }
        if inner.last.len() < LOG_SOURCES {
            inner.last.insert(source, now);
            tracing::warn!(?source, what, "remote input refused");
            return;
        }
        inner.others += 1;
        let due = inner
            .others_logged
            .is_none_or(|at| now.duration_since(at) >= LOG_EVERY);
        if due {
            tracing::warn!(
                count = inner.others,
                what,
                "remote input refused from many sources"
            );
            inner.others = 0;
            inner.others_logged = Some(now);
        }
    }

    /// Sources currently remembered.
    pub fn tracked(&self) -> usize {
        self.inner
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .last
            .len()
    }
}
