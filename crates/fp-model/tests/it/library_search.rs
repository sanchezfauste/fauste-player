#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Search, counts and sorting of the library (plan 3, task 2).

use std::path::PathBuf;
use std::time::Instant;

use fp_model::library_search::{
    SearchIndex, SortKeys, category_counts, fold, library_rows, query_words, sort_rows,
    sort_rows_by,
};
use fp_model::{
    Categories, Category, CategoryId, Library, LibraryColumn, LibrarySelection, LibrarySort,
    TableColumn, Track, TrackId, TrackKind,
};

fn track(id: u64, path: &str, title: &str, artist: &str, cat: Option<u64>) -> Track {
    let mut t = Track::new(TrackId(id), PathBuf::from(path));
    t.title = title.into();
    t.artist = artist.into();
    t.category = cat.map(CategoryId);
    t
}

fn cat(id: u64, parent: Option<u64>, name: &str) -> Category {
    Category {
        id: CategoryId(id),
        parent: parent.map(CategoryId),
        name: name.into(),
        color: None,
        kind: TrackKind::Music,
        watched_folder: None,
    }
}

fn library(tracks: Vec<Track>) -> Library {
    Library::from(tracks)
}

fn rows(
    lib: &Library,
    cats: &Categories,
    selection: LibrarySelection,
    whole: bool,
    query: &str,
) -> Vec<u64> {
    let index = SearchIndex::build(lib);
    library_rows(lib, cats, &index, selection, whole, query)
        .into_iter()
        .map(|t| t.0)
        .collect()
}

/// Parent 100 with child 101, and sibling 102.
fn tree() -> Categories {
    Categories::from(vec![
        cat(100, None, "Parent"),
        cat(101, Some(100), "Child"),
        cat(102, None, "Sibling"),
    ])
}

fn tree_library() -> Library {
    library(vec![
        track(1, "/m/a.wav", "A", "", Some(100)),
        track(2, "/m/b.wav", "B", "", Some(101)),
        track(3, "/m/c.wav", "C", "", Some(102)),
        track(4, "/m/d.wav", "D", "", None),
    ])
}

#[test]
fn fold_drops_accents_and_case() {
    assert_eq!(fold("Canción ÀÉÎ"), "cancion aei");
}

#[test]
fn a_search_is_case_and_accent_insensitive() {
    let lib = library(vec![track(1, "/m/x.wav", "Canción", "", None)]);
    assert_eq!(
        rows(
            &lib,
            &Categories::default(),
            LibrarySelection::All,
            false,
            "CANCION"
        ),
        vec![1]
    );
}

#[test]
fn every_word_must_match_somewhere() {
    let lib = library(vec![
        track(1, "/m/x.wav", "Canción", "López", None),
        track(2, "/m/y.wav", "Canción", "Pérez", None),
        track(3, "/m/z.wav", "Otra", "López", None),
    ]);
    assert_eq!(
        rows(
            &lib,
            &Categories::default(),
            LibrarySelection::All,
            false,
            "cancion lopez"
        ),
        vec![1]
    );
}

#[test]
fn the_file_name_is_searched() {
    let lib = library(vec![
        track(1, "/m/demo_take3.wav", "T", "", None),
        track(2, "/m/other.wav", "T", "", None),
    ]);
    assert_eq!(
        rows(
            &lib,
            &Categories::default(),
            LibrarySelection::All,
            false,
            "demo_take3"
        ),
        vec![1]
    );
}

#[test]
fn the_search_covers_the_selected_category_and_its_subcategories() {
    let got = rows(
        &tree_library(),
        &tree(),
        LibrarySelection::Category(CategoryId(100)),
        false,
        "",
    );
    assert_eq!(got, vec![1, 2]);
}

#[test]
fn whole_library_searches_everything() {
    let got = rows(
        &tree_library(),
        &tree(),
        LibrarySelection::Category(CategoryId(102)),
        true,
        "",
    );
    assert_eq!(got, vec![1, 2, 3, 4]);
}

#[test]
fn uncategorised_shows_tracks_without_a_category() {
    let got = rows(
        &tree_library(),
        &tree(),
        LibrarySelection::Uncategorised,
        false,
        "",
    );
    assert_eq!(got, vec![4]);
}

#[test]
fn an_empty_query_lists_the_whole_scope() {
    let got = rows(
        &tree_library(),
        &tree(),
        LibrarySelection::All,
        false,
        "   ",
    );
    assert_eq!(got, vec![1, 2, 3, 4]);
    assert!(query_words("  ").is_empty());
    assert_eq!(
        query_words(" Ab  Cé "),
        vec!["ab".to_owned(), "ce".to_owned()]
    );
}

