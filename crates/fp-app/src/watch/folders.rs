//! Follows the categories' watched folders for the services thread: it
//! compares paths and sends commands, and leaves every file-system call to
//! the watcher and to the `fp-folder-scan` thread.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError, TryLockError};
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, Sender};

use fp_model::{AppState, Categories, CategoryId, Command, FileState, TrackId, watching_category};

use super::debounce::Debouncer;
use super::scan::{Found, ScanAnswer, ScanJob, Scanner};
use super::{FolderEvent, FolderTrouble, FolderWatcher, WatchCell, WatchStatus};

/// What one round asks of the services.
#[derive(Debug, Default)]
pub(crate) struct WatchOutcome {
    pub commands: Vec<Command>,
    /// Missing tracks whose file is there again: to analyse at once.
    pub returned: Vec<TrackId>,
}

enum Request {
    Watch(PathBuf),
    Unwatch(PathBuf),
}

type SharedWatcher = Arc<Mutex<Box<dyn FolderWatcher>>>;

/// The `fp-folder-watch` thread: registering a folder with the system can
/// walk a whole tree or wait on a share, so it never runs on the services
/// thread. It ends when `requests` closes.
struct WatchThread {
    requests: Sender<Request>,
    results: Receiver<(PathBuf, Result<(), String>)>,
}

impl WatchThread {
    /// `None` (logged) when the thread cannot start.
    fn spawn(watcher: SharedWatcher) -> Option<WatchThread> {
        let (requests, request_rx) = crossbeam_channel::unbounded::<Request>();
        let (result_tx, results) = crossbeam_channel::unbounded();
        let spawned = std::thread::Builder::new()
            .name("fp-folder-watch".to_owned())
            .spawn(move || {
                while let Ok(request) = request_rx.recv() {
                    let mut watcher = watcher.lock().unwrap_or_else(PoisonError::into_inner);
                    match request {
                        Request::Watch(folder) => {
                            let result = watcher.watch(&folder);
                            if result_tx.send((folder, result)).is_err() {
                                break;
                            }
                        }
                        Request::Unwatch(folder) => watcher.unwatch(&folder),
                    }
                }
            });
        match spawned {
            Ok(_) => Some(WatchThread { requests, results }),
            Err(e) => {
                tracing::error!(error = %e, "cannot start the folder watch thread; folders are not followed");
                None
            }
        }
    }
}

/// The retry interval, the one missing files are looked for at.
fn retry_interval(state: &AppState) -> Duration {
    Duration::try_from_secs_f64(state.config.tuning.missing_recheck_ms / 1_000.0)
        .unwrap_or(Duration::from_secs(30))
}

pub(crate) struct FolderWatch {
    watcher: SharedWatcher,
    thread: Option<WatchThread>,
    status: WatchCell,
    /// `None` when the thread could not start: nothing is scanned.
    scanner: Option<Scanner>,
    /// The categories the folders were last derived from.
    seen: Option<Categories>,
    /// The folders followed, with the category that holds each.
    folders: BTreeMap<PathBuf, CategoryId>,
    /// Folders to walk whole at the next free scan.
    roots: BTreeSet<PathBuf>,
    debouncer: Debouncer,
    /// Folders the scan could not read.
    troubles: BTreeMap<PathBuf, FolderTrouble>,
    /// Folders the system refused to follow.
    not_followed: BTreeSet<PathBuf>,
    /// Bad stored folders already reported.
    warned: BTreeSet<PathBuf>,
    /// Commands the conductor could not queue.
    unsent: Vec<Command>,
    published: WatchStatus,
    /// When folders with a trouble are tried again; `None` while none has one.
    retry_at: Option<Instant>,
    /// Scan jobs sent.
    scans: u64,
}

impl FolderWatch {
    pub(crate) fn new(watcher: Box<dyn FolderWatcher>, status: WatchCell) -> Self {
        let watcher: SharedWatcher = Arc::new(Mutex::new(watcher));
        Self {
            thread: WatchThread::spawn(Arc::clone(&watcher)),
            watcher,
            status,
            scanner: Scanner::spawn(),
            seen: None,
            folders: BTreeMap::new(),
            roots: BTreeSet::new(),
            debouncer: Debouncer::default(),
            troubles: BTreeMap::new(),
            not_followed: BTreeSet::new(),
            warned: BTreeSet::new(),
            unsent: Vec::new(),
            published: WatchStatus::default(),
            retry_at: None,
            scans: 0,
        }
    }

