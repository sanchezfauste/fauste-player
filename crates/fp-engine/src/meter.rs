//! Level meters (meters spec M2): standard ballistics and EBU R128 loudness,
//! run by the conductor on what the mixer measured. Pure: time comes in as
//! `dt_secs`, settings as `&MeterConfig`.

use fp_model::{MeterBallistics, MeterConfig};

/// The lowest level a meter reports, in dBFS (below any display floor).
pub const SILENCE_DB: f32 = -120.0;
/// Loudness blocks: 5 ms, the conductor's tick, so the 400 ms and 3 s
/// windows slide finely enough to meet EBU Tech 3341's live-meter cases
/// (tones offset by 20 ms steps).
const BIN_SECS: f64 = 0.005;
const MOMENTARY_BINS: usize = 80;
const SHORT_TERM_BINS: usize = 600;
/// The VU responds to the rectified average; a sine's is 2/π of its peak,
/// and the scale is calibrated so that a sine reads its peak level (AES17).
const VU_SINE_CALIBRATION: f64 = std::f64::consts::FRAC_PI_2;
/// VU needle (IEC 60268-17): a second-order movement that overshoots by
/// 1.25 % (between the standard's 1 % and 1.5 %) and first reaches 99 % of
/// a step in 300 ms: ζ from the overshoot, ω₀ = 4.0536 / 0.3 s.
const VU_DAMPING: f64 = 0.812_717;
const VU_NATURAL_RAD_PER_SEC: f64 = 13.511_93;
/// Integration step of the needle.
const VU_STEP_SECS: f64 = 0.000_25;

/// What a player's sources measured since the last tick (mixer, meters spec M1).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct MeterInput {
    /// Sample or true peak, linear.
    pub peak: [f32; 2],
    pub sum_sq: [f64; 2],
    /// Sums of magnitudes (rectified), for the VU.
    pub sum_abs: [f64; 2],
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
            if let (Some(s), Some(o)) = (self.sum_abs.get_mut(ch), other.sum_abs.get(ch)) {
                *s += o;
            }
            if let (Some(k), Some(o)) = (self.k_sum.get_mut(ch), other.k_sum.get(ch)) {
                *k += o;
            }
        }
        // Both sources play over the same time: the frames are the same span.
        self.frames = self.frames.max(other.frames);
    }

    /// Appends a later measurement of the same source: the spans follow
    /// each other, so their frames add up.
    pub fn extend(&mut self, later: MeterInput) {
        let frames = self.frames.saturating_add(later.frames);
        self.merge(later);
        self.frames = frames;
    }

    /// The measurement with every non-finite value (a NaN or infinite
    /// sample in a file) read as silence, so it cannot stick in a meter.
    fn finite(mut self) -> Self {
        for v in &mut self.peak {
            if !v.is_finite() {
                *v = 0.0;
            }
        }
        for v in self
            .sum_sq
            .iter_mut()
            .chain(&mut self.sum_abs)
            .chain(&mut self.k_sum)
        {
            if !v.is_finite() {
                *v = 0.0;
            }
        }
        self
    }
}

/// What the meter shows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeterReading {
    pub level_db: [f32; 2],
    pub hold_db: [f32; 2],
    /// Average (RMS) level of the K-System, AES17: a sine reads its peak
    /// level.
    pub rms_db: [f32; 2],
    /// The highest level the bar reached since the last restart.
    pub max_db: f32,
    pub momentary_lufs: Option<f32>,
    pub short_term_lufs: Option<f32>,
}

impl Default for MeterReading {
    fn default() -> Self {
        Self {
            level_db: [SILENCE_DB; 2],
            hold_db: [SILENCE_DB; 2],
            rms_db: [SILENCE_DB; 2],
            max_db: SILENCE_DB,
            momentary_lufs: None,
            short_term_lufs: None,
        }
    }
}

/// How a preset moves: the input it reads, its rise time constant and its
/// fall rate.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Ballistics {
    /// A VU (the needle of `move_vu`) rather than a peak meter.
    vu: bool,
    attack_tau: f64,
    release_db_per_sec: f32,
}

/// EBU PPM integrator (two stages, ms), fitted so that 5 kHz bursts meet
/// EBU Tech 3205-E table 2 (100, 10, 5, 1.5 and 0.5 ms) within tolerance.
/// The fit's widest error is 86 % of a tolerance (the 10 ms point).
const EBU_PPM_TAUS: (f32, f32) = (2.25, 0.95);
/// The same integrator reads a burst 2 dB low (the IEC definition of the
/// integration time) at this duration, within Tech 3205's 10 ± 2 ms.
const EBU_PPM_INTEGRATION_MS: f32 = 8.365;

