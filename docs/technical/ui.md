# User interface (`fp-app`)

## Structure

| Module | Role |
|---|---|
| `main.rs` | Wiring: paths, logging, crash hook, store, engine, conductor, analyzer, services, eframe |
| `bootstrap.rs` | `AppPaths` from `FAUSTE_HOME` or the OS directories |
| `logging.rs` | tracing to a daily-rotated file (14 kept) via a non-blocking writer; stderr in debug builds; `RUST_LOG` overrides |
| `crash.rs` | Panic hook writing `crash-<nanos>.txt` (message, location, backtrace, version, OS), then chaining |
| `i18n.rs` | Fluent bundles (`locales/en-US`, `locales/es-ES`), per-key fallback to `en-US` |
| `services.rs` | The services thread (analysis and autosave; see [Persistence](persistence.md) and [Analysis](analysis.md)) |
| `ui/app.rs` | `AppUi`: the main screen, keyboard, notices, OS drops, file-dialog results. The window title (`cli::window_title`, the name and the version) and icon (`cli::window_icon`) come from the native frame; the top bar has no brand block |
| `ui/player.rs` | One player column: header, info row, transport, waveform, tabs, footer |
| `ui/table.rs` | The track table: virtualised rows, drag and drop, context menu, the configured columns, the header (drag to reorder, menu) and the live column resize |
| `ui/table_layout.rs` | The pure widths of the table's columns: `column_min`, `fit`, `column_px`, `resize_px`, `fractions_of`; unit-tested |
| `ui/reset_played.rs` | The Reset played question (O22): `show` returns `Some(true)`, `Some(false)` or `None`; the footer button in `player.rs` sets `ViewState::confirm_reset`, `AppUi` draws it below the close guard and sends `Command::ResetPlayed`; Esc and a deleted playlist close it; no shortcut or file drop acts under it |
| `ui/settings.rs` | The Settings modal (outputs, players and language, analysis, playlists and their table columns) |
| `ui/settings/columns.rs` | Settings → Playlists → Table columns: `column_rows`, the checkboxes, the arrows (`move_column`) and Default columns; every change goes through `Scene::set_table_columns` |
| `ui/settings/carts.rs`, `ui/settings/keys.rs` | Settings → Cartwall (pages, grid, cart editor, import and export) and → Keyboard shortcuts (capture, conflicts) |
| `ui/settings/remote.rs` | Settings → Remote: the HTTP and OSC switches, addresses, token, origins and senders, and each server's state read from the remote thread's status cell |
| `ui/cartwall.rs`, `ui/cart_view.rs` | The cartwall strip, and its pure view model (status, countdown, progress). The bar's "Stop all (n)" button sends `Command::StopAllCarts` and shows `cartwall.playing.len()`; it is the first item of a right-to-left row so it never gives way to the tabs. |
| `ui/playlist_files.rs` | Playlist import and export on helper threads (`FileOutcome`) |
| `ui/shell.rs` | Panic isolation around each frame, and the close check in `Shell::logic` |
| `ui/exit_guard.rs` | The "Audio is on air" modal: the `ExitIntent`, the list of what is sounding, and the stop commands |
| `ui/view.rs`, `ui/format.rs` | Pure view model: what to show, how to format it (unit-tested) |
| `ui/cue_window.rs` | One non-modal `egui::Window` per running CUE, drawn from the pure `view::cue_window_view`; icons through `ui/glyphs.rs` |
| `ui/widgets.rs`, `ui/glyphs.rs`, `ui/icons.rs`, `ui/theme.rs` | Painted widgets (tiles, segmented control, tabular times, meter, fader, waveform), drawn icons, the Nocturne theme. `ui/glyphs.rs` maps each transport action (`TransportAction`: play, next, pause, stop, fade stop, stop after, restart, previous, cue) to a Phosphor glyph or a drawn icon; the player, the cartwall and the playlist menu draw their transport icons only through it (`tests/glyphs.rs` guards that), and the CUE window too. The meter's geometry is the pure `meter_layout` (labels, lines, bars, readouts), unit-tested |
| `ui/about.rs` | The About window: version, copyright, bundled notices, and the third-party notices file (located at start-up, opened on a helper thread) |
| `ui/tag_editor.rs` | The tag editor modal (O23): `TagEditor` with its phases (`Reading`, `Unreadable`, `Ready` with a `Form` holding the sheet as read and the draft), the field boxes, the cover area and the pure checks it shares with `fp-model` |
| `tags.rs` | The `fp-tags` worker: `TagJob` in, `TagOutcome` out (see below and [Analysis](analysis.md)) |
| `ui/controller.rs` | The `Controller` trait between the UI and the rest |
| `ui/files.rs` | Accepted audio extensions and folder expansion |

