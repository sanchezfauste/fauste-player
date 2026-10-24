# User interface (`fp-app`)

## Structure

| Module | Role |
|---|---|
| `main.rs` | Wiring: paths, logging, crash hook, store, engine, conductor, analyzer, services, eframe |
| `bootstrap.rs` | `AppPaths` from `FAUSTE_HOME` or the OS directories |
| `logging.rs` | tracing to a daily-rotated file via a non-blocking writer, with identical consecutive lines collapsed (`Collapse`) and a per-file size cap (`Capped`), both on the log thread; `limits.max_log_file_mb`, `limits.max_log_files` and `limits.log_repeat_summary_secs` are read by `fp_store::peek_log_limits` before logging starts; the default filter is `info,pulseaudio=warn`; stderr in debug builds; `RUST_LOG` overrides |
| `crash.rs` | Panic hook writing `crash-<nanos>.txt` (message, location, backtrace, version, OS), then chaining |
| `i18n.rs` | The locale registry (`LOCALES`), language negotiation, Fluent bundles with per-key fallback to `en-US` |
| `services.rs` | The services thread (analysis and autosave; see [Persistence](persistence.md) and [Analysis](analysis.md)) |
| `ui/app.rs` | `AppUi`: the main screen, keyboard, notices, OS drops, file-dialog results. The window title (`cli::window_title`, the name and the version) and icon (`cli::window_icon`) are set on the `ViewportBuilder` in `main.rs` (`run`) and drawn by the native title bar; the top bar has no brand block |
| `ui/player.rs` | One player column: header, info row, transport, waveform, tabs, footer. A bit-perfect player (`fp_model::bit_perfect_player`; bit-perfect rate and fader spec BF7, BR15): `meter_column` draws its fader at the top and disabled while `fp_model::fader_locked`, with the tooltip `tip-volume-locked`; the button that started a fade drains a band (anchored to the button's right edge so its edge sweeps left to right, `player::sweep_band`; tone `theme::play_sweep` on Play/Next, `theme::sweep_tone` of the button's content on the others) timed by the pure `view::fade_remaining_fraction` over `PlayerView::fade_ms` (the running fade's length, segue overlaps included) from the frame that first showed the fade, restarted when `PlayerView::fade_seq` changes (start and sequence kept in egui temp memory; the engine reports no fade progress); the bar is drawn on the Play/Next tile when `!PlayerState::fade_stopping()` and the fade did not start from Previous (a Next crossfade or a segue overlap), on the Previous tile for `FadeOrigin::Previous` (`PlayerView::fade_origin`, set by the reducer in `begin_fade`) and on Fade stop for a fade stop, so a fade stop pressed during a crossfade moves it; the level under the fader is `widgets::fader_readout` (text from the pure `view::volume_readout`, `volume-readout-silent` for silence, dimmed when locked) on the row of the meter's loudness line; the Play/Next tooltip on air is `tip-next-bit-perfect`; the Play/Next tile is styled by `player::play_button_style()` (flat green, no outline, white icon; `theme::PLAY_FILL` is `NEXT_ROW` idle and on air; tokens `theme::PLAY_FILL*`, `PLAY_ICON`, `TileStyle::disabled_fill` keeps a disabled filled tile from looking enabled; `tests/play_button_ui.rs` pins each state's fill and the WCAG >= 3:1 contrast of the icon, also under the countdown band); while Next is counting a fade down (disabled, not clickable) the tile keeps the on-air fill and an undimmed icon; on a paused player it is `player::paused_play_style(widgets::blink(time))`, lit in phase with Pause and `PLAY_FILL_PAUSED_DIM` when dark; `meter_column` paints a 1 px `theme::DIVIDER` (the waveform border's colour, which `widgets::waveform` strokes with the same token) centred in the 10 px gap between the meter's edge and the fader's knob, stopping above the fader's level readout (which may overhang the gap); the BP badge is a disabled `widgets::tile` styled by `widgets::bp_tile_style` (green border and text when on; `TileStyle::dim_when_disabled` is false so the green is not dimmed); the waveform's MIX marker style comes from `view::mix_active` |
| `ui/wave_panel.rs` | The waveform panel the player column and the CUE window share (operator feedback 4, Q7): `show(ui, scene, view_state, &WavePanelInput)` draws `widgets::waveform` with the zoom, the follow after the grace, wheel and drag pan, the Full view button, the intro and outro badges and marker editing, keyed by `WaveKey`; it returns `WavePanelOutput { seek }`, which the player sends as `Seek` and the CUE window as `SeekCue` |
| `ui/header_layout.rs` | The player header's fit: the pure `fit(items, available, spacing)` says which items of the left group have room (the highest rank collapses first, rank 0 never), so no two header items overlap. `player::header` measures every item and the right group (CUE, mode, BP, gain badge) first, reserves the right group, and draws only the items `fit` keeps. A header badge that collapses is not drawn, so it also loses its accessible name. Unit-tested; `tests/header_overlap_ui.rs` checks the rectangles at the least width in three languages |
| `ui/table.rs` | The track table: virtualised rows (the selected one outlined by `selection_outline` in `theme::SELECTION_OUTLINE`, each cell painting its share: top and bottom edges, the left edge in the first column and the right edge in the last), drag and drop, context menu, the configured columns, the header (drag to reorder, menu) and the live column resize. The drag's drop index comes from the geometry in `table_layout`, never from the widget under the pointer; `DropTarget` is keyed by player, playlist and index, so only the table under the pointer draws the violet bar. A pointer counts only when the table's own layer is the top one there (`Context::layer_id_at`, the background layer when no area is there), so a floating window over the table (the CUE window) hides the rows under it. OS file drops use the same target (`ViewState.file_drop`); a pointer over the table but on no target sets `file_drop_refused`, and `AppUi::file_drops` then inserts nothing, so the end-of-list fallback is only for a drop whose position the system did not report or that is outside every table. Scrolling during a drag (operator feedback 4): while a `DragEntry` payload is held or the system hovers files over the window, and the pointer is in the body on the table's own layer, `drag_scroll_offset` sets the table's offset (`TableBuilder::vertical_scroll_offset`) before the table is built, from the body, offset and largest offset the last frame left (`ViewState.table_scroll`, keyed by player as egui keys the table's scroll state, `TableScroll`, used only when the last pass or the one before stored it, `Context::cumulative_pass_nr`). Files from the system count only once the pointer has moved since they began hovering (`ViewState.files_pointed`, set in `AppUi::ui` from the frame's `PointerMoved` events): some platforms send no pointer events with them, and a stale position in an edge zone would scroll to the end. Near an edge it adds `table_layout::drag_scroll_speed` times the frame's `stable_dt` (at most `MAX_DRAG_SCROLL_DT`, 0.1 s), clamped to the list's ends, and requests a repaint while it moves; it does not move on the frame the button is released, so the entry lands where the line was drawn. egui's `ScrollArea` ignores the wheel while a widget is dragged (`Context::dragged_id`), so the table takes the wheel itself then, and only then (no double scroll), the way egui's vertical-only `ScrollArea` reads it (`take_drag_wheel`: `smooth_scroll_delta.y`, or `x + y` with `Style::always_scroll_the_only_direction`). On the frame an entry is dropped egui has already cleared `dragged_id` while the rest of a wheel notch is still in `smooth_scroll_delta`: the table takes it without applying it, so `ScrollArea` does not apply it either and the entry lands on the line drawn the frame before. The drop index and the bar are computed after the table from its new offset, so they follow the pointer. A `DragColumn` (header) drag scrolls nothing. The row popup (`TIP_WIDTH`, wrapped, kept inside the window) is the only hover information on a row and its first line is the file error (`view::track_tooltip` takes the reason, `TipField::Problem`); the analysis flag comes from `view::analysis_flag` (see *Analysis activity*); the row menu's Re-analyse sends `ServiceRequest::ReanalyseTrack` through `Scene::request` (dropped without services or with a full queue), and is disabled with `menu-reanalyse-queued` or `menu-reanalyse-running` as its tooltip while `Scene::analysis` has the track queued or running. A double-click on a row sends `SetNext` also when egui counts it as a triple click (a double-click less than 0.6 s after a click elsewhere); `set_next_on_click` remembers the row set in the current burst of clicks (`ViewState.set_next_sent`), so a real triple click sends it once |
| `ui/view.rs` (`row_is_next`) | O37: `player.next == Some(entry)` whatever the row's `RowStatus`, so the playing row also draws the next arrow when it is its own next; the double-click stays a no-op on it and the context menu's Set as next is enabled there |
| `ui/tab_strip.rs` | The pure layout of the playlist tabs (O35; the "+" new-playlist button takes the end of the strip, so the tabs and arrows are laid out in the strip less `ADD_TAB_WIDTH`; its popup sends `Command::CreatePlaylistIn`, rule 29): `layout` (tab width, overflow, view width), `clamp_offset`, `reveal`, `step`; unit-tested. `tabs` in `player.rs` draws them clipped to the view, with arrows and the wheel; the scroll lives in `ViewState::tab_scroll` per player, and the shown tab is revealed again only when the key (shown playlist, tab count, view width) changes, so the operator's scrolling is not undone every frame |
| `ui/table_layout.rs` | The pure widths of the table's columns: `column_min`, `fit`, `column_px`, `resize_px`, `fractions_of`, and the drop geometry of an entry drag (operator feedback 4, Q4): `drop_index` (pointer, the scroll area's visible rect, its scroll offset, `ROW_HEIGHT` and the entry count give the boundary index, or `None` over the header, a column resize grab zone, the scroll bar, outside the body), `boundary_y` (the boundary's y, `None` outside the view) and `on_column_edge`; and the edge scroll of a drag, `drag_scroll_speed` (points per second, negative up: zero outside the body and outside the edge zones, `DRAG_SCROLL_EDGE` = 36 points or a third of a shorter body, ramping linearly up to `DRAG_SCROLL_MAX_SPEED` = 900 points per second at the edge); unit-tested |
| `ui/panel_layout.rs` | The main window's pure pixel pass (resizable layout spec RL11): RL8's interface constants (`MIN_PLAYER_WIDTH`, `MIN_PLAYER_HEIGHT`, `PANEL_GAP`, `CONTENT_HEIGHT_CAP`, `SCROLL_BAR_ROOM`), `arrange` (a `PassInput`: the players, the model's layout, the middle area, the cartwall's `CartwallSizes`, the maximized panel; to an `Arrangement`: the players' row, each shown player's width, the overflow, the cartwall's rectangle, the splitter beside it, whether it fits its content) on every dock, and `cartwall_range`; the splitter maths (`drag_pair`, `pair_shares`, `LiveSplit` for the players, `CartwallDrag` for the cartwall); `panel_on_air`; the maximize state (`Maximised`, `ArrangementKey`); see [Main window layout](#main-window-layout); unit-tested |
| `ui/panels.rs` | The main window's panels (resizable layout spec): `players_row` draws the shown players at the pass's widths, each column keyed by its `PlayerId`, with `player_splitter` in each gap; `cartwall_drag_layout` and `cartwall_splitter` for the cartwall's splitter; the generic `splitter`; `layout_controls` (the top bar's tiles and layout menu); `header_menu` (Maximize, Restore, Hide); `settle_view`; see [Main window layout](#main-window-layout) |
| `ui/reset_played.rs` | The Reset played question (O22): `show` returns `Some(true)`, `Some(false)` or `None`; the footer button in `player.rs` sets `ViewState::confirm_reset`, `AppUi` draws it below the close guard and sends `Command::ResetPlayed`; Esc and a deleted playlist close it; no shortcut or file drop acts under it |
| `ui/settings.rs` | The Settings modal (its text buttons all come from `widgets::text_button`: the painted text plus `SETTINGS_BUTTON_PAD` on each side, `SETTINGS_BUTTON_HEIGHT` high; `text_button_at_least` for buttons sharing a column width) (outputs, players and language, analysis, playlists and their table columns). Players shows `settings-players-bit-perfect-note` under the fade time and automatic mix while any player is a bit-perfect player; both settings stay editable (BR15) |
| `ui/settings/devices.rs` | Settings → Audio outputs, Advanced view: per routed device the bit-perfect switch, its own rate and buffer (`offered_rates`, `offered_buffers`) and its DSD mode with the reason a mode is missing (`dsd_not_offered`), then the DSD mix and silence; the Basic \| Advanced selector is `view_selector` in `ui/settings.rs` |
| `ui/settings/columns.rs` | Settings → Playlists → Table columns: `column_rows`, the checkboxes, the arrows (`move_column`) and Default columns; every change goes through `Scene::set_table_columns` |
| `ui/settings/carts.rs`, `ui/settings/keys.rs` | Settings → Cartwall (pages, grid, cart editor, import and export) and → Keyboard shortcuts (capture, conflicts) |
| `ui/settings/loudness.rs` | Settings → Loudness (loudness normalization spec N22): the Normalize loudness toggle, the target, true-peak ceiling, maximum boost and maximum trim sliders (ranges of N1), the notes that changes apply to tracks loaded afterwards and that a bit-perfect output is never normalized, and the count of tracks waiting for a measurement (`SettingsDeps::unmeasured`) |
| `ui/loudness_edit.rs` | The per-track loudness controls (N20, N21): `info`/`info_block` (the shared gain breakdown, below), the Exclude from normalization toggle and the Manual trim slider, each change one `Command::SetTrackLoudness`; and the Loudness… popup (`popup`, drawn by `AppUi` after the CUE windows) |
| `ui/settings/remote.rs` | Settings → Remote: the HTTP and OSC switches, addresses, token, origins and senders, and each server's state read from the remote thread's status cell |
| `ui/cartwall.rs`, `ui/cart_view.rs` | The cartwall panel (at the bottom or on a side, and its collapsed side strip, `collapsed_strip`), and its pure view model (status, countdown, progress). The bar's "Stop all (n)" button sends `Command::StopAllCarts` and shows `cartwall.playing.len()`; it is the first item of a right-to-left row so it never gives way to the tabs. Button height comes from `cart_view::button_height(available, rows)` (nominal 40 px, minimum 28 px) while the cartwall fits its content (`Arrangement::fit_content`; the 60 % cap is `panel_layout::CONTENT_HEIGHT_CAP`), and from `cart_view::filled_button_height` (minimum 28 px, no upper clamp) otherwise (RL13); widths fill the panel, never under `cart_view::MIN_BUTTON_WIDTH`; the grid scrolls only below the minimums. A cart's name is laid out by the pure `cart_view::fit_name`: its words wrap at their boundaries, and a word too wide for the button is cut with an ellipsis and ends the name, never split across lines. `cartwall::sizes` gives the pass its content and least heights, its least width (`min_width`: the header's fixed parts or the columns, whichever is larger) and its collapsed thickness. A button's colours come from the pure `cart_view::cart_look(color, playing)` (`CartLook`: fill, hover and pressed fills, border, elapsed-time fill, kind marker, waveform), with the palette in `theme::cart_color` and the strengths in `theme::CART_*`. The waveform behind the text spans `CartView::wave_range` (the play range, idle and playing carts only), reduced with `widgets::memo_columns_in` into a memo kept in egui temp data per cart (`("cart-wave", cart id)`) and drawn by `widgets::paint_wave_columns`. A playing cart's sweep (I19, I22): the fill grows from the left to the played share and a 2 pt line marks its edge; the position is `cart_view::CartSweep` (kept per cart in egui temp data, `("cart-sweep", cart id)`), the engine's last position carried on with the frame clock (clamped to the cart's end and to twice the step between changed snapshots (observed and smoothed by the sweep itself: the audio buffer's step), with two conductor ticks as the floor, never backwards unless the snapshot is, as on a loop), and the cartwall requests a repaint every frame while a cart with a known length plays, so the line does not depend on the players or meters being shown (before, only `view::animating` set the frame rate and a cart alone repainted at the idle rate). The cart editor's colour row (`settings/carts.rs`, `swatch`) exposes each swatch as a selected/unselected radio button. |
| `ui/playlist_files.rs` | Playlist import and export on helper threads (`FileOutcome`) |
| `ui/shell.rs` | Panic isolation around each frame, and the close check in `Shell::logic` |
| `ui/exit_guard.rs` | The "Audio is on air" modal: the `ExitIntent`, the list of what is sounding, and the stop commands |
| `ui/view.rs`, `ui/format.rs` | Pure view model: what to show, how to format it (unit-tested). `mix_active` decides whether a player's MIX marker (on the player and in its CUE window) is lit (violet, `theme::MIX_LINE` through `widgets::mix_marker_color`, the colour of the MIXING badge and apart from the orange outro; dim grey otherwise): Continuous mode and not a bit-perfect player (BR15) |
| `ui/cue_window.rs` | One non-modal `egui::Window` per running CUE, drawn from the pure `view::cue_window_view`; its waveform is `wave_panel` with `WaveKey::Cue`; icons through `ui/glyphs.rs` |
| `ui/widgets.rs`, `ui/glyphs.rs`, `ui/icons.rs`, `ui/theme.rs` | Painted widgets (tiles: `tile` paints `TileStyle::hover_border` and `hover_fill` while the pointer is over it or pressing it (`TileStyle::plain`: `NEUTRAL_800` fill, `NEUTRAL_500` border), and draws a `theme::FOCUS_RING` outline (inside its border) while it has keyboard focus; `tile` shows its label as the tooltip, so a caller with another tooltip uses `tile_without_tooltip` and never adds a second `on_hover_text`; the design's on/off `toggle` that Settings and the loudness controls share, segmented control, tabular times, meter, fader, waveform; `fader` takes an `enabled` flag: a disabled fader is greyed and senses hover only, so it takes no drag, click or wheel. The curve is `fp_model::volume` (`dB = 192 × (travel^(1/8) − 1)`; `travel_to_db`, `db_to_travel`, `gain_from_fader`, `fader_from_gain`), shared by the fader, MIDI and the remote API. A drag moves the travel by the pointer's movement since the previous frame (kept in egui temp memory from the press origin, so it never jumps to the pointer and a click alone sends nothing); Ctrl or Shift scale drag and wheel by `FADER_FINE` (0.1); the wheel is one `FADER_WHEEL_STEP` (0.025 of travel, 0.6 dB near 0 dB) per notch: `Line` deltas count as notches, `Point` deltas (trackpads) as notches of `line_scroll_speed` points, and the remainder is kept (`FaderWheel`, egui temp memory); a sideways delta counts only with Shift held (some systems send Shift + wheel that way), and while the fader is hovered it clears `smooth_scroll_delta`, so the players' row never scrolls under it. A drag that goes on with no stored state (through a lock) starts from the fader's own position, and the state is dropped when the drag ends or the fader locks. While the knob is held the fader draws the drag's travel, not the model's: the hand on the fader wins over a MIDI or remote change, which the next drag starts from. A double click sends travel 1.0 only on the knob (`FADER_KNOB_SLOP`, 6 px above or below), so two quick taps elsewhere on the track change nothing. The scale marks `FADER_MARKS_DB` (0, −2, −4, −6, −8, −10, −12, −15, −20, −25, −30, −40, −50, −∞) are drawn as a notch across the track (`FADER_NOTCH_REACH` each side) with the level on its right, away from the meter's own scale (the track sits at the fader's left, next to the meter; `FADER_WIDTH` is 30 px), at the travels `fader_mark_travels` computes through the curve, dimmed when locked), drawn icons, the Nocturne theme. `ui/glyphs.rs` maps each transport action (`TransportAction`: play, next, pause, stop, fade stop, stop after, restart, previous, cue) to a Phosphor glyph or a drawn icon; the player, the cartwall and the playlist menu draw their transport icons only through it (`tests/glyphs.rs` guards that), and the CUE window too. The meter's geometry is the pure `meter_layout` (a `Ruler` per side, the `ticks`, the label lines, the bars and the readouts), unit-tested; its width (`meter_width`) follows the scale's widest label, and the player's meter column follows it. Nothing is painted over the bars: `tests/meter_view.rs` reads the painted shapes and accepts in a bar only filled rects in the bar or zone colours |
| `ui/about.rs` | The About window: version, the author attribution required by `NOTICE`, copyright, licence (links open on a helper thread), the disclaimer, bundled notices, and the third-party notices file (located at start-up) |
| `ui/tag_editor.rs` | The tag editor modal (O23): `TagEditor` with its phases (`Reading`, `Unreadable`, `Ready` with a `Form` holding the sheet as read and the draft), the field boxes, the cover area and the pure checks it shares with `fp-model` |
| `tags.rs` | The `fp-tags` worker: `TagJob` in, `TagOutcome` out (see below and [Analysis](analysis.md)) |
| `ui/controller.rs` | The `Controller` trait between the UI and the rest |
| `ui/files.rs` | Accepted audio extensions and folder expansion |

## Main window layout

Resizable layout spec RL8, RL11. `AppUi::ui` splits the window into the
top bar (`TOP_BAR_HEIGHT`), the status bar and the middle area, and the
middle area (inside an 8-point margin) goes through the pure pixel pass
`panel_layout::arrange`. Its `PassInput` is every player id in display
order, the model's `Layout`, the area, the cartwall's `CartwallSizes`
(`cartwall::sizes`: its content height, its least height, its least width
and its thickness while collapsed) and the maximized panel; its
`Arrangement` is the players' row, each shown player's width, whether the
row overflows, the cartwall's rectangle (`None` while it is hidden), the
splitter beside it and whether it fits its content.

- **The cartwall** has a range, `panel_layout::cartwall_range`: from its
  least size (RL8: at the bottom `min_height`, the header and the rows at
  `MIN_BUTTON_HEIGHT`; on a side `min_width`, from `cartwall::min_width`:
  the larger of the header's fixed parts, Stop all measured with a
  two-digit count, and the page's columns at `MIN_BUTTON_WIDTH` with their
  gaps) to what leaves the shown players their minimum: at the bottom
  `panel_layout::players_least_height` (`MIN_PLAYER_HEIGHT`, plus
  `SCROLL_BAR_ROOM` when the row overflows, so three rows still fit above
  the scroll bar), on a side `MIN_PLAYER_WIDTH` each with their gaps; both
  ends within the shares a size can hold (`MIN_CARTWALL_SHARE` to
  `MAX_CARTWALL_SHARE`, 5 % to 95 % of the area along its axis), so a drag
  never draws a size its share cannot keep; never below the least size. At the bottom with `size: None` it fits its
  content as before, at most `CONTENT_HEIGHT_CAP` (60 %) of the area,
  unclamped; a share is clamped to the range. On a side (`on_a_side`) it is
  the full height of the area and its share of the width
  (`CartwallLayout::share`, 0.3 by default) is clamped to the range; the
  players take the rest of the width, and only they scroll when it is too
  narrow. It is never wider than the area. With no player shown it takes
  the whole area. `Arrangement::splitter` is the gap beside it (none while
  it is hidden or no player is shown).
- **The players** each get `MIN_PLAYER_WIDTH` first; the spare room is
  split by `Layout::shown_fractions`. When the minimums alone do not fit
  (by more than half a point) every column is at its minimum and the row
  scrolls horizontally (`panels::players_row`), with its scroll bar in a
  strip of `SCROLL_BAR_ROOM` under the columns.
- A column's egui id is `("player", PlayerId)`, so hiding a player never
  hands its table's scroll or its popups to its neighbour; its label is
  still `P<n>` from its place among all players.
- `AppUi::player_rects` and `AppUi::cartwall_rect` give the last frame's
  rectangles (tests).

**Splitters** (RL12). The 8-point gap between two shown players is a
splitter (`panels::player_splitter` over the generic `panels::splitter`,
which senses clicks and drags (`Sense::CLICK | Sense::DRAG`, not
focusable, so it is no Tab stop; a click is never a drag), shows the resize
cursor and paints a line only while hovered or dragged, and senses nothing
while the layout is locked). A drag lives in `ViewState::live_split`
(`panel_layout::LiveSplit`): `drag_pair` moves the pair with the pointer
and keeps its total and both minimums, and `panels::live_widths` applies it
before the row is drawn, so the drag shows in the same frame. On release
one `SetPlayerShares` goes out, only if the drag moved something, with
`pair_shares`: each player's room above its minimum (a player at its
minimum sends `SHARE_FLOOR` of the pair's room, as the model refuses 0), so
the next pass draws exactly the dragged widths. The released widths are
held until the model's shares change or `HOLD_SECS` pass, as the table's
column resize does; a change in the shown players or the row's width ends
the drag. Only a primary-button drag starts one. A double-click sends
`EqualPlayerWidths`.

**Cartwall splitter** (RL12). `Arrangement::splitter`, the gap beside the
cartwall, is a splitter (`panels::cartwall_splitter` over the generic
`panels::splitter`): `ResizeRow` at the bottom, `ResizeColumn` on a side;
inert while the layout is locked or the cartwall has no range
(`cartwall_range` is `None` while it is collapsed, on any dock). A drag
(primary button only) lives in `ViewState::cartwall_drag`
(`panel_layout::CartwallDrag`): it starts from the drawn size taken into
the range (a cartwall that fits its content can be taller than the range
allows), follows the pointer along the cartwall's axis within the range,
and `panels::cartwall_drag_layout`,
called before the pass, hands the pass a copy of the layout with the
drag's share, so the drag shows in the same frame. On release one
`SetCartwallSize(Some(share))` goes out, only if the drag moved something;
the share is the drawn size over the area, so the next pass draws the same
size. The size is held until the model's size changes or `HOLD_SECS` pass;
a change of dock, area or range, or a lock, ends the drag. A drag or a
double-click ends a maximise first, and a drag starts from the pass
without it (`normal`). A double-click sends `SetCartwallSize(None)` at the
bottom (fit the content) and `SetCartwallSize(Some(DEFAULT_SIDE_SHARE))`
on a side.

**Buttons scale** (RL13). `Arrangement::fit_content` is true only at the
bottom without a share, with nothing maximized and a player shown. Then
`cartwall::strip` sizes the buttons with `cart_view::button_height` (40
points at most, as before); otherwise with `cart_view::filled_button_height`
(what fits, never under `MIN_BUTTON_HEIGHT`, no upper clamp), so the grid
fills the panel. Widths always fill it, never under `MIN_BUTTON_WIDTH`.
Below the minimums the grid scrolls (`ScrollArea::both`). In a button
taller than `BUTTON_HEIGHT` the text block of a 40-point button is centred
vertically; the waveform, the elapsed-time fill and the CUE outline take
the whole button.

**Collapsed** (RL14). `CartwallSizes::collapsed` is the header's height
while the cartwall is collapsed. At the bottom the cartwall is then its
header, whatever its share; on a side it is a strip that wide
(`cartwall::collapsed_strip`): the expand caret (a tile labelled
`cartwall-title`, pointing at the players), the on-air dot while
`panel_on_air(Cartwall)`, and a background (`Sense::CLICK`, no Tab stop)
that opens `panels::header_menu`. A collapsed cartwall has no range, so its
splitter is inert; it is never maximized: the pass ignores it, the menu
disables Maximize, and `settle_view` ends a cartwall maximize when it
collapses.

**Top bar** (RL15). `top_bar` lays out its right-hand block (the clock,
Settings, About, the pending tile) first, from the right end, and passes
where it begins to `panels::layout_controls`, which draws one tile per
player (`player-label`, its place among all players) and one for the
cartwall (`layout-panel-carts`, the locale's `-cartwall` term, the word
`cartwall-title` shows in capitals), lit while shown, in a horizontal
`ScrollArea` (salt `panel-tiles`) bounded to the room left of that block;
the layout button follows the scroll area, so it is always reachable. The
player ids and the shown-panel count are worked out once per frame; a click sends
`SetPanelHidden`. A tile's accessible label and tooltip say what a click
does (`tip-layout-hide`, `tip-layout-show`, `tip-layout-show-on-air`); the
last shown panel's tile and every tile of a locked layout are disabled,
with `error-last-panel` or `error-layout-locked` as the tooltip. Every tile, shown or hidden,
draws `panel_layout::panel_activity` in its corner: a red square while a
player plays or any cart plays, two amber bars while a player is paused
(`panel_on_air` still decides the hidden tile's "on air" label). The layout button
(`tip-layout-menu`) opens an `egui::Popup::menu` with the dock radio items
(`SetCartwallDock`), Lock layout (`SetLayoutLocked`), Equal player widths
and Reset layout; while locked every entry but the lock is disabled. While
the cartwall is hidden, the `ToggleCartwall` shortcut maps to no command
(`shortcut_command`).

**Maximize** (RL16). `ViewState::maximised` holds a `panel_layout::Maximised`
(the panel and an `ArrangementKey`: the players, the hidden panels, the
dock and the lock); it is never saved and sends no command. The pass gives
a maximized player all the spare width and the others their minimum, and
shrinks the bottom cartwall to its least height; a maximized bottom
cartwall leaves the players their least height. `panels::settle_view`,
called before the keyboard, ends it when the key changed. A splitter drag
or double-click ends it; a drag starts from the pass without it (`AppUi::ui` runs the pass a
second time, `normal`, while a panel is maximized). The headers' background
(an `interact` with `Sense::CLICK`, not focusable, added before the
header's tiles, so the tiles keep their clicks) opens `panels::header_menu`:
Maximize or Restore, and Hide; both disabled while locked, Maximize also
with one panel shown. The same two actions are the icon buttons
`panels::header_buttons` (Hide: Phosphor `MINUS`; Maximize or Restore:
`CORNERS_OUT` or `CORNERS_IN`; 18 points square, `HEADER_BUTTONS_WIDTH` in
all), sharing `can_hide`, `can_maximise` and `toggle_maximise` with the
menu. They are drawn first in the right-to-left right group of a player's
header (its width is part of the group `header_layout::fit` reserves) and
at the right end of the cartwall's header (counted in
`cartwall::min_width`). The top bar's tiles are drawn in a centred layout
so they share the right-hand buttons' centre line.

**Hidden players** (RL17, RL18). `panels::settle_view` also drops the
`ViewState::selection` of every hidden player and clears `active_player`
when it is hidden, before `AppUi::keyboard` runs, so Delete never removes
an entry the operator cannot see, whatever hid the player. `file_drops`'s
fallback (no pointer position) is the last player used if shown, else the
first shown player; with none shown the audio files are refused with
`notice-drop-no-player` (playlist files are still imported).

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
   are dimmed and inert. A CUE dimmed for want of a Cue route
   (`OutputsConfig::player_has_cue`) takes `tip-cue-no-output` as its
   tooltip, and the row and cart menus' *Pre-listen on CUE* items say the
   same when disabled; an on-air Cue
   (`fp_model::cue_on_air::cue_is_on_air`) takes `tip-cue-on-air`. The
   model's on-air pre-listen request (`AppState::cue_on_air.request`) opens
   `ui::cue_on_air::show`, a modal whose answer is `Command::ForceCue` or
   `Command::CancelCueOnAir`; `ViewState::cue_answered` hides it until the
   snapshot drops it, and `AppUi::keyboard` gives Esc and Enter to Cancel.
   CA5's notice (`AppState::cue_on_air.notice`) goes to the status bar once
   per id (`ViewState::cue_notice_seen`);
5. requests a repaint: continuously while anything plays, fades or cues
   (`view::animating`, which includes a paused CUE so that its Pause blinks),
   otherwise every 100 ms for the clock. `widgets::{blink, paused_style}` give
   the amber blink of a paused Pause tile to the player and the CUE window.

The UI keeps only **view state**: selection, drag target,
the column edge being dragged (`LiveResize`), the Settings section. Everything else comes
from the snapshot.

`Controller` is implemented by `ConductorHandle`. The tests use a fake that
applies commands with `fp_model::apply` and records them and their refusals,
so the UI is tested end to end without audio.

## Analysis activity

Main spec §6.1. `main` hands `Services::activity()` (an `ActivityCell`,
`Arc<ArcSwap<AnalysisActivity>>`, `fp-app/src/activity.rs`) to
`AppUi::with_analysis_activity`; without it the cell stays idle. Each frame
loads it once (`load_full`, never a lock) into `Scene::analysis`. It feeds:

- the status bar: `analysis_status` gives "Analysing N tracks" (or
  `status-measuring` with only loudness-only jobs) and a tooltip with the
  running files (`tip-analysis-running`, one `• name` per line), the waiting
  count, the pass's progress and the loudness-only jobs. It is drawn left of
  the legend in the right-to-left group. `status_bar` budgets the room left
  of the platform label: an alert or notice on the left keeps its full width
  first (the shortcut hints may be cut to nothing), then the analysis item,
  then the legend, which stops at the first item that does not fit
  (`legend_width`), so items give way from the inside out and nothing leaves
  the bar at narrow widths or in long languages, busy or idle;
- the row flags: `view::analysis_flag` gives a waveform glyph for a running
  track (`flag-analysis-running`), the hourglass for a queued one
  (`flag-analysis-queued`) and, as before, for one not analysed and not
  queued (`flag-analysis-pending`), in the slot right of the title;
- the row menu's Re-analyse (above) and Settings → Analysis, whose
  **Re-analyse all tracks** is disabled and reads `settings-reanalyse-running`
  (done of total) while `AnalysisActivity::pass` is set.

The cell changes only when the services thread stores a new snapshot; the
idle repaint (100 ms) picks it up. Tests build a cell with
`support::activity_cell` and pass it through `harness_from`.

## Language switching

When `config.ui.language` changes (Settings → Players → Language), `AppUi`
rebuilds its `I18n` on the next frame. The first frame only records the
language the interface was built with.

## Marker editing

`wave_panel.rs::edit_markers` handles marker editing on the waveform:

- the context menu remembers the time where it was opened
  (`ViewState::wave_menu`, keyed by `WaveKey` (`Player(id)` or `Cue(id)`), so a player and its CUE window keep their own menu point and drag) and sends `SetMarker` or `ResetMarkers`;
- with Alt held, handles are drawn on the markers. A drag picks the marker
  nearest to the press origin (`ViewState::marker_drag`, keyed by `WaveKey` too) and sends a single
  `SetMarker` on release. A waveform click never seeks while Alt is held.
- when `MarkerFractions::ignored` is set (`!players.use_cue_markers`),
  `widgets::cue_edge_look` turns off the head and tail shading and the
  cue-in and cue-out lines are drawn at `theme::CUE_EDGE_IGNORED_ALPHA`.
  Alt-drag editing is unchanged. The countdowns, the duration column and the
  playlist totals use `Track::play_range`.
- Both waveforms place the markers with the pure `view::marker_fractions`;
  the CUE passes `ignored = true`, since a CUE plays the whole file.

## Track table layout, columns and follow

**Columns.** The table draws `config.ui.table_columns` (repaired with
`normalize_columns` on every frame, so a list set without `Config::validate`
still has Title and Duration). Each cell is a `match` on `TableColumn` in
`track_table`; the text of the plain columns is `view::cell_text`.
A row whose file is missing or unreadable starts its popup with the reason
(`Scene::file_reason`, without the path: the popup has its own Path row). A
cart on the cartwall gives the reason with the path (`Scene::file_tip`).

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

**Live resize (O16).** `egui_extras` 0.37 recomputes only the dragged column,
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
extension, then the sample rate and the bit depth when known. Under the
fields, a rule and the shared gain breakdown (`loudness_edit::info_block`
over `view::gain_info`, for `Holder::PlayerMain` of the list's player so a
bit-perfect player reads "none, bit-perfect"; loudness normalization spec
N19 and J1, see "Gain breakdown" below).

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

**Playlist keys.** `ui/playlist_keys.rs`, called at the end of `AppUi::keyboard`
(so every modal, a focused text field and a shortcut that took the key have
already returned or consumed it): Up and Down move `ViewState::selection` of the
active player (else the first shown player), scrolling the row with
`FollowScroll` (`TableScroll::reveal_align` picks the nearest edge); Enter
sends `SetNext`, and a second Enter on the same entry within egui's
`input_options.max_double_click_delay` sends `Play`. `ViewState::enter_armed`
holds the first Enter; another key, a click or a selection change clears it.
Nothing acts while a widget has the focus, and every command goes through
`command_available`.

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
`ViewState::wave_zoom` (`WaveZooms`) keeps a `WaveZoom` per zoomed waveform,
keyed by `WaveKey` (view, the entry it belongs to, when it was last moved);
`WaveZooms::get` returns it only for the entry it was made on, so no entry
means the full view. The
wheel is read from the frame's `MouseWheel` events while the waveform is
hovered, and the frame's scroll delta is then cleared so no scroll area
moves too. `widgets::waveform` reports a click's seek target and a drag's
sideways movement (`WaveOutput { response, seek, pan_dx }`); egui's click rule
is the drag threshold, so only `Response::clicked()` seeks and a drag never
does. Every waveform is seekable. On a stopped player the click sends
`Command::Seek`, which the model turns into the pending start (rule 3a), and
`view::player_view` shows that start as the stopped player's position; the
zoom of a stopped player still does not follow it. The player pans a zoomed view with `WaveView::pan` (a drag without zoom
does nothing). A held pan drag is flagged in egui temp data keyed on the
waveform id; `widgets::pan_dragging` reads it so the view does not follow the
playhead meanwhile. Alt-drag (markers) and a drag that starts under the
shield report no pan. The wheel rules are pure: `wave_view::wheel_notches`
converts lines, points and pages to notches and `WaveView::wheel` zooms or
pans by them. Each `WaveKey` has its own egui id (`WaveKey::id`), so the
memoised columns and the pan-drag flag of a player and its CUE never mix.

## CUE window

`ui/cue_window.rs` draws one window per player with a running CUE, from
`view::cue_window_view` (title, artist, elapsed, remaining, `paused`, the
position fraction and `can_load_next`). Its waveform is the shared panel
(`WaveKey::Cue(player)`): `cue_window_view` gives it `markers`
(`marker_fractions` on the whole file, cue edges dimmed), `intro` (never
blinking), `outro` (to the end of the file) and `mix_active` (the player's
mode). `show_all` receives `&mut ViewState`; for a player without a CUE,
`forget` drops its CUE key's zoom, menu point and marker drag. Its waveform, seek range and
remaining time run to the end of the file, because a CUE plays the whole
file. It only sends commands: `SeekCue` (a click on the waveform),
`SetCuePaused`, `CueToNext` (Set as next) and `SetCue(player, false)` (Stop
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
OFL) is embedded, subset to Latin, Latin Extended-A and -B, general
punctuation, currency signs, arrows and mathematical operators (the static
TTFs of Inter 4.1 through `fonttools subset` with
`--unicodes=U+0000-024F,U+0259,U+02B0-036F,U+1E9E,U+2000-206F,U+20A0-20C0,U+2113,U+2122,U+2190-21FF,U+2200-22FF,U+FEFF,U+FFFD --layout-features=*`), and Phosphor icons come from `egui-phosphor`, regular and
fill. The static TTFs come from the Inter 4.1 release archive,
<https://github.com/rsms/inter/releases/download/v4.1/Inter-4.1.zip>:
`extras/ttf/Inter-Regular.ttf`, `extras/ttf/Inter-Medium.ttf` and
`extras/ttf/Inter-SemiBold.ttf` become `assets/fonts/Inter-400.ttf`,
`Inter-500.ttf` and `Inter-600.ttf`. Their name table says version 4.001
(`Version 4.001;git-9221beed3`): that is release 4.1, not an older font. Waveform colours are a named palette (`WAVE_PALETTE`); an unknown name
falls back to Slate. Inter's digits are proportional, so times are painted
with `widgets::paint_tabular`/`tabular_label`, which centre every digit in a
cell as wide as the widest one: a countdown keeps its width as it runs. Fonts are installed on the first frame, and drawing starts
on the next one, when they are bound.

### Times

Every time that changes while it is shown keeps a steady width: it is drawn in
equal digit cells with `widgets::paint_tabular`, `paint_tabular_right` (the
text ends at a given x) or `tabular_label`. The sites are the player's
elapsed, total and remaining times and the big countdown, the cart countdown,
the waveform hover time, the marker-drag time, the CUE window times and the
top-bar clock. The intro and outro badges (`widgets::time_badge`) have a value
box sized for `00.0` (`time_badge_width`), so the badge does not change width
from 9.9 to 10.0.

## Window decorations

On Linux, `fp-app` enables winit's `wayland-csd-adwaita` feature. `eframe` is
built without its default features, which would otherwise bring it. A Wayland
compositor without server-side decorations (GNOME) leaves the frame to the
client, and without that feature winit draws a bare fallback frame with
non-standard buttons. With it, the title bar has the usual minimise, maximise
and close buttons. Windows and macOS use their native frames.

## Window system (Linux)

winit 0.30 reports files dragged from a file manager (`HoveredFile`,
`DroppedFile`) on X11, Windows and macOS, but not on Wayland. So on Linux
`main` passes eframe an `event_loop_builder` hook that picks the window
system from `ui.window_system` (`fp_app::window_system::choose`): `Auto`, the
default, takes X11 whenever `DISPLAY` is set, which on a Wayland desktop means
XWayland; `Wayland` keeps the native Wayland window (sharper on fractionally
scaled displays, no file drops). A choice that is not available leaves it to
winit. The choice is logged at start.

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

## Settings pending

`ui/pending.rs` turns `fp_model::live::pending` and `interruptions` into
text (`describe`, `describe_failure`, `target_label`; unit-tested) and draws
two modals in the close guard's style: the pending panel (`show_panel`,
opened from the top bar's **Settings pending** pill, whose tooltip lists the
same lines) and Apply now's confirmation (`show_confirm`). They are separate
windows: while the confirmation is open the panel is not drawn. **Apply now**
in the Settings footer sets `ViewState::apply_now_requested`, handled
later in the same frame; in the panel it is handled on the spot, so the
confirmation replaces the panel without a blank frame. Either way it sends
`Command::ApplySettingsNow` at once when `interruptions` is empty, else
sets `confirm_apply_now`. The confirmation also lists, in amber, the
players whose fader the forced items would lock below 100 %
(`apply-now-fader-reset` over `fp_model::faders_reset_by`, BF8). The
confirmation closes and sends
the command by itself when the list becomes empty; Cancel, `Esc` (answered
in `keyboard`, before Settings sees it) and the backdrop cancel it. Device
names come from `SettingsState::device_label` (the lists Settings read on
its helper thread; the id until then). The status bar's label follows
`state.live.audio_system_in_use`.

## Settings window

`settings::window_size` gives the one size of the window: 900 × 640, clamped to
the main window minus a margin (never below 320 × 300). It does not change when
a section is selected. The section header (title and **Restore defaults**) is
fixed; the body below it sits in `ScrollArea::both`. Rows use
`labelled_row`, with a label column of `LABEL_WIDTH` (180 px), so every section
shares one grid. Every drop-down goes through `settings::fitted_combo`: it is
at least its usual width (260 px for the language and audio system lists,
egui's default for the others, the room beside the bus label for a device
picker) and wider for a longer choice, but never wider than the column; a
narrower column truncates the shown choice (`ComboBox::truncate`), the hover
text gives it whole, and the open list always shows every choice in full.
The column is measured against the body's right edge (`column_width`, noted
by `set_body_right` before the rows), not `available_width`: a row whose
fixed parts run past the edge (a slider, the test buttons) makes the body
scroll sideways but does not widen the lists of the other rows. A route
picker keeps the bus label, device picker, channel pair and test button on
one line while the device picker gets at least `DEVICE_MIN_WIDTH`; a
narrower column moves the channel pair and test button to a line of their
own, and one under the other when even that does not fit, so the device
picker never runs over the label column. `tests/settings.rs` checks every
list on Audio outputs in 700 and 560 px windows and the language list at
560 px; below about 560 px the label column itself leaves the control
column almost no room, and at the 420 px minimum window it no longer fits.
The footer spans the window width and holds the pending notice and **Apply now** while an output change waits (`SettingsDeps::{pending, can_apply}`). A transient notice (`SettingsDeps::notice`) briefly takes the place of the pending text; **Apply now** stays.

