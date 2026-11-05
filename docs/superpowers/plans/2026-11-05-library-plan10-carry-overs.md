# Library carry-overs before 0.13.0 (plan 10)

**Goal:** close the small items plans 7–9 left before the 0.13.0 release.

**Spec:** `docs/superpowers/specs/2026-11-01-track-library-design.md` (LB54
note) and the on-air pre-listen spec §5.

## Tasks

- [x] The system Back cancels the on-air pre-listen question, as Esc does
  (the close guard, drawn over it, closes first). Test
  `cue_on_air_ui::back_cancels_the_question`.
- [x] LB54: with a stopped track and the card closed, the strip shows the
  track's title, dimmed, not the idle hint beside the caret. Test
  `lb54_a_stopped_track_shows_its_title_not_the_hint`; spec and guide.
- [x] Tests for the sideways card (LB55) and a long press on the card's
  waveform (LB56).
- [x] The library's search index is folded when a search first needs it,
  not when the window opens (~29 ms on a large library). Test
  `the_search_index_is_built_when_a_search_needs_it`.
- [x] `touch::tapped`: a second finger's tap that ends past the screen's
  edge is checked at the nearest point on the screen, where egui finds a
  layer (a modal's backdrop included), instead of being dropped.

## Not in this plan (rulings)

- `play-screenshots.sh` taps fixed coordinates: the script fixes the screen
  at 1080 × 2160 (`wm size`) and its images are reviewed in their PR.
- `Snapshot::of` clones the playlists per autosave: entries are small ids;
  a shared structure would change the model for microseconds.
- The not-followed folder's retry walk and the device re-read trusting
  MediaStore: both trade freshness against cost, a choice for the
  maintainer (asked).
