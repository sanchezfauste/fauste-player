//! The vocabulary between the UI, the model and the audio engine.

use std::path::PathBuf;

use crate::cartwall::{CartEdit, CartFileChange, CartPageImport};
use crate::config::{Config, OutputDevice, Route};
use crate::ids::{CartId, CartPageId, EntryId, PlayerId, PlaylistId, TrackId};
use crate::layout::{Dock, Panel};
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
    /// Library plan 4 LB34: the library table's selected row changed. A
    /// running preview switches to the new track; a stopped one only
    /// follows the bar. `None` leaves a running preview alone.
    SelectLibraryCue(Option<TrackId>),
    /// LB33: plays the bar's track from its cue-in (or where a seek set),
    /// or releases a held preview. Does nothing while it plays.
    PlayLibraryCue,
    /// LB33: holds or releases the library preview. Idempotent.
    SetLibraryCuePaused(bool),
    /// LB33: ends the library preview. Does nothing while stopped.
    StopLibraryCue,
    /// LB33: moves the running preview to `secs` (clamped to the file); while
    /// stopped it sets where Play starts. A broken value does nothing.
    SeekLibraryCue(f64),
    /// Spec O12 ("Set as next"): the cued entry becomes the player's
    /// explicit next and the CUE keeps running. Without a CUE it does
    /// nothing.
    CueToNext(PlayerId),
    /// On-air pre-listen spec CA4: starts `start` even on a sounding
    /// on-air output; the pre-listen is confirmed (CA5 does not stop it).
    /// On a pre-listen already running for that entry or cart it only
    /// confirms it. Answers a confirmation request for the same holder.
    ForceCue(CueStart),
    /// CA3: the operator cancelled the confirmation request `id`; a newer
    /// request stays.
    CancelCueOnAir(u64),
    SetVolume(PlayerId, f32),
    Seek(PlayerId, f64),
    ShowPlaylist(PlayerId, PlaylistId),
    SetColumnWidths(PlayerId, ColumnWidths),
    /// Resizable layout spec RL4: the shares of two neighbouring shown
    /// players after a splitter drag (`right` is the next shown player
    /// after `left`). Each share must be finite and above 0; the model
    /// scales the pair to keep its sum, so no other player moves.
    SetPlayerShares {
        left: (PlayerId, f32),
        right: (PlayerId, f32),
    },
    /// RL5: every player gets the same share.
    EqualPlayerWidths,
    /// RL6: moves the cartwall. A new dock resets its size: fit the content
    /// at the bottom, the default side share on a side. The same dock
    /// changes nothing.
    SetCartwallDock(Dock),
    /// RL6: the cartwall's share of the middle area, clamped to
    /// `MIN_CARTWALL_SHARE ..= MAX_CARTWALL_SHARE`; `None` fits the content
    /// at the bottom and is the default share on a side.
    SetCartwallSize(Option<f32>),
    /// RL7: hides or shows a panel. Never touches playback.
    SetPanelHidden {
        panel: Panel,
        hidden: bool,
    },
    /// RL9: while locked, every other layout command is refused.
    SetLayoutLocked(bool),
    /// Phone layout spec PL12: faders and waveforms ignore touches while
    /// set. Not a layout command: the layout lock does not refuse it.
    SetTouchLocked(bool),
    /// Feedback 2 spec O12: the CUE windows' content size in points, width
    /// and height, kept with the layout; one that is not finite and above
    /// zero changes nothing. Not a layout command: the layout lock does not
    /// refuse it.
    SetCueWindowSize([f32; 2]),
    /// Stores the library window's state (placement, selection, columns),
    /// repaired against the category tree. Not a layout command.
    SetLibraryView(Box<crate::library_view::LibraryView>),
    /// RL10: back to the default layout (unlocked only).
    ResetLayout,
    /// Stores an analysis result for a track (ignored if the track is gone).
    ApplyAnalysis {
        track: TrackId,
        analysis: Box<TrackAnalysis>,
    },
    /// Loudness normalization spec N6b: the loudness-only pass measured a
    /// track (`None`: nothing could be measured; it is not tried again
    /// until `version` changes). Ignored if the track is gone.
    ApplyLoudness {
        track: TrackId,
        loudness: Option<crate::loudness::LoudnessMeasurement>,
        version: u32,
    },
    /// Feedback 2 spec O23: the file's tags as read after the tag-only pass
    /// or written by the tag editor (ignored if the track is gone).
    ApplyTags {
        track: TrackId,
        tags: Box<TrackTags>,
    },
    /// The bit rate the bit-rate-only pass read from the file, in bit/s
    /// (`None`: it reports none); ignored if the track is gone.
    ApplyBitrate {
        track: TrackId,
        bitrate: Option<u32>,
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
    /// Rule 29: a new empty playlist, shown in `player` at once. An unknown
    /// player is refused and nothing is created.
    CreatePlaylistIn {
        name: String,
        player: PlayerId,
    },
    /// A new playlist filled with `paths` (playlist file import).
    CreatePlaylistFromPaths {
        name: String,
        paths: Vec<PathBuf>,
    },
    /// Puts files in the library, filed under `category` when given: a new
    /// track takes the category and its kind, an uncategorised one joins it,
    /// one already in a category keeps it (LB16).
    AddToLibrary {
        paths: Vec<PathBuf>,
        category: Option<crate::ids::CategoryId>,
    },
    /// LB12, LB13: files the tracks under `category` (`None`: uncategorised);
    /// they take the category's kind. Unknown tracks are skipped.
    MoveTracksToCategory {
        tracks: Vec<crate::ids::TrackId>,
        category: Option<crate::ids::CategoryId>,
    },
    /// LB13: the kind of each track, whatever its category.
    SetTracksKind {
        tracks: Vec<crate::ids::TrackId>,
        kind: crate::track::TrackKind,
    },
    /// LB17: removes the tracks from the library with their entries and
    /// their carts' files. Refused whole while any of them is on air (LB17b).
    RemoveTracks(Vec<crate::ids::TrackId>),
    /// LB21: removes `missing_tracks(category)` as found when it is applied.
    RemoveMissingTracks(crate::ids::CategoryId),
    /// LB11: deletes a category, its subcategories and every track in them
    /// (LB17), refused whole while any of those is on air (LB17b).
    DeleteCategory(crate::ids::CategoryId),
    /// Track categories (track library spec LB9, LB10).
    CreateCategory {
        parent: Option<crate::ids::CategoryId>,
        name: String,
        color: Option<crate::cartwall::CartColor>,
        kind: crate::track::TrackKind,
    },
    RenameCategory {
        category: crate::ids::CategoryId,
        name: String,
    },
    SetCategoryColor {
        category: crate::ids::CategoryId,
        color: Option<crate::cartwall::CartColor>,
    },
    /// Only the kind new tracks take; tracks already filed keep theirs.
    SetCategoryKind {
        category: crate::ids::CategoryId,
        kind: crate::track::TrackKind,
    },
    /// LB19: the folder `category` watches (`None`: none). Refused when
    /// another category watches the same folder.
    SetWatchedFolder {
        category: crate::ids::CategoryId,
        folder: Option<std::path::PathBuf>,
    },
    /// Moves a category with its subtree; `index` is among the new
    /// siblings and is clamped.
    MoveCategory {
        category: crate::ids::CategoryId,
        parent: Option<crate::ids::CategoryId>,
        index: usize,
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
    /// External editor spec EE5: the operator has seen the markers review
    /// notice; clears `markers_review` on each track (unknown ids ignored).
    MarkReviewed {
        tracks: Vec<TrackId>,
    },
    /// External editor spec EE5: records the stamp of a track that has none
    /// (ignored when it already has one or the track is gone).
    SetFileStamp {
        track: TrackId,
        stamp: crate::track::FileStamp,
    },
    /// Loudness normalization spec N10: excludes a track from
    /// normalization or trims its gain. Allowed while it plays: it changes
    /// what loads afterwards.
    SetTrackLoudness {
        track: TrackId,
        excluded: bool,
        trim_db: f32,
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

/// What a pre-listen plays (on-air pre-listen spec CA3, CA4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CueStart {
    /// `entry` on `player`'s Cue.
    Entry { player: PlayerId, entry: EntryId },
    /// A cart on the cartwall's Cue.
    Cart(CartId),
    /// A library track on the library CUE.
    Library(TrackId),
}

impl CueStart {
    /// The Cue holder it plays on.
    pub fn holder(self) -> Holder {
        match self {
            Self::Entry { player, .. } => Holder::PlayerCue(player),
            Self::Cart(_) => Holder::CartwallCue,
            Self::Library(_) => Holder::LibraryCue,
        }
    }
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
    /// `normalization` is the one the started source was opened with
    /// (loudness normalization N11): a re-preload that raced the transition
    /// may have asked for another.
    TransitionStarted {
        player: PlayerId,
        entry: EntryId,
        normalization: crate::loudness::Normalization,
    },
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
    /// The library CUE of `track` reached the end of its file (or could not
    /// start, or its output went away). Ignored unless `track` is still the
    /// one previewed (library plan 4, LB36).
    LibraryCueEnded { track: TrackId },
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
    /// Bit-perfect rate and fader spec BR1: `device`'s stream runs at
    /// `rate`, and it refused `refused` (ascending; B3.4). Reported when its
    /// bus opens and whenever either changes.
    StreamRate {
        device: OutputDevice,
        rate: u32,
        refused: Vec<u32>,
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
    /// The fixed gain of this source (loudness normalization spec N9),
    /// decided for the holder that plays it.
    pub normalization: crate::loudness::Normalization,
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
    /// is set, the current source keeps playing and fades out until then
    /// (overlap). With `rate_cut`, a hard transition on a bit-perfect
    /// device that reopens at that rate between the two tracks (bit-perfect
    /// rate and fader spec BR4, BR6); `None` for every other plan.
    StartNextAt {
        at_secs: f64,
        fade_current_until_secs: Option<f64>,
        rate_cut: Option<u32>,
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
    /// Pre-listen a track on the library CUE route (library plan 4, LB32);
    /// `paused` opens it held, silent until released.
    StartLibraryCue {
        request: LibraryCueRequest,
        paused: bool,
    },
    StopLibraryCue,
    /// Replace the library CUE's source by one at `secs` (LB33).
    SeekLibraryCue {
        secs: f64,
    },
    /// Hold or release the library CUE (LB33).
    SetLibraryCuePaused {
        paused: bool,
    },
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
    /// LB17c: `track` left the library by itself (its file is missing, it
    /// has no category and nothing uses it). The engine only logs it; an
    /// explicit removal (LB11, LB17) is not reported.
    TrackLeftLibrary {
        track: TrackId,
        path: PathBuf,
    },
}

/// Everything the engine needs to pre-listen one track in the library
/// (library plan 4, LB32): it plays from `from_secs` to the end of its file.
#[derive(Debug, Clone, PartialEq)]
pub struct LibraryCueRequest {
    pub track: TrackId,
    pub path: PathBuf,
    pub from_secs: f64,
    /// The file's format, once analysed.
    pub format: Option<AudioFormat>,
    /// The fixed gain of this preview (loudness normalization spec N9).
    pub normalization: crate::loudness::Normalization,
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
    /// The fixed gain of this cart (loudness normalization spec N9),
    /// decided for the holder that plays it.
    pub normalization: crate::loudness::Normalization,
}
