//! The touch layout (mobile spec §6): the form factor from the screen size
//! and what a phone shows. A tablet keeps the desktop layout.

use egui::Vec2;
use fp_model::PlayerId;

/// The shortest side, in points (Android's dp), from which a screen is a
/// tablet (spec §6).
const TABLET_SIDE: f32 = 600.0;

/// How much of the screen there is to lay out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormFactor {
    Phone { landscape: bool },
    Tablet,
}

pub fn form_factor(size: Vec2) -> FormFactor {
    if size.x.min(size.y) >= TABLET_SIDE {
        FormFactor::Tablet
    } else {
        FormFactor::Phone {
            landscape: size.x > size.y,
        }
    }
}

/// The phone's screens, chosen in the top bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PhoneScreen {
    #[default]
    Players,
    Cartwall,
}

/// What the phone shows: the screen, the player open on it (all players
/// when none) and, in landscape, which pair of players.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PhoneView {
    pub screen: PhoneScreen,
    pub open: Option<PlayerId>,
    pub pair: usize,
}

impl PhoneView {
    /// Follows the players there are: an open player that went away closes,
    /// and the pair stays within the players.
    pub fn settle(&mut self, players: &[PlayerId]) {
        if self.open.is_some_and(|id| !players.contains(&id)) {
            self.open = None;
        }
        self.pair = self.pair.min(Self::pairs(players.len()).saturating_sub(1));
    }

    /// How many pairs `count` players make (the last may hold one).
    pub fn pairs(count: usize) -> usize {
        count.div_ceil(2)
    }

    /// The players of the shown pair.
    pub fn pair_players(&self, players: &[PlayerId]) -> Vec<PlayerId> {
        players
            .iter()
            .skip(self.pair * 2)
            .take(2)
            .copied()
            .collect()
    }

    /// The next (`forward`) or the previous pair, if there is one.
    pub fn swipe(&mut self, forward: bool, count: usize) {
        let last = Self::pairs(count).saturating_sub(1);
        self.pair = if forward {
            (self.pair + 1).min(last)
        } else {
            self.pair.saturating_sub(1)
        };
    }
}