## Data flow

Each frame, `AppUi::ui`:

1. loads `ctl.model()` (an `Arc<AppState>`) and `ctl.telemetry()`;
2. drains refusals (shown as a transient notice) and results of helper threads
   (file dialogs, dropped folders, device scans);
3. handles the keyboard, unless a text field has focus. Bindings come from
   `config.shortcuts` (the key name and exact modifiers must match). Only
   first presses count, and repeats are ignored. Player and cart positions
   are resolved against the players and the cart page shown. A shortcut
   whose command `fp_model::command_available` rejects (R28) is dropped;
4. draws the screen and sends `Command`s through `ctl.send`. Transport
   buttons take `enabled` from `fp_model::availability`, so unavailable ones
   are dimmed and inert;
5. requests a repaint: continuously while anything plays, fades or cues,
   otherwise every 100 ms for the clock.

The UI keeps only **view state**: selection, drag target,
the column edge being dragged (`LiveResize`), the Settings section. Everything else comes
from the snapshot.

`Controller` is implemented by `ConductorHandle`. The tests use a fake that
applies commands with `fp_model::apply` and records them and their refusals,
so the UI is tested end to end without audio.

## Language switching

When `config.ui.language` changes (Settings → Players → Language), `AppUi`
rebuilds its `I18n` on the next frame. The first frame only records the
language the interface was built with.

## Marker editing

`player.rs::edit_markers` handles marker editing on the waveform:

- the context menu remembers the time where it was opened
  (`ViewState::wave_menu`) and sends `SetMarker` or `ResetMarkers`;
- with Alt held, handles are drawn on the markers. A drag picks the marker
  nearest to the press origin (`ViewState::marker_drag`) and sends a single
  `SetMarker` on release. A waveform click never seeks while Alt is held.
- when `MarkerFractions::ignored` is set (`!players.use_cue_markers`),
  `widgets::cue_edge_look` turns off the head and tail shading and the
  cue-in and cue-out lines are drawn at `theme::CUE_EDGE_IGNORED_ALPHA`.
  Alt-drag editing is unchanged. The countdowns, the duration column and the
  playlist totals use `Track::play_range`.

## Track table layout, columns and follow

**Columns.** The table draws `config.ui.table_columns` (repaired with
`normalize_columns` on every frame, so a list set without `Config::validate`
still has Title and Duration). Each cell is a `match` on `TableColumn` in
`track_table`; the text of the plain columns is `view::cell_text`.
A row whose file is missing or unreadable gives the reason and the path
(`Scene::file_tip`) on its icon in the `#` column and on its title, so the
reason shows without the `#` column too.

**Widths.** `ColumnWidths.fractions` is a map from column to fraction (the
old four-number array is converted when the session loads). `table_layout::column_px`
turns it, the shown columns and the table's width into pixels on every frame:
the stored fraction of each shown column scaled to the room left by the
columns that have none (they take their default width), the minimums
(`column_min`) winning, and the sum always equal to the width. Every column
is given to egui as `Column::exact(px)` with `resizable(false)`, so
`TableBuilder` keeps no width of its own (the old `TableBuilder::reset()` and
`ViewState::table_layout` are gone). `TableBuilder::min_scrolled_height(0.0)`
is set because its default of 200 points hides the last rows under the footer
in a short window.

