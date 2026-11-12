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
