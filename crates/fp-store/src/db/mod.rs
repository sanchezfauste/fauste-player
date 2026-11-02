//! The SQLite database of the library: opening, integrity check and
//! schema migrations.

mod backup;
mod schema;
pub mod snapshot;

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::{Connection, ErrorCode, OpenFlags, TransactionBehavior};

use crate::atomic::{backup_path, unused_sibling};

use schema::MIGRATIONS;
pub use schema::SCHEMA_VERSION;

/// Why a database could not be opened.
#[derive(Debug, thiserror::Error)]
pub enum OpenError {
    /// The file could not be created or read.
    #[error("cannot open the database: {0}")]
    Io(String),
    /// The file is not a database or fails the integrity check.
    #[error("the database is corrupt: {0}")]
    Corrupt(String),
    /// Another connection holds the file locked: it is in use, not damaged,
    /// so it must not be set aside.
    #[error("the database is busy: {0}")]
    Busy(String),
    /// The database was written by a newer version of the application.
    #[error("the database has schema {0}, newer than this version supports")]
    Newer(u32),
}

/// Runs the integrity check and returns the schema version, refusing a
/// newer one. It does not write to the database.
fn check(conn: &Connection) -> Result<u32, OpenError> {
    let result: String = conn
        .query_row("PRAGMA quick_check", [], |r| r.get(0))
        .map_err(classify)?;
    if result != "ok" {
        return Err(OpenError::Corrupt(result));
    }
    let version: u32 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .map_err(classify)?;
    if version > SCHEMA_VERSION {
        return Err(OpenError::Newer(version));
    }
    Ok(version)
}

fn classify(e: rusqlite::Error) -> OpenError {
    match e.sqlite_error_code() {
        Some(ErrorCode::NotADatabase | ErrorCode::DatabaseCorrupt) => {
            OpenError::Corrupt(e.to_string())
        }
        Some(ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked) => OpenError::Busy(e.to_string()),
        _ => OpenError::Io(e.to_string()),
    }
}

/// How long a connection waits for a lock held by another one.
#[cfg(not(test))]
const BUSY_TIMEOUT: Duration = Duration::from_secs(5);
/// Short in unit tests, which hold locks on purpose.
#[cfg(test)]
const BUSY_TIMEOUT: Duration = Duration::from_millis(50);

/// An open library database.
pub struct Db {
    pub(crate) conn: Connection,
    pub(crate) path: PathBuf,
    /// Not a file: nothing persists and `path` is only a name.
    in_memory: bool,
}

impl Db {
    /// Opens (creating it when missing) the database at `path`, checks its
    /// integrity and brings its schema up to [`SCHEMA_VERSION`].
    pub fn open(path: &Path) -> Result<Db, OpenError> {
        let mut checked = None;
        if path.exists() {
            // Look at the file read-only first: opening it for writing can
            // already delete a leftover `-wal`, and bytes we cannot read must
            // reach `open_with_fallback` unchanged.
            let probe = Connection::open_with_flags(
                path,
                OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
            )
            .map_err(classify)?;
            probe.busy_timeout(BUSY_TIMEOUT).map_err(classify)?;
            checked = Some(check(&probe)?);
        }
        let conn = Connection::open(path).map_err(classify)?;
        Self::prepare(conn, path, true, checked)
    }

    /// A new in-memory database (nothing persists), for when no file can be
    /// used. `path` is only the name the database stands for.
    pub(crate) fn in_memory(path: &Path) -> Result<Db, OpenError> {
        let conn = Connection::open_in_memory().map_err(classify)?;
        Self::prepare(conn, path, false, None)
    }

    /// `checked` is the schema version when the file was already checked
    /// (by the read-only probe of [`Db::open`]); otherwise it is checked here.
    fn prepare(
        mut conn: Connection,
        path: &Path,
        wal: bool,
        checked: Option<u32>,
    ) -> Result<Db, OpenError> {
        // Wait this long for another connection's lock instead of failing
        // at once with "database is locked".
        conn.busy_timeout(BUSY_TIMEOUT).map_err(classify)?;
        conn.pragma_update(None, "foreign_keys", true)
            .map_err(classify)?;
        conn.pragma_update(None, "synchronous", "NORMAL")
            .map_err(classify)?;
        let version = match checked {
            Some(version) => version,
            None => check(&conn)?,
        };
        // Only now, with the file known to be ours, may it be written to.
        if wal {
            let mode: String = conn
                .query_row("PRAGMA journal_mode=WAL", [], |r| r.get(0))
                .map_err(classify)?;
            if !mode.eq_ignore_ascii_case("wal") {
                return Err(OpenError::Io(format!("journal mode is {mode}, not wal")));
            }
        }
        for (index, sql) in MIGRATIONS.iter().enumerate().skip(version as usize) {
            let target = u32::try_from(index + 1).unwrap_or(u32::MAX);
            let tx = conn
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(classify)?;
            tx.execute_batch(sql).map_err(classify)?;
            tx.pragma_update(None, "user_version", target)
                .map_err(classify)?;
            tx.commit().map_err(classify)?;
        }
        Ok(Db {
            conn,
            path: path.to_path_buf(),
            in_memory: !wal,
        })
    }

