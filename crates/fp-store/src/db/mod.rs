//! The SQLite database of the library: opening, integrity check and
//! schema migrations.

mod backup;
mod schema;

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::{Connection, ErrorCode, TransactionBehavior};

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
    /// The database was written by a newer version of the application.
    #[error("the database has schema {0}, newer than this version supports")]
    Newer(u32),
}

fn classify(e: rusqlite::Error) -> OpenError {
    match e.sqlite_error_code() {
        Some(ErrorCode::NotADatabase | ErrorCode::DatabaseCorrupt) => {
            OpenError::Corrupt(e.to_string())
        }
        _ => OpenError::Io(e.to_string()),
    }
}

/// How long a connection waits for a lock held by another one.
const BUSY_TIMEOUT: Duration = Duration::from_secs(5);

/// An open library database.
pub struct Db {
    pub(crate) conn: Connection,
    pub(crate) path: PathBuf,
}

impl Db {
    /// Opens (creating it when missing) the database at `path`, checks its
    /// integrity and brings its schema up to [`SCHEMA_VERSION`].
    pub fn open(path: &Path) -> Result<Db, OpenError> {
        let conn = Connection::open(path).map_err(classify)?;
        Self::prepare(conn, path, true)
    }

    /// A new in-memory database (nothing persists), for when no file can be
    /// used. `path` is only the name the database stands for.
    fn in_memory(path: &Path) -> Result<Db, OpenError> {
        let conn = Connection::open_in_memory().map_err(classify)?;
        Self::prepare(conn, path, false)
    }

    fn prepare(mut conn: Connection, path: &Path, wal: bool) -> Result<Db, OpenError> {
        // Wait this long for another connection's lock instead of failing
        // at once with "database is locked".
        conn.busy_timeout(BUSY_TIMEOUT).map_err(classify)?;
        if wal {
            let mode: String = conn
                .query_row("PRAGMA journal_mode=WAL", [], |r| r.get(0))
                .map_err(classify)?;
            if !mode.eq_ignore_ascii_case("wal") {
                return Err(OpenError::Io(format!("journal mode is {mode}, not wal")));
            }
        }
        conn.pragma_update(None, "foreign_keys", true)
            .map_err(classify)?;
        conn.pragma_update(None, "synchronous", "NORMAL")
            .map_err(classify)?;
        let check: String = conn
            .query_row("PRAGMA quick_check", [], |r| r.get(0))
            .map_err(classify)?;
        if check != "ok" {
            return Err(OpenError::Corrupt(check));
        }
        let version: u32 = conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .map_err(classify)?;
        if version > SCHEMA_VERSION {
            return Err(OpenError::Newer(version));
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
        })
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

/// Renames `path` and its side files to `<path>.<kind>-<stamp>`. Returns
/// whether `path` is now free; nothing is ever deleted.
fn set_aside(path: &Path, kind: &str, warnings: &mut Vec<String>) -> bool {
    let target = unused_sibling(path, kind);
    if let Err(e) = fs::rename(path, &target) {
        warnings.push(format!("{}: could not set aside: {e}", path.display()));
        return false;
    }
    warnings.push(format!(
        "{}: set aside as {}",
        path.display(),
        target.display()
    ));
    let mut ok = true;
    for (from, to) in side_files(path).into_iter().zip(side_files(&target)) {
        if from.exists()
            && let Err(e) = fs::rename(&from, &to)
        {
            warnings.push(format!("{}: could not set aside: {e}", from.display()));
            ok = false;
        }
    }
    ok
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
    for n in 1..=backups {
        let backup = backup_path(path, n);
        if !backup.exists() {
            continue;
        }
        if let Err(e) = fs::copy(&backup, path) {
            warnings.push(format!("{}: could not copy: {e}", backup.display()));
            discard_copy(path);
            continue;
        }
        match Db::open(path) {
            Ok(db) => {
                warnings.push(format!("restored from {}", backup.display()));
                return Ok(Opened {
                    db,
                    source: DbSource::Backup(n),
                    warnings,
                });
            }
            Err(e) => {
                fail(&mut warnings, "backup unusable", &e);
                discard_copy(path);
            }
        }
    }
    match Db::open(path) {
        Ok(db) => Ok(Opened {
            db,
            source: DbSource::Fresh,
            warnings,
        }),
        Err(e) => {
            fail(&mut warnings, "could not create a new database", &e);
            in_memory(path, warnings)
        }
    }
}

/// The last resort: an in-memory database that stands for `path`.
fn in_memory(path: &Path, mut warnings: Vec<String>) -> Result<Opened, OpenError> {
    warnings.push(format!(
        "{}: using a temporary in-memory database; nothing will be saved",
        path.display()
    ));
    let db = Db::in_memory(path)?;
    Ok(Opened {
        db,
        source: DbSource::Fresh,
        warnings,
    })
}
