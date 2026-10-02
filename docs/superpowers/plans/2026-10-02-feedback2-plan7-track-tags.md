# Track Tags (Feedback 2, Plan 7) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `Track` carries year, genre, album artist, composer and comment, read from the file's tags in a cheap tag-only pass (O23); hovering a table row shows a tooltip with the track's tags, format and path; the row menu opens an "Edit tags…" modal that writes the tags into the file safely, on a helper thread.

**Architecture:**
- **Model first.** `Track` gains five tag fields and `tags_read`; `TrackTags` is the one value type for what the UI shows, what the editor edits and what the file reader returns. `Command::ApplyTags` stores it. Whether a track may be edited is a pure function, `fp_model::tag_edit_block`. The cap on tag text is a `Config` field, `limits.max_tag_chars`.
- **Tag I/O.** `fp-analysis` gains `tags.rs` on top of lofty (already a dependency, so no new crate): `read_track_tags`, `can_write_tags`, and `write_tags`, which copies the file to a temporary file in the same folder, writes only the changed fields to the copy, fsyncs it and renames it over the original. `fp-app` gains `tags.rs`, a small worker thread (`TagWorker`) that runs those jobs. The services thread uses one worker for the tag-only pass over tracks whose tags were not read yet; the UI uses another for saves.
- **UI.** `view::track_tooltip` is a pure function; the table shows it with `on_hover_ui`. The row context menu gets "Edit tags…", disabled with a tooltip reason from `tag_edit_block`; `ui/tag_editor.rs` draws the modal and `AppUi` runs the save and reports the outcome in the notice area.

**Tech Stack:** Rust, lofty 0.25.4 (existing dependency), egui/eframe 0.36.2, egui_kittest 0.36.2, `hound` (dev-dependency) for generated WAV files. No new dependency, so `cargo deny check` needs no new entry (Task 2 still runs it).

**Spec:** `docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md` §8 (item O23). Roadmap: `docs/superpowers/plans/2026-10-01-operator-feedback-2-roadmap.md` (plan 7, branch `feat/track-tags`). Plan 8 (track table: O7, O9, O16, O22, O24) builds its Year and Genre columns on the fields this plan adds.

## Decisions taken

The spec leaves these open. Each is the most conservative reading.

- **Edit and write only what changed.** The editor starts from the track's current tags. `write_tags(path, before, after, limits)` sets or removes only the fields where `after` differs from `before`. Opening the editor on an untagged file and changing only the artist therefore does not write the file-name title into the file. A field cleared in the editor removes that tag from the file.
- **The file is the truth after a save.** After the write, the worker reads the tags back from the file and `ApplyTags` stores what was read. A format that cannot hold a field (for example album artist in a RIFF INFO block) shows the field empty afterwards instead of pretending it was saved.
- **Which tag is edited.** The primary tag of the format if the file has one, otherwise the first tag it has (the same choice `read_tags` makes when reading), otherwise a new primary tag. Other tags, covers, the `INTRO` marker and unknown items stay as they are, because the same tag object is saved. If the file cannot be parsed with its covers, the save fails; it never retries without them (that would drop the cover).
- **Symlinks.** The write targets the canonical path, so a symlink is not replaced by a regular file.
- **Which tracks are blocked.** `tag_edit_block` refuses, in this order: a track whose file is not `Ok` (missing or unreadable), a format lofty cannot write (decided from the extension only, so the UI thread never touches the disk), a track whose tags are not read yet (`tags_read == false`, so the editor never shows stale fields or races the tag-only pass), a track that is the current entry of any player (playing or paused), a track that is the CUE entry of any player, and a track on a playing cart. It judges the track, not the entry: the same file in another playlist is blocked too, because it is the same file.
- **Re-checked at Save.** The block is evaluated again when Save is pressed and the modal shows the reason instead of saving. A track that goes on air after the write started is safe on Linux and macOS (the rename keeps the open file intact); on Windows the rename fails and the error is reported.
- **The tag-only pass.** Every `ApplyAnalysis` sets `tags_read` to false, so a track just analysed, a track of an older library (the field defaults to false) and a re-analysed track all get one cheap tag read afterwards. No analysis version bump, so the cache and the "analysed by an earlier version" notice are untouched. Failed reads still mark the track read (the reader degrades to the file-name title and empty fields), so nothing loops.
- **Text cap.** Tag text read from files and typed in the editor is trimmed and cut to `limits.max_tag_chars` characters (default 2000, range 64..=100000): a hostile 16 MiB comment must not enter the library file.
- **Year.** `Option<u32>`; the editor accepts empty or a whole number from 1 to 9999.
- **Tooltip content follows the spec** (title, artist, album, year, genre, duration, format, path; a missing field is left out). Album artist, composer and comment are in the editor and the model only. The codec is the upper-cased file extension (the model records no codec), the sample rate is in kHz and the bit depth only when known.
- **Failure keeps the modal open.** On a failed save the modal stays open with the operator's text and the notice area reports the reason; on success it closes.
- **Not exposed remotely.** The remote API, MIDI and the session file are unchanged; the library file simply gains the new fields.

## Global Constraints

