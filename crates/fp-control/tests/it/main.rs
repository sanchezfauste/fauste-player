//! The integration tests of `fp-control`, gathered in one binary: every test
//! file is a module here, so the crate's tests link once instead of once
//! per file. Tests that install process-wide state (a panic hook, a global
//! tracing subscriber) keep their own binary next to this directory.

mod feedback;
mod learn;
mod message;
mod router;
mod service;
mod support;
