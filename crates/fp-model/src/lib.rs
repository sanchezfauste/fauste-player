//! Domain model of Fauste Player: tracks, playlists, players, configuration
//! and the pure state machine that implements the player behaviour rules.
//! This crate performs no I/O and spawns no threads.

pub mod air_times;
pub mod as_run;
pub mod audio_change;
pub mod automation;
pub mod availability;
pub mod bit_perfect;
mod cart_rules;
pub mod cartwall;
pub mod category;
pub mod columns;
pub mod command;
pub mod config;
pub mod cue_on_air;
pub mod device_offer;
pub mod dsd;
mod entry_notice;
pub mod error;
pub mod external_edit;
pub mod filler;
pub mod ids;
pub mod insertion;
pub mod layout;
pub mod library_cue;
mod library_rules;
pub mod library_search;
pub mod library_view;
pub mod live;
pub mod local_time;
pub mod log_edit;
pub mod loudness;
pub mod midi;
pub mod on_air;
mod pending_start;
pub mod player;
pub mod playlist;
mod playout;
pub mod reducer;
pub mod remote;
pub mod reset_played;
pub mod restore;
pub mod rotation;
pub mod rotation_cursor;
pub mod schedule;
pub mod selection;
pub mod session;
pub mod shortcuts;
pub mod sounding;
pub mod state;
mod tag_edit;
mod tag_sheet;
pub mod track;
pub mod volume;
pub mod wall_time;

pub use air_times::AirTimes;
pub use audio_change::{AudioChange, audio_change};
pub use availability::{Availability, availability, command_available};
pub use bit_perfect::{bit_perfect_player, fader_locked, faders_reset_by, rate_cut};
pub use cartwall::{
    Cart, CartColor, CartEdit, CartFileChange, CartKind, CartPage, CartPageImport, Cartwall,
    CartwallSession, PlayingCart,
};
pub use category::{Categories, Category, check_name, watching_category};
pub use columns::{
    TableColumn, column_rows, default_columns, move_column, move_column_before, normalize_columns,
    with_column_shown,
};
pub use command::{
    CartRequest, Command, CueStart, EngineAction, EngineEvent, LibraryCueRequest, SOURCE_END,
    SourceRequest, TransitionPlan,
};
pub use config::{
    AnalysisSettings, AutomationConfig, BUFFER_FRAMES_RANGE, CartwallConfig, CartwallRoutes,
    Config, ConfigWarning, DeviceOverride, LibraryConfig, LibraryRoutes, Limits, LoudnessConfig,
    LoudnessReadout, MeterBallistics, MeterConfig, MeterSettings, OutputDevice, OutputsConfig,
    OutputsView, PlayerRoutes, PlayersConfig, Route, SAMPLE_RATE_RANGE, Tuning, UiConfig,
    WindowSystem,
};
pub use cue_on_air::{CueConfirmation, CueOnAir, CueStopped, needs_confirmation};
pub use device_offer::{
    DsdCaps, DsdNotOffered, buffer_is_reported, dsd_not_offered, effective_dsd_mode,
    offered_buffers, offered_dsd_modes, offered_rates, rate_is_reported,
};
pub use dsd::{
    BpBadge, DEFAULT_DSD_SILENCE_MS, DSD_WORD_BITS, DsdDevice, DsdFacts, DsdFallback, DsdMix,
    DsdOnAir, DsdOutput, DsdStreamMode, DsdTarget, bp_badge, dsd_decision, dsd_holds_others,
};
pub use entry_notice::{EntryNotice, entry_notice};
pub use error::ModelError;
pub use external_edit::{ExternalEditBlock, external_edit_block};
pub use ids::{
    BlockId, CartId, CartPageId, CategoryId, EntryId, IdGen, PlayerId, PlaylistId, RotationId,
    TemplateId, TrackId,
};
pub use insertion::after_next;
pub use layout::{
    CartwallLayout, DEFAULT_SIDE_SHARE, Dock, Layout, MAX_CARTWALL_SHARE, MIN_CARTWALL_SHARE,
    Panel, settle_layout,
};
pub use library_rules::{
    OnAirHolds, RemovalImpact, category_deletion, missing_tracks, removal_impact,
};
pub use library_view::{
    LibraryColumn, LibraryColumnWidth, LibrarySelection, LibrarySort, LibraryView,
    lenient_library_view,
};
pub use live::{
    BusyCause, DeviceSettings, Failure, Holder, LiveSettings, Pending, PendingItem, Target, Wanted,
    causes, configured_route, device_causes, device_settings, due, has_output_items, holders,
    interruptions, pending, target_and_wanted,
};
pub use loudness::{
    GainLimit, LOUDNESS_VERSION, LoudnessEdit, LoudnessMeasurement, Normalization,
    NormalizationStatus, holder_bit_perfect, loudness_unmeasured, max_trim_in_use, normalization,
    normalization_for, trim_in_use, wanted_gain_db,
};
pub use midi::{MidiAction, MidiBinding, MidiConfig, MidiDevice, MidiTrigger};
pub use on_air::{OnAir, on_air};
pub use player::{ColumnWidths, CueState, FadeOrigin, PlayMode, PlayerState, Transport};
pub use playlist::{Playlist, PlaylistEntry, Playlists};
pub use playout::playout_active;
pub use reducer::{apply, on_event, plan_for};
pub use remote::{HttpRemoteConfig, OscRemoteConfig, RemoteConfig, RemoteEventsConfig};
pub use reset_played::{can_reset_played, resettable_entries};
pub use restore::{SettingsSection, restore_defaults};
pub use rotation::{Rotation, RotationStep, Rotations, StepSource};
pub use session::{PlayerSession, RestoreParts};
pub use shortcuts::{KeyChord, Shortcut, ShortcutAction, default_shortcuts, player_command};
pub use sounding::track_sounding;
pub use state::AppState;
pub use tag_edit::{TagEditBlock, tag_edit_block};
pub use tag_sheet::{
    CoverArt, FieldProblem, TagField, TagFieldKind, TagSheet, changed_fields, cover_blocked,
    cover_changed, cover_unstored, field_problem, invalid_fields, unstored_fields,
};
pub use track::{
    AudioFormat, AudioHash, FileStamp, FileState, InvalidDate, Library, LibraryChanges, Marker,
    MarkerKind, MarkerSource, Markers, MarkersReview, PlayRange, Track, TrackAnalysis, TrackKind,
    TrackTags, parse_tag_date,
};
pub use wall_time::{SplitMix64, WallTime};
