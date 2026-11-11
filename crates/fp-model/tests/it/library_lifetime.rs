#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Track library spec LB14-LB17c: one track per file, tracks stay in the
//! library, and only unused, uncategorised, missing ones leave.

use std::path::PathBuf;

use fp_model::{
    AppState, CartPageId, Command, Config, FileState, Library, MarkerKind, PlaylistEntry,
    RestoreParts, TrackId, TrackKind, apply,
};

fn state() -> AppState {
    AppState::new(Config::default(), "P")
}

fn p(s: &str) -> PathBuf {
    PathBuf::from(s)
}

fn page(s: &AppState) -> CartPageId {
    s.cartwall.pages[0].id
}

fn insert(s: &mut AppState, path: &str) {
    let playlist = s.playlists.first_id().unwrap();
    apply(
        s,
        Command::InsertPaths {
            playlist,
            index: 0,
            paths: vec![p(path)],
        },
    )
    .unwrap();
}

fn make_category(s: &mut AppState, name: &str, kind: TrackKind) -> fp_model::CategoryId {
    apply(
        s,
        Command::CreateCategory {
            parent: None,
            name: name.into(),
            color: None,
            kind,
        },
    )
    .unwrap();
    s.categories.iter().map(|c| c.id).max().unwrap()
}

#[test]
fn the_same_path_reuses_its_track_everywhere() {
    let mut s = state();
    insert(&mut s, "/m/a.wav");
    insert(&mut s, "/m/a.wav");
    let pg = page(&s);
    apply(
        &mut s,
        Command::AssignCartFile {
            page: pg,
            index: 0,
            path: p("/m/a.wav"),
        },
    )
    .unwrap();
    assert_eq!(s.library.len(), 1);
    let id = s.library.iter().next().unwrap().id;
    let list = s.playlists.get(s.playlists.first_id().unwrap()).unwrap();
    assert!(list.entries.iter().all(|e| e.track == id));
    assert_eq!(list.entries.len(), 2);
    assert_eq!(s.cartwall.pages[0].carts[0].track, Some(id));
    assert_eq!(s.library.track_at(&p("/m/a.wav")), Some(id));
    s.library.get_mut(id).unwrap().duration_secs = 100.0;
    apply(
        &mut s,
        Command::SetMarker {
            track: id,
            kind: MarkerKind::CueIn,
            secs: Some(1.0),
        },
    )
    .unwrap();
    let cart_track = s.cartwall.pages[0].carts[0].track.unwrap();
    assert_eq!(
        s.library
            .get(cart_track)
            .unwrap()
            .markers
            .cue_in
            .unwrap()
            .secs,
        1.0
    );
}

#[test]
fn a_track_stays_when_its_last_entry_goes() {
    let mut s = state();
    insert(&mut s, "/m/a.wav");
    let entry = s.playlists.iter().next().unwrap().entries[0].id;
    apply(&mut s, Command::RemoveEntry(entry)).unwrap();
    assert_eq!(s.library.len(), 1);

    apply(&mut s, Command::CreatePlaylist { name: "B".into() }).unwrap();
    let b = s.playlists.iter().nth(1).unwrap().id;
    apply(
        &mut s,
        Command::InsertPaths {
            playlist: b,
            index: 0,
            paths: vec![p("/m/b.wav")],
        },
    )
    .unwrap();
    apply(&mut s, Command::DeletePlaylist(b)).unwrap();
    assert_eq!(s.library.len(), 2);

    let pg = page(&s);
    apply(
        &mut s,
        Command::AssignCartFile {
            page: pg,
            index: 0,
            path: p("/m/c.wav"),
        },
    )
    .unwrap();
    apply(&mut s, Command::ClearCartFile { page: pg, index: 0 }).unwrap();
    assert_eq!(s.library.len(), 3);

    apply(&mut s, Command::CreateCartPage { name: "2".into() }).unwrap();
    let pg2 = s.cartwall.pages[1].id;
    apply(
        &mut s,
        Command::AssignCartFile {
            page: pg2,
            index: 0,
            path: p("/m/d.wav"),
        },
    )
    .unwrap();
    apply(&mut s, Command::DeleteCartPage(pg2)).unwrap();
    assert_eq!(s.library.len(), 4);
    assert!(s.library.iter().all(|t| t.category.is_none()));
}

