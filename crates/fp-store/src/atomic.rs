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
    fs::rename(&tmp, path)?;
    sync_dir(dir)
}

fn rotate_backups(path: &Path, backups: usize) -> io::Result<()> {
    if backups == 0 || !path.exists() {
        return Ok(());
    }
    for n in (1..backups).rev() {
        let from = backup_path(path, n);
        if from.exists() {
            fs::rename(&from, backup_path(path, n + 1))?;
        }
    }
    // Copy (not rename) so the primary stays in place until the new one replaces it.
    fs::copy(path, backup_path(path, 1))?;
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

fn quarantine(path: &Path, warnings: &mut Vec<String>) {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let target = sibling(path, &format!("corrupt-{stamp}"));
    if let Err(e) = fs::rename(path, &target) {
        warnings.push(format!("{}: could not quarantine: {e}", path.display()));
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
                    warnings.push(format!("{}: {msg}", candidate.display()))
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
}
