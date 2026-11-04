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
    /// Tracks to check: each one whose file is not there is `gone`.
    pub known: Vec<(TrackId, PathBuf)>,
}

/// An audio file found, as seen under the folder and in its canonical form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    pub seen: PathBuf,
    pub canonical: PathBuf,
}

#[derive(Debug, Default)]
pub struct ScanAnswer {
    pub found: Vec<Found>,
    pub gone: Vec<TrackId>,
    /// Roots that could not be walked, and why.
    pub troubles: Vec<(PathBuf, FolderTrouble)>,
    /// Roots walked.
    pub readable: Vec<PathBuf>,
}

/// Walks `start` with an explicit stack. Folder symlinks are not followed
/// (no cycles); an unreadable subfolder is skipped and counted.
fn walk(start: &Path, found: &mut Vec<Found>, seen: &mut HashSet<PathBuf>, skipped: &mut usize) {
    let mut stack = vec![start.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => {
                *skipped += 1;
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
                    add(path, found, seen);
                }
            } else if kind.is_file() && is_audio(&path) {
                add(path, found, seen);
            }
        }
    }
}

fn add(path: PathBuf, found: &mut Vec<Found>, seen: &mut HashSet<PathBuf>) {
    if seen.insert(path.clone()) {
        let canonical = canonical_path(&path);
        found.push(Found {
            seen: path,
            canonical,
        });
    }
}

/// Runs one job (file-system work: helper thread only).
pub fn scan(job: &ScanJob) -> ScanAnswer {
    let mut answer = ScanAnswer::default();
    let mut seen = HashSet::new();
    let mut skipped = 0usize;
    for root in &job.roots {
        match std::fs::metadata(root) {
            Ok(meta) if meta.is_dir() => {
                if std::fs::read_dir(root).is_err() {
                    answer
                        .troubles
                        .push((root.clone(), FolderTrouble::Unreadable));
                    continue;
                }
                walk(root, &mut answer.found, &mut seen, &mut skipped);
                answer.readable.push(root.clone());
            }
            Ok(_) => answer
                .troubles
                .push((root.clone(), FolderTrouble::Unreadable)),
            Err(e) if e.kind() == ErrorKind::NotFound => {
                answer.troubles.push((root.clone(), FolderTrouble::Missing));
            }
            Err(_) => answer
                .troubles
                .push((root.clone(), FolderTrouble::Unreadable)),
        }
    }
    for path in &job.paths {
        // A path that vanished again, or cannot be read, adds nothing.
        if path.is_dir() {
            walk(path, &mut answer.found, &mut seen, &mut skipped);
        } else if path.is_file() && is_audio(path) {
            add(path.clone(), &mut answer.found, &mut seen);
        }
    }
    if skipped > 0 {
        tracing::warn!(skipped, "folders could not be read during a folder scan");
    }
    for (root, trouble) in &answer.troubles {
        tracing::debug!(root = %root.display(), ?trouble, "watched folder not scanned");
    }
    let blocked: Vec<&PathBuf> = answer.troubles.iter().map(|(root, _)| root).collect();
    answer.gone = job
        .known
        .iter()
        .filter(|(_, path)| !blocked.iter().any(|root| path.starts_with(root)) && !path.is_file())
        .map(|(id, _)| *id)
        .collect();
    answer
}

/// The `fp-folder-scan` thread: one job at a time, as the file probe.
pub struct Scanner {
    jobs: Sender<ScanJob>,
    answers: Receiver<ScanAnswer>,
    /// A job was sent and its answer has not come back.
    busy: bool,
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
                busy: false,
            }),
            Err(e) => {
                tracing::error!(error = %e, "cannot start the folder scanner; watched folders are not scanned");
                None
            }
        }
    }

    pub fn busy(&self) -> bool {
        self.busy
    }

    /// Queues a job; `false` if the thread is gone.
    pub fn send(&mut self, job: ScanJob) -> bool {
        if self.jobs.send(job).is_ok() {
            self.busy = true;
            true
        } else {
            false
        }
    }

    /// The answers that came back; never blocks.
    pub fn answers(&mut self) -> Vec<ScanAnswer> {
        let answers: Vec<ScanAnswer> = self.answers.try_iter().collect();
        if !answers.is_empty() {
            self.busy = false;
        }
        answers
    }
}
