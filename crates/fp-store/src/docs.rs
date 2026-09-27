//! On-disk document shapes. Bump a `*_SCHEMA` constant and append a
//! migration whenever a document's shape changes.

use serde::{Deserialize, Serialize};

use fp_model::{CartPage, CartwallSession, Config, IdGen, Library, PlayerSession, Playlists};

use crate::migrate::Migration;

pub const CONFIG_SCHEMA: u32 = 1;
pub const PLAYLISTS_SCHEMA: u32 = 2;
pub const SESSION_SCHEMA: u32 = 1;
pub const CARTS_SCHEMA: u32 = 1;

pub const CONFIG_MIGRATIONS: &[Migration] = &[];
pub const PLAYLISTS_MIGRATIONS: &[Migration] = &[playlists_v1_to_v2];
pub const SESSION_MIGRATIONS: &[Migration] = &[];
pub const CARTS_MIGRATIONS: &[Migration] = &[];

/// Schema 2 marks an entry played per player (`played_by`) instead of with
/// one `played` flag. Player ids live in the session, so the flag is kept as
/// it is and `AppState::restore` counts it as played by every player. The
/// bump itself matters: builds that require `played` reject a schema 2 file
/// as too new and keep it, instead of treating it as corrupt.
fn playlists_v1_to_v2(doc: serde_json::Value) -> Result<serde_json::Value, String> {
    Ok(doc)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigDoc {
    pub schema_version: u32,
    #[serde(default)]
    pub config: Config,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistsDoc {
    pub schema_version: u32,
    #[serde(default)]
    pub library: Library,
    #[serde(default)]
    pub playlists: Playlists,
    #[serde(default)]
    pub ids: IdGen,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDoc {
    pub schema_version: u32,
    #[serde(default)]
    pub players: Vec<PlayerSession>,
    #[serde(default)]
    pub cartwall: CartwallSession,
}

/// `carts.json`: the cartwall pages. Their files are tracks of the shared
/// library in `playlists.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartsDoc {
    pub schema_version: u32,
    #[serde(default)]
    pub pages: Vec<CartPage>,
}