Settings → Remote keeps what is being typed (`RemoteState::drafts` for text,
`RemoteState::numbers` for the drag values, which do not wait for Enter), so
opening another section applies it through `remote::flush`. The count of
outdated tracks (Settings → Analysis, and the start-up notice) comes from
`AppUi`'s `services::OutdatedCount`: the library is scanned once per model
snapshot, not on every frame (`SettingsDeps::outdated`).

Two sliders apply sizes that remove things on the way down: the cart page grid
(Settings → Cartwall, `Command::ResizeCartPage`, which truncates the trailing
carts) and the number of players (`settings::count_slider`,
`Command::SetPlayerCount`). Their number boxes use
`Slider::update_while_editing(false)`, so a typed number applies once, on Enter
or when the box loses focus (Escape cancels it), never its first digit: typing
12 over 4 would otherwise go through 1. Both also keep a draft while the
slider is dragged (`CartsState::grid_draft`, `SettingsState::player_count_draft`)
and send the command once, on release: dragging down to 1 and back would
otherwise remove the trailing carts or players on the way and bring them back
empty. Each arrow-key step applies at once. The other settings sliders apply every
value as it is typed or dragged, which only rewrites the configuration.

## Loudness normalization

The interface shows and edits [loudness normalization](../superpowers/specs/2026-10-08-loudness-normalization-design.md)
(spec §8) from the snapshot only; the gain itself is the model's
(`fp_model::loudness`) and the engine's.

