//! Level meters (meters spec M2): standard ballistics and EBU R128 loudness,
//! run by the conductor on what the mixer measured. Pure: time comes in as
//! `dt_secs`, settings as `&MeterConfig`.

use fp_model::{MeterBallistics, MeterConfig};

/// The lowest level a meter reports, in dBFS (below any display floor).
pub const SILENCE_DB: f32 = -120.0;
/// Loudness blocks (ITU-R BS.1770 uses 100 ms steps).
const BIN_SECS: f64 = 0.1;
const MOMENTARY_BINS: usize = 4;
const SHORT_TERM_BINS: usize = 30;
/// A sine's RMS is 3.01 dB under its peak; RMS meters are calibrated so a
/// sine reads its peak level (AES17).
const SINE_RMS_OFFSET_DB: f32 = 3.010_3;

/// What a player's sources measured since the last tick (mixer, meters spec M1).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct MeterInput {
    /// Sample or true peak, linear.
    pub peak: [f32; 2],
    pub sum_sq: [f64; 2],
    /// K-weighted sums of squares.
    pub k_sum: [f64; 2],
    pub frames: u64,
}

impl MeterInput {
    /// Combines two sources of one player (a crossfade).
    pub fn merge(&mut self, other: MeterInput) {
        for ch in 0..2 {
            if let (Some(p), Some(o)) = (self.peak.get_mut(ch), other.peak.get(ch)) {
                *p = p.max(*o);
            }
            if let (Some(s), Some(o)) = (self.sum_sq.get_mut(ch), other.sum_sq.get(ch)) {
                *s += o;
            }
            if let (Some(k), Some(o)) = (self.k_sum.get_mut(ch), other.k_sum.get(ch)) {
                *k += o;
            }
        }
        // Both sources play over the same time: the frames are the same span.
        self.frames = self.frames.max(other.frames);
    }
}

/// What the meter shows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeterReading {
    pub level_db: [f32; 2],
    pub hold_db: [f32; 2],
    pub momentary_lufs: Option<f32>,
    pub short_term_lufs: Option<f32>,
}

impl Default for MeterReading {
    fn default() -> Self {
        Self {
            level_db: [SILENCE_DB; 2],
            hold_db: [SILENCE_DB; 2],
            momentary_lufs: None,
            short_term_lufs: None,
        }
    }
}

/// How a preset moves: the input it reads, its rise time constant and its
/// fall rate.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Ballistics {
    rms: bool,
    attack_tau: f64,
    release_db_per_sec: f32,
}

fn ballistics(c: &MeterConfig) -> Ballistics {
    // A first-order rise with τ = integration / 3 reads a burst of the
    // integration time within about 0.5 dB of steady state.
    let peak = |integration_ms: f64, fall_db: f32, fall_secs: f32| Ballistics {
        rms: false,
        attack_tau: integration_ms / 3_000.0,
        release_db_per_sec: fall_db / fall_secs,
    };
    match c.ballistics {
        MeterBallistics::DigitalPeak => peak(0.0, 20.0, 1.7),
        MeterBallistics::EbuPpm => peak(10.0, 24.0, 2.8),
        MeterBallistics::DinPpm => peak(5.0, 20.0, 1.7),
        // 99 % in 300 ms: τ = 0.3 / ln(100), symmetric.
        MeterBallistics::Vu => Ballistics {
            rms: true,
            attack_tau: 0.3 / 100f64.ln(),
            release_db_per_sec: 20.0 / 1.7,
        },
        MeterBallistics::Custom => Ballistics {
            rms: false,
            attack_tau: f64::from(c.attack_ms) / 3_000.0,
            release_db_per_sec: c.release_db_per_sec,
        },
    }
}

fn to_db(linear: f32) -> f32 {
    if linear > 0.0 {
        (20.0 * linear.log10()).max(SILENCE_DB)
    } else {
        SILENCE_DB
    }
}

fn from_db(db: f32) -> f32 {
    if db <= SILENCE_DB {
        0.0
    } else {
        10f32.powf(db / 20.0)
    }
}

/// One 100 ms loudness block: K-weighted sums and the frames they cover.
#[derive(Debug, Clone, Copy, Default)]
struct Bin {
    k_sum: [f64; 2],
    frames: u64,
}

