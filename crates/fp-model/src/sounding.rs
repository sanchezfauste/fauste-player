//! External editor spec EE12: whether a track is audible right now, so the
//! app can refuse to replace or hand over a file that is on the air or
//! being pre-listened.

use crate::ids::TrackId;
use crate::state::AppState;

/// True when `track` is a player's current entry (any transport), its
/// pending start or its CUE entry; the last history entry of a player that
/// is fading; a playing cart or the cartwall CUE cart; or the library CUE
/// track (any transport).
pub fn track_sounding(state: &AppState, track: TrackId) -> bool {
    let of_entry = |entry| {
        state
            .playlists
            .entry(entry)
            .is_some_and(|e| e.track == track)
    };
    let of_cart = |cart| {
        state
            .cartwall
            .cart(cart)
            .is_some_and(|c| c.track == Some(track))
    };
    state.players.iter().any(|p| {
        p.current.is_some_and(of_entry)
            || p.pending_start.is_some_and(|(entry, _)| of_entry(entry))
            || p.cue.is_some_and(|c| of_entry(c.entry))
            || (p.fading && p.history.last().copied().is_some_and(of_entry))
    }) || state.cartwall.playing.iter().any(|c| of_cart(c.cart))
        || state.cartwall.cue.is_some_and(of_cart)
        || state.library_cue.track == Some(track)
}
