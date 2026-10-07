# Meter Rulers (Feedback 4, Plan 4) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The level meter draws nothing across its bars or the gap between them (Q11.1), and its scale is a ruler on each side of the bars: labels on both sides, a tick for every labelled mark, minor ticks between, and a thicker white alignment tick (Q11.2 to Q11.6), with the player column making room for the wider meter at 380 px (Q11.7).

**Architecture:**
- **Layout first.** `widgets::meter_layout` (`crates/fp-app/src/ui/widgets.rs`) stays the one pure source of geometry. It gains two `Ruler`s (a label anchor and a tick strip per side) and a `ticks: Vec<MeterTick>` list (major, minor, alignment). A pure `minor_marks(&MeterConfig)` gives the minor levels per scale; the layout keeps a minor tick only where it is at least 3 px from every other tick. The painting is then a loop over rulers and ticks; nothing else is painted outside the bars.
- **Nothing over the bars.** `reference_segments`, `LineShade`, `alignment_notches`, `lines_x` and the `METER_LINE_*` constants are deleted. A kittest reads the shapes `vu` really paints and checks that none crosses a bar edge or touches the gap.
- **Player column.** `METER_WIDTH` grows by 26 px; `top_block` in `player.rs` already derives the left part from `METER_COLUMN_WIDTH`, so the work is checking the 380 px geometry and, only if the existing countdown tests fail, a fixed ladder of small adjustments.

