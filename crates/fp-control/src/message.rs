//! MIDI channel-voice messages the surfaces send.

/// A message a control surface sends. Channels are 0-based (0–15).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MidiMessage {
    NoteOn {
        channel: u8,
        note: u8,
        velocity: u8,
    },
    NoteOff {
        channel: u8,
        note: u8,
    },
    ControlChange {
        channel: u8,
        controller: u8,
        value: u8,
    },
    /// 14 bits, 8192 at the centre.
    PitchBend {
        channel: u8,
        value: u16,
    },
}

/// The message in `bytes`, one complete message as the port delivers it.
/// Anything else (system messages, SysEx, other channel messages, running
/// status, truncated or malformed bytes) is `None`: never a panic.
pub fn parse(bytes: &[u8]) -> Option<MidiMessage> {
    let (&status, data) = bytes.split_first()?;
    if status < 0x80 {
        return None;
    }
    let channel = status & 0x0F;
    let two = || match data {
        [a, b, ..] if *a < 0x80 && *b < 0x80 => Some((*a, *b)),
        _ => None,
    };
    match status & 0xF0 {
        0x90 => {
            let (note, velocity) = two()?;
            Some(if velocity == 0 {
                MidiMessage::NoteOff { channel, note }
            } else {
                MidiMessage::NoteOn {
                    channel,
                    note,
                    velocity,
                }
            })
        }
        0x80 => two().map(|(note, _)| MidiMessage::NoteOff { channel, note }),
        0xB0 => two().map(|(controller, value)| MidiMessage::ControlChange {
            channel,
            controller,
            value,
        }),
        0xE0 => two().map(|(lsb, msb)| MidiMessage::PitchBend {
            channel,
            value: u16::from(msb) << 7 | u16::from(lsb),
        }),
        _ => None,
    }
}
