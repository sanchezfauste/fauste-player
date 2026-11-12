//! The versioned schema: migration *n* takes `user_version` from *n − 1*
//! to *n*.

/// The schema version this build writes and the newest it can read.
pub const SCHEMA_VERSION: u32 = 4;

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
    // 2: the category tree (a flat list in display order) and each track's
    // category. The column wins over the same field in the track's `doc`.
    "
CREATE TABLE categories (id INTEGER PRIMARY KEY, position INTEGER NOT NULL, doc TEXT NOT NULL);
ALTER TABLE tracks ADD COLUMN category INTEGER;
CREATE INDEX tracks_by_category ON tracks(category);
CREATE INDEX tracks_by_path ON tracks(path);
",
    // 3: automation (automation spec): the music filler's rotations, and
    // when each track last aired. The default rotation is a `kv` value.
    "
CREATE TABLE rotations (id INTEGER PRIMARY KEY, position INTEGER NOT NULL, doc TEXT NOT NULL);
CREATE TABLE air_times (track INTEGER PRIMARY KEY, at INTEGER NOT NULL) WITHOUT ROWID;
",
    // 4: the schedule (automation spec §10): day templates, and the date
    // plans keyed by the date's day number from the common era. The week
    // plan is a `kv` value.
    "
CREATE TABLE day_templates (id INTEGER PRIMARY KEY, position INTEGER NOT NULL, doc TEXT NOT NULL);
CREATE TABLE date_plans (id INTEGER PRIMARY KEY, position INTEGER NOT NULL, doc TEXT NOT NULL);
",
];
