//! Waits for a path to be quiet before it is looked at: a file still being
//! copied raises a burst of events.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Paths with the time of their last event.
#[derive(Debug, Default)]
pub struct Debouncer {
    last: HashMap<PathBuf, Instant>,
}

impl Debouncer {
    /// Records an event on `path` at `now`.
    pub fn note(&mut self, path: PathBuf, now: Instant) {
        self.last.insert(path, now);
    }

    /// The paths with no event for `quiet`, sorted, removed from the set.
    pub fn take_due(&mut self, now: Instant, quiet: Duration) -> Vec<PathBuf> {
        let mut due: Vec<PathBuf> = self
            .last
            .iter()
            .filter(|(_, at)| now.saturating_duration_since(**at) >= quiet)
            .map(|(path, _)| path.clone())
            .collect();
        for path in &due {
            self.last.remove(path);
        }
        due.sort();
        due
    }

    /// Forgets the paths at or under `folder` (it is no longer watched).
    pub fn purge_under(&mut self, folder: &std::path::Path) {
        self.last.retain(|path, _| !path.starts_with(folder));
    }

    pub fn is_empty(&self) -> bool {
        self.last.is_empty()
    }
}
