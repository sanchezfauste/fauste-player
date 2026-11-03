//! The library window's table (track library spec §7.4, §7.5): the search
//! field, the Whole library toggle and Add files above the track rows. The
//! columns are the playlists' choice (`ui.table_columns`, without `#`)
//! plus Category and Kind, after a narrow status cell; their widths and
//! order are `LibraryView::columns` and `LibraryView::sort`. Only the rows
//! in sight are laid out (LB28).

use egui::{
    Align, Align2, CursorIcon, Layout, Rect, ScrollArea, Sense, Ui, UiBuilder, Vec2, pos2, vec2,
};
use egui_phosphor::regular as icon;
use fp_model::library_view::{LibraryColumn, LibraryColumnWidth, LibrarySort, LibraryView};
use fp_model::{TableColumn, Track, TrackId};

use super::super::app::{DragTracks, PickTarget, Scene, ViewState};
use super::super::table::RowMenuOutcome;
use super::super::table_layout;
use super::super::theme;
use super::super::track_cells::{self, RowColours};
use super::super::widgets::{self, TileStyle, font};
use super::confirm::{Removal, ask_removal};
use super::menu::{self, MenuOutcome};
use super::state::{LibraryUi, LiveColumns, selected_among};
use super::{added_category, current_view, send};

const TOOLBAR_HEIGHT: f32 = 36.0;
const HEADER_HEIGHT: f32 = 24.0;
const ROW_HEIGHT: f32 = 28.0;
/// The leading cell with a track's status icon (missing, unreadable).
const STATUS_WIDTH: f32 = 26.0;
/// The side of a category's colour swatch in its cell.
const SWATCH: f32 = 8.0;

/// The table's columns: the playlists' choice without `#` (library rows
/// have no position), then Category and Kind.
pub(crate) fn columns(table_columns: &[TableColumn]) -> Vec<LibraryColumn> {
    fp_model::normalize_columns(table_columns)
        .into_iter()
        .filter(|c| *c != TableColumn::Number)
        .map(LibraryColumn::Tag)
        .chain([LibraryColumn::Category, LibraryColumn::Kind])
        .collect()
}

/// The Fluent key of a column's name.
fn column_key(column: LibraryColumn) -> String {
    match column {
        LibraryColumn::Tag(c) => format!("col-{}", c.name()),
        LibraryColumn::Category => "library-col-category".to_owned(),
        LibraryColumn::Kind => "library-col-kind".to_owned(),
    }
}

fn column_min(column: LibraryColumn) -> f32 {
    match column {
        LibraryColumn::Tag(c) => table_layout::column_min(c, 0),
        LibraryColumn::Category => 60.0,
        LibraryColumn::Kind => 50.0,
    }
}

/// A column's width before the operator sets one, in points.
fn default_width(column: LibraryColumn) -> f32 {
    match column {
        LibraryColumn::Tag(c) => match c {
            TableColumn::Title => 240.0,
            TableColumn::Artist | TableColumn::Album => 170.0,
            TableColumn::FileName | TableColumn::Comment => 200.0,
            TableColumn::Genre => 110.0,
            TableColumn::Number
            | TableColumn::Date
            | TableColumn::Duration
            | TableColumn::Intro => table_layout::column_min(c, 0),
        },
        LibraryColumn::Category => 130.0,
        LibraryColumn::Kind => 80.0,
    }
}

/// The pixel widths of `columns` in `room` points: the saved widths (or the
/// defaults) scaled to fill the room, none under its minimum.
fn column_px(view: &LibraryView, columns: &[LibraryColumn], room: f32) -> Vec<f32> {
    let wanted: Vec<f32> = columns
        .iter()
        .map(|c| {
            view.columns
                .iter()
                .find(|w| w.column == *c)
                .map_or_else(|| default_width(*c), |w| w.width)
        })
        .collect();
    let mins: Vec<f32> = columns.iter().map(|c| column_min(*c)).collect();
    table_layout::fit(&wanted, &mins, room)
}

