//! The waveform panel (operator feedback 4, Q7): the waveform with its
//! zoom, pan, Full view button, intro and outro badges and marker editing,
//! shared by the player column and the CUE window. It reads and writes the
//! view state of its `WaveKey` and sends only marker commands, which act on
//! a track; the caller turns the seek it returns into `Seek` or `SeekCue`.

use std::sync::Arc;

use egui::{Rect, RichText, Stroke, Ui, UiBuilder, pos2, vec2};
use fp_model::{Command, EntryId, MarkerKind, TrackId};

use super::app::{Scene, ViewState};
use super::format;
use super::theme;
use super::view::MarkerFractions;
use super::wave_view::{WaveKey, WaveView, WaveZoom, min_span, wheel_notches};
use super::widgets::{self, TileStyle, font};
use crate::services::TrackMedia;

/// The intro and outro countdowns drawn over the waveform (spec §3 rules
/// 18 and 19).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct WaveBadges {
    /// Seconds left of the intro.
    pub intro: Option<f64>,
    /// `Some(visible)` while the intro badge blinks.
    pub intro_blink: Option<bool>,
    /// Seconds from the outro to the end.
    pub outro: Option<f64>,
}

/// What the panel shows and allows (Q7.1).
pub(crate) struct WavePanelInput<'a> {
    pub key: WaveKey,
    /// The entry shown: a zoom belongs to it.
    pub entry: Option<EntryId>,
    /// Its track: edited markers belong to it.
    pub track: Option<TrackId>,
    pub media: Option<&'a Arc<TrackMedia>>,
    pub total: Option<f64>,
    /// The markers and the position, as fractions of `total`.
    pub markers: MarkerFractions,
    /// The MIX marker is drawn solid amber (Continuous mode).
    pub mix_active: bool,
    /// While zoomed, the view follows the position after the grace.
    pub follow: bool,
    pub badges: WaveBadges,
    /// The right-click menu and Alt-drag edit the markers.
    pub editable_markers: bool,
    pub height: f32,
}

/// What a frame of the panel reports.
pub(crate) struct WavePanelOutput {
    /// A click's time; the caller sends `Seek` or `SeekCue`.
    pub seek: Option<f64>,
}

fn label_key(key: WaveKey) -> &'static str {
    match key {
        WaveKey::Player(_) => "tip-waveform",
        WaveKey::Cue(_) => "tip-cue-waveform",
    }
}

