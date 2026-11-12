//! The integration tests of `fp-store`, gathered in one binary: every test
//! file is a module here, so the crate's tests link once instead of once
//! per file. Tests that install process-wide state (a panic hook, a global
//! tracing subscriber) keep their own binary next to this directory.

mod as_run_store;
mod automation_long_run;
mod automation_store;
mod carts;
mod categories_store;
mod db_fallback;
mod db_schema;
mod db_snapshot;
mod import;
mod library_view_store;
mod playlist_io;
mod schedule_store;
mod session_layout;
mod store_roundtrip;
mod support;
