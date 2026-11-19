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
    /// No track fitted the gap (AU25): the one that cuts least was chosen.
    pub unfitted: bool,
}

/// The time a pick must fit before a block (AU25): `gap_secs` from the
/// row's start to the block's start, which the filler's fade ends at.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fit {
    pub gap_secs: f64,
    pub max_cut_secs: f64,
    pub min_play_secs: f64,
}

/// How well a track fills a gap (AU25), best first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum FitClass {
    /// It plays at least the minimum and is faded with at most the cut.
    Ends,
    /// It ends inside the gap, leaving at least the minimum to a later
    /// track, and some track of the pool could close what is left (or
    /// enough is left for more).
    Room,
    /// It ends inside the gap, leaving at least the minimum.
    LooseRoom,
}

/// A track's audible length and its lead (to where the next row starts:
/// the segue point with `players.auto_segue`, else the end), as the
/// player plans it; `None` while its length is unknown.
pub(crate) fn timing(state: &AppState, track: TrackId) -> Option<(f64, f64)> {
    let t = state.library.get(track)?;
    let range = t.play_range(state.config.players.use_cue_markers);
    let end = range
        .known_end()
        .filter(|e| e.is_finite() && *e > range.cue_in)?;
    let segue = t
        .segue_start_secs()
        .filter(|s| state.config.players.auto_segue && *s >= range.cue_in && *s < end);
    Some((end - range.cue_in, segue.unwrap_or(end) - range.cue_in))
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
    pick_in(&Context::new(state, source, at, planned), rng)
}

fn pick_in(ctx: &Context<'_>, rng: &mut SplitMix64) -> Option<Picked> {
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
        unfitted: false,
    })
}

/// Whether a track of `track`'s timing fills `fit`'s gap without a
/// fragment: it ends the gap within the cut, or leaves at least the
/// minimum after it (AU25). A track of unknown length is not judged.
pub(crate) fn fits(state: &AppState, track: TrackId, fit: &Fit) -> bool {
    let Some((length, lead)) = timing(state, track) else {
        return true;
    };
    let gap = fit.gap_secs;
    if length >= gap {
        // The last before the block: it plays into the fade.
        return length - gap <= fit.max_cut_secs && gap >= fit.min_play_secs;
    }
    gap - lead >= fit.min_play_secs
}

