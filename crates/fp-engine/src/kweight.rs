//! K-weighting (ITU-R BS.1770): the pre-filter (a high shelf modelling the
//! head) and the RLB high-pass, as two biquads. Coefficients are derived for
//! any rate from the standard's analogue prototypes; at 48 kHz they
//! reproduce the standard's tables.

/// One biquad, normalised so that `a0` = 1.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Biquad {
    pub b0: f64,
    pub b1: f64,
    pub b2: f64,
    pub a1: f64,
    pub a2: f64,
}

/// The pre-filter and the RLB high-pass for `rate`.
pub fn coefficients(rate: u32) -> (Biquad, Biquad) {
    let rate = f64::from(rate.max(1));
    // Pre-filter: high shelf (BS.1770 prototype values).
    let f0 = 1_681.974_450_955_533;
    let gain_db = 3.999_843_853_973_347;
    let q = 0.707_175_236_955_419_6;
    let k = (std::f64::consts::PI * f0 / rate).tan();
    let vh = 10f64.powf(gain_db / 20.0);
    let vb = vh.powf(0.499_666_774_154_541_6);
    let a0 = 1.0 + k / q + k * k;
    let shelf = Biquad {
        b0: (vh + vb * k / q + k * k) / a0,
        b1: 2.0 * (k * k - vh) / a0,
        b2: (vh - vb * k / q + k * k) / a0,
        a1: 2.0 * (k * k - 1.0) / a0,
        a2: (1.0 - k / q + k * k) / a0,
    };
    // RLB weighting: second-order high-pass.
    let f0 = 38.135_470_876_024_44;
    let q = 0.500_327_037_323_877_3;
    let k = (std::f64::consts::PI * f0 / rate).tan();
    let a0 = 1.0 + k / q + k * k;
    let highpass = Biquad {
        b0: 1.0,
        b1: -2.0,
        b2: 1.0,
        a1: 2.0 * (k * k - 1.0) / a0,
        a2: (1.0 - k / q + k * k) / a0,
    };
    (shelf, highpass)
}

#[derive(Debug, Clone, Copy, Default)]
struct State {
    x1: f64,
    x2: f64,
    y1: f64,
    y2: f64,
}

impl State {
    fn process(&mut self, f: &Biquad, x: f64) -> f64 {
        let y = f.b0 * x + f.b1 * self.x1 + f.b2 * self.x2 - f.a1 * self.y1 - f.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

/// K-weighting of one channel. Real-time safe: no allocation.
#[derive(Debug, Clone, Copy, Default)]
pub struct KWeighting {
    shelf: Biquad,
    highpass: Biquad,
    s1: State,
    s2: State,
}

impl KWeighting {
    pub fn new(rate: u32) -> Self {
        let (shelf, highpass) = coefficients(rate);
        Self {
            shelf,
            highpass,
            s1: State::default(),
            s2: State::default(),
        }
    }

    pub fn process(&mut self, x: f64) -> f64 {
        let y = self.s1.process(&self.shelf, x);
        self.s2.process(&self.highpass, y)
    }
}
