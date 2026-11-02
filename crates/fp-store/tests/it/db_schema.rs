use fp_store::{Db, OpenError, SCHEMA_VERSION};
use tempfile::tempdir;

#[test]
fn a_new_database_gets_schema_2_wal_and_foreign_keys() {
    let dir = tempdir().unwrap();
    let db = Db::open(&dir.path().join("fauste.db")).unwrap();
    assert_eq!(SCHEMA_VERSION, 2);
    assert_eq!(db.pragma_i64("user_version"), 2);
    assert_eq!(db.pragma_text("journal_mode"), "wal");
    assert_eq!(db.pragma_i64("foreign_keys"), 1);
}

#[test]
fn reopening_keeps_the_schema() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("fauste.db");
    drop(Db::open(&path).unwrap());
    let db = Db::open(&path).unwrap();
    assert_eq!(db.pragma_i64("user_version"), 2);
}

#[test]
fn a_newer_schema_is_refused() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("fauste.db");
    drop(Db::open(&path).unwrap());
    let conn = rusqlite::Connection::open(&path).unwrap();
    conn.pragma_update(None, "user_version", 3).unwrap();
    drop(conn);
    assert!(matches!(Db::open(&path), Err(OpenError::Newer(3))));
}

#[test]
fn garbage_is_corrupt() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("fauste.db");
    std::fs::write(&path, vec![0xAB; 4096]).unwrap();
    assert!(matches!(Db::open(&path), Err(OpenError::Corrupt(_))));
}

#[test]
fn an_entry_needs_its_track_at_commit() {
    let dir = tempdir().unwrap();
    let mut db = Db::open(&dir.path().join("fauste.db")).unwrap();
    let tx = db.conn_mut().transaction().unwrap();
    tx.execute(
        "INSERT INTO playlists (id, position, name) VALUES (1, 0, 'p')",
        [],
    )
    .unwrap();
    tx.execute(
        "INSERT INTO entries (id, playlist, position, track, doc) VALUES (1, 1, 0, 99, '{}')",
        [],
    )
    .unwrap();
    assert!(tx.commit().is_err());
}

#[test]
fn a_missing_parent_directory_is_an_io_error() {
    let dir = tempdir().unwrap();
    let result = Db::open(&dir.path().join("missing/fauste.db"));
    assert!(matches!(result, Err(OpenError::Io(_))));
}
