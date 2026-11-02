//! The versioned schema: migration *n* takes `user_version` from *n − 1*
//! to *n*.

/// The schema version this build writes and the newest it can read.
pub const SCHEMA_VERSION: u32 = 1;

/// The migrations, in order; `MIGRATIONS[n - 1]` is migration *n*.
pub(crate) const MIGRATIONS: &[&str] = &[
    // 1: the key-value documents, the track library, playlists and their
    // entries, cart pages and players. `doc` columns hold the full JSON of
    // the model value; the key columns win over the same fields in `doc`.
    "
CREATE TABLE kv (key TEXT PRIMARY KEY, value TEXT NOT NULL) WITHOUT ROWID;
CREATE TABLE tracks (id INTEGER PRIMARY KEY, path TEXT NOT NULL, doc TEXT NOT NULL);
CREATE TABLE playlists (id INTEGER PRIMARY KEY, position INTEGER NOT NULL, name TEXT NOT NULL);
CREATE TABLE entries (
  id INTEGER PRIMARY KEY,
  playlist INTEGER NOT NULL REFERENCES playlists(id) ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
  position INTEGER NOT NULL,
  track INTEGER NOT NULL REFERENCES tracks(id) DEFERRABLE INITIALLY DEFERRED,
  doc TEXT NOT NULL
);
CREATE INDEX entries_by_playlist ON entries(playlist, position);
CREATE INDEX entries_by_track ON entries(track);
CREATE TABLE cart_pages (id INTEGER PRIMARY KEY, position INTEGER NOT NULL, doc TEXT NOT NULL);
CREATE TABLE players (id INTEGER PRIMARY KEY, position INTEGER NOT NULL, doc TEXT NOT NULL);
",
];