**Live resize (O16).** `egui_extras` 0.36 recomputes only the dragged column,
one frame late, so the table draws its own grab zones (`resize_handles`, after
the body so they win over the rows). Dragging one creates
`ViewState::live_resize`; `live_widths`, called before anything is drawn,
reads the raw pointer and calls the pure `resize_px` (the columns left of the
edge keep their width, the ones on its right share what is left in proportion
to their widths when the drag began, minimums respected). On release, if
anything moved, it sends one `SetColumnWidths` with `fractions_of(columns,
px)` and keeps drawing those widths for up to `HOLD_SECS` until the model has
them. A drag whose columns or table width changed is dropped. `AppUi::column_widths`
reports the pixels of the last frame (tests use it).

**Header.** Each header cell is dragged with a `DragColumn { index }` payload;
a drop on the left half of a cell sends `Scene::set_table_columns(move_column_before(..))`,
on the right half the slot after it; the drop line is drawn on the edge. The
cell's context menu (`header_menu`) lists the optional columns with
`with_column_shown`; the Settings list uses `with_column_shown` and
`move_column`. The payload type is not `DragEntry`, so a dragged track
never reorders columns nor the other way round. `set_table_columns` sends one
`UpdateConfig`, validated by the reducer, unless the validated list is
already in use.

**Follow.** `player::follow_current` watches each player's current entry
(`ViewState::followed`). A change waits in `follow_pending` until
`scene.time − table_touched ≥ ui.follow_current_grace_secs` (a scroll over
the table, an entry drag, an open row menu or a tab click update
`table_touched`); then it sends `ShowPlaylist` if needed and puts a
`FollowScroll { entry, align: TOP, animated: true }` in `follow_scroll`, which
`track_table` turns into `scroll_to_row` once the playlist is shown. A grace
of 0 never follows. At start-up `player::scroll_to_next_once` does the same
once per player (`ViewState::startup_scrolled`) for `view::start_scroll_target`,
the player's next entry when it is in the playlist the tab shows, with
`Align::Center` and no animation; the scroll area's own clamping keeps it
inside the list.

## Track tooltip and tag editor

**Tooltip.** The row shows `view::track_tooltip` through `on_hover_ui`, after
the usual tooltip delay: title, artist, album, date, genre, duration, format
and path, leaving out what the track lacks. The format is the upper-cased
extension, then the sample rate and the bit depth when known.

**Menu item.** *Edit tags…* is enabled by `view::tag_edit_availability`
(`fp_model::tag_edit_block` plus `fp_analysis::tags::can_write_tags`, judged
from the extension so nothing touches the disk on the UI thread). A refused
item is dimmed and its tooltip gives the reason (`block_key`). Choosing it
sets `ViewState::edit_tags`; `AppUi` hands it to `open_tag_editor` on the next
frame, which numbers the session, queues `TagJob::ReadSheet` and opens the
modal in its `Reading` phase.

**One frame.** At the start of each frame `AppUi` drains, before it builds
the `scene` (which borrows `self`, so all the `&mut self` work comes first):

- `tag_outcomes`: `TagOutcome::SheetRead` goes to the editor of that track;
  `SheetWritten` goes to `tag_saved`; `CoverLoaded` goes to the editor only if
  its track and session match;
- `cover_picks`: the path the image dialog sent back becomes
  `TagJob::LoadCover`.

**Save.** The modal draws the draft and returns an `EditorAnswer`. On `Save`
the rule is judged again (`tag_edit_availability` now: the track may have gone
on air since the modal last drew), the job is built by `TagEditor::save_job`
and sent with `start_tag_job`; the editor shows "Saving…" and cannot be
cancelled meanwhile. `tag_saved` takes the result: on success the re-read
summary goes to the library as `ApplyTags`, the notice names any field the
file did not keep, the player's cover follows when the cover changed
(`MediaCache::set_cover`; a track with no cache entry reads the new cover from
its next analysis) and the modal closes; on failure the modal stays open with
the reason, which is also in the notice.

