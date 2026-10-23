//! The main window's pixel pass (resizable layout spec RL11): from the
//! layout, the middle area and the panels' minimums (RL8) to the players'
//! widths and the cartwall's rectangle, with a maximised panel (RL16).
//! Pure and unit-tested; `ui/panels.rs` draws with it.

use egui::{Pos2, Rect, Vec2, pos2};
use fp_model::layout::{DEFAULT_SIDE_SHARE, Dock, MAX_CARTWALL_SHARE, MIN_CARTWALL_SHARE, Panel};
use fp_model::{AppState, Command, PlayerId, Transport};

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
/// RL8, RL11: the strip an overflowing players' row keeps under its
/// columns for the horizontal scroll bar: egui's bar (10 points wide, 4 of
/// inner margin) and 2 points of air.
pub const SCROLL_BAR_ROOM: f32 = 16.0;
/// RL13 ("fit the content"): a bottom cartwall that fits its content never
/// takes more than this share of the middle area's height, as before the
/// resizable layout.
pub const CONTENT_HEIGHT_CAP: f32 = 0.6;

/// The cartwall's sizes from its content (`cartwall::sizes`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CartwallSizes {
    /// What its content wants at the bottom: the header, or the header and
    /// the shown page's rows at their nominal height.
    pub content_height: f32,
    /// RL8: its least height: the header, or the header and the rows at the
    /// smallest button.
    pub min_height: f32,
    /// RL8: its least width on a side: the larger of its header's fixed
    /// parts and the shown page's columns at the smallest button.
    pub min_width: f32,
    /// RL14: while it is collapsed, its thickness (the header's height):
    /// its height at the bottom, its width on a side. `None` while open.
    pub collapsed: Option<f32>,
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
    /// RL12: the gap between the players and the cartwall, its splitter;
    /// `None` while the cartwall is hidden or no player is shown.
    pub splitter: Option<Rect>,
    /// RL13: the cartwall fits its content (at the bottom, without a share,
    /// nothing maximised, a player shown): its buttons keep `BUTTON_HEIGHT`
    /// at most. Otherwise they fill it.
    pub fit_content: bool,
}

/// RL11: the pass. The cartwall first, then the players split the rest:
/// each gets its minimum width and the spare room is split by share.
pub fn arrange(input: &PassInput<'_>) -> Arrangement {
    let layout = input.layout;
    let shown = layout.shown_players(input.players);
    // A hidden panel is never maximised (RL16), nor a collapsed cartwall:
    // it has nothing to fill the room with (RL14).
    let maximised = input.maximised.filter(|panel| match panel {
        Panel::Player(id) => shown.contains(id),
        Panel::Cartwall => !layout.cartwall.hidden && input.cartwall.collapsed.is_none(),
    });
    let area = input.area;
    let dock = layout.cartwall.dock;
    let (players_area, cartwall) = if layout.cartwall.hidden {
        (area, None)
    } else {
        match dock {
            Dock::Bottom if shown.is_empty() => {
                (Rect::from_min_size(area.min, Vec2::ZERO), Some(area))
            }
            Dock::Bottom => at_the_bottom(input, maximised),
            Dock::Left | Dock::Right => on_a_side(input, maximised),
        }
    };
    let splitter = match cartwall {
        Some(cart) if !shown.is_empty() => Some(gap_between(dock, players_area, cart)),
        _ => None,
    };
    let fit_content = dock == Dock::Bottom
        && !layout.cartwall.hidden
        && !shown.is_empty()
        && maximised.is_none()
        && layout.cartwall.share().is_none();
    let (widths, overflow) = player_widths(input, &shown, players_area.width(), maximised);
    Arrangement {
        players_area,
        widths,
        overflow,
        cartwall,
        splitter,
        fit_content,
    }
}

