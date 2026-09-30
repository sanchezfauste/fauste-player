# Feedback Plan 2 — Meter Column Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move the level meter and the volume fader into a column at the right
of the player, spanning the info and transport rows, with a dB scale, reference
lines and muted colours (F7, F9, F11).

**Architecture:** All in `fp-app`.
- A pure `widgets::meter_layout` computes where the labels, lines, bars, max
  readout and loudness line go for a given rectangle. It replaces
  `mark_rows`.
- `widgets::vu` draws from it at any height, and `widgets::fader` takes a
  height.
- `player.rs` splits the top block into a left part (info row and
  transport) and a right column (meter and fader). `elapsed / total` moves
  to its own row under the block.

**Tech Stack:** Rust 2024, egui 0.36, egui_kittest.

**Spec:** [`docs/superpowers/specs/2026-09-30-operator-feedback-design.md`](../specs/2026-09-30-operator-feedback-design.md) §3.2.
Roadmap: [`2026-09-30-operator-feedback-roadmap.md`](2026-09-30-operator-feedback-roadmap.md), plan 2.

## Global Constraints

- English code and docs; UI strings in both locales (this plan adds none).
- No `unwrap`/`expect`/`panic` outside tests; prefer `get`.
- Commit only with `cargo fmt --all --check`, `cargo clippy --workspace
  --all-targets -- -D warnings` and `cargo test --workspace` green.
- Conventional Commits (scope `ui`, `docs`), body says why, trailer
  `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Branch `feat/meter-column` from an up-to-date `master`.
- Colours, exactly: normal `#7fb08a`, warning `#d9b45a`, danger `#d8646a`.
  Reference lines `NEUTRAL_400` at 35 % (`gamma_multiply(0.35)`). Labels
  monospace, `NEUTRAL_400`.

## Review Focus

- **Short meters.** At the 64 px of a meter drawn elsewhere, or with the
  loudness line on, labels must still not overlap and the alignment line
  must stay (Task 2 tests 64 px and 136 px, with and without loudness).
- **Top and bottom marks.** The 0 dBFS mark sits on the bars' top edge; its
  label must not rise into the max readout, nor the bottom label fall into
  the loudness line (Task 2 test).
- **Every meter type.** EBU, DIN, VU and K-System scales label their own
  units (EBU ±12, DIN −50…+5, VU −20…+3, K relative to 0 = K reference)
  (Task 2 test).
- **Minimum player width (380 px).** The column must not squeeze the
  countdown out of the transport row (Task 4 test).
- **The max readout keeps its click** after the move (Task 3 keeps
  `clicking_the_maximum_restarts_it` green).

---

### Task 1: Muted meter colours (F9)

**Files:**
- Modify: `crates/fp-app/src/ui/theme.rs:32-34`
- Modify: `crates/fp-app/src/ui/widgets.rs` (`zone_colour`, max readout, loudness line)
- Test: `crates/fp-app/tests/theme.rs`

**Interfaces:**
- Produces: `theme::METER_NORMAL`, `theme::METER_WARNING`, `theme::METER_DANGER` (`Color32`). `VU_GREEN`, `VU_YELLOW`, `VU_RED` are removed.

- [ ] **Step 1: Write the failing test** — append to `crates/fp-app/tests/theme.rs`:

```rust
#[test]
fn meter_colours_are_the_muted_traffic_light() {
    assert_eq!(theme::METER_NORMAL, Color32::from_rgb(0x7f, 0xb0, 0x8a));
    assert_eq!(theme::METER_WARNING, Color32::from_rgb(0xd9, 0xb4, 0x5a));
    assert_eq!(theme::METER_DANGER, Color32::from_rgb(0xd8, 0x64, 0x6a));
}
```

- [ ] **Step 2: Run it** — `cargo test -p fp-app --test theme meter_colours`. Expected: compile error, no `METER_NORMAL`.

- [ ] **Step 3: Implement** — in `theme.rs` replace the three `VU_*` lines with:

