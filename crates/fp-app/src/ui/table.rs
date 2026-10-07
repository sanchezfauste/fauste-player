//! The track table of a player column (spec §8.3): virtualised rows,
//! resizable columns, row colours, context menu and drag and drop.

use egui::{Align, Color32, Layout, Rect, RichText, Sense, Ui, pos2, vec2};
use egui_extras::{Column, TableBuilder};
use egui_phosphor::regular as icon;
use fp_model::{ColumnWidths, Command, EntryId, PlayerId, PlaylistId, TableColumn, Transport};

use super::app::{DragColumn, DragEntry, DropTarget, FollowScroll, Scene, ViewState};
use super::format;
use super::glyphs::{self, TransportAction};
use super::table_layout;
use super::theme;
use super::view::{self, RowStatus};
use super::widgets::{self, font, font_medium};

const HEADER_HEIGHT: f32 = 24.0;
const ROW_HEIGHT: f32 = 28.0;
/// Width of the track info popup: fixed, so egui's sizing pass gives the
/// final size and the popup is never shown elsewhere first (Q9.3).
const TIP_WIDTH: f32 = 380.0;
/// How long a released column edge keeps its widths while the model catches
/// up with the command that stores them.
const HOLD_SECS: f64 = 0.5;

/// A column edge being dragged (feedback 2 spec O16): the widths are
/// recomputed from the pointer on every frame and sent once, on release.
pub(crate) struct LiveResize {
    player: PlayerId,
    /// The column whose right edge is dragged.
    edge: usize,
    columns: Vec<TableColumn>,
    /// The table width the drag began with.
    width: f32,
    start_px: Vec<f32>,
    start_x: f32,
    /// The widths now.
    px: Vec<f32>,
    /// Set on release: the stored widths at that moment, and until when the
    /// released widths are still drawn.
    released: Option<(ColumnWidths, f64)>,
}

fn header_label(ui: &mut Ui, text: &str, right: bool) {
    let label = egui::Label::new(
        RichText::new(text.to_uppercase())
            .font(font(10.0))
            .color(theme::NEUTRAL_500),
    )
    .selectable(false);
    if right {
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add_space(8.0);
            ui.add(label);
        });
    } else {
        ui.add_space(8.0);
        ui.add(label.truncate());
    }
}

/// The widths to draw: the model's, or the ones of a column edge being
/// dragged (or just released). Called before anything is drawn, so a drag
/// shows in the same frame as the pointer move.
#[allow(clippy::too_many_arguments)]
fn live_widths(
    ui: &Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    player: PlayerId,
    columns: &[TableColumn],
    mins: &[f32],
    stored: &ColumnWidths,
    width: f32,
    from_model: Vec<f32>,
) -> Vec<f32> {
    let Some(live) = view_state
        .live_resize
        .as_mut()
        .filter(|l| l.player == player)
    else {
        return from_model;
    };
    if live.released.is_none() {
        let (down, pointer) = ui.input(|i| (i.pointer.primary_down(), i.pointer.latest_pos()));
        if down {
            if let Some(pos) = pointer
                && let Some(start) = live.start_px.get(live.edge)
            {
                let wanted = start + pos.x - live.start_x;
                live.px = table_layout::resize_px(&live.start_px, mins, live.edge, wanted);
            }
        } else {
            // Only a drag that moved something stores widths: a click on an
            // edge, or a drag back to where it began, does not.
            let moved = live
                .px
                .iter()
                .zip(&live.start_px)
                .any(|(a, b)| (a - b).abs() > 0.5);
            let new = table_layout::fractions_of(&live.columns, &live.px);
            if moved && new != *stored {
                scene.ctl.send(Command::SetColumnWidths(player, new));
            }
            live.released = Some((stored.clone(), scene.time + HOLD_SECS));
        }
    }
    let stale = live.columns != columns
        || (live.width - width).abs() > 0.5
        || live
            .released
            .as_ref()
            .is_some_and(|(before, until)| scene.time >= *until || stored != before);
    if stale {
        view_state.live_resize = None;
        from_model
    } else {
        live.px.clone()
    }
}

