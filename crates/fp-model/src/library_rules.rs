//! Rules of the track library that the reducer and the cart rules share:
//! one track per file (LB14), tracks that stay (LB15, LB16) and the ones
//! that leave (LB17c). Pure functions on the state; the model does no file
//! system work, so paths compare as given.

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::error::ModelError;
use crate::ids::{CategoryId, TrackId};
use crate::state::AppState;
use crate::track::{FileState, MarkerSource, Track};

/// The existing track for `path`, or a new uncategorised one (LB14).
pub(crate) fn track_for_path(state: &mut AppState, path: PathBuf) -> TrackId {
    if let Some(id) = state.library.track_at(&path) {
        return id;
    }
    let id = state.ids.track();
    state.library.insert(Track::new(id, path));
    id
}

/// LB17c: removes each track that is uncategorised, missing and used by no
/// entry or cart. Every other track stays in the library (LB15, LB16).
pub(crate) fn release_tracks(state: &mut AppState, tracks: impl IntoIterator<Item = TrackId>) {
    for id in tracks {
        let leaves = state
            .library
            .get(id)
            .is_some_and(|t| t.category.is_none() && t.file_state == FileState::Missing)
            && !state.playlists.references_track(id)
            && !state.cartwall.references_track(id);
        if leaves {
            state.library.remove(id);
        }
    }
}

/// LB16, LB13: puts each file in the library, filed under `category` when
/// given. A new track, or an uncategorised one, joins the category and takes
/// its kind; a track already in a category keeps it.
pub(crate) fn add_to_library(
    state: &mut AppState,
    paths: Vec<PathBuf>,
    category: Option<CategoryId>,
) -> Result<(), ModelError> {
    let target = match category {
        Some(c) => {
            let kind = state
                .categories
                .get(c)
                .ok_or(ModelError::UnknownCategory(c))?
                .kind;
            Some((c, kind))
        }
        None => None,
    };
    for path in paths {
        let id = track_for_path(state, path);
        if let Some((c, kind)) = target
            && let Some(t) = state.library.get_mut(id)
            && t.category.is_none()
        {
            t.category = Some(c);
            t.kind = kind;
        }
    }
    Ok(())
}

/// True when the operator changed the track: a manual marker, a loudness
/// edit or a category.
fn has_operator_edit(t: &Track) -> bool {
    let m = &t.markers;
    [
        m.cue_in,
        m.intro_end,
        m.outro_start,
        m.segue_start,
        m.cue_out,
    ]
    .into_iter()
    .flatten()
    .any(|marker| marker.source == MarkerSource::Manual)
        || t.loudness_edit != Default::default()
        || t.category.is_some()
}

/// Merges tracks that share a path. The survivor is the lowest id among
/// those with an operator edit, else the lowest id; entries and carts are
/// re-pointed at it. Returns how many tracks were merged away.
pub(crate) fn merge_duplicate_paths(state: &mut AppState) -> usize {
    let mut groups: BTreeMap<PathBuf, Vec<TrackId>> = BTreeMap::new();
    for t in state.library.iter() {
        groups.entry(t.path.clone()).or_default().push(t.id);
    }
    let mut replaced: BTreeMap<TrackId, TrackId> = BTreeMap::new();
    for ids in groups.into_values().filter(|ids| ids.len() > 1) {
        let survivor = ids
            .iter()
            .copied()
            .find(|id| state.library.get(*id).is_some_and(has_operator_edit))
            .or_else(|| ids.first().copied());
        let Some(survivor) = survivor else { continue };
        for id in ids.into_iter().filter(|id| *id != survivor) {
            replaced.insert(id, survivor);
        }
    }
    if replaced.is_empty() {
        return 0;
    }
    let lists: Vec<_> = state.playlists.iter().map(|p| p.id).collect();
    for list in lists {
        if let Some(list) = state.playlists.get_mut(list) {
            for entry in &mut list.entries {
                if let Some(to) = replaced.get(&entry.track) {
                    entry.track = *to;
                }
            }
        }
    }
    for cart in state
        .cartwall
        .pages
        .iter_mut()
        .flat_map(|p| p.carts.iter_mut())
    {
        if let Some(to) = cart.track.and_then(|t| replaced.get(&t)) {
            cart.track = Some(*to);
        }
    }
    for id in replaced.keys() {
        state.library.remove(*id);
    }
    // `remove` unindexed only the paths it owned: index the survivors again.
    let survivors: Vec<_> = replaced.values().copied().collect();
    for id in survivors {
        if let Some(path) = state.library.get(id).map(|t| t.path.clone()) {
            state.library.set_path(id, path);
        }
    }
    replaced.len()
}