- All code, identifiers, comments, docs and commit messages are in English. Never mention other products.
- Spec O23 model: "`Track` gains `year`, `genre`, `album_artist`, `composer` and `comment`, read by `fp-analysis` with lofty. Loading is lenient. Tracks from earlier versions show the new fields empty until their tags are read again. That is a tag-only pass, not a full re-analysis."
- Spec O23 tooltip: "Hovering a table row, after the usual tooltip delay, shows the title, artist, album, year, genre, duration, format (codec, sample rate, bit depth) and path. A missing field is left out."
- Spec O23 editor: "The row context menu gains "Edit tags…". It opens a modal with title, artist, album, album artist, year, genre, composer and comment, for one track. **Save** writes the tags into the file on a helper thread, never on the UI thread: 1. copy the file to a temporary file in the same folder; 2. write the tags to the copy; 3. fsync it; 4. rename it over the original. On success, the library takes the new tags; markers and analysis are kept. On any error, the original file is untouched and the notice area reports it. The menu item is disabled, with the reason as a tooltip, when: the track is current or cued in any player; it is on a playing cart; its format has no writable tags in lofty; the file is missing."
- CLAUDE.md rule 4: anything an operator might change is a `Config` field with a documented default, a range in `Config::validate` and lenient loading (Task 1: `limits.max_tag_chars`).
- CLAUDE.md rule 6: no `unwrap`, `expect` or `panic` outside tests; `fp-analysis` denies `clippy::indexing_slicing` (use `get`).
- CLAUDE.md rules 8 and 9: the UI never blocks (tag reading and writing run on worker threads; the UI only checks the extension) and untrusted tags, covers and files degrade to "not available" with a log line. A panic inside tag code is caught and reported as a failed job.
- Behaviour lives in `fp-model` as pure functions (`apply_tags`, `tag_edit_block`, `needs_tag_read`, `TrackTags::clamped`, `parse_year`); the UI only displays and sends commands.
- Nothing goes on air by itself: this plan starts no audio.
- UI strings are Fluent messages in `crates/fp-app/locales/en-US/main.ftl` (source) and `es-ES/main.ftl`, always both (`tests/i18n.rs::both_locales_define_the_same_keys` checks it).
- Test files are generated in `tempfile` directories (WAV through `hound`, tagged through lofty); no network, no encoders, no real music.
- Commands: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p <crate> --test <file> <name>` (one filter per command). Build only into the repo's `target/`.
- Commit only when fmt, clippy and the whole suite pass (`cargo test --workspace`). Commits end with the co-author trailer the harness provides. `CHANGELOG.md` is never edited by hand.
- Documentation is part of the change (Task 6).

## Review Focus

Failure modes the spec implies and its tests do not name; each has a test in the task named.

1. A save that fails half way (read-only folder or file, the file vanished, a format lofty cannot write, a corrupt file): the original file must be byte-identical and no `*.fptag-*` temporary file may be left behind. (Task 2 `a_failed_write_leaves_the_original_and_no_temporary_file`, `an_unwritable_format_is_refused_without_touching_the_file`.)
2. A file whose tags the editor does not show must not lose them: covers, other tag types, items such as `INTRO`, and fields the operator did not change. (Task 2 `fields_that_did_not_change_are_not_written`, `a_cover_and_other_items_survive_a_write`.)
3. Hostile or huge tag text (a 5 MB comment, control characters, text with only spaces) must neither bloat the library nor crash. (Task 1 `tag_text_is_trimmed_and_cut`; Task 2 `a_huge_comment_is_cut_when_read`.)
4. Tracks that must not be edited or written: on air in a player, cued, on a playing cart, tags not read yet, missing file, unwritable format; and the state changing while the modal is open (the track goes on air, is removed from the library). (Task 1 `tag_edit_block_*`; Task 5 `the_menu_item_is_disabled_with_the_reason`, `saving_is_refused_when_the_track_went_on_air`, `the_modal_closes_when_the_track_is_removed`.)
5. A library from an earlier version (no new fields, `tags_read` false) must load, show empty fields, and be filled by the tag-only pass without a full analysis; a failed read must not loop. (Task 1 `an_old_library_entry_loads_with_empty_tags`; Task 3 `the_tag_pass_fills_the_tags_of_analysed_tracks_once`, `a_track_whose_tags_cannot_be_read_is_not_asked_again`.)

## File Structure

- Modify `crates/fp-model/src/track.rs` (`TrackTags`, `InvalidYear`, `parse_year`, new `Track` fields, `Track::tags`, `apply_tags`, `needs_tag_read`), `config.rs` (`Limits::max_tag_chars`), `command.rs` (`Command::ApplyTags`), `reducer.rs` (the arm), `lib.rs` (exports).
- Create `crates/fp-model/src/tag_edit.rs`: `TagEditBlock`, `tag_edit_block`.
- Create `crates/fp-model/tests/track_tags.rs`.
- Modify `crates/fp-analysis/src/metadata.rs` (`Tags` gains five fields), `lib.rs`; create `crates/fp-analysis/src/tags.rs` (`read_track_tags`, `can_write_tags`, `write_tags`, `TagWriteError`); create `crates/fp-analysis/tests/tags.rs`.
- Create `crates/fp-app/src/tags.rs` (`TagWorker`, `TagJob`, `TagOutcome`); modify `crates/fp-app/src/lib.rs`, `services.rs` (the tag-only pass); modify `crates/fp-app/tests/services.rs`.
- Modify `crates/fp-app/src/ui/view.rs` (`TrackTip`, `track_tooltip`, `tag_edit_availability`), `ui/table.rs` (tooltip, menu item), `ui/app.rs` (state, worker, modal, notice), `ui.rs`.
- Create `crates/fp-app/src/ui/tag_editor.rs`.
- Modify `crates/fp-app/locales/en-US/main.ftl`, `es-ES/main.ftl`.
- Create `crates/fp-app/tests/track_tags_ui.rs`; modify `crates/fp-app/tests/view.rs`.
- Modify docs: `docs/user/playlists.md`, `docs/technical/analysis.md`, `docs/technical/persistence.md`, `docs/technical/ui.md`, the spec (status and "As built" under §8), `docs/superpowers/plans/2026-10-01-operator-feedback-2-roadmap.md` (row 7 done), `README.md` (feature line).

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
- Produces (plan 8 and Tasks 2-5 rely on these exact names):
  - `pub struct TrackTags { pub title: String, pub artist: String, pub album: String, pub album_artist: String, pub year: Option<u32>, pub genre: String, pub composer: String, pub comment: String }` (`Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize`), with `fn clamped(self, max_chars: usize) -> TrackTags`.
  - `pub struct InvalidYear;` and `pub fn parse_year(text: &str) -> Result<Option<u32>, InvalidYear>`.
  - `Track` fields `pub year: Option<u32>`, `pub genre: String`, `pub album_artist: String`, `pub composer: String`, `pub comment: String`, `pub tags_read: bool` (all `#[serde(default)]`); methods `Track::tags(&self) -> TrackTags`, `Track::apply_tags(&mut self, tags: &TrackTags)`, `Track::needs_tag_read(&self) -> bool`.
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
    TrackId, TrackTags, apply, parse_year, tag_edit_block,
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
        year: Some(1999),
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
    assert_eq!(track.year, Some(1999));
}

#[test]
fn apply_tags_can_clear_a_field_but_never_the_title() {
    let (mut s, t) = ready(0);
    let cleared = TrackTags {
        title: String::new(),
        album: String::new(),
        year: None,
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
    assert_eq!(track.year, None);
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
    assert_eq!(track.year, None);
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
fn years_are_whole_numbers_from_1_to_9999() {
    assert_eq!(parse_year(""), Ok(None));
    assert_eq!(parse_year("  1984 "), Ok(Some(1984)));
    for bad in ["0", "10000", "-3", "19x4", "1984.5", "٣٣"] {
        assert!(parse_year(bad).is_err(), "{bad}");
    }
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
Expected: FAIL to compile (`TrackTags`, `tag_edit_block`, `Command::ApplyTags`, `parse_year` not found). `serde_json` is already a dev-dependency of `fp-model` (check `crates/fp-model/Cargo.toml`; add `serde_json.workspace = true` under `[dev-dependencies]` if it is missing).

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
    pub year: Option<u32>,
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
        self
    }
}

/// The year field holds something that is not a year.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidYear;

/// Empty is no year; otherwise a whole number from 1 to 9999.
pub fn parse_year(text: &str) -> Result<Option<u32>, InvalidYear> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(None);
    }
    if !text.chars().all(|c| c.is_ascii_digit()) {
        return Err(InvalidYear);
    }
    match text.parse::<u32>() {
        Ok(year @ 1..=9999) => Ok(Some(year)),
        _ => Err(InvalidYear),
    }
}
```

In `Track` add, after `analysis_version`:

```rust
    /// Tags beyond title, artist and album (feedback 2 spec O23). Libraries
    /// saved earlier have none until the tag-only pass reads them.
    #[serde(default)]
    pub year: Option<u32>,
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

