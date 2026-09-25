# Phase 1 · Plan 4 — The application (egui UI and wiring) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship the `fauste-player` binary (`fp-app`). It wires store, engine, conductor and analyzer together and draws the v3 design in egui: the player columns, the track tables, the status bar and the Settings modal (Phase 1 subset), with English/Spanish UI strings through Fluent.

**Architecture:**
- **Bootstrap:** resolves the paths (`FAUSTE_HOME` overrides the OS directories, for portable installs and tests), starts logging (tracing to a daily-rotated file in the log dir, plus stderr in debug builds) and installs a panic hook that writes crash reports. It then loads the `Store`, builds the `Engine` (the cpal default host plus the implicit Null), spawns the `Conductor` thread and the `Analyzer` pool, and starts a **services** thread.
- **Services thread:** turns analysis results into model commands, fills a UI media cache (peaks and cover thumbnails), submits unanalysed tracks, and autosaves:
  - config and playlists, debounced by `save_debounce_ms` whenever `model_version` changes;
  - the session on the same cadence while anything plays;
  - everything once more at shutdown.
- **UI:** reads `Arc<AppState>` and `Telemetry` snapshots every frame, sends `Command`s, and keeps only view state. Pure view-model functions (formatting, row status, badges) are unit-tested. Interaction is tested headlessly with `egui_kittest`. Every frame runs inside `catch_unwind` (spec §8.5).

**Tech Stack:** eframe/egui/egui_extras 0.36.2 (API verified by a compiled probe: `App::ui(&mut self, ui, frame)`, `egui::Panel::top`, `CentralPanel::show(ui, …)`, `egui::Modal`, `TableBuilder`), egui-phosphor 0.14.0, egui_kittest 0.36.2 (dev), rfd 0.17.2, fluent-bundle 0.16.0, unic-langid 0.9.6, sys-locale 0.3.2, tracing-subscriber 0.3.23, tracing-appender 0.2.5. Font: Inter 400/500/600 (SIL OFL 1.1, the licence shipped next to the fonts).

**Spec:** §8 (UI), §7 (autosave, crash recovery), §9 (logging, panic hook), §3 rules 1 and 16–20 (UI-level rules), §8.4 (Settings subset).

**Execution note:** as for plan 3 (the executor authors and executes; TDD for logic, headless UI tests for interactions, screenshots to compare with the design; one fresh reviewer at the end).

## Global Constraints

- **Rules carried over from plans 1–3:** English identifiers, no third-party product names, no hardcoded product limits, and the lints (`forbid(unsafe_code)` and no unwrap, expect or panic).
- **UI strings:** every visible string comes from Fluent.
  - `en-US` is the source; `es-ES` carries the design's Spanish labels.
  - A test fails if the two key sets differ.
