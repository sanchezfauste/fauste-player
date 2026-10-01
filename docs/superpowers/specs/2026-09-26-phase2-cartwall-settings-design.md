# Fauste Player — Phase 2 design: cartwall, full Settings, playlist files and marker editing

- **Status:** agreed scope from the main spec §13 (Phase 2); details decided here.
- **Parent spec:** [`2026-09-25-fauste-player-design.md`](2026-09-25-fauste-player-design.md). Everything there still holds. This document adds the Phase 2 parts, with section numbers prefixed `P2`.
- **Design source:** the v3 design (cartwall strip under the players; Settings sections "Cartwall" and "Keyboard shortcuts").
- **Exit criterion (spec §13):** parity with the v3 design feature set.

## P2.1 Scope

1. **Cartwall:**
   - cart pages with tabs, a collapsible strip and a configurable grid (default 2 × 8 = 16 carts);
   - per-cart name, file, kind, loop and exclusive settings;
   - carts overlap by default;
   - progress bar, countdown and playing dot on page tabs;
   - its own Main and Cue routes, and a Cue pre-listen.
2. **Settings, full set:**
   - a Cartwall section (pages, grid, cart editor, import/export);
   - a Keyboard shortcuts section (remappable);
   - a language selector;
   - cartwall outputs in Audio outputs;
   - M3U/PLS import and M3U export in Playlists.
3. **Playlist files:**
   - a tolerant M3U / M3U8 / PLS import;
   - M3U8 export;
   - cart page import/export (JSON);
   - fuzz targets for every parser.
4. **Marker editing:** set, move and clear intro end, outro start and MIX (segue start) on the waveform. Also cue-in and cue-out, which the engine already honours.
5. **INTRO tag convention:** an `INTRO` tag (seconds) sets the intro end.

Out of scope, still later phases: native backends (Phase 3), bit-perfect (Phase 4), packaging (Phase 5).

## P2.2 Names

| Concept | Name in code | Notes |
|---|---|---|
| The strip of buttons | `Cartwall` | broadcast "cart wall" |
| A tab of buttons | `CartPage` (`CartPageId`) | |
| A button | `Cart` (`CartId`, stable across edits) | from broadcast tape cartridges |
| Kind of cart | `CartKind::{Jingle, Effect, Spot}` | "spot" is the broadcast term for a commercial; a cart without a file is *empty* |
| Stops other carts | `exclusive` | |
| Repeats | `looped` | |
| Keyboard binding | `Shortcut { action, chord }`, `KeyChord { key, ctrl, alt, shift, command }` | |

## P2.3 Cartwall model (`fp-model`)

```rust
pub struct Cart {
    pub id: CartId,
    pub name: String,
    pub track: Option<TrackId>,   // library track, analysed like any other
    pub kind: CartKind,
    pub looped: bool,
    pub exclusive: bool,
}
pub struct CartPage { pub id: CartPageId, pub name: String, pub rows: u16, pub cols: u16, pub carts: Vec<Cart> } // len == rows*cols
pub struct Cartwall {
    pub pages: Vec<CartPage>,
    pub playing: Vec<PlayingCart>,      // { cart, looped } — never persisted
    pub cue: Option<CartId>,            // pre-listen on the cartwall Cue route
}
```

`AppState` gains `cartwall: Cartwall`. The grid size is per page and
validated to 1 … `limits.max_cart_rows` (default 8) × 1 …
`limits.max_cart_cols` (default 16). A new page uses `cartwall.default_rows`
(2) × `default_cols` (8) from config. Cart files are ordinary library tracks,
so analysis, file state and cue-in/cue-out apply to them unchanged.

### Rules

- **C1. Fire.** Firing a cart that has a playable track starts it on the
  cartwall Main route, from its cue-in. It keeps playing alongside everything
  else: carts overlap by default.
- **C2. Toggle.** Firing a playing cart stops it, with the de-click ramp.
- **C3. Exclusive.** Firing an exclusive cart first stops every other playing
  cart on every page.
- **C4. Loop.** A looped cart restarts at its cue-in when it reaches its
  cue-out, without a gap, until it is stopped.
- **C5. End.** A cart that is not looped stops at its cue-out and is removed
  from `playing` (`EngineEvent::CartEnded`).
- **C6. Unavailable.** An empty cart, or one whose file is Missing or
  Unreadable, does nothing when fired. The button shows why.
- **C7. No resurrection.** Playing carts are never saved. After a restart
  nothing plays.
- **C8. Editing.** Changing a playing cart's file, or deleting its page,
  stops it first. The last page cannot be deleted (`ModelError::LastCartPage`).
- **C9. Cue.** Pre-listening a cart plays it on the cartwall Cue route. Only
  one cart pre-listens at a time, and it never reaches Main.
