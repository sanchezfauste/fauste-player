//! Whether playout is active (mobile spec MB1): what a mobile shell keeps
//! its process alive for.

use crate::{AppState, Transport};

/// True while any player has a track loaded (current or next, even
/// stopped), is not stopped or pre-listens on CUE, or a cart plays or
/// pre-listens (mobile spec MB1). A stopped player with its next track
/// ready counts: the operator is about to start it.
pub fn playout_active(state: &AppState) -> bool {
    let players = state.players.iter().any(|p| {
        p.current.is_some()
            || p.next.is_some()
            || p.transport != Transport::Stopped
            || p.cue.is_some()
    });
    players || !state.cartwall.playing.is_empty() || state.cartwall.cue.is_some()
}
