//! Crash-safe persistence: atomic writes, rotating backups, corrupt-file
//! quarantine and versioned JSON documents.

pub mod atomic;
pub mod error;
pub mod paths;

pub use atomic::{LoadSource, Loaded, ParseError, backup_path, load_with_fallback, write_atomic};
pub use error::StoreError;
pub use paths::AppPaths;
