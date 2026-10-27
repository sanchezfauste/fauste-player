//! The integration tests of `fp-backends`, gathered in one binary: every test
//! file is a module here, so the crate's tests link once instead of once
//! per file. Tests that install process-wide state (a panic hook, a global
//! tracing subscriber) keep their own binary next to this directory.

mod aaudio_ids;
mod audio_path_audit;
mod backends;
mod conformance;
mod cpal;
mod dsd;
mod exclusive;
mod hosts;
