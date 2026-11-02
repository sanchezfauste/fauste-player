#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Track library spec LB11-LB13, LB17, LB17b: move, re-kind and remove
//! tracks, delete categories, and refuse whole what touches the air.

use crate::common::{entries, fixture, p0};
use fp_model::{
    AppState, CartId, CategoryId, Command, EntryId, ModelError, TrackId, TrackKind,
    category_deletion, removal_impact,
};

fn category(
    s: &mut AppState,
    parent: Option<CategoryId>,
    name: &str,
    kind: TrackKind,
) -> CategoryId {
    apply(
        s,
        Command::CreateCategory {
            parent,
            name: name.into(),
            color: None,
            kind,
        },
    );
    s.categories.iter().map(|c| c.id).max().unwrap()
}

fn apply(s: &mut AppState, c: Command) {
    fp_model::apply(s, c).unwrap();
}

fn tracks_of(s: &AppState) -> Vec<TrackId> {
    entries(s)
        .into_iter()
        .map(|e| s.playlists.entry(e).unwrap().track)
        .collect()
}

fn put_in_cart(s: &mut AppState, index: usize, track: TrackId) -> CartId {
    let page = s.cartwall.pages[0].id;
    apply(s, Command::AssignCartTrack { page, index, track });
    s.cartwall.pages[0].carts[index].id
}

#[test]
fn moving_tracks_sets_their_category_and_kind() {
    let mut s = fixture(3);
    let ads = category(&mut s, None, "Ads", TrackKind::Ad);
    let t = tracks_of(&s);
    apply(
        &mut s,
        Command::MoveTracksToCategory {
            tracks: vec![t[0], t[1]],
            category: Some(ads),
        },
    );
    for id in [t[0], t[1]] {
        let tr = s.library.get(id).unwrap();
        assert_eq!((tr.category, tr.kind), (Some(ads), TrackKind::Ad));
    }
    apply(
        &mut s,
        Command::MoveTracksToCategory {
            tracks: vec![t[0]],
            category: None,
        },
    );
    let tr = s.library.get(t[0]).unwrap();
    assert_eq!((tr.category, tr.kind), (None, TrackKind::Ad));

    let before = s.clone();
    let err = fp_model::apply(
        &mut s,
        Command::MoveTracksToCategory {
            tracks: vec![t[2]],
            category: Some(CategoryId(999)),
        },
    );
    assert_eq!(err, Err(ModelError::UnknownCategory(CategoryId(999))));
    assert_eq!(s, before);
}

#[test]
fn the_kind_can_be_changed_on_the_track() {
    let mut s = fixture(1);
    let ads = category(&mut s, None, "Ads", TrackKind::Ad);
    let t = tracks_of(&s)[0];
    apply(
        &mut s,
        Command::MoveTracksToCategory {
            tracks: vec![t],
            category: Some(ads),
        },
    );
    apply(
        &mut s,
        Command::SetTracksKind {
            tracks: vec![t],
            kind: TrackKind::Jingle,
        },
    );
    let tr = s.library.get(t).unwrap();
    assert_eq!((tr.category, tr.kind), (Some(ads), TrackKind::Jingle));
}

#[test]
fn removal_impact_counts_entries_and_carts() {
    let mut s = fixture(1);
    let playlist = s.playlists.first_id().unwrap();
    let t = tracks_of(&s)[0];
    // A second entry of the same track and a cart.
    let e0 = entries(&s)[0];
    apply(&mut s, Command::DuplicateEntry(e0));
    assert_eq!(s.playlists.get(playlist).unwrap().entries.len(), 2);
    put_in_cart(&mut s, 0, t);
    let impact = removal_impact(&s, &[t]);
    assert_eq!((impact.tracks, impact.entries, impact.carts), (1, 2, 1));
    assert!(impact.on_air.is_empty());
}

