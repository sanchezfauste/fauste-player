//! Per-sample gain ramps. Fades use an equal-power (sine/cosine) curve so a
//! crossfade keeps constant loudness; de-click ramps are linear.

use std::f32::consts::FRAC_PI_2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Curve {
    Linear,
    EqualPower,
}

/// Moves a gain from `from` to `to` over `len` samples, then holds `to`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ramp {
    from: f32,
    to: f32,
    len: u32,
    pos: u32,
    curve: Curve,
}

impl Ramp {
    /// A constant gain.
    pub fn hold(gain: f32) -> Self {
        Self {
            from: gain,
            to: gain,
            len: 0,
            pos: 0,
            curve: Curve::Linear,
        }
    }

    /// Starts a ramp from the current value of `self` to `to`.
    pub fn retarget(&mut self, to: f32, len: u32, curve: Curve) {
        *self = Self {
            from: self.value(),
            to,
            len,
            pos: 0,
            curve,
        };
    }

    /// Current gain without advancing.
    pub fn value(&self) -> f32 {
        if self.pos >= self.len {
            return self.to;
        }
        let t = self.pos as f32 / self.len as f32;
        let shaped = match self.curve {
            Curve::Linear => t,
            Curve::EqualPower if self.to >= self.from => (t * FRAC_PI_2).sin(),
            Curve::EqualPower => 1.0 - (t * FRAC_PI_2).cos(),
        };
        self.from + (self.to - self.from) * shaped
    }

    /// Returns the gain for the next sample and advances.
    pub fn next_gain(&mut self) -> f32 {
        let g = self.value();
        if self.pos < self.len {
            self.pos += 1;
        }
        g
    }

    pub fn is_done(&self) -> bool {
        self.pos >= self.len
    }

    pub fn target(&self) -> f32 {
        self.to
    }

    /// Samples left before the ramp reaches its target.
    pub fn remaining(&self) -> u32 {
        self.len - self.pos.min(self.len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_ramp_hits_exact_values_and_holds() {
        let mut r = Ramp::hold(0.0);
        r.retarget(1.0, 4, Curve::Linear);
        let g: Vec<f32> = (0..6).map(|_| r.next_gain()).collect();
        assert_eq!(g, vec![0.0, 0.25, 0.5, 0.75, 1.0, 1.0]);
        assert!(r.is_done());
    }

    #[test]
    fn equal_power_fade_out_keeps_constant_power_against_a_fade_in() {
        let mut out = Ramp::hold(1.0);
        out.retarget(0.0, 100, Curve::EqualPower);
        let mut inn = Ramp::hold(0.0);
        inn.retarget(1.0, 100, Curve::EqualPower);
        for _ in 0..100 {
            let (a, b) = (out.next_gain(), inn.next_gain());
            assert!((a * a + b * b - 1.0).abs() < 1e-5, "{a} {b}");
        }
        assert_eq!(out.next_gain(), 0.0);
    }

    #[test]
    fn retarget_starts_from_the_current_value() {
        let mut r = Ramp::hold(0.0);
        r.retarget(1.0, 2, Curve::Linear);
        r.next_gain();
        r.retarget(0.0, 2, Curve::Linear);
        assert_eq!(r.next_gain(), 0.5);
        assert_eq!(r.next_gain(), 0.25);
        assert_eq!(r.next_gain(), 0.0);
    }
}
