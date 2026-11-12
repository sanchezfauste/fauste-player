//! The schedule (automation spec AU5–AU14): blocks of items at a time of
//! day, day templates, the week plan, and the changes made for one date.

use serde::{Deserialize, Serialize};

use crate::error::ModelError;
use crate::ids::{BlockId, CategoryId, RotationId, TrackId};
use crate::local_time::TimeOfDay;
use crate::state::AppState;
use crate::track::TrackKind;

/// One thing in a block (AU6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Item {
    /// A track of the library, any kind.
    Track {
        track: TrackId,
        /// Marked time-critical by the operator (AU7).
        #[serde(default)]
        time_critical: bool,
    },
    /// One track of a category, chosen each time the item airs.
    Category {
        category: CategoryId,
        #[serde(default)]
        time_critical: bool,
    },
    /// From here to the end of the block, the music filler plays this
    /// rotation. It is always the block's last item.
    Rotation { rotation: RotationId },
}

impl Item {
    /// AU7: an Hourly ID track, a category of Hourly ID kind, or an item the
    /// operator marked. An unknown track or category is not.
    pub fn is_time_critical(&self, state: &AppState) -> bool {
        match self {
            Item::Track {
                track,
                time_critical,
            } => {
                *time_critical
                    || state
                        .library
                        .get(*track)
                        .is_some_and(|t| t.kind == TrackKind::HourlyId)
            }
            Item::Category {
                category,
                time_critical,
            } => {
                *time_critical
                    || state
                        .categories
                        .get(*category)
                        .is_some_and(|c| c.kind == TrackKind::HourlyId)
            }
            Item::Rotation { .. } => false,
        }
    }
}

/// A group of items with a start time (AU5). It lasts until the next
/// block's start.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub id: BlockId,
    pub start: TimeOfDay,
    #[serde(default)]
    pub items: Vec<Item>,
}

/// AU6: a rotation item, if any, is the last item.
pub fn check_items(items: &[Item]) -> Result<(), ModelError> {
    let rotation_at = items
        .iter()
        .position(|i| matches!(i, Item::Rotation { .. }));
    match rotation_at {
        Some(at) if at + 1 != items.len() => Err(ModelError::RotationNotLast),
        _ => Ok(()),
    }
}
