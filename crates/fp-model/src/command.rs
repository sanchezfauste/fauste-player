//! The vocabulary between the UI, the model and the audio engine.

use std::path::PathBuf;

use crate::ids::{EntryId, PlayerId, PlaylistId, TrackId};

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
}

/// Something the audio engine observed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineEvent {
    /// A crossfade or segue fade-out finished.
    FadeCompleted { player: PlayerId },
    /// The current source stopped: a scheduled `StopAt` was reached or a fade stop completed.
    ReachedEnd { player: PlayerId },
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
}
