//! Remote control over the network (remote control spec): an HTTP/JSON API
//! on its own thread. Requests become the same commands the interface
//! sends; the remote thread is never an audio or UI thread.

#![deny(clippy::indexing_slicing)]

pub mod api;
pub mod control;
pub mod dto;
pub mod events;
pub mod http;
pub mod osc;
mod osc_server;
pub mod repeat_log;
pub mod server;
pub mod throttle;

pub use server::{
    RemoteHandle, RemoteStatus, ServerError, ServerStatus, Timing, spawn, spawn_with,
};
