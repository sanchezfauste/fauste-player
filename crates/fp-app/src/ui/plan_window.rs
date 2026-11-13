//! A template or a date in a popup (automation rework AR21–AR22): the same
//! list as the Log column, which never plays, with its Add footer. It
//! floats over the main window, like the library, so tracks are dragged
//! from the library into it.

use egui::{Align, Layout, Rect, UiBuilder, pos2, vec2};
use fp_model::automation::{ScheduleRow, plan_view};
use fp_model::log_edit::EditTarget;

use super::app::{Scene, ViewState};

/// The footer's height, as a player's.
const FOOTER_HEIGHT: f32 = 24.0;

/// The popup's title: "Template · name" or "Day · date"; `None` when its
/// template no longer exists.
fn title(scene: &Scene<'_>, target: EditTarget) -> Option<String> {
    let t = scene.i18n;
    match target {
        EditTarget::Template(id) => scene.state.schedule.template(id).map(|template| {
            t.tr_args(
                "plan-window-template",
                &[("name", template.name.clone().into())],
            )
        }),
        EditTarget::Date(date) => Some(t.tr_args(
            "plan-window-date",
            &[("date", date.format("%Y-%m-%d").to_string().into())],
        )),
    }
}

/// Where the popup's Add puts items: its last block.
fn add_to(rows: &[ScheduleRow], target: EditTarget) -> Option<(EditTarget, fp_model::BlockId)> {
    rows.iter()
        .rev()
        .find_map(|r| match r {
            ScheduleRow::Block { block, .. } => Some(*block),
            ScheduleRow::Item { .. } => None,
        })
        .map(|block| (target, block))
}

/// Draws the open popup, if any; closes it when its template is gone.
pub(crate) fn draw(ctx: &egui::Context, scene: &Scene<'_>, view_state: &mut ViewState) {
    let Some(target) = view_state.plan_window else {
        return;
    };
    let Some(title) = title(scene, target) else {
        view_state.plan_window = None;
        return;
    };
    let rows = plan_view(scene.state, target);
    let template = match target {
        EditTarget::Template(id) => Some(id),
        EditTarget::Date(_) => None,
    };
    let mut open = true;
    egui::Window::new(title)
        .id(egui::Id::unique("plan-window"))
        .open(&mut open)
        .default_size(vec2(620.0, 520.0))
        // First shown in the middle of the screen, clear of the top bar.
        .pivot(egui::Align2::CENTER_CENTER)
        .default_pos(ctx.content_rect().center())
        .min_size(vec2(360.0, 200.0))
        .resizable(true)
        .collapsible(false)
        .show(ctx, |ui| {
            let area = ui.available_rect_before_wrap();
            let area = Rect::from_min_size(
                area.min,
                vec2(area.width(), area.height().max(ui.min_rect().height())),
            );
            let footer_top = area.bottom() - FOOTER_HEIGHT;
            let mut table_ui = ui.new_child(
                UiBuilder::new()
                    .max_rect(Rect::from_min_max(area.min, pos2(area.right(), footer_top)))
                    .layout(Layout::top_down(Align::Min)),
            );
            super::schedule::log_table(&mut table_ui, scene, view_state, &rows, template, false);
            let mut footer_ui = ui.new_child(
                UiBuilder::new()
                    .max_rect(Rect::from_min_max(pos2(area.left(), footer_top), area.max))
                    .layout(Layout::left_to_right(Align::Center)),
            );
            super::schedule::plan_footer(
                &mut footer_ui,
                scene,
                view_state,
                target,
                add_to(&rows, target),
                false,
            );
            ui.allocate_rect(area, egui::Sense::hover());
        });
    if !open {
        view_state.plan_window = None;
    }
}
