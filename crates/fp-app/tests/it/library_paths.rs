#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

use fp_app::ui::files::{audio_paths, canonical_path};

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
