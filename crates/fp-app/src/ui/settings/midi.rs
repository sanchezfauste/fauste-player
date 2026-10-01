//! Settings > MIDI (feedback spec §6.4): the switch, the ports and, per
//! player, each action's control with Learn and Clear. Everything goes
//! through the MIDI service's handle: nothing here touches a port.

use egui::{Key, RichText, Ui, vec2};
use fp_control::service::{MidiHandle, MidiRequest};
use fp_model::{MidiAction, MidiTrigger, ShortcutAction};

use super::super::app::Scene;
use super::super::theme;
use super::super::widgets::{self, TileStyle, font};
use super::{SettingsState, heading, update};

/// The actions a player offers to MIDI, with their names.
fn actions(n: u16) -> [(MidiAction, &'static str); 8] {
    use ShortcutAction as A;
    let b = MidiAction::Button;
    [
        (b(A::PlayPlayer(n)), "midi-action-play"),
        (b(A::PausePlayer(n)), "midi-action-pause"),
        (b(A::StopPlayer(n)), "midi-action-stop"),
        (b(A::FadeStopPlayer(n)), "midi-action-fade-stop"),
        (b(A::RestartPlayer(n)), "midi-action-restart"),
        (b(A::PreviousPlayer(n)), "midi-action-previous"),
        (b(A::CuePlayer(n)), "midi-action-cue"),
        (MidiAction::Volume(n), "midi-action-volume"),
    ]
}

fn trigger_text(scene: &Scene<'_>, trigger: MidiTrigger) -> String {
    let t = scene.i18n;
    match trigger {
        MidiTrigger::Note { channel, note } => t.tr_args(
            "midi-note",
            &[
                ("n", u16::from(note).into()),
                ("ch", (u16::from(channel) + 1).into()),
            ],
        ),
        MidiTrigger::ControlChange {
            channel,
            controller,
        } => t.tr_args(
            "midi-cc",
            &[
                ("n", u16::from(controller).into()),
                ("ch", (u16::from(channel) + 1).into()),
            ],
        ),
        MidiTrigger::PitchBend { channel } => t.tr_args(
            "midi-pitch-bend",
            &[("ch", (u16::from(channel) + 1).into())],
        ),
    }
}

fn button(ui: &mut Ui, text: &str) -> bool {
    let w = ui
        .painter()
        .layout_no_wrap(text.to_owned(), font(12.0), theme::TEXT)
        .size()
        .x
        + 20.0;
    widgets::tile(
        ui,
        vec2(w, 24.0),
        text,
        true,
        TileStyle::plain(),
        |p, r, c| {
            p.text(r.center(), egui::Align2::CENTER_CENTER, text, font(12.0), c);
        },
    )
    .clicked()
}

fn text(ui: &mut Ui, value: String, color: egui::Color32) {
    ui.add(egui::Label::new(RichText::new(value).font(font(12.0)).color(color)).selectable(false));
}

pub(super) fn section(
    ui: &mut Ui,
    scene: &Scene<'_>,
    st: &mut SettingsState,
    midi: Option<&MidiHandle>,
) {
    let t = scene.i18n;
    heading(ui, &t.tr("settings-tab-midi"));
    let Some(midi) = midi else {
        text(ui, t.tr("midi-unavailable"), theme::NEUTRAL_400);
        return;
    };
    // What the service learned since the last frame.
    while let Ok(learned) = midi.learned.try_recv() {
        update(scene, |c| {
            c.midi
                .bind(learned.action, &learned.device, learned.trigger);
        });
        if st.midi_learning == Some(learned.action) {
            st.midi_learning = None;
        }
    }
    if st.midi_learning.is_some() && ui.input(|i| i.key_pressed(Key::Escape)) {
        st.midi_learning = None;
        let _ = midi.requests.send(MidiRequest::CancelLearn);
    }
    let config = &scene.state.config.midi;
    let mut enabled = config.enabled;
    if ui.checkbox(&mut enabled, t.tr("midi-enabled")).changed() {
        update(scene, |c| c.midi.enabled = enabled);
    }
    let mut feedback = config.feedback;
    if ui.checkbox(&mut feedback, t.tr("midi-feedback")).changed() {
        update(scene, |c| c.midi.feedback = feedback);
    }
    ui.add_space(8.0);
    text(ui, t.tr("midi-ports"), theme::NEUTRAL_300);
    let status = midi.status.load();
    if status.inputs.is_empty() {
        text(ui, t.tr("midi-no-ports"), theme::NEUTRAL_500);
    }
    for (name, connected) in &status.inputs {
        let state = if *connected {
            t.tr("midi-connected")
        } else {
            t.tr("midi-missing")
        };
        text(ui, format!("{name} — {state}"), theme::NEUTRAL_400);
    }
    let players = scene.state.players.len() as u16;
    for n in 1..=players {
        ui.add_space(12.0);
        text(
            ui,
            t.tr_args("midi-player", &[("n", n.into())]),
            theme::TEXT,
        );
        for (action, key) in actions(n) {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = vec2(8.0, 0.0);
                ui.allocate_ui(vec2(120.0, 24.0), |ui| {
                    text(ui, t.tr(key), theme::NEUTRAL_300);
                });
                let bound = config
                    .binding(action)
                    .map(|b| format!("{} · {}", b.device, trigger_text(scene, b.trigger)))
                    .unwrap_or_else(|| t.tr("midi-unbound"));
                ui.allocate_ui(vec2(260.0, 24.0), |ui| {
                    text(ui, bound, theme::NEUTRAL_400);
                });
                let learning = st.midi_learning == Some(action);
                let label = if learning {
                    t.tr("midi-learning")
                } else {
                    t.tr("midi-learn")
                };
                if button(ui, &label) {
                    if learning {
                        st.midi_learning = None;
                        let _ = midi.requests.send(MidiRequest::CancelLearn);
                    } else {
                        st.midi_learning = Some(action);
                        let _ = midi.requests.send(MidiRequest::Learn(action));
                    }
                }
                if config.binding(action).is_some() && button(ui, &t.tr("midi-clear")) {
                    update(scene, |c| c.midi.unbind(action));
                }
            });
        }
    }
    if st.midi_learning.is_some() {
        // Answers arrive from the service's thread.
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(50));
    }
}
