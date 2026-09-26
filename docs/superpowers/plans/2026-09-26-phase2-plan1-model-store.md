# Phase 2 · Plan 1 — Model and store: cartwall, markers, shortcuts, playlist files

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans (or superpowers:subagent-driven-development) to implement this plan task by task.

**Goal:** Give `fp-model` and `fp-store` everything Phase 2 needs, with no UI and no audio:
- the cartwall state and its rules C1–C10;
- manual marker commands;
- the shortcut and language configuration;
- `carts.json` persistence;
- M3U/M3U8/PLS and cart-page file formats, with fuzz targets.

**Architecture:**
- `fp-model` gains a `cartwall` module (types and pure rules) and a `shortcuts` module (config types and defaults).
- `fp-store` gains `playlist_io` (parsers and writers) and the `CartsDoc` document.
- Fuzz targets live in a new top-level `fuzz/` crate that is excluded from the workspace.

**Spec:** [`docs/superpowers/specs/2026-09-26-phase2-cartwall-settings-design.md`](../specs/2026-09-26-phase2-cartwall-settings-design.md) (P2.2, P2.3, P2.5, P2.6, P2.7, P2.8, P2.10), within the parent spec.

## Global Constraints

- Rules carried over: English everywhere; no third-party product names; no `unsafe`; no unwrap, expect or panic outside tests; no hardcoded product limits (new limits go in `Limits` with validation); TDD.
- The model stays pure: no I/O and no threads.
- New persisted shapes get a `schema_version` and a migration path. `carts.json` starts at version 1.
- Library tracks are shared by playlists and carts. A track is dropped only when neither references it.

## Review Focus

1. **A playlist file from another OS** (Windows paths in an M3U read on Linux, a relative path with `..`, a `file://` URL with percent-encoding, a Latin-1 M3U). Every entry must resolve or be reported; there must be no panic.
2. **Removing the last playlist entry that references a track also used by a cart.** The track must stay in the library, and the cart must keep playing its file.
3. **Firing an exclusive cart while a looped cart and a cue play.** Every other playing cart stops; the cue is unaffected.
4. **Shrinking a cart page that has files in the removed cells.** It is refused with a clear error; nothing is lost.
5. **Setting a marker outside the cue range, or cue-in after cue-out.** It is clamped or refused; the scheduling of the current track must not break.

---

### Task 1: Cartwall types and rules (`fp-model`)

- **Ids and types:**
  - `CartId`, `CartPageId` (via `id_type!` and `IdGen`);
  - `CartKind { Jingle, Effect, Spot }`;
  - `Cart`, `CartPage { rows, cols, carts }`, `PlayingCart { cart, looped }`;
  - `Cartwall { pages, playing, cue }`;
  - `CartEdit { name, kind, looped, exclusive }`.
- **Config:**
  - `CartwallConfig { default_rows: 2, default_cols: 8 }`;
  - `Limits.max_cart_rows` (8) and `max_cart_cols` (16), validated.
- **State:** `AppState.cartwall`. A fresh state has one page ("Carts" by default; its name is passed in like the default playlist name) of `default_rows × default_cols` empty carts.
- **Commands:**
  - `FireCart`, `StopCart`, `StopAllCarts`, `CueCart`;
  - `CreateCartPage`, `RenameCartPage`, `DeleteCartPage`, `ResizeCartPage`;
  - `SetCart`, `AssignCartFile` (adds a `Track`), `ClearCartFile`;
  - `ImportCartPage(CartPageImport)`.
- **Engine actions:** `StartCart { cart, request, until_secs, looped }`, `StopCart`, `StartCartCue`, `StopCartCue`.
- **Engine events:** `CartEnded`, `CartFailed`, `CartCueEnded`.
- **Errors:** `UnknownCart`, `UnknownCartPage`, `LastCartPage`, `CartsWouldBeLost`, `CartGridOutOfRange`.
- **Library GC:** `forget_unreferenced` also checks cart references.
- **Tests** (`fp-model/tests/cartwall.rs`), one per rule:
  - `c1_firing_starts_the_cart_from_cue_in_and_overlaps`;
  - `c2_firing_a_playing_cart_stops_it`;
  - `c3_an_exclusive_cart_stops_every_other_cart`;
  - `c4_looped_carts_are_started_looped`;
  - `c5_a_cart_ends_at_cue_out`;
  - `c6_empty_or_unavailable_carts_do_nothing`;
  - `c7_playing_carts_are_not_restored`;
  - `c8_changing_the_file_of_a_playing_cart_stops_it`;
  - `c8_the_last_page_cannot_be_deleted`;
  - `c9_only_one_cart_pre_listens`;
  - `c10_stop_all_stops_carts_and_cue`;
  - `shrinking_a_page_that_would_drop_files_is_refused`;
  - `a_track_used_by_a_cart_survives_playlist_removal` (Review Focus 2);
  - `failed_carts_are_marked_unreadable_and_removed`.
- **Proptest:** for random fire, stop, edit and resize sequences, `playing` only holds carts that exist and have a playable track, and has no duplicates.

### Task 2: Marker commands (`fp-model`)

