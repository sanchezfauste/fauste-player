//! Opaque identifiers. Nothing in the model indexes a fixed-size array by
//! player or playlist number; everything is looked up by id.

use serde::{Deserialize, Serialize};

macro_rules! id_type {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub u64);
    };
}

id_type!(
    /// Identifies a player (one playout column with its own transport).
    PlayerId
);
id_type!(
    /// Identifies a playlist.
    PlaylistId
);
id_type!(
    /// Identifies one entry of a playlist. The same track can appear in
    /// several entries, and "played" is tracked per entry.
    EntryId
);
id_type!(
    /// Identifies a track (one audio file and its metadata).
    TrackId
);

id_type!(
    /// Identifies a cart page (one tab of the cartwall).
    CartPageId
);
id_type!(
    /// Identifies a cart (one button of the cartwall), stable across edits.
    CartId
);

/// Monotonic id generator shared by every id kind, so ids never collide
/// even across kinds. It is persisted with the playlists.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdGen {
    last: u64,
}

impl IdGen {
    /// Returns a fresh raw id.
    pub fn next_raw(&mut self) -> u64 {
        self.last += 1;
        self.last
    }

    /// Makes sure future ids are greater than `raw` (used after loading data).
    pub fn observe(&mut self, raw: u64) {
        self.last = self.last.max(raw);
    }

    pub fn player(&mut self) -> PlayerId {
        PlayerId(self.next_raw())
    }

    pub fn playlist(&mut self) -> PlaylistId {
        PlaylistId(self.next_raw())
    }

    pub fn entry(&mut self) -> EntryId {
        EntryId(self.next_raw())
    }

    pub fn track(&mut self) -> TrackId {
        TrackId(self.next_raw())
    }

    pub fn cart_page(&mut self) -> CartPageId {
        CartPageId(self.next_raw())
    }

    pub fn cart(&mut self) -> CartId {
        CartId(self.next_raw())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_and_increasing_across_kinds() {
        let mut ids = IdGen::default();
        let a = ids.entry();
        let b = ids.track();
        let c = ids.player();
        assert!(a.0 < b.0 && b.0 < c.0);
    }

    #[test]
    fn observe_moves_the_generator_past_existing_ids() {
        let mut ids = IdGen::default();
        ids.observe(41);
        ids.observe(7);
        assert_eq!(ids.player(), PlayerId(42));
    }

    #[test]
    fn ids_serialize_as_plain_numbers() {
        assert_eq!(serde_json::to_string(&EntryId(7)).unwrap(), "7");
        let back: EntryId = serde_json::from_str("7").unwrap();
        assert_eq!(back, EntryId(7));
    }
}
