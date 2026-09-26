use std::path::Path;

use serde::Serialize;
use serde::de::DeserializeOwned;

use fp_model::{AppState, EngineAction, Limits, PlayerId, RestoreParts};

use crate::atomic::{Loaded, ParseError, load_with_fallback, write_atomic};
use crate::docs::{
    CARTS_MIGRATIONS, CARTS_SCHEMA, CONFIG_MIGRATIONS, CONFIG_SCHEMA, CartsDoc, ConfigDoc,
    PLAYLISTS_MIGRATIONS, PLAYLISTS_SCHEMA, PlaylistsDoc, SESSION_MIGRATIONS, SESSION_SCHEMA,
    SessionDoc,
};
use crate::error::StoreError;
use crate::lenient::config_from_value;
use crate::migrate::{Migration, upgrade};
use crate::paths::AppPaths;

/// Result of loading everything at startup. Loading never fails: problems
/// become `warnings` and defaults are used instead.
#[derive(Debug)]
pub struct LoadedState {
    pub state: AppState,
    pub actions: Vec<EngineAction>,
    pub warnings: Vec<String>,
}

pub struct Store {
    paths: AppPaths,
    /// Limits used to read `config.json` itself; the loaded config's limits apply afterwards.
    limits: Limits,
}

fn parse_doc<T: DeserializeOwned>(
    bytes: &[u8],
    schema: u32,
    migrations: &[Migration],
) -> Result<T, ParseError> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| ParseError::Corrupt(e.to_string()))?;
    let value = upgrade(value, schema, migrations)?;
    serde_json::from_value(value).map_err(|e| ParseError::Corrupt(e.to_string()))
}

fn parse_config(bytes: &[u8]) -> Result<(fp_model::Config, Vec<String>), ParseError> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| ParseError::Corrupt(e.to_string()))?;
    let value = upgrade(value, CONFIG_SCHEMA, CONFIG_MIGRATIONS)?;
    let mut notes = Vec::new();
    let config = config_from_value(
        value.get("config").unwrap_or(&serde_json::Value::Null),
        &mut notes,
    );
    Ok((config, notes))
}

fn load_doc<T: DeserializeOwned>(
    path: &Path,
    backups: usize,
    max_bytes: u64,
    schema: u32,
    migrations: &[Migration],
) -> Loaded<T> {
    load_with_fallback(path, backups, max_bytes, |bytes| {
        parse_doc(bytes, schema, migrations)
    })
}

/// Runs `bytes` through every document parser and the restore, as loading
/// does. For the fuzz targets only.
#[doc(hidden)]
pub fn fuzz_documents(bytes: &[u8]) {
    let mut config = parse_config(bytes).map(|(c, _)| c).unwrap_or_default();
    let _ = config.validate();
    let lists: Option<PlaylistsDoc> = parse_doc(bytes, PLAYLISTS_SCHEMA, PLAYLISTS_MIGRATIONS).ok();
    let session: Option<SessionDoc> = parse_doc(bytes, SESSION_SCHEMA, SESSION_MIGRATIONS).ok();
    let carts: Option<CartsDoc> = parse_doc(bytes, CARTS_SCHEMA, CARTS_MIGRATIONS).ok();
    let (library, playlists, ids) = lists
        .map(|d| (d.library, d.playlists, d.ids))
        .unwrap_or_default();
    let (sessions, cartwall_session) = session.map(|d| (d.players, d.cartwall)).unwrap_or_default();
    let _ = AppState::restore(
        RestoreParts {
            config,
            library,
            playlists,
            cart_pages: carts.map(|d| d.pages).unwrap_or_default(),
            cartwall_session,
            ids,
        },
        &sessions,
        "Main",
    );
}

impl Store {
    pub fn new(paths: AppPaths, limits: Limits) -> Self {
        Self { paths, limits }
    }

    pub fn paths(&self) -> &AppPaths {
        &self.paths
    }

