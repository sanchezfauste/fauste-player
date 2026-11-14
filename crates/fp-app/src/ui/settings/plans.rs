//! Settings → Automation, Templates and days (automation rework AR20): the
//! day templates, the week plan and single dates. A template or a date is
//! edited in its popup (`ui::plan_window`), which Edit… opens.

use chrono::{Datelike, Days, Months, NaiveDate, Weekday};
use egui::{RichText, Ui, vec2};
use egui_phosphor::regular as icon;
use fp_model::log_edit::EditTarget;
use fp_model::{Command, TemplateId};

use super::{caption, note};
use crate::ui::app::{Scene, ViewState};
use crate::ui::theme;
use crate::ui::widgets::{font, font_semibold};

const WEEKDAYS: [(Weekday, &str); 7] = [
    (Weekday::Mon, "weekday-monday"),
    (Weekday::Tue, "weekday-tuesday"),
    (Weekday::Wed, "weekday-wednesday"),
    (Weekday::Thu, "weekday-thursday"),
    (Weekday::Fri, "weekday-friday"),
    (Weekday::Sat, "weekday-saturday"),
    (Weekday::Sun, "weekday-sunday"),
];

/// What the part remembers between frames (never saved).
#[derive(Debug, Default)]
pub(crate) struct PlansState {
    /// The selected template (`None` or gone: the first).
    template: Option<TemplateId>,
    /// The template name being typed.
    name_draft: Option<(TemplateId, String)>,
    /// The selected date (`None`: today) and the month the grid shows
    /// (its first day; `None`: the date's).
    date: Option<NaiveDate>,
    month: Option<NaiveDate>,
}

/// Opens a template or a date in its popup; Settings then closes, so a
/// name being typed is sent first.
fn open(scene: &Scene<'_>, view_state: &mut ViewState, st: &mut PlansState, target: EditTarget) {
    send_name(scene, st);
    view_state.plan_window = Some(target);
    view_state.plan_requested = true;
}

/// The template's name, or "?" when it is gone.
fn template_name(scene: &Scene<'_>, id: TemplateId) -> String {
    scene
        .state
        .schedule
        .template(id)
        .map_or_else(|| "?".to_owned(), |t| t.name.clone())
}

pub(super) fn section(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    st: &mut PlansState,
) {
    let t = scene.i18n;
    caption(ui, &t.tr("settings-plans"));
    ui.add_space(4.0);
    note(ui, &t.tr("settings-plans-hint"), theme::NEUTRAL_400);
    ui.add_space(6.0);
    ui.spacing_mut().item_spacing = vec2(8.0, 6.0);
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            ui.set_width(240.0);
            templates(ui, scene, view_state, st);
        });
        ui.add_space(24.0);
        ui.vertical(|ui| {
            ui.set_width(260.0);
            week(ui, scene);
        });
    });
    ui.add_space(12.0);
    days(ui, scene, view_state, st);
}

fn heading(ui: &mut Ui, text: &str) {
    ui.label(
        RichText::new(text)
            .font(font_semibold(13.0))
            .color(theme::TEXT),
    );
}

