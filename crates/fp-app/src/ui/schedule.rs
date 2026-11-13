//! The Log column's list (automation spec §8.1) and the strip above it;
//! its transport is the player header's (automation rework AR3).

use egui::{Align, Color32, Layout, RichText, Sense, Ui, UiBuilder, vec2};
use egui_phosphor::regular as icon;
use fp_model::Command;
use fp_model::automation::{ItemRef, RowStatus, ScheduleRow, schedule_view};
use fp_model::schedule::{BlockOrigin, Item};

use super::app::{Scene, ViewState};
use super::format;
use super::theme;
use super::widgets::{font, font_semibold};

/// The Log's strip where a player has its playlist tabs (`height` tall):
/// Edit log. Its transport is the player header's (rework AR3).
pub(crate) fn schedule_bar(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    height: f32,
) {
    let t = scene.i18n;
    if scene.state.automation.is_none() {
        return;
    }
    let (strip, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
    ui.painter().rect_filled(strip, 0.0, theme::NEUTRAL_900);
    let mut bar = ui.new_child(
        UiBuilder::new()
            .max_rect(strip.shrink2(vec2(8.0, 0.0)))
            .layout(Layout::left_to_right(Align::Center)),
    );
    bar.spacing_mut().item_spacing = vec2(8.0, 0.0);
    if !view_state.phone_active && bar.button(t.tr("schedule-edit")).clicked() {
        view_state.schedule_editor.open = true;
    }
}

/// The Log's list (rework AR14): the playlists' table look, a Time
/// column, block header rows, then each block's items; on air and next as
/// on a player, missed struck through (AU23), a late next in amber (AR11).
pub(crate) fn schedule_list(ui: &mut Ui, scene: &Scene<'_>) {
    use egui_extras::{Column, TableBuilder};
    let t = scene.i18n;
    let state = scene.state;
    let rows = schedule_view(state);
    let today = state.local_now.date();
    let has_today = rows
        .iter()
        .any(|r| matches!(r, ScheduleRow::Block { date, .. } if *date == today));
    // Scroll to the row on air, else next, once each time it changes.
    let target = rows.iter().find_map(|r| match r {
        ScheduleRow::Item {
            item,
            status: RowStatus::OnAir,
            ..
        } => Some(*item),
        _ => None,
    });
    let target = target.or_else(|| state.automation.as_ref().and_then(|a| a.next));
    let memory = ui.scope_id().with("schedule-scrolled");
    let scrolled: Option<ItemRef> = ui.data(|d| d.get_temp(memory)).flatten();
    let scroll_now = target.is_some() && target != scrolled;
    if !has_today {
        ui.add_space(6.0);
        ui.label(
            RichText::new(t.tr("schedule-empty"))
                .font(font(12.0))
                .color(theme::NEUTRAL_500),
        );
    }
    ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
    let width = (ui.available_width() - ui.spacing().scroll.allocated_width()).max(0.0);
    let artist_width = (width * 0.28).max(60.0);
    let mut builder = TableBuilder::new(ui)
        .id_salt("schedule-list")
        .striped(false)
        .resizable(false)
        .vscroll(true)
        .min_scrolled_height(0.0)
        .auto_shrink([false, false])
        .sense(Sense::click())
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
                    ScheduleRow::Block { .. } => block_row(&mut row, scene, r, today),
                    ScheduleRow::Item { item, .. } => {
                        item_row(&mut row, scene, r);
                        next_actions(scene, &row.response(), *item);
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
    today: chrono::NaiveDate,
) {
    let t = scene.i18n;
    let ScheduleRow::Block {
        date,
        start,
        origin,
        status,
        total_secs,
        overrun_secs,
        ..
    } = r
    else {
        return;
    };
    let colour = match status {
        RowStatus::Missed | RowStatus::Aired => theme::NEUTRAL_500,
        _ => theme::TEXT,
    };
    let count = scene
        .state
        .schedule
        .day(*date)
        .into_iter()
        .find(|b| matches!(r, ScheduleRow::Block { block, .. } if *block == b.id))
        .map_or(0, |b| b.items.len());
    let mut text = t.tr_args(
        "schedule-block-header",
        &[
            ("count", (count as f64).into()),
            ("length", format::clock(*total_secs).into()),
        ],
    );
    if *date < today {
        text = format!("{text} · {}", t.tr("schedule-yesterday"));
    } else if *date > today {
        text = format!("{text} · {}", t.tr("schedule-tomorrow"));
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
                "schedule-editor-overrun",
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

/// Setting next from the list: a double-click or the menu's "Play next";
/// on a next set by hand, the menu's "Back to the clock" (AU16).
fn next_actions(scene: &Scene<'_>, response: &egui::Response, item: ItemRef) {
    let t = scene.i18n;
    if response.double_clicked() {
        scene.ctl.send(Command::SetScheduleNext(item));
    }
    let by_hand = scene
        .state
        .automation
        .as_ref()
        .is_some_and(|a| a.next_by_hand && a.next == Some(item));
    response.context_menu(|ui| {
        if ui.button(t.tr("schedule-set-next")).clicked() {
            scene.ctl.send(Command::SetScheduleNext(item));
            ui.close();
        }
        if by_hand && ui.button(t.tr("schedule-clear-next")).clicked() {
            scene.ctl.send(Command::ClearScheduleNext);
            ui.close();
        }
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
