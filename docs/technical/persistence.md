# Persistence and configuration

## Files (`fp-store`)

| File | Document | Content |
|---|---|---|
| `config/config.json` | `ConfigDoc` | `schema_version`, `config` |
| `data/fauste.db` | SQLite database (`fp-store/src/db/`) | Everything that refers to tracks or playlist entries, so one change is one transaction (see [The database](#the-database)). Replaces `playlists.json`, `carts.json` and `session.json` of earlier versions, which are imported once (see [Import](#import-from-the-json-documents-importrs)) |
| `data/window.ron` | eframe's `FileStorage` (`persistence` feature, `NativeOptions::persist_window`, desktop only) | `window`: position, size, maximized and full screen; written on close and every 30 s; eframe moves a window whose monitor went away back on screen. No egui memory (`persist_egui_memory` is false); a missing or unreadable file opens the default window |

The paths come from `AppPaths::system()` (`directories::ProjectDirs` for
`org`/`Fauste`/`Fauste Player`), or from `AppPaths::under($FAUSTE_HOME)`. See
the user guide's [Data and backups](../user/data-and-backups.md) for the
concrete folders.

## One instance per data folder

- `fp-app/src/instance.rs` holds an exclusive lock on
  `<data>/instance.lock` (`File::try_lock`) for the life of the process. The
  OS releases it if the process dies.
- **A second start** (typically a playlist opened from the file manager)
  writes its playlist paths to `<data>/inbox/<pid>-<time>.txt`: it writes a
  `.part` file, then renames it. Then it exits.
- **The running instance** has an `fp-inbox` thread that collects those
  files every 500 ms, deletes them, and hands the paths to the interface,
  which imports them like any playlist file.
- **With no playlist**, the second start shows "already running".
- **Separate `FAUSTE_HOME` folders** have separate locks.

## The database

`data/fauste.db` holds the library (tracks with their manual markers, tags
and loudness), the playlists and their entries, the cart pages and the
session (players, history, pending start, the cartwall's state, the
layout), and automation's rotations, default rotation and last-aired times
(schema 3), its schedule: day templates, the week plan and date plans
(schema 4), and its as-run log (schema 5). Only `config.json` (hand-edited, read leniently) and
`window.ron` stay outside it. The model and the interface do not know about
SQL: `Store::load` returns the state the model restores from, and
`Store::save_state` takes the `AppState`.

### Schema (`db/schema.rs`)

`PRAGMA user_version` is the schema version (`SCHEMA_VERSION`, 4), and
`MIGRATIONS[n - 1]` takes it from *n − 1* to *n*, each in its own
transaction. The tables:

| Table | Content |
|---|---|
| `kv` | Key-value documents: the library's id counter and the other small values |
| `tracks` | `id`, `path`, `category` (schema 2; null: uncategorised) and `doc`: the track's JSON (tags, markers and the `analysis_version` of the automatic ones, `analyzed`, `loudness`, `loudness_pass`, `loudness_edit`, `tags_read`, and the external editor's `file_stamp`, `audio_hash` with its version, and `markers_review`; all optional, so older rows load without them). Indexes `tracks_by_path` and `tracks_by_category` (schema 2) |
| `categories` | `id`, `position` and `doc`: the category tree as a flat list in display order (`parent`, `name`, `color`, `kind`, `watched_folder`: an absolute path or absent, plan 5). `position` is the category's place in `Categories`; siblings keep their relative order |
| `playlists`, `entries` | A playlist's `position` and `name`; its entries with `playlist`, `position`, `track` and `doc` (`played_by`, `repeat`, `stop_after`). Deleting a playlist deletes its entries |
| `cart_pages` | `id`, `position`, and `doc`: the page with its carts (`color` is a lower-case name, left out when the cart has none) |
| `players` | `id`, `position`, and `doc`: playlist, current, next, `next_explicit`, mode, `stop_after_current`, position, volume, columns, history, `pending_start` |
| `rotations` | `id`, `position` and `doc` (schema 3): a rotation of the music filler (`name`, `steps`: each a `source`, `{"Category": id}` or `{"Kind": "Music"}`, with `min` and `max`). An unreadable record is dropped; a step's counts are brought into 1–99 with `min ≤ max` ([automation](automation.md)) |
| `air_times` | `track` and `at` (schema 3): when each track last aired, UTC Unix milliseconds. A row of a track not in the library is dropped on load |
| `day_templates` | `id`, `position` and `doc` (schema 4): a day template of the schedule (`name`, `blocks`: each an `id`, a `start` in seconds after local midnight and `items`, each `{"Track": {"track", "time_critical"}}`, `{"Category": {"category", "time_critical"}}` or `{"Rotation": {"rotation"}}`). An unreadable record is dropped; `Schedule::from_parts` drops a block out of the day or at a start an earlier block has, and items after a rotation ([automation](automation.md)) |
| `as_run` | `seq` (the entry's sequence number), `date` (its date's day number from the common era, indexed) and `doc` (schema 5): one as-run entry (`seq`, `date`, `scheduled`, `started`, `track`, `item`, `source` `Schedule` or `Filler`, `outcome` `Aired` or `Missed`; [automation](automation.md)). Appended only (`INSERT OR IGNORE`), never rewritten; `write_full` does not clear it. Rows dated before `as_run_keep_days` ago are deleted when a save's cutoff day changes. Loading reads only `MAX(seq) + 1`; `read_as_run` reads a range and skips a damaged row with a warning |
| `date_plans` | `id` (the date's day number from the common era, `NaiveDate::num_days_from_ce`), `position` and `doc` (schema 4): what one date changes (`date`, `template`, `changes`: `Changed {block, own_time}`, `Removed(id)`, `Added(block)`). A record whose date is not its key's is dropped; a missing template is cleared |

The default rotation is the `kv` value `default_rotation` (a rotation id,
or null); one that names no rotation falls back on the first. The schedule's player is the `kv` value `automation` (its player's
session and its playlist id; the list's rows are cleared on restore, and the
running state, next and missed marks are never saved). The music filler is the `kv` value `filler`: its player's session (as a
`players` row would hold it), its playlist (an ordinary `playlists` row,
listed nowhere), its rotation, its rotation cursor and the rows it
generated. It comes back last of the players and not playing; a record
whose playlist is gone is dropped, its playlist then shows as an ordinary
one, and a new filler is made at start. The session's cartwall state and its layout (`player_shares`,
`hidden_players`, the cartwall's dock and size, `locked`, `touch_locked`,
and `cue_window`, the CUE windows' content size) are JSON values
in `kv`. A `doc` column holds the model value's JSON, and the key columns
win over the same fields inside it. Foreign keys are on.

