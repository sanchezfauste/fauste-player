//! Track analysis (spec §6): tags and cover art, waveform peaks and the
//! automatic cue markers, cached per file and computed on a background pool.

#![deny(clippy::indexing_slicing)]

pub mod metadata;
pub mod signal;