**Tech Stack:** Rust, egui/eframe 0.36.2, egui_kittest 0.36.2. No new dependency, so `cargo deny check` needs no new entry. No strings change, so no Fluent message is added or edited (both `crates/fp-app/locales/en-US/main.ftl` and `es-ES/main.ftl` are left as they are; the meter's accessible names `meter-label`, `meter-loudness`, `meter-max` are untouched).

**Spec:** `docs/superpowers/specs/2026-10-06-operator-feedback-4-design.md` §5 (Q11, rules Q11.1 to Q11.7) and §9 (global constraints). Roadmap entry: §7, plan 4, branch `fix/operator-feedback-4`. Related, already merged: `docs/superpowers/specs/2026-09-27-meters-design.md` (M4), `docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md` (O11, O13).

## Global Constraints

- `CLAUDE.md` rules 1 to 10 apply to every task.
- All code, identifiers, comments, docs, specs, plans and commit messages are in English; never mention other playout or radio-automation products.
- UI strings are Fluent messages in both locales. This plan adds none.
- Real-time safety: the meter is painted on the UI thread from a snapshot (`MeterReading`); nothing here touches the device callback.
- `unsafe_code` forbidden; `unwrap`, `expect` and `panic` denied outside tests; prefer `get` over indexing (the `fp-app` crate allows indexing in tests only, as the existing test files do).
- The UI never blocks and never crashes on bad data: a rect too short to draw, a custom floor next to a mark, or `reference_dbfs` at the ends of its range lay out without panicking.
- Spec Q11 rules, verbatim:
  - **Q11.1** Nothing is drawn inside or across the level bars or the gap between them; the bars show only the level, the peak hold and the zones.
  - **Q11.2** The scale is a ruler on each side of the bars, with the labels on both sides.
  - **Q11.3** Every labelled mark has a tick on both rulers.
  - **Q11.4** Minor ticks sit between the labelled marks, as on a measuring ruler, with a spacing chosen per scale (for example every 1 dB on EBU PPM, every 5 dB on the digital scale below −20), and only where they stay at least 3 px apart.
  - **Q11.5** The alignment (reference) level is a thicker tick, always white, on both rulers, never inside a bar.
  - **Q11.6** The label crowding rules of M4 (both ends first, then top down 10 px apart) still hold on each ruler.
  - **Q11.7** The meter grows by one label column; the player column's layout makes room for it at the minimum width (380 px).
- Commit gate for every task: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`.
- Commit messages are Conventional Commits and end with the trailer `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- `CHANGELOG.md` is never edited by hand.

## Review Focus

Failure modes the spec implies but the Q11 test list does not name. Each has a test in the task that owns the code.

1. **A scale mark crowded out of the labels (most likely).** At 64 px many marks lose their label; they must still get a tick (a minor one), not vanish. Task 1, `crowded_marks_keep_a_minor_tick`.
2. **Unlabelled alignment level.** The digital meter's −18 dBFS has no label; its white tick must still be on both rulers. Task 1, `the_alignment_tick_is_thicker_and_unique`; Task 2, `the_alignment_tick_is_white_on_both_rulers`.
3. **Marks near the rect edge.** The floor and the top mark sit on the first and last pixel of the bars: their ticks must stay inside the meter's rect and not be doubled by a minor tick beside them. Task 1, `ticks_stay_inside_the_rect_and_apart`.
4. **Custom or odd scale settings.** A digital floor of −57 or −55 (not a multiple of 5) and an EBU `reference_dbfs` other than −18 shift the marks; minor ticks must follow and not land on a mark. Task 1, `minor_marks_follow_a_custom_floor_and_reference`.
5. **Loudness line on.** The bars are shorter (12 px less); minor ticks must thin out, never touch. Task 1, `ticks_stay_inside_the_rect_and_apart` runs both `loudness` values.

---

## Decisions taken

The spec leaves these open. Each is the most conservative reading.

Ruling: ticks sit in a 4 px strip outside each bar with a 1 px gap to the bar, and the labels sit outside the strip — this is what a measuring ruler looks like and it keeps both ticks and labels clear of the bars — a label column at the bar side would need to be re-measured if wrong.

Ruling: "one label column" is read as a label column on each side plus a tick strip, so `METER_WIDTH` goes from 52 to 78 px (2 × (18 label + 1 + 4 strip + 1) + 2 × 14 bar + 2 gap) — two rulers cannot share the old single 18 px column — the meter column in the player is 26 px wider and the left part shrinks by the same amount (Task 1 checks it fits at 380 px).

Ruling: a scale mark that loses its label to the crowding rules becomes a minor tick (the rule in the meters spec: "A mark without a label gets a minor tick") — it keeps the scale readable at 64 px — none if wrong beyond a few ticks.

Ruling: minor steps per scale, taken from the segment that starts at each scale mark: digital and custom 5 dB below −20 dBFS and 1 dB from −20 up; EBU PPM 1 dB everywhere; DIN PPM 5 dB below −20 dB relative to its 0 and 1 dB above; VU 5 dB below −10 VU, 1 dB from −10 to −3, 0.5 dB from −3 up; K-System 5 dB below −24 relative to its 0 and 1 dB above — the spec only gives two examples, and these follow the spacing of each standard's own marks — only the number of ticks changes if the maintainer prefers other steps (one table, `minor_step_db`).

Ruling: colours are a major tick in `NEUTRAL_400`, a minor tick in `NEUTRAL_400` at 60 % opacity and the alignment tick `Color32::WHITE` — same family as the labels, the alignment tick the only white thing on the meter (Q11.5) — three constants in `theme.rs`, pinned by `tests/theme.rs`.

Ruling: the "nothing is drawn over the bars" test reads the shapes `vu` paints through `Context::graphics_mut` inside the same `ui` closure, before the frame ends (the shapes are drained at the end of the pass) — it tests what is really painted, not only the layout — if the egui API differs, fall back to asserting on the layout rects (`ticks`, label anchors) and keep the painter loop trivial.

Ruling: the Q11 edits to the meters spec (M4) and to feedback 2 (O13) are already in the tree (checked while writing this plan: `2026-09-27-meters-design.md` lines 99, 110 and 111 and `2026-10-01-operator-feedback-2-design.md` line 182 carry them), and the "Main spec §8.3, Meter column" text the spec quotes is not present in any file of the repo — Task 3 verifies the first two with `grep`, adds the O13 "As built" correction and does not invent the third — if the maintainer has it elsewhere, one line to change.

Ruling: if the two existing 380 px countdown tests fail after the meter widens, a fixed ladder is applied in this order until they pass: `METER_COLUMN_GAP` 10 to 6, then `COUNTDOWN_MIN_SCALE` 18/38 to 16/38, then `LABEL_COLUMN` 18 to 16 — the last two shrink the countdown and the label column, which is visible, so they come last — the operator sees a slightly smaller hour-long countdown at the minimum width.

---

## File Structure

| Path | Change |
|---|---|
| `crates/fp-app/src/ui/widgets.rs` | `Ruler`, `Side`, `MeterTick`, `TickKind`, `minor_marks`, `minor_step_db`, `ruler_ticks`, `tick_colour`; `meter_layout` and `vu` rewritten; `reference_segments`, `LineShade`, notches, `lines_x` removed |
| `crates/fp-app/src/ui/theme.rs` | `METER_TICK`, `METER_TICK_MINOR_ALPHA`, `METER_ALIGNMENT_TICK` replace `METER_LINE_*` |
| `crates/fp-app/src/ui/player.rs` | `METER_COLUMN_WIDTH` follows the new width; ladder constants only if needed |
| `crates/fp-app/tests/meter_view.rs` | line tests become ruler, tick and paint tests |
| `crates/fp-app/tests/theme.rs` | pins the tick colours |
| `crates/fp-app/tests/main_screen.rs` | the 380 px geometry test |
| `docs/user/players.md`, `docs/technical/ui.md`, `docs/technical/audio-engine.md` | meter description |
| `docs/superpowers/specs/2026-10-06-operator-feedback-4-design.md` | "As built" notes under Plan 4 |
| `docs/images/main-screen.png`, `docs/images/guide/player.png` | refreshed screenshots |

---

### Task 1: Ruler layout and the player column (suggested executor: opus)

Pure geometry, with subtle invariants (crowding, spacing, edge clamping) and a change to a width that other layouts depend on.

**Files:**
- Modify: `crates/fp-app/src/ui/widgets.rs` (constants ~L612 to L630; `MeterLayout` ~L689; `meter_layout` ~L725 to L850; `scale_marks` is ~L351, unchanged)
- Modify: `crates/fp-app/src/ui/player.rs:419-425` only if the ladder is needed
- Test: `crates/fp-app/tests/meter_view.rs`, `crates/fp-app/tests/main_screen.rs`
- Docs and locales: none in this task (Task 3 writes the docs; no strings change, so `en-US/main.ftl` and `es-ES/main.ftl` are untouched).

**Interfaces:**
- Consumes: `scale_marks(&MeterConfig) -> Vec<f32>`, `alignment_dbfs`, `meter_position`, `mark_label`, `SAME_MARK_DB`, `LABEL_ROW` (all existing in `widgets.rs`).
- Produces (all `pub` in `fp_app::ui::widgets`):
  - `pub const LABEL_COLUMN: f32 = 18.0;` (was private) and `pub const METER_WIDTH: f32 = 78.0` (computed).
  - `pub enum Side { Left, Right }` (derives `Debug, Clone, Copy, PartialEq, Eq`).
  - `pub enum TickKind { Minor, Major, Alignment }` (same derives).
  - `pub struct MeterTick { pub y: f32, pub kind: TickKind }` (`Debug, Clone, PartialEq`).
  - `pub struct Ruler { pub side: Side, pub labels_x: f32, pub label_halign: egui::Align, pub ticks: egui::Rangef }` with `pub fn tick_rect(&self, t: &MeterTick) -> Rect`. `labels_x` is the label anchor: the right edge of the label column on the left ruler (`label_halign` is `Align::Max`), the left edge on the right ruler (`Align::Min`). `ticks` is the 4 px strip next to the bar; ticks are flush with the bar side of the strip.
  - `MeterLayout` gains `pub rulers: [Ruler; 2]` (left, right) and `pub ticks: Vec<MeterTick>` (sorted by `y`). The old fields `labels_right`, `lines_x`, `alignment_notches` stay for this task so `vu` still paints (Task 2 removes them).
  - `pub fn minor_marks(c: &MeterConfig) -> Vec<f32>`: the levels (dBFS, ascending) of the minor ticks, never on a scale mark.

- [x] **Step 1: Write the failing tests in `crates/fp-app/tests/meter_view.rs`**

Extend the `use fp_app::ui::widgets::{…}` list with `LABEL_COLUMN, MeterTick, Side, TickKind, minor_marks`. Append:

```rust
fn has(marks: &[f32], x: f32) -> bool {
    marks.iter().any(|m| near(*m, x))
}

fn minors_between(marks: &[f32], low: f32, high: f32) -> usize {
    marks.iter().filter(|m| **m > low + 1e-3 && **m < high - 1e-3).count()
}

#[test]
fn the_meter_has_a_ruler_on_each_side_of_the_bars() {
    let rect = column(136.0);
    let l = meter_layout(rect, &meter(MeterBallistics::DigitalPeak), false);
    let [left, right] = l.bars;
    let [lr, rr] = &l.rulers;
    assert_eq!((lr.side, rr.side), (Side::Left, Side::Right));
    assert!(lr.ticks.max <= left.left(), "left ticks end before the bar");
    assert!(lr.labels_x <= lr.ticks.min, "left labels sit outside the ticks");
    assert!(rr.ticks.min >= right.right(), "right ticks start after the bar");
    assert!(rr.labels_x >= rr.ticks.max, "right labels sit outside the ticks");
    assert_eq!(lr.label_halign, egui::Align::Max);
    assert_eq!(rr.label_halign, egui::Align::Min);
    // The label columns fit in the meter's rect.
    assert!(lr.labels_x - LABEL_COLUMN >= rect.left() - 0.01);
    assert!(rr.labels_x + LABEL_COLUMN <= rect.right() + 0.01);
    // The gap between the bars is unchanged and empty.
    assert!(near(right.left() - left.right(), 2.0));
    assert_eq!(METER_WIDTH, 78.0);
}

#[test]
fn every_labelled_mark_has_a_tick_on_both_rulers() {
    for ballistics in ALL_METERS {
        for height in [64.0, 136.0, 300.0] {
            let l = meter_layout(column(height), &meter(ballistics), false);
            for m in l.lines.iter().filter(|m| !m.label.is_empty()) {
                let tick = l
                    .ticks
                    .iter()
                    .find(|t| near(t.y, m.y) && t.kind != TickKind::Minor)
                    .unwrap_or_else(|| panic!("{ballistics:?} {height}: no tick for {}", m.label));
                let [lr, rr] = &l.rulers;
                let (a, b) = (lr.tick_rect(tick), rr.tick_rect(tick));
                assert!(near(a.center().y, tick.y) && near(b.center().y, tick.y));
                assert!(near(a.width(), b.width()), "the rulers mirror each other");
                assert!(near(a.right(), lr.ticks.max), "flush with the bar side");
                assert!(near(b.left(), rr.ticks.min), "flush with the bar side");
            }
        }
    }
}

#[test]
fn the_alignment_tick_is_thicker_and_unique() {
    for ballistics in ALL_METERS {
        let c = meter(ballistics);
        let l = meter_layout(column(136.0), &c, false);
        let alignment: Vec<&MeterTick> = l
            .ticks
            .iter()
            .filter(|t| t.kind == TickKind::Alignment)
            .collect();
        assert_eq!(alignment.len(), 1, "{ballistics:?}");
        let line = l.lines.iter().find(|m| m.alignment).unwrap();
        assert!(near(alignment[0].y, line.y), "{ballistics:?}");
        let [lr, rr] = &l.rulers;
        let a = lr.tick_rect(alignment[0]);
        let major = l.ticks.iter().find(|t| t.kind == TickKind::Major).unwrap();
        let m = lr.tick_rect(major);
        assert!(a.height() > m.height(), "{ballistics:?}: thicker");
        assert!(a.width() >= m.width(), "{ballistics:?}");
        assert!(rr.tick_rect(alignment[0]).height() > m.height());
        // Never inside a bar.
        for bar in l.bars {
            assert!(!a.intersects(bar) && !rr.tick_rect(alignment[0]).intersects(bar));
        }
    }
}

#[test]
fn minor_ticks_sit_between_the_marks_at_a_spacing_per_scale() {
    // Digital: 1 dB from -20 up, 5 dB below it (spec Q11.4).
    let c = meter(MeterBallistics::DigitalPeak);
    let m = minor_marks(&c);
    assert!(has(&m, -55.0) && has(&m, -45.0));
    assert!(has(&m, -19.0) && has(&m, -16.0) && has(&m, -1.0));
    assert!(!has(&m, -20.0) && !has(&m, -52.0), "never on a mark");
    assert_eq!(minors_between(&m, -60.0, -20.0), 2);
    assert_eq!(minors_between(&m, -20.0, 0.0), 16);
    // EBU: every 1 dB, three between each pair of marks, six gaps.
    let c = meter(MeterBallistics::EbuPpm);
    assert_eq!(minor_marks(&c).len(), 18);
    // DIN: 5 dB below -20 relative to its 0, 1 dB above.
    let c = meter(MeterBallistics::DinPpm);
    assert_eq!(minor_marks(&c).len(), 24);
    // VU: 5 dB below -10, 1 dB to -3, 0.5 dB above.
    let c = meter(MeterBallistics::Vu);
    let m = minor_marks(&c);
    assert_eq!(m.len(), 11);
    assert!(has(&m, c.reference_dbfs - 2.5) && has(&m, c.reference_dbfs - 15.0));
    // K-20 (0 at -20 dBFS): 1 dB between -24 and -20 relative to its 0.
    let c = meter(MeterBallistics::K20);
    let k = alignment_dbfs(&c);
    let m = minor_marks(&c);
    assert_eq!(minors_between(&m, k - 24.0, k - 20.0), 3);
    assert!(has(&m, k - 55.0));
}

#[test]
fn minor_marks_follow_a_custom_floor_and_reference() {
    let mut c = meter(MeterBallistics::DigitalPeak);
    c.floor_db = -57.0;
    let m = minor_marks(&c);
    assert!(has(&m, -52.0), "5 dB above the odd floor's first segment");
    assert!(!has(&m, -57.0), "the floor is a mark");
    let mut c = meter(MeterBallistics::EbuPpm);
    c.reference_dbfs = -20.0;
    let m = minor_marks(&c);
    assert!(has(&m, -20.0 + 1.0) && !has(&m, -20.0), "TEST is a mark");
    assert_eq!(m.len(), 18);
}

#[test]
fn ticks_stay_inside_the_rect_and_apart() {
    for ballistics in ALL_METERS {
        for height in [64.0, 136.0, 300.0] {
            for loudness in [false, true] {
                let rect = column(height);
                let l = meter_layout(rect, &meter(ballistics), loudness);
                let bars = l.bars[0];
                for (i, t) in l.ticks.iter().enumerate() {
                    assert!(
                        t.y >= bars.top() - 0.01 && t.y <= bars.bottom() + 0.01,
                        "{ballistics:?} {height} {loudness}: {t:?} outside the bars' height"
                    );
                    for r in &l.rulers {
                        assert!(rect.contains_rect(r.tick_rect(t)), "{t:?}");
                    }
                    if t.kind == TickKind::Minor {
                        for (j, o) in l.ticks.iter().enumerate() {
                            if i != j {
                                assert!(
                                    (o.y - t.y).abs() >= 3.0 - 0.01,
                                    "{ballistics:?} {height} {loudness}: minor {t:?} next to {o:?}"
                                );
                            }
                        }
                    }
                }
                let ys: Vec<f32> = l.ticks.iter().map(|t| t.y).collect();
                assert!(ys.windows(2).all(|p| p[0] <= p[1]), "sorted by y");
            }
        }
    }
}

#[test]
fn crowded_marks_keep_a_minor_tick() {
    // At 64 px labels are thinned out; every scale mark still has a tick.
    for ballistics in [MeterBallistics::DigitalPeak, MeterBallistics::DinPpm] {
        let c = meter(ballistics);
        let l = meter_layout(column(136.0), &c, false);
        let bars = l.bars[0];
        for mark in scale_marks(&c) {
            let y = bars.bottom() - meter_position(mark, &c) * bars.height();
            assert!(
                l.ticks.iter().any(|t| (t.y - y).abs() < 1.0),
                "{ballistics:?}: no tick at {mark}"
            );
        }
    }
    let c = meter(MeterBallistics::DigitalPeak);
    let small = meter_layout(column(64.0), &c, false);
    let labelled = small.lines.iter().filter(|m| !m.label.is_empty()).count();
    let majors = small.ticks.iter().filter(|t| t.kind == TickKind::Major).count();
    assert!(majors <= labelled, "a major tick only for a label");
}

#[test]
fn a_taller_meter_has_more_minor_ticks() {
    let c = meter(MeterBallistics::DigitalPeak);
    let count = |h: f32| {
        meter_layout(column(h), &c, false)
            .ticks
            .iter()
            .filter(|t| t.kind == TickKind::Minor)
            .count()
    };
    assert!(count(300.0) > count(64.0));
}
```

In `crates/fp-app/tests/main_screen.rs`, after `an_hour_long_countdown_fits_between_the_grid_and_the_meter` (~L640), add:

```rust
#[test]
fn the_meter_with_its_rulers_fits_the_minimum_player_width() {
    let (h, _) =
        support::harness_sized(quiet_meter(state(1, 3)), egui::vec2(380.0, 700.0), |ui| ui);
    let meter = h.get_by_label("Level meter").rect();
    let fader = h.get_by_label("Volume").rect();
    let grid = h.get_by_label("Stop after the current track").rect();
    assert_eq!(meter.width(), fp_app::ui::widgets::METER_WIDTH);
    assert!(fader.right() <= 380.0 + 0.5, "{fader:?}");
    assert!(meter.left() >= grid.right(), "{meter:?} {grid:?}");
    // The left part keeps room for the transport grid.
    assert!(grid.width() > 0.0 && meter.left() - grid.right() >= 0.0);
}
```

(If `get_by_label("Volume")` is not the fader's accessible name, copy the label the existing test `the_meter_and_fader_fill_the_height_they_are_given` style uses in `main_screen.rs`: run `grep -n "fader" crates/fp-app/tests/main_screen.rs` and use the label found there.)

- [x] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-app --test meter_view 2>&1 | tail -20`
Expected: compile error, `no field rulers` / unresolved imports `Side`, `TickKind`, `minor_marks`, `MeterTick`, `LABEL_COLUMN`.

- [x] **Step 3: Write the implementation in `widgets.rs`**

Replace the width constants (~L612) with:

```rust
/// Width of the label column on each side of the bars.
pub const LABEL_COLUMN: f32 = 18.0;
/// Gap between a ruler's tick strip and its bar (and between the strip and
/// its labels, on the side of the bar the labels are not on).
const RULER_GAP: f32 = 1.0;
/// Width of a ruler's tick strip: the length of a major tick.
const TICK_STRIP: f32 = 4.0;
/// Length of a minor tick.
const MINOR_TICK_LEN: f32 = 2.0;
/// Thickness of a major or minor tick; the alignment tick is
/// [`ALIGNMENT_LINE_THICKNESS`].
const TICK_THICKNESS: f32 = 1.0;
/// Minor ticks stay at least this far (px) from every other tick.
const MIN_TICK_SPACING: f32 = 3.0;
/// One side of the meter: label column, tick strip and their gaps.
const SIDE_WIDTH: f32 = LABEL_COLUMN + RULER_GAP + TICK_STRIP + RULER_GAP;
const BAR_WIDTH: f32 = 14.0;
const BAR_GAP: f32 = 2.0;
/// Width of the meter: a ruler on each side of the two bars.
pub const METER_WIDTH: f32 = 2.0 * SIDE_WIDTH + 2.0 * BAR_WIDTH + BAR_GAP;
```

Delete the old `LABEL_COLUMN`, `LABEL_GAP`, `BAR_WIDTH`, `BAR_GAP` and `METER_WIDTH` definitions they replace (keep `LABEL_GAP` out: it is no longer used; `ALIGNMENT_NOTCH` stays until Task 2).

Add after `MeterLine`'s impl:

```rust
/// A side of the meter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

/// The kind of a ruler tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickKind {
    /// Between the labelled marks, shorter and fainter.
    Minor,
    /// A labelled mark.
    Major,
    /// The alignment level: thicker, white.
    Alignment,
}

/// One tick of the rulers, at screen height `y`.
#[derive(Debug, Clone, PartialEq)]
pub struct MeterTick {
    pub y: f32,
    pub kind: TickKind,
}

/// One ruler: where its labels hang and the strip its ticks are drawn in.
#[derive(Debug, Clone, PartialEq)]
pub struct Ruler {
    pub side: Side,
    /// Label anchor: the right edge of the label column on the left ruler
    /// (`label_halign` is `Max`), the left edge on the right ruler (`Min`).
    pub labels_x: f32,
    pub label_halign: egui::Align,
    /// The tick strip, next to the bar.
    pub ticks: egui::Rangef,
}

impl Ruler {
    /// The rectangle `t` is painted in: flush with the bar side of the
    /// strip, centred on its `y`.
    pub fn tick_rect(&self, t: &MeterTick) -> Rect {
        let (len, thickness) = match t.kind {
            TickKind::Minor => (MINOR_TICK_LEN, TICK_THICKNESS),
            TickKind::Major => (TICK_STRIP, TICK_THICKNESS),
            TickKind::Alignment => (TICK_STRIP, ALIGNMENT_LINE_THICKNESS),
        };
        let x = match self.side {
            Side::Left => self.ticks.max - len..=self.ticks.max,
            Side::Right => self.ticks.min..=self.ticks.min + len,
        };
        Rect::from_x_y_ranges(x, t.y - thickness / 2.0..=t.y + thickness / 2.0)
    }
}
```

Add `pub rulers: [Ruler; 2]` (left, right) and `pub ticks: Vec<MeterTick>` to `MeterLayout`.

Add after `scale_marks`:

```rust
/// The step (dB) of the minor ticks in the scale segment that starts at
/// `from` dBFS: coarse in the sparse low part of each scale, fine where the
/// marks are close.
fn minor_step_db(c: &MeterConfig, from: f32) -> f32 {
    let below = |zero: f32, limit: f32| from - zero < limit - SAME_MARK_DB;
    match c.ballistics {
        MeterBallistics::EbuPpm => 1.0,
        MeterBallistics::DinPpm => {
            if below(c.reference_dbfs + PERMITTED_MAXIMUM_DB, -20.0) {
                5.0
            } else {
                1.0
            }
        }
        MeterBallistics::Vu => {
            if !below(c.reference_dbfs, -3.0) {
                0.5
            } else if !below(c.reference_dbfs, -10.0) {
                1.0
            } else {
                5.0
            }
        }
        MeterBallistics::K20 | MeterBallistics::K14 | MeterBallistics::K12 => {
            if below(alignment_dbfs(c), -24.0) {
                5.0
            } else {
                1.0
            }
        }
        MeterBallistics::DigitalPeak | MeterBallistics::Custom => {
            if below(0.0, -20.0) {
                5.0
            } else {
                1.0
            }
        }
    }
}

/// The levels (dBFS, ascending) of the minor ticks: between each pair of
/// adjacent scale marks, at the step of [`minor_step_db`], never on a mark.
pub fn minor_marks(c: &MeterConfig) -> Vec<f32> {
    let marks = scale_marks(c);
    let mut out = Vec::new();
    for pair in marks.windows(2) {
        let (Some(&a), Some(&b)) = (pair.first(), pair.get(1)) else {
            continue;
        };
        let step = minor_step_db(c, a);
        let mut n = 1.0_f32;
        while a + step * n < b - SAME_MARK_DB {
            out.push(a + step * n);
            n += 1.0;
        }
    }
    out
}

/// The ticks of both rulers: a major tick for every labelled mark, the
/// alignment tick, then the scale marks that lost their label and the
/// minor marks, each kept only where it is at least [`MIN_TICK_SPACING`]
/// from every tick already kept. Sorted by `y`.
fn ruler_ticks(lines: &[MeterLine], c: &MeterConfig, top: f32, bottom: f32) -> Vec<MeterTick> {
    let height = bottom - top;
    let y_of = |db: f32| bottom - meter_position(db, c) * height;
    let mut ticks: Vec<MeterTick> = lines
        .iter()
        .filter(|m| m.alignment || !m.label.is_empty())
        .map(|m| MeterTick {
            y: m.y,
            kind: if m.alignment {
                TickKind::Alignment
            } else {
                TickKind::Major
            },
        })
        .collect();
    let marks = scale_marks(c);
    let minors = minor_marks(c);
    for db in marks.iter().rev().chain(minors.iter().rev()) {
        if (db - alignment_dbfs(c)).abs() < SAME_MARK_DB {
            continue;
        }
        let y = y_of(*db);
        if y < top || y > bottom {
            continue;
        }
        if ticks.iter().all(|t| (t.y - y).abs() >= MIN_TICK_SPACING) {
            ticks.push(MeterTick {
                y,
                kind: TickKind::Minor,
            });
        }
    }
    ticks.sort_by(|a, b| a.y.total_cmp(&b.y));
    ticks
}
```

In `meter_layout`: change `let bars_left = rect.left() + LABEL_COLUMN + LABEL_GAP;` to `let bars_left = rect.left() + SIDE_WIDTH;`. After `bars_right` is computed add:

```rust
    let rulers = [
        Ruler {
            side: Side::Left,
            labels_x: rect.left() + LABEL_COLUMN,
            label_halign: egui::Align::Max,
            ticks: egui::Rangef::new(
                bars_left - RULER_GAP - TICK_STRIP,
                bars_left - RULER_GAP,
            ),
        },
        Ruler {
            side: Side::Right,
            labels_x: bars_right + RULER_GAP + TICK_STRIP + RULER_GAP,
            label_halign: egui::Align::Min,
            ticks: egui::Rangef::new(bars_right + RULER_GAP, bars_right + RULER_GAP + TICK_STRIP),
        },
    ];
```

After `lines.sort_by(...)`, build `let ticks = ruler_ticks(&lines, c, top, bottom);` and put `rulers` and `ticks` in the returned `MeterLayout` (keep `labels_right: rect.left() + LABEL_COLUMN`, `lines_x`, `alignment_notches` as they are; `vu` still uses them until Task 2). Update the doc comment of `meter_layout`: "the labels and ticks of a ruler on each side of the two bars".

- [x] **Step 4: Run the layout tests**

Run: `cargo test -p fp-app --test meter_view 2>&1 | tail -30`
Expected: all new tests PASS. The old tests (`lines_cross_both_bars_and_nothing_sits_between_them`, notches, segments) still pass because the old fields are kept. If `minor_marks_follow_a_custom_floor_and_reference` fails on a float edge, check `while a + step * n < b - SAME_MARK_DB` accumulates `n` as a float count, not a repeated sum.

- [x] **Step 5: Run the 380 px tests and apply the ladder only if needed**

Run: `cargo test -p fp-app --test main_screen -- the_meter_with_its_rulers at_the_minimum_player_width an_hour_long_countdown 2>&1 | tail -20`
Expected: PASS. If one of the two countdown tests fails (the meter column is 26 px wider), apply in `crates/fp-app/src/ui/player.rs` one rung at a time and re-run after each: (a) `const METER_COLUMN_GAP: f32 = 6.0;` (was 10.0, ~L420); (b) `const COUNTDOWN_MIN_SCALE: f32 = 16.0 / 38.0;` (~L425); (c) `pub const LABEL_COLUMN: f32 = 16.0;` in `widgets.rs` (then `METER_WIDTH` is 74 and `assert_eq!(METER_WIDTH, 78.0)` in the first test becomes `74.0`). Record the rung used in the commit body.

- [x] **Step 6: Commit**

```bash
if cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings \
   && cargo test --workspace; then
git add crates/fp-app/src/ui/widgets.rs crates/fp-app/src/ui/player.rs \
        crates/fp-app/tests/meter_view.rs crates/fp-app/tests/main_screen.rs
git commit -m "feat(ui): lay the meter out with a ruler on each side

meter_layout now returns a ruler per side (label anchor and tick strip)
and the ticks: a major tick per label, minor ticks between marks at a
step per scale, and the alignment tick. Minor ticks keep 3 px from every
other tick. The meter is 26 px wider; the player column still fits at
380 px (operator feedback 4, Q11.2 to Q11.7).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
fi
```

---

### Task 2: Paint the rulers and draw nothing over the bars (suggested executor: sonnet)

Mechanical: switch the painter to the new layout, delete the old API and constants, update the tests.

**Files:**
- Modify: `crates/fp-app/src/ui/widgets.rs` (`vu` ~L472 to L600: the scale loop and the notch loop; delete `reference_segments`, `LineShade`, `ALIGNMENT_NOTCH`, and the `labels_right`, `lines_x`, `alignment_notches` fields of `MeterLayout`; doc comment of `vu`)
- Modify: `crates/fp-app/src/ui/theme.rs:37-42`
- Modify: `crates/fp-app/tests/meter_view.rs`, `crates/fp-app/tests/theme.rs:130-140`
- Docs and locales: none in this task (Task 3); no strings.

**Interfaces:**
- Consumes: `MeterLayout::{rulers, ticks, lines, bars}`, `Ruler::tick_rect`, `TickKind` (Task 1).
- Produces: `pub fn tick_colour(kind: TickKind) -> Color32` in `widgets.rs`; in `theme.rs`: `pub const METER_TICK: Color32 = NEUTRAL_400;`, `pub const METER_TICK_MINOR_ALPHA: f32 = 0.60;`, `pub const METER_ALIGNMENT_TICK: Color32 = Color32::WHITE;`.

- [x] **Step 1: Write the failing tests**

In `crates/fp-app/tests/theme.rs` replace `the_meter_reference_lines_are_visible_over_both_parts_of_the_bar` with:

```rust
#[test]
fn the_meter_ticks_are_pinned() {
    // Operator feedback 4, Q11: the alignment tick is the only white mark
    // on the meter; minor ticks are the major tick's colour, fainter.
    assert_eq!(theme::METER_TICK, theme::NEUTRAL_400);
    assert_eq!(theme::METER_TICK_MINOR_ALPHA, 0.60);
    assert_eq!(theme::METER_ALIGNMENT_TICK, Color32::WHITE);
}
```

In `crates/fp-app/tests/meter_view.rs`: remove `LineShade` and `reference_segments` from the imports, add `tick_colour`; delete the tests `lines_cross_both_bars_and_nothing_sits_between_them`, `the_alignment_level_is_two_notches_at_the_outer_edges`, `a_reference_line_is_cut_dark_over_the_lit_bar_and_light_over_the_rest` and `reference_segments_are_unlit_at_the_floor_and_lit_at_the_top`. Append:

```rust
#[test]
fn tick_colours_follow_the_kind() {
    assert_eq!(tick_colour(TickKind::Major), fp_app::ui::theme::METER_TICK);
    assert_eq!(
        tick_colour(TickKind::Minor),
        fp_app::ui::theme::METER_TICK.gamma_multiply(fp_app::ui::theme::METER_TICK_MINOR_ALPHA)
    );
    assert_eq!(tick_colour(TickKind::Alignment), egui::Color32::WHITE);
}

#[test]
fn the_alignment_tick_is_white_on_both_rulers() {
    // The digital meter's alignment level (-18 dBFS) has no label: its tick
    // is still there, and white.
    let l = meter_layout(column(136.0), &meter(MeterBallistics::DigitalPeak), false);
    let a = l.lines.iter().find(|m| m.alignment).unwrap();
    assert!(a.label.is_empty());
    let tick = l.ticks.iter().find(|t| t.kind == TickKind::Alignment).unwrap();
    assert_eq!(tick_colour(tick.kind), egui::Color32::WHITE);
    for r in &l.rulers {
        assert!(near(r.tick_rect(tick).center().y, a.y));
    }
}

/// Spec Q11.1: of everything `vu` paints, only the bars' own content (the
/// background, the level, the hold and the zones) touches a bar, and nothing
/// touches the gap between them.
#[test]
fn nothing_is_drawn_over_the_bars_or_between_them() {
    use std::cell::RefCell;
    for ballistics in ALL_METERS {
        for loudness in [false, true] {
            let c = MeterConfig {
                loudness: if loudness {
                    LoudnessReadout::Momentary
                } else {
                    LoudnessReadout::Off
                },
                ..meter(ballistics)
            };
            let seen: RefCell<Vec<egui::Rect>> = RefCell::new(Vec::new());
            let bars: RefCell<[egui::Rect; 2]> = RefCell::new([egui::Rect::NOTHING; 2]);
            let mut h = egui_kittest::Harness::new_ui(|ui| {
                let labels = fp_app::ui::widgets::MeterLabels {
                    meter: "Level meter".to_owned(),
                    max: "Maximum".to_owned(),
                    max_tip: String::new(),
                };
                let mut r = reading(-9.0, -3.0);
                r.rms_db = [-14.0; 2];
                r.max_db = -3.0;
                r.lufs_momentary = Some(-23.0);
                let layer = ui.layer_id();
                let before = ui.ctx().graphics_mut(|g| g.entry(layer).all_entries().len());
                fp_app::ui::widgets::vu(ui, 136.0, &r, &c, &labels);
                let rect = ui.min_rect();
                let l = meter_layout(
                    egui::Rect::from_min_size(rect.min, egui::vec2(METER_WIDTH, 136.0)),
                    &c,
                    loudness,
                );
                *bars.borrow_mut() = l.bars;
                *seen.borrow_mut() = ui.ctx().graphics_mut(|g| {
                    g.entry(layer)
                        .all_entries()
                        .skip(before)
                        .map(|s| s.shape.visual_bounding_rect())
                        .collect()
                });
            });
            h.run();
            let [left, right] = *bars.borrow();
            let gap = egui::Rect::from_x_y_ranges(left.right()..=right.left(), left.y_range());
            let shapes = seen.borrow();
            assert!(!shapes.is_empty(), "{ballistics:?}: nothing painted");
            for s in shapes.iter() {
                assert!(
                    !(s.intersects(gap) && gap.width() > 0.0 && s.width() > 0.0
                        && s.intersects(gap.shrink(0.01))),
                    "{ballistics:?} {loudness}: {s:?} touches the gap {gap:?}"
                );
                for bar in [left, right] {
                    if s.intersects(bar.shrink(0.01)) {
                        assert!(
                            bar.expand(0.01).contains_rect(*s),
                            "{ballistics:?} {loudness}: {s:?} crosses the edge of {bar:?}"
                        );
                    }
                }
            }
        }
    }
}
```

(`rms_db`, `max_db` and `lufs_momentary` are real `MeterReading` fields; run `grep -n "pub " crates/fp-engine/src/meter.rs | head -30` to confirm the exact names and `LoudnessReadout` variant names, and adjust the two lines that set them.)

- [x] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-app --test meter_view nothing_is_drawn 2>&1 | tail -15` and `cargo test -p fp-app --test theme the_meter_ticks 2>&1 | tail -8`
Expected: compile errors (`tick_colour`, `METER_TICK` not found). After adding only the constants and `tick_colour` (Step 3, first part), `nothing_is_drawn_over_the_bars_or_between_them` FAILS on the digital meter (the reference line across the gap).

- [x] **Step 3: Implement**

`theme.rs`: replace the `METER_LINE_*` block with

```rust
/// Meter rulers (operator feedback 4, Q11): a major tick for every label,
/// a fainter minor tick between them, and the alignment level in white,
/// all outside the bars.
pub const METER_TICK: Color32 = NEUTRAL_400;
pub const METER_TICK_MINOR_ALPHA: f32 = 0.60;
pub const METER_ALIGNMENT_TICK: Color32 = Color32::WHITE;
```

`widgets.rs`: delete `LineShade` (and its `impl`), `reference_segments`, `ALIGNMENT_NOTCH`, and the fields `labels_right`, `lines_x`, `alignment_notches` of `MeterLayout` with the code that fills them (the `notch` closure and `alignment_notches` binding in `meter_layout`). Add:

```rust
/// The colour a ruler tick is painted in (constants in `theme`).
pub fn tick_colour(kind: TickKind) -> Color32 {
    match kind {
        TickKind::Major => theme::METER_TICK,
        TickKind::Minor => theme::METER_TICK.gamma_multiply(theme::METER_TICK_MINOR_ALPHA),
        TickKind::Alignment => theme::METER_ALIGNMENT_TICK,
    }
}
```

In `vu`, replace everything from the comment "The scale: a faint reference line…" through the `for notch in l.alignment_notches` loop with:

```rust
    // The rulers, outside the bars: the labels and a tick for every
    // labelled mark on each side, the minor ticks and the alignment tick.
    // Nothing is drawn over the bars or between them.
    for ruler in &l.rulers {
        for tick in &l.ticks {
            painter.rect_filled(ruler.tick_rect(tick), 0.0, tick_colour(tick.kind));
        }
        for m in l.lines.iter().filter(|m| !m.label.is_empty()) {
            painter.text(
                pos2(ruler.labels_x, m.label_y),
                Align2([ruler.label_halign, m.label_align]),
                &m.label,
                FontId::monospace(LABEL_FONT_SIZE),
                theme::NEUTRAL_400,
            );
        }
    }
```

Update the doc comment of `vu`: "the scale on a ruler on each side of the bars (labels, a tick for every label, minor ticks, a white alignment tick), a continuous bar per channel … the peak hold, … Nothing is drawn over the bars." Remove "reference lines across both bars". Remove the now-unused `level_y` binding only if the compiler says it is unused (it is still used by `column`).

- [x] **Step 4: Run all the affected tests**

Run: `cargo test -p fp-app --test meter_view --test theme 2>&1 | tail -15`
Expected: PASS. Then `grep -rn "reference_segments\|LineShade\|METER_LINE\|alignment_notches\|lines_x\|labels_right" crates docs/technical docs/user` must print nothing except the historical plans and `docs/technical/audio-engine.md` (Task 3).

- [x] **Step 5: Commit**

```bash
if cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings \
   && cargo test --workspace; then
git add crates/fp-app/src/ui/widgets.rs crates/fp-app/src/ui/theme.rs \
        crates/fp-app/tests/meter_view.rs crates/fp-app/tests/theme.rs
git commit -m "fix(ui): draw the meter's scale on two rulers, nothing over the bars

The reference lines across the bars read as a signal fault. The scale is
now a ruler on each side: labels, a tick per label, minor ticks and a
thicker white alignment tick. The bars and the gap between them show only
the level, the peak hold and the zones (operator feedback 4, Q11.1).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
fi
```

---

### Task 3: Documentation (suggested executor: sonnet)

**Files:**
- Modify: `docs/user/players.md:27-40` (the Stereo meter bullet)
- Modify: `docs/technical/ui.md:30` (the `ui/widgets.rs` row)
- Modify: `docs/technical/audio-engine.md:117-122` (the pure layout paragraph)
- Modify: `docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md` (As built under O13)
- Modify: `docs/superpowers/specs/2026-10-06-operator-feedback-4-design.md` (add "As built" to §5)
- Check: `README.md` (the feature list and the roadmap mention the meter; no change expected)
- Locales: no strings change, so `crates/fp-app/locales/en-US/main.ftl` and `es-ES/main.ftl` are not edited (state this in the commit body).

- [ ] **Step 1: Check what is already in the tree**

Run: `grep -n "rulers on both sides\|thicker white tick\|nothing is drawn over the bars" docs/superpowers/specs/2026-09-27-meters-design.md` (expect hits on lines 99, 110, 111) and `grep -n "Superseded by operator feedback 4" docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md` (expect line ~182). If a hit is missing, add the sentence from the Q11 "Spec lines that change" list of the feedback 4 spec to that file. Then `grep -rn "reference line\|notch" docs/user docs/technical README.md` lists every sentence still describing the old meter.

- [ ] **Step 2: Rewrite the user guide**

In `docs/user/players.md`, replace the sentence from "The scale is labelled on the left:" to "mark the alignment level (−18 dBFS)." with:

```markdown
The scale is a ruler on each side of the
    bars, labelled in the meter's own units on both sides. Its top and its
    bottom (for the digital meter, the scale floor) are always marked; every
    label has a tick on each ruler, and shorter ticks between the labels
    work like those of a measuring ruler (every 1 dB on the EBU meter, every
    5 dB below −20 on the digital one). The alignment level (−18 dBFS on the
    digital meter) is a thicker white tick on both rulers. Nothing is drawn
    over the bars or between them, so what you see in the bars is only the
    level, the peak hold and the colours.
```

Refresh the sentence under the heading image if it lists the meter as "meter" only (no change needed otherwise).

- [ ] **Step 3: Rewrite the technical docs**

`docs/technical/ui.md` line 30: replace "The meter's geometry is the pure `meter_layout` (labels, lines, bars, readouts), unit-tested" with "The meter's geometry is the pure `meter_layout` (a `Ruler` per side, the `ticks`, the label lines, the bars and the readouts), unit-tested; nothing is painted over the bars (`tests/meter_view.rs` reads the painted shapes)".

`docs/technical/audio-engine.md`: replace the sentence "…`widgets::reference_segments` says which pieces of a reference line lie over the lit part of a bar. The line colours and opacities are the `METER_LINE_*` constants in `ui/theme.rs`." with "…the scale is a ruler on each side of the bars (`MeterLayout::rulers`): a major tick for every label, minor ticks from `widgets::minor_marks` (a step per scale, kept 3 px apart) and a thicker white alignment tick (`MeterLayout::ticks`, painted with `widgets::tick_colour`). Nothing is drawn over the bars or between them. The tick colours are the `METER_TICK*` and `METER_ALIGNMENT_TICK` constants in `ui/theme.rs`."

- [ ] **Step 4: Spec notes**

In `2026-10-01-operator-feedback-2-design.md`, under the O13 "As built" bullet about `reference_segments`, append: "Removed by operator feedback 4, Q11: `reference_segments`, `LineShade`, the notches and the `METER_LINE_*` constants no longer exist."

In `2026-10-06-operator-feedback-4-design.md`, at the end of §5 (before the `---` at ~L497) add:

```markdown
- **As built.**
  - `meter_layout` returns two `Ruler`s and the `ticks`; `METER_WIDTH` is 78 px (was 52): per side an 18 px label column, a 4 px tick strip and two 1 px gaps.
  - Minor steps: digital and custom 5 dB below −20 dBFS and 1 dB above; EBU 1 dB; DIN 5 dB below −20 relative to its 0, then 1 dB; VU 5 dB below −10, 1 dB to −3, 0.5 dB above; K-System 5 dB below −24, then 1 dB. A scale mark that lost its label is drawn as a minor tick.
  - Ticks: major `NEUTRAL_400`, minor the same at 60 %, alignment white and 2 px thick (`METER_TICK`, `METER_TICK_MINOR_ALPHA`, `METER_ALIGNMENT_TICK`).
```

(If Task 1 needed a ladder rung, add one bullet naming it, with its measured effect.)

- [ ] **Step 5: Verify and commit**

Run: `grep -rn "reference_segments\|METER_LINE\|alignment_notches" docs/user docs/technical crates` (expect nothing).

```bash
if cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings \
   && cargo test --workspace; then
git add docs/user/players.md docs/technical/ui.md docs/technical/audio-engine.md \
        docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md \
        docs/superpowers/specs/2026-10-06-operator-feedback-4-design.md
git commit -m "docs(ui): describe the meter's two rulers

No strings change, so both locales are untouched.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
fi
```

---

### Task 4: Look at it and refresh the screenshots (suggested executor: sonnet)

Checks the look visually in the real app (Xvfb) and refreshes the images that show the meter: the README image and the player image of the guide.

**Files:**
- Modify (regenerated): `docs/images/main-screen.png`, `docs/images/guide/player.png`
- Modify only if the look is wrong: the constants in `crates/fp-app/src/ui/widgets.rs` (`TICK_STRIP`, `MINOR_TICK_LEN`, `RULER_GAP`) and `crates/fp-app/src/ui/theme.rs` (`METER_TICK_MINOR_ALPHA`), with `tests/theme.rs` and `tests/meter_view.rs` updated to match.
- Docs and locales: none beyond the images (no strings).

- [ ] **Step 1: Take the screenshots**

Needs `xvfb`, `xdotool`, ImageMagick, `ffmpeg`, `python3`, `curl` (CLAUDE.md, Testing notes). Run:

```bash
scripts/site/screenshots.sh 2>&1 | tail -30
```

Expected: the script builds the release binary and the demo session, checks in the log that the null backend is in use, and writes `docs/images/main-screen.png` and every `docs/images/guide/*.png`. This takes several minutes; run it with a 10 minute timeout.

- [ ] **Step 2: Look at the meter**

Open `docs/images/guide/player.png` and `docs/images/main-screen.png` with the Read tool (it shows images). Check, for the meter of every player:
1. nothing crosses the two bars or the gap; the bars show only level, peak hold and zones;
2. a ruler on each side, with the same labels on both and a tick for every label;
3. minor ticks are visible but quieter than the labelled ticks, evenly spaced, with no clumps near the top or the floor;
4. the alignment tick is white and thicker than the others, and no label is clipped at the left or right edge of the player column;
5. the countdown and the transport grid still fit beside the meter.

If a point fails, fix the constant named in **Files** (for example `METER_TICK_MINOR_ALPHA` 0.60 to 0.75 when minor ticks vanish on the dark background), update the test that pins it, rerun `cargo test -p fp-app --test theme --test meter_view`, and rerun Step 1 (`scripts/site/screenshots.sh --only main` is enough for the main image; the full run refreshes the guide image).

- [ ] **Step 3: Keep only the images that changed because of the meter**

Run: `git status --short docs/images`. Expected: `main-screen.png` and `guide/player.png` modified. Any other PNG that shows no meter (settings, tag editor, about, track menu, cart wall, CUE window) is regenerated byte-different only by noise: restore each with `git checkout -- <path>`. `docs/images/guide/settings-meters.png` shows the Settings view, not the meter: restore it too unless Settings changed.

- [ ] **Step 4: Commit**

```bash
if cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings \
   && cargo test --workspace; then
git add docs/images/main-screen.png docs/images/guide/player.png
git commit -m "docs(ui): refresh the screenshots for the meter rulers

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
fi
```

If Step 2 changed constants, include `crates/fp-app/src/ui/widgets.rs`, `theme.rs` and the two test files in this commit, with the subject `fix(ui): tune the meter ruler ticks` and the images in the same commit.

---

## Self-Review

- **Spec coverage.** Q11.1: Task 2 (`nothing_is_drawn_over_the_bars_or_between_them`, deletion of the line painter). Q11.2: Task 1 (`the_meter_has_a_ruler_on_each_side_of_the_bars`), Task 2 (painter loop over both rulers). Q11.3: Task 1 (`every_labelled_mark_has_a_tick_on_both_rulers`). Q11.4: Task 1 (`minor_ticks_sit_between_the_marks_at_a_spacing_per_scale`, `ticks_stay_inside_the_rect_and_apart`). Q11.5: Task 1 (`the_alignment_tick_is_thicker_and_unique`), Task 2 (`the_alignment_tick_is_white_on_both_rulers`, `tick_colours_follow_the_kind`). Q11.6: the crowding code is unchanged and its existing tests (`labels_never_overlap_and_the_alignment_line_stays`, `end_labels_stay_inside_the_rect_on_their_line`, `both_ends_stay_labelled_at_every_height`, which run at 64, 136 and 300 px) stay green; the labels are painted on both rulers from the same `lines`. Q11.7: Task 1 (`the_meter_with_its_rulers_fits_the_minimum_player_width` and the two existing 380 px countdown tests). Spec "Tests": `tests/theme.rs` drops `METER_LINE_*` and pins the new colours (Task 2); docs and screenshots (Tasks 3 and 4). Spec "Spec lines that change": checked and completed in Task 3.
- **Placeholder scan.** No TBD or "similar to". Two spots depend on names to confirm in the tree (the fader's accessible name in `main_screen.rs`, the `MeterReading` field names and `LoudnessReadout` variants); each has the exact `grep` that settles it.
- **Type consistency.** `Ruler { side, labels_x, label_halign, ticks }`, `Ruler::tick_rect(&MeterTick) -> Rect`, `MeterTick { y, kind }`, `TickKind::{Minor, Major, Alignment}`, `Side::{Left, Right}`, `minor_marks(&MeterConfig) -> Vec<f32>`, `tick_colour(TickKind) -> Color32` and `LABEL_COLUMN`, `METER_WIDTH` are used with the same names and signatures in Tasks 1 to 3. `METER_TICK`, `METER_TICK_MINOR_ALPHA`, `METER_ALIGNMENT_TICK` match between `theme.rs`, `tick_colour` and `tests/theme.rs`.
- **Review Focus.** All five lines have a named test in the owning task.
