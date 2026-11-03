//! The library CUE (library plan 4, LB32-LB37): the state of the pre-listen
//! the library window runs on its own holder. Runtime only, never saved
//! (rule 10: nothing goes on air by itself after a start).

use crate::ids::TrackId;
use crate::player::Transport;
use crate::state::AppState;

/// The library window's pre-listen.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LibraryCue {
    /// The bar's track (LB34).
    pub track: Option<TrackId>,
    /// Stopped, Playing or Paused; a held preview carries audio (Q4).
    pub transport: Transport,
    /// Where Play starts while stopped, in seconds.
    pub start_at: Option<f64>,
}

/// LB36: the engine reports that the preview of `track` ended (the end of
/// its file, a file that cannot play, its output gone): the bar stops and
/// keeps its track. Ignored unless `track` is still the bar's.
pub(crate) fn ended(state: &mut AppState, track: TrackId) {
    let cue = &mut state.library_cue;
    if cue.track == Some(track) {
        cue.transport = Transport::Stopped;
        cue.start_at = None;
    }
}
