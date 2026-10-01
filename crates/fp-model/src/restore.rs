//! "Restore defaults" in Settings (feedback 2 spec O2): one section's
//! fields go back to `Config::default()`, and nothing else changes.

use crate::config::{Config, PlayersConfig};

/// The Settings sections that offer "Restore defaults". Outputs and MIDI
/// depend on the hardware, Remote holds security settings, and Playlists
/// and Cartwall hold show data: they have no button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettingsSection {
    Players,
    Meters,
    Analysis,
    Shortcuts,
}

/// Resets the fields `section` shows. In Players, the player count is
/// kept (it adds or removes players, which is the show's layout, not a
/// preference) and so is the interface language (the operator's own).
pub fn restore_defaults(config: &mut Config, section: SettingsSection) {
    let defaults = Config::default();
    match section {
        SettingsSection::Players => {
            config.players = PlayersConfig {
                count: config.players.count,
                ..defaults.players
            };
        }
        SettingsSection::Meters => config.meter = defaults.meter,
        SettingsSection::Analysis => config.analysis = defaults.analysis,
        SettingsSection::Shortcuts => config.shortcuts = defaults.shortcuts,
    }
}
