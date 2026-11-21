//! The Log column's list (automation spec §8.1) and the strip above it;
//! its transport is the player header's (automation rework AR3).

use egui::{Align, Align2, Color32, Layout, RichText, Sense, Ui, UiBuilder, vec2};
use egui_phosphor::regular as icon;
use fp_model::Command;
use fp_model::automation::{ItemRef, RowStatus, ScheduleRow, schedule_view};
use fp_model::schedule::{BlockOrigin, Item};

use super::app::{DragTracks, Scene, ViewState};
use super::exit_guard::button;
use super::format;
use super::table_layout;
use super::theme;
use super::widgets::{self, TileStyle, font, font_medium, font_semibold};
use fp_model::ids::TemplateId;
use fp_model::log_edit::{EditTarget, LogEdit};

/// The Log's strip where a player has its playlist tabs (`height` tall):
/// the date it plays. Its transport is the player header's (rework AR3);
/// its templates are edited in Settings → Automation (AR20).
pub(crate) fn schedule_bar(ui: &mut Ui, scene: &Scene<'_>, height: f32) {
    if scene.state.automation.is_none() {
        return;
    }
    let (strip, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
    ui.painter().rect_filled(strip, 0.0, theme::NEUTRAL_900);
    let mut bar = ui.new_child(
        UiBuilder::new()
            .max_rect(strip.shrink2(vec2(10.0, 0.0)))
            .layout(Layout::left_to_right(Align::Center)),
    );
    bar.label(
        RichText::new(scene.state.local_now.date().format("%Y-%m-%d").to_string())
            .font(font(11.0))
            .color(theme::NEUTRAL_400),
    );
}

/// The Log's list (rework AR14), over the rows of `schedule_view`.
pub(crate) fn schedule_list(ui: &mut Ui, scene: &Scene<'_>, view_state: &mut ViewState) {
    let rows = schedule_view(scene.state);
    log_table(ui, scene, view_state, &rows, None, true);
}

/// What a row of a table edits: the template it shows, else the row's
/// date (the Log, or a date's popup).
fn target_of(template: Option<TemplateId>, date: chrono::NaiveDate) -> EditTarget {
    template.map_or(EditTarget::Date(date), EditTarget::Template)
}

/// A row selected on the Log (`log`) or in a popup (rework AR19c): an
/// item of `block`, or its header (`index: None`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LogSelection {
    pub log: bool,
    pub target: EditTarget,
    pub block: fp_model::BlockId,
    pub index: Option<usize>,
}

impl LogSelection {
    /// The selection of `row`, in a list of `template` (else of dates).
    fn of(row: &ScheduleRow, log: bool, template: Option<TemplateId>) -> Self {
        match row {
            ScheduleRow::Block { date, block, .. } => Self {
                log,
                target: target_of(template, *date),
                block: *block,
                index: None,
            },
            ScheduleRow::Item { item, .. } => Self {
                log,
                target: target_of(template, item.date),
                block: item.block,
                index: Some(item.index),
            },
        }
    }

    /// Whether `row` of a list of `template` is this one.
    fn is_row(&self, row: &ScheduleRow, template: Option<TemplateId>) -> bool {
        let LogSelection {
            target,
            block,
            index,
            ..
        } = Self::of(row, self.log, template);
        (target, block, index) == (self.target, self.block, self.index)
    }
}

/// A click on a row selects it (AR19c).
fn select_on_click(
    view_state: &mut ViewState,
    response: &egui::Response,
    log: bool,
    row: &ScheduleRow,
    template: Option<TemplateId>,
) {
    if response.clicked() || response.secondary_clicked() {
        view_state.log_selection = Some(LogSelection::of(row, log, template));
    }
}

/// The selected row's outline, as on a playlist.
fn outline(response: &egui::Response) {
    response
        .ctx
        .layer_painter(response.layer_id)
        .with_clip_rect(response.interact_rect)
        .rect_stroke(
            response.rect,
            0.0,
            egui::Stroke::new(2.0, theme::SELECTION_OUTLINE),
            egui::StrokeKind::Inside,
        );
}

/// Where Add puts items when a row of this list is selected (AR19c):
/// just below it, at its block's start for a header; `None` when nothing
/// here is selected or that place may not change. `log`: the Log's list,
/// else a popup's of `popup`.
fn selected_place(
    scene: &Scene<'_>,
    view_state: &ViewState,
    log: bool,
    popup: EditTarget,
) -> Option<Place> {
    let s = view_state
        .log_selection
        .filter(|s| s.log == log && (log || s.target == popup))?;
    let items = items_of(scene, s.target, s.block)?;
    let index = match s.index {
        Some(i) if i < items.len() => i + 1,
        Some(_) => return None,
        None => 0,
    };
    (!locked(scene, s.target, s.block, index)).then_some((s.target, s.block, index))
}

/// Where items are added: before `index` of a block of a date or a
/// template (`usize::MAX`: at its end).
pub(crate) type Place = (EditTarget, fp_model::BlockId, usize);

/// Sends the insert of `items` at `place`, and selects the last of them
/// once added, brought into view (AR19c), so the next Add follows it.
fn insert_at(
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    log: bool,
    place: Place,
    items: Vec<Item>,
) {
    let (target, block, index) = place;
    let landing = items_of(scene, target, block)
        .and_then(|list| fp_model::log_edit::landing(&list, index, &items));
    send_edit(
        scene,
        target,
        LogEdit::Insert {
            block,
            index,
            items,
        },
    );
    if let Some(index) = landing {
        let added = LogSelection {
            log,
            target,
            block,
            index: Some(index),
        };
        view_state.log_selection = Some(added);
        view_state.log_reveal = Some(added);
    }
}

