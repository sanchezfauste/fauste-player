//! External editor spec EE2-EE5: whether a track may be handed to the
//! operator's external editor right now.

use crate::ids::TrackId;
use crate::sounding::track_sounding;
use crate::state::AppState;

/// Why "Open in external editor" is disabled for a track.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalEditBlock {
    /// No editor is configured (EE4).
    NoEditor,
    /// The track is unknown or its file is missing or unreadable (EE4).
    FileUnavailable,
    /// The track is sounding (EE5).
    Sounding,
}

/// The first reason, in the order of EE4, why `track` cannot be opened in
/// the external editor; `None` when it can (EE2, EE3).
pub fn external_edit_block(state: &AppState, track: TrackId) -> Option<ExternalEditBlock> {
    if state.config.library.external_editor.is_none() {
        return Some(ExternalEditBlock::NoEditor);
    }
    if !state
        .library
        .get(track)
        .is_some_and(|t| t.file_state.is_playable())
    {
        return Some(ExternalEditBlock::FileUnavailable);
    }
    if track_sounding(state, track) {
        return Some(ExternalEditBlock::Sounding);
    }
    None
}
