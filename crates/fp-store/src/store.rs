use std::path::Path;
use std::time::{Duration, Instant};

use serde::Serialize;

use fp_model::{AppState, EngineAction, Limits, PlayerId, RestoreParts};

use crate::atomic::{ParseError, load_with_fallback, write_atomic};
use crate::db::snapshot::{self, Snapshot};
use crate::db::{Db, open_with_fallback, reopen_after_unreadable};
use crate::docs::{CONFIG_MIGRATIONS, CONFIG_SCHEMA, ConfigDoc};
use crate::error::StoreError;
use crate::lenient::config_from_value;
use crate::migrate::upgrade;
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
    /// The open database; none until `load` or the first `save_state` opens it.
    db: Option<Db>,
    /// What the database holds, as far as this store knows. None means
    /// unknown: the next save writes everything.
    saved: Option<Snapshot>,
    /// When the last backup of the database was made.
    last_backup: Option<Instant>,
}

pub(crate) fn parse_config(bytes: &[u8]) -> Result<(fp_model::Config, Vec<String>), ParseError> {
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

impl Store {
    pub fn new(paths: AppPaths, limits: Limits) -> Self {
        Self {
            paths,
            limits,
            db: None,
            saved: None,
            last_backup: None,
        }
    }

    pub fn paths(&self) -> &AppPaths {
        &self.paths
    }

    /// The open database (tests).
    #[doc(hidden)]
    pub fn db_for_tests(&mut self) -> Option<&mut Db> {
        self.db.as_mut()
    }

    pub fn load(&mut self, default_playlist_name: &str) -> LoadedState {
        let mut warnings = Vec::new();
        self.db = None;
        self.saved = None;

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

        let snap = match self.open_db(config.limits.backup_count) {
            Ok(opened_warnings) => {
                warnings.extend(opened_warnings);
                self.read_baseline(config.limits.backup_count, &mut warnings)
            }
            Err(e) => {
                warnings.push(format!("database: {e}; starting with the defaults"));
                Snapshot::default()
            }
        };
        // Without player rows the players' ids are not known.
        let players_known = !snap.players.is_empty();
        restore(config, snap, players_known, warnings, default_playlist_name)
    }

    /// Opens the database file (creating `data_dir`) and keeps it. Returns
    /// the warnings of the opening.
    fn open_db(&mut self, backups: usize) -> Result<Vec<String>, StoreError> {
        std::fs::create_dir_all(&self.paths.data_dir)?;
        let opened = open_with_fallback(&self.paths.database_file(), backups)
            .map_err(|e| StoreError::Io(std::io::Error::other(e.to_string())))?;
        self.db = Some(opened.db);
        Ok(opened.warnings)
    }

    /// Reads the snapshot of the open database and makes it the baseline
    /// for the next save. When the read repaired or dropped rows, the
    /// repaired snapshot is written first so disk and baseline agree; if
    /// that fails there is no baseline and the next save writes everything.
    /// When a table could not be read at all, the database is never
    /// written: the file is set aside and the backups are tried instead.
    fn read_baseline(&mut self, backups: usize, warnings: &mut Vec<String>) -> Snapshot {
        let Some(mut db) = self.db.take() else {
            return Snapshot::default();
        };
        let mut read = snapshot::read_checked(&db);
        if !read.complete {
            warnings.append(&mut read.warnings);
            let path = self.paths.database_file();
            drop(db);
            let reopened = reopen_after_unreadable(
                &path,
                backups,
                vec![format!(
                    "{}: a table could not be read; the file is set aside",
                    path.display()
                )],
            );
            match reopened {
                Ok(opened) => {
                    warnings.extend(opened.warnings);
                    db = opened.db;
                    read = snapshot::read_checked(&db);
                    if !read.complete {
                        // Even the replacement cannot be read: keep nothing.
                        warnings.append(&mut read.warnings);
                        warnings.push("database: not used in this session".to_owned());
                        return Snapshot::default();
                    }
                }
                Err(e) => {
                    warnings.push(format!("database: {e}; starting with the defaults"));
                    return Snapshot::default();
                }
            }
        }
        let repaired = !read.warnings.is_empty();
        let snap = read.snapshot;
        warnings.extend(read.warnings);
        self.saved = if repaired {
            match snapshot::write_full(&mut db, &snap) {
                Ok(()) => Some(snap.clone()),
                Err(e) => {
                    tracing::warn!("could not write back the repaired library: {e}");
                    warnings.push(format!(
                        "database: could not write back the repaired library: {e}"
                    ));
                    None
                }
            }
        } else {
            Some(snap.clone())
        };
        self.db = Some(db);
        snap
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

    /// Saves the playlists, library, carts, players and layout to the
    /// database: only the rows that changed since the last save. A backup
    /// follows when none was made yet or `tuning.backup_interval_secs` have
    /// passed since the last one; a failed backup is logged, not an error.
    pub fn save_state(
        &mut self,
        state: &AppState,
        position_of: impl Fn(PlayerId) -> f64,
        now: Instant,
    ) -> Result<(), StoreError> {
        let backups = state.config.limits.backup_count;
        if self.db.is_none() {
            for w in self.open_db(backups)? {
                tracing::warn!("{w}");
            }
            self.saved = None;
        }
        let Some(db) = self.db.as_mut() else {
            return Err(StoreError::Io(std::io::Error::other(
                "the database is not open",
            )));
        };
        let new = Snapshot::of(state, position_of);
        match &self.saved {
            Some(old) => snapshot::write_changes(db, old, &new)?,
            None => snapshot::write_full(db, &new)?,
        }
        self.saved = Some(new);

        let interval = Duration::from_secs(state.config.tuning.backup_interval_secs);
        let due = self
            .last_backup
            .is_none_or(|last| now.saturating_duration_since(last) >= interval);
        if due {
            match db.backup_rotating(backups) {
                Ok(()) => self.last_backup = Some(now),
                Err(e) => tracing::warn!("could not back up the database: {e}"),
            }
        }
        Ok(())
    }
}

/// Builds the state from the saved parts, repairing what a hand edit or an
/// older version left inconsistent; each repair adds a warning.
fn restore(
    config: fp_model::Config,
    snap: Snapshot,
    players_known: bool,
    mut warnings: Vec<String>,
    default_playlist_name: &str,
) -> LoadedState {
    let Snapshot {
        library,
        playlists,
        ids,
        players: sessions,
        cartwall: cartwall_session,
        layout,
        cart_pages,
    } = snap;
    // RL19: the restore repairs the layout; a copy says what it repaired.
    let saved_layout = layout.clone();
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
        // And in the playlists: ids a hand edit repeated get new ones.
        let mut lists = playlists.clone();
        probe_ids.observe(lists.max_raw_id());
        for s in &sessions {
            probe_ids.observe(s.id.0);
        }
        warnings.extend(lists.normalize(&mut probe_ids));
    }
    let (mut state, actions) = AppState::restore(
        RestoreParts {
            config,
            library,
            playlists,
            cart_pages,
            cartwall_session,
            layout,
            ids,
        },
        &sessions,
        default_playlist_name,
    );
    // The restore ran the same repair for the same players.
    let mut probe = saved_layout;
    warnings.extend(
        fp_model::settle_layout(&mut probe, &state.player_ids())
            .into_iter()
            .map(|note| format!("session: {note}")),
    );
    // Operator feedback 4, Q12: a removed player's routes and the
    // settings of a device no route names (kept until now, so a
    // route edit in Settings loses nothing) are forgotten.
    let outputs = &mut state.config.outputs;
    if players_known {
        let players: Vec<PlayerId> = state.players.iter().map(|p| p.id).collect();
        outputs.forget_routes_of_other_players(&players);
    }
    outputs.forget_unrouted_devices();
    LoadedState {
        state,
        actions,
        warnings,
    }
}

fn write_doc<T: Serialize>(path: &Path, doc: &T, backups: usize) -> Result<(), StoreError> {
    let bytes = serde_json::to_vec_pretty(doc).map_err(|e| StoreError::Serialize(e.to_string()))?;
    write_atomic(path, &bytes, backups)?;
    Ok(())
}

/// The log limits of the saved configuration, read before logging starts
/// (the full load reports problems through the log, so it comes after).
/// Read-only and silent: a missing, oversized or unreadable file gives the
/// defaults, and values are clamped like `Config::validate` does.
pub fn peek_log_limits(paths: &AppPaths) -> (u64, usize, u64) {
    let defaults = Limits::default();
    let file = paths.config_file();
    let read = std::fs::metadata(&file)
        .ok()
        .filter(|m| m.len() <= defaults.max_state_file_bytes)
        .and_then(|_| std::fs::read(&file).ok());
    let mut config = read
        .and_then(|bytes| parse_config(&bytes).ok())
        .map(|(config, _)| config)
        .unwrap_or_default();
    let _ = config.validate();
    (
        config.limits.max_log_file_mb,
        config.limits.max_log_files,
        config.limits.log_repeat_summary_secs,
    )
}

#[cfg(test)]
mod peek_tests {
    use super::*;

    #[test]
    fn log_limits_come_from_the_saved_config_clamped_or_default() {
        let dir = tempfile::tempdir().unwrap();
        let paths = AppPaths::under(dir.path());
        assert_eq!(peek_log_limits(&paths), (10, 14, 60), "no file");
        std::fs::create_dir_all(paths.config_file().parent().unwrap()).unwrap();
        std::fs::write(
            paths.config_file(),
            r#"{"schema_version":1,"config":{"limits":{"max_log_file_mb":50,"max_log_files":9999,"log_repeat_summary_secs":5}}}"#,
        )
        .unwrap();
        assert_eq!(peek_log_limits(&paths), (50, 365, 5));
        std::fs::write(paths.config_file(), "not json").unwrap();
        assert_eq!(peek_log_limits(&paths), (10, 14, 60), "corrupt file");
    }
}
