//! The library's category tree (track library spec §7.3, LB9–LB11, LB27):
//! *All tracks*, the categories in tree order and *Uncategorised*, each
//! with its colour and its count (its own tracks and its subcategories').
//! A click selects, the arrow opens or closes a category (saved, LB27), the
//! ＋ and the right-click menu create, rename, recolour, change the kind
//! and delete, and a category is moved by dragging it.

use std::collections::HashMap;

use egui::{Align2, Rect, Sense, Stroke, StrokeKind, Ui, pos2, vec2};
use egui_phosphor::regular as icon;
use fp_model::library_search::{CategoryCounts, category_counts};
use fp_model::{
    AppState, CartColor, Categories, Category, CategoryId, Command, LibrarySelection, LibraryView,
    TrackKind,
};

use super::super::app::{Scene, ViewState};
use super::super::settings::carts::color_key;
use super::super::theme;
use super::super::widgets::{self, TileStyle, font, font_semibold};
use super::category_dialog::{self, KINDS, kind_key};
use super::confirm::{Removal, ask_removal};
use super::state::{CountsCache, LibraryUi};

/// Where the tree is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TreeMode {
    /// The library window: *All tracks* and *Uncategorised* too, and a
    /// click selects what the table shows.
    Window,
    /// Settings → Library: the categories only, nothing to select.
    Settings,
}

/// The height of the header with the ＋.
const HEADER_HEIGHT: f32 = 30.0;
/// The height of one item.
const ROW_HEIGHT: f32 = 24.0;
/// How far each level is indented.
const INDENT: f32 = 14.0;
/// The width of the disclosure arrow.
const ARROW: f32 = 16.0;
/// The side of an item's colour swatch.
const SWATCH: f32 = 10.0;
/// The share of an item's height, at its top and bottom, that drops a
/// dragged category into the gap above or below it rather than onto it.
const GAP_SHARE: f32 = 0.25;

/// One item of the tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Item {
    All,
    Category {
        id: CategoryId,
        depth: usize,
        has_children: bool,
        expanded: bool,
    },
    Uncategorised,
}

/// What an item asked for this frame, done once the items are drawn.
enum Action {
    Select(LibrarySelection),
    Toggle(CategoryId),
    NewCategory(Option<CategoryId>),
    Rename(CategoryId),
    Color(CategoryId, Option<CartColor>),
    Kind(CategoryId, TrackKind),
    Delete(CategoryId),
    Drag(CategoryId),
}

/// The items shown: the categories in tree order, the children of a closed
/// category left out.
fn items(categories: &Categories, expanded: &[CategoryId], mode: TreeMode) -> Vec<Item> {
    let mut children: HashMap<Option<CategoryId>, Vec<&Category>> = HashMap::new();
    for c in categories.iter() {
        children.entry(c.parent).or_default().push(c);
    }
    let mut out = Vec::with_capacity(categories.len() + 2);
    if mode == TreeMode::Window {
        out.push(Item::All);
    }
    let mut stack: Vec<(&Category, usize)> = children
        .get(&None)
        .map(|roots| roots.iter().rev().map(|c| (*c, 0)).collect())
        .unwrap_or_default();
    // Each category once, whatever the data (a cycle is repaired on load).
    let mut seen = std::collections::HashSet::new();
    while let Some((c, depth)) = stack.pop() {
        if !seen.insert(c.id) {
            continue;
        }
        let kids = children.get(&Some(c.id));
        let has_children = kids.is_some_and(|k| !k.is_empty());
        let open = expanded.contains(&c.id);
        out.push(Item::Category {
            id: c.id,
            depth,
            has_children,
            expanded: open,
        });
        if open && let Some(kids) = kids {
            stack.extend(kids.iter().rev().map(|k| (*k, depth + 1)));
        }
    }
    if mode == TreeMode::Window {
        out.push(Item::Uncategorised);
    }
    out
}

