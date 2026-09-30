# Feedback Plan 1 — Interface Polish Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix six small interface issues from operator feedback:
- fixed-width times (F1);
- distinguishable output device names (F8);
- a segmented SINGLE|CONT control (F10);
- a "slate" default waveform colour (F14);
- a clearer stop-after-current icon (F19);
- the version in the top bar and an About window with the licence notices (F21).

**Architecture:** Everything is in the UI crate (`fp-app`), except:
- a `detail` field and a pure `device_labels` function in `fp-backends`;
- the default of `ui.wave_color` in `fp-model`.

New drawing helpers go in `ui/widgets.rs` and `ui/icons.rs`, and the About
window is a new module `ui/about.rs`. Opening the third-party notices file uses
the `open` crate on a helper thread (the UI never blocks).

**Tech Stack:** Rust 2024, egui/eframe 0.36, egui_kittest, cpal 0.18, Fluent.

**Spec:** [`docs/superpowers/specs/2026-09-30-operator-feedback-design.md`](../specs/2026-09-30-operator-feedback-design.md) §3.1.
Roadmap: [`2026-09-30-operator-feedback-roadmap.md`](2026-09-30-operator-feedback-roadmap.md).

## Global Constraints

- All code, comments and docs are in English. UI strings go in
  `crates/fp-app/locales/en-US/main.ftl` **and** `es-ES/main.ftl` (the i18n
  test fails otherwise).
- Never mention other playout products anywhere.
- `unwrap`, `expect` and `panic` are denied outside tests; prefer `get` over
  indexing.
- The UI thread never touches the file system or waits. The notices file is
  located at start-up and opened on a helper thread.
- Commit only when `cargo fmt --all --check && cargo clippy --workspace
  --all-targets -- -D warnings && cargo test --workspace` pass.
- Conventional Commits with scopes (`ui`, `backends`, `model`, `app`,
  `docs`). The body says why. It ends with the trailer
  `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Branch: `feat/interface-polish` from an up-to-date `master`. Ledger:
  `.superpowers/sdd/feedback-plan1/progress.md`.
- Copyright text, exactly: `Copyright © Marc Sánchez Fauste. All rights reserved.`

## Review Focus

- **Hour-long tracks and playlists.** `-1:02:03` is a different format from
  `-02:03`. The width may change at the hour boundary, but not from second to
  second (Task 1 tests both formats).
- **Devices with an empty or identical description.** The label must fall
  back to the name, and two identical labels must still differ (Task 2
  tests).
- **A config.json that stores `"sand"`** keeps sand; a missing field gets
  slate (Task 4 tests both).
- **A development build with no notices file.** The About button is disabled
  and clicking it does nothing (Task 6 test).
- **Esc with About open** closes About only; it must not also reach the
  player shortcuts or a playlist rename (Task 6 test).

---

### Task 1: Fixed-width times (F1)

**Files:**
- Modify: `crates/fp-app/src/ui/widgets.rs` (new helpers after `glyph`)
- Modify: `crates/fp-app/src/ui/player.rs` (`info_row` cue time, `transport` countdown and elapsed/total, `footer`)
- Test: `crates/fp-app/tests/tabular.rs` (new)

**Interfaces:**
- Produces:
  - `pub fn widgets::tabular_size(painter: &Painter, text: &str, font: &FontId) -> Vec2`;
  - `pub fn widgets::paint_tabular(painter: &Painter, left_top: Pos2, text: &str, font: &FontId, color: Color32) -> Rect`;
  - `pub fn widgets::tabular_label(ui: &mut Ui, text: &str, font: &FontId, color: Color32) -> Response`.

  Plan 2 uses them for `elapsed / total`.

- [ ] **Step 1: Write the failing test**

Create `crates/fp-app/tests/tabular.rs`:

```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Times keep their width as they change (feedback spec F1).

use egui_kittest::Harness;
use fp_app::ui::theme;
use fp_app::ui::widgets::{font, font_medium, tabular_size};

/// Widths of `texts` in `font`, measured with the app's fonts installed:
/// (plain proportional layout, tabular layout).
fn widths(texts: &'static [&'static str], font: egui::FontId) -> Vec<(f32, f32)> {
    let mut h = Harness::new_ui_state(
        move |ui, out: &mut Vec<(f32, f32)>| {
            theme::apply(ui.ctx());
            *out = texts
                .iter()
                .map(|t| {
                    let plain = ui
                        .painter()
                        .layout_no_wrap((*t).to_owned(), font.clone(), egui::Color32::WHITE)
                        .size()
                        .x;
                    (plain, tabular_size(ui.painter(), t, &font).x)
                })
                .collect();
        },
        Vec::new(),
    );
    // The fonts are installed on the first frame and used from the second.
    h.run_steps(3);
    h.state().clone()
}

#[test]
fn the_countdown_keeps_its_width_as_digits_change() {
    let w = widths(&["-00:00", "-11:11", "-88:88"], font_medium(38.0));
    // The proportional font really does move (the defect).
    assert!(w[1].0 < w[2].0, "{w:?}");
    // The tabular layout does not.
    assert_eq!(w[0].1, w[1].1);
    assert_eq!(w[1].1, w[2].1);
}

