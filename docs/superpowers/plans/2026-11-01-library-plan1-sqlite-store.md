# Track library, plan 1: SQLite store — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace `playlists.json`, `carts.json` and `session.json` with one SQLite database written incrementally, import the old files once, and make a large library cost nothing on paths that do not touch it. No visible change otherwise.

**Architecture:** `fp-store` gains a `db` module (schema, migrations, open with fallback, snapshot read/write, backups) behind the existing `Store` type; `Store::load` still returns `LoadedState` and a new `Store::save_state` replaces the three JSON saves. The services thread keeps owning the `Store`. `Library` moves to shared copy-on-write storage (`imbl::OrdMap<TrackId, Arc<Track>>`), which makes `AppState` clones cheap and gives the store an O(changed) diff.

**Tech Stack:** Rust, `rusqlite` (`bundled`, `backup` features), `imbl`, serde_json, tempfile.

**Spec:** `docs/superpowers/specs/2026-11-01-track-library-design.md` §4, §5.2 (LB1–LB8, LB7b–LB7d, LB12b), §13 (`tuning.backup_interval_secs`), §14.

## Global Constraints

- All code, comments, docs and commit messages in English; Conventional Commits with scopes `store`, `model`, `app`, `remote`, `docs`, `deps`.
- No `unsafe`; no `unwrap`/`expect`/`panic` outside tests; prefer `get` over indexing.
- Never crash on bad data: a bad database, row or JSON file degrades to defaults with a warning (spec LB7, LB7c; CLAUDE.md rule 9). The application always starts, and nothing plays after a start (rule 10).
- The UI and the audio callback never touch the database (LB3).
- `tuning.backup_interval_secs`: default 300, range 30–86 400, lenient loading.
- Database file `data/fauste.db`; backups `fauste.db.bak1` … `fauste.db.bak<limits.backup_count>`; quarantine names `fauste.db.corrupt-<stamp>` and `fauste.db.newer-<stamp>` (with their `-wal` and `-shm`).
- Imported JSON files are renamed `<name>.imported` (and `<name>.bakN` → `<name>.bakN.imported`).
- `config.json`, `window.ron`, the analysis cache, M3U/PLS and cart page files are unchanged.
- One integration-test binary per crate: new test files go in `crates/<crate>/tests/it/` with a `mod` line in `tests/it/main.rs`.
- Commit only with `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` green. Run `cargo deny check` after the dependency task.

## Review Focus

1. **The first start after the update on a store installation** (JSON files present, no database): everything comes across, the JSON files end as `*.imported`, and a second start does not import again. → Task 7 tests `import_moves_every_document_and_renames_the_files` and `a_second_start_does_not_import_again`.
2. **A crash in the middle of a save:** the database holds the previous save or the new one, never a mix. → Task 5 test `a_failed_save_leaves_the_previous_state`.
3. **A crash in the middle of the import:** the next start imports again from the untouched JSON files. → Task 7 test `an_interrupted_import_is_retried`.
4. **A downgrade then an upgrade again** (an older build wrote JSON after the `*.imported` rename; the database exists): the database wins and the new JSON files are left alone (LB7d). → Task 7 test `an_existing_database_is_never_overwritten_from_json`.
5. **A database from a newer version, or a corrupt one with a good backup:** the newer one is kept aside untouched, the corrupt one quarantined, the backup used. → Task 4 tests `a_newer_database_is_kept_aside`, `a_corrupt_database_falls_back_to_its_newest_backup`.

---

## File Structure