pub(crate) fn track_table(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    player: PlayerId,
    playlist: PlaylistId,
) {
    let Some(list) = scene.state.playlists.get(playlist) else {
        return;
    };
    let Ok(p) = scene.state.player(player) else {
        return;
    };
    let t = scene.i18n;
    let digits = format::number_width(list.entries.len());
    let use_markers = scene.state.config.players.use_cue_markers;
    // O24: one list of columns for every table.
    let columns = fp_model::normalize_columns(&scene.state.config.ui.table_columns);
    let mins: Vec<f32> = columns
        .iter()
        .map(|c| table_layout::column_min(*c, digits))
        .collect();
    // Proportional columns (feedback spec F6): pixel widths from the stored
    // fractions every frame, so that they follow the window (O24: fractions
    // keyed by column). While an edge is dragged the widths of all the
    // columns are recomputed from the pointer every frame and stored on
    // release (O16). The table never keeps widths of its own: every column
    // is given its exact width on every frame.
    let width = (ui.available_width() - ui.spacing().scroll.allocated_width()).max(0.0);
    let from_model = table_layout::column_px(&columns, &p.columns, width, digits);
    let px = live_widths(
        ui, scene, view_state, player, &columns, &mins, &p.columns, width, from_model,
    );
    view_state.widths.insert(player, px.clone());
    let area = ui.max_rect();
    // Read before the table's scroll area takes the wheel for itself.
    let wheel_over_table =
        ui.rect_contains_pointer(area) && ui.input(|i| i.smooth_scroll_delta != egui::Vec2::ZERO);
    let pressed_in_table = ui.rect_contains_pointer(area) && ui.input(|i| i.pointer.primary_down());
    ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
    let mut built = 0;
    let pointer = ui.ctx().pointer_hover_pos();
    let entries = &list.entries;
    let selected = view_state.selection.get(&player).copied();
    let mut clicked: Option<EntryId> = None;
    // O17: the entry a running CUE moves to after a primary click.
    let mut cue_follow: Option<EntryId> = None;
    // O23: the track whose tags the operator asked to edit.
    let mut edit_tags: Option<fp_model::TrackId> = None;
    let mut dragged: Option<EntryId> = None;
    let mut builder = TableBuilder::new(ui)
        .id_salt(("tracks", player.0))
        .striped(false)
        .resizable(false)
        .vscroll(true)
        // The table's default minimum body is 200 points: in a short window
        // the footer would cover the last rows, out of reach of the scroll.
        .min_scrolled_height(0.0)
        .auto_shrink([false, false])
        .sense(Sense::click_and_drag())
        .cell_layout(Layout::left_to_right(Align::Center));
    for w in &px {
        builder = builder.column(Column::exact(w.max(0.0)).clip(true));
    }
    // A current entry being followed: scroll its row to the top once.
    if let Some(FollowScroll {
        entry,
        align,
        animated,
    }) = view_state.follow_scroll.get(&player).copied()
    {
        match entries.iter().position(|e| e.id == entry) {
            Some(i) => {
                builder = builder
                    .scroll_to_row(i, Some(align))
                    .animate_scrolling(animated);
                view_state.follow_scroll.remove(&player);
            }
            // Not in this playlist: wait for its tab, unless it is gone.
            None if scene.state.playlists.find(entry).is_none() => {
                view_state.follow_scroll.remove(&player);
            }
            None => {}
        }
    }
    let mut menu_open = false;
    // O24: a header being dragged over another, and the move it ended in.
    let mut column_slot: Option<usize> = None;
    let mut column_move: Option<(usize, usize)> = None;
    let output = builder
        .header(HEADER_HEIGHT, |mut header| {
            for (index, column) in columns.iter().enumerate() {
                let (_, response) = header.col(|ui| {
                    let right = matches!(column, TableColumn::Duration | TableColumn::Intro);
                    header_label(ui, &t.tr(&format!("col-{}", column.name())), right);
                });
                if response.drag_started() {
                    response.dnd_set_drag_payload(DragColumn { index });
                }
                let right_half = |pos: egui::Pos2| pos.x > response.rect.center().x;
                if response.dnd_hover_payload::<DragColumn>().is_some()
                    && let Some(pos) = response.hover_pos()
                {
                    column_slot = Some(if right_half(pos) { index + 1 } else { index });
                }
                if let Some(payload) = response.dnd_release_payload::<DragColumn>() {
                    let at = response.interact_pointer_pos().or(response.hover_pos());
                    let slot = if at.is_some_and(right_half) {
                        index + 1
                    } else {
                        index
                    };
                    column_move = Some((payload.index, slot));
                }
                response.context_menu(|ui| {
                    menu_open = true;
                    header_menu(ui, scene, &columns);
                });
            }
        })
        .body(|body| {
            body.rows(ROW_HEIGHT, entries.len(), |mut row| {
                built += 1;
                let i = row.index();
                let Some(entry) = entries.get(i) else {
                    return;
                };
                let Some(track) = scene.state.library.get(entry.track) else {
                    return;
                };
                let status = view::row_status(scene.state, player, entry);
                let hi = matches!(status, RowStatus::Current | RowStatus::Next);
                let bg = match status {
                    RowStatus::Current => theme::ON_AIR_ROW,
                    RowStatus::Next => theme::NEXT_ROW,
                    _ if selected == Some(entry.id) => theme::ACCENT_900,
                    _ => Color32::TRANSPARENT,
                };
                let dimmed = status == RowStatus::Played;
                let text = if hi {
                    theme::NEUTRAL_100
                } else if dimmed {
                    theme::NEUTRAL_600
                } else {
                    theme::TEXT
                };
                let artist_color = if hi {
                    theme::NEUTRAL_100
                } else if dimmed {
                    theme::NEUTRAL_600
                } else {
                    theme::NEUTRAL_400
                };
                let row_font = if hi { font_medium(12.0) } else { font(12.0) };
                let line = |ui: &mut Ui| {
                    let r = ui.max_rect();
                    let full = Rect::from_min_max(r.min, pos2(r.max.x, r.min.y + ROW_HEIGHT));
                    ui.painter().rect_filled(full, 0.0, bg);
                    ui.painter().rect_filled(
                        Rect::from_min_size(
                            pos2(full.left(), full.bottom() - 1.0),
                            vec2(full.width(), 1.0),
                        ),
                        0.0,
                        theme::TEXT.gamma_multiply(0.06),
                    );
                };
                for column in &columns {
                    row.col(|ui| {
                        line(ui);
                        match column {
                            TableColumn::Number => {
                                ui.add_space(10.0);
                                let (glyph, color) = match status {
                                    RowStatus::Current => {
                                        let playing = p.transport == Transport::Playing;
                                        let g = if playing {
                                            egui_phosphor::fill::SPEAKER_HIGH
                                        } else {
                                            egui_phosphor::fill::PAUSE
                                        };
                                        (Some(g.to_owned()), theme::NEUTRAL_100)
                                    }
                                    RowStatus::Next => (
                                        Some(icon::ARROW_BEND_DOWN_RIGHT.to_owned()),
                                        theme::NEUTRAL_100,
                                    ),
                                    RowStatus::Unavailable => {
                                        (Some(view::file_icon(track).to_owned()), theme::AMBER)
                                    }
                                    _ => (None, theme::NEUTRAL_600),
                                };
                                if let RowStatus::OnAirElsewhere(n) = status {
                                    // Marked, not highlighted: it is another player's.
                                    let tip = scene
                                        .i18n
                                        .tr_args("tip-on-air-elsewhere", &[("n", n.into())]);
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(format!("P{n}"))
                                                .font(egui::FontId::proportional(11.0))
                                                .color(theme::ON_AIR_TEXT),
                                        )
                                        .selectable(false),
                                    )
                                    .on_hover_text(tip);
                                    return;
                                }
                                let label = match glyph {
                                    Some(g) if hi => g,
                                    Some(g) => format!("{g}{:0digits$}", i + 1),
                                    None => format!("{:0digits$}", i + 1),
                                };
                                let family = if matches!(status, RowStatus::Current) {
                                    egui::FontFamily::Name(theme::ICONS_FILL.into())
                                } else {
                                    egui::FontFamily::Proportional
                                };
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(label)
                                            .font(egui::FontId::new(12.0, family))
                                            .color(color),
                                    )
                                    .selectable(false),
                                );
                                // O37: the entry on air is also the next one.
                                if status == RowStatus::Current
                                    && view::row_is_next(scene.state, player, entry.id)
                                {
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(icon::ARROW_BEND_DOWN_RIGHT)
                                                .font(egui::FontId::proportional(12.0))
                                                .color(theme::NEUTRAL_100),
                                        )
                                        .selectable(false),
                                    )
                                    .on_hover_text(scene.i18n.tr("tip-next-again"));
                                }
                            }
                            TableColumn::Title => {
                                ui.add_space(8.0);
                                // The entry's repeat and stop icons sit before the title,
                                // in the row's text colour (feedback 2 spec O9); the
                                // "analysed by an earlier version" flag stays at the right.
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.spacing_mut().item_spacing.x = 4.0;
                                    ui.add_space(4.0);
                                    // Shown tracks are brought up to date anyway, so
                                    // the flag only stays on the ones waiting.
                                    if crate::services::outdated(track) {
                                        flag(ui, &t.tr("flag-outdated"), |p, r| {
                                            let c = theme::NEUTRAL_500;
                                            widgets::glyph(
                                                p,
                                                r,
                                                icon::ARROWS_CLOCKWISE,
                                                13.0,
                                                c,
                                                false,
                                            );
                                        });
                                    }
                                    if view::analysis_pending(track) {
                                        flag(ui, &t.tr("flag-analysis-pending"), |p, r| {
                                            widgets::glyph(
                                                p,
                                                r,
                                                icon::HOURGLASS,
                                                13.0,
                                                theme::NEUTRAL_400,
                                                false,
                                            );
                                        });
                                    }
                                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                                        ui.spacing_mut().item_spacing.x = 4.0;
                                        if entry.repeat {
                                            flag(ui, &t.tr("flag-repeat"), |p, r| {
                                                widgets::glyph(
                                                    p,
                                                    r,
                                                    icon::REPEAT,
                                                    13.0,
                                                    text,
                                                    false,
                                                );
                                            });
                                        }
                                        if entry.stop_after {
                                            flag(ui, &t.tr("flag-stop-after"), |p, r| {
                                                // Drawn into the flag's own 16x12 box, as before (paint would
                                                // centre the grid's 18x13 size instead).
                                                if let Some(d) =
                                                    glyphs::drawn(TransportAction::StopAfter)
                                                {
                                                    p.extend((d.draw)(r, text));
                                                }
                                            });
                                        }
                                        ui.add(
                                            egui::Label::new(
                                                RichText::new(&track.title)
                                                    .font(row_font.clone())
                                                    .color(text),
                                            )
                                            .selectable(false)
                                            .truncate(),
                                        );
                                    });
                                });
                            }
                            TableColumn::Artist => {
                                ui.add_space(8.0);
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(if track.artist.is_empty() {
                                            t.tr("unknown-artist")
                                        } else {
                                            track.artist.clone()
                                        })
                                        .font(font(12.0))
                                        .color(artist_color),
                                    )
                                    .selectable(false)
                                    .truncate(),
                                );
                            }
                            TableColumn::Duration | TableColumn::Intro => {
                                // Times sit at the right, like the duration.
                                let d = view::cell_text(track, *column, use_markers);
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.add_space(10.0);
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(d).font(font(12.0)).color(text),
                                        )
                                        .selectable(false),
                                    );
                                });
                            }
                            TableColumn::Album
                            | TableColumn::Date
                            | TableColumn::Genre
                            | TableColumn::FileName => {
                                ui.add_space(8.0);
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(view::cell_text(track, *column, use_markers))
                                            .font(font(12.0))
                                            .color(artist_color),
                                    )
                                    .selectable(false)
                                    .truncate(),
                                );
                            }
                        }
                    });
                }
                let response = row.response();
                if response.clicked() {
                    clicked = Some(entry.id);
                    cue_follow = view::cue_follow_target(scene.state, player, entry.id);
                }
                response.clone().on_hover_ui(|ui| {
                    track_tip(ui, scene, track);
                });
                // A double-click sets the row as next, the playing one included
                // (it then plays once more, rule 27a).
                if response.double_clicked() {
                    scene.ctl.send(Command::SetNext(player, entry.id));
                }
                if response.drag_started() {
                    response.dnd_set_drag_payload(DragEntry { entry: entry.id });
                    dragged = Some(entry.id);
                }
                response.context_menu(|ui| {
                    menu_open = true;
                    clicked = Some(entry.id);
                    if context_menu(ui, scene, player, playlist, entry.id, i, track).is_some() {
                        edit_tags = Some(track.id);
                    }
                });
            });
        });

    view_state.rows_built += built;
    if let Some(entry) = clicked.or(dragged) {
        view_state.selection.insert(player, entry);
        view_state.active_player = Some(player);
    }
    if let Some(entry) = cue_follow {
        scene.ctl.send(Command::CueEntry(player, entry));
    }
    if edit_tags.is_some() {
        view_state.edit_tags = edit_tags;
    }
    // Q4: the drop target comes from the pointer and the body's geometry,
    // never from the widget under it, and belongs to this table alone: it is
    // keyed by player and playlist.
    let body_rect = output.inner_rect;
    let scroll_y = output.state.offset.y;
    let grab = ui.style().interaction.resize_grab_radius_side;
    // A floating window over the table (the CUE window) hides the rows
    // under it: only a pointer on the table's own layer can drop.
    let layer = ui.layer_id();
    let on_table_layer = |p: &egui::Pos2| {
        ui.ctx()
            .layer_id_at(*p)
            .unwrap_or_else(egui::LayerId::background)
            == layer
    };
    let target = pointer
        .filter(on_table_layer)
        .filter(|p| !table_layout::on_column_edge(&px, area.left(), grab, p.x))
        .and_then(|p| table_layout::drop_index(body_rect, scroll_y, ROW_HEIGHT, entries.len(), p));
    if egui::DragAndDrop::has_payload_of_type::<DragEntry>(ui.ctx()) {
        match target {
            Some(index) => {
                view_state.drop = Some(DropTarget {
                    player,
                    playlist,
                    index,
                });
                // Drawn after the body, from the same geometry: at the
                // boundary even when the rows next to it were not built.
                if let Some(y) = table_layout::boundary_y(body_rect, scroll_y, ROW_HEIGHT, index) {
                    let top = if index > 0 && index == entries.len() {
                        y - 2.0
                    } else {
                        y
                    };
                    ui.painter().with_clip_rect(body_rect).rect_filled(
                        Rect::from_min_size(
                            pos2(body_rect.left(), top),
                            vec2(body_rect.width(), 2.0),
                        ),
                        0.0,
                        theme::ACCENT,
                    );
                }
            }
            None if view_state
                .drop
                .is_some_and(|d| d.player == player && d.playlist == playlist) =>
            {
                view_state.drop = None;
            }
            None => {}
        }
        if ui.input(|i| i.pointer.any_released())
            && let Some(index) = target
            && let Some(payload) = egui::DragAndDrop::payload::<DragEntry>(ui.ctx())
        {
            scene.ctl.send(Command::MoveEntry {
                entry: payload.entry,
                to: playlist,
                index,
            });
            view_state.drop = None;
        }
    }
    // OS file drops follow the same rules (Q4.6).
    if let Some(index) = target {
        view_state.file_drop = Some(DropTarget {
            player,
            playlist,
            index,
        });
    }
    resize_handles(ui, view_state, player, &columns, &px, area);
    if let Some((from, slot)) = column_move {
        scene.set_table_columns(fp_model::move_column_before(&columns, from, slot));
    }
    if egui::DragAndDrop::has_payload_of_type::<DragColumn>(ui.ctx()) {
        ui.set_cursor_icon(egui::CursorIcon::Grabbing);
        // Where the dragged header would land: a line on that column edge.
        if let Some(slot) = column_slot {
            let x = area.left() + px.iter().take(slot).sum::<f32>();
            ui.painter().rect_filled(
                Rect::from_min_size(pos2(x - 1.0, area.top()), vec2(2.0, HEADER_HEIGHT)),
                0.0,
                theme::ACCENT,
            );
        }
    }
    // The operator is using the table: scrolling it (wheel or scroll bar),
    // pressing in it, dragging an entry (for as long as the drag lasts), or
    // with a row menu open. Following waits (feedback spec F18).
    let entry_drag = egui::DragAndDrop::has_payload_of_type::<DragEntry>(ui.ctx());
    if wheel_over_table || pressed_in_table || menu_open || dragged.is_some() || entry_drag {
        view_state.table_touched.insert(player, scene.time);
    }
}

