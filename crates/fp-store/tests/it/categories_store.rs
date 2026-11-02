#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Categories and each track's category in the database (schema 2).

use std::path::PathBuf;

use fp_model::{
    AppState, CartColor, Category, CategoryId, Command, Config, EntryId, PlaylistEntry, PlaylistId,
    Track, TrackId, TrackKind, apply,
};
use fp_store::{AppPaths, Store};

use crate::support::{edit_db, save_all};

fn store(dir: &tempfile::TempDir) -> Store {
    Store::new(AppPaths::under(dir.path()), Config::default().limits)
}

fn count(s: &Store, sql: &str) -> i64 {
    let db = fp_store::Db::open(&s.paths().database_file()).unwrap();
    db.conn().query_row(sql, [], |r| r.get(0)).unwrap()
}

#[test]
fn categories_and_track_categories_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let mut state = AppState::new(Config::default(), "Main");
    apply(
        &mut state,
        Command::CreateCategory {
            parent: None,
            name: "A".into(),
            color: None,
            kind: TrackKind::Music,
        },
    )
    .unwrap();
    let a = state.categories.iter().next().unwrap().id;
    apply(
        &mut state,
        Command::CreateCategory {
            parent: Some(a),
            name: "B".into(),
            color: Some(CartColor::Amber),
            kind: TrackKind::Ad,
        },
    )
    .unwrap();
    let b = state.categories.iter().nth(1).unwrap().id;
    let id = state.ids.track();
    let mut track = Track::new(id, PathBuf::from("/m/a.wav"));
    track.category = Some(b);
    state.library.insert(track);
    save_all(&mut s, &state, 0.0);

    let loaded = store(&dir).load("Main");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    assert_eq!(loaded.state.categories, state.categories);
    assert_eq!(loaded.state.library.get(id).unwrap().category, Some(b));
}

const SCHEMA_1: &str = "
CREATE TABLE kv (key TEXT PRIMARY KEY, value TEXT NOT NULL) WITHOUT ROWID;
CREATE TABLE tracks (id INTEGER PRIMARY KEY, path TEXT NOT NULL, doc TEXT NOT NULL);
CREATE TABLE playlists (id INTEGER PRIMARY KEY, position INTEGER NOT NULL, name TEXT NOT NULL);
CREATE TABLE entries (
  id INTEGER PRIMARY KEY,
  playlist INTEGER NOT NULL REFERENCES playlists(id) ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
  position INTEGER NOT NULL,
  track INTEGER NOT NULL REFERENCES tracks(id) DEFERRABLE INITIALLY DEFERRED,
  doc TEXT NOT NULL
);
CREATE INDEX entries_by_playlist ON entries(playlist, position);
CREATE INDEX entries_by_track ON entries(track);
CREATE TABLE cart_pages (id INTEGER PRIMARY KEY, position INTEGER NOT NULL, doc TEXT NOT NULL);
CREATE TABLE players (id INTEGER PRIMARY KEY, position INTEGER NOT NULL, doc TEXT NOT NULL);
";

fn track_doc(id: u64, path: &str) -> String {
    serde_json::to_string(&Track::new(TrackId(id), PathBuf::from(path))).unwrap()
}

fn entry_doc(id: u64, track: u64) -> String {
    serde_json::to_string(&PlaylistEntry::new(EntryId(id), TrackId(track))).unwrap()
}

fn write_schema_1(dir: &tempfile::TempDir, tracks: &[(u64, &str)], entries: &[(u64, u64)]) {
    let paths = AppPaths::under(dir.path());
    std::fs::create_dir_all(&paths.data_dir).unwrap();
    let conn = rusqlite::Connection::open(paths.database_file()).unwrap();
    conn.execute_batch(SCHEMA_1).unwrap();
    conn.pragma_update(None, "user_version", 1).unwrap();
    for (id, path) in tracks {
        conn.execute(
            "INSERT INTO tracks (id, path, doc) VALUES (?1, ?2, ?3)",
            rusqlite::params![*id as i64, path, track_doc(*id, path)],
        )
        .unwrap();
    }
    conn.execute(
        "INSERT INTO playlists (id, position, name) VALUES (50, 0, 'Main')",
        [],
    )
    .unwrap();
    for (n, (id, track)) in entries.iter().enumerate() {
        conn.execute(
            "INSERT INTO entries (id, playlist, position, track, doc) VALUES (?1, 50, ?2, ?3, ?4)",
            rusqlite::params![*id as i64, n as i64, *track as i64, entry_doc(*id, *track)],
        )
        .unwrap();
    }
}

#[test]
fn a_schema_1_database_opens_with_every_track_uncategorised() {
    let dir = tempfile::tempdir().unwrap();
    write_schema_1(&dir, &[(1, "/m/a.wav")], &[(100, 1)]);
    let mut s = store(&dir);
    let loaded = s.load("Main");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    assert_eq!(loaded.state.library.get(TrackId(1)).unwrap().category, None);
    assert!(loaded.state.categories.is_empty());
    assert_eq!(
        loaded
            .state
            .playlists
            .get(PlaylistId(50))
            .unwrap()
            .entries
            .len(),
        1
    );
    save_all(&mut s, &loaded.state, 0.0);
    assert_eq!(count(&s, "PRAGMA user_version"), 2);
}

