//! The main window's panels (resizable layout spec RL11–RL18): the players'
//! row with its splitters, the top bar's panel tiles and layout menu, the
//! headers' menu, and the view state the layout keeps. The pixel maths is
//! `panel_layout`.

use std::collections::HashMap;

use egui::{Align, Layout, Rect, Sense, TextureHandle, Ui, UiBuilder, pos2, vec2};
use fp_model::{Command, PlayerId, TrackId};

use super::app::{Scene, ViewState};
use super::panel_layout::{self, Arrangement, LiveSplit, PANEL_GAP};
use super::player;

/// The shown players side by side (RL11), each at its width from the pass
/// or from a splitter's drag (RL12), with a splitter in each gap; the row
/// scrolls horizontally only when the minimums alone do not fit. `normal`
/// is the pass without a maximised panel: a drag starts from its widths.
pub(crate) fn players_row(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    covers: &mut HashMap<TrackId, TextureHandle>,
    arranged: &Arrangement,
    normal: &Arrangement,
) {
    let widths = live_widths(ui, scene, view_state, arranged);
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
                for (k, (id, width)) in widths.iter().enumerate() {
                    let (rect, _) =
                        ui.allocate_exact_size(vec2(*width, height - 2.0), Sense::hover());
                    view_state.player_rects.push((*id, rect));
                    if let Some((right, _)) = widths.get(k + 1) {
                        let gap = Rect::from_min_max(
                            pos2(rect.right(), rect.top()),
                            pos2(rect.right() + PANEL_GAP, rect.bottom()),
                        );
                        player_splitter(ui, scene, view_state, (*id, *right), gap, normal);
                    }
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

/// RL12: the widths to draw: the pass's, or a splitter drag's (or one just
/// released, until the model's echo). Called before the row is drawn, so a
/// drag shows in the same frame as the pointer move.
fn live_widths(
    ui: &Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    arranged: &Arrangement,
) -> Vec<(PlayerId, f32)> {
    let mut widths = arranged.widths.clone();
    let Some(live) = view_state.live_split.as_mut() else {
        return widths;
    };
    let shares = &scene.state.layout.player_shares;
    if live.is_dragging() {
        let (down, pointer) = ui.input(|i| (i.pointer.primary_down(), i.pointer.latest_pos()));
        if down {
            if let Some(pos) = pointer {
                live.follow(pos.x);
            }
        } else if let Some(command) =
            live.release(shares.clone(), scene.time + panel_layout::HOLD_SECS)
        {
            scene.ctl.send(command);
        }
    }
    let shown: Vec<PlayerId> = arranged.widths.iter().map(|(p, _)| *p).collect();
    if live.stale(&shown, arranged.players_area.width(), shares, scene.time) {
        view_state.live_split = None;
    } else {
        live.apply(&mut widths);
    }
    widths
}

/// RL12: a splitter's interaction in `gap`. While `active` it senses clicks
/// and drags (a drag starts past egui's threshold, so a click is never a
/// drag), shows `cursor` while hovered or `dragging`, and paints a line
/// along its middle then; at rest nothing is painted. Inactive (a locked
/// layout) it senses nothing and the pointer keeps the normal cursor:
/// `None`.
pub(crate) fn splitter(
    ui: &mut Ui,
    gap: Rect,
    id: egui::Id,
    cursor: egui::CursorIcon,
    active: bool,
    dragging: bool,
) -> Option<egui::Response> {
    if !active {
        return None;
    }
    let response = ui.interact(gap, id, Sense::click_and_drag());
    let hot = (response.hovered() && !ui.input(|i| i.pointer.any_down())) || dragging;
    if hot {
        ui.set_cursor_icon(cursor);
        let visuals = ui.visuals();
        let stroke = if dragging {
            visuals.widgets.active.bg_stroke
        } else {
            visuals.widgets.hovered.bg_stroke
        };
        let line = if gap.width() <= gap.height() {
            [
                pos2(gap.center().x, gap.top()),
                pos2(gap.center().x, gap.bottom()),
            ]
        } else {
            [
                pos2(gap.left(), gap.center().y),
                pos2(gap.right(), gap.center().y),
            ]
        };
        ui.painter().line_segment(line, stroke);
    }
    Some(response)
}

/// RL12: the splitter between two neighbouring shown players. A drag
/// starts a `LiveSplit` from the widths without a maximised panel; a
/// double-click gives every player the same width.
fn player_splitter(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    pair: (PlayerId, PlayerId),
    gap: Rect,
    normal: &Arrangement,
) {
    let dragging = view_state
        .live_split
        .as_ref()
        .is_some_and(|l| l.pair() == pair && l.is_dragging());
    let id = egui::Id::new(("player-splitter", pair.0.0, pair.1.0));
    let active = !scene.state.layout.locked;
    let Some(response) = splitter(
        ui,
        gap,
        id,
        egui::CursorIcon::ResizeColumn,
        active,
        dragging,
    ) else {
        return;
    };
    if response.double_clicked() {
        scene.ctl.send(Command::EqualPlayerWidths);
    }
    if response.drag_started() {
        let width = |p: PlayerId| {
            normal
                .widths
                .iter()
                .find(|(id, _)| *id == p)
                .map(|(_, w)| *w)
        };
        if let (Some(left), Some(right)) = (width(pair.0), width(pair.1)) {
            let start_x = ui
                .input(|i| i.pointer.press_origin())
                .map_or(gap.center().x, |origin| origin.x);
            let shown = normal.widths.iter().map(|(p, _)| *p).collect();
            view_state.live_split = Some(LiveSplit::new(
                pair,
                shown,
                normal.players_area.width(),
                start_x,
                (left, right),
            ));
        }
    }
}