/// The integrator scaled to an integration time of `ms` (IEC definition: a
/// 5 kHz burst that long reads 2 dB below the steady tone).
fn taus_for_integration(ms: f32) -> (f32, f32) {
    let scale = ms / EBU_PPM_INTEGRATION_MS;
    (EBU_PPM_TAUS.0 * scale, EBU_PPM_TAUS.1 * scale)
}

/// The programme-meter integrator the mixer runs per sample for `c` (its
/// two time constants in ms, 0 for none) and the fall rate in dB/s
/// (meters spec M1): a programme meter integrates every sample, so peaks
/// shorter than its integration time read lower.
pub fn mixer_integration(c: &MeterConfig) -> (f32, f32, f32) {
    let b = ballistics(c);
    let (tau1, tau2) = match c.ballistics {
        MeterBallistics::EbuPpm => EBU_PPM_TAUS,
        // IEC 60268-10 type I: 5 ms integration time.
        MeterBallistics::DinPpm => taus_for_integration(5.0),
        MeterBallistics::Custom if c.attack_ms > 0.0 => taus_for_integration(c.attack_ms),
        _ => (0.0, 0.0),
    };
    (tau1, tau2, b.release_db_per_sec)
}

fn ballistics(c: &MeterConfig) -> Ballistics {
    // Peak meters rise at once here: programme meters are integrated per
    // sample in the mixer (`mixer_integration`).
    let peak = |fall_db: f32, fall_secs: f32| Ballistics {
        vu: false,
        attack_tau: 0.0,
        release_db_per_sec: fall_db / fall_secs,
    };
    match c.ballistics {
        MeterBallistics::DigitalPeak => peak(20.0, 1.7),
        MeterBallistics::EbuPpm => peak(24.0, 2.8),
        MeterBallistics::DinPpm => peak(20.0, 1.5),
        // The needle's movement is `move_vu`; the fall is for the hold.
        MeterBallistics::Vu => Ballistics {
            vu: true,
            attack_tau: 0.0,
            release_db_per_sec: 20.0 / 1.7,
        },
        // K-System: one-sample rise, 26 dB in about 3 s.
        MeterBallistics::K20 | MeterBallistics::K14 | MeterBallistics::K12 => peak(26.0, 3.0),
        MeterBallistics::Custom => Ballistics {
            vu: false,
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

/// The last 3 s of blocks, in a ring allocated once (on the conductor
/// thread, never the audio one).
#[derive(Debug, Clone)]
struct LoudnessWindow {
    bins: Vec<Bin>,
    next: usize,
    filled: usize,
    current: Bin,
    current_secs: f64,
}

impl Default for LoudnessWindow {
    fn default() -> Self {
        Self {
            bins: vec![Bin::default(); SHORT_TERM_BINS],
            next: 0,
            filled: 0,
            current: Bin::default(),
            current_secs: 0.0,
        }
    }
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
#[derive(Debug, Clone, Default)]
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
    /// VU needles: position (linear) and velocity, per channel.
    vu: [(f64, f64); 2],
    /// The ballistics the state belongs to; a change starts from rest.
    ballistics: Option<MeterBallistics>,
    /// The two stages of the average's mean square, per channel.
    mean_square: [(f64, f64); 2],
    max_db: f32,
}

/// Device callbacks can be longer than a tick: a tick without audio this
/// soon after the last one only means the next block has not come yet.
const GAP_SECS: f64 = 0.05;
/// Assumed until audio has flowed long enough to measure the rate.
const DEFAULT_RATE: f64 = 48_000.0;

/// Moves a VU needle (`position`, `velocity`) for `span` seconds towards
/// `target`, the second-order movement of IEC 60268-17.
fn move_vu(position: &mut f64, velocity: &mut f64, target: f64, span: f64) {
    // The needle settles well within a second: a longer span (a conductor
    // stall, a resumed machine) needs no more work than that.
    let span = span.min(1.0);
    let steps = (span / VU_STEP_SECS).ceil().max(1.0);
    let h = span / steps;
    let w = VU_NATURAL_RAD_PER_SEC;
    for _ in 0..steps as usize {
        let accel = w * w * (target - *position) - 2.0 * VU_DAMPING * w * *velocity;
        *velocity += accel * h;
        *position += *velocity * h;
    }
}

/// Time constant of each of the K-System average's two stages: a step
/// reads 99 % of its RMS value (98.01 % of its mean square) in 600 ms, the
/// K-System's integration time, and falls the same way.
const K_AVERAGE_TAU_SECS: f64 = 0.6 / 5.839_793;

/// Moves the two stages of an average (mean squares) towards `target` for
/// `span` seconds, solved exactly for a constant input.
fn move_average((a, b): &mut (f64, f64), target: f64, span: f64) {
    let r = span / K_AVERAGE_TAU_SECS;
    let e = (-r).exp();
    let (da, db) = (*a - target, *b - target);
    *a = target + da * e;
    *b = target + db * e + da * r * e;
    // Silence would end in subnormal numbers: flush them.
    for stage in [&mut *a, &mut *b] {
        if *stage < 1e-20 {
            *stage = 0.0;
        }
    }
}

/// Moves one peak-meter channel's level for `span` seconds of audio.
fn move_level(linear: &mut f32, b: &Ballistics, input: &MeterInput, ch: usize, span: f64) {
    let target = input.peak.get(ch).copied().unwrap_or(0.0);
    let approach = if b.attack_tau > 0.0 {
        (1.0 - (-span / b.attack_tau).exp()) as f32
    } else {
        1.0
    };
    if target >= *linear {
        *linear += (target - *linear) * approach;
    } else {
        let fallen = to_db(*linear) - b.release_db_per_sec * span as f32;
        *linear = from_db(fallen.max(to_db(target)));
    }
}

impl MeterState {
    /// Restarts the maximum (a new entry, or the operator's click).
    pub fn reset_max(&mut self) {
        self.max_db = SILENCE_DB;
    }

    /// Advances the meter by `dt_secs` with what was measured meanwhile.
    pub fn update(&mut self, input: MeterInput, dt_secs: f64, c: &MeterConfig) -> MeterReading {
        let input = input.finite();
        if !self.started {
            self.hold_db = [SILENCE_DB; 2];
            self.max_db = SILENCE_DB;
            self.started = true;
        }
        if self.ballistics != Some(c.ballistics) {
            self.ballistics = Some(c.ballistics);
            self.linear = [0.0; 2];
            self.vu = [(0.0, 0.0); 2];
            self.pending_secs = 0.0;
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
            // What this tick measured, on the meter's own terms: the VU
            // reads the calibrated rectified average, the others the peak.
            let measured = if b.vu {
                let sum = input.sum_abs.get(ch).copied().unwrap_or(0.0);
                if input.frames > 0 {
                    sum / input.frames as f64 * VU_SINE_CALIBRATION
                } else {
                    0.0
                }
            } else {
                f64::from(input.peak.get(ch).copied().unwrap_or(0.0))
            };
            if let Some(span) = span {
                if b.vu {
                    let Some((position, velocity)) = self.vu.get_mut(ch) else {
                        continue;
                    };
                    move_vu(position, velocity, measured, span);
                    *linear = position.max(0.0) as f32;
                } else {
                    move_level(linear, &b, &input, ch, span);
                }
            }
            let db = to_db(*linear);
            if let Some(l) = level_db.get_mut(ch) {
                *l = db;
            }
            // Only what this tick measured counts: after a restart the bar
            // may still be falling from the previous entry.
            if input.frames > 0 {
                self.max_db = self.max_db.max(db.min(to_db(measured as f32)));
            }
            if let (Some(span), Some(ms), Some(sum)) =
                (span, self.mean_square.get_mut(ch), input.sum_sq.get(ch))
            {
                let block = if input.frames > 0 {
                    sum / input.frames as f64
                } else {
                    0.0
                };
                move_average(ms, block, span);
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
        // AES17: √2 × RMS, so a full-scale sine reads 0 dBFS.
        let rms_db = self
            .mean_square
            .map(|(_, ms)| to_db((2.0 * ms).sqrt() as f32));
        MeterReading {
            level_db,
            hold_db: self.hold_db,
            rms_db,
            max_db: self.max_db,
            momentary_lufs: self.loudness.lufs(MOMENTARY_BINS),
            short_term_lufs: self.loudness.lufs(SHORT_TERM_BINS),
        }
    }
}
