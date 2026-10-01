//! What is on air (feedback 2 spec O6): what closing or restarting the
//! application would cut. A CUE is pre-listening, not on air.

use crate::{AppState, CartId, EntryId, PlayerId, Transport};

/// One thing that is on air: a player or a cart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnAir {
    /// A player that is playing or paused, with its current entry.
    Player {
        player: PlayerId,
        entry: Option<EntryId>,
    },
    /// A playing cart.
    Cart(CartId),
}

/// Players in display order, then carts in firing order.
pub fn on_air(state: &AppState) -> Vec<OnAir> {
    let players = state
        .players
        .iter()
        .filter(|p| p.transport != Transport::Stopped)
        .map(|p| OnAir::Player {
            player: p.id,
            entry: p.current,
        });
    let carts = state.cartwall.playing.iter().map(|c| OnAir::Cart(c.cart));
    players.chain(carts).collect()
}