/// Whether a row may not change (AR19): only a date's rows are locked.
fn locked(scene: &Scene<'_>, target: EditTarget, block: fp_model::BlockId, index: usize) -> bool {
    match target {
        EditTarget::Date(date) => fp_model::log_edit::is_locked(scene.state, date, block, index),
        EditTarget::Template(_) => false,
    }
}

/// The table of the Log and of the template and date popups (rework AR14,
/// AR20–AR22): the playlists' table look, a Time column, block header
/// rows, then each block's items. When `playing` (the Log): on air and
/// next as on a player, missed struck through (AU23), a late next in amber
/// (AR11), the date of yesterday's and tomorrow's blocks, and the menu
/// that sets next. `template`: the rows are that template's, else each
/// row edits its date.
pub(crate) fn log_table(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    rows: &[ScheduleRow],
    template: Option<TemplateId>,
    playing: bool,
) {
    use egui_extras::{Column, TableBuilder};
    let t = scene.i18n;
    let state = scene.state;
    let today = state.local_now.date();
    let has_today = rows
        .iter()
        .any(|r| matches!(r, ScheduleRow::Block { date, .. } if *date == today || !playing));
    // Scroll to the row on air, else next, once each time it changes.
    let target = rows.iter().find_map(|r| match r {
        ScheduleRow::Item {
            item,
            status: RowStatus::OnAir,
            ..
        } => Some(*item),
        _ => None,
    });
    let target = target
        .or_else(|| state.automation.as_ref().and_then(|a| a.next))
        .filter(|_| playing);
    let memory = ui.scope_id().with("schedule-scrolled");
    let scrolled: Option<ItemRef> = ui.data(|d| d.get_temp(memory)).flatten();
    let scroll_now = target.is_some() && target != scrolled;
    if !has_today {
        ui.add_space(6.0);
        let empty = if playing {
            "schedule-empty"
        } else {
            "plan-window-empty"
        };
        ui.label(
            RichText::new(t.tr(empty))
                .font(font(12.0))
                .color(theme::NEUTRAL_500),
        );
    }
    ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
    let width = (ui.available_width() - ui.spacing().scroll.allocated_width()).max(0.0);
    // The playlists' column widths (O16): stored as the Log player's,
    // shared by the Log and its popups, resized by dragging an edge.
    let owner = state.automation.as_ref().map(|a| a.player);
    let stored = owner
        .and_then(|p| state.player(p).ok())
        .map(|p| p.columns.clone())
        .unwrap_or_default();
    let from_model = table_layout::log_px(&stored, width);
    let px = match owner {
        Some(owner) => super::table::live_widths(
            ui,
            scene,
            view_state,
            owner,
            &table_layout::LOG_COLUMNS,
            &table_layout::log_mins(),
            &stored,
            width,
            from_model,
        ),
        None => from_model,
    };
    let area = egui::Rect::from_min_max(ui.cursor().min, ui.max_rect().max);
    let column = |i: usize| Column::exact(px.get(i).copied().unwrap_or(0.0).max(0.0)).clip(true);
    let mut builder = TableBuilder::new(ui)
        .id_salt(("schedule-list", template, playing))
        .striped(false)
        .resizable(false)
        .vscroll(true)
        .min_scrolled_height(0.0)
        .auto_shrink([false, false])
        // On touch a finger drag scrolls the list (mobile spec §6), as in
        // the playlists; rows are moved with a mouse.
        .sense(if view_state.touch {
            Sense::click()
        } else {
            Sense::click_and_drag()
        })
        .cell_layout(Layout::left_to_right(Align::Center))
        .column(column(0))
        .column(column(1))
        .column(column(2))
        .column(column(3));
    // A row just added comes into view (AR19c), else the row on air.
    let reveal = view_state
        .log_reveal
        .filter(|r| r.log == playing)
        .and_then(|r| rows.iter().position(|row| r.is_row(row, template)));
    if let Some(index) = reveal {
        view_state.log_reveal = None;
        builder = builder.scroll_to_row(index, None);
    } else if scroll_now
        && let Some(index) = rows
            .iter()
            .position(|r| matches!(r, ScheduleRow::Item { item, .. } if Some(*item) == target))
    {
        builder = builder.scroll_to_row(index, Some(Align::Center));
    }
    builder
        .header(super::table::HEADER_HEIGHT, |mut header| {
            for (key, right) in [
                ("schedule-time", false),
                ("col-title", false),
                ("col-artist", false),
                ("col-duration", true),
            ] {
                header.col(|ui| super::track_cells::header_label(ui, &t.tr(key), right));
            }
        })
        .body(|body| {
            body.rows(super::table::ROW_HEIGHT, rows.len(), |mut row| {
                let Some(r) = rows.get(row.index()) else {
                    return;
                };
                match r {
                    ScheduleRow::Block {
                        date,
                        block,
                        start,
                        origin,
                        count,
                        ..
                    } => {
                        let edits = target_of(template, *date);
                        block_row(&mut row, scene, r, playing.then_some(today));
                        let response = row.response();
                        let header = BlockHeader {
                            target: edits,
                            block: *block,
                            start: *start,
                            origin: *origin,
                            count: *count,
                        };
                        block_actions(scene, view_state, &response, header, playing);
                        select_on_click(view_state, &response, playing, r, template);
                        drop_tracks(
                            scene, view_state, &response, edits, *block, 0, false, playing,
                        );
                        if view_state
                            .log_selection
                            .is_some_and(|s| s.log == playing && s.is_row(r, template))
                        {
                            outline(&response);
                        }
                    }
                    ScheduleRow::Item { item, what, .. } => {
                        let edits = target_of(template, item.date);
                        item_row(&mut row, scene, r);
                        let response = row.response();
                        row_actions(scene, &response, edits, *item, playing);
                        select_on_click(view_state, &response, playing, r, template);
                        if response.drag_started() && !locked(scene, edits, item.block, item.index)
                        {
                            response.dnd_set_drag_payload(DragLogItem {
                                target: edits,
                                block: item.block,
                                index: item.index,
                            });
                        }
                        // Nothing goes after a Music fill row (AU6).
                        let fill = matches!(what, Item::Rotation { .. });
                        drop_tracks(
                            scene, view_state, &response, edits, item.block, item.index, !fill,
                            playing,
                        );
                        if view_state
                            .log_selection
                            .is_some_and(|s| s.log == playing && s.is_row(r, template))
                        {
                            outline(&response);
                        }
                    }
                }
            });
        });
    if scroll_now {
        ui.data_mut(|d| d.insert_temp(memory, target));
    }
    if let Some(owner) = owner {
        super::table::resize_handles(ui, view_state, owner, &table_layout::LOG_COLUMNS, &px, area);
    }
}

