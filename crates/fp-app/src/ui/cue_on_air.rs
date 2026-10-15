//! Pre-listen on an on-air output (on-air pre-listen spec §5): the
//! confirmation a CA3 request asks for, and the notice CA5 leaves. The
//! rules are `fp_model::cue_on_air`; this module only describes and asks.

use egui::{RichText, vec2};
use fp_model::{AppState, CueConfirmation, CueStopped, Holder, Transport};

use super::app::Scene;
use super::exit_guard::button;
use super::pending::player_number;
use super::theme;
use super::widgets::{font, font_medium};
use crate::i18n::I18n;

/// What `holder` is doing on the output, for the question's body.
fn sounding_text(t: &I18n, state: &AppState, holder: Holder) -> String {
    let n = |p| player_number(state, p);
    match holder {
        Holder::PlayerMain(p) => {
            let paused = state
                .player(p)
                .is_ok_and(|pl| pl.transport == Transport::Paused);
            let key = if paused {
                "cue-on-air-player-paused"
            } else {
                "cue-on-air-player-playing"
            };
            t.tr_args(key, &[("n", n(p).into())])
        }
        Holder::PlayerCue(p) => t.tr_args("cue-on-air-player-cue", &[("n", n(p).into())]),
        Holder::CartwallMain => t.tr("cue-on-air-cartwall"),
        Holder::CartwallCue => t.tr("cue-on-air-cartwall-cue"),
    }
}

/// CA5's notice: who started on the pre-listen's output.
pub(crate) fn notice_text(t: &I18n, state: &AppState, stopped: &CueStopped) -> String {
    match stopped.by {
        Holder::PlayerMain(p) | Holder::PlayerCue(p) => t.tr_args(
            "cue-on-air-stopped-player",
            &[("n", player_number(state, p).into())],
        ),
        Holder::CartwallMain | Holder::CartwallCue => t.tr("cue-on-air-stopped-cartwall"),
    }
}

/// The question for `request`. `Some(true)` is *Pre-listen anyway*,
/// `Some(false)` is *Cancel* (the button or the backdrop; Esc and Enter are
/// answered by `AppUi::keyboard`), `None` keeps it open.
pub(crate) fn show(
    ctx: &egui::Context,
    scene: &Scene<'_>,
    request: &CueConfirmation,
) -> Option<bool> {
    let (t, state) = (scene.i18n, scene.state);
    let sounding = request
        .sounding
        .iter()
        .map(|h| sounding_text(t, state, *h))
        .collect::<Vec<_>>()
        .join(", ");
    let body = t.tr_args("cue-on-air-body", &[("sounding", sounding.into())]);
    let width = (ctx.content_rect().width() - 48.0).clamp(280.0, 440.0);
    let mut answer = None;
    let modal = egui::Modal::new(egui::Id::new("cue-on-air"))
        .frame(egui::Frame::new().fill(theme::SURFACE).inner_margin(20.0))
        .backdrop_color(theme::NEUTRAL_900.gamma_multiply(0.7))
        .show(ctx, |ui| {
            ui.set_width(width);
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
            ui.add(
                egui::Label::new(
                    RichText::new(t.tr("cue-on-air-title"))
                        .font(font_medium(18.0))
                        .color(theme::TEXT),
                )
                .selectable(false),
            );
            ui.add(
                egui::Label::new(
                    RichText::new(body.clone())
                        .font(font(12.0))
                        .color(theme::NEUTRAL_300),
                )
                .selectable(false)
                .wrap(),
            );
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if button(ui, &t.tr("exit-guard-cancel"), None) {
                    answer = Some(false);
                }
                if button(ui, &t.tr("cue-on-air-confirm"), Some(theme::ON_AIR_ROW)) {
                    answer = Some(true);
                }
            });
        });
    if answer.is_none() && modal.should_close() {
        answer = Some(false);
    }
    answer
}
