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
            categories: Default::default(),
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
    let state = loaded.state;
    save_all(&mut s, &state, 0.0);
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

/// Sets `limits.max_state_file_bytes` in `config.json`.
fn set_state_limit(s: &Store, bytes: u64) {
    fs::create_dir_all(&s.paths().config_dir).unwrap();
    let doc = serde_json::json!({
        "schema_version": 1,
        "config": {"limits": {"max_state_file_bytes": bytes}}
    });
    fs::write(s.paths().config_file(), doc.to_string()).unwrap();
}

/// Pads a JSON file with trailing spaces up to `bytes`.
fn pad(path: &Path, bytes: usize) {
    let mut text = fs::read(path).unwrap();
    text.resize(bytes, b' ');
    fs::write(path, text).unwrap();
}

const MIB: usize = 1024 * 1024;

#[test]
fn an_oversized_document_stops_the_import_until_it_can_be_read() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = with_json(&dir);
    set_state_limit(&s, MIB as u64);
    pad(&s.paths().playlists_file(), MIB + 10);

    let first = s.load("Main");
    assert!(
        first.warnings.iter().any(|w| w.contains("playlists.json")),
        "{:?}",
        first.warnings
    );
    assert!(!s.paths().database_file().exists());
    assert!(!data_files(&s).iter().any(|n| n.ends_with(".imported")));
    assert!(s.paths().session_file().exists() && s.paths().carts_file().exists());
    // What was read is shown, and nothing is saved this session.
    assert_eq!(
        first.state.cartwall.pages,
        state_of_fixtures().cartwall.pages
    );
    let state = first.state;
    s.save_state(&state, |_| 0.0, std::time::Instant::now())
        .unwrap();
    assert!(!s.paths().database_file().exists());

    set_state_limit(&s, 4 * MIB as u64);
    let second = store(&dir).load("Main");
    assert!(second.warnings.is_empty(), "{:?}", second.warnings);
    assert_eq!(second.state.library.len(), 3);
    assert!(!s.paths().playlists_file().exists());
    assert!(s.paths().data_dir.join("playlists.json.imported").exists());
    assert!(s.paths().data_dir.join("session.json.imported").exists());
}

#[test]
fn documents_that_are_all_unreadable_leave_no_database_and_no_renames() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = with_json(&dir);
    set_state_limit(&s, MIB as u64);
    for name in ["playlists.json", "carts.json", "session.json"] {
        pad(&s.paths().data_dir.join(name), MIB + 10);
    }
    let loaded = s.load("Main");
    assert!(loaded.warnings.len() >= 3, "{:?}", loaded.warnings);
    assert!(!s.paths().database_file().exists());
    assert!(!data_files(&s).iter().any(|n| n.ends_with(".imported")));
}

#[test]
fn a_stale_write_ahead_log_next_to_the_json_files_is_ignored() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = with_json(&dir);
    fs::write(s.paths().data_dir.join("fauste.db-wal"), b"garbage garbage").unwrap();
    fs::write(s.paths().data_dir.join("fauste.db-shm"), b"garbage").unwrap();
    let loaded = s.load("Main");
    same_state(&loaded.state, &state_of_fixtures());
    assert_eq!(count(&s, "tracks"), 3);
}

#[test]
fn a_crash_between_the_marker_and_the_rename_is_redone() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = with_json(&dir);
    let data = s.paths().data_dir.clone();
    fs::write(data.join("fauste.db.import-pending"), b"playlists.json\n").unwrap();
    fs::write(data.join("fauste.db.importing"), b"half").unwrap();
    let loaded = s.load("Main");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    same_state(&loaded.state, &state_of_fixtures());
    let names = data_files(&s);
    assert!(
        !names
            .iter()
            .any(|n| n.contains("importing") || n.contains("pending"))
    );
    assert!(data.join("session.json.imported").exists());
}

#[test]
fn a_json_file_that_appears_after_an_interrupted_import_is_not_renamed() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = with_json(&dir);
    s.fail_import_rename = true;
    s.load("Main");
    // A file that was not part of the import turns up before the retry.
    let extra = s.paths().data_dir.join("playlists.json.bak2");
    assert!(!extra.exists());
    fs::write(&extra, b"{}").unwrap();
    s.fail_import_rename = false;
    s.load("Main");
    assert!(extra.exists());
    assert!(!s.paths().playlists_file().exists());
}

#[test]
fn legacy_played_marks_without_a_session_are_reported() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = with_json(&dir);
    fs::remove_file(s.paths().session_file()).unwrap();
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
    assert!(
        loaded.warnings.iter().any(|w| w.contains("played")),
        "{:?}",
        loaded.warnings
    );
}

#[test]
fn a_too_new_json_file_blocks_the_import() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = with_json(&dir);
    // No readable backup: the document would be lost behind the database.
    fs::remove_file(s.paths().data_dir.join("playlists.json.bak1")).unwrap();
    fs::write(
        s.paths().playlists_file(),
        r#"{"schema_version": 99, "library": []}"#,
    )
    .unwrap();
    let before = data_files(&s);
    let loaded = s.load("Main");
    assert!(
        loaded
            .warnings
            .iter()
            .any(|w| w.contains("playlists.json") && w.contains("not read")),
        "{:?}",
        loaded.warnings
    );
    // The loader may add a `.newer-` copy; the originals stay.
    assert!(!s.paths().database_file().exists());
    for name in &before {
        assert!(data_files(&s).contains(name), "{name} is gone");
    }
    assert!(!data_files(&s).iter().any(|n| n.ends_with(".imported")));
}

#[test]
fn an_unreadable_bak_blocks_the_import() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = with_json(&dir);
    // The main file is damaged, and its backup cannot be read (a directory).
    fs::write(s.paths().carts_file(), b"not json").unwrap();
    let bak = fp_store::backup_path(&s.paths().carts_file(), 1);
    fs::create_dir(&bak).unwrap();
    let loaded = s.load("Main");
    assert!(
        loaded
            .warnings
            .iter()
            .any(|w| w.contains("carts.json.bak1")),
        "{:?}",
        loaded.warnings
    );
    assert!(!s.paths().database_file().exists());
    assert!(bak.exists());
    assert!(!data_files(&s).iter().any(|n| n.ends_with(".imported")));
}
