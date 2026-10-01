# Data and backups

## Where files are kept

| | Linux | Windows | macOS |
|---|---|---|---|
| Settings (`config.json`) | `~/.config/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\config\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Playlists and session | `~/.local/share/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\data\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Analysis cache | `~/.cache/fausteplayer/` | `%LOCALAPPDATA%\Fauste\Fauste Player\cache\` | `~/Library/Caches/org.Fauste.Fauste-Player/` |
| Logs and crash reports | `~/.local/share/fausteplayer/logs/` | `%LOCALAPPDATA%\Fauste\Fauste Player\data\logs\` | `~/Library/Application Support/org.Fauste.Fauste-Player/logs/` |

The Linux paths follow the XDG variables (`XDG_CONFIG_HOME` and so on) when
they are set.

## Portable mode

Set the environment variable `FAUSTE_HOME` to a folder, and everything is kept
there instead, in `config/`, `data/`, `cache/` and `logs/`. This is useful on
a USB stick, or to keep separate setups side by side.

## Files

| File | Content |
|---|---|
| `config.json` | Settings, outputs, analysis thresholds, advanced tuning |
| `playlists.json` | Playlists, tracks, played flags, manual markers |
| `session.json` | For each player: playlist shown, current and next tracks, mode, position, volume, column widths |

## Autosave and backups

- Changes are saved about one second after they happen. While anything
  plays, the session (positions) is refreshed at the same pace.
- Every save writes a temporary file and then replaces the old one, so a power
  cut never leaves a half-written file.
- The previous versions are kept as `.bak1`, `.bak2` and `.bak3`.
- If a file cannot be read, the newest good backup is used. The unreadable
  file is kept, renamed `*.corrupt-<time>`, for inspection. The application
  always starts.

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