#[test]
fn tenths_and_small_times_keep_their_width() {
    let w = widths(&[".1", ".8"], font_medium(17.0));
    assert_eq!(w[0].1, w[1].1);
    let w = widths(&["-11:11", "-40:08", "1:11:11", "8:08:08"], font(12.0));
    assert_eq!(w[0].1, w[1].1);
    // The hour format is wider, but steady within itself.
    assert_eq!(w[2].1, w[3].1);
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p fp-app --test tabular`
Expected: compile error, `cannot find function tabular_size in module widgets`.

- [ ] **Step 3: Implement the helpers**

In `crates/fp-app/src/ui/widgets.rs`, extend the `egui` import with `Pos2`:

```rust
use egui::{
    Align2, Color32, FontFamily, FontId, Painter, Pos2, Rect, Response, Sense, Stroke, StrokeKind,
    Ui, Vec2, WidgetInfo, WidgetType, pos2, vec2,
};
```

Then add after `glyph`:

```rust
/// Width of one digit cell: the widest of `0`–`9` in `font`. Times laid out
/// in such cells keep their width as they change (tabular figures, which
/// the UI font does not offer through egui).
fn digit_cell(painter: &Painter, font: &FontId) -> f32 {
    ('0'..='9')
        .map(|d| {
            painter
                .layout_no_wrap(d.to_string(), font.clone(), Color32::WHITE)
                .size()
                .x
        })
        .fold(0.0, f32::max)
}

/// The size `text` takes when painted by [`paint_tabular`].
pub fn tabular_size(painter: &Painter, text: &str, font: &FontId) -> Vec2 {
    let cell = digit_cell(painter, font);
    let mut size = Vec2::ZERO;
    for c in text.chars() {
        let glyph = painter
            .layout_no_wrap(c.to_string(), font.clone(), Color32::WHITE)
            .size();
        size.x += if c.is_ascii_digit() { cell } else { glyph.x };
        size.y = size.y.max(glyph.y);
    }
    size
}

/// Paints `text` from `left_top` with every digit centred in a cell of the
/// same width. Returns the rectangle it covers.
pub fn paint_tabular(
    painter: &Painter,
    left_top: Pos2,
    text: &str,
    font: &FontId,
    color: Color32,
) -> Rect {
    let cell = digit_cell(painter, font);
    let mut x = left_top.x;
    let mut height: f32 = 0.0;
    for c in text.chars() {
        let galley = painter.layout_no_wrap(c.to_string(), font.clone(), color);
        let size = galley.size();
        let width = if c.is_ascii_digit() { cell } else { size.x };
        painter.galley(pos2(x + (width - size.x) / 2.0, left_top.y), galley, color);
        x += width;
        height = height.max(size.y);
    }
    Rect::from_min_max(left_top, pos2(x, left_top.y + height))
}

/// A label whose digits keep their width (see [`paint_tabular`]). Its
/// accessible name is the text.
pub fn tabular_label(ui: &mut Ui, text: &str, font: &FontId, color: Color32) -> Response {
    let size = tabular_size(ui.painter(), text, font);
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    let owned = text.to_owned();
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, owned.clone()));
    if ui.is_rect_visible(rect) {
        paint_tabular(ui.painter(), rect.left_top(), text, font, color);
    }
    response
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `cargo test -p fp-app --test tabular`
Expected: PASS (2 tests).

- [ ] **Step 5: Use the helpers in the player column**

In `crates/fp-app/src/ui/player.rs`:

1. **Cue time** (`info_row`, inside `if pv.cueing`): replace the `ui.add(egui::Label::new(RichText::new(format!("{} {cue}", icon::HEADPHONES)) …))` call with:

```rust
widgets::tabular_label(ui, &cue, &font(11.0), theme::CUE);
ui.add(
    egui::Label::new(
        RichText::new(icon::HEADPHONES)
            .font(font(11.0))
            .color(theme::CUE),
    )
    .selectable(false),
);
```

(The layout is right-to-left, so the time is added first and the icon ends up
on its left.)

2. **Elapsed / total** (`transport`, the `right_to_left` block): replace the width
computation and the two `p.text` calls with tabular ones:

```rust
let total = pv.total.unwrap_or(0.0);
let total_text = format!("/ {}", format::clock(total));
let elapsed_text = format::clock(pv.elapsed);
let small = font(12.0);
let w = widgets::tabular_size(ui.painter(), &total_text, &small)
    .x
    .max(widgets::tabular_size(ui.painter(), &elapsed_text, &small).x)
    .max(40.0)
    + 10.0;
let h = 36.0;
let (rect, _) = ui.allocate_exact_size(vec2(w, h), Sense::hover());
let p = ui.painter();
p.rect_filled(
    Rect::from_min_size(rect.left_top(), vec2(1.0, h)),
    0.0,
    theme::NEUTRAL_800,
);
widgets::paint_tabular(
    p,
    pos2(rect.left() + 9.0, rect.top() + 2.0),
    &elapsed_text,
    &small,
    theme::NEUTRAL_300,
);
let bottom = widgets::tabular_size(p, &total_text, &small).y;
widgets::paint_tabular(
    p,
    pos2(rect.left() + 9.0, rect.bottom() - 2.0 - bottom),
    &total_text,
    &small,
    theme::NEUTRAL_500,
);
```

3. **Countdown**: replace the `LayoutJob` block (from `let mut job = egui::text::LayoutJob::default();` to `ui.add(egui::Label::new(job).selectable(false));`) with:

```rust
let big = font_medium(38.0);
let tenths_font = font_medium(17.0);
let main_size = widgets::tabular_size(ui.painter(), &main, &big);
let tenths_size = widgets::tabular_size(ui.painter(), &tenths, &tenths_font);
let (rect, response) = ui.allocate_exact_size(
    vec2(main_size.x + tenths_size.x, main_size.y),
    Sense::hover(),
);
let spoken = format!("{main}{tenths}");
response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, spoken.clone()));
widgets::paint_tabular(ui.painter(), rect.left_top(), &main, &big, colour);
// The tenths sit on the bottom of the big digits, smaller and dimmed.
widgets::paint_tabular(
    ui.painter(),
    pos2(rect.left() + main_size.x, rect.bottom() - tenths_size.y - 4.0),
    &tenths,
    &tenths_font,
    theme::NEUTRAL_500,
);
```

4. **Footer**: replace the two time labels in `footer`'s `right_to_left` block. For the total, use:

```rust
widgets::tabular_label(ui, &format::clock(times.total), &font(11.0), theme::NEUTRAL_300);
ui.add(
    egui::Label::new(
        RichText::new(t.tr("footer-total"))
            .font(font(11.0))
            .color(theme::NEUTRAL_500),
    )
    .selectable(false),
);
```

The layout is right-to-left, so the value comes first and the caption ends up
on its left. For the remaining time, replace the `-{}` label with:

```rust
widgets::tabular_label(
    ui,
    &format!("-{}", format::clock(times.remaining)),
    &font_medium(13.0),
    theme::TEXT,
);
```

- [ ] **Step 6: Run the whole UI test suite**

Run: `cargo test -p fp-app`
Expected: PASS.

- [ ] **Step 7: Check it by eye**

Run `cargo build --release -p fp-app`. Then take a screenshot of a playing
player with the `CLAUDE.md` procedure
(`env -u WAYLAND_DISPLAY FAUSTE_HOME=<scratch> target/release/fauste-player`,
`xwininfo`, `import`), twice about a second apart.

Check that:
- the countdown's left edge does not move;
- the tenths sit on the baseline of the big digits. If they float, adjust the
  `- 4.0` offset and note it in the ledger.

- [ ] **Step 8: Commit**

