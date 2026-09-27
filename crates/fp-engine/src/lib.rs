//! The audio engine (spec §4): decoded `Source`s feed per-device bus `Mixer`s
//! on real-time threads; per-player worker threads decode and resample; the
//! `Conductor` turns model `EngineAction`s into sample-accurate bus commands.

#![deny(clippy::indexing_slicing)]

pub mod atomic;
pub mod bus;
pub mod conductor;
pub mod decode;
pub mod engine;
pub mod kweight;
pub mod meter;
pub mod mixer;
pub mod ramp;
pub mod resample;
pub mod source;
pub mod truepeak;
pub mod worker;