/// A cell's text, left (or right) aligned, never selectable (the row
/// takes the clicks).
fn cell(ui: &mut Ui, text: RichText, right: bool) -> egui::Response {
    let label = egui::Label::new(text).selectable(false).truncate();
    if right {
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add_space(8.0);
            ui.add(label)
        })
        .inner
    } else {
        ui.add_space(8.0);
        ui.add(label)
    }
}

/// The background of one cell of a row, as the playlists paint it.
fn row_fill(ui: &Ui, fill: Color32) {
    let r = ui.max_rect();
    let full = egui::Rect::from_min_max(
        r.min,
        egui::pos2(r.max.x, r.min.y + super::table::ROW_HEIGHT),
    );
    super::track_cells::row_background(ui, full, fill);
}

/// A block's header row: its start, "Block · n items · length", the
/// date when not today, "changed for this day" and AU37's overrun.
fn block_row(
    row: &mut egui_extras::TableRow<'_, '_>,
    scene: &Scene<'_>,
    r: &ScheduleRow,
    today: Option<chrono::NaiveDate>,
) {
    let t = scene.i18n;
    let ScheduleRow::Block {
        date,
        start,
        origin,
        status,
        total_secs,
        overrun_secs,
        count,
        ..
    } = r
    else {
        return;
    };
    let colour = match status {
        RowStatus::Missed | RowStatus::Aired => theme::NEUTRAL_500,
        _ => theme::TEXT,
    };
    let mut text = t.tr_args(
        "schedule-block-header",
        &[
            ("count", (*count as f64).into()),
            ("length", format::clock(*total_secs).into()),
        ],
    );
    match today {
        Some(today) if *date < today => {
            text = format!("{text} · {}", t.tr("schedule-yesterday"));
        }
        Some(today) if *date > today => {
            text = format!("{text} · {}", t.tr("schedule-tomorrow"));
        }
        _ => {}
    }
    if *origin != BlockOrigin::Template {
        text = format!("{text} · {}", t.tr("schedule-changed-for-date"));
    }
    row.col(|ui| {
        row_fill(ui, theme::NEUTRAL_900);
        cell(
            ui,
            RichText::new(start.format("%H:%M").to_string())
                .font(font_semibold(12.0))
                .color(colour),
            false,
        );
    });
    row.col(|ui| {
        row_fill(ui, theme::NEUTRAL_900);
        cell(
            ui,
            RichText::new(text).font(font_semibold(12.0)).color(colour),
            false,
        );
    });
    row.col(|ui| {
        row_fill(ui, theme::NEUTRAL_900);
        if let Some(excess) = overrun_secs {
            let over = t.tr_args(
                "schedule-block-overrun",
                &[("time", format::clock(*excess).into())],
            );
            cell(
                ui,
                RichText::new(over).font(font(12.0)).color(theme::AMBER),
                false,
            );
        }
    });
    row.col(|ui| row_fill(ui, theme::NEUTRAL_900));
}

/// A row's menu. On the Log (`playing`), setting next: a double-click or
/// the menu's "Play next"; on a next set by hand, "Back to the clock"
/// (AU16). Then the edits (AR17), unless the row is locked (AR19).
fn row_actions(
    scene: &Scene<'_>,
    response: &egui::Response,
    target: EditTarget,
    item: ItemRef,
    playing: bool,
) {
    let t = scene.i18n;
    if playing && response.double_clicked() {
        scene.ctl.send(Command::SetScheduleNext(item));
    }
    let by_hand = scene
        .state
        .automation
        .as_ref()
        .is_some_and(|a| a.next_by_hand && a.next == Some(item));
    let editable = !locked(scene, target, item.block, item.index);
    if !playing && !editable {
        return;
    }
    response.context_menu(|ui| {
        if playing {
            if ui.button(t.tr("schedule-set-next")).clicked() {
                scene.ctl.send(Command::SetScheduleNext(item));
                ui.close();
            }
            if by_hand && ui.button(t.tr("schedule-clear-next")).clicked() {
                scene.ctl.send(Command::ClearScheduleNext);
                ui.close();
            }
        }
        if !editable {
            return;
        }
        let what =
            items_of(scene, target, item.block).and_then(|items| items.get(item.index).cloned());
        if playing {
            ui.separator();
        }
        if let Some(Item::Track { time_critical, .. } | Item::Category { time_critical, .. }) = what
        {
            let mut on = time_critical;
            if ui
                .checkbox(&mut on, t.tr("schedule-item-time-critical-check"))
                .clicked()
            {
                send_edit(
                    scene,
                    target,
                    LogEdit::SetHardStart {
                        block: item.block,
                        index: item.index,
                        on,
                    },
                );
                ui.close();
            }
        }
        let remove = if playing {
            "schedule-row-remove"
        } else {
            "schedule-row-remove-plan"
        };
        if ui.button(t.tr(remove)).clicked() {
            send_edit(
                scene,
                target,
                LogEdit::Remove {
                    block: item.block,
                    index: item.index,
                },
            );
            ui.close();
        }
    });
}

