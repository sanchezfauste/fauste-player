# Track library — Design Spec

- **Date:** 2026-11-01.
- **Status:** Approved (2026-11-01).
- **Amends:** [the main spec](2026-09-25-fauste-player-design.md) §1 non-goals
  ("a music library browser or database": now a goal) and its persistence
  section; [persistence](../../technical/persistence.md) (the JSON state
  documents are replaced); [the on-air pre-listen spec](2026-10-15-cue-on-air-output-design.md)
  (a new Cue holder); [the phone layout spec](2026-10-28-phone-layout-design.md)
  PL19b (the Music screen becomes the Library tab's Device music, in plan
  6 only).
- **Numbering:** rules are `LB<n>`. Each rule is testable and gets at least
  one test (§14).
- **Line numbers** are those of `master` at `0c3d7f34`.

---

## 1. Goal

A station keeps its audio in families: current music, older music, station
IDs, spots, and each show's own openers, beds and closers. Today Fauste
Player has no place for them. Tracks exist only while a playlist or a cart
points at them (`forget_unreferenced`, `crates/fp-model/src/reducer.rs:1239`),
and their markers and loudness edits are made on a player.

The **library** is that place:

1. tracks are organised in a **tree of categories** (with subcategories),
   one category per track;
2. a category can **watch a folder**: files added there join it, files
   removed are noticed;
3. a **text search** finds tracks, by default inside the selected category;
4. every track can be **pre-listened** from the library, with a transport
   like a player's, and its **markers, loudness edit and tags** edited there,
   exactly as on a player;
5. tracks are **sent to a playlist or a cart** by drag and drop or a menu.

The state of the whole application moves to **SQLite** first (§4): with
thousands of tracks, rewriting a JSON document on every marker edit no longer
makes sense.

### Why now

The next phase is **music automation**: templates (hour "clocks") made of
slots such as "station ID, 2 × current, 1 × gold", filled automatically from
categories. That phase is *not* in this spec, but it needs categories,
a durable library and a database to query; this spec builds them and keeps
the door open (§12).

## 2. Scope

**Goals:** §4–§11 on the desktop (Linux, Windows, macOS). The shared core
(store, model, search) works on Android from plan 1 on; the phone's own
library screen is plan 6, designed then (§11).

**Non-goals (later, or never in this spec):**

- Music automation, clocks, rotation and separation rules (next phase).
- A library window on a second monitor (a separate OS window). The library
  is a floating window inside the main window.
- Validity dates (start/end) for spots, and an as-run log.
- The library in the remote API and on MIDI.
- Watched folders on Android (§11).
- Copying or moving files: the library points at files where they are.
  Moving or renaming them is the operator's business; such tracks show as
  missing, as they do today.

**Anti-goals (a failure even if it works):**

- The library slowing what is on air: a big import or a folder rescan may
  never take decoding or analysis time from the players and carts, nor
  block the interface (main spec rules 5 and 8).
- Anything going on air because of the library: a pre-listen reaches an
  on-air output only through the existing confirmation (§8.3).
- A lost marker edit after a crash: edits are durable within the save
  debounce, as today.

## 3. Terms

- **Library:** every track the application knows, with its markers,
  loudness edit, tags and category.
- **Category:** a named node in a tree (a **subcategory** has a parent).
  "Category" is the usual term in radio programming for a family of audio
  that a schedule draws from.
- **Uncategorised:** not a category but the view of the tracks that have
  none.