/// RL6, RL8, RL11, RL12: the least and the most size, in points along its
/// axis (its height at the bottom, its width on a side), that the open
/// cartwall can take beside the shown players: its own minimum, and what
/// leaves the players theirs (at the bottom `players_least_height`, with
/// the scroll bar's strip of an overflowing row), both within the shares a
/// size can hold (`MIN_CARTWALL_SHARE` to `MAX_CARTWALL_SHARE` of the
/// area, so the drawn size never lags the pointer), and never below its
/// own minimum. `None` while it is hidden or collapsed, or with no player
/// shown: then it has no size to choose.
pub fn cartwall_range(input: &PassInput<'_>) -> Option<(f32, f32)> {
    let layout = input.layout;
    let sizes = input.cartwall;
    let shown = layout.shown_players(input.players).len();
    if layout.cartwall.hidden || sizes.collapsed.is_some() || shown == 0 {
        return None;
    }
    let area = input.area;
    let (own, room, extent) = if layout.cartwall.dock.is_side() {
        let count = shown as f32;
        let players = count * MIN_PLAYER_WIDTH + (count - 1.0) * PANEL_GAP;
        let extent = area.width();
        (sizes.min_width, extent - PANEL_GAP - players, extent)
    } else {
        let players = players_least_height(spare_width(shown, area.width()) < -0.5);
        let extent = area.height();
        (sizes.min_height, extent - PANEL_GAP - players, extent)
    };
    let least = own.max(MIN_CARTWALL_SHARE * extent);
    let most = room.min(MAX_CARTWALL_SHARE * extent);
    Some((least, most.max(least)))
}

/// RL8: the players' row's least height: `MIN_PLAYER_HEIGHT`, and the
/// scroll bar's strip when the row overflows (so three rows still fit
/// above the bar).
pub fn players_least_height(overflow: bool) -> f32 {
    if overflow {
        MIN_PLAYER_HEIGHT + SCROLL_BAR_ROOM
    } else {
        MIN_PLAYER_HEIGHT
    }
}

/// RL11: what is left of `width` once `count` players have their minimum
/// width and the gaps between them; below 0 they do not fit.
fn spare_width(count: usize, width: f32) -> f32 {
    let count = count as f32;
    width - PANEL_GAP * (count - 1.0).max(0.0) - MIN_PLAYER_WIDTH * count
}

/// The cartwall under the players: its content height ("fit the content",
/// exactly as before, not clamped further), or its share clamped to
/// `cartwall_range` (RL8, RL11); its least height next to a maximised
/// player, its most when it is maximised itself (RL16); collapsed, only its
/// header whatever its share (RL14).
fn at_the_bottom(input: &PassInput<'_>, maximised: Option<Panel>) -> (Rect, Option<Rect>) {
    let area = input.area;
    let sizes = input.cartwall;
    let height = match (
        cartwall_range(input),
        maximised,
        input.layout.cartwall.share(),
    ) {
        (None, _, _) => sizes.content_height,
        (Some((_, most)), Some(Panel::Cartwall), _) => most,
        (Some((least, _)), Some(Panel::Player(_)), _) => least,
        (Some(_), None, None) => sizes.content_height.min(area.height() * CONTENT_HEIGHT_CAP),
        (Some((least, most)), None, Some(share)) => (area.height() * share).min(most).max(least),
    };
    let top = (area.bottom() - height).max(area.top());
    let cartwall = Rect::from_min_max(pos2(area.left(), top), area.max);
    let players = Rect::from_min_max(
        area.min,
        pos2(area.right(), (top - PANEL_GAP).max(area.top())),
    );
    (players, Some(cartwall))
}

/// RL6: the cartwall the full height of the area, on its side: its share of
/// the width clamped to `cartwall_range` (RL8, RL11), its least width next
/// to a maximised player, its most when maximised itself (RL16), the
/// collapsed strip (RL14), or the whole area with no player shown. Never
/// wider than the area; the players take what is left.
fn on_a_side(input: &PassInput<'_>, maximised: Option<Panel>) -> (Rect, Option<Rect>) {
    let area = input.area;
    let layout = input.layout;
    let width = match (input.cartwall.collapsed, cartwall_range(input), maximised) {
        (Some(strip), _, _) => strip,
        (None, None, _) => area.width(),
        (None, Some((_, most)), Some(Panel::Cartwall)) => most,
        (None, Some((least, _)), Some(Panel::Player(_))) => least,
        (None, Some((least, most)), None) => {
            let share = layout.cartwall.share().unwrap_or(DEFAULT_SIDE_SHARE);
            (area.width() * share).min(most).max(least)
        }
    }
    .min(area.width())
    .max(0.0);
    if layout.cartwall.dock == Dock::Right {
        let left = area.right() - width;
        let cart = Rect::from_min_max(pos2(left, area.top()), area.max);
        let players = Rect::from_min_max(
            area.min,
            pos2((left - PANEL_GAP).max(area.left()), area.bottom()),
        );
        (players, Some(cart))
    } else {
        let right = area.left() + width;
        let cart = Rect::from_min_max(area.min, pos2(right, area.bottom()));
        let players = Rect::from_min_max(
            pos2((right + PANEL_GAP).min(area.right()), area.top()),
            area.max,
        );
        (players, Some(cart))
    }
}

