use std::path::{Path, PathBuf};

use directories::ProjectDirs;

/// Where the application keeps its files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub log_dir: PathBuf,
}

impl AppPaths {
    /// The OS-standard locations (XDG, %APPDATA%, ~/Library/Application Support).
    pub fn system() -> Option<Self> {
        let dirs = ProjectDirs::from("org", "Fauste", "Fauste Player")?;
        Some(Self {
            config_dir: dirs.config_dir().to_path_buf(),
            data_dir: dirs.data_dir().to_path_buf(),
            cache_dir: dirs.cache_dir().to_path_buf(),
            log_dir: dirs.data_local_dir().join("logs"),
        })
    }

    /// Everything under one root directory (tests, portable installs).
    pub fn under(root: &Path) -> Self {
        Self {
            config_dir: root.join("config"),
            data_dir: root.join("data"),
            cache_dir: root.join("cache"),
            log_dir: root.join("logs"),
        }
    }

    pub fn config_file(&self) -> PathBuf {
        self.config_dir.join("config.json")
    }

    pub fn playlists_file(&self) -> PathBuf {
        self.data_dir.join("playlists.json")
    }

    pub fn carts_file(&self) -> PathBuf {
        self.data_dir.join("carts.json")
    }

    pub fn session_file(&self) -> PathBuf {
        self.data_dir.join("session.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn under_places_files_in_separate_subdirectories() {
        let root = Path::new("/tmp/root");
        let p = AppPaths::under(root);
        assert_eq!(p.config_file(), root.join("config").join("config.json"));
        assert_eq!(p.session_file(), root.join("data").join("session.json"));
    }
}