#[test]
fn update_follows_an_edited_title() {
    let old = library(vec![track(1, "/m/x.wav", "Alpha", "", None)]);
    let mut index = SearchIndex::build(&old);
    let mut new = old.clone();
    new.get_mut(TrackId(1)).unwrap().title = "Omega".into();
    index.update(&old, &new);
    let words = |q: &str| query_words(q);
    assert!(index.matches(TrackId(1), &words("omega")));
    assert!(!index.matches(TrackId(1), &words("alpha")));
    let mut newer = new.clone();
    newer.remove(TrackId(1));
    index.update(&new, &newer);
    assert!(!index.matches(TrackId(1), &words("omega")));
}

#[test]
fn counts_include_subcategories() {
    let counts = category_counts(&tree_library(), &tree());
    assert_eq!(counts.all, 4);
    assert_eq!(counts.uncategorised, 1);
    assert_eq!(counts.by_category.get(&CategoryId(100)), Some(&2));
    assert_eq!(counts.by_category.get(&CategoryId(101)), Some(&1));
    assert_eq!(counts.by_category.get(&CategoryId(102)), Some(&1));
}

/// 20 categories with 100 subcategories each, a track in every one, plus a
/// cycle a hand edit left (counted once) and a track whose category is
/// unknown: counted in one pass, not a subtree walk per category.
#[test]
fn counts_of_a_wide_tree_are_quick_and_right() {
    let mut cats = Vec::new();
    let mut tracks = Vec::new();
    for top in 0..20u64 {
        let parent = 10_000 + top;
        cats.push(cat(parent, None, "Top"));
        for child in 0..100u64 {
            let id = parent * 1_000 + child;
            cats.push(cat(id, Some(parent), "Child"));
            tracks.push(track(id, &format!("/m/{id}.wav"), "T", "", Some(id)));
        }
    }
    cats.push(cat(9_001, Some(9_002), "Loop A"));
    cats.push(cat(9_002, Some(9_001), "Loop B"));
    tracks.push(track(9_001, "/m/loop.wav", "L", "", Some(9_001)));
    tracks.push(track(9_999, "/m/lost.wav", "X", "", Some(77_777)));
    let (lib, cats) = (library(tracks), Categories::from(cats));
    let started = Instant::now();
    let counts = category_counts(&lib, &cats);
    assert!(
        started.elapsed().as_secs_f64() < 1.0,
        "{:?}",
        started.elapsed()
    );
    assert_eq!(counts.by_category.get(&CategoryId(10_000)), Some(&100));
    assert_eq!(counts.by_category.get(&CategoryId(10_019_099)), Some(&1));
    assert_eq!(counts.by_category.get(&CategoryId(9_001)), Some(&1));
    assert_eq!(counts.by_category.get(&CategoryId(9_002)), Some(&1));
    assert_eq!(counts.by_category.get(&CategoryId(77_777)), None);
}

fn sorted(lib: &Library, cats: &Categories, column: LibraryColumn, ascending: bool) -> Vec<u64> {
    let mut ids = lib.ids();
    sort_rows(&mut ids, lib, cats, LibrarySort { column, ascending });
    ids.into_iter().map(|t| t.0).collect()
}

#[test]
fn sort_rows_orders_by_a_column_and_direction() {
    let lib = library(vec![
        track(1, "/m/1.wav", "beta", "", Some(102)),
        track(2, "/m/2.wav", "Álamo", "", None),
        track(3, "/m/3.wav", "Beta", "", Some(101)),
        track(4, "/m/4.wav", "cedro", "", Some(102)),
    ]);
    let cats = tree();
    let title = LibraryColumn::Tag(TableColumn::Title);
    // "beta" and "Beta" tie: by id.
    assert_eq!(sorted(&lib, &cats, title, true), vec![2, 1, 3, 4]);
    assert_eq!(sorted(&lib, &cats, title, false), vec![4, 1, 3, 2]);
    // Child < Sibling by name; uncategorised last; ties by id.
    assert_eq!(
        sorted(&lib, &cats, LibraryColumn::Category, true),
        vec![3, 1, 4, 2]
    );
    assert_eq!(
        sorted(&lib, &cats, LibraryColumn::Tag(TableColumn::Number), false),
        vec![4, 3, 2, 1]
    );
}

#[test]
#[cfg_attr(debug_assertions, ignore)]
fn searching_20000_tracks_takes_less_than_a_frame() {
    let tracks = (1..=20_000u64)
        .map(|i| {
            track(
                i,
                &format!("/music/folder{}/file{i}.flac", i % 50),
                &format!("Canción número {i}"),
                &format!("Artista {}", i % 300),
                Some(100 + i % 3),
            )
        })
        .collect();
    let lib = library(tracks);
    let cats = tree();
    let index = SearchIndex::build(&lib);
    let started = Instant::now();
    let got = library_rows(
        &lib,
        &cats,
        &index,
        LibrarySelection::All,
        true,
        "cancion 1999",
    );
    let took = started.elapsed();
    assert!(!got.is_empty());
    assert!(took.as_millis() < 16, "took {took:?}");
}

