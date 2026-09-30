//! Bindings turn MIDI messages into commands (feedback spec §6.3).

use std::collections::HashMap;

use fp_model::{
    AppState, Command, MidiAction, MidiBinding, MidiConfig, MidiTrigger, command_available,
    player_command,
};

use crate::message::MidiMessage;

/// A Control Change button fires when its value rises through this.
const CC_PRESSED: u8 = 64;

/// Routes messages from the surfaces to commands, remembering what edge
/// detection and soft takeover need between messages.
#[derive(Debug, Default)]
pub struct Router {
    bindings: Vec<MidiBinding>,
    /// Last value per (device, channel, controller).
    cc_last: HashMap<(String, u8, u8), u8>,
}

impl Router {
    pub fn new(config: &MidiConfig) -> Self {
        Self {
            bindings: config.bindings.clone(),
            ..Self::default()
        }
    }

    /// Follows a configuration change, keeping what it knows about controls.
    pub fn set_config(&mut self, config: &MidiConfig) {
        if self.bindings != config.bindings {
            self.bindings = config.bindings.clone();
        }
    }

    /// The command `msg` from `device` stands for, if any: a bound button
    /// that is pressed and whose action makes sense now (R28), or a bound
    /// fader that has picked up the volume.
    pub fn on_message(
        &mut self,
        device: &str,
        msg: MidiMessage,
        state: &AppState,
    ) -> Option<Command> {
        let (trigger, pressed) = match msg {
            MidiMessage::NoteOn { channel, note, .. } => {
                (MidiTrigger::Note { channel, note }, true)
            }
            MidiMessage::NoteOff { .. } => return None,
            MidiMessage::ControlChange {
                channel,
                controller,
                value,
            } => {
                let last = self
                    .cc_last
                    .insert((device.to_owned(), channel, controller), value)
                    .unwrap_or(0);
                (
                    MidiTrigger::ControlChange {
                        channel,
                        controller,
                    },
                    last < CC_PRESSED && value >= CC_PRESSED,
                )
            }
            MidiMessage::PitchBend { channel, .. } => (MidiTrigger::PitchBend { channel }, false),
        };
        let binding = self
            .bindings
            .iter()
            .find(|b| b.device == device && b.trigger == trigger)?;
        match binding.action {
            MidiAction::Button(action) => {
                if !pressed {
                    return None;
                }
                let command = player_command(state, action)?;
                command_available(state, &command).then_some(command)
            }
            MidiAction::Volume(_) => None,
        }
    }
}