/// RL12: the gap between the players' row and the cartwall.
fn gap_between(dock: Dock, players: Rect, cart: Rect) -> Rect {
    match dock {
        Dock::Bottom => Rect::from_min_max(
            pos2(cart.left(), players.bottom()),
            pos2(cart.right(), cart.top()),
        ),
        Dock::Left => Rect::from_min_max(
            pos2(cart.right(), cart.top()),
            pos2(players.left(), cart.bottom()),
        ),
        Dock::Right => Rect::from_min_max(
            pos2(players.right(), cart.top()),
            pos2(cart.left(), cart.bottom()),
        ),
    }
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
    let spare = spare_width(shown.len(), width);
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

/// RL12: how long a released splitter's widths are held while the model's
/// echo is on its way, as the table's column resize holds its widths.
pub const HOLD_SECS: f64 = 0.5;
/// RL12, RL4: the share a player dragged to its minimum width sends, as a
/// part of the pair's spare room; the model refuses 0.
pub const SHARE_FLOOR: f32 = 1e-4;

/// RL12: the pair's widths after the splitter between them moved `dx`
/// from where the drag began. The pair keeps its total and neither goes
/// under `MIN_PLAYER_WIDTH`; a pair without room, or a `dx` that is not a
/// number, does not move.
pub fn drag_pair(start: (f32, f32), dx: f32) -> (f32, f32) {
    let total = start.0 + start.1;
    if !dx.is_finite() || total < 2.0 * MIN_PLAYER_WIDTH {
        return start;
    }
    let left = (start.0 + dx).clamp(MIN_PLAYER_WIDTH, total - MIN_PLAYER_WIDTH);
    (left, total - left)
}

/// RL12, RL4: the shares a drag sends for the pair's new widths: each
/// one's room above its minimum, so that the next pass draws exactly these
/// widths (the model scales the pair to keep its sum). A player at its
/// minimum sends `SHARE_FLOOR` of the pair's room. `None` when the pair has
/// no room above its minimums.
pub fn pair_shares(widths: (f32, f32)) -> Option<(f32, f32)> {
    let room_of = |w: f32| {
        if w.is_finite() {
            (w - MIN_PLAYER_WIDTH).max(0.0)
        } else {
            0.0
        }
    };
    let (left, right) = (room_of(widths.0), room_of(widths.1));
    let room = left + right;
    if !(room.is_finite() && room > 0.0) {
        return None;
    }
    let floor = room * SHARE_FLOOR;
    Some((left.max(floor), right.max(floor)))
}

/// RL12: a player splitter being dragged, or just released: its widths are
/// drawn from the pointer on every frame and sent once, on release.
#[derive(Debug, Clone, PartialEq)]
pub struct LiveSplit {
    left: PlayerId,
    right: PlayerId,
    /// The shown players and the row's width the drag began with; a change
    /// in either ends it.
    shown: Vec<PlayerId>,
    row_width: f32,
    start_x: f32,
    start: (f32, f32),
    /// The pair's widths now.
    now: (f32, f32),
    /// Set on release: the shares the model had then, and until when the
    /// released widths are still drawn.
    released: Option<(Vec<(PlayerId, f32)>, f64)>,
}

impl LiveSplit {
    /// A drag of the splitter between `pair` (left, right), begun at
    /// pointer x `start_x` with the pair `start` wide.
    pub fn new(
        pair: (PlayerId, PlayerId),
        shown: Vec<PlayerId>,
        row_width: f32,
        start_x: f32,
        start: (f32, f32),
    ) -> Self {
        Self {
            left: pair.0,
            right: pair.1,
            shown,
            row_width,
            start_x,
            start,
            now: start,
            released: None,
        }
    }

    pub fn pair(&self) -> (PlayerId, PlayerId) {
        (self.left, self.right)
    }

    /// The button is still down.
    pub fn is_dragging(&self) -> bool {
        self.released.is_none()
    }

    /// The pointer is at `x`: the pair follows it (until the release).
    pub fn follow(&mut self, x: f32) {
        if self.is_dragging() {
            self.now = drag_pair(self.start, x - self.start_x);
        }
    }

    /// The button was released: the command to send, once, and only if the
    /// drag moved something. `shares` are the model's shares now, and the
    /// released widths are held until they change or until `until`.
    pub fn release(&mut self, shares: Vec<(PlayerId, f32)>, until: f64) -> Option<Command> {
        if !self.is_dragging() {
            return None;
        }
        self.released = Some((shares, until));
        if (self.now.0 - self.start.0).abs() <= 0.5 {
            return None;
        }
        let (left, right) = pair_shares(self.now)?;
        Some(Command::SetPlayerShares {
            left: (self.left, left),
            right: (self.right, right),
        })
    }

    /// Whether the drag's widths no longer apply: the shown players or the
    /// row's width changed, or, once released, the model's shares changed
    /// (its echo arrived) or the hold ran out.
    pub fn stale(
        &self,
        shown: &[PlayerId],
        row_width: f32,
        shares: &[(PlayerId, f32)],
        time: f64,
    ) -> bool {
        self.shown != shown
            || (self.row_width - row_width).abs() > 0.5
            || self
                .released
                .as_ref()
                .is_some_and(|(before, until)| time >= *until || before.as_slice() != shares)
    }

    /// `widths` with the pair's replaced by the drag's.
    pub fn apply(&self, widths: &mut [(PlayerId, f32)]) {
        for (id, width) in widths.iter_mut() {
            if *id == self.left {
                *width = self.now.0;
            } else if *id == self.right {
                *width = self.now.1;
            }
        }
    }
}

/// RL15: whether `panel` is on air, for a hidden panel's tile label
/// (`tip-layout-show-on-air`): a player whose current entry is playing or paused (as the close
/// guard counts it), or the cartwall while any cart plays.
pub fn panel_on_air(state: &AppState, panel: Panel) -> bool {
    match panel {
        Panel::Player(id) => state
            .player(id)
            .is_ok_and(|p| p.transport != Transport::Stopped),
        Panel::Cartwall => !state.cartwall.playing.is_empty(),
    }
}

/// What a panel's tile shows as its state icon (itr07 J7), shown or hidden.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelActivity {
    Idle,
    Playing,
    Paused,
}

