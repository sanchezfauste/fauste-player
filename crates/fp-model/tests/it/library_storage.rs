//! The library shares its storage between clones, so a snapshot of the state
//! costs nothing and a diff against an older snapshot skips what did not change.

use std::path::PathBuf;

use fp_model::{Command, Library, PlayMode, Track, TrackId, apply};

use crate::common::{fixture, p0};

fn library_of(n: u64) -> Library {
    (1..=n)
        .map(|i| Track::new(TrackId(i), PathBuf::from(format!("/m/{i}.flac"))))
        .collect::<Vec<_>>()
        .into()
}

#[test]
fn a_clone_shares_storage_until_a_track_changes() {
    let lib = library_of(20_000);
    let mut copy = lib.clone();
    assert!(copy.shares_storage(&lib));
    copy.get_mut(TrackId(5)).unwrap().title = "x".into();
    assert!(!copy.shares_storage(&lib));
    assert_eq!(lib.get(TrackId(5)).unwrap().title, "5");
}

#[test]
fn changes_since_lists_only_what_changed() {
    let old = library_of(1_000);
    let mut new = old.clone();
    new.get_mut(TrackId(7)).unwrap().title = "edited".into();
    new.remove(TrackId(9));
    new.insert(Track::new(TrackId(5_000), "/m/new.flac".into()));
    let c = new.changes_since(&old);
    let mut ids: Vec<u64> = c.upserted.iter().map(|t| t.id.0).collect();
    ids.sort();
    assert_eq!(ids, vec![7, 5_000]);
    assert_eq!(c.removed, vec![TrackId(9)]);
}

#[test]
fn an_unchanged_clone_has_no_changes() {
    let old = library_of(100);
    let copy = old.clone();
    let c = copy.changes_since(&old);
    assert!(c.upserted.is_empty() && c.removed.is_empty());
}

#[test]
fn commands_that_do_not_touch_tracks_keep_the_library_shared() {
    let state = fixture(3);
    let p = p0(&state);
    for command in [
        Command::SetVolume(p, 0.5),
        Command::Play(p),
        Command::Pause(p),
        Command::Stop(p),
        Command::SetMode(p, PlayMode::Single),
    ] {
        let mut s = state.clone();
        let _ = apply(&mut s, command.clone());
        assert!(s.library.shares_storage(&state.library), "{command:?}");
    }
}
