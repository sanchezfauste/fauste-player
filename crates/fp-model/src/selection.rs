//! Choosing the music filler's next track (automation spec AU31–AU34):
//! track and artist separation, a fair choice among the least recently
//! aired, and relaxing when nothing qualifies. Only classic, publicly
//! documented methods: least recently played order and simple separation
//! rules.

use std::collections::{HashMap, HashSet};

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
    let mut out: Vec<TrackId> = ctx
        .pool
        .iter()
        .copied()
        .filter(|t| ctx.track_ok(*t) && ctx.artist_ok(*t))
        .collect();
    ctx.order(&mut out);
    out
}

/// The next track from `source` (AU31–AU34), or `None` when it has no
/// playable track.
pub fn pick(
    state: &AppState,
    source: &StepSource,
    at: WallTime,
    planned: Planned<'_>,
    rng: &mut SplitMix64,
) -> Option<Picked> {
    let ctx = Context::new(state, source, at, planned);
    if ctx.pool.is_empty() {
        return None;
    }
    let both: Vec<TrackId> = ctx
        .pool
        .iter()
        .copied()
        .filter(|t| ctx.track_ok(*t) && ctx.artist_ok(*t))
        .collect();
    let (mut tier, relaxed) = if !both.is_empty() {
        (both, Relaxed::None)
    } else {
        let track_only: Vec<TrackId> = ctx
            .pool
            .iter()
            .copied()
            .filter(|t| ctx.track_ok(*t))
            .collect();
        if track_only.is_empty() {
            (ctx.pool.clone(), Relaxed::Track)
        } else {
            (track_only, Relaxed::Artist)
        }
    };
    ctx.order(&mut tier);
    let spread = tier.len().min(PICK_SPREAD);
    let index = usize::try_from(rng.below(spread as u64)).unwrap_or(0);
    tier.get(index).map(|track| Picked {
        track: *track,
        relaxed,
    })
}

/// What one choice needs, computed once.
struct Context<'a> {
    state: &'a AppState,
    at: WallTime,
    pool: Vec<TrackId>,
    /// Last aired time of every track that has one, with the planned
    /// overlay.
    last: HashMap<TrackId, WallTime>,
    /// Latest aired time per normalized artist, across the library.
    artists: HashMap<String, WallTime>,
    /// The aired times of the pool's tracks, ascending: how many aired
    /// after a time is a binary search, so a choice over a large library
    /// stays O(n log n).
    pool_times: Vec<WallTime>,
}

fn norm(artist: &str) -> String {
    artist.trim().to_lowercase()
}

impl<'a> Context<'a> {
    fn new(state: &'a AppState, source: &StepSource, at: WallTime, planned: Planned<'_>) -> Self {
        let mut last: HashMap<TrackId, WallTime> = state.air_times.iter().collect();
        for (track, when) in planned {
            let slot = last.entry(*track).or_insert(*when);
            if *when > *slot {
                *slot = *when;
            }
        }
        let mut artists: HashMap<String, WallTime> = HashMap::new();
        for (track, when) in &last {
            let Some(artist) = state.library.get(*track).map(|t| norm(&t.artist)) else {
                continue;
            };
            if artist.is_empty() {
                continue;
            }
            let slot = artists.entry(artist).or_insert(*when);
            if *when > *slot {
                *slot = *when;
            }
        }
        let pool = pool(state, source);
        let mut pool_times: Vec<WallTime> =
            pool.iter().filter_map(|t| last.get(t).copied()).collect();
        pool_times.sort();
        Self {
            state,
            at,
            pool,
            last,
            artists,
            pool_times,
        }
    }

    fn last(&self, track: TrackId) -> Option<WallTime> {
        self.last.get(&track).copied()
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
        let others = self.pool.len().saturating_sub(1);
        let needed = (usize::from(pct) * others).div_ceil(100);
        // Strictly later times only, so the track itself never counts.
        let since = self.pool_times.len() - self.pool_times.partition_point(|w| *w <= last);
        let by_pct = pct > 0 && since >= needed;
        by_time || by_pct
    }

    /// AU32: no track of the same artist aired within the separation.
    fn artist_ok(&self, track: TrackId) -> bool {
        let mins = self.state.config.automation.artist_separation_mins;
        if mins == 0 {
            return true;
        }
        let Some(artist) = self.state.library.get(track).map(|t| norm(&t.artist)) else {
            return true;
        };
        if artist.is_empty() {
            return true;
        }
        match self.artists.get(&artist) {
            // A time in the future (the clock went back) is "just aired".
            Some(when) => self.at.millis_since(*when) >= i64::from(mins) * 60_000,
            None => true,
        }
    }

    /// AU33: least recently aired first (never aired before all), then by id.
    fn order(&self, tracks: &mut [TrackId]) {
        tracks.sort_by_key(|t| (self.last(*t), *t));
    }
}
