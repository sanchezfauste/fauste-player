//! Atomic file replacement with rotating backups, and loading with fallback.

use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Why a candidate file could not be used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Damaged or invalid: the file is quarantined.
    Corrupt(String),
    /// Written by a newer version of the app: the file is left untouched.
    TooNew(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadSource {
    Primary,
    Backup(usize),
    Defaults,
}

#[derive(Debug)]
pub struct Loaded<T> {
    /// `None` means "use defaults".
    pub value: Option<T>,
    pub source: LoadSource,
    pub warnings: Vec<String>,
}

fn sibling(path: &Path, suffix: &str) -> PathBuf {
    let mut name = OsString::from(path.as_os_str());
    name.push(".");
    name.push(suffix);
    PathBuf::from(name)
}

pub fn backup_path(path: &Path, n: usize) -> PathBuf {
    sibling(path, &format!("bak{n}"))
}

/// Replaces `path` with `bytes` so that a crash at any point leaves either the
/// old or the new content in place, never a mix. It then keeps up to `backups`
/// previous versions.
pub fn write_atomic(path: &Path, bytes: &[u8], backups: usize) -> io::Result<()> {
    let dir = path.parent().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "path has no parent directory")
    })?;
    fs::create_dir_all(dir)?;
    let tmp = sibling(path, "tmp");
    {
        let mut file = File::create(&tmp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }
    rotate_backups(path, backups)?;
    retry_locked(|| fs::rename(&tmp, path))?;
    sync_dir(dir)
}

/// Attempts of an operation on a file another process holds for a moment
/// (on Windows, a virus scanner or indexer opening the file just written).
const LOCKED_ATTEMPTS: u32 = 5;
/// The first wait between attempts; it doubles each time (310 ms in all).
const LOCKED_FIRST_WAIT_MS: u64 = 10;

/// Runs `op`, retrying it a few times while it fails with "permission
/// denied", the error a sharing violation gives.
fn retry_locked<T>(mut op: impl FnMut() -> io::Result<T>) -> io::Result<T> {
    let mut wait = LOCKED_FIRST_WAIT_MS;
    let mut attempt = 1;
    loop {
        match op() {
            Err(e) if e.kind() == io::ErrorKind::PermissionDenied && attempt < LOCKED_ATTEMPTS => {
                std::thread::sleep(std::time::Duration::from_millis(wait));
                wait *= 2;
                attempt += 1;
            }
            result => return result,
        }
    }
}

fn rotate_backups(path: &Path, backups: usize) -> io::Result<()> {
    if backups == 0 || !path.exists() {
        return Ok(());
    }
    for n in (1..backups).rev() {
        let from = backup_path(path, n);
        if from.exists() {
            retry_locked(|| fs::rename(&from, backup_path(path, n + 1)))?;
        }
    }
    // Copy (not rename) so the primary stays in place until the new one replaces it.
    retry_locked(|| fs::copy(path, backup_path(path, 1)))?;
    Ok(())
}

#[cfg(unix)]
fn sync_dir(dir: &Path) -> io::Result<()> {
    File::open(dir)?.sync_all()
}

#[cfg(not(unix))]
fn sync_dir(_dir: &Path) -> io::Result<()> {
    Ok(())
}

fn read_limited(path: &Path, max_bytes: u64) -> io::Result<Option<Vec<u8>>> {
    let meta = match fs::metadata(path) {
        Ok(meta) => meta,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };
    if meta.len() > max_bytes {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("file is {} bytes, the limit is {max_bytes}", meta.len()),
        ));
    }
    fs::read(path).map(Some)
}

/// `path` with a `<kind>-<seconds>` suffix that no file has yet: a second
/// one within the same second gets `-2`, `-3`…
fn unused_sibling(path: &Path, kind: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let first = sibling(path, &format!("{kind}-{stamp}"));
    if !first.exists() {
        return first;
    }
    (2u32..)
        .map(|n| sibling(path, &format!("{kind}-{stamp}-{n}")))
        .find(|candidate| !candidate.exists())
        .unwrap_or(first)
}

fn quarantine(path: &Path, warnings: &mut Vec<String>) {
    let target = unused_sibling(path, "corrupt");
    if let Err(e) = fs::rename(path, &target) {
        warnings.push(format!("{}: could not quarantine: {e}", path.display()));
    }
}

/// Keeps a copy of a file written by a newer version under a name that backup
/// rotation never touches, so a later upgrade can still recover it. A copy
/// with identical content is not duplicated.
fn preserve(path: &Path, bytes: &[u8], warnings: &mut Vec<String>) {
    let (Some(dir), Some(name)) = (path.parent(), path.file_name()) else {
        return;
    };
    let prefix = format!("{}.newer-", name.to_string_lossy());
    let already = fs::read_dir(dir).into_iter().flatten().flatten().any(|e| {
        e.file_name().to_string_lossy().starts_with(&prefix)
            && fs::read(e.path()).is_ok_and(|existing| existing == bytes)
    });
    if already {
        return;
    }
    let target = unused_sibling(path, "newer");
    match fs::write(&target, bytes) {
        Ok(()) => warnings.push(format!(
            "{}: kept a copy at {}",
            path.display(),
            target.display()
        )),
        Err(e) => warnings.push(format!("{}: could not keep a copy: {e}", path.display())),
    }
}