```rust
/// Meter zones: muted traffic-light tones that sit with the Nocturne
/// neutrals (feedback spec F9).
pub const METER_NORMAL: Color32 = Color32::from_rgb(0x7f, 0xb0, 0x8a);
pub const METER_WARNING: Color32 = Color32::from_rgb(0xd9, 0xb4, 0x5a);
pub const METER_DANGER: Color32 = Color32::from_rgb(0xd8, 0x64, 0x6a);
```

In `widgets.rs`, replace every `theme::VU_GREEN` with `theme::METER_NORMAL`,
`theme::VU_YELLOW` with `theme::METER_WARNING` and `theme::VU_RED` with
`theme::METER_DANGER` (zone colours, the max readout in danger, the loudness
line on target). `grep -rn "VU_" crates` must then return nothing.

- [ ] **Step 4: Run** `cargo test -p fp-app --test theme`. Expected: PASS.

- [ ] **Step 5: Commit** — `feat(ui): give the meter muted zone colours` (body: the saturated green, yellow and red clashed with the Nocturne palette).

---

### Task 2: Meter geometry as a pure function (F7, F11)

**Files:**
- Modify: `crates/fp-app/src/ui/widgets.rs` (new `MeterLayout`, `MeterLine`, `meter_layout`, `mark_label`; remove `mark_rows` and `MIN_MARK_GAP`)
- Test: `crates/fp-app/tests/meter_view.rs` (replace the two `mark_rows` tests)

**Interfaces:**
- Produces:

```rust
pub struct MeterLine {
    /// Screen y of the line (the mark's true position on the scale).
    pub y: f32,
    /// Screen y the label is centred on (kept inside the bars' span).
    pub label_y: f32,
    pub label: String,
    /// The alignment level: a 2 px line instead of 1 px.
    pub alignment: bool,
}
pub struct MeterLayout {
    /// Right edge of the label column (labels are right-aligned to it).
    pub labels_right: f32,
    /// The left (L) and right (R) bars.
    pub bars: [Rect; 2],
    /// Where the maximum readout is drawn and clicked.
    pub max: Rect,
    /// The loudness line, when it is on.
    pub loudness: Option<Rect>,
    /// Horizontal span of every reference line: across both bars.
    pub lines_x: egui::Rangef,
    /// Top to bottom; the alignment line is always present.
    pub lines: Vec<MeterLine>,
}
pub const METER_WIDTH: f32; // 50.0: 18 label + 2 + 14 bar + 2 gap + 14 bar
pub fn mark_label(db: f32, c: &MeterConfig) -> String;
pub fn meter_layout(rect: Rect, c: &MeterConfig, loudness: bool) -> MeterLayout;
```

- [ ] **Step 1: Write the failing tests** — in `crates/fp-app/tests/meter_view.rs`, replace the `use fp_app::ui::widgets::{…}` list with

```rust
use fp_app::ui::widgets::{
    METER_WIDTH, Zone, alignment_dbfs, loudness_line, mark_label, max_readout, meter_layout,
    meter_position, scale_marks, zone_of,
};
```

delete `const BARS`, `marks_stay_inside_the_bars` and
`marks_too_close_to_read_are_left_out`, and add:

