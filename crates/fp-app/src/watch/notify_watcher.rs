//! The desktop [`FolderWatcher`], over the system's file-system
//! notifications (`inotify`, `FSEvents`, `ReadDirectoryChangesW`).

use std::path::Path;

use crossbeam_channel::Receiver;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

use super::{FolderEvent, FolderWatcher};

pub struct NotifyWatcher {
    watcher: RecommendedWatcher,
    events: Receiver<notify::Result<notify::Event>>,
}

impl NotifyWatcher {
    pub fn new() -> Result<NotifyWatcher, String> {
        let (tx, events) = crossbeam_channel::unbounded();
        let watcher = notify::recommended_watcher(move |event| {
            // The receiver is gone when the watcher is being dropped.
            let _ = tx.send(event);
        })
        .map_err(|e| e.to_string())?;
        Ok(NotifyWatcher { watcher, events })
    }
}

impl FolderWatcher for NotifyWatcher {
    fn watch(&mut self, folder: &Path) -> Result<(), String> {
        self.watcher
            .watch(folder, RecursiveMode::Recursive)
            .map_err(|e| e.to_string())
    }

    fn unwatch(&mut self, folder: &Path) {
        if let Err(e) = self.watcher.unwatch(folder) {
            tracing::debug!(folder = %folder.display(), error = %e, "unwatch refused");
        }
    }

    fn events(&mut self) -> Vec<FolderEvent> {
        let mut out = Vec::new();
        let mut errors = 0usize;
        for result in self.events.try_iter() {
            if result.is_err() {
                errors += 1;
            }
            out.extend(convert(result));
        }
        if errors > 0 {
            tracing::warn!(
                errors,
                "folder watcher errors; watched folders are scanned again"
            );
        }
        out
    }
}

/// What one notification means for the library. Opens and reads of a file
/// (the system raises them for our own scans and for playback) are not
/// changes and are dropped; a finished write is one.
pub fn convert(result: notify::Result<notify::Event>) -> Vec<FolderEvent> {
    use notify::EventKind;
    use notify::event::{AccessKind, AccessMode};
    match result {
        Err(_) => vec![FolderEvent::Rescan],
        Ok(event) if event.need_rescan() => vec![FolderEvent::Rescan],
        Ok(event) => {
            let finished_write = matches!(
                event.kind,
                EventKind::Access(AccessKind::Close(AccessMode::Write))
            );
            if matches!(event.kind, EventKind::Access(_)) && !finished_write {
                return Vec::new();
            }
            event.paths.into_iter().map(FolderEvent::Touched).collect()
        }
    }
}
