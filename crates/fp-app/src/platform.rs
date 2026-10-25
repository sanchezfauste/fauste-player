//! What the interface asks of the system it runs on: file pickers, links,
//! the audio focus and whether user files may be written (mobile spec §4).
//! The desktop uses the native dialogs; the mobile shell brings its own.

use std::path::PathBuf;

pub trait Platform: Send + Sync {
    /// Audio files chosen by the operator, starting in `start`; none when
    /// cancelled. Blocks: call it from a helper thread only (rule 8).
    fn pick_audio_files(&self, filter_name: &str, start: Option<PathBuf>) -> Vec<PathBuf>;
    /// One file with one of `extensions`. Blocks, as `pick_audio_files`.
    fn pick_file(
        &self,
        filter_name: &str,
        extensions: &[&str],
        start: Option<PathBuf>,
    ) -> Option<PathBuf>;
    /// A folder. Blocks, as `pick_audio_files`.
    fn pick_folder(&self, start: Option<PathBuf>) -> Option<PathBuf>;
    /// Where to save `file_name`; an empty `extensions` sets no filter.
    /// Blocks, as `pick_audio_files`.
    fn save_file(
        &self,
        filter_name: &str,
        extensions: &[&str],
        file_name: String,
    ) -> Option<PathBuf>;
    /// Opens a web address or a file with the system; never blocks.
    fn open_url(&self, url: &str) -> Result<(), String>;
    /// Another app holds the audio focus (mobile spec MB4). Playout goes
    /// on; the interface only says so.
    fn audio_focus_lost(&self) -> bool;
    /// Whether the operator may write their own files: tags, covers and
    /// playlist and cart page import and export (read-only on mobile, spec
    /// §2).
    fn can_write_user_files(&self) -> bool;
}

/// The native dialogs (`rfd`) and the system opener (`open`).
pub struct DesktopPlatform;

impl Platform for DesktopPlatform {
    fn pick_audio_files(&self, filter_name: &str, start: Option<PathBuf>) -> Vec<PathBuf> {
        let mut dialog =
            rfd::AsyncFileDialog::new().add_filter(filter_name, crate::ui::files::AUDIO_EXTENSIONS);
        if let Some(dir) = start {
            dialog = dialog.set_directory(dir);
        }
        pollster::block_on(dialog.pick_files())
            .map(|files| files.iter().map(|f| f.path().to_path_buf()).collect())
            .unwrap_or_default()
    }

    fn pick_file(
        &self,
        filter_name: &str,
        extensions: &[&str],
        start: Option<PathBuf>,
    ) -> Option<PathBuf> {
        let mut dialog = rfd::AsyncFileDialog::new().add_filter(filter_name, extensions);
        if let Some(dir) = start {
            dialog = dialog.set_directory(dir);
        }
        pollster::block_on(dialog.pick_file()).map(|f| f.path().to_path_buf())
    }

    fn pick_folder(&self, start: Option<PathBuf>) -> Option<PathBuf> {
        let mut dialog = rfd::AsyncFileDialog::new();
        if let Some(dir) = start {
            dialog = dialog.set_directory(dir);
        }
        pollster::block_on(dialog.pick_folder()).map(|f| f.path().to_path_buf())
    }

    fn save_file(
        &self,
        filter_name: &str,
        extensions: &[&str],
        file_name: String,
    ) -> Option<PathBuf> {
        let mut dialog = rfd::AsyncFileDialog::new();
        if !extensions.is_empty() {
            dialog = dialog.add_filter(filter_name, extensions);
        }
        pollster::block_on(dialog.set_file_name(file_name).save_file())
            .map(|f| f.path().to_path_buf())
    }

    fn open_url(&self, url: &str) -> Result<(), String> {
        let url = url.to_owned();
        std::thread::Builder::new()
            .name("fp-open".to_owned())
            .spawn(move || {
                if let Err(error) = open::that_detached(&url) {
                    tracing::warn!(%error, %url, "could not open the file or link");
                }
            })
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    fn audio_focus_lost(&self) -> bool {
        false
    }

    fn can_write_user_files(&self) -> bool {
        true
    }
}