#[test]
fn missing_uncategorised_unused_tracks_leave() {
    let mut s = state();
    let pg = page(&s);
    apply(
        &mut s,
        Command::AssignCartFile {
            page: pg,
            index: 0,
            path: p("/m/a.wav"),
        },
    )
    .unwrap();
    apply(&mut s, Command::ClearCartFile { page: pg, index: 0 }).unwrap();
    let a = s.library.track_at(&p("/m/a.wav")).unwrap();
    apply(
        &mut s,
        Command::SetFileState {
            track: a,
            state: FileState::Missing,
        },
    )
    .unwrap();
    assert!(s.library.get(a).is_none());
    assert_eq!(s.library.track_at(&p("/m/a.wav")), None);

    // Categorised: stays.
    let gold = make_category(&mut s, "Gold", TrackKind::Music);
    apply(
        &mut s,
        Command::AddToLibrary {
            paths: vec![p("/m/b.wav")],
            category: Some(gold),
        },
    )
    .unwrap();
    let b = s.library.track_at(&p("/m/b.wav")).unwrap();
    apply(
        &mut s,
        Command::SetFileState {
            track: b,
            state: FileState::Missing,
        },
    )
    .unwrap();
    assert!(s.library.get(b).is_some());

    // Used by an entry: stays, and leaves with its last entry.
    insert(&mut s, "/m/c.wav");
    let c = s.library.track_at(&p("/m/c.wav")).unwrap();
    apply(
        &mut s,
        Command::SetFileState {
            track: c,
            state: FileState::Missing,
        },
    )
    .unwrap();
    assert!(s.library.get(c).is_some());
    let entry = s.playlists.iter().next().unwrap().entries[0].id;
    apply(&mut s, Command::RemoveEntry(entry)).unwrap();
    assert!(s.library.get(c).is_none());
}

#[test]
fn add_to_library_takes_the_category_and_its_kind() {
    let mut s = state();
    let ads = make_category(&mut s, "Ads", TrackKind::Ad);
    let gold = make_category(&mut s, "Gold", TrackKind::Music);
    apply(
        &mut s,
        Command::AddToLibrary {
            paths: vec![p("/m/a.wav")],
            category: Some(ads),
        },
    )
    .unwrap();
    let a = s.library.track_at(&p("/m/a.wav")).unwrap();
    let t = s.library.get(a).unwrap();
    assert_eq!((t.category, t.kind), (Some(ads), TrackKind::Ad));

    apply(
        &mut s,
        Command::AddToLibrary {
            paths: vec![p("/m/a.wav")],
            category: Some(gold),
        },
    )
    .unwrap();
    assert_eq!(s.library.get(a).unwrap().category, Some(ads));
    assert_eq!(s.library.len(), 1);

    apply(
        &mut s,
        Command::AddToLibrary {
            paths: vec![p("/m/b.wav")],
            category: None,
        },
    )
    .unwrap();
    let b = s.library.track_at(&p("/m/b.wav")).unwrap();
    assert_eq!(s.library.get(b).unwrap().category, None);
    apply(
        &mut s,
        Command::AddToLibrary {
            paths: vec![p("/m/b.wav")],
            category: Some(gold),
        },
    )
    .unwrap();
    assert_eq!(s.library.get(b).unwrap().category, Some(gold));
}

fn parts(s: &AppState, library: Library) -> RestoreParts {
    RestoreParts {
        rotations: Default::default(),
        air_times: Default::default(),
        library_view: Default::default(),
        config: s.config.clone(),
        library,
        categories: Default::default(),
        playlists: s.playlists.clone(),
        cart_pages: s.cartwall.pages.clone(),
        cartwall_session: s.cartwall.session(),
        layout: s.layout.clone(),
        ids: s.ids.clone(),
    }
}

#[test]
fn restore_merges_tracks_that_share_a_path() {
    let mut s = state();
    let mut t1 = fp_model::Track::new(TrackId(1), p("/m/a.wav"));
    let mut t2 = fp_model::Track::new(TrackId(2), p("/m/a.wav"));
    t1.title = "one".into();
    t2.markers.cue_in = Some(fp_model::Marker {
        secs: 2.0,
        source: fp_model::MarkerSource::Manual,
    });
    let library = Library::from(vec![t1, t2]);
    let list = s.playlists.first_id().unwrap();
    let entry = PlaylistEntry::new(s.ids.entry(), TrackId(1));
    s.playlists.insert(list, 0, vec![entry]).unwrap();
    s.cartwall.pages[0].carts[0].track = Some(TrackId(2));
    let (r, _) = AppState::restore(parts(&s, library), &[], "P");
    assert_eq!(r.library.len(), 1);
    let t = r.library.get(TrackId(2)).unwrap();
    assert_eq!(t.markers.cue_in.unwrap().secs, 2.0);
    assert_eq!(
        r.playlists.iter().next().unwrap().entries[0].track,
        TrackId(2)
    );
    assert_eq!(r.cartwall.pages[0].carts[0].track, Some(TrackId(2)));
    assert_eq!(r.library.track_at(&p("/m/a.wav")), Some(TrackId(2)));
}

