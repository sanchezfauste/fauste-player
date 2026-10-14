//! Analysis of one file: decode once, derive envelope and markers, read tags.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;

use fp_decode::FileDecoder;
use fp_dsp::loudness::LoudnessMeter;
use fp_model::{
    AnalysisSettings, AudioFormat, LOUDNESS_VERSION, Limits, LoudnessMeasurement, TrackAnalysis,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::metadata::{read_intro, read_tags, thumbnail_png, title_from_file_name};
use crate::signal::{EnvelopeBuilder, WavePeak, detect_markers};

/// Everything analysis produces for one file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Analysis {
    /// Metadata and markers for the model.
    pub analysis: TrackAnalysis,
    /// Waveform: min, max and RMS per bucket, for the UI.
    pub peaks: Vec<WavePeak>,
    pub peak_bucket_secs: f64,
    /// Cover thumbnail as PNG, for the UI.
    pub cover_png: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AnalysisError {
    #[error("the file does not exist")]
    Missing,
    #[error("the file cannot be decoded: {0}")]
    Unreadable(String),
    #[error("the analysis was cancelled")]
    Cancelled,
}

/// Analyses `path` completely (spec §6).
pub fn analyze_file(
    path: &Path,
    settings: &AnalysisSettings,
    limits: &Limits,
) -> Result<Analysis, AnalysisError> {
    analyze_file_cancellable(path, settings, limits, &|| false)
}

/// As `analyze_file`, checking `cancelled` between decoded blocks.
pub fn analyze_file_cancellable(
    path: &Path,
    settings: &AnalysisSettings,
    limits: &Limits,
    cancelled: &dyn Fn() -> bool,
) -> Result<Analysis, AnalysisError> {
    match std::fs::metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Err(AnalysisError::Missing),
        _ => {}
    }
    let mut decoder = FileDecoder::open(path).map_err(AnalysisError::Unreadable)?;
    let mut builder = EnvelopeBuilder::new(
        decoder.sample_rate(),
        settings.rms_window_ms,
        settings.peak_bucket_ms,
    );
    let mut meter = Meter::new(decoder.sample_rate());
    let mut block = Vec::new();
    let mut decoded_any = false;
    loop {
        if cancelled() {
            return Err(AnalysisError::Cancelled);
        }
        block.clear();
        match decoder.next_block(&mut block) {
            Ok(true) => {
                decoded_any = true;
                builder.push(&block);
                meter.push(&block, path);
            }
            Ok(false) => break,
            // A damaged stretch after good audio: keep what decoded (the
            // engine plays up to the same point).
            Err(e) if decoded_any => {
                tracing::warn!(path = %path.display(), "decoding stopped early: {e}");
                break;
            }
            Err(e) => return Err(AnalysisError::Unreadable(e)),
        }
    }
    let envelope = builder.finish();
    let loudness = meter.finish(path);
    let markers = detect_markers(&envelope, settings);
    // Tag and image parsers run on untrusted data: a crash there must only
    // cost the metadata, never the (playable) audio.
    let tags = std::panic::catch_unwind(|| read_tags(path, limits)).unwrap_or_default();
    let (file_artist, file_title) = title_from_file_name(path);
    let cover_png = tags.cover.as_deref().and_then(|bytes| {
        std::panic::catch_unwind(|| thumbnail_png(bytes, limits, settings.cover_thumb_px))
            .ok()
            .flatten()
    });
    Ok(Analysis {
        analysis: TrackAnalysis {
            title: tags.title.or(Some(file_title)),
            artist: tags.artist.or(file_artist),
            album: tags.album,
            duration_secs: envelope.duration_secs,
            cue_in: Some(markers.cue_in),
            cue_out: Some(markers.cue_out),
            segue_start: markers.segue_start,
            outro_start: markers.outro_start,
            intro_end: read_intro(path).map(|secs| {
                secs.max(markers.cue_in)
                    .min(markers.cue_out.max(markers.cue_in))
            }),
            format: Some(AudioFormat {
                sample_rate: decoder.sample_rate(),
                bits: decoder.bits_per_sample(),
                channels: u32::try_from(decoder.channels()).unwrap_or(0),
                dsd_rate: decoder.dsd_rate(),
            }),
            version: crate::cache::ANALYSIS_VERSION,
            loudness,
            loudness_pass: Some(LOUDNESS_VERSION),
        },
        peak_bucket_secs: envelope.bucket_secs,
        peaks: envelope.peaks,
        cover_png,
    })
}

