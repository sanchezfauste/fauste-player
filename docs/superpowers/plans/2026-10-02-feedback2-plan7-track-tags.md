# Track Tags (Feedback 2, Plan 7) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `Track` carries the recording date, genre, album artist, composer and comment, read from the file's tags in a cheap tag-only pass (O23); hovering a table row shows a tooltip with the track's tags, format and path; the row menu opens an "Edit tags…" modal that edits the common tag fields of the file (10 always shown, 26 offered by an Add field menu, only those the format can store) and writes them into the file safely, on a helper thread, keeping every other tag as it is.

**Architecture:**
- **Model first.** `Track` gains five tag fields and `tags_read`; `TrackTags` is the one value type for what the UI shows, what the editor edits and what the file reader returns. `Command::ApplyTags` stores it. Whether a track may be edited is a pure function, `fp_model::tag_edit_block`. The cap on tag text is a `Config` field, `limits.max_tag_chars`.
- **Tag I/O.** `fp-analysis` gains `tags.rs` on top of lofty (already a dependency, so no new crate): `read_track_tags` (the summary the library keeps), `can_write_tags`, `write_tags` (the summary write) and, for the editor, `read_tag_sheet` and `write_tag_sheet`. Both writes go through one helper that copies the file to a temporary file in the same folder, changes only what changed in the copy, fsyncs it and renames it over the original. `fp-app` gains `tags.rs`, a small worker thread (`TagWorker`) that runs the read and write jobs. The services thread uses one worker for the tag-only pass over tracks whose tags were not read yet; the UI uses another for the editor.
- **The tag sheet.** `fp-model` owns `TagField` (the 36 fields of the editor in the shown order), `TagSheet` (the values per field, the fields the format can store, a count of other kept tags) and pure functions on sheets: which values are invalid, which fields changed, which changes the file did not keep. The file reader and writer map each field to lofty's `ItemKey` for the tag type of the file.
- **UI.** `view::track_tooltip` is a pure function; the table shows it with `on_hover_ui`. The row context menu gets "Edit tags…", disabled with a tooltip reason from `tag_edit_block`; `ui/tag_editor.rs` draws the modal over a draft of the sheet ("Reading tags…" until the worker answers) and `AppUi` runs the read and the save and reports the outcome in the modal and in the notice area.

**Tech Stack:** Rust, lofty 0.25.4 (existing dependency), egui/eframe 0.36.2, egui_kittest 0.36.2, `hound` (dev-dependency) for generated WAV files. No new dependency, so `cargo deny check` needs no new entry (Task 2 still runs it).

**Spec:** `docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md` §8 (item O23); its "Editor" bullet (fields, Add field, multi-value, validation, kept tags) is binding for Tasks 5 and 6. Roadmap: `docs/superpowers/plans/2026-10-01-operator-feedback-2-roadmap.md` (plan 7, branch `feat/track-tags`). Plan 8 (track table: O7, O9, O16, O22, O24) builds its Date and Genre columns on the fields this plan adds.

## Decisions taken

The spec leaves these open. Each is the most conservative reading.

- **Edit and write only what changed.** The editor starts from the sheet read from the file when it opens (not from the library, which keeps only the summary and shows the file name for a missing title). `write_tag_sheet(path, before, after, limits)` sets or removes only the fields where `after` differs from `before`. Opening the editor on an untagged file and changing only the artist writes only the artist. A field cleared in the editor removes that tag from the file; an added field left empty is not written. (`write_tags` stays as the summary-level write of Task 2; the editor no longer calls it.)
- **The fields.** Exactly the spec's 36: the ten always shown (title, artist, album, album artist, date, track number, disc number, genre, composer, comment) and 26 optional ones. Each maps to lofty's `ItemKey` for the tag type of the file (Task 5 lists the keys); a field the format cannot store is never offered by Add field, and an always-shown one is drawn disabled with a note.
- **Several values.** One item per value under the field's key (`Tag::push`), which each format stores its own way (ID3v2.4: one frame with the multi-value separator; Vorbis: one comment per value; RIFF INFO: repeated chunks). The multi-value fields are artist, album artist, genre, composer, mood, original artist, lyricist, conductor, remixer, arranger, performer and language; any other field the file happens to hold twice is shown one value per line as well, so nothing is merged silently. Comment and lyrics are one value over several lines.
- **Number and total.** One field with two boxes (`TagSheet::pair`). A total needs a number (lofty would write number 0), and a value written as `3/12` into one key is split when read. A changed pair rewrites both items.
- **What the sheet owns.** Items of the field's key with an empty description. A comment with a description, other keys, custom frames, unmapped frames and pictures are "other": kept as they are and counted in the "N other tags are kept" line (frames lofty keeps without mapping them cannot be counted, so the line says "and more" when the format holds such frames).
- **Validation.** Date and original release date through `parse_tag_date`; track and disc number, their totals and BPM are whole numbers (`u32`). Only a field the operator changed is checked: a value the file already had is kept as it is and never blocks Save. A changed field the format cannot store is invalid too, so the writer never silently drops a change.
- **The file is the truth after a save.** After the write, the worker reads the file back twice, as the summary (`ApplyTags` stores it) and as a sheet. The changed fields whose read-back value differs from what was written are named in the notice ("the file did not keep: …"), so a format quirk is never silent.
- **Which tag is edited.** The primary tag of the format if the file has one, otherwise the first tag it has (the same choice `read_tags` makes when reading), otherwise a new primary tag. Other tags, covers, the `INTRO` marker and unknown items stay as they are, because the same tag object is saved. If the file cannot be parsed with its covers, the save fails; it never retries without them (that would drop the cover).
- **Symlinks.** The write targets the canonical path, so a symlink is not replaced by a regular file.
- **Which tracks are blocked.** `tag_edit_block` refuses, in this order: a track whose file is not `Ok` (missing or unreadable), a format lofty cannot write (decided from the extension only, so the UI thread never touches the disk), a track whose tags are not read yet (`tags_read == false`, so the editor never shows stale fields or races the tag-only pass), a track that is the current entry of any player (playing or paused), a track that is the CUE entry of any player, and a track on a playing cart. It judges the track, not the entry: the same file in another playlist is blocked too, because it is the same file.
- **Re-checked at Save.** The block is evaluated again when Save is pressed and the modal shows the reason instead of saving. A track that goes on air after the write started is safe on Linux and macOS (the rename keeps the open file intact); on Windows the rename fails and the error is reported.
- **The tag-only pass.** Every `ApplyAnalysis` sets `tags_read` to false, so a track just analysed, a track of an older library (the field defaults to false) and a re-analysed track all get one cheap tag read afterwards. No analysis version bump, so the cache and the "analysed by an earlier version" notice are untouched. Failed reads still mark the track read (the reader degrades to the file-name title and empty fields), so nothing loops.
- **Text cap.** Tag text read from files and typed in the editor is trimmed and cut to `limits.max_tag_chars` characters (default 2000, range 64..=100000): a hostile 16 MiB comment must not enter the library file. A field keeps at most `limits.max_tag_values` values (default 32, range 1..=1000), so a file with thousands of artists cannot fill the modal.
- **Date.** The recording date, `Option<String>` holding canonical ISO 8601 text (`YYYY[-MM[-DD[THH[:MM[:SS]]]]]`, partial allowed), as the standards store it: ID3v2.4 `TDRC`, Vorbis `DATE`, MP4 `©day`, APE `Year`. `fp_model::parse_tag_date` validates it (empty is none; year 1..=9999, month 1..=12, day 1..=31, hour 0..=23, minute and second 0..=59). Reading maps lofty's `Timestamp` through its `Display`; writing parses the text back into a `Timestamp`. A date is never reduced to a year: an unrelated edit leaves a full date as it was. The editor accepts empty or a valid date.
- **Tooltip content follows the spec** (title, artist, album, date, genre, duration, format, path; a missing field is left out). Album artist, composer and comment are in the editor and the model only. The codec is the upper-cased file extension (the model records no codec), the sample rate is in kHz and the bit depth only when known.
- **Failure keeps the modal open.** On a failed save the modal stays open with the operator's text, the reason is shown in the modal and in the notice area; on success it closes. While the modal is open no keyboard shortcut acts (Delete would remove the selected entry).
- **Not exposed remotely.** The remote API, MIDI and the session file are unchanged; the library file simply gains the new fields.

## Global Constraints

- All code, identifiers, comments, docs and commit messages are in English. Never mention other products.
- Spec O23 model: "`Track` gains `date`, `genre`, `album_artist`, `composer` and `comment`, read by `fp-analysis` with lofty. Loading is lenient. Tracks from earlier versions show the new fields empty until their tags are read again. That is a tag-only pass, not a full re-analysis."
- Spec O23 tooltip: "Hovering a table row, after the usual tooltip delay, shows the title, artist, album, date (as stored), genre, duration, format (codec, sample rate, bit depth) and path. A missing field is left out."
- Spec O23 editor (binding, spec §8 "Editor"): "The row context menu gains "Edit tags…". It opens a modal for one track. The modal reads the file's tags on a helper thread when it opens and shows "Reading tags…" until they arrive. The library keeps only the summary fields above; the full set is read from the file each time." **Fields:** "The editor covers the fields that common players and tag editors show, not every key a format can hold. Each field uses its format's own standard mapping (ID3v2 frames, Vorbis comments, MP4 atoms, APE items, RIFF INFO) through lofty's `ItemKey`; nothing is renamed or invented. Always shown, in this order: Title, Artist, Album, Album artist, Date, Track number (number and total), Disc number (number and total), Genre, Composer, Comment. Shown when the file has them, and offered by an **Add field** menu otherwise: Subtitle, Grouping, BPM, Initial key, Mood, ISRC, Publisher, Catalog number, Copyright, Original artist, Original album, Original release date, Lyricist, Conductor, Remixer, Arranger, Performer, Language, Encoded by, Lyrics, Sort title, Sort artist, Sort album, Sort album artist, Sort composer, Artist website. **Add field** lists only the fields the file's tag format can store. A field the format cannot store is never shown as editable. Clearing a field removes it from the file. An added field left empty is not written. A field that holds several values (for example two artists) shows one value per line, and Save writes one value per line through the format's own multi-value mechanism. Date and Original release date are ISO 8601 (`YYYY`, `YYYY-MM` or `YYYY-MM-DD`, optional time). Track and disc number and total, and BPM, are whole numbers. An invalid value blocks **Save** and its field is marked. Everything else in the file (other standard keys, custom keys such as ID3v2 `TXXX` or private Vorbis keys, pictures, binary frames) is not shown and is kept byte for byte. The modal says how many such tags are kept." **Save** "writes the tags into the file on a helper thread, never on the UI thread: 1. copy the file to a temporary file in the same folder; 2. write the tags to the copy; 3. fsync it; 4. rename it over the original. On success, the library takes the new tags; markers and analysis are kept. On any error, the original file is untouched and the notice area reports it. The menu item is disabled, with the reason as a tooltip, when: the track is current or cued in any player; it is on a playing cart; its format has no writable tags in lofty; the file is missing."
- CLAUDE.md rule 4: anything an operator might change is a `Config` field with a documented default, a range in `Config::validate` and lenient loading (Task 1: `limits.max_tag_chars`; Task 5: `limits.max_tag_values`).
- CLAUDE.md rule 6: no `unwrap`, `expect` or `panic` outside tests; `fp-analysis` denies `clippy::indexing_slicing` (use `get`).
- CLAUDE.md rules 8 and 9: the UI never blocks (tag reading and writing run on worker threads; the UI only checks the extension) and untrusted tags, covers and files degrade to "not available" with a log line. A panic inside tag code is caught and reported as a failed job.
- Behaviour lives in `fp-model` as pure functions (`apply_tags`, `tag_edit_block`, `needs_tag_read`, `TrackTags::clamped`, `parse_tag_date`, and for the editor `TagSheet`, `invalid_fields`, `changed_fields`, `unstored_fields`); the UI only displays and sends commands.
- Nothing goes on air by itself: this plan starts no audio.
- UI strings are Fluent messages in `crates/fp-app/locales/en-US/main.ftl` (source) and `es-ES/main.ftl`, always both (`tests/i18n.rs::both_locales_define_the_same_keys` checks it).
- Test files are generated in `tempfile` directories (WAV through `hound`, tagged through lofty); no network, no encoders, no real music.
- Commands: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p <crate> --test <file> <name>` (one filter per command). Build only into the repo's `target/`.
- Commit only when fmt, clippy and the whole suite pass (`cargo test --workspace`). Commits end with the co-author trailer the harness provides (the commit commands below show the subject and the body only). `CHANGELOG.md` is never edited by hand.
- Documentation is part of the change (Task 7).

## Review Focus

Failure modes the spec implies and its tests do not name; each has a test in the task named.

1. A save that fails half way or must be refused (read-only folder or file, the file vanished, a format lofty cannot write, a corrupt file, an invalid value, a change to a field the format cannot store): the original file must be byte-identical and no `*.fptag-*` temporary file may be left behind. (Task 2 `a_failed_write_leaves_the_original_and_no_temporary_file`, `an_unwritable_format_is_refused_without_touching_the_file`; Task 5 `a_failed_sheet_write_leaves_the_original_and_no_temporary_file`, `an_invalid_value_is_refused_and_the_file_is_untouched`, `a_field_the_format_cannot_store_is_refused_before_the_disk_is_touched`; Task 6 `a_failed_save_keeps_the_modal_and_says_why`.)
2. A file whose tags the editor does not show must not lose them: covers, other tag types, custom ID3v2 frames (`TXXX`), comments with a description, URL frames (which lofty 0.25.4 drops on save unless re-added as text), items such as `INTRO`, and fields the operator did not change. (Task 2 `fields_that_did_not_change_are_not_written`, `a_cover_and_other_items_survive_a_write`; Task 5 `custom_items_and_pictures_survive_an_edit_untouched`, `a_summary_write_keeps_the_url_frames_too`, `only_the_changed_fields_are_written`.)
3. Hostile or huge tag text (a 5 MB comment, thousands of values in one field, control characters, text with only spaces) must neither bloat the library or the modal nor crash. (Task 1 `tag_text_is_trimmed_and_cut`; Task 2 `a_huge_comment_is_cut_when_read`; Task 5 `the_text_and_the_values_are_cut_when_read`, `what_is_written_is_cut_to_the_limits`.)
4. Tracks that must not be edited or written: on air in a player, cued, on a playing cart, tags not read yet, missing file, unwritable format; and the state changing while the modal is open (the track goes on air, is removed from the library). (Task 1 `tag_edit_block_*`; Task 6 `the_menu_item_is_disabled_with_the_reason`, `saving_is_refused_when_the_track_went_on_air`, `the_modal_closes_when_the_track_is_removed`, `no_shortcut_acts_under_the_editor`.)
5. A library from an earlier version (no new fields, `tags_read` false) must load, show empty fields, and be filled by the tag-only pass without a full analysis; a failed read must not loop. (Task 1 `an_old_library_entry_loads_with_empty_tags`; Task 3 `the_tag_pass_fills_the_tags_of_analysed_tracks_once`, `a_track_whose_tags_cannot_be_read_is_not_asked_again`.)

## File Structure

- Modify `crates/fp-model/src/track.rs` (`TrackTags`, `InvalidDate`, `parse_tag_date`, new `Track` fields, `Track::tags`, `apply_tags`, `needs_tag_read`), `config.rs` (`Limits::max_tag_chars`), `command.rs` (`Command::ApplyTags`), `reducer.rs` (the arm), `lib.rs` (exports).
- Create `crates/fp-model/src/tag_edit.rs`: `TagEditBlock`, `tag_edit_block`.
- Create `crates/fp-model/src/tag_sheet.rs`: `TagField`, `TagFieldKind`, `TagSheet`, `changed_fields`, `invalid_fields`, `unstored_fields`; `config.rs` also gets `Limits::max_tag_values`.
- Create `crates/fp-model/tests/track_tags.rs` and `tests/tag_sheet.rs`.
- Modify `crates/fp-analysis/src/metadata.rs` (`Tags` gains five fields), `lib.rs`; create `crates/fp-analysis/src/tags.rs` (`read_track_tags`, `can_write_tags`, `write_tags`, `TagWriteError`; Task 5 adds `safe_edit`, `storable_fields`, `read_tag_sheet`, `write_tag_sheet`); create `crates/fp-analysis/tests/tags.rs` and `tests/tag_sheet.rs`.
- Create `crates/fp-app/src/tags.rs` (`TagWorker`, `TagJob`, `TagOutcome`; Task 5 adds the sheet jobs and `SheetSaved`); modify `crates/fp-app/src/lib.rs`, `services.rs` (the tag-only pass); modify `crates/fp-app/tests/services.rs`.
- Modify `crates/fp-app/src/ui/view.rs` (`TrackTip`, `track_tooltip`, `tag_edit_availability`), `ui/table.rs` (tooltip, menu item), `ui/app.rs` (state, worker, modal, notice), `ui.rs`.
- Create `crates/fp-app/src/ui/tag_editor.rs`.
- Modify `crates/fp-app/locales/en-US/main.ftl`, `es-ES/main.ftl`.
- Create `crates/fp-app/tests/track_tags_ui.rs`; modify `crates/fp-app/tests/view.rs`.
- Modify docs (Task 7): `docs/user/playlists.md`, `docs/technical/analysis.md`, `docs/technical/persistence.md`, `docs/technical/ui.md`, the spec (status and "As built" under §8), `docs/superpowers/plans/2026-10-01-operator-feedback-2-roadmap.md` (row 7 done), `README.md` (feature line).

---

### Task 1: Tag fields, `ApplyTags` and the edit rule (model)

**Files:**
- Modify: `crates/fp-model/src/track.rs` (`Track`, near line 161; `Track::new`, near line 207; `apply_analysis`, near line 305)
- Modify: `crates/fp-model/src/config.rs` (`Limits`, near line 384; `Config::validate`, near line 502)
- Modify: `crates/fp-model/src/command.rs` (after `ApplyAnalysis`, near line 60)
- Modify: `crates/fp-model/src/reducer.rs` (after the `ApplyAnalysis` arm, near line 144)
- Modify: `crates/fp-model/src/lib.rs`
- Create: `crates/fp-model/src/tag_edit.rs`
- Test: Create `crates/fp-model/tests/track_tags.rs`

**Interfaces:**
- Consumes: `AppState::{library, players, playlists, cartwall}`, `Cartwall::{playing, cart}`, `PlayerState::{current, cue}`, `Playlists::entry`.
- Produces (plan 8 and Tasks 2-6 rely on these exact names):
  - `pub struct TrackTags { pub title: String, pub artist: String, pub album: String, pub album_artist: String, pub date: Option<String>, pub genre: String, pub composer: String, pub comment: String }` (`Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize`), with `fn clamped(self, max_chars: usize) -> TrackTags`.
  - `pub struct InvalidDate;` and `pub fn parse_tag_date(text: &str) -> Result<Option<String>, InvalidDate>` (pure; trims; empty is `Ok(None)`; otherwise exactly `YYYY[-MM[-DD[THH[:MM[:SS]]]]]` with ASCII digits and the ranges above, returning the trimmed text).
  - `Track` fields `pub date: Option<String>`, `pub genre: String`, `pub album_artist: String`, `pub composer: String`, `pub comment: String`, `pub tags_read: bool` (all `#[serde(default)]`); methods `Track::tags(&self) -> TrackTags`, `Track::apply_tags(&mut self, tags: &TrackTags)`, `Track::needs_tag_read(&self) -> bool`.
  - `Command::ApplyTags { track: TrackId, tags: Box<TrackTags> }`.
  - `Limits::max_tag_chars: usize` (default 2000).
  - `pub enum TagEditBlock { FileUnavailable, UnsupportedFormat, TagsNotRead, OnAir, Cued, OnCart }` and `pub fn tag_edit_block(state: &AppState, track: TrackId, format_writable: bool) -> Option<TagEditBlock>`.

- [ ] **Step 1: Write the failing tests**

Create `crates/fp-model/tests/track_tags.rs`:

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback 2 spec O23: tag fields, `ApplyTags` and the edit rule.

mod common;

use std::path::PathBuf;

use common::{entries, fixture, p0};
use fp_model::{
    AppState, CartEdit, CartKind, Command, Config, FileState, TagEditBlock, Track, TrackAnalysis,
    TrackId, TrackTags, apply, parse_tag_date, tag_edit_block,
};

fn track_of(state: &AppState, n: usize) -> TrackId {
    state.playlists.entry(entries(state)[n]).unwrap().track
}

fn tags() -> TrackTags {
    TrackTags {
        title: "Real Title".into(),
        artist: "Real Artist".into(),
        album: "An Album".into(),
        album_artist: "Various".into(),
        date: Some("1999-03-07".into()),
        genre: "Pop".into(),
        composer: "A. Composer".into(),
        comment: "A note".into(),
    }
}

/// A state whose track `n` is analysed, its tags are read and its file is fine.
fn ready(n: usize) -> (AppState, TrackId) {
    let mut s = fixture(3);
    let t = track_of(&s, n);
    apply(
        &mut s,
        Command::ApplyAnalysis {
            track: t,
            analysis: Box::new(TrackAnalysis {
                duration_secs: 180.0,
                ..TrackAnalysis::default()
            }),
        },
    )
    .unwrap();
    apply(
        &mut s,
        Command::ApplyTags {
            track: t,
            tags: Box::new(tags()),
        },
    )
    .unwrap();
    (s, t)
}

#[test]
fn apply_tags_stores_every_field_and_marks_the_tags_read() {
    let (s, t) = ready(0);
    let track = s.library.get(t).unwrap();
    assert_eq!(track.tags(), tags());
    assert!(track.tags_read);
    assert_eq!(track.title, "Real Title");
    assert_eq!(track.date.as_deref(), Some("1999-03-07"));
}

#[test]
fn apply_tags_can_clear_a_field_but_never_the_title() {
    let (mut s, t) = ready(0);
    let cleared = TrackTags {
        title: String::new(),
        album: String::new(),
        date: None,
        ..tags()
    };
    apply(
        &mut s,
        Command::ApplyTags {
            track: t,
            tags: Box::new(cleared),
        },
    )
    .unwrap();
    let track = s.library.get(t).unwrap();
    assert_eq!(track.album, "");
    assert_eq!(track.date, None);
    assert_eq!(track.title, "Real Title", "an empty title keeps the old one");
}

#[test]
fn apply_tags_for_a_removed_track_does_nothing() {
    let (mut s, _) = ready(0);
    let before = s.clone();
    apply(
        &mut s,
        Command::ApplyTags {
            track: TrackId(9999),
            tags: Box::new(tags()),
        },
    )
    .unwrap();
    assert_eq!(s, before);
}

#[test]
fn a_new_analysis_asks_for_the_tags_again_and_keeps_them_until_then() {
    let (mut s, t) = ready(0);
    assert!(!s.library.get(t).unwrap().needs_tag_read());
    apply(
        &mut s,
        Command::ApplyAnalysis {
            track: t,
            analysis: Box::new(TrackAnalysis {
                duration_secs: 180.0,
                ..TrackAnalysis::default()
            }),
        },
    )
    .unwrap();
    let track = s.library.get(t).unwrap();
    assert!(track.needs_tag_read());
    assert_eq!(track.genre, "Pop", "analysis does not touch the new fields");
}

#[test]
fn only_analysed_playable_tracks_need_a_tag_read() {
    let mut s = fixture(1);
    let t = track_of(&s, 0);
    assert!(!s.library.get(t).unwrap().needs_tag_read(), "not analysed");
    apply(
        &mut s,
        Command::ApplyAnalysis {
            track: t,
            analysis: Box::new(TrackAnalysis::default()),
        },
    )
    .unwrap();
    assert!(s.library.get(t).unwrap().needs_tag_read());
    s.library.get_mut(t).unwrap().file_state = FileState::Missing;
    assert!(!s.library.get(t).unwrap().needs_tag_read(), "no file to read");
}

#[test]
fn an_old_library_entry_loads_with_empty_tags() {
    let json = r#"{"id":7,"path":"/m/a.flac","title":"A","artist":"B","album":"C",
        "duration_secs":10.0,"kind":"Music","file_state":"Ok","markers":{},"analyzed":true}"#;
    let track: Track = serde_json::from_str(json).unwrap();
    assert_eq!(track.date, None);
    assert_eq!(track.genre, "");
    assert!(!track.tags_read);
    assert!(track.needs_tag_read());
}

#[test]
fn tag_text_is_trimmed_and_cut() {
    let long = format!("  {}  ", "é".repeat(100));
    let t = TrackTags {
        title: long,
        comment: "   ".into(),
        ..TrackTags::default()
    }
    .clamped(10);
    assert_eq!(t.title, "é".repeat(10));
    assert_eq!(t.comment, "", "only spaces is empty");
}

#[test]
fn a_tag_date_is_iso_8601_and_may_be_partial() {
    assert_eq!(parse_tag_date(""), Ok(None));
    assert_eq!(parse_tag_date("   "), Ok(None));
    for ok in [
        "1984",
        "0001",
        "9999",
        "2019-05",
        "2019-05-14",
        "2019-05-14T08",
        "2019-05-14T08:30",
        "2019-05-14T23:59:59",
        "2019-12-31T00:00:00",
    ] {
        assert_eq!(parse_tag_date(ok), Ok(Some(ok.to_owned())), "{ok}");
    }
    assert_eq!(
        parse_tag_date("  2019-05-14 "),
        Ok(Some("2019-05-14".to_owned())),
        "trimmed"
    );
}

#[test]
fn anything_but_an_iso_8601_date_is_refused() {
    for bad in [
        "0",
        "0000",
        "19",
        "10000",
        "-3",
        "19x4",
        "1984.5",
        "٣٣٣٣",
        "2019-5",
        "2019-00",
        "2019-13",
        "2019-05-00",
        "2019-05-32",
        "2019/05/14",
        "abc",
        "2019-05-14T",
        "2019-05-14T25",
        "2019-05-14T08:60",
        "2019-05-14T08:30:60",
        "2019-05-14 08:30",
        "2019-05-14T8",
        "2019-05-14T08:30:00:00",
        "2019-05-14T08:30:00Z",
        "2019-",
    ] {
        assert!(parse_tag_date(bad).is_err(), "{bad}");
    }
}

#[test]
fn a_date_round_trips_through_apply_tags() {
    let mut track = Track::new(TrackId(1), PathBuf::from("a.mp3"));
    track.apply_tags(&TrackTags {
        date: Some("2019-05-14".into()),
        ..TrackTags::default()
    });
    assert_eq!(track.tags().date.as_deref(), Some("2019-05-14"));
    track.apply_tags(&TrackTags::default());
    assert_eq!(track.tags().date, None);
}

#[test]
fn the_text_cap_is_a_validated_config_field() {
    let mut c = Config::default();
    assert_eq!(c.limits.max_tag_chars, 2000);
    c.limits.max_tag_chars = 3;
    let warnings = c.validate();
    assert_eq!(c.limits.max_tag_chars, 64);
    assert!(warnings.iter().any(|w| w.field == "limits.max_tag_chars"));
    c.limits.max_tag_chars = 10_000_000;
    c.validate();
    assert_eq!(c.limits.max_tag_chars, 100_000);
}

#[test]
fn tag_edit_block_allows_a_quiet_readable_track() {
    let (s, t) = ready(1);
    assert_eq!(tag_edit_block(&s, t, true), None);
}

#[test]
fn tag_edit_block_names_each_reason() {
    let (mut s, t) = ready(0);
    assert_eq!(
        tag_edit_block(&s, t, false),
        Some(TagEditBlock::UnsupportedFormat)
    );
    assert_eq!(
        tag_edit_block(&s, TrackId(9999), true),
        Some(TagEditBlock::FileUnavailable),
        "an unknown track"
    );
    s.library.get_mut(t).unwrap().tags_read = false;
    assert_eq!(tag_edit_block(&s, t, true), Some(TagEditBlock::TagsNotRead));
    s.library.get_mut(t).unwrap().tags_read = true;
    for state in [FileState::Missing, FileState::Unreadable] {
        s.library.get_mut(t).unwrap().file_state = state;
        assert_eq!(
            tag_edit_block(&s, t, true),
            Some(TagEditBlock::FileUnavailable)
        );
    }
}

#[test]
fn tag_edit_block_refuses_a_track_on_air_or_cued() {
    let (mut s, t) = ready(0);
    let p = p0(&s);
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    assert_eq!(tag_edit_block(&s, t, true), Some(TagEditBlock::Cued));
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(tag_edit_block(&s, t, true), Some(TagEditBlock::OnAir));
    apply(&mut s, Command::Pause(p)).unwrap();
    assert_eq!(
        tag_edit_block(&s, t, true),
        Some(TagEditBlock::OnAir),
        "paused is still on air"
    );
    apply(&mut s, Command::Stop(p)).unwrap();
    assert_eq!(tag_edit_block(&s, t, true), None);
}

#[test]
fn tag_edit_block_judges_the_file_not_the_entry() {
    let (mut s, t) = ready(0);
    let playlist = s.playlists.first_id().unwrap();
    apply(
        &mut s,
        Command::InsertTracks {
            playlist,
            index: 3,
            tracks: vec![t],
        },
    )
    .unwrap();
    let p = p0(&s);
    apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(tag_edit_block(&s, t, true), Some(TagEditBlock::OnAir));
}

#[test]
fn tag_edit_block_refuses_a_track_on_a_playing_cart() {
    let (mut s, _) = ready(0);
    let page = s.cartwall.pages[0].id;
    apply(
        &mut s,
        Command::AssignCartFile {
            page,
            index: 0,
            path: PathBuf::from("/carts/id.wav"),
        },
    )
    .unwrap();
    let cart_track = s.cartwall.pages[0].carts[0].track.unwrap();
    let cart = s.cartwall.pages[0].carts[0].id;
    apply(
        &mut s,
        Command::ApplyAnalysis {
            track: cart_track,
            analysis: Box::new(TrackAnalysis {
                duration_secs: 10.0,
                cue_out: Some(9.5),
                ..TrackAnalysis::default()
            }),
        },
    )
    .unwrap();
    apply(
        &mut s,
        Command::ApplyTags {
            track: cart_track,
            tags: Box::new(tags()),
        },
    )
    .unwrap();
    let edit = CartEdit {
        name: "ID".into(),
        kind: CartKind::Jingle,
        looped: false,
        exclusive: false,
    };
    apply(&mut s, Command::SetCart { page, index: 0, edit }).unwrap();
    assert_eq!(tag_edit_block(&s, cart_track, true), None, "a cart at rest");
    apply(&mut s, Command::FireCart(cart)).unwrap();
    assert_eq!(
        tag_edit_block(&s, cart_track, true),
        Some(TagEditBlock::OnCart)
    );
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-model --test track_tags`
Expected: FAIL to compile (`TrackTags`, `tag_edit_block`, `Command::ApplyTags`, `parse_tag_date` not found). `serde_json` is already a dev-dependency of `fp-model` (check `crates/fp-model/Cargo.toml`; add `serde_json.workspace = true` under `[dev-dependencies]` if it is missing).

- [ ] **Step 3: Implement**

`crates/fp-model/src/track.rs`, after `AudioFormat`:

```rust
/// The text tags of a file as the player shows and edits them (feedback 2
/// spec O23). Empty text and `None` mean the file has no such tag.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackTags {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_artist: String,
    pub date: Option<String>,
    pub genre: String,
    pub composer: String,
    pub comment: String,
}

/// Cuts `text` to `max_chars` characters after trimming it.
fn cut(text: &mut String, max_chars: usize) {
    let trimmed = text.trim();
    let end = trimmed
        .char_indices()
        .nth(max_chars)
        .map_or(trimmed.len(), |(i, _)| i);
    *text = trimmed.get(..end).unwrap_or_default().to_owned();
}

impl TrackTags {
    /// Every text field trimmed and cut to `max_chars` characters
    /// (`limits.max_tag_chars`): tag text comes from files and keyboards,
    /// and must not bloat the library.
    #[must_use]
    pub fn clamped(mut self, max_chars: usize) -> Self {
        for text in [
            &mut self.title,
            &mut self.artist,
            &mut self.album,
            &mut self.album_artist,
            &mut self.genre,
            &mut self.composer,
            &mut self.comment,
        ] {
            cut(text, max_chars);
        }
        if let Some(date) = &mut self.date {
            cut(date, max_chars);
        }
        self
    }
}

/// The date field holds something that is not an ISO 8601 date.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidDate;

/// Reads `digits` ASCII digits from the front of `text` as a number in
/// `range`, and returns it with the rest of the text.
fn field(text: &str, digits: usize, range: std::ops::RangeInclusive<u32>) -> Option<(u32, &str)> {
    let (head, rest) = (text.get(..digits)?, text.get(digits..)?);
    if !head.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let value = head.parse::<u32>().ok()?;
    range.contains(&value).then_some((value, rest))
}

