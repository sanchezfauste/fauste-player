//! Choosing the music filler's next track (automation spec AU31–AU34):
//! track and artist separation, a fair choice among the least recently
//! aired, and relaxing when nothing qualifies. Only classic, publicly
//! documented methods: least recently played order and simple separation
//! rules.

use std::collections::HashSet;

use crate::ids::TrackId;
use crate::rotation::StepSource;
use crate::state::AppState;
use crate::wall_time::{SplitMix64, WallTime};

/// Which rule had to give way for the pick (AU34).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relaxed {
    None,
    Artist,
    Track,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Picked {
    pub track: TrackId,
    pub relaxed: Relaxed,
}

/// Tracks queued but not yet aired, with their expected start: they count
/// as aired then (the filler's look-ahead).
pub type Planned<'a> = &'a [(TrackId, WallTime)];

/// The fair choice picks uniformly among this many least recently aired
/// eligible tracks (AU33). It tunes the algorithm; it is not an operator
/// limit.
pub const PICK_SPREAD: usize = 5;

/// The playable tracks of `source`, in id order: a category with all its
/// subcategories, or every track of a kind.
pub fn pool(state: &AppState, source: &StepSource) -> Vec<TrackId> {
    let mut out: Vec<TrackId> = match source {
        StepSource::Category(id) => {
            let tree: HashSet<_> = state.categories.subtree(*id).into_iter().collect();
            state
                .library
                .iter()
                .filter(|t| t.category.is_some_and(|c| tree.contains(&c)))
                .filter(|t| t.file_state.is_playable())
                .map(|t| t.id)
                .collect()
        }
        StepSource::Kind(kind) => state
            .library
            .iter()
            .filter(|t| t.kind == *kind && t.file_state.is_playable())
            .map(|t| t.id)
            .collect(),
    };
    out.sort();
    out
}

/// The tracks of `source` that meet both separation rules, least recently
/// aired first.
pub fn eligible(
    state: &AppState,
    source: &StepSource,
    at: WallTime,
    planned: Planned<'_>,
) -> Vec<TrackId> {
    let ctx = Context::new(state, source, at, planned);
    ctx.ordered
        .iter()
        .map(|(_, t)| *t)
        .filter(|t| ctx.track_ok(*t) && ctx.artist_ok(*t))
        .collect()
}

/// The next track from `source` (AU31–AU34), or `None` when it has no
/// playable track.
///
/// One pass over the pool in least-recently-aired order, checking the
/// rules lazily: it stops as soon as `PICK_SPREAD` tracks meet both, so a
/// large library costs a sort and a short scan, not a check of every track.
pub fn pick(
    state: &AppState,
    source: &StepSource,
    at: WallTime,
    planned: Planned<'_>,
    rng: &mut SplitMix64,
) -> Option<Picked> {
    let ctx = Context::new(state, source, at, planned);
    let mut both = Vec::with_capacity(PICK_SPREAD);
    let mut track_only = Vec::with_capacity(PICK_SPREAD);
    for (_, track) in &ctx.ordered {
        if !ctx.track_ok(*track) {
            continue;
        }
        if ctx.artist_ok(*track) {
            both.push(*track);
            if both.len() == PICK_SPREAD {
                break;
            }
        } else if track_only.len() < PICK_SPREAD {
            track_only.push(*track);
        }
    }
    let (tier, relaxed) = if !both.is_empty() {
        (both, Relaxed::None)
    } else if !track_only.is_empty() {
        (track_only, Relaxed::Artist)
    } else {
        let all: Vec<TrackId> = ctx
            .ordered
            .iter()
            .take(PICK_SPREAD)
            .map(|(_, t)| *t)
            .collect();
        (all, Relaxed::Track)
    };
    let index = usize::try_from(rng.below(tier.len() as u64)).unwrap_or(0);
    tier.get(index).map(|track| Picked {
        track: *track,
        relaxed,
    })
}