    /// Whether this is the in-memory stand-in for the file at its path.
    pub fn is_in_memory(&self) -> bool {
        self.in_memory
    }

    /// The connection (tests).
    #[doc(hidden)]
    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    /// A pragma's integer value (tests).
    #[doc(hidden)]
    pub fn pragma_i64(&self, name: &str) -> i64 {
        self.conn
            .query_row(&format!("PRAGMA {name}"), [], |r| r.get(0))
            .unwrap_or(-1)
    }

    /// A pragma's text value (tests).
    #[doc(hidden)]
    pub fn pragma_text(&self, name: &str) -> String {
        self.conn
            .query_row(&format!("PRAGMA {name}"), [], |r| r.get(0))
            .unwrap_or_default()
    }

    /// The connection, for transactions (tests).
    #[doc(hidden)]
    pub fn conn_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }
}

/// Which file [`open_with_fallback`] ended up using.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbSource {
    /// The main file (also a first start, when it did not exist).
    Main,
    /// The `n`-th backup (`<path>.bak<n>`), copied over the main file.
    Backup(usize),
    /// A new empty database: the main file and every backup were unusable.
    Fresh,
}

/// The result of [`open_with_fallback`].
pub struct Opened {
    pub db: Db,
    pub source: DbSource,
    /// One line per step that did not go as planned.
    pub warnings: Vec<String>,
}

/// The `-wal` and `-shm` files SQLite keeps beside a database.
fn side_files(path: &Path) -> [PathBuf; 2] {
    ["-wal", "-shm"].map(|suffix| {
        let mut name = path.as_os_str().to_owned();
        name.push(suffix);
        PathBuf::from(name)
    })
}

/// Renames `path` and its side files to `<path>.<kind>-<stamp>`. The side
/// files move first and the main file last, so a crash never leaves a
/// sidecar beside a missing main file; if any rename fails the ones already
/// done are undone. Returns whether `path` is now free; nothing is ever
/// deleted or overwritten.
fn set_aside(path: &Path, kind: &str, warnings: &mut Vec<String>) -> bool {
    set_aside_with(path, kind, warnings, |from, to| fs::rename(from, to))
}

/// [`set_aside`] with the rename as a parameter, so a test can make one
/// of them fail.
fn set_aside_with(
    path: &Path,
    kind: &str,
    warnings: &mut Vec<String>,
    rename: impl Fn(&Path, &Path) -> std::io::Result<()>,
) -> bool {
    let target = free_target(path, kind);
    let mut moves: Vec<(PathBuf, PathBuf)> = side_files(path)
        .into_iter()
        .zip(side_files(&target))
        .filter(|(from, _)| from.exists())
        .collect();
    if path.exists() {
        moves.push((path.to_path_buf(), target.clone()));
    }
    let mut done: Vec<&(PathBuf, PathBuf)> = Vec::new();
    for pair in &moves {
        if let Err(e) = rename(&pair.0, &pair.1) {
            warnings.push(format!("{}: could not set aside: {e}", pair.0.display()));
            for (from, to) in done.into_iter().rev() {
                if let Err(e) = rename(to, from) {
                    warnings.push(format!("{}: could not restore: {e}", from.display()));
                }
            }
            return false;
        }
        done.push(pair);
    }
    warnings.push(format!(
        "{}: set aside as {}",
        path.display(),
        target.display()
    ));
    true
}

/// A name for `path` with a `<kind>-<stamp>` suffix such that neither it nor
/// its side files exist.
fn free_target(path: &Path, kind: &str) -> PathBuf {
    let first = unused_sibling(path, kind);
    let free = |t: &Path| !t.exists() && side_files(t).iter().all(|s| !s.exists());
    if free(&first) {
        return first;
    }
    (2u32..)
        .map(|n| {
            let mut name = first.as_os_str().to_owned();
            name.push(format!("-{n}"));
            PathBuf::from(name)
        })
        .find(|t| free(t))
        .unwrap_or(first)
}

