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

/// The programme-meter integration time (ms, 0 for none) and fall rate
/// (dB/s) the mixer applies per sample for `c` (meters spec M1): a
/// programme meter integrates each sample, so peaks shorter than the
/// integration time read lower.
pub fn mixer_integration(c: &MeterConfig) -> (f32, f32) {
    let b = ballistics(c);
    let integration = match c.ballistics {
        MeterBallistics::EbuPpm => 10.0,
        MeterBallistics::DinPpm => 5.0,
        MeterBallistics::Custom => c.attack_ms,
        MeterBallistics::DigitalPeak | MeterBallistics::Vu => 0.0,
    };
    (integration, b.release_db_per_sec)
}

fn ballistics(c: &MeterConfig) -> Ballistics {
    // Peak meters rise at once here: programme meters are integrated per
    // sample in the mixer (`mixer_integration`).
    let peak = |fall_db: f32, fall_secs: f32| Ballistics {
        rms: false,
        attack_tau: 0.0,
        release_db_per_sec: fall_db / fall_secs,
    };
    match c.ballistics {
        MeterBallistics::DigitalPeak => peak(20.0, 1.7),
        MeterBallistics::EbuPpm => peak(24.0, 2.8),
        MeterBallistics::DinPpm => peak(20.0, 1.5),
        // 99 % in 300 ms: τ = 0.3 / ln(100), symmetric.
        MeterBallistics::Vu => Ballistics {
            rms: true,
            attack_tau: 0.3 / 100f64.ln(),
            release_db_per_sec: 20.0 / 1.7,
        },
        MeterBallistics::Custom => Ballistics {
            rms: false,
            attack_tau: 0.0,
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
    /// Adds what was measured over `dt`; `frames` also counts silence the
    /// meter infers when nothing played (so the value falls, not freezes).
    /// Time spanning several blocks is spread over them, frames included.
    fn add(&mut self, k_sum: [f64; 2], frames: u64, dt: f64) {
        for (k, add) in self.current.k_sum.iter_mut().zip(k_sum) {
            *k += add;
        }
        // Past a whole window, older time would only be overwritten.
        let dt = dt.min(BIN_SECS * SHORT_TERM_BINS as f64);
        let frames_per_sec = if dt > 0.0 { frames as f64 / dt } else { 0.0 };
        if dt <= 0.0 {
            self.current.frames += frames;
            return;
        }
        let mut left = dt;
        while left > 1e-12 {
            let part = left.min(BIN_SECS - self.current_secs).max(0.0);
            self.current.frames += (frames_per_sec * part).round() as u64;
            self.current_secs += part;
            left -= part;
            if self.current_secs + 1e-9 >= BIN_SECS {
                if let Some(slot) = self.bins.get_mut(self.next) {
                    *slot = self.current;
                }
                self.next = (self.next + 1) % SHORT_TERM_BINS;
                self.filled = (self.filled + 1).min(SHORT_TERM_BINS);
                self.current = Bin::default();
                self.current_secs = 0.0;
            } else if part <= 0.0 {
                break;
            }
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
    /// Seconds since a tick last brought audio.
    idle_secs: f64,
    /// Frames and seconds measured while audio flowed, for the rate.
    flowing_frames: f64,
    flowing_secs: f64,
    /// Time waited for a block, applied to the level when it comes.
    pending_secs: f64,
}

/// Device callbacks can be longer than a tick: a tick without audio this
/// soon after the last one only means the next block has not come yet.
const GAP_SECS: f64 = 0.05;
/// Assumed until audio has flowed long enough to measure the rate.
const DEFAULT_RATE: f64 = 48_000.0;

/// Moves one channel's level for `span` seconds of measured audio.
fn move_level(linear: &mut f32, b: &Ballistics, input: &MeterInput, ch: usize, span: f64) {
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
    let approach = if b.attack_tau > 0.0 {
        (1.0 - (-span / b.attack_tau).exp()) as f32
    } else {
        1.0
    };
    if target >= *linear || b.rms {
        // A VU moves with the same time constant both ways.
        *linear += (target - *linear) * approach;
    } else {
        let fallen = to_db(*linear) - b.release_db_per_sec * span as f32;
        *linear = from_db(fallen.max(to_db(target)));
    }
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
        if input.frames > 0 {
            self.idle_secs = 0.0;
            self.flowing_frames += input.frames as f64;
            self.flowing_secs += dt;
        } else {
            self.idle_secs += dt;
        }
        // A device block can span several ticks: a tick without audio this
        // soon after the last block only means the next one has not come.
        // The level then stands, and moves for the whole span when it does.
        let waiting = input.frames == 0 && self.idle_secs < GAP_SECS;
        let span = if waiting {
            self.pending_secs += dt;
            None
        } else {
            let span = self.pending_secs + dt;
            self.pending_secs = 0.0;
            Some(span)
        };
        let mut level_db = [SILENCE_DB; 2];
        for ch in 0..2 {
            let Some(linear) = self.linear.get_mut(ch) else {
                continue;
            };
            if let Some(span) = span {
                move_level(linear, &b, &input, ch, span);
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
        let frames = if input.frames == 0 && !waiting {
            // Nothing playing: silence, counted as time so the loudness
            // falls instead of freezing at the last value.
            let rate = if self.flowing_secs > 0.5 {
                self.flowing_frames / self.flowing_secs
            } else {
                DEFAULT_RATE
            };
            (dt * rate).round() as u64
        } else {
            input.frames
        };
        self.loudness.add(input.k_sum, frames, dt);
        MeterReading {
            level_db,
            hold_db: self.hold_db,
            momentary_lufs: self.loudness.lufs(MOMENTARY_BINS),
            short_term_lufs: self.loudness.lufs(SHORT_TERM_BINS),
        }
    }
}