- `SetMarker { track, kind, secs: Option<f64> }` stores a manual marker; `None` clears it.
- **Validation:** cue-in must be before cue-out (`InvalidMarker`). The other markers are clamped to `[cue_in, cue_out]` (the effective values).
- `ResetMarkers { track }` removes every manual marker and sets `analyzed = false`, so the services thread analyses the track again.
- A change to the current or next track of a player goes through `reconcile` and `plan_for` as usual. The existing preload refresh (a changed cue-in) covers the next track.
- **Tests:**
  - `a_manual_mix_point_changes_the_scheduled_transition`;
  - `cue_in_after_cue_out_is_refused`;
  - `markers_outside_the_cue_range_are_clamped`;
  - `reset_markers_requests_a_new_analysis_and_keeps_nothing_manual`;
  - `analysis_never_overwrites_a_manual_marker` (existing behaviour, extended to the intro end).

### Task 3: Shortcuts and language configuration (`fp-model`)

- `shortcuts` module:
  - `KeyChord { key: String /* egui::Key name */, ctrl, alt, shift, command }` with `Display` (for example `Ctrl+Space`);
  - `ShortcutAction` (spec P2.5);
  - `Shortcut { action, chord }`;
  - `default_shortcuts()`.
- `Config.shortcuts: Vec<Shortcut>`, defaulting to `default_shortcuts()`. Validation drops bindings of unknown keys, and duplicate chords (the first one wins), with warnings.
- `UiConfig.language` already exists; it gets documented values `None`, `"en-US"` and `"es-ES"`.
- `Command::SetShortcut { action, chord: Option<KeyChord> }` moves a chord from any other action to this one; `ResetShortcuts` restores the defaults.
- **Tests:**
  - `default_shortcuts_play_players_and_fire_carts`;
  - `rebinding_a_used_chord_moves_it`;
  - `unknown_keys_are_dropped_with_a_warning`;
  - `duplicate_chords_keep_the_first_binding`.

### Task 4: Persistence of carts and the cartwall session (`fp-store`)

- `CartsDoc { schema_version: 1, pages }` in `data/carts.json`. `AppPaths::carts_file()`.
- `Store::save_carts` and loading with the usual fallback and quarantine.
- `SessionDoc` gains `cartwall: { open, page }`, with `#[serde(default)]` and no schema bump; the existing lenient reading covers it.
- `RestoreParts` gains `cart_pages`. Restore never restores playing carts, and observes cart ids in `IdGen`.
- **Tests:**
  - `carts_round_trip`;
  - `a_corrupt_carts_file_falls_back_to_the_backup`;
  - `missing_carts_file_gives_one_default_page`;
  - `session_without_cartwall_fields_still_loads` (old files).

### Task 5: Playlist and cart-page files (`fp-store::playlist_io`)

- `parse_playlist(bytes: &[u8], source: &Path, limits: &Limits) -> Result<ImportedPlaylist { entries, skipped_streams, warnings }, PlaylistFileError>`.
  - Detect M3U, M3U8 or PLS by extension, then by content.
  - Handle `#EXTINF`, `file://` URLs with percent-decoding, relative paths, Windows separators on Unix (`\` to `/` only when the path is relative), `http(s)` streams (skipped and counted), BOMs, CRLF, UTF-8 with a Windows-1252 fallback, and the size limit.
- `write_m3u8(entries: &[(PathBuf, Option<String>, Option<f64>)]) -> String`.
- `parse_cart_page(bytes, source, limits) -> Result<CartPageImport, CartPageFileError>` and `write_cart_page(page, library) -> String` (format P2.7).
- `Command::CreatePlaylistFromPaths { name, paths }` in `fp-model` (a single command, so the new playlist id stays inside the model).
- **Tests** (`fp-store/tests/playlist_io.rs`):
  - `m3u_with_extinf_and_comments`;
  - `m3u8_utf8_and_bom`;
  - `m3u_latin1_fallback`;
  - `relative_and_parent_paths_resolve_against_the_playlist`;
  - `file_urls_are_percent_decoded`;
  - `windows_relative_paths_on_unix`;
  - `streams_are_skipped_and_counted`;
  - `pls_in_any_order`;
  - `oversized_files_are_refused`;
  - `m3u8_export_round_trips`;
  - `cart_page_round_trips`;
  - `cart_page_with_bad_grid_is_clamped`;
  - `cart_page_with_wrong_format_is_refused`.

### Task 6: Fuzz targets and nightly job

- `fuzz/Cargo.toml` (cargo-fuzz, `libfuzzer-sys`), excluded from the workspace (`exclude = ["fuzz"]`).
- Targets: `m3u`, `pls`, `cart_page` and `store_documents`. The last one feeds the bytes to the `config`, `playlists`, `session` and `carts` loaders through a new `fp_store::fuzz_load_document(kind, bytes)` helper (`#[doc(hidden)]`).
- `.github/workflows/fuzz.yml`: scheduled nightly plus `workflow_dispatch`, on the nightly toolchain, running each target for 300 s. Crashes are uploaded as artefacts.
- A local smoke run: `cargo +nightly fuzz run m3u -- -max_total_time=30`, if nightly is available. Otherwise log a ruling and rely on the CI job.
- Docs: `docs/technical/testing.md` gains a fuzzing section.

### Task 7: Docs and verification

- Update `docs/technical/persistence.md` (carts.json, shortcuts, limits) and `architecture.md` if needed.
- Full checks, a fresh review, fixes, then merge.