- **Nocturne theme** (from the design system's `styles.css`):

  | Token | Value |
  |---|---|
  | Background | `#161826` |
  | Surface | `#232532` |
  | Text | `#e9e9ed` |
  | Accent | `#9184d9` |

  Neutral ramp 100…900 = `#f3f5fe #e4e7f5 #cfd3e5 #b2b6ca #9397ab #75798c #595d6c #3f424d #292b31`; accent-400 is `#b5abfc`.

  The design's own `oklch` constants, converted exactly to sRGB:

  | Use | Colour |
  |---|---|
  | On-air row | `#b02a2d` (text `#ff645f`) |
  | Next row | `#146d34` (text `#3eab5e`) |
  | Amber | `#f0b135` |
  | Cue blue | `#6bcbf7` |
  | VU green | `#4cc157` |
  | VU yellow | `#f2cf3b` |
  | VU red | `#f1383e` |
  | Play green | `#3fc168` |
  | Intro shade / line | `#43b2e1` |
  | Outro shade | `#f4a25c` |
  | Outro line | `#fba962` |

  Waveform palette (played / unplayed):

  | Name | Played | Unplayed |
  |---|---|---|
  | Violet | `#b5abfc` | `#595d6c` |
  | Amber | `#f9b64f` | `#6e5232` |
  | Cyan | `#58d1e5` | `#355f6a` |
  | White | `#f3f5fe` | `#595d6c` |
  | Orange | `#f98942` | `#71462e` |
  | Magenta | `#e56bc1` | `#663d59` |
  | Ice | `#b7def3` | `#4f606d` |
  | Sand (default) | `#e0cfac` | `#675c4b` |

- **Visual rules:** corners are square (rounding 0) and the font is Inter.
- **Phase 1 Settings scope** (spec §8.4):
  - backend, sample rate and buffer size;
  - Main/Cue device and channel pair per player;
  - "Test Main" / "Test Cue";
  - the Players settings, the Analysis settings with "Re-analyse all", and Playlists (create, rename, delete, music folder).
- **Output changes** (backend, device, rate, buffer) take effect at the next start, and the modal says so. Ruling: rebuilding live buses under playing players is not in Phase 1. Everything else applies at once.

## Review Focus

1. **The UI thread panics in a draw function.** Expect audio to continue, a banner offering "Restart interface", and the next frame to draw normally. Test: `a_panicking_frame_shows_the_banner_and_keeps_running` (kittest).
2. **The app is closed while tracks are playing.** Expect the session to be saved with positions, and every player to come back paused at its position on the next start. Test: `shutdown_saves_the_session_with_positions`.
3. **A huge playlist** (5 000 entries). Expect the table to stay responsive, with only visible rows built. Test: `large_playlists_build_only_visible_rows`, counting row callbacks under kittest.
4. **Keyboard shortcuts while typing in a text field** (renaming a playlist). Expect keys 1–9, Delete and Backspace not to trigger players or removal. Test: `shortcuts_are_ignored_while_editing_text`.
5. **Files dropped from the OS that are not audio, or folders.** Expect non-audio files to be ignored, folders to be expanded to the audio files they contain (non-recursive in Phase 1), and nothing to crash. Test: `dropped_paths_keep_only_audio_files`.

---

### Task 1: Crate, bootstrap, logging and panic hook
- `crates/fp-app` produces the binary `fauste-player`.
- **`bootstrap::paths() -> AppPaths`** honours `FAUSTE_HOME`.
- **`logging::init(&AppPaths) -> Guard`** uses a non-blocking daily-rotated file and keeps 14 files.
- **`crash::install_panic_hook(log_dir)`** writes `crash-<unix>.txt` with the message, location, backtrace, version and OS, and then chains the previous hook.
- **Tests:**
  - `fauste_home_overrides_the_os_directories`;
  - `the_panic_hook_writes_a_crash_report` (a panic inside a `catch_unwind` in a test, then the report file must exist).

### Task 2: Internationalisation
- **Resources:** `locales/en-US/main.ftl` and `locales/es-ES/main.ftl`, embedded with `include_str!`.
- **`I18n::new(requested: Option<&str>)`** uses the config's `ui.language`, else `sys_locale::get_locale()`, else `en-US`. A missing key falls back per key to `en-US`, and then to the key itself.
- **Methods:** `tr(key) -> String` and `tr_args(key, &[(name, value)])`.
- **Tests:**
  - `both_locales_define_the_same_keys`;
  - `spanish_is_used_when_requested`;
  - `unknown_locales_fall_back_to_english`;
  - `missing_keys_fall_back_per_key`;
  - `arguments_are_substituted`.

### Task 3: View model (pure)
`ui::format` and `ui::view`:

- **Formatting:**
  - `countdown(remaining_secs) -> ("-02:27", ".2")`: the main text plus the tenths;
  - `clock(secs)`: `mm:ss`, or `h:mm:ss` from one hour;
  - `number_width(len)`: at least 2 digits, 3 from 100 entries;
  - `playlist_footer(entries, positions) -> (remaining, elapsed, total)` (spec §3 rule 20).
- **`player_view(&AppState, &Telemetry, player) -> PlayerView`** holds:
  - status (on air, stopped, paused), title, artist and the next line;
  - elapsed, remaining and total, with `end_warning` (red countdown, rule 17);
  - the intro badge (`Some(remaining)` only when `intro_end` is set and the position is before it, blinking in the last 3 s, rule 18) and the outro badge (rule 19);
  - the markers as fractions of the total, for the waveform.
- **`row_status(&AppState, player, entry) -> RowStatus`**: `Current`, `Next`, `Played`, `Unavailable` (Missing or Unreadable) or `Normal`.
- **`volume_db(gain)` / `gain_from_fader(pos)`:** a fader law of 0 dB at the top and −∞ at the bottom, with the dB tooltip.
- **Tests:** every function, plus rules 17–20 through `player_view`.

### Task 4: Engine test tone, and the services thread
- **Test tone:** `Engine::play_test_tone(route: &Route, frequency_hz, secs, level_db)`. It fills a source directly, with no worker (the ring holds the whole tone), then attaches it and starts it on the route's bus. `ConductorHandle::test_tone(route, freq)` sends it over a second bounded channel.
  - Engine test (Offline): `a_test_tone_plays_on_the_requested_route_only`.
- **`services::Services`**, with `step(now)` for tests and `spawn()` for production:
  - submits every track with `analyzed == false` once, and resubmits all on "Re-analyse all";
  - routes analyzer results to `ApplyAnalysis`, or to `SetFileState` for `Missing`/`Unreadable`, and puts peaks and cover into `MediaCache` (`Arc<RwLock<HashMap<TrackId, TrackMedia>>>`);
  - autosaves (debounced config and playlists, periodic session);
  - `shutdown()` saves everything with the positions from telemetry.
- **Tests** (Offline engine, temp `FAUSTE_HOME`):
  - `unanalysed_tracks_are_submitted_once_and_results_reach_the_model`;
  - `missing_files_are_marked`;
  - `changes_are_saved_after_the_debounce`;
  - `shutdown_saves_the_session_with_positions`.

### Task 5: Theme, fonts and drawn icons
- **`ui::theme`:** the colour constants, `apply(ctx)` (visuals, rounding 0, spacing) and `fonts()` (Inter, plus Phosphor regular and fill).
- **Icons drawn as shapes:** fade stop (a filled closed shape: a rectangle plus a ramp) and stop-after-current (a reversed return arrow plus a larger stop square).
- **Test:** `every_waveform_colour_name_resolves` (the config names → palette, with an unknown name falling back to Sand).

### Task 6: Main screen
- **Layout** (spec §8.3, v3):
  - top bar;
  - a horizontally scrollable row of player columns (min width 380 px);
  - the status bar.
- **Player column:**
  - header: `P1`, state dot and label, badges, BP (inactive), SINGLE/CONT and CUE;
  - info row: 64 px cover or vinyl placeholder, VU (20 segments with peak hold, fed by telemetry peaks), vertical fader, title, artist, and the next line;
  - transport grid: Play/NEXT (2 rows), Stop, Fade stop, Pause (blinks while paused) and Stop-after (disabled in Single, with a tooltip);
  - countdown with tenths, and `elapsed / total`;
  - waveform: played and unplayed colours, intro/outro shading, a dashed MIX marker, the playhead, a hover time and click to seek;
  - playlist tabs;
  - track table: resizable `#`, Title, Artist and Duration columns stored through `SetColumnWidths`; row colours from `row_status`; double-click sets next; context menu (Play now, Set as next, Pre-listen on CUE, Add tracks below…, Duplicate, Move to ▸, Remove); drag and drop within a list, across players and onto tabs; OS file drops;
  - footer: "+ Add" (rfd), the count and the times.
- **Keyboard:** keys 1–9 play; Delete/Backspace removes the selection; Esc closes. All of them are ignored while a text field has focus.
- **Refused commands** show as a transient notice in the status bar.
- **Tests (kittest):**
  - `clicking_play_sends_play`;
  - `double_clicking_a_row_sets_next`;
  - `the_context_menu_removes_an_entry`;
  - `number_keys_play_players`;
  - `shortcuts_are_ignored_while_editing_text`;
  - `large_playlists_build_only_visible_rows`;
  - `dropped_paths_keep_only_audio_files` (the pure filter function).

### Task 7: Settings modal
- **Sections:**
  - Audio outputs: backend list with availability, device lists from `enumerate_devices`, rate and buffer with the computed latency, per-player Main/Cue device and first channel, Test Main/Test Cue, and the "applies at next start" notice;
  - Players;
  - Analysis ("Re-analyse all");
  - Playlists (music folder via rfd, create, rename, delete, with the model's refusals shown).
- Config edits go through `Command::UpdateConfig`, after `Config::validate`, and `SetPlayerCount`.
- **Tests (kittest):**
  - `changing_fade_time_updates_the_config`;
  - `deleting_the_last_playlist_shows_the_refusal`.

### Task 8: Frame panic isolation, final wiring and visual check
- **Panic isolation:** `App::ui` wraps the frame in `catch_unwind`. On a panic it logs, sets `ui_degraded` and shows the banner; "Restart interface" resets the view state.
- **Test:** `a_panicking_frame_shows_the_banner_and_keeps_running`, using an injected panic hook in the view.
- **Run the real binary on this machine:**
  1. take X11 screenshots with 4 players and generated WAV files;
  2. compare them with the v3 design;
  3. fix the visible mismatches;
  4. record the remaining deviations in the ledger.
- **README:** a short `README.md` covering building, running, system packages and the fonts licence.