- **Watched folder:** a folder bound to a category (§6).
- **Library CUE:** the library's pre-listen holder (§8).
- **Kind:** the existing `TrackKind` (`Music`, `Jingle`, `Effect`, `Ad`,
  `Voice`, `crates/fp-model/src/track.rs:13`): what a track is used for on
  air. It is distinct from its category. The list is **fixed** (maintainer,
  2026-11-01): a kind exists only because the program treats it differently,
  and that behaviour is code; operators cannot define their own. Today no
  kind changes any behaviour; the automation phase gives them their meaning
  (and may extend the fixed list). In this project the kind is visible and
  editable (a column, the category's kind, the track menu) and does nothing
  else.

## 4. Persistence: SQLite (plan 1)

### 4.1 What goes where

Everything that refers to tracks or playlist entries moves into **one
database**, so a change is one transaction and a crash never leaves a
session pointing at an entry that is gone:

| Data | Store |
|---|---|
| Library (tracks, markers, tags, loudness, kind, category), categories, watched folders | `data/fauste.db` |
| Playlists and entries | `data/fauste.db` |
| Cart pages and carts | `data/fauste.db` |
| Session: player state, history, pending start, cartwall session, layout (layout as a JSON value in one row), the library window's state (§7.1) | `data/fauste.db` |
| Configuration | `config/config.json`, unchanged: hand-edited, lenient loading |
| Window geometry | `data/window.ron` (eframe), unchanged |
| Analysis cache (peaks, results) | its files, unchanged: large and rebuildable |
| Logs, crash reports, `inbox/`, `instance.lock` | unchanged |
| Playlist (M3U/PLS) and cart page files | unchanged: exchange formats |

`playlists.json`, `carts.json` and `session.json` are no longer read or
written.

### 4.2 Rules

- **LB1.** `fp-store` keeps its interface: `Store::load` still returns the
  state the model restores from (`RestoreParts`), and saving still takes the
  `AppState`. `fp-model` and the interface do not learn about SQL.
- **LB2.** A save writes only what changed since the last save: a marker
  edit updates one track's rows, not the library. *How* the store knows what
  changed (a version per entity, dirty sets from the model, a diff of
  snapshots) is the builder's call; the result must be measured: with 20 000
  tracks and 100 playlists, saving one marker edit costs well under 10 ms on
  the store thread.
- **LB3.** All database work runs on a thread that is neither the UI nor the
  audio thread (the services thread, or a store thread it owns). The UI never
  waits on the database (rule 8).
- **LB4.** The schema is versioned with `PRAGMA user_version` and numbered
  migrations from version 1. A database written by a newer version is not
  opened: it is copied to `fauste.db.newer-<stamp>`, and the application
  starts empty, as `load_with_fallback` does for JSON today.
- **LB5.** WAL journal mode; foreign keys on; the database is opened by one
  process only (the existing `instance.lock`).
- **LB6.** Backups: after a successful save at most once every
  `tuning.backup_interval_secs` (config, §13), the SQLite online backup API
  writes `fauste.db.bak1`, rotating up to `limits.backup_count` copies.
- **LB7.** Load with fallback: a database that fails `PRAGMA quick_check`
  or cannot be read is renamed `fauste.db.corrupt-<stamp>` (with its `-wal`
  and `-shm`), the newest readable backup is used, and when nothing works the
  application starts empty with a warning in the log. It always starts
  (rules 9 and 10). Rows that do not parse (an unknown enum value, a
  non-finite number) degrade to their default with a warning, like today's
  lenient loading; one bad row never discards the rest.
- **LB8.** `rusqlite` with the `bundled` SQLite, so every platform (Android
  included) has the same SQLite and FTS5 if the builder uses it. It goes
  through `cargo deny check`.

### 4.3 Session saves while playing

Today the session is rewritten every `save_debounce_ms` while a player
plays (`autosave`, `crates/fp-app/src/services.rs:1556`) to keep positions.
It becomes one small update of the players' rows on the same schedule.

### 4.4 Behaviour visible to the operator

None in plan 1, apart from the files in the data folder (the guide's
*Data and backups* page and the release notes say what happened to the old
files). Plan 1 is a pure
refactor: the whole suite passes unchanged except the tests that read or
write the JSON documents, which are rewritten against the database.

### 4.5 Import from the JSON documents

The application is on the Microsoft Store and the Snap Store, which update
by themselves, so existing installations must keep their data (maintainer,
2026-11-01).

- **LB7b.** At start, when `fauste.db` does not exist and any of
  `playlists.json`, `carts.json` or `session.json` (or their `.bakN`
  backups) does, the store reads them with today's lenient loaders
  (`load_doc`, `parse_session`, `parse_carts`, `load_with_fallback`, kept for
  this), restores the state as today, and writes it to a new database in one
  transaction. Only then are the JSON files renamed `*.imported` (with their
  backups), so a crash during the import retries it at the next start, and a
  downgrade can still find them by renaming them back.
- **LB7c.** The import never fails the start: what the old loaders would
  have dropped is dropped with the same warnings; if nothing can be read, the
  application starts empty as LB7 does. Library tracks, manual markers,
  loudness edits, tags, playlists with played marks, cart pages and the
  session (layout included) all come across.
- **LB7d.** The import runs once: an existing `fauste.db` is never
  overwritten from JSON files.

### 4.6 Everything that writes state

`examples/demo_session.rs`, `scripts/site/screenshots.sh` (scratch
`FAUSTE_HOME`), the store's fuzz entry point (`fuzz_documents`) and the
tests that build a `FAUSTE_HOME` move to the database. The screenshots must
still come out the same.

## 5. Model (plan 2)

### 5.1 Categories

```text
Category { id: CategoryId, parent: Option<CategoryId>, name, color,
           kind: TrackKind, watched_folder: Option<PathBuf>, order: u32 }
```

- `color` uses the cart colours (`cart colour spec`), so the same names and
  palette.
- `kind` is the kind a track gets when it joins the category (LB13).
- `order` is the order among siblings, set by drag and drop in the tree.

Rules (pure functions in `fp-model`, one test each):

- **LB9.** Names are trimmed and non-empty; siblings may not share a name
  (compared case-insensitively). Depth is at most
  `library.max_category_depth` (config).
- **LB10.** A category cannot be moved under itself or its descendants.
- **LB11.** Deleting a category deletes its subtree **and removes its
  tracks from the library** (their files are never touched), with two
  confirmations (maintainer, 2026-11-01): the first gives the number of
  subcategories and tracks; the second, shown only when some of those tracks
  are used, gives the number of playlist entries and carts that will lose
  them. Confirmed, those entries are removed from their playlists and those
  carts emptied, as if the operator had removed each one (LB17b).
- **LB12.** A track belongs to at most one category (`Track::category:
  Option<CategoryId>`). Moving it is one command; several selected tracks
  move together. *(My call.)* **Move to category ▸** lists categories only,
  and **Uncategorised**.
- **LB13.** A track that joins a category takes the category's kind; the
  kind can then be changed on the track. Changing a category's kind does not
  change its tracks.

### 5.1b Implementation notes (plan 2)

Where the implementation knowingly differs from, or settles, the text above:

- `Category::order` is the category's position in `Categories` (a `Vec` in
  tree order); the store writes it as the `categories.position` column.
  `watched_folder` arrives with plan 5 (serde default).