**Cover dialog.** `Change…` returns `EditorAnswer::ChangeCover`;
`start_cover_dialog` runs the file dialog on its own thread and the chosen
path comes back over a channel with the track and the session. The tag worker
checks and decodes the image (`TagJob::LoadCover`, `TagOutcome::CoverLoaded`),
so the UI thread never reads it. `cover_busy` blocks a second choice and the
save meanwhile.

**Sessions.** Each opening of the editor has an id (`TagEditor::session`). An
image choice or a `CoverLoaded` answer carries the id of the session that
asked, and one from a closed editor is dropped, so it never reaches a later
editor for the same track.

**Modal.** While the editor is open no keyboard shortcut acts (not even
Delete: `AppUi::keyboard` returns early) and files dropped on the window are
discarded. Esc or a click on the backdrop cancels, unless a save is running.
The settings window opened over it is drawn above, and the close guard above
both.

**Fields.** A field cut when read (`TagSheet::is_cut`) is shown read-only with
a note and is never copied back from its box into the draft. An existing
cover with no thumbnail shows "This cover cannot be shown; it is kept as it
is", and **Remove** is off unless the cover shown is a front cover and the
format can store pictures.

## Waveform view

`ui/wave_view.rs::WaveView { start_secs, span_secs }` is the one mapping
between seconds and pixels: drawing (`wave_columns_in` reduces only the
visible stretch, memoised per start, span and width), marker lines and
handles, the hover time, click-to-seek, drag-to-pan and the context menu all use it.
`ViewState::wave_zoom` keeps a `WaveZoom` per zoomed player (view, the entry
it belongs to, when it was last moved); no entry means the full view. The
wheel is read from the frame's `MouseWheel` events while the waveform is
hovered, and the frame's scroll delta is then cleared so no scroll area
moves too. `widgets::waveform` reports a click's seek target and a drag's
sideways movement (`WaveOutput { response, seek, pan_dx }`); egui's click rule
is the drag threshold, so only `Response::clicked()` seeks and a drag never
does. The player pans a zoomed view with `WaveView::pan` (a drag without zoom
does nothing). A held pan drag is flagged in egui temp data keyed on the
waveform id; `widgets::pan_dragging` reads it so the view does not follow the
playhead meanwhile. Alt-drag (markers) and a drag that starts under the
shield report no pan.

## CUE window

