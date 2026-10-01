# Missing files: say why, and notice when they come back

**Goal.** A track whose file is missing or unreadable shows a warning icon
and is skipped, but nothing tells the operator why, and a file that comes
back (a drive mounted after the program started) stays "missing" until
*Re-analyse all*. After this plan:

1. the warning icon (track table and cartwall) has a tooltip with the
   reason and the file path;
2. missing files are looked for again every `tuning.missing_recheck_ms`
   (default 30 s) and become playable by themselves when found.

**Spec.** `docs/superpowers/specs/2026-09-25-fauste-player-design.md`:
the `tuning` row of the config table, "Missing files come back by
themselves" in the analysis section, and the track-table tooltip.

**Design.**

- `Tuning::missing_recheck_ms: f64`, default 30 000, clamped to
  1 000 … 3 600 000 by `Config::validate`. In `tuning` (config file only),
  next to `reconnect_interval_ms`, which plays the same role for devices.
- `Services` keeps `last_recheck: Option<Instant>` and a probe thread
  (`fp-file-probe`). When the interval has passed and no look is running,
  the paths of the `Missing` tracks not in flight go to the probe, which
  checks each folder once and then each file in a folder that exists. The
  tracks found leave `failed` and join `forced`, so `submit_new` sends them
  to the analysis pool, which answers from the cache (path, size, mtime)
  when they were analysed before. Neither the services thread nor the pool
  waits on a share that is offline (review I1: the first version sent
  every missing track to the pool each interval).
- `route` sends `SetFileState` only when the state changes: a file still
  missing must not bump the model version and trigger an autosave every
  interval.
- `Unreadable` files are not retried by themselves: that would decode
  them every interval. *Re-analyse all* still checks them.
- Tooltip: a pure `view::file_problem(&Track) -> Option<&'static str>`
  gives the Fluent key (`file-missing-tip`, `file-unreadable-tip`, both
  with `$path`); the table's `#` label and the cart tile call
  `on_hover_text` with it.

## Tasks

### 1. `tuning.missing_recheck_ms`

- Test first (`fp-model` config tests): the default is 30 000; 10 and
  10 000 000 are clamped to 1 000 and 3 600 000 with a warning naming
  `tuning.missing_recheck_ms`; a config without the field loads with the
  default.
- Add the field, default and clamp.
- Docs: `docs/technical/persistence.md` tuning table.

### 2. Re-check missing files

- Tests first (`fp-app/tests/services.rs`, interval set to 1 000 ms):
  - `a_missing_file_that_comes_back_becomes_playable`: the path does not
    exist, the track turns `Missing`; the WAV is then written at that path;
    the track becomes `Ok` and analysed with no request from the operator.
  - `a_file_still_missing_is_looked_for_without_touching_the_model`: after
    the track turns `Missing`, several intervals pass; the analysis count
    grows (it is looked for) and `model_version` does not change.
- Implement `last_recheck`, the re-check in `analysis_step` (it now takes
  `now`), and the unchanged-state guard in `route`.
- Docs: `docs/technical/analysis.md` (how the app uses the analyzer),
  `docs/user/troubleshooting.md` ("A track shows a warning icon").

### 3. The tooltip

- Tests first: `view::file_problem` unit tests (Ok → None, Missing →
  `file-missing-tip`, Unreadable → `file-unreadable-tip`); a kittest that
  hovers a missing row's `#` label and finds a tooltip with the path; the
  same for a cart.
- Locale keys in `en-US` and `es-ES`.
- Docs: `docs/user/playlists.md` and `docs/user/cartwall.md` legends
  ("hover it for the reason").

### 4. Waveforms of loaded tracks come first

Found while testing: the analysis pool had a single FIFO queue, and the
services submitted the tracks on screen together with the rest of the
library. A track loaded while a library was being analysed (a playlist
just added, *Analyse now*, *Re-analyse all*) got its waveform last.

- Tests first: in `fp-analysis`, an urgent job goes ahead of ten queued
  ones, a promoted queued job goes ahead once, and promoting a job already
  answered does nothing. In `fp-app`, a track set as next while twelve are
  queued is analysed among the first.
- `Analyzer::submit_urgent` and `promote` (a second queue, taken first
  with `select_biased!`). The bookkeeping tracks urgent and running
  generations so nothing is analysed twice. `Services::submit_new` sends
  wanted tracks urgent and promotes wanted tracks already in flight.

### 5. Finish

- Gate (fmt, clippy, tests), a fresh review on the most capable model,
  PR, CI on the three OSes, merge.
