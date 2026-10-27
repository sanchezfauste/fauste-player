//! Fauste Player on Android (mobile spec §3–§5). The pure parts build and
//! test on every target; the activity glue and the JNI calls exist only on
//! Android, the one place `unsafe` is allowed (spec §8).

pub mod ime;
pub mod insets;
pub mod shell_state;

#[cfg(target_os = "android")]
mod android;
#[cfg(target_os = "android")]
mod jni;
#[cfg(target_os = "android")]
pub mod platform;
