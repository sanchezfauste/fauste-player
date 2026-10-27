//! Real-time priority for a callback thread the audio system started
//! without it. cpal promotes the threads of its ALSA, PipeWire and WASAPI
//! hosts, but its PulseAudio host renders on the client's own thread at
//! normal priority, where a busy machine (a package upgrade on two cores)
//! holds it back long enough to be heard.
//!
//! Asking the system (rtkit, over D-Bus) is I/O, so it cannot happen on the
//! callback: the callback names its thread once, and a helper thread asks.

use std::thread::{self, Thread};
use std::time::Duration;

/// How long the helper sleeps between looks when nobody wakes it (a stream
/// dropped before its first callback).
const IDLE: Duration = Duration::from_secs(1);

/// The callback's half: hands its thread over once.
pub(crate) struct Promoter<T> {
    slot: rtrb::Producer<T>,
    helper: Thread,
    offered: bool,
}

impl<T: Send + 'static> Promoter<T> {
    /// Starts the helper; `promote` runs on it with the first thread offered.
    /// The helper ends after that, or once the promoter is dropped.
    pub(crate) fn spawn(promote: impl FnOnce(T) + Send + 'static) -> std::io::Result<Self> {
        let (slot, mut offered) = rtrb::RingBuffer::new(1);
        let helper = thread::Builder::new()
            .name("fp-rt-promote".to_owned())
            .spawn(move || {
                loop {
                    if let Ok(info) = offered.pop() {
                        promote(info);
                        return;
                    }
                    if offered.is_abandoned() {
                        return;
                    }
                    thread::park_timeout(IDLE);
                }
            })?;
        Ok(Self {
            slot,
            helper: helper.thread().clone(),
            offered: false,
        })
    }

    /// Called on the callback thread at every callback; only the first call
    /// runs `info` (which names the calling thread) and wakes the helper.
    /// Real-time safe: a push into a preallocated slot and an unpark.
    pub(crate) fn offer(&mut self, info: impl FnOnce() -> Option<T>) {
        if self.offered {
            return;
        }
        self.offered = true;
        if let Some(info) = info() {
            let _ = self.slot.push(info);
            self.helper.unpark();
        }
    }
}

impl<T> Drop for Promoter<T> {
    fn drop(&mut self) {
        self.helper.unpark();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn the_first_offer_is_promoted_on_the_helper_thread() {
        let (tx, rx) = mpsc::channel();
        let mut promoter = Promoter::spawn(move |info: u32| {
            let _ = tx.send((info, thread::current().name().map(str::to_owned)));
        })
        .unwrap();
        promoter.offer(|| Some(7));
        promoter.offer(|| panic!("named once"));
        let (info, name) = rx.recv_timeout(Duration::from_secs(5)).unwrap();
        assert_eq!(info, 7);
        assert_eq!(name.as_deref(), Some("fp-rt-promote"));
        assert!(rx.recv_timeout(Duration::from_millis(100)).is_err());
    }

    #[test]
    fn a_thread_that_cannot_be_named_is_not_promoted() {
        let (tx, rx) = mpsc::channel::<u32>();
        let mut promoter = Promoter::spawn(move |info| {
            let _ = tx.send(info);
        })
        .unwrap();
        promoter.offer(|| None);
        promoter.offer(|| Some(1));
        drop(promoter);
        assert!(rx.recv_timeout(Duration::from_millis(200)).is_err());
    }
}
