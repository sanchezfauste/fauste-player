//! LED feedback (feedback spec §6.3): each bound button shows its player's
//! state on the surface.

use std::collections::HashMap;

use fp_model::{
    AppState, MidiAction, MidiConfig, MidiTrigger, ShortcutAction, Transport, availability,
};

/// Velocity (Note) or value (CC) of a lit button; 0 is unlit.
const LIT: u8 = 127;

/// One LED message: the output port and the three MIDI bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Led {
    pub output: String,
    pub bytes: [u8; 3],
}

/// What every bound button should show now. `blink_on` is the phase of the
/// 500 ms blink (Pause while paused). Empty when feedback is off.
pub fn desired(config: &MidiConfig, state: &AppState, blink_on: bool) -> Vec<Led> {
    if !config.feedback {
        return Vec::new();
    }
    config
        .bindings
        .iter()
        .filter_map(|b| {
            let MidiAction::Button(action) = b.action else {
                return None;
            };
            let n = action.position()?;
            let player = state.players.get(usize::from(n).checked_sub(1)?)?;
            let can = availability(state, player.id);
            let on = match action {
                ShortcutAction::PlayPlayer(_) => player.transport == Transport::Playing,
                ShortcutAction::PausePlayer(_) => player.transport == Transport::Paused && blink_on,
                ShortcutAction::CuePlayer(_) => player.cue.is_some(),
                ShortcutAction::StopPlayer(_) => can.stop,
                ShortcutAction::FadeStopPlayer(_) => can.fade_stop,
                ShortcutAction::RestartPlayer(_) => can.restart,
                ShortcutAction::PreviousPlayer(_) => can.previous,
                _ => return None,
            };
            let value = if on { LIT } else { 0 };
            let bytes = match b.trigger {
                MidiTrigger::Note { channel, note } => [0x90 | (channel & 0x0F), note, value],
                MidiTrigger::ControlChange {
                    channel,
                    controller,
                } => [0xB0 | (channel & 0x0F), controller, value],
                MidiTrigger::PitchBend { .. } => return None,
            };
            let output = config
                .devices
                .iter()
                .find(|d| d.input == b.device)
                .and_then(|d| d.output.clone())
                .unwrap_or_else(|| b.device.clone());
            Some(Led { output, bytes })
        })
        .collect()
}

/// What was last sent, so only changes go out.
#[derive(Debug, Default)]
pub struct Feedback {
    sent: HashMap<(String, [u8; 2]), u8>,
}

impl Feedback {
    /// The LEDs of `desired` that differ from what was last sent.
    pub fn changes(&mut self, desired: Vec<Led>) -> Vec<Led> {
        desired
            .into_iter()
            .filter(|led| {
                let [status, number, value] = led.bytes;
                self.sent
                    .insert((led.output.clone(), [status, number]), value)
                    != Some(value)
            })
            .collect()
    }

    /// Forgets what was sent (after a reconnection): everything goes again.
    pub fn reset(&mut self) {
        self.sent.clear();
    }
}