/// The counts, computed again only when the library or the categories
/// changed since the last time.
fn refresh_counts(library: &mut LibraryUi, state: &AppState) {
    let fresh = library.counts.as_ref().is_some_and(|c| {
        c.library.shares_storage(&state.library) && c.categories == state.categories
    });
    if !fresh {
        library.counts = Some(CountsCache {
            library: state.library.clone(),
            categories: state.categories.clone(),
            counts: category_counts(&state.library, &state.categories),
        });
    }
}

fn count_of(counts: Option<&CategoryCounts>, item: Item) -> usize {
    let Some(counts) = counts else { return 0 };
    match item {
        Item::All => counts.all,
        Item::Uncategorised => counts.uncategorised,
        Item::Category { id, .. } => counts.by_category.get(&id).copied().unwrap_or(0),
    }
}

/// The category tree, filling `ui`.
pub(crate) fn category_tree(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    mode: TreeMode,
) {
    let view = super::current_view(&scene.state.library_view, &view_state.library);
    refresh_counts(&mut view_state.library, scene.state);
    let mut actions = Vec::new();
    header(ui, scene, mode, &view, &mut actions);
    let shown = items(&scene.state.categories, &view.expanded, mode);
    let counts = view_state.library.counts.as_ref().map(|c| &c.counts);
    let dragging = view_state.library.tree_drag;
    let mut drawn: Vec<(Item, Rect)> = Vec::new();
    let body = ui.available_rect_before_wrap();
    egui::ScrollArea::vertical()
        .id_salt(("library-tree", mode == TreeMode::Window))
        .auto_shrink([false, false])
        .show_rows(ui, ROW_HEIGHT, shown.len(), |ui, range| {
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            for item in shown.get(range).unwrap_or_default() {
                let rect = row(
                    ui,
                    scene,
                    *item,
                    count_of(counts, *item),
                    mode,
                    &view,
                    dragging,
                    &mut actions,
                );
                drawn.push((*item, rect));
            }
        });
    drop_category(ui, scene, &mut view_state.library, &drawn, body);
    for action in actions {
        act(scene, view_state, action);
    }
}

/// The header: the tree's title and the ＋ that creates a category under
/// the selected one (at the top level when none is).
fn header(
    ui: &mut Ui,
    scene: &Scene<'_>,
    mode: TreeMode,
    view: &LibraryView,
    actions: &mut Vec<Action>,
) {
    let t = scene.i18n;
    let (rect, _) =
        ui.allocate_exact_size(vec2(ui.available_width(), HEADER_HEIGHT), Sense::hover());
    let painter = ui.painter();
    painter.hline(
        rect.x_range(),
        rect.bottom() - 0.5,
        Stroke::new(1.0, theme::DIVIDER),
    );
    painter.text(
        pos2(rect.left() + 10.0, rect.center().y),
        Align2::LEFT_CENTER,
        t.tr("library-tree-title"),
        font_semibold(11.0),
        theme::NEUTRAL_400,
    );
    let size = 22.0;
    let plus = Rect::from_center_size(
        pos2(rect.right() - 6.0 - size / 2.0, rect.center().y),
        vec2(size, size),
    );
    let label = t.tr("library-tree-new");
    let clicked = ui
        .scope_builder(egui::UiBuilder::new().max_rect(plus), |ui| {
            widgets::tile(
                ui,
                plus.size(),
                &label,
                true,
                TileStyle {
                    border_width: 0.0,
                    ..TileStyle::plain()
                },
                |p, r, c| {
                    p.text(r.center(), Align2::CENTER_CENTER, icon::PLUS, font(12.0), c);
                },
            )
        })
        .inner
        .clicked();
    if clicked {
        let parent = match (mode, view.selection) {
            (TreeMode::Window, LibrarySelection::Category(id)) => Some(id),
            _ => None,
        };
        actions.push(Action::NewCategory(parent));
    }
}