- **Gain breakdown (J1).** `view::gain_info(track, config, bit_perfect,
  actual) -> Vec<InfoLine>` is the one pure builder of everything that
  explains a track's gain, and `loudness_edit::info_block` the one renderer.
  An `InfoLine` has a `kind` (`Measured`, `TruePeak`, `Target`, `AutoGain`,
  `Trim`, `Wanted`, `Limit`, `Note`, `Result`), a label `Message` and an
  `InfoValue` (`Text` for a figure with its unit, `Message` for a phrase).
  The order is fixed, `Result` is always the last line, and every figure
  comes from `fp_model::normalization`, `trim_in_use` and `wanted_gain_db`.
  `actual` is the `Normalization` of a source on air (the badge passes it for
  every status, the row tooltip of the entry on air too): when the current
  settings would give another (N11) the lines are a `Note`
  (`tip-gain-earlier-settings`) and the gain it kept. `info_block` draws
  label left and value right with tabular digits (`widgets::paint_tabular_right`),
  notes across the width, and `Result` under a rule in semibold; a line too
  long for one row wraps (the label to at most half the width), so a long
  language or a narrow window keep every line inside. Each row is one
  accessibility node named `Label: value`; the gain badge, a tile named
  `widgets::info_text` and showing the block with `on_hover_ui`, is named by all the
  lines. The same block is drawn by the badge tooltip, the Loudness…
  popup (under its controls, so the breakdown growing never moves the slider), the cart editor and the playlist row tooltip,
  so they cannot drift apart.
