//! Domain model of Fauste Player: tracks, playlists, players, configuration
//! and the pure state machine that implements the player behaviour rules.
//! This crate performs no I/O and spawns no threads.

pub mod availability;
mod cart_rules;
pub mod cartwall;
pub mod command;
pub mod config;
pub mod error;
pub mod ids;
pub mod midi;
pub mod on_air;
pub mod player;
pub mod playlist;
pub mod reducer;
pub mod remote;
pub mod restart;
pub mod restore;
pub mod session;
pub mod shortcuts;
pub mod state;
pub mod track;
pub mod volume;

pub use availability::{Availability, availability, command_available};
pub use cartwall::{
    Cart, CartEdit, CartFileChange, CartKind, CartPage, CartPageImport, Cartwall, CartwallSession,
    PlayingCart,
};
pub use command::{
    CartRequest, Command, EngineAction, EngineEvent, SOURCE_END, SourceRequest, TransitionPlan,
};
pub use config::{
    AnalysisSettings, CartwallConfig, CartwallRoutes, Config, ConfigWarning, Limits,
    LoudnessReadout, MeterBallistics, MeterConfig, MeterSettings, OutputDevice, OutputsConfig,
    PlayerRoutes, PlayersConfig, Route, Tuning, UiConfig,
};
pub use error::ModelError;
pub use ids::{CartId, CartPageId, EntryId, IdGen, PlayerId, PlaylistId, TrackId};
pub use midi::{MidiAction, MidiBinding, MidiConfig, MidiDevice, MidiTrigger};
pub use on_air::{OnAir, on_air};
pub use player::{ColumnWidths, CueState, PlayMode, PlayerState, Transport};
pub use playlist::{Playlist, PlaylistEntry, Playlists};
pub use reducer::{apply, on_event, plan_for};
pub use remote::{HttpRemoteConfig, OscRemoteConfig, RemoteConfig, RemoteEventsConfig};
pub use restart::{RestartReason, restart_pending};
pub use restore::{SettingsSection, restore_defaults};
pub use session::{PlayerSession, RestoreParts};
pub use shortcuts::{KeyChord, Shortcut, ShortcutAction, default_shortcuts, player_command};
pub use state::AppState;
pub use track::{
    AudioFormat, FileState, Library, Marker, MarkerKind, MarkerSource, Markers, PlayRange, Track,
    TrackAnalysis, TrackKind,
};
