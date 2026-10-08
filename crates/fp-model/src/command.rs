//! The vocabulary between the UI, the model and the audio engine.

use std::path::PathBuf;

use crate::cartwall::{CartEdit, CartFileChange, CartPageImport};
use crate::config::{Config, OutputDevice, Route};
use crate::ids::{CartId, CartPageId, EntryId, PlayerId, PlaylistId, TrackId};
use crate::live::{DeviceSettings, Holder, Target, Wanted};
use crate::player::{ColumnWidths, PlayMode};
use crate::shortcuts::{KeyChord, ShortcutAction};
use crate::track::{AudioFormat, FileState, MarkerKind, TrackAnalysis, TrackTags};

/// A user intent, sent by the UI.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Play(PlayerId),
    Pause(PlayerId),
    Stop(PlayerId),
    FadeStop(PlayerId),
    /// R23: back to the current entry's cue-in; a paused player stays
    /// paused.
    Restart(PlayerId),
    /// R24: crossfade back to the last entry this player left.
    Previous(PlayerId),
    SetNext(PlayerId, EntryId),
    InsertPaths {
        playlist: PlaylistId,
        index: usize,
        paths: Vec<PathBuf>,
    },
    /// Entries for tracks already in the library (remote control spec
    /// §3.4): the same tracks, with their markers and analysis.
    InsertTracks {
        playlist: PlaylistId,
        index: usize,
        tracks: Vec<TrackId>,
    },
    SetMode(PlayerId, PlayMode),
    ToggleStopAfterCurrent(PlayerId),
    ToggleCue(PlayerId),
    /// Cue on or off, whatever it is now: sending it twice is harmless
    /// (remote control spec §2). Likewise the other `Set…` toggles below.
    SetCue(PlayerId, bool),
    SetStopAfterCurrent(PlayerId, bool),
    CueEntry(PlayerId, EntryId),
    /// Feedback 2 spec O12: moves the running CUE to `secs` of its entry
    /// (clamped to the file). A broken value, or no CUE, does nothing.
    SeekCue(PlayerId, f64),
    /// Spec O12: holds or releases the running CUE. Idempotent; without a
    /// CUE it does nothing.
    SetCuePaused(PlayerId, bool),
    /// Spec O12 ("Set as next"): the cued entry becomes the player's
    /// explicit next and the CUE keeps running. Without a CUE it does
    /// nothing.
    CueToNext(PlayerId),
    SetVolume(PlayerId, f32),
    Seek(PlayerId, f64),
    ShowPlaylist(PlayerId, PlaylistId),
    SetColumnWidths(PlayerId, ColumnWidths),
    /// Stores an analysis result for a track (ignored if the track is gone).
    ApplyAnalysis {
        track: TrackId,
        analysis: Box<TrackAnalysis>,
    },
    /// Feedback 2 spec O23: the file's tags as read after the tag-only pass
    /// or written by the tag editor (ignored if the track is gone).
    ApplyTags {
        track: TrackId,
        tags: Box<TrackTags>,
    },
    /// Operator feedback 4, Q1.2: the length the file's header declares,
    /// stored only while the track is not analysed and `secs` is finite
    /// and positive (ignored if the track is gone). The analysis replaces
    /// it.
    SetDuration {
        track: TrackId,
        secs: f64,
    },
    /// Records that a track's file is missing or unreadable (or back to Ok).
    SetFileState {
        track: TrackId,
        state: FileState,
    },
    RemoveEntry(EntryId),
    MoveEntry {
        entry: EntryId,
        to: PlaylistId,
        index: usize,
    },
    DuplicateEntry(EntryId),
    /// Feedback 2 spec O22: clears the played mark of every entry of the
    /// playlist except the current entry of a player.
    ResetPlayed(PlaylistId),
    /// R26: the entry repeats until the operator moves on.
    ToggleEntryRepeat(EntryId),
    /// R27: the player stops after the entry, every time it plays.
    ToggleEntryStopAfter(EntryId),
    SetEntryRepeat(EntryId, bool),
    SetEntryStopAfter(EntryId, bool),
    CreatePlaylist {
        name: String,
    },
    /// A new playlist filled with `paths` (playlist file import).
    CreatePlaylistFromPaths {
        name: String,
        paths: Vec<PathBuf>,
    },
    RenamePlaylist {
        playlist: PlaylistId,
        name: String,
    },
    DeletePlaylist(PlaylistId),
    SetPlayerCount(usize),
    /// Replaces the configuration (already validated by the caller); the
    /// player count is kept, use `SetPlayerCount` to change it.
    UpdateConfig(Box<Config>),
    /// Fires a cart, or stops it if it is playing (rules C1–C3, C6).
    FireCart(CartId),
    StopCart(CartId),
    /// Stops every playing cart and the cart cue (C10).
    StopAllCarts,
    /// Pre-listens a cart on the cartwall Cue route, or stops it (C9).
    CueCart(CartId),
    /// Pre-listens a cart, or stops it, whatever it does now.
    SetCartCue(CartId, bool),
    CreateCartPage {
        name: String,
    },
    RenameCartPage {
        page: CartPageId,
        name: String,
    },
    DeleteCartPage(CartPageId),
    /// Changes a page's grid; carts keep their row-major position.
    ResizeCartPage {
        page: CartPageId,
        rows: u16,
        cols: u16,
    },
    SetCart {
        page: CartPageId,
        index: usize,
        edit: CartEdit,
    },
    /// Renames and/or resizes a page as one change: refused whole (C11).
    EditCartPage {
        page: CartPageId,
        name: Option<String>,
        /// Rows and columns.
        grid: Option<(u16, u16)>,
    },
    /// Edits a cart and its file as one change: refused whole (C11); a new
    /// file stops the cart first (C8).
    EditCart {
        page: CartPageId,
        index: usize,
        edit: CartEdit,
        file: CartFileChange,
    },
    /// Gives a cart a file (a new library track). Stops it if playing (C8).
    AssignCartFile {
        page: CartPageId,
        index: usize,
        path: PathBuf,
    },
    /// Gives a cart a track already in the library. Stops it if playing (C8).
    AssignCartTrack {
        page: CartPageId,
        index: usize,
        track: TrackId,
    },
    ClearCartFile {
        page: CartPageId,
        index: usize,
    },
    ImportCartPage(Box<CartPageImport>),
    ShowCartPage(CartPageId),
    SetCartwallOpen(bool),
    /// Places a manual marker (clamped into the cue range), or with `None`
    /// clears it and lets analysis fill it again.
    SetMarker {
        track: TrackId,
        kind: MarkerKind,
        secs: Option<f64>,
    },
    /// Drops every manual marker of a track and analyses it again.
    ResetMarkers {
        track: TrackId,
    },
    /// Binds `chord` to `action` (taking it from any other action), or with
    /// `None` unbinds the action.
    SetShortcut {
        action: ShortcutAction,
        chord: Option<KeyChord>,
    },
    ResetShortcuts,
    /// Resets one Settings section to its defaults (feedback 2 spec O2).
    RestoreDefaults(crate::restore::SettingsSection),
    /// Live settings spec L20: every pending output change now, forced,
    /// even where it briefly interrupts the audio on a device.
    ApplySettingsNow,
}

