//! Walks watched folders. File-system work: it runs on the `fp-folder-scan`
//! thread only.

use std::collections::HashSet;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crossbeam_channel::{Receiver, Sender};
use fp_model::TrackId;

use super::FolderTrouble;
use crate::ui::files::{canonical_path, is_audio};

/// One round of file-system work.
#[derive(Debug, Clone, Default)]
pub struct ScanJob {
    /// Folders to walk whole, with their subfolders.
    pub roots: Vec<PathBuf>,
    /// Paths an event touched: a file is looked at, a folder walked.
    pub paths: Vec<PathBuf>,
    /// Tracks to check: each one whose file is not there is `gone` (under
    /// a root) or `vanished` (under event paths only).
    pub known: Vec<(TrackId, PathBuf)>,
    /// Watched folders looked at, not walked: the ones an event path lies
    /// in, and every followed folder on a retry (LB22). One that is not
    /// usable is a trouble, and nothing under it is marked.
    pub checks: Vec<PathBuf>,
    /// Checks that hold tracks of the library; roots hold the tracks of
    /// `known` under them. Found empty, such a folder counts as not there.
    /// The folder of an event path is not listed: the event may have
    /// removed its last file.
    pub holding: Vec<PathBuf>,
}

/// An audio file found, as seen under the folder and in its canonical form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    pub seen: PathBuf,
    pub canonical: PathBuf,
    /// Found under one of the job's event paths, not by a root walk.
    pub event: bool,
}

#[derive(Debug, Default)]
pub struct ScanAnswer {
    pub found: Vec<Found>,
    /// Known tracks whose file is not there, under a root walk.
    pub gone: Vec<TrackId>,
    /// Known tracks whose file is not there, under event paths only: an
    /// editor may be replacing the file, so they are looked at again.
    pub vanished: Vec<TrackId>,
    /// Roots that could not be walked, and why.
    pub troubles: Vec<(PathBuf, FolderTrouble)>,
    /// Roots walked (checks are never listed).
    pub readable: Vec<PathBuf>,
}

/// Walks `start` with an explicit stack. Folder symlinks are not followed
/// (no cycles); an unreadable subfolder is skipped and counted. One deleted
/// meanwhile (an ordinary removal) is not counted.
fn walk(
    start: &Path,
    event: bool,
    found: &mut Vec<Found>,
    seen: &mut HashSet<PathBuf>,
    skipped: &mut usize,
) {
    let mut stack = vec![start.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(e) => {
                if e.kind() != ErrorKind::NotFound {
                    *skipped += 1;
                }
                continue;
            }
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            if kind.is_dir() {
                stack.push(path);
            } else if kind.is_symlink() {
                // A file symlink is read; a folder symlink is not followed.
                if path.is_file() && is_audio(&path) {
                    add(path, event, found, seen);
                }
            } else if kind.is_file() && is_audio(&path) {
                add(path, event, found, seen);
            }
        }
    }
}

fn add(path: PathBuf, event: bool, found: &mut Vec<Found>, seen: &mut HashSet<PathBuf>) {
    if seen.insert(path.clone()) {
        let canonical = canonical_path(&path);
        found.push(Found {
            seen: path,
            canonical,
            event,
        });
    }
}

/// The file is certainly not there: not found, or not a file. Any other
/// error (permission denied, a flaky share) says nothing, so no mark.
fn is_gone(path: &Path) -> bool {
    match std::fs::metadata(path) {
        Ok(meta) => !meta.is_file(),
        Err(e) => matches!(e.kind(), ErrorKind::NotFound | ErrorKind::NotADirectory),
    }
}

/// Whether a watched folder can be walked (LB22). Not there, or empty
/// while it holds tracks (a mount point with nothing mounted): `Missing`.
fn look(folder: &Path, holds: bool) -> Result<(), FolderTrouble> {
    match std::fs::metadata(folder) {
        Ok(meta) if meta.is_dir() => match std::fs::read_dir(folder) {
            Err(_) => Err(FolderTrouble::Unreadable),
            Ok(mut entries) => {
                if holds && entries.next().is_none() {
                    Err(FolderTrouble::Missing)
                } else {
                    Ok(())
                }
            }
        },
        Ok(_) => Err(FolderTrouble::Unreadable),
        Err(e) if e.kind() == ErrorKind::NotFound => Err(FolderTrouble::Missing),
        Err(_) => Err(FolderTrouble::Unreadable),
    }
}

