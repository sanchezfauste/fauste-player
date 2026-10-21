//! The main window's panels (resizable layout spec RL11–RL18): the players'
//! row with its splitters, the top bar's panel tiles and layout menu, the
//! headers' menu, and the view state the layout keeps. The pixel maths is
//! `panel_layout`.

use std::collections::HashMap;

use egui::{Align, Layout, Rect, Sense, TextureHandle, Ui, UiBuilder, pos2, vec2};
use fp_model::layout::{DEFAULT_SIDE_SHARE, Dock, Panel};
use fp_model::{AppState, Command, PlayerId, TrackId};

use super::app::{Scene, ViewState};
use super::panel_layout::{
    self, Arrangement, CartwallDrag, LiveSplit, Maximised, PANEL_GAP, PassInput,
};
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
    // The horizontal scroll bar gets its own strip instead of covering the
    // footers; the pass keeps that strip out of the players' least height.
    let bar = if arranged.overflow {
        panel_layout::SCROLL_BAR_ROOM
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
    // Not focusable: a splitter is no Tab stop (it has no keyboard use).
    let response = ui.interact(gap, id, Sense::CLICK | Sense::DRAG);
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
        // RL16: like a drag, it first ends a maximised panel.
        view_state.maximised = None;
        scene.ctl.send(Command::EqualPlayerWidths);
    }
    if response.drag_started_by(egui::PointerButton::Primary) {
        // RL16: a drag first ends a maximised panel, then applies itself to
        // the sizes from before (`normal`).
        view_state.maximised = None;
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

/// What every top-bar tile reads, worked out once per frame.
struct TileContext {
    /// How many panels are shown (`Layout::shown_panels`).
    shown: usize,
}

/// The room the layout button takes right of the tiles: its gap and its
/// 24-point tile.
const LAYOUT_BUTTON_ROOM: f32 = 4.0 + 4.0 + 24.0;

/// RL15: on the left of the top bar, one tile per player (`P1` … `Pn`) and
/// one for the cartwall, lit while the panel is shown, then the layout
/// menu. The tiles never reach `right` (where the clock and the buttons on
/// the right begin): past it they scroll sideways, and the layout button
/// stays outside the scroll, always reachable.
pub(crate) fn layout_controls(ui: &mut Ui, scene: &Scene<'_>, right: f32) {
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing = vec2(4.0, 0.0);
        let t = scene.i18n;
        let players = scene.state.player_ids();
        let context = TileContext {
            shown: scene.state.layout.shown_panels(&players),
        };
        let room = (right - ui.cursor().left() - LAYOUT_BUTTON_ROOM).max(0.0);
        // An explicit salt: the players' row's scroll area is another child
        // of the root, and the two must not share their state.
        egui::ScrollArea::horizontal()
            .id_salt("panel-tiles")
            .max_width(room)
            .auto_shrink([true, true])
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = vec2(4.0, 0.0);
                    for (n, id) in players.iter().enumerate() {
                        let name = t.tr_args("player-label", &[("n", (n + 1).into())]);
                        panel_tile(ui, scene, &context, Panel::Player(*id), &name);
                    }
                    let name = t.tr("layout-panel-carts");
                    panel_tile(ui, scene, &context, Panel::Cartwall, &name);
                });
            });
        ui.add_space(4.0);
        layout_menu(ui, scene);
    });
}

