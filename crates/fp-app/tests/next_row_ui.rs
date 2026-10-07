#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Plan 13, O37 and O38: the entry on air can be its own next, and the
//! stop-after control is available in SINGLE mode while the entry repeats.

mod support;

use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use fp_app::ui::app::AppUi;
use fp_model::{AppState, Command, PlayMode, apply};
use support::{harness, state};

const NEXT_ARROW: &str = egui_phosphor::regular::ARROW_BEND_DOWN_RIGHT;
const STOP_AFTER: &str = "Stop after the current track";
const NOT_AVAILABLE: &str =
    "Not available in SINGLE mode unless the track repeats: every track already stops at its end";

fn arrows(h: &Harness<'_, AppUi>) -> usize {
    h.query_all_by_label_contains(NEXT_ARROW).count()
}

fn playing() -> AppState {
    let mut s = state(1, 3);
    let p = s.players[0].id;
    apply(&mut s, Command::Play(p)).unwrap();
    s
}

fn playing_as_next() -> AppState {
    let mut s = playing();
    let p = s.players[0].id;
    let current = s.players[0].current.unwrap();
    apply(&mut s, Command::SetNext(p, current)).unwrap();
    s
}

#[test]
fn the_playing_row_shows_the_next_arrow_once_it_is_set_as_next() {
    let (h, _) = harness(playing());
    assert_eq!(arrows(&h), 1, "only the derived next row has it");
    let (h, _) = harness(playing_as_next());
    assert_eq!(
        arrows(&h),
        1,
        "the playing row has it and the next row has lost it"
    );
}

#[test]
fn set_as_next_is_enabled_on_the_playing_row_and_play_now_is_not() {
    let (mut h, _) = harness(playing());
    h.get_all_by_label("Song 1")
        .last()
        .unwrap()
        .click_secondary();
    h.run_steps(2);
    assert!(
        !h.get_by_label("Set as next").accesskit_node().is_disabled(),
        "Set as next"
    );
    assert!(
        h.get_by_label("Play now").accesskit_node().is_disabled(),
        "Play now"
    );
}

#[test]
fn set_as_next_is_disabled_on_the_playing_row_that_is_already_the_next() {
    let (mut h, _) = harness(playing_as_next());
    h.get_all_by_label("Song 1")
        .last()
        .unwrap()
        .click_secondary();
    h.run_steps(2);
    assert!(h.get_by_label("Set as next").accesskit_node().is_disabled());
}

#[test]
fn set_as_next_on_the_playing_row_sends_set_next_of_the_current() {
    let (mut h, fake) = harness(playing());
    let (p, e) = (fake.player(0), fake.entries());
    h.get_all_by_label("Song 1")
        .last()
        .unwrap()
        .click_secondary();
    h.run_steps(2);
    h.get_by_label("Set as next").click();
    h.run_steps(2);
    assert_eq!(fake.take_sent(), vec![Command::SetNext(p, e[0])]);
}

#[test]
fn a_double_click_on_the_playing_row_sets_it_as_next() {
    let (mut h, fake) = harness(playing());
    let (p, e) = (fake.player(0), fake.entries());
    h.get_all_by_label("Song 1").last().unwrap().click();
    h.step();
    h.get_all_by_label("Song 1").last().unwrap().click();
    h.run_steps(2);
    assert!(fake.take_sent().contains(&Command::SetNext(p, e[0])));
}

#[test]
fn single_mode_with_a_repeating_entry_enables_stop_after_current() {
    let mut s = playing();
    let (p, e) = (s.players[0].id, s.players[0].current.unwrap());
    apply(&mut s, Command::SetMode(p, PlayMode::Single)).unwrap();
    apply(&mut s, Command::ToggleEntryRepeat(e)).unwrap();
    let (mut h, fake) = harness(s);
    let button = h.get_by_label(STOP_AFTER);
    assert!(!button.accesskit_node().is_disabled());
    button.click();
    h.run_steps(2);
    assert_eq!(fake.take_sent(), vec![Command::ToggleStopAfterCurrent(p)]);
    assert!(h.query_by_label(NOT_AVAILABLE).is_none());
}

#[test]
fn single_mode_without_repeat_disables_stop_after_current_with_the_single_tip() {
    let mut s = playing();
    let p = s.players[0].id;
    apply(&mut s, Command::SetMode(p, PlayMode::Single)).unwrap();
    let (mut h, fake) = harness(s);
    let button = h.get_by_label(NOT_AVAILABLE);
    assert!(button.accesskit_node().is_disabled());
    button.click();
    h.run_steps(2);
    assert!(fake.take_sent().is_empty());
    assert!(h.query_by_label(STOP_AFTER).is_none());
}
