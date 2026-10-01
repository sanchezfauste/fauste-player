#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Feedback 2 spec O6: closing while audio is on air asks first.

mod support;

use egui::{Key, ViewportCommand, ViewportEvent, ViewportId};
use egui_kittest::kittest::Queryable;
use fp_model::{Command, PlayingCart};
use support::{harness, state};

const GUARD: &str = "Audio is on air";

fn request_close(h: &mut egui_kittest::Harness<'static, fp_app::ui::app::AppUi>) {
    h.input_mut()
        .viewports
        .entry(ViewportId::ROOT)
        .or_default()
        .events
        .push(ViewportEvent::Close);
    h.run_steps(2);
}

fn sent_viewport_command(
    h: &egui_kittest::Harness<'static, fp_app::ui::app::AppUi>,
    wanted: &ViewportCommand,
) -> bool {
    h.output()
        .viewport_output
        .get(&ViewportId::ROOT)
        .is_some_and(|v| v.commands.contains(wanted))
}

fn playing(players: usize) -> fp_model::AppState {
    let mut s = state(players, 3);
    let p = s.players[0].id;
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    s
}

#[test]
fn closing_with_nothing_on_air_shows_no_dialog() {
    let (mut h, fake) = harness(state(1, 3));
    request_close(&mut h);
    assert!(h.query_by_label(GUARD).is_none());
    assert!(!sent_viewport_command(&h, &ViewportCommand::CancelClose));
    assert!(fake.take_sent().is_empty());
}

#[test]
fn closing_while_playing_cancels_and_asks() {
    let (mut h, fake) = harness(playing(1));
    request_close(&mut h);
    assert!(h.query_by_label(GUARD).is_some());
    assert!(h.query_by_label_contains("P1 — Song 1").is_some());
    assert!(fake.take_sent().is_empty());
}

#[test]
fn cancel_and_escape_keep_everything_playing() {
    let (mut h, fake) = harness(playing(1));
    request_close(&mut h);
    h.get_by_label("Cancel").click();
    h.run_steps(2);
    assert!(h.query_by_label(GUARD).is_none());
    request_close(&mut h);
    h.key_press(Key::Escape);
    h.run_steps(2);
    assert!(h.query_by_label(GUARD).is_none());
    assert!(fake.take_sent().is_empty());
}

#[test]
fn stop_and_close_stops_players_and_carts_then_closes() {
    let mut s = playing(2);
    let cart = s.cartwall.pages[0].carts[0].id;
    s.cartwall.playing.push(PlayingCart {
        cart,
        looped: false,
    });
    let p = s.players[0].id;
    let (mut h, fake) = harness(s);
    request_close(&mut h);
    h.get_by_label("Stop and close").click();
    h.run_steps(1);
    assert_eq!(
        fake.take_sent(),
        vec![Command::Stop(p), Command::StopAllCarts]
    );
    assert!(sent_viewport_command(&h, &ViewportCommand::Close));
}

#[test]
fn a_second_close_request_keeps_one_dialog_and_stops_nothing() {
    let (mut h, fake) = harness(playing(1));
    request_close(&mut h);
    request_close(&mut h);
    assert_eq!(h.query_all_by_label(GUARD).count(), 1);
    assert!(fake.take_sent().is_empty());
}

#[test]
fn the_guard_shows_over_settings() {
    let (mut h, _) = harness(playing(1));
    h.get_by_label_contains("Settings").click();
    h.run_steps(2);
    request_close(&mut h);
    assert!(h.query_by_label(GUARD).is_some());
}

#[test]
fn shortcuts_do_nothing_while_the_guard_is_open() {
    let (mut h, fake) = harness(playing(1));
    request_close(&mut h);
    h.key_press(Key::Space);
    h.key_press(Key::Num1);
    h.run_steps(2);
    assert!(fake.take_sent().is_empty());
}

#[test]
fn cancel_works_over_settings() {
    let (mut h, fake) = harness(playing(1));
    h.get_by_label_contains("Settings").click();
    h.run_steps(2);
    request_close(&mut h);
    h.get_by_label("Cancel").click();
    h.run_steps(2);
    assert!(h.query_by_label(GUARD).is_none());
    assert!(fake.take_sent().is_empty());
}