- **C10. Stop all.** `StopAllCarts` stops every playing cart, and the cue.
- **C11. Atomic edits.** `EditCartPage` (name and grid) and `EditCart` (a
  cart's fields and its file: keep, another library track, or none) check
  everything first and apply all or nothing. Keeping the same track is not
  a new file, so it does not stop the cart (C8). The remote API edits pages
  and carts through them.

### Commands, actions and events

| Kind | Additions |
|---|---|
| `Command` | `FireCart(CartId)`, `StopCart(CartId)`, `StopAllCarts`, `CueCart(CartId)` (toggle), `CreateCartPage { name }`, `RenameCartPage`, `DeleteCartPage`, `ResizeCartPage { page, rows, cols }`, `SetCart { page, index, cart: CartEdit }`, `EditCartPage { page, name, grid }`, `EditCart { page, index, edit, file: CartFileChange }` (C11), `AssignCartFile { page, index, path }` (adds the track to the library), `ImportCartPage(CartPageImport)`, `SetMarker { track, kind, secs: Option<f64> }`, `ResetMarkers { track }`, `CreatePlaylistFromPaths { name, paths }` |
| `EngineAction` | `StartCart { cart, request: SourceRequest, until_secs: f64, looped: bool }`, `StopCart { cart }`, `StartCartCue { cart, request }`, `StopCartCue` |
| `EngineEvent` | `CartEnded { cart }`, `CartFailed { cart }`, `CartCueEnded` |

`ResizeCartPage` keeps existing carts by position (row-major). Shrinking is
refused if it would drop carts that have a file (`ModelError::CartsWouldBeLost`).

## P2.4 Cartwall engine (`fp-engine`)

- A dedicated **cartwall worker** (`fp-cartwall`), built like a player
  worker, decodes every cart source. Carts are sources in the mixer like any
  other, so there is no new real-time code.
- **Routes:** `config.outputs.cartwall: { main: Option<Route>, cue: Option<Route> }`.
  - Main falls back to the default output, as for players.
  - A Cue never falls back: a Cue route to a backend this machine does not
    have, or one equal to Main, means no cue. This rule applies to player Cue
    routes too.
  - Mixer capacity grows with the slots in use on the bus, and before every
    fire.
- **Start and end:** the source is opened at cue-in and started as soon as it
  is ready. The worker bounds it at `until` (cue-out), so a cart that is not
  looped ends exactly there (`Finished` becomes `CartEnded`); no `StopAt` is
  needed.
- **Loop:** at `until` (or at the end of the file when the length is unknown)
  the worker reopens the file at cue-in and keeps filling the same ring. A
  resampled stream is opened with an aligned pre-roll whose warm-up output is
  dropped. The first sample therefore matches continuous playback, at every
  cue-in and at every loop point.
- **Stop:** a de-click ramp (also for a start that is requested but not yet
  confirmed), then `Detach`. The slot is released when the mixer returns it.
- **Telemetry:** `Telemetry.carts: Vec<(CartId, CartTelemetry { position_secs, peak })>`
  and `cart_cue`. A looped cart's position wraps on the pass length the
  worker measured.
- **Known limit:** a marker change on a cart's track applies from the next
  fire.

## P2.5 Keyboard shortcuts

- `config.shortcuts: Vec<Shortcut>`. `ShortcutAction` has these variants:
  - `PlayPlayer(n)`, `PausePlayer(n)`, `StopPlayer(n)`, `FadeStopPlayer(n)`,
    `CuePlayer(n)`;
  - `FireCart(slot)` (a slot of the page shown), `StopAllCarts`;
  - `ToggleCartwall`, `NextCartPage`, `PreviousCartPage`.

  Here `n` and `slot` are 1-based positions, not ids, so a shortcut survives
  player and page changes.
- **Defaults:** `1`…`9` play players 1–9, `F1`…`F12` fire carts 1–12 of the
  page shown, and `Ctrl+Space` stops all carts.
- `Delete`/`Backspace` (remove the selection) and `Esc` (close) are fixed and
  cannot be rebound.
- **Settings → Keyboard shortcuts:** one row per action with its chord. Click
  a row and press a key to rebind it. A chord already bound elsewhere is
  shown as a conflict, and saving it moves the chord to the new action.
  **Reset to defaults** restores the list.
- Chords are ignored while a text field has focus. Key repeats never fire an
  action (the Phase 1 rule).

## P2.6 Language selector

**Settings → Players → Language:** System, English, Español. It is stored in
`config.ui.language`, and the interface switches on the next frame.

## P2.7 Playlist files (`fp-store::playlist_io`)

- **Import:** `parse_playlist(bytes, source_path, limits) -> Result<Vec<ImportedEntry { path, title, duration_secs }>, PlaylistFileError>`.
  - The format is detected by extension (`.m3u`, `.m3u8`, `.pls`), then by
    content.
  - **M3U/M3U8:** `#EXTM3U` is optional. `#EXTINF:<secs>,<Artist - Title>`
    is used as a hint. Blank lines, comments and unknown directives are
    ignored. `file://` URLs are decoded. Relative paths are resolved against
    the playlist's folder. `http(s)://` entries are skipped and counted.
  - **Encoding:** M3U8 is UTF-8. M3U is UTF-8 when valid, otherwise
    Windows-1252. On Unix, non-UTF-8 byte paths are kept as raw `OsString`.
  - **PLS:** `[playlist]` with `FileN`, `TitleN` and `LengthN` in any order;
    `NumberOfEntries` is only a hint.
  - Input larger than `limits.max_playlist_file_bytes` is refused. The parser
    never panics (fuzzed).
- **Import UI:** **Settings → Playlists → Import M3U / PLS…** creates a new
  playlist named after the file (`CreatePlaylistFromPaths`). Missing files are
  added and shown as unavailable, so nothing is silently dropped. A notice
  reports skipped streams.
- **Export:** every playlist row has **M3U** (save dialog). It writes M3U8
  (`#EXTM3U`, then `#EXTINF:<rounded secs>,<Artist - Title>` and the
  absolute path for each entry).
- **Cart page files:** `*.cartpage.json`, with `{ "format": "fauste-cart-page",
  "version": 1, "name", "rows", "cols", "carts": [{ "position", "name", "kind",
  "file", "loop", "exclusive" }] }`. Import validates the whole document,
  clamps the grid, and resolves relative files against the JSON's folder.
  Size is limited by `max_playlist_file_bytes`.
- **Persistence:** `data/carts.json` (`CartsDoc`, `schema_version` 1) holds
  pages and carts. Their tracks live in `playlists.json`'s library, which
  becomes the shared library (no schema change needed). The session stores
  `cartwall: { open, page }`.

## P2.8 Marker editing

- The **waveform context menu** (right-click) of a player offers:
  - *Set cue in here*, *Set intro end here*, *Set outro start here*,
    *Set MIX point here*, *Set cue out here*;
  - *Reset markers to automatic*.
- **Modifier + drag:** hold `Alt` (`Option` on macOS) over the waveform to
  show a handle on every marker. Drag a handle to move that marker; it becomes
  manual. The time under the pointer is shown while dragging, and the value
  is committed on release (one `SetMarker`).
- **Validation (model):**
  - `cue_in < cue_out`;
  - intro end, outro start and MIX lie within `[cue_in, cue_out]`.

  A value outside is clamped, and an impossible one is refused
  (`ModelError::InvalidMarker`).
- **Reset:** manual markers are removed and the track is analysed again
  (`analyzed = false`). The automatic values come back from the cache or from
  a new analysis.
- **Live effect:** a change to the current track reschedules its transition
  through the existing reconcile path (spec §6, "Playability before analysis").
- **INTRO tag:** analysis reads the tag, and the result is stored as an
  automatic intro end, so a manual one still wins. Sources:
  - ID3v2 `TXXX:INTRO`;
  - a Vorbis/FLAC comment `INTRO`;
  - an MP4 freeform `----:com.apple.iTunes:INTRO`;
  - an APE item `INTRO`.

  The value is seconds (`12.5`) or `m:ss(.f)`.

## P2.9 Cartwall UI (v3 layout)

- The strip sits under the players:
  - a header with a caret and **CARTWALL** (click to collapse), the page tabs
    (red dot when a cart plays on that page) and the hint *Click to fire ·
    click again to stop*;
  - a grid of `cols` columns, each button 40 px tall.
- **Button:**
  - a kind dot (Jingle accent, Effect amber, Spot neutral), the name, the
    kind label, and the time (`-mm:ss` countdown while playing);
  - a red border and a translucent red progress bar that shrinks while
    playing;
  - a warning style when the file is unavailable, and a dimmed "Empty" style
    when there is none;
  - right-click opens: *Pre-listen on CUE*, *Stop*, *Edit…* (opens Settings on
    that cart).
- **Settings → Cartwall:**
  - page tabs, **New page**, **Import…**, **Export…** and **Delete page**
    (refused for the last page);
  - the page name, and rows × cols;
  - the grid, where a click selects a cart;
  - the cart editor: name, file (**Choose…**, and duration shown), kind,
    **Loop**, and **Stop other carts when fired**.
- **Settings → Audio outputs** gains a **Cartwall** row with Main and Cue
  pickers and test buttons.

## P2.10 Security and robustness

- Fuzz targets in `fuzz/` (cargo-fuzz, nightly):
  - `m3u`, `pls`;
  - `cart_page`;
  - `store_documents` (each JSON document, through the lenient loaders).

  A scheduled CI job runs each target for 5 minutes per night.
- Imported paths are only read. Export writes only where the user chose in a
  save dialog.

## P2.11 Plans

| Plan | Content |
|---|---|
| Phase 2 · plan 1 | Model and store: cartwall state and rules C1–C10, markers commands and validation, shortcuts config, language, `carts.json`, playlist and cart-page file formats, fuzz targets and CI job |
| Phase 2 · plan 2 | Engine and analysis: cartwall worker, routes, loops, cart telemetry, cart cue; INTRO tag |
| Phase 2 · plan 3 | UI: cartwall strip, Settings Cartwall and Shortcuts sections, language selector, cartwall outputs, M3U import/export, marker editing on the waveform, docs |
