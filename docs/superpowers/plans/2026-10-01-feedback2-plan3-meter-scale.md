# Meter Scale (Feedback 2, Plan 3) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Every meter type always shows and labels both ends of its scale, with each label sitting on its own reference line inside the meter's rect (O11), and the reference lines are easier to see: lighter over the unlit bar, a dark cut over the lit bar (O13).

**Architecture:**
- **Layout is pure.** `widgets::meter_layout` already computes every line and label position without painting. This plan extends it: `scale_marks` adds the digital floor as a mark; `MeterLine` gains `label_align` so a label is centred on its line, or hangs from or rests on it when the rect edge is too close; the two end labels are placed before the alignment label, which gives way to them.
- **Painting only executes.** A new pure function, `widgets::reference_segments`, splits one reference line into one segment per bar (and the gap) and says whether each is `Lit` or `Unlit`. `vu` paints the segments with colours and opacities that are constants in `ui/theme.rs`.
- **Tests.** `tests/meter_view.rs` runs every meter type at 64, 136 and 300 px (with and without the loudness line, and for several digital floors). `tests/theme.rs` pins the constants. A screenshot in Xvfb tunes and confirms the look.

**Tech Stack:** Rust, egui/eframe 0.36.2, egui_kittest 0.36.2. No new dependency and no new UI string (so no locale change).

**Spec:** `docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md` §4 (items O11 and O13). Roadmap: `docs/superpowers/plans/2026-10-01-operator-feedback-2-roadmap.md` (plan 3, branch `fix/meter-scale`). The meter itself is specified in `docs/superpowers/specs/2026-09-27-meters-design.md` (M4).

## Global Constraints

