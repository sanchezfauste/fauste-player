//! Remote control over the network (remote control spec): an HTTP/JSON API
//! on its own thread. Requests become the same commands the interface
//! sends; the remote thread is never an audio or UI thread.

#![deny(clippy::indexing_slicing)]

pub mod api;
pub mod control;
pub mod dto;
