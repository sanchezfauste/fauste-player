# Operator Feedback 2 — Design Spec

- **Date:** 2026-10-01
- **Status:** Approved. Plans 1 to 9, 12 and 13 are built; each plan's section ends with its "As built" notes.
- **Extends:** [the main design spec](2026-09-25-fauste-player-design.md) (§2 threads,
  §3 rules, §6 analysis, §8 UI), the [meters spec](2026-09-27-meters-design.md)
  (M4 display), the [cartwall and settings spec](2026-09-26-phase2-cartwall-settings-design.md)
  and the [first operator feedback spec](2026-09-30-operator-feedback-design.md)
  (F2 is replaced by O10 here).
- **Scope:** 38 items of operator feedback (O1–O38), grouped into thirteen
  plans. O25–O36 were added on 2026-10-02 and O37–O38 on 2026-10-03.
  Plan 12 runs after plan 9, then plan 13, then plan 10.
  Each plan is written in full just before it runs, against the code the
  previous plan left. Each reaches `master` through its own pull request and
  updates the main spec, the user guide and the technical docs for what it
  changes.

---

## 1. Items and plans

| Item | Summary | Plan |
|---|---|---|
| O1 | Project website with downloads and the user guide | 11 |
| O2 | Restore defaults in Settings sections | 2 |
| O3 | Settings window layout and a fixed size | 2 |
| O4 | Restart the app to apply pending settings | 2 |
| O5 | Hide the Null backend from the audio system list | 2 |
| O6 | Confirm before closing while audio is on air | 1 |
| O7 | Tables open scrolled to the next entry | 8 |
| O8 | Banner for per-entry repeat and stop on the current entry | 6 |
| O9 | Per-entry icons before the title | 8 |
| O10 | Waveform: click seeks, drag pans | 6 |
| O11 | Meter scale: missing or misplaced end marks | 3 |
| O12 | CUE window with position, seek, pause and "load as next" | 6 |
| O13 | More visible meter reference lines | 3 |
| O14 | Standard window controls | 1 |
| O15 | Option to ignore cue-in and cue-out | 5 |
| O16 | Live column resizing | 8 |
| O17 | CUE follows the selected or next entry | 6 |
| O18 | One stop icon everywhere | 4 |
| O19 | Stop all carts | 4 |
| O20 | A visible way to open About | 1 |
| O21 | Unfinished parts of earlier plans | 9 |
| O22 | Reset the played marks of a playlist | 8 |
| O23 | Tag tooltip and tag editor | 7 |
| O24 | Choose and order the table columns | 8 |
| O25 | DSD output: native, DoP or conversion to PCM | 10 |
| O26 | Audit of the audio path (no dropouts, no clicks) | 10 |
| O27 | Show "No output (silent)" in the audio system list again | 10 |
| O28 | Screenshots in the user guide | 11 |
| O29 | A truer README screenshot | 11 |
| O30 | Repository description and website link | 11 |
| O31 | The cartwall fits its rows without scrolling | 12 |
| O32 | One title bar: icon, name and version in the native one | 12 |
| O33 | Countdowns and live times keep a fixed width | 12 |
| O34 | Digital peak meter readings above 0 dBFS | 10 |
| O35 | Playlist tabs shrink and scroll instead of overflowing | 12 |
| O36 | No console window on Windows | 12 |
| O37 | The playing entry can be set as next, to play once more | 13 |
| O38 | Stop after current in Single mode for a repeating entry | 13 |

| Plan | Title | Items | Depends on |
|---|---|---|---|
| 1 | Window and lifecycle | O6, O14, O20 | — |
| 2 | Settings window | O2, O3, O4, O5 | 1 (the on-air confirmation) |
| 3 | Meter scale | O11, O13 | — |
| 4 | Cartwall stop | O18, O19 | — |
| 5 | Cue markers off | O15 | — |
| 6 | Player and CUE | O8, O10, O12, O17 | 4 (the shared icons) |
| 7 | Track tags | O23 | — |
| 8 | Track table | O7, O9, O16, O22, O24 | 7 (the tag fields) |
| 9 | Audit follow-ups | O21 | — |
| 10 | Audio path | O25, O26, O27, O34 | 2 (O27 reverses part of O5) |
| 11 | Website and guide | O1, O28, O29, O30 | all (it publishes the final docs) |
| 12 | Window and layout | O31, O32, O33, O35, O36 | — (runs after 9, before 10) |
| 13 | Player rules | O37, O38 | — (runs after 12, before 10) |

---

## 2. Plan 1 — Window and lifecycle

- **O6 Close confirmation.**
  - A pure function `fp_model::on_air(&AppState) -> Vec<OnAirItem>` lists what
    is sounding: every player that is Playing or Paused (with its current
    entry) and every cart that is playing. CUE is not on air.
  - The UI intercepts the window's close request. When the list is empty, the
    app closes as today. Otherwise a modal names each item ("P1 — Title",
    "Cart 3 — Title") and offers **Cancel** (default, also Esc and closing the
    modal) and **Stop and close**.
  - **Stop and close** stops everything, saves the session and closes.
  - The same modal guards the restart of O4 (plan 2), with the words "restart"
    instead of "close".
- **O14 Window controls.** The window keeps native decorations. On GNOME
  Wayland the compositor offers no server-side decorations, so winit draws its
  own, with non-standard buttons.
  - Task 1 is a time-boxed investigation of: the winit/sctk theme options that
    give the standard minimise (_), maximise (two windows / square) and close
    (✕) glyphs; and running through XWayland on GNOME so that the compositor
    draws native decorations.
  - The choice is recorded as a `Ruling` and in `docs/technical/ui.md`. Windows
    and macOS are unchanged.