/// The order a click on `column`'s header asks for: ascending by it, or the
/// other way when the table is already ordered by it.
fn next_sort(current: Option<LibrarySort>, column: LibraryColumn) -> LibrarySort {
    match current {
        Some(sort) if sort.column == column => LibrarySort {
            column,
            ascending: !sort.ascending,
        },
        _ => LibrarySort {
            column,
            ascending: true,
        },
    }
}

/// The search field, the Whole library toggle, Add files and the table.
pub(crate) fn table(ui: &mut Ui, scene: &Scene<'_>, view_state: &mut ViewState) {
    toolbar(ui, scene, &mut view_state.library);
    let area = ui.available_rect_before_wrap();
    let columns = columns(&scene.state.config.ui.table_columns);
    let room = (area.width() - STATUS_WIDTH - ui.spacing().scroll.allocated_width()).max(0.0);
    let from_view = column_px(&scene.state.library_view, &columns, room);
    let px = live_px(scene, &mut view_state.library, &columns, from_view);
    let header = Rect::from_min_size(area.min, vec2(area.width(), HEADER_HEIGHT));
    header_row(ui, scene, &mut view_state.library, header, &columns, &px);
    let body = Rect::from_min_max(pos2(area.left(), header.bottom()), area.max);
    let mut body_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(body)
            .id_salt("library-rows")
            .layout(Layout::top_down(Align::Min)),
    );
    body_ui.set_clip_rect(body.intersect(ui.clip_rect()));
    rows(&mut body_ui, scene, view_state, &columns, &px);
    ui.advance_cursor_after_rect(area);
}

/// The row above the table: the search field, then the Whole library
/// toggle and Add files at the right.
fn toolbar(ui: &mut Ui, scene: &Scene<'_>, library: &mut LibraryUi) {
    let t = scene.i18n;
    let saved = &scene.state.library_view;
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), TOOLBAR_HEIGHT),
        Layout::right_to_left(Align::Center),
        |ui| {
            ui.set_min_height(TOOLBAR_HEIGHT);
            ui.spacing_mut().item_spacing.x = 8.0;
            ui.add_space(8.0);
            let add = t.tr("library-add-files");
            if widgets::text_button(ui, Some(icon::PLUS), &add, true, TileStyle::plain()).clicked()
            {
                scene.pick_files(PickTarget::Library(added_category(saved)));
            }
            ui.add_space(4.0);
            // The toggle's name is painted beside it; a click on either
            // switches the scope (LB29).
            let whole = t.tr("library-whole-library");
            let galley = ui
                .painter()
                .layout_no_wrap(whole.clone(), font(12.0), theme::NEUTRAL_300);
            let (text_rect, text) = ui.allocate_exact_size(galley.size(), Sense::click());
            ui.painter()
                .galley(text_rect.min, galley, theme::NEUTRAL_300);
            let mut on = saved.whole_library;
            let toggled = widgets::toggle(ui, &mut on, &whole);
            if toggled || text.clicked() {
                let view = current_view(saved, library);
                send(
                    scene,
                    LibraryView {
                        whole_library: !saved.whole_library,
                        ..view
                    },
                );
            }
            let hint = t.tr("library-search-hint");
            let label = t.tr("library-search");
            let search = ui.add(
                egui::TextEdit::singleline(&mut library.search)
                    .hint_text(&hint)
                    .background_color(theme::NEUTRAL_800)
                    .font(font(12.0))
                    .desired_width((ui.available_width() - 8.0).max(40.0)),
            );
            search.widget_info(|| {
                egui::WidgetInfo::labeled(egui::Role::TextInput, true, label.clone())
            });
        },
    );
}

