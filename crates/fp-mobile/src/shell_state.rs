//! What the Kotlin shell polls from the session's model snapshot, so it is
//! right with no activity on screen.

use fp_model::{AppState, Transport, playout_active};

/// Bit of `shell_state`: run the foreground service (MB1).
pub const PLAYOUT_ACTIVE: i32 = 1;
/// Bit of `shell_state`: keep the screen on (MB8).
pub const KEEP_SCREEN_ON: i32 = 2;

/// `PLAYOUT_ACTIVE` while playout is active, with `KEEP_SCREEN_ON` when
/// the setting asks for it too.
pub fn shell_state(state: &AppState) -> i32 {
    if !playout_active(state) {
        return 0;
    }
    if state.config.ui.keep_screen_on {
        PLAYOUT_ACTIVE | KEEP_SCREEN_ON
    } else {
        PLAYOUT_ACTIVE
    }
}

/// The notification's text: one "P<n> — <title>" line per player playing
/// or paused with a track, then one line per playing cart; empty when
/// nothing is on air.
pub fn on_air_text(state: &AppState) -> String {
    let players = state.players.iter().enumerate().filter_map(|(i, p)| {
        if p.transport == Transport::Stopped {
            return None;
        }
        let track = p.current.and_then(|e| state.track_for_entry(e))?;
        Some(format!("P{} — {}", i + 1, track.title))
    });
    let carts = state
        .cartwall
        .playing
        .iter()
        .filter_map(|c| state.cartwall.cart(c.cart))
        .map(|c| format!("▶ {}", c.name));
    players.chain(carts).collect::<Vec<_>>().join("\n")
}
