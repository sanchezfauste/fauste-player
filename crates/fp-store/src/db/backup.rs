//! Rotating backups of the database, taken with SQLite's online backup.

use std::fs;
use std::io;
use std::path::Path;

use rusqlite::backup::Progress;

use super::Db;
use crate::atomic::{backup_path, sibling};
use crate::error::StoreError;

impl Db {
    /// Copies the database to `<path>.bak1` and shifts the older backups up,
    /// keeping at most `backups` of them. The copy is written to
    /// `<path>.bak-tmp` first, so a failure leaves the existing backups as
    /// they were and never a partial `bak1`. With `backups == 0` it does
    /// nothing.
    pub fn backup_rotating(&self, backups: usize) -> Result<(), StoreError> {
        // The stand-in for the file has nothing to back up, and rotating it
        // in would push the real backups out.
        if backups == 0 || self.in_memory {
            return Ok(());
        }
        let tmp = sibling(&self.path, "bak-tmp");
        // A temporary file left by a crashed backup is not a backup.
        match fs::remove_file(&tmp) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
        if let Err(e) = self
            .conn
            .backup(rusqlite::MAIN_DB, &tmp, None::<fn(Progress)>)
        {
            let _ = fs::remove_file(&tmp);
            return Err(io::Error::other(e).into());
        }
        if let Err(e) = rotate(&self.path, &tmp, backups) {
            let _ = fs::remove_file(&tmp);
            return Err(e.into());
        }
        Ok(())
    }
}

fn rotate(path: &Path, tmp: &Path, backups: usize) -> io::Result<()> {
    for n in (1..backups).rev() {
        let from = backup_path(path, n);
        if from.exists() {
            fs::rename(&from, backup_path(path, n + 1))?;
        }
    }
    fs::rename(tmp, backup_path(path, 1))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn an_in_memory_database_never_rotates_the_real_backups() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("fauste.db");
        let bak1 = backup_path(&path, 1);
        fs::write(&bak1, b"real backup").unwrap();
        let db = Db::in_memory(&path).unwrap();
        db.backup_rotating(3).unwrap();
        assert_eq!(fs::read(&bak1).unwrap(), b"real backup");
        assert!(!backup_path(&path, 2).exists());
        assert!(!sibling(&path, "bak-tmp").exists());
    }
}
