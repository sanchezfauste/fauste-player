#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

//! The watch building blocks: debounce, folder scan and the system watcher.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use fp_app::watch::FolderTrouble;
use fp_app::watch::debounce::Debouncer;
use fp_app::watch::scan::{ScanJob, scan};
use fp_model::TrackId;

fn touch(path: &Path) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, b"x").unwrap();
}

fn root_job(root: &Path) -> ScanJob {
    ScanJob {
        roots: vec![root.to_path_buf()],
        paths: vec![],
        known: vec![],
    }
}

fn seen_sorted(answer: &fp_app::watch::scan::ScanAnswer) -> Vec<PathBuf> {
    let mut seen: Vec<PathBuf> = answer.found.iter().map(|f| f.seen.clone()).collect();
    seen.sort();
    seen
}

#[test]
fn a_path_is_due_only_after_a_quiet_period() {
    let t0 = Instant::now();
    let p = PathBuf::from("/m/a.mp3");
    let mut d = Debouncer::default();
    d.note(p.clone(), t0);
    d.note(p.clone(), t0 + Duration::from_millis(600));
    assert!(
        d.take_due(t0 + Duration::from_millis(1400), Duration::from_secs(1))
            .is_empty()
    );
    assert_eq!(
        d.take_due(t0 + Duration::from_millis(1600), Duration::from_secs(1)),
        vec![p]
    );
    assert!(d.is_empty());
}

#[test]
fn paths_come_due_independently_and_sorted() {
    let t0 = Instant::now();
    let quiet = Duration::from_secs(1);
    let (a, b, c) = (
        PathBuf::from("/m/a"),
        PathBuf::from("/m/b"),
        PathBuf::from("/m/c"),
    );
    let mut d = Debouncer::default();
    d.note(b.clone(), t0);
    d.note(a.clone(), t0 + Duration::from_millis(500));
    assert_eq!(
        d.take_due(t0 + Duration::from_millis(1100), quiet),
        vec![b.clone()]
    );
    assert!(!d.is_empty());
    assert_eq!(
        d.take_due(t0 + Duration::from_millis(1600), quiet),
        vec![a.clone()]
    );
    d.note(c.clone(), t0);
    d.note(a.clone(), t0);
    assert_eq!(d.take_due(t0 + quiet, quiet), vec![a, c]);
}

#[test]
fn scan_finds_audio_files_in_subfolders_only() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    for f in [
        "a.mp3",
        "B.FLAC",
        "sub/deeper/c.wav",
        "film.mp4",
        "notes.txt",
    ] {
        touch(&root.join(f));
    }
    let answer = scan(&root_job(&root));
    let mut want = vec![
        root.join("a.mp3"),
        root.join("B.FLAC"),
        root.join("sub/deeper/c.wav"),
    ];
    want.sort();
    assert_eq!(seen_sorted(&answer), want);
    assert_eq!(answer.readable, vec![root]);
    assert!(answer.troubles.is_empty());
}

#[test]
fn scan_reports_a_missing_root_without_gone_marks() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("absent");
    let job = ScanJob {
        roots: vec![root.clone()],
        paths: vec![],
        known: vec![(TrackId(1), root.join("x.mp3"))],
    };
    let answer = scan(&job);
    assert_eq!(answer.troubles, vec![(root, FolderTrouble::Missing)]);
    assert!(answer.gone.is_empty());
    assert!(answer.readable.is_empty());
}

#[test]
fn scan_marks_known_tracks_whose_file_is_gone() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    touch(&root.join("here.mp3"));
    let job = ScanJob {
        roots: vec![root.clone()],
        paths: vec![],
        known: vec![
            (TrackId(1), root.join("here.mp3")),
            (TrackId(2), root.join("deleted.mp3")),
        ],
    };
    assert_eq!(scan(&job).gone, vec![TrackId(2)]);
}

#[test]
fn scan_walks_a_touched_folder_and_looks_at_a_touched_file() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    touch(&root.join("new/one.mp3"));
    touch(&root.join("new/two.ogg"));
    touch(&root.join("single.wav"));
    let job = ScanJob {
        roots: vec![],
        paths: vec![root.join("new"), root.join("single.wav")],
        known: vec![],
    };
    let mut want = vec![
        root.join("new/one.mp3"),
        root.join("new/two.ogg"),
        root.join("single.wav"),
    ];
    want.sort();
    assert_eq!(seen_sorted(&scan(&job)), want);
}

#[cfg(unix)]
#[test]
fn scan_does_not_follow_folder_symlinks() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    touch(&root.join("a.mp3"));
    touch(&root.join("sub/b.mp3"));
    std::os::unix::fs::symlink(&root, root.join("loop")).unwrap();
    let answer = scan(&root_job(&root));
    let mut want = vec![root.join("a.mp3"), root.join("sub/b.mp3")];
    want.sort();
    assert_eq!(seen_sorted(&answer), want);
}

#[cfg(not(target_os = "android"))]
#[test]
fn notify_watcher_refuses_a_missing_folder() {
    use fp_app::watch::FolderWatcher;
    use fp_app::watch::notify_watcher::NotifyWatcher;
    let dir = tempfile::tempdir().unwrap();
    let mut w = NotifyWatcher::new().unwrap();
    assert!(w.watch(&dir.path().join("absent")).is_err());
    assert!(w.watch(dir.path()).is_ok());
}
