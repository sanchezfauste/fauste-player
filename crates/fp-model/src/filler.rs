//! The music filler (automation spec AU27–AU29): a player of its own,
//! kept last in `AppState::players`, that fills its own playlist from a
//! rotation and trims what it played. The playlist is never listed with
//! the operator's playlists.

use std::collections::HashSet;

use crate::command::EngineAction;
use crate::ids::{EntryId, PlayerId, PlaylistId, RotationId};
use crate::player::{PlayMode, PlayerState};
use crate::playlist::Playlist;
use crate::rotation_cursor::RotationCursor;
use crate::state::AppState;

#[derive(Debug, Clone, PartialEq)]
pub struct FillerState {
    pub player: PlayerId,
    pub playlist: PlaylistId,
    /// The rotation it plays; `None` plays the default (AU4).
    pub rotation: Option<RotationId>,
    pub cursor: RotationCursor,
    /// Rows the filler added and may replace (AU28); every other row of
    /// the playlist was added or moved by the operator.
    pub generated: HashSet<EntryId>,
    /// The last top-up found nothing to play (AU34 alert); runtime only.
    pub starved: bool,
}

/// Creates the filler (a stopped player last in `players` and an empty
/// playlist named `playlist_name`) when there is none; returns the engine
/// actions.
pub fn ensure(state: &mut AppState, playlist_name: &str) -> Vec<EngineAction> {
    if state.filler.is_some() {
        return Vec::new();
    }
    let playlist = state.ids.playlist();
    state.playlists.add(Playlist::new(playlist, playlist_name));
    let player = state.ids.player();
    state
        .players
        .push(PlayerState::new(player, playlist, PlayMode::Continuous));
    let cursor = RotationCursor::new(state.rotations.default_id().unwrap_or(RotationId(0)));
    state.filler = Some(FillerState {
        player,
        playlist,
        rotation: None,
        cursor,
        generated: HashSet::new(),
        starved: false,
    });
    vec![EngineAction::AddPlayer { player }]
}
