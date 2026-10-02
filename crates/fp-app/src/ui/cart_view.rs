//! What each cartwall button shows (Phase 2 spec P2.9). Pure functions.

use fp_engine::conductor::Telemetry;
use fp_model::{AppState, CartId, CartKind, CartPageId};

use super::format;

/// Height of a cart button when there is room (feedback 2 spec O31).
pub const BUTTON_HEIGHT: f32 = 40.0;
/// The least a cart button shrinks to: room for the name and the detail line.
pub const MIN_BUTTON_HEIGHT: f32 = 28.0;
/// Gap between cart buttons and below the header.
pub const GAP: f32 = 6.0;

/// The height of a cart button when `rows` rows share `available` pixels
/// (gaps included): what fits, between `MIN_BUTTON_HEIGHT` and
/// `BUTTON_HEIGHT`. The grid scrolls only when even the minimum does not fit.
pub fn button_height(available: f32, rows: usize) -> f32 {
    if rows == 0 || available.is_nan() {
        return BUTTON_HEIGHT;
    }
    let gaps = GAP * (rows - 1) as f32;
    ((available - gaps) / rows as f32).clamp(MIN_BUTTON_HEIGHT, BUTTON_HEIGHT)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CartStatus {
    /// No file.
    Empty,
    /// The file is missing or cannot be decoded.
    Unavailable,
    Idle,
    Playing,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CartView {
    pub name: String,
    pub kind: CartKind,
    pub status: CartStatus,
    /// The length when idle, `-mm:ss` left while playing; empty if unknown.
    pub time: String,
    /// Share of the cart still to play (1 at the start, 0 at the end).
    pub remaining_fraction: f32,
    pub looped: bool,
    pub exclusive: bool,
    pub cueing: bool,
}

pub fn cart_view(state: &AppState, telemetry: &Telemetry, id: CartId) -> Option<CartView> {
    let cart = state.cartwall.cart(id)?;
    let track = cart.track.and_then(|t| state.library.get(t));
    let playing = state.cartwall.is_playing(id);
    let status = match track {
        None => CartStatus::Empty,
        Some(t) if !t.file_state.is_playable() => CartStatus::Unavailable,
        Some(_) if playing => CartStatus::Playing,
        Some(_) => CartStatus::Idle,
    };
    let mut view = CartView {
        name: cart.name.clone(),
        kind: cart.kind,
        status,
        time: String::new(),
        remaining_fraction: 0.0,
        looped: cart.looped,
        exclusive: cart.exclusive,
        cueing: state.cartwall.cue == Some(id),
    };
    let Some(track) = track else {
        return Some(view);
    };
    if view.name.is_empty() {
        view.name = track.title.clone();
    }
    let length = track.play_length_secs();
    let known = track.duration_secs > 0.0 && length > 0.0;
    if status == CartStatus::Playing {
        let position = telemetry
            .carts
            .iter()
            .find(|(c, _)| *c == id)
            .map_or(track.cue_in_secs(), |(_, t)| t.position_secs);
        if known {
            let left = (track.cue_out_secs() - position).clamp(0.0, length);
            view.time = format!("-{}", format::clock(left));
            view.remaining_fraction = (left / length) as f32;
        } else {
            view.remaining_fraction = 1.0;
        }
    } else if known {
        view.time = format::clock(length);
    }
    Some(view)
}

/// Whether any cart of `page` is on air (the red dot on its tab).
pub fn page_on_air(state: &AppState, page: CartPageId) -> bool {
    state
        .cartwall
        .page(page)
        .is_some_and(|p| p.carts.iter().any(|c| state.cartwall.is_playing(c.id)))
}