/// The widths to draw: the view's, or those of a column edge being dragged,
/// until the snapshot holds the widths it sent.
fn live_px(
    scene: &Scene<'_>,
    library: &mut LibraryUi,
    columns: &[LibraryColumn],
    from_view: Vec<f32>,
) -> Vec<f32> {
    let saved = &scene.state.library_view.columns;
    match &library.live_columns {
        Some(live)
            if live.columns == columns
                && live.released.as_ref().is_none_or(|before| before == saved) =>
        {
            live.px.clone()
        }
        Some(_) => {
            library.live_columns = None;
            from_view
        }
        None => from_view,
    }
}

/// The column headers: a click orders the table by that column, again the
/// other way; a drag of an edge between two columns resizes them.
fn header_row(
    ui: &mut Ui,
    scene: &Scene<'_>,
    library: &mut LibraryUi,
    header: Rect,
    columns: &[LibraryColumn],
    px: &[f32],
) {
    let t = scene.i18n;
    let saved = &scene.state.library_view;
    ui.painter().hline(
        header.x_range(),
        header.bottom() - 0.5,
        egui::Stroke::new(1.0, theme::DIVIDER),
    );
    let mut x = header.left() + STATUS_WIDTH;
    let mut clicked = None;
    for (column, w) in columns.iter().zip(px) {
        let cell = Rect::from_min_size(pos2(x, header.top()), vec2(*w, HEADER_HEIGHT));
        x += w;
        let name = t.tr(&column_key(*column));
        let shown = match saved.sort {
            Some(sort) if sort.column == *column => {
                let caret = if sort.ascending {
                    icon::CARET_UP
                } else {
                    icon::CARET_DOWN
                };
                format!("{name} {caret}")
            }
            _ => name.clone(),
        };
        let right = matches!(column, LibraryColumn::Tag(c) if track_cells::right_aligned(*c));
        let mut cell_ui = ui.new_child(
            UiBuilder::new()
                .max_rect(cell)
                .layout(Layout::left_to_right(Align::Center)),
        );
        cell_ui.set_clip_rect(cell.intersect(ui.clip_rect()));
        track_cells::header_label(&mut cell_ui, &shown, right);
        let response = ui.interact(
            cell,
            ui.scope_id().with(("library-header", *column)),
            Sense::click(),
        );
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::Role::ColumnHeader, true, name.clone())
        });
        if response.clicked() {
            clicked = Some(*column);
        }
    }
    if let Some(column) = clicked {
        let view = current_view(saved, library);
        send(
            scene,
            LibraryView {
                sort: Some(next_sort(saved.sort, column)),
                ..view
            },
        );
    }
    resize_handles(ui, scene, library, header, columns, px);
}

/// The grab zones on the edges between the columns' headers: a drag
/// resizes the column on the left and shares the change among those on
/// its right (as the playlists' tables do); the widths are sent when the
/// drag ends.
fn resize_handles(
    ui: &mut Ui,
    scene: &Scene<'_>,
    library: &mut LibraryUi,
    header: Rect,
    columns: &[LibraryColumn],
    px: &[f32],
) {
    let grab = ui.style().interaction.resize_grab_radius_side;
    let mins: Vec<f32> = columns.iter().map(|c| column_min(*c)).collect();
    let mut edge_x = header.left() + STATUS_WIDTH;
    for (edge, w) in px.iter().enumerate().take(px.len().saturating_sub(1)) {
        edge_x += w;
        let rect = Rect::from_min_max(
            pos2(edge_x - grab, header.top()),
            pos2(edge_x + grab, header.bottom()),
        );
        let response = ui
            .interact(
                rect,
                ui.scope_id().with(("library-edge", edge)),
                Sense::drag(),
            )
            .on_hover_and_drag_cursor(CursorIcon::ResizeColumn);
        let pointer = response
            .interact_pointer_pos()
            .or_else(|| ui.input(|i| i.pointer.latest_pos()));
        if response.drag_started()
            && let Some(p) = pointer
        {
            library.live_columns = Some(LiveColumns {
                edge,
                columns: columns.to_vec(),
                start_px: px.to_vec(),
                start_x: p.x,
                px: px.to_vec(),
                released: None,
            });
        }
        let Some(live) = library.live_columns.as_mut().filter(|l| l.edge == edge) else {
            continue;
        };
        if response.dragged()
            && let Some(p) = pointer
            && let Some(start) = live.start_px.get(edge)
        {
            live.px =
                table_layout::resize_px(&live.start_px, &mins, edge, start + p.x - live.start_x);
        }
        if response.drag_stopped() {
            let saved = &scene.state.library_view;
            let moved = live
                .px
                .iter()
                .zip(&live.start_px)
                .any(|(a, b)| (a - b).abs() > 0.5);
            let widths: Vec<LibraryColumnWidth> = live
                .columns
                .iter()
                .zip(&live.px)
                .map(|(column, width)| LibraryColumnWidth {
                    column: *column,
                    width: *width,
                })
                .collect();
            if !moved || widths == saved.columns {
                library.live_columns = None;
                continue;
            }
            live.released = Some(saved.columns.clone());
            let view = current_view(saved, library);
            send(
                scene,
                LibraryView {
                    columns: widths,
                    ..view
                },
            );
        }
    }
}

