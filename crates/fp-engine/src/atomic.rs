//! Lock-free scalar cells shared between threads.

use std::sync::atomic::{AtomicU32, Ordering};

/// An `f32` readable and writable from any thread without locks.
#[derive(Debug, Default)]
pub struct AtomicF32(AtomicU32);

impl AtomicF32 {
    pub fn new(value: f32) -> Self {
        Self(AtomicU32::new(value.to_bits()))
    }

    pub fn load(&self) -> f32 {
        f32::from_bits(self.0.load(Ordering::Relaxed))
    }

    pub fn store(&self, value: f32) {
        self.0.store(value.to_bits(), Ordering::Relaxed);
    }

    /// Stores `max(current, value)`.
    pub fn fetch_max(&self, value: f32) {
        let mut current = self.0.load(Ordering::Relaxed);
        while value > f32::from_bits(current) {
            match self.0.compare_exchange_weak(
                current,
                value.to_bits(),
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => return,
                Err(actual) => current = actual,
            }
        }
    }

    /// Returns the value and resets it to zero (peak meters).
    pub fn take(&self) -> f32 {
        f32::from_bits(self.0.swap(0f32.to_bits(), Ordering::Relaxed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_loads_and_tracks_the_maximum() {
        let a = AtomicF32::new(0.25);
        assert_eq!(a.load(), 0.25);
        a.fetch_max(0.5);
        a.fetch_max(0.1);
        assert_eq!(a.take(), 0.5);
        assert_eq!(a.load(), 0.0);
    }
}