    /// How many scan jobs were sent.
    #[cfg(feature = "test-hooks")]
    pub(crate) fn scans(&self) -> u64 {
        self.scans
    }

    /// Commands the conductor could not queue: sent first next round.
    pub(crate) fn requeue(&mut self, unsent: Vec<Command>) {
        self.unsent.extend(unsent);
    }

    /// One round on the services thread; never blocks.
    pub(crate) fn step(&mut self, state: &AppState, now: Instant) -> WatchOutcome {
        let mut out = WatchOutcome {
            commands: std::mem::take(&mut self.unsent),
            returned: Vec::new(),
        };
        self.follow_categories(state);
        self.take_watch_results();
        self.retry_troubled(state, now);
        self.take_events(now);
        for answer in self
            .scanner
            .as_mut()
            .map(Scanner::answers)
            .unwrap_or_default()
        {
            self.apply_answer(state, answer, &mut out);
        }
        self.send_scan(state, now);
        self.publish();
        out
    }

    /// (1) The folders follow the categories.
    fn follow_categories(&mut self, state: &AppState) {
        if self.seen.as_ref() == Some(&state.categories) {
            return;
        }
        self.seen = Some(state.categories.clone());
        let mut wanted = BTreeMap::new();
        let mut bad = BTreeSet::new();
        for category in state.categories.iter() {
            let Some(folder) = &category.watched_folder else {
                continue;
            };
            // An empty path would match every file.
            if folder.as_os_str().is_empty() || !folder.is_absolute() {
                if self.warned.insert(folder.clone()) {
                    tracing::warn!(
                        folder = %folder.display(),
                        category = category.id.0,
                        "a watched folder is not an absolute path; it is not watched"
                    );
                }
                bad.insert(folder.clone());
                continue;
            }
            wanted.insert(folder.clone(), category.id);
        }
        self.warned.retain(|folder| bad.contains(folder));
        let dropped: Vec<PathBuf> = self
            .folders
            .keys()
            .filter(|folder| !wanted.contains_key(*folder))
            .cloned()
            .collect();
        for folder in dropped {
            if let Some(thread) = &self.thread {
                let _ = thread.requests.send(Request::Unwatch(folder.clone()));
            }
            self.debouncer.purge_under(&folder);
            self.troubles.remove(&folder);
            self.not_followed.remove(&folder);
            self.roots.remove(&folder);
        }
        for folder in wanted.keys() {
            if self.folders.contains_key(folder) {
                continue;
            }
            if let Some(thread) = &self.thread {
                let _ = thread.requests.send(Request::Watch(folder.clone()));
            }
            self.roots.insert(folder.clone());
        }
        self.folders = wanted;
    }

    /// What the watch thread did: a folder the system refused is scanned
    /// (already queued) but not followed.
    fn take_watch_results(&mut self) {
        let Some(thread) = &self.thread else {
            return;
        };
        for (folder, result) in thread.results.try_iter() {
            if !self.folders.contains_key(&folder) {
                continue;
            }
            match result {
                Err(why) => {
                    // Retries fail the same way: the log says it once.
                    if self.not_followed.insert(folder.clone()) {
                        tracing::warn!(
                            folder = %folder.display(),
                            %why,
                            "a watched folder is not followed; it is scanned at start and on retries only"
                        );
                    }
                }
                // Also ends a refusal from an earlier call that was stale by
                // the time it answered. Events may have been missed meanwhile.
                Ok(()) => {
                    if self.not_followed.remove(&folder) {
                        tracing::info!(folder = %folder.display(), "a watched folder is followed again");
                        self.roots.insert(folder);
                    }
                }
            }
        }
    }

    /// Every `tuning.missing_recheck_ms`, the folders with a trouble are
    /// watched again and walked again. The watch call goes through the
    /// helper thread like every other.
    fn retry_troubled(&mut self, state: &AppState, now: Instant) {
        let troubled: Vec<PathBuf> = self
            .folders
            .keys()
            .filter(|f| self.troubles.contains_key(*f) || self.not_followed.contains(*f))
            .cloned()
            .collect();
        if troubled.is_empty() {
            self.retry_at = None;
            return;
        }
        let Some(due) = self.retry_at else {
            self.retry_at = Some(now + retry_interval(state));
            return;
        };
        if now < due {
            return;
        }
        self.retry_at = Some(now + retry_interval(state));
        for folder in troubled {
            if let Some(thread) = &self.thread {
                let _ = thread.requests.send(Request::Watch(folder.clone()));
            }
            self.roots.insert(folder);
        }
    }

