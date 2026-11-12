//! Walking a rotation (automation spec AU2, AU34): the steps in order,
//! again from the first after the last, each playing a count drawn afresh
//! in its range every time it comes up. A step that draws 0 sits out that
//! turn; when every step of a pass draws 0, the next pass draws at least 1
//! for each, so the rotation always plays. A step that yields nothing is
//! skipped; a rotation that yields nothing falls back on the default.

use serde::{Deserialize, Serialize};

use crate::ids::RotationId;
use crate::rotation::Rotation;
use crate::selection::{Fit, Picked, Planned, pick_fitted};
use crate::state::AppState;
use crate::wall_time::{SplitMix64, WallTime};

/// Where a walk through a rotation stands. Plain data, saved with the
/// filler.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RotationCursor {
    pub rotation: RotationId,
    /// The step being played.
    pub step: usize,
    /// Tracks the step still plays before the next step.
    pub left: u8,
    /// Nothing played yet: the walk starts at the first step.
    pub fresh: bool,
}

impl RotationCursor {
    pub fn new(rotation: RotationId) -> Self {
        Self {
            rotation,
            step: 0,
            left: 0,
            fresh: true,
        }
    }
}

/// What one call of [`next_track`] found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorPick {
    pub picked: Option<Picked>,
    /// The steps of the cursor's rotation that yielded nothing on this call,
    /// for the caller's log.
    pub skipped_steps: Vec<usize>,
    /// AU34: the track came from the default rotation (or nothing did,
    /// after trying it).
    pub used_default: bool,
}

/// The next track of the cursor's rotation, moving the cursor on.
pub fn next_track(
    state: &AppState,
    cursor: &mut RotationCursor,
    at: WallTime,
    planned: Planned<'_>,
    rng: &mut SplitMix64,
) -> CursorPick {
    next_track_fitted(state, cursor, at, planned, rng, None)
}

/// `next_track`, each step's choice preferring tracks that `fit` (AU25).
pub fn next_track_fitted(
    state: &AppState,
    cursor: &mut RotationCursor,
    at: WallTime,
    planned: Planned<'_>,
    rng: &mut SplitMix64,
    fit: Option<&Fit>,
) -> CursorPick {
    let mut out = CursorPick {
        picked: None,
        skipped_steps: Vec::new(),
        used_default: false,
    };
    let default = state.rotations.default_rotation();
    let own = state.rotations.get(cursor.rotation);
    if let Some(rotation) = own {
        out.picked = walk(
            state,
            rotation,
            cursor,
            at,
            planned,
            rng,
            fit,
            &mut out.skipped_steps,
        );
        if out.picked.is_some() {
            return out;
        }
    }
    // An unknown rotation, or one whose steps all came up empty: the
    // default, through a cursor of its own (the caller's keeps its place).
    if let Some(fallback) = default.filter(|d| own.is_none_or(|r| r.id != d.id)) {
        let mut temporary = RotationCursor::new(fallback.id);
        let mut ignored = Vec::new();
        out.picked = walk(
            state,
            fallback,
            &mut temporary,
            at,
            planned,
            rng,
            fit,
            &mut ignored,
        );
        out.used_default = true;
    } else if own.is_none() {
        out.used_default = true;
    }
    out
}

/// Two passes at most over `rotation`'s steps from where `cursor` stands;
/// in the second, a step draws at least 1 (a pass of zero draws must not
/// leave the rotation silent).
#[allow(clippy::too_many_arguments)]
fn walk(
    state: &AppState,
    rotation: &Rotation,
    cursor: &mut RotationCursor,
    at: WallTime,
    planned: Planned<'_>,
    rng: &mut SplitMix64,
    fit: Option<&Fit>,
    skipped: &mut Vec<usize>,
) -> Option<Picked> {
    let len = rotation.steps.len();
    if len == 0 {
        return None;
    }
    for turn in 0..2 * len {
        let at_least_one = turn >= len;
        if cursor.fresh || cursor.step >= len {
            cursor.fresh = false;
            cursor.step = 0;
            cursor.left = draw(rotation, 0, at_least_one, rng);
        } else if cursor.left == 0 {
            cursor.step = (cursor.step + 1) % len;
            cursor.left = draw(rotation, cursor.step, at_least_one, rng);
        }
        if cursor.left == 0 {
            // Drew 0: the step sits out this turn.
            continue;
        }
        let step = rotation.steps.get(cursor.step)?;
        match pick_fitted(state, &step.source, at, planned, rng, fit) {
            Some(picked) => {
                cursor.left = cursor.left.saturating_sub(1);
                return Some(picked);
            }
            None => {
                if !skipped.contains(&cursor.step) {
                    skipped.push(cursor.step);
                }
                cursor.left = 0;
            }
        }
    }
    None
}

/// A step's count for this turn, in its range; at least 1 when asked.
fn draw(rotation: &Rotation, step: usize, at_least_one: bool, rng: &mut SplitMix64) -> u8 {
    rotation.steps.get(step).map_or(1, |s| {
        let min = if at_least_one { s.min.max(1) } else { s.min };
        rng.in_range(min, s.max.max(min))
    })
}
