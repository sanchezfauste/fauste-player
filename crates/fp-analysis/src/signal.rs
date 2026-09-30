//! Pure signal analysis: a mono RMS envelope, waveform peaks and the
//! automatic cue markers derived from them.

use fp_model::AnalysisSettings;
use serde::{Deserialize, Serialize};

/// Level reported for digital silence.
pub const FLOOR_DB: f32 = -120.0;

/// Mono loudness envelope and waveform peaks of one file.
#[derive(Debug, Clone, PartialEq)]
pub struct Envelope {
    pub duration_secs: f64,
    /// Length of one RMS window in seconds (the last window may be shorter).
    pub window_secs: f64,
    /// RMS level of each window in dBFS (floored at `FLOOR_DB`).
    pub rms_db: Vec<f32>,
    /// Length of one peak bucket in seconds.
    pub bucket_secs: f64,
    /// Min, max and RMS level of the mono signal per bucket.
    pub peaks: Vec<WavePeak>,
    /// Largest absolute sample of either channel per bucket, in dBFS: what
    /// trimming looks at, since a mono mix hides one-sided or antiphase
    /// audio.
    pub peak_db: Vec<f32>,
}

/// One waveform bucket of the mono signal, scaled to `i16` (full scale =
/// `i16::MAX`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct WavePeak {
    pub min: i16,
    pub max: i16,
    pub rms: i16,
}

/// Builds an `Envelope` incrementally from interleaved stereo blocks.
#[derive(Debug, Clone)]
pub struct EnvelopeBuilder {
    rate: f64,
    window: usize,
    bucket: usize,
    frames: u64,
    sum_sq: f64,
    in_window: usize,
    rms_db: Vec<f32>,
    min: f32,
    max: f32,
    bucket_sum_sq: f64,
    in_bucket: usize,
    peaks: Vec<WavePeak>,
    abs_max: f32,
    peak_db: Vec<f32>,
}

fn to_db(rms: f64) -> f32 {
    if rms <= 0.0 {
        FLOOR_DB
    } else {
        ((20.0 * rms.log10()) as f32).max(FLOOR_DB)
    }
}

fn to_i16(v: f32) -> i16 {
    (v.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16
}

impl EnvelopeBuilder {
    pub fn new(sample_rate: u32, window_ms: u32, bucket_ms: u32) -> Self {
        let rate = f64::from(sample_rate.max(1));
        let frames_for = |ms: u32| ((rate * f64::from(ms.max(1)) / 1000.0).round() as usize).max(1);
        Self {
            rate,
            window: frames_for(window_ms),
            bucket: frames_for(bucket_ms),
            frames: 0,
            sum_sq: 0.0,
            in_window: 0,
            rms_db: Vec::new(),
            min: 0.0,
            max: 0.0,
            bucket_sum_sq: 0.0,
            in_bucket: 0,
            peaks: Vec::new(),
            abs_max: 0.0,
            peak_db: Vec::new(),
        }
    }

    /// Adds interleaved stereo samples (mono = the average of both channels).
    pub fn push(&mut self, stereo: &[f32]) {
        for frame in stereo.as_chunks::<2>().0 {
            let [l, r] = *frame;
            let m = (l + r) * 0.5;
            self.sum_sq += f64::from(m) * f64::from(m);
            self.in_window += 1;
            if self.in_window == self.window {
                self.close_window();
            }
            self.min = self.min.min(m);
            self.max = self.max.max(m);
            self.abs_max = self.abs_max.max(l.abs()).max(r.abs());
            self.bucket_sum_sq += f64::from(m) * f64::from(m);
            self.in_bucket += 1;
            if self.in_bucket == self.bucket {
                self.close_bucket();
            }
            self.frames += 1;
        }
    }

    fn close_window(&mut self) {
        let rms = (self.sum_sq / self.in_window.max(1) as f64).sqrt();
        self.rms_db.push(to_db(rms));
        self.sum_sq = 0.0;
        self.in_window = 0;
    }

    fn close_bucket(&mut self) {
        let rms = (self.bucket_sum_sq / self.in_bucket.max(1) as f64).sqrt();
        self.peaks.push(WavePeak {
            min: to_i16(self.min),
            max: to_i16(self.max),
            rms: to_i16(rms as f32),
        });
        self.peak_db.push(to_db(f64::from(self.abs_max)));
        self.abs_max = 0.0;
        self.bucket_sum_sq = 0.0;
        self.min = 0.0;
        self.max = 0.0;
        self.in_bucket = 0;
    }

    pub fn finish(mut self) -> Envelope {
        if self.in_window > 0 {
            self.close_window();
        }
        if self.in_bucket > 0 {
            self.close_bucket();
        }
        Envelope {
            duration_secs: self.frames as f64 / self.rate,
            window_secs: self.window as f64 / self.rate,
            rms_db: self.rms_db,
            bucket_secs: self.bucket as f64 / self.rate,
            peaks: self.peaks,
            peak_db: self.peak_db,
        }
    }
}

impl Envelope {
    pub fn from_stereo(samples: &[f32], sample_rate: u32, window_ms: u32, bucket_ms: u32) -> Self {
        let mut b = EnvelopeBuilder::new(sample_rate, window_ms, bucket_ms);
        b.push(samples);
        b.finish()
    }

    fn window_end(&self, i: usize) -> f64 {
        ((i + 1) as f64 * self.window_secs).min(self.duration_secs)
    }

    fn bucket_start(&self, i: usize) -> f64 {
        (i as f64 * self.bucket_secs).min(self.duration_secs)
    }

    fn bucket_end(&self, i: usize) -> f64 {
        ((i + 1) as f64 * self.bucket_secs).min(self.duration_secs)
    }
}

/// The automatic cue points of spec §6 (seconds). `intro_end` is never
/// detected automatically.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AutoMarkers {
    pub cue_in: f64,
    pub cue_out: f64,
    pub segue_start: Option<f64>,
    pub outro_start: Option<f64>,
}

