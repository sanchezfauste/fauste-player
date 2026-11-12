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
    // The window settles its size over a few frames.
    h.run_steps(6);
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

/// A 10:00 block on Base with "Top of the Hour" (id 9000, `secs` long).
fn with_block(start: (u32, u32, u32), secs: f64) -> (AppState, TemplateId, fp_model::BlockId) {
    use fp_model::schedule::{Block, Item};
    let (mut s, base, _) = with_schedule();
    let mut t = fp_model::Track::new(fp_model::TrackId(9000), "/m/top.wav".into());
    t.title = "Top of the Hour".into();
    t.duration_secs = secs;
    s.library.insert(t);
    let start = fp_model::local_time::TimeOfDay::hms(start.0, start.1, start.2).unwrap();
    apply(
        &mut s,
        Command::AddBlock {
            template: base,
            start,
        },
    )
    .unwrap();
    let block = s.schedule.template(base).unwrap().blocks[0].id;
    apply(
        &mut s,
        Command::UpdateBlock {
            template: base,
            block: Block {
                id: block,
                start,
                items: vec![Item::Track {
                    track: fp_model::TrackId(9000),
                    time_critical: false,
                }],
            },
        },
    )
    .unwrap();
    (s, base, block)
}

/// Replaces the text of the field labelled `label` and presses Enter.
fn retype(h: &mut egui_kittest::Harness<'_, fp_app::ui::app::AppUi>, label: &str, text: &str) {
    let role = egui::accesskit::Role::TextInput;
    h.get_by_role_and_label(role, label).click();
    h.run_steps(2);
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    h.run_steps(1);
    h.get_by_role_and_label(role, label).type_text(text);
    h.run_steps(1);
    h.key_press(egui::Key::Enter);
    h.run_steps(2);
}

#[test]
fn new_template_sends_create_template() {
    let (s, _, _) = with_schedule();
    let (mut h, fake) = harness(s);
    open(&mut h);
    h.get_by_label("New template").click();
    h.run_steps(2);
    assert!(fake.take_sent().contains(&Command::CreateTemplate {
        name: "New template".into()
    }));
}

#[test]
fn a_valid_start_sends_update_block() {
    let (s, base, block) = with_block((10, 0, 0), 12.0);
    let (mut h, fake) = harness(s);
    open(&mut h);
    retype(&mut h, "Starts at", "11:30:00");
    let sent = fake.take_sent();
    assert!(
        sent.iter()
            .any(|c| matches!(c, Command::UpdateBlock { template, block: b }
            if *template == base && b.id == block
                && b.start == fp_model::local_time::TimeOfDay::hms(11, 30, 0).unwrap())),
        "{sent:?}"
    );
}

#[test]
fn a_bad_start_is_not_sent() {
    let (s, _, _) = with_block((10, 0, 0), 12.0);
    let (mut h, fake) = harness(s);
    open(&mut h);
    retype(&mut h, "Starts at", "25:00");
    assert!(
        !fake
            .take_sent()
            .iter()
            .any(|c| matches!(c, Command::UpdateBlock { .. }))
    );
    assert!(h.query_by_label("Use HH:MM:SS").is_some());
}

#[test]
fn the_overrun_shows_on_the_block() {
    let (s, _, _) = with_block((23, 59, 0), 120.0);
    let (mut h, _fake) = harness(s);
    open(&mut h);
    assert!(h.query_by_label("Too long by 01:00").is_some());
}

#[test]
fn remove_block_sends_remove_block() {
    let (s, base, block) = with_block((10, 0, 0), 12.0);
    let (mut h, fake) = harness(s);
    open(&mut h);
    h.get_by_label("Remove block").click();
    h.run_steps(2);
    let sent = fake.take_sent();
    assert!(
        sent.contains(&Command::RemoveBlock {
            template: base,
            block
        }),
        "{sent:?}"
    );
}