- Paths are compared as given by the model; canonicalisation happens in
  fp-app's helper threads where paths enter. Paths stored before schema 2 are
  kept as written, and Android MediaStore paths are not canonicalised, so a
  symlinked path saved earlier would not merge with its canonical twin.
- Uniqueness of paths is enforced by the model and merged at restore (the
  survivor is the lowest id among tracks with an operator edit, else the
  lowest id), not by a unique SQL index.
- *Add to library* with a category: a new track takes the category and its
  kind; an uncategorised existing track joins it; one already categorised
  keeps its category.
- A pre-listened (CUE) entry counts as on air for LB17b when removing tracks
  or deleting a category; removing a single entry is unchanged.
- The model cannot log: it reports each LB17c removal as an engine action
  (`TrackLeftLibrary`, with the path), which the engine logs. Explicit
  removals (LB11, LB17) are not reported.
- LB17c is also checked when a track is taken out of its category (moved
  to *uncategorised*): a missing, unused track then leaves.
- An unknown category on a track loads as uncategorised; a category with an
  unknown parent, the first category of a cycle (in stored order) and one
  deeper than 32 levels load at the root, each with a warning.

### 5.2 Size and the snapshot

The library stays in `AppState` (the model's rules need it), but today the
conductor clones `AppState` on every change (`crates/fp-engine/src/conductor.rs:364`),
the services thread walks every track on every step
(`crates/fp-app/src/services.rs:1117, 1368, 1391`) and the remote events
compare every track per snapshot (`crates/fp-remote/src/events.rs:120`).

- **LB12b.** With 20 000 tracks, a command that changes nothing in the
  library (a play, a fader move) costs no work proportional to the library
  on the conductor, services or remote threads. *Builder's call:* how (for
  example shared copy-on-write track storage with a library version, so
  unchanged libraries are compared by pointer). This lands in plan 1 or 2,
  before the library can grow; it gets a measured test like LB2.

### 5.3 One track per file

- **LB14.** The library holds at most one track per file (its canonical
  path; a path that cannot be canonicalised is compared as written). Adding
  a file that is already in the library, from anywhere (library, playlist,
  cart, an imported M3U, a watched folder), reuses that track, so its
  markers, loudness edit and category are shared everywhere it is used.

### 5.4 Tracks that live in the library

Today a track is forgotten when nothing references it. From now on:

- **LB15.** A track never leaves the library by itself (maintainer,
  2026-11-01): only *Remove from library* (LB17) and deleting its category
  (LB11) remove it, both explicit and confirmed, with the single exception
  of LB17c. A track stays when no
  playlist entry or cart uses it any more, when its file disappears (it is
  marked missing, as today) and when it has no category.
- **LB16.** Files added anywhere (a playlist, a cart, an imported M3U, a
  drop) join the library, uncategorised, and stay there (LB15). The model's
  `forget_unreferenced` (`crates/fp-model/src/reducer.rs:1239`) goes.
- **LB17.** *Remove from library* (one or several tracks) follows the same
  two confirmations as LB11: the second only when a track is used, and then
  its playlist entries and carts go too. Removing a track never touches its
  file.
- **LB17b.** *(My call.)* An entry that is on air (playing, paused, loaded
  as current, or pre-listened) cannot be removed today (`EntryOnAir`), nor a
  cart that plays. A deletion that would touch one is refused as a whole
  before any confirmation, and the interface names what holds it; nothing is
  half done. Removing a player's *next* entry picks the next one as removing
  it from the playlist does today.
- **LB17c.** The one implicit removal (maintainer, 2026-11-01): an
  **uncategorised** track whose file is gone from disk and that no playlist
  entry or cart uses leaves the library, with a line in the log. It is
  checked when the file is found missing (the existing file check, the start
  rescan) and when its last use goes. A categorised track is never removed
  this way.
- **LB18.** Markers, loudness edit, tags and re-analysis behave exactly as
  they do on a player (markers spec, loudness spec N10, tag editor): the
  library sends the same commands. A change shows at once everywhere the
  track is used, and a change to the track on air applies to its next
  transition, as today.

## 6. Watched folders (plan 5, desktop only)

- **LB19.** A category may watch one folder (with its subfolders). The same
  folder, or a folder inside another watched folder, may be watched by only
  one category; the innermost watched folder wins for a file under two.