/// A block's items on the target.
fn items_of(scene: &Scene<'_>, target: EditTarget, block: fp_model::BlockId) -> Option<Vec<Item>> {
    match target {
        EditTarget::Date(date) => fp_model::automation::block_items(scene.state, date, block),
        EditTarget::Template(template) => scene
            .state
            .schedule
            .template(template)
            .and_then(|t| t.block(block))
            .map(|b| b.items.clone()),
    }
}

fn send_edit(scene: &Scene<'_>, target: EditTarget, edit: LogEdit) {
    scene.ctl.send(Command::EditPlan { target, edit });
}

/// What a block header's menu acts on.
#[derive(Debug, Clone, Copy)]
struct BlockHeader {
    target: EditTarget,
    block: fp_model::BlockId,
    start: chrono::NaiveDateTime,
    origin: BlockOrigin,
    count: usize,
}

/// A block header's menu, on a click or a right-click: add a track, a
/// category track or a Music fill at its end, unless it already played
/// (AR19); then Change time…, Remove block and Back to the template
/// (AR17), unless it is locked.
fn block_actions(
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    response: &egui::Response,
    header: BlockHeader,
    log: bool,
) {
    let BlockHeader {
        target,
        block,
        start,
        origin,
        count,
    } = header;
    let addable = open_at_end(scene, target, block, count);
    let editable = !locked(scene, target, block, 0);
    if !addable && !editable {
        return;
    }
    let t = scene.i18n;
    let menu = |ui: &mut Ui, view_state: &mut ViewState| {
        if addable {
            for kind in [AddKind::Track, AddKind::CategoryTrack, AddKind::MusicFill] {
                let text = t.tr(kind.key());
                let item = ui.button(format!("{}  {text}", icon::PLUS));
                item.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::Role::Button, true, text.clone())
                });
                if item.clicked() {
                    let place = (target, block, usize::MAX);
                    start_add(scene, view_state, kind, target, Some(place), log);
                    ui.close();
                }
            }
        }
        if !editable {
            return;
        }
        if addable {
            ui.separator();
        }
        if ui.button(t.tr("schedule-block-move")).clicked() {
            view_state.log_dialog = Some(LogDialog::MoveBlock {
                target,
                block,
                time: start.format("%H:%M:%S").to_string(),
            });
            ui.close();
        }
        if ui.button(t.tr("schedule-block-remove")).clicked() {
            send_edit(scene, target, LogEdit::RemoveBlock { block });
            ui.close();
        }
        if origin != BlockOrigin::Template && ui.button(t.tr("schedule-date-revert")).clicked() {
            send_edit(scene, target, LogEdit::Revert { block });
            ui.close();
        }
    };
    // Its own id: the right-click menu's is the response's popup id.
    egui::Popup::menu(response)
        .id(response.id.with("block-click-menu"))
        .show(|ui| menu(ui, view_state));
    response.context_menu(|ui| menu(ui, view_state));
}

/// Library tracks dropped on a row go before it, into its block (AR17,
/// AR19b), or after it on its lower half (as on a playlist), so a block's
/// end takes them too; a header takes them at the block's start, and its
/// insertion line is under it.
#[allow(clippy::too_many_arguments)]
fn drop_tracks(
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    response: &egui::Response,
    target: EditTarget,
    block: fp_model::BlockId,
    index: usize,
    item: bool,
    log: bool,
) {
    let r = response.rect;
    let ctx = &response.ctx;
    let pointer = ctx
        .pointer_hover_pos()
        .or_else(|| ctx.pointer_interact_pos());
    let after = item && pointer.is_some_and(|p| p.y > r.center().y);
    let index = index + usize::from(after);
    // Files from the system go where the pointer is, on this row's own
    // layer (not under a window over it), as on a playlist (Q4.6).
    let files_here =
        pointer.is_some_and(|p| r.contains(p) && ctx.layer_id_at(p) == Some(response.layer_id));
    if locked(scene, target, block, index) {
        if files_here {
            view_state.file_drop_refused = true;
        }
        return;
    }
    if files_here {
        view_state.log_file_drop = Some((target, block, index));
    }
    let files_hovering = ctx.input(|i| !i.raw.hovered_files.is_empty());
    // A row only moves within its own date or template: elsewhere no
    // insertion line promises a drop.
    if (files_here && files_hovering)
        || response.dnd_hover_payload::<DragTracks>().is_some()
        || response
            .dnd_hover_payload::<DragLogItem>()
            .is_some_and(|from| from.target == target)
    {
        // A header takes them at its block's start: the line goes under
        // it, never on the previous block's end.
        let y = if after || !item {
            r.bottom() - 1.0
        } else {
            r.top()
        };
        response
            .ctx
            .layer_painter(egui::LayerId::new(egui::Order::Foreground, response.id))
            .hline(r.x_range(), y, egui::Stroke::new(2.0, theme::ACCENT));
    }
    // A row dragged here moves before this row (same date or template).
    // `dnd_release_payload` takes the payload whatever its type: ask for
    // the type being dragged only.
    let row_dragged = egui::DragAndDrop::has_payload_of_type::<DragLogItem>(&response.ctx);
    if row_dragged
        && let Some(from) = response.dnd_release_payload::<DragLogItem>()
        && from.target == target
        && (from.block, from.index) != (block, index)
    {
        send_edit(
            scene,
            target,
            LogEdit::Move {
                from: (from.block, from.index),
                to: (block, index),
            },
        );
    }
    if !row_dragged && let Some(payload) = response.dnd_release_payload::<DragTracks>() {
        let items = payload
            .tracks
            .iter()
            .map(|track| Item::Track {
                track: *track,
                time_critical: false,
            })
            .collect();
        insert_at(scene, view_state, log, (target, block, index), items);
    }
}

