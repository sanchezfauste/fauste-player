//! The library window (track library spec §7.1, LB24–LB26): a floating,
//! non-modal window over the main area with its own title bar (minimise,
//! maximise or restore, close), moved by its title bar and resized from its
//! edges. Its place and state are `AppState::library_view`; a move, resize
//! or split drag is drawn here and sent as one `Command::SetLibraryView`
//! when it ends.

mod category_dialog;
pub(crate) mod confirm;
mod menu;
pub(crate) use menu::move_to_category;
pub(crate) mod state;
mod table;
pub(crate) mod tree;

use egui::{
    Align, Align2, CursorIcon, Frame, Id, Layout, Rect, Response, Sense, Stroke, Ui, UiBuilder,
    Vec2, pos2, vec2,
};
use egui_phosphor::regular as icon;
use fp_model::library_view::{LibrarySelection, LibraryView};
use fp_model::{CategoryId, Command, TrackId};

use super::app::{Scene, ViewState};
use super::panels;
use super::theme;
use super::widgets::{self, font, font_semibold};
use state::{Grip, LibraryUi, LiveRect, LiveTree, Pending, Reveal};

/// The height of the window's title bar.
pub(crate) const TITLE_BAR_HEIGHT: f32 = 28.0;
/// The least size the window is resized to.
pub(crate) const MIN_SIZE: Vec2 = vec2(480.0, 320.0);
/// The tree's width until the operator moves the splitter.
pub(crate) const DEFAULT_TREE_WIDTH: f32 = 220.0;
/// The share of the main area the window takes at first, across and down.
const DEFAULT_SHARE: f32 = 0.7;
/// The width of the band along each edge that resizes the window.
const GRIP: f32 = 4.0;
/// The width of the splitter between the tree and the table.
const SPLITTER: f32 = 6.0;
/// The least widths of the tree and of the table.
const MIN_TREE: f32 = 120.0;
const MIN_TABLE: f32 = 200.0;

/// Where the window opens before the operator places it: centred in
/// `area`, at 70 % of its width and height.
pub(crate) fn default_rect(area: Rect) -> Rect {
    Rect::from_center_size(area.center(), area.size() * DEFAULT_SHARE)
}

/// `rect` brought inside `area`: at least the least size (but no larger
/// than the area), then moved in until no part of it is outside.
pub(crate) fn clamp_rect(rect: Rect, area: Rect) -> Rect {
    let w = rect.width().max(MIN_SIZE.x).min(area.width().max(0.0));
    let h = rect.height().max(MIN_SIZE.y).min(area.height().max(0.0));
    let x = rect.left().min(area.right() - w).max(area.left());
    let y = rect.top().min(area.bottom() - h).max(area.top());
    Rect::from_min_size(pos2(x, y), vec2(w, h))
}

/// The rectangle a drag of `grip` by `delta` makes of `start`, inside
/// `area`: a move keeps the size, a resize keeps the opposite edges and
/// stops at the least size.
pub(crate) fn dragged_rect(start: Rect, grip: Grip, delta: Vec2, area: Rect) -> Rect {
    match grip {
        Grip::Move => clamp_rect(start.translate(delta), area),
        Grip::Resize {
            left,
            right,
            top,
            bottom,
        } => {
            let mut r = start;
            if left {
                r.min.x = (start.min.x + delta.x)
                    .min(start.max.x - MIN_SIZE.x)
                    .max(area.left());
            }
            if right {
                r.max.x = (start.max.x + delta.x)
                    .max(start.min.x + MIN_SIZE.x)
                    .min(area.right());
            }
            if top {
                r.min.y = (start.min.y + delta.y)
                    .min(start.max.y - MIN_SIZE.y)
                    .max(area.top());
            }
            if bottom {
                r.max.y = (start.max.y + delta.y)
                    .max(start.min.y + MIN_SIZE.y)
                    .min(area.bottom());
            }
            clamp_rect(r, area)
        }
    }
}

fn to_saved(rect: Rect) -> [f32; 4] {
    [rect.left(), rect.top(), rect.width(), rect.height()]
}

fn from_saved([x, y, w, h]: [f32; 4]) -> Rect {
    Rect::from_min_size(pos2(x, y), vec2(w, h))
}

