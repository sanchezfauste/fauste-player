//! The per-track loudness controls (loudness normalization spec N20, N21):
//! the measurement, Exclude from normalization and the trim. The waveform's
//! Loudness… popup and the cart editor both draw them. They read only the
//! snapshot; every change is one `SetTrackLoudness`.

use egui::{RichText, Stroke, Ui, vec2};
use egui_phosphor::regular as icon;
use fp_model::{Command, Track, TrackId};

use super::app::{Scene, ViewState};
use super::theme;
use super::view;
use super::widgets::{self, TileStyle, font, font_medium, font_semibold};
use crate::i18n::I18n;

/// The popup's width.
const WIDTH: f32 = 300.0;

/// The gain information of `track` as the player it plays on gives it
/// (`holder`, when known): see [`view::gain_info`]. `on_air` is the gain a
/// source of this track on air started with.
pub(crate) fn info(
    scene: &Scene<'_>,
    track: &Track,
    holder: Option<fp_model::Holder>,
    on_air: Option<&fp_model::Normalization>,
) -> Vec<view::InfoLine> {
    let bit_perfect = holder.is_some_and(|h| fp_model::holder_bit_perfect(scene.state, h));
    view::gain_info(track, &scene.state.config.loudness, bit_perfect, on_air)
}

/// The gap between the label and the value column.
const GAP: f32 = 12.0;

/// Draws the gain information in the width the `ui` has: label on the left,
/// value on the right (tabular digits), notes across, and the resulting gain
/// last under a rule, in bold. Where a tooltip, the loudness editor and the
/// playlist row show a track's gain, this is what they draw. A label or a
/// value too long for its line wraps (the label to at most half the width).
pub(crate) fn info_block(ui: &mut Ui, t: &I18n, lines: &[view::InfoLine]) {
    let width = ui.available_width();
    for line in lines {
        let label = widgets::tr_message(t, &line.label);
        match line.kind {
            view::InfoKind::Note => {
                ui.add_space(2.0);
                ui.add(
                    egui::Label::new(
                        RichText::new(label)
                            .font(font(11.0))
                            .color(theme::NEUTRAL_500),
                    )
                    .selectable(false)
                    .wrap(),
                );
            }
            view::InfoKind::Result => {
                ui.add_space(3.0);
                let y = ui.cursor().top();
                ui.painter().hline(
                    ui.min_rect().x_range(),
                    y,
                    Stroke::new(1.0, theme::NEUTRAL_700),
                );
                ui.add_space(5.0);
                info_row(
                    ui,
                    width,
                    &label,
                    &widgets::info_value(t, &line.value),
                    true,
                );
            }
            _ => info_row(
                ui,
                width,
                &label,
                &widgets::info_value(t, &line.value),
                false,
            ),
        }
    }
}

/// A galley that breaks a word wider than `wrap` rather than overflow.
fn galley(
    painter: &egui::Painter,
    text: &str,
    font: egui::FontId,
    color: egui::Color32,
    wrap: f32,
) -> std::sync::Arc<egui::Galley> {
    let mut job = egui::text::LayoutJob::simple(text.to_owned(), font, color, wrap);
    job.wrap.break_anywhere = true;
    painter.layout_job(job)
}

