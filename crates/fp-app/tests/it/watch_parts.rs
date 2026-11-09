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
        ..ScanJob::default()
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
        ..ScanJob::default()
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
        ..ScanJob::default()
    };
    assert_eq!(scan(&job).gone, vec![TrackId(2)]);
}

/// External editor spec EE7.2: a known track gone under an event path (an
/// editor may be replacing it) is reported apart, to be looked at again; a
/// file found under one is marked as found by an event.
#[test]
fn scan_reports_a_track_gone_after_an_event_apart() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    touch(&root.join("here.mp3"));
    let job = ScanJob {
        paths: vec![root.join("here.mp3"), root.join("gone.mp3")],
        known: vec![
            (TrackId(1), root.join("here.mp3")),
            (TrackId(2), root.join("gone.mp3")),
        ],
        ..ScanJob::default()
    };
    let answer = scan(&job);
    assert!(answer.gone.is_empty());
    assert_eq!(answer.vanished, vec![TrackId(2)]);
    assert_eq!(answer.found.len(), 1);
    assert!(answer.found[0].event);
    let walked = scan(&root_job(&root));
    assert!(walked.found.iter().all(|f| !f.event));
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
        ..ScanJob::default()
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

#[cfg(unix)]
#[test]
fn scan_does_not_mark_tracks_gone_when_a_stat_is_refused() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    touch(&root.join("sub/x.mp3"));
    let sub = root.join("sub");
    std::fs::set_permissions(&sub, std::fs::Permissions::from_mode(0o000)).unwrap();
    let job = ScanJob {
        roots: vec![root.clone()],
        paths: vec![],
        known: vec![(TrackId(1), root.join("sub/x.mp3"))],
        ..ScanJob::default()
    };
    let answer = scan(&job);
    std::fs::set_permissions(&sub, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(answer.gone.is_empty());
}

#[cfg(unix)]
#[test]
fn scan_does_not_walk_a_touched_folder_symlink() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    touch(&root.join("real/a.mp3"));
    std::os::unix::fs::symlink(root.join("real"), root.join("link")).unwrap();
    let job = ScanJob {
        roots: vec![],
        paths: vec![root.join("link")],
        known: vec![],
        ..ScanJob::default()
    };
    assert!(scan(&job).found.is_empty());
}

#[test]
fn the_scanner_is_busy_until_every_job_is_answered() {
    use fp_app::watch::scan::Scanner;
    let dir = tempfile::tempdir().unwrap();
    let mut s = Scanner::spawn().unwrap();
    assert!(!s.busy());
    assert!(s.send(root_job(dir.path())));
    assert!(s.send(root_job(dir.path())));
    assert!(s.busy());
    let mut got = 0;
    let end = Instant::now() + Duration::from_secs(10);
    while got < 2 && Instant::now() < end {
        got += s.answers().len();
        if got == 1 {
            assert!(s.busy());
        }
        std::thread::yield_now();
    }
    assert_eq!(got, 2);
    assert!(!s.busy());
}

#[cfg(not(target_os = "android"))]
mod convert {
    use fp_app::watch::FolderEvent;
    use fp_app::watch::notify_watcher::convert;
    use notify::event::{AccessKind, AccessMode, CreateKind, ModifyKind, RemoveKind, RenameMode};
    use notify::{Event, EventKind};
    use std::path::PathBuf;

    fn ev(kind: EventKind) -> Event {
        Event::new(kind).add_path(PathBuf::from("/m/a.mp3"))
    }
    fn touched() -> Vec<FolderEvent> {
        vec![FolderEvent::Touched(PathBuf::from("/m/a.mp3"))]
    }

    #[test]
    fn access_events_are_dropped_except_a_finished_write() {
        assert!(convert(Ok(ev(EventKind::Access(AccessKind::Open(AccessMode::Any))))).is_empty());
        assert!(
            convert(Ok(ev(EventKind::Access(AccessKind::Close(
                AccessMode::Read
            )))))
            .is_empty()
        );
        assert_eq!(
            convert(Ok(ev(EventKind::Access(AccessKind::Close(
                AccessMode::Write
            ))))),
            touched()
        );
    }