/// The grab zones on the column edges (feedback 2 spec O16): they start a
/// drag, which `live_widths` follows from the next frame on, and draw the
/// separator lines. They sit over the rows, as the table's own did.
fn resize_handles(
    ui: &mut Ui,
    view_state: &mut ViewState,
    player: PlayerId,
    columns: &[TableColumn],
    px: &[f32],
    area: Rect,
) {
    let grab = ui.style().interaction.resize_grab_radius_side;
    let mut x = area.left();
    for (edge, w) in px.iter().take(columns.len().saturating_sub(1)).enumerate() {
        x += w;
        let rect = Rect::from_min_max(pos2(x - grab, area.top()), pos2(x + grab, area.bottom()));
        let id = ui.id().with(("column-edge", player.0, edge));
        let response = ui.interact(rect, id, Sense::drag());
        let dragging = view_state
            .live_resize
            .as_ref()
            .is_some_and(|l| l.player == player && l.edge == edge && l.released.is_none());
        if response.drag_started() {
            let start_x = ui
                .input(|i| i.pointer.press_origin())
                .map_or(x, |origin| origin.x);
            view_state.live_resize = Some(LiveResize {
                player,
                edge,
                columns: columns.to_vec(),
                width: px.iter().sum(),
                start_px: px.to_vec(),
                start_x,
                px: px.to_vec(),
                released: None,
            });
        }
        let hot = (response.hovered() && !ui.input(|i| i.pointer.any_down())) || dragging;
        if hot {
            ui.set_cursor_icon(egui::CursorIcon::ResizeColumn);
        }
        let visuals = ui.visuals();
        let stroke = if dragging {
            visuals.widgets.active.bg_stroke
        } else if hot {
            visuals.widgets.hovered.bg_stroke
        } else {
            visuals.widgets.noninteractive.bg_stroke
        };
        ui.painter()
            .line_segment([pos2(x, area.top()), pos2(x, area.bottom())], stroke);
    }
}

