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
use super::widgets::{self, TileStyle, font, font_medium};

/// The popup's width.
const WIDTH: f32 = 300.0;

/// The measurement line of `track` ("Loudness: −14.2 LUFS, true peak …" or
/// "Loudness: not measured").
pub(crate) fn measurement(scene: &Scene<'_>, track: &Track) -> String {
    view::loudness_lines(track)
        .first()
        .map(|m| widgets::tr_message(scene.i18n, m))
        .unwrap_or_default()
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

/// The trim the slider shows: the stored one within ± `max_trim_db` (as
/// the gain rule limits it, N10); one that is not a number counts as none.
fn trim_shown(scene: &Scene<'_>, track: &Track) -> f32 {
    let max = max_trim(scene);
    let trim = track.loudness_edit.trim_db;
    if trim.is_finite() {
        trim.clamp(-max, max)
    } else {
        0.0
    }
}

fn max_trim(scene: &Scene<'_>) -> f32 {
    let max = scene.state.config.loudness.max_trim_db;
    if max.is_finite() { max.max(0.0) } else { 0.0 }
}

/// The Trim slider: −`max_trim_db` ..= +`max_trim_db` in 0.5 dB steps; a
/// double-click puts it back to 0 (N20). Dimmed while the track is
/// excluded.
pub(crate) fn trim(ui: &mut Ui, scene: &Scene<'_>, track: &Track) {
    let excluded = track.loudness_edit.excluded;
    let max = max_trim(scene);
    let mut value = trim_shown(scene, track);
    let label = scene.i18n.tr("loudness-trim");
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

/// N20: the Loudness… popup of the track the waveform menu chose. It holds
/// that track whatever the players show next, and closes with its button
/// or by itself once the track leaves the library.
pub(crate) fn popup(ctx: &egui::Context, scene: &Scene<'_>, view_state: &mut ViewState) {
    let Some(id) = view_state.loudness_popup else {
        return;
    };
    let Some(track) = scene.state.library.get(id) else {
        view_state.loudness_popup = None;
        return;
    };
    let t = scene.i18n;
    let mut close = false;
    egui::Window::new(t.tr("settings-tab-loudness"))
        .id(egui::Id::new("loudness-popup"))
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
            ui.set_width(WIDTH);
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
            ui.add(
                egui::Label::new(
                    RichText::new(measurement(scene, track))
                        .font(font(12.0))
                        .color(theme::NEUTRAL_400),
                )
                .selectable(false)
                .wrap(),
            );
            ui.horizontal(|ui| {
                exclude(ui, scene, track);
                ui.add(
                    egui::Label::new(
                        RichText::new(t.tr("loudness-exclude"))
                            .font(font(13.0))
                            .color(theme::TEXT),
                    )
                    .selectable(false),
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
        });
    if close {
        view_state.loudness_popup = None;
    }
}