The library window's state (`LibraryView`, `fp-model/src/library_view.rs`) is
the `library_view` row of `kv`, written with the session by
`Command::SetLibraryView`: whether the window is open, its rectangle
(`[x, y, width, height]` in points, none for the default place), minimised and
maximised, the selected item (`All`, `Uncategorised` or a category), the
expanded categories, the Whole library scope, the column widths, the sort and
the tree's width. The search text is not saved. It loads leniently
(`lenient_library_view`: a value that is not a view is the default, each field
falls back on its own) and is repaired against the categories on restore
(`LibraryView::repaired`: an unknown selection becomes `All`, unknown expanded
ids and unusable sizes are dropped).

Schema 2 (track library, plan 2) adds the `categories` table, the
`tracks.category` column and the two track indexes. A schema 1 database
migrates in place: every track is uncategorised and nothing is lost. A build
that only knows schema 1 sees a `user_version` above its own, so it sets the
file aside as `.newer-<stamp>` (see below).

Schema 3 (automation, plan 1) adds the `rotations` and `air_times` tables;
the default rotation is the `kv` value `default_rotation`. A schema 2
database migrates in place with no rotations and no air times; at every
start the app calls `rotation::ensure_default` (`crates/fp-app/src/session.rs`),
which creates the rotation named by `rotation-default-name` when there is
none. A schema 2 build sets a schema 3 file aside as `.newer-<stamp>`.

Schema 4 (automation, plan 3) adds the `day_templates` and `date_plans`
tables; the week plan is the `kv` value `week_plan` (seven template ids,
Monday first; a day naming no known template gets the first). A schema 3
database migrates in place with no templates; at every start the app calls
`schedule::ensure_default`, which creates the template named by
`schedule-template-default-name` for all seven days when there is none,
then `schedule::prune_dates` with today's local date.

Schema 5 (automation, plan 8) adds the `as_run` table. A save appends the
`Snapshot::as_run` entries from the last save's `as_run_next` on, and
deletes the rows older than `automation.as_run_keep_days` counted from the
local date (`as_run_before`). `Store::save_state` holds that cutoff to its
first one of the session plus the whole days of real (`Instant`) time
since, plus one, so a clock that jumps ahead does not delete the log
(AU40); loading resumes the model's numbering at one
past the highest saved sequence (`AsRunLog::resume_at`).

**The path rule.** The library holds one track per file (LB14). Uniqueness is
enforced by the model (`Library::track_at`, a path index; `track_for_path`
reuses the track of a path), not by a unique SQL index, so `tracks_by_path` is
a plain index. The model compares paths as given: fp-app's helper threads
canonicalise (`canonical_path`: `std::fs::canonicalize`, the path as given when
that fails, Windows verbatim prefixes simplified) where paths enter, namely the
file picks, drops, folder scans, the cart file pick, the cart page import and
the M3U/PLS import. Paths stored before schema 2 are kept as written, so a
symlinked path saved earlier is not merged with its canonical twin. Android
MediaStore paths are not canonicalised.

**Tracks stay.** A track is no longer forgotten when its last playlist entry
or cart goes (LB15). The one implicit removal (LB17c) is a track that is
uncategorised, `Missing` and used by nothing (`release_tracks`, called when the
file is found missing, when its last use goes and when it is taken out of its
category). The model cannot log, so
`release_tracks` returns `EngineAction::TrackLeftLibrary` with the track and
its path, and the engine, which executes the actions on the conductor thread,
logs `track left the library (file missing, uncategorised, unused)` with the
path and does nothing else. An explicit removal (*Remove from library*,
deleting a category) returns no such action.

