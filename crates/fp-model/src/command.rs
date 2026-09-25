//! The vocabulary between the UI, the model and the audio engine.

use std::path::PathBuf;

use crate::config::Config;
use crate::ids::{EntryId, PlayerId, PlaylistId, TrackId};
use crate::player::{ColumnWidths, PlayMode};

/// A user intent, sent by the UI.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Play(PlayerId),
    Pause(PlayerId),
    Stop(PlayerId),
    FadeStop(PlayerId),
    SetNext(PlayerId, EntryId),
    InsertPaths {
        playlist: PlaylistId,
        index: usize,
        paths: Vec<PathBuf>,
    },
    SetMode(PlayerId, PlayMode),
    ToggleStopAfterCurrent(PlayerId),
    ToggleCue(PlayerId),
    CueEntry(PlayerId, EntryId),
    SetVolume(PlayerId, f32),
    Seek(PlayerId, f64),
    ShowPlaylist(PlayerId, PlaylistId),
    SetColumnWidths(PlayerId, ColumnWidths),
    RemoveEntry(EntryId),
    MoveEntry {
        entry: EntryId,
        to: PlaylistId,
        index: usize,
    },
    DuplicateEntry(EntryId),
    CreatePlaylist {
        name: String,
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
}

/// Something the audio engine observed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineEvent {
    /// A crossfade or segue fade-out finished.
    FadeCompleted { player: PlayerId },
    /// The current source stopped: a scheduled `StopAt` was reached or a fade stop completed.
    ReachedEnd { player: PlayerId },
    /// The engine started the next source as scheduled by a `StartNextAt` plan.
    TransitionStarted { player: PlayerId },
    /// A source could not be decoded or read.
    SourceFailed { player: PlayerId, entry: EntryId },
    /// The cue source reached its end.
    CueEnded { player: PlayerId },
}

/// Everything the engine needs to open and position one source.
#[derive(Debug, Clone, PartialEq)]
pub struct SourceRequest {
    pub entry: EntryId,
    pub track: TrackId,
    pub path: PathBuf,
    pub from_secs: f64,
}

/// What must happen at the end of the current track, in seconds of that track.
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
    SetVolume {
        player: PlayerId,
        volume: f32,
    },
    Seek {
        player: PlayerId,
        secs: f64,
    },
}
