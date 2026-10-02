//! Thread priority (main spec §2.2): the decoder threads run above normal
//! priority and the analysis pool below it. Neither is real time: only the
//! device callback is, and a decoder that took more than its share could
//! starve the interface.
//!
//! The system may refuse the change (on Linux, raising a priority needs
//! `RLIMIT_NICE` or `CAP_SYS_NICE`). The thread then keeps the normal
//! priority, and the first refusal of the run is logged; later ones are not,
//! since they would repeat for every thread.

use std::sync::atomic::{AtomicBool, Ordering};

use thread_priority::{ThreadPriority, ThreadPriorityValue, set_current_thread_priority};

/// Above normal on the crate's 0–99 scale (normal is about 50): nice −5 on
/// Linux and "above normal" on Windows.
#[cfg(not(target_os = "macos"))]
const ABOVE_NORMAL: u8 = 60;

/// On macOS the crate checks the value against the range of the normal
/// scheduling policy (15..=47, the default being 31), so 60 would always be
/// refused. 37 is above the default and inside the range; raising within
/// that policy needs no privilege.
#[cfg(target_os = "macos")]
const ABOVE_NORMAL: u8 = 37;

/// What a thread asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    /// Above normal, not real time: decoder threads.
    AboveNormal,
    /// The lowest, short of idle: background analysis.
    Low,
}

impl Priority {
    fn level(self) -> Option<ThreadPriority> {
        match self {
            Priority::Low => Some(ThreadPriority::Min),
            Priority::AboveNormal => ThreadPriorityValue::try_from(ABOVE_NORMAL)
                .ok()
                .map(ThreadPriority::Crossplatform),
        }
    }
}

/// Set once the first refusal was logged.
static REFUSED: AtomicBool = AtomicBool::new(false);

/// True the first time it is called with `flag`, false afterwards.
fn first(flag: &AtomicBool) -> bool {
    !flag.swap(true, Ordering::Relaxed)
}

/// Gives the calling thread `priority`; `role` names it in the log. Returns
/// whether the system accepted. A refusal is logged once per run and
/// otherwise ignored: the thread simply keeps the normal priority.
pub fn set_current(priority: Priority, role: &str) -> bool {
    let Some(level) = priority.level() else {
        return false;
    };
    match set_current_thread_priority(level) {
        Ok(()) => true,
        Err(error) => {
            if first(&REFUSED) {
                tracing::warn!(
                    role,
                    ?priority,
                    %error,
                    "the system refused a thread priority change; threads keep the normal priority"
                );
            }
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_first_refusal_is_reported() {
        let flag = AtomicBool::new(false);
        assert!(first(&flag));
        assert!(!first(&flag));
        assert!(!first(&flag));
    }

    /// The nice value of the calling thread (Linux).
    #[cfg(target_os = "linux")]
    fn nice() -> i32 {
        let stat = std::fs::read_to_string("/proc/thread-self/stat").unwrap();
        // The command name is in parentheses and may hold spaces: count
        // the fields after the last parenthesis (field 19 is the nice).
        let after = stat.rsplit_once(')').unwrap().1;
        after.split_whitespace().nth(16).unwrap().parse().unwrap()
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn low_priority_lowers_the_thread_and_only_that_thread() {
        let before = nice();
        let inside = std::thread::spawn(|| {
            assert!(set_current(Priority::Low, "test"));
            nice()
        })
        .join()
        .unwrap();
        assert_eq!(inside, 19);
        assert_eq!(nice(), before, "the thread that spawned it is untouched");
    }

    /// Whether it is accepted depends on the privileges of whoever runs the
    /// tests; either way the thread ends at the priority the answer says.
    #[cfg(target_os = "linux")]
    #[test]
    fn above_normal_is_accepted_or_leaves_the_thread_alone() {
        let (accepted, inside) = std::thread::spawn(|| {
            let before = nice();
            (set_current(Priority::AboveNormal, "test"), (before, nice()))
        })
        .join()
        .unwrap();
        let (before, after) = inside;
        if accepted {
            assert!(after < before, "{before} -> {after}");
        } else {
            assert_eq!(after, before);
        }
    }

    /// Raising within the normal policy needs no privilege on macOS, so the
    /// request must be accepted there.
    #[cfg(target_os = "macos")]
    #[test]
    fn above_normal_is_accepted_on_macos() {
        let accepted = std::thread::spawn(|| set_current(Priority::AboveNormal, "test"))
            .join()
            .unwrap();
        assert!(accepted);
    }
}
