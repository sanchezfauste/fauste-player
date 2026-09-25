//! Playlists: ordered lists of entries pointing at library tracks.

use serde::{Deserialize, Serialize};

use crate::error::ModelError;
use crate::ids::{EntryId, PlaylistId, TrackId};
use crate::track::Library;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaylistEntry {
    pub id: EntryId,
    pub track: TrackId,
    pub played: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Playlist {
    pub id: PlaylistId,
    pub name: String,
    pub entries: Vec<PlaylistEntry>,
}

impl Playlist {
    pub fn new(id: PlaylistId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            entries: Vec::new(),
        }
    }

    pub fn position(&self, entry: EntryId) -> Option<usize> {
        self.entries.iter().position(|e| e.id == entry)
    }
}

/// All playlists in display (tab) order.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Playlists {
    lists: Vec<Playlist>,
}

impl Playlists {
    pub fn iter(&self) -> impl Iterator<Item = &Playlist> {
        self.lists.iter()
    }

    pub fn len(&self) -> usize {
        self.lists.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lists.is_empty()
    }

    pub fn first_id(&self) -> Option<PlaylistId> {
        self.lists.first().map(|p| p.id)
    }

    pub fn get(&self, id: PlaylistId) -> Option<&Playlist> {
        self.lists.iter().find(|p| p.id == id)
    }

    pub fn get_mut(&mut self, id: PlaylistId) -> Option<&mut Playlist> {
        self.lists.iter_mut().find(|p| p.id == id)
    }

    pub fn add(&mut self, playlist: Playlist) {
        self.lists.push(playlist);
    }

    pub fn rename(&mut self, id: PlaylistId, name: String) -> Result<(), ModelError> {
        self.get_mut(id)
            .ok_or(ModelError::UnknownPlaylist(id))?
            .name = name;
        Ok(())
    }

    /// Removes a playlist. The last remaining playlist cannot be removed.
    pub fn remove(&mut self, id: PlaylistId) -> Result<Playlist, ModelError> {
        let pos = self
            .lists
            .iter()
            .position(|p| p.id == id)
            .ok_or(ModelError::UnknownPlaylist(id))?;
        if self.lists.len() == 1 {
            return Err(ModelError::LastPlaylist);
        }
        Ok(self.lists.remove(pos))
    }

    pub fn find(&self, entry: EntryId) -> Option<(PlaylistId, usize)> {
        self.lists
            .iter()
            .find_map(|p| p.position(entry).map(|i| (p.id, i)))
    }

    pub fn entry(&self, entry: EntryId) -> Option<&PlaylistEntry> {
        self.lists
            .iter()
            .find_map(|p| p.entries.iter().find(|e| e.id == entry))
    }

    pub fn mark_played(&mut self, entry: EntryId) {
        if let Some(e) = self
            .lists
            .iter_mut()
            .find_map(|p| p.entries.iter_mut().find(|e| e.id == entry))
        {
            e.played = true;
        }
    }

    /// The first entry after `entry`, in the same playlist, whose track is playable.
    pub fn next_playable_after(&self, entry: EntryId, library: &Library) -> Option<EntryId> {
        let (pl, pos) = self.find(entry)?;
        self.get(pl)?
            .entries
            .iter()
            .skip(pos + 1)
            .find(|e| library.is_playable(e.track))
            .map(|e| e.id)
    }

    pub fn first_playable(&self, playlist: PlaylistId, library: &Library) -> Option<EntryId> {
        self.get(playlist)?
            .entries
            .iter()
            .find(|e| library.is_playable(e.track))
            .map(|e| e.id)
    }

    /// Inserts entries at `index` (clamped to the list length).
    pub fn insert(
        &mut self,
        playlist: PlaylistId,
        index: usize,
        entries: Vec<PlaylistEntry>,
    ) -> Result<(), ModelError> {
        let list = self
            .get_mut(playlist)
            .ok_or(ModelError::UnknownPlaylist(playlist))?;
        let at = index.min(list.entries.len());
        list.entries.splice(at..at, entries);
        Ok(())
    }

    pub fn remove_entry(&mut self, entry: EntryId) -> Result<PlaylistEntry, ModelError> {
        let (pl, pos) = self.find(entry).ok_or(ModelError::UnknownEntry(entry))?;
        let list = self.get_mut(pl).ok_or(ModelError::UnknownPlaylist(pl))?;
        Ok(list.entries.remove(pos))
    }

    /// Moves `entry` to drop position `index` of playlist `to`, where `index`
    /// is measured before the entry is taken out of its current place.
    pub fn move_entry(
        &mut self,
        entry: EntryId,
        to: PlaylistId,
        index: usize,
    ) -> Result<(), ModelError> {
        if self.get(to).is_none() {
            return Err(ModelError::UnknownPlaylist(to));
        }
        let (from, pos) = self.find(entry).ok_or(ModelError::UnknownEntry(entry))?;
        let moved = self.remove_entry(entry)?;
        let target = if from == to && pos < index {
            index - 1
        } else {
            index
        };
        self.insert(to, target, vec![moved])
    }

