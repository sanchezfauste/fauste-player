#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Live settings spec §9: the Settings pending pill and panel, the
//! Settings footer, Apply now and its confirmation, the status bar label.

mod support;

use egui::Key;
use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::AppUi;
use fp_app::ui::controller::Controller;
use fp_model::{
    AppState, Command, EngineEvent, Holder, OutputDevice, Route, Target, Wanted, device_settings,
};
use support::{Fake, harness, state};

const PILL: &str = "Settings pending";
const NOTICE: &str = "Some changes wait until the outputs they affect are free.";
const CONFIRM: &str = "Apply now?";

fn null(device: &str) -> OutputDevice {
    OutputDevice {
        backend: "null".into(),
        device: device.into(),
    }
}

/// `state(players, 3)` as the engine reports it at start: every Main on
/// the null output, every Cue on `phones`.
fn reported(players: usize) -> AppState {
    let mut s = state(players, 3);
    fp_model::on_event(
        &mut s,
        EngineEvent::AudioSystemInUse {
            configured: None,
            in_use: "null".into(),
        },
    );
    for p in s.players.clone() {
        let main = null("null");
        let running = device_settings(&s.config.outputs, &main);
        fp_model::on_event(
            &mut s,
            EngineEvent::Placed {
                holder: Holder::PlayerMain(p.id),
                route: None,
                device: main,
                running,
            },
        );
        let phones = null("phones");
        let running = device_settings(&s.config.outputs, &phones);
        let route = Some(Route {
            backend: "null".into(),
            device: "phones".into(),
            first_channel: 0,
        });
        fp_model::on_event(
            &mut s,
            EngineEvent::Placed {
                holder: Holder::PlayerCue(p.id),
                route,
                device: phones,
                running,
            },
        );
    }
    s
}

fn playing(players: usize) -> AppState {
    let mut s = reported(players);
    let p = s.players[0].id;
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    s
}

/// A new global buffer, through the model as Settings sends it.
fn set_buffer(h: &mut Harness<'static, AppUi>, fake: &Fake, frames: u32) {
    let mut config = fake.state.load().config.clone();
    config.outputs.buffer_frames = frames;
    fake.send(Command::UpdateConfig(Box::new(config)));
    fake.take_sent();
    h.run_steps(2);
}

fn open_settings_and_apply(h: &mut Harness<'static, AppUi>) {
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_label("Apply now").click();
    h.run_steps(2);
}

#[test]
fn nothing_waiting_shows_no_pill_and_settings_applies_at_once() {
    let (mut h, _fake) = harness(reported(1));
    assert!(h.query_by_label(PILL).is_none());
    h.get_by_label("Settings").click();
    h.run_steps(2);
    assert!(h.query_by_label(NOTICE).is_none());
    assert!(h.query_by_label("Apply now").is_none());
}

#[test]
fn a_waiting_change_shows_the_pill_and_the_panel_lists_it_with_its_cause() {
    let (mut h, fake) = harness(playing(1));
    set_buffer(&mut h, &fake, 1024);
    assert!(h.query_by_role_and_label(Role::Button, PILL).is_some());
    h.get_by_label(PILL).click();
    h.run_steps(2);
    assert!(
        h.query_by_label("null: buffer 512 → 1024 — waiting for P1 playing")
            .is_some()
    );
}

#[test]
fn apply_now_with_something_on_air_asks_first_and_cancel_sends_nothing() {
    let (mut h, fake) = harness(playing(1));
    set_buffer(&mut h, &fake, 1024);
    h.get_by_label("Settings").click();
    h.run_steps(2);
    assert!(h.query_by_label(NOTICE).is_some());
    h.get_by_label("Apply now").click();
    h.run_steps(2);
    assert!(h.query_by_label(CONFIRM).is_some());
    assert!(h.query_by_label("null: P1 playing").is_some());
    h.get_by_label("Cancel").click();
    h.run_steps(2);
    assert!(h.query_by_label(CONFIRM).is_none());
    assert!(fake.take_sent().is_empty());
}

#[test]
fn interrupt_and_apply_sends_apply_settings_now() {
    let (mut h, fake) = harness(playing(1));
    set_buffer(&mut h, &fake, 1024);
    open_settings_and_apply(&mut h);
    h.get_by_label("Interrupt and apply").click();
    h.run_steps(2);
    assert_eq!(fake.take_sent(), vec![Command::ApplySettingsNow]);
    assert!(h.query_by_label(CONFIRM).is_none());
}

#[test]
fn apply_now_with_nothing_on_air_applies_at_once() {
    let (mut h, fake) = harness(reported(1));
    set_buffer(&mut h, &fake, 1024);
    open_settings_and_apply(&mut h);
    assert!(h.query_by_label(CONFIRM).is_none());
    assert_eq!(fake.take_sent(), vec![Command::ApplySettingsNow]);
}

#[test]
fn the_confirmation_applies_by_itself_when_what_it_would_cut_ends() {
    let (mut h, fake) = harness(playing(1));
    set_buffer(&mut h, &fake, 1024);
    open_settings_and_apply(&mut h);
    assert!(h.query_by_label(CONFIRM).is_some());
    let p = fake.player(0);
    fake.send(Command::Stop(p));
    fake.take_sent();
    h.run_steps(2);
    assert!(h.query_by_label(CONFIRM).is_none());
    assert_eq!(fake.take_sent(), vec![Command::ApplySettingsNow]);
}

#[test]
fn esc_cancels_the_confirmation_and_keeps_settings_open() {
    let (mut h, fake) = harness(playing(1));
    set_buffer(&mut h, &fake, 1024);
    open_settings_and_apply(&mut h);
    h.key_press(Key::Escape);
    h.run_steps(2);
    assert!(h.query_by_label(CONFIRM).is_none());
    assert!(h.query_by_label(NOTICE).is_some(), "Settings is still open");
    assert!(fake.take_sent().is_empty());
}