#[test]
fn remove_tracks_removes_their_entries_and_empties_their_carts() {
    let mut s = fixture(4);
    let t = tracks_of(&s);
    let e = entries(&s);
    let cart = put_in_cart(&mut s, 0, t[1]);
    let page = s.cartwall.pages[0].id;
    apply(
        &mut s,
        Command::EditCart {
            page,
            index: 0,
            edit: fp_model::CartEdit {
                name: "Jingle".into(),
                kind: Default::default(),
                looped: false,
                exclusive: false,
                color: None,
            },
            file: fp_model::CartFileChange::Keep,
        },
    );
    // Player 0 is stopped with the second entry as its next.
    let p = p0(&s);
    apply(&mut s, Command::SetNext(p, e[1]));
    apply(&mut s, Command::RemoveTracks(vec![t[1]]));
    assert!(s.library.get(t[1]).is_none());
    assert!(entries(&s).iter().all(|x| *x != e[1]));
    let c = s.cartwall.cart(cart).unwrap();
    assert_eq!(c.track, None);
    assert_eq!(c.name, "Jingle");
    assert_eq!(s.players[0].next, Some(e[2]));
}

fn on_air_setup() -> (AppState, Vec<TrackId>, EntryId) {
    let mut s = fixture(3);
    let ads = category(&mut s, None, "Ads", TrackKind::Ad);
    let t = tracks_of(&s);
    apply(
        &mut s,
        Command::MoveTracksToCategory {
            tracks: vec![t[0], t[1]],
            category: Some(ads),
        },
    );
    let p = p0(&s);
    apply(&mut s, Command::Play(p));
    let current = s.players[0].current.unwrap();
    (s, t, current)
}

#[test]
fn a_deletion_touching_an_on_air_entry_is_refused_whole() {
    let (mut s, t, current) = on_air_setup();
    let on_air_track = s.playlists.entry(current).unwrap().track;
    assert!(t[..2].contains(&on_air_track));
    let ads = s.categories.iter().next().unwrap().id;
    let before = s.clone();
    let tracks = vec![t[0], t[1]];
    for c in [Command::RemoveTracks(tracks), Command::DeleteCategory(ads)] {
        match fp_model::apply(&mut s, c) {
            Err(ModelError::LibraryOnAir(h)) => {
                assert_eq!(h.entries, vec![current]);
                assert!(h.carts.is_empty());
            }
            other => panic!("expected LibraryOnAir, got {other:?}"),
        }
        assert_eq!(s, before);
    }
    let impact = removal_impact(&s, &[t[0], t[1]]);
    assert_eq!(impact.on_air.entries, vec![current]);
}

#[test]
fn a_playing_or_pre_listened_cart_refuses_the_removal() {
    for fire in [true, false] {
        let mut s = fixture(2);
        let t = tracks_of(&s);
        let cart = put_in_cart(&mut s, 0, t[0]);
        apply(
            &mut s,
            if fire {
                Command::FireCart(cart)
            } else {
                Command::CueCart(cart)
            },
        );
        let before = s.clone();
        match fp_model::apply(&mut s, Command::RemoveTracks(vec![t[0], t[1]])) {
            Err(ModelError::LibraryOnAir(h)) => assert_eq!(h.carts, vec![cart]),
            other => panic!("expected LibraryOnAir, got {other:?}"),
        }
        assert_eq!(s, before);
    }
}

#[test]
fn deleting_a_category_removes_its_subtree_and_tracks() {
    let mut s = fixture(3);
    let a = category(&mut s, None, "A", TrackKind::Music);
    let b = category(&mut s, Some(a), "B", TrackKind::Music);
    let c = category(&mut s, None, "C", TrackKind::Music);
    let t = tracks_of(&s);
    for (track, cat) in [(t[0], a), (t[1], b), (t[2], c)] {
        apply(
            &mut s,
            Command::MoveTracksToCategory {
                tracks: vec![track],
                category: Some(cat),
            },
        );
    }
    let (subs, impact) = category_deletion(&s, a).unwrap();
    assert_eq!((subs, impact.tracks, impact.entries), (1, 2, 2));
    assert!(category_deletion(&s, CategoryId(999)).is_none());
    apply(&mut s, Command::DeleteCategory(a));
    assert_eq!(
        s.categories.iter().map(|x| x.id).collect::<Vec<_>>(),
        vec![c]
    );
    assert_eq!(s.library.ids(), vec![t[2]]);
    assert_eq!(entries(&s).len(), 1);
}