/// I3: the folded sort keys are kept per track, so a library change folds
/// only the tracks it touched, and the order with the kept keys is the
/// order `sort_rows` gives.
#[test]
fn sort_keys_fold_only_the_tracks_a_change_touched() {
    let tracks = (1..=500u64)
        .map(|i| {
            track(
                i,
                &format!("/m/{i}.wav"),
                &format!("Título {}", 500 - i),
                "",
                Some(100 + i % 3),
            )
        })
        .collect();
    let old = library(tracks);
    let cats = tree();
    let title = LibraryColumn::Tag(TableColumn::Title);
    let mut keys = SortKeys::build(&old, &cats, title);
    assert_eq!(keys.column(), title);
    let mut new = old.clone();
    new.get_mut(TrackId(7)).unwrap().title = "Álef".into();
    new.get_mut(TrackId(9)).unwrap().duration_secs = 12.0;
    new.remove(TrackId(11));
    assert_eq!(keys.update(&old, &new, &cats), 2, "two tracks changed");
    for ascending in [true, false] {
        let sort = LibrarySort {
            column: title,
            ascending,
        };
        let mut expected = new.ids();
        sort_rows(&mut expected, &new, &cats, sort);
        let mut got = new.ids();
        sort_rows_by(&mut got, &keys, ascending);
        assert_eq!(got, expected);
        if ascending {
            assert_eq!(got.first(), Some(&TrackId(7)), "the renamed title leads");
        }
    }
    // Nothing changed: nothing folded.
    assert_eq!(keys.update(&new, &new.clone(), &cats), 0);
}

/// A category column folds again when a category is renamed: its key is
/// the category's name.
#[test]
fn category_sort_keys_follow_a_renamed_category() {
    let lib = tree_library();
    let cats = tree();
    let mut keys = SortKeys::build(&lib, &cats, LibraryColumn::Category);
    let mut renamed = cats.clone().into_vec();
    for c in &mut renamed {
        if c.id == CategoryId(102) {
            c.name = "Aaa".into();
        }
    }
    let renamed = Categories::from(renamed);
    keys.update(&lib, &lib.clone(), &renamed);
    let sort = LibrarySort {
        column: LibraryColumn::Category,
        ascending: true,
    };
    let mut expected = lib.ids();
    sort_rows(&mut expected, &lib, &renamed, sort);
    let mut got = lib.ids();
    sort_rows_by(&mut got, &keys, true);
    assert_eq!(got, expected);
}

/// LB31 with a sort: after a library change, keeping the rows and their
/// order by title over 20 000 tracks stays well within a frame.
#[test]
#[cfg_attr(debug_assertions, ignore)]
fn a_change_to_20000_sorted_tracks_takes_less_than_a_frame() {
    let tracks = (1..=20_000u64)
        .map(|i| {
            track(
                i,
                &format!("/music/folder{}/file{i}.flac", i % 50),
                &format!("Canción número {}", (i * 7919) % 20_000),
                &format!("Artista {}", i % 300),
                Some(100 + i % 3),
            )
        })
        .collect();
    let old = library(tracks);
    let cats = tree();
    let title = LibraryColumn::Tag(TableColumn::Title);
    let mut index = SearchIndex::build(&old);
    let mut keys = SortKeys::build(&old, &cats, title);
    let mut new = old.clone();
    for i in 1..=50u64 {
        new.get_mut(TrackId(i * 300)).unwrap().duration_secs = 90.0;
    }
    let started = Instant::now();
    index.update(&old, &new);
    keys.update(&old, &new, &cats);
    let mut rows = library_rows(&new, &cats, &index, LibrarySelection::All, true, "");
    sort_rows_by(&mut rows, &keys, true);
    let took = started.elapsed();
    eprintln!("rows + sort after a change: {took:?}");
    assert_eq!(rows.len(), 20_000);
    assert!(took.as_millis() < 8, "took {took:?}");
}

/// External editor spec EE11: *Markers to review* lists the flagged
/// tracks, whatever their category, and the counts tell how many.
#[test]
fn markers_to_review_lists_only_flagged_tracks() {
    let mut lib = tree_library();
    for id in [2, 4] {
        lib.get_mut(TrackId(id)).unwrap().markers_review = Some(fp_model::MarkersReview {
            dropped: Vec::new(),
        });
    }
    let got = rows(&lib, &tree(), LibrarySelection::MarkersToReview, false, "");
    assert_eq!(got, vec![2, 4]);
    let searched = rows(&lib, &tree(), LibrarySelection::MarkersToReview, false, "d");
    assert_eq!(searched, vec![4]);
    assert_eq!(category_counts(&lib, &tree()).markers_review, 2);
    assert_eq!(category_counts(&tree_library(), &tree()).markers_review, 0);
}