/// The tree's width inside a body `total` points wide.
fn tree_width(width: f32, total: f32) -> f32 {
    width.min(total - SPLITTER - MIN_TABLE).max(MIN_TREE)
}

fn send(scene: &Scene<'_>, view: LibraryView) {
    scene.ctl.send(Command::SetLibraryView(Box::new(view)));
}

/// The view to build a send on: the snapshot's, with the rectangle and
/// tree width of a finished drag the snapshot does not hold yet, so that a
/// second view sent meanwhile keeps it.
pub(crate) fn current_view(saved: &LibraryView, library: &LibraryUi) -> LibraryView {
    let mut view = saved.clone();
    if let Some(pending) = &library.pending_rect {
        view.rect = Some(to_saved(pending.value));
    }
    if let Some(pending) = &library.pending_tree {
        view.tree_width = Some(pending.value);
    }
    view
}

/// *Show in library* (track library spec §9): opens the window on the
/// category of `track` (*Uncategorised* when it has none), clears the
/// search and, once the table lists it, selects its row and scrolls to it.
pub(crate) fn show_track(scene: &Scene<'_>, view_state: &mut ViewState, track: TrackId) {
    let state = scene.state;
    let Some(found) = state.library.get(track) else {
        return;
    };
    let library = &mut view_state.library;
    let mut view = current_view(&state.library_view, library);
    let selection = match found.category {
        Some(id) if state.categories.get(id).is_some() => LibrarySelection::Category(id),
        _ => LibrarySelection::Uncategorised,
    };
    // The category is shown in the tree: its ancestors open.
    if let LibrarySelection::Category(id) = selection {
        let mut up = state.categories.get(id).and_then(|c| c.parent);
        while let Some(parent) = up {
            if !view.expanded.contains(&parent) {
                view.expanded.push(parent);
            }
            up = state.categories.get(parent).and_then(|c| c.parent);
        }
    }
    library.search.clear();
    library.reveal = Some(Reveal::new(track, selection, &state.library_view));
    send(
        scene,
        LibraryView {
            open: true,
            minimised: false,
            selection,
            ..view
        },
    );
}

/// Opens the window, or closes it (the top bar's Library tile, LB24).
pub(crate) fn toggle(scene: &Scene<'_>, library: &LibraryUi) {
    let view = current_view(&scene.state.library_view, library);
    send(
        scene,
        LibraryView {
            open: !view.open,
            ..view
        },
    );
}

/// Forgets a finished drag's value once the snapshot has moved on from the view
/// it was sent against (it then holds the drag's view, or a later one).
fn settle(library: &mut LibraryUi, saved: &LibraryView) {
    if library
        .pending_rect
        .as_ref()
        .is_some_and(|p| *p.sent_against != *saved)
    {
        library.pending_rect = None;
    }
    if library
        .pending_tree
        .as_ref()
        .is_some_and(|p| *p.sent_against != *saved)
    {
        library.pending_tree = None;
    }
}

/// Draws the library window while it is open (never on a phone).
pub(crate) fn draw(ctx: &egui::Context, scene: &Scene<'_>, view_state: &mut ViewState) {
    view_state.library.drawn = None;
    view_state.library.drawn_layer = None;
    let saved = &scene.state.library_view;
    let area = match view_state.middle {
        Some(area) if saved.open && !view_state.phone_active => area,
        _ => {
            let library = &mut view_state.library;
            library.live = None;
            library.live_tree = None;
            library.pending_rect = None;
            library.pending_tree = None;
            return;
        }
    };
    settle(&mut view_state.library, saved);
    // LB26: the saved rectangle is drawn inside the area, whatever it says.
    let normal = clamp_rect(
        match (&view_state.library.live, &view_state.library.pending_rect) {
            (Some(live), _) => live.rect,
            (None, Some(pending)) => pending.value,
            (None, None) => saved.rect.map_or_else(|| default_rect(area), from_saved),
        },
        area,
    );
    let base = if saved.maximised { area } else { normal };
    let shown = if saved.minimised {
        Rect::from_min_size(base.min, vec2(base.width(), TITLE_BAR_HEIGHT))
    } else {
        base
    };
    let title = scene.i18n.tr("library-window-title");
    let response = egui::Window::new(title.clone())
        .id(Id::unique("library-window"))
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .constrain(false)
        .fixed_rect(shown)
        .frame(
            Frame::new()
                .fill(theme::SURFACE)
                .stroke(Stroke::new(1.0, theme::NEUTRAL_800))
                .inner_margin(0.0),
        )
        .show(ctx, |ui| {
            ui.set_min_size(ui.max_rect().size());
            let movable = !saved.maximised;
            title_bar(ui, scene, view_state, shown, normal, area, &title, movable);
            if !saved.minimised {
                let body = Rect::from_min_max(
                    pos2(shown.left(), shown.top() + TITLE_BAR_HEIGHT),
                    shown.max,
                )
                .shrink(1.0);
                split(ui, scene, view_state, body, tree, table);
            }
            if movable && !saved.minimised {
                edges(ui, scene, &mut view_state.library, shown, area);
            }
        });
    view_state.library.drawn = response.as_ref().map(|r| r.response.rect);
    view_state.library.drawn_layer = response.map(|r| r.response.layer_id);
}