/// Something the audio engine observed.
#[derive(Debug, Clone, PartialEq)]
pub enum EngineEvent {
    /// A crossfade or segue fade-out finished.
    FadeCompleted { player: PlayerId },
    /// The source of `entry` stopped: a scheduled `StopAt` was reached or a
    /// fade stop completed. Ignored unless `entry` is still the player's current.
    ReachedEnd { player: PlayerId, entry: EntryId },
    /// The engine started `entry` as scheduled by a `StartNextAt` plan. The
    /// model follows the engine even if the next changed in the meantime.
    TransitionStarted { player: PlayerId, entry: EntryId },
    /// A source could not be decoded or read.
    SourceFailed { player: PlayerId, entry: EntryId },
    /// The source prepared for what plays next could not be opened; the
    /// source on air, if any, is not affected.
    PreloadFailed { player: PlayerId, entry: EntryId },
    /// The cue source of `entry` reached its end (or could not start).
    /// Ignored unless `entry` is still the player's cue.
    CueEnded { player: PlayerId, entry: EntryId },
    /// A cart that is not looped reached its cue-out (C5).
    CartEnded { cart: CartId },
    /// A cart's file could not be decoded or read.
    CartFailed { cart: CartId },
    /// The pre-listen of `cart` reached its end (or could not start).
    /// Ignored unless `cart` is still the one pre-listened.
    CartCueEnded { cart: CartId },
    /// `entry` reaches its Main device as DSD, unchanged (spec O25).
    /// `hold_others` is the mix policy in force (`DsdMix::HoldOthers`).
    /// Ignored unless `entry` is the player's current.
    DsdStarted {
        player: PlayerId,
        entry: EntryId,
        hold_others: bool,
    },
    /// The DSD stream of `entry` ended or was switched to PCM.
    DsdEnded { player: PlayerId, entry: EntryId },
    /// Live settings spec L3: the engine bound `holder` to the bus of
    /// `device` (a player added, a route or the audio system applied, the
    /// cartwall's first cart), holding `route`; the device runs with
    /// `running`.
    Placed {
        holder: Holder,
        route: Option<Route>,
        device: OutputDevice,
        running: DeviceSettings,
    },
    /// L3: the engine holds `route` for `holder`, which has no bus (a
    /// player without a usable Cue route, the cartwall before its first
    /// cart).
    Unplaced {
        holder: Holder,
        route: Option<Route>,
    },
    /// L3: a player was removed; its holders are forgotten.
    Gone { holder: Holder },
    /// The audio system the engine runs with (`outputs.backend`) and the
    /// backend it really chose; reported at start and after
    /// `ApplyAudioSystem`.
    AudioSystemInUse {
        configured: Option<String>,
        in_use: String,
    },
    /// L8, L14: the engine ran an `Apply…` action for `target`.
    Applied {
        target: Target,
        wanted: Wanted,
        outcome: Result<(), String>,
    },
}

