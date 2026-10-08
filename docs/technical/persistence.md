# Persistence and configuration

## Files (`fp-store`)

| File | Document | Content |
|---|---|---|
| `config/config.json` | `ConfigDoc` | `schema_version`, `config` |
| `data/playlists.json` | `PlaylistsDoc` | `schema_version`, `library` (tracks with manual markers and the `analysis_version` of their automatic ones; an older version is analysed again when the operator asks), `playlists` (entries with `played_by`: the players that played each one, and `repeat` / `stop_after` when set), `ids`. Tracks also hold the tags the library shows: `date`, `genre`, `album_artist`, `composer`, `comment` and `tags_read`, all optional on load (a library from an earlier version loads with them empty and the tag-only pass fills them in). The editor's tag sheet is never persisted: it reads the file each time. Schema 2; a schema 1 file's shared `played: true` is read as played by every player on restore. |
| `data/carts.json` | `CartsDoc` | `schema_version`, `pages[]` (id, name, rows, cols, `carts[]` with id, name, track, kind, looped, exclusive); cart files are tracks of the `playlists.json` library |
| `data/session.json` | `SessionDoc` | `schema_version`, `players[]` (playlist, current, next, next_explicit, mode, stop_after_current, position_secs, volume, columns (`ColumnWidths`: fractions keyed by column name, for example `{"fractions": {"title": 0.5, "artist": 0.4, "duration": 0.1}}`; the old `[#, Title, Artist, Duration]` array converts, anything else loads as the default layout, unknown names are dropped), history: the entries the player left, oldest first — loaded leniently: an unparsable history is empty, pending_start: `[entry, secs]`, where Play starts the next entry of a stopped player (rule 3a); kept only while that entry is still the next, loaded leniently (anything that does not parse is none, and the load reports a warning that is logged)), `cartwall` (open, page shown) |

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

## Writing

`write_atomic(path, bytes, backups)`:

1. writes `<file>.tmp` and `fsync`s it;
2. rotates `<file>` → `.bak1` → … → `.bakN` (`limits.backup_count`, 3);
3. renames the temporary file over the target;
4. `fsync`s the directory (Unix).

A rename or copy that fails with "permission denied" (on Windows, a
sharing violation while a scanner or indexer holds the file) is retried up to
five times, waiting 10 ms and doubling (`retry_locked`).

## Loading

`load_with_fallback` tries the file, then `.bak1`…`.bakN`. A file that
fails to parse or validate is renamed `*.corrupt-<stamp>` and kept (a
second one within the same second gets `-2`, `-3`…).
A document written by a *newer* version of the application is rejected
(this build cannot know its shape), and a copy is kept as `*.newer-<stamp>`
so a later save never destroys it. Input larger than `limits.max_state_file_bytes` is refused. If
everything fails, the store starts from defaults, and the application always
starts.

On restore, a playlist or entry whose id is already taken (a hand-edited
`playlists.json`) gets a new id; the first holder keeps its own
(`Playlists::normalize`, like `Cartwall::normalize` for carts).

`config.json` is read **leniently** (`lenient.rs`): each field is taken on its
own, so one bad value falls back to its default instead of discarding the
file. `Config::validate` then clamps every value into range and returns
warnings, which are logged.

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
  the file's folder, and the grid clamped to the limits.
- All parsers are fuzzed (see [Testing](testing.md)).

## Migrations

Each document has a `*_SCHEMA` constant. When a shape changes, bump it and
append a `vN → vN+1` function in `migrate.rs`, with a test that loads a
fixture of the old shape.

## Autosave and restore

- The services thread watches `Telemetry::model_version`. It saves config,
  playlists and session `tuning.save_debounce_ms` (1 s) after the first
  unsaved change. The version is read *before* the snapshot, so a save can
  never record a version newer than the data it wrote.
- While any player plays, the session (positions) is saved at the same pace.
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
| `fade_ms` | 1000 | 50 … 10000 |
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

### `outputs` (Settings → Audio outputs; applied at the next start)

| Field | Default | Meaning |
|---|---|---|
| `backend` | none (platform default) | backend id |
| `sample_rate` | 48000 | Hz |
| `buffer_frames` | 512 | frames per block |
| `routes[]` | empty | `{ player, main: Route?, cue: Route? }`, where `Route` is `{ backend, device, first_channel }` |
| `cartwall` | none | `{ main: Route?, cue: Route? }` for the cartwall |
| `bit_perfect[]` | empty | `{ backend, device }` of devices played bit-perfect (exclusive access, rate follows the files) |
| `dsd_output[]` | empty | `{ backend, device, mode }` with `mode` `Pcm`, `Dop` or `Native`: what a bit-perfect device receives from a DSD track. A device not listed here, or not in `bit_perfect`, converts DSD to PCM. A device listed twice keeps its first mode (warning) |
| `dsd_mix` | `ConvertToPcm` | `ConvertToPcm` or `HoldOthers`: what happens when another source needs an output carrying DSD |
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
| `outputs_view` | `Basic` | `Basic` or `Advanced`: what Settings → Audio outputs shows. It changes no output setting |
| `follow_current_grace_secs` | 10 | 0 … 600 (seconds after the operator uses a zoomed waveform or a playlist table before it follows what plays; 0 never follows) |
| `table_columns` | `["number", "title", "artist", "duration"]` | an ordered list of `number`, `title`, `artist`, `album`, `date`, `genre`, `duration`, `intro`, `file_name`; unknown names are dropped when the file is read, duplicates keep their first place, and a missing `title` or `duration` is added back (`Config::validate` warns) |
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
| `gain_smoothing_ms` | 20 | volume smoothing |
| `save_debounce_ms` | 1000 | autosave delay |
| `missing_recheck_ms` | 30000 | how often files not found are looked for again |
