//! One instance at a time. The running instance holds a lock file; a second
//! start (a playlist opened from the file manager during a show) hands its
//! playlists over through an inbox folder instead of opening the same
//! devices and state files twice.

use std::fs::{self, File, OpenOptions, TryLockError};
use std::io;
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crossbeam_channel::Sender;

const LOCK_FILE: &str = "instance.lock";
const INBOX: &str = "inbox";

/// Held for the life of the running instance; released when dropped (and
/// by the OS if the process dies).
#[derive(Debug)]
pub struct InstanceLock {
    _file: File,
}

/// Takes the instance lock in `dir`. `None` when another instance holds it.
pub fn acquire(dir: &Path) -> io::Result<Option<InstanceLock>> {
    fs::create_dir_all(dir)?;
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(dir.join(LOCK_FILE))?;
    match file.try_lock() {
        Ok(()) => Ok(Some(InstanceLock { _file: file })),
        Err(TryLockError::WouldBlock) => Ok(None),
        Err(TryLockError::Error(e)) => Err(e),
    }
}

/// Hands `paths` to the running instance of `dir`.
pub fn deliver(dir: &Path, paths: &[PathBuf]) -> io::Result<()> {
    let inbox = dir.join(INBOX);
    fs::create_dir_all(&inbox)?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let name = format!("{}-{stamp}", std::process::id());
    let text: String = paths
        .iter()
        .map(|p| format!("{}\n", p.to_string_lossy()))
        .collect();
    // Written aside, then renamed: the reader never sees half a file.
    let partial = inbox.join(format!("{name}.part"));
    fs::write(&partial, text)?;
    fs::rename(partial, inbox.join(format!("{name}.txt")))
}

/// Takes every path delivered to `dir` so far, oldest first.
pub fn collect(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir.join(INBOX)) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "txt"))
        .collect();
    files.sort();
    let mut paths = Vec::new();
    for file in files {
        let text = fs::read_to_string(&file).unwrap_or_default();
        if let Err(e) = fs::remove_file(&file) {
            tracing::warn!(file = %file.display(), "cannot remove a handed-over file: {e}");
            continue;
        }
        paths.extend(text.lines().filter(|l| !l.is_empty()).map(PathBuf::from));
    }
    paths
}

/// Watches the inbox of `dir` every `interval` and sends each delivered
/// path on `tx`. Ends when the receiver is gone.
pub fn watch(dir: PathBuf, tx: Sender<PathBuf>, interval: Duration) -> io::Result<JoinHandle<()>> {
    std::thread::Builder::new()
        .name("fp-inbox".to_owned())
        .spawn(move || {
            loop {
                for path in collect(&dir) {
                    if tx.send(path).is_err() {
                        return;
                    }
                }
                std::thread::sleep(interval);
            }
        })
}