- **Messages.** The pure functions in `view.rs` return a `view::Message`
  (a Fluent key and its arguments, already formatted with
  `view::signed_db` and one decimal with the true minus sign); the painting
  code translates them with `widgets::tr_message`. A value that is not a
  number reads `0.0` (a gain) or `—`.
- **Gain badge (N18).** `view::gain_badge(&Normalization, &LoudnessConfig,
  &Track)` gives the badge of the player's `PlayerState::on_air` while the
  player is not stopped (`PlayerView::gain`): the signed gain (`Applied`,
  tone `Normal`, with the limit that held a boost back), **0.0 dB**
  amber (`NotMeasured`, `Excluded`, and `BitPerfect` while
  `normalize` is on; with it off a bit-perfect player plays at unity gain
  anyway and shows nothing), and nothing for `Off` and `Silent`. When
  `fp_model::normalization(config, track, false)` now differs from `on_air`
  (the settings, the measurement or the edit changed since the track
  started, N11), the tooltip is `tip-gain-earlier-settings` and the gain it kept: the
  current target, measurement and trim did not give that gain, so they are
  not quoted beside it. `player.rs`
  paints it as a tile left of **BP**, named by the plain text of the gain breakdown. It shows
  the gain the source was started with: for a scheduled transition, the
  one the engine reports in `TransitionStarted`, so a re-preload that
  races a transition the engine had already committed does not change it.
