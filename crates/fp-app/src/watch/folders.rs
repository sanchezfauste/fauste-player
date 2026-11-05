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
use super::{FolderEvent, FolderTrouble, FolderWatcher, WatchCell, WatchStatus, usable_folder};

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

/// Folders of `kept` nested in one of `unwatched` or holding one: the
/// system's watch on them can go with it (inotify removes the watches of
/// every path under the one unwatched).
fn near(unwatched: &[PathBuf], kept: &[PathBuf]) -> Vec<PathBuf> {
    kept.iter()
        .filter(|kept| {
            unwatched
                .iter()
                .any(|gone| kept.starts_with(gone) || gone.starts_with(kept))
        })
        .cloned()
        .collect()
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
    /// Folders to look at, not walk, at the next free scan.
    checks: BTreeSet<PathBuf>,
    debouncer: Debouncer,
    /// Folders the scan could not read.
    troubles: BTreeMap<PathBuf, FolderTrouble>,
    /// Folders the system refused to follow.
    not_followed: BTreeSet<PathBuf>,
    /// The watch thread is gone (it could not start, or it ended): no
    /// folder is followed.
    watch_lost: bool,
    /// Watch calls asked for and not answered yet, per folder.
    pending: BTreeMap<PathBuf, usize>,
    /// Folders a retry walks once their watch call has answered.
    walk_after_watch: BTreeSet<PathBuf>,
    /// Bad stored folders already reported.
    warned: BTreeSet<PathBuf>,
    /// Commands the conductor could not queue.
    unsent: Vec<Command>,
    published: WatchStatus,
    /// When the folders are looked at again (the troubled ones watched and
    /// walked again); `None` while none is followed.
    retry_at: Option<Instant>,
    /// Scan jobs sent.
    scans: u64,
}

