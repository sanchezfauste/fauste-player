//! Fauste Player desktop application.

/// Reverse-DNS application ID, shared by the window, the desktop entry,
/// AppStream, the Flatpak, the snap and the macOS bundle.
pub const APP_ID: &str = "com.sanchezfauste.FaustePlayer";

pub mod activity;
pub mod bootstrap;
pub mod cli;
pub mod crash;
pub mod header;
pub mod i18n;
pub mod instance;
pub mod logging;
pub mod midi;
pub mod remote;
pub mod services;
pub mod session;
pub mod tags;
pub mod ui;
pub mod window_system;