- **Loudness… popup (N20).** The waveform menu (`wave_panel::edit_markers`,
  so the player column and the CUE window) sets `ViewState::loudness_popup`
  to the track's id. `loudness_edit::popup` draws a non-modal window for
  that track whatever the players show next, and clears the field when it
  is closed or the track leaves the library. The trim slider's rail senses
  drags only, so its double-click (reset to 0) is read from the pointer
  while the pointer is on it, and egui's triple click counts too (a
  double-click within 0.6 s of the click that opened the popup).
- **Cart editor (N21).** Settings → Cartwall draws `loudness_edit::exclude`
  and `loudness_edit::trim` as two rows of the cart's editor, the
  gain breakdown (`info_block`) above them.
- **Settings → Loudness (N22)** is `settings/loudness.rs`, restorable
  (`SettingsSection::Loudness`). The count of tracks waiting for a
  measurement comes from `services::UnmeasuredCount`
  (`fp_model::loudness_unmeasured`, once per snapshot, as
  `OutdatedCount`).

## Panic isolation

`Shell::ui` runs `AppUi::ui` inside `catch_unwind`. After a panic it draws only
a banner with **Restart interface**, which calls `AppUi::reset_view`.
The `test-hooks` feature adds `AppUi::fail_next_frame` and
`Services::fail_next_step` for tests. Release builds do not contain them.

