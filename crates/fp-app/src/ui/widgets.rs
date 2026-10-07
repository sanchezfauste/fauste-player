//! Custom-drawn parts of a player column: tile buttons, the VU meter, the
//! volume fader and the waveform.

use egui::{
    Align2, Color32, FontFamily, FontId, Painter, Pos2, Rect, Response, Sense, Stroke, StrokeKind,
    Ui, Vec2, WidgetInfo, WidgetType, pos2, vec2,
};

use fp_engine::meter::MeterReading;
use fp_model::{LoudnessReadout, MeterBallistics, MeterConfig};

use super::format;
use super::theme::{self, WaveColors};
use super::view::MarkerFractions;
use super::wave_view::WaveView;
use crate::services::TrackMedia;
use fp_analysis::WavePeak;
use std::sync::Arc;

pub fn font(size: f32) -> FontId {
    FontId::proportional(size)
}

pub fn font_medium(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(theme::MEDIUM.into()))
}

pub fn font_semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(theme::SEMIBOLD.into()))
}

/// Colours of a flat, square button.
#[derive(Debug, Clone, Copy)]
pub struct TileStyle {
    pub fill: Color32,
    pub border: Color32,
    pub border_width: f32,
    pub content: Color32,
    pub hover_fill: Color32,
    pub hover_content: Color32,
    pub active_fill: Color32,
}

impl TileStyle {
    pub fn plain() -> Self {
        Self {
            fill: Color32::TRANSPARENT,
            border: theme::NEUTRAL_700,
            border_width: 1.0,
            content: theme::NEUTRAL_300,
            hover_fill: theme::NEUTRAL_900,
            hover_content: theme::TEXT,
            active_fill: theme::NEUTRAL_800,
        }
    }
}

/// The blink phase of a paused button: lit for the first half of each
/// second, dark for the second half (main spec rule 6).
pub fn blink(time: f64) -> bool {
    (time * 2.0).floor() as i64 % 2 == 0
}

/// The look of the Pause button of something that is paused: amber, blinking
/// with `blink`.
pub fn paused_style(blink: bool) -> TileStyle {
    TileStyle {
        fill: if blink {
            theme::AMBER_BG
        } else {
            Color32::TRANSPARENT
        },
        content: if blink {
            theme::AMBER_TEXT
        } else {
            theme::AMBER_DIM
        },
        ..TileStyle::plain()
    }
}

/// A button whose content is painted by `paint`. `label` is its accessible
/// name and tooltip.
pub fn tile(
    ui: &mut Ui,
    size: Vec2,
    label: &str,
    enabled: bool,
    style: TileStyle,
    paint: impl FnOnce(&Painter, Rect, Color32),
) -> Response {
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(size, sense);
    let owned = label.to_owned();
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, enabled, owned.clone()));
    if ui.is_rect_visible(rect) {
        let (fill, content) = if !enabled {
            (style.fill, style.content.gamma_multiply(0.45))
        } else if response.is_pointer_button_down_on() {
            (style.active_fill, style.hover_content)
        } else if response.hovered() {
            (style.hover_fill, style.hover_content)
        } else {
            (style.fill, style.content)
        };
        let painter = ui.painter();
        painter.rect_filled(rect, 0.0, fill);
        let border = if enabled {
            style.border
        } else {
            style.border.gamma_multiply(0.45)
        };
        painter.rect_stroke(
            rect,
            0.0,
            Stroke::new(style.border_width, border),
            StrokeKind::Inside,
        );
        paint(painter, rect, content);
    }
    response.on_hover_text(label)
}

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
        response
            .widget_info(|| WidgetInfo::selected(WidgetType::RadioButton, true, on, label.clone()));
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
            ui.painter().text(
                r.center(),
                Align2::CENTER_CENTER,
                option.text,
                text_font.clone(),
                content,
            );
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

/// Paints a Phosphor glyph centred in `rect`.
pub fn glyph(painter: &Painter, rect: Rect, icon: &str, size: f32, color: Color32, fill: bool) {
    let family = if fill {
        FontFamily::Name(theme::ICONS_FILL.into())
    } else {
        FontFamily::Proportional
    };
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        icon,
        FontId::new(size, family),
        color,
    );
}

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

/// As [`paint_tabular`], with the text ending at `right_top.x`.
pub fn paint_tabular_right(
    painter: &Painter,
    right_top: Pos2,
    text: &str,
    font: &FontId,
    color: Color32,
) -> Rect {
    let width = tabular_size(painter, text, font).x;
    paint_tabular(
        painter,
        pos2(right_top.x - width, right_top.y),
        text,
        font,
        color,
    )
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

/// Colour zone of a meter level (meters spec M4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Zone {
    Normal,
    Warning,
    Danger,
}

/// The digital scale's deflection (meters spec M4), in % of 0 dBFS: the
/// slope in %/dB of each 10 dB below −20, from −30 down to −70. The last
/// slope carries on below −70, so a lower floor still has height.
const DIGITAL_SLOPES: [f32; 5] = [2.0, 1.5, 0.75, 0.5, 0.25];

fn digital_deflection(db: f32) -> f32 {
    if db >= -20.0 {
        return 50.0 + (db + 20.0) * 2.5;
    }
    let mut top = 50.0;
    let mut at = -20.0;
    for slope in DIGITAL_SLOPES {
        if db >= at - 10.0 {
            return top - (at - db) * slope;
        }
        top -= 10.0 * slope;
        at -= 10.0;
    }
    top - (at - db) * DIGITAL_SLOPES.last().copied().unwrap_or(0.25)
}

/// The K-System scale: linear in dB from the top to −24 over this share of
/// the height, then to −60 below it.
const K_LINEAR_SHARE: f32 = 0.8;

