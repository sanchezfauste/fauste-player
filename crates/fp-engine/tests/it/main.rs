//! The integration tests of `fp-engine`, gathered in one binary: every test
//! file is a module here, so the crate's tests link once instead of once
//! per file. Tests that install process-wide state (a panic hook, a global
//! tracing subscriber) keep their own binary next to this directory.

// The allocation audits (`mixer`, `dsd_mixer`, `audio_path_audit`) need it;
// it only acts inside `assert_no_alloc` blocks.
#[cfg(debug_assertions)]
#[global_allocator]
static ALLOCATOR: assert_no_alloc::AllocDisabler = assert_no_alloc::AllocDisabler;

mod audio_path_audit;
mod bit_perfect;
mod bus;
mod cartwall;
mod conductor;
mod decode;
mod device_overrides;
mod dsd_mixer;
mod dsd_output;
mod dsd_real_music;
mod engine;
mod live_settings;
mod metering;
mod mixer;
mod reporting;
mod review_fixes;
mod support;
mod worker;