/// The title bar: the title, then minimise, maximise or restore, and close
/// (the look of a panel header's buttons). Dragging it moves the window.
#[allow(clippy::too_many_arguments)]
fn title_bar(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    shown: Rect,
    normal: Rect,
    area: Rect,
    title: &str,
    movable: bool,
) {
    let saved = &current_view(&scene.state.library_view, &view_state.library);
    let bar = Rect::from_min_size(shown.min, vec2(shown.width(), TITLE_BAR_HEIGHT));
    let painter = ui.painter();
    painter.rect_filled(bar, 0.0, theme::NEUTRAL_900);
    if !saved.minimised {
        painter.hline(
            bar.x_range(),
            bar.bottom() - 0.5,
            Stroke::new(1.0, theme::DIVIDER),
        );
    }
    if movable {
        let response = ui.interact(bar, ui.scope_id().with("library-title-bar"), Sense::drag());
        grip_drag(
            scene,
            &mut view_state.library,
            &response,
            Grip::Move,
            normal,
            area,
        );
    }
    let mut row = ui.new_child(
        UiBuilder::new()
            .max_rect(bar.shrink2(vec2(10.0, 0.0)))
            .layout(Layout::left_to_right(Align::Center)),
    );
    // Painted, not a label: the window itself carries the title for
    // assistive technology.
    let buttons = 3.0 * panels::HEADER_BUTTON_SIZE + 2.0 * panels::HEADER_BUTTONS_SPACING;
    let room = Rect::from_min_max(
        pos2(bar.left() + 10.0, bar.top()),
        pos2(bar.right() - 10.0 - buttons - 8.0, bar.bottom()),
    );
    ui.painter()
        .with_clip_rect(room.intersect(ui.clip_rect()))
        .text(
            pos2(room.left(), room.center().y),
            Align2::LEFT_CENTER,
            title,
            font_semibold(12.0),
            theme::TEXT,
        );
    row.with_layout(Layout::right_to_left(Align::Center), |ui| {
        ui.spacing_mut().item_spacing = vec2(panels::HEADER_BUTTONS_SPACING, 0.0);
        let t = scene.i18n;
        let button = |ui: &mut Ui, key: &str, glyph: &str| {
            let label = t.tr(key);
            widgets::tile_without_tooltip(
                ui,
                vec2(panels::HEADER_BUTTON_SIZE, panels::HEADER_BUTTON_SIZE),
                &label,
                true,
                panels::header_button_style(),
                |p, r, c| {
                    p.text(r.center(), Align2::CENTER_CENTER, glyph, font(12.0), c);
                },
            )
            .on_hover_text(label)
            .clicked()
        };
        if button(ui, "library-window-close", icon::X) {
            send(
                scene,
                LibraryView {
                    open: false,
                    ..saved.clone()
                },
            );
        }
        let (key, glyph) = if saved.maximised {
            ("library-window-restore", icon::CORNERS_IN)
        } else {
            ("library-window-maximise", icon::CORNERS_OUT)
        };
        if button(ui, key, glyph) {
            send(
                scene,
                LibraryView {
                    maximised: !saved.maximised,
                    minimised: false,
                    ..saved.clone()
                },
            );
        }
        let key = if saved.minimised {
            "library-window-expand"
        } else {
            "library-window-minimise"
        };
        if button(ui, key, icon::MINUS) {
            send(
                scene,
                LibraryView {
                    minimised: !saved.minimised,
                    ..saved.clone()
                },
            );
        }
    });
}

