//! Per-player state.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::columns::TableColumn;
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

/// The track table's column widths as fractions of its width, keyed by
/// column and summing to 1 over the columns that were shown when they were
/// stored; `None` is the default layout (feedback 2 spec O24). A column
/// missing from the map takes its default width. Widths saved by earlier
/// versions as four fractions (`#`, Title, Artist, Duration) are converted
/// when they are four good numbers; pixel widths and anything else give the
/// default layout.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ColumnWidths {
    #[serde(deserialize_with = "lenient_fractions")]
    pub fractions: Option<BTreeMap<TableColumn, f32>>,
}

/// The four columns the old fraction array described, in its order.
const LEGACY_COLUMNS: [TableColumn; 4] = [
    TableColumn::Number,
    TableColumn::Title,
    TableColumn::Artist,
    TableColumn::Duration,
];

fn lenient_fractions<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<Option<BTreeMap<TableColumn, f32>>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Raw {
        Legacy(Vec<f32>),
        Keyed(BTreeMap<String, f32>),
        Other(serde::de::IgnoredAny),
    }
    Ok(match Option::<Raw>::deserialize(d)? {
        Some(Raw::Legacy(values)) if values.len() == LEGACY_COLUMNS.len() => {
            Some(LEGACY_COLUMNS.into_iter().zip(values).collect())
        }
        Some(Raw::Keyed(map)) => Some(
            map.into_iter()
                .filter_map(|(name, f)| TableColumn::from_name(&name).map(|c| (c, f)))
                .collect(),
        ),
        _ => None,
    })
}

impl ColumnWidths {
    /// Widths from `(column, fraction)` pairs, normalised.
    pub fn keyed(pairs: impl IntoIterator<Item = (TableColumn, f32)>) -> Self {
        Self {
            fractions: Some(pairs.into_iter().collect()),
        }
        .normalized()
    }

    /// The stored fraction of `column`, if any.
    pub fn fraction(&self, column: TableColumn) -> Option<f32> {
        self.fractions.as_ref()?.get(&column).copied()
    }

    /// Fractions scaled to sum to 1; broken ones (not finite, negative, none
    /// left, all zero) give the default layout.
    pub fn normalized(self) -> Self {
        let fractions = self.fractions.and_then(|f| {
            let valid = f.values().all(|x| x.is_finite() && *x >= 0.0);
            let sum: f32 = f.values().sum();
            (valid && sum > 0.0).then(|| f.into_iter().map(|(c, x)| (c, x / sum)).collect())
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
    /// The DSD stream the engine reported for the entry on air (spec O25);
    /// runtime only, never saved.
    pub dsd: Option<crate::dsd::DsdOnAir>,
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
            dsd: None,
        }
    }
}
