//! When each track last aired (automation spec AU35): one time per track,
//! so it is bounded by the library.

use std::collections::HashMap;

use crate::ids::TrackId;
use crate::state::AppState;
use crate::wall_time::WallTime;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AirTimes {
    times: HashMap<TrackId, WallTime>,
}

impl AirTimes {
    pub fn get(&self, track: TrackId) -> Option<WallTime> {
        self.times.get(&track).copied()
    }

    /// Records that `track` aired at `at`. The latest airing is the
    /// truth, even when its time is earlier than the stored one (a stamp a
    /// wrong clock left in the future is replaced).
    pub fn record(&mut self, track: TrackId, at: WallTime) {
        self.times.insert(track, at);
    }

    /// Keeps the later of the stored time and `at` (loading, merging).
    pub fn record_later(&mut self, track: TrackId, at: WallTime) {
        let slot = self.times.entry(track).or_insert(at);
        if at > *slot {
            *slot = at;
        }
    }

    /// Brings every time after `now` back to `now` (the clock went back,
    /// or was ahead when the times were stored), so no track or artist is
    /// held back for as long as the clock was wrong. Returns how many
    /// changed.
    pub fn clamp_future(&mut self, now: WallTime) -> usize {
        let mut changed = 0;
        for at in self.times.values_mut() {
            if *at > now {
                *at = now;
                changed += 1;
            }
        }
        changed
    }

    pub fn retain_tracks(&mut self, keep: impl Fn(TrackId) -> bool) {
        self.times.retain(|t, _| keep(*t));
    }

    pub fn remove(&mut self, track: TrackId) {
        self.times.remove(&track);
    }

    pub fn iter(&self) -> impl Iterator<Item = (TrackId, WallTime)> + '_ {
        self.times.iter().map(|(t, w)| (*t, *w))
    }

    pub fn len(&self) -> usize {
        self.times.len()
    }

    pub fn is_empty(&self) -> bool {
        self.times.is_empty()
    }
}

impl FromIterator<(TrackId, WallTime)> for AirTimes {
    fn from_iter<I: IntoIterator<Item = (TrackId, WallTime)>>(iter: I) -> Self {
        let mut out = Self::default();
        for (track, at) in iter {
            out.record_later(track, at);
        }
        out
    }
}

/// AU35: `track` started on an on-air holder (a player's Main or the
/// cartwall's Main) at `state.wall_now`.
pub(crate) fn record_track(state: &mut AppState, track: TrackId) {
    let now = state.wall_now;
    state.air_times.record(track, now);
}
