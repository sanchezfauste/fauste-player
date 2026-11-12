# Data and backups

## Where files are kept

| | Linux | Windows | macOS |
|---|---|---|---|
| Settings (`config.json`) | `~/.config/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\config\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Library, playlists, carts and session (`fauste.db`) | `~/.local/share/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\data\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Analysis cache | `~/.cache/fausteplayer/` | `%LOCALAPPDATA%\Fauste\Fauste Player\cache\` | `~/Library/Caches/org.Fauste.Fauste-Player/` |
| Logs and crash reports | `~/.local/share/fausteplayer/logs/` | `%LOCALAPPDATA%\Fauste\Fauste Player\data\logs\` | `~/Library/Application Support/org.Fauste.Fauste-Player/logs/` |

The Linux paths follow the XDG variables (`XDG_CONFIG_HOME` and so on) when
they are set.

The snap keeps its files in its own folder instead: settings in
`~/snap/fauste-player/current/.config/fausteplayer/`, the database
and logs in `~/snap/fauste-player/current/.local/share/fausteplayer/`, and
the analysis cache in `~/snap/fauste-player/common/.cache/fausteplayer/`.
`sudo snap remove fauste-player` deletes them too (it keeps a snapshot for
a while; `--purge` does not).

## Portable mode

Set the environment variable `FAUSTE_HOME` to a folder, and everything is kept
there instead, in `config/`, `data/`, `cache/` and `logs/`. This is useful on
a USB stick, or to keep separate setups side by side.

## Files

| File | Content |
|---|---|
| `config.json` | Settings, outputs, analysis thresholds, advanced tuning |
| `fauste.db` | The library (tracks, their categories, manual markers, tags, loudness), the playlists with their played flags, the cart pages, the automation's rotations, schedule and as-run log (kept `automation.as_run_keep_days` days, 90 by default), and the session: for each player the playlist shown, current and next tracks, mode, position, volume and column widths (by column), and the window layout |
| `fauste.db.bak1`, `.bak2`, `.bak3` | Copies of the database (see below) |
| `window.ron` | The main window's size, position and whether it was maximized (computer only; on Wayland the system chooses the position) |

## Autosave and backups

- Changes are saved about one second after they happen, and only what
  changed is written. While anything plays, the session (positions) is
  refreshed at the same pace.
- The database is written in transactions, so a power cut never leaves a
  half-saved change.
- At the first save of each session, and then about every five minutes
  while changes are being saved, the application copies the database to
  `fauste.db.bak1` and shifts the older copies up, to `.bak2` and `.bak3`.
  Restarting several times in a row therefore pushes the older backups out.
- If the database cannot be read, the newest good backup is used. The
  unreadable file is kept, renamed `fauste.db.corrupt-<time>`, for
  inspection. If every copy fails, the application starts empty. It always
  starts.
- A copy that cannot be used is kept too: when the application starts from
  another copy (or an empty database), the unusable ones are renamed
  `fauste.db.bak1.corrupt-<time>` (or `.newer-<time>`) so the next backup
  never pushes them out.
- If another copy of the application is using the database, this one runs
  without saving and says so at start.
- A database written by a newer version of the application is not opened:
  it is kept as `fauste.db.newer-<time>`, and the application starts from
  the newest backup it can read, or empty when there is none, so opening
  the data with an older version never destroys it.
- `config.json` is saved the same way as before: a temporary file replaces
  the old one, and the previous versions are kept as `.bak1`, `.bak2` and
  `.bak3` beside it.
- If the database cannot be used at all in a session (for example, it
  cannot be renamed because the folder is read-only), the application
  runs without saving until the next start, and the log says so.

## Playlists and session from an earlier version

Earlier versions kept the playlists in `playlists.json`, the cart pages in
`carts.json` and the session in `session.json`. The first start of this
version reads them (with their `.bak` copies) once and puts everything in
`fauste.db`: the library with its markers, tags and loudness, the playlists
with their played flags, the cart pages and the session. Then the old files
are renamed with `.imported` at the end (`playlists.json.imported`,
`playlists.json.bak1.imported`, and so on). They are no longer used and you
can delete them once you are sure everything is there.

To go back to an earlier version:

1. Close the application.
2. Move `fauste.db`, `fauste.db-wal`, `fauste.db-shm` and every
   `fauste.db.bak*` to a folder outside the data folder. Do not delete them.
3. Remove `.imported` from the names of the old files.
4. Start the older version.

Everything saved since the update stays only in the folder you moved the
files to: the old files do not have it. If you upgrade again later, the
old files are imported again, as long as no `fauste.db` or `fauste.db.bak*`
is in the data folder.

If a start is interrupted during the import, the next one finishes it. If
one of the old files cannot be read (too large, or written by a newer
version), the application imports nothing and runs without saving that
session, so the old files stay where they are; the log names the file. Fix or
remove it and start again. Once `fauste.db` exists, old JSON files are
never imported again.

## Crash recovery

After a crash or a restart, every player comes back with its playlist, its
current and next tracks, and its position, but **paused or stopped**. Nothing
goes on air by itself. A player that was at the very end of its track comes back
at the start of that track instead, so pressing Play plays it rather than
ending it at once.

## Editing `config.json` by hand

Close the application first (if audio is on air, it asks before closing).
Unknown or out-of-range values are corrected to the nearest valid value when
the file is loaded, and the corrections are logged. The `limits` and `tuning` sections hold advanced values (resource
limits, engine timing) that are not in the Settings window.