**Restore.** The store reads the categories in `position` order and repairs
them (`Categories::repaired`: an unknown parent, the first category of a
cycle, or a category deeper than 32 levels, the most `library.max_category_depth`
allows, goes to the root, one warning each; the repair and the tree walks are
iterative, so no depth of damaged data can overflow the stack); a track whose category is unknown loads
uncategorised, with a warning. The `tracks.category` column wins over the same
field in `doc`. `AppState::restore` takes the categories (`RestoreParts`). Tracks that share a path (schema 1 created a new
track on every add) are merged by `merge_duplicate_paths`: the survivor is the
lowest id among the tracks with an operator edit (a manual marker, a loudness
edit other than the default, or a category), else the lowest id; playlist
entries and carts are re-pointed to it.

Two categories that watch the same folder (a hand-edited or old database):
`Categories::repaired` keeps the folder on the first category in stored order
and clears it on the others, one warning each. A `watched_folder` that is empty
or relative loads as stored, but the watch ignores it with a log line (see
[Architecture](architecture.md)).
Unreferenced tracks are kept.

A new field that loads as its default when it is missing and that older
builds ignore needs no migration, as before. A new shape is a new migration
with a test that opens a database of the old schema.

### Opening and fallback (`db/mod.rs`)

`Db::open` looks at an existing file read-only first (a leftover `-wal`
is not deleted and nothing is written to bytes that cannot be read), runs
`PRAGMA quick_check` and checks `user_version`, then switches the file to
WAL and migrates. The connection waits up to five seconds for a lock.

`open_with_fallback` never fails and never destroys data:

- A database that is not one or fails the check is renamed, with its
  `-wal` and `-shm`, to `fauste.db.corrupt-<stamp>`; one written by a
  *newer* version (`user_version` above `SCHEMA_VERSION`) is renamed to
  `fauste.db.newer-<stamp>`, so a later save never destroys it. A second
  one within the same second gets `-2`, `-3`… The side files move first and
  the main file last; if a rename fails, the ones done are undone.
- Then `fauste.db.bak1` … `.bakN` (`limits.backup_count`, 3) are tried in
  order. A backup is *copied* over the main name, never moved, and a copy
  that cannot be read in full is deleted again. A backup that turns out
  unusable is kept, not rotated away: once a database is in place (a later
  backup was restored, or a new one was created) it is renamed with its side
  files to `fauste.db.bakN.corrupt-<stamp>`, or `fauste.db.bakN.newer-<stamp>`
  when a newer version wrote it. The set-aside is deferred until then so that
  a session that cannot start a database leaves every backup in place.
- If nothing is usable, the application starts with an empty database and
  a warning in the log.
- A file that another connection holds locked is in use, not damaged: it
  is left alone and the session runs on an in-memory database. The start
  reports it as a load warning ("the database is in use by another process (is
  another copy running?)"), besides the log line.

### The disk is untrusted (`DiskState`)

`Store` tracks whether this session may write the file: `Unopened` (the
first save opens it), `Open`, or `Disabled`. It becomes `Disabled` when
the file could not be set aside, when a table could not be read (the file
is set aside and the backups are tried; if the replacement cannot be read
either), or when the database is busy. From then on saves go to an
in-memory stand-in and nothing in this session writes to the file or its
backups; the log says "changes are not saved this session". A start never
writes over a database it could not read.

When the read repaired or dropped rows (an unknown enum value, a
non-finite number, a document that does not parse), each is one warning,
the other rows are unaffected, and the repaired state is written back.

### Writing (`db/snapshot.rs`)

A `Snapshot` is the part of the `AppState` that is saved. The store keeps
the last one it wrote, and `write_changes` writes only the rows that differ
from it, in one transaction, so a marker edit updates one track's row.
`write_full` runs only when there is no baseline: an unopened store, a repair
at load that could not be written back, or a disabled disk. After a load the
baseline is the snapshot just read, and a failed save keeps the old baseline,
so the next save writes the difference. `Store::load` (import, check, read)
runs on the start-up thread before the interface loop; every save runs on the
services thread. Neither runs on the interface or the audio thread while
playing. A failed save leaves the state dirty and the services thread retries
it at its next autosave step.

A database file that is missing while a backup exists is not recreated: the
open restores the first usable backup (with a warning), so the next backup
rotation cannot push the real backups out. A track row that cannot be read is
dropped together with the playlist entries that pointed to it (one warning per
playlist), so the repaired snapshot is consistent when it is written back.

### Backups (`db/backup.rs`)

After a successful save, at most once every `tuning.backup_interval_secs`
(300), `backup_rotating` copies the database with SQLite's online backup
into `fauste.db.bak-tmp`, then rotates `.bak1` → … → `.bakN` and renames
the temporary file to `.bak1`. A failure leaves the existing backups as they
were. A temporary file left by a crash is not a backup and is deleted. An
in-memory stand-in never rotates, so the real backups are not pushed out.

