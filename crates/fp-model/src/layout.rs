//! The main window's arrangement (resizable layout spec RL1–RL10, RL19):
//! the players' shares of the width, the hidden panels, where the cartwall
//! sits and how large it is, and the lock. Pure rules: the interface turns
//! them into rectangles (RL11).

use serde::{Deserialize, Serialize};

use crate::error::ModelError;
use crate::ids::PlayerId;
use crate::state::AppState;

/// RL6: the share of the middle area's width the cartwall gets when it is
/// docked on a side.
pub const DEFAULT_SIDE_SHARE: f32 = 0.3;
/// RL6: the smallest share of the middle area the cartwall's `size` holds.
pub const MIN_CARTWALL_SHARE: f32 = 0.05;
/// RL6: the largest share of the middle area the cartwall's `size` holds.
pub const MAX_CARTWALL_SHARE: f32 = 0.95;

/// Where the cartwall sits (spec §2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Dock {
    /// Under the players (today's place).
    #[default]
    Bottom,
    /// The full height of the middle area, left of the players.
    Left,
    /// The full height of the middle area, right of the players.
    Right,
}

impl Dock {
    /// Every dock, in the order the layout menu lists them.
    pub const ALL: [Dock; 3] = [Dock::Bottom, Dock::Left, Dock::Right];

    /// `Left` or `Right`.
    pub fn is_side(self) -> bool {
        !matches!(self, Dock::Bottom)
    }
}

/// One panel of the middle area (spec §2): a player column or the
/// cartwall.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Panel {
    Player(PlayerId),
    Cartwall,
}

/// The cartwall's place and size (RL1).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct CartwallLayout {
    pub dock: Dock,
    /// The cartwall's share of the middle area: of its height at the
    /// bottom, of its width on a side. `None` fits the content (bottom
    /// only).
    pub size: Option<f32>,
    pub hidden: bool,
}

impl CartwallLayout {
    /// RL6, RL11: the share the cartwall takes. At the bottom, `size`
    /// (`None`: fit the content); on a side, `size` or the default side
    /// share. A share is always within `MIN_CARTWALL_SHARE ..=
    /// MAX_CARTWALL_SHARE`; a non-finite `size` counts as `None`.
    pub fn share(&self) -> Option<f32> {
        let size = self.size.filter(|s| s.is_finite());
        let size = if self.dock.is_side() {
            size.or(Some(DEFAULT_SIDE_SHARE))
        } else {
            size
        };
        size.map(|s| s.clamp(MIN_CARTWALL_SHARE, MAX_CARTWALL_SHARE))
    }
}

/// The layout of the main window (RL1). Session state, saved with the
/// players; not part of `Config`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Layout {
    /// Each player's share of the players' width (RL2).
    pub player_shares: Vec<(PlayerId, f32)>,
    pub hidden_players: Vec<PlayerId>,
    pub cartwall: CartwallLayout,
    pub locked: bool,
}

impl Layout {
    /// RL7: whether `panel` is hidden.
    pub fn is_hidden(&self, panel: Panel) -> bool {
        match panel {
            Panel::Player(id) => self.hidden_players.contains(&id),
            Panel::Cartwall => self.cartwall.hidden,
        }
    }

    /// The shown players among `players` (`AppState::player_ids`), in
    /// display order.
    pub fn shown_players(&self, players: &[PlayerId]) -> Vec<PlayerId> {
        players
            .iter()
            .copied()
            .filter(|p| !self.hidden_players.contains(p))
            .collect()
    }

    /// How many panels are shown: the shown players and the cartwall.
    pub fn shown_panels(&self, players: &[PlayerId]) -> usize {
        self.shown_players(players).len() + usize::from(!self.cartwall.hidden)
    }

    /// RL7, RL15: `panel` is the only shown panel, so it cannot be hidden
    /// (its top-bar tile is disabled). False for a player not in
    /// `players`.
    pub fn is_last_shown(&self, players: &[PlayerId], panel: Panel) -> bool {
        let known = match panel {
            Panel::Player(id) => players.contains(&id),
            Panel::Cartwall => true,
        };
        known && !self.is_hidden(panel) && self.shown_panels(players) == 1
    }

