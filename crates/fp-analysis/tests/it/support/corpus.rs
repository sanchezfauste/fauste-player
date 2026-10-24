//! The local real-music corpus: `FAUSTE_TEST_MUSIC`, else `test-music/` at
//! the repository root (see its README). It is never in git and never in CI.

use std::path::{Path, PathBuf};

/// Extensions the player reads (kept in step with `fp-app`'s list).
const EXTENSIONS: &[&str] = &[
    "aac", "adts", "aif", "aifc", "aiff", "ape", "caf", "dff", "dsf", "flac", "m4a", "m4b", "mka",
    "mp1", "mp2", "mp3", "oga", "ogg", "opus", "wav", "wave", "weba", "wv",
];

/// The corpus folder in use.
pub fn dir() -> PathBuf {
    std::env::var_os("FAUSTE_TEST_MUSIC").map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test-music"),
        PathBuf::from,
    )
}

/// Every audio file of the corpus, sorted; empty (with a note) when there
/// is none.
pub fn files() -> Vec<PathBuf> {
    files_in(&dir())
}

/// Every audio file under `root`, sorted; empty (with a note) when there is
/// none.
pub fn files_in(root: &Path) -> Vec<PathBuf> {
    let root = root.to_owned();
    let mut out = Vec::new();
    let mut pending = vec![root.clone()];
    while let Some(d) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| EXTENSIONS.iter().any(|x| x.eq_ignore_ascii_case(e)))
            {
                out.push(path);
            }
        }
    }
    out.sort();
    if out.is_empty() {
        eprintln!(
            "no real-music corpus in {} (see test-music/README.md); nothing to check",
            root.display()
        );
    }
    out
}
