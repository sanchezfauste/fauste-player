//! The integration tests of `fp-analysis`, gathered in one binary: every test
//! file is a module here, so the crate's tests link once instead of once
//! per file. Tests that install process-wide state (a panic hook, a global
//! tracing subscriber) keep their own binary next to this directory.

mod analyze;
mod analyzer;
#[path = "support/corpus.rs"]
mod corpus;
mod intro_tag;
mod long_files;
mod loudness_pass;
mod metadata;
mod real_music;
mod tag_cover;
mod tag_sheet;
mod tags;
