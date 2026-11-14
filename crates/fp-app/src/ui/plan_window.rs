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

/// Where the popup's Add puts items: its last block that has not played.
fn add_to(
    scene: &Scene<'_>,
    rows: &[ScheduleRow],
    target: EditTarget,
) -> Option<(EditTarget, fp_model::BlockId)> {
    rows.iter()
        .rev()
        .find_map(|r| match r {
            ScheduleRow::Block { block, count, .. }
                if super::schedule::open_at_end(scene, target, *block, *count) =>
            {
                Some(*block)
            }
            _ => None,
        })
        .map(|block| (target, block))
}

/// Closes the popup, and the dialog it opened.
fn close(view_state: &mut ViewState, target: EditTarget) {
    view_state.plan_window = None;
    if view_state
        .log_dialog
        .as_ref()
        .is_some_and(|d| d.target() == target)
    {
        view_state.log_dialog = None;
    }
}

/// Draws the open popup, if any; closes it when its template is gone or
/// its date has passed (past dates are not edited, AR19).
pub(crate) fn draw(ctx: &egui::Context, scene: &Scene<'_>, view_state: &mut ViewState) {
    let Some(target) = view_state.plan_window else {
        return;
    };
    let past = matches!(target, EditTarget::Date(d) if d < scene.state.local_now.date());
    let Some(title) = title(scene, target).filter(|_| !past) else {
        close(view_state, target);
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
                add_to(scene, &rows, target),
                false,
            );
            ui.allocate_rect(area, egui::Sense::hover());
        });
    if !open {
        close(view_state, target);
    }
}
