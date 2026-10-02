//! The track table of a player column (spec §8.3): virtualised rows,
//! resizable columns, row colours, context menu and drag and drop.

use egui::{Align, Color32, Layout, Rect, RichText, Sense, Ui, pos2, vec2};
use egui_extras::{Column, TableBuilder};
use egui_phosphor::regular as icon;
use fp_model::{ColumnWidths, Command, EntryId, PlayerId, PlaylistId, TableColumn, Transport};

use super::app::{DragEntry, DropTarget, FollowScroll, Scene, ViewState};
use super::format;
use super::glyphs::{self, TransportAction};
use super::table_layout;
use super::theme;
use super::view::{self, RowStatus};
use super::widgets::{self, font, font_medium};

const HEADER_HEIGHT: f32 = 24.0;
const ROW_HEIGHT: f32 = 28.0;
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
    let mut hovered_row: Option<(usize, bool)> = None;
    let mut pointer_row: Option<(usize, bool)> = None;
    let pointer = ui.ctx().pointer_hover_pos();
    let mut released: Option<(DragEntry, usize)> = None;
    let entries = &list.entries;
    let drop = view_state.drop.filter(|d| d.playlist == playlist);
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
    builder
        .header(HEADER_HEIGHT, |mut header| {
            for column in &columns {
                header.col(|ui| {
                    let right = matches!(column, TableColumn::Duration | TableColumn::Intro);
                    header_label(ui, &t.tr(&format!("col-{}", column.name())), right);
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
                    if let Some(d) = drop {
                        let y = if d.index == i {
                            Some(full.top())
                        } else if d.index == entries.len() && i + 1 == entries.len() {
                            Some(full.bottom() - 2.0)
                        } else {
                            None
                        };
                        if let Some(y) = y {
                            ui.painter().rect_filled(
                                Rect::from_min_size(pos2(full.left(), y), vec2(full.width(), 2.0)),
                                0.0,
                                theme::ACCENT,
                            );
                        }
                    }
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
                                let number = ui.add(
                                    egui::Label::new(
                                        RichText::new(label)
                                            .font(egui::FontId::new(12.0, family))
                                            .color(color),
                                    )
                                    .selectable(false),
                                );
                                if status == RowStatus::Unavailable
                                    && let Some(tip) = scene.file_tip(entry.track)
                                {
                                    number.on_hover_text(tip);
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
                if response.double_clicked() && status != RowStatus::Current {
                    scene.ctl.send(Command::SetNext(player, entry.id));
                }
                if response.drag_started() {
                    response.dnd_set_drag_payload(DragEntry { entry: entry.id });
                    dragged = Some(entry.id);
                }
                if let Some(p) = pointer.filter(|p| response.rect.contains(*p)) {
                    pointer_row = Some((i, p.y > response.rect.center().y));
                }
                if response.dnd_hover_payload::<DragEntry>().is_some()
                    && let Some(pos) = response.hover_pos()
                {
                    hovered_row = Some((i, pos.y > response.rect.center().y));
                }
                if let Some(payload) = response.dnd_release_payload::<DragEntry>() {
                    let below = response
                        .interact_pointer_pos()
                        .or(response.hover_pos())
                        .is_some_and(|p| p.y > response.rect.center().y);
                    released = Some((*payload, if below { i + 1 } else { i }));
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
    // Drop target for entries dragged inside the app.
    let pointer_in = ui
        .ctx()
        .pointer_hover_pos()
        .is_some_and(|p| area.contains(p));
    if egui::DragAndDrop::has_payload_of_type::<DragEntry>(ui.ctx()) {
        match hovered_row {
            Some((i, below)) => {
                view_state.drop = Some(DropTarget {
                    playlist,
                    index: if below { i + 1 } else { i },
                });
            }
            None if pointer_in => {
                view_state.drop = Some(DropTarget {
                    playlist,
                    index: entries.len(),
                });
            }
            None => {
                if view_state.drop.is_some_and(|d| d.playlist == playlist) {
                    view_state.drop = None;
                }
            }
        }
    }
    if let Some((payload, index)) = released {
        scene.ctl.send(Command::MoveEntry {
            entry: payload.entry,
            to: playlist,
            index,
        });
        view_state.drop = None;
    } else if pointer_in
        && ui.input(|i| i.pointer.any_released())
        && let Some(payload) = egui::DragAndDrop::payload::<DragEntry>(ui.ctx())
    {
        // Released below the last row.
        scene.ctl.send(Command::MoveEntry {
            entry: payload.entry,
            to: playlist,
            index: entries.len(),
        });
        view_state.drop = None;
    }
    // OS file drops land at the hovered row, or at the end.
    if pointer_in {
        view_state.file_drop = Some(DropTarget {
            playlist,
            index: pointer_row
                .map(|(i, below)| if below { i + 1 } else { i })
                .unwrap_or(entries.len()),
        });
    }
    resize_handles(ui, view_state, player, &columns, &px, area);
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
        !own_current,
    )
    .clicked()
    {
        scene.ctl.send(Command::SetNext(player, entry));
        ui.close();
    }
    if labelled(
        ui,
        glyphs::glyph_text(TransportAction::Cue),
        "menu-cue",
        true,
    )
    .clicked()
    {
        scene.ctl.send(Command::CueEntry(player, entry));
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

/// The row tooltip: label and value per line, at most as wide as the window.
fn track_tip(ui: &mut Ui, scene: &Scene<'_>, track: &fp_model::Track) {
    let t = scene.i18n;
    ui.set_max_width(420.0);
    egui::Grid::new("track-tip")
        .num_columns(2)
        .spacing(vec2(10.0, 3.0))
        .show(ui, |ui| {
            for (field, value) in view::track_tooltip(track) {
                let key = match field {
                    view::TipField::Title => "tip-field-title",
                    view::TipField::Artist => "tip-field-artist",
                    view::TipField::Album => "tip-field-album",
                    view::TipField::Date => "tip-field-date",
                    view::TipField::Genre => "tip-field-genre",
                    view::TipField::Duration => "tip-field-duration",
                    view::TipField::Format => "tip-field-format",
                    view::TipField::Path => "tip-field-path",
                };
                ui.label(
                    RichText::new(t.tr(key))
                        .font(font(11.0))
                        .color(theme::NEUTRAL_400),
                );
                ui.add(
                    egui::Label::new(RichText::new(value).font(font(12.0)).color(theme::TEXT))
                        .wrap(),
                );
                ui.end_row();
            }
        });
}
