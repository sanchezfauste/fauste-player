//! True peak (ITU-R BS.1770-5, Annex 2): the signal oversampled 4× with
//! the order-48, 4-phase FIR interpolator the Recommendation publishes,
//! and the largest magnitude of the four interpolated phases. Computed in
//! floating point, so the Annex's 12.04 dB headroom step is not needed.

const TAPS_PER_PHASE: usize = 12;

/// ITU-R BS.1770-5, Annex 2: the interpolator's coefficients, per phase.
/// Written as published; every value is a multiple of 2^-13, so each is
/// exact in an `f32` despite the digits.
#[rustfmt::skip]
#[allow(clippy::excessive_precision)]
pub const PHASES: [[f32; TAPS_PER_PHASE]; 4] = [
    [0.001_708_984_375, 0.010_986_328_125, -0.019_653_320_312_5, 0.033_203_125,
     -0.059_448_242_187_5, 0.137_329_101_562_5, 0.972_167_968_75, -0.102_294_921_875,
     0.047_607_421_875, -0.026_611_328_125, 0.014_892_578_125, -0.008_300_781_25],
    [-0.029_174_804_687_5, 0.029_296_875, -0.051_757_812_5, 0.089_111_328_125,
     -0.166_503_906_25, 0.465_087_890_625, 0.779_785_156_25, -0.200_317_382_812_5,
     0.101_562_5, -0.058_227_539_062_5, 0.033_081_054_687_5, -0.018_920_898_437_5],
    [-0.018_920_898_437_5, 0.033_081_054_687_5, -0.058_227_539_062_5, 0.101_562_5,
     -0.200_317_382_812_5, 0.779_785_156_25, 0.465_087_890_625, -0.166_503_906_25,
     0.089_111_328_125, -0.051_757_812_5, 0.029_296_875, -0.029_174_804_687_5],
    [-0.008_300_781_25, 0.014_892_578_125, -0.026_611_328_125, 0.047_607_421_875,
     -0.102_294_921_875, 0.972_167_968_75, 0.137_329_101_562_5, -0.059_448_242_187_5,
     0.033_203_125, -0.019_653_320_312_5, 0.010_986_328_125, 0.001_708_984_375],
];

/// One channel's interpolator. Real-time safe: fixed arrays, no allocation.
#[derive(Debug, Clone, Copy, Default)]
pub struct TruePeak {
    /// The last 12 input samples, twice over so every window is contiguous.
    history: [f32; 2 * TAPS_PER_PHASE],
    next: usize,
}

impl TruePeak {
    /// Feeds one sample; returns the largest magnitude among the four
    /// interpolated points it produces.
    pub fn push(&mut self, x: f32) -> f32 {
        if let Some(slot) = self.history.get_mut(self.next) {
            *slot = x;
        }
        if let Some(slot) = self.history.get_mut(self.next + TAPS_PER_PHASE) {
            *slot = x;
        }
        self.next = (self.next + 1) % TAPS_PER_PHASE;
        // history[next .. next + 12] runs oldest to newest; tap k weighs
        // x[n − k], the newest first.
        let Some(window) = self.history.get(self.next..self.next + TAPS_PER_PHASE) else {
            return x.abs();
        };
        let mut peak = 0.0f32;
        for phase in &PHASES {
            let acc: f32 = phase
                .iter()
                .zip(window.iter().rev())
                .map(|(h, s)| h * s)
                .sum();
            peak = peak.max(acc.abs());
        }
        peak
    }
}
