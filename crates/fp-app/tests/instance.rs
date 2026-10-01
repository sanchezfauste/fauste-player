#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! One instance at a time: a second start hands its playlists to the
//! running one instead of opening the same devices and state twice.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use fp_app::instance::{acquire, acquire_with_retry, collect, deliver, watch};

#[test]
fn a_lock_held_for_an_instant_is_taken_on_a_retry() {
    // A restarting instance probes the lock while it waits for the new one.
    let dir = tempfile::tempdir().unwrap();
    let probe = acquire(dir.path()).unwrap().unwrap();
    let release = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(10));
        drop(probe);
    });
    // A generous budget (about a second) against a 10 ms hold, so a slow
    // runner cannot fail it; production uses fewer, shorter tries.
    let lock = acquire_with_retry(dir.path(), 50, Duration::from_millis(20)).unwrap();
    release.join().unwrap();
    assert!(lock.is_some());
}

#[test]
fn a_running_instance_still_wins_after_the_retries() {
    let dir = tempfile::tempdir().unwrap();
    let _running = acquire(dir.path()).unwrap().unwrap();
    assert!(
        acquire_with_retry(dir.path(), 3, Duration::from_millis(1))
            .unwrap()
            .is_none()
    );
}

#[test]
fn a_second_instance_cannot_take_the_lock_until_the_first_ends() {
    let dir = tempfile::tempdir().unwrap();
    let first = acquire(dir.path()).unwrap();
    assert!(first.is_some());
    assert!(acquire(dir.path()).unwrap().is_none(), "already running");
    drop(first);
    assert!(acquire(dir.path()).unwrap().is_some(), "free again");
}

#[test]
fn delivered_paths_are_collected_once() {
    let dir = tempfile::tempdir().unwrap();
    let paths = vec![PathBuf::from("/music/Night.m3u"), PathBuf::from("b c.pls")];
    deliver(dir.path(), &paths).unwrap();
    assert_eq!(collect(dir.path()), paths);
    assert!(collect(dir.path()).is_empty(), "taken, not repeated");
}

#[test]
fn the_watcher_passes_delivered_paths_on() {
    let dir = tempfile::tempdir().unwrap();
    let (tx, rx) = crossbeam_channel::unbounded();
    let _watcher = watch(dir.path().to_path_buf(), tx, Duration::from_millis(10)).unwrap();
    deliver(dir.path(), &[PathBuf::from("a.m3u")]).unwrap();
    let got = rx
        .recv_deadline(Instant::now() + Duration::from_secs(5))
        .unwrap();
    assert_eq!(got, PathBuf::from("a.m3u"));
}