fn info_row(ui: &mut Ui, width: f32, label: &str, value: &str, strong: bool) {
    let (font, label_color) = if strong {
        (font_semibold(13.0), theme::TEXT)
    } else {
        (font(12.0), theme::NEUTRAL_400)
    };
    let value_color = theme::TEXT;
    let painter = ui.painter().clone();
    let natural_label = painter
        .layout_no_wrap(label.to_owned(), font.clone(), label_color)
        .size()
        .x;
    let natural_value = widgets::tabular_size(&painter, value, &font).x;
    let one_line = natural_label + GAP + natural_value <= width;
    let label_wrap = if one_line {
        natural_label
    } else {
        natural_label.min(width * 0.5)
    };
    let label_galley = galley(&painter, label, font.clone(), label_color, label_wrap);
    // Where the label's longest word is wider than its column, the value
    // goes under it instead of beside it.
    let stacked = !one_line && label_galley.size().x > label_wrap + 0.5;
    let label_galley = if stacked {
        galley(&painter, label, font.clone(), label_color, width)
    } else {
        label_galley
    };
    let value_galley = (!one_line).then(|| {
        let wrap = if stacked {
            width
        } else {
            (width - label_galley.size().x - GAP).max(20.0)
        };
        galley(&painter, value, font.clone(), value_color, wrap)
    });
    let height = match (&value_galley, stacked) {
        (Some(v), true) => label_galley.size().y + v.size().y,
        (Some(v), false) => label_galley.size().y.max(v.size().y),
        (None, _) => label_galley
            .size()
            .y
            .max(widgets::tabular_size(&painter, value, &font).y),
    };
    let (rect, response) = ui.allocate_exact_size(vec2(width, height), egui::Sense::hover());
    let name = format!("{label}: {value}");
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Label, true, name.clone()));
    if !ui.is_rect_visible(rect) {
        return;
    }
    let label_height = label_galley.size().y;
    painter.galley(rect.left_top(), label_galley, label_color);
    match value_galley {
        Some(galley) => {
            let x = rect.right() - galley.size().x;
            let y = if stacked {
                rect.top() + label_height
            } else {
                rect.top()
            };
            painter.galley(egui::pos2(x, y), galley, value_color);
        }
        None => {
            widgets::paint_tabular_right(
                &painter,
                egui::pos2(rect.right(), rect.top()),
                value,
                &font,
                value_color,
            );
        }
    }
}

fn send(scene: &Scene<'_>, track: TrackId, excluded: bool, trim_db: f32) {
    scene.ctl.send(Command::SetTrackLoudness {
        track,
        excluded,
        trim_db,
    });
}

/// The Exclude from normalization switch; sends the change.
pub(crate) fn exclude(ui: &mut Ui, scene: &Scene<'_>, track: &Track) {
    let edit = track.loudness_edit;
    let mut excluded = edit.excluded;
    if widgets::toggle(ui, &mut excluded, &scene.i18n.tr("loudness-exclude")) {
        send(scene, track.id, excluded, trim_shown(scene, track));
    }
}

/// The trim the slider shows: the one in use (`fp_model::trim_in_use`,
/// as the gain rule limits it, N7).
fn trim_shown(scene: &Scene<'_>, track: &Track) -> f32 {
    fp_model::trim_in_use(&scene.state.config.loudness, &track.loudness_edit)
}

fn max_trim(scene: &Scene<'_>) -> f32 {
    fp_model::max_trim_in_use(&scene.state.config.loudness)
}

/// The Trim slider: −`max_trim_db` ..= +`max_trim_db` in 0.5 dB steps; a
/// double-click puts it back to 0 (N20). Dimmed while the track is
/// excluded.
pub(crate) fn trim(ui: &mut Ui, scene: &Scene<'_>, track: &Track) {
    let excluded = track.loudness_edit.excluded;
    let max = max_trim(scene);
    let mut value = trim_shown(scene, track);
    let label = scene.i18n.tr("loudness-trim");
    // The rail gives way in a narrow window, so the value box stays inside.
    let rail = (ui.available_width() - 64.0).clamp(30.0, ui.spacing().slider_width);
    ui.spacing_mut().slider_width = rail;
    let response = ui.add_enabled(
        !excluded,
        egui::Slider::new(&mut value, -max..=max)
            .step_by(0.5)
            .suffix(" dB"),
    );
    let v = f64::from(value);
    response.widget_info(|| egui::WidgetInfo::slider(!excluded, v, label.clone()));
    // The rail senses drags only: the double-click is read from the
    // pointer. egui counts a double-click less than 0.6 s after another
    // click (the one that opened the popup) as a triple click.
    let double_clicked = !excluded
        && response.contains_pointer()
        && ui.input(|i| {
            let b = egui::PointerButton::Primary;
            i.pointer.button_double_clicked(b) || i.pointer.button_triple_clicked(b)
        });
    if double_clicked {
        send(scene, track.id, excluded, 0.0);
    } else if response.changed() {
        send(scene, track.id, excluded, value);
    }
}