/// One item: its arrow, swatch, name and count, its menu and its drag.
/// Returns where it was drawn.
#[allow(clippy::too_many_arguments)]
fn row(
    ui: &mut Ui,
    scene: &Scene<'_>,
    item: Item,
    count: usize,
    mode: TreeMode,
    view: &LibraryView,
    dragging: Option<CategoryId>,
    actions: &mut Vec<Action>,
) -> Rect {
    let t = scene.i18n;
    let category = match item {
        Item::Category { id, .. } => scene.state.categories.get(id),
        _ => None,
    };
    let (name, color, depth) = match item {
        Item::All => (t.tr("library-tree-all"), None, 0),
        Item::Uncategorised => (t.tr("library-tree-uncategorised"), None, 0),
        Item::Category { depth, .. } => (
            category.map(|c| c.name.clone()).unwrap_or_default(),
            category.and_then(|c| c.color),
            depth,
        ),
    };
    let selection = match item {
        Item::All => LibrarySelection::All,
        Item::Uncategorised => LibrarySelection::Uncategorised,
        Item::Category { id, .. } => LibrarySelection::Category(id),
    };
    let selected = mode == TreeMode::Window && view.selection == selection;
    let sense = match item {
        Item::Category { .. } => Sense::click_and_drag(),
        _ => Sense::click(),
    };
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), ROW_HEIGHT), sense);
    let label = name.clone();
    response.widget_info(|| {
        let mut info = if mode == TreeMode::Window {
            egui::WidgetInfo::selected(egui::Role::TreeItem, true, selected, label.clone())
        } else {
            egui::WidgetInfo::labeled(egui::Role::TreeItem, true, label.clone())
        };
        info.current_text_value = Some(count.to_string());
        info
    });
    let being_dragged = matches!(item, Item::Category { id, .. } if dragging == Some(id));
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let fill = if selected {
            Some(theme::ACCENT_900)
        } else if response.hovered() && dragging.is_none() {
            Some(theme::NEUTRAL_900)
        } else {
            None
        };
        if let Some(fill) = fill {
            painter.rect_filled(rect, 0.0, fill);
        }
        let left = rect.left() + 6.0 + depth as f32 * INDENT;
        if let Item::Category {
            has_children: true,
            expanded,
            ..
        } = item
        {
            let glyph = if expanded {
                icon::CARET_DOWN
            } else {
                icon::CARET_RIGHT
            };
            painter.text(
                pos2(left + ARROW / 2.0, rect.center().y),
                Align2::CENTER_CENTER,
                glyph,
                font(11.0),
                theme::NEUTRAL_400,
            );
        }
        let swatch = Rect::from_center_size(
            pos2(left + ARROW + SWATCH / 2.0, rect.center().y),
            vec2(SWATCH, SWATCH),
        );
        match color {
            Some(c) => {
                painter.rect_filled(swatch, 2.0, theme::cart_color(c));
            }
            None => {
                painter.rect_stroke(
                    swatch,
                    2.0,
                    Stroke::new(1.0, theme::NEUTRAL_700),
                    StrokeKind::Inside,
                );
            }
        }
        let count_text = count.to_string();
        let count_galley = painter.layout_no_wrap(count_text, font(11.0), theme::NEUTRAL_500);
        let count_pos = pos2(
            rect.right() - 8.0 - count_galley.size().x,
            rect.center().y - count_galley.size().y / 2.0,
        );
        let name_rect = Rect::from_min_max(
            pos2(swatch.right() + 8.0, rect.top()),
            pos2(count_pos.x - 6.0, rect.bottom()),
        );
        let text = if being_dragged {
            theme::NEUTRAL_500
        } else if selected {
            theme::TEXT
        } else {
            theme::NEUTRAL_300
        };
        painter
            .with_clip_rect(name_rect.intersect(ui.clip_rect()))
            .text(
                pos2(name_rect.left(), rect.center().y),
                Align2::LEFT_CENTER,
                &name,
                font(12.0),
                text,
            );
        painter.galley(count_pos, count_galley, theme::NEUTRAL_500);
    }
    let mut arrow_clicked = false;
    if let Item::Category {
        id,
        depth,
        has_children: true,
        expanded,
    } = item
    {
        let left = rect.left() + 6.0 + depth as f32 * INDENT;
        let arrow = Rect::from_min_size(pos2(left, rect.top()), vec2(ARROW, ROW_HEIGHT));
        let key = if expanded {
            "library-tree-collapse"
        } else {
            "library-tree-expand"
        };
        let label = t.tr_args(key, &[("name", name.clone().into())]);
        let arrow_response = ui.interact(
            arrow,
            ui.scope_id().with(("library-tree-arrow", id)),
            Sense::click(),
        );
        arrow_response
            .widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, label.clone()));
        if arrow_response.clicked() {
            arrow_clicked = true;
            actions.push(Action::Toggle(id));
        }
    }
    if response.clicked() && !arrow_clicked && mode == TreeMode::Window && !selected {
        actions.push(Action::Select(selection));
    }
    if let Item::Category { id, .. } = item {
        if response.drag_started() {
            actions.push(Action::Drag(id));
        }
        response.context_menu(|ui| {
            if let Some(action) = menu(ui, scene, id, category) {
                actions.push(action);
                ui.close();
            }
        });
    }
    rect
}

