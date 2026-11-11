//! State snapshots in the database: read leniently, written in full or
//! row by changed row. Each write is one transaction, so a failed save
//! leaves the previous state.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use fp_model::rotation::{Rotation, STEP_COUNT_RANGE};
use fp_model::{
    AirTimes, AppState, CartPage, CartwallSession, Categories, Category, CategoryId, IdGen, Layout,
    Library, LibraryView, PlayerId, PlayerSession, Playlist, PlaylistEntry, Playlists, RotationId,
    Track, TrackId, WallTime,
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
const KV_LIBRARY_VIEW: &str = "library_view";
const KV_DEFAULT_ROTATION: &str = "default_rotation";
const KV_FILLER: &str = "filler";

/// Everything the database holds, as plain model values.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Snapshot {
    pub library: Library,
    /// The category tree, in display order.
    pub categories: Vec<Category>,
    pub playlists: Playlists,
    pub ids: IdGen,
    pub players: Vec<PlayerSession>,
    pub cartwall: CartwallSession,
    pub layout: Layout,
    pub library_view: LibraryView,
    pub cart_pages: Vec<CartPage>,
    /// The music filler's rotations, in display order (schema 3).
    pub rotations: Vec<Rotation>,
    pub default_rotation: Option<RotationId>,
    /// When each track last aired (schema 3).
    pub air_times: AirTimes,
    /// The music filler (`kv` value `filler`); `None` before it exists.
    pub filler: Option<fp_model::filler::FillerSession>,
}