| File | Responsibility |
|---|---|
| `crates/fp-model/src/track.rs` | `Library` on `imbl::OrdMap<TrackId, Arc<Track>>`; `shares_storage`, `changes_since` |
| `crates/fp-model/src/config.rs` | `Tuning::backup_interval_secs` |
| `crates/fp-store/src/db/mod.rs` | `Db` (open, schema, migrations, quick check), quarantine and fallback |
| `crates/fp-store/src/db/schema.rs` | `MIGRATIONS: &[&str]` (SQL), `SCHEMA_VERSION` |
| `crates/fp-store/src/db/snapshot.rs` | `Snapshot` (what is saved), `read`, `write_full`, `write_changes` |
| `crates/fp-store/src/db/backup.rs` | online backup with rotation |
| `crates/fp-store/src/legacy.rs` | today's JSON loading, kept for the import and the fuzz target |
| `crates/fp-store/src/store.rs` | `Store::load` (database, fallback, import), `save_state`, `save_config`; the restore shared by both paths |
| `crates/fp-app/src/services.rs` | autosave through `save_state`; library walks skipped when the library is unchanged |
| `crates/fp-remote/src/events.rs` | track events skipped when the library is unchanged |
| `crates/fp-app/examples/demo_session.rs`, `scripts/site/screenshots.sh` | write and read the database |
| `docs/technical/persistence.md`, `docs/technical/testing.md`, `docs/user/data-and-backups.md` | documentation |

---

### Task 1: Library on shared copy-on-write storage (LB12b, model part)

**Files:**
- Modify: `Cargo.toml` (workspace dependency `imbl`), `crates/fp-model/Cargo.toml`, `crates/fp-model/src/track.rs:608-670`
- Test: `crates/fp-model/tests/it/library_storage.rs` (new, with its `mod` line)

**Interfaces:**
- Produces:
  - `Library` keeps its public methods (`insert`, `get`, `get_mut`, `remove`, `iter`, `iter_mut`, `len`, `is_playable`, `max_raw_id`, serde as a JSON array).
  - `Library::shares_storage(&self, other: &Library) -> bool` — true when both are clones with no mutation since (`OrdMap::ptr_eq`).
  - `Library::changes_since(&self, old: &Library) -> LibraryChanges<'_>` with `pub struct LibraryChanges<'a> { pub upserted: Vec<&'a Track>, pub removed: Vec<TrackId> }`; tracks whose `Arc` is shared with `old` are skipped without comparing them.

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn a_clone_shares_storage_until_a_track_changes() {
    let lib = library_of(20_000);
    let mut copy = lib.clone();
    assert!(copy.shares_storage(&lib));
    copy.get_mut(TrackId(5)).unwrap().title = "x".into();
    assert!(!copy.shares_storage(&lib));
}

#[test]
fn changes_since_lists_only_what_changed() {
    let old = library_of(1_000);
    let mut new = old.clone();
    new.get_mut(TrackId(7)).unwrap().title = "edited".into();
    new.remove(TrackId(9));
    new.insert(Track::new(TrackId(5_000), "/m/new.flac".into()));
    let c = new.changes_since(&old);
    let mut ids: Vec<u64> = c.upserted.iter().map(|t| t.id.0).collect();
    ids.sort();
    assert_eq!(ids, vec![7, 5_000]);
    assert_eq!(c.removed, vec![TrackId(9)]);
}

#[test]
fn commands_that_do_not_touch_tracks_keep_the_library_shared() {
    // A restored state with 2 players, a playlist of 3 analysed tracks and a cart.
    let (state, p1, _) = state_with_tracks(3);
    for command in [
        Command::SetVolume(p1, 0.5),
        Command::Play(p1),
        Command::Pause(p1),
        Command::Stop(p1),
        Command::SetMode(p1, PlayMode::Single),
    ] {
        let mut s = state.clone();
        let _ = fp_model::apply(&mut s, command.clone());
        assert!(s.library.shares_storage(&state.library), "{command:?}");
    }
}
```

`library_of(n)` and `state_with_tracks(n)` are helpers in the test file (use the existing `fp-model` test support if one fits). Adapt the command names to the real `Command` variants.

- [ ] **Step 2: Run them and watch them fail**

Run: `cargo test -p fp-model --test it library_storage::`
Expected: compile errors (`shares_storage`, `changes_since` not found).

- [ ] **Step 3: Implement**

Add `imbl` to the workspace (latest release, MPL-2.0 is allowed by `deny.toml`). `Library { tracks: imbl::OrdMap<TrackId, Arc<Track>> }`; `get_mut` returns `Arc::make_mut`. Keep `iter_mut`, but replace every non-test call that does not need to change every track with a loop over ids that calls `get_mut` only where a track changes (grep `library.iter_mut`); the third test catches the rest. `changes_since` uses `OrdMap::diff` and skips pairs whose values are `Arc::ptr_eq`.

- [ ] **Step 4: Run the tests and the whole `fp-model` suite**

Run: `cargo test -p fp-model`
Expected: PASS.

- [ ] **Step 5: Commit**

`git commit -m "perf(model): share the library between state snapshots"`

---

### Task 2: Skip whole-library walks when the library is unchanged (LB12b, services and remote)

**Files:**
- Modify: `crates/fp-app/src/services.rs` (the library walks around lines 1117, 1368 and 1391), `crates/fp-remote/src/events.rs:120`
- Test: `crates/fp-app/tests/it/services.rs`, `crates/fp-remote/tests/it/events.rs`

**Interfaces:**
- Consumes: `Library::shares_storage`, `Library::changes_since` (Task 1).
- Produces: `Services::library_walks(&self) -> u64` behind the `test-hooks` feature of `fp-app` (the number of steps that walked the whole library).

- [ ] **Step 1: Write the failing tests**

```rust
// services.rs (it): after the first step, steps over models whose library
// shares storage with the last walked one do not walk it again.
#[test]
fn an_unchanged_library_is_not_walked_again() {
    let mut h = services_with_tracks(500);   // existing support, adapted
    h.step();
    let walks = h.services.library_walks();
    h.send(Command::SetVolume(h.player(0), 0.3));
    h.step();
    h.step();
    assert_eq!(h.services.library_walks(), walks);
    h.edit_title_of_first_track("x");
    h.step();
    assert_eq!(h.services.library_walks(), walks + 1);
}

