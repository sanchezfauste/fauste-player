//! Analysis of one file: decode once, derive envelope and markers, read tags.

use std::path::Path;

use fp_decode::FileDecoder;
use fp_model::{AnalysisSettings, Limits, TrackAnalysis};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::metadata::{read_tags, thumbnail_png, title_from_file_name};
use crate::signal::{EnvelopeBuilder, detect_markers};

/// Everything analysis produces for one file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Analysis {
    /// Metadata and markers for the model.
    pub analysis: TrackAnalysis,
    /// Waveform: min/max per bucket, for the UI.
    pub peaks: Vec<(i16, i16)>,
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
}

/// Analyses `path` completely (spec §6).
pub fn analyze_file(
    path: &Path,
    settings: &AnalysisSettings,
    limits: &Limits,
) -> Result<Analysis, AnalysisError> {
    if !path.exists() {
        return Err(AnalysisError::Missing);
    }
    let mut decoder = FileDecoder::open(path).map_err(AnalysisError::Unreadable)?;
    let mut builder = EnvelopeBuilder::new(
        decoder.sample_rate(),
        settings.rms_window_ms,
        settings.peak_bucket_ms,
    );
    let mut block = Vec::new();
    loop {
        block.clear();
        match decoder.next_block(&mut block) {
            Ok(true) => builder.push(&block),
            Ok(false) => break,
            Err(e) => return Err(AnalysisError::Unreadable(e)),
        }
    }
    let envelope = builder.finish();
    let markers = detect_markers(&envelope, settings);
    let tags = read_tags(path);
    let (file_artist, file_title) = title_from_file_name(path);
    let cover_png = tags
        .cover
        .as_deref()
        .and_then(|bytes| thumbnail_png(bytes, limits, settings.cover_thumb_px));
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
        },
        peak_bucket_secs: envelope.bucket_secs,
        peaks: envelope.peaks,
        cover_png,
    })
}