impl FolderWatch {
    pub(crate) fn new(watcher: Box<dyn FolderWatcher>, status: WatchCell) -> Self {
        let watcher: SharedWatcher = Arc::new(Mutex::new(watcher));
        let thread = WatchThread::spawn(Arc::clone(&watcher));
        Self {
            // Already logged by `spawn`.
            watch_lost: thread.is_none(),
            thread,
            watcher,
            status,
            scanner: Scanner::spawn(),
            seen: None,
            folders: BTreeMap::new(),
            roots: BTreeSet::new(),
            checks: BTreeSet::new(),
            debouncer: Debouncer::default(),
            troubles: BTreeMap::new(),
            not_followed: BTreeSet::new(),
            pending: BTreeMap::new(),
            walk_after_watch: BTreeSet::new(),
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
        self.retry(state, now);
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

    /// The watch thread is gone: every folder shows as not followed, and
    /// the log says it once.
    fn lose_watch(&mut self) {
        if !self.watch_lost {
            self.watch_lost = true;
            tracing::error!(
                "the folder watch thread has ended; watched folders are scanned at start and on retries only"
            );
        }
    }

    fn not_followed(&self, folder: &Path) -> bool {
        self.watch_lost || self.not_followed.contains(folder)
    }

    /// Asks the watch thread to follow `folder`; `false` if it is gone.
    fn watch(&mut self, folder: &Path) -> bool {
        let sent = self.thread.as_ref().is_some_and(|t| {
            t.requests
                .send(Request::Watch(folder.to_path_buf()))
                .is_ok()
        });
        if sent {
            *self.pending.entry(folder.to_path_buf()).or_default() += 1;
        } else {
            self.lose_watch();
        }
        sent
    }

    /// Asks the watch thread to stop following each of `folders`, then
    /// watches again, and walks, the folders of `kept` that the system may
    /// have dropped with them; events may have been missed.
    fn unwatch(&mut self, folders: &[PathBuf], kept: &[PathBuf]) {
        for folder in folders {
            let sent = self
                .thread
                .as_ref()
                .is_some_and(|t| t.requests.send(Request::Unwatch(folder.clone())).is_ok());
            if !sent {
                self.lose_watch();
            }
        }
        for folder in near(folders, kept) {
            if self.troubles.contains_key(&folder) {
                continue;
            }
            self.watch(&folder);
            self.roots.insert(folder);
        }
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
            if !usable_folder(folder) {
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
        let old = std::mem::replace(&mut self.folders, wanted);
        let dropped: Vec<PathBuf> = old
            .keys()
            .filter(|folder| !self.folders.contains_key(*folder))
            .cloned()
            .collect();
        let kept: Vec<PathBuf> = old
            .keys()
            .filter(|folder| self.folders.contains_key(*folder))
            .cloned()
            .collect();
        for folder in &dropped {
            self.debouncer.purge_under(folder);
            self.troubles.remove(folder);
            self.not_followed.remove(folder);
            self.roots.remove(folder);
            self.checks.remove(folder);
            self.walk_after_watch.remove(folder);
        }
        if !dropped.is_empty() {
            self.unwatch(&dropped, &kept);
        }
        let added: Vec<PathBuf> = self
            .folders
            .keys()
            .filter(|folder| !old.contains_key(*folder))
            .cloned()
            .collect();
        for folder in added {
            self.watch(&folder);
            self.roots.insert(folder);
        }
    }

    /// What the watch thread did: a folder the system refused is scanned
    /// (already queued) but not followed.
    fn take_watch_results(&mut self) {
        let Some(thread) = &self.thread else {
            return;
        };
        let mut results = Vec::new();
        let mut ended = false;
        loop {
            match thread.results.try_recv() {
                Ok(result) => results.push(result),
                Err(crossbeam_channel::TryRecvError::Empty) => break,
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    ended = true;
                    break;
                }
            }
        }
        if ended {
            self.lose_watch();
        }
        for (folder, result) in results {
            if let Some(count) = self.pending.get_mut(&folder) {
                *count = count.saturating_sub(1);
                if *count == 0 {
                    self.pending.remove(&folder);
                }
            }
            if !self.folders.contains_key(&folder) {
                continue;
            }
            if self.walk_after_watch.remove(&folder) {
                self.roots.insert(folder.clone());
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

    /// Every `tuning.missing_recheck_ms`: a folder with a trouble is
    /// watched again and walked again (once the watch call answers), and
    /// every other one is looked at, so one lost while running (a drive
    /// unplugged, whose watch the system drops quietly) becomes a trouble.
    /// The calls go through the helper threads like every other.
    fn retry(&mut self, state: &AppState, now: Instant) {
        if self.folders.is_empty() {
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
        let folders: Vec<PathBuf> = self.folders.keys().cloned().collect();
        for folder in folders {
            if !self.troubles.contains_key(&folder) && !self.not_followed(&folder) {
                self.checks.insert(folder);
                continue;
            }
            // While a call for it is stuck (a share that hangs), no other
            // one piles up behind it; the walk goes on.
            let stuck = self.pending.contains_key(&folder);
            if !stuck && self.watch(&folder) {
                self.walk_after_watch.insert(folder);
            } else {
                self.roots.insert(folder);
            }
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
                // A folder itself, or one holding it, counts too (removed,
                // renamed, unmounted): `send_scan` walks the folder.
                FolderEvent::Touched(path) => {
                    if self
                        .folders
                        .keys()
                        .any(|folder| path.starts_with(folder) || folder.starts_with(&path))
                    {
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
        let mut lost = Vec::new();
        for (folder, trouble) in troubles {
            if !self.folders.contains_key(&folder) {
                continue;
            }
            let before = self.troubles.insert(folder.clone(), trouble);
            if before != Some(trouble) {
                tracing::warn!(folder = %folder.display(), ?trouble, "a watched folder cannot be scanned");
            }
            if before.is_none() {
                lost.push(folder);
            }
        }
        // Its watch is dropped; the retry sets it again.
        if !lost.is_empty() {
            let kept: Vec<PathBuf> = self
                .folders
                .keys()
                .filter(|folder| !lost.contains(*folder))
                .cloned()
                .collect();
            self.unwatch(&lost, &kept);
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
        let due = if self.debouncer.is_empty() {
            Vec::new()
        } else {
            self.debouncer.take_due(now, quiet)
        };
        if self.roots.is_empty() && due.is_empty() && self.checks.is_empty() {
            return;
        }
        // A path that is a watched folder, or holds one, is that folder's
        // root walk: its rule marks nothing when the folder is not there.
        let mut paths = Vec::new();
        for path in due {
            let held: Vec<&PathBuf> = self
                .folders
                .keys()
                .filter(|folder| folder.starts_with(&path))
                .collect();
            if !held.is_empty() {
                self.roots.extend(held.into_iter().cloned());
            } else if self.folders.keys().any(|folder| path.starts_with(folder)) {
                paths.push(path);
            }
        }
        let roots: Vec<PathBuf> = std::mem::take(&mut self.roots).into_iter().collect();
        let mut checks = std::mem::take(&mut self.checks);
        // A watched folder nested in a root is looked at with it: one lost
        // at that moment blocks the marks of its own tracks.
        checks.extend(
            self.folders
                .keys()
                .filter(|folder| roots.iter().any(|root| folder.starts_with(root)))
                .cloned(),
        );
        checks.retain(|folder| !roots.contains(folder));
        // The retry's and the nested checks: an empty one that holds
        // tracks is not there.
        // One pass over the library: each track looks its folders up.
        let holding = {
            let mut held = HashSet::new();
            if !checks.is_empty() {
                for t in state.library.iter() {
                    if t.file_state == FileState::Missing {
                        continue;
                    }
                    held.extend(t.path.ancestors().filter(|a| checks.contains(*a)));
                }
            }
            checks
                .iter()
                .filter(|folder| held.contains(folder.as_path()))
                .cloned()
                .collect()
        };
        // The folders the event paths lie in are looked at too.
        for path in &paths {
            checks.extend(
                self.folders
                    .keys()
                    .filter(|folder| path.starts_with(folder) && !roots.contains(*folder))
                    .cloned(),
            );
        }
        let checks: Vec<PathBuf> = checks.into_iter().collect();
        // The tracks of a folder in trouble wait for its own walk: a walk
        // of a folder holding it must not mark them.
        let troubled: HashSet<&Path> = self
            .troubles
            .keys()
            .filter(|folder| !roots.contains(*folder))
            .map(PathBuf::as_path)
            .collect();
        let under: HashSet<&Path> = roots
            .iter()
            .chain(paths.iter())
            .map(PathBuf::as_path)
            .collect();
        // One pass over the library: each track looks its folders up
        // (`Path::ancestors` starts with the path itself, as `starts_with`).
        let known = state
            .library
            .iter()
            .filter(|t| t.file_state != FileState::Missing)
            .filter(|t| {
                let mut troubled_above = false;
                let mut listed = false;
                for a in t.path.ancestors() {
                    troubled_above |= troubled.contains(a);
                    listed |= under.contains(a);
                }
                listed && !troubled_above
            })
            .map(|t| (t.id, t.path.clone()))
            .collect();
        let Some(scanner) = self.scanner.as_mut() else {
            return;
        };
        if scanner.send(ScanJob {
            roots,
            paths,
            known,
            checks,
            holding,
        }) {
            self.scans += 1;
        }
    }

    fn publish(&mut self) {
        let mut troubles = BTreeMap::new();
        for (folder, category) in &self.folders {
            let trouble = self.troubles.get(folder).copied().or_else(|| {
                self.not_followed(folder)
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
