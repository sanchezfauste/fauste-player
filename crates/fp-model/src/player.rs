//! Per-player state.

use serde::{Deserialize, Serialize};

use crate::command::TransitionPlan;
use crate::ids::{EntryId, PlayerId, PlaylistId};

/// `Single` stops after every track; `Continuous` chains tracks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PlayMode {
    Single,
    #[default]
    Continuous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Transport {
    #[default]
    Stopped,
    Playing,
    Paused,
}

/// Pre-listen (cue) of one entry on the player's Cue bus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CueState {
    pub entry: EntryId,
}

/// User-resized track-table column widths, in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ColumnWidths {
    pub number: Option<f32>,
    pub title: Option<f32>,
    pub duration: f32,
}

impl Default for ColumnWidths {
    fn default() -> Self {
        Self {
            number: None,
            title: None,
            duration: 52.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerState {
    pub id: PlayerId,
    /// Playlist shown in this player's tab strip.
    pub playlist: PlaylistId,
    pub current: Option<EntryId>,
    pub next: Option<EntryId>,
    /// True when the user chose `next` (double-click); false when it was
    /// derived from the playlist order and should follow playlist edits.
    pub next_explicit: bool,
    pub transport: Transport,
    pub fading: bool,
    pub mode: PlayMode,
    pub stop_after_current: bool,
    pub cue: Option<CueState>,
    /// Linear gain 0.0–1.0.
    pub volume: f32,
    pub columns: ColumnWidths,
    /// Entry the engine was last asked to preload.
    pub(crate) preloaded: Option<EntryId>,
    /// Transition plan the engine was last given.
    pub(crate) scheduled: Option<TransitionPlan>,
    /// A fade stop is running: no transition may start.
    pub(crate) fade_stop_pending: bool,
}

impl PlayerState {
    pub fn new(id: PlayerId, playlist: PlaylistId, mode: PlayMode) -> Self {
        Self {
            id,
            playlist,
            current: None,
            next: None,
            next_explicit: false,
            transport: Transport::Stopped,
            fading: false,
            mode,
            stop_after_current: false,
            cue: None,
            volume: 1.0,
            columns: ColumnWidths::default(),
            preloaded: None,
            scheduled: None,
            fade_stop_pending: false,
        }
    }
}
