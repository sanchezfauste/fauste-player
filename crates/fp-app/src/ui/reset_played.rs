//! The confirmation of Reset played (feedback 2 spec O22): clearing the
//! played marks of a playlist waits for the operator.

use egui::{RichText, vec2};

use super::app::Scene;
use super::theme;
use super::widgets::{self, TileStyle, font};

fn button(ui: &mut egui::Ui, label: &str) -> bool {
    let w = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font(12.0), theme::NEUTRAL_300)
        .size()
        .x
        + 32.0;
    widgets::tile(
        ui,
        vec2(w, 28.0),
        label,
        true,
        TileStyle::plain(),
        |p, r, c| {
            p.text(
                r.center(),
                egui::Align2::CENTER_CENTER,
                label,
                font(12.0),
                c,
            );
        },
    )
    .clicked()
}

/// Draws the question. `Some(true)` is "reset", `Some(false)` is "cancel"
/// (the button, Esc or a click on the backdrop), `None` keeps it open.
pub(crate) fn show(ctx: &egui::Context, scene: &Scene<'_>) -> Option<bool> {
    let t = scene.i18n;
    let width = (ctx.content_rect().width() - 48.0).clamp(280.0, 380.0);
    let mut answer = None;
    let modal = egui::Modal::new(egui::Id::new("reset-played"))
        .frame(egui::Frame::new().fill(theme::SURFACE).inner_margin(20.0))
        .backdrop_color(theme::NEUTRAL_900.gamma_multiply(0.5))
        .show(ctx, |ui| {
            ui.set_width(width);
            ui.spacing_mut().item_spacing = vec2(8.0, 12.0);
            ui.add(
                egui::Label::new(
                    RichText::new(t.tr("reset-played-question"))
                        .font(font(13.0))
                        .color(theme::TEXT),
                )
                .selectable(false)
                .wrap(),
            );
            ui.horizontal(|ui| {
                if button(ui, &t.tr("reset-played-cancel")) {
                    answer = Some(false);
                }
                if button(ui, &t.tr("reset-played-confirm")) {
                    answer = Some(true);
                }
            });
        });
    if answer.is_none() && modal.should_close() {
        answer = Some(false);
    }
    answer
}