#[test]
fn a_pre_listened_entry_refuses_removal() {
    let mut s = fixture(2);
    let t = tracks_of(&s);
    let e = entries(&s);
    let p = p0(&s);
    apply(&mut s, Command::CueEntry(p, e[1]));
    assert!(s.players[0].cue.is_some());
    let before = s.clone();
    match fp_model::apply(&mut s, Command::RemoveTracks(vec![t[1]])) {
        Err(ModelError::LibraryOnAir(h)) => assert_eq!(h.entries, vec![e[1]]),
        other => panic!("expected LibraryOnAir, got {other:?}"),
    }
    assert_eq!(s, before);
}

/// LB17b: a next whose entry goes moves past every removed successor.
#[test]
fn a_removed_next_hops_past_every_removed_successor() {
    let mut s = fixture(5);
    let t = tracks_of(&s);
    let e = entries(&s);
    let p = p0(&s);
    apply(&mut s, Command::SetNext(p, e[1]));
    apply(&mut s, Command::RemoveTracks(vec![t[1], t[2]]));
    assert_eq!(s.players[0].next, Some(e[3]));
    assert_eq!(entries(&s), vec![e[0], e[3], e[4]]);
}

/// 2 000 used tracks with two entries each, filed under one category, after
/// 4 000 uncategorised entries.
fn big_category() -> (AppState, CategoryId) {
    let mut s = fp_model::AppState::new(fp_model::Config::default(), "Main");
    let playlist = s.playlists.first_id().unwrap();
    let paths: Vec<_> = (0..2_000)
        .map(|i| std::path::PathBuf::from(format!("/music/{i}.flac")))
        .collect();
    for _ in 0..2 {
        apply(
            &mut s,
            Command::InsertPaths {
                playlist,
                index: 0,
                paths: paths.clone(),
            },
        );
    }
    let gold = category(&mut s, None, "Gold", TrackKind::Music);
    let tracks = s.library.ids();
    assert_eq!(tracks.len(), 2_000);
    apply(
        &mut s,
        Command::MoveTracksToCategory {
            tracks,
            category: Some(gold),
        },
    );
    // 4 000 entries that stay, before them: each removed entry is found
    // past them.
    let kept: Vec<_> = (0..4_000)
        .map(|i| std::path::PathBuf::from(format!("/kept/{i}.flac")))
        .collect();
    apply(
        &mut s,
        Command::InsertPaths {
            playlist,
            index: 0,
            paths: kept,
        },
    );
    (s, gold)
}

#[test]
fn deleting_a_big_category_removes_every_entry() {
    let (mut s, gold) = big_category();
    assert_eq!(entries(&s).len(), 8_000);
    apply(&mut s, Command::DeleteCategory(gold));
    assert_eq!(entries(&s).len(), 4_000);
    assert_eq!(s.library.len(), 4_000);
    assert!(s.library.iter().all(|t| t.category.is_none()));
    assert!(s.categories.is_empty());
}

/// The removal is one pass over the playlists, not one per entry (one pass
/// per entry took about 120 ms in a debug build, the batch about 16 ms).
/// Timed, so ignored by default: run with `--ignored`.
#[test]
#[ignore = "timing"]
fn deleting_a_big_category_is_fast() {
    let (mut s, gold) = big_category();
    let started = std::time::Instant::now();
    apply(&mut s, Command::DeleteCategory(gold));
    let took = started.elapsed();
    assert!(took < std::time::Duration::from_millis(50), "took {took:?}");
}