- All code, identifiers, comments, docs and commit messages are in English. Never mention other products.
- No hardcoded product limits: the scale comes from `MeterConfig` (`floor_db` range −96 … −20, `reference_dbfs`, `ballistics`). The two new opacities and two colours are visual constants in `ui/theme.rs`, like the other theme constants, not operator settings.
- The UI never blocks and never panics: no `unwrap`, `expect` or indexing in `src/` (use `get`). Layout must stay finite for any rect height, including 0 (test `a_meter_too_short_to_draw_lays_out_without_panicking` keeps passing).
- Spec §4 O11: "the top of the scale (0 dB or the type's maximum) and the bottom (the floor, such as −60) are always marked and labelled"; "End labels are clamped inside the meter's rect (aligned to the edge, not centred over it), so they are never cut off or shifted away from their line"; "Every label has its reference line"; "Tests in `tests/meter_view.rs` run every meter type at 64, 136 and 300 px. A screenshot in Xvfb confirms the result."
- Spec §4 O13: "Over the unlit part of the bars, lines are drawn in `NEUTRAL_400` at about 60 % opacity (was 35 %). Over the lit part, they are drawn as a dark cut (`NEUTRAL_900`, about 50 %), so they read as segment gaps. Both values are constants in `ui/theme.rs`. They are tuned with a screenshot and pinned by `tests/theme.rs`."
- Commands: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p fp-app --test <file> <name>`. Build only into the repo's `target/` (never set `CARGO_TARGET_DIR`).
- Commit only when fmt, clippy and the whole suite pass (`cargo test --workspace`). Commits end with the co-author trailer the harness provides.
- Documentation is part of the change (Task 4). `CHANGELOG.md` is never edited by hand.

## Review Focus

Failure modes the spec implies and no obvious test covers; each has a test in the task named.

1. A digital or custom meter with a floor that is not a multiple of 10 or 5 (−55, −96, −20): the bottom label must be the floor itself, not the nearest round mark above it. (Task 1, `the_digital_floor_is_always_the_bottom_mark`.)
2. A short meter (64 px) with the loudness line on: the two ends must survive crowding by the alignment label and by each other. (Task 1, `both_ends_stay_labelled_at_every_height`.)
3. The bottom label when there is no loudness line: the bars end at the rect's bottom, so a centred label would be cut off by 5 px. It must rest on its line instead. (Task 2, `end_labels_stay_inside_the_rect_on_their_line`.)
4. A level at the floor (silence, −120 dB) or above the top (an over): the line segments must be all unlit at the floor and all lit at the top, never a panic or a NaN. (Task 3, `reference_segments_are_unlit_at_the_floor_and_lit_at_the_top`.)
5. K-System meters, where the bar has a dimmed peak part above the solid RMS part: the cut follows the peak (outer) level, so a line inside the dimmed part is cut dark too. (Task 3, `a_reference_line_is_cut_dark_over_the_lit_bar_and_light_over_the_rest`, which states the rule; Task 3 step 7 paints it.)

## Decisions

- **Label anchoring.** The spec says end labels are "aligned to the edge, not centred over it". The code today clamps a label's centre into the bars' span, which moves the top and bottom labels about 4.5 px off their lines. This plan instead keeps every label centred on its line and falls back to `Align::Min` (label hangs below the line) or `Align::Max` (label rests on the line) only when a centred label would cross the rect's top or bottom. With no loudness line the bottom end always rests on its line (`Align::Max`).
- **The floor is a mark.** `scale_marks` for digital peak and custom prepends `floor_db` when no round mark sits within `SAME_MARK_DB` of it, so the bottom end is the floor (−55 reads `-55`). A floor 1–4 dB above a round mark simply loses that round mark's label to the crowding rule, which keeps the ends.
- **Ends beat the alignment label.** When the alignment label would touch an end label, the alignment label is dropped (its heavier line and notches stay). The existing rule that the alignment line always exists is unchanged.
- **What "lit" means.** A line segment is lit when the bar's level (the peak, outer level on K-System meters) is at or above the line. The gap between the two bars is never lit.
- **No new locale strings.** This plan adds no UI text.

## File Structure

- Modify `crates/fp-app/src/ui/widgets.rs`: `scale_marks` (floor mark), `MeterLine` (`label_align`, `label_centre`), `meter_layout` (placement and ends first), `LineShade` and `reference_segments`, the painting in `vu`; remove `REFERENCE_LINE_ALPHA`.
- Modify `crates/fp-app/src/ui/theme.rs`: four constants.
- Modify `crates/fp-app/tests/meter_view.rs`, `crates/fp-app/tests/theme.rs`.
- Modify `docs/user/players.md`, `docs/technical/audio-engine.md`, the spec (status line and a note under §4).

---

### Task 1: Both ends of every scale are marked and labelled (O11, part 1)

**Files:**
- Modify: `crates/fp-app/src/ui/widgets.rs` (`scale_marks` near line 357, `meter_layout` near line 642)
- Test: `crates/fp-app/tests/meter_view.rs`

**Interfaces:**
- Consumes: `scale_marks(&MeterConfig) -> Vec<f32>`, `meter_layout(Rect, &MeterConfig, bool) -> MeterLayout`, `mark_label`, the test helpers `column(height)`, `meter(ballistics)`, `ALL_METERS`.
- Produces: `scale_marks` whose first element is `floor_db` for `DigitalPeak` and `Custom`; a `meter_layout` where both end labels are always present and the alignment label gives way to them.

- [ ] **Step 1: Write the failing tests**

Add to `crates/fp-app/tests/meter_view.rs`, after `both_ends_of_every_scale_are_labelled`:

```rust
#[test]
fn the_digital_floor_is_always_the_bottom_mark() {
    for ballistics in [MeterBallistics::DigitalPeak, MeterBallistics::Custom] {
        for floor in [-96.0, -90.0, -55.0, -52.0, -45.0, -20.0] {
            let c = MeterConfig {
                ballistics,
                floor_db: floor,
                reference_dbfs: floor + 10.0,
                ..MeterConfig::default()
            };
            let marks = scale_marks(&c);
            assert_eq!(marks.first(), Some(&floor), "{ballistics:?} {floor}");
            assert_eq!(marks.last(), Some(&0.0));
            assert!(
                marks.windows(2).all(|w| w[0] < w[1]),
                "strictly increasing: {marks:?}"
            );
        }
    }
}