/// Where `db` (dBFS) sits on the chosen meter's scale, from 0 (bottom) to 1
/// (top), after its standard (meters spec M4). Levels above the top sit at
/// the top.
pub fn meter_position(db: f32, c: &MeterConfig) -> f32 {
    if db.is_nan() {
        return 0.0;
    }
    let p = match c.ballistics {
        MeterBallistics::EbuPpm => (db - c.reference_dbfs + 12.0) / 24.0,
        MeterBallistics::DinPpm => {
            // Relative to +5 dB on the DIN scale (0 = alignment + 9 dB).
            let din = db - c.reference_dbfs - 9.0;
            let quarter = |d: f32| 10f32.powf((d - 5.0) / 80.0);
            (quarter(din) - quarter(-50.0)) / (1.0 - quarter(-50.0))
        }
        MeterBallistics::Vu => 10f32.powf((db - c.reference_dbfs - 3.0) / 20.0),
        MeterBallistics::K20 | MeterBallistics::K14 | MeterBallistics::K12 => {
            let k = c.ballistics.k_reference_dbfs().unwrap_or(-20.0);
            let rel = db - k;
            let top = -k;
            if rel >= -24.0 {
                (1.0 - K_LINEAR_SHARE) + K_LINEAR_SHARE * (rel + 24.0) / (top + 24.0)
            } else {
                (1.0 - K_LINEAR_SHARE) * (rel + 60.0) / 36.0
            }
        }
        MeterBallistics::DigitalPeak | MeterBallistics::Custom => {
            let bottom = digital_deflection(c.floor_db.min(-1.0));
            if db <= c.floor_db {
                0.0
            } else {
                (digital_deflection(db.min(0.0)) - bottom) / (100.0 - bottom)
            }
        }
    };
    p.clamp(0.0, 1.0)
}

/// The chosen scale's marks, in dBFS, bottom first (meters spec M4).
pub fn scale_marks(c: &MeterConfig) -> Vec<f32> {
    let r = c.reference_dbfs;
    match c.ballistics {
        MeterBallistics::EbuPpm => (-3..=3).map(|i| r + 4.0 * i as f32).collect(),
        MeterBallistics::DinPpm => [-50.0, -40.0, -30.0, -20.0, -10.0, -5.0, 0.0, 5.0]
            .iter()
            .map(|d| r + 9.0 + d)
            .collect(),
        MeterBallistics::Vu => [
            -20.0, -10.0, -7.0, -5.0, -3.0, -2.0, -1.0, 0.0, 1.0, 2.0, 3.0,
        ]
        .iter()
        .map(|vu| r + vu)
        .collect(),
        MeterBallistics::K20 | MeterBallistics::K14 | MeterBallistics::K12 => {
            let k = c.ballistics.k_reference_dbfs().unwrap_or(-20.0);
            let mut marks: Vec<f32> = [-60.0, -50.0, -40.0, -30.0]
                .into_iter()
                .chain((-6..=1).map(|i| 4.0 * i as f32))
                .map(|rel| k + rel)
                .collect();
            marks.push(0.0);
            marks
        }
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
    }
}

