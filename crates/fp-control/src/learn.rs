//! MIDI learn (feedback spec §6.4): which control a message offers for an
//! action.

use fp_model::{MidiAction, MidiTrigger};

use crate::message::MidiMessage;

/// The control `msg` would bind to `action`, if it suits it: a button
/// takes a pressed Note or a Control Change at 64 or above; a volume takes
/// a Control Change or a Pitch Bend.
pub fn trigger_for(action: MidiAction, msg: MidiMessage) -> Option<MidiTrigger> {
    match (action, msg) {
        (MidiAction::Button(_), MidiMessage::NoteOn { channel, note, .. }) => {
            Some(MidiTrigger::Note { channel, note })
        }
        (
            MidiAction::Button(_),
            MidiMessage::ControlChange {
                channel,
                controller,
                value,
            },
        ) if value >= 64 => Some(MidiTrigger::ControlChange {
            channel,
            controller,
        }),
        (
            MidiAction::Volume(_),
            MidiMessage::ControlChange {
                channel,
                controller,
                ..
            },
        ) => Some(MidiTrigger::ControlChange {
            channel,
            controller,
        }),
        (MidiAction::Volume(_), MidiMessage::PitchBend { channel, .. }) => {
            Some(MidiTrigger::PitchBend { channel })
        }
        _ => None,
    }
}