### Import from the JSON documents (`import.rs`)

Versions before the database kept `playlists.json`, `carts.json` and
`session.json` (and `.bak1`…`.bakN` of each) in `data/`. They are no longer
read or written once the database exists. When `fauste.db` does not exist
(nor a `fauste.db.bakN`) and any of those files does, the start imports
them once:

1. The documents are read with the earlier loaders (their `.bakN` fallback,
   migrations and lenient parsing, with the same warnings).
2. The snapshot is written to `fauste.db.importing`, a database of its own,
   after deleting any leftover of an earlier attempt, and folded into one
   file (`wal_checkpoint(TRUNCATE)`).
3. The marker `fauste.db.import-pending` is created. It lists the JSON files
   this import consumed, one name per line. Then `fauste.db.importing` is
   renamed to `fauste.db`: from here the import is committed.
4. The consumed files are renamed to `*.imported` (`playlists.json.imported`,
   `playlists.json.bak1.imported`…), the directory is synced and the marker
   is removed. The `.imported` files are kept so that a downgrade can find
   them by renaming them back (see below).

Recovery after an interruption:

- **A marker next to `fauste.db`:** the commit happened. The next start only
  finishes step 4, for the files the marker names (a JSON file that appeared
  later is not touched).
- **A marker and no `fauste.db`:** the commit never happened. The marker is
  deleted and the import starts again, from scratch.
- **An existing `fauste.db` (or `.bakN`) without a marker always wins.** JSON
  files that turn up later are never imported.
- **A JSON file that is left unread blocks the import.** If a file of a
  document exists but the loader neither read nor quarantined it (it is
  larger than `limits.max_state_file_bytes`, written by a newer version, or
  an I/O error), importing the rest would leave it behind a database that
  does not hold it. Nothing is imported or saved in that session (the state
  comes from what was read, but `DiskState` stays disabled), the log names
  the files, and the import is tried again at the next start.
- **A failure writing the database** has the same effect: the temporary file
  and the marker are removed, the JSON files stay, the session runs without
  saving and the next start retries.
- **Nothing readable** at all starts empty, as the fallback does.

Downgrade: the older version reads the JSON files again, so the operator
closes the application, moves `fauste.db`, `-wal`, `-shm` and every
`fauste.db.bak*` out of the data folder (never deletes them), removes
`.imported` from the old names and starts the older version. What was saved
since the update stays only in the moved copy. Upgrading again imports the
JSON files again, but only while no `fauste.db` or `fauste.db.bakN` is in the
data folder, because an existing database or backup always wins. Deleting
`fauste.db` alone is not enough: a left-over `.bakN` would be restored and
the JSON files ignored.

On restore, a playlist or entry whose id is already taken (a hand-edited
document) gets a new id; the first holder keeps its own
(`Playlists::normalize`, like `Cartwall::normalize` for carts). The import
applies the same rule before writing, because the database keys its rows by
id.

## Atomic files (`atomic.rs`)

`config.json` is the only file still written with
`write_atomic(path, bytes, backups)`:

1. writes `<file>.tmp` and `fsync`s it;
2. rotates `<file>` → `.bak1` → … → `.bakN` (`limits.backup_count`, 3);
3. renames the temporary file over the target;
4. `fsync`s the directory (Unix).

A rename or copy that fails with "permission denied" (on Windows, a
sharing violation while a scanner or indexer holds the file) is retried up to
five times, waiting 10 ms and doubling (`retry_locked`). The `*.imported` renames of the JSON import use it too.

`load_with_fallback` tries the file, then `.bak1`…`.bakN`. A file that
fails to parse or validate is renamed `*.corrupt-<stamp>` and kept (a
second one within the same second gets `-2`, `-3`…).
A document written by a *newer* version of the application is rejected
(this build cannot know its shape), and a copy is kept as `*.newer-<stamp>`
so a later save never destroys it. Input larger than `limits.max_state_file_bytes` is refused. If
everything fails, the store starts from defaults, and the application always
starts. The import uses it for the old documents.

`config.json` is read **leniently** (`lenient.rs`): each field is taken on its
own, so one bad value falls back to its default instead of discarding the
file. `Config::validate` then clamps every value into range and returns
warnings, which are logged. A field that was renamed is listed in
`lenient::RENAMED` (section, old name, new name): its old name is read into
the new one when the new one is absent (`players.fade_ms` →
`players.crossfade_ms`). A rename needs no schema upgrade, so an older
version can still read the file. It ignores the new name and uses its
default.

## Playlist and cart page files (`playlist_io.rs`)