/// The step (dB) of the minor ticks in the scale segment that starts at
/// `from` dBFS: coarse in the sparse low part of each scale, fine where the
/// marks are close (operator feedback 4, Q11.4).
fn minor_step_db(c: &MeterConfig, from: f32) -> f32 {
    // Whether `from` lies below `limit` dB relative to the scale's `zero`.
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

/// The steps (dB, in the scale's own units) a ruler segment's minor ticks
/// may take, finest first: a segment takes the finest one that is not finer
/// than its scale's spacing ([`minor_step_db`]) and keeps its ticks
/// [`MIN_TICK_SPACING`] apart.
const MINOR_STEP_LADDER: [f32; 6] = [0.5, 1.0, 2.0, 2.5, 5.0, 10.0];

/// The multiples of `step` dB in the scale's own units (counted from
/// [`scale_zero`]) strictly between `low` and `high` dBFS, ascending.
fn step_multiples(low: f32, high: f32, step: f32, c: &MeterConfig) -> Vec<f32> {
    let mut out = Vec::new();
    if !(low.is_finite() && high.is_finite() && step.is_finite() && step > 0.0) {
        return out;
    }
    let zero = scale_zero(c);
    // A count, not a running sum, so the levels do not drift.
    let mut k = ((low - zero) / step).floor();
    loop {
        let db = zero + k * step;
        if db >= high - SAME_MARK_DB {
            return out;
        }
        if db > low + SAME_MARK_DB {
            out.push(db);
        }
        k += 1.0;
    }
}

/// The levels (dBFS, ascending) of the finest minor ticks: between each
/// pair of adjacent scale marks, every multiple of [`minor_step_db`] in the
/// scale's own units, never on a mark. The layout rules a short meter more
/// coarsely (see [`ruler_ticks`]).
pub fn minor_marks(c: &MeterConfig) -> Vec<f32> {
    let marks = scale_marks(c);
    marks
        .windows(2)
        .filter_map(|pair| Some((*pair.first()?, *pair.get(1)?)))
        .flat_map(|(a, b)| step_multiples(a, b, minor_step_db(c, a), c))
        .collect()
}

/// The ticks of both rulers, sorted by `y`:
/// - a major tick for every labelled mark, and the alignment tick;
/// - a minor tick for every scale mark that lost its label, where it stays
///   [`MIN_TICK_SPACING`] from the ticks above;
/// - between each two of those, minor ticks as on a measuring ruler: every
///   multiple of the finest step of [`MINOR_STEP_LADDER`] that keeps them
///   [`MIN_TICK_SPACING`] apart, or none.
fn ruler_ticks(
    lines: &[MeterLine],
    marks: &[f32],
    c: &MeterConfig,
    top: f32,
    bottom: f32,
) -> Vec<MeterTick> {
    let height = bottom - top;
    let half = TICK_THICKNESS / 2.0;
    let y_of = |db: f32| bottom - meter_position(db, c) * height;
    let mut ticks: Vec<MeterTick> = lines
        .iter()
        .filter(|m| m.alignment || !m.label.is_empty())
        .map(|m| MeterTick {
            db: m.db,
            y: m.y,
            kind: if m.alignment {
                TickKind::Alignment
            } else {
                TickKind::Major
            },
        })
        .collect();
    ticks.sort_by(|a, b| a.y.total_cmp(&b.y));
    // The scale marks that lost their label, top down as the labels are
    // kept; `ticks` stays sorted by `y`.
    for &db in marks.iter().rev() {
        if (db - alignment_dbfs(c)).abs() < SAME_MARK_DB {
            continue;
        }
        let y = y_of(db).clamp(top + half, (bottom - half).max(top + half));
        let at = ticks.partition_point(|t| t.y < y);
        let clear = |i: Option<usize>| {
            i.and_then(|i| ticks.get(i))
                .is_none_or(|t: &MeterTick| (t.y - y).abs() >= MIN_TICK_SPACING)
        };
        if clear(at.checked_sub(1)) && clear(Some(at)) {
            ticks.insert(
                at,
                MeterTick {
                    db,
                    y,
                    kind: TickKind::Minor,
                },
            );
        }
    }
    // Then rule each segment between two of those ticks (bottom up: `y`
    // falls as the level rises).
    let mut minors = Vec::new();
    for pair in ticks.windows(2).rev() {
        let (Some(high), Some(low)) = (pair.first(), pair.get(1)) else {
            continue;
        };
        let finest = minor_step_db(c, low.db);
        let fits = |levels: &[f32]| {
            let ys = std::iter::once(low.y)
                .chain(levels.iter().map(|db| y_of(*db)))
                .chain(std::iter::once(high.y));
            ys.clone()
                .zip(ys.skip(1))
                .all(|(a, b)| a - b >= MIN_TICK_SPACING - 0.01)
        };
        let ruled = MINOR_STEP_LADDER
            .iter()
            .filter(|step| **step >= finest - SAME_MARK_DB)
            .map(|step| step_multiples(low.db, high.db, *step, c))
            .find(|levels| fits(levels));
        minors.extend(ruled.into_iter().flatten().map(|db| MeterTick {
            db,
            y: y_of(db),
            kind: TickKind::Minor,
        }));
    }
    ticks.extend(minors);
    ticks.sort_by(|a, b| a.y.total_cmp(&b.y));
    ticks
}

/// The zone `db` falls in: the configured ones, or the K-System's own
/// (amber from 0, red above +4).
pub fn zone_of(db: f32, c: &MeterConfig) -> Zone {
    let danger = zone_start(Zone::Danger, c);
    let in_danger = if c.ballistics.k_reference_dbfs().is_some() {
        db > danger
    } else {
        db >= danger
    };
    if in_danger {
        Zone::Danger
    } else if db >= zone_start(Zone::Warning, c) {
        Zone::Warning
    } else {
        Zone::Normal
    }
}

/// The maximum readout: one decimal with its sign, or a dash when nothing
/// was measured.
pub fn max_readout(db: f32) -> String {
    if db.is_nan() || db <= -100.0 {
        return "—".to_owned();
    }
    let tenths = (db * 10.0).round();
    if tenths > 0.0 {
        format!("+{:.1}", tenths / 10.0)
    } else if tenths == 0.0 {
        "0.0".to_owned()
    } else {
        format!("{:.1}", tenths / 10.0)
    }
}

/// The loudness line under the meter: the value in LUFS (one decimal, or a
/// dash when nothing is measured) and whether it is within ±1 LU of the
/// target. `None` when the readout is off.
pub fn loudness_line(r: &MeterReading, c: &MeterConfig) -> Option<(String, bool)> {
    let value = match c.loudness {
        LoudnessReadout::Off => return None,
        LoudnessReadout::Momentary => r.momentary_lufs,
        LoudnessReadout::ShortTerm => r.short_term_lufs,
    };
    Some(match value {
        Some(lufs) => (
            // One decimal while it fits the meter's width.
            if lufs > -99.95 {
                format!("{lufs:.1}")
            } else {
                format!("{lufs:.0}")
            },
            (lufs - c.loudness_target_lufs).abs() <= 1.0,
        ),
        None => ("—".to_owned(), false),
    })
}

fn zone_colour(zone: Zone) -> Color32 {
    match zone {
        Zone::Normal => theme::METER_NORMAL,
        Zone::Warning => theme::METER_WARNING,
        Zone::Danger => theme::METER_DANGER,
    }
}

/// Accessible names of the meter's parts.
pub struct MeterLabels {
    /// The meter's accessible name (the loudness line when it is on).
    pub meter: String,
    /// The maximum readout.
    pub max: String,
    /// Tooltip of the maximum readout.
    pub max_tip: String,
}

/// Height of the maximum readout above the bars.
const MAX_LINE_HEIGHT: f32 = 11.0;

/// Stereo level meter (meters spec M4, feedback spec §3.2): the scale on a
/// ruler on each side of the bars (labels, a tick for every label, minor
/// ticks, a white alignment tick), a continuous bar per channel on the
/// scale of the chosen meter's standard, the peak hold, the maximum readout
/// above and the loudness line below when it is on. Nothing is drawn over
/// the bars. K-System meters show the average (RMS) as the solid body and the
/// peak dimmed above it. Returns true when the operator clicked the maximum
/// to restart it.
pub fn vu(
    ui: &mut Ui,
    height: f32,
    reading: &MeterReading,
    c: &MeterConfig,
    labels: &MeterLabels,
) -> bool {
    let (rect, response) = ui.allocate_exact_size(vec2(meter_width(c), height), Sense::hover());
    let meter_label = labels.meter.clone();
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Label, true, meter_label.clone())
    });
    let line = loudness_line(reading, c);
    let l = meter_layout(rect, c, line.is_some());
    let max_response = ui
        .interact(l.max, response.id.with("max"), Sense::click())
        .on_hover_text(&labels.max_tip);
    let max_label = labels.max.clone();
    max_response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, true, max_label.clone())
    });
    let clicked = max_response.clicked();
    if !ui.is_rect_visible(rect) {
        return clicked;
    }
    let painter = ui.painter();
    painter.text(
        l.max.center_top(),
        Align2::CENTER_TOP,
        max_readout(reading.max_db),
        FontId::monospace(9.0),
        if zone_of(reading.max_db, c) == Zone::Danger {
            theme::METER_DANGER
        } else {
            theme::NEUTRAL_400
        },
    );
    let [first, _] = l.bars;
    let (bars_top, bars_bottom) = (first.top(), first.bottom());
    let height = (bars_bottom - bars_top).max(1.0);
    let y_of = |db: f32| bars_bottom - meter_position(db, c) * height;
    let level_y = [0usize, 1].map(|ch| y_of(reading.level_db.get(ch).copied().unwrap_or(-120.0)));
    // Where each zone starts on screen, bottom up.
    let zones = [
        (bars_bottom, Zone::Normal),
        (y_of(zone_start(Zone::Warning, c)), Zone::Warning),
        (y_of(zone_start(Zone::Danger, c)), Zone::Danger),
    ];
    // Paints a bar from `bottom` up to `top` (screen y) in the colours of
    // the zones it crosses.
    let column = |x: egui::Rangef, top: f32, bottom: f32, alpha: f32| {
        for (i, &(zone_bottom, zone)) in zones.iter().enumerate() {
            let zone_top = zones.get(i + 1).map_or(bars_top, |z| z.0);
            let low = zone_bottom.min(bottom);
            let high = zone_top.max(top);
            if low > high {
                painter.rect_filled(
                    Rect::from_x_y_ranges(x, high..=low),
                    0.0,
                    zone_colour(zone).gamma_multiply(alpha),
                );
            }
        }
    };
    let k_system = c.ballistics.k_reference_dbfs().is_some();
    for (ch, bar) in l.bars.iter().enumerate() {
        let x = bar.x_range();
        painter.rect_filled(*bar, 0.0, theme::NEUTRAL_800.gamma_multiply(0.55));
        let level = level_y.get(ch).copied().unwrap_or(bars_bottom);
        if k_system {
            let rms = y_of(reading.rms_db.get(ch).copied().unwrap_or(-120.0)).max(level);
            column(x, level, rms, 0.45);
            column(x, rms, bars_bottom, 1.0);
        } else {
            column(x, level, bars_bottom, 1.0);
        }
        let hold = reading.hold_db.get(ch).copied().unwrap_or(-120.0);
        let hold_y = y_of(hold);
        if c.peak_hold_in_use() && hold_y < bars_bottom && hold_y <= level {
            painter.rect_filled(
                Rect::from_x_y_ranges(x, hold_y..=(hold_y + 2.0).min(bars_bottom)),
                0.0,
                zone_colour(zone_of(hold, c)),
            );
        }
    }
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
    if let (Some((text, on_target)), Some(r)) = (line, l.loudness) {
        painter.text(
            r.center_bottom(),
            Align2::CENTER_BOTTOM,
            &text,
            FontId::monospace(9.0),
            if on_target {
                theme::METER_NORMAL
            } else {
                theme::NEUTRAL_400
            },
        );
    }
    clicked
}

