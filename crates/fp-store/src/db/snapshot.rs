//! State snapshots in the database: read leniently, written in full or
//! row by changed row. Each write is one transaction, so a failed save
//! leaves the previous state.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use fp_model::{
    AppState, CartPage, CartwallSession, IdGen, Layout, Library, PlayerId, PlayerSession, Playlist,
    PlaylistEntry, Playlists, Track,
};
use rusqlite::types::Value as SqlValue;
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use super::Db;
use crate::error::StoreError;

const KV_IDS: &str = "ids";
const KV_CARTWALL: &str = "cartwall_session";
const KV_LAYOUT: &str = "layout";

/// Everything the database holds, as plain model values.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Snapshot {
    pub library: Library,
    pub playlists: Playlists,
    pub ids: IdGen,
    pub players: Vec<PlayerSession>,
    pub cartwall: CartwallSession,
    pub layout: Layout,
    pub cart_pages: Vec<CartPage>,
}

impl Snapshot {
    /// The part of `state` that is saved. `position_of` gives a player's
    /// playback position in seconds.
    pub fn of(state: &AppState, position_of: impl Fn(PlayerId) -> f64) -> Snapshot {
        Snapshot {
            library: state.library.clone(),
            playlists: state.playlists.clone(),
            ids: state.ids.clone(),
            players: state.sessions(position_of),
            cartwall: state.cartwall.session(),
            layout: state.layout.clone(),
            cart_pages: state.cartwall.pages.clone(),
        }
    }
}

fn db_error(e: impl std::fmt::Display) -> StoreError {
    StoreError::Io(std::io::Error::other(e.to_string()))
}

fn json_error(e: impl std::fmt::Display) -> StoreError {
    StoreError::Serialize(e.to_string())
}

fn key(raw: u64) -> i64 {
    i64::try_from(raw).unwrap_or(i64::MAX)
}

fn raw(key: i64) -> u64 {
    u64::try_from(key).unwrap_or_default()
}

fn position(index: usize) -> i64 {
    i64::try_from(index).unwrap_or(i64::MAX)
}

fn doc<T: Serialize>(value: &T) -> Result<String, StoreError> {
    serde_json::to_string(value).map_err(json_error)
}

// ---------------------------------------------------------------- reading

/// Reads the snapshot. A row or document that is unusable is repaired (or
/// dropped) with one warning; the other rows are never affected.
pub fn read(db: &Db) -> (Snapshot, Vec<String>) {
    let read = read_checked(db);
    (read.snapshot, read.warnings)
}

/// What [`read_checked`] found.
#[derive(Debug)]
pub struct Read {
    pub snapshot: Snapshot,
    pub warnings: Vec<String>,
    /// False when a whole table or document could not be read (as opposed
    /// to a row that was repaired or dropped): the snapshot is then missing
    /// data that is still on disk, and the database must not be written to.
    pub complete: bool,
}

/// Like [`read`], and says whether every table could be read.
pub fn read_checked(db: &Db) -> Read {
    let mut warnings = Notes::default();
    let conn = &db.conn;
    let snapshot = Snapshot {
        library: read_library(conn, &mut warnings),
        playlists: read_playlists(conn, &mut warnings),
        ids: read_kv(conn, KV_IDS, "ids", &mut warnings),
        players: read_players(conn, &mut warnings),
        cartwall: read_kv(conn, KV_CARTWALL, "the cartwall session", &mut warnings),
        layout: read_layout(conn, &mut warnings),
        cart_pages: read_cart_pages(conn, &mut warnings),
    };
    Read {
        snapshot,
        complete: !warnings.unreadable,
        warnings: warnings.lines,
    }
}

/// The warnings of a read, and whether a table could not be read at all.
#[derive(Default)]
struct Notes {
    lines: Vec<String>,
    unreadable: bool,
}

impl Notes {
    fn push(&mut self, line: String) {
        self.lines.push(line);
    }

    fn extend(&mut self, lines: impl IntoIterator<Item = String>) {
        self.lines.extend(lines);
    }
}

