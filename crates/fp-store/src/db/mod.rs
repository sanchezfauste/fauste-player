//! The SQLite database of the library: opening, integrity check and
//! schema migrations.

mod schema;

use std::path::{Path, PathBuf};

use rusqlite::{Connection, ErrorCode};

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

/// An open library database.
pub struct Db {
    pub(crate) conn: Connection,
    #[allow(dead_code)]
    pub(crate) path: PathBuf,
}

impl Db {
    /// Opens (creating it when missing) the database at `path`, checks its
    /// integrity and brings its schema up to [`SCHEMA_VERSION`].
    pub fn open(path: &Path) -> Result<Db, OpenError> {
        let mut conn = Connection::open(path).map_err(classify)?;
        let mode: String = conn
            .query_row("PRAGMA journal_mode=WAL", [], |r| r.get(0))
            .map_err(classify)?;
        if !mode.eq_ignore_ascii_case("wal") {
            return Err(OpenError::Io(format!("journal mode is {mode}, not wal")));
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
            let tx = conn.transaction().map_err(classify)?;
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
