//! MIDI control configuration (feedback spec §6.2).

use serde::{Deserialize, Serialize};

use crate::config::ConfigWarning;
use crate::shortcuts::ShortcutAction;

/// MIDI control surfaces: off until the operator turns them on.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MidiConfig {
    pub enabled: bool,
    /// How often ports are looked for again (hot-plug), in milliseconds.
    pub rescan_interval_ms: u32,
    /// Light the surfaces' buttons to show the players' state.
    pub feedback: bool,
    /// Per input port, where its LED feedback goes when it is not the
    /// output port of the same name.
    pub devices: Vec<MidiDevice>,
    pub bindings: Vec<MidiBinding>,
}

impl Default for MidiConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            rescan_interval_ms: 2000,
            feedback: true,
            devices: Vec::new(),
            bindings: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MidiDevice {
    /// The input port's name.
    pub input: String,
    /// The output port for its feedback; `None` uses the same name.
    #[serde(default)]
    pub output: Option<String>,
}

/// One control of one device bound to one action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MidiBinding {
    /// The input port's name.
    pub device: String,
    pub trigger: MidiTrigger,
    pub action: MidiAction,
}

/// A control on a surface. Channels are 0-based (0–15).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MidiTrigger {
    Note { channel: u8, note: u8 },
    ControlChange { channel: u8, controller: u8 },
    PitchBend { channel: u8 },
}

/// What a control does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MidiAction {
    /// A per-player transport action (Play/Next, Pause, Stop, Fade stop,
    /// Restart, Previous, Cue).
    Button(ShortcutAction),
    /// The volume fader of the n-th player (1-based).
    Volume(u16),
}

impl MidiAction {
    /// Whether MIDI can do this: per-player transport actions and volumes.
    pub fn is_valid(self) -> bool {
        match self {
            Self::Button(a) => matches!(
                a,
                ShortcutAction::PlayPlayer(_)
                    | ShortcutAction::PausePlayer(_)
                    | ShortcutAction::StopPlayer(_)
                    | ShortcutAction::FadeStopPlayer(_)
                    | ShortcutAction::RestartPlayer(_)
                    | ShortcutAction::PreviousPlayer(_)
                    | ShortcutAction::CuePlayer(_)
            ),
            Self::Volume(n) => n >= 1,
        }
    }
}

impl MidiTrigger {
    fn in_range(self) -> bool {
        match self {
            Self::Note { channel, note } => channel < 16 && note < 128,
            Self::ControlChange {
                channel,
                controller,
            } => channel < 16 && controller < 128,
            Self::PitchBend { channel } => channel < 16,
        }
    }
}

impl MidiConfig {
    /// Binds `trigger` of `device` to `action` (MIDI learn): the action's
    /// previous binding goes, and so does any other action bound to the same
    /// control.
    pub fn bind(&mut self, action: MidiAction, device: &str, trigger: MidiTrigger) {
        self.bindings
            .retain(|b| b.action != action && !(b.device == device && b.trigger == trigger));
        self.bindings.push(MidiBinding {
            device: device.to_owned(),
            trigger,
            action,
        });
    }

    /// Removes the binding of `action`, if any.
    pub fn unbind(&mut self, action: MidiAction) {
        self.bindings.retain(|b| b.action != action);
    }

    /// The binding of `action`, if any.
    pub fn binding(&self, action: MidiAction) -> Option<&MidiBinding> {
        self.bindings.iter().find(|b| b.action == action)
    }

    /// Drops bindings that cannot work: an action MIDI does not offer, a
    /// value out of MIDI's range, a volume on a note.
    pub(crate) fn validate(&mut self, w: &mut Vec<ConfigWarning>) {
        self.bindings.retain(|b| {
            let volume_on_note = matches!(b.action, MidiAction::Volume(_))
                && matches!(b.trigger, MidiTrigger::Note { .. });
            let ok = b.action.is_valid() && b.trigger.in_range() && !volume_on_note;
            if !ok {
                w.push(ConfigWarning {
                    field: "midi.bindings",
                    message: format!("{b:?} cannot be used; dropped"),
                });
            }
            ok
        });
    }
}