- **LB20.** Files with an audio extension (`AUDIO_EXTENSIONS`) appearing in
  the folder join its category (LB14 if already known: an uncategorised track
  joins; a track already in another category keeps its category). Subfolders
  do not create subcategories.
- **LB21.** A file that disappears from the folder leaves its track in the
  library, marked missing, as for any file gone from disk (LB15, LB17c). It comes
  back to normal if the file returns. A category's menu offers **Remove
  missing tracks…** (confirmed, with their number; tracks in use are kept),
  so cleaning up stays explicit.
- **LB22.** The folder is followed with file-system notifications (`notify`
  crate) while the application runs, debounced by
  `library.watch_debounce_ms`, and **rescanned at start** (changes made
  while the application was closed). Scans run on a helper thread and never
  block the interface. A folder that is missing or unreadable is reported in
  the library and the log, and retried; its tracks are not removed.
- **LB23.** Files that join go to the analysis at low priority, behind
  anything a player or cart needs (the pool's urgent queue). A watched folder
  of thousands of files may take a while; the library shows the progress the
  analysis already reports.

### 6.1 Implementation notes (plan 5)

Where the implementation settles or differs from the text above:

- **LB19.** Nested watched folders are allowed; only the same folder on two
  categories is refused (`ModelError::FolderAlreadyWatched`, naming the
  category). A file under two goes to the innermost folder
  (`watching_category`, the longest matching ancestor by path components).
  Folders compare as stored (canonicalised by the folder dialog's helper
  thread). On restore `Categories::repaired` keeps a duplicated folder on the
  first category in stored order. Setting, changing or clearing a folder
  never moves tracks.
- **Threads.** The services thread never touches the file system. Scans run
  on `fp-folder-scan`; the watcher's `watch` and `unwatch` calls run on
  `fp-folder-watch`; the `notify` watcher has its own thread. Only an access
  event from a closed write counts: opening or reading a file is ignored.
- **LB21/LB22.** A watched folder whose stored path is empty or relative is
  ignored with a log line (the reducer accepts it; a hand-edited path must not
  watch every file), and its ⟳ mark shows it as not watched. A folder that is
  missing, unreadable or refused by the system is still scanned where it can
  be and retried every `tuning.missing_recheck_ms` (30 s).
  - A watched folder that is not there, cannot be read, or is empty while
    tracks of the library are under it (a mount point with nothing mounted)
    is a *Missing* or *Unreadable* trouble: nothing under it is marked
    missing, its watch is dropped, and the retry watches and walks it again.
    The empty rule applies to the folder's own walk and to the retry, not to
    the folder of an event (that event may have removed its last file). A
    folder emptied on purpose while tracks of it are still known therefore
    stays a trouble until the operator stops watching it or puts files
    back; its tracks are cleared by removing them from the table or by
    re-analysing them (the analysis marks them missing).
  - A walk of a folder holding a watched folder in trouble leaves the
    tracks under the troubled one out, and every watched folder nested in a
    root is looked at in the same job: an outer walk never marks the tracks
    of a lost nested folder.
  - An event on the watched folder itself, or on a folder holding it (moved,
    renamed, unmounted), walks that folder under the rule above instead of
    checking its tracks one by one. The folder of any other event path is
    looked at in the same job, so a file event racing an unmount marks
    nothing either.
  - At every retry each followed folder is looked at (not walked), so a
    folder lost while the application runs (a drive unplugged; the system
    drops its watch without an event) becomes a trouble.
  - Unwatching a folder (stopped, changed, deleted or lost) watches again,
    and walks, every followed folder nested in it or holding it: the Linux
    watch drops every path under the one removed.
  - A retry sends no new watch call for a folder while one is unanswered (a
    share that hangs), and walks a refused folder once per retry. If the watch
    thread ends, every folder shows *not followed* and is scanned at start and
    on retries only.
  - Only a file that is really not there (`NotFound`) marks a track missing;
    a permission error does not. Lost events (the rescan flag or a watcher
    error) rescan every watched folder.
- **Scanning.** Folder symlinks are not followed (no cycles), by the scans
  nor by the system watch (`notify` with `follow_symlinks` off); file
  symlinks are read and their canonical path joins. The category is decided from the
  path as found under the watched folder.
- **Config.** `library.watch_debounce_ms`: default 1 000, range 100–60 000,
  config file only.
- **Remove missing tracks…** is offered on every category and covers it and
  its subcategories. The count is taken when the item is clicked; with none, a
  notice replaces the question. The model recomputes the set when the command
  arrives, so a track that became used or came back meanwhile is kept.
- **UI.** The ⟳ mark and the items show in the window and in Settings →
  Library on every desktop build, the narrow layout included; the gate is the
  Android build.
- **Licence.** `notify` is CC0-1.0, allowed for that crate alone (a
  `deny.toml` exception and the narrowest `about.toml` acceptance), as CC0
  grants no patent licence.
- **The analysis exception.** The analysis of a track that is not analysed
  yet still marks it missing when its file is not there, also under an
  unmounted watched folder; it predates the watch, applies to every library
  track and only marks. The watch's guarantee (nothing marked because its
  folder is not there) covers the watch's own scans.

