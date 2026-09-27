//! True peak: the peak of the signal oversampled 4×, as ITU-R BS.1770
//! recommends for true-peak meters. A 48-tap windowed-sinc interpolator in
//! four phases of 12 taps; each phase is normalised to unity gain.

const FACTOR: usize = 4;
const TAPS_PER_PHASE: usize = 12;
const TAPS: usize = FACTOR * TAPS_PER_PHASE;

/// One channel's interpolator. Real-time safe: fixed arrays, no allocation.
#[derive(Debug, Clone, Copy)]
pub struct TruePeak {
    phases: [[f32; TAPS_PER_PHASE]; FACTOR],
    history: [f32; TAPS_PER_PHASE],
    next: usize,
}

impl Default for TruePeak {
    fn default() -> Self {
        let mut phases = [[0.0f32; TAPS_PER_PHASE]; FACTOR];
        let centre = (TAPS as f64 - 1.0) / 2.0;
        let span = TAPS as f64 - 1.0;
        for (p, phase) in phases.iter_mut().enumerate() {
            let mut sum = 0.0f64;
            let mut taps = [0.0f64; TAPS_PER_PHASE];
            for (k, tap) in taps.iter_mut().enumerate() {
                let n = (FACTOR * k + p) as f64;
                let t = (n - centre) / FACTOR as f64;
                let sinc = if t.abs() < 1e-12 {
                    1.0
                } else {
                    (std::f64::consts::PI * t).sin() / (std::f64::consts::PI * t)
                };
                let x = std::f64::consts::TAU * n / span;
                let window = 0.42 - 0.5 * x.cos() + 0.08 * (2.0 * x).cos();
                *tap = sinc * window;
                sum += *tap;
            }
            for (out, tap) in phase.iter_mut().zip(taps) {
                *out = (tap / sum) as f32;
            }
        }
        Self {
            phases,
            history: [0.0; TAPS_PER_PHASE],
            next: 0,
        }
    }
}

impl TruePeak {
    /// Feeds one sample; returns the largest magnitude among the four
    /// interpolated points it adds.
    pub fn push(&mut self, x: f32) -> f32 {
        if let Some(slot) = self.history.get_mut(self.next) {
            *slot = x;
        }
        self.next = (self.next + 1) % TAPS_PER_PHASE;
        let mut peak = 0.0f32;
        for phase in &self.phases {
            let mut acc = 0.0f32;
            // Newest sample first: history[next - 1 - k].
            for (k, tap) in phase.iter().enumerate() {
                let i = (self.next + TAPS_PER_PHASE - 1 - k) % TAPS_PER_PHASE;
                acc += self.history.get(i).copied().unwrap_or(0.0) * tap;
            }
            peak = peak.max(acc.abs());
        }
        peak
    }
}
