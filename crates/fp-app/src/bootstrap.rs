//! Where the application keeps its files, and process-wide setup.

use std::ffi::OsString;
use std::path::PathBuf;

use fp_store::AppPaths;

/// Environment variable that puts every file under one directory (portable
/// installs, several instances side by side, tests).
pub const HOME_VAR: &str = "FAUSTE_HOME";

/// `FAUSTE_HOME` if set, else the OS-standard directories.
pub fn paths_from(home: Option<OsString>) -> Option<AppPaths> {
    match home.filter(|h| !h.is_empty()) {
        Some(root) => Some(AppPaths::under(&PathBuf::from(root))),
        None => AppPaths::system(),
    }
}

pub fn paths() -> Option<AppPaths> {
    paths_from(std::env::var_os(HOME_VAR))
}
