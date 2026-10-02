# Track Table (Feedback 2, Plan 8) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Every player's track table opens scrolled to its next entry (O7), draws the per-entry repeat and stop icons before the title (O9), recomputes all its columns while a column edge is dragged (O16), has a **Reset played** button that clears the played marks of its playlist after a question (O22), and shows the columns the operator chose, in the order the operator chose, from one global list that can be edited from Settings, from the table header's menu and by dragging a header (O24). The Date and Genre columns use the tag fields plan 7 added.

**Architecture:**
- **Model first.** `fp-model` gets `TableColumn` and the pure rules on the ordered column list (`normalize_columns`, `with_column_shown`, `move_column`, `move_column_before`, `column_rows`); the list is `ui.table_columns` in `Config`, repaired by `Config::validate` and loaded leniently (the store's per-element loader already drops unknown names). `ColumnWidths` becomes fractions keyed by column and converts the old four-number form. `Command::ResetPlayed(PlaylistId)` and the pure `can_reset_played` / `resettable_entries` carry O22. `Track::intro_secs` feeds the Intro column.
- **The table owns its widths.** `ui/table_layout.rs` holds the pure functions from stored fractions to pixels (`column_px`, `fit`), from a dragged edge to the widths of all columns (`resize_px`) and back (`fractions_of`). The table gives every column `Column::exact(px)` on every frame and draws its own resize handles (see the O16 decision below); `TableBuilder` keeps only the virtualised rows, the header row and `scroll_to_row`.
- **UI.** `ui/table.rs` draws the configured columns, the header's drag-to-reorder and context menu; `ui/reset_played.rs` is the question; `ui/settings/columns.rs` is the Settings list; `player.rs` adds the footer button and the start-up scroll; `view.rs` gains `cell_text` and `start_scroll_target`.

**Tech Stack:** Rust, egui/eframe 0.36.2, egui_extras 0.36.2 (`TableBuilder`), egui_kittest 0.36.2. No new dependency, so `cargo deny check` needs no new entry.

**Spec:** `docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md` §9 (items O7, O9, O16, O22, O24) and §13 (global constraints; §12 before the O25–O30 renumbering). Roadmap: `docs/superpowers/plans/2026-10-01-operator-feedback-2-roadmap.md` (plan 8, branch `feat/table-columns`). Plan 7 (merged) added `Track::{date, genre, album_artist, composer, comment}`, which the Date and Genre columns use.

## Decision O16: the table draws its own resize handles

The spec asks Task 1 to find out whether `egui_extras::TableBuilder` can recompute the other columns while an edge is dragged. This was settled while writing the plan, by reading `egui_extras` 0.36.2 (`src/table.rs`, `src/sizing.rs`), so no task is a research step.

- **It cannot.** In `Table::body`, the drag handler of a resizable column changes only the dragged column's width (`*column_width = new_width`), after the header and the body were laid out with the widths of the previous frame; the other columns are never touched. `TableState::load` keeps resizable columns at their stored width on the next frame (`Size::exact(*prev_width)`), so they do not follow either. The only column that follows is a trailing `Column::remainder()`, and the spec needs all of them to follow. `Column::remainder()` in the middle is treated as resizable and holds its width, which is why the old table needed `TableBuilder::reset()` and a copy of the layout in `ViewState` to apply stored fractions.
- **It cannot be steered from outside either.** The handle's response id is `state_id.with("resize_column").with(i)` (the header even derives it from `ui.id()` instead of `state_id`), both private; reading the drag through `Context::read_response` would depend on that derivation across egui_extras versions.
- **Decision.** Every column is `Column::exact(px)` with `.resizable(false)`, and `px` comes from `table_layout::column_px` on every frame, so egui keeps no width of its own (no `reset()`, no `ViewState::table_layout`, no `ViewState::resizing`). The table draws its own grab zones on the column edges after the body (`resize_handles`, so they win over the rows, as the table's own handles did). A press on one starts a `LiveResize`; from then on `live_widths` runs at the start of every frame, before anything is drawn, reads the raw pointer and computes the widths of all the columns with the pure `resize_px`, so the drag shows in the same frame as the pointer move. On release it sends `Command::SetColumnWidths` once with the fractions of the shown columns (`fractions_of`), only if something moved, and keeps drawing the released widths for up to half a second until the model has them (`HOLD_SECS`). A drag whose columns or table width changed meanwhile is dropped.
- **Consequences.** Double-click on an edge no longer auto-sizes (all columns clip, so "the content's width" meant little). While a drag runs the columns on the right of the edge share what is left in proportion to their width when the drag began; the ones on the left do not move.

Related finding, fixed in Task 5: `TableBuilder::min_scrolled_height` defaults to 200 points. In a window short enough that the table body is less than 200 points high, the footer is drawn over the last rows and no scrolling brings them out. The table sets `min_scrolled_height(0.0)`.

## Decisions taken

The spec leaves these open. Each is the most conservative reading.

- **"Centred".** The next entry's row is scrolled to the middle of the table body (`scroll_to_row(i, Some(Align::Center))`, without the glide). When the entry is within half a screen of either end the scroll area clamps the offset, so the table shows its first or last rows instead of padding the list; that is the visible-and-centred-as-far-as-it-can reading, and it is what the table already did for `Align::TOP` at the end of a list.
- **"At start-up".** The first frame in which a player's column is drawn (so a player added later gets it once too). Both an explicit and a derived next count. It scrolls only when that entry is in the playlist the player's tab shows: the tab is never switched for it (following the current entry does switch, because that entry is on air). Afterwards the operator and the existing follow-the-current rule own the scroll position.
- **ResetPlayed while an entry plays.** "Except the current one" is read as: the current entry of *any* player keeps its marks (its row is drawn as the current one, and the mark is added when the entry is left, as always). That includes marks other players made on that entry. Marks of other playlists, the repeat and stop-after flags, the players' histories and every explicit next are untouched. A derived next is recomputed with `refresh_next`, the existing rule; no rule reads the marks today, so it stays as it was. An unknown playlist is `ModelError::UnknownPlaylist` and changes nothing.
- **The Reset played button** is enabled only when `can_reset_played` (some entry that is not current has a mark); the question's Esc, Cancel and backdrop click cancel; no shortcut and no file drop acts while it is open; it closes by itself when its playlist is deleted. It is not exposed through the remote API or MIDI.
- **The column list is global**, as the spec says: one `ui.table_columns` for every player and playlist. Widths stay per player (they were, in `session.json`).
- **A repaired list.** Duplicates: the first wins. A missing Title goes first, or right after `#` when `#` leads the list; a missing Duration goes last; an empty list therefore becomes Title and Duration. Unknown names are dropped by the store's lenient loader (it keeps every element that parses); a value that is not a list gives the default. `Config::validate` repairs the rest and says so. The default list is the one the table always had: `#`, Title, Artist, Duration.
- **Showing a column** appends it at the end of the list; the operator moves it from there. Hiding a required column does nothing.
- **Old widths.** `{"fractions": [a, b, c, d]}` maps cleanly when it is exactly four numbers: they become `#`, Title, Artist and Duration. Not exactly four numbers, or a pixel form from before F6, or any other type, gives the default layout. Broken numbers (not finite, negative, all zero) give the default layout. Unknown column names in the new form are dropped.
- **Widths and the shown list.** The fractions are those of the columns shown when they were stored. A column shown later has no fraction and takes its default width, and the others keep their proportions in what is left; a hidden column's room goes to the others in proportion. The next drag stores the shown columns only.
- **Default widths and minimums.** `#`, Date, Duration and Intro take their minimum (`#` grows with the digits of the playlist length, Date 84 points, the times 52); the text columns share the rest as Title 3, Artist 2, Album 2, File name 2, Genre 1.5, which for the default list is the old 60:40. Minimums win; when even they do not fit they are scaled down.
- **Intro** is the time from the start of the audible part (the cue-in, when cue markers are in use) to the intro-end marker, empty when there is no marker or it is not after the start.
- **Header gestures.** Any column can be dragged, required or not. A drop on the left half of a header cell puts the column before it, on the right half after it; a drop anywhere else cancels. The resize grab zones sit over the header edges and win over the header drag. The header menu lists the optional columns only (checkable); choosing one closes the menu.
- **Settings list.** The shown columns in their order, then the hidden ones in the fixed order of `TableColumn::ALL`. Only shown columns have arrows (off at the ends). Title and Duration are ticked and disabled, with a tooltip. **Default columns** is off while the list is the default. Playlists has no "Restore defaults" in its section header (no `SettingsSection` for it), and this plan adds none.
- **Cells.** Date shows the stored ISO text; an empty field is an empty cell; Artist keeps its "Unknown artist" text. The `P2` mark of an entry on air in another player and the status icons stay in the `#` column; without that column they are not shown (the row colours remain).
- **The outdated-analysis flag.** It stays at the right of the title; only the repeat and stop-after icons move before it.

## Global Constraints

- All code, identifiers, comments, docs, specs, plans and commit messages are in English. Never mention other playout, radio-automation or tag-editor products.
- Spec §9 (binding; the spec's own words):

> - **O7 Scroll to next.** When the app opens, each player's table scrolls so
>   that its next entry is visible and centred. This happens once, at start-up.
> - **O9 Icon position.** The per-entry repeat and stop icons are drawn before
>   the title, not after it.
> - **O16 Live resize.** While a column edge is dragged, the other columns are
>   recomputed every frame. The widths are stored on release, as today. Task 1
>   checks whether `egui_extras::TableBuilder` allows this or whether the table
>   draws its own resize handles.
> - **O22 Reset played.**
>   - A button next to "Add tracks", with a confirmation ("Clear the played mark
>     of every track in this playlist?").
>   - `Command::ResetPlayed(PlaylistId)` clears the played mark of every entry
>     except the current one. A non-explicit next is recomputed by the existing
>     rules.
> - **O24 Columns.**
>   - A global, ordered list `ui.table_columns: Vec<TableColumn>`, loaded
>     leniently: unknown columns are dropped, and missing required columns are
>     added back.
>   - Required columns: Title and Duration. Optional columns: `#`, Artist,
>     Album, Date, Genre, Intro, File name.
>   - All columns, required or not, can be reordered:
>     - by dragging a header in the table;
>     - in Settings > Playlists, with a checkbox list and up and down buttons.
>   - The table header's context menu also shows and hides the optional columns.
>   - Column widths become fractions keyed by column. Widths stored in the old
>     `[f32; 4]` form are converted when they map cleanly; otherwise the default
>     layout is used.

(The O16 bullet's "Task 1 checks" is settled by the decision above; Task 6 builds the result.)

- Spec §13 (the former §12; binding):

> `CLAUDE.md` rules 1–10 apply to every plan. In particular:
>
> - English everywhere, and UI strings in both locales;
> - no product names;
> - operator values are `Config` fields with defaults, ranges and lenient
>   loading;
> - the real-time path never allocates, locks, logs or panics;
> - behaviour lives in `fp-model`;
> - the UI never blocks (tag writing, restart and file work run on helper
>   threads);
> - bad data never crashes;
> - nothing goes on air by itself.
>
> TDD throughout. Each plan ends with a docs task and a review by a fresh
> reviewer.

- CLAUDE.md rule 4: every operator value is a `Config` field with a documented default, a range in `Config::validate` and lenient loading (here `ui.table_columns`, repaired by `validate`; the widths are session data, loaded leniently by `ColumnWidths`).
- CLAUDE.md rule 6: no `unwrap`, `expect` or `panic` outside tests; prefer `get` to indexing in `fp-app` code.
- CLAUDE.md rules 8 and 9: the UI never blocks and bad data degrades (widths, column lists and sessions from hand-edited or older files).
- Behaviour lives in `fp-model` as pure functions with one test per rule (`normalize_columns`, `with_column_shown`, `move_column`, `move_column_before`, `column_rows`, `resettable_entries`, `can_reset_played`, `ColumnWidths::normalized`, `Track::intro_secs`); the UI only displays and sends commands. The layout arithmetic is pure too (`table_layout`).
- UI strings are Fluent messages in `crates/fp-app/locales/en-US/main.ftl` (source) and `es-ES/main.ftl`, always both (`tests/i18n.rs::both_locales_define_the_same_keys` checks it).
- Commands: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p <crate> --test <file>` (one filter per command). Build only into the repo's `target/`.
- Commit only when fmt, clippy and the whole suite pass: `export PATH=$HOME/.cargo/bin:$PATH; cargo fmt --all --check && cargo clippy --workspace --all-targets -q -- -D warnings && cargo test --workspace -q`. Commits end with the co-author trailer the harness provides (the commit commands below show the subject only). `CHANGELOG.md` is never edited by hand.
- Documentation is part of the change (Task 8).

## Review Focus

Failure modes the spec implies and its tests do not name; each has a test in the task named.

1. **Odd widths and column lists in a file** (NaN, negative or huge fractions, unknown or repeated columns, an empty list, a list or widths of the wrong type, the pixel form from before the proportional columns): the table must show a sensible layout, never NaN or negative pixels, never a panic. (Task 1 `unknown_columns_are_dropped_and_the_others_kept`, `a_table_columns_value_that_is_not_a_list_loads_as_the_default`, `an_empty_list_keeps_only_the_required_columns`; Task 6 `an_old_array_that_does_not_map_cleanly_gives_the_default_layout`, `a_value_of_the_wrong_type_gives_the_default_layout`, `nonsense_widths_never_give_nonsense_pixels`, `a_list_without_the_required_columns_still_shows_them`.)
2. **Many columns in the narrowest player** (nine columns at 380 points, a playlist of thousands of entries that widens `#`): the minimums win and are scaled down, nothing overflows, nothing goes negative. (Task 6 `nine_columns_in_the_narrowest_player_never_overflow_or_go_negative`, `the_minimums_win_in_a_narrow_table`, `fit_scales_the_minimums_when_nothing_fits`, `every_step_of_a_drag_keeps_the_total_and_the_minimums`.)
3. **An interrupted or empty drag** (a click on an edge, a drag back to where it began, the column list changed or the window resized while dragging): nothing is stored, the stale drag is dropped. (Task 6 `a_plain_click_on_an_edge_stores_nothing`, `a_drag_back_to_where_it_began_restores_the_widths`, `changing_the_columns_during_a_drag_ends_it_without_storing_anything`, `the_window_growing_does_not_store_widths_and_keeps_the_shares`.)
4. **Gestures that look alike** (a track dragged over the header, a header dropped on the rows, a drag that starts on a column edge, a click near an edge): no column moves for a track, no entry moves for a column, an edge drag never reorders. (Task 6 `a_track_dragged_over_the_header_reorders_no_column`; Task 7 `dropping_a_header_on_itself_or_outside_the_header_changes_nothing` (which also asserts that no entry moves), `dragging_from_a_column_edge_resizes_and_never_reorders`.)
5. **Reset played at the edges** (the entry that is current on one player marked by another, the playlist deleted while the question is open, nothing to clear, an unknown playlist, a shortcut pressed under the question). (Task 2 `the_current_entry_is_left_as_it_is_while_it_plays`, `an_unknown_playlist_is_refused_and_changes_nothing`, `a_mark_on_the_current_entry_alone_does_not_enable_the_button`; Task 3 `the_question_closes_when_the_playlist_is_deleted`, `no_shortcut_acts_under_the_question`.)
6. **The start-up scroll at its edges** (next in another playlist, an empty playlist, a short list, near either end, a short window, a later change of next): no tab switch, no crash, once only, the last row reachable. (Task 5 `a_next_in_another_playlist_does_not_switch_the_tab_or_scroll`, `an_empty_list_is_fine`, `near_the_end_the_table_scrolls_as_far_as_it_can`, `the_last_row_is_reachable_in_a_short_window`, `it_happens_once_a_later_next_does_not_scroll`.)

## File Structure

- Create `crates/fp-model/src/columns.rs` (`TableColumn`, the list rules) and `reset_played.rs` (`resettable_entries`, `can_reset_played`, the reducer helper); modify `lib.rs`, `config.rs` (`UiConfig::table_columns`, `validate`), `command.rs` (`ResetPlayed`), `reducer.rs`, `player.rs` (`ColumnWidths` keyed by column), `session.rs`, `track.rs` (`Track::intro_secs`).
- Create `crates/fp-model/tests/table_columns.rs`, `reset_played.rs`, `column_widths.rs`, `intro_column.rs`; modify `tests/editing.rs` and `tests/session.rs` for the new `ColumnWidths`.
- Modify `crates/fp-store/src/lenient.rs` (tests only: the lenient loader needs no change).
- Create `crates/fp-app/src/ui/table_layout.rs` (pure widths), `ui/reset_played.rs` (the question) and `ui/settings/columns.rs` (the Settings list); modify `ui.rs`, `ui/app.rs` (`ViewState` fields, `FollowScroll`, `DragColumn`, `Scene::set_table_columns`, the modal and its keyboard gate), `ui/player.rs` (footer button, start-up scroll), `ui/table.rs` (the table), `ui/view.rs` (`cell_text`, `start_scroll_target`; `column_px` moves to `table_layout`), `ui/settings.rs`.
- Create `crates/fp-app/tests/reset_played_ui.rs`, `table_icons.rs`, `table_start.rs`, `table_layout.rs`, `table_columns_ui.rs`, `table_header_ui.rs`; modify `tests/main_screen.rs` and `tests/view.rs`.
- Modify `crates/fp-app/locales/en-US/main.ftl`, `es-ES/main.ftl`.
- Modify docs (Task 8): `docs/user/playlists.md`, `docs/user/settings.md`, `docs/user/data-and-backups.md`, `docs/technical/ui.md`, `docs/technical/persistence.md`, the spec (status and "As built" under §9), `docs/superpowers/plans/2026-10-01-operator-feedback-2-roadmap.md` (row 8 done), `README.md`.

---

### Task 1: The column list: `TableColumn`, its rules and `ui.table_columns` (model)

**Files:**
- Create: `crates/fp-model/src/columns.rs`
- Modify: `crates/fp-model/src/lib.rs`, `crates/fp-model/src/config.rs`
- Test: `crates/fp-model/tests/table_columns.rs` (create), `crates/fp-store/src/lenient.rs` (its test module)

**Interfaces:**
- Consumes: `Config::validate` (the existing `ConfigWarning` mechanism), the store's `config_from_value` (no change: it already keeps every array element that parses and drops the others with a warning).
- Produces (all re-exported from `fp_model`):
  - `enum TableColumn { Number, Title, Artist, Album, Date, Genre, Duration, Intro, FileName }` (`Copy`, `Ord`, `Hash`, serde `snake_case`: `"file_name"`), with `TableColumn::ALL: [TableColumn; 9]`, `is_required(self) -> bool` (Title, Duration), `name(self) -> &'static str` (the file name of the column and the suffix of the `col-` and `column-name-` messages) and `from_name(&str) -> Option<TableColumn>`.
  - `default_columns() -> Vec<TableColumn>` (`#`, Title, Artist, Duration), `normalize_columns(&[TableColumn]) -> Vec<TableColumn>`, `with_column_shown(&[TableColumn], TableColumn, bool) -> Vec<TableColumn>`, `move_column(&[TableColumn], from: usize, to: usize) -> Vec<TableColumn>`, `column_rows(&[TableColumn]) -> Vec<(TableColumn, bool)>`.
  - `UiConfig::table_columns: Vec<TableColumn>`.

- [ ] **Step 1: Write the failing tests**

`crates/fp-model/tests/table_columns.rs` (new file):

```rust
#![allow(clippy::unwrap_used)]
//! The columns of the track table (feedback 2 spec O24).

use fp_model::TableColumn::{Album, Artist, Date, Duration, FileName, Genre, Intro, Number, Title};
use fp_model::{
    Config, TableColumn, column_rows, default_columns, move_column, normalize_columns,
    with_column_shown,
};

#[test]
fn the_default_is_what_the_table_always_showed() {
    assert_eq!(default_columns(), vec![Number, Title, Artist, Duration]);
    assert_eq!(Config::default().ui.table_columns, default_columns());
}

#[test]
fn title_and_duration_are_the_required_columns() {
    let required: Vec<_> = TableColumn::ALL
        .into_iter()
        .filter(|c| c.is_required())
        .collect();
    assert_eq!(required, vec![Title, Duration]);
    assert_eq!(TableColumn::ALL.len(), 9);
}

#[test]
fn a_repeated_column_keeps_its_first_place() {
    assert_eq!(
        normalize_columns(&[Title, Artist, Title, Duration, Artist]),
        vec![Title, Artist, Duration]
    );
}

#[test]
fn a_missing_title_goes_first_or_after_the_number() {
    assert_eq!(
        normalize_columns(&[Artist, Duration]),
        vec![Title, Artist, Duration]
    );
    assert_eq!(
        normalize_columns(&[Number, Artist, Duration]),
        vec![Number, Title, Artist, Duration]
    );
}

#[test]
fn a_missing_duration_goes_last() {
    assert_eq!(
        normalize_columns(&[Title, Album]),
        vec![Title, Album, Duration]
    );
}

#[test]
fn an_empty_list_keeps_only_the_required_columns() {
    assert_eq!(normalize_columns(&[]), vec![Title, Duration]);
}

#[test]
fn a_good_list_is_not_changed_and_normalising_is_idempotent() {
    let list = vec![Duration, Genre, Title, Number, Intro, FileName, Date, Album];
    assert_eq!(normalize_columns(&list), list);
    let bad = [Artist, Artist];
    let once = normalize_columns(&bad);
    assert_eq!(normalize_columns(&once), once);
}

#[test]
fn showing_a_column_appends_it_and_showing_it_twice_changes_nothing() {
    let list = default_columns();
    let shown = with_column_shown(&list, Album, true);
    assert_eq!(shown, vec![Number, Title, Artist, Duration, Album]);
    assert_eq!(with_column_shown(&shown, Album, true), shown);
}

#[test]
fn hiding_a_column_removes_it_but_never_a_required_one() {
    let list = vec![Number, Title, Artist, Duration];
    assert_eq!(
        with_column_shown(&list, Artist, false),
        vec![Number, Title, Duration]
    );
    assert_eq!(with_column_shown(&list, Title, false), list);
    assert_eq!(with_column_shown(&list, Duration, false), list);
    assert_eq!(with_column_shown(&list, Genre, false), list);
}

#[test]
fn any_column_can_move_even_a_required_one() {
    let list = vec![Number, Title, Artist, Duration];
    assert_eq!(
        move_column(&list, 3, 0),
        vec![Duration, Number, Title, Artist]
    );
    assert_eq!(
        move_column(&list, 1, 2),
        vec![Number, Artist, Title, Duration]
    );
    assert_eq!(move_column(&list, 2, 2), list);
}

#[test]
fn a_move_outside_the_list_is_clamped() {
    let list = vec![Number, Title, Artist, Duration];
    assert_eq!(
        move_column(&list, 0, 99),
        vec![Title, Artist, Duration, Number]
    );
    assert_eq!(
        move_column(&list, 99, 0),
        vec![Duration, Number, Title, Artist]
    );
}

#[test]
fn the_settings_rows_list_the_shown_columns_then_the_hidden_ones() {
    let rows = column_rows(&[Duration, Title]);
    let shown: Vec<_> = rows.iter().filter(|(_, on)| *on).map(|(c, _)| *c).collect();
    let hidden: Vec<_> = rows
        .iter()
        .filter(|(_, on)| !*on)
        .map(|(c, _)| *c)
        .collect();
    assert_eq!(shown, vec![Duration, Title]);
    assert_eq!(
        hidden,
        vec![Number, Artist, Album, Date, Genre, Intro, FileName]
    );
    assert_eq!(rows.len(), 9);
    assert!(rows[..2].iter().all(|(_, on)| *on));
}

#[test]
fn validate_repairs_the_list_and_says_so() {
    let mut c = Config::default();
    c.ui.table_columns = vec![Artist, Artist];
    let warnings = c.validate();
    assert_eq!(c.ui.table_columns, vec![Title, Artist, Duration]);
    assert!(warnings.iter().any(|w| w.field == "ui.table_columns"));
    assert!(
        Config::default()
            .validate()
            .iter()
            .all(|w| w.field != "ui.table_columns")
    );
}

#[test]
fn the_list_is_written_with_stable_names() {
    let json = serde_json::to_string(&vec![Number, FileName, Intro]).unwrap();
    assert_eq!(json, r#"["number","file_name","intro"]"#);
}

#[test]
fn a_column_is_found_by_the_name_it_has_in_files() {
    for column in TableColumn::ALL {
        assert_eq!(TableColumn::from_name(column.name()), Some(column));
    }
    assert_eq!(TableColumn::from_name("file_name"), Some(FileName));
    assert_eq!(TableColumn::from_name("file-name"), None);
    assert_eq!(TableColumn::from_name("bpm"), None);
}
```

`crates/fp-store/src/lenient.rs`:

```diff
--- a/crates/fp-store/src/lenient.rs
+++ b/crates/fp-store/src/lenient.rs
@@ -154,4 +154,42 @@ mod tests {
         assert_eq!(c.remote.http.port, 7380);
         assert_eq!(warnings.len(), 1);
     }
+
+    fn columns(json: &str) -> (Vec<fp_model::TableColumn>, Vec<String>) {
+        let user: serde_json::Value = serde_json::from_str(json).unwrap();
+        let mut warnings = Vec::new();
+        let mut c = config_from_value(&user, &mut warnings);
+        let _ = c.validate();
+        (c.ui.table_columns, warnings)
+    }
+
+    #[test]
+    fn a_config_without_table_columns_gets_the_default() {
+        let (list, warnings) = columns(r#"{"ui":{"language":"es-ES"}}"#);
+        assert_eq!(list, fp_model::default_columns());
+        assert!(warnings.is_empty(), "{warnings:?}");
+    }
+
+    #[test]
+    fn unknown_columns_are_dropped_and_the_others_kept() {
+        use fp_model::TableColumn::{Album, Duration, Title};
+        let (list, warnings) =
+            columns(r#"{"ui":{"table_columns":["title","bpm","album","duration"]}}"#);
+        assert_eq!(list, vec![Title, Album, Duration]);
+        assert_eq!(warnings.len(), 1, "{warnings:?}");
+    }
+
+    #[test]
+    fn missing_required_columns_are_added_back() {
+        use fp_model::TableColumn::{Artist, Duration, Title};
+        let (list, _) = columns(r#"{"ui":{"table_columns":["artist"]}}"#);
+        assert_eq!(list, vec![Title, Artist, Duration]);
+    }
+
+    #[test]
+    fn a_table_columns_value_that_is_not_a_list_loads_as_the_default() {
+        let (list, warnings) = columns(r#"{"ui":{"table_columns":"title"}}"#);
+        assert_eq!(list, fp_model::default_columns());
+        assert_eq!(warnings.len(), 1, "{warnings:?}");
+    }
 }
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `cargo test -p fp-model --test table_columns` — expected: does not compile (`TableColumn` and `normalize_columns` are not in `fp_model`).
Run: `cargo test -p fp-store --lib lenient` — expected: does not compile (`fp_model::TableColumn` does not exist; `UiConfig` has no `table_columns`).

- [ ] **Step 3: Implement the rules**

`crates/fp-model/src/columns.rs` (new file):

```rust
//! The columns of the track table (feedback 2 spec O24): which exist, which
//! are required, and the pure rules that edit the ordered list the operator
//! keeps in `ui.table_columns`.

use serde::{Deserialize, Serialize};

/// A column of the track table. The names are the names of the data they
/// show; `Number` is the `#` column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TableColumn {
    /// The position of the entry in its playlist, with its status icon.
    Number,
    Title,
    Artist,
    Album,
    Date,
    Genre,
    Duration,
    /// How long the intro lasts, from the start of the audible part.
    Intro,
    FileName,
}

impl TableColumn {
    /// Every column, in the order Settings lists the hidden ones.
    pub const ALL: [TableColumn; 9] = [
        TableColumn::Number,
        TableColumn::Title,
        TableColumn::Artist,
        TableColumn::Album,
        TableColumn::Date,
        TableColumn::Genre,
        TableColumn::Duration,
        TableColumn::Intro,
        TableColumn::FileName,
    ];

    /// Title and Duration are always shown.
    pub fn is_required(self) -> bool {
        matches!(self, TableColumn::Title | TableColumn::Duration)
    }

    /// The name used in files and as the suffix of the `col-` messages.
    pub fn name(self) -> &'static str {
        match self {
            TableColumn::Number => "number",
            TableColumn::Title => "title",
            TableColumn::Artist => "artist",
            TableColumn::Album => "album",
            TableColumn::Date => "date",
            TableColumn::Genre => "genre",
            TableColumn::Duration => "duration",
            TableColumn::Intro => "intro",
            TableColumn::FileName => "file_name",
        }
    }
}

impl TableColumn {
    /// The column called `name` in a file, if this version has one.
    pub fn from_name(name: &str) -> Option<TableColumn> {
        TableColumn::ALL.into_iter().find(|c| c.name() == name)
    }
}

/// The columns of a table that was never configured.
pub fn default_columns() -> Vec<TableColumn> {
    vec![
        TableColumn::Number,
        TableColumn::Title,
        TableColumn::Artist,
        TableColumn::Duration,
    ]
}

/// A column list the table can use: every column at most once (the first
/// wins) and the required ones present. A missing Title goes first, after
/// `#` when that leads the list; a missing Duration goes last.
pub fn normalize_columns(list: &[TableColumn]) -> Vec<TableColumn> {
    let mut out: Vec<TableColumn> = Vec::with_capacity(list.len() + 2);
    for column in list {
        if !out.contains(column) {
            out.push(*column);
        }
    }
    if !out.contains(&TableColumn::Title) {
        let at = usize::from(out.first() == Some(&TableColumn::Number));
        out.insert(at, TableColumn::Title);
    }
    if !out.contains(&TableColumn::Duration) {
        out.push(TableColumn::Duration);
    }
    out
}

/// `list` with `column` shown (appended at the end) or hidden. A required
/// column cannot be hidden.
pub fn with_column_shown(
    list: &[TableColumn],
    column: TableColumn,
    shown: bool,
) -> Vec<TableColumn> {
    let mut out = normalize_columns(list);
    let present = out.contains(&column);
    if shown && !present {
        out.push(column);
    } else if !shown && present && !column.is_required() {
        out.retain(|c| *c != column);
    }
    out
}

/// `list` with the column at position `from` moved so that it ends at
/// position `to` (both clamped to the list). Any column can move.
pub fn move_column(list: &[TableColumn], from: usize, to: usize) -> Vec<TableColumn> {
    let mut out = normalize_columns(list);
    if out.is_empty() {
        return out;
    }
    let last = out.len() - 1;
    let (from, to) = (from.min(last), to.min(last));
    let column = out.remove(from);
    out.insert(to, column);
    out
}

/// The rows of the Settings list: the shown columns in their order (`true`),
/// then the hidden ones in the order of [`TableColumn::ALL`] (`false`).
pub fn column_rows(list: &[TableColumn]) -> Vec<(TableColumn, bool)> {
    let shown = normalize_columns(list);
    let hidden = TableColumn::ALL.iter().filter(|c| !shown.contains(c));
    shown
        .iter()
        .map(|c| (*c, true))
        .chain(hidden.map(|c| (*c, false)))
        .collect()
}
```

`crates/fp-model/src/lib.rs`:

```diff
--- a/crates/fp-model/src/lib.rs
+++ b/crates/fp-model/src/lib.rs
@@ -5,6 +5,7 @@
 pub mod availability;
 mod cart_rules;
 pub mod cartwall;
+pub mod columns;
 pub mod command;
 pub mod config;
 mod entry_notice;
@@ -31,6 +32,9 @@ pub use cartwall::{
     Cart, CartEdit, CartFileChange, CartKind, CartPage, CartPageImport, Cartwall, CartwallSession,
     PlayingCart,
 };
+pub use columns::{
+    TableColumn, column_rows, default_columns, move_column, normalize_columns, with_column_shown,
+};
 pub use command::{
     CartRequest, Command, EngineAction, EngineEvent, SOURCE_END, SourceRequest, TransitionPlan,
 };
```

`crates/fp-model/src/config.rs`:

```diff
--- a/crates/fp-model/src/config.rs
+++ b/crates/fp-model/src/config.rs
@@ -6,6 +6,7 @@ use std::path::PathBuf;
 
 use serde::{Deserialize, Serialize};
 
+use crate::columns::{TableColumn, default_columns, normalize_columns};
 use crate::ids::PlayerId;
 use crate::player::PlayMode;
 use crate::shortcuts::{Shortcut, default_shortcuts};
@@ -365,6 +366,9 @@ pub struct UiConfig {
     /// playlist), it stops following what plays for this long (seconds);
     /// 0 never follows.
     pub follow_current_grace_secs: f64,
+    /// The columns of the track tables, in order; one list for every player
+    /// and playlist (feedback 2 spec O24). Title and Duration are required.
+    pub table_columns: Vec<TableColumn>,
 }
 
 impl Default for UiConfig {
@@ -374,6 +378,7 @@ impl Default for UiConfig {
             music_dir: None,
             language: None,
             follow_current_grace_secs: 10.0,
+            table_columns: default_columns(),
         }
     }
 }
@@ -601,6 +606,14 @@ impl Config {
             "ui.follow_current_grace_secs",
             &mut w,
         );
+        let columns = normalize_columns(&self.ui.table_columns);
+        if columns != self.ui.table_columns {
+            w.push(ConfigWarning {
+                field: "ui.table_columns",
+                message: format!("{:?} repaired to {columns:?}", self.ui.table_columns),
+            });
+            self.ui.table_columns = columns;
+        }
 
         let a = &mut self.analysis;
         clamp_to(
```

- [ ] **Step 4: Run the tests to see them pass**

Run: `cargo test -p fp-model --test table_columns` — expected: 15 passed.
Run: `cargo test -p fp-store --lib lenient` — expected: 6 passed.

- [ ] **Step 5: Gate and commit**

```bash
export PATH=$HOME/.cargo/bin:$PATH; cargo fmt --all --check && cargo clippy --workspace --all-targets -q -- -D warnings && cargo test --workspace -q && git add crates/fp-model crates/fp-store && git commit -m "feat(model): the table column list and its rules (O24)"
```

### Task 2: `Command::ResetPlayed` (model)

**Files:**
- Create: `crates/fp-model/src/reset_played.rs`
- Modify: `crates/fp-model/src/lib.rs`, `crates/fp-model/src/command.rs`, `crates/fp-model/src/reducer.rs`
- Test: `crates/fp-model/tests/reset_played.rs` (create)

**Interfaces:**
- Consumes: `Playlists::entry_mut`, `PlaylistEntry::{played_by, legacy_played}`, `reducer::refresh_next` (crate-private, already used after every playlist edit).
- Produces:
  - `Command::ResetPlayed(PlaylistId)`: clears the played marks of the playlist except on the current entry of any player; `Err(ModelError::UnknownPlaylist)` for an unknown playlist.
  - `fp_model::resettable_entries(&AppState, PlaylistId) -> Vec<EntryId>` and `fp_model::can_reset_played(&AppState, PlaylistId) -> bool` (what the Reset played button's enabled state uses).

- [ ] **Step 1: Write the failing tests**

`crates/fp-model/tests/reset_played.rs` (new file):

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback 2 spec O22: `Command::ResetPlayed`.

mod common;

use common::{entries, fixture, p0};
use fp_model::{
    AppState, Command, ModelError, PlayerId, PlaylistId, Transport, apply, can_reset_played,
    resettable_entries,
};

/// Plays and stops the first `n` entries on player 0 (each is marked
/// played by it), leaving the player stopped with entry `n` as its next.
fn play_through(s: &mut AppState, n: usize) {
    let (e, p) = (entries(s), p0(s));
    for entry in e.iter().take(n) {
        apply(s, Command::SetNext(p, *entry)).unwrap();
        apply(s, Command::Play(p)).unwrap();
        apply(s, Command::Stop(p)).unwrap();
    }
}

fn marked(s: &AppState) -> Vec<bool> {
    let playlist = s.playlists.first_id().unwrap();
    s.playlists
        .get(playlist)
        .unwrap()
        .entries
        .iter()
        .map(|e| !e.played_by.is_empty())
        .collect()
}

fn playlist(s: &AppState) -> PlaylistId {
    s.playlists.first_id().unwrap()
}

fn reset(s: &mut AppState) {
    let list = playlist(s);
    apply(s, Command::ResetPlayed(list)).unwrap();
}

#[test]
fn every_mark_of_the_playlist_is_cleared() {
    let mut s = fixture(5);
    play_through(&mut s, 3);
    assert_eq!(marked(&s), [true, true, true, false, false]);
    reset(&mut s);
    assert_eq!(marked(&s), [false; 5]);
}

#[test]
fn the_marks_of_every_player_are_cleared() {
    let mut s = fixture(4);
    let (e, p1) = (entries(&s), s.players[1].id);
    play_through(&mut s, 1);
    apply(&mut s, Command::SetNext(p1, e[1])).unwrap();
    apply(&mut s, Command::Play(p1)).unwrap();
    apply(&mut s, Command::Stop(p1)).unwrap();
    assert_eq!(marked(&s), [true, true, false, false]);
    reset(&mut s);
    assert_eq!(marked(&s), [false; 4]);
}

#[test]
fn the_current_entry_is_left_as_it_is_while_it_plays() {
    let mut s = fixture(4);
    let (e, p) = (entries(&s), p0(&s));
    play_through(&mut s, 2);
    // Entry 0 is also marked by player 1, which plays it now.
    let p1 = s.players[1].id;
    apply(&mut s, Command::SetNext(p1, e[0])).unwrap();
    apply(&mut s, Command::Play(p1)).unwrap();
    reset(&mut s);
    assert_eq!(marked(&s), [true, false, false, false]);
    assert_eq!(s.player(p1).unwrap().current, Some(e[0]));
    assert_eq!(s.player(p1).unwrap().transport, Transport::Playing);
    assert_eq!(s.player(p).unwrap().transport, Transport::Stopped);
}

#[test]
fn a_current_entry_is_marked_when_it_is_left_as_usual() {
    let mut s = fixture(3);
    let (e, p) = (entries(&s), p0(&s));
    apply(&mut s, Command::SetNext(p, e[0])).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    reset(&mut s);
    apply(&mut s, Command::Stop(p)).unwrap();
    assert_eq!(marked(&s), [true, false, false]);
}

#[test]
fn another_playlist_keeps_its_marks() {
    let mut s = fixture(3);
    let main = playlist(&s);
    apply(
        &mut s,
        Command::CreatePlaylistFromPaths {
            name: "Other".into(),
            paths: vec!["/music/other.flac".into()],
        },
    )
    .unwrap();
    let other = s.playlists.iter().nth(1).unwrap().id;
    let other_entry = s.playlists.get(other).unwrap().entries[0].id;
    s.playlists.mark_played(other_entry, p0(&s));
    play_through(&mut s, 2);
    apply(&mut s, Command::ResetPlayed(main)).unwrap();
    assert_eq!(marked(&s), [false; 3]);
    assert!(s.playlists.entry(other_entry).unwrap().is_played_by(p0(&s)));
}

#[test]
fn an_explicit_next_and_a_derived_next_both_survive() {
    let mut s = fixture(5);
    let (e, p) = (entries(&s), p0(&s));
    play_through(&mut s, 2);
    assert_eq!(s.player(p).unwrap().next, Some(e[2]), "derived");
    reset(&mut s);
    assert_eq!(s.player(p).unwrap().next, Some(e[2]));
    assert!(!s.player(p).unwrap().next_explicit);
    apply(&mut s, Command::SetNext(p, e[4])).unwrap();
    reset(&mut s);
    assert_eq!(s.player(p).unwrap().next, Some(e[4]));
    assert!(s.player(p).unwrap().next_explicit);
}

#[test]
fn a_derived_next_after_the_current_is_recomputed() {
    let mut s = fixture(4);
    let (e, p) = (entries(&s), p0(&s));
    apply(&mut s, Command::SetNext(p, e[1])).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(s.player(p).unwrap().next, Some(e[2]));
    reset(&mut s);
    assert_eq!(s.player(p).unwrap().next, Some(e[2]));
}

#[test]
fn flags_and_history_are_not_touched() {
    let mut s = fixture(3);
    let (e, p) = (entries(&s), p0(&s));
    apply(&mut s, Command::ToggleEntryRepeat(e[0])).unwrap();
    apply(&mut s, Command::ToggleEntryStopAfter(e[1])).unwrap();
    play_through(&mut s, 2);
    let history = s.player(p).unwrap().history.clone();
    assert!(!history.is_empty());
    reset(&mut s);
    assert!(s.playlists.entry(e[0]).unwrap().repeat);
    assert!(s.playlists.entry(e[1]).unwrap().stop_after);
    assert_eq!(s.player(p).unwrap().history, history);
}

#[test]
fn an_unknown_playlist_is_refused_and_changes_nothing() {
    let mut s = fixture(3);
    play_through(&mut s, 2);
    let before = marked(&s);
    let err = apply(&mut s, Command::ResetPlayed(PlaylistId(999_999))).unwrap_err();
    assert_eq!(err, ModelError::UnknownPlaylist(PlaylistId(999_999)));
    assert_eq!(marked(&s), before);
}

#[test]
fn resetting_twice_or_an_untouched_list_is_harmless() {
    let mut s = fixture(3);
    reset(&mut s);
    play_through(&mut s, 1);
    reset(&mut s);
    reset(&mut s);
    assert_eq!(marked(&s), [false; 3]);
}

#[test]
fn the_button_has_something_to_do_only_when_a_mark_can_be_cleared() {
    let mut s = fixture(3);
    let list = playlist(&s);
    assert!(!can_reset_played(&s, list));
    play_through(&mut s, 1);
    assert!(can_reset_played(&s, list));
    assert_eq!(resettable_entries(&s, list), vec![entries(&s)[0]]);
    apply(&mut s, Command::ResetPlayed(list)).unwrap();
    assert!(!can_reset_played(&s, list));
    assert!(!can_reset_played(&s, PlaylistId(999_999)));
}

#[test]
fn a_mark_on_the_current_entry_alone_does_not_enable_the_button() {
    let mut s = fixture(2);
    let (e, p) = (entries(&s), p0(&s));
    play_through(&mut s, 1);
    let p1: PlayerId = s.players[1].id;
    apply(&mut s, Command::SetNext(p1, e[0])).unwrap();
    apply(&mut s, Command::Play(p1)).unwrap();
    assert_eq!(s.player(p).unwrap().current, None);
    assert!(!can_reset_played(&s, playlist(&s)));
}
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `cargo test -p fp-model --test reset_played` — expected: does not compile (`Command::ResetPlayed`, `can_reset_played` and `resettable_entries` do not exist).

- [ ] **Step 3: Implement**

`crates/fp-model/src/reset_played.rs` (new file):

```rust
//! Feedback 2 spec O22: clearing the played marks of one playlist.

use crate::error::ModelError;
use crate::ids::{EntryId, PlaylistId};
use crate::state::AppState;

/// The entries of `playlist` that `Command::ResetPlayed` would clear: those
/// with a played mark that are not the current entry of any player. The
/// current entry keeps its marks (it is on air, and its row is drawn as the
/// current one); it is marked when it is left, as always.
pub fn resettable_entries(state: &AppState, playlist: PlaylistId) -> Vec<EntryId> {
    let Some(list) = state.playlists.get(playlist) else {
        return Vec::new();
    };
    list.entries
        .iter()
        .filter(|e| !e.played_by.is_empty() || e.legacy_played)
        .filter(|e| !state.players.iter().any(|p| p.current == Some(e.id)))
        .map(|e| e.id)
        .collect()
}

/// Whether the Reset played button has anything to do for `playlist`.
pub fn can_reset_played(state: &AppState, playlist: PlaylistId) -> bool {
    !resettable_entries(state, playlist).is_empty()
}

/// Clears the played marks of `resettable_entries`. History, the next
/// entries and every other mark (other playlists) stay as they are.
pub(crate) fn reset_played(state: &mut AppState, playlist: PlaylistId) -> Result<(), ModelError> {
    if state.playlists.get(playlist).is_none() {
        return Err(ModelError::UnknownPlaylist(playlist));
    }
    for entry in resettable_entries(state, playlist) {
        if let Some(e) = state.playlists.entry_mut(entry) {
            e.played_by.clear();
            e.legacy_played = false;
        }
    }
    // A derived next is recomputed by the existing rules (they do not read
    // the marks today, so it stays; an explicit next is never touched).
    crate::reducer::refresh_next(state);
    Ok(())
}
```

`crates/fp-model/src/lib.rs`:

```diff
--- a/crates/fp-model/src/lib.rs
+++ b/crates/fp-model/src/lib.rs
@@ -17,6 +17,7 @@ pub mod player;
 pub mod playlist;
 pub mod reducer;
 pub mod remote;
+pub mod reset_played;
 pub mod restart;
 pub mod restore;
 pub mod session;
@@ -52,6 +53,7 @@ pub use player::{ColumnWidths, CueState, PlayMode, PlayerState, Transport};
 pub use playlist::{Playlist, PlaylistEntry, Playlists};
 pub use reducer::{apply, on_event, plan_for};
 pub use remote::{HttpRemoteConfig, OscRemoteConfig, RemoteConfig, RemoteEventsConfig};
+pub use reset_played::{can_reset_played, resettable_entries};
 pub use restart::{RestartReason, restart_pending};
 pub use restore::{SettingsSection, restore_defaults};
 pub use session::{PlayerSession, RestoreParts};
```

`crates/fp-model/src/command.rs`:

```diff
--- a/crates/fp-model/src/command.rs
+++ b/crates/fp-model/src/command.rs
@@ -79,6 +79,9 @@ pub enum Command {
         index: usize,
     },
     DuplicateEntry(EntryId),
+    /// Feedback 2 spec O22: clears the played mark of every entry of the
+    /// playlist except the current entry of a player.
+    ResetPlayed(PlaylistId),
     /// R26: the entry repeats until the operator moves on.
     ToggleEntryRepeat(EntryId),
     /// R27: the player stops after the entry, every time it plays.
```

`crates/fp-model/src/reducer.rs`:

```diff
--- a/crates/fp-model/src/reducer.rs
+++ b/crates/fp-model/src/reducer.rs
@@ -197,6 +197,7 @@ pub fn apply(state: &mut AppState, command: Command) -> Result<Vec<EngineAction>
             state.playlists.duplicate(entry, new_id)?;
             refresh_next(state);
         }
+        Command::ResetPlayed(playlist) => crate::reset_played::reset_played(state, playlist)?,
         Command::CreatePlaylist { name } => {
             let id = state.ids.playlist();
             state.playlists.add(Playlist::new(id, name));
```

- [ ] **Step 4: Run the tests to see them pass**

Run: `cargo test -p fp-model --test reset_played` — expected: 12 passed.

- [ ] **Step 5: Gate and commit**

```bash
export PATH=$HOME/.cargo/bin:$PATH; cargo fmt --all --check && cargo clippy --workspace --all-targets -q -- -D warnings && cargo test --workspace -q && git add crates/fp-model && git commit -m "feat(model): ResetPlayed clears the played marks of a playlist (O22)"
```

### Task 3: The Reset played button and its question (UI)

**Files:**
- Create: `crates/fp-app/src/ui/reset_played.rs`
- Modify: `crates/fp-app/src/ui.rs`, `crates/fp-app/src/ui/app.rs`, `crates/fp-app/src/ui/player.rs`, `crates/fp-app/locales/en-US/main.ftl`, `crates/fp-app/locales/es-ES/main.ftl`
- Test: `crates/fp-app/tests/reset_played_ui.rs` (create)

**Interfaces:**
- Consumes: `fp_model::can_reset_played`, `Command::ResetPlayed` (Task 2); `widgets::tile`; the modal pattern of `exit_guard` and Settings' restore question.
- Produces: `ViewState::confirm_reset: Option<PlaylistId>`; `reset_played::show(&egui::Context, &Scene) -> Option<bool>` (`Some(true)` reset, `Some(false)` cancelled, `None` still open); messages `footer-reset-played`, `tip-reset-played` (the button's accessible name), `reset-played-question` (the spec's sentence), `reset-played-cancel`, `reset-played-confirm`.

- [ ] **Step 1: Write the failing tests**

`crates/fp-app/tests/reset_played_ui.rs` (new file):

```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Feedback 2 spec O22: the Reset played button and its question.

mod support;

use std::sync::Arc;

use egui::{Event, Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use fp_app::ui::app::AppUi;
use fp_model::{AppState, Command, PlaylistId, apply};
use support::{Fake, harness, state};

const BUTTON: &str = "Clear the played marks of this playlist";
const QUESTION: &str = "Clear the played mark of every track in this playlist?";

/// The model with the first `n` entries played on P1 (and P1 stopped).
fn played(n: usize) -> AppState {
    let mut s = state(1, 5);
    let p = s.players[0].id;
    let e: Vec<_> = s
        .playlists
        .iter()
        .next()
        .unwrap()
        .entries
        .iter()
        .map(|e| e.id)
        .collect();
    for entry in e.iter().take(n) {
        apply(&mut s, Command::SetNext(p, *entry)).unwrap();
        apply(&mut s, Command::Play(p)).unwrap();
        apply(&mut s, Command::Stop(p)).unwrap();
    }
    s
}

fn playlist(fake: &Fake) -> PlaylistId {
    fake.state.load().playlists.first_id().unwrap()
}

fn button_enabled(h: &Harness<'_, AppUi>) -> bool {
    !h.get_by_label(BUTTON).accesskit_node().is_disabled()
}

fn press(h: &mut Harness<'_, AppUi>, key: Key) {
    h.event(Event::Key {
        key,
        physical_key: Some(key),
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    });
    h.event(Event::Key {
        key,
        physical_key: Some(key),
        pressed: false,
        repeat: false,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(2);
}

fn open(h: &mut Harness<'_, AppUi>) {
    h.get_by_label(BUTTON).click();
    h.run_steps(3);
}

fn marked(fake: &Arc<Fake>) -> usize {
    let s = fake.state.load();
    s.playlists
        .iter()
        .next()
        .unwrap()
        .entries
        .iter()
        .filter(|e| !e.played_by.is_empty())
        .count()
}

#[test]
fn the_button_is_off_while_nothing_is_played() {
    let (h, _) = harness(state(1, 5));
    assert!(!button_enabled(&h));
}

#[test]
fn the_button_is_on_when_a_mark_can_be_cleared() {
    let (h, _) = harness(played(2));
    assert!(button_enabled(&h));
}

#[test]
fn pressing_it_asks_first_and_sends_nothing() {
    let (mut h, fake) = harness(played(2));
    fake.take_sent();
    open(&mut h);
    assert!(h.query_by_label(QUESTION).is_some());
    assert!(fake.take_sent().is_empty());
    assert_eq!(marked(&fake), 2);
}

#[test]
fn confirming_resets_the_playlist_and_closes_the_question() {
    let (mut h, fake) = harness(played(2));
    fake.take_sent();
    open(&mut h);
    h.get_by_label("Reset played").click();
    h.run_steps(3);
    assert_eq!(
        fake.take_sent(),
        vec![Command::ResetPlayed(playlist(&fake))]
    );
    assert!(h.query_by_label(QUESTION).is_none());
    assert_eq!(marked(&fake), 0);
    assert!(!button_enabled(&h), "nothing left to clear");
}

#[test]
fn cancel_changes_nothing() {
    let (mut h, fake) = harness(played(2));
    fake.take_sent();
    open(&mut h);
    h.get_by_label("Cancel").click();
    h.run_steps(3);
    assert!(h.query_by_label(QUESTION).is_none());
    assert!(fake.take_sent().is_empty());
    assert_eq!(marked(&fake), 2);
}

#[test]
fn escape_cancels() {
    let (mut h, fake) = harness(played(2));
    fake.take_sent();
    open(&mut h);
    press(&mut h, Key::Escape);
    assert!(h.query_by_label(QUESTION).is_none());
    assert!(fake.take_sent().is_empty());
}

#[test]
fn no_shortcut_acts_under_the_question() {
    let (mut h, fake) = harness(played(2));
    h.get_by_label("Song 4").click();
    h.run_steps(2);
    open(&mut h);
    fake.take_sent();
    press(&mut h, Key::Delete);
    press(&mut h, Key::Space);
    assert!(h.query_by_label(QUESTION).is_some());
    assert!(fake.take_sent().is_empty(), "Delete and Space did nothing");
}

#[test]
fn the_question_closes_when_the_playlist_is_deleted() {
    let mut s = played(2);
    apply(
        &mut s,
        Command::CreatePlaylist {
            name: "Other".into(),
        },
    )
    .unwrap();
    let (mut h, fake) = harness(s);
    open(&mut h);
    let main = playlist(&fake);
    let mut next = (*fake.state.load_full()).clone();
    apply(&mut next, Command::DeletePlaylist(main)).unwrap();
    fake.state.store(Arc::new(next));
    h.run_steps(3);
    assert!(h.query_by_label(QUESTION).is_none());
    assert!(
        !fake
            .take_sent()
            .iter()
            .any(|c| matches!(c, Command::ResetPlayed(_)))
    );
}

#[test]
fn an_unplayed_current_entry_alone_leaves_the_button_off() {
    let mut s = state(1, 5);
    let p = s.players[0].id;
    apply(&mut s, Command::Play(p)).unwrap();
    let (h, _) = harness(s);
    assert!(!button_enabled(&h));
}
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `cargo test -p fp-app --test reset_played_ui` — expected: FAIL (the button "Clear the played marks of this playlist" is not found).

- [ ] **Step 3: Implement**

`crates/fp-app/src/ui/reset_played.rs` (new file):

```rust
//! The confirmation of Reset played (feedback 2 spec O22): clearing the
//! played marks of a playlist waits for the operator.

use egui::{RichText, vec2};

use super::app::Scene;
use super::theme;
use super::widgets::{self, TileStyle, font};

fn button(ui: &mut egui::Ui, label: &str) -> bool {
    let w = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font(12.0), theme::NEUTRAL_300)
        .size()
        .x
        + 32.0;
    widgets::tile(
        ui,
        vec2(w, 28.0),
        label,
        true,
        TileStyle::plain(),
        |p, r, c| {
            p.text(
                r.center(),
                egui::Align2::CENTER_CENTER,
                label,
                font(12.0),
                c,
            );
        },
    )
    .clicked()
}

/// Draws the question. `Some(true)` is "reset", `Some(false)` is "cancel"
/// (the button, Esc or a click on the backdrop), `None` keeps it open.
pub(crate) fn show(ctx: &egui::Context, scene: &Scene<'_>) -> Option<bool> {
    let t = scene.i18n;
    let width = (ctx.content_rect().width() - 48.0).clamp(280.0, 380.0);
    let mut answer = None;
    let modal = egui::Modal::new(egui::Id::new("reset-played"))
        .frame(egui::Frame::new().fill(theme::SURFACE).inner_margin(20.0))
        .backdrop_color(theme::NEUTRAL_900.gamma_multiply(0.5))
        .show(ctx, |ui| {
            ui.set_width(width);
            ui.spacing_mut().item_spacing = vec2(8.0, 12.0);
            ui.add(
                egui::Label::new(
                    RichText::new(t.tr("reset-played-question"))
                        .font(font(13.0))
                        .color(theme::TEXT),
                )
                .selectable(false)
                .wrap(),
            );
            ui.horizontal(|ui| {
                if button(ui, &t.tr("reset-played-cancel")) {
                    answer = Some(false);
                }
                if button(ui, &t.tr("reset-played-confirm")) {
                    answer = Some(true);
                }
            });
        });
    if answer.is_none() && modal.should_close() {
        answer = Some(false);
    }
    answer
}
```

`crates/fp-app/src/ui.rs`:

```diff
--- a/crates/fp-app/src/ui.rs
+++ b/crates/fp-app/src/ui.rs
@@ -14,6 +14,7 @@ pub mod icons;
 mod notice;
 mod player;
 pub mod playlist_files;
+mod reset_played;
 mod settings;
 pub mod shell;
 mod table;
```

`crates/fp-app/src/ui/app.rs`:

```diff
--- a/crates/fp-app/src/ui/app.rs
+++ b/crates/fp-app/src/ui/app.rs
@@ -28,6 +28,7 @@ use super::files::{AUDIO_EXTENSIONS, audio_paths};
 use super::notice;
 use super::player;
 use super::playlist_files::{self, FileOutcome};
+use super::reset_played;
 use super::settings::{self, SettingsDeps, SettingsState};
 use super::tag_editor;
 use super::theme;
@@ -108,6 +109,9 @@ pub(crate) struct ViewState {
     pub edit_tags: Option<TrackId>,
     /// The tag editor, while it is open.
     pub(crate) tag_editor: Option<tag_editor::TagEditor>,
+    /// The playlist whose Reset played waits for the operator's answer
+    /// (feedback 2 spec O22).
+    pub confirm_reset: Option<PlaylistId>,
     notice: Option<(String, f64)>,
 }
 
@@ -665,6 +669,22 @@ impl AppUi {
                 }
             }
         }
+        // O22: Reset played asks before it clears the marks of a playlist.
+        if let Some(playlist) = self.view.confirm_reset {
+            if state.playlists.get(playlist).is_none() {
+                // The playlist went away meanwhile: nothing left to confirm.
+                self.view.confirm_reset = None;
+            } else {
+                match reset_played::show(&ctx, &scene) {
+                    Some(true) => {
+                        scene.ctl.send(Command::ResetPlayed(playlist));
+                        self.view.confirm_reset = None;
+                    }
+                    Some(false) => self.view.confirm_reset = None,
+                    None => {}
+                }
+            }
+        }
         // O4: Restart now asks the close guard first when audio is on air.
         if std::mem::take(&mut self.view.restart_requested) {
             if fp_model::on_air(&state).is_empty() {
@@ -703,7 +723,7 @@ impl AppUi {
             }
         }
         // Files dropped under the tag editor are discarded, like shortcuts.
-        if take_drops && self.view.tag_editor.is_none() {
+        if take_drops && self.view.tag_editor.is_none() && self.view.confirm_reset.is_none() {
             self.file_drops(&ctx, &state);
         }
         let busy = state
@@ -930,6 +950,24 @@ impl AppUi {
         if self.view.tag_editor.is_some() {
             return;
         }
+        // So is the Reset played question: Esc cancels it, nothing else acts.
+        if self.view.confirm_reset.is_some() {
+            let escape = ctx.input(|i| {
+                i.events.iter().any(|e| {
+                    matches!(e, egui::Event::Key {
+                        key: Key::Escape,
+                        pressed: true,
+                        repeat: false,
+                        modifiers,
+                        ..
+                    } if modifiers.is_none())
+                })
+            });
+            if escape {
+                self.view.confirm_reset = None;
+            }
+            return;
+        }
         // Configured shortcuts whose key the toolkit knows.
         let bindings: Vec<(Key, &KeyChord, ShortcutAction)> = state
             .config
```

`crates/fp-app/src/ui/player.rs`:

```diff
--- a/crates/fp-app/src/ui/player.rs
+++ b/crates/fp-app/src/ui/player.rs
@@ -101,7 +101,7 @@ pub(crate) fn column(
             .max_rect(footer_rect)
             .layout(Layout::left_to_right(Align::Center)),
     );
-    footer(&mut footer_ui, scene, id, player.playlist);
+    footer(&mut footer_ui, scene, view_state, id, player.playlist);
 }
 
 /// Follows the player's current entry in its table (feedback spec F18):
@@ -1129,7 +1129,13 @@ fn tabs(
     ui.painter().rect_filled(bottom, 0.0, theme::NEUTRAL_800);
 }
 
-fn footer(ui: &mut Ui, scene: &Scene<'_>, id: PlayerId, playlist: PlaylistId) {
+fn footer(
+    ui: &mut Ui,
+    scene: &Scene<'_>,
+    view_state: &mut ViewState,
+    id: PlayerId,
+    playlist: PlaylistId,
+) {
     let t = scene.i18n;
     let rect = ui.max_rect();
     ui.painter().rect_filled(rect, 0.0, theme::NEUTRAL_900);
@@ -1172,6 +1178,34 @@ fn footer(ui: &mut Ui, scene: &Scene<'_>, id: PlayerId, playlist: PlaylistId) {
             .map_or(0, |p| p.entries.len());
         scene.pick_files(playlist, len);
     }
+    // O22: clear the played marks of this playlist, after a question.
+    let reset = t.tr("footer-reset-played");
+    let reset_width = ui
+        .painter()
+        .layout_no_wrap(reset.clone(), font(10.0), theme::NEUTRAL_300)
+        .size()
+        .x
+        + 26.0;
+    if widgets::tile(
+        ui,
+        vec2(reset_width, 18.0),
+        &t.tr("tip-reset-played"),
+        fp_model::can_reset_played(scene.state, playlist),
+        TileStyle::plain(),
+        |p, r, c| {
+            p.text(
+                r.center(),
+                egui::Align2::CENTER_CENTER,
+                format!("{} {reset}", icon::ARROW_COUNTER_CLOCKWISE),
+                font(10.0),
+                c,
+            );
+        },
+    )
+    .clicked()
+    {
+        view_state.confirm_reset = Some(playlist);
+    }
     let len = scene
         .state
         .playlists
```

`crates/fp-app/locales/en-US/main.ftl`:

```diff
--- a/crates/fp-app/locales/en-US/main.ftl
+++ b/crates/fp-app/locales/en-US/main.ftl
@@ -495,3 +495,9 @@ tags-cover-unsupported = it is not a JPEG or PNG image
 tags-cover-undecodable = it cannot be read as an image of a size the player accepts
 tags-error-cover-not-stored = this format cannot store a cover
 tags-error-cover = the cover is not a usable JPEG or PNG image
+
+footer-reset-played = Reset played
+tip-reset-played = Clear the played marks of this playlist
+reset-played-question = Clear the played mark of every track in this playlist?
+reset-played-cancel = Cancel
+reset-played-confirm = Reset played
```

`crates/fp-app/locales/es-ES/main.ftl`:

```diff
--- a/crates/fp-app/locales/es-ES/main.ftl
+++ b/crates/fp-app/locales/es-ES/main.ftl
@@ -495,3 +495,9 @@ tags-cover-unsupported = no es una imagen JPEG ni PNG
 tags-cover-undecodable = no se puede leer como una imagen de un tamaño que acepta el reproductor
 tags-error-cover-not-stored = este formato no puede guardar una carátula
 tags-error-cover = la carátula no es una imagen JPEG o PNG utilizable
+
+footer-reset-played = Reiniciar
+tip-reset-played = Quitar las marcas de reproducida de esta playlist
+reset-played-question = ¿Quitar la marca de reproducida a todas las pistas de esta playlist?
+reset-played-cancel = Cancelar
+reset-played-confirm = Reiniciar
```

- [ ] **Step 4: Run the tests to see them pass**

Run: `cargo test -p fp-app --test reset_played_ui` — expected: 9 passed. Run `cargo test -p fp-app --test i18n` — expected: pass (both locales define the same keys).

Check that `no_shortcut_acts_under_the_question` guards the gate: remove the `return;` of the new `confirm_reset` block in `AppUi::keyboard`, see the test fail, put it back.

- [ ] **Step 5: Gate and commit**

```bash
export PATH=$HOME/.cargo/bin:$PATH; cargo fmt --all --check && cargo clippy --workspace --all-targets -q -- -D warnings && cargo test --workspace -q && git add crates/fp-app && git commit -m "feat(ui): a Reset played button with a question (O22)"
```

### Task 4: The repeat and stop icons before the title (UI)

**Files:**
- Modify: `crates/fp-app/src/ui/table.rs` (the Title cell)
- Test: `crates/fp-app/tests/table_icons.rs` (create), `crates/fp-app/tests/main_screen.rs` (`flagged_entries_show_their_icons`)

**Interfaces:**
- Consumes: the existing `flag(ui, label, paint)` helper of `table.rs`.
- Produces: the Title cell lays out, left to right, the repeat icon, the stop-after icon, then the title; the outdated-analysis flag stays at the right.

- [ ] **Step 1: Write the failing tests**

`crates/fp-app/tests/table_icons.rs` (new file):

```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Feedback 2 spec O9: the per-entry repeat and stop icons are drawn before
//! the title.

mod support;

use egui::Rect;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::AppUi;
use fp_model::{AppState, Command, EntryId};
use support::{harness, harness_sized, state};

fn entries(s: &AppState) -> Vec<EntryId> {
    s.playlists
        .iter()
        .next()
        .unwrap()
        .entries
        .iter()
        .map(|x| x.id)
        .collect()
}

/// The title label of a table row (the one in the row of `icon`, when
/// given; the table's rows are below the header).
fn title_in_row(h: &Harness<'_, AppUi>, title: &str, row_y: f32) -> Rect {
    h.query_all_by_label(title)
        .map(|n| n.rect())
        .find(|r| (r.center().y - row_y).abs() < 4.0)
        .unwrap_or_else(|| panic!("no {title} label in the row at {row_y}"))
}

fn flagged(repeat: &[usize], stop: &[usize]) -> AppState {
    let mut s = state(1, 4);
    let e = entries(&s);
    for i in repeat {
        fp_model::apply(&mut s, Command::ToggleEntryRepeat(e[*i])).unwrap();
    }
    for i in stop {
        fp_model::apply(&mut s, Command::ToggleEntryStopAfter(e[*i])).unwrap();
    }
    s
}

#[test]
fn the_repeat_icon_comes_before_the_title_in_its_row() {
    let (h, _) = harness(flagged(&[1], &[]));
    let icon = h.get_by_label("Repeats").rect();
    let title = title_in_row(&h, "Song 2", icon.center().y);
    assert!(icon.right() <= title.left() + 0.5, "{icon:?} {title:?}");
}

#[test]
fn the_stop_icon_comes_before_the_title_in_its_row() {
    let (h, _) = harness(flagged(&[], &[2]));
    let icon = h.get_by_label("Stops after").rect();
    let title = title_in_row(&h, "Song 3", icon.center().y);
    assert!(icon.right() <= title.left() + 0.5, "{icon:?} {title:?}");
}

#[test]
fn both_icons_keep_their_order_before_the_title() {
    let (h, _) = harness(flagged(&[1], &[1]));
    let repeat = h.get_by_label("Repeats").rect();
    let stop = h.get_by_label("Stops after").rect();
    let title = title_in_row(&h, "Song 2", repeat.center().y);
    assert!(repeat.right() <= stop.left() + 0.5, "{repeat:?} {stop:?}");
    assert!(stop.right() <= title.left() + 0.5, "{stop:?} {title:?}");
}

#[test]
fn a_flagged_title_makes_room_and_an_unflagged_one_does_not() {
    let (h, _) = harness(flagged(&[1], &[]));
    let icon = h.get_by_label("Repeats").rect();
    let flagged_title = title_in_row(&h, "Song 2", icon.center().y);
    let plain: Vec<Rect> = ["Song 3", "Song 4"]
        .iter()
        .map(|t| {
            h.query_all_by_label(t)
                .map(|n| n.rect())
                .max_by(|a, b| a.top().total_cmp(&b.top()))
                .unwrap()
        })
        .collect();
    assert!((plain[0].left() - plain[1].left()).abs() < 0.5);
    assert!(
        flagged_title.left() > plain[0].left() + 10.0,
        "{flagged_title:?} {plain:?}"
    );
}

#[test]
fn the_icons_stay_in_a_narrow_table_and_the_title_gives_way() {
    let (h, _) = harness_sized(flagged(&[0], &[0]), egui::vec2(380.0, 700.0), |ui| ui);
    let repeat = h.get_by_label("Repeats").rect();
    let stop = h.get_by_label("Stops after").rect();
    assert!(
        repeat.left() >= 0.0 && stop.right() <= 380.0,
        "{repeat:?} {stop:?}"
    );
    assert!(repeat.right() <= stop.left() + 0.5);
}

#[test]
fn the_outdated_flag_stays_after_the_title() {
    let mut s = flagged(&[1], &[]);
    let track = s.playlists.entry(entries(&s)[1]).unwrap().track;
    let t = s.library.get_mut(track).unwrap();
    t.analyzed = true;
    t.analysis_version = 0;
    let (h, _) = harness(s);
    let outdated = h
        .get_by_label_contains("Analysed by an earlier version")
        .rect();
    let repeat = h.get_by_label("Repeats").rect();
    let title = title_in_row(&h, "Song 2", repeat.center().y);
    assert!(
        outdated.left() >= title.right() - 0.5,
        "{outdated:?} {title:?}"
    );
}
```

`crates/fp-app/tests/main_screen.rs`:

```diff
--- a/crates/fp-app/tests/main_screen.rs
+++ b/crates/fp-app/tests/main_screen.rs
@@ -706,9 +706,9 @@ fn flagged_entries_show_their_icons() {
     assert!(
         h.query_all_by_label("Song 1").any(|n| {
             let song = n.rect();
-            repeat.left() > song.left() && (repeat.center().y - song.center().y).abs() < 4.0
+            repeat.right() <= song.left() + 0.5 && (repeat.center().y - song.center().y).abs() < 4.0
         }),
-        "the icon sits in the row of its entry"
+        "the icon sits before the title, in the row of its entry"
     );
 }
 
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `cargo test -p fp-app --test table_icons` — expected: 4 of 6 fail (the icons are at x of about 560 and the titles at 67: the icon is after the title). Run: `cargo test -p fp-app --test main_screen flagged_entries_show_their_icons` — expected: FAIL.

- [ ] **Step 3: Implement**

`crates/fp-app/src/ui/table.rs`:

```diff
--- a/crates/fp-app/src/ui/table.rs
+++ b/crates/fp-app/src/ui/table.rs
@@ -263,25 +263,12 @@ pub(crate) fn track_table(
                 row.col(|ui| {
                     line(ui);
                     ui.add_space(8.0);
-                    // The entry's flags sit at the right of the title, in the
-                    // row's text colour (feedback spec §2.3).
+                    // The entry's repeat and stop icons sit before the title,
+                    // in the row's text colour (feedback 2 spec O9); the
+                    // "analysed by an earlier version" flag stays at the right.
                     ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                         ui.spacing_mut().item_spacing.x = 4.0;
                         ui.add_space(4.0);
-                        if entry.stop_after {
-                            flag(ui, &t.tr("flag-stop-after"), |p, r| {
-                                // Drawn into the flag's own 16x12 box, as before (paint would
-                                // centre the grid's 18x13 size instead).
-                                if let Some(d) = glyphs::drawn(TransportAction::StopAfter) {
-                                    p.extend((d.draw)(r, text));
-                                }
-                            });
-                        }
-                        if entry.repeat {
-                            flag(ui, &t.tr("flag-repeat"), |p, r| {
-                                widgets::glyph(p, r, icon::REPEAT, 13.0, text, false);
-                            });
-                        }
                         // Shown tracks are brought up to date anyway, so
                         // the flag only stays on the ones waiting.
                         if crate::services::outdated(track) {
@@ -291,6 +278,21 @@ pub(crate) fn track_table(
                             });
                         }
                         ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
+                            ui.spacing_mut().item_spacing.x = 4.0;
+                            if entry.repeat {
+                                flag(ui, &t.tr("flag-repeat"), |p, r| {
+                                    widgets::glyph(p, r, icon::REPEAT, 13.0, text, false);
+                                });
+                            }
+                            if entry.stop_after {
+                                flag(ui, &t.tr("flag-stop-after"), |p, r| {
+                                    // Drawn into the flag's own 16x12 box, as before (paint would
+                                    // centre the grid's 18x13 size instead).
+                                    if let Some(d) = glyphs::drawn(TransportAction::StopAfter) {
+                                        p.extend((d.draw)(r, text));
+                                    }
+                                });
+                            }
                             ui.add(
                                 egui::Label::new(
                                     RichText::new(&track.title)
```

- [ ] **Step 4: Run the tests to see them pass**

Run: `cargo test -p fp-app --test table_icons` — expected: 6 passed. Run: `cargo test -p fp-app --test main_screen` — expected: 46 passed.

- [ ] **Step 5: Gate and commit**

```bash
export PATH=$HOME/.cargo/bin:$PATH; cargo fmt --all --check && cargo clippy --workspace --all-targets -q -- -D warnings && cargo test --workspace -q && git add crates/fp-app && git commit -m "feat(ui): the repeat and stop icons come before the title (O9)"
```

### Task 5: Tables open scrolled to the next entry (UI)

**Files:**
- Modify: `crates/fp-app/src/ui/view.rs` (`start_scroll_target`), `crates/fp-app/src/ui/app.rs` (`FollowScroll`, `ViewState::startup_scrolled`), `crates/fp-app/src/ui/player.rs` (`scroll_to_next_once`), `crates/fp-app/src/ui/table.rs` (the follow block, `min_scrolled_height`)
- Test: `crates/fp-app/tests/table_start.rs` (create), `crates/fp-app/tests/view.rs`

**Interfaces:**
- Consumes: `view::row_status`-style access to `AppState`; the existing `ViewState::follow_scroll` consumed by `track_table`.
- Produces: `view::start_scroll_target(&AppState, PlayerId) -> Option<EntryId>` (the player's next, when it is in the playlist the tab shows); `app::FollowScroll { entry, align: egui::Align, animated: bool }` (`ViewState::follow_scroll` now maps a player to one); `ViewState::startup_scrolled: HashSet<PlayerId>`.

- [ ] **Step 1: Write the failing tests**

`crates/fp-app/tests/table_start.rs` (new file):

```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Feedback 2 spec O7: a table opens scrolled to its next entry.

mod support;

use std::sync::Arc;

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::AppUi;
use fp_model::{AppState, Command, EntryId, PlaylistId, apply};
use support::{harness, state};

const ROW_HEIGHT: f32 = 28.0;

fn entries(s: &AppState) -> Vec<EntryId> {
    s.playlists
        .iter()
        .next()
        .unwrap()
        .entries
        .iter()
        .map(|e| e.id)
        .collect()
}

fn with_next(tracks: usize, next: usize) -> AppState {
    let mut s = state(1, tracks);
    let p = s.players[0].id;
    let e = entries(&s)[next];
    apply(&mut s, Command::SetNext(p, e)).unwrap();
    s
}

/// The table's rows, as the vertical range between its header and the footer.
fn body(h: &Harness<'_, AppUi>) -> (f32, f32) {
    let top = h
        .query_all_by_label("TITLE")
        .map(|n| n.rect().bottom())
        .fold(f32::INFINITY, f32::min);
    let bottom = h
        .query_all_by_label("Add tracks to this playlist")
        .map(|n| n.rect().top())
        .fold(f32::INFINITY, f32::min);
    (top, bottom)
}

/// The row labelled `label` in the table, if it is on screen.
fn row_center(h: &Harness<'_, AppUi>, label: &str) -> Option<f32> {
    let (top, bottom) = body(h);
    h.query_all_by_label(label)
        .map(|n| n.rect())
        .find(|r| r.top() >= top && r.bottom() <= bottom)
        .map(|r| r.center().y)
}

#[test]
fn the_next_entry_is_visible_and_centred() {
    let (mut h, _) = harness(with_next(300, 150));
    h.run_steps(4);
    let (top, bottom) = body(&h);
    let y = row_center(&h, "Song 151").expect("the next row is on screen");
    let middle = (top + bottom) / 2.0;
    assert!(
        (y - middle).abs() <= ROW_HEIGHT,
        "row at {y}, middle of the table at {middle}"
    );
    assert!(row_center(&h, "Song 1").is_none());
}

#[test]
fn near_the_start_the_table_stays_at_the_top() {
    let (mut h, _) = harness(with_next(300, 2));
    h.run_steps(4);
    assert!(row_center(&h, "Song 1").is_some());
    assert!(row_center(&h, "Song 3").is_some());
}

#[test]
fn near_the_end_the_table_scrolls_as_far_as_it_can() {
    let (mut h, _) = harness(with_next(300, 298));
    h.run_steps(4);
    assert!(row_center(&h, "Song 299").is_some());
    assert!(
        row_center(&h, "Song 300").is_some(),
        "the last row is in view, not under the footer"
    );
}

#[test]
fn the_last_row_is_reachable_in_a_short_window() {
    // The table once kept a 200-point minimum height: in a short window
    // the footer covered its last rows, whatever the scroll.
    let (mut h, _) =
        support::harness_sized(with_next(300, 299), egui::vec2(1000.0, 600.0), |ui| ui);
    h.run_steps(4);
    assert!(row_center(&h, "Song 300").is_some());
}

#[test]
fn it_happens_once_a_later_next_does_not_scroll() {
    let (mut h, fake) = harness(with_next(300, 150));
    h.run_steps(4);
    let p = fake.player(0);
    let e = fake.entries();
    let mut s = (*fake.state.load_full()).clone();
    apply(&mut s, Command::SetNext(p, e[10])).unwrap();
    fake.state.store(Arc::new(s));
    h.run_steps(4);
    assert!(row_center(&h, "Song 11").is_none());
    assert!(
        row_center(&h, "Song 151").is_some(),
        "the view did not move"
    );
}

#[test]
fn a_short_list_needs_no_scrolling() {
    let (mut h, _) = harness(with_next(5, 3));
    h.run_steps(4);
    assert!(row_center(&h, "Song 1").is_some());
    assert!(row_center(&h, "Song 5").is_some());
}

#[test]
fn an_empty_list_is_fine() {
    let (mut h, _) = harness(state(1, 0));
    h.run_steps(4);
    assert!(h.query_by_label("TITLE").is_some());
}

#[test]
fn each_player_scrolls_to_its_own_next() {
    let mut s = state(2, 300);
    let e = entries(&s);
    let (p0, p1) = (s.players[0].id, s.players[1].id);
    apply(&mut s, Command::SetNext(p0, e[100])).unwrap();
    apply(&mut s, Command::SetNext(p1, e[250])).unwrap();
    let (mut h, _) = harness(s);
    h.run_steps(4);
    assert!(row_center(&h, "Song 101").is_some());
    assert!(row_center(&h, "Song 251").is_some());
}

#[test]
fn a_next_in_another_playlist_does_not_switch_the_tab_or_scroll() {
    let mut s = state(1, 300);
    let main: PlaylistId = s.playlists.first_id().unwrap();
    apply(
        &mut s,
        Command::CreatePlaylistFromPaths {
            name: "Other".into(),
            paths: (1..=50)
                .map(|n| std::path::PathBuf::from(format!("/music/Other {n}.mp3")))
                .collect(),
        },
    )
    .unwrap();
    let other = s.playlists.iter().nth(1).unwrap().entries[40].id;
    let p = s.players[0].id;
    apply(&mut s, Command::SetNext(p, other)).unwrap();
    let (mut h, fake) = harness(s);
    h.run_steps(4);
    assert!(
        !fake
            .take_sent()
            .iter()
            .any(|c| matches!(c, Command::ShowPlaylist(..)))
    );
    assert_eq!(fake.state.load().player(p).unwrap().playlist, main);
    assert!(
        row_center(&h, "Song 1").is_some(),
        "the table stays at the top"
    );
}
```

`crates/fp-app/tests/view.rs`:

```diff
--- a/crates/fp-app/tests/view.rs
+++ b/crates/fp-app/tests/view.rs
@@ -13,7 +13,7 @@ use fp_app::ui::format::{clock, countdown, number_width};
 use fp_app::ui::view::{
     PlayerStatus, RowStatus, TipField, cue_follow_target, cue_window_view, fader_from_gain,
     file_icon, file_problem, gain_from_fader, player_view, playlist_times, row_status, shown_entry,
-    tag_edit_availability, track_tooltip, volume_db,
+    start_scroll_target, tag_edit_availability, track_tooltip, volume_db,
 };
 use fp_model::{
     AppState, AudioFormat, Command, Config, EntryId, FileState, MarkerKind, PlayerId, Track, apply,
@@ -612,3 +612,33 @@ fn tag_edit_availability_combines_the_rule_and_the_extension() {
         Some(fp_model::TagEditBlock::OnAir)
     );
 }
+
+#[test]
+fn the_start_scroll_goes_to_the_next_entry_of_the_shown_playlist() {
+    let (mut s, e, p) = state(5);
+    assert_eq!(start_scroll_target(&s, p), Some(e[0]), "the derived next");
+    apply(&mut s, Command::SetNext(p, e[3])).unwrap();
+    assert_eq!(start_scroll_target(&s, p), Some(e[3]));
+}
+
+#[test]
+fn there_is_no_start_scroll_without_a_next_or_for_another_playlist() {
+    let (mut s, _, p) = state(0);
+    assert_eq!(start_scroll_target(&s, p), None, "an empty playlist");
+    apply(
+        &mut s,
+        Command::CreatePlaylistFromPaths {
+            name: "Other".into(),
+            paths: vec![PathBuf::from("/m/other.flac")],
+        },
+    )
+    .unwrap();
+    let other = s.playlists.iter().nth(1).unwrap().entries[0].id;
+    apply(&mut s, Command::SetNext(p, other)).unwrap();
+    assert_eq!(
+        start_scroll_target(&s, p),
+        None,
+        "next is in another playlist"
+    );
+    assert_eq!(start_scroll_target(&s, PlayerId(999_999)), None);
+}
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `cargo test -p fp-app --test view start_scroll` — expected: does not compile (`start_scroll_target` does not exist). Run: `cargo test -p fp-app --test table_start` — expected: 5 of the 9 fail: `the_next_entry_is_visible_and_centred`, `each_player_scrolls_to_its_own_next` and `it_happens_once_a_later_next_does_not_scroll` (nothing scrolls yet), and `near_the_end_the_table_scrolls_as_far_as_it_can` and `the_last_row_is_reachable_in_a_short_window` (the table's default minimum body height of 200 points puts the last rows under the footer).

- [ ] **Step 3: Implement**

`crates/fp-app/src/ui/view.rs`:

```diff
--- a/crates/fp-app/src/ui/view.rs
+++ b/crates/fp-app/src/ui/view.rs
@@ -288,6 +288,16 @@ pub fn row_status(state: &AppState, player: PlayerId, entry: &PlaylistEntry) ->
     }
 }
 
+/// O7: the entry a player's table scrolls to when the application starts:
+/// its next entry, when that is in the playlist the table shows. A next in
+/// another playlist is left alone (the tab is not switched for it).
+pub fn start_scroll_target(state: &AppState, player: PlayerId) -> Option<EntryId> {
+    let p = state.player(player).ok()?;
+    let next = p.next?;
+    let (playlist, _) = state.playlists.find(next)?;
+    (playlist == p.playlist).then_some(next)
+}
+
 #[derive(Debug, Clone, Copy, PartialEq)]
 pub struct PlaylistTimes {
     pub total: f64,
```

`crates/fp-app/src/ui/app.rs`:

```diff
--- a/crates/fp-app/src/ui/app.rs
+++ b/crates/fp-app/src/ui/app.rs
@@ -63,6 +63,17 @@ pub(crate) struct Picked {
     paths: Vec<PathBuf>,
 }
 
+/// A row a player's table scrolls to once its playlist is shown.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub(crate) struct FollowScroll {
+    pub entry: EntryId,
+    /// Where the row ends up: the top (following the current entry) or the
+    /// middle (the next entry at start-up).
+    pub align: Align,
+    /// Whether the table glides to the row; start-up jumps.
+    pub animated: bool,
+}
+
 /// Everything the UI remembers between frames.
 #[derive(Default)]
 pub(crate) struct ViewState {
@@ -93,8 +104,11 @@ pub(crate) struct ViewState {
     /// A current entry the table will follow once the operator's grace has
     /// passed (feedback spec F18).
     pub follow_pending: HashMap<PlayerId, EntryId>,
-    /// A row the table scrolls to the top once its playlist is shown.
-    pub follow_scroll: HashMap<PlayerId, EntryId>,
+    /// A row the table scrolls to once its playlist is shown.
+    pub follow_scroll: HashMap<PlayerId, FollowScroll>,
+    /// The players whose table already had its start-up scroll to the next
+    /// entry (feedback 2 spec O7); it happens once, on their first frame.
+    pub startup_scrolled: HashSet<PlayerId>,
     /// The table width and column fractions each player's table was last
     /// laid out with (a change resets egui's column widths).
     pub table_layout: HashMap<PlayerId, (f32, Option<[f32; 4]>)>,
```

`crates/fp-app/src/ui/player.rs`:

```diff
--- a/crates/fp-app/src/ui/player.rs
+++ b/crates/fp-app/src/ui/player.rs
@@ -10,7 +10,7 @@ use egui::{
 use egui_phosphor::regular as icon;
 use fp_model::{Command, MarkerKind, PlayMode, PlayerId, PlaylistId, TrackId};
 
-use super::app::{DragEntry, Scene, ViewState};
+use super::app::{DragEntry, FollowScroll, Scene, ViewState};
 use super::format;
 use super::glyphs::{self, TransportAction};
 use super::table;
@@ -82,6 +82,7 @@ pub(crate) fn column(
             wave(ui, scene, view_state, id, &pv);
             time_row(ui, &pv);
         });
+    scroll_to_next_once(scene, view_state, id);
     follow_current(scene, view_state, id, player.playlist);
     tabs(ui, scene, view_state, id, player.playlist);
     let footer_top = ui.max_rect().bottom() - FOOTER_HEIGHT;
@@ -143,7 +144,34 @@ fn follow_current(scene: &Scene<'_>, view_state: &mut ViewState, id: PlayerId, s
     if playlist != shown {
         scene.ctl.send(Command::ShowPlaylist(id, playlist));
     }
-    view_state.follow_scroll.insert(id, entry);
+    view_state.follow_scroll.insert(
+        id,
+        FollowScroll {
+            entry,
+            align: Align::TOP,
+            animated: true,
+        },
+    );
+}
+
+/// O7: the first time a player's column is drawn (the start of the
+/// application, or a player added later), its table scrolls so that the
+/// next entry is in the middle. Never again: after that the operator, and
+/// the following of the current entry, own the scroll position.
+fn scroll_to_next_once(scene: &Scene<'_>, view_state: &mut ViewState, id: PlayerId) {
+    if !view_state.startup_scrolled.insert(id) {
+        return;
+    }
+    if let Some(entry) = view::start_scroll_target(scene.state, id) {
+        view_state.follow_scroll.insert(
+            id,
+            FollowScroll {
+                entry,
+                align: Align::Center,
+                animated: false,
+            },
+        );
+    }
 }
 
 fn small_caps(text: &str, color: Color32) -> RichText {
```

`crates/fp-app/src/ui/table.rs`:

```diff
--- a/crates/fp-app/src/ui/table.rs
+++ b/crates/fp-app/src/ui/table.rs
@@ -6,7 +6,7 @@ use egui_extras::{Column, TableBuilder};
 use egui_phosphor::regular as icon;
 use fp_model::{ColumnWidths, Command, EntryId, PlayerId, PlaylistId, Transport};
 
-use super::app::{DragEntry, DropTarget, Scene, ViewState};
+use super::app::{DragEntry, DropTarget, FollowScroll, Scene, ViewState};
 use super::format;
 use super::glyphs::{self, TransportAction};
 use super::theme;
@@ -89,10 +89,17 @@ pub(crate) fn track_table(
         builder.reset();
     }
     // A current entry being followed: scroll its row to the top once.
-    if let Some(entry) = view_state.follow_scroll.get(&player).copied() {
+    if let Some(FollowScroll {
+        entry,
+        align,
+        animated,
+    }) = view_state.follow_scroll.get(&player).copied()
+    {
         match entries.iter().position(|e| e.id == entry) {
             Some(i) => {
-                builder = builder.scroll_to_row(i, Some(Align::TOP));
+                builder = builder
+                    .scroll_to_row(i, Some(align))
+                    .animate_scrolling(animated);
                 view_state.follow_scroll.remove(&player);
             }
             // Not in this playlist: wait for its tab, unless it is gone.
@@ -107,6 +114,9 @@ pub(crate) fn track_table(
         .striped(false)
         .resizable(true)
         .vscroll(true)
+        // The table's default minimum body is 200 points: in a short window
+        // the footer would cover the last rows, out of reach of the scroll.
+        .min_scrolled_height(0.0)
         .auto_shrink([false, false])
         .sense(Sense::click_and_drag())
         .cell_layout(Layout::left_to_right(Align::Center))
```

- [ ] **Step 4: Run the tests to see them pass**

Run: `cargo test -p fp-app --test view` — expected: pass. Run: `cargo test -p fp-app --test table_start` — expected: 9 passed. Run: `cargo test -p fp-app --test table_follow` — expected: pass (following the current entry is unchanged).

Check that the tests discriminate: set `.min_scrolled_height(200.0)` and see two tests fail; set `align: Align::TOP` in `scroll_to_next_once` and see `the_next_entry_is_visible_and_centred` and `near_the_start_the_table_stays_at_the_top` fail; put both back.

- [ ] **Step 5: Gate and commit**

```bash
export PATH=$HOME/.cargo/bin:$PATH; cargo fmt --all --check && cargo clippy --workspace --all-targets -q -- -D warnings && cargo test --workspace -q && git add crates/fp-app && git commit -m "feat(ui): tables open scrolled to the next entry and keep their last rows reachable (O7)"
```

### Task 6: Widths keyed by column, the configured columns and live resizing (model and UI)

This is the largest task. Its steps keep the order: model (widths, Intro), pure layout, then the table. The crate does not compile between step 3 and step 5, because `ColumnWidths` changes shape; do not commit before step 6.

**Files:**
- Create: `crates/fp-app/src/ui/table_layout.rs`
- Modify: `crates/fp-model/src/player.rs`, `session.rs`, `track.rs`; `crates/fp-app/src/ui.rs`, `ui/app.rs`, `ui/table.rs` (rewritten above `context_menu`), `ui/view.rs`; both `main.ftl`
- Test: `crates/fp-model/tests/column_widths.rs`, `intro_column.rs` (create); `tests/editing.rs`, `tests/session.rs`; `crates/fp-app/tests/table_layout.rs`, `table_columns_ui.rs` (create); `tests/main_screen.rs`, `tests/view.rs`

**Interfaces:**
- Consumes: `TableColumn`, `normalize_columns` (Task 1); `config.ui.table_columns`; `Command::SetColumnWidths(PlayerId, ColumnWidths)` (existing).
- Produces:
  - `ColumnWidths { fractions: Option<BTreeMap<TableColumn, f32>> }` (no longer `Copy`), `ColumnWidths::keyed(impl IntoIterator<Item = (TableColumn, f32)>) -> Self` (normalised), `fraction(&self, TableColumn) -> Option<f32>`, `normalized(self) -> Self`; serde: the new form is an object keyed by `TableColumn::name()`, the old four-number array converts to `#`, Title, Artist, Duration.
  - `Track::intro_secs(&self, use_markers: bool) -> Option<f64>`.
  - `table_layout::{column_min(TableColumn, number_digits: usize) -> f32, fit(&[f32], &[f32], f32) -> Vec<f32>, column_px(&[TableColumn], &ColumnWidths, width: f32, number_digits: usize) -> Vec<f32>, resize_px(start: &[f32], mins: &[f32], edge: usize, new_width: f32) -> Vec<f32>, fractions_of(&[TableColumn], &[f32]) -> ColumnWidths}`.
  - `view::cell_text(&Track, TableColumn, use_markers: bool) -> String`.
  - `AppUi::column_widths(PlayerId) -> Option<Vec<f32>>` (the pixel widths of the last frame, in the order of `ui.table_columns`; was `[f32; 4]`).
  - Messages `col-album`, `col-date`, `col-genre`, `col-intro`, `col-file_name`.

- [ ] **Step 1: Write the failing tests**

Model:

`crates/fp-model/tests/column_widths.rs` (new file):

```rust
#![allow(clippy::unwrap_used, clippy::float_cmp)]
//! Feedback 2 spec O24: column widths are fractions keyed by column, and the
//! old four-number form is converted when it maps cleanly.

use fp_model::TableColumn::{Album, Artist, Duration, Number, Title};
use fp_model::{ColumnWidths, PlayerSession, TableColumn};

fn widths(json: &str) -> ColumnWidths {
    serde_json::from_str(json).unwrap()
}

#[test]
fn keyed_widths_round_trip_with_stable_names() {
    let w = ColumnWidths::keyed([(Title, 3.0), (Artist, 1.0)]);
    let json = serde_json::to_value(&w).unwrap();
    assert_eq!(json["fractions"]["title"], 0.75);
    assert_eq!(json["fractions"]["artist"], 0.25);
    let back: ColumnWidths = serde_json::from_value(json).unwrap();
    assert_eq!(back, w);
}

#[test]
fn the_old_four_fractions_become_the_four_default_columns() {
    let w = widths(r#"{"fractions":[0.05,0.5,0.35,0.1]}"#);
    assert_eq!(w.fraction(Number), Some(0.05));
    assert_eq!(w.fraction(Title), Some(0.5));
    assert_eq!(w.fraction(Artist), Some(0.35));
    assert_eq!(w.fraction(Duration), Some(0.1));
    assert_eq!(w.fraction(Album), None);
}

#[test]
fn an_old_array_that_does_not_map_cleanly_gives_the_default_layout() {
    for json in [
        r#"{"fractions":[0.5,0.5]}"#,
        r#"{"fractions":[0.1,0.2,0.3,0.4,0.5]}"#,
        r#"{"fractions":[]}"#,
        r#"{"fractions":["a","b","c","d"]}"#,
    ] {
        assert_eq!(widths(json).normalized().fractions, None, "{json}");
    }
}

#[test]
fn an_old_array_with_broken_numbers_is_dropped_when_normalised() {
    let w = widths(r#"{"fractions":[-1,0.5,0.35,0.1]}"#);
    assert_eq!(w.normalized().fractions, None);
    let zeros = widths(r#"{"fractions":[0,0,0,0]}"#);
    assert_eq!(zeros.normalized().fractions, None);
}

#[test]
fn an_old_array_is_scaled_to_sum_to_one() {
    let w = widths(r#"{"fractions":[1,3,2,2]}"#).normalized();
    assert!((w.fraction(Title).unwrap() - 0.375).abs() < 1e-6);
}

#[test]
fn unknown_columns_are_dropped_and_known_ones_kept() {
    let w = widths(r#"{"fractions":{"title":2.0,"bpm":9.0,"artist":2.0}}"#).normalized();
    assert_eq!(w.fraction(Title), Some(0.5));
    assert_eq!(w.fraction(Artist), Some(0.5));
    assert_eq!(w.fractions.unwrap().len(), 2);
}

#[test]
fn only_unknown_columns_give_the_default_layout() {
    let w = widths(r#"{"fractions":{"bpm":1.0}}"#).normalized();
    assert_eq!(w.fractions, None);
}

#[test]
fn a_value_of_the_wrong_type_gives_the_default_layout() {
    for json in [
        r#"{"fractions":"wide"}"#,
        r#"{"fractions":7}"#,
        r#"{"fractions":{"title":"a"}}"#,
        r#"{"fractions":null}"#,
        r#"{}"#,
    ] {
        assert_eq!(widths(json).normalized().fractions, None, "{json}");
    }
}

#[test]
fn a_session_with_the_old_widths_loads_them_keyed() {
    let session: PlayerSession = serde_json::from_value(serde_json::json!({
        "id": 1,
        "playlist": 1,
        "columns": {"fractions": [0.1, 0.5, 0.3, 0.1]}
    }))
    .unwrap();
    assert_eq!(session.columns.fraction(Artist), Some(0.3));
}

#[test]
fn normalising_twice_changes_nothing() {
    let once = ColumnWidths::keyed([(Title, 3.0), (Artist, 2.0), (Duration, 1.0)]);
    assert_eq!(once.clone().normalized(), once);
    assert_eq!(
        TableColumn::from_name("file_name"),
        Some(TableColumn::FileName)
    );
    assert_eq!(TableColumn::from_name("file-name"), None);
}
```

`crates/fp-model/tests/intro_column.rs` (new file):

```rust
#![allow(clippy::unwrap_used)]
//! Feedback 2 spec O24: the Intro column shows how long the intro lasts.

use std::path::PathBuf;

use fp_model::{MarkerKind, Track, TrackId};

fn track(cue_in: Option<f64>, intro_end: Option<f64>) -> Track {
    let mut t = Track::new(TrackId(1), PathBuf::from("/m/a.flac"));
    t.duration_secs = 200.0;
    t.markers.set_manual(MarkerKind::CueIn, cue_in);
    t.markers.set_manual(MarkerKind::IntroEnd, intro_end);
    t
}

#[test]
fn the_intro_runs_from_the_start_to_its_marker() {
    assert_eq!(track(None, Some(12.0)).intro_secs(true), Some(12.0));
}

#[test]
fn with_cue_markers_on_it_runs_from_the_cue_in() {
    let t = track(Some(2.0), Some(12.0));
    assert_eq!(t.intro_secs(true), Some(10.0));
    assert_eq!(t.intro_secs(false), Some(12.0), "markers off: from 0");
}

#[test]
fn without_an_intro_marker_there_is_none() {
    assert_eq!(track(None, None).intro_secs(true), None);
}

#[test]
fn a_marker_not_after_the_start_gives_none() {
    assert_eq!(track(Some(5.0), Some(5.0)).intro_secs(true), None);
    assert_eq!(track(Some(8.0), Some(5.0)).intro_secs(true), None);
}
```

`crates/fp-model/tests/editing.rs`:

```diff
--- a/crates/fp-model/tests/editing.rs
+++ b/crates/fp-model/tests/editing.rs
@@ -5,8 +5,8 @@ mod common;
 
 use common::{entries, fixture, p0};
 use fp_model::{
-    ColumnWidths, Command, Config, EngineAction, EngineEvent, ModelError, PlaylistId, Transport,
-    apply, on_event,
+    ColumnWidths, Command, Config, EngineAction, EngineEvent, ModelError, PlaylistId, TableColumn,
+    Transport, apply, on_event,
 };
 
 #[test]
@@ -286,10 +286,13 @@ fn update_config_keeps_the_player_count() {
 fn column_widths_are_stored_per_player() {
     let mut state = fixture(1);
     let p = p0(&state);
-    let widths = ColumnWidths {
-        fractions: Some([0.1, 0.5, 0.3, 0.1]),
-    };
-    apply(&mut state, Command::SetColumnWidths(p, widths)).unwrap();
+    let widths = ColumnWidths::keyed([
+        (TableColumn::Number, 0.1),
+        (TableColumn::Title, 0.5),
+        (TableColumn::Artist, 0.3),
+        (TableColumn::Duration, 0.1),
+    ]);
+    apply(&mut state, Command::SetColumnWidths(p, widths.clone())).unwrap();
     assert_eq!(state.player(p).unwrap().columns, widths);
     assert_eq!(state.players[1].columns, ColumnWidths::default());
     assert_eq!(state.player(p).unwrap().transport, Transport::Stopped);
```

`crates/fp-model/tests/session.rs`:

```diff
--- a/crates/fp-model/tests/session.rs
+++ b/crates/fp-model/tests/session.rs
@@ -216,20 +216,26 @@ fn a_broken_history_loads_as_empty() {
 
 #[test]
 fn column_fractions_are_normalised_and_old_pixel_widths_ignored() {
-    use fp_model::ColumnWidths;
-    let c = ColumnWidths {
-        fractions: Some([1.0, 3.0, 2.0, 2.0]),
-    }
-    .normalized();
+    use fp_model::{ColumnWidths, TableColumn};
+    let c = ColumnWidths::keyed([
+        (TableColumn::Number, 1.0),
+        (TableColumn::Title, 3.0),
+        (TableColumn::Artist, 2.0),
+        (TableColumn::Duration, 2.0),
+    ]);
     let f = c.fractions.unwrap();
-    assert!((f.iter().sum::<f32>() - 1.0).abs() < 1e-6);
-    assert!((f[1] - 0.375).abs() < 1e-6);
-    for broken in [[f32::NAN, 1.0, 1.0, 1.0], [-1.0, 1.0, 1.0, 1.0], [0.0; 4]] {
+    assert!((f.values().sum::<f32>() - 1.0).abs() < 1e-6);
+    assert!((f[&TableColumn::Title] - 0.375).abs() < 1e-6);
+    for broken in [f32::NAN, -1.0] {
         let c = ColumnWidths {
-            fractions: Some(broken),
+            fractions: Some([(TableColumn::Title, broken), (TableColumn::Artist, 1.0)].into()),
         };
         assert_eq!(c.normalized().fractions, None, "{broken:?}");
     }
+    let zero = ColumnWidths {
+        fractions: Some([(TableColumn::Title, 0.0)].into()),
+    };
+    assert_eq!(zero.normalized().fractions, None);
     let old: ColumnWidths =
         serde_json::from_str(r#"{"number":40.0,"title":300.0,"duration":52.0}"#).unwrap();
     assert_eq!(old, ColumnWidths::default());
```

Pure layout:

`crates/fp-app/tests/table_layout.rs` (new file):

```rust
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
//! The widths of the track table's columns (feedback 2 spec O16 and O24).

use fp_app::ui::table_layout::{column_min, column_px, fit, fractions_of, resize_px};
use fp_model::TableColumn::{Album, Artist, Date, Duration, FileName, Genre, Intro, Number, Title};
use fp_model::{ColumnWidths, TableColumn, default_columns};

fn sum(px: &[f32]) -> f32 {
    px.iter().sum()
}

fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.01
}

fn mins(columns: &[TableColumn]) -> Vec<f32> {
    columns.iter().map(|c| column_min(*c, 2)).collect()
}

// --- column_px

#[test]
fn the_default_layout_is_what_the_table_always_had() {
    let px = column_px(&default_columns(), &ColumnWidths::default(), 1000.0, 2);
    let number_min = column_min(Number, 2);
    assert!(near(px[0], number_min), "{px:?}");
    assert!(near(px[3], column_min(Duration, 2)));
    let rest = 1000.0 - number_min - column_min(Duration, 2);
    assert!(near(px[1], rest * 0.6) && near(px[2], rest * 0.4), "{px:?}");
    assert!(near(sum(&px), 1000.0));
}

#[test]
fn narrow_columns_stay_at_their_minimum_and_text_columns_share_the_rest() {
    let columns = [
        Number, Title, Artist, Album, Date, Genre, Duration, Intro, FileName,
    ];
    let px = column_px(&columns, &ColumnWidths::default(), 1600.0, 3);
    assert!(near(px[0], column_min(Number, 3)));
    assert!(
        near(px[4], 84.0) && near(px[6], 52.0) && near(px[7], 52.0),
        "{px:?}"
    );
    // Title 3 : Artist 2 : Album 2 : Genre 1.5 : File name 2.
    assert!(
        near(px[1] / px[2], 1.5) && near(px[2], px[3]) && near(px[8], px[2]),
        "{px:?}"
    );
    assert!(near(px[5] / px[2], 0.75));
    assert!(near(sum(&px), 1600.0));
}

#[test]
fn stored_fractions_scale_with_the_width() {
    let w = ColumnWidths::keyed([
        (Number, 0.05),
        (Title, 0.5),
        (Artist, 0.35),
        (Duration, 0.1),
    ]);
    let a = column_px(&default_columns(), &w, 1000.0, 2);
    let b = column_px(&default_columns(), &w, 1500.0, 2);
    assert!(near(sum(&b), 1500.0));
    assert!(
        near(b[1] / a[1], 1.5) && near(b[2] / a[2], 1.5),
        "{a:?} {b:?}"
    );
}

#[test]
fn a_shown_column_without_a_stored_width_takes_its_default_and_the_rest_keep_their_shares() {
    let w = ColumnWidths::keyed([
        (Number, 0.05),
        (Title, 0.5),
        (Artist, 0.35),
        (Duration, 0.1),
    ]);
    let columns = [Number, Title, Artist, Date, Duration];
    let px = column_px(&columns, &w, 1000.0, 2);
    assert!(near(px[3], 84.0), "Date at its default: {px:?}");
    assert!(near(sum(&px), 1000.0));
    // The stored ones keep their proportions: Title:Artist is still 0.5:0.35.
    assert!(near(px[1] / px[2], 0.5 / 0.35), "{px:?}");
}

#[test]
fn a_hidden_column_gives_its_room_to_the_others_in_proportion() {
    let w = ColumnWidths::keyed([
        (Number, 0.05),
        (Title, 0.5),
        (Artist, 0.35),
        (Duration, 0.1),
    ]);
    let columns = [Title, Duration];
    let px = column_px(&columns, &w, 1000.0, 2);
    assert!(near(sum(&px), 1000.0));
    assert!(near(px[0] / px[1], 5.0), "0.5 : 0.1 -> {px:?}");
}

#[test]
fn a_reordered_list_keeps_each_columns_width() {
    let w = ColumnWidths::keyed([(Title, 0.5), (Artist, 0.3), (Duration, 0.2)]);
    let a = column_px(&[Title, Artist, Duration], &w, 1000.0, 2);
    let b = column_px(&[Duration, Artist, Title], &w, 1000.0, 2);
    assert!(
        near(a[0], b[2]) && near(a[1], b[1]) && near(a[2], b[0]),
        "{a:?} {b:?}"
    );
}

#[test]
fn the_minimums_win_in_a_narrow_table() {
    let w = ColumnWidths::keyed([
        (Number, 0.01),
        (Title, 0.6),
        (Artist, 0.38),
        (Duration, 0.01),
    ]);
    let px = column_px(&default_columns(), &w, 360.0, 2);
    assert!(
        px[0] >= column_min(Number, 2) - 0.01 && px[3] >= 52.0 - 0.01,
        "{px:?}"
    );
    assert!(near(sum(&px), 360.0));
    let tiny = column_px(&default_columns(), &ColumnWidths::default(), 50.0, 2);
    assert!(tiny.iter().all(|w| *w >= 0.0 && w.is_finite()), "{tiny:?}");
    assert!(near(sum(&tiny), 50.0));
}

#[test]
fn nonsense_widths_never_give_nonsense_pixels() {
    for width in [f32::NAN, f32::INFINITY, -5.0, 0.0] {
        let px = column_px(&default_columns(), &ColumnWidths::default(), width, 2);
        assert!(
            px.iter().all(|w| w.is_finite() && *w >= 0.0),
            "{width}: {px:?}"
        );
    }
    let broken = ColumnWidths {
        fractions: Some([(Title, f32::NAN)].into()),
    };
    let px = column_px(&default_columns(), &broken, 800.0, 2);
    assert!(near(sum(&px), 800.0), "{px:?}");
}

// --- fit

#[test]
fn fit_pins_what_is_too_narrow_and_shares_the_rest() {
    let px = fit(&[10.0, 300.0, 300.0], &[50.0, 60.0, 60.0], 600.0);
    assert!(
        near(px[0], 50.0) && near(px[1], 275.0) && near(px[2], 275.0),
        "{px:?}"
    );
}

#[test]
fn fit_scales_the_minimums_when_nothing_fits() {
    let px = fit(&[1.0, 1.0], &[60.0, 60.0], 60.0);
    assert!(near(px[0], 30.0) && near(px[1], 30.0), "{px:?}");
    assert!(fit(&[], &[], 100.0).is_empty());
}

// --- resize_px

#[test]
fn dragging_an_edge_moves_all_the_columns_on_its_right() {
    let columns = [Number, Title, Artist, Album, Duration];
    let m = mins(&columns);
    let start = [44.0, 300.0, 200.0, 200.0, 100.0];
    let out = resize_px(&start, &m, 1, 400.0);
    assert!(near(out[0], 44.0), "left of the edge: unchanged");
    assert!(near(out[1], 400.0));
    assert!(near(sum(&out), sum(&start)));
    // Artist, Album and Duration share what is left (400) as 200:200:100.
    assert!(
        near(out[2], 160.0) && near(out[3], 160.0) && near(out[4], 80.0),
        "{out:?}"
    );
}

#[test]
fn every_step_of_a_drag_keeps_the_total_and_the_minimums() {
    let columns = [Number, Title, Artist, Album, Duration];
    let m = mins(&columns);
    let start = [44.0, 300.0, 200.0, 200.0, 100.0];
    for i in 0..=100 {
        let wanted = -50.0 + i as f32 * 12.0;
        for edge in 0..5 {
            let out = resize_px(&start, &m, edge, wanted);
            assert!(
                near(sum(&out), sum(&start)),
                "edge {edge} wanted {wanted}: {out:?}"
            );
            for (w, min) in out.iter().zip(&m) {
                assert!(
                    *w >= min - 0.01 && w.is_finite(),
                    "edge {edge} wanted {wanted}: {out:?}"
                );
            }
        }
    }
}

#[test]
fn an_edge_cannot_be_pushed_past_the_minimums_of_the_columns_after_it() {
    let columns = [Title, Artist, Duration];
    let m = mins(&columns);
    let start = [400.0, 300.0, 100.0];
    let out = resize_px(&start, &m, 0, 10_000.0);
    assert!(near(out[1], m[1]) && near(out[2], m[2]), "{out:?}");
    assert!(near(sum(&out), 800.0));
}

#[test]
fn an_edge_cannot_shrink_a_column_below_its_minimum() {
    let columns = [Title, Artist, Duration];
    let m = mins(&columns);
    let out = resize_px(&[400.0, 300.0, 100.0], &m, 0, -50.0);
    assert!(near(out[0], m[0]), "{out:?}");
}

#[test]
fn the_last_column_has_no_edge_and_bad_input_changes_nothing() {
    let start = [300.0, 300.0, 100.0];
    let m = mins(&[Title, Artist, Duration]);
    assert_eq!(resize_px(&start, &m, 2, 500.0), start);
    assert_eq!(resize_px(&start, &m, 9, 500.0), start);
    assert_eq!(resize_px(&start, &m, 0, f32::NAN), start);
    assert!(resize_px(&[], &[], 0, 5.0).is_empty());
}

#[test]
fn a_drag_back_to_where_it_began_restores_the_widths() {
    let m = mins(&[Title, Artist, Album, Duration]);
    let start = [300.0, 200.0, 200.0, 100.0];
    let out = resize_px(&start, &m, 1, 200.0);
    for (a, b) in out.iter().zip(&start) {
        assert!(near(*a, *b), "{out:?}");
    }
}

// --- fractions_of

#[test]
fn the_stored_widths_are_each_columns_share() {
    let columns = [Title, Genre, Duration];
    let w = fractions_of(&columns, &[600.0, 300.0, 100.0]);
    assert!(near(w.fraction(Title).unwrap(), 0.6));
    assert!(near(w.fraction(Genre).unwrap(), 0.3));
    assert!(near(w.fraction(Duration).unwrap(), 0.1));
    assert_eq!(w.fraction(Artist), None);
}
```

Table (these also pin the Review Focus lines 1 to 3):

`crates/fp-app/tests/table_columns_ui.rs` (new file):

```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Feedback 2 spec O16 and O24: the table draws the configured columns and
//! resizes them live.

mod support;

use egui::{Event, Modifiers, PointerButton, Pos2, pos2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::AppUi;
use fp_app::ui::table_layout::column_min;
use fp_model::TableColumn::{Album, Artist, Date, Duration, FileName, Genre, Intro, Number, Title};
use fp_model::{AppState, Command, MarkerKind, TableColumn};
use support::{Fake, harness, harness_sized, state};

fn with_columns(mut s: AppState, columns: &[TableColumn]) -> AppState {
    s.config.ui.table_columns = columns.to_vec();
    s
}

fn tagged(mut s: AppState) -> AppState {
    for (i, t) in s.library.iter_mut().enumerate() {
        t.duration_secs = 200.0;
        t.album = format!("Album {}", i + 1);
        t.date = Some(format!("{}-05-14", 2000 + i));
        t.genre = format!("Genre {}", i + 1);
        t.markers
            .set_manual(MarkerKind::IntroEnd, Some(12.0 + i as f64));
    }
    s
}

fn header_left(h: &Harness<'_, AppUi>, label: &str) -> f32 {
    h.get_by_label(label).rect().left()
}

#[test]
fn the_default_columns_are_number_title_artist_and_duration() {
    let (h, _) = harness(state(1, 3));
    for label in ["#", "TITLE", "ARTIST", "DUR."] {
        assert!(h.query_by_label(label).is_some(), "{label}");
    }
    for label in ["ALBUM", "DATE", "GENRE", "INTRO", "FILE NAME"] {
        assert!(h.query_by_label(label).is_none(), "{label}");
    }
}

#[test]
fn the_optional_columns_show_their_headers_and_cells() {
    let columns = [Title, Album, Date, Genre, FileName, Intro, Duration];
    let (h, _) = harness(with_columns(tagged(state(1, 3)), &columns));
    for label in ["ALBUM", "DATE", "GENRE", "FILE NAME", "INTRO"] {
        assert!(h.query_by_label(label).is_some(), "{label}");
    }
    assert!(h.query_by_label("#").is_none() && h.query_by_label("ARTIST").is_none());
    assert!(h.query_by_label("Album 2").is_some());
    assert!(h.query_by_label("2001-05-14").is_some());
    assert!(h.query_by_label("Genre 3").is_some());
    assert!(h.query_by_label("Song 2.mp3").is_some(), "the file name");
    assert!(h.query_by_label("00:13").is_some(), "the intro of track 2");
}

#[test]
fn the_columns_follow_the_order_of_the_list() {
    let (h, _) = harness(with_columns(
        tagged(state(1, 3)),
        &[Duration, Genre, Title, Album],
    ));
    let x: Vec<f32> = ["DUR.", "GENRE", "TITLE", "ALBUM"]
        .iter()
        .map(|l| header_left(&h, l))
        .collect();
    assert!(x.windows(2).all(|w| w[0] < w[1]), "{x:?}");
}

#[test]
fn a_list_without_the_required_columns_still_shows_them() {
    // Set directly, as a test or a bug could: the table repairs the list.
    let (h, _) = harness(with_columns(state(1, 3), &[Artist]));
    assert!(h.query_by_label("TITLE").is_some());
    assert!(h.query_by_label("DUR.").is_some());
    assert!(h.query_by_label("ARTIST").is_some());
}

#[test]
fn a_track_without_a_value_shows_an_empty_cell() {
    let (h, _) = harness(with_columns(state(1, 2), &[Title, Date, Intro, Duration]));
    assert!(h.query_by_label("DATE").is_some());
    assert!(h.query_by_label_contains("-05-14").is_none());
}

#[test]
fn every_column_fits_the_table_width() {
    let all = [
        Number, Title, Artist, Album, Date, Genre, Duration, Intro, FileName,
    ];
    let (h, fake) = harness(with_columns(state(1, 3), &all));
    let px = h.state().column_widths(fake.player(0)).unwrap();
    assert_eq!(px.len(), 9);
    let digits = 1;
    for (w, c) in px.iter().zip(all) {
        assert!(*w >= column_min(c, digits) - 0.5, "{c:?} {w}");
    }
}

// --- live resizing (O16)

fn press(h: &mut Harness<'_, AppUi>, at: Pos2, pressed: bool) {
    h.event(Event::PointerButton {
        pos: at,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    });
}

/// The point on the edge between the columns whose headers are `left_of`
/// and `right_of` (the right one starts at the edge; its label sits 8
/// points inside).
fn edge(h: &Harness<'_, AppUi>, right_of: &str) -> Pos2 {
    let r = h.get_by_label(right_of).rect();
    pos2(r.left() - 8.0, r.center().y)
}

fn widths(h: &Harness<'_, AppUi>, fake: &Fake) -> Vec<f32> {
    h.state().column_widths(fake.player(0)).unwrap()
}

fn resize_commands(fake: &Fake) -> Vec<Command> {
    fake.take_sent()
        .into_iter()
        .filter(|c| matches!(c, Command::SetColumnWidths(..)))
        .collect()
}

fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 1.0
}

#[test]
fn dragging_an_edge_recomputes_the_other_columns_every_frame() {
    let columns = [Number, Title, Artist, Album, Duration];
    let (mut h, fake) = harness(with_columns(state(1, 3), &columns));
    let before = widths(&h, &fake);
    let from = edge(&h, "ARTIST");
    h.event(Event::PointerMoved(from));
    h.run_steps(1);
    press(&mut h, from, true);
    h.run_steps(2);
    fake.take_sent();
    let mut seen = vec![before.clone()];
    for step in 1..=6 {
        h.event(Event::PointerMoved(pos2(
            from.x + step as f32 * 15.0,
            from.y,
        )));
        h.run_steps(1);
        seen.push(widths(&h, &fake));
    }
    // Title grew with the pointer, and on every frame.
    for pair in seen.windows(2) {
        assert!(pair[1][1] > pair[0][1] + 5.0, "{pair:?}");
    }
    let last = seen.last().unwrap();
    assert!(near(last[1], before[1] + 90.0), "{before:?} -> {last:?}");
    // The columns left of the edge did not move; those on its right shrank
    // together and still fill the table.
    assert!(near(last[0], before[0]));
    assert!(last[2] < before[2] && last[3] < before[3] && last[4] <= before[4]);
    assert!(near(last.iter().sum::<f32>(), before.iter().sum::<f32>()));
    assert!(
        resize_commands(&fake).is_empty(),
        "nothing is stored while the button is down"
    );
}

#[test]
fn the_widths_are_stored_once_on_release() {
    let columns = [Number, Title, Artist, Album, Duration];
    let (mut h, fake) = harness(with_columns(state(1, 3), &columns));
    let from = edge(&h, "ARTIST");
    h.event(Event::PointerMoved(from));
    h.run_steps(1);
    press(&mut h, from, true);
    h.run_steps(2);
    for step in 1..=6 {
        h.event(Event::PointerMoved(pos2(
            from.x + step as f32 * 15.0,
            from.y,
        )));
        h.run_steps(1);
    }
    let live = widths(&h, &fake);
    press(&mut h, pos2(from.x + 90.0, from.y), false);
    h.run_steps(3);
    let sent = resize_commands(&fake);
    assert_eq!(sent.len(), 1, "{sent:?}");
    let Command::SetColumnWidths(p, stored) = &sent[0] else {
        panic!()
    };
    assert_eq!(*p, fake.player(0));
    let total: f32 = live.iter().sum();
    for (c, w) in columns.iter().zip(&live) {
        let f = stored.fraction(*c).expect("every shown column is stored");
        assert!((f - w / total).abs() < 0.01, "{c:?}: {f} vs {}", w / total);
    }
    // The model has them now, and the table keeps drawing them.
    h.run_steps(30);
    let after = widths(&h, &fake);
    for (a, b) in after.iter().zip(&live) {
        assert!(near(*a, *b), "{after:?} vs {live:?}");
    }
}

#[test]
fn a_plain_click_on_an_edge_stores_nothing() {
    let (mut h, fake) = harness(state(1, 3));
    let from = edge(&h, "ARTIST");
    h.event(Event::PointerMoved(from));
    h.run_steps(1);
    press(&mut h, from, true);
    h.run_steps(1);
    press(&mut h, from, false);
    h.run_steps(3);
    assert!(resize_commands(&fake).is_empty());
}

#[test]
fn a_drag_cannot_squeeze_a_column_below_its_minimum() {
    let columns = [Title, Artist, Duration];
    let (mut h, fake) = harness(with_columns(state(1, 3), &columns));
    let from = edge(&h, "ARTIST");
    h.event(Event::PointerMoved(from));
    h.run_steps(1);
    press(&mut h, from, true);
    h.run_steps(2);
    for step in 1..=20 {
        h.event(Event::PointerMoved(pos2(
            from.x + step as f32 * 50.0,
            from.y,
        )));
        h.run_steps(1);
    }
    let w = widths(&h, &fake);
    assert!(
        w[1] >= column_min(Artist, 1) - 0.5 && w[2] >= column_min(Duration, 1) - 0.5,
        "{w:?}"
    );
    press(&mut h, pos2(from.x + 1000.0, from.y), false);
    h.run_steps(2);
}

#[test]
fn the_window_growing_does_not_store_widths_and_keeps_the_shares() {
    let columns = [Title, Artist, Album, Duration];
    let (mut h, fake) = harness(with_columns(state(1, 3), &columns));
    let before = widths(&h, &fake);
    fake.take_sent();
    h.set_size(egui::vec2(1400.0, 700.0));
    h.run_steps(3);
    let after = widths(&h, &fake);
    assert!(after.iter().sum::<f32>() > before.iter().sum::<f32>() + 300.0);
    assert!(resize_commands(&fake).is_empty());
    assert!(
        near(after[1] / after[2], before[1] / before[2])
            || (after[1] / after[2] - before[1] / before[2]).abs() < 0.05
    );
}

#[test]
fn a_column_shown_later_takes_its_default_width_and_keeps_the_others_shares() {
    let (mut h, fake) = harness(state(1, 3));
    let p = fake.player(0);
    let mut s = (*fake.state.load_full()).clone();
    s.config.ui.table_columns = vec![Number, Title, Artist, Date, Duration];
    fake.state.store(std::sync::Arc::new(s));
    h.run_steps(3);
    let after = h.state().column_widths(p).unwrap();
    assert_eq!(after.len(), 5);
    assert!(near(after[3], 84.0), "{after:?}");
    assert!(near(
        after.iter().sum::<f32>(),
        widths(&h, &fake).iter().sum::<f32>()
    ));
}

#[test]
fn nine_columns_in_the_narrowest_player_never_overflow_or_go_negative() {
    let all = [
        Number, Title, Artist, Album, Date, Genre, Duration, Intro, FileName,
    ];
    let (h, fake) = harness_sized(
        with_columns(state(1, 3), &all),
        egui::vec2(380.0, 700.0),
        |ui| ui,
    );
    let px = h.state().column_widths(fake.player(0)).unwrap();
    assert_eq!(px.len(), 9);
    assert!(px.iter().all(|w| w.is_finite() && *w >= 0.0), "{px:?}");
    assert!(px.iter().sum::<f32>() <= 380.0, "{px:?}");
}

#[test]
fn changing_the_columns_during_a_drag_ends_it_without_storing_anything() {
    let columns = [Number, Title, Artist, Album, Duration];
    let (mut h, fake) = harness(with_columns(state(1, 3), &columns));
    let from = edge(&h, "ARTIST");
    h.event(Event::PointerMoved(from));
    h.run_steps(1);
    press(&mut h, from, true);
    h.run_steps(2);
    h.event(Event::PointerMoved(pos2(from.x + 40.0, from.y)));
    h.run_steps(1);
    fake.take_sent();
    let mut s = (*fake.state.load_full()).clone();
    s.config.ui.table_columns = vec![Title, Artist, Duration];
    fake.state.store(std::sync::Arc::new(s));
    h.event(Event::PointerMoved(pos2(from.x + 80.0, from.y)));
    h.run_steps(2);
    press(&mut h, pos2(from.x + 80.0, from.y), false);
    h.run_steps(3);
    assert!(resize_commands(&fake).is_empty());
    assert_eq!(widths(&h, &fake).len(), 3);
}

#[test]
fn a_track_dragged_over_the_header_reorders_no_column() {
    let (mut h, fake) = harness(state(1, 3));
    fake.take_sent();
    let row = centre(&h, "Song 2");
    let header = left_of(&h, "#");
    drag(&mut h, row, header);
    assert!(
        !fake
            .take_sent()
            .iter()
            .any(|c| matches!(c, Command::UpdateConfig(_)))
    );
    assert_eq!(
        header_order(&h, &["#", "TITLE", "ARTIST", "DUR."]),
        ["#", "TITLE", "ARTIST", "DUR."]
    );
}

fn centre(h: &Harness<'_, AppUi>, label: &str) -> Pos2 {
    h.get_by_label(label).rect().center()
}

fn left_of(h: &Harness<'_, AppUi>, label: &str) -> Pos2 {
    let r = h.get_by_label(label).rect();
    pos2(r.left() - 4.0, r.center().y)
}

fn header_order(h: &Harness<'_, AppUi>, labels: &[&str]) -> Vec<String> {
    let mut found: Vec<(f32, String)> = labels
        .iter()
        .filter_map(|l| {
            h.query_by_label(l)
                .map(|n| (n.rect().left(), (*l).to_owned()))
        })
        .collect();
    found.sort_by(|a, b| a.0.total_cmp(&b.0));
    found.into_iter().map(|(_, l)| l).collect()
}

/// Presses at `from`, moves to `to` in steps and releases there.
fn drag(h: &mut Harness<'_, AppUi>, from: Pos2, to: Pos2) {
    h.event(Event::PointerMoved(from));
    h.run_steps(1);
    press(h, from, true);
    h.run_steps(1);
    for step in 1..=10 {
        let t = step as f32 / 10.0;
        h.event(Event::PointerMoved(pos2(
            from.x + (to.x - from.x) * t,
            from.y + (to.y - from.y) * t,
        )));
        h.run_steps(1);
    }
    press(h, to, false);
    h.run_steps(3);
}
```

Existing tests that move with the new shape (`ColumnWidths` is built with `keyed`, `column_widths` is a `Vec`, the old `view::column_px` tests move to `table_layout.rs`):

`crates/fp-app/tests/main_screen.rs`:

```diff
--- a/crates/fp-app/tests/main_screen.rs
+++ b/crates/fp-app/tests/main_screen.rs
@@ -712,9 +712,9 @@ fn flagged_entries_show_their_icons() {
     );
 }
 
-fn shares(w: [f32; 4]) -> [f32; 4] {
+fn shares(w: &[f32]) -> Vec<f32> {
     let sum: f32 = w.iter().sum();
-    w.map(|x| x / sum)
+    w.iter().map(|x| x / sum).collect()
 }
 
 #[test]
@@ -729,7 +729,7 @@ fn columns_fill_the_table_and_keep_their_shares_when_the_window_grows() {
     h.set_size(egui::vec2(1400.0, 700.0));
     h.run_steps(3);
     let after = h.state().column_widths(p).unwrap();
-    let (a, b) = (shares(before), shares(after));
+    let (a, b) = (shares(&before), shares(&after));
     for i in 1..3 {
         assert!((a[i] - b[i]).abs() < 0.03, "{before:?} → {after:?}");
     }
@@ -744,14 +744,17 @@ fn stored_fractions_are_applied() {
         &mut s,
         Command::SetColumnWidths(
             p,
-            fp_model::ColumnWidths {
-                fractions: Some([0.1, 0.3, 0.5, 0.1]),
-            },
+            fp_model::ColumnWidths::keyed([
+                (fp_model::TableColumn::Number, 0.1),
+                (fp_model::TableColumn::Title, 0.3),
+                (fp_model::TableColumn::Artist, 0.5),
+                (fp_model::TableColumn::Duration, 0.1),
+            ]),
         ),
     )
     .unwrap();
     let (h, _) = harness(s);
-    let w = shares(h.state().column_widths(p).unwrap());
+    let w = shares(&h.state().column_widths(p).unwrap());
     assert!(w[2] > w[1], "Artist wider than Title as stored: {w:?}");
 }
 
```

`crates/fp-app/tests/view.rs`:

```diff
--- a/crates/fp-app/tests/view.rs
+++ b/crates/fp-app/tests/view.rs
@@ -275,45 +275,6 @@ fn playlist_times_follow_each_player() {
     assert_eq!(theirs.elapsed, 0.0, "P2 has played nothing: {theirs:?}");
 }
 
-mod columns {
-    use fp_app::ui::view::column_px;
-
-    fn sum(px: [f32; 4]) -> f32 {
-        px.iter().sum()
-    }
-
-    #[test]
-    fn the_default_gives_the_minimums_and_splits_the_rest_60_40() {
-        let px = column_px(None, 1000.0, 40.0, 60.0);
-        assert_eq!((px[0], px[3]), (40.0, 60.0));
-        assert!(
-            (px[1] - 540.0).abs() < 0.01 && (px[2] - 360.0).abs() < 0.01,
-            "{px:?}"
-        );
-        assert!((sum(px) - 1000.0).abs() < 0.01);
-    }
-
-    #[test]
-    fn fractions_scale_with_the_width() {
-        let f = Some([0.05, 0.5, 0.35, 0.1]);
-        let a = column_px(f, 1000.0, 40.0, 60.0);
-        let b = column_px(f, 1500.0, 40.0, 60.0);
-        assert!((sum(b) - 1500.0).abs() < 0.01);
-        assert!((b[1] / a[1] - 1.5).abs() < 0.01, "{a:?} {b:?}");
-        assert!((b[2] / a[2] - 1.5).abs() < 0.01);
-    }
-
-    #[test]
-    fn the_minimums_win_in_a_narrow_table() {
-        let px = column_px(Some([0.01, 0.6, 0.38, 0.01]), 360.0, 40.0, 60.0);
-        assert!(px[0] >= 40.0 && px[3] >= 60.0, "{px:?}");
-        assert!(px.iter().all(|w| *w >= 0.0 && w.is_finite()));
-        assert!((sum(px) - 360.0).abs() < 0.01);
-        let tiny = column_px(None, 50.0, 40.0, 60.0);
-        assert!(tiny.iter().all(|w| *w >= 0.0 && w.is_finite()), "{tiny:?}");
-    }
-}
-
 fn mark(s: &mut AppState, track: fp_model::TrackId, kind: MarkerKind, secs: f64) {
     apply(
         s,
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `cargo test -p fp-model --test column_widths --test intro_column` — expected: does not compile (`ColumnWidths::keyed`, `Track::intro_secs`). The fp-app tests do not compile either (`table_layout`, `cell_text`).

- [ ] **Step 3: Implement the model**

`crates/fp-model/src/player.rs`:

```diff
--- a/crates/fp-model/src/player.rs
+++ b/crates/fp-model/src/player.rs
@@ -1,7 +1,11 @@
 //! Per-player state.
 
+use std::collections::BTreeMap;
+
 use serde::{Deserialize, Serialize};
 
+use crate::columns::TableColumn;
+
 use crate::command::TransitionPlan;
 use crate::ids::{EntryId, PlayerId, PlaylistId};
 
@@ -29,23 +33,72 @@ pub struct CueState {
     pub paused: bool,
 }
 
-/// The track table's column widths as fractions of its width (`#`, Title,
-/// Artist, Duration), summing to 1; `None` is the default layout
-/// (feedback spec §2.3). Pixel widths saved by earlier versions are ignored.
-#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
+/// The track table's column widths as fractions of its width, keyed by
+/// column and summing to 1 over the columns that were shown when they were
+/// stored; `None` is the default layout (feedback 2 spec O24). A column
+/// missing from the map takes its default width. Widths saved by earlier
+/// versions as four fractions (`#`, Title, Artist, Duration) are converted
+/// when they are four good numbers; pixel widths and anything else give the
+/// default layout.
+#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
 #[serde(default)]
 pub struct ColumnWidths {
-    pub fractions: Option<[f32; 4]>,
+    #[serde(deserialize_with = "lenient_fractions")]
+    pub fractions: Option<BTreeMap<TableColumn, f32>>,
+}
+
+/// The four columns the old fraction array described, in its order.
+const LEGACY_COLUMNS: [TableColumn; 4] = [
+    TableColumn::Number,
+    TableColumn::Title,
+    TableColumn::Artist,
+    TableColumn::Duration,
+];
+
+fn lenient_fractions<'de, D: serde::Deserializer<'de>>(
+    d: D,
+) -> Result<Option<BTreeMap<TableColumn, f32>>, D::Error> {
+    #[derive(Deserialize)]
+    #[serde(untagged)]
+    enum Raw {
+        Legacy(Vec<f32>),
+        Keyed(BTreeMap<String, f32>),
+        Other(serde::de::IgnoredAny),
+    }
+    Ok(match Option::<Raw>::deserialize(d)? {
+        Some(Raw::Legacy(values)) if values.len() == LEGACY_COLUMNS.len() => {
+            Some(LEGACY_COLUMNS.into_iter().zip(values).collect())
+        }
+        Some(Raw::Keyed(map)) => Some(
+            map.into_iter()
+                .filter_map(|(name, f)| TableColumn::from_name(&name).map(|c| (c, f)))
+                .collect(),
+        ),
+        _ => None,
+    })
 }
 
 impl ColumnWidths {
-    /// Fractions scaled to sum to 1; broken ones (not finite, negative, all
-    /// zero) give the default layout.
+    /// Widths from `(column, fraction)` pairs, normalised.
+    pub fn keyed(pairs: impl IntoIterator<Item = (TableColumn, f32)>) -> Self {
+        Self {
+            fractions: Some(pairs.into_iter().collect()),
+        }
+        .normalized()
+    }
+
+    /// The stored fraction of `column`, if any.
+    pub fn fraction(&self, column: TableColumn) -> Option<f32> {
+        self.fractions.as_ref()?.get(&column).copied()
+    }
+
+    /// Fractions scaled to sum to 1; broken ones (not finite, negative, none
+    /// left, all zero) give the default layout.
     pub fn normalized(self) -> Self {
         let fractions = self.fractions.and_then(|f| {
-            let valid = f.iter().all(|x| x.is_finite() && *x >= 0.0);
-            let sum: f32 = f.iter().sum();
-            (valid && sum > 0.0).then(|| f.map(|x| x / sum))
+            let valid = f.values().all(|x| x.is_finite() && *x >= 0.0);
+            let sum: f32 = f.values().sum();
+            (valid && sum > 0.0).then(|| f.into_iter().map(|(c, x)| (c, x / sum)).collect())
         });
         Self { fractions }
     }
```

`crates/fp-model/src/session.rs`:

```diff
--- a/crates/fp-model/src/session.rs
+++ b/crates/fp-model/src/session.rs
@@ -99,7 +99,7 @@ impl AppState {
                     0.0
                 },
                 volume: p.volume,
-                columns: p.columns,
+                columns: p.columns.clone(),
                 history: p.history.clone(),
             })
             .collect()
@@ -261,7 +261,7 @@ fn restore_player(state: &mut AppState, s: &PlayerSession) -> PlayerState {
     };
     player.stop_after_current = s.mode == PlayMode::Continuous && s.stop_after_current;
     player.volume = volume;
-    player.columns = s.columns.normalized();
+    player.columns = s.columns.clone().normalized();
     // Only entries that still exist, and no more than the configured depth.
     let kept: Vec<EntryId> = s
         .history
```

`crates/fp-model/src/track.rs`:

```diff
--- a/crates/fp-model/src/track.rs
+++ b/crates/fp-model/src/track.rs
@@ -433,6 +433,16 @@ impl Track {
             }
         }
     }
+
+    /// Feedback 2 spec O24 (Intro column): how long the intro lasts, from
+    /// the start of the audible part (the cue-in, with markers on) to the
+    /// intro-end marker. `None` without a marker, or when the marker is not
+    /// after the start.
+    pub fn intro_secs(&self, use_markers: bool) -> Option<f64> {
+        let end = self.intro_end_secs()?;
+        let length = end - self.play_range(use_markers).cue_in;
+        (length > 0.0).then_some(length)
+    }
 }
 
 /// What analysis learned about a file (spec §6). Metadata fields are
```

Run: `cargo test -p fp-model` — expected: pass (the whole model suite, including `session` and `editing`).

- [ ] **Step 4: Implement the pure layout and the cell text**

`crates/fp-app/src/ui/table_layout.rs` (new file):

```rust
//! The widths of the track table's columns (feedback 2 spec O16 and O24):
//! pure functions from the stored fractions to pixels, and from a drag of a
//! column edge to the widths of all the columns, recomputed every frame.

use fp_model::{ColumnWidths, TableColumn};

/// Width of one digit of the `#` column, and what its icon and padding take.
const DIGIT: f32 = 8.0;
const NUMBER_PADDING: f32 = 26.0;

/// The narrowest a column may be, in points. The `#` column grows with the
/// number of digits of the playlist length.
pub fn column_min(column: TableColumn, number_digits: usize) -> f32 {
    match column {
        TableColumn::Number => number_digits as f32 * DIGIT + NUMBER_PADDING,
        TableColumn::Title | TableColumn::Artist | TableColumn::Album | TableColumn::FileName => {
            60.0
        }
        TableColumn::Genre => 50.0,
        // "2019-05-14" fits.
        TableColumn::Date => 84.0,
        // "00:00:00" fits.
        TableColumn::Duration | TableColumn::Intro => 52.0,
    }
}

/// How a column shares the room that the fixed ones leave (the default
/// layout): the text columns by these weights, the others at their minimum.
fn weight(column: TableColumn) -> f32 {
    match column {
        TableColumn::Title => 3.0,
        TableColumn::Artist | TableColumn::Album | TableColumn::FileName => 2.0,
        TableColumn::Genre => 1.5,
        TableColumn::Number | TableColumn::Date | TableColumn::Duration | TableColumn::Intro => 0.0,
    }
}

/// Widths close to `wanted` that respect `mins` and add up to `total`:
/// columns that would be too narrow take their minimum and the others share
/// the rest in proportion to what they wanted. When even the minimums do not
/// fit, they are all scaled down.
pub fn fit(wanted: &[f32], mins: &[f32], total: f32) -> Vec<f32> {
    let n = wanted.len().min(mins.len());
    let total = if total.is_finite() {
        total.max(0.0)
    } else {
        0.0
    };
    let wanted: Vec<f32> = wanted
        .iter()
        .take(n)
        .map(|w| if w.is_finite() { w.max(0.0) } else { 0.0 })
        .collect();
    let mins: Vec<f32> = mins.iter().take(n).map(|m| m.max(0.0)).collect();
    let min_sum: f32 = mins.iter().sum();
    if n == 0 {
        return Vec::new();
    }
    if min_sum >= total {
        let scale = if min_sum > 0.0 { total / min_sum } else { 0.0 };
        return mins.iter().map(|m| m * scale).collect();
    }
    let mut pinned = vec![false; n];
    loop {
        let pinned_sum: f32 = mins
            .iter()
            .zip(&pinned)
            .filter(|(_, p)| **p)
            .map(|(m, _)| m)
            .sum();
        let free_wanted: f32 = wanted
            .iter()
            .zip(&pinned)
            .filter(|(_, p)| !**p)
            .map(|(w, _)| w)
            .sum();
        let free_total = total - pinned_sum;
        let free_count = pinned.iter().filter(|p| !**p).count();
        let share = |w: f32| {
            if free_wanted > 0.0 {
                w * free_total / free_wanted
            } else {
                free_total / free_count as f32
            }
        };
        let mut changed = false;
        for ((pin, w), m) in pinned.iter_mut().zip(&wanted).zip(&mins) {
            if !*pin && share(*w) < *m {
                *pin = true;
                changed = true;
            }
        }
        if !changed {
            return wanted
                .iter()
                .zip(&mins)
                .zip(&pinned)
                .map(|((w, m), pin)| if *pin { *m } else { share(*w) })
                .collect();
        }
    }
}

/// The default layout of `columns` in a table `width` wide: the narrow
/// columns (`#`, Date, Duration, Intro) at their minimum and the text
/// columns sharing the rest, Title 3, Artist 2, Album 2, File name 2 and
/// Genre 1.5. With `#`, Title, Artist and Duration that is the layout the
/// table always had: the minimums for `#` and Duration and 60:40 of the rest.
fn default_px(columns: &[TableColumn], width: f32, digits: usize) -> Vec<f32> {
    let mins: Vec<f32> = columns.iter().map(|c| column_min(*c, digits)).collect();
    let fixed: f32 = columns
        .iter()
        .zip(&mins)
        .filter(|(c, _)| weight(**c) == 0.0)
        .map(|(_, m)| m)
        .sum();
    let weights: f32 = columns.iter().map(|c| weight(*c)).sum();
    let rest = (width - fixed).max(0.0);
    let wanted: Vec<f32> = columns
        .iter()
        .zip(&mins)
        .map(|(c, m)| {
            if weights > 0.0 && weight(*c) > 0.0 {
                rest * weight(*c) / weights
            } else {
                *m
            }
        })
        .collect();
    fit(&wanted, &mins, width)
}

/// Pixel widths of `columns` for a table `width` wide. The stored fractions
/// of the columns that have one are used, scaled to the room left by the
/// columns without one (a column shown after the widths were stored), which
/// take their default width; with no stored fractions every column does.
/// The minimums win, and the widths always add up to `width`.
pub fn column_px(
    columns: &[TableColumn],
    stored: &ColumnWidths,
    width: f32,
    number_digits: usize,
) -> Vec<f32> {
    let width = if width.is_finite() {
        width.max(0.0)
    } else {
        0.0
    };
    let default = default_px(columns, width, number_digits);
    let stored = stored.clone().normalized();
    let have: Vec<Option<f32>> = columns.iter().map(|c| stored.fraction(*c)).collect();
    let stored_sum: f32 = have.iter().flatten().sum();
    if stored_sum <= 0.0 {
        return default;
    }
    let reserved: f32 = have
        .iter()
        .zip(&default)
        .filter(|(f, _)| f.is_none())
        .map(|(_, d)| d)
        .sum();
    let room = (width - reserved).max(0.0);
    let wanted: Vec<f32> = have
        .iter()
        .zip(&default)
        .map(|(f, d)| f.map_or(*d, |f| f / stored_sum * room))
        .collect();
    let mins: Vec<f32> = columns
        .iter()
        .map(|c| column_min(*c, number_digits))
        .collect();
    fit(&wanted, &mins, width)
}

/// The widths while the right edge of column `edge` is dragged so that the
/// column is `new_width` wide (O16). The columns on its left keep their
/// width; the ones on its right share what is left in proportion to their
/// width when the drag began, so every one of them moves on every frame. No
/// column goes under its minimum, and the total never changes. The last
/// column has no edge to its right: nothing moves.
pub fn resize_px(start: &[f32], mins: &[f32], edge: usize, new_width: f32) -> Vec<f32> {
    let n = start.len().min(mins.len());
    if edge + 1 >= n || !new_width.is_finite() {
        return start.to_vec();
    }
    let total: f32 = start.iter().sum();
    let left: f32 = start.iter().take(edge).sum();
    let right_min: f32 = mins.iter().take(n).skip(edge + 1).sum();
    let lowest = mins.get(edge).copied().unwrap_or(0.0);
    let highest = (total - left - right_min).max(lowest);
    let width = new_width.clamp(lowest, highest);
    let mut out: Vec<f32> = start.iter().take(edge).copied().collect();
    out.push(width);
    let rest = total - left - width;
    let wanted: Vec<f32> = start.iter().take(n).skip(edge + 1).copied().collect();
    let right_mins: Vec<f32> = mins.iter().take(n).skip(edge + 1).copied().collect();
    out.extend(fit(&wanted, &right_mins, rest));
    out
}

/// The widths to store for `px`: each column's share of the total.
pub fn fractions_of(columns: &[TableColumn], px: &[f32]) -> ColumnWidths {
    ColumnWidths::keyed(columns.iter().copied().zip(px.iter().copied()))
}
```

`crates/fp-app/src/ui.rs`:

```diff
--- a/crates/fp-app/src/ui.rs
+++ b/crates/fp-app/src/ui.rs
@@ -18,6 +18,7 @@ mod reset_played;
 mod settings;
 pub mod shell;
 mod table;
+pub mod table_layout;
 mod tag_editor;
 pub mod theme;
 pub mod view;
```

`crates/fp-app/src/ui/view.rs`:

```diff
--- a/crates/fp-app/src/ui/view.rs
+++ b/crates/fp-app/src/ui/view.rs
@@ -2,7 +2,8 @@
 //! the engine telemetry. Pure functions: everything here is unit-tested.
 
 use fp_model::{
-    AppState, EntryId, FileState, PlayMode, PlayerId, PlaylistEntry, PlaylistId, Track, Transport,
+    AppState, EntryId, FileState, PlayMode, PlayerId, PlaylistEntry, PlaylistId, TableColumn,
+    Track, Transport,
 };
 
 #[derive(Debug, Clone, Copy, PartialEq, Eq)]
@@ -358,44 +359,35 @@ pub fn volume_db(gain: f32) -> Option<f32> {
     (gain > 0.0 && !gain.is_nan()).then(|| 20.0 * gain.log10())
 }
 
-/// Title and Artist are never narrower than this while the table allows.
-const TEXT_COLUMN_MIN: f32 = 60.0;
-
-/// Pixel widths of the track table's columns (`#`, Title, Artist, Duration)
-/// for a table `width` wide (feedback spec §2.3): the stored fractions, or
-/// by default the minimums for `#` and Duration and 60:40 of the rest. The
-/// minimums win, and the widths always sum to `width`.
-pub fn column_px(
-    fractions: Option<[f32; 4]>,
-    width: f32,
-    number_min: f32,
-    duration_min: f32,
-) -> [f32; 4] {
-    let width = if width.is_finite() {
-        width.max(0.0)
-    } else {
-        0.0
-    };
-    let wanted = fp_model::ColumnWidths { fractions }
-        .normalized()
-        .fractions
-        .map(|f| f.map(|x| x * width));
-    let [n, t, a, d] = wanted.unwrap_or([number_min, 0.6, 0.4, duration_min]);
-    let (mut number, mut duration) = (n.max(number_min), d.max(duration_min));
-    if number + duration > width {
-        // Not even the minimums fit: share what there is.
-        let scale = width / (number + duration).max(f32::EPSILON);
-        number *= scale;
-        duration *= scale;
-        return [number, 0.0, 0.0, duration];
-    }
-    let rest = width - number - duration;
-    let share = if t + a > 0.0 { t / (t + a) } else { 0.6 };
-    let mut title = rest * share;
-    if rest >= 2.0 * TEXT_COLUMN_MIN {
-        title = title.clamp(TEXT_COLUMN_MIN, rest - TEXT_COLUMN_MIN);
+/// The text a table cell shows for `column` (feedback 2 spec O24). The `#`
+/// column draws its own icon and number, so it has none here. A column the
+/// track has no value for is empty; the times follow the play range
+/// (`use_markers` is `players.use_cue_markers`).
+pub fn cell_text(track: &Track, column: TableColumn, use_markers: bool) -> String {
+    match column {
+        TableColumn::Number => String::new(),
+        TableColumn::Title => track.title.clone(),
+        TableColumn::Artist => track.artist.clone(),
+        TableColumn::Album => track.album.clone(),
+        TableColumn::Date => track.date.clone().unwrap_or_default(),
+        TableColumn::Genre => track.genre.clone(),
+        TableColumn::Duration => {
+            if track.duration_secs > 0.0 {
+                super::format::clock(track.play_range(use_markers).length())
+            } else {
+                String::new()
+            }
+        }
+        TableColumn::Intro => track
+            .intro_secs(use_markers)
+            .map(super::format::clock)
+            .unwrap_or_default(),
+        TableColumn::FileName => track
+            .path
+            .file_name()
+            .map(|n| n.to_string_lossy().into_owned())
+            .unwrap_or_default(),
     }
-    [number, title, rest - title, duration]
 }
 
 /// A line of the row tooltip (feedback 2 spec O23).
```

`crates/fp-app/src/ui/app.rs`:

```diff
--- a/crates/fp-app/src/ui/app.rs
+++ b/crates/fp-app/src/ui/app.rs
@@ -81,8 +81,11 @@ pub(crate) struct ViewState {
     pub active_player: Option<PlayerId>,
     pub drop: Option<DropTarget>,
     pub file_drop: Option<DropTarget>,
-    pub widths: HashMap<PlayerId, [f32; 4]>,
-    pub resizing: HashSet<PlayerId>,
+    /// The pixel widths of each player's table columns in the last frame,
+    /// in the order of `ui.table_columns`.
+    pub widths: HashMap<PlayerId, Vec<f32>>,
+    /// The column edge being dragged (feedback 2 spec O16).
+    pub(crate) live_resize: Option<super::table::LiveResize>,
     pub rows_built: usize,
     pub settings_open: bool,
     pub about_open: bool,
@@ -109,9 +112,6 @@ pub(crate) struct ViewState {
     /// The players whose table already had its start-up scroll to the next
     /// entry (feedback 2 spec O7); it happens once, on their first frame.
     pub startup_scrolled: HashSet<PlayerId>,
-    /// The table width and column fractions each player's table was last
-    /// laid out with (a change resets egui's column widths).
-    pub table_layout: HashMap<PlayerId, (f32, Option<[f32; 4]>)>,
     /// Zoomed waveforms; a player without one shows the whole track.
     pub wave_zoom: HashMap<PlayerId, super::wave_view::WaveZoom>,
     /// A marker being dragged on a waveform, and the track it belongs to.
@@ -408,9 +408,10 @@ impl AppUi {
         self.view.rows_built
     }
 
-    /// The pixel widths of a player's table columns in the last frame.
-    pub fn column_widths(&self, player: PlayerId) -> Option<[f32; 4]> {
-        self.view.widths.get(&player).copied()
+    /// The pixel widths of a player's table columns in the last frame, in
+    /// the order of `ui.table_columns`.
+    pub fn column_widths(&self, player: PlayerId) -> Option<Vec<f32>> {
+        self.view.widths.get(&player).cloned()
     }
 
     pub fn ui(&mut self, ui: &mut Ui) {
```

- [ ] **Step 5: Rewrite the table**

In `crates/fp-app/src/ui/table.rs` replace everything above `fn context_menu(` (the imports, the constants, `header_label`, `track_table`, and what replaces `store_widths`) with the following; keep `context_menu`, `flag` and `track_tip` as they are. The old `DURATION_MIN`, `store_widths`, `ViewState::table_layout` and `ViewState::resizing` are gone (the app.rs hunk above removed the last two).

```rust
//! The track table of a player column (spec §8.3): virtualised rows,
//! resizable columns, row colours, context menu and drag and drop.

use egui::{Align, Color32, Layout, Rect, RichText, Sense, Ui, pos2, vec2};
use egui_extras::{Column, TableBuilder};
use egui_phosphor::regular as icon;
use fp_model::{ColumnWidths, Command, EntryId, PlayerId, PlaylistId, TableColumn, Transport};

use super::app::{DragEntry, DropTarget, FollowScroll, Scene, ViewState};
use super::format;
use super::glyphs::{self, TransportAction};
use super::table_layout;
use super::theme;
use super::view::{self, RowStatus};
use super::widgets::{self, font, font_medium};

const HEADER_HEIGHT: f32 = 24.0;
const ROW_HEIGHT: f32 = 28.0;
/// How long a released column edge keeps its widths while the model catches
/// up with the command that stores them.
const HOLD_SECS: f64 = 0.5;

/// A column edge being dragged (feedback 2 spec O16): the widths are
/// recomputed from the pointer on every frame and sent once, on release.
pub(crate) struct LiveResize {
    player: PlayerId,
    /// The column whose right edge is dragged.
    edge: usize,
    columns: Vec<TableColumn>,
    /// The table width the drag began with.
    width: f32,
    start_px: Vec<f32>,
    start_x: f32,
    /// The widths now.
    px: Vec<f32>,
    /// Set on release: the stored widths at that moment, and until when the
    /// released widths are still drawn.
    released: Option<(ColumnWidths, f64)>,
}

fn header_label(ui: &mut Ui, text: &str, right: bool) {
    let label = egui::Label::new(
        RichText::new(text.to_uppercase())
            .font(font(10.0))
            .color(theme::NEUTRAL_500),
    )
    .selectable(false);
    if right {
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add_space(8.0);
            ui.add(label);
        });
    } else {
        ui.add_space(8.0);
        ui.add(label.truncate());
    }
}

/// The widths to draw: the model's, or the ones of a column edge being
/// dragged (or just released). Called before anything is drawn, so a drag
/// shows in the same frame as the pointer move.
#[allow(clippy::too_many_arguments)]
fn live_widths(
    ui: &Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    player: PlayerId,
    columns: &[TableColumn],
    mins: &[f32],
    stored: &ColumnWidths,
    width: f32,
    from_model: Vec<f32>,
) -> Vec<f32> {
    let Some(live) = view_state
        .live_resize
        .as_mut()
        .filter(|l| l.player == player)
    else {
        return from_model;
    };
    if live.released.is_none() {
        let (down, pointer) = ui.input(|i| (i.pointer.primary_down(), i.pointer.latest_pos()));
        if down {
            if let Some(pos) = pointer
                && let Some(start) = live.start_px.get(live.edge)
            {
                let wanted = start + pos.x - live.start_x;
                live.px = table_layout::resize_px(&live.start_px, mins, live.edge, wanted);
            }
        } else {
            // Only a drag that moved something stores widths: a click on an
            // edge, or a drag back to where it began, does not.
            let moved = live
                .px
                .iter()
                .zip(&live.start_px)
                .any(|(a, b)| (a - b).abs() > 0.5);
            let new = table_layout::fractions_of(&live.columns, &live.px);
            if moved && new != *stored {
                scene.ctl.send(Command::SetColumnWidths(player, new));
            }
            live.released = Some((stored.clone(), scene.time + HOLD_SECS));
        }
    }
    let stale = live.columns != columns
        || (live.width - width).abs() > 0.5
        || live
            .released
            .as_ref()
            .is_some_and(|(before, until)| scene.time >= *until || stored != before);
    if stale {
        view_state.live_resize = None;
        from_model
    } else {
        live.px.clone()
    }
}

pub(crate) fn track_table(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    player: PlayerId,
    playlist: PlaylistId,
) {
    let Some(list) = scene.state.playlists.get(playlist) else {
        return;
    };
    let Ok(p) = scene.state.player(player) else {
        return;
    };
    let t = scene.i18n;
    let digits = format::number_width(list.entries.len());
    let use_markers = scene.state.config.players.use_cue_markers;
    // O24: one list of columns for every table.
    let columns = fp_model::normalize_columns(&scene.state.config.ui.table_columns);
    let mins: Vec<f32> = columns
        .iter()
        .map(|c| table_layout::column_min(*c, digits))
        .collect();
    // Proportional columns (feedback spec F6): pixel widths from the stored
    // fractions every frame, so that they follow the window (O24: fractions
    // keyed by column). While an edge is dragged the widths of all the
    // columns are recomputed from the pointer every frame and stored on
    // release (O16). The table never keeps widths of its own: every column
    // is given its exact width on every frame.
    let width = (ui.available_width() - ui.spacing().scroll.allocated_width()).max(0.0);
    let from_model = table_layout::column_px(&columns, &p.columns, width, digits);
    let px = live_widths(
        ui, scene, view_state, player, &columns, &mins, &p.columns, width, from_model,
    );
    view_state.widths.insert(player, px.clone());
    let area = ui.max_rect();
    // Read before the table's scroll area takes the wheel for itself.
    let wheel_over_table =
        ui.rect_contains_pointer(area) && ui.input(|i| i.smooth_scroll_delta != egui::Vec2::ZERO);
    let pressed_in_table = ui.rect_contains_pointer(area) && ui.input(|i| i.pointer.primary_down());
    ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
    let mut built = 0;
    let mut hovered_row: Option<(usize, bool)> = None;
    let mut pointer_row: Option<(usize, bool)> = None;
    let pointer = ui.ctx().pointer_hover_pos();
    let mut released: Option<(DragEntry, usize)> = None;
    let entries = &list.entries;
    let drop = view_state.drop.filter(|d| d.playlist == playlist);
    let selected = view_state.selection.get(&player).copied();
    let mut clicked: Option<EntryId> = None;
    // O17: the entry a running CUE moves to after a primary click.
    let mut cue_follow: Option<EntryId> = None;
    // O23: the track whose tags the operator asked to edit.
    let mut edit_tags: Option<fp_model::TrackId> = None;
    let mut dragged: Option<EntryId> = None;
    let mut builder = TableBuilder::new(ui)
        .id_salt(("tracks", player.0))
        .striped(false)
        .resizable(false)
        .vscroll(true)
        // The table's default minimum body is 200 points: in a short window
        // the footer would cover the last rows, out of reach of the scroll.
        .min_scrolled_height(0.0)
        .auto_shrink([false, false])
        .sense(Sense::click_and_drag())
        .cell_layout(Layout::left_to_right(Align::Center));
    for w in &px {
        builder = builder.column(Column::exact(w.max(0.0)));
    }
    // A current entry being followed: scroll its row to the top once.
    if let Some(FollowScroll {
        entry,
        align,
        animated,
    }) = view_state.follow_scroll.get(&player).copied()
    {
        match entries.iter().position(|e| e.id == entry) {
            Some(i) => {
                builder = builder
                    .scroll_to_row(i, Some(align))
                    .animate_scrolling(animated);
                view_state.follow_scroll.remove(&player);
            }
            // Not in this playlist: wait for its tab, unless it is gone.
            None if scene.state.playlists.find(entry).is_none() => {
                view_state.follow_scroll.remove(&player);
            }
            None => {}
        }
    }
    let mut menu_open = false;
    builder
        .header(HEADER_HEIGHT, |mut header| {
            for column in &columns {
                header.col(|ui| {
                    let right = matches!(column, TableColumn::Duration | TableColumn::Intro);
                    header_label(ui, &t.tr(&format!("col-{}", column.name())), right);
                });
            }
        })
        .body(|body| {
            body.rows(ROW_HEIGHT, entries.len(), |mut row| {
                built += 1;
                let i = row.index();
                let Some(entry) = entries.get(i) else {
                    return;
                };
                let Some(track) = scene.state.library.get(entry.track) else {
                    return;
                };
                let status = view::row_status(scene.state, player, entry);
                let hi = matches!(status, RowStatus::Current | RowStatus::Next);
                let bg = match status {
                    RowStatus::Current => theme::ON_AIR_ROW,
                    RowStatus::Next => theme::NEXT_ROW,
                    _ if selected == Some(entry.id) => theme::ACCENT_900,
                    _ => Color32::TRANSPARENT,
                };
                let dimmed = status == RowStatus::Played;
                let text = if hi {
                    theme::NEUTRAL_100
                } else if dimmed {
                    theme::NEUTRAL_600
                } else {
                    theme::TEXT
                };
                let artist_color = if hi {
                    theme::NEUTRAL_100
                } else if dimmed {
                    theme::NEUTRAL_600
                } else {
                    theme::NEUTRAL_400
                };
                let row_font = if hi { font_medium(12.0) } else { font(12.0) };
                let line = |ui: &mut Ui| {
                    let r = ui.max_rect();
                    let full = Rect::from_min_max(r.min, pos2(r.max.x, r.min.y + ROW_HEIGHT));
                    ui.painter().rect_filled(full, 0.0, bg);
                    ui.painter().rect_filled(
                        Rect::from_min_size(
                            pos2(full.left(), full.bottom() - 1.0),
                            vec2(full.width(), 1.0),
                        ),
                        0.0,
                        theme::TEXT.gamma_multiply(0.06),
                    );
                    if let Some(d) = drop {
                        let y = if d.index == i {
                            Some(full.top())
                        } else if d.index == entries.len() && i + 1 == entries.len() {
                            Some(full.bottom() - 2.0)
                        } else {
                            None
                        };
                        if let Some(y) = y {
                            ui.painter().rect_filled(
                                Rect::from_min_size(pos2(full.left(), y), vec2(full.width(), 2.0)),
                                0.0,
                                theme::ACCENT,
                            );
                        }
                    }
                };
                for column in &columns {
                    row.col(|ui| {
                        line(ui);
                        match column {
                            TableColumn::Number => {
                                ui.add_space(10.0);
                                let (glyph, color) = match status {
                                    RowStatus::Current => {
                                        let playing = p.transport == Transport::Playing;
                                        let g = if playing {
                                            egui_phosphor::fill::SPEAKER_HIGH
                                        } else {
                                            egui_phosphor::fill::PAUSE
                                        };
                                        (Some(g.to_owned()), theme::NEUTRAL_100)
                                    }
                                    RowStatus::Next => (
                                        Some(icon::ARROW_BEND_DOWN_RIGHT.to_owned()),
                                        theme::NEUTRAL_100,
                                    ),
                                    RowStatus::Unavailable => {
                                        (Some(view::file_icon(track).to_owned()), theme::AMBER)
                                    }
                                    _ => (None, theme::NEUTRAL_600),
                                };
                                if let RowStatus::OnAirElsewhere(n) = status {
                                    // Marked, not highlighted: it is another player's.
                                    let tip = scene
                                        .i18n
                                        .tr_args("tip-on-air-elsewhere", &[("n", n.into())]);
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(format!("P{n}"))
                                                .font(egui::FontId::proportional(11.0))
                                                .color(theme::ON_AIR_TEXT),
                                        )
                                        .selectable(false),
                                    )
                                    .on_hover_text(tip);
                                    return;
                                }
                                let label = match glyph {
                                    Some(g) if hi => g,
                                    Some(g) => format!("{g}{:0digits$}", i + 1),
                                    None => format!("{:0digits$}", i + 1),
                                };
                                let family = if matches!(status, RowStatus::Current) {
                                    egui::FontFamily::Name(theme::ICONS_FILL.into())
                                } else {
                                    egui::FontFamily::Proportional
                                };
                                let number = ui.add(
                                    egui::Label::new(
                                        RichText::new(label)
                                            .font(egui::FontId::new(12.0, family))
                                            .color(color),
                                    )
                                    .selectable(false),
                                );
                                if status == RowStatus::Unavailable
                                    && let Some(tip) = scene.file_tip(entry.track)
                                {
                                    number.on_hover_text(tip);
                                }
                            }
                            TableColumn::Title => {
                                ui.add_space(8.0);
                                // The entry's repeat and stop icons sit before the title,
                                // in the row's text colour (feedback 2 spec O9); the
                                // "analysed by an earlier version" flag stays at the right.
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.spacing_mut().item_spacing.x = 4.0;
                                    ui.add_space(4.0);
                                    // Shown tracks are brought up to date anyway, so
                                    // the flag only stays on the ones waiting.
                                    if crate::services::outdated(track) {
                                        flag(ui, &t.tr("flag-outdated"), |p, r| {
                                            let c = theme::NEUTRAL_500;
                                            widgets::glyph(
                                                p,
                                                r,
                                                icon::ARROWS_CLOCKWISE,
                                                13.0,
                                                c,
                                                false,
                                            );
                                        });
                                    }
                                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                                        ui.spacing_mut().item_spacing.x = 4.0;
                                        if entry.repeat {
                                            flag(ui, &t.tr("flag-repeat"), |p, r| {
                                                widgets::glyph(
                                                    p,
                                                    r,
                                                    icon::REPEAT,
                                                    13.0,
                                                    text,
                                                    false,
                                                );
                                            });
                                        }
                                        if entry.stop_after {
                                            flag(ui, &t.tr("flag-stop-after"), |p, r| {
                                                // Drawn into the flag's own 16x12 box, as before (paint would
                                                // centre the grid's 18x13 size instead).
                                                if let Some(d) =
                                                    glyphs::drawn(TransportAction::StopAfter)
                                                {
                                                    p.extend((d.draw)(r, text));
                                                }
                                            });
                                        }
                                        ui.add(
                                            egui::Label::new(
                                                RichText::new(&track.title)
                                                    .font(row_font.clone())
                                                    .color(text),
                                            )
                                            .selectable(false)
                                            .truncate(),
                                        );
                                    });
                                });
                            }
                            TableColumn::Artist => {
                                ui.add_space(8.0);
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(if track.artist.is_empty() {
                                            t.tr("unknown-artist")
                                        } else {
                                            track.artist.clone()
                                        })
                                        .font(font(12.0))
                                        .color(artist_color),
                                    )
                                    .selectable(false)
                                    .truncate(),
                                );
                            }
                            TableColumn::Duration | TableColumn::Intro => {
                                // Times sit at the right, like the duration.
                                let d = view::cell_text(track, *column, use_markers);
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.add_space(10.0);
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(d).font(font(12.0)).color(text),
                                        )
                                        .selectable(false),
                                    );
                                });
                            }
                            TableColumn::Album
                            | TableColumn::Date
                            | TableColumn::Genre
                            | TableColumn::FileName => {
                                ui.add_space(8.0);
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(view::cell_text(track, *column, use_markers))
                                            .font(font(12.0))
                                            .color(artist_color),
                                    )
                                    .selectable(false)
                                    .truncate(),
                                );
                            }
                        }
                    });
                }
                let response = row.response();
                if response.clicked() {
                    clicked = Some(entry.id);
                    cue_follow = view::cue_follow_target(scene.state, player, entry.id);
                }
                response.clone().on_hover_ui(|ui| {
                    track_tip(ui, scene, track);
                });
                if response.double_clicked() && status != RowStatus::Current {
                    scene.ctl.send(Command::SetNext(player, entry.id));
                }
                if response.drag_started() {
                    response.dnd_set_drag_payload(DragEntry { entry: entry.id });
                    dragged = Some(entry.id);
                }
                if let Some(p) = pointer.filter(|p| response.rect.contains(*p)) {
                    pointer_row = Some((i, p.y > response.rect.center().y));
                }
                if response.dnd_hover_payload::<DragEntry>().is_some()
                    && let Some(pos) = response.hover_pos()
                {
                    hovered_row = Some((i, pos.y > response.rect.center().y));
                }
                if let Some(payload) = response.dnd_release_payload::<DragEntry>() {
                    let below = response
                        .interact_pointer_pos()
                        .or(response.hover_pos())
                        .is_some_and(|p| p.y > response.rect.center().y);
                    released = Some((*payload, if below { i + 1 } else { i }));
                }
                response.context_menu(|ui| {
                    menu_open = true;
                    clicked = Some(entry.id);
                    if context_menu(ui, scene, player, playlist, entry.id, i, track).is_some() {
                        edit_tags = Some(track.id);
                    }
                });
            });
        });

    view_state.rows_built += built;
    if let Some(entry) = clicked.or(dragged) {
        view_state.selection.insert(player, entry);
        view_state.active_player = Some(player);
    }
    if let Some(entry) = cue_follow {
        scene.ctl.send(Command::CueEntry(player, entry));
    }
    if edit_tags.is_some() {
        view_state.edit_tags = edit_tags;
    }
    // Drop target for entries dragged inside the app.
    let pointer_in = ui
        .ctx()
        .pointer_hover_pos()
        .is_some_and(|p| area.contains(p));
    if egui::DragAndDrop::has_payload_of_type::<DragEntry>(ui.ctx()) {
        match hovered_row {
            Some((i, below)) => {
                view_state.drop = Some(DropTarget {
                    playlist,
                    index: if below { i + 1 } else { i },
                });
            }
            None if pointer_in => {
                view_state.drop = Some(DropTarget {
                    playlist,
                    index: entries.len(),
                });
            }
            None => {
                if view_state.drop.is_some_and(|d| d.playlist == playlist) {
                    view_state.drop = None;
                }
            }
        }
    }
    if let Some((payload, index)) = released {
        scene.ctl.send(Command::MoveEntry {
            entry: payload.entry,
            to: playlist,
            index,
        });
        view_state.drop = None;
    } else if pointer_in
        && ui.input(|i| i.pointer.any_released())
        && let Some(payload) = egui::DragAndDrop::payload::<DragEntry>(ui.ctx())
    {
        // Released below the last row.
        scene.ctl.send(Command::MoveEntry {
            entry: payload.entry,
            to: playlist,
            index: entries.len(),
        });
        view_state.drop = None;
    }
    // OS file drops land at the hovered row, or at the end.
    if pointer_in {
        view_state.file_drop = Some(DropTarget {
            playlist,
            index: pointer_row
                .map(|(i, below)| if below { i + 1 } else { i })
                .unwrap_or(entries.len()),
        });
    }
    resize_handles(ui, view_state, player, &columns, &px, area);
    // The operator is using the table: scrolling it (wheel or scroll bar),
    // pressing in it, dragging an entry (for as long as the drag lasts), or
    // with a row menu open. Following waits (feedback spec F18).
    let entry_drag = egui::DragAndDrop::has_payload_of_type::<DragEntry>(ui.ctx());
    if wheel_over_table || pressed_in_table || menu_open || dragged.is_some() || entry_drag {
        view_state.table_touched.insert(player, scene.time);
    }
}

