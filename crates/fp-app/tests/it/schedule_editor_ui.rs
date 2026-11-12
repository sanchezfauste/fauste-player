#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Automation spec §8.2: the schedule editor.

use crate::support;

use egui_kittest::kittest::Queryable;
use fp_model::{AppState, Command, TemplateId, apply};
use support::{harness, state};

/// The Schedule column shown, a "Base" template and a "Weekend" one.
fn with_schedule() -> (AppState, TemplateId, TemplateId) {
    let mut s = state(4, 3);
    fp_model::rotation::ensure_default(&mut s, "Music");
    fp_model::filler::ensure(&mut s, "Music filler");
    fp_model::automation::ensure(&mut s, "Schedule");
    fp_model::schedule::ensure_default(&mut s, "Base");
    apply(
        &mut s,
        Command::CreateTemplate {
            name: "Weekend".into(),
        },
    )
    .unwrap();
    let _ = apply(&mut s, Command::SetLayoutLocked(false));
    let p = s.automation.as_ref().unwrap().player;
    s.layout.hidden_players.retain(|x| *x != p);
    s.local_now = chrono::NaiveDate::from_ymd_opt(2026, 11, 17)
        .unwrap()
        .and_hms_opt(8, 0, 0)
        .unwrap();
    let base = s.schedule.templates()[0].id;
    let weekend = s.schedule.templates()[1].id;
    (s, base, weekend)
}

fn open(h: &mut egui_kittest::Harness<'_, fp_app::ui::app::AppUi>) {
    h.get_by_label("Edit schedule").click();
    h.run_steps(2);
}

#[test]
fn the_edit_button_opens_the_editor() {
    let (s, _, _) = with_schedule();
    let (mut h, _fake) = harness(s);
    assert!(h.query_by_label("Schedule editor").is_none());
    open(&mut h);
    assert!(h.query_by_label("Schedule editor").is_some());
}

#[test]
fn the_week_tab_sets_a_weekday_template() {
    let (s, _, weekend) = with_schedule();
    let (mut h, fake) = harness(s);
    open(&mut h);
    h.get_by_label("Week plan").click();
    h.run_steps(2);
    h.get_by_label("Saturday").click();
    h.run_steps(2);
    h.get_by_label("Weekend").click();
    h.run_steps(2);
    assert!(fake.take_sent().contains(&Command::SetWeekTemplate {
        weekday: chrono::Weekday::Sat,
        template: weekend,
    }));
}

#[test]
fn rotations_opens_the_automation_settings() {
    let (s, _, _) = with_schedule();
    let (mut h, _fake) = harness(s);
    open(&mut h);
    h.get_by_label("Rotations…").click();
    h.run_steps(3);
    assert!(h.query_all_by_label_contains("Look-ahead").count() >= 1);
}