## Internationalisation

- Every visible string is a Fluent message in `locales/en-US/main.ftl` (the
  source) and in every other `locales/<tag>/main.ftl`. Every locale,
  English and Spanish included, was produced with AI. A new or changed
  string goes into every locale file in the same change.
- `i18n::LOCALES` is the registry: one `Locale` per embedded file, with its
  BCP-47 `tag`, its `name` in its own language (what the Settings drop-down
  shows) and its `source` (`include_str!`). en-US comes first, the rest are ordered by `name`; the
  Settings drop-down lists "System" and then the registry in that order.
- `resolve` picks the locale: the configured tag when it negotiates to a
  registered locale, else the OS locale (`sys-locale`), else en-US; an
  unknown or empty configured tag does not hide the OS locale, and Settings
  shows it as "System". `negotiate` matches one tag: the same tag
  (case-insensitive, `_` read as `-`, a POSIX `.UTF-8` or `@modifier`
  suffix ignored), else the first locale with the same language subtag
  (`fr-CA` gives `fr-FR`, `pt-BR` gives `pt-PT`, `ca-ES-valencia` gives
  `ca-ES`); `C`, `POSIX` and an empty tag match nothing. A message missing
  from a locale falls back to en-US. A file with a syntax error keeps its
  valid messages and logs a warning (`bundle_from`).
