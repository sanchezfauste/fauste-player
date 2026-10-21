//! The main window's pixel pass (resizable layout spec RL11): from the
//! layout, the middle area and the panels' minimums (RL8) to the players'
//! widths and the cartwall's rectangle, with a maximised panel (RL16).
//! Pure and unit-tested; `ui/panels.rs` draws with it.

use egui::{Rect, Vec2, pos2};
use fp_model::PlayerId;
use fp_model::layout::Panel;

/// The gap between two panels; between two players it is a splitter
/// (RL12).
pub const PANEL_GAP: f32 = 8.0;
/// RL8: a player column's least width: its fixed parts (the widest meter
/// with its rulers, the fader with its scale, the transport) beside a
/// countdown at its smallest.
pub const MIN_PLAYER_WIDTH: f32 = 380.0;
/// RL8: the players' row's least height: a column's header, info row and
/// transport, waveform, time line, tabs, table header, three rows and
/// footer (430 points, borders and margins included), and the 2 points the
/// row keeps under each column.
pub const MIN_PLAYER_HEIGHT: f32 = 432.0;
/// RL13 ("fit the content"): a bottom cartwall that fits its content never
/// takes more than this share of the middle area's height, as before the
/// resizable layout.
pub const CONTENT_HEIGHT_CAP: f32 = 0.6;

/// The cartwall's heights from its content (`cartwall::sizes`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CartwallSizes {
    /// What its content wants at the bottom: the header, or the header and
    /// the shown page's rows at their nominal height.
    pub content_height: f32,
    /// RL8: its least height: the header, or the header and the rows at the
    /// smallest button.
    pub min_height: f32,
}

/// What the pass reads.
#[derive(Debug, Clone, Copy)]
pub struct PassInput<'a> {
    /// Every player, in display order (`AppState::player_ids`).
    pub players: &'a [PlayerId],
    pub layout: &'a fp_model::layout::Layout,
    /// The middle area, inside its margin.
    pub area: Rect,
    pub cartwall: CartwallSizes,
    /// RL16: the panel that takes all the free space for now.
    pub maximised: Option<Panel>,
}

/// Where everything goes this frame.
#[derive(Debug, Clone, PartialEq)]
pub struct Arrangement {
    /// The players' row (its horizontal scroll area). Empty when no player
    /// is shown.
    pub players_area: Rect,
    /// Each shown player with its column's width, in display order.
    pub widths: Vec<(PlayerId, f32)>,
    /// RL11: the minimums alone do not fit, so the row scrolls.
    pub overflow: bool,
    /// The cartwall's rectangle; `None` while it is hidden.
    pub cartwall: Option<Rect>,
}

/// RL11: the pass. The cartwall first, then the players split the rest:
/// each gets its minimum width and the spare room is split by share.
pub fn arrange(input: &PassInput<'_>) -> Arrangement {
    let layout = input.layout;
    let shown = layout.shown_players(input.players);
    // A hidden panel is never maximised (RL16).
    let maximised = input.maximised.filter(|panel| match panel {
        Panel::Player(id) => shown.contains(id),
        Panel::Cartwall => !layout.cartwall.hidden,
    });
    let area = input.area;
    let (players_area, cartwall) = if layout.cartwall.hidden {
        (area, None)
    } else if shown.is_empty() {
        (Rect::from_min_size(area.min, Vec2::ZERO), Some(area))
    } else {
        // Plan 3 makes this a `match` on `layout.cartwall.dock`, with an arm
        // for the side docks; until then every dock is laid out here.
        at_the_bottom(input, maximised)
    };
    let (widths, overflow) = player_widths(input, &shown, players_area.width(), maximised);
    Arrangement {
        players_area,
        widths,
        overflow,
        cartwall,
    }
}

/// The cartwall under the players: its content height ("fit the content",
/// exactly as before), or its share clamped between its minimum and what
/// leaves the players `MIN_PLAYER_HEIGHT` (RL8, RL11); its minimum next to
/// a maximised player, everything but the players' least height when it is
/// maximised itself (RL16).
fn at_the_bottom(input: &PassInput<'_>, maximised: Option<Panel>) -> (Rect, Option<Rect>) {
    let area = input.area;
    let sizes = input.cartwall;
    let room = area.height() - PANEL_GAP - MIN_PLAYER_HEIGHT;
    let height = match (maximised, input.layout.cartwall.share()) {
        (Some(Panel::Cartwall), _) => room.max(sizes.min_height),
        (Some(Panel::Player(_)), _) => sizes.min_height,
        (None, None) => sizes.content_height.min(area.height() * CONTENT_HEIGHT_CAP),
        (None, Some(share)) => (area.height() * share).min(room).max(sizes.min_height),
    };
    let top = (area.bottom() - height).max(area.top());
    let cartwall = Rect::from_min_max(pos2(area.left(), top), area.max);
    let players = Rect::from_min_max(
        area.min,
        pos2(area.right(), (top - PANEL_GAP).max(area.top())),
    );
    (players, Some(cartwall))
}

/// RL11: each shown player gets `MIN_PLAYER_WIDTH`, and the spare room of
/// `width` is split by the players' fractions (all of it to a maximised
/// player, RL16). When the minimums alone do not fit, every column is at
/// its minimum and the row overflows (half a point of rounding is not an
/// overflow, as before).
fn player_widths(
    input: &PassInput<'_>,
    shown: &[PlayerId],
    width: f32,
    maximised: Option<Panel>,
) -> (Vec<(PlayerId, f32)>, bool) {
    if shown.is_empty() {
        return (Vec::new(), false);
    }
    let count = shown.len() as f32;
    let spare = width - PANEL_GAP * (count - 1.0) - MIN_PLAYER_WIDTH * count;
    if spare <= 0.0 {
        let widths = shown.iter().map(|p| (*p, MIN_PLAYER_WIDTH)).collect();
        return (widths, spare < -0.5);
    }
    let fractions: Vec<(PlayerId, f32)> = match maximised {
        Some(Panel::Player(big)) => shown
            .iter()
            .map(|p| (*p, if *p == big { 1.0 } else { 0.0 }))
            .collect(),
        _ => input.layout.shown_fractions(input.players),
    };
    let widths = fractions
        .into_iter()
        .map(|(p, f)| (p, MIN_PLAYER_WIDTH + spare * f))
        .collect();
    (widths, false)
}
