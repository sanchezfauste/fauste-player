//! The close guard (feedback 2 spec O6): closing the window while audio is
//! on air waits for the operator, who sees what will stop.

use egui::{RichText, vec2};
use fp_model::{AppState, Command, OnAir};

use super::app::Scene;
use super::theme;
use super::widgets::{self, TileStyle, font, font_medium};

/// What the operator asked for and the guard is confirming.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitIntent {
    /// Close the window and quit.
    Close,
}

/// The commands that stop everything in `items`: each player, then all
/// carts at once.
pub(crate) fn stop_commands(items: &[OnAir]) -> Vec<Command> {
    let mut commands: Vec<Command> = items
        .iter()
        .filter_map(|item| match item {
            OnAir::Player { player, .. } => Some(Command::Stop(*player)),
            OnAir::Cart(_) => None,
        })
        .collect();
    if items.iter().any(|i| matches!(i, OnAir::Cart(_))) {
        commands.push(Command::StopAllCarts);
    }
    commands
}

fn describe(scene: &Scene<'_>, state: &AppState, item: &OnAir) -> String {
    let t = scene.i18n;
    match item {
        OnAir::Player { player, entry } => {
            let n = state
                .players
                .iter()
                .position(|p| p.id == *player)
                .map_or(0, |i| i + 1);
            let title = entry
                .and_then(|e| state.track_for_entry(e))
                .map(|track| track.title.clone())
                .filter(|title| !title.is_empty());
            match title {
                Some(title) => {
                    t.tr_args("on-air-player", &[("n", n.into()), ("title", title.into())])
                }
                None => t.tr_args("on-air-player-empty", &[("n", n.into())]),
            }
        }
        OnAir::Cart(id) => {
            let cart = state.cartwall.cart(*id);
            let title = cart
                .map(|c| c.name.clone())
                .filter(|name| !name.is_empty())
                .or_else(|| {
                    cart.and_then(|c| c.track)
                        .and_then(|track| state.library.get(track))
                        .map(|track| track.title.clone())
                })
                .unwrap_or_default();
            t.tr_args("on-air-cart", &[("title", title.into())])
        }
    }
}

fn button(ui: &mut egui::Ui, label: &str, border: Option<egui::Color32>) -> bool {
    let w = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font(12.0), theme::NEUTRAL_300)
        .size()
        .x
        + 32.0;
    let mut style = TileStyle::plain();
    if let Some(border) = border {
        style.border = border;
    }
    widgets::tile(ui, vec2(w, 28.0), label, true, style, |p, r, c| {
        p.text(
            r.center(),
            egui::Align2::CENTER_CENTER,
            label,
            font(12.0),
            c,
        );
    })
    .clicked()
}

/// Draws the guard. `Some(true)` is "stop and go on", `Some(false)` is
/// "cancel", `None` keeps it open.
pub(crate) fn show(
    ctx: &egui::Context,
    scene: &Scene<'_>,
    intent: ExitIntent,
    items: &[OnAir],
) -> Option<bool> {
    let t = scene.i18n;
    let (body, confirm) = match intent {
        ExitIntent::Close => ("exit-guard-close-body", "exit-guard-close-confirm"),
    };
    let width = (ctx.content_rect().width() - 48.0).clamp(280.0, 440.0);
    let mut answer = None;
    egui::Modal::new(egui::Id::new("exit-guard"))
        .frame(egui::Frame::new().fill(theme::SURFACE).inner_margin(20.0))
        .backdrop_color(theme::NEUTRAL_900.gamma_multiply(0.7))
        .show(ctx, |ui| {
            ui.set_width(width);
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
            ui.add(
                egui::Label::new(
                    RichText::new(t.tr("exit-guard-title"))
                        .font(font_medium(18.0))
                        .color(theme::TEXT),
                )
                .selectable(false),
            );
            ui.add(
                egui::Label::new(
                    RichText::new(t.tr(body))
                        .font(font(12.0))
                        .color(theme::NEUTRAL_300),
                )
                .selectable(false)
                .wrap(),
            );
            for item in items {
                ui.add(
                    egui::Label::new(
                        RichText::new(describe(scene, scene.state, item))
                            .font(font(12.0))
                            .color(theme::TEXT),
                    )
                    .selectable(false)
                    .truncate(),
                );
            }
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if button(ui, &t.tr("exit-guard-cancel"), None) {
                    answer = Some(false);
                }
                if button(ui, &t.tr(confirm), Some(theme::ON_AIR_ROW)) {
                    answer = Some(true);
                }
            });
        });
    answer
}

#[cfg(test)]
mod tests {
    use super::*;
    use fp_model::{CartId, PlayerId};

    #[test]
    fn stop_commands_stop_each_player_and_all_carts_once() {
        let p = PlayerId(1);
        let items = [
            OnAir::Player {
                player: p,
                entry: None,
            },
            OnAir::Cart(CartId(1)),
            OnAir::Cart(CartId(2)),
        ];
        assert_eq!(
            stop_commands(&items),
            vec![Command::Stop(p), Command::StopAllCarts]
        );
        assert!(stop_commands(&[]).is_empty());
    }
}
