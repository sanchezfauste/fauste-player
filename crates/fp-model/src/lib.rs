//! Domain model of Fauste Player: tracks, playlists, players, configuration
//! and the pure state machine that implements the player behaviour rules.
//! This crate performs no I/O and spawns no threads.

pub mod command;
pub mod config;
pub mod error;
pub mod ids;
pub mod player;
pub mod playlist;
pub mod reducer;
pub mod state;
pub mod track;

pub use command::{Command, EngineAction, EngineEvent, SourceRequest, TransitionPlan};
pub use config::{
    AnalysisSettings, Config, ConfigWarning, Limits, OutputsConfig, PlayerRoutes, PlayersConfig,
    Route, Tuning, UiConfig,
};
pub use error::ModelError;
pub use ids::{EntryId, IdGen, PlayerId, PlaylistId, TrackId};
pub use player::{ColumnWidths, CueState, PlayMode, PlayerState, Transport};
pub use playlist::{Playlist, PlaylistEntry, Playlists};
pub use reducer::{apply, on_event};
pub use state::AppState;
pub use track::{FileState, Library, Marker, MarkerKind, MarkerSource, Markers, Track, TrackKind};