    pub fn load(&self, default_playlist_name: &str) -> LoadedState {
        let mut warnings = Vec::new();

        // Read leniently: a hand-edited value of the wrong type falls back to its
        // default (with a warning) instead of discarding the whole file.
        let config_doc = load_with_fallback(
            &self.paths.config_file(),
            self.limits.backup_count,
            self.limits.max_state_file_bytes,
            parse_config,
        );
        warnings.extend(config_doc.warnings);
        let mut config = match config_doc.value {
            Some((config, notes)) => {
                warnings.extend(notes);
                config
            }
            None => Default::default(),
        };
        warnings.extend(config.validate().into_iter().map(|w| w.to_string()));
        let limits = config.limits.clone();

        let lists: Loaded<PlaylistsDoc> = load_doc(
            &self.paths.playlists_file(),
            limits.backup_count,
            limits.max_state_file_bytes,
            PLAYLISTS_SCHEMA,
            PLAYLISTS_MIGRATIONS,
        );
        warnings.extend(lists.warnings);
        let session: Loaded<SessionDoc> = load_doc(
            &self.paths.session_file(),
            limits.backup_count,
            limits.max_state_file_bytes,
            SESSION_SCHEMA,
            SESSION_MIGRATIONS,
        );
        warnings.extend(session.warnings);
        let carts: Loaded<CartsDoc> = load_doc(
            &self.paths.carts_file(),
            limits.backup_count,
            limits.max_state_file_bytes,
            CARTS_SCHEMA,
            CARTS_MIGRATIONS,
        );
        warnings.extend(carts.warnings);

        let (library, playlists, ids) = lists
            .value
            .map(|d| (d.library, d.playlists, d.ids))
            .unwrap_or_default();
        let (sessions, cartwall_session) = session
            .value
            .map(|d| (d.players, d.cartwall))
            .unwrap_or_default();
        let cart_pages = carts.value.map(|d| d.pages).unwrap_or_default();
        // Report what the restore will repair in the cart pages (the restore
        // itself applies the same normalisation).
        {
            let mut probe = fp_model::Cartwall {
                pages: cart_pages.clone(),
                ..fp_model::Cartwall::default()
            };
            let mut probe_ids = ids.clone();
            probe_ids.observe(probe.max_raw_id());
            warnings.extend(probe.normalize(&config.limits, &mut probe_ids, &library));
        }
        let (state, actions) = AppState::restore(
            RestoreParts {
                config,
                library,
                playlists,
                cart_pages,
                cartwall_session,
                ids,
            },
            &sessions,
            default_playlist_name,
        );
        LoadedState {
            state,
            actions,
            warnings,
        }
    }

    pub fn save_config(&self, state: &AppState) -> Result<(), StoreError> {
        let doc = ConfigDoc {
            schema_version: CONFIG_SCHEMA,
            config: state.config.clone(),
        };
        write_doc(
            &self.paths.config_file(),
            &doc,
            state.config.limits.backup_count,
        )
    }

    pub fn save_playlists(&self, state: &AppState) -> Result<(), StoreError> {
        let doc = PlaylistsDoc {
            schema_version: PLAYLISTS_SCHEMA,
            library: state.library.clone(),
            playlists: state.playlists.clone(),
            ids: state.ids.clone(),
        };
        write_doc(
            &self.paths.playlists_file(),
            &doc,
            state.config.limits.backup_count,
        )
    }

    pub fn save_carts(&self, state: &AppState) -> Result<(), StoreError> {
        let doc = CartsDoc {
            schema_version: CARTS_SCHEMA,
            pages: state.cartwall.pages.clone(),
        };
        write_doc(
            &self.paths.carts_file(),
            &doc,
            state.config.limits.backup_count,
        )
    }

    pub fn save_session(
        &self,
        state: &AppState,
        position_of: impl Fn(PlayerId) -> f64,
    ) -> Result<(), StoreError> {
        let doc = SessionDoc {
            schema_version: SESSION_SCHEMA,
            players: state.sessions(position_of),
            cartwall: state.cartwall.session(),
        };
        write_doc(
            &self.paths.session_file(),
            &doc,
            state.config.limits.backup_count,
        )
    }
}

fn write_doc<T: Serialize>(path: &Path, doc: &T, backups: usize) -> Result<(), StoreError> {
    let bytes = serde_json::to_vec_pretty(doc).map_err(|e| StoreError::Serialize(e.to_string()))?;
    write_atomic(path, &bytes, backups)?;
    Ok(())
}
