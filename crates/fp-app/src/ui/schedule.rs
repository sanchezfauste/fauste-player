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

/// The schedule grouped by block (§8.1): each block's start, then its
/// items; on air and next as on a player, missed struck through (AU23).
pub(crate) fn schedule_list(ui: &mut Ui, scene: &Scene<'_>) {
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
    egui::ScrollArea::vertical()
        .id_salt("schedule-list")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 1.0);
            if !has_today {
                ui.add_space(6.0);
                ui.label(
                    RichText::new(t.tr("schedule-empty"))
                        .font(font(12.0))
                        .color(theme::NEUTRAL_500),
                );
            }
            for row in &rows {
                match row {
                    ScheduleRow::Block {
                        date,
                        start,
                        origin,
                        status,
                        ..
                    } => {
                        let mut text = start.format("%H:%M:%S").to_string();
                        if *date < today {
                            text = format!("{text} · {}", t.tr("schedule-yesterday"));
                        } else if *date > today {
                            text = format!("{text} · {}", t.tr("schedule-tomorrow"));
                        }
                        if *origin != BlockOrigin::Template {
                            text = format!("{text} · {}", t.tr("schedule-changed-for-date"));
                        }
                        let colour = match status {
                            RowStatus::Missed | RowStatus::Aired => theme::NEUTRAL_500,
                            _ => theme::TEXT,
                        };
                        ui.add_space(4.0);
                        egui::Frame::new()
                            .fill(theme::NEUTRAL_900)
                            .inner_margin(egui::Margin::symmetric(8, 3))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.label(
                                    RichText::new(text).font(font_semibold(12.0)).color(colour),
                                );
                            });
                    }
                    ScheduleRow::Item {
                        item,
                        what,
                        length_secs,
                        time_critical,
                        status,
                        ..
                    } => {
                        let response =
                            item_row(ui, scene, what, *length_secs, *time_critical, *status);
                        if scroll_now && Some(*item) == target {
                            response.scroll_to_me(Some(Align::Center));
                        }
                        next_actions(scene, &response, *item);
                    }
                }
            }
        });
    if scroll_now {
        ui.data_mut(|d| d.insert_temp(memory, target));
    }
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

/// One item of a block; the response senses clicks (setting next).
fn item_row(
    ui: &mut Ui,
    scene: &Scene<'_>,
    what: &Item,
    length_secs: Option<f64>,
    time_critical: bool,
    status: RowStatus,
) -> egui::Response {
    let t = scene.i18n;
    let (fill, colour) = match status {
        RowStatus::OnAir => (theme::ON_AIR_ROW, theme::TEXT),
        RowStatus::Next => (theme::NEXT_ROW, theme::TEXT),
        RowStatus::Aired | RowStatus::Missed => (Color32::TRANSPARENT, theme::NEUTRAL_500),
        RowStatus::Coming => (Color32::TRANSPARENT, theme::TEXT),
    };
    let text = item_text(scene, what);
    let missed = status == RowStatus::Missed;
    egui::Frame::new()
        .fill(fill)
        .inner_margin(egui::Margin::symmetric(8, 2))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = vec2(6.0, 0.0);
                if time_critical {
                    ui.label(
                        RichText::new(icon::CLOCK)
                            .font(font(12.0))
                            .color(theme::AMBER),
                    )
                    .on_hover_text(t.tr("schedule-time-critical"));
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if let Some(secs) = length_secs {
                        ui.label(
                            RichText::new(format::clock(secs))
                                .font(font(12.0))
                                .color(colour),
                        );
                    }
                    let mut rich = RichText::new(text.clone()).font(font(12.0)).color(colour);
                    if missed {
                        rich = rich.strikethrough();
                    }
                    let response = ui
                        .with_layout(Layout::left_to_right(Align::Center), |ui| {
                            ui.add(egui::Label::new(rich).sense(Sense::click()).truncate())
                        })
                        .inner;
                    if missed {
                        let label = format!("{text} — {}", t.tr("schedule-missed"));
                        response.widget_info(|| {
                            egui::WidgetInfo::labeled(egui::Role::Label, true, &label)
                        });
                        response.on_hover_text(t.tr("schedule-missed"))
                    } else {
                        response
                    }
                })
                .inner
            })
            .inner
        })
        .inner
}