/// A row of a Log, template or date table being dragged (AR17).
#[derive(Debug, Clone, Copy, PartialEq)]
struct DragLogItem {
    target: EditTarget,
    block: fp_model::BlockId,
    index: usize,
}

/// The Log's open dialog (AR17).
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum LogDialog {
    Category {
        place: Place,
        log: bool,
        choice: Option<fp_model::CategoryId>,
    },
    MusicFill {
        place: Place,
        log: bool,
        choice: Option<fp_model::RotationId>,
    },
    AddBlock {
        target: EditTarget,
        time: String,
    },
    MoveBlock {
        target: EditTarget,
        block: fp_model::BlockId,
        time: String,
    },
}

impl LogDialog {
    /// The date or template the dialog edits.
    pub(crate) fn target(&self) -> EditTarget {
        match self {
            Self::Category { place, .. } | Self::MusicFill { place, .. } => place.0,
            Self::AddBlock { target, .. } | Self::MoveBlock { target, .. } => *target,
        }
    }
}

/// Whether items may still be added at the end of a block (AR19): not to
/// a block that already played.
pub(crate) fn open_at_end(
    scene: &Scene<'_>,
    target: EditTarget,
    block: fp_model::BlockId,
    len: usize,
) -> bool {
    !locked(scene, target, block, len)
}

/// Where Add puts items: the block of next when it is today or later,
/// else today's first block still to come, else today's last block that
/// has not played.
fn add_target(scene: &Scene<'_>) -> Option<(EditTarget, fp_model::BlockId)> {
    let state = scene.state;
    let today = state.local_now.date();
    if let Some(next) = state.automation.as_ref().and_then(|a| a.next)
        && next.date >= today
    {
        return Some((EditTarget::Date(next.date), next.block));
    }
    let day = state.schedule.day(today);
    let now = fp_model::local_time::TimeOfDay::new(chrono::Timelike::num_seconds_from_midnight(
        &state.local_now.time(),
    ));
    let target = EditTarget::Date(today);
    day.iter()
        .find(|b| {
            now.is_some_and(|n| b.start >= n) && open_at_end(scene, target, b.id, b.items.len())
        })
        .or_else(|| {
            day.iter()
                .rev()
                .find(|b| open_at_end(scene, target, b.id, b.items.len()))
        })
        .map(|b| (target, b.id))
}

/// The Log's footer (AR17): Add, with its menu, and when the next block
/// starts.
pub(crate) fn log_footer(ui: &mut Ui, scene: &Scene<'_>, view_state: &mut ViewState) {
    let today = EditTarget::Date(scene.state.local_now.date());
    let add_to = add_target(scene);
    plan_footer(ui, scene, view_state, today, add_to, true);
}

/// The starts of the target's blocks.
fn starts_of(scene: &Scene<'_>, target: EditTarget) -> Vec<fp_model::local_time::TimeOfDay> {
    match target {
        EditTarget::Date(date) => scene
            .state
            .schedule
            .day(date)
            .into_iter()
            .map(|b| b.start)
            .collect(),
        EditTarget::Template(template) => scene
            .state
            .schedule
            .template(template)
            .map(|t| t.blocks.iter().map(|b| b.start).collect())
            .unwrap_or_default(),
    }
}

/// What an Add button adds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AddKind {
    /// Opens the library, to drag tracks from.
    Track,
    CategoryTrack,
    MusicFill,
    Block,
}

impl AddKind {
    pub(crate) fn key(self) -> &'static str {
        match self {
            Self::Track => "schedule-add-track",
            Self::CategoryTrack => "schedule-add-category-track",
            Self::MusicFill => "schedule-add-music-fill",
            Self::Block => "schedule-add-block",
        }
    }
}

/// Starts adding `kind`: items go to `add_to`, a block to `blocks`;
/// `log`: from the Log's list, else a popup's.
pub(crate) fn start_add(
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    kind: AddKind,
    blocks: EditTarget,
    add_to: Option<Place>,
    log: bool,
) {
    match (kind, add_to) {
        (AddKind::Track, _) => {
            let mut view = scene.state.library_view.clone();
            view.open = true;
            scene.ctl.send(Command::SetLibraryView(Box::new(view)));
        }
        (AddKind::CategoryTrack, Some(place)) => {
            let choice = scene.state.categories.iter().next().map(|c| c.id);
            view_state.log_dialog = Some(LogDialog::Category { place, log, choice });
        }
        (AddKind::MusicFill, Some(place)) => {
            let choice = scene.state.rotations.default_id();
            view_state.log_dialog = Some(LogDialog::MusicFill { place, log, choice });
        }
        (AddKind::Block, _) => {
            let time = fp_model::schedule::free_start(starts_of(scene, blocks))
                .map(|s| s.to_string())
                .unwrap_or_default();
            view_state.log_dialog = Some(LogDialog::AddBlock {
                target: blocks,
                time,
            });
        }
        (AddKind::CategoryTrack | AddKind::MusicFill, None) => {}
    }
}

