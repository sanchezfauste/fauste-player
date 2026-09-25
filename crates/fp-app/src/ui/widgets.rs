//! Custom-drawn parts of a player column: tile buttons, the VU meter, the
//! volume fader and the waveform.

use egui::{
    Align2, Color32, FontFamily, FontId, Painter, Rect, Response, Sense, Stroke, StrokeKind, Ui,
    Vec2, WidgetInfo, WidgetType, pos2, vec2,
};

use super::format;
use super::theme::{self, WaveColors};
use super::view::MarkerFractions;
use crate::services::TrackMedia;

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

/// Meter range: the bottom segment lights at this level.
const VU_FLOOR_DB: f32 = -48.0;
const VU_SEGMENTS: usize = 20;

/// Linear peak to meter fraction (0..1).
pub fn vu_fraction(peak: f32) -> f32 {
    if peak <= 0.0 || !peak.is_finite() {
        return 0.0;
    }
    let db = 20.0 * peak.log10();
    ((db - VU_FLOOR_DB) / -VU_FLOOR_DB).clamp(0.0, 1.0)
}

/// Stereo meter: 20 segments per channel, green/yellow/red, with peak hold.
pub fn vu(ui: &mut Ui, levels: [f32; 2], holds: [f32; 2]) {
    let (rect, _) = ui.allocate_exact_size(vec2(30.0, 64.0), Sense::hover());
    if !ui.is_rect_visible(rect) {
        return;
    }
    let painter = ui.painter();
    let seg_h = rect.height() / VU_SEGMENTS as f32;
    for (ch, (level, hold)) in levels.iter().zip(holds).enumerate() {
        let x = rect.left() + ch as f32 * 16.0;
        let lit = (level * VU_SEGMENTS as f32).round() as usize;
        let peak = (hold * VU_SEGMENTS as f32).round() as usize;
        for i in 0..VU_SEGMENTS {
            let on = i < lit || (peak > 0 && i == peak - 1);
            let colour = if i >= VU_SEGMENTS - 3 {
                theme::VU_RED
            } else if i >= VU_SEGMENTS - 7 {
                theme::VU_YELLOW
            } else {
                theme::VU_GREEN
            };
            let fill = if on {
                colour
            } else {
                theme::NEUTRAL_800.gamma_multiply(0.55)
            };
            let top = rect.bottom() - (i + 1) as f32 * seg_h + 1.0;
            painter.rect_filled(
                Rect::from_min_size(pos2(x, top), vec2(14.0, seg_h - 1.5)),
                0.0,
                fill,
            );
        }
    }
}

/// Vertical volume fader (drag or wheel). Returns the new fader position
/// (0 bottom … 1 top) when the user moved it.
pub fn fader(ui: &mut Ui, position: f32, label: &str) -> Option<f32> {
    let (rect, response) = ui.allocate_exact_size(vec2(22.0, 64.0), Sense::click_and_drag());
    let owned = label.to_owned();
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Slider, true, owned.clone()));
    let mut changed = None;
    if (response.dragged() || response.clicked())
        && let Some(p) = response.interact_pointer_pos()
    {
        changed = Some((1.0 - (p.y - rect.top()) / rect.height()).clamp(0.0, 1.0));
    }
    if response.hovered() {
        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll != 0.0 {
            changed = Some((position + scroll.signum() * 0.05).clamp(0.0, 1.0));
        }
    }
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

/// What the waveform shows.
pub struct WaveInput<'a> {
    pub media: Option<&'a TrackMedia>,
    pub total: Option<f64>,
    pub markers: MarkerFractions,
    pub colors: WaveColors,
    /// The MIX marker is drawn solid amber in Continuous mode, dim otherwise.
    pub mix_active: bool,
    pub mix_label: &'a str,
    pub accessible_label: &'a str,
}

/// Draws the waveform; returns the seek target (seconds) on click.
pub fn waveform(ui: &mut Ui, height: f32, input: &WaveInput<'_>) -> (Response, Option<f64>) {
    let size = vec2(ui.available_width(), height);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
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
    let mid = inner.center().y;
    let w = inner.width();
    let x_of = |f: f32| inner.left() + f * w;
    painter.rect_filled(
        Rect::from_min_size(pos2(inner.left(), mid), vec2(w, 1.0)),
        0.0,
        theme::NEUTRAL_800,
    );
    let Some(total) = input.total.filter(|t| *t > 0.0) else {
        return (response, None);
    };
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
        let peaks = &media.peaks;
        let covered = media.peak_bucket_secs * peaks.len() as f64;
        let span = covered.max(total);
        let amp = (inner.height() / 2.0 - 3.0).max(1.0);
        let mut x = 0.0_f32;
        while x < w {
            let t0 = f64::from(x / w) * span;
            let t1 = f64::from((x + 2.0) / w) * span;
            let a0 = (t0 / media.peak_bucket_secs) as usize;
            let a1 = ((t1 / media.peak_bucket_secs) as usize).max(a0 + 1);
            let level = peaks
                .get(a0.min(peaks.len())..a1.min(peaks.len()))
                .unwrap_or_default()
                .iter()
                .map(|(lo, hi)| lo.unsigned_abs().max(hi.unsigned_abs()))
                .max()
                .unwrap_or(0);
            let a = f32::from(level) / f32::from(i16::MAX as u16) * amp;
            let px = inner.left() + x;
            let color = if px < play_x {
                input.colors.played
            } else {
                input.colors.unplayed
            };
            painter.rect_filled(
                Rect::from_min_size(pos2(px, mid - a), vec2(1.0, (a * 1.96).max(1.0))),
                0.0,
                color,
            );
            x += 2.0;
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
    let mut seek = None;
    if let Some(p) = response.hover_pos() {
        let f = ((p.x - inner.left()) / w).clamp(0.0, 1.0);
        painter.rect_filled(
            Rect::from_min_size(pos2(p.x, inner.top()), vec2(1.0, inner.height())),
            0.0,
            theme::TEXT.gamma_multiply(0.6),
        );
        let text = format::clock(f64::from(f) * total);
        let galley = painter.layout_no_wrap(text, font(10.0), theme::TEXT);
        let tw = galley.size().x + 8.0;
        let lx = (p.x + 4.0).min(inner.right() - tw);
        let bg = Rect::from_min_size(pos2(lx, inner.top() + 13.0), vec2(tw, 14.0));
        painter.rect_filled(bg, 0.0, theme::NEUTRAL_800);
        painter.galley(pos2(lx + 4.0, bg.top() + 1.0), galley, theme::TEXT);
        if response.clicked() {
            seek = Some(f64::from(f) * total);
        }
    }
    (response, seek)
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
    let val = painter.layout_no_wrap(value.to_owned(), font_medium(22.0), text);
    let width = cap.size().x + 6.0 + val.size().x + 16.0;
    let height = val.size().y + 4.0;
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
    painter.galley(
        pos2(rect.left() + 8.0 + cap.size().x + 6.0, rect.top() + 2.0),
        val,
        text,
    );
}
