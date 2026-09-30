#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback spec §2: per-entry repeat (R26) and stop after (R27).

mod common;

use common::{entries, fixture, p0};
use fp_model::{AppState, Command, EntryId, PlayerId, apply};

fn three() -> (AppState, PlayerId, [EntryId; 3]) {
    let state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    (state, p, [e[0], e[1], e[2]])
}

fn entry(s: &AppState, e: EntryId) -> &fp_model::PlaylistEntry {
    s.playlists.entry(e).unwrap()
}

#[test]
fn the_toggles_flip_each_flag() {
    let (mut s, _, [a, _, _]) = three();
    assert!(!entry(&s, a).repeat && !entry(&s, a).stop_after);
    apply(&mut s, Command::ToggleEntryRepeat(a)).unwrap();
    apply(&mut s, Command::ToggleEntryStopAfter(a)).unwrap();
    assert!(entry(&s, a).repeat && entry(&s, a).stop_after);
    apply(&mut s, Command::ToggleEntryRepeat(a)).unwrap();
    assert!(!entry(&s, a).repeat);
    assert!(apply(&mut s, Command::ToggleEntryRepeat(EntryId(999_999))).is_err());
}

#[test]
fn duplicating_an_entry_copies_its_flags() {
    let (mut s, _, [a, _, _]) = three();
    apply(&mut s, Command::ToggleEntryRepeat(a)).unwrap();
    apply(&mut s, Command::ToggleEntryStopAfter(a)).unwrap();
    apply(&mut s, Command::DuplicateEntry(a)).unwrap();
    let copy = entries(&s)[1];
    assert_ne!(copy, a);
    assert!(entry(&s, copy).repeat && entry(&s, copy).stop_after);
}

#[test]
fn flags_are_saved_only_when_set_and_old_files_load_without_them() {
    let (mut s, _, [a, b, _]) = three();
    apply(&mut s, Command::ToggleEntryRepeat(a)).unwrap();
    let json = serde_json::to_value(entry(&s, a)).unwrap();
    assert_eq!(json["repeat"], true);
    assert!(json.get("stop_after").is_none());
    assert!(
        serde_json::to_value(entry(&s, b))
            .unwrap()
            .get("repeat")
            .is_none()
    );
    let old: fp_model::PlaylistEntry =
        serde_json::from_value(serde_json::json!({"id": 7, "track": 3})).unwrap();
    assert!(!old.repeat && !old.stop_after);
}
