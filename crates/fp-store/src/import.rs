//! The one-time import of the JSON documents of earlier versions
//! (`playlists.json`, `carts.json`, `session.json`) into the database.
//!
//! The import never overwrites anything and can be interrupted at any
//! point:
//!
//! 1. the documents are read with the legacy parsers (their `.bakN`
//!    fallback and warnings included);
//! 2. the snapshot is written to `fauste.db.importing`, a database of its
//!    own. A leftover of an earlier attempt is deleted first, so a retry
//!    starts from scratch and never duplicates rows;
//! 3. the marker `fauste.db.import-pending` is created, then the new
//!    database is renamed to `fauste.db`: from here on the import is
//!    committed;
//! 4. the JSON files and their backups are renamed to `*.imported` and the
//!    marker is removed.
//!
//! A start that finds the marker next to `fauste.db` knows the commit
//! happened and only finishes step 4. Without the marker, an existing
//! database (or database backup) always wins: JSON files that turn up
//! later are never imported.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use fp_model::{CartPage, Limits, PlayerSession};

use crate::atomic::{Loaded, backup_path, load_with_fallback, sibling};
use crate::db::Db;
use crate::db::snapshot::{self, Snapshot};
use crate::docs::{PLAYLISTS_MIGRATIONS, PLAYLISTS_SCHEMA, PlaylistsDoc};
use crate::error::StoreError;
use crate::legacy::{load_doc, parse_carts, parse_session};
use crate::paths::AppPaths;

/// What the import did.
#[derive(Debug)]
pub(crate) enum Imported {
    /// Nothing to import (or the renames of an earlier import were
    /// finished): load the database as usual.
    Nothing,
    /// The documents are in `fauste.db`: load it as usual.
    Committed,
    /// The documents were read but could not be written to the database.
    /// The session runs on this snapshot without touching the disk, so the
    /// next start imports again.
    Failed(Box<Snapshot>),
}

fn importing_file(paths: &AppPaths) -> PathBuf {
    sibling(&paths.database_file(), "importing")
}

fn marker_file(paths: &AppPaths) -> PathBuf {
    sibling(&paths.database_file(), "import-pending")
}

/// The three documents and their backups, in the order they are renamed.
fn json_files(paths: &AppPaths, backups: usize) -> Vec<PathBuf> {
    [
        paths.playlists_file(),
        paths.carts_file(),
        paths.session_file(),
    ]
    .into_iter()
    .flat_map(|main| {
        std::iter::once(main.clone()).chain((1..=backups).map(move |n| backup_path(&main, n)))
    })
    .collect()
}

fn imported_name(file: &Path) -> PathBuf {
    sibling(file, "imported")
}

/// Renames every JSON file that exists to `<name>.imported`, going on past
/// a failure. Returns the first error.
fn rename_json_files(paths: &AppPaths, backups: usize, fail: bool) -> io::Result<()> {
    if fail {
        return Err(io::Error::other("injected failure"));
    }
    let mut first = None;
    for file in json_files(paths, backups) {
        if !file.exists() {
            continue;
        }
        if let Err(e) = fs::rename(&file, imported_name(&file)) {
            tracing::warn!("{}: could not rename: {e}", file.display());
            first.get_or_insert(e);
        }
    }
    first.map_or(Ok(()), Err)
}

fn remove_if_exists(path: &Path) {
    if let Err(e) = fs::remove_file(path)
        && e.kind() != io::ErrorKind::NotFound
    {
        tracing::warn!("{}: could not delete: {e}", path.display());
    }
}

fn remove_importing(paths: &AppPaths) {
    let file = importing_file(paths);
    for suffix in ["-wal", "-shm", "-journal"] {
        let mut name = file.clone().into_os_string();
        name.push(suffix);
        remove_if_exists(Path::new(&name));
    }
    remove_if_exists(&file);
}

/// Runs the import when it applies. `fail_rename` is the test hook that
/// makes the step that renames the JSON files fail.
pub(crate) fn import_legacy(
    paths: &AppPaths,
    limits: &Limits,
    fail_rename: bool,
    warnings: &mut Vec<String>,
) -> Imported {
    let backups = limits.backup_count;
    let database = paths.database_file();
    let marker = marker_file(paths);

    // An earlier import committed and was interrupted before its renames.
    if marker.exists() {
        if database.exists() {
            match rename_json_files(paths, backups, fail_rename) {
                Ok(()) => remove_if_exists(&marker),
                Err(e) => warnings.push(format!(
                    "import: could not rename the JSON files after the import: {e}"
                )),
            }
            return Imported::Nothing;
        }
        // The commit never happened.
        remove_if_exists(&marker);
    }

    // An existing database wins, even over newer JSON files.
    let has_database =
        database.exists() || (1..=backups).any(|n| backup_path(&database, n).exists());
    if has_database || !json_files(paths, backups).iter().any(|f| f.exists()) {
        return Imported::Nothing;
    }

    remove_importing(paths);
    let Some(snap) = read_documents(paths, limits, warnings) else {
        return Imported::Nothing;
    };
    let summary = format!(
        "{} tracks, {} playlists, {} cart pages and {} players",
        snap.library.len(),
        snap.playlists.len(),
        snap.cart_pages.len(),
        snap.players.len()
    );
    if let Err(e) = commit(paths, &snap) {
        remove_importing(paths);
        remove_if_exists(&marker);
        warnings.push(format!(
            "import: could not write the database: {e}; the JSON files are kept for the next start"
        ));
        return Imported::Failed(Box::new(snap));
    }
    tracing::info!("imported the JSON documents into the database: {summary}");
    match rename_json_files(paths, backups, fail_rename) {
        Ok(()) => remove_if_exists(&marker),
        Err(e) => warnings.push(format!(
            "import: could not rename the JSON files after the import: {e}"
        )),
    }
    Imported::Committed
}

