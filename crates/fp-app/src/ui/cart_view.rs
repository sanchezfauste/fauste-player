//! What each cartwall button shows (Phase 2 spec P2.9). Pure functions.

use egui::Color32;
use fp_engine::conductor::Telemetry;
use fp_model::{AppState, CartColor, CartId, CartKind, CartPageId};

use super::{format, theme};

/// Height of a cart button when there is room (feedback 2 spec O31).
pub const BUTTON_HEIGHT: f32 = 40.0;
/// The least a cart button shrinks to: room for the name and the detail line.
pub const MIN_BUTTON_HEIGHT: f32 = 28.0;
/// RL8, RL13: the least width of a cart button; a grid narrower than its
/// columns at this width scrolls.
pub const MIN_BUTTON_WIDTH: f32 = 56.0;
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

/// The mark that ends a cut word.
const ELLIPSIS: &str = "…";

/// A cart's name laid out for a button `width` points wide, as `measure`
/// gives a line's width: its words, as many to a line as fit, the lines
/// joined by `\n`. A word is never split across lines: one that does not
/// fit a line of its own is cut with an ellipsis, on its own line, and
/// ends the name. Runs of spaces collapse to one.
pub fn fit_name(name: &str, width: f32, measure: impl Fn(&str) -> f32) -> String {
    let fits = |text: &str| measure(text) <= width;
    let mut lines: Vec<String> = Vec::new();
    let mut line = String::new();
    for word in name.split_whitespace() {
        let joined = if line.is_empty() {
            word.to_owned()
        } else {
            format!("{line} {word}")
        };
        if fits(&joined) {
            line = joined;
            continue;
        }
        if !line.is_empty() {
            lines.push(std::mem::take(&mut line));
        }
        if fits(word) {
            line = word.to_owned();
            continue;
        }
        let cut = word
            .char_indices()
            .rev()
            .map(|(end, _)| end)
            .filter(|end| *end > 0)
            .filter_map(|end| word.get(..end))
            .map(|prefix| format!("{prefix}{ELLIPSIS}"))
            .find(|cut| fits(cut))
            .unwrap_or_else(|| ELLIPSIS.to_owned());
        lines.push(cut);
        return lines.join("\n");
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines.join("\n")
}

/// RL13: the height of a cart button when the grid fills a panel that does
/// not fit its content: what fits, never under `MIN_BUTTON_HEIGHT`, with no
/// upper clamp. The grid scrolls only when even the minimum does not fit.
pub fn filled_button_height(available: f32, rows: usize) -> f32 {
    if rows == 0 || !available.is_finite() {
        return BUTTON_HEIGHT;
    }
    let gaps = GAP * (rows - 1) as f32;
    ((available - gaps) / rows as f32).max(MIN_BUTTON_HEIGHT)
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
    pub looped: bool,
    pub exclusive: bool,
    pub cueing: bool,
    /// The cart's colour (cart colour spec CC1).
    pub color: Option<CartColor>,
    /// The play range the waveform spans, `(start_secs, span_secs)` (cart
    /// colour spec CC8): only for an idle or playing cart whose length is
    /// known.
    pub wave_range: Option<(f64, f64)>,
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
        looped: cart.looped,
        exclusive: cart.exclusive,
        cueing: state.cartwall.cue == Some(id),
        color: cart.color,
        wave_range: None,
    };
    let Some(track) = track else {
        return Some(view);
    };
    if view.name.is_empty() {
        view.name = track.title.clone();
    }
    let length = track.play_length_secs();
    let known = track.duration_secs > 0.0 && length > 0.0;
    if known && matches!(status, CartStatus::Idle | CartStatus::Playing) {
        view.wave_range = Some((track.cue_in_secs(), length));
    }
    if status == CartStatus::Playing {
        let position = telemetry
            .carts
            .iter()
            .find(|(c, _)| *c == id)
            .map_or(track.cue_in_secs(), |(_, t)| t.position_secs);
        if known {
            let left = (track.cue_out_secs() - position).clamp(0.0, length);
            view.time = format!("-{}", format::clock(left));
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

/// The playing position a cart button draws, between engine snapshots.
///
/// The engine publishes a position every few milliseconds, but the interface
/// draws on its own clock; the sweep carries the last snapshot forward with
/// the frame clock, so the line moves a little every frame (sub-pixel) instead
/// of in the steps of whatever snapshot happened to arrive. It never moves
/// backwards, except when the snapshot itself does (a loop pass, a restart):
/// a snapshot a little behind what was drawn holds the line for a moment
/// instead of stepping it back. The extrapolation past the last snapshot is
/// capped, so an engine that stops reporting does not run the line to the end.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CartSweep {
    /// The last position the engine reported.
    snapshot: f64,
    /// The frame clock when it was first seen.
    since: f64,
    /// The position drawn on the previous frame.
    shown: f64,
}

impl CartSweep {
    pub fn new(position: f64, now: f64) -> Self {
        Self {
            snapshot: position,
            since: now,
            shown: position,
        }
    }

    /// The position to draw at frame time `now`, given the engine's latest
    /// `position` and the cart's `end` (seconds); never past `end`, and never
    /// more than `max_ahead` seconds past the snapshot.
    pub fn advance(&mut self, position: f64, now: f64, end: f64, max_ahead: f64) -> f64 {
        if position < self.snapshot {
            *self = Self::new(position, now);
            return position;
        }
        if position > self.snapshot {
            self.snapshot = position;
            self.since = now;
        }
        let ahead = self.snapshot + (now - self.since).clamp(0.0, max_ahead.max(0.0));
        let shown = ahead.max(self.shown).min(end).max(position);
        self.shown = shown;
        shown
    }
}

/// The colours of a cart button (cart colour spec CC6–CC8).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CartLook {
    pub fill: Color32,
    pub hover_fill: Color32,
    pub active_fill: Color32,
    pub border: Color32,
    /// The elapsed-time fill while it plays.
    pub progress: Color32,
    /// The kind marker of an idle or playing cart; `None`: the kind's own.
    pub marker: Option<Color32>,
    /// The waveform behind the text.
    pub wave: Color32,
}

pub fn cart_look(color: Option<CartColor>, playing: bool) -> CartLook {
    let wave_alpha = if playing {
        theme::CART_WAVE_PLAYING_ALPHA
    } else {
        theme::CART_WAVE_IDLE_ALPHA
    };
    match color {
        None => CartLook {
            fill: theme::NEUTRAL_900,
            hover_fill: theme::NEUTRAL_800,
            active_fill: theme::NEUTRAL_700,
            border: if playing {
                theme::ON_AIR_TEXT
            } else {
                theme::NEUTRAL_800
            },
            progress: theme::ON_AIR_ROW.gamma_multiply(0.5),
            marker: None,
            wave: theme::CART_WAVE_PLAIN.gamma_multiply(wave_alpha),
        },
        Some(color) => {
            let tone = theme::cart_color(color);
            let wash = if playing {
                theme::CART_WASH_PLAYING
            } else {
                theme::CART_WASH_IDLE
            };
            CartLook {
                fill: theme::NEUTRAL_900.lerp_to_gamma(tone, wash),
                hover_fill: theme::NEUTRAL_800.lerp_to_gamma(tone, wash),
                active_fill: theme::NEUTRAL_700.lerp_to_gamma(tone, wash),
                border: tone,
                progress: tone.gamma_multiply(theme::CART_PROGRESS_ALPHA),
                marker: Some(tone),
                wave: tone.gamma_multiply(wave_alpha),
            }
        }
    }
}
