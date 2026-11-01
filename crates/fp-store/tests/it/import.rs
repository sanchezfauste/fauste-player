#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};

use fp_model::{AppState, Config, RestoreParts};
use fp_store::docs::{CartsDoc, PlaylistsDoc, SessionDoc};
use fp_store::{AppPaths, Db, Store};

use crate::support::save_all;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/it/fixtures/json-0.12")
}

fn store(dir: &tempfile::TempDir) -> Store {
    Store::new(AppPaths::under(dir.path()), Config::default().limits)
}

/// Copies the fixtures into the data directory of `dir`.
fn with_json(dir: &tempfile::TempDir) -> Store {
    let s = store(dir);
    fs::create_dir_all(&s.paths().data_dir).unwrap();
    for entry in fs::read_dir(fixtures()).unwrap() {
        let entry = entry.unwrap();
        fs::copy(entry.path(), s.paths().data_dir.join(entry.file_name())).unwrap();
    }
    s
}

fn data_files(s: &Store) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(&s.paths().data_dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

/// The state the JSON documents give, built from the parsed documents the
/// way loading restores them.
fn state_of_fixtures() -> AppState {
    let read = |name: &str| fs::read(fixtures().join(name)).unwrap();
    let lists: PlaylistsDoc = serde_json::from_slice(&read("playlists.json")).unwrap();
    let session: SessionDoc = serde_json::from_slice(&read("session.json")).unwrap();
    let carts: CartsDoc = serde_json::from_slice(&read("carts.json")).unwrap();
    AppState::restore(
        RestoreParts {
            config: Config::default(),
            library: lists.library,
            playlists: lists.playlists,
            cart_pages: carts.pages,
            cartwall_session: session.cartwall,
            layout: session.layout,
            ids: lists.ids,
        },
        &session.players,
        "Main",
    )
    .0
}

fn same_state(a: &AppState, b: &AppState) {
    assert_eq!(a.library, b.library);
    assert_eq!(a.playlists, b.playlists);
    assert_eq!(a.cartwall.pages, b.cartwall.pages);
    assert_eq!(a.layout, b.layout);
    assert_eq!(a.sessions(|_| 0.0), b.sessions(|_| 0.0));
}

fn count(s: &Store, table: &str) -> i64 {
    let db = Db::open(&s.paths().database_file()).unwrap();
    db.conn()
        .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}

#[test]
fn import_moves_every_document_and_renames_the_files() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = with_json(&dir);
    let loaded = s.load("Main");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    let expected = state_of_fixtures();
    same_state(&loaded.state, &expected);
    // The fixtures carry what the import must not lose.
    let first = loaded.state.library.iter().next().unwrap();
    assert!(first.markers.cue_in.is_some() && first.markers.segue_start.is_some());
    assert!(loaded.state.layout.locked);
    // The pending start is stored (the restore then drops it from the
    // running state, so look at the database).
    let db = Db::open(&s.paths().database_file()).unwrap();
    let pending: i64 = db
        .conn()
        .query_row(
            "SELECT count(*) FROM players WHERE doc LIKE '%\"pending_start\":[%'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(pending, 1);
    drop(db);

    assert_eq!(
        data_files(&s),
        [
            "carts.json.imported",
            "fauste.db",
            "fauste.db-shm",
            "fauste.db-wal",
            "playlists.json.bak1.imported",
            "playlists.json.imported",
            "session.json.imported",
        ]
    );
    // The same state comes from the database alone.
    let again = store(&dir).load("Main");
    same_state(&again.state, &expected);
}

#[test]
fn a_second_start_does_not_import_again() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = with_json(&dir);
    s.load("Main");
    let tracks = count(&s, "tracks");
    // The renamed files are not documents any more.
    let mut again = store(&dir);
    let loaded = again.load("Main");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    assert_eq!(count(&again, "tracks"), tracks);
}

#[test]
fn a_corrupt_playlists_json_imports_its_backup() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = with_json(&dir);
    fs::write(s.paths().playlists_file(), b"{ not json").unwrap();
    let loaded = s.load("Main");
    assert!(
        loaded.warnings.iter().any(|w| w.contains("playlists.json")),
        "{:?}",
        loaded.warnings
    );
    same_state(&loaded.state, &state_of_fixtures());
    assert!(!s.paths().playlists_file().exists());
}

#[test]
fn an_interrupted_import_is_finished_on_the_next_start() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = with_json(&dir);
    s.fail_import_rename = true;
    let first = s.load("Main");
    assert!(first.warnings.iter().any(|w| w.contains("import")));
    // The database was committed, the JSON files are still there.
    assert!(s.paths().playlists_file().exists());
    let (tracks, entries) = (count(&s, "tracks"), count(&s, "entries"));
    assert!(tracks > 0 && entries > 0);

    s.fail_import_rename = false;
    let second = s.load("Main");
    assert!(second.warnings.is_empty(), "{:?}", second.warnings);
    same_state(&second.state, &state_of_fixtures());
    assert!(!s.paths().playlists_file().exists());
    assert!(s.paths().data_dir.join("playlists.json.imported").exists());
    assert_eq!(
        (count(&s, "tracks"), count(&s, "entries")),
        (tracks, entries)
    );
    assert!(!data_files(&s).iter().any(|n| n.contains("import-pending")));
}

