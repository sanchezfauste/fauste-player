//! Choosing which dropped or picked files go into a playlist.

use std::path::{Path, PathBuf};

/// File extensions of the audio formats that can be played (audio formats
/// spec F1). Video containers (`mkv`, `mp4`, `webm`) are left out so that
/// folder scans do not pick up films.
pub const AUDIO_EXTENSIONS: &[&str] = &[
    "aac", "adts", "aif", "aifc", "aiff", "ape", "caf", "dff", "dsf", "flac", "m4a", "m4b", "mka",
    "mp1", "mp2", "mp3", "oga", "ogg", "opus", "wav", "wave", "weba", "wv",
];

pub fn is_audio(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| AUDIO_EXTENSIONS.iter().any(|a| a.eq_ignore_ascii_case(e)))
}

/// The canonical form of `path` (symlinks and `..` resolved), or the path as
/// given when the file system cannot say (LB14). It touches the file system:
/// call it on a helper thread.
pub fn canonical_path(path: &Path) -> PathBuf {
    match std::fs::canonicalize(path) {
        #[cfg(windows)]
        Ok(canonical) => simplified_verbatim(&canonical),
        #[cfg(not(windows))]
        Ok(canonical) => canonical,
        Err(_) => path.to_path_buf(),
    }
}

/// Windows `canonicalize` answers verbatim paths. `\\?\C:\a.wav` becomes
/// `C:\a.wav` and `\\?\UNC\server\share\a.wav` becomes
/// `\\server\share\a.wav`, the forms every other producer of paths uses, so
/// one file stays one track. Any other verbatim form (`\\?\Volume{..}`), and a
/// path of `MAX_PATH` characters or more (which only the verbatim form can
/// open), stays as it is. Pure string work: it runs on every platform.
pub fn simplified_verbatim(path: &Path) -> PathBuf {
    const MAX_PATH: usize = 260;
    let Some(text) = path.to_str() else {
        return path.to_path_buf();
    };
    let simple = if let Some(rest) = text.strip_prefix(r"\\?\UNC\") {
        Some(format!(r"\\{rest}"))
    } else {
        text.strip_prefix(r"\\?\")
            .filter(|rest| {
                let b = rest.as_bytes();
                b.len() >= 3 && b[0].is_ascii_alphabetic() && b[1] == b':' && b[2] == b'\\'
            })
            .map(str::to_owned)
    };
    match simple {
        Some(s) if s.len() < MAX_PATH => PathBuf::from(s),
        _ => path.to_path_buf(),
    }
}

/// Keeps the audio files, in the given order. A folder is replaced by the
/// audio files directly inside it, sorted by name (sub-folders are not
/// entered). Paths that no longer exist are dropped. The paths come back
/// canonical.
pub fn audio_paths(paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for path in paths {
        if path.is_dir() {
            let Ok(read) = std::fs::read_dir(path) else {
                continue;
            };
            let mut inside: Vec<PathBuf> = read
                .filter_map(Result::ok)
                .map(|e| e.path())
                .filter(|p| p.is_file() && is_audio(p))
                .collect();
            inside.sort_by_key(|p| p.file_name().map(|n| n.to_string_lossy().to_lowercase()));
            out.extend(inside.iter().map(|p| canonical_path(p)));
        } else if path.is_file() && is_audio(path) {
            out.push(canonical_path(path));
        }
    }
    out
}
