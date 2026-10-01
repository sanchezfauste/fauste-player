//! Starting the application again (feedback 2 spec O4). `main` calls
//! `relaunch` after the normal shutdown (session saved, audio stopped,
//! instance lock released). The new process starts like any other start:
//! nothing goes on air by itself.

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::bootstrap::HOME_VAR;

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
            flatpak: set("FLATPAK_ID").is_some(),
            home: set(HOME_VAR),
        }
    }

    /// How to start the application again; `None` when this executable
    /// is unknown.
    pub fn plan(&self) -> Option<Relaunch> {
        if let Some(image) = &self.appimage {
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
#[allow(clippy::zombie_processes)] // the parent exits; the child is not waited for
pub fn spawn(plan: &Relaunch) -> std::io::Result<()> {
    Command::new(&plan.program)
        .args(&plan.args)
        .stdin(Stdio::null())
        .spawn()
        .map(|_child| ())
}

/// Starts the application again, the same way it was started.
pub fn relaunch() -> std::io::Result<()> {
    let plan = Launcher::from_env().plan().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "the application's executable is unknown",
        )
    })?;
    tracing::info!(program = ?plan.program, args = ?plan.args, "starting again");
    spawn(&plan)
}