## 7. The library window (plan 3)

### 7.1 Window

- **LB24.** A **Library** tile in the top bar, after Cartwall and before the
  layout button, opens and closes the library window. It is lit while the
  window is open.
- **LB25.** The window floats inside the main window, is **not modal**
  (players, cartwall, shortcuts and the rest keep working), is moved by its
  title bar and resized from its edges. Its title bar has the same buttons as
  the panels: **minimise** (only the title bar stays), **maximise** (fills the
  area between the top bar and the status bar; again to restore) and
  **close**.
- **LB26.** Its position, size, minimised and maximised state, whether it is
  open, the selected category, the column widths and the search scope are
  saved with the session and restored at start (clamped to the window, so it
  never opens off screen).

### 7.2 Layout of the window

From top to bottom (mockup:
`.superpowers/brainstorm/*/content/library-floating.html`, git-ignored; the
look follows the Nocturne design system):

1. **Preview bar (§8),** like a player's header: Play, Pause, Stop; the big
   remaining time; title, artist and category; a small meter of the library
   CUE; the waveform with markers, zoomable as on a player.
2. **Category tree** (left) with a **＋** button, and below it a resizable
   split with the **table** (right): **search field** and a **Whole library**
   toggle above the table.

### 7.3 Category tree

- Items: *All tracks*, the category tree, *Uncategorised*; each with its
  colour, a ⟳ mark when it watches a folder, and its track count (its own and
  its subcategories').
- **＋** and the right-click menu create, rename, recolour, change kind,
  set or clear the watched folder (a folder picker on a helper thread), and
  delete (LB11). Categories move by drag and drop (LB10).
- Selecting a category shows its tracks **and its subcategories' tracks**.
- **LB27.** The tree stays usable with hundreds of categories: subtrees
  collapse, and the expanded state is saved with the window state.

### 7.4 Table

- **LB28.** Only visible rows are laid out and drawn (`show_rows` or
  equivalent), so 20 000 tracks scroll at the frame rate.
- Columns: the same choice as the playlists (`ui.table_columns`) plus
  **Category** and **Kind**, sortable by clicking a header. The order and
  widths are the library's own.
- Rows show the same status as playlists (missing, unreadable, waiting for
  analysis).
- **Add files** (file picker) and file drops on the window add files to the
  selected category (to *Uncategorised* when *All tracks* or *Uncategorised*
  is selected).
- Right-click menu on one or several rows: **Add to ▸** each player's
  playlists, **Add to cart ▸** (the current page's empty carts),
  **Move to category ▸**, **Edit tags…**, **Loudness…**, **Re-analyse**,
  **Reset markers to automatic**, **Remove from library** (LB17), **Show
  file location**.
- Rows are dragged onto a playlist (inserted where dropped, as tracks
  dragged between playlists) or onto a cart. Several selected rows drag
  together.

### 7.5 Search

- **LB29.** The search field filters as you type. By default it searches the
  **selected category and its subcategories**; **Whole library** searches
  everything. The scope is remembered (LB26).
- **LB30.** Matching is case- and accent-insensitive ("cancion" finds
  "Canción"), on title, artist, album, album artist, composer, genre,
  comment and the file name; every word typed must match somewhere (AND).
- **LB31.** Results come within a frame for 20 000 tracks and never block
  the interface. *Builder's call:* an in-memory normalised index in
  `fp-model` (the library is in memory anyway) or SQLite FTS5 queried off the
  UI thread; measure it.

## 8. Pre-listen in the library (plan 4)

### 8.1 Output

- **LB32.** A new holder, `Holder::LibraryCue`, with its route
  `outputs.library.cue: Option<Route>` (config), set in **Settings →
  Outputs** like the cartwall's Cue. Without a route the preview is
  unavailable, and the preview bar says why and links to Settings.

### 8.2 Behaviour

- **LB33.** Play starts the selected track from its cue-in; Pause holds it;
  Stop ends it. The waveform seeks on click, as on a player.
- **LB34.** While the preview plays (or is paused), selecting another row
  switches the preview to that track from its cue-in, in the same state
  (playing stays playing, paused stays paused). A stopped preview stays
  stopped.
- **LB35.** The preview stops when its track is removed from the library,
  and when the window is closed (not when it is minimised): when the
  window goes from open to closed. A change of the library view while the
  window stays closed (the phone's Library tab, which has no window,
  §11.1) does not stop it.
- **LB36.** The preview plays the whole file and shows its markers, as the
  players' CUE does; loudness normalization applies as on the players' CUE.

### 8.3 On air

- **LB37.** `Holder::LibraryCue` follows the on-air pre-listen rules CA1–CA6
  like the players' and the cartwall's Cue: on an on-air output that sounds,
  it does not start without **Pre-listen anyway**, and it stops when
  something starts there. `CueStart` gains `Library(TrackId)`.

### 8.4 Implementation notes (plan 4)

- The selected track is the most recently selected row (the last one
  clicked when several are selected).
- Play on a paused preview resumes it; Play on a playing one does nothing
  (Stop, then Play, restarts from the cue-in).
- A click on the waveform while the preview is stopped sets where the next
  Play starts, as a stopped player's seek does; it is dropped when the bar's
  track changes. While playing or paused it seeks. A click or the release
  seeks; a drag does not. NaN is ignored and other times are clamped into the
  file.
- Selecting nothing (no row selected, or the selected row filtered out)
  leaves a running preview as it is; a stopped bar empties.
- Selecting a track that cannot be played stops a running preview: the bar
  always shows the selected track and never shows one track while another
  sounds.
- A running preview that switches to another track moves without asking (CA3
  asks only for a pre-listen that starts from nothing); a switch while paused
  starts the new track held.
- LB17b: the library CUE does not hold a track on air; removing its track
  stops it (LB35).
- Engine: the library CUE has its own worker (`fp-library-cue`) created on
  the first start. Its placement is reported from the engine's start and
  after each settings update until then, so Settings never shows a route
  change that needs no apply. Pause and seek keep a held preview held; every
  tail (stop, seek, replacement) is de-clicked, because a preview may share a
  channel with a Main (CA7); a forced rate change reopens it at its position,
  also while held.
- The small meter has its own reading in the telemetry; it is not a
  programme meter (CA9). Its maximum restarts when the bar's track changes
  and when it is clicked.
- Without a route the transport is dimmed with the reason as its tooltip;
  the bar still shows the track, its waveform and marker editing (LB18 needs
  no output), and a line says why with a link that opens Settings → Audio
  outputs on the Library row.
- The Library row of Settings → Outputs (CUE route, Test Cue, on-air note)
  is in the Basic and Advanced views. It was hidden on the phone layout
  until plan 6 gave the phone a library screen; it is now on every layout
  (LB46).
- The CA5 notice's "by" is always a Main holder; the library CUE folds into
  the cartwall's wording.
- The preview stops when the window closes (not when minimised), and a
  preview never runs after a restart (nothing goes on air by itself).

## 9. Settings (plan 3)

- **Settings → Library:** the category tree with the same actions as the
  window (LB9–LB11), for operators who prefer to set things up there.
- **Settings → Outputs:** the library CUE route (LB32), in the Basic and
  Advanced views.

## 10. Playlists, carts and the rest

- Adding a file to a playlist or a cart in any way follows LB14: if the file
  is in the library, its library track is used.
- The playlists' track menu gains **Show in library** (opens the window,
  selects the track's category and the row) and **Move to category ▸**.
- The remote API keeps working on tracks as today; the library and its
  categories are not exposed (non-goal).

## 11. Mobile

- Plans 1–5 keep Android building and starting (`scripts/package/android.sh`
  and the emulator check in `docs/technical/mobile.md`); plan 1 is checked on
  the emulator, since the store changes there too.
- The model, store, search and preview rules are shared; the window (§7) is
  desktop only.
- **Plan 6** (its own short design session first): the phone's library
  screen replaces the Music screen (phone spec PL19b), with categories and
  search. Watched folders do not exist on Android (no direct file-system
  access to music; a later idea is binding a category to a MediaStore
  folder through the existing observer). Pre-listen on a phone, which often
  has a single output, is decided then.

### 11.1 The phone's library screen (plan 6)

Decided with the maintainer on 2026-11-04. It applies to the **phone
layout** only; a tablet keeps what it has today.

**Screens.**

- **LB38.** The phone's Music tab becomes the **Library** tab. It opens on
  the **category selector**: *All tracks*, the category tree (each
  subcategory indented under its parent, with its colour and its track
  count as in §7.3), *Uncategorised*, and **Device music**, which is the
  Music screen of the phone layout spec (PL17–PL19) unchanged. *(My
  call.)* Device music stays because it is how files reach the library on
  a phone: no file dialogs, no drops, no watched folders.
- **LB39.** A tap on a category opens its **track list** on a screen of
  its own. The list shows the category's tracks and its subcategories'
  tracks, as in the window. A back button in the list's header, and the
  system Back (PL2), return to the selector. The tab remembers the open
  category while the app runs.
- **LB40.** The track list has a search field. It filters the open
  category as LB29–LB31 do. A **Whole library** toggle searches
  everything, and the scope is remembered as in LB26.
- **LB41.** A track row shows the title, the artist and the duration
  (the file name when the title tag is missing), the status marks of §7.4
  (missing, unreadable, waiting for analysis) and a check box. Only
  visible rows are laid out (LB28).

**Sending tracks to a playlist.**

- **LB42.** A tap on a row ticks or unticks it, as on the Music screen
  (PL18). While something is ticked, the selection bar of PL19 appears,
  with the same target, the same **Add at the end** and **Add after the
  next**, and the same rule: the tracks keep the order they were ticked
  in, and adding clears the selection. Both screens use one bar.
- **LB43.** Touching and holding a row opens its **menu**:
  - **Add to ▸**: each player's playlists, at the end;
  - **Pre-listen** (LB45);
  - **Move to category ▸**: LB12, with the categories and
    *Uncategorised*;
  - **Re-analyse**;
  - **Remove from library**: LB17, with its confirmations.
  
  While rows are ticked, the menu acts on every ticked row and **Pre-listen**
  is not shown. *(My call.)* Carts, tag and loudness editing and marker
  editing stay desktop-only in this plan.
- **LB44.** *(My call.)* The track list's header has a **＋** button that
  opens Device music for that category. There the selection bar also
  offers **Add to <category>**, which runs *Add to library* with that
  category, with the plan 2 rule for tracks that already have a category
  (§5.1b). Back returns to the category's list. Device music opened from
  the selector has no such button.

**Pre-listen.**

- **LB45.** A row's **🎧** button (and *Pre-listen* in its menu) plays the
  track on the library CUE (`Holder::LibraryCue`, §8), from its cue-in. A
  slim strip at the top of the track list shows the track's title, its
  remaining time and **Pause/Play** and **Stop**. It has no waveform and no
  marker editing *(my call)*. The 🎧 of another row switches the preview to
  that track, as LB34 does. The preview stops when its track is removed
  (LB35) and when the operator leaves the Library tab.
- **LB46.** On the phone, a library CUE with no route plays on the
  **default output** (maintainer: "by default the default one"). The
  desktop keeps LB32: no route means no preview. The route can still be
  chosen in Settings → Outputs → Library, which the phone layout now
  shows (§8.4 hid it until this plan). *As built:* the fallback is the
  config field `outputs.library.default_output`, true by default on
  Android (phones and tablets) and false elsewhere, with no Settings
  control. While it is true, the Library picker's empty choice reads
  *System default*, as the Main pickers' does, and its Test Cue plays on
  the default output.
- **LB47.** The preview may share its output with what is on air
  (maintainer). When that output already sounds, the on-air pre-listen
  rules apply unchanged (LB37, CA1–CA6). The preview asks with **Pre-listen
  anyway** before it starts, and it stops when something starts on that
  output. Nothing else warns or blocks it.

**Categories.**

- **LB48.** Categories are created, renamed, recoloured, given a kind and
  deleted in **Settings → Library**, which already works by touch. *(My
  call.)* The selector has no menu of its own.

**Settled details** *(my calls, after a builder check)*.

- The track list's ticks and Device music's ticks are **two separate
  selections**: library tracks in one, device files in the other. The
  track list's selection is cleared when the operator leaves the list
  (back to the selector, or another category). Device music keeps its
  own selection as it does today (PL19).
- Device music opened from a category's **＋** has a selection bar with
  **Add to <category>** only: no player target, and no pick-one mode.
  Device music still has no 🎧, because a loose file has no pre-listen
  (maintainer, 2026-10-31).
- When rows are ticked, touching and holding a row that is not ticked
  ticks it first. The menu then acts on every ticked row, and its header
  names how many. Removing from the library keeps its LB17 confirmations.
- With no route, the phone's library CUE is placed on the default output
  the way a Main without a route is (L16). The on-air check is then the
  existing one (`cue_is_on_air` on the placed outputs). "Something starts
  on that output" means what CA1–CA6 already mean, so there is no new
  confirmation path.
- The open category is remembered (as built, across restarts too: see the
  implementation notes below). A category that
  no longer exists sends the tab back to the selector. Back goes from the
  list to the selector, and from the selector it follows PL2.
- "Phone layout" is the existing phone layout of the touch interface, as
  the other PL rules use it. The tablet's Music tab and media browser
  window are untouched.
- Plan 6 is checked on the emulator with the release APK, as plan 1 was,
  in addition to its `egui_kittest` tests.

**Implementation notes (plan 6).**

- The default-output fallback of LB46 is a config field,
  `outputs.library.default_output`, whose default is the platform
  (`cfg!(target_os = "android")`): the model and the engine cannot see the
  layout, so an Android tablet gets it too. It has no Settings control; an
  operator who wants no preview on Android sets it to false in
  `config.json`. The library CUE without a route is placed on the default
  output as a route-less Main is (L16), and moves with it when the default
  output changes (L17).
- The open category is the shared `library_view.selection`, sent with
  `SetLibraryView`; the phone keeps only the page (selector, list, Device
  music) in its view state. The category is therefore saved with the
  session and comes back after a restart; the settled
  detail first said "for the session only".
- LB35 stops the preview only when the window goes from open to closed.
  The earlier rule (any `SetLibraryView` with the window closed) stopped
  every phone preview on a category change or a *Whole library* toggle,
  against LB45; the desktop's behaviour is unchanged.
- The preview strip is drawn only on the track list. A preview started
  there goes on while the operator is on the selector or on Device music;
  only leaving the Library tab (by a tab or by Back) stops it, and so does
  a window that stops being a phone frame (a resize, a foldable opened),
  since that leaves the tab too.
- A 🎧 on a track that cannot be played (no Cue output, a missing or
  unreadable file) is disabled with the window's reason and sends nothing;
  a running preview of another track keeps running. (§8.4's "a track that
  cannot be played stops the preview" is about the window's selection, not
  a button.)
- LB42's "same target" is read as the same rule: the track list keeps its
  own bar target, separate from Device music's, since the two selections
  are separate (settled details). The operator picks the target once on
  each page.
- A tick the search hides is dropped, as in the window and on Device
  music. Durations read `03:25` (`format::clock`), as Device music's do.
- The menu is the window's row menu in a phone scope
  (`MenuScope::Phone { ticked }`), whose header names the count as the
  window's does. The ticks stay after a menu action, so the
  operator may chain actions (move, then add). Row taps are ignored while
  a popup is open, because egui's long touch ends with a primary click and
  would otherwise toggle the held row's tick.
- The *Add to <category>* bar of LB44 is its own small bar beside the
  selection bar: *Selected: N* and one button (*Add to Uncategorised*
  from *All tracks* and *Uncategorised*, the window's *Add files* rule).
  The ＋ has its own label, *Add from device music*. Device music keeps its ticks
  between visits (PL19), so ticks left from an earlier visit are added by
  *Add to <category>* too; the bar shows their count. A system Back during
  a pick (PL19b) leaves the ＋ mode as it was.

**Not in plan 6.**

- Watched folders on Android (maintainer: "OK, no watched folders"). The
  maintainer's later idea is a category bound to a device folder, with a
  button that re-reads it on demand. It is recorded here and not built.
- Carts from the library, tag, loudness and marker editing on the phone.

## 12. What the automation phase will need (not built now)

Recorded so this phase does not close doors:

- Querying a category's tracks with their last play time: the database
  makes it a query. *Builder's call:* whether plan 1 already adds a
  `last_played` per track (cheap, and useful in the table) or leaves it to
  the automation phase.
- Categories stable by id (renames do not break a template).
- **Patents.** The library itself (categories, markers, pre-listen, folder
  watching, search) is long-established practice. The automation phase must
  use only classic, publicly documented methods (category slots in a
  template, least-recently-played order, simple separation rules) described
  in their own terms, never reproduce another product's behaviour, and get a
  freedom-to-operate review by a lawyer before it ships. Nothing in this spec
  implements selection or rotation.

## 13. Configuration

New `Config` fields (rule 4), each with a default, a range in
`Config::validate` and lenient loading:

| Field | Default | Range | Meaning |
|---|---|---|---|
| `tuning.backup_interval_secs` | 300 | 30–86 400 | LB6 (next to `save_debounce_ms`) |
| `library.max_category_depth` | 8 | 1–32 | LB9 |
| `library.watch_debounce_ms` | 1 000 | 100–60 000 | LB22 |
| `outputs.library.cue` | none | a route | LB32 |

The JSON-specific `limits.max_state_file_bytes` keeps applying to
`config.json`; the builder decides whether the database needs a size guard.

## 14. Testing

- One test per rule LB1–LB48 at the lowest level that can see it: `fp-model`
  for LB9–LB18 (LB12b measured), LB29–LB30, LB33–LB37; `fp-store` (temporary directories) for
  LB1, LB4–LB7, LB14 on reload; `fp-app` services for LB2–LB3, LB19–LB23
  (with a fake watcher and a temporary folder; no sleeping, as the engine
  tests); `egui_kittest` for LB24–LB28 and the menus.
- LB2 and LB31 have a measured test with 20 000 tracks (ignored in debug if
  too slow, run in CI in release, the builder's call).
- A crash-safety test: the store killed between two saves reopens with one
  of them, never a mix.
- Import tests (LB7b–LB7d) from JSON fixtures of the current schemas
  (playlists 2, and the session and carts schemas of `master`), including a
  corrupt main file with a good `.bak1`, and an interrupted import.
- Engine tests for `Holder::LibraryCue` use the `Offline` backend.

## 15. Open points for the maintainer

None.

## 16. Plans

Each plan is one branch and one PR from an up-to-date `master` (other
agents work in parallel; expect rebases on `track.rs`, `playlist.rs` and
`fp-store`):

1. **SQLite store** — §4 with the one-time import (LB7b–LB7d), no other
   visible change; Android checked on the emulator.
2. **Categories in the model and the store** — §5 (LB9–LB18), commands and
   tests; no window yet.
3. **Library window** — §7, §9, §10: tile, floating window, tree, table,
   search, add/drag to playlists and carts, Settings → Library. The library
   CUE route in Settings → Outputs (§9) and the preview bar (§7.2 item 1)
   move to plan 4, which creates the holder they need.
4. **Library pre-listen** — §8: holder, route, preview bar, marker editing.
5. **Watched folders** — §6.
6. **Phone library screen** — §11.1 (LB38–LB48).

## 17. Documentation

Each plan updates, in the same PR: `README.md` (features), the user guide
(`docs/user/`: a new *Library* page, *Data and backups*, *Settings*,
*Playlists*, *Markers and mixing*, *Layout*), the technical docs
(`persistence.md`, `architecture.md`, `ui.md`, `audio-engine.md` for the
holder), UI strings in every locale (en-US and es-ES written, the rest
translated), and `CLAUDE.md` where commands or layout change.
