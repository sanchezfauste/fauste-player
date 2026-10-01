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
| `ui/app.rs` | `AppUi`: the main screen, keyboard, notices, OS drops, file-dialog results |
| `ui/player.rs` | One player column: header, info row, transport, waveform, tabs, footer |
| `ui/table.rs` | The track table (virtualised rows, drag and drop, context menu, column widths) |
| `ui/settings.rs` | The Settings modal (outputs, players and language, analysis, playlists) |
| `ui/settings/carts.rs`, `ui/settings/keys.rs` | Settings → Cartwall (pages, grid, cart editor, import and export) and → Keyboard shortcuts (capture, conflicts) |
| `ui/settings/remote.rs` | Settings → Remote: the HTTP and OSC switches, addresses, token, origins and senders, and each server's state read from the remote thread's status cell |
| `ui/cartwall.rs`, `ui/cart_view.rs` | The cartwall strip, and its pure view model (status, countdown, progress) |
| `ui/playlist_files.rs` | Playlist import and export on helper threads (`FileOutcome`) |
| `ui/shell.rs` | Panic isolation around each frame, and the close check in `Shell::logic` |
| `ui/exit_guard.rs` | The "Audio is on air" modal: the `ExitIntent`, the list of what is sounding, and the stop commands |
| `ui/view.rs`, `ui/format.rs` | Pure view model: what to show, how to format it (unit-tested) |
| `ui/widgets.rs`, `ui/icons.rs`, `ui/theme.rs` | Painted widgets (tiles, segmented control, tabular times, meter, fader, waveform), drawn icons, the Nocturne theme. The meter's geometry is the pure `meter_layout` (labels, lines, bars, readouts), unit-tested |
| `ui/about.rs` | The About window: version, copyright, bundled notices, and the third-party notices file (located at start-up, opened on a helper thread) |
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
column widths being dragged, the Settings section. Everything else comes
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

## Track table layout and follow

`view::column_px` turns `ColumnWidths.fractions` (or the default layout) into
pixel widths for the table's width every frame; `ViewState::table_layout`
remembers the width and fractions last applied, and `TableBuilder::reset()`
runs when either changes (not during a handle drag), since egui keeps the
widths it was first given. On handle release the widths are stored back as
fractions.

`player::follow_current` watches each player's current entry
(`ViewState::followed`). A change waits in `follow_pending` until
`scene.time − table_touched ≥ ui.follow_current_grace_secs` (a scroll over
the table, an entry drag, an open row menu or a tab click update
`table_touched`); then it sends `ShowPlaylist` if needed and puts the entry
in `follow_scroll`, which `track_table` turns into `scroll_to_row(i,
Align::TOP)` once the playlist is shown. A grace of 0 never follows.

## Waveform view

`ui/wave_view.rs::WaveView { start_secs, span_secs }` is the one mapping
between seconds and pixels: drawing (`wave_columns_in` reduces only the
visible stretch, memoised per start, span and width), marker lines and
handles, the hover time, drag-to-seek and the context menu all use it.
`ViewState::wave_zoom` keeps a `WaveZoom` per zoomed player (view, the entry
it belongs to, when it was last moved); no entry means the full view. The
wheel is read from the frame's `MouseWheel` events while the waveform is
hovered, and the frame's scroll delta is then cleared so no scroll area
moves too. A drag to seek lives in egui temp data (`SeekDrag`) keyed on the
waveform id.

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
and **Restart now** while `fp_model::restart_pending` is not empty.

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