    /// Inserts an unplayed copy of `entry` right after it.
    pub fn duplicate(&mut self, entry: EntryId, new_id: EntryId) -> Result<EntryId, ModelError> {
        let (pl, pos) = self.find(entry).ok_or(ModelError::UnknownEntry(entry))?;
        let original = *self.entry(entry).ok_or(ModelError::UnknownEntry(entry))?;
        let copy = PlaylistEntry {
            id: new_id,
            track: original.track,
            played: false,
        };
        self.insert(pl, pos + 1, vec![copy])?;
        Ok(new_id)
    }

    pub fn references_track(&self, track: TrackId) -> bool {
        self.lists
            .iter()
            .any(|p| p.entries.iter().any(|e| e.track == track))
    }

    pub fn max_raw_id(&self) -> u64 {
        self.lists
            .iter()
            .flat_map(|p| std::iter::once(p.id.0).chain(p.entries.iter().map(|e| e.id.0)))
            .max()
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::track::{FileState, Track};
    use std::path::PathBuf;

    fn setup() -> (Playlists, Library) {
        let mut lib = Library::default();
        for t in 1..=4 {
            lib.insert(Track::new(
                TrackId(t),
                PathBuf::from(format!("/m/{t}.flac")),
            ));
        }
        let mut lists = Playlists::default();
        let mut a = Playlist::new(PlaylistId(100), "A");
        for (e, t) in [(1, 1), (2, 2), (3, 3)] {
            a.entries.push(PlaylistEntry {
                id: EntryId(e),
                track: TrackId(t),
                played: false,
            });
        }
        lists.add(a);
        lists.add(Playlist::new(PlaylistId(200), "B"));
        (lists, lib)
    }

    fn ids(lists: &Playlists, pl: u64) -> Vec<u64> {
        lists
            .get(PlaylistId(pl))
            .unwrap()
            .entries
            .iter()
            .map(|e| e.id.0)
            .collect()
    }

    #[test]
    fn moving_down_in_the_same_list_uses_the_drop_line_before_removal() {
        let (mut lists, _) = setup();
        lists.move_entry(EntryId(1), PlaylistId(100), 2).unwrap();
        assert_eq!(ids(&lists, 100), vec![2, 1, 3]);
    }

    #[test]
    fn moving_to_another_list_clamps_the_index() {
        let (mut lists, _) = setup();
        lists.move_entry(EntryId(2), PlaylistId(200), 99).unwrap();
        assert_eq!(ids(&lists, 100), vec![1, 3]);
        assert_eq!(ids(&lists, 200), vec![2]);
    }

    #[test]
    fn next_playable_skips_missing_and_unreadable_tracks() {
        let (lists, mut lib) = setup();
        lib.get_mut(TrackId(2)).unwrap().file_state = FileState::Missing;
        assert_eq!(
            lists.next_playable_after(EntryId(1), &lib),
            Some(EntryId(3))
        );
        lib.get_mut(TrackId(3)).unwrap().file_state = FileState::Unreadable;
        assert_eq!(lists.next_playable_after(EntryId(1), &lib), None);
        assert_eq!(
            lists.first_playable(PlaylistId(100), &lib),
            Some(EntryId(1))
        );
    }

    #[test]
    fn duplicate_inserts_an_unplayed_copy_right_after() {
        let (mut lists, _) = setup();
        lists.mark_played(EntryId(1));
        let copy = lists.duplicate(EntryId(1), EntryId(9)).unwrap();
        assert_eq!(copy, EntryId(9));
        assert_eq!(ids(&lists, 100), vec![1, 9, 2, 3]);
        assert!(!lists.entry(EntryId(9)).unwrap().played);
        assert_eq!(lists.entry(EntryId(9)).unwrap().track, TrackId(1));
    }

    #[test]
    fn the_last_playlist_cannot_be_removed() {
        let (mut lists, _) = setup();
        lists.remove(PlaylistId(200)).unwrap();
        assert_eq!(lists.remove(PlaylistId(100)), Err(ModelError::LastPlaylist));
        assert_eq!(
            lists.remove(PlaylistId(5)),
            Err(ModelError::UnknownPlaylist(PlaylistId(5)))
        );
    }

    #[test]
    fn unknown_entries_are_reported() {
        let (mut lists, _) = setup();
        assert_eq!(
            lists.remove_entry(EntryId(77)),
            Err(ModelError::UnknownEntry(EntryId(77)))
        );
        assert_eq!(
            lists.move_entry(EntryId(1), PlaylistId(5), 0),
            Err(ModelError::UnknownPlaylist(PlaylistId(5)))
        );
        assert_eq!(ids(&lists, 100), vec![1, 2, 3]);
    }

    #[test]
    fn references_and_max_id() {
        let (lists, _) = setup();
        assert!(lists.references_track(TrackId(3)));
        assert!(!lists.references_track(TrackId(4)));
        assert_eq!(lists.max_raw_id(), 200);
    }
}
