#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback spec §6.4: MIDI learn binds the next matching message.

use fp_control::learn::trigger_for;
use fp_control::message::MidiMessage;
use fp_model::{MidiAction, MidiTrigger, ShortcutAction};

const PLAY: MidiAction = MidiAction::Button(ShortcutAction::PlayPlayer(1));
const VOLUME: MidiAction = MidiAction::Volume(1);

#[test]
fn a_button_learns_a_pressed_note_or_control() {
    let on = MidiMessage::NoteOn {
        channel: 2,
        note: 36,
        velocity: 90,
    };
    assert_eq!(
        trigger_for(PLAY, on),
        Some(MidiTrigger::Note {
            channel: 2,
            note: 36
        })
    );
    let cc = MidiMessage::ControlChange {
        channel: 0,
        controller: 20,
        value: 127,
    };
    assert_eq!(
        trigger_for(PLAY, cc),
        Some(MidiTrigger::ControlChange {
            channel: 0,
            controller: 20
        })
    );
    // A release or a fader resting low is not a press.
    let low = MidiMessage::ControlChange {
        channel: 0,
        controller: 20,
        value: 10,
    };
    assert_eq!(trigger_for(PLAY, low), None);
    assert_eq!(
        trigger_for(
            PLAY,
            MidiMessage::NoteOff {
                channel: 0,
                note: 36
            }
        ),
        None
    );
    assert_eq!(
        trigger_for(
            PLAY,
            MidiMessage::PitchBend {
                channel: 0,
                value: 9000
            }
        ),
        None
    );
}

#[test]
fn a_volume_learns_a_control_change_or_pitch_bend() {
    let cc = MidiMessage::ControlChange {
        channel: 3,
        controller: 7,
        value: 10,
    };
    assert_eq!(
        trigger_for(VOLUME, cc),
        Some(MidiTrigger::ControlChange {
            channel: 3,
            controller: 7
        })
    );
    let pb = MidiMessage::PitchBend {
        channel: 1,
        value: 0,
    };
    assert_eq!(
        trigger_for(VOLUME, pb),
        Some(MidiTrigger::PitchBend { channel: 1 })
    );
    let on = MidiMessage::NoteOn {
        channel: 0,
        note: 1,
        velocity: 100,
    };
    assert_eq!(trigger_for(VOLUME, on), None);
}