/// One row as loosely typed values, so a value of the wrong storage type
/// spoils only its own row.
type Raw = Vec<SqlValue>;

fn int(v: &SqlValue) -> Option<i64> {
    match v {
        SqlValue::Integer(i) => Some(*i),
        _ => None,
    }
}

fn text(v: &SqlValue) -> Option<&str> {
    match v {
        SqlValue::Text(t) => Some(t),
        _ => None,
    }
}

/// Runs a query returning `columns` columns per row. A failure of the query
/// itself is one warning and no rows; a row that cannot be read is one
/// warning and is skipped.
fn rows(
    conn: &Connection,
    sql: &str,
    columns: usize,
    what: &str,
    warnings: &mut Notes,
) -> Vec<Raw> {
    let mut out = Vec::new();
    let result = conn.prepare(sql).and_then(|mut stmt| {
        let mut cursor = stmt.query([])?;
        while let Some(row) = cursor.next()? {
            match (0..columns)
                .map(|i| row.get::<_, SqlValue>(i))
                .collect::<rusqlite::Result<Raw>>()
            {
                Ok(raw) => out.push(raw),
                Err(e) => warnings.push(format!("database: a row of {what} is unreadable: {e}")),
            }
        }
        Ok(())
    });
    if let Err(e) = result {
        warnings.unreadable = true;
        warnings.push(format!("database: cannot read {what}: {e}"));
    }
    out
}

fn read_library(conn: &Connection, warnings: &mut Notes) -> Library {
    let rows = rows(
        conn,
        "SELECT id, path, doc FROM tracks ORDER BY id",
        3,
        "the tracks",
        warnings,
    );
    let mut tracks = Vec::with_capacity(rows.len());
    for row in rows {
        let [id, path, doc] = row.as_slice() else {
            continue;
        };
        let (Some(id), Some(path)) = (int(id), text(path)) else {
            warnings.push(format!(
                "library: a track record with an unusable id or path was dropped ({})",
                id_text(id)
            ));
            continue;
        };
        let id = fp_model::TrackId(raw(id));
        let path = PathBuf::from(path);
        let parsed = text(doc)
            .ok_or_else(|| "not text".to_owned())
            .and_then(|t| serde_json::from_str::<Track>(t).map_err(|e| e.to_string()));
        let track = match parsed {
            Ok(mut track) => {
                track.id = id;
                track.path = path;
                track
            }
            Err(e) => {
                warnings.push(format!(
                    "library: the record of track {} is not valid ({e}); loaded without its analysis",
                    id.0
                ));
                Track::new(id, path)
            }
        };
        tracks.push(track);
    }
    Library::from(tracks)
}

fn id_text(v: &SqlValue) -> String {
    format!("{v:?}")
}

/// Parses the `doc` column; a value that is not text is as bad as one that
/// does not parse.
fn parse_doc<T: DeserializeOwned>(doc: &SqlValue) -> Result<T, String> {
    let text = text(doc).ok_or_else(|| "not text".to_owned())?;
    serde_json::from_str(text).map_err(|e| e.to_string())
}