    /// (2) Events wait for their path to be quiet.
    fn take_events(&mut self, now: Instant) {
        // A watch call in progress holds the watcher: the events stay
        // queued in it until the next round.
        let events = match self.watcher.try_lock() {
            Ok(mut watcher) if self.thread.is_some() => watcher.events(),
            Ok(_) => Vec::new(),
            Err(TryLockError::Poisoned(e)) => e.into_inner().events(),
            Err(TryLockError::WouldBlock) => Vec::new(),
        };
        for event in events {
            match event {
                FolderEvent::Touched(path) => {
                    if self.folders.keys().any(|folder| path.starts_with(folder)) {
                        self.debouncer.note(path, now);
                    }
                }
                FolderEvent::Rescan => self.roots.extend(self.folders.keys().cloned()),
            }
        }
    }

    /// (3) What a scan found.
    fn apply_answer(&mut self, state: &AppState, answer: ScanAnswer, out: &mut WatchOutcome) {
        let ScanAnswer {
            found,
            gone,
            troubles,
            readable,
        } = answer;
        let missing: HashMap<&Path, TrackId> = if found.is_empty() {
            HashMap::new()
        } else {
            state
                .library
                .iter()
                .filter(|t| t.file_state == FileState::Missing)
                .map(|t| (t.path.as_path(), t.id))
                .collect()
        };
        let mut joining: BTreeMap<CategoryId, Vec<PathBuf>> = BTreeMap::new();
        for Found { seen, canonical } in found {
            // A category deleted or cleared meanwhile watches nothing.
            let Some(category) = watching_category(&state.categories, &seen) else {
                continue;
            };
            if let Some(track) = missing.get(canonical.as_path()) {
                out.returned.push(*track);
            } else {
                joining.entry(category).or_default().push(canonical);
            }
        }
        for (category, paths) in joining {
            out.commands.push(Command::AddToLibrary {
                paths,
                category: Some(category),
            });
        }
        if !gone.is_empty() {
            let already: HashSet<TrackId> = state
                .library
                .iter()
                .filter(|t| t.file_state == FileState::Missing)
                .map(|t| t.id)
                .collect();
            for track in gone.into_iter().filter(|t| !already.contains(t)) {
                out.commands.push(Command::SetFileState {
                    track,
                    state: FileState::Missing,
                });
            }
        }
        for (folder, trouble) in troubles {
            if !self.folders.contains_key(&folder) {
                continue;
            }
            if self.troubles.insert(folder.clone(), trouble) != Some(trouble) {
                tracing::warn!(folder = %folder.display(), ?trouble, "a watched folder cannot be scanned");
            }
        }
        for folder in readable {
            if self.troubles.remove(&folder).is_some() {
                tracing::info!(folder = %folder.display(), "a watched folder can be scanned again");
            }
        }
    }

    /// (4) One job when the scanner is free and work waits.
    fn send_scan(&mut self, state: &AppState, now: Instant) {
        let Some(scanner) = self.scanner.as_mut() else {
            return;
        };
        if scanner.busy() {
            return;
        }
        let quiet = Duration::from_millis(state.config.library.watch_debounce_ms);
        let paths = if self.debouncer.is_empty() {
            Vec::new()
        } else {
            self.debouncer.take_due(now, quiet)
        };
        if self.roots.is_empty() && paths.is_empty() {
            return;
        }
        let roots: Vec<PathBuf> = std::mem::take(&mut self.roots).into_iter().collect();
        let known = state
            .library
            .iter()
            .filter(|t| t.file_state != FileState::Missing)
            .filter(|t| {
                roots
                    .iter()
                    .chain(paths.iter())
                    .any(|under| t.path.starts_with(under))
            })
            .map(|t| (t.id, t.path.clone()))
            .collect();
        if scanner.send(ScanJob {
            roots,
            paths,
            known,
        }) {
            self.scans += 1;
        }
    }

    fn publish(&mut self) {
        let mut troubles = BTreeMap::new();
        for (folder, category) in &self.folders {
            let trouble = self.troubles.get(folder).copied().or_else(|| {
                self.not_followed
                    .contains(folder)
                    .then_some(FolderTrouble::NotFollowed)
            });
            if let Some(trouble) = trouble {
                troubles.insert(*category, trouble);
            }
        }
        if troubles != self.published.troubles {
            self.published = WatchStatus { troubles };
            self.status
                .store(std::sync::Arc::new(self.published.clone()));
        }
    }
}