// events.rs (it): a new snapshot that shares the library emits no track
// events, and one changed track emits exactly one.
#[test]
fn track_events_follow_library_changes_only() { /* two snapshots: shared → 0 Track events; one title edited → 1 */ }
```

- [ ] **Step 2: Run them and watch them fail**

Run: `cargo test -p fp-app --features test-hooks --test it services::an_unchanged_library_is_not_walked_again` and `cargo test -p fp-remote --test it events::track_events_follow_library_changes_only`
Expected: FAIL (`library_walks` missing; the events test may already pass for the counts, in which case keep it as a guard and note it in the ledger).

- [ ] **Step 3: Implement**

Services keep the last walked `Library` clone (O(1)) and skip the per-step walks when `shares_storage` is true, unless something else those walks depend on changed (analysis results arriving, the forced set, the analyse-outdated flag): keep that list explicit next to the gate. Remote events compute `changed_tracks` from `new.library.changes_since(&old.library)`.

- [ ] **Step 4: Run the crates' suites**

Run: `cargo test -p fp-app --features test-hooks && cargo test -p fp-remote`
Expected: PASS.

- [ ] **Step 5: Commit**

`git commit -m "perf(app): skip library walks while the library is unchanged"`

---

### Task 3: Dependencies, `Tuning::backup_interval_secs` and the database schema (LB4, LB5, LB8)

**Files:**
- Modify: `Cargo.toml`, `crates/fp-store/Cargo.toml` (`rusqlite = { version = "<latest>", features = ["bundled", "backup"] }`), `crates/fp-model/src/config.rs` (`Tuning`, its `Default`, `validate`)
- Create: `crates/fp-store/src/db/mod.rs`, `crates/fp-store/src/db/schema.rs`
- Test: `crates/fp-store/tests/it/db_schema.rs` (new), `crates/fp-model/src/config.rs` unit tests

**Interfaces:**
- Produces:
  - `pub const SCHEMA_VERSION: u32 = 1;`
  - `pub struct Db { conn: rusqlite::Connection, path: PathBuf }`
  - `Db::open(path: &Path) -> Result<Db, OpenError>` creates the file when missing, sets `journal_mode=WAL`, `foreign_keys=ON`, `synchronous=NORMAL`, runs `PRAGMA quick_check` (anything but `ok` is `OpenError::Corrupt`), refuses `user_version > SCHEMA_VERSION` with `OpenError::Newer(u32)`, then runs the pending migrations in one transaction each.
  - `pub enum OpenError { Io(String), Corrupt(String), Newer(u32) }`

Schema version 1 (migration 1), exactly:

```sql
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
```

`kv` keys: `ids` (`IdGen`), `cartwall_session` (`CartwallSession`), `layout` (`Layout`). `doc` columns hold the full serde JSON of the row's model value (`Track`, `PlaylistEntry`, `CartPage`, `PlayerSession`); the key columns win over the same fields in `doc` on read.

- [ ] **Step 1: Write the failing tests**

```rust
#[test] fn a_new_database_gets_schema_1_wal_and_foreign_keys() {
    let dir = tempdir().unwrap();
    let db = Db::open(&dir.path().join("fauste.db")).unwrap();
    assert_eq!(db.pragma_i64("user_version"), 1);
    assert_eq!(db.pragma_text("journal_mode"), "wal");
    assert_eq!(db.pragma_i64("foreign_keys"), 1);
}
#[test] fn a_newer_schema_is_refused() { /* set user_version = 2 by hand → Err(OpenError::Newer(2)) */ }
#[test] fn garbage_is_corrupt() { /* write 4 KiB of 0xAB → Err(OpenError::Corrupt(_)) */ }
#[test] fn an_entry_needs_its_track_at_commit() { /* insert an entry whose track does not exist, commit → Err */ }
```

and in `config.rs`: `backup_interval_defaults_to_300_and_is_clamped_to_30_86400`.

`pragma_i64` / `pragma_text` are `#[doc(hidden)] pub` helpers on `Db` for tests.