/// The day templates (AU9): the list, Edit…, New, Duplicate, Delete and
/// the name.
fn templates(ui: &mut Ui, scene: &Scene<'_>, view_state: &mut ViewState, st: &mut PlansState) {
    let t = scene.i18n;
    let schedule = &scene.state.schedule;
    heading(ui, &t.tr("settings-plans-templates"));
    let selected = st
        .template
        .filter(|id| schedule.template(*id).is_some())
        .or_else(|| schedule.templates().first().map(|t| t.id));
    for template in schedule.templates() {
        let response = ui.selectable_label(
            selected == Some(template.id),
            RichText::new(template.name.as_str()).font(font(13.0)),
        );
        if response.clicked() {
            send_name(scene, st);
            st.template = Some(template.id);
        }
        if response.double_clicked() {
            open(scene, view_state, st, EditTarget::Template(template.id));
        }
    }
    ui.add_space(6.0);
    ui.horizontal_wrapped(|ui| {
        if let Some(id) = selected
            && ui.button(t.tr("settings-plan-edit")).clicked()
        {
            open(scene, view_state, st, EditTarget::Template(id));
        }
        if ui.button(t.tr("schedule-template-new")).clicked() {
            scene.ctl.send(Command::CreateTemplate {
                name: t.tr("schedule-template-new"),
            });
        }
        if let Some(id) = selected {
            if ui.button(t.tr("schedule-template-duplicate")).clicked() {
                let name = t.tr_args(
                    "schedule-template-copy-name",
                    &[("name", template_name(scene, id).into())],
                );
                scene
                    .ctl
                    .send(Command::DuplicateTemplate { template: id, name });
            }
            if ui.button(t.tr("schedule-template-delete")).clicked() {
                scene.ctl.send(Command::DeleteTemplate(id));
            }
        }
    });
    if let Some(id) = selected {
        ui.add_space(6.0);
        rename(ui, scene, st, id);
    }
}

/// The template's name, sent when the field loses focus.
fn rename(ui: &mut Ui, scene: &Scene<'_>, st: &mut PlansState, id: TemplateId) {
    let t = scene.i18n;
    let current = template_name(scene, id);
    let draft = match &mut st.name_draft {
        Some((d, text)) if *d == id => text,
        slot => &mut slot.insert((id, current.clone())).1,
    };
    let label = ui.label(RichText::new(t.tr("schedule-template-name")).font(font(12.0)));
    let response = ui
        .add(egui::TextEdit::singleline(draft).desired_width(200.0))
        .labelled_by(label.id);
    if response.lost_focus() {
        send_name(scene, st);
    } else if !response.has_focus() {
        st.name_draft = None;
    }
}

/// Sends the name being typed, if it changes the template's, and forgets
/// it.
fn send_name(scene: &Scene<'_>, st: &mut PlansState) {
    let Some((id, draft)) = st.name_draft.take() else {
        return;
    };
    let name = draft.trim().to_owned();
    if !name.is_empty() && name != template_name(scene, id) {
        scene
            .ctl
            .send(Command::RenameTemplate { template: id, name });
    }
}

/// The week plan (AU9): each weekday's template.
fn week(ui: &mut Ui, scene: &Scene<'_>) {
    let t = scene.i18n;
    let schedule = &scene.state.schedule;
    heading(ui, &t.tr("settings-plans-week"));
    for (weekday, key) in WEEKDAYS {
        let current = schedule.week().get(weekday);
        let mut choice = current;
        egui::ComboBox::from_label(RichText::new(t.tr(key)).font(font(13.0)).color(theme::TEXT))
            .width(140.0)
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

fn first_of_month(date: NaiveDate) -> NaiveDate {
    date.with_day(1).unwrap_or(date)
}

/// Single dates (AU10): the month grid, the chosen date's template and
/// Edit day….
fn days(ui: &mut Ui, scene: &Scene<'_>, view_state: &mut ViewState, st: &mut PlansState) {
    let t = scene.i18n;
    let today = scene.state.local_now.date();
    let date = st.date.unwrap_or(today);
    heading(ui, &t.tr("settings-plans-days"));
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            ui.set_width(240.0);
            month_grid(ui, scene, st, date, today);
        });
        ui.add_space(24.0);
        ui.vertical(|ui| {
            ui.set_width(260.0);
            date_part(ui, scene, view_state, st, date, today);
        });
    });
}