/// The menu of the table header: shows and hides the optional columns
/// (feedback 2 spec O24). Title and Duration are always shown, so they are
/// not offered.
fn header_menu(ui: &mut Ui, scene: &Scene<'_>, columns: &[TableColumn]) {
    let t = scene.i18n;
    ui.set_min_width(200.0);
    for column in TableColumn::ALL.into_iter().filter(|c| !c.is_required()) {
        let mut shown = columns.contains(&column);
        let label = t.tr(&format!("column-name-{}", column.name()));
        if ui.checkbox(&mut shown, label).changed() {
            scene.set_table_columns(fp_model::with_column_shown(columns, column, shown));
            ui.close();
        }
    }
}

fn context_menu(
    ui: &mut Ui,
    scene: &Scene<'_>,
    player: PlayerId,
    playlist: PlaylistId,
    entry: EntryId,
    index: usize,
    track: &fp_model::Track,
) -> Option<fp_model::TrackId> {
    let t = scene.i18n;
    let mut edit_tags = None;
    ui.set_min_width(240.0);
    ui.add(
        egui::Label::new(
            RichText::new(&track.title)
                .font(font(11.0))
                .color(theme::NEUTRAL_400),
        )
        .selectable(false)
        .truncate(),
    );
    ui.separator();
    // Players are independent: only this player's own current entry cannot
    // be chosen; removing needs the entry off air everywhere (rule 13).
    let own_current = scene
        .state
        .player(player)
        .is_ok_and(|p| p.current == Some(entry));
    // O37: the entry on air can be set as next, unless it already is.
    let self_next = scene
        .state
        .player(player)
        .is_ok_and(|p| p.next == Some(entry) && p.next_explicit);
    let on_air = scene.state.is_on_air(entry);
    // The item's text is its accessible label; tests find it by the plain text.
    let labelled = |ui: &mut Ui, glyph: &str, key: &str, enabled: bool| {
        let text = t.tr(key);
        let response = ui.add_enabled(enabled, egui::Button::new(format!("{glyph}  {text}")));
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, text.clone())
        });
        response
    };
    let state = scene.state.player(player).ok();
    let fading = state.is_some_and(|p| p.fading);
    if labelled(
        ui,
        glyphs::glyph_text(TransportAction::Play),
        "menu-play-now",
        !own_current && !fading,
    )
    .clicked()
    {
        // Play resumes a paused track (rule 4): stop it first so that the
        // chosen entry is the one that starts.
        if state.is_some_and(|p| p.transport == Transport::Paused) {
            scene.ctl.send(Command::Stop(player));
        }
        scene.ctl.send(Command::SetNext(player, entry));
        scene.ctl.send(Command::Play(player));
        ui.close();
    }
    if labelled(
        ui,
        icon::ARROW_BEND_DOWN_RIGHT,
        "menu-set-next",
        !(own_current && self_next),
    )
    .clicked()
    {
        scene.ctl.send(Command::SetNext(player, entry));
        ui.close();
    }
    let can_cue = scene.state.config.outputs.player_has_cue(player);
    if labelled(
        ui,
        glyphs::glyph_text(TransportAction::Cue),
        "menu-cue",
        can_cue,
    )
    .on_disabled_hover_text(t.tr("tip-cue-no-output"))
    .clicked()
    {
        scene.ctl.send(Command::CueEntry(player, entry));
        ui.close();
    }
    if labelled(ui, icon::ARROWS_CLOCKWISE, "menu-reanalyse", true).clicked() {
        scene.request(crate::services::ServiceRequest::ReanalyseTrack(track.id));
        ui.close();
    }
    let block = view::tag_edit_availability(scene.state, track.id);
    let edit = labelled(ui, icon::PENCIL_SIMPLE, "menu-edit-tags", block.is_none());
    let edit = match block {
        Some(reason) => edit.on_disabled_hover_text(t.tr(super::tag_editor::block_key(reason))),
        None => edit,
    };
    if edit.clicked() {
        edit_tags = Some(track.id);
        ui.close();
    }
    ui.separator();
    if labelled(ui, icon::PLUS, "menu-add-below", true).clicked() {
        scene.pick_files(playlist, index + 1);
        ui.close();
    }
    if labelled(ui, icon::COPY, "menu-duplicate", true).clicked() {
        scene.ctl.send(Command::DuplicateEntry(entry));
        ui.close();
    }
    ui.separator();
    // Checkable: each shows whether the entry has the flag (R26, R27).
    let flags = scene.state.playlists.entry(entry);
    let mut repeat = flags.is_some_and(|e| e.repeat);
    if ui.checkbox(&mut repeat, t.tr("menu-repeat")).clicked() {
        scene.ctl.send(Command::ToggleEntryRepeat(entry));
        ui.close();
    }
    let mut stop_after = flags.is_some_and(|e| e.stop_after);
    if ui
        .checkbox(&mut stop_after, t.tr("menu-stop-after"))
        .clicked()
    {
        scene.ctl.send(Command::ToggleEntryStopAfter(entry));
        ui.close();
    }
    let others: Vec<_> = scene
        .state
        .playlists
        .iter()
        .filter(|pl| pl.id != playlist)
        .map(|pl| (pl.id, pl.name.clone(), pl.entries.len()))
        .collect();
    if !others.is_empty() {
        ui.menu_button(
            format!("{}  {}", icon::ARROW_RIGHT, t.tr("menu-move-to")),
            |ui| {
                for (id, name, len) in others {
                    if ui.button(name).clicked() {
                        scene.ctl.send(Command::MoveEntry {
                            entry,
                            to: id,
                            index: len,
                        });
                        ui.close();
                    }
                }
            },
        );
    }
    ui.separator();
    let remove = labelled(ui, icon::TRASH, "menu-remove", !on_air);
    let remove = if on_air {
        remove.on_disabled_hover_text(t.tr("menu-remove-on-air"))
    } else {
        remove
    };
    if remove.clicked() {
        scene.ctl.send(Command::RemoveEntry(entry));
        ui.close();
    }
    edit_tags
}

