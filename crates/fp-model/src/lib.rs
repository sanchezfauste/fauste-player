//! Domain model of Fauste Player: tracks, playlists, players, configuration
//! and the pure state machine that implements the player behaviour rules.
//! This crate performs no I/O and spawns no threads.

mod cart_rules;
pub mod cartwall;
pub mod command;
pub mod config;
pub mod error;
pub mod ids;
pub mod player;
pub mod playlist;
pub mod reducer;
pub mod session;
pub mod state;
pub mod track;

pub use cartwall::{Cart, CartEdit, CartKind, CartPage, CartPageImport, Cartwall, PlayingCart};
pub use command::{
    CartRequest, Command, EngineAction, EngineEvent, SOURCE_END, SourceRequest, TransitionPlan,
};
pub use config::{
    AnalysisSettings, CartwallConfig, Config, ConfigWarning, Limits, OutputsConfig, PlayerRoutes,
    PlayersConfig, Route, Tuning, UiConfig,
};
pub use error::ModelError;
pub use ids::{CartId, CartPageId, EntryId, IdGen, PlayerId, PlaylistId, TrackId};
pub use player::{ColumnWidths, CueState, PlayMode, PlayerState, Transport};
pub use playlist::{Playlist, PlaylistEntry, Playlists};
pub use reducer::{apply, on_event, plan_for};
pub use session::{PlayerSession, RestoreParts};
pub use state::AppState;
pub use track::{
    FileState, Library, Marker, MarkerKind, MarkerSource, Markers, Track, TrackAnalysis, TrackKind,
};