/// The bands along the edges and the corners that resize the window.
fn edges(ui: &mut Ui, scene: &Scene<'_>, library: &mut LibraryUi, shown: Rect, area: Rect) {
    let r = shown;
    let corner = 2.0 * GRIP;
    let resize = |left, right, top, bottom| Grip::Resize {
        left,
        right,
        top,
        bottom,
    };
    // The corners last, so that they win over the edges they overlap.
    let grips = [
        (
            Rect::from_min_max(r.min, pos2(r.left() + GRIP, r.bottom())),
            resize(true, false, false, false),
            CursorIcon::ResizeHorizontal,
        ),
        (
            Rect::from_min_max(pos2(r.right() - GRIP, r.top()), r.max),
            resize(false, true, false, false),
            CursorIcon::ResizeHorizontal,
        ),
        (
            Rect::from_min_max(r.min, pos2(r.right(), r.top() + GRIP)),
            resize(false, false, true, false),
            CursorIcon::ResizeVertical,
        ),
        (
            Rect::from_min_max(pos2(r.left(), r.bottom() - GRIP), r.max),
            resize(false, false, false, true),
            CursorIcon::ResizeVertical,
        ),
        (
            Rect::from_min_size(r.min, vec2(corner, corner)),
            resize(true, false, true, false),
            CursorIcon::ResizeNwSe,
        ),
        (
            Rect::from_min_size(pos2(r.right() - corner, r.top()), vec2(corner, corner)),
            resize(false, true, true, false),
            CursorIcon::ResizeNeSw,
        ),
        (
            Rect::from_min_size(pos2(r.left(), r.bottom() - corner), vec2(corner, corner)),
            resize(true, false, false, true),
            CursorIcon::ResizeNeSw,
        ),
        (
            Rect::from_min_max(r.max - vec2(corner, corner), r.max),
            resize(false, true, false, true),
            CursorIcon::ResizeNwSe,
        ),
    ];
    for (n, (rect, grip, cursor)) in grips.into_iter().enumerate() {
        let response = ui
            .interact(rect, ui.scope_id().with(("library-grip", n)), Sense::drag())
            .on_hover_and_drag_cursor(cursor);
        grip_drag(scene, library, &response, grip, shown, area);
    }
}

/// Follows a drag of the title bar or of an edge: the window is drawn where
/// the drag puts it, and its view is sent once, when the drag ends.
fn grip_drag(
    scene: &Scene<'_>,
    library: &mut LibraryUi,
    response: &Response,
    grip: Grip,
    normal: Rect,
    area: Rect,
) {
    if response.drag_started() {
        let origin = response
            .ctx
            .input(|i| i.pointer.press_origin())
            .or(response.interact_pointer_pos());
        if let Some(origin) = origin {
            library.live = Some(LiveRect {
                grip,
                start: normal,
                origin,
                rect: normal,
            });
        }
    }
    let Some(live) = library.live.as_mut().filter(|l| l.grip == grip) else {
        return;
    };
    if !(response.dragged() || response.drag_stopped()) {
        return;
    }
    let pointer = response
        .interact_pointer_pos()
        .or_else(|| response.ctx.input(|i| i.pointer.latest_pos()));
    if let Some(pointer) = pointer {
        live.rect = dragged_rect(live.start, grip, pointer - live.origin, area);
    }
    if response.drag_stopped() {
        // A press without a move (a click) changes nothing.
        let (rect, start) = (live.rect, live.start);
        library.live = None;
        if rect == start {
            return;
        }
        let saved = &scene.state.library_view;
        library.pending_rect = Some(Pending {
            value: rect,
            sent_against: Box::new(saved.clone()),
        });
        send(scene, current_view(saved, library));
    }
}

