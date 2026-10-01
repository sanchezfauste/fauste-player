#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Settings > MIDI (feedback spec §6.4).

mod support;

use std::sync::Arc;

use arc_swap::ArcSwap;
use egui::Key;
use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::AppUi;
use fp_control::service::{Learned, MidiHandle, MidiRequest, MidiStatus};
use fp_model::{MidiAction, MidiTrigger, ShortcutAction};
use support::{harness_from, state};

struct Surface {
    requests: crossbeam_channel::Receiver<MidiRequest>,
    answers: crossbeam_channel::Sender<Learned>,
}

fn with_midi() -> (Harness<'static, AppUi>, Arc<support::Fake>, Surface) {
    let (req_tx, req_rx) = crossbeam_channel::unbounded();
    let (learned_tx, learned_rx) = crossbeam_channel::unbounded();
    let handle = MidiHandle {
        status: Arc::new(ArcSwap::from_pointee(MidiStatus {
            inputs: vec![("APC mini".into(), true), ("Keys".into(), false)],
        })),
        requests: req_tx,
        learned: learned_rx,
    };
    let (mut h, fake) = harness_from(state(1, 0), move |ui| ui.with_midi(handle));
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "MIDI").click();
    h.run_steps(2);
    (
        h,
        fake,
        Surface {
            requests: req_rx,
            answers: learned_tx,
        },
    )
}

#[test]
fn the_section_lists_the_ports_and_their_state() {
    let (h, _, _) = with_midi();
    assert!(h.query_by_label_contains("APC mini").is_some());
    assert!(h.query_by_label_contains("not connected").is_some());
}

#[test]
fn learn_stores_the_control_that_answers() {
    let (mut h, fake, surface) = with_midi();
    h.get_all_by_label("Learn").next().unwrap().click();
    h.run_steps(2);
    let play = MidiAction::Button(ShortcutAction::PlayPlayer(1));
    assert_eq!(
        surface.requests.try_recv().unwrap(),
        MidiRequest::Learn(play)
    );
    assert!(h.query_by_label("Move a control…").is_some());
    let trigger = MidiTrigger::Note {
        channel: 0,
        note: 36,
    };
    surface
        .answers
        .send(Learned {
            action: play,
            device: "APC mini".into(),
            trigger,
        })
        .unwrap();
    h.run_steps(3);
    let config = fake.state.load().config.midi.clone();
    assert_eq!(config.binding(play).unwrap().trigger, trigger);
    assert!(h.query_by_label_contains("Note 36, channel 1").is_some());
}

#[test]
fn clear_removes_a_binding() {
    let (mut h, fake, surface) = with_midi();
    let play = MidiAction::Button(ShortcutAction::PlayPlayer(1));
    h.get_all_by_label("Learn").next().unwrap().click();
    h.run_steps(2);
    let trigger = MidiTrigger::Note {
        channel: 0,
        note: 36,
    };
    surface
        .answers
        .send(Learned {
            action: play,
            device: "APC mini".into(),
            trigger,
        })
        .unwrap();
    h.run_steps(3);
    h.get_all_by_label("Clear").next().unwrap().click();
    h.run_steps(2);
    assert!(fake.state.load().config.midi.binding(play).is_none());
}

#[test]
fn escape_cancels_learning_without_closing_settings() {
    let (mut h, _, surface) = with_midi();
    h.get_all_by_label("Learn").next().unwrap().click();
    h.run_steps(2);
    let _ = surface.requests.try_recv();
    h.key_press(Key::Escape);
    h.run_steps(2);
    assert_eq!(
        surface.requests.try_recv().unwrap(),
        MidiRequest::CancelLearn
    );
    assert!(
        h.query_by_role_and_label(Role::Button, "MIDI").is_some(),
        "Settings stays open"
    );
    assert!(h.query_by_label("Move a control…").is_none());
}

#[test]
fn turning_midi_on_updates_the_config() {
    let (mut h, fake, _) = with_midi();
    h.get_by_label("Use MIDI control surfaces").click();
    h.run_steps(2);
    assert!(fake.state.load().config.midi.enabled);
}

#[test]
fn closing_settings_while_learning_cancels_it() {
    let (mut h, _, surface) = with_midi();
    h.get_all_by_label("Learn").next().unwrap().click();
    h.run_steps(2);
    let _ = surface.requests.try_recv();
    h.get_by_label("Close").click();
    h.run_steps(3);
    assert_eq!(
        surface.requests.try_recv().unwrap(),
        MidiRequest::CancelLearn
    );
}
