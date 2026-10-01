//! The notice after an update (plan 4, item 16): tracks an earlier version
//! analysed keep their analysis until the operator asks for a new one,
//! here at start-up or later in Settings > Analysis.

use egui::{RichText, vec2};

use super::app::Scene;
use super::theme;
use super::widgets::{self, TileStyle, font, font_medium};

/// What the operator chose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Answer {
    AnalyseNow,
    Later,
}

/// A button as wide as its label; `accent` for the main action.
fn button(ui: &mut egui::Ui, text: &str, accent: bool) -> bool {
    let w = ui
        .painter()
        .layout_no_wrap(text.to_owned(), font_medium(13.0), theme::TEXT)
        .size()
        .x
        + 28.0;
    let style = if accent {
        TileStyle {
            fill: theme::ACCENT,
            border: theme::ACCENT,
            content: theme::NEUTRAL_900,
            hover_fill: theme::ACCENT_400,
            hover_content: theme::NEUTRAL_900,
            active_fill: theme::ACCENT_300,
            ..TileStyle::plain()
        }
    } else {
        TileStyle::plain()
    };
    widgets::tile(ui, vec2(w, 30.0), text, true, style, |p, r, c| {
        p.text(
            r.center(),
            egui::Align2::CENTER_CENTER,
            text,
            font_medium(13.0),
            c,
        );
    })
    .clicked()
}

/// Draws the notice for `count` outdated tracks; `Some` once answered.
pub(crate) fn show(ctx: &egui::Context, scene: &Scene<'_>, count: usize) -> Option<Answer> {
    let t = scene.i18n;
    let mut answer = None;
    let width = (ctx.content_rect().width() - 48.0).clamp(320.0, 480.0);
    egui::Modal::new(egui::Id::new("outdated-analysis"))
        .frame(egui::Frame::new().fill(theme::SURFACE).inner_margin(20.0))
        .backdrop_color(theme::NEUTRAL_900.gamma_multiply(0.7))
        .show(ctx, |ui| {
            ui.set_width(width);
            ui.spacing_mut().item_spacing = vec2(8.0, 10.0);
            ui.add(
                egui::Label::new(
                    RichText::new(t.tr("outdated-title"))
                        .font(font_medium(16.0))
                        .color(theme::TEXT),
                )
                .selectable(false),
            );
            ui.add(
                egui::Label::new(
                    RichText::new(t.tr_args("outdated-body", &[("count", count.into())]))
                        .font(font(13.0))
                        .color(theme::NEUTRAL_300),
                )
                .wrap()
                .selectable(false),
            );
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if button(ui, &t.tr("outdated-now"), true) {
                    answer = Some(Answer::AnalyseNow);
                }
                if button(ui, &t.tr("outdated-later"), false) {
                    answer = Some(Answer::Later);
                }
            });
        });
    answer
}
