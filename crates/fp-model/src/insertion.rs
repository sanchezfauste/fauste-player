//! Where tracks go when they are put "after the next" of a player (phone
//! layout spec PL10b, PL19).

use crate::ids::{PlayerId, PlaylistId};
use crate::state::AppState;

/// The playlist `player` shows and the index right after its next entry
/// when that playlist holds it; otherwise the end of that playlist. `None`
/// for an unknown player.
pub fn after_next(state: &AppState, player: PlayerId) -> Option<(PlaylistId, usize)> {
    let p = state.player(player).ok()?;
    let list = state.playlists.get(p.playlist)?;
    let index = p
        .next
        .and_then(|next| list.entries.iter().position(|e| e.id == next))
        .map_or(list.entries.len(), |i| i + 1);
    Some((p.playlist, index))
}