- The About window shows the same disclaimer (`about-disclaimer`) in every
  locale: it covers the AI-assisted development, the "as is" provision and the
  use at one's own risk, so no per-language translation notice exists.
- Plural categories come from CLDR through `fluent-bundle`
  (`intl_pluralrules`): Polish selects `one`, `few`, `many` and `other`.
  Every plural variant shows the number through its placeable
  (`{ $count } track`), never a literal `1` or a word: in French 0 is also
  `one`, so a literal would show "1 piste" for an empty playlist.
  `tests/i18n.rs` formats every number-selecting en-US message in every
  locale with 0, 1, 2 and 5 and checks the output shows exactly that number.
- `tests/i18n.rs` checks that every file under `locales/` is registered (and
  every entry has its file), that the registry is ordered, that every locale
  parses cleanly and has exactly the en-US message ids, and that every
  message uses the same variables (and message and term references) as in
  en-US, so a misspelt or dropped placeable fails.
  `tests/theme.rs` checks that each embedded Inter weight has a glyph for
  every character any locale file shows, and for the letters of the planned
  European languages.
- Arguments use `tr_args(key, &[("name", value.into())])`.
- To add a language: add `locales/<tag>/main.ftl` with every en-US message
  translated, and one `Locale` entry in `LOCALES` at its place by name. Nothing
  else changes: Settings, negotiation and About follow the registry.
