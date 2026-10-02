//! Track analysis (spec §6): tags and cover art, waveform peaks and the
//! automatic cue markers, cached per file and computed on a background pool.

#![deny(clippy::indexing_slicing)]

pub mod analyze;
pub mod analyzer;
pub mod cache;
pub mod metadata;
pub mod signal;
pub mod tags;

pub use analyze::{Analysis, AnalysisError, analyze_file, analyze_file_cancellable};
pub use signal::WavePeak;