#[test]
fn an_unknown_category_on_a_track_loads_uncategorised() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let mut state = AppState::new(Config::default(), "Main");
    let id = state.ids.track();
    state
        .library
        .insert(Track::new(id, PathBuf::from("/m/a.wav")));
    save_all(&mut s, &state, 0.0);
    edit_db(&s, |db| {
        db.conn()
            .execute("UPDATE tracks SET category = 999", [])
            .unwrap();
    });
    let loaded = store(&dir).load("Main");
    assert_eq!(loaded.state.library.get(id).unwrap().category, None);
    let hits: Vec<_> = loaded
        .warnings
        .iter()
        .filter(|w| w.contains("unknown category"))
        .collect();
    assert_eq!(hits.len(), 1, "{:?}", loaded.warnings);
}

#[test]
fn duplicate_paths_are_merged_on_reload() {
    let dir = tempfile::tempdir().unwrap();
    write_schema_1(
        &dir,
        &[(1, "/m/a.wav"), (2, "/m/a.wav")],
        &[(100, 1), (101, 2)],
    );
    let mut s = store(&dir);
    let loaded = s.load("Main");
    assert_eq!(loaded.state.library.len(), 1);
    let list = loaded.state.playlists.get(PlaylistId(50)).unwrap();
    assert_eq!(list.entries.len(), 2);
    assert_eq!(list.entries[0].track, list.entries[1].track);
    save_all(&mut s, &loaded.state, 0.0);
    assert_eq!(count(&s, "SELECT COUNT(*) FROM tracks"), 1);
    let reloaded = store(&dir).load("Main");
    assert_eq!(reloaded.state.library.len(), 1);
    assert_eq!(
        reloaded
            .state
            .playlists
            .get(PlaylistId(50))
            .unwrap()
            .entries
            .len(),
        2
    );
}

#[test]
fn a_damaged_category_tree_is_repaired_on_load() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let state = AppState::new(Config::default(), "Main");
    save_all(&mut s, &state, 0.0);
    let cat = |id: u64, parent: Option<u64>| Category {
        id: CategoryId(id),
        parent: parent.map(CategoryId),
        name: format!("c{id}"),
        color: None,
        kind: TrackKind::Music,
    };
    edit_db(&s, |db| {
        for (pos, c) in [cat(2, Some(1)), cat(1, None), cat(3, Some(77))]
            .iter()
            .enumerate()
        {
            db.conn()
                .execute(
                    "INSERT INTO categories (id, position, doc) VALUES (?1, ?2, ?3)",
                    rusqlite::params![c.id.0 as i64, pos as i64, serde_json::to_string(c).unwrap()],
                )
                .unwrap();
        }
    });
    let loaded = store(&dir).load("Main");
    let order: Vec<u64> = loaded.state.categories.iter().map(|c| c.id.0).collect();
    assert_eq!(order, [1, 2, 3]);
    assert_eq!(
        loaded.state.categories.get(CategoryId(3)).unwrap().parent,
        None
    );
    assert!(loaded.warnings.iter().any(|w| w.contains("unknown parent")));
}

fn root_category(state: &mut AppState, name: &str) -> CategoryId {
    apply(
        state,
        Command::CreateCategory {
            parent: None,
            name: name.into(),
            color: None,
            kind: TrackKind::Music,
        },
    )
    .unwrap();
    state.categories.iter().map(|c| c.id).max().unwrap()
}

/// An incremental save writes a track whose category alone changed, and
/// deletes the row of a deleted category.
#[test]
fn a_changed_track_category_and_a_deleted_category_are_saved() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let mut state = AppState::new(Config::default(), "Main");
    let a = root_category(&mut state, "A");
    let b = root_category(&mut state, "B");
    apply(
        &mut state,
        Command::AddToLibrary {
            paths: vec![PathBuf::from("/m/a.wav")],
            category: Some(a),
        },
    )
    .unwrap();
    let id = state.library.track_at(&PathBuf::from("/m/a.wav")).unwrap();
    save_all(&mut s, &state, 0.0);
    let category_of = |s: &Store| {
        count(
            s,
            &format!("SELECT category FROM tracks WHERE id = {}", id.0),
        )
    };
    assert_eq!(category_of(&s), a.0 as i64);

    apply(
        &mut state,
        Command::MoveTracksToCategory {
            tracks: vec![id],
            category: Some(b),
        },
    )
    .unwrap();
    save_all(&mut s, &state, 0.0);
    assert_eq!(category_of(&s), b.0 as i64);

    apply(&mut state, Command::DeleteCategory(a)).unwrap();
    save_all(&mut s, &state, 0.0);
    let rows = |s: &Store, c: CategoryId| {
        count(
            s,
            &format!("SELECT COUNT(*) FROM categories WHERE id = {}", c.0),
        )
    };
    assert_eq!((rows(&s, a), rows(&s, b)), (0, 1));
    assert_eq!(category_of(&s), b.0 as i64);
}
