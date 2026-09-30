//! Control surfaces (feedback spec §6): MIDI messages from any connected
//! port become the same commands the interface sends, bound per device and
//! control, with soft takeover for faders, LED feedback and MIDI learn.
//! MIDI threads are never audio threads.

#![deny(clippy::indexing_slicing)]

pub mod message;
