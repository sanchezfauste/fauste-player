//! Rule 3a, pending start (operator feedback 4, Q8): where Play starts the
//! next entry of a stopped player, when the operator chose it with a seek.
//! A pending start is not on air: it only changes where Play starts.

use crate::command::SourceRequest;
use crate::ids::EntryId;
use crate::player::{PlayerState, Transport};
use crate::state::AppState;

/// Q8.2: the pending start that a seek to `secs` stores on a stopped player
/// whose next is `entry`, raised to the entry's cue-in. `None` for a
/// broken `secs`, an entry whose file cannot be played, or a start at or
/// past the end of the play range (Play would end the track at once).
pub(crate) fn pending_start_at(
    state: &AppState,
    entry: EntryId,
    secs: f64,
) -> Option<(EntryId, f64)> {
    if !secs.is_finite() {
        return None;
    }
    state.playable_request(entry)?;
    let track = state.track_for_entry(entry)?;
    let range = track.play_range(state.config.players.use_cue_markers);
    let at = secs.max(range.cue_in);
    if range.known_end().is_some_and(|end| at >= end) {
        return None;
    }
    Some((entry, at))
}

/// Q8.4: a pending start lives only while its player is stopped and its
/// entry is the player's next. It is clamped again to the entry's current
/// play range, which an analysis or a marker may have moved.
pub(crate) fn settle(state: &mut AppState) {
    let kept: Vec<Option<(EntryId, f64)>> = state
        .players
        .iter()
        .map(|p| {
            let (entry, secs) = p.pending_start?;
            if p.transport == Transport::Stopped && p.next == Some(entry) {
                pending_start_at(state, entry, secs)
            } else {
                None
            }
        })
        .collect();
    for (player, kept) in state.players.iter_mut().zip(kept) {
        player.pending_start = kept;
    }
}

/// Q8.3: what Play while Stopped sends for `request`: the same entry at the
/// pending start taken from the player, when that start is for it.
pub(crate) fn start_request(
    state: &AppState,
    pending: Option<(EntryId, f64)>,
    request: SourceRequest,
) -> SourceRequest {
    match pending {
        Some((entry, secs)) if entry == request.entry => {
            state.request_at(entry, secs).unwrap_or(request)
        }
        _ => request,
    }
}

/// Q8.7: where the engine preloads `entry` for `player`: at the pending
/// start while the player is stopped with one for it, else at the cue-in.
pub(crate) fn preload_request(
    state: &AppState,
    player: &PlayerState,
    entry: EntryId,
) -> Option<SourceRequest> {
    match player.pending_start {
        Some((e, secs)) if e == entry && player.transport == Transport::Stopped => {
            state.request_at(e, secs)
        }
        _ => state.request_from_cue_in(entry),
    }
}
