//! When a track's tags may be edited (feedback 2 spec O23): a pure rule of
//! the state, so the menu and the Save button agree.

use crate::ids::TrackId;
use crate::state::AppState;

/// Why the tags of a track cannot be edited now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagEditBlock {
    /// The file is missing or cannot be read (or the track is unknown).
    FileUnavailable,
    /// The format has no writable tags.
    UnsupportedFormat,
    /// The tag-only pass has not read this track's tags yet.
    TagsNotRead,
    /// The track is the current entry of a player, playing or paused.
    OnAir,
    /// The track is the CUE entry of a player.
    Cued,
    /// The track is on a playing cart.
    OnCart,
}

/// The first reason `track` cannot be edited, or `None`. `format_writable`
/// says whether the file's format has writable tags (`fp-analysis` decides
/// that from the extension). It judges the file, so the same track in
/// another playlist is refused too.
pub fn tag_edit_block(
    state: &AppState,
    track: TrackId,
    format_writable: bool,
) -> Option<TagEditBlock> {
    let Some(t) = state.library.get(track) else {
        return Some(TagEditBlock::FileUnavailable);
    };
    if !t.file_state.is_playable() {
        return Some(TagEditBlock::FileUnavailable);
    }
    if !format_writable {
        return Some(TagEditBlock::UnsupportedFormat);
    }
    if !t.tags_read {
        return Some(TagEditBlock::TagsNotRead);
    }
    let is_this_track = |entry| {
        state
            .playlists
            .entry(entry)
            .is_some_and(|e| e.track == track)
    };
    if state
        .players
        .iter()
        .any(|p| p.current.is_some_and(is_this_track))
    {
        return Some(TagEditBlock::OnAir);
    }
    if state
        .players
        .iter()
        .any(|p| p.cue.is_some_and(|c| is_this_track(c.entry)))
    {
        return Some(TagEditBlock::Cued);
    }
    let on_cart = state.cartwall.playing.iter().any(|playing| {
        state
            .cartwall
            .cart(playing.cart)
            .is_some_and(|c| c.track == Some(track))
    });
    on_cart.then_some(TagEditBlock::OnCart)
}