`ui/cue_window.rs` draws one window per player with a running CUE, from
`view::cue_window_view` (title, artist, elapsed, remaining, `paused`, the
position fraction and `can_load_next`). Its waveform, seek range and
remaining time run to the end of the file, because a CUE plays the whole
file. It only sends commands: `SeekCue` (a click on the waveform),
`SetCuePaused`, `CueToNext` (Load as next) and `SetCue(player, false)` (Stop
and the close button). The model rules behind it: `seek_cue` leaves `paused`
alone (a seek on a paused CUE stays paused), `set_cue_paused` only emits on a
change, `cue_entry` clears `paused` (a moved CUE restarts unpaused from the
entry's cue-in), and `follow_cue` runs on `SetNext` (a double-click on a row)
to move a running CUE to a playable new next; `CueToNext` only sets the next,
since the CUE is already on that entry. A single click on a
table row does the same through `view::cue_follow_target`, which gives the
clicked entry when the player has a CUE on another entry and the file can be
played; the table then sends `CueEntry`. A right-click only selects.

## Nothing blocks the UI thread

Native file and folder dialogs (`rfd::AsyncFileDialog` driven by `pollster`),
scans of dropped folders and device enumeration all run on helper threads.
Their results come back through channels.

`main.rs` looks for the third-party notices (`about::find_notices`: next to
the executable, `../Resources/licenses`, `../share/doc/fauste-player`,
`../share/licenses/org.fauste.FaustePlayer`) once before the first frame, and
the About window opens the file with `open` on the `fp-open-notices` thread.

## Theme and fonts

`ui/theme.rs` holds the Nocturne tokens as sRGB constants. The design's
`oklch` colours are converted once. Corners are square. Inter (400/500/600,
OFL) is embedded, and Phosphor icons come from `egui-phosphor`, regular and
fill. Waveform colours are a named palette (`WAVE_PALETTE`); an unknown name
falls back to Slate. Inter's digits are proportional, so times are painted
with `widgets::paint_tabular`/`tabular_label`, which centre every digit in a
cell as wide as the widest one: a countdown keeps its width as it runs. Fonts are installed on the first frame, and drawing starts
on the next one, when they are bound.

## Window decorations

On Linux, `fp-app` enables winit's `wayland-csd-adwaita` feature. `eframe` is
built without its default features, which would otherwise bring it. A Wayland
compositor without server-side decorations (GNOME) leaves the frame to the
client, and without that feature winit draws a bare fallback frame with
non-standard buttons. With it, the title bar has the usual minimise, maximise
and close buttons. Windows and macOS use their native frames.

## Exit guard

`fp_model::on_air` decides what is on air: players that are playing or paused
and playing carts. A player CUE or a cartwall CUE does not count.

The close is checked from eframe's `logic`, in `Shell::logic`, not from the
drawing pass, so it also runs while the window is minimised. When a close is
requested and something is on air, `AppUi::guard_close` cancels it, restores
and focuses the window and sets `view.exit_guard`. The next frame draws the
modal (`exit_guard::show`): **Cancel**, `Esc` (answered in `keyboard` before
the text-field check) or a click on the backdrop (the modal's `should_close`)
dismisses it, **Stop and close** sends the stop commands (every player on air,
all carts) and then closes; the session is saved on shutdown as before. The
guard dismisses itself if nothing is on air any more. It takes precedence over
Settings and About (it is drawn after them, so it is the top modal), and
keyboard shortcuts are ignored while it is open; MIDI and remote commands still
act. Each cart is listed with its 1-based position on its page. In degraded
mode (the banner after a UI panic) the close is not guarded, because the dialog
cannot be drawn. `ExitIntent` names why the guard opened, so other exits can
reuse it.

`ExitIntent::Restart` opens the same modal for **Restart now** while something
is on air (its own text: "Stop and restart"). With nothing on air,
`begin_restart` sets the restart flag, marks the close as confirmed and sends
the viewport close command. See "Restart" in `architecture.md`.

## Settings window

`settings::window_size` gives the one size of the window: 900 × 640, clamped to
the main window minus a margin (never below 320 × 300). It does not change when
a section is selected. The section header (title and **Restore defaults**) is
fixed; the body below it sits in `ScrollArea::both`. Rows use
`labelled_row`, with a label column of `LABEL_WIDTH` (180 px), so every section
shares one grid. The footer spans the window width and holds the restart notice
and **Restart now** while `fp_model::restart_pending` is not empty. A transient
notice (`SettingsDeps::notice`) briefly takes the place of the restart text;
**Restart now** stays.

Settings → Remote keeps what is being typed (`RemoteState::drafts` for text,
`RemoteState::numbers` for the drag values, which do not wait for Enter), so
opening another section applies it through `remote::flush`. The count of
outdated tracks (Settings → Analysis, and the start-up notice) comes from
`AppUi`'s `services::OutdatedCount`: the library is scanned once per model
snapshot, not on every frame (`SettingsDeps::outdated`).

## Panic isolation

`Shell::ui` runs `AppUi::ui` inside `catch_unwind`. After a panic it draws only
a banner with **Restart interface**, which calls `AppUi::reset_view`.
The `test-hooks` feature adds `AppUi::fail_next_frame` and
`Services::fail_next_step` for tests. Release builds do not contain them.

## Internationalisation

- Every visible string is a Fluent message in `locales/en-US/main.ftl`
  (the source) and `locales/es-ES/main.ftl`.
- `tests/i18n.rs` fails if the two key sets differ.
- Arguments use `tr_args(key, &[("name", value.into())])`.
- To add a language, add `locales/<tag>/main.ftl`, register it in `i18n.rs`,
  and extend the key-set test.
