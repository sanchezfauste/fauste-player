//! Per-player state.

use serde::{Deserialize, Serialize};

/// `Single` stops after every track; `Continuous` chains tracks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PlayMode {
    Single,
    #[default]
    Continuous,
}