In `Track::new` add `year: None, genre: String::new(), album_artist: String::new(), composer: String::new(), comment: String::new(), tags_read: false,`. In the `impl Track` block add:

```rust
    /// The tags as shown and edited.
    pub fn tags(&self) -> TrackTags {
        TrackTags {
            title: self.title.clone(),
            artist: self.artist.clone(),
            album: self.album.clone(),
            album_artist: self.album_artist.clone(),
            year: self.year,
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
        self.year = tags.year;
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

`lib.rs`: add `mod tag_edit;`, `pub use tag_edit::{TagEditBlock, tag_edit_block};` and extend the `track` re-export with `InvalidYear, TrackTags, parse_year`.

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
  - `Tags` gains `year: Option<u32>`, `genre`, `album_artist`, `composer`, `comment: Option<String>` (all `None` when absent).
  - `fp_analysis::tags::read_track_tags(path: &Path, limits: &Limits) -> TrackTags`: never fails; the title falls back to the file name (and the artist to the file name's `Artist - `), text is cut to `limits.max_tag_chars`.
  - `fp_analysis::tags::can_write_tags(path: &Path) -> bool`: from the extension only, no I/O.
  - `fp_analysis::tags::write_tags(path: &Path, before: &TrackTags, after: &TrackTags, limits: &Limits) -> Result<(), TagWriteError>`.
  - `#[derive(Debug, Clone, PartialEq, Eq)] pub enum TagWriteError { Unsupported, NotFound, Denied, Other(String) }` with `Display`.

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
        year: Some(1999),
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
    assert_eq!((t.year, t.genre.as_str(), t.comment.as_str()), (None, "", ""));
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
        year: None,
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
```

`image` is already a dev-usable dependency of `fp-analysis` (normal dependency); `lofty` too.

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-analysis --test tags`
Expected: FAIL to compile (`fp_analysis::tags` not found).

- [ ] **Step 3: Implement**

`metadata.rs`: add to `Tags`

```rust
    pub year: Option<u32>,
    pub genre: Option<String>,
    pub album_artist: Option<String>,
    pub composer: Option<String>,
    pub comment: Option<String>,
```

and in `read_tags`, where `Tags { title: ..., cover }` is built:

```rust
    Tags {
        title: non_empty(tag.title()),
        artist: non_empty(tag.artist()),
        album: non_empty(tag.album()),
        year: tag.year().filter(|y| (1..=9999).contains(y)),
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
    /// Anything else, with the system's or lofty's own description.
    Other(String),
}

impl std::fmt::Display for TagWriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported => f.write_str("this format has no writable tags"),
            Self::NotFound => f.write_str("the file was not found"),
            Self::Denied => f.write_str("permission denied"),
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
        year: tags.year,
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
    change_tag(tag, before, after);
    file.save_to_path(temp, WriteOptions::default())
        .map_err(lofty_error)?;
    std::fs::OpenOptions::new()
        .write(true)
        .open(temp)?
        .sync_all()?;
    std::fs::rename(temp, real)?;
    Ok(())
}

fn change_tag(tag: &mut Tag, before: &TrackTags, after: &TrackTags) {
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
    if before.year != after.year {
        match after.year {
            Some(year) => tag.set_year(year),
            None => tag.remove_year(),
        }
    }
}
```

Notes for the implementer: (1) `file.primary_tag_mut()` then `file.first_tag_mut()` in the `match` can hit a borrow-checker limit; if it does, compute `let use_primary = file.primary_tag().is_some();` first and branch on it. (2) If `TaggedFileExt` / `AudioFile` imports are reported unused or missing, follow the compiler: the tests in `crates/fp-analysis/tests/metadata.rs` show the working `lofty::prelude::*` calls. (3) `tag.set_year`/`remove_year` are `Accessor` methods; if `remove_*` differ in this lofty version, check `~/.cargo/registry/src/*/lofty-0.25.4/src/tag/accessor.rs`. (4) A panic from `lofty::read_from_path` is caught; the `FileType` import is used by `can_write_tags`.

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
- Produces (Task 5 relies on these):
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