- **O20 About.** An info button (Phosphor `INFO`) in the top bar, next to the
  Settings button, opens About. Clicking the name and version still does.

## 3. Plan 2 — Settings window

- **O3 Layout.**
  - The Settings window has one fixed size, whatever the section: a constant
    (about 900 × 640 logical px), clamped to the viewport. The section body
    scrolls inside it.
  - Every section lays out its rows with one shared grid helper: a fixed-width
    label column and controls that fill the rest.
  - The footer bar spans the full window width.
  - In Outputs, the rows have equal width and the Test Main and Test Cue
    buttons line up in columns.
  - Every section is reviewed. Kittest geometry tests check that the footer
    spans the window, that Outputs' test buttons share their x positions, and
    that switching sections never changes the window size.
- **O2 Restore defaults.**
  - Sections with a button: Players, Meters, Analysis and Shortcuts. Players
    keeps the player count and the interface language. Sections without one:
    Outputs and MIDI (they depend on the hardware), Remote (it holds security
    settings), Playlists, and Cartwall (the maintainer's decision: its pages
    are the operator's own content).
  - The button sits in the section header and asks for confirmation
    ("Restore the default values of this section?").
  - The rule is pure: `fp_model::restore_defaults(&mut Config, SettingsSection)`
    resets that section's whole configuration group to `Config::default()`,
    including fields the section does not show. One test per
    section checks that the other sections are untouched.
- **O4 Restart.**
  - At start-up the app keeps the configuration the engine was built with.
  - A pure function `fp_model::restart_pending(started: &Config, current:
    &Config) -> Vec<RestartReason>` lists the changed fields that apply only on
    restart. The final set: audio system, sample rate, buffer size, routes
    (players and cartwall), bit-perfect devices, limits and tuning. The
    player count is not in it: the player count applies live. Nothing
    changes limits or tuning while the app runs (they are edited in the
    configuration file with the app closed), so in practice they apply at
    the next start without raising the notice.
  - While the list is not empty:
    - the Settings footer shows "Some changes take effect after a restart."
      with a **Restart now** button;
    - the top bar shows a small "Restart pending" pill that runs **Restart
      now**: at once when nothing is on air, through the on-air guard
      otherwise.
  - **Restart now** asks the plan 1 on-air confirmation when something is on
    air. It then stops everything, saves the session, starts the same
    executable with no arguments (the playlists given at the first start were
    imported and saved already, and passing them again would import them
    twice), with the same environment, and exits. In an AppImage it starts
    `$APPIMAGE`; in a Flatpak it uses `flatpak-spawn` and waits for the
    hand-off (`tuning.restart_handoff_ms`); a hand-off that times out is
    reported as a failed restart. After the
    restart nothing is on air (rule 10).
  - The existing note "Changes to audio outputs take effect the next time…" is
    replaced by this notice.
- **O5 Null backend.** The `null` backend is left out of the audio system
  list, unless it is the one configured. It is then shown as "No output
  (silent)". It stays available in the configuration file for tests and
  headless use. (O27 reverses the hiding: it is always listed, last.)

## 4. Plan 3 — Meter scale

- **O11 End marks.**
  - For every meter type (digital peak, EBU PPM, DIN PPM, VU, K-20/14/12,
    custom), the top of the scale (0 dB or the type's maximum) and the bottom
    (the floor, such as −60) are always marked and labelled.
  - End labels are clamped inside the meter's rect (aligned to the edge, not
    centred over it), so they are never cut off or shifted away from their
    line.
  - Every label has its reference line.
  - Tests in `tests/meter_view.rs` run every meter type at 64, 136 and 300 px.
    A screenshot in Xvfb confirms the result.
- **O13 Reference lines.**
  - Over the unlit part of the bars, lines are drawn in `NEUTRAL_400` at about
    60 % opacity (was 35 %).
  - Over the lit part, they are drawn as a dark cut (`NEUTRAL_900`, about
    50 %), so they read as segment gaps.
  - Both values are constants in `ui/theme.rs`. They are tuned with a
    screenshot and pinned by `tests/theme.rs`.
- **As built.**
  - The digital scale's floor is a mark of its own (`scale_marks`), so a −55
    floor is labelled `-55`.
  - A label is centred on its line; only where a centred label would cross
    the meter's rect does it rest on the line (bottom) or hang from it (top).
  - The alignment label gives way to an end label; its heavier line and
    notches stay. An end that is itself the alignment level keeps that
    label, and the other end yields if they crowd.
  - A reference-line piece is lit when the bar's level (the peak on K-System
    meters) is at or above the line; the gap between the bars is never lit.
  - Constants: `METER_LINE_UNLIT` / `_ALPHA` (`NEUTRAL_400`, 0.60) and
    `METER_LINE_LIT` / `_ALPHA` (`NEUTRAL_900`, 0.50), pinned by
    `tests/theme.rs`.

## 5. Plan 4 — Cartwall stop

- **O18 Shared icons.**
  - One module, `ui/glyphs.rs`, maps each transport action (play, pause, stop,
    fade stop, stop after, restart, previous, cue) to its icon.
  - The player, the cartwall and the CUE window (plan 6) draw icons only
    through it.
  - The cart stop becomes the player's stop icon.
- **O19 Stop all carts.**
  - A button at the right end of the cartwall bar, after the page and collapse
    controls.
  - Its label is "Stop all", followed by the number of playing carts in
    parentheses ("Stop all (2)"). With no cart playing it is disabled and has
    no count.
  - `Command::StopAllCarts` stops every playing cart the same way the
    individual stop does, with one rule test.
  - `ShortcutAction::StopAllCarts` is added with no default key.