    /// RL2: the share of `player`: its saved share when that is finite and
    /// above 0, else the mean of the valid shares of `players` (hidden ones
    /// included), else 1.0.
    pub fn share(&self, players: &[PlayerId], player: PlayerId) -> f32 {
        self.valid_share(player)
            .unwrap_or_else(|| self.mean_share(players))
    }

    /// RL2: the shown players with their fraction of the players' width;
    /// the fractions add up to 1. The pixel pass (RL11) adds the minimums.
    pub fn shown_fractions(&self, players: &[PlayerId]) -> Vec<(PlayerId, f32)> {
        let shown = self.shown_players(players);
        let shares: Vec<f64> = shown
            .iter()
            .map(|p| f64::from(self.share(players, *p)))
            .collect();
        let sum: f64 = shares.iter().sum();
        shown
            .into_iter()
            .zip(shares)
            .map(|(p, s)| (p, (s / sum) as f32))
            .collect()
    }

    /// RL4 (spec §2): `right` is the next shown player after `left`.
    /// Hidden players between them do not count.
    pub fn are_neighbours(&self, players: &[PlayerId], left: PlayerId, right: PlayerId) -> bool {
        self.shown_players(players)
            .windows(2)
            .any(|pair| pair == [left, right])
    }

    fn valid_share(&self, player: PlayerId) -> Option<f32> {
        self.player_shares
            .iter()
            .find(|(id, _)| *id == player)
            .map(|(_, share)| *share)
            .filter(|share| share.is_finite() && *share > 0.0)
    }

    /// Summed in `f64`, so a hand-edited `f32::MAX` cannot overflow.
    fn mean_share(&self, players: &[PlayerId]) -> f32 {
        let valid: Vec<f64> = players
            .iter()
            .filter_map(|p| self.valid_share(*p))
            .map(f64::from)
            .collect();
        if valid.is_empty() {
            1.0
        } else {
            (valid.iter().sum::<f64>() / valid.len() as f64) as f32
        }
    }
}

/// RL9: every layout command but `SetLayoutLocked` is refused while the
/// layout is locked. Checked before anything else.
pub(crate) fn unlocked(state: &AppState) -> Result<(), ModelError> {
    if state.layout.locked {
        Err(ModelError::LayoutLocked)
    } else {
        Ok(())
    }
}

fn usable_share(share: f32) -> bool {
    share.is_finite() && share > 0.0
}

/// Every player of `players` with its RL2 share, in display order.
fn resolved_shares(layout: &Layout, players: &[PlayerId]) -> Vec<(PlayerId, f32)> {
    players
        .iter()
        .map(|p| (*p, layout.share(players, *p)))
        .collect()
}

/// RL4: the pair takes the new shares, scaled to keep the pair's sum; every
/// current player's share is written down (shares of players that are gone
/// are dropped).
pub(crate) fn set_player_shares(
    state: &mut AppState,
    left: (PlayerId, f32),
    right: (PlayerId, f32),
) -> Result<(), ModelError> {
    unlocked(state)?;
    let players = state.player_ids();
    let layout = &state.layout;
    if !layout.are_neighbours(&players, left.0, right.0)
        || !usable_share(left.1)
        || !usable_share(right.1)
    {
        return Err(ModelError::BadLayout);
    }
    let before =
        f64::from(layout.share(&players, left.0)) + f64::from(layout.share(&players, right.0));
    let scale = before / (f64::from(left.1) + f64::from(right.1));
    let new_left = (f64::from(left.1) * scale) as f32;
    let new_right = (f64::from(right.1) * scale) as f32;
    if !usable_share(new_left) || !usable_share(new_right) {
        return Err(ModelError::BadLayout);
    }
    let mut shares = resolved_shares(layout, &players);
    for (id, share) in &mut shares {
        if *id == left.0 {
            *share = new_left;
        } else if *id == right.0 {
            *share = new_right;
        }
    }
    state.layout.player_shares = shares;
    Ok(())
}

/// RL5: every current player gets share 1.0.
pub(crate) fn equal_player_widths(state: &mut AppState) -> Result<(), ModelError> {
    unlocked(state)?;
    state.layout.player_shares = state.players.iter().map(|p| (p.id, 1.0)).collect();
    Ok(())
}
