# Feedback Plan 5 — Waveform: Drag, Zoom and Whole File Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The waveform seeks on release after a drag (F2), zooms and pans
with the wheel (F17), and shows clearly that the whole file is drawn by
dimming the trimmed head and tail (F20).

**Architecture:**
- A pure `WaveView { start_secs, span_secs }` in a new module
  `ui/wave_view.rs` maps between seconds and pixels. It also zooms around a
  point, pans, follows the playhead, and gives the trimmed regions.
- `wave_columns` gains a start offset, so only the visible range is reduced.
- `widgets::waveform` draws through a `WaveView` and handles drag-to-seek.
- `player.rs` keeps a per-player zoom in `ViewState` and handles:
  - the wheel;
  - following the playhead;
  - the "Full view" button;
  - resetting the zoom on a track change.

  Marker editing uses the same view.
- New config field: `ui.follow_current_grace_secs` (default 10, range
  0–600). Plan 7 reuses it.

**Tech Stack:** Rust 2024, egui 0.36, egui_kittest.

**Spec:** [`docs/superpowers/specs/2026-09-30-operator-feedback-design.md`](../specs/2026-09-30-operator-feedback-design.md) §3.3. Roadmap: plan 5.

## Global Constraints

- The deepest zoom is one analysis bucket per pixel: `peak_bucket_secs ×
  width`. It is derived, not a constant.
- `ui.follow_current_grace_secs` is a `Config` field: default 10, range 0–600
  in `validate`, loaded leniently.
- A plain click still seeks at once. Alt-drag stays marker editing and never
  seeks. Releasing a drag outside the waveform, or pressing Esc, cancels.
- UI strings go in both locales: `wave-full-view` = "Full view" / "Vista
  completa".
- English. No `unwrap` or `expect` outside tests. Gate before every commit.
  Conventional Commits with the trailer. Branch: `feat/waveform-zoom`.

## Review Focus

- **Zoom at the edges of the file.** The view never goes before 0 or past
  the end, and zooming out fully gives the full view (Task 2 tests).
- **Very short or untracked files.** A zero or unknown duration draws the
  empty waveform and never divides by zero (Task 2 tests).
- **A drag that starts with Alt, or turns into a marker drag.** It never
  seeks (Task 4 test).
- **A track change while zoomed or dragging.** The zoom resets and a pending
  drag is dropped (Task 5 test).
- **The wheel over the waveform must not scroll the players row** (Task 5
  test).

---

### Task 1: F20 investigation (done while planning)

A probe analysed a 12.3456 s WAV with 3 s of silence at each end:
- `peaks.len() × bucket_secs` came to 12.35 s, which covers the decoded
  duration of 12.3456 s;
- the span drawn is `max(covered, total)`.

So the whole file is drawn. The "late start" is the silent head drawing
nothing. Ledger it as a ruling. There is no code change; Task 3 dims the
trimmed regions.

### Task 2: `WaveView`

**Files:** Create `crates/fp-app/src/ui/wave_view.rs` (`pub mod wave_view;` in `ui.rs`). Modify `widgets::wave_columns` (new `wave_columns_in(peaks, bucket_secs, start_secs, span_secs, columns)`; `wave_columns` calls it with 0). Tests: `crates/fp-app/tests/waveform_view.rs`.

**Interfaces:**

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaveView { pub start_secs: f64, pub span_secs: f64 }
impl WaveView {
    pub fn full(total: f64) -> Self;
    pub fn is_full(&self, total: f64) -> bool;
    pub fn x_of(&self, secs: f64, rect: Rect) -> f32;
    pub fn secs_at(&self, x: f32, rect: Rect) -> f64;          // clamped to the view
    pub fn zoom_at(&self, x: f32, rect: Rect, factor: f64, total: f64, min_span: f64) -> Self;
    pub fn pan(&self, dx: f32, rect: Rect, total: f64) -> Self;
    pub fn follow(&self, position: f64, total: f64) -> Self;   // keeps it, or puts it 10 % in
    pub fn trimmed(&self, rect: Rect, cue_in: Option<f64>, cue_out: Option<f64>, total: f64) -> [Option<Rect>; 2];
}
pub fn min_span(bucket_secs: f64, width: f32) -> f64;
```

Tests: zoom keeps the time under the pointer; clamped to `[0, total]`; min
span is one bucket per pixel; zooming out past total gives `full`; pan clamps;
follow leaves a visible position alone and brings an outside one in; `total`
0 or NaN gives a harmless full view; `wave_columns_in` over a sub-range equals
the matching slice of a full reduction at the same scale; trimmed rects.

### Task 3: Trimmed regions

`waveform` shades `trimmed(...)` with `NEUTRAL_950`-like dimming
(`Color32::BLACK.gamma_multiply(0.45)`) and draws 1 px `NEUTRAL_500` lines at
cue-in and cue-out. Test: geometry via `WaveView::trimmed` (Task 2) plus a
kittest that a trimmed track still reports the waveform node.

### Task 4: Drag to seek

`waveform` (input gains `view: WaveView`) keeps a drag preview in egui temp
data keyed on the waveform id: started by a primary drag without Alt; drawn as
a line with the time; on release inside, `seek = secs_at(x)`; outside or after
Esc, nothing. Tests (kittest, `markers_ui.rs` style events): release inside →
one `Seek` at the release time (±1 s of 180 s); release outside → none; Esc
then release inside → none; Alt-drag → no `Seek`.

### Task 5: Zoom, pan, follow, Full view

- `ViewState.wave_zoom: HashMap<PlayerId, WaveZoom { view, entry, panned_at }>`
  (absent = full view).
- In `player::wave`, while hovered: each wheel event zooms by 0.8 / 1.25
  around the pointer (vertical delta) or pans by 10 % of the span (Shift, or a
  horizontal delta), and the frame's scroll delta is zeroed so the players row
  does not scroll. `panned_at = scene.time` on pan.
- Each frame: a different current entry drops the zoom; a full span drops it;
  if the playhead is outside the view and `scene.time − panned_at ≥
  follow_current_grace_secs`, `follow`.
- A "Full view" tile (tooltip "Full view") in the top-right corner while zoomed;
  the outro badge moves left of it.
- `edit_markers` and the hover tooltip use the same view.
- Config: `UiConfig.follow_current_grace_secs: f64` (10, 0–600).
- Tests (kittest): wheel-up over the waveform then click at the centre seeks
  near the zoomed centre, not the full-view centre; "Full view" appears only
  when zoomed and resets; the zoom resets on a track change; an Alt marker
  drag in a zoomed view sets the zoomed time; the wheel does not change the
  players row scroll offset. Config test for the new field.

### Task 6: Docs, review, PR

Main spec §8.3 (waveform), `docs/user/players.md`, `markers-and-mixing.md`,
`docs/technical/ui.md`, `persistence.md` (`ui.follow_current_grace_secs`),
settings doc if listed. Review, PR `feat(ui): waveform drag, zoom and trimmed
regions`, CI, merge.