impl Snapshot {
    /// The part of `state` that is saved. `position_of` gives a player's
    /// playback position in seconds.
    pub fn of(state: &AppState, position_of: impl Fn(PlayerId) -> f64) -> Snapshot {
        Snapshot {
            library: state.library.clone(),
            categories: state.categories.clone().into_vec(),
            playlists: state.playlists.clone(),
            ids: state.ids.clone(),
            players: state.sessions(&position_of),
            cartwall: state.cartwall.session(),
            layout: state.layout.clone(),
            library_view: state.library_view.clone(),
            cart_pages: state.cartwall.pages.clone(),
            rotations: state.rotations.iter().cloned().collect(),
            default_rotation: state.rotations.default_id(),
            air_times: state.air_times.clone(),
            filler: state.filler_session(&position_of),
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
    let categories = read_categories(conn, &mut warnings);
    let library = read_library(conn, &categories, &mut warnings);
    let snapshot = Snapshot {
        categories,
        playlists: read_playlists(conn, &library, &mut warnings),
        library,
        ids: read_kv(conn, KV_IDS, "ids", &mut warnings),
        players: read_players(conn, &mut warnings),
        cartwall: read_kv(conn, KV_CARTWALL, "the cartwall session", &mut warnings),
        layout: read_layout(conn, &mut warnings),
        library_view: read_library_view(conn, &mut warnings),
        cart_pages: read_cart_pages(conn, &mut warnings),
        rotations: read_rotations(conn, &mut warnings),
        default_rotation: read_kv(conn, KV_DEFAULT_ROTATION, "default rotation", &mut warnings),
        air_times: AirTimes::default(),
        filler: read_kv(conn, KV_FILLER, "music filler", &mut warnings),
    };
    let mut snapshot = snapshot;
    snapshot.air_times = read_air_times(conn, &snapshot.library, &mut warnings);
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

/// Reads the category tree in `position` order and repairs it.
fn read_categories(conn: &Connection, warnings: &mut Notes) -> Vec<Category> {
    let rows = rows(
        conn,
        "SELECT id, doc FROM categories ORDER BY position, id",
        2,
        "the categories",
        warnings,
    );
    let mut list = Vec::with_capacity(rows.len());
    for row in rows {
        let [id, doc] = row.as_slice() else {
            continue;
        };
        let Some(id) = int(id).map(|i| CategoryId(raw(i))) else {
            warnings.push(format!(
                "categories: a category record with an unusable id was dropped ({})",
                id_text(&row[0])
            ));
            continue;
        };
        match parse_doc::<Category>(doc) {
            Ok(mut category) => {
                category.id = id;
                list.push(category);
            }
            Err(e) => warnings.push(format!(
                "categories: the record of category {} is not valid ({e}); dropped",
                id.0
            )),
        }
    }
    let (fixed, notes) = Categories::from(list).repaired();
    warnings.extend(notes);
    fixed.into_vec()
}

/// Reads the rotations in `position` order. An unparsable record is
/// dropped; a step's counts are brought into `STEP_COUNT_RANGE` with
/// `min <= max`.
fn read_rotations(conn: &Connection, warnings: &mut Notes) -> Vec<Rotation> {
    let rows = rows(
        conn,
        "SELECT id, doc FROM rotations ORDER BY position, id",
        2,
        "the rotations",
        warnings,
    );
    let mut list = Vec::with_capacity(rows.len());
    for row in rows {
        let [id, doc] = row.as_slice() else {
            continue;
        };
        let Some(id) = int(id).map(|i| RotationId(raw(i))) else {
            warnings.push(format!(
                "rotations: a rotation record with an unusable id was dropped ({})",
                id_text(id)
            ));
            continue;
        };
        let mut rotation = match parse_doc::<Rotation>(doc) {
            Ok(rotation) => rotation,
            Err(e) => {
                warnings.push(format!(
                    "rotations: the record of rotation {} is not valid ({e}); dropped",
                    id.0
                ));
                continue;
            }
        };
        rotation.id = id;
        for step in &mut rotation.steps {
            let (lo, hi) = (*STEP_COUNT_RANGE.start(), *STEP_COUNT_RANGE.end());
            let (min, max) = (step.min.clamp(lo, hi), step.max.clamp(lo, hi));
            let (min, max) = (min.min(max), min.max(max));
            if (min, max) != (step.min, step.max) {
                warnings.push(format!(
                    "rotations: a step of rotation {} counted {}–{}; now {min}–{max}",
                    id.0, step.min, step.max
                ));
                step.min = min;
                step.max = max;
            }
        }
        list.push(rotation);
    }
    list
}

/// Reads the air times of the tracks in `library`; a row of another track
/// is dropped, like the time of a removed track.
fn read_air_times(conn: &Connection, library: &Library, warnings: &mut Notes) -> AirTimes {
    rows(
        conn,
        "SELECT track, at FROM air_times",
        2,
        "the air times",
        warnings,
    )
    .into_iter()
    .filter_map(|row| match row.as_slice() {
        [track, at] => Some((TrackId(raw(int(track)?)), WallTime(int(at)?))),
        _ => None,
    })
    .filter(|(track, _)| library.get(*track).is_some())
    .collect()
}

fn read_library(conn: &Connection, categories: &[Category], warnings: &mut Notes) -> Library {
    let rows = rows(
        conn,
        "SELECT id, path, doc, category FROM tracks ORDER BY id",
        4,
        "the tracks",
        warnings,
    );
    let mut tracks = Vec::with_capacity(rows.len());
    for row in rows {
        let [id, path, doc, category] = row.as_slice() else {
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
        let mut track = match parsed {
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
        // The column wins over the document; a category that does not exist
        // (or a column that is not an integer) leaves the track uncategorised.
        track.category = match category {
            SqlValue::Null => None,
            other => {
                let known = int(other)
                    .map(|c| CategoryId(raw(c)))
                    .filter(|c| categories.iter().any(|k| k.id == *c));
                if known.is_none() {
                    warnings.push(format!(
                        "library: track {} is in an unknown category; loaded uncategorised",
                        track.id.0
                    ));
                }
                known
            }
        };
        tracks.push(track);
    }
    Library::from(tracks)
}

/// Longest text of a bad id value shown in a warning.
const ID_TEXT_CHARS: usize = 64;

/// A bad id value for a warning: text and blobs are cut, so a huge value
/// cannot flood the warnings.
fn id_text(v: &SqlValue) -> String {
    let shown = match v {
        SqlValue::Text(t) => t.clone(),
        SqlValue::Blob(b) => String::from_utf8_lossy(b).into_owned(),
        other => return format!("{other:?}"),
    };
    let mut cut: String = shown.chars().take(ID_TEXT_CHARS).collect();
    if shown.chars().count() > ID_TEXT_CHARS {
        cut.push('…');
    }
    format!("{cut:?}")
}

/// Parses the `doc` column; a value that is not text is as bad as one that
/// does not parse.
fn parse_doc<T: DeserializeOwned>(doc: &SqlValue) -> Result<T, String> {
    let text = text(doc).ok_or_else(|| "not text".to_owned())?;
    serde_json::from_str(text).map_err(|e| e.to_string())
}

/// Reads the playlists. An entry whose track is not in `library` (its row
/// was unreadable) is dropped, so the snapshot written back is consistent;
/// so is an entry whose playlist is missing. One warning per playlist.
fn read_playlists(conn: &Connection, library: &Library, warnings: &mut Notes) -> Playlists {
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
        let before = list.entries.len();
        list.entries.retain(|e| library.get(e.track).is_some());
        let dropped = before - list.entries.len();
        if dropped > 0 {
            warnings.push(format!(
                "playlists: {dropped} entries of \"{}\" were dropped: their track cannot be read",
                list.name
            ));
        }
        playlists.add(list);
    }
    let mut orphans: Vec<(i64, usize)> =
        by_playlist.into_iter().map(|(k, v)| (k, v.len())).collect();
    orphans.sort_unstable();
    for (playlist, count) in orphans {
        warnings.push(format!(
            "playlists: {count} entries of the missing playlist {playlist} were dropped"
        ));
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

fn read_library_view(conn: &Connection, warnings: &mut Notes) -> LibraryView {
    let Some(text) = kv_text(conn, KV_LIBRARY_VIEW, warnings) else {
        return LibraryView::default();
    };
    let value: Value = match serde_json::from_str(&text) {
        Ok(value) => value,
        Err(e) => {
            warnings.push(format!(
                "session: the saved library window is not valid ({e}); the default is used"
            ));
            return LibraryView::default();
        }
    };
    let view = fp_model::lenient_library_view(value.clone()).unwrap_or_default();
    // A field that fell back on its default (or lost an element) is reported.
    if let Value::Object(saved) = &value {
        for (key, v) in saved {
            if !library_view_field_kept(key, v, &view) {
                warnings.push(format!(
                    "session: the saved library window's `{key}` is not valid; its default is used"
                ));
            }
        }
    } else {
        warnings
            .push("session: the saved library window is not valid; the default is used".to_owned());
    }
    view
}

/// Whether the saved `key` was kept as it is in `view`: its value parses
/// into the field's own type, equals what the lenient load kept, and is
/// within range. Compared as typed values, never as JSON: an `f32` written
/// back to JSON widens (268.8 becomes 268.799...), so a text or `Value`
/// comparison would report every fractional size.
fn library_view_field_kept(key: &str, saved: &Value, view: &LibraryView) -> bool {
    use fp_model::library_view::{good_rect, good_size};
    match key {
        "open" => parses_to(saved, &view.open),
        "rect" => parses_to(saved, &view.rect) && view.rect.is_none_or(good_rect),
        "minimised" => parses_to(saved, &view.minimised),
        "maximised" => parses_to(saved, &view.maximised),
        "selection" => parses_to(saved, &view.selection),
        "expanded" => parses_to(saved, &view.expanded),
        "whole_library" => parses_to(saved, &view.whole_library),
        "columns" => {
            parses_to(saved, &view.columns) && view.columns.iter().all(|c| good_size(c.width))
        }
        "sort" => parses_to(saved, &view.sort),
        "tree_width" => parses_to(saved, &view.tree_width) && view.tree_width.is_none_or(good_size),
        "cue_volume" => {
            parses_to(saved, &view.cue_volume) && (0.0..=1.0).contains(&view.cue_volume)
        }
        // A key the view does not have is not kept.
        _ => false,
    }
}

/// Whether `saved` parses strictly into `T` and equals `kept`.
fn parses_to<T: DeserializeOwned + PartialEq>(saved: &Value, kept: &T) -> bool {
    T::deserialize(saved).is_ok_and(|v| v == *kept)
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
        // Parsed once: as a value, then into the session.
        let parsed = text(doc)
            .ok_or_else(|| "not text".to_owned())
            .and_then(|t| serde_json::from_str::<Value>(t).map_err(|e| e.to_string()))
            .and_then(|v| {
                let had_pending = v.get("pending_start").is_some_and(|p| !p.is_null());
                serde_json::from_value::<PlayerSession>(v)
                    .map(|p| (p, had_pending))
                    .map_err(|e| e.to_string())
            });
        match parsed {
            Ok((mut player, had_pending)) => {
                player.id = id;
                // The model drops a pending start that does not parse; say so.
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
        "categories",
        "cart_pages",
        "players",
        "rotations",
        "air_times",
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
    for (index, category) in snap.categories.iter().enumerate() {
        upsert_doc(&tx, "categories", category.id.0, index, category)?;
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
    write_kv(&tx, KV_LIBRARY_VIEW, &snap.library_view)?;
    for (index, rotation) in snap.rotations.iter().enumerate() {
        upsert_doc(&tx, "rotations", rotation.id.0, index, rotation)?;
    }
    write_kv(&tx, KV_DEFAULT_ROTATION, &snap.default_rotation)?;
    write_air_times(&tx, &AirTimes::default(), &snap.air_times)?;
    write_kv(&tx, KV_FILLER, &snap.filler)?;
    tx.commit().map_err(db_error)
}

/// Writes the air times that differ between `old` and `new`, and deletes
/// the tracks `new` no longer has.
fn write_air_times(tx: &Transaction<'_>, old: &AirTimes, new: &AirTimes) -> Result<(), StoreError> {
    let mut upsert = tx
        .prepare_cached(
            "INSERT INTO air_times (track, at) VALUES (?1, ?2) \
             ON CONFLICT(track) DO UPDATE SET at = excluded.at",
        )
        .map_err(db_error)?;
    for (track, at) in new.iter() {
        if old.get(track) != Some(at) {
            upsert
                .execute(params![key(track.0), at.0])
                .map_err(db_error)?;
        }
    }
    let mut delete = tx
        .prepare_cached("DELETE FROM air_times WHERE track = ?1")
        .map_err(db_error)?;
    for (track, _) in old.iter() {
        if new.get(track).is_none() {
            delete.execute([key(track.0)]).map_err(db_error)?;
        }
    }
    Ok(())
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

    sync_docs(&tx, "categories", &old.categories, &new.categories, |c| {
        c.id.0
    })?;
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
    if old.library_view != new.library_view {
        write_kv(&tx, KV_LIBRARY_VIEW, &new.library_view)?;
    }
    if old.layout != new.layout {
        write_kv(&tx, KV_LAYOUT, &new.layout)?;
    }
    sync_docs(&tx, "rotations", &old.rotations, &new.rotations, |r| r.id.0)?;
    if old.default_rotation != new.default_rotation {
        write_kv(&tx, KV_DEFAULT_ROTATION, &new.default_rotation)?;
    }
    write_air_times(&tx, &old.air_times, &new.air_times)?;
    if old.filler != new.filler {
        write_kv(&tx, KV_FILLER, &new.filler)?;
    }
    tx.commit().map_err(db_error)
}

fn upsert_track(tx: &Transaction<'_>, track: &Track) -> Result<(), StoreError> {
    let text = doc(track)?;
    tx.prepare_cached(
        "INSERT INTO tracks (id, path, doc, category) VALUES (?1, ?2, ?3, ?4) \
         ON CONFLICT(id) DO UPDATE SET path = excluded.path, doc = excluded.doc, \
         category = excluded.category",
    )
    .and_then(|mut s| {
        s.execute(params![
            key(track.id.0),
            track.path.to_string_lossy(),
            text,
            track.category.map(|c| key(c.0))
        ])
    })
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

    #[test]
    fn id_text_cuts_long_text_and_blobs() {
        let long = "x".repeat(500);
        let text = id_text(&SqlValue::Text(long.clone()));
        assert!(text.chars().count() < 80, "{text}");
        assert!(text.contains('…'));
        assert!(id_text(&SqlValue::Blob(long.into_bytes())).chars().count() < 80);
        assert_eq!(id_text(&SqlValue::Text("ab".into())), "\"ab\"");
        assert_eq!(id_text(&SqlValue::Integer(7)), "Integer(7)");
    }

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

    #[test]
    fn write_changes_writes_only_the_changed_category_row() {
        let dir = tempfile::tempdir().unwrap();
        let mut db = Db::open(&dir.path().join("fauste.db")).unwrap();
        let cat = |id: u64, name: &str| Category {
            id: CategoryId(id),
            parent: None,
            name: name.into(),
            color: None,
            kind: fp_model::TrackKind::Music,
            watched_folder: None,
        };
        let mut old = snapshot();
        old.categories = vec![cat(1, "A"), cat(2, "B"), cat(3, "C")];
        write_full(&mut db, &old).unwrap();

        let mut new = old.clone();
        new.categories[1].name = "Bee".into();
        let before = db.conn.total_changes();
        write_changes(&mut db, &old, &new).unwrap();
        assert_eq!(db.conn.total_changes() - before, 1);
        assert_eq!(read(&db).0, new);
    }
}