/// Deletes the copy of a backup that `open_with_fallback` made at `path`.
fn discard_copy(path: &Path) {
    let _ = fs::remove_file(path);
    for side in side_files(path) {
        let _ = fs::remove_file(side);
    }
}

/// Opens the database at `path`; when that fails, falls back to
/// `<path>.bak1` … `<path>.bak<backups>` in order, and finally to a new
/// empty database. It never fails and never destroys data: an unusable main
/// file is renamed (with its side files) to `*.corrupt-<stamp>`, or to
/// `*.newer-<stamp>` when a newer version wrote it; a backup is copied, never
/// moved. When the main file cannot be set aside, nothing is written over it
/// and the result is an in-memory database. Every deviation leaves a warning.
///
/// The only error is the in-memory database itself failing to open (SQLite
/// out of memory), since the code may not panic.
pub fn open_with_fallback(path: &Path, backups: usize) -> Result<Opened, OpenError> {
    let mut warnings = Vec::new();
    let fail = |warnings: &mut Vec<String>, what: &str, e: &OpenError| {
        warnings.push(format!("{}: {what}: {e}", path.display()));
    };
    if !path.exists() && (1..=backups).any(|n| backup_path(path, n).exists()) {
        // A new file here would rotate the backups away: restore instead.
        warnings.push(format!(
            "{}: the database file is missing; restoring from a backup",
            path.display()
        ));
        // A `-wal` or `-shm` left beside the missing file belongs to some
        // other database and would be replayed onto the restored copy.
        if side_files(path).iter().any(|f| f.exists()) && !set_aside(path, "corrupt", &mut warnings)
        {
            return in_memory(path, warnings);
        }
        return from_backups(path, backups, warnings);
    }
    match Db::open(path) {
        Ok(db) => {
            return Ok(Opened {
                db,
                source: DbSource::Main,
                warnings,
            });
        }
        Err(e) => {
            fail(&mut warnings, "unusable", &e);
            if matches!(e, OpenError::Busy(_)) {
                // In use by someone else: leave it alone, write nothing.
                warnings.push(format!(
                    "{}: the database is in use by another process (is another copy running?)",
                    path.display()
                ));
                return in_memory(path, warnings);
            }
            if path.exists() {
                // Corrupt, newer or unreadable: keep the bytes, free the name.
                let kind = if matches!(e, OpenError::Newer(_)) {
                    "newer"
                } else {
                    "corrupt"
                };
                if !set_aside(path, kind, &mut warnings) {
                    return in_memory(path, warnings);
                }
            }
        }
    }
    from_backups(path, backups, warnings)
}

/// The main file is free: restores the first usable backup over it, or
/// creates a new database.
fn from_backups(
    path: &Path,
    backups: usize,
    mut warnings: Vec<String>,
) -> Result<Opened, OpenError> {
    let fail = |warnings: &mut Vec<String>, what: &str, e: &OpenError| {
        warnings.push(format!("{}: {what}: {e}", path.display()));
    };
    // A backup that opened but cannot be read in full: the data may be
    // there, so the session must not start a new database over it.
    let mut incomplete = false;
    // Backups that cannot be used: kept under another name once the
    // session has a database, so the rotation never deletes them.
    let mut unusable: Vec<(PathBuf, &str)> = Vec::new();
    for n in 1..=backups {
        let backup = backup_path(path, n);
        if !backup.exists() {
            continue;
        }
        if let Err(e) = fs::copy(&backup, path) {
            warnings.push(format!("{}: could not copy: {e}", backup.display()));
            discard_copy(path);
            unusable.push((backup, "corrupt"));
            continue;
        }
        match Db::open(path) {
            Ok(db) if !snapshot::read_checked(&db).complete => {
                drop(db);
                incomplete = true;
                warnings.push(format!(
                    "{}: backup unusable: a table cannot be read",
                    backup.display()
                ));
                discard_copy(path);
                unusable.push((backup, "corrupt"));
            }
            Ok(db) => {
                // The ones skipped on the way would be rotated away.
                for (bad, kind) in &unusable {
                    // A failure has already left a warning.
                    set_aside(bad, kind, &mut warnings);
                }
                warnings.push(format!("restored from {}", backup.display()));
                return Ok(Opened {
                    db,
                    source: DbSource::Backup(n),
                    warnings,
                });
            }
            Err(e) => {
                warnings.push(format!("{}: backup unusable: {e}", backup.display()));
                discard_copy(path);
                let kind = if matches!(e, OpenError::Newer(_)) {
                    "newer"
                } else {
                    "corrupt"
                };
                unusable.push((backup, kind));
            }
        }
    }
    if incomplete {
        return in_memory(path, warnings);
    }
    match Db::open(path) {
        Ok(db) => {
            // Every backup failed and the next one rotates them: keep
            // what could not be read.
            for (backup, kind) in &unusable {
                // A failure has already left a warning.
                set_aside(backup, kind, &mut warnings);
            }
            Ok(Opened {
                db,
                source: DbSource::Fresh,
                warnings,
            })
        }
        Err(e) => {
            fail(&mut warnings, "could not create a new database", &e);
            in_memory(path, warnings)
        }
    }
}