/// The rows in sight (LB28): click selects, Ctrl (Cmd) click adds and
/// Shift click extends from the anchor. A right-click opens the row menu
/// and a drag carries the selection to a playlist or a cart (track library
/// spec §7.4); on a row outside the selection both act on that row alone.
fn rows(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    columns: &[LibraryColumn],
    px: &[f32],
) {
    let library = &mut view_state.library;
    let count = library.rows(scene.state).len();
    let mut clicked: Option<(TrackId, egui::Modifiers)> = None;
    // A right-click or a drag on a row outside the selection selects it.
    let mut taken: Option<TrackId> = None;
    let mut outcome: Option<MenuOutcome> = None;
    // On touch a finger drag scrolls the table instead.
    let sense = if view_state.touch {
        Sense::click()
    } else {
        Sense::click_and_drag()
    };
    {
        let rows = library.rows.as_ref().map_or(&[][..], |c| c.rows.as_slice());
        let selected = &library.selected;
        // What a menu or a drag on the row of `id` acts on.
        let targets = |id: TrackId| {
            if selected.contains(&id) {
                selected_among(rows, selected)
            } else {
                vec![id]
            }
        };
        ui.spacing_mut().item_spacing = Vec2::ZERO;
        ScrollArea::vertical()
            .id_salt("library-table-scroll")
            .auto_shrink([false, false])
            .show_rows(ui, ROW_HEIGHT, count, |ui, range| {
                for i in range {
                    let (rect, response) =
                        ui.allocate_exact_size(vec2(ui.available_width(), ROW_HEIGHT), sense);
                    let Some(&id) = rows.get(i) else { continue };
                    let Some(track) = scene.state.library.get(id) else {
                        continue;
                    };
                    let is_selected = selected.contains(&id);
                    response.widget_info(|| {
                        egui::WidgetInfo::selected(egui::Role::Row, true, is_selected, "")
                    });
                    if response.clicked() {
                        clicked = Some((id, click_modifiers(ui)));
                    }
                    if response.drag_started() {
                        egui::DragAndDrop::set_payload(
                            ui.ctx(),
                            DragTracks {
                                tracks: targets(id),
                            },
                        );
                        taken = Some(id);
                    }
                    // A right-click, or a long press on a touch screen
                    // (egui counts it as a secondary click).
                    if response.secondary_clicked() {
                        taken = Some(id);
                    }
                    response.context_menu(|ui| {
                        if let Some(o) = menu::row_menu(ui, scene, &targets(id)) {
                            outcome = Some(o);
                        }
                    });
                    if ui.is_rect_visible(rect) {
                        row(ui, scene, track, rect, columns, px, is_selected);
                    }
                }
            });
    }
    if let Some((id, modifiers)) = clicked {
        let cache = library.rows.take();
        let rows = cache.as_ref().map_or(&[][..], |c| c.rows.as_slice());
        library.click(rows, id, modifiers);
        library.rows = cache;
    }
    if let Some(id) = taken {
        library.take_row(id);
    }
    match outcome {
        Some(MenuOutcome::Row(RowMenuOutcome::EditTags(id))) => view_state.edit_tags = Some(id),
        Some(MenuOutcome::Row(RowMenuOutcome::Loudness(id))) => {
            view_state.loudness_popup = Some((id, None));
        }
        Some(MenuOutcome::Remove(tracks)) => {
            ask_removal(scene, view_state, Removal::Tracks(tracks));
        }
        Some(MenuOutcome::Notice(text)) => view_state.set_notice(text, scene.time),
        None => {}
    }
}