/// Draws the panel and handles its zoom, pan and marker editing.
pub(crate) fn show(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    input: &WavePanelInput<'_>,
) -> WavePanelOutput {
    let t = scene.i18n;
    let key = input.key;
    let total = input.total.filter(|t| *t > 0.0);
    // A zoom belongs to the entry it was made on.
    let mut zoom = view_state
        .wave_zoom
        .get(key, input.entry)
        .filter(|_| total.is_some());
    let wave_id = key.id();
    // While zoomed, follow the position once the operator's last move is
    // older than the grace (feedback spec F17), but never under a held drag.
    let dragging = widgets::pan_dragging(ui, wave_id)
        || view_state.marker_drag.is_some_and(|(k, _, _)| k == key);
    if let (Some(z), Some(total), Some(f)) = (zoom.as_mut(), total, input.markers.position) {
        let grace = scene.state.config.ui.follow_current_grace_secs;
        if dragging {
            z.moved_at = scene.time;
        } else if grace > 0.0 && scene.time - z.moved_at >= grace && input.follow {
            z.view = z.view.follow(f64::from(f) * total, total);
        }
    }
    let view = zoom.map(|z| z.view);
    // Where the Full view button goes while zoomed: no seek starts under it.
    let wave_rect = Rect::from_min_size(ui.cursor().min, vec2(ui.available_width(), input.height));
    let full_view_text = t.tr("wave-full-view");
    let full_view_width = ui
        .painter()
        .layout_no_wrap(full_view_text.clone(), font(10.0), theme::TEXT)
        .size()
        .x
        + 12.0;
    let button_rect = Rect::from_min_size(
        pos2(
            wave_rect.right() - 4.0 - full_view_width,
            wave_rect.top() + 4.0,
        ),
        vec2(full_view_width, 18.0),
    );
    let full_view_button = zoom.map(|_| button_rect);
    let mix_label = t.tr("mix-marker");
    let label = t.tr(label_key(key));
    let wave_input = widgets::WaveInput {
        id: wave_id,
        media: input.media,
        total: input.total,
        markers: input.markers,
        colors: theme::wave_colors(&scene.state.config.ui.wave_color),
        mix_active: input.mix_active,
        mix_label: &mix_label,
        accessible_label: &label,
        view,
        shield: full_view_button,
    };
    let output = widgets::waveform(ui, input.height, &wave_input);
    let (response, seek, pan_dx) = (output.response, output.seek, output.pan_dx);
    let rect = response.rect;
    if input.editable_markers
        && let (Some(track), Some(total)) = (input.track, total)
    {
        let shown = view.unwrap_or_else(|| WaveView::full(total));
        edit_markers(
            ui,
            scene,
            view_state,
            key,
            track,
            shown,
            total,
            input.markers,
            &response,
        );
    }
    // The wheel zooms around the pointer; Shift or a sideways wheel pans.
    if let (Some(entry), Some(total), Some(p)) = (input.entry, total, response.hover_pos()) {
        let inner = rect.shrink(1.0);
        let bucket = input.media.map_or(
            f64::from(scene.state.config.analysis.peak_bucket_ms) / 1000.0,
            |m| m.peak_bucket_secs,
        );
        let min = min_span(bucket, inner.width());
        let wheels: Vec<(egui::Vec2, bool)> = ui.input(|i| {
            i.events
                .iter()
                .filter_map(|e| match e {
                    egui::Event::MouseWheel {
                        unit,
                        delta,
                        modifiers,
                        ..
                    } if *delta != egui::Vec2::ZERO && !modifiers.command => {
                        Some((wheel_notches(*unit, *delta), modifiers.shift))
                    }
                    _ => None,
                })
                .collect()
        });
        if !wheels.is_empty() {
            let mut v = view.unwrap_or_else(|| WaveView::full(total));
            for (notches, shift) in wheels {
                v = v.wheel(notches, shift, p.x, inner, total, min);
            }
            zoom = (!v.is_full(total)).then_some(WaveZoom {
                view: v,
                entry,
                moved_at: scene.time,
            });
            // The wheel was for the waveform, not for a scroll area around it.
            ui.ctx()
                .input_mut(|i| i.smooth_scroll_delta = egui::Vec2::ZERO);
            ui.ctx().request_repaint();
        }
    }
    // Dragging pans a zoomed view; without zoom a drag does nothing (O10).
    if pan_dx != 0.0
        && let (Some(z), Some(total)) = (zoom.as_mut(), total)
    {
        z.view = z.view.pan(pan_dx, rect.shrink(1.0), total);
        z.moved_at = scene.time;
    }
    let painter = ui.painter_at(rect);
    let mut badge_right = rect.right() - 4.0;
    if zoom.is_some() {
        let button = button_rect;
        let text = full_view_text;
        let mut child = ui.new_child(UiBuilder::new().max_rect(button));
        if widgets::tile(
            &mut child,
            button.size(),
            &text,
            true,
            TileStyle::plain(),
            |p, r, c| {
                p.text(
                    r.center(),
                    egui::Align2::CENTER_CENTER,
                    &text,
                    font(10.0),
                    c,
                );
            },
        )
        .clicked()
        {
            zoom = None;
        }
        badge_right = button.left() - 4.0;
    }
    view_state.wave_zoom.set(key, zoom);
    if let Some(left) = input.badges.intro {
        let fill = if input.badges.intro_blink == Some(true) {
            theme::INTRO_BADGE_BLINK
        } else {
            theme::INTRO_BADGE_BG
        };
        widgets::time_badge(
            &painter,
            rect.left_top() + vec2(4.0, 4.0),
            false,
            &t.tr("wave-intro"),
            &format!("{left:.1}"),
            (
                fill.gamma_multiply(0.95),
                theme::INTRO,
                theme::INTRO_BADGE_TEXT,
            ),
        );
    }
    if let Some(left) = input.badges.outro {
        widgets::time_badge(
            &painter,
            pos2(badge_right, rect.top() + 4.0),
            true,
            &t.tr("wave-outro"),
            &format!("{left:.1}"),
            (
                theme::OUTRO_BADGE_BG.gamma_multiply(0.9),
                theme::OUTRO_LINE,
                theme::OUTRO_BADGE_TEXT,
            ),
        );
    }
    WavePanelOutput { seek }
}

