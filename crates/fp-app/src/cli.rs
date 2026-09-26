//! Command-line arguments: `--version` and `--help` for packages and smoke
//! tests, and playlist files, which a file manager passes when a playlist is
//! opened with the application (Phase 5).

use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// What the binary was asked to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Invocation {
    Version,
    Help,
    /// Start the application, importing `playlists` as new playlists.
    /// `ignored` are paths of other kinds, logged and left alone.
    Run {
        playlists: Vec<PathBuf>,
        ignored: Vec<PathBuf>,
    },
}

/// Extensions of the playlist files the application imports.
const PLAYLIST_EXTENSIONS: &[&str] = &["m3u", "m3u8", "pls"];

fn is_playlist(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()).is_some_and(|e| {
        PLAYLIST_EXTENSIONS
            .iter()
            .any(|p| e.eq_ignore_ascii_case(p))
    })
}

/// Parses the arguments after the program name.
pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Invocation, String> {
    let mut playlists = Vec::new();
    let mut ignored = Vec::new();
    let mut options = true;
    for arg in args {
        if options {
            match arg.to_str() {
                Some("--version" | "-V") => return Ok(Invocation::Version),
                Some("--help" | "-h") => return Ok(Invocation::Help),
                Some("--") => {
                    options = false;
                    continue;
                }
                // macOS passes a process serial number to apps it launches.
                Some(s) if s.starts_with("-psn_") => continue,
                Some(s) if s.starts_with('-') => {
                    return Err(format!("unknown option {s} (see --help)"));
                }
                _ => {}
            }
        }
        let path = PathBuf::from(arg);
        if is_playlist(&path) {
            playlists.push(path);
        } else {
            ignored.push(path);
        }
    }
    Ok(Invocation::Run { playlists, ignored })
}

/// The text `--help` prints.
pub fn usage() -> String {
    format!(
        "Fauste Player {}\n\n\
         Usage: fauste-player [OPTIONS] [PLAYLIST...]\n\n\
         Each PLAYLIST (.m3u, .m3u8, .pls) is imported as a new playlist.\n\n\
         Options:\n  -h, --help     Print this help\n  -V, --version  Print the version\n\n\
         Environment:\n  FAUSTE_HOME    Keep configuration, data and logs in this folder\n",
        env!("CARGO_PKG_VERSION")
    )
}

/// The application icon, for the window and the task bar.
pub fn window_icon() -> Option<egui::IconData> {
    const PNG: &[u8] = include_bytes!("../../../packaging/icons/fauste-player-256.png");
    eframe::icon_data::from_png_bytes(PNG).ok()
}