/// A small flag icon with `label` as its accessible name and tooltip.
fn flag(ui: &mut Ui, label: &str, paint: impl FnOnce(&egui::Painter, Rect)) {
    let (rect, response) = ui.allocate_exact_size(vec2(16.0, 12.0), Sense::hover());
    let owned = label.to_owned();
    response
        .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, owned.clone()));
    if ui.is_rect_visible(rect) {
        paint(ui.painter(), rect);
    }
    response.on_hover_text(label);
}

/// The row tooltip: the reason the file cannot be played, when there is one,
/// then label and value per line. Fixed width, never wider than the window.
fn track_tip(ui: &mut Ui, scene: &Scene<'_>, track: &fp_model::Track) {
    let t = scene.i18n;
    ui.set_width(TIP_WIDTH.min((ui.ctx().content_rect().width() - 16.0).max(120.0)));
    let lines = view::track_tooltip(track, scene.file_tip(track.id).as_deref());
    let mut rest = lines.as_slice();
    if let Some(((view::TipField::Problem, reason), tail)) = rest.split_first() {
        ui.add(
            egui::Label::new(
                RichText::new(format!("{}  {reason}", view::file_icon(track)))
                    .font(font(12.0))
                    .color(theme::AMBER),
            )
            .wrap(),
        );
        ui.add_space(4.0);
        rest = tail;
    }
    // No `Grid`: it learns its column widths from the previous frame, so the
    // popup would be laid out three times and move on each (Q9.3). The label
    // column is measured here instead, so one pass gives the final layout.
    let keys: Vec<(&'static str, &String)> = rest
        .iter()
        .filter_map(|(field, value)| {
            let key = match field {
                view::TipField::Problem => return None,
                view::TipField::Title => "tip-field-title",
                view::TipField::Artist => "tip-field-artist",
                view::TipField::Album => "tip-field-album",
                view::TipField::Date => "tip-field-date",
                view::TipField::Genre => "tip-field-genre",
                view::TipField::Duration => "tip-field-duration",
                view::TipField::Format => "tip-field-format",
                view::TipField::Path => "tip-field-path",
            };
            Some((key, value))
        })
        .collect();
    let key_font = font(11.0);
    let key_width = keys
        .iter()
        .map(|(key, _)| {
            ui.painter()
                .layout_no_wrap(t.tr(key), key_font.clone(), theme::NEUTRAL_400)
                .size()
                .x
        })
        .fold(0.0_f32, f32::max);
    for (key, value) in keys {
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;
            ui.add_sized(
                vec2(key_width, 14.0),
                egui::Label::new(
                    RichText::new(t.tr(key))
                        .font(key_font.clone())
                        .color(theme::NEUTRAL_400),
                ),
            );
            ui.add(
                egui::Label::new(
                    RichText::new(value.as_str())
                        .font(font(12.0))
                        .color(theme::TEXT),
                )
                .wrap(),
            );
        });
    }
}