- **As built.**
  - `ui/glyphs.rs` has `TransportAction` (the spec's list plus `Next`, the play
    button while a player is on air). The player, the cartwall menu and the
    playlist menu draw only through it; `tests/glyphs.rs` guards that.
  - The "Stop all (n)" button is the first item of a right-to-left row, so
    it is always whole at the right end; the title, tabs and hint take the
    rest. The count is `cartwall.playing.len()`, carts on every page. The
    messages are `cartwall-stop-all` and `cartwall-stop-all-count`.
  - `ShortcutAction::StopAllCarts` already existed, bound to `Ctrl+Space`
    by default. The default stays: removing it would change a documented
    and tested behaviour. It can be rebound or unbound. The shortcut is not
    dimmed with the button: it also stops the cart CUE, so it works with no
    cart playing.

## 6. Plan 5 — Cue markers off

- **O15.** A new field, `players.use_cue_markers: bool`, defaults to `true`
  and loads leniently. It is independent of `players.auto_segue`, which already
  turns the MIX off.
  - When it is `false`, every player entry plays from 0 to the end of the file:
    automatic and manual cue-in and cue-out are ignored, but kept.
  - The rule lives in the model's effective-range function, which the engine
    requests, the countdowns and the MIX validation already share. One test per
    consumer.
  - It applies to players only; carts are unchanged.
  - The waveform draws the cue-in and cue-out marks dimmed while they are
    ignored.
  - Settings > Players gets a toggle "Use cue-in and cue-out", next to
    "Automatic mix at the MIX point".
- **As built.**
  - `Track::play_range(use_markers) -> PlayRange` is the effective-range
    function. There was none before: each consumer read the cue methods
    directly. Now `request_from_cue_in`, `request_at`, `plan_for`, the session
    restore, `set_marker` (MIX, intro, outro), `player_view`,
    `playlist_times`, the remote player DTO and the remote seek bounds use it.
    Carts keep the marker-based methods.
  - Cue-in and cue-out edits are validated against the kept markers even
    while they are ignored; intro, outro and MIX against the effective range.
  - The player's CUE follows the range. Toggling the setting re-preloads and
    re-plans; it never restarts a track on air.
  - The waveform does not shade the head and tail while the marks are
    ignored; the lines are drawn at `CUE_EDGE_IGNORED_ALPHA` (0.35).
  - The messages are `settings-use-cue-markers` and
    `settings-hint-use-cue-markers`.

## 7. Plan 6 — Player and CUE

- **O8 Per-entry banner.** When the current entry has per-entry repeat (F4) or
  stop after this entry (F5), the player header shows a notice in the style of
  stop after current: "This track will repeat" or "Stops after this track".
  Stop after current, when also set, takes precedence.
- **O10 Click seeks, drag pans.**
  - A press and release that stays within the drag threshold seeks there.
  - Dragging pans the zoomed view and never seeks. Without zoom, a drag does
    nothing.
  - The F2 seek preview on drag is removed. Alt-drag marker editing is
    unchanged.
- **O12 CUE window.**
  - When a player starts a CUE, a floating, non-modal window opens for it.
    Several CUE windows stack.
  - It shows: the title and artist; the waveform with the CUE position, where a
    click seeks the CUE; the elapsed and remaining time; and the buttons
    **Pause/Resume**, **Stop** and **Load as next**.
  - **Load as next** makes the cued entry the player's explicit next and keeps
    the CUE running.
  - Closing the window stops the CUE.
  - Model:
    - `CueState` gains `paused: bool`;
    - new commands `SeekCue(PlayerId, f64)`, `SetCuePaused(PlayerId, bool)`
      and `CueToNext(PlayerId)`;
    - each command has rule tests.
  - Engine: the CUE source supports seek and pause, with Offline-backend tests.
    The CUE position reaches the UI through the snapshot.
- **O17 CUE follows.** While a player's CUE is running:
  - setting that player's next (double-click) moves the CUE to the new next,
    from its cue-in. This is a model rule on `SetNext`.
  - selecting a row with a single click in that player's table moves the CUE
    to that entry. Selection is UI state, so the UI sends `CueEntry`.
- **As built.**
  - O8: `fp_model::entry_notice(state, player) -> Option<EntryNotice>` (`Repeats`, `StopsAfter`) decides; the header draws an outlined amber badge (`badge-entry-repeat`, `badge-entry-stop`) whose tooltip and accessible name are the spec's sentences. None while stopped, during a fade stop, or while stop after current is set; the entry's stop-after wins over its repeat; an unreadable file does not repeat.
  - O10: `widgets::waveform` returns `WaveOutput { response, seek, pan_dx }`. Only `Response::clicked()` seeks. A primary drag that does not start with Alt or under the Full view button pans a zoomed view and does nothing otherwise. The hover time stays; the drag preview and Esc-cancel are gone.
  - O12: `CueState { entry, paused }`; `Command::{SeekCue, SetCuePaused, CueToNext}`; `EngineAction::{SeekCue, SetCuePaused}`. A CUE plays whole files, so the window's waveform and times cover the whole file. A moved CUE starts unpaused. `CueToNext` uses `set_next` (it becomes explicit; the current entry is refused). The window is `ui/cue_window.rs`, one per running CUE, with its own close button that stops the CUE. The remote API and MIDI do not expose pause or seek of the CUE.
  - O17: `SetNext` calls `follow_cue`: the CUE moves to a playable new next from its cue-in unless it is already there. The table sends `CueEntry` on a primary click through `view::cue_follow_target` (not for the cued row, a missing or unreadable file, or a player without a CUE).

## 8. Plan 7 — Track tags