```bash
git add crates/fp-app/src/ui/widgets.rs crates/fp-app/src/ui/player.rs crates/fp-app/tests/tabular.rs
git commit -m "fix(ui): keep countdowns and times at a fixed width

The UI font has proportional digits, so the countdown and the playlist
times shifted every tenth of a second. Digits are now laid out in cells
as wide as the widest digit.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: Output device labels (F8)

**Files:**
- Modify: `crates/fp-backends/src/lib.rs` (`DeviceInfo`, new `device_labels`)
- Modify: `crates/fp-backends/src/cpal_backend.rs:232-270` (`enumerate_devices`)
- Modify: `crates/fp-backends/src/null.rs:54`, `crates/fp-backends/src/offline.rs:172` (add `detail: None`)
- Modify: `crates/fp-app/src/ui/settings.rs` (route picker around line 775, bit-perfect list around line 665)
- Test: `crates/fp-backends/tests/backends.rs`

**Interfaces:**
- Produces:
  - `DeviceInfo.detail: Option<String>`;
  - `pub fn fp_backends::device_labels(devices: &[DeviceInfo]) -> Vec<String>`.

- [ ] **Step 1: Write the failing test**

Append to `crates/fp-backends/tests/backends.rs` (and add `DeviceInfo, device_labels` to the `use fp_backends::{…}` list):

```rust
fn device(id: &str, name: &str, detail: Option<&str>) -> DeviceInfo {
    DeviceInfo {
        id: DeviceId(id.to_owned()),
        name: name.to_owned(),
        detail: detail.map(str::to_owned),
        channels: 2,
        sample_rates: vec![(44_100, 48_000)],
        buffer_frames: None,
        exclusive_capable: false,
        rate_switching: false,
    }
}

#[test]
fn device_labels_tell_same_named_devices_apart() {
    let list = [
        device("alsa:front:CARD=PCH,DEV=0", "HDA Intel PCH", Some("Front output / input")),
        device("alsa:surround51:CARD=PCH,DEV=0", "HDA Intel PCH", Some("5.1 Surround output to Front, Center, Rear and Subwoofer speakers")),
        device("alsa:hw:CARD=PCH,DEV=0", "HDA Intel PCH", Some("Direct hardware device without any conversions")),
    ];
    assert_eq!(
        device_labels(&list),
        vec![
            "HDA Intel PCH — Front output / input".to_owned(),
            "HDA Intel PCH — 5.1 Surround output to Front, Center, Rear and Subwoofer speakers".to_owned(),
            "HDA Intel PCH — Direct hardware device without any conversions".to_owned(),
        ]
    );
}

#[test]
fn device_labels_fall_back_to_the_name_and_then_the_id() {
    let list = [
        device("a", "Speakers", None),
        device("b", "Speakers", Some("  ")),
        device("c", "Headphones", Some("Headphones")),
        device("d", "USB DAC", None),
    ];
    assert_eq!(
        device_labels(&list),
        vec![
            "Speakers (a)".to_owned(),
            "Speakers (b)".to_owned(),
            "Headphones".to_owned(),
            "USB DAC".to_owned(),
        ]
    );
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p fp-backends --test backends device_labels`
Expected: compile errors, since `DeviceInfo` has no field `detail` and
`device_labels` is not found.

- [ ] **Step 3: Implement**

In `crates/fp-backends/src/lib.rs`, add the field after `name` in `DeviceInfo`:

```rust
    /// What sets this device apart from others with the same name (the
    /// output profile of a sound card), when the backend says.
    pub detail: Option<String>,
```

Then add after the struct:

```rust
/// The labels an output picker shows for `devices`, in order: the name,
/// then ` — detail` when the backend gives one that adds something, then
/// ` (id)` for any label two devices still share, so that every choice can
/// be told apart.
pub fn device_labels(devices: &[DeviceInfo]) -> Vec<String> {
    let base: Vec<String> = devices
        .iter()
        .map(|d| {
            match d
                .detail
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty() && *s != d.name)
            {
                Some(detail) => format!("{} — {detail}", d.name),
                None => d.name.clone(),
            }
        })
        .collect();
    base.iter()
        .zip(devices)
        .map(|(label, d)| {
            if base.iter().filter(|l| *l == label).count() > 1 {
                format!("{label} ({})", d.id.0)
            } else {
                label.clone()
            }
        })
        .collect()
}
```

In `crates/fp-backends/src/null.rs` and `offline.rs`, add `detail: None,` after `name` in each `DeviceInfo { … }`.

In `crates/fp-backends/src/cpal_backend.rs`, `enumerate_devices`, replace the `let name = …;` statement with:

```rust
            let description = device.description().ok();
            let name = description
                .as_ref()
                .map_or_else(|| id.to_string(), |d| d.name().to_owned());
            let detail = description.as_ref().and_then(device_detail);
```

Then add `detail,` after `name,` in the `DeviceInfo { … }` literal, and this
free function at the bottom of the file, before any `#[cfg(test)]`:

```rust
/// The first description line beyond the name: ALSA lists every output
/// profile of a card under the card's name, and this line names the profile.
fn device_detail(d: &cpal::DeviceDescription) -> Option<String> {
    d.extended()
        .map(str::trim)
        .find(|line| !line.is_empty() && *line != d.name())
        .map(str::to_owned)
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-backends`
Expected: PASS.

- [ ] **Step 5: Show the labels in Settings**

In `crates/fp-app/src/ui/settings.rs`, in the route picker (the block that
builds `shown` and the `ComboBox` around line 775):

1. Compute the labels once, before `let shown`:

```rust
        let labels = fp_backends::device_labels(devices);
```

2. Replace the `shown` computation with:

```rust
        let shown = device
            .as_ref()
            .map(|d| {
                devices
                    .iter()
                    .zip(&labels)
                    .find(|(x, _)| &x.id.0 == d)
                    .map_or_else(|| d.clone(), |(_, label)| label.clone())
            })
            .unwrap_or_else(|| none_text.clone());
```

3. Replace `for d in devices {` with `for (d, label) in devices.iter().zip(&labels) {`, and `ui.selectable_label(on, &d.name)` with `ui.selectable_label(on, label)`.

In the bit-perfect list (around line 665), replace the `info` and `name`
lines with:

```rust
        let backend_devices = backends
            .iter()
            .find(|b| b.id == device.backend)
            .map(|b| b.devices.as_slice())
            .unwrap_or_default();
        let labels = fp_backends::device_labels(backend_devices);
        let found = backend_devices
            .iter()
            .zip(&labels)
            .find(|(d, _)| d.id.0 == device.device);
        let info = found.map(|(d, _)| d);
        let name = found.map_or(device.device.as_str(), |(_, label)| label.as_str());
```

(`capable` keeps using `info`.)

- [ ] **Step 6: Run the settings tests**

Run: `cargo test -p fp-app --test settings`
Expected: PASS. Offline devices have no detail and unique names, so their
labels are unchanged.

- [ ] **Step 7: Check on real hardware (Linux)**

Run the release build, open Settings → Audio outputs with the ALSA backend,
and open a device picker. Check that the entries of one card now differ.
Record the output in the ledger.

- [ ] **Step 8: Commit**