/// The right-click menu of a category.
fn menu(
    ui: &mut Ui,
    scene: &Scene<'_>,
    id: CategoryId,
    category: Option<&Category>,
) -> Option<Action> {
    let t = scene.i18n;
    let mut action = None;
    ui.set_min_width(200.0);
    let item = |ui: &mut Ui, glyph: &str, key: &str| {
        let text = t.tr(key);
        let response = ui.button(format!("{glyph}  {text}"));
        response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, text.clone()));
        response.clicked()
    };
    if item(ui, icon::FOLDER_PLUS, "library-menu-new-subcategory") {
        action = Some(Action::NewCategory(Some(id)));
    }
    if item(ui, icon::PENCIL_SIMPLE, "library-menu-rename") {
        action = Some(Action::Rename(id));
    }
    let current_color = category.and_then(|c| c.color);
    let colour = t.tr("library-menu-colour");
    ui.menu_button(format!("{}  {colour}", icon::PALETTE), |ui| {
        let choices = std::iter::once(None).chain(CartColor::ALL.into_iter().map(Some));
        for choice in choices {
            let text = t.tr(color_key(choice));
            let response = ui.add(egui::Button::selectable(current_color == choice, &text));
            response.widget_info(|| {
                egui::WidgetInfo::selected(
                    egui::Role::Button,
                    true,
                    current_color == choice,
                    text.clone(),
                )
            });
            if response.clicked() {
                action = Some(Action::Color(id, choice));
            }
        }
    })
    .response
    .widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, colour.clone()));
    let current_kind = category.map(|c| c.kind);
    let kind_label = t.tr("library-menu-kind");
    ui.menu_button(format!("{}  {kind_label}", icon::TAG), |ui| {
        for kind in KINDS {
            let text = t.tr(kind_key(kind));
            let on = current_kind == Some(kind);
            let response = ui.add(egui::Button::selectable(on, &text));
            response.widget_info(|| {
                egui::WidgetInfo::selected(egui::Role::Button, true, on, text.clone())
            });
            if response.clicked() {
                action = Some(Action::Kind(id, kind));
            }
        }
    })
    .response
    .widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, kind_label.clone()));
    ui.separator();
    if item(ui, icon::TRASH, "library-menu-delete") {
        action = Some(Action::Delete(id));
    }
    action
}

/// How the tree shows where a dragged category would land.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mark {
    /// A line above the item.
    Above,
    /// A line below the item.
    Below,
    /// An outline around the item.
    Onto,
}