/// The grab zones on the column edges (feedback 2 spec O16): they start a
/// drag, which `live_widths` follows from the next frame on, and draw the
/// separator lines. They sit over the rows, as the table's own did.
fn resize_handles(
    ui: &mut Ui,
    view_state: &mut ViewState,
    player: PlayerId,
    columns: &[TableColumn],
    px: &[f32],
    area: Rect,
) {
    let grab = ui.style().interaction.resize_grab_radius_side;
    let mut x = area.left();
    for (edge, w) in px.iter().take(columns.len().saturating_sub(1)).enumerate() {
        x += w;
        let rect = Rect::from_min_max(pos2(x - grab, area.top()), pos2(x + grab, area.bottom()));
        let id = ui.id().with(("column-edge", player.0, edge));
        let response = ui.interact(rect, id, Sense::drag());
        let dragging = view_state
            .live_resize
            .as_ref()
            .is_some_and(|l| l.player == player && l.edge == edge && l.released.is_none());
        if response.drag_started() {
            let start_x = ui
                .input(|i| i.pointer.press_origin())
                .map_or(x, |origin| origin.x);
            view_state.live_resize = Some(LiveResize {
                player,
                edge,
                columns: columns.to_vec(),
                width: px.iter().sum(),
                start_px: px.to_vec(),
                start_x,
                px: px.to_vec(),
                released: None,
            });
        }
        let hot = (response.hovered() && !ui.input(|i| i.pointer.any_down())) || dragging;
        if hot {
            ui.set_cursor_icon(egui::CursorIcon::ResizeColumn);
        }
        let visuals = ui.visuals();
        let stroke = if dragging {
            visuals.widgets.active.bg_stroke
        } else if hot {
            visuals.widgets.hovered.bg_stroke
        } else {
            visuals.widgets.noninteractive.bg_stroke
        };
        ui.painter()
            .line_segment([pos2(x, area.top()), pos2(x, area.bottom())], stroke);
    }
}
```

`crates/fp-app/locales/en-US/main.ftl`:

```diff
--- a/crates/fp-app/locales/en-US/main.ftl
+++ b/crates/fp-app/locales/en-US/main.ftl
@@ -50,6 +50,11 @@ col-number = #
 col-title = Title
 col-artist = Artist
 col-duration = Dur.