- [ ] **Step 2: Run them and watch them fail**

Run: `cargo test -p fp-store --test it db_schema:: && cargo test -p fp-model backup_interval`
Expected: compile errors.

- [ ] **Step 3: Implement** `Db::open`, the migration runner (`MIGRATIONS: &[&str]`, migration *n* takes `user_version` from *n−1* to *n*), and the config field.

- [ ] **Step 4: Run the tests, then `cargo deny check`**

Expected: PASS; no new licence or advisory errors.

- [ ] **Step 5: Commit**

`git commit -m "feat(store): SQLite database with a versioned schema"`

---

### Task 4: Open with fallback, quarantine and backups (LB4, LB6, LB7)

**Files:**
- Modify: `crates/fp-store/src/db/mod.rs`
- Create: `crates/fp-store/src/db/backup.rs`
- Test: `crates/fp-store/tests/it/db_fallback.rs` (new)

**Interfaces:**
- Consumes: `Db::open`, `OpenError` (Task 3); `backup_path` and the quarantine stamp helper in `crates/fp-store/src/atomic.rs` (reuse, do not copy).
- Produces:
  - `pub struct Opened { pub db: Db, pub source: DbSource, pub warnings: Vec<String> }`, `pub enum DbSource { Main, Backup(usize), Fresh }`.
  - `open_with_fallback(path: &Path, backups: usize) -> Opened`: main file → on `Corrupt` rename it (and `-wal`, `-shm`) to `*.corrupt-<stamp>`, on `Newer` rename to `*.newer-<stamp>`; then `bak1..=backups` in order, each **copied** to `path` and opened; else a fresh empty database. Never fails; every step leaves a warning.
  - `Db::backup_rotating(&self, backups: usize) -> Result<(), StoreError>`: online backup to `<path>.bak-tmp`, rotate `bak(N-1)`→`bakN` … `bak1`→`bak2`, rename the temporary file to `bak1`. With `backups == 0` it does nothing.

- [ ] **Step 1: Write the failing tests**

`a_corrupt_database_falls_back_to_its_newest_backup` (two valid backups with different `kv` rows; garbage main → `DbSource::Backup(1)`, the `.corrupt-` file exists, `bak1` still exists), `a_newer_database_is_kept_aside` (main at `user_version` 2 → renamed `.newer-`, its bytes unchanged, `DbSource::Fresh` when no backup), `without_anything_readable_it_starts_fresh`, `backups_rotate_up_to_the_limit` (4 backups with `backups = 3` → `bak1..bak3`, `bak1` the newest content), `zero_backups_writes_none`.

- [ ] **Step 2: Run them and watch them fail**

Run: `cargo test -p fp-store --test it db_fallback::`
Expected: compile errors.

- [ ] **Step 3: Implement** in `db/mod.rs` and `db/backup.rs` (`rusqlite::Connection::backup(DatabaseName::Main, tmp, None)`).