/// The modifiers held when the primary button was released: those of the
/// release itself, which a fast click may not leave in `modifiers` for the
/// frame.
fn click_modifiers(ui: &Ui) -> egui::Modifiers {
    ui.input(|i| {
        i.events
            .iter()
            .rev()
            .find_map(|e| match e {
                egui::Event::PointerButton {
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers,
                    ..
                } => Some(*modifiers),
                _ => None,
            })
            .unwrap_or(i.modifiers)
    })
}

/// One row: its background, the status cell and the columns' cells.
fn row(
    ui: &mut Ui,
    scene: &Scene<'_>,
    track: &Track,
    rect: Rect,
    columns: &[LibraryColumn],
    px: &[f32],
    selected: bool,
) {
    let fill = if selected {
        theme::ACCENT_900
    } else {
        egui::Color32::TRANSPARENT
    };
    track_cells::row_background(ui, rect, fill);
    if selected {
        track_cells::selection_outline(ui, rect, true, true);
    }
    let colours = RowColours {
        text: theme::TEXT,
        secondary: theme::NEUTRAL_400,
    };
    let clip = ui.clip_rect();
    let cell_ui = |ui: &mut Ui, cell: Rect| {
        let mut child = ui.new_child(
            UiBuilder::new()
                .max_rect(cell)
                .layout(Layout::left_to_right(Align::Center)),
        );
        child.set_clip_rect(cell.intersect(clip));
        child
    };
    let status = Rect::from_min_size(rect.min, vec2(STATUS_WIDTH, ROW_HEIGHT));
    let mut status_ui = cell_ui(ui, status);
    status_ui.add_space(6.0);
    track_cells::file_flag(&mut status_ui, scene, track);
    let mut x = status.right();
    for (column, w) in columns.iter().zip(px) {
        let cell = Rect::from_min_size(pos2(x, rect.top()), vec2(*w, ROW_HEIGHT));
        x += w;
        match column {
            LibraryColumn::Tag(c) => {
                let mut cell = cell_ui(ui, cell);
                track_cells::tag_cell(&mut cell, scene, track, *c, colours);
            }
            LibraryColumn::Category => {
                if let Some(category) = track.category.and_then(|c| scene.state.categories.get(c)) {
                    let painter = ui.painter().with_clip_rect(cell.intersect(clip));
                    let mut left = cell.left() + 8.0;
                    if let Some(c) = category.color {
                        let swatch = Rect::from_center_size(
                            pos2(left + SWATCH / 2.0, cell.center().y),
                            vec2(SWATCH, SWATCH),
                        );
                        painter.rect_filled(swatch, 2.0, theme::cart_color(c));
                        left += SWATCH + 6.0;
                    }
                    painter.text(
                        pos2(left, cell.center().y),
                        Align2::LEFT_CENTER,
                        &category.name,
                        font(12.0),
                        colours.secondary,
                    );
                }
            }
            LibraryColumn::Kind => {
                let kind = scene.i18n.tr(super::category_dialog::kind_key(track.kind));
                ui.painter().with_clip_rect(cell.intersect(clip)).text(
                    pos2(cell.left() + 8.0, cell.center().y),
                    Align2::LEFT_CENTER,
                    kind,
                    font(12.0),
                    colours.secondary,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fp_model::{AppState, Command};
    use std::path::PathBuf;

    fn listed(library: &mut LibraryUi, state: &AppState, track: TrackId) -> bool {
        library.rows(state).contains(&track)
    }

    fn library_state(names: &[&str]) -> AppState {
        let mut s = AppState::new(fp_model::Config::default(), "Main");
        fp_model::apply(
            &mut s,
            Command::AddToLibrary {
                paths: names
                    .iter()
                    .map(|n| PathBuf::from(format!("/lib/{n}.mp3")))
                    .collect(),
                category: None,
            },
        )
        .unwrap();
        s
    }

    fn id(s: &AppState, name: &str) -> TrackId {
        s.library
            .track_at(&PathBuf::from(format!("/lib/{name}.mp3")))
            .unwrap()
    }

    #[test]
    fn the_columns_drop_the_number_and_add_category_and_kind() {
        let cols = columns(&[
            TableColumn::Number,
            TableColumn::Title,
            TableColumn::Duration,
        ]);
        assert_eq!(
            cols,
            vec![
                LibraryColumn::Tag(TableColumn::Title),
                LibraryColumn::Tag(TableColumn::Duration),
                LibraryColumn::Category,
                LibraryColumn::Kind,
            ]
        );
    }

    #[test]
    fn a_header_click_sorts_up_then_down_and_another_column_starts_up() {
        let title = LibraryColumn::Tag(TableColumn::Title);
        let up = next_sort(None, title);
        assert!(up.ascending);
        let down = next_sort(Some(up), title);
        assert!(!down.ascending);
        let kind = next_sort(Some(down), LibraryColumn::Kind);
        assert_eq!(kind.column, LibraryColumn::Kind);
        assert!(kind.ascending);
    }

    #[test]
    fn the_rows_follow_an_edit_of_the_library_and_the_search() {
        let mut s = library_state(&["Alpha", "Beta"]);
        let mut library = LibraryUi::default();
        let alpha = id(&s, "Alpha");
        assert!(listed(&mut library, &s, alpha));
        library.search = "beta".into();
        assert!(!listed(&mut library, &s, alpha), "the search filters");
        s.library.get_mut(alpha).unwrap().title = "Beta too".into();
        assert!(
            listed(&mut library, &s, alpha),
            "the index follows the edit"
        );
    }

    #[test]
    fn selected_rows_whose_track_is_gone_are_dropped() {
        let mut s = library_state(&["Alpha", "Beta"]);
        let mut library = LibraryUi::default();
        let (alpha, beta) = (id(&s, "Alpha"), id(&s, "Beta"));
        let rows = library.rows(&s).to_vec();
        library.click(&rows, alpha, egui::Modifiers::NONE);
        library.click(&rows, beta, egui::Modifiers::COMMAND);
        assert_eq!(library.selected, vec![alpha, beta]);
        s.library.remove(beta);
        library.rows(&s);
        assert_eq!(library.selected, vec![alpha]);
        assert_eq!(library.anchor, None, "the anchor was the removed row");
    }

    #[test]
    fn ctrl_click_removes_a_selected_row_and_shift_click_spans_from_the_anchor() {
        let s = library_state(&["A", "B", "C", "D"]);
        let mut library = LibraryUi::default();
        let rows = library.rows(&s).to_vec();
        let [a, b, c, d] = [rows[0], rows[1], rows[2], rows[3]];
        library.click(&rows, d, egui::Modifiers::NONE);
        library.click(&rows, b, egui::Modifiers::SHIFT);
        assert_eq!(library.selected, vec![b, c, d], "upwards too");
        library.click(&rows, c, egui::Modifiers::COMMAND);
        assert_eq!(library.selected, vec![b, d]);
        library.click(&rows, a, egui::Modifiers::COMMAND | egui::Modifiers::SHIFT);
        assert_eq!(library.selected, vec![b, d, a, c], "added to the selection");
    }
}
