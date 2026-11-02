#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::Path;

use fp_store::db::{DbSource, open_with_fallback};
use fp_store::{Db, backup_path};
use tempfile::tempdir;

fn names(dir: &Path) -> Vec<String> {
    let mut v: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    v.sort();
    v
}

fn has_prefix(dir: &Path, prefix: &str) -> bool {
    names(dir).iter().any(|n| n.starts_with(prefix))
}

fn set_marker(db: &Db, value: &str) {
    db.conn()
        .execute(
            "INSERT OR REPLACE INTO kv (key, value) VALUES ('marker', ?1)",
            [value],
        )
        .unwrap();
}

fn marker(db: &Db) -> Option<String> {
    db.conn()
        .query_row("SELECT value FROM kv WHERE key = 'marker'", [], |r| {
            r.get(0)
        })
        .ok()
}

/// A valid database file holding `value`, written at `path`.
fn make_db(path: &Path, value: &str) {
    let db = Db::open(path).unwrap();
    set_marker(&db, value);
}

#[test]
fn a_missing_database_is_created_silently() {
    let dir = tempdir().unwrap();
    let opened = open_with_fallback(&dir.path().join("fauste.db"), 3).unwrap();
    assert_eq!(opened.source, DbSource::Main);
    assert!(opened.warnings.is_empty());
}

#[test]
fn a_corrupt_database_falls_back_to_its_newest_backup() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("fauste.db");
    make_db(&backup_path(&path, 1), "newest");
    make_db(&backup_path(&path, 2), "older");
    fs::write(&path, vec![0xAB; 4096]).unwrap();
    let opened = open_with_fallback(&path, 3).unwrap();
    assert_eq!(opened.source, DbSource::Backup(1));
    assert_eq!(marker(&opened.db).as_deref(), Some("newest"));
    assert!(!opened.warnings.is_empty());
    assert!(has_prefix(dir.path(), "fauste.db.corrupt-"));
    assert!(backup_path(&path, 1).exists());
    assert!(backup_path(&path, 2).exists());
}

#[test]
fn an_unusable_backup_is_skipped_and_set_aside() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("fauste.db");
    fs::write(backup_path(&path, 1), vec![0xCD; 4096]).unwrap();
    make_db(&backup_path(&path, 2), "older");
    fs::write(&path, vec![0xAB; 4096]).unwrap();
    let opened = open_with_fallback(&path, 3).unwrap();
    assert_eq!(opened.source, DbSource::Backup(2));
    assert_eq!(marker(&opened.db).as_deref(), Some("older"));
    assert!(!backup_path(&path, 1).exists(), "its name is free");
    let aside: Vec<_> = names(dir.path())
        .into_iter()
        .filter(|n| n.starts_with("fauste.db.bak1.corrupt-"))
        .collect();
    assert_eq!(aside.len(), 1, "{aside:?}");
    assert_eq!(
        fs::read(dir.path().join(&aside[0])).unwrap(),
        vec![0xCD; 4096],
        "the bytes are kept"
    );
}

#[test]
fn a_newer_database_is_kept_aside() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("fauste.db");
    make_db(&path, "future");
    {
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.pragma_update(None, "user_version", 3).unwrap();
    }
    let before = fs::read(&path).unwrap();
    let opened = open_with_fallback(&path, 3).unwrap();
    assert_eq!(opened.source, DbSource::Fresh);
    assert_eq!(marker(&opened.db), None);
    assert!(!opened.warnings.is_empty());
    let kept = names(dir.path())
        .into_iter()
        .find(|n| n.starts_with("fauste.db.newer-") && !n.ends_with("-wal"))
        .unwrap();
    assert_eq!(fs::read(dir.path().join(kept)).unwrap(), before);
}

#[test]
fn without_anything_readable_it_starts_fresh() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("fauste.db");
    fs::write(&path, vec![0xAB; 4096]).unwrap();
    fs::write(backup_path(&path, 1), vec![0xCD; 4096]).unwrap();
    let opened = open_with_fallback(&path, 3).unwrap();
    assert_eq!(opened.source, DbSource::Fresh);
    assert_eq!(opened.db.pragma_i64("user_version"), 2);
    assert!(path.exists());
}

#[test]
fn an_unopenable_directory_gives_an_in_memory_database() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("missing/fauste.db");
    let opened = open_with_fallback(&path, 3).unwrap();
    assert_eq!(opened.source, DbSource::Fresh);
    assert_eq!(opened.db.pragma_i64("user_version"), 2);
    assert!(!opened.warnings.is_empty());
    assert!(!path.exists());
}

