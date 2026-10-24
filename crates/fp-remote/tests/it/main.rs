//! The integration tests of `fp-remote`, gathered in one binary: every test
//! file is a module here, so the crate's tests link once instead of once
//! per file. Tests that install process-wide state (a panic hook, a global
//! tracing subscriber) keep their own binary next to this directory.

mod api;
mod dto;
mod edit;
mod events;
mod guard;
mod http;
mod osc;
mod osc_server;
mod plan_cost;
mod repeat_log;
mod server;
mod sse;
mod support;
mod throttle;