fn read_playlists(conn: &Connection, warnings: &mut Notes) -> Playlists {
    let lists = rows(
        conn,
        "SELECT id, name FROM playlists ORDER BY position, id",
        2,
        "the playlists",
        warnings,
    );
    let entry_rows = rows(
        conn,
        "SELECT id, playlist, track, doc FROM entries ORDER BY playlist, position, id",
        4,
        "the playlist entries",
        warnings,
    );
    let mut by_playlist: HashMap<i64, Vec<PlaylistEntry>> = HashMap::new();
    for row in entry_rows {
        let [id, playlist, track, doc] = row.as_slice() else {
            continue;
        };
        let (Some(id), Some(playlist), Some(track)) = (int(id), int(playlist), int(track)) else {
            warnings.push(format!(
                "playlists: an entry record with an unusable id, playlist or track was dropped ({})",
                id_text(&row[0])
            ));
            continue;
        };
        let (id, track) = (fp_model::EntryId(raw(id)), fp_model::TrackId(raw(track)));
        let entry = match parse_doc::<PlaylistEntry>(doc) {
            Ok(mut entry) => {
                entry.id = id;
                entry.track = track;
                entry
            }
            Err(e) => {
                warnings.push(format!(
                    "playlists: the record of entry {} is not valid ({e}); loaded without its marks",
                    id.0
                ));
                PlaylistEntry::new(id, track)
            }
        };
        by_playlist.entry(playlist).or_default().push(entry);
    }
    let mut playlists = Playlists::default();
    for row in lists {
        let [id, name] = row.as_slice() else {
            continue;
        };
        let (Some(id), Some(name)) = (int(id), text(name)) else {
            warnings.push(format!(
                "playlists: a playlist record with an unusable id or name was dropped ({})",
                id_text(&row[0])
            ));
            continue;
        };
        let mut list = Playlist::new(fp_model::PlaylistId(raw(id)), name);
        list.entries = by_playlist.remove(&id).unwrap_or_default();
        playlists.add(list);
    }
    playlists
}

/// A document of `kv`: a missing key is the default without a warning, one
/// that does not parse is the default with one.
fn kv_text(conn: &Connection, name: &str, warnings: &mut Notes) -> Option<String> {
    match conn
        .query_row("SELECT value FROM kv WHERE key = ?1", [name], |r| r.get(0))
        .optional()
    {
        Ok(text) => text,
        Err(e) => {
            warnings.unreadable = true;
            warnings.push(format!("database: cannot read {name}: {e}"));
            None
        }
    }
}

fn read_kv<T: DeserializeOwned + Default>(
    conn: &Connection,
    name: &str,
    what: &str,
    warnings: &mut Notes,
) -> T {
    let Some(text) = kv_text(conn, name, warnings) else {
        return T::default();
    };
    serde_json::from_str(&text).unwrap_or_else(|e| {
        warnings.push(format!(
            "session: the saved {what} is not valid ({e}); the default is used"
        ));
        T::default()
    })
}

fn read_layout(conn: &Connection, warnings: &mut Notes) -> Layout {
    let Some(text) = kv_text(conn, KV_LAYOUT, warnings) else {
        return Layout::default();
    };
    let value: Value = match serde_json::from_str(&text) {
        Ok(value) => value,
        Err(e) => {
            warnings.push(format!(
                "session: the saved layout is not valid ({e}); the default layout is used"
            ));
            return Layout::default();
        }
    };
    warnings.extend(crate::layout_notes::layout_notes(Some(&value)));
    fp_model::layout::lenient_layout(value).unwrap_or_default()
}

fn read_players(conn: &Connection, warnings: &mut Notes) -> Vec<PlayerSession> {
    let rows = rows(
        conn,
        "SELECT id, doc FROM players ORDER BY position, id",
        2,
        "the players",
        warnings,
    );
    let mut players = Vec::with_capacity(rows.len());
    for row in rows {
        let [id, doc] = row.as_slice() else {
            continue;
        };
        let Some(id) = int(id).map(|i| PlayerId(raw(i))) else {
            warnings.push(format!(
                "session: a player record with an unusable id was dropped ({})",
                id_text(&row[0])
            ));
            continue;
        };
        match parse_doc::<PlayerSession>(doc) {
            Ok(mut player) => {
                player.id = id;
                // The model drops a pending start that does not parse; say so.
                let had_pending = text(doc)
                    .and_then(|t| serde_json::from_str::<Value>(t).ok())
                    .and_then(|v| v.get("pending_start").map(|p| !p.is_null()))
                    .unwrap_or(false);
                if had_pending && player.pending_start.is_none() {
                    warnings.push(format!(
                        "session: the saved pending start of player {id:?} is not valid; ignored"
                    ));
                }
                players.push(player);
            }
            Err(e) => warnings.push(format!(
                "session: the record of player {} is not valid ({e}); dropped",
                id.0
            )),
        }
    }
    players
}

