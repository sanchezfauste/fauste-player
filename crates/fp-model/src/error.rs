use thiserror::Error;

use crate::ids::{CartId, CartPageId, EntryId, PlayerId, PlaylistId};

/// Why a command was refused. A refused command never changes the state.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ModelError {
    #[error("unknown player {0:?}")]
    UnknownPlayer(PlayerId),
    #[error("unknown playlist {0:?}")]
    UnknownPlaylist(PlaylistId),
    #[error("unknown playlist entry {0:?}")]
    UnknownEntry(EntryId),
    #[error("there are no playlists")]
    NoPlaylists,
    #[error("the last playlist cannot be deleted")]
    LastPlaylist,
    #[error("playlist {0:?} contains an entry that is on air")]
    PlaylistOnAir(PlaylistId),
    #[error("entry {0:?} is on air and cannot be removed")]
    EntryOnAir(EntryId),
    #[error("the current entry cannot be set as next")]
    NextIsCurrent,
    #[error("stop after current is only available in continuous mode")]
    StopAfterInSingle,
    #[error("player count {requested} is outside 1..={max}")]
    PlayerCountOutOfRange { requested: usize, max: usize },
    #[error("player {0:?} is busy and cannot be removed")]
    PlayerBusy(PlayerId),
    #[error("unknown cart {0:?}")]
    UnknownCart(CartId),
    #[error("unknown cart page {0:?}")]
    UnknownCartPage(CartPageId),
    #[error("unknown cart position {0} on the page")]
    UnknownCartPosition(usize),
    #[error("the last cart page cannot be deleted")]
    LastCartPage,
    #[error("the new grid would drop carts that have a file")]
    CartsWouldBeLost,
    #[error("the cart grid is outside the configured limits")]
    CartGridOutOfRange,
}
