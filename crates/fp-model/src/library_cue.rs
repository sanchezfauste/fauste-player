//! The library CUE (library plan 4, LB32-LB37): the state of the pre-listen
//! the library window runs on its own holder. Runtime only, never saved
//! (rule 10: nothing goes on air by itself after a start).

use crate::ids::TrackId;
use crate::player::Transport;

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