+col-album = Album
+col-date = Date
+col-genre = Genre
+col-intro = Intro
+col-file_name = File name
 footer-add = Add
 footer-count = { $count ->
     [one] 1 track
```

`crates/fp-app/locales/es-ES/main.ftl`:

```diff
--- a/crates/fp-app/locales/es-ES/main.ftl
+++ b/crates/fp-app/locales/es-ES/main.ftl
@@ -50,6 +50,11 @@ col-number = #
 col-title = Título
 col-artist = Artista
 col-duration = Dur.
+col-album = Álbum
+col-date = Fecha
+col-genre = Género
+col-intro = Intro
+col-file_name = Archivo
 footer-add = Añadir
 footer-count = { $count ->
     [one] 1 pista
```

- [ ] **Step 6: Run the tests to see them pass**

Run: `cargo test -p fp-app --test table_layout` — expected: 17 passed. Run: `cargo test -p fp-app --test table_columns_ui` — expected: 15 passed. Run: `cargo test -p fp-app --test main_screen --test view --test table_follow --test table_icons --test table_start` — expected: pass.

Check that the live test discriminates: in `live_widths` skip the `live.px = ...` assignment, and `dragging_an_edge_recomputes_the_other_columns_every_frame` fails.

- [ ] **Step 7: Gate and commit**

```bash
export PATH=$HOME/.cargo/bin:$PATH; cargo fmt --all --check && cargo clippy --workspace --all-targets -q -- -D warnings && cargo test --workspace -q && git add crates && git commit -m "feat(ui): the table draws the configured columns and resizes them live (O16, O24)"
```

### Task 7: Reordering, showing and hiding the columns (UI)

**Files:**
- Create: `crates/fp-app/src/ui/settings/columns.rs`
- Modify: `crates/fp-model/src/columns.rs`, `crates/fp-model/src/lib.rs`, `crates/fp-app/src/ui/app.rs`, `ui/table.rs`, `ui/settings.rs`, both `main.ftl`
- Test: `crates/fp-app/tests/table_header_ui.rs` (create), `crates/fp-model/tests/table_columns.rs`

**Interfaces:**
- Consumes: `with_column_shown`, `move_column`, `column_rows`, `default_columns` (Task 1); `table_layout` and the table's header (Task 6).
- Produces: `fp_model::move_column_before(&[TableColumn], from: usize, slot: usize) -> Vec<TableColumn>` (a header dropped before the column now at `slot`; `slot == len` is the end); `Scene::set_table_columns(&self, Vec<TableColumn>)` (sends one `UpdateConfig` unless the list is already in use); `app::DragColumn { index }` (the drag payload of a header); messages `column-name-<name>` (the menu and Settings names), `settings-columns`, `settings-hint-columns`, `settings-column-up`, `settings-column-down` (`{ $column }`), `settings-columns-default`, `settings-column-required`.

- [ ] **Step 1: Write the failing tests**

`crates/fp-model/tests/table_columns.rs`:

```diff
--- a/crates/fp-model/tests/table_columns.rs
+++ b/crates/fp-model/tests/table_columns.rs
@@ -3,8 +3,8 @@
 
 use fp_model::TableColumn::{Album, Artist, Date, Duration, FileName, Genre, Intro, Number, Title};
 use fp_model::{
-    Config, TableColumn, column_rows, default_columns, move_column, normalize_columns,
-    with_column_shown,
+    Config, TableColumn, column_rows, default_columns, move_column, move_column_before,
+    normalize_columns, with_column_shown,
 };
 
 #[test]
@@ -152,11 +152,38 @@ fn the_list_is_written_with_stable_names() {
 }
 
 #[test]
-fn a_column_is_found_by_the_name_it_has_in_files() {
-    for column in TableColumn::ALL {
-        assert_eq!(TableColumn::from_name(column.name()), Some(column));
-    }
-    assert_eq!(TableColumn::from_name("file_name"), Some(FileName));
-    assert_eq!(TableColumn::from_name("file-name"), None);
-    assert_eq!(TableColumn::from_name("bpm"), None);
+fn a_header_dropped_before_a_slot_lands_just_before_that_column() {
+    let list = vec![Number, Title, Artist, Duration];
+    assert_eq!(
+        move_column_before(&list, 0, 3),
+        vec![Title, Artist, Number, Duration]
+    );
+    assert_eq!(
+        move_column_before(&list, 3, 0),
+        vec![Duration, Number, Title, Artist]
+    );
+    assert_eq!(
+        move_column_before(&list, 3, 1),
+        vec![Number, Duration, Title, Artist]
+    );
+}
+
+#[test]
+fn a_header_dropped_at_the_end_goes_last() {
+    let list = vec![Number, Title, Artist, Duration];
+    assert_eq!(
+        move_column_before(&list, 0, 4),
+        vec![Title, Artist, Duration, Number]
+    );
+    assert_eq!(
+        move_column_before(&list, 0, 99),
+        vec![Title, Artist, Duration, Number]
+    );
+}
+
+#[test]
+fn a_header_dropped_beside_itself_changes_nothing() {
+    let list = vec![Number, Title, Artist, Duration];
+    assert_eq!(move_column_before(&list, 2, 2), list);
+    assert_eq!(move_column_before(&list, 2, 3), list);
 }
```

`crates/fp-app/tests/table_header_ui.rs` (new file):

```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Feedback 2 spec O24: reordering, showing and hiding the table columns
//! from the table header and from Settings.

mod support;

use egui::accesskit::Role;
use egui::{Event, Modifiers, PointerButton, Pos2, pos2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use fp_app::ui::app::AppUi;
use fp_model::TableColumn::{Album, Artist, Date, Duration, Genre, Number, Title};
use fp_model::{Command, TableColumn};
use support::{Fake, harness, state};

fn press(h: &mut Harness<'_, AppUi>, at: Pos2, pressed: bool) {
    h.event(Event::PointerButton {
        pos: at,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    });
}

/// Presses at `from`, moves to `to` in steps and releases there.
fn drag(h: &mut Harness<'_, AppUi>, from: Pos2, to: Pos2) {
    h.event(Event::PointerMoved(from));
    h.run_steps(1);
    press(h, from, true);
    h.run_steps(1);
    for step in 1..=10 {
        let t = step as f32 / 10.0;
        h.event(Event::PointerMoved(pos2(
            from.x + (to.x - from.x) * t,
            from.y + (to.y - from.y) * t,
        )));
        h.run_steps(1);
    }
    press(h, to, false);
    h.run_steps(3);
}

fn centre(h: &Harness<'_, AppUi>, label: &str) -> Pos2 {
    h.get_by_label(label).rect().center()
}

/// A point in the left part of the header cell labelled `label`.
fn left_of(h: &Harness<'_, AppUi>, label: &str) -> Pos2 {
    let r = h.get_by_label(label).rect();
    pos2(r.left() - 4.0, r.center().y)
}

/// A point in the right part of the cell whose header is `label` (the
/// right-aligned time columns have their text there; any other cell is
/// reached from the label's right).
fn right_of(h: &Harness<'_, AppUi>, label: &str, cell_width: f32) -> Pos2 {
    let r = h.get_by_label(label).rect();
    pos2(r.left() - 8.0 + cell_width - 4.0, r.center().y)
}

fn column_updates(fake: &Fake) -> Vec<Vec<TableColumn>> {
    fake.take_sent()
        .into_iter()
        .filter_map(|c| match c {
            Command::UpdateConfig(config) => Some(config.ui.table_columns),
            _ => None,
        })
        .collect()
}

fn header_order(h: &Harness<'_, AppUi>, labels: &[&str]) -> Vec<String> {
    let mut found: Vec<(f32, String)> = labels
        .iter()
        .filter_map(|l| {
            h.query_by_label(l)
                .map(|n| (n.rect().left(), (*l).to_owned()))
        })
        .collect();
    found.sort_by(|a, b| a.0.total_cmp(&b.0));
    found.into_iter().map(|(_, l)| l).collect()
}

const HEADERS: [&str; 4] = ["#", "TITLE", "ARTIST", "DUR."];

// --- drag a header

#[test]
fn dragging_a_header_onto_another_moves_the_column_there() {
    let (mut h, fake) = harness(state(1, 3));
    fake.take_sent();
    let from = centre(&h, "ARTIST");
    let to = left_of(&h, "#");
    drag(&mut h, from, to);
    assert_eq!(
        column_updates(&fake),
        vec![vec![Artist, Number, Title, Duration]]
    );
    assert_eq!(header_order(&h, &HEADERS), ["ARTIST", "#", "TITLE", "DUR."]);
}

#[test]
fn a_header_dropped_on_the_right_half_goes_after_that_column() {
    let (mut h, fake) = harness(state(1, 3));
    fake.take_sent();
    let from = centre(&h, "#");
    let width = h.state().column_widths(fake.player(0)).unwrap()[1];
    let to = right_of(&h, "TITLE", width);
    drag(&mut h, from, to);
    assert_eq!(
        column_updates(&fake),
        vec![vec![Title, Number, Artist, Duration]]
    );
}

#[test]
fn a_required_column_can_be_moved_too() {
    let (mut h, fake) = harness(state(1, 3));
    fake.take_sent();
    let from = centre(&h, "TITLE");
    let to = left_of(&h, "#");
    drag(&mut h, from, to);
    assert_eq!(
        column_updates(&fake),
        vec![vec![Title, Number, Artist, Duration]]
    );
}

#[test]
fn dropping_a_header_on_itself_or_outside_the_header_changes_nothing() {
    let (mut h, fake) = harness(state(1, 3));
    fake.take_sent();
    let from = centre(&h, "ARTIST");
    drag(&mut h, from, pos2(from.x + 2.0, from.y));
    let row = centre(&h, "Song 2");
    let artist = centre(&h, "ARTIST");
    drag(&mut h, artist, row);
    let sent = fake.take_sent();
    assert!(
        !sent.iter().any(|c| matches!(c, Command::UpdateConfig(_))),
        "{sent:?}"
    );
    assert!(
        !sent.iter().any(|c| matches!(c, Command::MoveEntry { .. })),
        "a column dropped on the rows moves no entry"
    );
    assert_eq!(header_order(&h, &HEADERS), HEADERS);
}

#[test]
fn dragging_from_a_column_edge_resizes_and_never_reorders() {
    let (mut h, fake) = harness(state(1, 3));
    fake.take_sent();
    let r = h.get_by_label("ARTIST").rect();
    let edge = pos2(r.left() - 8.0, r.center().y);
    drag(&mut h, edge, pos2(edge.x + 60.0, edge.y));
    assert!(column_updates(&fake).is_empty(), "no column move");
    assert_eq!(header_order(&h, &HEADERS), HEADERS);
}

// --- the header menu

fn open_menu(h: &mut Harness<'_, AppUi>) {
    h.get_by_label("TITLE").click_secondary();
    h.run_steps(2);
}

#[test]
fn the_header_menu_offers_the_optional_columns_only() {
    let (mut h, _) = harness(state(1, 3));
    open_menu(&mut h);
    for name in [
        "Number (#)",
        "Artist",
        "Album",
        "Date",
        "Genre",
        "Intro",
        "File name",
    ] {
        assert!(
            h.query_by_role_and_label(Role::CheckBox, name).is_some(),
            "{name}"
        );
    }
    assert!(h.query_by_role_and_label(Role::CheckBox, "Title").is_none());
    assert!(
        h.query_by_role_and_label(Role::CheckBox, "Duration")
            .is_none()
    );
}

#[test]
fn the_header_menu_shows_a_hidden_column() {
    let (mut h, fake) = harness(state(1, 3));
    fake.take_sent();
    open_menu(&mut h);
    h.get_by_role_and_label(Role::CheckBox, "Album").click();
    h.run_steps(3);
    assert_eq!(
        column_updates(&fake),
        vec![vec![Number, Title, Artist, Duration, Album]]
    );
    assert!(h.query_by_label("ALBUM").is_some());
}

#[test]
fn the_header_menu_hides_a_shown_column() {
    let (mut h, fake) = harness(state(1, 3));
    fake.take_sent();
    open_menu(&mut h);
    h.get_by_role_and_label(Role::CheckBox, "Artist").click();
    h.run_steps(3);
    assert_eq!(column_updates(&fake), vec![vec![Number, Title, Duration]]);
    assert!(h.query_by_label("ARTIST").is_none());
}

#[test]
fn the_header_menu_marks_the_shown_columns() {
    let (mut h, _) = harness(state(1, 3));
    open_menu(&mut h);
    assert_eq!(
        h.get_by_role_and_label(Role::CheckBox, "Artist")
            .accesskit_node()
            .toggled(),
        Some(egui::accesskit::Toggled::True)
    );
    assert_eq!(
        h.get_by_role_and_label(Role::CheckBox, "Album")
            .accesskit_node()
            .toggled(),
        Some(egui::accesskit::Toggled::False)
    );
}

#[test]
fn a_hidden_column_comes_back_at_the_end_and_a_shown_one_leaves() {
    let mut s = state(1, 3);
    s.config.ui.table_columns = vec![Duration, Title, Date];
    let (mut h, fake) = harness(s);
    fake.take_sent();
    open_menu(&mut h);
    h.get_by_role_and_label(Role::CheckBox, "Genre").click();
    h.run_steps(3);
    assert_eq!(
        column_updates(&fake),
        vec![vec![Duration, Title, Date, Genre]]
    );
}

// --- Settings

fn open_settings(h: &mut Harness<'_, AppUi>) {
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Playlists").click();
    h.run_steps(3);
}

#[test]
fn settings_lists_every_column_and_marks_the_shown_ones() {
    let (mut h, _) = harness(state(1, 3));
    open_settings(&mut h);
    for name in [
        "Number (#)",
        "Title",
        "Artist",
        "Album",
        "Date",
        "Genre",
        "Duration",
        "Intro",
        "File name",
    ] {
        assert!(
            h.query_by_role_and_label(Role::CheckBox, name).is_some(),
            "{name}"
        );
    }
    assert_eq!(
        h.get_by_role_and_label(Role::CheckBox, "Artist")
            .accesskit_node()
            .toggled(),
        Some(egui::accesskit::Toggled::True)
    );
    assert_eq!(
        h.get_by_role_and_label(Role::CheckBox, "Album")
            .accesskit_node()
            .toggled(),
        Some(egui::accesskit::Toggled::False)
    );
}

#[test]
fn the_required_columns_cannot_be_unticked() {
    let (mut h, fake) = harness(state(1, 3));
    open_settings(&mut h);
    assert!(
        h.get_by_role_and_label(Role::CheckBox, "Title")
            .accesskit_node()
            .is_disabled()
    );
    assert!(
        h.get_by_role_and_label(Role::CheckBox, "Duration")
            .accesskit_node()
            .is_disabled()
    );
    assert!(
        !h.get_by_role_and_label(Role::CheckBox, "Artist")
            .accesskit_node()
            .is_disabled()
    );
    fake.take_sent();
    h.get_by_role_and_label(Role::CheckBox, "Title").click();
    h.run_steps(2);
    assert!(column_updates(&fake).is_empty());
}

#[test]
fn ticking_a_column_in_settings_shows_it() {
    let (mut h, fake) = harness(state(1, 3));
    open_settings(&mut h);
    fake.take_sent();
    h.get_by_role_and_label(Role::CheckBox, "Genre").click();
    h.run_steps(3);
    assert_eq!(
        column_updates(&fake),
        vec![vec![Number, Title, Artist, Duration, Genre]]
    );
}

#[test]
fn unticking_an_optional_column_hides_it() {
    let (mut h, fake) = harness(state(1, 3));
    open_settings(&mut h);
    fake.take_sent();
    h.get_by_role_and_label(Role::CheckBox, "Number (#)")
        .click();
    h.run_steps(3);
    assert_eq!(column_updates(&fake), vec![vec![Title, Artist, Duration]]);
}

#[test]
fn the_arrows_move_a_column_up_and_down() {
    let (mut h, fake) = harness(state(1, 3));
    open_settings(&mut h);
    fake.take_sent();
    h.get_by_label("Move Artist up").click();
    h.run_steps(3);
    assert_eq!(
        column_updates(&fake),
        vec![vec![Number, Artist, Title, Duration]]
    );
    h.get_by_label("Move Number (#) down").click();
    h.run_steps(3);
    assert_eq!(
        column_updates(&fake),
        vec![vec![Artist, Number, Title, Duration]]
    );
}

#[test]
fn the_arrows_are_off_at_the_ends_and_for_hidden_columns() {
    let (mut h, _) = harness(state(1, 3));
    open_settings(&mut h);
    let off =
        |h: &Harness<'_, AppUi>, label: &str| h.get_by_label(label).accesskit_node().is_disabled();
    assert!(off(&h, "Move Number (#) up"));
    assert!(off(&h, "Move Duration down"));
    assert!(!off(&h, "Move Duration up"));
    assert!(!off(&h, "Move Number (#) down"));
    assert!(off(&h, "Move Album up") && off(&h, "Move Album down"));
}

#[test]
fn a_required_column_can_move_in_settings_too() {
    let (mut h, fake) = harness(state(1, 3));
    open_settings(&mut h);
    fake.take_sent();
    h.get_by_label("Move Duration up").click();
    h.run_steps(3);
    assert_eq!(
        column_updates(&fake),
        vec![vec![Number, Title, Duration, Artist]]
    );
}

#[test]
fn default_columns_restores_the_list_and_is_off_when_it_is_the_default() {
    let mut s = state(1, 3);
    s.config.ui.table_columns = vec![Duration, Title, Album];
    let (mut h, fake) = harness(s);
    open_settings(&mut h);
    fake.take_sent();
    h.get_by_label("Default columns").click();
    h.run_steps(3);
    assert_eq!(
        column_updates(&fake),
        vec![vec![Number, Title, Artist, Duration]]
    );
    assert!(
        h.get_by_label("Default columns")
            .accesskit_node()
            .is_disabled()
    );
}

#[test]
fn the_table_follows_a_change_made_in_settings() {
    let (mut h, _) = harness(state(1, 3));
    open_settings(&mut h);
    h.get_by_role_and_label(Role::CheckBox, "Date").click();
    h.run_steps(3);
    h.get_by_label("Close").click();
    h.run_steps(3);
    assert!(h.query_by_label("DATE").is_some());
}
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `cargo test -p fp-model --test table_columns` — expected: does not compile (`move_column_before`). Run: `cargo test -p fp-app --test table_header_ui` — expected: 17 of the 19 fail (no drag moves a column, the header menu does not open, Settings has no column list); the two that pass only assert that nothing happens.

- [ ] **Step 3: Implement**

`crates/fp-model/src/columns.rs`:

```diff
--- a/crates/fp-model/src/columns.rs
+++ b/crates/fp-model/src/columns.rs
@@ -125,6 +125,18 @@ pub fn move_column(list: &[TableColumn], from: usize, to: usize) -> Vec<TableCol
     out
 }
 
+/// `list` with the column at position `from` moved to sit just before the
+/// column now at position `slot` (`slot` equal to the length is the end):
+/// where a header dropped on the left half of a cell, or on the right half
+/// of the one before it, ends up. Dropping a column on either side of
+/// itself changes nothing.
+pub fn move_column_before(list: &[TableColumn], from: usize, slot: usize) -> Vec<TableColumn> {
+    let len = normalize_columns(list).len();
+    let slot = slot.min(len);
+    let to = if from < slot { slot - 1 } else { slot };
+    move_column(list, from, to)
+}
+
 /// The rows of the Settings list: the shown columns in their order (`true`),
 /// then the hidden ones in the order of [`TableColumn::ALL`] (`false`).
 pub fn column_rows(list: &[TableColumn]) -> Vec<(TableColumn, bool)> {
```

`crates/fp-model/src/lib.rs`:

```diff
--- a/crates/fp-model/src/lib.rs
+++ b/crates/fp-model/src/lib.rs
@@ -34,7 +34,8 @@ pub use cartwall::{
     PlayingCart,
 };
 pub use columns::{
-    TableColumn, column_rows, default_columns, move_column, normalize_columns, with_column_shown,
+    TableColumn, column_rows, default_columns, move_column, move_column_before, normalize_columns,
+    with_column_shown,
 };
 pub use command::{
     CartRequest, Command, EngineAction, EngineEvent, SOURCE_END, SourceRequest, TransitionPlan,
```

`crates/fp-app/src/ui/app.rs`:

```diff
--- a/crates/fp-app/src/ui/app.rs
+++ b/crates/fp-app/src/ui/app.rs
@@ -44,6 +44,13 @@ const NOTICE_SECS: f64 = 5.0;
 /// Idle repaint period (the clock).
 const IDLE_REPAINT: Duration = Duration::from_millis(100);
 
+/// The payload of a column header being dragged (feedback 2 spec O24): its
+/// position in the list of columns.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub(crate) struct DragColumn {
+    pub index: usize,
+}
+
 /// The payload of an entry being dragged.
 #[derive(Debug, Clone, Copy, PartialEq, Eq)]
 pub(crate) struct DragEntry {
@@ -153,6 +160,17 @@ impl Scene<'_> {
         Some(self.i18n.tr_args(key, &[("path", path.into())]))
     }
 
+    /// Sends the new list of table columns (feedback 2 spec O24), unless it
+    /// is the one in use. The model repairs it (`Config::validate`).
+    pub fn set_table_columns(&self, columns: Vec<fp_model::TableColumn>) {
+        let mut config = self.state.config.clone();
+        config.ui.table_columns = columns;
+        let _ = config.validate();
+        if config.ui.table_columns != self.state.config.ui.table_columns {
+            self.ctl.send(Command::UpdateConfig(Box::new(config)));
+        }
+    }
+
     /// Opens the native file dialog without blocking the interface.
     pub fn pick_files(&self, playlist: PlaylistId, index: usize) {
         let tx = self.picks.clone();
```

`crates/fp-app/src/ui/table.rs`:

```diff
--- a/crates/fp-app/src/ui/table.rs
+++ b/crates/fp-app/src/ui/table.rs
@@ -6,7 +6,7 @@ use egui_extras::{Column, TableBuilder};
 use egui_phosphor::regular as icon;
 use fp_model::{ColumnWidths, Command, EntryId, PlayerId, PlaylistId, TableColumn, Transport};
 
-use super::app::{DragEntry, DropTarget, FollowScroll, Scene, ViewState};
+use super::app::{DragColumn, DragEntry, DropTarget, FollowScroll, Scene, ViewState};
 use super::format;
 use super::glyphs::{self, TransportAction};
 use super::table_layout;
@@ -206,13 +206,38 @@ pub(crate) fn track_table(
         }
     }
     let mut menu_open = false;
+    // O24: a header being dragged over another, and the move it ended in.
+    let mut column_slot: Option<usize> = None;
+    let mut column_move: Option<(usize, usize)> = None;
     builder
         .header(HEADER_HEIGHT, |mut header| {
-            for column in &columns {
-                header.col(|ui| {
+            for (index, column) in columns.iter().enumerate() {
+                let (_, response) = header.col(|ui| {
                     let right = matches!(column, TableColumn::Duration | TableColumn::Intro);
                     header_label(ui, &t.tr(&format!("col-{}", column.name())), right);
                 });
+                if response.drag_started() {
+                    response.dnd_set_drag_payload(DragColumn { index });
+                }
+                let right_half = |pos: egui::Pos2| pos.x > response.rect.center().x;
+                if response.dnd_hover_payload::<DragColumn>().is_some()
+                    && let Some(pos) = response.hover_pos()
+                {
+                    column_slot = Some(if right_half(pos) { index + 1 } else { index });
+                }
+                if let Some(payload) = response.dnd_release_payload::<DragColumn>() {
+                    let at = response.interact_pointer_pos().or(response.hover_pos());
+                    let slot = if at.is_some_and(right_half) {
+                        index + 1
+                    } else {
+                        index
+                    };
+                    column_move = Some((payload.index, slot));
+                }
+                response.context_menu(|ui| {
+                    menu_open = true;
+                    header_menu(ui, scene, &columns);
+                });
             }
         })
         .body(|body| {
@@ -556,6 +581,21 @@ pub(crate) fn track_table(
         });
     }
     resize_handles(ui, view_state, player, &columns, &px, area);
+    if let Some((from, slot)) = column_move {
+        scene.set_table_columns(fp_model::move_column_before(&columns, from, slot));
+    }
+    if egui::DragAndDrop::has_payload_of_type::<DragColumn>(ui.ctx()) {
+        ui.set_cursor_icon(egui::CursorIcon::Grabbing);
+        // Where the dragged header would land: a line on that column edge.
+        if let Some(slot) = column_slot {
+            let x = area.left() + px.iter().take(slot).sum::<f32>();
+            ui.painter().rect_filled(
+                Rect::from_min_size(pos2(x - 1.0, area.top()), vec2(2.0, HEADER_HEIGHT)),
+                0.0,
+                theme::ACCENT,
+            );
+        }
+    }
     // The operator is using the table: scrolling it (wheel or scroll bar),
     // pressing in it, dragging an entry (for as long as the drag lasts), or
     // with a row menu open. Following waits (feedback spec F18).
@@ -619,6 +659,22 @@ fn resize_handles(
     }
 }
 
+/// The menu of the table header: shows and hides the optional columns
+/// (feedback 2 spec O24). Title and Duration are always shown, so they are
+/// not offered.
+fn header_menu(ui: &mut Ui, scene: &Scene<'_>, columns: &[TableColumn]) {
+    let t = scene.i18n;
+    ui.set_min_width(200.0);
+    for column in TableColumn::ALL.into_iter().filter(|c| !c.is_required()) {
+        let mut shown = columns.contains(&column);
+        let label = t.tr(&format!("column-name-{}", column.name()));
+        if ui.checkbox(&mut shown, label).changed() {
+            scene.set_table_columns(fp_model::with_column_shown(columns, column, shown));
+            ui.close();
+        }
+    }
+}
+
 fn context_menu(
     ui: &mut Ui,
     scene: &Scene<'_>,
```

`crates/fp-app/src/ui/settings.rs`:

```diff
--- a/crates/fp-app/src/ui/settings.rs
+++ b/crates/fp-app/src/ui/settings.rs
@@ -18,6 +18,7 @@ use fp_model::{
 use super::app::Scene;
 
 mod carts;
+mod columns;
 mod keys;
 mod meters;
 mod midi;
@@ -1747,6 +1748,9 @@ fn playlists(ui: &mut Ui, scene: &Scene<'_>, st: &mut SettingsState) {
             }
         });
     }
+    // O24: the columns of every player's table.
+    ui.add_space(16.0);
+    columns::section(ui, scene);
 }
 
 fn pick_folder(scene: &Scene<'_>) -> Option<Receiver<Option<PathBuf>>> {
```

`crates/fp-app/src/ui/settings/columns.rs` (new file):

```rust
//! Settings → Playlists → Table columns (feedback 2 spec O24): which
//! columns the track tables show and in which order, for every player.

use egui::{Align, Layout, RichText, Ui, vec2};
use egui_phosphor::regular as icon;
use fp_model::{column_rows, default_columns, move_column, normalize_columns, with_column_shown};

use super::row;
use crate::i18n::Arg;
use crate::ui::app::Scene;
use crate::ui::theme;
use crate::ui::widgets::{self, TileStyle, font};

/// A small arrow button; `label` is its accessible name and tooltip.
fn arrow(ui: &mut Ui, label: &str, glyph: &str, enabled: bool) -> bool {
    widgets::tile(
        ui,
        vec2(28.0, 24.0),
        label,
        enabled,
        TileStyle::plain(),
        |p, r, c| widgets::glyph(p, r, glyph, 14.0, c, false),
    )
    .clicked()
}

pub(super) fn section(ui: &mut Ui, scene: &Scene<'_>) {
    let t = scene.i18n;
    let list = normalize_columns(&scene.state.config.ui.table_columns);
    let rows = column_rows(&list);
    row(
        ui,
        &t.tr("settings-columns"),
        Some(&t.tr("settings-hint-columns")),
        |ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 2.0);
                for (column, shown) in rows {
                    let name = t.tr(&format!("column-name-{}", column.name()));
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = vec2(6.0, 0.0);
                        let mut on = shown;
                        let text = RichText::new(&name).font(font(12.0)).color(theme::TEXT);
                        let changed = ui
                            .allocate_ui_with_layout(
                                vec2(220.0, 24.0),
                                Layout::left_to_right(Align::Center),
                                |ui| {
                                    let check = ui.add_enabled(
                                        !column.is_required(),
                                        egui::Checkbox::new(&mut on, text),
                                    );
                                    if column.is_required() {
                                        check
                                            .on_disabled_hover_text(
                                                t.tr("settings-column-required"),
                                            )
                                            .changed()
                                    } else {
                                        check.changed()
                                    }
                                },
                            )
                            .inner;
                        if changed {
                            scene.set_table_columns(with_column_shown(&list, column, on));
                        }
                        // Only the shown columns have a place to move to.
                        let at = list.iter().position(|c| *c == column);
                        let args = [("column", Arg::Text(name.clone()))];
                        let up = t.tr_args("settings-column-up", &args);
                        let down = t.tr_args("settings-column-down", &args);
                        if arrow(ui, &up, icon::CARET_UP, at.is_some_and(|i| i > 0))
                            && let Some(i) = at
                        {
                            scene.set_table_columns(move_column(&list, i, i - 1));
                        }
                        if arrow(
                            ui,
                            &down,
                            icon::CARET_DOWN,
                            at.is_some_and(|i| i + 1 < list.len()),
                        ) && let Some(i) = at
                        {
                            scene.set_table_columns(move_column(&list, i, i + 1));
                        }
                    });
                }
                ui.add_space(6.0);
                let label = t.tr("settings-columns-default");
                let width = ui
                    .painter()
                    .layout_no_wrap(label.clone(), font(12.0), theme::TEXT)
                    .size()
                    .x
                    + 20.0;
                let is_default = list == default_columns();
                if widgets::tile(
                    ui,
                    vec2(width, 24.0),
                    &label,
                    !is_default,
                    TileStyle::plain(),
                    |p, r, c| {
                        p.text(
                            r.center(),
                            egui::Align2::CENTER_CENTER,
                            &label,
                            font(12.0),
                            c,
                        );
                    },
                )
                .clicked()
                {
                    scene.set_table_columns(default_columns());
                }
            });
        },
    );
}
```

`crates/fp-app/locales/en-US/main.ftl`:

```diff
--- a/crates/fp-app/locales/en-US/main.ftl
+++ b/crates/fp-app/locales/en-US/main.ftl
@@ -506,3 +506,19 @@ tip-reset-played = Clear the played marks of this playlist
 reset-played-question = Clear the played mark of every track in this playlist?
 reset-played-cancel = Cancel
 reset-played-confirm = Reset played
+
+column-name-number = Number (#)
+column-name-title = Title
+column-name-artist = Artist
+column-name-album = Album
+column-name-date = Date
+column-name-genre = Genre
+column-name-duration = Duration
+column-name-intro = Intro
+column-name-file_name = File name
+settings-columns = Table columns
+settings-hint-columns = The columns of every player's table. You can also drag a header in a table, or right-click one.
+settings-column-up = Move { $column } up
+settings-column-down = Move { $column } down
+settings-columns-default = Default columns
+settings-column-required = Always shown
```

`crates/fp-app/locales/es-ES/main.ftl`:

```diff
--- a/crates/fp-app/locales/es-ES/main.ftl
+++ b/crates/fp-app/locales/es-ES/main.ftl
@@ -506,3 +506,19 @@ tip-reset-played = Quitar las marcas de reproducida de esta playlist
 reset-played-question = ¿Quitar la marca de reproducida a todas las pistas de esta playlist?
 reset-played-cancel = Cancelar
 reset-played-confirm = Reiniciar
+
+column-name-number = Número (#)
+column-name-title = Título
+column-name-artist = Artista
+column-name-album = Álbum
+column-name-date = Fecha
+column-name-genre = Género
+column-name-duration = Duración
+column-name-intro = Intro
+column-name-file_name = Nombre de archivo
+settings-columns = Columnas de la tabla
+settings-hint-columns = Las columnas de la tabla de cada reproductor. También puedes arrastrar una cabecera en una tabla, o hacer clic derecho en ella.
+settings-column-up = Subir { $column }
+settings-column-down = Bajar { $column }
+settings-columns-default = Columnas por defecto
+settings-column-required = Siempre visible
```

- [ ] **Step 4: Run the tests to see them pass**

Run: `cargo test -p fp-model --test table_columns` — expected: 18 passed. Run: `cargo test -p fp-app --test table_header_ui` — expected: 19 passed. Run: `cargo test -p fp-app --test settings --test settings_layout --test settings_restore --test i18n` — expected: pass (the Playlists section got longer; the layout test checks it still fits).

- [ ] **Step 5: Gate and commit**

```bash
export PATH=$HOME/.cargo/bin:$PATH; cargo fmt --all --check && cargo clippy --workspace --all-targets -q -- -D warnings && cargo test --workspace -q && git add crates && git commit -m "feat(ui): reorder, show and hide the table columns (O24)"
```
### Task 8: Documentation

**Files:**
- Modify: `docs/user/playlists.md` (the table, the footer, the menu), `docs/user/settings.md` (Playlists), `docs/user/data-and-backups.md` (session file)
- Modify: `docs/technical/ui.md` (module table, "Track table layout and follow"), `docs/technical/persistence.md` (session `columns`, the `ui` table)
- Modify: `docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md` (status and "As built" under §9)
- Modify: `docs/superpowers/plans/2026-10-01-operator-feedback-2-roadmap.md` (row 8 status `done`)
- Modify: `README.md` (the features list)
- `CLAUDE.md`: only if a command or layout line changed (none does: the new files sit in existing crates; leave it).

**Interfaces:** none (prose). Every claim must match the code as built: re-read `fp-model/src/columns.rs`, `player.rs` (`ColumnWidths`), `reset_played.rs`, `fp-app/src/ui/table.rs`, `table_layout.rs`, `reset_played.rs`, `settings/columns.rs` and `player.rs` (`footer`, `scroll_to_next_once`) before writing.

- [ ] **Step 1: Update the user guide**

`docs/user/playlists.md`:

- Replace the table under "The track table" and the paragraph after it with:

```markdown
| Column | Content |
|---|---|
| `#` | Position, zero-padded; an icon replaces it for the current and next tracks |
| Title | From the tags, or the file name (`Artist - Title.mp3` is split). The repeat and stop-after icons of a track sit before it |
| Artist | From the tags; "Unknown artist" when there is none |
| Album | From the tags |
| Date | The recording date as the file stores it (`2019`, `2019-05` or `2019-05-14`, with a time if there is one) |
| Genre | From the tags |
| Dur. | Playing length, from cue-in to cue-out (the whole file with **Use cue-in and cue-out** off) |
| Intro | How long the intro lasts, from where the track starts playing to its intro marker; empty when the track has no intro marker |
| File name | The name of the file, with its extension |

A new installation shows `#`, Title, Artist and Dur. The other columns are
optional; see **Choosing the columns** below. A track that lacks a value
shows an empty cell.

The columns fill the table and keep their proportions when the window is
resized; the text columns get the most room. Drag the header separators to
change the proportions: the columns to the right of the separator follow the
pointer on every frame (they share what is left in proportion to their
width), the ones to its left stay, and the widths are saved when you let go.
No column gets narrower than its minimum. The widths are remembered per
player; a column you show later starts with its default width and the others
keep their proportions.
```

- After it, add a section:

```markdown
### Choosing the columns

Title and Dur. are always shown. Every other column can be shown or hidden,
and any column, those two included, can be moved. The list is the same for
every player and playlist, and it is saved in `config.json` as
`ui.table_columns`. Three ways to change it:

- **Settings → Playlists → Table columns:** tick the columns to show; the
  arrows move a shown column up or down (they read left to right in the
  tables). **Default columns** goes back to `#`, Title, Artist and Dur.
- **Right-click a header:** a menu with a checkbox for every optional column.
  A column you show appears at the right end; drag it from there.
- **Drag a header** onto another: drop it on the left half of a header to put
  the column before it, on the right half to put it after it. Dropping
  anywhere else does nothing.

A name in `ui.table_columns` that this version does not know is ignored, and
a missing Title or Dur. is added back.
```

- In "When a player moves on to another track…" keep the paragraph and add before it:

```markdown
When the application opens, each table scrolls so that its player's next
track is in the middle of the table (as near as the ends of the list allow).
That happens once, at start-up, and only when the next track is in the
playlist the table shows.
```

- In the context menu table, change the two flag descriptions: "A repeat icon shows before the title" and "The stop-after icon shows before the title".
- In "## Footer": replace the paragraph with:

```markdown
**+ Add** opens a file dialog, starting in the music folder set in
Settings. **Reset played** (the arrow icon next to it) clears the dimmed
"already played" mark of every track of the playlist, for every player,
after asking "Clear the played mark of every track in this playlist?"
(**Cancel**, Esc or a click outside keep the marks). The track that is on
air keeps its state and is marked when the player leaves it. The button is
dimmed when there is nothing to clear. The footer also shows the number of
tracks, the time left in the playlist and its total length.
```

`docs/user/settings.md`, "## Playlists": add the bullet

```markdown
- **Table columns:** which columns the track tables show and in which order,
  for every player: a checkbox per column (Title and Dur. cannot be turned
  off), up and down arrows for the shown ones, and **Default columns**. See
  [Playlists](playlists.md).
```

`docs/user/data-and-backups.md`: in the `session.json` row replace "column widths" by "column widths (by column)".

- [ ] **Step 2: Update the technical docs**

`docs/technical/ui.md`:

- Module table: change the `ui/table.rs` row to "The track table: virtualised rows, drag and drop, context menu, the configured columns, the header (drag to reorder, menu) and the live column resize"; add rows for `ui/table_layout.rs` ("The pure widths of the table's columns: `column_min`, `fit`, `column_px`, `resize_px`, `fractions_of`; unit-tested"), `ui/reset_played.rs` ("The Reset played question (O22): `show` returns `Some(true)`, `Some(false)` or `None`; the footer button in `player.rs` sets `ViewState::confirm_reset`, `AppUi` draws it below the close guard and sends `Command::ResetPlayed`; Esc and a deleted playlist close it; no shortcut or file drop acts under it") and `ui/settings/columns.rs` ("Settings → Playlists → Table columns: `column_rows`, the checkboxes, the arrows (`move_column`) and Default columns; every change goes through `Scene::set_table_columns`"). Change the `ui/settings.rs` row to mention playlists and their columns.
- Replace the section "Track table layout and follow" with:

```markdown
## Track table layout, columns and follow

**Columns.** The table draws `config.ui.table_columns` (repaired with
`normalize_columns` on every frame, so a list set without `Config::validate`
still has Title and Duration). Each cell is a `match` on `TableColumn` in
`track_table`; the text of the plain columns is `view::cell_text`.

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
`with_column_shown`. The payload type is not `DragEntry`, so a dragged track
never reorders columns nor the other way round. `set_table_columns` sends one
`UpdateConfig`, validated by the reducer.

**Follow.** `player::follow_current` watches each player's current entry
(`ViewState::followed`). A change waits in `follow_pending` until
`scene.time − table_touched ≥ ui.follow_current_grace_secs` (a scroll over the
table, an entry drag, an open row menu or a tab click update `table_touched`);
then it sends `ShowPlaylist` if needed and puts a `FollowScroll { entry, align:
TOP, animated: true }` in `follow_scroll`, which `track_table` turns into
`scroll_to_row` once the playlist is shown. A grace of 0 never follows. At
start-up `player::scroll_to_next_once` does the same once per player
(`ViewState::startup_scrolled`) for `view::start_scroll_target`, the player's
next entry when it is in the playlist the tab shows, with `Align::Center` and
no animation; the scroll area's own clamping keeps it inside the list.
```

- In "Data flow", the sentence "The UI keeps only **view state**: selection, drag target, column widths being dragged, the Settings section." stays true: change "column widths being dragged" to "the column edge being dragged (`LiveResize`)".

`docs/technical/persistence.md`:

- In the `session.json` row, after `columns` add: "(`ColumnWidths`: fractions keyed by column name, for example `{"fractions": {"title": 0.5, "artist": 0.4, "duration": 0.1}}`; the old `[#, Title, Artist, Duration]` array converts, anything else loads as the default layout, unknown names are dropped)".
- In the `ui` table add the row `| `table_columns` | `["number", "title", "artist", "duration"]` | an ordered list of `number`, `title`, `artist`, `album`, `date`, `genre`, `duration`, `intro`, `file_name`; unknown names are dropped when the file is read, duplicates keep their first place, and a missing `title` or `duration` is added back (`Config::validate` warns) |`.

- [ ] **Step 3: Update the spec, roadmap and README**

Spec, header: replace the Status line with `- **Status:** Approved. Plans 1 to 8 are built; each plan's section ends with its "As built" notes.` and, at the end of §9 (after the O24 bullets), add:

```markdown
- **As built.**
  - O16: `egui_extras` 0.36's `TableBuilder` cannot recompute the other columns during a drag (it changes only the dragged column, a frame late, and keeps resizable widths), so the table draws its own resize handles. Every column is `Column::exact(px)` from `table_layout::column_px`; `live_widths` follows the raw pointer on every frame with the pure `resize_px`; one `SetColumnWidths` is sent on release, only if something moved. A double click on an edge no longer auto-sizes.
  - O7: `view::start_scroll_target` (the player's next, when it is in the shown playlist) and `player::scroll_to_next_once` give `FollowScroll { align: Center, animated: false }` once per player, on the first frame its column is drawn. `TableBuilder::min_scrolled_height(0.0)` stops the footer from covering the last rows in a short window.
  - O9: the Title cell draws the repeat icon, the stop-after icon and then the title; the outdated-analysis flag stays at the right.
  - O22: `Command::ResetPlayed(PlaylistId)`; `fp_model::{resettable_entries, can_reset_played}`. The current entry of any player keeps its marks; other playlists, flags, histories and explicit nexts are untouched; the derived next is recomputed with `refresh_next`. The footer button asks first (`reset_played::show`) and is dimmed when nothing can be cleared.
  - O24: `fp_model::TableColumn` (`Number`, `Title`, `Artist`, `Album`, `Date`, `Genre`, `Duration`, `Intro`, `FileName`; file names are snake case), `ui.table_columns` with `normalize_columns` (first duplicate wins, Title first or after `#`, Duration last) in `Config::validate`; `with_column_shown`, `move_column`, `move_column_before` and `column_rows` are the pure edit rules. `ColumnWidths.fractions` is `Option<BTreeMap<TableColumn, f32>>`; the old four-number array converts to `#`, Title, Artist and Duration. `Track::intro_secs` feeds Intro. The list is edited in Settings → Playlists (`settings/columns.rs`), in the header's menu and by dragging a header (`DragColumn`); `Scene::set_table_columns` sends one `UpdateConfig`.
  - Not exposed through the remote API, MIDI or the session file beyond the widths.
```

Roadmap: set row 8's status to `done`. README, features list, after the tag editor bullet:

```markdown
- **Configurable track tables:** choose and order the columns (title, artist, album, date, genre, intro, file name and more) from Settings, the header menu or by dragging a header; columns resize live; tables open at the next track; **Reset played** clears a playlist's played marks.
```

- [ ] **Step 4: Check the docs**

Run: `grep -rn "table_columns\|Reset played\|Choosing the columns\|live_resize\|scroll_to_next_once" docs README.md crates/fp-app/locales | head -60` and confirm every place that should mention them does. `grep -rn "column_px\|table_layout\|TableBuilder::reset" docs/technical` must show only the new text (nothing says the table is reset on a layout change). Run `scripts/check-commits.sh origin/master` after committing.

- [ ] **Step 5: Commit**

```bash
git add docs README.md && git commit -m "docs: describe the track table changes (scroll to next, icons, live resize, reset played, columns)"
```

(Docs change no code, but the commit rule still stands: run the gate first.)

---

## Self-Review

**Spec coverage.**
- O7 (scroll to next, centred, once at start-up): Task 5 (`the_next_entry_is_visible_and_centred`, `it_happens_once_a_later_next_does_not_scroll`, `each_player_scrolls_to_its_own_next`; the ends, the other playlist and the short window in Review Focus line 6).
- O9 (icons before the title): Task 4 (`the_repeat_icon_comes_before_the_title_in_its_row`, `the_stop_icon_...`, `both_icons_keep_their_order_before_the_title`, `a_flagged_title_makes_room_...`, the narrow table, the outdated flag staying at the right; `flagged_entries_show_their_icons` updated).
- O16 (live resize, widths stored on release, "Task 1 checks…"): the decision section above settles the check; Task 6 implements it (`dragging_an_edge_recomputes_the_other_columns_every_frame`, `the_widths_are_stored_once_on_release`, `resize_px` tests, `a_plain_click_on_an_edge_stores_nothing`, `resizing_the_window_does_not_store_column_widths`).
- O22 (button next to "Add tracks", the spec's question, `ResetPlayed` except the current one, derived next recomputed): Task 2 (rules) and Task 3 (button, question, both locales).
- O24, one bullet at a time:
  - the global ordered list `ui.table_columns`, loaded leniently (unknown dropped, required added back): Task 1 (`normalize_columns` tests, the four lenient-loading tests, `validate_repairs_the_list_and_says_so`);
  - required Title and Duration, optional `#`, Artist, Album, Date, Genre, Intro, File name: Task 1 (`title_and_duration_are_the_required_columns`), cells in Task 6 (`the_optional_columns_show_their_headers_and_cells`);
  - all columns reorderable, by dragging a header: Task 7 (`dragging_a_header_onto_another_moves_the_column_there`, `a_required_column_can_be_moved_too`); in Settings with a checkbox list and up and down buttons: Task 7 (`settings_lists_every_column_...`, `the_arrows_move_a_column_up_and_down`, `a_required_column_can_move_in_settings_too`);
  - the header's context menu shows and hides the optional columns: Task 7 (`the_header_menu_*`);
  - widths as fractions keyed by column, the old `[f32; 4]` converted when it maps cleanly, otherwise the default layout: Task 6 (`column_widths.rs`, `a_session_with_the_old_widths_loads_them_keyed`).
- Docs, spec "As built", roadmap row 8, README: Task 8. Both locales inside Tasks 3, 6 and 7 (`tests/i18n.rs`). `Config` field with a default, a range (repair) and lenient loading: Task 1.

**Placeholder scan.** No TBD. Every code block in Tasks 1 to 7 is the text that was compiled: the plan was built by applying its blocks, task by task, to a scratch copy of `88c3cd1`, and each of the seven commits passes `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace`.

**Type consistency.** `TableColumn` and the list rules (Task 1) are what `ColumnWidths` (Task 6), `table_layout`, `cell_text`, the header (Tasks 6 and 7) and Settings (Task 7) use. `Command::ResetPlayed(PlaylistId)` and `can_reset_played(&AppState, PlaylistId)` (Task 2) are what the footer (Task 3) calls. `FollowScroll` (Task 5) is what `follow_current` and `scroll_to_next_once` insert and `track_table` (Task 6 rewrites that function and keeps the block) consumes. `AppUi::column_widths` returns `Option<Vec<f32>>` from Task 6 on. Message keys: `col-<name>` (`TableColumn::name`, with `col-file_name`) in Task 6; `column-name-<name>` and the `settings-*` column keys in Task 7; the Reset played keys in Task 3.

**Review Focus.** Each of the six lines names tests in the task that owns the code.

**Known weak spots to watch in review.**
- The grab zones span the whole table height, over the rows (as the table's own did): a press within five points of an edge never selects a row. The tests drive pointer events through kittest, not a real window; a visual check under Xvfb (CLAUDE.md, "Testing notes") of dragging an edge, dragging a header and the drop line is worth doing once.
- While a drag runs the pointer is read at the start of the frame; a pointer released outside the window ends the drag at the next frame that sees `primary_down` false.
- The Spanish strings are the maintainer's to polish ("Reiniciar" for Reset played, "Nombre de archivo", "Subir" and "Bajar").
- Windows and macOS were not run; nothing in this plan is platform specific, but CI decides.
- `ui.table_columns` is edited as a whole through `UpdateConfig`; two quick header gestures in one frame send two commands, the second built from the same snapshot (the last wins), which is harmless for gestures a person makes one at a time.
