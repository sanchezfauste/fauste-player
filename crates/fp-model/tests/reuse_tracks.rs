#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Remote control spec §3.4: a client reuses tracks already loaded, so their
//! markers and analysis come along.

mod common;

use common::{entries, fixture};
use fp_model::{Command, MarkerKind, TrackId, apply};

#[test]
fn inserting_a_track_shares_it_with_its_markers() {
    let mut s = fixture(3);
    let t = s.playlists.entry(entries(&s)[0]).unwrap().track;
    apply(
        &mut s,
        Command::SetMarker {
            track: t,
            kind: MarkerKind::IntroEnd,
            secs: Some(5.0),
        },
    )
    .unwrap();
    apply(&mut s, Command::CreatePlaylist { name: "B".into() }).unwrap();
    let b = s.playlists.iter().nth(1).unwrap().id;
    apply(
        &mut s,
        Command::InsertTracks {
            playlist: b,
            index: 0,
            tracks: vec![t],
        },
    )
    .unwrap();
    let copy = &s.playlists.get(b).unwrap().entries[0];
    assert_eq!(copy.track, t, "the same track, not a new one");
    assert_eq!(
        s.library.get(t).unwrap().markers.intro_end.unwrap().secs,
        5.0
    );
}

#[test]
fn inserting_an_unknown_track_is_refused() {
    let mut s = fixture(1);
    let list = s.playlists.first_id().unwrap();
    assert!(
        apply(
            &mut s,
            Command::InsertTracks {
                playlist: list,
                index: 0,
                tracks: vec![TrackId(999_999)]
            }
        )
        .is_err()
    );
    assert_eq!(s.playlists.get(list).unwrap().entries.len(), 1);
}

#[test]
fn a_cart_can_take_a_loaded_track() {
    let mut s = fixture(1);
    let t = s.playlists.entry(entries(&s)[0]).unwrap().track;
    let page = s.cartwall.pages[0].id;
    apply(
        &mut s,
        Command::AssignCartTrack {
            page,
            index: 2,
            track: t,
        },
    )
    .unwrap();
    assert_eq!(s.cartwall.pages[0].carts[2].track, Some(t));
    // Replacing it keeps the track: the playlist still uses it.
    apply(&mut s, Command::ClearCartFile { page, index: 2 }).unwrap();
    assert!(s.library.get(t).is_some());
    assert!(
        apply(
            &mut s,
            Command::AssignCartTrack {
                page,
                index: 2,
                track: TrackId(999_999)
            }
        )
        .is_err()
    );
}
