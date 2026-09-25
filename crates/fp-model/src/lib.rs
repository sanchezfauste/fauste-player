//! Domain model of Fauste Player: tracks, playlists, players, configuration
//! and the pure state machine that implements the player behaviour rules.
//! This crate performs no I/O and spawns no threads.

pub mod ids;

pub use ids::{EntryId, IdGen, PlayerId, PlaylistId, TrackId};
