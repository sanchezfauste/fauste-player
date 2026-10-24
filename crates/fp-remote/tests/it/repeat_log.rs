//! When a failure that repeats is logged (plan 4, item 2).

use std::time::{Duration, Instant};

use fp_remote::repeat_log::RepeatLog;

#[test]
fn the_same_error_is_logged_once_then_after_the_interval() {
    let every = Duration::from_secs(300);
    let mut log = RepeatLog::new(every);
    let t0 = Instant::now();
    assert!(log.should_log("in use", t0));
    for k in 1..10 {
        assert!(!log.should_log("in use", t0 + Duration::from_secs(2 * k)));
    }
    assert!(log.should_log("in use", t0 + every));
    assert!(log.should_log("permission denied", t0 + every + Duration::from_secs(2)));
    log.clear();
    assert!(log.should_log("permission denied", t0 + every + Duration::from_secs(4)));
}
