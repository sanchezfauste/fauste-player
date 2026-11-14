#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Automation rework AR20: templates, the week plan and dates in Settings →
//! Automation.

use crate::support;

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::AppUi;
use fp_model::{AppState, Command, TemplateId, apply};
use support::{Fake, harness, state};

/// "Base" and "Weekend"; today is Tuesday 2026-11-17.
fn with_templates() -> (AppState, TemplateId, TemplateId) {
    let mut s = state(2, 1);
    fp_model::rotation::ensure_default(&mut s, "Music");
    fp_model::filler::ensure(&mut s, "Music fill");
    fp_model::automation::ensure(&mut s, "Log");
    fp_model::schedule::ensure_default(&mut s, "Base");
    apply(
        &mut s,
        Command::CreateTemplate {
            name: "Weekend".into(),
        },
    )
    .unwrap();
    s.local_now = chrono::NaiveDate::from_ymd_opt(2026, 11, 17)
        .unwrap()
        .and_hms_opt(8, 0, 0)
        .unwrap();
    let base = s.schedule.templates()[0].id;
    let weekend = s.schedule.templates()[1].id;
    (s, base, weekend)
}

fn open(s: AppState) -> (Harness<'static, AppUi>, std::sync::Arc<Fake>) {
    let (mut h, fake) = harness(s);
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Automation").click();
    h.run_steps(2);
    fake.take_sent();
    (h, fake)
}

/// Scrolls Settings until the widget labelled `label` is in view (the
/// part is below the rotations), so a click lands on it.
fn reveal(h: &mut Harness<'static, AppUi>, label: &str) {
    for _ in 0..80 {
        let rect = h.get_by_label(label).rect();
        let dy = if rect.top() < 120.0 {
            2.0
        } else if rect.bottom() > 560.0 {
            -2.0
        } else {
            // Scrolling is animated: let it settle before the click.
            h.run_steps(20);
            return;
        };
        let at = h.get_by_label("Close").rect().center() - egui::vec2(0.0, 200.0);
        h.event(egui::Event::PointerMoved(at));
        h.event(egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Line,
            delta: egui::vec2(0.0, dy),
            phase: egui::TouchPhase::Move,
            source: egui::MouseWheelSource::Unknown,
            modifiers: egui::Modifiers::NONE,
        });
        h.run_steps(4);
    }
}

#[test]
fn edit_opens_the_popup_and_closes_settings() {
    let (s, _, _) = with_templates();
    let (mut h, _fake) = open(s);
    reveal(&mut h, "Weekend");
    h.get_by_label("Weekend").click();
    h.run_steps(2);
    reveal(&mut h, "Edit…");
    h.get_by_label("Edit…").click();
    h.run_steps(4);
    assert!(h.query_by_label("Template · Weekend").is_some());
    // Settings is closed: its sections are gone.
    assert!(
        h.query_by_role_and_label(Role::Button, "Automation")
            .is_none()
    );
}

#[test]
fn the_week_plan_sets_a_weekday_template() {
    let (s, _, weekend) = with_templates();
    let (mut h, fake) = open(s);
    reveal(&mut h, "Saturday");
    h.get_by_label("Saturday").click();
    h.run_steps(2);
    let options: Vec<_> = h.query_all_by_label("Weekend").collect();
    options.last().unwrap().click();
    h.run_steps(2);
    assert!(fake.take_sent().contains(&Command::SetWeekTemplate {
        weekday: chrono::Weekday::Sat,
        template: weekend,
    }));
}

#[test]
fn a_date_can_use_another_template() {
    let (s, _, weekend) = with_templates();
    let (mut h, fake) = open(s);
    reveal(&mut h, "19");
    h.get_by_label("19").click();
    h.run_steps(2);
    reveal(&mut h, "Template");
    h.get_by_label("Template").click();
    h.run_steps(2);
    let options: Vec<_> = h.query_all_by_label("Weekend").collect();
    options.last().unwrap().click();
    h.run_steps(2);
    assert!(fake.take_sent().contains(&Command::SetDateTemplate {
        date: chrono::NaiveDate::from_ymd_opt(2026, 11, 19).unwrap(),
        template: Some(weekend),
    }));
}

#[test]
fn edit_day_opens_the_dates_popup() {
    let (s, _, _) = with_templates();
    let (mut h, _fake) = open(s);
    reveal(&mut h, "19");
    h.get_by_label("19").click();
    h.run_steps(2);
    reveal(&mut h, "Edit day…");
    h.get_by_label("Edit day…").click();
    h.run_steps(4);
    assert!(h.query_by_label("Day · 2026-11-19").is_some());
}

#[test]
fn new_template_sends_create_template() {
    let (s, _, _) = with_templates();
    let (mut h, fake) = open(s);
    reveal(&mut h, "New template");
    h.get_by_label("New template").click();
    h.run_steps(2);
    assert!(
        fake.take_sent()
            .iter()
            .any(|c| matches!(c, Command::CreateTemplate { .. }))
    );
}

#[test]
fn the_log_has_no_edit_button() {
    let (s, _, _) = with_templates();
    let (h, _fake) = harness(s);
    assert!(h.query_by_label("Edit log").is_none());
}

#[test]
fn edit_on_the_open_template_still_closes_settings() {
    let (s, base, _) = with_templates();
    let (mut h, _fake) = support::harness_with(s, |ui| {
        ui.open_plan_window(fp_model::log_edit::EditTarget::Template(base))
    });
    h.run_steps(2);
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Automation").click();
    h.run_steps(2);
    reveal(&mut h, "Edit…");
    h.get_by_label("Edit…").click();
    h.run_steps(4);
    assert!(
        h.query_by_role_and_label(Role::Button, "Automation")
            .is_none()
    );
    assert!(h.query_by_label("Template · Base").is_some());
}

#[test]
fn a_name_typed_before_edit_is_kept() {
    let (s, _, weekend) = with_templates();
    let (mut h, fake) = open(s);
    reveal(&mut h, "Weekend");
    h.get_by_label("Weekend").click();
    h.run_steps(2);
    reveal(&mut h, "Edit…");
    h.get(
        egui_kittest::kittest::by()
            .role(Role::TextInput)
            .value("Weekend"),
    )
    .focus();
    h.run_steps(2);
    h.event(egui::Event::Key {
        key: egui::Key::End,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    });
    h.event(egui::Event::Text(" days".into()));
    h.run_steps(2);
    reveal(&mut h, "Edit…");
    h.get_by_label("Edit…").click();
    h.run_steps(4);
    assert!(fake.take_sent().contains(&Command::RenameTemplate {
        template: weekend,
        name: "Weekend days".into(),
    }));
}
