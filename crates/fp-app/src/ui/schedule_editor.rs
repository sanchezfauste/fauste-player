//! The schedule editor (automation spec §8.2): a floating window, opened
//! from the Schedule column, that edits the day templates, the week plan
//! and single dates. Every edit is a model command.

use chrono::Weekday;
use egui::{RichText, vec2};
use fp_model::{Command, TemplateId};

use super::app::{Scene, ViewState};
use super::theme;
use super::widgets::font;

/// Which part of the editor shows.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum Tab {
    #[default]
    Templates,
    Week,
    Days,
}

/// What the editor remembers between frames (never saved).
#[derive(Debug, Default)]
pub(crate) struct EditorState {
    pub open: bool,
    pub tab: Tab,
}

pub(crate) const WEEKDAYS: [(Weekday, &str); 7] = [
    (Weekday::Mon, "weekday-monday"),
    (Weekday::Tue, "weekday-tuesday"),
    (Weekday::Wed, "weekday-wednesday"),
    (Weekday::Thu, "weekday-thursday"),
    (Weekday::Fri, "weekday-friday"),
    (Weekday::Sat, "weekday-saturday"),
    (Weekday::Sun, "weekday-sunday"),
];

/// Draws the editor while it is open (never on a phone).
pub(crate) fn draw(ctx: &egui::Context, scene: &Scene<'_>, view_state: &mut ViewState) {
    if !view_state.schedule_editor.open || view_state.phone_active {
        return;
    }
    let t = scene.i18n;
    let mut open = true;
    egui::Window::new(t.tr("schedule-editor-title"))
        .id(egui::Id::unique("schedule-editor"))
        .open(&mut open)
        .default_size(vec2(720.0, 560.0))
        .resizable(true)
        .collapsible(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let editor = &mut view_state.schedule_editor;
                for (tab, key) in [
                    (Tab::Templates, "schedule-editor-templates"),
                    (Tab::Week, "schedule-editor-week"),
                    (Tab::Days, "schedule-editor-days"),
                ] {
                    ui.selectable_value(&mut editor.tab, tab, t.tr(key));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(t.tr("schedule-editor-rotations")).clicked() {
                        view_state.open_automation_settings = true;
                    }
                });
            });
            ui.separator();
            match view_state.schedule_editor.tab {
                Tab::Templates => {}
                Tab::Week => week(ui, scene),
                Tab::Days => {}
            }
        });
    if !open {
        view_state.schedule_editor.open = false;
    }
}

/// The template's name, or "?" when it is gone.
pub(crate) fn template_name(scene: &Scene<'_>, id: TemplateId) -> String {
    scene
        .state
        .schedule
        .template(id)
        .map_or_else(|| "?".to_owned(), |t| t.name.clone())
}

/// The week plan (AU9): each weekday's template.
fn week(ui: &mut egui::Ui, scene: &Scene<'_>) {
    let t = scene.i18n;
    let schedule = &scene.state.schedule;
    ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
    for (weekday, key) in WEEKDAYS {
        let current = schedule.week().get(weekday);
        let mut choice = current;
        egui::ComboBox::from_label(RichText::new(t.tr(key)).font(font(13.0)).color(theme::TEXT))
            .width(200.0)
            .selected_text(current.map(|c| template_name(scene, c)).unwrap_or_default())
            .show_ui(ui, |ui| {
                for template in schedule.templates() {
                    ui.selectable_value(&mut choice, Some(template.id), template.name.as_str());
                }
            });
        if let Some(template) = choice
            && choice != current
        {
            scene
                .ctl
                .send(Command::SetWeekTemplate { weekday, template });
        }
    }
}