/// Size of the scale's labels.
const LABEL_FONT_SIZE: f32 = 8.0;

/// Advance (px) of one character of the labels' font: the monospace font
/// (Hack, 1233/2048 em) at [`LABEL_FONT_SIZE`].
const LABEL_ADVANCE: f32 = LABEL_FONT_SIZE * 1233.0 / 2048.0;
/// Clear gap between a ruler's tick strip and its labels, so a label's
/// minus never reads as part of a tick. The strip itself is flush with its
/// bar.
const LABEL_TICK_GAP: f32 = 3.0;
/// Width of a ruler's tick strip: the length of a major tick.
const TICK_STRIP: f32 = 4.0;
/// Length of a minor tick.
const MINOR_TICK_LEN: f32 = 2.0;
/// Thickness of a major or minor tick; the alignment tick is
/// [`ALIGNMENT_LINE_THICKNESS`] thick.
const TICK_THICKNESS: f32 = 1.0;
/// Minor ticks stay at least this far (px) from every other tick (Q11.4).
const MIN_TICK_SPACING: f32 = 3.0;
const BAR_WIDTH: f32 = 14.0;
const BAR_GAP: f32 = 2.0;

/// Width of the label column on each side of the bars for the scale
/// `marks` of `c`: as wide as its widest label (3 characters, 15 px; EBU's
/// `TEST`, 20 px).
fn label_column_for(marks: &[f32], c: &MeterConfig) -> f32 {
    let chars = marks
        .iter()
        .map(|m| mark_label(*m, c).chars().count())
        .max()
        .unwrap_or(0);
    (chars as f32 * LABEL_ADVANCE).ceil()
}

/// Width of one side of the meter with a `label_column` px label column:
/// the labels, their gap to the ticks and the tick strip.
fn side_width(label_column: f32) -> f32 {
    label_column + LABEL_TICK_GAP + TICK_STRIP
}

/// Width of the label column on each side of meter `c`'s bars.
pub fn label_column(c: &MeterConfig) -> f32 {
    label_column_for(&scale_marks(c), c)
}

/// Width of meter `c`: a ruler on each side of the two bars.
pub fn meter_width(c: &MeterConfig) -> f32 {
    2.0 * side_width(label_column(c)) + 2.0 * BAR_WIDTH + BAR_GAP
}
/// Height of the loudness line under the bars.
const LOUDNESS_LINE_HEIGHT: f32 = 12.0;
/// The closest two labels may be, centre to centre (monospace 9 px).
const LABEL_ROW: f32 = 10.0;
/// Two levels closer than this (dB) are the same mark.
const SAME_MARK_DB: f32 = 0.05;
/// Thickness of the alignment line and tick: the vertical extent of the
/// horizontal marks (not their length).
const ALIGNMENT_LINE_THICKNESS: f32 = 2.0;