- `parse_playlist` reads M3U, M3U8 and PLS. It is tolerant:
  - `#EXTINF` hints are used (the title starts after the first comma outside
    quotes, so quoted attributes may hold commas); comments and unknown lines
    are ignored;
  - BOM and CRLF are handled;
  - text is UTF-8, with a Windows-1252 fallback for M3U;
  - `file://` URLs are percent-decoded, as UTF-8 or, when that fails, as
    Windows-1252 (older players encode "ú" as `%FA`);
  - PLS entries are keyed by their number as written, so `File1` and
    `File01` are two entries, each with its own `Title…` and `Length…`;
  - relative paths (with `/` or `\\` separators) are resolved against the
    playlist's folder; absolute paths from another OS (`C:\\…` on Linux) are
    kept as written and show as unavailable;
  - streams are skipped and counted.

  Input is capped at `limits.max_playlist_file_bytes`, and `read_bounded`
  refuses a longer file after reading one byte past the limit, without
  loading it whole. `write_m3u8` writes `#EXTINF` and the paths; a relative
  path starting with `#` is written as `./#…` so it is not read back as a
  comment.
- Cart pages use a versioned JSON format (`"format": "fauste-cart-page"`,
  `"version": 1`), with 1-based positions, relative files resolved against
  the file's folder, and the grid clamped to the limits. Each cart may carry
  `color` (a lower-case name, written only when set; an unknown one imports as
  none, reported through `parse_cart_page_noting`'s notes, which the import
  logs). The version stays 1: older readers ignore the field.
- All parsers are fuzzed (see [Testing](testing.md)).

## Migrations

