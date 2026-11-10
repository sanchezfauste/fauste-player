//! What the interface asks of the system it runs on: file pickers, links,
//! the audio focus and whether user files may be written (mobile spec §4).
//! The desktop uses the native dialogs; the mobile shell brings its own.

use std::path::{Path, PathBuf};

pub trait Platform: Send + Sync {
    /// Audio files chosen by the operator, starting in `start`; none when
    /// cancelled. Blocks: call it from a helper thread only (rule 8).
    fn pick_audio_files(&self, filter_name: &str, start: Option<PathBuf>) -> Vec<PathBuf>;
    /// One file with one of `extensions`; an empty `extensions` sets no
    /// filter. Blocks, as `pick_audio_files`.
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
    /// The operator left with Back and there was nothing to go back to
    /// (phone layout spec PL23, PL24): with `playing` the app goes to the
    /// background and plays on, otherwise it saves and ends. Nothing by
    /// default: a desktop window closes through its own button.
    fn leave(&self, _playing: bool) {}
    /// Whether this system can start another program (the external editor,
    /// spec EE3): not inside a Flatpak or snap sandbox, not on Android.
    fn can_launch_programs(&self) -> bool;
    /// Starts `program` with `file` as its only argument, on a helper
    /// thread, and never makes the interface wait for it. `done` gets the
    /// result of the start (not of the editor's work).
    fn launch_editor(
        &self,
        program: &Path,
        file: &Path,
        done: Box<dyn FnOnce(Result<(), String>) + Send>,
    );
}

/// The command that starts `program` on `file`: through `open -a` for a
/// macOS application bundle, otherwise the program itself. No shell and no
/// string interpolation: the paths are passed as arguments.
#[cfg(not(target_os = "android"))]
fn editor_command(program: &Path, file: &Path) -> std::process::Command {
    let bundle = program
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("app"));
    if cfg!(target_os = "macos") && bundle {
        let mut command = std::process::Command::new("open");
        command.arg("-a").arg(program).arg(file);
        command
    } else {
        let mut command = std::process::Command::new(program);
        command.arg(file);
        command
    }
}

/// The native dialogs (`rfd`) and the system opener (`open`).
#[cfg(not(target_os = "android"))]
pub struct DesktopPlatform;

#[cfg(not(target_os = "android"))]
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
        let mut dialog = rfd::AsyncFileDialog::new();
        if !extensions.is_empty() {
            dialog = dialog.add_filter(filter_name, extensions);
        }
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

    fn can_launch_programs(&self) -> bool {
        std::env::var_os("FLATPAK_ID").is_none() && std::env::var_os("SNAP").is_none()
    }

    /// The `fp-editor` thread spawns the program, reports the start and
    /// then waits for the editor to end, so that no zombie process stays.
    fn launch_editor(
        &self,
        program: &Path,
        file: &Path,
        done: Box<dyn FnOnce(Result<(), String>) + Send>,
    ) {
        let mut command = editor_command(program, file);
        let slot = std::sync::Arc::new(std::sync::Mutex::new(Some(done)));
        let in_thread = slot.clone();
        let spawned = std::thread::Builder::new()
            .name("fp-editor".to_owned())
            .spawn(move || {
                let Some(done) = in_thread.lock().ok().and_then(|mut d| d.take()) else {
                    return;
                };
                match command.spawn() {
                    Ok(mut child) => {
                        done(Ok(()));
                        let _ = child.wait();
                    }
                    Err(e) => done(Err(e.to_string())),
                }
            });
        if let Err(e) = spawned
            && let Some(done) = slot.lock().ok().and_then(|mut d| d.take())
        {
            done(Err(e.to_string()));
        }
    }
}

/// What the interface uses on Android until the mobile shell gives its
/// own platform: nothing can be picked, opened or written.
#[cfg(target_os = "android")]
pub struct NoPlatform;

#[cfg(target_os = "android")]
impl Platform for NoPlatform {
    fn pick_audio_files(&self, _filter_name: &str, _start: Option<PathBuf>) -> Vec<PathBuf> {
        Vec::new()
    }

    fn pick_file(
        &self,
        _filter_name: &str,
        _extensions: &[&str],
        _start: Option<PathBuf>,
    ) -> Option<PathBuf> {
        None
    }

    fn pick_folder(&self, _start: Option<PathBuf>) -> Option<PathBuf> {
        None
    }

    fn save_file(
        &self,
        _filter_name: &str,
        _extensions: &[&str],
        _file_name: String,
    ) -> Option<PathBuf> {
        None
    }

    fn open_url(&self, _url: &str) -> Result<(), String> {
        Err("not available".to_owned())
    }

    fn audio_focus_lost(&self) -> bool {
        false
    }

    fn can_write_user_files(&self) -> bool {
        false
    }

    fn can_launch_programs(&self) -> bool {
        false
    }

    fn launch_editor(
        &self,
        _program: &Path,
        _file: &Path,
        done: Box<dyn FnOnce(Result<(), String>) + Send>,
    ) {
        done(Err("not available".to_owned()));
    }
}