/// One labelled mark of the meter's scale (see [`meter_layout`]).
#[derive(Debug, Clone, PartialEq)]
pub struct MeterLine {
    /// The mark's level, dBFS.
    pub db: f32,
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
    /// The level the tick marks, dBFS.
    pub db: f32,
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

/// The colour a ruler tick is painted in (constants in `theme`).
pub fn tick_colour(kind: TickKind) -> Color32 {
    match kind {
        TickKind::Major => theme::METER_TICK,
        TickKind::Minor => theme::METER_TICK.gamma_multiply(theme::METER_TICK_MINOR_ALPHA),
        TickKind::Alignment => theme::METER_ALIGNMENT_TICK,
    }
}

/// Where the meter draws each of its parts (feedback spec §3.2).
#[derive(Debug, Clone, PartialEq)]
pub struct MeterLayout {
    pub bars: [Rect; 2],
    pub max: Rect,
    pub loudness: Option<Rect>,
    pub lines: Vec<MeterLine>,
    /// The rulers, left and right of the bars.
    pub rulers: [Ruler; 2],
    /// The ticks both rulers draw, sorted by `y`.
    pub ticks: Vec<MeterTick>,
}

/// The level (dBFS) that is 0 in the chosen meter's own units: EBU TEST,
/// DIN 0, 0 VU, the K-System's 0, and 0 dBFS on the digital meter.
pub fn scale_zero(c: &MeterConfig) -> f32 {
    match c.ballistics {
        MeterBallistics::EbuPpm | MeterBallistics::Vu => c.reference_dbfs,
        MeterBallistics::DinPpm => c.reference_dbfs + PERMITTED_MAXIMUM_DB,
        MeterBallistics::K20 | MeterBallistics::K14 | MeterBallistics::K12 => alignment_dbfs(c),
        MeterBallistics::DigitalPeak | MeterBallistics::Custom => 0.0,
    }
}

/// The label of the scale mark at `db` dBFS, in the chosen meter's own
/// units: EBU relative to TEST (shown as `TEST`), DIN to its 0, VU to 0 VU,
/// K-System to its 0, the digital meter in dBFS.
pub fn mark_label(db: f32, c: &MeterConfig) -> String {
    let value = (db - scale_zero(c)).round() as i32;
    if c.ballistics == MeterBallistics::EbuPpm && value == 0 {
        "TEST".to_owned()
    } else if value > 0 {
        format!("+{value}")
    } else {
        value.to_string()
    }
}

/// Lays the meter out in `rect`: the maximum readout on top, the loudness
/// line at the bottom when `loudness`, the two bars, and the labels and
/// ticks of a ruler on each side of them. Labels are kept top down while
/// they have room; the alignment mark always keeps its line and label.
pub fn meter_layout(rect: Rect, c: &MeterConfig, loudness: bool) -> MeterLayout {
    let marks = scale_marks(c);
    let labels = label_column_for(&marks, c);
    let bars_left = rect.left() + side_width(labels);
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
    let rulers = [
        Ruler {
            side: Side::Left,
            labels_x: rect.left() + labels,
            label_halign: egui::Align::Max,
            ticks: egui::Rangef::new(bars_left - TICK_STRIP, bars_left),
        },
        Ruler {
            side: Side::Right,
            labels_x: bars_right + TICK_STRIP + LABEL_TICK_GAP,
            label_halign: egui::Align::Min,
            ticks: egui::Rangef::new(bars_right, bars_right + TICK_STRIP),
        },
    ];
    let height = bottom - top;
    let y_of = |db: f32| bottom - meter_position(db, c) * height;
    let line = |db: f32, alignment: bool| {
        let width = if alignment {
            ALIGNMENT_LINE_THICKNESS
        } else {
            1.0
        };
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
            db,
            y,
            label_y: y,
            label_align,
            label: mark_label(db, c),
            alignment,
        }
    };
    let mut alignment = line(alignment_dbfs(c), true);
    // The alignment level is labelled only where the scale names it (EBU
    // TEST, K 0, 0 VU); elsewhere (the digital meter's −18) the heavier
    // line alone marks it, and the round labels stay.
    if !marks
        .iter()
        .any(|m| (m - alignment_dbfs(c)).abs() < SAME_MARK_DB)
    {
        alignment.label.clear();
    }
    let mut lines: Vec<MeterLine> = Vec::new();
    let crowds = |kept: &MeterLine, candidate: &MeterLine| {
        !kept.label.is_empty()
            && !candidate.label.is_empty()
            && (kept.label_centre() - candidate.label_centre()).abs() < LABEL_ROW
    };
    // Both ends of the scale first, so its range is always labelled.
    let ends = [marks.last().copied(), marks.first().copied()];
    // An end that is the alignment level is labelled by the alignment line:
    // that label is kept, and the other end yields if they crowd.
    let alignment_is_end = ends
        .iter()
        .flatten()
        .any(|m| (m - alignment_dbfs(c)).abs() < SAME_MARK_DB);
    for mark in ends.into_iter().flatten() {
        if (mark - alignment_dbfs(c)).abs() < SAME_MARK_DB {
            continue;
        }
        let candidate = line(mark, false);
        let crowded = lines
            .iter()
            .chain(alignment_is_end.then_some(&alignment))
            .any(|kept| crowds(kept, &candidate));
        if !crowded {
            lines.push(candidate);
        }
    }
    // The alignment label gives way to an end label (not to its own); its
    // line stays.
    if !alignment_is_end && lines.iter().any(|kept| crowds(kept, &alignment)) {
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
    lines.push(alignment);
    lines.sort_by(|a, b| a.label_centre().total_cmp(&b.label_centre()));
    let ticks = ruler_ticks(&lines, &marks, c, top, bottom);
    MeterLayout {
        bars,
        max: Rect::from_x_y_ranges(bars_left..=bars_right, rect.top()..=top),
        loudness: loudness
            .then(|| Rect::from_x_y_ranges(bars_left..=bars_right, bottom..=rect.bottom())),
        lines,
        rulers,
        ticks,
    }
}

/// The alignment level the meter marks, in dBFS: the K-System's 0, or
/// `reference_dbfs` (EBU TEST, DIN −9, 0 VU).
pub fn alignment_dbfs(c: &MeterConfig) -> f32 {
    c.ballistics.k_reference_dbfs().unwrap_or(c.reference_dbfs)
}

/// Permitted maximum level above alignment on the programme meters' scales
/// (EBU +9, DIN 0).
const PERMITTED_MAXIMUM_DB: f32 = 9.0;

/// The level (dBFS) where `zone` starts on the chosen meter (meters spec
/// M4): the configured zones on the digital scale; the scale's own on the
/// others, which have no yellow band.
fn zone_start(zone: Zone, c: &MeterConfig) -> f32 {
    let r = c.reference_dbfs;
    let (warning, danger) = match c.ballistics {
        MeterBallistics::K20 | MeterBallistics::K14 | MeterBallistics::K12 => {
            let k = alignment_dbfs(c);
            (k, k + 4.0)
        }
        // The VU scale's red arc runs from 0 VU.
        MeterBallistics::Vu => (r, r),
        MeterBallistics::EbuPpm | MeterBallistics::DinPpm => {
            (r + PERMITTED_MAXIMUM_DB, r + PERMITTED_MAXIMUM_DB)
        }
        MeterBallistics::DigitalPeak | MeterBallistics::Custom => (c.warning_dbfs, c.danger_dbfs),
    };
    match zone {
        Zone::Normal => f32::NEG_INFINITY,
        Zone::Warning => warning,
        Zone::Danger => danger,
    }
}

/// Width of the volume fader.
pub const FADER_WIDTH: f32 = 22.0;

/// Fader travel moved by one mouse-wheel event (3 dB near the top).
const FADER_WHEEL_STEP: f32 = 0.05;

/// Vertical volume fader (drag or wheel). Returns the new fader position
/// (0 bottom … 1 top) when the user moved it.
pub fn fader(ui: &mut Ui, height: f32, position: f32, label: &str) -> Option<f32> {
    let (rect, response) =
        ui.allocate_exact_size(vec2(FADER_WIDTH, height), Sense::click_and_drag());
    let owned = label.to_owned();
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Slider, true, owned.clone()));
    let mut changed = None;
    if (response.dragged() || response.clicked())
        && let Some(p) = response.interact_pointer_pos()
    {
        changed = Some((1.0 - (p.y - rect.top()) / rect.height()).clamp(0.0, 1.0));
    }
    if response.hovered() {
        // One fixed step per wheel event: the smoothed scroll delta spreads a
        // notch over many frames and would add up to a huge jump.
        let steps: f32 = ui.input(|i| {
            i.events
                .iter()
                .filter_map(|e| match e {
                    egui::Event::MouseWheel { delta, .. } if delta.y != 0.0 => {
                        Some(delta.y.signum())
                    }
                    _ => None,
                })
                .sum()
        });
        if steps != 0.0 {
            changed = Some((position + steps * FADER_WHEEL_STEP).clamp(0.0, 1.0));
        }
    }
    // Holding the knob still must not resend the same volume every frame.
    let changed = changed.filter(|v| (v - position).abs() > f32::EPSILON);
    let shown = changed.unwrap_or(position);
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let track = Rect::from_center_size(rect.center(), vec2(4.0, rect.height()));
        painter.rect_filled(track, 0.0, theme::NEUTRAL_800);
        let level = Rect::from_min_max(
            pos2(track.left(), track.bottom() - shown * rect.height()),
            track.right_bottom(),
        );
        painter.rect_filled(level, 0.0, theme::NEUTRAL_300);
        let knob_bottom = rect.bottom() - shown * (rect.height() - 6.0);
        let knob = Rect::from_min_max(
            pos2(rect.left() + 1.0, knob_bottom - 6.0),
            pos2(rect.right() - 1.0, knob_bottom),
        );
        painter.rect_filled(knob, 0.0, theme::NEUTRAL_100);
        painter.rect_stroke(
            knob,
            0.0,
            Stroke::new(1.0, theme::NEUTRAL_900),
            StrokeKind::Outside,
        );
    }
    response.on_hover_text(label);
    changed
}

