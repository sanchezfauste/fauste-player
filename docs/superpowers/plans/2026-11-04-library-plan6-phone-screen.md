# Track library plan 6 — the phone's library screen Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** On the phone layout the Music tab becomes a Library tab: a
category selector, then the category's track list with search, ticks, a
selection bar, a touch-and-hold menu and a 🎧 pre-listen, with Device music
(today's Music screen) kept for bringing files in.

**Architecture:** One model change: `LibraryRoutes::default_output`
(a config field that is `true` by default on Android) lets the library CUE
play on the default output when it has no route; the model's
`library_has_cue` and the engine's placement of `Holder::LibraryCue` follow
it, so the on-air rules (CA1–CA6) need nothing new. The UI is a new module
`crates/fp-app/src/ui/phone_library.rs` that reuses the window's pieces:
`LibraryUi::rows` (search, scope and rows), `menu::row_menu` (with a phone
scope), `confirm::ask_removal` (LB17) and the library CUE commands. The
Music screen (`music.rs`) becomes the tab's *Device music* page; its
selection bar is shared by both pages.

**Tech Stack:** Rust, egui 0.37 / egui_kittest, Fluent.

**Spec:** `docs/superpowers/specs/2026-11-01-track-library-design.md`
§11.1 (LB38–LB48 and its *Settled details*), with §7.4, §7.5, §8 and
LB17 for the behaviour it reuses; the phone layout spec
`docs/superpowers/specs/2026-10-28-phone-layout-design.md` PL2, PL17–PL19b.

## Global Constraints

- English in every file; UI strings are Fluent messages in
  `crates/fp-app/locales/en-US/main.ftl`, with es-ES written and the other
  nine locales (ca-ES, de-DE, eu-ES, fr-FR, gl-ES, it-IT, nl-NL, pl-PL,
  pt-PT) translated in the same task (`tests/it/i18n.rs` checks parity).
- Never mention other playout or radio-automation products.
- Phone layout only: a tablet and the desktop are unchanged, except the
  config field of Task 1 (whose default is `false` off Android).
- Rule 8: the UI never blocks; the phone screen reads the snapshot and
  sends commands. Rule 10: nothing plays without the operator's 🎧 or Play.
- `unwrap`/`expect`/`panic` denied outside tests; prefer `get`.
- LB28: only visible rows are laid out (`show_rows` or the existing
  virtualised list of `music.rs`).
- Touch targets at least `touch::TARGET` high.
- Android must still build: `cargo ndk -P 29 -t arm64-v8a clippy --locked
  -p fp-mobile -p fp-app --lib -- -D warnings` and
  `scripts/check-mobile-isolation.sh` (run in Task 1 and Task 7).
- UI tests use `egui_kittest` with `Harness::builder().with_step_dt(0.02)`
  and the recording `Fake` controller (`crates/fp-app/tests/it/support`);
  the phone layout is the one `tests/it/music_ui.rs` and
  `tests/it/touch_ui.rs` already build. New test files go into
  `crates/fp-app/tests/it/` with a `mod` line in `tests/it/main.rs`.
- Commit only with `cargo fmt --all --check`, `cargo clippy --workspace
  --all-targets -- -D warnings` and `cargo test --workspace` green.

## Review Focus

- **A category deleted in Settings → Library while its list is open:**
  the tab shows the selector, ticks are dropped, nothing panics (Task 2
  test `a_deleted_open_category_returns_to_the_selector`).
- **Ticks that leave the list** (a search that hides them, a track removed
  from the library): the bar's count and *Add* use only the listed ticked
  tracks, in tick order (Task 3 test
  `ticks_hidden_by_the_search_are_not_added`).
- **🎧 on a missing or unreadable track:** nothing starts, the row says why
  (the same reason as the window), and a preview already running is
  stopped, as §8.4 rules for the window (Task 5 test
  `prelisten_of_a_missing_track_starts_nothing`).
- **Pre-listen on the default output while a player sounds there:** the
  existing *Pre-listen anyway* question appears; *Cancel* starts nothing
  (Task 1 model test `lb47_a_default_output_preview_asks_while_main_sounds_there`
  and Task 5 UI test `prelisten_on_a_sounding_output_asks_first`).
- **Leaving the tab with a preview running** (another tab, or the pick
  mode of PL19b taking the tab): the preview stops (Task 5 test
  `leaving_the_library_tab_stops_the_preview`).

## Rulings made while writing this plan

- **LB46 on Android, not only the phone layout.** Ruling: the fallback is
  the config field `outputs.library.default_output` (serde default
  `cfg!(target_os = "android")`), so an Android tablet with the library
  window gets the same default. — Why: the model and the engine cannot see
  the layout, and a layout switch must not move a running preview; a
  config field is rule 4's place for it. — Cost if wrong: one changed
  default; the desktop is unaffected. The spec's LB46 is amended to say so
  (Task 7).
- **`default_output` has no Settings control.** The Library route
  picker's empty choice reads *Default output* while the field is true
  (Task 4), *None* otherwise; the field itself is config-file only, like
  `shared_buffer_ms`.
- **The open category is the shared `library_view.selection`**
  (`Command::SetLibraryView`, as the window's tree sends). The phone keeps
  only which page is open (selector, list, device) in `ViewState`, so
  "remembered for the session" (§11.1) holds for the page; the selected
  category persists as the window's does. *Whole library* is the same
  `library_view.whole_library` (LB40 says "as in LB26").
- **The phone's search text** is `LibraryUi::search`, shared with the
  window (which the phone layout never shows).
- **The phone's ticks** are a new `PhoneLibraryView::ticked: Vec<TrackId>`
  (tick order), separate from `LibraryUi::selected` and from
  `MusicView::ticked` (§11.1 *Settled details*).
- **Device music stays `music.rs`.** `PhoneScreen::Music` is renamed
  `PhoneScreen::Library`; the Music screen is drawn when the tab's page is
  `LibraryPage::Device`. PL19b's pick mode opens the tab on that page and
  returns where it did before.
- **The menu reuses `menu::row_menu`** with a new
  `MenuScope { Window, Phone }` argument: the phone scope shows *Add to ▸*,
  *Pre-listen* (one track, nothing ticked), *Move to category ▸*,
  *Re-analyse* and *Remove from library*, and hides carts, kind, tags,
  loudness, reset markers and show file location (LB43).
- **The preview strip** reads the existing `view::LibraryCueView`; its
  Play goes through `Command::PlayLibraryCue`, so the model's CA6
  confirmation path and the app's `cue_question` dialog apply unchanged.

---

### Task 1: The library CUE on the default output (model, config, engine)

**Files:**
- Modify: `crates/fp-model/src/config.rs` (`LibraryRoutes`, `OutputsConfig::library_has_cue`, `OutputsConfig::cue_shares_main` doc)
- Modify: `crates/fp-engine/src/engine/library.rs` (`ensure_library`, `move_library_cue`), `crates/fp-engine/src/engine.rs` (`report_library_route`)
- Test: `crates/fp-model/tests/it/` (the config and library CUE tests already there), `crates/fp-engine/tests/it/` (the library CUE engine tests already there)

**Interfaces:**
- Produces: `LibraryRoutes { cue: Option<Route>, default_output: bool }`,
  `#[serde(default = "default_library_default_output")]` on the field,
  `fn default_library_default_output() -> bool { cfg!(target_os = "android") }`,
  also used by `LibraryRoutes::default()` (hand-written `Default`).
- `OutputsConfig::library_has_cue()` is true when the route is usable
  (as today) **or** `cue.is_none() && default_output`.
- Engine: one private helper `fn library_target(&self, route: Option<&Route>) -> Option<(BusKey, u16)>`
  in `engine/library.rs`: `route.and_then(cue_target)`, else
  `Some((self.default_output(), 0))` when `settings.library_routes.default_output`,
  else `None`. Used by `ensure_library`, `move_library_cue` and
  `report_library_route`.

- [ ] **Step 1: Write the failing model tests**
  - `lb46_the_default_output_gives_the_library_a_cue`: a `Config` with
    `outputs.library = LibraryRoutes { cue: None, default_output: true }`
    → `library_has_cue()`; with `default_output: false` → not.
  - `lb46_an_old_config_without_the_field_loads_the_platform_default`:
    JSON `{"cue": null}` deserialises to `default_output ==
    cfg!(target_os = "android")` (false on the test host).
  - `lb46_play_is_available_without_a_route_on_the_default_output`:
    `fp_model::command_available(&state, &Command::PlayLibraryCue)` with a
    playable selected library track, no route, `default_output: true`.
  - `lb47_a_default_output_preview_asks_while_main_sounds_there`: no
    library route, `default_output: true`, placements (as the existing
    cue-on-air tests set them) put `Holder::LibraryCue` and a playing
    player's `PlayerMain` on the same device →
    `cue_on_air::needs_confirmation(&state, &Command::PlayLibraryCue)`.
- [ ] **Step 2: Run them; expect FAIL** (`default_output` does not exist).
  Run: `cargo test -p fp-model --test it lb46_ lb47_`
- [ ] **Step 3: Implement the field and `library_has_cue`.**
- [ ] **Step 4: Write the failing engine test**
  `lb46_the_library_cue_without_a_route_plays_on_the_default_output`
  (Offline backend, `now` driven): settings with no library route and
  `default_output: true`; `StartLibraryCue` → an `EngineEvent::Placed`
  for `Holder::LibraryCue` on the default output's bus, and the source
  attaches there. With `default_output: false` the existing behaviour
  (no bus, the preview ends at once) is kept by the existing test.
- [ ] **Step 5: Implement `library_target` and use it in the three places.**
- [ ] **Step 6: Run** `cargo test -p fp-model --test it lb4` and
  `cargo test -p fp-engine --test it library`; expect PASS. Then the
  Android clippy and `scripts/check-mobile-isolation.sh`.
- [ ] **Step 7: Document** the field in `docs/technical/persistence.md`
  (config reference, next to `outputs.library.cue`).
- [ ] **Step 8: Commit** `feat(model): the library CUE may play on the default output (LB46)`.

### Task 2: The Library tab, its selector and its pages

**Files:**
- Create: `crates/fp-app/src/ui/phone_library.rs`
- Modify: `crates/fp-app/src/ui/touch.rs` (`PhoneScreen::Music` → `PhoneScreen::Library`, tab key `touch-library`, icon `icon::BOOKS`), `crates/fp-app/src/ui/app.rs` (`ViewState::phone_library`), `crates/fp-app/src/ui/music.rs` (pick mode opens the Library tab on the Device page), `crates/fp-app/src/ui/mod.rs`
- Modify: every locale's `main.ftl` (`touch-library`, `phone-library-device-music`, `phone-library-back`; drop `touch-music` if nothing else uses it)
- Test: `crates/fp-app/tests/it/phone_library_ui.rs` (new), and update `tests/it/music_ui.rs` / `touch_ui.rs` for the rename

**Interfaces:**
- Produces in `phone_library.rs`:
  ```rust
  pub enum LibraryPage { #[default] Selector, Tracks, Device }
  pub struct PhoneLibraryView {
      pub page: LibraryPage,
      /// Ticked library tracks, in tick order (LB42).
      pub ticked: Vec<TrackId>,
      /// Device music opened from a list's ＋: the category its files join
      /// (`Some(None)`: uncategorised). `None`: opened from the selector.
      pub adding_to: Option<Option<CategoryId>>,
  }
  impl PhoneLibraryView {
      /// PL2 Back: Device → Tracks (when `adding_to`) or Selector; Tracks →
      /// Selector (ticks cleared); false on the Selector.
      pub fn back(&mut self) -> bool;
      /// Opens `selection`'s list: sends `SetLibraryView` with it, clears ticks.
      pub fn open(&mut self, scene: &Scene<'_>, selection: LibrarySelection);
  }
  pub(crate) fn screen(ui: &mut Ui, scene: &Scene<'_>, view_state: &mut ViewState);
  ```
  `screen` draws the selector, `tracks_page` (Task 3; a stub listing
  nothing until then) or `music::screen` for `Device`.
- The selector rows, in order: *All tracks*, the categories in tree order
  (indented 16 pt per depth, colour swatch, count from the existing
  `CountsCache`/`category_counts`), *Uncategorised*, then *Device music*
  (icon `MUSIC_NOTES`). Rows are `touch::TARGET`-plus high (48 pt, as
  `music.rs`'s `GROUP_ROW`).
- Consumes: `LibraryUi`, `LibrarySelection`, the existing tree order
  helper in `library/menu.rs` (`tree_order`; make it `pub(crate)`).

- [ ] **Step 1: Write the failing UI tests** in `phone_library_ui.rs`:
  - `lb38_the_library_tab_opens_on_the_selector`: phone layout, the
    Library tab → labels *All tracks*, two categories, *Uncategorised*,
    *Device music* in that order.
  - `lb39_a_category_opens_its_list_and_back_returns`: tap a category →
    the Fake records `SetLibraryView` with `Category(id)`, the page is
    `Tracks`; the header's back button → `Selector`; Back again (PL2)
    leaves the tab as Back does from any tab's top.
  - `lb38_device_music_is_todays_music_screen`: tap *Device music* → the
    Folders | Artists | Albums switch is shown; Back → selector.
  - `pl19b_a_pick_opens_device_music_and_returns`: the existing pick test,
    now landing on the Library tab's Device page and returning to the
    screen that asked.
  - `a_deleted_open_category_returns_to_the_selector`: page `Tracks` on a
    category the next snapshot no longer has → `Selector`, ticks empty.
- [ ] **Step 2: Run** `cargo test -p fp-app --test it phone_library_ui`; expect FAIL.
- [ ] **Step 3: Implement** the module, the rename, the strings (all 11
  locales) and the Back wiring (`touch.rs`'s Back handling calls
  `PhoneLibraryView::back` on this tab).
- [ ] **Step 4: Run** the new tests and the whole `music_ui`/`touch_ui`
  files; expect PASS.
- [ ] **Step 5: Commit** `feat(ui): the phone's Library tab and its category selector (LB38, LB39)`.

### Task 3: The track list: search, rows, ticks and the selection bar

**Files:**
- Modify: `crates/fp-app/src/ui/phone_library.rs` (`tracks_page`)
- Modify: `crates/fp-app/src/ui/music.rs` (`selection_bar` becomes shared)
- Modify: locales (`phone-library-search-hint`, `phone-library-whole`, `phone-library-empty`)
- Test: `crates/fp-app/tests/it/phone_library_ui.rs`

**Interfaces:**
- `music::selection_bar` takes what to add instead of `&mut MusicView`:
  ```rust
  pub(crate) enum Picked<'a> { Files(&'a mut Vec<PathBuf>), Tracks(&'a mut Vec<TrackId>) }
  pub(crate) fn selection_bar(ui, scene, picked: Picked<'_>, target: &mut Option<(PlayerId, PlaylistId)>, shown: Option<PlayerId>, wide: bool)
  ```
  `Files` sends `Command::InsertPaths` (as today), `Tracks` sends
  `Command::InsertTracks`; both clear the list. The phone library keeps its
  own `target: Option<(PlayerId, PlaylistId)>` in `PhoneLibraryView`.
- Rows come from `view_state.library.rows(state)` (`LibraryUi::rows`);
  the header has the back button, the selection's name, and a **＋**
  (Task 6 wires it); under it the search field (`LibraryUi::search`) and
  the *Whole library* toggle (sends `SetLibraryView` with
  `whole_library` flipped).
- A row: check box, title (file name when the title is empty), artist,
  duration (`format` helpers as `music.rs`), the status marks the window
  table uses for missing / unreadable / waiting for analysis (reuse
  `track_cells` or `view` helpers), and room on the right for 🎧 (Task 5).
  A tap toggles the tick.

- [ ] **Step 1: Write the failing UI tests**:
  - `lb40_the_search_filters_the_open_category`: category with "Canción
    uno" and "Other"; type `cancion` → only the first row is shown.
  - `lb40_whole_library_searches_everything`: the toggle → the Fake
    records `SetLibraryView` with `whole_library: true`; a track of
    another category matching the search is listed.
  - `lb41_a_row_shows_title_artist_duration_and_status`: a missing track
    shows its title, artist, `3:25` and the missing mark.
  - `lb42_ticks_add_in_tick_order_and_clear`: tick B then A, *Add at the
    end* → `InsertTracks { tracks: [B, A], index: len }`; no row ticked
    after.
  - `lb42_add_after_the_next_uses_the_players_next`: as PL19's existing
    test, with tracks.
  - `ticks_hidden_by_the_search_are_not_added`: tick A and B, search
    hides B → the bar says 1 and adds `[A]`.
  - `lb41_twenty_thousand_tracks_lay_out_only_visible_rows`: a library of
    20 000 tracks; one frame lays out fewer than 100 rows (count rows by
    their accessible label, as `library_table.rs`'s LB28 test does).
- [ ] **Step 2: Run them; expect FAIL.**
- [ ] **Step 3: Implement** `tracks_page` and the shared bar; `music.rs`
  keeps passing `Picked::Files`.
- [ ] **Step 4: Run** `phone_library_ui`, `music_ui`; expect PASS.
- [ ] **Step 5: Commit** `feat(ui): the phone's track list with search, ticks and the selection bar (LB40–LB42)`.

### Task 4: Settings → Outputs → Library on the phone

**Files:**
- Modify: `crates/fp-app/src/ui/settings.rs` (`outputs`: drop the `if !phone` around the Library row; the route picker's empty choice label)
- Modify: locales (`settings-route-default-output` if no existing message fits; reuse the Main pickers' "Default output" message when there is one)
- Test: `crates/fp-app/tests/it/settings.rs` (or the phone settings test file already there)

- [ ] **Step 1: Write the failing tests**:
  - `lb46_the_phone_shows_the_library_outputs_row`: phone layout,
    Settings → Outputs → the *Library* row is present.
  - `lb46_the_empty_library_route_reads_default_output_when_it_falls_back`:
    `default_output: true` → the picker's current label is the default
    output message; `false` → today's empty label.
- [ ] **Step 2: Run; expect FAIL.**
- [ ] **Step 3: Implement.** Update the §8.4 note in
  `crates/fp-app/src/ui/settings.rs` ("desktop only (for now)").
- [ ] **Step 4: Run** the settings tests; expect PASS.
- [ ] **Step 5: Commit** `feat(ui): the library pre-listen route on the phone's Settings (LB46)`.

### Task 5: Pre-listen from the list (🎧 and the strip)

**Files:**
- Modify: `crates/fp-app/src/ui/phone_library.rs` (🎧 per row, `preview_strip`), `crates/fp-app/src/ui/touch.rs` or `app.rs` (stop on leaving the tab)
- Modify: locales (`phone-library-prelisten`, reuse `library-preview-*` for the strip's buttons)
- Test: `crates/fp-app/tests/it/phone_library_ui.rs`

**Interfaces:**
- 🎧 on a row sends `Command::SelectLibraryCue(Some(id))` and, when the
  library CUE is stopped, `Command::PlayLibraryCue` (LB45; while it plays
  or is paused the select alone switches it, LB34). Disabled with the
  same tooltip as the window's Play when `command_available` says no
  (no route, not playable).
- `fn prelisten(scene: &Scene<'_>, track: TrackId)` is `pub(crate)` so the
  menu (Task 6) calls it.
- The strip, shown at the top of the list while the library CUE has a
  track and is not stopped: title, remaining time (as the window's
  countdown), Pause/Play (`SetLibraryCuePaused`), Stop (`StopLibraryCue`).
  No waveform.
- Leaving the Library tab (any `PhoneScreen` change away from it,
  including PL19b's pick taking the tab to Device and back is *not*
  leaving it) sends `StopLibraryCue` when the CUE is not stopped.

- [ ] **Step 1: Write the failing tests**:
  - `lb45_the_headphones_start_the_track_from_its_cue_in`: tap 🎧 → the
    Fake records `SelectLibraryCue(Some(id))` then `PlayLibraryCue`.
  - `lb45_another_rows_headphones_switch_a_running_preview`: CUE playing
    track A; 🎧 on B → only `SelectLibraryCue(Some(B))`.
  - `lb45_the_strip_pauses_and_stops`: CUE playing → the strip shows the
    title; Pause → `SetLibraryCuePaused(true)`; Stop → `StopLibraryCue`.
  - `leaving_the_library_tab_stops_the_preview`: CUE playing; tap the
    Live tab → `StopLibraryCue`.
  - `prelisten_of_a_missing_track_starts_nothing`: 🎧 of a missing track
    is disabled (no `PlayLibraryCue` recorded).
  - `prelisten_on_a_sounding_output_asks_first`: a state whose
    `cue_on_air.request` is set after `PlayLibraryCue` (the model's CA6
    path) → the *Pre-listen anyway* dialog is shown on the phone layout.
- [ ] **Step 2: Run; expect FAIL.**
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run** `phone_library_ui`; expect PASS.
- [ ] **Step 5: Commit** `feat(ui): pre-listen from the phone's library (LB45, LB47)`.

### Task 6: The touch-and-hold menu, and Device music from ＋

**Files:**
- Modify: `crates/fp-app/src/ui/library/menu.rs` (`row_menu(ui, scene, tracks, scope: MenuScope)`; the window passes `MenuScope::Window`)
- Modify: `crates/fp-app/src/ui/phone_library.rs` (long press → menu; the header's ＋), `crates/fp-app/src/ui/music.rs` (the bar's *Add to <category>* when `adding_to` is set)
- Modify: locales (`phone-library-add-to-category` with a `$category` argument, `phone-library-add-to-uncategorised`, `library-menu-prelisten`)
- Test: `crates/fp-app/tests/it/phone_library_ui.rs`, `crates/fp-app/tests/it/library_menu.rs` (the window keeps every item)

**Interfaces:**
- `pub(crate) enum MenuScope { Window, Phone }`; Phone shows *Add to ▸*,
  *Pre-listen* (only when `tracks.len() == 1` and nothing is ticked;
  calls `phone_library::prelisten`), *Move to category ▸*, *Re-analyse*,
  *Remove from library*; hides the rest. `MenuOutcome::Remove` is
  answered with `confirm::ask_removal` as the window does.
- Long press on a row opens `Response::context_menu`, as the tree's
  touch menu does (`library/tree.rs`; egui opens it on a long touch,
  tests use a secondary click): when nothing is ticked, the menu acts on that row;
  when rows are ticked and the row is not, it is ticked first; the menu
  acts on every listed ticked row, in tick order.
- ＋ sets `adding_to = Some(added_category(&state.library_view))`
  (`library::added_category`) and the page to `Device`. On Device with
  `adding_to`, the bar shows only **Add to <category>** (or *Add to
  Uncategorised*) and sends `Command::AddToLibrary { paths, category }`,
  clears the device ticks, and returns to the list. No pick mode, no
  player target.

- [ ] **Step 1: Write the failing tests**:
  - `lb43_the_phone_menu_has_only_its_items`: long press → *Add to*,
    *Pre-listen*, *Move to category*, *Re-analyse*, *Remove from library*
    present; *Add to cart*, *Kind*, *Edit tags…*, *Loudness…*, *Reset
    markers*, *Show file location* absent.
  - `lb43_a_long_press_on_an_unticked_row_joins_the_ticks`: A ticked;
    long press B → menu heading "2 tracks"; *Move to category ▸ Gold* →
    `MoveTracksToCategory { tracks: [A, B], .. }`.
  - `lb43_pre_listen_is_not_offered_while_rows_are_ticked`.
  - `lb43_remove_asks_first`: *Remove from library* → the LB17 question
    is shown; nothing sent until confirmed.
  - `lb44_plus_opens_device_music_and_adds_to_the_category`: ＋ on
    category Gold → Device page, its bar reads *Add to Gold*; tick two
    files, tap it → `AddToLibrary { paths: [..], category: Some(gold) }`;
    page back to `Tracks`.
  - `lb44_device_music_from_the_selector_has_no_add_to_category`.
  - `the_window_menu_keeps_every_item` in `library_menu.rs` (cart, kind,
    tags, loudness, reset markers, show file location still there).
- [ ] **Step 2: Run; expect FAIL.**
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run** `phone_library_ui`, `library_menu`, `music_ui`; expect PASS.
- [ ] **Step 5: Commit** `feat(ui): the phone library's menu and adding device music to a category (LB43, LB44)`.

### Task 7: Documentation, spec notes and the Android check

**Files:**
- Modify: `docs/user/android.md` (the Library tab replaces the Music
  tab: selector, list, ticks, menu, 🎧 and the default output, ＋ and
  Device music, categories in Settings → Library), `docs/user/library.md`
  (a short "On a phone" section pointing to android.md)
- Modify: `docs/technical/mobile.md` (drop "pre-listen of PL18 is not
  built" for library tracks: a loose device file still has none),
  `docs/technical/ui.md` if it lists the phone tabs
- Modify: the spec §11.1 LB46 ("On Android" and the config field; record
  this plan's rulings as *Implementation notes (plan 6)*), §8.4's
  "hidden on the phone layout until plan 6" line; the phone layout spec
  PL19b (the Music screen is now the Library tab's Device music page)
- Modify: `README.md` if its Android feature list names the Music tab
- Modify: `scripts/package/play-screenshots.sh` only if it taps the
  Music tab by name (check; adjust the tap, do not retake images)

- [ ] **Step 1: Write the docs and spec notes.**
- [ ] **Step 2: Run** `scripts/site/build.sh` and
  `scripts/site/check-links.sh target/site`; expect no dead links.
- [ ] **Step 3: Run** the Android clippy and
  `scripts/check-mobile-isolation.sh`; expect PASS.
- [ ] **Step 4: Commit** `docs: the phone's library screen`.
- [ ] **Step 5 (controller): emulator check** with the release APK
  (`scripts/package/android.sh release`, AVD `fauste-test`): the Library
  tab opens on the selector, a category's list, ticks and *Add at the
  end*, 🎧 on the default output, the menu, ＋ → Device music → *Add to
  <category>*. Record the result in the ledger.
