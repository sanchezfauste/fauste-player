//! Crash-safe persistence: atomic writes, rotating backups, corrupt-file
//! quarantine and versioned JSON documents.

pub mod atomic;
pub mod docs;
pub mod error;
pub mod lenient;
pub mod migrate;
pub mod paths;
pub mod store;

pub use atomic::{LoadSource, Loaded, ParseError, backup_path, load_with_fallback, write_atomic};
pub use docs::{
    CONFIG_SCHEMA, ConfigDoc, PLAYLISTS_SCHEMA, PlaylistsDoc, SESSION_SCHEMA, SessionDoc,
};
pub use error::StoreError;
pub use migrate::{Migration, upgrade};
pub use paths::AppPaths;
pub use store::{LoadedState, Store};
