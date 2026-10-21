//! The main window's panels (resizable layout spec RL11–RL18): the players'
//! row with its splitters, the top bar's panel tiles and layout menu, the
//! headers' menu, and the view state the layout keeps. The pixel maths is
//! `panel_layout`.

use std::collections::HashMap;

use egui::{Align, Layout, Sense, TextureHandle, Ui, UiBuilder, vec2};
use fp_model::TrackId;

use super::app::{Scene, ViewState};
use super::panel_layout::{Arrangement, PANEL_GAP};
use super::player;

/// The shown players side by side (RL11), each at its width from the pass;
/// the row scrolls horizontally only when the minimums alone do not fit.
pub(crate) fn players_row(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    covers: &mut HashMap<TrackId, TextureHandle>,
    arranged: &Arrangement,
) {
    let widths = arranged.widths.clone();
    if widths.is_empty() {
        return;
    }
    // The horizontal scroll bar gets its own strip instead of covering the footers.
    let bar = if arranged.overflow {
        ui.spacing().scroll.bar_width + ui.spacing().scroll.bar_inner_margin + 2.0
    } else {
        0.0
    };
    let height = ui.available_height() - bar;
    egui::ScrollArea::horizontal()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing = vec2(PANEL_GAP, 0.0);
                for (id, width) in &widths {
                    let (rect, _) =
                        ui.allocate_exact_size(vec2(*width, height - 2.0), Sense::hover());
                    view_state.player_rects.push((*id, rect));
                    if !ui.is_rect_visible(rect) {
                        continue;
                    }
                    let Ok(index) = scene.state.player_index(*id) else {
                        continue;
                    };
                    // RL11: the id comes from the player, not from its place
                    // among the shown ones; its label stays `P<n>` (`index`).
                    let mut column_ui = ui.new_child(
                        UiBuilder::new()
                            .max_rect(rect)
                            .id_salt(("player", id.0))
                            .layout(Layout::top_down(Align::Min)),
                    );
                    column_ui.set_clip_rect(rect.intersect(ui.clip_rect()));
                    player::column(&mut column_ui, scene, view_state, covers, index);
                }
            });
        });
}