/// A footer tile's width for `shown`.
fn footer_tile_width(ui: &Ui, shown: &str) -> f32 {
    ui.painter()
        .layout_no_wrap(shown.to_owned(), font(10.0), theme::NEUTRAL_300)
        .size()
        .x
        + 26.0
}

/// A Log, template or date footer (AR17, AR21): one Add tile per kind
/// (a playlist footer's look), whose items go below the selected row
/// (AR19c), else to the end of `add_to`'s block, and whose Block… adds
/// to `blocks`; the tiles that do not fit go behind ⋯. On the Log
/// (`playing`), when the next block starts, if it fits.
pub(crate) fn plan_footer(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    blocks: EditTarget,
    add_to: Option<(EditTarget, fp_model::BlockId)>,
    playing: bool,
) {
    let add_to = selected_place(scene, view_state, playing, blocks)
        .or(add_to.map(|(target, block)| (target, block, usize::MAX)));
    let t = scene.i18n;
    let rect = ui.max_rect();
    ui.painter().rect_filled(rect, 0.0, theme::NEUTRAL_900);
    ui.painter().rect_filled(
        egui::Rect::from_min_size(rect.left_top(), vec2(rect.width(), 1.0)),
        0.0,
        theme::NEUTRAL_800,
    );
    ui.add_space(10.0);
    const GAP: f32 = 6.0;
    ui.spacing_mut().item_spacing = vec2(GAP, 0.0);
    let tip = t.tr(if playing {
        "tip-schedule-add"
    } else {
        "tip-plan-add"
    });
    let kinds: Vec<AddKind> = [
        AddKind::Track,
        AddKind::CategoryTrack,
        AddKind::MusicFill,
        AddKind::Block,
    ]
    .into_iter()
    .filter(|k| add_to.is_some() || !matches!(k, AddKind::CategoryTrack | AddKind::MusicFill))
    .collect();
    let tiles: Vec<(AddKind, String, String, f32)> = kinds
        .iter()
        .map(|&kind| {
            let text = t.tr(kind.key());
            let shown = format!("{} {text}", icon::PLUS);
            let width = footer_tile_width(ui, &shown);
            (kind, text, shown, width)
        })
        .collect();
    // All of them, or as many as fit beside the ⋯ tile.
    let room = ui.available_width() - 10.0;
    let total: f32 = tiles.iter().map(|(.., w)| w + GAP).sum();
    let shown_count = if total <= room {
        tiles.len()
    } else {
        let mut used = 18.0 + GAP;
        tiles
            .iter()
            .take_while(|(.., w)| {
                used += w + GAP;
                used <= room
            })
            .count()
    };
    let mut chosen = None;
    for (kind, text, shown, width) in tiles.iter().take(shown_count) {
        let clicked = widgets::tile_without_tooltip(
            ui,
            vec2(*width, 18.0),
            text,
            true,
            TileStyle::plain(),
            |p, r, c| {
                p.text(r.center(), Align2::CENTER_CENTER, shown, font(10.0), c);
            },
        )
        .on_hover_text(&tip)
        .clicked();
        if clicked {
            chosen = Some(*kind);
        }
    }
    if let Some(rest) = tiles.get(shown_count..).filter(|r| !r.is_empty()) {
        let more = widgets::tile(
            ui,
            vec2(18.0, 18.0),
            &t.tr("tip-schedule-add-more"),
            true,
            TileStyle::plain(),
            |p, r, c| {
                p.text(
                    r.center(),
                    Align2::CENTER_CENTER,
                    icon::DOTS_THREE,
                    font(12.0),
                    c,
                );
            },
        );
        egui::Popup::menu(&more).show(|ui| {
            for (kind, text, shown, _) in rest {
                let item = ui.button(RichText::new(shown).font(font(12.0)));
                item.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::Role::Button, true, text.clone())
                });
                if item.clicked() {
                    chosen = Some(*kind);
                    ui.close();
                }
            }
        });
    }
    if let Some(kind) = chosen {
        start_add(scene, view_state, kind, blocks, add_to, playing);
    }
    if let Some(start) = fp_model::automation::coming_start(scene.state).filter(|_| playing) {
        let text = RichText::new(t.tr_args(
            "schedule-next-block",
            &[("time", start.format("%H:%M").to_string().into())],
        ))
        .font(font(11.0))
        .color(theme::NEUTRAL_400);
        // Shown when it fits after the buttons, never over them.
        let needed = ui
            .painter()
            .layout_no_wrap(text.text().to_owned(), font(11.0), theme::NEUTRAL_400)
            .size()
            .x
            + 16.0;
        if ui.available_width() >= needed {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(10.0);
                ui.add(egui::Label::new(text).selectable(false));
            });
        }
    }
}