/// How strongly the waveform's peak outline shows over the background,
/// relative to its solid RMS body.
const WAVE_PEAK_ALPHA: f32 = 0.45;

/// Rounding slack, in buckets, for column boundaries.
const BUCKET_EPSILON: f64 = 1e-6;

/// One pixel column of the waveform, as fractions of full scale.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct WaveColumn {
    /// The largest magnitude among the column's buckets.
    pub peak: f32,
    /// The RMS level of the column: the root of its buckets' mean square.
    pub rms: f32,
}

/// Reduces analysis buckets of `bucket_secs` to `columns` equal columns
/// spanning `span_secs`. Every column takes at least one bucket, so there
/// are no gaps when there are more columns than buckets; columns past the
/// audio are empty.
pub fn wave_columns(
    peaks: &[WavePeak],
    bucket_secs: f64,
    span_secs: f64,
    columns: usize,
) -> Vec<WaveColumn> {
    wave_columns_in(peaks, bucket_secs, 0.0, span_secs, columns)
}

/// As [`wave_columns`], for the `span_secs` starting at `start_secs`: only
/// the buckets of that stretch are reduced.
pub fn wave_columns_in(
    peaks: &[WavePeak],
    bucket_secs: f64,
    start_secs: f64,
    span_secs: f64,
    columns: usize,
) -> Vec<WaveColumn> {
    let full = f32::from(i16::MAX);
    let mut out = vec![WaveColumn::default(); columns];
    let valid = |secs: f64| secs.is_finite() && secs > 0.0;
    if !valid(bucket_secs) || !valid(span_secs) {
        return out;
    }
    // The bucket where column `c` starts. The nudge keeps a boundary that
    // division leaves a hair below a whole bucket (28.999… for 29) on it,
    // so the last bucket is not lost; a huge span saturates.
    let bucket_at = |c: usize| {
        ((start_secs.max(0.0) + (c as f64 / columns as f64) * span_secs) / bucket_secs
            + BUCKET_EPSILON) as usize
    };
    for (c, column) in out.iter_mut().enumerate() {
        let a0 = bucket_at(c);
        let a1 = bucket_at(c + 1).max(a0.saturating_add(1));
        let Some(buckets) = peaks.get(a0.min(peaks.len())..a1.min(peaks.len())) else {
            continue;
        };
        if buckets.is_empty() {
            continue;
        }
        let peak = buckets
            .iter()
            .map(|p| p.min.unsigned_abs().max(p.max.unsigned_abs()))
            .max()
            .unwrap_or(0);
        let mean_square = buckets
            .iter()
            .map(|p| (f32::from(p.rms) / full).powi(2))
            .sum::<f32>()
            / buckets.len() as f32;
        *column = WaveColumn {
            peak: (f32::from(peak) / full).min(1.0),
            rms: mean_square.sqrt().min(1.0),
        };
    }
    out
}

/// The last reduction one waveform drew: it is reused until the track,
/// the span or the width changes.
#[derive(Clone)]
pub struct WaveMemo {
    /// Held, so no other track can take its address while it is the key.
    media: Arc<TrackMedia>,
    start_secs: f64,
    span_secs: f64,
    columns: Arc<[WaveColumn]>,
}

/// The columns of `media` over `span_secs` at `columns` pixels, from `memo`
/// when it holds them, reduced (and remembered) otherwise.
pub fn memo_columns(
    memo: &mut Option<WaveMemo>,
    media: &Arc<TrackMedia>,
    span_secs: f64,
    columns: usize,
) -> Arc<[WaveColumn]> {
    memo_columns_in(memo, media, 0.0, span_secs, columns)
}

/// As [`memo_columns`], for the stretch starting at `start_secs`.
pub fn memo_columns_in(
    memo: &mut Option<WaveMemo>,
    media: &Arc<TrackMedia>,
    start_secs: f64,
    span_secs: f64,
    columns: usize,
) -> Arc<[WaveColumn]> {
    if let Some(m) = memo.as_ref().filter(|m| {
        Arc::ptr_eq(&m.media, media)
            && m.start_secs.to_bits() == start_secs.to_bits()
            && m.span_secs.to_bits() == span_secs.to_bits()
            && m.columns.len() == columns
    }) {
        return Arc::clone(&m.columns);
    }
    let reduced: Arc<[WaveColumn]> = wave_columns_in(
        &media.peaks,
        media.peak_bucket_secs,
        start_secs,
        span_secs,
        columns,
    )
    .into();
    *memo = Some(WaveMemo {
        media: Arc::clone(media),
        start_secs,
        span_secs,
        columns: Arc::clone(&reduced),
    });
    reduced
}