#[test]
fn restore_keeps_unreferenced_tracks() {
    let s = state();
    let library = Library::from(vec![fp_model::Track::new(TrackId(5), p("/m/a.wav"))]);
    let (r, _) = AppState::restore(parts(&s, library), &[], "P");
    assert!(r.library.get(TrackId(5)).is_some());
}

fn left(actions: &[fp_model::EngineAction]) -> Vec<(TrackId, PathBuf)> {
    actions
        .iter()
        .filter_map(|a| match a {
            fp_model::EngineAction::TrackLeftLibrary { track, path } => {
                Some((*track, path.clone()))
            }
            _ => None,
        })
        .collect()
}

/// LB17c: the model reports the implicit removal, so it can be logged.
#[test]
fn an_implicit_removal_is_reported_and_an_explicit_one_is_not() {
    let mut s = state();
    let pg = page(&s);
    apply(
        &mut s,
        Command::AssignCartFile {
            page: pg,
            index: 0,
            path: p("/m/a.wav"),
        },
    )
    .unwrap();
    apply(&mut s, Command::ClearCartFile { page: pg, index: 0 }).unwrap();
    let a = s.library.track_at(&p("/m/a.wav")).unwrap();
    let out = apply(
        &mut s,
        Command::SetFileState {
            track: a,
            state: FileState::Missing,
        },
    )
    .unwrap();
    assert_eq!(left(&out), vec![(a, p("/m/a.wav"))]);

    // The last entry of a missing track goes: reported too.
    insert(&mut s, "/m/c.wav");
    let c = s.library.track_at(&p("/m/c.wav")).unwrap();
    let out = apply(
        &mut s,
        Command::SetFileState {
            track: c,
            state: FileState::Missing,
        },
    )
    .unwrap();
    assert!(left(&out).is_empty());
    let entry = s.playlists.iter().next().unwrap().entries[0].id;
    let out = apply(&mut s, Command::RemoveEntry(entry)).unwrap();
    assert_eq!(left(&out), vec![(c, p("/m/c.wav"))]);

    // Explicit removals of missing, uncategorised tracks used by an entry
    // and by a cart are not implicit departures.
    insert(&mut s, "/m/d.wav");
    let d = s.library.track_at(&p("/m/d.wav")).unwrap();
    apply(
        &mut s,
        Command::AssignCartFile {
            page: pg,
            index: 1,
            path: p("/m/e.wav"),
        },
    )
    .unwrap();
    let e = s.library.track_at(&p("/m/e.wav")).unwrap();
    for t in [d, e] {
        apply(
            &mut s,
            Command::SetFileState {
                track: t,
                state: FileState::Missing,
            },
        )
        .unwrap();
    }
    let out = apply(&mut s, Command::RemoveTracks(vec![d, e])).unwrap();
    assert!(left(&out).is_empty(), "{out:?}");
    assert!(s.library.get(d).is_none() && s.library.get(e).is_none());
}

/// LB17c: a missing, unused track taken out of its category leaves.
#[test]
fn a_missing_unused_track_taken_out_of_its_category_leaves() {
    let mut s = state();
    let gold = make_category(&mut s, "Gold", TrackKind::Music);
    apply(
        &mut s,
        Command::AddToLibrary {
            paths: vec![p("/m/a.wav"), p("/m/b.wav")],
            category: Some(gold),
        },
    )
    .unwrap();
    let a = s.library.track_at(&p("/m/a.wav")).unwrap();
    let b = s.library.track_at(&p("/m/b.wav")).unwrap();
    apply(
        &mut s,
        Command::SetFileState {
            track: a,
            state: FileState::Missing,
        },
    )
    .unwrap();
    assert!(s.library.get(a).is_some());
    let out = apply(
        &mut s,
        Command::MoveTracksToCategory {
            tracks: vec![a, b],
            category: None,
        },
    )
    .unwrap();
    assert_eq!(left(&out), vec![(a, p("/m/a.wav"))]);
    assert!(s.library.get(a).is_none());
    assert!(s.library.get(b).is_some_and(|t| t.category.is_none()));
}
