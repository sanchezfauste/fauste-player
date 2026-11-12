use thiserror::Error;

use crate::ids::{CartId, CartPageId, CategoryId, EntryId, PlayerId, PlaylistId, TrackId};

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
    #[error("in single mode, stop after current is only available while the current entry repeats")]
    StopAfterInSingle,
    #[error("player count {requested} is outside 1..={max}")]
    PlayerCountOutOfRange { requested: usize, max: usize },
    #[error("player {0:?} is busy and cannot be removed")]
    PlayerBusy(PlayerId),
    #[error("unknown track {0:?}")]
    UnknownTrack(TrackId),
    #[error("the marker would put cue-in at or after cue-out")]
    InvalidMarker,
    #[error("the loudness trim must be a number within the maximum trim")]
    InvalidTrim,
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
    /// Bit-perfect rate and fader spec BF3.
    #[error("the volume of player {} is held at 100 % on a bit-perfect output", .0.0)]
    FaderLocked(PlayerId),
    /// Resizable layout spec RL4, RL6: a resize between players that are not
    /// neighbours, or a share or size that is not a usable number.
    #[error("that layout change is not valid")]
    BadLayout,
    /// RL7: the last shown panel cannot be hidden.
    #[error("the last shown panel cannot be hidden")]
    LastPanel,
    /// LB17b: tracks to remove are on air; nothing was removed.
    #[error("{} entries and {} carts of these tracks are on air", .0.entries.len(), .0.carts.len())]
    LibraryOnAir(crate::library_rules::OnAirHolds),
    #[error("unknown category {0:?}")]
    UnknownCategory(CategoryId),
    /// LB19: the category that already watches the folder.
    #[error("category {0:?} already watches that folder")]
    FolderAlreadyWatched(CategoryId),
    #[error("a category name must not be empty")]
    CategoryNameEmpty,
    #[error("a category with that name already exists there")]
    CategoryNameTaken,
    #[error("categories cannot be nested that deep")]
    CategoryTooDeep,
    #[error("a category cannot be moved into itself")]
    CategoryIntoItself,
    /// RL9: the layout is locked.
    #[error("the layout is locked")]
    LayoutLocked,
    #[error("unknown rotation {0:?}")]
    UnknownRotation(crate::ids::RotationId),
    /// AU1: a step's counts must lie in 1–99 with min ≤ max.
    #[error("a rotation step's counts must lie in 1–99, the first not above the second")]
    InvalidRotationStep,
    #[error("a rotation name must not be empty")]
    EmptyRotationName,
    /// A rotation with no step would give the filler nothing to play.
    #[error("a rotation needs at least one step")]
    EmptyRotation,
    /// AU4: there is always a rotation to fall back on.
    #[error("the last rotation cannot be deleted")]
    LastRotation,
    /// AU6: a rotation fills the rest of its block.
    #[error("a rotation must be the last item of a block")]
    RotationNotLast,
    #[error("unknown template {0:?}")]
    UnknownTemplate(crate::ids::TemplateId),
    #[error("unknown block {0:?}")]
    UnknownBlock(crate::ids::BlockId),
    #[error("a template name must not be empty")]
    EmptyTemplateName,
    /// AU9: every weekday needs a template.
    #[error("the last template cannot be deleted")]
    LastTemplate,
    #[error("the week plan uses this template")]
    TemplateInUse,
    /// AU5: a block lasts until the next one's start.
    #[error("another block starts at that time")]
    BlockTimeTaken,
    /// Automation spec §8.1: the music filler's list belongs to the filler;
    /// it is not renamed, deleted or shown on another player, and the
    /// filler shows no other list.
    #[error("{0:?} is the music filler's list")]
    FillerPlaylist(crate::ids::PlaylistId),
}