/// What the waveform shows.
pub struct WaveInput<'a> {
    /// Stable per player: keys the memoised columns.
    pub id: egui::Id,
    pub media: Option<&'a Arc<TrackMedia>>,
    pub total: Option<f64>,
    pub markers: MarkerFractions,
    pub colors: WaveColors,
    /// The MIX marker is drawn solid amber in Continuous mode, dim otherwise.
    pub mix_active: bool,
    pub mix_label: &'a str,
    pub accessible_label: &'a str,
    /// The stretch shown; `None` is the whole track.
    pub view: Option<WaveView>,
    /// An area drawn over the waveform (a button) where no seek starts.
    pub shield: Option<Rect>,
}

/// What the waveform reports for a frame (feedback 2 spec O10).
pub struct WaveOutput {
    pub response: Response,
    /// A click (a press and release within egui's drag threshold): the
    /// time under it. A drag never seeks.
    pub seek: Option<f64>,
    /// How far, in pixels, a primary drag that started on the waveform
    /// moved sideways this frame (positive: to the right). The caller pans
    /// a zoomed view by it; Alt-drag (marker editing) and a drag that
    /// starts under the shield (a button over the waveform) report 0.
    pub pan_dx: f32,
}

/// Whether a pan drag is held on the waveform `id` (the zoomed view then
/// does not follow the playhead).
pub fn pan_dragging(ui: &Ui, id: egui::Id) -> bool {
    ui.data(|d| d.get_temp::<bool>(id.with("pan-drag")).is_some())
}

/// Draws the waveform; reports a click's seek target and a drag's sideways
/// movement.
pub fn waveform(ui: &mut Ui, height: f32, input: &WaveInput<'_>) -> WaveOutput {
    let size = vec2(ui.available_width(), height);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click_and_drag());
    let owned = input.accessible_label.to_owned();
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Other, true, owned.clone()));
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, theme::NEUTRAL_900);
    painter.rect_stroke(
        rect,
        0.0,
        Stroke::new(1.0, theme::NEUTRAL_800),
        StrokeKind::Inside,
    );
    let inner = rect.shrink(1.0);
    // A player with no waveform lets go of the last track's columns.
    let memo_id = input.id.with("wave-columns");
    if input.media.is_none_or(|m| m.peaks.is_empty()) {
        ui.data_mut(|d| d.remove_temp::<Option<WaveMemo>>(memo_id));
    }
    let mid = inner.center().y;
    let w = inner.width();
    painter.rect_filled(
        Rect::from_min_size(pos2(inner.left(), mid), vec2(w, 1.0)),
        0.0,
        theme::NEUTRAL_800,
    );
    let pan_id = input.id.with("pan-drag");
    let Some(total) = input.total.filter(|t| *t > 0.0) else {
        ui.data_mut(|d| d.remove_temp::<bool>(pan_id));
        return WaveOutput {
            response,
            seek: None,
            pan_dx: 0.0,
        };
    };
    let view = input.view.unwrap_or_else(|| WaveView::full(total));
    // Markers are fractions of the track; the view maps their times.
    let x_of = |f: f32| view.x_of(f64::from(f) * total, inner);
    let m = input.markers;
    if let Some(f) = m.intro_end {
        painter.rect_filled(
            Rect::from_min_max(inner.left_top(), pos2(x_of(f), inner.bottom())),
            0.0,
            theme::INTRO.gamma_multiply(0.18),
        );
    }
    if let Some(f) = m.outro_start {
        painter.rect_filled(
            Rect::from_min_max(pos2(x_of(f), inner.top()), inner.right_bottom()),
            0.0,
            theme::OUTRO_SHADE.gamma_multiply(0.16),
        );
    }
    let play_x = m.position.map_or(inner.left(), x_of);
    if let Some(media) = input.media.filter(|m| !m.peaks.is_empty()) {
        let amp = (inner.height() / 2.0 - 3.0).max(1.0);
        // One column per pixel: the peaks as a faint outline, the RMS level
        // as the solid body inside it, all in one mesh.
        let mut memo = ui
            .data(|d| d.get_temp::<Option<WaveMemo>>(memo_id))
            .flatten();
        let columns = memo_columns_in(
            &mut memo,
            media,
            view.start_secs,
            view.span_secs,
            w as usize,
        );
        ui.data_mut(|d| d.insert_temp(memo_id, memo));
        let mut mesh = egui::Mesh::default();
        for (i, column) in columns.iter().enumerate() {
            let px = inner.left() + i as f32;
            let color = if px < play_x {
                input.colors.played
            } else {
                input.colors.unplayed
            };
            let peak = column.peak * amp;
            if peak > 0.0 {
                mesh.add_colored_rect(
                    Rect::from_min_max(pos2(px, mid - peak), pos2(px + 1.0, mid + peak)),
                    color.gamma_multiply(WAVE_PEAK_ALPHA),
                );
            }
            let rms = column.rms * amp;
            if rms > 0.0 {
                mesh.add_colored_rect(
                    Rect::from_min_max(pos2(px, mid - rms), pos2(px + 1.0, mid + rms)),
                    color,
                );
            }
        }
        painter.add(mesh);
    }
    // The trimmed head and tail are drawn dimmed, with a thin line at the
    // cue points: the whole file is shown, and where playback starts and
    // ends is plain (feedback spec F20).
    let look = cue_edge_look(m.ignored);
    if look.shade_trimmed {
        let secs = |f: Option<f32>| f.map(|f| f64::from(f) * total);
        for (region, edge) in view
            .trimmed(inner, secs(m.cue_in), secs(m.cue_out), total)
            .into_iter()
            .zip([true, false])
        {
            if let Some(r) = region {
                painter.rect_filled(r, 0.0, Color32::BLACK.gamma_multiply(TRIMMED_DIM));
                let x = if edge { r.right() } else { r.left() };
                painter.rect_filled(
                    Rect::from_min_size(pos2(x, inner.top()), vec2(1.0, inner.height())),
                    0.0,
                    theme::NEUTRAL_500.gamma_multiply(look.line_alpha),
                );
            }
        }
    } else {
        for f in [m.cue_in, m.cue_out].into_iter().flatten() {
            let x = x_of(f);
            if x >= inner.left() && x <= inner.right() {
                painter.rect_filled(
                    Rect::from_min_size(pos2(x, inner.top()), vec2(1.0, inner.height())),
                    0.0,
                    theme::NEUTRAL_500.gamma_multiply(look.line_alpha),
                );
            }
        }
    }
    let label_font = font_semibold(9.0);
    if let Some(f) = m.intro_end {
        painter.rect_filled(
            Rect::from_min_size(pos2(x_of(f), inner.top()), vec2(1.0, inner.height())),
            0.0,
            theme::CUE,
        );
    }
    if let Some(f) = m.outro_start {
        painter.rect_filled(
            Rect::from_min_size(pos2(x_of(f), inner.top()), vec2(1.0, inner.height())),
            0.0,
            theme::OUTRO_LINE,
        );
    }
    if let Some(f) = m.segue_start {
        let mx = x_of(f);
        let color = if input.mix_active {
            theme::AMBER
        } else {
            theme::NEUTRAL_600.gamma_multiply(0.6)
        };
        let mut y = inner.top();
        while y < inner.bottom() {
            painter.rect_filled(Rect::from_min_size(pos2(mx, y), vec2(1.0, 2.0)), 0.0, color);
            y += 4.0;
        }
        let tag = Rect::from_min_size(pos2(mx - 22.0, inner.bottom() - 13.0), vec2(22.0, 11.0));
        painter.rect_filled(tag, 0.0, color);
        painter.text(
            tag.center(),
            Align2::CENTER_CENTER,
            input.mix_label,
            label_font,
            theme::MIX_TEXT,
        );
    }
    painter.rect_filled(
        Rect::from_min_size(pos2(play_x - 1.0, inner.top()), vec2(2.0, inner.height())),
        0.0,
        theme::TEXT,
    );
    // Alt (Option) is for marker editing: it never seeks or pans.
    let alt = ui.input(|i| i.modifiers.alt);
    let shielded = |p: Pos2| input.shield.is_some_and(|r| r.contains(p));
    let origin = ui.input(|i| i.pointer.press_origin());
    let panning =
        response.dragged_by(egui::PointerButton::Primary) && !alt && !origin.is_some_and(shielded);
    ui.data_mut(|d| {
        if panning {
            d.insert_temp(pan_id, true);
        } else {
            d.remove_temp::<bool>(pan_id);
        }
    });
    let pan_dx = if panning {
        response.drag_delta().x
    } else {
        0.0
    };
    let mut seek = None;
    // The hover line shows the time under the pointer; a drag pans instead.
    if !response.dragged()
        && let Some(p) = response.hover_pos().filter(|p| !shielded(*p))
    {
        let x = p.x;
        painter.rect_filled(
            Rect::from_min_size(pos2(x, inner.top()), vec2(1.0, inner.height())),
            0.0,
            theme::TEXT.gamma_multiply(0.6),
        );
        let text = format::clock(view.secs_at(x, inner));
        let tw = tabular_size(&painter, &text, &font(10.0)).x + 8.0;
        let lx = (x + 4.0).min(inner.right() - tw);
        let bg = Rect::from_min_size(pos2(lx, inner.top() + 13.0), vec2(tw, 14.0));
        painter.rect_filled(bg, 0.0, theme::NEUTRAL_800);
        paint_tabular(
            &painter,
            pos2(lx + 4.0, bg.top() + 1.0),
            &text,
            &font(10.0),
            theme::TEXT,
        );
        // A click seeks. On a stopped player it sets where Play starts the
        // next track (rule 3a); the CUE window seeks the CUE.
        if response.clicked() && !alt {
            seek = Some(view.secs_at(p.x, inner));
        }
    }
    WaveOutput {
        response,
        seek,
        pan_dx,
    }
}

