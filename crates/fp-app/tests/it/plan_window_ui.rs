#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Automation rework AR21–AR22: a template or a date in a playlist-like
//! popup.

use crate::support;

use egui_kittest::kittest::Queryable;
use fp_model::log_edit::{EditTarget, LogEdit};
use fp_model::schedule::{Block, Item};
use fp_model::{AppState, Command, TemplateId, apply};
use support::{harness_with, state};

/// "Base" with a 10:00 block holding "Top of the Hour", and "Weekend";
/// today is Tuesday 2026-11-17, 08:00.
fn with_templates() -> (AppState, TemplateId, TemplateId) {
    let mut s = state(4, 3);
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
    let mut t = fp_model::Track::new(fp_model::TrackId(9000), "/m/top.wav".into());
    t.title = "Top of the Hour".into();
    t.duration_secs = 12.0;
    s.library.insert(t);
    let base = s.schedule.templates()[0].id;
    let weekend = s.schedule.templates()[1].id;
    let start = fp_model::local_time::TimeOfDay::hms(10, 0, 0).unwrap();
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
    (s, base, weekend)
}

fn open(
    s: AppState,
    target: EditTarget,
) -> (
    egui_kittest::Harness<'static, fp_app::ui::app::AppUi>,
    std::sync::Arc<support::Fake>,
) {
    let (mut h, fake) = harness_with(s, |ui| ui.open_plan_window(target));
    h.run_steps(4);
    (h, fake)
}

fn edits(sent: &[Command]) -> Vec<(EditTarget, LogEdit)> {
    sent.iter()
        .filter_map(|c| match c {
            Command::EditPlan { target, edit } => Some((*target, edit.clone())),
            _ => None,
        })
        .collect()
}

#[test]
fn the_popup_lists_a_templates_blocks() {
    let (s, base, _) = with_templates();
    let (h, _fake) = open(s, EditTarget::Template(base));
    assert!(h.query_by_label("Template · Base").is_some());
    // The Log shows yesterday's, today's and tomorrow's 10:00 block (its
    // column is hidden here); the popup shows the template's one.
    assert_eq!(h.query_all_by_label_contains("Top of the Hour").count(), 1);
    assert!(h.query_by_label_contains("10:00:00").is_some());
}

#[test]
fn add_block_in_the_popup_edits_the_template() {
    let (s, base, _) = with_templates();
    let (mut h, fake) = open(s, EditTarget::Template(base));
    fake.take_sent();
    h.get_by_label("Add to the list").click();
    h.run();
    h.get_by_label("Block…").click();
    h.run();
    h.get_by_label("OK").click();
    h.run();
    let sent = edits(&fake.take_sent());
    assert_eq!(
        sent,
        vec![(
            EditTarget::Template(base),
            LogEdit::AddBlock {
                start: fp_model::local_time::TimeOfDay::hms(11, 0, 0).unwrap()
            }
        )]
    );
}

#[test]
fn remove_in_the_popup_edits_the_template() {
    let (s, base, _) = with_templates();
    let block = s.schedule.template(base).unwrap().blocks[0].id;
    let (mut h, fake) = open(s, EditTarget::Template(base));
    fake.take_sent();
    h.get_by_label_contains("Top of the Hour").click_secondary();
    h.run();
    h.get_by_label("Remove").click();
    h.run();
    assert_eq!(
        edits(&fake.take_sent()),
        vec![(
            EditTarget::Template(base),
            LogEdit::Remove { block, index: 0 }
        )]
    );
}

#[test]
fn the_popup_closes_when_its_template_is_deleted() {
    let (mut s, _, weekend) = with_templates();
    apply(&mut s, Command::DeleteTemplate(weekend)).unwrap();
    let (h, _fake) = open(s, EditTarget::Template(weekend));
    assert!(h.query_by_label_contains("Template ·").is_none());
}

#[test]
fn a_date_popup_edits_that_date() {
    let (s, _, _) = with_templates();
    let thursday = chrono::NaiveDate::from_ymd_opt(2026, 11, 19).unwrap();
    let (mut h, fake) = open(s, EditTarget::Date(thursday));
    assert!(h.query_by_label("Day · 2026-11-19").is_some());
    fake.take_sent();
    h.get_by_label_contains("Top of the Hour").click_secondary();
    h.run();
    h.get_by_label("Remove").click();
    h.run();
    let sent = edits(&fake.take_sent());
    assert!(matches!(
        sent.as_slice(),
        [(EditTarget::Date(d), LogEdit::Remove { index: 0, .. })] if *d == thursday
    ));
}

#[test]
fn tracks_dropped_on_the_popup_go_into_the_template() {
    let (s, base, _) = with_templates();
    let block = s.schedule.template(base).unwrap().blocks[0].id;
    let (mut h, fake) = open(s, EditTarget::Template(base));
    fake.take_sent();
    let at = h.get_by_label_contains("Top of the Hour").rect().center();
    h.event(egui::Event::PointerMoved(at));
    h.run();
    fp_app::ui::app::AppUi::start_track_drag(&h.ctx, vec![fp_model::TrackId(1)]);
    h.run();
    h.event(egui::Event::PointerButton {
        pos: at,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::NONE,
    });
    h.run();
    assert_eq!(
        edits(&fake.take_sent()),
        vec![(
            EditTarget::Template(base),
            LogEdit::Insert {
                block,
                index: 0,
                items: vec![Item::Track {
                    track: fp_model::TrackId(1),
                    time_critical: false,
                }],
            }
        )]
    );
}

#[test]
fn an_empty_template_has_no_blocks_not_none_today() {
    let (s, _, weekend) = with_templates();
    let (h, _fake) = open(s, EditTarget::Template(weekend));
    assert!(h.query_by_label("Template · Weekend").is_some());
    assert!(h.query_by_label("No blocks yet").is_some());
    assert!(h.query_by_label("No blocks today").is_none());
}

#[test]
fn a_past_dates_popup_closes() {
    let (s, _, _) = with_templates();
    let monday = chrono::NaiveDate::from_ymd_opt(2026, 11, 16).unwrap();
    let (h, _fake) = open(s, EditTarget::Date(monday));
    assert!(h.query_by_label_contains("Day ·").is_none());
}

#[test]
fn closing_the_popup_drops_its_open_dialog() {
    let (s, base, _) = with_templates();
    let (mut h, fake) = open(s, EditTarget::Template(base));
    h.get_by_label("Add to the list").click();
    h.run();
    h.get_by_label("Block…").click();
    h.run();
    assert!(h.query_by_label("OK").is_some());
    h.get_by_label("Close window").click();
    h.run();
    h.run();
    assert!(h.query_by_label_contains("Template ·").is_none());
    assert!(h.query_by_label("OK").is_none(), "the dialog went with it");
    fake.take_sent();
}

#[test]
fn todays_popup_adds_nothing_to_a_block_that_already_played() {
    let (mut s, _, _) = with_templates();
    let today = s.local_now.date();
    s.local_now = today.and_hms_opt(11, 0, 0).unwrap();
    let block = s.schedule.day(today)[0].id;
    s.automation
        .as_mut()
        .unwrap()
        .started
        .insert((today, block));
    let (mut h, _fake) = open(s, EditTarget::Date(today));
    assert!(h.query_by_label("Day · 2026-11-17").is_some());
    h.get_by_label("Add to the list").click();
    h.run();
    assert!(h.query_by_label("Block…").is_some());
    assert!(h.query_by_label("Category track…").is_none());
}