#[test]
fn backups_rotate_up_to_the_limit() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("fauste.db");
    let db = Db::open(&path).unwrap();
    for n in 1..=4 {
        set_marker(&db, &format!("v{n}"));
        db.backup_rotating(3).unwrap();
    }
    for (n, expected) in [(1, "v4"), (2, "v3"), (3, "v2")] {
        let copy = Db::open(&backup_path(&path, n)).unwrap();
        assert_eq!(marker(&copy).as_deref(), Some(expected));
    }
    assert!(!backup_path(&path, 4).exists());
    assert!(!dir.path().join("fauste.db.bak-tmp").exists());
}

#[test]
fn zero_backups_writes_none() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("fauste.db");
    let db = Db::open(&path).unwrap();
    db.backup_rotating(0).unwrap();
    assert!(!names(dir.path()).iter().any(|n| n.contains(".bak")));
}

#[test]
fn a_stale_temporary_backup_is_replaced() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("fauste.db");
    fs::write(dir.path().join("fauste.db.bak-tmp"), b"stale").unwrap();
    let db = Db::open(&path).unwrap();
    db.backup_rotating(1).unwrap();
    assert!(Db::open(&backup_path(&path, 1)).is_ok());
    assert!(!dir.path().join("fauste.db.bak-tmp").exists());
}

#[test]
fn side_files_travel_with_the_main_file_and_keep_their_bytes() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("fauste.db");
    fs::write(&path, vec![0xAB; 4096]).unwrap();
    fs::write(dir.path().join("fauste.db-wal"), vec![0x11; 64]).unwrap();
    fs::write(dir.path().join("fauste.db-shm"), vec![0x22; 64]).unwrap();
    let opened = open_with_fallback(&path, 0).unwrap();
    assert_eq!(opened.source, DbSource::Fresh);
    let aside = names(dir.path())
        .into_iter()
        .find(|n| {
            n.starts_with("fauste.db.corrupt-") && !n.ends_with("-wal") && !n.ends_with("-shm")
        })
        .unwrap();
    assert_eq!(fs::read(dir.path().join(&aside)).unwrap(), vec![0xAB; 4096]);
    assert_eq!(
        fs::read(dir.path().join(format!("{aside}-wal"))).unwrap(),
        vec![0x11; 64]
    );
    // The -shm is SQLite's rebuildable shared-memory index, which it
    // rewrites when it looks at a file: it only has to travel along.
    assert!(dir.path().join(format!("{aside}-shm")).exists());
}

#[test]
fn an_existing_side_file_target_is_never_overwritten() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("fauste.db");
    fs::write(&path, vec![0xAB; 4096]).unwrap();
    fs::write(dir.path().join("fauste.db-wal"), vec![0x11; 64]).unwrap();
    // Occupy the wal name of every candidate stamp the next second or two.
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    for s in stamp - 1..=stamp + 2 {
        fs::write(
            dir.path().join(format!("fauste.db.corrupt-{s}-wal")),
            b"precious",
        )
        .unwrap();
    }
    open_with_fallback(&path, 0).unwrap();
    for s in stamp - 1..=stamp + 2 {
        assert_eq!(
            fs::read(dir.path().join(format!("fauste.db.corrupt-{s}-wal"))).unwrap(),
            b"precious"
        );
    }
    assert!(
        names(dir.path())
            .iter()
            .any(|n| n.ends_with("-wal") && fs::read(dir.path().join(n)).unwrap() == vec![0x11; 64])
    );
}

#[test]
fn a_newer_database_in_rollback_journal_mode_is_not_modified() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("fauste.db");
    {
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.execute_batch("CREATE TABLE t (x); PRAGMA user_version = 3;")
            .unwrap();
    }
    let before = fs::read(&path).unwrap();
    open_with_fallback(&path, 0).unwrap();
    let kept = names(dir.path())
        .into_iter()
        .find(|n| n.starts_with("fauste.db.newer-"))
        .unwrap();
    assert_eq!(fs::read(dir.path().join(kept)).unwrap(), before);
}

#[test]
fn a_newer_backup_is_set_aside_as_newer() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("fauste.db");
    make_db(&backup_path(&path, 1), "future");
    {
        let conn = rusqlite::Connection::open(backup_path(&path, 1)).unwrap();
        conn.pragma_update(None, "user_version", 99).unwrap();
    }
    make_db(&backup_path(&path, 2), "older");
    fs::write(&path, vec![0xAB; 4096]).unwrap();
    let opened = open_with_fallback(&path, 3).unwrap();
    assert_eq!(opened.source, DbSource::Backup(2));
    let all = names(dir.path());
    assert!(!backup_path(&path, 1).exists());
    assert!(
        all.iter().any(|n| n.starts_with("fauste.db.bak1.newer-")),
        "{all:?}"
    );
    assert!(
        !all.iter().any(|n| n.starts_with("fauste.db.bak1.corrupt-")),
        "{all:?}"
    );
}