/// Where a dragged category lands when dropped at `y` on `target`'s item
/// of `rect`: in the gap above or below it (among its siblings; below an
/// open category with children, its first child), or onto it (its last
/// child). `None`: on itself.
fn drop_target(
    categories: &Categories,
    dragged: CategoryId,
    target: Item,
    rect: Rect,
    y: f32,
) -> Option<(Option<CategoryId>, usize, Mark)> {
    let Item::Category {
        id: target,
        has_children,
        expanded,
        ..
    } = target
    else {
        return None;
    };
    if target == dragged {
        return None;
    }
    let c = categories.get(target)?;
    let share = (y - rect.top()) / rect.height().max(1.0);
    // Indices count the siblings without the dragged category, as
    // `MoveCategory` takes them.
    let siblings = |parent: Option<CategoryId>| {
        categories
            .children(parent)
            .filter(|s| s.id != dragged)
            .map(|s| s.id)
            .collect::<Vec<_>>()
    };
    if share < GAP_SHARE {
        let at = siblings(c.parent).iter().position(|s| *s == target)?;
        Some((c.parent, at, Mark::Above))
    } else if share > 1.0 - GAP_SHARE {
        // Below an open category, the line sits above its first child.
        if has_children && expanded {
            return Some((Some(target), 0, Mark::Below));
        }
        let at = siblings(c.parent).iter().position(|s| *s == target)?;
        Some((c.parent, at + 1, Mark::Below))
    } else {
        Some((Some(target), siblings(Some(target)).len(), Mark::Onto))
    }
}

/// Follows a category being dragged: shows where it would land, and sends
/// the move when it is dropped on another category. A new parent that is
/// closed opens, so the moved category stays in sight.
fn drop_category(
    ui: &Ui,
    scene: &Scene<'_>,
    library: &mut LibraryUi,
    drawn: &[(Item, Rect)],
    body: Rect,
) {
    let Some(dragged) = library.tree_drag else {
        return;
    };
    let (pointer, released, down) = ui.input(|i| {
        (
            i.pointer.hover_pos().or_else(|| i.pointer.interact_pos()),
            i.pointer.primary_released(),
            i.pointer.primary_down(),
        )
    });
    let target = pointer.filter(|p| body.contains(*p)).and_then(|p| {
        drawn.iter().find_map(|(item, rect)| {
            if !rect.y_range().contains(p.y) {
                return None;
            }
            drop_target(&scene.state.categories, dragged, *item, *rect, p.y).map(|to| (to, *rect))
        })
    });
    if let Some(((_, _, mark), rect)) = target {
        let painter = ui.painter();
        let stroke = Stroke::new(2.0, theme::ACCENT);
        match mark {
            Mark::Onto => {
                painter.rect_stroke(rect.shrink(1.0), 2.0, stroke, StrokeKind::Inside);
            }
            Mark::Above => {
                painter.hline(rect.x_range(), rect.top(), stroke);
            }
            Mark::Below => {
                painter.hline(rect.x_range(), rect.bottom(), stroke);
            }
        }
    }
    if released {
        library.tree_drag = None;
        if let Some(((parent, index, _), _)) = target {
            scene.ctl.send(Command::MoveCategory {
                category: dragged,
                parent,
                index,
            });
            if let Some(parent) = parent {
                category_dialog::expand(scene, library, parent);
            }
        }
    } else if !down {
        library.tree_drag = None;
    }
}

/// Does what an item asked for.
fn act(scene: &Scene<'_>, view_state: &mut ViewState, action: Action) {
    let send = |view: LibraryView| scene.ctl.send(Command::SetLibraryView(Box::new(view)));
    match action {
        Action::Select(selection) => {
            let view = super::current_view(&scene.state.library_view, &view_state.library);
            send(LibraryView { selection, ..view });
        }
        Action::Toggle(id) => {
            let mut view = super::current_view(&scene.state.library_view, &view_state.library);
            if view.expanded.contains(&id) {
                view.expanded.retain(|e| *e != id);
            } else {
                view.expanded.push(id);
            }
            send(view);
        }
        Action::NewCategory(parent) => category_dialog::new_category(scene, view_state, parent),
        Action::Rename(id) => category_dialog::rename(scene, view_state, id),
        Action::Color(category, color) => {
            scene
                .ctl
                .send(Command::SetCategoryColor { category, color });
        }
        Action::Kind(category, kind) => {
            scene.ctl.send(Command::SetCategoryKind { category, kind });
        }
        Action::Delete(id) => ask_removal(scene, view_state, Removal::Category(id)),
        Action::Drag(id) => view_state.library.tree_drag = Some(id),
    }
}
