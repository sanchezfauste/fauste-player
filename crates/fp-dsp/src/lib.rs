//! Signal processing shared by the audio engine (the level meters, meters
//! spec M1) and the analysis (loudness normalization spec N4): the ITU-R
//! BS.1770 K-weighting, the true-peak interpolator and the gated
//! integrated-loudness meter. Pure computation: no I/O, no threads, no
//! dependency on other workspace crates.

#![deny(clippy::indexing_slicing)]

pub mod kweight;
pub mod truepeak;
