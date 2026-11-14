//! The Log column's list (automation spec §8.1) and the strip above it;
//! its transport is the player header's (automation rework AR3).

use egui::{Align, Color32, Layout, RichText, Sense, Ui, UiBuilder, vec2};
use egui_phosphor::regular as icon;
use fp_model::Command;
use fp_model::automation::{ItemRef, RowStatus, ScheduleRow, schedule_view};
use fp_model::schedule::{BlockOrigin, Item};

use super::app::{DragTracks, Scene, ViewState};
use super::format;
use super::theme;
use super::widgets::{font, font_semibold};
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
    let artist_width = (width * 0.28).max(60.0);
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
        .column(Column::exact(TIME_WIDTH))
        .column(Column::remainder().clip(true))
        .column(Column::exact(artist_width).clip(true))
        .column(Column::exact(DURATION_WIDTH));
    if scroll_now
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
                        ..
                    } => {
                        let edits = target_of(template, *date);
                        block_row(&mut row, scene, r, playing.then_some(today));
                        let response = row.response();
                        block_actions(scene, view_state, &response, edits, *block, *start, *origin);
                        drop_tracks(scene, &response, edits, *block, 0, false);
                    }
                    ScheduleRow::Item { item, .. } => {
                        let edits = target_of(template, item.date);
                        item_row(&mut row, scene, r);
                        let response = row.response();
                        row_actions(scene, &response, edits, *item, playing);
                        if response.drag_started() && !locked(scene, edits, item.block, item.index)
                        {
                            response.dnd_set_drag_payload(DragLogItem {
                                target: edits,
                                block: item.block,
                                index: item.index,
                            });
                        }
                        drop_tracks(scene, &response, edits, item.block, item.index, true);
                    }
                }
            });
        });
    if scroll_now {
        ui.data_mut(|d| d.insert_temp(memory, target));
    }
}

/// The Time column's width (`HH:MM:SS` with room) and the duration's.
const TIME_WIDTH: f32 = 76.0;
const DURATION_WIDTH: f32 = 60.0;

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

/// A block header's menu: Change time…, Remove block, Back to the
/// template (AR17), unless the block is locked.
fn block_actions(
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    response: &egui::Response,
    target: EditTarget,
    block: fp_model::BlockId,
    start: chrono::NaiveDateTime,
    origin: BlockOrigin,
) {
    let t = scene.i18n;
    if locked(scene, target, block, 0) {
        return;
    }
    response.context_menu(|ui| {
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
    });
}

