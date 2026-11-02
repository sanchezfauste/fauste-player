//! Rules of the track library that the reducer and the cart rules share:
//! one track per file (LB14), tracks that stay (LB15, LB16) and the ones
//! that leave (LB17c). Pure functions on the state; the model does no file
//! system work, so paths compare as given.

use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;

use crate::command::EngineAction;
use crate::error::ModelError;
use crate::ids::{CartId, CategoryId, EntryId, TrackId};
use crate::state::AppState;
use crate::track::{FileState, MarkerSource, Track, TrackKind};

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
/// entry or cart, and reports it (`TrackLeftLibrary`) so it is logged. Every
/// other track stays in the library (LB15, LB16).
pub(crate) fn release_tracks(
    state: &mut AppState,
    tracks: impl IntoIterator<Item = TrackId>,
    out: &mut Vec<EngineAction>,
) {
    for id in tracks {
        let leaves = state
            .library
            .get(id)
            .is_some_and(|t| t.category.is_none() && t.file_state == FileState::Missing)
            && !state.playlists.references_track(id)
            && !state.cartwall.references_track(id);
        if leaves && let Some(track) = state.library.remove(id) {
            out.push(EngineAction::TrackLeftLibrary {
                track: id,
                path: track.path,
            });
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

/// What a removal holds on air (LB17b).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OnAirHolds {
    pub entries: Vec<EntryId>,
    pub carts: Vec<CartId>,
}

impl OnAirHolds {
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty() && self.carts.is_empty()
    }
}

/// What removing tracks takes with it (LB17), for the confirmation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RemovalImpact {
    pub tracks: usize,
    /// Playlist entries that go.
    pub entries: usize,
    /// Carts that lose their file.
    pub carts: usize,
    pub on_air: OnAirHolds,
}

/// Counts what removing `tracks` affects. Unknown tracks are ignored.
pub fn removal_impact(state: &AppState, tracks: &[TrackId]) -> RemovalImpact {
    let set: HashSet<TrackId> = tracks
        .iter()
        .copied()
        .filter(|t| state.library.get(*t).is_some())
        .collect();
    let mut impact = RemovalImpact {
        tracks: set.len(),
        ..RemovalImpact::default()
    };
    for entry in state.playlists.iter().flat_map(|p| p.entries.iter()) {
        if set.contains(&entry.track) {
            impact.entries += 1;
            let cued = state
                .players
                .iter()
                .any(|p| p.cue.is_some_and(|c| c.entry == entry.id));
            if state.is_on_air(entry.id) || cued {
                impact.on_air.entries.push(entry.id);
            }
        }
    }
    for cart in state
        .cartwall
        .pages
        .iter()
        .flat_map(|p| p.carts.iter())
        .filter(|c| c.track.is_some_and(|t| set.contains(&t)))
    {
        impact.carts += 1;
        if state.cartwall.is_playing(cart.id) || state.cartwall.cue == Some(cart.id) {
            impact.on_air.carts.push(cart.id);
        }
    }
    impact
}

/// LB11: the subcategories and the impact of deleting `category` with its
/// subtree and their tracks; `None` when the category is unknown.
pub fn category_deletion(state: &AppState, category: CategoryId) -> Option<(usize, RemovalImpact)> {
    state.categories.get(category)?;
    let subtree = state.categories.subtree(category);
    let tracks = tracks_in(state, &subtree);
    Some((
        subtree.len().saturating_sub(1),
        removal_impact(state, &tracks),
    ))
}

fn tracks_in(state: &AppState, categories: &[CategoryId]) -> Vec<TrackId> {
    let categories: HashSet<CategoryId> = categories.iter().copied().collect();
    state
        .library
        .iter()
        .filter(|t| t.category.is_some_and(|c| categories.contains(&c)))
        .map(|t| t.id)
        .collect()
}

/// LB12, LB13: files the tracks under `category` and gives them its kind.
/// Taken out of every category (`None`), a missing, unused track leaves
/// the library (LB17c).
pub(crate) fn move_tracks(
    state: &mut AppState,
    tracks: Vec<TrackId>,
    category: Option<CategoryId>,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    let kind = match category {
        Some(c) => Some(
            state
                .categories
                .get(c)
                .ok_or(ModelError::UnknownCategory(c))?
                .kind,
        ),
        None => None,
    };
    for id in &tracks {
        if let Some(t) = state.library.get_mut(*id) {
            t.category = category;
            if let Some(kind) = kind {
                t.kind = kind;
            }
        }
    }
    if category.is_none() {
        release_tracks(state, tracks, out);
    }
    Ok(())
}

/// LB13: sets the kind of each track.
pub(crate) fn set_tracks_kind(
    state: &mut AppState,
    tracks: Vec<TrackId>,
    kind: TrackKind,
) -> Result<(), ModelError> {
    for id in tracks {
        if let Some(t) = state.library.get_mut(id) {
            t.kind = kind;
        }
    }
    Ok(())
}

/// LB17, LB17b: removes the tracks with their entries, empties their carts,
/// all or nothing.
pub(crate) fn remove_tracks(
    state: &mut AppState,
    tracks: &[TrackId],
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    let impact = removal_impact(state, tracks);
    if !impact.on_air.is_empty() {
        return Err(ModelError::LibraryOnAir(impact.on_air));
    }
    let set: HashSet<TrackId> = tracks.iter().copied().collect();
    let entries: HashSet<EntryId> = state
        .playlists
        .iter()
        .flat_map(|p| p.entries.iter())
        .filter(|e| set.contains(&e.track))
        .map(|e| e.id)
        .collect();
    // The tracks go below: no implicit release (LB17c).
    crate::reducer::take_out_entries(state, &entries, out);
    crate::cart_rules::empty_carts_of(state, &set, out);
    for id in set {
        state.library.remove(id);
    }
    crate::reducer::refresh_next(state);
    Ok(())
}

/// LB11: removes the category, its subcategories and their tracks.
pub(crate) fn delete_category(
    state: &mut AppState,
    category: CategoryId,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    if state.categories.get(category).is_none() {
        return Err(ModelError::UnknownCategory(category));
    }
    let subtree = state.categories.subtree(category);
    let tracks = tracks_in(state, &subtree);
    remove_tracks(state, &tracks, out)?;
    state.categories.remove_all(&subtree);
    Ok(())
}
