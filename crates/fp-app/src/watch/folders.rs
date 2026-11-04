//! Follows the categories' watched folders for the services thread: it
//! compares paths and sends commands, and leaves every file-system call to
//! the watcher and to the `fp-folder-scan` thread.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

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

pub(crate) struct FolderWatch {
    watcher: Box<dyn FolderWatcher>,
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
    /// Scan jobs sent.
    scans: u64,
}

impl FolderWatch {
    pub(crate) fn new(watcher: Box<dyn FolderWatcher>, status: WatchCell) -> Self {
        Self {
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
            scans: 0,
        }
    }

    /// How many scan jobs were sent.
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
            self.watcher.unwatch(&folder);
            self.troubles.remove(&folder);
            self.not_followed.remove(&folder);
            self.roots.remove(&folder);
        }
        for folder in wanted.keys() {
            if self.folders.contains_key(folder) {
                continue;
            }
            if let Err(why) = self.watcher.watch(folder) {
                tracing::warn!(
                    folder = %folder.display(),
                    %why,
                    "a watched folder is not followed; it is scanned at start and on retries only"
                );
                self.not_followed.insert(folder.clone());
            }
            self.roots.insert(folder.clone());
        }
        self.folders = wanted;
    }

    /// (2) Events wait for their path to be quiet.
    fn take_events(&mut self, now: Instant) {
        for event in self.watcher.events() {
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
