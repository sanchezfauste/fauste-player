//! The notice a player's header shows for its current entry's own flags
//! (feedback 2 spec O8). A pure function of the state: the UI only draws it.

use crate::ids::PlayerId;
use crate::player::Transport;
use crate::state::AppState;

/// What the current entry will do when it ends, beyond what the player's own
/// "Stop after" badge already says.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryNotice {
    /// The entry repeats (R26).
    Repeats,
    /// The entry stops the player when it ends (R27).
    StopsAfter,
}

/// The notice for `player`'s current entry, if any.
///
/// None while the player is stopped or a fade stop runs, and while stop after
/// current is set (its own badge says it). Otherwise the entry's stop-after
/// wins over its repeat, as in the rules; a file that can no longer be opened
/// plays its pass out and does not repeat.
pub fn entry_notice(state: &AppState, player: PlayerId) -> Option<EntryNotice> {
    let p = state.player(player).ok()?;
    if p.transport == Transport::Stopped || p.fade_stopping() || p.stop_after_current {
        return None;
    }
    let entry = state.playlists.entry(p.current?)?;
    if entry.stop_after {
        return Some(EntryNotice::StopsAfter);
    }
    let playable = state
        .track_for_entry(entry.id)
        .is_some_and(|t| t.file_state.is_playable());
    (entry.repeat && playable).then_some(EntryNotice::Repeats)
}
