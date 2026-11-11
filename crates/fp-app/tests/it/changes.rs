#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The change-check thread and its comparison rule (EE7).

use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use fp_app::changes::{ChangeCheck, Check, compare};
use fp_model::{FileStamp, IdGen};

fn stamp(size: u64, modified_ns: u64) -> FileStamp {
    FileStamp { size, modified_ns }
}

#[test]
fn same_stamp_is_same() {
    assert_eq!(compare(Some(stamp(1, 2)), Ok(stamp(1, 2))), Check::Same);
}

#[test]
fn a_different_size_is_changed() {
    assert_eq!(compare(Some(stamp(1, 2)), Ok(stamp(3, 2))), Check::Changed);
}

#[test]
fn a_different_time_is_changed() {
    assert_eq!(compare(Some(stamp(1, 2)), Ok(stamp(1, 9))), Check::Changed);
}

#[test]
fn no_stored_stamp_is_unstamped() {
    assert_eq!(
        compare(None, Ok(stamp(1, 2))),
        Check::Unstamped(stamp(1, 2))
    );
}

#[test]
fn an_error_is_unknown() {
    let err = || Err(io::Error::from(io::ErrorKind::NotFound));
    assert_eq!(compare(Some(stamp(1, 2)), err()), Check::Unknown);
    assert_eq!(compare(None, err()), Check::Unknown);
}

fn wait_answers(check: &mut ChangeCheck) -> Vec<fp_app::changes::ChangeAnswer> {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let answers = check.answers();
        if !answers.is_empty() {
            return answers;
        }
        assert!(Instant::now() < deadline, "the thread answers");
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn stamp_of(path: &std::path::Path) -> FileStamp {
    let meta = std::fs::metadata(path).unwrap();
    FileStamp::from_metadata(meta.len(), meta.modified().unwrap()).unwrap()
}

#[test]
fn the_thread_reports_a_rewritten_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("a.wav");
    std::fs::write(&path, b"first").unwrap();
    let stored = stamp_of(&path);
    std::fs::write(&path, b"a longer second version").unwrap();
    let other = dir.path().join("b.wav");
    std::fs::write(&other, b"x").unwrap();
    let other_stamp = stamp_of(&other);
    let gone = dir.path().join("gone.wav");
    let fresh = dir.path().join("c.wav");
    std::fs::write(&fresh, b"yy").unwrap();

    let mut ids = IdGen::default();
    let (a, b, g, c) = (ids.track(), ids.track(), ids.track(), ids.track());
    let mut check = ChangeCheck::spawn().unwrap();
    assert!(!check.busy());
    assert!(check.send(vec![
        (a, path, Some(stored)),
        (b, other, Some(other_stamp)),
        (g, gone, Some(stamp(1, 1))),
        (c, fresh.clone(), None),
    ]));
    assert!(check.busy());
    assert!(!check.send(Vec::new()), "one batch at a time");
    let answers = wait_answers(&mut check);
    assert!(!check.busy());
    assert_eq!(answers.len(), 1);
    assert_eq!(answers[0].changed, vec![a]);
    assert_eq!(answers[0].unstamped, vec![(c, stamp_of(&fresh))]);
}

/// A thread that ended without answering (it panicked) is noticed: the
/// check is no longer busy and says it is gone, so nothing waits on it.
#[test]
fn a_thread_that_died_is_noticed() {
    let mut ids = IdGen::default();
    let mut check = ChangeCheck::spawn_dying().unwrap();
    assert!(check.send(vec![(ids.track(), PathBuf::from("/nowhere.wav"), None)]));
    let deadline = Instant::now() + Duration::from_secs(10);
    while !check.gone() {
        assert!(check.answers().is_empty());
        assert!(Instant::now() < deadline, "the death is noticed");
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(!check.busy());
}
