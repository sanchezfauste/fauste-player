//! Crash-safe persistence: atomic writes, rotating backups, corrupt-file
//! quarantine and versioned JSON documents.

pub mod atomic;
pub mod db;
pub mod docs;
pub mod error;
mod import;
mod layout_notes;
mod legacy;
pub mod lenient;
pub mod migrate;
pub mod paths;
pub mod playlist_io;
pub mod store;

pub use atomic::{LoadSource, Loaded, ParseError, backup_path, load_with_fallback, write_atomic};
pub use db::snapshot::Snapshot;
pub use db::{Db, DbSource, OpenError, Opened, SCHEMA_VERSION, open_with_fallback};
pub use docs::{
    CONFIG_SCHEMA, ConfigDoc, PLAYLISTS_SCHEMA, PlaylistsDoc, SESSION_SCHEMA, SessionDoc,
};
pub use error::StoreError;
pub use legacy::fuzz_documents;
pub use migrate::{Migration, upgrade};
pub use paths::AppPaths;
pub use store::{LoadedState, Store, peek_log_limits};
