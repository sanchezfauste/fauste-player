#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The library window's state in the database (the `library_view` row of `kv`).

use fp_model::{
    AppState, Command, Config, LibraryColumn, LibraryColumnWidth, LibrarySort, LibraryView, apply,
};
use fp_store::{AppPaths, Store};

use crate::support::{edit_kv, save_all};

fn store(dir: &tempfile::TempDir) -> Store {
    Store::new(AppPaths::under(dir.path()), Config::default().limits)
}

#[test]
fn the_library_view_survives_a_reload() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let mut state = AppState::new(Config::default(), "Main");
    let view = LibraryView {
        open: true,
        rect: Some([10.0, 20.0, 640.0, 480.0]),
        maximised: true,
        whole_library: true,
        columns: vec![LibraryColumnWidth {
            column: LibraryColumn::Category,
            width: 90.0,
        }],
        sort: Some(LibrarySort {
            column: LibraryColumn::Kind,
            ascending: false,
        }),
        tree_width: Some(150.0),
        ..LibraryView::default()
    };
    apply(&mut state, Command::SetLibraryView(Box::new(view.clone()))).unwrap();
    save_all(&mut s, &state, 0.0);
    let loaded = store(&dir).load("Main");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    assert_eq!(loaded.state.library_view, view);
    // A change save writes it too.
    let mut changed = view;
    changed.open = false;
    apply(
        &mut state,
        Command::SetLibraryView(Box::new(changed.clone())),
    )
    .unwrap();
    save_all(&mut s, &state, 0.0);
    assert_eq!(store(&dir).load("Main").state.library_view, changed);
}

#[test]
fn a_bad_library_view_loads_the_default_with_a_warning() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let mut state = AppState::new(Config::default(), "Main");
    apply(
        &mut state,
        Command::SetLibraryView(Box::new(LibraryView {
            open: true,
            ..LibraryView::default()
        })),
    )
    .unwrap();
    save_all(&mut s, &state, 0.0);
    edit_kv(&s, "library_view", |v| {
        *v = serde_json::json!({"open": "yes", "rect": 3});
    });
    let loaded = store(&dir).load("Main");
    assert!(!loaded.state.library_view.open);
    assert_eq!(loaded.state.library_view.rect, None);
    assert!(!loaded.warnings.is_empty());
}

/// I1: an `f32` such as 268.8 is not exactly representable; a saved view
/// holding one loads as it is, with no warning (so the load is not a repair
/// and the database is not written back in full).
#[test]
fn a_library_view_with_fractional_sizes_loads_with_no_warning() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let mut state = AppState::new(Config::default(), "Main");
    let view = LibraryView {
        open: true,
        rect: Some([268.8, 151.2, 1344.0, 604.8]),
        columns: vec![LibraryColumnWidth {
            column: LibraryColumn::Category,
            width: 312.7,
        }],
        tree_width: Some(187.3),
        ..LibraryView::default()
    };
    apply(&mut state, Command::SetLibraryView(Box::new(view.clone()))).unwrap();
    save_all(&mut s, &state, 0.0);
    let loaded = store(&dir).load("Main");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    assert_eq!(loaded.state.library_view, view);
}

/// A saved size that parses but is not usable is reported by its field.
#[test]
fn a_library_view_with_an_unusable_size_names_the_field() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let state = AppState::new(Config::default(), "Main");
    save_all(&mut s, &state, 0.0);
    edit_kv(&s, "library_view", |v| {
        *v = serde_json::json!({"open": true, "rect": [1.5, 2.5, -3.0, 4.0], "tree_width": 120.4});
    });
    let loaded = store(&dir).load("Main");
    assert!(loaded.state.library_view.open);
    assert_eq!(loaded.warnings.len(), 1, "{:?}", loaded.warnings);
    assert!(
        loaded.warnings[0].contains("`rect`"),
        "{:?}",
        loaded.warnings
    );
}
