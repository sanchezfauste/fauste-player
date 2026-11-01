//! The JSON documents of earlier versions (`playlists.json`, `carts.json`,
//! `session.json`): their parsers, kept unchanged for the import into the
//! database and for the fuzz targets.

use std::path::Path;

use serde::de::DeserializeOwned;

use fp_model::{AppState, RestoreParts};

use crate::atomic::{Loaded, ParseError, load_with_fallback};
use crate::docs::{
    CARTS_MIGRATIONS, CARTS_SCHEMA, CartsDoc, PLAYLISTS_MIGRATIONS, PLAYLISTS_SCHEMA, PlaylistsDoc,
    SESSION_MIGRATIONS, SESSION_SCHEMA, SessionDoc,
};
use crate::migrate::{Migration, upgrade};
use crate::store::parse_config;

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

/// Parses `session.json`. A saved pending start that does not parse loads
/// as none (the model decides how); each one dropped is reported.
pub(crate) fn parse_session(bytes: &[u8]) -> Result<(SessionDoc, Vec<String>), ParseError> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| ParseError::Corrupt(e.to_string()))?;
    let value = upgrade(value, SESSION_SCHEMA, SESSION_MIGRATIONS)?;
    let raw: Vec<bool> = value
        .get("players")
        .and_then(serde_json::Value::as_array)
        .map(|players| {
            players
                .iter()
                .map(|p| p.get("pending_start").is_some_and(|v| !v.is_null()))
                .collect()
        })
        .unwrap_or_default();
    let layout_notes = crate::layout_notes::layout_notes(value.get("layout"));
    let doc: SessionDoc =
        serde_json::from_value(value).map_err(|e| ParseError::Corrupt(e.to_string()))?;
    let mut notes: Vec<String> = raw
        .iter()
        .zip(&doc.players)
        .filter(|(had, p)| **had && p.pending_start.is_none())
        .map(|(_, p)| {
            format!(
                "session: the saved pending start of player {:?} is not valid; ignored",
                p.id
            )
        })
        .collect();
    notes.extend(layout_notes);
    Ok((doc, notes))
}

pub(crate) fn load_doc<T: DeserializeOwned>(
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

/// Parses `carts.json`. A cart colour that does not parse loads as none
/// (the model decides how, cart colour spec CC3); each one is reported.
pub(crate) fn parse_carts(bytes: &[u8]) -> Result<(CartsDoc, Vec<String>), ParseError> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| ParseError::Corrupt(e.to_string()))?;
    let value = upgrade(value, CARTS_SCHEMA, CARTS_MIGRATIONS)?;
    let notes = unknown_cart_colors(&value);
    let doc: CartsDoc =
        serde_json::from_value(value).map_err(|e| ParseError::Corrupt(e.to_string()))?;
    Ok((doc, notes))
}

/// One note per cart whose `color` is present, not `null` and not a known
/// name.
pub(crate) fn unknown_cart_colors(value: &serde_json::Value) -> Vec<String> {
    items(value.get("pages"))
        .iter()
        .flat_map(|page| items(page.get("carts")))
        .filter_map(|cart| {
            let raw = cart.get("color").filter(|c| !c.is_null())?;
            let known = raw.as_str().and_then(fp_model::CartColor::from_name);
            known.is_none().then(|| {
                let id = cart
                    .get("id")
                    .map_or_else(|| "?".to_owned(), crate::lenient::quoted);
                format!(
                    "carts: cart {id} has an unknown colour {}; it has none",
                    crate::lenient::quoted(raw)
                )
            })
        })
        .collect()
}

/// The elements of a JSON array, or none.
fn items(value: Option<&serde_json::Value>) -> &[serde_json::Value] {
    value
        .and_then(serde_json::Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
}

/// Runs `bytes` through every document parser and the restore, as loading
/// does. For the fuzz targets only.
#[doc(hidden)]
pub fn fuzz_documents(bytes: &[u8]) {
    let mut config = parse_config(bytes).map(|(c, _)| c).unwrap_or_default();
    let _ = config.validate();
    let lists: Option<PlaylistsDoc> = parse_doc(bytes, PLAYLISTS_SCHEMA, PLAYLISTS_MIGRATIONS).ok();
    let session: Option<SessionDoc> = parse_session(bytes).ok().map(|(d, _)| d);
    let carts: Option<CartsDoc> = parse_carts(bytes).ok().map(|(d, _)| d);
    let (library, playlists, ids) = lists
        .map(|d| (d.library, d.playlists, d.ids))
        .unwrap_or_default();
    let (sessions, cartwall_session, layout) = session
        .map(|d| (d.players, d.cartwall, d.layout))
        .unwrap_or_default();
    let _ = AppState::restore(
        RestoreParts {
            config,
            library,
            playlists,
            cart_pages: carts.map(|d| d.pages).unwrap_or_default(),
            cartwall_session,
            layout,
            ids,
        },
        &sessions,
        "Main",
    );
}
