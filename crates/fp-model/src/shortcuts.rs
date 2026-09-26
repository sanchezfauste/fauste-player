//! Keyboard shortcuts (Phase 2 spec P2.5). Players and carts are addressed
//! by 1-based position, not by id, so a binding survives player and page
//! changes. Key names follow the UI toolkit's names (`"1"`, `"A"`, `"F1"`,
//! `"Space"`); the UI ignores names it does not know.

use std::fmt;

use serde::{Deserialize, Serialize};

/// A key with its modifiers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyChord {
    pub key: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub ctrl: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub alt: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub shift: bool,
    /// The macOS Command key.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub command: bool,
}

impl KeyChord {
    /// A key without modifiers.
    pub fn key(key: &str) -> Self {
        Self {
            key: key.to_owned(),
            ctrl: false,
            alt: false,
            shift: false,
            command: false,
        }
    }

    pub fn ctrl(key: &str) -> Self {
        Self {
            ctrl: true,
            ..Self::key(key)
        }
    }
}

impl fmt::Display for KeyChord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (on, name) in [
            (self.ctrl, "Ctrl+"),
            (self.alt, "Alt+"),
            (self.shift, "Shift+"),
            (self.command, "Cmd+"),
        ] {
            if on {
                f.write_str(name)?;
            }
        }
        f.write_str(&self.key)
    }
}

/// Something a shortcut can do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShortcutAction {
    /// Play / next on the n-th player (1-based).
    PlayPlayer(u16),
    PausePlayer(u16),
    StopPlayer(u16),
    FadeStopPlayer(u16),
    CuePlayer(u16),
    /// Fire the n-th cart (1-based) of the page shown.
    FireCart(u16),
    StopAllCarts,
    ToggleCartwall,
    NextCartPage,
    PreviousCartPage,
}

impl ShortcutAction {
    /// The 1-based player or cart position the action targets, if any.
    pub fn position(self) -> Option<u16> {
        match self {
            Self::PlayPlayer(n)
            | Self::PausePlayer(n)
            | Self::StopPlayer(n)
            | Self::FadeStopPlayer(n)
            | Self::CuePlayer(n)
            | Self::FireCart(n) => Some(n),
            Self::StopAllCarts
            | Self::ToggleCartwall
            | Self::NextCartPage
            | Self::PreviousCartPage => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shortcut {
    pub action: ShortcutAction,
    pub chord: KeyChord,
}

/// `1`…`9` play players 1–9, `F1`…`F12` fire carts 1–12 of the page shown,
/// `Ctrl+Space` stops every cart.
pub fn default_shortcuts() -> Vec<Shortcut> {
    let players = (1..=9u16).map(|n| Shortcut {
        action: ShortcutAction::PlayPlayer(n),
        chord: KeyChord::key(&n.to_string()),
    });
    let carts = (1..=12u16).map(|n| Shortcut {
        action: ShortcutAction::FireCart(n),
        chord: KeyChord::key(&format!("F{n}")),
    });
    players
        .chain(carts)
        .chain(std::iter::once(Shortcut {
            action: ShortcutAction::StopAllCarts,
            chord: KeyChord::ctrl("Space"),
        }))
        .collect()
}
