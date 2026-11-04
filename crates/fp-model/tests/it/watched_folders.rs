#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Watched folders (track library spec LB19).

use std::path::{Path, PathBuf};

use fp_model::{
    AppState, Categories, Category, CategoryId, Command, Config, ModelError, TrackKind, apply,
    watching_category,
};

fn state() -> AppState {
    AppState::new(Config::default(), "P")
}

fn create(s: &mut AppState, name: &str) -> CategoryId {
    let before = s.categories.max_raw_id();
    apply(
        s,
        Command::CreateCategory {
            parent: None,
            name: name.into(),
            color: None,
            kind: TrackKind::Music,
        },
    )
    .unwrap();
    s.categories
        .iter()
        .map(|c| c.id)
        .find(|id| id.0 > before)
        .unwrap()
}

fn watch(s: &mut AppState, category: CategoryId, folder: &str) -> Result<(), ModelError> {
    apply(
        s,
        Command::SetWatchedFolder {
            category,
            folder: Some(PathBuf::from(folder)),
        },
    )
    .map(|_| ())
}

fn folder(s: &AppState, id: CategoryId) -> Option<PathBuf> {
    s.categories.get(id).unwrap().watched_folder.clone()
}

#[test]
fn setting_a_watched_folder_stores_it_and_none_clears_it() {
    let mut s = state();
    let a = create(&mut s, "A");
    watch(&mut s, a, "/m/ids").unwrap();
    assert_eq!(folder(&s, a), Some(PathBuf::from("/m/ids")));
    apply(
        &mut s,
        Command::SetWatchedFolder {
            category: a,
            folder: None,
        },
    )
    .unwrap();
    assert_eq!(folder(&s, a), None);
}

#[test]
fn the_same_folder_cannot_be_watched_twice() {
    let mut s = state();
    let a = create(&mut s, "A");
    let b = create(&mut s, "B");
    watch(&mut s, a, "/m").unwrap();
    assert_eq!(
        watch(&mut s, b, "/m"),
        Err(ModelError::FolderAlreadyWatched(a))
    );
    assert_eq!(folder(&s, b), None);
    watch(&mut s, a, "/m").unwrap();
}

#[test]
fn nested_folders_may_be_watched_and_the_innermost_wins() {
    let mut s = state();
    let a = create(&mut s, "A");
    let b = create(&mut s, "B");
    watch(&mut s, a, "/m").unwrap();
    watch(&mut s, b, "/m/gold").unwrap();
    let at = |p: &str| watching_category(&s.categories, Path::new(p));
    assert_eq!(at("/m/gold/x/1.mp3"), Some(b));
    assert_eq!(at("/m/new/2.mp3"), Some(a));
    assert_eq!(at("/m/golden/3.mp3"), Some(a));
    assert_eq!(at("/other/4.mp3"), None);
}

#[test]
fn an_unknown_category_is_refused() {
    let mut s = state();
    assert!(matches!(
        watch(&mut s, CategoryId(77), "/m"),
        Err(ModelError::UnknownCategory(_))
    ));
}

fn cat(id: u64, folder: &str) -> Category {
    Category {
        id: CategoryId(id),
        parent: None,
        name: format!("c{id}"),
        color: None,
        kind: TrackKind::Music,
        watched_folder: Some(PathBuf::from(folder)),
    }
}

#[test]
fn repair_clears_a_folder_watched_twice() {
    let (fixed, warnings) = Categories::from(vec![cat(1, "/m"), cat(2, "/m")]).repaired();
    assert_eq!(
        fixed.get(CategoryId(1)).unwrap().watched_folder,
        Some(PathBuf::from("/m"))
    );
    assert_eq!(fixed.get(CategoryId(2)).unwrap().watched_folder, None);
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(warnings[0].contains("c2"), "{warnings:?}");
}

#[test]
fn deleting_a_category_drops_its_folder() {
    let mut s = state();
    let a = create(&mut s, "A");
    watch(&mut s, a, "/m").unwrap();
    apply(&mut s, Command::DeleteCategory(a)).unwrap();
    assert_eq!(
        watching_category(&s.categories, Path::new("/m/1.mp3")),
        None
    );
}