(`hound` is already a dev-dependency of `fp-app`; `lofty` is not, so add `lofty.workspace = true` under `[dev-dependencies]` in `crates/fp-app/Cargo.toml`, and `tempfile.workspace = true` too if it is missing. Task 5's tests need the same two.)

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
  - `pub enum TipField { Title, Artist, Album, Year, Genre, Duration, Format, Path }` (`Debug, Clone, Copy, PartialEq, Eq`).
  - `pub fn track_tooltip(track: &Track) -> Vec<(TipField, String)>`: only the fields the track has, in that order. Duration as `format::clock`, format as `FLAC · 44.1 kHz · 16 bit` (codec from the upper-cased extension, rate in kHz with at most one decimal, bit depth only when known; any part that is unknown is left out; no line when nothing is known), path as `track.path.display()`.
  - Locale keys `tip-field-title`, `tip-field-artist`, `tip-field-album`, `tip-field-year`, `tip-field-genre`, `tip-field-duration`, `tip-field-format`, `tip-field-path`, and `tip-format-rate` (`{ $value } kHz`), `tip-format-bits` (`{ $bits } bit`). Plan 8's Year and Genre column headers can reuse the first labels.

- [ ] **Step 1: Write the failing tests**

In `crates/fp-app/tests/view.rs` add (importing `fp_app::ui::view::{TipField, track_tooltip}` and `fp_model::{AudioFormat, Track}`):

```rust
fn tip_track() -> Track {
    let mut t = Track::new(fp_model::TrackId(1), PathBuf::from("/m/Artist - Song.flac"));
    t.title = "Song".into();
    t.artist = "Artist".into();
    t.album = "Album".into();
    t.year = Some(1999);
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
            TipField::Year,
            TipField::Genre,
            TipField::Duration,
            TipField::Format,
            TipField::Path
        ]
    );
    let value = |f| tip.iter().find(|(g, _)| *g == f).unwrap().1.clone();
    assert_eq!(value(TipField::Year), "1999");
    assert_eq!(value(TipField::Duration), "03:20");
    assert_eq!(value(TipField::Format), "FLAC · 44.1 kHz · 16 bit");
    assert_eq!(value(TipField::Path), "/m/Artist - Song.flac");
}

#[test]
fn a_missing_field_is_left_out() {
    let mut t = tip_track();
    t.album.clear();
    t.year = None;
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
    t.year = Some(1999);
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
    Year,
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
        TipField::Year,
        &track.year.map(|y| y.to_string()).unwrap_or_default(),
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
                    view::TipField::Year => "tip-field-year",
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
tip-field-year = Year
tip-field-genre = Genre
tip-field-duration = Duration
tip-field-format = Format
tip-field-path = Path
```

`es-ES/main.ftl`: `Título`, `Artista`, `Álbum`, `Año`, `Género`, `Duración`, `Formato`, `Ruta`.

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

### Task 5: "Edit tags…", the modal and the save (UI)

**Files:**
- Modify: `crates/fp-app/src/ui/view.rs` (`tag_edit_availability`)
- Create: `crates/fp-app/src/ui/tag_editor.rs`
- Modify: `crates/fp-app/src/ui.rs` (`mod tag_editor;`)
- Modify: `crates/fp-app/src/ui/table.rs` (`context_menu` returns the track to edit; the item)
- Modify: `crates/fp-app/src/ui/app.rs` (`ViewState::tag_editor`, the worker, the modal, the outcome)
- Modify: `crates/fp-app/locales/en-US/main.ftl`, `es-ES/main.ftl`
- Test: Modify `crates/fp-app/tests/track_tags_ui.rs`, `crates/fp-app/tests/view.rs`

**Interfaces:**
- Consumes: `fp_model::{tag_edit_block, TagEditBlock, TrackTags, parse_year, Command::ApplyTags}`, `fp_analysis::tags::{can_write_tags, TagWriteError}`, `crate::tags::{TagWorker, TagJob, TagOutcome}`.
- Produces:
  - `view::tag_edit_availability(state: &AppState, track: TrackId) -> Option<TagEditBlock>`: `tag_edit_block(state, track, can_write_tags(&path))`, with the extension check on the track's path (no I/O).
  - `pub(crate) struct TagEditor` (in `tag_editor.rs`) with `TagEditor::open(track: &Track) -> TagEditor` (the draft starts as `track.tags()` and `year_text` as the year), `track: TrackId`, `draft: TrackTags`, `original: TrackTags`, `year_text: String`, `saving: bool`; `fn changed(&self) -> bool`; `pub(crate) fn show(ctx, scene, editor: &mut TagEditor, block: Option<TagEditBlock>) -> EditorAnswer` with `pub(crate) enum EditorAnswer { Open, Cancel, Save }`.
  - `ViewState::tag_editor: Option<TagEditor>`; `AppUi` owns `tag_worker: Option<TagWorker>` (spawned at the first save, with `ctx.request_repaint` as its repaint callback).
  - Locale keys: `menu-edit-tags`, `menu-edit-tags-file`, `-format`, `-unread`, `-on-air`, `-cued`, `-cart` (the disabled reasons), `tags-editor-title`, `tags-field-album-artist`, `tags-field-composer`, `tags-field-comment` (the others reuse the `tip-field-*` labels), `tags-year-invalid`, `tags-save`, `tags-cancel`, `tags-saving`, `tags-saved`, `tags-save-failed`, `tags-error-unsupported`, `tags-error-not-found`, `tags-error-denied`, `tags-error-other`.

- [ ] **Step 1: Write the failing tests**

In `crates/fp-app/tests/view.rs` add tests for `tag_edit_availability` (it needs a state with an analysed, tag-read track; reuse that file's `state` helper and set `analyzed = true`, `tags_read = true` on the tracks):

```rust
#[test]
fn tag_edit_availability_combines_the_rule_and_the_extension() {
    let (mut s, entries, p) = state(2);
    for t in s.library.iter_mut() {
        t.analyzed = true;
        t.tags_read = true;
    }
    let track = |s: &AppState, e: EntryId| s.playlists.entry(e).unwrap().track;
    let t0 = track(&s, entries[0]);
    assert_eq!(tag_edit_availability(&s, t0), None, "a flac at rest");
    s.library.get_mut(t0).unwrap().path = PathBuf::from("/m/a.dsf");
    assert_eq!(
        tag_edit_availability(&s, t0),
        Some(fp_model::TagEditBlock::UnsupportedFormat)
    );
    s.library.get_mut(t0).unwrap().path = PathBuf::from("/m/a.flac");
    apply(&mut s, Command::SetNext(p, entries[0])).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(
        tag_edit_availability(&s, t0),
        Some(fp_model::TagEditBlock::OnAir)
    );
}
```

(import `tag_edit_availability` in the `use fp_app::ui::view::{...}` list.)

In `crates/fp-app/tests/track_tags_ui.rs` add (a `wav` helper like Task 2's, and `open_editor`, which right-clicks "Song 1", clicks "Edit tags…" and runs two frames):

```rust
use egui::accesskit::Role;
use egui_kittest::kittest::Queryable;
use fp_model::{Command, TagEditBlock};

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

fn open_editor(h: &mut egui_kittest::Harness<'static, fp_app::ui::app::AppUi>) {
    h.get_by_label("Song 1").click_secondary();
    h.run_steps(2);
    h.get_by_label("Edit tags…").click();
    h.run_steps(3);
}

fn field(h: &egui_kittest::Harness<'static, fp_app::ui::app::AppUi>, label: &str) -> String {
    h.get_by_role_and_label(Role::TextInput, label)
        .accesskit_node()
        .value()
        .unwrap_or_default()
        .to_owned()
}

#[test]
fn the_menu_item_opens_an_editor_with_the_current_tags() {
    let dir = tempfile::tempdir().unwrap();
    let (mut h, _fake) = harness(tagged_state(Some(&wav(dir.path(), "a.wav"))));
    open_editor(&mut h);
    assert!(h.query_by_label("Edit tags").is_some(), "the modal title");
    assert_eq!(field(&h, "Title"), "Song 1");
    assert_eq!(field(&h, "Artist"), "The Artist");
    assert_eq!(field(&h, "Year"), "1999");
    assert_eq!(field(&h, "Genre"), "Pop");
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
        (TagEditBlock::OnAir, "A track that is on air cannot be edited"),
        (TagEditBlock::Cued, "A track that is on CUE cannot be edited"),
        (TagEditBlock::OnCart, "A track on a playing cart cannot be edited"),
    ];
    for (block, text) in reasons {
        let mut s = tagged_state(None);
        let first = s.library.iter().next().unwrap().id;
        match block {
            TagEditBlock::FileUnavailable => {
                s.library.get_mut(first).unwrap().file_state = FileState::Missing;
            }
            TagEditBlock::UnsupportedFormat => {
                s.library.get_mut(first).unwrap().path = PathBuf::from("/music/Song 1.dsf");
            }
            TagEditBlock::TagsNotRead => s.library.get_mut(first).unwrap().tags_read = false,
            TagEditBlock::OnAir => {
                let p = s.players[0].id;
                let e = s.playlists.get(s.playlists.first_id().unwrap()).unwrap().entries[0].id;
                fp_model::apply(&mut s, Command::SetNext(p, e)).unwrap();
                fp_model::apply(&mut s, Command::Play(p)).unwrap();
            }
            TagEditBlock::Cued => {
                let p = s.players[0].id;
                let e = s.playlists.get(s.playlists.first_id().unwrap()).unwrap().entries[0].id;
                fp_model::apply(&mut s, Command::CueEntry(p, e)).unwrap();
            }
            TagEditBlock::OnCart => {
                // A playing cart whose track is the first track.
                put_on_a_playing_cart(&mut s, first);
            }
        }
        let (mut h, _fake) = harness(s);
        h.get_by_label_contains("Song 1").click_secondary();
        h.run_steps(2);
        let item = h.get_by_label("Edit tags…");
        assert!(item.accesskit_node().is_disabled(), "{block:?}");
        item.hover();
        h.run_steps(40);
        assert!(h.query_by_label_contains(text).is_some(), "{block:?}: {text}");
    }
}

#[test]
fn saving_writes_the_file_and_the_library_takes_the_tags() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "a.wav");
    let (mut h, fake) = harness(tagged_state(Some(&path)));
    open_editor(&mut h);
    type_into(&mut h, "Genre", "Jazz");
    h.get_by_role_and_label(Role::Button, "Save").click();
    wait_for(&mut h, || {
        fake.state.load().library.iter().next().unwrap().genre == "Jazz"
    });
    let file = lofty::read_from_path(&path).unwrap();
    assert_eq!(file.primary_tag().unwrap().genre().as_deref(), Some("Jazz"));
    let track = fake.state.load().library.iter().next().unwrap().clone();
    assert_eq!(track.year, None, "the file has no year: the read-back wins");
    assert!(track.tags_read);
    assert_eq!(track.duration_secs, 200.0, "analysis data is kept");
    assert!(h.query_by_label("Edit tags").is_none(), "the modal closed");
}

#[test]
fn a_failed_save_keeps_the_modal_and_says_why() {
    let (mut h, fake) = harness(tagged_state(Some(Path::new("/nonexistent/folder/a.wav"))));
    open_editor(&mut h);
    type_into(&mut h, "Genre", "Jazz");
    h.get_by_role_and_label(Role::Button, "Save").click();
    wait_for_text(&mut h, "were not saved");
    assert!(h.query_by_label("Edit tags").is_some(), "the modal stays");
    assert_eq!(field(&h, "Genre"), "Jazz", "the text is kept");
    assert_eq!(
        fake.state.load().library.iter().next().unwrap().genre,
        "Pop",
        "the library is unchanged"
    );
}

#[test]
fn save_needs_a_change_and_a_valid_year() {
    let dir = tempfile::tempdir().unwrap();
    let (mut h, _fake) = harness(tagged_state(Some(&wav(dir.path(), "a.wav"))));
    open_editor(&mut h);
    assert!(
        h.get_by_role_and_label(Role::Button, "Save")
            .accesskit_node()
            .is_disabled(),
        "nothing changed"
    );
    type_into(&mut h, "Year", "19x4");
    assert!(
        h.get_by_role_and_label(Role::Button, "Save")
            .accesskit_node()
            .is_disabled()
    );
    assert!(h.query_by_label_contains("whole number from 1 to 9999").is_some());
}

#[test]
fn cancel_closes_the_editor_and_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "a.wav");
    let before = std::fs::read(&path).unwrap();
    let (mut h, fake) = harness(tagged_state(Some(&path)));
    open_editor(&mut h);
    type_into(&mut h, "Genre", "Jazz");
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
    let path = wav(dir.path(), "a.wav");
    let before = std::fs::read(&path).unwrap();
    let (mut h, fake) = harness(tagged_state(Some(&path)));
    open_editor(&mut h);
    type_into(&mut h, "Genre", "Jazz");
    // The operator (or a remote client) starts the track meanwhile.
    let p = fake.player(0);
    let e = fake.entries()[0];
    fake.send(Command::SetNext(p, e));
    fake.send(Command::Play(p));
    h.run_steps(3);
    assert!(
        h.get_by_role_and_label(Role::Button, "Save")
            .accesskit_node()
            .is_disabled(),
        "Save is off while the track is on air"
    );
    assert!(h.query_by_label_contains("A track that is on air cannot be edited").is_some());
    assert_eq!(std::fs::read(&path).unwrap(), before);
}

#[test]
fn the_modal_closes_when_the_track_is_removed() {
    let dir = tempfile::tempdir().unwrap();
    let (mut h, fake) = harness(tagged_state(Some(&wav(dir.path(), "a.wav"))));
    open_editor(&mut h);
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
```

Write the three helpers the sketches use, in the same file, fully: `type_into(h, label, text)` (focus the text input with that label, select all with `Modifiers::COMMAND + A`, type the text one character per frame, as `remote_settings.rs::type_into` does), `wait_for(h, condition)` (up to 200 iterations of `h.run_steps(1)` plus a 10 ms sleep, panicking with a message if the condition never holds) and `wait_for_text(h, text)` (the same, until `h.query_by_label_contains(text).is_some()`), and `put_on_a_playing_cart(state, track)` (assign a cart file, point the cart's `track` at `track` through `state.cartwall.pages[0].carts[0].track = Some(track)` and `state.cartwall.playing.push(fp_model::PlayingCart { cart, looped: false })`, as the cartwall tests do).

- [ ] **Step 2: Run the tests to verify they fail**

Run each: `cargo test -p fp-app --test track_tags_ui the_menu_item_opens_an_editor_with_the_current_tags` and `cargo test -p fp-app --test view tag_edit_availability_combines_the_rule_and_the_extension`
Expected: FAIL (no item / not found).

- [ ] **Step 3: Implement**

`view.rs`:

```rust
/// Why the tags of `track` cannot be edited now, if they cannot: the model's
/// rule plus the format's writability, judged from the path's extension (no
/// I/O on the interface thread).
pub fn tag_edit_availability(
    state: &AppState,
    track: fp_model::TrackId,
) -> Option<fp_model::TagEditBlock> {
    let writable = state
        .library
        .get(track)
        .is_some_and(|t| fp_analysis::tags::can_write_tags(&t.path));
    fp_model::tag_edit_block(state, track, writable)
}
```

`tag_editor.rs`: a modal in the style of `notice.rs` (`egui::Modal`, `theme::SURFACE`, `widgets::tile`-based buttons). Its parts:

```rust
//! The tag editor (feedback 2 spec O23): a modal over one track's tags.
//! It only edits a draft; saving is the application's job (`AppUi`).

use egui::{RichText, vec2};
use fp_model::{TagEditBlock, Track, TrackId, TrackTags, parse_year};

use super::app::Scene;
use super::theme;
use super::widgets::{self, TileStyle, font, font_medium};

/// What the operator did this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EditorAnswer {
    Open,
    Cancel,
    Save,
}

pub(crate) struct TagEditor {
    pub track: TrackId,
    pub original: TrackTags,
    pub draft: TrackTags,
    pub year_text: String,
    /// A save is running on the tag worker.
    pub saving: bool,
}

impl TagEditor {
    pub fn open(track: &Track) -> Self {
        let tags = track.tags();
        Self {
            track: track.id,
            year_text: tags.year.map(|y| y.to_string()).unwrap_or_default(),
            original: tags.clone(),
            draft: tags,
            saving: false,
        }
    }

    /// The draft as the operator typed it: trimmed, the year parsed.
    /// `None` while the year text is not a year.
    pub fn parsed(&self, max_chars: usize) -> Option<TrackTags> {
        let year = parse_year(&self.year_text).ok()?;
        Some(
            TrackTags {
                year,
                ..self.draft.clone()
            }
            .clamped(max_chars),
        )
    }

    /// Something differs from what the file holds and every field is valid.
    pub fn can_save(&self, max_chars: usize) -> bool {
        !self.saving
            && self
                .parsed(max_chars)
                .is_some_and(|tags| tags != self.original)
    }
}
```

`show` draws the modal. The buttons are built like `notice.rs::button`, with an `enabled` flag:

```rust
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

/// A labelled text field; the label names it for accessibility.
fn row(ui: &mut egui::Ui, label: &str, text: &mut String, max_chars: usize, tall: bool) {
    let name = ui.label(RichText::new(label).font(font(12.0)).color(theme::NEUTRAL_400));
    let edit = if tall {
        egui::TextEdit::multiline(text).desired_rows(3)
    } else {
        egui::TextEdit::singleline(text)
    };
    ui.add(edit.char_limit(max_chars).desired_width(320.0))
        .labelled_by(name.id);
    ui.end_row();
}

pub(crate) fn show(
    ctx: &egui::Context,
    scene: &Scene<'_>,
    editor: &mut TagEditor,
    block: Option<TagEditBlock>,
) -> EditorAnswer {
    let t = scene.i18n;
    let max = scene.state.config.limits.max_tag_chars;
    let file_name = scene
        .state
        .library
        .get(editor.track)
        .and_then(|track| track.path.file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let width = (ctx.content_rect().width() - 48.0).clamp(320.0, 520.0);
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
            ui.add_enabled_ui(!editor.saving, |ui| {
                egui::Grid::new("tag-editor-fields")
                    .num_columns(2)
                    .spacing(vec2(10.0, 6.0))
                    .show(ui, |ui| {
                        let d = &mut editor.draft;
                        row(ui, &t.tr("tip-field-title"), &mut d.title, max, false);
                        row(ui, &t.tr("tip-field-artist"), &mut d.artist, max, false);
                        row(ui, &t.tr("tip-field-album"), &mut d.album, max, false);
                        row(ui, &t.tr("tags-field-album-artist"), &mut d.album_artist, max, false);
                        row(ui, &t.tr("tip-field-year"), &mut editor.year_text, 4, false);
                        row(ui, &t.tr("tip-field-genre"), &mut d.genre, max, false);
                        row(ui, &t.tr("tags-field-composer"), &mut d.composer, max, false);
                        row(ui, &t.tr("tags-field-comment"), &mut d.comment, max, true);
                    });
            });
            let warn = |ui: &mut egui::Ui, text: String| {
                ui.add(
                    egui::Label::new(RichText::new(text).font(font(12.0)).color(theme::AMBER))
                        .wrap()
                        .selectable(false),
                );
            };
            if parse_year(&editor.year_text).is_err() {
                warn(ui, t.tr("tags-year-invalid"));
            }
            if let Some(reason) = block {
                warn(ui, t.tr(block_key(reason)));
            }
            if editor.saving {
                ui.add(
                    egui::Label::new(
                        RichText::new(t.tr("tags-saving"))
                            .font(font(12.0))
                            .color(theme::NEUTRAL_300),
                    )
                    .selectable(false),
                );
            }
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let can_save = editor.can_save(max) && block.is_none();
                if button(ui, &t.tr("tags-save"), can_save, true) {
                    answer = EditorAnswer::Save;
                }
                if button(ui, &t.tr("tags-cancel"), !editor.saving, false) {
                    answer = EditorAnswer::Cancel;
                }
            });
        });
    // Escape or a click on the backdrop cancels, unless a save is running.
    if modal.should_close() && !editor.saving {
        answer = EditorAnswer::Cancel;
    }
    answer
}
```

Add a helper shared with the menu so the reason text lives once:

```rust
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
```

`table.rs`: change `context_menu` to return `Option<TrackId>` and take the entry's track (`entry.track`) as a new `track: fp_model::TrackId` argument. After the "Pre-listen on CUE" item and before the separator that precedes "Add tracks below…", add:

```rust
    let block = view::tag_edit_availability(scene.state, track);
    let edit = labelled(ui, icon::PENCIL_SIMPLE, "menu-edit-tags", block.is_none());
    let edit = match block {
        Some(reason) => edit.on_disabled_hover_text(t.tr(super::tag_editor::block_key(reason))),
        None => edit,
    };
    if edit.clicked() {
        opened = Some(track);
        ui.close();
    }
```

with `let mut opened = None;` at the top of `context_menu` and `opened` returned at the end. At the call site (near line 367) capture it:

```rust
                response.context_menu(|ui| {
                    menu_open = true;
                    clicked = Some(entry.id);
                    if let Some(track) =
                        context_menu(ui, scene, player, playlist, entry.id, entry.track, i, &track.title)
                    {
                        edit_tags = Some(track);
                    }
                });
```

declare `let mut edit_tags: Option<fp_model::TrackId> = None;` next to `let mut cue_follow`, and after the selection block:

```rust
    if let Some(id) = edit_tags
        && let Some(t) = scene.state.library.get(id)
    {
        view_state.tag_editor = Some(super::tag_editor::TagEditor::open(t));
    }
```

(The inner closure variable `track` shadows the library track; rename the inner binding if the compiler complains.)

`app.rs`: add `pub(crate) tag_editor: Option<super::tag_editor::TagEditor>,` to `ViewState`; add to `AppUi` `tag_worker: Option<TagWorker>` (initialised `None`); and in `ui()`:

1. Early in the frame, next to the `files_rx` handling, drain the worker:

```rust
        let outcomes: Vec<TagOutcome> = self
            .tag_worker
            .as_ref()
            .map(|w| w.results().try_iter().collect())
            .unwrap_or_default();
        for outcome in outcomes {
            if let TagOutcome::Written { track, result } = outcome {
                let title = state_title(&self.ctl.model(), track);
                match result {
                    Ok(tags) => {
                        self.ctl.send(Command::ApplyTags {
                            track,
                            tags: Box::new(tags),
                        });
                        self.view.notice = Some((
                            self.i18n.tr_args("tags-saved", &[("title", title.into())]),
                            time + NOTICE_SECS,
                        ));
                        if self.view.tag_editor.as_ref().is_some_and(|e| e.track == track) {
                            self.view.tag_editor = None;
                        }
                    }
                    Err(e) => {
                        let text = self.i18n.tr_args(
                            "tags-save-failed",
                            &[("title", title.into()), ("error", tag_error_text(&self.i18n, &e).into())],
                        );
                        self.view.notice = Some((text, time + NOTICE_SECS));
                        if let Some(editor) = &mut self.view.tag_editor
                            && editor.track == track
                        {
                            editor.saving = false;
                        }
                    }
                }
            }
        }
```

with `state_title(state, track) -> String` returning the track's title or empty and `tag_error_text(i18n, &TagWriteError) -> String` mapping `Unsupported`/`NotFound`/`Denied` to `tags-error-unsupported`/`-not-found`/`-denied` and `Other(detail)` to `tags-error-other` with `{ $detail }`.

2. Where the other dialogs are drawn (after `cue_window::show_all`, inside the branch that draws About and the outdated notice so Settings and the close guard stay above it; draw it when `!self.view.settings_open`, before the exit guard):

```rust
        if let Some(editor) = &mut self.view.tag_editor {
            if state.library.get(editor.track).is_none() {
                self.view.tag_editor = None;
            } else {
                let block = view::tag_edit_availability(&state, editor.track);
                match tag_editor::show(&ctx, &scene, editor, block) {
                    tag_editor::EditorAnswer::Cancel => self.view.tag_editor = None,
                    tag_editor::EditorAnswer::Save => self.start_tag_save(&ctx, &state),
                    tag_editor::EditorAnswer::Open => {}
                }
            }
        }
```

3. `start_tag_save`:

```rust
    /// Hands the draft to the tag worker. Re-checks the rule first: the
    /// track may have gone on air since the modal opened.
    fn start_tag_save(&mut self, ctx: &egui::Context, state: &AppState) {
        let Some(editor) = &mut self.view.tag_editor else {
            return;
        };
        let max = state.config.limits.max_tag_chars;
        let (Some(after), Some(track)) = (editor.parsed(max), state.library.get(editor.track))
        else {
            return;
        };
        if view::tag_edit_availability(state, editor.track).is_some() || !editor.can_save(max) {
            return;
        }
        if self.tag_worker.is_none() {
            let repaint = ctx.clone();
            self.tag_worker =
                TagWorker::spawn(Box::new(move || repaint.request_repaint())).ok();
        }
        let job = TagJob::Write {
            track: editor.track,
            path: track.path.clone(),
            before: Box::new(editor.original.clone()),
            after: Box::new(after),
            limits: state.config.limits.clone(),
        };
        match &self.tag_worker {
            Some(worker) if worker.submit(job) => editor.saving = true,
            _ => {
                tracing::error!("the tag worker is not running");
                self.view.notice = Some((
                    self.i18n.tr("tags-error-worker"),
                    ctx.input(|i| i.time) + NOTICE_SECS,
                ));
            }
        }
    }
```

Add the key `tags-error-worker` to the locale list below. While `editor.saving` is true the frame keeps repainting through the worker's callback; no polling is needed.

Locale additions, `en-US/main.ftl`:

```
menu-edit-tags = Edit tags…
menu-edit-tags-file = The file is missing or cannot be read
menu-edit-tags-format = This format has no writable tags
menu-edit-tags-unread = The tags have not been read yet
menu-edit-tags-on-air = A track that is on air cannot be edited
menu-edit-tags-cued = A track that is on CUE cannot be edited
menu-edit-tags-cart = A track on a playing cart cannot be edited
tags-editor-title = Edit tags
tags-field-album-artist = Album artist
tags-field-composer = Composer
tags-field-comment = Comment
tags-year-invalid = The year must be a whole number from 1 to 9999.
tags-save = Save
tags-cancel = Cancel
tags-saving = Saving…
tags-saved = Tags saved: { $title }
tags-save-failed = The tags of “{ $title }” were not saved: { $error }
tags-error-unsupported = this format has no writable tags
tags-error-not-found = the file was not found
tags-error-denied = no permission to write the file or its folder
tags-error-other = { $detail }
tags-error-worker = The tag editor is not available.
```

and the matching `es-ES/main.ftl` (`menu-edit-tags = Editar etiquetas…`, `menu-edit-tags-file = El archivo no existe o no se puede leer`, `menu-edit-tags-format = Este formato no admite escribir etiquetas`, `menu-edit-tags-unread = Aún no se han leído las etiquetas`, `menu-edit-tags-on-air = No se pueden editar las etiquetas de una pista que está sonando`, `menu-edit-tags-cued = No se pueden editar las etiquetas de una pista en CUE`, `menu-edit-tags-cart = No se pueden editar las etiquetas de una pista en un cart que suena`, `tags-editor-title = Editar etiquetas`, `tags-field-album-artist = Artista del álbum`, `tags-field-composer = Compositor`, `tags-field-comment = Comentario`, `tags-year-invalid = El año debe ser un número entero de 1 a 9999.`, `tags-save = Guardar`, `tags-cancel = Cancelar`, `tags-saving = Guardando…`, `tags-saved = Etiquetas guardadas: { $title }`, `tags-save-failed = No se guardaron las etiquetas de «{ $title }»: { $error }`, `tags-error-unsupported = este formato no admite escribir etiquetas`, `tags-error-not-found = no se encontró el archivo`, `tags-error-denied = sin permiso para escribir el archivo o su carpeta`, `tags-error-other = { $detail }`, `tags-error-worker = El editor de etiquetas no está disponible.`). The modal's text-field labels reuse `tip-field-title`, `-artist`, `-album`, `-year`, `-genre`.

The test strings in the tests above ("Save", "Cancel", the reasons, "Edit tags", "were not saved") must match these messages; keep them in step if a wording changes.

- [ ] **Step 4: Run the tests to verify they pass**

Run each test of `track_tags_ui` by name, `cargo test -p fp-app --test view`, `cargo test -p fp-app --test i18n`, then `cargo test -p fp-app --test main_screen` (the context-menu tests there must still pass: the new item sits among the existing ones) and `cargo test -p fp-app --test glyphs`.
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-app
git commit -m "feat(ui): edit a track's tags from the row menu, saved off the UI thread"
```

---

### Task 6: Documentation

**Files:**
- Modify: `docs/user/playlists.md` (the row table, the Mouse menu table)
- Modify: `docs/technical/analysis.md` (Tags section, "How the app uses it")
- Modify: `docs/technical/persistence.md` (the `limits` table, the library fields)
- Modify: `docs/technical/ui.md` (module table, table section)
- Modify: `docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md` (status and "As built" under §8)
- Modify: `docs/superpowers/plans/2026-10-01-operator-feedback-2-roadmap.md` (row 7 status `done`)
- Modify: `README.md` (the features list)
- `CLAUDE.md`: only if a command or layout line changed (none does: the new files sit in existing crates; leave it).

**Interfaces:** none (prose). Every claim must match the code as built: re-read `track.rs` (`TrackTags`, `apply_tags`), `tag_edit.rs`, `fp-analysis/src/tags.rs`, `fp-app/src/tags.rs`, `services.rs::tag_pass`, `view.rs::track_tooltip` and `tag_editor.rs` before writing.

- [ ] **Step 1: Update the user guide**

`docs/user/playlists.md`:
- Before "## Mouse" add a short **Track tooltip** paragraph: hover a row for a moment to see its title, artist, album, year, genre, length, format (type, sample rate, bit depth when known) and file path; fields the file does not have are left out.
- In the context menu table add, after "Pre-listen on CUE": `| Edit tags… | Open the tag editor: title, artist, album, album artist, year, genre, composer and comment of this track. **Save** writes them into the audio file; **Cancel** (or Esc) closes without writing. The item is dimmed, with the reason when you hover it, while the track is on air, on CUE or on a playing cart, while its tags have not been read yet, when the file is missing, and for formats whose tags cannot be written (for example DSD) |`.
- Add a paragraph **Editing tags** after the table: how a save works (the file is copied next to the original, the copy gets the tags, is synced and replaces the original, so a failure leaves the file as it was and the reason shows in the status bar; the editor stays open to retry), that only the fields you changed are written, that clearing a field removes that tag, that covers and other tags stay, that a format that cannot store a field shows it empty afterwards, and that the new tags show in the table at once while markers and the waveform are kept. Mention that tracks of an earlier version get their year, genre and other tags filled in quietly in the background after an update (no full analysis).

- [ ] **Step 2: Update the technical docs**

`docs/technical/analysis.md`: in "Tags and covers (`metadata.rs`)" add that `Tags` now carries year, genre, album artist, composer and comment; add a section **Track tags (`tags.rs`)** covering `read_track_tags` (file-name fallbacks, cut to `limits.max_tag_chars`), `can_write_tags` (extension only, from lofty's `FileType::tag_support`), and `write_tags` (the four steps, only changed fields, the tag edited is the primary one or else the first, canonical path, covers kept because the file is parsed with them or the write fails, error mapping to `TagWriteError`, the temp file name `name.fptag-<pid>.ext` removed on error). In "How the app uses it" describe the tag-only pass: `Track::needs_tag_read`, `Services::tag_pass`, the `fp-tags` worker (`fp-app/src/tags.rs`), that every `ApplyAnalysis` resets `tags_read`, and that no analysis version bump or cache change is involved.

`docs/technical/persistence.md`: add `max_tag_chars` (2000; range 64 … 100000) to the `limits` table, and in the library section list the new `Track` fields (`year`, `genre`, `album_artist`, `composer`, `comment`, `tags_read`), all optional on load.

`docs/technical/ui.md`: add rows for `ui/tag_editor.rs` (the modal and its draft) and for the worker in `fp-app/src/tags.rs`; in the table section describe the row tooltip (`view::track_tooltip`, `on_hover_ui`, the usual delay) and the menu item (`view::tag_edit_availability`, the reason tooltip, the save through `AppUi::start_tag_save`, the outcome drained at the start of each frame, the re-check at Save).

- [ ] **Step 3: Update the spec, roadmap and README**

Spec §8 (after the editor bullets), mark the plan done and add:

```markdown
- **As built.**
  - Model: `TrackTags` is the one value type for read, edit and show; `Track` gains `year: Option<u32>`, `genre`, `album_artist`, `composer`, `comment` and `tags_read`; `Command::ApplyTags` stores tags (an empty field clears, except the title); `fp_model::tag_edit_block(state, track, format_writable)` gives the reason an edit is refused (`FileUnavailable`, `UnsupportedFormat`, `TagsNotRead`, `OnAir`, `Cued`, `OnCart`) and judges the file, not the entry. `limits.max_tag_chars` (2000, 64..=100000) cuts tag text.
  - Tag-only pass: every `ApplyAnalysis` clears `tags_read`; `Services::tag_pass` sends analysed, readable tracks with unread tags to the `fp-tags` worker; no analysis version bump.
  - Tooltip: `view::track_tooltip` (title, artist, album, year, genre, duration, format, path); the codec is the upper-cased extension.
  - Editor: `fp_analysis::tags::write_tags(path, before, after, limits)` writes only the changed fields into a synced copy that replaces the original; the result is read back from the file and applied. On failure the modal stays open and the notice area gives the reason.
```

Roadmap: set row 7's status to `done`. README: add to the features list `- **Tag editor and track tooltip**: hover a track for its tags, format and path; edit title, artist, album, year, genre and more, written safely into the file.`

- [ ] **Step 4: Check the docs**

Run: `grep -rn "Edit tags\|max_tag_chars\|tags_read" docs README.md crates/fp-app/locales | head -40` and confirm that every place that should mention them does. Then `scripts/check-commits.sh origin/master` after committing.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add docs README.md
git commit -m "docs: describe the track tooltip and the tag editor"
```

---

## Self-Review

**Spec coverage.**
- Model (`year`, `genre`, `album_artist`, `composer`, `comment`; lenient loading; tag-only pass): Task 1 (fields, `serde(default)`, `an_old_library_entry_loads_with_empty_tags`) and Task 3 (the pass, no full analysis); reading with lofty in Task 2.
- Tooltip (delay, listed fields, missing left out): Task 4, including `the_tooltip_waits_for_the_usual_delay`.
- Editor: menu item and eight-field modal (Task 5); Save on a helper thread with copy, write, fsync, rename (Task 2 `write_tags`, Task 3 worker, Task 5 `start_tag_save`); library takes the new tags and markers and analysis are kept (Task 5 `saving_writes_the_file_and_the_library_takes_the_tags` asserts `duration_secs` stays); on any error the original is untouched and the notice reports it (Task 2 failure tests, Task 5 `a_failed_save_keeps_the_modal_and_says_why`); the four disabled cases with the reason as a tooltip (Task 1 rule, Task 5 `the_menu_item_is_disabled_with_the_reason`).
- Docs, spec "As built", roadmap row 7, README: Task 6. Both locales inside Tasks 4 and 5. `Config` field with default, range and lenient loading: Task 1.

**Placeholder scan.** No TBD. Task 5's test helpers `type_into`, `wait_for`, `wait_for_text` and `put_on_a_playing_cart` are specified by the existing helper they mirror (`remote_settings.rs::type_into`, the cartwall tests) and by their exact behaviour; the implementer writes them in `track_tags_ui.rs` before the tests that use them.

**Type consistency.** `TrackTags` (Task 1) is the value of `Command::ApplyTags`, `Tags`-to-`TrackTags` in `read_track_tags` (Task 2), `TagJob`/`TagOutcome` (Task 3) and `TagEditor` (Task 5). `tag_edit_block(state, track, format_writable)` (Task 1) is wrapped by `view::tag_edit_availability(state, track)` (Task 5), which `table.rs` and `app.rs` both call. `write_tags(path, before, after, limits)` has the same shape in Tasks 2, 3 and 5. `TagWriteError` variants match `tag_error_text`. Locale keys named in Task 5 are the ones its tests look for.

**Review Focus.** Each of the five lines names a test in the task that owns the code.

**Known weak spots to watch in review.** Borrow-checker friction in `rewrite` (primary versus first tag) and in `tag_pass` is flagged in the steps with the fallback. RIFF INFO cannot hold album artist, which is why the tests write to a fresh WAV (ID3v2 is lofty's primary tag for WAV) and the decisions list says the read-back wins. The tooltip on the row and the tooltips of the small flag icons overlap in the same row: egui shows the hovered widget's, and `hovering_an_unavailable_row_says_why` guards that.
