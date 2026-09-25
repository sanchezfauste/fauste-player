//! Choosing which dropped or picked files go into a playlist.

use std::path::{Path, PathBuf};

/// File extensions the decoder can open.
pub const AUDIO_EXTENSIONS: &[&str] = &[
    "aac", "adts", "aif", "aifc", "aiff", "caf", "flac", "m4a", "m4b", "mka", "mkv", "mp1", "mp2",
    "mp3", "mp4", "mpa", "oga", "ogg", "opus", "wav", "wave", "webm",
];

pub fn is_audio(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| AUDIO_EXTENSIONS.iter().any(|a| a.eq_ignore_ascii_case(e)))
}

/// Keeps the audio files, in the given order. A folder is replaced by the
/// audio files directly inside it, sorted by name (sub-folders are not
/// entered). Paths that no longer exist are dropped.
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
            out.extend(inside);
        } else if path.is_file() && is_audio(path) {
            out.push(path.clone());
        }
    }
    out
}
