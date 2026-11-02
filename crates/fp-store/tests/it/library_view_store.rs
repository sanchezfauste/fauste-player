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
