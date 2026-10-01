#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Feedback 2 spec O4: settings that wait for a restart, and Restart now.

mod support;

use std::sync::Arc;
use std::sync::atomic::Ordering;

use arc_swap::ArcSwap;
use egui::accesskit::Role;
use egui::{ViewportCommand, ViewportId};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::AppUi;
use fp_model::Command;
use support::{Fake, harness, harness_from, state};

const NOTICE: &str = "Some changes take effect after a restart.";
const PILL: &str = "Restart pending";

fn sent_viewport_command(h: &Harness<'static, AppUi>, wanted: &ViewportCommand) -> bool {
    h.output()
        .viewport_output
        .get(&ViewportId::ROOT)
        .is_some_and(|v| v.commands.contains(wanted))
}

fn set_rate(h: &mut Harness<'static, AppUi>, fake: &Fake, rate: u32) {
    let mut s = (**fake.state.load()).clone();
    s.config.outputs.sample_rate = rate;
    fake.state.store(Arc::new(s));
    h.run_steps(2);
}

fn playing(players: usize) -> fp_model::AppState {
    let mut s = state(players, 3);
    let p = s.players[0].id;
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    s
}

#[test]
fn nothing_changed_shows_no_notice_and_no_pill() {
    let (mut h, _) = harness(state(1, 3));
    assert!(h.query_by_label(PILL).is_none());
    h.get_by_label("Settings").click();
    h.run_steps(2);
    assert!(h.query_by_label(NOTICE).is_none());
    assert!(h.query_by_label("Restart now").is_none());
}

#[test]
fn a_live_setting_needs_no_restart() {
    let (mut h, fake) = harness(state(1, 3));
    let mut s = (**fake.state.load()).clone();
    s.config.players.fade_ms = 3000;
    fake.state.store(Arc::new(s));
    h.run_steps(2);
    assert!(h.query_by_label(PILL).is_none());
}

#[test]
fn a_restart_only_change_shows_the_pill_and_the_notice() {
    let (mut h, fake) = harness(state(1, 3));
    set_rate(&mut h, &fake, 44_100);
    assert!(h.query_by_role_and_label(Role::Button, PILL).is_some());
    h.get_by_label("Settings").click();
    h.run_steps(2);
    assert!(h.query_by_label(NOTICE).is_some());
    assert!(h.query_by_label("Restart now").is_some());
}

#[test]
fn changing_back_hides_the_pill() {
    let (mut h, fake) = harness(state(1, 3));
    set_rate(&mut h, &fake, 44_100);
    set_rate(&mut h, &fake, 48_000);
    assert!(h.query_by_label(PILL).is_none());
}

#[test]
fn restart_now_with_nothing_on_air_closes_for_a_restart() {
    let (mut h, fake) = harness(state(1, 3));
    set_rate(&mut h, &fake, 44_100);
    h.get_by_label("Settings").click();
    h.run_steps(2);
    fake.take_sent();
    h.get_by_label("Restart now").click();
    h.run_steps(1);
    assert!(h.query_by_label("Audio is on air").is_none());
    assert!(sent_viewport_command(&h, &ViewportCommand::Close));
    assert!(h.state().restart_flag().load(Ordering::Acquire));
    assert!(fake.take_sent().is_empty());
}

#[test]
fn the_pill_restarts_at_once_when_nothing_is_on_air() {
    let (mut h, fake) = harness(state(1, 3));
    set_rate(&mut h, &fake, 44_100);
    fake.take_sent();
    h.get_by_label(PILL).click();
    h.run_steps(1);
    // No confirmation: nothing would be cut.
    assert!(h.query_by_label("Audio is on air").is_none());
    assert!(sent_viewport_command(&h, &ViewportCommand::Close));
    assert!(h.state().restart_flag().load(Ordering::Acquire));
    assert!(fake.take_sent().is_empty());
}

#[test]
fn the_pill_asks_first_while_audio_is_on_air() {
    let (mut h, fake) = harness(playing(1));
    let p = fake.player(0);
    set_rate(&mut h, &fake, 44_100);
    h.get_by_label(PILL).click();
    h.run_steps(2);
    assert!(h.query_by_label("Audio is on air").is_some());
    assert!(
        h.query_by_label("Restarting stops everything that is playing:")
            .is_some()
    );
    assert!(!h.state().restart_flag().load(Ordering::Acquire));
    h.get_by_label("Stop and restart").click();
    h.run_steps(1);
    assert_eq!(fake.take_sent(), vec![Command::Stop(p)]);
    assert!(sent_viewport_command(&h, &ViewportCommand::Close));
    assert!(h.state().restart_flag().load(Ordering::Acquire));
}

#[test]
fn cancelling_the_guard_keeps_running() {
    let (mut h, fake) = harness(playing(1));
    set_rate(&mut h, &fake, 44_100);
    h.get_by_label(PILL).click();
    h.run_steps(2);
    h.get_by_label("Cancel").click();
    h.run_steps(2);
    assert!(h.query_by_label("Audio is on air").is_none());
    assert!(!h.state().restart_flag().load(Ordering::Acquire));
    assert!(!sent_viewport_command(&h, &ViewportCommand::Close));
    assert!(fake.take_sent().is_empty());
}

#[test]
fn restart_now_applies_a_remote_draft_first() {
    let cell = Arc::new(ArcSwap::from_pointee(fp_remote::RemoteStatus::default()));
    let (mut h, fake) = harness_from(state(1, 0), move |ui| ui.with_remote_status(cell));
    set_rate(&mut h, &fake, 44_100);
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Remote").click();
    h.run_steps(2);
    h.get_all_by_role_and_label(Role::TextInput, "Address")
        .next()
        .unwrap()
        .focus();
    h.run_steps(2);
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    h.run_steps(1);
    for c in "10.0.0.9".chars() {
        h.get_all_by_role_and_label(Role::TextInput, "Address")
            .next()
            .unwrap()
            .type_text(&c.to_string());
        h.run_steps(1);
    }
    // The field still has focus: the address is only a draft.
    assert_ne!(fake.state.load().config.remote.http.bind, "10.0.0.9");
    // An accessibility click presses no pointer, so the field keeps its
    // focus and its own commit never runs: only the flush saves the draft
    // before the window closes.
    h.get_by_label("Restart now").click_accesskit();
    h.step();
    assert!(h.state().restart_flag().load(Ordering::Acquire));
    assert_eq!(fake.state.load().config.remote.http.bind, "10.0.0.9");
}