/// The open dialog of the Log or a popup, if any: a combo or a time, then
/// OK. Drawn once a frame by the app.
pub(crate) fn log_dialog(ctx: &egui::Context, scene: &Scene<'_>, view_state: &mut ViewState) {
    let t = scene.i18n;
    // A dialog left open past midnight edits a date that may no longer
    // change (AR19).
    if view_state.log_dialog.as_ref().is_some_and(
        |d| matches!(d.target(), EditTarget::Date(d) if d < scene.state.local_now.date()),
    ) {
        view_state.log_dialog = None;
    }
    let Some(dialog) = view_state.log_dialog.as_mut() else {
        return;
    };
    let title = match dialog {
        LogDialog::Category { .. } => t.tr("schedule-add-category-track"),
        LogDialog::MusicFill { .. } => t.tr("schedule-add-music-fill"),
        LogDialog::AddBlock { .. } => t.tr("schedule-add-block"),
        LogDialog::MoveBlock { .. } => t.tr("schedule-block-move"),
    };
    let mut done: Option<bool> = None;
    let width = (ctx.content_rect().width() - 48.0).clamp(280.0, 380.0);
    // The look of the app's other dialogs: a title, labelled fields, then
    // Cancel and OK. Enter confirms; Esc or the backdrop cancels.
    let modal = egui::Modal::new(egui::Id::unique("log-dialog"))
        .frame(egui::Frame::new().fill(theme::SURFACE).inner_margin(20.0))
        .backdrop_color(theme::NEUTRAL_900.gamma_multiply(0.7))
        .show(ctx, |ui| {
            ui.set_width(width);
            ui.spacing_mut().item_spacing = vec2(8.0, 10.0);
            ui.add(
                egui::Label::new(
                    RichText::new(title.trim_end_matches('…'))
                        .font(font_medium(18.0))
                        .color(theme::TEXT),
                )
                .selectable(false)
                .truncate(),
            );
            let mut enter = false;
            match dialog {
                LogDialog::Category { choice, .. } => {
                    let name = |id: Option<fp_model::CategoryId>| {
                        id.and_then(|id| scene.state.categories.get(id))
                            .map(|c| c.name.clone())
                            .unwrap_or_default()
                    };
                    field(ui, &t.tr("schedule-category"), |ui, label| {
                        egui::ComboBox::from_id_salt("log-dialog-category")
                            .width(ui.available_width())
                            .selected_text(name(*choice))
                            .show_ui(ui, |ui| {
                                for c in scene.state.categories.iter() {
                                    ui.selectable_value(choice, Some(c.id), c.name.clone());
                                }
                            })
                            .response
                            .labelled_by(label);
                    });
                }
                LogDialog::MusicFill { choice, .. } => {
                    let name = |id: Option<fp_model::RotationId>| {
                        id.and_then(|id| scene.state.rotations.get(id))
                            .map(|r| r.name.clone())
                            .unwrap_or_default()
                    };
                    field(ui, &t.tr("filler-rotation"), |ui, label| {
                        egui::ComboBox::from_id_salt("log-dialog-rotation")
                            .width(ui.available_width())
                            .selected_text(name(*choice))
                            .show_ui(ui, |ui| {
                                for r in scene.state.rotations.iter() {
                                    ui.selectable_value(choice, Some(r.id), r.name.clone());
                                }
                            })
                            .response
                            .labelled_by(label);
                    });
                }
                LogDialog::AddBlock { time, .. } | LogDialog::MoveBlock { time, .. } => {
                    field(ui, &t.tr("schedule-block-start"), |ui, label| {
                        let edit = ui
                            .add(egui::TextEdit::singleline(time).desired_width(90.0))
                            .labelled_by(label);
                        enter = edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                        if fp_model::local_time::TimeOfDay::parse(time).is_none() {
                            ui.add(
                                egui::Label::new(
                                    RichText::new(t.tr("schedule-bad-time"))
                                        .font(font(11.0))
                                        .color(theme::AMBER),
                                )
                                .selectable(false),
                            );
                        }
                    });
                }
            }
            let ready = match dialog {
                LogDialog::Category { choice, .. } => choice.is_some(),
                LogDialog::MusicFill { choice, .. } => choice.is_some(),
                LogDialog::AddBlock { time, .. } | LogDialog::MoveBlock { time, .. } => {
                    fp_model::local_time::TimeOfDay::parse(time).is_some()
                }
            };
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if button(ui, &t.tr("schedule-dialog-cancel"), None) {
                    done = Some(false);
                }
                ui.add_enabled_ui(ready, |ui| {
                    if button(ui, &t.tr("schedule-dialog-ok"), Some(theme::ACCENT)) {
                        done = Some(true);
                    }
                });
            });
            if enter && ready {
                done = Some(true);
            }
        });
    if done.is_none() && modal.should_close() {
        done = Some(false);
    }
    match done {
        Some(true) => {
            let parse = |s: &str| fp_model::local_time::TimeOfDay::parse(s);
            match dialog.clone() {
                LogDialog::Category {
                    place,
                    log,
                    choice: Some(category),
                } => insert_at(
                    scene,
                    view_state,
                    log,
                    place,
                    vec![Item::Category {
                        category,
                        time_critical: false,
                    }],
                ),
                LogDialog::MusicFill {
                    place,
                    log,
                    choice: Some(rotation),
                } => insert_at(
                    scene,
                    view_state,
                    log,
                    place,
                    vec![Item::Rotation { rotation }],
                ),
                LogDialog::AddBlock { target, time } => {
                    if let Some(start) = parse(&time) {
                        send_edit(scene, target, LogEdit::AddBlock { start });
                    }
                }
                LogDialog::MoveBlock {
                    target,
                    block,
                    time,
                } => {
                    if let Some(start) = parse(&time) {
                        send_edit(scene, target, LogEdit::MoveBlock { block, start });
                    }
                }
                _ => {}
            }
            view_state.log_dialog = None;
        }
        Some(false) => view_state.log_dialog = None,
        None => {}
    }
}

/// One labelled row of a dialog: the label, then `content`, which is
/// given the label's id to name its control.
fn field(ui: &mut Ui, label: &str, content: impl FnOnce(&mut Ui, egui::Id)) {
    ui.horizontal(|ui| {
        let label = ui.add(
            egui::Label::new(
                RichText::new(label)
                    .font(font(12.0))
                    .color(theme::NEUTRAL_400),
            )
            .selectable(false),
        );
        content(ui, label.id);
    });
}

