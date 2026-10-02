//! Feedback 2 spec O22: clearing the played marks of one playlist.

use crate::error::ModelError;
use crate::ids::{EntryId, PlaylistId};
use crate::state::AppState;

/// The entries of `playlist` that `Command::ResetPlayed` would clear: those
/// with a played mark that are not the current entry of any player. The
/// current entry keeps its marks (it is on air, and its row is drawn as the
/// current one); it is marked when it is left, as always.
pub fn resettable_entries(state: &AppState, playlist: PlaylistId) -> Vec<EntryId> {
    let Some(list) = state.playlists.get(playlist) else {
        return Vec::new();
    };
    list.entries
        .iter()
        .filter(|e| !e.played_by.is_empty() || e.legacy_played)
        .filter(|e| !state.players.iter().any(|p| p.current == Some(e.id)))
        .map(|e| e.id)
        .collect()
}

/// Whether the Reset played button has anything to do for `playlist`.
pub fn can_reset_played(state: &AppState, playlist: PlaylistId) -> bool {
    !resettable_entries(state, playlist).is_empty()
}

/// Clears the played marks of `resettable_entries`. History, the next
/// entries and every other mark (other playlists) stay as they are.
pub(crate) fn reset_played(state: &mut AppState, playlist: PlaylistId) -> Result<(), ModelError> {
    if state.playlists.get(playlist).is_none() {
        return Err(ModelError::UnknownPlaylist(playlist));
    }
    for entry in resettable_entries(state, playlist) {
        if let Some(e) = state.playlists.entry_mut(entry) {
            e.played_by.clear();
            e.legacy_played = false;
        }
    }
    // A derived next is recomputed by the existing rules (they do not read
    // the marks today, so it stays; an explicit next is never touched).
    crate::reducer::refresh_next(state);
    Ok(())
}
