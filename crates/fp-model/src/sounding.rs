//! External editor spec EE5: whether a track is audible right now, so the
//! app can refuse to replace or hand over a file that is on the air or
//! being pre-listened.

use crate::ids::TrackId;
use crate::state::AppState;

/// True when `track` is a player's current entry (any transport), its
/// pending start or its CUE entry; the last history entry of a player that
/// is fading; a playing cart or the cartwall CUE cart; or the library CUE
/// track while it plays or is paused (holding its audio). A library bar
/// stopped on a track (a row only selected) holds no source and does not
/// count. One rule for "Open in external editor" and the change detection
/// (external editor spec EE5).
pub fn track_sounding(state: &AppState, track: TrackId) -> bool {
    on_players_or_carts(state, track)
        || (state.library_cue.track == Some(track)
            && state.library_cue.transport != crate::player::Transport::Stopped)
}

/// The players' and the carts' part of [`track_sounding`].
fn on_players_or_carts(state: &AppState, track: TrackId) -> bool {
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
}