/// How dark the trimmed head and tail of the waveform are drawn.
const TRIMMED_DIM: f32 = 0.45;

/// How the waveform draws the cue-in and cue-out edges.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CueEdgeLook {
    /// Shade the trimmed head and tail (playback skips them).
    pub shade_trimmed: bool,
    /// Opacity of the edge lines.
    pub line_alpha: f32,
}

/// Cue edges are plain while players use them; while ignored the whole file
/// plays, so nothing is shaded and the two lines are only dimmed marks.
pub fn cue_edge_look(ignored: bool) -> CueEdgeLook {
    if ignored {
        CueEdgeLook {
            shade_trimmed: false,
            line_alpha: theme::CUE_EDGE_IGNORED_ALPHA,
        }
    } else {
        CueEdgeLook {
            shade_trimmed: true,
            line_alpha: 1.0,
        }
    }
}

/// The width of an intro / outro badge: the value box is sized for `00.0`
/// (feedback 2 spec O33), so the badge does not change width from 9.9 to 10.0.
pub fn time_badge_width(painter: &Painter, caption: &str, value: &str) -> f32 {
    let cap = painter.layout_no_wrap(caption.to_owned(), font_semibold(9.0), Color32::WHITE);
    let big = font_medium(22.0);
    let value_w = tabular_size(painter, value, &big)
        .x
        .max(tabular_size(painter, "00.0", &big).x);
    cap.size().x + 6.0 + value_w + 16.0
}

/// A small outlined badge with a caption and a big number (intro / outro).
pub fn time_badge(
    painter: &Painter,
    anchor: egui::Pos2,
    right_aligned: bool,
    caption: &str,
    value: &str,
    colors: (Color32, Color32, Color32),
) {
    let (fill, border, text) = colors;
    let cap = painter.layout_no_wrap(caption.to_owned(), font_semibold(9.0), text);
    let big = font_medium(22.0);
    let width = time_badge_width(painter, caption, value);
    let height = tabular_size(painter, value, &big).y + 4.0;
    let left = if right_aligned {
        anchor.x - width
    } else {
        anchor.x
    };
    let rect = Rect::from_min_size(pos2(left, anchor.y), vec2(width, height));
    painter.rect_filled(rect, 0.0, fill);
    painter.rect_stroke(rect, 0.0, Stroke::new(1.0, border), StrokeKind::Inside);
    let baseline = rect.bottom() - 2.0;
    painter.galley(
        pos2(rect.left() + 8.0, baseline - cap.size().y - 3.0),
        cap.clone(),
        text,
    );
    paint_tabular(
        painter,
        pos2(rect.left() + 8.0 + cap.size().x + 6.0, rect.top() + 2.0),
        value,
        &big,
        text,
    );
}
