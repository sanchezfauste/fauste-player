//! The media browser: where a platform without file dialogs (Android) lets
//! the operator pick music (mobile spec MB7). The platform hands over the
//! device's audio files (`set_files`); its pickers call `browse` on their
//! helper thread, which waits until the interface's window answers.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};

/// What a request asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowseMode {
    /// Any number of audio files.
    AudioFiles,
    /// One file with one of these extensions.
    OneFile { extensions: Vec<String> },
    /// One folder holding audio files.
    Folder,
}

/// The device's audio files and at most one pending request. Clones share
/// them: the platform, the pickers' threads and the interface hold one each.
#[derive(Clone, Default)]
pub struct MediaBrowser {
    shared: Arc<Shared>,
}

#[derive(Default)]
struct Shared {
    inner: Mutex<Inner>,
    answered: Condvar,
}

#[derive(Default)]
struct Inner {
    files: Arc<Vec<PathBuf>>,
    next_id: u64,
    request: Option<(u64, BrowseMode, Option<PathBuf>)>,
    /// Answers not yet collected by their `browse` call.
    answers: Vec<(u64, Vec<PathBuf>)>,
    repaint: Option<egui::Context>,
}

impl MediaBrowser {
    /// The device's audio files, replacing the earlier list.
    pub fn set_files(&self, files: Vec<PathBuf>) {
        let repaint = {
            let mut inner = self.lock();
            inner.files = Arc::new(files);
            inner.repaint.clone()
        };
        if let Some(ctx) = repaint {
            ctx.request_repaint();
        }
    }

    pub fn files(&self) -> Arc<Vec<PathBuf>> {
        self.lock().files.clone()
    }

    /// Asks the operator and waits for the answer; nothing chosen when
    /// cancelled or when another request replaces this one. Blocks: call it
    /// from a helper thread only (rule 8).
    pub fn browse(&self, mode: BrowseMode, start: Option<PathBuf>) -> Vec<PathBuf> {
        let mut inner = self.lock();
        inner.next_id += 1;
        let id = inner.next_id;
        if let Some((replaced, _, _)) = inner.request.take() {
            inner.answers.push((replaced, Vec::new()));
            self.shared.answered.notify_all();
        }
        inner.request = Some((id, mode, start));
        if let Some(ctx) = &inner.repaint {
            ctx.request_repaint();
        }
        loop {
            if let Some(at) = inner.answers.iter().position(|(a, _)| *a == id) {
                return inner.answers.swap_remove(at).1;
            }
            inner = self
                .shared
                .answered
                .wait(inner)
                .unwrap_or_else(PoisonError::into_inner);
        }
    }

    /// The request waiting for an answer: its id, what it asks for and
    /// where it starts.
    pub fn pending(&self) -> Option<(u64, BrowseMode, Option<PathBuf>)> {
        self.lock().request.clone()
    }

    /// Answers request `request`; ignored when it is no longer pending.
    pub fn answer(&self, request: u64, chosen: Vec<PathBuf>) {
        let mut inner = self.lock();
        if inner
            .request
            .as_ref()
            .is_some_and(|(id, _, _)| *id == request)
        {
            inner.request = None;
            inner.answers.push((request, chosen));
            self.shared.answered.notify_all();
        }
    }

    /// The interface to repaint when a request arrives or the files change.
    pub fn set_repaint(&self, ctx: egui::Context) {
        self.lock().repaint = Some(ctx);
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.shared
            .inner
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}

/// The folders holding the files `mode` can choose, each with those files,
/// both sorted without regard to case.
pub fn folders(files: &[PathBuf], mode: &BrowseMode) -> Vec<(PathBuf, Vec<PathBuf>)> {
    let mut grouped: BTreeMap<(String, PathBuf), Vec<PathBuf>> = BTreeMap::new();
    for file in files.iter().filter(|f| matches(f, mode)) {
        let folder = file.parent().map(Path::to_path_buf).unwrap_or_default();
        grouped
            .entry((folder.to_string_lossy().to_lowercase(), folder))
            .or_default()
            .push(file.clone());
    }
    grouped
        .into_iter()
        .map(|((_, folder), mut files)| {
            files.sort_by_cached_key(|f| (sort_key(f), f.clone()));
            (folder, files)
        })
        .collect()
}

fn sort_key(file: &Path) -> String {
    file.file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default()
}

fn matches(file: &Path, mode: &BrowseMode) -> bool {
    match mode {
        BrowseMode::AudioFiles | BrowseMode::Folder => super::files::is_audio(file),
        BrowseMode::OneFile { extensions } => file
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| extensions.iter().any(|x| x.eq_ignore_ascii_case(e))),
    }
}