/// Spec O11: every meter type at 64, 136 and 300 px, with and without the
/// loudness line, and for several digital floors.
#[test]
fn both_ends_stay_labelled_at_every_height() {
    for ballistics in ALL_METERS {
        for floor in [-96.0, -60.0, -55.0, -20.0] {
            for height in [64.0, 136.0, 300.0] {
                for loudness in [false, true] {
                    let c = MeterConfig {
                        ballistics,
                        floor_db: floor,
                        reference_dbfs: if floor > -30.0 { floor + 5.0 } else { -18.0 },
                        ..MeterConfig::default()
                    };
                    let marks = scale_marks(&c);
                    let l = meter_layout(column(height), &c, loudness);
                    let labelled: Vec<&str> = l
                        .lines
                        .iter()
                        .filter(|m| !m.label.is_empty())
                        .map(|m| m.label.as_str())
                        .collect();
                    for end in [marks.first().unwrap(), marks.last().unwrap()] {
                        let want = mark_label(*end, &c);
                        assert!(
                            labelled.contains(&want.as_str()),
                            "{ballistics:?} floor {floor} {height}px loudness {loudness}: \
                             {want} in {labelled:?}"
                        );
                    }
                    // Every label has its line, and the lines are on the bars.
                    for m in l.lines.iter().filter(|m| !m.label.is_empty()) {
                        assert!(m.y >= l.bars[0].top() && m.y <= l.bars[0].bottom());
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 2: Run them to verify they fail**

Run: `cargo test -p fp-app --test meter_view the_digital_floor_is_always_the_bottom_mark both_ends_stay_labelled_at_every_height`
(cargo takes one filter; run each: `cargo test -p fp-app --test meter_view the_digital_floor` then `cargo test -p fp-app --test meter_view both_ends_stay`.)
Expected: FAIL. The floor test fails for −55 (first mark −50), the other for the same floor and for crowded 64 px cases.

- [ ] **Step 3: Implement**

In `widgets.rs`, replace the digital arm of `scale_marks`:

```rust
        MeterBallistics::DigitalPeak | MeterBallistics::Custom => {
            // The floor is the bottom of the scale, so it is always a mark;
            // the round marks above it stay.
            let mut marks: Vec<f32> = [
                -60.0, -50.0, -40.0, -35.0, -30.0, -25.0, -20.0, -15.0, -10.0, -5.0, 0.0,
            ]
            .into_iter()
            .filter(|m| *m > c.floor_db + SAME_MARK_DB)
            .collect();
            marks.insert(0, c.floor_db);
            marks
        }
```

In `meter_layout`, replace the block from `let mut lines: Vec<MeterLine> = Vec::new();` down to (not including) `let notch = |x: f32| {` with a version that places both ends first against each other only, then lets the alignment label give way to them, then fills the middle:

```rust
    let mut lines: Vec<MeterLine> = Vec::new();
    let crowds = |kept: &MeterLine, candidate: &MeterLine| {
        !kept.label.is_empty()
            && !candidate.label.is_empty()
            && (kept.label_y - candidate.label_y).abs() < LABEL_ROW
    };
    // Both ends of the scale first, so its range is always labelled.
    let ends = [marks.last().copied(), marks.first().copied()];
    for mark in ends.into_iter().flatten() {
        if (mark - alignment_dbfs(c)).abs() < SAME_MARK_DB {
            continue;
        }
        let candidate = line(mark, false);
        if !lines.iter().any(|kept| crowds(kept, &candidate)) {
            lines.push(candidate);
        }
    }
    // The alignment label gives way to an end label; its line stays.
    if lines.iter().any(|kept| crowds(kept, &alignment)) {
        alignment.label.clear();
    }
    // Then top down, so where the scale is dense the upper marks are kept.
    let middle = marks
        .iter()
        .rev()
        .skip(1)
        .take(marks.len().saturating_sub(2));
    for mark in middle.copied() {
        if (mark - alignment_dbfs(c)).abs() < SAME_MARK_DB {
            continue;
        }
        let candidate = line(mark, false);
        let crowded = lines
            .iter()
            .chain(std::iter::once(&alignment))
            .any(|kept| crowds(kept, &candidate));
        if !crowded {
            lines.push(candidate);
        }
    }
```

(The `label_y` field is still the label's centre in this task; Task 2 changes the placement and the comparison.)

- [ ] **Step 4: Run the tests**

Run: `cargo test -p fp-app --test meter_view`
Expected: all PASS, including the existing `labels_never_overlap_and_the_alignment_line_stays` and `both_ends_of_every_scale_are_labelled`.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy -p fp-app --all-targets -- -D warnings
git add crates/fp-app/src/ui/widgets.rs crates/fp-app/tests/meter_view.rs
git commit -m "fix(ui): always mark and label both ends of the meter scale

The digital scale's bottom mark was the nearest round level above the
floor (−50 for a −55 floor), and the alignment label could crowd out an
end label. The floor is now a mark, and the ends are placed before the
alignment label, which gives way to them.

Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>"
```

---

### Task 2: Labels sit on their lines, clamped inside the rect (O11, part 2)

**Files:**
- Modify: `crates/fp-app/src/ui/widgets.rs` (`MeterLine`, `meter_layout`, the label painting in `vu`)
- Test: `crates/fp-app/tests/meter_view.rs`

**Interfaces:**
- Consumes: Task 1's `meter_layout`.
- Produces: `MeterLine { y, label_y, label_align: egui::Align, label, alignment }` and `MeterLine::label_centre(&self) -> f32`. `label_y` is the anchor: the line's `y`. `label_align` is `Center` (label centred on `y`), `Min` (label's top edge at `y`) or `Max` (label's bottom edge at `y`). Task 3 does not depend on these.

- [ ] **Step 1: Write the failing test and update the old checks**

Add to `tests/meter_view.rs`:

```rust
#[test]
fn end_labels_stay_inside_the_rect_on_their_line() {
    for ballistics in ALL_METERS {
        for height in [64.0, 136.0, 300.0] {
            for loudness in [false, true] {
                let c = meter(ballistics);
                let rect = column(height);
                let l = meter_layout(rect, &c, loudness);
                for m in l.lines.iter().filter(|m| !m.label.is_empty()) {
                    let centre = m.label_centre();
                    assert!(
                        centre - LABEL_ROW / 2.0 >= rect.top() - 0.01
                            && centre + LABEL_ROW / 2.0 <= rect.bottom() + 0.01,
                        "{ballistics:?} {height} {loudness}: {} outside the rect",
                        m.label
                    );
                    // Never shifted off its line: the line is within the
                    // label's own height.
                    assert!(
                        (centre - m.y).abs() <= LABEL_ROW / 2.0 + 0.01,
                        "{ballistics:?} {height}: {} is off its line",
                        m.label
                    );
                    assert_eq!(m.label_y, m.y, "the anchor is the line");
                }
            }
        }
    }
    // Without the loudness line the bars end at the rect's bottom, so the
    // bottom label rests on its line instead of being centred over it.
    let c = meter(MeterBallistics::DigitalPeak);
    let l = meter_layout(column(136.0), &c, false);
    let bottom = l.lines.first().unwrap();
    assert_eq!(bottom.label, "-60");
    assert_eq!(bottom.label_align, egui::Align::Max);
    // With room above and below, a label is centred on its line.
    let middle = l.lines.iter().find(|m| m.label == "-30").unwrap();
    assert_eq!(middle.label_align, egui::Align::Center);
}
```

Order note: `meter_layout` sorts lines by `label_centre()`, so the lowest label (−60, bottom of the screen) is the last element, not the first. Use `l.lines.last()` for the bottom, and `first()` for the top (`"0"`). Fix that in the test as you write it: replace `l.lines.first().unwrap()` with `l.lines.iter().find(|m| m.label == "-60").unwrap()`.

Update the two existing checks that read `label_y` as a centre:
- In `labels_never_overlap_and_the_alignment_line_stays`, change `.map(|m| m.label_y)` to `.map(|m| m.label_centre())`.
- In `labels_stay_between_the_readouts`, replace the body of the `for m in &l.lines` loop with the rect check (labels are inside the meter's rect, not only the bars' span):

```rust
        let rect = column(136.0);
        for m in &l.lines {
            assert!(
                m.label_centre() - LABEL_ROW / 2.0 >= rect.top() - 0.01,
                "{}",
                m.label
            );
            assert!(
                m.label_centre() + LABEL_ROW / 2.0 <= rect.bottom() + 0.01,
                "{}",
                m.label
            );
        }
```

and in `a_meter_too_short_to_draw_lays_out_without_panicking` also assert `m.label_centre().is_finite()`.

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p fp-app --test meter_view end_labels_stay`
Expected: FAIL to compile (`label_centre`, `label_align` missing).

- [ ] **Step 3: Implement**

In `widgets.rs`, replace `MeterLine`:

```rust
/// One labelled mark of the meter's scale (see [`meter_layout`]).
#[derive(Debug, Clone, PartialEq)]
pub struct MeterLine {
    pub y: f32,
    /// Where the label is anchored vertically: the line's `y`.
    pub label_y: f32,
    /// How the label hangs from `label_y`: centred on it, or its top
    /// (`Min`) or bottom (`Max`) edge on it where the rect's edge is too
    /// close for a centred label.
    pub label_align: egui::Align,
    /// Empty for an alignment level the scale does not name.
    pub label: String,
    pub alignment: bool,
}

impl MeterLine {
    /// The vertical centre of the label's row.
    pub fn label_centre(&self) -> f32 {
        let half = LABEL_ROW / 2.0;
        match self.label_align {
            egui::Align::Min => self.label_y + half,
            egui::Align::Center => self.label_y,
            egui::Align::Max => self.label_y - half,
        }
    }
}
```

In `meter_layout`, delete the `label_y` closure (the one with "The label is centred on its line but kept inside the bars' span") and change `line` to:

```rust
    let line = |db: f32, alignment: bool| {
        let width = if alignment { ALIGNMENT_LINE_WIDTH } else { 1.0 };
        let y = y_of(db).clamp(
            top + width / 2.0,
            (bottom - width / 2.0).max(top + width / 2.0),
        );
        // Centred on the line, unless that would cross the rect's edge:
        // then the label hangs from or rests on the line, inside the rect.
        let half = LABEL_ROW / 2.0;
        let label_align = if y - half < rect.top() {
            egui::Align::Min
        } else if y + half > rect.bottom() {
            egui::Align::Max
        } else {
            egui::Align::Center
        };
        MeterLine {
            y,
            label_y: y,
            label_align,
            label: mark_label(db, c),
            alignment,
        }
    };
```

Change `crowds` to compare centres: `(kept.label_centre() - candidate.label_centre()).abs() < LABEL_ROW`. Change the sort to `lines.sort_by(|a, b| a.label_centre().total_cmp(&b.label_centre()));`.

In `vu`, replace the label painting (`Align2::RIGHT_CENTER` call) with:

```rust
        painter.text(
            pos2(l.labels_right, m.label_y),
            Align2([egui::Align::RIGHT, m.label_align]),
            &m.label,
            FontId::monospace(LABEL_FONT_SIZE),
            theme::NEUTRAL_400,
        );
```

(Leave the reference-line painting as is; Task 3 rewrites it.)

- [ ] **Step 4: Run the tests**

Run: `cargo test -p fp-app --test meter_view`
Expected: all PASS. If a pre-existing assertion about sorted label order fails, the sort key is the only thing to check.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy -p fp-app --all-targets -- -D warnings
git add crates/fp-app/src/ui/widgets.rs crates/fp-app/tests/meter_view.rs
git commit -m "fix(ui): keep meter labels on their lines and inside the meter

Labels were centred on their line but clamped into the bars' span, so
the end labels sat about 5 px away from their lines. A label is now
centred on its line, and rests on or hangs from it only where the
meter's rect edge is too close.

Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>"
```

---

### Task 3: Reference lines: lighter over the unlit bar, a dark cut over the lit bar (O13)

**Files:**
- Modify: `crates/fp-app/src/ui/theme.rs` (constants after `METER_DANGER`)
- Modify: `crates/fp-app/src/ui/widgets.rs` (`LineShade`, `reference_segments`, `vu`, remove `REFERENCE_LINE_ALPHA`)
- Test: `crates/fp-app/tests/theme.rs`, `crates/fp-app/tests/meter_view.rs`

**Interfaces:**
- Consumes: `MeterLayout` (`bars`, `lines`) from Task 2; `theme::NEUTRAL_400`, `theme::NEUTRAL_900`.
- Produces: `theme::METER_LINE_UNLIT`, `METER_LINE_UNLIT_ALPHA`, `METER_LINE_LIT`, `METER_LINE_LIT_ALPHA`; `widgets::LineShade { Lit, Unlit }`; `widgets::reference_segments(l: &MeterLayout, y: f32, level_y: [f32; 2]) -> [(Rect, LineShade); 3]` (bar 0, the gap, bar 1; `level_y` are the screen y of each channel's level).

- [ ] **Step 1: Write the failing tests**

Add to `tests/theme.rs`:

```rust
#[test]
fn the_meter_reference_lines_are_visible_over_both_parts_of_the_bar() {
    // Unlit part: NEUTRAL_400 at about 60 % (was 35 %); lit part: a dark
    // cut, NEUTRAL_900 at about 50 % (operator feedback 2, O13).
    assert_eq!(theme::METER_LINE_UNLIT, theme::NEUTRAL_400);
    assert_eq!(theme::METER_LINE_UNLIT_ALPHA, 0.60);
    assert_eq!(theme::METER_LINE_LIT, theme::NEUTRAL_900);
    assert_eq!(theme::METER_LINE_LIT_ALPHA, 0.50);
}
```

Add to `tests/meter_view.rs` (extend the `use fp_app::ui::widgets::{…}` list with `LineShade, reference_segments`):

```rust
#[test]
fn a_reference_line_is_cut_dark_over_the_lit_bar_and_light_over_the_rest() {
    let l = meter_layout(column(136.0), &meter(MeterBallistics::DigitalPeak), false);
    let y = 100.0;
    // Left level above the line (lit there), right level below it.
    let [left, gap, right] = reference_segments(&l, y, [80.0, 120.0]);
    assert_eq!(left.1, LineShade::Lit);
    assert_eq!(right.1, LineShade::Unlit);
    assert_eq!(gap.1, LineShade::Unlit, "the gap between the bars is never lit");
    assert_eq!(left.0.x_range(), l.bars[0].x_range());
    assert_eq!(right.0.x_range(), l.bars[1].x_range());
    assert!(gap.0.left() >= l.bars[0].right() && gap.0.right() <= l.bars[1].left());
    for (r, _) in [left, gap, right] {
        assert!((r.center().y - y).abs() < 1e-3 && (r.height() - 1.0).abs() < 1e-3);
    }
}

#[test]
fn reference_segments_are_unlit_at_the_floor_and_lit_at_the_top() {
    let c = meter(MeterBallistics::DigitalPeak);
    let l = meter_layout(column(136.0), &c, false);
    let (top, bottom) = (l.bars[0].top(), l.bars[0].bottom());
    for m in &l.lines {
        // Silence: the level sits at the bottom of the bars.
        let at_floor = reference_segments(&l, m.y, [bottom, bottom]);
        assert_eq!(at_floor[0].1, LineShade::Unlit, "{}", m.label);
        assert_eq!(at_floor[2].1, LineShade::Unlit, "{}", m.label);
        // An over: the level is at (or past) the top.
        let at_top = reference_segments(&l, m.y, [top, top - 50.0]);
        assert_eq!(at_top[0].1, LineShade::Lit, "{}", m.label);
        assert_eq!(at_top[2].1, LineShade::Lit, "{}", m.label);
    }
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p fp-app --test theme the_meter_reference_lines` then `cargo test -p fp-app --test meter_view reference`
Expected: FAIL to compile (constants and function missing).

- [ ] **Step 3: Add the constants**

In `theme.rs`, after `METER_DANGER`:

```rust
/// Meter reference lines (operator feedback 2, O13): light over the unlit
/// part of a bar, a dark cut (a segment gap) over the lit part.
pub const METER_LINE_UNLIT: Color32 = NEUTRAL_400;
pub const METER_LINE_UNLIT_ALPHA: f32 = 0.60;
pub const METER_LINE_LIT: Color32 = NEUTRAL_900;
pub const METER_LINE_LIT_ALPHA: f32 = 0.50;
```

- [ ] **Step 4: Implement the pure function**

In `widgets.rs`, delete `REFERENCE_LINE_ALPHA` and its doc comment, and add near `MeterLayout`:

```rust
/// Whether a piece of a reference line lies over the lit part of a bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineShade {
    Lit,
    Unlit,
}

impl LineShade {
    /// The colour the piece is painted in (constants in `theme`).
    pub fn colour(self) -> Color32 {
        match self {
            LineShade::Lit => theme::METER_LINE_LIT.gamma_multiply(theme::METER_LINE_LIT_ALPHA),
            LineShade::Unlit => {
                theme::METER_LINE_UNLIT.gamma_multiply(theme::METER_LINE_UNLIT_ALPHA)
            }
        }
    }
}

/// The reference line at screen height `y` as three pieces: over the first
/// bar, over the gap between the bars and over the second bar. A piece over
/// a bar is lit when that bar's level (`level_y`, screen height of each
/// channel's level; the peak on K-System meters) is at or above the line;
/// the gap is never lit.
pub fn reference_segments(
    l: &MeterLayout,
    y: f32,
    level_y: [f32; 2],
) -> [(Rect, LineShade); 3] {
    let [first, second] = l.bars;
    let piece = |x: egui::Rangef| Rect::from_x_y_ranges(x, y - 0.5..=y + 0.5);
    let shade = |ch: usize| {
        if level_y.get(ch).is_some_and(|ly| *ly <= y) {
            LineShade::Lit
        } else {
            LineShade::Unlit
        }
    };
    [
        (piece(first.x_range()), shade(0)),
        (
            piece(egui::Rangef::new(first.right(), second.left())),
            LineShade::Unlit,
        ),
        (piece(second.x_range()), shade(1)),
    ]
}
```

- [ ] **Step 5: Run the pure tests**

Run: `cargo test -p fp-app --test theme the_meter_reference_lines` then `cargo test -p fp-app --test meter_view reference`
Expected: PASS (the file may fail to compile until step 6 removes the old constant's use; do step 6 first if so).

- [ ] **Step 6: Paint with the segments**

In `vu`, the per-channel loop already computes `level` for each bar. Keep a record of both levels' screen heights before the scale loop:

```rust
    let level_y = [0usize, 1].map(|ch| y_of(reading.level_db.get(ch).copied().unwrap_or(-120.0)));
```

(place it right after `y_of` is defined, before the `for (ch, bar)` loop). Replace the reference-line painting inside `for m in &l.lines` with:

```rust
    for m in &l.lines {
        for (piece, shade) in reference_segments(&l, m.y, level_y) {
            painter.rect_filled(piece, 0.0, shade.colour());
        }
        painter.text(
            pos2(l.labels_right, m.label_y),
            Align2([egui::Align::RIGHT, m.label_align]),
            &m.label,
            FontId::monospace(LABEL_FONT_SIZE),
            theme::NEUTRAL_400,
        );
    }
```

K-System meters keep working: `level` there is the peak (outer) level, so the cut covers the dimmed peak part too, as the Decisions state.

- [ ] **Step 7: Run everything for the crate**

Run: `cargo test -p fp-app --test meter_view` and `cargo test -p fp-app --test theme` and `cargo test -p fp-app --test main_screen`
Expected: PASS.

- [ ] **Step 8: Commit**

```bash
cargo fmt --all && cargo clippy -p fp-app --all-targets -- -D warnings
git add crates/fp-app/src/ui/widgets.rs crates/fp-app/src/ui/theme.rs crates/fp-app/tests/meter_view.rs crates/fp-app/tests/theme.rs
git commit -m "fix(ui): make the meter reference lines easier to see

The lines were 35 % NEUTRAL_400 over the whole bar, nearly invisible on
the lit part. Over the unlit part they are now 60 %, and over the lit
part a dark cut (NEUTRAL_900, 50 %) that reads as a segment gap. The
colours and opacities are theme constants.

Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>"
```

---

### Task 4: Screenshot check, tuning and documentation

**Files:**
- Modify (only if the screenshot calls for it): `crates/fp-app/src/ui/theme.rs` and `crates/fp-app/tests/theme.rs` (the two alphas)
- Modify: `docs/user/players.md` (the "Stereo meter" bullets), `docs/technical/audio-engine.md` (the "interface only draws" bullet), `docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md`

**Interfaces:** Consumes everything above; produces nothing new.

- [ ] **Step 1: Build and capture each meter type in Xvfb**

Follow the CLAUDE.md screenshot recipe (`xvfb`, `xdotool` and ImageMagick `import` must be installed):

```sh
cargo build --release -p fp-app
Xvfb :77 -screen 0 1920x1080x24 -nolisten tcp &
mkdir -p /tmp/fp-meter   # scratch state only; builds still go to target/
```

Seed `/tmp/fp-meter/config.json` by running the app once, then set `"meter": {"ballistics": "<type>"}` inside `"config"` for `DigitalPeak`, `EbuPpm`, `DinPpm`, `Vu`, `K20`, and a digital run with `"floor_db": -55`. For each: start with `env -u WAYLAND_DISPLAY DISPLAY=:77 FAUSTE_HOME=/tmp/fp-meter target/release/fauste-player &`, play tones through `FAUSTE_HOME=/tmp/fp-meter cargo run -p fp-app --example demo_session -- <music dir>` plus the remote API (`docs/user/remote-control.md`) so the bars are part lit, find the window with `DISPLAY=:77 xwininfo -name "Fauste Player"`, and capture with `DISPLAY=:77 import -window <id> shot.png`. Also capture a window taller and shorter than default.

- [ ] **Step 2: Check the screenshots against the spec**

Confirm, for every type: the top and the bottom of the scale carry a label next to their line, no label is cut off, every label has a visible line, the lines read over the unlit part and show as dark cuts over the lit part, and the K-20 dimmed peak part is cut dark too. If the lines are too faint or too heavy, change `METER_LINE_UNLIT_ALPHA` / `METER_LINE_LIT_ALPHA` in `theme.rs` to the tuned values and update the matching asserts in `tests/theme.rs` in the same commit. Stop the app and the Xvfb server when done (`kill %1 %2` or by pid).

- [ ] **Step 3: Update the user guide**

In `docs/user/players.md`, replace the sentence "The scale is labelled on the left, with a faint line across both bars for each label;" with:

```markdown
The scale is labelled on the left: its top and its bottom (for the digital
meter, the scale floor) are always marked, and every label has a line across
both bars. The lines are light over the empty part of a bar and show as dark
cuts over the lit part, so they stay readable at any level;
```

(keep the following words about the alignment notches).

- [ ] **Step 4: Update the technical docs**

In `docs/technical/audio-engine.md`, extend the "The interface only draws" bullet with:

```markdown
  The layout is pure (`widgets::meter_layout`): both ends of the scale are
  always labelled (`scale_marks` includes the digital floor; the alignment
  label gives way to an end label), each label is centred on its line or, at
  the rect's edge, rests on or hangs from it (`MeterLine::label_align`), and
  `widgets::reference_segments` says which pieces of a reference line lie
  over the lit part of a bar. The line colours and opacities are the
  `METER_LINE_*` constants in `ui/theme.rs`.
```

- [ ] **Step 5: Update the spec**

In `docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md`, under "## 4. Plan 3 — Meter scale", add at the end of the section:

```markdown
- **As built.**
  - The digital scale's floor is a mark of its own (`scale_marks`), so a −55
    floor is labelled `-55`.
  - A label is centred on its line; only where a centred label would cross
    the meter's rect does it rest on the line (bottom) or hang from it (top).
  - The alignment label gives way to an end label; its heavier line and
    notches stay.
  - A reference-line piece is lit when the bar's level (the peak on K-System
    meters) is at or above the line; the gap between the bars is never lit.
  - Constants: `METER_LINE_UNLIT` / `_ALPHA` (`NEUTRAL_400`, 0.60) and
    `METER_LINE_LIT` / `_ALPHA` (`NEUTRAL_900`, 0.50), pinned by
    `tests/theme.rs`.
```

(If Step 2 tuned the alphas, write the tuned values.) Do not touch the roadmap table; the coordinator updates it. `CHANGELOG.md` and `AGENTS.md` are not edited.

- [ ] **Step 6: Run the whole gate**

Run:

```sh
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
```

Expected: all green.

- [ ] **Step 7: Commit**

```bash
git add docs/user/players.md docs/technical/audio-engine.md docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md crates/fp-app/src/ui/theme.rs crates/fp-app/tests/theme.rs
git commit -m "docs(ui): describe the meter scale marks and reference lines

Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>"
```

(If the alphas did not change, `theme.rs` and `tests/theme.rs` are simply not staged.)

---

## Self-review

- **O11:** ends always marked and labelled for every type (Task 1, floor mark and ends first), clamped inside the rect and on their line (Task 2), every label has its line (Task 1 test, existing painting), tests at 64/136/300 px for every type (Tasks 1 and 2), screenshot (Task 4).
- **O13:** `NEUTRAL_400` at 60 % over the unlit part, `NEUTRAL_900` at 50 % over the lit part, constants in `theme.rs`, pinned in `tests/theme.rs`, tuned by screenshot (Task 3 and Task 4).
- **Types:** `MeterLine::label_centre`, `label_align`, `LineShade`, `reference_segments(&MeterLayout, f32, [f32; 2]) -> [(Rect, LineShade); 3]` are named the same in every task.
- **Locales:** no new UI strings, so neither `main.ftl` changes.
