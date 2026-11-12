//! The Schedule column (automation spec §8.1): automation's Start, Stop
//! after current and state above the schedule's list.

use egui::{Align, Layout, RichText, Sense, Ui, UiBuilder, vec2};
use fp_model::Command;

use super::app::Scene;
use super::theme;
use super::widgets::font;

/// Start, Stop after current and the running/stopped state, in the strip
/// where a player has its playlist tabs (`height` tall).
pub(crate) fn schedule_bar(ui: &mut Ui, scene: &Scene<'_>, height: f32) {
    let t = scene.i18n;
    let Some(a) = scene.state.automation.as_ref() else {
        return;
    };
    let (strip, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
    ui.painter().rect_filled(strip, 0.0, theme::NEUTRAL_900);
    let mut bar = ui.new_child(
        UiBuilder::new()
            .max_rect(strip.shrink2(vec2(8.0, 0.0)))
            .layout(Layout::left_to_right(Align::Center)),
    );
    bar.spacing_mut().item_spacing = vec2(8.0, 0.0);
    let start = bar
        .add_enabled(!a.running, egui::Button::new(t.tr("schedule-start")))
        .on_hover_text(t.tr("tip-schedule-start"));
    if start.clicked() {
        scene.ctl.send(Command::StartAutomation);
    }
    let stop = bar
        .add_enabled(
            a.running,
            egui::Button::new(t.tr("schedule-stop-after-current")),
        )
        .on_hover_text(t.tr("tip-schedule-stop-after-current"));
    if stop.clicked() {
        scene.ctl.send(Command::StopAutomationAfterCurrent);
    }
    let (text, colour) = if a.running {
        (t.tr("schedule-running"), theme::PLAY)
    } else {
        (t.tr("schedule-stopped"), theme::NEUTRAL_400)
    };
    bar.add(egui::Label::new(RichText::new(text).font(font(11.0)).color(colour)).truncate());
}