/// The recording date as the standards store it (ID3v2.4 `TDRC`, Vorbis
/// `DATE`, MP4 `©day`, APE `Year`): an ISO 8601 timestamp that may be
/// partial, `YYYY[-MM[-DD[THH[:MM[:SS]]]]]`. Empty is no date. The text
/// comes back trimmed; the day is not checked against the month.
pub fn parse_tag_date(text: &str) -> Result<Option<String>, InvalidDate> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(None);
    }
    // (separator, digits, range) of each field after the year.
    const FIELDS: [(char, usize, std::ops::RangeInclusive<u32>); 5] = [
        ('-', 2, 1..=12),
        ('-', 2, 1..=31),
        ('T', 2, 0..=23),
        (':', 2, 0..=59),
        (':', 2, 0..=59),
    ];
    let (_, mut rest) = field(text, 4, 1..=9999).ok_or(InvalidDate)?;
    for (separator, digits, range) in FIELDS {
        let Some(after) = rest.strip_prefix(separator) else {
            break;
        };
        let (_, next) = field(after, digits, range).ok_or(InvalidDate)?;
        rest = next;
    }
    if rest.is_empty() {
        Ok(Some(text.to_owned()))
    } else {
        Err(InvalidDate)
    }
}
```

In `Track` add, after `analysis_version`:

```rust
    /// Tags beyond title, artist and album (feedback 2 spec O23). Libraries
    /// saved earlier have none until the tag-only pass reads them.
    #[serde(default)]
    pub date: Option<String>,
    #[serde(default)]
    pub genre: String,
    #[serde(default)]
    pub album_artist: String,
    #[serde(default)]
    pub composer: String,
    #[serde(default)]
    pub comment: String,
    /// The tags above were read from the file after the last analysis. False
    /// asks for the tag-only pass (`needs_tag_read`).
    #[serde(default)]
    pub tags_read: bool,
```

In `Track::new` add `date: None, genre: String::new(), album_artist: String::new(), composer: String::new(), comment: String::new(), tags_read: false,`. In the `impl Track` block add:

```rust
    /// The tags as shown and edited.
    pub fn tags(&self) -> TrackTags {
        TrackTags {
            title: self.title.clone(),
            artist: self.artist.clone(),
            album: self.album.clone(),
            album_artist: self.album_artist.clone(),
            date: self.date.clone(),
            genre: self.genre.clone(),
            composer: self.composer.clone(),
            comment: self.comment.clone(),
        }
    }

    /// Stores what the file's tags say now. Unlike analysis, an empty field
    /// clears the old value (the tag is gone); the title is the exception, as
    /// a track always has one.
    pub fn apply_tags(&mut self, tags: &TrackTags) {
        if !tags.title.is_empty() {
            self.title.clone_from(&tags.title);
        }
        self.artist.clone_from(&tags.artist);
        self.album.clone_from(&tags.album);
        self.album_artist.clone_from(&tags.album_artist);
        self.date.clone_from(&tags.date);
        self.genre.clone_from(&tags.genre);
        self.composer.clone_from(&tags.composer);
        self.comment.clone_from(&tags.comment);
        self.tags_read = true;
    }

    /// Whether the tag-only pass should read this track's file: analysed,
    /// readable, and not read since the last analysis.
    pub fn needs_tag_read(&self) -> bool {
        self.analyzed && !self.tags_read && self.file_state.is_playable()
    }
```

In `apply_analysis`, before `self.analyzed = true;` add `// Title, artist and album come from this analysis; the rest wait for the tag-only pass.\n        self.tags_read = false;`.

`config.rs`: add to `Limits` `/// Longest tag text kept per field, in characters (tags are trimmed and cut).\n    pub max_tag_chars: usize,`, default `max_tag_chars: 2000,`, and in `validate` after the `max_cover_pixels` clamp:

```rust
        clamp_to(
            &mut l.max_tag_chars,
            64,
            100_000,
            "limits.max_tag_chars",
            &mut w,
        );
```

`command.rs`: import `TrackTags` in the `use crate::track::{...}` line and add after `ApplyAnalysis`:

```rust
    /// Feedback 2 spec O23: the file's tags as read after the tag-only pass
    /// or written by the tag editor (ignored if the track is gone).
    ApplyTags {
        track: TrackId,
        tags: Box<TrackTags>,
    },
```

`reducer.rs`, after the `ApplyAnalysis` arm:

```rust
        Command::ApplyTags { track, tags } => {
            if let Some(t) = state.library.get_mut(track) {
                t.apply_tags(&tags);
            }
        }
```

(Titles are shown by the table and the players, so no `refresh_next` is needed.)

Create `crates/fp-model/src/tag_edit.rs`:

```rust
//! When a track's tags may be edited (feedback 2 spec O23): a pure rule of
//! the state, so the menu and the Save button agree.

use crate::ids::TrackId;
use crate::state::AppState;

/// Why the tags of a track cannot be edited now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagEditBlock {
    /// The file is missing or cannot be read (or the track is unknown).
    FileUnavailable,
    /// The format has no writable tags.
    UnsupportedFormat,
    /// The tag-only pass has not read this track's tags yet.
    TagsNotRead,
    /// The track is the current entry of a player, playing or paused.
    OnAir,
    /// The track is the CUE entry of a player.
    Cued,
    /// The track is on a playing cart.
    OnCart,
}

/// The first reason `track` cannot be edited, or `None`. `format_writable`
/// says whether the file's format has writable tags (`fp-analysis` decides
/// that from the extension). It judges the file, so the same track in
/// another playlist is refused too.
pub fn tag_edit_block(
    state: &AppState,
    track: TrackId,
    format_writable: bool,
) -> Option<TagEditBlock> {
    let Some(t) = state.library.get(track) else {
        return Some(TagEditBlock::FileUnavailable);
    };
    if !t.file_state.is_playable() {
        return Some(TagEditBlock::FileUnavailable);
    }
    if !format_writable {
        return Some(TagEditBlock::UnsupportedFormat);
    }
    if !t.tags_read {
        return Some(TagEditBlock::TagsNotRead);
    }
    let is_this_track = |entry| {
        state
            .playlists
            .entry(entry)
            .is_some_and(|e| e.track == track)
    };
    if state
        .players
        .iter()
        .any(|p| p.current.is_some_and(is_this_track))
    {
        return Some(TagEditBlock::OnAir);
    }
    if state
        .players
        .iter()
        .any(|p| p.cue.is_some_and(|c| is_this_track(c.entry)))
    {
        return Some(TagEditBlock::Cued);
    }
    let on_cart = state.cartwall.playing.iter().any(|playing| {
        state
            .cartwall
            .cart(playing.cart)
            .is_some_and(|c| c.track == Some(track))
    });
    on_cart.then_some(TagEditBlock::OnCart)
}
```

`lib.rs`: add `mod tag_edit;`, `pub use tag_edit::{TagEditBlock, tag_edit_block};` and extend the `track` re-export with `InvalidDate, TrackTags, parse_tag_date`.

- [ ] **Step 4: Run the tests to verify they pass**

Run each: `cargo test -p fp-model --test track_tags`, then `cargo test -p fp-model` (whole crate: the existing analysis tests must still pass, and any non-exhaustive `Command` match the compiler reports elsewhere in the workspace gets an `ApplyTags` arm next to `ApplyAnalysis`).
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-model
git commit -m "feat(model): track tag fields, ApplyTags and the tag edit rule"
```

---

### Task 2: Reading more tags and writing them safely (`fp-analysis`)

**Files:**
- Modify: `crates/fp-analysis/src/metadata.rs` (`Tags`, `read_tags`)
- Modify: `crates/fp-analysis/src/lib.rs` (`pub mod tags;`)
- Create: `crates/fp-analysis/src/tags.rs`
- Test: Create `crates/fp-analysis/tests/tags.rs`

**Interfaces:**
- Consumes: `fp_model::{TrackTags, Limits}`, `metadata::{read_tags, title_from_file_name}`.
- Produces:
  - `Tags` gains `date: Option<String>` (kept only if `fp_model::parse_tag_date` accepts it, else `None` with a debug log line), `genre`, `album_artist`, `composer`, `comment: Option<String>` (all `None` when absent).
  - `fp_analysis::tags::read_track_tags(path: &Path, limits: &Limits) -> TrackTags`: never fails; the title falls back to the file name (and the artist to the file name's `Artist - `), text is cut to `limits.max_tag_chars`.
  - `fp_analysis::tags::can_write_tags(path: &Path) -> bool`: from the extension only, no I/O.
  - `fp_analysis::tags::write_tags(path: &Path, before: &TrackTags, after: &TrackTags, limits: &Limits) -> Result<(), TagWriteError>`.
  - `#[derive(Debug, Clone, PartialEq, Eq)] pub enum TagWriteError { Unsupported, NotFound, Denied, InvalidDate, Other(String) }` with `Display`.

- [ ] **Step 1: Write the failing tests**

Create `crates/fp-analysis/tests/tags.rs`:

```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Feedback 2 spec O23: reading the extra tags and writing them safely.

use std::io::Cursor;
use std::path::{Path, PathBuf};

use fp_analysis::tags::{TagWriteError, can_write_tags, read_track_tags, write_tags};
use fp_model::{Limits, TrackTags};
use lofty::config::WriteOptions;
use lofty::picture::{MimeType, Picture, PictureType};
use lofty::prelude::*;
use lofty::tag::{ItemKey, Tag};

fn wav(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join(name);
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 44_100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(&path, spec).unwrap();
    for i in 0..4_410 {
        w.write_sample((i % 1000) as i16).unwrap();
        w.write_sample(0i16).unwrap();
    }
    w.finalize().unwrap();
    path
}

fn limits() -> Limits {
    Limits::default()
}

fn full() -> TrackTags {
    TrackTags {
        title: "Song Title".into(),
        artist: "The Artist".into(),
        album: "An Album".into(),
        album_artist: "Various Artists".into(),
        date: Some("1999-03-07".into()),
        genre: "Pop".into(),
        composer: "A. Composer".into(),
        comment: "A note".into(),
    }
}

fn untagged_tags(path: &Path) -> TrackTags {
    read_track_tags(path, &limits())
}

fn png() -> Vec<u8> {
    let img = image::RgbImage::from_pixel(40, 30, image::Rgb([200, 30, 30]));
    let mut out = Vec::new();
    image::DynamicImage::ImageRgb8(img)
        .write_to(&mut Cursor::new(&mut out), image::ImageFormat::Png)
        .unwrap();
    out
}

/// Files in `dir`, to prove no temporary file is left.
fn names(dir: &Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    v.sort();
    v
}

#[test]
fn an_untagged_file_reads_as_its_file_name() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "Marta Oliva - Carretera norte.wav");
    let t = untagged_tags(&path);
    assert_eq!(t.title, "Carretera norte");
    assert_eq!(t.artist, "Marta Oliva");
    assert_eq!((t.date, t.genre.as_str(), t.comment.as_str()), (None, "", ""));
}

#[test]
fn every_field_survives_a_write_and_a_read() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "x.wav");
    let before = untagged_tags(&path);
    write_tags(&path, &before, &full(), &limits()).unwrap();
    assert_eq!(read_track_tags(&path, &limits()), full());
    assert_eq!(names(dir.path()), ["x.wav"], "no temporary file is left");
}

#[test]
fn the_audio_is_untouched_by_a_write() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "x.wav");
    let before = untagged_tags(&path);
    let samples = |p: &Path| -> Vec<i16> {
        hound::WavReader::open(p)
            .unwrap()
            .samples::<i16>()
            .map(Result::unwrap)
            .collect()
    };
    let audio = samples(&path);
    write_tags(&path, &before, &full(), &limits()).unwrap();
    assert_eq!(samples(&path), audio);
}

#[test]
fn a_cleared_field_removes_the_tag() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "x.wav");
    let before = untagged_tags(&path);
    write_tags(&path, &before, &full(), &limits()).unwrap();
    let tagged = read_track_tags(&path, &limits());
    let cleared = TrackTags {
        album: String::new(),
        date: None,
        composer: String::new(),
        album_artist: String::new(),
        ..tagged.clone()
    };
    write_tags(&path, &tagged, &cleared, &limits()).unwrap();
    assert_eq!(read_track_tags(&path, &limits()), cleared);
}

#[test]
fn fields_that_did_not_change_are_not_written() {
    // The title shown for an untagged file is its file name; saving a new
    // artist must not turn that name into a title tag.
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "Just a name.wav");
    let before = untagged_tags(&path);
    let after = TrackTags {
        artist: "Someone".into(),
        ..before.clone()
    };
    write_tags(&path, &before, &after, &limits()).unwrap();
    let tag = lofty::read_from_path(&path).unwrap();
    let tag = tag.primary_tag().unwrap();
    assert_eq!(tag.artist().as_deref(), Some("Someone"));
    assert_eq!(tag.title(), None, "the file name was not written as a title");
}

#[test]
fn a_cover_and_other_items_survive_a_write() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "x.wav");
    {
        let mut file = lofty::read_from_path(&path).unwrap();
        let ty = file.primary_tag_type();
        file.insert_tag(Tag::new(ty));
        let tag = file.primary_tag_mut().unwrap();
        tag.set_title("Old".into());
        tag.insert_text(ItemKey::Publisher, "A Label".into());
        tag.push_picture(
            Picture::unchecked(png())
                .pic_type(PictureType::CoverFront)
                .mime_type(MimeType::Png)
                .build(),
        );
        file.save_to_path(&path, WriteOptions::default()).unwrap();
    }
    let before = untagged_tags(&path);
    let after = TrackTags {
        title: "New".into(),
        ..before.clone()
    };
    write_tags(&path, &before, &after, &limits()).unwrap();
    let file = lofty::read_from_path(&path).unwrap();
    let tag = file.primary_tag().unwrap();
    assert_eq!(tag.title().as_deref(), Some("New"));
    assert_eq!(tag.get_string(ItemKey::Publisher), Some("A Label"));
    assert_eq!(tag.pictures().len(), 1, "the cover is kept");
}

#[test]
fn a_failed_write_leaves_the_original_and_no_temporary_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("broken.wav");
    // A RIFF header with no body: lofty cannot parse it.
    std::fs::write(&path, b"RIFF\x04\x00\x00\x00WAVE").unwrap();
    let original = std::fs::read(&path).unwrap();
    let result = write_tags(&path, &TrackTags::default(), &full(), &limits());
    assert!(matches!(result, Err(TagWriteError::Other(_))), "{result:?}");
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert_eq!(names(dir.path()), ["broken.wav"]);
}

#[test]
fn a_missing_file_is_reported_as_not_found() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("gone.wav");
    let result = write_tags(&path, &TrackTags::default(), &full(), &limits());
    assert_eq!(result, Err(TagWriteError::NotFound));
    assert!(names(dir.path()).is_empty());
}

#[test]
fn an_unwritable_format_is_refused_without_touching_the_file() {
    let dir = tempfile::tempdir().unwrap();
    for name in ["a.dsf", "a.dff", "a.caf", "a.mka", "a.txt", "noextension"] {
        let path = dir.path().join(name);
        std::fs::write(&path, b"not audio").unwrap();
        assert!(!can_write_tags(&path), "{name}");
        let result = write_tags(&path, &TrackTags::default(), &full(), &limits());
        assert_eq!(result, Err(TagWriteError::Unsupported), "{name}");
        assert_eq!(std::fs::read(&path).unwrap(), b"not audio");
    }
    for name in ["a.mp3", "a.FLAC", "a.wav", "a.ogg", "a.opus", "a.m4a", "a.wv", "a.ape", "a.aiff"] {
        assert!(can_write_tags(Path::new(name)), "{name}");
    }
}

#[cfg(unix)]
#[test]
fn a_read_only_folder_is_reported_as_denied_and_changes_nothing() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "x.wav");
    let original = std::fs::read(&path).unwrap();
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o555)).unwrap();
    let result = write_tags(&path, &untagged_tags(&path), &full(), &limits());
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
    // A superuser can write anywhere: the check only applies to the rest.
    if result.is_err() {
        assert_eq!(result, Err(TagWriteError::Denied));
        assert_eq!(std::fs::read(&path).unwrap(), original);
        assert_eq!(names(dir.path()), ["x.wav"]);
    }
}

#[cfg(unix)]
#[test]
fn a_symlink_stays_a_symlink() {
    let dir = tempfile::tempdir().unwrap();
    let real = wav(dir.path(), "real.wav");
    let link = dir.path().join("link.wav");
    std::os::unix::fs::symlink(&real, &link).unwrap();
    write_tags(&link, &untagged_tags(&real), &full(), &limits()).unwrap();
    assert!(std::fs::symlink_metadata(&link).unwrap().file_type().is_symlink());
    assert_eq!(read_track_tags(&real, &limits()), full());
}

#[test]
fn a_huge_comment_is_cut_when_read() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "x.wav");
    let huge = TrackTags {
        comment: "x".repeat(5_000_000),
        ..TrackTags::default()
    };
    // A direct lofty write: `write_tags` would cut it first.
    {
        let mut file = lofty::read_from_path(&path).unwrap();
        let ty = file.primary_tag_type();
        file.insert_tag(Tag::new(ty));
        file.primary_tag_mut().unwrap().set_comment(huge.comment.clone());
        file.save_to_path(&path, WriteOptions::default()).unwrap();
    }
    let small = Limits {
        max_tag_chars: 100,
        ..Limits::default()
    };
    assert_eq!(read_track_tags(&path, &small).comment.chars().count(), 100);
}

#[test]
fn what_is_written_is_cut_to_the_limit() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "x.wav");
    let small = Limits {
        max_tag_chars: 10,
        ..Limits::default()
    };
    let after = TrackTags {
        comment: "y".repeat(50),
        ..untagged_tags(&path)
    };
    write_tags(&path, &untagged_tags(&path), &after, &small).unwrap();
    assert_eq!(read_track_tags(&path, &small).comment, "y".repeat(10));
}

fn tagged_by_date(path: &Path, date: &str) -> TrackTags {
    let before = untagged_tags(path);
    let after = TrackTags {
        date: Some(date.into()),
        ..before.clone()
    };
    write_tags(path, &before, &after, &limits()).unwrap();
    read_track_tags(path, &limits())
}

#[test]
fn a_full_date_is_written_and_read_back() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "x.wav");
    assert_eq!(
        tagged_by_date(&path, "2019-05-14").date.as_deref(),
        Some("2019-05-14")
    );
}

#[test]
fn a_bare_year_is_written_and_read_back() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "x.wav");
    assert_eq!(tagged_by_date(&path, "2019").date.as_deref(), Some("2019"));
}

#[test]
fn a_date_with_a_time_is_written_and_read_back() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "x.wav");
    assert_eq!(
        tagged_by_date(&path, "2019-05-14T08:30").date.as_deref(),
        Some("2019-05-14T08:30")
    );
}

#[test]
fn an_unrelated_edit_leaves_a_full_date_untouched() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "x.wav");
    let tagged = tagged_by_date(&path, "2019-05-14");
    let after = TrackTags {
        artist: "Someone".into(),
        ..tagged.clone()
    };
    write_tags(&path, &tagged, &after, &limits()).unwrap();
    let read = read_track_tags(&path, &limits());
    assert_eq!(read.date.as_deref(), Some("2019-05-14"));
    assert_eq!(read.artist, "Someone");
}

#[test]
fn clearing_the_date_removes_it() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "x.wav");
    let tagged = tagged_by_date(&path, "2019-05-14");
    let after = TrackTags {
        date: None,
        ..tagged.clone()
    };
    write_tags(&path, &tagged, &after, &limits()).unwrap();
    assert_eq!(read_track_tags(&path, &limits()).date, None);
}

#[test]
fn an_invalid_date_is_refused_and_the_file_is_untouched() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "x.wav");
    let original = std::fs::read(&path).unwrap();
    let before = untagged_tags(&path);
    let after = TrackTags {
        date: Some("last spring".into()),
        ..before.clone()
    };
    let result = write_tags(&path, &before, &after, &limits());
    assert_eq!(result, Err(TagWriteError::InvalidDate));
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert_eq!(names(dir.path()), ["x.wav"]);
}
```

`image` is already a dev-usable dependency of `fp-analysis` (normal dependency); `lofty` too.

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-analysis --test tags`
Expected: FAIL to compile (`fp_analysis::tags` not found).

- [ ] **Step 3: Implement**

`metadata.rs`: add to `Tags`

```rust
    /// The recording date, ISO 8601 text as `fp_model::parse_tag_date` accepts.
    pub date: Option<String>,
    pub genre: Option<String>,
    pub album_artist: Option<String>,
    pub composer: Option<String>,
    pub comment: Option<String>,
```

and, above `read_tags`, the helper (lofty's `Timestamp` prints as ISO 8601):

```rust
/// The recording date of `tag` as ISO 8601 text, if it is a valid one.
fn tag_date(tag: &lofty::tag::Tag) -> Option<String> {
    let text = tag.date()?.to_string();
    match fp_model::parse_tag_date(&text) {
        Ok(date) => date,
        Err(_) => {
            tracing::debug!("ignoring a tag date that is not ISO 8601: {text:?}");
            None
        }
    }
}
```

and in `read_tags`, where `Tags { title: ..., cover }` is built:

```rust
    Tags {
        title: non_empty(tag.title()),
        artist: non_empty(tag.artist()),
        album: non_empty(tag.album()),
        date: tag_date(tag),
        genre: non_empty(tag.genre()),
        album_artist: non_empty(tag.get_string(lofty::tag::ItemKey::AlbumArtist).map(Into::into)),
        composer: non_empty(tag.get_string(lofty::tag::ItemKey::Composer).map(Into::into)),
        comment: non_empty(tag.comment()),
        cover,
    }
```

(`Tags` derives `Default`, so the early returns stay as they are; `non_empty` takes `Option<Cow<str>>`, hence `.map(Into::into)` on the `&str`.)

`lib.rs`: add `pub mod tags;`.

Create `crates/fp-analysis/src/tags.rs`:

```rust
//! Reading and writing the text tags of a track (feedback 2 spec O23).
//!
//! Reading degrades to "nothing" like `metadata::read_tags`. Writing never
//! touches the original until the new file is complete: the tags go into a
//! copy next to it, which is synced and then renamed over the original.

use std::path::{Path, PathBuf};

use fp_model::{Limits, TrackTags};
use lofty::config::WriteOptions;
use lofty::file::{FileType, TaggedFileExt};
use lofty::prelude::*;
use lofty::tag::{ItemKey, Tag};

use crate::metadata::{read_tags, title_from_file_name};

/// Why tags could not be written. The original file is untouched in every
/// case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagWriteError {
    /// The format has no writable tags.
    Unsupported,
    /// The file or its folder is gone.
    NotFound,
    /// No permission to write there.
    Denied,
    /// The date is not an ISO 8601 date (`fp_model::parse_tag_date`).
    InvalidDate,
    /// Anything else, with the system's or lofty's own description.
    Other(String),
}

impl std::fmt::Display for TagWriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported => f.write_str("this format has no writable tags"),
            Self::NotFound => f.write_str("the file was not found"),
            Self::Denied => f.write_str("permission denied"),
            Self::InvalidDate => f.write_str("the date is not an ISO 8601 date"),
            Self::Other(detail) => f.write_str(detail),
        }
    }
}

impl From<std::io::Error> for TagWriteError {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::NotFound => Self::NotFound,
            std::io::ErrorKind::PermissionDenied => Self::Denied,
            _ => Self::Other(e.to_string()),
        }
    }
}

fn lofty_error(e: impl std::fmt::Display) -> TagWriteError {
    TagWriteError::Other(e.to_string())
}

/// The tags of `path` as the player shows them: the file name stands in for
/// a missing title (and its `Artist - ` for a missing artist), the other
/// fields stay empty. Never fails; a panic in the tag parser costs only the
/// tags.
pub fn read_track_tags(path: &Path, limits: &Limits) -> TrackTags {
    let tags = std::panic::catch_unwind(|| read_tags(path, limits)).unwrap_or_default();
    let (file_artist, file_title) = title_from_file_name(path);
    TrackTags {
        title: tags.title.unwrap_or(file_title),
        artist: tags.artist.or(file_artist).unwrap_or_default(),
        album: tags.album.unwrap_or_default(),
        album_artist: tags.album_artist.unwrap_or_default(),
        date: tags.date,
        genre: tags.genre.unwrap_or_default(),
        composer: tags.composer.unwrap_or_default(),
        comment: tags.comment.unwrap_or_default(),
    }
    .clamped(limits.max_tag_chars)
}

/// Whether the format of `path` has writable tags in lofty, judged from the
/// extension only: no I/O, so the interface can ask on its own thread.
pub fn can_write_tags(path: &Path) -> bool {
    FileType::from_path(path).is_some_and(|ft| ft.tag_support(ft.primary_tag_type()).is_writable())
}

/// Writes the fields where `after` differs from `before` into the file's
/// tags: text is set, an empty field removes the tag. Other fields, other
/// tags, covers and unknown items are kept.
///
/// 1. the file is copied to a temporary file in the same folder;
/// 2. the copy gets the new tags;
/// 3. the copy is synced to disk;
/// 4. the copy is renamed over the original.
///
/// Any error removes the copy and leaves the original as it was.
pub fn write_tags(
    path: &Path,
    before: &TrackTags,
    after: &TrackTags,
    limits: &Limits,
) -> Result<(), TagWriteError> {
    // A symlink is written through, not replaced.
    let real = path.canonicalize()?;
    if !can_write_tags(&real) {
        return Err(TagWriteError::Unsupported);
    }
    let after = after.clone().clamped(limits.max_tag_chars);
    let temp = temp_path(&real).ok_or_else(|| lofty_error("the file has no name"))?;
    let result = rewrite(&real, &temp, before, &after, limits);
    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    result
}

/// `name.fptag-<pid>.ext` next to `real`: the extension stays, so the format
/// is recognised the same way.
fn temp_path(real: &Path) -> Option<PathBuf> {
    let stem = real.file_stem()?.to_string_lossy();
    let name = match real.extension() {
        Some(ext) => format!("{stem}.fptag-{}.{}", std::process::id(), ext.to_string_lossy()),
        None => format!("{stem}.fptag-{}", std::process::id()),
    };
    Some(real.with_file_name(name))
}

fn rewrite(
    real: &Path,
    temp: &Path,
    before: &TrackTags,
    after: &TrackTags,
    limits: &Limits,
) -> Result<(), TagWriteError> {
    std::fs::copy(real, temp)?;
    // Parsed with its covers: if that fails the write fails, because saving
    // a tag read without them would drop them.
    let limit = usize::try_from(limits.max_cover_bytes)
        .unwrap_or(usize::MAX)
        .saturating_add(1024 * 1024)
        .max(16 * 1024 * 1024);
    lofty::config::apply_global_options(
        lofty::config::GlobalOptions::new().allocation_limit(limit),
    );
    let mut file = std::panic::catch_unwind(|| lofty::read_from_path(temp))
        .map_err(|_| lofty_error("the tag parser failed"))?
        .map_err(lofty_error)?;
    // The tag `read_tags` shows: the primary one, else the first.
    if file.primary_tag().is_none() && file.first_tag().is_none() {
        let ty = file.primary_tag_type();
        file.insert_tag(Tag::new(ty));
    }
    let tag = match file.primary_tag_mut() {
        Some(tag) => tag,
        None => file
            .first_tag_mut()
            .ok_or_else(|| lofty_error("the file has no tag to edit"))?,
    };
    change_tag(tag, before, after)?;
    file.save_to_path(temp, WriteOptions::default())
        .map_err(lofty_error)?;
    std::fs::OpenOptions::new()
        .write(true)
        .open(temp)?
        .sync_all()?;
    std::fs::rename(temp, real)?;
    Ok(())
}

