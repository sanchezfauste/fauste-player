//! What the analysis is doing (main spec §6.1), published by the services
//! thread for the interface: the status bar, the row icons, the row menu
//! and Settings → Analysis. The UI only loads the snapshot; it never waits
//! on the services thread.

use std::collections::HashSet;
use std::sync::Arc;

use arc_swap::ArcSwap;
use fp_model::TrackId;

/// The progress of a pass over the whole library (*Re-analyse all*, or a
/// change of the analysis settings).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PassProgress {
    /// Tracks of the pass that have an answer.
    pub done: usize,
    /// Tracks in the pass.
    pub total: usize,
}

/// One snapshot of the analysis activity. Jobs that only fetch a cart's
/// peaks are left out.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AnalysisActivity {
    /// Tracks waiting for their analysis.
    pub queued: HashSet<TrackId>,
    /// Tracks being analysed now, sorted.
    pub running: Vec<TrackId>,
    /// Loudness-only jobs queued or running (loudness normalization N6b).
    pub loudness: usize,
    /// The library pass, while one runs.
    pub pass: Option<PassProgress>,
}

/// Where a track is in the analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackActivity {
    Queued,
    Running,
}

impl AnalysisActivity {
    /// Tracks queued or running.
    pub fn tracks(&self) -> usize {
        self.queued.len() + self.running.len()
    }

    /// Whether anything is analysed or measured.
    pub fn is_busy(&self) -> bool {
        self.tracks() > 0 || self.loudness > 0
    }

    /// Where `track` is in the analysis, if anywhere.
    pub fn of(&self, track: TrackId) -> Option<TrackActivity> {
        if self.running.contains(&track) {
            Some(TrackActivity::Running)
        } else if self.queued.contains(&track) {
            Some(TrackActivity::Queued)
        } else {
            None
        }
    }
}

/// The shared snapshot: the services thread stores, the UI loads.
pub type ActivityCell = Arc<ArcSwap<AnalysisActivity>>;

/// A cell holding an idle snapshot.
pub fn idle_cell() -> ActivityCell {
    Arc::new(ArcSwap::from_pointee(AnalysisActivity::default()))
}