/// Loudness normalization spec N6b: decodes `path` once and measures only
/// its loudness (no envelope, markers, tags or cover). `Ok(None)` when
/// nothing could be measured (shorter than 400 ms, or a failure inside
/// the meter, logged). Like the analysis, a damaged stretch after good
/// audio ends the measurement there (N4).
pub fn measure_loudness_cancellable(
    path: &Path,
    cancelled: &dyn Fn() -> bool,
) -> Result<Option<LoudnessMeasurement>, AnalysisError> {
    match std::fs::metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Err(AnalysisError::Missing),
        _ => {}
    }
    let mut decoder = FileDecoder::open(path).map_err(AnalysisError::Unreadable)?;
    let mut meter = Meter::new(decoder.sample_rate());
    let mut block = Vec::new();
    let mut decoded_any = false;
    loop {
        if cancelled() {
            return Err(AnalysisError::Cancelled);
        }
        block.clear();
        match decoder.next_block(&mut block) {
            Ok(true) => {
                decoded_any = true;
                meter.push(&block, path);
            }
            Ok(false) => break,
            Err(e) if decoded_any => {
                tracing::warn!(path = %path.display(), "decoding stopped early: {e}");
                break;
            }
            Err(e) => return Err(AnalysisError::Unreadable(e)),
        }
    }
    Ok(meter.finish(path))
}

/// The loudness meter of one file, isolated from the rest of the analysis:
/// a panic inside it costs only the measurement (spec N5).
struct Meter {
    meter: Option<LoudnessMeter>,
    rate: u32,
    frames: u64,
}

impl Meter {
    fn new(rate: u32) -> Self {
        Self {
            meter: Some(LoudnessMeter::new(rate)),
            rate,
            frames: 0,
        }
    }

    /// Feeds one decoded block of interleaved stereo (whole frames: the
    /// decoder never splits one).
    fn push(&mut self, stereo: &[f32], path: &Path) {
        self.frames = self
            .frames
            .saturating_add(u64::try_from(stereo.len() / 2).unwrap_or(u64::MAX));
        if let Some(m) = self.meter.as_mut()
            && catch_unwind(AssertUnwindSafe(|| m.push(stereo))).is_err()
        {
            tracing::warn!(path = %path.display(), "the loudness measurement crashed");
            self.meter = None;
        }
    }

    /// The measurement (N5): `None` for a file under one 400 ms block (no
    /// warning: short effects are expected), or, with a warning, after a
    /// crash, for a value that is not finite, or for a true peak no real
    /// programme has (a corrupt file).
    fn finish(self, path: &Path) -> Option<LoudnessMeasurement> {
        let meter = self.meter?;
        let result = catch_unwind(AssertUnwindSafe(|| meter.finish()))
            .ok()
            .flatten();
        let measurement = result.and_then(|l| {
            let m = LoudnessMeasurement {
                integrated_lufs: l.integrated_lufs.map(|v| v as f32),
                true_peak_dbtp: l.true_peak_dbtp as f32,
            };
            let finite =
                m.integrated_lufs.is_none_or(f32::is_finite) && m.true_peak_dbtp.is_finite();
            finite.then_some(m)
        });
        if measurement.is_none() {
            // 400 ms is four tenths of the rate in frames.
            let short = self.frames.saturating_mul(10) < u64::from(self.rate.max(1)) * 4;
            if short {
                tracing::debug!(path = %path.display(), "too short to measure its loudness");
            } else {
                tracing::warn!(path = %path.display(), "the loudness could not be measured");
            }
        }
        measurement
    }
}