- [ ] **Step 4: Run the tests**

Expected: PASS.

- [ ] **Step 5: Commit**

`git commit -m "feat(store): database fallback, quarantine and rotating backups"`

---

### Task 5: Snapshots: read leniently, write in full or by changes (LB2, LB7 rows)

**Files:**
- Create: `crates/fp-store/src/db/snapshot.rs`
- Test: `crates/fp-store/tests/it/db_snapshot.rs` (new); a crate-internal unit test in `snapshot.rs` for the failed save

**Interfaces:**
- Consumes: `Db` (Task 3), `Library::changes_since` (Task 1).
- Produces:

```rust
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
    pub fn of(state: &AppState, position_of: impl Fn(PlayerId) -> f64) -> Snapshot;
}
pub fn read(db: &Db) -> (Snapshot, Vec<String>);
pub fn write_full(db: &mut Db, snap: &Snapshot) -> Result<(), StoreError>;
pub fn write_changes(db: &mut Db, old: &Snapshot, new: &Snapshot) -> Result<(), StoreError>;
```

`Snapshot::of` uses `state.sessions(position_of)`, `state.cartwall.session()`, `state.layout`, `state.cartwall.pages`. Each write is one transaction. `write_changes`: tracks from `changes_since`; a playlist that differs from its old self (by id, `PartialEq`) has its row upserted and its entries deleted and reinserted; removed playlists are deleted (entries cascade); all playlist `position`s rewritten when the order changed; cart pages and players rewritten row by row when they differ; `kv` keys rewritten when they differ. `write_full` deletes every row and inserts all.

Lenient `read` (one warning per repair, never drops other rows): a track whose `doc` does not parse is `Track::new(id, path)`; an entry whose `doc` does not parse is `PlaylistEntry::new(id, track)`; a player or cart page that does not parse is dropped; `ids` and `cartwall_session` that do not parse are their defaults; `layout` goes through `fp_model::layout::lenient_layout` and the notes of `crate::layout_notes::layout_notes`, as `parse_session` does today.

- [ ] **Step 1: Write the failing tests**

```rust
#[test] fn a_snapshot_round_trips() { /* rich snapshot (played marks, repeat, stop_after, pending start, layout, carts with colours) → write_full → read == snapshot, no warnings */ }
#[test] fn changes_write_only_what_changed() {
    // write_full(A); B = A with one track's cue-in edited; write_changes(A, B);
    // read == B, and `total_changes()` of the transaction is 1 row.
}
#[test] fn removing_a_playlist_removes_its_entries() { /* … */ }
#[test] fn a_bad_track_row_loads_as_a_bare_track_with_a_warning() { /* UPDATE tracks SET doc='{' → Track::new(id, path) and 1 warning; the other tracks intact */ }
#[test] fn a_bad_player_row_is_dropped_with_a_warning() { /* … */ }
#[test] fn a_bad_layout_loads_as_the_default_with_its_notes() { /* … */ }
#[test] #[ignore = "measured; run with --release"]
fn one_marker_edit_saves_in_under_10_ms_with_20000_tracks() { /* 20 000 tracks, 100 playlists × 50 entries; time write_changes of one edit < 10 ms */ }
```

Unit test in `snapshot.rs`: `a_failed_save_leaves_the_previous_state` — a `#[cfg(test)]` fail point makes `write_changes` return an error after its first statement; reading afterwards gives the previous snapshot.

- [ ] **Step 2: Run them and watch them fail**

Run: `cargo test -p fp-store db_snapshot && cargo test -p fp-store --lib snapshot`
Expected: compile errors.

- [ ] **Step 3: Implement** `snapshot.rs`.

- [ ] **Step 4: Run the tests, and the measured one in release**

Run: `cargo test -p fp-store && cargo test --release -p fp-store --test it db_snapshot::one_marker_edit -- --ignored`
Expected: PASS; record the measured time in the ledger.

- [ ] **Step 5: Commit**

`git commit -m "feat(store): save state snapshots by changed rows"`

---

### Task 6: `Store` on the database (LB1, LB3, LB6)