/// What an item plays, in words.
pub(crate) fn item_text(scene: &Scene<'_>, what: &Item) -> String {
    let t = scene.i18n;
    let state = scene.state;
    let missing = || t.tr("schedule-item-missing");
    match what {
        Item::Track { track, .. } => state.library.get(*track).map_or_else(missing, |tr| {
            if tr.artist.trim().is_empty() {
                tr.title.clone()
            } else {
                format!("{} — {}", tr.title, tr.artist)
            }
        }),
        Item::Category { category, .. } => {
            state.categories.get(*category).map_or_else(missing, |c| {
                t.tr_args("schedule-item-category", &[("name", c.name.clone().into())])
            })
        }
        Item::Rotation { rotation } => state.rotations.get(*rotation).map_or_else(missing, |r| {
            t.tr_args("schedule-item-rotation", &[("name", r.name.clone().into())])
        }),
    }
}

/// One item row: Time, Title (hard start clock, the music fill icon, a
/// chosen category track with its category after it), Artist (or "until"
/// for the music fill), Duration.
fn item_row(row: &mut egui_extras::TableRow<'_, '_>, scene: &Scene<'_>, r: &ScheduleRow) {
    let t = scene.i18n;
    let state = scene.state;
    let ScheduleRow::Item {
        what,
        length_secs,
        time_critical,
        status,
        starts_at,
        track,
        late,
        nothing_to_play,
        ..
    } = r
    else {
        return;
    };
    let hi = matches!(status, RowStatus::OnAir | RowStatus::Next);
    let (fill, colour, second) = match status {
        RowStatus::OnAir => (theme::ON_AIR_ROW, theme::NEUTRAL_100, theme::NEUTRAL_100),
        RowStatus::Next => (theme::NEXT_ROW, theme::NEUTRAL_100, theme::NEUTRAL_100),
        RowStatus::Aired | RowStatus::Missed => {
            (Color32::TRANSPARENT, theme::NEUTRAL_600, theme::NEUTRAL_600)
        }
        RowStatus::Coming => (Color32::TRANSPARENT, theme::TEXT, theme::NEUTRAL_400),
    };
    let row_font = if hi {
        super::widgets::font_medium(12.0)
    } else {
        font(12.0)
    };
    let missed = *status == RowStatus::Missed;
    let chosen = track.and_then(|id| state.library.get(id));
    let (title, artist) = match (what, chosen) {
        (Item::Rotation { .. }, _) => (
            item_text(scene, what),
            length_secs
                .map(|secs| {
                    let until =
                        *starts_at + chrono::TimeDelta::milliseconds((secs * 1000.0) as i64);
                    t.tr_args(
                        "schedule-until",
                        &[("time", until.format("%H:%M").to_string().into())],
                    )
                })
                .unwrap_or_default(),
        ),
        (Item::Category { .. }, Some(tr)) => (
            format!("{} · {}", tr.title, item_text(scene, what)),
            tr.artist.clone(),
        ),
        (_, Some(tr)) => (tr.title.clone(), tr.artist.clone()),
        // AR15: it will be missed; the Log says why beforehand.
        (Item::Category { .. }, None) if *nothing_to_play => (
            item_text(scene, what),
            t.tr("schedule-category-nothing-to-play"),
        ),
        (_, None) => (item_text(scene, what), String::new()),
    };
    let second = if *nothing_to_play && !hi {
        theme::AMBER
    } else {
        second
    };
    row.col(|ui| {
        row_fill(ui, fill);
        let colour = if *late { theme::AMBER } else { colour };
        let response = cell(
            ui,
            RichText::new(starts_at.format("%H:%M:%S").to_string())
                .font(row_font.clone())
                .color(colour),
            false,
        );
        if *late {
            let label = format!(
                "{} — {}",
                starts_at.format("%H:%M:%S"),
                t.tr("schedule-late")
            );
            response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Label, true, &label));
            response.on_hover_text(t.tr("schedule-late"));
        }
    });
    row.col(|ui| {
        row_fill(ui, fill);
        ui.add_space(8.0);
        ui.spacing_mut().item_spacing = vec2(4.0, 0.0);
        if *time_critical {
            ui.label(RichText::new(icon::CLOCK).font(font(12.0)).color(if hi {
                theme::NEUTRAL_100
            } else {
                theme::AMBER
            }))
            .on_hover_text(t.tr("schedule-time-critical"));
        }
        if matches!(what, Item::Rotation { .. }) {
            ui.label(RichText::new(icon::SHUFFLE).font(font(12.0)).color(if hi {
                theme::NEUTRAL_100
            } else {
                theme::ACCENT
            }));
        }
        let mut rich = RichText::new(title.clone())
            .font(row_font.clone())
            .color(colour);
        if missed {
            rich = rich.strikethrough();
        }
        let response = ui.add(egui::Label::new(rich).selectable(false).truncate());
        if missed {
            let label = format!("{title} — {}", t.tr("schedule-missed"));
            response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Label, true, &label));
            response.on_hover_text(t.tr("schedule-missed"));
        }
    });
    row.col(|ui| {
        row_fill(ui, fill);
        cell(
            ui,
            RichText::new(artist).font(row_font.clone()).color(second),
            false,
        );
    });
    row.col(|ui| {
        row_fill(ui, fill);
        if let Some(secs) = length_secs {
            cell(
                ui,
                RichText::new(format::clock(*secs))
                    .font(row_font)
                    .color(colour),
                true,
            );
        }
    });
}