fn read_cart_pages(conn: &Connection, warnings: &mut Notes) -> Vec<CartPage> {
    let rows = rows(
        conn,
        "SELECT id, doc FROM cart_pages ORDER BY position, id",
        2,
        "the cart pages",
        warnings,
    );
    let mut pages = Vec::with_capacity(rows.len());
    for row in rows {
        let [id, doc] = row.as_slice() else {
            continue;
        };
        let Some(id) = int(id).map(|i| fp_model::CartPageId(raw(i))) else {
            warnings.push(format!(
                "carts: a page record with an unusable id was dropped ({})",
                id_text(&row[0])
            ));
            continue;
        };
        match parse_doc::<CartPage>(doc) {
            Ok(mut page) => {
                page.id = id;
                // Cart colour spec CC3: a colour that does not parse loads
                // as none; say so.
                if let Some(value) = text(doc).and_then(|t| serde_json::from_str::<Value>(t).ok()) {
                    warnings.extend(crate::legacy::unknown_cart_colors(
                        &serde_json::json!({ "pages": [value] }),
                    ));
                }
                pages.push(page);
            }
            Err(e) => warnings.push(format!(
                "carts: the record of page {} is not valid ({e}); dropped",
                id.0
            )),
        }
    }
    pages
}

// ---------------------------------------------------------------- writing