/// Everything the engine needs to open and position one source.
#[derive(Debug, Clone, PartialEq)]
pub struct SourceRequest {
    pub entry: EntryId,
    pub track: TrackId,
    pub path: PathBuf,
    pub from_secs: f64,
    /// The file's format, once analysed (bit-perfect buses follow its rate).
    pub format: Option<AudioFormat>,
}

/// Transition point meaning "the natural end of the decoded source". Used
/// while a track's duration is still unknown (not yet analysed), so that no
/// transition is ever scheduled at 0 s.
pub const SOURCE_END: f64 = f64::INFINITY;

/// What must happen at the end of the current track, in seconds of that track
/// (`SOURCE_END` when the end is not known yet).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransitionPlan {
    StopAt {
        at_secs: f64,
    },
    /// Start the preloaded next at `at_secs`. When `fade_current_until_secs`
    /// is set, the current source keeps playing and fades out until then (overlap).
    StartNextAt {
        at_secs: f64,
        fade_current_until_secs: Option<f64>,
    },
}

/// Work for the audio engine, produced by the model.
#[derive(Debug, Clone, PartialEq)]
pub enum EngineAction {
    /// Prepare (or with `None`, drop) the source that will play next.
    Preload {
        player: PlayerId,
        request: Option<SourceRequest>,
    },
    /// Start playing now (normally the preloaded source).
    StartCurrent {
        player: PlayerId,
        request: SourceRequest,
    },
    /// Fade the current source out over `fade_ms` while `request` starts now at full level.
    Crossfade {
        player: PlayerId,
        request: SourceRequest,
        fade_ms: u32,
    },
    /// Fade the current source out over `fade_ms`, then stop and report `ReachedEnd`.
    FadeOutAndStop {
        player: PlayerId,
        fade_ms: u32,
    },
    Pause {
        player: PlayerId,
    },
    Resume {
        player: PlayerId,
    },
    StopNow {
        player: PlayerId,
    },
    /// Replace the transition plan of the current source (`None` cancels it).
    Schedule {
        player: PlayerId,
        plan: Option<TransitionPlan>,
    },
    AddPlayer {
        player: PlayerId,
    },
    RemovePlayer {
        player: PlayerId,
    },
    StartCue {
        player: PlayerId,
        request: SourceRequest,
    },
    StopCue {
        player: PlayerId,
    },
    /// Replace the CUE source by one at `secs`; it stays held if the CUE is.
    SeekCue {
        player: PlayerId,
        secs: f64,
    },
    /// Hold or release the CUE source.
    SetCuePaused {
        player: PlayerId,
        paused: bool,
    },
    SetVolume {
        player: PlayerId,
        volume: f32,
    },
    /// Switch the player's DSD stream to PCM now (spec O25: the fader left
    /// unity).
    LeaveDsd {
        player: PlayerId,
    },
    Seek {
        player: PlayerId,
        secs: f64,
    },
    /// Open `request` at its position and hold it paused (crash recovery).
    LoadPaused {
        player: PlayerId,
        request: SourceRequest,
    },
    /// Play a cart on the cartwall Main route.
    StartCart(CartRequest),
    StopCart {
        cart: CartId,
    },
    /// Pre-listen a cart on the cartwall Cue route.
    StartCartCue(CartRequest),
    StopCartCue,
    /// L10: the configuration the engine takes for new buses and new
    /// holders, and the tuning it reads live. It changes no open bus.
    UpdateSettings(Box<Config>),
    /// L17: change the audio system the default output uses. Not forced,
    /// it waits until every bus is quiet (L11).
    ApplyAudioSystem {
        backend: Option<String>,
        force: bool,
    },
    /// L16: move `holder` to `route`. Not forced, it waits until the
    /// holder's own sources are quiet (L6, L11).
    ApplyRoute {
        holder: Holder,
        route: Option<Route>,
        force: bool,
    },
    /// L12: run `device` with `settings`. Not forced, it waits until the
    /// device's bus is quiet (L11).
    ApplyDevice {
        device: OutputDevice,
        settings: DeviceSettings,
        force: bool,
    },
}

/// Everything the engine needs to play one cart.
#[derive(Debug, Clone, PartialEq)]
pub struct CartRequest {
    pub cart: CartId,
    pub track: TrackId,
    pub path: PathBuf,
    /// Cue-in.
    pub from_secs: f64,
    /// Cue-out, or `SOURCE_END` while the length is unknown.
    pub until_secs: f64,
    pub looped: bool,
    /// The file's format, once analysed.
    pub format: Option<AudioFormat>,
}