```bash
git add crates/fp-backends crates/fp-app/src/ui/settings.rs
git commit -m "fix(backends): tell same-named output devices apart

ALSA lists every output profile of a card (front, surround, hw…) under
the card's name, so the output picker showed identical entries. The
description's profile line now follows the name, and any label that is
still shared gets the device id.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: Segmented SINGLE|CONT control (F10)

**Files:**
- Modify: `crates/fp-app/src/ui/widgets.rs` (new `segmented`)
- Modify: `crates/fp-app/src/ui/player.rs` (`header`, the `for (mode, key, tip)` loop)
- Test: `crates/fp-app/tests/main_screen.rs`

**Interfaces:**
- Produces: `pub fn widgets::segmented(ui: &mut Ui, id: egui::Id, options: &[Segment<'_>], selected: usize) -> Option<usize>` (`id` must be unique per control, e.g. per player) with `pub struct Segment<'a> { pub text: &'a str, pub label: &'a str }`.

- [ ] **Step 1: Write the failing tests**

Append to `crates/fp-app/tests/main_screen.rs`:

```rust
#[test]
fn single_and_cont_are_one_joined_control() {
    let (mut h, fake) = harness(state(1, 3));
    let single = h.get_by_label("Stop after every track").rect();
    let cont = h.get_by_label("Continuous: chain tracks at the mix point").rect();
    // Side by side with no gap, SINGLE first.
    assert!((single.right() - cont.left()).abs() < 0.5, "{single:?} {cont:?}");
    assert_eq!(single.top(), cont.top());
    // Continuous is the default: clicking it does nothing, SINGLE switches.
    h.get_by_label("Continuous: chain tracks at the mix point").click();
    h.run_steps(2);
    assert!(sent(&fake).is_empty());
    h.get_by_label("Stop after every track").click();
    h.run_steps(2);
    assert_eq!(
        sent(&fake),
        vec![Command::SetMode(fake.player(0), fp_model::PlayMode::Single)]
    );
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p fp-app --test main_screen single_and_cont`
Expected: FAIL on the gap assertion. Today the tiles sit 6 px apart.

- [ ] **Step 3: Implement `segmented`**

In `crates/fp-app/src/ui/widgets.rs`, after `tile`:

```rust
/// One option of a [`segmented`] control: its visible text and its
/// accessible name (also the tooltip).
pub struct Segment<'a> {
    pub text: &'a str,
    pub label: &'a str,
}

/// A segmented control: the options side by side inside one border, the
/// selected one filled, so they read as one choice. Returns the index of an
/// option clicked when it is not the selected one.
pub fn segmented(
    ui: &mut Ui,
    id: egui::Id,
    options: &[Segment<'_>],
    selected: usize,
) -> Option<usize> {
    const HEIGHT: f32 = 20.0;
    const PAD: f32 = 7.0;
    let text_font = font_semibold(9.0);
    let widths: Vec<f32> = options
        .iter()
        .map(|o| {
            ui.painter()
                .layout_no_wrap(o.text.to_owned(), text_font.clone(), theme::TEXT)
                .size()
                .x
                + 2.0 * PAD
        })
        .collect();
    let (rect, _) = ui.allocate_exact_size(vec2(widths.iter().sum(), HEIGHT), Sense::hover());
    let mut clicked = None;
    let mut x = rect.left();
    for (i, (option, width)) in options.iter().zip(&widths).enumerate() {
        let r = Rect::from_min_size(pos2(x, rect.top()), vec2(*width, HEIGHT));
        x += width;
        let on = i == selected;
        let response = ui
            .interact(r, id.with(i), Sense::click())
            .on_hover_text(option.label);
        let label = option.label.to_owned();
        response.widget_info(|| WidgetInfo::selected(WidgetType::RadioButton, true, on, label.clone()));
        if response.clicked() && !on {
            clicked = Some(i);
        }
        if ui.is_rect_visible(r) {
            let fill = if on {
                theme::NEUTRAL_700
            } else if response.hovered() {
                theme::NEUTRAL_900
            } else {
                Color32::TRANSPARENT
            };
            let content = if on || response.hovered() {
                theme::TEXT
            } else {
                theme::NEUTRAL_500
            };
            ui.painter().rect_filled(r, 0.0, fill);
            ui.painter().text(r.center(), Align2::CENTER_CENTER, option.text, text_font.clone(), content);
        }
    }
    ui.painter().rect_stroke(
        rect,
        0.0,
        Stroke::new(1.0, theme::NEUTRAL_700),
        StrokeKind::Inside,
    );
    clicked
}
```

- [ ] **Step 4: Use it in the header**

In `crates/fp-app/src/ui/player.rs`, `header`, replace the whole `for (mode, key, tip) in [ … ] { … }` loop with:

```rust
                let single_text = t.tr("mode-single");
                let cont_text = t.tr("mode-cont");
                let single_tip = t.tr("tip-single");
                let cont_tip = t.tr("tip-cont");
                let modes = [PlayMode::Single, PlayMode::Continuous];
                let selected = usize::from(pv.mode == PlayMode::Continuous);
                if let Some(mode) = widgets::segmented(
                    ui,
                    egui::Id::new(("play-mode", id)),
                    &[
                        widgets::Segment {
                            text: &single_text,
                            label: &single_tip,
                        },
                        widgets::Segment {
                            text: &cont_text,
                            label: &cont_tip,
                        },
                    ],
                    selected,
                )
                .and_then(|i| modes.get(i).copied())
                {
                    scene.ctl.send(Command::SetMode(id, mode));
                }
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p fp-app --test main_screen`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/fp-app/src/ui/widgets.rs crates/fp-app/src/ui/player.rs crates/fp-app/tests/main_screen.rs
git commit -m "fix(ui): draw SINGLE and CONT as one segmented control

Two separate tiles with a gap read as two independent switches. One
border around both makes it clear they are one choice.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: "Slate" as the default waveform colour (F14)

**Files:**
- Modify: `crates/fp-app/src/ui/theme.rs` (`WAVE_PALETTE`, `DEFAULT_WAVE`, `wave_colors` doc)
- Modify: `crates/fp-model/src/config.rs:280` (`UiConfig::default`) and its tests module
- Test: `crates/fp-app/tests/theme.rs`, `crates/fp-model/src/config.rs` (tests module)

**Interfaces:**
- Produces: a palette entry `"slate"` and `ui.wave_color` defaulting to `"slate"`.

- [ ] **Step 1: Write the failing tests**

In `crates/fp-app/tests/theme.rs`, replace the body of `every_waveform_colour_name_resolves` with:

```rust
    assert_eq!(WAVE_PALETTE.len(), 9);
    for (name, colours) in WAVE_PALETTE {
        assert_eq!(wave_colors(name), *colours, "{name}");
        assert_eq!(
            wave_colors(&name.to_uppercase()),
            *colours,
            "{name} ignores case"
        );
    }
    assert_eq!(
        wave_colors("sand").played,
        Color32::from_rgb(0xe0, 0xcf, 0xac)
    );
    let slate = wave_colors("slate");
    assert_eq!(slate.played, theme::NEUTRAL_300);
    assert_eq!(slate.unplayed, Color32::from_rgb(0x4a, 0x4e, 0x5c));
    assert_eq!(wave_colors("no such colour"), slate);
```

In `crates/fp-model/src/config.rs`, tests module, add:

```rust
    #[test]
    fn the_default_waveform_colour_is_slate_and_a_stored_one_is_kept() {
        assert_eq!(Config::default().ui.wave_color, "slate");
        let c: Config = serde_json::from_str(r#"{"ui":{"wave_color":"sand"}}"#).unwrap();
        assert_eq!(c.ui.wave_color, "sand");
        let c: Config = serde_json::from_str(r#"{"ui":{}}"#).unwrap();
        assert_eq!(c.ui.wave_color, "slate");
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-app --test theme && cargo test -p fp-model the_default_waveform_colour`
Expected: both FAIL (palette length 8; default `"sand"`).

- [ ] **Step 3: Implement**

In `crates/fp-app/src/ui/theme.rs`:
- append `("slate", wave([0xcf, 0xd3, 0xe5], [0x4a, 0x4e, 0x5c])),` to `WAVE_PALETTE`;
- set `const DEFAULT_WAVE: WaveColors = wave([0xcf, 0xd3, 0xe5], [0x4a, 0x4e, 0x5c]);`;
- change the doc of `wave_colors` to "an unknown name falls back to Slate".

In `crates/fp-model/src/config.rs`, `UiConfig::default`, set `wave_color: "slate".to_owned(),`.

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-app --test theme && cargo test -p fp-model`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/fp-app/src/ui/theme.rs crates/fp-app/tests/theme.rs crates/fp-model/src/config.rs
git commit -m "feat(ui): make slate the default waveform colour

The warm sand default clashed with the cool Nocturne palette. Slate
uses the theme's neutrals, so the coloured markers stand out. A saved
configuration keeps its colour.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: "Play then stop" icon (F19)

**Files:**
- Modify: `crates/fp-app/src/ui/icons.rs` (`stop_after`)
- Test: `crates/fp-app/tests/theme.rs`

**Interfaces:**
- Produces: `icons::stop_after(rect: Rect, color: Color32) -> Vec<Shape>`, a triangle then a square. The signature is unchanged; plan 6 reuses it for entry flags.

- [ ] **Step 1: Write the failing test**

Append to `crates/fp-app/tests/theme.rs`:

```rust
#[test]
fn stop_after_is_a_play_triangle_then_a_stop_square() {
    use fp_app::ui::icons;
    // The player draws it in an 18 × 13 rectangle.
    let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(18.0, 13.0));
    let shapes = icons::stop_after(rect, theme::TEXT);
    assert_eq!(shapes.len(), 2);
    let triangle = shapes[0].visual_bounding_rect();
    let square = shapes[1].visual_bounding_rect();
    assert!(matches!(shapes[1], egui::Shape::Rect(_)), "{:?}", shapes[1]);
    assert!(triangle.right() < square.left(), "{triangle:?} {square:?}");
    assert!((square.width() - square.height()).abs() < 0.01, "{square:?}");
    assert!((triangle.center().y - square.center().y).abs() < 0.01);
    for r in [triangle, square] {
        assert!(rect.contains_rect(r), "{r:?}");
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p fp-app --test theme stop_after_is`
Expected: FAIL. Today's icon has 3 shapes.

- [ ] **Step 3: Implement**

Replace `stop_after` in `crates/fp-app/src/ui/icons.rs` (and drop the now
unused `PathShape` and `Pos2` imports if clippy flags them):

```rust
/// Stop after current: a play triangle followed by a stop square ("play,
/// then stop"), built from the two standard transport glyphs.
pub fn stop_after(rect: Rect, color: Color32) -> Vec<Shape> {
    let r = rect.shrink(rect.height() * 0.1);
    // The square's side: 80 % of the height, unless the width is the limit
    // (triangle 0.8 side + gap 0.25 side + square 1 side).
    let side = (r.height() * 0.8).min(r.width() / 2.05);
    let triangle_width = side * 0.8;
    let gap = side * 0.25;
    let left = r.center().x - (triangle_width + gap + side) / 2.0;
    let top = r.center().y - side / 2.0;
    let triangle = vec![
        pos2(left, top),
        pos2(left + triangle_width, top + side / 2.0),
        pos2(left, top + side),
    ];
    let square = Rect::from_min_size(
        pos2(left + triangle_width + gap, top),
        egui::vec2(side, side),
    );
    vec![
        Shape::convex_polygon(triangle, color, Stroke::NONE),
        Shape::rect_filled(square, 0.0, color),
    ]
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-app --test theme`
Expected: PASS, including `drawn_icons_stay_inside_their_rectangle`.

- [ ] **Step 5: Commit**

```bash
git add crates/fp-app/src/ui/icons.rs crates/fp-app/tests/theme.rs
git commit -m "fix(ui): redraw the stop-after-current icon as play then stop

The return arrow into a square looked improvised. A play triangle
followed by a stop square reads as \"play, then stop\" with two standard
transport glyphs.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 6: Version in the top bar and the About window (F21)

**Files:**
- Create: `crates/fp-app/src/ui/about.rs`
- Create: `crates/fp-app/assets/licenses/Phosphor-MIT.txt`
- Modify: `crates/fp-app/src/ui.rs` (add `pub mod about;`)
- Modify: `crates/fp-app/src/ui/app.rs` (`ViewState.about_open`, `AppUi` fields and builders, `top_bar`, `ui`, `keyboard`)
- Modify: `crates/fp-app/src/main.rs:158` (locate the notices at start-up)
- Modify: `Cargo.toml` (workspace dependency `open`), `crates/fp-app/Cargo.toml`
- Modify: `crates/fp-app/locales/en-US/main.ftl`, `crates/fp-app/locales/es-ES/main.ftl`
- Modify: `scripts/package-release.sh`, `scripts/package/macos.sh`, `crates/fp-app/Cargo.toml` (deb and rpm assets), `packaging/windows/main.wxs`: ship `Phosphor-MIT.txt` next to `Inter-OFL.txt`
- Test: `crates/fp-app/tests/about.rs` (new)

**Interfaces:**
- Produces:
  - `pub const about::VERSION: &str`;
  - `pub type about::NoticeOpener = Arc<dyn Fn(&Path) + Send + Sync>`;
  - `pub fn about::notice_candidates(exe_dir: &Path) -> Vec<PathBuf>`;
  - `pub fn about::find_notices(exe: &Path) -> Option<PathBuf>`;
  - `pub fn about::system_opener() -> NoticeOpener`;
  - `AppUi::with_notices(self, Option<PathBuf>) -> Self`;
  - `AppUi::with_notice_opener(self, NoticeOpener) -> Self`.

- [ ] **Step 1: Write the failing tests**

Create `crates/fp-app/tests/about.rs`:

```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The version and the About window (feedback spec F21).

mod support;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use egui::Key;
use egui_kittest::kittest::Queryable;
use fp_app::ui::about::{self, NoticeOpener, find_notices, notice_candidates};
use support::{harness, harness_from, state};

const ABOUT: &str = "About Fauste Player";

fn recorder() -> (NoticeOpener, Arc<Mutex<Vec<PathBuf>>>) {
    let opened = Arc::new(Mutex::new(Vec::new()));
    let seen = opened.clone();
    let opener: NoticeOpener = Arc::new(move |p: &Path| seen.lock().unwrap().push(p.to_owned()));
    (opener, opened)
}

#[test]
fn the_top_bar_shows_the_version() {
    let (h, _) = harness(state(1, 1));
    assert!(h.query_by_label_contains(about::VERSION).is_some());
}

#[test]
fn clicking_the_name_opens_about_and_escape_closes_it() {
    let (mut h, fake) = harness(state(1, 1));
    h.get_by_label(ABOUT).click();
    h.run_steps(2);
    assert!(h.query_by_label_contains("All rights reserved").is_some());
    assert!(h.query_by_label_contains(&format!("Version {}", about::VERSION)).is_some());
    h.key_press(Key::Escape);
    h.run_steps(2);
    assert!(h.query_by_label_contains("All rights reserved").is_none());
    // Esc only closed the window.
    assert!(fake.take_sent().is_empty());
}

#[test]
fn player_shortcuts_do_nothing_while_about_is_open() {
    let (mut h, fake) = harness(state(1, 1));
    h.get_by_label(ABOUT).click();
    h.run_steps(2);
    h.key_press(Key::Num1);
    h.run_steps(2);
    assert!(fake.take_sent().is_empty());
}

#[test]
fn the_bundled_licences_can_be_read() {
    let (mut h, _) = harness(state(1, 1));
    h.get_by_label(ABOUT).click();
    h.run_steps(2);
    h.get_by_label("Inter font — SIL Open Font License 1.1").click();
    h.run_steps(2);
    assert!(h.query_by_label_contains("SIL OPEN FONT LICENSE Version 1.1").is_some());
    h.get_by_label("Phosphor Icons — MIT License").click();
    h.run_steps(2);
    assert!(h.query_by_label_contains("Permission is hereby granted").is_some());
}

#[test]
fn third_party_licences_open_the_installed_file() {
    let (opener, opened) = recorder();
    let file = PathBuf::from("/opt/fauste/licenses/THIRD-PARTY.html");
    let path = file.clone();
    let (mut h, _) = harness_from(state(1, 1), move |ui| {
        ui.with_notices(Some(path)).with_notice_opener(opener)
    });
    h.get_by_label(ABOUT).click();
    h.run_steps(2);
    h.get_by_label("Third-party licences").click();
    h.run_steps(2);
    assert_eq!(*opened.lock().unwrap(), vec![file]);
}

#[test]
fn without_installed_notices_the_button_does_nothing_and_says_why() {
    let (opener, opened) = recorder();
    let (mut h, _) = harness_from(state(1, 1), move |ui| {
        ui.with_notices(None).with_notice_opener(opener)
    });
    h.get_by_label(ABOUT).click();
    h.run_steps(2);
    h.get_by_label("Third-party licences").click();
    h.run_steps(2);
    assert!(opened.lock().unwrap().is_empty());
    assert!(h.query_by_label_contains("installed with release packages").is_some());
}

#[test]
fn notices_are_looked_up_where_the_packages_install_them() {
    let dir = Path::new("/usr/bin");
    let c = notice_candidates(dir);
    assert!(c.contains(&dir.join("licenses").join("THIRD-PARTY.html")));
    assert!(c.contains(&dir.join("../share/doc/fauste-player/THIRD-PARTY.html")));
    assert!(c.contains(&dir.join("../Resources/licenses/THIRD-PARTY.html")));
}

#[test]
fn find_notices_returns_an_installed_file() {
    let root = tempfile::tempdir().unwrap();
    let bin = root.path().join("bin");
    let doc = root.path().join("share/doc/fauste-player");
    std::fs::create_dir_all(&bin).unwrap();
    std::fs::create_dir_all(&doc).unwrap();
    let exe = bin.join("fauste-player");
    assert_eq!(find_notices(&exe), None);
    std::fs::write(doc.join("THIRD-PARTY.html"), "<p>notices</p>").unwrap();
    let found = find_notices(&exe).unwrap();
    assert_eq!(found.canonicalize().unwrap(), doc.join("THIRD-PARTY.html").canonicalize().unwrap());
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-app --test about`
Expected: compile error, `unresolved import fp_app::ui::about`.

- [ ] **Step 3: Add the Phosphor notice and the `open` dependency**

Create `crates/fp-app/assets/licenses/Phosphor-MIT.txt`. Before writing it,
check the copyright line against the upstream licence at
`https://github.com/phosphor-icons/core/blob/main/LICENSE` (the icon set that
`egui-phosphor` bundles). If the year or holder differs, use upstream's
exactly and note it in the ledger.

```
MIT License

Copyright (c) 2023 Phosphor Icons

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

Add the dependency. Take the latest 5.x reported by `cargo search open
--limit 1` and pin it like the other workspace dependencies:

```toml
# Cargo.toml, [workspace.dependencies]
open = "5.3"
```

```toml
# crates/fp-app/Cargo.toml, [dependencies]
open.workspace = true
```

Then run `cargo deny check`. Expected: pass (`open` is MIT).

- [ ] **Step 4: Write `ui/about.rs`**

Create `crates/fp-app/src/ui/about.rs`:

```rust
//! The About window (feedback spec §3.1, F21): the version, the copyright
//! and the notices the licences of the bundled components require.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use egui::{RichText, ScrollArea, Ui, vec2};

use super::app::Scene;
use super::theme;
use super::widgets::{self, TileStyle, font, font_medium};

/// The version shown in the top bar and the About window.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The Inter font's copyright line and licence, shipped with the binary.
const INTER_OFL: &str = include_str!("../../assets/fonts/OFL.txt");
/// Phosphor Icons' copyright and permission notice.
const PHOSPHOR_MIT: &str = include_str!("../../assets/licenses/Phosphor-MIT.txt");

/// The name of the third-party notices file release builds install.
const NOTICES_FILE: &str = "THIRD-PARTY.html";

/// Opens the third-party notices file. The default hands it to the system
/// on a helper thread; tests record the call.
pub type NoticeOpener = Arc<dyn Fn(&Path) + Send + Sync>;

/// Where the release packages install the notices, relative to the
/// directory of the executable.
pub fn notice_candidates(exe_dir: &Path) -> Vec<PathBuf> {
    vec![
        // Release archives and the Windows installer.
        exe_dir.join("licenses").join(NOTICES_FILE),
        // The macOS app bundle.
        exe_dir.join("../Resources/licenses").join(NOTICES_FILE),
        // deb, rpm and AppImage.
        exe_dir.join("../share/doc/fauste-player").join(NOTICES_FILE),
        // Flatpak.
        exe_dir
            .join("../share/licenses/org.fauste.FaustePlayer")
            .join(NOTICES_FILE),
    ]
}

/// The installed notices file, if any. It reads the file system, so it is
/// called once at start-up, never from a frame.
pub fn find_notices(exe: &Path) -> Option<PathBuf> {
    let dir = exe.parent()?;
    notice_candidates(dir).into_iter().find(|p| p.is_file())
}

/// Opens a file with the system's default application on a helper thread,
/// so the interface never waits for it.
pub fn system_opener() -> NoticeOpener {
    Arc::new(|path: &Path| {
        let path = path.to_owned();
        let spawned = std::thread::Builder::new()
            .name("fp-open-notices".to_owned())
            .spawn(move || {
                if let Err(error) = open::that_detached(&path) {
                    tracing::warn!(%error, path = %path.display(), "could not open the licence notices");
                }
            });
        if let Err(error) = spawned {
            tracing::warn!(%error, "could not start the notices opener");
        }
    })
}

fn text(ui: &mut Ui, value: String, size: f32, color: egui::Color32) {
    ui.add(
        egui::Label::new(RichText::new(value).font(font(size)).color(color))
            .selectable(false)
            .wrap(),
    );
}

fn notice(ui: &mut Ui, title: &str, body: &str) {
    egui::CollapsingHeader::new(RichText::new(title).font(font_medium(12.0)).color(theme::TEXT))
        .id_salt(title)
        .show(ui, |ui| {
            ui.add(
                egui::Label::new(
                    RichText::new(body)
                        .font(egui::FontId::monospace(10.0))
                        .color(theme::NEUTRAL_400),
                )
                .wrap(),
            );
        });
}

/// Draws the About window. Returns whether it stays open.
pub(crate) fn show(
    ctx: &egui::Context,
    scene: &Scene<'_>,
    notices: Option<&Path>,
    opener: &NoticeOpener,
) -> bool {
    let t = scene.i18n;
    let mut open = true;
    let screen = ctx.content_rect();
    let width = (screen.width() - 48.0).clamp(320.0, 560.0);
    let height = (screen.height() - 82.0).clamp(300.0, 560.0);
    egui::Modal::new(egui::Id::new("about"))
        .frame(egui::Frame::new().fill(theme::SURFACE).inner_margin(20.0))
        .backdrop_color(theme::NEUTRAL_900.gamma_multiply(0.7))
        .show(ctx, |ui| {
            ui.set_width(width);
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
            ui.add(
                egui::Label::new(
                    RichText::new(t.tr("app-name"))
                        .font(font_medium(18.0))
                        .color(theme::TEXT),
                )
                .selectable(false),
            );
            text(
                ui,
                t.tr_args("about-version", &[("version", VERSION.into())]),
                12.0,
                theme::NEUTRAL_400,
            );
            text(ui, t.tr("about-copyright"), 12.0, theme::NEUTRAL_300);
            ui.separator();
            text(ui, t.tr("about-bundled"), 12.0, theme::NEUTRAL_300);
            ScrollArea::vertical()
                .max_height((height - 220.0).max(80.0))
                .show(ui, |ui| {
                    notice(ui, &t.tr("about-inter"), INTER_OFL);
                    notice(ui, &t.tr("about-phosphor"), PHOSPHOR_MIT);
                });
            text(ui, t.tr("about-crates"), 12.0, theme::NEUTRAL_300);
            ui.horizontal(|ui| {
                let label = t.tr("about-third-party");
                let w = ui
                    .painter()
                    .layout_no_wrap(label.clone(), font(12.0), theme::NEUTRAL_300)
                    .size()
                    .x
                    + 24.0;
                let clicked = widgets::tile(
                    ui,
                    vec2(w, 24.0),
                    &label,
                    notices.is_some(),
                    TileStyle::plain(),
                    |p, r, c| {
                        p.text(r.center(), egui::Align2::CENTER_CENTER, &label, font(12.0), c);
                    },
                )
                .clicked();
                match notices {
                    Some(path) if clicked => opener(path),
                    Some(_) => {}
                    None => text(ui, t.tr("about-third-party-missing"), 11.0, theme::NEUTRAL_500),
                }
            });
            ui.add_space(4.0);
            let close = t.tr("about-close");
            let w = ui
                .painter()
                .layout_no_wrap(close.clone(), font(12.0), theme::NEUTRAL_300)
                .size()
                .x
                + 24.0;
            if widgets::tile(ui, vec2(w, 24.0), &close, true, TileStyle::plain(), |p, r, c| {
                p.text(r.center(), egui::Align2::CENTER_CENTER, &close, font(12.0), c);
            })
            .clicked()
            {
                open = false;
            }
        });
    open
}
```

Add `pub mod about;` to `crates/fp-app/src/ui.rs`, in alphabetical order
before `pub mod app;`.

- [ ] **Step 5: Add the strings**

Append to `crates/fp-app/locales/en-US/main.ftl`:

```
tip-about = About Fauste Player
about-version = Version { $version }
about-copyright = Copyright © Marc Sánchez Fauste. All rights reserved.
about-bundled = This program includes the following components under their own licences:
about-inter = Inter font — SIL Open Font License 1.1
about-phosphor = Phosphor Icons — MIT License
about-crates = It is also built with open-source Rust libraries, each under its own licence. Their notices are in the third-party licences file.
about-third-party = Third-party licences
about-third-party-missing = The third-party licences file is installed with release packages.
about-close = Close
```

Append to `crates/fp-app/locales/es-ES/main.ftl`:

```
tip-about = Acerca de Fauste Player
about-version = Versión { $version }
about-copyright = Copyright © Marc Sánchez Fauste. Todos los derechos reservados.
about-bundled = Este programa incluye los siguientes componentes bajo sus propias licencias:
about-inter = Fuente Inter — SIL Open Font License 1.1
about-phosphor = Phosphor Icons — Licencia MIT
about-crates = También está construido con bibliotecas Rust de código abierto, cada una bajo su propia licencia. Sus avisos están en el fichero de licencias de terceros.
about-third-party = Licencias de terceros
about-third-party-missing = El fichero de licencias de terceros se instala con los paquetes de release.
about-close = Cerrar
```

- [ ] **Step 6: Wire it into the app**

In `crates/fp-app/src/ui/app.rs`:

1. Add `use super::about::{self, NoticeOpener};` next to the other `super::` imports.

2. In `ViewState`, after `settings_open`, add:

```rust
    pub about_open: bool,
```

3. In `AppUi`, after `service_faults`, add these fields, and initialise them in
`AppUi::new` with `notices: None,` and `opener: about::system_opener(),`:

```rust
    /// The installed third-party notices, found at start-up.
    notices: Option<PathBuf>,
    opener: NoticeOpener,
```

4. Add the builders next to `with_platform`:

```rust
    /// The installed third-party notices file (see `about::find_notices`).
    pub fn with_notices(mut self, notices: Option<PathBuf>) -> Self {
        self.notices = notices;
        self
    }

    /// How the About window opens the notices file.
    pub fn with_notice_opener(mut self, opener: NoticeOpener) -> Self {
        self.opener = opener;
        self
    }
```

5. In `ui`, replace the `else` branch of `if self.view.settings_open { … }` with:

```rust
        } else {
            self.settings_shown = false;
            if self.view.about_open {
                self.view.about_open =
                    about::show(&ctx, &scene, self.notices.as_deref(), &self.opener);
            } else {
                self.file_drops(&ctx, &state);
            }
        }
```

6. In `keyboard`, right after the `if self.view.settings_open { … return; }`
block, add:

```rust
        if self.view.about_open {
            if escape {
                self.view.about_open = false;
            }
            return;
        }
```

7. In `top_bar`, replace the `app-name` `ui.add(…)` with:

```rust
    let name = ui.add(
        egui::Label::new(
            RichText::new(scene.i18n.tr("app-name"))
                .font(font_medium(12.0))
                .color(theme::TEXT),
        )
        .selectable(false),
    );
    let version = ui.add(
        egui::Label::new(
            RichText::new(format!("v{}", about::VERSION))
                .font(font(11.0))
                .color(theme::NEUTRAL_500),
        )
        .selectable(false),
    );
    let about_label = scene.i18n.tr("tip-about");
    let response = ui
        .interact(name.rect.union(version.rect), ui.id().with("about"), Sense::click())
        .on_hover_text(&about_label);
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, true, about_label.clone())
    });
    if response.clicked() {
        view_state.about_open = true;
    }
```

In `crates/fp-app/src/main.rs`, extend the builder chain at line 158 with a
start-up lookup. This runs before the first frame, so it is not a UI-thread
file scan:

```rust
        .with_platform(platform)
        .with_notices(
            std::env::current_exe()
                .ok()
                .and_then(|exe| fp_app::ui::about::find_notices(&exe)),
        );
```

- [ ] **Step 7: Run the tests to verify they pass**

Run: `cargo test -p fp-app --test about && cargo test -p fp-app --test i18n`
Expected: PASS. If a kittest query cannot see a `CollapsingHeader` body
label, check how the header's accessible name is exposed with
`h.get_by_label_contains("Inter font")` and adjust the query, not the
behaviour. Record it in the ledger.

- [ ] **Step 8: Ship the Phosphor notice in the packages**

Next to every place that ships `Inter-OFL.txt`, add `Phosphor-MIT.txt`:
- `scripts/package-release.sh`: after the `OFL.txt` copy, add
  `cp "${root}/crates/fp-app/assets/licenses/Phosphor-MIT.txt" "${stage}/licenses/Phosphor-MIT.txt"`.
- `scripts/package/macos.sh`: after line 59, the same copy into
  `"${app}/Contents/Resources/licenses/Phosphor-MIT.txt"`.
- `crates/fp-app/Cargo.toml`: in the deb assets, add
  `["assets/licenses/Phosphor-MIT.txt", "usr/share/doc/fauste-player/Phosphor-MIT.txt", "644"],`.
  In the rpm assets, add
  `{ source = "assets/licenses/Phosphor-MIT.txt", dest = "/usr/share/doc/fauste-player/Phosphor-MIT.txt", mode = "644", doc = true },`.
- `packaging/windows/main.wxs`: a component `iconlicense` like
  `fontlicense` (`Name='Phosphor-MIT.txt'`, source
  `crates\fp-app\assets\licenses\Phosphor-MIT.txt`, `KeyPath='yes'`), and its
  `ComponentRef` next to `fontlicense`'s.
- `packaging/flatpak/org.fauste.FaustePlayer.yml`: after the `Inter-OFL.txt`
  install line, add
  `- install -Dm644 crates/fp-app/assets/licenses/Phosphor-MIT.txt /app/share/licenses/org.fauste.FaustePlayer/Phosphor-MIT.txt`.

Run `bash -n scripts/package-release.sh scripts/package/macos.sh`. Expected:
no output.

- [ ] **Step 9: Check it by eye**

Run the release build and take a screenshot of the top bar and of the About
window with a licence expanded.

- [ ] **Step 10: Commit**

```bash
git add Cargo.toml Cargo.lock crates/fp-app scripts packaging
git commit -m "feat(ui): show the version and an About window

The top bar now shows the version, and clicking the name opens an About
window with the copyright and the notices the bundled font and icons
require. A button opens the installed third-party licences file on a
helper thread, and the Phosphor notice now ships with every package.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 7: Docs, verification and pull request

**Files:**
- Modify:
  - `README.md` (features: About; Licence section mentions the About window);
  - `docs/user/getting-started.md` (top bar: version and About);
  - `docs/user/players.md` (segmented mode control, stop-after icon);
  - `docs/user/settings.md` (output device labels);
  - `docs/technical/ui.md` (tabular times, `segmented`, `about.rs`, notices lookup, Slate fallback at line 77);
  - `docs/technical/persistence.md:170` (`wave_color` default `slate`, list includes `slate`);
  - `docs/technical/backends.md` (`DeviceInfo.detail`, `device_labels`);
  - `docs/superpowers/specs/2026-09-25-fauste-player-design.md` (§8.1 waveform colours: add Slate, default **Slate**; §8.3 header: segmented SINGLE|CONT; §8.1 top bar: version and About);
  - `CLAUDE.md` only if a command or layout changed (it should not).

- [ ] **Step 1: Update the docs listed above**

Keep every statement consistent with the code, in the existing style: short
sections and English.

- [ ] **Step 2: Full verification** (`superpowers:verification-before-completion`)

Run:

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo deny check
```

Expected: all green. Paste the summary into the ledger.

- [ ] **Step 3: Commit the docs**

```bash
git add README.md docs CLAUDE.md
git commit -m "docs: describe the interface polish

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

- [ ] **Step 4: Review and pull request**

1. Run `superpowers:requesting-code-review` with a fresh reviewer on the most
   capable model. Fix Critical and Important findings test-first, and defer
   the Minor ones to the ledger.
2. Run `scripts/check-commits.sh origin/master`.
3. Push `feat/interface-polish` and open the PR with `gh pr create`
   (title: `feat(ui): interface polish from operator feedback`), following
   `.github/pull_request_template.md`. The body ends with
   `🤖 Generated with [Claude Code](https://claude.com/claude-code)`.
4. Run `gh pr checks --watch`. Fix any red check on the branch.
5. When CI is green and the review is done:
   `gh pr merge --merge --delete-branch`, then update the local `master`.