/// One panel tile: a click hides or shows the panel (RL7). Its label says
/// what a click does; a hidden panel on air shows a red dot. Disabled for
/// the last shown panel and while the layout is locked.
fn panel_tile(ui: &mut Ui, scene: &Scene<'_>, context: &TileContext, panel: Panel, name: &str) {
    let t = scene.i18n;
    let layout = &scene.state.layout;
    let shown = !layout.is_hidden(panel);
    let on_air = !shown && panel_layout::panel_on_air(scene.state, panel);
    let last = shown && context.shown == 1;
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

/// Brings the view state in line with the model before the frame's input
/// is handled: a maximised panel ends when the arrangement changed (RL16),
/// and a collapsed cartwall is not maximised (RL14);
/// a hidden player keeps no selection and is not the last player used, so
/// Delete cannot remove an entry the operator cannot see (RL17).
pub(crate) fn settle_view(view_state: &mut ViewState, state: &AppState) {
    // RL16: the arrangement changed; RL14: a collapsed cartwall is never
    // maximised, and expanding it does not maximise it again.
    let ends = view_state
        .maximised
        .as_ref()
        .is_some_and(|m| !m.holds(state) || (m.panel == Panel::Cartwall && !state.cartwall.open));
    if ends {
        view_state.maximised = None;
    }
    let hidden = &state.layout.hidden_players;
    view_state
        .selection
        .retain(|player, _| !hidden.contains(player));
    if view_state
        .active_player
        .is_some_and(|player| hidden.contains(&player))
    {
        view_state.active_player = None;
    }
}

/// RL16: the menu of a panel header's background (right click): Maximize,
/// or Restore on the maximised panel, and Hide. Both are disabled while the
/// layout is locked; Maximize also with one panel shown and on a collapsed
/// cartwall (RL14), Hide on the last shown panel.
pub(crate) fn header_menu(
    response: &egui::Response,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    panel: Panel,
) {
    response.context_menu(|ui| {
        let t = scene.i18n;
        let layout = &scene.state.layout;
        let players = scene.state.player_ids();
        ui.set_min_width(160.0);
        let maximised = view_state
            .maximised
            .as_ref()
            .is_some_and(|m| m.panel == panel);
        if maximised {
            if ui.button(t.tr("layout-restore")).clicked() {
                view_state.maximised = None;
                ui.close();
            }
        } else {
            // RL14: a collapsed cartwall has nothing to fill the room with.
            let collapsed = panel == Panel::Cartwall && !scene.state.cartwall.open;
            let can = !layout.locked && layout.shown_panels(&players) > 1 && !collapsed;
            let item = egui::Button::new(t.tr("layout-maximise"));
            if ui.add_enabled(can, item).clicked() {
                view_state.maximised = Some(Maximised::new(panel, scene.state));
                ui.close();
            }
        }
        let can_hide = !layout.locked && !layout.is_last_shown(&players, panel);
        if ui
            .add_enabled(can_hide, egui::Button::new(t.tr("layout-hide")))
            .clicked()
        {
            scene.ctl.send(Command::SetPanelHidden {
                panel,
                hidden: true,
            });
            ui.close();
        }
    });
}

/// RL12: a cartwall splitter drag, before the pass: it follows the pointer,
/// sends `SetCartwallSize` once on release, and ends when it no longer
/// applies (the layout was locked; the dock, the area or the cartwall's
/// range changed; or, once released, the model's echo arrived or the hold
/// ran out). While it lives: the layout the pass draws, with the drag's
/// size. `input` is the pass without a maximised panel, on the model's
/// layout.
pub(crate) fn cartwall_drag_layout(
    ui: &Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    input: &PassInput<'_>,
) -> Option<fp_model::layout::Layout> {
    let layout = &scene.state.layout;
    if layout.locked {
        view_state.cartwall_drag = None;
        return None;
    }
    let drag = view_state.cartwall_drag.as_mut()?;
    if drag.is_dragging() {
        let (down, pointer) = ui.input(|i| (i.pointer.primary_down(), i.pointer.latest_pos()));
        if down {
            if let Some(pos) = pointer {
                drag.follow(pos);
            }
        } else if let Some(command) =
            drag.release(layout.cartwall.size, scene.time + panel_layout::HOLD_SECS)
        {
            scene.ctl.send(command);
        }
    }
    let range = panel_layout::cartwall_range(input);
    if drag.stale(
        layout.cartwall.dock,
        input.area,
        range,
        layout.cartwall.size,
        scene.time,
    ) {
        view_state.cartwall_drag = None;
        return None;
    }
    let mut live = layout.clone();
    live.cartwall.size = Some(drag.share());
    Some(live)
}

/// RL12: the splitter between the players and the cartwall: horizontal at
/// the bottom, vertical on a side. A primary-button drag ends a maximised
/// panel and starts a `CartwallDrag` from the size without it (RL16); a
/// double-click ends a maximised panel too and goes back to fitting the
/// content at the bottom, or to the default share on a side. Inert while
/// the layout is locked or the cartwall has no size to choose (collapsed).
pub(crate) fn cartwall_splitter(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    input: &PassInput<'_>,
    arranged: &Arrangement,
    normal: &Arrangement,
) {
    let Some(gap) = arranged.splitter else {
        return;
    };
    let layout = &scene.state.layout;
    let dock = layout.cartwall.dock;
    let range = panel_layout::cartwall_range(input);
    let dragging = view_state
        .cartwall_drag
        .as_ref()
        .is_some_and(CartwallDrag::is_dragging);
    let cursor = if dock.is_side() {
        egui::CursorIcon::ResizeColumn
    } else {
        egui::CursorIcon::ResizeRow
    };
    let active = !layout.locked && range.is_some();
    let id = egui::Id::new("cartwall-splitter");
    let Some(response) = splitter(ui, gap, id, cursor, active, dragging) else {
        return;
    };
    if response.double_clicked() {
        // RL16: like a drag, it first ends a maximised panel.
        view_state.maximised = None;
        let size = dock.is_side().then_some(DEFAULT_SIDE_SHARE);
        scene.ctl.send(Command::SetCartwallSize(size));
    }
    if response.drag_started_by(egui::PointerButton::Primary) {
        // RL16: a drag first ends a maximised panel, then applies itself to
        // the size from before (`normal`).
        view_state.maximised = None;
        if let (Some(range), Some(cart)) = (range, normal.cartwall) {
            let start = if dock.is_side() {
                cart.width()
            } else {
                cart.height()
            };
            let origin = ui
                .input(|i| i.pointer.press_origin())
                .unwrap_or(gap.center());
            view_state.cartwall_drag =
                Some(CartwallDrag::new(dock, input.area, range, origin, start));
        }
    }
}
