//! Helpers for the tests that save through `Store::save_state` and edit
//! rows of the database behind the store's back.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::time::Instant;

use fp_model::AppState;
use fp_store::{Db, Store};

/// Saves the state the way the application does: the configuration, then
/// the database (positions of the players at `position` seconds).
pub fn save_all(s: &mut Store, state: &AppState, position: f64) {
    s.save_config(state).unwrap();
    s.save_state(state, |_| position, Instant::now()).unwrap();
}

/// Runs `edit` on a second connection to the store's database file.
pub fn edit_db(s: &Store, edit: impl FnOnce(&mut Db)) {
    let mut db = Db::open(&s.paths().database_file()).unwrap();
    edit(&mut db);
}

/// The JSON document of a row of `table` (a `doc` column), edited by `edit`.
pub fn edit_doc(s: &Store, table: &str, id: u64, edit: impl FnOnce(&mut serde_json::Value)) {
    edit_db(s, |db| {
        let text: String = db
            .conn()
            .query_row(
                &format!("SELECT doc FROM {table} WHERE id = ?1"),
                [id as i64],
                |r| r.get(0),
            )
            .unwrap();
        let mut doc: serde_json::Value = serde_json::from_str(&text).unwrap();
        edit(&mut doc);
        db.conn()
            .execute(
                &format!("UPDATE {table} SET doc = ?1 WHERE id = ?2"),
                rusqlite::params![doc.to_string(), id as i64],
            )
            .unwrap();
    });
}

/// The JSON value stored under `key` in `kv`, edited by `edit`.
pub fn edit_kv(s: &Store, key: &str, edit: impl FnOnce(&mut serde_json::Value)) {
    edit_db(s, |db| {
        let text: String = db
            .conn()
            .query_row("SELECT value FROM kv WHERE key = ?1", [key], |r| r.get(0))
            .unwrap();
        let mut value: serde_json::Value = serde_json::from_str(&text).unwrap();
        edit(&mut value);
        db.conn()
            .execute(
                "UPDATE kv SET value = ?1 WHERE key = ?2",
                rusqlite::params![value.to_string(), key],
            )
            .unwrap();
    });
}
