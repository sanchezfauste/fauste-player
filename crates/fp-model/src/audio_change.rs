//! Whether a file's audio changed between two analyses (external editor
//! spec EE10). Pure.

use crate::track::AudioHash;

/// Whether the audio of a file changed between two analyses (EE10).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioChange {
    Same,
    Changed,
    /// A fingerprint is missing or they come from different versions.
    Unknown,
}

/// Compares the stored fingerprint with the new one.
pub fn audio_change(old: Option<AudioHash>, new: Option<u64>, new_version: u32) -> AudioChange {
    match (old, new) {
        (Some(old), Some(new)) if old.version == new_version => {
            if old.value == new {
                AudioChange::Same
            } else {
                AudioChange::Changed
            }
        }
        _ => AudioChange::Unknown,
    }
}
