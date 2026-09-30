#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback spec §6.2: MIDI configuration.

use fp_model::{Config, MidiAction, MidiBinding, MidiConfig, MidiTrigger, ShortcutAction, volume};

fn note(n: u8) -> MidiTrigger {
    MidiTrigger::Note {
        channel: 0,
        note: n,
    }
}

fn binding(device: &str, trigger: MidiTrigger, action: MidiAction) -> MidiBinding {
    MidiBinding {
        device: device.to_owned(),
        trigger,
        action,
    }
}

#[test]
fn midi_is_off_by_default_with_feedback_and_a_two_second_rescan() {
    let m = Config::default().midi;
    assert!(!m.enabled);
    assert!(m.feedback);
    assert_eq!(m.rescan_interval_ms, 2000);
    assert!(m.bindings.is_empty());
}

#[test]
fn the_rescan_interval_is_clamped() {
    let mut c = Config::default();
    c.midi.rescan_interval_ms = 10;
    assert!(!c.validate().is_empty());
    assert_eq!(c.midi.rescan_interval_ms, 250);
    c.midi.rescan_interval_ms = 1_000_000;
    c.validate();
    assert_eq!(c.midi.rescan_interval_ms, 60_000);
}

#[test]
fn bindings_round_trip_and_invalid_ones_are_dropped_with_a_warning() {
    let mut c = Config::default();
    c.midi.bindings = vec![
        binding(
            "APC",
            note(36),
            MidiAction::Button(ShortcutAction::PlayPlayer(1)),
        ),
        binding(
            "APC",
            MidiTrigger::ControlChange {
                channel: 0,
                controller: 7,
            },
            MidiAction::Volume(1),
        ),
        // Not a per-player transport action.
        binding(
            "APC",
            note(37),
            MidiAction::Button(ShortcutAction::StopAllCarts),
        ),
        // Out of MIDI range.
        binding(
            "APC",
            MidiTrigger::Note {
                channel: 16,
                note: 1,
            },
            MidiAction::Button(ShortcutAction::StopPlayer(1)),
        ),
        // A volume on a note.
        binding("APC", note(40), MidiAction::Volume(1)),
    ];
    let json = serde_json::to_string(&c.midi).unwrap();
    let back: MidiConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back, c.midi);
    let warnings = c.validate();
    assert_eq!(c.midi.bindings.len(), 2, "{:?}", c.midi.bindings);
    assert_eq!(
        warnings
            .iter()
            .filter(|w| w.field == "midi.bindings")
            .count(),
        3
    );
}

#[test]
fn binding_a_trigger_used_elsewhere_moves_it() {
    let mut m = MidiConfig::default();
    let play = MidiAction::Button(ShortcutAction::PlayPlayer(1));
    let stop = MidiAction::Button(ShortcutAction::StopPlayer(1));
    m.bind(play, "APC", note(36));
    m.bind(stop, "APC", note(36));
    assert_eq!(m.bindings, vec![binding("APC", note(36), stop)]);
    // Binding an action again replaces its previous trigger.
    m.bind(stop, "APC", note(38));
    assert_eq!(m.bindings, vec![binding("APC", note(38), stop)]);
    // The same trigger on another device is a different control.
    m.bind(play, "Other", note(38));
    assert_eq!(m.bindings.len(), 2);
    m.unbind(play);
    assert_eq!(m.bindings, vec![binding("APC", note(38), stop)]);
}

#[test]
fn the_fader_curve_is_shared() {
    assert_eq!(volume::gain_from_fader(1.0), 1.0);
    assert_eq!(volume::gain_from_fader(0.0), 0.0);
    let g = volume::gain_from_fader(0.5);
    assert!((volume::fader_from_gain(g) - 0.5).abs() < 1e-5);
}