```rust
/// Label rows are this far apart at least (monospace 9 px text).
const LABEL_ROW: f32 = 10.0;

fn column(height: f32) -> egui::Rect {
    egui::Rect::from_min_size(egui::pos2(100.0, 20.0), egui::vec2(METER_WIDTH, height))
}

#[test]
fn labels_never_overlap_and_the_alignment_line_stays() {
    for ballistics in ALL_METERS {
        for height in [64.0, 136.0] {
            for loudness in [false, true] {
                let c = meter(ballistics);
                let l = meter_layout(column(height), &c, loudness);
                let ys: Vec<f32> = l.lines.iter().map(|m| m.label_y).collect();
                for pair in ys.windows(2) {
                    assert!(
                        pair[1] - pair[0] >= LABEL_ROW,
                        "{ballistics:?} {height} {loudness}: {ys:?}"
                    );
                }
                assert_eq!(
                    l.lines.iter().filter(|m| m.alignment).count(),
                    1,
                    "{ballistics:?} {height}"
                );
            }
        }
    }
}

#[test]
fn lines_cross_both_bars_and_nothing_sits_between_them() {
    let l = meter_layout(column(136.0), &meter(MeterBallistics::DigitalPeak), false);
    let [left, right] = l.bars;
    assert!(left.right() < right.left(), "a gap between the channels");
    assert!(l.lines_x.min <= left.left() && l.lines_x.max >= right.right());
    assert!(l.labels_right <= left.left());
    for m in &l.lines {
        assert!(m.y >= left.top() && m.y <= left.bottom(), "{}", m.y);
    }
}

#[test]
fn labels_stay_between_the_readouts() {
    for loudness in [false, true] {
        let l = meter_layout(column(136.0), &meter(MeterBallistics::DigitalPeak), loudness);
        let bars = l.bars[0];
        for m in &l.lines {
            assert!(m.label_y - LABEL_ROW / 2.0 >= bars.top() - 0.01, "{}", m.label);
            assert!(m.label_y + LABEL_ROW / 2.0 <= bars.bottom() + 0.01, "{}", m.label);
        }
        assert!(l.max.bottom() <= bars.top());
        if let Some(r) = l.loudness {
            assert!(r.top() >= bars.bottom());
        }
        assert_eq!(l.loudness.is_some(), loudness);
    }
}

#[test]
fn a_tall_digital_meter_labels_its_main_marks() {
    let l = meter_layout(column(136.0), &meter(MeterBallistics::DigitalPeak), false);
    let labels: Vec<&str> = l.lines.iter().map(|m| m.label.as_str()).collect();
    for want in ["0", "-10", "-20", "-30", "-40", "-60"] {
        assert!(labels.contains(&want), "{labels:?}");
    }
    // A very tall meter labels every mark.
    let c = meter(MeterBallistics::DigitalPeak);
    let tall = meter_layout(column(600.0), &c, false);
    assert_eq!(tall.lines.len(), scale_marks(&c).len() + 1);
}

#[test]
fn each_meter_labels_its_own_units() {
    let ebu = meter(MeterBallistics::EbuPpm);
    assert_eq!(mark_label(ebu.reference_dbfs + 8.0, &ebu), "+8");
    assert_eq!(mark_label(ebu.reference_dbfs, &ebu), "TEST");
    let din = meter(MeterBallistics::DinPpm);
    assert_eq!(mark_label(din.reference_dbfs + 9.0, &din), "0");
    assert_eq!(mark_label(din.reference_dbfs + 9.0 - 50.0, &din), "-50");
    let vu = meter(MeterBallistics::Vu);
    assert_eq!(mark_label(vu.reference_dbfs - 20.0, &vu), "-20");
    assert_eq!(mark_label(vu.reference_dbfs + 3.0, &vu), "+3");
    let k = meter(MeterBallistics::K14);
    assert_eq!(mark_label(-14.0, &k), "0");
    assert_eq!(mark_label(-10.0, &k), "+4");
    assert_eq!(mark_label(0.0, &k), "+14");
    let d = meter(MeterBallistics::DigitalPeak);
    assert_eq!(mark_label(-18.0, &d), "-18");
    assert_eq!(mark_label(0.0, &d), "0");
}
```

(`ALL_METERS` and `meter()` already exist further down the file; move
`ALL_METERS` above the new tests if the compiler asks.)

- [ ] **Step 2: Run** `cargo test -p fp-app --test meter_view`. Expected: compile errors (`meter_layout`, `mark_label`, `METER_WIDTH` not found).

- [ ] **Step 3: Implement** — in `widgets.rs`, replace `ALIGNMENT_MARK_WIDTH`, `MIN_MARK_GAP` and `mark_rows` with:

