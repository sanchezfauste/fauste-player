//! Bindings turn MIDI messages into commands (feedback spec §6.3).

use std::collections::HashMap;

use fp_model::{
    AppState, Command, MidiAction, MidiBinding, MidiConfig, MidiTrigger, command_available,
    player_command,
};

use crate::message::MidiMessage;

/// A Control Change button fires when its value rises through this.
const CC_PRESSED: u8 = 64;
/// Half a 7-bit step of fader travel: closer than this is "at" a position.
const PICKUP_TOLERANCE: f32 = 0.5 / 127.0;

/// Soft takeover of one fader binding: it moves the volume only once the
/// physical control has reached or crossed where the volume is.
#[derive(Debug, Default, Clone, Copy)]
struct Pickup {
    picked: bool,
    /// The control's last position, in fader travel.
    last_input: Option<f32>,
    /// The travel this fader last set.
    last_set: Option<f32>,
    /// The model's travel at the previous message.
    model_seen: Option<f32>,
}

/// Routes messages from the surfaces to commands, remembering what edge
/// detection and soft takeover need between messages.
#[derive(Debug, Default)]
pub struct Router {
    bindings: Vec<MidiBinding>,
    /// Last value per (device, channel, controller).
    cc_last: HashMap<(String, u8, u8), u8>,
    /// Soft takeover per fader (device and control).
    pickups: HashMap<(String, MidiTrigger), Pickup>,
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
        let mut position = None;
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
                // The first value heard is the control's state, not a press
                // (a latched control reports it on connection: rule 10).
                let last = self
                    .cc_last
                    .insert((device.to_owned(), channel, controller), value)
                    .unwrap_or(value);
                (
                    MidiTrigger::ControlChange {
                        channel,
                        controller,
                    },
                    {
                        position = Some(f32::from(value) / 127.0);
                        last < CC_PRESSED && value >= CC_PRESSED
                    },
                )
            }
            MidiMessage::PitchBend { channel, value } => {
                position = Some(f32::from(value) / 16_383.0);
                (MidiTrigger::PitchBend { channel }, false)
            }
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
            MidiAction::Volume(n) => {
                let position = position?;
                let player = state.players.get(usize::from(n).checked_sub(1)?)?;
                let travel = fp_model::volume::fader_from_gain(player.volume);
                let pickup = self
                    .pickups
                    .entry((device.to_owned(), trigger))
                    .or_default();
                // Moved by other means since this fader last set it: pick
                // it up again first. A model that has not caught up with this
                // fader's own messages yet (unchanged since the last one) is
                // not a move by other means.
                let model_moved = pickup
                    .model_seen
                    .is_some_and(|seen| (seen - travel).abs() > PICKUP_TOLERANCE);
                pickup.model_seen = Some(travel);
                if model_moved
                    && pickup
                        .last_set
                        .is_some_and(|set| (set - travel).abs() > PICKUP_TOLERANCE)
                {
                    pickup.picked = false;
                    pickup.last_set = None;
                }
                if !pickup.picked {
                    let crossed = pickup.last_input.is_some_and(|last| {
                        (last - travel).signum() != (position - travel).signum()
                    });
                    pickup.picked = crossed || (position - travel).abs() <= PICKUP_TOLERANCE;
                }
                pickup.last_input = Some(position);
                if !pickup.picked {
                    return None;
                }
                pickup.last_set = Some(position);
                Some(Command::SetVolume(
                    player.id,
                    fp_model::volume::gain_from_fader(position),
                ))
            }
        }
    }
}