/// Library tracks dropped on a row go before it, into its block (AR17,
/// AR19b), or after it on its lower half (as on a playlist), so a block's
/// end takes them too; a header takes them at the block's start.
fn drop_tracks(
    scene: &Scene<'_>,
    response: &egui::Response,
    target: EditTarget,
    block: fp_model::BlockId,
    index: usize,
    item: bool,
) {
    let r = response.rect;
    let after = item
        && response
            .ctx
            .pointer_interact_pos()
            .is_some_and(|p| p.y > r.center().y);
    let index = index + usize::from(after);
    if locked(scene, target, block, index) {
        return;
    }
    // A row only moves within its own date or template: elsewhere no
    // insertion line promises a drop.
    if response.dnd_hover_payload::<DragTracks>().is_some()
        || response
            .dnd_hover_payload::<DragLogItem>()
            .is_some_and(|from| from.target == target)
    {
        let y = if after { r.bottom() - 1.0 } else { r.top() };
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
        send_edit(
            scene,
            target,
            LogEdit::Insert {
                block,
                index,
                items,
            },
        );
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
        target: EditTarget,
        block: fp_model::BlockId,
        choice: Option<fp_model::CategoryId>,
    },
    MusicFill {
        target: EditTarget,
        block: fp_model::BlockId,
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
            Self::Category { target, .. }
            | Self::MusicFill { target, .. }
            | Self::AddBlock { target, .. }
            | Self::MoveBlock { target, .. } => *target,
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
        .find(|b| now.is_some_and(|n| b.start >= n))
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

/// A Log, template or date footer (AR17, AR21): Add, with its menu, whose
/// items go to `add_to`'s block and whose Block… adds to `blocks`; on the
/// Log (`playing`), when the next block starts.
pub(crate) fn plan_footer(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    blocks: EditTarget,
    add_to: Option<(EditTarget, fp_model::BlockId)>,
    playing: bool,
) {
    let t = scene.i18n;
    let rect = ui.max_rect();
    ui.painter().rect_filled(rect, 0.0, theme::NEUTRAL_900);
    ui.painter().rect_filled(
        egui::Rect::from_min_size(rect.left_top(), vec2(rect.width(), 1.0)),
        0.0,
        theme::NEUTRAL_800,
    );
    ui.add_space(10.0);
    ui.spacing_mut().item_spacing = vec2(6.0, 0.0);
    let tip = if playing {
        "tip-schedule-add"
    } else {
        "tip-plan-add"
    };
    let add = ui
        .add(egui::Button::new(
            RichText::new(format!("{} {}", icon::PLUS, t.tr("footer-add"))).font(font(10.0)),
        ))
        .on_hover_text(t.tr(tip));
    add.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, t.tr(tip)));
    egui::Popup::menu(&add).show(|ui| {
        if ui.button(t.tr("schedule-add-track")).clicked() {
            let mut view = scene.state.library_view.clone();
            view.open = true;
            scene.ctl.send(Command::SetLibraryView(Box::new(view)));
            ui.close();
        }
        if let Some((target, block)) = add_to {
            if ui.button(t.tr("schedule-add-category-track")).clicked() {
                let choice = scene.state.categories.iter().next().map(|c| c.id);
                view_state.log_dialog = Some(LogDialog::Category {
                    target,
                    block,
                    choice,
                });
                ui.close();
            }
            if ui.button(t.tr("schedule-add-music-fill")).clicked() {
                let choice = scene.state.rotations.default_id();
                view_state.log_dialog = Some(LogDialog::MusicFill {
                    target,
                    block,
                    choice,
                });
                ui.close();
            }
        }
        ui.separator();
        if ui.button(t.tr("schedule-add-block")).clicked() {
            let time = fp_model::schedule::free_start(starts_of(scene, blocks))
                .map(|s| s.to_string())
                .unwrap_or_default();
            view_state.log_dialog = Some(LogDialog::AddBlock {
                target: blocks,
                time,
            });
            ui.close();
        }
    });
    if let Some(start) = fp_model::automation::coming_start(scene.state).filter(|_| playing) {
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add_space(10.0);
            ui.label(
                RichText::new(t.tr_args(
                    "schedule-next-block",
                    &[("time", start.format("%H:%M").to_string().into())],
                ))
                .font(font(11.0))
                .color(theme::NEUTRAL_400),
            );
        });
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
    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            match dialog {
                LogDialog::Category { choice, .. } => {
                    let name = |id: Option<fp_model::CategoryId>| {
                        id.and_then(|id| scene.state.categories.get(id))
                            .map(|c| c.name.clone())
                            .unwrap_or_default()
                    };
                    egui::ComboBox::from_label(t.tr("schedule-category"))
                        .selected_text(name(*choice))
                        .show_ui(ui, |ui| {
                            for c in scene.state.categories.iter() {
                                ui.selectable_value(choice, Some(c.id), c.name.clone());
                            }
                        });
                }
                LogDialog::MusicFill { choice, .. } => {
                    let name = |id: Option<fp_model::RotationId>| {
                        id.and_then(|id| scene.state.rotations.get(id))
                            .map(|r| r.name.clone())
                            .unwrap_or_default()
                    };
                    egui::ComboBox::from_label(t.tr("filler-rotation"))
                        .selected_text(name(*choice))
                        .show_ui(ui, |ui| {
                            for r in scene.state.rotations.iter() {
                                ui.selectable_value(choice, Some(r.id), r.name.clone());
                            }
                        });
                }
                LogDialog::AddBlock { time, .. } | LogDialog::MoveBlock { time, .. } => {
                    let label = ui.label(t.tr("schedule-block-start"));
                    ui.add(egui::TextEdit::singleline(time).desired_width(90.0))
                        .labelled_by(label.id);
                    if fp_model::local_time::TimeOfDay::parse(time).is_none() {
                        ui.label(
                            RichText::new(t.tr("schedule-bad-time"))
                                .font(font(11.0))
                                .color(theme::AMBER),
                        );
                    }
                }
            }
            ui.horizontal(|ui| {
                let ready = match dialog {
                    LogDialog::Category { choice, .. } => choice.is_some(),
                    LogDialog::MusicFill { choice, .. } => choice.is_some(),
                    LogDialog::AddBlock { time, .. } | LogDialog::MoveBlock { time, .. } => {
                        fp_model::local_time::TimeOfDay::parse(time).is_some()
                    }
                };
                if ui
                    .add_enabled(ready, egui::Button::new(t.tr("schedule-dialog-ok")))
                    .clicked()
                {
                    done = Some(true);
                }
                if ui.button(t.tr("schedule-dialog-cancel")).clicked() {
                    done = Some(false);
                }
            });
        });
    match done {
        Some(true) => {
            let parse = |s: &str| fp_model::local_time::TimeOfDay::parse(s);
            match dialog.clone() {
                LogDialog::Category {
                    target,
                    block,
                    choice: Some(category),
                } => send_edit(
                    scene,
                    target,
                    LogEdit::Insert {
                        block,
                        index: usize::MAX,
                        items: vec![Item::Category {
                            category,
                            time_critical: false,
                        }],
                    },
                ),
                LogDialog::MusicFill {
                    target,
                    block,
                    choice: Some(rotation),
                } => send_edit(
                    scene,
                    target,
                    LogEdit::Insert {
                        block,
                        index: usize::MAX,
                        items: vec![Item::Rotation { rotation }],
                    },
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
        (_, None) => (item_text(scene, what), String::new()),
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