```rust
/// Width of the meter: the label column, then two bars with a gap.
const LABEL_COLUMN: f32 = 18.0;
const LABEL_GAP: f32 = 2.0;
const BAR_WIDTH: f32 = 14.0;
const BAR_GAP: f32 = 2.0;
pub const METER_WIDTH: f32 = LABEL_COLUMN + LABEL_GAP + 2.0 * BAR_WIDTH + BAR_GAP;
/// Height of the loudness line under the bars.
const LOUDNESS_LINE_HEIGHT: f32 = 12.0;
/// The closest two labels may be, centre to centre (monospace 9 px).
const LABEL_ROW: f32 = 10.0;
/// Thickness of the alignment line; the other lines are 1 px.
const ALIGNMENT_LINE_WIDTH: f32 = 2.0;

/// One labelled mark of the meter's scale (see [`meter_layout`]).
#[derive(Debug, Clone, PartialEq)]
pub struct MeterLine {
    pub y: f32,
    pub label_y: f32,
    pub label: String,
    pub alignment: bool,
}

/// Where the meter draws each of its parts (feedback spec §3.2).
#[derive(Debug, Clone, PartialEq)]
pub struct MeterLayout {
    pub labels_right: f32,
    pub bars: [Rect; 2],
    pub max: Rect,
    pub loudness: Option<Rect>,
    pub lines_x: egui::Rangef,
    pub lines: Vec<MeterLine>,
}

/// The label of the scale mark at `db` dBFS, in the chosen meter's own
/// units: EBU relative to TEST (shown as `TEST`), DIN to its 0, VU to 0 VU,
/// K-System to its 0, the digital meter in dBFS.
pub fn mark_label(db: f32, c: &MeterConfig) -> String {
    let zero = match c.ballistics {
        MeterBallistics::EbuPpm => c.reference_dbfs,
        MeterBallistics::DinPpm => c.reference_dbfs + PERMITTED_MAXIMUM_DB,
        MeterBallistics::Vu => c.reference_dbfs,
        MeterBallistics::K20 | MeterBallistics::K14 | MeterBallistics::K12 => alignment_dbfs(c),
        MeterBallistics::DigitalPeak | MeterBallistics::Custom => 0.0,
    };
    let value = (db - zero).round() as i32;
    if c.ballistics == MeterBallistics::EbuPpm && value == 0 {
        "TEST".to_owned()
    } else if value > 0 {
        format!("+{value}")
    } else {
        value.to_string()
    }
}

/// Lays the meter out in `rect`: the maximum readout on top, the loudness
/// line at the bottom when `loudness`, the labels on the left and the two
/// bars. Labels are kept top down while they have room; the alignment mark
/// always keeps its line and label.
pub fn meter_layout(rect: Rect, c: &MeterConfig, loudness: bool) -> MeterLayout {
    let bars_left = rect.left() + LABEL_COLUMN + LABEL_GAP;
    let top = rect.top() + MAX_LINE_HEIGHT;
    let bottom = if loudness {
        rect.bottom() - LOUDNESS_LINE_HEIGHT
    } else {
        rect.bottom()
    }
    .max(top + 1.0);
    let bar = |i: f32| {
        let x = bars_left + i * (BAR_WIDTH + BAR_GAP);
        Rect::from_x_y_ranges(x..=x + BAR_WIDTH, top..=bottom)
    };
    let bars = [bar(0.0), bar(1.0)];
    let bars_right = bars_left + 2.0 * BAR_WIDTH + BAR_GAP;
    let height = bottom - top;
    let y_of = |db: f32| bottom - meter_position(db, c) * height;
    // The label is centred on its line but kept inside the bars' span.
    let label_y = |y: f32| {
        let half = LABEL_ROW / 2.0;
        y.clamp(top + half, (bottom - half).max(top + half))
    };
    let line = |db: f32, alignment: bool| {
        let width = if alignment { ALIGNMENT_LINE_WIDTH } else { 1.0 };
        let y = y_of(db).clamp(top + width / 2.0, (bottom - width / 2.0).max(top));
        MeterLine {
            y,
            label_y: label_y(y),
            label: mark_label(db, c),
            alignment,
        }
    };
    let alignment = line(alignment_dbfs(c), true);
    let mut lines: Vec<MeterLine> = Vec::new();
    // Top down, so where the scale is dense the upper marks are kept.
    for mark in scale_marks(c).into_iter().rev() {
        let candidate = line(mark, false);
        let crowded = lines
            .iter()
            .chain(std::iter::once(&alignment))
            .any(|kept| (kept.label_y - candidate.label_y).abs() < LABEL_ROW);
        if !crowded {
            lines.push(candidate);
        }
    }
    lines.push(alignment);
    lines.sort_by(|a, b| a.label_y.total_cmp(&b.label_y));
    MeterLayout {
        labels_right: rect.left() + LABEL_COLUMN,
        bars,
        max: Rect::from_x_y_ranges(bars_left..=bars_right, rect.top()..=top),
        loudness: loudness.then(|| {
            Rect::from_x_y_ranges(bars_left..=bars_right, bottom..=rect.bottom())
        }),
        lines_x: egui::Rangef::new(bars_left, bars_right),
        lines,
    }
}
```

