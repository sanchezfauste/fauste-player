#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Random sequences of playlist edits must behave exactly like a trivially
//! correct reference implementation built on plain vectors, and must never
//! lose or duplicate entry ids.

use fp_model::{EntryId, Playlist, PlaylistEntry, PlaylistId, Playlists, TrackId};
use proptest::prelude::*;

#[derive(Debug, Clone)]
enum Op {
    Insert {
        list: usize,
        index: usize,
        track: u64,
    },
    Remove {
        pick: usize,
    },
    Move {
        pick: usize,
        list: usize,
        index: usize,
    },
    Duplicate {
        pick: usize,
    },
}

fn op() -> impl Strategy<Value = Op> {
    prop_oneof![
        (0..3usize, 0..20usize, 0..50u64).prop_map(|(list, index, track)| Op::Insert {
            list,
            index,
            track
        }),
        (0..100usize).prop_map(|pick| Op::Remove { pick }),
        (0..100usize, 0..3usize, 0..20usize).prop_map(|(pick, list, index)| Op::Move {
            pick,
            list,
            index
        }),
        (0..100usize).prop_map(|pick| Op::Duplicate { pick }),
    ]
}

/// Reference model: one vector of (entry id, track id) per playlist.
type Reference = Vec<Vec<(u64, u64)>>;

fn all_entries(r: &Reference) -> Vec<u64> {
    r.iter().flatten().map(|(e, _)| *e).collect()
}

fn locate(r: &Reference, id: u64) -> (usize, usize) {
    r.iter()
        .enumerate()
        .find_map(|(li, l)| l.iter().position(|(e, _)| *e == id).map(|p| (li, p)))
        .unwrap()
}

proptest! {
    #[test]
    fn playlist_operations_match_reference(ops in proptest::collection::vec(op(), 1..60)) {
        let mut lists = Playlists::default();
        for i in 0..3u64 {
            lists.add(Playlist::new(PlaylistId(i), format!("L{i}")));
        }
        let mut reference: Reference = vec![Vec::new(); 3];
        let mut next_entry = 100u64;

        for op in ops {
            match op {
                Op::Insert { list, index, track } => {
                    next_entry += 1;
                    let entry = PlaylistEntry { id: EntryId(next_entry), track: TrackId(track), played: false };
                    lists.insert(PlaylistId(list as u64), index, vec![entry]).unwrap();
                    let at = index.min(reference[list].len());
                    reference[list].insert(at, (next_entry, track));
                }
                Op::Remove { pick } => {
                    let ids = all_entries(&reference);
                    if ids.is_empty() { continue; }
                    let id = ids[pick % ids.len()];
                    lists.remove_entry(EntryId(id)).unwrap();
                    let (li, pos) = locate(&reference, id);
                    reference[li].remove(pos);
                }
                Op::Move { pick, list, index } => {
                    let ids = all_entries(&reference);
                    if ids.is_empty() { continue; }
                    let id = ids[pick % ids.len()];
                    lists.move_entry(EntryId(id), PlaylistId(list as u64), index).unwrap();
                    let (from, pos) = locate(&reference, id);
                    let item = reference[from].remove(pos);
                    let target = if from == list && pos < index { index - 1 } else { index };
                    let at = target.min(reference[list].len());
                    reference[list].insert(at, item);
                }
                Op::Duplicate { pick } => {
                    let ids = all_entries(&reference);
                    if ids.is_empty() { continue; }
                    let id = ids[pick % ids.len()];
                    next_entry += 1;
                    lists.duplicate(EntryId(id), EntryId(next_entry)).unwrap();
                    let (li, pos) = locate(&reference, id);
                    let track = reference[li][pos].1;
                    reference[li].insert(pos + 1, (next_entry, track));
                }
            }
            let actual: Reference = lists
                .iter()
                .map(|p| p.entries.iter().map(|e| (e.id.0, e.track.0)).collect())
                .collect();
            prop_assert_eq!(&actual, &reference);
            let mut unique = all_entries(&reference);
            unique.sort_unstable();
            unique.dedup();
            prop_assert_eq!(unique.len(), all_entries(&reference).len());
        }
    }
}