- **Model.**
  - `Track` gains `date`, `genre`, `album_artist`, `composer` and `comment`,
    read by `fp-analysis` with lofty. The `date` is the recording date, kept
    as the standard ISO 8601 text (`YYYY`, `YYYY-MM` or `YYYY-MM-DD`, with an
    optional time; ID3v2.4 `TDRC`, Vorbis `DATE`, MP4 `©day`, APE `Year`).
    It is never reduced to a year, so a full date in a file survives an edit
    of another field.
  - Loading is lenient. Tracks from earlier versions show the new fields empty
    until their tags are read again. That is a tag-only pass, not a full
    re-analysis.
- **Tooltip.** Hovering a table row, after the usual tooltip delay, shows the
  title, artist, album, date (as stored), genre, duration, format (codec,
  sample rate, bit depth) and path. A missing field is left out.
- **Editor.**
  - The row context menu gains "Edit tags…". It opens a modal for one track.
    The modal reads the file's tags on a helper thread when it opens and shows
    "Reading tags…" until they arrive. The library keeps only the summary
    fields above; the full set is read from the file each time.
  - **Fields.** The editor covers the fields that common players and tag
    editors show, not every key a format can hold. Each field uses its
    format's own standard mapping (ID3v2 frames, Vorbis comments, MP4 atoms,
    APE items, RIFF INFO) through lofty's `ItemKey`; nothing is renamed or
    invented.
    - Always shown, in this order: Title, Artist, Album, Album artist, Date,
      Track number (number and total), Disc number (number and total), Genre,
      Composer, Comment.
    - Shown when the file has them, and offered by an **Add field** menu
      otherwise: Subtitle, Grouping, BPM, Initial key, Mood, ISRC, Publisher,
      Catalog number, Copyright, Original artist, Original album, Original
      release date, Lyricist, Conductor, Remixer, Arranger, Performer,
      Language, Encoded by, Lyrics, Sort title, Sort artist, Sort album,
      Sort album artist, Sort composer, Artist website.
    - **Add field** lists only the fields the file's tag format can store. A
      field the format cannot store is never shown as editable.
    - Clearing a field removes it from the file. An added field left empty is
      not written.
    - A field that holds several values (for example two artists) shows one
      value per line, and Save writes one value per line through the format's
      own multi-value mechanism.
    - Date and Original release date are ISO 8601 (`YYYY`, `YYYY-MM` or
      `YYYY-MM-DD`, optional time). Track and disc number and total, and BPM,
      are whole numbers. An invalid value blocks **Save** and its field is
      marked.
    - Everything else in the file (other standard keys, custom keys such as
      ID3v2 `TXXX` or private Vorbis keys, pictures other than the front
      cover, binary frames) is not shown and is kept as it is: values,
      pictures and frames the editor cannot map are not changed. The modal
      says how many such tags are kept.
    - Two caveats, stated here so they are not a surprise:
      - the format re-encodes the items it maps when the file is saved, so
        the bytes of a kept item can differ (for example the text encoding or
        the order of frames) while its value does not;
      - the count of kept tags covers what the tag library can enumerate. When
        the format also holds frames it cannot count, the modal says "and
        more".
  - **Cover.**
    - The modal shows the front cover as a thumbnail. If the file has no front
      cover it shows the first picture, the rule the library's cover
      thumbnail follows. The image is decoded on a helper thread under
      `limits.max_cover_bytes` and `limits.max_cover_pixels`, with the same
      safe decoding as the library's thumbnails.
    - **Change…** opens a file dialog, on a helper thread, for a JPEG or PNG
      image of at most `limits.max_cover_bytes` that decodes. If it does not,
      the editor says why and nothing changes. **Remove** clears the front
      cover.
    - The change is staged in the draft and written by **Save**, through the
      same safe write as the fields. **Cancel** discards it. The new cover is
      written as a picture of type front cover with its MIME type.
    - Other pictures (back cover, artist and so on) are never touched. A
      picture that is only shown because the file has no front cover is kept
      as it is: **Remove** is off for it, and **Change…** adds a front cover.
    - A format that cannot store pictures (RIFF INFO, AIFF text, ID3v1) shows
      the cover area disabled, with a note.
    - After a successful save, the cover the application shows elsewhere (the
      player's cover and the remote API's cover of a track the player holds)
      shows the new cover, or none after a removal.
  - **Save** writes the tags and the cover into the file on a helper thread,
    never on the UI thread:
    1. copy the file to a temporary file in the same folder;
    2. write the tags to the copy;
    3. fsync it;
    4. rename it over the original.
  - On success, the library takes the new tags; markers and analysis are kept.
  - On any error, the original file is untouched and the notice area reports
    it.
  - The menu item is disabled, with the reason as a tooltip, when:
    - the track is current or cued in any player;
    - it is on a playing cart;
    - its format has no writable tags in lofty;
    - the file is missing.