/// `pick`, preferring with a `fit` the tracks that fill the gap before a
/// block without a fragment (AU25), a greedy fit of known lengths: a
/// track that ends the gap within the cut or leaves room for a later
/// track that could close it, with both separation rules, then with the
/// track rule alone; then one that leaves any room, the same way; then
/// each with separation relaxed. When no track fits, the one that
/// cuts least wins (then the one that leaves least, then the order) among
/// the tracks the separation rules allow, relaxed in the same order, and
/// the pick says so.
pub fn pick_fitted(
    state: &AppState,
    source: &StepSource,
    at: WallTime,
    planned: Planned<'_>,
    rng: &mut SplitMix64,
    fit: Option<&Fit>,
) -> Option<Picked> {
    let Some(fit) = fit.filter(|f| f.gap_secs.is_finite() && f.gap_secs > 0.0) else {
        return pick(state, source, at, planned, rng);
    };
    let ctx = Context::new(state, source, at, planned);
    let timings: Vec<Option<(f64, f64)>> =
        ctx.ordered.iter().map(|(_, t)| timing(state, *t)).collect();
    let mut lengths: Vec<f64> = timings.iter().flatten().map(|(l, _)| *l).collect();
    lengths.sort_by(f64::total_cmp);
    let longest = lengths.last().copied().unwrap_or(0.0);
    // Far from the block every known track leaves room enough: the
    // choice is the ordinary one (a track of unknown length may play).
    if fit.gap_secs >= 2.0 * longest + fit.min_play_secs {
        return pick_in(&ctx, rng);
    }
    let closable = |rest: f64| {
        let i = lengths.partition_point(|l| *l < rest);
        rest >= longest + fit.min_play_secs
            || lengths
                .get(i)
                .is_some_and(|l| *l <= rest + fit.max_cut_secs)
    };
    let class_of = |timing: Option<(f64, f64)>| {
        let (length, lead) = timing?;
        let gap = fit.gap_secs;
        if length >= gap && length - gap <= fit.max_cut_secs && gap >= fit.min_play_secs {
            return Some(FitClass::Ends);
        }
        // Still audible at the fade, it is the last before the block
        // (nothing is queued after it), whatever its segue.
        if length >= gap {
            return None;
        }
        let rest = gap - lead;
        if rest >= fit.min_play_secs {
            return Some(if closable(rest) {
                FitClass::Room
            } else {
                FitClass::LooseRoom
            });
        }
        None
    };
    let classes: Vec<Option<FitClass>> = timings.iter().map(|t| class_of(*t)).collect();
    let meets = |track: TrackId, relaxed: Relaxed| match relaxed {
        Relaxed::None => ctx.track_ok(track) && ctx.artist_ok(track),
        Relaxed::Artist => ctx.track_ok(track),
        Relaxed::Track => true,
    };
    // The fit and the separation rules both, whenever possible: a song
    // that ends the gap or leaves room a later one can close, with both
    // rules, then with the artist rule relaxed; then one that leaves any
    // room; then the track rule relaxed too.
    let (ends, room, loose) = (FitClass::Ends, FitClass::Room, FitClass::LooseRoom);
    let passes = [
        (ends, Relaxed::None),
        (room, Relaxed::None),
        (ends, Relaxed::Artist),
        (room, Relaxed::Artist),
        (loose, Relaxed::None),
        (loose, Relaxed::Artist),
        (ends, Relaxed::Track),
        (room, Relaxed::Track),
        (loose, Relaxed::Track),
    ];
    for (class, relaxed) in passes {
        let tier: Vec<TrackId> = ctx
            .ordered
            .iter()
            .zip(&classes)
            .filter(|((_, t), c)| **c == Some(class) && meets(*t, relaxed))
            .map(|((_, t), _)| *t)
            .take(PICK_SPREAD)
            .collect();
        if tier.is_empty() {
            continue;
        }
        let index = usize::try_from(rng.below(tier.len() as u64)).unwrap_or(0);
        return tier.get(index).map(|track| Picked {
            track: *track,
            relaxed,
            unfitted: false,
        });
    }
    // Nothing fits: the least cut, then the least left, then the order,
    // among the tracks that meet the separation rules, relaxed as for any
    // choice (AU34); a track of unknown length comes after every known
    // one, whatever the separation.
    let cost = |timing: Option<(f64, f64)>| match timing {
        Some((length, _)) if length >= fit.gap_secs => (length - fit.gap_secs, 0.0),
        Some((_, lead)) => (0.0, (fit.gap_secs - lead).max(0.0)),
        None => (f64::INFINITY, f64::INFINITY),
    };
    let tiers = [Relaxed::None, Relaxed::Artist, Relaxed::Track];
    tiers
        .into_iter()
        .map(|r| (true, r))
        .chain(tiers.into_iter().map(|r| (false, r)))
        .find_map(|(known, relaxed)| {
            let best = ctx
                .ordered
                .iter()
                .zip(&timings)
                .filter(|((_, t), timing)| meets(*t, relaxed) && (!known || timing.is_some()))
                .map(|((_, t), timing)| (*t, cost(*timing)))
                .reduce(|best, next| {
                    let (b, n) = (best.1, next.1);
                    if n.0.total_cmp(&b.0).then(n.1.total_cmp(&b.1)).is_lt() {
                        next
                    } else {
                        best
                    }
                })?;
            Some(Picked {
                track: best.0,
                relaxed,
                unfitted: true,
            })
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
        // What aired (or is planned) up to `at`: a time planned after it
        // (a block's song) is not an airing since.
        let mut pool_times: Vec<WallTime> = ordered
            .iter()
            .filter_map(|(_, t)| {
                let planned = planned
                    .iter()
                    .filter(|(p, w)| *p == *t && *w <= at)
                    .map(|(_, w)| *w)
                    .max();
                state.air_times.get(*t).max(planned)
            })
            .collect();
        pool_times.sort();
        let mins = state.config.automation.artist_separation_mins;
        let mut recent_artists: Vec<(String, WallTime)> = Vec::new();
        if mins > 0 {
            let window = i64::from(mins) * 60_000;
            // A planned time counts within the separation either side of
            // `at` (a block's song ahead); an aired one after `at` (the
            // clock gone back) counts as within.
            let recent = state
                .air_times
                .iter()
                .filter(|(_, when)| at.millis_since(*when) < window)
                .chain(
                    planned
                        .iter()
                        .copied()
                        .filter(|(_, when)| at.millis_since(*when).abs() < window),
                );
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

    /// The track's last airing up to `at`, aired or planned.
    fn last(&self, track: TrackId) -> Option<WallTime> {
        let planned = self
            .planned
            .iter()
            .filter(|(p, w)| *p == track && *w <= self.at)
            .map(|(_, w)| *w)
            .max();
        self.state.air_times.get(track).max(planned)
    }

    /// The track's first planned airing after `at` (a block's song).
    fn ahead(&self, track: TrackId) -> Option<WallTime> {
        self.planned
            .iter()
            .filter(|(p, w)| *p == track && *w > self.at)
            .map(|(_, w)| *w)
            .min()
    }

    /// AU31: eligible again by time or by the share of the pool aired
    /// since; a criterion set to 0 is off, and with both off every track
    /// is eligible.
    fn track_ok(&self, track: TrackId) -> bool {
        let cfg = &self.state.config.automation;
        let (mins, pct) = (cfg.track_separation_mins, cfg.track_separation_percent);
        // A planned airing ahead holds it back within the separation time
        // (the share of the pool has no meaning ahead).
        if mins > 0
            && self
                .ahead(track)
                .is_some_and(|w| w.millis_since(self.at) < i64::from(mins) * 60_000)
        {
            return false;
        }
        let Some(last) = self.last(track) else {
            return true;
        };
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