/// Writes `snap` to the temporary database and renames it to `fauste.db`
/// behind the marker.
fn commit(paths: &AppPaths, snap: &Snapshot) -> Result<(), StoreError> {
    fs::create_dir_all(&paths.data_dir)?;
    let importing = importing_file(paths);
    let mut db = Db::open(&importing).map_err(|e| io::Error::other(e.to_string()))?;
    snapshot::write_full(&mut db, snap)?;
    // Fold the write-ahead log into the file, so the file alone is the
    // database once renamed.
    db.conn()
        .query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |_| Ok(()))
        .map_err(|e| io::Error::other(e.to_string()))?;
    drop(db);
    for suffix in ["-wal", "-shm"] {
        let mut name = importing.clone().into_os_string();
        name.push(suffix);
        remove_if_exists(Path::new(&name));
    }
    fs::write(marker_file(paths), b"")?;
    fs::rename(&importing, paths.database_file())?;
    Ok(())
}

/// Reads the three documents. None when none of them could be read.
fn read_documents(
    paths: &AppPaths,
    limits: &Limits,
    warnings: &mut Vec<String>,
) -> Option<Snapshot> {
    let lists: Loaded<PlaylistsDoc> = load_doc(
        &paths.playlists_file(),
        limits.backup_count,
        limits.max_state_file_bytes,
        PLAYLISTS_SCHEMA,
        PLAYLISTS_MIGRATIONS,
    );
    warnings.extend(lists.warnings);
    let session = load_with_fallback(
        &paths.session_file(),
        limits.backup_count,
        limits.max_state_file_bytes,
        parse_session,
    );
    warnings.extend(session.warnings);
    let carts = load_with_fallback(
        &paths.carts_file(),
        limits.backup_count,
        limits.max_state_file_bytes,
        parse_carts,
    );
    warnings.extend(carts.warnings);
    if lists.value.is_none() && session.value.is_none() && carts.value.is_none() {
        return None;
    }

    let (library, mut playlists, mut ids) = lists
        .value
        .map(|d| (d.library, d.playlists, d.ids))
        .unwrap_or_default();
    let (players, cartwall_session, layout) = session
        .value
        .map(|(doc, notes)| {
            warnings.extend(notes);
            (doc.players, doc.cartwall, doc.layout)
        })
        .unwrap_or_default();
    let cart_pages: Vec<CartPage> = carts
        .value
        .map(|(doc, notes)| {
            warnings.extend(notes);
            doc.pages
        })
        .unwrap_or_default();

    // The database keys its rows by id, so ids a hand edit repeated get new
    // ones (as the restore does).
    ids.observe(library.max_raw_id());
    ids.observe(playlists.max_raw_id());
    for player in &players {
        ids.observe(player.id.0);
    }
    warnings.extend(playlists.normalize(&mut ids));
    let mut players_seen = std::collections::HashSet::new();
    let players: Vec<PlayerSession> = players
        .into_iter()
        .filter(|p| {
            let fresh = players_seen.insert(p.id);
            if !fresh {
                warnings.push(format!(
                    "import: player {:?} appears twice; the later one is dropped",
                    p.id
                ));
            }
            fresh
        })
        .collect();

    // The old shared `played` flag is not stored: it becomes marks for
    // every player now (without a session the players are unknown, and the
    // marks are lost).
    if !players.is_empty() {
        let ids_of: Vec<_> = players.iter().map(|p| p.id).collect();
        playlists.convert_legacy_played(&ids_of);
    }

    // A reference to a track the library does not have cannot be stored:
    // the entry goes, the cart is emptied (as the restore does).
    for id in playlists.iter().map(|p| p.id).collect::<Vec<_>>() {
        if let Some(list) = playlists.get_mut(id) {
            let before = list.entries.len();
            list.entries.retain(|e| library.get(e.track).is_some());
            let dropped = before - list.entries.len();
            if dropped > 0 {
                warnings.push(format!(
                    "import: {dropped} entries of playlist \"{}\" name a track the library does not have; dropped",
                    list.name
                ));
            }
        }
    }
    let mut walls = fp_model::Cartwall {
        pages: cart_pages,
        ..fp_model::Cartwall::default()
    };
    ids.observe(walls.max_raw_id());
    warnings.extend(walls.normalize(limits, &mut ids, &library));

    Some(Snapshot {
        library,
        playlists,
        ids,
        players,
        cartwall: cartwall_session,
        layout,
        cart_pages: walls.pages,
    })
}