    #[test]
    fn changes_are_touched() {
        for kind in [
            EventKind::Create(CreateKind::File),
            EventKind::Modify(ModifyKind::Any),
            EventKind::Remove(RemoveKind::File),
        ] {
            assert_eq!(convert(Ok(ev(kind))), touched());
        }
        let rename = Event::new(EventKind::Modify(ModifyKind::Name(RenameMode::Both)))
            .add_path(PathBuf::from("/m/old.mp3"))
            .add_path(PathBuf::from("/m/new.mp3"));
        assert_eq!(
            convert(Ok(rename)),
            vec![
                FolderEvent::Touched(PathBuf::from("/m/old.mp3")),
                FolderEvent::Touched(PathBuf::from("/m/new.mp3"))
            ]
        );
    }

    #[test]
    fn a_rescan_flag_or_an_error_is_a_rescan() {
        let flagged = ev(EventKind::Other).set_flag(notify::event::Flag::Rescan);
        assert_eq!(convert(Ok(flagged)), vec![FolderEvent::Rescan]);
        let err = notify::Error::generic("lost");
        assert_eq!(convert(Err(err)), vec![FolderEvent::Rescan]);
    }
}

#[test]
fn scan_reports_an_empty_root_holding_known_tracks_missing() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    let job = ScanJob {
        roots: vec![root.clone()],
        known: vec![(TrackId(1), root.join("x.mp3"))],
        ..ScanJob::default()
    };
    let answer = scan(&job);
    assert_eq!(answer.troubles, vec![(root, FolderTrouble::Missing)]);
    assert!(answer.gone.is_empty());
    assert!(answer.readable.is_empty());
}

#[test]
fn scan_walks_an_empty_root_with_no_known_tracks() {
    let dir = tempfile::tempdir().unwrap();
    let answer = scan(&root_job(dir.path()));
    assert!(answer.troubles.is_empty());
    assert_eq!(answer.readable, vec![dir.path().to_path_buf()]);
}

#[test]
fn scan_checks_a_folder_without_walking_it() {
    let dir = tempfile::tempdir().unwrap();
    let here = dir.path().join("here");
    let absent = dir.path().join("absent");
    let empty = dir.path().join("empty");
    touch(&here.join("a.mp3"));
    std::fs::create_dir_all(&empty).unwrap();
    let job = ScanJob {
        checks: vec![here.clone(), absent.clone(), empty.clone()],
        holding: vec![empty.clone()],
        // An event path under the absent folder: not marked.
        paths: vec![absent.join("b.mp3")],
        known: vec![(TrackId(1), absent.join("b.mp3"))],
        ..ScanJob::default()
    };
    let answer = scan(&job);
    assert!(answer.found.is_empty(), "a check walks nothing");
    let mut troubles = answer.troubles.clone();
    troubles.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(
        troubles,
        vec![
            (absent, FolderTrouble::Missing),
            (empty, FolderTrouble::Missing)
        ]
    );
    assert!(answer.gone.is_empty());
    assert!(answer.readable.is_empty(), "a check clears nothing");
}

#[cfg(target_os = "linux")]
#[test]
fn notify_watcher_does_not_follow_folder_symlinks() {
    use fp_app::watch::notify_watcher::NotifyWatcher;
    use fp_app::watch::{FolderEvent, FolderWatcher};
    let dir = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let root = dir.path().join("root");
    std::fs::create_dir_all(&root).unwrap();
    std::os::unix::fs::symlink(outside.path(), root.join("link")).unwrap();
    let mut w = NotifyWatcher::new().unwrap();
    w.watch(&root).unwrap();
    // The outside write first: a followed link would report it before the
    // inside one, which arrives through the same queue.
    touch(&outside.path().join("out.mp3"));
    touch(&root.join("in.mp3"));
    let mut events = Vec::new();
    let end = Instant::now() + Duration::from_secs(10);
    while Instant::now() < end
        && !events
            .iter()
            .any(|e| *e == FolderEvent::Touched(root.join("in.mp3")))
    {
        events.extend(w.events());
        std::thread::yield_now();
    }
    assert!(
        events
            .iter()
            .any(|e| *e == FolderEvent::Touched(root.join("in.mp3")))
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, FolderEvent::Touched(p) if p.starts_with(root.join("link")))),
        "{events:?}"
    );
}