/// The window's body: the tree on the left and the table on the right of a
/// splitter whose place is `LibraryView::tree_width`.
fn split(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    body: Rect,
    left: impl FnOnce(&mut Ui, &Scene<'_>, &mut ViewState),
    right: impl FnOnce(&mut Ui, &Scene<'_>, &mut ViewState),
) {
    let saved = &scene.state.library_view;
    let wanted = view_state
        .library
        .live_tree
        .map(|t| t.width)
        .or(view_state.library.pending_tree.as_ref().map(|p| p.value))
        .or(saved.tree_width)
        .unwrap_or(DEFAULT_TREE_WIDTH);
    let width = tree_width(wanted, body.width());
    let tree_rect = Rect::from_min_size(body.min, vec2(width, body.height()));
    let table_rect = Rect::from_min_max(pos2(body.left() + width + SPLITTER, body.top()), body.max);
    let clip = ui.clip_rect();
    let mut tree_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(tree_rect)
            .id_salt("library-tree")
            .layout(Layout::top_down(Align::Min)),
    );
    tree_ui.set_clip_rect(tree_rect.intersect(clip));
    left(&mut tree_ui, scene, view_state);
    let mut table_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(table_rect)
            .id_salt("library-table")
            .layout(Layout::top_down(Align::Min)),
    );
    table_ui.set_clip_rect(table_rect.intersect(clip));
    right(&mut table_ui, scene, view_state);
    splitter(ui, scene, &mut view_state.library, body, width);
}

/// The splitter between the tree and the table: a drag moves it, and the
/// new width is sent when the drag ends.
fn splitter(ui: &mut Ui, scene: &Scene<'_>, library: &mut LibraryUi, body: Rect, width: f32) {
    let rect = Rect::from_min_size(
        pos2(body.left() + width, body.top()),
        vec2(SPLITTER, body.height()),
    );
    let response = ui
        .interact(rect, ui.scope_id().with("library-splitter"), Sense::drag())
        .on_hover_and_drag_cursor(CursorIcon::ResizeHorizontal);
    let colour = if response.hovered() || response.dragged() {
        theme::NEUTRAL_700
    } else {
        theme::DIVIDER
    };
    ui.painter()
        .vline(rect.center().x, rect.y_range(), Stroke::new(1.0, colour));
    if response.drag_started()
        && let Some(origin) = response
            .ctx
            .input(|i| i.pointer.press_origin())
            .or(response.interact_pointer_pos())
    {
        library.live_tree = Some(LiveTree {
            start: width,
            origin: origin.x,
            width,
        });
    }
    let Some(live) = library.live_tree.as_mut() else {
        return;
    };
    if !(response.dragged() || response.drag_stopped()) {
        return;
    }
    if let Some(pointer) = response
        .interact_pointer_pos()
        .or_else(|| response.ctx.input(|i| i.pointer.latest_pos()))
    {
        live.width = tree_width(live.start + pointer.x - live.origin, body.width());
    }
    if response.drag_stopped() {
        let (width, start) = (live.width, live.start);
        library.live_tree = None;
        if width == start {
            return;
        }
        let saved = &scene.state.library_view;
        library.pending_tree = Some(Pending {
            value: width,
            sent_against: Box::new(saved.clone()),
        });
        send(scene, current_view(saved, library));
    }
}

/// The category tree (track library spec §7.3).
fn tree(ui: &mut Ui, scene: &Scene<'_>, view_state: &mut ViewState) {
    tree::category_tree(ui, scene, view_state, tree::TreeMode::Window);
}

/// The library's dialogs (a category's name, colour and kind) and its
/// removal questions, over everything else.
pub(crate) fn dialogs(ctx: &egui::Context, scene: &Scene<'_>, view_state: &mut ViewState) {
    category_dialog::draw(ctx, scene, view_state);
    confirm::draw_confirmations(ctx, scene, view_state);
}

/// The search field, the Whole library toggle and the table (track library
/// spec §7.4, §7.5).
fn table(ui: &mut Ui, scene: &Scene<'_>, view_state: &mut ViewState) {
    table::table(ui, scene, view_state);
}

/// The category that added files join (track library spec §7.4): the
/// selected one; none (uncategorised) with *All tracks* or
/// *Uncategorised* selected.
pub(crate) fn added_category(view: &LibraryView) -> Option<CategoryId> {
    match view.selection {
        LibrarySelection::Category(id) => Some(id),
        LibrarySelection::All | LibrarySelection::Uncategorised => None,
    }
}