/// Loads the primary file, falling back to `.bak1`…`.bakN`. Never fails:
/// when nothing is usable it returns `value: None` (use defaults).
pub fn load_with_fallback<T>(
    path: &Path,
    backups: usize,
    max_bytes: u64,
    parse: impl Fn(&[u8]) -> Result<T, ParseError>,
) -> Loaded<T> {
    let mut warnings = Vec::new();
    let candidates = std::iter::once((LoadSource::Primary, path.to_path_buf()))
        .chain((1..=backups).map(|n| (LoadSource::Backup(n), backup_path(path, n))));
    for (source, candidate) in candidates {
        match read_limited(&candidate, max_bytes) {
            Ok(None) => {}
            Ok(Some(bytes)) => match parse(&bytes) {
                Ok(value) => {
                    return Loaded {
                        value: Some(value),
                        source,
                        warnings,
                    };
                }
                Err(ParseError::TooNew(msg)) => {
                    warnings.push(format!("{}: {msg}", candidate.display()));
                    preserve(&candidate, &bytes, &mut warnings);
                }
                Err(ParseError::Corrupt(msg)) => {
                    warnings.push(format!(
                        "{}: unreadable ({msg}); quarantined",
                        candidate.display()
                    ));
                    quarantine(&candidate, &mut warnings);
                }
            },
            Err(e) => warnings.push(format!("{}: {e}", candidate.display())),
        }
    }
    Loaded {
        value: None,
        source: LoadSource::Defaults,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_text(bytes: &[u8]) -> Result<String, ParseError> {
        let s = std::str::from_utf8(bytes).map_err(|e| ParseError::Corrupt(e.to_string()))?;
        if s.starts_with("ok:") {
            Ok(s.to_owned())
        } else if s.starts_with("future:") {
            Err(ParseError::TooNew("written by a newer version".into()))
        } else {
            Err(ParseError::Corrupt("bad content".into()))
        }
    }

    fn names(dir: &Path) -> Vec<String> {
        let mut v: Vec<String> = fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        v.sort();
        v
    }

    #[test]
    fn writes_rotate_backups_and_leave_no_temp_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("sub/state.json");
        for n in 1..=4 {
            write_atomic(&file, format!("ok:{n}").as_bytes(), 2).unwrap();
        }
        assert_eq!(fs::read_to_string(&file).unwrap(), "ok:4");
        assert_eq!(fs::read_to_string(backup_path(&file, 1)).unwrap(), "ok:3");
        assert_eq!(fs::read_to_string(backup_path(&file, 2)).unwrap(), "ok:2");
        assert_eq!(
            names(&dir.path().join("sub")),
            vec!["state.json", "state.json.bak1", "state.json.bak2"]
        );
    }

    #[test]
    fn zero_backups_keeps_only_the_primary() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("s.json");
        write_atomic(&file, b"ok:1", 0).unwrap();
        write_atomic(&file, b"ok:2", 0).unwrap();
        assert_eq!(names(dir.path()), vec!["s.json"]);
    }

    #[test]
    fn first_run_loads_defaults_silently() {
        let dir = tempfile::tempdir().unwrap();
        let loaded = load_with_fallback(&dir.path().join("none.json"), 3, 1024, parse_text);
        assert_eq!((loaded.value, loaded.source), (None, LoadSource::Defaults));
        assert!(loaded.warnings.is_empty());
    }

    #[test]
    fn truncated_primary_falls_back_to_backup_and_is_quarantined() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("s.json");
        write_atomic(&file, b"ok:1", 3).unwrap();
        write_atomic(&file, b"ok:2", 3).unwrap();
        fs::write(&file, b"{trunc").unwrap(); // crash mid-write of a non-atomic writer
        fs::write(dir.path().join("s.json.tmp"), b"stale").unwrap(); // leftover temp file
        let loaded = load_with_fallback(&file, 3, 1024, parse_text);
        assert_eq!(loaded.value.as_deref(), Some("ok:1"));
        assert_eq!(loaded.source, LoadSource::Backup(1));
        assert_eq!(loaded.warnings.len(), 1);
        assert!(!file.exists());
        assert!(
            names(dir.path())
                .iter()
                .any(|n| n.starts_with("s.json.corrupt-"))
        );
    }

    #[test]
    fn too_new_files_are_left_in_place() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("s.json");
        fs::write(&file, b"future:9").unwrap();
        let loaded = load_with_fallback(&file, 3, 1024, parse_text);
        assert_eq!(loaded.source, LoadSource::Defaults);
        assert!(loaded.warnings[0].contains("newer"));
        assert!(file.exists());
    }

    #[test]
    fn oversized_files_are_refused_without_reading_them() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("s.json");
        fs::write(&file, vec![b'x'; 2048]).unwrap();
        let loaded = load_with_fallback(&file, 0, 1024, parse_text);
        assert_eq!(loaded.source, LoadSource::Defaults);
        assert!(loaded.warnings[0].contains("limit"));
        assert!(file.exists());
    }

    #[test]
    fn two_quarantines_in_the_same_second_keep_both_files() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("s.json");
        fs::write(&file, b"bad one").unwrap();
        load_with_fallback(&file, 0, 1024, parse_text);
        fs::write(&file, b"bad two").unwrap();
        load_with_fallback(&file, 0, 1024, parse_text);
        let kept: Vec<String> = names(dir.path())
            .into_iter()
            .filter(|n| n.starts_with("s.json.corrupt-"))
            .collect();
        assert_eq!(kept.len(), 2, "{kept:?}");
    }

    #[test]
    fn a_briefly_locked_file_is_retried() {
        let mut calls = 0;
        let result = retry_locked(|| {
            calls += 1;
            if calls < 3 {
                Err(io::Error::from(io::ErrorKind::PermissionDenied))
            } else {
                Ok(calls)
            }
        });
        assert_eq!(result.unwrap(), 3);
        let mut calls = 0;
        let result: io::Result<()> = retry_locked(|| {
            calls += 1;
            Err(io::Error::from(io::ErrorKind::NotFound))
        });
        assert!(result.is_err());
        assert_eq!(calls, 1, "other errors are not retried");
    }
}