#[test]
fn a_leftover_temporary_database_is_replaced_by_a_fresh_import() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = with_json(&dir);
    // A crash during the write left a partial file behind.
    fs::write(s.paths().data_dir.join("fauste.db.importing"), b"half").unwrap();
    let loaded = s.load("Main");
    same_state(&loaded.state, &state_of_fixtures());
    assert!(!s.paths().data_dir.join("fauste.db.importing").exists());
    assert_eq!(count(&s, "tracks"), 3);
}

#[test]
fn an_existing_database_is_never_overwritten_from_json() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let state = s.load("Main").state;
    save_all(&mut s, &state, 0.0);
    let before = data_files(&s);
    // Newer JSON files turn up next to the database.
    let mut s = with_json(&dir);
    let loaded = s.load("Main");
    assert_eq!(loaded.state.library.len(), 0);
    assert!(s.paths().playlists_file().exists());
    assert_eq!(count(&s, "tracks"), 0);
    assert!(!data_files(&s).iter().any(|n| n.ends_with(".imported")));
    assert!(before.iter().all(|n| data_files(&s).contains(n)));
}

#[test]
fn a_database_backup_alone_also_blocks_the_import() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let state = s.load("Main").state;
    save_all(&mut s, &state, 0.0);
    drop(s);
    let paths = AppPaths::under(dir.path());
    let db = paths.database_file();
    // Only a backup of the database is left.
    let bak = fp_store::backup_path(&db, 1);
    assert!(bak.exists());
    for suffix in ["", "-wal", "-shm"] {
        let mut name = db.clone().into_os_string();
        name.push(suffix);
        let _ = fs::remove_file(name);
    }
    let mut s = with_json(&dir);
    let loaded = s.load("Main");
    assert_eq!(loaded.state.library.len(), 0);
    assert!(s.paths().playlists_file().exists());
}

#[test]
fn nothing_readable_starts_empty_with_warnings() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    fs::create_dir_all(&s.paths().data_dir).unwrap();
    for name in ["playlists.json", "carts.json", "session.json"] {
        fs::write(s.paths().data_dir.join(name), b"garbage").unwrap();
    }
    let loaded = s.load("Main");
    assert_eq!(loaded.state.players.len(), 4);
    assert_eq!(loaded.state.library.len(), 0);
    assert!(loaded.warnings.len() >= 3, "{:?}", loaded.warnings);
    // The next start has nothing left to read (the files are quarantined).
    let again = s.load("Main");
    assert!(again.warnings.is_empty(), "{:?}", again.warnings);
}

#[test]
fn entries_and_carts_of_tracks_the_library_lacks_are_dropped_with_a_warning() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = with_json(&dir);
    let path = s.paths().playlists_file();
    let mut doc: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    // Remove the first track from the library, keeping its references.
    let gone = doc["library"].as_array_mut().unwrap().remove(0)["id"].clone();
    fs::write(&path, serde_json::to_vec(&doc).unwrap()).unwrap();
    let carts_path = s.paths().carts_file();
    let mut carts: serde_json::Value =
        serde_json::from_slice(&fs::read(&carts_path).unwrap()).unwrap();
    carts["pages"][0]["carts"][1]["track"] = gone.clone();
    fs::write(&carts_path, serde_json::to_vec(&carts).unwrap()).unwrap();

    let loaded = s.load("Main");
    assert!(loaded.warnings.iter().any(|w| w.contains("import")));
    assert_eq!(loaded.state.library.len(), 2);
    let list = loaded.state.playlists.first_id().unwrap();
    assert_eq!(loaded.state.playlists.get(list).unwrap().entries.len(), 2);
    assert!(loaded.state.cartwall.pages[0].carts[1].track.is_none());
    // The database accepted the result: a save works and reloads the same.
    let mut state = loaded.state;
    save_all(&mut s, &state, 0.0);
    state.layout.locked = false;
    let again = store(&dir).load("Main");
    assert_eq!(again.state.library, state.library);
    assert!(again.warnings.is_empty(), "{:?}", again.warnings);
}

#[test]
fn a_schema_1_playlists_file_counts_its_played_entries_for_every_player() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = with_json(&dir);
    let path = s.paths().playlists_file();
    let mut doc: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    doc["schema_version"] = 1.into();
    for list in doc["playlists"].as_array_mut().unwrap() {
        for entry in list["entries"].as_array_mut().unwrap() {
            let obj = entry.as_object_mut().unwrap();
            let played = obj.remove("played_by").is_some();
            obj.insert("played".into(), played.into());
        }
    }
    fs::write(&path, serde_json::to_vec(&doc).unwrap()).unwrap();

    let loaded = s.load("Main");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    let playlist = loaded.state.playlists.first_id().unwrap();
    let entries = &loaded.state.playlists.get(playlist).unwrap().entries;
    let players: Vec<_> = loaded.state.players.iter().map(|p| p.id).collect();
    assert!(players.iter().all(|p| entries[0].is_played_by(*p)));
    assert!(players.iter().all(|p| !entries[1].is_played_by(*p)));
}