/// Called when the open database at `path` (already closed by the caller)
/// turned out to be unreadable in part: sets the file aside as
/// `*.corrupt-<stamp>` with its side files and goes on through the
/// backups, like [`open_with_fallback`]. When the file cannot be set aside
/// nothing is written over it: the result is an in-memory database.
pub(crate) fn reopen_after_unreadable(
    path: &Path,
    backups: usize,
    mut warnings: Vec<String>,
) -> Result<Opened, OpenError> {
    if !set_aside(path, "corrupt", &mut warnings) {
        return in_memory(path, warnings);
    }
    from_backups(path, backups, warnings)
}

/// The last resort: an in-memory database that stands for `path`.
fn in_memory(path: &Path, mut warnings: Vec<String>) -> Result<Opened, OpenError> {
    warnings.push(format!(
        "{}: using a temporary in-memory database; changes are not saved this session",
        path.display()
    ));
    let db = Db::in_memory(path)?;
    Ok(Opened {
        db,
        source: DbSource::Fresh,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn busy_and_locked_are_not_damage() {
        for code in [rusqlite::ffi::SQLITE_BUSY, rusqlite::ffi::SQLITE_LOCKED] {
            let e = rusqlite::Error::SqliteFailure(rusqlite::ffi::Error::new(code), None);
            assert!(matches!(classify(e), OpenError::Busy(_)));
        }
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn a_locked_database_is_left_alone() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("fauste.db");
        drop(Db::open(&path).unwrap());
        let holder = Connection::open(&path).unwrap();
        holder
            .execute_batch(
                "PRAGMA locking_mode=EXCLUSIVE; BEGIN EXCLUSIVE; \
                 INSERT INTO kv (key, value) VALUES ('a', 'b');",
            )
            .unwrap();
        let before = fs::read(&path).unwrap();
        let opened = open_with_fallback(&path, 3).unwrap();
        assert_eq!(opened.source, DbSource::Fresh);
        assert!(!opened.warnings.is_empty());
        assert_eq!(fs::read(&path).unwrap(), before);
        assert!(fs::read_dir(dir.path()).unwrap().all(|e| {
            !e.unwrap()
                .file_name()
                .to_string_lossy()
                .contains(".corrupt-")
        }));
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn set_aside_rolls_back_when_a_side_file_cannot_move() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("fauste.db");
        fs::write(&path, b"main").unwrap();
        let [wal, shm] = side_files(&path);
        fs::write(&wal, b"wal").unwrap();
        fs::write(&shm, b"shm").unwrap();
        let target = free_target(&path, "corrupt");
        let mut warnings = Vec::new();
        // The `-shm` rename fails after the `-wal` one succeeded (a failure
        // an operating system cannot be made to give on demand).
        let moved = set_aside_with(&path, "corrupt", &mut warnings, |from, to| {
            if from.to_string_lossy().ends_with("-shm") {
                Err(std::io::Error::other("injected"))
            } else {
                fs::rename(from, to)
            }
        });
        assert!(!moved);
        assert_eq!(fs::read(&path).unwrap(), b"main");
        assert_eq!(fs::read(&wal).unwrap(), b"wal");
        assert_eq!(fs::read(&shm).unwrap(), b"shm");
        assert!(!side_files(&target)[0].exists(), "the -wal move was undone");
        assert!(!target.exists());
        assert!(warnings.iter().any(|w| w.contains("could not set aside")));
    }
}