/// A player is playing or paused; the cartwall plays while any cart does.
pub fn panel_activity(state: &AppState, panel: Panel) -> PanelActivity {
    match panel {
        Panel::Player(id) => match state.player(id).map(|p| p.transport) {
            Ok(Transport::Playing) => PanelActivity::Playing,
            Ok(Transport::Paused) => PanelActivity::Paused,
            _ => PanelActivity::Idle,
        },
        Panel::Cartwall if !state.cartwall.playing.is_empty() => PanelActivity::Playing,
        Panel::Cartwall => PanelActivity::Idle,
    }
}

/// RL16: what ends a maximised panel when it changes: the players, the
/// hidden panels, the cartwall's dock and the lock. A resize does not.
#[derive(Debug, Clone, PartialEq)]
pub struct ArrangementKey {
    players: Vec<PlayerId>,
    hidden_players: Vec<PlayerId>,
    cartwall_hidden: bool,
    dock: Dock,
    locked: bool,
}

impl ArrangementKey {
    pub fn of(state: &AppState) -> Self {
        let layout = &state.layout;
        Self {
            players: state.player_ids(),
            hidden_players: layout.hidden_players.clone(),
            cartwall_hidden: layout.cartwall.hidden,
            dock: layout.cartwall.dock,
            locked: layout.locked,
        }
    }
}

/// RL16: the panel maximised for the moment. View state only: never saved,
/// no command.
#[derive(Debug, Clone, PartialEq)]
pub struct Maximised {
    pub panel: Panel,
    key: ArrangementKey,
}

impl Maximised {
    pub fn new(panel: Panel, state: &AppState) -> Self {
        Self {
            panel,
            key: ArrangementKey::of(state),
        }
    }

