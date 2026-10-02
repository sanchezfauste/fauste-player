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
    /// The CUE window's Pause is on: the source is held on the Cue bus.
    pub paused: bool,
}

/// The track table's column widths as fractions of its width (`#`, Title,
/// Artist, Duration), summing to 1; `None` is the default layout
/// (feedback spec §2.3). Pixel widths saved by earlier versions are ignored.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ColumnWidths {
    pub fractions: Option<[f32; 4]>,
}

impl ColumnWidths {
    /// Fractions scaled to sum to 1; broken ones (not finite, negative, all
    /// zero) give the default layout.
    pub fn normalized(self) -> Self {
        let fractions = self.fractions.and_then(|f| {
            let valid = f.iter().all(|x| x.is_finite() && *x >= 0.0);
            let sum: f32 = f.iter().sum();
            (valid && sum > 0.0).then(|| f.map(|x| x / sum))
        });
        Self { fractions }
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
    /// Entries this player left, oldest first (R25); Previous pops from the
    /// end.
    pub history: Vec<EntryId>,
    /// Entry the engine was last asked to preload.
    /// Entry and start position the engine was last asked to preload (the
    /// position changes when analysis finds the real cue-in).
    pub(crate) preloaded: Option<(EntryId, f64)>,
    /// Transition plan the engine was last given.
    pub(crate) scheduled: Option<TransitionPlan>,
    /// A fade stop is running: no transition may start.
    pub(crate) fade_stop_pending: bool,
}

impl PlayerState {
    /// True while a fade stop runs (as opposed to a crossfade into the next).
    pub fn fade_stopping(&self) -> bool {
        self.fade_stop_pending
    }

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
            history: Vec::new(),
            preloaded: None,
            scheduled: None,
            fade_stop_pending: false,
        }
    }
}