/// The chosen date, its template and Edit day….
fn date_part(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    st: &mut PlansState,
    date: NaiveDate,
    today: NaiveDate,
) {
    let t = scene.i18n;
    let schedule = &scene.state.schedule;
    ui.label(
        RichText::new(date.format("%Y-%m-%d").to_string())
            .font(font_semibold(13.0))
            .color(theme::TEXT),
    );
    let own = schedule.date_plan(date).and_then(|p| p.template);
    let weekday_text = t.tr_args(
        "schedule-date-weekday-template",
        &[(
            "name",
            schedule
                .week()
                .get(date.weekday())
                .map(|id| template_name(scene, id))
                .unwrap_or_default()
                .into(),
        )],
    );
    let mut choice = own;
    egui::ComboBox::from_label(t.tr("schedule-date-template"))
        .width(160.0)
        .selected_text(own.map_or_else(|| weekday_text.clone(), |id| template_name(scene, id)))
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut choice, None, weekday_text.as_str());
            for template in schedule.templates() {
                ui.selectable_value(&mut choice, Some(template.id), template.name.as_str());
            }
        });
    if choice != own {
        scene.ctl.send(Command::SetDateTemplate {
            date,
            template: choice,
        });
    }
    // Past dates are not edited (AR19).
    if ui
        .add_enabled(
            date >= today,
            egui::Button::new(t.tr("settings-plan-edit-day")),
        )
        .clicked()
    {
        open(scene, view_state, st, EditTarget::Date(date));
    }
}

/// Previous and Next month, Today, and the month's days (Monday first);
/// the chosen date is highlighted and dates with changes are amber.
fn month_grid(
    ui: &mut Ui,
    scene: &Scene<'_>,
    st: &mut PlansState,
    date: NaiveDate,
    today: NaiveDate,
) {
    let t = scene.i18n;
    let month = st.month.unwrap_or_else(|| first_of_month(date));
    ui.horizontal(|ui| {
        let previous = ui
            .button(icon::CARET_LEFT)
            .on_hover_text(t.tr("schedule-date-prev-month"));
        previous.widget_info(|| {
            egui::WidgetInfo::labeled(egui::Role::Button, true, t.tr("schedule-date-prev-month"))
        });
        if previous.clicked() {
            st.month = month.checked_sub_months(Months::new(1));
        }
        ui.label(
            RichText::new(format!(
                "{} {}",
                t.tr(&format!("month-{}", month.month())),
                month.year()
            ))
            .font(font_semibold(13.0)),
        );
        let next = ui
            .button(icon::CARET_RIGHT)
            .on_hover_text(t.tr("schedule-date-next-month"));
        next.widget_info(|| {
            egui::WidgetInfo::labeled(egui::Role::Button, true, t.tr("schedule-date-next-month"))
        });
        if next.clicked() {
            st.month = month.checked_add_months(Months::new(1));
        }
        if ui.button(t.tr("schedule-date-today")).clicked() {
            st.date = Some(today);
            st.month = Some(first_of_month(today));
        }
    });
    let schedule = &scene.state.schedule;
    egui::Grid::new("settings-plan-month")
        .num_columns(7)
        .spacing(vec2(2.0, 2.0))
        .show(ui, |ui| {
            for (_, key) in WEEKDAYS {
                let short = t.tr(&key.replacen("weekday-", "weekday-short-", 1));
                ui.label(
                    RichText::new(short)
                        .font(font(11.0))
                        .color(theme::NEUTRAL_500),
                );
            }
            ui.end_row();
            let lead = month.weekday().num_days_from_monday();
            for _ in 0..lead {
                ui.label("");
            }
            let mut day = month;
            let mut column = lead;
            while day.month() == month.month() {
                let changed = schedule.date_plan(day).is_some();
                let mut text = RichText::new(day.day().to_string()).font(font(12.0));
                if changed {
                    text = text.color(theme::AMBER);
                }
                if day == today {
                    text = text.underline();
                }
                let mut response = ui.add_sized(
                    vec2(32.0, 24.0),
                    egui::Button::selectable(day == date, text),
                );
                if changed {
                    response = response.on_hover_text(t.tr("schedule-date-changes"));
                }
                if response.clicked() {
                    st.date = Some(day);
                }
                column += 1;
                if column.is_multiple_of(7) {
                    ui.end_row();
                }
                let Some(next) = day.checked_add_days(Days::new(1)) else {
                    break;
                };
                day = next;
            }
        });
}
