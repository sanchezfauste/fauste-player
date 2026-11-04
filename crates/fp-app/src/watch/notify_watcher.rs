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
        for event in self.events.try_iter() {
            match event {
                Ok(event) if event.need_rescan() => out.push(FolderEvent::Rescan),
                Ok(event) => out.extend(event.paths.into_iter().map(FolderEvent::Touched)),
                Err(e) => {
                    tracing::warn!(error = %e, "folder watcher error; watched folders are scanned again");
                    out.push(FolderEvent::Rescan);
                }
            }
        }
        out
    }
}
