//! The main window's arrangement (resizable layout spec RL1–RL10, RL19):
//! the players' shares of the width, the hidden panels, where the cartwall
//! sits and how large it is, and the lock. Pure rules: the interface turns
//! them into rectangles (RL11).

use serde::{Deserialize, Deserializer, Serialize};

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

/// One value read leniently: anything that does not parse is `Other`.
#[derive(Deserialize)]
#[serde(untagged)]
enum Lenient<T> {
    Valid(T),
    Other(serde::de::IgnoredAny),
}

impl<T> Lenient<T> {
    fn ok(self) -> Option<T> {
        match self {
            Lenient::Valid(v) => Some(v),
            Lenient::Other(_) => None,
        }
    }
}

/// RL19: a part that does not parse loads as its default.
fn lenient<'de, D: Deserializer<'de>, T: Deserialize<'de> + Default>(d: D) -> Result<T, D::Error> {
    Ok(Lenient::<T>::deserialize(d)?.ok().unwrap_or_default())
}

/// RL19: a list keeps every element that parses; a value that is not a
/// list is empty.
fn lenient_items<'de, D: Deserializer<'de>, T: Deserialize<'de>>(d: D) -> Result<Vec<T>, D::Error> {
    let items = Lenient::<Vec<Lenient<T>>>::deserialize(d)?
        .ok()
        .unwrap_or_default();
    Ok(items.into_iter().filter_map(Lenient::ok).collect())
}

/// RL19: the `deserialize_with` of a session's `layout`: a value that is
/// not a layout loads as the default, and inside it each part falls back
/// on its own. The store reports what fell back (`fp-store`'s
/// `layout_notes`).
pub fn lenient_layout<'de, D: Deserializer<'de>>(d: D) -> Result<Layout, D::Error> {
    lenient(d)
}

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
    #[serde(deserialize_with = "lenient")]
    pub dock: Dock,
    /// The cartwall's share of the middle area: of its height at the
    /// bottom, of its width on a side. `None` fits the content (bottom
    /// only).
    #[serde(deserialize_with = "lenient")]
    pub size: Option<f32>,
    #[serde(deserialize_with = "lenient")]
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
    #[serde(deserialize_with = "lenient_items")]
    pub player_shares: Vec<(PlayerId, f32)>,
    #[serde(deserialize_with = "lenient_items")]
    pub hidden_players: Vec<PlayerId>,
    #[serde(deserialize_with = "lenient")]
    pub cartwall: CartwallLayout,
    #[serde(deserialize_with = "lenient")]
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

/// RL6: a new dock resets the size.
pub(crate) fn set_cartwall_dock(state: &mut AppState, dock: Dock) -> Result<(), ModelError> {
    unlocked(state)?;
    let cartwall = &mut state.layout.cartwall;
    if cartwall.dock != dock {
        cartwall.dock = dock;
        cartwall.size = dock.is_side().then_some(DEFAULT_SIDE_SHARE);
    }
    Ok(())
}

/// RL6: the size, clamped; `None` is the default share on a side.
pub(crate) fn set_cartwall_size(state: &mut AppState, size: Option<f32>) -> Result<(), ModelError> {
    unlocked(state)?;
    if size.is_some_and(|s| !s.is_finite()) {
        return Err(ModelError::BadLayout);
    }
    let cartwall = &mut state.layout.cartwall;
    cartwall.size = CartwallLayout { size, ..*cartwall }.share();
    Ok(())
}

/// RL7: hides or shows `panel`; the last shown panel stays.
pub(crate) fn set_panel_hidden(
    state: &mut AppState,
    panel: Panel,
    hidden: bool,
) -> Result<(), ModelError> {
    unlocked(state)?;
    if let Panel::Player(id) = panel {
        state.player(id)?;
    }
    let players = state.player_ids();
    let layout = &mut state.layout;
    if layout.is_hidden(panel) == hidden {
        return Ok(());
    }
    if hidden && layout.is_last_shown(&players, panel) {
        return Err(ModelError::LastPanel);
    }
    match panel {
        Panel::Player(id) if hidden => layout.hidden_players.push(id),
        Panel::Player(id) => layout.hidden_players.retain(|p| *p != id),
        Panel::Cartwall => layout.cartwall.hidden = hidden,
    }
    Ok(())
}

/// RL10: the default layout, unlocked.
pub(crate) fn reset_layout(state: &mut AppState) -> Result<(), ModelError> {
    unlocked(state)?;
    state.layout = Layout::default();
    Ok(())
}

/// RL3 and RL19's repairs, for `players` (`AppState::player_ids`): drops
/// the shares and hidden flags of players that are not in `players`, a
/// second share or flag of the same player (the first wins) and shares
/// that are not finite and above 0; shows every panel when none is shown;
/// fits the cartwall's size to its dock (`CartwallLayout::share`).
/// Returns one note per repair, for the load warnings. Idempotent.
pub fn settle_layout(layout: &mut Layout, players: &[PlayerId]) -> Vec<String> {
    let mut notes = Vec::new();
    let mut seen = Vec::new();
    layout.player_shares.retain(|(id, share)| {
        let keep = if !players.contains(id) {
            notes.push(format!(
                "layout: the share of player {} was dropped: there is no such player",
                id.0
            ));
            false
        } else if seen.contains(id) {
            notes.push(format!(
                "layout: a second share of player {} was dropped",
                id.0
            ));
            false
        } else if !usable_share(*share) {
            notes.push(format!(
                "layout: the share {share} of player {} is not above 0; it was dropped",
                id.0
            ));
            false
        } else {
            true
        };
        if keep {
            seen.push(*id);
        }
        keep
    });
    let mut seen = Vec::new();
    layout.hidden_players.retain(|id| {
        let keep = if !players.contains(id) {
            notes.push(format!(
                "layout: the hidden flag of player {} was dropped: there is no such player",
                id.0
            ));
            false
        } else if seen.contains(id) {
            notes.push(format!("layout: player {} was hidden twice", id.0));
            false
        } else {
            true
        };
        if keep {
            seen.push(*id);
        }
        keep
    });
    if layout.shown_panels(players) == 0 {
        layout.hidden_players.clear();
        layout.cartwall.hidden = false;
        notes.push("layout: every panel was hidden; all are shown again".to_owned());
    }
    let fitted = layout.cartwall.share();
    if fitted.map(f32::to_bits) != layout.cartwall.size.map(f32::to_bits) {
        notes.push(format!(
            "layout: the cartwall's size {:?} does not fit its dock; it is now {fitted:?}",
            layout.cartwall.size
        ));
        layout.cartwall.size = fitted;
    }
    notes
}
