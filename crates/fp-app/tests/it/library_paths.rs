#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

use fp_app::services::left_library;
use fp_app::ui::files::{audio_paths, canonical_path};
use fp_model::{CategoryId, FileState, Library, Track, TrackId};

#[cfg(unix)]
#[test]
fn audio_paths_are_canonical() {
    let dir = tempfile::tempdir().unwrap();
    let real = std::fs::canonicalize(dir.path()).unwrap();
    let a = real.join("a.wav");
    std::fs::write(&a, b"x").unwrap();
    let link = real.join("link.wav");
    std::os::unix::fs::symlink(&a, &link).unwrap();
    assert_eq!(audio_paths(std::slice::from_ref(&link)), vec![a.clone()]);

    std::fs::create_dir(real.join("sub")).unwrap();
    let dotted = real.join("sub").join("..").join("a.wav");
    assert_eq!(audio_paths(&[dotted]), vec![a]);
}

#[test]
fn canonical_path_keeps_a_path_that_does_not_exist() {
    let p = PathBuf::from("/definitely/not/here.wav");
    assert_eq!(canonical_path(&p), p);
}

fn track(id: u64, state: FileState, category: Option<u64>) -> Track {
    let mut t = Track::new(TrackId(id), PathBuf::from(format!("/m/{id}.wav")));
    t.file_state = state;
    t.category = category.map(CategoryId);
    t
}

#[test]
fn services_log_a_track_that_left_the_library() {
    let mut old = Library::default();
    old.insert(track(1, FileState::Missing, None));
    old.insert(track(2, FileState::Missing, Some(7)));
    old.insert(track(3, FileState::Ok, None));
    let mut new = old.clone();
    new.remove(TrackId(1));
    new.remove(TrackId(3));
    let left = left_library(&old, &new);
    let paths: Vec<_> = left.iter().map(|t| t.path.clone()).collect();
    assert_eq!(paths, vec![PathBuf::from("/m/1.wav")]);
    assert!(left_library(&new, &new).is_empty());
}

#[test]
fn verbatim_windows_paths_are_simplified() {
    use fp_app::ui::files::simplified_verbatim;
    use std::path::Path;
    let s = |p: &str| simplified_verbatim(Path::new(p));
    assert_eq!(s(r"\\?\C:\Music\a.wav"), Path::new(r"C:\Music\a.wav"));
    assert_eq!(
        s(r"\\?\UNC\server\share\x.wav"),
        Path::new(r"\\server\share\x.wav")
    );
    assert_eq!(s(r"C:\Music\a.wav"), Path::new(r"C:\Music\a.wav"));
    assert_eq!(s("/music/a.wav"), Path::new("/music/a.wav"));
    let volume = r"\\?\Volume{1234}\a.wav";
    assert_eq!(s(volume), Path::new(volume));
    let long = format!(r"\\?\C:\{}", "a".repeat(300));
    assert_eq!(s(&long), Path::new(&long));
}