/// What one choice needs, computed once.
struct Context<'a> {
    state: &'a AppState,
    at: WallTime,
    planned: Planned<'a>,
    /// The pool with each track's last aired time, least recently aired
    /// first (never aired before all), then by id (AU33).
    ordered: Vec<(Option<WallTime>, TrackId)>,
    /// The aired times of the pool's tracks, ascending: how many aired
    /// after a time is a binary search.
    pool_times: Vec<WallTime>,
    /// The normalized artists aired within the artist separation (or at a
    /// time after `at`), each with its latest time. Only these can block a
    /// candidate, so the rest of the library is never normalized.
    recent_artists: Vec<(String, WallTime)>,
}

/// Trimmed and lower-cased (AU32).
fn norm(artist: &str) -> String {
    artist.trim().to_lowercase()
}

/// `artist` normalized equals `key` (already normalized), without
/// allocating.
fn same_artist(artist: &str, key: &str) -> bool {
    artist
        .trim()
        .chars()
        .flat_map(char::to_lowercase)
        .eq(key.chars())
}

impl<'a> Context<'a> {
    fn new(state: &'a AppState, source: &StepSource, at: WallTime, planned: Planned<'a>) -> Self {
        let last_of = |t: TrackId| {
            let planned = planned
                .iter()
                .filter(|(p, _)| *p == t)
                .map(|(_, w)| *w)
                .max();
            state.air_times.get(t).max(planned)
        };
        let mut ordered: Vec<(Option<WallTime>, TrackId)> = pool(state, source)
            .into_iter()
            .map(|t| (last_of(t), t))
            .collect();
        ordered.sort();
        let mut pool_times: Vec<WallTime> = ordered.iter().filter_map(|(w, _)| *w).collect();
        pool_times.sort();
        let mins = state.config.automation.artist_separation_mins;
        let mut recent_artists: Vec<(String, WallTime)> = Vec::new();
        if mins > 0 {
            let window = i64::from(mins) * 60_000;
            let recent = state
                .air_times
                .iter()
                .chain(planned.iter().copied())
                .filter(|(_, when)| at.millis_since(*when) < window);
            for (track, when) in recent {
                let Some(artist) = state.library.get(track).map(|t| norm(&t.artist)) else {
                    continue;
                };
                if artist.is_empty() {
                    continue;
                }
                match recent_artists.iter_mut().find(|(a, _)| *a == artist) {
                    Some((_, latest)) => *latest = (*latest).max(when),
                    None => recent_artists.push((artist, when)),
                }
            }
        }
        Self {
            state,
            at,
            planned,
            ordered,
            pool_times,
            recent_artists,
        }
    }

    fn last(&self, track: TrackId) -> Option<WallTime> {
        let planned = self
            .planned
            .iter()
            .filter(|(p, _)| *p == track)
            .map(|(_, w)| *w)
            .max();
        self.state.air_times.get(track).max(planned)
    }

    /// AU31: eligible again by time or by the share of the pool aired
    /// since; a criterion set to 0 is off, and with both off every track
    /// is eligible.
    fn track_ok(&self, track: TrackId) -> bool {
        let Some(last) = self.last(track) else {
            return true;
        };
        let cfg = &self.state.config.automation;
        let (mins, pct) = (cfg.track_separation_mins, cfg.track_separation_percent);
        if mins == 0 && pct == 0 {
            return true;
        }
        let by_time = mins > 0 && self.at.millis_since(last) >= i64::from(mins) * 60_000;
        let others = self.ordered.len().saturating_sub(1);
        let needed = (usize::from(pct) * others).div_ceil(100);
        // Strictly later times only, so the track itself never counts.
        let since = self.pool_times.len() - self.pool_times.partition_point(|w| *w <= last);
        let by_pct = pct > 0 && since >= needed;
        by_time || by_pct
    }

    /// AU32: no track of the same artist aired within the separation (a
    /// time after `at`, the clock gone back, counts as within).
    fn artist_ok(&self, track: TrackId) -> bool {
        if self.recent_artists.is_empty() {
            return true;
        }
        let Some(artist) = self.state.library.get(track).map(|t| t.artist.as_str()) else {
            return true;
        };
        if artist.trim().is_empty() {
            return true;
        }
        !self
            .recent_artists
            .iter()
            .any(|(key, _)| same_artist(artist, key))
    }
}