/// Runs one job (file-system work: helper thread only).
pub fn scan(job: &ScanJob) -> ScanAnswer {
    let mut answer = ScanAnswer::default();
    let mut seen = HashSet::new();
    let mut skipped = 0usize;
    for folder in &job.checks {
        if let Err(trouble) = look(folder, job.holding.contains(folder)) {
            answer.troubles.push((folder.clone(), trouble));
        }
    }
    for root in &job.roots {
        let holds = job.known.iter().any(|(_, path)| path.starts_with(root));
        match look(root, holds) {
            Ok(()) => {
                walk(root, false, &mut answer.found, &mut seen, &mut skipped);
                answer.readable.push(root.clone());
            }
            Err(trouble) => answer.troubles.push((root.clone(), trouble)),
        }
    }
    for path in &job.paths {
        // A path that vanished again, or cannot be read, adds nothing.
        // Not followed when it is a link to a folder, as in the walk.
        let kind = std::fs::symlink_metadata(path).map(|m| m.file_type());
        if kind.as_ref().is_ok_and(|k| k.is_dir()) {
            walk(path, true, &mut answer.found, &mut seen, &mut skipped);
        } else if path.is_file() && is_audio(path) {
            add(path.clone(), true, &mut answer.found, &mut seen);
        }
    }
    if skipped > 0 {
        tracing::warn!(skipped, "folders could not be read during a folder scan");
    }
    for (root, trouble) in &answer.troubles {
        tracing::debug!(root = %root.display(), ?trouble, "watched folder not scanned");
    }
    let blocked: Vec<&PathBuf> = answer.troubles.iter().map(|(root, _)| root).collect();
    for (id, path) in &job.known {
        if blocked.iter().any(|root| path.starts_with(root)) || !is_gone(path) {
            continue;
        }
        if job.roots.iter().any(|root| path.starts_with(root)) {
            answer.gone.push(*id);
        } else {
            answer.vanished.push(*id);
        }
    }
    answer
}

/// The `fp-folder-scan` thread: one job at a time, as the file probe.
pub struct Scanner {
    jobs: Sender<ScanJob>,
    answers: Receiver<ScanAnswer>,
    /// Jobs sent and answers received: busy while they differ.
    sent: u64,
    received: u64,
    /// The thread has ended.
    dead: bool,
}

impl Scanner {
    /// `None` (logged) when the thread cannot start.
    pub fn spawn() -> Option<Scanner> {
        let (jobs, job_rx) = crossbeam_channel::unbounded::<ScanJob>();
        let (answer_tx, answers) = crossbeam_channel::unbounded::<ScanAnswer>();
        let spawned = std::thread::Builder::new()
            .name("fp-folder-scan".to_owned())
            .spawn(move || {
                // Ends when the services drop their side.
                while let Ok(job) = job_rx.recv() {
                    if answer_tx.send(scan(&job)).is_err() {
                        break;
                    }
                }
            });
        match spawned {
            Ok(_) => Some(Scanner {
                jobs,
                answers,
                sent: 0,
                received: 0,
                dead: false,
            }),
            Err(e) => {
                tracing::error!(error = %e, "cannot start the folder scanner; watched folders are not scanned");
                None
            }
        }
    }

    /// A job was sent and not answered yet (never while the thread is gone).
    pub fn busy(&self) -> bool {
        !self.dead && self.received < self.sent
    }

    /// Queues a job; `false` if the thread is gone.
    pub fn send(&mut self, job: ScanJob) -> bool {
        if self.dead || self.jobs.send(job).is_err() {
            self.dead = true;
            return false;
        }
        self.sent += 1;
        true
    }

    /// The answers that came back; never blocks.
    pub fn answers(&mut self) -> Vec<ScanAnswer> {
        let mut out = Vec::new();
        loop {
            match self.answers.try_recv() {
                Ok(answer) => {
                    self.received += 1;
                    out.push(answer);
                }
                Err(crossbeam_channel::TryRecvError::Empty) => break,
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    self.dead = true;
                    break;
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_folder_deleted_before_its_walk_is_not_counted() {
        let dir = tempfile::tempdir().unwrap();
        let (mut found, mut seen, mut skipped) = (Vec::new(), HashSet::new(), 0);
        walk(
            &dir.path().join("deleted"),
            false,
            &mut found,
            &mut seen,
            &mut skipped,
        );
        assert_eq!(skipped, 0);
    }
}
