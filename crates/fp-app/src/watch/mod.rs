//! Watched folders: the building blocks the services thread drives.
//!
//! A [`FolderWatcher`] reports file-system events, the [`debounce`]
//! module waits for a path to be quiet, and the [`scan`] module walks
//! folders on a helper thread (never on the services or the UI thread: a
//! share that is offline can block for seconds).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use arc_swap::ArcSwap;
use fp_model::CategoryId;

pub mod debounce;
pub(crate) mod folders;
#[cfg(not(target_os = "android"))]
pub mod notify_watcher;
pub mod scan;

/// What a [`FolderWatcher`] reports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FolderEvent {
    /// Something changed at this path: created, written, removed, or
    /// renamed from or to it.
    Touched(PathBuf),
    /// Events were lost: every watched folder is scanned again.
    Rescan,
}

/// Follows folders through the system's file-system notifications.
pub trait FolderWatcher: Send {
    /// Follows `folder` and its subfolders; `Err`: why the system refused.
    fn watch(&mut self, folder: &Path) -> Result<(), String>;
    /// Follows `folder` alone, not its subfolders (the folder of a file
    /// opened in the external editor). By default as [`Self::watch`].
    fn watch_flat(&mut self, folder: &Path) -> Result<(), String> {
        self.watch(folder)
    }
    fn unwatch(&mut self, folder: &Path);
    /// The events since the last call; never blocks.
    fn events(&mut self) -> Vec<FolderEvent>;
}

/// Whether a stored watched folder can be followed: an absolute path. An
/// empty one would match every file; a relative one depends on where the
/// application was started. Others are ignored (and shown as such).
pub fn usable_folder(folder: &Path) -> bool {
    !folder.as_os_str().is_empty() && folder.is_absolute()
}

/// Why a watched folder is not being followed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FolderTrouble {
    /// The folder is not there (a drive that is not mounted).
    Missing,
    /// The folder is there but cannot be read.
    Unreadable,
    /// The system refused to follow it; it is scanned at start and on
    /// every retry only.
    NotFollowed,
}

/// The state of the watched folders, for the UI to display.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct WatchStatus {
    pub troubles: BTreeMap<CategoryId, FolderTrouble>,
}

/// Published by the services thread, read by the UI.
pub type WatchCell = Arc<ArcSwap<WatchStatus>>;

/// A cell with no trouble.
pub fn idle_watch_cell() -> WatchCell {
    Arc::new(ArcSwap::from_pointee(WatchStatus::default()))
}
