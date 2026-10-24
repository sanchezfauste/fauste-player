//! The integration tests of `fp-decode`, gathered in one binary: every test
//! file is a module here, so the crate's tests link once instead of once
//! per file. Tests that install process-wide state (a panic hook, a global
//! tracing subscriber) keep their own binary next to this directory.

mod ape;
mod dsd;
mod encoded_pcm;
mod formats;
mod gapless;
mod info;
mod matroska;
mod opus;
#[path = "support/opus_files.rs"]
mod opus_files;
mod surround;
mod wavpack;