Note: when the alignment level coincides with a scale mark (DIN 0 is not the
alignment, but EBU TEST and K 0 are), the scale mark is dropped as crowded and
only the alignment line remains, so a label is never drawn twice.

- [ ] **Step 4: Run** `cargo test -p fp-app --test meter_view`. Expected: the new tests PASS. `vu` still calls `mark_rows`, so fix the build by replacing, in `vu`, the block that draws marks between the channels with a temporary no-op is **not** allowed: go straight to Task 3's Step 3 for `vu` if the build breaks, and commit Tasks 2 and 3 together (ledger it).

- [ ] **Step 5: Commit** — `feat(ui): lay the meter out with a labelled dB scale` (body: labels chosen so they never overlap, lines across both bars instead of marks in the gap).

---

### Task 3: Draw the meter from its layout (F7)

**Files:**
- Modify: `crates/fp-app/src/ui/widgets.rs` (`vu`, `fader`)
- Modify: `crates/fp-app/src/ui/player.rs` (callers pass a height)
- Test: `crates/fp-app/tests/meter_view.rs`

**Interfaces:**
- Consumes: `meter_layout`, `MeterLayout`, `METER_WIDTH` (Task 2).
- Produces: `pub fn vu(ui: &mut Ui, height: f32, reading: &MeterReading, c: &MeterConfig, labels: &MeterLabels) -> bool` and `pub fn fader(ui: &mut Ui, height: f32, position: f32, label: &str) -> Option<f32>`. `MeterLabels` gains `pub meter: String` (the meter's accessible name, e.g. the loudness line or "Level meter").

- [ ] **Step 1: Write the failing test** — append to `meter_view.rs`:

```rust
#[test]
fn the_meter_and_fader_fill_the_height_they_are_given() {
    let mut h = egui_kittest::Harness::new_ui(|ui| {
        ui.horizontal(|ui| {
            let labels = fp_app::ui::widgets::MeterLabels {
                meter: "Level meter".to_owned(),
                loudness: String::new(),
                max: "Maximum".to_owned(),
                max_tip: String::new(),
            };
            fp_app::ui::widgets::vu(ui, 120.0, &MeterReading::default(), &MeterConfig::default(), &labels);
            fp_app::ui::widgets::fader(ui, 120.0, 0.5, "Volume");
        });
    });
    h.run();
    let meter = h.get_by_label("Level meter").rect();
    let fader = h.get_by_label("Volume").rect();
    assert_eq!(meter.height(), 120.0);
    assert_eq!(meter.width(), METER_WIDTH);
    assert_eq!(fader.height(), 120.0);
    assert!(fader.left() >= meter.right());
}
```

- [ ] **Step 2: Run** `cargo test -p fp-app --test meter_view the_meter_and_fader`. Expected: compile error (arity of `vu`/`fader`, no `meter` field).

- [ ] **Step 3: Implement.**
  - `MeterLabels`: add `pub meter: String` first, documented "The meter's accessible name".
  - `vu`: allocate `vec2(METER_WIDTH, height)`; always set `response.widget_info(Label, labels.meter)`; compute `let l = meter_layout(rect, c, line.is_some())`; the max readout interacts on `l.max` and is drawn centred at `l.max.center_top()`; bars are `l.bars` (background, zone columns, K-System RMS, peak hold exactly as today but with `x..=x+14` replaced by the bar rect's x-range, and `bars_top/bars_bottom` by the bar rect); then the lines, **after** the bars:

```rust
    for m in &l.lines {
        let width = if m.alignment { 2.0 } else { 1.0 };
        let colour = if m.alignment {
            theme::NEUTRAL_300
        } else {
            theme::NEUTRAL_400.gamma_multiply(0.35)
        };
        painter.rect_filled(
            Rect::from_x_y_ranges(l.lines_x, m.y - width / 2.0..=m.y + width / 2.0),
            0.0,
            colour,
        );
        painter.text(
            pos2(l.labels_right, m.label_y),
            Align2::RIGHT_CENTER,
            &m.label,
            FontId::monospace(8.0),
            theme::NEUTRAL_400,
        );
    }
```

    and the loudness text centred in `l.loudness`. The old "marks in the gap" loop is deleted (F7). The loudness line's accessible name moves to `labels.meter` (the player passes the loudness text there when the line is on, `t.tr("meter-label")` otherwise — see Task 4).
  - `fader`: take `height` and allocate `vec2(22.0, height)`.
  - `player.rs`: pass `64.0` to both for now (Task 4 moves them), and build `MeterLabels { meter: <loudness text or "Level meter">, … }`. Add the string `meter-label = Level meter` / `meter-label = Medidor de nivel` to both locales.

- [ ] **Step 4: Run** `cargo test -p fp-app`. Expected: PASS, including `clicking_the_maximum_restarts_it` and `the_player_meter_reads_its_telemetry` (update the latter's query if it looked the loudness node up by `Role::Label`; keep the assertion on the loudness text).

- [ ] **Step 5: Commit** — `fix(ui): draw meter reference lines across both bars` (body: the marks painted in the 2 px gap looked like stray dots; lines at 35 % across both bars with a labelled scale replace them).

---

### Task 4: The right-hand column (F11)

**Files:**
- Modify: `crates/fp-app/src/ui/player.rs` (`show`, `info_row`, `transport`, new `meter_column` and `time_row`)
- Test: `crates/fp-app/tests/main_screen.rs`

**Interfaces:**
- Consumes: `vu(ui, height, …)`, `fader(ui, height, …)`, `METER_WIDTH`, `tabular_size`/`paint_tabular` (plan 1).

- [ ] **Step 1: Write the failing test** — append to `main_screen.rs`:

```rust
#[test]
fn the_meter_and_fader_form_a_column_right_of_the_transport() {
    let (h, _) = harness(state(1, 3));
    let play = h.get_by_label("Play").rect();
    let meter = h.get_by_label("Level meter").rect();
    let fader = h.query_all_by_label_contains("Volume").next().unwrap().rect();
    let wave = h.get_by_label("Waveform: click to seek").rect();
    let countdown = h.get_by_label("-00:00.0").rect();
    // The column spans the info row and the transport row.
    assert!(meter.top() < play.top() - 40.0, "{meter:?} {play:?}");
    assert!((meter.bottom() - play.bottom()).abs() <= 1.0, "{meter:?} {play:?}");
    assert!(meter.left() > countdown.right(), "{meter:?} {countdown:?}");
    assert!(fader.left() >= meter.right());
    assert!((fader.bottom() - meter.bottom()).abs() <= 1.0);
    // The waveform keeps the column's full width.
    assert!(wave.right() >= fader.right() - 1.0, "{wave:?} {fader:?}");
    // Elapsed / total sits under the block, right-aligned.
    let time = h.get_by_label("00:00 / 00:00").rect();
    assert!(time.top() >= play.bottom());
    assert!(time.bottom() <= wave.top());
    assert!((time.right() - fader.right()).abs() <= 1.0);
}

#[test]
fn at_the_minimum_player_width_the_countdown_still_fits() {
    let (h, _) = support::harness_sized(state(1, 3), egui::vec2(380.0, 700.0));
    let countdown = h.get_by_label("-00:00.0").rect();
    let meter = h.get_by_label("Level meter").rect();
    assert!(countdown.right() <= meter.left(), "{countdown:?} {meter:?}");
}
```

Add to `tests/support/mod.rs` a `harness_sized(state, size)` that is
`harness_from` with `.with_size(size)` (refactor `harness_from` to call it
with `vec2(1000.0, 700.0)`).

- [ ] **Step 2: Run** `cargo test -p fp-app --test main_screen the_meter_and_fader at_the_minimum`. Expected: FAIL (the meter is in the info row; no `00:00 / 00:00` node).

- [ ] **Step 3: Implement** in `player.rs`:
  - In `show`, replace the `info_row(…)` and `transport(…)` calls with `top_block(ui, scene, covers, id, &pv, &telemetry)` followed by `time_row(ui, &pv)`.
  - `top_block`: allocate `vec2(ui.available_width(), PLAY_SIZE * 2.0 + 8.0)`; the column width is `widgets::METER_WIDTH + 6.0 + 22.0`; split with `Rect::from_min_max`; draw `info_row` and `transport` in a `top_down` child on the left (the 8 px item spacing between them stays), and `meter_column` in a `left_to_right` child on the right. The gap between the two parts is 10 px.
  - `meter_column`: the `MeterLabels`, `vu(ui, height, …)` and `fader(ui, height, …)` code moved from `info_row`, with `height` the block's height.
  - `info_row`: drop the meter and fader.
  - `transport`: drop the `elapsed / total` box (the `right_to_left` block keeps only the countdown).
  - `time_row`: a row of 14 px, `right_to_left`, one `tabular_label` of `format!("{} / {}", format::clock(pv.elapsed), format::clock(pv.total.unwrap_or(0.0)))` in `font(12.0)`, `NEUTRAL_400`. Its accessible name is that text. Use `ui.add_space(-4.0)` before it, as in mockup C.

- [ ] **Step 4: Run** `cargo test -p fp-app`. Expected: PASS. Then build release and check by eye with the `CLAUDE.md` screenshot procedure (a paused demo session): the column lines up with the cover's top and the Play button's bottom; labels readable; no dots between bars.

- [ ] **Step 5: Commit** — `feat(ui): move the meter and fader into a column on the right` (body: operators asked for a taller meter with a readable scale; the column spans the info and transport rows as in the approved mockup).

---

### Task 5: Docs, review and pull request

**Files:** `docs/superpowers/specs/2026-09-27-meters-design.md` (M4: layout, colours, labels, lines), `docs/superpowers/specs/2026-09-25-fauste-player-design.md` §8.3 (info row, transport, time row), `docs/user/players.md` (meter and fader column), `docs/technical/ui.md` (meter layout).

- [ ] **Step 1:** update the docs; commit `docs: describe the meter column`.
- [ ] **Step 2:** full verification: fmt, clippy, `cargo test --workspace`.
- [ ] **Step 3:** final review (fresh reviewer, most capable model); fix Critical/Important test-first; ledger Minors.
- [ ] **Step 4:** `scripts/check-commits.sh origin/master`; push; `gh pr create` (title `feat(ui): meter and fader column with a dB scale`); `gh pr checks --watch`; `gh pr merge --merge --delete-branch`; update local `master`.