/// Marker editing on the waveform (Phase 2 spec P2.8): a context menu that
/// places a marker where it was opened, and Alt-drag on marker handles.
#[allow(clippy::too_many_arguments)]
fn edit_markers(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    key: WaveKey,
    track: TrackId,
    view: WaveView,
    total: f64,
    markers: MarkerFractions,
    response: &egui::Response,
) {
    let t = scene.i18n;
    let inner = response.rect.shrink(1.0);
    let secs_at = |x: f32| view.secs_at(x, inner);
    let x_of = |f: f32| view.x_of(f64::from(f) * total, inner);
    let m = markers;
    let handles = [
        (MarkerKind::CueIn, m.cue_in),
        (MarkerKind::IntroEnd, m.intro_end),
        (MarkerKind::OutroStart, m.outro_start),
        (MarkerKind::SegueStart, m.segue_start),
        (MarkerKind::CueOut, m.cue_out),
    ];
    if response.secondary_clicked()
        && let Some(p) = response.interact_pointer_pos()
    {
        view_state.wave_menu.insert(key, secs_at(p.x));
    }
    let at = view_state.wave_menu.get(&key).copied();
    let open = response.context_menu(|ui| {
        ui.set_min_width(220.0);
        let item = |ui: &mut Ui, key: &str| {
            let text = t.tr(key);
            let r = ui.button(&text);
            r.widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::Button, true, text.clone())
            });
            r.clicked()
        };
        if let Some(secs) = at {
            ui.add(
                egui::Label::new(
                    RichText::new(format::clock(secs))
                        .font(font(11.0))
                        .color(theme::NEUTRAL_400),
                )
                .selectable(false),
            );
            ui.separator();
            for (kind, key) in [
                (MarkerKind::CueIn, "wave-set-cue-in"),
                (MarkerKind::IntroEnd, "wave-set-intro"),
                (MarkerKind::OutroStart, "wave-set-outro"),
                (MarkerKind::SegueStart, "wave-set-mix"),
                (MarkerKind::CueOut, "wave-set-cue-out"),
            ] {
                if item(ui, key) {
                    scene.ctl.send(Command::SetMarker {
                        track,
                        kind,
                        secs: Some(secs),
                    });
                    ui.close();
                }
            }
            ui.separator();
        }
        if item(ui, "wave-reset") {
            scene.ctl.send(Command::ResetMarkers { track });
            ui.close();
        }
    });
    // Once the menu is closed, the point it was opened at is forgotten.
    if open.is_none() && !response.secondary_clicked() {
        view_state.wave_menu.remove(&key);
    }
    let alt = ui.input(|i| i.modifiers.alt);
    // The drag starts once the pointer has moved; pick the marker under
    // the point where the button went down.
    if alt
        && response.drag_started_by(egui::PointerButton::Primary)
        && let Some(p) = ui.input(|i| i.pointer.press_origin())
    {
        // The nearest marker within reach of the pointer.
        let nearest = handles
            .iter()
            .filter_map(|(kind, f)| f.map(|f| (*kind, (x_of(f) - p.x).abs())))
            .filter(|(_, d)| *d <= 8.0)
            .min_by(|a, b| a.1.total_cmp(&b.1));
        view_state.marker_drag = nearest.map(|(kind, _)| (key, kind, track));
    }
    // A drag belongs to the track it started on: if the player moved on,
    // it is dropped.
    if view_state
        .marker_drag
        .is_some_and(|(k, _, t)| k == key && t != track)
    {
        view_state.marker_drag = None;
    }
    let dragging = view_state
        .marker_drag
        .filter(|(k, _, _)| *k == key)
        .map(|(_, k, _)| k);
    let painter = ui.painter_at(response.rect);
    if alt || dragging.is_some() {
        for (kind, f) in handles {
            if let Some(f) = f {
                let x = x_of(f);
                let active = dragging == Some(kind);
                let color = if active {
                    theme::TEXT
                } else {
                    theme::ACCENT_300
                };
                painter.add(egui::Shape::convex_polygon(
                    vec![
                        pos2(x - 5.0, inner.top()),
                        pos2(x + 5.0, inner.top()),
                        pos2(x, inner.top() + 7.0),
                    ],
                    color,
                    Stroke::NONE,
                ));
            }
        }
    }
    if let Some(kind) = dragging
        && let Some(p) = ui.ctx().pointer_latest_pos()
    {
        let x = p.x.clamp(inner.left(), inner.right());
        painter.rect_filled(
            Rect::from_min_size(pos2(x, inner.top()), vec2(1.0, inner.height())),
            0.0,
            theme::TEXT,
        );
        let drag_time = format::clock(secs_at(x));
        let drag_font = font(10.0);
        let drag_h = widgets::tabular_size(&painter, &drag_time, &drag_font).y;
        widgets::paint_tabular(
            &painter,
            pos2(x + 4.0, inner.bottom() - 4.0 - drag_h),
            &drag_time,
            &drag_font,
            theme::TEXT,
        );
        if response.drag_stopped() || !ui.input(|i| i.pointer.primary_down()) {
            scene.ctl.send(Command::SetMarker {
                track,
                kind,
                secs: Some(secs_at(x)),
            });
            view_state.marker_drag = None;
        }
    }
}
