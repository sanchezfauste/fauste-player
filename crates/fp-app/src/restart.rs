//! Starting the application again (feedback 2 spec O4). `main` calls
//! `relaunch` after the normal shutdown (session saved, audio stopped,
//! instance lock released). The new process starts like any other start:
//! nothing goes on air by itself.

use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use crate::bootstrap::HOME_VAR;
use crate::instance;

/// How often the instance lock is looked at while waiting for the handoff.
const HANDOFF_POLL: Duration = Duration::from_millis(50);

/// The program to start, and its arguments. No playlist arguments are
/// passed: those of the first start were imported and saved already.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Relaunch {
    pub program: OsString,
    pub args: Vec<OsString>,
}

/// Where the running application came from.
#[derive(Debug, Clone, Default)]
pub struct Launcher {
    /// This executable.
    pub exe: Option<PathBuf>,
    /// The AppImage file (`$APPIMAGE`): its mounted copy goes away with
    /// this process.
    pub appimage: Option<OsString>,
    /// The AppImage's mount folder (`$APPDIR`).
    pub appdir: Option<PathBuf>,
    /// Inside a Flatpak sandbox, which ends with this process.
    pub flatpak: bool,
    /// `FAUSTE_HOME`, which a new Flatpak sandbox does not inherit.
    pub home: Option<OsString>,
}

/// Linux names an executable that a package upgrade replaced
/// `<path> (deleted)`; the new one is at `<path>`.
fn live_path(exe: PathBuf) -> PathBuf {
    match exe.to_str().and_then(|s| s.strip_suffix(" (deleted)")) {
        Some(path) => PathBuf::from(path),
        None => exe,
    }
}

impl Launcher {
    pub fn from_env() -> Self {
        let set = |name: &str| std::env::var_os(name).filter(|v| !v.is_empty());
        Self {
            exe: std::env::current_exe().ok(),
            appimage: set("APPIMAGE"),
            appdir: set("APPDIR").map(PathBuf::from),
            flatpak: set("FLATPAK_ID").is_some(),
            home: set(HOME_VAR),
        }
    }

    /// True when this process must stay until the new instance runs: a
    /// Flatpak sandbox ends with this process, and every process left in
    /// it (the launcher too) is killed.
    pub fn needs_handoff(&self) -> bool {
        self.flatpak && self.own_appimage().is_none()
    }

    /// `$APPIMAGE` when this executable runs from that image's mount
    /// folder. The AppImage runtime passes both variables to every process
    /// started from it, so they may belong to another application.
    fn own_appimage(&self) -> Option<&OsString> {
        let image = self.appimage.as_ref()?;
        let appdir = self.appdir.as_ref()?;
        let exe = self.exe.as_ref()?;
        exe.starts_with(appdir).then_some(image)
    }

    /// How to start the application again; `None` when this executable
    /// is unknown.
    pub fn plan(&self) -> Option<Relaunch> {
        if let Some(image) = self.own_appimage() {
            return Some(Relaunch {
                program: image.clone(),
                args: Vec::new(),
            });
        }
        let exe = live_path(self.exe.clone()?);
        if self.flatpak {
            // A new sandbox through the Flatpak portal, which every app
            // may use.
            let mut args = Vec::new();
            if let Some(home) = &self.home {
                let mut env = OsString::from(format!("--env={HOME_VAR}="));
                env.push(home);
                args.push(env);
            }
            args.push(exe.into_os_string());
            return Some(Relaunch {
                program: "flatpak-spawn".into(),
                args,
            });
        }
        Some(Relaunch {
            program: exe.into_os_string(),
            args: Vec::new(),
        })
    }
}

/// Starts `plan` and leaves it running: this process ends right after.
pub fn spawn(plan: &Relaunch) -> io::Result<Child> {
    Command::new(&plan.program)
        .args(&plan.args)
        .stdin(Stdio::null())
        .spawn()
}

/// How the wait for the new instance ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Handoff {
    /// The new instance holds the instance lock in the data folder.
    Taken,
    /// `timeout` passed first.
    TimedOut,
}

/// Waits, at most `timeout`, until the new instance holds the instance
/// lock in `lock_dir`. An error when `launcher` ends with a failure first.
pub fn wait_for_handoff(
    launcher: &mut Child,
    lock_dir: &Path,
    timeout: Duration,
) -> io::Result<Handoff> {
    let start = Instant::now();
    let deadline = start.checked_add(timeout).unwrap_or(start);
    let mut launcher_done = false;
    loop {
        if instance::is_held(lock_dir)? {
            return Ok(Handoff::Taken);
        }
        if !launcher_done && let Some(status) = launcher.try_wait()? {
            if !status.success() {
                return Err(io::Error::other(format!(
                    "the launcher ended with {status}"
                )));
            }
            // Its work is done; the new instance may still be starting.
            launcher_done = true;
        }
        let now = Instant::now();
        if now >= deadline {
            return Ok(Handoff::TimedOut);
        }
        std::thread::sleep(HANDOFF_POLL.min(deadline - now));
    }
}

/// Starts the application again, the same way it was started. Inside a
/// Flatpak it waits, at most `handoff`, for the new instance to take the
/// instance lock in `lock_dir`.
pub fn relaunch(lock_dir: &Path, handoff: Duration) -> io::Result<()> {
    let launcher = Launcher::from_env();
    let plan = launcher.plan().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "the application's executable is unknown",
        )
    })?;
    tracing::info!(program = ?plan.program, args = ?plan.args, "starting again");
    let mut child = spawn(&plan)?;
    if launcher.needs_handoff() {
        match wait_for_handoff(&mut child, lock_dir, handoff)? {
            Handoff::Taken => tracing::info!("the new instance is running"),
            Handoff::TimedOut => tracing::warn!(
                ?handoff,
                "the new instance did not start in time; ending anyway"
            ),
        }
    }
    Ok(())
}