fn change_tag(tag: &mut Tag, before: &TrackTags, after: &TrackTags) -> Result<(), TagWriteError> {
    fn text(
        tag: &mut Tag,
        old: &str,
        new: &str,
        set: fn(&mut Tag, String),
        remove: fn(&mut Tag),
    ) {
        if old == new {
            return;
        }
        if new.is_empty() {
            remove(tag);
        } else {
            set(tag, new.to_owned());
        }
    }
    fn item(tag: &mut Tag, key: ItemKey, old: &str, new: &str) {
        if old == new {
            return;
        }
        if new.is_empty() {
            tag.remove_key(key);
        } else {
            tag.insert_text(key, new.to_owned());
        }
    }
    text(tag, &before.title, &after.title, |t, v| t.set_title(v), |t| t.remove_title());
    text(tag, &before.artist, &after.artist, |t, v| t.set_artist(v), |t| t.remove_artist());
    text(tag, &before.album, &after.album, |t, v| t.set_album(v), |t| t.remove_album());
    text(tag, &before.genre, &after.genre, |t, v| t.set_genre(v), |t| t.remove_genre());
    text(tag, &before.comment, &after.comment, |t, v| t.set_comment(v), |t| t.remove_comment());
    item(tag, ItemKey::AlbumArtist, &before.album_artist, &after.album_artist);
    item(tag, ItemKey::Composer, &before.composer, &after.composer);
    if before.date != after.date {
        match &after.date {
            Some(text) => {
                let timestamp = text
                    .parse::<Timestamp>()
                    .map_err(|_| TagWriteError::InvalidDate)?;
                tag.set_date(timestamp);
            }
            None => tag.remove_date(),
        }
    }
    Ok(())
}
```

Notes for the implementer: (1) `file.primary_tag_mut()` then `file.first_tag_mut()` in the `match` can hit a borrow-checker limit; if it does, compute `let use_primary = file.primary_tag().is_some();` first and branch on it. (2) If `TaggedFileExt` / `AudioFile` imports are reported unused or missing, follow the compiler: the tests in `crates/fp-analysis/tests/metadata.rs` show the working `lofty::prelude::*` calls. (3) `tag.set_date`/`remove_date`/`date` are `Accessor` methods taking and giving `lofty::tag::items::Timestamp` (see `src/tag/items/timestamp.rs`: `FromStr` parses ISO 8601, `Display` writes it); if `remove_*` differ in this lofty version, check `~/.cargo/registry/src/*/lofty-0.25.4/src/tag/accessor.rs`. (4) A panic from `lofty::read_from_path` is caught; the `FileType` import is used by `can_write_tags`.

- [ ] **Step 4: Run the tests to verify they pass**

Run each: `cargo test -p fp-analysis --test tags`, then `cargo test -p fp-analysis` (the existing `metadata` and `analyze` tests must still pass), then `cargo deny check`.
Expected: PASS; `cargo deny check` reports no new advisory, licence or source (lofty and `image` are already allowed).

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-analysis
git commit -m "feat(analysis): read more tags and write them through a synced copy"
```

---

### Task 3: The tag worker and the tag-only pass (`fp-app` services)

**Files:**
- Create: `crates/fp-app/src/tags.rs`
- Modify: `crates/fp-app/src/lib.rs` (`pub mod tags;`)
- Modify: `crates/fp-app/src/services.rs` (`Services` fields, `new`, `analysis_step`, a new `tag_pass`)
- Test: Modify `crates/fp-app/tests/services.rs`

**Interfaces:**
- Consumes: `fp_analysis::tags::{read_track_tags, write_tags, TagWriteError}`, `Track::needs_tag_read`, `Command::ApplyTags`, `ConductorHandle::send`.
- Produces (Tasks 5 and 6 rely on these):
  - `pub enum TagJob { Read { track: TrackId, path: PathBuf, limits: Limits }, Write { track: TrackId, path: PathBuf, before: Box<TrackTags>, after: Box<TrackTags>, limits: Limits } }`.
  - `pub enum TagOutcome { Read { track: TrackId, tags: TrackTags }, Written { track: TrackId, result: Result<TrackTags, TagWriteError> } }` (`Written` carries the tags read back from the file).
  - `pub struct TagWorker` with `TagWorker::spawn(repaint: Box<dyn Fn() + Send>) -> std::io::Result<TagWorker>`, `fn submit(&self, job: TagJob) -> bool` (never blocks; `false` if the worker is gone), `fn results(&self) -> &Receiver<TagOutcome>`. Dropping it ends the thread after the job in progress.

The file's `Rig` (`rig`, `rig_with(files, dir, delay, prepare)`, `Rig::run_until(what, done)`, `wav(dir, name, secs)`, `outdated_rig`) already ticks the conductor, steps the services and renders the offline device; the tests below use it unchanged.

- [ ] **Step 1: Write the failing tests**

In `crates/fp-app/tests/services.rs` add:

```rust
/// Gives `path` a title, a genre and a comment, as a tagging tool would.
fn tag_file(path: &Path) {
    use lofty::prelude::*;
    let mut file = lofty::read_from_path(path).unwrap();
    let ty = file.primary_tag_type();
    file.insert_tag(lofty::tag::Tag::new(ty));
    let tag = file.primary_tag_mut().unwrap();
    tag.set_title("Tagged".into());
    tag.set_genre("Jazz".into());
    tag.set_comment("Take two".into());
    file.save_to_path(path, lofty::config::WriteOptions::default())
        .unwrap();
}

fn only_track(r: &Rig) -> fp_model::Track {
    r.handle.model.load().library.iter().next().unwrap().clone()
}

#[test]
fn the_tag_pass_fills_the_tags_of_analysed_tracks_once() {
    let dir = tempfile::tempdir().unwrap();
    let a = wav(dir.path(), "a.wav", 1);
    tag_file(&a);
    let mut r = rig(&[a], dir);
    r.run_until("the tags", |r| only_track(r).tags_read);
    let t = only_track(&r);
    assert_eq!(
        (t.title.as_str(), t.genre.as_str(), t.comment.as_str()),
        ("Tagged", "Jazz", "Take two")
    );
    assert!(t.analyzed);
    let version = r.handle.telemetry.load().model_version;
    let until = r.now + Duration::from_millis(500);
    r.run_until("a quiet period", |r| r.now >= until);
    assert_eq!(
        r.handle.telemetry.load().model_version,
        version,
        "no further ApplyTags once the tags are read"
    );
    assert_eq!(r.analyses.load(Ordering::SeqCst), 1, "no second analysis");
}

#[test]
fn a_track_whose_tags_cannot_be_read_is_not_asked_again() {
    let dir = tempfile::tempdir().unwrap();
    let junk = dir.path().join("Band - Junk.wav");
    std::fs::write(&junk, b"this is not audio").unwrap();
    // Analysed already (so no analysis runs), with its tags still unread.
    let mut r = rig_with(&[junk], dir, Duration::ZERO, |state| {
        for track in state.library.iter_mut() {
            track.analyzed = true;
            track.format = Some(fp_model::AudioFormat {
                sample_rate: 44_100,
                bits: Some(16),
                channels: 2,
            });
            track.analysis_version = fp_analysis::cache::ANALYSIS_VERSION;
        }
    });
    r.run_until("the pass to answer", |r| only_track(r).tags_read);
    let t = only_track(&r);
    assert_eq!((t.title.as_str(), t.artist.as_str()), ("Junk", "Band"));
    let version = r.handle.telemetry.load().model_version;
    let until = r.now + Duration::from_millis(500);
    r.run_until("a quiet period", |r| r.now >= until);
    assert_eq!(r.handle.telemetry.load().model_version, version);
}

#[test]
fn a_missing_track_is_not_read() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = rig_with(
        &[PathBuf::from("/definitely/missing.flac")],
        dir,
        Duration::ZERO,
        |state| {
            for track in state.library.iter_mut() {
                track.analyzed = true;
                track.file_state = FileState::Missing;
            }
        },
    );
    let until = r.now + Duration::from_millis(500);
    r.run_until("a quiet period", |r| r.now >= until);
    assert!(!only_track(&r).tags_read);
}

#[test]
fn a_new_analysis_result_brings_the_tag_pass_back() {
    let dir = tempfile::tempdir().unwrap();
    let a = wav(dir.path(), "a.wav", 1);
    tag_file(&a);
    let mut r = rig(&[a.clone()], dir);
    r.run_until("the tags", |r| only_track(r).tags_read);
    // The operator edits the genre with another tool, then asks for a new
    // analysis: the file's new tags must reach the library too.
    {
        use lofty::prelude::*;
        let mut file = lofty::read_from_path(&a).unwrap();
        file.primary_tag_mut().unwrap().set_genre("Blues".into());
        file.save_to_path(&a, lofty::config::WriteOptions::default())
            .unwrap();
    }
    let track = only_track(&r).id;
    r.handle.send(Command::ApplyAnalysis {
        track,
        analysis: Box::new(fp_model::TrackAnalysis::default()),
    });
    r.run_until("the genre read again", |r| only_track(r).genre == "Blues");
    assert!(only_track(&r).tags_read);
}
```

(`hound` is already a dev-dependency of `fp-app`; `lofty` is not, so add `lofty.workspace = true` under `[dev-dependencies]` in `crates/fp-app/Cargo.toml`, and `tempfile.workspace = true` too if it is missing. Task 6's tests need the same two.)

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-app --test services the_tag_pass_fills_the_tags_of_analysed_tracks_once`
Expected: FAIL (the tags stay empty and `tags_read` stays false).

- [ ] **Step 3: Implement**

Create `crates/fp-app/src/tags.rs`:

```rust
//! The tag worker (feedback 2 spec O23): one thread that reads and writes
//! tags, so neither the interface nor the services thread waits on a disk.
//! It follows the file probe's pattern: jobs in, outcomes out, both on
//! unbounded channels, and the thread ends when its handle is dropped.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;
use std::thread::JoinHandle;

use crossbeam_channel::{Receiver, Sender};
use fp_analysis::tags::{TagWriteError, read_track_tags, write_tags};
use fp_model::{Limits, TrackId, TrackTags};

/// One piece of work for the worker.
#[derive(Debug, Clone)]
pub enum TagJob {
    /// Read the tags of a file (the tag-only pass).
    Read {
        track: TrackId,
        path: PathBuf,
        limits: Limits,
    },
    /// Write the fields where `after` differs from `before`, then read the
    /// file again.
    Write {
        track: TrackId,
        path: PathBuf,
        before: Box<TrackTags>,
        after: Box<TrackTags>,
        limits: Limits,
    },
}

/// What a job produced.
#[derive(Debug, Clone, PartialEq)]
pub enum TagOutcome {
    Read {
        track: TrackId,
        tags: TrackTags,
    },
    /// `Ok` carries the tags as the file holds them after the write.
    Written {
        track: TrackId,
        result: Result<TrackTags, TagWriteError>,
    },
}

fn run(job: TagJob) -> TagOutcome {
    match job {
        TagJob::Read {
            track,
            path,
            limits,
        } => TagOutcome::Read {
            track,
            tags: read_track_tags(&path, &limits),
        },
        TagJob::Write {
            track,
            path,
            before,
            after,
            limits,
        } => {
            let result = write_tags(&path, &before, &after, &limits)
                .map(|()| read_track_tags(&path, &limits));
            TagOutcome::Written { track, result }
        }
    }
}

/// A panic inside a job is a failed job, not a dead worker.
fn run_contained(job: TagJob) -> TagOutcome {
    let (track, write) = match &job {
        TagJob::Read { track, .. } => (*track, false),
        TagJob::Write { track, .. } => (*track, true),
    };
    catch_unwind(AssertUnwindSafe(|| run(job))).unwrap_or_else(|_| {
        tracing::error!(?track, "a tag job panicked");
        if write {
            TagOutcome::Written {
                track,
                result: Err(TagWriteError::Other("the tag code failed".to_owned())),
            }
        } else {
            TagOutcome::Read {
                track,
                tags: TrackTags::default(),
            }
        }
    })
}

pub struct TagWorker {
    jobs: Option<Sender<TagJob>>,
    results: Receiver<TagOutcome>,
    thread: Option<JoinHandle<()>>,
}

impl TagWorker {
    /// Starts the thread. `repaint` runs after every outcome (the interface
    /// passes its context's repaint request).
    pub fn spawn(repaint: Box<dyn Fn() + Send>) -> std::io::Result<Self> {
        let (jobs_tx, jobs_rx) = crossbeam_channel::unbounded::<TagJob>();
        let (results_tx, results) = crossbeam_channel::unbounded();
        let thread = std::thread::Builder::new()
            .name("fp-tags".to_owned())
            .spawn(move || {
                while let Ok(job) = jobs_rx.recv() {
                    if results_tx.send(run_contained(job)).is_err() {
                        break;
                    }
                    repaint();
                }
            })?;
        Ok(Self {
            jobs: Some(jobs_tx),
            results,
            thread: Some(thread),
        })
    }

    /// Queues a job; never blocks. `false` if the worker is gone.
    pub fn submit(&self, job: TagJob) -> bool {
        self.jobs.as_ref().is_some_and(|tx| tx.send(job).is_ok())
    }

    pub fn results(&self) -> &Receiver<TagOutcome> {
        &self.results
    }
}

impl Drop for TagWorker {
    fn drop(&mut self) {
        // Closing the channel ends the loop after the job in progress.
        self.jobs = None;
        if let Some(thread) = self.thread.take()
            && thread.join().is_err()
        {
            tracing::error!("the tag worker panicked");
        }
    }
}
```

`lib.rs`: add `pub mod tags;` (alphabetical, after `services`).

`services.rs`: import `crate::tags::{TagJob, TagOutcome, TagWorker}` and `fp_model::TrackTags` is not needed. Add fields to `Services`:

```rust
    /// Reads the tags of tracks that wait for the tag-only pass (feedback 2
    /// spec O23); `None` if its thread could not start.
    tag_worker: Option<TagWorker>,
    /// Tracks sent to it and not answered yet.
    tags_in_flight: HashSet<TrackId>,
```

In `new`: `tag_worker: TagWorker::spawn(Box::new(|| {})).map_err(|e| tracing::error!(error = %e, "cannot start the tag worker; tags stay unread")).ok(), tags_in_flight: HashSet::new(),`. In `analysis_step`, after `self.submit_new(state);` call `self.tag_pass(state);` and add:

```rust
    /// The tag-only pass: tracks analysed but not read since (new ones, ones
    /// of an older library, re-analysed ones) get their tags read on the tag
    /// worker. A read that cannot parse the file still answers, with the
    /// file-name title and empty fields, so a track is never asked twice for
    /// the same analysis.
    fn tag_pass(&mut self, state: &AppState) {
        let Some(worker) = &self.tag_worker else {
            return;
        };
        self.tags_in_flight
            .retain(|id| state.library.get(*id).is_some());
        let limits = &state.config.limits;
        for track in state.library.iter() {
            if track.needs_tag_read() && self.tags_in_flight.insert(track.id) {
                let job = TagJob::Read {
                    track: track.id,
                    path: track.path.clone(),
                    limits: limits.clone(),
                };
                if !worker.submit(job) {
                    tracing::error!("the tag worker stopped; tags stay unread");
                    self.tag_worker = None;
                    return;
                }
            }
        }
        let outcomes: Vec<TagOutcome> = worker.results().try_iter().collect();
        for outcome in outcomes {
            if let TagOutcome::Read { track, tags } = outcome {
                // A full queue: forget it, the next round asks again.
                if self.conductor.send(Command::ApplyTags {
                    track,
                    tags: Box::new(tags),
                }) {
                    self.tags_in_flight.remove(&track);
                } else {
                    self.tags_in_flight.remove(&track);
                }
            }
        }
    }
```

Simplify the duplicated branch while writing it: in both cases the id leaves `tags_in_flight`; when the send fails the track still `needs_tag_read`, so the next round submits it again, which is the intended retry. Borrow note: take `let limits = state.config.limits.clone();` before borrowing `self.tag_worker`, and collect the outcomes before calling `self.conductor.send`, if the borrow checker asks.

- [ ] **Step 4: Run the tests to verify they pass**

Run each of the four tests by name with `cargo test -p fp-app --test services <name>`, then `cargo test -p fp-app --test services` for the whole file (the other rigs must still pass: an `ApplyTags` more does not change their model versions' meaning).
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-app
git commit -m "feat(app): a tag worker and the tag-only pass for tracks with unread tags"
```

---

### Task 4: The row tooltip (UI)

**Files:**
- Modify: `crates/fp-app/src/ui/view.rs` (new pure `TrackTip`, `track_tooltip`)
- Modify: `crates/fp-app/src/ui/table.rs` (show it on the row response, near line 339)
- Modify: `crates/fp-app/locales/en-US/main.ftl`, `es-ES/main.ftl`
- Test: Modify `crates/fp-app/tests/view.rs`; create `crates/fp-app/tests/track_tags_ui.rs`

**Interfaces:**
- Consumes: `fp_model::Track` (tag fields from Task 1, `format: Option<AudioFormat>`, `duration_secs`, `path`).
- Produces:
  - `pub enum TipField { Title, Artist, Album, Date, Genre, Duration, Format, Path }` (`Debug, Clone, Copy, PartialEq, Eq`).
  - `pub fn track_tooltip(track: &Track) -> Vec<(TipField, String)>`: only the fields the track has, in that order. Duration as `format::clock`, format as `FLAC · 44.1 kHz · 16 bit` (codec from the upper-cased extension, rate in kHz with at most one decimal, bit depth only when known; any part that is unknown is left out; no line when nothing is known), path as `track.path.display()`.
  - Locale keys `tip-field-title`, `tip-field-artist`, `tip-field-album`, `tip-field-date`, `tip-field-genre`, `tip-field-duration`, `tip-field-format`, `tip-field-path`, and `tip-format-rate` (`{ $value } kHz`), `tip-format-bits` (`{ $bits } bit`). Plan 8's Date and Genre column headers can reuse the first labels.

- [ ] **Step 1: Write the failing tests**

In `crates/fp-app/tests/view.rs` add (importing `fp_app::ui::view::{TipField, track_tooltip}` and `fp_model::{AudioFormat, Track}`):

```rust
fn tip_track() -> Track {
    let mut t = Track::new(fp_model::TrackId(1), PathBuf::from("/m/Artist - Song.flac"));
    t.title = "Song".into();
    t.artist = "Artist".into();
    t.album = "Album".into();
    t.date = Some("1999-03-07".into());
    t.genre = "Pop".into();
    t.duration_secs = 200.0;
    t.format = Some(AudioFormat {
        sample_rate: 44_100,
        bits: Some(16),
        channels: 2,
    });
    t
}

#[test]
fn the_tooltip_lists_what_the_track_has_in_order() {
    let tip = track_tooltip(&tip_track());
    let fields: Vec<TipField> = tip.iter().map(|(f, _)| *f).collect();
    assert_eq!(
        fields,
        [
            TipField::Title,
            TipField::Artist,
            TipField::Album,
            TipField::Date,
            TipField::Genre,
            TipField::Duration,
            TipField::Format,
            TipField::Path
        ]
    );
    let value = |f| tip.iter().find(|(g, _)| *g == f).unwrap().1.clone();
    assert_eq!(value(TipField::Date), "1999-03-07");
    assert_eq!(value(TipField::Duration), "03:20");
    assert_eq!(value(TipField::Format), "FLAC · 44.1 kHz · 16 bit");
    assert_eq!(value(TipField::Path), "/m/Artist - Song.flac");
}

#[test]
fn a_missing_field_is_left_out() {
    let mut t = tip_track();
    t.album.clear();
    t.date = None;
    t.genre.clear();
    t.duration_secs = 0.0;
    t.format = None;
    let fields: Vec<TipField> = track_tooltip(&t).iter().map(|(f, _)| *f).collect();
    assert_eq!(fields, [TipField::Title, TipField::Artist, TipField::Path]);
}

#[test]
fn the_format_line_leaves_out_what_is_unknown() {
    let mut t = tip_track();
    t.format = Some(AudioFormat {
        sample_rate: 48_000,
        bits: None,
        channels: 2,
    });
    let line = track_tooltip(&t)
        .into_iter()
        .find(|(f, _)| *f == TipField::Format)
        .unwrap()
        .1;
    assert_eq!(line, "FLAC · 48 kHz", "lossy: no bit depth; whole kHz without a decimal");
    t.path = PathBuf::from("/m/no extension");
    t.format = None;
    assert!(track_tooltip(&t).iter().all(|(f, _)| *f != TipField::Format));
}
```

Create `crates/fp-app/tests/track_tags_ui.rs`:

```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Feedback 2 spec O23: the row tooltip and the tag editor.

mod support;

use std::path::{Path, PathBuf};

use fp_model::{AppState, AudioFormat, FileState};
use support::{harness, state};

/// `state(1, 3)` whose first track has tags, a format and a real path.
fn tagged_state(path: Option<&Path>) -> AppState {
    let mut s = state(1, 3);
    let first = s.library.iter().next().unwrap().id;
    let t = s.library.get_mut(first).unwrap();
    t.title = "Song 1".into();
    t.artist = "The Artist".into();
    t.album = "An Album".into();
    t.date = Some("1999-03-07".into());
    t.genre = "Pop".into();
    t.duration_secs = 200.0;
    t.analyzed = true;
    t.tags_read = true;
    t.file_state = FileState::Ok;
    t.format = Some(AudioFormat {
        sample_rate: 44_100,
        bits: Some(16),
        channels: 2,
    });
    if let Some(path) = path {
        t.path = PathBuf::from(path);
    }
    s
}

#[test]
fn hovering_a_row_shows_the_tags_format_and_path() {
    let (mut h, _fake) = harness(tagged_state(None));
    h.get_by_label("Song 1").hover();
    h.run_steps(40);
    for text in [
        "The Artist",
        "An Album",
        "1999",
        "Pop",
        "03:20",
        "MP3 · 44.1 kHz · 16 bit",
        "/music/Song 1.mp3",
    ] {
        assert!(
            h.query_by_label_contains(text).is_some(),
            "the tooltip shows {text}"
        );
    }
}

#[test]
fn the_tooltip_waits_for_the_usual_delay() {
    let (mut h, _fake) = harness(tagged_state(None));
    h.get_by_label("Song 1").hover();
    h.run_steps(2);
    assert!(h.query_by_label_contains("An Album").is_none());
}
```

(The row's own title label is "Song 1"; the tooltip's labels are separate nodes. If the row exposes the title text in a way that makes `get_by_label("Song 1")` ambiguous once the tooltip is up, query the tooltip's other lines only, as above.)

- [ ] **Step 2: Run the tests to verify they fail**

Run each: `cargo test -p fp-app --test view the_tooltip_lists_what_the_track_has_in_order` and `cargo test -p fp-app --test track_tags_ui hovering_a_row_shows_the_tags_format_and_path`
Expected: FAIL to compile / FAIL (no tooltip).

- [ ] **Step 3: Implement**

`view.rs` (it has no i18n; the values are language-neutral, the labels come from the caller):

```rust
/// A line of the row tooltip (feedback 2 spec O23).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipField {
    Title,
    Artist,
    Album,
    Date,
    Genre,
    Duration,
    Format,
    Path,
}

/// What the row tooltip shows for `track`: the fields it has, in this
/// order. Album artist, composer and comment are left to the editor.
pub fn track_tooltip(track: &Track) -> Vec<(TipField, String)> {
    let mut lines = Vec::new();
    let mut text = |field, value: &str| {
        if !value.is_empty() {
            lines.push((field, value.to_owned()));
        }
    };
    text(TipField::Title, &track.title);
    text(TipField::Artist, &track.artist);
    text(TipField::Album, &track.album);
    text(
        TipField::Date,
        track.date.as_deref().unwrap_or_default(),
    );
    text(TipField::Genre, &track.genre);
    if track.duration_secs.is_finite() && track.duration_secs > 0.0 {
        text(TipField::Duration, &super::format::clock(track.duration_secs));
    }
    text(TipField::Format, &format_line(track));
    text(TipField::Path, &track.path.display().to_string());
    lines
}

/// `FLAC · 44.1 kHz · 16 bit`: the codec is the file extension, in capitals.
fn format_line(track: &Track) -> String {
    let mut parts = Vec::new();
    if let Some(ext) = track.path.extension().and_then(|e| e.to_str()) {
        parts.push(ext.to_uppercase());
    }
    if let Some(f) = track.format {
        if f.sample_rate > 0 {
            let khz = f64::from(f.sample_rate) / 1000.0;
            let text = if (khz.fract()).abs() < 1e-9 {
                format!("{khz:.0}")
            } else {
                format!("{khz:.1}")
            };
            parts.push(format!("{text} kHz"));
        }
        if let Some(bits) = f.bits {
            parts.push(format!("{bits} bit"));
        }
    }
    parts.join(" · ")
}
```

The tests above assert the English unit text `kHz` and `bit`, which are language-neutral SI/technical abbreviations, so they stay in code and the planned `tip-format-rate` and `tip-format-bits` keys are not needed: drop them from the locale additions. Only the field labels are Fluent messages.

`table.rs`: after the `response.double_clicked()` handling (near line 342) add a tooltip on the row. `on_hover_ui` respects egui's tooltip delay:

```rust
                response.clone().on_hover_ui(|ui| {
                    track_tip(ui, scene, track);
                });
```

and the helper at the end of the file:

```rust
/// The row tooltip: label and value per line, at most as wide as the window.
fn track_tip(ui: &mut Ui, scene: &Scene<'_>, track: &fp_model::Track) {
    let t = scene.i18n;
    ui.set_max_width(420.0);
    egui::Grid::new("track-tip")
        .num_columns(2)
        .spacing(vec2(10.0, 3.0))
        .show(ui, |ui| {
            for (field, value) in view::track_tooltip(track) {
                let key = match field {
                    view::TipField::Title => "tip-field-title",
                    view::TipField::Artist => "tip-field-artist",
                    view::TipField::Album => "tip-field-album",
                    view::TipField::Date => "tip-field-date",
                    view::TipField::Genre => "tip-field-genre",
                    view::TipField::Duration => "tip-field-duration",
                    view::TipField::Format => "tip-field-format",
                    view::TipField::Path => "tip-field-path",
                };
                ui.label(
                    RichText::new(t.tr(key))
                        .font(font(11.0))
                        .color(theme::NEUTRAL_400),
                );
                ui.add(
                    egui::Label::new(RichText::new(value).font(font(12.0)).color(theme::TEXT))
                        .wrap(),
                );
                ui.end_row();
            }
        });
}
```

(`track` is already in scope in the row closure, where the title is drawn; `.clone()` on the `Response` is cheap. If `on_hover_ui` on the row also shows when the pointer is over the small icons, which have their own tooltip, the icon's tooltip is the one egui shows because the icon is the hovered widget; check with the existing `hovering_an_unavailable_row_says_why` test, which must keep passing.)

Locale additions, `en-US/main.ftl` (after `tip-on-air-elsewhere`):

```
tip-field-title = Title
tip-field-artist = Artist
tip-field-album = Album
tip-field-date = Date
tip-field-genre = Genre
tip-field-duration = Duration
tip-field-format = Format
tip-field-path = Path
```

`es-ES/main.ftl`: `Título`, `Artista`, `Álbum`, `Fecha`, `Género`, `Duración`, `Formato`, `Ruta`.

- [ ] **Step 4: Run the tests to verify they pass**

Run each of the new tests by name, then `cargo test -p fp-app --test main_screen`, `cargo test -p fp-app --test i18n`.
Expected: PASS (the earlier hover tests still pass).

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-app
git commit -m "feat(ui): a tooltip with the tags, format and path on each table row"
```

---

### Task 5: The tag sheet (model, analysis and worker)

The editor shows a **sheet**: every field of the spec's "Editor" bullet (10 always shown, 26 optional), read from the file and written back through the safe copy of Task 2. This task builds the sheet and everything under it, with no UI. Three parts, one commit each: the pure model (`fp-model`), the file reader and writer (`fp-analysis`), and the worker jobs (`fp-app`).

lofty 0.25.4 facts the code relies on (checked against `src/tag/item.rs`, `src/tag/mod.rs`, `src/id3/v2/tag.rs` and `src/id3/v2/tag/conversion.rs`, and by the round-trip tests below; every call in this task exists in that version):

- **Keys.** `lofty::tag::ItemKey` has one variant per field: `TrackTitle`, `TrackArtist`, `AlbumTitle`, `AlbumArtist`, `RecordingDate`, `TrackNumber`/`TrackTotal`, `DiscNumber`/`DiscTotal`, `Genre`, `Composer`, `Comment`, `TrackSubtitle`, `ContentGroup` (grouping), `IntegerBpm` and `Bpm`, `InitialKey`, `Mood`, `Isrc`, `Publisher`, `CatalogNumber`, `CopyrightMessage`, `OriginalArtist`, `OriginalAlbumTitle`, `OriginalReleaseDate`, `Lyricist`, `Conductor`, `Remixer`, `Arranger`, `Performer`, `Language`, `EncodedBy`, `Lyrics` and `UnsyncLyrics`, `TrackTitleSortOrder`, `TrackArtistSortOrder`, `AlbumTitleSortOrder`, `AlbumArtistSortOrder`, `ComposerSortOrder`, `TrackArtistUrl` (artist website). `ItemKey::supported_keys(tag_type: TagType) -> &'static [ItemKey]` lists what a tag type can store (it also covers ID3v1, which `ItemKey::map_key` does not).
- **Format differences.** ID3v2 stores BPM only as `IntegerBpm` (`TBPM`) and lyrics only as `UnsyncLyrics` (`USLT`) and has no performer frame; Vorbis comments store `Bpm` and `Lyrics`; RIFF INFO stores 11 of the 36 fields (no album artist, no disc number); AIFF text chunks store four (title, artist, comment, copyright). A field therefore tries its `ItemKey`s in order and uses the first the tag type supports.
- **Numbers.** `TrackNumber` and `TrackTotal` are two items that ID3v2 merges into one `TRCK` frame (`n/t`), `DiscNumber` and `DiscTotal` into `TPOS`. Writing a total without its number makes lofty write number `0` (observed), so the sheet refuses a total without a number.
- **Several values.** `Tag::push(TagItem)` appends without replacing, so several items can share one `ItemKey`. ID3v2.4 joins them into one frame with its multi-value separator and reads them back as separate items; Vorbis writes one comment per value; RIFF INFO writes repeated chunks (a round-trip test below proves it). `Tag::insert_text` and `Tag::insert` replace every item of the key, so the sheet uses `Tag::take_filter` and `Tag::push` instead.
- **Items with a description.** An ID3v2 comment can carry a description (`TagItem::description()`); the plain comment is the one with an empty description. The sheet owns only items whose description is empty and leaves the others alone.
- **Unknown items.** A parsed `Tag` keeps the frames lofty cannot map (for example a custom `TXXX`) in a companion tag (`GlobalOptions::preserve_format_specific_items` is on by default, `Tag::has_format_specific_items()` tells) and writes them back untouched.
- **A lofty 0.25.4 defect.** ID3v2 URL frames that lofty maps (`WOAR`, `WCOM`, ...) are parsed as `ItemValue::Locator`, and saving a `Tag` drops locators (its frame builder only accepts `ItemValue::Text`; observed with a probe). The shared writer therefore re-adds every ID3v2 locator as text before editing (`urls_as_text`), which makes lofty write them back as URL frames. Without it a save would delete URLs the editor does not show.

**Files:**
- Create: `crates/fp-model/src/tag_sheet.rs`
- Modify: `crates/fp-model/src/track.rs` (`cut` becomes `pub(crate)`), `lib.rs` (module and exports), `config.rs` (`Limits::max_tag_values`)
- Test: Create `crates/fp-model/tests/tag_sheet.rs`
- Modify: `crates/fp-analysis/src/tags.rs`, `crates/fp-analysis/src/metadata.rs` (`tag_date` becomes `pub(crate)`)
- Test: Create `crates/fp-analysis/tests/tag_sheet.rs` (`tests/tags.rs` must keep passing unchanged)
- Modify: `crates/fp-app/src/tags.rs` (jobs, outcomes, tests)

**Interfaces:**
- Consumes: `fp_model::{parse_tag_date, Limits, TrackTags}`, `fp_analysis::tags::{can_write_tags, read_track_tags, write_tags, TagWriteError}` (Task 2), `TagWorker` (Task 3).
- Produces (Task 6 relies on these exact names):
  - `fp_model::TagField`: 36 variants in the shown order (`Title, Artist, Album, AlbumArtist, Date, TrackNumber, DiscNumber, Genre, Composer, Comment`, then `Subtitle, Grouping, Bpm, InitialKey, Mood, Isrc, Publisher, CatalogNumber, Copyright, OriginalArtist, OriginalAlbum, OriginalReleaseDate, Lyricist, Conductor, Remixer, Arranger, Performer, Language, EncodedBy, Lyrics, SortTitle, SortArtist, SortAlbum, SortAlbumArtist, SortComposer, ArtistWebsite`); `Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash`; `TagField::ALL: [TagField; 36]`; `fn always_shown(self) -> bool`; `fn kind(self) -> TagFieldKind`; `fn is_multi_value(self) -> bool`; `fn slug(self) -> &'static str` (`"album-artist"`, ...; the locale key is `tag-field-<slug>`).
  - `fp_model::TagFieldKind { Text, LongText, Date, Pair, Whole }`.
  - `fp_model::TagSheet` (`Debug, Clone, Default, PartialEq, Eq`) with `pub other_kept: usize`, `pub other_kept_more: bool` and `new(storable: impl IntoIterator<Item = TagField>, other_kept: usize, other_kept_more: bool)`, `values(field) -> &[String]`, `set_values(field, Vec<String>)`, `can_store(field) -> bool`, `has(field) -> bool`, `text(field) -> String`, `set_text(field, &str, as_lines: bool)`, `pair(field) -> (String, String)`, `set_pair(field, &str, &str)`, `shows_lines(field) -> bool`, `visible_fields(&BTreeSet<TagField>) -> Vec<TagField>`, `addable_fields(&BTreeSet<TagField>) -> Vec<TagField>`, `clamped(self, max_chars: usize, max_values: usize) -> TagSheet`.
  - `fp_model::{changed_fields, invalid_fields, unstored_fields}`: `fn changed_fields(before: &TagSheet, after: &TagSheet) -> Vec<TagField>`, `fn invalid_fields(before: &TagSheet, after: &TagSheet) -> Vec<TagField>`, `fn unstored_fields(before: &TagSheet, after: &TagSheet, read_back: &TagSheet) -> Vec<TagField>`.
  - `Limits::max_tag_values: usize` (default 32, range 1..=1000).
  - `fp_analysis::tags::storable_fields(tag_type: lofty::tag::TagType) -> Vec<TagField>`, `read_tag_sheet(path: &Path, limits: &Limits) -> Option<TagSheet>` (`None`: no writable tags or unparsable file), `write_tag_sheet(path: &Path, before: &TagSheet, after: &TagSheet, limits: &Limits) -> Result<(), TagWriteError>`, and `TagWriteError::InvalidField(TagField)`.
  - `fp_app::tags::{TagJob::ReadSheet { track, path, limits }, TagJob::WriteSheet { track, path, before: Box<TagSheet>, after: Box<TagSheet>, limits }, TagOutcome::SheetRead { track, sheet: Option<Box<TagSheet>> }, TagOutcome::SheetWritten { track, result: Result<Box<SheetSaved>, TagWriteError> }, SheetSaved { tags: TrackTags, sheet: Option<TagSheet> }}`.

#### Part A: the sheet in `fp-model`

- [ ] **Step 1: Write the failing tests**

Create `crates/fp-model/tests/tag_sheet.rs`:

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback 2 spec O23: the tag sheet, its fields and its pure rules.

use std::collections::BTreeSet;

use fp_model::{
    Config, TagField, TagFieldKind, TagSheet, changed_fields, invalid_fields, unstored_fields,
};

fn all_storable() -> TagSheet {
    TagSheet::new(TagField::ALL, 0, false)
}

fn lines(values: &[&str]) -> Vec<String> {
    values.iter().map(|v| (*v).to_owned()).collect()
}

#[test]
fn there_are_exactly_the_fields_of_the_spec_in_order() {
    let labels: Vec<&str> = TagField::ALL.iter().map(|f| f.slug()).collect();
    assert_eq!(
        labels,
        [
            "title",
            "artist",
            "album",
            "album-artist",
            "date",
            "track-number",
            "disc-number",
            "genre",
            "composer",
            "comment",
            "subtitle",
            "grouping",
            "bpm",
            "initial-key",
            "mood",
            "isrc",
            "publisher",
            "catalog-number",
            "copyright",
            "original-artist",
            "original-album",
            "original-release-date",
            "lyricist",
            "conductor",
            "remixer",
            "arranger",
            "performer",
            "language",
            "encoded-by",
            "lyrics",
            "sort-title",
            "sort-artist",
            "sort-album",
            "sort-album-artist",
            "sort-composer",
            "artist-website",
        ]
    );
    let unique: BTreeSet<_> = TagField::ALL.iter().collect();
    assert_eq!(unique.len(), 36);
    let mut sorted = TagField::ALL;
    sorted.sort();
    assert_eq!(
        sorted,
        TagField::ALL,
        "the derived order is the shown order"
    );
}

#[test]
fn ten_fields_are_always_shown_and_the_first_ten() {
    let always: Vec<TagField> = TagField::ALL
        .into_iter()
        .filter(|f| f.always_shown())
        .collect();
    assert_eq!(always, TagField::ALL[..10]);
}

#[test]
fn field_kinds_follow_the_spec() {
    assert_eq!(TagField::Date.kind(), TagFieldKind::Date);
    assert_eq!(TagField::OriginalReleaseDate.kind(), TagFieldKind::Date);
    assert_eq!(TagField::TrackNumber.kind(), TagFieldKind::Pair);
    assert_eq!(TagField::DiscNumber.kind(), TagFieldKind::Pair);
    assert_eq!(TagField::Bpm.kind(), TagFieldKind::Whole);
    assert_eq!(TagField::Lyrics.kind(), TagFieldKind::LongText);
    assert_eq!(TagField::Comment.kind(), TagFieldKind::LongText);
    assert_eq!(TagField::Title.kind(), TagFieldKind::Text);
    assert!(TagField::Artist.is_multi_value());
    assert!(!TagField::Title.is_multi_value());
    assert!(!TagField::Lyrics.is_multi_value());
}

#[test]
fn values_are_tidied_when_set() {
    let mut s = all_storable();
    s.set_values(TagField::Artist, lines(&["  A  ", "", "B"]));
    assert_eq!(s.values(TagField::Artist), ["A", "B"]);
    s.set_values(TagField::Artist, lines(&["", "  "]));
    assert!(!s.has(TagField::Artist));
    s.set_text(TagField::Genre, "Pop\n\n Rock \n", true);
    assert_eq!(s.values(TagField::Genre), ["Pop", "Rock"]);
    assert_eq!(s.text(TagField::Genre), "Pop\nRock");
    s.set_text(TagField::Lyrics, "one\ntwo\n", false);
    assert_eq!(
        s.values(TagField::Lyrics),
        ["one\ntwo"],
        "one value, inner break kept"
    );
}

#[test]
fn a_number_and_its_total_are_one_field() {
    let mut s = all_storable();
    s.set_pair(TagField::TrackNumber, "03", " 12 ");
    assert_eq!(s.pair(TagField::TrackNumber), ("3".into(), "12".into()));
    s.set_pair(TagField::TrackNumber, "", "12");
    assert_eq!(s.pair(TagField::TrackNumber), (String::new(), "12".into()));
    s.set_pair(TagField::TrackNumber, "", "");
    assert!(!s.has(TagField::TrackNumber));
    assert_eq!(
        s.pair(TagField::TrackNumber),
        (String::new(), String::new())
    );
}

#[test]
fn bpm_loses_leading_zeros_but_keeps_other_text() {
    let mut s = all_storable();
    s.set_text(TagField::Bpm, "0128", false);
    assert_eq!(s.values(TagField::Bpm), ["128"]);
    s.set_text(TagField::Bpm, "fast", false);
    assert_eq!(s.values(TagField::Bpm), ["fast"]);
}

#[test]
fn the_editor_shows_the_always_shown_fields_and_the_ones_the_file_has() {
    let mut s = TagSheet::new(
        [
            TagField::Title,
            TagField::Bpm,
            TagField::Mood,
            TagField::Isrc,
        ],
        0,
        false,
    );
    s.set_text(TagField::Bpm, "120", false);
    let none = BTreeSet::new();
    let visible = s.visible_fields(&none);
    assert_eq!(
        visible[..10],
        TagField::ALL[..10],
        "even those it cannot store"
    );
    assert!(visible.contains(&TagField::Bpm), "the file has it");
    assert!(!visible.contains(&TagField::Mood), "not present, not added");
    let added = BTreeSet::from([TagField::Mood, TagField::Lyrics]);
    let visible = s.visible_fields(&added);
    assert!(visible.contains(&TagField::Mood));
    assert!(
        !visible.contains(&TagField::Lyrics),
        "an added field the format cannot store is not shown"
    );
}

#[test]
fn add_field_offers_only_what_the_format_can_store_and_is_missing() {
    let mut s = TagSheet::new(
        [
            TagField::Title,
            TagField::Bpm,
            TagField::Mood,
            TagField::Isrc,
        ],
        0,
        false,
    );
    s.set_text(TagField::Bpm, "120", false);
    let mut added = BTreeSet::new();
    assert_eq!(
        s.addable_fields(&added),
        [TagField::Mood, TagField::Isrc],
        "Title is always shown, Bpm is present, the rest cannot be stored"
    );
    added.insert(TagField::Mood);
    assert_eq!(s.addable_fields(&added), [TagField::Isrc]);
}

#[test]
fn text_is_cut_and_the_values_limited() {
    let mut s = all_storable();
    s.set_values(TagField::Artist, lines(&["abcdefghij", "b", "c", "d"]));
    s.set_pair(TagField::TrackNumber, "1", "99999");
    let s = s.clamped(4, 2);
    assert_eq!(s.values(TagField::Artist), ["abcd", "b"]);
    assert_eq!(
        s.pair(TagField::TrackNumber),
        ("1".into(), "9999".into()),
        "a pair keeps both its entries"
    );
}

#[test]
fn the_changed_fields_come_in_editor_order() {
    let before = all_storable();
    let mut after = before.clone();
    after.set_text(TagField::Genre, "Jazz", true);
    after.set_text(TagField::Title, "New", false);
    assert_eq!(
        changed_fields(&before, &after),
        [TagField::Title, TagField::Genre]
    );
    assert!(changed_fields(&before, &before).is_empty());
}

#[test]
fn the_values_of_a_changed_field_are_validated() {
    let before = all_storable();
    let invalid = |f: &dyn Fn(&mut TagSheet)| {
        let mut after = before.clone();
        f(&mut after);
        invalid_fields(&before, &after)
    };
    assert_eq!(
        invalid(&|s| s.set_text(TagField::Date, "2019-13", false)),
        [TagField::Date]
    );
    assert!(invalid(&|s| s.set_text(TagField::Date, "2019-05-14T10:30", false)).is_empty());
    assert_eq!(
        invalid(&|s| s.set_text(TagField::OriginalReleaseDate, "May 1999", false)),
        [TagField::OriginalReleaseDate]
    );
    assert_eq!(
        invalid(&|s| s.set_pair(TagField::TrackNumber, "3a", "")),
        [TagField::TrackNumber]
    );
    assert_eq!(
        invalid(&|s| s.set_pair(TagField::DiscNumber, "1", "-2")),
        [TagField::DiscNumber]
    );
    assert_eq!(
        invalid(&|s| s.set_pair(TagField::TrackNumber, "", "12")),
        [TagField::TrackNumber],
        "a total needs a number (the file would get track 0)"
    );
    assert!(invalid(&|s| s.set_pair(TagField::TrackNumber, "3", "12")).is_empty());
    assert_eq!(
        invalid(&|s| s.set_text(TagField::Bpm, "120.5", false)),
        [TagField::Bpm]
    );
    assert_eq!(
        invalid(&|s| s.set_text(TagField::Bpm, "99999999999", false)),
        [TagField::Bpm],
        "beyond u32"
    );
    assert!(invalid(&|s| s.set_text(TagField::Bpm, "128", false)).is_empty());
    assert_eq!(
        invalid(&|s| s.set_text(TagField::Date, "1999\n2000", true)),
        [TagField::Date],
        "one value only"
    );
}

#[test]
fn clearing_a_field_is_always_valid() {
    let mut before = all_storable();
    before.set_text(TagField::Date, "1999", false);
    before.set_pair(TagField::TrackNumber, "3", "12");
    let mut after = before.clone();
    after.set_text(TagField::Date, "", false);
    after.set_pair(TagField::TrackNumber, "", "");
    assert!(invalid_fields(&before, &after).is_empty());
}

#[test]
fn a_value_the_operator_did_not_touch_never_blocks_the_save() {
    // The file holds a track number that is not a number; leaving it alone
    // keeps it as it is.
    let mut before = all_storable();
    before.set_pair(TagField::TrackNumber, "A1", "");
    let mut after = before.clone();
    after.set_text(TagField::Genre, "Jazz", true);
    assert!(invalid_fields(&before, &after).is_empty());
    after.set_pair(TagField::TrackNumber, "A2", "");
    assert_eq!(invalid_fields(&before, &after), [TagField::TrackNumber]);
}

#[test]
fn a_change_to_a_field_the_format_cannot_store_is_invalid() {
    let before = TagSheet::new([TagField::Title], 0, false);
    let mut after = before.clone();
    after.set_text(TagField::AlbumArtist, "X", true);
    assert_eq!(invalid_fields(&before, &after), [TagField::AlbumArtist]);
}

#[test]
fn unstored_fields_are_the_changes_the_file_did_not_keep() {
    let before = all_storable();
    let mut after = before.clone();
    after.set_text(TagField::Genre, "Jazz", true);
    after.set_text(TagField::Mood, "Calm", true);
    let mut read_back = before.clone();
    read_back.set_text(TagField::Genre, "Jazz", true);
    assert_eq!(
        unstored_fields(&before, &after, &read_back),
        [TagField::Mood]
    );
    let mut other = read_back.clone();
    other.set_text(TagField::Title, "Changed by someone else", false);
    assert_eq!(
        unstored_fields(&before, &after, &other),
        [TagField::Mood],
        "only the fields the operator changed are judged"
    );
}

#[test]
fn the_value_cap_is_a_validated_config_field() {
    let mut c = Config::default();
    assert_eq!(c.limits.max_tag_values, 32);
    c.limits.max_tag_values = 0;
    let warnings = c.validate();
    assert_eq!(c.limits.max_tag_values, 1);
    assert!(warnings.iter().any(|w| w.field == "limits.max_tag_values"));
    c.limits.max_tag_values = 5_000;
    c.validate();
    assert_eq!(c.limits.max_tag_values, 1000);
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-model --test tag_sheet`
Expected: FAIL to compile (`TagField`, `TagSheet`, `changed_fields`, ... are not in `fp_model`).

- [ ] **Step 3: Implement**

Create `crates/fp-model/src/tag_sheet.rs`. Everything the editor decides without a file lives here: the fields and their order, which are always shown, how a value is typed, what `Add field` offers, and the validation and diff rules.

```rust
//! The tag sheet (feedback 2 spec O23): every field the tag editor can show
//! for one file, as the file holds it. The sheet is a plain value: reading
//! and writing the file is `fp-analysis`' job, drawing it is the UI's. What
//! is decided here, as pure functions, is which fields exist, which are
//! shown, which values are invalid and which fields changed.

use std::collections::{BTreeMap, BTreeSet};

use crate::track::{cut, parse_tag_date};

/// A field of the tag editor, in the order the editor shows them. Each one
/// maps to the format's own standard key (ID3v2 frame, Vorbis comment, MP4
/// atom, APE item, RIFF INFO chunk); `fp-analysis` knows how.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TagField {
    // Always shown.
    Title,
    Artist,
    Album,
    AlbumArtist,
    Date,
    TrackNumber,
    DiscNumber,
    Genre,
    Composer,
    Comment,
    // Shown when the file has them, offered by "Add field" otherwise.
    Subtitle,
    Grouping,
    Bpm,
    InitialKey,
    Mood,
    Isrc,
    Publisher,
    CatalogNumber,
    Copyright,
    OriginalArtist,
    OriginalAlbum,
    OriginalReleaseDate,
    Lyricist,
    Conductor,
    Remixer,
    Arranger,
    Performer,
    Language,
    EncodedBy,
    Lyrics,
    SortTitle,
    SortArtist,
    SortAlbum,
    SortAlbumArtist,
    SortComposer,
    ArtistWebsite,
}

/// How a field's value is typed and checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagFieldKind {
    /// Free text on one line.
    Text,
    /// Free text over several lines (comment, lyrics).
    LongText,
    /// ISO 8601 date (`parse_tag_date`).
    Date,
    /// A number and its total, both whole numbers (track, disc).
    Pair,
    /// A whole number (BPM).
    Whole,
}

impl TagField {
    /// Every field, in the order the editor shows them: the ten always-shown
    /// ones first, then the 26 optional ones.
    pub const ALL: [TagField; 36] = [
        TagField::Title,
        TagField::Artist,
        TagField::Album,
        TagField::AlbumArtist,
        TagField::Date,
        TagField::TrackNumber,
        TagField::DiscNumber,
        TagField::Genre,
        TagField::Composer,
        TagField::Comment,
        TagField::Subtitle,
        TagField::Grouping,
        TagField::Bpm,
        TagField::InitialKey,
        TagField::Mood,
        TagField::Isrc,
        TagField::Publisher,
        TagField::CatalogNumber,
        TagField::Copyright,
        TagField::OriginalArtist,
        TagField::OriginalAlbum,
        TagField::OriginalReleaseDate,
        TagField::Lyricist,
        TagField::Conductor,
        TagField::Remixer,
        TagField::Arranger,
        TagField::Performer,
        TagField::Language,
        TagField::EncodedBy,
        TagField::Lyrics,
        TagField::SortTitle,
        TagField::SortArtist,
        TagField::SortAlbum,
        TagField::SortAlbumArtist,
        TagField::SortComposer,
        TagField::ArtistWebsite,
    ];

    /// Shown for every file, whether it has the field or not.
    pub fn always_shown(self) -> bool {
        matches!(
            self,
            Self::Title
                | Self::Artist
                | Self::Album
                | Self::AlbumArtist
                | Self::Date
                | Self::TrackNumber
                | Self::DiscNumber
                | Self::Genre
                | Self::Composer
                | Self::Comment
        )
    }

    pub fn kind(self) -> TagFieldKind {
        match self {
            Self::Date | Self::OriginalReleaseDate => TagFieldKind::Date,
            Self::TrackNumber | Self::DiscNumber => TagFieldKind::Pair,
            Self::Bpm => TagFieldKind::Whole,
            Self::Comment | Self::Lyrics => TagFieldKind::LongText,
            _ => TagFieldKind::Text,
        }
    }

    /// The field can hold several values (two artists), shown one per line
    /// and written one per value through the format's own mechanism.
    pub fn is_multi_value(self) -> bool {
        matches!(
            self,
            Self::Artist
                | Self::AlbumArtist
                | Self::Genre
                | Self::Composer
                | Self::Mood
                | Self::OriginalArtist
                | Self::Lyricist
                | Self::Conductor
                | Self::Remixer
                | Self::Arranger
                | Self::Performer
                | Self::Language
        )
    }

    /// The suffix of the field's label key in the locale files
    /// (`tag-field-<slug>`).
    pub fn slug(self) -> &'static str {
        match self {
            Self::Title => "title",
            Self::Artist => "artist",
            Self::Album => "album",
            Self::AlbumArtist => "album-artist",
            Self::Date => "date",
            Self::TrackNumber => "track-number",
            Self::DiscNumber => "disc-number",
            Self::Genre => "genre",
            Self::Composer => "composer",
            Self::Comment => "comment",
            Self::Subtitle => "subtitle",
            Self::Grouping => "grouping",
            Self::Bpm => "bpm",
            Self::InitialKey => "initial-key",
            Self::Mood => "mood",
            Self::Isrc => "isrc",
            Self::Publisher => "publisher",
            Self::CatalogNumber => "catalog-number",
            Self::Copyright => "copyright",
            Self::OriginalArtist => "original-artist",
            Self::OriginalAlbum => "original-album",
            Self::OriginalReleaseDate => "original-release-date",
            Self::Lyricist => "lyricist",
            Self::Conductor => "conductor",
            Self::Remixer => "remixer",
            Self::Arranger => "arranger",
            Self::Performer => "performer",
            Self::Language => "language",
            Self::EncodedBy => "encoded-by",
            Self::Lyrics => "lyrics",
            Self::SortTitle => "sort-title",
            Self::SortArtist => "sort-artist",
            Self::SortAlbum => "sort-album",
            Self::SortAlbumArtist => "sort-album-artist",
            Self::SortComposer => "sort-composer",
            Self::ArtistWebsite => "artist-website",
        }
    }
}

/// Tidies the lines of one field: trimmed, empty lines dropped. A number and
/// total field always holds exactly two entries (number, total), numbers in
/// their plain form (`03` is `3`), or nothing when both are empty.
fn canonical(field: TagField, lines: Vec<String>) -> Vec<String> {
    if field.kind() == TagFieldKind::Pair {
        let mut parts = lines.into_iter().map(|l| plain_number(&l));
        let (number, total) = (
            parts.next().unwrap_or_default(),
            parts.next().unwrap_or_default(),
        );
        return if number.is_empty() && total.is_empty() {
            Vec::new()
        } else {
            vec![number, total]
        };
    }
    lines
        .into_iter()
        .map(|l| tidy(field, &l))
        .filter(|l| !l.is_empty())
        .collect()
}

/// `text` trimmed; a whole number loses its leading zeros.
fn plain_number(text: &str) -> String {
    let text = text.trim();
    match text.parse::<u32>() {
        Ok(n) if text.bytes().all(|b| b.is_ascii_digit()) => n.to_string(),
        _ => text.to_owned(),
    }
}

fn tidy(field: TagField, text: &str) -> String {
    if field.kind() == TagFieldKind::Whole {
        plain_number(text)
    } else {
        text.trim().to_owned()
    }
}

fn is_whole(text: &str) -> bool {
    text.is_empty() || (text.bytes().all(|b| b.is_ascii_digit()) && text.parse::<u32>().is_ok())
}

/// What a file's tag holds, field by field, and what its format can store.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TagSheet {
    values: BTreeMap<TagField, Vec<String>>,
    storable: BTreeSet<TagField>,
    /// Tags the sheet does not show (other standard keys, custom keys,
    /// pictures). They are kept as they are.
    pub other_kept: usize,
    /// The format also holds items the reader cannot count, so `other_kept`
    /// is a lower bound.
    pub other_kept_more: bool,
}

impl TagSheet {
    /// An empty sheet for a format that can store `storable`.
    pub fn new(
        storable: impl IntoIterator<Item = TagField>,
        other_kept: usize,
        other_kept_more: bool,
    ) -> Self {
        Self {
            values: BTreeMap::new(),
            storable: storable.into_iter().collect(),
            other_kept,
            other_kept_more,
        }
    }

    /// The values of `field`: empty when the file has none; two entries
    /// (number, total) for a number and total field.
    pub fn values(&self, field: TagField) -> &[String] {
        self.values.get(&field).map_or(&[], Vec::as_slice)
    }

    /// Replaces the values of `field` (tidied; none left removes it).
    pub fn set_values(&mut self, field: TagField, values: Vec<String>) {
        let values = canonical(field, values);
        if values.is_empty() {
            self.values.remove(&field);
        } else {
            self.values.insert(field, values);
        }
    }

    /// The file's format can store `field`.
    pub fn can_store(&self, field: TagField) -> bool {
        self.storable.contains(&field)
    }

    /// The file has a value for `field`.
    pub fn has(&self, field: TagField) -> bool {
        self.values.contains_key(&field)
    }

    /// The values as one text, one per line: what a text box holds.
    pub fn text(&self, field: TagField) -> String {
        self.values(field).join("\n")
    }

    /// Sets the values from a text box. With `as_lines` every non-empty line
    /// is a value; without, the whole text is one value.
    pub fn set_text(&mut self, field: TagField, text: &str, as_lines: bool) {
        let values = if as_lines {
            text.lines().map(str::to_owned).collect()
        } else {
            vec![text.to_owned()]
        };
        self.set_values(field, values);
    }

    /// The number and the total of a track or disc field (empty if unset).
    pub fn pair(&self, field: TagField) -> (String, String) {
        let values = self.values(field);
        (
            values.first().cloned().unwrap_or_default(),
            values.get(1).cloned().unwrap_or_default(),
        )
    }

    pub fn set_pair(&mut self, field: TagField, number: &str, total: &str) {
        self.set_values(field, vec![number.to_owned(), total.to_owned()]);
    }

    /// Whether the editor shows `field` one value per line: it can hold
    /// several, or the file holds several now.
    pub fn shows_lines(&self, field: TagField) -> bool {
        field.is_multi_value() || self.values(field).len() > 1
    }

    /// The fields to draw, in order: the always-shown ones (the editor greys
    /// out those the format cannot store), then each optional field the file
    /// has or the operator `added` (only if the format can store it).
    pub fn visible_fields(&self, added: &BTreeSet<TagField>) -> Vec<TagField> {
        TagField::ALL
            .into_iter()
            .filter(|f| {
                f.always_shown() || (self.can_store(*f) && (self.has(*f) || added.contains(f)))
            })
            .collect()
    }

    /// What **Add field** offers: optional fields the format can store that
    /// the file does not have and the operator has not added yet.
    pub fn addable_fields(&self, added: &BTreeSet<TagField>) -> Vec<TagField> {
        TagField::ALL
            .into_iter()
            .filter(|f| {
                !f.always_shown() && self.can_store(*f) && !self.has(*f) && !added.contains(f)
            })
            .collect()
    }

    /// Every line cut to `max_chars` characters and at most `max_values`
    /// values per field (a number and total field keeps its two). Tag text
    /// comes from files and keyboards and must not bloat memory or the
    /// window.
    #[must_use]
    pub fn clamped(self, max_chars: usize, max_values: usize) -> Self {
        let mut out = Self {
            values: BTreeMap::new(),
            ..self.clone()
        };
        for (field, lines) in self.values {
            let keep = if field.kind() == TagFieldKind::Pair {
                lines.len()
            } else {
                max_values
            };
            let lines = lines
                .into_iter()
                .take(keep)
                .map(|mut line| {
                    cut(&mut line, max_chars);
                    line
                })
                .collect();
            out.set_values(field, lines);
        }
        out
    }
}

/// The fields whose values differ between two sheets, in editor order.
pub fn changed_fields(before: &TagSheet, after: &TagSheet) -> Vec<TagField> {
    TagField::ALL
        .into_iter()
        .filter(|f| before.values(*f) != after.values(*f))
        .collect()
}

/// The changed fields (`after` against `before`) whose value is not valid:
/// a date that is not ISO 8601, a number, total or BPM that is not a whole
/// number, a total without a number, several lines in a one-value field of
/// those kinds, or a field the file's format cannot store. A value the file
/// already held and the operator did not touch is never reported: it is kept
/// as it is.
pub fn invalid_fields(before: &TagSheet, after: &TagSheet) -> Vec<TagField> {
    changed_fields(before, after)
        .into_iter()
        .filter(|f| {
            let values = after.values(*f);
            let valid = match f.kind() {
                TagFieldKind::Text | TagFieldKind::LongText => true,
                TagFieldKind::Date => {
                    values.len() <= 1 && values.iter().all(|v| parse_tag_date(v).is_ok())
                }
                TagFieldKind::Whole => values.len() <= 1 && values.iter().all(|v| is_whole(v)),
                TagFieldKind::Pair => {
                    let (number, total) = after.pair(*f);
                    is_whole(&number)
                        && is_whole(&total)
                        && (total.is_empty() || !number.is_empty())
                }
            };
            !valid || !after.can_store(*f)
        })
        .collect()
}

/// The fields the operator changed that the file does not hold as written
/// after the save (`read_back`): the format dropped or reshaped them.
pub fn unstored_fields(before: &TagSheet, after: &TagSheet, read_back: &TagSheet) -> Vec<TagField> {
    changed_fields(before, after)
        .into_iter()
        .filter(|f| after.values(*f) != read_back.values(*f))
        .collect()
}
```

Apply the three small changes (the `cut` helper is reused for each line, the module is exported, the new limit is a validated `Config` field per CLAUDE.md rule 4):

```diff
--- a/crates/fp-model/src/track.rs
+++ b/crates/fp-model/src/track.rs
@@ -172,7 +172,7 @@
 }
 
 /// Cuts `text` to `max_chars` characters after trimming it.
-fn cut(text: &mut String, max_chars: usize) {
+pub(crate) fn cut(text: &mut String, max_chars: usize) {
     let trimmed = text.trim();
     let end = trimmed
         .char_indices()
--- a/crates/fp-model/src/lib.rs
+++ b/crates/fp-model/src/lib.rs
@@ -22,6 +22,7 @@
 pub mod shortcuts;
 pub mod state;
 mod tag_edit;
+mod tag_sheet;
 pub mod track;
 pub mod volume;
 
@@ -53,6 +54,9 @@
 pub use shortcuts::{KeyChord, Shortcut, ShortcutAction, default_shortcuts, player_command};
 pub use state::AppState;
 pub use tag_edit::{TagEditBlock, tag_edit_block};
+pub use tag_sheet::{
+    TagField, TagFieldKind, TagSheet, changed_fields, invalid_fields, unstored_fields,
+};
 pub use track::{
     AudioFormat, FileState, InvalidDate, Library, Marker, MarkerKind, MarkerSource, Markers,
     PlayRange, Track, TrackAnalysis, TrackKind, TrackTags, parse_tag_date,
--- a/crates/fp-model/src/config.rs
+++ b/crates/fp-model/src/config.rs
@@ -396,6 +396,9 @@
     pub max_cart_cols: u16,
     /// Longest tag text kept per field, in characters (tags are trimmed and cut).
     pub max_tag_chars: usize,
+    /// Most values kept per tag field (two artists are two values), so a
+    /// hostile file cannot fill the tag editor with thousands of lines.
+    pub max_tag_values: usize,
 }
 
 impl Default for Limits {
@@ -411,6 +414,7 @@
             max_cart_rows: 8,
             max_cart_cols: 16,
             max_tag_chars: 2000,
+            max_tag_values: 32,
         }
     }
 }
@@ -529,6 +533,13 @@
             &mut w,
         );
         clamp_to(
+            &mut l.max_tag_values,
+            1,
+            1000,
+            "limits.max_tag_values",
+            &mut w,
+        );
+        clamp_to(
             &mut l.max_state_file_bytes,
             MIB,
             1024 * MIB,
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-model --test tag_sheet` then `cargo test -p fp-model`
Expected: PASS (16 tests in `tag_sheet`).

- [ ] **Step 5: Commit**

```bash
if cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings \
   && cargo test --workspace; then
  git add crates/fp-model
  git commit -m "feat(model): the tag sheet, its 36 fields and its pure rules" -m "The tag editor covers the fields common players show. Which fields exist, which are always shown, what Add field offers, how a value is validated and which fields changed are pure functions of two sheets, so the UI and the file writer cannot disagree."
fi
```

#### Part B: reading and writing the sheet in `fp-analysis`

- [ ] **Step 6: Write the failing tests**

Create `crates/fp-analysis/tests/tag_sheet.rs`. Its helpers build a fresh WAV (ID3v2 is lofty's primary tag type for it), a WAV with a RIFF INFO chunk, and a synthesized ID3v2.4 MP3; the "other tags" MP3 carries a custom `TXXX`, a comment with a description, two URL frames (`WOAR`, which is a field, and `WCOM`, which is not) and a picture.

```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Feedback 2 spec O23: the tag sheet read from and written to real files.

use std::io::Cursor;
use std::path::{Path, PathBuf};

use fp_analysis::tags::{
    TagWriteError, read_tag_sheet, storable_fields, write_tag_sheet, write_tags,
};
use fp_model::{Limits, TagField, TagSheet, TrackTags};
use lofty::config::WriteOptions;
use lofty::id3::v2::{
    AttachedPictureFrame, CommentFrame, ExtendedTextFrame, Frame, FrameId, Id3v2Tag, UrlLinkFrame,
};
use lofty::picture::{MimeType, Picture, PictureType};
use lofty::prelude::*;
use lofty::tag::{ItemKey, Tag, TagType};

fn limits() -> Limits {
    Limits::default()
}

fn wav(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join(name);
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 8_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(&path, spec).unwrap();
    for _ in 0..800 {
        w.write_sample(0i16).unwrap();
    }
    w.finalize().unwrap();
    path
}

/// A WAV that already has a RIFF INFO chunk (a fresh WAV would get an ID3v2
/// tag, lofty's primary tag type for it), with a title and an encoder.
fn riff_wav(dir: &Path, name: &str) -> PathBuf {
    let path = wav(dir, name);
    let mut file = lofty::read_from_path(&path).unwrap();
    file.insert_tag(Tag::new(TagType::RiffInfo));
    let tag = file.tag_mut(TagType::RiffInfo).unwrap();
    tag.set_title("Old title".into());
    tag.insert_text(ItemKey::EncoderSoftware, "An Encoder 1.0".into());
    file.save_to_path(&path, WriteOptions::default()).unwrap();
    path
}

/// A minimal MP3: MPEG-1 Layer III frames (128 kbps, 44.1 kHz, 417 bytes
/// each) with silent payloads.
fn mp3(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join(name);
    let mut bytes = Vec::new();
    for _ in 0..10 {
        let mut frame = vec![0u8; 417];
        frame[..4].copy_from_slice(&0xFFFB_9064u32.to_be_bytes());
        bytes.extend_from_slice(&frame);
    }
    std::fs::write(&path, bytes).unwrap();
    path
}

fn png() -> Vec<u8> {
    let img = image::RgbImage::from_pixel(40, 30, image::Rgb([200, 30, 30]));
    let mut out = Vec::new();
    image::DynamicImage::ImageRgb8(img)
        .write_to(&mut Cursor::new(&mut out), image::ImageFormat::Png)
        .unwrap();
    out
}

/// An MP3 with frames the sheet does not own: a custom `TXXX`, a comment
/// with a description, a URL frame lofty maps (`WCOM`), a picture; and the
/// plain fields title and comment.
fn mp3_with_other_tags(dir: &Path) -> PathBuf {
    let path = mp3(dir, "other.mp3");
    let mut tag = Id3v2Tag::new();
    tag.insert(Frame::UserText(ExtendedTextFrame::new(
        lofty::TextEncoding::UTF8,
        "MY_CUSTOM".to_owned(),
        "custom value".to_owned(),
    )));
    tag.insert(Frame::Comment(CommentFrame::new(
        lofty::TextEncoding::UTF8,
        *b"eng",
        "ReplayNote".to_owned(),
        "000 111".to_owned(),
    )));
    tag.insert(Frame::Comment(CommentFrame::new(
        lofty::TextEncoding::UTF8,
        *b"eng",
        String::new(),
        "a plain comment".to_owned(),
    )));
    tag.insert(Frame::Url(UrlLinkFrame::new(
        FrameId::new("WCOM").unwrap(),
        "http://shop.example",
    )));
    tag.insert(Frame::Url(UrlLinkFrame::new(
        FrameId::new("WOAR").unwrap(),
        "http://artist.example",
    )));
    tag.insert(Frame::Picture(AttachedPictureFrame::new(
        lofty::TextEncoding::UTF8,
        Picture::unchecked(png())
            .pic_type(PictureType::CoverFront)
            .mime_type(MimeType::Png)
            .build(),
    )));
    tag.set_title("Old title".to_owned());
    tag.save_to_path(&path, WriteOptions::default()).unwrap();
    path
}

fn sheet(path: &Path) -> TagSheet {
    read_tag_sheet(path, &limits()).expect("a sheet")
}

fn raw_contains(path: &Path, needle: &[u8]) -> bool {
    std::fs::read(path)
        .unwrap()
        .windows(needle.len())
        .any(|w| w == needle)
}

fn names(dir: &Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    v.sort();
    v
}

/// Saves `edit` applied to the sheet of `path`, and returns the sheet read
/// back.
fn edit(path: &Path, change: impl FnOnce(&mut TagSheet)) -> TagSheet {
    let before = sheet(path);
    let mut after = before.clone();
    change(&mut after);
    write_tag_sheet(path, &before, &after, &limits()).unwrap();
    sheet(path)
}

#[test]
fn what_a_format_can_store_decides_the_fields() {
    let id3 = storable_fields(TagType::Id3v2);
    let missing: Vec<_> = TagField::ALL
        .into_iter()
        .filter(|f| !id3.contains(f))
        .collect();
    assert_eq!(
        missing,
        [TagField::Performer],
        "ID3v2 has no performer frame"
    );

    let riff = storable_fields(TagType::RiffInfo);
    for stored in [
        TagField::Title,
        TagField::Artist,
        TagField::Album,
        TagField::Date,
        TagField::TrackNumber,
        TagField::Genre,
        TagField::Composer,
        TagField::Comment,
        TagField::Copyright,
        TagField::Language,
        TagField::EncodedBy,
    ] {
        assert!(riff.contains(&stored), "{stored:?}");
    }
    for not_stored in [TagField::AlbumArtist, TagField::DiscNumber, TagField::Bpm] {
        assert!(!riff.contains(&not_stored), "{not_stored:?}");
    }
    assert_eq!(
        storable_fields(TagType::AiffText),
        [
            TagField::Title,
            TagField::Artist,
            TagField::Comment,
            TagField::Copyright
        ]
    );
    let vorbis = storable_fields(TagType::VorbisComments);
    assert!(vorbis.contains(&TagField::Bpm) && vorbis.contains(&TagField::Lyrics));
    let mp4 = storable_fields(TagType::Mp4Ilst);
    assert!(mp4.contains(&TagField::Bpm) && !mp4.contains(&TagField::Publisher));
    assert!(storable_fields(TagType::Id3v1).contains(&TagField::Title));
}

#[test]
fn add_field_never_offers_what_the_file_cannot_store() {
    let dir = tempfile::tempdir().unwrap();
    let s = sheet(&riff_wav(dir.path(), "x.wav"));
    let added = std::collections::BTreeSet::new();
    assert_eq!(
        s.addable_fields(&added),
        [TagField::Copyright, TagField::Language, TagField::EncodedBy],
        "RIFF INFO stores 11 fields; the 25 others cannot be added"
    );
    assert!(!s.can_store(TagField::AlbumArtist));
    let s = sheet(&mp3(dir.path(), "x.mp3"));
    let offered = s.addable_fields(&added);
    assert_eq!(
        offered.len(),
        26 - 1,
        "every optional field but the performer"
    );
    assert!(!offered.contains(&TagField::Performer));
}

#[test]
fn an_untagged_file_gives_an_empty_sheet_of_its_primary_tag() {
    let dir = tempfile::tempdir().unwrap();
    let s = sheet(&wav(dir.path(), "x.wav"));
    assert!(TagField::ALL.iter().all(|f| !s.has(*f)));
    assert!(s.can_store(TagField::AlbumArtist), "a new WAV gets ID3v2");
    assert_eq!(s.other_kept, 0);
}

#[test]
fn a_file_without_a_sheet_says_so() {
    let dir = tempfile::tempdir().unwrap();
    assert!(read_tag_sheet(&dir.path().join("missing.wav"), &limits()).is_none());
    let dsf = dir.path().join("x.dsf");
    std::fs::write(&dsf, b"DSD ").unwrap();
    assert!(
        read_tag_sheet(&dsf, &limits()).is_none(),
        "no writable tags"
    );
    let broken = dir.path().join("broken.wav");
    std::fs::write(&broken, b"RIFF\x04\x00\x00\x00WAVE").unwrap();
    assert!(read_tag_sheet(&broken, &limits()).is_none());
}

#[test]
fn riff_info_round_trips_its_fields_with_values_one_per_line() {
    let dir = tempfile::tempdir().unwrap();
    let path = riff_wav(dir.path(), "x.wav");
    let s = edit(&path, |s| {
        s.set_text(TagField::Artist, "First Artist\nSecond Artist", true);
        s.set_text(TagField::Album, "An Album", false);
        s.set_text(TagField::Date, "1999-03-07", false);
        s.set_pair(TagField::TrackNumber, "3", "12");
        s.set_text(TagField::Genre, "Pop", true);
        s.set_text(TagField::Composer, "A. Composer", true);
        s.set_text(TagField::Comment, "A note\nover two lines", false);
        s.set_text(TagField::Copyright, "(c) 1999", false);
        s.set_text(TagField::Language, "eng", true);
    });
    assert_eq!(s.values(TagField::Title), ["Old title"], "untouched");
    assert_eq!(
        s.values(TagField::Artist),
        ["First Artist", "Second Artist"]
    );
    assert_eq!(s.values(TagField::Album), ["An Album"]);
    assert_eq!(s.values(TagField::Date), ["1999-03-07"]);
    assert_eq!(s.pair(TagField::TrackNumber), ("3".into(), "12".into()));
    assert_eq!(s.values(TagField::Genre), ["Pop"]);
    assert_eq!(s.values(TagField::Composer), ["A. Composer"]);
    assert_eq!(s.values(TagField::Comment), ["A note\nover two lines"]);
    assert_eq!(s.values(TagField::Copyright), ["(c) 1999"]);
    assert_eq!(s.values(TagField::Language), ["eng"]);
    assert_eq!(s.other_kept, 1, "the encoder software item is not a field");
    // The tag stayed a RIFF INFO tag: no ID3v2 appeared.
    let file = lofty::read_from_path(&path).unwrap();
    assert!(file.tag(TagType::Id3v2).is_none());
}

#[test]
fn a_riff_number_written_as_one_value_is_split() {
    let dir = tempfile::tempdir().unwrap();
    let path = riff_wav(dir.path(), "x.wav");
    let mut file = lofty::read_from_path(&path).unwrap();
    file.tag_mut(TagType::RiffInfo)
        .unwrap()
        .insert_text(ItemKey::TrackNumber, "3/12".into());
    file.save_to_path(&path, WriteOptions::default()).unwrap();
    assert_eq!(
        sheet(&path).pair(TagField::TrackNumber),
        ("3".into(), "12".into())
    );
}

#[test]
fn a_field_the_format_cannot_store_is_refused_before_the_disk_is_touched() {
    let dir = tempfile::tempdir().unwrap();
    let path = riff_wav(dir.path(), "x.wav");
    let original = std::fs::read(&path).unwrap();
    let before = sheet(&path);
    let mut after = before.clone();
    after.set_text(TagField::AlbumArtist, "Various", true);
    assert_eq!(
        write_tag_sheet(&path, &before, &after, &limits()),
        Err(TagWriteError::InvalidField(TagField::AlbumArtist))
    );
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert_eq!(names(dir.path()), ["x.wav"]);
}

#[test]
fn every_id3v2_field_round_trips() {
    let dir = tempfile::tempdir().unwrap();
    let path = mp3(dir.path(), "x.mp3");
    let wanted: Vec<(TagField, &str)> = vec![
        (TagField::Title, "Song Title"),
        (TagField::Artist, "Artist One\nArtist Two"),
        (TagField::Album, "An Album"),
        (TagField::AlbumArtist, "Various Artists"),
        (TagField::Date, "2019-05-14"),
        (TagField::Genre, "Pop\nRock"),
        (TagField::Composer, "A. Composer\nB. Composer"),
        (TagField::Comment, "A note\nover two lines"),
        (TagField::Subtitle, "A subtitle"),
        (TagField::Grouping, "A grouping"),
        (TagField::Bpm, "128"),
        (TagField::InitialKey, "Am"),
        (TagField::Mood, "Calm"),
        (TagField::Isrc, "ESABC1900001"),
        (TagField::Publisher, "A Label"),
        (TagField::CatalogNumber, "CAT-001"),
        (TagField::Copyright, "(c) 2019"),
        (TagField::OriginalArtist, "Original Artist"),
        (TagField::OriginalAlbum, "Original Album"),
        (TagField::OriginalReleaseDate, "1975"),
        (TagField::Lyricist, "A Lyricist"),
        (TagField::Conductor, "A Conductor"),
        (TagField::Remixer, "A Remixer"),
        (TagField::Arranger, "An Arranger"),
        (TagField::Language, "eng"),
        (TagField::EncodedBy, "Someone"),
        (TagField::Lyrics, "first line\nsecond line"),
        (TagField::SortTitle, "Title, Song"),
        (TagField::SortArtist, "Artist, One"),
        (TagField::SortAlbum, "Album, An"),
        (TagField::SortAlbumArtist, "Artists, Various"),
        (TagField::SortComposer, "Composer, A."),
        (TagField::ArtistWebsite, "http://artist.example"),
    ];
    let before = sheet(&path);
    let mut after = before.clone();
    for (field, text) in &wanted {
        after.set_text(*field, text, after.shows_lines(*field));
    }
    after.set_pair(TagField::TrackNumber, "3", "12");
    after.set_pair(TagField::DiscNumber, "1", "2");
    write_tag_sheet(&path, &before, &after, &limits()).unwrap();

    let read = sheet(&path);
    for (field, _) in &wanted {
        assert_eq!(read.values(*field), after.values(*field), "{field:?}");
    }
    assert_eq!(read.pair(TagField::TrackNumber), ("3".into(), "12".into()));
    assert_eq!(read.pair(TagField::DiscNumber), ("1".into(), "2".into()));
    assert_eq!(read.values(TagField::Artist), ["Artist One", "Artist Two"]);
    assert_eq!(read.values(TagField::Lyrics), ["first line\nsecond line"]);
    assert!(!read.has(TagField::Performer));
    // The ID3v2.4 frames are the standard ones.
    for frame in [
        &b"TPE1"[..],
        b"TPE2",
        b"TDRC",
        b"TRCK",
        b"TPOS",
        b"TCON",
        b"TCOM",
        b"COMM",
        b"TIT3",
        b"TIT1",
        b"TBPM",
        b"TKEY",
        b"TMOO",
        b"TSRC",
        b"TPUB",
        b"TCOP",
        b"TOPE",
        b"TOAL",
        b"TDOR",
        b"TEXT",
        b"TPE3",
        b"TPE4",
        b"TLAN",
        b"TENC",
        b"USLT",
        b"TSOT",
        b"TSOP",
        b"TSOA",
        b"TSO2",
        b"TSOC",
        b"WOAR",
    ] {
        assert!(
            raw_contains(&path, frame),
            "frame {}",
            String::from_utf8_lossy(frame)
        );
    }
    let file = lofty::read_from_path(&path).unwrap();
    let tag = file.primary_tag().unwrap();
    assert_eq!(tag.get_strings(ItemKey::TrackArtist).count(), 2);
}

#[test]
fn only_the_changed_fields_are_written() {
    let dir = tempfile::tempdir().unwrap();
    let path = mp3(dir.path(), "x.mp3");
    edit(&path, |s| {
        s.set_text(TagField::Title, "Kept title", false);
        s.set_text(TagField::Mood, "Calm", false);
        s.set_text(TagField::Genre, "Pop", true);
    });
    let s = edit(&path, |s| {
        s.set_text(TagField::Genre, "Jazz", true);
    });
    assert_eq!(s.values(TagField::Genre), ["Jazz"]);
    assert_eq!(s.values(TagField::Title), ["Kept title"]);
    assert_eq!(s.values(TagField::Mood), ["Calm"]);
}

#[test]
fn a_cleared_field_is_removed_and_an_empty_added_field_is_not_written() {
    let dir = tempfile::tempdir().unwrap();
    let path = mp3(dir.path(), "x.mp3");
    edit(&path, |s| {
        s.set_text(TagField::Mood, "Calm", false);
        s.set_pair(TagField::TrackNumber, "3", "12");
        s.set_text(TagField::Date, "1999", false);
    });
    let s = edit(&path, |s| {
        s.set_text(TagField::Mood, "", false);
        s.set_pair(TagField::TrackNumber, "", "");
        s.set_text(TagField::Date, "", false);
        s.set_text(TagField::Isrc, "  ", false);
    });
    assert!(!s.has(TagField::Mood));
    assert!(!s.has(TagField::TrackNumber));
    assert!(!s.has(TagField::Date));
    assert!(!s.has(TagField::Isrc));
    assert!(!raw_contains(&path, b"TMOO"));
    assert!(!raw_contains(&path, b"TSRC"));
    assert!(!raw_contains(&path, b"TRCK"));
}

#[test]
fn custom_items_and_pictures_survive_an_edit_untouched() {
    let dir = tempfile::tempdir().unwrap();
    let path = mp3_with_other_tags(dir.path());
    let before = sheet(&path);
    assert_eq!(before.values(TagField::Comment), ["a plain comment"]);
    assert_eq!(
        before.values(TagField::ArtistWebsite),
        ["http://artist.example"]
    );
    assert_eq!(
        before.other_kept, 3,
        "the described comment, the WCOM url and the picture"
    );
    assert!(
        before.other_kept_more,
        "the TXXX frame is kept but lofty does not count it"
    );

    let after = edit(&path, |s| {
        s.set_text(TagField::Title, "New title", false);
        s.set_text(TagField::Comment, "a new comment", false);
        s.set_text(TagField::Genre, "Pop", true);
    });
    assert_eq!(after.values(TagField::Comment), ["a new comment"]);
    assert_eq!(after.other_kept, 3, "nothing was lost");
    assert!(raw_contains(&path, b"MY_CUSTOM"), "the custom frame");
    assert!(raw_contains(&path, b"custom value"));
    assert!(raw_contains(&path, b"ReplayNote"), "the described comment");
    assert!(raw_contains(&path, b"000 111"));
    assert!(
        raw_contains(&path, b"WCOM"),
        "the url lofty reads as a locator"
    );
    assert!(raw_contains(&path, b"http://shop.example"));
    assert!(raw_contains(&path, b"WOAR"));
    let file = lofty::read_from_path(&path).unwrap();
    let tag = file.primary_tag().unwrap();
    assert_eq!(tag.picture_count(), 1, "the cover");
    assert_eq!(tag.pictures()[0].data(), png().as_slice(), "byte for byte");
}

#[test]
fn a_summary_write_keeps_the_url_frames_too() {
    let dir = tempfile::tempdir().unwrap();
    let path = mp3_with_other_tags(dir.path());
    let before = fp_analysis::tags::read_track_tags(&path, &limits());
    let after = TrackTags {
        title: "Another".into(),
        ..before.clone()
    };
    write_tags(&path, &before, &after, &limits()).unwrap();
    assert!(raw_contains(&path, b"WCOM"));
    assert!(raw_contains(&path, b"WOAR"));
}

#[test]
fn an_invalid_value_is_refused_and_the_file_is_untouched() {
    let dir = tempfile::tempdir().unwrap();
    let path = mp3(dir.path(), "x.mp3");
    let original = std::fs::read(&path).unwrap();
    let before = sheet(&path);
    for (field, bad) in [
        (TagField::Date, "2019-13"),
        (TagField::OriginalReleaseDate, "May 1999"),
        (TagField::Bpm, "fast"),
    ] {
        let mut after = before.clone();
        after.set_text(field, bad, false);
        assert_eq!(
            write_tag_sheet(&path, &before, &after, &limits()),
            Err(TagWriteError::InvalidField(field))
        );
    }
    let mut after = before.clone();
    after.set_pair(TagField::TrackNumber, "x", "");
    assert_eq!(
        write_tag_sheet(&path, &before, &after, &limits()),
        Err(TagWriteError::InvalidField(TagField::TrackNumber))
    );
    let mut after = before.clone();
    after.set_pair(TagField::DiscNumber, "", "2");
    assert_eq!(
        write_tag_sheet(&path, &before, &after, &limits()),
        Err(TagWriteError::InvalidField(TagField::DiscNumber)),
        "a total without a number would be written as disc 0"
    );
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert_eq!(names(dir.path()), ["x.mp3"]);
}

#[test]
fn a_failed_sheet_write_leaves_the_original_and_no_temporary_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("broken.wav");
    std::fs::write(&path, b"RIFF\x04\x00\x00\x00WAVE").unwrap();
    let original = std::fs::read(&path).unwrap();
    let before = TagSheet::new(storable_fields(TagType::Id3v2), 0, false);
    let mut after = before.clone();
    after.set_text(TagField::Title, "x", false);
    let result = write_tag_sheet(&path, &before, &after, &limits());
    assert!(matches!(result, Err(TagWriteError::Other(_))), "{result:?}");
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert_eq!(names(dir.path()), ["broken.wav"]);
}

#[test]
fn an_unwritable_format_is_refused_without_touching_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("x.dsf");
    std::fs::write(&path, b"DSD fake").unwrap();
    let before = TagSheet::new([TagField::Title], 0, false);
    let mut after = before.clone();
    after.set_text(TagField::Title, "x", false);
    assert_eq!(
        write_tag_sheet(&path, &before, &after, &limits()),
        Err(TagWriteError::Unsupported)
    );
    assert_eq!(std::fs::read(&path).unwrap(), b"DSD fake");
    assert_eq!(names(dir.path()), ["x.dsf"]);
}

#[test]
fn the_text_and_the_values_are_cut_when_read() {
    let dir = tempfile::tempdir().unwrap();
    let path = mp3(dir.path(), "x.mp3");
    let many: Vec<String> = (0..50)
        .map(|n| format!("Artist {n} with a long name"))
        .collect();
    let big = Limits {
        max_tag_values: 100,
        ..limits()
    };
    let before = read_tag_sheet(&path, &big).unwrap();
    let mut after = before.clone();
    after.set_values(TagField::Artist, many);
    write_tag_sheet(&path, &before, &after, &big).unwrap();

    let small = Limits {
        max_tag_chars: 64,
        max_tag_values: 3,
        ..limits()
    };
    let s = read_tag_sheet(&path, &small).unwrap();
    assert_eq!(s.values(TagField::Artist).len(), 3);
    assert!(
        s.values(TagField::Artist)
            .iter()
            .all(|a| a.chars().count() <= 64)
    );
}

#[test]
fn what_is_written_is_cut_to_the_limits() {
    let dir = tempfile::tempdir().unwrap();
    let path = mp3(dir.path(), "x.mp3");
    let small = Limits {
        max_tag_chars: 64,
        max_tag_values: 2,
        ..limits()
    };
    let before = read_tag_sheet(&path, &small).unwrap();
    let mut after = before.clone();
    after.set_text(TagField::Genre, "A\nB\nC\nD", true);
    after.set_text(TagField::Comment, &"x".repeat(500), false);
    write_tag_sheet(&path, &before, &after, &small).unwrap();
    let read = read_tag_sheet(&path, &limits()).unwrap();
    assert_eq!(read.values(TagField::Genre), ["A", "B"]);
    assert_eq!(read.values(TagField::Comment)[0].chars().count(), 64);
}

#[test]
fn a_full_date_is_not_reduced_by_an_unrelated_edit() {
    let dir = tempfile::tempdir().unwrap();
    let path = mp3(dir.path(), "x.mp3");
    edit(&path, |s| {
        s.set_text(TagField::Date, "2019-05-14T10:30", false)
    });
    let s = edit(&path, |s| s.set_text(TagField::Genre, "Pop", true));
    assert_eq!(s.values(TagField::Date), ["2019-05-14T10:30"]);
}
```

- [ ] **Step 7: Run the tests to verify they fail**

Run: `cargo test -p fp-analysis --test tag_sheet`
Expected: FAIL to compile (`read_tag_sheet`, `write_tag_sheet`, `storable_fields` and `TagWriteError::InvalidField` do not exist).

- [ ] **Step 8: Implement**

`crates/fp-analysis/src/metadata.rs`: `tag_date` becomes `pub(crate)` so the sheet reads the date the way the summary does.

`crates/fp-analysis/src/tags.rs`: factor the four safe-write steps into one helper, `safe_edit`, shared by `write_tags` (which keeps its signature and tests) and the new `write_tag_sheet`; share the lofty option and the choice of tag with the new reader; add the sheet. Apply the diff (the new section at the end is the whole sheet reader and writer):

```diff
--- a/crates/fp-analysis/src/metadata.rs
+++ b/crates/fp-analysis/src/metadata.rs
@@ -29,7 +29,7 @@
 }
 
 /// The recording date of `tag` as ISO 8601 text, if it is a valid one.
-fn tag_date(tag: &lofty::tag::Tag) -> Option<String> {
+pub(crate) fn tag_date(tag: &lofty::tag::Tag) -> Option<String> {
     let text = tag.date()?.to_string();
     match fp_model::parse_tag_date(&text) {
         Ok(date) => date,
--- a/crates/fp-analysis/src/tags.rs
+++ b/crates/fp-analysis/src/tags.rs
@@ -6,14 +6,16 @@
 
 use std::path::{Path, PathBuf};
 
-use fp_model::{Limits, TrackTags};
+use fp_model::{
+    Limits, TagField, TagFieldKind, TagSheet, TrackTags, changed_fields, invalid_fields,
+};
 use lofty::config::WriteOptions;
-use lofty::file::{FileType, TaggedFileExt};
+use lofty::file::{FileType, TaggedFile, TaggedFileExt};
 use lofty::prelude::*;
 use lofty::tag::items::Timestamp;
-use lofty::tag::{ItemKey, Tag};
+use lofty::tag::{ItemKey, ItemValue, Tag, TagItem, TagType};
 
-use crate::metadata::{read_tags, title_from_file_name};
+use crate::metadata::{read_tags, tag_date, title_from_file_name};
 
 /// Why tags could not be written. The original file is untouched in every
 /// case.
@@ -27,6 +29,8 @@
     Denied,
     /// The date is not an ISO 8601 date (`fp_model::parse_tag_date`).
     InvalidDate,
+    /// The value of this field is not valid (`fp_model::invalid_fields`).
+    InvalidField(TagField),
     /// Anything else, with the system's or lofty's own description.
     Other(String),
 }
@@ -38,6 +42,7 @@
             Self::NotFound => f.write_str("the file was not found"),
             Self::Denied => f.write_str("permission denied"),
             Self::InvalidDate => f.write_str("the date is not an ISO 8601 date"),
+            Self::InvalidField(field) => write!(f, "the {} is not valid", field.slug()),
             Self::Other(detail) => f.write_str(detail),
         }
     }
@@ -99,14 +104,24 @@
     after: &TrackTags,
     limits: &Limits,
 ) -> Result<(), TagWriteError> {
+    let after = after.clone().clamped(limits.max_tag_chars);
+    safe_edit(path, limits, |tag| change_tag(tag, before, &after))
+}
+
+/// The one safe way to change a file's tag: the four steps of `write_tags`
+/// around `edit`, which receives the tag the editor works on.
+fn safe_edit(
+    path: &Path,
+    limits: &Limits,
+    edit: impl FnOnce(&mut Tag) -> Result<(), TagWriteError>,
+) -> Result<(), TagWriteError> {
     // A symlink is written through, not replaced.
     let real = path.canonicalize()?;
     if !can_write_tags(&real) {
         return Err(TagWriteError::Unsupported);
     }
-    let after = after.clone().clamped(limits.max_tag_chars);
     let temp = temp_path(&real).ok_or_else(|| lofty_error("the file has no name"))?;
-    let result = rewrite(&real, &temp, before, &after, limits);
+    let result = rewrite(&real, &temp, limits, edit);
     if result.is_err() {
         let _ = std::fs::remove_file(&temp);
     }
@@ -128,38 +143,54 @@
     Some(real.with_file_name(name))
 }
 
+/// Lets lofty parse tags up to the configured cover size: its own limit is
+/// 16 MiB, which would drop every tag of a file with a large cover. Its
+/// options are per thread, so every reader and writer calls this first.
+fn allow_large_tags(limits: &Limits) {
+    let limit = usize::try_from(limits.max_cover_bytes)
+        .unwrap_or(usize::MAX)
+        .saturating_add(1024 * 1024)
+        .max(16 * 1024 * 1024);
+    lofty::config::apply_global_options(
+        lofty::config::GlobalOptions::new().allocation_limit(limit),
+    );
+}
+
+/// The tag the editor shows and edits: the primary one of the format, else
+/// the first (the tag `read_tags` reads).
+fn edited_tag(file: &TaggedFile) -> Option<&Tag> {
+    file.primary_tag().or_else(|| file.first_tag())
+}
+
+fn edited_tag_mut(file: &mut TaggedFile) -> Option<&mut Tag> {
+    if file.primary_tag().is_some() {
+        file.primary_tag_mut()
+    } else {
+        file.first_tag_mut()
+    }
+}
+
 fn rewrite(
     real: &Path,
     temp: &Path,
-    before: &TrackTags,
-    after: &TrackTags,
     limits: &Limits,
+    edit: impl FnOnce(&mut Tag) -> Result<(), TagWriteError>,
 ) -> Result<(), TagWriteError> {
     std::fs::copy(real, temp)?;
     // Parsed with its covers: if that fails the write fails, because saving
     // a tag read without them would drop them.
-    let limit = usize::try_from(limits.max_cover_bytes)
-        .unwrap_or(usize::MAX)
-        .saturating_add(1024 * 1024)
-        .max(16 * 1024 * 1024);
-    lofty::config::apply_global_options(
-        lofty::config::GlobalOptions::new().allocation_limit(limit),
-    );
+    allow_large_tags(limits);
     let mut file = std::panic::catch_unwind(|| lofty::read_from_path(temp))
         .map_err(|_| lofty_error("the tag parser failed"))?
         .map_err(lofty_error)?;
-    // The tag `read_tags` shows: the primary one, else the first.
-    if file.primary_tag().is_none() && file.first_tag().is_none() {
+    if edited_tag(&file).is_none() {
         let ty = file.primary_tag_type();
         file.insert_tag(Tag::new(ty));
     }
-    let tag = match file.primary_tag_mut() {
-        Some(tag) => tag,
-        None => file
-            .first_tag_mut()
-            .ok_or_else(|| lofty_error("the file has no tag to edit"))?,
-    };
-    change_tag(tag, before, after)?;
+    let tag =
+        edited_tag_mut(&mut file).ok_or_else(|| lofty_error("the file has no tag to edit"))?;
+    urls_as_text(tag);
+    edit(tag)?;
     file.save_to_path(temp, WriteOptions::default())
         .map_err(lofty_error)?;
     std::fs::OpenOptions::new()
@@ -170,6 +201,28 @@
     Ok(())
 }
 
+/// ID3v2 URL frames (`WOAR`, `WCOM`, ...) come out of a parsed file as
+/// `ItemValue::Locator`, and lofty 0.25.4 drops a locator when it builds the
+/// frames of a saved tag. Re-adding them as text makes it write them back as
+/// URL frames, so a save never loses a URL the editor does not show.
+fn urls_as_text(tag: &mut Tag) {
+    if tag.tag_type() != TagType::Id3v2 {
+        return;
+    }
+    let mut keys = Vec::new();
+    for item in tag.items() {
+        if item.value().locator().is_some() && !keys.contains(&item.key()) {
+            keys.push(item.key());
+        }
+    }
+    for key in keys {
+        let urls: Vec<String> = tag.take_strings(key).collect();
+        for url in urls {
+            tag.push(TagItem::new(key, ItemValue::Text(url)));
+        }
+    }
+}
+
 fn change_tag(tag: &mut Tag, before: &TrackTags, after: &TrackTags) -> Result<(), TagWriteError> {
     fn text(tag: &mut Tag, old: &str, new: &str, set: fn(&mut Tag, String), remove: fn(&mut Tag)) {
         if old == new {
@@ -245,4 +298,242 @@
         }
     }
     Ok(())
+}
+
+// ---------------------------------------------------------------------------
+// The tag sheet: every field of the editor, read from and written to the tag.
+// ---------------------------------------------------------------------------
+
+/// The item keys a field may be stored under, best first. A tag type stores
+/// the field under the first of them its format maps (BPM is `TBPM` in
+/// ID3v2, which lofty calls `IntegerBpm`; lyrics are `USLT` there, which it
+/// calls `UnsyncLyrics`).
+fn candidates(field: TagField) -> &'static [ItemKey] {
+    use TagField as F;
+    match field {
+        F::Title => &[ItemKey::TrackTitle],
+        F::Artist => &[ItemKey::TrackArtist],
+        F::Album => &[ItemKey::AlbumTitle],
+        F::AlbumArtist => &[ItemKey::AlbumArtist],
+        F::Date => &[ItemKey::RecordingDate],
+        F::TrackNumber => &[ItemKey::TrackNumber],
+        F::DiscNumber => &[ItemKey::DiscNumber],
+        F::Genre => &[ItemKey::Genre],
+        F::Composer => &[ItemKey::Composer],
+        F::Comment => &[ItemKey::Comment],
+        F::Subtitle => &[ItemKey::TrackSubtitle],
+        F::Grouping => &[ItemKey::ContentGroup],
+        F::Bpm => &[ItemKey::IntegerBpm, ItemKey::Bpm],
+        F::InitialKey => &[ItemKey::InitialKey],
+        F::Mood => &[ItemKey::Mood],
+        F::Isrc => &[ItemKey::Isrc],
+        F::Publisher => &[ItemKey::Publisher],
+        F::CatalogNumber => &[ItemKey::CatalogNumber],
+        F::Copyright => &[ItemKey::CopyrightMessage],
+        F::OriginalArtist => &[ItemKey::OriginalArtist],
+        F::OriginalAlbum => &[ItemKey::OriginalAlbumTitle],
+        F::OriginalReleaseDate => &[ItemKey::OriginalReleaseDate],
+        F::Lyricist => &[ItemKey::Lyricist],
+        F::Conductor => &[ItemKey::Conductor],
+        F::Remixer => &[ItemKey::Remixer],
+        F::Arranger => &[ItemKey::Arranger],
+        F::Performer => &[ItemKey::Performer],
+        F::Language => &[ItemKey::Language],
+        F::EncodedBy => &[ItemKey::EncodedBy],
+        F::Lyrics => &[ItemKey::Lyrics, ItemKey::UnsyncLyrics],
+        F::SortTitle => &[ItemKey::TrackTitleSortOrder],
+        F::SortArtist => &[ItemKey::TrackArtistSortOrder],
+        F::SortAlbum => &[ItemKey::AlbumTitleSortOrder],
+        F::SortAlbumArtist => &[ItemKey::AlbumArtistSortOrder],
+        F::SortComposer => &[ItemKey::ComposerSortOrder],
+        F::ArtistWebsite => &[ItemKey::TrackArtistUrl],
+    }
+}
+
+/// Where one field lives in one tag type.
+#[derive(Debug, Clone, Copy)]
+struct Slot {
+    field: TagField,
+    key: ItemKey,
+    /// The total's key of a number and total field, if the type has one.
+    total: Option<ItemKey>,
+}
+
+impl Slot {
+    /// The slot of `field` in tags of `tag_type`, if the format can store it.
+    fn of(field: TagField, tag_type: TagType) -> Option<Self> {
+        let stores = |key: &ItemKey| ItemKey::supported_keys(tag_type).contains(key);
+        let key = candidates(field).iter().copied().find(stores)?;
+        let total = match field {
+            TagField::TrackNumber => Some(ItemKey::TrackTotal),
+            TagField::DiscNumber => Some(ItemKey::DiscTotal),
+            _ => None,
+        }
+        .filter(stores);
+        Some(Self { field, key, total })
+    }
+
+    /// Items of these keys, with no description, are the field's.
+    fn owns(&self, item: &TagItem) -> bool {
+        if !item.description().is_empty() {
+            return false;
+        }
+        let key = item.key();
+        key == self.key
+            || self.total == Some(key)
+            || (self.field == TagField::Date && key == ItemKey::Year)
+    }
+}
+
+/// Every field `tag_type` can store, in editor order.
+pub fn storable_fields(tag_type: TagType) -> Vec<TagField> {
+    slots(tag_type).into_iter().map(|s| s.field).collect()
+}
+
+fn slots(tag_type: TagType) -> Vec<Slot> {
+    TagField::ALL
+        .into_iter()
+        .filter_map(|field| Slot::of(field, tag_type))
+        .collect()
+}
+
+/// The text of an item; an ID3v2 URL is a locator, not text.
+fn item_text(item: &TagItem) -> Option<&str> {
+    item.value().text().or_else(|| item.value().locator())
+}
+
+fn texts(tag: &Tag, slot: &Slot, key: ItemKey) -> Vec<String> {
+    tag.get_items(key)
+        .filter(|item| slot.owns(item))
+        .filter_map(item_text)
+        .map(str::to_owned)
+        .collect()
+}
+
+fn read_slot(tag: &Tag, slot: &Slot) -> Vec<String> {
+    match slot.field.kind() {
+        TagFieldKind::Date if slot.field == TagField::Date => tag_date(tag).into_iter().collect(),
+        TagFieldKind::Pair => {
+            let number = texts(tag, slot, slot.key).into_iter().next();
+            let total = slot
+                .total
+                .and_then(|key| texts(tag, slot, key).into_iter().next());
+            // Some taggers write "3/12" into a number key of their own.
+            match (number, total) {
+                (Some(n), None) => match n.split_once('/') {
+                    Some((n, t)) => vec![n.to_owned(), t.to_owned()],
+                    None => vec![n, String::new()],
+                },
+                (n, t) => vec![n.unwrap_or_default(), t.unwrap_or_default()],
+            }
+        }
+        _ => texts(tag, slot, slot.key),
+    }
+}
+
+/// The sheet of `tag`: the values of the fields its format can store, and
+/// how many other tags (other keys, items with a description, pictures) it
+/// keeps.
+fn sheet_of(tag: &Tag, limits: &Limits) -> TagSheet {
+    let slots = slots(tag.tag_type());
+    let other_items = tag
+        .items()
+        .filter(|item| !slots.iter().any(|slot| slot.owns(item)))
+        .count();
+    let other = other_items + usize::try_from(tag.picture_count()).unwrap_or(usize::MAX);
+    let mut sheet = TagSheet::new(
+        slots.iter().map(|s| s.field),
+        other,
+        tag.has_format_specific_items(),
+    );
+    for slot in &slots {
+        sheet.set_values(slot.field, read_slot(tag, slot));
+    }
+    sheet.clamped(limits.max_tag_chars, limits.max_tag_values)
+}
+
+/// The tag sheet of `path`, or `None` when the format has no writable tags
+/// or the file cannot be parsed (a panic in the parser included). A file
+/// with no tag yet gives an empty sheet of its format's primary tag type.
+pub fn read_tag_sheet(path: &Path, limits: &Limits) -> Option<TagSheet> {
+    if !can_write_tags(path) {
+        return None;
+    }
+    allow_large_tags(limits);
+    let file = std::panic::catch_unwind(|| lofty::read_from_path(path))
+        .map_err(|_| tracing::warn!(path = %path.display(), "the tag parser failed"))
+        .ok()?
+        .map_err(|e| tracing::debug!(path = %path.display(), "cannot read the tags: {e}"))
+        .ok()?;
+    Some(match edited_tag(&file) {
+        Some(tag) => sheet_of(tag, limits),
+        None => sheet_of(&Tag::new(file.primary_tag_type()), limits),
+    })
+}
+
+/// Writes the fields where `after` differs from `before` into the file,
+/// through the same safe copy as `write_tags`. A changed field is replaced
+/// by its new values, one item per value, or removed when it is empty. Items
+/// the sheet does not own (other keys, items with a description such as an
+/// ID3v2 comment with a description, custom frames, pictures) are not touched.
+///
+/// Refuses, before touching the disk, a sheet with a field that
+/// `fp_model::invalid_fields` reports.
+pub fn write_tag_sheet(
+    path: &Path,
+    before: &TagSheet,
+    after: &TagSheet,
+    limits: &Limits,
+) -> Result<(), TagWriteError> {
+    let after = after
+        .clone()
+        .clamped(limits.max_tag_chars, limits.max_tag_values);
+    if let Some(field) = invalid_fields(before, &after).first() {
+        return Err(TagWriteError::InvalidField(*field));
+    }
+    safe_edit(path, limits, |tag| change_sheet(tag, before, &after))
+}
+
+fn change_sheet(tag: &mut Tag, before: &TagSheet, after: &TagSheet) -> Result<(), TagWriteError> {
+    for field in changed_fields(before, after) {
+        let slot = Slot::of(field, tag.tag_type())
+            .ok_or_else(|| lofty_error("the tag cannot store this field"))?;
+        let values = after.values(field);
+        match field.kind() {
+            TagFieldKind::Date if field == TagField::Date => match values.first() {
+                Some(text) => {
+                    let timestamp = text
+                        .parse::<Timestamp>()
+                        .map_err(|_| TagWriteError::InvalidDate)?;
+                    tag.set_date(timestamp);
+                }
+                None => tag.remove_date(),
+            },
+            TagFieldKind::Pair => {
+                let (number, total) = after.pair(field);
+                replace(tag, &slot, slot.key, &[number])?;
+                if let Some(key) = slot.total {
+                    replace(tag, &slot, key, &[total])?;
+                }
+            }
+            _ => replace(tag, &slot, slot.key, values)?,
+        }
+    }
+    Ok(())
+}
+
+/// Replaces the field's items under `key` by one item per non-empty value.
+fn replace(
+    tag: &mut Tag,
+    slot: &Slot,
+    key: ItemKey,
+    values: &[String],
+) -> Result<(), TagWriteError> {
+    drop(tag.take_filter(key, |item| slot.owns(item)));
+    for value in values.iter().filter(|v| !v.is_empty()) {
+        if !tag.push(TagItem::new(key, ItemValue::Text(value.clone()))) {
+            return Err(lofty_error("the tag cannot store this field"));
+        }
+    }
+    Ok(())
 }
```

Why each part is shaped as it is:

- `candidates` gives each field its `ItemKey`s in order; `Slot::of(field, tag_type)` picks the first the tag type supports, so the same sheet works for ID3v2, Vorbis, MP4, APE, RIFF INFO and AIFF text. `storable_fields(tag_type)` is the list `Add field` is filtered by.
- `Slot::owns` is what a field may read and replace: items of its key (a date also owns `Year`, a number its total) with an empty description. Everything else is "other": counted in `TagSheet::other_kept` together with the pictures, never touched.
- A field is replaced by `take_filter` (remove its items) and `push` (one item per value). A changed number or total rewrites both items of the pair.
- `write_tag_sheet` clamps the new sheet (`max_tag_chars`, `max_tag_values`), refuses invalid values with `InvalidField` before copying anything, and writes only `changed_fields`.
- `read_tag_sheet` never panics (the parser runs under `catch_unwind`) and degrades to `None` with a log line.

- [ ] **Step 9: Watch the URL tests guard the defect**

Temporarily comment out the `urls_as_text(tag);` line in `rewrite` and run `cargo test -p fp-analysis --test tag_sheet`.
Expected: exactly `custom_items_and_pictures_survive_an_edit_untouched` and `a_summary_write_keeps_the_url_frames_too` FAIL. Restore the line.

- [ ] **Step 10: Run the tests to verify they pass**

Run: `cargo test -p fp-analysis --test tag_sheet` (18 tests) and `cargo test -p fp-analysis --test tags` (the 20 tests of Task 2 still pass through the refactor).
Expected: PASS.

- [ ] **Step 11: Commit**

```bash
if cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings \
   && cargo test --workspace; then
  git add crates/fp-analysis
  git commit -m "feat(analysis): read and write the whole tag sheet through the safe copy" -m "One helper now does the copy, write, fsync and rename for both the summary write and the sheet write. The sheet changes only the fields the operator changed and never touches items it does not own. ID3v2 URL frames are re-added as text first because lofty drops parsed locators on save."
fi
```

#### Part C: the worker reads and writes sheets

- [ ] **Step 12: Write the failing tests**

In the `tests` module of `crates/fp-app/src/tags.rs` add the tests below (a `wav` helper, `next_outcome`, `sheet_job`, and five tests: a sheet read, a read of a file without tags, a sheet write answered with both read-backs, a failing write, and the shutdown rule for sheet reads and writes). The two tests that already exist stay.

```diff
--- pieces/tags_old_tests.rs	2026-10-02 13:53:58.437019331 +0200
+++ pieces/tags_new_tests.rs	2026-10-02 13:53:58.437079962 +0200
@@ -90,4 +90,147 @@
         assert_eq!(outcomes, 2, "the read in progress and the write");
         assert_eq!(read_track_tags(&path, &limits).title, "Saved at shutdown");
     }
+
+    fn wav(dir: &std::path::Path, name: &str) -> PathBuf {
+        let path = dir.join(name);
+        let spec = hound::WavSpec {
+            channels: 1,
+            sample_rate: 44_100,
+            bits_per_sample: 16,
+            sample_format: hound::SampleFormat::Int,
+        };
+        let mut w = hound::WavWriter::create(&path, spec).unwrap();
+        for i in 0..4_410 {
+            w.write_sample((i % 100) as i16).unwrap();
+        }
+        w.finalize().unwrap();
+        path
+    }
+
+    fn next_outcome(worker: &TagWorker) -> TagOutcome {
+        worker
+            .results()
+            .recv_timeout(std::time::Duration::from_secs(10))
+            .expect("an outcome")
+    }
+
+    fn sheet_job(track: u64, path: &std::path::Path) -> TagJob {
+        TagJob::ReadSheet {
+            track: TrackId(track),
+            path: path.to_path_buf(),
+            limits: Limits::default(),
+        }
+    }
+
+    #[test]
+    fn a_sheet_read_answers_with_the_sheet() {
+        let dir = tempfile::tempdir().unwrap();
+        let path = wav(dir.path(), "x.wav");
+        let worker = TagWorker::spawn(Box::new(|| {})).unwrap();
+        assert!(worker.submit(sheet_job(7, &path)));
+        match next_outcome(&worker) {
+            TagOutcome::SheetRead {
+                track,
+                sheet: Some(sheet),
+            } => {
+                assert_eq!(track, TrackId(7));
+                assert!(sheet.can_store(fp_model::TagField::AlbumArtist));
+            }
+            other => panic!("{other:?}"),
+        }
+    }
+
+    #[test]
+    fn a_sheet_read_of_a_file_without_tags_answers_none() {
+        let worker = TagWorker::spawn(Box::new(|| {})).unwrap();
+        assert!(worker.submit(sheet_job(1, std::path::Path::new("/nonexistent/x.wav"))));
+        assert_eq!(
+            next_outcome(&worker),
+            TagOutcome::SheetRead {
+                track: TrackId(1),
+                sheet: None
+            }
+        );
+    }
+
+    #[test]
+    fn a_sheet_write_answers_with_the_file_as_it_is_now() {
+        let dir = tempfile::tempdir().unwrap();
+        let path = wav(dir.path(), "x.wav");
+        let limits = Limits::default();
+        let before = read_tag_sheet(&path, &limits).unwrap();
+        let mut after = before.clone();
+        after.set_text(fp_model::TagField::Title, "Sheet title", false);
+        after.set_text(fp_model::TagField::Mood, "Calm", false);
+        let worker = TagWorker::spawn(Box::new(|| {})).unwrap();
+        assert!(worker.submit(TagJob::WriteSheet {
+            track: TrackId(3),
+            path: path.clone(),
+            before: Box::new(before),
+            after: Box::new(after.clone()),
+            limits,
+        }));
+        match next_outcome(&worker) {
+            TagOutcome::SheetWritten {
+                track,
+                result: Ok(saved),
+            } => {
+                assert_eq!(track, TrackId(3));
+                assert_eq!(saved.tags.title, "Sheet title");
+                assert_eq!(saved.sheet, Some(after), "the file kept all of it");
+            }
+            other => panic!("{other:?}"),
+        }
+    }
+
+    #[test]
+    fn a_sheet_write_that_fails_says_why() {
+        let path = PathBuf::from("/nonexistent/folder/x.wav");
+        let before = TagSheet::new([fp_model::TagField::Title], 0, false);
+        let mut after = before.clone();
+        after.set_text(fp_model::TagField::Title, "x", false);
+        let worker = TagWorker::spawn(Box::new(|| {})).unwrap();
+        assert!(worker.submit(TagJob::WriteSheet {
+            track: TrackId(4),
+            path,
+            before: Box::new(before),
+            after: Box::new(after),
+            limits: Limits::default(),
+        }));
+        assert_eq!(
+            next_outcome(&worker),
+            TagOutcome::SheetWritten {
+                track: TrackId(4),
+                result: Err(TagWriteError::NotFound)
+            }
+        );
+    }
+
+    #[test]
+    fn dropping_the_worker_skips_queued_sheet_reads_and_runs_sheet_writes() {
+        let dir = tempfile::tempdir().unwrap();
+        let path = wav(dir.path(), "x.wav");
+        let limits = Limits::default();
+        let before = read_tag_sheet(&path, &limits).unwrap();
+        let mut after = before.clone();
+        after.set_text(fp_model::TagField::Title, "Saved at shutdown", false);
+        let outcomes = outcomes_after_drop(|worker| {
+            for n in 1..=QUEUED {
+                assert!(worker.submit(sheet_job(n as u64, &path)));
+            }
+            assert!(worker.submit(TagJob::WriteSheet {
+                track: TrackId(99),
+                path: path.clone(),
+                before: Box::new(before.clone()),
+                after: Box::new(after.clone()),
+                limits: limits.clone(),
+            }));
+        });
+        assert_eq!(outcomes, 2, "the read in progress and the write");
+        let sheet = read_tag_sheet(&path, &limits).unwrap();
+        assert_eq!(
+            sheet.values(fp_model::TagField::Title),
+            ["Saved at shutdown"]
+        );
+    }
 }
```

- [ ] **Step 13: Run the tests to verify they fail**

Run: `cargo test -p fp-app --lib tags::`
Expected: FAIL to compile (`TagJob::ReadSheet`, `TagJob::WriteSheet`, `TagOutcome::SheetRead`, `TagOutcome::SheetWritten`, `SheetSaved` do not exist).

- [ ] **Step 14: Implement**

Add the sheet jobs and outcomes to the worker. After a sheet write the worker reads the file twice: the summary `TrackTags` (what the library keeps, sent through `ApplyTags`) and the sheet (so the application can tell which changed fields the file did not keep). The job kind is now an enum so a panic is answered with the right outcome and queued reads of both kinds are skipped at shutdown while writes still run.

```diff
--- a/crates/fp-app/src/tags.rs
+++ b/crates/fp-app/src/tags.rs
@@ -10,12 +10,29 @@
 use std::thread::JoinHandle;
 
 use crossbeam_channel::{Receiver, Sender};
-use fp_analysis::tags::{TagWriteError, read_track_tags, write_tags};
-use fp_model::{Limits, TrackId, TrackTags};
+use fp_analysis::tags::{
+    TagWriteError, read_tag_sheet, read_track_tags, write_tag_sheet, write_tags,
+};
+use fp_model::{Limits, TagSheet, TrackId, TrackTags};
 
 /// One piece of work for the worker.
 #[derive(Debug, Clone)]
 pub enum TagJob {
+    /// Read the whole tag sheet of a file (the editor opening).
+    ReadSheet {
+        track: TrackId,
+        path: PathBuf,
+        limits: Limits,
+    },
+    /// Write the fields where `after` differs from `before`, then read the
+    /// file again, both as a sheet and as the summary the library keeps.
+    WriteSheet {
+        track: TrackId,
+        path: PathBuf,
+        before: Box<TagSheet>,
+        after: Box<TagSheet>,
+        limits: Limits,
+    },
     /// Read the tags of a file (the tag-only pass).
     Read {
         track: TrackId,
@@ -33,9 +50,28 @@
     },
 }
 
+/// What the file holds after a sheet was written.
+#[derive(Debug, Clone, PartialEq)]
+pub struct SheetSaved {
+    /// For the library (`Command::ApplyTags`).
+    pub tags: TrackTags,
+    /// For comparing with what was written; `None` if the file could not be
+    /// read again as a sheet.
+    pub sheet: Option<TagSheet>,
+}
+
 /// What a job produced.
 #[derive(Debug, Clone, PartialEq)]
 pub enum TagOutcome {
+    /// `None`: the format has no writable tags or the file cannot be read.
+    SheetRead {
+        track: TrackId,
+        sheet: Option<Box<TagSheet>>,
+    },
+    SheetWritten {
+        track: TrackId,
+        result: Result<Box<SheetSaved>, TagWriteError>,
+    },
     Read {
         track: TrackId,
         tags: TrackTags,
@@ -49,6 +85,29 @@
 
 fn run(job: TagJob) -> TagOutcome {
     match job {
+        TagJob::ReadSheet {
+            track,
+            path,
+            limits,
+        } => TagOutcome::SheetRead {
+            track,
+            sheet: read_tag_sheet(&path, &limits).map(Box::new),
+        },
+        TagJob::WriteSheet {
+            track,
+            path,
+            before,
+            after,
+            limits,
+        } => {
+            let result = write_tag_sheet(&path, &before, &after, &limits).map(|()| {
+                Box::new(SheetSaved {
+                    tags: read_track_tags(&path, &limits),
+                    sheet: read_tag_sheet(&path, &limits),
+                })
+            });
+            TagOutcome::SheetWritten { track, result }
+        }
         TagJob::Read {
             track,
             path,
@@ -71,24 +130,62 @@
     }
 }
 
+/// What a job is, for answering when it panics and for skipping it at
+/// shutdown.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+enum Kind {
+    Read,
+    ReadSheet,
+    Write,
+    WriteSheet,
+}
+
+impl TagJob {
+    fn kind(&self) -> Kind {
+        match self {
+            Self::Read { .. } => Kind::Read,
+            Self::ReadSheet { .. } => Kind::ReadSheet,
+            Self::Write { .. } => Kind::Write,
+            Self::WriteSheet { .. } => Kind::WriteSheet,
+        }
+    }
+
+    fn track(&self) -> TrackId {
+        match self {
+            Self::Read { track, .. }
+            | Self::ReadSheet { track, .. }
+            | Self::Write { track, .. }
+            | Self::WriteSheet { track, .. } => *track,
+        }
+    }
+
+    /// A read nobody will see is not worth the disk once the worker is
+    /// being dropped; a write is a save the operator asked for.
+    fn is_read(&self) -> bool {
+        matches!(self.kind(), Kind::Read | Kind::ReadSheet)
+    }
+}
+
 /// A panic inside a job is a failed job, not a dead worker.
 fn run_contained(job: TagJob) -> TagOutcome {
-    let (track, write) = match &job {
-        TagJob::Read { track, .. } => (*track, false),
-        TagJob::Write { track, .. } => (*track, true),
-    };
+    let (track, kind) = (job.track(), job.kind());
     catch_unwind(AssertUnwindSafe(|| run(job))).unwrap_or_else(|_| {
         tracing::error!(?track, "a tag job panicked");
-        if write {
-            TagOutcome::Written {
-                track,
-                result: Err(TagWriteError::Other("the tag code failed".to_owned())),
-            }
-        } else {
-            TagOutcome::Read {
+        let failed = || TagWriteError::Other("the tag code failed".to_owned());
+        match kind {
+            Kind::Read => TagOutcome::Read {
                 track,
                 tags: TrackTags::default(),
-            }
+            },
+            Kind::ReadSheet => TagOutcome::SheetRead { track, sheet: None },
+            Kind::Write => TagOutcome::Written {
+                track,
+                result: Err(failed()),
+            },
+            Kind::WriteSheet => TagOutcome::SheetWritten {
+                track,
+                result: Err(failed()),
+            },
         }
     })
 }
@@ -115,7 +212,7 @@
                 while let Ok(job) = jobs_rx.recv() {
                     // A read that nobody will see is not worth the disk; a
                     // write is a save the operator asked for and still runs.
-                    if stopped.load(Ordering::Acquire) && matches!(job, TagJob::Read { .. }) {
+                    if stopped.load(Ordering::Acquire) && job.is_read() {
                         continue;
                     }
                     if results_tx.send(run_contained(job)).is_err() {
```

- [ ] **Step 15: Run the tests to verify they pass**

Run: `cargo test -p fp-app --lib tags::` (7 tests) and `cargo test -p fp-app --test services` (the services thread still sends only `Read` jobs).
Expected: PASS.

- [ ] **Step 16: Commit**

```bash
if cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings \
   && cargo test --workspace; then
  git add crates/fp-app
  git commit -m "feat(app): the tag worker reads and writes whole tag sheets"
fi
```

---

### Task 6: The tag editor (UI)

**Files:**
- Modify: `crates/fp-app/src/ui/view.rs` (`tag_edit_availability`)
- Create: `crates/fp-app/src/ui/tag_editor.rs`
- Modify: `crates/fp-app/src/ui.rs` (`mod tag_editor;`)
- Modify: `crates/fp-app/src/ui/table.rs` (`context_menu` returns the track to edit; the item)
- Modify: `crates/fp-app/src/ui/app.rs` (`ViewState::edit_tags` and `tag_editor`, the worker, the modal, the outcomes, the keyboard gate)
- Modify: `crates/fp-app/locales/en-US/main.ftl`, `es-ES/main.ftl`
- Test: Modify `crates/fp-app/tests/track_tags_ui.rs`, `crates/fp-app/tests/view.rs`

**Interfaces:**
- Consumes: Task 5 (`TagField`, `TagFieldKind`, `TagSheet`, `changed_fields`, `invalid_fields`, `unstored_fields`, `read_tag_sheet`, `TagJob::{ReadSheet, WriteSheet}`, `TagOutcome::{SheetRead, SheetWritten}`, `SheetSaved`, `TagWriteError::InvalidField`), Task 1 (`tag_edit_block`, `TagEditBlock`, `Command::ApplyTags`), `fp_analysis::tags::can_write_tags`.
- Produces:
  - `view::tag_edit_availability(state: &AppState, track: TrackId) -> Option<TagEditBlock>`: `tag_edit_block(state, track, can_write_tags(&path))`, with the extension check on the track's path (no I/O).
  - `pub(crate) struct TagEditor` (in `tag_editor.rs`): `TagEditor::reading(track) -> TagEditor` (shows "Reading tags…"), `fn arrived(&mut self, sheet: Option<TagSheet>)`, `fn changed(&self) -> Vec<TagField>`, `fn invalid(&self) -> Vec<TagField>`, `fn can_save(&self) -> bool`, `fn save_job(&self, path: &Path, limits: &Limits) -> Option<TagJob>`, `fn unstored(&self, saved: Option<&TagSheet>, limits: &Limits) -> Vec<TagField>`, `pub track`, `pub saving`, `pub error`; `pub(crate) fn show(ctx, scene, editor: &mut TagEditor, block: Option<TagEditBlock>) -> EditorAnswer` with `pub(crate) enum EditorAnswer { Open, Cancel, Save }`; `field_key(TagField) -> String` (`tag-field-<slug>`) and `block_key(TagEditBlock) -> &'static str`.
  - `ViewState::edit_tags: Option<TrackId>` (the table's request, taken at the start of the next frame) and `ViewState::tag_editor: Option<TagEditor>`; `AppUi` owns `tag_worker: Option<TagWorker>` (spawned at first use, with `ctx.request_repaint` as its repaint callback).
  - Locale keys (both locales): `menu-edit-tags` and `menu-edit-tags-file`, `-format`, `-unread`, `-on-air`, `-cued`, `-cart` (the disabled reasons), `tags-editor-title`, `tags-reading`, `tags-unreadable`, `tags-add-field`, `tags-total`, `tags-not-stored`, `tags-date-invalid`, `tags-number-invalid`, `tags-others-kept`, `tags-others-kept-more`, `tags-others-kept-uncounted`, `tags-save`, `tags-cancel`, `tags-saving`, `tags-saved`, `tags-saved-partly`, `tags-save-failed`, `tags-error-unsupported`, `-not-found`, `-denied`, `-invalid-date`, `-invalid-field`, `-other`, `-worker`, and the 36 labels `tag-field-<slug>`.

How the modal behaves (the spec's "Editor" bullet):

- Opening it queues `TagJob::ReadSheet` and shows "Reading tags…" (Save disabled, Cancel enabled). The answer fills the form; `None` shows "The tags of this file cannot be read…".
- Always-shown fields come first, in order; each optional field the file has follows; the **Add field** menu lists only optional fields the format can store (`TagSheet::addable_fields`). An always-shown field the format cannot store (album artist and disc number in RIFF INFO) is drawn disabled with "This format cannot store this field."
- A field with several values (two artists, or a field the file holds twice) and the multi-value fields are one multi-line box, one value per line; Comment and Lyrics are multi-line boxes with one value; a track or disc field is two small boxes, number and total; Date and BPM are short boxes.
- A changed field with an invalid value is marked (amber label and frame, a hint under it) and **Save** is disabled; so it is when nothing changed, while a save runs, and when `tag_edit_block` says the track went on air.
- "N other tags are kept as they are." counts the sheet's other tags; the "more" wording says lofty also holds frames it cannot count.
- **Save** is judged against the rule again in the frame it is pressed. The job runs on the tag worker. On success the library takes the read-back tags (`ApplyTags`), the modal closes and the notice area says so, or says which changed fields the file did not keep. On failure the modal stays open with the operator's text, the reason shown in the modal and in the notice area.
- The modal is modal for the keyboard too: no shortcut, not even Delete, acts under it.

- [ ] **Step 1: Write the failing tests**

In `crates/fp-app/tests/view.rs` (it also adds `tag_edit_availability` to the `use fp_app::ui::view::{...}` list):

```diff
--- a/crates/fp-app/tests/view.rs
+++ b/crates/fp-app/tests/view.rs
@@ -13,7 +13,7 @@
 use fp_app::ui::view::{
     PlayerStatus, RowStatus, TipField, cue_follow_target, cue_window_view, fader_from_gain,
     file_icon, file_problem, gain_from_fader, player_view, playlist_times, row_status, shown_entry,
-    track_tooltip, volume_db,
+    tag_edit_availability, track_tooltip, volume_db,
 };
 use fp_model::{
     AppState, AudioFormat, Command, Config, EntryId, FileState, MarkerKind, PlayerId, Track, apply,
@@ -588,3 +588,27 @@
             .all(|(f, _)| *f != TipField::Format)
     );
 }
+
+#[test]
+fn tag_edit_availability_combines_the_rule_and_the_extension() {
+    let (mut s, entries, p) = state(2);
+    for t in s.library.iter_mut() {
+        t.analyzed = true;
+        t.tags_read = true;
+    }
+    let track = |s: &AppState, e: EntryId| s.playlists.entry(e).unwrap().track;
+    let t0 = track(&s, entries[0]);
+    assert_eq!(tag_edit_availability(&s, t0), None, "a flac at rest");
+    s.library.get_mut(t0).unwrap().path = PathBuf::from("/m/a.dsf");
+    assert_eq!(
+        tag_edit_availability(&s, t0),
+        Some(fp_model::TagEditBlock::UnsupportedFormat)
+    );
+    s.library.get_mut(t0).unwrap().path = PathBuf::from("/m/a.flac");
+    apply(&mut s, Command::SetNext(p, entries[0])).unwrap();
+    apply(&mut s, Command::Play(p)).unwrap();
+    assert_eq!(
+        tag_edit_availability(&s, t0),
+        Some(fp_model::TagEditBlock::OnAir)
+    );
+}
```

In `crates/fp-app/tests/track_tags_ui.rs` change the imports at the top as below and append the editor tests. The helpers fix the three things that make modal tests flaky: the table row is the lowest "Song 1" node (the player header shows the title too, and a CUE window or a cart can as well), a menu needs three frames to open, and the modal sizes itself over a few frames, so `open_ready` settles it before any click aims at a button. The file-based tests read the files back with `read_tag_sheet`, the same reader the editor uses.

```diff
--- a/crates/fp-app/tests/track_tags_ui.rs
+++ b/crates/fp-app/tests/track_tags_ui.rs
@@ -8,7 +8,19 @@
 
 mod support;
 
-use egui_kittest::kittest::Queryable;
-use fp_model::{AppState, AudioFormat, FileState};
+use std::path::{Path, PathBuf};
+
+use egui::accesskit::Role;
+use egui_kittest::kittest::{NodeT, Queryable};
+use fp_analysis::tags::read_tag_sheet;
+use fp_app::ui::controller::Controller;
+use fp_model::{
+    AppState, AudioFormat, Command, FileState, Limits, TagEditBlock, TagField, TagSheet,
+};
+use lofty::config::WriteOptions;
+use lofty::prelude::*;
+use lofty::tag::{ItemKey, Tag, TagType};
 use support::{harness, state};
 
+type Ui = egui_kittest::Harness<'static, fp_app::ui::app::AppUi>;
+
```

```rust
// ---------------------------------------------------------------------------
// The editor.
// ---------------------------------------------------------------------------

fn wav(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join(name);
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 8_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(&path, spec).unwrap();
    for _ in 0..800 {
        w.write_sample(0i16).unwrap();
    }
    w.finalize().unwrap();
    path
}

/// A WAV tagged as a tagging tool would: an ID3v2 tag (lofty's primary tag
/// for WAV) with two artists, a track number and total, a BPM and a title.
fn tagged_wav(dir: &Path, name: &str) -> PathBuf {
    let path = wav(dir, name);
    let mut file = lofty::read_from_path(&path).unwrap();
    let ty = file.primary_tag_type();
    file.insert_tag(Tag::new(ty));
    let tag = file.primary_tag_mut().unwrap();
    tag.set_title("Song 1".into());
    tag.set_genre("Pop".into());
    tag.push(lofty::tag::TagItem::new(
        ItemKey::TrackArtist,
        lofty::tag::ItemValue::Text("First Artist".into()),
    ));
    tag.push(lofty::tag::TagItem::new(
        ItemKey::TrackArtist,
        lofty::tag::ItemValue::Text("Second Artist".into()),
    ));
    tag.insert_text(ItemKey::TrackNumber, "3".into());
    tag.insert_text(ItemKey::TrackTotal, "12".into());
    tag.insert_text(ItemKey::IntegerBpm, "128".into());
    file.save_to_path(&path, WriteOptions::default()).unwrap();
    path
}

/// A WAV whose tag is RIFF INFO, a format that cannot store an album artist.
fn riff_wav(dir: &Path, name: &str) -> PathBuf {
    let path = wav(dir, name);
    let mut file = lofty::read_from_path(&path).unwrap();
    file.insert_tag(Tag::new(TagType::RiffInfo));
    file.tag_mut(TagType::RiffInfo)
        .unwrap()
        .set_title("Song 1".into());
    file.save_to_path(&path, WriteOptions::default()).unwrap();
    path
}

/// `tagged_state` whose first track is the file at `path`.
fn state_at(path: &Path) -> AppState {
    let mut s = tagged_state();
    let first = s.library.iter().next().unwrap().id;
    s.library.get_mut(first).unwrap().path = path.to_path_buf();
    s
}

/// Right-clicks the table row of "Song 1": the lowest node with that text,
/// as the player header and a CUE window show it higher up.
fn right_click_the_row(h: &Ui) {
    h.get_all_by_label("Song 1")
        .max_by(|a, b| a.rect().min.y.total_cmp(&b.rect().min.y))
        .unwrap()
        .click_secondary();
}

fn open_editor(h: &mut Ui) {
    right_click_the_row(h);
    h.run_steps(3);
    h.get_by_label("Edit tags…").click();
    h.run_steps(2);
}

/// Opens the editor and waits until its sheet has arrived and the modal,
/// which is centred and sizes itself over a few frames, has settled (a click
/// aimed at an earlier frame's layout would miss).
fn open_ready(h: &mut Ui) {
    open_editor(h);
    wait_for_text(h, "Add field");
    h.run_steps(10);
}

fn wait_for(h: &mut Ui, what: &str, done: impl Fn(&mut Ui) -> bool) {
    for _ in 0..300 {
        h.run_steps(1);
        if done(h) {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    panic!("never happened: {what}");
}

fn wait_for_text(h: &mut Ui, text: &str) {
    wait_for(h, text, |h| {
        h.query_all_by_label_contains(text).next().is_some()
    });
}

/// The text boxes labelled `label`: one line or several.
fn inputs<'a>(h: &'a Ui, label: &'a str) -> impl Iterator<Item = egui_kittest::Node<'a>> + 'a {
    h.query_all_by_role_and_label(Role::TextInput, label)
        .chain(h.query_all_by_role_and_label(Role::MultilineTextInput, label))
}

fn input_value(h: &Ui, label: &str, nth: usize) -> String {
    inputs(h, label)
        .nth(nth)
        .unwrap()
        .accesskit_node()
        .value()
        .unwrap_or_default()
        .to_owned()
}

fn field(h: &Ui, label: &str) -> String {
    input_value(h, label, 0)
}

fn type_into(h: &mut Ui, label: &str, nth: usize, text: &str) {
    inputs(h, label).nth(nth).unwrap().focus();
    h.run_steps(2);
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    h.run_steps(1);
    if text.is_empty() {
        h.key_press(egui::Key::Delete);
        h.run_steps(1);
    }
    for c in text.chars() {
        inputs(h, label).nth(nth).unwrap().type_text(&c.to_string());
        h.run_steps(1);
    }
}

fn save_enabled(h: &Ui) -> bool {
    !h.get_by_role_and_label(Role::Button, "Save")
        .accesskit_node()
        .is_disabled()
}

fn sheet_of(path: &Path) -> TagSheet {
    read_tag_sheet(path, &Limits::default()).unwrap()
}

fn put_on_a_playing_cart(state: &mut AppState, track: fp_model::TrackId) {
    let cart = state.cartwall.pages[0].carts[0].id;
    state.cartwall.pages[0].carts[0].track = Some(track);
    state.cartwall.playing.push(fp_model::PlayingCart {
        cart,
        looped: false,
    });
}

#[test]
fn the_editor_says_it_is_reading_until_the_sheet_arrives() {
    let dir = tempfile::tempdir().unwrap();
    let (mut h, _fake) = harness(state_at(&tagged_wav(dir.path(), "a.wav")));
    open_editor(&mut h);
    // The sheet is requested on the frame that opens the editor and handled
    // on the next one, so this frame still waits for it.
    assert!(h.query_by_label("Reading tags…").is_some());
    assert!(!save_enabled(&h));
    wait_for_text(&mut h, "Add field");
    assert!(h.query_by_label("Reading tags…").is_none());
}

#[test]
fn the_editor_shows_what_the_file_holds() {
    let dir = tempfile::tempdir().unwrap();
    let (mut h, _fake) = harness(state_at(&tagged_wav(dir.path(), "a.wav")));
    open_ready(&mut h);
    assert!(h.query_by_label("Edit tags").is_some(), "the modal title");
    assert_eq!(field(&h, "Title"), "Song 1");
    assert_eq!(
        field(&h, "Artist"),
        "First Artist\nSecond Artist",
        "one value per line"
    );
    assert_eq!(field(&h, "Genre"), "Pop");
    assert_eq!(field(&h, "Track number"), "3");
    assert_eq!(field(&h, "Total"), "12", "the track total");
    assert_eq!(field(&h, "BPM"), "128", "an optional field the file has");
    for always in [
        "Album",
        "Album artist",
        "Date",
        "Disc number",
        "Composer",
        "Comment",
    ] {
        assert!(
            inputs(&h, always).next().is_some(),
            "{always} is always shown"
        );
    }
    assert!(
        inputs(&h, "Mood").next().is_none(),
        "an optional field the file lacks is not shown"
    );
}

#[test]
fn add_field_offers_what_the_format_can_store_and_shows_the_choice() {
    let dir = tempfile::tempdir().unwrap();
    let (mut h, _fake) = harness(state_at(&tagged_wav(dir.path(), "a.wav")));
    open_ready(&mut h);
    h.get_by_label("Add field").click();
    h.run_steps(3);
    let offered = |h: &Ui, name: &str| h.query_by_role_and_label(Role::Button, name).is_some();
    assert!(offered(&h, "Mood"));
    assert!(offered(&h, "Lyrics"));
    assert!(!offered(&h, "Performer"), "ID3v2 has no performer frame");
    assert!(!offered(&h, "BPM"), "the file already has the BPM");
    h.get_by_role_and_label(Role::Button, "Mood").click();
    h.run_steps(3);
    assert!(inputs(&h, "Mood").next().is_some());
}

#[test]
fn a_format_that_cannot_store_a_field_shows_it_disabled_and_never_offers_it() {
    let dir = tempfile::tempdir().unwrap();
    let (mut h, _fake) = harness(state_at(&riff_wav(dir.path(), "a.wav")));
    open_ready(&mut h);
    assert!(
        inputs(&h, "Album artist")
            .next()
            .unwrap()
            .accesskit_node()
            .is_disabled()
    );
    assert_eq!(
        h.query_all_by_label_contains("cannot store this field")
            .count(),
        2,
        "the album artist and the disc number"
    );
    h.get_by_label("Add field").click();
    h.run_steps(3);
    assert!(
        h.query_by_role_and_label(Role::Button, "Copyright")
            .is_some()
    );
    for missing in ["Mood", "Lyrics", "Publisher", "Album artist"] {
        assert!(
            h.query_by_role_and_label(Role::Button, missing).is_none(),
            "{missing} is not offered"
        );
    }
}

#[test]
fn an_unreadable_file_cannot_be_edited() {
    let (mut h, _fake) = harness(state_at(Path::new("/nonexistent/folder/a.wav")));
    open_editor(&mut h);
    wait_for_text(&mut h, "cannot be read, so they cannot be edited");
    assert!(!save_enabled(&h));
    h.get_by_role_and_label(Role::Button, "Cancel").click();
    h.run_steps(3);
    assert!(h.query_by_label("Edit tags").is_none());
}

#[test]
fn the_menu_item_is_disabled_with_the_reason() {
    let reasons = [
        (
            TagEditBlock::FileUnavailable,
            "The file is missing or cannot be read",
        ),
        (
            TagEditBlock::UnsupportedFormat,
            "This format has no writable tags",
        ),
        (TagEditBlock::TagsNotRead, "The tags have not been read yet"),
        (
            TagEditBlock::OnAir,
            "A track that is on air cannot be edited",
        ),
        (
            TagEditBlock::Cued,
            "A track that is on CUE cannot be edited",
        ),
        (
            TagEditBlock::OnCart,
            "A track on a playing cart cannot be edited",
        ),
    ];
    for (block, text) in reasons {
        let mut s = tagged_state();
        let first = s.library.iter().next().unwrap().id;
        let p = s.players[0].id;
        let e = s
            .playlists
            .get(s.playlists.first_id().unwrap())
            .unwrap()
            .entries[0]
            .id;
        match block {
            TagEditBlock::FileUnavailable => {
                s.library.get_mut(first).unwrap().file_state = FileState::Missing;
            }
            TagEditBlock::UnsupportedFormat => {
                s.library.get_mut(first).unwrap().path = PathBuf::from("/music/Song 1.dsf");
            }
            TagEditBlock::TagsNotRead => s.library.get_mut(first).unwrap().tags_read = false,
            TagEditBlock::OnAir => {
                fp_model::apply(&mut s, Command::SetNext(p, e)).unwrap();
                fp_model::apply(&mut s, Command::Play(p)).unwrap();
            }
            TagEditBlock::Cued => {
                fp_model::apply(&mut s, Command::CueEntry(p, e)).unwrap();
            }
            TagEditBlock::OnCart => {
                put_on_a_playing_cart(&mut s, first);
                s.cartwall.pages[0].carts[0].name = "Jingle".into();
            }
        }
        let (mut h, _fake) = harness(s);
        right_click_the_row(&h);
        h.run_steps(3);
        let item = h
            .query_by_label("Edit tags…")
            .unwrap_or_else(|| panic!("no menu item for {block:?}"));
        assert!(item.accesskit_node().is_disabled(), "{block:?}");
        item.hover();
        h.run_steps(40);
        assert!(
            h.query_by_label_contains(text).is_some(),
            "{block:?}: {text}"
        );
    }
}

#[test]
fn saving_writes_the_file_and_the_library_takes_the_tags() {
    let dir = tempfile::tempdir().unwrap();
    let path = tagged_wav(dir.path(), "a.wav");
    let (mut h, fake) = harness(state_at(&path));
    open_ready(&mut h);
    type_into(&mut h, "Genre", 0, "Jazz");
    type_into(&mut h, "Total", 0, "14");
    assert!(save_enabled(&h));
    h.get_by_role_and_label(Role::Button, "Save").click();
    wait_for(&mut h, "the library takes the genre", |_| {
        fake.state.load().library.iter().next().unwrap().genre == "Jazz"
    });
    let sheet = sheet_of(&path);
    assert_eq!(sheet.values(TagField::Genre), ["Jazz"]);
    assert_eq!(sheet.pair(TagField::TrackNumber), ("3".into(), "14".into()));
    assert_eq!(
        sheet.values(TagField::Artist),
        ["First Artist", "Second Artist"],
        "untouched fields stay"
    );
    let track = fake.state.load().library.iter().next().unwrap().clone();
    assert_eq!(track.date, None, "the file has no date: the read-back wins");
    assert!(track.tags_read);
    assert_eq!(track.duration_secs, 200.0, "analysis data is kept");
    h.run_steps(3);
    assert!(h.query_by_label("Edit tags").is_none(), "the modal closed");
}

#[test]
fn an_added_field_is_saved_and_an_empty_one_is_not() {
    let dir = tempfile::tempdir().unwrap();
    let path = tagged_wav(dir.path(), "a.wav");
    let (mut h, _fake) = harness(state_at(&path));
    open_ready(&mut h);
    for name in ["Mood", "ISRC"] {
        h.get_by_label("Add field").click();
        h.run_steps(3);
        h.get_by_role_and_label(Role::Button, name).click();
        h.run_steps(3);
    }
    assert!(!save_enabled(&h), "an empty added field changes nothing");
    type_into(&mut h, "Mood", 0, "Calm");
    h.get_by_role_and_label(Role::Button, "Save").click();
    wait_for(&mut h, "the file has the mood", |_| {
        sheet_of(&path).values(TagField::Mood) == ["Calm"]
    });
    assert!(!sheet_of(&path).has(TagField::Isrc));
}

#[test]
fn a_failed_save_keeps_the_modal_and_says_why() {
    let dir = tempfile::tempdir().unwrap();
    let path = tagged_wav(dir.path(), "a.wav");
    let (mut h, fake) = harness(state_at(&path));
    open_ready(&mut h);
    type_into(&mut h, "Genre", 0, "Jazz");
    // The file vanishes after the sheet was read.
    std::fs::remove_file(&path).unwrap();
    h.get_by_role_and_label(Role::Button, "Save").click();
    wait_for_text(&mut h, "were not saved");
    assert!(h.query_by_label("Edit tags").is_some(), "the modal stays");
    assert_eq!(field(&h, "Genre"), "Jazz", "the text is kept");
    assert!(
        h.query_all_by_label_contains("the file was not found")
            .count()
            >= 1,
        "the reason is shown"
    );
    assert_eq!(
        fake.state.load().library.iter().next().unwrap().genre,
        "Pop",
        "the library is unchanged"
    );
    assert!(save_enabled(&h), "the operator can try again");
}

#[test]
fn save_needs_a_change_and_valid_values() {
    let dir = tempfile::tempdir().unwrap();
    let (mut h, _fake) = harness(state_at(&tagged_wav(dir.path(), "a.wav")));
    open_ready(&mut h);
    assert!(!save_enabled(&h), "nothing changed");
    type_into(&mut h, "Date", 0, "2019-13");
    assert!(!save_enabled(&h));
    assert!(
        h.query_by_label_contains("Use YYYY, YYYY-MM or YYYY-MM-DD")
            .is_some()
    );
    type_into(&mut h, "Date", 0, "2019-05-14");
    assert!(save_enabled(&h), "a valid date");
    type_into(&mut h, "BPM", 0, "fast");
    assert!(!save_enabled(&h));
    assert!(h.query_by_label_contains("Use whole numbers").is_some());
    type_into(&mut h, "BPM", 0, "130");
    assert!(save_enabled(&h));
    type_into(&mut h, "Total", 0, "x");
    assert!(!save_enabled(&h), "a total that is not a number");
    type_into(&mut h, "Total", 0, "12");
    type_into(&mut h, "Track number", 0, "");
    assert!(!save_enabled(&h), "a total needs a number");
}

#[test]
fn cancel_closes_the_editor_and_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let path = tagged_wav(dir.path(), "a.wav");
    let before = std::fs::read(&path).unwrap();
    let (mut h, fake) = harness(state_at(&path));
    open_ready(&mut h);
    type_into(&mut h, "Genre", 0, "Jazz");
    h.get_by_role_and_label(Role::Button, "Cancel").click();
    h.run_steps(3);
    assert!(h.query_by_label("Edit tags").is_none());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert!(
        fake.take_sent()
            .iter()
            .all(|c| !matches!(c, Command::ApplyTags { .. }))
    );
}

#[test]
fn saving_is_refused_when_the_track_went_on_air() {
    let dir = tempfile::tempdir().unwrap();
    let path = tagged_wav(dir.path(), "a.wav");
    let before = std::fs::read(&path).unwrap();
    let (mut h, fake) = harness(state_at(&path));
    open_ready(&mut h);
    type_into(&mut h, "Genre", 0, "Jazz");
    // The operator (or a remote client) starts the track meanwhile.
    let p = fake.player(0);
    let e = fake.entries()[0];
    fake.send(Command::SetNext(p, e));
    fake.send(Command::Play(p));
    h.run_steps(3);
    assert!(!save_enabled(&h), "Save is off while the track is on air");
    assert!(
        h.query_by_label_contains("A track that is on air cannot be edited")
            .is_some()
    );
    assert_eq!(std::fs::read(&path).unwrap(), before);
}

#[test]
fn the_modal_closes_when_the_track_is_removed() {
    let dir = tempfile::tempdir().unwrap();
    let (mut h, fake) = harness(state_at(&tagged_wav(dir.path(), "a.wav")));
    open_ready(&mut h);
    // The library loses the track (the tests' controller has no command for
    // it, so the snapshot is edited directly).
    let mut s = (**fake.state.load()).clone();
    let ids: Vec<_> = s.library.iter().map(|t| t.id).collect();
    for id in ids {
        s.library.remove(id);
    }
    fake.state.store(std::sync::Arc::new(s));
    h.run_steps(3);
    assert!(h.query_by_label("Edit tags").is_none());
}

#[test]
fn no_shortcut_acts_under_the_editor() {
    let dir = tempfile::tempdir().unwrap();
    let (mut h, fake) = harness(state_at(&tagged_wav(dir.path(), "a.wav")));
    open_ready(&mut h);
    fake.take_sent();
    h.key_press(egui::Key::Delete);
    h.key_press(egui::Key::Space);
    h.run_steps(3);
    assert!(
        fake.take_sent().is_empty(),
        "Delete must not remove the entry"
    );
}

#[test]
fn every_field_has_a_label_in_both_languages() {
    for lang in ["en-US", "es-ES"] {
        let i18n = fp_app::i18n::I18n::new(Some(lang));
        for field in TagField::ALL {
            let key = format!("tag-field-{}", field.slug());
            assert_ne!(i18n.tr(&key), key, "{lang}: {key}");
        }
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-app --test view tag_edit_availability_combines_the_rule_and_the_extension` and `cargo test -p fp-app --test track_tags_ui the_editor_shows_what_the_file_holds`
Expected: FAIL (`tag_edit_availability` does not exist; there is no "Edit tags…" item).

- [ ] **Step 3: Implement**

`view.rs`, `ui.rs`, `table.rs`:

```diff
--- a/crates/fp-app/src/ui/view.rs
+++ b/crates/fp-app/src/ui/view.rs
@@ -448,3 +448,17 @@
     }
     parts.join(" · ")
 }
+
+/// Why the tags of `track` cannot be edited now, if they cannot: the model's
+/// rule plus the format's writability, judged from the path's extension (no
+/// I/O on the interface thread).
+pub fn tag_edit_availability(
+    state: &AppState,
+    track: fp_model::TrackId,
+) -> Option<fp_model::TagEditBlock> {
+    let writable = state
+        .library
+        .get(track)
+        .is_some_and(|t| fp_analysis::tags::can_write_tags(&t.path));
+    fp_model::tag_edit_block(state, track, writable)
+}
--- a/crates/fp-app/src/ui.rs
+++ b/crates/fp-app/src/ui.rs
@@ -17,6 +17,7 @@
 mod settings;
 pub mod shell;
 mod table;
+mod tag_editor;
 pub mod theme;
 pub mod view;
 pub mod wave_view;
--- a/crates/fp-app/src/ui/table.rs
+++ b/crates/fp-app/src/ui/table.rs
@@ -81,6 +81,8 @@
     let mut clicked: Option<EntryId> = None;
     // O17: the entry a running CUE moves to after a primary click.
     let mut cue_follow: Option<EntryId> = None;
+    // O23: the track whose tags the operator asked to edit.
+    let mut edit_tags: Option<fp_model::TrackId> = None;
     let mut dragged: Option<EntryId> = None;
     let mut builder = TableBuilder::new(ui).id_salt(("tracks", player.0));
     if reset {
@@ -370,7 +372,9 @@
                 response.context_menu(|ui| {
                     menu_open = true;
                     clicked = Some(entry.id);
-                    context_menu(ui, scene, player, playlist, entry.id, i, &track.title);
+                    if context_menu(ui, scene, player, playlist, entry.id, i, track).is_some() {
+                        edit_tags = Some(track.id);
+                    }
                 });
             });
         });
@@ -382,6 +386,9 @@
     if let Some(entry) = cue_follow {
         scene.ctl.send(Command::CueEntry(player, entry));
     }
+    if edit_tags.is_some() {
+        view_state.edit_tags = edit_tags;
+    }
     // Drop target for entries dragged inside the app.
     let pointer_in = ui
         .ctx()
@@ -490,13 +497,14 @@
     playlist: PlaylistId,
     entry: EntryId,
     index: usize,
-    title: &str,
-) {
+    track: &fp_model::Track,
+) -> Option<fp_model::TrackId> {
     let t = scene.i18n;
+    let mut edit_tags = None;
     ui.set_min_width(240.0);
     ui.add(
         egui::Label::new(
-            RichText::new(title)
+            RichText::new(&track.title)
                 .font(font(11.0))
                 .color(theme::NEUTRAL_400),
         )
@@ -561,6 +569,16 @@
         scene.ctl.send(Command::CueEntry(player, entry));
         ui.close();
     }
+    let block = view::tag_edit_availability(scene.state, track.id);
+    let edit = labelled(ui, icon::PENCIL_SIMPLE, "menu-edit-tags", block.is_none());
+    let edit = match block {
+        Some(reason) => edit.on_disabled_hover_text(t.tr(super::tag_editor::block_key(reason))),
+        None => edit,
+    };
+    if edit.clicked() {
+        edit_tags = Some(track.id);
+        ui.close();
+    }
     ui.separator();
     if labelled(ui, icon::PLUS, "menu-add-below", true).clicked() {
         scene.pick_files(playlist, index + 1);
@@ -621,6 +639,7 @@
         scene.ctl.send(Command::RemoveEntry(entry));
         ui.close();
     }
+    edit_tags
 }
 
 /// A small flag icon with `label` as its accessible name and tooltip.
```

`ui/tag_editor.rs` (new). Its `tests` module holds the unit tests of the draft logic: the sheet is taken once, an untouched draft cannot be saved, the boxes become the sheet (multi-value lines, a comment as one value), an invalid value blocks Save, the job carries the original and the draft, and what the file did not keep is named.

```rust
//! The tag editor (feedback 2 spec O23): a modal over the tag sheet of one
//! track. It edits a draft; reading the file and saving are the
//! application's job (`AppUi`), through the tag worker.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use egui::{RichText, vec2};
use fp_model::{
    Limits, TagEditBlock, TagField, TagFieldKind, TagSheet, TrackId, changed_fields,
    invalid_fields, unstored_fields,
};

use super::app::Scene;
use super::theme;
use super::widgets::{self, TileStyle, font, font_medium};
use crate::tags::TagJob;

/// What the operator did this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EditorAnswer {
    Open,
    Cancel,
    Save,
}

/// The sheet and what the text boxes hold.
struct Form {
    /// What the file held when the sheet was read.
    original: TagSheet,
    /// `original` with the operator's changes.
    edited: TagSheet,
    /// The text of each box: the text itself, or the number and the total of
    /// a track or disc field.
    boxes: BTreeMap<TagField, (String, String)>,
}

impl Form {
    fn new(original: TagSheet) -> Self {
        let boxes = TagField::ALL
            .into_iter()
            .map(|field| {
                let text = if field.kind() == TagFieldKind::Pair {
                    original.pair(field)
                } else {
                    (original.text(field), String::new())
                };
                (field, text)
            })
            .collect();
        Self {
            edited: original.clone(),
            original,
            boxes,
        }
    }

    fn changed(&self) -> Vec<TagField> {
        changed_fields(&self.original, &self.edited)
    }

    fn invalid(&self) -> Vec<TagField> {
        invalid_fields(&self.original, &self.edited)
    }

    /// Copies the boxes into the edited sheet; `true` if that changed it.
    fn sync(&mut self) -> bool {
        let before = self.edited.clone();
        for (field, (first, second)) in &self.boxes {
            if field.kind() == TagFieldKind::Pair {
                self.edited.set_pair(*field, first, second);
            } else {
                let as_lines = self.original.shows_lines(*field);
                self.edited.set_text(*field, first, as_lines);
            }
        }
        self.edited != before
    }
}

enum Phase {
    /// The tag worker is reading the file.
    Reading,
    /// The format has no writable tags or the file cannot be read.
    Unreadable,
    Ready(Box<Form>),
}

pub(crate) struct TagEditor {
    pub track: TrackId,
    /// A save is running on the tag worker.
    pub saving: bool,
    /// Why the last save failed, shown in the modal.
    pub error: Option<String>,
    /// Optional fields the operator added with **Add field**.
    added: BTreeSet<TagField>,
    phase: Phase,
}

impl TagEditor {
    /// An editor waiting for the sheet of `track`.
    pub fn reading(track: TrackId) -> Self {
        Self {
            track,
            saving: false,
            error: None,
            added: BTreeSet::new(),
            phase: Phase::Reading,
        }
    }

    /// The tag worker's answer to the read; ignored unless one is awaited.
    pub fn arrived(&mut self, sheet: Option<TagSheet>) {
        if matches!(self.phase, Phase::Reading) {
            self.phase = match sheet {
                Some(sheet) => Phase::Ready(Box::new(Form::new(sheet))),
                None => Phase::Unreadable,
            };
        }
    }

    fn form(&self) -> Option<&Form> {
        match &self.phase {
            Phase::Ready(form) => Some(form),
            _ => None,
        }
    }

    /// The fields the operator changed, in editor order.
    pub fn changed(&self) -> Vec<TagField> {
        self.form().map(Form::changed).unwrap_or_default()
    }

    /// The changed fields whose value is not valid.
    pub fn invalid(&self) -> Vec<TagField> {
        self.form().map(Form::invalid).unwrap_or_default()
    }

    /// Something changed, every changed value is valid and no save runs.
    pub fn can_save(&self) -> bool {
        !self.saving && !self.changed().is_empty() && self.invalid().is_empty()
    }

    /// The job that writes the draft, if it can be saved.
    pub fn save_job(&self, path: &Path, limits: &Limits) -> Option<TagJob> {
        let form = self.form().filter(|_| self.can_save())?;
        Some(TagJob::WriteSheet {
            track: self.track,
            path: path.to_path_buf(),
            before: Box::new(form.original.clone()),
            after: Box::new(form.edited.clone()),
            limits: limits.clone(),
        })
    }

    /// The changed fields the file does not hold as written, judged against
    /// the sheet read back after the save.
    pub fn unstored(&self, saved: Option<&TagSheet>, limits: &Limits) -> Vec<TagField> {
        match (self.form(), saved) {
            (Some(form), Some(saved)) => {
                let written = form
                    .edited
                    .clone()
                    .clamped(limits.max_tag_chars, limits.max_tag_values);
                unstored_fields(&form.original, &written, saved)
            }
            _ => Vec::new(),
        }
    }
}

/// The Fluent key of the label of `field`.
pub(crate) fn field_key(field: TagField) -> String {
    format!("tag-field-{}", field.slug())
}

/// The Fluent key of the reason `block` gives for a disabled edit.
pub(crate) fn block_key(block: TagEditBlock) -> &'static str {
    match block {
        TagEditBlock::FileUnavailable => "menu-edit-tags-file",
        TagEditBlock::UnsupportedFormat => "menu-edit-tags-format",
        TagEditBlock::TagsNotRead => "menu-edit-tags-unread",
        TagEditBlock::OnAir => "menu-edit-tags-on-air",
        TagEditBlock::Cued => "menu-edit-tags-cued",
        TagEditBlock::OnCart => "menu-edit-tags-cart",
    }
}

/// A button as wide as its label; `accent` for the main action.
fn button(ui: &mut egui::Ui, text: &str, enabled: bool, accent: bool) -> bool {
    let w = ui
        .painter()
        .layout_no_wrap(text.to_owned(), font_medium(13.0), theme::TEXT)
        .size()
        .x
        + 28.0;
    let style = if accent {
        TileStyle {
            fill: theme::ACCENT,
            border: theme::ACCENT,
            content: theme::NEUTRAL_900,
            hover_fill: theme::ACCENT_400,
            hover_content: theme::NEUTRAL_900,
            active_fill: theme::ACCENT_300,
            ..TileStyle::plain()
        }
    } else {
        TileStyle::plain()
    };
    widgets::tile(ui, vec2(w, 30.0), text, enabled, style, |p, r, c| {
        p.text(
            r.center(),
            egui::Align2::CENTER_CENTER,
            text,
            font_medium(13.0),
            c,
        );
    })
    .clicked()
}

fn note(ui: &mut egui::Ui, text: String, color: egui::Color32) {
    ui.add(
        egui::Label::new(RichText::new(text).font(font(12.0)).color(color))
            .wrap()
            .selectable(false),
    );
}

/// The boxes of one field: a number and a total, several lines, or one line.
fn field_boxes(
    ui: &mut egui::Ui,
    t: &crate::i18n::I18n,
    field: TagField,
    name: &egui::Response,
    boxes: &mut (String, String),
    lines: bool,
    max_chars: usize,
) {
    match field.kind() {
        TagFieldKind::Pair => {
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut boxes.0)
                        .char_limit(10)
                        .desired_width(64.0),
                )
                .labelled_by(name.id);
                let total = ui.label(
                    RichText::new(t.tr("tags-total"))
                        .font(font(12.0))
                        .color(theme::NEUTRAL_400),
                );
                ui.add(
                    egui::TextEdit::singleline(&mut boxes.1)
                        .char_limit(10)
                        .desired_width(64.0),
                )
                .labelled_by(total.id);
            });
        }
        TagFieldKind::Date => {
            ui.add(
                egui::TextEdit::singleline(&mut boxes.0)
                    .char_limit(19)
                    .hint_text("YYYY-MM-DD")
                    .desired_width(180.0),
            )
            .labelled_by(name.id);
        }
        TagFieldKind::Whole => {
            ui.add(
                egui::TextEdit::singleline(&mut boxes.0)
                    .char_limit(10)
                    .desired_width(90.0),
            )
            .labelled_by(name.id);
        }
        TagFieldKind::LongText | TagFieldKind::Text if lines || field == TagField::Comment => {
            let rows = boxes.0.lines().count().clamp(2, 6);
            ui.add(
                egui::TextEdit::multiline(&mut boxes.0)
                    .char_limit(max_chars)
                    .desired_rows(rows)
                    .desired_width(320.0),
            )
            .labelled_by(name.id);
        }
        TagFieldKind::LongText => {
            ui.add(
                egui::TextEdit::multiline(&mut boxes.0)
                    .char_limit(max_chars)
                    .desired_rows(6)
                    .desired_width(320.0),
            )
            .labelled_by(name.id);
        }
        TagFieldKind::Text => {
            ui.add(
                egui::TextEdit::singleline(&mut boxes.0)
                    .char_limit(max_chars)
                    .desired_width(320.0),
            )
            .labelled_by(name.id);
        }
    }
}

pub(crate) fn show(
    ctx: &egui::Context,
    scene: &Scene<'_>,
    editor: &mut TagEditor,
    block: Option<TagEditBlock>,
) -> EditorAnswer {
    let t = scene.i18n;
    let limits = &scene.state.config.limits;
    let max = limits.max_tag_chars;
    let file_name = scene
        .state
        .library
        .get(editor.track)
        .and_then(|track| track.path.file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let width = (ctx.content_rect().width() - 48.0).clamp(360.0, 560.0);
    let list_height = (ctx.content_rect().height() - 260.0).clamp(160.0, 560.0);
    let invalid = editor.invalid();
    let mut can_save = false;
    let TagEditor {
        saving,
        error,
        added,
        phase,
        ..
    } = editor;
    let saving = *saving;
    let mut answer = EditorAnswer::Open;
    let modal = egui::Modal::new(egui::Id::new("tag-editor"))
        .frame(egui::Frame::new().fill(theme::SURFACE).inner_margin(20.0))
        .backdrop_color(theme::NEUTRAL_900.gamma_multiply(0.7))
        .show(ctx, |ui| {
            ui.set_width(width);
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
            ui.add(
                egui::Label::new(
                    RichText::new(t.tr("tags-editor-title"))
                        .font(font_medium(18.0))
                        .color(theme::TEXT),
                )
                .selectable(false),
            );
            ui.add(
                egui::Label::new(
                    RichText::new(file_name)
                        .font(font(11.0))
                        .color(theme::NEUTRAL_400),
                )
                .selectable(false)
                .truncate(),
            );
            match phase {
                Phase::Reading => note(ui, t.tr("tags-reading"), theme::NEUTRAL_300),
                Phase::Unreadable => note(ui, t.tr("tags-unreadable"), theme::AMBER),
                Phase::Ready(form) => {
                    ui.add_enabled_ui(!saving, |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(list_height)
                            .auto_shrink([false, true])
                            .show(ui, |ui| {
                                egui::Grid::new("tag-editor-fields")
                                    .num_columns(2)
                                    .spacing(vec2(10.0, 6.0))
                                    .show(ui, |ui| {
                                        for field in form.original.visible_fields(added) {
                                            let bad = invalid.contains(&field);
                                            let storable = form.original.can_store(field);
                                            let label = ui.label(
                                                RichText::new(t.tr(&field_key(field)))
                                                    .font(font(12.0))
                                                    .color(if bad {
                                                        theme::AMBER
                                                    } else {
                                                        theme::NEUTRAL_400
                                                    }),
                                            );
                                            let lines = form.original.shows_lines(field);
                                            ui.vertical(|ui| {
                                                let frame = egui::Frame::new()
                                                    .inner_margin(2.0)
                                                    .corner_radius(3.0)
                                                    .stroke(if bad {
                                                        egui::Stroke::new(1.0, theme::AMBER)
                                                    } else {
                                                        egui::Stroke::NONE
                                                    });
                                                frame.show(ui, |ui| {
                                                    ui.add_enabled_ui(storable, |ui| {
                                                        if let Some(boxes) =
                                                            form.boxes.get_mut(&field)
                                                        {
                                                            field_boxes(
                                                                ui, t, field, &label, boxes, lines,
                                                                max,
                                                            );
                                                        }
                                                    });
                                                });
                                                if bad {
                                                    let key = if field.kind() == TagFieldKind::Date
                                                    {
                                                        "tags-date-invalid"
                                                    } else {
                                                        "tags-number-invalid"
                                                    };
                                                    note(ui, t.tr(key), theme::AMBER);
                                                } else if !storable {
                                                    note(
                                                        ui,
                                                        t.tr("tags-not-stored"),
                                                        theme::NEUTRAL_500,
                                                    );
                                                }
                                            });
                                            ui.end_row();
                                        }
                                    });
                            });
                        // The boxes were edited above: bring the sheet and
                        // what depends on it (the marks, Save) up to date.
                        if form.sync() {
                            ctx.request_repaint();
                        }
                        can_save = !saving
                            && block.is_none()
                            && !form.changed().is_empty()
                            && form.invalid().is_empty();
                        let addable = form.original.addable_fields(added);
                        ui.add_enabled_ui(!addable.is_empty(), |ui| {
                            ui.menu_button(t.tr("tags-add-field"), |ui| {
                                for field in &addable {
                                    if ui.button(t.tr(&field_key(*field))).clicked() {
                                        added.insert(*field);
                                        ui.close();
                                    }
                                }
                            });
                        });
                        let (kept, more) =
                            (form.original.other_kept, form.original.other_kept_more);
                        let key = match (kept, more) {
                            (0, false) => None,
                            (0, true) => Some("tags-others-kept-uncounted"),
                            (_, false) => Some("tags-others-kept"),
                            (_, true) => Some("tags-others-kept-more"),
                        };
                        if let Some(key) = key {
                            note(
                                ui,
                                t.tr_args(key, &[("count", kept.into())]),
                                theme::NEUTRAL_400,
                            );
                        }
                    });
                }
            }
            if let Some(reason) = block {
                note(ui, t.tr(block_key(reason)), theme::AMBER);
            }
            if let Some(text) = error.as_deref() {
                note(ui, text.to_owned(), theme::AMBER);
            }
            if saving {
                note(ui, t.tr("tags-saving"), theme::NEUTRAL_300);
            }
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if button(ui, &t.tr("tags-save"), can_save, true) {
                    answer = EditorAnswer::Save;
                }
                if button(ui, &t.tr("tags-cancel"), !saving, false) {
                    answer = EditorAnswer::Cancel;
                }
            });
        });
    // Escape or a click on the backdrop cancels, unless a save is running.
    if modal.should_close() && !saving {
        answer = EditorAnswer::Cancel;
    }
    answer
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sheet() -> TagSheet {
        let mut s = TagSheet::new(TagField::ALL, 2, false);
        s.set_text(TagField::Title, "Song", false);
        s.set_text(TagField::Artist, "One\nTwo", true);
        s.set_pair(TagField::TrackNumber, "3", "12");
        s
    }

    fn ready() -> TagEditor {
        let mut e = TagEditor::reading(TrackId(1));
        e.arrived(Some(sheet()));
        e
    }

    /// Types `text` into the box of `field`, as a frame of `show` would.
    fn type_into(e: &mut TagEditor, field: TagField, text: &str) {
        let Phase::Ready(form) = &mut e.phase else {
            panic!("not ready");
        };
        form.boxes.get_mut(&field).unwrap().0 = text.to_owned();
        form.sync();
    }

    #[test]
    fn the_sheet_is_taken_once_while_reading() {
        let mut e = TagEditor::reading(TrackId(1));
        assert!(e.form().is_none());
        e.arrived(Some(sheet()));
        assert!(e.form().is_some());
        e.arrived(None);
        assert!(e.form().is_some(), "a late answer changes nothing");
        let mut e = TagEditor::reading(TrackId(1));
        e.arrived(None);
        assert!(matches!(e.phase, Phase::Unreadable));
    }

    #[test]
    fn an_untouched_draft_cannot_be_saved() {
        let e = ready();
        assert!(e.changed().is_empty());
        assert!(!e.can_save());
        assert!(
            e.save_job(Path::new("/x.mp3"), &Limits::default())
                .is_none()
        );
    }

    #[test]
    fn the_boxes_become_the_sheet() {
        let mut e = ready();
        type_into(&mut e, TagField::Artist, "One\n\nTwo\nThree");
        type_into(&mut e, TagField::Comment, "a\nb");
        let Phase::Ready(form) = &e.phase else {
            panic!()
        };
        assert_eq!(
            form.edited.values(TagField::Artist),
            ["One", "Two", "Three"]
        );
        assert_eq!(
            form.edited.values(TagField::Comment),
            ["a\nb"],
            "a comment is one value over two lines"
        );
        assert_eq!(e.changed(), [TagField::Artist, TagField::Comment]);
    }

    #[test]
    fn an_invalid_value_blocks_the_save() {
        let mut e = ready();
        type_into(&mut e, TagField::Date, "2019-13");
        assert_eq!(e.invalid(), [TagField::Date]);
        assert!(!e.can_save());
        type_into(&mut e, TagField::Date, "2019-05-14");
        assert!(e.invalid().is_empty());
        assert!(e.can_save());
        e.saving = true;
        assert!(!e.can_save(), "no second save while one runs");
    }

    #[test]
    fn the_save_job_carries_what_was_read_and_what_is_wanted() {
        let mut e = ready();
        type_into(&mut e, TagField::Genre, "Jazz");
        let job = e.save_job(Path::new("/x.mp3"), &Limits::default()).unwrap();
        let TagJob::WriteSheet {
            track,
            path,
            before,
            after,
            ..
        } = job
        else {
            panic!("not a sheet write");
        };
        assert_eq!(track, TrackId(1));
        assert_eq!(path, Path::new("/x.mp3"));
        assert!(!before.has(TagField::Genre));
        assert_eq!(after.values(TagField::Genre), ["Jazz"]);
    }

    #[test]
    fn what_the_file_did_not_keep_is_named() {
        let mut e = ready();
        type_into(&mut e, TagField::Genre, "Jazz");
        type_into(&mut e, TagField::Mood, "Calm");
        let mut saved = sheet();
        saved.set_text(TagField::Genre, "Jazz", true);
        let limits = Limits::default();
        assert_eq!(e.unstored(Some(&saved), &limits), [TagField::Mood]);
        assert!(
            e.unstored(None, &limits).is_empty(),
            "no read-back, no claim"
        );
    }

    #[test]
    fn every_field_and_block_has_a_key() {
        assert_eq!(field_key(TagField::AlbumArtist), "tag-field-album-artist");
        assert_eq!(block_key(TagEditBlock::OnAir), "menu-edit-tags-on-air");
    }
}
```

`app.rs`. Borrowing: `scene` borrows `self.ctl`, `self.i18n` and others for the rest of the frame, so everything that needs `&mut self` runs before it is built (`tag_outcomes`, `open_tag_editor`), and the save runs through the free function `start_tag_job` on the disjoint field `self.tag_worker`.

```diff
--- a/crates/fp-app/src/ui/app.rs
+++ b/crates/fp-app/src/ui/app.rs
@@ -29,10 +29,12 @@
 use super::player;
 use super::playlist_files::{self, FileOutcome};
 use super::settings::{self, SettingsDeps, SettingsState};
+use super::tag_editor;
 use super::theme;
 use super::widgets::{self, TileStyle, font, font_medium};
 use crate::i18n::I18n;
 use crate::services::{MediaCache, ServiceRequest};
+use crate::tags::{TagJob, TagOutcome, TagWorker};
 
 const MIN_COLUMN_WIDTH: f32 = 380.0;
 const TOP_BAR_HEIGHT: f32 = 34.0;
@@ -101,6 +103,11 @@
     pub marker_drag: Option<(PlayerId, fp_model::MarkerKind, TrackId)>,
     /// A cart to open in Settings → Cartwall (`Edit…` on a cart).
     pub edit_cart: Option<(fp_model::CartPageId, usize)>,
+    /// A track whose tags the operator asked to edit (feedback 2 spec O23);
+    /// the next frame opens the editor.
+    pub edit_tags: Option<TrackId>,
+    /// The tag editor, while it is open.
+    pub(crate) tag_editor: Option<tag_editor::TagEditor>,
     notice: Option<(String, f64)>,
 }
 
@@ -200,6 +207,9 @@
     /// Set once a restart is confirmed; `main` reads it after the window
     /// closes.
     restart: Arc<AtomicBool>,
+    /// Reads and writes tags off the interface thread; started by the first
+    /// use of the tag editor.
+    tag_worker: Option<TagWorker>,
 }
 
 impl AppUi {
@@ -238,6 +248,7 @@
             remote_status: None,
             started,
             restart: Arc::new(AtomicBool::new(false)),
+            tag_worker: None,
         }
     }
 
@@ -423,6 +434,10 @@
                 });
             }
         }
+        self.tag_outcomes(&state, time);
+        if let Some(track) = self.view.edit_tags.take() {
+            self.open_tag_editor(&ctx, &state, track);
+        }
         self.keyboard(&ctx, &state);
         let pending = fp_model::restart_pending(&self.started, &state.config);
         let scene = Scene {
@@ -557,6 +572,38 @@
                 take_drops = true;
             }
         }
+        // O23: the tag editor, under the dialogs that must stay above it.
+        if !self.view.settings_open
+            && let Some(editor) = &mut self.view.tag_editor
+        {
+            if state.library.get(editor.track).is_none() {
+                self.view.tag_editor = None;
+            } else {
+                let block = super::view::tag_edit_availability(&state, editor.track);
+                match tag_editor::show(&ctx, &scene, editor, block) {
+                    tag_editor::EditorAnswer::Cancel => self.view.tag_editor = None,
+                    tag_editor::EditorAnswer::Save => {
+                        let job = state
+                            .library
+                            .get(editor.track)
+                            .and_then(|track| editor.save_job(&track.path, &state.config.limits));
+                        // The rule is judged again now: the track may have
+                        // gone on air since the modal last drew.
+                        let started = block.is_none()
+                            && job
+                                .is_some_and(|job| start_tag_job(&mut self.tag_worker, &ctx, job));
+                        if started {
+                            editor.saving = true;
+                            editor.error = None;
+                        } else if block.is_none() {
+                            tracing::error!("the tag worker is not running");
+                            editor.error = Some(scene.i18n.tr("tags-error-worker"));
+                        }
+                    }
+                    tag_editor::EditorAnswer::Open => {}
+                }
+            }
+        }
         // O4: Restart now asks the close guard first when audio is on air.
         if std::mem::take(&mut self.view.restart_requested) {
             if fp_model::on_air(&state).is_empty() {
@@ -608,6 +655,112 @@
         }
     }
 
+    /// O23: opens the tag editor on `track` and asks the tag worker for its
+    /// sheet.
+    fn open_tag_editor(&mut self, ctx: &egui::Context, state: &AppState, track: TrackId) {
+        let Some(t) = state.library.get(track) else {
+            return;
+        };
+        let job = TagJob::ReadSheet {
+            track,
+            path: t.path.clone(),
+            limits: state.config.limits.clone(),
+        };
+        if start_tag_job(&mut self.tag_worker, ctx, job) {
+            self.view.tag_editor = Some(tag_editor::TagEditor::reading(track));
+        } else {
+            tracing::error!("the tag worker is not running");
+            self.view.notice = Some((
+                self.i18n.tr("tags-error-worker"),
+                ctx.input(|i| i.time) + NOTICE_SECS,
+            ));
+        }
+    }
+
+    /// O23: takes what the tag worker finished: a sheet for the editor, or
+    /// the result of a save.
+    fn tag_outcomes(&mut self, state: &AppState, time: f64) {
+        let outcomes: Vec<TagOutcome> = self
+            .tag_worker
+            .as_ref()
+            .map(|w| w.results().try_iter().collect())
+            .unwrap_or_default();
+        for outcome in outcomes {
+            match outcome {
+                TagOutcome::SheetRead { track, sheet } => {
+                    if let Some(editor) = &mut self.view.tag_editor
+                        && editor.track == track
+                    {
+                        editor.arrived(sheet.map(|s| *s));
+                    }
+                }
+                TagOutcome::SheetWritten { track, result } => {
+                    self.tag_saved(state, time, track, result);
+                }
+                // The services thread's reads and the summary writes have
+                // their own consumers.
+                TagOutcome::Read { .. } | TagOutcome::Written { .. } => {}
+            }
+        }
+    }
+
+    fn tag_saved(
+        &mut self,
+        state: &AppState,
+        time: f64,
+        track: TrackId,
+        result: Result<Box<crate::tags::SheetSaved>, fp_analysis::tags::TagWriteError>,
+    ) {
+        let title = state
+            .library
+            .get(track)
+            .map(|t| t.title.clone())
+            .unwrap_or_default();
+        let editor = self.view.tag_editor.as_mut().filter(|e| e.track == track);
+        match result {
+            Ok(saved) => {
+                let unstored = editor
+                    .as_ref()
+                    .map(|e| e.unstored(saved.sheet.as_ref(), &state.config.limits))
+                    .unwrap_or_default();
+                let text = if unstored.is_empty() {
+                    self.i18n.tr_args("tags-saved", &[("title", title.into())])
+                } else {
+                    let names: Vec<String> = unstored
+                        .iter()
+                        .map(|f| self.i18n.tr(&tag_editor::field_key(*f)))
+                        .collect();
+                    self.i18n.tr_args(
+                        "tags-saved-partly",
+                        &[("title", title.into()), ("fields", names.join(", ").into())],
+                    )
+                };
+                self.ctl.send(Command::ApplyTags {
+                    track,
+                    tags: Box::new(saved.tags),
+                });
+                self.view.notice = Some((text, time + NOTICE_SECS));
+                if editor.is_some() {
+                    self.view.tag_editor = None;
+                }
+            }
+            Err(e) => {
+                let text = self.i18n.tr_args(
+                    "tags-save-failed",
+                    &[
+                        ("title", title.into()),
+                        ("error", tag_error_text(&self.i18n, &e).into()),
+                    ],
+                );
+                if let Some(editor) = editor {
+                    editor.saving = false;
+                    editor.error = Some(text.clone());
+                }
+                self.view.notice = Some((text, time + NOTICE_SECS));
+            }
+        }
+    }
+
     /// O6: a close request while something is on air waits for the
     /// operator. Runs once per frame from `eframe::App::logic`, which
     /// eframe also calls while the window is minimized or hidden (when
@@ -653,6 +806,10 @@
         if ctx.text_edit_focused() {
             return;
         }
+        // The tag editor is modal: no shortcut, not even Delete, acts under it.
+        if self.view.tag_editor.is_some() {
+            return;
+        }
         // Configured shortcuts whose key the toolkit knows.
         let bindings: Vec<(Key, &KeyChord, ShortcutAction)> = state
             .config
@@ -933,6 +1090,32 @@
     }
 }
 
+/// Queues `job` on the tag worker, starting the worker first if it is not
+/// running yet. `false` if there is no worker to give it to.
+fn start_tag_job(worker: &mut Option<TagWorker>, ctx: &egui::Context, job: TagJob) -> bool {
+    if worker.is_none() {
+        let repaint = ctx.clone();
+        *worker = TagWorker::spawn(Box::new(move || repaint.request_repaint())).ok();
+    }
+    worker.as_ref().is_some_and(|w| w.submit(job))
+}
+
+/// Why a save failed, in the interface language.
+fn tag_error_text(i18n: &I18n, error: &fp_analysis::tags::TagWriteError) -> String {
+    use fp_analysis::tags::TagWriteError as E;
+    match error {
+        E::Unsupported => i18n.tr("tags-error-unsupported"),
+        E::NotFound => i18n.tr("tags-error-not-found"),
+        E::Denied => i18n.tr("tags-error-denied"),
+        E::InvalidDate => i18n.tr("tags-error-invalid-date"),
+        E::InvalidField(field) => i18n.tr_args(
+            "tags-error-invalid-field",
+            &[("field", i18n.tr(&tag_editor::field_key(*field)).into())],
+        ),
+        E::Other(detail) => i18n.tr_args("tags-error-other", &[("detail", detail.clone().into())]),
+    }
+}
+
 pub(crate) fn error_text(i18n: &I18n, error: &ModelError) -> String {
     match error {
         ModelError::LastPlaylist => i18n.tr("error-last-playlist"),
```

Locale additions. Append to `crates/fp-app/locales/en-US/main.ftl`:

```

menu-edit-tags = Edit tags…
menu-edit-tags-file = The file is missing or cannot be read
menu-edit-tags-format = This format has no writable tags
menu-edit-tags-unread = The tags have not been read yet
menu-edit-tags-on-air = A track that is on air cannot be edited
menu-edit-tags-cued = A track that is on CUE cannot be edited
menu-edit-tags-cart = A track on a playing cart cannot be edited
tags-editor-title = Edit tags
tags-reading = Reading tags…
tags-unreadable = The tags of this file cannot be read, so they cannot be edited.
tags-add-field = Add field
tags-total = Total
tags-not-stored = This format cannot store this field.
tags-date-invalid = Use YYYY, YYYY-MM or YYYY-MM-DD (optionally with a time).
tags-number-invalid = Use whole numbers; a total needs a number.
tags-others-kept = { $count ->
    [one] 1 other tag is kept as it is.
   *[other] { $count } other tags are kept as they are.
}
tags-others-kept-more = { $count ->
    [one] 1 other tag is kept as it is, and more that are not counted.
   *[other] { $count } other tags are kept as they are, and more that are not counted.
}
tags-others-kept-uncounted = Other tags are kept as they are.
tags-save = Save
tags-cancel = Cancel
tags-saving = Saving…
tags-saved = Tags saved: { $title }
tags-saved-partly = Tags saved: { $title }. The file did not keep: { $fields }
tags-save-failed = The tags of “{ $title }” were not saved: { $error }
tags-error-unsupported = this format has no writable tags
tags-error-not-found = the file was not found
tags-error-denied = no permission to write the file or its folder
tags-error-invalid-date = the date is not a valid ISO 8601 date
tags-error-invalid-field = “{ $field }” is not valid
tags-error-other = { $detail }
tags-error-worker = The tag editor is not available.
tag-field-title = Title
tag-field-artist = Artist
tag-field-album = Album
tag-field-album-artist = Album artist
tag-field-date = Date
tag-field-track-number = Track number
tag-field-disc-number = Disc number
tag-field-genre = Genre
tag-field-composer = Composer
tag-field-comment = Comment
tag-field-subtitle = Subtitle
tag-field-grouping = Grouping
tag-field-bpm = BPM
tag-field-initial-key = Initial key
tag-field-mood = Mood
tag-field-isrc = ISRC
tag-field-publisher = Publisher
tag-field-catalog-number = Catalog number
tag-field-copyright = Copyright
tag-field-original-artist = Original artist
tag-field-original-album = Original album
tag-field-original-release-date = Original release date
tag-field-lyricist = Lyricist
tag-field-conductor = Conductor
tag-field-remixer = Remixer
tag-field-arranger = Arranger
tag-field-performer = Performer
tag-field-language = Language
tag-field-encoded-by = Encoded by
tag-field-lyrics = Lyrics
tag-field-sort-title = Sort title
tag-field-sort-artist = Sort artist
tag-field-sort-album = Sort album
tag-field-sort-album-artist = Sort album artist
tag-field-sort-composer = Sort composer
tag-field-artist-website = Artist website
```

and to `crates/fp-app/locales/es-ES/main.ftl`:

```

menu-edit-tags = Editar etiquetas…
menu-edit-tags-file = El archivo no existe o no se puede leer
menu-edit-tags-format = Este formato no admite escribir etiquetas
menu-edit-tags-unread = Aún no se han leído las etiquetas
menu-edit-tags-on-air = No se pueden editar las etiquetas de una pista que está sonando
menu-edit-tags-cued = No se pueden editar las etiquetas de una pista en CUE
menu-edit-tags-cart = No se pueden editar las etiquetas de una pista en un cart que suena
tags-editor-title = Editar etiquetas
tags-reading = Leyendo etiquetas…
tags-unreadable = No se pueden leer las etiquetas de este archivo, así que no se pueden editar.
tags-add-field = Añadir campo
tags-total = Total
tags-not-stored = Este formato no puede guardar este campo.
tags-date-invalid = Usa AAAA, AAAA-MM o AAAA-MM-DD (opcionalmente con hora).
tags-number-invalid = Usa números enteros; un total necesita un número.
tags-others-kept = { $count ->
    [one] Se conserva 1 etiqueta más tal como está.
   *[other] Se conservan { $count } etiquetas más tal como están.
}
tags-others-kept-more = { $count ->
    [one] Se conserva 1 etiqueta más tal como está, y otras que no se cuentan.
   *[other] Se conservan { $count } etiquetas más tal como están, y otras que no se cuentan.
}
tags-others-kept-uncounted = Se conservan otras etiquetas tal como están.
tags-save = Guardar
tags-cancel = Cancelar
tags-saving = Guardando…
tags-saved = Etiquetas guardadas: { $title }
tags-saved-partly = Etiquetas guardadas: { $title }. El archivo no conservó: { $fields }
tags-save-failed = No se guardaron las etiquetas de «{ $title }»: { $error }
tags-error-unsupported = este formato no admite escribir etiquetas
tags-error-not-found = no se encontró el archivo
tags-error-denied = sin permiso para escribir el archivo o su carpeta
tags-error-invalid-date = la fecha no es una fecha ISO 8601 válida
tags-error-invalid-field = «{ $field }» no es válido
tags-error-other = { $detail }
tags-error-worker = El editor de etiquetas no está disponible.
tag-field-title = Título
tag-field-artist = Artista
tag-field-album = Álbum
tag-field-album-artist = Artista del álbum
tag-field-date = Fecha
tag-field-track-number = Número de pista
tag-field-disc-number = Número de disco
tag-field-genre = Género
tag-field-composer = Compositor
tag-field-comment = Comentario
tag-field-subtitle = Subtítulo
tag-field-grouping = Agrupación
tag-field-bpm = BPM
tag-field-initial-key = Tonalidad inicial
tag-field-mood = Estado de ánimo
tag-field-isrc = ISRC
tag-field-publisher = Editorial
tag-field-catalog-number = Número de catálogo
tag-field-copyright = Copyright
tag-field-original-artist = Artista original
tag-field-original-album = Álbum original
tag-field-original-release-date = Fecha de lanzamiento original
tag-field-lyricist = Letrista
tag-field-conductor = Director
tag-field-remixer = Remezclador
tag-field-arranger = Arreglista
tag-field-performer = Intérprete
tag-field-language = Idioma
tag-field-encoded-by = Codificado por
tag-field-lyrics = Letra
tag-field-sort-title = Ordenar por título
tag-field-sort-artist = Ordenar por artista
tag-field-sort-album = Ordenar por álbum
tag-field-sort-album-artist = Ordenar por artista del álbum
tag-field-sort-composer = Ordenar por compositor
tag-field-artist-website = Web del artista
```

The test strings ("Save", "Cancel", the reasons, "Edit tags", "Reading tags…", "were not saved", "Use YYYY, YYYY-MM or YYYY-MM-DD", "Use whole numbers", "cannot store this field") must match these messages; keep them in step if a wording changes.

- [ ] **Step 4: Run the tests to verify they pass**

Run each test of `track_tags_ui` by name, then `cargo test -p fp-app --test track_tags_ui` three times (the modal tests wait on a real thread), `cargo test -p fp-app --lib tag_editor`, `cargo test -p fp-app --test view`, `cargo test -p fp-app --test i18n`, `cargo test -p fp-app --test main_screen` (the context-menu tests there must still pass: the new item sits among the existing ones) and `cargo test -p fp-app --test glyphs`.
Expected: PASS.

- [ ] **Step 5: Look at it**

Following CLAUDE.md "Testing notes" (Xvfb, a scratch `FAUSTE_HOME`, `examples/demo_session` with a music folder, the remote API or `xdotool` to right-click a row), open "Edit tags…" on an MP3, a FLAC and a WAV with only RIFF INFO, and check: 10 fields on a bare file, the BPM and a second artist shown when present, the Add field menu, the amber marks on a bad date, the "N other tags" line, and that nothing is cut off at the default 1600×940 window and at 1280×720. Fix layout problems found here in `tag_editor.rs` (sizes are constants there), not in the tests.

- [ ] **Step 6: Commit**

```bash
if cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings \
   && cargo test --workspace; then
  git add crates/fp-app
  git commit -m "feat(ui): edit a track's tags from the row menu, saved off the UI thread"
fi
```

---

### Task 7: Documentation

**Files:**
- Modify: `docs/user/playlists.md` (the row table, the Mouse menu table), `docs/user/` page for the config file if it lists the `limits` keys
- Modify: `docs/technical/analysis.md` (Tags section, "How the app uses it")
- Modify: `docs/technical/persistence.md` (the `limits` table, the library fields)
- Modify: `docs/technical/ui.md` (module table, table section)
- Modify: `docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md` (status and "As built" under §8)
- Modify: `docs/superpowers/plans/2026-10-01-operator-feedback-2-roadmap.md` (row 7 status `done`)
- Modify: `README.md` (the features list)
- `CLAUDE.md`: only if a command or layout line changed (none does: the new files sit in existing crates; leave it).

**Interfaces:** none (prose). Every claim must match the code as built: re-read `track.rs` (`TrackTags`, `apply_tags`), `tag_sheet.rs`, `tag_edit.rs`, `fp-analysis/src/tags.rs`, `fp-app/src/tags.rs`, `services.rs::tag_pass`, `view.rs::track_tooltip` and `tag_editor.rs` before writing.

- [ ] **Step 1: Update the user guide**

`docs/user/playlists.md`:
- Before "## Mouse" add a short **Track tooltip** paragraph: hover a row for a moment to see its title, artist, album, date, genre, length, format (type, sample rate, bit depth when known) and file path; fields the file does not have are left out.
- In the context menu table add, after "Pre-listen on CUE": `| Edit tags… | Open the tag editor for this track. **Save** writes the changes into the audio file; **Cancel** (or Esc) closes without writing. The item is dimmed, with the reason when you hover it, while the track is on air, on CUE or on a playing cart, while its tags have not been read yet, when the file is missing, and for formats whose tags cannot be written (for example DSD) |`.
- Add a section **Editing tags** after the table, in this order:
  1. *What you see.* The editor reads the file when it opens ("Reading tags…"). Always shown: title, artist, album, album artist, date, track number and total, disc number and total, genre, composer, comment. Shown when the file has them: subtitle, grouping, BPM, initial key, mood, ISRC, publisher, catalog number, copyright, original artist, original album, original release date, lyricist, conductor, remixer, arranger, performer, language, encoded by, lyrics, sort title, sort artist, sort album, sort album artist, sort composer, artist website.
  2. *Add field.* The menu below the fields lists the other fields; it offers only what the file's tag format can store (a WAV with RIFF INFO, an AIFF or an old ID3v1 tag store fewer fields than ID3v2, FLAC or MP4). A field the format cannot store is greyed out with a note. Clearing a field removes it from the file; an added field left empty is not written.
  3. *Several values.* Fields that can hold several values (artists, genres, composers, credits, mood, language) show one value per line; Save writes one value per line in the format's own way. Comment and lyrics are free text over several lines.
  4. *Checks.* Date and original release date are ISO 8601 (`2019`, `2019-05` or `2019-05-14`, optionally with a time); track and disc number, their totals and the BPM are whole numbers, and a total needs its number. A field with an invalid value is marked and **Save** stays off. A value the file already had and you did not touch is kept as it is.
  5. *What is kept.* Everything the editor does not show (other standard keys, custom keys, pictures, binary frames) stays in the file; the editor says how many such tags are kept.
  6. *How a save works.* The file is copied next to the original, the copy gets the tags, is synced and replaces the original, so a failure leaves the file as it was; the reason shows in the editor (which stays open to retry) and in the status bar. Only the fields you changed are written. After a save the table shows the new tags at once while markers and the waveform are kept; if the file did not keep a field you changed, the status bar names it.
  7. Tracks of an earlier version get their date, genre and other tags filled in quietly in the background after an update (no full analysis).
- If the user guide lists the `limits` keys of the configuration file, add `limits.max_tag_chars` (2000) and `limits.max_tag_values` (32).

- [ ] **Step 2: Update the technical docs**

`docs/technical/analysis.md`: in "Tags and covers (`metadata.rs`)" add that `Tags` now carries the recording date (ISO 8601 text, as the standards store it: ID3v2.4 `TDRC`, Vorbis `DATE`, MP4 `©day`, APE `Year`), genre, album artist, composer and comment. Add a section **Track tags (`tags.rs`)** with three parts:
- the summary API: `read_track_tags` (file-name fallbacks, cut to `limits.max_tag_chars`), `can_write_tags` (extension only, from lofty's `FileType::tag_support`) and `write_tags` (changed fields only);
- the tag sheet: `TagField` (36 fields in the shown order, 10 always shown), `storable_fields(tag_type)` (the first `ItemKey` of each field's candidate list that `ItemKey::supported_keys` lists), `read_tag_sheet` (the tag `read_tags` reads: primary, else first; items of the field's key with an empty description; a number and total pair, splitting `3/12` written into one key; `other_kept` counts the other items and the pictures, `other_kept_more` says the format also holds frames lofty cannot count), `write_tag_sheet` (clamps to `max_tag_chars` and `max_tag_values`, refuses `invalid_fields`, replaces each changed field with `take_filter` and `push`, one item per value; never touches items it does not own);
- the safe write: `safe_edit` is the one helper (canonical path so a symlink is not replaced, extension check, copy to `name.fptag-<pid>.ext`, parse with covers or fail, `urls_as_text`, edit, `save_to_path`, fsync, rename; the copy is removed on any error), and `TagWriteError` with its mapping. Explain the lofty defect that makes `urls_as_text` necessary (locators are dropped on save) and name the tests that guard it.

In "How the app uses it" describe the tag-only pass: `Track::needs_tag_read`, `Services::tag_pass`, the `fp-tags` worker (`fp-app/src/tags.rs`; its four jobs `Read`, `Write`, `ReadSheet`, `WriteSheet`, panics contained, queued reads skipped at shutdown while writes finish), that every `ApplyAnalysis` resets `tags_read`, and that no analysis version bump or cache change is involved.

`docs/technical/persistence.md`: add `max_tag_chars` (2000; range 64 … 100000) and `max_tag_values` (32; range 1 … 1000) to the `limits` table, and in the library section list the new `Track` fields (`date`, `genre`, `album_artist`, `composer`, `comment`, `tags_read`), all optional on load. The tag sheet is never persisted: the editor reads the file each time.

`docs/technical/ui.md`: add rows for `ui/tag_editor.rs` (the modal, its draft and its phases) and for the worker in `fp-app/src/tags.rs`; in the table section describe the row tooltip (`view::track_tooltip`, `on_hover_ui`, the usual delay) and the menu item (`view::tag_edit_availability`, the reason tooltip, `ViewState::edit_tags` handed to `AppUi::open_tag_editor` on the next frame, the sheet arriving through `TagOutcome::SheetRead`, the save through `start_tag_job`, the outcomes drained at the start of each frame, the re-check at Save, the keyboard gate while the modal is open, why `scene` is built after the `&mut self` work).

- [ ] **Step 3: Update the spec, roadmap and README**

Spec §8 (after the editor bullets), mark the plan done and add:

```markdown
- **As built.**
  - Model: `TrackTags` is the summary the library keeps (read, tooltip, `ApplyTags`); `Track` gains `date: Option<String>` (ISO 8601 text, validated by `parse_tag_date`), `genre`, `album_artist`, `composer`, `comment` and `tags_read`; `fp_model::tag_edit_block(state, track, format_writable)` gives the reason an edit is refused (`FileUnavailable`, `UnsupportedFormat`, `TagsNotRead`, `OnAir`, `Cued`, `OnCart`) and judges the file, not the entry. `limits.max_tag_chars` (2000, 64..=100000) cuts tag text and `limits.max_tag_values` (32, 1..=1000) the values of a field.
  - Tag-only pass: every `ApplyAnalysis` clears `tags_read`; `Services::tag_pass` sends analysed, readable tracks with unread tags to the `fp-tags` worker; no analysis version bump.
  - Tooltip: `view::track_tooltip` (title, artist, album, date, genre, duration, format, path); the codec is the upper-cased extension.
  - Editor: `TagSheet` and `TagField` (36 fields, 10 always shown) with the pure rules `invalid_fields`, `changed_fields`, `unstored_fields`; `fp_analysis::tags::{read_tag_sheet, write_tag_sheet}` read and write them through lofty's `ItemKey`, the write using the same synced-copy helper as `write_tags` and touching only the fields that changed. A multi-value field is one item per value; a number and its total are two items (ID3v2 merges them into `TRCK`/`TPOS`); a total needs a number. Items the sheet does not own, including URL frames and comments with a description, are kept (the shared writer re-adds ID3v2 locators as text because lofty 0.25.4 drops them on save). `other_kept` counts the other items and pictures; frames lofty keeps without mapping them are not counted, and the modal says so. After a save the library takes the re-read summary; on failure the modal stays open and the notice area gives the reason.
```

Roadmap: set row 7's status to `done`. README: add to the features list `- **Tag editor and track tooltip**: hover a track for its tags, format and path; edit the common tag fields (title, artist, album, date, track and disc numbers, genre, BPM, key, lyrics and more) of MP3, FLAC, MP4, WAV and other files, written safely into the file.`

- [ ] **Step 4: Check the docs**

Run: `grep -rn "Edit tags\|max_tag_chars\|max_tag_values\|tags_read\|read_tag_sheet" docs README.md crates/fp-app/locales | head -60` and confirm that every place that should mention them does. Then `scripts/check-commits.sh origin/master` after committing.

- [ ] **Step 5: Commit**

```bash
if cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings \
   && cargo test --workspace; then
  git add docs README.md
  git commit -m "docs: describe the track tooltip and the tag editor"
fi
```

---

## Self-Review

**Spec coverage.**
- Model (`date`, `genre`, `album_artist`, `composer`, `comment`; lenient loading; tag-only pass): Task 1 (fields, `serde(default)`, `an_old_library_entry_loads_with_empty_tags`) and Task 3 (the pass, no full analysis); reading with lofty in Task 2.
- Tooltip (delay, listed fields, missing left out): Task 4, including `the_tooltip_waits_for_the_usual_delay`.
- Editor, one bullet at a time:
  - opens on a helper thread, "Reading tags…" until the sheet arrives, the library keeps only the summary: Task 5 (`ReadSheet`), Task 6 (`the_editor_says_it_is_reading_until_the_sheet_arrives`);
  - the 10 always-shown fields in order and the 26 optional ones, exactly the spec's names and order: Task 5 `there_are_exactly_the_fields_of_the_spec_in_order`, `ten_fields_are_always_shown_and_the_first_ten`; shown when the file has them: Task 6 `the_editor_shows_what_the_file_holds`;
  - Add field lists only what the format can store, and an unstorable field is never editable: Task 5 `add_field_offers_only_what_the_format_can_store_and_is_missing`, `what_a_format_can_store_decides_the_fields`, `add_field_never_offers_what_the_file_cannot_store`, `a_change_to_a_field_the_format_cannot_store_is_invalid`; Task 6 `add_field_offers_what_the_format_can_store_and_shows_the_choice`, `a_format_that_cannot_store_a_field_shows_it_disabled_and_never_offers_it`;
  - clearing removes, an empty added field is not written: Task 5 `a_cleared_field_is_removed_and_an_empty_added_field_is_not_written`; Task 6 `an_added_field_is_saved_and_an_empty_one_is_not`;
  - several values, one per line: Task 5 `riff_info_round_trips_its_fields_with_values_one_per_line`, `every_id3v2_field_round_trips` (two artists as two `TPE1` values); Task 6 `the_editor_shows_what_the_file_holds` (artist box);
  - date, original release date, track and disc number and total, BPM validated, invalid value blocks Save and is marked: Task 5 `the_values_of_a_changed_field_are_validated`, `an_invalid_value_is_refused_and_the_file_is_untouched`; Task 6 `save_needs_a_change_and_valid_values`;
  - everything else kept byte for byte and counted: Task 5 `custom_items_and_pictures_survive_an_edit_untouched` (custom `TXXX`, described comment, URL frame, picture; the picture is compared byte for byte; mapped items are re-encoded by lofty with the same values, which the test checks by value, and unmapped frames stay verbatim). The count of other tags is in the sheet (`other_kept`) and in the modal.
  - Save on a helper thread with copy, write, fsync, rename (Task 2 `safe_edit` steps, Task 3 worker, Task 5 `write_tag_sheet`, Task 6 `start_tag_job`); the library takes the new tags and markers and analysis are kept (Task 6 `saving_writes_the_file_and_the_library_takes_the_tags` asserts `duration_secs` stays); on any error the original is untouched and the notice reports it (Task 5 failure tests, Task 6 `a_failed_save_keeps_the_modal_and_says_why`); the disabled cases with the reason as a tooltip (Task 1 rule, Task 6 `the_menu_item_is_disabled_with_the_reason`).
- Docs, spec "As built", roadmap row 7, README: Task 7. Both locales inside Tasks 4 and 6 (`every_field_has_a_label_in_both_languages`, `tests/i18n.rs`). `Config` fields with default, range and lenient loading: Task 1 (`max_tag_chars`) and Task 5 (`max_tag_values`).

**Placeholder scan.** No TBD. The code in Tasks 5 and 6 was compiled and its tests were run against the tree as of Task 4 before the plan was written, so the diffs and files are the real text.

**Type consistency.** `TrackTags` (Task 1) is still the value of `Command::ApplyTags`, the result of `read_track_tags` (Task 2), the tag-only pass (Task 3) and the `tags` of `SheetSaved` (Task 5, applied in Task 6). `TagSheet` and `TagField` (Task 5, `fp-model`) are what `read_tag_sheet`/`write_tag_sheet` (Task 5, `fp-analysis`), `TagJob::{ReadSheet, WriteSheet}`/`TagOutcome::{SheetRead, SheetWritten}` (Task 5, `fp-app`) and `TagEditor` (Task 6) pass around. `tag_edit_block(state, track, format_writable)` (Task 1) is wrapped by `view::tag_edit_availability(state, track)` (Task 6), which `table.rs` and `app.rs` both call. `TagWriteError` variants match `tag_error_text` (including `InvalidField`). Locale keys named in Task 6 are the ones its tests look for; the 36 `tag-field-<slug>` keys follow `TagField::slug`.

**Review Focus.** Each of the five lines names a test in the task that owns the code.

**Known weak spots to watch in review.**
- The modal sizes itself over a few frames (egui `Modal` plus a `Grid` inside a `ScrollArea`), so UI tests settle it before clicking; if the layout is changed, re-run `track_tags_ui` several times.
- `TagSheet::other_kept` is a lower bound for formats where lofty keeps frames it does not map (ID3v2 `TXXX` with an unknown description, for example); the modal says "and more" then. If the maintainer wants an exact count it needs lofty's per-format tag types instead of the generic `Tag`.
- Mapped items are rebuilt by lofty from the generic `Tag` on every save (values are preserved, frame encoding details such as the text encoding may differ), so "byte for byte" holds for unmapped items and pictures, and "same value" for mapped ones.
- The editor's save path never calls `write_tags` (kept as the summary-level write of Task 2 and still used by its tests and by `TagJob::Write`).
- An ID3v1-only file is edited as ID3v1 (its fields are short and few); the editor shows what the format can store and Add field offers nothing more.
- The Spanish field names are the usual ones from common players; the maintainer may want to adjust them.
- RIFF INFO cannot hold album artist, which is why Task 2's tests write to a fresh WAV (ID3v2 is lofty's primary tag for WAV) and Task 6's RIFF test uses a WAV with an explicit INFO chunk.
