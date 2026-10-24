#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback spec §6.1: parsing never panics; unknown messages are ignored.

use fp_control::message::{MidiMessage, parse};

#[test]
fn note_on_off_cc_and_pitch_bend_are_read() {
    assert_eq!(
        parse(&[0x92, 36, 100]),
        Some(MidiMessage::NoteOn {
            channel: 2,
            note: 36,
            velocity: 100
        })
    );
    assert_eq!(
        parse(&[0x80, 36, 64]),
        Some(MidiMessage::NoteOff {
            channel: 0,
            note: 36
        })
    );
    assert_eq!(
        parse(&[0x9F, 36, 0]),
        Some(MidiMessage::NoteOff {
            channel: 15,
            note: 36
        }),
        "velocity 0 is a note off"
    );
    assert_eq!(
        parse(&[0xB0, 7, 127]),
        Some(MidiMessage::ControlChange {
            channel: 0,
            controller: 7,
            value: 127
        })
    );
    // 14 bits, least significant byte first.
    assert_eq!(
        parse(&[0xE1, 0x7F, 0x7F]),
        Some(MidiMessage::PitchBend {
            channel: 1,
            value: 16_383
        })
    );
    assert_eq!(
        parse(&[0xE1, 0x00, 0x40]),
        Some(MidiMessage::PitchBend {
            channel: 1,
            value: 8_192
        })
    );
}

#[test]
fn anything_else_is_ignored() {
    for bytes in [
        &[][..],
        &[0xF8][..],             // clock
        &[0xFE][..],             // active sensing
        &[0xF0, 0x7E, 0xF7][..], // SysEx
        &[0xC0, 5][..],          // program change
        &[0xD0, 5][..],          // channel pressure
        &[0xA0, 5, 5][..],       // poly pressure
        &[36, 100][..],          // running status: no status byte
        &[0x90, 0x90, 100][..],  // status where data should be
        &[0x90, 36, 0x80][..],
    ] {
        assert_eq!(parse(bytes), None, "{bytes:?}");
    }
}

#[test]
fn every_truncated_message_is_ignored() {
    for full in [&[0x90u8, 36, 100][..], &[0xB0, 7, 1][..], &[0xE0, 1, 2][..]] {
        for n in 0..full.len() {
            assert_eq!(parse(&full[..n]), None, "{:?}", &full[..n]);
        }
    }
}

#[test]
fn alsa_port_numbers_are_not_part_of_the_name() {
    use fp_control::ports::stable_name;
    assert_eq!(
        stable_name("APC MINI:APC MINI MIDI 1 20:0"),
        "APC MINI:APC MINI MIDI 1"
    );
    assert_eq!(stable_name("APC mini"), "APC mini");
    assert_eq!(stable_name("Port 2"), "Port 2");
}
