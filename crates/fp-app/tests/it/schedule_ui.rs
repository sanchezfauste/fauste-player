#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Automation spec §8.1: the Schedule column.

use crate::support;

use egui_kittest::kittest::Queryable;
use fp_model::{AppState, Command, apply};
use support::{harness, state};

/// Four players, three tracks, the filler and the schedule's player, its
/// column shown.
fn with_schedule() -> AppState {
    let mut s = state(4, 3);
    fp_model::rotation::ensure_default(&mut s, "Music");
    fp_model::filler::ensure(&mut s, "Music filler");
    fp_model::automation::ensure(&mut s, "Schedule");
    fp_model::schedule::ensure_default(&mut s, "Base");
    let _ = apply(&mut s, Command::SetLayoutLocked(false));
    let p = s.automation.as_ref().unwrap().player;
    s.layout.hidden_players.retain(|x| *x != p);
    s
}

#[test]
fn start_sends_start_automation() {
    let (mut h, fake) = harness(with_schedule());
    h.get_by_label("Start").click();
    h.run();
    assert!(fake.take_sent().contains(&Command::StartAutomation));
}

#[test]
fn stop_after_current_sends_its_command_while_running() {
    let mut s = with_schedule();
    s.automation.as_mut().unwrap().running = true;
    let (mut h, fake) = harness(s);
    assert!(h.query_by_label("Running").is_some());
    h.get_by_label("Stop after current").click();
    h.run();
    assert!(
        fake.take_sent()
            .contains(&Command::StopAutomationAfterCurrent)
    );
}

#[test]
fn the_state_says_stopped_after_a_start_of_the_app() {
    let (h, _fake) = harness(with_schedule());
    assert!(h.query_by_label("Stopped").is_some());
}

/// Today at 08:00, a 10:00 block with "Top of the Hour" (id 9000) and a
/// category item.
fn with_block() -> (AppState, fp_model::automation::ItemRef) {
    use fp_model::schedule::{Block, Item};
    let mut s = with_schedule();
    let date = chrono::NaiveDate::from_ymd_opt(2026, 11, 17).unwrap();
    s.local_now = date.and_hms_opt(8, 0, 0).unwrap();
    let mut t = fp_model::Track::new(fp_model::TrackId(9000), "/m/top.wav".into());
    t.title = "Top of the Hour".into();
    t.duration_secs = 12.0;
    s.library.insert(t);
    let base = s.schedule.templates()[0].id;
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
    let item = fp_model::automation::ItemRef {
        date,
        block,
        index: 0,
    };
    (s, item)
}

#[test]
fn the_list_shows_blocks_and_their_items() {
    let (s, _) = with_block();
    let (h, _fake) = harness(s);
    // The template is every day's: today's, yesterday's last and
    // tomorrow's first block are all at 10:00.
    assert_eq!(h.query_all_by_label_contains("10:00:00").count(), 3);
    assert_eq!(h.query_all_by_label_contains("Top of the Hour").count(), 3);
    assert_eq!(h.query_all_by_label_contains("tomorrow").count(), 1);
}

#[test]
fn a_missed_item_is_struck_through() {
    let (mut s, item) = with_block();
    // Missed, and not next: the clock had moved on.
    let a = s.automation.as_mut().unwrap();
    a.missed.insert(item);
    a.next = None;
    let (h, _fake) = harness(s);
    let row = h.get_by_label_contains("did not air");
    assert!(
        format!("{row:?}").contains("Top of the Hour"),
        "the missed row is the item's"
    );
}

#[test]
fn an_empty_schedule_says_so() {
    let (h, _fake) = harness(with_schedule());
    assert!(h.query_by_label("No blocks today").is_some());
}

#[test]
fn a_deleted_track_shows_not_available() {
    let (mut s, _) = with_block();
    s.library.remove(fp_model::TrackId(9000));
    let (h, _fake) = harness(s);
    assert!(h.query_all_by_label_contains("Not available").count() >= 1);
}

/// Today's row of "Top of the Hour" (yesterday's last block comes first).
fn todays_row(h: &egui_kittest::Harness<'_, fp_app::ui::app::AppUi>) -> egui::Pos2 {
    h.get_all_by_label_contains("Top of the Hour")
        .nth(1)
        .unwrap()
        .rect()
        .center()
}

#[test]
fn double_click_sets_the_item_as_next() {
    use egui::{Event, Modifiers, PointerButton};
    let (s, item) = with_block();
    let (mut h, fake) = harness(s);
    let at = todays_row(&h);
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    for _ in 0..2 {
        for pressed in [true, false] {
            h.event(Event::PointerButton {
                pos: at,
                button: PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            });
        }
        h.step();
    }
    h.run_steps(2);
    assert!(fake.take_sent().contains(&Command::SetScheduleNext(item)));
}

#[test]
fn the_menu_sets_and_clears_next() {
    let (mut s, item) = with_block();
    let (mut h, fake) = harness(s.clone());
    h.get_all_by_label_contains("Top of the Hour")
        .nth(1)
        .unwrap()
        .click_secondary();
    h.run_steps(2);
    h.get_by_label("Play next").click();
    h.run_steps(2);
    assert!(fake.take_sent().contains(&Command::SetScheduleNext(item)));
    // A next set by hand can go back to the clock.
    {
        let a = s.automation.as_mut().unwrap();
        a.next = Some(item);
        a.next_by_hand = true;
    }
    let (mut h, fake) = harness(s);
    h.get_all_by_label_contains("Top of the Hour")
        .nth(1)
        .unwrap()
        .click_secondary();
    h.run_steps(2);
    h.get_by_label("Back to the clock").click();
    h.run_steps(2);
    assert!(fake.take_sent().contains(&Command::ClearScheduleNext));
}