// Test-only fail point: the next `write_changes` fails after its first
// stage. It costs nothing in release builds.
#[cfg(test)]
thread_local! {
    static FAIL_AFTER_FIRST_STAGE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
fn fail_point() -> Result<(), StoreError> {
    if FAIL_AFTER_FIRST_STAGE.with(|f| f.replace(false)) {
        return Err(db_error("injected failure"));
    }
    Ok(())
}

#[cfg(not(test))]
#[inline(always)]
fn fail_point() -> Result<(), StoreError> {
    Ok(())
}

/// Replaces the database content with `snap`.
pub fn write_full(db: &mut Db, snap: &Snapshot) -> Result<(), StoreError> {
    let tx = db
        .conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(db_error)?;
    for table in [
        "entries",
        "playlists",
        "tracks",
        "cart_pages",
        "players",
        "kv",
    ] {
        tx.execute(&format!("DELETE FROM {table}"), [])
            .map_err(db_error)?;
    }
    for track in snap.library.iter() {
        upsert_track(&tx, track)?;
    }
    for (index, list) in snap.playlists.iter().enumerate() {
        upsert_playlist(&tx, list, index)?;
        insert_entries(&tx, list)?;
    }
    for (index, page) in snap.cart_pages.iter().enumerate() {
        upsert_doc(&tx, "cart_pages", page.id.0, index, page)?;
    }
    for (index, player) in snap.players.iter().enumerate() {
        upsert_doc(&tx, "players", player.id.0, index, player)?;
    }
    write_kv(&tx, KV_IDS, &snap.ids)?;
    write_kv(&tx, KV_CARTWALL, &snap.cartwall)?;
    write_kv(&tx, KV_LAYOUT, &snap.layout)?;
    tx.commit().map_err(db_error)
}

/// Brings the database from `old` to `new` writing only the rows that
/// differ, in one transaction.
///
/// Caller contract:
/// - `old` must be exactly what the database holds. Rows on disk that `old`
///   does not know about are never deleted.
/// - After an `Err` nothing was written: keep `old` as the baseline.
/// - A baseline that comes from a [`read`] with warnings must first be
///   written with [`write_full`], so repaired and dropped rows and stale
///   positions are normalised.
pub fn write_changes(db: &mut Db, old: &Snapshot, new: &Snapshot) -> Result<(), StoreError> {
    let tx = db
        .conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(db_error)?;

    // Stage 1: the entries of every changed or removed playlist, and the
    // removed playlists. Done first so an entry that moved between
    // playlists never meets its old row.
    let old_lists: HashMap<_, _> = old.playlists.iter().map(|p| (p.id, p)).collect();
    let new_ids: HashSet<_> = new.playlists.iter().map(|p| p.id).collect();
    let changed: Vec<(usize, &Playlist)> = new
        .playlists
        .iter()
        .enumerate()
        .filter(|(_, p)| old_lists.get(&p.id) != Some(p))
        .collect();
    {
        let mut delete_entries = tx
            .prepare_cached("DELETE FROM entries WHERE playlist = ?1")
            .map_err(db_error)?;
        for (_, list) in &changed {
            delete_entries.execute([key(list.id.0)]).map_err(db_error)?;
        }
        let mut delete_list = tx
            .prepare_cached("DELETE FROM playlists WHERE id = ?1")
            .map_err(db_error)?;
        for id in old_lists.keys().filter(|id| !new_ids.contains(id)) {
            delete_entries.execute([key(id.0)]).map_err(db_error)?;
            delete_list.execute([key(id.0)]).map_err(db_error)?;
        }
    }
    fail_point()?;

    // Stage 2: tracks.
    let tracks = new.library.changes_since(&old.library);
    for track in &tracks.upserted {
        upsert_track(&tx, track)?;
    }
    {
        let mut delete = tx
            .prepare_cached("DELETE FROM tracks WHERE id = ?1")
            .map_err(db_error)?;
        for id in &tracks.removed {
            delete.execute([key(id.0)]).map_err(db_error)?;
        }
    }

    // Stage 3: playlists and their entries.
    for (index, list) in &changed {
        upsert_playlist(&tx, list, *index)?;
        insert_entries(&tx, list)?;
    }
    let order = |lists: &Playlists| lists.iter().map(|p| p.id).collect::<Vec<_>>();
    if order(&old.playlists) != order(&new.playlists) {
        let changed_ids: HashSet<_> = changed.iter().map(|(_, p)| p.id).collect();
        let mut update = tx
            .prepare_cached("UPDATE playlists SET position = ?1 WHERE id = ?2")
            .map_err(db_error)?;
        for (index, list) in new.playlists.iter().enumerate() {
            if !changed_ids.contains(&list.id) {
                update
                    .execute(params![position(index), key(list.id.0)])
                    .map_err(db_error)?;
            }
        }
    }

    sync_docs(&tx, "cart_pages", &old.cart_pages, &new.cart_pages, |p| {
        p.id.0
    })?;
    sync_docs(&tx, "players", &old.players, &new.players, |p| p.id.0)?;

    if old.ids != new.ids {
        write_kv(&tx, KV_IDS, &new.ids)?;
    }
    if old.cartwall != new.cartwall {
        write_kv(&tx, KV_CARTWALL, &new.cartwall)?;
    }
    if old.layout != new.layout {
        write_kv(&tx, KV_LAYOUT, &new.layout)?;
    }
    tx.commit().map_err(db_error)
}

fn upsert_track(tx: &Transaction<'_>, track: &Track) -> Result<(), StoreError> {
    let text = doc(track)?;
    tx.prepare_cached(
        "INSERT INTO tracks (id, path, doc) VALUES (?1, ?2, ?3) \
         ON CONFLICT(id) DO UPDATE SET path = excluded.path, doc = excluded.doc",
    )
    .and_then(|mut s| s.execute(params![key(track.id.0), track.path.to_string_lossy(), text]))
    .map(drop)
    .map_err(db_error)
}

fn upsert_playlist(tx: &Transaction<'_>, list: &Playlist, index: usize) -> Result<(), StoreError> {
    tx.prepare_cached(
        "INSERT INTO playlists (id, position, name) VALUES (?1, ?2, ?3) \
         ON CONFLICT(id) DO UPDATE SET position = excluded.position, name = excluded.name",
    )
    .and_then(|mut s| s.execute(params![key(list.id.0), position(index), list.name]))
    .map(drop)
    .map_err(db_error)
}

fn insert_entries(tx: &Transaction<'_>, list: &Playlist) -> Result<(), StoreError> {
    let mut insert = tx
        .prepare_cached(
            "INSERT INTO entries (id, playlist, position, track, doc) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .map_err(db_error)?;
    for (index, entry) in list.entries.iter().enumerate() {
        insert
            .execute(params![
                key(entry.id.0),
                key(list.id.0),
                position(index),
                key(entry.track.0),
                doc(entry)?
            ])
            .map_err(db_error)?;
    }
    Ok(())
}

fn upsert_doc<T: Serialize>(
    tx: &Transaction<'_>,
    table: &str,
    id: u64,
    index: usize,
    value: &T,
) -> Result<(), StoreError> {
    let text = doc(value)?;
    tx.prepare_cached(&format!(
        "INSERT INTO {table} (id, position, doc) VALUES (?1, ?2, ?3) \
         ON CONFLICT(id) DO UPDATE SET position = excluded.position, doc = excluded.doc"
    ))
    .and_then(|mut s| s.execute(params![key(id), position(index), text]))
    .map(drop)
    .map_err(db_error)
}

/// Writes the rows of `table` (id, position, doc) that differ between
/// `old` and `new`, and deletes the ones that are gone.
fn sync_docs<T: Serialize + PartialEq>(
    tx: &Transaction<'_>,
    table: &str,
    old: &[T],
    new: &[T],
    id_of: impl Fn(&T) -> u64,
) -> Result<(), StoreError> {
    let old_at: HashMap<u64, (usize, &T)> = old
        .iter()
        .enumerate()
        .map(|(i, v)| (id_of(v), (i, v)))
        .collect();
    let mut kept = HashSet::new();
    for (index, value) in new.iter().enumerate() {
        let id = id_of(value);
        kept.insert(id);
        let same = old_at
            .get(&id)
            .is_some_and(|(at, before)| *at == index && *before == value);
        if !same {
            upsert_doc(tx, table, id, index, value)?;
        }
    }
    let mut delete = tx
        .prepare_cached(&format!("DELETE FROM {table} WHERE id = ?1"))
        .map_err(db_error)?;
    for id in old_at.keys().filter(|id| !kept.contains(id)) {
        delete.execute([key(*id)]).map_err(db_error)?;
    }
    Ok(())
}

fn write_kv<T: Serialize>(tx: &Transaction<'_>, name: &str, value: &T) -> Result<(), StoreError> {
    let text = doc(value)?;
    tx.prepare_cached(
        "INSERT INTO kv (key, value) VALUES (?1, ?2) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .and_then(|mut s| s.execute(params![name, text]))
    .map(drop)
    .map_err(db_error)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use fp_model::{PlaylistId, TrackId};

    fn snapshot() -> Snapshot {
        let mut snap = Snapshot::default();
        snap.library
            .insert(Track::new(TrackId(1), PathBuf::from("/m/a.wav")));
        let mut list = Playlist::new(PlaylistId(2), "Main");
        list.entries
            .push(PlaylistEntry::new(fp_model::EntryId(3), TrackId(1)));
        snap.playlists.add(list);
        snap
    }

    #[test]
    fn a_failed_save_leaves_the_previous_state() {
        let dir = tempfile::tempdir().unwrap();
        let mut db = Db::open(&dir.path().join("fauste.db")).unwrap();
        let old = snapshot();
        write_full(&mut db, &old).unwrap();

        let mut new = old.clone();
        new.playlists.get_mut(PlaylistId(2)).unwrap().name = "Renamed".into();
        new.playlists
            .get_mut(PlaylistId(2))
            .unwrap()
            .entries
            .clear();
        new.library
            .insert(Track::new(TrackId(9), PathBuf::from("/m/b.wav")));
        FAIL_AFTER_FIRST_STAGE.with(|f| f.set(true));
        assert!(write_changes(&mut db, &old, &new).is_err());

        let (back, warnings) = read(&db);
        assert!(warnings.is_empty());
        assert_eq!(back, old);
        // The fail point is spent: the same save now succeeds.
        write_changes(&mut db, &old, &new).unwrap();
        assert_eq!(read(&db).0, new);
    }
}