The database has numbered migrations (see [The database](#the-database)).
`config.json` has a `CONFIG_SCHEMA` constant: when its shape changes, bump
it and append a `vN → vN+1` function in `migrate.rs`, with a test that loads
a fixture of the old shape. The old documents keep their `*_SCHEMA`
constants and migrations, because the import reads them.
A new field that loads as its default when it is missing, and that older
builds ignore, needs no bump.

## Autosave and restore

- The services thread watches `Telemetry::model_version`. It saves the
  config and the database `tuning.save_debounce_ms` (1 s) after the first
  unsaved change. The version is read *before* the snapshot, so a save can
  never record a version newer than the data it wrote.
- While any player plays, the session (positions) is saved at the same pace:
  one small update of the players' rows.
- On shutdown, a final save runs with the positions from the engine,
  before the conductor stops.
- On start, `AppState::restore` rebuilds players with their current entry
  **paused** at the saved position (`EngineAction::LoadPaused`). Nothing goes
  on air by itself.

## Configuration reference

Every value has a default in `fp-model/src/config.rs` and a valid range in
`Config::validate`.

### `players` (Settings → Players)

| Field | Default | Range |
|---|---|---|
| `count` | 4 | 1 … `limits.max_players` |
| `default_mode` | `Continuous` | `Single`, `Continuous` |
| `crossfade_ms` | 1000 | 50 … 10000 (Play while on air and Previous; earlier versions wrote it as `fade_ms`, which the lenient loader reads into this field) |
| `fade_stop_ms` | 2000 | 50 … 10000 (Fade stop) |
| `auto_segue` | true | |
| `use_cue_markers` | true | Players honour cue-in and cue-out; off plays 0 to the end of the file. No range; a wrong type loads as the default |
| `end_warning_secs` | 10 | 0 … 120 |
| `history_len` | 50 | 0 … 1000 (entries Previous can go back; not in Settings) |

### `analysis` (Settings → Analysis)

| Field | Default | Range |
|---|---|---|
| `trim_threshold_db` | −60 | −120 … −20 |
| `trim_margin_ms` | 20 | 0 … 1000 |
| `segue_drop_db` | 15 | 3 … 40 |
| `segue_max_secs` | 4 | 0 … 60 |
| `outro_drop_db` | 6 | 0 … 40 |
| `outro_max_secs` | 30 | 0 … 300 |
| `markers_min_duration_secs` | 60 | 0 … 3600 |
| `peak_bucket_ms` | 10 | 1 … 1000 |
| `rms_window_ms` | 50 | 5 … 1000 |
| `cover_thumb_px` | 128 | 16 … 1024 |

The fields `silence_threshold_db` and `segue_threshold_db` of earlier
versions are ignored on load with a warning (their meaning changed), like
any field this version does not have.

### `outputs` (Settings → Audio outputs; applied while running once nothing plays on the affected device)

| Field | Default | Meaning |
|---|---|---|
| `backend` | none (platform default) | backend id |
| `sample_rate` | 48000 | Hz |
| `buffer_frames` | 512 | frames per block |
| `routes[]` | empty | `{ player, main: Route?, cue: Route? }`, where `Route` is `{ backend, device, first_channel }` |
| `cartwall` | none | `{ main: Route?, cue: Route? }` for the cartwall |
| `library` | none, `default_output` per platform | `{ cue: Route?, default_output: bool }` for the library's pre-listen. `default_output` (LB46) lets the library CUE play on the default output when `cue` is empty; it defaults to true on Android and false elsewhere, has no Settings control and is config file only; an older file without it loads the platform default |
| `bit_perfect[]` | empty | `{ backend, device }` of devices played bit-perfect (exclusive access, rate follows the files) |
| `dsd_output[]` | empty | `{ backend, device, mode }` with `mode` `Pcm`, `Dop` or `Native`: what a bit-perfect device receives from a DSD track. A device not listed here, or not in `bit_perfect`, converts DSD to PCM. A device listed twice keeps its first mode (warning) |
| `dsd_mix` | `ConvertToPcm` | `ConvertToPcm` or `HoldOthers`: what happens when another source needs an output carrying DSD |
| `shared_buffer_ms` | 50 | the least a shared output (PulseAudio, PipeWire, an ALSA device that is not the hardware, WASAPI shared mode) asks for, in ms, up to the largest buffer the device reports; `buffer_frames` wins when it is longer and a device's own buffer (one that differs from `buffer_frames`) is used as it is; 0 to 500 (clamped, with a warning; 500 is `tuning.play_out_max_ms`'s default, and a stream's two blocks are held when it reopens), 0 is no floor; read when an output opens; config file only |
| `dsd_silence_ms` | 200 | DSD silence at a DSD stream's start, end and switch to PCM; 0 to 2000 (clamped, with a warning) |
| `device_overrides[]` | empty | `{ device: { backend, device }, sample_rate?, buffer_frames? }`: a device's own rate and buffer, used instead of `sample_rate` and `buffer_frames` when it opens (operator feedback 4, Q12). Same ranges as the global fields; an out-of-range value is dropped with a warning and the device uses the global one; a device listed twice keeps its first values (warning); an entry with neither is removed |

An entry in `bit_perfect`, `dsd_output` or `device_overrides` for a device no route names (no player's Main or Cue, nor the cartwall's) is removed without a warning on load (`OutputsConfig::forget_unrouted_devices`, operator feedback 4, Q12), not while the application runs, so a route edit in Settings loses nothing: a device that leaves every route starts again from the global values when it is routed again after a start. Before that, when the session file loaded, the routes of players that no longer exist are dropped (`forget_routes_of_other_players`): a removed player's id is never used again.

### `meter` (Settings → Meters; applied at once)

| Field | Default | Range | Meaning |
|---|---|---|---|
| `ballistics` | `DigitalPeak` | `DigitalPeak`, `EbuPpm`, `DinPpm`, `Vu`, `K20`, `K14`, `K12`, `Custom` | meter type and scale (IEC 60268-18, 60268-10 IIb and I, 60268-17, K-System) |
| `attack_ms` | 5 | 0 … 1000 | integration time for `Custom`, by the IEC definition: a 5 kHz tone burst that long reads 2 dB low (the EBU preset's is 8.37 ms by this definition); 0 = instant |
| `release_db_per_sec` | 11.8 | 1 … 100 | fall for `Custom` |
| `true_peak` | false | | 4× oversampled peak |
| `floor_db` | −60 | −96 … −20 | bottom of the digital scale (digital peak, custom) |
| `peak_hold_secs` | 2 | 0 … 10 | 0 turns the hold off |
| `reference_dbfs` | −18 | −30 … 0 | alignment mark |
| `warning_dbfs` | −9 | −30 … 0 | yellow from here |
| `danger_dbfs` | −3 | −30 … 0 | red from here (raised to the warning level if below it) |
| `loudness` | `ShortTerm` | `Off`, `Momentary`, `ShortTerm` | the LUFS line |
| `loudness_target_lufs` | −23 | −36 … −10 | green within ±1 LU |

### `ui`

| Field | Default | Meaning |
|---|---|---|
| `window_system` | `Auto` | `Auto`, `X11` or `Wayland`, Linux only, read at start: the window system the window opens on. `Auto` takes X11 when `DISPLAY` is set (natively or through XWayland), since winit 0.30 delivers file drops only through X11; `X11` and `Wayland` take that one when it is there. Anything else leaves the choice to winit |
| `keep_screen_on` | `true` | Mobile only (spec MB8): the screen stays on while playout is active (`fp_model::playout_active`). Ignored on desktop |
| `outputs_view` | `Basic` | `Basic` or `Advanced`: what Settings → Audio outputs shows. It changes no output setting |
| `max_fps` | 30 | 0 … 240 (the most frames a second the interface draws while something plays, fades or cues; 0 draws every display frame; input is answered at once whatever the limit; config file only) |
| `follow_current_grace_secs` | 10 | 0 … 600 (seconds after the operator uses a zoomed waveform or a playlist table before it follows what plays; 0 never follows) |
| `table_columns` | `["number", "title", "artist", "duration"]` | an ordered list of `number`, `title`, `artist`, `album`, `date`, `genre`, `duration`, `intro`, `file_name`, `comment`; unknown names are dropped when the file is read, duplicates keep their first place, and a missing `title` or `duration` is added back (`Config::validate` warns) |
| `wave_color` | `slate` | `violet`, `amber`, `cyan`, `white`, `orange`, `magenta`, `ice`, `sand`, `slate` |
| `music_dir` | none | the folder where file dialogs start |
| `language` | none (OS locale) | BCP-47 tag of a registered locale (`en-US`, `es-ES`, …) |

### `cartwall`

| Field | Default | Range |
|---|---|---|
| `default_rows` | 2 | 1 … `limits.max_cart_rows` |
| `default_cols` | 8 | 1 … `limits.max_cart_cols` |

### `shortcuts` (Settings → Keyboard shortcuts)

A list of `{ "action": …, "chord": { "key": …, "ctrl"?, "alt"?, "shift"?, "command"? } }`.
Actions are `PlayPlayer(n)`, `PausePlayer(n)`, `StopPlayer(n)`,
`FadeStopPlayer(n)`, `CuePlayer(n)`, `FireCart(n)` (cart *n* of the page
shown), `StopAllCarts`, `ToggleCartwall`, `NextCartPage` and
`PreviousCartPage`, with 1-based positions. Keys use the interface
toolkit's names (`"1"`, `"A"`, `"F1"`, `"Space"`). Validation keeps one
chord per action and one action per chord. The defaults are `1`…`9` (play
players 1–9), `F1`…`F12` (fire carts 1–12) and `Ctrl+Space` (stop all
carts).

### `midi` (Settings → MIDI)

| Field | Default | Range |
|---|---|---|
| `enabled` | false | |
| `rescan_interval_ms` | 2000 | 250 … 60000 |
| `feedback` | true | |
| `devices` | `[]` | `{ "input": port name, "output": port name or null }`: where an input's LED feedback goes |
| `bindings` | `[]` | `{ "device": port name, "trigger": { "Note": { "channel", "note" } } \| { "ControlChange": { "channel", "controller" } } \| { "PitchBend": { "channel" } }, "action": { "Button": shortcut action } \| { "Volume": n } }`; channels 0–15. Bindings to other actions, out of MIDI's range, or a volume on a note are dropped with a warning |

### `remote` (Settings → Remote)

| Field | Default | Range |
|---|---|---|
| `http.enabled` | false | |
| `http.bind` | `"127.0.0.1"` | an IPv4 or IPv6 literal, else `127.0.0.1` with a warning |
| `http.port` | 7380 | 1024 … 65535 |
| `http.token` | `""` | empty, or at least 16 characters (a shorter one is dropped with a warning) |
| `http.cors_origins` | `[]` | `http://` or `https://` origins without a path; `"*"` only with a token |
| `http.max_event_clients` | 16 | 1 … 256 |
| `http.request_timeout_ms` | 10000 | 1000 … 120000 |
| `http.max_body_bytes` | 65536 | 1024 … 1048576 |
| `osc.enabled` | false | |
| `osc.bind` | `"127.0.0.1"` | an IPv4 or IPv6 literal, else `127.0.0.1` with a warning |
| `osc.port` | 7381 | 1024 … 65535; moved off the HTTP port on the same bind |
| `osc.allowed_sources` | `["127.0.0.1/32", "::1/128"]` | addresses or CIDR subnets; invalid ones dropped with a warning |
| `osc.max_subscribers` | 16 | 1 … 256 |
| `osc.subscription_ttl_secs` | 60 | 5 … 3600 |
| `events.position_interval_ms` | 250 | 50 … 5000 |

See [Remote control API](remote-api.md).

### `loudness` (applied to sources loaded afterwards)

Loudness normalization (loudness normalization spec §4): one fixed gain
per track, decided when it is loaded. A change never touches what is
sounding and never waits: the next source loaded (the preload of each
player's next is sent again) carries the new gain. A file without the
section loads the defaults; a value that is not a number takes its
default, and one out of range is clamped, each with a warning.

| Field | Default | Range | Meaning |
|---|---|---|---|
| `normalize` | true | | normalization on or off; on for new installations, while a file that stores `false` keeps it off (loudness normalization spec D11) |
| `target_lufs` | −16 | −36 … −6 | integrated loudness every track is brought to; −16 follows the AES streaming recommendations (TD1008, 2021, superseding TD1004.1.15-10: −16 to −20 LUFS), not broadcast (R128 −23, A/85 −24) |
| `true_peak_ceiling_dbtp` | −1 | −12 … 0 | a boost may not take the measured true peak above this; it never cuts a track; −1 is the EBU R128 maximum true peak, also used by AES (−2 ahead of low-bit-rate encoders is common practice, not from the standards) |
| `max_boost_db` | 12 | 0 … 24 | the largest gain a track may get; cuts are not limited; a product choice, no standard |
| `max_trim_db` | 12 | 0 … 24 | the per-track trim is refused beyond ± this, and a stored trim beyond it (the maximum lowered since) is limited to it; a product choice, no standard |

A source on a bit-perfect device gets no gain (N8). The meter's own
`meter.loudness_target_lufs` is a separate value.

### `library` (config file only)

| Field | Default | Meaning |
|---|---|---|
| `max_category_depth` | 8 | the deepest category tree a new category or a move may build; restore repairs anything deeper than 32 levels |
| `watch_debounce_ms` | 1000 | quiet time a path of a watched folder needs after its last event before it is read (100–60000); read while the application runs |

### `automation` (Settings → Automation, desktop only)

The music filler's selection rules ([automation](automation.md)); 0 turns
a criterion off.

| Field | Default | Meaning |
|---|---|---|
| `track_separation_mins` | 180 | a track may air again once this many minutes have passed since it last aired (0–10080) |
| `track_separation_percent` | 50 | …or once this percentage of the other tracks of its step's source have aired since (0–95) |
| `filler_lookahead_mins` | 30 | the music filler keeps this many minutes of tracks queued ahead (5–240) |
| `filler_played_kept` | 50 | the music filler keeps this many played rows in its list; older ones are removed (10–1000) |
| `artist_separation_mins` | 30 | no track of an artist that aired less than this many minutes ago (0–1440; the artist compared trimmed and case-insensitively) |
| `as_run_keep_days` | 90 | the as-run log keeps this many days in the database (1–3650; config file only for now) |
| `overrides_keep_days` | 30 | the schedule's date overrides older than this many days are deleted at start-up (1–3650; config file only for now) |
| `late_tolerance_secs` | 120 | a block that started less than this many seconds ago is still next by the clock, and a time-critical item may start this late (0–1800; config file only for now) |
| `block_fade_ms` | 2000 | the music filler fades out over this many milliseconds, ending at a block's start (0–10000; config file only for now) |
| `max_fade_cut_secs` | 30 | the song the music filler fades at a block's start loses at most this many seconds (0–300; config file only for now) |
| `min_song_play_secs` | 45 | the music filler never starts a song that would play less than this many seconds before its fade at a block's start (0–600; config file only for now) |

### `limits` (config file only)

Changed while the application runs (live settings spec §6, §15): new cover
and tag limits apply to analyses started afterwards (the analysis pool and the
remote's cache reader each take one snapshot of the limits per analysis; the
cache keeps no limits of its own and the key is built from that snapshot, so
entries made under the old cover limits simply miss, nothing is analysed again, existing tags are not
trimmed again). `max_crash_reports` follows too. `max_players` and the cart
grid limits are read at start only. The others are read where they are used.

| Field | Default |
|---|---|
| `max_players` | 16 |
| `max_cover_bytes` | 20 MiB |
| `max_cover_pixels` | 8000 |
| `max_tag_chars` | 2000 (range 64 … 100000; longest tag text kept per field, in characters) |
| `max_tag_values` | 32 (range 1 … 1000; most values kept per tag field) |
| `max_state_file_bytes` | 50 MiB |
| `max_playlist_file_bytes` | 10 MiB |
| `backup_count` | 3 |
| `max_crash_reports` | 20 (crash reports written per run) |
| `max_log_file_mb` | 10 (range 1 … 1024; a day's log file takes no more lines once it holds this, after one closing line; read at start only, before the rest of the configuration, by `peek_log_limits`) |
| `max_log_files` | 14 (range 2 … 365, because the appender prunes to `max - 1` files at start and 1 would delete today's earlier file; daily log files kept; read at start only) |
| `log_repeat_summary_secs` | 60 (range 1 … 3600; a long run of identical log lines gets its "repeated N times" line at least this often; read at start only by `peek_log_limits`) |
| `max_cart_rows` | 8 |
| `max_cart_cols` | 16 |

### `tuning` (config file only)

Every value is read while the application runs (live settings spec §7): a new value applies at its next use.

| Field | Default | Meaning |
|---|---|---|
| `declick_ms` | 5 | stop ramp |
| `pause_ramp_ms` | 10 | pause/resume ramp |
| `prebuffer_secs` | 5 | ring size per source |
| `ready_threshold_ms` | 500 | buffered before a source counts as ready |
| `mixer_headroom` | 2 | slot capacity multiplier |
| `max_commands_per_block` | 256 | mixer commands drained per block |
| `schedule_lead_ms` | 200 | how early transitions are dispatched |
| `conductor_tick_ms` | 5 | conductor period |
| `watchdog_timeout_ms` | 500 | missing heartbeat that marks a bus lost |
| `watchdog_startup_grace_ms` | 5000 | grace after opening a device |
| `reconnect_interval_ms` | 2000 | device retry period |
| `device_busy_retries` | 3 | how many more times one rate change (the new rate and the way back together) tries a device that answered busy; only on an idle output (0–5) |
| `device_busy_retry_ms` | 20 | wait before each of those tries. The conductor waits meanwhile, delaying every output's planned events: at most retries × wait per rate change (60 ms by default, 200 ms at most); keep it below `schedule_lead_ms` (0–40) |
| `play_out_max_ms` | 500 | the longest a rate change on a bit-perfect output waits for the device to play what its stream still holds before closing it (bit-perfect spec BR17); the wait is what the stream reports, bounded by this so that a bogus measurement cannot hold the output silent (0–2000; 0 never waits) |
| `gain_smoothing_ms` | 20 | volume smoothing |
| `save_debounce_ms` | 1000 | autosave delay |
| `missing_recheck_ms` | 30000 | how often files not found are looked for again |
| `backup_interval_secs` | 300 | how often the library database is copied to a rotating backup (30 to 86400) |