- **As built.**
  - Model: `TrackTags` is the summary the library keeps (read, tooltip, `ApplyTags`); `Track` gains `date: Option<String>` (ISO 8601 text, validated by `parse_tag_date`), `genre`, `album_artist`, `composer`, `comment` and `tags_read`; `fp_model::tag_edit_block(state, track, format_writable)` gives the reason an edit is refused (`FileUnavailable`, `UnsupportedFormat`, `TagsNotRead`, `OnAir`, `Cued`, `OnCart`) and judges the file, not the entry. `limits.max_tag_chars` (2000, 64..=100000) cuts tag text and `limits.max_tag_values` (32, 1..=1000) the values of a field.
  - Tag-only pass: every `ApplyAnalysis` clears `tags_read`; `Services::tag_pass` sends analysed, readable tracks with unread tags to the `fp-tags` worker; no analysis version bump.
  - Tooltip: `view::track_tooltip` (title, artist, album, date, genre, duration, format, path); the codec is the upper-cased extension.
  - Editor: `TagSheet` and `TagField` (36 fields, 10 always shown) with the pure rules `invalid_fields`, `changed_fields`, `unstored_fields`; `fp_analysis::tags::{read_tag_sheet, write_tag_sheet}` read and write them through lofty's `ItemKey`, the write using the same synced-copy helper as `write_tags` and touching only the fields that changed. A multi-value field is one item per value; a number and its total are two items (ID3v2 merges them into `TRCK`/`TPOS`); a total needs a number. Items the sheet does not own, including URL frames and comments with a description, are kept (the shared writer re-adds ID3v2 locators as text because lofty 0.25.4 drops them on save). `other_kept` counts the other items and pictures; frames lofty keeps without mapping them are not counted, and the modal says so. After a save the library takes the re-read summary; on failure the modal stays open and the notice area gives the reason.
  - Cut fields: a field whose text exceeded `limits.max_tag_chars` or that had more than `limits.max_tag_values` values when read is marked (`TagSheet::is_cut`), shown read-only with "Too long to edit here; kept as it is in the file", and never written back: `fp_model::field_problem` reports `FieldProblem::TooLongToEdit` for a change to it, `invalid_fields` includes it, and `write_tag_sheet` refuses the change.
  - Cover: `CoverArt` in the sheet is the front cover, or the first picture when there is none (`metadata::display_picture`, shared with the library's thumbnail); `cover_changed`, `cover_blocked` and `cover_unstored` are pure. **Change…** takes a JPEG or PNG within `limits.max_cover_bytes` and `limits.max_cover_pixels` through a helper-thread dialog and the tag worker (`load_cover_file`); **Remove** clears the front cover and is off when the file has none (a non-front picture shown is display-only); both are staged in the draft and written by Save through `write_tag_sheet`, which replaces or removes every front-cover picture and never touches other pictures, and refuses the change for a format with no pictures (RIFF INFO, AIFF text, ID3v1). An existing cover that cannot be shown (an undecodable image, or GIF, BMP or WebP) is announced with "This cover cannot be shown; it is kept as it is"; Change… and Remove still work on it. After a save the player's cover follows the file (`MediaCache::set_cover`); a track with no cache entry reads the new cover from its next analysis, because the analysis cache is keyed on the file's size and modification time.
  - Modal: while the editor is open no keyboard shortcut acts and files dropped on the window are discarded. Each editor session has an id, returned with an image choice and its decoded cover, so a stale choice from a closed editor never reaches a later one.
  - Two lofty 0.25.4 defects are worked around and tested: URL frames are dropped on save unless re-added as text, and `Tag::take_filter` reorders the items it leaves.

## 9. Plan 8 — Track table

- **O7 Scroll to next.** When the app opens, each player's table scrolls so
  that its next entry is visible and centred. This happens once, at start-up.
- **O9 Icon position.** The per-entry repeat and stop icons are drawn before
  the title, not after it.
- **O16 Live resize.** While a column edge is dragged, the other columns are
  recomputed every frame. The widths are stored on release, as today. Task 1
  checks whether `egui_extras::TableBuilder` allows this or whether the table
  draws its own resize handles.
- **O22 Reset played.**
  - A button next to "Add tracks", with a confirmation ("Clear the played mark
    of every track in this playlist?").
  - `Command::ResetPlayed(PlaylistId)` clears the played mark of every entry
    except the current one. A non-explicit next is recomputed by the existing
    rules.
- **O24 Columns.**
  - A global, ordered list `ui.table_columns: Vec<TableColumn>`, loaded
    leniently: unknown columns are dropped, and missing required columns are
    added back.
  - Required columns: Title and Duration. Optional columns: `#`, Artist,
    Album, Date, Genre, Intro, File name.
  - All columns, required or not, can be reordered:
    - by dragging a header in the table;
    - in Settings > Playlists, with a checkbox list and up and down buttons.
  - The table header's context menu also shows and hides the optional columns.
  - Column widths become fractions keyed by column. Widths stored in the old
    `[f32; 4]` form are converted when they map cleanly; otherwise the default
    layout is used.

- **As built.**
  - O16: `egui_extras` 0.36's `TableBuilder` cannot recompute the other columns during a drag (it changes only the dragged column, a frame late, and keeps resizable widths), so the table draws its own resize handles. Every column is `Column::exact(px)` from `table_layout::column_px`; `live_widths` follows the raw pointer on every frame with the pure `resize_px`; one `SetColumnWidths` is sent on release, only if something moved. A double click on an edge no longer auto-sizes.
  - O7: `view::start_scroll_target` (the player's next, when it is in the shown playlist) and `player::scroll_to_next_once` give `FollowScroll { align: Center, animated: false }` once per player, on the first frame its column is drawn. `TableBuilder::min_scrolled_height(0.0)` stops the footer from covering the last rows in a short window.
  - O9: the Title cell draws the repeat icon, the stop-after icon and then the title; the outdated-analysis flag stays at the right.
  - O22: `Command::ResetPlayed(PlaylistId)`; `fp_model::{resettable_entries, can_reset_played}`. The current entry of any player keeps its marks; other playlists, flags, histories and explicit nexts are untouched; the derived next is recomputed with `refresh_next`. The footer button asks first (`reset_played::show`) and is dimmed when nothing can be cleared.
  - O24: `fp_model::TableColumn` (`Number`, `Title`, `Artist`, `Album`, `Date`, `Genre`, `Duration`, `Intro`, `FileName`; file names are snake case), `ui.table_columns` with `normalize_columns` (first duplicate wins; a missing Title goes first, after `#` when that leads; a missing Duration goes last) in `Config::validate`; `with_column_shown`, `move_column`, `move_column_before` and `column_rows` are the pure edit rules. `ColumnWidths.fractions` is `Option<BTreeMap<TableColumn, f32>>`; the old four-number array converts to `#`, Title, Artist and Duration. `Track::intro_secs` feeds Intro. The list is edited in Settings → Playlists (`settings/columns.rs`), in the header's menu and by dragging a header (`DragColumn`); `Scene::set_table_columns` sends one `UpdateConfig`.
  - Not exposed through the remote API, MIDI or the session file beyond the widths.

## 10. Plan 9 — Audit follow-ups

The audit of every earlier plan found these items still open.

- **Remote plan 4 minors.**
  - M1: a port typed into a drag value in Settings > Remote is kept when
    another section opens.
  - M4: carts with no analysed format are analysed even when "Later" is chosen.
  - M5: `GET /tracks/{id}/peaks` works for outdated tracks.
  - M6: `outdated_tracks` is recomputed when the library changes, not every
    frame.
  - M7: rename `ALIGNMENT_LINE_WIDTH`, and remove the 5.5 s sleep in the
    `bind_log` test.
  - M2 (Matroska Opus pre-skip, ±24 samples) stays a recorded ruling.
- **Missing-file plan minor M2.** The missing-file reason tooltip also shows on
  the title cell.
- **Docs.** `docs/user/players.md` says that only digital peak, K-System and
  custom meters show a peak hold.
- **Thread priority** (main spec §2.2):
  - decoder threads run above normal priority (not real time);
  - the analysis pool runs at low priority;
  - this uses the `thread-priority` crate, after `cargo deny check`;
  - when the system refuses the change, the app logs it once and goes on;
  - `docs/technical/threading-and-realtime.md` is aligned.
- **Long-file check.**
  1. In Xvfb, play a four-hour file in the app.
  2. Seek to 3:59:00 through the remote API.
  3. Record the result in the ledger.
- **As built.**
  - M1: `number()` in `ui/settings/remote.rs` no longer calls `update_while_editing(false)`, so the typed value reaches `RemoteState::numbers` and `remote::flush` applies it when another section opens (clamped to the range, as Enter would); Esc cancels.
  - M4: `Services::cart_tracks`; a cart's track that is analysed, readable and has no format is analysed at once even after "Later". A cart track with a format but an older analysis version still waits.
  - M5: `Bridge::analysis` analyses a track the cache has no current entry for, on the request's blocking thread, one at a time (`Bridge::on_demand`), and caches it. A track that is not analysed is still `404 not_analyzed`; an undecodable file is `404 not_found`.
  - M6: `services::OutdatedCount` counts once per model snapshot; Settings gets it through `SettingsDeps::outdated`.
  - M7: `ALIGNMENT_LINE_THICKNESS`; `fp_remote::{Timing, spawn_with}` and a `bind_log` test that waits for four published status snapshots instead of sleeping 5.5 s.
  - M2 of the remote plan (Matroska Opus seek to 0 drops the pre-skip by timestamp, ±24 samples, 0.5 ms) stays a recorded ruling; nothing changed.
  - Missing-file M2: the reason tooltip is also on the title label of an unavailable row.
  - Docs: `players.md` says which meters have a peak hold (checked against `MeterBallistics::settings()`).
  - Thread priority: `fp_decode::priority` over `thread-priority` 3.1.1 (MIT; `cargo deny check` passed with no change to `deny.toml`). Decode workers ask for `AboveNormal` (value 60: nice −5, above normal on Windows; 37 on macOS, since the crate checks the value against SCHED_OTHER's 15..=47 range there), the analysis pool for `Low` (nice 19, lowest). A refusal (the usual answer to a raise for an ordinary Linux user) is one `warn` per run. Persistence stays on the services thread; main spec §2.2 says so now. `threading-and-realtime.md` is aligned.
  - Long-file check: a four-hour mono 44.1 kHz WAV (1.27 GB) played in the application under Xvfb, silently (fader at 0, no sound server, no usable ALSA device, so the timeline ran on the virtual clock); the seek to 3:59:00 through `POST /players/{id}/seek` was accepted in 10 ms and the position read 14340.99, 14341.99 and 14343.01 s one, two and three seconds later; seeking back to 100 s and to 14399.5 s worked and the player stopped at the end; no errors in the log. The compressed (MP3) variant was not repeated. No defect found.

## 11. Plan 10 — Audio path

- **O26 Audit first.** Task 1 reviews the whole path from file to device and
  writes the result to `docs/technical/audio-path-audit.md`. Every claim is
  backed by a test or by a line of code. It covers:
  - decoding: errors in the middle of a file, gapless starts and ends, seeking,
    and the decode-ahead margin against slow disks;
  - resampling quality and its passband, and the switch between resampled and
    bit-perfect playback;
  - the mixer: summing, headroom and clipping, and gain ramps on every start,
    stop, pause, seek, fade and fader move (no step in level, so no click);
  - output: the f32 to integer conversion, and dither when the device is not
    bit-perfect and narrower than the source;
  - buffer sizes against underruns, xrun counting and reporting, device loss
    and recovery, and rate changes on bit-perfect devices;
  - real-time safety of the callback (CLAUDE.md rule 5).

  Every defect the audit confirms is fixed test-first in this plan. Anything
  too large becomes its own plan item in the roadmap. Findings that need no
  change are recorded as such.
- **O25 DSD output.**
  - Each bit-perfect device gets a DSD mode, `DsdOutput`:
    - `Pcm` (the default): DSD is converted to PCM, as today;
    - `Dop`: DSD over PCM (DoP 1.1), 24-bit samples at the DSD rate ÷ 16,
      with the alternating 0x05/0xFA markers;
    - `Native`: raw DSD to the device. It is offered only on Linux, through
      ALSA on hardware devices that report a DSD sample format.

    The setting sits next to the device's bit-perfect switch, and only the
    modes the device can take are offered.
  - DSD reaches the device unchanged only on a bit-perfect device that is idle
    when the DSD track starts, and only when the device accepts the rate the
    track needs. DoP for DSD256 needs 705.6 kHz, for example. Otherwise the
    track is converted to PCM, as today, and the log says why. While DSD goes
    out unchanged, the player's **BP** indicator reads **DSD**. The fader and
    the gain stay at unity, and the meters show the level of the PCM
    conversion, computed alongside.
  - Starts, stops and every switch between DSD and PCM send DSD silence
    (0x69) for a configurable time, so that the DAC locks without a pop.
  - **Mixing.** DSD cannot be mixed. What happens when another source needs
    the same output (the segue to the next track, a cart, another player) is
    a setting, `audio.dsd_mix`:
    - `ConvertToPcm` (the default): the DSD track goes on as PCM from that
      moment, mixed as usual, and **BP** goes off.
    - `HoldOthers`: nothing interrupts the DSD stream. The player's next track
      starts only when the DSD track ends, with no overlap. Other sources
      routed to that output are muted on it while the DSD plays, and a notice
      says so.
  - Both settings are `Config` fields with defaults, ranges and lenient
    loading. They are documented in the user guide's bit-perfect page.
- **O34 Readings above 0 dBFS.** The audit explains when the digital peak
  meter reads above 0 dBFS. The mix is floating point, so a sample can exceed
  full scale when a file already has overs, a gain or fader is above unity,
  or several sources sum on one output. It confirms what the device receives
  (the f32 to integer conversion clips), and how the meter shows that. If the
  operator needs more, the audit proposes either a clear over indicator or an
  optional output limiter (off by default, never on a bit-perfect path), and
  the maintainer chooses before any change is built.
- **O27 No output.** The internal `null` backend shows again in the audio
  system list, after the real systems, as "No output (silent)". It discards
  audio at real-time pace. This reverses the hiding part of O5; the name O5
  chose stays.

## 12. Plan 11 — Website and guide

- **Guide.**
  - mdBook builds the user guide straight from `docs/user/`; nothing is
    copied into the repository.
  - The only new file there is `docs/user/SUMMARY.md`, mdBook's table of
    contents. `docs/user/README.md` stays the guide's first page.
  - The build copies `docs/images/` so that image links resolve.
- **Landing page.**
  - `site/index.html` and its assets use the Nocturne colours from
    `ui/theme.rs`. The page shows the screenshot, the main features, and a
    download section with Windows, macOS and Linux tabs.
  - A small script picks the visitor's tab from `navigator.userAgentData` (or
    `navigator.userAgent`). Without JavaScript all three tabs show.
  - The Linux tab lists .deb, .rpm, AppImage, Flatpak and tar.gz for x86_64
    and aarch64.
  - Every tab also links the releases page.
- **Download links.** `scripts/site/build.sh` writes the links at build time
  from the latest release (`gh release view`). Visitors' browsers never call
  the GitHub API.
- **Workflow.**
  - `.github/workflows/pages.yml` builds with a pinned mdBook and deploys with
    `actions/deploy-pages`.
  - It runs on pushes to `master` that touch `docs/user/**`, `docs/images/**`
    or `site/**`, on every published release, and by hand. The site therefore
    follows the docs and every release.
- **Maintainer step.** GitHub Pages must use "GitHub Actions" as its source.
- `README.md` links the website, and `CLAUDE.md` documents the local build
  command.
- **O28 Screenshots in the guide.**
  - Each guide page whose subject is a screen or a window gets a screenshot
    of it: the main screen, a player, the playlist, the cartwall, the CUE
    window, the tag editor, Settings and its sections, and the remote API
    page if it has a UI.
  - Each is taken in English, in Xvfb, from a scripted scene, as CLAUDE.md
    describes, and saved under `docs/images/guide/`.
  - A script regenerates them, so that they follow the UI.
- **O29 README screenshot.**
  - In one of the two playing players, the entry marked next is 3 or 4
    entries after the one on air, not the following one.
  - Every track in the scene has its own waveform: the generated tones
    differ in their envelope, so that no two waveforms look alike.
- **O30 Repository About.** The GitHub description is refreshed, and the
  website URL is set as the repository homepage (`gh repo edit
  --description … --homepage …`) once the site is live. README and the
  site link each other.

## 13. Plan 12 — Window and layout

- **O31 Cartwall without scrolling.** The configured rows always fit the
  cartwall area: the cart buttons take their height from the space there is,
  down to a minimum height. The area scrolls only when the rows do not fit
  even at that minimum.
- **O32 One title bar.** The native title bar shows the icon, the name and
  the version ("Fauste Player 1.2.3") on Windows, Linux and macOS. The app's
  own top bar no longer repeats the icon, the name or the version; it keeps
  everything else it shows.
- **O33 Fixed-width times.** Every time that changes while it is shown (the
  cart countdowns, the intro and outro countdowns, elapsed and remaining
  times) keeps the same width: digits use tabular (fixed-width) figures, or
  the text sits in a box sized for its longest value, so nothing beside it
  moves.
- **O35 Playlist tabs.** Tabs shrink, down to a minimum width, and cut long
  names with "…"; the full name shows in a tooltip. When the tabs still do
  not fit, the tab strip scrolls sideways (wheel and arrow buttons at its
  ends), and the selected tab always stays in view.
- **O36 No console on Windows.** The Windows executable is a GUI program
  (`windows_subsystem = "windows"`), so no console window opens with it. The
  plan keeps `--version` and `--help` useful without `unsafe` code (for
  example, debug builds stay console programs), and checks that CI and the
  packaging scripts do not depend on console output from the release binary.
- **As built.**
  - O31: `cart_view::button_height` gives the button height from the room the
    grid has, between `MIN_BUTTON_HEIGHT` and `BUTTON_HEIGHT` (40 px, as
    before); the minimum is 28 px, not 24, because at 24 the name and detail
    lines overlap. The 60% cap is unchanged. The grid zeroes the vertical item
    spacing, so `GAP` is the only row gap and the grid does not overflow.
  - O32: `cli::window_title` gives the native title ("Fauste Player <version>")
    and the icon on Windows and Linux; the top-bar brand block and the Fluent
    key `tip-about-name` are removed. macOS shows the name and the version
    only: its title bars draw no icon, by design. `docs/images/main-screen.png`
    still shows the old brand block; it is refreshed at the next screenshot
    update.
  - O33: `paint_tabular_right`, `time_badge_width` and `time_badge` in
    `widgets`. Changed: the cart countdown, the waveform hover time, the
    marker-drag time, the top-bar clock and the intro and outro badges (value
    box sized for `00.0`). Left alone because they were already tabular: the
    player elapsed, total and remaining times, the big countdown and the CUE
    window times.
  - O35: `ui/tab_strip.rs` (pure layout) with a 72 px minimum tab width, equal
    widths, a name cut with "…" (full name in the tooltip), arrow buttons and
    the wheel; `ViewState::tab_scroll` keeps the offset and a reveal key brings
    the shown tab into view only when the shown playlist changes, so a manual
    scroll is not undone.
  - O36: `windows_subsystem = "windows"` for release builds on Windows (debug
    builds keep a console); `cli::emit` prints on a console and shows a message
    box otherwise, used for `--version`, `--help` and start-up errors;
    `scripts/check-windows-gui.sh` checks the PE Subsystem field and runs from
    `scripts/package/windows.sh`. No workflow or packaging script depends on
    console output from the Windows release binary. The check by hand on a real
    Windows machine (no console on double-click; `--version` shows a message
    box) has not been done yet. The ignored-argument warning is logged once
    logging is up, so the GUI build keeps it in the log.

## 14. Plan 13 — Player rules

- **O37 Play the current entry once more.** "Set as next" is accepted for
  the entry that is on air (the `NextIsCurrent` refusal goes). The player
  plays that entry once more from its cue-in when the current pass ends,
  with the same gapless hard transition as a repeating entry (main spec
  rule 27), and then goes on with the entry that follows it. Unlike
  `repeat`, it acts once: the second pass is an ordinary play (it becomes
  current again, is marked played and enters the history), and the next
  entry is then worked out as usual. Everything that already wins over the
  next entry still wins: Single mode, stop after current, an entry's "stop
  after", a fade stop. In Single mode the player stops at the end and Play
  starts the entry again. The playlist marks the playing row as next too,
  and its "Set as next" action is enabled on that row. The remote API and
  MIDI follow the same rule.
- **O38 Stop after current in Single mode.** In Single mode, while the
  current entry has `repeat`, the stop-after-current control is available:
  it ends the repeat at the cue-out of the pass that is playing (main spec
  rule 27 already lets stop-after-current end a repeat) and shows the same
  notice as in Continuous mode. Switching to Single mode keeps the flag while
  the current entry repeats. In Single mode without a repeating entry it is
  refused as today (`StopAfterInSingle`, the remote API's `409`), since the
  player stops at the end anyway.
- **As built.**
  - **O37.** `set_next` accepts the entry on air; the self-next is `next == current` with `next_explicit` (a derived next never wraps, so it is always an operator choice). `plan_for` gives `StartNextAt` at cue-out, hard, never a segue (`StopAt` only as a fallback for an unplayable file); `preload_target` is unchanged, and a too-broad playable filter in it was removed so that an ordinary next whose file went missing is still preloaded. `on_event(TransitionStarted)` tells a repeat pass (`repeating`) from the replay (`next == entry`), and the replay runs `advance_to`, the ordinary play bookkeeping. `NextIsCurrent` and its message are gone. Repeat wins over a self-next, and the self-next survives Stop. Failing preloads and sources, and a `SetFileState` marking the file missing or unreadable (added after the final review: it stopped a Continuous player at cue-out), replace a self-next by the following playable entry; the session restore keeps an explicit self-next. After replay then leaving, the history is [e, e] (it records entries left). The engine is unchanged (tests only). UI: the playing row keeps the on-air look and also shows the next arrow; the context menu "Set as next" is enabled on it (disabled when the row already is the explicit next); double-click on it still does nothing; the CUE window's "Load as next" accepts the entry on air.
  - **O38.** `repeat_entry` (R26 without the flag test) guards `ToggleStopAfterCurrent`, `SetMode`, `availability` and the restore; `reconcile` drops a Single-mode flag as soon as the entry stops repeating. The remote API and MIDI follow `availability`; no code of their own (MIDI has no stop-after or set-next action). The tooltip `tip-stop-after-single` shows only when the control is unavailable.

## 15. Global constraints

`CLAUDE.md` rules 1–10 apply to every plan. In particular:

- English everywhere, and UI strings in both locales;
- no product names;
- operator values are `Config` fields with defaults, ranges and lenient
  loading;
- the real-time path never allocates, locks, logs or panics;
- behaviour lives in `fp-model`;
- the UI never blocks (tag writing, restart and file work run on helper
  threads);
- bad data never crashes;
- nothing goes on air by itself.

TDD throughout. Each plan ends with a docs task and a review by a fresh
reviewer.