/// Whether a player that is playing or paused has `track` as its current
/// entry: a change of the track's loudness settings waits for its next load.
fn sounding(scene: &Scene<'_>, track: TrackId) -> bool {
    scene.state.players.iter().any(|p| {
        p.transport != fp_model::Transport::Stopped
            && p.current
                .and_then(|e| scene.state.track_for_entry(e))
                .is_some_and(|t| t.id == track)
    })
}

/// N20: the Loudness… popup of the track the waveform or row menu chose. It holds
/// that track whatever the players show next, and closes with its button
/// or by itself once the track leaves the library.
pub(crate) fn popup(ctx: &egui::Context, scene: &Scene<'_>, view_state: &mut ViewState) {
    let Some((id, holder)) = view_state.loudness_popup else {
        return;
    };
    let Some(track) = scene.state.library.get(id) else {
        view_state.loudness_popup = None;
        return;
    };
    let t = scene.i18n;
    let mut close = false;
    egui::Window::new(t.tr("settings-tab-loudness"))
        .id(egui::Id::unique("loudness-popup"))
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .default_pos(egui::pos2(120.0, 140.0))
        .frame(
            egui::Frame::new()
                .fill(theme::SURFACE)
                .stroke(Stroke::new(1.0, theme::NEUTRAL_700))
                .inner_margin(12.0),
        )
        .show(ctx, |ui| {
            ui.set_width(WIDTH.min((ctx.content_rect().width() - 40.0).max(150.0)));
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
            ui.horizontal(|ui| {
                ui.add(
                    egui::Label::new(
                        RichText::new(t.tr("settings-tab-loudness").to_uppercase())
                            .font(font(10.0))
                            .color(theme::ACCENT_300),
                    )
                    .selectable(false),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let label = t.tr("loudness-close");
                    close = widgets::tile(
                        ui,
                        vec2(22.0, 22.0),
                        &label,
                        true,
                        TileStyle::plain(),
                        |p, r, c| {
                            p.text(
                                r.center(),
                                egui::Align2::CENTER_CENTER,
                                icon::X,
                                font(12.0),
                                c,
                            );
                        },
                    )
                    .clicked();
                });
            });
            // The track it edits, which need not be the one on air.
            ui.add(
                egui::Label::new(
                    RichText::new(&track.title)
                        .font(font_medium(15.0))
                        .color(theme::TEXT),
                )
                .selectable(false)
                .truncate(),
            );
            info_block(ui, t, &info(scene, track, holder, None));
            ui.horizontal_wrapped(|ui| {
                exclude(ui, scene, track);
                ui.add(
                    egui::Label::new(
                        RichText::new(t.tr("loudness-exclude"))
                            .font(font(13.0))
                            .color(theme::TEXT),
                    )
                    .selectable(false)
                    .wrap(),
                );
            });
            ui.horizontal(|ui| {
                ui.add(
                    egui::Label::new(
                        RichText::new(t.tr("loudness-trim"))
                            .font(font(13.0))
                            .color(theme::TEXT),
                    )
                    .selectable(false),
                );
                trim(ui, scene, track);
            });
            ui.add(
                egui::Label::new(
                    RichText::new(t.tr("loudness-trim-hint"))
                        .font(font(11.0))
                        .color(theme::NEUTRAL_500),
                )
                .selectable(false)
                .wrap(),
            );
            if sounding(scene, id) {
                ui.add(
                    egui::Label::new(
                        RichText::new(t.tr("loudness-trim-next-load"))
                            .font(font(12.0))
                            .color(theme::NEUTRAL_400),
                    )
                    .selectable(false)
                    .wrap(),
                );
            }
        });
    if close {
        view_state.loudness_popup = None;
    }
}
