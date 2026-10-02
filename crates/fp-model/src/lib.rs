//! Domain model of Fauste Player: tracks, playlists, players, configuration
//! and the pure state machine that implements the player behaviour rules.
//! This crate performs no I/O and spawns no threads.

pub mod availability;
mod cart_rules;
pub mod cartwall;
pub mod columns;
pub mod command;
pub mod config;
mod entry_notice;
pub mod error;
pub mod ids;
pub mod midi;
pub mod on_air;
pub mod player;
pub mod playlist;
pub mod reducer;
pub mod remote;
pub mod reset_played;
pub mod restart;
pub mod restore;
pub mod session;
pub mod shortcuts;
pub mod state;
mod tag_edit;
mod tag_sheet;
pub mod track;
pub mod volume;

pub use availability::{Availability, availability, command_available};
pub use cartwall::{
    Cart, CartEdit, CartFileChange, CartKind, CartPage, CartPageImport, Cartwall, CartwallSession,
    PlayingCart,
};
pub use columns::{
    TableColumn, column_rows, default_columns, move_column, normalize_columns, with_column_shown,
};
pub use command::{
    CartRequest, Command, EngineAction, EngineEvent, SOURCE_END, SourceRequest, TransitionPlan,
};
pub use config::{
    AnalysisSettings, CartwallConfig, CartwallRoutes, Config, ConfigWarning, Limits,
    LoudnessReadout, MeterBallistics, MeterConfig, MeterSettings, OutputDevice, OutputsConfig,
    PlayerRoutes, PlayersConfig, Route, Tuning, UiConfig,
};
pub use entry_notice::{EntryNotice, entry_notice};
pub use error::ModelError;
pub use ids::{CartId, CartPageId, EntryId, IdGen, PlayerId, PlaylistId, TrackId};
pub use midi::{MidiAction, MidiBinding, MidiConfig, MidiDevice, MidiTrigger};
pub use on_air::{OnAir, on_air};
pub use player::{ColumnWidths, CueState, PlayMode, PlayerState, Transport};
pub use playlist::{Playlist, PlaylistEntry, Playlists};
pub use reducer::{apply, on_event, plan_for};
pub use remote::{HttpRemoteConfig, OscRemoteConfig, RemoteConfig, RemoteEventsConfig};
pub use reset_played::{can_reset_played, resettable_entries};
pub use restart::{RestartReason, restart_pending};
pub use restore::{SettingsSection, restore_defaults};
pub use session::{PlayerSession, RestoreParts};
pub use shortcuts::{KeyChord, Shortcut, ShortcutAction, default_shortcuts, player_command};
pub use state::AppState;
pub use tag_edit::{TagEditBlock, tag_edit_block};
pub use tag_sheet::{
    CoverArt, FieldProblem, TagField, TagFieldKind, TagSheet, changed_fields, cover_blocked,
    cover_changed, cover_unstored, field_problem, invalid_fields, unstored_fields,
};
pub use track::{
    AudioFormat, FileState, InvalidDate, Library, Marker, MarkerKind, MarkerSource, Markers,
    PlayRange, Track, TrackAnalysis, TrackKind, TrackTags, parse_tag_date,
};