**Files:**
- Create: `crates/fp-store/src/legacy.rs` (move `parse_session`, `parse_carts`, `load_doc` and the JSON part of today's `Store::load` there; `fuzz_documents` keeps calling them)
- Modify: `crates/fp-store/src/store.rs`, `crates/fp-store/src/paths.rs` (`database_file() -> data_dir/fauste.db`), `crates/fp-store/src/lib.rs`
- Test: rewrite `crates/fp-store/tests/it/store_roundtrip.rs`, `session_layout.rs`, `carts.rs` against the database (keep every behaviour they assert; replace file edits by row edits through a `#[doc(hidden)] pub fn Store::db_for_tests(&mut self) -> &mut Db` or a second `Db::open` on the same file)

**Interfaces:**
- Consumes: Tasks 3–5.
- Produces:
  - `Store::load(&mut self, default_playlist_name: &str) -> LoadedState` — config from JSON as today, then `open_with_fallback`, `snapshot::read`, then the same restore and repair code as today (factor it into `fn restore(config, snap: Snapshot, players_known: bool, warnings) -> LoadedState`; `players_known` is "the snapshot has player rows"). Keeps the open `Db` and the read snapshot as the last saved one.
  - `Store::save_state(&mut self, state: &AppState, position_of: impl Fn(PlayerId) -> f64, now: Instant) -> Result<(), StoreError>` — opens the database if `load` did not (a fresh home: `demo_session`), writes `write_changes` against the last saved snapshot (`write_full` when there is none), stores the new snapshot only after the commit, then runs `backup_rotating` when no backup was made yet or `tuning.backup_interval_secs` passed since the last one.
  - `Store::save_config` unchanged. `save_playlists`, `save_carts` and `save_session` are removed.

- [ ] **Step 1: Rewrite the three test files against `save_state`/`load`** (they fail to compile), adding `the_first_save_makes_a_backup_and_the_next_waits_for_the_interval` (`now`, `now + 10 s` → one backup; `now + 301 s` → two).

- [ ] **Step 2: Run them and watch them fail**

Run: `cargo test -p fp-store`
Expected: compile errors.

- [ ] **Step 3: Implement**

- [ ] **Step 4: Run `fp-store`, then the workspace (callers in `fp-app` do not compile yet: fix them in Task 8; run `cargo test -p fp-store` only here)**

Expected: `fp-store` PASS.

- [ ] **Step 5: Commit**

`git commit -m "feat(store)!: keep playlists, carts and session in the database"`

(Body: the JSON documents are no longer written; the next commit imports them.)

---

### Task 7: One-time import of the JSON documents (LB7b–LB7d)

**Files:**
- Modify: `crates/fp-store/src/store.rs`, `crates/fp-store/src/legacy.rs`
- Create: `crates/fp-store/tests/it/import.rs`, fixtures under `crates/fp-store/tests/it/fixtures/json-0.12/` (a `playlists.json` schema 2 with manual markers, loudness edit, played marks, repeat; `carts.json` with a colour; `session.json` with a layout and a pending start; plus a `playlists.json.bak1`)

**Interfaces:**
- Consumes: `legacy` loaders, `snapshot::write_full`.
- Produces: in `Store::load`, before `open_with_fallback`: when neither `fauste.db` nor any `fauste.db.bakN` exists and any of the three JSON files (or their `.bakN`) exists, read them with the legacy loaders (their warnings kept), build the `Snapshot` from the parsed documents (players straight from the session document, not recomputed), `write_full` into a new database, and only after the commit rename every JSON file and backup to `*.imported`. Then continue as a normal load. A log line says how many tracks, playlists, cart pages and players came across.

- [ ] **Step 1: Write the failing tests**

`import_moves_every_document_and_renames_the_files` (copy the fixtures into a temp home; `load` → state equals the state today's JSON load gave for the same files — compute it in the test from a second copy with `legacy::load_json`; the JSON files are gone and `*.imported` exist), `a_second_start_does_not_import_again`, `a_corrupt_playlists_json_imports_its_backup` (main file garbage, good `.bak1`), `an_interrupted_import_is_retried` (fail point: the rename step fails → the next `load` imports again and ends with one database, no duplicate rows), `an_existing_database_is_never_overwritten_from_json` (database exists, a newer `playlists.json` appears → untouched, not imported), `nothing_readable_starts_empty_with_warnings`.

- [ ] **Step 2: Run them and watch them fail**

Run: `cargo test -p fp-store --test it import::`
Expected: FAIL.

- [ ] **Step 3: Implement**

- [ ] **Step 4: Run `cargo test -p fp-store`**

Expected: PASS.

- [ ] **Step 5: Commit**

`git commit -m "feat(store): import the JSON documents once into the database"`

---

### Task 8: Services, start-up, demo session and screenshots on the database

**Files:**
- Modify: `crates/fp-app/src/services.rs:1556-1613` (`autosave`, `save_all`, `save_session`), `crates/fp-app/src/session.rs:52` (`let mut store`), `crates/fp-app/examples/demo_session.rs:135-140`, `scripts/site/screenshots.sh:244,350-361,477`, anything else `cargo check --workspace --all-targets` reports (Android: `crates/fp-mobile`)
- Test: `crates/fp-app/tests/it/services.rs`, `crates/fp-app/tests/it/app_session.rs`

**Interfaces:**
- Consumes: `Store::load`, `Store::save_state`, `Store::save_config`.
- Produces: `save_all` = `save_config` + `save_state`; the playing-session path calls `save_state` too (the diff writes only the players' rows). Errors are logged as today and retried on the next step.

- [ ] **Step 1: Adapt the services and session tests** that read `playlists.json`/`session.json` to read the database (`Db::open` on `paths.database_file()` and `snapshot::read`), and add `while_playing_only_the_players_rows_are_written` (two autosaves while playing; the second changes `players` rows only — check `tracks` rows' `doc` byte-equal).

- [ ] **Step 2: Run them and watch them fail**

Run: `cargo test -p fp-app --test it services:: app_session::`
Expected: FAIL / compile errors.

- [ ] **Step 3: Implement** the callers. In `screenshots.sh`, read player ids with `sqlite3`-free Python (`import sqlite3`; `SELECT id FROM players ORDER BY position`) and `saved_measured` with `SELECT id, json_extract(doc, '$.loudness') FROM tracks`.

- [ ] **Step 4: Full verification**

Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo deny check`
Then: `scripts/site/screenshots.sh --out target/shots-check` and `scripts/site/compare-screenshots.sh target/shots-check` — expect no image beyond the masks to differ.
Expected: all green.

- [ ] **Step 5: Commit**

`git commit -m "feat(app): save the state through the database"`

---

### Task 9: Documentation and the Android check

**Files:**
- Modify: `docs/technical/persistence.md` (files table, writing, loading, backups, import; remove the JSON documents' sections that no longer apply, keep `config.json`), `docs/technical/testing.md` (the `store_documents` fuzz row: now the import parsers), `docs/user/data-and-backups.md` (the files, backups, what an update does with the old files), `docs/technical/architecture.md` if it names the documents, `README.md` if it does.

- [ ] **Step 1: Update the docs** so every statement about the data files matches Tasks 3–8 (grep `playlists.json`, `session.json`, `carts.json` across `docs/user`, `docs/technical`, `README.md`, `CLAUDE.md`; `docs/i18n` is not touched).

- [ ] **Step 2: Android**

Run: `PATH=$HOME/.cargo/bin:$PATH scripts/package/android.sh release`, install on the `fauste-test` emulator, start it, add a track to a playlist, force-stop, start again: the track is there and `adb shell run-as com.sanchezfauste.FaustePlayer ls files/…/data` shows `fauste.db` (see `docs/technical/mobile.md` for the data path and the emulator notes).
Expected: builds; state survives the restart.

- [ ] **Step 3: Full verification again** (as Task 8 Step 4, without the screenshots).

- [ ] **Step 4: Commit**

`git commit -m "docs: the state lives in a SQLite database"`

---

## After the plan

Whole-branch review with `superpowers:requesting-code-review` (fresh reviewer, most capable model); fix Critical and Important findings test-first; record deviations in `.superpowers/sdd/library-plan1/progress.md` as `Ruling: <decision> — <why> — <cost if wrong>`. Then PR from `feat/library-sqlite-store` (branched from an up-to-date `master` after the spec PR merges), CI on all three OSes, merge.