/// Detects the cue points of spec §6 from an envelope.
pub fn detect_markers(env: &Envelope, s: &AnalysisSettings) -> AutoMarkers {
    // Trim on the stereo bucket peaks, so no audible sample is ever cut.
    let threshold = s.trim_threshold_db as f32;
    let first = env.peak_db.iter().position(|db| *db >= threshold);
    let last = env.peak_db.iter().rposition(|db| *db >= threshold);
    let (Some(first), Some(last)) = (first, last) else {
        // Nothing audible: keep the whole file, and no transition markers.
        return AutoMarkers {
            cue_in: 0.0,
            cue_out: env.duration_secs,
            segue_start: None,
            outro_start: None,
        };
    };
    let margin = f64::from(s.trim_margin_ms) / 1000.0;
    let cue_in = (env.bucket_start(first) - margin).max(0.0);
    let cue_out = (env.bucket_end(last) + margin).min(env.duration_secs);
    if env.duration_secs < s.markers_min_duration_secs {
        return AutoMarkers {
            cue_in,
            cue_out,
            segue_start: None,
            outro_start: None,
        };
    }
    let clamp = |t: f64, max_back: f64| t.max(cue_out - max_back).max(cue_in).min(cue_out);

    // The body: the RMS windows that overlap [cue_in, cue_out].
    let windows = env.rms_db.len();
    let window = env.window_secs.max(f64::EPSILON);
    let first_w = ((cue_in / window) as usize).min(windows.saturating_sub(1));
    let last_w = (((cue_out / window).ceil() as usize).saturating_sub(1))
        .clamp(first_w, windows.saturating_sub(1));
    let mut body: Vec<f32> = env
        .rms_db
        .get(first_w..=last_w)
        .map(<[f32]>::to_vec)
        .unwrap_or_default();
    body.sort_by(f32::total_cmp);
    let median = body.get(body.len() / 2).copied().unwrap_or(FLOOR_DB);
    // The last window still at or above `level`, as a time.
    let last_at_or_above = |level: f32| {
        env.rms_db
            .get(..=last_w)
            .and_then(|w| w.iter().rposition(|db| *db >= level))
            .map(|i| env.window_end(i))
    };

    // Segue: where the level has fallen `segue_drop_db` below the body's
    // median, so loud and quiet masters get the same overlap.
    let segue_start =
        last_at_or_above(median - s.segue_drop_db as f32).map(|t| clamp(t, s.segue_max_secs));

    // Outro: where the level falls `outro_drop_db` below the track's median.
    let outro = last_at_or_above(median - s.outro_drop_db as f32).unwrap_or(cue_out);
    // An outro that would start at the very end is no outro.
    let outro_start = Some(clamp(outro, s.outro_max_secs)).filter(|t| *t < cue_out);

    AutoMarkers {
        cue_in,
        cue_out,
        segue_start,
        outro_start,
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {
    use super::*;
    use fp_model::AnalysisSettings;

    const RATE: u32 = 8_000;

    /// Stereo samples of a sine at `amp` for `secs`, with `amp` optionally
    /// shaped by `gain(t)` (t in seconds from the start of this segment).
    fn tone(secs: f64, amp: f32, gain: impl Fn(f64) -> f32) -> Vec<f32> {
        let n = (secs * f64::from(RATE)) as usize;
        (0..n)
            .flat_map(|i| {
                let t = i as f64 / f64::from(RATE);
                let v = amp * gain(t) * ((i as f32) * 0.3).sin();
                [v, v]
            })
            .collect()
    }

    fn silence(secs: f64) -> Vec<f32> {
        vec![0.0; (secs * f64::from(RATE)) as usize * 2]
    }

    fn envelope(samples: &[f32]) -> Envelope {
        Envelope::from_stereo(samples, RATE, 50, 10)
    }

    fn settings() -> AnalysisSettings {
        AnalysisSettings::default()
    }

    /// A 1 kHz-ish tone at a fixed level in dBFS (peak).
    fn tone_db(secs: f64, db: f32) -> Vec<f32> {
        tone(secs, 10f32.powf(db / 20.0), |_| 1.0)
    }

    /// Every bucket whose stereo peak reaches the trim threshold lies in
    /// `[cue_in, cue_out]`.
    fn assert_nothing_audible_is_trimmed(env: &Envelope, m: &AutoMarkers, s: &AnalysisSettings) {
        for (i, db) in env.peak_db.iter().enumerate() {
            if *db >= s.trim_threshold_db as f32 {
                let start = i as f64 * env.bucket_secs;
                let end = ((i + 1) as f64 * env.bucket_secs).min(env.duration_secs);
                assert!(
                    start >= m.cue_in - 1e-9 && end <= m.cue_out + 1e-9,
                    "bucket {i} ({db} dB) at {start}–{end} outside {}–{}",
                    m.cue_in,
                    m.cue_out
                );
            }
        }
    }

    #[test]
    fn trimming_keeps_every_bucket_at_or_above_the_threshold() {
        let click: Vec<f32> = tone_db(0.002, -30.0);
        let fade = tone(3.0, 0.1, |t| 10f32.powf(-30.0 * t as f32 / 3.0 / 20.0));
        let s: Vec<f32> = [
            silence(0.5),
            click,
            silence(1.0),
            tone_db(5.0, -6.0),
            fade,
            silence(1.0),
        ]
        .concat();
        let env = envelope(&s);
        let m = detect_markers(&env, &settings());
        assert_nothing_audible_is_trimmed(&env, &m, &settings());
        assert!(m.cue_in <= 0.5, "the click at 0.5 s stays: {}", m.cue_in);
    }

    #[test]
    fn a_soft_fade_in_is_not_cut() {
        // −70 → −20 dBFS over 4 s: it crosses −60 dBFS at 0.8 s.
        let s = tone(4.0, 1.0, |t| {
            10f32.powf((-70.0 + 50.0 * t as f32 / 4.0) / 20.0)
        });
        let env = envelope(&s);
        let m = detect_markers(&env, &settings());
        assert_nothing_audible_is_trimmed(&env, &m, &settings());
        assert!(m.cue_in <= 0.8, "cue_in {}", m.cue_in);
    }

    #[test]
    fn one_channel_or_antiphase_audio_is_not_trimmed_away() {
        let left_only: Vec<f32> = tone_db(2.0, -20.0)
            .chunks(2)
            .flat_map(|f| [f.first().copied().unwrap_or(0.0), 0.0])
            .collect();
        let antiphase: Vec<f32> = tone_db(2.0, -20.0)
            .chunks(2)
            .flat_map(|f| {
                let v = f.first().copied().unwrap_or(0.0);
                [v, -v]
            })
            .collect();
        let s: Vec<f32> = [
            silence(1.0),
            antiphase,
            silence(1.0),
            left_only,
            silence(1.0),
        ]
        .concat();
        let env = envelope(&s);
        let m = detect_markers(&env, &settings());
        assert!(m.cue_in <= 1.0 && m.cue_out >= 6.0, "{m:?}");
        assert_nothing_audible_is_trimmed(&env, &m, &settings());
    }

    #[test]
    fn the_margin_moves_cue_in_back_and_cue_out_forward() {
        let s: Vec<f32> = [silence(1.0), tone_db(3.0, -6.0), silence(1.0)].concat();
        let env = envelope(&s);
        let tight = AnalysisSettings {
            trim_margin_ms: 0,
            ..settings()
        };
        let wide = AnalysisSettings {
            trim_margin_ms: 200,
            ..settings()
        };
        let a = detect_markers(&env, &tight);
        let b = detect_markers(&env, &wide);
        assert!((a.cue_in - b.cue_in - 0.2).abs() < 1e-6, "{a:?} {b:?}");
        assert!((b.cue_out - a.cue_out - 0.2).abs() < 1e-6, "{a:?} {b:?}");
        // Clamped to the file.
        let huge = AnalysisSettings {
            trim_margin_ms: 1000,
            ..settings()
        };
        let c = detect_markers(&envelope(&tone_db(3.0, -6.0)), &huge);
        assert_eq!((c.cue_in, c.cue_out), (0.0, 3.0));
    }

    /// `secs` of tone whose level falls linearly in dB by `drop` from `db`.
    fn fade_db(secs: f64, db: f32, drop: f32) -> Vec<f32> {
        tone(secs, 1.0, |t| {
            10f32.powf((db - drop * t as f32 / secs as f32) / 20.0)
        })
    }

    fn overlap(m: &AutoMarkers) -> f64 {
        m.cue_out - m.segue_start.unwrap_or(m.cue_out)
    }

    #[test]
    fn the_same_fade_gives_the_same_overlap_on_loud_and_quiet_masters() {
        let loud: Vec<f32> = [tone_db(70.0, -5.0), fade_db(6.0, -5.0, 40.0)].concat();
        let quiet: Vec<f32> = [tone_db(70.0, -17.0), fade_db(6.0, -17.0, 40.0)].concat();
        let a = detect_markers(&envelope(&loud), &settings());
        let b = detect_markers(&envelope(&quiet), &settings());
        assert!((overlap(&a) - overlap(&b)).abs() < 0.1, "{a:?} {b:?}");
        assert!(overlap(&a) > 0.5, "{a:?}");
    }

    #[test]
    fn the_overlap_never_exceeds_segue_max() {
        let s: Vec<f32> = [tone_db(70.0, -6.0), fade_db(20.0, -6.0, 60.0)].concat();
        let m = detect_markers(&envelope(&s), &settings());
        assert!(overlap(&m) <= settings().segue_max_secs + 1e-9, "{m:?}");
        assert_eq!(settings().segue_max_secs, 4.0);
    }

    #[test]
    fn a_track_that_ends_at_full_level_has_a_tiny_overlap() {
        let m = detect_markers(&envelope(&tone_db(90.0, -6.0)), &settings());
        assert!(overlap(&m) <= 0.1, "{m:?}");
    }

    #[test]
    fn silence_at_both_ends_is_trimmed() {
        let s: Vec<f32> = [silence(1.0), tone(60.0, 0.5, |_| 1.0), silence(2.0)].concat();
        let m = detect_markers(&envelope(&s), &settings());
        assert!((m.cue_in - 1.0).abs() < 0.06, "cue_in {}", m.cue_in);
        assert!((m.cue_out - 61.0).abs() < 0.06, "cue_out {}", m.cue_out);
    }

    #[test]
    fn segue_starts_where_the_tail_drops_below_the_body_by_segue_drop() {
        let s: Vec<f32> = [
            tone(60.0, 0.5, |_| 1.0),
            tone(6.0, 0.5, |t| (1.0 - t / 6.0) as f32),
        ]
        .concat();
        let m = detect_markers(&envelope(&s), &settings());
        // The body is at 0.5/√2 RMS; 15 dB below it, (1 − t/6) = 10^(−15/20)
        // at t ≈ 4.93 s into the fade.
        let segue = m.segue_start.unwrap();
        assert!((segue - 64.93).abs() < 0.12, "segue {segue}");
        assert!(segue <= m.cue_out);
    }

    #[test]
    fn segue_is_limited_to_max_seconds_before_cue_out() {
        let s: Vec<f32> = [
            tone(60.0, 0.5, |_| 1.0),
            tone(20.0, 0.5, |t| (1.0 - t / 20.0).powi(4) as f32),
        ]
        .concat();
        let m = detect_markers(&envelope(&s), &settings());
        let segue = m.segue_start.unwrap();
        assert!(
            segue >= m.cue_out - 8.0 - 1e-9,
            "segue {segue} cue_out {}",
            m.cue_out
        );
    }

    #[test]
    fn a_silent_file_keeps_the_whole_duration_and_gets_no_segue() {
        let m = detect_markers(&envelope(&silence(90.0)), &settings());
        assert_eq!((m.cue_in, m.cue_out), (0.0, 90.0));
        assert_eq!((m.segue_start, m.outro_start), (None, None));
    }

    #[test]
    fn a_steady_quiet_file_gets_no_overlap_and_no_empty_outro() {
        // ≈ −25 dBFS RMS throughout: the level never falls, so the next
        // track starts at the very end.
        let m = detect_markers(&envelope(&tone(120.0, 0.08, |_| 1.0)), &settings());
        assert!(m.cue_out > 119.9);
        assert!(
            overlap(&m) <= 0.05,
            "a soft piece must not be talked over by the next track: {m:?}"
        );
        assert_eq!(
            m.outro_start, None,
            "an outro that starts at the very end is no outro"
        );
    }

    #[test]
    fn short_tracks_get_no_segue_or_outro() {
        let m = detect_markers(&envelope(&tone(30.0, 0.5, |_| 1.0)), &settings());
        assert_eq!((m.segue_start, m.outro_start), (None, None));
        assert!(m.cue_out > 29.9);
    }

    #[test]
    fn outro_starts_where_the_level_drops_below_the_median() {
        let s: Vec<f32> = [
            tone(60.0, 0.5, |_| 1.0),
            tone(20.0, 0.1, |_| 1.0),
            silence(1.0),
        ]
        .concat();
        let m = detect_markers(&envelope(&s), &settings());
        let outro = m.outro_start.unwrap();
        assert!((outro - 60.0).abs() < 0.06, "outro {outro}");
    }

    #[test]
    fn peaks_hold_min_and_max_per_bucket() {
        // 10 ms buckets at 8 kHz = 80 frames.
        let mut s = vec![0.0f32; 160 * 2];
        s[2 * 3] = 0.5;
        s[2 * 3 + 1] = 0.5;
        s[2 * 100] = -1.0;
        s[2 * 100 + 1] = -1.0;
        let e = envelope(&s);
        assert_eq!(e.peaks.len(), 2);
        assert_eq!((e.peaks[0].min, e.peaks[0].max), (0, i16::MAX / 2));
        assert_eq!((e.peaks[1].min, e.peaks[1].max), (-i16::MAX, 0));
    }

    #[test]
    fn peaks_hold_the_rms_level_of_each_bucket() {
        // 10 ms buckets at 8 kHz = 80 frames: a constant 0.5, then a single
        // full-scale sample among 79 silent ones.
        let mut s = vec![0.5f32; 80 * 2];
        s.extend(vec![0.0f32; 80 * 2]);
        s[2 * 100] = -1.0;
        s[2 * 100 + 1] = -1.0;
        let e = envelope(&s);
        assert_eq!(e.peaks[0].rms, i16::MAX / 2);
        let expected = f32::from(i16::MAX) / 80f32.sqrt();
        assert!(
            (f32::from(e.peaks[1].rms) - expected).abs() <= 1.0,
            "{}",
            e.peaks[1].rms
        );
    }

    #[test]
    fn streaming_equals_one_shot() {
        let s: Vec<f32> = [tone(3.0, 0.5, |t| (t / 3.0) as f32), silence(0.5)].concat();
        let one = envelope(&s);
        let mut b = EnvelopeBuilder::new(RATE, 50, 10);
        for chunk in s.chunks(2 * 333) {
            b.push(chunk);
        }
        assert_eq!(b.finish(), one);
    }
}
