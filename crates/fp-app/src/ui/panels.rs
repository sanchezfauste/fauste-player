//! The main window's panels (resizable layout spec RL11–RL18): the players'
//! row with its splitters, the top bar's panel tiles and layout menu, the
//! headers' menu, and the view state the layout keeps. The pixel maths is
//! `panel_layout`.

use std::collections::HashMap;

use egui::{Align, Layout, Rect, Sense, TextureHandle, Ui, UiBuilder, pos2, vec2};
use fp_model::layout::{Dock, Panel};
use fp_model::{Command, PlayerId, TrackId};

use super::app::{Scene, ViewState};
use super::panel_layout::{self, Arrangement, LiveSplit, PANEL_GAP};
use super::player;
use super::theme;
use super::widgets::{self, TileStyle, font};

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

/// RL15: on the left of the top bar, one tile per player (`P1` … `Pn`) and
/// one for the cartwall (`Carts`), lit while the panel is shown, then the
/// layout menu.
pub(crate) fn layout_controls(ui: &mut Ui, scene: &Scene<'_>) {
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing = vec2(4.0, 0.0);
        let t = scene.i18n;
        for (n, id) in scene.state.player_ids().into_iter().enumerate() {
            let name = t.tr_args("player-label", &[("n", (n + 1).into())]);
            panel_tile(ui, scene, Panel::Player(id), &name);
        }
        panel_tile(ui, scene, Panel::Cartwall, &t.tr("layout-panel-carts"));
        ui.add_space(4.0);
        layout_menu(ui, scene);
    });
}

/// One panel tile: a click hides or shows the panel (RL7). Its label says
/// what a click does; a hidden panel on air shows a red dot. Disabled for
/// the last shown panel and while the layout is locked.
fn panel_tile(ui: &mut Ui, scene: &Scene<'_>, panel: Panel, name: &str) {
    let t = scene.i18n;
    let layout = &scene.state.layout;
    let players = scene.state.player_ids();
    let shown = !layout.is_hidden(panel);
    let on_air = !shown && panel_layout::panel_on_air(scene.state, panel);
    let last = layout.is_last_shown(&players, panel);
    let enabled = !layout.locked && !last;
    let key = match (shown, on_air) {
        (true, _) => "tip-layout-hide",
        (false, true) => "tip-layout-show-on-air",
        (false, false) => "tip-layout-show",
    };
    let label = t.tr_args(key, &[("panel", name.to_owned().into())]);
    let style = if shown {
        TileStyle {
            fill: theme::NEUTRAL_800,
            border: theme::NEUTRAL_700,
            content: theme::TEXT,
            hover_fill: theme::NEUTRAL_700,
            ..TileStyle::plain()
        }
    } else {
        TileStyle {
            border: theme::NEUTRAL_800,
            content: theme::NEUTRAL_500,
            hover_fill: theme::NEUTRAL_800,
            ..TileStyle::plain()
        }
    };
    let width = ui
        .painter()
        .layout_no_wrap(name.to_owned(), font(11.0), theme::TEXT)
        .size()
        .x
        + 18.0;
    let response = widgets::tile_without_tooltip(
        ui,
        vec2(width.max(30.0), 24.0),
        &label,
        enabled,
        style,
        |p, r, c| {
            p.text(r.center(), egui::Align2::CENTER_CENTER, name, font(11.0), c);
            if on_air {
                p.rect_filled(
                    Rect::from_min_size(pos2(r.right() - 8.0, r.top() + 2.0), vec2(6.0, 6.0)),
                    0.0,
                    theme::ON_AIR_TEXT,
                );
            }
        },
    );
    let tip = if layout.locked {
        t.tr("error-layout-locked")
    } else if last {
        t.tr("error-last-panel")
    } else {
        label
    };
    if response.on_hover_text(tip).clicked() {
        scene.ctl.send(Command::SetPanelHidden {
            panel,
            hidden: shown,
        });
    }
}

fn dock_key(dock: Dock) -> &'static str {
    match dock {
        Dock::Bottom => "layout-dock-bottom",
        Dock::Left => "layout-dock-left",
        Dock::Right => "layout-dock-right",
    }
}

/// RL15: the layout button and its menu: the cartwall's dock, Lock layout,
/// Equal player widths and Reset layout. While the layout is locked every
/// entry but Lock layout is disabled.
fn layout_menu(ui: &mut Ui, scene: &Scene<'_>) {
    let t = scene.i18n;
    let label = t.tr("tip-layout-menu");
    let style = TileStyle {
        border: theme::NEUTRAL_800,
        hover_fill: theme::NEUTRAL_800,
        ..TileStyle::plain()
    };
    let response = widgets::tile(ui, vec2(24.0, 24.0), &label, true, style, |p, r, c| {
        p.text(
            r.center(),
            egui::Align2::CENTER_CENTER,
            egui_phosphor::regular::LAYOUT,
            font(14.0),
            c,
        );
    });
    egui::Popup::menu(&response).show(|ui| {
        ui.set_min_width(220.0);
        let layout = &scene.state.layout;
        let locked = layout.locked;
        for dock in Dock::ALL {
            let checked = layout.cartwall.dock == dock;
            let radio = egui::RadioButton::new(checked, t.tr(dock_key(dock)));
            if ui.add_enabled(!locked, radio).clicked() && !checked {
                scene.ctl.send(Command::SetCartwallDock(dock));
                ui.close();
            }
        }
        ui.separator();
        let mut lock = locked;
        if ui.checkbox(&mut lock, t.tr("layout-lock")).changed() {
            scene.ctl.send(Command::SetLayoutLocked(lock));
        }
        ui.separator();
        let equal = egui::Button::new(t.tr("layout-equal-widths"));
        if ui.add_enabled(!locked, equal).clicked() {
            scene.ctl.send(Command::EqualPlayerWidths);
            ui.close();
        }
        let reset = egui::Button::new(t.tr("layout-reset"));
        if ui.add_enabled(!locked, reset).clicked() {
            scene.ctl.send(Command::ResetLayout);
            ui.close();
        }
    });
}