/// The last 3 s of 100 ms blocks, in a fixed ring.
#[derive(Debug, Clone, Copy, Default)]
struct LoudnessWindow {
    bins: [Bin; SHORT_TERM_BINS],
    next: usize,
    filled: usize,
    current: Bin,
    current_secs: f64,
}

impl LoudnessWindow {
    fn add(&mut self, input: &MeterInput, dt: f64) {
        for (k, add) in self.current.k_sum.iter_mut().zip(input.k_sum) {
            *k += add;
        }
        self.current.frames += input.frames;
        self.current_secs += dt;
        if self.current_secs + 1e-9 >= BIN_SECS {
            if let Some(slot) = self.bins.get_mut(self.next) {
                *slot = self.current;
            }
            self.next = (self.next + 1) % SHORT_TERM_BINS;
            self.filled = (self.filled + 1).min(SHORT_TERM_BINS);
            self.current = Bin::default();
            self.current_secs -= BIN_SECS;
        }
    }

    /// Loudness over the last `count` blocks; `None` for silence or no audio.
    fn lufs(&self, count: usize) -> Option<f32> {
        if self.filled < count {
            return None;
        }
        let mut sum = [0.0f64; 2];
        let mut frames = 0u64;
        for back in 1..=count {
            let i = (self.next + SHORT_TERM_BINS - back) % SHORT_TERM_BINS;
            let bin = self.bins.get(i)?;
            sum[0] += bin.k_sum[0];
            sum[1] += bin.k_sum[1];
            frames += bin.frames;
        }
        if frames == 0 {
            return None;
        }
        let z = (sum[0] + sum[1]) / frames as f64;
        (z > 0.0).then(|| (-0.691 + 10.0 * z.log10()) as f32)
    }
}

/// One player's meter.
#[derive(Debug, Clone, Copy, Default)]
pub struct MeterState {
    linear: [f32; 2],
    hold_db: [f32; 2],
    hold_left: [f32; 2],
    loudness: LoudnessWindow,
    started: bool,
}

impl MeterState {
    /// Advances the meter by `dt_secs` with what was measured meanwhile.
    pub fn update(&mut self, input: MeterInput, dt_secs: f64, c: &MeterConfig) -> MeterReading {
        if !self.started {
            self.hold_db = [SILENCE_DB; 2];
            self.started = true;
        }
        let b = ballistics(c);
        let dt = dt_secs.max(0.0);
        let mut level_db = [SILENCE_DB; 2];
        for ch in 0..2 {
            let target = if b.rms {
                let sum = input.sum_sq.get(ch).copied().unwrap_or(0.0);
                let rms = if input.frames > 0 {
                    (sum / input.frames as f64).sqrt() as f32
                } else {
                    0.0
                };
                from_db(to_db(rms) + SINE_RMS_OFFSET_DB)
            } else {
                input.peak.get(ch).copied().unwrap_or(0.0)
            };
            let Some(linear) = self.linear.get_mut(ch) else {
                continue;
            };
            let approach = if b.attack_tau > 0.0 {
                (1.0 - (-dt / b.attack_tau).exp()) as f32
            } else {
                1.0
            };
            if target >= *linear {
                *linear += (target - *linear) * approach;
            } else if b.rms {
                // VU: the same time constant both ways.
                *linear += (target - *linear) * approach;
            } else {
                let fallen = to_db(*linear) - b.release_db_per_sec * dt as f32;
                *linear = from_db(fallen.max(to_db(target)));
            }
            let db = to_db(*linear);
            if let Some(l) = level_db.get_mut(ch) {
                *l = db;
            }
            if let (Some(hold), Some(left)) = (self.hold_db.get_mut(ch), self.hold_left.get_mut(ch))
            {
                if c.peak_hold_secs <= 0.0 {
                    *hold = db;
                } else if db >= *hold {
                    *hold = db;
                    *left = c.peak_hold_secs;
                } else if *left > 0.0 {
                    *left -= dt as f32;
                } else {
                    *hold = (*hold - b.release_db_per_sec * dt as f32).max(db);
                }
            }
        }
        self.loudness.add(&input, dt);
        MeterReading {
            level_db,
            hold_db: self.hold_db,
            momentary_lufs: self.loudness.lufs(MOMENTARY_BINS),
            short_term_lufs: self.loudness.lufs(SHORT_TERM_BINS),
        }
    }
}