#[test]
fn apply_now_in_the_panel_asks_first_and_the_panel_gives_way_to_the_question() {
    let (mut h, fake) = harness(playing(1));
    set_buffer(&mut h, &fake, 1024);
    h.get_by_label(PILL).click();
    h.run_steps(2);
    h.get_by_label("Apply now").click();
    h.run_steps(2);
    assert!(h.query_by_label(CONFIRM).is_some());
    assert!(
        h.query_by_label("null: buffer 512 → 1024 — waiting for P1 playing")
            .is_none(),
        "the panel and the question are separate windows"
    );
    assert!(fake.take_sent().is_empty());
    h.get_by_label("Interrupt and apply").click();
    h.run_steps(2);
    assert_eq!(fake.take_sent(), vec![Command::ApplySettingsNow]);
}

#[test]
fn esc_closes_the_panel() {
    let (mut h, fake) = harness(playing(1));
    set_buffer(&mut h, &fake, 1024);
    h.get_by_label(PILL).click();
    h.run_steps(2);
    h.key_press(Key::Escape);
    h.run_steps(2);
    assert!(
        h.query_by_label("null: buffer 512 → 1024 — waiting for P1 playing")
            .is_none()
    );
    assert!(fake.take_sent().is_empty());
}

#[test]
fn the_panel_closes_by_itself_when_nothing_waits_any_more() {
    let (mut h, fake) = harness(playing(1));
    set_buffer(&mut h, &fake, 1024);
    h.get_by_label(PILL).click();
    h.run_steps(2);
    set_buffer(&mut h, &fake, 512);
    assert!(h.query_by_label(PILL).is_none());
    assert!(h.query_by_label("Close").is_none());
}

#[test]
fn the_status_bar_names_the_audio_system_in_use() {
    let (h, _fake) = harness(reported(1));
    let os = match std::env::consts::OS {
        "linux" => "Linux",
        "windows" => "Windows",
        "macos" => "macOS",
        other => other,
    };
    assert!(
        h.query_by_label(&format!("{os} · No output (silent)"))
            .is_some()
    );
}

#[test]
fn apply_now_in_the_panel_with_nothing_on_air_sends_at_once() {
    let (mut h, fake) = harness(reported(1));
    set_buffer(&mut h, &fake, 1024);
    h.get_by_label(PILL).click();
    h.run_steps(2);
    h.get_by_label("Apply now").click();
    h.run_steps(2);
    assert!(h.query_by_label(CONFIRM).is_none());
    assert_eq!(fake.take_sent(), vec![Command::ApplySettingsNow]);
}

#[test]
fn close_button_closes_the_panel_and_keeps_the_pill() {
    let (mut h, fake) = harness(playing(1));
    set_buffer(&mut h, &fake, 1024);
    h.get_by_label(PILL).click();
    h.run_steps(2);
    h.get_by_label("Close").click();
    h.run_steps(2);
    assert!(
        h.query_by_label("null: buffer 512 → 1024 — waiting for P1 playing")
            .is_none()
    );
    assert!(h.query_by_role_and_label(Role::Button, PILL).is_some());
    assert!(fake.take_sent().is_empty());
}

#[test]
fn clicking_the_backdrop_closes_the_panel() {
    let (mut h, fake) = harness(playing(1));
    set_buffer(&mut h, &fake, 1024);
    h.get_by_label(PILL).click();
    h.run_steps(2);
    // Far from the centred panel.
    h.hover_at(egui::pos2(2.0, 2.0));
    h.event(egui::Event::PointerButton {
        pos: egui::pos2(2.0, 2.0),
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::NONE,
    });
    h.event(egui::Event::PointerButton {
        pos: egui::pos2(2.0, 2.0),
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::NONE,
    });
    h.run_steps(2);
    assert!(
        h.query_by_label("null: buffer 512 → 1024 — waiting for P1 playing")
            .is_none()
    );
    assert!(fake.take_sent().is_empty());
}

#[test]
fn the_panel_shows_what_the_device_refused() {
    let (mut h, fake) = harness(playing(1));
    set_buffer(&mut h, &fake, 1024);
    let device = null("null");
    let mut next = (**fake.state.load()).clone();
    let wanted = Wanted::Device(device_settings(&next.config.outputs, &device));
    fp_model::on_event(
        &mut next,
        EngineEvent::Applied {
            target: Target::Device(device),
            wanted,
            outcome: Err("no such buffer".into()),
        },
    );
    fake.state.store(std::sync::Arc::new(next));
    h.run_steps(2);
    h.get_by_label(PILL).click();
    h.run_steps(2);
    assert!(
        h.query_by_label_contains("did not take").is_some(),
        "the refusal line"
    );
    assert!(h.query_by_label_contains("no such buffer").is_some());
}

#[test]
fn cancel_in_the_confirmation_brings_the_panel_back() {
    let (mut h, fake) = harness(playing(1));
    set_buffer(&mut h, &fake, 1024);
    h.get_by_label(PILL).click();
    h.run_steps(2);
    h.get_by_label("Apply now").click();
    h.run_steps(2);
    assert!(h.query_by_label(CONFIRM).is_some());
    h.get_by_label("Cancel").click();
    h.run_steps(2);
    assert!(h.query_by_label(CONFIRM).is_none());
    assert!(
        h.query_by_label("null: buffer 512 → 1024 — waiting for P1 playing")
            .is_some(),
        "the panel is back"
    );
    assert!(fake.take_sent().is_empty());
}