    /// Whether it still holds: nothing that ends it changed since.
    pub fn holds(&self, state: &AppState) -> bool {
        self.key == ArrangementKey::of(state)
    }
}

/// RL12: the cartwall splitter being dragged, or just released: the size
/// is drawn from the pointer on every frame and sent once, on release, as
/// the cartwall's share of the middle area.
#[derive(Debug, Clone, PartialEq)]
pub struct CartwallDrag {
    dock: Dock,
    /// The middle area and the cartwall's range the drag began with; a
    /// change in either ends it.
    area: Rect,
    range: (f32, f32),
    /// The pointer where the drag began.
    origin: Pos2,
    /// The cartwall's size, in points along its axis, then and now.
    start: f32,
    now: f32,
    /// Set on release: the model's size then, and until when the released
    /// size is still drawn.
    released: Option<(Option<f32>, f64)>,
}

impl CartwallDrag {
    /// A drag of the splitter of a cartwall docked at `dock`, begun at
    /// `origin` with the cartwall `start` points along its axis. The start
    /// is taken into the range: a cartwall that fits its content can be
    /// drawn outside it (RL11), and the drag starts from the size it can
    /// keep, so a drag back to where it began sends nothing.
    pub fn new(dock: Dock, area: Rect, range: (f32, f32), origin: Pos2, start: f32) -> Self {
        let start = start.min(range.1).max(range.0);
        Self {
            dock,
            area,
            range,
            origin,
            start,
            now: start,
            released: None,
        }
    }

    /// The button is still down.
    pub fn is_dragging(&self) -> bool {
        self.released.is_none()
    }

    /// The pointer is at `pointer`: the size follows it (until the
    /// release), within the range. Dragging up grows a bottom cartwall,
    /// dragging right a left one, dragging left a right one.
    pub fn follow(&mut self, pointer: Pos2) {
        if !self.is_dragging() {
            return;
        }
        let moved = pointer - self.origin;
        let grow = match self.dock {
            Dock::Bottom => -moved.y,
            Dock::Left => moved.x,
            Dock::Right => -moved.x,
        };
        if grow.is_finite() {
            self.now = (self.start + grow).min(self.range.1).max(self.range.0);
        }
    }

    /// The size now, in points along the cartwall's axis.
    pub fn size(&self) -> f32 {
        self.now
    }

    /// The size now as the cartwall's share of the middle area: what the
    /// pass draws while the drag lives, and what the release sends.
    pub fn share(&self) -> f32 {
        let extent = if self.dock.is_side() {
            self.area.width()
        } else {
            self.area.height()
        };
        if extent > 0.0 {
            // The bounds are constants, so `clamp` cannot panic; `now` is
            // always a number (`follow` keeps it so).
            (self.now / extent).clamp(MIN_CARTWALL_SHARE, MAX_CARTWALL_SHARE)
        } else {
            MIN_CARTWALL_SHARE
        }
    }

    /// The button was released: the command to send, once, and only if
    /// the drag moved something. `size` is the model's size now; the
    /// released size is held until it changes or until `until`.
    pub fn release(&mut self, size: Option<f32>, until: f64) -> Option<Command> {
        if !self.is_dragging() {
            return None;
        }
        self.released = Some((size, until));
        ((self.now - self.start).abs() > 0.5).then(|| Command::SetCartwallSize(Some(self.share())))
    }

    /// Whether the drag no longer applies: the dock, the area or the range
    /// changed (or there is no range: hidden or collapsed), or, once
    /// released, the model's size changed (its echo arrived) or the hold
    /// ran out.
    pub fn stale(
        &self,
        dock: Dock,
        area: Rect,
        range: Option<(f32, f32)>,
        size: Option<f32>,
        time: f64,
    ) -> bool {
        let moved = |a: Pos2, b: Pos2| (a - b).length() > 0.5;
        self.dock != dock
            || moved(self.area.min, area.min)
            || moved(self.area.max, area.max)
            || range.is_none_or(|(least, most)| {
                (least - self.range.0).abs() > 0.5 || (most - self.range.1).abs() > 0.5
            })
            || self
                .released
                .is_some_and(|(before, until)| time >= until || before != size)
    }
}
