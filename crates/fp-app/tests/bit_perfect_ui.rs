#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Bit-perfect rate and fader spec: the locked fader (BF7), the cues that a
//! bit-perfect player does not mix (BR15).

mod support;

use egui_kittest::kittest::Queryable;
use fp_app::ui::controller::Controller;
use fp_model::Command;
use support::{harness, lock_player, state};

const LOCKED: &str = "Volume held at 100 %: this player's output is bit-perfect.";

fn locked(players: usize) -> fp_model::AppState {
    let mut s = state(players, 3);
    lock_player(&mut s, 0);
    assert!(fp_model::fader_locked(&s, s.players[0].id));
    s
}

fn volume_commands(sent: &[Command]) -> usize {
    sent.iter()
        .filter(|c| matches!(c, Command::SetVolume(..)))
        .count()
}

fn wheel_down(h: &mut egui_kittest::Harness<'static, fp_app::ui::app::AppUi>) {
    h.event(egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Line,
        delta: egui::vec2(0.0, -1.0),
        phase: egui::TouchPhase::Move,
        modifiers: egui::Modifiers::NONE,
    });
}

#[test]
fn bf7_a_locked_fader_takes_no_drag_and_no_wheel() {
    let (mut h, fake) = harness(locked(1));
    h.run_steps(2);
    let at = h.get_by_label(LOCKED).rect().center();
    h.hover_at(at);
    h.step();
    wheel_down(&mut h);
    h.run_steps(5);
    h.event(egui::Event::PointerButton {
        pos: at,
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::NONE,
    });
    h.step();
    let lower = at + egui::vec2(0.0, 40.0);
    h.event(egui::Event::PointerMoved(lower));
    h.step();
    h.event(egui::Event::PointerButton {
        pos: lower,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::NONE,
    });
    h.run_steps(5);
    assert_eq!(volume_commands(&fake.take_sent()), 0);
    assert_eq!(fake.state.load().players[0].volume, 1.0);
    assert!(
        fake.rejected.lock().unwrap().is_empty(),
        "nothing was refused: nothing was sent"
    );
}

#[test]
fn bf7_a_locked_fader_is_drawn_at_the_top() {
    let mut s = locked(1);
    // A frame where the state has not caught up with the lock yet.
    s.players[0].volume = 0.5;
    let (mut h, _fake) = harness(s);
    h.run_steps(2);
    // Pixels cannot be read here: the fader is the locked widget, with the
    // same geometry as an unlocked one, and `meter_column` paints it at
    // travel 1.0 whatever the volume says.
    let locked_rect = h.get_by_label(LOCKED).rect();
    let (mut h2, _) = harness(state(1, 3));
    h2.run_steps(2);
    let free_rect = h2.get_by_label_contains("Volume").rect();
    assert_eq!(locked_rect.size(), free_rect.size());
}

#[test]
fn bf7_an_unlocked_fader_still_moves() {
    let (mut h, fake) = harness(state(1, 3));
    h.run_steps(2);
    let at = h.get_by_label_contains("Volume").rect().center();
    h.hover_at(at);
    h.step();
    wheel_down(&mut h);
    h.run_steps(5);
    assert_eq!(volume_commands(&fake.take_sent()), 1);
}

#[test]
fn br15_the_next_tooltip_says_there_is_no_crossfade() {
    let (mut h, fake) = harness(locked(1));
    h.run_steps(2);
    assert!(
        h.query_all_by_label("Play").next().is_some(),
        "stopped: the usual Play"
    );
    assert!(
        h.query_by_label_contains("without a crossfade").is_none(),
        "only on air"
    );
    fake.send(Command::Play(fake.player(0)));
    h.run_steps(2);
    assert!(
        h.query_by_label("Next track, at once and without a crossfade (bit-perfect output)")
            .is_some()
    );
    assert!(h.query_by_label("Fade into the next track").is_none());
}

const NOTE: &str = "Players on a bit-perfect output never mix or crossfade";

fn open_players(h: &mut egui_kittest::Harness<'static, fp_app::ui::app::AppUi>) {
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(egui::accesskit::Role::Button, "Players")
        .click();
    h.run_steps(2);
}

#[test]
fn br15_the_players_note_shows_only_while_a_bit_perfect_player_exists() {
    let (mut h, _) = harness(state(2, 3));
    open_players(&mut h);
    assert!(h.query_by_label_contains(NOTE).is_none());
    let (mut h, fake) = harness(locked(2));
    open_players(&mut h);
    assert!(
        h.query_by_label_contains(NOTE).is_some(),
        "P1 is idle but bit-perfect"
    );
    // The settings it explains stay editable.
    assert!(
        h.query_by_role_and_label(egui::accesskit::Role::Slider, "Fade time")
            .is_some()
    );
    // Bit-perfect turned off for the device.
    let mut s = (**fake.state.load()).clone();
    let dac = fp_model::OutputDevice {
        backend: "null".into(),
        device: "dac".into(),
    };
    s.live.devices.get_mut(&dac).unwrap().bit_perfect = false;
    fake.state.store(std::sync::Arc::new(s));
    h.run_steps(2);
    assert!(h.query_by_label_contains(NOTE).is_none());
}

#[test]
fn bf9_the_bit_perfect_hint_says_what_it_costs() {
    let i18n = fp_app::i18n::I18n::new(Some("en-US"));
    let hint = i18n.tr("settings-bit-perfect-hint");
    for phrase in [
        "A special mode",
        "never crossfade or mix",
        "short silence",
        "held at 100 %",
    ] {
        assert!(hint.contains(phrase), "{phrase:?} missing from {hint:?}");
    }
}
