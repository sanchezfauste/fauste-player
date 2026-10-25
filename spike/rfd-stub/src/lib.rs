//! SPIKE ONLY: stands in for `rfd` on Android. Pickers answer with the
//! audio files found in /sdcard/Music (matching the filters), so the spike
//! can test real-path access without a picker UI.
#[cfg(not(target_os = "android"))]
pub use rfd::*;
#[cfg(target_os = "android")]
pub use android::*;

#[cfg(target_os = "android")]
mod android {
use std::future::{Future, ready};
use std::path::{Path, PathBuf};

const ROOT: &str = "/sdcard/Music";

pub struct FileHandle(PathBuf);
impl FileHandle {
    pub fn path(&self) -> &Path {
        &self.0
    }
}

#[derive(Default)]
pub struct AsyncFileDialog {
    exts: Vec<String>,
}

impl AsyncFileDialog {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add_filter(mut self, _name: impl Into<String>, exts: &[impl ToString]) -> Self {
        self.exts.extend(exts.iter().map(|e| e.to_string().to_lowercase()));
        self
    }
    pub fn set_directory<P: AsRef<Path>>(self, _dir: P) -> Self {
        self
    }
    pub fn set_file_name(self, _name: impl Into<String>) -> Self {
        self
    }
    pub fn set_title(self, _title: impl Into<String>) -> Self {
        self
    }
    fn files(&self) -> Vec<FileHandle> {
        let mut out: Vec<PathBuf> = std::fs::read_dir(ROOT)
            .map(|d| d.flatten().map(|e| e.path()).collect())
            .unwrap_or_default();
        out.retain(|p| {
            let ext = p.extension().map(|e| e.to_string_lossy().to_lowercase());
            self.exts.is_empty() || ext.is_some_and(|e| self.exts.contains(&e))
        });
        out.sort();
        log_pick(&out);
        out.into_iter().map(FileHandle).collect()
    }
    pub fn pick_files(self) -> impl Future<Output = Option<Vec<FileHandle>>> {
        let f = self.files();
        ready((!f.is_empty()).then_some(f))
    }
    pub fn pick_file(self) -> impl Future<Output = Option<FileHandle>> {
        ready(self.files().into_iter().next())
    }
    pub fn pick_folder(self) -> impl Future<Output = Option<FileHandle>> {
        ready(Some(FileHandle(PathBuf::from(ROOT))))
    }
    pub fn save_file(self) -> impl Future<Output = Option<FileHandle>> {
        ready(None)
    }
}

fn log_pick(paths: &[PathBuf]) {
    tracing::info!(?paths, "rfd-stub picked");
}

#[derive(Clone, Copy)]
pub enum MessageLevel {
    Info,
    Warning,
    Error,
}

#[derive(Default)]
pub struct MessageDialog;
impl MessageDialog {
    pub fn new() -> Self {
        Self
    }
    pub fn set_title(self, _t: impl Into<String>) -> Self {
        self
    }
    pub fn set_description(self, _t: impl Into<String>) -> Self {
        self
    }
    pub fn set_level(self, _l: MessageLevel) -> Self {
        self
    }
    pub fn show(self) {}
}
}
