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

/// Whether a macOS application bundle is started through `open -a`.
#[cfg(not(target_os = "android"))]
fn through_open(program: &Path) -> bool {
    cfg!(target_os = "macos")
        && program
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("app"))
}

/// The command that starts `program` on `file`: through `open -a` for a
/// macOS application bundle, otherwise the program itself. No shell and no
/// string interpolation: the paths are passed as arguments. The editor
/// gets no terminal: its standard streams are null.
#[cfg(not(target_os = "android"))]
fn editor_command(program: &Path, file: &Path) -> std::process::Command {
    use std::process::Stdio;
    let mut command = if through_open(program) {
        let mut command = std::process::Command::new("open");
        command.arg("-a").arg(program).arg(file);
        command
    } else {
        let mut command = std::process::Command::new(program);
        command.arg(file);
        command
    };
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command
}

/// Runs `command` on the calling thread (the `fp-editor` helper). With
/// `wait` (`open -a`, which ends once the application has the file), the
/// exit status is the result: a bundle that cannot be found fails there.
/// Otherwise the start is the result, reported at once, and the thread
/// then waits for the editor to end so that no zombie process stays.
#[cfg(not(target_os = "android"))]
fn run_launch(
    mut command: std::process::Command,
    wait: bool,
    done: Box<dyn FnOnce(Result<(), String>) + Send>,
) {
    match command.spawn() {
        Ok(mut child) if wait => done(match child.wait() {
            Ok(status) if status.success() => Ok(()),
            Ok(status) => Err(format!("the opener ended with {status}")),
            Err(e) => Err(e.to_string()),
        }),
        Ok(mut child) => {
            done(Ok(()));
            let _ = child.wait();
        }
        Err(e) => done(Err(e.to_string())),
    }
}

/// Whether another program may be started (spec §6): not inside a Flatpak
/// (`/.flatpak-info` exists there) nor a snap (`SNAP` is set and holds the
/// running program; a stray `SNAP` in the environment does not count).
#[cfg(not(target_os = "android"))]
fn launch_allowed(flatpak_info: bool, snap: Option<&std::ffi::OsStr>, exe: Option<&Path>) -> bool {
    let in_snap = match (snap, exe) {
        (Some(snap), Some(exe)) if !snap.is_empty() => exe.starts_with(snap),
        _ => false,
    };
    !flatpak_info && !in_snap
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

    /// Looked at once per run (the menu asks every frame).
    fn can_launch_programs(&self) -> bool {
        static ALLOWED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *ALLOWED.get_or_init(|| {
            let exe = std::env::current_exe().ok();
            launch_allowed(
                Path::new("/.flatpak-info").exists(),
                std::env::var_os("SNAP").as_deref(),
                exe.as_deref(),
            )
        })
    }

    /// The `fp-editor` thread starts the program (see `run_launch`).
    fn launch_editor(
        &self,
        program: &Path,
        file: &Path,
        done: Box<dyn FnOnce(Result<(), String>) + Send>,
    ) {
        let command = editor_command(program, file);
        let wait = through_open(program);
        let slot = std::sync::Arc::new(std::sync::Mutex::new(Some(done)));
        let in_thread = slot.clone();
        let spawned = std::thread::Builder::new()
            .name("fp-editor".to_owned())
            .spawn(move || {
                let Some(done) = in_thread.lock().ok().and_then(|mut d| d.take()) else {
                    return;
                };
                run_launch(command, wait, done);
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

#[cfg(all(test, unix, not(target_os = "android")))]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use std::ffi::OsStr;
    use std::sync::mpsc;
    use std::time::Duration;

    const WAIT: Duration = Duration::from_secs(30);

    fn launched(command: std::process::Command, wait: bool) -> Result<(), String> {
        let (tx, rx) = mpsc::channel();
        run_launch(
            command,
            wait,
            Box::new(move |r| {
                let _ = tx.send(r);
            }),
        );
        rx.recv_timeout(WAIT).unwrap()
    }

    /// M6: a Flatpak is told by `/.flatpak-info`; a snap by `SNAP` holding
    /// the running program (a stray `SNAP` in the environment is not one).
    #[test]
    fn the_sandbox_is_told_by_its_own_marks() {
        let exe = Path::new("/snap/fauste-player/12/bin/fauste-player");
        let snap = OsStr::new("/snap/fauste-player/12");
        assert!(launch_allowed(false, None, Some(exe)));
        assert!(!launch_allowed(true, None, Some(exe)));
        assert!(!launch_allowed(false, Some(snap), Some(exe)));
        assert!(launch_allowed(
            false,
            Some(snap),
            Some(Path::new("/usr/bin/fauste-player"))
        ));
        assert!(launch_allowed(false, Some(OsStr::new("")), Some(exe)));
        assert!(launch_allowed(false, Some(snap), None));
    }

    /// M4: when the start is only known once the opener ends (`open -a`
    /// on macOS), its exit status decides: a failure is reported.
    #[test]
    fn a_waited_launch_reports_a_failed_exit() {
        assert!(launched(std::process::Command::new("false"), true).is_err());
        assert_eq!(launched(std::process::Command::new("true"), true), Ok(()));
    }

    /// M4: a program started directly reports its start at once, whatever
    /// it does later.
    #[test]
    fn a_direct_launch_reports_the_start() {
        assert_eq!(launched(std::process::Command::new("false"), false), Ok(()));
        assert!(launched(std::process::Command::new("/nonexistent/editor"), false).is_err());
    }

    /// M5: the editor gets no terminal: stdin, stdout and stderr are null.
    #[cfg(target_os = "linux")]
    #[test]
    fn the_editor_gets_null_standard_streams() {
        let dir = tempfile::tempdir().unwrap();
        let script = dir.path().join("check.sh");
        std::fs::write(
            &script,
            "for n in 0 1 2; do [ \"$(readlink /proc/$$/fd/$n)\" = /dev/null ] || exit 1; done\n",
        )
        .unwrap();
        let command = editor_command(Path::new("/bin/sh"), &script);
        assert_eq!(launched(command, true), Ok(()));
    }
}
